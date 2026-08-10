#![allow(clippy::many_single_char_names, clippy::suspicious_operation_groupings)]

use super::trim_common_ends;
use crate::diff::data::DiffOp;
use crate::diff::intern::{Interner, intern_both};

/// A matching run: `a[x..u] == b[y..v]`.
#[derive(Debug, Clone, Copy)]
struct Snake {
    x: usize,
    y: usize,
    u: usize,
    v: usize,
}

#[must_use]
pub fn compute_diff(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let (prefix_len, suffix_len, a_mid, b_mid) = trim_common_ends(a, b);
    if a_mid.is_empty() && b_mid.is_empty() {
        return a.iter().map(|s| DiffOp::Equal(s.to_string())).collect();
    }

    let middle = if a_mid.is_empty() {
        b_mid
            .iter()
            .map(|s| DiffOp::Insert(s.to_string()))
            .collect()
    } else if b_mid.is_empty() {
        a_mid
            .iter()
            .map(|s| DiffOp::Delete(s.to_string()))
            .collect()
    } else if a_mid.len() == 1 && b_mid.len() == 1 {
        if a_mid[0] == b_mid[0] {
            vec![DiffOp::Equal(a_mid[0].to_string())]
        } else {
            vec![
                DiffOp::Delete(a_mid[0].to_string()),
                DiffOp::Insert(b_mid[0].to_string()),
            ]
        }
    } else {
        let (interner, a_ids, b_ids) = intern_both(a_mid, b_mid);
        diff_u32(&a_ids, &b_ids, &interner)
    };

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

/// Linear-space Myers diff over interned token IDs.
///
/// Runs forward and reverse search fronts simultaneously, finds the "middle
/// snake" where they meet, and recurses on the two halves. Peak memory is
/// `O(n + m)` (two shared front arrays) instead of the full-trace
/// implementation's `O(D · (n + m))`.
#[must_use]
pub(crate) fn diff_u32(a: &[u32], b: &[u32], interner: &Interner<'_>) -> Vec<DiffOp> {
    let mut vf = vec![-1isize; 2 * (a.len() + b.len()) + 3];
    let mut vb = vec![-1isize; 2 * (a.len() + b.len()) + 3];
    let mut out = Vec::new();
    diff_recursive(a, b, interner, &mut vf, &mut vb, &mut out);
    out
}

fn diff_recursive(
    a: &[u32],
    b: &[u32],
    interner: &Interner<'_>,
    vf: &mut [isize],
    vb: &mut [isize],
    out: &mut Vec<DiffOp>,
) {
    if a.is_empty() {
        for &id in b {
            out.push(DiffOp::Insert(interner.resolve(id).to_string()));
        }
        return;
    }
    if b.is_empty() {
        for &id in a {
            out.push(DiffOp::Delete(interner.resolve(id).to_string()));
        }
        return;
    }

    let lead = common_prefix_len(a, b);
    if lead > 0 {
        for &id in &a[..lead] {
            out.push(DiffOp::Equal(interner.resolve(id).to_string()));
        }
        diff_recursive(&a[lead..], &b[lead..], interner, vf, vb, out);
        return;
    }

    let trail = common_suffix_len(a, b);
    if trail > 0 {
        diff_recursive(
            &a[..a.len() - trail],
            &b[..b.len() - trail],
            interner,
            vf,
            vb,
            out,
        );
        for &id in &a[a.len() - trail..] {
            out.push(DiffOp::Equal(interner.resolve(id).to_string()));
        }
        return;
    }

    let snake = find_middle_snake(a, b, vf, vb);
    if snake.x == snake.u && snake.y == snake.v {
        let n = a.len();
        let m = b.len();
        if snake.x == 0 && snake.y == 0 {
            out.push(DiffOp::Delete(interner.resolve(a[0]).to_string()));
            out.push(DiffOp::Insert(interner.resolve(b[0]).to_string()));
            diff_recursive(&a[1..], &b[1..], interner, vf, vb, out);
        } else if snake.x == n && snake.y == m {
            diff_recursive(&a[..n - 1], &b[..m - 1], interner, vf, vb, out);
            out.push(DiffOp::Delete(interner.resolve(a[n - 1]).to_string()));
            out.push(DiffOp::Insert(interner.resolve(b[m - 1]).to_string()));
        } else {
            diff_recursive(&a[..snake.x], &b[..snake.y], interner, vf, vb, out);
            diff_recursive(&a[snake.x..], &b[snake.y..], interner, vf, vb, out);
        }
        return;
    }

    diff_recursive(&a[..snake.x], &b[..snake.y], interner, vf, vb, out);
    for &id in &a[snake.x..snake.u] {
        out.push(DiffOp::Equal(interner.resolve(id).to_string()));
    }
    diff_recursive(&a[snake.u..], &b[snake.v..], interner, vf, vb, out);
}

/// Run the forward (top-left) and reverse (bottom-right) Myers fronts until
/// they overlap, returning the "middle snake" where the shortest path crosses
/// the diagonal.
fn find_middle_snake(a: &[u32], b: &[u32], vf: &mut [isize], vb: &mut [isize]) -> Snake {
    let n = a.len().cast_signed();
    let m = b.len().cast_signed();
    let max = (n + m).cast_unsigned();
    let delta = n - m;
    let odd = delta % 2 != 0;
    let off = max + 1;

    vf[off + 1] = 0;
    vb[off + 1] = 0;
    for d in 0..=max.div_ceil(2) {
        let di = d.cast_signed();
        // Forward frontier (original coordinates).
        for k in (-di..=di).step_by(2) {
            let ki = (off.cast_signed() + k).cast_unsigned();
            let x = if k == -di || (k != di && vf[ki - 1] < vf[ki + 1]) {
                vf[ki + 1] // down (insert)
            } else {
                vf[ki - 1] + 1 // right (delete)
            };

            let mut sx = x;
            let mut sy = x - k;
            while sx < n && sy < m && a[sx.cast_unsigned()] == b[sy.cast_unsigned()] {
                sx += 1;
                sy += 1;
            }

            vf[ki] = sx;
            if odd && k >= delta - (di - 1) && k <= delta + (di - 1) {
                let vb_idx = (off.cast_signed() + delta - k).cast_unsigned();
                if vf[ki] + vb[vb_idx] >= n {
                    return Snake {
                        x: x.cast_unsigned(),
                        y: (x - k).cast_unsigned(),
                        u: sx.cast_unsigned(),
                        v: sy.cast_unsigned(),
                    };
                }
            }
        }

        // Reverse frontier (reversed coordinates).
        for k in (-di..=di).step_by(2) {
            let ki = (off.cast_signed() + k).cast_unsigned();
            let x = if k == -di || (k != di && vb[ki - 1] < vb[ki + 1]) {
                vb[ki + 1]
            } else {
                vb[ki - 1] + 1
            };

            let mut sx = x;
            let mut sy = x - k;
            while sx < n
                && sy < m
                && a[(n - 1 - sx).cast_unsigned()] == b[(m - 1 - sy).cast_unsigned()]
            {
                sx += 1;
                sy += 1;
            }

            vb[ki] = sx;
            let fk = delta - k;
            if !odd && fk >= -di && fk <= di {
                let vf_idx = (off.cast_signed() + fk).cast_unsigned();
                if vf[vf_idx] + vb[ki] >= n {
                    return Snake {
                        x: (n - sx).cast_unsigned(),
                        y: (m - (sx - k)).cast_unsigned(),
                        u: (n - x).cast_unsigned(),
                        v: (m - (x - k)).cast_unsigned(),
                    };
                }
            }
        }
    }

    unreachable!("middle snake search failed");
}

fn common_prefix_len(a: &[u32], b: &[u32]) -> usize {
    let mut i = 0;
    while i < a.len() && i < b.len() && a[i] == b[i] {
        i += 1;
    }
    i
}

fn common_suffix_len(a: &[u32], b: &[u32]) -> usize {
    let mut i = 0;
    while i < a.len() && i < b.len() && a[a.len() - 1 - i] == b[b.len() - 1 - i] {
        i += 1;
    }
    i
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

    fn count_eq(diff: &[DiffOp]) -> usize {
        diff.iter()
            .filter(|op| matches!(op, DiffOp::Equal(_)))
            .count()
    }

    /// Assert the new linear-space diff matches the known-correct full-trace
    /// reference: same edit-script length (both minimal) and same LCS length.
    fn assert_equivalent(a: &[&str], b: &[&str]) {
        let new = compute_diff(a, b);
        let reference = compute_diff_reference(a, b);
        assert_eq!(
            new.len(),
            reference.len(),
            "edit-script length differs for {a:?} vs {b:?}"
        );
        assert_eq!(
            count_eq(&new),
            count_eq(&reference),
            "LCS length differs for {a:?} vs {b:?}"
        );
        assert_eq!(
            apply_diff(a, &new),
            b,
            "round-trip failed for {a:?} vs {b:?}"
        );
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
        let a_refs: Vec<_> = a.iter().map(String::as_str).collect();
        let b_refs: Vec<_> = b.iter().map(String::as_str).collect();

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

    #[test]
    fn test_no_common_large() {
        let a_strs: Vec<String> = (0..5000).map(|i| format!("a{i}")).collect();
        let b_strs: Vec<String> = (0..5000).map(|i| format!("b{i}")).collect();
        let a: Vec<&str> = a_strs.iter().map(String::as_str).collect();
        let b: Vec<&str> = b_strs.iter().map(String::as_str).collect();
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
        assert_eq!(count_eq(&diff), 0);
        let deletes = diff
            .iter()
            .filter(|o| matches!(o, DiffOp::Delete(_)))
            .count();
        let inserts = diff
            .iter()
            .filter(|o| matches!(o, DiffOp::Insert(_)))
            .count();
        assert_eq!(deletes, 5000);
        assert_eq!(inserts, 5000);
    }

    #[test]
    fn test_single_change_in_tenk() {
        let a_strs: Vec<String> = (0..10_000).map(|i| format!("line{i}")).collect();
        let mut b_strs: Vec<String> = a_strs.clone();
        b_strs[5000] = "CHANGED".to_string();
        let a: Vec<&str> = a_strs.iter().map(String::as_str).collect();
        let b: Vec<&str> = b_strs.iter().map(String::as_str).collect();
        let diff = compute_diff(&a, &b);
        assert_eq!(apply_diff(&a, &diff), b);
        assert_eq!(count_eq(&diff), 9999);
        assert_eq!(
            diff.iter()
                .filter(|o| matches!(o, DiffOp::Delete(_)))
                .count(),
            1
        );
        assert_eq!(
            diff.iter()
                .filter(|o| matches!(o, DiffOp::Insert(_)))
                .count(),
            1
        );
    }

    #[test]
    fn test_alternating() {
        let a_strs: Vec<String> = (0..2000)
            .map(|i| {
                if i % 2 == 0 {
                    "even".to_string()
                } else {
                    "odd".to_string()
                }
            })
            .collect();
        let b_strs: Vec<String> = (0..2000)
            .map(|i| {
                if i % 2 == 0 {
                    "odd".to_string()
                } else {
                    "even".to_string()
                }
            })
            .collect();
        let a: Vec<&str> = a_strs.iter().map(String::as_str).collect();
        let b: Vec<&str> = b_strs.iter().map(String::as_str).collect();
        assert_equivalent(&a, &b);
    }

    #[test]
    fn test_reference_equivalence_corners() {
        let cases: Vec<(Vec<&str>, Vec<&str>)> = vec![
            (vec!["x", "y", "z"], vec!["a", "b", "c"]),
            (vec!["a", "b", "c"], vec!["a", "X", "c"]),
            (vec!["a"], vec!["b"]),
            (vec![], vec!["a", "b"]),
            (vec!["a", "a", "b"], vec!["a", "b", "b"]),
            (vec!["a", "b", "c", "d", "e"], vec!["a", "b", "X", "d", "e"]),
            (vec!["a", "b"], vec!["a", "b", "c"]),
            (vec!["b", "c"], vec!["a", "b", "c"]),
        ];
        for (a, b) in cases {
            assert_equivalent(&a, &b);
        }
    }

    proptest::proptest! {
        #[test]
        fn prop_myers_round_trip_small_alphabet(
            a in proptest::collection::vec("[a-c]{0,4}", 0..16),
            b in proptest::collection::vec("[a-c]{0,4}", 0..16),
        ) {
            let a_refs: Vec<&str> = a.iter().map(String::as_str).collect();
            let b_refs: Vec<&str> = b.iter().map(String::as_str).collect();
            let diff = compute_diff(&a_refs, &b_refs);
            let reference = compute_diff_reference(&a_refs, &b_refs);
            assert_eq!(apply_diff(&a_refs, &diff), b);
            assert_eq!(diff.len(), reference.len(), "edit-script length differs");
            assert_eq!(count_eq(&diff), count_eq(&reference), "LCS length differs");
        }

        #[test]
        fn prop_myers_round_trip_large_alphabet(
            a in proptest::collection::vec("[a-z]{0,6}", 0..20),
            b in proptest::collection::vec("[a-z]{0,6}", 0..20),
        ) {
            let a_refs: Vec<&str> = a.iter().map(String::as_str).collect();
            let b_refs: Vec<&str> = b.iter().map(String::as_str).collect();
            let diff = compute_diff(&a_refs, &b_refs);
            let reference = compute_diff_reference(&a_refs, &b_refs);
            assert_eq!(apply_diff(&a_refs, &diff), b);
            assert_eq!(diff.len(), reference.len(), "edit-script length differs");
            assert_eq!(count_eq(&diff), count_eq(&reference), "LCS length differs");
        }
    }

    // --- Reference implementation (full-trace Myers), kept for differential
    // --- testing against the linear-space rewrite. Test-only

    fn compute_diff_reference(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
        let (prefix_len, suffix_len, a_mid, b_mid) = trim_common_ends(a, b);
        if a_mid.is_empty() && b_mid.is_empty() {
            return a.iter().map(|s| DiffOp::Equal(s.to_string())).collect();
        }
        let middle = compute_diff_inner_ref(a_mid, b_mid);
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

    fn compute_diff_inner_ref(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
        let n = a.len().cast_signed();
        let m = b.len().cast_signed();
        let max = (n + m).cast_unsigned();
        let mut v = vec![0isize; 2 * max + 1];
        let mut trace = Vec::new();

        for d in 0..=max {
            for k in (-(d.cast_signed())..=d.cast_signed()).step_by(2) {
                let index = (max.cast_signed() + k).cast_unsigned();
                let x_start = match (k == -(d.cast_signed()), k == d.cast_signed()) {
                    (true, _) => safe_get_ref(&v, max, k + 1),
                    (_, true) => safe_get_ref(&v, max, k - 1) + 1,
                    _ => {
                        let down = safe_get_ref(&v, max, k + 1);
                        let right = safe_get_ref(&v, max, k - 1);
                        if right < down { down } else { right + 1 }
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
                    return backtrack_ref(&trace, a, b);
                }
            }
            trace.push(v.clone());
        }
        unreachable!("reference Myers diff failed");
    }

    fn backtrack_ref(trace: &[Vec<isize>], a: &[&str], b: &[&str]) -> Vec<DiffOp> {
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
            let down_x = safe_get_ref(prev_v, max, k + 1);
            let right_x = safe_get_ref(prev_v, max, k - 1);
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

    fn safe_get_ref(v: &[isize], max: usize, k: isize) -> isize {
        let idx = (max.cast_signed() + k).cast_unsigned();
        v.get(idx).copied().unwrap_or(0)
    }
}
