Feature: Novel provenance is auditable
  A narrative payoff must reference an actual chapter of the 120-chapter Mao edition.

  Scenario: All playable scenes map to real chapter numbers
    Given a "guanyu" campaign with seed 201
    Then every literary scene references a chapter from 1 to 120
    And all narrative prerequisites have earlier setup chapters
    And all character bonds refer to a playable literary scene

  Scenario: Novel and history are labeled separately in the project
    Given a "zhuge" campaign with seed 211
    Then the literary catalog is explicitly distinguished from historical evidence

  Scenario: Revealing an unseen payoff is not allowed without its planted clue
    Given a "guanyu" campaign with seed 221
    Then act 1 has no available lore scene
    And the flag "baima_debt" is absent

Scenario: The source catalog includes every chapter title and traceable link
    Given a "zhuge" campaign with seed 241
    Then novel index contains all 120 numbered chapters

Scenario: Lu Bu keeps Xiapi order records before the old White Gate choice
    Given a "lubu" campaign with seed 191
    And the flag "halberd_shot" was earned earlier
    And the flag "xiaopei_courier_guard" was earned earlier
    And the lore scene "xiapi_orders"
    When I choose 1
    Then the flag "xiapi_orders_account" is present
    And my deck contains "hold_line"
    And the bond with "侯成" is active
    And act 2 offers the lore scene "whitegate"
    When I save and restore
    Then the flag "xiapi_orders_account" is present
    And the lore scene "whitegate"
    When I choose 1
    Then the hidden conclusion "xiapi_orders_debt" is unlocked

  Scenario: Lu Bu hides the punishment and can still reach White Gate
    Given a "lubu" campaign with seed 192
    And the flag "halberd_shot" was earned earlier
    And the flag "xiaopei_courier_guard" was earned earlier
    And the lore scene "xiapi_orders"
    When I choose 2
    Then the flag "xiapi_orders_account" is absent
    And act 2 offers the lore scene "whitegate"
    And the bond with "侯成" is absent

  Scenario: Cao Cao records Xiapi flood before Guandu and later settles the debt
    Given a "cao" campaign with seed 193
    And the flag "shouchun_grain_account" was earned earlier
    And the flag "xu_capital" was earned earlier
    And the lore scene "anzhong"
    When I choose 1
    Then the flag "anzhong_field_report" is present
    And the lore scene "xiapi_flood"
    When I choose 1
    Then the flag "xiapi_flood_ledger" is present
    And my deck contains "granary_support"
    And the bond with "荀攸" is active
    And act 1 offers the lore scene "guandu"
    When I save and restore
    Then the flag "xiapi_flood_ledger" is present
    And the lore scene "guandu"
    When I choose 1
    Then the hidden conclusion "xiapi_water_debt" is unlocked

  Scenario: Cao Cao can reach Liaodong without the optional Xiapi ledger
    Given a "cao" campaign with seed 194
    And the flag "guandu_grain" was earned earlier
    Then act 2 offers the lore scene "liaodong"
    And the flag "xiapi_flood_ledger" is absent

  Scenario: Guan Yu records the released captives and guards the Xiapi household before Jingzhou
    Given a "guanyu" campaign with seed 221
    And the flag "anxi_seal" was earned earlier
    And the flag "wenjiu_oath" was earned earlier
    And the lore scene "wangzhong_prisoners"
    When I choose 1
    Then the flag "wangzhong_release_record" is present
    And my deck contains "hold_line"
    And the bond with "刘备" is active
    And the lore scene "xiapi_household"
    When I choose 1
    Then the flag "xiapi_household_ledger" is present
    When I save and restore
    Then the flag "xiapi_household_ledger" is present
    And act 2 offers the lore scene "jingzhou"
    And the lore scene "jingzhou"
    When I choose 1
    Then the hidden conclusion "xiapi_household_debt" is unlocked

  Scenario: Guan Yu refuses the prisoner ledger without locking the old Jingzhou choice
    Given a "guanyu" campaign with seed 222
    And the flag "anxi_seal" was earned earlier
    And the flag "wenjiu_oath" was earned earlier
    And the lore scene "wangzhong_prisoners"
    When I choose 2
    Then the flag "wangzhong_release_record" is absent
    And act 2 offers the lore scene "jingzhou"

  Scenario: Cao Cao keeps Jiping testimony and the palace casualty ledger until Guandu
    Given a "cao" campaign with seed 223
    And the flag "xu_capital" was earned earlier
    And the flag "xuhuang_honor" was earned earlier
    And the lore scene "jiping_testimony"
    When I choose 1
    Then the flag "jiping_case_record" is present
    And my deck contains "qimou"
    And act 1 offers the lore scene "palace_ledger"
    And the lore scene "palace_ledger"
    When I choose 1
    Then the flag "palace_purge_record" is present
    And the bond with "程昱" is active
    And act 1 offers the lore scene "guandu"
    When I save and restore
    Then the flag "palace_purge_record" is present
    And the lore scene "guandu"
    When I choose 1
    Then the hidden conclusion "jiping_palace_debt" is unlocked

  Scenario: Refusing Jiping testimony preserves the old Guandu route
    Given a "cao" campaign with seed 224
    And the flag "xu_capital" was earned earlier
    And the flag "xuhuang_honor" was earned earlier
    And the lore scene "jiping_testimony"
    When I choose 2
    Then the flag "jiping_case_record" is absent
    And act 1 offers the lore scene "guandu"

  Scenario: Guan Yu preserves three terms and returns the seal before the Huarong payoff
    Given a "guanyu" campaign with seed 2526
    And the flag "wenjiu_oath" was earned earlier
    And the flag "baima_debt" was earned earlier
    And the lore scene "baima_terms"
    When I choose 1
    Then the flag "three_terms_record" is present
    And the lore scene "seal_return"
    When I choose 1
    Then the flag "seal_return_record" is present
    And the lore scene "huarong"
    When I choose 1
    Then the hidden conclusion "three_terms_huarong" is unlocked


  Scenario: Guan Yu keeps the five passes and Gucheng witness ledger until Huarong
    Given a "guanyu" campaign with seed 2728
    And the flag "seal_return_record" was earned earlier
    And the flag "baima_debt" was earned earlier
    And the lore scene "five_passes"
    When I choose 1
    Then the flag "five_passes_record" is present
    And my deck contains "hold_line"
    And the bond with "普净" is active
    And the lore scene "gucheng_witness"
    When I choose 1
    Then the flag "gucheng_witness_ledger" is present
    When I save and restore
    Then the flag "gucheng_witness_ledger" is present
    And act 1 offers the lore scene "huarong"
    And the lore scene "huarong"
    When I choose 1
    Then the hidden conclusion "five_passes_huarong" is unlocked

  Scenario: Refusing the five passes ledger preserves the old Huarong route
    Given a "guanyu" campaign with seed 2729
    And the flag "seal_return_record" was earned earlier
    And the flag "baima_debt" was earned earlier
    And the lore scene "five_passes"
    When I choose 2
    Then the flag "five_passes_record" is absent
    And act 1 offers the lore scene "huarong"
    And the lore scene "huarong"
    When I choose 1
    Then the hidden conclusion "sword_debt" is unlocked
    And the flag "gucheng_witness_ledger" is absent

  Scenario: Sun family succession record pays after the old wedding and Yiling
    Given a "sun" campaign with seed 2929
    And the flag "taishi_trust" was earned earlier
    And the lore scene "sun_succession"
    When I choose 1
    Then the flag "sun_succession_account" is present
    And the bond with "孙权" is active
    When I save and restore
    Then the flag "sun_succession_account" is present
    And the lore scene "jiangdong_wedding"
    When I choose 1
    Then the flag "jiangdong_covenant" is present
    And act 2 offers the lore scene "yiling"
    And the lore scene "yiling"
    When I choose 1
    Then the hidden conclusion "jiangdong_succession_debt" is unlocked

  Scenario: Sun may refuse succession record without losing the legacy Yiling path
    Given a "sun" campaign with seed 2930
    And the flag "taishi_trust" was earned earlier
    And the lore scene "sun_succession"
    When I choose 2
    Then the flag "sun_succession_account" is absent
    And the lore scene "jiangdong_wedding"
    When I choose 1
    Then the flag "jiangdong_covenant" is present
    And act 2 offers the lore scene "yiling"
