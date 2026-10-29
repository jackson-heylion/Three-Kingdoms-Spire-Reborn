Feature: Chapters 001-003 early literary variants and historical accountability

  Scenario: Guan preserves Taoyuan and Anxi ledgers to unlock one source debt
    Given a "guan" campaign with seed 1001
    And the lore scene "taoyuan"
    When I choose 1
    Then the flag "taoyuan_oath" is present
    And the lore scene "anxi"
    When I choose 1
    Then the flag "anxi_seal" is present
    And the hidden conclusion "taoyuan_anxi_source_debt" is unlocked
    When I save and restore
    Then the flag "anxi_seal" is present
    And the hidden conclusion "taoyuan_anxi_source_debt" is unlocked

  Scenario: Guan refuses Anxi and cannot record its accepted flag
    Given a "guan" campaign with seed 1002
    And the lore scene "taoyuan"
    When I choose 1
    And the lore scene "anxi"
    When I choose 2
    Then the flag "anxi_seal" is absent

  Scenario: Lu Bu records Ding Yuan's responsibility without rewriting the novel
    Given a "lu" campaign with seed 1003
    And the lore scene "redhare"
    When I choose 1
    Then the flag "red_hare" is present
    And my deck contains "blood_courage"
    When I save and restore
    Then the flag "red_hare" is present

  Scenario: Lu Bu refuses to accept the old Ding Yuan ledger
    Given a "lu" campaign with seed 1004
    And the lore scene "redhare"
    When I choose 3
    Then the flag "red_hare" is absent
