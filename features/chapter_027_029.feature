Feature: Chapters 27–29 document escort, Gucheng testimony and Jiangdong succession
  All three chapters use pinned single-witness readings; alternate history and original choices stay separate.

  Scenario: Guan Yu keeps the five passes and Gucheng witness ledger until Huarong
    Given a "guanyu" campaign with seed 2728
    And the flag "seal_return_record" was earned earlier
    And the flag "baima_debt" was earned earlier
    And the lore scene "five_passes"
    When I choose 1
    Then the flag "five_passes_record" is present
    And my deck contains "hold_line"
    And the bond with "普净" is active
    And the lore scene "gucheng_witness"
    When I choose 1
    Then the flag "gucheng_witness_ledger" is present
    When I save and restore
    Then the flag "gucheng_witness_ledger" is present
    And act 1 offers the lore scene "huarong"
    And the lore scene "huarong"
    When I choose 1
    Then the hidden conclusion "five_passes_huarong" is unlocked

  Scenario: Refusing the five passes ledger preserves the old Huarong route
    Given a "guanyu" campaign with seed 2729
    And the flag "seal_return_record" was earned earlier
    And the flag "baima_debt" was earned earlier
    And the lore scene "five_passes"
    When I choose 2
    Then the flag "five_passes_record" is absent
    And act 1 offers the lore scene "huarong"
    And the lore scene "huarong"
    When I choose 1
    Then the hidden conclusion "sword_debt" is unlocked
    And the flag "gucheng_witness_ledger" is absent

  Scenario: Sun family succession record pays after the old wedding and Yiling
    Given a "sun" campaign with seed 2929
    And the flag "taishi_trust" was earned earlier
    And the lore scene "sun_succession"
    When I choose 1
    Then the flag "sun_succession_account" is present
    And the bond with "孙权" is active
    When I save and restore
    Then the flag "sun_succession_account" is present
    And the lore scene "jiangdong_wedding"
    When I choose 1
    Then the flag "jiangdong_covenant" is present
    And act 2 offers the lore scene "yiling"
    And the lore scene "yiling"
    When I choose 1
    Then the hidden conclusion "jiangdong_succession_debt" is unlocked

  Scenario: Sun may refuse succession record without losing the legacy Yiling path
    Given a "sun" campaign with seed 2930
    And the flag "taishi_trust" was earned earlier
    And the lore scene "sun_succession"
    When I choose 2
    Then the flag "sun_succession_account" is absent
    And the lore scene "jiangdong_wedding"
    When I choose 1
    Then the flag "jiangdong_covenant" is present
    And act 2 offers the lore scene "yiling"
