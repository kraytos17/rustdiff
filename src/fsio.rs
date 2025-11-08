use std::fs::File;
use std::io::{Read, Result};

/// Read the entire file contents into a single UTF-8 string.
///
/// Useful for word-level or character-level diffs.
pub fn read_file(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
