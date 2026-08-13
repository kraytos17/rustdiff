#![no_main]

//! Round-trip fuzz target over the interner + diff cores.
//!
//! Splits arbitrary bytes into two token sequences and asserts that both the
//! histogram and Myers paths always produce a valid edit script (no panics, no
//! out-of-bounds indexing, and `Diff::validate_round_trip` holds).
//!
//! Requires nightly + cargo-fuzz:
//!
//! ```sh
//! cargo install cargo-fuzz
//! cargo +nightly fuzz run diff_round_trip
//! ```

use libfuzzer_sys::fuzz_target;
use rustdiff::diff::core::{compute_histogram_diff, myers::compute_diff};
use rustdiff::diff::data::{Diff, Op};

fn split_tokens(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|&b| b == b'\n')
        .map(|line| String::from_utf8_lossy(line).into_owned())
        .collect()
}

fn assert_round_trip(a: &[&str], b: &[&str], ops: &[Op]) {
    let diff = Diff {
        ops: ops.to_vec(),
        old_tokens: a.iter().map(ToString::to_string).collect(),
        new_tokens: b.iter().map(ToString::to_string).collect(),
    };
    assert!(diff.validate_round_trip(a, b), "invalid round-trip");
}

fuzz_target!(|data: &[u8]| {
    let mid = data.len() / 2;
    let a_tokens = split_tokens(&data[..mid]);
    let b_tokens = split_tokens(&data[mid..]);
    let a: Vec<&str> = a_tokens.iter().map(String::as_str).collect();
    let b: Vec<&str> = b_tokens.iter().map(String::as_str).collect();

    let histogram = compute_histogram_diff(&a, &b);
    assert_round_trip(&a, &b, &histogram);

    let myers = compute_diff(&a, &b);
    assert_round_trip(&a, &b, &myers);
});
