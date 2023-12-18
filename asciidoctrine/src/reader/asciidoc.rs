pub use crate::ast::*;
use crate::options::Opts;
use crate::util::{Env, Environment};
use crate::reader::*;
use crate::Result;
use pest::iterators::Pair;
use pest::Parser;

pub struct AsciidocReader {}

impl AsciidocReader {
  pub fn new() -> Self {
    AsciidocReader {}
  }
}

impl crate::Reader for AsciidocReader {
  fn parse<'a>(&self, input: &'a str, args: &Opts, env: &mut Env) -> Result<AST<'a>> {
    let ast = AsciidocParser::parse(Rule::asciidoc, input)?;

    let mut attributes = Vec::new();
    if let Some(path) = &args.input {
      if let Some(path) = path.to_str() {
        attributes.push(Attribute {
          key: "source".to_string(),
          value: AttributeValue::String(path.to_string()),
        });
      }
    }

    let mut elements = Vec::new();

    for element in ast {
      if let Some(element) = process_element(element, env) {
        elements.push(element);
      }
    }

    Ok(AST {
      content: input,
      elements,
      attributes,
    })
  }
}

#[derive(Parser, Debug, Copy, Clone)]
#[grammar = "reader/asciidoc.pest"]
pub struct AsciidocParser;

fn process_element<'a>(
  element: Pair<'a, asciidoc::Rule>,
  env: &mut Env,
) -> Option<ElementSpan<'a>> {
  let mut base = set_span(&element);

  let element = match element.as_rule() {
    Rule::delimited_block => Some(process_delimited_block(element, env)),
    Rule::title => Some(process_title(element, base)),
    Rule::header | Rule::title_block => {
      Some(element.into_inner().fold(base, |base, subelement| {
        match subelement.as_rule() {
          Rule::title => process_title(subelement, base.clone()),
          Rule::anchor => process_anchor(subelement, base),
          // We just take the attributes at the beginning
          // of the element.
          _ => base,
        }
      }))
    }
    Rule::paragraph => Some(process_paragraph(element)),
    Rule::block | Rule::list => extract_inner_rule(element, env),
    Rule::list_paragraph => Some(process_paragraph(element)),
    Rule::other_list_inline => Some(from_element(&element, Element::Text)),
    Rule::continuation => None,
    Rule::bullet_list => Some(process_children(
      element.clone(),
      set_span(&element).element(Element::List(ListType::Bullet)),
      env,
    )),
    Rule::numbered_list => Some(process_children(
      element.clone(),
      set_span(&element).element(Element::List(ListType::Number)),
      env,
    )),
    Rule::bullet_list_element | Rule::number_bullet_list_element => Some(
      element
        .clone()
        .into_inner()
        .fold(set_span(&element), |base, sub| match sub.as_rule() {
          Rule::bullet | Rule::number_bullet => {
            base.element(Element::ListItem(sub.as_str().trim().len() as u32))
          }
          Rule::list_element => process_children(sub, base, env),
          Rule::EOI => base,
          _ => {
            let mut base = base;
            base.children.push(set_span(&sub));
            base
          }
        }),
    ),
    Rule::image_block => Some(process_image(element, env)),
    Rule::inline => Some(process_inline(element, base)),
    Rule::EOI => None,
    _ => Some(base),
  };

  element
}

fn process_anchor<'a>(element: Pair<'a, asciidoc::Rule>, base: ElementSpan<'a>) -> ElementSpan<'a> {
  element
    .into_inner()
    .fold(base, |base, element| match element.as_rule() {
      Rule::inline_anchor => process_inline_anchor(element, base),
      _ => base,
    })
}

fn process_inline_anchor<'a>(
  element: Pair<'a, asciidoc::Rule>,
  base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  element.into_inner().fold(base, |base, element| {
    match element.as_rule() {
      Rule::identifier => base.add_attribute(Attribute {
        key: "anchor".to_string(),
        value: AttributeValue::Ref(element.as_str()),
      }),
      // TODO Fehler abfangen und anzeigen
      _ => base,
    }
  })
}

fn process_inline_attribute_list<'a>(
  element: Pair<'a, asciidoc::Rule>,
  base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  element.into_inner().fold(base, |base, sub| {
    match sub.as_rule() {
      Rule::attribute => {
        sub.into_inner().fold(base, |base, sub| {
          match sub.as_rule() {
            Rule::attribute_value => {
              base.add_positional_attribute(AttributeValue::Ref(sub.as_str()))
            }
            Rule::named_attribute => {
              let mut key = None;
              let mut value = None;

              for subelement in sub.into_inner() {
                match subelement.as_rule() {
                  Rule::identifier => key = Some(subelement.as_str()),
                  Rule::attribute_value => {
                    value = Some(subelement.into_inner().concat());
                  }
                  // TODO Fehler abfangen und anzeigen
                  _ => (),
                }
              }

              base.add_attribute(Attribute {
                key: key.unwrap().to_string(),
                value: AttributeValue::String(value.unwrap()),
              })
            }
            // TODO Fehler abfangen und anzeigen
            _ => base,
          }
        })
      }
      // TODO Fehler abfangen und anzeigen
      _ => base,
    }
  })
}

fn process_attribute_list<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  for element in element.into_inner() {
    match element.as_rule() {
      Rule::inline_attribute_list => {
        base = process_inline_attribute_list(element, base);
      }
      _ => (),
    };
  }
  base
}

fn process_blocktitle<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  for element in element.into_inner() {
    match element.as_rule() {
      Rule::line => {
        base.attributes.push(Attribute {
          key: "title".to_string(), // TODO
          value: AttributeValue::Ref(element.as_str()),
        });
      }
      _ => (),
    };
  }
  base
}

fn process_delimited_block<'a>(
  element: Pair<'a, asciidoc::Rule>,
  env: &mut Env,
) -> ElementSpan<'a> {
  let mut base = set_span(&element);

  for subelement in element.into_inner() {
    match subelement.as_rule() {
      Rule::anchor => {
        base = process_anchor(subelement, base);
      }
      Rule::attribute_list => {
        base = process_attribute_list(subelement, base);
      }
      Rule::blocktitle => {
        base = process_blocktitle(subelement, base);
      }
      Rule::delimited_table => {
        base.element = Element::Table;
        base = process_inner_table(subelement, base, env);
      }
      Rule::delimited_comment => {
        base.element = Element::TypedBlock {
          kind: BlockType::Comment,
        };
        base = process_delimited_inner(subelement, base, env);
      }
      Rule::delimited_source => {
        base.element = Element::TypedBlock {
          kind: BlockType::Listing,
        };
        base = process_delimited_inner(subelement, base, env);
      }
      Rule::delimited_literal => {
        base.element = Element::TypedBlock {
          kind: BlockType::Listing,
        };
        base = process_delimited_inner(subelement, base, env);
      }
      Rule::delimited_example => {
        base.element = Element::TypedBlock {
          kind: BlockType::Example,
        };
        base = process_delimited_inner(subelement, base, env);
      }
      // We just take the attributes at the beginning
      // of the element.
      _ => {
        break;
      } // TODO improve matching
    }
  }

  base
}

fn process_delimited_inner<'a>(
  element: Pair<'a, asciidoc::Rule>,
  base: ElementSpan<'a>,
  env: &mut Env,
) -> ElementSpan<'a> {
  element.into_inner().fold(base, |base, element| {
    let mut base = base;

    match element.as_rule() {
      Rule::delimited_inner => {
        if let Element::TypedBlock {
          kind: BlockType::Example,
        } = base.element
        {
          let ast = AsciidocParser::parse(Rule::asciidoc, element.as_str()).unwrap();

          for element in ast {
            if let Some(e) = process_element(element, env) {
              base.children.push(e);
            }
          }
        }
        base.add_attribute(Attribute {
          key: "content".to_string(),
          value: AttributeValue::Ref(element.as_str()),
        })
      }
      _ => base,
    }
  })
}

fn process_title<'a>(element: Pair<'a, asciidoc::Rule>, base: ElementSpan<'a>) -> ElementSpan<'a> {
  match element.as_rule() {
    Rule::title => {
      element.into_inner().fold(base, |base, subelement| {
        match subelement.as_rule() {
          Rule::atx_title_style => base.element(Element::Title {
            level: subelement.as_str().trim().len() as u32,
          }),
          Rule::setext_title_style => base.clone().element(Element::Title {
            level: match subelement.as_str().chars().next().unwrap() {
              '=' => 1,
              '-' => 2,
              '~' => 3,
              '^' => 4,
              _ => {
                return base.element(Element::Error("Unsupported title formatting".to_string()));
              }
            },
          }),
          Rule::line => base.add_attribute(Attribute {
            key: "name".to_string(),
            value: AttributeValue::Ref(subelement.as_str()),
          }),
          // We just take the attributes at the beginning
          // of the element.
          _ => base.element(Element::Error("Unsupported title formatting".to_string())),
        }
      })
    }
    _ => base,
  }
}

fn parse_paragraph<'a>(content: &'a str) -> Vec<ElementSpan<'a>> {
  let mut out = vec![];

  let ast = AsciidocParser::parse(Rule::inline_parser, content).unwrap();

  for element in ast {
    for subelement in element.into_inner() {
      if subelement.as_rule() != Rule::EOI {
        out.push(match subelement.as_rule() {
          Rule::other_inline | Rule::other_list_inline => from_element(&subelement, Element::Text),
          Rule::inline => process_inline(subelement.clone(), set_span(&subelement)),
          _ => set_span(&subelement),
        });
      }
    }
  }

  out
}

fn process_paragraph<'a>(element: Pair<'a, asciidoc::Rule>) -> ElementSpan<'a> {
  let mut base = from_element(&element, Element::Paragraph);

  base.children = parse_paragraph(element.as_str())
    .into_iter()
    .map(|child| child.add_offset(&base))
    .collect();

  base
}

fn process_inline<'a>(element: Pair<'a, asciidoc::Rule>, base: ElementSpan<'a>) -> ElementSpan<'a> {
  element
    .into_inner()
    .fold(base, |base, element| match element.as_rule() {
      Rule::link => process_link(element, base),
      Rule::xref => process_xref(element, base),
      Rule::monospaced | Rule::strong | Rule::emphasized => {
        let base = base.element(Element::Styled).add_attribute(Attribute {
          key: "style".to_string(),
          value: AttributeValue::Ref(match element.as_rule() {
            Rule::monospaced => "monospaced",
            Rule::strong => "strong",
            Rule::emphasized => "em",
            _ => "not_supported",
          }),
        });

        let base = match concat_elements(element.clone(), Rule::linechar, "") {
          Some(content) => base.add_attribute(Attribute {
            key: "content".to_string(),
            value: AttributeValue::String(content),
          }),
          None => base,
        };
        element
          .into_inner()
          .fold(base, |base, subelement| match subelement.as_rule() {
            Rule::inline_anchor => process_inline_anchor(subelement, base),
            _ => base,
          })
      }
      _ => base,
    })
}

fn process_link<'a>(element: Pair<'a, asciidoc::Rule>, base: ElementSpan<'a>) -> ElementSpan<'a> {
  element
    .into_inner()
    .fold(base.element(Element::Link), |base, element| {
      match element.as_rule() {
        Rule::url => {
          let base = base.add_attribute(Attribute {
            key: "url".to_string(),
            value: AttributeValue::Ref(element.as_str()),
          });
          let element = element.into_inner().next().unwrap(); // TODO Fehler möglich?
          base.add_attribute(Attribute {
            key: "protocol".to_string(),
            value: AttributeValue::Ref(element.as_str()),
          })
        }
        Rule::inline_attribute_list => process_inline_attribute_list(element, base),
        _ => base.add_child(set_span(&element)),
      }
    })
}

fn process_xref<'a>(element: Pair<'a, asciidoc::Rule>, base: ElementSpan<'a>) -> ElementSpan<'a> {
  let base = element
    .clone()
    .into_inner()
    .fold(base.element(Element::XRef), |base, element| {
      match element.as_rule() {
        Rule::identifier => base.add_attribute(Attribute {
          key: "id".to_string(),
          value: AttributeValue::Ref(element.as_str()),
        }),
        Rule::word => base,
        _ => base,
      }
    });

  match concat_elements(element, Rule::word, " ") {
    Some(content) => base.add_attribute(Attribute {
      key: "content".to_string(),
      value: AttributeValue::String(content),
    }),
    None => base,
  }
}

fn process_image<'a>(element: Pair<'a, asciidoc::Rule>, env: &mut Env) -> ElementSpan<'a> {
  let base = element.clone().into_inner().flatten().fold(
    set_span(&element).element(Element::Image),
    |base, element| match element.as_rule() {
      Rule::url => base.add_attribute(Attribute {
        key: "path".to_string(),
        value: AttributeValue::Ref(element.as_str()),
      }),
      Rule::path => base.add_attribute(Attribute {
        key: "path".to_string(),
        value: AttributeValue::Ref(element.as_str()),
      }),
      Rule::inline_attribute_list => process_inline_attribute_list(element, base),
      _ => base,
    },
  );

  match base.get_attribute("opts") {
    Some("inline") => match base.get_attribute("path") {
      Some(path) => match env.read_to_string(path) {
        Ok(content) => base.add_attribute(Attribute {
          key: "content".to_string(),
          value: AttributeValue::String(content),
        }),
        Err(e) => base.clone().element(Element::Error(format!(
          "couldn't read content of image file {} ({})",
          path, e
        ))),
      },
      None => base.element(Element::Error(
        "There was no path of inline image defined".to_string(),
      )),
    },
    Some(_) | None => base,
  }
}

#[derive(Debug, PartialEq)]
enum ColKind {
  Default,
  Asciidoc,
}

#[derive(Debug, PartialEq)]
struct ColumnFormat {
  length: usize,
  kind: ColKind,
}

fn parse_column_format(input: &str) -> ColumnFormat {
  ColumnFormat {
    length: 1,
    kind: match input {
      "a" => ColKind::Asciidoc,
      _ => ColKind::Default,
    },
  }
}

fn parse_columns_format(input: &str) -> Vec<ColumnFormat> {
  input
    .split(',')
    .map(|input| parse_column_format(input.trim()))
    .collect()
}

fn parse_columns_format_from_content(input: &str) -> Vec<ColumnFormat> {
  input
    .lines()
    .next()
    .unwrap_or("")
    .matches('|')
    .map(|_| ColumnFormat {
      length: 1,
      kind: ColKind::Default,
    })
    .collect()
}

fn process_inner_table<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
  env: &mut Env,
) -> ElementSpan<'a> {
  let content = element
    .into_inner()
    .find(|sub| sub.as_rule() == Rule::delimited_inner)
    .unwrap()
    .as_str();

  let col_format = match base.get_attribute("cols") {
    Some(fmt) => parse_columns_format(fmt),
    None => parse_columns_format_from_content(content),
  };

  base.attributes.push(Attribute {
    key: "content".to_string(),
    value: AttributeValue::Ref(content),
  });
  base.children = process_table_content(content, col_format, env);

  base
}

fn process_table_content<'a>(
  input: &'a str,
  col_format: Vec<ColumnFormat>,
  env: &mut Env,
) -> Vec<ElementSpan<'a>> {
  let ast = AsciidocParser::parse(Rule::table_inner, input).unwrap();

  let mut cells = vec![];

  for element in ast {
    for (subelement, fmt) in element.into_inner().zip(col_format.iter().cycle()) {
      cells.push(process_table_cell(subelement, fmt, env))
    }
  }

  let mut rows = vec![];
  let len = col_format.len();
  for chunk in cells.chunks(len) {
    rows.push(ElementSpan {
      element: Element::TableRow,
      source: None,
      content: "",
      start: 0,
      end: 0,
      start_line: 0,
      start_col: 0,
      end_line: 0,
      end_col: 0,
      children: chunk.to_vec(),
      positional_attributes: vec![],
      attributes: vec![],
    })
  }

  rows
}

fn process_table_cell<'a>(
  element: Pair<'a, asciidoc::Rule>,
  fmt: &ColumnFormat,
  env: &mut Env,
) -> ElementSpan<'a> {
  let mut base = set_span(&element).element(Element::TableCell);

  let content = element
    .into_inner()
    .find(|sub| sub.as_rule() == Rule::table_cell_content)
    .unwrap()
    .as_str()
    .trim();

  base.content = content;
  base.children = match fmt.kind {
    ColKind::Asciidoc => {
      let ast = AsciidocParser::parse(Rule::asciidoc, content).unwrap();

      let mut elements = vec![];

      for element in ast {
        if let Some(element) = process_element(element, env) {
          elements.push(element);
        }
      }
      elements
    }
    ColKind::Default => {
      let mut base = base.clone();
      base.element = Element::Paragraph;
      base.children = parse_paragraph(content);

      vec![base]
    }
  };

  base
}

// Helper functions

fn concat_elements<'a>(
  element: Pair<'a, asciidoc::Rule>,
  filter: asciidoc::Rule,
  join: &str,
) -> Option<String> {
  let elements: Vec<_> = element
    .into_inner()
    .filter(|e| e.as_rule() == filter)
    .map(|e| e.as_str())
    .collect();

  if elements.len() > 0 {
    Some(elements.join(join))
  } else {
    None
  }
}

fn process_children<'a>(
  element: Pair<'a, asciidoc::Rule>,
  base: ElementSpan<'a>,
  env: &mut Env,
) -> ElementSpan<'a> {
  let mut base = base;

  base.children = element
    .into_inner()
    .filter_map(|sub| process_element(sub, env))
    .collect();

  base
}

fn extract_inner_rule<'a>(
  element: Pair<'a, asciidoc::Rule>,
  env: &mut Env,
) -> Option<ElementSpan<'a>> {
  let base = set_span(&element);
  match element.into_inner().next() {
    Some(element) => process_element(element, env),
    None => Some(base.element(Element::Error(
      "must have a subfield in the parser but nothing is found".to_string(),
    ))),
  }
}

fn set_span<'a>(element: &Pair<'a, asciidoc::Rule>) -> ElementSpan<'a> {
  from_element(
    element,
    Element::Error(format!("Not implemented:{:?}", element)),
  )
}

fn from_element<'a>(rule: &Pair<'a, asciidoc::Rule>, element: Element<'a>) -> ElementSpan<'a> {
  let (start_line, start_col) = rule.as_span().start_pos().line_col();
  let (end_line, end_col) = rule.as_span().end_pos().line_col();

  ElementSpan {
    element,
    source: None, // TODO
    content: rule.as_str(),
    children: Vec::new(),
    attributes: Vec::new(),
    positional_attributes: Vec::new(),
    start: rule.as_span().start(),
    end: rule.as_span().end(),
    start_line,
    start_col,
    end_line,
    end_col,
  }
}

#[cfg(test)]
mod test {
  use pretty_assertions::assert_eq;

  use super::*;

  #[test]
  fn test_table() {
    let out = parse_columns_format(r#"1,a"#);
    assert_eq!(
      out,
      vec![
        ColumnFormat {
          length: 1,
          kind: ColKind::Default
        },
        ColumnFormat {
          length: 1,
          kind: ColKind::Asciidoc
        },
      ]
    );
  }

}
