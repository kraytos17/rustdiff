use crate::diff::data::{DiffOp, Hunk};
use std::fmt::Write;

/// Render a unified diff similar to `git diff --unified`.
///
/// Includes hunk headers (`@@ -a,b +c,d @@`) and configurable context.
pub fn render_unified_diff(old_name: &str, new_name: &str, diffs: &[DiffOp]) -> String {
    const CONTEXT: usize = 4; // Number of unchanged context lines
    let mut out = String::new();

    writeln!(out, "--- {old_name}").unwrap();
    writeln!(out, "+++ {new_name}").unwrap();

    let hunks = group_into_hunks(diffs, CONTEXT);
    for hunk in hunks {
        writeln!(
            out,
            "@@ -{},{} +{},{} @@",
            hunk.start_a, hunk.len_a, hunk.start_b, hunk.len_b
        )
        .unwrap();

        for op in &hunk.ops {
            match op {
                DiffOp::Equal(line) => writeln!(out, " {line}").unwrap(),
                DiffOp::Insert(line) => writeln!(out, "+{line}").unwrap(),
                DiffOp::Delete(line) => writeln!(out, "-{line}").unwrap(),
            }
        }
    }

    out
}

/// Group the raw [`DiffOp`]s into hunks with context lines.
///
/// This approximates Git’s unified diff behavior.
fn group_into_hunks(diffs: &[DiffOp], context: usize) -> Vec<Hunk> {
    let mut hunks = Vec::new();
    let mut current_hunk = Vec::new();

    let mut old_line = 1;
    let mut new_line = 1;
    let mut start_a = 1;
    let mut start_b = 1;

    let mut in_hunk = false;
    let mut context_buffer: Vec<DiffOp> = Vec::new();
    for op in diffs {
        match op {
            DiffOp::Equal(line) => {
                if in_hunk {
                    // Add equal lines inside or right after a change (context)
                    current_hunk.push(DiffOp::Equal(line.clone()));
                    if context_buffer.len() >= context {
                        // Enough trailing context -> close hunk
                        let len_a = current_hunk
                            .iter()
                            .filter(|o| matches!(o, DiffOp::Equal(_) | DiffOp::Delete(_)))
                            .count();

                        let len_b = current_hunk
                            .iter()
                            .filter(|o| matches!(o, DiffOp::Equal(_) | DiffOp::Insert(_)))
                            .count();

                        hunks.push(Hunk {
                            start_a,
                            start_b,
                            len_a,
                            len_b,
                            ops: current_hunk.clone(),
                        });

                        current_hunk.clear();
                        in_hunk = false;
                        context_buffer.clear();
                    } else {
                        context_buffer.push(DiffOp::Equal(line.clone()));
                    }
                }

                old_line += 1;
                new_line += 1;
            }
            DiffOp::Insert(line) => {
                if !in_hunk {
                    in_hunk = true;
                    start_a = old_line;
                    start_b = new_line;
                    let start_idx = if context_buffer.len() > context {
                        context_buffer.len() - context
                    } else {
                        0
                    };

                    for prev in &context_buffer[start_idx..] {
                        current_hunk.push(prev.clone());
                    }

                    context_buffer.clear();
                }
                current_hunk.push(DiffOp::Insert(line.clone()));
                new_line += 1;
            }
            DiffOp::Delete(line) => {
                if !in_hunk {
                    in_hunk = true;
                    start_a = old_line;
                    start_b = new_line;
                    let start_idx = if context_buffer.len() > context {
                        context_buffer.len() - context
                    } else {
                        0
                    };

                    for prev in &context_buffer[start_idx..] {
                        current_hunk.push(prev.clone());
                    }

                    context_buffer.clear();
                }
                current_hunk.push(DiffOp::Delete(line.clone()));
                old_line += 1;
            }
        }
    }

    if in_hunk && !current_hunk.is_empty() {
        let len_a = current_hunk
            .iter()
            .filter(|o| matches!(o, DiffOp::Equal(_) | DiffOp::Delete(_)))
            .count();

        let len_b = current_hunk
            .iter()
            .filter(|o| matches!(o, DiffOp::Equal(_) | DiffOp::Insert(_)))
            .count();

        hunks.push(Hunk {
            start_a,
            start_b,
            len_a,
            len_b,
            ops: current_hunk,
        });
    }

    hunks
}
