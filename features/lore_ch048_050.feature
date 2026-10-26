Feature: Chapters 048-050 distinguish novel Red Cliffs from history and keep original outcomes
  Cao's optional retrospective adds a once-only debt; existing Red Cliffs and Huarong remain playable.

  Scenario: Cao audits the false letter, chained ships and ignored banquet warning
    Given a "cao" campaign with seed 4850
    And the flag "guandu_grain" was earned earlier
    And the lore scene "jiangkou_inquest"
    When I choose 1
    Then the flag "jiangkou_inquest_record" is present
    And the lore scene "chain_ship_ledger"
    When I choose 1
    Then the flag "chain_ship_risk_record" is present
    When I save and restore
    Then the flag "chain_ship_risk_record" is present
    And the lore scene "river_banquet_inquest"
    When I choose 1
    Then the flag "river_banquet_record" is present
    And the hidden conclusion "redcliff_warning_ledger" is unlocked

  Scenario: Refusing the chapter 048 ledger never blocks the old Liaodong route
    Given a "cao" campaign with seed 4851
    And the flag "guandu_grain" was earned earlier
    And the flag "chain_ship_risk_record" was earned earlier
    And the lore scene "river_banquet_inquest"
    When I choose 2
    Then the flag "river_banquet_record" is absent
    And the lore scene "liaodong"
    When I choose 1
    Then the flag "liaodong_letter" is present

  Scenario: Old Zhuge Red Cliffs and Guan Yu Huarong scenes need no new chapter 048 flag
    Given a "zhuge" campaign with seed 4852
    And the flag "longzhong_plan" was earned earlier
    And the lore scene "redcliff"
    When I choose 1
    Then the flag "redcliff_wind" is present
    And the lore scene "redcliff"
    When I choose 2
    Then the flag "redcliff_wind" is present

  Scenario: Huarong original mercy remains a choice without the Cao ledger
    Given a "guanyu" campaign with seed 4853
    And the flag "baima_debt" was earned earlier
    And the lore scene "huarong"
    When I choose 1
    Then the flag "huarong_mercy" is present
