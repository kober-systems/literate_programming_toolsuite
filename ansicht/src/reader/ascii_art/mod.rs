use std::cmp::Ordering;

use crate::{
  ast::{Element as AstElement, ElementSpan, Slice, TextPosition},
  SequenceDiagramElement, AST,
};
pub mod tokenizer;
pub use tokenizer::Token;
mod elements;
pub use elements::*;

pub struct AsciiArtReader {}

impl AsciiArtReader {
  pub fn new() -> Self {
    Self {}
  }

  pub fn parse<'a>(&self, input: &'a str) -> AST<'a> {
    let ascii_elements = parse_elements(input);
    let elements = parse_sequence_diagram(ascii_elements, input);
    AST {
      content: input,
      elements,
    }
  }
}

struct Participant {
  name: String,
  lifeline_col: usize,
}

fn parse_sequence_diagram(elements: Vec<Element>, input: &str) -> Vec<ElementSpan> {
  let lines: Vec<&str> = input.lines().collect();
  let (elements, participants) = extract_participants(elements, &lines);
  if participants.is_empty() {
    return vec![];
  }

  let (elements, mut result) = extract_checked_states(elements, &lines, &participants);
  let (remaining, messages) = extract_messages(elements, &lines, input, &participants);
  result.extend(messages);

  for element in remaining {
    eprintln!("element not supported:\n{}\n", element.render_ascii(input));
  }

  result.sort_by(|a, b| match (&a.position, &b.position) {
    (
      TextPosition::Slice(Slice { start: a_start, .. }),
      TextPosition::Slice(Slice { start: b_start, .. }),
    ) => a_start.cmp(&b_start),
    _ => Ordering::Equal,
  });
  result
}

fn extract_participants(
  elements: Vec<Element>,
  lines: &[&str],
) -> (Vec<Element>, Vec<Participant>) {
  let mut remaining = vec![];
  let mut participants = vec![];

  for element in elements {
    if let Some(participant) = participant_from_block(&element, lines) {
      participants.push(participant);
    } else {
      remaining.push(element);
    }
  }

  participants.sort_by_key(|p| p.lifeline_col);
  (remaining, participants)
}

fn participant_from_block(element: &Element, lines: &[&str]) -> Option<Participant> {
  let Element::Block {
    inner_elements,
    border,
    ..
  } = element
  else {
    return None;
  };

  if is_checked_state_block(element, lines) {
    return None;
  }

  let bounds = element.get_bounds();
  let lifeline_col = border.iter().find_map(|token| match token {
    Token::ConnectionSign { line, column }
      if *line == bounds.end.line
        && *column > bounds.start.column
        && *column < bounds.end.column =>
    {
      Some(*column)
    }
    _ => None,
  })?;

  let name_line = inner_elements.iter().find_map(|element| match element {
    Element::Text { tokens, .. } => tokens.first().and_then(|token| match token {
      Token::Text { line, .. } => Some(*line),
      _ => None,
    }),
    _ => None,
  })?;

  let name = text_between(
    lines,
    name_line,
    bounds.start.column + 1,
    bounds.end.column - 1,
  )
  .trim()
  .to_string();

  if name.is_empty() {
    None
  } else {
    Some(Participant { name, lifeline_col })
  }
}

fn extract_checked_states(
  elements: Vec<Element>,
  lines: &[&str],
  participants: &[Participant],
) -> (Vec<Element>, Vec<ElementSpan>) {
  let mut remaining = vec![];
  let mut result = vec![];

  for element in elements {
    let Element::Block { inner_elements, .. } = &element else {
      remaining.push(element);
      continue;
    };

    if !is_checked_state_block(&element, lines) {
      remaining.push(element);
      continue;
    }

    let bounds = element.get_bounds();
    let name_line = inner_elements.iter().find_map(|element| match element {
      Element::Text { tokens, .. } => tokens.first().and_then(|token| match token {
        Token::Text { line, .. } => Some(*line),
        _ => None,
      }),
      _ => None,
    });

    let Some(name_line) = name_line else {
      continue;
    };
    let name = text_between(lines, name_line, bounds.start.column + 1, bounds.end.column - 1)
      .trim()
      .to_string();
    if name.is_empty() {
      continue;
    }

    result.push(ElementSpan {
      source: None,
      position: TextPosition::Slice(Slice {
        start: bounds.start.line,
        end: bounds.end.line,
      }),
      element: AstElement::Sequence(SequenceDiagramElement::CheckedState {
        name,
        participants: participants.iter().map(|p| p.name.clone()).collect(),
      }),
      children: vec![],
      attrs: vec![],
    });
  }

  (remaining, result)
}

fn is_checked_state_block(element: &Element, lines: &[&str]) -> bool {
  let bounds = element.get_bounds();
  let top = lines.get(bounds.start.line);
  let bottom = lines.get(bounds.end.line);

  let top_left = top.and_then(|line| line.chars().nth(bounds.start.column));
  let top_right = top.and_then(|line| line.chars().nth(bounds.end.column));
  let bottom_left = bottom.and_then(|line| line.chars().nth(bounds.start.column));
  let bottom_right = bottom.and_then(|line| line.chars().nth(bounds.end.column));

  matches!(top_left, Some('╔'))
    && matches!(top_right, Some('╗'))
    && matches!(bottom_left, Some('╚'))
    && matches!(bottom_right, Some('╝'))
}

fn participant_at_lifeline_col<'a>(
  participants: &'a [Participant],
  column: usize,
) -> Option<&'a Participant> {
  participants.iter().find(|participant| participant.lifeline_col == column)
}

fn resolve_participants_by_columns<'a>(
  participants: &'a [Participant],
  from_column: usize,
  to_column: usize,
) -> Option<(&'a Participant, &'a Participant)> {
  let from = participant_at_lifeline_col(participants, from_column)?;
  let to = participant_at_lifeline_col(participants, to_column)?;
  Some((from, to))
}

fn lifeline_column_for_connection(
  elements: &[Element],
  connection_id: usize,
) -> Option<usize> {
  let Element::Connection { tokens, .. } = elements.iter().find(|element| {
    matches!(element, Element::Connection { id, .. } if *id == connection_id)
  })?
  else {
    return None;
  };

  if tokens.is_empty()
    || !tokens
      .iter()
      .all(|token| matches!(token, Token::VLine { .. }))
  {
    return None;
  }

  let mut columns = tokens.iter().map(|token| match token {
    Token::VLine { column, .. } => *column,
    _ => unreachable!(),
  });

  let column = columns.next()?;

  columns.all(|other_column| other_column == column).then_some(column)
}

fn resolve_message_participants_by_connection_ids<'a>(
  elements: &[Element],
  participants: &'a [Participant],
  from: usize,
  to: usize,
) -> Option<(&'a Participant, &'a Participant)> {
  let from_column = lifeline_column_for_connection(elements, from)?;
  let to_column = lifeline_column_for_connection(elements, to)?;

  resolve_participants_by_columns(participants, from_column, to_column)
}

fn resolve_message_participants<'a>(
  elements: &[Element],
  participants: &'a [Participant],
  from: usize,
  to: usize,
) -> Option<(&'a Participant, &'a Participant)> {
  resolve_message_participants_by_connection_ids(elements, participants, from, to)
}

fn extract_messages(
  elements: Vec<Element>,
  lines: &[&str],
  input: &str,
  participants: &[Participant],
) -> (Vec<Element>, Vec<ElementSpan>) {
  let mut result = vec![];
  let mut pending_text: Option<String> = None;
  let mut consumed_ids = vec![];

  for element in &elements {
    match element {
      Element::Text { .. } | Element::Connection { .. } => {
        consumed_ids.push(element_id(element));
      }
      _ => {}
    }

    match element {
      Element::Text { tokens, .. } => {
        let Some(first) = tokens.first() else {
          continue;
        };
        let Some(last) = tokens.last() else {
          continue;
        };
        let first = first.get_bounds();
        let last = last.get_bounds();

        let text = text_between(lines, first.start.line, first.start.column, last.end.column)
          .trim()
          .to_string();
        if !text.is_empty() {
          pending_text = Some(text);
        }
      }
      Element::Connection { from, to, tokens, .. }
        if tokens.iter().any(|token| matches!(token, Token::Arrow { .. }))
          && tokens.iter().any(|token| matches!(token, Token::HLine { .. })) =>
      {
        let resolved = resolve_message_participants(&elements, participants, *from, *to);

        if let Some((from, to)) = resolved {
          let bounds = element.get_bounds();
          let message = pending_text.take().unwrap_or_default();
          result.push(ElementSpan {
            source: None,
            position: TextPosition::Slice(Slice {
              start: bounds.start.line,
              end: bounds.end.line,
            }),
            element: AstElement::Sequence(SequenceDiagramElement::Message {
              from: from.name.clone(),
              to: to.name.clone(),
              message,
              meta: None,
            }),
            children: vec![],
            attrs: vec![],
          });
        } else {
          eprintln!("Connection not found:\n{:?}\n{}\n", element, element.render_ascii(input));
        }
      }
      _ => {}
    }
  }

  let remaining = elements
    .into_iter()
    .filter(|element| !consumed_ids.contains(&element_id(element)))
    .collect();

  (remaining, result)
}

fn element_id(element: &Element) -> usize {
  match element {
    Element::Block { id, .. }
    | Element::Connection { id, .. }
    | Element::Text { id, .. }
    | Element::Unknown { id, .. } => *id,
  }
}

fn text_between(lines: &[&str], line: usize, column_start: usize, column_end: usize) -> String {
  lines
    .get(line)
    .map(|line| {
      line
        .chars()
        .skip(column_start)
        .take(column_end - column_start + 1)
        .collect()
    })
    .unwrap_or_default()
}

