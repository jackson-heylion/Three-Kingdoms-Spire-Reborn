Feature: Potions share the ordered combat trigger pipeline
  Potion consumption is audited and status reactions are not bypassed.

Scenario: Poison potion triggers Toxic Reversal once and survives save restore
  Given a "zhuge" campaign with seed 8181
  When I start a battle
  And I put card "toxic_reversal" in hand
  And I play the last card
  And I prepare potion "poison"
  And I use potion 1 against enemy 1
  Then the first enemy has at least 13 poison
  And combat event "potion_used" from "poison" occurs exactly once
  And combat event "status_applied" from "poison" occurs exactly once
  And combat event "power_triggered" from "toxic_reversal" occurs exactly once
  And the event sequence strictly increases
  When I save and restore
  Then the first enemy has at least 13 poison

Scenario: Weakness potion audits every affected foe once
  Given a "guanyu" campaign with seed 8182
  When I start a battle
  And I duplicate the first enemy as a second combatant
  And I prepare potion "weak"
  And I use potion 1 against enemy 1
  Then combat event "potion_used" from "weak" occurs exactly once
  And combat event "status_applied" from "weak" occurs exactly twice
  And the event sequence strictly increases
  When I save and restore
  Then combat event "status_applied" from "weak" occurs exactly twice

Scenario: Invalid targeted potion cannot consume inventory or append events
  Given a "guanyu" campaign with seed 8183
  When I start a battle
  And I prepare potion "poison"
  And I remember the current snapshot
  And I attempt to use potion 1 against enemy 999
  Then the action is rejected
  And the snapshot is unchanged
