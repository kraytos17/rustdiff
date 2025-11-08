mod cli;
mod diff;
mod fsio;

use clap::Parser;
use cli::Cli;
use diff::data::DiffStats;
use diff::modes::{diff_lines, diff_words};
use diff::render::{render_line_diff, render_unified_diff, render_word_diff};
use fsio::read_file;
use std::{fs::File, io::Write, process};

fn main() {
    let opts = Cli::parse();
    let old_text = read_or_exit(&opts.old_file);
    let new_text = read_or_exit(&opts.new_file);

    let diffs = if opts.word {
        diff_words(&old_text, &new_text)
    } else {
        diff_lines(&old_text, &new_text)
    };

    if opts.summary {
        let stats = DiffStats::from_ops(&diffs);
        println!(
            "Changes: +{}, -{} (total {})",
            stats.inserts, stats.deletes, stats.changes
        );
        return;
    }

    let rendered = if opts.word {
        render_word_diff(&diffs, opts.color)
    } else if opts.unified > 0 {
        render_unified_diff(&opts.old_file, &opts.new_file, &diffs)
    } else {
        render_line_diff(&diffs, opts.color)
    };

    let rendered = if opts.compact {
        compact_diff_output(&rendered)
    } else {
        rendered
    };

    let output_path = &opts.output;
    if let Err(e) = write_output(output_path, &rendered) {
        eprintln!("Error writing diff to {output_path}: {e}");
        process::exit(1);
    }

    println!("Diff written to {output_path}");
}

fn read_or_exit(path: &str) -> String {
    match read_file(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading {path}: {e}");
            process::exit(1);
        }
    }
}

/// Write rendered diff output to a file.
fn write_output(path: &str, contents: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())
}

/// Compact diff output by removing unchanged sections.
///
/// Keeps only:
/// - Lines starting with '+', '-'
/// - Diff headers: '@@', '---', '+++'
fn compact_diff_output(rendered: &str) -> String {
    rendered
        .lines()
        .filter(|line| {
            line.starts_with('+')
                || line.starts_with('-')
                || line.starts_with("@@")
                || line.starts_with("---")
                || line.starts_with("+++")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
