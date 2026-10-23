#!/usr/bin/env python3
"""Summarize the reproducible tks-sim --build-archetypes JSON fixture.

Standard library only. This is an injected starting-deck *fixture*, never a
natural-drafting or human win-rate estimator. Battle survival with a nominated
counter enemy is measured on mixed real encounters, NOT an isolated boss
counterfactual. It cannot establish enemy causality or human win rates.
"""
import json
import sys
from collections import defaultdict
from pathlib import Path


def summarize(payload):
    if not isinstance(payload, dict) or type(payload.get("schema_version")) is not int or payload["schema_version"] != 1:
        raise ValueError("unknown build-balance report schema")
    cells = payload.get("cells")
    if not isinstance(cells, list) or not cells:
        raise ValueError("empty or invalid build experiment")
    totals = defaultdict(lambda: dict(n=0, base=0, build=0, floor_base=0,
                                      floor_build=0, high_n=0, high_base=0,
                                      high_build=0, counter_deaths=0,
                                      base_battles=0, base_survived=0,
                                      build_battles=0, build_survived=0,
                                      base_damage=0, build_damage=0))
    seen = set()
    for cell in cells:
        if not isinstance(cell, dict):
            raise ValueError("invalid fixture cell object")
        hero, blueprint = cell.get("hero"), cell.get("id")
        if not all(isinstance(value, str) and value.strip()
                   for value in (hero, blueprint)):
            raise ValueError("invalid hero or blueprint identifier")
        key = (hero, blueprint)
        asc = cell["ascension"]
        if type(asc) is not int or asc < 0 or asc > 10:
            raise ValueError(f"invalid ascension: {asc}")
        if (*key, asc) in seen:
            raise ValueError(f"duplicate fixture cell: {key}/{asc}")
        seen.add((*key, asc))
        n = cell["samples"]
        b, c = cell["baseline_wins"], cell["blueprint_wins"]
        if type(n) is not int or n <= 0 or any(type(w) is not int for w in (b, c)) or not 0 <= b <= n or not 0 <= c <= n:
            raise ValueError(f"invalid sample or result: {key}/{asc}")
        deaths = cell["counter_last_encounter_deaths"]
        if type(deaths) is not int or not 0 <= deaths <= n:
            raise ValueError(f"invalid counter death count: {key}/{asc}")
        exposure_keys = ("baseline_counter_exposed_runs",
                         "blueprint_counter_exposed_runs")
        if any(field in cell for field in exposure_keys) and not all(
                field in cell for field in exposure_keys):
            raise ValueError(f"incomplete paired exposure metrics (missing exposure): {key}/{asc}")
        for side in ("baseline", "blueprint"):
            field = f"{side}_counter_exposed_runs"
            if field in cell:
                exposed = cell[field]
                if type(exposed) is not int or not 0 <= exposed <= n:
                    raise ValueError(f"invalid counter exposure count: {key}/{asc}/{side}")
        if any(type(cell[field]) is not int or cell[field] < 0
               for field in ("baseline_floor_sum", "blueprint_floor_sum")):
            raise ValueError(f"invalid floor totals: {key}/{asc}")
        metric_keys = ("baseline_counter_battles", "blueprint_counter_battles")
        if any(field in cell for field in metric_keys) and not all(
                isinstance(cell.get(field), dict) for field in metric_keys):
            raise ValueError(f"incomplete paired counter metrics: {key}/{asc}")
        if all(field in cell for field in metric_keys) and not all(
                f"{side}_counter_exposed_runs" in cell for side in ("baseline", "blueprint")):
            raise ValueError(f"counter metrics missing exposure denominator: {key}/{asc}")
        t = totals[key]
        t["n"] += n
        t["base"] += b
        t["build"] += c
        t["floor_base"] += cell["baseline_floor_sum"]
        t["floor_build"] += cell["blueprint_floor_sum"]
        t["counter_deaths"] += deaths
        for side, label in (("baseline", "base"), ("blueprint", "build")):
            metrics = cell.get(f"{side}_counter_battles")
            if metrics is None:
                continue  # Read pre-instrumentation reports without inventing data.
            if (not isinstance(metrics, dict)
                    or not all(type(metrics.get(field)) is int
                               and metrics[field] >= 0
                               for field in ("encounters", "survived",
                                             "hero_hp_damage", "turns_completed"))
                    or metrics["survived"] > metrics["encounters"]
                    or metrics["encounters"] < cell[f"{side}_counter_exposed_runs"]):
                raise ValueError(f"invalid encounter denominator: {key}/{asc}/{side}")
            t[f"{label}_battles"] += metrics["encounters"]
            t[f"{label}_survived"] += metrics["survived"]
            t[f"{label}_damage"] += metrics["hero_hp_damage"]
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
    if len(seen) != 15 * 11 or any(
            {asc for hero, bid, asc in seen if (hero, bid) == (h, b)} != set(range(11))
            for h, builds in by_hero.items() for b, _ in builds):
        print("Coverage warning: incomplete 15-build x 11-ascension matrix; "
              "do not treat this as a full maturity experiment.")
    print("Character | Blueprint | Baseline | Fixture | Mean floor delta | Asc. 8–10")
    print("--- | --- | ---: | ---: | ---: | ---")
    for hero in sorted(by_hero):
        for bid, t in sorted(by_hero[hero]):
            delta = (t["floor_build"] - t["floor_base"]) / t["n"]
            high = (f"{t['high_base']}/{t['high_n']} → "
                    f"{t['high_build']}/{t['high_n']}") if t["high_n"] else "n/a (no asc. 8–10)"
            print(f"{hero} | {bid} | {t['base']}/{t['n']} | "
                  f"{t['build']}/{t['n']} | {delta:+.2f} | {high}")
    if all("baseline_counter_battles" in cell and "blueprint_counter_battles" in cell
           for cell in payload["cells"]):
        print("\nCounter encounter (mixed-team) | Baseline survived/encounters | "
              "Fixture survived/encounters | Baseline HP damage/encounter | "
              "Fixture HP damage/encounter")
        print("--- | ---: | ---: | ---: | ---:")
        for hero in sorted(by_hero):
            for bid, t in sorted(by_hero[hero]):
                bd = (f"{t['base_damage'] / t['base_battles']:.1f}"
                      if t["base_battles"] else "n/a")
                fd = (f"{t['build_damage'] / t['build_battles']:.1f}"
                      if t["build_battles"] else "n/a")
                print(f"{hero} / {bid} | {t['base_survived']}/{t['base_battles']} | "
                      f"{t['build_survived']}/{t['build_battles']} | {bd} | {fd}")
    print("\nWarning: injected decks only; no natural acquisition, human play, "
          "confidence intervals or isolated boss counterfactuals; mixed-foe "
          "survival is not proof of a named enemy causing the outcome.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
