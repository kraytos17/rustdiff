//! Text and HTML renderers for computed diffs.

/// HTML renderers: self-contained pages with view-time JavaScript.
pub mod html;
/// Simple line renderer with `-`/`+` markers.
pub mod line;
/// Git-style unified renderer with hunks and context lines.
pub mod unified;
/// Inline word renderer with `[-old+new]` replacement markers.
pub mod word;

pub use html::{
    HtmlTheme, render_numbered_html, render_side_by_side_html, render_unified_html,
    render_word_html,
};
pub use line::render_line_diff;
pub use unified::render_unified_diff;
pub use word::render_word_diff;
