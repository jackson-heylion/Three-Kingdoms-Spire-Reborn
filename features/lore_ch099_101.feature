Feature: Chapters 099-101 two-county settlement, Jigu losses and Lucheng dispatches

  Scenario: Two counties and Lucheng wheat pay a once-only civilian debt
    Given a "zhuge" campaign with seed 99101
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "wudu_yinping_civil_register"
    When I choose 1
    Then the flag "wudu_yinping_civil_record" is present
    And the lore scene "lucheng_wheat_household"
    When I choose 1
    Then the flag "lucheng_wheat_rotation_record" is present
    And the hidden conclusion "wudu_lucheng_civil_debt" is unlocked
    When I save and restore
    Then the flag "lucheng_wheat_rotation_record" is present

  Scenario: Rejecting the two-county civilian account forfeits the wheat debt
    Given a "zhuge" campaign with seed 99102
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "wudu_yinping_civil_register"
    When I choose 2
    Then the flag "wudu_yinping_civil_record" is absent

  Scenario: Jigu wounded soldiers and Li Yan dispatch letters are linked
    Given a "zhuge" campaign with seed 99103
    And the flag "redcliff_wind" was earned earlier
    And the lore scene "jigu_defiance_casualties"
    When I choose 1
    Then the flag "jigu_casualty_record" is present
    And the lore scene "liyan_dispatch_evidence"
    When I choose 1
    Then the hidden conclusion "jigu_liyan_dispatch_debt" is unlocked

  Scenario: Wei court archives cannot resurrect Cao Cao or alter Cao Zhen
    Given a "cao" campaign with seed 99104
    And the flag "guandu_grain" was earned earlier
    And the lore scene "wei_caozhen_death_archive"
    When I choose 1
    Then the flag "wei_caozhen_archive_record" is present
    When I save and restore
    Then the flag "wei_caozhen_archive_record" is present
