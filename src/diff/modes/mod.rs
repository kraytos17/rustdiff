pub mod line;
pub mod word;

pub use line::diff_lines;
pub use word::diff_words;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DiffAlgorithm {
    Histogram,
    Myers,
}
