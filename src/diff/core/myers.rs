use super::{Snake, trim_common_ends};
use crate::diff::data::{Op, coalesce, u32_len};
use crate::diff::intern::intern_both;

/// Compute a linear-space Myers diff over two `&str` token sequences, emitting
/// run-length-encoded ops.
#[must_use]
pub fn compute_diff(a: &[&str], b: &[&str]) -> Vec<Op> {
    let (prefix_len, suffix_len, a_mid, b_mid) = trim_common_ends(a, b);
    if a_mid.is_empty() && b_mid.is_empty() {
        return if a.is_empty() {
            Vec::new()
        } else {
            vec![Op::equal(0, u32_len(a.len()))]
        };
    }

    let middle = if a_mid.is_empty() {
        vec![Op::insert(u32_len(prefix_len), u32_len(b_mid.len()))]
    } else if b_mid.is_empty() {
        vec![Op::delete(u32_len(prefix_len), u32_len(a_mid.len()))]
    } else if a_mid.len() == 1 && b_mid.len() == 1 {
        if a_mid[0] == b_mid[0] {
            vec![Op::equal(u32_len(prefix_len), 1)]
        } else {
            vec![
                Op::delete(u32_len(prefix_len), 1),
                Op::insert(u32_len(prefix_len), 1),
            ]
        }
    } else {
        let (_interner, a_ids, b_ids) = intern_both(a_mid, b_mid);
        diff_u32(&a_ids, &b_ids, u32_len(prefix_len), u32_len(prefix_len))
    };

    let mut result = Vec::with_capacity(3);
    if prefix_len > 0 {
        result.push(Op::equal(0, u32_len(prefix_len)));
    }

    result.extend(middle);
    if suffix_len > 0 {
        result.push(Op::equal(
            u32_len(a.len() - suffix_len),
            u32_len(suffix_len),
        ));
    }
    coalesce(&mut result);
    result
}

/// Linear-space Myers diff over interned token IDs, emitting run-length ops.
///
/// `base_a`/`base_b` are the positions of `a[0]`/`b[0]` within the caller's
/// full token arrays, so emitted `Op`s carry absolute indices.
#[must_use]
pub(crate) fn diff_u32(a: &[u32], b: &[u32], base_a: u32, base_b: u32) -> Vec<Op> {
    let mut vf = vec![-1isize; 2 * (a.len() + b.len()) + 3];
    let mut vb = vec![-1isize; 2 * (a.len() + b.len()) + 3];
    let mut out = Vec::new();
    diff_recursive(a, b, base_a, base_b, &mut vf, &mut vb, &mut out);
    out
}

fn diff_recursive(
    a: &[u32],
    b: &[u32],
    base_a: u32,
    base_b: u32,
    vf: &mut [isize],
    vb: &mut [isize],
    out: &mut Vec<Op>,
) {
    if a.is_empty() {
        if !b.is_empty() {
            out.push(Op::insert(base_b, u32_len(b.len())));
        }
        return;
    }
    if b.is_empty() {
        out.push(Op::delete(base_a, u32_len(a.len())));
        return;
    }

    let lead = common_prefix_len(a, b);
    if lead > 0 {
        out.push(Op::equal(base_a, u32_len(lead)));
        diff_recursive(
            &a[lead..],
            &b[lead..],
            base_a + u32_len(lead),
            base_b + u32_len(lead),
            vf,
            vb,
            out,
        );
        return;
    }

    let trail = common_suffix_len(a, b);
    if trail > 0 {
        diff_recursive(
            &a[..a.len() - trail],
            &b[..b.len() - trail],
            base_a,
            base_b,
            vf,
            vb,
            out,
        );
        out.push(Op::equal(base_a + u32_len(a.len() - trail), u32_len(trail)));
        return;
    }

    let snake = find_middle_snake(a, b, vf, vb);
    if snake.x == snake.u && snake.y == snake.v {
        let n = a.len();
        let m = b.len();
        if snake.x == 0 && snake.y == 0 {
            out.push(Op::delete(base_a, 1));
            out.push(Op::insert(base_b, 1));
            diff_recursive(&a[1..], &b[1..], base_a + 1, base_b + 1, vf, vb, out);
        } else if snake.x == n && snake.y == m {
            diff_recursive(&a[..n - 1], &b[..m - 1], base_a, base_b, vf, vb, out);
            out.push(Op::delete(base_a + u32_len(n - 1), 1));
            out.push(Op::insert(base_b + u32_len(m - 1), 1));
        } else {
            diff_recursive(&a[..snake.x], &b[..snake.y], base_a, base_b, vf, vb, out);
            diff_recursive(
                &a[snake.x..],
                &b[snake.y..],
                base_a + u32_len(snake.x),
                base_b + u32_len(snake.y),
                vf,
                vb,
                out,
            );
        }
        return;
    }

    diff_recursive(&a[..snake.x], &b[..snake.y], base_a, base_b, vf, vb, out);
    out.push(Op::equal(base_a + u32_len(snake.x), u32_len(snake.len())));
    diff_recursive(
        &a[snake.u..],
        &b[snake.v..],
        base_a + u32_len(snake.u),
        base_b + u32_len(snake.v),
        vf,
        vb,
        out,
    );
}

/// Run the forward (top-left) and reverse (bottom-right) Myers fronts until
/// they overlap, returning the "middle snake" where the shortest path crosses
/// the diagonal.
#[allow(
    clippy::many_single_char_names,
    clippy::suspicious_operation_groupings,
    reason = "x/y/k/d/sx/sy and the vf[ki - 1] < vf[ki + 1] diagonals follow Myers' paper notation"
)]
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
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

fn common_suffix_len(a: &[u32], b: &[u32]) -> usize {
    a.iter()
        .rev()
        .zip(b.iter().rev())
        .take_while(|(x, y)| x == y)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::data::OpKind;

    fn s<'a>(seq: &'a [&'a str]) -> Vec<&'a str> {
        seq.to_vec()
    }

    fn assert_round_trip(a: &[&str], b: &[&str], ops: &[Op]) {
        let old_tokens: Vec<String> = a.iter().copied().map(str::to_owned).collect();
        let new_tokens: Vec<String> = b.iter().copied().map(str::to_owned).collect();
        let diff = crate::diff::data::Diff {
            ops: ops.to_vec(),
            old_tokens,
            new_tokens,
        };
        assert!(diff.validate_round_trip(a, b), "round-trip failed");
    }

    fn sum_eq(ops: &[Op]) -> usize {
        ops.iter()
            .filter(|op| op.kind == OpKind::Equal)
            .map(|op| op.len as usize)
            .sum()
    }

    /// Brute-force LCS length (edit distance = n + m - 2 * LCS).
    fn lcs_len(a: &[&str], b: &[&str]) -> usize {
        let n = a.len();
        let m = b.len();
        let mut dp = vec![vec![0usize; m + 1]; n + 1];
        for i in 1..=n {
            for j in 1..=m {
                dp[i][j] = if a[i - 1] == b[j - 1] {
                    dp[i - 1][j - 1] + 1
                } else {
                    dp[i - 1][j].max(dp[i][j - 1])
                };
            }
        }
        dp[n][m]
    }

    #[test]
    fn test_identical_sequences() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["a", "b", "c"]);
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(ops, vec![Op::equal(0, 3)]);
    }

    #[test]
    fn test_deletion_from_middle() {
        let a = s(&["a", "b", "c", "d"]);
        let b = s(&["a", "c", "d"]);
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(
            ops,
            vec![Op::equal(0, 1), Op::delete(1, 1), Op::equal(2, 2)]
        );
    }

    #[test]
    fn test_insertion_at_start() {
        let a = s(&["b", "c"]);
        let b = s(&["a", "b", "c"]);
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(ops, vec![Op::insert(0, 1), Op::equal(0, 2)]);
    }

    #[test]
    fn test_replacement() {
        let a = s(&["I", "love", "Rust"]);
        let b = s(&["I", "hate", "Rust"]);
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(
            ops,
            vec![
                Op::equal(0, 1),
                Op::delete(1, 1),
                Op::insert(1, 1),
                Op::equal(2, 1),
            ]
        );
    }

    #[test]
    fn test_completely_different_sequences() {
        let a = s(&["x", "y", "z"]);
        let b = s(&["a", "b", "c"]);
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(ops, vec![Op::delete(0, 3), Op::insert(0, 3)]);
    }

    #[test]
    fn test_empty_to_nonempty() {
        let a = s(&[]);
        let b = s(&["hello", "world"]);
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(ops, vec![Op::insert(0, 2)]);
    }

    #[test]
    fn test_nonempty_to_empty() {
        let a = s(&["bye", "now"]);
        let b = s(&[]);
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(ops, vec![Op::delete(0, 2)]);
    }

    #[test]
    fn test_repeated_elements() {
        let a = s(&["a", "a", "b"]);
        let b = s(&["a", "b", "b"]);
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(sum_eq(&ops), lcs_len(&a, &b));
    }

    #[test]
    fn test_insert_delete_mix() {
        let a = s(&["a", "b", "x", "d"]);
        let b = s(&["a", "b", "c", "d", "e"]);
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(sum_eq(&ops), lcs_len(&a, &b));
    }

    #[test]
    fn test_empty_both() {
        let a = s(&[]);
        let b = s(&[]);
        let ops = compute_diff(&a, &b);
        assert!(ops.is_empty());
    }

    #[test]
    fn test_longer_random_sequences() {
        let a: Vec<_> = (0..50).map(|i| i.to_string()).collect();
        let b: Vec<_> = (25..75).map(|i| i.to_string()).collect();
        let a_refs: Vec<_> = a.iter().map(String::as_str).collect();
        let b_refs: Vec<_> = b.iter().map(String::as_str).collect();

        let ops = compute_diff(&a_refs, &b_refs);
        assert_round_trip(&a_refs, &b_refs, &ops);
        assert_eq!(sum_eq(&ops), lcs_len(&a_refs, &b_refs));
    }

    #[test]
    fn test_partial_overlap_sequences() {
        let a = s(&["A", "B", "C", "D", "E"]);
        let b = s(&["B", "C", "F", "E"]);
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(sum_eq(&ops), lcs_len(&a, &b));
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
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(
            ops,
            vec![
                Op::equal(0, 2),
                Op::delete(2, 1),
                Op::insert(2, 1),
                Op::equal(3, 2),
            ]
        );
    }

    #[test]
    fn test_myers_large_prefix_small_change() {
        let a_strs: Vec<String> = (0..100).map(|i| format!("line_{i}")).collect();
        let mut b_strs: Vec<String> = (0..100).map(|i| format!("line_{i}")).collect();

        b_strs[50] = "CHANGED".to_string();
        let a: Vec<&str> = a_strs.iter().map(String::as_str).collect();
        let b: Vec<&str> = b_strs.iter().map(String::as_str).collect();
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(sum_eq(&ops), lcs_len(&a, &b));
    }

    #[test]
    fn test_no_common_large() {
        let a_strs: Vec<String> = (0..5000).map(|i| format!("a{i}")).collect();
        let b_strs: Vec<String> = (0..5000).map(|i| format!("b{i}")).collect();
        let a: Vec<&str> = a_strs.iter().map(String::as_str).collect();
        let b: Vec<&str> = b_strs.iter().map(String::as_str).collect();
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(sum_eq(&ops), 0);
        assert_eq!(ops, vec![Op::delete(0, 5000), Op::insert(0, 5000)]);
    }

    #[test]
    fn test_single_change_in_tenk() {
        let a_strs: Vec<String> = (0..10_000).map(|i| format!("line{i}")).collect();
        let mut b_strs: Vec<String> = a_strs.clone();
        b_strs[5000] = "CHANGED".to_string();
        let a: Vec<&str> = a_strs.iter().map(String::as_str).collect();
        let b: Vec<&str> = b_strs.iter().map(String::as_str).collect();
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(sum_eq(&ops), lcs_len(&a, &b));
        assert_eq!(
            ops.len(),
            4,
            "single change should be 4 runs (prefix, delete, insert, suffix), got {ops:?}"
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
        let ops = compute_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(sum_eq(&ops), lcs_len(&a, &b));
    }

    proptest::proptest! {
        #[test]
        fn prop_myers_minimal_small_alphabet(
            a in proptest::collection::vec("[a-c]{0,4}", 0..16),
            b in proptest::collection::vec("[a-c]{0,4}", 0..16),
        ) {
            let a_refs: Vec<&str> = a.iter().map(String::as_str).collect();
            let b_refs: Vec<&str> = b.iter().map(String::as_str).collect();
            let ops = compute_diff(&a_refs, &b_refs);
            assert_round_trip(&a_refs, &b_refs, &ops);
            assert_eq!(sum_eq(&ops), lcs_len(&a_refs, &b_refs), "not minimal");
        }

        #[test]
        fn prop_myers_minimal_large_alphabet(
            a in proptest::collection::vec("[a-z]{0,6}", 0..20),
            b in proptest::collection::vec("[a-z]{0,6}", 0..20),
        ) {
            let a_refs: Vec<&str> = a.iter().map(String::as_str).collect();
            let b_refs: Vec<&str> = b.iter().map(String::as_str).collect();
            let ops = compute_diff(&a_refs, &b_refs);
            assert_round_trip(&a_refs, &b_refs, &ops);
            assert_eq!(sum_eq(&ops), lcs_len(&a_refs, &b_refs), "not minimal");
        }
    }
}
