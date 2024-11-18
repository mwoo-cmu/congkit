use anyhow::Result;
use congkit::{CongkitDB, CongkitFilter};
use std::{fs, io::Write};

fn main() -> Result<()> {
    let txt = fs::read_to_string("data/table.txt")?;
    let entries = CongkitDB::to_entries(&txt, &CongkitFilter::all());
    println!("{}", entries.len());
    let mut full = fs::File::create("data/full_table.dat")?;
    full.write_all(&bitcode::encode(&entries))?;
    let trimmed = CongkitDB::to_entries(
        &txt,
        &CongkitFilter {
            chinese: false,
            big5: true,
            hkscs: true,
            taiwanese: false,
            ..Default::default()
        },
    );
    println!("{}", trimmed.len());
    let mut trim = fs::File::create("data/trimmed_table.dat")?;
    trim.write_all(&bitcode::encode(&trimmed))?;
    Ok(())
}
