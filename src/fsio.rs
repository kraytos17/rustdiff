use memmap2::Mmap;
use std::fs::File;
use std::io::{self, Read};

/// A file's contents, either read into memory or memory-mapped.
///
/// Large files are mapped rather than copied into a `String` to avoid a
/// whole-file read + copy; small files are read normally.
pub enum Source {
    Small(String),
    Mapped(Mmap),
}

/// Files at or above this size (in bytes) are memory-mapped.
const MMAP_THRESHOLD: u64 = 1 << 20;

impl Source {
    /// The file contents as a UTF-8 string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the mapped bytes are not valid UTF-8.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        match self {
            Self::Small(s) => Ok(s.as_str()),
            Self::Mapped(m) => std::str::from_utf8(&m[..]),
        }
    }
}

/// Read a file's contents, memory-mapping files at or above the mmap threshold
/// (1 MiB).
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read, or if a small file is
/// not valid UTF-8. (Mapped files validate UTF-8 lazily via [`Source::as_str`].)
pub fn read_file(path: &str) -> io::Result<Source> {
    let mut file = File::open(path)?;
    if file.metadata()?.len() >= MMAP_THRESHOLD {
        // SAFETY: the mapping is read-only and we never modify the file. As with
        // any mmap, a concurrent external writer could fault the process (SIGBUS),
        // which is the standard tradeoff for a one-shot CLI tool.
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Source::Mapped(mmap))
    } else {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Source::Small(contents))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_file_round_trip() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("rustdiff_fsio_{}", std::process::id()));
        std::fs::write(&path, b"hello\nworld\n").unwrap();

        let source = read_file(path.to_str().unwrap()).unwrap();
        assert!(matches!(source, Source::Small(_)));
        assert_eq!(source.as_str().unwrap(), "hello\nworld\n");

        let _ = std::fs::remove_file(path);
    }
}
