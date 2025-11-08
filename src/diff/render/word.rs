use crate::diff::data::DiffOp;
use std::fmt::Write;

const RED: &str = "\x1B[31m";
const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";

/// Render inline, human-readable word diffs.
/// Adjacent insert/delete pairs are treated as replacements.
/// Whitespace-only tokens are folded logically into neighbors.
pub fn render_word_diff(diffs: &[DiffOp], color: bool) -> String {
    let mut output = String::new();
    let mut line_buf = String::new();

    let mut i = 0;
    while i < diffs.len() {
        match &diffs[i] {
            DiffOp::Equal(text) => {
                line_buf.push_str(text);
            }
            DiffOp::Insert(insert_text) => {
                // Look ahead for potential replacement groups
                let (consumed, delete_text) = find_matching_delete(&diffs[i..]);
                if let Some(delete_text) = delete_text {
                    render_grouped(&mut line_buf, delete_text, insert_text, color);
                    i += consumed;
                    continue;
                }

                render_insert(&mut line_buf, insert_text, color);
                i += 1;
                continue;
            }
            DiffOp::Delete(delete_text) => {
                // Look ahead for potential replacement groups
                let (consumed, insert_text) = find_matching_insert(&diffs[i..]);
                if let Some(insert_text) = insert_text {
                    render_grouped(&mut line_buf, delete_text, insert_text, color);
                    i += consumed;
                    continue;
                }

                render_delete(&mut line_buf, delete_text, color);
                i += 1;
                continue;
            }
        }

        // flush lines when newline detected
        if let Some(last) = op_text(&diffs[i.min(diffs.len() - 1)]).chars().last()
            && last == '\n'
        {
            output.push_str(&line_buf);
            line_buf.clear();
        }

        i += 1;
    }

    if !line_buf.is_empty() {
        output.push_str(&line_buf);
    }

    output
}

/// Look for a delete operation that matches this insert, skipping whitespace
fn find_matching_delete(ops: &[DiffOp]) -> (usize, Option<&str>) {
    let mut i = 1;
    let mut skip_whitespace = false;
    while i < ops.len() {
        match &ops[i] {
            DiffOp::Delete(text) => {
                // Found a delete - return it
                return (i + 1, Some(text));
            }
            DiffOp::Equal(text) if text.trim().is_empty() => {
                // Skip whitespace between insert and delete
                skip_whitespace = true;
                i += 1;
            }
            _ => {
                // Non-whitespace equal or another insert - no matching delete
                break;
            }
        }
    }

    (if skip_whitespace { i } else { 1 }, None)
}

/// Look for an insert operation that matches this delete, skipping whitespace
fn find_matching_insert(ops: &[DiffOp]) -> (usize, Option<&str>) {
    let mut i = 1;
    let mut skip_whitespace = false;

    while i < ops.len() {
        match &ops[i] {
            DiffOp::Insert(text) => {
                // Found an insert - return it
                return (i + 1, Some(text));
            }
            DiffOp::Equal(text) if text.trim().is_empty() => {
                skip_whitespace = true;
                i += 1;
            }
            _ => {
                break;
            }
        }
    }

    (if skip_whitespace { i } else { 1 }, None)
}

fn split_trailing_space(s: &str) -> (&str, &str) {
    let trimmed = s.trim_end_matches(|c: char| c.is_whitespace());
    let space = &s[trimmed.len()..];
    (trimmed, space)
}

fn render_grouped(buf: &mut String, old: &str, new: &str, color: bool) {
    let (old_word, old_space) = split_trailing_space(old);
    let (new_word, new_space) = split_trailing_space(new);
    let space = if new_space.is_empty() {
        old_space
    } else {
        new_space
    };

    if color {
        write!(
            buf,
            "{RED}[-{old_word}]{RESET}{GREEN}[+{new_word}]{RESET}{space}"
        )
        .unwrap();
    } else {
        write!(buf, "[-{old_word}+{new_word}]{space}").unwrap();
    }
}

fn render_insert(buf: &mut String, text: &str, color: bool) {
    let (word, space) = split_trailing_space(text);
    if color {
        write!(buf, "{GREEN}[+{word}]{RESET}{space}").unwrap();
    } else {
        write!(buf, "[+{word}]{space}").unwrap();
    }
}

fn render_delete(buf: &mut String, text: &str, color: bool) {
    let (word, space) = split_trailing_space(text);
    if color {
        write!(buf, "{RED}[-{word}]{RESET}{space}").unwrap();
    } else {
        write!(buf, "[-{word}]{space}").unwrap();
    }
}

fn op_text(op: &DiffOp) -> &str {
    match op {
        DiffOp::Equal(s) | DiffOp::Insert(s) | DiffOp::Delete(s) => s.as_str(),
    }
}
