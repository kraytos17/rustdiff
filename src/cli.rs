use clap::{ArgAction, ArgGroup, Parser, ValueHint};

/// A high-performance, human-readable diff generator written in pure Rust.
///
/// Supports line- and word-level modes, colorized output, compact diffs,
/// and unified diff formatting with context lines.
#[derive(Parser, Debug)]
#[command(
    author = "Soumil Kumar",
    version,
    about = "A high-performance, pure Rust diff generator",
    long_about = None,
    disable_help_subcommand = true,
    group(
        ArgGroup::new("mode")
            .args(["line", "word"])
            .required(false)
    )
)]
pub struct Cli {
    /// Path to the old/original file
    #[arg(
        value_name = "OLD",
        value_hint = ValueHint::FilePath,
        help = "Path to the old/original file"
    )]
    pub old_file: String,

    /// Path to the new/modified file
    #[arg(
        value_name = "NEW",
        value_hint = ValueHint::FilePath,
        help = "Path to the new/modified file"
    )]
    pub new_file: String,

    /// Output diff file (default: changes.diff)
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "changes.diff",
        value_hint = ValueHint::FilePath,
        help = "Write diff output to this file (default: changes.diff)"
    )]
    pub output: String,

    /// Enable colorized diff output (ANSI colors)
    #[arg(
        short,
        long,
        action = ArgAction::SetTrue,
        help = "Enable colorized diff output (ANSI colors)"
    )]
    pub color: bool,

    /// Number of context lines to display in unified mode
    #[arg(
        short = 'u',
        long = "unified",
        value_name = "N",
        help = "Number of context lines in unified diff"
    )]
    pub unified: Option<usize>,

    /// Hide unchanged lines in output (compact mode)
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Hide unchanged lines in output (compact diff)"
    )]
    pub compact: bool,

    /// Show only a summary of changes instead of full diff
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Show a summary (number of insertions/deletions)"
    )]
    pub summary: bool,

    /// Use word-level diff instead of line-level
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Use word-level diff instead of line-level"
    )]
    pub word: bool,

    /// Use line-level diff (default)
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Use line-level diff (default mode)"
    )]
    pub line: bool,
}
