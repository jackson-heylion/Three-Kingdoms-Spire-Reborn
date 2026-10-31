Feature: Chapters 058-060 pinned edition variants and playable optional accountability
  Scenario: Cao preserves Tongguan Weinan and Zhang Song responsibility
    Given a "cao" campaign with seed 58060
    And the flag "mateng_case_record" was earned earlier
    And the lore scene "tongguan_retreat"
    When I choose 1
    Then the flag "tongguan_retreat_record" is present
    And the lore scene "weinan_division"
    When I choose 1
    Then the flag "weinan_division_record" is present
    And the lore scene "zhangsong_audience"
    When I choose 1
    Then the flag "zhangsong_envoy_record" is present
    And the hidden conclusion "xuchang_shu_envoy_debt" is unlocked
    When I save and restore
    Then the flag "zhangsong_envoy_record" is present

  Scenario: Refusal does not create Zhang Song evidence
    Given a "cao" campaign with seed 58061
    And the flag "weinan_division_record" was earned earlier
    And the lore scene "zhangsong_audience"
    When I choose 2
    Then the flag "zhangsong_envoy_record" is absent
