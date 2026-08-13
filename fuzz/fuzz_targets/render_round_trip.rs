#![no_main]
#![allow(unused_must_use)]

//! Renderer fuzz target over the full diff -> render pipeline.
//!
//! Splits arbitrary bytes into two texts and runs every terminal and HTML
//! renderer over the resulting diffs (both algorithms, both modes, both
//! themes). The assertion is simply "no panic": ops come from the cores on the
//! same tokens, so slicing is always in bounds; a renderer panic on adversarial
//! input is a bug (this complements `diff_round_trip`, which only exercises the
//! cores).
//!
//! Requires nightly + cargo-fuzz:
//!
//! ```sh
//! cargo +nightly fuzz run render_round_trip
//! ```

use libfuzzer_sys::fuzz_target;
use rustdiff::diff::modes::{DiffAlgorithm, diff_lines, diff_words};
use rustdiff::diff::render::html::{
    HtmlTheme, render_numbered_html, render_side_by_side_html, render_unified_html,
    render_word_html,
};
use rustdiff::diff::render::{render_line_diff, render_unified_diff, render_word_diff};

fuzz_target!(|data: &[u8]| {
    let mid = data.len() / 2;
    let old = String::from_utf8_lossy(&data[..mid]);
    let new = String::from_utf8_lossy(&data[mid..]);

    for algorithm in [DiffAlgorithm::Histogram, DiffAlgorithm::Myers] {
        let line_diff = diff_lines(&old, &new, algorithm).unwrap();
        let word_diff = diff_words(&old, &new, algorithm).unwrap();

        render_line_diff(&line_diff, false);
        render_line_diff(&line_diff, true);
        render_unified_diff("old", "new", &line_diff, 0, false);
        render_unified_diff("old", "new", &line_diff, 3, true);
        render_unified_diff("old", "new", &line_diff, 10, false);
        render_word_diff(&word_diff, false);
        render_word_diff(&word_diff, true);

        render_numbered_html(&line_diff, Some(HtmlTheme::Dark));
        render_numbered_html(&line_diff, None);
        render_unified_html(&line_diff, 3, "old", "new", Some(HtmlTheme::Dark));
        render_unified_html(&line_diff, 3, "old", "new", Some(HtmlTheme::Light));
        render_side_by_side_html(&line_diff, "old", "new", Some(HtmlTheme::Dark));
        render_word_html(&word_diff, Some(HtmlTheme::Dark));
        render_word_html(&word_diff, Some(HtmlTheme::Light));
    }
});
