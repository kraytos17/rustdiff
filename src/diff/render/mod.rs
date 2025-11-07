use crate::diff::data::DiffOp;
use std::fmt::Write;

pub fn render_diff(diffs: &[DiffOp], color: bool, unified: usize) -> String {
    let mut output = String::new();
    let mut context = Vec::new();

    for op in diffs {
        match op {
            DiffOp::Equal(line) => {
                if unified > 0 {
                    context.push(format!("  {line}"));
                    if context.len() > unified * 2 {
                        context.remove(0);
                    }
                } else {
                    writeln!(output, "  {line}").unwrap();
                }
            }
            DiffOp::Insert(line) => {
                if color {
                    writeln!(output, "\x1b[32m+ {line}\x1b[0m").unwrap();
                } else {
                    writeln!(output, "+ {line}").unwrap();
                }
            }
            DiffOp::Delete(line) => {
                if color {
                    writeln!(output, "\x1b[31m- {line}\x1b[0m").unwrap();
                } else {
                    writeln!(output, "- {line}").unwrap();
                }
            }
        }
    }

    if !context.is_empty() {
        for line in context {
            writeln!(output, "{line}").unwrap();
        }
    }

    output
}
