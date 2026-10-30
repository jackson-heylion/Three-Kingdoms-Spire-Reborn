Feature: 091-093 memorial, northern route and Tianshui responsibility

  Scenario: Lushui memorial carries old Nanzhong costs into the northern campaign
    Given a "zhuge" campaign with seed 91093
    And the flag "xier_spring_record" was earned earlier
    And the flag "nanzhong_vine_fire_record" was earned earlier
    And the flag "nanzhong_local_rule_record" was earned earlier
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "lushui_memorial_rites"
    When I choose 1
    Then the flag "lushui_memorial_record" is present
    And the hidden conclusion "nanzhong_mourning_debt" is unlocked
    And the lore scene "chushi_memorial_supply"
    When I choose 1
    Then the flag "chushi_supply_record" is present
    And the lore scene "ziwu_route_risk_hearing"
    When I choose 1
    Then the hidden conclusion "redcliff_northern_route_debt" is unlocked
    When I save and restore
    Then the flag "ziwu_risk_record" is present

  Scenario: Tianshui false dispatch precedes the real surrender and unlocks once
    Given a "zhuge" campaign with seed 92093
    And the flag "chushi_supply_record" was earned earlier
    And the lore scene "ziwu_route_risk_hearing"
    When I choose 1
    And the lore scene "tianshui_false_dispatch_review"
    When I choose 1
    Then the flag "tianshui_false_dispatch_record" is present
    And the lore scene "tianshui_recruitment_receipt"
    When I choose 1
    Then the hidden conclusion "tianshui_truth_debt" is unlocked
    When I save and restore
    Then the hidden conclusion "tianshui_truth_debt" is unlocked

  Scenario: Profit at Tianshui cannot fabricate a prior warning
    Given a "zhuge" campaign with seed 93093
    And the flag "chushi_supply_record" was earned earlier
    And the lore scene "ziwu_route_risk_hearing"
    When I choose 1
    And the lore scene "tianshui_false_dispatch_review"
    When I choose 2
    Then the flag "tianshui_false_dispatch_record" is absent
