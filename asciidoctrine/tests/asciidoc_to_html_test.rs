use anyhow::Result;
use asciidoctrine::{self, *};
use clap::Parser;
use pretty_assertions::assert_eq;
use std::io::BufWriter;

#[test]
fn bullet_list_with_dashes() -> Result<()> {
  let content = r#"
- This
- is
- a
- list
-- with subpoints
--- and deeper
---- nested subpoints
- Next normal point
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<ul>
  <li>
    <p>This</p>
  </li>
  <li>
    <p>is</p>
  </li>
  <li>
    <p>a</p>
  </li>
  <li>
    <p>list</p>
    <ul>
      <li>
        <p>with subpoints</p>
        <ul>
          <li>
            <p>and deeper</p>
            <ul>
              <li>
                <p>nested subpoints</p>
              </li>
            </ul>
          </li>
        </ul>
      </li>
    </ul>
  </li>
  <li>
    <p>Next normal point</p>
  </li>
</ul>
"#
  );

  Ok(())
}

#[test]
fn bullet_list() -> Result<()> {
  let content = r#"
* This
* is
* a
* list
** with subpoints
*** and deeper
**** nested subpoints
* Next normal point
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<ul>
  <li>
    <p>This</p>
  </li>
  <li>
    <p>is</p>
  </li>
  <li>
    <p>a</p>
  </li>
  <li>
    <p>list</p>
    <ul>
      <li>
        <p>with subpoints</p>
        <ul>
          <li>
            <p>and deeper</p>
            <ul>
              <li>
                <p>nested subpoints</p>
              </li>
            </ul>
          </li>
        </ul>
      </li>
    </ul>
  </li>
  <li>
    <p>Next normal point</p>
  </li>
</ul>
"#
  );

  Ok(())
}

#[test]
fn collapsible_blocks() -> Result<()> {
  let content = r#"
[%collapsible]
====
Additional Information, that will only be shown on demand.
====
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
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
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
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
fn formated_table() -> Result<()> {
  let content = r#"
[cols="1,a"]
|===
|
Here inline *markup* _text_ is rendered specially

But paragraphs

* or
** Lists
** are not handled as such
|
In this cell *markup* _text_ is handeled specially

We can even have multiple paragraphs

* List
** with
** multiple
* entries
|===
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<table class="tableblock frame-all grid-all stretch">
  <colgroup>
    <col style="width: 50%;">
    <col style="width: 50%;">
  </colgroup>
  <tbody>
    <tr>
      <td><p>Here inline <strong>markup</strong> <em>text</em> is rendered specially

But paragraphs

* or
** Lists
** are not handled as such</p></td>
      <td>
        <p>In this cell <strong>markup</strong> <em>text</em> is handeled specially</p>
        <p>We can even have multiple paragraphs</p>
        <ul>
          <li>
            <p>List</p>
            <ul>
              <li>
                <p>with</p>
              </li>
              <li>
                <p>multiple</p>
              </li>
            </ul>
          </li>
          <li>
            <p>entries</p>
          </li>
        </ul>
      </td>
    </tr>
  </tbody>
</table>
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
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
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
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
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
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
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
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
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
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
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
fn numbered_list() -> Result<()> {
  let content = r#"
. This
. is
. a
.. nested
. numbered list
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<ol class="arabic">
  <li>
    <p>This</p>
  </li>
  <li>
    <p>is</p>
  </li>
  <li>
    <p>a</p>
    <ol class="loweralpha" type="a">
      <li>
        <p>nested</p>
      </li>
    </ol>
  </li>
  <li>
    <p>numbered list</p>
  </li>
</ol>
"#
  );

  Ok(())
}

#[test]
fn simple_table() -> Result<()> {
  let content = r#"
|===
| Col1 | Col2
| Cel1 | Cel2
| Cel3 | Cel4
|===
"#;
  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
  let mut env = util::Env::Cache(util::Cache::new());
  let ast = reader.parse(content, &opts, &mut env)?;

  let mut buf = BufWriter::new(Vec::new());
  let mut writer = HtmlWriter::new();
  writer.write(ast, &opts, &mut buf)?;

  let output = String::from_utf8(buf.into_inner()?)?;
  assert_eq!(
    output,
    r#"<table class="tableblock frame-all grid-all stretch">
  <colgroup>
    <col style="width: 50%;">
    <col style="width: 50%;">
  </colgroup>
  <tbody>
    <tr>
      <td><p>Col1</p></td>
      <td><p>Col2</p></td>
    </tr>
    <tr>
      <td><p>Cel1</p></td>
      <td><p>Cel2</p></td>
    </tr>
    <tr>
      <td><p>Cel3</p></td>
      <td><p>Cel4</p></td>
    </tr>
  </tbody>
</table>
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
  let opts = options::Opts::parse_from(vec!["asciidoctrine", "--template", "-"]);
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

