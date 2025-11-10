mod cli;
mod diff;
mod fsio;

use crate::cli::ColorMode;
use crate::diff::data::DiffStats;
use crate::diff::modes::{diff_lines, diff_words};
use crate::diff::render::{
    render_diff_outputs, render_line_diff, render_side_by_side_html, render_unified_diff,
    render_word_diff,
};
use clap::Parser;
use cli::Cli;
use fsio::read_file;
use std::{
    fs::File,
    io::{self, Write},
    process,
};

fn main() {
    let opts = Cli::parse();
    let old_text = read_or_exit(&opts.old_file);
    let new_text = read_or_exit(&opts.new_file);

    let is_tty = stdout_is_terminal();
    let is_stdout = opts.output == "-";
    let use_color = match opts.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => is_stdout && is_tty && !opts.html,
    };

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
        render_word_diff(&diffs, use_color)
    } else if let Some(context_lines) = opts.unified {
        render_unified_diff(
            &opts.old_file,
            &opts.new_file,
            &diffs,
            context_lines,
            use_color,
        )
    } else if opts.compact {
        render_unified_diff(&opts.old_file, &opts.new_file, &diffs, 0, use_color)
    } else {
        render_line_diff(&diffs, use_color)
    };

    let output_path = &opts.output;
    if let Err(e) = write_output(output_path, &rendered) {
        eprintln!("Error writing diff to {output_path}: {e}");
        process::exit(1);
    }

    if opts.html {
        let base_name = output_path.trim_end_matches(".diff");
        if opts.side_by_side {
            if let Err(e) = render_side_by_side_html(&rendered, base_name) {
                eprintln!("Error generating side-by-side HTML diff: {e}");
            } else {
                println!("Side-by-side HTML diff exported to {base_name}_side_by_side.html");
            }
        } else if let Err(e) = render_diff_outputs(&rendered, base_name) {
            eprintln!("Error generating HTML diff: {e}");
        } else {
            println!("HTML diff exported to {base_name}.html");
        }
    }

    if !opts.side_by_side {
        println!("Diff written to {output_path}");
    }
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

fn write_output(path: &str, contents: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())
}

fn stdout_is_terminal() -> bool {
    use std::os::unix::io::AsRawFd;
    unsafe { libc::isatty(io::stdout().as_raw_fd()) == 1 }
}
