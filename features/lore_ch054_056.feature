Feature: Chapters 054-056 cross-hero novel accountability and legacy continuity
  Scenario: Zhuge records the Jingzhou pledge and detects the false passage
    Given a "zhuge" campaign with seed 5456
    And the flag "jingxiang_lease_record" was earned earlier
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "jingzhou_pledge_copy"
    When I choose 1
    Then the flag "jingzhou_pledge_copy_record" is present
    When I save and restore
    Then the flag "jingzhou_pledge_copy_record" is present
    And the lore scene "jingzhou_false_passage"
    When I choose 1
    Then the flag "jingzhou_false_passage_record" is present
    And the hidden conclusion "jingzhou_pledge_reckoning" is unlocked

  Scenario: Sun records her own Nanxu departure and later Yiling debt
    Given a "sun" campaign with seed 5556
    And the flag "jiangdong_covenant" was earned earlier
    And the lore scene "nanxu_escape_witness"
    When I choose 1
    Then the flag "nanxu_escape_witness_record" is present
    And the lore scene "yiling"
    When I choose 1
    Then the hidden conclusion "nanxu_yiling_witness" is unlocked

  Scenario: Cao records the Tongque dispatch and receives the older Guandu debt
    Given a "cao" campaign with seed 5656
    And the flag "guandu_grain" was earned earlier
    And the flag "river_banquet_record" was earned earlier
    And the lore scene "tongque_dispatch_ledger"
    When I choose 1
    Then the flag "tongque_dispatch_record" is present
    And the hidden conclusion "tongque_river_warning" is unlocked

  Scenario: Refusing the Nanxu ledger does not block the old Yiling route
    Given a "sun" campaign with seed 5557
    And the flag "jiangdong_covenant" was earned earlier
    And the lore scene "nanxu_escape_witness"
    When I choose 2
    Then the flag "nanxu_escape_witness_record" is absent
    And the lore scene "yiling"
    When I choose 1
    Then the flag "yiling_ashes" is present
