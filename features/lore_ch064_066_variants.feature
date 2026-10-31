Feature: Chapters 064-066 fixed dual-edition local evidence and optional outcomes
  Scenario: Zhuge preserves Jinyan captives and Chengdu civilian land records
    Given a "zhuge" campaign with seed 64065
    And the flag "luofeng_mourning_record" was earned earlier
    And the lore scene "jinyan_bridge_review"
    When I choose 1
    Then the flag "jinyan_bridge_record" is present
    And the lore scene "chengdu_land_decree"
    When I choose 1
    Then the flag "chengdu_land_record" is present
    When I save and restore
    Then the flag "chengdu_land_record" is present

  Scenario: Guanyu may refuse to turn the Lukou diplomatic dispute into a record
    Given a "guanyu" campaign with seed 65066
    And the flag "ma_chao_letter_record" was earned earlier
    And the lore scene "lukou_terms_record"
    When I choose 2
    Then the flag "lukou_terms_record" is absent
