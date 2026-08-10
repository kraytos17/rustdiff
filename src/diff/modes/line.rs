use crate::diff::core::{compute_histogram_diff, myers::compute_diff};
use crate::diff::data::Diff;
use crate::diff::modes::DiffAlgorithm;
use memchr::memchr_iter;

pub fn diff_lines(old: &str, new: &str, algorithm: DiffAlgorithm) -> Diff {
    let old_lines = split_and_trim_lines(old);
    let new_lines = split_and_trim_lines(new);
    let old_refs: Vec<&str> = old_lines.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_lines.iter().map(String::as_str).collect();

    let ops = match algorithm {
        DiffAlgorithm::Histogram => compute_histogram_diff(&old_refs, &new_refs),
        DiffAlgorithm::Myers => compute_diff(&old_refs, &new_refs),
    };

    Diff {
        ops,
        old_tokens: old_lines,
        new_tokens: new_lines,
    }
}

/// Split on `\n` (replicating `str::lines()` semantics: a trailing `\r` is
/// trimmed from each line, and a trailing newline does not add an empty line).
/// Uses `memchr` for a guaranteed SIMD newline scan on large inputs.
fn split_and_trim_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut start = 0;
    for idx in memchr_iter(b'\n', text.as_bytes()) {
        lines.push(text[start..idx].trim_end_matches('\r').to_string());
        start = idx + 1;
    }
    if start < text.len() {
        lines.push(text[start..].trim_end_matches('\r').to_string());
    }
    lines
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
