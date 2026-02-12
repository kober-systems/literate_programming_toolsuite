use anyhow::Result;
use asciidoctrine::reader::markdown::MarkdownReader;
use asciidoctrine::{self, *};
use clap::Parser;
use pretty_assertions::assert_eq;

#[test]
fn parse_empty_document() -> Result<()> {
    let ast = AST {
        content: "",
        elements: Vec::new(),
        attributes: Vec::new(),
    };

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    assert_eq!(ast, reader.parse("", &opts, &mut env)?);
    Ok(())
}

#[test]
fn parse_whitespace_only() -> Result<()> {
    let ast = AST {
        content: "  ",
        elements: Vec::new(),
        attributes: Vec::new(),
    };

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    assert_eq!(ast, reader.parse("  ", &opts, &mut env)?);
    Ok(())
}

// --------------------------------------------------------------------------
// Headers
// --------------------------------------------------------------------------

#[test]
fn parse_basic_header() -> Result<()> {
    let input = "# test\n";

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::Title { level: 1 });
    assert_eq!(ast.elements[0].children.len(), 1);
    assert_eq!(ast.elements[0].children[0].element, Element::Text);
    assert_eq!(ast.elements[0].children[0].content, "test");

    Ok(())
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

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 6);
    for (i, expected_level) in [1, 2, 3, 4, 5, 6].iter().enumerate() {
        assert_eq!(
            ast.elements[i].element,
            Element::Title { level: *expected_level }
        );
    }

    Ok(())
}

#[test]
fn parse_setext_headers() -> Result<()> {
    let input = r#"Level 1
=======

Level 2
-------
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 2);
    assert_eq!(ast.elements[0].element, Element::Title { level: 1 });
    assert_eq!(ast.elements[1].element, Element::Title { level: 2 });

    Ok(())
}

// --------------------------------------------------------------------------
// Paragraphs
// --------------------------------------------------------------------------

#[test]
fn parse_basic_paragraph() -> Result<()> {
    let input = "This is a basic paragraph.\n";

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::Paragraph);
    assert_eq!(ast.elements[0].children.len(), 1);
    assert_eq!(ast.elements[0].children[0].element, Element::Text);

    Ok(())
}

#[test]
fn parse_multiple_paragraphs() -> Result<()> {
    let input = r#"First paragraph.

Second paragraph.

Third paragraph.
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 3);
    for elem in &ast.elements {
        assert_eq!(elem.element, Element::Paragraph);
    }

    Ok(())
}

// --------------------------------------------------------------------------
// Links
// --------------------------------------------------------------------------

#[test]
fn parse_inline_link() -> Result<()> {
    let input = "This has a [link](https://example.com) in it.\n";

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::Paragraph);

    // Find the link element in children
    let link = ast.elements[0]
        .children
        .iter()
        .find(|c| matches!(c.element, Element::Link));

    assert!(link.is_some());
    let link = link.unwrap();
    assert_eq!(link.get_attribute("url"), Some("https://example.com"));
    assert_eq!(link.get_attribute("protocol"), Some("https"));

    Ok(())
}

#[test]
fn parse_link_with_title() -> Result<()> {
    let input = r#"[Link](https://example.com "Title text")"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let link = &ast.elements[0].children.iter()
        .find(|c| matches!(c.element, Element::Link));

    assert!(link.is_some());
    let link = link.unwrap();
    assert_eq!(link.get_attribute("title"), Some("Title text"));

    Ok(())
}

#[test]
fn parse_autolink() -> Result<()> {
    let input = "<https://example.com>\n";

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let link = ast.elements[0].children.iter()
        .find(|c| matches!(c.element, Element::Link));

    assert!(link.is_some());
    let link = link.unwrap();
    assert_eq!(link.get_attribute("url"), Some("https://example.com"));

    Ok(())
}

// --------------------------------------------------------------------------
// Images
// --------------------------------------------------------------------------

#[test]
fn parse_image() -> Result<()> {
    let input = "![Alt text](image.png)\n";

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let img = ast.elements[0].children.iter()
        .find(|c| matches!(c.element, Element::Image));

    assert!(img.is_some());
    let img = img.unwrap();
    assert_eq!(img.get_attribute("path"), Some("image.png"));

    Ok(())
}

#[test]
fn parse_image_with_title() -> Result<()> {
    let input = r#"![Alt text](image.png "Image title")"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let img = ast.elements[0].children.iter()
        .find(|c| matches!(c.element, Element::Image));

    assert!(img.is_some());
    let img = img.unwrap();
    assert_eq!(img.positional_attributes.len(), 1);
    assert_eq!(img.positional_attributes[0].as_str(), "Image title");

    Ok(())
}

// --------------------------------------------------------------------------
// Inline Formatting
// --------------------------------------------------------------------------

#[test]
fn parse_bold() -> Result<()> {
    let input = "This is **bold** text.\n";

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let strong = ast.elements[0].children.iter()
        .find(|c| matches!(c.element, Element::Styled) && c.get_attribute("style") == Some("strong"));

    assert!(strong.is_some());

    Ok(())
}

#[test]
fn parse_italic() -> Result<()> {
    let input = "This is *italic* text.\n";

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let em = ast.elements[0].children.iter()
        .find(|c| matches!(c.element, Element::Styled) && c.get_attribute("style") == Some("em"));

    assert!(em.is_some());

    Ok(())
}

#[test]
fn parse_inline_code() -> Result<()> {
    let input = "This has `inline code` in it.\n";

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let code = ast.elements[0].children.iter()
        .find(|c| matches!(c.element, Element::Styled) && c.get_attribute("style") == Some("monospaced"));

    assert!(code.is_some());
    let code = code.unwrap();
    assert_eq!(code.get_attribute("content"), Some("inline code"));

    Ok(())
}

#[test]
fn parse_strikethrough() -> Result<()> {
    let input = "This is ~~strikethrough~~ text.\n";

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let strike = ast.elements[0].children.iter()
        .find(|c| matches!(c.element, Element::Styled) && c.get_attribute("style") == Some("strikethrough"));

    assert!(strike.is_some());

    Ok(())
}

#[test]
fn parse_combined_formatting() -> Result<()> {
    let input = "This has **bold** and *italic* and `code` together.\n";

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements[0].element, Element::Paragraph);

    let has_bold = ast.elements[0].children.iter()
        .any(|c| matches!(c.element, Element::Styled) && c.get_attribute("style") == Some("strong"));
    let has_italic = ast.elements[0].children.iter()
        .any(|c| matches!(c.element, Element::Styled) && c.get_attribute("style") == Some("em"));
    let has_code = ast.elements[0].children.iter()
        .any(|c| matches!(c.element, Element::Styled) && c.get_attribute("style") == Some("monospaced"));

    assert!(has_bold);
    assert!(has_italic);
    assert!(has_code);

    Ok(())
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

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(
        ast.elements[0].element,
        Element::TypedBlock {
            kind: BlockType::Listing
        }
    );
    assert_eq!(ast.elements[0].get_attribute("content"), Some("code here\nmore code\n"));

    Ok(())
}

#[test]
fn parse_code_block_with_language() -> Result<()> {
    let input = r#"```rust
fn main() {
    println!("Hello");
}
```
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(
        ast.elements[0].element,
        Element::TypedBlock {
            kind: BlockType::Listing
        }
    );
    assert_eq!(ast.elements[0].positional_attributes.len(), 2);
    assert_eq!(ast.elements[0].positional_attributes[0].as_str(), "source");
    assert_eq!(ast.elements[0].positional_attributes[1].as_str(), "rust");

    Ok(())
}

#[test]
fn parse_indented_code_block() -> Result<()> {
    let input = r#"Normal paragraph.

    indented code
    more code

Back to normal.
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let code_block = ast.elements.iter()
        .find(|e| matches!(e.element, Element::TypedBlock { kind: BlockType::Listing }));

    assert!(code_block.is_some());

    Ok(())
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

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::List(ListType::Bullet));
    assert_eq!(ast.elements[0].children.len(), 3);

    for child in &ast.elements[0].children {
        assert!(matches!(child.element, Element::ListItem(_)));
    }

    Ok(())
}

#[test]
fn parse_numbered_list() -> Result<()> {
    let input = r#"1. First item
2. Second item
3. Third item
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::List(ListType::Number));
    assert_eq!(ast.elements[0].children.len(), 3);

    Ok(())
}

#[test]
fn parse_nested_list() -> Result<()> {
    let input = r#"- Item 1
  - Nested 1.1
  - Nested 1.2
- Item 2
  - Nested 2.1
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::List(ListType::Bullet));

    // First item should have a nested list
    let first_item = &ast.elements[0].children[0];
    let has_nested_list = first_item.children.iter()
        .any(|c| matches!(c.element, Element::List(ListType::Bullet)));

    assert!(has_nested_list);

    Ok(())
}

#[test]
fn parse_mixed_list() -> Result<()> {
    let input = r#"- Bullet item
  1. Numbered sub-item
  2. Another numbered
- Another bullet
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::List(ListType::Bullet));

    Ok(())
}

#[test]
fn parse_task_list() -> Result<()> {
    let input = r#"- [x] Completed task
- [ ] Incomplete task
- [x] Another completed
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::List(ListType::Bullet));

    // Check first item is checked
    let first_item = &ast.elements[0].children[0];
    assert_eq!(first_item.get_attribute("checked"), Some("true"));

    // Check second item is unchecked
    let second_item = &ast.elements[0].children[1];
    assert_eq!(second_item.get_attribute("checked"), Some("false"));

    Ok(())
}

// --------------------------------------------------------------------------
// Blockquotes
// --------------------------------------------------------------------------

#[test]
fn parse_blockquote() -> Result<()> {
    let input = r#"> This is a quote.
> It spans multiple lines.
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

    Ok(())
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

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let rule = ast.elements.iter()
        .find(|e| matches!(e.element, Element::ExternalContent));

    assert!(rule.is_some());
    let rule = rule.unwrap();
    assert_eq!(rule.get_attribute("type"), Some("horizontal-rule"));

    Ok(())
}

#[test]
fn parse_horizontal_rule_variants() -> Result<()> {
    let input = r#"---

***

___
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let rules: Vec<_> = ast.elements.iter()
        .filter(|e| matches!(e.element, Element::ExternalContent))
        .collect();

    assert_eq!(rules.len(), 3);

    Ok(())
}

// --------------------------------------------------------------------------
// HTML Passthrough
// --------------------------------------------------------------------------

#[test]
fn parse_inline_html() -> Result<()> {
    let input = r#"This has <em>HTML</em> inline.
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    let html = ast.elements[0].children.iter()
        .find(|c| matches!(c.element, Element::TypedBlock { kind: BlockType::Passtrough }));

    assert!(html.is_some());

    Ok(())
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

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    // Document should have multiple elements
    assert!(ast.elements.len() > 10);

    // Should have titles
    let titles: Vec<_> = ast.elements.iter()
        .filter(|e| matches!(e.element, Element::Title { .. }))
        .collect();
    assert_eq!(titles.len(), 3);

    // Should have code block
    let code_blocks: Vec<_> = ast.elements.iter()
        .filter(|e| matches!(e.element, Element::TypedBlock { kind: BlockType::Listing }))
        .collect();
    assert!(code_blocks.len() >= 1);

    // Should have lists
    let lists: Vec<_> = ast.elements.iter()
        .filter(|e| matches!(e.element, Element::List(_)))
        .collect();
    assert!(lists.len() >= 2);

    // Should have blockquote
    let quotes: Vec<_> = ast.elements.iter()
        .filter(|e| matches!(e.element, Element::TypedBlock { kind: BlockType::Quote }))
        .collect();
    assert_eq!(quotes.len(), 1);

    Ok(())
}

// --------------------------------------------------------------------------
// Edge Cases
// --------------------------------------------------------------------------

#[test]
fn parse_escaped_characters() -> Result<()> {
    let input = r#"This has \*escaped\* asterisks and \[brackets\].
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    // Should parse without treating escaped chars as formatting
    assert_eq!(ast.elements.len(), 1);

    Ok(())
}

#[test]
fn parse_reference_links() -> Result<()> {
    let input = r#"This is a [reference link][ref].

[ref]: https://example.com "Title"
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    // Should resolve reference link
    let link = ast.elements[0].children.iter()
        .find(|c| matches!(c.element, Element::Link));

    assert!(link.is_some());
    let link = link.unwrap();
    assert_eq!(link.get_attribute("url"), Some("https://example.com"));

    Ok(())
}

#[test]
fn parse_footnotes() -> Result<()> {
    let input = r#"This has a footnote[^1].

[^1]: This is the footnote text.
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    // Footnotes are supported by pulldown-cmark with ENABLE_FOOTNOTES
    // The exact AST structure depends on implementation details
    assert!(ast.elements.len() >= 1);

    Ok(())
}

#[test]
fn parse_empty_lines_in_lists() -> Result<()> {
    let input = r#"- Item 1

- Item 2

- Item 3
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    // Empty lines create loose list (items wrapped in paragraphs)
    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::List(ListType::Bullet));

    Ok(())
}

#[test]
fn parse_code_in_list() -> Result<()> {
    let input = r#"- Item with code:

  ```rust
  fn test() {}
  ```

- Another item
"#;

    let reader = MarkdownReader::new();
    let opts = options::Opts::parse_from(vec![""].into_iter());
    let mut env = util::Env::Cache(util::Cache::new());
    let ast = reader.parse(input, &opts, &mut env)?;

    assert_eq!(ast.elements.len(), 1);
    assert_eq!(ast.elements[0].element, Element::List(ListType::Bullet));

    Ok(())
}
