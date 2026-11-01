Feature: Chapters 010-012 pinned Mao and Jiajing local witnesses and optional playable debts

  Scenario: Cao preserves Xuzhou and Yanzhou reports before Puyang ration settlement
    Given a "cao" campaign with seed 101201
    And the lore scene "xuzhou"
    When I choose 1
    Then the flag "xuzhou_debt" is present
    And the lore scene "xuzhou_yanzhou_dispatch_archive"
    When I choose 1
    Then the flag "xuzhou_yanzhou_dispatch_record" is present
    And the lore scene "puyang"
    When I choose 1
    Then the hidden conclusion "xuzhou_yanzhou_ration_debt" is unlocked
    When I save and restore
    Then the flag "puyang_supply" is present

  Scenario: Guan keeps Beihai civilian record and later Xuzhou family liability
    Given a "guan" campaign with seed 111201
    And the flag "taoyuan_oath" was earned earlier
    And the flag "xuzhou_gate_account" was earned earlier
    And the lore scene "beihai_xuzhou_relief_archive"
    When I choose 1
    Then the flag "beihai_xuzhou_relief_record" is present
    And the hidden conclusion "beihai_xuzhou_civil_debt" is unlocked
    When I save and restore
    Then the hidden conclusion "beihai_xuzhou_civil_debt" is unlocked

  Scenario: Lu Bu refuses to forge a Taishan ambush victory
    Given a "lubu" campaign with seed 111202
    And the flag "changan_account" was earned earlier
    And the flag "xiaopei_refuge" was earned earlier
    And the lore scene "yanzhou_taishan_ambush_archive"
    When I choose 2
    Then the flag "yanzhou_taishan_ambush_record" is absent

  Scenario: Cao declines the Yanzhou dispatch and restoration cannot invent the report
    Given a "cao" campaign with seed 101209
    And the flag "xuzhou_debt" was earned earlier
    And the lore scene "xuzhou_yanzhou_dispatch_archive"
    When I choose 3
    Then the flag "xuzhou_yanzhou_dispatch_record" is absent
    When I save and restore
    Then the flag "xuzhou_yanzhou_dispatch_record" is absent
