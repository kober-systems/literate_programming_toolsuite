use anyhow::Result;
use ansicht::*;

#[test]
fn service_discovery_cucumber() -> Result<()> {
  let ast = reader::AsciiArtReader::new().parse(SERVICE_DISCOVERY_ASCII);

  let mut writer = writer::cucumber::CucumberWriter {
    feature_name: "Service Discovery".to_string(),
    perspective: "Client".to_string(),
  };

  let mut output = Vec::new();
  writer.write(ast, &mut output)?;

  let actual = String::from_utf8(output)?;
  assert_eq!(actual, SERVICE_DISCOVERY_FEATURE.replace('\r', ""));

  Ok(())
}

const SERVICE_DISCOVERY_ASCII: &str =
  include_str!("examples/sequence-diagram/service_discovery.ascii");

const SERVICE_DISCOVERY_FEATURE: &str =
  include_str!("examples/sequence-diagram/service_discovery.feature");
