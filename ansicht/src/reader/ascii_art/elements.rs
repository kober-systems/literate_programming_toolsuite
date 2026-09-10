pub use super::tokenizer::*;

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

  pub fn render_ascii(&self, source: &str) -> String {
    let bounds = self.render_bounds();
    let lines: Vec<&str> = source.lines().collect();

    (bounds.start.line..=bounds.end.line)
      .map(|line_idx| {
        lines
          .get(line_idx)
          .map(|line| {
            line
              .chars()
              .skip(bounds.start.column)
              .take(bounds.end.column - bounds.start.column + 1)
              .collect::<String>()
          })
          .unwrap_or_default()
      })
      .collect::<Vec<_>>()
      .join("\n")
  }

  fn render_bounds(&self) -> BoundingBox {
    let mut bounds = self.get_bounds();

    match self {
      Element::Block { inner_elements, .. } | Element::Connection { inner_elements, .. } => {
        for inner in inner_elements {
          bounds = union_bounds(bounds, inner.render_bounds());
        }
      }
      _ => {}
    }

    bounds
  }
}

pub fn parse_elements(input: &str) -> Vec<Element> {
  elements_from_tokens(parse_tokens(input), input)
}

fn elements_from_tokens(input: Vec<Token>, text: &str) -> Vec<Element> {
  use Element::*;

  let all_tokens = input.clone();
  let mut possible_blocks: Vec<PartialElement> = vec![];
  let mut texts: Vec<Vec<Token>> = vec![];
  let mut blocks = vec![];
  let mut next_id = 0;

  for (index, token) in input.into_iter().enumerate() {
    if matches!(token, Token::Text { .. })
      || is_hline_embedded_in_text(&all_tokens, index)
    {
      if let Some(previous) = texts.last_mut() {
        if tokens_are_adjacent(previous.last().unwrap(), &token) {
          previous.push(token);
          continue;
        }
      }
      texts.push(vec![token]);
      continue;
    }

    if possible_blocks.is_empty() {
      possible_blocks.push(PartialElement::new(token));
    } else {
      let mut token_used = false;
      possible_blocks = possible_blocks
        .into_iter()
        .filter_map(|mut started_block| {
          if !started_block.can_continue_block(&token, text) {
            // No block can be continued -> it must be a new one
            return Some(started_block);
          }

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
          Some(started_block)
        })
        .collect();

      // If token wasn't used to continue any existing block, start a new one
      if !token_used {
        possible_blocks.push(PartialElement::new(token));
      }
    }
  }

  let mut out = vec![];

  // Find if the texts belong into a block
  for text_tokens in texts.into_iter() {
    let text = Text {
      id: next_id,
      tokens: text_tokens,
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

  // Keep every structural token that was not consumed by a recognized
  // element. Previously these tokens, including incomplete block candidates,
  // disappeared silently.
  for token in all_tokens {
    if !element_owns_token(&out, &token) {
      out.push(Unknown {
        id: next_id,
        tokens: vec![token],
      });
      next_id += 1;
    }
  }

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

  connections.extend(
    tokens
      .iter()
      .filter_map(|token| vline_connection_between_blocks(token, tokens, blocks, next_id)),
  );

  // TODO this should be removed. We do not have lifelines here
  let arrow_connections: Vec<_> = tokens
    .iter()
    .filter_map(|token| arrow_connection_between_lifelines(token, tokens, &connections, next_id))
    .collect();
  connections.extend(arrow_connections);

  connections
}

// TODO We need an equvalent hline_connection_between_blocks
fn vline_connection_between_blocks(
  token: &Token,
  all_tokens: &[Token],
  blocks: &[Element],
  next_id: &mut usize,
) -> Option<Element> {
  let Token::VLine {
    column,
    line_start,
    ..
  } = token
  else {
    return None;
  };

  if blocks.iter().any(|block| match block {
    Element::Block { border, .. } => border.contains(token),
    _ => false,
  }) {
    return None;
  }

  // Only the first fragment can start a connection. Later fragments will
  // have no connection sign directly above them and are therefore ignored.
  let from = element_with_connection_sign(blocks, line_start - 1, *column)?;
  let mut lifeline_tokens = vec![*token];
  let mut current_end = token.get_bounds().end.line;

  let mut fragments: Vec<Token> = all_tokens
    .iter()
    .copied()
    .filter(|candidate| {
      matches!(candidate, Token::VLine { column: candidate_column, .. } if candidate_column == column)
    })
    .collect();
  fragments.sort_by_key(|candidate| candidate.get_bounds().start.line);

  for fragment in fragments {
    let Token::VLine {
      line_start: fragment_start,
      line_end: fragment_end,
      ..
    } = fragment
    else {
      unreachable!();
    };

    if fragment_start <= current_end {
      continue;
    }

    let gap_is_crossed = (current_end + 1..fragment_start).all(|line| {
      all_tokens.iter().any(|candidate| match candidate {
        Token::HLine {
          line: candidate_line,
          column_start,
          column_end,
        } => {
          *candidate_line == line && *column_start <= *column && *column <= *column_end
        }
        Token::Arrow {
          line: candidate_line,
          column: candidate_column,
        } => *candidate_line == line && *candidate_column == *column,
        Token::Text {
          line: candidate_line,
          column_start,
          column_end,
        } => {
          *candidate_line == line
            && *column_start <= *column
            && *column <= *column_end
        }
        _ => false,
      })
    });

    if !gap_is_crossed {
      break;
    }

    lifeline_tokens.push(fragment);
    current_end = fragment_end;
  }

  let to = element_with_connection_sign(blocks, current_end + 1, *column)?;

  let connection = Element::Connection {
    id: *next_id,
    from,
    to,
    inner_elements: vec![],
    tokens: lifeline_tokens,
  };
  *next_id += 1;
  Some(connection)
}

enum ArrowDirection {
  Forward,
  Reverse,
}

struct ArrowHLine {
  column_start: usize,
  column_end: usize,
  token: Token,
  direction: ArrowDirection,
}

fn arrow_hline_at(tokens: &[Token], line: usize, column: usize) -> Option<ArrowHLine> {
  tokens.iter().find_map(|token| match token {
    Token::HLine {
      line: hline_line,
      column_start,
      column_end,
    } if hline_line == &line && *column_end + 1 == column => Some(ArrowHLine {
      column_start: *column_start,
      column_end: *column_end,
      token: *token,
      direction: ArrowDirection::Forward,
    }),
    Token::HLine {
      line: hline_line,
      column_start,
      column_end,
    } if hline_line == &line && *column_start == column + 1 => Some(ArrowHLine {
      column_start: *column_start,
      column_end: *column_end,
      token: *token,
      direction: ArrowDirection::Reverse,
    }),
    _ => None,
  })
}

// TODO entfernen
fn arrow_connection_between_lifelines(
  token: &Token,
  tokens: &[Token],
  connections: &[Element],
  next_id: &mut usize,
) -> Option<Element> {
  let Token::Arrow { line, column } = token else {
    return None;
  };

  let ArrowHLine {
    column_start,
    column_end,
    token: hline,
    direction,
  } = arrow_hline_at(tokens, *line, *column)?;

  let (from, to, tokens) = match direction {
    ArrowDirection::Forward => (
      lifeline_connection_at(connections, column_start - 1, *line),
      lifeline_connection_at(connections, column_end + 2, *line),
      vec![hline, *token],
    ),
    ArrowDirection::Reverse => (
      lifeline_connection_at(connections, column_end + 1, *line),
      lifeline_connection_at(connections, column_start - 2, *line),
      vec![*token, hline],
    ),
  };

  match (from, to) {
    (Some(from), Some(to)) => {
      let connection = Element::Connection {
        id: *next_id,
        from,
        to,
        inner_elements: vec![],
        tokens,
      };
      *next_id += 1;
      Some(connection)
    }
    _ => None,
  }
}

fn element_with_connection_sign(
  elements: &[Element],
  line: usize,
  column: usize,
) -> Option<usize> {
  elements.iter().find_map(|element| match element {
    Element::Block { id, border, .. }
      if border.contains(&Token::ConnectionSign { line, column }) =>
    {
      Some(*id)
    }
    Element::Connection { id, tokens, .. }
      if tokens.iter().any(|token| {
        matches!(token, Token::ConnectionSign { line: token_line, column: token_column }
          if *token_line == line && *token_column == column)
      }) =>
    {
      Some(*id)
    }
    _ => None,
  })
}

fn lifeline_connection_at(connections: &[Element], column: usize, line: usize) -> Option<usize> {
  connections.iter().find_map(|connection| match connection {
    Element::Connection { id, tokens, .. }
      if tokens.iter().any(|token| {
        matches!(token, Token::VLine { column: vline_column, line_start, line_end }
          if *vline_column == column && *line_start <= line && *line_end >= line)
      }) =>
    {
      Some(*id)
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
      && self.tokens.iter().any(|token| matches!(token, Token::VLine { .. }))
  }
}

fn element_owns_token(elements: &[Element], token: &Token) -> bool {
  elements.iter().any(|element| match element {
    Element::Block {
      inner_elements,
      border,
      ..
    } => border.contains(token) || element_owns_token(inner_elements, token),
    Element::Connection {
      inner_elements,
      tokens,
      ..
    } => tokens.contains(token) || element_owns_token(inner_elements, token),
    Element::Text { tokens, .. } | Element::Unknown { tokens, .. } => tokens.contains(token),
  })
}

fn is_hline_embedded_in_text(tokens: &[Token], index: usize) -> bool {
  let Token::HLine {
    line,
    column_start,
    column_end,
  } = tokens[index]
  else {
    return false;
  };

  let Some(previous) = index.checked_sub(1).and_then(|i| tokens.get(i)) else {
    return false;
  };
  let Some(next) = tokens.get(index + 1) else {
    return false;
  };

  matches!(previous, Token::Text { line: previous_line, column_end: previous_end, .. }
    if *previous_line == line && *previous_end + 1 == column_start)
    && matches!(next, Token::Text { line: next_line, column_start: next_start, .. }
      if *next_line == line && column_end + 1 == *next_start)
}

fn tokens_are_adjacent(previous: &Token, next: &Token) -> bool {
  let previous = previous.get_bounds();
  let next = next.get_bounds();
  previous.end.line == next.start.line && previous.end.column + 1 == next.start.column
}

fn union_bounds(a: BoundingBox, b: BoundingBox) -> BoundingBox {
  BoundingBox {
    start: Coordinate {
      line: a.start.line.min(b.start.line),
      column: a.start.column.min(b.start.column),
    },
    end: Coordinate {
      line: a.end.line.max(b.end.line),
      column: a.end.column.max(b.end.column),
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  // parse elements

  #[test]
  fn empty_string_to_elements() {
    let elements = parse_elements("");
    assert_eq!(elements, vec![]);

    let elements = parse_elements(
      r"

        ",
    );
    assert_eq!(elements, vec![]);
  }

  #[test]
  fn simple_text_to_elements() {
    use Token::*;
    let elements = parse_elements("Some simple Text");
    assert_eq!(
      elements,
      vec![Element::Text {
        id: 0,
        tokens: vec![Text {
          line: 0,
          column_start: 0,
          column_end: 15,
        }]
      }]
    );

    let elements = parse_elements(
      r"
        Some Text on another line

        ",
    );
    assert_eq!(
      elements,
      vec![Element::Text {
        id: 0,
        tokens: vec![Text {
          line: 1,
          column_start: 8,
          column_end: 32,
        }]
      }]
    );
  }

  #[test]
  fn sequence_diagram_with_message_to_elements() {
    use Token::*;
    let elements = parse_elements(
      r"
    ┌──────┐     ┌──────┐
    │Client│     │Target│
    └──┬───┘     └──┬───┘
       │            │
       │  message   │
       │───────────>│
       │            │
    ┌──┴───┐     ┌──┴───┐
    │Client│     │Target│
    └──────┘     └──────┘
  ",
    );

    assert_eq!(
      elements,
      vec![
        Element::Block {
          id: 0,
          inner_elements: vec![Element::Text {
            id: 4,
            tokens: vec![Text {
              line: 2,
              column_start: 5,
              column_end: 10
            }],
          }],
          border: vec![
            ConnectionSign { line: 1, column: 4 },
            HLine {
              line: 1,
              column_start: 5,
              column_end: 10
            },
            ConnectionSign {
              line: 1,
              column: 11
            },
            VLine {
              column: 4,
              line_start: 2,
              line_end: 2
            },
            VLine {
              column: 11,
              line_start: 2,
              line_end: 2
            },
            ConnectionSign { line: 3, column: 4 },
            HLine {
              line: 3,
              column_start: 5,
              column_end: 6
            },
            ConnectionSign { line: 3, column: 7 },
            HLine {
              line: 3,
              column_start: 8,
              column_end: 10
            },
            ConnectionSign {
              line: 3,
              column: 11
            },
          ],
        },
        Element::Block {
          id: 1,
          inner_elements: vec![Element::Text {
            id: 5,
            tokens: vec![Text {
              line: 2,
              column_start: 18,
              column_end: 23
            }],
          }],
          border: vec![
            ConnectionSign {
              line: 1,
              column: 17
            },
            HLine {
              line: 1,
              column_start: 18,
              column_end: 23
            },
            ConnectionSign {
              line: 1,
              column: 24
            },
            VLine {
              column: 17,
              line_start: 2,
              line_end: 2
            },
            VLine {
              column: 24,
              line_start: 2,
              line_end: 2
            },
            ConnectionSign {
              line: 3,
              column: 17
            },
            HLine {
              line: 3,
              column_start: 18,
              column_end: 19
            },
            ConnectionSign {
              line: 3,
              column: 20
            },
            HLine {
              line: 3,
              column_start: 21,
              column_end: 23
            },
            ConnectionSign {
              line: 3,
              column: 24
            },
          ],
        },
        Element::Connection {
          id: 9,
          from: 0,
          to: 2,
          inner_elements: vec![],
          tokens: vec![VLine {
            column: 7,
            line_start: 4,
            line_end: 7,
          }],
        },
        Element::Connection {
          id: 10,
          from: 1,
          to: 3,
          inner_elements: vec![],
          tokens: vec![VLine {
            column: 20,
            line_start: 4,
            line_end: 7,
          }],
        },
        Element::Text {
          id: 6,
          tokens: vec![Text {
            line: 5,
            column_start: 10,
            column_end: 16
          }],
        },
        Element::Connection {
          id: 11,
          from: 9,
          to: 10,
          inner_elements: vec![],
          tokens: vec![
            HLine {
              line: 6,
              column_start: 8,
              column_end: 18
            },
            Arrow {
              line: 6,
              column: 19
            },
          ],
        },
        Element::Block {
          id: 2,
          inner_elements: vec![Element::Text {
            id: 7,
            tokens: vec![Text {
              line: 9,
              column_start: 5,
              column_end: 10
            }],
          }],
          border: vec![
            ConnectionSign { line: 8, column: 4 },
            HLine {
              line: 8,
              column_start: 5,
              column_end: 6
            },
            ConnectionSign { line: 8, column: 7 },
            HLine {
              line: 8,
              column_start: 8,
              column_end: 10
            },
            ConnectionSign {
              line: 8,
              column: 11
            },
            VLine {
              column: 4,
              line_start: 9,
              line_end: 9
            },
            VLine {
              column: 11,
              line_start: 9,
              line_end: 9
            },
            ConnectionSign {
              line: 10,
              column: 4
            },
            HLine {
              line: 10,
              column_start: 5,
              column_end: 10
            },
            ConnectionSign {
              line: 10,
              column: 11
            },
          ],
        },
        Element::Block {
          id: 3,
          inner_elements: vec![Element::Text {
            id: 8,
            tokens: vec![Text {
              line: 9,
              column_start: 18,
              column_end: 23
            }],
          }],
          border: vec![
            ConnectionSign {
              line: 8,
              column: 17
            },
            HLine {
              line: 8,
              column_start: 18,
              column_end: 19
            },
            ConnectionSign {
              line: 8,
              column: 20
            },
            HLine {
              line: 8,
              column_start: 21,
              column_end: 23
            },
            ConnectionSign {
              line: 8,
              column: 24
            },
            VLine {
              column: 17,
              line_start: 9,
              line_end: 9
            },
            VLine {
              column: 24,
              line_start: 9,
              line_end: 9
            },
            ConnectionSign {
              line: 10,
              column: 17
            },
            HLine {
              line: 10,
              column_start: 18,
              column_end: 23
            },
            ConnectionSign {
              line: 10,
              column: 24
            },
          ],
        },
      ]
    );
  }

  #[test]
  fn boxes_with_lifeline_connector_to_elements() {
    use Token::*;
    let elements = parse_elements(
      r"
    ┌──────┐
    │Client│
    └──┬───┘
       │
    ┌──┴───┐
    │Client│
    └──────┘
  ",
    );

    assert_eq!(
      elements,
      vec![
        Element::Block {
          id: 0,
          inner_elements: vec![Element::Text {
            id: 2,
            tokens: vec![Text {
              line: 2,
              column_start: 5,
              column_end: 10
            },],
          }],
          border: vec![
            ConnectionSign { line: 1, column: 4 },
            HLine {
              line: 1,
              column_start: 5,
              column_end: 10
            },
            ConnectionSign {
              line: 1,
              column: 11
            },
            VLine {
              column: 4,
              line_start: 2,
              line_end: 2
            },
            VLine {
              column: 11,
              line_start: 2,
              line_end: 2
            },
            ConnectionSign { line: 3, column: 4 },
            HLine {
              line: 3,
              column_start: 5,
              column_end: 6
            },
            ConnectionSign { line: 3, column: 7 },
            HLine {
              line: 3,
              column_start: 8,
              column_end: 10
            },
            ConnectionSign {
              line: 3,
              column: 11
            },
          ],
        },
        Element::Connection {
          id: 4,
          from: 0,
          to: 1,
          inner_elements: vec![],
          tokens: vec![VLine {
            column: 7,
            line_start: 4,
            line_end: 4,
          }],
        },
        Element::Block {
          id: 1,
          inner_elements: vec![Element::Text {
            id: 3,
            tokens: vec![Text {
              line: 6,
              column_start: 5,
              column_end: 10
            },],
          }],
          border: vec![
            ConnectionSign { line: 5, column: 4 },
            HLine {
              line: 5,
              column_start: 5,
              column_end: 6
            },
            ConnectionSign { line: 5, column: 7 },
            HLine {
              line: 5,
              column_start: 8,
              column_end: 10
            },
            ConnectionSign {
              line: 5,
              column: 11
            },
            VLine {
              column: 4,
              line_start: 6,
              line_end: 6
            },
            VLine {
              column: 11,
              line_start: 6,
              line_end: 6
            },
            ConnectionSign { line: 7, column: 4 },
            HLine {
              line: 7,
              column_start: 5,
              column_end: 10
            },
            ConnectionSign {
              line: 7,
              column: 11
            },
          ],
        },
      ]
    );
  }

  #[test]
  fn single_box_to_elements() {
    use Token::*;
    let elements = parse_elements(SINGLE_BOX);
    assert_eq!(
      elements,
      vec![Element::Block {
        id: 0,
        inner_elements: vec![Element::Text {
          id: 1,
          tokens: vec![Text {
            line: 3,
            column_start: 6,
            column_end: 8
          },],
        }],
        border: vec![
          ConnectionSign { line: 2, column: 4 },
          HLine {
            line: 2,
            column_start: 5,
            column_end: 9
          },
          ConnectionSign {
            line: 2,
            column: 10
          },
          VLine {
            column: 4,
            line_start: 3,
            line_end: 3
          },
          VLine {
            column: 10,
            line_start: 3,
            line_end: 3
          },
          ConnectionSign { line: 4, column: 4 },
          HLine {
            line: 4,
            column_start: 5,
            column_end: 9
          },
          ConnectionSign {
            line: 4,
            column: 10
          },
        ],
      },]
    );
  }

  #[test]
  fn check_if_box_can_be_continued() {
    let mut tokens = parse_tokens(SINGLE_BOX);
    tokens.reverse();

    let next_token = tokens.pop().unwrap();
    let mut started_block = PartialElement::new(next_token);
    // HLine
    let next_token = tokens.pop().unwrap();
    assert_eq!(
      started_block.can_continue_block(&next_token, SINGLE_BOX),
      true
    );
    assert_eq!(started_block.add_token(next_token), false);

    // ConnectionSign
    let next_token = tokens.pop().unwrap();
    assert_eq!(
      started_block.can_continue_block(&next_token, SINGLE_BOX),
      true
    );
    assert_eq!(started_block.add_token(next_token), false);
    assert_eq!(started_block.clock_cycle_end.line, 2);
    assert_eq!(started_block.clock_cycle_end.column, 10);
    assert_eq!(started_block.counter_clock_cycle_end.line, 2);
    assert_eq!(started_block.counter_clock_cycle_end.column, 4);

    // VLine
    let next_token = tokens.pop().unwrap();
    assert_eq!(
      started_block.can_continue_block(&next_token, SINGLE_BOX),
      true
    );
    assert_eq!(started_block.add_token(next_token), false);
    assert_eq!(started_block.clock_cycle_end.line, 2);
    assert_eq!(started_block.clock_cycle_end.column, 10);
    assert_eq!(started_block.counter_clock_cycle_end.line, 3);
    assert_eq!(started_block.counter_clock_cycle_end.column, 4);

    // Text
    let next_token = tokens.pop().unwrap();
    assert_eq!(
      started_block.can_continue_block(&next_token, SINGLE_BOX),
      false
    );

    // VLine
    let next_token = tokens.pop().unwrap();
    assert_eq!(
      started_block.can_continue_block(&next_token, SINGLE_BOX),
      true
    );
    assert_eq!(started_block.add_token(next_token), false);
    assert_eq!(started_block.clock_cycle_end.line, 3);
    assert_eq!(started_block.clock_cycle_end.column, 10);
    assert_eq!(started_block.counter_clock_cycle_end.line, 3);
    assert_eq!(started_block.counter_clock_cycle_end.column, 4);

    // ConnectionSign
    let next_token = tokens.pop().unwrap();
    assert_eq!(
      started_block.can_continue_block(&next_token, SINGLE_BOX),
      true
    );
    assert_eq!(started_block.add_token(next_token), false);
    assert_eq!(started_block.clock_cycle_end.line, 3);
    assert_eq!(started_block.clock_cycle_end.column, 10);
    assert_eq!(started_block.counter_clock_cycle_end.line, 4);
    assert_eq!(started_block.counter_clock_cycle_end.column, 4);

    // HLine
    let next_token = tokens.pop().unwrap();
    assert_eq!(
      started_block.can_continue_block(&next_token, SINGLE_BOX),
      true
    );
    assert_eq!(started_block.add_token(next_token), false);
    assert_eq!(started_block.clock_cycle_end.line, 3);
    assert_eq!(started_block.clock_cycle_end.column, 10);
    assert_eq!(started_block.counter_clock_cycle_end.line, 4);
    assert_eq!(started_block.counter_clock_cycle_end.column, 9);

    // ConnectionSign
    let next_token = tokens.pop().unwrap();
    assert_eq!(
      started_block.can_continue_block(&next_token, SINGLE_BOX),
      true
    );
    assert_eq!(started_block.add_token(next_token), true);
    assert_eq!(started_block.clock_cycle_end.line, 4);
    assert_eq!(started_block.clock_cycle_end.column, 10);
    assert_eq!(started_block.counter_clock_cycle_end.line, 4);
    assert_eq!(started_block.counter_clock_cycle_end.column, 9);
  }

  const SINGLE_BOX: &str = r"

    +-----+
    | Box |
    +-----+
  ";
}
