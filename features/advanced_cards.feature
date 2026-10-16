Feature: X-cost techniques, selected pile searches and battle-only cards
  These scenarios perform the same Game::apply actions as the CLI and future graphical clients.

  Scenario: Guan Yu X attack consumes exactly the available energy
    Given a "guanyu" campaign with seed 981
    When I start a battle
    And I make the first enemy healthy with 200 HP
    And I replace hand with "yidao_x"
    And I set combat energy to 3
    And I remember the first enemy health
    And I play the last card
    Then battle energy equals 0
    And the first enemy loses at least 18
    And combat event "card_played" from "yidao_x" exists
    And the event sequence strictly increases

  Scenario: Lu Bu zero-energy X attack cannot manufacture damage
    Given a "lubu" campaign with seed 982
    When I start a battle
    And I make the first enemy healthy with 200 HP
    And I replace hand with "huaji_x"
    And I set combat energy to 0
    And I remember the first enemy health
    And I play the last card
    Then battle energy equals 0
    And the first enemy health is unchanged

  Scenario: Zhuge Liang can choose the second card from the draw pile
    Given a "zhuge" campaign with seed 983
    When I start a battle
    And I replace draw pile with "defend,strike,qinglong"
    And I replace hand with "xingpan_tutor"
    And I remember the current snapshot
    And I attempt to play the last card selecting 99
    Then the action is rejected
    And the snapshot is unchanged
    When I play the last card selecting 2
    Then my hand contains "strike"
    And combat event "card_tutored" from "strike" occurs exactly once
    When I save and restore
    Then my hand contains "strike"

  Scenario: Sun Shangxiang may copy a chosen discarded card without changing the campaign deck
    Given a "sun" campaign with seed 984
    When I start a battle
    And I replace discard pile with "strike,defend"
    And I replace hand with "feiyu_copy"
    And I play the last card selecting 2
    Then my hand has a temporary "strike"
    And combat event "card_copied" from "strike" occurs exactly once
    When I save and restore
    Then my hand has a temporary "strike"
    When I play the last card
    Then my exhaust pile has a temporary "strike"

  Scenario: Cao Cao generated guards are temporary and leave the battle hand at turn end
    Given a "cao" campaign with seed 985
    When I start a battle
    And I replace hand with "muji"
    And I play the last card
    Then my hand has a temporary "defend"
    And combat event "card_generated" from "defend" exists
    When I end turn
    Then my exhaust pile has a temporary "defend"
    When I save and restore
    Then my exhaust pile has a temporary "defend"

  Scenario: Neutral X defense upgrades the hand's guard without overspending energy
    Given a "guanyu" campaign with seed 986
    When I start a battle
    And I replace hand with "x_bulwark"
    And I set combat energy to 2
    And I play the last card
    Then battle energy equals 0
    And battle guard is at least 10
