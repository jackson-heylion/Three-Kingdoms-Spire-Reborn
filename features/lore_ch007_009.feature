Feature: Chapters 007-009 fixed variant ledgers and Chang'an accountability

  Scenario: Lu Bu accepts three responsibility ledgers and unlocks the old debt
    Given a "lu" campaign with seed 7009
    And the lore scene "redhare"
    When I choose 1
    Then the flag "red_hare" is present
    And the lore scene "pavilion"
    When I choose 1
    Then the flag "phoenix_pavilion" is present
    And the lore scene "changan"
    When I choose 1
    Then the flag "changan_account" is present
    And the hidden conclusion "changan_court_military_debt" is unlocked
    When I save and restore
    Then the hidden conclusion "changan_court_military_debt" is unlocked

  Scenario: Refusal of the Pavilion ledger cannot create its accepted flag
    Given a "lu" campaign with seed 7010
    And the lore scene "pavilion"
    When I choose 2
    Then the flag "phoenix_pavilion" is absent

  Scenario: Sun preserves the father's source ledger
    Given a "sun" campaign with seed 7011
    And the lore scene "sun_father_letter"
    When I choose 1
    Then the flag "sun_father_dispatch" is present
    When I save and restore
    Then the flag "sun_father_dispatch" is present

  Scenario: Sun rejects the source ledger without receiving its accepted flag
    Given a "sun" campaign with seed 7012
    And the lore scene "sun_father_letter"
    When I choose 2
    Then the flag "sun_father_dispatch" is absent

  Scenario: Sun declines the father's report and restoration cannot invent the family flag
    Given a "sun" campaign with seed 7013
    And the lore scene "sun_father_letter"
    When I choose 3
    Then the flag "sun_father_dispatch" is absent
    When I save and restore
    Then the flag "sun_father_dispatch" is absent
