use crate::ast::*;

pub fn extract_table_of_contents<'a>(input: &AST<'a>, level: u32) -> AST<'a> {
  let elements = input.elements.iter().filter_map(|el| {
    match el.element {
      Element::Title { level: l } => if l <= level {
        Some(el.clone())
      } else {
        None
      },
      _ => None,
    }
  }).collect();

  AST {
    elements,
    ..AST::default()
  }
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;
  use super::*;

  #[test]
  fn find_toc_in_flat_ast() {
    let input = AST {
      elements: vec![
        new_element(Element::Title { level: 1 }, "Main Header"),
        new_element(Element::Paragraph, "Some Text"),
        new_element(Element::Title { level: 2 }, "Subheader"),
        new_element(Element::Paragraph, "Paragraph Text"),
        new_element(Element::Title { level: 3 }, "SubSubheader"),
        new_element(Element::Title { level: 2 }, "Next Subheader"),
        new_element(Element::Paragraph, "blah blah blah"),
      ],
      ..AST::default()
    };
    assert_eq!(
      extract_table_of_contents(&input, 2),
      AST {
        elements: vec![
          new_element(Element::Title { level: 1 }, "Main Header"),
          new_element(Element::Title { level: 2 }, "Subheader"),
          new_element(Element::Title { level: 2 }, "Next Subheader"),
        ],
        ..AST::default()
      }
    );
  }

  fn new_element<'a>(element: Element<'a>, content: &'a str) -> ElementSpan<'a> {
    ElementSpan {
      element,
      source: None,
      content,
      children: vec![],
      attributes: vec![],
      positional_attributes: vec![],
      start: 0,
      end: 0,
      start_line: 0,
      start_col: 0,
      end_line: 0,
      end_col: 0,
    }
  }
}
