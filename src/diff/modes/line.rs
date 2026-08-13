use crate::diff::core::{compute_histogram_diff, myers::compute_diff};
use crate::diff::data::{Diff, ensure_within_u32};
use crate::diff::modes::{DiffAlgorithm, DiffOptions, keys_for};

/// Compute a line-level diff.
///
/// # Errors
///
/// Returns a `String` error if either input has more than `MAX_TOKENS` lines,
/// which the `u32`-indexed core cannot address.
pub fn diff_lines(old: &str, new: &str, algorithm: DiffAlgorithm) -> Result<Diff, String> {
    diff_lines_with(old, new, algorithm, DiffOptions::default())
}

/// Compute a line-level diff with [`DiffOptions`] normalization.
///
/// # Errors
///
/// Returns a `String` error if either input has more than `MAX_TOKENS` lines,
/// which the `u32`-indexed core cannot address.
pub fn diff_lines_with(
    old: &str,
    new: &str,
    algorithm: DiffAlgorithm,
    opts: DiffOptions,
) -> Result<Diff, String> {
    let old_lines = split_and_trim_lines(old);
    let new_lines = split_and_trim_lines(new);

    ensure_within_u32(old_lines.len(), "lines")?;
    ensure_within_u32(new_lines.len(), "lines")?;

    let old_keys = keys_for(&old_lines, opts);
    let new_keys = keys_for(&new_lines, opts);
    let old_refs: Vec<&str> = old_keys.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_keys.iter().map(String::as_str).collect();
    let diff_ops = match algorithm {
        DiffAlgorithm::Histogram => compute_histogram_diff(&old_refs, &new_refs),
        DiffAlgorithm::Myers => compute_diff(&old_refs, &new_refs),
    };

    Ok(Diff {
        ops: diff_ops,
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
    use crate::diff::data::OpKind;

    fn opts(ignore_whitespace: bool, ignore_case: bool) -> DiffOptions {
        DiffOptions {
            ignore_whitespace,
            ignore_case,
        }
    }

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

    #[test]
    fn test_ignore_whitespace_makes_whitespace_diffs_invisible() {
        let diff = diff_lines_with(
            "fn  foo ( x )\n",
            "fn foo(x)\n",
            DiffAlgorithm::Histogram,
            opts(true, false),
        )
        .unwrap();
        assert!(
            diff.ops.iter().all(|op| op.kind == OpKind::Equal),
            "ops: {diff:?}"
        );
    }

    #[test]
    fn test_ignore_case_makes_case_diffs_invisible() {
        let diff = diff_lines_with(
            "Hello World\n",
            "hello world\n",
            DiffAlgorithm::Histogram,
            opts(false, true),
        )
        .unwrap();
        assert!(
            diff.ops.iter().all(|op| op.kind == OpKind::Equal),
            "ops: {diff:?}"
        );
    }

    #[test]
    fn test_normalization_composes() {
        let diff = diff_lines_with(
            "Fn  Foo\n",
            "fn foo\n",
            DiffAlgorithm::Histogram,
            opts(true, true),
        )
        .unwrap();
        assert!(
            diff.ops.iter().all(|op| op.kind == OpKind::Equal),
            "ops: {diff:?}"
        );
    }

    #[test]
    fn test_normalization_keeps_rendered_text_original() {
        let diff = diff_lines_with(
            "HELLO  WORLD\n",
            "hello world\n",
            DiffAlgorithm::Histogram,
            opts(true, true),
        )
        .unwrap();
        assert_eq!(diff.old_tokens, vec!["HELLO  WORLD".to_string()]);
        assert_eq!(diff.new_tokens, vec!["hello world".to_string()]);
    }
}
