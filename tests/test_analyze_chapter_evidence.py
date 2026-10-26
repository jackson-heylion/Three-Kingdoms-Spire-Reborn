"""Evidence accounting regression tests."""
import csv
import importlib.util
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "chapter_audit", ROOT / "scripts/analyze_chapter_evidence.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ChapterEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "data").mkdir()
        (self.root / "docs").mkdir()
        self.rows = []
        for i in range(1, 121):
            ch = f"{i:03}"
            self.rows.append(dict(chapter=ch, short_title=ch,
                                  chapter_url=f"https://zh.wikisource.org/wiki/三國演義/第{ch}回",
                                  body_review_status=module.PENDING,
                                  body_evidence="尚无正文核读记录", game_beats="无"))
        for i, oldid in ((1, 111), (2, 222)):
            self.rows[i - 1]["body_review_status"] = module.READ
            self.rows[i - 1]["body_evidence"] = f"oldid={oldid}"
        self.rows[75]["body_review_status"] = module.PARTIAL
        (self.root / "docs/CHAPTERS_001_002_READING.md").write_text(
            "oldid=111\noldid=222\n", encoding="utf-8")
        (self.root / "docs/CHAPTER_003_READING.md").write_text(
            "oldid=333\n", encoding="utf-8")
        self.write_rows()

    def write_rows(self):
        with (self.root / "data/chapter_body_audit.tsv").open(
            "w", encoding="utf-8", newline=""
        ) as file:
            writer = csv.DictWriter(file, fieldnames=module.FIELDS, delimiter="\t")
            writer.writeheader()
            writer.writerows(self.rows)

    def test_unregistered_documents_are_not_promoted(self):
        report = module.audit(self.root)
        self.assertEqual((report["registered_single_edition_readings"],
                          report["partial_readings"], report["pending_review"]), (2, 1, 117))
        self.assertEqual(report["chapters_with_evidence_files"], 3)
        self.assertEqual(report["unregistered_evidence_chapters"], ["003"])
        self.assertFalse(report["collation_verified_by_this_tool"])
        self.assertFalse(report["gameplay_verified_by_this_tool"])

    def test_grouped_readings_require_each_chapter_pin(self):
        (self.root / "docs/CHAPTER_003_READING.md").unlink()
        (self.root / "docs/CHAPTERS_003_004_READING.md").write_text(
            "Chapter 003 oldid=333; chapter 004 oldid=444\\n", encoding="utf-8")
        for index, revision in ((2, 333), (3, 444)):
            self.rows[index]["body_review_status"] = module.READ
            self.rows[index]["body_evidence"] = f"oldid={revision}"
        self.write_rows()
        report = module.audit(self.root)
        self.assertEqual(report["registered_single_edition_readings"], 4)
        self.assertEqual(report["unregistered_evidence_count"], 0)
        self.rows[3]["body_evidence"] = "oldid=999"
        self.write_rows()
        with self.assertRaisesRegex(module.EvidenceError, "differs from evidence"):
            module.audit(self.root)

    def test_grouped_and_single_evidence_cannot_overlap(self):
        (self.root / "docs/CHAPTERS_002_003_READING.md").write_text(
            "oldid=222\\noldid=333\\n", encoding="utf-8")
        with self.assertRaisesRegex(module.EvidenceError, "duplicate evidence"):
            module.audit(self.root)

    def test_rejects_duplicate_chapter_row(self):
        self.rows[5]["chapter"] = "005"
        self.write_rows()
        with self.assertRaisesRegex(module.EvidenceError, "out of order"):
            module.audit(self.root)

    def test_rejects_unpinned_reading(self):
        self.rows[0]["body_evidence"] = "not pinned"
        self.write_rows()
        with self.assertRaisesRegex(module.EvidenceError, "lacks oldid"):
            module.audit(self.root)

    def test_rejects_mismatched_revision(self):
        self.rows[0]["body_evidence"] = "oldid=999"
        self.write_rows()
        with self.assertRaisesRegex(module.EvidenceError, "differs from evidence"):
            module.audit(self.root)

    def test_rejects_missing_registered_evidence(self):
        (self.root / "docs/CHAPTERS_001_002_READING.md").unlink()
        with self.assertRaisesRegex(module.EvidenceError, "without evidence"):
            module.audit(self.root)

    def test_rejects_false_complete_status(self):
        self.rows[2]["body_review_status"] = "已完成全回正文审校"
        self.write_rows()
        with self.assertRaisesRegex(module.EvidenceError, "overstated"):
            module.audit(self.root)

    def test_real_repository_ledger_is_structurally_consistent(self):
        if not (ROOT / "data/chapter_body_audit.tsv").exists():
            self.skipTest("repository checkout unavailable")
        result = module.audit(ROOT)
        self.assertEqual(result["ledger_rows"], 120)
        self.assertEqual(sum(result[key] for key in (
            "registered_single_edition_readings", "partial_readings", "pending_review")), 120)


if __name__ == "__main__":
    unittest.main()
