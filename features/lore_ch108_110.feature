Feature: Chapters 108-110 Dongxing snow, Caofang deposition and Taoxi responsibility

  Scenario: Dongxing bridge rescue pays the old Yiling civilian debt once
    Given a "sun" campaign with seed 108110
    And the flag "yiling_ashes" was earned earlier
    And the lore scene "dongxing_snow_bridge_ledger"
    When I choose 1
    Then the flag "dongxing_snow_bridge_record" is present
    And the hidden conclusion "yiling_dongxing_snow_debt" is unlocked
    When I save and restore
    Then the flag "dongxing_snow_bridge_record" is present

  Scenario: Refusing the Dongxing rescue cannot claim the old debt
    Given a "sun" campaign with seed 108111
    And the flag "yiling_ashes" was earned earlier
    And the lore scene "dongxing_snow_bridge_ledger"
    When I choose 2
    Then the flag "dongxing_snow_bridge_record" is absent

  Scenario: Xincheng fever and Sun Jun banquet remain distinct later archives
    Given a "sun" campaign with seed 108112
    And the lore scene "xincheng_siege_sickness"
    When I choose 1
    Then the flag "xincheng_sickness_record" is present
    And the lore scene "sun_jun_banquet_inquest"
    When I choose 1
    Then the flag "sun_jun_banquet_record" is present

  Scenario: Wuzhang withdrawal and Tielong water pay the later Shu debt
    Given a "zhuge" campaign with seed 108113
    And the flag "wuzhang_withdrawal_record" was earned earlier
    And the lore scene "qushan_crossbow_aftercare"
    When I choose 1
    And the lore scene "tielong_spring_rescue"
    When I choose 1
    Then the hidden conclusion "wuzhang_tielong_water_debt" is unlocked
    And the lore scene "taoxi_didao_withdrawal"
    When I choose 1
    Then the flag "taoxi_didao_withdrawal_record" is present

  Scenario: Gaoping oath and Shouchun forged edict pay one Wei successor debt
    Given a "cao" campaign with seed 108114
    And the flag "gaoping_oath_court_record" was earned earlier
    And the lore scene "caofang_deposition_receipt"
    When I choose 1
    And the lore scene "shouchun_false_edict_audit"
    When I choose 1
    Then the hidden conclusion "gaoping_shouchun_edict_debt" is unlocked
    When I save and restore
    Then the flag "shouchun_false_edict_record" is present
