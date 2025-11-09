mod cli;
mod diff;
mod fsio;

use crate::diff::data::DiffStats;
use crate::diff::modes::{diff_lines, diff_words};
use crate::diff::render::{
    render_line_diff, render_unified_diff, render_word_diff, write_diff_outputs,
};
use clap::Parser;
use cli::Cli;
use fsio::read_file;
use std::{fs::File, io::Write, process};

fn main() {
    let opts = Cli::parse();
    let old_text = read_or_exit(&opts.old_file);
    let new_text = read_or_exit(&opts.new_file);

    if opts.word && opts.unified.is_some() {
        eprintln!("Error: --word and --unified flags cannot be used together.");
        eprintln!("The --unified format is strictly line-based.");
        process::exit(1);
    }
    if opts.word && opts.compact {
        eprintln!("Error: --compact mode is not compatible with --word diff.");
        eprintln!("Tip: --compact only filters unchanged *lines*, not words.");
        process::exit(1);
    }

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
    } else if let Some(context_lines) = opts.unified {
        render_unified_diff(
            &opts.old_file,
            &opts.new_file,
            &diffs,
            context_lines,
            opts.color,
        )
    } else if opts.compact {
        render_unified_diff(&opts.old_file, &opts.new_file, &diffs, 0, opts.color)
    } else {
        render_line_diff(&diffs, opts.color)
    };

    let output_path = &opts.output;
    if let Err(e) = write_output(output_path, &rendered) {
        eprintln!("Error writing diff to {output_path}: {e}");
        process::exit(1);
    }
    if opts.html {
        let base_name = output_path.trim_end_matches(".diff");
        if let Err(e) = write_diff_outputs(&rendered, base_name) {
            eprintln!("Error generating HTML diff: {e}");
        } else {
            println!("HTML diff exported to {base_name}.html");
        }
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

fn write_output(path: &str, contents: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())
}

// /// Compact diff output by removing unchanged sections.
// ///
// /// Keeps only:
// /// - Lines starting with '+', '-'
// /// - Diff headers: '@@', '---', '+++'
// fn compact_diff_output(rendered: &str) -> String {
//     rendered
//         .lines()
//         .filter(|line| {
//             line.starts_with('+')
//                 || line.starts_with('-')
//                 || line.starts_with("@@")
//                 || line.starts_with("---")
//                 || line.starts_with("+++")
//         })
//         .collect::<Vec<_>>()
//         .join("\n")
// }
