#![allow(dead_code)]

use anyhow::Result;
use ansicht::*;
use ansicht::reader::ascii_art::tokenizer::{sort_tokens, Token};
use std::fs;
use std::path::PathBuf;

pub fn service_discovery_happy_path_elements() -> Vec<SequenceDiagramElement> {
  vec![
    SequenceDiagramElement::message("Client", "Device", "get_hashes"),
    SequenceDiagramElement::message("Device", "Client", "current hash"),
    SequenceDiagramElement::message("Client", "Device", "get_number_protocols"),
    SequenceDiagramElement::message("Device", "Client", "1"),
    SequenceDiagramElement::message("Client", "Device", "get_protocol_schema(0)"),
    SequenceDiagramElement::message("Device", "Client", "schema data"),
  ]
}

pub fn oauth_happy_path_elements() -> Vec<SequenceDiagramElement> {
  let participants = vec![
    "End User".to_string(),
    "User's Browser".to_string(),
    "Client Application".to_string(),
    "Authorization Server".to_string(),
    "Resource Server".to_string(),
  ];

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
}

pub fn oauth_happy_path_tokens() -> Vec<Token> {
  use Token::*;

  let tokens = [
    block_end_user_border(), block_end_user_text(),
    block_users_browser_border(), block_users_browser_text(),
    block_client_application_border(), block_client_application_text(),
    block_authorisation_server_border(), block_authorisation_server_text(),
    block_resource_server_border(), block_resource_server_text(),
    block_state1_initial_redirect_border(), block_state1_initial_redirect_text(),
    con_state1_state2_lifeline_enduser(),
    con_state1_state2_lifeline_users_browser(),
    con_state1_state2_lifeline_client_application(),
    con_state1_state2_lifeline_authorisation_server(),
    con_state1_state2_lifeline_resource_server(),
    con_enduser_users_browser_msg_request_access_text(),
    con_enduser_users_browser_msg_request_access(),
    con_users_browser_client_application_msg_request_access_text(),
    con_users_browser_client_application_msg_request_access(),
    con_client_application_users_browser_msg_redirect_to_authserver_text(),
    con_client_application_users_browser_msg_redirect_to_authserver(),
    con_users_browser_authorisation_server_msg_follow_redirect_text(),
    con_users_browser_authorisation_server_msg_follow_redirect(),
    block_state2_user_grants_consent_border(), block_state2_user_grants_consent_text(),
    con_state2_state3_lifeline_enduser(),
    con_state2_state3_lifeline_users_browser(),
    con_state2_state3_lifeline_client_application(),
    con_state2_state3_lifeline_authorisation_server(),
    con_state2_state3_lifeline_resource_server(),
    con_client_application_users_browser_msg_display_consent_form_text(),
    con_client_application_users_browser_msg_display_consent_form(),
    con_users_browser_enduser_msg_display_consent_form_text(),
    con_users_browser_enduser_msg_display_consent_form(),
    con_enduser_users_browser_msg_grant_consent_text(),
    con_enduser_users_browser_msg_grant_consent(),
    con_users_browser_authorisation_server_msg_grant_consent_text(),
    con_users_browser_authorisation_server_msg_grant_consent(),
    con_authorisation_server_users_browser_msg_redirect_authorisation_code_text(),
    con_authorisation_server_users_browser_msg_redirect_authorisation_code(),
    con_users_browser_client_application_msg_follow_redirect_text(),
    con_users_browser_client_application_msg_follow_redirect(),
    block_state3_token_exchange_border(), block_state3_token_exchange_text(),
    con_state3_state4_lifeline_enduser(),
    con_state3_state4_lifeline_users_browser(),
    con_state3_state4_lifeline_client_application(),
    con_state3_state4_lifeline_authorisation_server(),
    con_state3_state4_lifeline_resource_server(),
    con_client_application_authorisation_server_msg_exchange_code_text(),
    con_client_application_authorisation_server_msg_exchange_code(),
    con_authorisation_server_client_application_msg_respond_with_access_token_text(),
    con_authorisation_server_client_application_msg_respond_with_access_token(),
    con_client_application_resource_server_msg_request_resource_text(),
    con_client_application_resource_server_msg_request_resource(),
    con_resource_server_client_application_msg_send_resource_text(),
    con_resource_server_client_application_msg_send_resource(),
    con_client_application_users_browser_msg_display_resource_text(),
    con_client_application_users_browser_msg_display_resource(),
    con_users_browser_enduser_msg_display_resource_text(),
    con_users_browser_enduser_msg_display_resource(),
    block_end_user_border2(), block_end_user_text2(),
    block_users_browser_border2(), block_users_browser_text2(),
    block_client_application_border2(), block_client_application_text2(),
    block_authorisation_server_border2(), block_authorisation_server_text2(),
    block_resource_server_border2(), block_resource_server_text2(),
  ].concat();
  sort_tokens(tokens)
}

pub fn block_end_user_border() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 0,
        column: 5,
    },
    HLine {
        line: 0,
        column_start: 6,
        column_end: 13,
    },
    ConnectionSign {
        line: 0,
        column: 14,
    },
    VLine {
        column: 5,
        line_start: 1,
        line_end: 1,
    },
    VLine {
        column: 14,
        line_start: 1,
        line_end: 1,
    },
    ConnectionSign {
        line: 2,
        column: 5,
    },
    HLine {
        line: 2,
        column_start: 6,
        column_end: 8,
    },
    ConnectionSign {
        line: 2,
        column: 9,
    },
    HLine {
        line: 2,
        column_start: 10,
        column_end: 13,
    },
    ConnectionSign {
        line: 2,
        column: 14,
    },
  ]
}

pub fn block_end_user_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 1,
        column_start: 6,
        column_end: 13,
    },
  ]
}

pub fn block_users_browser_border() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 0,
        column: 23,
    },
    HLine {
        line: 0,
        column_start: 24,
        column_end: 37,
    },
    ConnectionSign {
        line: 0,
        column: 38,
    },
    VLine {
        column: 23,
        line_start: 1,
        line_end: 1,
    },
    VLine {
        column: 38,
        line_start: 1,
        line_end: 1,
    },
    ConnectionSign {
        line: 2,
        column: 23,
    },
    HLine {
        line: 2,
        column_start: 24,
        column_end: 29,
    },
    ConnectionSign {
        line: 2,
        column: 30,
    },
    HLine {
        line: 2,
        column_start: 31,
        column_end: 37,
    },
    ConnectionSign {
        line: 2,
        column: 38,
    },
  ]
}

pub fn block_users_browser_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 1,
        column_start: 24,
        column_end: 37,
    },
  ]
}

pub fn block_client_application_border() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 0,
        column: 97,
    },
    HLine {
        line: 0,
        column_start: 98,
        column_end: 115,
    },
    ConnectionSign {
        line: 0,
        column: 116,
    },
    VLine {
        column: 97,
        line_start: 1,
        line_end: 1,
    },
    VLine {
        column: 116,
        line_start: 1,
        line_end: 1,
    },
    ConnectionSign {
        line: 2,
        column: 97,
    },
    HLine {
        line: 2,
        column_start: 98,
        column_end: 105,
    },
    ConnectionSign {
        line: 2,
        column: 106,
    },
    HLine {
        line: 2,
        column_start: 107,
        column_end: 115,
    },
    ConnectionSign {
        line: 2,
        column: 116,
    },
  ]
}

pub fn block_client_application_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 1,
        column_start: 98,
        column_end: 115,
    },
  ]
}

pub fn block_authorisation_server_border() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 0,
        column: 122,
    },
    HLine {
        line: 0,
        column_start: 123,
        column_end: 142,
    },
    ConnectionSign {
        line: 0,
        column: 143,
    },
    VLine {
        column: 122,
        line_start: 1,
        line_end: 1,
    },
    VLine {
        column: 143,
        line_start: 1,
        line_end: 1,
    },
    ConnectionSign {
        line: 2,
        column: 122,
    },
    HLine {
        line: 2,
        column_start: 123,
        column_end: 131,
    },
    ConnectionSign {
        line: 2,
        column: 132,
    },
    HLine {
        line: 2,
        column_start: 133,
        column_end: 142,
    },
    ConnectionSign {
        line: 2,
        column: 143,
    },
  ]
}

pub fn block_authorisation_server_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 1,
        column_start: 123,
        column_end: 142,
    },
  ]
}

pub fn block_resource_server_border() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 0,
        column: 147,
    },
    HLine {
        line: 0,
        column_start: 148,
        column_end: 162,
    },
    ConnectionSign {
        line: 0,
        column: 163,
    },
    VLine {
        column: 147,
        line_start: 1,
        line_end: 1,
    },
    VLine {
        column: 163,
        line_start: 1,
        line_end: 1,
    },
    ConnectionSign {
        line: 2,
        column: 147,
    },
    HLine {
        line: 2,
        column_start: 148,
        column_end: 153,
    },
    ConnectionSign {
        line: 2,
        column: 154,
    },
    HLine {
        line: 2,
        column_start: 155,
        column_end: 162,
    },
    ConnectionSign {
        line: 2,
        column: 163,
    },
  ]
}

pub fn block_resource_server_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 1,
        column_start: 148,
        column_end: 162,
    },
  ]
}

pub fn block_state1_initial_redirect_border() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 3,
        column: 5,
    },
    HLine {
        line: 3,
        column_start: 6,
        column_end: 8,
    },
    ConnectionSign {
        line: 3,
        column: 9,
    },
    HLine {
        line: 3,
        column_start: 10,
        column_end: 29,
    },
    ConnectionSign {
        line: 3,
        column: 30,
    },
    HLine {
        line: 3,
        column_start: 31,
        column_end: 105,
    },
    ConnectionSign {
        line: 3,
        column: 106,
    },
    HLine {
        line: 3,
        column_start: 107,
        column_end: 131,
    },
    ConnectionSign {
        line: 3,
        column: 132,
    },
    HLine {
        line: 3,
        column_start: 133,
        column_end: 153,
    },
    ConnectionSign {
        line: 3,
        column: 154,
    },
    HLine {
        line: 3,
        column_start: 155,
        column_end: 162,
    },
    ConnectionSign {
        line: 3,
        column: 163,
    },
    VLine {
        column: 5,
        line_start: 4,
        line_end: 4,
    },
    VLine {
        column: 163,
        line_start: 4,
        line_end: 4,
    },
    ConnectionSign {
        line: 5,
        column: 5,
    },
    HLine {
        line: 5,
        column_start: 6,
        column_end: 8,
    },
    ConnectionSign {
        line: 5,
        column: 9,
    },
    HLine {
        line: 5,
        column_start: 10,
        column_end: 29,
    },
    ConnectionSign {
        line: 5,
        column: 30,
    },
    HLine {
        line: 5,
        column_start: 31,
        column_end: 105,
    },
    ConnectionSign {
        line: 5,
        column: 106,
    },
    HLine {
        line: 5,
        column_start: 107,
        column_end: 131,
    },
    ConnectionSign {
        line: 5,
        column: 132,
    },
    HLine {
        line: 5,
        column_start: 133,
        column_end: 153,
    },
    ConnectionSign {
        line: 5,
        column: 154,
    },
    HLine {
        line: 5,
        column_start: 155,
        column_end: 162,
    },
    ConnectionSign {
        line: 5,
        column: 163,
    },
  ]
}

pub fn block_state1_initial_redirect_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 4,
        column_start: 6,
        column_end: 39,
    },
  ]
}

pub fn con_state1_state2_lifeline_enduser() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 9,
        line_start: 6,
        line_end: 18,
    },
  ]
}

pub fn con_state1_state2_lifeline_users_browser() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 30,
        line_start: 6,
        line_end: 18,
    },
  ]
}

pub fn con_state1_state2_lifeline_client_application() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 106,
        line_start: 6,
        line_end: 16,
    },
    VLine {
        column: 106,
        line_start: 18,
        line_end: 18,
    },
  ]
}

pub fn con_state1_state2_lifeline_authorisation_server() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 132,
        line_start: 6,
        line_end: 18,
    },
  ]
}

pub fn con_state1_state2_lifeline_resource_server() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 154,
        line_start: 6,
        line_end: 18,
    },
  ]
}

pub fn con_enduser_users_browser_msg_request_access_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 7,
        column_start: 13,
        column_end: 26,
    },
  ]
}

pub fn con_enduser_users_browser_msg_request_access() -> Vec<Token> {
  use Token::*;

  vec![
    HLine {
        line: 8,
        column_start: 10,
        column_end: 28,
    },
    Arrow {
        line: 8,
        column: 29,
    },
  ]
}

pub fn con_users_browser_client_application_msg_request_access_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 10,
        column_start: 62,
        column_end: 75,
    },
  ]
}

pub fn con_users_browser_client_application_msg_request_access() -> Vec<Token> {
  use Token::*;

  vec![
    HLine {
        line: 11,
        column_start: 31,
        column_end: 104,
    },
    Arrow {
        line: 11,
        column: 105,
    },
  ]
}

pub fn con_client_application_users_browser_msg_redirect_to_authserver_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 13,
        column_start: 31,
        column_end: 78,
    },
    HLine {
        line: 13,
        column_start: 79,
        column_end: 79,
    },
    Text {
        line: 13,
        column_start: 80,
        column_end: 105,
    },
  ]
}

pub fn con_client_application_users_browser_msg_redirect_to_authserver() -> Vec<Token> {
  use Token::*;

  vec![
    Arrow {
        line: 14,
        column: 31,
    },
    HLine {
        line: 14,
        column_start: 32,
        column_end: 104,
    },
  ]
}

pub fn con_users_browser_authorisation_server_msg_follow_redirect_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 16,
        column_start: 74,
        column_end: 88,
    },
  ]
}

pub fn con_users_browser_authorisation_server_msg_follow_redirect() -> Vec<Token> {
  use Token::*;

  vec![
    HLine {
        line: 17,
        column_start: 31,
        column_end: 130,
    },
    Arrow {
        line: 17,
        column: 131,
    },
  ]
}

pub fn con_authorization_server_users_browser_msg_display_consent_form_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 23,
        column_start: 72,
        column_end: 91,
    },
  ]
}

pub fn con_authorization_server_users_browser_msg_display_consent_form() -> Vec<Token> {
  use Token::*;

  vec![
    Arrow {
        line: 24,
        column: 31,
    },
    HLine {
        line: 24,
        column_start: 32,
        column_end: 130,
    },
  ]
}

pub fn con_users_browser_end_user_msg_display_consent_form_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 26,
        column_start: 10,
        column_end: 29,
    },
  ]
}

pub fn con_users_browser_end_user_msg_display_consent_form() -> Vec<Token> {
  use Token::*;

  vec![
    Arrow {
        line: 27,
        column: 10,
    },
    HLine {
        line: 27,
        column_start: 11,
        column_end: 29,
    },
  ]
}

pub fn con_end_user_users_browser_msg_grant_consent_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 29,
        column_start: 14,
        column_end: 26,
    },
  ]
}

pub fn con_end_user_users_browser_msg_grant_consent() -> Vec<Token> {
  use Token::*;

  vec![
    HLine {
        line: 30,
        column_start: 10,
        column_end: 28,
    },
    Arrow {
        line: 30,
        column: 29,
    },
  ]
}

pub fn block_state2_user_grants_consent_border() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 19,
        column: 5,
    },
    HLine {
        line: 19,
        column_start: 6,
        column_end: 8,
    },
    ConnectionSign {
        line: 19,
        column: 9,
    },
    HLine {
        line: 19,
        column_start: 10,
        column_end: 29,
    },
    ConnectionSign {
        line: 19,
        column: 30,
    },
    HLine {
        line: 19,
        column_start: 31,
        column_end: 105,
    },
    ConnectionSign {
        line: 19,
        column: 106,
    },
    HLine {
        line: 19,
        column_start: 107,
        column_end: 131,
    },
    ConnectionSign {
        line: 19,
        column: 132,
    },
    HLine {
        line: 19,
        column_start: 133,
        column_end: 153,
    },
    ConnectionSign {
        line: 19,
        column: 154,
    },
    HLine {
        line: 19,
        column_start: 155,
        column_end: 162,
    },
    ConnectionSign {
        line: 19,
        column: 163,
    },
    VLine {
        column: 5,
        line_start: 20,
        line_end: 20,
    },
    VLine {
        column: 163,
        line_start: 20,
        line_end: 20,
    },
    ConnectionSign {
        line: 21,
        column: 5,
    },
    HLine {
        line: 21,
        column_start: 6,
        column_end: 8,
    },
    ConnectionSign {
        line: 21,
        column: 9,
    },
    HLine {
        line: 21,
        column_start: 10,
        column_end: 29,
    },
    ConnectionSign {
        line: 21,
        column: 30,
    },
    HLine {
        line: 21,
        column_start: 31,
        column_end: 105,
    },
    ConnectionSign {
        line: 21,
        column: 106,
    },
    HLine {
        line: 21,
        column_start: 107,
        column_end: 131,
    },
    ConnectionSign {
        line: 21,
        column: 132,
    },
    HLine {
        line: 21,
        column_start: 133,
        column_end: 153,
    },
    ConnectionSign {
        line: 21,
        column: 154,
    },
    HLine {
        line: 21,
        column_start: 155,
        column_end: 162,
    },
    ConnectionSign {
        line: 21,
        column: 163,
    },
  ]
}

pub fn block_state2_user_grants_consent_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 20,
        column_start: 6,
        column_end: 24,
    },
  ]
}

pub fn con_state2_state3_lifeline_enduser() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 9,
        line_start: 22,
        line_end: 40,
    },
  ]
}

pub fn con_state2_state3_lifeline_users_browser() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 30,
        line_start: 22,
        line_end: 40,
    },
  ]
}

pub fn con_state2_state3_lifeline_client_application() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 106,
        line_start: 22,
        line_end: 23,
    },
    VLine {
        column: 106,
        line_start: 25,
        line_end: 32,
    },
    VLine {
        column: 106,
        line_start: 34,
        line_end: 35,
    },
    VLine {
        column: 106,
        line_start: 37,
        line_end: 40,
    },
  ]
}

pub fn con_state2_state3_lifeline_authorisation_server() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 132,
        line_start: 22,
        line_end: 40,
    },
  ]
}

pub fn con_state2_state3_lifeline_resource_server() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 154,
        line_start: 22,
        line_end: 40,
    },
  ]
}

pub fn con_client_application_users_browser_msg_display_consent_form_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 23,
        column_start: 72,
        column_end: 91,
    },
  ]
}

pub fn con_client_application_users_browser_msg_display_consent_form() -> Vec<Token> {
  use Token::*;

  vec![
    Arrow {
        line: 24,
        column: 31,
    },
    HLine {
        line: 24,
        column_start: 32,
        column_end: 130,
    },
  ]
}

pub fn con_users_browser_enduser_msg_display_consent_form_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 26,
        column_start: 10,
        column_end: 29,
    },
  ]
}

pub fn con_users_browser_enduser_msg_display_consent_form() -> Vec<Token> {
  use Token::*;

  vec![
    Arrow {
        line: 27,
        column: 10,
    },
    HLine {
        line: 27,
        column_start: 11,
        column_end: 29,
    },
  ]
}

pub fn con_enduser_users_browser_msg_grant_consent_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 29,
        column_start: 14,
        column_end: 26,
    },
  ]
}

pub fn con_enduser_users_browser_msg_grant_consent() -> Vec<Token> {
  use Token::*;

  vec![
    HLine {
        line: 30,
        column_start: 10,
        column_end: 28,
    },
    Arrow {
        line: 30,
        column: 29,
    },
  ]
}

pub fn con_users_browser_authorisation_server_msg_grant_consent_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 32,
        column_start: 75,
        column_end: 87,
    },
  ]
}

pub fn con_users_browser_authorisation_server_msg_grant_consent() -> Vec<Token> {
  use Token::*;

  vec![
    HLine {
        line: 33,
        column_start: 31,
        column_end: 130,
    },
    Arrow {
        line: 33,
        column: 131,
    },
  ]
}

pub fn con_authorisation_server_users_browser_msg_redirect_authorisation_code_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 35,
        column_start: 66,
        column_end: 97,
    },
  ]
}

pub fn con_authorisation_server_users_browser_msg_redirect_authorisation_code() -> Vec<Token> {
  use Token::*;

  vec![
    Arrow {
        line: 36,
        column: 31,
    },
    HLine {
        line: 36,
        column_start: 32,
        column_end: 130,
    },
  ]
}

pub fn con_users_browser_client_application_msg_follow_redirect_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 38,
        column_start: 56,
        column_end: 80,
    },
  ]
}

pub fn con_users_browser_client_application_msg_follow_redirect() -> Vec<Token> {
  use Token::*;

  vec![
    HLine {
        line: 39,
        column_start: 31,
        column_end: 104,
    },
    Arrow {
        line: 39,
        column: 105,
    },
  ]
}

pub fn block_state3_token_exchange_border() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 41,
        column: 5,
    },
    HLine {
        line: 41,
        column_start: 6,
        column_end: 8,
    },
    ConnectionSign {
        line: 41,
        column: 9,
    },
    HLine {
        line: 41,
        column_start: 10,
        column_end: 29,
    },
    ConnectionSign {
        line: 41,
        column: 30,
    },
    HLine {
        line: 41,
        column_start: 31,
        column_end: 105,
    },
    ConnectionSign {
        line: 41,
        column: 106,
    },
    HLine {
        line: 41,
        column_start: 107,
        column_end: 131,
    },
    ConnectionSign {
        line: 41,
        column: 132,
    },
    HLine {
        line: 41,
        column_start: 133,
        column_end: 153,
    },
    ConnectionSign {
        line: 41,
        column: 154,
    },
    HLine {
        line: 41,
        column_start: 155,
        column_end: 162,
    },
    ConnectionSign {
        line: 41,
        column: 163,
    },
    VLine {
        column: 5,
        line_start: 42,
        line_end: 42,
    },
    VLine {
        column: 163,
        line_start: 42,
        line_end: 42,
    },
    ConnectionSign {
        line: 43,
        column: 5,
    },
    HLine {
        line: 43,
        column_start: 6,
        column_end: 8,
    },
    ConnectionSign {
        line: 43,
        column: 9,
    },
    HLine {
        line: 43,
        column_start: 10,
        column_end: 29,
    },
    ConnectionSign {
        line: 43,
        column: 30,
    },
    HLine {
        line: 43,
        column_start: 31,
        column_end: 105,
    },
    ConnectionSign {
        line: 43,
        column: 106,
    },
    HLine {
        line: 43,
        column_start: 107,
        column_end: 131,
    },
    ConnectionSign {
        line: 43,
        column: 132,
    },
    HLine {
        line: 43,
        column_start: 133,
        column_end: 153,
    },
    ConnectionSign {
        line: 43,
        column: 154,
    },
    HLine {
        line: 43,
        column_start: 155,
        column_end: 162,
    },
    ConnectionSign {
        line: 43,
        column: 163,
    },
  ]
}

pub fn block_state3_token_exchange_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 42,
        column_start: 6,
        column_end: 39,
    },
  ]
}

pub fn con_state3_state4_lifeline_enduser() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 9,
        line_start: 44,
        line_end: 61,
    },
  ]
}

pub fn con_state3_state4_lifeline_users_browser() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 30,
        line_start: 44,
        line_end: 61,
    },
  ]
}

pub fn con_state3_state4_lifeline_client_application() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 106,
        line_start: 44,
        line_end: 61,
    },
  ]
}

pub fn con_state3_state4_lifeline_authorisation_server() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 132,
        line_start: 44,
        line_end: 50,
    },
    VLine {
        column: 132,
        line_start: 53,
        line_end: 53,
    },
    VLine {
        column: 132,
        line_start: 56,
        line_end: 61,
    },
  ]
}

pub fn con_state3_state4_lifeline_resource_server() -> Vec<Token> {
  use Token::*;

  vec![
    VLine {
        column: 154,
        line_start: 44,
        line_end: 61,
    },
  ]
}

pub fn con_client_application_authorisation_server_msg_exchange_code_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 45,
        column_start: 108,
        column_end: 130,
    },
  ]
}

pub fn con_client_application_authorisation_server_msg_exchange_code() -> Vec<Token> {
  use Token::*;

  vec![
    HLine {
        line: 46,
        column_start: 107,
        column_end: 130,
    },
    Arrow {
        line: 46,
        column: 131,
    },
  ]
}

pub fn con_authorisation_server_client_application_msg_respond_with_access_token_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 48,
        column_start: 107,
        column_end: 131,
    },
  ]
}

pub fn con_authorisation_server_client_application_msg_respond_with_access_token() -> Vec<Token> {
  use Token::*;

  vec![
    Arrow {
        line: 49,
        column: 107,
    },
    HLine {
        line: 49,
        column_start: 108,
        column_end: 130,
    },
  ]
}

pub fn con_client_application_resource_server_msg_request_resource_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 51,
        column_start: 108,
        column_end: 153,
    },
  ]
}

pub fn con_client_application_resource_server_msg_request_resource() -> Vec<Token> {
  use Token::*;

  vec![
    HLine {
        line: 52,
        column_start: 107,
        column_end: 152,
    },
    Arrow {
        line: 52,
        column: 153,
    },
  ]
}

pub fn con_resource_server_client_application_msg_send_resource_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 54,
        column_start: 115,
        column_end: 145,
    },
  ]
}

pub fn con_resource_server_client_application_msg_send_resource() -> Vec<Token> {
  use Token::*;

  vec![
    Arrow {
        line: 55,
        column: 107,
    },
    HLine {
        line: 55,
        column_start: 108,
        column_end: 152,
    },
  ]
}

pub fn con_client_application_users_browser_msg_display_resource_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 57,
        column_start: 61,
        column_end: 76,
    },
  ]
}

pub fn con_client_application_users_browser_msg_display_resource() -> Vec<Token> {
  use Token::*;

  vec![
    Arrow {
        line: 58,
        column: 31,
    },
    HLine {
        line: 58,
        column_start: 32,
        column_end: 104,
    },
  ]
}

pub fn con_users_browser_enduser_msg_display_resource_text() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 60,
        column_start: 12,
        column_end: 27,
    },
  ]
}

pub fn con_users_browser_enduser_msg_display_resource() -> Vec<Token> {
  use Token::*;

  vec![
    Arrow {
        line: 61,
        column: 10,
    },
    HLine {
        line: 61,
        column_start: 11,
        column_end: 29,
    },
  ]
}

pub fn block_end_user_border2() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 62,
        column: 5,
    },
    HLine {
        line: 62,
        column_start: 6,
        column_end: 8,
    },
    ConnectionSign {
        line: 62,
        column: 9,
    },
    HLine {
        line: 62,
        column_start: 10,
        column_end: 13,
    },
    ConnectionSign {
        line: 62,
        column: 14,
    },
    VLine {
        column: 5,
        line_start: 63,
        line_end: 63,
    },
    VLine {
        column: 14,
        line_start: 63,
        line_end: 63,
    },
    ConnectionSign {
        line: 64,
        column: 5,
    },
    HLine {
        line: 64,
        column_start: 6,
        column_end: 13,
    },
    ConnectionSign {
        line: 64,
        column: 14,
    },
  ]
}

pub fn block_end_user_text2() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 63,
        column_start: 6,
        column_end: 13,
    },
  ]
}

pub fn block_users_browser_border2() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 62,
        column: 23,
    },
    HLine {
        line: 62,
        column_start: 24,
        column_end: 29,
    },
    ConnectionSign {
        line: 62,
        column: 30,
    },
    HLine {
        line: 62,
        column_start: 31,
        column_end: 37,
    },
    ConnectionSign {
        line: 62,
        column: 38,
    },
    VLine {
        column: 23,
        line_start: 63,
        line_end: 63,
    },
    VLine {
        column: 38,
        line_start: 63,
        line_end: 63,
    },
    ConnectionSign {
        line: 64,
        column: 23,
    },
    HLine {
        line: 64,
        column_start: 24,
        column_end: 37,
    },
    ConnectionSign {
        line: 64,
        column: 38,
    },
  ]
}

pub fn block_users_browser_text2() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 63,
        column_start: 24,
        column_end: 37,
    },
  ]
}

pub fn block_client_application_border2() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 62,
        column: 97,
    },
    HLine {
        line: 62,
        column_start: 98,
        column_end: 105,
    },
    ConnectionSign {
        line: 62,
        column: 106,
    },
    HLine {
        line: 62,
        column_start: 107,
        column_end: 115,
    },
    ConnectionSign {
        line: 62,
        column: 116,
    },
    VLine {
        column: 97,
        line_start: 63,
        line_end: 63,
    },
    VLine {
        column: 116,
        line_start: 63,
        line_end: 63,
    },
    ConnectionSign {
        line: 64,
        column: 97,
    },
    HLine {
        line: 64,
        column_start: 98,
        column_end: 115,
    },
    ConnectionSign {
        line: 64,
        column: 116,
    },
  ]
}

pub fn block_client_application_text2() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 63,
        column_start: 98,
        column_end: 115,
    },
  ]
}

pub fn block_authorisation_server_border2() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 62,
        column: 122,
    },
    HLine {
        line: 62,
        column_start: 123,
        column_end: 131,
    },
    ConnectionSign {
        line: 62,
        column: 132,
    },
    HLine {
        line: 62,
        column_start: 133,
        column_end: 142,
    },
    ConnectionSign {
        line: 62,
        column: 143,
    },
    VLine {
        column: 122,
        line_start: 63,
        line_end: 63,
    },
    VLine {
        column: 143,
        line_start: 63,
        line_end: 63,
    },
    ConnectionSign {
        line: 64,
        column: 122,
    },
    HLine {
        line: 64,
        column_start: 123,
        column_end: 142,
    },
    ConnectionSign {
        line: 64,
        column: 143,
    },
  ]
}

pub fn block_authorisation_server_text2() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 63,
        column_start: 123,
        column_end: 142,
    },
  ]
}

pub fn block_resource_server_border2() -> Vec<Token> {
  use Token::*;

  vec![
    ConnectionSign {
        line: 62,
        column: 147,
    },
    HLine {
        line: 62,
        column_start: 148,
        column_end: 153,
    },
    ConnectionSign {
        line: 62,
        column: 154,
    },
    HLine {
        line: 62,
        column_start: 155,
        column_end: 162,
    },
    ConnectionSign {
        line: 62,
        column: 163,
    },
    VLine {
        column: 147,
        line_start: 63,
        line_end: 63,
    },
    VLine {
        column: 163,
        line_start: 63,
        line_end: 63,
    },
    ConnectionSign {
        line: 64,
        column: 147,
    },
    HLine {
        line: 64,
        column_start: 148,
        column_end: 162,
    },
    ConnectionSign {
        line: 64,
        column: 163,
    },
  ]
}

pub fn block_resource_server_text2() -> Vec<Token> {
  use Token::*;

  vec![
    Text {
        line: 63,
        column_start: 148,
        column_end: 162,
    },
  ]
}

pub fn read_example(name: &str) -> Result<String> {
  Ok(fs::read_to_string(
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("tests/examples/sequence-diagram")
      .join(name),
  )?)
}

