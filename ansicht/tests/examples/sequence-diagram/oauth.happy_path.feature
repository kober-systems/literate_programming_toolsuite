Feature: OAuth happy path

  Scenario: Initial Redirect for Authorization
    Given Initial Redirect for Authorization
    When End User sends "Request Access" to User's Browser
    Then User's Browser responds with "Request Access" to Client Application
    When Client Application sends "Redirect to AuthServer (client_id, response_type=code, redirect_uri, scope)" to User's Browser
    Then User's Browser responds with "Follow Redirect" to Authorization Server
    And User Grants Consent

  Scenario: User Grants Consent
    Given User Grants Consent
    When Authorization Server sends "Display Consent Form" to User's Browser
    Then User's Browser responds with "Display Consent Form" to End User
    When End User sends "Grant Consent" to User's Browser
    Then User's Browser responds with "Grant Consent" to Authorization Server
    When Authorization Server sends "Redirect with Authorization Code" to User's Browser
    Then User's Browser responds with "Follow Redirect with Code" to Client Application
    And Token Exchange and Resource Access

  Scenario: Token Exchange and Resource Access
    Given Token Exchange and Resource Access
    When Client Application sends "Exchange Code for Token" to Authorization Server
    And Authorization Server sends "Respond with Access Token" to Client Application
    And Client Application sends "Request Protected Resource (with Access Token)" to Resource Server
    And Resource Server sends "Respond with Protected Resource" to Client Application
    And Client Application sends "Display Resource" to User's Browser
    Then User's Browser responds with "Display Resource" to End User
