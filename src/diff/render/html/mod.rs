//! HTML rendering: self-contained diff pages with view-time JavaScript (theme
//! toggle, change navigation, collapsible regions, line-wrap).
//!
//! The page shell is assembled in `document`, CSS lives in `css`, and the
//! inline scripts live in `js`.

mod css;
mod document;
mod js;

use crate::diff::data::{Diff, OpKind};
use crate::diff::render::unified::group_into_hunks;
use document::{esc, html_document};
use std::fmt::Write as _;

/// HTML color theme for generated diff pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum HtmlTheme {
    /// Dark theme (GitHub-dark style).
    Dark,
    /// Light theme (GitHub-light style).
    Light,
}

/// Theme selection for a generated page.
///
/// `None` means "follow the viewer's `prefers-color-scheme` at open time",
/// with the in-page toggle remembered via `localStorage`. `Some(theme)` bakes
/// that theme in as the authoritative default.
pub type ThemeOption = Option<HtmlTheme>;

/// Equal runs longer than this many rows are collapsed behind a "show" gap row
/// in the numbered and side-by-side views.
const COLLAPSE_THRESHOLD: usize = 6;

/// A "show N unchanged lines" gap row for a collapsed equal run.
fn gap_row(len: usize) -> String {
    format!(
        "<tr class=\"gap\"><td colspan=\"2\"><span class=\"gap-label\">{len} unchanged lines</span> <button class=\"expand\" type=\"button\">Show</button></td></tr>\n"
    )
}

/// Git-style unified HTML diff: file headers, hunk headers, tinted add/delete
/// rows, and per-row old/new line numbers.
#[must_use]
pub fn render_unified_html(
    diff: &Diff,
    context: usize,
    old_name: &str,
    new_name: &str,
    theme: ThemeOption,
) -> String {
    let mut body = String::new();
    writeln!(
        body,
        "<div class=\"file-head\"><code>--- {}</code></div>\n<div class=\"file-head\"><code>+++ {}</code></div>",
        esc(old_name),
        esc(new_name)
    )
    .unwrap();
    body.push_str("<table>\n");

    let hunks = group_into_hunks(&diff.ops, context);
    for hunk in hunks {
        writeln!(
            body,
            "<tr class=\"hunk\"><td colspan=\"3\"><pre>@@ -{},{} +{},{} @@</pre></td></tr>",
            hunk.start_a, hunk.len_a, hunk.start_b, hunk.len_b
        )
        .unwrap();

        let mut old_ln = hunk.start_a;
        let mut new_ln = hunk.start_b;
        for op in &hunk.ops {
            let tokens = diff.tokens_for(op.kind);
            let start = op.start as usize;
            for line in &tokens[start..start + op.len as usize] {
                match op.kind {
                    OpKind::Equal => {
                        writeln!(
                            body,
                            "<tr class=\"ctx\"><td class=\"ln\">{old_ln}</td><td class=\"ln\">{new_ln}</td><td class=\"txt\"><pre>{}</pre></td></tr>",
                            esc(line)
                        )
                        .unwrap();
                        old_ln += 1;
                        new_ln += 1;
                    }
                    OpKind::Delete => {
                        writeln!(
                            body,
                            "<tr class=\"del\" aria-label=\"deleted line\"><td class=\"ln\">{old_ln}</td><td class=\"ln empty\"></td><td class=\"txt\"><pre>{}</pre></td></tr>",
                            esc(line)
                        )
                        .unwrap();
                        old_ln += 1;
                    }
                    OpKind::Insert => {
                        writeln!(
                            body,
                            "<tr class=\"add\" aria-label=\"added line\"><td class=\"ln empty\"></td><td class=\"ln\">{new_ln}</td><td class=\"txt\"><pre>{}</pre></td></tr>",
                            esc(line)
                        )
                        .unwrap();
                        new_ln += 1;
                    }
                }
            }
        }
    }

    body.push_str("</table>\n");
    html_document(&format!("{old_name} \u{2192} {new_name}"), &body, theme)
}

/// Simple numbered listing with tinted add/delete rows. Long unchanged runs
/// are collapsed behind a "show" gap row.
#[must_use]
pub fn render_numbered_html(diff: &Diff, theme: ThemeOption) -> String {
    let mut body = String::new();
    body.push_str("<table>\n");

    let mut ln = 0usize;
    for op in &diff.ops {
        let tokens = diff.tokens_for(op.kind);
        let start = op.start as usize;
        let len = op.len as usize;
        let collapsible = op.kind == OpKind::Equal && len > COLLAPSE_THRESHOLD;
        if collapsible {
            body.push_str(&gap_row(len));
        }

        let base_class = match op.kind {
            OpKind::Equal => "ctx",
            OpKind::Delete => "del",
            OpKind::Insert => "add",
        };
        let class = if collapsible {
            "ctx collapsed"
        } else {
            base_class
        };

        let aria = match op.kind {
            OpKind::Delete => " aria-label=\"deleted line\"",
            OpKind::Insert => " aria-label=\"added line\"",
            OpKind::Equal => "",
        };
        for text in &tokens[start..start + len] {
            ln += 1;
            writeln!(
                body,
                "<tr class=\"{class}\"{aria}><td class=\"ln\">{ln}</td><td class=\"txt\"><pre>{}</pre></td></tr>",
                esc(text)
            )
            .unwrap();
        }
    }
    body.push_str("</table>\n");
    html_document("Diff", &body, theme)
}

/// Side-by-side HTML built from the op stream.
///
/// Adjacent Delete-to-Insert runs are paired into one row; insert-only and
/// delete-only rows leave the opposite cell empty. Alignment is structural,
/// never derived from line content.
#[allow(
    clippy::too_many_lines,
    reason = "one branch per op-stream shape (equal/paired/delete-only/insert-only)"
)]
#[must_use]
pub fn render_side_by_side_html(
    diff: &Diff,
    old_name: &str,
    new_name: &str,
    theme: ThemeOption,
) -> String {
    let mut body = String::new();
    writeln!(
        body,
        "<div class=\"file-head\"><code>{} \u{2192} {}</code></div>",
        esc(old_name),
        esc(new_name)
    )
    .unwrap();
    body.push_str("<table>\n<thead><tr><th>Old</th><th>New</th></tr></thead>\n<tbody>\n");

    let mut old_ln = 1;
    let mut new_ln = 1;
    let mut i = 0;
    while i < diff.ops.len() {
        let op = diff.ops[i];
        match op.kind {
            OpKind::Equal => {
                let start = op.start as usize;
                let len = op.len as usize;
                let collapsible = len > COLLAPSE_THRESHOLD;
                if collapsible {
                    body.push_str(&gap_row(len));
                }
                let row_class = if collapsible {
                    " class=\"ctx collapsed\""
                } else {
                    ""
                };

                for line in &diff.old_tokens[start..start + len] {
                    writeln!(
                        body,
                        "<tr{row_class}><td class=\"cell ctx\"><span class=\"ln\">{old_ln}</span><pre>{}</pre></td><td class=\"cell ctx\"><span class=\"ln\">{new_ln}</span><pre>{}</pre></td></tr>",
                        esc(line),
                        esc(line)
                    )
                    .unwrap();
                    old_ln += 1;
                    new_ln += 1;
                }
                i += 1;
            }
            OpKind::Delete if i + 1 < diff.ops.len() && diff.ops[i + 1].kind == OpKind::Insert => {
                let del = &diff.ops[i];
                let ins = &diff.ops[i + 1];
                let del_lines =
                    &diff.old_tokens[del.start as usize..(del.start + del.len) as usize];
                let ins_lines =
                    &diff.new_tokens[ins.start as usize..(ins.start + ins.len) as usize];
                for k in 0..del.len.max(ins.len) {
                    let k = k as usize;
                    let left = del_lines.get(k).map(|l| {
                        format!("<span class=\"ln\">{old_ln}</span><pre>{}</pre>", esc(l))
                    });
                    let right = ins_lines.get(k).map(|l| {
                        format!("<span class=\"ln\">{new_ln}</span><pre>{}</pre>", esc(l))
                    });

                    if left.is_some() {
                        old_ln += 1;
                    }
                    if right.is_some() {
                        new_ln += 1;
                    }

                    let left_cell = left.map_or_else(
                        || "<td class=\"cell del\" aria-label=\"deleted line\"></td>".to_string(),
                        |h| format!("<td class=\"cell del\" aria-label=\"deleted line\">{h}</td>"),
                    );
                    let right_cell = right.map_or_else(
                        || "<td class=\"cell add\" aria-label=\"added line\"></td>".to_string(),
                        |h| format!("<td class=\"cell add\" aria-label=\"added line\">{h}</td>"),
                    );
                    writeln!(body, "<tr class=\"chg\">{left_cell}{right_cell}</tr>").unwrap();
                }
                i += 2;
            }
            OpKind::Delete => {
                let start = op.start as usize;
                for line in &diff.old_tokens[start..start + op.len as usize] {
                    writeln!(
                        body,
                        "<tr class=\"chg\"><td class=\"cell del\" aria-label=\"deleted line\"><span class=\"ln\">{old_ln}</span><pre>{}</pre></td><td class=\"cell\"></td></tr>",
                        esc(line)
                    )
                    .unwrap();
                    old_ln += 1;
                }
                i += 1;
            }
            OpKind::Insert => {
                let start = op.start as usize;
                for line in &diff.new_tokens[start..start + op.len as usize] {
                    writeln!(
                        body,
                        "<tr class=\"chg\"><td class=\"cell\"></td><td class=\"cell add\" aria-label=\"added line\"><span class=\"ln\">{new_ln}</span><pre>{}</pre></td></tr>",
                        esc(line)
                    )
                    .unwrap();
                    new_ln += 1;
                }
                i += 1;
            }
        }
    }

    body.push_str("</tbody>\n</table>\n");
    html_document("Side-by-Side Diff", &body, theme)
}

/// Word-level inline HTML: per-line rows with changed words wrapped in
/// `<del>`/`<ins>`.
#[must_use]
pub fn render_word_html(diff: &Diff, theme: ThemeOption) -> String {
    let mut body = String::new();
    let mut line = String::new();
    for (kind, text) in diff.edits() {
        match kind {
            OpKind::Equal => line.push_str(&esc(text)),
            OpKind::Delete => write!(line, "<del>{}</del>", esc(text)).unwrap(),
            OpKind::Insert => write!(line, "<ins>{}</ins>", esc(text)).unwrap(),
        }
        if text.ends_with('\n') {
            writeln!(body, "<pre>{line}</pre>").unwrap();
            line.clear();
        }
    }
    if !line.is_empty() {
        writeln!(body, "<pre>{line}</pre>").unwrap();
    }
    html_document("Word Diff", &body, theme)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::data::Op;

    fn diff(ops: Vec<Op>, old: &[&str], new: &[&str]) -> Diff {
        Diff {
            ops,
            old_tokens: old.iter().copied().map(str::to_owned).collect(),
            new_tokens: new.iter().copied().map(str::to_owned).collect(),
        }
    }

    #[test]
    fn test_esc() {
        // html_escape's text mode escapes & < > (quotes are only special in
        // attributes, which we never populate from user data).
        assert_eq!(esc("a<b>&\"'"), "a&lt;b&gt;&amp;\"'");
        assert_eq!(esc("plain"), "plain");
    }

    #[test]
    fn test_unified_empty() {
        let d = diff(vec![], &[], &[]);
        let html = render_unified_html(&d, 3, "old", "new", Some(HtmlTheme::Dark));

        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("--- old"));
        assert!(html.contains("+++ new"));
        assert!(!html.contains("<tr class=\"ctx\""));
    }

    #[test]
    fn test_unified_simple_change() {
        let a = ["a", "b", "c"];
        let b = ["a", "X", "c"];
        let d = diff(
            vec![
                Op::equal(0, 1),
                Op::delete(1, 1),
                Op::insert(1, 1),
                Op::equal(2, 1),
            ],
            &a,
            &b,
        );

        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert!(html.contains("@@ -1,3 +1,3 @@"));
        assert!(html.contains("<tr class=\"del\""), "missing delete row");
        assert!(html.contains("<tr class=\"add\""), "missing add row");
        assert!(html.contains(">b</pre>"), "missing deleted content");
        assert!(html.contains(">X</pre>"), "missing inserted content");
    }

    #[test]
    fn test_unified_escapes_content() {
        let d = diff(vec![Op::insert(0, 1)], &[], &["<script>alert(1)</script>"]);
        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>alert"));
    }

    #[test]
    fn test_unified_delete_run_two_lines() {
        let d = diff(
            vec![Op::delete(0, 2), Op::insert(0, 1)],
            &["x", "y"],
            &["z"],
        );

        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert_eq!(html.matches("class=\"del\"").count(), 2);
        assert!(html.contains(">x</pre>"));
        assert!(html.contains(">y</pre>"));
    }

    #[test]
    fn test_numbered_rows() {
        let d = diff(vec![Op::equal(0, 1), Op::insert(1, 1)], &["a"], &["a", "b"]);
        let html = render_numbered_html(&d, Some(HtmlTheme::Dark));
        assert!(html.contains("<tr class=\"ctx\""));
        assert!(html.contains("<tr class=\"add\""));
    }

    #[test]
    fn test_side_by_side_pairs_replacement() {
        let d = diff(
            vec![
                Op::equal(0, 1),
                Op::delete(1, 1),
                Op::insert(1, 1),
                Op::equal(2, 1),
            ],
            &["a", "b", "c"],
            &["a", "X", "c"],
        );

        let html = render_side_by_side_html(&d, "o", "n", Some(HtmlTheme::Dark));
        // Paired row: both cells present, delete + add.
        assert!(html.contains("class=\"cell del\" aria-label"));
        assert!(html.contains("class=\"cell add\" aria-label"));
        // Content without -/+ markers.
        assert!(html.contains(">b</pre>"));
        assert!(html.contains(">X</pre>"));
        assert!(!html.contains(">+"));
    }

    #[test]
    fn test_side_by_side_delete_only() {
        let d = diff(vec![Op::delete(0, 1)], &["b"], &[]);
        let html = render_side_by_side_html(&d, "o", "n", Some(HtmlTheme::Dark));
        assert!(html.contains("class=\"cell del\" aria-label"));
        assert!(
            html.contains("<td class=\"cell\"></td>"),
            "empty right cell"
        );
    }

    #[test]
    fn test_side_by_side_insert_only() {
        let d = diff(vec![Op::insert(0, 1)], &[], &["X"]);
        let html = render_side_by_side_html(&d, "o", "n", Some(HtmlTheme::Dark));
        assert!(html.contains("<td class=\"cell\"></td>"), "empty left cell");
        assert!(html.contains("class=\"cell add\" aria-label"));
    }

    #[test]
    fn test_side_by_side_run_mismatch() {
        // Delete run of 2 paired with insert run of 1 → 2 rows; the second
        // row's right cell is empty.
        let d = diff(
            vec![Op::delete(0, 2), Op::insert(0, 1)],
            &["x", "y"],
            &["z"],
        );

        let html = render_side_by_side_html(&d, "o", "n", Some(HtmlTheme::Dark));
        assert_eq!(html.matches("class=\"cell del\" aria-label").count(), 2);
        assert_eq!(html.matches("class=\"cell add\" aria-label").count(), 2);
        assert!(
            html.contains("class=\"cell add\" aria-label=\"added line\"></td>"),
            "padded cell"
        );
    }

    #[test]
    fn test_side_by_side_content_starting_with_marker() {
        // A content line whose text begins with "-" must render as context.
        let d = diff(
            vec![Op::equal(0, 1), Op::delete(1, 1), Op::insert(1, 1)],
            &["-foo", "b"],
            &["-foo", "X"],
        );

        let html = render_side_by_side_html(&d, "o", "n", Some(HtmlTheme::Dark));
        assert!(
            html.contains("<td class=\"cell ctx\">"),
            "content line classified wrong"
        );
        assert!(html.contains(">-foo</pre>"));
    }

    #[test]
    fn test_word_inline() {
        let d = diff(
            vec![
                Op::equal(0, 1),
                Op::delete(1, 1),
                Op::insert(1, 1),
                Op::equal(2, 1),
            ],
            &["hello ", "world", "\n"],
            &["hello ", "rust", "\n"],
        );

        let html = render_word_html(&d, Some(HtmlTheme::Dark));
        assert!(html.contains("<del>world</del>"));
        assert!(html.contains("<ins>rust</ins>"));
        assert!(html.contains("hello "));
    }

    #[test]
    fn test_light_theme_vars() {
        let d = diff(vec![], &[], &[]);
        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Light));
        assert!(html.contains("--bg:#ffffff"), "light background missing");

        let dark = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert!(dark.contains("--bg:#0d1117"), "dark background missing");
    }

    #[test]
    fn test_baked_theme_authoritative() {
        let d = diff(vec![], &[], &[]);
        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Light));
        assert!(
            html.contains(r#"<html lang="en" data-theme="light">"#),
            "baked light theme must be authoritative"
        );
    }

    #[test]
    fn test_system_theme_uses_prefers_color_scheme() {
        let d = diff(vec![], &[], &[]);
        let html = render_unified_html(&d, 3, "o", "n", None);
        assert!(
            html.contains(r#"data-theme=""#),
            "no explicit theme means an empty data-theme default"
        );
        assert!(
            html.contains("prefers-color-scheme"),
            "system theme must consult prefers-color-scheme"
        );
    }

    #[test]
    fn test_theme_toggle_present() {
        let d = diff(vec![], &[], &[]);
        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert!(
            html.contains("id=\"theme-toggle\""),
            "toggle button missing"
        );
        assert!(html.contains("localStorage"), "theme persistence missing");
        assert!(html.contains("addEventListener"), "toggle script missing");
    }

    #[test]
    fn test_print_and_responsive_css() {
        let d = diff(vec![], &[], &[]);
        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert!(html.contains("@media print"), "print stylesheet missing");
        assert!(
            html.contains("@media (max-width: 640px)"),
            "responsive stylesheet missing"
        );
    }

    #[test]
    fn test_nav_buttons_present() {
        let d = diff(vec![], &[], &[]);
        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert!(html.contains("id=\"prev-change\""), "prev button missing");
        assert!(html.contains("id=\"next-change\""), "next button missing");
        assert!(html.contains("id=\"wrap-toggle\""), "wrap button missing");
        assert!(html.contains("scrollIntoView"), "nav JS missing");
        assert!(html.contains("keydown"), "keyboard handler missing");
    }

    #[test]
    fn test_side_by_side_marks_change_rows() {
        let d = diff(
            vec![Op::equal(0, 1), Op::delete(1, 1), Op::insert(1, 1)],
            &["a", "b"],
            &["a", "X"],
        );
        let html = render_side_by_side_html(&d, "o", "n", Some(HtmlTheme::Dark));
        assert!(html.contains("class=\"chg\""), "change row not marked");

        let equal_only = diff(vec![Op::equal(0, 1)], &["a"], &["a"]);
        let html = render_side_by_side_html(&equal_only, "o", "n", Some(HtmlTheme::Dark));
        assert!(
            !html.contains("class=\"chg\""),
            "equal-only must have no chg rows"
        );
    }

    #[test]
    fn test_numbered_long_equal_run_collapses() {
        // 10 equal lines (above COLLAPSE_THRESHOLD) then a change.
        let old: Vec<String> = (0..10).map(|i| format!("line {i}")).collect();
        let old_refs: Vec<&str> = old.iter().map(String::as_str).collect();
        let new: Vec<String> = (0..10).map(|i| format!("line {i}")).collect();
        let mut new_refs: Vec<&str> = new.iter().map(String::as_str).collect();
        new_refs.push("CHANGED");
        let d = Diff {
            ops: vec![Op::equal(0, 10), Op::insert(10, 1)],
            old_tokens: old_refs.iter().map(ToString::to_string).collect(),
            new_tokens: new_refs.iter().map(ToString::to_string).collect(),
        };
        let html = render_numbered_html(&d, Some(HtmlTheme::Dark));
        assert!(html.contains("class=\"gap\""), "gap row missing");
        assert!(
            html.contains("class=\"ctx collapsed\""),
            "collapsed rows missing"
        );
    }

    #[test]
    fn test_numbered_short_equal_run_not_collapsed() {
        let d = diff(
            vec![Op::equal(0, 3), Op::insert(3, 1)],
            &["a", "b", "c"],
            &["a", "b", "c", "d"],
        );
        let html = render_numbered_html(&d, Some(HtmlTheme::Dark));
        assert!(
            !html.contains("class=\"gap\""),
            "short run must not collapse"
        );
        assert!(
            !html.contains("ctx collapsed"),
            "no collapsed rows expected"
        );
    }

    #[test]
    fn test_side_by_side_long_equal_run_collapses() {
        let old: Vec<String> = (0..10).map(|i| format!("line {i}")).collect();
        let old_refs: Vec<&str> = old.iter().map(String::as_str).collect();
        let d = Diff {
            ops: vec![Op::equal(0, 10)],
            old_tokens: old_refs.iter().map(ToString::to_string).collect(),
            new_tokens: old_refs.iter().map(ToString::to_string).collect(),
        };
        let html = render_side_by_side_html(&d, "o", "n", Some(HtmlTheme::Dark));
        assert!(html.contains("class=\"gap\""), "gap row missing");
        assert!(html.contains("ctx collapsed"), "collapsed rows missing");
    }

    #[test]
    fn test_print_reveals_collapsed() {
        let d = diff(vec![], &[], &[]);
        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert!(
            html.contains("tr.collapsed { display: table-row; }"),
            "print must reveal collapsed rows"
        );
    }

    #[test]
    fn test_wrap_toggle_present() {
        let d = diff(vec![], &[], &[]);
        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert!(html.contains("wrap-off"), "wrap-off CSS missing");
        assert!(html.contains("rustdiff-wrap"), "wrap persistence missing");
    }

    #[test]
    fn test_xss_filenames_not_in_scripts() {
        let payload = "</script><script>alert(1)</script>";
        let d = diff(vec![Op::insert(0, 1)], &[], &["x"]);
        let html = render_unified_html(&d, 3, payload, "n", Some(HtmlTheme::Dark));
        assert!(
            !html.contains("</script><script>"),
            "script breakout from filename"
        );
        assert_eq!(
            html.matches("<script>").count(),
            2,
            "only the two static script blocks may exist"
        );
        assert!(
            html.contains("&lt;/script&gt;"),
            "filename must be escaped in the title/headers"
        );
    }

    #[test]
    fn test_accessibility_aria_labels() {
        let d = diff(vec![Op::delete(0, 1), Op::insert(0, 1)], &["old"], &["new"]);
        let unified = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert!(
            unified.contains("aria-label=\"deleted line\""),
            "unified delete row lacks aria-label"
        );
        assert!(
            unified.contains("aria-label=\"added line\""),
            "unified add row lacks aria-label"
        );

        let side = render_side_by_side_html(&d, "o", "n", Some(HtmlTheme::Dark));
        assert!(side.contains("aria-label=\"deleted line\""));
        assert!(side.contains("aria-label=\"added line\""));

        let numbered = render_numbered_html(&d, Some(HtmlTheme::Dark));
        assert!(numbered.contains("aria-label=\"deleted line\""));
        assert!(numbered.contains("aria-label=\"added line\""));
    }

    #[test]
    fn test_monospace_font_stack() {
        let d = diff(vec![], &[], &[]);
        let html = render_unified_html(&d, 3, "o", "n", Some(HtmlTheme::Dark));
        assert!(
            html.contains("ui-monospace, SFMono-Regular"),
            "proper monospace font stack missing"
        );
    }
}
