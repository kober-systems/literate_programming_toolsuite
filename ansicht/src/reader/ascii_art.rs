use std::{cmp::Ordering, collections::HashMap};

use crate::AST;

pub struct AsciiArtReader {}

impl AsciiArtReader {
  pub fn new() -> Self {
    Self {}
  }

  pub fn parse<'a>(&self, input: &'a str) -> AST<'a> {
    AST {
      content: input,
      elements: vec![],
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

pub struct BoundingBox {
  pub start: Coordinate,
  pub end: Coordinate,
}

pub struct Coordinate {
  pub line: usize,
  pub column: usize,
}

#[derive(Debug, PartialEq)]
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
  elements_from_tokens(parse_tokens(input))
}

fn elements_from_tokens(input: Vec<Token>) -> Vec<Element> {
  use Element::*;

  let mut texts = vec![];
  for token in input.into_iter() {
    match token {
      Token::Text {
        line: _,
        column_start: _,
        column_end: _,
      } => {
        texts.push(token);
      }
      _ => {}
    }
  }

  let mut next_id = 0;
  let mut out = vec![];
  for text in texts.into_iter() {
    out.push(Text {
      id: next_id,
      tokens: vec![text],
    });
    next_id += 1;
  }

  out
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

  fn can_continue_block(&self, next_token: &Token, text: &str) -> bool {
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

  fn add_token(&mut self, token: Token) {
    let BoundingBox { start: _, end } = token.get_bounds();
    self.tokens.push(token);
    if end.line >= self.clock_cycle_end.line && end.column >= self.clock_cycle_end.column {
      self.clock_cycle_end = end;
    } else {
      self.counter_clock_cycle_end = end;
    }
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
          '+' => Some(ConnectionSign {
            line: line_number,
            column: col,
          }),
          '>' | '<' | 'v' | '^' => Some(Arrow {
            line: line_number,
            column: col,
          }),
          '-' => Some(HLine {
            line: line_number,
            column_start: col,
            column_end: col,
          }),
          '|' => Some(VLine {
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
    started_block.add_token(next_token);

    // ConnectionSign
    let next_token = tokens.pop().unwrap();
    assert_eq!(
      started_block.can_continue_block(&next_token, SINGLE_BOX),
      true
    );
    started_block.add_token(next_token);
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
    started_block.add_token(next_token);
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
    started_block.add_token(next_token);
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
    started_block.add_token(next_token);
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
    started_block.add_token(next_token);
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
    started_block.add_token(next_token);
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
}

