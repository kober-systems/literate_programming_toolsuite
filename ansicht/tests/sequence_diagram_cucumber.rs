use anyhow::Result;
use ansicht::*;
use pretty_assertions::assert_eq;

mod sequence_diagram_fixtures;
use sequence_diagram_fixtures::read_example;
mod test_helpers;
use test_helpers::sequence_diagram_elements;

#[test]
fn service_discovery_cucumber() -> Result<()> {
  let content = read_example("service_discovery.ascii")?;
  let ast = reader::AsciiArtReader::new().parse(&content);

  let mut writer = writer::cucumber::CucumberWriter {
    feature_name: "Service Discovery".to_string(),
    perspective: "Device".to_string(),
  };

  let mut output = Vec::new();
  writer.write(ast, &mut output)?;

  let actual = String::from_utf8(output)?;
  assert_eq!(actual, read_example("service_discovery.feature")?.replace('\r', ""));

  Ok(())
}

#[test]
fn oauth_happy_path_cucumber() -> Result<()> {
  let content = read_example("oauth.happy_path.mermaid")?;
  let ast = reader::MermaidReader::new().parse(&content)?;

  let mut writer = writer::cucumber::CucumberWriter {
    feature_name: "OAuth happy path".to_string(),
    perspective: "User's Browser".to_string(),
  };

  let mut output = Vec::new();
  writer.write(ast, &mut output)?;

  let actual = String::from_utf8(output)?;
  assert_eq!(actual, read_example("oauth.happy_path.feature")?.replace('\r', ""));

  Ok(())
}

