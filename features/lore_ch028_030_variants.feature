Feature: Chapters 028-030 fixed dual-edition witnesses and optional responsibility

  Scenario: Guan keeps the Gucheng military witness after five passes
    Given a "guan" campaign with seed 28030
    And the flag "five_passes_record" was earned earlier
    And the lore scene "gucheng_witness"
    When I choose 1
    Then the flag "gucheng_witness_ledger" is present
    When I save and restore
    Then the flag "gucheng_witness_ledger" is present

  Scenario: Sun refuses a succession account without rewriting the old marriage
    Given a "sun" campaign with seed 29030
    And the flag "taishi_trust" was earned earlier
    And the lore scene "sun_succession"
    When I choose 2
    Then the flag "sun_succession_account" is absent
    When I save and restore
    Then the flag "sun_succession_account" is absent

  Scenario: Cao records Guandu letters without changing the novel's victory
    Given a "cao" campaign with seed 30030
    And the flag "guandu_grain" was earned earlier
    And the lore scene "guandu_letters"
    When I choose 1
    Then the flag "guandu_captured_letters" is present
    When I save and restore
    Then the flag "guandu_captured_letters" is present

  Scenario: Refusing the Guandu responsibility archive never creates its flag after V2 restore
    Given a "cao" campaign with seed 280302
    And the flag "xu_capital" was earned earlier
    And the lore scene "guandu"
    When I choose 3
    Then the flag "guandu_grain" is absent
    When I save and restore
    Then the flag "guandu_grain" is absent
