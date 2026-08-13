use crate::diff::core::{compute_histogram_diff, myers::compute_diff};
use crate::diff::data::{Diff, ensure_within_u32};
use crate::diff::modes::DiffAlgorithm;
use regex::Regex;
use std::sync::LazyLock;

static WORD_TOKEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\[-.*?\+.*?\]|[^\s]+\s*|\n)").unwrap());

/// Compute a word-level diff.
///
/// # Errors
///
/// Returns a `String` error if either input has more than `MAX_TOKENS` tokens,
/// which the `u32`-indexed core cannot address.
pub fn diff_words(
    old_text: &str,
    new_text: &str,
    algorithm: DiffAlgorithm,
) -> Result<Diff, String> {
    let old_tokens = tokenize(&old_text.replace("\r\n", "\n"));
    let new_tokens = tokenize(&new_text.replace("\r\n", "\n"));

    ensure_within_u32(old_tokens.len(), "tokens")?;
    ensure_within_u32(new_tokens.len(), "tokens")?;

    let old_refs: Vec<&str> = old_tokens.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_tokens.iter().map(String::as_str).collect();
    let ops = match algorithm {
        DiffAlgorithm::Histogram => compute_histogram_diff(&old_refs, &new_refs),
        DiffAlgorithm::Myers => compute_diff(&old_refs, &new_refs),
    };

    Ok(Diff {
        ops,
        old_tokens,
        new_tokens,
    })
}

fn tokenize(text: &str) -> Vec<String> {
    WORD_TOKEN_RE
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic_words() {
        let tokens = tokenize("hello world");
        assert_eq!(tokens, vec!["hello ", "world"]);
    }

    #[test]
    fn test_tokenize_marker_token() {
        let tokens = tokenize("[-old+new]");
        assert_eq!(tokens, vec!["[-old+new]"]);
    }

    #[test]
    fn test_tokenize_newline() {
        let tokens = tokenize("a\nb\n");
        assert_eq!(tokens, vec!["a\n", "b\n"]);
    }

    #[test]
    fn test_tokenize_marker_with_text() {
        let tokens = tokenize("foo [-a+b] bar");
        assert_eq!(tokens, vec!["foo ", "[-a+b]", "bar"]);
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    fn round_trip(old: &str, new: &str, algorithm: DiffAlgorithm) {
        let diff = diff_words(old, new, algorithm).unwrap();
        let old_tokens = tokenize(&old.replace("\r\n", "\n"));
        let new_tokens = tokenize(&new.replace("\r\n", "\n"));
        let old_refs: Vec<&str> = old_tokens.iter().map(String::as_str).collect();
        let new_refs: Vec<&str> = new_tokens.iter().map(String::as_str).collect();
        assert!(
            diff.validate_round_trip(&old_refs, &new_refs),
            "round-trip failed"
        );
    }

    #[test]
    fn test_diff_words_round_trip() {
        round_trip("hello world", "hello rust", DiffAlgorithm::Histogram);
        round_trip("hello world", "hello rust", DiffAlgorithm::Myers);
    }

    #[test]
    fn test_diff_words_crlf_normalized() {
        let diff = diff_words(
            "hello\r\nworld\r\n",
            "hello\r\nrust\r\n",
            DiffAlgorithm::Histogram,
        )
        .unwrap();

        for (_, text) in diff.edits() {
            assert!(!text.contains('\r'), "CR leaked into diff op: {text:?}");
        }
    }

    #[test]
    fn test_diff_words_runs_are_small() {
        // A single-word change should produce a small number of runs.
        let diff = diff_words(
            "hello world foo",
            "hello rust foo",
            DiffAlgorithm::Histogram,
        )
        .unwrap();

        assert!(
            diff.ops.len() <= 8,
            "expected few runs, got {}: {diff:?}",
            diff.ops.len()
        );
    }
}
