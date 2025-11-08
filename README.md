# rustdiff

A fast, human‑readable diff generator written in pure Rust. Supports line, word, and unified diff formats with optional ANSI color output.

## Features

- Line diffs via Patience + Myers fallback: [`diff::modes::diff_lines`](src/diff/modes/line.rs)
- Word diffs with custom tokenizer: [`diff::modes::diff_words`](src/diff/modes/word.rs)
- Unified diff rendering (hunks + context): [`diff::render::render_unified_diff`](src/diff/render/unified.rs)
- Plain line diff rendering: [`diff::render::render_line_diff`](src/diff/render/line.rs)
- Inline word replacement markup: [`diff::render::render_word_diff`](src/diff/render/word.rs)
- Summary stats: [`diff::data::DiffStats`](src/diff/data.rs)
- Patience diff implementation: [`diff::core::patience::compute_patience_diff`](src/diff/core/patience.rs)
- Myers O(ND) core algorithm: [`diff::core::myers::compute_diff`](src/diff/core/myers.rs)
- Compact output mode (only changes)
- Color output (ANSI)

## Installation

Build from source:

```sh
git clone <repo>
cd rustdiff
cargo build --release
```

Binary will be in `target/release/rustdiff`.

## CLI Usage

Entry point: [`main`](src/main.rs) using Clap definition in [`cli::Cli`](src/cli.rs).

```sh
rustdiff <OLD> <NEW> [options]
```

Options:

| Flag | Purpose |
|------|---------|
| `--word` | Word-level diff |
| `--line` | Line-level diff (default) |
| `-u, --unified N` | Unified diff with N context lines (default 3) |
| `--compact` | Remove unchanged lines (keep +, -, headers) |
| `--color` | Enable ANSI color |
| `--summary` | Print only counts |
| `-o, --output FILE` | Write diff (default: changes.diff) |

Examples:

```sh
rustdiff old.txt new.txt --word --color --compact -o diff.txt
rustdiff old.txt new.txt -u 5 --color
rustdiff old.txt new.txt --summary
```

## Output Examples

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
@@ -10,4 +10,5 @@
  context
-context removed
+context added
```

Word diff inline (replacement grouping):

```
The quick [-brown+red] fox [+swiftly] jumps
```

With color enabled, insertions are green, deletions red.

## Library Use

Operations produce `Vec<DiffOp>` where [`DiffOp`](src/diff/data.rs) = `Equal(String) | Insert(String) | Delete(String)`.

Flow:

1. Tokenize: [`diff_lines`](src/diff/modes/line.rs) or [`diff_words`](src/diff/modes/word.rs)
2. Generate ops (Patience anchors + Myers fallback)
3. Render: line / word / unified
4. Optional stats: [`DiffStats::from_ops`](src/diff/data.rs)

## Algorithms

Patience diff:
- Unique anchors
- LIS on anchors: [`longest_increasing_subsequence`](src/diff/core/patience.rs)
- Myers on unmatched spans

Myers worst-case: O((N+M)D) with N old, M new, D edit distance.

Unified hunks grouped via [`group_into_hunks`](src/diff/render/unified.rs) -> [`Hunk`](src/diff/data.rs).

## Word Tokenization

Rules in [`tokenize`](src/diff/modes/word.rs):
- Collapse consecutive whitespace to single space token
- Normalize replacement markers `[-old+new]`
- Treat replacements atomically for coherence

## Architecture

- Core algorithms: [`diff/core`](src/diff/core/mod.rs)
- Data types: [`diff/data.rs`](src/diff/data.rs)
- Modes: [`diff/modes`](src/diff/modes/mod.rs)
- Renderers: [`diff/render`](src/diff/render/mod.rs)
- CLI: [`cli.rs`](src/cli.rs)
- FS I/O: [`fsio.rs`](src/fsio.rs)
- Entry: [`main.rs`](src/main.rs)

## Compact Mode

Implemented in `compact_diff_output` (main): keeps lines starting with `+`, `-`, `@@`, `---`, `+++`.

## Limitations / Notes

- Word mode treats punctuation as part of tokens (simple splitter).
- No binary file handling (assumes UTF‑8): [`read_file`](src/fsio.rs).
- Unified diff context size currently fixed (`CONTEXT = 4`); CLI `--unified` value not yet wired through.
- Colored word-level inline replacements may not render correctly in some pagers (ANSI sequences inside `[-old+new]`). Use a pager that preserves color:
  - Pipe directly: `rustdiff a b --word --color | less -R`
  - Or view saved file: `cat diff.txt | bat --paging=always`
  - Plain `less` without `-R` or some terminals may show raw escape codes.

## Roadmap

- Adjustable unified context lines
- Character-level diff mode
- Whitespace-ignore flag
- JSON output format
- Benchmarks / profiling

## Testing

```sh
cargo run -- old.txt new.txt --color
```

(Adjust file paths.)

## License

MIT (see [LICENSE](LICENSE)).
