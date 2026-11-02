Feature: Chapters 055-057 pinned Mao text and optional cross-act responsibility
  Scenario: Sun preserves Nanxu agency and the Yiling echo only once
    Given a "sun" campaign with seed 55057
    And the flag "jiangdong_covenant" was earned earlier
    And the flag "yiling_ashes" was earned earlier
    And the lore scene "nanxu_escape_witness"
    When I choose 1
    Then the flag "nanxu_escape_witness_record" is present
    And the hidden conclusion "nanxu_yiling_witness" is unlocked
    When I save and restore
    Then the flag "nanxu_escape_witness_record" is present

  Scenario: Sun declines Nanxu evidence but the old Yiling story remains available
    Given a "sun" campaign with seed 55058
    And the flag "jiangdong_covenant" was earned earlier
    And the lore scene "nanxu_escape_witness"
    When I choose 2
    Then the flag "nanxu_escape_witness_record" is absent
    And the lore scene "yiling"
    When I choose 1
    Then the flag "yiling_ashes" is present

  Scenario: Cao records Cheng Yu, Xun You and the later Weinan cost
    Given a "cao" campaign with seed 56057
    And the flag "guandu_grain" was earned earlier
    And the flag "river_banquet_record" was earned earlier
    And the lore scene "tongque_dispatch_ledger"
    When I choose 1
    Then the flag "tongque_dispatch_record" is present
    And the lore scene "mateng_case"
    When I choose 1
    Then the flag "mateng_case_record" is present
    And the lore scene "tongguan_retreat"
    When I choose 1
    Then the flag "tongguan_retreat_record" is present
    And the lore scene "weinan_division"
    When I choose 1
    Then the hidden conclusion "xuchang_weinan_reckoning" is unlocked

  Scenario: Cao refuses Mateng evidence without blocking old Liaodong
    Given a "cao" campaign with seed 56058
    And the flag "guandu_grain" was earned earlier
    And the flag "tongque_dispatch_record" was earned earlier
    And the lore scene "mateng_case"
    When I choose 2
    Then the flag "mateng_case_record" is absent
    When I save and restore
    Then the flag "mateng_case_record" is absent
    And the lore scene "liaodong"
    When I choose 1
    Then the flag "liaodong_letter" is present

  Scenario: Zhuge refuses the false passage record but can still review Leiyang
    Given a "zhuge" campaign with seed 57057
    And the flag "jingzhou_pledge_copy_record" was earned earlier
    And the flag "phoenix_letter" was earned earlier
    And the flag "longzhong_plan" was earned earlier
    And the lore scene "jingzhou_false_passage"
    When I choose 2
    Then the flag "jingzhou_false_passage_record" is absent
    And the lore scene "leiyang_review"
    When I choose 1
    Then the flag "leiyang_case_record" is present
    And the hidden conclusion "phoenix_office_reckoning" is unlocked
