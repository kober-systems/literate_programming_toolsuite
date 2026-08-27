use anyhow::Result;
use ansicht::*;
use pretty_assertions::assert_eq;

mod test_helpers;
use test_helpers::sequence_diagram_elements;
mod sequence_diagram_fixtures;
use sequence_diagram_fixtures::read_example;

#[test]
fn oauth_happy_path() -> Result<()> {
  let content = read_example("oauth.happy_path.plantuml")?;
  let reader = reader::PlantUmlReader::new();
  let ast = reader.parse(&content)?;

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    sequence_diagram_fixtures::oauth_happy_path_elements(),
  );

  Ok(())
}

#[test]
fn service_discovery_happy_path() -> Result<()> {
  let content = read_example("service_discovery.plantuml")?;
  let reader = reader::PlantUmlReader::new();
  let ast = reader.parse(&content)?;

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    sequence_diagram_fixtures::service_discovery_happy_path_elements()
  );

  Ok(())
}

