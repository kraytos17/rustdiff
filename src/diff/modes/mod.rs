//! Diff tokenization modes and options.

/// Line-mode tokenizer and `diff_lines` entry points.
pub mod line;
/// Word-mode tokenizer and `diff_words` entry points.
pub mod word;

pub use line::{diff_lines, diff_lines_with};
pub use word::{diff_words, diff_words_with};

#[cfg(test)]
mod proptests;

use clap::ValueEnum;
use std::borrow::Cow;

/// Which core algorithm to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DiffAlgorithm {
    /// Histogram anchoring (default): fastest on typical code, falls back to
    /// Myers for pathological inputs.
    Histogram,
    /// Linear-space Myers: minimal edit script, guaranteed.
    Myers,
}

/// Normalization applied to tokens before the diff core compares them.
///
/// Normalization only affects the *keys* fed to the core; the tokens stored in
/// [`crate::diff::data::Diff`] are kept verbatim so rendered output shows the
/// original text. Keys keep the same length and order as the original token
/// arrays, so op indices stay aligned with the render arrays.
#[derive(Debug, Default, Clone, Copy)]
pub struct DiffOptions {
    /// Ignore all whitespace within tokens (applies to line and word mode).
    pub ignore_whitespace: bool,
    /// Ignore case when comparing tokens.
    pub ignore_case: bool,
    /// Treat all blank lines as identical (line mode only; word mode disables
    /// this because line breaks are structural there).
    pub ignore_blank_lines: bool,
}

impl DiffOptions {
    /// Whether no normalization is requested (the common fast path).
    #[must_use]
    pub const fn is_identity(&self) -> bool {
        !self.ignore_whitespace && !self.ignore_case
    }
}

/// Produce the token keys fed to the diff core, borrowing the originals when no
/// normalization is requested and owning normalized copies otherwise. The
/// caller keeps the returned `Cow` alive and builds refs from it; keys have the
/// same length and order as the original token array, so op indices stay
/// aligned with the render arrays.
pub(crate) fn keys_for(tokens: &[String], opts: DiffOptions) -> Cow<'_, [String]> {
    if opts.is_identity() {
        Cow::Borrowed(tokens)
    } else {
        Cow::Owned(tokens.iter().map(|t| normalize_token(t, opts)).collect())
    }
}

fn normalize_token(token: &str, opts: DiffOptions) -> String {
    if opts.ignore_whitespace {
        let stripped: String = token.split_whitespace().collect();
        if opts.ignore_case {
            stripped.to_lowercase()
        } else {
            stripped
        }
    } else if opts.ignore_case {
        token.to_lowercase()
    } else {
        token.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_token_identity() {
        assert_eq!(normalize_token(" Foo ", DiffOptions::default()), " Foo ");
    }

    #[test]
    fn test_normalize_token_ignore_whitespace() {
        let opts = DiffOptions {
            ignore_whitespace: true,
            ignore_case: false,
            ignore_blank_lines: false,
        };
        assert_eq!(normalize_token("a \t b", opts), "ab");
        assert_eq!(normalize_token("  ", opts), "");
    }

    #[test]
    fn test_normalize_token_ignore_case() {
        let opts = DiffOptions {
            ignore_whitespace: false,
            ignore_case: true,
            ignore_blank_lines: false,
        };
        assert_eq!(normalize_token("Hello", opts), "hello");
    }

    #[test]
    fn test_normalize_token_compose() {
        let opts = DiffOptions {
            ignore_whitespace: true,
            ignore_case: true,
            ignore_blank_lines: false,
        };
        assert_eq!(normalize_token(" HeLLo ", opts), "hello");
    }

    #[test]
    fn test_keys_for_identity_borrows_originals() {
        let tokens = vec!["a".to_string(), "b".to_string()];
        let keys = keys_for(&tokens, DiffOptions::default());
        assert!(matches!(keys, Cow::Borrowed(_)));

        let refs: Vec<&str> = keys.iter().map(String::as_str).collect();
        assert_eq!(refs, vec!["a", "b"]);
    }

    #[test]
    fn test_keys_for_normalized_keeps_length() {
        let tokens = vec!["A ".to_string(), "\n".to_string()];
        let keys = keys_for(
            &tokens,
            DiffOptions {
                ignore_whitespace: true,
                ignore_case: true,
                ignore_blank_lines: false,
            },
        );

        assert!(matches!(keys, Cow::Owned(_)));
        assert_eq!(&*keys, &["a".to_string(), String::new()]);
        let refs: Vec<&str> = keys.iter().map(String::as_str).collect();
        assert_eq!(refs, vec!["a", ""]);
    }
}
