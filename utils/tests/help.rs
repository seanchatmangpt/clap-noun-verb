use clap_noun_verb_utils::help::{format_box_text, format_table, style_header, style_item};

#[test]
fn test_style_header() {
    let s = style_header("My Title");
    assert!(s.contains("My Title"));
    assert!(s.contains("\x1b[1m"));
}

#[test]
fn test_style_item() {
    let s = style_item("start", "Start the service");
    assert!(s.contains("start"));
    assert!(s.contains("Start the service"));
}

#[test]
fn test_format_box_text() {
    let s = format_box_text("Hello\nWorld");
    assert!(s.contains("┌───────┐"));
    assert!(s.contains("│ Hello │"));
    assert!(s.contains("│ World │"));
    assert!(s.contains("└───────┘"));
}

#[test]
fn test_format_table() {
    let headers = vec!["COMMAND", "DESCRIPTION"];
    let rows = vec![
        vec!["start".to_string(), "Start the server".to_string()],
        vec!["stop".to_string(), "Stop the server".to_string()],
    ];
    let table = format_table(&headers, &rows);
    assert!(table.contains("COMMAND"));
    assert!(table.contains("DESCRIPTION"));
    assert!(table.contains("start"));
    assert!(table.contains("Start the server"));
    assert!(table.contains("stop"));
    assert!(table.contains("Stop the server"));
}
