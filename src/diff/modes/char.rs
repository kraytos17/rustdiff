use crate::diff::core::patience::compute_patience_diff;
use crate::diff::data::DiffOp;

pub fn diff_chars(old_text: &str, new_text: &str) -> Vec<DiffOp> {
    let old_chars: Vec<String> = old_text.chars().map(|c| c.to_string()).collect();
    let new_chars: Vec<String> = new_text.chars().map(|c| c.to_string()).collect();

    let old_refs: Vec<&str> = old_chars.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_chars.iter().map(String::as_str).collect();

    compute_patience_diff(&old_refs, &new_refs)
}
