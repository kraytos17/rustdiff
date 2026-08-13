pub mod html;
pub mod line;
pub mod unified;
pub mod word;

pub use html::{
    HtmlTheme, render_numbered_html, render_side_by_side_html, render_unified_html,
    render_word_html,
};
pub use line::render_line_diff;
pub use unified::render_unified_diff;
pub use word::render_word_diff;
