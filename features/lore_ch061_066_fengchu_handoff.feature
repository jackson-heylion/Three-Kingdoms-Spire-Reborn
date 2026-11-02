Feature: Chapter 057 Fengchu and Leiyang responsibility handoff for later Yizhou arc
  # Injected Story setup proves transactional behavior, not natural 21-floor reachability.
  Scenario: Fengchu letter offers a separate Leiyang decision before leaving the story node
    Given a "zhuge" campaign with seed 61066
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "fengchu"
    When I choose 1
    Then the flag "phoenix_letter" is present
    And the current literary chapter is "leiyang_review"
    When I save and restore
    Then the current literary chapter is "leiyang_review"
    When I choose 1
    Then the flag "leiyang_case_record" is present

  Scenario: Refusing Fengchu responsibility does not create a Leiyang record
    Given a "zhuge" campaign with seed 61067
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "fengchu"
    When I choose 2
    Then the flag "phoenix_letter" is absent
    And the flag "leiyang_case_record" is absent
