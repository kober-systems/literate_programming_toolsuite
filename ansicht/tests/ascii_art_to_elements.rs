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

