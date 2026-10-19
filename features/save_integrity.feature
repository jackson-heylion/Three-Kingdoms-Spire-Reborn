Feature: Save-state integrity before load and exit
  Parsed JSON is not necessarily a valid game. Reject impossible values,
  keep the live session intact and allow a valid snapshot to be resumed.

  Scenario: Forged negative gold must not be accepted as a recoverable campaign
    Given a "guanyu" campaign with seed 96013
    When I forge a negative-gold save document
    Then the forged save is rejected as "存档属性无效"
    And the snapshot is unchanged
    When I save and restore
    Then the snapshot is unchanged

  Scenario: An enemy with zero maximum HP must not enter a live battle
    Given a "guanyu" campaign with seed 96014
    When I enter the first battle for save validation
    And I forge a zero-health-cap enemy save document
    Then the forged save is rejected as "无效战斗数据"
    And the snapshot is unchanged
    When I save and restore
    Then the snapshot is unchanged
