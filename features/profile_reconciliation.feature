Feature: Recover committed campaign starts after an interrupted chronicle write
  A new campaign's save.json may commit before its independent chronicle.json.
  The persisted start receipt allows recovery exactly once without double counting.

  Scenario: Replaying the same committed start across restarts is idempotent
    Given a "cao" campaign with seed 96154
    And I assign a recoverable start receipt
    When I reconcile the campaign from its saved snapshot
    Then exactly one campaign start is recorded
    When I reload and reconcile the same receipt
    Then exactly one campaign start is recorded

  Scenario: A committed victory repairs its missing permanent result once
    Given a "cao" campaign with seed 96155
    And I assign a recoverable start receipt
    And I mark the campaign as a completed victory
    When I reconcile the campaign from its saved snapshot
    Then exactly one campaign start is recorded
    And exactly one permanent campaign victory is recorded
    When I reload and reconcile the same receipt
    Then exactly one permanent campaign victory is recorded
