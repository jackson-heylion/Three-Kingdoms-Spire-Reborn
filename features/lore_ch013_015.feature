Feature: Chapters 013-015 fixed novel variants and source-graded responsibility

  Scenario: Lu Bu keeps Xiaopei's guest-host ledger without changing the novel
    Given a "lu" campaign with seed 13013
    And the lore scene "xiaopei"
    When I choose 1
    Then the flag "xiaopei_refuge" is present
    When I save and restore
    Then the flag "xiaopei_refuge" is present

  Scenario: Cao preserves Xu capital and Xu Huang's refusal to kill his former lord
    Given a "cao" campaign with seed 14014
    And the lore scene "xudu"
    When I choose 1
    Then the flag "xu_capital" is present
    And the lore scene "xuhuang"
    When I choose 1
    Then the flag "xuhuang_honor" is present
    When I save and restore
    Then the flag "xuhuang_honor" is present

  Scenario: Sun's three accepted ledgers unlock the existing cross-act Taishi reward
    Given a "sun" campaign with seed 15015
    And the lore scene "sun_father_letter"
    When I choose 1
    And the lore scene "shenting"
    When I choose 1
    And the lore scene "sun_yiling_shelter"
    When I choose 1
    Then the hidden conclusion "taishi_return" is unlocked
    When I save and restore
    Then the hidden conclusion "taishi_return" is unlocked

  Scenario: Sun refuses Shenting's ledger and cannot claim its accepted flag
    Given a "sun" campaign with seed 15016
    And the lore scene "shenting"
    When I choose 2
    Then the flag "taishi_trust" is absent
