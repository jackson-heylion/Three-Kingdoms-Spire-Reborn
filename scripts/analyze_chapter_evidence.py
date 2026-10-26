#!/usr/bin/env python3
"""Audit chapter evidence metadata; do not infer literary collation or gameplay."""
import argparse
import csv
import json
import re
from collections import Counter
from pathlib import Path

FIELDS = ("chapter", "short_title", "chapter_url", "body_review_status", "body_evidence", "game_beats")
READ = "正文通读/未完成异本对校"
PARTIAL = "正文片段核查/未通读"
PENDING = "未完成全文审校"
OLDID = re.compile(r"oldid\s*=\s*(\d+)")
FILE = re.compile(r"CHAPTER_(\d{3})_READING\.md\Z")
GROUP = re.compile(r"CHAPTERS_(\d{3})_(\d{3})_READING\.md\Z")


class EvidenceError(ValueError):
    pass


def audit(root: Path) -> dict:
    root = Path(root)
    with (root / "data/chapter_body_audit.tsv").open(encoding="utf-8", newline="") as source:
        reader = csv.DictReader(source, delimiter="\t")
        if tuple(reader.fieldnames or ()) != FIELDS:
            raise EvidenceError("chapter ledger header mismatch")
        rows = list(reader)
    if len(rows) != 120:
        raise EvidenceError(f"expected 120 chapter rows, got {len(rows)}")
    counts = Counter()
    for i, row in enumerate(rows, 1):
        ch = f"{i:03}"
        if row["chapter"] != ch:
            raise EvidenceError(f"chapter {i}: duplicate, missing or out of order")
        if any(not row.get(key) or not row[key].strip() for key in FIELDS):
            raise EvidenceError(f"chapter {ch}: missing required field")
        if not row["chapter_url"].endswith(f"/第{ch}回"):
            raise EvidenceError(f"chapter {ch}: wrong source URL")
        status = row["body_review_status"]
        if status not in (READ, PARTIAL, PENDING):
            raise EvidenceError(f"chapter {ch}: unknown or overstated status {status}")
        if status == READ and not OLDID.search(row["body_evidence"]):
            raise EvidenceError(f"chapter {ch}: registered reading lacks oldid")
        counts[status] += 1

    evidence = {}
    for path in (root / "docs").glob("CHAPTER*_READING.md"):
        single = FILE.fullmatch(path.name)
        grouped = GROUP.fullmatch(path.name)
        if single:
            chapters = [int(single.group(1))]
        elif grouped:
            start, end = map(int, grouped.groups())
            if not 1 <= start <= end <= 120:
                raise EvidenceError(f"invalid grouped evidence {path.name}")
            chapters = range(start, end + 1)
        else:
            continue
        for chapter in chapters:
            ch = f"{chapter:03}"
            if not 1 <= chapter <= 120 or ch in evidence:
                raise EvidenceError(f"invalid or duplicate evidence {ch}")
            evidence[ch] = path

    for ch, path in evidence.items():
        revisions = set(OLDID.findall(path.read_text(encoding="utf-8")))
        if not revisions:
            raise EvidenceError(f"chapter {ch}: evidence missing fixed oldid")
        row = rows[int(ch) - 1]
        if row["body_review_status"] == READ:
            registered = OLDID.search(row["body_evidence"])
            if registered is None or registered.group(1) not in revisions:
                raise EvidenceError(f"chapter {ch}: registered oldid differs from evidence")
    missing = sorted(row["chapter"] for row in rows
                     if row["body_review_status"] == READ and row["chapter"] not in evidence)
    if missing:
        raise EvidenceError(f"registered reading without evidence: {', '.join(missing)}")
    unregistered = sorted(ch for ch in evidence if rows[int(ch) - 1]["body_review_status"] != READ)
    return {
        "ledger_rows": 120,
        "registered_single_edition_readings": counts[READ],
        "partial_readings": counts[PARTIAL],
        "pending_review": counts[PENDING],
        "chapters_with_evidence_files": len(evidence),
        "unregistered_evidence_chapters": unregistered,
        "unregistered_evidence_count": len(unregistered),
        "collation_verified_by_this_tool": False,
        "gameplay_verified_by_this_tool": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    result = audit(args.root)
    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        print(f"{result['registered_single_edition_readings']}/120 registered, "
              f"{result['chapters_with_evidence_files']} evidence files, "
              f"{result['unregistered_evidence_count']} pending reconciliation; "
              "no collation or gameplay verified")


if __name__ == "__main__":
    main()
