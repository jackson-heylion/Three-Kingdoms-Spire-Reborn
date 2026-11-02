Feature: Chapters 031-033 fixed JiaJing contrasts and optional Cao Cao accountability
  Scenario: Cao records Cangting and Ye before the old Liaodong plan and settlement
    Given a "cao" campaign with seed 31033
    And the flag "guandu_grain" was earned earlier
    And the flag "guandu_captured_letters" was earned earlier
    And the lore scene "cangting_memorial"
    When I choose 1
    Then the flag "cangting_memorial_record" is present
    And the lore scene "ye_flood_ledger"
    When I choose 1
    Then the flag "ye_flood_record" is present
    When I save and restore
    Then the flag "ye_flood_record" is present
    And the lore scene "liaodong"
    When I choose 1
    Then the flag "liaodong_letter" is present
    And the lore scene "northern_settlement"
    When I choose 1
    Then the flag "northern_settlement_account" is present

  Scenario: Cao refuses the Cangting archive but can still follow the old Liaodong plan
    Given a "cao" campaign with seed 31034
    And the flag "guandu_grain" was earned earlier
    And the flag "guandu_captured_letters" was earned earlier
    And the lore scene "cangting_memorial"
    When I choose 2
    Then the flag "cangting_memorial_record" is absent
    And the lore scene "liaodong"
    When I choose 1
    Then the flag "liaodong_letter" is present

  Scenario: Cao refuses the Ye flood archive without gaining its responsibility flag
    Given a "cao" campaign with seed 31035
    And the flag "cangting_memorial_record" was earned earlier
    And the lore scene "ye_flood_ledger"
    When I choose 2
    Then the flag "ye_flood_record" is absent
    When I save and restore
    Then the flag "ye_flood_record" is absent

  Scenario: Cao refuses the northern aftermath account without forged credit after restore
    Given a "cao" campaign with seed 31036
    And the flag "guandu_grain" was earned earlier
    And the flag "liaodong_letter" was earned earlier
    And the lore scene "northern_settlement"
    When I choose 2
    Then the flag "northern_settlement_account" is absent
    When I save and restore
    Then the flag "northern_settlement_account" is absent
