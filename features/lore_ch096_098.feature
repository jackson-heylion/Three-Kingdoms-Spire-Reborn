Feature: Chapters 096-098 Jieting law, Shiting intelligence and Chencang supply consequences

  Scenario: Jieting evidence leads to one military-law and family conclusion
    Given a "zhuge" campaign with seed 9698
    And the flag "jieting_water_record" was earned earlier
    And the lore scene "masu_family_account"
    When I choose 1
    Then the flag "masu_family_record" is present
    And the hidden conclusion "jieting_masu_family_debt" is unlocked
    When I save and restore
    Then the flag "masu_family_record" is present

  Scenario: Refusing family accountability does not pay the Jieting conclusion
    Given a "zhuge" campaign with seed 9699
    And the flag "jieting_water_record" was earned earlier
    And the lore scene "masu_family_account"
    When I choose 2
    Then the flag "masu_family_record" is absent

  Scenario: Chencang siege and retreat preserve the two distinct logistics costs
    Given a "zhuge" campaign with seed 9700
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "chencang_siege_wounded_ledger"
    When I choose 1
    Then the flag "chencang_siege_record" is present
    And the lore scene "chencang_retreat_supply_audit"
    When I choose 1
    Then the flag "chencang_retreat_supply_record" is present
    And the hidden conclusion "redcliff_chencang_retreat_debt" is unlocked

  Scenario: Wu archives separate Shiting surrender from Huanglong envoy identity
    Given a "sun" campaign with seed 9701
    And the lore scene "shiting_false_defection_audit"
    When I choose 1
    Then the flag "shiting_spy_record" is present
    And the lore scene "wu_han_enthronement_dispatch"
    When I choose 2
    Then the flag "wu_han_enthronement_record" is absent
