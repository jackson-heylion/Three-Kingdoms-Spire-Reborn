Feature: Chapter 075-077 fixed novel outcomes and retrospective responsibility

  Scenario: Guan preserves wound and Maicheng relief chain
    Given a "guan" campaign with seed 7577
    And the flag "anxi_seal" was earned earlier
    And the flag "jingxiang_border_record" was earned earlier
    And the flag "fancheng_flood_record" was earned earlier
    And the lore scene "blade_wound_watch"
    When I choose 1
    Then the flag "blade_wound_record" is present
    And the lore scene "maicheng_relief_review"
    When I choose 1
    Then the flag "maicheng_relief_record" is present
    And the hidden conclusion "jingzhou_maicheng_courier_debt" is unlocked
    When I save and restore
    Then the flag "maicheng_relief_record" is present

  Scenario: Guan declines the wound record and cannot claim full chain
    Given a "guan" campaign with seed 7578
    And the flag "anxi_seal" was earned earlier
    And the flag "jingxiang_border_record" was earned earlier
    And the flag "fancheng_flood_record" was earned earlier
    And the lore scene "blade_wound_watch"
    When I choose 2
    Then the flag "blade_wound_record" is absent

  Scenario: Sun records the Jingzhou civilian aftermath
    Given a "sun" campaign with seed 7677
    And the flag "ruxu_rescue_record" was earned earlier
    And the flag "yiling_ashes" was earned earlier
    And the lore scene "white_cloak_signal_audit"
    When I choose 1
    Then the flag "white_cloak_signal_record" is present
    And the lore scene "jingzhou_aftermath_courier"
    When I choose 1
    Then the flag "jingzhou_aftermath_record" is present
    And the hidden conclusion "jiangdong_jingzhou_aftermath_debt" is unlocked

  Scenario: Cao records Luoyang rites without asserting supernatural proof
    Given a "cao" campaign with seed 7777
    And the flag "guandu_grain" was earned earlier
    And the flag "pangde_yujin_record" was earned earlier
    And the lore scene "luoyang_head_rites"
    When I choose 1
    Then the flag "luoyang_head_record" is present
    And the hidden conclusion "guandu_luoyang_rites_debt" is unlocked

  Scenario: Zhuge keeps Chengdu dispatch chronology
    Given a "zhuge" campaign with seed 7778
    And the flag "redcliff_wind" was earned earlier
    And the flag "hanzhong_king_record" was earned earlier
    And the lore scene "chengdu_mourning_register"
    When I choose 1
    Then the flag "chengdu_mourning_record" is present
    And the hidden conclusion "redcliff_chengdu_mourning_debt" is unlocked

  Scenario: Guan refusing the warning does not gain its evidence flag
    Given a "guan" campaign with seed 75073
    And the flag "lukou_terms_record" was earned earlier
    And the lore scene "jingxiang_border_warning"
    When I choose 2
    Then the flag "jingxiang_border_record" is absent
