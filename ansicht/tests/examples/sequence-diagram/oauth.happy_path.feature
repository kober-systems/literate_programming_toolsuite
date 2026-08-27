Feature: OAuth happy path

  Scenario: Initial Redirect for Authorization
    Given Initial Redirect for Authorization
    When End User requests access
    And User's Browser requests access from Client Application
    And Client Application redirects User's Browser to Authorization Server with client_id, response_type=code, redirect_uri, and scope
    And User's Browser follows the redirect to Authorization Server
    Then User Grants Consent

  Scenario: User Grants Consent
    Given User Grants Consent
    When Authorization Server displays the consent form
    And User's Browser displays the consent form to End User
    And End User grants consent
    And User's Browser sends the consent to Authorization Server
    And Authorization Server redirects with an authorization code
    And User's Browser follows the redirect to Client Application with the code
    Then Token Exchange and Resource Access

  Scenario: Token Exchange and Resource Access
    Given Token Exchange and Resource Access
    When Client Application exchanges the code for an access token
    And Authorization Server responds with an access token
    And Client Application requests the protected resource with the access token
    And Resource Server responds with the protected resource
    And Client Application displays the resource to User's Browser
    Then User's Browser displays the resource to End User
