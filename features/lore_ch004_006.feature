Feature: Chapters 004-006 pinned literary variants and optional source debt

  Scenario: Cao records the assassination and seal provenance to earn a once-only debt
    Given a "cao" campaign with seed 4006
    And the lore scene "sevenstar"
    When I choose 1
    Then the flag "sevenstar_vow" is present
    And the lore scene "imperial_jade"
    When I choose 1
    Then the flag "jade_whisper" is present
    And the hidden conclusion "sevenstar_jade_source_debt" is unlocked
    When I save and restore
    Then the hidden conclusion "sevenstar_jade_source_debt" is unlocked

  Scenario: Cao refuses the seal responsibility and cannot claim the debt
    Given a "cao" campaign with seed 4007
    And the lore scene "sevenstar"
    When I choose 1
    And the lore scene "imperial_jade"
    When I choose 2
    Then the flag "jade_whisper" is absent

  Scenario: Guan records the historical attribution while preserving the novel's victory
    Given a "guan" campaign with seed 4008
    And the lore scene "wenjiu"
    When I choose 1
    Then the flag "wenjiu_oath" is present
    When I save and restore
    Then the flag "wenjiu_oath" is present

  Scenario: Guan declines the source ledger
    Given a "guan" campaign with seed 4009
    And the lore scene "wenjiu"
    When I choose 3
    Then the flag "wenjiu_oath" is absent

  Scenario: Cao refuses the sevenstar archive and V2 restore cannot fabricate its oath
    Given a "cao" campaign with seed 4010
    And the lore scene "sevenstar"
    When I choose 3
    Then the flag "sevenstar_vow" is absent
    When I save and restore
    Then the flag "sevenstar_vow" is absent
