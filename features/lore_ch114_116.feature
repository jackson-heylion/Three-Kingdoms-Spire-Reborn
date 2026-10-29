Feature: Chapters 114-116 successor responsibility and inherited debts

  Scenario: Wei palace witness and Wang Jing family create one inherited debt
    Given a "cao" campaign with seed 114116
    And the flag "caofang_deposition_record" was earned earlier
    And the lore scene "wei_caomao_palace_evidence"
    When I choose 1
    And the lore scene "wei_wangjing_family_relief"
    When I choose 1
    Then the hidden conclusion "caofang_caomao_wangjing_debt" is unlocked
    When I save and restore
    Then the flag "wei_wangjing_family_relief_record" is present

  Scenario: Refusing palace evidence cannot unlock its debt
    Given a "cao" campaign with seed 114117
    And the flag "caofang_deposition_record" was earned earlier
    And the lore scene "wei_caomao_palace_evidence"
    When I choose 2
    Then the flag "caomao_palace_evidence_record" is absent

  Scenario: Tanshan burned grain is followed by Taoyang casualties
    Given a "zhuge" campaign with seed 114118
    And the flag "qishan_retreat_courier_record" was earned earlier
    And the lore scene "tanshan_grain_fire_audit"
    When I choose 1
    And the lore scene "taoyang_xiahouba_casualty"
    When I choose 1
    Then the hidden conclusion "tanshan_taoyang_grain_debt" is unlocked

  Scenario: Tazhong and Yangping pay the Qishan warning
    Given a "zhuge" campaign with seed 114119
    And the flag "qishan_retreat_courier_record" was earned earlier
    And the lore scene "tanshan_grain_fire_audit"
    When I choose 1
    And the lore scene "taoyang_xiahouba_casualty"
    When I choose 1
    And the lore scene "tazhong_farming_dispatch"
    When I choose 1
    And the lore scene "yangping_gate_betrayal"
    When I choose 1
    Then the hidden conclusion "qishan_tazhong_yangping_debt" is unlocked

  Scenario: Wu records a request but not an arrived army
    Given a "sun" campaign with seed 114120
    And the flag "sunxiu_court_review_record" was earned earlier
    And the lore scene "wu_relief_warning_archive"
    When I choose 1
    Then the flag "wu_relief_warning_record" is present
    And the hidden conclusion "sunxiu_wu_relief_warning_debt" is unlocked
  Scenario: The Hu household account survives the Tazhong choice and V2 restoration
    Given a "zhuge" campaign with seed 114121
    And the flag "taoyang_xiahouba_casualty_record" was earned earlier
    And the lore scene "tazhong_farming_dispatch"
    When I choose 1
    Then the flag "tazhong_farming_dispatch_record" is present
    When I save and restore
    Then the flag "tazhong_farming_dispatch_record" is present
