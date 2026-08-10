//! Memory-regression guard for the linear-space Myers rewrite.
//!
//! The full-trace implementation was O(D·(N+M)) — for 5,000 vs 5,000
//! all-different lines that means ~1.6 GB of `trace` rows. The linear-space
//! rewrite must stay O(N+M), i.e. a few MB.
//!
//! ```sh
//! cargo test --test memory -- --test-threads=1
//! ```

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use rustdiff::diff::core::myers::compute_diff;

#[test]
fn myers_memory_stays_linear() {
    let a_strs: Vec<String> = (0..5000).map(|i| format!("a{i}")).collect();
    let b_strs: Vec<String> = (0..5000).map(|i| format!("b{i}")).collect();
    let a: Vec<&str> = a_strs.iter().map(String::as_str).collect();
    let b: Vec<&str> = b_strs.iter().map(String::as_str).collect();

    let _profiler = dhat::Profiler::new_heap();
    let ops = compute_diff(&a, &b);
    assert_eq!(
        ops.len(),
        2,
        "all-different diff should be delete-run + insert-run"
    );

    let stats = dhat::HeapStats::get();
    assert!(
        stats.max_bytes < 256 * 1024 * 1024,
        "peak heap {} MB suggests O(D·(N+M)) regressed; expected O(N+M)",
        stats.max_bytes / (1024 * 1024)
    );
}
