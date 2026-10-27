Feature: Chapters 111-113 Dangu defeat, Shouchun siege and Sun Xiu court responsibility

  Scenario: Dangu casualty roll follows Taoxi and unlocks one survivor debt
    Given a "zhuge" campaign with seed 111113
    And the flag "taoxi_didao_withdrawal_record" was earned earlier
    And the lore scene "dangu_casualty_roll"
    When I choose 1
    Then the flag "dangu_casualty_roll_record" is present
    And the hidden conclusion "taoxi_dangu_casualty_debt" is unlocked
    When I save and restore
    Then the flag "dangu_casualty_roll_record" is present

  Scenario: Refusing Dangu record cannot claim its old debt
    Given a "zhuge" campaign with seed 111114
    And the flag "taoxi_didao_withdrawal_record" was earned earlier
    And the lore scene "dangu_casualty_roll"
    When I choose 2
    Then the flag "dangu_casualty_roll_record" is absent

  Scenario: Shouchun civilians pay an inherited Wei accountability debt
    Given a "cao" campaign with seed 111115
    And the flag "shouchun_false_edict_record" was earned earlier
    And the lore scene "wei_huainan_summons_audit"
    When I choose 1
    And the lore scene "shouchun_city_captives"
    When I choose 1
    Then the flag "shouchun_city_captives_record" is present
    And the hidden conclusion "huainan_shouchun_captives_debt" is unlocked

  Scenario: Sun Xiu succession pays the old Dongxing archive once
    Given a "sun" campaign with seed 111116
    And the flag "dongxing_snow_bridge_record" was earned earlier
    And the lore scene "zhugedan_wu_hostage_receipt"
    When I choose 1
    And the lore scene "shouchun_wu_relief"
    When I choose 1
    And the lore scene "sunxiu_court_review"
    When I choose 1
    Then the hidden conclusion "dongxing_sunxiu_court_debt" is unlocked
    When I save and restore
    Then the flag "sunxiu_court_review_record" is present
