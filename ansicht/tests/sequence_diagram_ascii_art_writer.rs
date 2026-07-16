use anyhow::Result;
use ansicht::*;
use pretty_assertions::assert_eq;

#[test]
fn oauth_happy_path_ascii_art_writer() -> Result<()> {
  let ast = reader::PlantUmlReader::new().parse(OAUTH_HAPPY_PATH_PLANTUML)?;

  let mut writer = writer::ascii_art::AsciiArtWriter::new();
  let mut output = Vec::new();
  writer.write(ast, &mut output)?;

  let actual = String::from_utf8(output)?.replace('\r', "");
  assert_eq!(actual, OAUTH_HAPPY_PATH_ASCII.replace('\r', ""));

  Ok(())
}

#[test]
fn service_discovery_ascii_art_writer() -> Result<()> {
  let ast = reader::CucumberReader::new().parse(SERVICE_DISCOVERY_FEATURE)?;

  let mut writer = writer::ascii_art::AsciiArtWriter::new();
  let mut output = Vec::new();
  writer.write(ast, &mut output)?;

  let actual = String::from_utf8(output)?.replace('\r', "");

  // Verify structure rather than exact formatting
  // Should contain box drawing characters
  assert!(actual.contains("┌"), "Should contain top-left corner");
  assert!(actual.contains("┐"), "Should contain top-right corner");
  assert!(actual.contains("└"), "Should contain bottom-left corner");
  assert!(actual.contains("┘"), "Should contain bottom-right corner");
  assert!(actual.contains("│"), "Should contain vertical lines");

  // Should contain participant names
  assert!(
    actual.contains("Client"),
    "Should contain Client participant"
  );
  assert!(
    actual.contains("Device"),
    "Should contain Device participant"
  );

  // Should contain messages
  assert!(
    actual.contains("get_hashes"),
    "Should contain first message"
  );
  assert!(
    actual.contains("current hash"),
    "Should contain response message"
  );
  assert!(
    actual.contains("schema data"),
    "Should contain last message"
  );

  // Should have arrows
  assert!(actual.contains("─"), "Should contain horizontal lines");
  assert!(
    actual.contains(">") || actual.contains("<"),
    "Should contain arrow markers"
  );

  // Verify formatting
  assert_eq!(actual, SERVICE_DISCOVERY_ASCII.replace('\r', ""));

  Ok(())
}

const SERVICE_DISCOVERY_ASCII: &str =
  include_str!("examples/sequence-diagram/service_discovery.ascii");

const SERVICE_DISCOVERY_FEATURE: &str =
  include_str!("examples/sequence-diagram/service_discovery.feature");

const OAUTH_HAPPY_PATH_PLANTUML: &str =
  include_str!("examples/sequence-diagram/oauth.happy_path.plantuml");

const OAUTH_HAPPY_PATH_ASCII: &str =
  include_str!("examples/sequence-diagram/oauth.happy_path.ascii");
