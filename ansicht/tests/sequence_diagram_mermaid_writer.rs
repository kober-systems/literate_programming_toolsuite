use anyhow::Result;
use ansicht::*;

#[test]
fn service_discovery_mermaid_writer() -> Result<()> {
  let ast = reader::AsciiArtReader::new().parse(SERVICE_DISCOVERY_ASCII);

  let mut writer = writer::mermaid::MermaidWriter;
  let mut output = Vec::new();
  writer.write(ast, &mut output)?;

  let actual = String::from_utf8(output)?;
  assert_eq!(actual, SERVICE_DISCOVERY_MERMAID);

  Ok(())
}

const SERVICE_DISCOVERY_ASCII: &str =
  include_str!("examples/sequence-diagram/service_discovery.ascii");

const SERVICE_DISCOVERY_MERMAID: &str =
  include_str!("examples/sequence-diagram/service_discovery.mermaid");
