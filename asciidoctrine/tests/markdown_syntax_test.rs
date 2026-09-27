use anyhow::Result;
use asciidoctrine::reader::markdown::MarkdownReader;
use asciidoctrine::{self, *};
use clap::Parser;
use pretty_assertions::assert_eq;

#[test]
fn parse_empty_document() -> Result<()> {
    check("", expected("", vec![]))
}

#[test]
fn parse_whitespace_only() -> Result<()> {
    check("  ", expected("  ", vec![]))
}

// --------------------------------------------------------------------------
// Headers
// --------------------------------------------------------------------------

#[test]
fn parse_basic_header() -> Result<()> {
    let input = "# test\n";

    check(input, expected(input, vec![node(input, "# test\n", Element::Title { level: 1 }, vec![text(input, "test")])]))
}

#[test]
fn parse_all_header_levels() -> Result<()> {
    let input = r#"# Level 1
## Level 2
### Level 3
#### Level 4
##### Level 5
###### Level 6
"#;

    let titles = (1..=6).map(|level| {
        let prefix = format!("{} ", "#".repeat(level));
        let line = input.lines().find(|line| line.starts_with(&prefix)).unwrap();
        let start = input.find(line).unwrap();
        at(input, start, &input[start..start + line.len() + 1], Element::Title { level: level as u32 },
            vec![text(input, line.strip_prefix(&prefix).unwrap())])
    }).collect();
    check(input, expected(input, titles))
}

#[test]
fn parse_setext_headers() -> Result<()> {
    let input = r#"Level 1
=======

Level 2
-------
"#;

    check(input, expected(input, vec![
        node(input, "Level 1\n=======\n", Element::Title { level: 1 }, vec![text(input, "Level 1")]),
        node(input, "Level 2\n-------\n", Element::Title { level: 2 }, vec![text(input, "Level 2")]),
    ]))
}

// --------------------------------------------------------------------------
// Paragraphs
// --------------------------------------------------------------------------

#[test]
fn parse_basic_paragraph() -> Result<()> {
    let input = "This is a basic paragraph.\n";
    check(input, expected(input, vec![plain(input, "This is a basic paragraph.")]))
}

#[test]
fn parse_multiple_paragraphs() -> Result<()> {
    let input = r#"First paragraph.

Second paragraph.

Third paragraph.
"#;

    check(input, expected(input, vec![
        plain(input, "First paragraph."), plain(input, "Second paragraph."), plain(input, "Third paragraph."),
    ]))
}

// --------------------------------------------------------------------------
// Links
// --------------------------------------------------------------------------

#[test]
fn parse_inline_link() -> Result<()> {
    let input = "This has a [link](https://example.com) in it.\n";
    let linked = with_string_attr(link(input, "[link](https://example.com)", "link", "https://example.com"), "protocol", "https");
    check(input, expected(input, vec![paragraph(input, "This has a [link](https://example.com) in it.", vec![
        text(input, "This has a "), linked, text(input, " in it."),
    ])]))
}

#[test]
fn parse_link_with_title() -> Result<()> {
    let input = r#"[Link](https://example.com "Title text")"#;
    let linked = with_string_attr(with_string_attr(link(input, input, "Link", "https://example.com"), "title", "Title text"), "protocol", "https");
    check(input, expected(input, vec![paragraph(input, input, vec![linked])]))
}

#[test]
fn parse_autolink() -> Result<()> {
    let input = "<https://example.com>\n";
    let linked = with_string_attr(link(input, "<https://example.com>", "https://example.com", "https://example.com"), "protocol", "https");
    check(input, expected(input, vec![paragraph(input, "<https://example.com>", vec![linked])]))
}

// --------------------------------------------------------------------------
// Images
// --------------------------------------------------------------------------

#[test]
fn parse_image() -> Result<()> {
    let input = "![Alt text](image.png)\n";
    let image = with_string_attr(node(input, "![Alt text](image.png)", Element::Image, vec![text(input, "Alt text")]), "path", "image.png");
    check(input, expected(input, vec![paragraph(input, "![Alt text](image.png)", vec![image])]))
}

#[test]
fn parse_image_with_title() -> Result<()> {
    let input = "![Alt text](image.png \"Image title\")";
    let mut image = with_string_attr(node(input, input, Element::Image, vec![text(input, "Alt text")]), "path", "image.png");
    image.positional_attributes.push(AttributeValue::String("Image title".into()));
    check(input, expected(input, vec![paragraph(input, input, vec![image])]))
}

// --------------------------------------------------------------------------
// Inline Formatting
// --------------------------------------------------------------------------

#[test]
fn parse_bold() -> Result<()> {
    let input = "This is **bold** text.\n";
    check(input, expected(input, vec![paragraph(input, "This is **bold** text.", vec![
        text(input, "This is "), styled(input, "**bold**", "strong", vec![text(input, "bold")]), text(input, " text."),
    ])]))
}

#[test]
fn parse_italic() -> Result<()> {
    let input = "This is *italic* text.\n";
    check(input, expected(input, vec![paragraph(input, "This is *italic* text.", vec![
        text(input, "This is "), styled(input, "*italic*", "em", vec![text(input, "italic")]), text(input, " text."),
    ])]))
}

#[test]
fn parse_inline_code() -> Result<()> {
    let input = "This has `inline code` in it.\n";
    check(input, expected(input, vec![paragraph(input, "This has `inline code` in it.", vec![
        text(input, "This has "), with_string_attr(styled(input, "`inline code`", "monospaced", vec![]), "content", "inline code"), text(input, " in it."),
    ])]))
}

#[test]
fn parse_strikethrough() -> Result<()> {
    let input = "This is ~~strikethrough~~ text.\n";
    check(input, expected(input, vec![paragraph(input, "This is ~~strikethrough~~ text.", vec![
        text(input, "This is "), styled(input, "~~strikethrough~~", "strikethrough", vec![text(input, "strikethrough")]), text(input, " text."),
    ])]))
}

#[test]
fn parse_combined_formatting() -> Result<()> {
    let input = "This has **bold** and *italic* and `code` together.\n";
    check(input, expected(input, vec![paragraph(input, "This has **bold** and *italic* and `code` together.", vec![
        text(input, "This has "), styled(input, "**bold**", "strong", vec![text(input, "bold")]),
        text(input, " and "), styled(input, "*italic*", "em", vec![text(input, "italic")]),
        at(input, input.find(" and `code`").unwrap(), " and ", Element::Text, vec![]),
        with_string_attr(styled(input, "`code`", "monospaced", vec![]), "content", "code"),
        text(input, " together."),
    ])]))
}

// --------------------------------------------------------------------------
// Code Blocks
// --------------------------------------------------------------------------

#[test]
fn parse_fenced_code_block() -> Result<()> {
    let input = r#"```
code here
more code
```
"#;
    let block = with_string_attr(node(input, &input[..input.len() - 1], Element::TypedBlock { kind: BlockType::Listing }, vec![]), "content", "code here\nmore code\n");
    check(input, expected(input, vec![block]))
}

#[test]
fn parse_code_block_with_language() -> Result<()> {
    let input = r#"```rust
fn main() {
    println!("Hello");
}
```
"#;
    let mut block = with_string_attr(node(input, &input[..input.len() - 1], Element::TypedBlock { kind: BlockType::Listing }, vec![]),
        "content", "fn main() {\n    println!(\"Hello\");\n}\n");
    block.positional_attributes = vec![AttributeValue::String("source".into()), AttributeValue::String("rust".into())];
    check(input, expected(input, vec![block]))
}

#[test]
fn parse_indented_code_block() -> Result<()> {
    let input = r#"Normal paragraph.

    indented code
    more code

Back to normal.
"#;

    let code = with_string_attr(node(input, "indented code\n    more code\n", Element::TypedBlock { kind: BlockType::Listing }, vec![]),
        "content", "indented code\nmore code\n");
    check(input, expected(input, vec![plain(input, "Normal paragraph."), code, plain(input, "Back to normal.")]))
}

// --------------------------------------------------------------------------
// Lists
// --------------------------------------------------------------------------

#[test]
fn parse_bullet_list() -> Result<()> {
    let input = r#"- Item 1
- Item 2
- Item 3
"#;
    check(input, expected(input, vec![node(input, input, Element::List(ListType::Bullet), vec![
        list_item(input, "- Item 1\n", vec![text(input, "Item 1")]),
        list_item(input, "- Item 2\n", vec![text(input, "Item 2")]),
        list_item(input, "- Item 3\n", vec![text(input, "Item 3")]),
    ])]))
}

#[test]
fn parse_numbered_list() -> Result<()> {
    let input = r#"1. First item
2. Second item
3. Third item
"#;
    check(input, expected(input, vec![node(input, input, Element::List(ListType::Number), vec![
        list_item(input, "1. First item\n", vec![text(input, "First item")]),
        list_item(input, "2. Second item\n", vec![text(input, "Second item")]),
        list_item(input, "3. Third item\n", vec![text(input, "Third item")]),
    ])]))
}

#[test]
fn parse_nested_list() -> Result<()> {
    let input = r#"- Item 1
  - Nested 1.1
  - Nested 1.2
- Item 2
  - Nested 2.1
"#;

    check(input, expected(input, vec![node(input, input, Element::List(ListType::Bullet), vec![
        list_item(input, "- Item 1\n  - Nested 1.1\n  - Nested 1.2\n", vec![
            text(input, "Item 1"), node(input, "  - Nested 1.1\n  - Nested 1.2\n", Element::List(ListType::Bullet), vec![
                list_item(input, "  - Nested 1.1\n", vec![text(input, "Nested 1.1")]),
                list_item(input, "  - Nested 1.2\n", vec![text(input, "Nested 1.2")]),
            ]),
        ]),
        list_item(input, "- Item 2\n  - Nested 2.1\n", vec![
            text(input, "Item 2"), node(input, "  - Nested 2.1\n", Element::List(ListType::Bullet), vec![
                list_item(input, "  - Nested 2.1\n", vec![text(input, "Nested 2.1")]),
            ]),
        ]),
    ])]))
}

#[test]
fn parse_mixed_list() -> Result<()> {
    let input = r#"- Bullet item
  1. Numbered sub-item
  2. Another numbered
- Another bullet
"#;

    check(input, expected(input, vec![node(input, input, Element::List(ListType::Bullet), vec![
        list_item(input, "- Bullet item\n  1. Numbered sub-item\n  2. Another numbered\n", vec![
            text(input, "Bullet item"), node(input, "  1. Numbered sub-item\n  2. Another numbered\n", Element::List(ListType::Number), vec![
                list_item(input, "  1. Numbered sub-item\n", vec![text(input, "Numbered sub-item")]),
                list_item(input, "  2. Another numbered\n", vec![text(input, "Another numbered")]),
            ]),
        ]),
        list_item(input, "- Another bullet\n", vec![text(input, "Another bullet")]),
    ])]))
}

#[test]
fn parse_task_list() -> Result<()> {
    let input = r#"- [x] Completed task
- [ ] Incomplete task
- [x] Another completed
"#;
    check(input, expected(input, vec![node(input, input, Element::List(ListType::Bullet), vec![
        with_string_attr(list_item(input, "- [x] Completed task\n", vec![text(input, "Completed task")]), "checked", "true"),
        with_string_attr(list_item(input, "- [ ] Incomplete task\n", vec![text(input, "Incomplete task")]), "checked", "false"),
        with_string_attr(list_item(input, "- [x] Another completed\n", vec![text(input, "Another completed")]), "checked", "true"),
    ])]))
}

// --------------------------------------------------------------------------
// Blockquotes
// --------------------------------------------------------------------------

#[test]
fn parse_blockquote() -> Result<()> {
    let input = r#"> This is a quote.
> It spans multiple lines.
"#;
    let quote = node(input, input, Element::TypedBlock { kind: BlockType::Quote }, vec![
        paragraph(input, "This is a quote.\n> It spans multiple lines.", vec![
            text(input, "This is a quote."), node(input, "\n", Element::Text, vec![]),
            text(input, "It spans multiple lines."),
        ]),
    ]);
    check(input, expected(input, vec![quote]))
}

#[test]
fn parse_nested_blockquote() -> Result<()> {
    let input = r#"> Outer quote
> > Nested quote
> Back to outer
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(
        ast.elements[0].element,
        Element::TypedBlock {
            kind: BlockType::Quote
        }
    );

    // Should contain a nested quote
    let has_nested = ast.elements[0].children.iter()
        .any(|c| matches!(c.element, Element::TypedBlock { kind: BlockType::Quote }));

    assert!(has_nested);

    Ok(())
}

// --------------------------------------------------------------------------
// Tables
// --------------------------------------------------------------------------

#[test]
fn parse_basic_table() -> Result<()> {
    let input = r#"| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::Table);

    // Should have rows (header + 2 data rows)
    assert!(ast.elements[0].children.len() >= 2);

    // First should be header row
    let first_row = &ast.elements[0].children[0];
    assert_eq!(first_row.element, Element::TableRow);

    Ok(())
}

#[test]
fn parse_table_with_alignment() -> Result<()> {
    let input = r#"| Left | Center | Right |
|:-----|:------:|------:|
| L    | C      | R     |
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::Table);

    Ok(())
}

#[test]
fn parse_table_without_header() -> Result<()> {
    let input = r#"| Cell 1 | Cell 2 |
| Cell 3 | Cell 4 |
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    // Without pipe alignment row, this might not be parsed as table
    // or might be parsed differently depending on GFM implementation
    // This test documents the behavior
    Ok(())
}

// --------------------------------------------------------------------------
// Horizontal Rules
// --------------------------------------------------------------------------

#[test]
fn parse_horizontal_rule() -> Result<()> {
    let input = r#"Before rule

---

After rule
"#;
    check(input, expected(input, vec![
        plain(input, "Before rule"), with_attr(node(input, "---\n", Element::ExternalContent, vec![]), "type", "horizontal-rule"),
        plain(input, "After rule"),
    ]))
}

#[test]
fn parse_horizontal_rule_variants() -> Result<()> {
    let input = r#"---

***

___
"#;
    check(input, expected(input, vec![
        with_attr(node(input, "---\n", Element::ExternalContent, vec![]), "type", "horizontal-rule"),
        with_attr(node(input, "***\n", Element::ExternalContent, vec![]), "type", "horizontal-rule"),
        with_attr(node(input, "___\n", Element::ExternalContent, vec![]), "type", "horizontal-rule"),
    ]))
}

// --------------------------------------------------------------------------
// HTML Passthrough
// --------------------------------------------------------------------------

#[test]
fn parse_inline_html() -> Result<()> {
    let input = "This has <em>HTML</em> inline.\n";
    check(input, expected(input, vec![paragraph(input, "This has <em>HTML</em> inline.", vec![
        text(input, "This has "), with_string_attr(node(input, "<em>", Element::TypedBlock { kind: BlockType::Passtrough }, vec![]), "content", "<em>"),
        text(input, "HTML"), with_string_attr(node(input, "</em>", Element::TypedBlock { kind: BlockType::Passtrough }, vec![]), "content", "</em>"),
        text(input, " inline."),
    ])]))
}

#[test]
fn parse_html_block() -> Result<()> {
    let input = r#"<div class="custom">
  <p>Raw HTML content</p>
</div>
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let html = ast.elements.iter()
        .find(|e| matches!(e.element, Element::TypedBlock { kind: BlockType::Passtrough }));

    assert!(html.is_some());

    Ok(())
}

// --------------------------------------------------------------------------
// Complex Documents
// --------------------------------------------------------------------------

#[test]
fn parse_complex_document() -> Result<()> {
    let input = r#"# Document Title

This is an introduction paragraph with **bold** and *italic* text.

## Section 1

Here's a [link](https://example.com) and some `inline code`.

```rust
fn main() {
    println!("Hello, world!");
}
```

## Section 2

A bullet list:

- Item 1
- Item 2
  - Nested item
- Item 3

And a numbered list:

1. First
2. Second
3. Third

> A blockquote with multiple lines.
> This is the second line.

---

Final paragraph.
"#;

    let intro = paragraph(input, "This is an introduction paragraph with **bold** and *italic* text.", vec![
        text(input, "This is an introduction paragraph with "), styled(input, "**bold**", "strong", vec![text(input, "bold")]),
        text(input, " and "), styled(input, "*italic*", "em", vec![text(input, "italic")]), text(input, " text."),
    ]);
    let section = paragraph(input, "Here's a [link](https://example.com) and some `inline code`.", vec![
        text(input, "Here's a "), with_string_attr(link(input, "[link](https://example.com)", "link", "https://example.com"), "protocol", "https"),
        text(input, " and some "), with_string_attr(styled(input, "`inline code`", "monospaced", vec![]), "content", "inline code"),
        at(input, input.find("`inline code`.").unwrap() + "`inline code`".len(), ".", Element::Text, vec![]),
    ]);
    let mut code = with_string_attr(node(input, "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```", Element::TypedBlock { kind: BlockType::Listing }, vec![]),
        "content", "fn main() {\n    println!(\"Hello, world!\");\n}\n");
    code.positional_attributes = vec![AttributeValue::String("source".into()), AttributeValue::String("rust".into())];
    check(input, expected(input, vec![
        node(input, "# Document Title\n", Element::Title { level: 1 }, vec![text(input, "Document Title")]), intro,
        node(input, "## Section 1\n", Element::Title { level: 2 }, vec![text(input, "Section 1")]), section, code,
        node(input, "## Section 2\n", Element::Title { level: 2 }, vec![text(input, "Section 2")]),
        plain(input, "A bullet list:"),
        node(input, "- Item 1\n- Item 2\n  - Nested item\n- Item 3\n\n", Element::List(ListType::Bullet), vec![
            list_item(input, "- Item 1\n", vec![text(input, "Item 1")]),
            list_item(input, "- Item 2\n  - Nested item\n", vec![text(input, "Item 2"),
                node(input, "  - Nested item\n", Element::List(ListType::Bullet), vec![list_item(input, "  - Nested item\n", vec![text(input, "Nested item")])])]),
            list_item(input, "- Item 3\n", vec![text(input, "Item 3")]),
        ]),
        plain(input, "And a numbered list:"),
        node(input, "1. First\n2. Second\n3. Third\n\n", Element::List(ListType::Number), vec![
            list_item(input, "1. First\n", vec![text(input, "First")]),
            list_item(input, "2. Second\n", vec![text(input, "Second")]),
            list_item(input, "3. Third\n", vec![text(input, "Third")]),
        ]),
        node(input, "> A blockquote with multiple lines.\n> This is the second line.\n", Element::TypedBlock { kind: BlockType::Quote }, vec![
            paragraph(input, "A blockquote with multiple lines.\n> This is the second line.", vec![
                text(input, "A blockquote with multiple lines."),
                at(input, input.find("A blockquote with multiple lines.\n").unwrap() + "A blockquote with multiple lines.".len(), "\n", Element::Text, vec![]),
                text(input, "This is the second line."),
            ]),
        ]),
        with_attr(node(input, "---\n", Element::ExternalContent, vec![]), "type", "horizontal-rule"),
        plain(input, "Final paragraph."),
    ]))
}

// --------------------------------------------------------------------------
// Edge Cases
// --------------------------------------------------------------------------

#[test]
fn parse_escaped_characters() -> Result<()> {
    let input = "This has \\*escaped\\* asterisks and \\[brackets\\].\n";
    check(input, expected(input, vec![paragraph(input, "This has \\*escaped\\* asterisks and \\[brackets\\].", vec![
        text(input, "This has "), text(input, "*escaped"),
        at(input, input.find("\\* asterisks").unwrap() + 1, "* asterisks and ", Element::Text, vec![]),
        text(input, "[brackets"), text(input, "]."),
    ])]))
}

#[test]
fn parse_reference_links() -> Result<()> {
    let input = r#"This is a [reference link][ref].

[ref]: https://example.com "Title"
"#;

    let linked = with_string_attr(with_string_attr(link(input, "[reference link][ref]", "reference link", "https://example.com"), "title", "Title"), "protocol", "https");
    check(input, expected(input, vec![paragraph(input, "This is a [reference link][ref].", vec![
        text(input, "This is a "), linked,
        at(input, input.find("[reference link][ref].").unwrap() + "[reference link][ref]".len(), ".", Element::Text, vec![]),
    ])]))
}

#[test]
fn parse_footnotes() -> Result<()> {
    let input = r#"This has a footnote[^1].

[^1]: This is the footnote text.
"#;
    // Footnote references are not represented by the reader; definitions use a generic tag.
    let definition = node(input, "[^1]: This is the footnote text.\n", Element::Text, vec![
        plain(input, "This is the footnote text."),
    ]);
    check(input, expected(input, vec![
        paragraph(input, "This has a footnote[^1].", vec![text(input, "This has a footnote"), text(input, ".")]),
        definition,
    ]))
}

#[test]
fn parse_empty_lines_in_lists() -> Result<()> {
    let input = r#"- Item 1

- Item 2

- Item 3
"#;
    check(input, expected(input, vec![node(input, input, Element::List(ListType::Bullet), vec![
        list_item(input, "- Item 1\n", vec![plain(input, "Item 1")]),
        list_item(input, "- Item 2\n", vec![plain(input, "Item 2")]),
        list_item(input, "- Item 3\n", vec![plain(input, "Item 3")]),
    ])]))
}

#[test]
fn parse_code_in_list() -> Result<()> {
    let input = r#"- Item with code:

  ```rust
  fn test() {}
  ```

- Another item
"#;

    let mut code = with_string_attr(node(input, "```rust\n  fn test() {}\n  ```", Element::TypedBlock { kind: BlockType::Listing }, vec![]), "content", "fn test() {}\n");
    code.positional_attributes = vec![AttributeValue::String("source".into()), AttributeValue::String("rust".into())];
    check(input, expected(input, vec![node(input, input, Element::List(ListType::Bullet), vec![
        list_item(input, "- Item with code:\n\n  ```rust\n  fn test() {}\n  ```\n", vec![plain(input, "Item with code:"), code]),
        list_item(input, "- Another item\n", vec![plain(input, "Another item")]),
    ])]))
}

// --------------------------------------------------------------------------
// Helper Functions
// --------------------------------------------------------------------------

// Construct expected trees from the fixture, never from the parser's output.
// Source ranges and line/column positions are part of the AST contract too.
fn at<'a>(input: &'a str, start: usize, source: &'a str, element: Element<'a>, children: Vec<ElementSpan<'a>>) -> ElementSpan<'a> {
    assert_eq!(&input[start..start + source.len()], source);
    let position = |offset: usize| {
        let prefix = &input[..offset];
        (prefix.bytes().filter(|&b| b == b'\n').count() + 1,
         prefix.rsplit('\n').next().unwrap().chars().count() + 1)
    };
    let (start_line, start_col) = position(start);
    let (end_line, end_col) = position(start + source.len());
    ElementSpan {
        source: None, content: source, element, start, end: start + source.len(),
        start_line, start_col, end_line, end_col, children,
        positional_attributes: vec![], attributes: vec![],
    }
}

fn node<'a>(input: &'a str, source: &'a str, element: Element<'a>, children: Vec<ElementSpan<'a>>) -> ElementSpan<'a> {
    let start = input.find(source).expect("expected source in fixture");
    // pulldown-cmark starts nested list spans at the marker, after indentation.
    if matches!(element, Element::List(_)) && source.starts_with("  ") {
        return at(input, start + 2, &source[2..], element, children);
    }
    at(input, start, source, element, children)
}

fn text<'a>(input: &'a str, source: &'a str) -> ElementSpan<'a> {
    node(input, source, Element::Text, vec![])
}

fn paragraph<'a>(input: &'a str, source: &'a str, children: Vec<ElementSpan<'a>>) -> ElementSpan<'a> {
    let start = input.find(source).expect("expected paragraph in fixture");
    let end = start + source.len();
    let end = if input[end..].starts_with('\n') { end + 1 } else { end };
    at(input, start, &input[start..end], Element::Paragraph, children)
}

fn plain<'a>(input: &'a str, source: &'a str) -> ElementSpan<'a> {
    paragraph(input, source, vec![text(input, source)])
}

fn attr<'a>(key: &str, value: &'a str) -> Attribute<'a> {
    Attribute { key: key.into(), value: AttributeValue::Ref(value) }
}

fn with_attr<'a>(mut span: ElementSpan<'a>, key: &str, value: &'a str) -> ElementSpan<'a> {
    span.attributes.push(attr(key, value));
    span
}

fn with_string_attr<'a>(mut span: ElementSpan<'a>, key: &str, value: &str) -> ElementSpan<'a> {
    span.attributes.push(Attribute { key: key.into(), value: AttributeValue::String(value.into()) });
    span
}

fn styled<'a>(input: &'a str, source: &'a str, style: &'a str, children: Vec<ElementSpan<'a>>) -> ElementSpan<'a> {
    with_attr(node(input, source, Element::Styled, children), "style", style)
}

fn link<'a>(input: &'a str, source: &'a str, label: &'a str, url: &'a str) -> ElementSpan<'a> {
    with_string_attr(node(input, source, Element::Link, vec![text(input, label)]), "url", url)
}

fn list_item<'a>(input: &'a str, source: &'a str, children: Vec<ElementSpan<'a>>) -> ElementSpan<'a> {
    let start = input.find(source).expect("expected list item in fixture");
    let end = start + source.len();
    let end = if input[end..].starts_with('\n') { end + 1 } else { end };
    let start = if source.starts_with("  ") { start + 2 } else { start };
    at(input, start, &input[start..end], Element::ListItem(1), children)
}

fn table_row<'a>(input: &'a str, source: &'a str, cells: &[&'a str]) -> ElementSpan<'a> {
    let row_start = input.find(source).expect("expected row");
    let mut next = row_start + 1;
    let mut children = Vec::new();
    for &cell_text in cells {
        let end = next + input[next..row_start + source.len()].find('|').expect("expected cell delimiter");
        let content = &input[next..end];
        let text_start = next + content.find(cell_text).expect("expected cell text");
        let cell = at(input, next, content, Element::TableCell,
            vec![at(input, text_start, cell_text, Element::Text, vec![])]);
        children.push(cell);
        next = end + 1;
    }
    at(input, row_start, source, Element::TableRow, children)
}

fn expected<'a>(input: &'a str, elements: Vec<ElementSpan<'a>>) -> AST<'a> {
    AST { content: input, elements, attributes: vec![] }
}

fn check(input: &str, expected: AST<'_>) -> Result<()> {
    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    assert_eq!(reader.parse(input, &opts, &mut env)?, expected);
    Ok(())
}

