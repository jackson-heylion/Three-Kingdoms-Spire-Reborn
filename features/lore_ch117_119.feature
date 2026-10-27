Feature: Chapters 117-119 successor archives and inherited responsibility

  Scenario: Jiange warning, Mianzhu loss and Chengdu surrender create one debt
    Given a "zhuge" campaign with seed 117119
    And the flag "yinping_jiangge_defense_record" was earned earlier
    And the lore scene "jiange_yinping_detour_warning"
    When I choose 1
    And the lore scene "mianzhu_family_casualties"
    When I choose 1
    And the lore scene "chengdu_surrender_household"
    When I choose 1
    Then the hidden conclusion "jiange_mianzhu_chengdu_debt" is unlocked
    When I save and restore
    Then the flag "chengdu_surrender_household_record" is present

  Scenario: Refusing Mianzhu casualties does not pay its inherited debt
    Given a "zhuge" campaign with seed 117120
    And the flag "jiange_detour_warning_record" was earned earlier
    And the lore scene "mianzhu_family_casualties"
    When I choose 2
    Then the flag "mianzhu_family_casualties_record" is absent

  Scenario: Wei records Yinping, Chengdu and Zhong Hui rebellion
    Given a "cao" campaign with seed 117121
    And the flag "dingjun_civilian_protection_record" was earned earlier
    And the lore scene "wei_yinping_march_log"
    When I choose 1
    And the lore scene "wei_chengdu_occupation_receipt"
    When I choose 1
    And the lore scene "zhonghui_false_edict_inquest"
    When I choose 1
    And the lore scene "wei_jin_abdication_court"
    When I choose 1
    Then the hidden conclusion "yinping_zhonghui_court_debt" is unlocked

  Scenario: Wu dispatched aid but must not claim to have reached Mianzhu
    Given a "sun" campaign with seed 117122
    And the flag "wu_relief_warning_record" was earned earlier
    And the lore scene "wu_mianzhu_relief_dispatch"
    When I choose 1
    And the lore scene "wu_relief_arrival_dispute"
    When I choose 1
    Then the hidden conclusion "wu_mianzhu_relief_debt" is unlocked
