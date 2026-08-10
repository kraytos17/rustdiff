use super::trim_common_ends;
use crate::diff::core::myers::compute_diff;
use crate::diff::data::DiffOp;
use rapidhash::RapidHashMap;

#[must_use]
pub fn compute_patience_diff(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let (prefix_len, suffix_len, a_mid, b_mid) = trim_common_ends(a, b);
    if a_mid.is_empty() && b_mid.is_empty() {
        return a.iter().map(|s| DiffOp::Equal(s.to_string())).collect();
    }

    let middle = compute_patience_diff_inner(a_mid, b_mid);
    let mut result = Vec::with_capacity(prefix_len + middle.len() + suffix_len);
    for item in a.iter().take(prefix_len) {
        result.push(DiffOp::Equal(item.to_string()));
    }

    result.extend(middle);
    for item in a.iter().skip(a.len() - suffix_len) {
        result.push(DiffOp::Equal(item.to_string()));
    }
    result
}

fn compute_patience_diff_inner(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let anchors = find_unique_anchors(a, b);
    if anchors.is_empty() {
        return compute_diff(a, b);
    }

    let mut result = Vec::new();
    let mut last_a = 0;
    let mut last_b = 0;
    for (ai, bi) in anchors {
        if ai > last_a || bi > last_b {
            let sub_diff = compute_diff(&a[last_a..ai], &b[last_b..bi]);

            if !sub_diff.is_empty() {
                result.extend(sub_diff);
            }
        }

        result.push(DiffOp::Equal(a[ai].to_string()));
        last_a = ai + 1;
        last_b = bi + 1;
    }

    if last_a < a.len() || last_b < b.len() {
        let sub_diff = compute_diff(&a[last_a..], &b[last_b..]);

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

    let unique_b: RapidHashMap<&'a str, usize> = b
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

fn count_freq<'a>(seq: &'a [&'a str]) -> RapidHashMap<&'a str, usize> {
    let mut map = RapidHashMap::default();
    for &s in seq {
        *map.entry(s).or_insert(0) += 1;
    }
    map
}

/// Compute the Longest Increasing Subsequence (LIS) of the provided pairs
/// where pairs are (`a_idx`, `b_idx`) and are already in increasing `a_idx` order.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::data::DiffOp;

    fn s<'a>(seq: &'a [&'a str]) -> Vec<&'a str> {
        seq.to_vec()
    }

    /// Apply a diff to the original sequence to reconstruct the target.
    /// This will panic if the diff is invalid.
    fn apply_diff(a: &[&str], diff: &[DiffOp]) -> Vec<String> {
        let mut result = Vec::new();
        let mut ai = 0;
        for op in diff {
            match op {
                DiffOp::Equal(s) => {
                    assert_eq!(
                        Some(s.as_str()),
                        a.get(ai).copied(),
                        "Equal operation mismatched original sequence at index {ai}",
                    );
                    result.push(s.clone());
                    ai += 1;
                }
                DiffOp::Insert(s) => result.push(s.clone()),
                DiffOp::Delete(s) => {
                    assert_eq!(
                        Some(s.as_str()),
                        a.get(ai).copied(),
                        "Delete operation removed wrong element at index {ai}",
                    );
                    ai += 1;
                }
            }
        }
        result
    }

    #[test]
    fn test_count_freq() {
        let seq = s(&["a", "b", "a", "c", "a", "b"]);
        let freq = count_freq(&seq);
        let mut expected = RapidHashMap::default();

        expected.insert("a", 3);
        expected.insert("b", 2);
        expected.insert("c", 1);
        assert_eq!(freq, expected);
    }

    #[test]
    fn test_lis_empty() {
        let pairs = vec![];
        assert_eq!(longest_increasing_subsequence(&pairs), vec![]);
    }

    #[test]
    fn test_lis_single() {
        let pairs = vec![(1, 1)];
        assert_eq!(longest_increasing_subsequence(&pairs), vec![(1, 1)]);
    }

    #[test]
    fn test_lis_all_increasing() {
        let pairs = vec![(1, 1), (2, 2), (3, 3)];
        assert_eq!(longest_increasing_subsequence(&pairs), pairs);
    }

    #[test]
    fn test_lis_all_decreasing() {
        let pairs = vec![(1, 3), (2, 2), (3, 1)];
        let lis = longest_increasing_subsequence(&pairs);
        assert_eq!(lis.len(), 1);
        assert!(pairs.contains(&lis[0]));
    }

    #[test]
    fn test_lis_standard_case() {
        let pairs = vec![
            (0, 8),
            (1, 4),
            (2, 12),
            (3, 2),
            (4, 10),
            (5, 6),
            (6, 14),
            (7, 1),
            (8, 9),
        ];

        let expected = vec![(3, 2), (5, 6), (8, 9)];
        assert_eq!(longest_increasing_subsequence(&pairs), expected);
    }

    #[test]
    fn test_find_anchors_simple() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["x", "b", "y"]);
        let expected = vec![(1, 1)];
        assert_eq!(find_unique_anchors(&a, &b), expected);
    }

    #[test]
    fn test_find_anchors_none() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["x", "y", "z"]);
        assert!(find_unique_anchors(&a, &b).is_empty());
    }

    #[test]
    fn test_find_anchors_non_unique() {
        let a = s(&["a", "x", "a"]);
        let b = s(&["a", "y", "a"]);
        assert!(find_unique_anchors(&a, &b).is_empty());
    }

    #[test]
    fn test_find_anchors_lis_filter() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["c", "b", "a"]);
        let anchors = find_unique_anchors(&a, &b);
        assert_eq!(anchors.len(), 1);
    }

    #[test]
    fn test_patience_identical() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["a", "b", "c"]);
        let diff = compute_patience_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
        assert!(diff.iter().all(|op| matches!(op, DiffOp::Equal(_))));
    }

    #[test]
    fn test_patience_no_anchors() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["x", "y", "z"]);
        let diff = compute_patience_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
    }

    #[test]
    fn test_patience_simple_anchor() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["a", "X", "c"]);
        let diff = compute_patience_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
        let expected = vec![
            DiffOp::Equal("a".to_string()),
            DiffOp::Delete("b".to_string()),
            DiffOp::Insert("X".to_string()),
            DiffOp::Equal("c".to_string()),
        ];

        assert_eq!(diff, expected);
    }

    #[test]
    fn test_patience_multiple_anchors() {
        let a = s(&["a", "b", "c", "d", "e"]);
        let b = s(&["a", "X", "c", "Y", "e"]);
        let diff = compute_patience_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
        let expected = vec![
            DiffOp::Equal("a".to_string()),
            DiffOp::Delete("b".to_string()),
            DiffOp::Insert("X".to_string()),
            DiffOp::Equal("c".to_string()),
            DiffOp::Delete("d".to_string()),
            DiffOp::Insert("Y".to_string()),
            DiffOp::Equal("e".to_string()),
        ];

        assert_eq!(diff, expected);
    }

    #[test]
    fn test_patience_out_of_order_anchor() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["c", "b", "a"]);
        let diff = compute_patience_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
    }

    #[test]
    fn test_patience_whitespace_filter_simple() {
        let a = s(&["a", " ", "b"]);
        let b = s(&["a", "\t", "b"]);
        let diff = compute_patience_diff(&a, &b);
        let expected = vec![
            DiffOp::Equal("a".to_string()),
            DiffOp::Delete(" ".to_string()),
            DiffOp::Insert("\t".to_string()),
            DiffOp::Equal("b".to_string()),
        ];

        assert_eq!(diff, expected);
    }

    #[test]
    fn test_trim_common_ends_none() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["x", "y", "z"]);
        let (prefix, suffix, a_mid, b_mid) = trim_common_ends(&a, &b);
        assert_eq!(prefix, 0);
        assert_eq!(suffix, 0);
        assert_eq!(a_mid, &a[..]);
        assert_eq!(b_mid, &b[..]);
    }

    #[test]
    fn test_trim_common_ends_full() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["a", "b", "c"]);
        let (prefix, suffix, a_mid, b_mid) = trim_common_ends(&a, &b);
        assert_eq!(prefix, 3);
        assert_eq!(suffix, 0);
        assert!(a_mid.is_empty());
        assert!(b_mid.is_empty());
    }

    #[test]
    fn test_trim_common_ends_prefix_only() {
        let a = s(&["a", "b", "x", "y"]);
        let b = s(&["a", "b", "p", "q"]);
        let (prefix, suffix, a_mid, b_mid) = trim_common_ends(&a, &b);
        assert_eq!(prefix, 2);
        assert_eq!(suffix, 0);
        assert_eq!(a_mid, &["x", "y"]);
        assert_eq!(b_mid, &["p", "q"]);
    }

    #[test]
    fn test_trim_common_ends_suffix_only() {
        let a = s(&["x", "y", "a", "b"]);
        let b = s(&["p", "q", "a", "b"]);
        let (prefix, suffix, a_mid, b_mid) = trim_common_ends(&a, &b);
        assert_eq!(prefix, 0);
        assert_eq!(suffix, 2);
        assert_eq!(a_mid, &["x", "y"]);
        assert_eq!(b_mid, &["p", "q"]);
    }

    #[test]
    fn test_trim_common_ends_both() {
        let a = s(&["a", "b", "x", "c", "d"]);
        let b = s(&["a", "b", "y", "c", "d"]);
        let (prefix, suffix, a_mid, b_mid) = trim_common_ends(&a, &b);
        assert_eq!(prefix, 2);
        assert_eq!(suffix, 2);
        assert_eq!(a_mid, &["x"]);
        assert_eq!(b_mid, &["y"]);
    }

    #[test]
    fn test_trim_common_ends_partial_overlap() {
        let a = s(&["a", "b", "c", "d"]);
        let b = s(&["a", "x", "c", "d"]);
        let (prefix, suffix, a_mid, b_mid) = trim_common_ends(&a, &b);
        assert_eq!(prefix, 1);
        assert_eq!(suffix, 2);
        assert_eq!(a_mid, &["b"]);
        assert_eq!(b_mid, &["x"]);
    }

    #[test]
    fn test_trim_common_ends_prefix_overlaps_suffix_guard() {
        let a = s(&["a"]);
        let b = s(&["a"]);
        let (prefix, suffix, a_mid, b_mid) = trim_common_ends(&a, &b);
        assert_eq!(prefix, 1);
        assert_eq!(suffix, 0);
        assert!(a_mid.is_empty());
        assert!(b_mid.is_empty());
    }

    #[test]
    fn test_patience_change_in_middle_with_common_ends() {
        let a = s(&["a", "b", "c", "d", "e"]);
        let b = s(&["a", "b", "X", "d", "e"]);
        let diff = compute_patience_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
        assert!(matches!(diff[0], DiffOp::Equal(ref s) if s == "a"));
        assert!(matches!(diff[1], DiffOp::Equal(ref s) if s == "b"));
        assert!(matches!(diff[2], DiffOp::Delete(ref s) if s == "c"));
        assert!(matches!(diff[3], DiffOp::Insert(ref s) if s == "X"));
        assert!(matches!(diff[4], DiffOp::Equal(ref s) if s == "d"));
        assert!(matches!(diff[5], DiffOp::Equal(ref s) if s == "e"));
    }

    #[test]
    fn test_patience_large_prefix_small_change() {
        let a_strs: Vec<String> = (0..100).map(|i| format!("line_{i}")).collect();
        let mut b_strs: Vec<String> = (0..100).map(|i| format!("line_{i}")).collect();
        b_strs[50] = "CHANGED".to_string();
        let a: Vec<&str> = a_strs.iter().map(String::as_str).collect();
        let b: Vec<&str> = b_strs.iter().map(String::as_str).collect();
        let diff = compute_patience_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
    }
}
