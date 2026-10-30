Feature: Chapters 084-086 Yiling losses, Yong'an regency and Deng Zhi alliance

  Scenario: Zhuge records delayed fire warning, regency and envoy responsibility
    Given a "zhuge" campaign with seed 8486
    And the flag "redcliff_wind" was earned earlier
    And the flag "yiling_camp_map_record" was earned earlier
    And the lore scene "fishbelly_eightfold_report"
    When I choose 1
    Then the flag "fishbelly_report_record" is present
    And the lore scene "yongan_regency_register"
    When I choose 1
    Then the flag "yongan_regency_record" is present
    And the lore scene "dengzhi_alliance_dispatch"
    When I choose 1
    Then the flag "dengzhi_alliance_record" is present
    And the hidden conclusion "redcliff_yongan_envoy_debt" is unlocked
    When I save and restore
    Then the flag "dengzhi_alliance_record" is present

  Scenario: Zhuge refuses to record the fire warning and loses its flag
    Given a "zhuge" campaign with seed 8487
    And the flag "yiling_camp_map_record" was earned earlier
    And the lore scene "fishbelly_eightfold_report"
    When I choose 2
    Then the flag "fishbelly_report_record" is absent

  Scenario: Sun archival perspective retains both civilian ledgers
    Given a "sun" campaign with seed 8488
    And the flag "jiangdong_covenant" was earned earlier
    And the flag "yiling_defense_record" was earned earlier
    And the lore scene "yiling_fire_refuge_ledger"
    When I choose 1
    Then the flag "yiling_fire_refuge_record" is present
    And the lore scene "guangling_river_defense"
    When I choose 1
    Then the flag "guangling_defense_record" is present
    And the hidden conclusion "yiling_guangling_civilian_debt" is unlocked

  Scenario: Wei posthumous court audits the failed three-front campaign
    Given a "cao" campaign with seed 8489
    And the flag "guandu_grain" was earned earlier
    And the flag "wei_wu_tribute_record" was earned earlier
    And the lore scene "wei_three_fronts_review"
    When I choose 1
    Then the flag "wei_three_fronts_record" is present
    And the hidden conclusion "guandu_wei_three_fronts_debt" is unlocked

  Scenario: Sun keeps the 082-084 envoy, defensive and fire relief ledgers
    Given a "sun" campaign with seed 82084
    And the flag "jiangdong_covenant" was earned earlier
    And the flag "langzhong_heads_record" was earned earlier
    And the lore scene "wu_wei_envoy_audit"
    When I choose 1
    Then the flag "wu_wei_envoy_record" is present
    And the lore scene "yiling_defense_memorial"
    When I choose 1
    Then the flag "yiling_defense_record" is present
    And the lore scene "yiling_fire_refuge_ledger"
    When I choose 1
    Then the flag "yiling_fire_refuge_record" is present
    And the hidden conclusion "yiling_envoy_fire_civilian_debt" is unlocked
    When I save and restore
    Then the flag "yiling_fire_refuge_record" is present

  Scenario: Sun refuses the fire relief account and receives no record
    Given a "sun" campaign with seed 83084
    And the flag "yiling_defense_record" was earned earlier
    And the lore scene "yiling_fire_refuge_ledger"
    When I choose 2
    Then the flag "yiling_fire_refuge_record" is absent
