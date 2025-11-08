use crate::diff::core::patience::compute_patience_diff;
use crate::diff::data::DiffOp;

/// Compute a word-aware diff between two texts and return `Vec<DiffOp>`.
///
/// - Strips existing ANSI color codes before diffing.
/// - Tokenizes text into canonical word/space tokens.
/// - Treats `[-old+new]` markers as atomic tokens (normalized).
/// - Collapses all whitespace runs into a single `" "` token.
pub fn diff_words(old_text: &str, new_text: &str) -> Vec<DiffOp> {
    let old_tokens = tokenize(old_text);
    let new_tokens = tokenize(new_text);

    let old_refs: Vec<&str> = old_tokens.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_tokens.iter().map(String::as_str).collect();

    compute_patience_diff(&old_refs, &new_refs)
}

/// Tokenize text into a Vec<String>.
///
/// Rules:
/// - `[-old+new]` → single normalized token (whitespace trimmed inside)
/// - consecutive whitespace collapsed to one `" "` token
/// - everything else split by whitespace
fn tokenize(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut prev_was_space = false;
    let mut i = 0;
    let len = bytes.len();

    while i < len {
        let b = bytes[i];
        if b.is_ascii_whitespace() {
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            if !prev_was_space {
                tokens.push(" ".to_string());
                prev_was_space = true;
            }
            continue;
        }
        if b == b'[' && i + 1 < len && bytes[i + 1] == b'-' {
            let start = i;
            i += 2;
            while i < len && bytes[i] != b'+' {
                i += 1;
            }

            if i < len {
                i += 1;
                while i < len && bytes[i] != b']' {
                    i += 1;
                }

                if i < len {
                    i += 1;
                    let mut tok = String::from_utf8_lossy(&bytes[start..i]).to_string();
                    if let Some(inner) = tok.strip_prefix('[').and_then(|s| s.strip_suffix(']'))
                        && let Some(stripped) = inner.strip_prefix('-')
                    {
                        let mut parts = stripped.splitn(2, '+');
                        let left = parts.next().unwrap_or("").trim();
                        let right = parts.next().unwrap_or("").trim();
                        if left.is_empty() && right.is_empty() {
                            continue;
                        }
                        tok = format!("[-{left}+{right}]");
                    }

                    tokens.push(tok);
                    prev_was_space = false;
                    continue;
                }
            }

            i = start;
        }

        let start = i;
        while i < len && !bytes[i].is_ascii_whitespace() && bytes[i] != b'[' {
            i += 1;
        }

        let tok = String::from_utf8_lossy(&bytes[start..i]).to_string();
        tokens.push(tok);
        prev_was_space = false;
    }

    tokens
}
