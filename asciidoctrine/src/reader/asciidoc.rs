pub use crate::ast::*;
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
  fn parse<'a>(&self, input: &'a str) -> Result<AST<'a>> {
    parse_ast(input)
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
        for subelement in element.into_inner() {
          match subelement.as_rule() {
            Rule::identifier => {
              base.attributes.push(Attribute {
                key: "anchor".to_string(),
                value: AttributeValue::Ref(subelement.as_str()),
              });
            }
            // TODO Fehler abfangen und anzeigen
            _ => (),
          }
        }
      }
      _ => (),
    };
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

fn process_delimited_inner<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  for element in element.into_inner() {
    match element.as_rule() {
      Rule::delimited_inner => {
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

fn process_xref<'a>(
  element: Pair<'a, asciidoc::Rule>,
  mut base: ElementSpan<'a>,
) -> ElementSpan<'a> {
  base.element = Element::XRef;
  for element in element.into_inner() {
    match element.as_rule() {
      Rule::identifier => {
        base.attributes.push(Attribute {
          key: "id".to_string(),
          value: AttributeValue::Ref(element.as_str()),
        });
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
      _ => {
        base.element = Element::Error("Not implemented".to_string());
      }
    };
  }
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
    start_line: start_line,
    start_col: start_col,
    end_line: end_line,
    end_col: end_col,
  }
}

fn process_element<'a>(element: Pair<'a, asciidoc::Rule>) -> Option<ElementSpan<'a>> {
  let mut base = set_span(&element);

  let element = match element.as_rule() {
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
    Rule::block => {
      for subelement in element.into_inner() {
        if let Some(e) = process_element(subelement) {
          base = e;
        }
      }
      Some(base)
    }
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
          // TODO wir müssen die anderen delimitets verarbeiten
          Rule::delimited_source => {
            base.element = Element::TypedBlock {
              kind: BlockType::Listing,
            };
            base = process_delimited_inner(subelement, base);
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
    Rule::paragraph => {
      base.element = Element::Paragraph;
      for subelement in element.into_inner() {
        let mut sub = set_span(&subelement);
        match subelement.as_rule() {
          Rule::other_inline => {
            sub.element = Element::Text;
            // TODO Newlines entfernen? Als Attribut?
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
      Some(base)
    }
    Rule::delimited_source => {
      None // TODO
    }
    Rule::delimited_inner => {
      None // TODO
    }
    Rule::EOI => None,
    _ => {
      base.element = Element::Error("Not implemented".to_string()); // TODO
      Some(base)
    }
  };

  element
}

// TODO Add Options
// TODO Add Result
pub fn parse_ast(input: &str) -> Result<AST> {
  // TODO Den Text parsen
  // TODO Result auswerten
  let ast = AsciidocParser::parse(Rule::asciidoc, input).expect("couldn't parse input.");

  let mut attributes = Vec::new();
  let mut elements = Vec::new();

  for element in ast {
    //println!("{:#?}", element); // TODO entfernen
    if let Some(element) = process_element(element) {
      elements.push(element);
    }
  }

  Ok(AST {
    content: input,
    elements: elements,
    attributes: attributes,
  })
}
