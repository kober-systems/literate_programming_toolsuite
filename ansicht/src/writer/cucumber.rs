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
    let mut last_message = LastMessage::Feature;
    let mut elements = ast.elements.iter().peekable();

    while let Some(element_span) = elements.next() {
      match &element_span.element {
        Element::Sequence(SequenceDiagramElement::CheckedState { name, .. }) => {
          current_scenario = Some(name.clone());
          writeln!(out, "\n  Scenario: {}", name).map_err(|e| {
            crate::Error::ParseError(format!("Failed to write scenario: {}", e))
          })?;

          if !is_next_checked_state(&mut elements) {
            writeln!(out, "    Given {}", name).map_err(|e| {
              crate::Error::ParseError(format!("Failed to write step: {}", e))
            })?;
          } else {
            // TODO
          }
          last_message = LastMessage::Given;
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
            writeln!(out, "    Then {} responds with \"{}\" to {}", from, message, to)
              .map_err(|e| crate::Error::ParseError(format!("Failed to write step: {}", e)))?;
            last_message = LastMessage::Then;
          } else {
            match last_message {
              LastMessage::When => {
                writeln!(out, "    And {} sends \"{}\" to {}", from, message, to)
                  .map_err(|e| crate::Error::ParseError(format!("Failed to write step: {}", e)))?;
                last_message = LastMessage::When;
              }
              _ => {
                writeln!(out, "    When {} sends \"{}\" to {}", from, message, to)
                  .map_err(|e| crate::Error::ParseError(format!("Failed to write step: {}", e)))?;
                last_message = LastMessage::When;
              }
            }
          }

          if is_next_checked_state(&mut elements) {
            if let Some(next) = elements.peek() {
              if let Element::Sequence(SequenceDiagramElement::CheckedState { name, .. }) = &next.element {
                match last_message {
                  LastMessage::Then => {
                    writeln!(out, "    And {}", name).map_err(|e| {
                      crate::Error::ParseError(format!("Failed to write step: {}", e))
                    })?;
                  }
                  _ => {
                    writeln!(out, "    Then {}", name).map_err(|e| {
                      crate::Error::ParseError(format!("Failed to write step: {}", e))
                    })?;
                  }
                }
              }
            }
          }
        }
        other => todo!("not implemented {:?}", other)
      }
    }

    Ok(())
  }
}

enum LastMessage {
  Feature,
  Scenario,
  Given,
  When,
  Then,
}

fn is_next_checked_state<'a, I>(elements: &mut std::iter::Peekable<I>) -> bool
where
  I: Iterator<Item = &'a crate::ast::ElementSpan>,
{
  elements.peek().is_some_and(|next| {
    matches!(
      next.element,
      Element::Sequence(SequenceDiagramElement::CheckedState { .. })
    )
  })
}
