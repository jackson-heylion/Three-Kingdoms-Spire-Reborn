Feature: P1 deterministic campaign replay and branching map contracts

  Scenario: A fresh campaign has a verifiable empty action history
    Given a "guan" campaign with seed 990011
    Then the map has valid explicit connected routes
    And the full action replay verifies 0 steps
    When I save and restore
    Then the full action replay verifies 0 steps

  Scenario: A real battle round can be replayed from the beginning
    Given a "cao" campaign with seed 990012
    When I start a battle
    And I end turn
    Then the full action replay verifies 2 steps
    When I save and restore
    Then the full action replay verifies 2 steps

  Scenario: Invalid commands do not enter the replay journal
    Given a "sun" campaign with seed 990013
    And I remember the current snapshot
    When I attempt to choose 99
    Then the action is rejected
    And the snapshot is unchanged
    And the full action replay verifies 0 steps

  Scenario: Replay rejects tampered live game state
    Given a "lubu" campaign with seed 990014
    When I tamper with non-action gold
    Then the full replay rejects the drift

  Scenario: Routes and player focus are reproduced after cross-floor save
    Given a "zhuge" campaign with seed 990015
    Then the map has valid explicit connected routes
    When I set doctrine "zhuge_scheme"
    And I start a battle
    Then the full action replay verifies 2 steps
    When I save and restore
    Then the full action replay verifies 2 steps
