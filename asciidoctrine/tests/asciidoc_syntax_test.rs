use anyhow::Result;
use asciidoctrine::reader::asciidoc::AsciidocReader;
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

  let reader = AsciidocReader::new();
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

  let reader = AsciidocReader::new();
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
  let ast = AST {
    content: "= test\n",
    elements: vec![ElementSpan {
      source: None,
      content: "= test",
      element: Element::Title { level: 1 },
      start: 0,
      end: 6,
      start_line: 1,
      start_col: 1,
      end_line: 1,
      end_col: 7,
      children: Vec::new(),
      positional_attributes: Vec::new(),
      attributes: vec![Attribute {
        key: "name".to_string(),
        value: AttributeValue::Ref("test"),
      }],
    }],
    attributes: Vec::new(),
  };

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  //assert_eq!(ast, asciidoc::parse_ast("= test")); // TODO
  let mut env = util::Env::Cache(util::Cache::new());
  assert_eq!(ast, reader.parse("= test\n", &opts, &mut env)?);

  // TODO author_info, attributes, etc
  Ok(())
}

#[test]
fn parse_title_with_anchor() -> Result<()> {
  let ast = AST {
    content: "[[test-anchor]]\n== test\n",
    elements: vec![ElementSpan {
      source: None,
      content: "[[test-anchor]]\n== test",
      element: Element::Title { level: 2 },
      start: 0,
      end: 23,
      start_line: 1,
      start_col: 1,
      end_line: 2,
      end_col: 8,
      children: Vec::new(),
      positional_attributes: Vec::new(),
      attributes: vec![
        Attribute {
          key: "anchor".to_string(),
          value: AttributeValue::Ref("test-anchor"),
        },
        Attribute {
          key: "name".to_string(),
          value: AttributeValue::Ref("test"),
        },
      ],
    }],
    attributes: Vec::new(),
  };

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  assert_eq!(
    ast,
    reader.parse("[[test-anchor]]\n== test\n", &opts, &mut env)?
  );
  Ok(())
}

#[test]
fn parse_atx_header() -> Result<()> {
  let ast = AST {
    content: "== test\n",
    elements: vec![ElementSpan {
      source: None,
      content: "== test",
      element: Element::Title { level: 2 },
      start: 0,
      end: 7,
      start_line: 1,
      start_col: 1,
      end_line: 1,
      end_col: 8,
      children: Vec::new(),
      positional_attributes: Vec::new(),
      attributes: vec![Attribute {
        key: "name".to_string(),
        value: AttributeValue::Ref("test"),
      }],
    }],
    attributes: Vec::new(),
  };

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  assert_eq!(ast, reader.parse("== test\n", &opts, &mut env)?);
  Ok(())
}

#[test]
fn parse_setext_header() -> Result<()> {
  let ast = AST {
    content: "test\n====\n",
    elements: vec![ElementSpan {
      source: None,
      content: "test\n====",
      element: Element::Title { level: 1 },
      start: 0,
      end: 9,
      start_line: 1,
      start_col: 1,
      end_line: 2,
      end_col: 5,
      children: Vec::new(),
      positional_attributes: Vec::new(),
      attributes: vec![Attribute {
        key: "name".to_string(),
        value: AttributeValue::Ref("test"),
      }],
    }],
    attributes: Vec::new(),
  };

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  assert_eq!(ast, reader.parse("test\n====\n", &opts, &mut env)?);

  // TODO Andere Titel
  Ok(())
}

// --------------------------------------------------------------------------
// Delimited blocks
// --------------------------------------------------------------------------

#[test]
fn parse_code() -> Result<()> {
  let input = r#"
[source, lua]
.this is a test snippet
----------------------------------------
require "mytestmodule"

----------------------------------------
"#;

  let code = r#"require "mytestmodule"
"#;

  let ast = AST {
    content: input,
    elements: vec![ElementSpan {
      source: None,
      content: input.trim(),
      element: Element::TypedBlock {
        kind: BlockType::Listing,
      },
      start: 1,
      end: 144,
      start_line: 2,
      start_col: 1,
      end_line: 7,
      end_col: 41,
      children: Vec::new(),
      positional_attributes: vec![AttributeValue::Ref("source"), AttributeValue::Ref("lua")],
      attributes: vec![
        Attribute {
          key: "title".to_string(),
          value: AttributeValue::Ref("this is a test snippet"),
        },
        Attribute {
          key: "content".to_string(),
          value: AttributeValue::Ref(code),
        },
      ],
    }],
    attributes: Vec::new(),
  };

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  assert_eq!(ast, reader.parse(input, &opts, &mut env)?);
  Ok(())
}

#[test]
fn parse_embedded_code_block() -> Result<()> {
  let input = r#"
[source, asciidoc]
.this is a test snippet
....
this is an embedded asciidoc snippet

[source, lua]
.this is a embedded snippet
----------------------------------------
require "mytestmodule"

----------------------------------------

asciidoctrine dont sees it.
....
"#;

  let code = r#"this is an embedded asciidoc snippet

[source, lua]
.this is a embedded snippet
----------------------------------------
require "mytestmodule"

----------------------------------------

asciidoctrine dont sees it."#;

  let ast = AST {
    content: input,
    elements: vec![ElementSpan {
      source: None,
      content: input.trim(),
      element: Element::TypedBlock {
        kind: BlockType::Listing,
      },
      start: 1,
      end: 268,
      start_line: 2,
      start_col: 1,
      end_line: 15,
      end_col: 5,
      children: Vec::new(),
      positional_attributes: vec![
        AttributeValue::Ref("source"),
        AttributeValue::Ref("asciidoc"),
      ],
      attributes: vec![
        Attribute {
          key: "title".to_string(),
          value: AttributeValue::Ref("this is a test snippet"),
        },
        Attribute {
          key: "content".to_string(),
          value: AttributeValue::Ref(code),
        },
      ],
    }],
    attributes: Vec::new(),
  };

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  assert_eq!(ast, reader.parse(input, &opts, &mut env)?);
  Ok(())
}

// --------------------------------------------------------------------------
// Paragraphs
// --------------------------------------------------------------------------

#[test]
fn parse_basic_paragraph_with_links_and_references() -> Result<()> {
  let input = r#"
This is a basic paragraph. It has a link to https://www.mytestsite.org[A test website] and
it has an internal <<reference>>. Both should be parsed.

"#;

  let ast = AST {
    content: input,
    elements: vec![ElementSpan {
      source: None,
      content: input.trim(),
      element: Element::Paragraph,
      start: 1,
      end: 148,
      start_line: 2,
      start_col: 1,
      end_line: 3,
      end_col: 57,
      children: vec![
        ElementSpan {
          source: None,
          content: "This is a basic paragraph. It has a link to ",
          element: Element::Text,
          start: 1,
          end: 45,
          start_line: 2,
          start_col: 1,
          end_line: 2,
          end_col: 45,
          children: Vec::new(),
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
        ElementSpan {
          source: None,
          content: "https://www.mytestsite.org[A test website]",
          element: Element::Link,
          start: 45,
          end: 87,
          start_line: 2,
          start_col: 45,
          end_line: 2,
          end_col: 87,
          children: Vec::new(),
          positional_attributes: vec![AttributeValue::Ref("A test website")],
          attributes: vec![
            Attribute {
              key: "url".to_string(),
              value: AttributeValue::Ref("https://www.mytestsite.org"),
            },
            Attribute {
              key: "protocol".to_string(),
              value: AttributeValue::Ref("https"),
            },
          ],
        },
        ElementSpan {
          source: None,
          content: " and\nit has an internal ",
          element: Element::Text,
          start: 87,
          end: 111,
          start_line: 2,
          start_col: 87,
          end_line: 3,
          end_col: 20,
          children: Vec::new(),
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
        ElementSpan {
          source: None,
          content: "<<reference>>",
          element: Element::XRef,
          start: 111,
          end: 124,
          start_line: 3,
          start_col: 20,
          end_line: 3,
          end_col: 33,
          children: Vec::new(),
          positional_attributes: Vec::new(),
          attributes: vec![Attribute {
            key: "id".to_string(),
            value: AttributeValue::Ref("reference"),
          }],
        },
        ElementSpan {
          source: None,
          content: ". Both should be parsed.",
          element: Element::Text,
          start: 124,
          end: 148,
          start_line: 3,
          start_col: 33,
          end_line: 3,
          end_col: 57,
          children: Vec::new(),
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
      ],
      positional_attributes: Vec::new(),
      attributes: Vec::new(),
    }],
    attributes: Vec::new(),
  };

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  assert_eq!(ast, reader.parse(input, &opts, &mut env)?);
  Ok(())
}

#[test]
fn parse_basic_paragraph_with_inline_anchor() -> Result<()> {
  let input = r#"
This is a basic paragraph. It has an inline [[myanchor]]`anchor`.

"#;

  let ast = AST {
    content: input,
    elements: vec![ElementSpan {
      source: None,
      content: input.trim(),
      element: Element::Paragraph,
      start: 1,
      end: 66,
      start_line: 2,
      start_col: 1,
      end_line: 2,
      end_col: 66,
      children: vec![
        ElementSpan {
          source: None,
          content: "This is a basic paragraph. It has an inline ",
          element: Element::Text,
          start: 1,
          end: 45,
          start_line: 2,
          start_col: 1,
          end_line: 2,
          end_col: 45,
          children: Vec::new(),
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
        ElementSpan {
          source: None,
          content: "[[myanchor]]`anchor`",
          element: Element::Styled,
          start: 45,
          end: 65,
          start_line: 2,
          start_col: 45,
          end_line: 2,
          end_col: 65,
          children: Vec::new(),
          positional_attributes: vec![],
          attributes: vec![
            Attribute {
              key: "style".to_string(),
              value: AttributeValue::Ref("monospaced"),
            },
            Attribute {
              key: "content".to_string(),
              value: AttributeValue::String("anchor".to_string()),
            },
            Attribute {
              key: "anchor".to_string(),
              value: AttributeValue::Ref("myanchor"),
            },
          ],
        },
        ElementSpan {
          source: None,
          content: ".",
          element: Element::Text,
          start: 65,
          end: 66,
          start_line: 2,
          start_col: 65,
          end_line: 2,
          end_col: 66,
          children: Vec::new(),
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
      ],
      positional_attributes: Vec::new(),
      attributes: Vec::new(),
    }],
    attributes: Vec::new(),
  };

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  assert_eq!(ast, reader.parse(input, &opts, &mut env)?);
  Ok(())
}

// TODO Link, References
// TODO Links mit Leerzeichen in der Attribut Liste (Würde ich das auch in anderen Attribut Listen zulassen?)
// TODO bold, italian, auch ob es in einem Wort ignoriert wird
// Attributliste oder Anchor zu Beginn eines Paragraphs

// --------------------------------------------------------------------------
// Lists
// --------------------------------------------------------------------------

#[test]
fn parse_bullet_list() -> Result<()> {
  let input = r#"
* Item 1
* Item 2
* Item 3
"#;

  let ast = AST {
    content: input,
    elements: vec![ElementSpan {
      source: None,
      content: input.trim_start(),
      element: Element::List(ListType::Bullet),
      start: 1,
      end: 28,
      start_line: 2,
      start_col: 1,
      end_line: 5,
      end_col: 1,
      children: vec![
        ElementSpan {
          source: None,
          content: "* Item 1\n",
          element: Element::ListItem(1),
          start: 1,
          end: 10,
          start_line: 2,
          start_col: 1,
          end_line: 3,
          end_col: 1,
          children: vec![ElementSpan {
            source: None,
            content: "Item 1",
            start: 3,
            end: 9,
            start_line: 2,
            start_col: 3,
            end_line: 2,
            end_col: 9,
            element: Element::Paragraph,
            children: vec![ElementSpan {
              source: None,
              content: "Item 1",
              start: 3,
              end: 9,
              start_line: 2,
              start_col: 3,
              end_line: 2,
              end_col: 9,
              element: Element::Text,
              children: vec![],
              positional_attributes: vec![],
              attributes: vec![],
            }],
            positional_attributes: vec![],
            attributes: vec![],
          }],
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
        ElementSpan {
          source: None,
          content: "* Item 2\n",
          element: Element::ListItem(1),
          start: 10,
          end: 19,
          start_line: 3,
          start_col: 1,
          end_line: 4,
          end_col: 1,
          children: vec![ElementSpan {
            source: None,
            content: "Item 2",
            start: 12,
            end: 18,
            start_line: 3,
            start_col: 3,
            end_line: 3,
            end_col: 9,
            element: Element::Paragraph,
            children: vec![ElementSpan {
              source: None,
              content: "Item 2",
              start: 12,
              end: 18,
              start_line: 3,
              start_col: 3,
              end_line: 3,
              end_col: 9,
              element: Element::Text,
              children: vec![],
              positional_attributes: vec![],
              attributes: vec![],
            }],
            positional_attributes: vec![],
            attributes: vec![],
          }],
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
        ElementSpan {
          source: None,
          content: "* Item 3\n",
          element: Element::ListItem(1),
          start: 19,
          end: 28,
          start_line: 4,
          start_col: 1,
          end_line: 5,
          end_col: 1,
          children: vec![ElementSpan {
            source: None,
            content: "Item 3",
            start: 21,
            end: 27,
            start_line: 4,
            start_col: 3,
            end_line: 4,
            end_col: 9,
            element: Element::Paragraph,
            children: vec![ElementSpan {
              source: None,
              content: "Item 3",
              start: 21,
              end: 27,
              start_line: 4,
              start_col: 3,
              end_line: 4,
              end_col: 9,
              element: Element::Text,
              children: vec![],
              positional_attributes: vec![],
              attributes: vec![],
            }],
            positional_attributes: vec![],
            attributes: vec![],
          }],
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
      ],
      positional_attributes: Vec::new(),
      attributes: Vec::new(),
    }],
    attributes: Vec::new(),
  };

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  assert_eq!(ast, reader.parse(input, &opts, &mut env)?);
  Ok(())
}

#[test]
fn parse_nested_bullet_list() -> Result<()> {
  let input = r#"
* Item 1
*** Item 2
** Item 3
"#;

  let ast = AST {
    content: input,
    elements: vec![ElementSpan {
      source: None,
      content: input.trim_start(),
      element: Element::List(ListType::Bullet),
      start: 1,
      end: 31,
      start_line: 2,
      start_col: 1,
      end_line: 5,
      end_col: 1,
      children: vec![
        ElementSpan {
          source: None,
          content: "* Item 1\n",
          element: Element::ListItem(1),
          start: 1,
          end: 10,
          start_line: 2,
          start_col: 1,
          end_line: 3,
          end_col: 1,
          children: vec![ElementSpan {
            source: None,
            content: "Item 1",
            start: 3,
            end: 9,
            start_line: 2,
            start_col: 3,
            end_line: 2,
            end_col: 9,
            element: Element::Paragraph,
            children: vec![ElementSpan {
              source: None,
              content: "Item 1",
              start: 3,
              end: 9,
              start_line: 2,
              start_col: 3,
              end_line: 2,
              end_col: 9,
              element: Element::Text,
              children: vec![],
              positional_attributes: vec![],
              attributes: vec![],
            }],
            positional_attributes: vec![],
            attributes: vec![],
          }],
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
        ElementSpan {
          source: None,
          content: "*** Item 2\n",
          element: Element::ListItem(3),
          start: 10,
          end: 21,
          start_line: 3,
          start_col: 1,
          end_line: 4,
          end_col: 1,
          children: vec![ElementSpan {
            source: None,
            content: "Item 2",
            start: 14,
            end: 20,
            start_line: 3,
            start_col: 5,
            end_line: 3,
            end_col: 11,
            element: Element::Paragraph,
            children: vec![ElementSpan {
              source: None,
              content: "Item 2",
              start: 14,
              end: 20,
              start_line: 3,
              start_col: 5,
              end_line: 3,
              end_col: 11,
              element: Element::Text,
              children: vec![],
              positional_attributes: vec![],
              attributes: vec![],
            }],
            positional_attributes: vec![],
            attributes: vec![],
          }],
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
        ElementSpan {
          source: None,
          content: "** Item 3\n",
          element: Element::ListItem(2),
          start: 21,
          end: 31,
          start_line: 4,
          start_col: 1,
          end_line: 5,
          end_col: 1,
          children: vec![ElementSpan {
            source: None,
            content: "Item 3",
            start: 24,
            end: 30,
            start_line: 4,
            start_col: 4,
            end_line: 4,
            end_col: 10,
            element: Element::Paragraph,
            children: vec![ElementSpan {
              source: None,
              content: "Item 3",
              start: 24,
              end: 30,
              start_line: 4,
              start_col: 4,
              end_line: 4,
              end_col: 10,
              element: Element::Text,
              children: vec![],
              positional_attributes: vec![],
              attributes: vec![],
            }],
            positional_attributes: vec![],
            attributes: vec![],
          }],
          positional_attributes: Vec::new(),
          attributes: Vec::new(),
        },
      ],
      positional_attributes: Vec::new(),
      attributes: Vec::new(),
    }],
    attributes: Vec::new(),
  };

  let reader = AsciidocReader::new();
  let opts = options::Opts::parse_from(vec![""].into_iter());
  let mut env = util::Env::Cache(util::Cache::new());
  assert_eq!(ast, reader.parse(input, &opts, &mut env)?);
  Ok(())
}

// Bullet Lists, Numbered Lists, Attribute Lists, Checked Lists
// List Continuation, Blocks in Lists

// TODO Tables

// TODO Include
// with anchor
