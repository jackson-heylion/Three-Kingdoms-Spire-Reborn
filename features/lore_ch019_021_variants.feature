Feature: Chapters 019-021 pinned edition distinctions and optional playable responsibility
  Scenario: Lu Bu's Xiapi refusal and White Gate debt is paid once
    Given a "lubu" campaign with seed 19019
    And the flag "xiaopei_courier_guard" was earned earlier
    And the flag "halberd_shot" was earned earlier
    And the lore scene "xiapi_orders"
    When I choose 1
    Then the flag "xiapi_orders_account" is present
    And the lore scene "whitegate"
    When I choose 1
    Then the flag "white_gate" is present
    And the hidden conclusion "xiapi_orders_debt" is unlocked
    When I save and restore
    Then the flag "white_gate" is present

  Scenario: Guan Yu's Xutian restraint and Xuzhou ledger preserve later accountability
    Given a "guanyu" campaign with seed 20021
    And the flag "wenjiu_oath" was earned earlier
    And the flag "jingzhou_warning" was earned earlier
    And the lore scene "xutian_guard"
    When I choose 1
    Then the flag "xutian_guarded_emperor" is present
    And the lore scene "xuzhou_gate"
    When I choose 1
    Then the flag "xuzhou_gate_account" is present
    And the hidden conclusion "xutian_xuzhou_debt" is unlocked

  Scenario: Refusing the Xuzhou ledger cannot forge its later flag
    Given a "guanyu" campaign with seed 21021
    And the flag "xutian_guarded_emperor" was earned earlier
    And the lore scene "xuzhou_gate"
    When I choose 2
    Then the flag "xuzhou_gate_account" is absent
