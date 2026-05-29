/// Helper to print a bold, styled section header.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::help::style_header;
///
/// let header = style_header("Usage");
/// assert_eq!(header, "\x1b[1m\x1b[34mUsage\x1b[0m");
/// ```
pub fn style_header(title: &str) -> String {
    format!("\x1b[1m\x1b[34m{}\x1b[0m", title)
}

/// Helper to print styled command/argument descriptions.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::help::style_item;
///
/// let item = style_item("help", "Print help message");
/// assert_eq!(item, "  \x1b[32mhelp              \x1b[0m Print help message");
/// ```
pub fn style_item(name: &str, description: &str) -> String {
    format!("  \x1b[32m{: <18}\x1b[0m {}", name, description)
}

fn char_display_width(c: char) -> usize {
    let cp = c as u32;
    if (0x4E00..=0x9FFF).contains(&cp) // CJK Unified Ideographs
        || (0x3400..=0x4DBF).contains(&cp) // CJK Ext A
        || (0x20000..=0x2EBEF).contains(&cp) // CJK Ext B-F
        || (0xF900..=0xFAFF).contains(&cp) // CJK Compatibility
        || (0xFF00..=0xFFEE).contains(&cp) // Fullwidth forms
        || (0x3000..=0x303F).contains(&cp) // CJK Symbols and Punctuation
        || (0x3040..=0x309F).contains(&cp) // Hiragana
        || (0x30A0..=0x30FF).contains(&cp) // Katakana
        || (0xAC00..=0xD7AF).contains(&cp) // Hangul Syllables
        || (0x1F300..=0x1FAFF).contains(&cp)
    // Emojis / Pictographs
    {
        2
    } else {
        1
    }
}

/// Expands tabs to spaces with a tab width of 4.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::help::expand_line;
///
/// let expanded = expand_line("a\tb");
/// assert_eq!(expanded, "a   b");
/// ```
pub fn expand_line(line: &str) -> String {
    let mut expanded = String::new();
    let mut col = 0;
    for c in line.chars() {
        if c == '\t' {
            let spaces = 4 - (col % 4);
            expanded.push_str(&" ".repeat(spaces));
            col += spaces;
        } else {
            expanded.push(c);
            col += char_display_width(c);
        }
    }
    expanded
}

/// Returns the display width of a string, taking into account tabs and CJK/emoji characters.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::help::display_width;
///
/// assert_eq!(display_width("abc"), 3);
/// assert_eq!(display_width("a\tb"), 5);
/// assert_eq!(display_width("你好"), 4);
/// ```
pub fn display_width(s: &str) -> usize {
    let mut col = 0;
    for c in s.chars() {
        if c == '\t' {
            let spaces = 4 - (col % 4);
            col += spaces;
        } else {
            col += char_display_width(c);
        }
    }
    col
}

/// Helper to format a block of text within an ASCII box.
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::help::format_box_text;
///
/// let boxed = format_box_text("Hello");
/// let expected = "┌───────┐\n│ Hello │\n└───────┘\n";
/// assert_eq!(boxed, expected);
/// ```
pub fn format_box_text(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let expanded_lines: Vec<String> = lines.iter().map(|l| expand_line(l)).collect();
    let max_width = expanded_lines.iter().map(|l| display_width(l)).max().unwrap_or(0);

    let mut boxed = String::new();
    boxed.push('┌');
    boxed.push_str(&"─".repeat(max_width + 2));
    boxed.push_str("┐\n");

    for line in expanded_lines {
        let width = display_width(&line);
        let padding = " ".repeat(max_width - width);
        boxed.push_str(&format!("│ {}{} │\n", line, padding));
    }

    boxed.push('└');
    boxed.push_str(&"─".repeat(max_width + 2));
    boxed.push_str("┘\n");
    boxed
}

/// Helper to format a table of values (e.g. commands or parameters).
///
/// # Examples
///
/// ```
/// use clap_noun_verb_utils::help::format_table;
///
/// let headers = vec!["Command", "Description"];
/// let rows = vec![
///     vec!["run".to_string(), "Execute".to_string()],
///     vec!["stop".to_string(), "Stop".to_string()],
/// ];
/// let table = format_table(&headers, &rows);
/// assert!(table.contains("Command"));
/// assert!(table.contains("run"));
/// ```
pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if headers.is_empty() {
        return String::new();
    }

    let col_count = headers.len();
    let mut col_widths = vec![0; col_count];

    let expanded_headers: Vec<String> = headers.iter().map(|h| expand_line(h)).collect();
    for (i, header) in expanded_headers.iter().enumerate() {
        col_widths[i] = display_width(header);
    }

    let mut processed_rows: Vec<Vec<Vec<String>>> = Vec::new();
    for row in rows {
        let mut processed_row = vec![Vec::new(); col_count];
        for (i, val) in row.iter().enumerate() {
            if i < col_count {
                let lines: Vec<String> = if val.is_empty() {
                    vec![String::new()]
                } else {
                    val.lines().map(expand_line).collect()
                };
                for line in &lines {
                    let w = display_width(line);
                    if w > col_widths[i] {
                        col_widths[i] = w;
                    }
                }
                processed_row[i] = lines;
            }
        }
        for item in processed_row.iter_mut().take(col_count).skip(row.len()) {
            *item = vec![String::new()];
        }
        processed_rows.push(processed_row);
    }

    let mut output = String::new();

    // Header row
    for (i, header) in expanded_headers.iter().enumerate() {
        let w = display_width(header);
        let padding = " ".repeat(col_widths[i] - w);
        output.push_str(&format!("{}{}", header, padding));
        output.push(' ');
    }
    output.push('\n');

    // Separator row
    for width in &col_widths {
        output.push_str(&"-".repeat(*width));
        output.push(' ');
    }
    output.push('\n');

    // Rows
    for processed_row in processed_rows {
        let height = processed_row.iter().map(|col| col.len()).max().unwrap_or(0);
        for line_idx in 0..height {
            for i in 0..col_count {
                let line_text = processed_row[i].get(line_idx).map(|s| s.as_str()).unwrap_or("");
                let w = display_width(line_text);
                let padding = " ".repeat(col_widths[i] - w);
                output.push_str(&format!("{}{}", line_text, padding));
                output.push(' ');
            }
            output.push('\n');
        }
    }

    output
}
