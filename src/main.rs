use clap::Parser;
use rustdiff::cli::{Cli, ColorMode};
use rustdiff::diff::data::{Diff, DiffStats, OpKind};
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
    process::exit(match run(&opts) {
        Ok(has_changes) if opts.exit_code && has_changes => 1,
        Ok(_) => 0,
        Err(message) => {
            eprintln!("{message}");
            2
        }
    });
}

/// Full CLI pipeline: read inputs, diff, render, write. Returns whether the
/// inputs differed (drives the `--exit-code` status) and propagates errors as
/// `Err(message)` (drives exit code 2).
fn run(opts: &Cli) -> Result<bool, String> {
    let old = read_source(&opts.old_file, !opts.no_mmap)?;
    let new = read_source(&opts.new_file, !opts.no_mmap)?;
    let old_text = source_str(&old, &opts.old_file)?;
    let new_text = source_str(&new, &opts.new_file)?;
    let diff = if opts.word {
        diff_words(old_text, new_text, opts.diff_algorithm)
    } else {
        diff_lines(old_text, new_text, opts.diff_algorithm)
    }?;

    let has_changes = diff.ops.iter().any(|op| op.kind != OpKind::Equal);
    if opts.summary {
        let stats = DiffStats::from_ops(&diff.ops);
        println!(
            "Changes: +{}, -{} (total {})",
            stats.inserts, stats.deletes, stats.changes
        );
        return Ok(has_changes);
    }

    let use_color = match opts.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => opts.output == "-" && stdout_is_terminal(),
    };

    let output_path = &opts.output;
    write_output(output_path, &render_text(opts, &diff, use_color))
        .map_err(|e| format!("Error writing diff to {output_path}: {e}"))?;

    if opts.html {
        let html_path = format!("{}.html", html_base(output_path));
        std::fs::write(&html_path, render_html(opts, &diff))
            .map_err(|e| format!("Error generating HTML diff: {e}"))?;
        println!("HTML diff exported to {html_path}");
    }
    if opts.output != "-" {
        println!("Diff written to {output_path}");
    }
    Ok(has_changes)
}

/// Pick the terminal text renderer from the requested mode/format flags.
fn render_text(opts: &Cli, diff: &Diff, use_color: bool) -> String {
    if opts.word {
        if opts.unified.is_some() || opts.compact {
            render_unified_diff(
                &opts.old_file,
                &opts.new_file,
                diff,
                opts.unified.unwrap_or(0),
                use_color,
            )
        } else {
            render_word_diff(diff, use_color)
        }
    } else if let Some(context_lines) = opts.unified {
        render_unified_diff(
            &opts.old_file,
            &opts.new_file,
            diff,
            context_lines,
            use_color,
        )
    } else if opts.compact {
        render_unified_diff(&opts.old_file, &opts.new_file, diff, 0, use_color)
    } else {
        render_line_diff(diff, use_color)
    }
}

/// Pick the HTML renderer for the requested view.
fn render_html(opts: &Cli, diff: &Diff) -> String {
    if opts.side_by_side {
        render_side_by_side_html(diff, &opts.old_file, &opts.new_file, opts.html_theme)
    } else if opts.word {
        render_word_html(diff, opts.html_theme)
    } else {
        render_unified_html(
            diff,
            opts.unified.unwrap_or(3),
            &opts.old_file,
            &opts.new_file,
            opts.html_theme,
        )
    }
}

fn read_source(path: &str, use_mmap: bool) -> Result<Source, String> {
    read_file(path, use_mmap).map_err(|e| format!("Error reading {path}: {e}"))
}

fn source_str<'a>(source: &'a Source, path: &str) -> Result<&'a str, String> {
    source
        .as_str()
        .map_err(|e| format!("Error reading {path}: {e}"))
}

/// Base filename for derived outputs (e.g. the HTML file): strip a single
/// trailing `.diff` suffix.
fn html_base(output: &str) -> &str {
    output.strip_suffix(".diff").unwrap_or(output)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rustdiff::diff::modes::DiffAlgorithm;

    fn cli(old_file: String, new_file: String) -> Cli {
        Cli {
            old_file,
            new_file,
            output: "-".to_string(),
            color: ColorMode::Never,
            diff_algorithm: DiffAlgorithm::Histogram,
            html: false,
            html_theme: None,
            side_by_side: false,
            unified: None,
            compact: false,
            summary: false,
            word: false,
            line: false,
            exit_code: false,
            no_mmap: true,
        }
    }

    fn temp_file(name: &str, contents: &str) -> String {
        let path =
            std::env::temp_dir().join(format!("rustdiff_main_{}_{name}", std::process::id()));
        std::fs::write(&path, contents).unwrap();
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn test_html_base_plain() {
        assert_eq!(html_base("changes"), "changes");
    }

    #[test]
    fn test_html_base_single_suffix() {
        assert_eq!(html_base("my.diff"), "my");
    }

    #[test]
    fn test_html_base_double_suffix() {
        assert_eq!(html_base("report.diff.diff"), "report.diff");
    }

    #[test]
    fn test_html_base_other_extension() {
        assert_eq!(html_base("foo.patch"), "foo.patch");
    }

    #[test]
    fn run_reports_no_changes_for_identical_inputs() {
        let file = temp_file("identical", "hello\nworld\n");
        let opts = cli(file.clone(), file);
        assert!(
            !run(&opts).unwrap(),
            "identical inputs must report no changes"
        );
    }

    #[test]
    fn run_reports_changes_for_differing_inputs() {
        let old = temp_file("old", "alpha\nbeta\n");
        let new = temp_file("new", "alpha\nBETA\n");
        let opts = cli(old, new);
        assert!(run(&opts).unwrap(), "differing inputs must report changes");
    }

    #[test]
    fn run_errors_on_missing_file() {
        let opts = cli(
            "/nonexistent/rustdiff_old".to_string(),
            "/nonexistent/rustdiff_new".to_string(),
        );
        let err = run(&opts).unwrap_err();
        assert!(err.contains("Error reading"), "got: {err}");
    }
}
