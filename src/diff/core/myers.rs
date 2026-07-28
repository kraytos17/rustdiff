#![allow(clippy::many_single_char_names, clippy::suspicious_operation_groupings)]

use super::trim_common_ends;
use crate::diff::data::DiffOp;

pub fn compute_diff(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let (prefix_len, suffix_len, a_mid, b_mid) = trim_common_ends(a, b);
    if a_mid.is_empty() && b_mid.is_empty() {
        return a.iter().map(|s| DiffOp::Equal(s.to_string())).collect();
    }

    let middle = compute_diff_inner(a_mid, b_mid);
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

fn compute_diff_inner(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let n = a.len().cast_signed();
    let m = b.len().cast_signed();
    let max = (n + m).cast_unsigned();
    let mut v = vec![0isize; 2 * max + 1];
    let mut trace = Vec::new();

    for d in 0..=max {
        for k in (-(d.cast_signed())..=d.cast_signed()).step_by(2) {
            let index = (max.cast_signed() + k).cast_unsigned();
            let x_start = match (k == -(d.cast_signed()), k == d.cast_signed()) {
                (true, _) => safe_get(&v, max, k + 1),     // Down (insert)
                (_, true) => safe_get(&v, max, k - 1) + 1, // Right (delete)
                _ => {
                    let down = safe_get(&v, max, k + 1);
                    let right = safe_get(&v, max, k - 1);
                    if right < down {
                        down // Insert
                    } else {
                        right + 1 // Delete
                    }
                }
            };

            let mut x = x_start;
            let mut y = x - k;
            while x < n && y < m && a[x.cast_unsigned()] == b[y.cast_unsigned()] {
                x += 1;
                y += 1;
            }

            v[index] = x;
            if x == n && y == m {
                trace.push(v.clone());
                return backtrack(&trace, a, b);
            }
        }
        trace.push(v.clone());
    }
    unreachable!("Myers diff algorithm failed — unexpected termination");
}

/// Backtrack through the trace to reconstruct the diff operations.
fn backtrack(trace: &[Vec<isize>], a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let n = a.len().cast_signed();
    let m = b.len().cast_signed();
    let max = (n + m).cast_unsigned();
    let mut x = n;
    let mut y = m;

    let mut diffs = Vec::new();
    let zero_vec = vec![0isize; 2 * max + 1];
    for (d, _) in trace.iter().enumerate().rev() {
        let prev_v = if d == 0 { &zero_vec } else { &trace[d - 1] };
        let k = x - y;
        let down_x = safe_get(prev_v, max, k + 1);
        let right_x = safe_get(prev_v, max, k - 1);
        let came_from_insert = if k == -(d.cast_signed()) {
            true
        } else if k == d.cast_signed() {
            false
        } else {
            right_x < down_x
        };

        let x_start = if came_from_insert {
            down_x
        } else {
            right_x + 1
        };
        let y_start = x_start - k;
        while x > x_start && y > y_start {
            diffs.push(DiffOp::Equal(a[(x - 1).cast_unsigned()].to_string()));
            x -= 1;
            y -= 1;
        }

        if x == 0 && y == 0 {
            break;
        }
        if came_from_insert {
            if y > 0 {
                diffs.push(DiffOp::Insert(b[(y - 1).cast_unsigned()].to_string()));
                y -= 1;
            }
        } else if x > 0 {
            diffs.push(DiffOp::Delete(a[(x - 1).cast_unsigned()].to_string()));
            x -= 1;
        }
    }

    while x > 0 && y > 0 {
        diffs.push(DiffOp::Equal(a[(x - 1).cast_unsigned()].to_string()));
        x -= 1;
        y -= 1;
    }
    while x > 0 {
        diffs.push(DiffOp::Delete(a[(x - 1).cast_unsigned()].to_string()));
        x -= 1;
    }
    while y > 0 {
        diffs.push(DiffOp::Insert(b[(y - 1).cast_unsigned()].to_string()));
        y -= 1;
    }

    diffs.reverse();
    diffs
}

#[inline]
fn safe_get(v: &[isize], max: usize, k: isize) -> isize {
    let idx = (max.cast_signed() + k).cast_unsigned();
    v.get(idx).copied().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::data::DiffOp;

    /// Apply a diff to the original sequence to reconstruct the target.
    fn apply_diff(a: &[&str], diff: &[DiffOp]) -> Vec<String> {
        let mut result = Vec::new();
        let mut ai = 0;
        for op in diff {
            match op {
                DiffOp::Equal(s) => {
                    assert_eq!(
                        Some(s.as_str()),
                        a.get(ai).copied(),
                        "Equal operation mismatched original sequence"
                    );
                    result.push(s.clone());
                    ai += 1;
                }
                DiffOp::Insert(s) => result.push(s.clone()),
                DiffOp::Delete(s) => {
                    assert_eq!(
                        Some(s.as_str()),
                        a.get(ai).copied(),
                        "Delete operation removed wrong element"
                    );
                    ai += 1;
                }
            }
        }
        result
    }

    fn s<'a>(seq: &'a [&'a str]) -> Vec<&'a str> {
        seq.to_vec()
    }

    #[test]
    fn test_identical_sequences() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["a", "b", "c"]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
        assert!(diff.iter().all(|op| matches!(op, DiffOp::Equal(_))));
    }

    #[test]
    fn test_insertion_at_end() {
        let a = s(&["a", "b"]);
        let b = s(&["a", "b", "c"]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
    }

    #[test]
    fn test_insertion_at_start() {
        let a = s(&["b", "c"]);
        let b = s(&["a", "b", "c"]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
    }

    #[test]
    fn test_deletion_from_middle() {
        let a = s(&["a", "b", "c", "d"]);
        let b = s(&["a", "c", "d"]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
    }

    #[test]
    fn test_replacement() {
        let a = s(&["I", "love", "Rust"]);
        let b = s(&["I", "hate", "Rust"]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
    }

    #[test]
    fn test_completely_different_sequences() {
        let a = s(&["x", "y", "z"]);
        let b = s(&["a", "b", "c"]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);

        // All deletes then all inserts (number sanity)
        let deletes = diff
            .iter()
            .filter(|d| matches!(d, DiffOp::Delete(_)))
            .count();
        let inserts = diff
            .iter()
            .filter(|d| matches!(d, DiffOp::Insert(_)))
            .count();
        assert_eq!(deletes, 3);
        assert_eq!(inserts, 3);
    }

    #[test]
    fn test_empty_to_nonempty() {
        let a = s(&[]);
        let b = s(&["hello", "world"]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
        assert!(diff.iter().all(|op| matches!(op, DiffOp::Insert(_))));
    }

    #[test]
    fn test_nonempty_to_empty() {
        let a = s(&["bye", "now"]);
        let b = s(&[]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
        assert!(diff.iter().all(|op| matches!(op, DiffOp::Delete(_))));
    }

    #[test]
    fn test_repeated_elements() {
        let a = s(&["a", "a", "b"]);
        let b = s(&["a", "b", "b"]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
    }

    #[test]
    fn test_insert_delete_mix() {
        let a = s(&["a", "b", "x", "d"]);
        let b = s(&["a", "b", "c", "d", "e"]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
    }

    #[test]
    fn test_empty_both() {
        let a = s(&[]);
        let b = s(&[]);
        let diff = compute_diff(&a, &b);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_longer_random_sequences() {
        let a: Vec<_> = (0..50).map(|i| i.to_string()).collect();
        let b: Vec<_> = (25..75).map(|i| i.to_string()).collect();
        let a_refs: Vec<_> = a.iter().map(|s| s.as_str()).collect();
        let b_refs: Vec<_> = b.iter().map(|s| s.as_str()).collect();

        let diff = compute_diff(&a_refs, &b_refs);
        assert_eq!(apply_diff(&a_refs, &diff), b_refs);
    }

    #[test]
    fn test_partial_overlap_sequences() {
        let a = s(&["A", "B", "C", "D", "E"]);
        let b = s(&["B", "C", "F", "E"]);
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
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
    fn test_trim_common_ends_identical_short() {
        let a = s(&["a"]);
        let b = s(&["a"]);
        let (prefix, _suffix, a_mid, _b_mid) = trim_common_ends(&a, &b);
        assert_eq!(prefix, 1);
        assert!(a_mid.is_empty());
    }

    #[test]
    fn test_myers_change_in_middle_with_common_ends() {
        let a = s(&["a", "b", "c", "d", "e"]);
        let b = s(&["a", "b", "X", "d", "e"]);
        let diff = compute_diff(&a, &b);

        assert_eq!(apply_diff(&a, &diff), b);
        assert!(matches!(diff[0], DiffOp::Equal(ref s) if s == "a"));
        assert!(matches!(diff[1], DiffOp::Equal(ref s) if s == "b"));
        assert!(matches!(diff[2], DiffOp::Delete(ref s) if s == "c"));
        assert!(matches!(diff[3], DiffOp::Insert(ref s) if s == "X"));
        assert!(matches!(diff[4], DiffOp::Equal(ref s) if s == "d"));
        assert!(matches!(diff[5], DiffOp::Equal(ref s) if s == "e"));
    }

    #[test]
    fn test_myers_large_prefix_small_change() {
        let a_strs: Vec<String> = (0..100).map(|i| format!("line_{i}")).collect();
        let mut b_strs: Vec<String> = (0..100).map(|i| format!("line_{i}")).collect();

        b_strs[50] = "CHANGED".to_string();
        let a: Vec<&str> = a_strs.iter().map(String::as_str).collect();
        let b: Vec<&str> = b_strs.iter().map(String::as_str).collect();
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
    }
}
