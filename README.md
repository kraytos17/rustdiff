# rustdiff

Fast, human‑readable text diffs in pure Rust for Linux. Supports line and word modes, unified formatting with configurable context, compact and summary output, ANSI colors with controllable mode, and HTML export (numbered and side‑by‑side).

- Entry point: [`main`](src/main.rs)
- CLI definition: [`cli::Cli`](src/cli.rs)

## Features

- Line diffs (Patience + Myers): [`diff::modes::diff_lines`](src/diff/modes/line.rs)
- Word diffs with inline replacements: [`diff::modes::diff_words`](src/diff/modes/word.rs)
- Unified diff with context lines and headers: [`diff::render::render_unified_diff`](src/diff/render/unified.rs)
  - Hunk grouping: `group_into_hunks` (internal) in [src/diff/render/unified.rs](src/diff/render/unified.rs)
- Plain line rendering: [`diff::render::render_line_diff`](src/diff/render/line.rs)
- Inline word replacement markup and colors: [`diff::render::render_word_diff`](src/diff/render/word.rs)
- HTML export from ANSI output (numbered): [`diff::render::render_diff_outputs`](src/diff/render/mod.rs)
- Side‑by‑side HTML export: [`diff::render::render_side_by_side_html`](src/diff/render/side_by_side.rs)
- Stats: [`diff::data::DiffStats`](src/diff/data.rs)
- Core algorithms:
  - Patience (unique anchors + LIS): [`diff::core::patience::compute_patience_diff`](src/diff/core/patience.rs)
  - Myers minimal edit script: [`diff::core::myers::compute_diff`](src/diff/core/myers.rs)

Data types:
- Ops: [`diff::data::DiffOp`](src/diff/data.rs) = Equal(String) | Insert(String) | Delete(String)
- Unified hunks: [`diff::data::Hunk`](src/diff/data.rs)

## Install (Linux)

Build from source:

```sh
cargo build --release
```

Run from target:

```sh
./target/release/rustdiff --help
```

Install to PATH (user):

```sh
cargo install --path .
```

## CLI

Synopsis:

```sh
rustdiff <OLD> <NEW> [flags]
```

Primary flags (see [`cli::Cli`](src/cli.rs)):
- --line: line-level diff (default)
- --word: word-level diff
- -u, --unified N: unified diff with N context lines
- --compact: only changes (unified with 0 context)
- --summary: only print counts
- --color <auto|always|never>: color mode (default: auto) — see Color behavior
- -o, --output FILE: write to file (default: changes.diff). Use `-o -` to write to stdout
- --html: also generate HTML next to the diff
- --side-by-side: generate side‑by‑side HTML (requires `--html`, conflicts with `--word`, `--unified`, `--compact`, `--summary`)

Color behavior (implemented in [`main`](src/main.rs) via [`cli::ColorMode`](src/cli.rs)):
- auto: enabled only when writing to stdout, stdout is a TTY, and `--html` is not set
- always: force colors
- never: disable colors

Practically: with the default `-o changes.diff`, colors are not included unless `--color always` is used or you write to stdout with `-o -`.

### Examples

```sh
# Line diff (plain)
rustdiff old.txt new.txt

# Force colors and write to terminal
rustdiff old.txt new.txt --color always -o -

# Unified with 5 lines of context and color auto (TTY only)
rustdiff old.txt new.txt -u 5 --color auto -o -

# Word-level inline replacements (not supported with unified/compact)
rustdiff old.txt new.txt --word --color always -o -

# Compact (only +/- and headers)
rustdiff old.txt new.txt --compact

# Summary only
rustdiff old.txt new.txt --summary

# Write to custom file and export numbered HTML
rustdiff old.txt new.txt --color never -o my.diff --html

# Side-by-side HTML (requires --html; line mode only)
rustdiff old.txt new.txt -o my.diff --html --side-by-side
```

If `--html` is set:
- Numbered HTML and a `.diff` are written via [`diff::render::render_diff_outputs`](src/diff/render/mod.rs) to `<base>.html` and `<base>.diff`.
- With `--side-by-side`, a full-width side-by-side HTML is written via [`diff::render::render_side_by_side_html`](src/diff/render/side_by_side.rs) to `<base>_side_by_side.html`.

## Output examples

Line diff ([`diff::render::render_line_diff`](src/diff/render/line.rs)):

```
  unchanged line
- removed line
+ added line
```

Unified ([`diff::render::render_unified_diff`](src/diff/render/unified.rs)):

```
--- old.txt
+++ new.txt
@@ -10,2 +10,3 @@
-context removed
+context added
+new line
```

Word inline ([`diff::render::render_word_diff`](src/diff/render/word.rs)):

```
The quick [-brown+red] fox [+swiftly] jumps
```

With colors enabled, deletions are red and insertions green.

## Programmatic use (internal modules)

```rs
use rustdiff::diff::modes::{diff_lines, diff_words};
use rustdiff::diff::render::{render_line_diff, render_unified_diff, render_word_diff};
use rustdiff::diff::data::DiffStats;

let ops = diff_lines("a\nb\n", "a\nX\n");
let text = render_unified_diff("old", "new", &ops, 3, true);
let stats = DiffStats::from_ops(&ops);
```

Key APIs:
- [`diff::modes::diff_lines`](src/diff/modes/line.rs)
- [`diff::modes::diff_words`](src/diff/modes/word.rs)
- [`diff::render::render_unified_diff`](src/diff/render/unified.rs)
- [`diff::render::render_line_diff`](src/diff/render/line.rs)
- [`diff::render::render_word_diff`](src/diff/render/word.rs)
- [`diff::render::render_diff_outputs`](src/diff/render/mod.rs)
- [`diff::render::render_side_by_side_html`](src/diff/render/side_by_side.rs)
- [`diff::data::DiffStats`](src/diff/data.rs)

## Algorithms

- Patience diff:
  - Unique anchors + LIS to stabilize matching
  - Unmatched spans fall back to Myers
  - Implementation: [`diff::core::patience::compute_patience_diff`](src/diff/core/patience.rs)

- Myers diff:
  - Worst-case complexity $O((N+M)D)$ with $N$ old length, $M$ new length, $D$ edit distance
  - Implementation: [`diff::core::myers::compute_diff`](src/diff/core/myers.rs)

Unified grouping builds hunks with configurable context in `group_into_hunks` (see [src/diff/render/unified.rs](src/diff/render/unified.rs)) producing [`diff::data::Hunk`](src/diff/data.rs).

## I/O and platform

- Reads whole files as UTF‑8 strings: [`fsio::read_file`](src/fsio.rs)
- Writes chosen output file in [`main`](src/main.rs)
- Linux-focused TTY detection via `libc::isatty` (colors auto-detect on Unix terminals)

## Compatibility notes

- `--word` is not supported with `--unified` or `--compact` output modes.
- `--side-by-side` requires `--html` and conflicts with `--word`, `--unified`, `--compact`, `--summary`.

## Development

- Format: `cargo fmt --all`
- Lint: `cargo clippy -- -D warnings`
- Test: `cargo test --all --verbose`
  - Unit tests in [`diff::core::myers`](src/diff/core/myers.rs) and [`diff::core::patience`](src/diff/core/patience.rs)
- CI: [GitHub Actions workflow](.github/workflows/ci.yml)

## License

MIT — see [LICENSE](LICENSE)