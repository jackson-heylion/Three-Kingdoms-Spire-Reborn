Feature: Distinct boss telegraphs and coordinated Three Kingdoms warfare
  Real map-to-boss encounters test combat outcomes, turn-based counterplay,
  battle snapshots and allied enemy defense.

  Scenario: Zhang Jiao thunder punishes exposed heroes with weakness
    Given a "guanyu" campaign with seed 1401
    When I deploy boss "zhangjiao"
    Then the first foe telegraphs "雷咒"
    When I clear my guard
    And I end turn
    Then the hero is weakened
    And combat event "enemy_technique" from "zhangjiao" exists
    When I save and restore
    Then the hero is weakened

  Scenario: Sun Jian tiger rush is a distinctive multi-hit attack
    Given a "guanyu" campaign with seed 1402
    When I deploy boss "sunjian"
    Then the first foe telegraphs "虎袭"
    When I end turn
    Then combat event "enemy_technique" from "sunjian" exists
    And the event sequence strictly increases

  Scenario: Dong Zhuo tyranny seizes gold without driving it negative
    Given a "guanyu" campaign with seed 1403
    When I deploy boss "dongzhuo"
    Then the first foe telegraphs "暴政"
    When I end turn
    Then hero gold is at most 80
    And combat event "enemy_technique" from "dongzhuo" exists
    When I save and restore
    Then hero gold is at most 80

  Scenario: Yuan Shao protects his alliance with a formation
    Given a "guanyu" campaign with seed 1404
    When I deploy boss "yuanshao"
    Then the first foe telegraphs "盟军列阵"
    When I end turn
    Then the first foe has at least 9 guard
    And combat event "enemy_technique" from "yuanshao" exists

  Scenario: Sima Yi telegraphs delayed energy sabotage
    Given a "guanyu" campaign with seed 1405
    When I deploy boss "simayi"
    Then the first foe telegraphs "暗度陈仓"
    When I end turn
    Then battle energy equals 2
    And combat event "enemy_technique" from "simayi" exists
    When I save and restore
    Then battle energy equals 2

  Scenario: Zuo Ci creates an illusion that recovers actual health
    Given a "guanyu" campaign with seed 1406
    When I deploy boss "zuoci"
    Then the first foe telegraphs "幻身遁甲"
    When I wound the boss by 8 health
    And I remember the first enemy health
    And I end turn
    Then boss health has increased
    And combat event "enemy_technique" from "zuoci" exists

  Scenario: Sima Yi reacts to high-frequency card play in his next intent
    Given a "guanyu" campaign with seed 1407
    When I deploy boss "simayi"
    And I play four free cards
    And I end turn
    Then the first enemy telegraphs calculated sabotage
    And the event sequence strictly increases
    When I save and restore
    Then the first enemy telegraphs calculated sabotage

  Scenario: Ordinary officers protect another living combatant
    Given a "guanyu" campaign with seed 1408
    When I start a battle
    And I stage coordinated officers
    And I end turn
    Then the second foe has at least 4 guard
    And combat event "enemy_technique" from "officer" exists
