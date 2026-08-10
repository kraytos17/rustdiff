use crate::diff::data::DiffOp;
use std::fmt::Write;

const RED: &str = "\x1B[31m";
const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

#[must_use]
pub fn render_line_diff(diffs: &[DiffOp], color: bool) -> String {
    let mut output = String::new();
    for op in diffs {
        match op {
            DiffOp::Equal(text) => {
                writeln!(output, "  {text}").unwrap();
            }
            DiffOp::Insert(text) => {
                if color {
                    writeln!(output, "{GREEN}+ {text}{RESET}").unwrap();
                } else {
                    writeln!(output, "+ {text}").unwrap();
                }
            }
            DiffOp::Delete(text) => {
                if color {
                    writeln!(output, "{RED}- {text}{RESET}").unwrap();
                } else {
                    writeln!(output, "- {text}").unwrap();
                }
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_line_diff_empty() {
        assert_eq!(render_line_diff(&[], false), "");
    }

    #[test]
    fn test_render_line_diff_equal_only() {
        let diffs = vec![DiffOp::Equal("hello".into()), DiffOp::Equal("world".into())];
        let result = render_line_diff(&diffs, false);
        assert_eq!(result, "  hello\n  world\n");
    }

    #[test]
    fn test_render_line_diff_insert_only() {
        let diffs = vec![DiffOp::Insert("added".into())];
        let result = render_line_diff(&diffs, false);
        assert_eq!(result, "+ added\n");
    }

    #[test]
    fn test_render_line_diff_delete_only() {
        let diffs = vec![DiffOp::Delete("removed".into())];
        let result = render_line_diff(&diffs, false);
        assert_eq!(result, "- removed\n");
    }

    #[test]
    fn test_render_line_diff_mixed() {
        let diffs = vec![
            DiffOp::Equal("keep".into()),
            DiffOp::Delete("old".into()),
            DiffOp::Insert("new".into()),
        ];
        let result = render_line_diff(&diffs, false);
        assert_eq!(result, "  keep\n- old\n+ new\n");
    }

    #[test]
    fn test_render_line_diff_color() {
        let diffs = vec![DiffOp::Insert("green".into()), DiffOp::Delete("red".into())];
        let result = render_line_diff(&diffs, true);
        assert!(result.contains("\x1B[32m"), "missing green");
        assert!(result.contains("\x1B[31m"), "missing red");
        assert!(result.contains("\x1B[0m"), "missing reset");
    }

    #[test]
    fn test_render_line_diff_no_color() {
        let diffs = vec![
            DiffOp::Insert("plain".into()),
            DiffOp::Delete("plain".into()),
        ];
        let result = render_line_diff(&diffs, false);
        assert!(!result.contains('\x1B'), "unexpected escape codes");
    }
}
