Feature: Chapters 051-053 optional narrative accountability
  Scenario: Zhuge records Nanjun and Jingxiang debts
    Given a "zhuge" campaign with seed 5152
    And the flag "jiangdong_council_record" was earned earlier
    And the lore scene "nanjun_claim_ledger"
    When I choose 1
    Then the flag "nanjun_claim_record" is present
    And the lore scene "jingxiang_lease_ledger"
    When I choose 1
    Then the flag "jingxiang_lease_record" is present
    And the hidden conclusion "nanjun_jingxiang_debt" is unlocked

  Scenario: Guan Yu records Changsha mercy
    Given a "guanyu" campaign with seed 5153
    And the flag "huarong_mercy" was earned earlier
    And the lore scene "changsha_mercy"
    When I choose 1
    Then the flag "changsha_mercy_record" is present
    When I save and restore
    Then the flag "changsha_mercy_record" is present

  Scenario: Sun family Hefei debt is paid after Yiling
    Given a "sun" campaign with seed 5353
    And the flag "jiangdong_muster_record" was earned earlier
    And the lore scene "hefei_memorial"
    When I choose 1
    Then the flag "hefei_casualty_record" is present
    And the lore scene "yiling"
    When I choose 1
    Then the flag "yiling_ashes" is present
    And the hidden conclusion "hefei_yiling_memorial" is unlocked
