Feature: Advanced deterministic combat reactions and card lifecycle
  Powers and keywords are exercised through real commands and inspected after serialization.

  Scenario: A retained ambush card crosses the turn while a fleeting strike exhausts
    Given a "guanyu" campaign with seed 718
    When I start a battle
    And I put card "hold_line" in hand
    And I put card "fleeting_blade" in hand
    And I end turn
    Then the current scene is combat
    And my hand contains "hold_line"
    And my exhaust pile contains "fleeting_blade"
    And combat event "card_exhausted" from "fleeting_blade" occurs exactly once
    When I save and restore
    Then my hand contains "hold_line"
    And my exhaust pile contains "fleeting_blade"

  Scenario: Zhuge Liang gains a shield for every successfully drawn card
    Given a "zhuge" campaign with seed 719
    When I start a battle
    And I put card "wisdom_on_draw" in hand
    And I play the last card
    Then the power "wisdom_on_draw" has at least 1 stacks
    When I put card "qimou" in hand
    And I play the last card
    Then combat event "card_drawn" from "strike" exists
    And combat event "power_triggered" from "wisdom_on_draw" exists
    And the event sequence strictly increases
    When I save and restore
    Then the power "wisdom_on_draw" has at least 1 stacks

  Scenario: Poison amplification runs once rather than self-triggering recursively
    Given a "zhuge" campaign with seed 720
    When I start a battle
    And I put card "toxic_reversal" in hand
    And I play the last card
    And I put card "duji" in hand
    And I play the last card
    Then the first enemy has at least 5 poison
    And combat event "status_applied" from "poison" occurs exactly once
    And combat event "power_triggered" from "toxic_reversal" occurs exactly once
    When I save and restore
    Then the first enemy has at least 5 poison

  Scenario: Guan Yu receives a victory drum once after an actual defeat
    Given a "guanyu" campaign with seed 721
    When I start a battle
    And I duplicate the first enemy as a second combatant
    And I put card "victory_drum" in hand
    And I play the last card
    And the first enemy has one health and no guard
    And I put card "strike" in hand
    And I play the last card
    Then combat event "foe_defeated" from "hero" occurs exactly once
    And combat event "power_triggered" from "victory_drum" occurs exactly once
    And the event sequence strictly increases
    When I save and restore
    Then the power "victory_drum" has at least 1 stacks

  Scenario: Advanced reactive powers still preserve invalid-action atomicity
    Given a "sun" campaign with seed 722
    When I start a battle
    And I put card "arrows_on_discard" in hand
    And I play the last card
    And I remember the current snapshot
    And I attempt to choose 999
    Then the action is rejected
    And the snapshot is unchanged

  Scenario: Final thorn retaliation wins before turn-44 supply exhaustion
    Given a "guanyu" campaign with seed 93241
    When I start a battle
    And I prepare a last-foe thorn counter at turn 44
    And I end turn
    Then the counterattack wins before supply exhaustion
    When I save and restore
    Then the counterattack wins before supply exhaustion

  Scenario: Mutual lethal retaliation is still a player loss at turn 44
    Given a "guanyu" campaign with seed 93242
    When I start a battle
    And I prepare a mutual-lethal thorn counter at turn 44
    And I end turn
    Then the simultaneous lethal counterattack is a loss
    When I save and restore
    Then the simultaneous lethal counterattack is a loss
