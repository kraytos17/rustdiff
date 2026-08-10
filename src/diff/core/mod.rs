pub mod histogram;
pub mod myers;

pub use histogram::compute_histogram_diff;

/// A matching run: `a[x..u] == b[y..v]`.
#[derive(Debug, Clone, Copy)]
pub(super) struct Snake {
    pub x: usize,
    pub y: usize,
    pub u: usize,
    pub v: usize,
}

impl Snake {
    pub(super) const fn len(&self) -> usize {
        self.u - self.x
    }
}

#[allow(clippy::suspicious_operation_groupings)]
fn trim_common_ends<'a>(
    a: &'a [&'a str],
    b: &'a [&'a str],
) -> (usize, usize, &'a [&'a str], &'a [&'a str]) {
    let mut start = 0;
    while start < a.len() && start < b.len() && a[start] == b[start] {
        start += 1;
    }

    let mut end_a = a.len();
    let mut end_b = b.len();
    while end_a > start && end_b > start && a[end_a - 1] == b[end_b - 1] {
        end_a -= 1;
        end_b -= 1;
    }
    (start, a.len() - end_a, &a[start..end_a], &b[start..end_b])
}
