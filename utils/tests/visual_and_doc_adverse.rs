mod common;

use clap::{Arg, Command};
use clap_noun_verb_utils::{
    help::{format_box_text, format_table},
    markdown,
};

#[test]
fn test_markdown_adverse_subcommands() {
    // Subcommands with spaces, special characters, and nested hierarchies
    let cmd = Command::new("root-app")
        .version("1.2.3")
        .about("App with weird subcommands")
        .subcommand(
            Command::new("sub command space")
                .about("Subcommand with spaces")
                .arg(Arg::new("arg1").help("An argument").action(clap::ArgAction::Set))
                .subcommand(
                    Command::new("nested hierarchical sub").about("Deeply nested subcommand"),
                ),
        )
        .subcommand(
            Command::new("sub-@special!-characters").about("Subcommand with special characters"),
        )
        .subcommand(Command::new("sub-emoji-😀-and-🚀").about("Subcommand with emojis"))
        .subcommand(
            Command::new("sub-\"quotes\"-and-\\backslashes\\")
                .about("Subcommand with quotes and backslashes"),
        );

    let mut buf = Vec::new();
    markdown::generate_markdown(&cmd, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();

    println!("--- Generated Markdown ---");
    println!("{}", output);
    println!("--------------------------");

    // Assert that the subcommands exist in the markdown
    assert!(output.contains("# root-app"));
    assert!(output.contains("## sub command space"));
    assert!(output.contains("### nested hierarchical sub"));
    assert!(output.contains("## sub-@special!-characters"));
    assert!(output.contains("## sub-emoji-😀-and-🚀"));
    assert!(output.contains("## sub-\"quotes\"-and-\\backslashes\\"));

    // Verify table of contents and slug generation
    // slugify("sub command space") -> "sub-command-space"
    assert!(output.contains("- [`sub command space`](#sub-command-space)"));
    // slugify("nested hierarchical sub") -> "nested-hierarchical-sub"
    assert!(output.contains("- [`nested hierarchical sub`](#nested-hierarchical-sub)"));
    // slugify("sub-@special!-characters") -> "sub-special-characters" (ignoring @ and !)
    assert!(output.contains("- [`sub-@special!-characters`](#sub-special-characters)"));
    // slugify("sub-emoji-😀-and-🚀") -> "sub-emoji--and-" (ignoring emoji chars)
    assert!(output.contains("- [`sub-emoji-😀-and-🚀`](#sub-emoji--and-)"));
    // slugify("sub-\"quotes\"-and-\\backslashes\\") -> "sub-quotes-and-backslashes" (ignoring quotes and backslashes)
    assert!(
        output.contains("- [`sub-\"quotes\"-and-\\backslashes\\`](#sub-quotes-and-backslashes)")
    );
}

#[test]
fn test_layout_boxes_and_tables_adverse() {
    // 1. Test layout boxes
    // A. CJK Text
    let cjk_text = "こんにちは世界\n中文测试";
    let boxed_cjk = format_box_text(cjk_text);
    println!("--- Boxed CJK ---\n{}", boxed_cjk);
    // Width: "こんにちは世界" is 7 characters. Since they are hiragana/kanji, each is width 2. Total width: 14.
    // "中文测试" is 4 characters. Each is width 2. Total width: 8.
    // max_width should be 14.
    // Top border width: max_width + 2 = 16 dashes.
    // Check top border: ┌ + 16 dashes + ┐
    assert!(boxed_cjk.contains("┌────────────────┐"));
    assert!(boxed_cjk.contains("│ こんにちは世界 │"));
    assert!(boxed_cjk.contains("│ 中文测试       │")); // padded by 6 spaces

    // B. Emojis
    let emoji_text = "🦀🦀🦀\n😀 smiley 😃";
    let boxed_emoji = format_box_text(emoji_text);
    println!("--- Boxed Emoji ---\n{}", boxed_emoji);
    // "🦀🦀🦀": 3 emojis * 2 width = 6 width.
    // "😀 smiley 😃": "😀" (2) + " " (1) + "smiley" (6) + " " (1) + "😃" (2) = 12 width.
    // max_width should be 12.
    // Top border width: max_width + 2 = 14 dashes.
    assert!(boxed_emoji.contains("┌──────────────┐"));
    assert!(boxed_emoji.contains("│ 🦀🦀🦀       │")); // padded by 6 spaces, plus 1 space before │ = 7 spaces total
    assert!(boxed_emoji.contains("│ 😀 smiley 😃 │")); // padded by 0 spaces, plus 1 space before │ = 1 space total

    // C. Tab spacing
    let tab_text = "col1\tcol2\nshort\tlonger_col";
    let boxed_tab = format_box_text(tab_text);
    println!("--- Boxed Tab ---\n{}", boxed_tab);
    // Let's compute widths:
    // Tab expansion details:
    // For "col1\tcol2":
    // "c" (col=0), "o" (col=1), "l" (col=2), "1" (col=3)
    // "\t": col is 4. spaces = 4 - (4%4) = 4. spaces = 4. "    ". col becomes 8.
    // "c" (col=8), "o" (col=9), "l" (col=10), "2" (col=11)
    // Total width = 12. Total expanded: "col1    col2"
    // For "short\tlonger_col":
    // "s"(0), "h"(1), "o"(2), "r"(3), "t"(4)
    // "\t": col is 5. spaces = 4 - (5%4) = 3. "   ". col becomes 8.
    // "l"(8), "o"(9), "n"(10), "g"(11), "e"(12), "r"(13), "_"(14), "c"(15), "o"(16), "l"(17)
    // Total width = 18. Total expanded: "short   longer_col"
    // max_width should be 18.
    // Top border width: 18 + 2 = 20 dashes.
    assert!(boxed_tab.contains("┌────────────────────┐"));
    assert!(boxed_tab.contains("│ col1    col2       │"));
    assert!(boxed_tab.contains("│ short   longer_col │"));

    // D. Combining characters (umlauts/diaeresis)
    // "e\u{0308}" is "e" with combining diaeresis (appears as ë, width 1 in terminal).
    // Let's see how our library measures it.
    let comb_text = "e\u{0308}\nxx";
    let boxed_comb = format_box_text(comb_text);
    println!("--- Boxed Combining Characters ---\n{}", boxed_comb);

    // 2. Test Tables
    // A. CJK, Emojis, Tabs, and Multi-line cells with newlines
    let headers = vec!["Command", "Description / Details"];
    let rows = vec![
        vec![
            "start-🚀".to_string(),
            "Start the service immediately.\nSupports auto-restart.\nTabbed\tDetail.".to_string(),
        ],
        vec!["运行".to_string(), "Run CJK command.\n第二行".to_string()],
    ];

    let table_output = format_table(&headers, &rows);
    println!("--- Table Output ---\n{}", table_output);

    // Verify header and columns alignment
    assert!(table_output.contains("Command"));
    assert!(table_output.contains("Description / Details"));
    assert!(table_output.contains("start-🚀"));
    assert!(table_output.contains("运行"));
    assert!(table_output.contains("Start the service immediately."));
    assert!(table_output.contains("Supports auto-restart."));
    assert!(table_output.contains("Tabbed  Detail."));
    assert!(table_output.contains("Run CJK command."));
    assert!(table_output.contains("第二行"));
}
