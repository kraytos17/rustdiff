# rustdiff

A command-line text diff tool written in Rust for Linux. Compares two text
files and renders the differences as plain line diffs, unified diffs with
configurable context, word-level inline diffs, or HTML.

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
- HTML export: unified, side-by-side, and word-inline layouts, dark or light theme
- Output to a file or stdout
- Optional parallel diffing (`--features parallel`) — off by default, see below

## Install

Requires Rust 1.97 or newer.

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

### Options

| Option | Description |
| ------ | ----------- |
| `-o, --output <FILE>` | Write output to `FILE` (default: `changes.diff`). Use `-o -` for stdout |
| `-u, --unified <N>` | Unified diff with `N` context lines |
| `--compact` | Show only changes (unified with 0 context lines) |
| `--summary` | Print insertion/deletion counts and exit |
| `--word` | Word-level diff with inline replacements |
| `--line` | Line-level diff (default) |
| `--diff-algorithm <algo>` | `histogram` (default) or `myers` |
| `--color <mode>` | `auto`, `always`, or `never` (default: `auto`) |
| `--html` | Write a unified HTML diff to `<output>.html` |
| `--side-by-side` | Side-by-side HTML layout instead of unified (requires `--html`) |
| `--html-theme <theme>` | HTML color theme: `dark` (default) or `light` |

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

# Write a unified HTML diff
rustdiff old.txt new.txt -o my.diff --html

# Side-by-side HTML with a light theme
rustdiff old.txt new.txt -o my.diff --html --side-by-side --html-theme light

# Word-level HTML with inline highlighting
rustdiff old.txt new.txt --word -o my.diff --html
```

With `--html`, `rustdiff` writes `<output>.html` next to the chosen output.
`--html` uses a unified layout (respecting `-u N`, default 3 context lines),
`--side-by-side` switches to a two-column layout, and `--word` produces
inline word highlighting. Colors are always applied in the HTML — no
`--color` flag is needed.

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

## Parallel diffing

Building with `--features parallel` runs the histogram's independent
sub-problems concurrently via `rayon`. It is off by default: on typical small
inputs the scheduling overhead outweighs the gain, and only large regions
(16k+ tokens) actually parallelize.

## Library use

The diff core is also exposed as a library (`rustdiff::diff`):

```rs
use rustdiff::diff::modes::{diff_lines, DiffAlgorithm};
use rustdiff::diff::render::render_unified_diff;
use rustdiff::diff::data::DiffStats;

let diff = diff_lines("a\nb\n", "a\nX\n", DiffAlgorithm::Histogram);
let text = render_unified_diff("old", "new", &diff, 3, true);
let stats = DiffStats::from_ops(&diff.ops);
```

Key types and functions:

- `diff::modes::diff_lines`, `diff::modes::diff_words`
- `diff::core::histogram::compute_histogram_diff`
- `diff::core::myers::compute_diff`
- `diff::render::render_line_diff`, `render_unified_diff`, `render_word_diff`
- `diff::render::html::render_unified_html`, `render_side_by_side_html`,
  `render_word_html`, `render_numbered_html`
- `diff::data::Diff`, `Op`, `Hunk`, `DiffStats`

## Compatibility notes

- `--side-by-side` requires `--html` and conflicts with `--word`.
- With `--word` plus `--unified` or `--compact`, each word token is rendered
  on its own line rather than inline.

## Development

```sh
cargo fmt --all -- --check              # format
cargo clippy -- -D warnings -W clippy::pedantic -W clippy::nursery   # lint
cargo test --all-targets                # test
```

Build warnings are denied via `.cargo/config.toml`. CI runs the same format,
lint, and test checks on every push and builds a release binary on tags. See
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

A `cargo-fuzz` target (`fuzz/`) round-trips arbitrary byte inputs through both
diff engines. Requires nightly (not part of stable CI):

```sh
cargo install cargo-fuzz
cargo +nightly fuzz run diff_round_trip
```

## License

MIT — see [LICENSE](LICENSE).
