use clap::Parser;
use rustdiff::cli::Cli;
use rustdiff::cli::ColorMode;
use rustdiff::diff::data::DiffStats;
use rustdiff::diff::modes::{diff_lines, diff_words};
use rustdiff::diff::render::html::{
    render_side_by_side_html, render_unified_html, render_word_html,
};
use rustdiff::diff::render::{render_line_diff, render_unified_diff, render_word_diff};
use rustdiff::fsio::{Source, read_file};
use std::{
    fs::File,
    io::{self, IsTerminal, Write},
    process,
};

fn main() {
    let opts = Cli::parse();
    let old = read_or_exit(&opts.old_file);
    let new = read_or_exit(&opts.new_file);
    let old_text = source_str_or_exit(&old, &opts.old_file);
    let new_text = source_str_or_exit(&new, &opts.new_file);

    let is_tty = stdout_is_terminal();
    let is_stdout = opts.output == "-";
    let use_color = match opts.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => is_stdout && is_tty,
    };

    let diff = if opts.word {
        diff_words(old_text, new_text, opts.diff_algorithm)
    } else {
        diff_lines(old_text, new_text, opts.diff_algorithm)
    };

    if opts.summary {
        let stats = DiffStats::from_ops(&diff.ops);
        println!(
            "Changes: +{}, -{} (total {})",
            stats.inserts, stats.deletes, stats.changes
        );
        return;
    }

    let rendered = if opts.word {
        if opts.unified.is_some() || opts.compact {
            render_unified_diff(
                &opts.old_file,
                &opts.new_file,
                &diff,
                opts.unified.unwrap_or(0),
                use_color,
            )
        } else {
            render_word_diff(&diff, use_color)
        }
    } else if let Some(context_lines) = opts.unified {
        render_unified_diff(
            &opts.old_file,
            &opts.new_file,
            &diff,
            context_lines,
            use_color,
        )
    } else if opts.compact {
        render_unified_diff(&opts.old_file, &opts.new_file, &diff, 0, use_color)
    } else {
        render_line_diff(&diff, use_color)
    };

    let output_path = &opts.output;
    if let Err(e) = write_output(output_path, &rendered) {
        eprintln!("Error writing diff to {output_path}: {e}");
        process::exit(1);
    }
    if opts.html {
        let base_name = output_path.trim_end_matches(".diff");
        let html = if opts.side_by_side {
            render_side_by_side_html(&diff, &opts.old_file, &opts.new_file, opts.html_theme)
        } else if opts.word {
            render_word_html(&diff, opts.html_theme)
        } else {
            render_unified_html(
                &diff,
                opts.unified.unwrap_or(3),
                &opts.old_file,
                &opts.new_file,
                opts.html_theme,
            )
        };

        let html_path = format!("{base_name}.html");
        if let Err(e) = std::fs::write(&html_path, html) {
            eprintln!("Error generating HTML diff: {e}");
            process::exit(1);
        } else {
            println!("HTML diff exported to {html_path}");
        }
    }
    if opts.output != "-" {
        println!("Diff written to {output_path}");
    }
}

fn read_or_exit(path: &str) -> Source {
    match read_file(path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error reading {path}: {e}");
            process::exit(1);
        }
    }
}

fn source_str_or_exit<'a>(source: &'a Source, path: &'a str) -> &'a str {
    match source.as_str() {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error reading {path}: {e}");
            process::exit(1);
        }
    }
}

fn write_output(path: &str, contents: &str) -> io::Result<()> {
    if path == "-" {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(contents.as_bytes())
    } else {
        let mut file = File::create(path)?;
        file.write_all(contents.as_bytes())
    }
}

fn stdout_is_terminal() -> bool {
    io::stdout().is_terminal()
}
