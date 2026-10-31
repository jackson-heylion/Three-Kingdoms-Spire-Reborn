Feature: Chapters 025-027 fixed Mao and Jiajing variants, optional Guan ledgers
  Scenario: Guan preserves three terms, returns the seal and reaches Huarong without duplicate debts
    Given a "guan" campaign with seed 25027
    And the flag "wenjiu_oath" was earned earlier
    And the lore scene "baima"
    When I choose 1
    Then the flag "baima_debt" is present
    And the lore scene "baima_terms"
    When I choose 1
    Then the flag "three_terms_record" is present
    And the lore scene "seal_return"
    When I choose 1
    Then the flag "seal_return_record" is present
    And the lore scene "five_passes"
    When I choose 1
    Then the flag "five_passes_record" is present
    When I save and restore
    Then the flag "five_passes_record" is present
    And the lore scene "gucheng_witness"
    When I choose 1
    Then the flag "gucheng_witness_ledger" is present
    And the lore scene "huarong"
    When I choose 1
    Then the hidden conclusion "three_terms_huarong" is unlocked
    And the hidden conclusion "five_passes_huarong" is unlocked

  Scenario: Refusing the optional three terms ledger does not erase the older Baima route
    Given a "guan" campaign with seed 25028
    And the flag "wenjiu_oath" was earned earlier
    And the lore scene "baima"
    When I choose 1
    And the lore scene "baima_terms"
    When I choose 2
    Then the flag "three_terms_record" is absent
    And the lore scene "huarong"
    When I choose 1
    Then the hidden conclusion "sword_debt" is unlocked

  Scenario: Refusing the seal ledger never fabricates its accepted flag
    Given a "guan" campaign with seed 25029
    And the flag "three_terms_record" was earned earlier
    And the lore scene "seal_return"
    When I choose 2
    Then the flag "seal_return_record" is absent
