use crate::diff::data::{DiffOp, Hunk};
use std::fmt::Write;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

/// Render a unified diff similar to `git diff --unified`, with optional ANSI colors.
///
/// Includes hunk headers (`@@ -a,b +c,d @@`) and configurable context lines.
pub fn render_unified_diff(
    old_name: &str,
    new_name: &str,
    diffs: &[DiffOp],
    context: usize,
    color: bool,
) -> String {
    let mut out = String::new();
    if color {
        writeln!(out, "{GRAY}--- {old_name}{RESET}").unwrap();
        writeln!(out, "{GRAY}+++ {new_name}{RESET}").unwrap();
    } else {
        writeln!(out, "--- {old_name}").unwrap();
        writeln!(out, "+++ {new_name}").unwrap();
    }

    let hunks = group_into_hunks(diffs, context);
    for hunk in hunks {
        if color {
            writeln!(
                out,
                "{CYAN}@@ -{},{} +{},{} @@{RESET}",
                hunk.start_a, hunk.len_a, hunk.start_b, hunk.len_b
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "@@ -{},{} +{},{} @@",
                hunk.start_a, hunk.len_a, hunk.start_b, hunk.len_b
            )
            .unwrap();
        }

        for op in &hunk.ops {
            match op {
                DiffOp::Equal(line) => {
                    writeln!(out, " {line}").unwrap();
                }
                DiffOp::Insert(line) => {
                    if color {
                        writeln!(out, "{GREEN}+{line}{RESET}").unwrap();
                    } else {
                        writeln!(out, "+{line}").unwrap();
                    }
                }
                DiffOp::Delete(line) => {
                    if color {
                        writeln!(out, "{RED}-{line}{RESET}").unwrap();
                    } else {
                        writeln!(out, "-{line}").unwrap();
                    }
                }
            }
        }
    }

    out
}

/// Group the raw [`DiffOp`]s into hunks with context lines.
fn group_into_hunks(diffs: &[DiffOp], context: usize) -> Vec<Hunk> {
    let mut hunks = Vec::new();
    let mut idx = 0;
    let mut old_line = 1;
    let mut new_line = 1;

    while idx < diffs.len() {
        let mut context_start_idx = idx;
        let mut context_start_a = old_line;
        let mut context_start_b = new_line;

        while let Some(op) = diffs.get(idx) {
            if !matches!(op, DiffOp::Equal(_)) {
                break;
            }

            if idx - context_start_idx >= context {
                context_start_a += 1;
                context_start_b += 1;
                context_start_idx += 1;
            }

            old_line += 1;
            new_line += 1;
            idx += 1;
        }

        if idx >= diffs.len() {
            break;
        }

        let mut hunk_ops: Vec<DiffOp> = diffs[context_start_idx..idx].to_vec();
        let hunk_start_a = context_start_a;
        let hunk_start_b = context_start_b;
        let mut trailing_context_count = 0;

        while let Some(op) = diffs.get(idx) {
            match op {
                DiffOp::Insert(_) => {
                    hunk_ops.push(op.clone());
                    new_line += 1;
                    trailing_context_count = 0;
                }
                DiffOp::Delete(_) => {
                    hunk_ops.push(op.clone());
                    old_line += 1;
                    trailing_context_count = 0;
                }
                DiffOp::Equal(_) => {
                    if trailing_context_count >= context {
                        break;
                    }

                    hunk_ops.push(op.clone());
                    old_line += 1;
                    new_line += 1;
                    trailing_context_count += 1;
                }
            }
            idx += 1;
        }

        let len_a = hunk_ops
            .iter()
            .filter(|op| !matches!(op, DiffOp::Insert(_)))
            .count();
        let len_b = hunk_ops
            .iter()
            .filter(|op| !matches!(op, DiffOp::Delete(_)))
            .count();

        hunks.push(Hunk {
            start_a: hunk_start_a,
            start_b: hunk_start_b,
            len_a,
            len_b,
            ops: hunk_ops,
        });
    }

    hunks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(text: &str) -> DiffOp {
        DiffOp::Equal(text.to_string())
    }
    fn i(text: &str) -> DiffOp {
        DiffOp::Insert(text.to_string())
    }
    fn d(text: &str) -> DiffOp {
        DiffOp::Delete(text.to_string())
    }

    #[test]
    fn test_group_into_hunks_all_equal() {
        let diffs = vec![e("a"), e("b"), e("c")];
        let hunks = group_into_hunks(&diffs, 3);
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_group_into_hunks_single_change() {
        let diffs = vec![e("a"), e("b"), d("x"), i("y"), e("c"), e("d")];
        let hunks = group_into_hunks(&diffs, 1);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].start_a, 2);
        assert_eq!(hunks[0].start_b, 2);
    }

    #[test]
    fn test_group_into_hunks_two_distant_changes() {
        let diffs = vec![
            e("a"),
            d("x"),
            i("y"),
            e("b"),
            e("c"),
            e("d"),
            e("e"),
            d("p"),
            i("q"),
            e("f"),
        ];
        let hunks = group_into_hunks(&diffs, 1);
        assert_eq!(hunks.len(), 2);
    }

    #[test]
    fn test_group_into_hunks_context_boundary() {
        let diffs = vec![
            e("a"),
            e("b"),
            e("c"),
            e("d"),
            e("e"),
            d("x"),
            i("y"),
            e("f"),
            e("g"),
            e("h"),
        ];
        let hunks = group_into_hunks(&diffs, 2);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].ops.len(), 6);
    }

    #[test]
    fn test_group_into_hunks_change_at_start() {
        let diffs = vec![d("a"), i("b"), e("c"), e("d")];
        let hunks = group_into_hunks(&diffs, 2);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].start_a, 1);
        assert_eq!(hunks[0].start_b, 1);
    }

    #[test]
    fn test_group_into_hunks_change_at_end() {
        let diffs = vec![e("a"), e("b"), d("c"), i("d")];
        let hunks = group_into_hunks(&diffs, 2);
        assert_eq!(hunks.len(), 1);
    }

    // --- render_unified_diff ---

    #[test]
    fn test_render_unified_diff_empty() {
        let result = render_unified_diff("old", "new", &[], 3, false);
        assert_eq!(result, "--- old\n+++ new\n");
    }

    #[test]
    fn test_render_unified_diff_basic() {
        let diffs = vec![e("a"), d("x"), i("y"), e("b")];
        let result = render_unified_diff("f1", "f2", &diffs, 0, false);
        assert!(result.starts_with("--- f1\n+++ f2\n"));
        assert!(result.contains("@@ -2,1 +2,1 @@"));
        assert!(result.contains("-x"));
        assert!(result.contains("+y"));
    }

    #[test]
    fn test_render_unified_diff_compact_mode() {
        let diffs = vec![e("ctx"), d("old"), i("new"), e("trail")];
        let result = render_unified_diff("o", "n", &diffs, 0, false);
        assert!(!result.contains("ctx"));
        assert!(result.contains("-old"));
        assert!(result.contains("+new"));
    }

    #[test]
    fn test_render_unified_diff_color() {
        let diffs = vec![d("red"), i("green")];
        let result = render_unified_diff("o", "n", &diffs, 0, true);
        assert!(result.contains("\x1b[31m"), "missing red");
        assert!(result.contains("\x1b[32m"), "missing green");
        assert!(result.contains("\x1b[36m"), "missing cyan hunk header");
        assert!(result.contains("\x1b[90m"), "missing gray file header");
    }
}
