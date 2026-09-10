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
  bounds: BoundingBox,
}

fn parse_sequence_diagram(elements: &[Element], input: &str) -> Vec<ElementSpan> {
  let lines: Vec<&str> = input.lines().collect();
  let participants = extract_participants(elements, &lines);
  if participants.is_empty() {
    return vec![];
  }

  let mut result = extract_checked_states(elements, &lines, input, &participants);
  result.extend(extract_messages(elements, &lines, input, &participants));
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
      bounds,
    })
  }
}

fn extract_checked_states(
  elements: &[Element],
  lines: &[&str],
  input: &str,
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
        eprintln!("element not supported:\n{}\n", element.render_ascii(input));
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

fn participant_at_lifeline_col<'a>(
  participants: &'a [Participant],
  column: usize,
) -> Option<&'a Participant> {
  participants.iter().find(|participant| participant.lifeline_col == column)
}

fn participant_inside_bounds(participant: &Participant, bounds: &BoundingBox) -> bool {
  bounds.start.column <= participant.lifeline_col && participant.lifeline_col <= bounds.end.column
}

fn scoped_participants_by_checked_state<'a>(
  participants: &'a [Participant],
  bounds: &BoundingBox,
) -> Vec<&'a Participant> {
  participants
    .iter()
    .filter(|participant| participant_inside_bounds(participant, bounds))
    .collect()
}

fn resolve_participants_by_ids<'a>(
  participants: &'a [Participant],
  from: usize,
  to: usize,
) -> Option<(&'a Participant, &'a Participant)> {
  let from = participants.iter().find(|participant| participant.id == from)?;
  let to = participants.iter().find(|participant| participant.id == to)?;
  Some((from, to))
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

fn checked_state_bounds_for_element(
  element: &Element,
  elements: &[Element],
  lines: &[&str],
) -> Option<BoundingBox> {
  elements
    .iter()
    .filter(|candidate| {
      matches!(candidate, Element::Block { .. })
        && is_checked_state_block(candidate, lines)
        && element.is_inside_bounds_of(candidate)
    })
    .map(|candidate| candidate.get_bounds())
    .min_by_key(|bounds| {
      (
        bounds.end.line - bounds.start.line,
        bounds.end.column - bounds.start.column,
      )
    })
}

fn connection_endpoint_columns(tokens: &[Token]) -> Option<(usize, usize)> {
  let (line, column_start, column_end) = tokens.iter().find_map(|token| match token {
    Token::HLine {
      line,
      column_start,
      column_end,
    } => Some((*line, *column_start, *column_end)),
    _ => None,
  })?;

  let arrow_column = tokens.iter().find_map(|token| match token {
    Token::Arrow { line: arrow_line, column } if *arrow_line == line => Some(*column),
    _ => None,
  })?;

  if column_end + 1 == arrow_column {
    Some((column_start.checked_sub(1)?, column_end.checked_add(2)?))
  } else if column_start == arrow_column + 1 {
    Some((column_end.checked_add(1)?, column_start.checked_sub(2)?))
  } else {
    None
  }
}

fn resolve_message_participants<'a>(
  element: &Element,
  elements: &[Element],
  lines: &[&str],
  participants: &'a [Participant],
  from: usize,
  to: usize,
  tokens: &[Token],
) -> Option<(&'a Participant, &'a Participant)> {
  resolve_message_participants_in_checked_state(element, elements, lines, participants, tokens)
    .or_else(|| resolve_message_participants_by_id_or_columns(participants, from, to, tokens))
}

fn resolve_message_participants_in_checked_state<'a>(
  element: &Element,
  elements: &[Element],
  lines: &[&str],
  participants: &'a [Participant],
  tokens: &[Token],
) -> Option<(&'a Participant, &'a Participant)> {
  let bounds = checked_state_bounds_for_element(element, elements, lines)?;
  let (from_column, to_column) = connection_endpoint_columns(tokens)?;
  let scoped_participants = scoped_participants_by_checked_state(participants, &bounds);
  let from = scoped_participants
    .iter()
    .copied()
    .find(|participant| participant.lifeline_col == from_column)?;
  let to = scoped_participants
    .iter()
    .copied()
    .find(|participant| participant.lifeline_col == to_column)?;
  Some((from, to))
}

fn resolve_message_participants_by_id_or_columns<'a>(
  participants: &'a [Participant],
  from: usize,
  to: usize,
  tokens: &[Token],
) -> Option<(&'a Participant, &'a Participant)> {
  resolve_participants_by_ids(participants, from, to).or_else(|| {
    connection_endpoint_columns(tokens).and_then(|(from_column, to_column)| {
      resolve_participants_by_columns(participants, from_column, to_column)
    })
  })
}

fn extract_messages(
  elements: &[Element],
  lines: &[&str],
  input: &str,
  participants: &[Participant],
) -> Vec<ElementSpan> {
  let mut result = vec![];
  let mut pending_text: Option<String> = None;

  for element in elements {
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
        let resolved = resolve_message_participants(
          element,
          elements,
          lines,
          participants,
          *from,
          *to,
          tokens,
        );

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
      _ => {
        eprintln!("Not a message:\n{:?}\n{}\n", element, element.render_ascii(input));
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

