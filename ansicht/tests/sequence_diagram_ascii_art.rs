use anyhow::Result;
use ansicht::*;
use pretty_assertions::assert_eq;

mod sequence_diagram_fixtures;
mod test_helpers;
use test_helpers::sequence_diagram_elements;

#[test]
fn oauth_happy_path() -> Result<()> {
  let content = read_example("oauth.happy_path.ascii")?;
  let reader = reader::AsciiArtReader::new();
  let ast = reader.parse(&content);

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    sequence_diagram_fixtures::oauth_happy_path_elements()
  );

  Ok(())
}

#[test]
fn oauth_happy_path_compact() -> Result<()> {
  let content = read_example("oauth.happy_path.compact.ascii")?;
  let reader = reader::AsciiArtReader::new();
  let ast = reader.parse(&content);

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    sequence_diagram_fixtures::oauth_happy_path_elements()
  );

  Ok(())
}

#[test]
fn single_checked_state() -> Result<()> {
  let content = r"
     ┌─────┐      ┌─────┐
     │Alice│      │ Bob │
     └──┬──┘      └──┬──┘
    ╔═══╧════════════╧═══╗
    ║My State            ║
    ╚═══╤════════════╤═══╝
     ┌──┴──┐      ┌──┴──┐
     │Alice│      │ Bob │
     └─────┘      └─────┘
  ";
  let reader = reader::AsciiArtReader::new();
  let ast = reader.parse(content);

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    vec![SequenceDiagramElement::CheckedState {
      name: "My State".to_string(),
      participants: vec!["Alice".to_string(), "Bob".to_string()],
    }]
  );

  Ok(())
}

#[test]
fn service_discovery_flow() -> Result<()> {
  let content = read_example("service_discovery.ascii")?;
  let reader = reader::AsciiArtReader::new();
  let ast = reader.parse(&content);

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    sequence_diagram_fixtures::service_discovery_happy_path_elements(),
  );

  Ok(())
}

