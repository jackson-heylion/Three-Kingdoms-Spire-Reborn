Feature: Chapters 105-107 successor archives, Liaodong rain and Gaoping oath

  Scenario: Wuzhang withdrawal, double memorial and Qushan water grant one successor debt
    Given a "zhuge" campaign with seed 105107
    And the flag "wuzhang_withdrawal_record" was earned earlier
    And the lore scene "weiyan_double_memorial_audit"
    When I choose 1
    Then the flag "weiyan_double_memorial_record" is present
    And the lore scene "qushan_water_rescue_dispatch"
    When I choose 1
    Then the hidden conclusion "wuzhang_qushan_supply_debt" is unlocked
    When I save and restore
    Then the flag "qushan_water_rescue_record" is present

  Scenario: Refusing the Wei Yan double memorial cannot pay the Qushan debt
    Given a "zhuge" campaign with seed 105108
    And the flag "wuzhang_withdrawal_record" was earned earlier
    And the lore scene "weiyan_double_memorial_audit"
    When I choose 2
    Then the flag "weiyan_double_memorial_record" is absent

  Scenario: Wei successor court keeps labor siege regency and Gaoping promises
    Given a "cao" campaign with seed 105109
    And the lore scene "wei_palace_labor_register"
    When I choose 1
    And the lore scene "liaodong_rain_siege_ledger"
    When I choose 1
    And the lore scene "wei_regency_sickbed_register"
    When I choose 1
    And the lore scene "gaoping_oath_court_review"
    When I choose 1
    Then the hidden conclusion "wei_palace_gaoping_oath_debt" is unlocked
    When I save and restore
    Then the flag "gaoping_oath_court_record" is present

  Scenario: Wu border condolence remains an optional later court archive
    Given a "sun" campaign with seed 105110
    And the lore scene "zongyu_wu_border_receipt"
    When I choose 1
    Then the flag "zongyu_border_reply_record" is present
    When I save and restore
    Then the flag "zongyu_border_reply_record" is present
