//! # rustdiff
//!
//! A high-performance, pure Rust diff engine and CLI. It computes line- and
//! word-level diffs with a linear-space Myers core and a histogram anchor
//! heuristic, and renders them as plain, unified, or word-inline text with
//! optional ANSI color, or as self-contained HTML pages.
//!
//! The crate is split into three public modules:
//!
//! - [`diff`] — the diff engine: tokenization modes, `u32` interning, the core
//!   algorithms, and the text/HTML renderers.
//! - [`cli`] — the clap-derived command-line interface used by the `rustdiff`
//!   binary.
//! - [`fsio`] — memory-mapped or buffered file loading.
#![deny(missing_docs)]

/// The clap-derived command-line interface used by the `rustdiff` binary.
pub mod cli;
/// The diff engine: tokenization, interning, algorithms, and renderers.
pub mod diff;
/// Memory-mapped or buffered file loading.
pub mod fsio;
