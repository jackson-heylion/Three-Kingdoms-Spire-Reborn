Feature: 079-081 fixed-edition Wei succession and Shu eastern-campaign responsibility

  Scenario: Cao posthumous archives retain coerced abdication and one-time accountability
    Given a "cao" campaign with seed 79081
    And the flag "guandu_grain" was earned earlier
    And the flag "luoyang_head_record" was earned earlier
    And the lore scene "huatuo_prison_review"
    When I choose 1
    Then the flag "huatuo_prison_record" is present
    And the lore scene "ye_court_succession_review"
    When I choose 1
    Then the flag "ye_succession_record" is present
    And the lore scene "abdication_seal_audit"
    When I choose 1
    Then the hidden conclusion "guandu_succession_abdication_debt" is unlocked
    When I save and restore
    Then the flag "abdication_seal_record" is present

  Scenario: Zhuge refusal at Shangyong cannot be counted as a legitimate court ledger
    Given a "zhuge" campaign with seed 81081
    And the flag "jingzhou_mourning_record" was earned earlier
    And the lore scene "shangyong_refusal_trial"
    When I choose 2
    Then the flag "shangyong_trial_record" is absent
