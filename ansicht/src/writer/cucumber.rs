use std::io::Write;

use crate::{Element, SequenceDiagramElement, Writer, AST};

pub struct CucumberWriter {
  pub feature_name: String,
  pub perspective: String,
}

impl<T: Write> Writer<T> for CucumberWriter {
  fn write<'a>(&mut self, ast: AST<'a>, mut out: T) -> crate::Result<()> {
    writeln!(out, "Feature: {}", self.feature_name).map_err(|e| {
      crate::Error::ParseError(format!("Failed to write feature: {}", e))
    })?;

    let mut current_scenario: Option<String> = None;

    for element_span in &ast.elements {
      match &element_span.element {
        Element::Sequence(SequenceDiagramElement::CheckedState { name, .. }) => {
          current_scenario = Some(name.clone());
          writeln!(out, "\n  Scenario: {}", name).map_err(|e| {
            crate::Error::ParseError(format!("Failed to write scenario: {}", e))
          })?;
        }
        Element::Sequence(SequenceDiagramElement::Message {
          from,
          to,
          message,
          ..
        }) => {
          if current_scenario.is_none() {
            current_scenario = Some("Interactions".to_string());
            writeln!(out, "\n  Scenario: Interactions").map_err(|e| {
              crate::Error::ParseError(format!("Failed to write scenario: {}", e))
            })?;
          }

          if from == &self.perspective {
            writeln!(out, "    When {} sends \"{}\" to {}", from, message, to)
              .map_err(|e| crate::Error::ParseError(format!("Failed to write step: {}", e)))?;
          } else {
            writeln!(out, "    Then {} responds with \"{}\"", from, message)
              .map_err(|e| crate::Error::ParseError(format!("Failed to write step: {}", e)))?;
          }
        }
        _ => {
          // Ignore non-sequence elements
        }
      }
    }

    Ok(())
  }
}
