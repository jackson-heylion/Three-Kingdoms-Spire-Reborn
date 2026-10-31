Feature: Chapters 055-057 fixed edition differences and optional responsibility records
  Scenario: Sun keeps Nanxu and later river-child testimony once
    Given a "sun" campaign with seed 55057
    And the flag "jiangdong_covenant" was earned earlier
    And the lore scene "nanxu_escape_witness"
    When I choose 1
    Then the flag "nanxu_escape_witness_record" is present
    And the lore scene "river_child_witness"
    When I choose 1
    Then the flag "river_child_witness_record" is present
    And the hidden conclusion "nanxu_river_witness" is unlocked
    When I save and restore
    Then the flag "river_child_witness_record" is present

  Scenario: Cao keeps diplomatic and Mateng accountability but cannot revive the dead
    Given a "cao" campaign with seed 56057
    And the flag "guandu_grain" was earned earlier
    And the flag "river_banquet_record" was earned earlier
    And the lore scene "tongque_dispatch_ledger"
    When I choose 1
    Then the flag "tongque_dispatch_record" is present
    And the lore scene "mateng_case"
    When I choose 1
    Then the flag "mateng_case_record" is present
    When I save and restore
    Then the flag "mateng_case_record" is present

  Scenario: Refusing the false-passage account does not forge a guard ledger
    Given a "zhuge" campaign with seed 56058
    And the flag "jingzhou_pledge_copy_record" was earned earlier
    And the lore scene "jingzhou_false_passage"
    When I choose 2
    Then the flag "jingzhou_false_passage_record" is absent
