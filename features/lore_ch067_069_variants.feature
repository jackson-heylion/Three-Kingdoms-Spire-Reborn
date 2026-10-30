Feature: Chapters 067-069 fixed dual-edition provenance and optional consequences
  Scenario: Cao records Hanzhong, Zuoci and Xudu without promoting divination to fact
    Given a "cao" campaign with seed 67069
    And the flag "zhangsong_envoy_record" was earned earlier
    And the lore scene "hanzhong_civil_ledger"
    When I choose 1
    Then the flag "hanzhong_civil_record" is present
    And the lore scene "zuoci_court_record"
    When I choose 1
    Then the flag "zuoci_court_record" is present
    And the hidden conclusion "hanzhong_zuoci_debt" is unlocked
    And the lore scene "xudu_fire_inquest"
    When I choose 1
    Then the flag "xudu_fire_record" is present
    When I save and restore
    Then the flag "xudu_fire_record" is present

  Scenario: Refusing Xudu evidence cannot fabricate a record
    Given a "cao" campaign with seed 67070
    And the flag "zuoci_court_record" was earned earlier
    And the lore scene "xudu_fire_inquest"
    When I choose 2
    Then the flag "xudu_fire_record" is absent
