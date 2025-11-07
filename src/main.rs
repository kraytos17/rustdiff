mod cli;
mod diff;
mod fsio;

use crate::diff::data::DiffOp;
use clap::Parser;
use cli::Cli;
use diff::{core::compute_diff, render::render_diff};
use fsio::read_lines;
use std::{fmt::Write as FmtWrite, fs::File, io::Write as IoWrite, process, string::String};

fn main() {
    let opts = Cli::parse();
    let old_lines = read_lines(&opts.old_file).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", opts.old_file, e);
        process::exit(1);
    });

    let new_lines = read_lines(&opts.new_file).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", opts.new_file, e);
        process::exit(1);
    });

    let old_refs: Vec<&str> = old_lines.iter().map(String::as_str).collect();
    let new_refs: Vec<&str> = new_lines.iter().map(String::as_str).collect();
    let diff = compute_diff(&old_refs, &new_refs);

    if opts.summary {
        let (inserts, deletes) = diff.iter().fold((0, 0), |(i, d), op| match op {
            DiffOp::Insert(_) => (i + 1, d),
            DiffOp::Delete(_) => (i, d + 1),
            DiffOp::Equal(_) => (i, d),
        });

        println!("Changes: +{inserts}, -{deletes}");
        return;
    }

    let rendered = render_diff(&diff, opts.color, opts.unified);
    let mut output = String::new();

    writeln!(output, "--- {}", opts.old_file).unwrap();
    writeln!(output, "+++ {}", opts.new_file).unwrap();
    writeln!(output, "@@").unwrap();
    write!(output, "{rendered}").unwrap();

    let output_path = "changes.diff";
    let mut file = File::create(output_path).unwrap_or_else(|e| {
        eprintln!("Error creating {output_path}: {e}");
        process::exit(1);
    });

    file.write_all(output.as_bytes()).unwrap_or_else(|e| {
        eprintln!("Error writing diff: {e}");
        process::exit(1);
    });

    println!("Diff written to {output_path}");
}
