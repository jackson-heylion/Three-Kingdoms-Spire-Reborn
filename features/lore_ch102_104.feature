Feature: Chapters 102-104 Weibei bridge, wooden ox, Upper Valley and Wuzhang succession

  Scenario: Wooden ox and Wuzhang succession award one recorded supply debt
    Given a "zhuge" campaign with seed 102104
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "wooden_ox_transport_audit"
    When I choose 1
    Then the flag "weinan_transport_record" is present
    And the lore scene "wuzhang_succession_register"
    When I choose 1
    Then the hidden conclusion "redcliff_wuzhang_transport_debt" is unlocked
    When I save and restore
    Then the flag "wuzhang_succession_record" is present

  Scenario: Refusing the transport audit cannot unlock the supply debt
    Given a "zhuge" campaign with seed 102105
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "wooden_ox_transport_audit"
    When I choose 2
    Then the flag "weinan_transport_record" is absent

  Scenario: Bridge losses and fire risk must precede the withdrawal conclusion
    Given a "zhuge" campaign with seed 102106
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "weibei_bridge_loss_ledger"
    When I choose 1
    Then the flag "weibei_bridge_losses_record" is present
    And the lore scene "wooden_ox_transport_audit"
    When I choose 1
    And the lore scene "shangfang_fire_account"
    When I choose 1
    Then the flag "shangfang_fire_account_record" is present
    And the lore scene "wuzhang_withdrawal_evidence"
    When I choose 1
    Then the hidden conclusion "weibei_shangfang_withdrawal_debt" is unlocked

  Scenario: Wu three-front withdrawal is a later archive not a living Sun heroine
    Given a "sun" campaign with seed 102107
    And the flag "jiangdong_covenant" was earned earlier
    And the lore scene "wu_threefront_retreat_archive"
    When I choose 1
    Then the flag "wu_threefront_retreat_record" is present
    When I save and restore
    Then the flag "wu_threefront_retreat_record" is present
