//! Core diff algorithms and shared helpers.

/// Histogram (patience-style) anchor diff, the default algorithm.
pub mod histogram;
/// Linear-space Myers diff.
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

fn trim_common_ends<'a>(
    a: &'a [&'a str],
    b: &'a [&'a str],
) -> (usize, usize, &'a [&'a str], &'a [&'a str]) {
    let start = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suffix = a[start..]
        .iter()
        .rev()
        .zip(b[start..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();

    (
        start,
        suffix,
        &a[start..a.len() - suffix],
        &b[start..b.len() - suffix],
    )
}
