Feature: Choices in Romance of the Three Kingdoms have consequences
  Literary foreshadowing must survive save/restore, and NPC relationships change combat.

  Scenario: Guan Yu remembers the White Horse debt and later meets the Huarong crossroads
    Given a "guanyu" campaign with seed 101
    And the lore scene "baima"
    When I choose 1
    Then the flag "baima_debt" is present
    And the bond with "曹操" is active
    And act 1 offers the lore scene "huarong"
    When I save and restore
    Then the flag "baima_debt" is present
    And act 1 offers the lore scene "huarong"

  Scenario: A mercenary choice blocks Guan Yu's future repayment
    Given a "guanyu" campaign with seed 101
    And the lore scene "baima"
    When I choose 2
    Then the flag "baima_debt" is absent
    And the bond with "曹操" is absent
    And act 1 has no available lore scene

  Scenario: Three Brothers' vow produces actual combat protection
    Given a "guanyu" campaign with seed 111
    And the lore scene "taoyuan"
    When I choose 1
    Then the flag "taoyuan_oath" is present
    And my deck contains "oath_guard"
    When I start a battle
    Then battle guard is at least 10

  Scenario: Zhuge Liang's introduction sets up the Red Cliffs tactical payoff
    Given a "zhuge" campaign with seed 121
    And the lore scene "longzhong"
    When I choose 1
    Then the flag "longzhong_plan" is present
    And my deck contains "thought_spring"
    And the bond with "刘备" is active
    And act 1 offers the lore scene "borrow_arrows"
    When I start a battle
    Then battle scheme is at least 2

  Scenario: Sun Shangxiang inherits the Jiangdong marriage dilemma
    Given a "sun" campaign with seed 131
    And the lore scene "jiangdong_wedding"
    When I choose 1
    Then the flag "jiangdong_covenant" is present
    And my deck contains "arrows_on_discard"
    And the bond with "刘备" is active
    And the bond with "孙权" is active
    And act 1 offers the lore scene "river"
    When I start a battle
    Then battle combo is at least 2

  Scenario: Cao Cao's command rises from preserving the Han court
    Given a "cao" campaign with seed 141
    And the lore scene "xudu"
    When I choose 1
    Then the flag "xu_capital" is present
    And my deck contains "granary_support"
    And the bond with "荀彧" is active
    And act 1 offers the lore scene "guandu"
    When I start a battle
    Then battle command is at least 2

  Scenario: Lu Bu's first betrayal is remembered on the battlefield
    Given a "lubu" campaign with seed 151
    And the lore scene "redhare"
    When I choose 1
    Then the flag "red_hare" is present
    And my deck contains "blood_courage"
    And the bond with "丁原" is active
    And act 0 offers the lore scene "pavilion"
    When I start a battle
    Then battle fury is at least 3

  Scenario: Past history without its prerequisite must not unlock a future chapter
    Given a "zhuge" campaign with seed 161
    Then act 1 has no available lore scene

Scenario: Guan Yu's debt and mercy reveal a once-only hidden ending
    Given a "guanyu" campaign with seed 231
    And the flag "baima_debt" was earned earlier
    And the lore scene "huarong"
    When I choose 1
    Then the flag "huarong_mercy" is present
    And the hidden conclusion "sword_debt" is unlocked
    When I save and restore
    Then the hidden conclusion "sword_debt" is unlocked

  Scenario: An actual connected-map story lane opens the correct literary scene
    Given a "guanyu" campaign with seed 251
    When I travel to a story node
    Then the current literary chapter is "taoyuan"
    When I choose 1
    Then the flag "taoyuan_oath" is present
    When I save and restore
    Then the flag "taoyuan_oath" is present


  Scenario: Anxi anti-corruption choice is required before Jingzhou responsibility and Maicheng reckoning
    Given a "guanyu" campaign with seed 113
    And the lore scene "taoyuan"
    When I choose 1
    Then the flag "taoyuan_oath" is present
    And act 0 offers the lore scene "anxi"
    And act 2 does not offer the lore scene "jingzhou"
    And the lore scene "anxi"
    When I choose 1
    Then the flag "anxi_seal" is present
    And the bond with "张飞" is active
    And act 2 offers the lore scene "jingzhou"
    When I save and restore
    Then the flag "anxi_seal" is present
    And act 2 offers the lore scene "jingzhou"

  Scenario: Pocketing Anxi compensation forfeits the Jingzhou chain
    Given a "guanyu" campaign with seed 114
    And the lore scene "taoyuan"
    When I choose 1
    And the lore scene "anxi"
    When I choose 2
    Then the flag "anxi_seal" is absent
    And act 2 does not offer the lore scene "jingzhou"

  Scenario: Lu Bu keeps the Seven Armies military ledger and reconciles it at White Gate
    Given a "lubu" campaign with seed 171
    And the flag "halberd_shot" was earned earlier
    And the lore scene "seven_armies"
    When I choose 1
    Then the flag "seven_armies_account" is present
    And my deck contains "hold_line"
    And the bond with "陈登" is active
    And act 2 offers the lore scene "whitegate"
    When I save and restore
    Then the flag "seven_armies_account" is present
    And the lore scene "whitegate"
    When I choose 1
    Then the hidden conclusion "seven_armies_debt" is unlocked

  Scenario: Lu Bu's mercenary Seven Armies choice cannot claim the ledger
    Given a "lubu" campaign with seed 172
    And the flag "halberd_shot" was earned earlier
    And the lore scene "seven_armies"
    When I choose 2
    Then the flag "seven_armies_account" is absent
    And act 2 offers the lore scene "whitegate"
    And the bond with "陈登" is absent

  Scenario: Cao Cao assumes the Shouchun grain debt and carries it to Guandu
    Given a "cao" campaign with seed 173
    And the flag "puyang_supply" was earned earlier
    And the flag "xu_capital" was earned earlier
    And the lore scene "shouchun_grain"
    When I choose 1
    Then the flag "shouchun_grain_account" is present
    And my deck contains "granary_support"
    And the bond with "王垕" is active
    And act 1 offers the lore scene "anzhong"
    And the lore scene "anzhong"
    When I choose 2
    Then act 1 offers the lore scene "guandu"
    When I save and restore
    Then the flag "shouchun_grain_account" is present
    And the lore scene "guandu"
    When I choose 1
    Then the hidden conclusion "shouchun_grain_debt" is unlocked

  Scenario: Cao Cao's blame-shifting choice cannot block the old Guandu arc
    Given a "cao" campaign with seed 174
    And the flag "puyang_supply" was earned earlier
    And the flag "xu_capital" was earned earlier
    And the lore scene "shouchun_grain"
    When I choose 2
    Then the flag "shouchun_grain_account" is absent
    And act 1 offers the lore scene "guandu"
    And the bond with "王垕" is absent

  Scenario: Cao Cao keeps both Anzhong reports and learns from the later defeat
    Given a "cao" campaign with seed 181
    And the flag "shouchun_grain_account" was earned earlier
    And the flag "xu_capital" was earned earlier
    And the lore scene "anzhong"
    When I choose 1
    Then the flag "anzhong_field_report" is present
    And my deck contains "qimou"
    And the bond with "贾诩" is active
    And act 1 offers the lore scene "guandu"
    When I save and restore
    Then the flag "anzhong_field_report" is present
    And the lore scene "guandu"
    When I choose 1
    Then the hidden conclusion "anzhong_reckoning" is unlocked

  Scenario: Cao Cao's selective Anzhong report cannot block the old Guandu arc
    Given a "cao" campaign with seed 182
    And the flag "shouchun_grain_account" was earned earlier
    And the flag "xu_capital" was earned earlier
    And the lore scene "anzhong"
    When I choose 2
    Then the flag "anzhong_field_report" is absent
    And act 1 offers the lore scene "guandu"
    And the bond with "贾诩" is absent

  Scenario: Lu Bu spares the intercepted courier and faces the choice at White Gate
    Given a "lubu" campaign with seed 183
    And the flag "halberd_shot" was earned earlier
    And the flag "seven_armies_account" was earned earlier
    And the lore scene "xiaopei_courier"
    When I choose 1
    Then the flag "xiaopei_courier_guard" is present
    And my deck contains "hold_line"
    And the bond with "陈宫" is active
    And act 2 offers the lore scene "whitegate"
    When I save and restore
    Then the flag "xiaopei_courier_guard" is present
    And the lore scene "whitegate"
    When I choose 1
    Then the hidden conclusion "xiaopei_courier_reckoning" is unlocked

  Scenario: Lu Bu's execution of the courier does not lock the old White Gate route
    Given a "lubu" campaign with seed 184
    And the flag "halberd_shot" was earned earlier
    And the flag "seven_armies_account" was earned earlier
    And the lore scene "xiaopei_courier"
    When I choose 2
    Then the flag "xiaopei_courier_guard" is absent
    And act 2 offers the lore scene "whitegate"

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
