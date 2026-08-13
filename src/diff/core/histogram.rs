use super::{Snake, trim_common_ends};
use crate::diff::core::myers::diff_u32;
use crate::diff::data::{Op, coalesce, u32_len};
use crate::diff::intern::intern_both;
use rapidhash::{HashMapExt, RapidHashMap};
use smallvec::SmallVec;
use std::cmp::Reverse;

/// Anchors only on tokens occurring at most this many times on either side.
/// Above this, histogram falls back to Myers to avoid quadratic behavior.
const MAX_OCCURRENCES: u32 = 64;

/// Histogram diff: picks the least-frequent common token as an anchor.
///
/// Generalizes patience's "unique on both sides" rule to "occur the fewest
/// times on both sides," finding useful anchors even in repetitive files and
/// only degrading to Myers for pathological repetition.
///
/// Mirrors [`crate::diff::core::myers::compute_diff`]'s structure: trim common
/// `&str` ends, intern the remaining middle, run the `u32` core, reattach.
#[must_use]
pub fn compute_histogram_diff(a: &[&str], b: &[&str]) -> Vec<Op> {
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
        histogram_inner_u32(&a_ids, &b_ids, u32_len(prefix_len), u32_len(prefix_len))
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

fn histogram_inner_u32(a: &[u32], b: &[u32], base_a: u32, base_b: u32) -> Vec<Op> {
    if a.is_empty() || b.is_empty() {
        return diff_u32(a, b, base_a, base_b);
    }

    let counts_a = build_counts(a);
    let counts_b = build_counts(b);
    let Some(id) = find_rarest_common_token(&counts_a, &counts_b) else {
        return diff_u32(a, b, base_a, base_b);
    };

    let apos = positions_of(a, id);
    let bpos = positions_of(b, id);
    let snake = find_best_snake(a, b, &apos, &bpos);
    let (mut left, right) = parallel_halves(
        (&a[..snake.x], &b[..snake.y], base_a, base_b),
        (
            &a[snake.u..],
            &b[snake.v..],
            base_a + u32_len(snake.u),
            base_b + u32_len(snake.v),
        ),
    );

    left.push(Op::equal(base_a + u32_len(snake.x), u32_len(snake.len())));
    left.extend(right);
    left
}

/// Diff the two independent halves around a snake. With the `parallel` feature,
/// large regions run concurrently via `rayon::join`; small regions (and the
/// default build) run sequentially to avoid join overhead.
fn parallel_halves(
    left: (&[u32], &[u32], u32, u32),
    right: (&[u32], &[u32], u32, u32),
) -> (Vec<Op>, Vec<Op>) {
    #[cfg(feature = "parallel")]
    {
        /// Only parallelize regions large enough to amortize task scheduling.
        const PARALLEL_THRESHOLD: usize = 16_384;
        let total = left.0.len() + left.1.len() + right.0.len() + right.1.len();
        if total >= PARALLEL_THRESHOLD {
            return rayon::join(
                || histogram_inner_u32(left.0, left.1, left.2, left.3),
                || histogram_inner_u32(right.0, right.1, right.2, right.3),
            );
        }
    }
    (
        histogram_inner_u32(left.0, left.1, left.2, left.3),
        histogram_inner_u32(right.0, right.1, right.2, right.3),
    )
}

/// Count occurrences of each token, capped at `MAX_OCCURRENCES + 1` so that
/// "too frequent" is detectable without exact counts. Building cheap `u32`
/// counts before collecting any positions keeps the Myers-fallback path (no
/// shared token) from paying for position tables it never uses.
fn build_counts(a: &[u32]) -> RapidHashMap<u32, u32> {
    let mut counts = RapidHashMap::with_capacity(a.len().min(4096));
    for &id in a {
        let c = counts.entry(id).or_insert(0);
        if *c < MAX_OCCURRENCES + 1 {
            *c += 1;
        }
    }
    counts
}

/// Positions of `id` in `seq`. Only ever called for the already-chosen rarest
/// token, so this is one scan per side rather than a full position table.
fn positions_of(seq: &[u32], id: u32) -> SmallVec<[u32; 4]> {
    seq.iter()
        .enumerate()
        .filter(|(_, t)| **t == id)
        .map(|(i, _)| u32::try_from(i).expect("token position exceeds u32"))
        .collect()
}

/// Pick the token present in both sides whose combined occurrence count is
/// lowest (generalizing patience's "unique" = count 1). Ties break by token ID
/// for deterministic output.
fn find_rarest_common_token(
    counts_a: &RapidHashMap<u32, u32>,
    counts_b: &RapidHashMap<u32, u32>,
) -> Option<u32> {
    counts_a
        .iter()
        .filter(|(_, ca)| **ca > 0 && **ca <= MAX_OCCURRENCES)
        .filter_map(|(&id, &ca)| {
            counts_b
                .get(&id)
                .filter(|cb| **cb > 0 && **cb <= MAX_OCCURRENCES)
                .map(|&cb| (id, ca + cb))
        })
        .min_by_key(|c| (c.1, c.0))
        .map(|(id, _)| id)
}

/// Extend every `(a_pos, b_pos)` occurrence pair of the chosen token into a
/// maximal snake and return the longest; ties break toward the snake closest to
/// the region's center (best-balanced recursion).
fn find_best_snake(a: &[u32], b: &[u32], apos: &[u32], bpos: &[u32]) -> Snake {
    let n = a.len();
    let m = b.len();

    apos.iter()
        .flat_map(|&ai| {
            bpos.iter()
                .map(move |&bi| extend_snake(a, b, ai as usize, bi as usize))
        })
        .max_by_key(|s| (s.len(), Reverse(center_dist(s, n, m))))
        .expect("rarest common token always yields a non-empty snake")
}

#[allow(
    clippy::suspicious_operation_groupings,
    reason = "a[i - 1 - left] == b[j - 1 - left] compares the same offset in two distinct sequences"
)]
fn extend_snake(a: &[u32], b: &[u32], i: usize, j: usize) -> Snake {
    let mut left = 0;
    while i > left && j > left && a[i - 1 - left] == b[j - 1 - left] {
        left += 1;
    }

    let mut right = 0;
    while i + right + 1 < a.len() && j + right + 1 < b.len() && a[i + right + 1] == b[j + right + 1]
    {
        right += 1;
    }

    Snake {
        x: i - left,
        y: j - left,
        u: i + right + 1,
        v: j + right + 1,
    }
}

/// Distance of the snake's start-sum to the region's center (smaller = more
/// central split).
const fn center_dist(s: &Snake, n: usize, m: usize) -> usize {
    (2 * (s.x + s.y)).abs_diff(n + m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::core::myers::compute_diff;
    use crate::diff::data::{Diff, OpKind};

    fn s<'a>(seq: &'a [&'a str]) -> Vec<&'a str> {
        seq.to_vec()
    }

    fn assert_round_trip(a: &[&str], b: &[&str], ops: &[Op]) {
        let old_tokens: Vec<String> = a.iter().copied().map(str::to_owned).collect();
        let new_tokens: Vec<String> = b.iter().copied().map(str::to_owned).collect();
        let diff = Diff {
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

    fn sum_changes(ops: &[Op]) -> usize {
        ops.iter()
            .filter(|op| op.kind != OpKind::Equal)
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
    fn test_histogram_simple_anchor() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["a", "X", "c"]);
        let ops = compute_histogram_diff(&a, &b);
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
    fn test_histogram_multiple_anchors() {
        let a = s(&["a", "b", "c", "d", "e"]);
        let b = s(&["a", "X", "c", "Y", "e"]);
        let ops = compute_histogram_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(
            ops,
            vec![
                Op::equal(0, 1),
                Op::delete(1, 1),
                Op::insert(1, 1),
                Op::equal(2, 1),
                Op::delete(3, 1),
                Op::insert(3, 1),
                Op::equal(4, 1),
            ]
        );
    }

    #[test]
    fn test_histogram_no_common_falls_back_to_myers() {
        let a = s(&["x", "y", "z"]);
        let b = s(&["a", "b", "c"]);
        let ops = compute_histogram_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        let reference = compute_diff(&a, &b);
        assert_eq!(ops, reference);
    }

    #[test]
    fn test_histogram_repetitive_falls_back() {
        let a_strs: Vec<String> = (0..70).map(|_| "x".to_string()).collect();
        let mut b_strs: Vec<String> = a_strs.clone();
        b_strs.push("CHANGED".to_string());
        let a: Vec<&str> = a_strs.iter().map(String::as_str).collect();
        let b: Vec<&str> = b_strs.iter().map(String::as_str).collect();
        let ops = compute_histogram_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(sum_eq(&ops), 70);
    }

    #[test]
    fn test_histogram_identical() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["a", "b", "c"]);
        let ops = compute_histogram_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(ops, vec![Op::equal(0, 3)]);
    }

    #[test]
    fn test_histogram_out_of_order_anchor() {
        let a = s(&["a", "b", "c"]);
        let b = s(&["c", "b", "a"]);
        let ops = compute_histogram_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
    }

    #[test]
    fn test_histogram_empty() {
        assert!(compute_histogram_diff(&[], &[]).is_empty());
        let a = s(&[]);
        let b = s(&["hello"]);
        let ops = compute_histogram_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        assert_eq!(ops, vec![Op::insert(0, 1)]);
    }

    #[test]
    fn test_histogram_repeated_tokens_anchor() {
        let a = s(&["x", "a", "x", "b", "x"]);
        let b = s(&["x", "a", "x", "C", "x"]);
        let ops = compute_histogram_diff(&a, &b);
        assert_round_trip(&a, &b, &ops);
        let reference = compute_diff(&a, &b);
        assert_eq!(sum_eq(&ops), sum_eq(&reference));
    }

    proptest::proptest! {
        #[test]
        fn prop_histogram_valid_small(
            a in proptest::collection::vec("[a-c]{0,4}", 0..16),
            b in proptest::collection::vec("[a-c]{0,4}", 0..16),
        ) {
            let a_refs: Vec<&str> = a.iter().map(String::as_str).collect();
            let b_refs: Vec<&str> = b.iter().map(String::as_str).collect();
            let ops = compute_histogram_diff(&a_refs, &b_refs);
            assert_round_trip(&a_refs, &b_refs, &ops);
            let lcs = lcs_len(&a_refs, &b_refs);
            assert!(sum_eq(&ops) <= lcs, "histogram exceeds LCS");
            assert!(sum_changes(&ops) >= a_refs.len() + b_refs.len() - 2 * lcs, "histogram below minimal");
        }

        #[test]
        fn prop_histogram_valid_large(
            a in proptest::collection::vec("[a-z]{0,6}", 0..24),
            b in proptest::collection::vec("[a-z]{0,6}", 0..24),
        ) {
            let a_refs: Vec<&str> = a.iter().map(String::as_str).collect();
            let b_refs: Vec<&str> = b.iter().map(String::as_str).collect();
            let ops = compute_histogram_diff(&a_refs, &b_refs);
            assert_round_trip(&a_refs, &b_refs, &ops);
            let lcs = lcs_len(&a_refs, &b_refs);
            assert!(sum_eq(&ops) <= lcs, "histogram exceeds LCS");
            assert!(sum_changes(&ops) >= a_refs.len() + b_refs.len() - 2 * lcs, "histogram below minimal");
        }
    }
}
