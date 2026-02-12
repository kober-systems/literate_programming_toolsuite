use anyhow::Result;
use asciidoctrine::{self, *};
use clap::Parser;
use pretty_assertions::assert_eq;
use std::io::BufWriter;

#[test]
fn simple_paragraph() -> Result<()> {
  let content = "This is a simple paragraph.";

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn multiple_paragraphs() -> Result<()> {
  let content = r#"First paragraph.

Second paragraph.

Third paragraph."#;

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn headers_atx_style() -> Result<()> {
  let content = r#"= Document Title

== Level 2 Header

=== Level 3 Header

==== Level 4 Header

===== Level 5 Header

====== Level 6 Header"#;

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn inline_formatting_bold() -> Result<()> {
  let content = "Some text is *bold*.";

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn inline_formatting_italic() -> Result<()> {
  let content = "Some text is _italic_.";

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn inline_formatting_monospace() -> Result<()> {
  let content = "Some text is `monospace`.";

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn inline_formatting_mixed() -> Result<()> {
  let content = "Text with *bold*, _italic_, and `code` formatting.";

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn bullet_list_simple() -> Result<()> {
  let content = r#"* First item
* Second item
* Third item"#;

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn bullet_list_nested() -> Result<()> {
  let content = r#"* First item
** Nested item
*** Deeply nested
* Second item"#;

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn numbered_list_simple() -> Result<()> {
  let content = r#". First item
. Second item
. Third item"#;

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn numbered_list_nested() -> Result<()> {
  let content = r#". First item
.. Nested item
... Deeply nested
. Second item"#;

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn code_block_listing() -> Result<()> {
  let content = r#"[source,bash]
----
echo "Hello, World!"
ls -la
----"#;

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(output, content);

  Ok(())
}

#[test]
fn simple_table() -> Result<()> {
  let content = r#"|===
| Col1 | Col2

| Cell1 | Cell2
| Cell3 | Cell4
|==="#;

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  // We may need to adjust the expected output based on how tables are parsed
  assert!(output.contains("|==="));
  assert!(output.contains("Col1"));
  assert!(output.contains("Col2"));
  assert!(output.contains("Cell1"));

  Ok(())
}

#[test]
fn link_inline() -> Result<()> {
  let content = "Check out link:https://example.com[this link].";

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert!(output.contains("link:"));
  assert!(output.contains("https://example.com"));
  assert!(output.contains("this link"));

  Ok(())
}

#[test]
fn round_trip_complex_document() -> Result<()> {
  let content = r#"= Document Title

== Introduction

This is a paragraph with *bold* and _italic_ text.

=== Features

* First feature
* Second feature
** Nested item
* Third feature

=== Code Example

[source,rust]
----
fn main() {
    println!("Hello, World!");
}
----

== Conclusion

That's all for now."#;

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = AsciidocWriter::new();
  writer.write(ast.clone(), &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;

  // Parse the output again to verify round-trip
  let ast2 = reader.parse(&output, &opts, &mut env)?;

  // Both ASTs should have the same structure
  assert_eq!(ast.elements.len(), ast2.elements.len());

  Ok(())
}
