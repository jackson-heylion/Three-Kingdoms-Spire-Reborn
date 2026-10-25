#!/usr/bin/env python3
"""Cross-platform real-process E2E for the seven-source authority projector."""
from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile


HEROES = ("guanyu", "zhuge", "sun", "cao", "lubu")


def invoke(command: list[str], *, stdin: str = "") -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command, input=stdin, capture_output=True, text=True,
        timeout=45, check=False,
    )


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: authority_cli_e2e.py <tks> <tks-authority>")
    tks, authority = (str(pathlib.Path(path).resolve()) for path in sys.argv[1:])
    with tempfile.TemporaryDirectory(prefix="tks-authority-e2e-") as tmp:
        root = pathlib.Path(tmp)
        for hero in HEROES:
            for ascension in (0, 10):
                directory = root / f"{hero}-{ascension}"
                game = invoke(
                    [tks, "--plain", "--save-dir", str(directory)],
                    stdin=f"custom {hero} 777 {ascension}\nchoose 1\nquit\n",
                )
                assert game.returncode == 0, (hero, ascension, game.stderr[-1500:])
                save = directory / "save.json"
                payload = json.loads(save.read_text(encoding="utf-8"))

                good = invoke([authority, str(save)])
                assert good.returncode == 0, (hero, ascension, good.stderr)
                report = json.loads(good.stdout)
                assert report["revision"] == payload["revision"] == 1
                assert report["active_combat"] and report["floor"] == 1
                assert report["stage"] == "combat"
                assert report["turn"] == 1 and report["foe_count"] >= 1
                assert len(report["journal_domains"]) == 7
                assert not report["whole_game_reconstructed"]
                assert not report["cryptographically_authenticated"]

                tampered = json.loads(json.dumps(payload))
                tampered["gold"] += 1
                forged = directory / "forged-snapshot.json"
                forged.write_text(json.dumps(tampered), encoding="utf-8")
                rejected = invoke([authority, str(forged)])
                assert rejected.returncode != 0, "forged Game snapshot passed authority check"

                tampered = json.loads(json.dumps(payload))
                changed = tampered["battle_resource_history"][-1]["change"]
                assert changed["type"] == "enter", changed
                changed["state"]["turn"] += 1
                forged = directory / "forged-journal.json"
                forged.write_text(json.dumps(tampered), encoding="utf-8")
                rejected = invoke([authority, str(forged)])
                assert rejected.returncode != 0, "cross-journal turn conflict passed"
                assert "cross-journal" in rejected.stderr, rejected.stderr

                tampered = json.loads(json.dumps(payload))
                tampered["campaign_history"][-1]["state"]["floor"] = 2
                forged = directory / "forged-campaign.json"
                forged.write_text(json.dumps(tampered), encoding="utf-8")
                rejected = invoke([authority, str(forged)])
                assert rejected.returncode != 0, "cross-journal campaign floor forgery passed"
                assert "cross-journal" in rejected.stderr, rejected.stderr

                for field, forged_value in (
                    ("seed", payload["seed"] + 1),
                    ("run_id", payload["run_id"] + "-forged"),
                ):
                    tampered = json.loads(json.dumps(payload))
                    tampered[field] = forged_value
                    forged = directory / f"forged-{field}.json"
                    forged.write_text(json.dumps(tampered), encoding="utf-8")
                    rejected = invoke([authority, str(forged)])
                    assert rejected.returncode != 0, (
                        field, "forged run identity passed authority audit"
                    )

                assert json.loads(save.read_text(encoding="utf-8")) == payload
                print(f"PASS {hero} ascension={ascension}: seven streams/restart/identity tamper")
    print("PASS 10 cross-platform real-process composite authority scenarios")


if __name__ == "__main__":
    main()
