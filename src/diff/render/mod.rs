pub mod line;
pub mod unified;
pub mod word;

pub use line::render_line_diff;
pub use unified::render_unified_diff;
pub use word::render_word_diff;
