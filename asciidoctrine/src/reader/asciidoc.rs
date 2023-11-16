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

fn process_anchor<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  for element in element.into_inner() {
    match element.as_rule() {
      Rule::inline_anchor => {
        base = process_inline_anchor(element, base);
      }
      _ => (),
    };
  }
  base
}

fn process_inline_anchor<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  for element in element.into_inner() {
    match element.as_rule() {
      Rule::identifier => {
        base.attributes.push(Attribute {
          key: "anchor".to_string(),
          value: AttributeValue::Ref(element.as_str()),
        });
      }
      // TODO Fehler abfangen und anzeigen
      _ => (),
    }
  }
  base
}

fn process_inline_attribute_list<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  for subelement in element.into_inner() {
    match subelement.as_rule() {
      Rule::attribute => {
        for subelement in subelement.into_inner() {
          match subelement.as_rule() {
            Rule::attribute_value => {
              // TODO Wir müssen unterschiedlich damit umgehen, ob ein oder mehrere
              // identifier existieren
              base
                .positional_attributes
                .push(AttributeValue::Ref(subelement.as_str()));
            }
            Rule::named_attribute => {
              let mut key = None;
              let mut value = None;

              for subelement in subelement.into_inner() {
                match subelement.as_rule() {
                  Rule::identifier => key = Some(subelement.as_str()),
                  Rule::attribute_value => {
                    value = Some(subelement.into_inner().concat());
                  }
                  // TODO Fehler abfangen und anzeigen
                  _ => (),
                }
              }

              base.attributes.push(Attribute {
                key: key.unwrap().to_string(),
                value: AttributeValue::String(value.unwrap()),
              });
            }
            // TODO Fehler abfangen und anzeigen
            _ => (),
          }
        }
      }
      // TODO Fehler abfangen und anzeigen
      _ => (),
    }
  }
  base
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

fn process_delimited_inner<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
  env: &mut Env,
) -> ElementSpan<'a> {
  for element in element.into_inner() {
    match element.as_rule() {
      Rule::delimited_inner => {
        if let Element::TypedBlock { kind: BlockType::Example } = base.element {
          let ast = AsciidocParser::parse(Rule::asciidoc, element.as_str()).unwrap();

          for element in ast {
            if let Some(e) = process_element(element, env) {
              base.children.push(e);
            }
          }
        }
        base.attributes.push(Attribute {
          key: "content".to_string(), // TODO
          value: AttributeValue::Ref(element.as_str()),
        });
      }
      _ => (),
    };
  }
  base
}

fn process_inner_table<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
  env: &mut Env,
) -> ElementSpan<'a> {
  for element in element.into_inner() {
    match element.as_rule() {
      Rule::delimited_inner => {
        let ast = AsciidocParser::parse(Rule::table_inner, element.as_str()).unwrap();

        for element in ast {
          for subelement in element.into_inner() {
            if let Some(e) = process_element(subelement, env) {
              base.children.push(e);
            }
          }
        }
        base.attributes.push(Attribute {
          key: "content".to_string(),
          value: AttributeValue::Ref(element.as_str()),
        });
      }
      _ => (),
    };
  }
  base
}

fn process_table_row<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
  env: &mut Env,
) -> ElementSpan<'a> {
  base.element = Element::TableRow;
  for cell_element in element.into_inner() {
      let cell = process_table_cell(cell_element, base.clone(), env);
      base.children.push(cell);
  }
  base
}

fn process_table_cell<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
  _env: &mut Env,
) -> ElementSpan<'a> {
  base.element = Element::TableCell;
  base.content = element.into_inner().find(|sub| sub.as_rule() == Rule::table_cell_content).unwrap().as_str().trim();
  base
}

fn process_title<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> Option<ElementSpan<'a>> {
  match element.as_rule() {
    Rule::title => {
      for subelement in element.into_inner() {
        match subelement.as_rule() {
          Rule::atx_title_style => {
            base.element = Element::Title {
              level: subelement.as_str().trim().len() as u32,
            };
          }
          Rule::setext_title_style => {
            let ch = subelement.as_str().chars().next().unwrap(); // TODO Check None?
            let level;

            match ch {
              '=' => {
                level = 1;
              }
              '-' => {
                level = 2;
              }
              '~' => {
                level = 3;
              }
              '^' => {
                level = 4;
              }
              _ => {
                base.element = Element::Error("Unsupported title formatting".to_string());
                break;
              }
            }
            base.element = Element::Title {
              level: level as u32,
            };
          }
          Rule::line => {
            base.attributes.push(Attribute {
              key: "name".to_string(),
              value: AttributeValue::Ref(subelement.as_str()),
            });
          }
          // We just take the attributes at the beginning
          // of the element.
          _ => {
            break; // TODO Error
          } // TODO improve matching
        }
      }
    }
    _ => {
      base.element = Element::Error("Not implemented".to_string());
    } // TODO
  };

  Some(base)
}

fn process_paragraph<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  base.element = Element::Paragraph;

  for subelement in element.into_inner() {
    let mut sub = set_span(&subelement);
    match subelement.as_rule() {
      Rule::other_inline => {
        sub.element = Element::Text;
        // TODO Newlines entfernen? Als Attribut?
      }
      Rule::other_list_inline => {
        sub.element = Element::Text;
      }
      Rule::inline => {
        sub = process_inline(subelement, sub);
      }
      _ => {
        sub.element = Element::Error("Not implemented!".to_string());
      }
    }
    base.children.push(sub);
  }

  base
}

fn process_inline<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  for element in element.into_inner() {
    match element.as_rule() {
      Rule::link => {
        base = process_link(element, base);
      }
      Rule::xref => {
        base = process_xref(element, base);
      }
      Rule::monospaced => {
        base.element = Element::Styled;
        base.attributes.push(Attribute {
          key: "style".to_string(),
          value: AttributeValue::Ref("monospaced"),
        });

        if let Some(content) = concat_elements(element.clone(), Rule::linechar, "") {
          base.attributes.push(Attribute {
            key: "content".to_string(),
            value: AttributeValue::String(content),
          });
        };
        for subelement in element.into_inner() {
          match subelement.as_rule() {
            Rule::inline_anchor => {
              base = process_inline_anchor(subelement, base);
            }
            _ => (),
          }
        }
      }
      Rule::strong => {
        base.element = Element::Styled;
        base.attributes.push(Attribute {
          key: "style".to_string(),
          value: AttributeValue::Ref("strong"),
        });

        if let Some(content) = concat_elements(element, Rule::linechar, "") {
          base.attributes.push(Attribute {
            key: "content".to_string(),
            value: AttributeValue::String(content),
          });
        };
      }
      Rule::emphasized => {
        base.element = Element::Styled;
        base.attributes.push(Attribute {
          key: "style".to_string(),
          value: AttributeValue::Ref("em"),
        });

        if let Some(content) = concat_elements(element, Rule::linechar, "") {
          base.attributes.push(Attribute {
            key: "content".to_string(),
            value: AttributeValue::String(content),
          });
        };
      }
      _ => {
        base.element = Element::Error("Not implemented".to_string());
      }
    };
  }
  base
}

fn process_link<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  base.element = Element::Link;
  for element in element.into_inner() {
    match element.as_rule() {
      Rule::url => {
        base.attributes.push(Attribute {
          key: "url".to_string(),
          value: AttributeValue::Ref(element.as_str()),
        });
        let element = element.into_inner().next().unwrap(); // TODO Fehler möglich?
        base.attributes.push(Attribute {
          key: "protocol".to_string(),
          value: AttributeValue::Ref(element.as_str()),
        });
      }
      Rule::inline_attribute_list => {
        base = process_inline_attribute_list(element, base);
      }
      _ => {
        let mut sub = set_span(&element);
        sub.element = Element::Error("Not implemented".to_string());
        base.children.push(sub);
      }
    };
  }
  base
}

fn process_xref<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  base.element = Element::XRef;
  for element in element.clone().into_inner() {
    match element.as_rule() {
      Rule::identifier => {
        base.attributes.push(Attribute {
          key: "id".to_string(),
          value: AttributeValue::Ref(element.as_str()),
        });
      }
      Rule::word => {}
      _ => (),
    };
  }

  if let Some(content) = concat_elements(element, Rule::word, " ") {
    base.attributes.push(Attribute {
      key: "content".to_string(),
      value: AttributeValue::String(content),
    });
  };

  base
}

fn process_image<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
  env: &mut Env,
) -> ElementSpan<'a> {
  base.element = Element::Image;
  for element in element.into_inner().flatten() {
    match element.as_rule() {
      Rule::url => {
        base.attributes.push(Attribute {
          key: "path".to_string(),
          value: AttributeValue::Ref(element.as_str()),
        });
      }
      Rule::path => {
        base.attributes.push(Attribute {
          key: "path".to_string(),
          value: AttributeValue::Ref(element.as_str()),
        });
      }
      Rule::inline_attribute_list => {
        base = process_inline_attribute_list(element, base);
      }
      _ => (),
    };
  }

  // TODO Prüfen ob eine inline Anweisung vorhanden ist und
  // falls ja, die Datei einlesen
  if let Some(value) = base.get_attribute("opts") {
    if value == "inline" {
      // TODO Die Datei einlesen
      if let Some(path) = base.get_attribute("path") {
        match env.read_to_string(path) {
          Ok(content) => {
            base.attributes.push(Attribute {
              key: "content".to_string(),
              value: AttributeValue::String(content),
            });
          }
          Err(e) => {
            error!("couldn't read content of image file {} ({})", path, e);
          }
        }
      } else {
        error!("There was no path of inline image defined");
      }
    }
  }

  base
}

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

fn set_span<'a>(element: &Pair<'a, asciidoc::Rule>) -> ElementSpan<'a> {
  let (start_line, start_col) = element.as_span().start_pos().line_col();
  let (end_line, end_col) = element.as_span().end_pos().line_col();

  ElementSpan {
    element: Element::Error("Root".to_string()),
    source: None, // TODO
    content: element.as_str(),
    children: Vec::new(),
    attributes: Vec::new(),
    positional_attributes: Vec::new(),
    start: element.as_span().start(),
    end: element.as_span().end(),
    start_line,
    start_col,
    end_line,
    end_col,
  }
}

fn process_element<'a>(element: Pair<'a, asciidoc::Rule>, env: &mut Env) -> Option<ElementSpan<'a>> {
  let mut base = set_span(&element);

  let element = match element.as_rule() {
    Rule::delimited_block => {
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
          Rule::delimited_table => {
            base.element = Element::Table;
            base = process_inner_table(subelement, base, env);
          }
          // We just take the attributes at the beginning
          // of the element.
          _ => {
            break;
          } // TODO improve matching
        }
      }
      Some(base)
    }
    Rule::header => {
      for subelement in element.into_inner() {
        match subelement.as_rule() {
          Rule::title => {
            if let Some(e) = process_title(subelement, base.clone()) {
              base = e;
            }
          }
          // We just take the attributes at the beginning
          // of the element.
          _ => {
            break;
          } // TODO improve matching
        }
      }
      // TODO
      Some(base)
    }
    Rule::title => process_title(element, base),
    Rule::title_block => {
      for subelement in element.into_inner() {
        match subelement.as_rule() {
          Rule::title => {
            if let Some(e) = process_title(subelement, base.clone()) {
              base = e;
            }
          }
          Rule::anchor => {
            base = process_anchor(subelement, base);
          }
          // We just take the attributes at the beginning
          // of the element.
          _ => {
            break;
          } // TODO improve matching
        }
      }
      Some(base)
    }
    Rule::paragraph => Some(process_paragraph(element, base)),
    Rule::list => {
      for subelement in element.into_inner() {
        if let Some(e) = process_element(subelement, env) {
          base = e;
        }
      }
      Some(base)
    }
    Rule::list_paragraph => Some(process_paragraph(element, base)),
    Rule::other_list_inline => {
      base.element = Element::Text;
      Some(base)
    }
    Rule::continuation => None,
    Rule::bullet_list => {
      base.element = Element::List(ListType::Bullet);

      for subelement in element.into_inner() {
        if let Some(e) = process_element(subelement, env) {
          base.children.push(e);
        }
      }

      Some(base)
    }
    Rule::bullet_list_element => {
      for subelement in element.into_inner() {
        match subelement.as_rule() {
          Rule::bullet => {
            base.element = Element::ListItem(subelement.as_str().trim().len() as u32);
          }
          Rule::list_element => {
            for subelement in subelement.into_inner() {
              if let Some(e) = process_element(subelement, env) {
                base.children.push(e);
              }
            }
          }
          _ => {
            let mut e = set_span(&subelement);
            e.element = Element::Error("Not implemented".to_string());
            base.children.push(e);
          }
        }
      }

      Some(base)
    }
    Rule::numbered_list => {
      base.element = Element::List(ListType::Number);

      for subelement in element.into_inner() {
        if let Some(e) = process_element(subelement, env) {
          base.children.push(e);
        }
      }

      Some(base)
    }
    Rule::number_bullet_list_element => {
      for subelement in element.into_inner() {
        match subelement.as_rule() {
          Rule::number_bullet => {
            base.element = Element::ListItem(subelement.as_str().trim().len() as u32);
          }
          Rule::list_element => {
            for subelement in subelement.into_inner() {
              if let Some(e) = process_element(subelement, env) {
                base.children.push(e);
              }
            }
          }
          _ => {
            let mut e = set_span(&subelement);
            e.element = Element::Error("Not implemented".to_string());
            base.children.push(e);
          }
        }
      }

      Some(base)
    }
    Rule::image_block => Some(process_image(element, base, env)),
    Rule::block => {
      for subelement in element.into_inner() {
        if let Some(e) = process_element(subelement, env) {
          base = e;
        }
      }
      Some(base)
    }
    Rule::inline => Some(process_inline(element, base)),
    Rule::table_row => Some(process_table_row(element, base, env)),
    Rule::table_cell => Some(process_table_cell(element, base, env)),
    Rule::EOI => None,
    _ => {
      base.element = Element::Error(format!("Not implemented:{:?}", element));
      Some(base)
    }
  };

  element
}
