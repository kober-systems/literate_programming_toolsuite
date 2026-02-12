use ansicht::reader::{parse_elements, Element, Token};
use pretty_assertions::assert_eq;

#[test]
fn empty_input() {
  let elements = parse_elements("");
  assert_eq!(elements, vec![]);
}

#[test]
fn whitespace_only() {
  let elements = parse_elements("   \n  \n   ");
  assert_eq!(elements, vec![]);
}

#[test]
fn single_line_text() {
  let elements = parse_elements("Hello World");
  assert_eq!(
    elements,
    vec![Element::Text {
      id: 0,
      tokens: vec![Token::Text {
        line: 0,
        column_start: 0,
        column_end: 10,
      }]
    }]
  );
}

#[test]
fn simple_box() {
  let input = r"
    +-----+
    | Box |
    +-----+
  ";
  let elements = parse_elements(input);

  assert_eq!(elements.len(), 1);
  match &elements[0] {
    Element::Block {
      id,
      inner_elements,
      border,
    } => {
      assert_eq!(*id, 0);
      assert_eq!(inner_elements.len(), 1);
      assert_eq!(border.len(), 8);

      // Check that the inner element is text
      match &inner_elements[0] {
        Element::Text { id, tokens } => {
          assert_eq!(*id, 1);
          assert_eq!(tokens.len(), 1);
          match tokens[0] {
            Token::Text {
              line,
              column_start,
              column_end,
            } => {
              assert_eq!(line, 2);
              assert_eq!(column_start, 6);
              assert_eq!(column_end, 8);
            }
            _ => panic!("Expected Text token"),
          }
        }
        _ => panic!("Expected Text element inside block"),
      }
    }
    _ => panic!("Expected Block element"),
  }
}

#[test]
fn box_with_multiline_text() {
  let input = r"
    +---------------+
    | This text     |
    | spans multiple|
    | lines.        |
    +---------------+
  ";
  let elements = parse_elements(input);

  assert_eq!(elements.len(), 1);
  match &elements[0] {
    Element::Block {
      id: _,
      inner_elements,
      border: _,
    } => {
      // Should have 3 text elements inside
      assert_eq!(inner_elements.len(), 3);
      assert!(matches!(inner_elements[0], Element::Text { .. }));
      assert!(matches!(inner_elements[1], Element::Text { .. }));
      assert!(matches!(inner_elements[2], Element::Text { .. }));
    }
    _ => panic!("Expected Block element"),
  }
}

#[test]
fn two_boxes_side_by_side() {
  use Token::*;
  let input = r"
    +-----+        +-----+
    | Box |        | Box |
    +-----+        +-----+
  ";
  let elements = parse_elements(input);

  assert_eq!(elements.len(), 2);

  // First box
  assert_element_matches!(
    &elements[0],
    Element::Block {
      inner_elements: [Element::Text {
        tokens: vec![Text {
          line: 2,
          column_start: 6,
          column_end: 8,
        }],
        ..
      }],
      border: [
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
      ]
    }
  );

  // Second box
  assert_element_matches!(
    &elements[1],
    Element::Block {
      inner_elements: [Element::Text {
        tokens: vec![Text {
          line: 2,
          column_start: 21,
          column_end: 23,
        }],
        ..
      }],
      border: [
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
          column: 19,
          line_start: 2,
          line_end: 2
        },
        VLine {
          column: 25,
          line_start: 2,
          line_end: 2
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
    }
  );
}

#[test]
fn two_boxes_stacked() {
  let input = r"
    +-----+
    | Box |
    +-----+

    +-----+
    | Box |
    +-----+
  ";
  let elements = parse_elements(input);

  assert_eq!(elements.len(), 2);
  assert!(matches!(elements[0], Element::Block { .. }));
  assert!(matches!(elements[1], Element::Block { .. }));
}

#[test]
fn box_with_empty_content() {
  let input = r"
    +-----+
    |     |
    +-----+
  ";
  let elements = parse_elements(input);

  assert_eq!(elements.len(), 1);
  match &elements[0] {
    Element::Block {
      id: _,
      inner_elements,
      border: _,
    } => {
      // Empty box should have no inner elements
      assert_eq!(inner_elements.len(), 0);
    }
    _ => panic!("Expected Block element"),
  }
}

#[test]
fn text_outside_box() {
  let input = r"
Some text before
    +-----+
    | Box |
    +-----+
Some text after
  ";
  let elements = parse_elements(input);

  // Should have text and block elements
  assert!(elements.len() >= 2);

  let mut has_text = false;
  let mut has_block = false;

  for element in elements {
    match element {
      Element::Text { .. } => has_text = true,
      Element::Block { .. } => has_block = true,
      _ => {}
    }
  }

  assert!(has_text, "Should have at least one text element");
  assert!(has_block, "Should have at least one block element");
}

#[test]
fn wide_box() {
  let input = r"
    +--------------------+
    | Wide Box           |
    +--------------------+
  ";
  let elements = parse_elements(input);

  assert_eq!(elements.len(), 1);
  match &elements[0] {
    Element::Block {
      id: _,
      inner_elements,
      border,
    } => {
      assert_eq!(inner_elements.len(), 1);
      assert!(border.len() > 0);
    }
    _ => panic!("Expected Block element"),
  }
}

#[test]
fn tall_box() {
  let input = r"
    +------+
    | Line1|
    | Line2|
    | Line3|
    | Line4|
    | Line5|
    +------+
  ";
  let elements = parse_elements(input);

  assert_eq!(elements.len(), 1);
  match &elements[0] {
    Element::Block {
      id: _,
      inner_elements,
      border: _,
    } => {
      // Should have 5 text elements
      assert_eq!(inner_elements.len(), 5);
    }
    _ => panic!("Expected Block element"),
  }
}

#[test]
fn three_boxes_in_row() {
  let input = r"
    +---+  +---+  +---+
    | A |  | B |  | C |
    +---+  +---+  +---+
  ";
  let elements = parse_elements(input);

  assert_eq!(
    elements,
    vec![
      Element::Block {
        id: 0,
        inner_elements: vec![Element::Text {
          id: 3,
          tokens: vec![Token::Text {
            line: 2,
            column_start: 6,
            column_end: 6,
          },],
        },],
        border: vec![
          Token::ConnectionSign { line: 1, column: 4 },
          Token::HLine {
            line: 1,
            column_start: 5,
            column_end: 7,
          },
          Token::ConnectionSign { line: 1, column: 8 },
          Token::VLine {
            column: 4,
            line_start: 2,
            line_end: 2,
          },
          Token::VLine {
            column: 8,
            line_start: 2,
            line_end: 2,
          },
          Token::ConnectionSign { line: 3, column: 4 },
          Token::HLine {
            line: 3,
            column_start: 5,
            column_end: 7,
          },
          Token::ConnectionSign { line: 3, column: 8 },
        ],
      },
      Element::Block {
        id: 1,
        inner_elements: vec![Element::Text {
          id: 4,
          tokens: vec![Token::Text {
            line: 2,
            column_start: 13,
            column_end: 13,
          },],
        },],
        border: vec![
          Token::ConnectionSign {
            line: 1,
            column: 11
          },
          Token::HLine {
            line: 1,
            column_start: 12,
            column_end: 14,
          },
          Token::ConnectionSign {
            line: 1,
            column: 15
          },
          Token::VLine {
            column: 11,
            line_start: 2,
            line_end: 2,
          },
          Token::VLine {
            column: 15,
            line_start: 2,
            line_end: 2,
          },
          Token::ConnectionSign {
            line: 3,
            column: 11
          },
          Token::HLine {
            line: 3,
            column_start: 12,
            column_end: 14,
          },
          Token::ConnectionSign {
            line: 3,
            column: 15
          },
        ],
      },
      Element::Block {
        id: 2,
        inner_elements: vec![Element::Text {
          id: 5,
          tokens: vec![Token::Text {
            line: 2,
            column_start: 20,
            column_end: 20,
          },],
        },],
        border: vec![
          Token::ConnectionSign {
            line: 1,
            column: 18
          },
          Token::HLine {
            line: 1,
            column_start: 19,
            column_end: 21,
          },
          Token::ConnectionSign {
            line: 1,
            column: 22
          },
          Token::VLine {
            column: 18,
            line_start: 2,
            line_end: 2,
          },
          Token::VLine {
            column: 22,
            line_start: 2,
            line_end: 2,
          },
          Token::ConnectionSign {
            line: 3,
            column: 18
          },
          Token::HLine {
            line: 3,
            column_start: 19,
            column_end: 21,
          },
          Token::ConnectionSign {
            line: 3,
            column: 22
          },
        ],
      },
    ]
  );
}

#[test]
fn minimal_box() {
  let input = r"
    +-+
    | |
    +-+
  ";
  let elements = parse_elements(input);

  assert_eq!(elements.len(), 1);
  assert!(matches!(elements[0], Element::Block { .. }));
}

#[test]
fn box_with_single_character_text() {
  let input = r"
    +---+
    | X |
    +---+
  ";
  let elements = parse_elements(input);

  assert_eq!(elements.len(), 1);
  match &elements[0] {
    Element::Block {
      id: _,
      inner_elements,
      border: _,
    } => {
      assert_eq!(inner_elements.len(), 1);
      match &inner_elements[0] {
        Element::Text { id: _, tokens } => {
          assert_eq!(tokens.len(), 1);
        }
        _ => panic!("Expected Text element"),
      }
    }
    _ => panic!("Expected Block element"),
  }
}

/// helper macro for more ergonimic tests
macro_rules! assert_element_matches {
  // Block pattern with inner_elements and border
  ($actual:expr, Element::Block {
    inner_elements: [$($inner:tt)*],
    border: [$($border:expr),* $(,)?]
    $(,)?
  }) => {{
    match $actual {
      Element::Block { id: _, inner_elements, border } => {
        let expected_border = vec![$($border),*];
        assert_eq!(border.as_slice(), expected_border.as_slice(), "border mismatch");

        // Match inner elements
        assert_element_matches!(@inner_elements inner_elements, [$($inner)*]);
      }
      other => panic!("Expected Element::Block, got {:?}", other),
    }
  }};

  // Helper: match inner elements array
  (@inner_elements $actual:expr, [$(Element::Text {
    tokens: $tokens:expr,
    ..
  }),* $(,)?]) => {{
    let expected_texts: Vec<Vec<Token>> = vec![$($tokens),*];
    assert_eq!($actual.len(), expected_texts.len(), "inner_elements length mismatch");

    for (i, (actual_elem, expected_tokens)) in $actual.iter().zip(expected_texts.iter()).enumerate() {
      match actual_elem {
        Element::Text { id: _, tokens } => {
          assert_eq!(tokens, expected_tokens, "inner_elements[{}] tokens mismatch", i);
        }
        other => panic!("Expected Element::Text at index {}, got {:?}", i, other),
      }
    }
  }};

  // Text pattern with .. (ignore id)
  ($actual:expr, Element::Text {
    tokens: $tokens:expr,
    ..
  }) => {{
    match $actual {
      Element::Text { id: _, tokens } => {
        assert_eq!(tokens, &$tokens, "tokens mismatch");
      }
      other => panic!("Expected Element::Text, got {:?}", other),
    }
  }};
}

