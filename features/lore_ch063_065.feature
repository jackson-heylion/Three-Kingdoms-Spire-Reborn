Feature: Chapters 063-065 literary consequences without alternate deaths
  Scenario: Zhuge records Luofeng and Chengdu land governance
    Given a "zhuge" campaign with seed 6363
    And the flag "fu_pass_record" was earned earlier
    And the lore scene "luofeng_mourning"
    When I choose 1
    Then the flag "luofeng_mourning_record" is present
    And the lore scene "jinyan_bridge_review"
    When I choose 1
    Then the flag "jinyan_bridge_record" is present
    And the lore scene "chengdu_land_decree"
    When I choose 1
    Then the flag "chengdu_land_record" is present
    And the hidden conclusion "luofeng_chengdu_reckoning" is unlocked
    When I save and restore
    Then the flag "chengdu_land_record" is present

  Scenario: Refusing Luofeng record never plants responsibility
    Given a "zhuge" campaign with seed 6364
    And the flag "fu_pass_record" was earned earlier
    And the lore scene "luofeng_mourning"
    When I choose 2
    Then the flag "luofeng_mourning_record" is absent

  Scenario: Guan keeps the Jingzhou seal and Ma Chao letter
    Given a "guan" campaign with seed 6365
    And the flag "hanjin_ferry_record" was earned earlier
    And the flag "jingzhou_warning" was earned earlier
    And the lore scene "jingzhou_handover"
    When I choose 1
    Then the flag "jingzhou_handover_record" is present
    And the lore scene "ma_chao_letter"
    When I choose 1
    Then the flag "ma_chao_letter_record" is present
    And the hidden conclusion "jingzhou_guard_letter_debt" is unlocked

  Scenario: Refusing the letter cannot claim its consequence
    Given a "guan" campaign with seed 6366
    And the flag "jingzhou_handover_record" was earned earlier
    And the lore scene "ma_chao_letter"
    When I choose 2
    Then the flag "ma_chao_letter_record" is absent
