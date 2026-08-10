use rapidhash::{HashMapExt, RapidHashMap};

/// Maps distinct tokens to dense `u32` IDs and back.
///
/// The diff algorithms compare token IDs (single-register equality on a
/// contiguous `Vec<u32>`) instead of scanning `&str` bytes, rendering resolves
/// IDs back to the original text via [`Interner::resolve`].
pub struct Interner<'a> {
    ids: RapidHashMap<&'a str, u32>,
    tokens: Vec<&'a str>,
}

impl<'a> Interner<'a> {
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            ids: RapidHashMap::with_capacity(cap),
            tokens: Vec::with_capacity(cap),
        }
    }

    /// Returns the ID for `s`, assigning a new dense ID if not yet present.
    ///
    /// # Panics
    ///
    /// Panics if the number of distinct tokens exceeds `u32::MAX`.
    pub fn intern(&mut self, s: &'a str) -> u32 {
        if let Some(&id) = self.ids.get(s) {
            return id;
        }

        let id = u32::try_from(self.tokens.len()).expect("distinct token count exceeds u32");
        self.tokens.push(s);
        self.ids.insert(s, id);
        id
    }

    /// Returns the token text for a previously assigned ID.
    #[must_use]
    pub fn resolve(&self, id: u32) -> &'a str {
        self.tokens[id as usize]
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.tokens.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Intern both sequences into a shared token space, returning the interner
/// plus each sequence as `Vec<u32>` of dense, contiguous token IDs.
#[must_use]
pub fn intern_both<'a>(a: &[&'a str], b: &[&'a str]) -> (Interner<'a>, Vec<u32>, Vec<u32>) {
    let mut interner = Interner::with_capacity(a.len() + b.len());
    let ia = a.iter().map(|&s| interner.intern(s)).collect();
    let ib = b.iter().map(|&s| interner.intern(s)).collect();
    (interner, ia, ib)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_dedup() {
        let mut interner = Interner::with_capacity(4);
        let a = interner.intern("hello");
        let b = interner.intern("hello");
        assert_eq!(a, b);
    }

    #[test]
    fn test_intern_distinct() {
        let mut interner = Interner::with_capacity(4);
        let a = interner.intern("foo");
        let b = interner.intern("bar");
        assert_ne!(a, b);
    }

    #[test]
    fn test_intern_dense_ids() {
        let mut interner = Interner::with_capacity(8);
        let ids: Vec<u32> = ["x", "y", "x", "z", "y"]
            .into_iter()
            .map(|s| interner.intern(s))
            .collect();
        assert_eq!(ids, vec![0, 1, 0, 2, 1]);
        assert_eq!(interner.len(), 3);
    }

    #[test]
    fn test_resolve_round_trip() {
        let mut interner = Interner::with_capacity(4);
        let id = interner.intern("round trip");
        assert_eq!(interner.resolve(id), "round trip");
    }

    #[test]
    fn test_intern_both_shared_space() {
        let a = ["a", "b", "c"];
        let b = ["b", "c", "d"];
        let (interner, ia, ib) = intern_both(&a, &b);
        assert_eq!(ia, vec![0, 1, 2]);
        assert_eq!(ib, vec![1, 2, 3]);
        assert_eq!(interner.len(), 4);
        assert_eq!(interner.resolve(1), "b");
        assert_eq!(interner.resolve(3), "d");
    }

    #[test]
    fn test_intern_both_empty() {
        let (interner, ia, ib) = intern_both(&[], &[]);
        assert!(interner.is_empty());
        assert!(ia.is_empty());
        assert!(ib.is_empty());
    }
}
