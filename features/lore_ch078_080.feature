Feature: Chapter 078-080 mourning, posthumous Wei succession and Shu legitimacy

  Scenario: Cao archives medical case through Wei abdication without posthumous action
    Given a "cao" campaign with seed 7880
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
    Then the flag "abdication_seal_record" is present
    And the hidden conclusion "guandu_succession_abdication_debt" is unlocked
    When I save and restore
    Then the flag "abdication_seal_record" is present

  Scenario: Cao declining Huatuo case cannot earn the complete chain
    Given a "cao" campaign with seed 7881
    And the flag "luoyang_head_record" was earned earlier
    And the lore scene "huatuo_prison_review"
    When I choose 2
    Then the flag "huatuo_prison_record" is absent

  Scenario: Zhuge records mourning, Shangyong responsibility and the false death rumor
    Given a "zhuge" campaign with seed 7980
    And the flag "redcliff_wind" was earned earlier
    And the flag "chengdu_mourning_record" was earned earlier
    And the lore scene "jingzhou_mourning_counsel"
    When I choose 1
    Then the flag "jingzhou_mourning_record" is present
    And the lore scene "shangyong_refusal_trial"
    When I choose 1
    Then the flag "shangyong_trial_record" is present
    And the lore scene "chengdu_legitimacy_review"
    When I choose 1
    Then the flag "chengdu_legitimacy_record" is present
    And the hidden conclusion "redcliff_chengdu_legitimacy_debt" is unlocked

  Scenario: Zhuge rejecting the Shangyong record loses the full reward
    Given a "zhuge" campaign with seed 7981
    And the flag "jingzhou_mourning_record" was earned earlier
    And the lore scene "shangyong_refusal_trial"
    When I choose 2
    Then the flag "shangyong_trial_record" is absent
