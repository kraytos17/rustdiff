#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffOp<'a> {
    Equal(&'a str),
    Insert(&'a str),
    Delete(&'a str),
}

pub fn compute_diff<'a>(a: &'a [&'a str], b: &'a [&'a str]) -> Vec<DiffOp<'a>> {
    let n = a.len();
    let m = b.len();
    let max = n + m;
    let mut v = vec![0isize; 2 * max + 1];
    let mut trace = Vec::with_capacity(max);

    for d in 0..=max {
        trace.push(v.clone());
        for k in (-(d as isize)..=d as isize).step_by(2) {
            let index = (max as isize + k) as usize;
            let x_start: isize;

            if k == -(d as isize)
                || (k != d as isize
                    && v[(max as isize + k - 1) as usize] <= v[(max as isize + k + 1) as usize])
            {
                x_start = v[(max as isize + k + 1) as usize];
            } else {
                x_start = v[(max as isize + k - 1) as usize] + 1;
            }

            let mut x = x_start;
            let mut y = x - k;

            while x < n as isize && y < m as isize && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }

            v[index] = x;

            if x == n as isize && y == m as isize {
                return backtrack(&trace, a, b);
            }
        }
    }

    unreachable!()
}

fn backtrack<'a>(trace: &Vec<Vec<isize>>, a: &'a [&'a str], b: &'a [&'a str]) -> Vec<DiffOp<'a>> {
    let n = a.len();
    let m = b.len();
    let max = n + m;
    let mut x = n as isize;
    let mut y = m as isize;
    let mut result = Vec::with_capacity(n + m);

    for (d, v) in trace.iter().enumerate().rev() {
        let k = x - y;
        let index = (max as isize + k) as usize;
        let (prev_k, prev_x) = if k == -(d as isize)
            || (k != d as isize && v[(index - 1) as usize] < v[(index + 1) as usize])
        {
            let pk = k + 1;
            (pk, v[(max as isize + pk) as usize])
        } else {
            let pk = k - 1;
            (pk, v[(max as isize + pk) as usize] + 1)
        };

        let prev_y = prev_x - prev_k;
        while x > prev_x && y > prev_y {
            x -= 1;
            y -= 1;
            result.push(DiffOp::Equal(a[x as usize]));
        }

        if x == prev_x {
            y -= 1;
            if y >= 0 {
                result.push(DiffOp::Insert(b[y as usize]));
            }
        } else {
            x -= 1;
            if x >= 0 {
                result.push(DiffOp::Delete(a[x as usize]));
            }
        }
    }

    result.reverse();
    result
}
