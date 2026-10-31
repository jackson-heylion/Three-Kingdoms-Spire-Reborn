Feature: Chapters 045-047 preserve Cao's failed intelligence and chained-ship responsibility
  Novel outcomes remain unchanged, and the existing Zhuge arrow-borrowing scene is reused.

  Scenario: Cao records the forged letter and the chain-ship warning across acts
    Given a "cao" campaign with seed 4547
    And the flag "guandu_grain" was earned earlier
    And the lore scene "jiangkou_inquest"
    When I choose 1
    Then the flag "jiangkou_inquest_record" is present
    When I save and restore
    Then the flag "jiangkou_inquest_record" is present
    And the lore scene "chain_ship_ledger"
    When I choose 1
    Then the flag "chain_ship_risk_record" is present
    And the hidden conclusion "river_spy_chain_debt" is unlocked

  Scenario: Refusing the new false-letter record cannot lock the old Liaodong arc
    Given a "cao" campaign with seed 4548
    And the flag "guandu_grain" was earned earlier
    And the lore scene "jiangkou_inquest"
    When I choose 2
    Then the flag "jiangkou_inquest_record" is absent
    And act 2 offers the lore scene "liaodong"

  Scenario: The existing Zhuge arrow choice survives fixed edition count differences
    Given a "zhuge" campaign with seed 46048
    And the flag "longzhong_plan" was earned earlier
    And the lore scene "borrow_arrows"
    When I choose 1
    Then the flag "mist_arrows" is present
    When I save and restore
    Then the flag "mist_arrows" is present
