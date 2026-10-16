Feature: Run-scoped measured combat telemetry is deterministic and save-portable

  Scenario: Cao Cao troops report actual outgoing attacks by the end of turn one
    Given a "cao" campaign with seed 60201
    When I start a battle
    And I end turn
    Then run troop attacks are at least one
    And the previous turn telemetry has completed turn one
    When I save and restore
    Then run troop attacks are at least one

  Scenario: Cao Cao command proc reports exact extra energy and guard
    Given a "cao" campaign with seed 60202
    When I start a battle
    And I replace hand with "defend"
    And I set command to two
    And I play the last card
    Then run command energy refunds equal one
    When I save and restore
    Then run command energy refunds equal one

  Scenario: Sun Shangxiang manual discard reports actual combo guard
    Given a "sun" campaign with seed 60203
    When I start a battle
    And I gain one manual discard token
    And I put card "defend" in hand
    And I manually discard the last card
    Then run Sun discards equal one
    When I save and restore
    Then run Sun discards equal one

  Scenario: Sun Shangxiang third attack increments exactly one measured combo trigger
    Given a "sun" campaign with seed 60204
    When I start a battle
    And I replace hand with "strike"
    And I put card "strike" in hand
    And I put card "strike" in hand
    And I play the last card
    And I play the last card
    And I play the last card
    Then run Sun combo triggers equal one
    When I save and restore
    Then run Sun combo triggers equal one
