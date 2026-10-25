# P1 action revision overflow guard (2026-10-25)

Every successful `Game::apply` must advance its `u64` revision exactly once. The existing `apply_unchecked` uses `saturating_add(1)`; at `u64::MAX`, it could accept an action without advancing the revision when a legacy save has no authority journals. Such a commit breaks the per-action event journal contract and makes later reconstruction ambiguous.

`src/action_commit.rs` now rejects an exhausted revision before cloning or mutating the game. Rejection preserves the entire serialized snapshot, RNG and all present journals. Tests cover modern snapshots, parseable legacy snapshots without authority journals, and ordinary sequential commits. No save schema or game balance changes.

**Verification:** source/diff preflight only in this runtime; local Cargo is unavailable. Final-HEAD GitHub Actions on macOS 14 and Ubuntu, plus portable determinism, must pass before merge. This narrow integrity fix does not provide complete event sourcing, mature build balance, 120-chapter collation, internationalization or P1/1.0 acceptance.
