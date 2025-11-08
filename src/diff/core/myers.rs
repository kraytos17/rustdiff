use crate::diff::data::DiffOp;

/// Compute the minimal edit script between two slices using Myers' O(ND) diff algorithm.
///
/// # Parameters
/// - `a`: old sequence
/// - `b`: new sequence
///
/// # Returns
/// A vector of [`DiffOp`] representing insertions, deletions, and equal elements.
///
/// # Notes
/// Uses safe indexing and works for both line and word-level diffing.
pub fn compute_diff(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let n = a.len() as isize;
    let m = b.len() as isize;
    let max = (n + m) as usize;
    let mut v = vec![0isize; 2 * max + 1];
    let mut trace = Vec::new();

    for d in 0..=max {
        for k in (-(d as isize)..=d as isize).step_by(2) {
            let index = (max as isize + k) as usize;
            let x_start = match (k == -(d as isize), k == d as isize) {
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
            while x < n && y < m && a[x as usize] == b[y as usize] {
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
    let n = a.len() as isize;
    let m = b.len() as isize;
    let max = (n + m) as usize;
    let mut x = n;
    let mut y = m;

    let mut diffs = Vec::new();
    let zero_vec = vec![0isize; 2 * max + 1];
    for (d, _) in trace.iter().enumerate().rev() {
        // Find the 'v' from the previous step (d-1)
        let prev_v = if d == 0 { &zero_vec } else { &trace[d - 1] };
        let k = x - y;
        let down_x = safe_get(prev_v, max, k + 1);
        let right_x = safe_get(prev_v, max, k - 1);
        let came_from_insert = if k == -(d as isize) {
            true // Must have come from k+1, no k-1 exists at this edge
        } else if k == d as isize {
            // Must have come from k-1, no k+1 exists at this edge
            false
        } else {
            // We were in the middle, pick the one with the higher x
            right_x < down_x
        };

        let x_start = if came_from_insert {
            down_x
        } else {
            right_x + 1 // +1 because a Delete moves *one step right*
        };

        let y_start = x_start - k;
        while x > x_start && y > y_start {
            diffs.push(DiffOp::Equal(a[(x - 1) as usize].to_string()));
            x -= 1;
            y -= 1;
        }

        if x == 0 && y == 0 {
            break;
        }
        if came_from_insert {
            if y > 0 {
                diffs.push(DiffOp::Insert(b[(y - 1) as usize].to_string()));
                y -= 1;
            }
        } else if x > 0 {
            diffs.push(DiffOp::Delete(a[(x - 1) as usize].to_string()));
            x -= 1;
        }
    }

    while x > 0 && y > 0 {
        diffs.push(DiffOp::Equal(a[(x - 1) as usize].to_string()));
        x -= 1;
        y -= 1;
    }
    while x > 0 {
        diffs.push(DiffOp::Delete(a[(x - 1) as usize].to_string()));
        x -= 1;
    }
    while y > 0 {
        diffs.push(DiffOp::Insert(b[(y - 1) as usize].to_string()));
        y -= 1;
    }

    diffs.reverse();
    diffs
}

/// Safe index access for `v` that clamps out-of-range accesses.
/// Returns 0 if the index is invalid.
#[inline]
fn safe_get(v: &[isize], max: usize, k: isize) -> isize {
    let idx = (max as isize + k) as usize;
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
}
