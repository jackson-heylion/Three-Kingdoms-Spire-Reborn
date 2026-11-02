Feature: Chapters 037-039 keep pinned literary differences and cross-act debts
  Scenario: Zhuge carries third-visit conditions through Bowang to Red Cliffs once
    Given a "zhuge" campaign with seed 37039
    And the lore scene "longzhong"
    When I choose 1
    Then the flag "longzhong_plan" is present
    And the lore scene "longzhong_letter"
    When I choose 1
    Then the flag "longzhong_visit_record" is present
    And the lore scene "longzhong_conditions"
    When I choose 1
    Then the flag "longzhong_conditions_record" is present
    And the lore scene "bowang_orders"
    When I choose 1
    Then the flag "bowang_orders_record" is present
    And the lore scene "redcliff"
    When I choose 1
    Then the hidden conclusion "longzhong_strategy_debt" is unlocked
    When I save and restore
    Then the flag "redcliff_wind" is present

  Scenario: Guan can refuse the Xu Shu record without inventing a witness
    Given a "guanyu" campaign with seed 37040
    And the flag "fancheng_counsel_record" was earned earlier
    And the lore scene "xushu_departure"
    When I choose 2
    Then the flag "xushu_departure_record" is absent
    When I save and restore
    Then the flag "xushu_departure_record" is absent

  Scenario: Sun's Jiangxia pardon remains an optional separate witness
    Given a "sunshangxiang" campaign with seed 37041
    And the flag "sun_succession_account" was earned earlier
    And the lore scene "jiangdong_hostage"
    When I choose 1
    Then the flag "jiangdong_hostage_record" is present
    And the lore scene "jiangxia_pardon"
    When I choose 1
    Then the flag "jiangxia_pardon_record" is present
    When I save and restore
    Then the flag "jiangxia_pardon_record" is present

  Scenario: Refusing Bowang must not block an older Longzhong payoff
    Given a "zhuge" campaign with seed 37042
    And the lore scene "longzhong"
    When I choose 1
    And the lore scene "longzhong_letter"
    When I choose 1
    And the lore scene "longzhong_conditions"
    When I choose 1
    And the lore scene "bowang_orders"
    When I choose 2
    Then the flag "bowang_orders_record" is absent
    And the lore scene "redcliff"
    When I choose 1
    Then the hidden conclusion "longzhong_strategy_debt" is unlocked
    When I save and restore
    Then the hidden conclusion "longzhong_strategy_debt" is unlocked


  Scenario: Bowang responsibility survives civilian evacuation and Dangyang to Red Cliffs once
    Given a "zhuge" campaign with seed 37044
    And the lore scene "longzhong"
    When I choose 1
    And the lore scene "longzhong_letter"
    When I choose 1
    And the lore scene "longzhong_conditions"
    When I choose 1
    And the lore scene "bowang_orders"
    When I choose 1
    And the lore scene "xinye_evacuation"
    When I choose 1
    And the lore scene "dangyang_ledger"
    When I choose 1
    And the lore scene "redcliff"
    When I choose 1
    Then the hidden conclusion "xinye_dangyang_debt" is unlocked
    When I save and restore
    Then the hidden conclusion "xinye_dangyang_debt" is unlocked
