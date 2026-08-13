use crate::diff::data::{Diff, OpKind};
use std::fmt::Write;

const RED: &str = "\x1B[31m";
const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

/// Render a simple line diff: context lines prefixed with two spaces, inserts
/// with `+ `, deletes with `- `, optionally ANSI-colored.
#[must_use]
pub fn render_line_diff(diff: &Diff, color: bool) -> String {
    let mut output = String::new();
    for op in &diff.ops {
        let tokens = diff.tokens_for(op.kind);
        let start = op.start as usize;
        for text in &tokens[start..start + op.len as usize] {
            match op.kind {
                OpKind::Equal => {
                    writeln!(output, "  {text}").unwrap();
                }
                OpKind::Insert => {
                    if color {
                        writeln!(output, "{GREEN}+ {text}{RESET}").unwrap();
                    } else {
                        writeln!(output, "+ {text}").unwrap();
                    }
                }
                OpKind::Delete => {
                    if color {
                        writeln!(output, "{RED}- {text}{RESET}").unwrap();
                    } else {
                        writeln!(output, "- {text}").unwrap();
                    }
                }
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::data::Op;

    fn diff(ops: Vec<Op>, old: &[&str], new: &[&str]) -> Diff {
        Diff {
            ops,
            old_tokens: old.iter().copied().map(str::to_owned).collect(),
            new_tokens: new.iter().copied().map(str::to_owned).collect(),
        }
    }

    #[test]
    fn test_render_line_diff_empty() {
        assert_eq!(render_line_diff(&diff(vec![], &[], &[]), false), "");
    }

    #[test]
    fn test_render_line_diff_equal_only() {
        let d = diff(
            vec![Op::equal(0, 2)],
            &["hello", "world"],
            &["hello", "world"],
        );
        assert_eq!(render_line_diff(&d, false), "  hello\n  world\n");
    }

    #[test]
    fn test_render_line_diff_insert_only() {
        let d = diff(vec![Op::insert(0, 1)], &[], &["added"]);
        assert_eq!(render_line_diff(&d, false), "+ added\n");
    }

    #[test]
    fn test_render_line_diff_delete_only() {
        let d = diff(vec![Op::delete(0, 1)], &["removed"], &[]);
        assert_eq!(render_line_diff(&d, false), "- removed\n");
    }

    #[test]
    fn test_render_line_diff_mixed() {
        let d = diff(
            vec![Op::equal(0, 1), Op::delete(1, 1), Op::insert(1, 1)],
            &["keep", "old"],
            &["keep", "new"],
        );
        assert_eq!(render_line_diff(&d, false), "  keep\n- old\n+ new\n");
    }

    #[test]
    fn test_render_line_diff_color() {
        let d = diff(
            vec![Op::insert(0, 1), Op::delete(0, 1)],
            &["red"],
            &["green"],
        );
        let result = render_line_diff(&d, true);
        assert!(result.contains("\x1B[32m"), "missing green");
        assert!(result.contains("\x1B[31m"), "missing red");
        assert!(result.contains("\x1B[0m"), "missing reset");
    }

    #[test]
    fn test_render_line_diff_no_color() {
        let d = diff(
            vec![Op::insert(0, 1), Op::delete(0, 1)],
            &["plain"],
            &["plain"],
        );
        let result = render_line_diff(&d, false);
        assert!(!result.contains('\x1B'), "unexpected escape codes");
    }

    #[test]
    fn test_render_line_diff_long_run() {
        // A Delete run of 2 lines renders both lines.
        let d = diff(vec![Op::delete(0, 2)], &["a", "b"], &[]);
        assert_eq!(render_line_diff(&d, false), "- a\n- b\n");
    }
}
