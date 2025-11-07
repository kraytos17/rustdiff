use crate::diff::core::myers::compute_diff;
use crate::diff::data::DiffOp;
use std::collections::HashMap;

pub fn compute_patience_diff(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let anchors = find_unique_anchors(a, b);
    if anchors.is_empty() {
        return compute_diff(a, b);
    }

    let mut result = Vec::new();
    let mut last_a = 0;
    let mut last_b = 0;

    for (ai, bi) in anchors {
        if ai > last_a || bi > last_b {
            result.extend(compute_diff(&a[last_a..ai], &b[last_b..bi]));
        }

        result.push(DiffOp::Equal(a[ai].to_string()));
        last_a = ai + 1;
        last_b = bi + 1;
    }

    if last_a < a.len() || last_b < b.len() {
        result.extend(compute_diff(&a[last_a..], &b[last_b..]));
    }

    result
}

fn find_unique_anchors<'a>(a: &'a [&'a str], b: &'a [&'a str]) -> Vec<(usize, usize)> {
    let freq_a = count_freq(a);
    let freq_b = count_freq(b);

    let mut pairs = Vec::new();
    for (i, sa) in a.iter().enumerate() {
        if freq_a[sa] == 1
            && let Some(j) = b.iter().position(|sb| *sb == *sa && freq_b[sb] == 1)
        {
            pairs.push((i, j));
        }
    }

    pairs.sort_by_key(|&(_, j)| j);

    longest_increasing_subsequence(pairs)
}

fn count_freq<'a>(seq: &'a [&'a str]) -> HashMap<&'a str, usize> {
    let mut map = HashMap::new();
    for &s in seq {
        *map.entry(s).or_insert(0) += 1;
    }

    map
}

fn longest_increasing_subsequence(pairs: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let mut last_b = None;
    for (a_idx, b_idx) in pairs {
        if last_b.is_none_or(|prev_b| b_idx > prev_b) {
            result.push((a_idx, b_idx));
            last_b = Some(b_idx);
        }
    }

    result
}
