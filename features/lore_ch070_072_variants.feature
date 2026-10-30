Feature: Chapters 070-072 pinned JiaJing and Mao responsibility chain
  Scenario: Zhuge preserves Wakou, Dingjun and Hanzhong records
    Given a "zhuge" campaign with seed 70072
    And the flag "chengdu_land_record" was earned earlier
    And the lore scene "wakou_civilian_route"
    When I choose 1
    Then the flag "wakou_civilian_record" is present
    And the lore scene "dingjun_signal_register"
    When I choose 1
    Then the flag "dingjun_signal_record" is present
    And the hidden conclusion "wakou_dingjun_debt" is unlocked
    And the lore scene "hanzhong_route_review"
    When I choose 1
    Then the flag "hanzhong_route_record" is present
    When I save and restore
    Then the flag "hanzhong_route_record" is present

  Scenario: Refusing Wakou evidence does not fabricate a flag
    Given a "zhuge" campaign with seed 70073
    And the flag "chengdu_land_record" was earned earlier
    And the lore scene "wakou_civilian_route"
    When I choose 2
    Then the flag "wakou_civilian_record" is absent

  Scenario: Cao preserves Hanshui and Xiegu without a Zhuge flag
    Given a "cao" campaign with seed 71072
    And the flag "xudu_fire_record" was earned earlier
    And the lore scene "hanshui_retreat_roll"
    When I choose 1
    Then the flag "hanshui_retreat_record" is present
    And the lore scene "xiegu_ration_inquest"
    When I choose 1
    Then the flag "xiegu_ration_record" is present
