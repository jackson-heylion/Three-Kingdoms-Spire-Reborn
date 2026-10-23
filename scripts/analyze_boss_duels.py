"""Audit and compare paired synthetic Boss duels; never infer natural build strength."""
import argparse
from collections import Counter, defaultdict
from hashlib import sha256
import json
from math import comb
from pathlib import Path

BOSSES = {"simayi", "zuoci"}
ASCENSIONS = {0, 10}
SIDES = ("baseline", "curated")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def natural(value):
    return type(value) is int and value >= 0


def exact_mcnemar(baseline_only, curated_only):
    """Two-sided exact conditional binomial test, null when no discordant pairs."""
    n = baseline_only + curated_only
    if not n:
        return None
    tail = sum(comb(n, k) for k in range(min(baseline_only, curated_only) + 1))
    return min(1.0, 2.0 * tail / (2 ** n))


def validate(report):
    require(type(report) is dict, "report must be an object")
    require(type(report.get("schema_version")) is int and report["schema_version"] == 1,
            "unsupported Boss duel schema_version")
    require(report.get("fixture") == "single_floor_one_boss_120hp" and
            report.get("policy") == "first_playable", "unsupported fixture/policy")
    cells = report.get("cells")
    require(type(cells) is list and len(cells) == 120 and
            type(report.get("pairs")) is int and report["pairs"] == 120,
            "expected exactly 120 paired Boss duel cells")
    keys = set()
    groups = Counter()
    plans = {}
    totals = Counter()
    for cell in cells:
        require(type(cell) is dict, "cell must be an object")
        plan, hero, boss, asc, seed = (cell.get(k) for k in
                                       ("plan", "hero", "boss", "ascension", "seed"))
        require(type(plan) is str and bool(plan) and type(hero) is str and bool(hero),
                "plan/hero must be nonempty strings")
        require(type(boss) is str and boss in BOSSES and type(asc) is int and asc in ASCENSIONS and natural(seed),
                "invalid boss/ascension/seed")
        key = (plan, boss, asc, seed)
        require(key not in keys, "duplicate Boss duel cell")
        keys.add(key)
        require(plan not in plans or plans[plan] == hero, "plan mapped to multiple heroes")
        plans[plan] = hero
        groups[(plan, boss, asc)] += 1
        for side in SIDES:
            d = cell.get(side)
            require(type(d) is dict, f"{side} must be an object")
            require(type(d.get("won")) is bool and type(d.get("phase_seen")) is bool,
                    f"{side} win/phase must be boolean")
            require(all(natural(d.get(k)) for k in ("hp_damage", "techniques", "actions"))
                    and d["actions"] > 0, f"{side} counters must be nonnegative integers")
            totals[f"{side}_wins"] += int(d["won"])
            totals["gross_hp_damage"] += d["hp_damage"]
            totals["phase_observations"] += int(d["phase_seen"])
            totals["technique_events"] += d["techniques"]
            totals["total_actions"] += d["actions"]
    require(len(plans) == 15 and len(set(plans.values())) == 5 and
            Counter(plans.values()) == Counter({h: 3 for h in set(plans.values())}),
            "expected three plans for each of five heroes")
    require(len(groups) == 60 and set(groups.values()) == {2},
            "expected two seeds per plan/Boss/ascension cell")
    for k, expected in totals.items():
        require(type(report.get(k)) is int and report[k] == expected,
                f"aggregate mismatch: {k}")
    return cells


def summarize_cells(cells):
    outcomes = Counter()
    metrics = Counter()
    for c in cells:
        a, b = c["baseline"], c["curated"]
        if a["won"] and b["won"]:
            outcomes["both_win"] += 1
        elif a["won"]:
            outcomes["baseline_only"] += 1
        elif b["won"]:
            outcomes["curated_only"] += 1
        else:
            outcomes["neither_win"] += 1
        for side, d in (("baseline", a), ("curated", b)):
            metrics[f"{side}_damage"] += d["hp_damage"]
            metrics[f"{side}_phases"] += int(d["phase_seen"])
            metrics[f"{side}_techniques"] += d["techniques"]
            metrics[f"{side}_actions"] += d["actions"]
    n = len(cells)
    baseline_wins = outcomes["both_win"] + outcomes["baseline_only"]
    curated_wins = outcomes["both_win"] + outcomes["curated_only"]
    return {
        "pairs": n,
        "baseline_wins": baseline_wins,
        "curated_wins": curated_wins,
        "win_delta": curated_wins - baseline_wins,
        "both_win": outcomes["both_win"],
        "baseline_only": outcomes["baseline_only"],
        "curated_only": outcomes["curated_only"],
        "neither_win": outcomes["neither_win"],
        "mcnemar_exact_p": exact_mcnemar(outcomes["baseline_only"], outcomes["curated_only"]),
        **dict(sorted(metrics.items())),
    }


def holm_adjust(rows):
    """Adjust exploratory plan-by-Boss p-values; absent discordance is not evidence."""
    ordered = sorted((row["mcnemar_exact_p"] if row["mcnemar_exact_p"] is not None
                      else 1.0, i) for i, row in enumerate(rows))
    adjusted = [None] * len(rows)
    floor = 0.0
    for rank, (p, i) in enumerate(ordered):
        floor = max(floor, min(1.0, (len(ordered) - rank) * p))
        if rows[i]["mcnemar_exact_p"] is not None:
            adjusted[i] = floor
    for row, p in zip(rows, adjusted):
        row["holm_adjusted_p"] = p


def analyze(report):
    cells = validate(report)
    by_plan_boss = defaultdict(list)
    by_plan = defaultdict(list)
    by_boss = defaultdict(list)
    by_ascension = defaultdict(list)
    for c in cells:
        by_plan_boss[(c["plan"], c["boss"])].append(c)
        by_plan[c["plan"]].append(c)
        by_boss[c["boss"]].append(c)
        by_ascension[c["ascension"]].append(c)
    pb = [dict(plan=plan, boss=boss, hero=items[0]["hero"],
               **summarize_cells(items))
          for (plan, boss), items in sorted(by_plan_boss.items())]
    holm_adjust(pb)
    normalized = json.dumps(report, ensure_ascii=False, sort_keys=True,
                            separators=(",", ":")).encode("utf-8")
    return {
        "schema_version": 1,
        "source_sha256_canonical_json": sha256(normalized).hexdigest(),
        "fixture": report["fixture"],
        "policy": report["policy"],
        "overall": summarize_cells(cells),
        "by_plan_boss": pb,
        "by_plan": [dict(plan=k, hero=v[0]["hero"], **summarize_cells(v))
                    for k, v in sorted(by_plan.items())],
        "by_boss": [dict(boss=k, **summarize_cells(v)) for k, v in sorted(by_boss.items())],
        "by_ascension": [dict(ascension=k, **summarize_cells(v))
                         for k, v in sorted(by_ascension.items())],
        "warnings": [
            "Synthetic first-floor Boss fixture; decks are injected, not naturally drafted.",
            "Only four paired seeds per plan/Boss; per-cell rates are exploratory.",
            "Gross HP damage includes defeats and is not normalized for fight length.",
            "Holm correction is exploratory; no causal Boss-counter or maturity claim.",
        ],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("output", nargs="?", type=Path)
    args = parser.parse_args()
    result = analyze(json.loads(args.report.read_text(encoding="utf-8")))
    payload = json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(payload, encoding="utf-8")
    else:
        print(payload, end="")
    overall = result["overall"]
    print(f"BOSS_DUEL_ANALYSIS pairs={overall['pairs']} "
          f"baseline_only={overall['baseline_only']} "
          f"curated_only={overall['curated_only']} "
          f"exact_p={overall['mcnemar_exact_p']}", file=__import__("sys").stderr)


if __name__ == "__main__":
    main()
