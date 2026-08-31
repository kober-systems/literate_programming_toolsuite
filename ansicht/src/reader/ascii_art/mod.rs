use std::{cmp::Ordering, collections::HashMap};

use crate::{
  ast::{Element as AstElement, ElementSpan, Slice, TextPosition},
  SequenceDiagramElement, AST,
};
mod tokenizer;
pub use tokenizer::*;

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

#[derive(Debug, PartialEq)]
pub enum Element {
  Block {
    id: usize,
    inner_elements: Vec<Element>,
    border: Vec<Token>,
  },
  Connection {
    id: usize,
    from: usize,
    to: usize,
    /// e.g. attached text
    inner_elements: Vec<Element>,
    tokens: Vec<Token>,
  },
  Text {
    id: usize,
    tokens: Vec<Token>,
  },
  /// Tokens that could not be made sense of
  Unknown {
    id: usize,
    tokens: Vec<Token>,
  },
}

impl Element {
  pub fn get_bounds(&self) -> BoundingBox {
    use Element::*;

    match self {
      Block {
        id: _,
        inner_elements: _,
        border,
      } => {
        let first = border.first().unwrap().get_bounds();
        let last = border.last().unwrap().get_bounds();
        BoundingBox {
          start: first.start,
          end: last.end,
        }
      }
      Connection {
        id: _,
        from: _,
        to: _,
        inner_elements: _,
        tokens,
      } => {
        let first = tokens.first().unwrap().get_bounds();
        let last = tokens.last().unwrap().get_bounds();
        BoundingBox {
          start: first.start,
          end: last.end,
        }
      }
      Text { id: _, tokens } => {
        let first = tokens.first().unwrap().get_bounds();
        let last = tokens.last().unwrap().get_bounds();
        BoundingBox {
          start: first.start,
          end: last.end,
        }
      }
      Unknown { id: _, tokens } => {
        let first = tokens.first().unwrap().get_bounds();
        let last = tokens.last().unwrap().get_bounds();
        BoundingBox {
          start: first.start,
          end: last.end,
        }
      }
    }
  }

  pub fn is_inside_bounds_of(&self, element: &Element) -> bool {
    let outer_bounds = element.get_bounds();
    let inner_bounds = self.get_bounds();

    outer_bounds.start.line <= inner_bounds.start.line
      && outer_bounds.start.column <= inner_bounds.start.column
      && outer_bounds.end.line >= inner_bounds.end.line
      && outer_bounds.end.column >= inner_bounds.end.column
  }

  pub fn add_inner_element(&mut self, element: Element) {
    use Element::*;

    match self {
      Block {
        id: _,
        inner_elements,
        border: _,
      } => {
        inner_elements.push(element);
      }
      Connection {
        id: _,
        from: _,
        to: _,
        inner_elements,
        tokens: _,
      } => {
        inner_elements.push(element);
      }
      Text { id: _, tokens: _ } => {}    // TODO
      Unknown { id: _, tokens: _ } => {} // TODO
    }
  }
}

pub fn parse_elements(input: &str) -> Vec<Element> {
  elements_from_tokens(parse_tokens(input), input)
}

fn elements_from_tokens(input: Vec<Token>, text: &str) -> Vec<Element> {
  use Element::*;

  let all_tokens = input.clone();
  let mut possible_blocks: Vec<PartialElement> = vec![];
  let mut texts = vec![];
  let mut blocks = vec![];
  let mut next_id = 0;
  for token in input.into_iter() {
    match token {
      Token::Text {
        line: _,
        column_start: _,
        column_end: _,
      } => {
        texts.push(token);
      }
      token => {
        if possible_blocks.is_empty() {
          possible_blocks.push(PartialElement::new(token));
        } else {
          let mut token_used = false;
          possible_blocks = possible_blocks
            .into_iter()
            .filter_map(|mut started_block| {
              if started_block.can_continue_block(&token, text) {
                token_used = true;
                if started_block.add_token(token) {
                  blocks.push(Block {
                    id: next_id,
                    inner_elements: vec![],
                    border: started_block.tokens,
                  });
                  next_id += 1;
                  return None;
                }
                return Some(started_block);
              }
              Some(started_block)
            })
            .collect();

          // If token wasn't used to continue any existing block, start a new one
          if !token_used {
            possible_blocks.push(PartialElement::new(token));
          }
        }
      }
    }
  }

  let mut out = vec![];
  for text in texts.into_iter() {
    let text = Text {
      id: next_id,
      tokens: vec![text],
    };
    next_id += 1;

    let mut owning_block = None;
    for block in blocks.iter_mut() {
      if text.is_inside_bounds_of(block) {
        owning_block = Some(block);
        break;
      }
    }
    match owning_block {
      Some(block) => {
        block.add_inner_element(text);
      }
      None => {
        out.push(text);
      }
    }
  }
  let mut connections = connections_between_blocks(&all_tokens, &blocks, &mut next_id);

  out.append(&mut blocks);
  out.append(&mut connections);

  out.sort_by(|a, b| {
    let a = a.get_bounds();
    let b = b.get_bounds();
    a.start
      .line
      .cmp(&b.start.line)
      .then(a.start.column.cmp(&b.start.column))
  });

  out
}

fn connections_between_blocks(
  tokens: &[Token],
  blocks: &[Element],
  next_id: &mut usize,
) -> Vec<Element> {
  let mut connections = vec![];

  for token in tokens {
    let Token::VLine {
      column,
      line_start,
      line_end,
    } = token
    else {
      continue;
    };

    if blocks.iter().any(|block| match block {
      Element::Block { border, .. } => border.contains(token),
      _ => false,
    }) {
      continue;
    }

    let from = blocks.iter().find_map(|block| match block {
      Element::Block { id, border, .. }
        if border.contains(&Token::ConnectionSign {
          line: line_start - 1,
          column: *column,
        }) =>
      {
        Some(*id)
      }
      _ => None,
    });

    let to = blocks.iter().find_map(|block| match block {
      Element::Block { id, border, .. }
        if border.contains(&Token::ConnectionSign {
          line: line_end + 1,
          column: *column,
        }) =>
      {
        Some(*id)
      }
      _ => None,
    });

    if let (Some(from), Some(to)) = (from, to) {
      connections.push(Element::Connection {
        id: *next_id,
        from,
        to,
        inner_elements: vec![],
        tokens: vec![*token],
      });
      *next_id += 1;
    }
  }

  for token in tokens {
    let Token::Arrow { line, column } = token else {
      continue;
    };

    if let Some(hline) = tokens.iter().find_map(|token| match token {
      Token::HLine {
        line: hline_line,
        column_start,
        column_end,
      } if hline_line == line && *column_end + 1 == *column => {
        Some((*column_start, *column_end, *token))
      }
      _ => None,
    }) {
      let (column_start, column_end, hline) = hline;
      let from = lifeline_from_at(&connections, column_start - 1, *line);
      let to = lifeline_from_at(&connections, column_end + 2, *line);

      if let (Some(from), Some(to)) = (from, to) {
        connections.push(Element::Connection {
          id: *next_id,
          from,
          to,
          inner_elements: vec![],
          tokens: vec![hline, *token],
        });
        *next_id += 1;
      }
    } else if let Some(hline) = tokens.iter().find_map(|token| match token {
      Token::HLine {
        line: hline_line,
        column_start,
        column_end,
      } if hline_line == line && *column_start == *column + 1 => {
        Some((*column_start, *column_end, *token))
      }
      _ => None,
    }) {
      let (column_start, column_end, hline) = hline;
      let from = lifeline_from_at(&connections, column_end + 1, *line);
      let to = lifeline_from_at(&connections, column_start - 2, *line);

      if let (Some(from), Some(to)) = (from, to) {
        connections.push(Element::Connection {
          id: *next_id,
          from,
          to,
          inner_elements: vec![],
          tokens: vec![*token, hline],
        });
        *next_id += 1;
      }
    }
  }

  connections
}

fn lifeline_from_at(connections: &[Element], column: usize, line: usize) -> Option<usize> {
  connections.iter().find_map(|connection| match connection {
    Element::Connection { from, tokens, .. }
      if tokens.iter().any(|token| {
        matches!(token, Token::VLine { column: vline_column, line_start, line_end }
          if *vline_column == column && *line_start <= line && *line_end >= line)
      }) =>
    {
      Some(*from)
    }
    _ => None,
  })
}

struct PartialElement {
  clock_cycle_end: Coordinate,
  counter_clock_cycle_end: Coordinate,
  tokens: Vec<Token>,
}

impl PartialElement {
  fn new(token: Token) -> Self {
    let BoundingBox { start, end } = token.get_bounds();
    Self {
      tokens: vec![token],
      counter_clock_cycle_end: start,
      clock_cycle_end: end,
    }
  }

  fn can_continue_block(&self, next_token: &Token, _text: &str) -> bool {
    use Token::*;

    match next_token {
      HLine {
        line,
        column_start,
        column_end: _,
      } => {
        if self.clock_cycle_end.line == *line && self.clock_cycle_end.column + 1 == *column_start {
          true
        } else if self.counter_clock_cycle_end.line == *line
          && self.counter_clock_cycle_end.column + 1 == *column_start
        {
          true
        } else if self.tokens.iter().any(|token| {
          matches!(token, ConnectionSign { line: sign_line, column } if sign_line == line && *column + 1 == *column_start)
        }) {
          true
        } else {
          false
        }
      }
      ConnectionSign { line, column } => {
        if self.clock_cycle_end.line == *line && self.clock_cycle_end.column + 1 == *column {
          true
        } else if self.clock_cycle_end.line + 1 == *line && self.clock_cycle_end.column == *column {
          true
        } else if self.counter_clock_cycle_end.line + 1 == *line
          && self.counter_clock_cycle_end.column == *column
        {
          true
        } else if self.tokens.iter().any(|token| {
          matches!(token, HLine { line: hline_line, column_end, .. } if hline_line == line && *column_end + 1 == *column)
        }) {
          true
        } else {
          false
        }
      }
      VLine {
        column,
        line_start,
        line_end: _,
      } => {
        if self.clock_cycle_end.column == *column && self.clock_cycle_end.line + 1 == *line_start {
          true
        } else if self.counter_clock_cycle_end.column == *column
          && self.counter_clock_cycle_end.line + 1 == *line_start
        {
          true
        } else {
          false
        }
      }
      _ => false,
    }
  }

  /// Add a token to the partial element
  ///
  /// return: is the elment closed by this token
  fn add_token(&mut self, token: Token) -> bool {
    let BoundingBox { start: _, end } = token.get_bounds();
    self.tokens.push(token);
    if end.line >= self.clock_cycle_end.line && end.column >= self.clock_cycle_end.column {
      self.clock_cycle_end = end;
    } else {
      self.counter_clock_cycle_end = end;
    }

    self.clock_cycle_end.line == self.counter_clock_cycle_end.line
      && self.clock_cycle_end.column == self.counter_clock_cycle_end.column + 1
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
  let mut result: Vec<ElementSpan> = elements
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

