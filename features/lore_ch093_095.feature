Feature: Chapters 093-095 recruitment, Xincheng intelligence and Jieting retreat accountability

  Scenario: Tianshui warning and real surrender complete one handover
    Given a "zhuge" campaign with seed 9395
    And the flag "tianshui_false_dispatch_record" was earned earlier
    And the lore scene "tianshui_recruitment_receipt"
    When I choose 1
    Then the flag "tianshui_recruit_record" is present
    And the hidden conclusion "tianshui_truth_debt" is unlocked
    When I save and restore
    Then the flag "tianshui_recruit_record" is present

  Scenario: Refusal never grants the Tianshui responsibility reward
    Given a "zhuge" campaign with seed 9396
    And the flag "tianshui_false_dispatch_record" was earned earlier
    And the lore scene "tianshui_recruitment_receipt"
    When I choose 2
    Then the flag "tianshui_recruit_record" is absent

  Scenario: Redcliff and Jieting water report pay only for the accepted record
    Given a "zhuge" campaign with seed 9397
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "jieting_water_order"
    When I choose 1
    Then the flag "jieting_water_record" is present
    And the hidden conclusion "redcliff_jieting_supply_debt" is unlocked
    And the lore scene "xicheng_civilian_retreat"
    When I choose 1
    Then the flag "xicheng_civilian_record" is present

  Scenario: Wei court history separates Wang Lang and Meng Da from Cao Cao's life
    Given a "cao" campaign with seed 9398
    And the lore scene "wanglang_death_archive"
    When I choose 1
    Then the flag "wanglang_archive_record" is present
    And the lore scene "xincheng_secret_dispatch"
    When I choose 2
    Then the flag "xincheng_secret_record" is absent
