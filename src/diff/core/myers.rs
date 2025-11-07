use crate::diff::data::DiffOp;

/// Compute the minimal edit script between two slices using Myers' O(ND) diff algorithm.
///
/// # Parameters
/// - `a`: old sequence
/// - `b`: new sequence
///
/// # Returns
/// A vector of `DiffOp` representing insertions, deletions, and equal elements.
pub fn compute_diff(a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let n = a.len();
    let m = b.len();
    let max = n + m;
    let mut v = vec![0isize; 2 * max + 1];
    let mut trace = Vec::new();

    // Forward pass: build edit graph frontier
    for d in 0..=max {
        trace.push(v.clone());
        for k in (-(d as isize)..=d as isize).step_by(2) {
            let index = (max as isize + k) as usize;

            // Choose whether to go "down" (insert) or "right" (delete)
            let x_start = if k == -(d as isize)
                || (k != d as isize
                    && v[(max as isize + k - 1) as usize] < v[(max as isize + k + 1) as usize])
            {
                v[(max as isize + k + 1) as usize] // down (insert)
            } else {
                v[(max as isize + k - 1) as usize] + 1 // right (delete)
            };

            let mut x = x_start;
            let mut y = x - k;

            // Follow diagonal (equal items)
            while x < n as isize && y < m as isize && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }

            v[index] = x;

            // ✅ Found the end of the path (exact, not >=)
            if x == n as isize && y == m as isize {
                return backtrack(&trace, a, b);
            }
        }
    }

    unreachable!("Myers diff algorithm failed — this should never happen");
}

fn backtrack(trace: &[Vec<isize>], a: &[&str], b: &[&str]) -> Vec<DiffOp> {
    let n = a.len();
    let m = b.len();
    let max = n + m;
    let mut x = n as isize;
    let mut y = m as isize;
    let mut diffs = Vec::new();

    for (d, v) in trace.iter().enumerate().rev() {
        let k = x - y;
        let index = (max as isize + k) as usize;
        let (prev_k, x_start);
        if k == -(d as isize) || (k != d as isize && v[index - 1] < v[index + 1]) {
            prev_k = k + 1;
            x_start = v[(max as isize + prev_k) as usize];
        } else {
            prev_k = k - 1;
            x_start = v[(max as isize + prev_k) as usize] + 1;
        }

        let y_start = x_start - prev_k;
        let x_mid = x_start;
        let y_mid = y_start;

        // Follow diagonal back (equal lines)
        while x > x_mid && y > y_mid {
            diffs.push(DiffOp::Equal(a[(x - 1) as usize].to_string()));
            x -= 1;
            y -= 1;
        }

        // Insert / delete step
        if x_mid < x {
            diffs.push(DiffOp::Delete(a[(x_mid) as usize].to_string()));
            x -= 1;
        } else if y_mid < y {
            diffs.push(DiffOp::Insert(b[(y_mid) as usize].to_string()));
            y -= 1;
        }
    }

    diffs.reverse();
    diffs
}
