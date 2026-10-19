Feature: Local save writes never expose half-copied backup generations
  Process locks prevent cooperating writers; every JSON replacement must also
  survive a failed staging or backup operation without damaging live save bytes.

  Scenario: Two completed writes retain the previous durable generation
    Given an isolated save directory
    When I write two atomic save generations
    Then the primary is generation 2 and the backup is generation 1
    And no staged files remain in the save directory

  Scenario: Corrupted primary is restored without overwriting the verified backup
    Given an isolated save directory
    When I write two atomic save generations
    And I corrupt the primary and restore the verified backup
    Then the primary and backup both contain intact generation 1
    And no staged files remain in the save directory

  Scenario: A failed backup rename leaves the current primary untouched
    Given an isolated save directory
    When I write a first atomic save
    And I block backup replacement with a directory
    And a second atomic save is refused
    Then the previous primary is intact and no staged files remain
