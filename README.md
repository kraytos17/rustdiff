# rustdiff

Fast, human‑readable text diffs in pure Rust for Linux. Supports line and word modes, unified output with adjustable context, ANSI colors, and optional HTML export.

- Entry point: [`main`](src/main.rs)
- CLI definition: [`cli::Cli`](src/cli.rs)

## Features

- Line diffs (Patience + Myers): [`diff::modes::diff_lines`](src/diff/modes/line.rs)
- Word diffs with inline replacements: [`diff::modes::diff_words`](src/diff/modes/word.rs)
- Unified diff with context lines and headers: [`diff::render::render_unified_diff`](src/diff/render/unified.rs)
- Plain line rendering: [`diff::render::render_line_diff`](src/diff/render/line.rs)
- Inline word replacement markup and colors: [`diff::render::render_word_diff`](src/diff/render/word.rs)
- HTML export from ANSI output: [`diff::render::write_diff_outputs`](src/diff/render/mod.rs)
- Stats: [`diff::data::DiffStats`](src/diff/data.rs)
- Core algorithms:
  - Patience (unique anchors + LIS): [`diff::core::patience::compute_patience_diff`](src/diff/core/patience.rs)
  - Myers O(ND) minimal edit script: [`diff::core::myers::compute_diff`](src/diff/core/myers.rs)

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

If you downloaded a standalone binary, make it executable on Linux:

```sh
chmod +x ./rustdiff && ./rustdiff --help
```

## CLI

Synopsis:

```sh
rustdiff <OLD> <NEW> [flags]
```

Common flags (see [`cli::Cli`](src/cli.rs)):
- --line: line-level diff (default)
- --word: word-level diff
- -u, --unified N: unified diff with N context lines
- --compact: only changes (implemented as unified with 0 context)
- --color: enable ANSI colors
- --summary: only print counts
- -o, --output FILE: write to file (default: changes.diff)
- --html: also generate HTML next to .diff

Compatibility checks in [`main`](src/main.rs):
- --word cannot be combined with --unified
- --word cannot be combined with --compact

### Examples

```sh
# Line diff (plain)
rustdiff old.txt new.txt

# Unified with 5 lines of context and color
rustdiff old.txt new.txt -u 5 --color

# Word-level inline replacements with color
rustdiff old.txt new.txt --word --color

# Compact (only +/- and headers)
rustdiff old.txt new.txt --compact

# Summary only
rustdiff old.txt new.txt --summary

# Write to custom file and export HTML
rustdiff old.txt new.txt --color -o my.diff --html
```

If `--html` is set, an HTML file is written next to the diff via [`diff::render::write_diff_outputs`](src/diff/render/mod.rs). The converter numbers lines and transforms ANSI sequences using ansi-to-html.

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

With `--color`, deletions are red and insertions green.

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
- [`diff::data::DiffStats`](src/diff/data.rs)

## Algorithms

- Patience diff:
  - Unique anchors + LIS to stabilize matching
  - Unmatched spans fall back to Myers
  - Implementation: [`diff::core::patience::compute_patience_diff`](src/diff/core/patience.rs)

- Myers diff:
  - Worst-case complexity O((N+M)D) with N old length, M new length, D edit distance
  - Implementation: [`diff::core::myers::compute_diff`](src/diff/core/myers.rs)

Unified grouping builds hunks with configurable context in [`diff::render::group_into_hunks`](src/diff/render/unified.rs) producing [`diff::data::Hunk`](src/diff/data.rs).

## I/O

- Reads whole files as UTF‑8 strings: [`fsio::read_file`](src/fsio.rs)
- Writes chosen output file in [`main`](src/main.rs)
- Optional HTML export alongside `.diff`: [`diff::render::write_diff_outputs`](src/diff/render/mod.rs)

## Compact and summary modes

- Compact: implemented by calling unified renderer with 0 context, keeping only `+`, `-`, and hunk/file headers.
- Summary: [`diff::data::DiffStats`](src/diff/data.rs) counts operations; `changes = inserts + deletes`.

## Development

- Format: `cargo fmt --all`
- Lint: `cargo clippy -- -D warnings`
- Test: `cargo test --all --verbose` (unit tests in [`diff::core::myers`](src/diff/core/myers.rs) and [`diff::core::patience`](src/diff/core/patience.rs))

## Limitations

- Text only; assumes UTF‑8 input ([`fsio::read_file`](src/fsio.rs))
- Word tokenizer is whitespace-based; punctuation is not split
- Some pagers may not render ANSI inside `[-old+new]`. Use `less -R` or view HTML export

## License

MIT — see [LICENSE](LICENSE)