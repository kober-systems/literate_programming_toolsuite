use std::{cmp::Ordering, collections::HashMap};

use crate::{
  ast::{Element as AstElement, ElementSpan, Slice, TextPosition},
  SequenceDiagramElement, AST,
};

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

pub struct BoundingBox {
  pub start: Coordinate,
  pub end: Coordinate,
}

pub struct Coordinate {
  pub line: usize,
  pub column: usize,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token {
  HLine {
    line: usize,
    column_start: usize,
    column_end: usize,
  },
  VLine {
    column: usize,
    line_start: usize,
    line_end: usize,
  },
  Text {
    line: usize,
    column_start: usize,
    column_end: usize,
  },
  ConnectionSign {
    line: usize,
    column: usize,
  },
  Arrow {
    line: usize,
    column: usize,
  },
}

impl Token {
  fn get_bounds(&self) -> BoundingBox {
    use Token::*;

    match self {
      ConnectionSign { line, column } => BoundingBox {
        start: Coordinate {
          line: *line,
          column: *column,
        },
        end: Coordinate {
          line: *line,
          column: *column,
        },
      },
      Arrow { line, column } => BoundingBox {
        start: Coordinate {
          line: *line,
          column: *column,
        },
        end: Coordinate {
          line: *line,
          column: *column,
        },
      },
      HLine {
        line,
        column_start,
        column_end,
      } => BoundingBox {
        start: Coordinate {
          line: *line,
          column: *column_start,
        },
        end: Coordinate {
          line: *line,
          column: *column_end,
        },
      },
      Text {
        line,
        column_start,
        column_end,
      } => BoundingBox {
        start: Coordinate {
          line: *line,
          column: *column_start,
        },
        end: Coordinate {
          line: *line,
          column: *column_end,
        },
      },
      VLine {
        column,
        line_start,
        line_end,
      } => BoundingBox {
        start: Coordinate {
          line: *line_start,
          column: *column,
        },
        end: Coordinate {
          line: *line_end,
          column: *column,
        },
      },
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

fn parse_tokens(input: &str) -> Vec<Token> {
  use Token::*;

  let tokens: Vec<_> = input
    .lines()
    .enumerate()
    .flat_map(|(line_number, line)| {
      line
        .chars()
        .enumerate()
        .filter_map(move |(col, sign)| match sign {
          ' ' => None,
          '+' | '┌' | '┐' | '┘' | '└' | '┬' | '┴' | '╔' | '╗' | '╝' | '╚' | '╤' | '╧' => {
            Some(ConnectionSign {
              line: line_number,
              column: col,
            })
          }
          '>' | '<' | 'v' | '^' => Some(Arrow {
            line: line_number,
            column: col,
          }),
          '-' | '─' => Some(HLine {
            line: line_number,
            column_start: col,
            column_end: col,
          }),
          '|' | '│' => Some(VLine {
            column: col,
            line_start: line_number,
            line_end: line_number,
          }),
          _ => Some(Text {
            line: line_number,
            column_start: col,
            column_end: col,
          }),
        })
    })
    .collect();

  let tokens = condense_horizontal(tokens);
  let mut tokens = condense_vertical(tokens);
  tokens.sort_by(|a, b| {
    let a = a.get_bounds();
    let b = b.get_bounds();
    if a.start.line > b.start.line {
      return Ordering::Greater;
    }
    if a.start.line == b.start.line {
      if a.start.column > b.start.column {
        return Ordering::Greater;
      }
      return Ordering::Less;
    }
    Ordering::Less
  });
  tokens
}

fn condense_horizontal(input: Vec<Token>) -> Vec<Token> {
  use Token::*;

  input.into_iter().fold(vec![], |mut out, token| {
    if let Some(last_token) = out.pop() {
      match token {
        HLine {
          line,
          column_start,
          column_end,
        } => {
          let column_start = match last_token {
            HLine {
              line: _,
              column_start: start,
              column_end: _,
            } => start,
            _ => {
              out.push(last_token);
              column_start
            }
          };
          out.push(HLine {
            line,
            column_start,
            column_end,
          })
        }
        Text {
          line,
          column_start,
          column_end,
        } => {
          let column_start = match last_token {
            Text {
              line: _,
              column_start: start,
              column_end: _,
            } => start,
            _ => {
              out.push(last_token);
              column_start
            }
          };
          out.push(Text {
            line,
            column_start,
            column_end,
          })
        }
        token => {
          out.push(last_token);
          out.push(token)
        }
      }
    } else {
      out.push(token)
    }
    out
  })
}

fn condense_vertical(input: Vec<Token>) -> Vec<Token> {
  let vlines: HashMap<usize, Vec<Token>> = HashMap::default();
  let tokens: Vec<Token> = vec![];

  let (vlines, tokens) =
    input
      .into_iter()
      .fold((vlines, tokens), |(mut vlines, mut tokens), token| {
        use Token::*;

        match token {
          VLine {
            column,
            line_start,
            line_end,
          } => {
            if let Some(vlines_on_column) = vlines.get_mut(&column) {
              let (old_column, old_line_start, old_line_end) = match vlines_on_column.pop() {
                Some(VLine {
                  column,
                  line_start,
                  line_end,
                }) => (column, line_start, line_end),
                _ => unreachable!(),
              };
              if old_column == column && line_start <= old_line_end + 1 && line_end > old_line_end {
                vlines_on_column.insert(
                  0,
                  VLine {
                    column,
                    line_start: old_line_start,
                    line_end,
                  },
                );
              } else {
                vlines_on_column.insert(
                  0,
                  VLine {
                    column: old_column,
                    line_start: old_line_start,
                    line_end: old_line_end,
                  },
                );
                vlines_on_column.insert(
                  0,
                  VLine {
                    column,
                    line_start,
                    line_end,
                  },
                );
              }
            } else {
              vlines.insert(
                column,
                vec![VLine {
                  column,
                  line_start,
                  line_end,
                }],
              );
            }
          }
          token => tokens.push(token),
        }

        (vlines, tokens)
      });

  let vlines = vlines.into_iter().fold(vec![], |mut out, (_, mut vlines)| {
    out.append(&mut vlines);
    out
  });

  let (mut out, mut after) =
    tokens
      .into_iter()
      .fold((vec![], vlines), |(mut out, vlines), token| {
        use Token::*;

        let (line, column_start) = match token {
          ConnectionSign { line, column } => (line, column),
          Arrow { line, column } => (line, column),
          HLine {
            line,
            column_start,
            column_end: _,
          } => (line, column_start),
          Text {
            line,
            column_start,
            column_end: _,
          } => (line, column_start),
          VLine {
            column: _,
            line_start: _,
            line_end: _,
          } => unreachable!(),
        };

        let (mut before, after): (Vec<_>, Vec<_>) =
          vlines.into_iter().partition(|token| match token {
            VLine {
              column,
              line_start,
              line_end: _,
            } => {
              if *line_start < line || (*line_start == line && *column <= column_start) {
                true
              } else {
                false
              }
            }
            _ => unreachable!(),
          });
        out.append(&mut before);
        out.push(token);
        (out, after)
      });
  out.append(&mut after);
  out
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
  extract_messages(elements, &lines, &participants)
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
      _ => {}
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

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  // string to tokens

  #[test]
  fn empty_string_to_tokens() {
    let tokens = parse_tokens("");
    assert_eq!(tokens, vec![]);

    let tokens = parse_tokens(
      r"

        ",
    );
    assert_eq!(tokens, vec![]);
  }

  #[test]
  fn single_box_to_tokens() {
    use Token::*;
    let tokens = parse_tokens(SINGLE_BOX);
    assert_eq!(
      tokens,
      vec![
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
        Text {
          line: 3,
          column_start: 6,
          column_end: 8
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
      ]
    );
  }

  #[test]
  fn single_multiline_box_to_tokens() {
    use Token::*;
    let tokens = parse_tokens(BOX_WITH_MULTILINE_TEXT);
    assert_eq!(
      tokens,
      vec![
        ConnectionSign { line: 1, column: 4 },
        HLine {
          line: 1,
          column_start: 5,
          column_end: 19
        },
        ConnectionSign {
          line: 1,
          column: 20
        },
        VLine {
          column: 4,
          line_start: 2,
          line_end: 4
        },
        Text {
          line: 2,
          column_start: 6,
          column_end: 14
        },
        VLine {
          column: 20,
          line_start: 2,
          line_end: 4
        },
        Text {
          line: 3,
          column_start: 6,
          column_end: 19
        },
        Text {
          line: 4,
          column_start: 6,
          column_end: 11
        },
        ConnectionSign { line: 5, column: 4 },
        HLine {
          line: 5,
          column_start: 5,
          column_end: 19
        },
        ConnectionSign {
          line: 5,
          column: 20
        },
      ]
    );
  }

  #[test]
  fn multiple_boxes_in_the_same_row_to_tokens() {
    use Token::*;
    let tokens = parse_tokens(TWO_BOXES_IN_THE_SAME_ROWS);
    assert_eq!(
      tokens,
      vec![
        ConnectionSign { line: 1, column: 4 },
        HLine {
          line: 1,
          column_start: 5,
          column_end: 9
        },
        ConnectionSign {
          line: 1,
          column: 10
        },
        ConnectionSign {
          line: 1,
          column: 19
        },
        HLine {
          line: 1,
          column_start: 20,
          column_end: 24
        },
        ConnectionSign {
          line: 1,
          column: 25
        },
        VLine {
          column: 4,
          line_start: 2,
          line_end: 2
        },
        Text {
          line: 2,
          column_start: 6,
          column_end: 8
        },
        VLine {
          column: 10,
          line_start: 2,
          line_end: 2
        },
        VLine {
          column: 19,
          line_start: 2,
          line_end: 2
        },
        Text {
          line: 2,
          column_start: 21,
          column_end: 23
        },
        VLine {
          column: 25,
          line_start: 2,
          line_end: 2
        },
        ConnectionSign { line: 3, column: 4 },
        HLine {
          line: 3,
          column_start: 5,
          column_end: 9
        },
        ConnectionSign {
          line: 3,
          column: 10
        },
        ConnectionSign {
          line: 3,
          column: 19
        },
        HLine {
          line: 3,
          column_start: 20,
          column_end: 24
        },
        ConnectionSign {
          line: 3,
          column: 25
        },
      ]
    );
  }

  #[test]
  fn multiple_boxes_in_the_same_column_to_tokens() {
    use Token::*;
    let tokens = parse_tokens(TWO_BOXES_IN_THE_SAME_COLUMNS);
    assert_eq!(
      tokens,
      vec![
        ConnectionSign { line: 1, column: 4 },
        HLine {
          line: 1,
          column_start: 5,
          column_end: 9
        },
        ConnectionSign {
          line: 1,
          column: 10
        },
        VLine {
          column: 4,
          line_start: 2,
          line_end: 2
        },
        Text {
          line: 2,
          column_start: 6,
          column_end: 8
        },
        VLine {
          column: 10,
          line_start: 2,
          line_end: 2
        },
        ConnectionSign { line: 3, column: 4 },
        HLine {
          line: 3,
          column_start: 5,
          column_end: 9
        },
        ConnectionSign {
          line: 3,
          column: 10
        },
        ConnectionSign { line: 5, column: 4 },
        HLine {
          line: 5,
          column_start: 5,
          column_end: 9
        },
        ConnectionSign {
          line: 5,
          column: 10
        },
        VLine {
          column: 4,
          line_start: 6,
          line_end: 6
        },
        Text {
          line: 6,
          column_start: 6,
          column_end: 8
        },
        VLine {
          column: 10,
          line_start: 6,
          line_end: 6
        },
        ConnectionSign { line: 7, column: 4 },
        HLine {
          line: 7,
          column_start: 5,
          column_end: 9
        },
        ConnectionSign {
          line: 7,
          column: 10
        },
      ]
    );
  }

  #[test]
  fn two_connected_boxes_to_tokens() {
    use Token::*;
    let tokens = parse_tokens(TWO_CONNECTED_BOXES);
    assert_eq!(
      tokens,
      vec![
        ConnectionSign { line: 1, column: 4 },
        HLine {
          line: 1,
          column_start: 5,
          column_end: 9
        },
        ConnectionSign {
          line: 1,
          column: 10
        },
        VLine {
          column: 4,
          line_start: 2,
          line_end: 2
        },
        Text {
          line: 2,
          column_start: 6,
          column_end: 8
        },
        VLine {
          column: 10,
          line_start: 2,
          line_end: 2
        },
        HLine {
          line: 2,
          column_start: 11,
          column_end: 12
        },
        ConnectionSign {
          line: 2,
          column: 13
        },
        ConnectionSign { line: 3, column: 4 },
        HLine {
          line: 3,
          column_start: 5,
          column_end: 9
        },
        ConnectionSign {
          line: 3,
          column: 10
        },
        VLine {
          column: 13,
          line_start: 3,
          line_end: 5
        },
        ConnectionSign {
          line: 5,
          column: 17
        },
        HLine {
          line: 5,
          column_start: 18,
          column_end: 22
        },
        ConnectionSign {
          line: 5,
          column: 23
        },
        ConnectionSign {
          line: 6,
          column: 13
        },
        HLine {
          line: 6,
          column_start: 14,
          column_end: 15
        },
        Arrow {
          line: 6,
          column: 16,
        },
        VLine {
          column: 17,
          line_start: 6,
          line_end: 6
        },
        Text {
          line: 6,
          column_start: 19,
          column_end: 21
        },
        VLine {
          column: 23,
          line_start: 6,
          line_end: 6
        },
        ConnectionSign {
          line: 7,
          column: 17
        },
        HLine {
          line: 7,
          column_start: 18,
          column_end: 22
        },
        ConnectionSign {
          line: 7,
          column: 23
        },
      ]
    );
  }

  #[test]
  fn single_styled_box_to_tokens() {
    use Token::*;
    let tokens = parse_tokens(SINGLE_BOX_STYLE);
    assert_eq!(
      tokens,
      vec![
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
        Text {
          line: 3,
          column_start: 6,
          column_end: 8
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
      ]
    );
  }

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
          from: 0,
          to: 1,
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

  const BOX_WITH_MULTILINE_TEXT: &str = r"
    +---------------+
    | This text     |
    | spans multiple|
    | lines.        |
    +---------------+
  ";

  const TWO_BOXES_IN_THE_SAME_ROWS: &str = r"
    +-----+        +-----+
    | Box |        | Box |
    +-----+        +-----+
  ";

  const TWO_BOXES_IN_THE_SAME_COLUMNS: &str = r"
    +-----+
    | Box |
    +-----+

    +-----+
    | Box |
    +-----+
  ";

  const TWO_CONNECTED_BOXES: &str = r"
    +-----+
    | Box |--+
    +-----+  |
             |
             |   +-----+
             +-->| Box |
                 +-----+
  ";

  const SINGLE_BOX_STYLE: &str = r"

    ┌─────┐
    │ Box │
    └─────┘
  ";
}
