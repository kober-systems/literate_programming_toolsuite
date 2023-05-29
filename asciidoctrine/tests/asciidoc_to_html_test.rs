use anyhow::Result;
use asciidoctrine::{self, *};
use clap::Parser;
use pretty_assertions::assert_eq;
use std::io::BufWriter;

#[test]
fn collapsible_blocks() -> Result<()> {
  let content = r#"
[%collapsible]
====
Additional Information, that will only be shown on demand.
====
"#;
  let reader = AsciidocReader::new();
  let mut opts = options::Opts::parse_from(vec!["--template", "-"].into_iter());
  opts.template = Some("-".into());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<details>
  <summary class="title">Details</summary>
  <div class="content">
    <div class="paragraph">
      <p>Additional Information, that will only be shown on demand.</p>
    </div>
  </div>
</details>
"#
  );

  Ok(())
}

#[test]
fn collapsible_blocks_open() -> Result<()> {
  let content = r#"
[%collapsible%open]
====
This Information is visible by default.
====
"#;
  let reader = AsciidocReader::new();
  let mut opts = options::Opts::parse_from(vec!["--template", "-"].into_iter());
  opts.template = Some("-".into());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<details open>
  <summary class="title">Details</summary>
  <div class="content">
    <div class="paragraph">
      <p>This Information is visible by default.</p>
    </div>
  </div>
</details>
"#
  );

  Ok(())
}

#[test]
fn atx_headers() -> Result<()> {
  let content = r#"
= This is a header

== This is a subheader

=== This is a subsubheader

==== This is a subsubsubheader
"#;
  let reader = AsciidocReader::new();
  let mut opts = options::Opts::parse_from(vec!["--template", "-"].into_iter());
  opts.template = Some("-".into());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<h1>This is a header</h1>
<h2 id="_this_is_a_subheader">This is a subheader</h2>
<h3 id="_this_is_a_subsubheader">This is a subsubheader</h3>
<h4 id="_this_is_a_subsubsubheader">This is a subsubsubheader</h4>
"#
  );

  Ok(())
}

#[test]
fn setext_headers() -> Result<()> {
  let content = r#"
This is a header
================

This is a subheader
-------------------

This is a subsubheader
~~~~~~~~~~~~~~~~~~~~~~

This is a subsubsubheader
^^^^^^^^^^^^^^^^^^^^^^^^^
"#;
  let reader = AsciidocReader::new();
  let mut opts = options::Opts::parse_from(vec!["--template", "-"].into_iter());
  opts.template = Some("-".into());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<h1>This is a header</h1>
<h2 id="_this_is_a_subheader">This is a subheader</h2>
<h3 id="_this_is_a_subsubheader">This is a subsubheader</h3>
<h4 id="_this_is_a_subsubsubheader">This is a subsubsubheader</h4>
"#
  );

  Ok(())
}

#[test]
fn inline_bold() -> Result<()> {
  let content = r#"
Some text is *bold*.
"#;
  let reader = AsciidocReader::new();
  let mut opts = options::Opts::parse_from(vec!["--template", "-"].into_iter());
  opts.template = Some("-".into());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<p>Some text is <strong>bold</strong>.</p>
"#
  );

  Ok(())
}

#[test]
fn inline_italic() -> Result<()> {
  let content = r#"
Some text is _italic_.
"#;
  let reader = AsciidocReader::new();
  let mut opts = options::Opts::parse_from(vec!["--template", "-"].into_iter());
  opts.template = Some("-".into());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<p>Some text is <em>italic</em>.</p>
"#
  );

  Ok(())
}

#[test]
fn inline_monospaced() -> Result<()> {
  let content = r#"
Some text is `monospaced`.
"#;
  let reader = AsciidocReader::new();
  let mut opts = options::Opts::parse_from(vec!["--template", "-"].into_iter());
  opts.template = Some("-".into());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<p>Some text is <code>monospaced</code>.</p>
"#
  );

  Ok(())
}

#[test]
fn sourcecode_blocks() -> Result<()> {
  let content = r#"
[source, bash]
----
echo "hello world!"
----
[source, bash]
....
echo "hello world!"
....
"#;
  let reader = AsciidocReader::new();
  let mut opts = options::Opts::parse_from(vec!["--template", "-"].into_iter());
  opts.template = Some("-".into());
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<div class="listingblock">
  <pre>echo "hello world!"</pre>
</div>
<div class="listingblock">
  <pre>echo "hello world!"</pre>
</div>
"#
  );

  Ok(())
}

