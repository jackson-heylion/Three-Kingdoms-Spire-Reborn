Feature: Chapters 022-024 fixed Mao and Jiajing variants with optional records
  Scenario: Guan keeps the prisoner-release and household ledgers
    Given a "guan" campaign with seed 22024
    And the flag "wenjiu_oath" was earned earlier
    And the lore scene "wangzhong_prisoners"
    When I choose 1
    Then the flag "wangzhong_release_record" is present
    And the lore scene "xiapi_household"
    When I choose 1
    Then the flag "xiapi_household_ledger" is present
    When I save and restore
    Then the flag "xiapi_household_ledger" is present

  Scenario: Cao records Jiping but refuses to falsify the palace ledger
    Given a "cao" campaign with seed 23024
    And the flag "xuhuang_honor" was earned earlier
    And the lore scene "jiping_testimony"
    When I choose 1
    Then the flag "jiping_case_record" is present
    And the lore scene "palace_ledger"
    When I choose 2
    Then the flag "palace_purge_record" is absent
    When I save and restore
    Then the flag "palace_purge_record" is absent

  Scenario: Refusing Guan's optional prisoner record does not forge a release
    Given a "guan" campaign with seed 22025
    And the flag "wenjiu_oath" was earned earlier
    And the lore scene "wangzhong_prisoners"
    When I choose 2
    Then the flag "wangzhong_release_record" is absent


  Scenario: Rejecting Jiping testimony survives V2 and leaves the old Guandu route playable
    Given a "cao" campaign with seed 220240
    And the flag "xuhuang_honor" was earned earlier
    And the lore scene "jiping_testimony"
    When I choose 2
    Then the flag "jiping_case_record" is absent
    When I save and restore
    Then the flag "jiping_case_record" is absent
    And the flag "xu_capital" was earned earlier
    And the lore scene "guandu"
    When I choose 1
    Then the flag "guandu_grain" is present
