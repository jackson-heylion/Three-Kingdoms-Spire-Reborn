Feature: Chapters 087-089 Yongchang, first five captures and water accountability

  Scenario: Zhuge preserves the complete first-five-captures civilian ledger
    Given a "zhuge" campaign with seed 8789
    And the flag "redcliff_wind" was earned earlier
    And the flag "dengzhi_alliance_record" was earned earlier
    And the lore scene "nanzhong_yongchang_memorial"
    When I choose 1
    Then the flag "nanzhong_yongchang_record" is present
    And the lore scene "nanzhong_first_release_ledger"
    When I choose 1
    Then the flag "nanzhong_first_release_record" is present
    And the lore scene "lushui_ferry_care"
    When I choose 1
    Then the flag "lushui_ferry_record" is present
    And the lore scene "xier_spring_civilian_water"
    When I choose 1
    Then the flag "xier_spring_record" is present
    And the hidden conclusion "redcliff_nanzhong_water_debt" is unlocked
    When I save and restore
    Then the flag "xier_spring_record" is present

  Scenario: Refusing the Lushui civilian account blocks the final conclusion
    Given a "zhuge" campaign with seed 8790
    And the flag "nanzhong_first_release_record" was earned earlier
    And the lore scene "lushui_ferry_care"
    When I choose 2
    Then the flag "lushui_ferry_record" is absent

  Scenario: Sun posthumous archive distinguishes Wu remote appointment
    Given a "sun" campaign with seed 8791
    And the flag "jiangdong_covenant" was earned earlier
    And the flag "guangling_defense_record" was earned earlier
    And the lore scene "wu_nanzhong_border_archive"
    When I choose 1
    Then the flag "wu_nanzhong_border_record" is present
    And the hidden conclusion "jiangdong_nanzhong_envoy_debt" is unlocked

  Scenario: Rejecting Wu archive does not grant historical accountability
    Given a "sun" campaign with seed 8792
    And the flag "guangling_defense_record" was earned earlier
    And the lore scene "wu_nanzhong_border_archive"
    When I choose 2
    Then the flag "wu_nanzhong_border_record" is absent
