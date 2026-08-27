pub use crate::ast::*;
use crate::Result;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

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

    let mut aliases: HashMap<String, String> = HashMap::new();
    let mut participants_list = Vec::new();
    let mut elements = Vec::new();
    let mut has_checked_state = false;

    for pair in ast {
      parse_pair(
        pair,
        &mut aliases,
        &mut participants_list,
        &mut elements,
        &mut has_checked_state,
      );
    }

    if !has_checked_state {
      elements.insert(
        0,
        create_checked_state("Service Discovery".to_string(), participants_list),
      );
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

fn parse_pair<'a>(
  pair: Pair<'a, Rule>,
  aliases: &mut HashMap<String, String>,
  participants: &mut Vec<String>,
  elements: &mut Vec<ElementSpan>,
  has_checked_state: &mut bool,
) {
  match pair.as_rule() {
    Rule::mermaid | Rule::sequence_diagram => {
      for inner in pair.into_inner() {
        parse_pair(inner, aliases, participants, elements, has_checked_state);
      }
    }
    Rule::participant_declaration => {
      if let Some((alias, display_name)) = parse_participant_declaration(pair) {
        aliases.insert(alias, display_name.clone());
        push_unique(participants, display_name);
      }
    }
    Rule::note => {
      if let Some(name) = parse_note(pair) {
        elements.push(create_checked_state(name, participants.clone()));
        *has_checked_state = true;
      }
    }
    Rule::message => {
      if let Some((from, to, message)) = parse_message(pair, aliases) {
        push_unique(participants, from.clone());
        push_unique(participants, to.clone());
        elements.push(create_message(from, to, message));
      }
    }
    Rule::activation | Rule::deactivation | Rule::blank_line => {}
    _ => {
      for inner in pair.into_inner() {
        parse_pair(inner, aliases, participants, elements, has_checked_state);
      }
    }
  }
}

fn parse_participant_declaration<'a>(pair: Pair<'a, Rule>) -> Option<(String, String)> {
  let line = pair.as_str().trim();
  let line = line.strip_prefix("participant")?.trim();
  let (alias, display_name) = line.split_once(" as ")?;

  let alias = alias.trim();
  let display_name = display_name.trim();

  if alias.is_empty() || display_name.is_empty() {
    None
  } else {
    Some((alias.to_string(), display_name.to_string()))
  }
}

fn parse_note<'a>(pair: Pair<'a, Rule>) -> Option<String> {
  let (_, text) = pair.as_str().split_once(':')?;
  let text = text.trim();

  if text.is_empty() {
    None
  } else {
    Some(text.to_string())
  }
}

fn parse_message<'a>(
  pair: Pair<'a, Rule>,
  aliases: &HashMap<String, String>,
) -> Option<(String, String, String)> {
  let line = pair.as_str().trim();
  let (arrow_index, arrow_len) = if let Some(index) = line.find("-->>") {
    (index, 4)
  } else if let Some(index) = line.find("->>") {
    (index, 3)
  } else {
    return None;
  };

  let from = resolve_participant(&line[..arrow_index], aliases);
  let rest = &line[arrow_index + arrow_len..];
  let (to, message) = rest.split_once(':')?;

  let from = from.trim().to_string();
  let to = resolve_participant(to, aliases);
  let message = message.trim().to_string();

  if from.is_empty() || to.is_empty() {
    None
  } else {
    Some((from, to, message))
  }
}

fn resolve_participant(name: &str, aliases: &HashMap<String, String>) -> String {
  aliases
    .get(name.trim())
    .cloned()
    .unwrap_or_else(|| name.trim().to_string())
}

fn push_unique(items: &mut Vec<String>, item: String) {
  if !items.contains(&item) {
    items.push(item);
  }
}

fn create_checked_state(name: String, participants: Vec<String>) -> ElementSpan {
  ElementSpan {
    source: None,
    position: TextPosition::Slice(Slice { start: 0, end: 0 }),
    element: Element::Sequence(SequenceDiagramElement::CheckedState { name, participants }),
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
