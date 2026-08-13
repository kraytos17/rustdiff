//! CSS for generated diff pages, organized as one concern per chunk.
//!
//! Chunks are individual `&'static str` consts assembled in `document.rs`.

/// Theme variable blocks: dark is the default, light overrides via `data-theme`.
pub(super) const THEME_STYLE: &str = concat!(
    ":root { ",
    "--bg:#0d1117; --panel:#161b22; --text:#c9d1d9; --border:#30363d; \
     --add-bg:#132c18; --del-bg:#2c1515; --add:#56d364; --del:#ff7b72; \
     --ln:#6e7681; --hunk:#1f6feb; --header:#30363d;",
    " }\n",
    ":root[data-theme=\"light\"] { ",
    "--bg:#ffffff; --panel:#f6f8fa; --text:#24292f; --border:#d0d7de; \
     --add-bg:#e6ffec; --del-bg:#ffebe9; --add:#1a7f37; --del:#cf222e; \
     --ln:#6e7781; --hunk:#0969da; --header:#d0d7de;",
    " }\n"
);

pub(super) const BASE_CSS: &str = r#"
* { box-sizing: border-box; }
body { background: var(--bg); color: var(--text);
       font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas,
                    "Liberation Mono", monospace;
       margin: 0; padding: 2rem; }
h2 { font-size: 1.1rem; margin: 0 0 0.5rem; }
pre { margin: 0; white-space: pre-wrap; overflow-wrap: break-word; }
code { font-family: inherit; }
del, ins { text-decoration: none; }
ins { color: var(--add); background: var(--add-bg); }
del { color: var(--del); background: var(--del-bg); }

.file-head { padding: 0.3rem 0.5rem; color: var(--ln);
             background: var(--panel); border-bottom: 1px solid var(--border); }

table { border-collapse: collapse; width: 100%; }
td { vertical-align: top; padding: 0; }

/* unified / numbered view */
tr.add { background: var(--add-bg); }
tr.del { background: var(--del-bg); }
tr.ctx:nth-child(even) { background: rgba(128,128,128,0.06); }
td.ln { width: 3.2em; color: var(--ln); text-align: right; padding-right: 0.6em;
        user-select: none; white-space: nowrap; }
td.ln.empty { border-right: 1px solid var(--border); }
td.txt { padding-left: 0.5em; }
tr.hunk td { color: var(--hunk); padding: 0.3em 0.5em;
             background: var(--panel); border-top: 1px solid var(--border);
             border-bottom: 1px solid var(--border); }

/* side-by-side view */
.cell { width: 50%; padding: 0.1rem 0.5rem; border: 1px solid var(--border); }
.cell.add { background: var(--add-bg); }
.cell.del { background: var(--del-bg); }
.cell .ln { display: inline-block; width: 3em; text-align: right; color: var(--ln);
            padding-right: 0.8em; user-select: none; }
thead th { text-align: center; color: var(--text); padding: 0.5rem;
           background: var(--panel); border-bottom: 1px solid var(--border); }

footer { text-align: center; color: var(--ln); font-size: 0.85rem;
         padding: 1rem; border-top: 1px solid var(--border); }
"#;

/// Toolbar and its buttons (theme, navigation, wrap).
pub(super) const TOOLBAR_CSS: &str = r"
.toolbar { text-align: right; margin-bottom: 0.5rem; }
.toolbar button { background: var(--panel); color: var(--text);
                  border: 1px solid var(--border); border-radius: 4px;
                  padding: 0.25rem 0.6rem; cursor: pointer; font: inherit;
                  margin-left: 0.25rem; }
.toolbar button:focus-visible { outline: 2px solid var(--hunk); outline-offset: 2px; }
";

/// Collapsible unchanged-region rows (numbered / side-by-side views).
pub(super) const COLLAPSE_CSS: &str = r"
tr.collapsed { display: none; }
tr.gap td { color: var(--ln); background: var(--panel); text-align: center;
            padding: 0.3rem; font-size: 0.85rem; border: 1px solid var(--border); }
tr.gap.hidden { display: none; }
";

/// Line-wrap toggle (off = horizontal scroll instead of wrapping).
pub(super) const WRAP_CSS: &str = r"
body.wrap-off pre { white-space: pre; overflow-x: auto; }
";

pub(super) const PRINT_CSS: &str = r"
@media print {
  :root {
    --bg:#ffffff; --panel:#f6f8fa; --text:#24292f; --border:#d0d7de;
    --add-bg:#e6ffec; --del-bg:#ffebe9; --add:#1a7f37; --del:#cf222e;
    --ln:#6e7781; --hunk:#0969da; --header:#d0d7de;
  }
  body { padding: 0; }
  footer { display: none; }
  tr.collapsed { display: table-row; }
  tr.gap { display: none; }
}
";

pub(super) const RESPONSIVE_CSS: &str = r"
@media (max-width: 640px) {
  td.cell { display: block; width: 100%; }
  thead { display: none; }
}
";
