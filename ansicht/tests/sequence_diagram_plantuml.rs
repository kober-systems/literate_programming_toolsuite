use anyhow::Result;
use ansicht::*;
use pretty_assertions::assert_eq;

#[test]
fn oauth_happy_path() -> Result<()> {
  let content = OAUTH_HAPPY_PATH_PLANTUML;
  let reader = reader::PlantUmlReader::new();
  let ast = reader.parse(content)?;

  let participants = vec![
    "End User".to_string(),
    "User's Browser".to_string(),
    "Client Application".to_string(),
    "Authorization Server".to_string(),
    "Resource Server".to_string(),
  ];

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    vec![
      SequenceDiagramElement::CheckedState {
        name: "Initial Redirect for Authorization".to_string(),
        participants: participants.clone(),
      },
      SequenceDiagramElement::message("End User", "User's Browser", "Request Access"),
      SequenceDiagramElement::message("User's Browser", "Client Application", "Request Access"),
      SequenceDiagramElement::message(
        "Client Application",
        "User's Browser",
        "Redirect to AuthServer (client_id, response_type=code, redirect_uri, scope)",
      ),
      SequenceDiagramElement::message("User's Browser", "Authorization Server", "Follow Redirect"),
      SequenceDiagramElement::CheckedState {
        name: "User Grants Consent".to_string(),
        participants: participants.clone(),
      },
      SequenceDiagramElement::message(
        "Authorization Server",
        "User's Browser",
        "Display Consent Form",
      ),
      SequenceDiagramElement::message("User's Browser", "End User", "Display Consent Form"),
      SequenceDiagramElement::message("End User", "User's Browser", "Grant Consent"),
      SequenceDiagramElement::message("User's Browser", "Authorization Server", "Grant Consent"),
      SequenceDiagramElement::message(
        "Authorization Server",
        "User's Browser",
        "Redirect with Authorization Code",
      ),
      SequenceDiagramElement::message(
        "User's Browser",
        "Client Application",
        "Follow Redirect with Code",
      ),
      SequenceDiagramElement::CheckedState {
        name: "Token Exchange and Resource Access".to_string(),
        participants: participants.clone(),
      },
      SequenceDiagramElement::message(
        "Client Application",
        "Authorization Server",
        "Exchange Code for Token",
      ),
      SequenceDiagramElement::message(
        "Authorization Server",
        "Client Application",
        "Respond with Access Token",
      ),
      SequenceDiagramElement::message(
        "Client Application",
        "Resource Server",
        "Request Protected Resource (with Access Token)",
      ),
      SequenceDiagramElement::message(
        "Resource Server",
        "Client Application",
        "Respond with Protected Resource",
      ),
      SequenceDiagramElement::message("Client Application", "User's Browser", "Display Resource"),
      SequenceDiagramElement::message("User's Browser", "End User", "Display Resource"),
    ]
  );

  Ok(())
}

#[test]
fn service_discovery_happy_path() -> Result<()> {
  let content = SERVICE_DISCOVERY_PLANTUML;
  let reader = reader::PlantUmlReader::new();
  let ast = reader.parse(content)?;

  assert_eq!(
    sequence_diagram_elements(ast.elements),
    vec![
      SequenceDiagramElement::CheckedState {
        name: "Service Discovery".to_string(),
        participants: vec!["Client".to_string(), "Device".to_string(),],
      },
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

const SERVICE_DISCOVERY_PLANTUML: &str =
  include_str!("examples/sequence-diagram/service_discovery.plantuml");

const OAUTH_HAPPY_PATH_PLANTUML: &str =
  include_str!("examples/sequence-diagram/oauth.happy_path.plantuml");
