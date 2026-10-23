"""Deterministic tests for paired Boss duel evidence and anti-overclaim guards."""
import importlib.util
import json
import os
from pathlib import Path
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "analyze_boss_duels.py"
SPEC = importlib.util.spec_from_file_location("analyze_boss_duels", SCRIPT)
analyzer = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(analyzer)


def fixture():
    cells = []
    for plan_index in range(15):
        for boss in ("simayi", "zuoci"):
            for asc in (0, 10):
                for trial in range(2):
                    side = lambda won: {"won": won, "hp_damage": 7,
                                        "phase_seen": True, "techniques": 2, "actions": 10}
                    cells.append({"plan": f"plan_{plan_index}",
                                  "hero": f"hero_{plan_index // 3}",
                                  "boss": boss, "ascension": asc,
                                  "seed": 10000 + plan_index * 100 + asc * 10 + trial,
                                  "baseline": side(trial == 0),
                                  "curated": side(True)})
    return {"schema_version": 1, "fixture": "single_floor_one_boss_120hp",
            "policy": "first_playable", "pairs": 120,
            "baseline_wins": 60, "curated_wins": 120,
            "gross_hp_damage": 240 * 7, "phase_observations": 240,
            "technique_events": 480, "total_actions": 2400, "cells": cells}


class BossDuelAnalyzerTests(unittest.TestCase):
    def test_full_fixture_paired_discordance_and_strata(self):
        result = analyzer.analyze(fixture())
        self.assertEqual(result["overall"]["curated_only"], 60)
        self.assertEqual(result["overall"]["baseline_only"], 0)
        self.assertEqual(result["overall"]["both_win"], 60)
        self.assertEqual(len(result["by_plan_boss"]), 30)
        self.assertEqual(len(result["by_plan"]), 15)
        self.assertEqual(len(result["by_boss"]), 2)
        self.assertEqual(len(result["by_ascension"]), 2)
        self.assertTrue(all(row["pairs"] == 4 for row in result["by_plan_boss"]))
        self.assertTrue(all(row["holm_adjusted_p"] == 1.0
                            for row in result["by_plan_boss"]))
        self.assertEqual(result, analyzer.analyze(fixture()))

    def test_exact_mcnemar_discordant_only(self):
        self.assertIsNone(analyzer.exact_mcnemar(0, 0))
        self.assertEqual(analyzer.exact_mcnemar(1, 0), 1.0)
        self.assertEqual(analyzer.exact_mcnemar(0, 5), 0.0625)
        self.assertEqual(analyzer.exact_mcnemar(2, 2), 1.0)
        self.assertEqual(analyzer.exact_mcnemar(0, 5), analyzer.exact_mcnemar(5, 0))

    def test_duplicate_and_inconsistent_aggregates_fail_closed(self):
        for field, value, pattern in (("baseline_wins", 59, "aggregate mismatch"),
                                      ("pairs", 119, "120 paired"),
                                      ("schema_version", True, "schema_version")):
            report = fixture()
            report[field] = value
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, pattern):
                analyzer.analyze(report)
        report = fixture()
        report["cells"][1]["seed"] = report["cells"][0]["seed"]
        with self.assertRaisesRegex(ValueError, "duplicate"):
            analyzer.analyze(report)

    def test_invalid_types_and_missing_group_fail_closed(self):
        for field, value in (("won", 1), ("actions", True), ("hp_damage", -1)):
            report = fixture()
            report["cells"][0]["baseline"][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                analyzer.analyze(report)
        report = fixture()
        report["cells"][0]["boss"] = "invalid"
        with self.assertRaisesRegex(ValueError, "invalid boss"):
            analyzer.analyze(report)
        report = fixture()
        report["cells"][0]["hero"] = "other"
        with self.assertRaisesRegex(ValueError, "multiple heroes"):
            analyzer.analyze(report)

    def test_holm_adjustment_monotone_and_missing(self):
        rows = [{"mcnemar_exact_p": p} for p in (0.02, None, 0.01, 0.04)]
        analyzer.holm_adjust(rows)
        self.assertEqual([r["holm_adjusted_p"] for r in rows], [0.06, None, 0.04, 0.08])

    def test_real_ci_report_if_present(self):
        temp = os.environ.get("RUNNER_TEMP")
        if not temp:
            self.skipTest("GitHub runner report not available locally")
        report = Path(temp) / "tks-boss-duels.json"
        self.assertTrue(report.is_file(), "CI must export Boss duel report before analyzer tests")
        result = analyzer.analyze(json.loads(report.read_text(encoding="utf-8")))
        summary = Path(temp) / "tks-boss-duel-analysis.json"
        summary.write_text(json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
                           encoding="utf-8")
        self.assertEqual(result["overall"]["pairs"], 120)
        print("Verified actual CI Boss duel analysis:",
              {k: result["overall"][k] for k in
               ("baseline_only", "curated_only", "mcnemar_exact_p")})


if __name__ == "__main__":
    unittest.main()
