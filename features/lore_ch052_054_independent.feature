Feature: Mao 052-054 fixed chapter evidence preserves optional playable responsibility

  Scenario: Zhuge keeps Jingxiang evidence before the signed pledge
    Given a "zhuge" campaign with seed 52054
    And the flag "nanjun_claim_record" was earned earlier
    And the lore scene "jingxiang_lease_ledger"
    When I choose 1
    Then the flag "jingxiang_lease_record" is present
    When I save and restore
    Then the flag "jingxiang_lease_record" is present
    And the lore scene "jingzhou_pledge_copy"
    When I choose 1
    Then the flag "jingzhou_pledge_copy_record" is present

  Scenario: Refusing Jingxiang evidence cannot forge the later signed pledge
    Given a "zhuge" campaign with seed 52055
    And the flag "nanjun_claim_record" was earned earlier
    And the lore scene "jingxiang_lease_ledger"
    When I choose 2
    Then the flag "jingxiang_lease_record" is absent
    When I save and restore
    Then the flag "jingxiang_lease_record" is absent
    And the flag "jingzhou_pledge_copy_record" is absent

  Scenario: Guan records the Changsha mercy debt without altering the Hefei witness
    Given a "guan" campaign with seed 53054
    And the flag "huarong_mercy" was earned earlier
    And the lore scene "changsha_mercy"
    When I choose 1
    Then the flag "changsha_mercy_record" is present
    And the flag "hefei_casualty_record" is absent

  Scenario: Sun declines the Hefei casualty ledger without a false flag
    Given a "sun" campaign with seed 53055
    And the flag "jiangdong_muster_record" was earned earlier
    And the lore scene "hefei_memorial"
    When I choose 2
    Then the flag "hefei_casualty_record" is absent
    When I save and restore
    Then the flag "hefei_casualty_record" is absent
