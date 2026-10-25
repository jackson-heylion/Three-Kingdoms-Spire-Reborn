#!/usr/bin/env python3
"""Real-binary CLI journey E2E: ten scenarios, persistent saves, process restarts.

Unlike Game::apply and synthetic state fixtures, every action goes through
the shipped tks --plain command parser and the durable save/chronicle files.
Runs without extra Python dependencies on macOS and Linux.
This covers a first-floor combat turn, NOT a complete 21-floor campaign.
"""
from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile


HEROES = ("guanyu", "zhuge", "sun", "cao", "lubu")
ASCENSIONS = (0, 10)


def cli(binary: pathlib.Path, directory: pathlib.Path, commands: str) -> str:
    result = subprocess.run(
        [str(binary), "--plain", "--save-dir", str(directory)],
        input=commands,
        text=True,
        capture_output=True,
        timeout=45,
        check=False,
    )
    if result.returncode != 0:
        raise AssertionError(
            f"CLI exited {result.returncode}; commands={commands!r}\n"
            f"stdout tail={result.stdout[-3000:]}\nstderr tail={result.stderr[-3000:]}"
        )
    return result.stdout


def read_json(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def verify_journey(binary: pathlib.Path, root: pathlib.Path, hero: str, ascension: int) -> None:
    directory = root / f"{hero}-{ascension}"
    seed = 777  # Same seed, separate save directory and real program for each hero/difficulty.
    cli(binary, directory, f"custom {hero} {seed} {ascension}\nchoose 1\nquit\n")
    saved = directory / "save.json"
    first = read_json(saved)

    assert first["version"] == 2 and first["stage"] == "combat", (hero, ascension, first["stage"])
    assert first["floor"] == 1 and first["revision"] == 1, (hero, ascension, first["revision"])
    assert first["seed"] == seed and first["ascension"] == ascension
    assert len(first["action_history"]) == 1, "first route should be one committed action"
    assert first["battle"]["hand"], "first combat must have playable hand UI"
    assert first["battle"]["event_seq"] > 0, "first combat must have audited events"

    # A rejected command must never advance revision, RNG, action history or saved gameplay.
    invalid = cli(binary, directory, "play 999\nverify\nverify-events\nquit\n")
    assert "错误：" in invalid, "invalid hand index must be reported to CLI user"
    assert "回放校验通过" in invalid, "reloaded run failed deterministic action replay"
    assert "错误：" not in invalid[invalid.index("回放校验通过"):], "event verification reported failure"
    assert read_json(saved) == first, "rejected action or read-only verifier mutated persisted Game"

    # Commit one actual enemy turn, verify both histories, terminate and restart.
    advanced = cli(binary, directory, "end\nverify\nverify-events\nquit\n")
    assert "错误：" not in advanced, f"valid turn/verification unexpectedly failed: {advanced[-2500:]}"
    second = read_json(saved)
    assert second["revision"] == first["revision"] + 1, "end turn did not commit exactly once"
    assert len(second["action_history"]) == 2, "route and turn must both be journaled"
    assert second["rng_state"] != 0
    assert second["battle"]["event_seq"] >= first["battle"]["event_seq"]
    assert second["battle"]["turn"] >= 2, "real enemy turn did not resolve"
    assert second["event_history"], "full combat event archive missing"

    readonly = cli(binary, directory, "status\nhistory\npowers\npiles\neventsource\nverify\nverify-events\nquit\n")
    assert "读取进度成功" in readonly, "fresh process did not load saved run"
    assert "回放校验通过" in readonly, "cross-process replay failed"
    assert "错误：" not in readonly, f"read-only CLI journey failed: {readonly[-2500:]}"
    assert read_json(saved) == second, "read-only commands or restart mutated persisted Game"
    profile = read_json(directory / "chronicle.json")
    assert profile["started"] == 1, "repeated loads counted the same campaign more than once"


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: python3 tests/cli_journey_e2e.py <path-to-tks>")
    binary = pathlib.Path(sys.argv[1]).resolve()
    if not binary.is_file():
        raise SystemExit(f"CLI binary does not exist: {binary}")

    with tempfile.TemporaryDirectory(prefix="tks-cli-e2e-") as temporary:
        root = pathlib.Path(temporary)
        for hero in HEROES:
            for ascension in ASCENSIONS:
                try:
                    verify_journey(binary, root, hero, ascension)
                except Exception as exc:
                    raise AssertionError(
                        f"CLI E2E failed for hero={hero} ascension={ascension}: {exc}"
                    ) from exc
                print(f"PASS hero={hero} ascension={ascension} start/reject/turn/restart/replay")
    print("PASS 10 real CLI journeys: 5 heroes x 2 ascensions x 4 processes")


if __name__ == "__main__":
    main()
