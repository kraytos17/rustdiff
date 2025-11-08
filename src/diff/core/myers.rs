use crate::diff::data::DiffOp;

/// Compute the minimal edit script between two slices using Myers' O(ND) diff algorithm.
///
/// # Parameters
/// - `a`: old sequence
/// - `b`: new sequence
///
/// # Returns
/// A vector of `DiffOp` representing insertions, deletions, and equal elements.
///
/// # Notes
/// Uses safe indexing and works for both line and word-level diffing.
pub fn compute_diff(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let n = a.len();
    let m = b.len();
    let max = n + m;
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
            while x < n as isize && y < m as isize && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }

            v[index] = x;
            if x == n as isize && y == m as isize {
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
        let prev_v = if d == 0 { &zero_vec } else { &trace[d - 1] };
        let k = x - y;
        let k_clamped = k.clamp(-(d as isize), d as isize);
        let down_x = safe_get(prev_v, max, k_clamped + 1);
        let right_x = safe_get(prev_v, max, k_clamped - 1);

        let came_from_insert = if k == -(d as isize) {
            true
        } else if k == d as isize {
            false
        } else {
            right_x < down_x
        };

        let (prev_k, x_start) = if came_from_insert {
            (k + 1, down_x)
        } else {
            (k - 1, right_x + 1)
        };

        let y_start = x_start - prev_k;

        // Trace equal diagonal elements safely
        while x > x_start && y > y_start && x > 0 && y > 0 {
            diffs.push(DiffOp::Equal(a[(x - 1) as usize].to_string()));
            x -= 1;
            y -= 1;
        }

        // Handle deletions
        if x > x_start && x > 0 {
            diffs.push(DiffOp::Delete(a[(x - 1) as usize].to_string()));
            x -= 1;
        }

        // Handle insertions
        if y > y_start && y > 0 {
            diffs.push(DiffOp::Insert(b[(y - 1) as usize].to_string()));
            y -= 1;
        }
    }

    // Handle any remaining items
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
    let idx = max.wrapping_add_signed(k);
    v.get(idx).copied().unwrap_or(0)
}
