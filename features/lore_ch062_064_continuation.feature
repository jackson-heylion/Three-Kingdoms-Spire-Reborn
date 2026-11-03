Feature: Chapters 062-064 opt-in western expedition responsibility without extra map nodes
  Scenario: Zhuge preserves three separate western ledgers at one natural Story node
    Given a "zhuge" campaign with seed 61068
    And the flag "leiyang_case_record" was earned earlier
    And the lore scene "yizhou_map_audit"
    When I choose 4
    Then the flag "yizhou_map_record" is present
    And the current literary chapter is "fu_pass_review"
    When I save and restore
    Then the current literary chapter is "fu_pass_review"
    When I choose 4
    Then the flag "fu_pass_record" is present
    And the current literary chapter is "luofeng_mourning"
    When I choose 4
    Then the flag "luofeng_mourning_record" is present
    And the current literary chapter is "jinyan_bridge_review"
    When I choose 1
    Then the flag "jinyan_bridge_record" is present
    And the flag "chengdu_land_record" is absent

  Scenario: Declining the Fu pass account cannot forge Luofeng responsibility
    Given a "zhuge" campaign with seed 61068
    And the flag "leiyang_case_record" was earned earlier
    And the lore scene "yizhou_map_audit"
    When I choose 4
    Then the current literary chapter is "fu_pass_review"
    When I choose 2
    Then the flag "fu_pass_record" is absent
    And the flag "luofeng_mourning_record" is absent

  Scenario: An unearned Leiyang record cannot open the western chain
    Given a "zhuge" campaign with seed 61069
    And the lore scene "yizhou_map_audit"
    When I remember the current snapshot
    And I attempt to choose 4
    Then the snapshot is unchanged
    And the flag "fu_pass_record" is absent
