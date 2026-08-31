use std::{cmp::Ordering, collections::HashMap};

pub fn parse_tokens(input: &str) -> Vec<Token> {
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
          '-' | '─' | '═' | '=' => Some(HLine {
            line: line_number,
            column_start: col,
            column_end: col,
          }),
          '|' | '│' | '║' => Some(VLine {
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
  pub fn get_bounds(&self) -> BoundingBox {
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

pub struct BoundingBox {
  pub start: Coordinate,
  pub end: Coordinate,
}

pub struct Coordinate {
  pub line: usize,
  pub column: usize,
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
