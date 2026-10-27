Feature: Chapters 066-068 records and cross-act consequences
  Scenario: Guan preserves the Lukou treaty record
    Given a "guan" campaign with seed 6668
    And the flag "ma_chao_letter_record" was earned earlier
    And the flag "jingzhou_handover_record" was earned earlier
    And the lore scene "lukou_terms_record"
    When I choose 1
    Then the flag "lukou_terms_record" is present
    And the hidden conclusion "jingzhou_lukou_debt" is unlocked
    When I save and restore
    Then the flag "lukou_terms_record" is present

  Scenario: Refusing the Lukou ledger cannot claim the treaty consequence
    Given a "guan" campaign with seed 6669
    And the flag "ma_chao_letter_record" was earned earlier
    And the flag "jingzhou_handover_record" was earned earlier
    And the lore scene "lukou_terms_record"
    When I choose 2
    Then the flag "lukou_terms_record" is absent

  Scenario: Sun records the Xiaoyao bridge and Ruxu rescue
    Given a "sun" campaign with seed 6768
    And the flag "hefei_casualty_record" was earned earlier
    And the lore scene "xiaoyao_ferry_witness"
    When I choose 1
    Then the flag "xiaoyao_ferry_record" is present
    And the lore scene "ruxu_rescue_roll"
    When I choose 1
    Then the flag "ruxu_rescue_record" is present
    And the hidden conclusion "hefei_ruxu_debt" is unlocked

  Scenario: Cao separates Hanzhong records from Zuoci rumors
    Given a "cao" campaign with seed 6769
    And the flag "zhangsong_envoy_record" was earned earlier
    And the lore scene "hanzhong_civil_ledger"
    When I choose 1
    Then the flag "hanzhong_civil_record" is present
    And the lore scene "zuoci_court_record"
    When I choose 1
    Then the flag "zuoci_court_record" is present
    And the hidden conclusion "hanzhong_zuoci_debt" is unlocked

  Scenario: Zhuge records the Xiangshui diplomatic cost
    Given a "zhuge" campaign with seed 6770
    And the flag "chengdu_land_record" was earned earlier
    And the flag "yizhou_map_record" was earned earlier
    And the lore scene "xiangshui_partition"
    When I choose 1
    Then the flag "xiangshui_partition_record" is present
    And the hidden conclusion "xiangshui_yizhou_reckoning" is unlocked
