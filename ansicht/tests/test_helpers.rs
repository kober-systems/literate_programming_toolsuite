use ansicht::*;
use pretty_assertions::assert_eq;

pub fn sequence_diagram_elements(input: Vec<ElementSpan>) -> Vec<SequenceDiagramElement> {
  input
    .into_iter()
    .map(|e| match e.element {
      Element::Sequence(e) => e,
      _ => panic!("element {:#?} is no sequence diagram element", e),
    })
    .collect()
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

pub(crate) use assert_element_matches;
