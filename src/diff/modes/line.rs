use crate::diff::core::{compute_histogram_diff, myers::compute_diff};
use crate::diff::data::{Diff, ensure_within_u32};
use crate::diff::modes::DiffAlgorithm;

/// Compute a line-level diff.
///
/// # Errors
///
/// Returns a `String` error if either input has more than `MAX_TOKENS` lines,
/// which the `u32`-indexed core cannot address.
pub fn diff_lines(old: &str, new: &str, algorithm: DiffAlgorithm) -> Result<Diff, String> {
    let old_lines = split_and_trim_lines(old);
    let new_lines = split_and_trim_lines(new);

    ensure_within_u32(old_lines.len(), "lines")?;
    ensure_within_u32(new_lines.len(), "lines")?;

    let old_refs: Vec<&str> = old_lines.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_lines.iter().map(String::as_str).collect();
    let ops = match algorithm {
        DiffAlgorithm::Histogram => compute_histogram_diff(&old_refs, &new_refs),
        DiffAlgorithm::Myers => compute_diff(&old_refs, &new_refs),
    };

    Ok(Diff {
        ops,
        old_tokens: old_lines,
        new_tokens: new_lines,
    })
}

/// Split on `\n` via [`str::lines`] semantics: a trailing `\r` is trimmed from
/// each line ending (CRLF), and a trailing newline does not add an empty line.
/// `str::lines` is itself memchr-accelerated, so large inputs still get a fast
/// newline scan.
fn split_and_trim_lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_basic() {
        assert_eq!(split_and_trim_lines("a\nb\n"), vec!["a", "b"]);
    }

    #[test]
    fn test_split_crlf() {
        assert_eq!(split_and_trim_lines("a\r\nb\r\n"), vec!["a", "b"]);
    }

    #[test]
    fn test_split_empty() {
        assert!(split_and_trim_lines("").is_empty());
    }

    #[test]
    fn test_split_no_trailing_newline() {
        assert_eq!(split_and_trim_lines("a\nb"), vec!["a", "b"]);
    }

    #[test]
    fn test_split_empty_lines() {
        assert_eq!(split_and_trim_lines("a\n\nb\n"), vec!["a", "", "b"]);
    }
}
