use std::io::Write;

use crate::{Element, SequenceDiagramElement, Writer, AST};

pub struct MermaidWriter;

impl<T: Write> Writer<T> for MermaidWriter {
  fn write<'a>(&mut self, ast: AST<'a>, mut out: T) -> crate::Result<()> {
    writeln!(out, "sequenceDiagram").map_err(|e| {
      crate::Error::ParseError(format!("Failed to write mermaid: {}", e))
    })?;

    let mut message_index = 0;
    for element_span in &ast.elements {
      match &element_span.element {
        Element::Sequence(SequenceDiagramElement::CheckedState { .. }) => {
          // Skip CheckedState elements in Mermaid output (no direct equivalent)
        }
        Element::Sequence(SequenceDiagramElement::Message {
          from,
          to,
          message,
          ..
        }) => {
          // Use alternating arrow style: even indices use solid (->>) arrows,
          // odd indices use dashed (-->>) arrows
          let arrow = if message_index % 2 == 0 { "->>" } else { "-->>" };

          writeln!(out, "    {}{}{}: {}", from, arrow, to, message).map_err(|e| {
            crate::Error::ParseError(format!("Failed to write step: {}", e))
          })?;

          message_index += 1;
        }
        _ => {
          // Ignore other element types
        }
      }
    }

    Ok(())
  }
}
