pub use crate::ast::*;
use crate::Result;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

pub struct MermaidReader {}

impl MermaidReader {
  pub fn new() -> Self {
    MermaidReader {}
  }
}

impl crate::Reader for MermaidReader {
  fn parse<'a>(&self, input: &'a str) -> Result<AST<'a>> {
    let ast = MermaidParser::parse(Rule::mermaid, input)
      .map_err(|e| crate::Error::ParseError(format!("Failed to parse mermaid: {}", e)))?;

    let mut participants_list = Vec::new();
    let mut interactions = Vec::new();

    for element in ast {
      if element.as_rule() == Rule::interaction {
        let interaction = parse_interaction(element);
        if let Some((from, to, message)) = interaction {
          // Track participants in order of first appearance
          if !participants_list.contains(&from) {
            participants_list.push(from.clone());
          }
          if !participants_list.contains(&to) {
            participants_list.push(to.clone());
          }
          interactions.push((from, to, message));
        }
      }
    }

    // Build the output elements
    let mut elements = Vec::new();

    // Add CheckedState with all participants
    elements.push(create_checked_state(participants_list));

    // Add Message elements for each interaction
    for (from, to, message) in interactions {
      elements.push(create_message(from, to, message));
    }

    Ok(AST {
      content: input,
      elements,
    })
  }
}

#[derive(Parser, Debug, Copy, Clone)]
#[grammar = "reader/mermaid.pest"]
pub struct MermaidParser;

fn parse_interaction<'a>(element: Pair<'a, Rule>) -> Option<(String, String, String)> {
  let mut from = String::new();
  let mut to = String::new();
  let mut message = String::new();

  for sub in element.into_inner() {
    match sub.as_rule() {
      Rule::participant => {
        if from.is_empty() {
          from = sub.as_str().trim().to_string();
        } else {
          to = sub.as_str().trim().to_string();
        }
      }
      Rule::message => {
        message = sub.as_str().trim().to_string();
      }
      _ => {}
    }
  }

  if !from.is_empty() && !to.is_empty() {
    Some((from, to, message))
  } else {
    None
  }
}

fn create_checked_state(participants: Vec<String>) -> ElementSpan {
  ElementSpan {
    source: None,
    position: TextPosition::Slice(Slice { start: 0, end: 0 }),
    element: Element::Sequence(SequenceDiagramElement::CheckedState {
      name: "Service Discovery".to_string(),
      participants,
    }),
    children: Vec::new(),
    attrs: Vec::new(),
  }
}

fn create_message(from: String, to: String, message: String) -> ElementSpan {
  ElementSpan {
    source: None,
    position: TextPosition::Slice(Slice { start: 0, end: 0 }),
    element: Element::Sequence(SequenceDiagramElement::Message {
      from,
      to,
      message,
      meta: None,
    }),
    children: Vec::new(),
    attrs: Vec::new(),
  }
}
