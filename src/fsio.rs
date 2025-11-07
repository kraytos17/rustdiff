use std::fs::File;
use std::io::{BufRead, BufReader, Result};

pub fn read_lines(path: &str) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}
