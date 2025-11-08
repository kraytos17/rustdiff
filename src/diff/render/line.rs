use crate::diff::data::DiffOp;
use std::fmt::Write;

const RED: &str = "\x1B[31m";
const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

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
