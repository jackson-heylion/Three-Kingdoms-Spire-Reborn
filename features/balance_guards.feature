Feature: Measured hero balance adjustments retain transparent tactics and save integrity

  Scenario: Sun Shangxiang every third attack converts combo into defense
    Given a "sun" campaign with seed 8081
    When I start a battle
    And I put card "strike" in hand
    And I play the last card
    And I put card "strike" in hand
    And I play the last card
    And I put card "strike" in hand
    And I play the last card
    Then battle guard is at least 3
    And combat event "power_triggered" from "sun_combo_guard" occurs exactly once
    When I save and restore
    Then battle guard is at least 3

  Scenario: Sun Shangxiang has defensive positioning on a manual discard
    Given a "sun" campaign with seed 8082
    When I start a battle
    And I gain one manual discard token
    And I put card "defend" in hand
    And I manually discard the last card
    Then battle guard is at least 2
    And combat event "power_triggered" from "sun_discard_guard" occurs exactly once
    When I save and restore
    Then battle guard is at least 2

  Scenario: Cao Cao retains troop identity but starts with reduced passive attack
    Given a "cao" campaign with seed 8083
    When I start a battle
    Then battle troops attack equals 2
    When I save and restore
    Then battle troops attack equals 2
