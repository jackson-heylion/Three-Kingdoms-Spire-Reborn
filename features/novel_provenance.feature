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
