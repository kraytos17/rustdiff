#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpKind {
    Equal,
    Insert,
    Delete,
}

/// One run of same-kind edits: `[start, start + len)` into the relevant side's
/// token array (`old_tokens` for Equal/Delete, `new_tokens` for Insert).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Op {
    pub kind: OpKind,
    pub start: u32,
    pub len: u32,
}

impl Op {
    #[must_use]
    pub const fn equal(start: u32, len: u32) -> Self {
        Self {
            kind: OpKind::Equal,
            start,
            len,
        }
    }

    #[must_use]
    pub const fn insert(start: u32, len: u32) -> Self {
        Self {
            kind: OpKind::Insert,
            start,
            len,
        }
    }

    #[must_use]
    pub const fn delete(start: u32, len: u32) -> Self {
        Self {
            kind: OpKind::Delete,
            start,
            len,
        }
    }
}

/// A computed diff: run-length-encoded ops plus the token arrays they index into.
///
/// Renders resolve each run back to text lazily against `old_tokens`/`new_tokens`.
#[derive(Debug, Clone)]
pub struct Diff {
    pub ops: Vec<Op>,
    pub old_tokens: Vec<String>,
    pub new_tokens: Vec<String>,
}

impl Diff {
    /// The token array an op of `kind` indexes into: `old_tokens` for
    /// Equal/Delete, `new_tokens` for Insert.
    #[must_use]
    pub fn tokens_for(&self, kind: OpKind) -> &[String] {
        match kind {
            OpKind::Equal | OpKind::Delete => &self.old_tokens,
            OpKind::Insert => &self.new_tokens,
        }
    }

    /// Unroll every run into individual `(OpKind, &str)` token edits.
    ///
    /// The word render needs per-token granularity for its replacement-grouping
    /// lookahead, so it consumes this stream rather than the runs directly.
    #[must_use]
    pub fn edits(&self) -> Vec<(OpKind, &str)> {
        let capacity = self.ops.iter().map(|op| op.len as usize).sum();
        let mut edits = Vec::with_capacity(capacity);
        for op in &self.ops {
            let start = op.start as usize;
            for text in &self.tokens_for(op.kind)[start..start + op.len as usize] {
                edits.push((op.kind, text.as_str()));
            }
        }
        edits
    }

    /// Verify the ops transform `a` into `b`: Equal/Delete ops consume matching
    /// ranges of `a` in order, Insert ops consume matching ranges of `b`, and
    /// both sequences are fully consumed.
    #[must_use]
    pub fn validate_round_trip(&self, a: &[&str], b: &[&str]) -> bool {
        let mut ai = 0usize;
        let mut bi = 0usize;
        for op in &self.ops {
            let start = op.start as usize;
            let len = op.len as usize;
            match op.kind {
                OpKind::Equal => {
                    if ai + len > a.len() || bi + len > b.len() || start + len > a.len() {
                        return false;
                    }
                    if a[ai..ai + len] != a[start..start + len]
                        || a[start..start + len] != b[bi..bi + len]
                    {
                        return false;
                    }

                    ai += len;
                    bi += len;
                }
                OpKind::Delete => {
                    if ai + len > a.len() || start + len > a.len() {
                        return false;
                    }
                    if a[ai..ai + len] != a[start..start + len] {
                        return false;
                    }
                    ai += len;
                }
                OpKind::Insert => {
                    if bi + len > b.len() || start + len > b.len() {
                        return false;
                    }
                    if b[bi..bi + len] != b[start..start + len] {
                        return false;
                    }
                    bi += len;
                }
            }
        }
        ai == a.len() && bi == b.len()
    }
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub ops: Vec<Op>,
    pub start_a: usize,
    pub start_b: usize,
    pub len_a: usize,
    pub len_b: usize,
}

#[derive(Debug, Default, Clone)]
pub struct DiffStats {
    pub inserts: usize,
    pub deletes: usize,
    pub changes: usize,
}

impl DiffStats {
    /// Sum run lengths so counts reflect per-line insertions/deletions.
    #[must_use]
    pub fn from_ops(ops: &[Op]) -> Self {
        let inserts = ops
            .iter()
            .filter(|op| op.kind == OpKind::Insert)
            .map(|op| op.len as usize)
            .sum();
        let deletes = ops
            .iter()
            .filter(|op| op.kind == OpKind::Delete)
            .map(|op| op.len as usize)
            .sum();

        Self {
            inserts,
            deletes,
            changes: inserts + deletes,
        }
    }
}

/// Merge adjacent same-kind runs whose ranges are contiguous.
pub(crate) fn coalesce(ops: &mut Vec<Op>) {
    let mut out: Vec<Op> = Vec::with_capacity(ops.len());
    for op in ops.drain(..) {
        if let Some(last) = out.last_mut()
            && last.kind == op.kind
            && last.start + last.len == op.start
        {
            last.len += op.len;
            continue;
        }
        out.push(op);
    }
    *ops = out;
}

/// Maximum number of tokens the `u32`-indexed core can address. Inputs above
/// this are rejected up front with a clean error instead of panicking in
/// [`u32_len`].
pub(crate) const MAX_TOKENS: usize = u32::MAX as usize;

/// Reject token counts the `u32`-indexed core cannot address.
pub(crate) fn ensure_within_u32(count: usize, what: &str) -> Result<(), String> {
    if count > MAX_TOKENS {
        Err(format!(
            "file too large to diff: exceeds {MAX_TOKENS} {what}"
        ))
    } else {
        Ok(())
    }
}

/// Convert a `usize` length/position to `u32`, the op field width.
///
/// Callers guard with [`ensure_within_u32`] before invoking the core, so this
/// panic is a defensive backstop, not a reachable failure path for the CLI.
pub(crate) fn u32_len(len: usize) -> u32 {
    u32::try_from(len).expect("diff length exceeds u32")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_constructors() {
        assert_eq!(Op::equal(0, 3).kind, OpKind::Equal);
        assert_eq!(Op::insert(1, 2).start, 1);
        assert_eq!(Op::delete(2, 1).len, 1);
    }

    #[test]
    fn test_ensure_within_u32_accepts_small() {
        assert_eq!(ensure_within_u32(0, "lines"), Ok(()));
        assert_eq!(ensure_within_u32(1000, "tokens"), Ok(()));
        assert_eq!(ensure_within_u32(MAX_TOKENS, "lines"), Ok(()));
    }

    #[test]
    fn test_ensure_within_u32_rejects_overflow() {
        let err = ensure_within_u32(MAX_TOKENS + 1, "lines").unwrap_err();
        assert!(err.contains("too large to diff"), "got: {err}");
    }

    #[test]
    fn test_coalesce_merges_adjacent() {
        let mut ops = vec![
            Op::equal(0, 2),
            Op::equal(2, 1),
            Op::delete(3, 1),
            Op::delete(4, 1),
            Op::insert(5, 1),
        ];
        coalesce(&mut ops);
        assert_eq!(
            ops,
            vec![Op::equal(0, 3), Op::delete(3, 2), Op::insert(5, 1)]
        );
    }

    #[test]
    fn test_edits_unrolls_runs() {
        let diff = Diff {
            ops: vec![Op::equal(0, 1), Op::delete(1, 1), Op::insert(0, 1)],
            old_tokens: vec!["a".to_string(), "b".to_string()],
            new_tokens: vec!["c".to_string()],
        };

        let edits = diff.edits();
        assert_eq!(
            edits,
            vec![
                (OpKind::Equal, "a"),
                (OpKind::Delete, "b"),
                (OpKind::Insert, "c"),
            ]
        );
    }

    #[test]
    fn test_stats_sums_runs() {
        let ops = vec![Op::equal(0, 5), Op::delete(5, 2), Op::insert(5, 3)];

        let stats = DiffStats::from_ops(&ops);
        assert_eq!(stats.inserts, 3);
        assert_eq!(stats.deletes, 2);
        assert_eq!(stats.changes, 5);
    }

    #[test]
    fn test_validate_round_trip_ok() {
        let a = ["a", "b", "c", "d"];
        let b = ["a", "x", "c", "d"];
        let diff = Diff {
            ops: vec![
                Op::equal(0, 1),
                Op::delete(1, 1),
                Op::insert(1, 1),
                Op::equal(2, 2),
            ],
            old_tokens: a.iter().copied().map(str::to_owned).collect(),
            new_tokens: b.iter().copied().map(str::to_owned).collect(),
        };
        assert!(diff.validate_round_trip(&a, &b));
    }

    #[test]
    fn test_validate_round_trip_bad() {
        // Equal op claims a match but the ranges differ on the b side.
        let a = ["a", "b"];
        let b = ["x", "b"];
        let diff = Diff {
            ops: vec![Op::equal(0, 2)],
            old_tokens: a.iter().copied().map(str::to_owned).collect(),
            new_tokens: b.iter().copied().map(str::to_owned).collect(),
        };
        assert!(!diff.validate_round_trip(&a, &b));
    }
}
