use crate::diff::data::{Diff, OpKind};
use std::fmt::Write;

const RED: &str = "\x1B[31m";
const GREEN: &str = "\x1B[32m";
const RESET: &str = "\x1B[0m";
const MAX_LOOKAHEAD: usize = 8;

/// Render inline, human-readable word diffs.
/// Adjacent insert/delete pairs are treated as replacements.
/// Whitespace-only tokens are folded logically into neighbors.
#[must_use]
pub fn render_word_diff(diff: &Diff, color: bool) -> String {
    let edits = diff.edits();
    render_word_edits(&edits, color)
}

fn render_word_edits(edits: &[(OpKind, &str)], color: bool) -> String {
    let mut output = String::new();
    let mut line_buf = String::new();
    let mut i = 0;
    while i < edits.len() {
        match edits[i] {
            (OpKind::Equal, text) => {
                line_buf.push_str(text);
                if text.ends_with('\n') {
                    output.push_str(&line_buf);
                    line_buf.clear();
                }
                i += 1;
            }
            (OpKind::Insert, insert_text) => {
                let (consumed, delete_text) = find_matching(&edits[i..], OpKind::Delete);
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
            (OpKind::Delete, delete_text) => {
                let (consumed, insert_text) = find_matching(&edits[i..], OpKind::Insert);
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

/// Look for an edit of `target` kind that matches this one, skipping
/// whitespace-only edits in between. Returns the number of edits consumed and
/// the matched text (bounded by `MAX_LOOKAHEAD`).
fn find_matching<'a>(edits: &'a [(OpKind, &'a str)], target: OpKind) -> (usize, Option<&'a str>) {
    let mut i = 1;
    let mut skip_whitespace = false;
    let mut steps = 0;
    while i < edits.len() && steps < MAX_LOOKAHEAD {
        match edits[i] {
            (kind, text) if kind == target => return (i + 1, Some(text)),
            (OpKind::Equal, text) if text.trim().is_empty() => {
                skip_whitespace = true;
                steps += 1;
                i += 1;
            }
            _ => break,
        }
    }

    (if skip_whitespace { i } else { 1 }, None)
}

fn split_trailing_space(s: &str) -> (&str, &str) {
    let trimmed = s.trim_end_matches(char::is_whitespace);
    s.split_at(trimmed.len())
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
    use crate::diff::data::{Diff, Op};

    fn diff(ops: Vec<Op>, old: &[&str], new: &[&str]) -> Diff {
        Diff {
            ops,
            old_tokens: old.iter().copied().map(str::to_owned).collect(),
            new_tokens: new.iter().copied().map(str::to_owned).collect(),
        }
    }

    #[test]
    fn test_find_matching_delete_adjacent() {
        let edits = vec![(OpKind::Insert, "hello"), (OpKind::Delete, "world")];
        let (consumed, text) = find_matching(&edits, OpKind::Delete);
        assert_eq!(consumed, 2);
        assert_eq!(text, Some("world"));
    }

    #[test]
    fn test_find_matching_delete_skips_whitespace() {
        let edits = vec![
            (OpKind::Insert, "hello"),
            (OpKind::Equal, " "),
            (OpKind::Delete, "world"),
        ];

        let (consumed, text) = find_matching(&edits, OpKind::Delete);
        assert_eq!(consumed, 3);
        assert_eq!(text, Some("world"));
    }

    #[test]
    fn test_find_matching_delete_bounded_lookahead() {
        let mut edits = vec![(OpKind::Insert, "hello")];
        for _ in 0..MAX_LOOKAHEAD {
            edits.push((OpKind::Equal, " "));
        }

        edits.push((OpKind::Delete, "world"));
        let (consumed, text) = find_matching(&edits, OpKind::Delete);
        assert!(text.is_none());
        assert_eq!(consumed, 1 + MAX_LOOKAHEAD);
    }

    #[test]
    fn test_find_matching_insert_adjacent() {
        let edits = vec![(OpKind::Delete, "hello"), (OpKind::Insert, "world")];
        let (consumed, text) = find_matching(&edits, OpKind::Insert);
        assert_eq!(consumed, 2);
        assert_eq!(text, Some("world"));
    }

    #[test]
    fn test_find_matching_insert_skips_whitespace() {
        let edits = vec![
            (OpKind::Delete, "hello"),
            (OpKind::Equal, " "),
            (OpKind::Insert, "world"),
        ];

        let (consumed, text) = find_matching(&edits, OpKind::Insert);
        assert_eq!(consumed, 3);
        assert_eq!(text, Some("world"));
    }

    #[test]
    fn test_find_matching_insert_bounded_lookahead() {
        let mut edits = vec![(OpKind::Delete, "hello")];
        for _ in 0..MAX_LOOKAHEAD {
            edits.push((OpKind::Equal, " "));
        }

        edits.push((OpKind::Insert, "world"));
        let (consumed, text) = find_matching(&edits, OpKind::Insert);
        assert!(text.is_none());
        assert_eq!(consumed, 1 + MAX_LOOKAHEAD);
    }

    #[test]
    fn test_find_matching_delete_no_match() {
        let edits = vec![(OpKind::Insert, "hello"), (OpKind::Insert, "world")];
        let (consumed, text) = find_matching(&edits, OpKind::Delete);
        assert!(text.is_none());
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_render_word_diff_replacement_grouped() {
        let d = diff(
            vec![
                Op::equal(0, 1),
                Op::delete(1, 1),
                Op::insert(1, 1),
                Op::equal(2, 1),
            ],
            &["hello ", "world", "\n"],
            &["hello ", "rust", "\n"],
        );
        assert_eq!(render_word_diff(&d, false), "hello [-world+rust]\n");
    }

    #[test]
    fn test_render_word_diff_standalone_insert() {
        let d = diff(
            vec![Op::equal(0, 1), Op::insert(1, 1), Op::equal(1, 1)],
            &["hello ", "\n"],
            &["hello ", "world", "\n"],
        );
        assert_eq!(render_word_diff(&d, false), "hello [+world]\n");
    }

    #[test]
    fn test_render_word_diff_standalone_delete() {
        let d = diff(
            vec![Op::equal(0, 1), Op::delete(1, 1), Op::equal(2, 1)],
            &["hello ", "world", "\n"],
            &["hello ", "\n"],
        );
        assert_eq!(render_word_diff(&d, false), "hello [-world]\n");
    }

    #[test]
    fn test_render_word_diff_insert_run() {
        // An Insert run of 2 unrolls into two standalone insert markers.
        let d = diff(
            vec![Op::equal(0, 1), Op::insert(1, 2), Op::equal(1, 1)],
            &["hello ", "\n"],
            &["hello ", "very", " big", "\n"],
        );

        let result = render_word_diff(&d, false);
        assert!(result.contains("[+very]"), "result: {result:?}");
        assert!(result.contains("[+ big]"), "result: {result:?}");
    }
}
