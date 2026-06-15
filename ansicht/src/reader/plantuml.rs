pub use crate::ast::*;
use crate::Result;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

pub struct PlantUmlReader {}

impl PlantUmlReader {
  pub fn new() -> Self {
    PlantUmlReader {}
  }
}

impl crate::Reader for PlantUmlReader {
  fn parse<'a>(&self, input: &'a str) -> Result<AST<'a>> {
    let ast = PlantUmlParser::parse(Rule::plantuml, input)
      .map_err(|e| crate::Error::ParseError(format!("Failed to parse plantuml: {}", e)))?;

    let mut aliases = HashMap::new();
    let mut participants = Vec::new();
    let mut events = Vec::new();

    for pair in ast {
      parse_pair(pair, &mut aliases, &mut participants, &mut events);
    }

    let has_checked_state = events
      .iter()
      .any(|event| matches!(event, PlantUmlEvent::CheckedState(_)));

    let mut elements = Vec::new();
    if !has_checked_state {
      elements.push(create_checked_state(
        "Service Discovery".to_string(),
        participants.clone(),
      ));
    }

    for event in events {
      match event {
        PlantUmlEvent::CheckedState(name) => {
          elements.push(create_checked_state(name, participants.clone()));
        }
        PlantUmlEvent::Message { from, to, message } => {
          elements.push(create_message(from, to, message));
        }
      }
    }

    Ok(AST {
      content: input,
      elements,
    })
  }
}

#[derive(Parser, Debug)]
#[grammar = "reader/plantuml.pest"]
pub struct PlantUmlParser;

enum PlantUmlEvent {
  CheckedState(String),
  Message {
    from: String,
    to: String,
    message: String,
  },
}

fn parse_pair(
  pair: Pair<Rule>,
  aliases: &mut HashMap<String, String>,
  participants: &mut Vec<String>,
  events: &mut Vec<PlantUmlEvent>,
) {
  match pair.as_rule() {
    Rule::participant_declaration => {
      if let Some((alias, display_name)) = parse_participant(pair) {
        aliases.insert(alias, display_name.clone());
        push_unique(participants, display_name);
      }
    }
    Rule::note_across => {
      if let Some(name) = parse_note_across(pair) {
        events.push(PlantUmlEvent::CheckedState(name));
      }
    }
    Rule::message_interaction => {
      if let Some((from, to, message)) = parse_message(pair, aliases) {
        push_unique(participants, from.clone());
        push_unique(participants, to.clone());
        events.push(PlantUmlEvent::Message { from, to, message });
      }
    }
    _ => {
      for child in pair.into_inner() {
        parse_pair(child, aliases, participants, events);
      }
    }
  }
}

fn parse_participant(pair: Pair<Rule>) -> Option<(String, String)> {
  let declaration = pair.into_inner().next()?;
  let mut names = declaration
    .into_inner()
    .filter(|pair| pair.as_rule() == Rule::declaration_name)
    .map(|pair| pair.as_str().trim().to_string())
    .collect::<Vec<_>>();

  match names.len() {
    0 => None,
    1 => {
      let name = names.remove(0);
      Some((name.clone(), name))
    }
    _ => {
      let left = names.remove(0);
      let right = names.remove(0);

      if is_quoted(right.as_str()) {
        Some((unquote(&left), unquote(&right)))
      } else {
        Some((unquote(&right), unquote(&left)))
      }
    }
  }
}

fn parse_note_across(pair: Pair<Rule>) -> Option<String> {
  let message = pair
    .into_inner()
    .find(|pair| pair.as_rule() == Rule::note_text)?
    .as_str()
    .trim()
    .to_string();

  if message.is_empty() {
    None
  } else {
    Some(message)
  }
}

fn parse_message(
  pair: Pair<Rule>,
  aliases: &HashMap<String, String>,
) -> Option<(String, String, String)> {
  let mut from = String::new();
  let mut to = String::new();
  let mut arrow = String::new();
  let mut message = String::new();

  for sub in pair.into_inner() {
    match sub.as_rule() {
      Rule::message_sender => from = resolve_participant(sub.as_str(), aliases),
      Rule::message_receiver => to = resolve_participant(sub.as_str(), aliases),
      Rule::arrow => arrow = sub.as_str().to_string(),
      Rule::message_text => message = sub.as_str().trim().to_string(),
      _ => {}
    }
  }

  if from.is_empty() || to.is_empty() {
    None
  } else if arrow.contains('<') && !arrow.contains('>') {
    Some((to, from, message))
  } else {
    Some((from, to, message))
  }
}

fn resolve_participant(name: &str, aliases: &HashMap<String, String>) -> String {
  let name = unquote(name.trim());
  aliases.get(&name).cloned().unwrap_or(name)
}

fn is_quoted(input: &str) -> bool {
  input.starts_with('"') && input.ends_with('"') && input.len() >= 2
}

fn unquote(input: &str) -> String {
  let input = input.trim();
  if is_quoted(input) {
    input[1..input.len() - 1].to_string()
  } else {
    input.to_string()
  }
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
    element: Element::Sequence(SequenceDiagramElement::CheckedState {
      name,
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
