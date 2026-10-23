"""Regression tests for build evidence input validation (standard library only)."""
import contextlib
import copy
import importlib.util
import io
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "analyze_builds.py"
SPEC = importlib.util.spec_from_file_location("analyze_builds", SCRIPT)
analyzer = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(analyzer)


def cell(ascension=8):
    return {
        "hero": "guan", "id": "guan_burst", "ascension": ascension,
        "samples": 2, "baseline_wins": 1, "blueprint_wins": 2,
        "baseline_floor_sum": 15, "blueprint_floor_sum": 18,
        "counter_last_encounter_deaths": 0,
        "baseline_counter_exposed_runs": 1,
        "blueprint_counter_exposed_runs": 1,
        "baseline_counter_battles": {
            "encounters": 1, "survived": 1, "hero_hp_damage": 3,
            "turns_completed": 2,
        },
        "blueprint_counter_battles": {
            "encounters": 2, "survived": 2, "hero_hp_damage": 5,
            "turns_completed": 4,
        },
    }


class BuildEvidenceTests(unittest.TestCase):
    def report(self, entry):
        return {"schema_version": 1, "cells": [entry]}

    def test_valid_paired_metrics_and_legacy_report(self):
        grouped, seen = analyzer.summarize(self.report(cell()))
        self.assertEqual(len(seen), 1)
        self.assertEqual(grouped["guan"][0][1]["build_battles"], 2)
        legacy = cell()
        for key in ("baseline_counter_battles", "blueprint_counter_battles",
                    "baseline_counter_exposed_runs", "blueprint_counter_exposed_runs"):
            legacy.pop(key)
        grouped, _ = analyzer.summarize(self.report(legacy))
        self.assertEqual(grouped["guan"][0][1]["build_battles"], 0)

    def test_invalid_denominators_are_rejected(self):
        for key, value in (
            ("samples", True), ("baseline_wins", True),
            ("ascension", True), ("baseline_counter_exposed_runs", -1),
            ("blueprint_counter_exposed_runs", 3),
            ("counter_last_encounter_deaths", 3),
            ("baseline_floor_sum", -1),
        ):
            with self.subTest(key=key, value=value):
                entry = cell()
                entry[key] = value
                with self.assertRaises(ValueError):
                    analyzer.summarize(self.report(entry))

    def test_paired_battle_metrics_cannot_be_half_missing(self):
        entry = cell()
        entry.pop("blueprint_counter_battles")
        with self.assertRaisesRegex(ValueError, "incomplete paired"):
            analyzer.summarize(self.report(entry))

    def test_null_metrics_and_missing_exposure_are_rejected(self):
        for key, value in (("baseline_counter_battles", None),
                           ("baseline_counter_exposed_runs", None)):
            with self.subTest(key=key):
                entry = cell()
                entry[key] = value
                with self.assertRaises(ValueError):
                    analyzer.summarize(self.report(entry))
        entry = cell()
        entry.pop("baseline_counter_exposed_runs")
        with self.assertRaisesRegex(ValueError, "missing exposure"):
            analyzer.summarize(self.report(entry))

    def test_bad_battle_metrics_are_rejected(self):
        for key, value in (("encounters", True), ("survived", 3),
                           ("hero_hp_damage", -1), ("turns_completed", False)):
            with self.subTest(key=key):
                entry = cell()
                entry["baseline_counter_battles"][key] = value
                with self.assertRaises(ValueError):
                    analyzer.summarize(self.report(entry))

    def test_partial_low_ascension_report_does_not_crash_or_claim_coverage(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "partial.json"
            path.write_text(json.dumps(self.report(cell(ascension=0))), encoding="utf-8")
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                self.assertEqual(analyzer.main(["analyze_builds.py", str(path)]), 0)
            self.assertIn("Coverage warning", output.getvalue())
            self.assertIn("n/a (no asc. 8", output.getvalue())

    def test_schema_must_be_integer_one_and_payload_an_object(self):
        for payload in (None, [], True, {"schema_version": True, "cells": [cell()]},
                        {"schema_version": "1", "cells": [cell()]}):
            with self.subTest(payload=payload):
                with self.assertRaises(ValueError):
                    analyzer.summarize(payload)

    def test_cell_must_be_an_object_with_nonblank_string_identity(self):
        for invalid in (None, [], 12, "not an object"):
            with self.subTest(cell=invalid):
                with self.assertRaisesRegex(ValueError, "invalid fixture cell"):
                    analyzer.summarize({"schema_version": 1, "cells": [invalid]})
        for field, value in (("hero", []), ("hero", "  "),
                             ("id", None), ("id", 123), ("id", "")):
            with self.subTest(field=field, value=value):
                entry = cell()
                entry[field] = value
                with self.assertRaisesRegex(ValueError, "invalid hero or blueprint"):
                    analyzer.summarize(self.report(entry))

    def test_unpaired_exposure_metrics_are_rejected_even_in_legacy_reports(self):
        for missing in ("baseline_counter_exposed_runs",
                        "blueprint_counter_exposed_runs"):
            entry = cell()
            for field in ("baseline_counter_battles", "blueprint_counter_battles"):
                entry.pop(field)
            entry.pop(missing)
            with self.subTest(missing=missing):
                with self.assertRaisesRegex(ValueError, "incomplete paired exposure"):
                    analyzer.summarize(self.report(entry))

    def test_duplicate_cell_is_rejected(self):
        entry = cell()
        with self.assertRaisesRegex(ValueError, "duplicate fixture cell"):
            analyzer.summarize({"schema_version": 1,
                                "cells": [copy.deepcopy(entry), copy.deepcopy(entry)]})


if __name__ == "__main__":
    unittest.main()
