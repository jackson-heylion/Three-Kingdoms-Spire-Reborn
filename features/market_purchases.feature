Feature: Market card purchases permanently grow the deck
  Military-market purchases must be inventory transfers, not gold-only transactions.

  Scenario: Bought card survives save restore and leaving the military market
    Given a "guanyu" campaign with seed 93401
    When I stage a market offering "huogong"
    And I buy market slot 1
    Then market gold is 45 and the shelf is empty
    And my permanent deck has exactly one "huogong"
    When I save and restore
    Then my permanent deck has exactly one "huogong"
    And market gold is 45 and the shelf is empty
    When I leave the market
    Then my permanent deck has exactly one "huogong"

  Scenario: Already sold market stock cannot be charged twice
    Given a "guanyu" campaign with seed 93402
    When I stage a market offering "huogong"
    And I buy market slot 1
    And I remember the current snapshot
    And I attempt to buy market slot 1
    Then the action is rejected
    And the snapshot is unchanged

  Scenario: Insufficient gold leaves deck stock revision and RNG intact
    Given a "zhuge" campaign with seed 93403
    When I stage a market offering "huogong"
    And I reduce market gold to 44
    And I remember the current snapshot
    And I attempt to buy market slot 1
    Then the action is rejected
    And the snapshot is unchanged
