Feature: P1 durable command commits
  A failed local save must never publish an action to the current session.

  Scenario: Failed action save can retry without duplicating the action
    Given an isolated save directory
    And a "guanyu" campaign with seed 96102
    When I write the initial campaign snapshot
    And I block backup replacement with a directory
    And I remember the current snapshot
    And I attempt a persisted choice while disk write is blocked
    Then the action is rejected
    And the snapshot is unchanged
    And the disk snapshot matches the current campaign
    When I unblock save replacement
    And I retry the persisted choice
    Then the campaign revision is exactly one
    And the disk snapshot matches the current campaign

  Scenario: A failed new-run save retains the current campaign until retry
    Given an isolated save directory
    And a "guanyu" campaign with seed 96115
    When I write the initial campaign snapshot
    And I block backup replacement with a directory
    And I remember the current snapshot
    And I attempt a persisted new campaign while disk write is blocked
    Then the action is rejected
    And the snapshot is unchanged
    And the disk snapshot matches the current campaign
    When I unblock save replacement
    And I retry the persisted new campaign
    Then the replacement campaign seed is 96114
    And the disk snapshot matches the current campaign
