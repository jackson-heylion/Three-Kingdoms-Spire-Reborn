Feature: Chapters 040-042 preserve civilian and ferry accountability across editions
  Scenario: Zhuge carries Xinye evacuation evidence into Dangyang once
    Given a "zhuge" campaign with seed 40042
    And the flag "bowang_orders_record" was earned earlier
    And the lore scene "xinye_evacuation"
    When I choose 1
    Then the flag "xinye_civilians" is present
    And the lore scene "dangyang_ledger"
    When I choose 1
    Then the flag "dangyang_refuge_account" is present
    And the lore scene "redcliff"
    When I choose 1
    Then the flag "redcliff_wind" is present
    And the hidden conclusion "xinye_dangyang_debt" is unlocked
    When I save and restore
    Then the flag "dangyang_refuge_account" is present

  Scenario: Refusal does not invent a civilian flag or block the old Red Cliffs
    Given a "zhuge" campaign with seed 40043
    And the flag "bowang_orders_record" was earned earlier
    And the lore scene "xinye_evacuation"
    When I choose 2
    Then the flag "xinye_civilians" is absent
    And the lore scene "redcliff"
    When I choose 1
    Then the flag "redcliff_wind" is present

  Scenario: Guan's Hanjin ferry record pays at Huarong
    Given a "guanyu" campaign with seed 42043
    And the flag "xutian_guarded_emperor" was earned earlier
    And the lore scene "hanjin_ferry"
    When I choose 1
    Then the flag "hanjin_ferry_record" is present
    And the lore scene "huarong"
    When I choose 1
    Then the hidden conclusion "hanjin_huarong_debt" is unlocked
