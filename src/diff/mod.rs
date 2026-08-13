//! The diff engine: tokenization modes, `u32` interning, core algorithms, and
//! text/HTML renderers.

/// Core diff algorithms (linear-space Myers + histogram anchoring) and shared
/// helpers.
pub mod core;
/// Core data types: `Op`, `Diff`, `Hunk`, `DiffStats`.
pub mod data;
/// String-to-`u32` interning so the core compares dense IDs instead of text.
pub mod intern;
/// Tokenization modes and diff options.
pub mod modes;
/// Text (line/unified/word) and HTML renderers.
pub mod render;
