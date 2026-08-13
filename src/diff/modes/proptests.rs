//! Mode-level proptests exercising the tokenizer -> interner -> core ->
//! `coalesce` pipeline as a whole. Core-only proptests live inside the cores;
//! these cover the full `diff_lines` / `diff_words` entry points and the
//! cross-algorithm invariants.

use crate::diff::data::{Diff, Op, OpKind};
use crate::diff::modes::{DiffAlgorithm, diff_lines, diff_words};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

fn refs(tokens: &[String]) -> Vec<&str> {
    tokens.iter().map(String::as_str).collect()
}

fn sum_changes(ops: &[Op]) -> usize {
    ops.iter()
        .filter(|op| op.kind != OpKind::Equal)
        .map(|op| op.len as usize)
        .sum()
}

fn assert_valid(diff: &Diff) -> Result<(), TestCaseError> {
    let old_refs = refs(&diff.old_tokens);
    let new_refs = refs(&diff.new_tokens);
    prop_assert!(
        diff.validate_round_trip(&old_refs, &new_refs),
        "round-trip failed"
    );
    Ok(())
}

proptest! {
    #[test]
    fn prop_cross_algorithm_line(
        old in "[\n \tA-Za-z]{0,200}",
        new in "[\n \tA-Za-z]{0,200}",
    ) {
        let hist = diff_lines(&old, &new, DiffAlgorithm::Histogram).unwrap();
        let myers = diff_lines(&old, &new, DiffAlgorithm::Myers).unwrap();
        assert_valid(&hist)?;
        assert_valid(&myers)?;

        // Histogram is a heuristic: it must never produce FEWER edits than the
        // minimal (Myers) script for the same input.
        prop_assert!(
            sum_changes(&hist.ops) >= sum_changes(&myers.ops),
            "histogram produced fewer edits than minimal Myers"
        );

        // Determinism: identical input yields identical ops.
        let hist2 = diff_lines(&old, &new, DiffAlgorithm::Histogram).unwrap();
        prop_assert_eq!(hist.ops, hist2.ops);
        let myers2 = diff_lines(&old, &new, DiffAlgorithm::Myers).unwrap();
        prop_assert_eq!(myers.ops, myers2.ops);
    }

    #[test]
    fn prop_cross_algorithm_word(
        old in "[\n \tA-Za-z]{0,200}",
        new in "[\n \tA-Za-z]{0,200}",
    ) {
        let hist = diff_words(&old, &new, DiffAlgorithm::Histogram).unwrap();
        let myers = diff_words(&old, &new, DiffAlgorithm::Myers).unwrap();
        assert_valid(&hist)?;
        assert_valid(&myers)?;

        prop_assert!(
            sum_changes(&hist.ops) >= sum_changes(&myers.ops),
            "histogram produced fewer edits than minimal Myers"
        );

        let hist2 = diff_words(&old, &new, DiffAlgorithm::Histogram).unwrap();
        prop_assert_eq!(hist.ops, hist2.ops);
    }
}
