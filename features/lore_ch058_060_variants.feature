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


  Scenario: Zhuge preserves the Yizhou map and later Fu Pass without duplicate record
    Given a "zhuge" campaign with seed 58063
    And the flag "leiyang_case_record" was earned earlier
    And the lore scene "yizhou_map_audit"
    When I choose 1
    Then the flag "yizhou_map_record" is present
    When I save and restore
    Then the flag "yizhou_map_record" is present
    And the lore scene "fu_pass_review"
    When I choose 1
    Then the flag "fu_pass_record" is present

  Scenario: Zhuge declines the Yizhou map without blocking the old Longzhong story
    Given a "zhuge" campaign with seed 58064
    And the flag "leiyang_case_record" was earned earlier
    And the lore scene "yizhou_map_audit"
    When I choose 2
    Then the flag "yizhou_map_record" is absent
    When I save and restore
    Then the flag "yizhou_map_record" is absent
    And the lore scene "longzhong"
    When I choose 1
    Then the flag "longzhong_plan" is present
