use regex::Regex;

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
    let mut tokens = Vec::new();
    let re = Regex::new(r"(\[-.*?\+.*?\]|[^\s]+\s*|\n)").unwrap();
    for cap in re.find_iter(text) {
        tokens.push(cap.as_str().to_string());
    }

    tokens
}
