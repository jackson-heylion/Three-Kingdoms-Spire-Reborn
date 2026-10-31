Feature: Chapters 016-018 pinned witnesses and optional three-chapter responsibility
  Scenario: Lu Bu's archery promise leads through seven armies and the Xiaopei courier to White Gate
    Given a "lubu" campaign with seed 16018
    And the flag "phoenix_pavilion" was earned earlier
    And the lore scene "halberd"
    When I choose 1
    Then the flag "halberd_shot" is present
    And the lore scene "seven_armies"
    When I choose 1
    Then the flag "seven_armies_account" is present
    And the lore scene "xiaopei_courier"
    When I choose 1
    Then the flag "xiaopei_courier_guard" is present
    When I save and restore
    Then the flag "xiaopei_courier_guard" is present
    And the lore scene "whitegate"
    When I choose 1
    Then the hidden conclusion "xiaopei_courier_reckoning" is unlocked

  Scenario: Cao's grain ledger and Anzhong report pay at Guandu
    Given a "cao" campaign with seed 17018
    And the flag "puyang_supply" was earned earlier
    And the flag "xu_capital" was earned earlier
    And the lore scene "shouchun_grain"
    When I choose 1
    Then the flag "shouchun_grain_account" is present
    And the lore scene "anzhong"
    When I choose 1
    Then the flag "anzhong_field_report" is present
    When I save and restore
    Then the flag "anzhong_field_report" is present
    And the lore scene "guandu"
    When I choose 1
    Then the hidden conclusion "anzhong_reckoning" is unlocked

  Scenario: Refusing the seven-army record does not softlock the older White Gate route
    Given a "lubu" campaign with seed 17019
    And the flag "halberd_shot" was earned earlier
    And the lore scene "seven_armies"
    When I choose 2
    Then the flag "seven_armies_account" is absent
    And act 2 offers the lore scene "whitegate"
