Feature: Chapters 034-036 pinned witnesses and optional Guan Yu responsibility chain
  Scenario: Guan records Tanxi, Xinye and Fancheng before the old Jingzhou payoff
    Given a "guan" campaign with seed 34036
    And the flag "ancient_reunion" was earned earlier
    And the flag "anxi_seal" was earned earlier
    And the lore scene "tanxi_warning"
    When I choose 1
    Then the flag "tanxi_warning_record" is present
    And the lore scene "xinye_counsel"
    When I choose 1
    Then the flag "xinye_counsel_record" is present
    When I save and restore
    Then the flag "xinye_counsel_record" is present
    And the lore scene "fancheng_counsel"
    When I choose 1
    Then the flag "fancheng_counsel_record" is present
    And the lore scene "xushu_departure"
    When I choose 1
    Then the flag "xushu_departure_record" is present
    And the lore scene "jingzhou"
    When I choose 1
    Then the hidden conclusion "fancheng_xushu_jingzhou" is unlocked

  Scenario: Guan refuses Tanxi archive but old Jingzhou scene stays playable
    Given a "guan" campaign with seed 34037
    And the flag "ancient_reunion" was earned earlier
    And the flag "anxi_seal" was earned earlier
    And the lore scene "tanxi_warning"
    When I choose 2
    Then the flag "tanxi_warning_record" is absent
    And the lore scene "jingzhou"
    When I choose 1
    Then the flag "jingzhou_warning" is present

  Scenario: Guan refuses Xinye archive without inventing a second Lü Xiang death
    Given a "guan" campaign with seed 34038
    And the flag "tanxi_warning_record" was earned earlier
    And the lore scene "xinye_counsel"
    When I choose 2
    Then the flag "xinye_counsel_record" is absent
    And the lore scene "jingzhou"
    When I choose 1
    Then the flag "jingzhou_warning" is present

  Scenario: Guan refuses Fancheng evidence but old Jingzhou remains accessible
    Given a "guan" campaign with seed 34039
    And the flag "xinye_counsel_record" was earned earlier
    And the flag "anxi_seal" was earned earlier
    And the lore scene "fancheng_counsel"
    When I choose 2
    Then the flag "fancheng_counsel_record" is absent
    When I save and restore
    Then the flag "fancheng_counsel_record" is absent
    And the lore scene "jingzhou"
    When I choose 1
    Then the flag "jingzhou_warning" is present
