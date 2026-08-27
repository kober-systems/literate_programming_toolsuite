Feature: Service Discovery

  Scenario: Interactions
    When Client sends "get_hashes" to Device
    Then Device responds with "current hash" to Client
    When Client sends "get_number_protocols" to Device
    Then Device responds with "1" to Client
    When Client sends "get_protocol_schema(0)" to Device
    Then Device responds with "schema data" to Client
