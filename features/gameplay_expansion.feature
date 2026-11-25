Feature: Gameplay expansion delivers three named tiers, 5x8 acts, special encounters, cross-act causality, echoes, and saturation bounds
  Expanded five-act campaigns must stay deterministic, replay-verified, and isolated from legacy 21-floor and epoch-0 saves.

  Scenario: Three named campaign tiers start 40-floor five-act runs and reject unknown tier names
    Then the expanded catalog exposes exactly 3 named tiers "warlords", "conqueror", "destiny"
    Given an expanded "guanyu" campaign on "conqueror" with seed 401
    And I remember the current snapshot
    Then the game uses expanded 40-floor rules on tier "conqueror"
    When I parse and fail an invalid tier "mythic"
    Then the action is rejected
    And the snapshot is unchanged

  Scenario: Five-act map places bosses strictly on floors 8, 16, 24, 32, and 40 without entering act 6
    Given an expanded "cao" campaign on "warlords" with seed 402
    Then all 5 expanded act maps place bosses on row 7 with 250-seed act 1 to 4 mid-act stories
    When I defeat the expanded boss on floor 40
    Then the campaign ends in victory on floor 40

  Scenario: Breakout turn deadline, siege assault, convoy protection, and boss phase two settle deterministically
    Given an expanded "cao" campaign on "warlords" with seed 403
    When I play expanded floor 1 breakout past the 4-turn limit
    Then the campaign ends in defeat diagnosing "突围超时"
    Given an expanded "guanyu" campaign on "warlords" with seed 404
    When I assault the siege gate on floor 4 until breach
    Then the siege gate falls cleanly into battle reward
    Given an expanded "sun" campaign on "warlords" with seed 405
    Then floor 2 convoy starts with 12/12 durability
    Given an expanded "zhuge" campaign on "conqueror" with seed 405
    When I trigger phase two on the act 5 boss "dengai"
    Then foe 1 is in phase 1

  Scenario: Cross-act council vows and counterfactual declines persist without duplicate settlement on save and restore
    Given an expanded "cao" campaign on "warlords" with seed 406
    When I open and commit wuchao council option 1 on act 2 story
    Then the action history commits once
    When I enter floor 17 battle under wuchao plan
    And I save and restore
    Then wuchao raid settles once on act 3 and does not duplicate on further action or reload
    When a counterfactual "cao" expanded campaign declines wuchao option 3 on act 2 story and enters floor 17
    Then wuchao decline settles without triggering raid

  Scenario: Shared and hero tactical echoes trigger via real Turn 1 card plays and stay suppressed on epoch zero
    Given an expanded "sun" campaign on "warlords" with seed 407
    When Sun completes the "south_wilds" vow choice cleanly
    And I play two real defend cards on Turn 1 against floor 32 boss "lumeng" and end turn
    Then combat event source "seven_captures_pacification" is emitted
    When the same setup runs on an epoch-zero expanded save
    Then combat event source "seven_captures_pacification" is suppressed

  Scenario: Expanded 40-floor economy enforces per-act recovery caps, progressive card removal prices, 2-potion boons, and stall exhaustion
    Given an expanded "guanyu" campaign on "warlords" with seed 408
    Then victory recovery caps step down across all 5 acts as 16, 14, 12, 10, 8
    And progressive market card removals rise from 75 to 175 via real Market actions without changing legacy 75 cost
    And camp drill max hp gains saturate at 0 after 8 real Camp actions
    And expanded herb garden grants 2 potions instead of 3
    When an expanded battle stalls past turn 40 with 100 block
    Then the campaign ends in defeat diagnosing "粮尽"

  Scenario: Legacy 21-floor V2 saves preserve old difficulty and recovery semantics alongside expanded campaigns
    Given a "guanyu" campaign with seed 409
    Then the game uses legacy 21-floor rules with no recovery cap and 75-gold removal
    And deterministic action replay verifies cleanly
