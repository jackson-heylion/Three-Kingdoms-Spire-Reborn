#!/usr/bin/env python3
"""Cross-process, real-binary external SHA-256 checkpoint acceptance.

Run on macOS and Linux; only Python standard library is needed.
A checkpoint stored alongside an attacker-writable save is NOT trusted.
"""
from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile


def execute(*args: str, input_text: str | None = None, success: bool = True) -> str:
    result = subprocess.run(
        args, input=input_text, text=True, capture_output=True, timeout=60, check=False
    )
    if success and result.returncode != 0:
        raise AssertionError(f"{args}: rc={result.returncode}: {result.stderr[-3000:]}")
    if not success and result.returncode == 0:
        raise AssertionError(f"{args}: unexpected success: {result.stdout[-3000:]}")
    return result.stdout if success else result.stderr


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: checkpoint_cli_e2e.py <tks> <tks-attest>")
    tks, attest = (str(pathlib.Path(arg).resolve()) for arg in sys.argv[1:])
    with tempfile.TemporaryDirectory(prefix="tks-checkpoint-e2e-") as temporary:
        root = pathlib.Path(temporary)
        save_dir = root / "mutable-game"
        external = root / "independent-trusted-storage"
        external.mkdir()
        save = save_dir / "save.json"

        execute(tks, "--plain", "--save-dir", str(save_dir),
                input_text="new guanyu 777\nchoose 1\nquit\n")
        initial = save.read_bytes()
        latest = json.loads(execute(attest, str(save)))
        first = json.loads(execute(attest, str(save), "0"))
        assert latest["revision"] == 1 and first["revision"] == 0
        assert latest["digest_sha256"] != first["digest_sha256"]
        anchor = external / "anchor.json"
        anchor.write_text(json.dumps(latest), encoding="utf-8")
        earlier = external / "revision-zero.json"
        earlier.write_text(json.dumps(first), encoding="utf-8")

        assert "verified revision=1" in execute(attest, str(save), "--verify", str(anchor))
        assert "verified revision=0" in execute(attest, str(save), "--verify", str(earlier))
        assert save.read_bytes() == initial, "read-only audit changed live save"

        edited = json.loads(initial)
        edited["gold"] += 1
        save.write_text(json.dumps(edited), encoding="utf-8")
        error = execute(attest, str(save), "--verify", str(anchor), success=False)
        assert "checkpoint export refused" in error, error
        save.write_bytes(initial)

        # A completely valid replacement campaign must not match an external
        # digest anchored before its actions/seed were rewritten.
        execute(tks, "--plain", "--save-dir", str(save_dir),
                input_text="new guanyu 778\nchoose 1\nquit\n")
        rejected = execute(attest, str(save), "--verify", str(anchor), success=False)
        assert "digest mismatch" in rejected or "different campaign" in rejected, rejected
        print("PASS real CLI checkpoint export/verify, historical anchor, tamper and rewritten campaign")


if __name__ == "__main__":
    main()
