Feature: Literary choices are portable and transactional
  A failed action cannot create phantom bonds or spoil the deterministic campaign.

  Scenario: Rejected story choice does not change the live snapshot
    Given a "guanyu" campaign with seed 171
    And the lore scene "baima"
    When I remember the current snapshot
    And I attempt to choose 100
    Then the action is rejected
    And the snapshot is unchanged
    And the flag "baima_debt" is absent

  Scenario: Undoing history is not an allowed side effect of serializing
    Given a "cao" campaign with seed 181
    And the lore scene "xudu"
    When I choose 1
    And I save and restore
    Then the flag "xu_capital" is present
    And the bond with "荀彧" is active
    When I start a battle
    And I save and restore
    Then the current scene is combat
    And battle command is at least 2

  Scenario: Game play is deterministic even after mid-combat save and resume
    Given a "guanyu" campaign with seed 191
    When I start a battle
    And I save and restore
    Then the current scene is combat
    When I end turn
    And I save and restore
    Then the current scene is combat

  Scenario: A second terminal cannot overwrite an active single-player session
    Given an isolated save directory
    When I start a writer in that directory
    Then another writer is refused without modifying the save
    When I close the first writer
    Then a new writer can take the same directory lock

  Scenario: A leftover lock file does not make a completed session inaccessible
    Given an isolated save directory
    When I start a writer in that directory
    And I close the first writer
    Then a new writer can take the same directory lock
