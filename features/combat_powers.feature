Feature: Persistent 三国 class powers have deterministic combat semantics
  These checks exercise Game::apply instead of duplicating game rules in test descriptions.

  Scenario: Guan Yu's Oath Guard triggers once per attack card
    Given a "guanyu" campaign with seed 701
    When I start a battle
    And I put card "oath_guard" in hand
    And I play the last card
    Then the power "oath_guard" has at least 1 stacks
    When I put card "strike" in hand
    And I play the last card
    Then battle guard is at least 7
    And combat event "power_triggered" from "oath_guard" exists
    And combat event "damage_dealt" from "hero" exists
    And the event sequence strictly increases
    When I save and restore
    Then the power "oath_guard" has at least 1 stacks

  Scenario: Zhuge Liang's thought power survives save before triggering another skill
    Given a "zhuge" campaign with seed 702
    When I start a battle
    And I put card "thought_spring" in hand
    And I play the last card
    Then the power "thought_spring" has at least 1 stacks
    And combat event "power_triggered" from "thought_spring" exists
    When I save and restore
    Then the event sequence strictly increases

  Scenario: Sun Shangxiang's flying arrows trigger on deliberate hand discard
    Given a "sun" campaign with seed 703
    When I start a battle
    And I put card "arrows_on_discard" in hand
    And I play the last card
    Then the power "arrows_on_discard" has at least 1 stacks
    When I gain one manual discard token
    And I put card "defend" in hand
    And I remember the first enemy health
    And I manually discard the last card
    Then the first enemy loses at least 4
    And combat event "card_discarded" from "defend" exists
    And combat event "power_triggered" from "arrows_on_discard" exists
    And the event sequence strictly increases

  Scenario: Cao Cao's granary support remains active across turns and snapshots
    Given a "cao" campaign with seed 704
    When I start a battle
    And I put card "granary_support" in hand
    And I play the last card
    Then the power "granary_support" has at least 1 stacks
    When I save and restore
    Then the power "granary_support" has at least 1 stacks
    When I end turn
    Then the current scene is combat
    And the event sequence strictly increases

  Scenario: Lu Bu's blood courage is a persistent battle power
    Given a "lubu" campaign with seed 705
    When I start a battle
    And I put card "blood_courage" in hand
    And I play the last card
    Then the power "blood_courage" has at least 1 stacks
    And combat event "power_gained" from "浴血鏖战" exists
    When I save and restore
    Then the power "blood_courage" has at least 1 stacks
