Feature: Chapters 090-092 seven captures, mourning and northern campaign accountability

  Scenario: Seven captures and mourning create one civilian conclusion
    Given a "zhuge" campaign with seed 9092
    And the flag "xier_spring_record" was earned earlier
    And the lore scene "nanzhong_vine_fire_account"
    When I choose 1
    Then the flag "nanzhong_vine_fire_record" is present
    And the lore scene "nanzhong_local_governance"
    When I choose 1
    Then the flag "nanzhong_local_rule_record" is present
    And the lore scene "lushui_memorial_rites"
    When I choose 1
    Then the flag "lushui_memorial_record" is present
    And the hidden conclusion "nanzhong_mourning_debt" is unlocked
    When I save and restore
    Then the flag "lushui_memorial_record" is present

  Scenario: Refusal of the fire account does not earn civilian conclusion
    Given a "zhuge" campaign with seed 9093
    And the flag "xier_spring_record" was earned earlier
    And the lore scene "nanzhong_vine_fire_account"
    When I choose 2
    Then the flag "nanzhong_vine_fire_record" is absent

  Scenario: Memorial and unexecuted Ziwu proposal preserve northern accountability
    Given a "zhuge" campaign with seed 9094
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "chushi_memorial_supply"
    When I choose 1
    Then the flag "chushi_supply_record" is present
    And the lore scene "ziwu_route_risk_hearing"
    When I choose 1
    Then the flag "ziwu_risk_record" is present
    And the hidden conclusion "redcliff_northern_route_debt" is unlocked

  Scenario: Tianshui warning remains an optional consequence rather than a forced surrender
    Given a "zhuge" campaign with seed 9095
    And the flag "ziwu_risk_record" was earned earlier
    And the lore scene "tianshui_false_dispatch_review"
    When I choose 1
    Then the flag "tianshui_false_dispatch_record" is present
