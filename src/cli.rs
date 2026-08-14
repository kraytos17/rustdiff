use crate::diff::modes::DiffAlgorithm;
use crate::diff::render::html::HtmlTheme;
use clap::{ArgAction, ArgGroup, Parser, ValueEnum, ValueHint};

/// A high-performance, human-readable diff generator written in pure Rust.
///
/// Supports line and word-level modes, colorized output, compact diffs,
/// and unified diff formatting with context lines.
#[derive(Parser, Debug)]
#[command(
    author = "Soumil Kumar",
    version,
    about = "A high-performance, pure Rust diff generator",
    disable_help_subcommand = true,
    group(
        ArgGroup::new("output_mode")
            .args(["unified", "compact", "summary"])
            .multiple(false)
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

    /// When to use terminal colors (ANSI)
    #[arg(
        long,
        value_enum,
        default_value = "auto",
        help = "When to use terminal colors: auto, always, or never"
    )]
    pub color: ColorMode,

    /// Diff algorithm to use
    #[arg(
        long,
        value_enum,
        default_value = "histogram",
        help = "Diff algorithm: histogram (default) or myers"
    )]
    pub diff_algorithm: DiffAlgorithm,

    /// HTML export options
    #[command(flatten)]
    pub html: HtmlArgs,

    /// Output format: unified, compact, or summary
    #[command(flatten)]
    pub format: OutputArgs,

    /// Use word-level diff instead of line-level
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Use word-level diff instead of line-level"
    )]
    pub word: bool,

    /// Process/I-O behavior toggles
    #[command(flatten)]
    pub behavior: BehaviorArgs,

    /// Ignore-* normalization flags
    #[command(flatten)]
    pub ignore: IgnoreArgs,

    /// Cap on the Myers edit distance per region
    #[arg(
        long,
        value_name = "N",
        help = "Degrade regions whose Myers edit distance would exceed N to a full delete+insert (off by default)"
    )]
    pub max_edit_distance: Option<u32>,
}

/// HTML export options (`--html`, `--side-by-side`, `--html-theme`, `--html-output`).
#[derive(clap::Args, Debug)]
pub struct HtmlArgs {
    /// Export the diff as HTML
    #[arg(id = "html", long, help = "Generate colorized HTML diff output")]
    pub enabled: bool,

    /// Generate side-by-side HTML diff (implies --html)
    #[arg(
        long,
        help = "Render a side-by-side HTML diff (requires --html)",
        requires = "html",
        conflicts_with_all = ["word"]
    )]
    pub side_by_side: bool,

    /// HTML color theme (default: follow the viewer's OS preference)
    #[arg(
        long,
        value_enum,
        help = "HTML color theme: dark or light (default: follow the viewer's OS preference)"
    )]
    pub theme: Option<HtmlTheme>,

    /// Explicit HTML output path (default: derived from --output)
    #[arg(
        id = "html_output",
        long = "html-output",
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
        help = "Write the HTML diff here instead of deriving the path from --output"
    )]
    pub output: Option<String>,
}

/// Output format: unified, compact, or summary.
#[derive(clap::Args, Debug)]
pub struct OutputArgs {
    /// Number of context lines to display in unified mode
    #[arg(
        short = 'u',
        long = "unified",
        value_name = "N",
        help = "Show unified diff with N context lines"
    )]
    pub unified: Option<usize>,

    /// Hide unchanged lines (compact diff)
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
        help = "Show a summary (insertions/deletions only)"
    )]
    pub summary: bool,
}

/// Process and I/O behavior toggles.
#[derive(clap::Args, Debug)]
pub struct BehaviorArgs {
    /// Exit with 0 if no differences, 1 if differences, 2 on error (POSIX diff)
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Exit 0 if no differences, 1 if differences found, 2 on error"
    )]
    pub exit_code: bool,

    /// Disable memory-mapping for file input
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Read files into memory instead of memory-mapping large files"
    )]
    pub no_mmap: bool,

    /// Verify the diff is reversible before writing output
    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Verify the computed diff is reversible (round-trip check)"
    )]
    pub verify: bool,
}

/// Flags that normalize tokens before diffing.
#[derive(clap::Args, Debug)]
pub struct IgnoreArgs {
    /// Ignore whitespace differences within tokens
    #[arg(
        short = 'w',
        long = "ignore-whitespace",
        action = ArgAction::SetTrue,
        help = "Ignore whitespace differences within tokens (line and word mode)"
    )]
    pub whitespace: bool,

    /// Ignore case differences between tokens
    #[arg(
        short = 'i',
        long = "ignore-case",
        action = ArgAction::SetTrue,
        help = "Ignore case differences when comparing tokens"
    )]
    pub case: bool,

    /// Ignore blank-line changes (line mode)
    #[arg(
        short = 'B',
        long = "ignore-blank-lines",
        action = ArgAction::SetTrue,
        help = "Treat all blank lines as identical, so blank-line changes are ignored (line mode)"
    )]
    pub blank_lines: bool,
}

/// When to use ANSI terminal colors.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ColorMode {
    /// Color only when writing to a terminal.
    Auto,
    /// Always emit color codes.
    Always,
    /// Never emit color codes.
    Never,
}
