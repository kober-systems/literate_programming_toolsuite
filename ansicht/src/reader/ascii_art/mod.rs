use std::cmp::Ordering;

use crate::{
  ast::{Element as AstElement, ElementSpan, Slice, TextPosition},
  SequenceDiagramElement, AST,
};
mod tokenizer;
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
    let elements = parse_sequence_diagram(&ascii_elements, input);
    AST {
      content: input,
      elements,
    }
  }
}

struct Participant {
  id: usize,
  name: String,
  lifeline_col: usize,
}

fn parse_sequence_diagram(elements: &[Element], input: &str) -> Vec<ElementSpan> {
  let lines: Vec<&str> = input.lines().collect();
  let participants = extract_participants(elements, &lines);
  if participants.is_empty() {
    return vec![];
  }

  let mut result = extract_checked_states(elements, &lines, &participants);
  result.extend(extract_messages(elements, &lines, &participants));
  result.sort_by(|a, b| match (&a.position, &b.position) {
    (
      TextPosition::Slice(Slice { start: a_start, .. }),
      TextPosition::Slice(Slice { start: b_start, .. }),
    ) => a_start.cmp(&b_start),
    _ => Ordering::Equal,
  });
  result
}

fn extract_participants(elements: &[Element], lines: &[&str]) -> Vec<Participant> {
  let mut participants: Vec<Participant> = elements
    .iter()
    .filter_map(|element| participant_from_block(element, lines))
    .collect();

  participants.sort_by_key(|p| p.lifeline_col);
  participants
}

fn participant_from_block(element: &Element, lines: &[&str]) -> Option<Participant> {
  let Element::Block {
    id,
    inner_elements,
    border,
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
    Some(Participant {
      id: *id,
      name,
      lifeline_col,
    })
  }
}

fn extract_checked_states(
  elements: &[Element],
  lines: &[&str],
  participants: &[Participant],
) -> Vec<ElementSpan> {
  let result: Vec<ElementSpan> = elements
    .iter()
    .filter_map(|element| {
      let Element::Block {
        inner_elements,
        border: _,
        ..
      } = element
      else {
        eprintln!("element {:?} not supported", element);
        return None;
      };

      if !is_checked_state_block(element, lines) {
        return None;
      }

      let bounds = element.get_bounds();
      let name_line = inner_elements.iter().find_map(|element| match element {
        Element::Text { tokens, .. } => tokens.first().and_then(|token| match token {
          Token::Text { line, .. } => Some(*line),
          _ => None,
        }),
        _ => None,
      })?;

      let name = text_between(lines, name_line, bounds.start.column + 1, bounds.end.column - 1)
        .trim()
        .to_string();

      if name.is_empty() {
        return None;
      }

      Some(ElementSpan {
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
      })
    })
    .collect();
  result
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

fn extract_messages(
  elements: &[Element],
  lines: &[&str],
  participants: &[Participant],
) -> Vec<ElementSpan> {
  let mut result = vec![];
  let mut pending_text: Option<String> = None;

  for element in elements {
    match element {
      Element::Text { tokens, .. } => {
        let Some(Token::Text {
          line,
          column_start,
          column_end,
        }) = tokens.first()
        else {
          continue;
        };

        let text = text_between(lines, *line, *column_start, *column_end)
          .trim()
          .to_string();
        if !text.is_empty() {
          pending_text = Some(text);
        }
      }
      Element::Connection {
        from, to, tokens, ..
      } if tokens
        .iter()
        .any(|token| matches!(token, Token::Arrow { .. }))
        && tokens
          .iter()
          .any(|token| matches!(token, Token::HLine { .. })) =>
      {
        let from = participants.iter().find(|p| p.id == *from);
        let to = participants.iter().find(|p| p.id == *to);

        if let (Some(from), Some(to)) = (from, to) {
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
        }
      }
      _ => {
        eprintln!("Not a message {:?}\n", element);
      }
    }
  }

  result
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

