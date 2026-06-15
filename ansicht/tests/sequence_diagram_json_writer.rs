use ansicht::{
  writer::json::JsonWriter, CursorPosition, Element, ElementSpan, SequenceDiagramElement,
  TextPosition, Writer, AST,
};
use serde_json::Value;

#[test]
fn writes_ast_as_json() {
  let ast = AST {
    content: "Alice -> Bob: hello",
    elements: vec![ElementSpan {
      source: None,
      position: TextPosition::BoundingBox {
        start: CursorPosition { line: 1, column: 1 },
        end: CursorPosition {
          line: 1,
          column: 20,
        },
      },
      element: Element::Sequence(SequenceDiagramElement::message("Alice", "Bob", "hello")),
      children: Vec::new(),
      attrs: Vec::new(),
    }],
  };

  let mut output = Vec::new();
  JsonWriter.write(ast, &mut output).unwrap();

  let json: Value = serde_json::from_slice(&output).unwrap();
  assert_eq!(json["content"], "Alice -> Bob: hello");
  assert_eq!(
    json["elements"][0]["element"]["Sequence"]["Message"]["from"],
    "Alice"
  );
  assert_eq!(
    json["elements"][0]["element"]["Sequence"]["Message"]["to"],
    "Bob"
  );
  assert_eq!(
    json["elements"][0]["element"]["Sequence"]["Message"]["message"],
    "hello"
  );
}
