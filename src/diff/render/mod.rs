pub mod line;
pub mod side_by_side;
pub mod unified;
pub mod word;

pub use line::render_line_diff;
pub use side_by_side::render_side_by_side_html;
pub use unified::render_unified_diff;
pub use word::render_word_diff;

use ansi_to_html::convert;
use std::fs::File;
use std::io::{self, Write};

pub fn render_diff_outputs(diff_text: &str, base_name: &str) -> io::Result<()> {
    let diff_path = format!("{base_name}.diff");
    let mut diff_file = File::create(&diff_path)?;
    diff_file.write_all(diff_text.as_bytes())?;

    if let Ok(html_body) = convert(diff_text) {
        let numbered_lines: String = html_body
            .lines()
            .enumerate()
            .map(|(i, line)| {
                format!(
                    r#"<tr><td class="num">{}</td><td class="line">{line}</td></tr>"#,
                    i + 1
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let html_path = format!("{base_name}.html");
        let mut html_file = File::create(&html_path)?;

        html_file.write_all(
            format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Diff Viewer</title>
<style>
body {{
  font-family: monospace;
  background-color: #1e1e1e;
  color: #dcdcdc;
  padding: 1em;
  line-height: 1.4;
}}
table.diff {{
  border-collapse: collapse;
  width: 100%;
}}
td.num {{
  width: 3em;
  color: #888;
  text-align: right;
  padding-right: 1em;
  border-right: 1px solid #333;
  vertical-align: top;
  user-select: none;
}}
td.line {{
  white-space: pre-wrap;
  padding-left: 1em;
}}
.diff-insert {{ color: #00c853; }}
.diff-delete {{ color: #ff5252; }}
</style>
</head>
<body>
<table class="diff">
{numbered_lines}
</table>
</body>
</html>"#
            )
            .as_bytes(),
        )?;
    }

    Ok(())
}
