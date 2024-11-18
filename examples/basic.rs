use anyhow::Result;
use congkit::{CongkitDB, CongkitFilter, CongkitVersion};
use std::fs;

fn main() -> Result<()> {
    // let dat = include_bytes!("../data/full_table.dat");
    let txt = fs::read_to_string("data/table.txt")?;
    let db = CongkitDB::from_txt(&txt, CongkitVersion::V3, CongkitFilter::chinese());
    println!("{:?}", db.get_radicals("hqi rgpd gi rkm ehbk ilil"));
    println!("{:?}", db.get_code(&"寫".chars().next().unwrap()));
    println!(
        "{:?}",
        db.get_codes(vec![
            "我".chars().next().unwrap(),
            "你".chars().next().unwrap(),
            "佢".chars().next().unwrap()
        ])
    );
    println!("{:?}", db.get_characters("*hqi"));
    println!(
        "{:?}",
        db.get_chars_mult(vec!["onf*".to_string(), "jh*f".to_string()])
    );
    Ok(())
}
