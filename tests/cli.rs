use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

fn bin() -> Command {
    Command::cargo_bin("rustdiff").unwrap()
}

fn write(path: &PathBuf, contents: &str) {
    fs::write(path, contents).unwrap();
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rustdiff_cli_{}_{name}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn old_new_pair(dir: &std::path::Path) -> (PathBuf, PathBuf) {
    let old = dir.join("old.txt");
    let new = dir.join("new.txt");
    write(&old, "alpha\nbeta\ngamma\n");
    write(&new, "alpha\nBETA\ngamma\n");
    (old, new)
}

#[test]
fn diff_to_stdout_shows_changes() {
    let dir = temp_dir("stdout");
    let (old, new) = old_new_pair(&dir);
    bin()
        .args([old.to_str().unwrap(), new.to_str().unwrap(), "-o", "-"])
        .assert()
        .success()
        .stdout(predicate::str::contains("- beta"))
        .stdout(predicate::str::contains("+ BETA"));
}

#[test]
fn default_output_file_is_changes_diff() {
    let dir = temp_dir("default_out");
    let (old, new) = old_new_pair(&dir);
    bin()
        .current_dir(&dir)
        .args([old.to_str().unwrap(), new.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Diff written to changes.diff"));
    assert!(dir.join("changes.diff").exists());
}

#[test]
fn exit_code_zero_when_identical() {
    let dir = temp_dir("exit0");
    let file = dir.join("same.txt");
    write(&file, "hello\nworld\n");
    bin()
        .args([
            "--exit-code",
            "-o",
            "-",
            file.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .code(0);
}

#[test]
fn exit_code_one_when_differ() {
    let dir = temp_dir("exit1");
    let (old, new) = old_new_pair(&dir);
    bin()
        .args([
            "--exit-code",
            "-o",
            "-",
            old.to_str().unwrap(),
            new.to_str().unwrap(),
        ])
        .assert()
        .code(1);
}

#[test]
fn exit_code_two_on_error() {
    bin()
        .args(["--exit-code", "-o", "-", "/nonexistent/a", "/nonexistent/b"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Error reading"));
}

#[test]
fn summary_prints_counts() {
    let dir = temp_dir("summary");
    let (old, new) = old_new_pair(&dir);
    bin()
        .args([old.to_str().unwrap(), new.to_str().unwrap(), "--summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Changes: +1, -1 (total 2)"));
}

#[test]
fn exit_code_with_summary() {
    let dir = temp_dir("summary_exit");
    let (old, new) = old_new_pair(&dir);
    bin()
        .args([
            old.to_str().unwrap(),
            new.to_str().unwrap(),
            "--summary",
            "--exit-code",
        ])
        .assert()
        .code(1);
}

#[test]
fn html_output_derives_name_from_default() {
    let dir = temp_dir("html_default");
    let (old, new) = old_new_pair(&dir);
    bin()
        .current_dir(&dir)
        .args([old.to_str().unwrap(), new.to_str().unwrap(), "--html"])
        .assert()
        .success();
    let html = fs::read_to_string(dir.join("changes.html")).unwrap();
    assert!(html.starts_with("<!DOCTYPE html>"));
}

#[test]
fn html_output_from_custom_output_path() {
    let dir = temp_dir("html_custom");
    let (old, new) = old_new_pair(&dir);
    bin()
        .current_dir(&dir)
        .args([
            old.to_str().unwrap(),
            new.to_str().unwrap(),
            "-o",
            "foo.patch",
            "--html",
        ])
        .assert()
        .success();
    assert!(dir.join("foo.patch.html").exists());
}

#[test]
fn html_base_strips_only_one_suffix() {
    let dir = temp_dir("html_double_suffix");
    let (old, new) = old_new_pair(&dir);
    bin()
        .current_dir(&dir)
        .args([
            old.to_str().unwrap(),
            new.to_str().unwrap(),
            "-o",
            "x.diff.diff",
            "--html",
        ])
        .assert()
        .success();
    assert!(dir.join("x.diff.html").exists());
}

#[test]
fn side_by_side_html_has_old_new_headers() {
    let dir = temp_dir("html_side_by_side");
    let (old, new) = old_new_pair(&dir);
    bin()
        .current_dir(&dir)
        .args([
            old.to_str().unwrap(),
            new.to_str().unwrap(),
            "--html",
            "--side-by-side",
        ])
        .assert()
        .success();
    let html = fs::read_to_string(dir.join("changes.html")).unwrap();
    assert!(html.contains("<th>Old</th>"));
    assert!(html.contains("<th>New</th>"));
}

#[test]
fn word_diff_to_stdout() {
    let dir = temp_dir("word");
    let (old, new) = old_new_pair(&dir);
    bin()
        .args([
            old.to_str().unwrap(),
            new.to_str().unwrap(),
            "--word",
            "-o",
            "-",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[-beta+BETA]"));
}

#[test]
fn diff_algorithms_agree_on_summary() {
    let dir = temp_dir("algorithms");
    let (old, new) = old_new_pair(&dir);
    let myers = bin()
        .args([
            old.to_str().unwrap(),
            new.to_str().unwrap(),
            "--summary",
            "--diff-algorithm",
            "myers",
        ])
        .output()
        .unwrap();
    let histogram = bin()
        .args([
            old.to_str().unwrap(),
            new.to_str().unwrap(),
            "--summary",
            "--diff-algorithm",
            "histogram",
        ])
        .output()
        .unwrap();
    assert!(myers.status.success());
    assert!(histogram.status.success());
    assert_eq!(myers.stdout, histogram.stdout);
}

#[test]
fn invalid_flag_exits_with_error() {
    bin().arg("--bogus").assert().code(2);
}

#[test]
fn no_mmap_flag_reads_small_files() {
    let dir = temp_dir("no_mmap");
    let (old, new) = old_new_pair(&dir);
    bin()
        .args([
            old.to_str().unwrap(),
            new.to_str().unwrap(),
            "--no-mmap",
            "--summary",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Changes:"));
}

#[test]
fn ignore_case_makes_case_only_diff_vanish() {
    let dir = temp_dir("ignore_case");
    let old = dir.join("old.txt");
    let new = dir.join("new.txt");
    write(&old, "Hello World\n");
    write(&new, "hello world\n");
    bin()
        .args([
            "--ignore-case",
            "--exit-code",
            "-o",
            "-",
            old.to_str().unwrap(),
            new.to_str().unwrap(),
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("  Hello World\n"));
}

#[test]
fn ignore_whitespace_makes_whitespace_only_diff_vanish() {
    let dir = temp_dir("ignore_ws");
    let old = dir.join("old.txt");
    let new = dir.join("new.txt");
    write(&old, "fn  foo ( x )\n");
    write(&new, "fn foo(x)\n");
    bin()
        .args([
            "--ignore-whitespace",
            "--exit-code",
            "-o",
            "-",
            old.to_str().unwrap(),
            new.to_str().unwrap(),
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("  fn  foo ( x )\n"));
}

#[test]
fn ignore_flags_still_detect_real_changes() {
    let dir = temp_dir("ignore_real_change");
    let old = dir.join("old.txt");
    let new = dir.join("new.txt");
    write(&old, "alpha beta\n");
    write(&new, "alpha GAMMA\n");
    bin()
        .args([
            "--ignore-whitespace",
            "--ignore-case",
            "--exit-code",
            "-o",
            "-",
            old.to_str().unwrap(),
            new.to_str().unwrap(),
        ])
        .assert()
        .code(1);
}

#[test]
fn stdin_supports_dash_for_new_side() {
    let dir = temp_dir("stdin_new");
    let old = dir.join("old.txt");
    write(&old, "alpha\nbeta\n");
    bin()
        .args([old.to_str().unwrap(), "-", "--summary"])
        .write_stdin("alpha\nBETA\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Changes: +1, -1"));
}

#[test]
fn stdin_supports_dash_for_old_side() {
    let dir = temp_dir("stdin_old");
    let new = dir.join("new.txt");
    write(&new, "alpha\nBETA\n");
    bin()
        .args(["-", new.to_str().unwrap(), "--summary"])
        .write_stdin("alpha\nbeta\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Changes: +1, -1"));
}

#[test]
fn both_stdin_inputs_error() {
    bin()
        .args(["-", "-", "--summary"])
        .write_stdin("x\n")
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "cannot read both inputs from stdin",
        ));
}

#[test]
fn verify_flag_passes_on_valid_diff() {
    let dir = temp_dir("verify");
    let (old, new) = old_new_pair(&dir);
    bin()
        .args([
            "--verify",
            old.to_str().unwrap(),
            new.to_str().unwrap(),
            "--summary",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Changes:"));
}

#[test]
fn verify_flag_passes_on_identical_inputs() {
    let dir = temp_dir("verify_identical");
    let file = dir.join("same.txt");
    write(&file, "hello\nworld\n");
    bin()
        .args(["--verify", file.to_str().unwrap(), file.to_str().unwrap()])
        .assert()
        .success();
}
