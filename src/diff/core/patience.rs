use crate::diff::core::myers::compute_diff;
use crate::diff::data::DiffOp;
use std::collections::HashMap;

/// Compute a Patience Diff between two sequences.
///
/// Uses Myers’ algorithm for unmatched regions and unique anchors for stability.
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
            let mut sub_diff = compute_diff(&a[last_a..ai], &b[last_b..bi]);
            sub_diff.retain(|op| match op {
                DiffOp::Insert(s) | DiffOp::Delete(s) => !s.trim().is_empty(),
                DiffOp::Equal(_) => true,
            });

            if !sub_diff.is_empty() {
                result.extend(sub_diff);
            }
        }

        result.push(DiffOp::Equal(a[ai].to_string()));
        last_a = ai + 1;
        last_b = bi + 1;
    }

    if last_a < a.len() || last_b < b.len() {
        let mut sub_diff = compute_diff(&a[last_a..], &b[last_b..]);
        sub_diff.retain(|op| match op {
            DiffOp::Insert(s) | DiffOp::Delete(s) => !s.trim().is_empty(),
            DiffOp::Equal(_) => true,
        });

        if !sub_diff.is_empty() {
            result.extend(sub_diff);
        }
    }

    result
}

/// Find unique anchor pairs (lines that appear exactly once in both sides)
/// and return the Longest Increasing Subsequence (LIS) of such pairs.
fn find_unique_anchors<'a>(a: &'a [&'a str], b: &'a [&'a str]) -> Vec<(usize, usize)> {
    let freq_a = count_freq(a);
    let freq_b = count_freq(b);

    let unique_b: HashMap<&'a str, usize> = b
        .iter()
        .enumerate()
        .filter_map(|(j, &sb)| (freq_b.get(sb).copied().unwrap_or(0) == 1).then_some((sb, j)))
        .collect();

    let pairs: Vec<(usize, usize)> = a
        .iter()
        .enumerate()
        .filter_map(|(i, &sa)| {
            (freq_a.get(sa).copied().unwrap_or(0) == 1)
                .then_some(unique_b.get(sa).map(|&j| (i, j)))
                .flatten()
        })
        .collect();

    longest_increasing_subsequence(&pairs)
}

fn count_freq<'a>(seq: &'a [&'a str]) -> HashMap<&'a str, usize> {
    let mut map = HashMap::new();
    for &s in seq {
        *map.entry(s).or_insert(0) += 1;
    }
    map
}

/// Compute the Longest Increasing Subsequence (LIS) of the provided pairs
/// where pairs are (a_idx, b_idx) and are already in increasing `a_idx` order.
/// Returns a subsequence of pairs (owned) in increasing `a_idx` order.
fn longest_increasing_subsequence(pairs: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if pairs.is_empty() {
        return Vec::new();
    }

    // tails_b: the last b_idx value for a subsequence of length `len` (index = len-1)
    let mut tails_b: Vec<usize> = Vec::new();
    // tails_idx: the index in `pairs` of the element that currently ends the subsequence
    let mut tails_idx: Vec<usize> = Vec::new();
    // prev: for each element in `pairs`, the index of the previous element in the subsequence
    let mut prev: Vec<Option<usize>> = vec![None; pairs.len()];
    for (idx, &(_, b_idx)) in pairs.iter().enumerate() {
        let pos = match tails_b.binary_search(&b_idx) {
            Ok(p) | Err(p) => p,
        };

        if pos == tails_b.len() {
            tails_b.push(b_idx);
            tails_idx.push(idx);
        } else {
            tails_b[pos] = b_idx;
            tails_idx[pos] = idx;
        }

        if pos > 0 {
            prev[idx] = Some(tails_idx[pos - 1]);
        }
    }

    let mut lis: Vec<(usize, usize)> = Vec::new();
    if let Some(&last_idx) = tails_idx.last() {
        let mut cur = Some(last_idx);
        while let Some(ci) = cur {
            lis.push(pairs[ci]);
            cur = prev[ci];
        }

        lis.reverse();
    }

    lis
}
