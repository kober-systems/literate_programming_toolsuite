use anyhow::Result;
use ansicht::*;
use pretty_assertions::assert_eq;

#[test]
fn service_discovery_cucumber_reader() -> Result<()> {
  let content = SERVICE_DISCOVERY_FEATURE;
  let reader = reader::CucumberReader::new();
  let ast = reader.parse(content)?;

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    vec![
      SequenceDiagramElement::message("Client", "Device", "get_hashes"),
      SequenceDiagramElement::message("Device", "Client", "current hash"),
      SequenceDiagramElement::message("Client", "Device", "get_number_protocols"),
      SequenceDiagramElement::message("Device", "Client", "1"),
      SequenceDiagramElement::message("Client", "Device", "get_protocol_schema(0)"),
      SequenceDiagramElement::message("Device", "Client", "schema data"),
    ]
  );

  Ok(())
}

fn sequence_diagram_elements(input: Vec<ElementSpan>) -> Vec<SequenceDiagramElement> {
  input
    .into_iter()
    .map(|e| match e.element {
      Element::Sequence(e) => e,
      _ => panic!("element {:#?} is no sequence diagram element", e),
    })
    .collect()
}

const SERVICE_DISCOVERY_FEATURE: &str =
  include_str!("examples/sequence-diagram/service_discovery.feature");
