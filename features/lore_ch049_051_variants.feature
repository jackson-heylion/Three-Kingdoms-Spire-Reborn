Feature: Chapters 049-051 electronic witnesses preserve the existing playable choices
  Scenario: Red Cliffs witness survives a V2 restore
    Given a "zhuge" campaign with seed 49051
    And the flag "longzhong_plan" was earned earlier
    And the lore scene "redcliff"
    When I choose 1
    Then the flag "redcliff_wind" is present
    When I save and restore
    Then the flag "redcliff_wind" is present

  Scenario: Huarong refusal does not mint the mercy ledger
    Given a "guanyu" campaign with seed 50051
    And the flag "baima_debt" was earned earlier
    And the lore scene "huarong"
    When I choose 2
    Then the flag "huarong_mercy" is absent
    When I save and restore
    Then the flag "huarong_mercy" is absent

  Scenario: Nanjun remains an optional predecessor of Jingxiang
    Given a "zhuge" campaign with seed 51052
    And the flag "jiangdong_council_record" was earned earlier
    And the lore scene "nanjun_claim_ledger"
    When I choose 1
    Then the flag "nanjun_claim_record" is present
    And the lore scene "jingxiang_lease_ledger"
    When I choose 1
    Then the flag "jingxiang_lease_record" is present
