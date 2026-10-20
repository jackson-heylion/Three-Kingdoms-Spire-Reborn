#!/usr/bin/env python3
"""Summarize the reproducible tks-sim --build-archetypes JSON fixture.

Standard library only. This is an injected starting-deck *fixture*, never a
natural-drafting or human win-rate estimator. Counter-death events lack an
encounter exposure denominator and cannot be interpreted as probabilities.
"""
import json
import sys
from collections import defaultdict
from pathlib import Path


def summarize(payload):
    if payload.get("schema_version") != 1:
        raise ValueError("unknown build-balance report schema")
    cells = payload.get("cells")
    if not isinstance(cells, list) or not cells:
        raise ValueError("empty or invalid build experiment")
    totals = defaultdict(lambda: dict(n=0, base=0, build=0, floor_base=0,
                                      floor_build=0, high_n=0, high_base=0,
                                      high_build=0, counter_deaths=0))
    seen = set()
    for cell in cells:
        key = (cell["hero"], cell["id"])
        asc = cell["ascension"]
        if not isinstance(asc, int) or asc < 0 or asc > 10:
            raise ValueError(f"invalid ascension: {asc}")
        if (*key, asc) in seen:
            raise ValueError(f"duplicate fixture cell: {key}/{asc}")
        seen.add((*key, asc))
        n = cell["samples"]
        b, c = cell["baseline_wins"], cell["blueprint_wins"]
        if not isinstance(n, int) or n <= 0 or not 0 <= b <= n or not 0 <= c <= n:
            raise ValueError(f"invalid sample or result: {key}/{asc}")
        t = totals[key]
        t["n"] += n
        t["base"] += b
        t["build"] += c
        t["floor_base"] += cell["baseline_floor_sum"]
        t["floor_build"] += cell["blueprint_floor_sum"]
        t["counter_deaths"] += cell["counter_last_encounter_deaths"]
        if asc >= 8:
            t["high_n"] += n
            t["high_base"] += b
            t["high_build"] += c
    by_hero = defaultdict(list)
    for key, t in totals.items():
        by_hero[key[0]].append((key[1], t))
    return by_hero, seen


def main(argv):
    if len(argv) != 2:
        print("Usage: python3 scripts/analyze_builds.py tks-builds.json", file=sys.stderr)
        return 2
    payload = json.loads(Path(argv[1]).read_text(encoding="utf-8"))
    by_hero, seen = summarize(payload)
    print(f"Builds: {sum(map(len, by_hero.values()))}; cells: {len(seen)}")
    print("Character | Blueprint | Baseline | Fixture | Mean floor delta | Asc. 8–10")
    print("--- | --- | ---: | ---: | ---: | ---")
    for hero in sorted(by_hero):
        for bid, t in sorted(by_hero[hero]):
            delta = (t["floor_build"] - t["floor_base"]) / t["n"]
            print(f"{hero} | {bid} | {t['base']}/{t['n']} | "
                  f"{t['build']}/{t['n']} | {delta:+.2f} | "
                  f"{t['high_base']}/{t['high_n']} → "
                  f"{t['high_build']}/{t['high_n']}")
    print("\nWarning: injected decks only; no natural acquisition, human play, "
          "confidence intervals or per-boss exposure denominators.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
