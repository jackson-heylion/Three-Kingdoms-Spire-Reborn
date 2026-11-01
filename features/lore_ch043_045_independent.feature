Feature: Independent fixed Mao 043-045 responsibility choices remain optional
  Scenario: Cao preserves the false-letter inquiry through the later chain-ship risk
    Given a "cao" campaign with seed 430450
    And the flag "guandu_grain" was earned earlier
    And the lore scene "jiangkou_inquest"
    When I choose 1
    Then the flag "jiangkou_inquest_record" is present
    And the lore scene "chain_ship_ledger"
    When I choose 1
    Then the flag "chain_ship_risk_record" is present
    When I save and restore
    Then the flag "chain_ship_risk_record" is present

  Scenario: Refusing the false-letter inquiry does not forge a flag or softlock old Liaodong
    Given a "cao" campaign with seed 430451
    And the flag "guandu_grain" was earned earlier
    And the lore scene "jiangkou_inquest"
    When I choose 2
    Then the flag "jiangkou_inquest_record" is absent
    When I save and restore
    Then the flag "jiangkou_inquest_record" is absent
    And the lore scene "liaodong"
    When I choose 1
    Then the flag "liaodong_letter" is present
