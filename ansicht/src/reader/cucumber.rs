pub use crate::ast::*;
use crate::Result;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

pub struct CucumberReader {}

impl CucumberReader {
  pub fn new() -> Self {
    CucumberReader {}
  }
}

impl crate::Reader for CucumberReader {
  fn parse<'a>(&self, input: &'a str) -> Result<AST<'a>> {
    let ast = CucumberParser::parse(Rule::feature_file, input)
      .map_err(|e| crate::Error::ParseError(format!("Failed to parse cucumber: {}", e)))?;

    let mut elements = Vec::new();
    let mut _feature_name = String::new();

    for element in ast {
      if element.as_rule() == Rule::feature {
        _feature_name = parse_feature(element, &mut elements)?;
      }
    }

    Ok(AST {
      content: input,
      elements,
    })
  }
}

#[derive(Parser, Debug)]
#[grammar = "reader/cucumber.pest"]
pub struct CucumberParser;

fn parse_feature(pair: Pair<Rule>, elements: &mut Vec<ElementSpan>) -> Result<String> {
  let mut feature_name = String::new();

  for sub in pair.into_inner() {
    match sub.as_rule() {
      Rule::feature_name => {
        feature_name = sub.as_str().trim().to_string();
      }
      Rule::scenario => {
        parse_scenario(sub, &feature_name, elements)?;
      }
      Rule::blank_line => {
        // Ignore blank lines
      }
      _ => {}
    }
  }

  Ok(feature_name)
}

fn parse_scenario(
  pair: Pair<Rule>,
  _feature_name: &str,
  elements: &mut Vec<ElementSpan>,
) -> Result<()> {
  let mut steps = Vec::new();

  for sub in pair.into_inner() {
    match sub.as_rule() {
      Rule::step => {
        if let Some(parsed_step) = parse_step(sub) {
          steps.push(parsed_step);
        }
      }
      Rule::blank_line => {
        // Ignore blank lines
      }
      _ => {}
    }
  }

  // Infer missing "to" values for "Then" steps from previous "When" step
  for i in 0..steps.len() {
    if steps[i].to.is_empty() && i > 0 {
      // This is a "Then" step; infer "to" from previous "When" step
      steps[i].to = steps[i - 1].from.clone();
    }
  }

  // Extract participants from steps
  let mut participants = Vec::new();
  for step in &steps {
    if !participants.contains(&step.from) {
      participants.push(step.from.clone());
    }
    if !participants.contains(&step.to) {
      participants.push(step.to.clone());
    }
  }

  // Add Message elements
  for step in steps {
    elements.push(ElementSpan {
      source: None,
      position: TextPosition::Slice(Slice { start: 0, end: 0 }),
      element: Element::Sequence(SequenceDiagramElement::Message {
        from: step.from,
        to: step.to,
        message: step.message,
        meta: None,
      }),
      children: Vec::new(),
      attrs: Vec::new(),
    });
  }

  Ok(())
}

struct ParsedStep {
  from: String,
  to: String,
  message: String,
}

fn parse_step(pair: Pair<Rule>) -> Option<ParsedStep> {
  let mut keyword = String::new();
  let mut text = String::new();

  for sub in pair.into_inner() {
    match sub.as_rule() {
      Rule::step_keyword => {
        keyword = sub.as_str().to_string();
      }
      Rule::step_text => {
        text = sub.as_str().trim().to_string();
      }
      _ => {}
    }
  }

  match keyword.as_str() {
    "When" => parse_when_step(&text),
    "Then" => parse_then_step(&text),
    _ => None,
  }
}

fn parse_when_step(text: &str) -> Option<ParsedStep> {
  // Pattern: <from> sends "<message>" to <to>
  let parts: Vec<&str> = text.splitn(4, ' ').collect();
  if parts.len() < 4 {
    return None;
  }

  let from = parts[0].to_string();

  if parts[1] != "sends" {
    return None;
  }

  // Extract quoted message and "to" part
  if !parts[2].starts_with('"') {
    return None;
  }

  let rest = parts[2..].join(" ");
  let (message, to) = extract_quoted_and_target(&rest)?;

  Some(ParsedStep { from, message, to })
}

fn parse_then_step(text: &str) -> Option<ParsedStep> {
  // Pattern: <from> responds with "<message>"
  let parts: Vec<&str> = text.splitn(4, ' ').collect();
  if parts.len() < 4 {
    return None;
  }

  let from = parts[0].to_string();

  if parts[1] != "responds" || parts[2] != "with" {
    return None;
  }

  let rest = parts[3..].join(" ");
  let message = extract_quoted(&rest)?;

  Some(ParsedStep {
    from,
    message,
    to: String::new(), // Will be filled in by context
  })
}

fn extract_quoted(text: &str) -> Option<String> {
  if !text.starts_with('"') {
    return None;
  }
  let end = text[1..].find('"')?;
  Some(text[1..1 + end].to_string())
}

fn extract_quoted_and_target(text: &str) -> Option<(String, String)> {
  if !text.starts_with('"') {
    return None;
  }
  let end = text[1..].find('"')?;
  let message = text[1..1 + end].to_string();
  let rest = text[1 + end + 1..].trim();

  // Should be "to <participant>"
  let to_parts: Vec<&str> = rest.split_whitespace().collect();
  if to_parts.len() < 2 || to_parts[0] != "to" {
    return None;
  }

  Some((message, to_parts[1].to_string()))
}
