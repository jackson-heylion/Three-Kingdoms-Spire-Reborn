Feature: 076-078 fixed JiaJing/Mao textual contrasts and irreversible story responsibility

  Scenario: Guan preserves Maicheng aid evidence without changing the fixed defeat
    Given a "guan" campaign with seed 76077
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

  Scenario: Sun keeps posthumous Jingzhou evidence without granting a supernatural victory
    Given a "sun" campaign with seed 77078
    And the flag "ruxu_rescue_record" was earned earlier
    And the flag "yiling_ashes" was earned earlier
    And the lore scene "white_cloak_signal_audit"
    When I choose 1
    Then the flag "white_cloak_signal_record" is present
    And the lore scene "jingzhou_aftermath_courier"
    When I choose 1
    Then the flag "jingzhou_aftermath_record" is present
    And the hidden conclusion "jiangdong_jingzhou_aftermath_debt" is unlocked

  Scenario: Cao refusing the rites cannot claim a Huatuo follow-up flag
    Given a "cao" campaign with seed 78079
    And the flag "pangde_yujin_record" was earned earlier
    And the lore scene "luoyang_head_rites"
    When I choose 2
    Then the flag "luoyang_head_record" is absent
    And the lore scene "huatuo_prison_review"
    When I choose 1
    Then the flag "huatuo_prison_record" is absent
