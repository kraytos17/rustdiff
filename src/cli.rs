use clap::{ArgAction, Parser, ValueHint};

#[derive(Parser, Debug)]
#[command(
    author = "Soumil Kumar",
    version,
    about = "A high-performance, pure Rust diff generator",
    long_about = None,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[arg(
        value_name = "OLD",
        value_hint = ValueHint::FilePath,
        help = "Path to the old/original file"
    )]
    pub old_file: String,

    #[arg(
        value_name = "NEW",
        value_hint = ValueHint::FilePath,
        help = "Path to the new/modified file"
    )]
    pub new_file: String,

    #[arg(
        short,
        long,
        action = ArgAction::SetTrue,
        help = "Enable colorized diff output"
    )]
    pub color: bool,

    #[arg(
        short = 'u',
        long = "unified",
        default_value_t = 3,
        value_name = "N",
        help = "Number of context lines to display (default: 3)"
    )]
    pub unified: usize,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Hide unchanged lines in output"
    )]
    pub compact: bool,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "Show a summary instead of full diff output"
    )]
    pub summary: bool,
}
