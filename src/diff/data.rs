#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffOp {
    Equal(String),
    Insert(String),
    Delete(String),
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub ops: Vec<DiffOp>,
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
    pub fn from_ops(ops: &[DiffOp]) -> Self {
        let mut stats = Self::default();
        for op in ops {
            match op {
                DiffOp::Insert(_) => stats.inserts += 1,
                DiffOp::Delete(_) => stats.deletes += 1,
                DiffOp::Equal(_) => {}
            }
        }

        stats.changes = stats.inserts + stats.deletes;
        stats
    }
}
