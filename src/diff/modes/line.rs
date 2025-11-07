use crate::diff::{core::compute_patience_diff, data::DiffOp};

pub fn diff_lines(old: &str, new: &str) -> Vec<DiffOp> {
    let old_lines = split_and_trim_lines(old);
    let new_lines = split_and_trim_lines(new);
    let old_refs: Vec<&str> = old_lines.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_lines.iter().map(String::as_str).collect();

    compute_patience_diff(&old_refs, &new_refs)
}

fn split_and_trim_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect()
}
