Feature: Chapters 046-048 independent Mao reread keeps original optional paths
  Scenario: Refusing arrow evidence cannot lock Red Cliffs
    Given a "zhuge" campaign with seed 460481
    And the flag "longzhong_plan" was earned earlier
    And the lore scene "borrow_arrows"
    When I choose 2
    Then the flag "mist_arrows" is absent
    When I save and restore
    Then the flag "mist_arrows" is absent
    And the lore scene "redcliff"
    When I choose 1
    Then the flag "redcliff_wind" is present

  Scenario: Refusing Cao's chained ship record cannot lock Liaodong
    Given a "cao" campaign with seed 470481
    And the flag "guandu_grain" was earned earlier
    And the lore scene "jiangkou_inquest"
    When I choose 1
    Then the flag "jiangkou_inquest_record" is present
    And the lore scene "chain_ship_ledger"
    When I choose 2
    Then the flag "chain_ship_risk_record" is absent
    When I save and restore
    Then the flag "chain_ship_risk_record" is absent
    And the lore scene "liaodong"
    When I choose 1
    Then the flag "liaodong_letter" is present
