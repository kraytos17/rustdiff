use crate::diff::core::{compute_histogram_diff, myers::compute_diff};
use crate::diff::data::Diff;
use crate::diff::modes::DiffAlgorithm;

pub fn diff_lines(old: &str, new: &str, algorithm: DiffAlgorithm) -> Diff {
    let old_lines = split_and_trim_lines(old);
    let new_lines = split_and_trim_lines(new);
    let old_refs: Vec<&str> = old_lines.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_lines.iter().map(String::as_str).collect();

    let ops = match algorithm {
        DiffAlgorithm::Histogram => compute_histogram_diff(&old_refs, &new_refs),
        DiffAlgorithm::Myers => compute_diff(&old_refs, &new_refs),
    };

    Diff {
        ops,
        old_tokens: old_lines,
        new_tokens: new_lines,
    }
}

fn split_and_trim_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect()
}
