use anyhow::Result;
use ansicht::*;

mod sequence_diagram_fixtures;
use sequence_diagram_fixtures::read_example;

#[test]
fn service_discovery_mermaid_writer() -> Result<()> {
  let content = read_example("service_discovery.ascii")?;
  let ast = reader::AsciiArtReader::new().parse(&content);

  let mut writer = writer::mermaid::MermaidWriter;
  let mut output = Vec::new();
  writer.write(ast, &mut output)?;

  let actual = String::from_utf8(output)?;
  assert_eq!(actual, read_example("service_discovery.mermaid")?.replace('\r', ""));

  Ok(())
}

