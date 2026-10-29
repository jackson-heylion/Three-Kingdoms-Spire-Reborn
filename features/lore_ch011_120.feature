Feature: Chapter 011 archives and 120 reunification successor debts

  Scenario: Beihai rescue and later Xuzhou accountability pay once
    Given a "guan" campaign with seed 11120
    And the flag "taoyuan_oath" was earned earlier
    And the flag "xuzhou_gate_account" was earned earlier
    And the lore scene "beihai_xuzhou_relief_archive"
    When I choose 1
    Then the hidden conclusion "beihai_xuzhou_civil_debt" is unlocked
    When I save and restore
    Then the flag "beihai_xuzhou_relief_record" is present

  Scenario: Cao records Yanzhou alarm rather than claiming the Xu letter alone ended siege
    Given a "cao" campaign with seed 11121
    And the flag "xuzhou_debt" was earned earlier
    And the flag "puyang_supply" was earned earlier
    And the lore scene "xuzhou_yanzhou_dispatch_archive"
    When I choose 1
    Then the hidden conclusion "xuzhou_yanzhou_ration_debt" is unlocked

  Scenario: Lu Bu acknowledges unused Taishan ambush and later Xiaopei refuge
    Given a "lubu" campaign with seed 11122
    And the flag "changan_account" was earned earlier
    And the flag "xiaopei_refuge" was earned earlier
    And the lore scene "yanzhou_taishan_ambush_archive"
    When I choose 1
    Then the hidden conclusion "changan_yanzhou_xiaopei_debt" is unlocked

  Scenario: Wu court records both Sun Hao memorial and Stone City surrender
    Given a "sun" campaign with seed 12001
    And the flag "wu_relief_arrival_dispute_record" was earned earlier
    And the lore scene "sunhao_lukang_memorial"
    When I choose 1
    And the lore scene "wu_stonecity_surrender_roll"
    When I choose 1
    Then the hidden conclusion "wu_sunhao_stonecity_debt" is unlocked

  Scenario: Jin court records border settlement and Wang Jun fleet without killing surrenderers
    Given a "cao" campaign with seed 12002
    And the flag "wei_jin_abdication_record" was earned earlier
    And the lore scene "jin_yanghu_border_ledger"
    When I choose 1
    And the lore scene "jin_duyu_campaign_ledger"
    When I choose 1
    And the lore scene "jin_wangjun_chain_ledger"
    When I choose 1
    Then the hidden conclusion "jin_yanghu_wangjun_debt" is unlocked
    When I save and restore
    Then the flag "jin_wangjun_chain_record" is present

  Scenario: Refusing Stone City household record does not claim Wu civilian debt
    Given a "sun" campaign with seed 12003
    And the flag "wu_relief_arrival_dispute_record" was earned earlier
    And the lore scene "sunhao_lukang_memorial"
    When I choose 1
    And the lore scene "wu_stonecity_surrender_roll"
    When I choose 2
    Then the flag "wu_stonecity_surrender_record" is absent
