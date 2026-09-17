use anyhow::Result;
use ansicht::*;
use pretty_assertions::assert_eq;

mod sequence_diagram_fixtures;
use sequence_diagram_fixtures::read_example;
#[path = "common/test_helpers.rs"]
mod test_helpers;
use test_helpers::sequence_diagram_elements;

#[test]
fn service_discovery_happy_path() -> Result<()> {
  let content = read_example("sequence-diagram/service_discovery.mermaid")?;
  let reader = reader::MermaidReader::new();
  let ast = reader.parse(&content)?;

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    sequence_diagram_fixtures::service_discovery_happy_path_elements()
  );

  Ok(())
}

#[test]
fn oauth_happy_path_mermaid() -> Result<()> {
  let content = read_example("sequence-diagram/oauth.happy_path.mermaid")?;

  let reader = reader::MermaidReader::new();
  let ast = reader.parse(&content)?;

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    sequence_diagram_fixtures::oauth_happy_path_elements(),
  );

  Ok(())
}

