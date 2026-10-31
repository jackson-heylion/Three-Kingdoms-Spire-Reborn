Feature: Chapters 042-044 record ferry, alliance and command responsibility
  The historical and novel outcomes remain fixed; optional ledgers may change resources.

  Scenario: Guan Yu remembers the Xutian warning and Hanjin ferry before Huarong
    Given a "guanyu" campaign with seed 4201
    And the flag "xutian_guarded_emperor" was earned earlier
    And the lore scene "hanjin_ferry"
    When I choose 1
    Then the flag "hanjin_ferry_record" is present
    And my deck contains "hold_line"
    When I save and restore
    Then the flag "hanjin_ferry_record" is present
    And the lore scene "huarong"
    When I choose 1
    Then the hidden conclusion "hanjin_huarong_debt" is unlocked

  Scenario: Guan Yu may refuse the ferry ledger and still choose Huarong
    Given a "guanyu" campaign with seed 4202
    And the flag "xutian_guarded_emperor" was earned earlier
    And the lore scene "hanjin_ferry"
    When I choose 2
    Then the flag "hanjin_ferry_record" is absent
    And the lore scene "huarong"
    When I choose 1
    Then the flag "huarong_mercy" is present

  Scenario: Zhuge records Lu Su's debate and Zhou Yu's command after Dangyang
    Given a "zhuge" campaign with seed 4301
    And the flag "dangyang_refuge_account" was earned earlier
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "chaisang_envoy"
    When I choose 1
    Then the flag "chaisang_envoy_record" is present
    And act 2 offers the lore scene "jiangdong_council"
    And the lore scene "jiangdong_council"
    When I choose 1
    Then the flag "jiangdong_council_record" is present
    And the hidden conclusion "chaisang_redcliff_debt" is unlocked

  Scenario: Sun records Jiangdong mobilization without changing the old marriage and Yiling
    Given a "sun" campaign with seed 4401
    And the flag "jiangdong_hostage_record" was earned earlier
    And the lore scene "jiangdong_muster"
    When I choose 1
    Then the flag "jiangdong_muster_record" is present
    And the lore scene "jiangdong_wedding"
    When I choose 1
    Then the flag "jiangdong_covenant" is present
    And the lore scene "yiling"
    When I choose 1
    Then the hidden conclusion "jiangdong_muster_debt" is unlocked

  Scenario: Chaisang edition notes preserve Zhuge's optional record after restore
    Given a "zhuge" campaign with seed 43045
    And the flag "dangyang_refuge_account" was earned earlier
    And the lore scene "chaisang_envoy"
    When I choose 1
    Then the flag "chaisang_envoy_record" is present
    When I save and restore
    Then the flag "chaisang_envoy_record" is present
    And the lore scene "jiangdong_council"
    When I choose 1
    Then the flag "jiangdong_council_record" is present

  Scenario: Refusing the council report does not create a command record
    Given a "zhuge" campaign with seed 44045
    And the flag "chaisang_envoy_record" was earned earlier
    And the lore scene "jiangdong_council"
    When I choose 2
    Then the flag "jiangdong_council_record" is absent
    When I save and restore
    Then the flag "jiangdong_council_record" is absent
