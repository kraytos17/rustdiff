use crate::diff::core::patience::compute_patience_diff;
use crate::diff::data::DiffOp;

pub fn diff_words(old_text: &str, new_text: &str) -> Vec<DiffOp> {
    let old_tokens = tokenize(old_text);
    let new_tokens = tokenize(new_text);

    let old_refs: Vec<&str> = old_tokens.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_tokens.iter().map(String::as_str).collect();

    compute_patience_diff(&old_refs, &new_refs)
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            current.push(ch);
        } else {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            if ch.is_whitespace() {
                tokens.push(" ".to_string());
            } else {
                tokens.push(ch.to_string());
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}
