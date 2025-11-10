use ansi_to_html::convert;
use html_escape::encode_text;
use std::fs::File;
use std::io::{self, Write};

/// Convert an ANSI-colored diff to a full-width, side-by-side HTML layout.
pub fn render_side_by_side_html(diff_text: &str, base_name: &str) -> io::Result<()> {
    let html_path = format!("{base_name}_side_by_side.html");
    let mut html_file = File::create(&html_path)?;

    let (left_lines, right_lines) = split_diff_into_columns(diff_text);
    write_html_header(&mut html_file)?;
    write_html_rows(&mut html_file, &left_lines, &right_lines)?;
    write_html_footer(&mut html_file)?;

    Ok(())
}

fn split_diff_into_columns(diff_text: &str) -> (Vec<Option<String>>, Vec<Option<String>>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut lines = diff_text.lines().peekable();

    while let Some(line) = lines.next() {
        match line.chars().next() {
            Some('-') => {
                if let Some(next) = lines.peek()
                    && next.starts_with('+')
                {
                    let add_line = lines.next().unwrap();
                    left.push(Some(line.to_string()));
                    right.push(Some(add_line.to_string()));
                    continue;
                }
                left.push(Some(line.to_string()));
                right.push(None);
            }
            Some('+') => {
                left.push(None);
                right.push(Some(line.to_string()));
            }
            _ => {
                left.push(Some(line.to_string()));
                right.push(Some(line.to_string()));
            }
        }
    }

    (left, right)
}

fn write_html_header(file: &mut File) -> io::Result<()> {
    write!(
        file,
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>RustDiff – Side-by-Side Diff</title>
<style>
:root {{
  --bg: #0d1117;
  --panel: #161b22;
  --text: #c9d1d9;
  --border: #30363d;
  --header: #1f6feb;
  --del-bg: #2c1515;
  --add-bg: #132c18;
  --del-color: #ff7b72;
  --add-color: #56d364;
}}

body {{
  background: var(--bg);
  color: var(--text);
  font-family: "Fira Code", monospace;
  margin: 0;
  padding: 2rem;
  display: flex;
  justify-content: center;
}}

.container {{
  width: 100%;
  max-width: 1300px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 0 10px rgba(0,0,0,0.3);
}}

h2 {{
  background: var(--header);
  color: white;
  text-align: center;
  margin: 0;
  padding: 1rem;
  font-weight: 600;
}}

table {{
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}}

thead th {{
  background: var(--panel);
  color: #8ab4f8;
  text-align: center;
  padding: 0.7rem;
  font-size: 1rem;
  border-bottom: 1px solid var(--border);
  width: 50%;
}}

tbody td {{
  width: 50%;
  vertical-align: top;
  border-right: 1px solid var(--border);
  padding: 0;
}}

tr:not(:last-child) td {{
  border-bottom: none;
}}

tbody td:last-child {{
  border-right: none;
}}

.diff-line {{
  display: flex;
  align-items: flex-start;
  padding: 0.2rem 0.5rem;
  white-space: pre-wrap;
  overflow-wrap: break-word;
}}

.line-num {{
  width: 3rem;
  text-align: right;
  color: #6e7681;
  padding-right: 0.8rem;
  user-select: none;
  flex-shrink: 0;
}}

.text {{
  flex: 1;
  text-align: left;
}}

.deleted {{ background: var(--del-bg); color: var(--del-color); }}
.added {{ background: var(--add-bg); color: var(--add-color); }}
.context {{ background: transparent; color: var(--text); }}
.context:nth-child(even) {{
  background-color: rgba(255, 255, 255, 0.02);
}}

tr:hover td {{
  background-color: rgba(255,255,255,0.03);
}}

footer {{
  text-align: center;
  color: #6e7681;
  font-size: 0.85rem;
  padding: 1rem;
  border-top: 1px solid var(--border);
}}
</style>
</head>
<body>
<div class="container">
<h2>Side-by-Side Diff</h2>
<table>
<thead>
<tr><th>Old File</th><th>New File</th></tr>
</thead>
<tbody>
"#
    )
}

fn write_html_rows(
    file: &mut File,
    left_lines: &[Option<String>],
    right_lines: &[Option<String>],
) -> io::Result<()> {
    let mut left_num = 1;
    let mut right_num = 1;

    for (left, right) in left_lines.iter().zip(right_lines) {
        writeln!(file, "<tr>")?;
        if let Some(text) = left {
            let html = ansi_to_html_line(text);
            let class = if text.starts_with('-') {
                "deleted"
            } else {
                "context"
            };
            writeln!(
                file,
                r#"<td><div class="diff-line {class}"><span class="line-num">{left_num}</span><span class="text">{html}</span></div></td>"#
            )?;
            left_num += 1;
        } else {
            writeln!(
                file,
                r#"<td><div class="diff-line"><span class="line-num"></span><span class="text"></span></div></td>"#
            )?;
        }

        if let Some(text) = right {
            let html = ansi_to_html_line(text);
            let class = if text.starts_with('+') {
                "added"
            } else {
                "context"
            };
            writeln!(
                file,
                r#"<td><div class="diff-line {class}"><span class="line-num">{right_num}</span><span class="text">{html}</span></div></td>"#
            )?;
            right_num += 1;
        } else {
            writeln!(
                file,
                r#"<td><div class="diff-line"><span class="line-num"></span><span class="text"></span></div></td>"#
            )?;
        }

        writeln!(file, "</tr>")?;
    }

    Ok(())
}

fn write_html_footer(file: &mut File) -> io::Result<()> {
    write!(
        file,
        r#"</tbody>
</table>
<footer>Generated by <b>rustdiff</b></footer>
</div>
</body>
</html>"#
    )
}

fn ansi_to_html_line(line: &str) -> String {
    match convert(line) {
        Ok(html) => html,
        Err(_) => encode_text(line).to_string(),
    }
}
