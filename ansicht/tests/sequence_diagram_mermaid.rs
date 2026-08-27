use anyhow::Result;
use ansicht::*;
use pretty_assertions::assert_eq;

mod sequence_diagram_fixtures;
use sequence_diagram_fixtures::read_example;
mod test_helpers;
use test_helpers::sequence_diagram_elements;

#[test]
fn service_discovery_happy_path() -> Result<()> {
  let content = read_example("service_discovery.mermaid")?;
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
  let content = read_example("oauth.happy_path.mermaid")?;

  let reader = reader::MermaidReader::new();
  let ast = reader.parse(&content)?;

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    sequence_diagram_fixtures::oauth_happy_path_elements(),
  );

  Ok(())
}

