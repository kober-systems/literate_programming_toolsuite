use anyhow::Result;
use ansicht::*;
use pretty_assertions::assert_eq;

#[ignore]
#[test]
fn oauth_happy_path() -> Result<()> {
  let content = OAUTH_HAPPY_PATH_ASCII;
  let reader = reader::AsciiArtReader::new();
  let ast = reader.parse(content);

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    vec![
      SequenceDiagramElement::CheckedState {
        name: "Initial Redirect for Authorization".to_string(),
        participants: vec![
          "End User".to_string(),
          "User's Browser".to_string(),
          "Client Application".to_string(),
          "Authorization Server".to_string(),
          "Resource Server".to_string(),
        ],
      },
      SequenceDiagramElement::message("End User", "User's Browser", "Request Access",),
      SequenceDiagramElement::message("User's Browser", "Client Application", "Request Access",),
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

const OAUTH_HAPPY_PATH_ASCII: &str =
  include_str!("examples/sequence-diagram/oauth.happy_path.ascii");

#[test]
fn service_discovery_flow() -> Result<()> {
  let content = SERVICE_DISCOVERY_ASCII;
  let reader = reader::AsciiArtReader::new();
  let ast = reader.parse(content);

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

const SERVICE_DISCOVERY_ASCII: &str =
  include_str!("examples/sequence-diagram/service_discovery.ascii");
