Feature: 088-090 fixed-edition Southland costs remain playable without changing novel outcomes

  Scenario: Lushui casualties flow through Xier and seven captures to the memorial
    Given a "zhuge" campaign with seed 88090
    And the flag "redcliff_wind" was earned earlier
    And the flag "nanzhong_first_release_record" was earned earlier
    And the lore scene "lushui_ferry_care"
    When I choose 1
    Then the flag "lushui_ferry_record" is present
    And the lore scene "xier_spring_civilian_water"
    When I choose 1
    Then the flag "xier_spring_record" is present
    And the lore scene "nanzhong_vine_fire_account"
    When I choose 1
    Then the flag "nanzhong_vine_fire_record" is present
    And the lore scene "nanzhong_local_governance"
    When I choose 1
    Then the flag "nanzhong_local_rule_record" is present
    And the lore scene "lushui_memorial_rites"
    When I choose 1
    Then the hidden conclusion "nanzhong_mourning_debt" is unlocked
    When I save and restore
    Then the flag "lushui_memorial_record" is present

  Scenario: Refusing the Lushui casualty account cannot forge the next prerequisite
    Given a "zhuge" campaign with seed 89090
    And the flag "nanzhong_first_release_record" was earned earlier
    And the lore scene "lushui_ferry_care"
    When I choose 2
    Then the flag "lushui_ferry_record" is absent
