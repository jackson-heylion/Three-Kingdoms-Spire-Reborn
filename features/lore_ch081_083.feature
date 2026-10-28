Feature: Chapters 081-083 Eastward campaign counsel, Wu-Wei diplomacy, and Yiling defensive records

  Scenario: Zhuge preserves the advice and camp map before the later fire
    Given a "zhuge" campaign with seed 8183
    And the flag "redcliff_wind" was earned earlier
    And the flag "chengdu_legitimacy_record" was earned earlier
    And the lore scene "eastward_campaign_counsel"
    When I choose 1
    Then the flag "eastward_counsel_record" is present
    And the lore scene "yiling_camp_map_warning"
    When I choose 1
    Then the flag "yiling_camp_map_record" is present
    And the hidden conclusion "redcliff_yiling_counsel_debt" is unlocked
    When I save and restore
    Then the flag "yiling_camp_map_record" is present

  Scenario: Refusing the Eastward counsel prevents the complete Zhuge payoff
    Given a "zhuge" campaign with seed 8184
    And the flag "chengdu_legitimacy_record" was earned earlier
    And the lore scene "eastward_campaign_counsel"
    When I choose 2
    Then the flag "eastward_counsel_record" is absent

  Scenario: Sun keeps Langzhong murder receipt and Wu-Wei envoy records
    Given a "sun" campaign with seed 8283
    And the flag "jiangdong_covenant" was earned earlier
    And the flag "jingzhou_aftermath_record" was earned earlier
    And the lore scene "langzhong_heads_receipt"
    When I choose 1
    Then the flag "langzhong_heads_record" is present
    And the lore scene "wu_wei_envoy_audit"
    When I choose 1
    Then the flag "wu_wei_envoy_record" is present
    And the lore scene "yiling_defense_memorial"
    When I choose 1
    Then the flag "yiling_defense_record" is present
    And the hidden conclusion "jiangdong_yiling_guard_debt" is unlocked

  Scenario: Sun refuses the first receipt without creating its flag
    Given a "sun" campaign with seed 8284
    And the flag "jingzhou_aftermath_record" was earned earlier
    And the lore scene "langzhong_heads_receipt"
    When I choose 2
    Then the flag "langzhong_heads_record" is absent

  Scenario: Cao campaign audits posthumous Wei court decisions without reviving Cao Cao
    Given a "cao" campaign with seed 8285
    And the flag "guandu_grain" was earned earlier
    And the flag "abdication_seal_record" was earned earlier
    And the lore scene "wei_wu_tribute_review"
    When I choose 1
    Then the flag "wei_wu_tribute_record" is present
    And the hidden conclusion "guandu_wei_wu_tribute_debt" is unlocked
