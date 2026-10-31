Feature: Chapters 052-054 fixed Mao and Jiajing local variants with playable responsibility
  Scenario: Zhuge carries the Nanjun lease into the Jingzhou pledge
    Given a "zhuge" campaign with seed 52054
    And the flag "jiangdong_council_record" was earned earlier
    And the lore scene "nanjun_claim_ledger"
    When I choose 1
    Then the flag "nanjun_claim_record" is present
    And the lore scene "jingxiang_lease_ledger"
    When I choose 1
    Then the flag "jingxiang_lease_record" is present
    And the lore scene "jingzhou_pledge_copy"
    When I choose 1
    Then the flag "jingzhou_pledge_copy_record" is present
    When I save and restore
    Then the flag "jingzhou_pledge_copy_record" is present

  Scenario: Guan Yu's Changsha mercy is independent of Sun's Hefei ledger
    Given a "guanyu" campaign with seed 53054
    And the flag "huarong_mercy" was earned earlier
    And the lore scene "changsha_mercy"
    When I choose 1
    Then the flag "changsha_mercy_record" is present

  Scenario: Refusing Sun's Hefei record does not forge a Changsha record
    Given a "sun" campaign with seed 53055
    And the flag "jiangdong_muster_record" was earned earlier
    And the lore scene "hefei_memorial"
    When I choose 2
    Then the flag "hefei_casualty_record" is absent
    And the flag "changsha_mercy_record" is absent
