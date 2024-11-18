use anyhow::Result;
use bimap::BiMap;
use bitcode::{Decode, Encode};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
pub enum CongkitVersion {
    V3,
    V5,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CongkitFilter {
    pub chinese: bool,
    pub big5: bool,
    pub hkscs: bool,
    pub taiwanese: bool,
    pub kanji: bool,
    pub hiragana: bool,
    pub katakana: bool,
    pub punctuation: bool,
    pub misc: bool,
}

impl Default for CongkitFilter {
    fn default() -> Self {
        Self::chinese()
    }
}

impl CongkitFilter {
    pub fn all() -> Self {
        Self {
            chinese: true,
            big5: true,
            hkscs: true,
            taiwanese: true,
            kanji: true,
            hiragana: true,
            katakana: true,
            punctuation: true,
            misc: true,
        }
    }

    pub fn chinese() -> Self {
        Self {
            chinese: true,
            big5: true,
            hkscs: true,
            taiwanese: true,
            kanji: false,
            hiragana: false,
            katakana: false,
            punctuation: false,
            misc: false,
        }
    }

    pub fn japanese() -> Self {
        Self {
            chinese: false,
            big5: false,
            hkscs: false,
            taiwanese: false,
            kanji: true,
            hiragana: true,
            katakana: true,
            punctuation: false,
            misc: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Encode, Decode, PartialEq)]
pub struct Entry {
    traditional: char,
    simplified: char,
    chinese: bool,
    big5: bool,
    hkscs: bool,
    taiwanese: bool,
    kanji: bool,
    hiragana: bool,
    katakana: bool,
    punctuation: bool,
    misc: bool,
    v3: String,
    v5: String,
    code: String,
    shortcut: String,
    order: i32,
}

// #[derive(Debug, Deserialize, Serialize, Encode, Decode, PartialEq)]
// pub struct EntryTrimmed {
//     traditional: char,
//     chinese: bool,
//     big5: bool,
//     hkscs: bool,
//     v3: String,
//     v5: String,
//     shortcut: String,
//     order: i32,
// }

#[derive(Debug)]
pub struct CongkitDB {
    entries: HashMap<char, Entry>,
    version: CongkitVersion,
    radicals: BiMap<char, char>,
}

impl Default for CongkitDB {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            version: CongkitVersion::V3,
            radicals: BiMap::from_iter([
                ('日', 'a'),
                ('月', 'b'),
                ('金', 'c'),
                ('木', 'd'),
                ('水', 'e'),
                ('火', 'f'),
                ('土', 'g'),
                ('竹', 'h'),
                ('戈', 'i'),
                ('十', 'j'),
                ('大', 'k'),
                ('中', 'l'),
                ('一', 'm'),
                ('弓', 'n'),
                ('人', 'o'),
                ('心', 'p'),
                ('手', 'q'),
                ('口', 'r'),
                ('尸', 's'),
                ('廿', 't'),
                ('山', 'u'),
                ('女', 'v'),
                ('田', 'w'),
                ('難', 'x'),
                ('卜', 'y'),
            ]),
        }
    }
}

impl CongkitDB {
    pub fn get_radical(&self, key: &char) -> Option<char> {
        self.radicals.get_by_right(key).copied()
    }

    pub fn get_key(&self, radical: &char) -> Option<char> {
        self.radicals.get_by_left(radical).copied()
    }

    pub fn get_radicals(&self, code: &str) -> String {
        code.chars()
            .map(|c| match self.get_radical(&c) {
                Some(ch) => ch,
                None => c,
            })
            .collect::<String>()
    }

    pub fn get_code(&self, character: &char) -> Option<String> {
        Some(self.entries.get(character)?.code.clone())
    }

    pub fn get_codes(&self, chars: Vec<char>) -> Vec<Option<String>> {
        chars
            .iter()
            .map(|c| self.get_code(c))
            .collect::<Vec<Option<String>>>()
    }

    pub fn get_characters(&self, code: &str) -> Result<Vec<char>> {
        let re = Regex::new(&format!("^{}$", code.replace('*', ".+")))?;
        let mut filt = self
            .entries
            .values()
            .filter(|entry| re.is_match(&entry.code))
            .collect::<Vec<&Entry>>();
        filt.sort_by(|a, b| a.order.cmp(&b.order));
        Ok(filt
            .iter()
            .map(|entry| entry.traditional)
            .collect::<Vec<char>>())
    }

    pub fn get_chars_mult(&self, codes: Vec<String>) -> Result<HashMap<String, Vec<char>>> {
        let mut chars: HashMap<String, Vec<&Entry>> = HashMap::new();
        let mut regexes: HashMap<String, Regex> = HashMap::new();
        for c in codes.into_iter() {
            chars.insert(c.clone(), Vec::new());
            regexes.insert(
                c.clone(),
                Regex::new(&format!("^{}$", c.replace('*', ".+")))?,
            );
        }
        for ent in self.entries.values() {
            for (code, re) in regexes.iter() {
                if re.is_match(&ent.code) {
                    chars.get_mut(code).unwrap().push(ent);
                }
            }
        }
        for matches in chars.values_mut() {
            matches.sort_by(|a, b| a.order.cmp(&b.order));
        }
        Ok(chars
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.iter().map(|ent| ent.traditional).collect::<Vec<char>>(),
                )
            })
            .collect::<HashMap<String, Vec<char>>>())
    }

    fn from_entry_vec(entry_vec: Vec<Entry>, version: CongkitVersion) -> Self {
        let entries = entry_vec
            .into_iter()
            .map(|mut entry| {
                entry.code = match version {
                    CongkitVersion::V3 => entry.v3.clone(),
                    CongkitVersion::V5 => entry.v5.clone(),
                };
                (entry.traditional, entry)
            })
            .collect::<HashMap<char, Entry>>();
        Self {
            entries,
            version,
            ..Default::default()
        }
    }

    fn apply_filters(entry: &Entry, filter: &CongkitFilter) -> bool {
        (entry.chinese && filter.chinese)
            || (entry.big5 && filter.big5)
            || (entry.hkscs && filter.hkscs)
            || (entry.taiwanese && filter.taiwanese)
            || (entry.kanji && filter.kanji)
            || (entry.hiragana && filter.hiragana)
            || (entry.katakana && filter.katakana)
            || (entry.punctuation && filter.punctuation)
            || (entry.misc && filter.misc)
    }

    pub fn from_data(data: &[u8], version: CongkitVersion, filter: CongkitFilter) -> Result<Self> {
        let entries_vec: Vec<Entry> = bitcode::decode(data)?;
        let entries = entries_vec
            .into_iter()
            .filter(|entry| Self::apply_filters(entry, &filter))
            .collect::<Vec<Entry>>();
        Ok(Self::from_entry_vec(entries, version))
    }

    pub fn to_entries(txt: &str, filter: &CongkitFilter) -> Vec<Entry> {
        txt.split('\n')
            .filter(|line| !(line.starts_with("# ") || line.is_empty()))
            .map(|line| {
                let fields = line.split(' ').collect::<Vec<&str>>();
                Entry {
                    traditional: fields.first().unwrap().chars().nth(0).unwrap(),
                    simplified: fields.get(1).unwrap().chars().nth(0).unwrap(),
                    chinese: *fields.get(2).unwrap() == "1",
                    big5: *fields.get(3).unwrap() == "1",
                    hkscs: *fields.get(4).unwrap() == "1",
                    taiwanese: *fields.get(5).unwrap() == "1",
                    kanji: *fields.get(6).unwrap() == "1",
                    hiragana: *fields.get(7).unwrap() == "1",
                    katakana: *fields.get(8).unwrap() == "1",
                    punctuation: *fields.get(9).unwrap() == "1",
                    misc: *fields.get(10).unwrap() == "1",
                    v3: fields.get(11).unwrap().to_string(),
                    v5: fields.get(12).unwrap().to_string(),
                    code: "".to_string(),
                    shortcut: fields.get(13).unwrap().to_string(),
                    order: fields.get(14).unwrap().parse().unwrap(),
                }
            })
            .filter(|entry| Self::apply_filters(entry, filter))
            .collect::<Vec<Entry>>()
    }

    pub fn from_txt(txt: &str, version: CongkitVersion, filter: CongkitFilter) -> Self {
        let entries = Self::to_entries(txt, &filter);
        Self::from_entry_vec(entries, version)
    }

    // pub fn new(version: CongkitVersion, filter: CongkitFilter) -> Self {
    //     // filters should be applied to entries here, and then the hashmaps build from there.
    //     Self {
    //         ..Default::default()
    //     }
    // }
}
