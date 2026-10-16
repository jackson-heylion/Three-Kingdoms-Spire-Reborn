Feature: Fair opt-in archetype drafting from normal campaign rewards
  Hero archetype preference nudges one of three offers, never injects a starter deck.
  All preferences survive V2 snapshots and cannot be altered mid-combat.

  Scenario: Sun Shangxiang chooses a discard doctrine before departing
    Given a "sun" campaign with seed 9011
    Then the active doctrine is automatic and not yet inferred
    When I set doctrine "sun_discard"
    Then the active doctrine is "sun_discard"
    When I save and restore
    Then the active doctrine is "sun_discard"

  Scenario: Wrong hero doctrine is rejected without mutation
    Given a "sun" campaign with seed 9012
    And I remember the current snapshot
    When I attempt doctrine "cao_troops"
    Then the action is rejected
    And the snapshot is unchanged

  Scenario: Combat does not allow switching doctrine or rerolling fate
    Given a "sun" campaign with seed 9013
    When I set doctrine "sun_combo"
    And I start a battle
    And I remember the current snapshot
    And I attempt doctrine "sun_discard"
    Then the action is rejected
    And the snapshot is unchanged
    And the active doctrine is "sun_combo"

  Scenario: Real owned synergies infer a focus but duplicated cards do not
    Given a "sun" campaign with seed 9014
    When I add permanent card "huanshe"
    And I add permanent card "huanshe"
    Then the active doctrine is automatic and not yet inferred
    When I add permanent card "feishi"
    Then the active doctrine is "sun_discard"
    When I save and restore
    Then the active doctrine is "sun_discard"

  Scenario: Guided victory still offers three legal distinct cards
    Given a "sun" campaign with seed 9015
    When I set doctrine "sun_discard"
    And I start a battle
    And the first enemy has one health and no guard
    And I replace hand with "strike"
    And I play the last card
    Then the reward screen has three distinct legal cards
    When I save and restore
    Then the reward screen has three distinct legal cards
