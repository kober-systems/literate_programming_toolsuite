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
  vec![]
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
    let tokens = parse_tokens(single_box);
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

  const single_box: &str = r"

    +-----+
    | Box |
    +-----+
  ";
}

