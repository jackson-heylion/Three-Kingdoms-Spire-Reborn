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
    And act 2 has no available lore scene
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
    And act 2 has no available lore scene
