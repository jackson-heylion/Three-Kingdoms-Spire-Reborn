Feature: Chapters 072-074 Xiegu, Hanzhong kingship and Fancheng flood responsibility
  Scenario: Cao preserves Xiegu and Fancheng command debts
    Given a "cao" campaign with seed 7274
    And the flag "guandu_grain" was earned earlier
    And the flag "hanshui_retreat_record" was earned earlier
    And the lore scene "xiegu_ration_inquest"
    When I choose 1
    Then the flag "xiegu_ration_record" is present
    And the lore scene "pangde_yujin_ledger"
    When I choose 1
    Then the flag "pangde_yujin_record" is present
    And the hidden conclusion "xiegu_fancheng_command_debt" is unlocked
    When I save and restore
    Then the flag "pangde_yujin_record" is present

  Scenario: Cao refuses Xiegu record and cannot earn full chain
    Given a "cao" campaign with seed 7275
    And the flag "guandu_grain" was earned earlier
    And the flag "hanshui_retreat_record" was earned earlier
    And the lore scene "xiegu_ration_inquest"
    When I choose 2
    Then the flag "xiegu_ration_record" is absent

  Scenario: Zhuge keeps the route and the Hanzhong king memorial
    Given a "zhuge" campaign with seed 7374
    And the flag "redcliff_wind" was earned earlier
    And the flag "dingjun_signal_record" was earned earlier
    And the lore scene "hanzhong_route_review"
    When I choose 1
    Then the flag "hanzhong_route_record" is present
    And the lore scene "hanzhong_king_memorial"
    When I choose 1
    Then the flag "hanzhong_king_record" is present
    And the hidden conclusion "hanzhong_king_route_debt" is unlocked

  Scenario: Guan preserves the Jingxiang warning and flood civilians
    Given a "guan" campaign with seed 7475
    And the flag "anxi_seal" was earned earlier
    And the flag "lukou_terms_record" was earned earlier
    And the lore scene "jingxiang_border_warning"
    When I choose 1
    Then the flag "jingxiang_border_record" is present
    And the lore scene "fancheng_flood_witness"
    When I choose 1
    Then the flag "fancheng_flood_record" is present
    And the hidden conclusion "jingzhou_flood_warning_debt" is unlocked

  Scenario: Guan rejects border warning and receives no warning flag
    Given a "guan" campaign with seed 7476
    And the flag "lukou_terms_record" was earned earlier
    And the lore scene "jingxiang_border_warning"
    When I choose 2
    Then the flag "jingxiang_border_record" is absent
