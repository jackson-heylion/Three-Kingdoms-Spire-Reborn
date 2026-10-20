Feature: Deterministic dynamic reaction subscriptions in unranked combat
  Scenario: A custom reaction becomes part of the saved action stream
    Given a custom "zhuge" campaign with seed 925144
    When I choose 1
    And I register a one-shot weak reduction listener
    Then the listener and complete action replay agree after save

  Scenario: Ranked campaigns cannot register free combat modifiers
    Given a "zhuge" campaign with seed 925144
    When I choose 1
    And I attempt to register a listener outside custom mode
    Then the action is rejected
    And the snapshot is unchanged
