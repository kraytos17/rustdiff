use crate::diff::data::DiffOp;
use std::fmt::Write;

const RED: &str = "\x1B[31m";
const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";
const MAX_LOOKAHEAD: usize = 8;

/// Render inline, human-readable word diffs.
/// Adjacent insert/delete pairs are treated as replacements.
/// Whitespace-only tokens are folded logically into neighbors.
#[must_use]
pub fn render_word_diff(diffs: &[DiffOp], color: bool) -> String {
    let mut output = String::new();
    let mut line_buf = String::new();

    let mut i = 0;
    while i < diffs.len() {
        match &diffs[i] {
            DiffOp::Equal(text) => {
                line_buf.push_str(text);
                if text.ends_with('\n') {
                    output.push_str(&line_buf);
                    line_buf.clear();
                }
                i += 1;
            }
            DiffOp::Insert(insert_text) => {
                let (consumed, delete_text) = find_matching_delete(&diffs[i..]);
                if let Some(delete_text) = delete_text {
                    if render_grouped(&mut line_buf, delete_text, insert_text, color) {
                        output.push_str(&line_buf);
                        line_buf.clear();
                    }
                    i += consumed;
                } else {
                    if render_insert(&mut line_buf, insert_text, color) {
                        output.push_str(&line_buf);
                        line_buf.clear();
                    }
                    i += 1;
                }
            }
            DiffOp::Delete(delete_text) => {
                let (consumed, insert_text) = find_matching_insert(&diffs[i..]);
                if let Some(insert_text) = insert_text {
                    if render_grouped(&mut line_buf, delete_text, insert_text, color) {
                        output.push_str(&line_buf);
                        line_buf.clear();
                    }
                    i += consumed;
                } else {
                    if render_delete(&mut line_buf, delete_text, color) {
                        output.push_str(&line_buf);
                        line_buf.clear();
                    }
                    i += 1;
                }
            }
        }
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
    let mut steps = 0;
    while i < ops.len() && steps < MAX_LOOKAHEAD {
        match &ops[i] {
            DiffOp::Delete(text) => {
                return (i + 1, Some(text));
            }
            DiffOp::Equal(text) if text.trim().is_empty() => {
                skip_whitespace = true;
                steps += 1;
                i += 1;
            }
            _ => {
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
    let mut steps = 0;

    while i < ops.len() && steps < MAX_LOOKAHEAD {
        match &ops[i] {
            DiffOp::Insert(text) => {
                return (i + 1, Some(text));
            }
            DiffOp::Equal(text) if text.trim().is_empty() => {
                skip_whitespace = true;
                steps += 1;
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

fn render_grouped(buf: &mut String, old: &str, new: &str, color: bool) -> bool {
    let (old_word, old_space) = split_trailing_space(old);
    let (new_word, new_space) = split_trailing_space(new);

    if color {
        write!(
            buf,
            "{RED}[-{old_word}]{RESET}{GREEN}[+{new_word}]{RESET}{old_space}{new_space}"
        )
        .unwrap();
    } else {
        write!(buf, "[-{old_word}+{new_word}]{old_space}{new_space}").unwrap();
    }

    old_space.contains('\n') || new_space.contains('\n')
}

fn render_insert(buf: &mut String, text: &str, color: bool) -> bool {
    let (word, space) = split_trailing_space(text);
    if color {
        write!(buf, "{GREEN}[+{word}]{RESET}{space}").unwrap();
    } else {
        write!(buf, "[+{word}]{space}").unwrap();
    }

    space.contains('\n')
}

fn render_delete(buf: &mut String, text: &str, color: bool) -> bool {
    let (word, space) = split_trailing_space(text);
    if color {
        write!(buf, "{RED}[-{word}]{RESET}{space}").unwrap();
    } else {
        write!(buf, "[-{word}]{space}").unwrap();
    }

    space.contains('\n')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::data::DiffOp;

    #[test]
    fn test_find_matching_delete_adjacent() {
        let ops = vec![
            DiffOp::Insert("hello".into()),
            DiffOp::Delete("world".into()),
        ];
        let (consumed, text) = find_matching_delete(&ops);
        assert_eq!(consumed, 2);
        assert_eq!(text, Some("world"));
    }

    #[test]
    fn test_find_matching_delete_skips_whitespace() {
        let ops = vec![
            DiffOp::Insert("hello".into()),
            DiffOp::Equal(" ".into()),
            DiffOp::Delete("world".into()),
        ];
        let (consumed, text) = find_matching_delete(&ops);
        assert_eq!(consumed, 3);
        assert_eq!(text, Some("world"));
    }

    #[test]
    fn test_find_matching_delete_bounded_lookahead() {
        let mut ops = vec![DiffOp::Insert("hello".into())];
        for _ in 0..MAX_LOOKAHEAD {
            ops.push(DiffOp::Equal(" ".into()));
        }
        ops.push(DiffOp::Delete("world".into()));
        let (consumed, text) = find_matching_delete(&ops);
        assert!(text.is_none());
        assert_eq!(consumed, 1 + MAX_LOOKAHEAD);
    }

    #[test]
    fn test_find_matching_insert_adjacent() {
        let ops = vec![
            DiffOp::Delete("hello".into()),
            DiffOp::Insert("world".into()),
        ];
        let (consumed, text) = find_matching_insert(&ops);
        assert_eq!(consumed, 2);
        assert_eq!(text, Some("world"));
    }

    #[test]
    fn test_find_matching_insert_skips_whitespace() {
        let ops = vec![
            DiffOp::Delete("hello".into()),
            DiffOp::Equal(" ".into()),
            DiffOp::Insert("world".into()),
        ];
        let (consumed, text) = find_matching_insert(&ops);
        assert_eq!(consumed, 3);
        assert_eq!(text, Some("world"));
    }

    #[test]
    fn test_find_matching_insert_bounded_lookahead() {
        let mut ops = vec![DiffOp::Delete("hello".into())];
        for _ in 0..MAX_LOOKAHEAD {
            ops.push(DiffOp::Equal(" ".into()));
        }
        ops.push(DiffOp::Insert("world".into()));
        let (consumed, text) = find_matching_insert(&ops);
        assert!(text.is_none());
        assert_eq!(consumed, 1 + MAX_LOOKAHEAD);
    }

    #[test]
    fn test_find_matching_delete_no_match() {
        let ops = vec![
            DiffOp::Insert("hello".into()),
            DiffOp::Insert("world".into()),
        ];
        let (consumed, text) = find_matching_delete(&ops);
        assert!(text.is_none());
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_render_word_diff_replacement_grouped() {
        let diffs = vec![
            DiffOp::Equal("hello ".into()),
            DiffOp::Delete("world".into()),
            DiffOp::Insert("rust".into()),
            DiffOp::Equal("\n".into()),
        ];
        let result = render_word_diff(&diffs, false);
        assert_eq!(result, "hello [-world+rust]\n");
    }

    #[test]
    fn test_render_word_diff_standalone_insert() {
        let diffs = vec![
            DiffOp::Equal("hello ".into()),
            DiffOp::Insert("world".into()),
            DiffOp::Equal("\n".into()),
        ];
        let result = render_word_diff(&diffs, false);
        assert_eq!(result, "hello [+world]\n");
    }

    #[test]
    fn test_render_word_diff_standalone_delete() {
        let diffs = vec![
            DiffOp::Equal("hello ".into()),
            DiffOp::Delete("world".into()),
            DiffOp::Equal("\n".into()),
        ];
        let result = render_word_diff(&diffs, false);
        assert_eq!(result, "hello [-world]\n");
    }
}
