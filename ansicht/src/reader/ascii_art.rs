use std::collections::HashMap;

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
  pub line_start: usize,
  pub column_start: usize,
  pub line_end: usize,
  pub column_end: usize,
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
  condense_vertical(tokens)
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
              line,
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
              line,
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
}

