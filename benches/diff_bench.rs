use criterion::{Criterion, criterion_group, criterion_main};
use rustdiff::diff::modes::{DiffAlgorithm, diff_lines, diff_words};

mod fixtures {
    use std::fmt::Write as _;

    /// ~2,000 lines with a handful of changes (typical source-diff shape).
    pub fn typical() -> (String, String) {
        let mut old = String::with_capacity(64_000);
        let mut new = String::with_capacity(64_000);
        for i in 0..2000 {
            writeln!(old, "line {i} unchanged").unwrap();
            if i == 500 || i == 1000 || i == 1500 {
                writeln!(new, "line {i} MODIFIED").unwrap();
            } else {
                writeln!(new, "line {i} unchanged").unwrap();
            }
        }
        (old, new)
    }

    /// No shared lines — patience falls through to Myers on the whole input.
    pub fn rewritten() -> (String, String) {
        let mut old = String::with_capacity(64_000);
        let mut new = String::with_capacity(64_000);
        for i in 0..2000 {
            writeln!(old, "old token {i}").unwrap();
            writeln!(new, "new token {i}").unwrap();
        }
        (old, new)
    }

    /// Highly repetitive input (generated config / log shape).
    pub fn repetitive() -> (String, String) {
        let block_old = "key = value\nkey = value\nother = thing\n";
        let block_new = "key = value\nother = thing\nother = thing\n";
        (block_old.repeat(1000), block_new.repeat(1000))
    }

    /// One long single line (minified-style) for word-mode.
    pub fn minified() -> (String, String) {
        let mut old = String::new();
        let mut new = String::new();
        write!(old, "{}wold", "word ".repeat(3000)).unwrap();
        writeln!(new, "{}wnew", "word ".repeat(3000)).unwrap();
        (old, new)
    }
}

fn bench_diff(c: &mut Criterion) {
    let (typical_old, typical_new) = fixtures::typical();
    let (rewritten_old, rewritten_new) = fixtures::rewritten();
    let (repetitive_old, repetitive_new) = fixtures::repetitive();
    let (min_old, min_new) = fixtures::minified();

    for algorithm in [DiffAlgorithm::Histogram, DiffAlgorithm::Myers] {
        let label = match algorithm {
            DiffAlgorithm::Histogram => "histogram",
            DiffAlgorithm::Myers => "myers",
        };
        c.benchmark_group(format!("diff_lines/{label}"))
            .bench_function("typical", |b| {
                b.iter(|| diff_lines(&typical_old, &typical_new, algorithm));
            })
            .bench_function("rewritten", |b| {
                b.iter(|| diff_lines(&rewritten_old, &rewritten_new, algorithm));
            })
            .bench_function("repetitive", |b| {
                b.iter(|| diff_lines(&repetitive_old, &repetitive_new, algorithm));
            });
    }

    c.benchmark_group("diff_words/histogram")
        .bench_function("minified", |b| {
            b.iter(|| diff_words(&min_old, &min_new, DiffAlgorithm::Histogram));
        });
    c.benchmark_group("diff_words/myers")
        .bench_function("minified", |b| {
            b.iter(|| diff_words(&min_old, &min_new, DiffAlgorithm::Myers));
        });
}

criterion_group!(benches, bench_diff);
criterion_main!(benches);
