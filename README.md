# rustdiff

A command-line text diff tool written in Rust. Compares two text files and
renders the differences as plain line diffs, unified diffs with configurable
context, word-level inline diffs, or interactive HTML.

```
rustdiff <OLD> <NEW> [OPTIONS]
```

## Features

- Line-level diffs (default)
- Word-level diffs with inline `[-old+new]` replacement markers
- Unified diff output with configurable context lines
- Compact output (changes only, no context)
- Summary output (insertion/deletion counts)
- ANSI colors with `auto`, `always`, and `never` modes
- Interactive HTML export: unified, side-by-side, and word-inline layouts
- POSIX-style exit codes for scripting/CI (`--exit-code`)
- `--ignore-whitespace` / `--ignore-case` / `--ignore-blank-lines` filters
- Read either input from stdin (`-`)
- Output to a file or stdout
- Optional parallel diffing (`--features parallel`) — off by default, see below

## Install

Requires Rust 1.97 or newer (the crate's MSRV; enforced in CI).

Build from source:

```sh
cargo build --release
./target/release/rustdiff --help
```

Install to your PATH:

```sh
cargo install --path .
```

## Usage

```
rustdiff <OLD> <NEW> [OPTIONS]
```

The two positional arguments are paths to the original and modified files.
Pass `-` for either to read that side from stdin (at most one side may be
`-`).

### Options

| Option | Description |
| ------ | ----------- |
| `-o, --output <FILE>` | Write output to `FILE` (default: `changes.diff`). Use `-o -` for stdout |
| `-u, --unified <N>` | Unified diff with `N` context lines |
| `--compact` | Show only changes (unified with 0 context lines) |
| `--summary` | Print insertion/deletion counts and exit |
| `--word` | Word-level diff with inline replacements |
| `--diff-algorithm <algo>` | `histogram` (default) or `myers` |
| `--color <mode>` | `auto`, `always`, or `never` (default: `auto`) |
| `--exit-code` | Exit `0` if no differences, `1` if differences found, `2` on error |
| `-w, --ignore-whitespace` | Ignore whitespace within tokens (line and word mode) |
| `-i, --ignore-case` | Ignore case when comparing tokens |
| `-B, --ignore-blank-lines` | Ignore changes that are only blank lines (line mode) |
| `--no-mmap` | Read files into memory instead of memory-mapping large files |
| `--verify` | Verify the computed diff is reversible before writing output |
| `--max-edit-distance <N>` | Degrade regions whose Myers edit distance would exceed `N` to a full delete+insert (off by default) |
| `--html` | Write an HTML diff (layout chosen below) |
| `--html-theme <theme>` | `dark` or `light`; default follows the viewer's OS preference |
| `--html-output <FILE>` | Write the HTML here instead of deriving it from `--output` |
| `--side-by-side` | Side-by-side HTML layout instead of unified (requires `--html`) |

### Exit codes

By default `rustdiff` exits `0` on success and `2` on error. With `--exit-code`
it behaves like POSIX `diff`:

- `0` — no differences
- `1` — differences found
- `2` — error (missing file, unreadable input, too large, etc.)

This makes the tool directly usable in CI scripts:

```sh
rustdiff committed/generated.rs generated.rs --exit-code -o - || exit 1
```

### Color behavior

- `auto`: colors only when writing to stdout and stdout is a terminal
- `always`: force colors on
- `never`: force colors off

In practice, the default output file (`changes.diff`) contains no ANSI codes
unless you pass `--color always`. The HTML output is always colorized
independently — `--color` only affects the text output.

### Examples

```sh
# Line diff written to changes.diff
rustdiff old.txt new.txt

# Write to stdout
rustdiff old.txt new.txt -o -

# Colorized output to a terminal
rustdiff old.txt new.txt --color always -o -

# Unified diff with 5 context lines
rustdiff old.txt new.txt -u 5

# Compact diff (changes only)
rustdiff old.txt new.txt --compact

# Summary only
rustdiff old.txt new.txt --summary

# Word-level inline diff
rustdiff old.txt new.txt --word -o -

# Force the Myers algorithm instead of histogram
rustdiff old.txt new.txt --diff-algorithm myers

# Ignore whitespace, case, and blank-line-only changes (exit 0)
rustdiff old.txt new.txt --ignore-whitespace --ignore-case --ignore-blank-lines --exit-code

# Diff against stdin
rustdiff old.txt - --summary < generated.txt

# Fail a build if generated code changed
rustdiff build/gen.rs expected.rs --exit-code -o - || exit 1

# Verify the diff is reversible before writing it
rustdiff old.txt new.txt --verify

# Cap Myers work on pathological inputs (degrade expensive regions to
# delete+insert instead of spinning)
rustdiff old.txt new.txt --max-edit-distance 1000000

# Write a unified HTML diff
rustdiff old.txt new.txt -o my.diff --html

# Side-by-side HTML with a baked light theme
rustdiff old.txt new.txt -o my.diff --html --side-by-side --html-theme light

# Word-level HTML with inline highlighting
rustdiff old.txt new.txt --word -o my.diff --html

# Write the HTML to an explicit path (no .diff file needed)
rustdiff old.txt new.txt --html --html-output report.html
```

With `--html`, `rustdiff` writes `<output>.html` next to the chosen output
unless `--html-output` overrides the path. `--html` uses a unified layout
(respecting `-u N`, default 3 context lines), `--side-by-side` switches to a
two-column layout, and `--word` produces inline word highlighting. Colors are
always applied in the HTML — no `--color` flag is needed.

### Interactive HTML

Generated HTML pages are self-contained (no network or build step) and include
a small amount of view-time JavaScript:

- **Jump to next/previous change** — the `Prev`/`Next` toolbar buttons, or the
  `n`/`j` (next) and `p`/`k` (previous) keys.
- **Collapsible unchanged regions** — long runs of unchanged lines in the
  numbered and side-by-side views collapse behind a "Show N unchanged lines"
  toggle; printing always reveals them.
- **Line-wrap toggle** — switch between wrapping and horizontal scroll, with
  the choice remembered in `localStorage`.
- **Theme toggle** — switch dark/light at view time, remembered per-browser;
  the default (no `--html-theme`) follows the viewer's `prefers-color-scheme`.

## Output formats

Line diff:

```
  unchanged line
- removed line
+ added line
```

Unified diff:

```
--- old.txt
+++ new.txt
@@ -10,2 +10,3 @@
  context
-removed line
+added line
+new line
```

Word diff — deleted and inserted words are grouped as replacements where they
are adjacent:

```
The quick [-brown+red] fox [+swiftly] jumps
```

With colors enabled, deletions are red and insertions green.

## Algorithms

`rustdiff` ships two diff engines, selectable with `--diff-algorithm`:

- **Histogram** (default). Picks the least-frequent token shared by both sides
  as an anchor, extends it into a maximal matching run, and recurses on the two
  halves. Produces readable diffs and degrades gracefully on repetitive input;
  for tokens that occur more than 64 times it falls back to Myers over that
  region.
- **Myers**. Produces a minimal edit script in linear space using a
  middle-snake divide-and-conquer search. Worst-case time is
  `O((N + M) * D)` where `N` and `M` are the input lengths and `D` is the edit
  distance.

Both operate over interned token IDs and emit run-length-encoded ops, so a
large mostly-unchanged file produces only a handful of diff records.

`--max-edit-distance <N>` caps how far the Myers search will go: any region
whose edit distance would exceed `N` degrades to a full delete + insert (still
a valid, reversible edit script, just not minimal). It is off by default and
mainly useful for bounding worst-case time on pathological inputs.

## Parallel diffing

Building with `--features parallel` runs the histogram's independent
sub-problems concurrently via `rayon`. It is off by default: on typical small
inputs the scheduling overhead outweighs the gain, and only large regions
(16k+ tokens) actually parallelize.

## Library use

The diff engine is also exposed as a library (`rustdiff::diff`). Both the
library and binary are covered by `#![deny(missing_docs)]`; run
`cargo doc --open` for the full API reference.

```rs
use rustdiff::diff::data::DiffStats;
use rustdiff::diff::modes::{DiffAlgorithm, DiffOptions, diff_lines, diff_lines_with};
use rustdiff::diff::render::render_unified_diff;

let diff = diff_lines("a\nb\n", "a\nX\n", DiffAlgorithm::Histogram).unwrap();
let text = render_unified_diff("old", "new", &diff, 3, true);
let stats = DiffStats::from_ops(&diff.ops);

// Normalization and a Myers edit-distance cap are also available:
let opts = DiffOptions {
    ignore_case: true,
    max_edit_distance: Some(1000),
    ..DiffOptions::default()
};
let diff = diff_lines_with("a\nB\n", "A\nb\n", DiffAlgorithm::Myers, opts)?;
```

Key types and functions:

- `diff::modes::{diff_lines, diff_words, diff_lines_with, diff_words_with}`,
  `DiffAlgorithm`, `DiffOptions`
- `diff::core::histogram::{compute_histogram_diff, compute_histogram_diff_limited}`,
  `diff::core::myers::{compute_diff, compute_diff_limited}` (the `_limited`
  variants accept an `Option<u32>` edit-distance cap)
- `diff::data::{Diff, Op, OpKind, Hunk, DiffStats}`, `Diff::validate_round_trip`
- `diff::intern::Interner`
- `diff::render::{render_line_diff, render_unified_diff, render_word_diff}`
- `diff::render::html::{render_unified_html, render_side_by_side_html,
  render_word_html, render_numbered_html}`, `HtmlTheme`
- `fsio::{Source, read_file}`

## Compatibility notes

- `--side-by-side` requires `--html` and conflicts with `--word`.
- With `--word` plus `--unified` or `--compact`, each word token is rendered
  on its own line rather than inline.
- `--ignore-blank-lines` applies to line mode only; in word mode it is ignored
  because line breaks are structural tokens there.

## Man page

A `rustdiff(1)` man page is generated from the CLI definition:

```sh
cargo run --example gen-man > rustdiff.1
```

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for commit conventions and the release
workflow.

The changelog and GitHub release notes are generated from conventional commits
with [`git-cliff`](https://git-cliff.org) (config in `cliff.toml`). Regenerate
`CHANGELOG.md` at release time with:

```sh
git cliff -o CHANGELOG.md
```

```sh
cargo fmt --all -- --check              # format
cargo clippy -- -D warnings -W clippy::pedantic -W clippy::nursery   # lint
cargo test --all-targets                # test
cargo test --all-features               # test with the parallel feature
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps   # docs (denies missing_docs)
```

Build warnings are denied via `.cargo/config.toml`. CI (`cargo fmt`, clippy,
tests, and docs on stable; a feature matrix covering `--no-default-features`
and `--all-features`; an MSRV check at 1.97; and `cargo-audit`) runs on every
push and builds a release binary on tags. See
[`.github/workflows/ci.yml`](.github/workflows/ci.yml).

To build with the optional parallel diffing feature:

```sh
cargo build --release --features parallel
```

### Memory regression

A `dhat`-based test asserts the Myers path stays O(N+M) in peak heap. Run it
alone for accurate numbers:

```sh
cargo test --test memory -- --test-threads=1
```

### Fuzzing

Two `cargo-fuzz` targets under `fuzz/` exercise the engine end to end.
`diff_round_trip` runs arbitrary byte inputs through both diff cores and
checks round-trip validity; `render_round_trip` pushes the same inputs through
every text and HTML renderer. Requires nightly (not part of stable CI):

```sh
cargo install cargo-fuzz
cargo +nightly fuzz run diff_round_trip
cargo +nightly fuzz run render_round_trip
```

## License

MIT — see [LICENSE](LICENSE).
