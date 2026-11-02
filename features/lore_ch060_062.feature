Feature: Chapters 060-062 original literary accountability without rewriting history
  Scenario: Cao records Zhang Song's dismissal after the Weinan inquiry
    Given a "cao" campaign with seed 6060
    And the flag "mateng_case_record" was earned earlier
    And the flag "tongguan_retreat_record" was earned earlier
    And the flag "weinan_division_record" was earned earlier
    And the lore scene "zhangsong_audience"
    When I choose 1
    Then the flag "zhangsong_envoy_record" is present
    And the hidden conclusion "xuchang_shu_envoy_debt" is unlocked
    When I save and restore
    Then the flag "zhangsong_envoy_record" is present

  Scenario: Zhuge's retrospective map and Fu pass reports settle once
    Given a "zhuge" campaign with seed 6061
    And the flag "leiyang_case_record" was earned earlier
    And the lore scene "yizhou_map_audit"
    When I choose 1
    Then the flag "yizhou_map_record" is present
    And the lore scene "fu_pass_review"
    When I choose 1
    Then the flag "fu_pass_record" is present
    And the hidden conclusion "yizhou_fu_pass_debt" is unlocked

  Scenario: Sun records the false mother illness without changing A Dou's rescue
    Given a "sun" campaign with seed 6062
    And the flag "nanxu_escape_witness_record" was earned earlier
    And the lore scene "river_child_witness"
    When I choose 1
    Then the flag "river_child_witness_record" is present
    And the hidden conclusion "nanxu_river_witness" is unlocked

  Scenario: Refusing the river account plants no responsibility flag
    Given a "sun" campaign with seed 6063
    And the flag "nanxu_escape_witness_record" was earned earlier
    And the lore scene "river_child_witness"
    When I choose 2
    Then the flag "river_child_witness_record" is absent

  Scenario: Refusing the Fu pass report plants no responsibility flag
    Given a "zhuge" campaign with seed 6064
    And the flag "yizhou_map_record" was earned earlier
    And the lore scene "fu_pass_review"
    When I choose 2
    Then the flag "fu_pass_record" is absent

  Scenario: Fu pass refusal cannot claim the subsequent Luofeng responsibility
    Given a "zhuge" campaign with seed 61062
    And the flag "yizhou_map_record" was earned earlier
    And the lore scene "fu_pass_review"
    When I choose 2
    Then the flag "fu_pass_record" is absent
    When I save and restore
    Then the flag "fu_pass_record" is absent

  Scenario: Leiyang witness may deliberately switch a third-act story to Yizhou
    Given a "zhuge" campaign with seed 61068
    And the flag "leiyang_case_record" was earned earlier
    And the lore scene "jiangwei"
    When I choose 4
    Then the current literary chapter is "yizhou_map_audit"
    When I save and restore
    Then the current literary chapter is "yizhou_map_audit"
    When I choose 1
    Then the flag "yizhou_map_record" is present
    And the flag "successor_scroll" is absent

  Scenario: Without Leiyang the alternate scene is rejected atomically
    Given a "zhuge" campaign with seed 61069
    And the lore scene "jiangwei"
    When I remember the current snapshot
    And I attempt to choose 4
    Then the snapshot is unchanged
    And the flag "yizhou_map_record" is absent
