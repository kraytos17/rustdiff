use crate::diff::data::{Diff, Hunk, Op, OpKind, u32_len};
use std::fmt::Write;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

/// Render a unified diff similar to `git diff --unified`, with optional ANSI colors.
///
/// Includes hunk headers (`@@ -a,b +c,d @@`) and configurable context lines.
#[must_use]
pub fn render_unified_diff(
    old_name: &str,
    new_name: &str,
    diff: &Diff,
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

    let hunks = group_into_hunks(&diff.ops, context);
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
            let tokens = match op.kind {
                OpKind::Equal | OpKind::Delete => &diff.old_tokens,
                OpKind::Insert => &diff.new_tokens,
            };

            let start = op.start as usize;
            for line in &tokens[start..start + op.len as usize] {
                match op.kind {
                    OpKind::Equal => {
                        writeln!(out, " {line}").unwrap();
                    }
                    OpKind::Insert => {
                        if color {
                            writeln!(out, "{GREEN}+{line}{RESET}").unwrap();
                        } else {
                            writeln!(out, "+{line}").unwrap();
                        }
                    }
                    OpKind::Delete => {
                        if color {
                            writeln!(out, "{RED}-{line}{RESET}").unwrap();
                        } else {
                            writeln!(out, "-{line}").unwrap();
                        }
                    }
                }
            }
        }
    }

    out
}

/// Group ops into hunks with context lines.
///
/// Runs are expanded to per-line `(kind, a_pos, b_pos)` entries so context
/// windowing counts lines (an Equal run of `len` is `len` context lines), then
/// each hunk's selected lines are re-encoded back into contiguous runs.
#[allow(clippy::too_many_lines)]
fn group_into_hunks(ops: &[Op], context: usize) -> Vec<Hunk> {
    let mut lines: Vec<(OpKind, usize, usize)> = Vec::new();
    let mut a_pos = 0usize;
    let mut b_pos = 0usize;
    for op in ops {
        match op.kind {
            OpKind::Equal => {
                for _ in 0..op.len {
                    lines.push((OpKind::Equal, a_pos, b_pos));
                    a_pos += 1;
                    b_pos += 1;
                }
            }
            OpKind::Delete => {
                for _ in 0..op.len {
                    lines.push((OpKind::Delete, a_pos, b_pos));
                    a_pos += 1;
                }
            }
            OpKind::Insert => {
                for _ in 0..op.len {
                    lines.push((OpKind::Insert, a_pos, b_pos));
                    b_pos += 1;
                }
            }
        }
    }

    let mut hunks = Vec::new();
    let mut idx = 0usize;
    let mut old_line = 1usize;
    let mut new_line = 1usize;

    while idx < lines.len() {
        let mut context_start_idx = idx;
        let mut context_start_a = old_line;
        let mut context_start_b = new_line;

        while idx < lines.len() && lines[idx].0 == OpKind::Equal {
            if idx - context_start_idx >= context {
                context_start_a += 1;
                context_start_b += 1;
                context_start_idx += 1;
            }
            old_line += 1;
            new_line += 1;
            idx += 1;
        }
        if idx >= lines.len() {
            break;
        }

        let mut hunk_lines: Vec<(OpKind, usize, usize)> = lines[context_start_idx..idx].to_vec();
        let hunk_start_a = context_start_a;
        let hunk_start_b = context_start_b;
        let mut trailing_context_count = 0;

        while idx < lines.len() {
            let (kind, _, _) = lines[idx];
            match kind {
                OpKind::Insert => {
                    hunk_lines.push(lines[idx]);
                    new_line += 1;
                    trailing_context_count = 0;
                }
                OpKind::Delete => {
                    hunk_lines.push(lines[idx]);
                    old_line += 1;
                    trailing_context_count = 0;
                }
                OpKind::Equal => {
                    if trailing_context_count >= context {
                        break;
                    }
                    hunk_lines.push(lines[idx]);
                    old_line += 1;
                    new_line += 1;
                    trailing_context_count += 1;
                }
            }
            idx += 1;
        }

        let mut hunk_ops: Vec<Op> = Vec::new();
        for (kind, a_idx, b_idx) in hunk_lines {
            let start = match kind {
                OpKind::Equal | OpKind::Delete => a_idx,
                OpKind::Insert => b_idx,
            };
            if let Some(last) = hunk_ops.last_mut()
                && last.kind == kind
                && last.start as usize + last.len as usize == start
            {
                last.len += 1;
                continue;
            }
            hunk_ops.push(Op {
                kind,
                start: u32_len(start),
                len: 1,
            });
        }

        let len_a = hunk_ops
            .iter()
            .filter(|op| op.kind != OpKind::Insert)
            .map(|op| op.len as usize)
            .sum();
        let len_b = hunk_ops
            .iter()
            .filter(|op| op.kind != OpKind::Delete)
            .map(|op| op.len as usize)
            .sum();

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
    use crate::diff::data::Diff;

    fn e(start: u32, len: u32) -> Op {
        Op::equal(start, len)
    }

    fn i(start: u32, len: u32) -> Op {
        Op::insert(start, len)
    }

    fn d(start: u32, len: u32) -> Op {
        Op::delete(start, len)
    }

    fn lines_in(hunk: &Hunk) -> usize {
        hunk.ops.iter().map(|op| op.len as usize).sum()
    }

    fn diff(ops: Vec<Op>, old: &[&str], new: &[&str]) -> Diff {
        Diff {
            ops,
            old_tokens: old.iter().copied().map(str::to_owned).collect(),
            new_tokens: new.iter().copied().map(str::to_owned).collect(),
        }
    }

    #[test]
    fn test_group_into_hunks_all_equal() {
        let ops = vec![e(0, 3)];
        let hunks = group_into_hunks(&ops, 3);
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_group_into_hunks_single_change() {
        let ops = vec![e(0, 1), e(1, 1), d(2, 1), i(2, 1), e(3, 1), e(4, 1)];
        let hunks = group_into_hunks(&ops, 1);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].start_a, 2);
        assert_eq!(hunks[0].start_b, 2);
    }

    #[test]
    fn test_group_into_hunks_two_distant_changes() {
        let ops = vec![
            e(0, 1),
            d(1, 1),
            i(1, 1),
            e(2, 1),
            e(3, 1),
            e(4, 1),
            e(5, 1),
            d(6, 1),
            i(6, 1),
            e(7, 1),
        ];
        let hunks = group_into_hunks(&ops, 1);
        assert_eq!(hunks.len(), 2);
    }

    #[test]
    fn test_group_into_hunks_context_boundary() {
        let ops = vec![
            e(0, 1),
            e(1, 1),
            e(2, 1),
            e(3, 1),
            e(4, 1),
            d(5, 1),
            i(5, 1),
            e(6, 1),
            e(7, 1),
            e(8, 1),
        ];

        let hunks = group_into_hunks(&ops, 2);
        assert_eq!(hunks.len(), 1);
        assert_eq!(lines_in(&hunks[0]), 6);
        assert_eq!(hunks[0].ops.len(), 4, "6 lines re-encode to 4 runs");
    }

    #[test]
    fn test_group_into_hunks_change_at_start() {
        let ops = vec![d(0, 1), i(0, 1), e(1, 1), e(2, 1)];
        let hunks = group_into_hunks(&ops, 2);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].start_a, 1);
        assert_eq!(hunks[0].start_b, 1);
    }

    #[test]
    fn test_group_into_hunks_change_at_end() {
        let ops = vec![e(0, 1), e(1, 1), d(2, 1), i(2, 1)];
        let hunks = group_into_hunks(&ops, 2);
        assert_eq!(hunks.len(), 1);
    }

    #[test]
    fn test_group_into_hunks_long_equal_run_trimmed() {
        // 10 leading equal lines (context 2 keeps only 2), change, 10 trailing
        // equal lines (context 2 keeps only 2): 2+2+2 = 6 lines in the hunk.
        let ops = vec![e(0, 10), d(10, 1), i(10, 1), e(11, 10)];
        let hunks = group_into_hunks(&ops, 2);
        assert_eq!(hunks.len(), 1);
        assert_eq!(lines_in(&hunks[0]), 6);
        assert_eq!(hunks[0].ops.len(), 4, "6 lines re-encode to 4 runs");
    }

    #[test]
    fn test_render_unified_diff_empty() {
        let d = diff(vec![], &[], &[]);
        let result = render_unified_diff("old", "new", &d, 3, false);
        assert_eq!(result, "--- old\n+++ new\n");
    }

    #[test]
    fn test_render_unified_diff_basic() {
        let d = diff(
            vec![e(0, 1), d(1, 1), i(1, 1), e(2, 1)],
            &["a", "x", "b"],
            &["a", "y", "b"],
        );

        let result = render_unified_diff("f1", "f2", &d, 0, false);
        assert!(result.starts_with("--- f1\n+++ f2\n"));
        assert!(result.contains("@@ -2,1 +2,1 @@"));
        assert!(result.contains("-x"));
        assert!(result.contains("+y"));
    }

    #[test]
    fn test_render_unified_diff_compact_mode() {
        let d = diff(
            vec![e(0, 1), d(1, 1), i(1, 1), e(2, 1)],
            &["ctx", "old", "trail"],
            &["ctx", "new", "trail"],
        );

        let result = render_unified_diff("o", "n", &d, 0, false);
        assert!(!result.contains("ctx"));
        assert!(!result.contains("trail"));
        assert!(result.contains("-old"));
        assert!(result.contains("+new"));
    }

    #[test]
    fn test_render_unified_diff_color() {
        let d = diff(vec![d(0, 1), i(0, 1)], &["red"], &["green"]);
        let result = render_unified_diff("o", "n", &d, 0, true);
        assert!(result.contains("\x1b[31m"), "missing red");
        assert!(result.contains("\x1b[32m"), "missing green");
        assert!(result.contains("\x1b[36m"), "missing cyan hunk header");
        assert!(result.contains("\x1b[90m"), "missing gray file header");
    }

    #[test]
    fn test_render_unified_diff_run_unrolls() {
        // A Delete run of 2 renders 2 lines.
        let d = diff(vec![d(0, 2), i(0, 1)], &["old1", "old2"], &["new"]);
        let result = render_unified_diff("o", "n", &d, 0, false);
        assert!(result.contains("-old1"));
        assert!(result.contains("-old2"));
        assert!(result.contains("+new"));
    }
}
