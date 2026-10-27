Feature: Chapters 069-071 Xudu and Hanzhong responsibility records
  Scenario: Cao records Xudu and Hanshui
    Given a "cao" campaign with seed 6971
    And the flag "zuoci_court_record" was earned earlier
    And the lore scene "xudu_fire_inquest"
    When I choose 1
    Then the flag "xudu_fire_record" is present
    And the lore scene "hanshui_retreat_roll"
    When I choose 1
    Then the flag "hanshui_retreat_record" is present
    And the hidden conclusion "xudu_hanshui_debt" is unlocked
    When I save and restore
    Then the flag "hanshui_retreat_record" is present

  Scenario: Cao refuses Xudu responsibility
    Given a "cao" campaign with seed 6972
    And the flag "zuoci_court_record" was earned earlier
    And the lore scene "xudu_fire_inquest"
    When I choose 2
    Then the flag "xudu_fire_record" is absent

  Scenario: Zhuge records Wakou and Dingjun
    Given a "zhuge" campaign with seed 7071
    And the flag "chengdu_land_record" was earned earlier
    And the lore scene "wakou_civilian_route"
    When I choose 1
    Then the flag "wakou_civilian_record" is present
    And the lore scene "dingjun_signal_register"
    When I choose 1
    Then the flag "dingjun_signal_record" is present
    And the hidden conclusion "wakou_dingjun_debt" is unlocked

  Scenario: Refusing the Wakou civilian record blocks the optional reward
    Given a "zhuge" campaign with seed 7072
    And the flag "chengdu_land_record" was earned earlier
    And the lore scene "wakou_civilian_route"
    When I choose 2
    Then the flag "wakou_civilian_record" is absent
