use crate::diff::core::patience::compute_patience_diff;
use crate::diff::data::DiffOp;
use regex::Regex;
use std::sync::LazyLock;

static WORD_TOKEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\[-.*?\+.*?\]|[^\s]+\s*|\n)").unwrap());

pub fn diff_words(old_text: &str, new_text: &str) -> Vec<DiffOp> {
    let old_tokens = tokenize(&old_text.replace("\r\n", "\n"));
    let new_tokens = tokenize(&new_text.replace("\r\n", "\n"));

    let old_refs: Vec<&str> = old_tokens.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_tokens.iter().map(String::as_str).collect();

    compute_patience_diff(&old_refs, &new_refs)
}

fn tokenize(text: &str) -> Vec<String> {
    WORD_TOKEN_RE
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic_words() {
        let tokens = tokenize("hello world");
        assert_eq!(tokens, vec!["hello ", "world"]);
    }

    #[test]
    fn test_tokenize_marker_token() {
        let tokens = tokenize("[-old+new]");
        assert_eq!(tokens, vec!["[-old+new]"]);
    }

    #[test]
    fn test_tokenize_newline() {
        let tokens = tokenize("a\nb\n");
        assert_eq!(tokens, vec!["a\n", "b\n"]);
    }

    #[test]
    fn test_tokenize_marker_with_text() {
        let tokens = tokenize("foo [-a+b] bar");
        assert_eq!(tokens, vec!["foo ", "[-a+b]", "bar"]);
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_diff_words_round_trip() {
        let old = "hello world";
        let new = "hello rust";
        let diffs = diff_words(old, new);
        let mut result: Vec<String> = Vec::new();
        let mut ai = 0;
        let old_tokens = tokenize(old);
        for op in &diffs {
            match op {
                DiffOp::Equal(s) => {
                    assert_eq!(Some(s.as_str()), old_tokens.get(ai).map(String::as_str));
                    result.push(s.clone());
                    ai += 1;
                }
                DiffOp::Insert(s) => result.push(s.clone()),
                DiffOp::Delete(s) => {
                    assert_eq!(Some(s.as_str()), old_tokens.get(ai).map(String::as_str));
                    ai += 1;
                }
            }
        }
        let new_tokens: Vec<String> = tokenize(new);
        assert_eq!(result, new_tokens);
    }

    #[test]
    fn test_diff_words_crlf_normalized() {
        let old = "hello\r\nworld\r\n";
        let new = "hello\r\nrust\r\n";
        let diffs = diff_words(old, new);
        for op in &diffs {
            match op {
                DiffOp::Equal(s) | DiffOp::Insert(s) | DiffOp::Delete(s) => {
                    assert!(!s.contains('\r'), "CR leaked into diff op: {s:?}");
                }
            }
        }
    }
}
