Feature: Chapters 039-041 optional literary responsibility and cross-act consequences
  Scenario: Zhuge records Bowang and Xinye before settling Dangyang after Red Cliffs
    Given a "zhuge" campaign with seed 3940
    And the flag "longzhong_conditions_record" was earned earlier
    And the lore scene "bowang_orders"
    When I choose 1
    Then the flag "bowang_orders_record" is present
    And my deck contains "huogong"
    And the lore scene "xinye_evacuation"
    When I choose 1
    Then the flag "xinye_civilians" is present
    When I save and restore
    Then the flag "xinye_civilians" is present
    And the lore scene "redcliff"
    When I choose 1
    Then the flag "redcliff_wind" is present
    And the lore scene "dangyang_ledger"
    When I choose 1
    Then the hidden conclusion "xinye_dangyang_debt" is unlocked

  Scenario: Zhuge refuses the optional Bowang record and keeps the old Red Cliffs route
    Given a "zhuge" campaign with seed 3941
    And the flag "longzhong_plan" was earned earlier
    And the flag "longzhong_conditions_record" was earned earlier
    And the lore scene "bowang_orders"
    When I choose 2
    Then the flag "bowang_orders_record" is absent
    And act 1 offers the lore scene "borrow_arrows"
    And the lore scene "redcliff"
    When I choose 1
    Then the hidden conclusion "xinye_dangyang_debt" is absent

  Scenario: Sun remembers Su Fei's pardon through the Yiling aftermath
    Given a "sun" campaign with seed 3939
    And the flag "jiangdong_hostage_record" was earned earlier
    And the lore scene "jiangxia_pardon"
    When I choose 1
    Then the flag "jiangxia_pardon_record" is present
    And the lore scene "jiangdong_wedding"
    When I choose 1
    Then the flag "jiangdong_covenant" is present
    And the lore scene "yiling"
    When I choose 1
    Then the hidden conclusion "jiangxia_mercy_debt" is unlocked

  Scenario: Sun refuses the new ledger but keeps the original wedding
    Given a "sun" campaign with seed 3942
    And the flag "jiangdong_hostage_record" was earned earlier
    And the lore scene "jiangxia_pardon"
    When I choose 2
    Then the flag "jiangxia_pardon_record" is absent
    And the lore scene "jiangdong_wedding"
    When I choose 1
    Then the flag "jiangdong_covenant" is present
