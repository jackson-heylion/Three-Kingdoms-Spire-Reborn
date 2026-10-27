Feature: Chapters 057-059 evidence-backed choices and cross-act consequences
  Scenario: Zhuge records Leiyang responsibility without changing Fengchu's letter
    Given a "zhuge" campaign with seed 5757
    And the flag "longzhong_plan" was earned earlier
    And the flag "phoenix_letter" was earned earlier
    And the lore scene "leiyang_review"
    When I choose 1
    Then the flag "leiyang_case_record" is present
    And the hidden conclusion "phoenix_office_reckoning" is unlocked
    When I save and restore
    Then the flag "leiyang_case_record" is present

  Scenario: Cao keeps the Mateng, Tongguan and Weinan chain
    Given a "cao" campaign with seed 5859
    And the flag "guandu_grain" was earned earlier
    And the flag "tongque_dispatch_record" was earned earlier
    And the lore scene "mateng_case"
    When I choose 1
    Then the flag "mateng_case_record" is present
    And the lore scene "tongguan_retreat"
    When I choose 1
    Then the flag "tongguan_retreat_record" is present
    And the lore scene "weinan_division"
    When I choose 1
    Then the flag "weinan_division_record" is present
    And the hidden conclusion "xuchang_weinan_reckoning" is unlocked

  Scenario: Refusing Mateng responsibility cannot plant its flag
    Given a "cao" campaign with seed 5860
    And the flag "guandu_grain" was earned earlier
    And the flag "tongque_dispatch_record" was earned earlier
    And the lore scene "mateng_case"
    When I choose 2
    Then the flag "mateng_case_record" is absent

  Scenario: Refusing the Leiyang office record cannot plant its flag
    Given a "zhuge" campaign with seed 5758
    And the flag "longzhong_plan" was earned earlier
    And the flag "phoenix_letter" was earned earlier
    And the lore scene "leiyang_review"
    When I choose 2
    Then the flag "leiyang_case_record" is absent
