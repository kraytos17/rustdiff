use crate::diff::core::compute_histogram_diff_limited;
use crate::diff::core::myers::compute_diff_limited;
use crate::diff::data::{Diff, Op, OpKind, ensure_within_u32};
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
    let mut diff_ops = match algorithm {
        DiffAlgorithm::Histogram => {
            compute_histogram_diff_limited(&old_refs, &new_refs, opts.max_edit_distance)
        }
        DiffAlgorithm::Myers => compute_diff_limited(&old_refs, &new_refs, opts.max_edit_distance),
    };
    if opts.ignore_blank_lines {
        drop_blank_only_runs(&mut diff_ops, &old_lines, &new_lines);
    }

    Ok(Diff {
        ops: diff_ops,
        old_tokens: old_lines,
        new_tokens: new_lines,
    })
}

/// Remove insert/delete runs consisting solely of blank lines, so blank-line
/// changes are invisible (like `diff -B`). Equal runs are never touched.
fn drop_blank_only_runs(ops: &mut Vec<Op>, old_tokens: &[String], new_tokens: &[String]) {
    ops.retain(|op| {
        if op.kind == OpKind::Equal {
            return true;
        }
        let tokens = match op.kind {
            OpKind::Insert => new_tokens,
            _ => old_tokens,
        };
        let start = op.start as usize;
        let all_blank = tokens[start..start + op.len as usize]
            .iter()
            .all(|t| t.trim().is_empty());
        !all_blank
    });
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
            ignore_blank_lines: false,
            max_edit_distance: None,
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

    #[test]
    fn test_ignore_blank_lines_makes_blank_diffs_invisible() {
        let opts = DiffOptions {
            ignore_whitespace: false,
            ignore_case: false,
            ignore_blank_lines: true,
            max_edit_distance: None,
        };

        let diff =
            diff_lines_with("a\n\nb\n", "a\n\n\nb\n", DiffAlgorithm::Histogram, opts).unwrap();
        assert!(
            diff.ops.iter().all(|op| op.kind == OpKind::Equal),
            "blank-line-only change must be invisible: {diff:?}"
        );
    }

    #[test]
    fn test_ignore_blank_lines_keeps_real_changes() {
        let opts = DiffOptions {
            ignore_whitespace: false,
            ignore_case: false,
            ignore_blank_lines: true,
            max_edit_distance: None,
        };

        let diff =
            diff_lines_with("a\nb\n", "a\nCHANGED\n", DiffAlgorithm::Histogram, opts).unwrap();
        assert!(
            diff.ops.iter().any(|op| op.kind == OpKind::Delete),
            "real change must survive"
        );
    }
}
