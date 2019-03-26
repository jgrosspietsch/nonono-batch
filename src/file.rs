extern crate nonogram;
extern crate serde_json;

use nonogram::Nonogram;
use std::error::Error;
use std::fs;
use std::io::prelude::*;

pub fn write_to_file(puzzles: &[Nonogram], path: &str) -> Result<(), Box<Error>> {
    let json = serde_json::to_string(&puzzles)?;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}
