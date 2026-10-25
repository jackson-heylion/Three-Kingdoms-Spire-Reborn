#!/usr/bin/env python3
"""Cross-platform real-process E2E for the nine-source authority projector."""
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
                assert len(report["journal_domains"]) == 9
                assert report["active_battle_reconstructed"] is True
                assert report["event_archive_reconciled"] is True
                assert report["event_boundaries_reconciled"] is True
                assert report["checked_revisions"] == 2
                assert report["archived_encounters"] == 1
                assert not report["whole_game_reconstructed"]
                assert not report["cryptographically_authenticated"]

                tampered = json.loads(json.dumps(payload))
                tampered["battle"]["fury"] += 1
                forged = directory / "forged-battle-runtime.json"
                forged.write_text(json.dumps(tampered), encoding="utf-8")
                rejected = invoke([authority, str(forged)])
                assert rejected.returncode != 0, "untracked combat fury passed audit"

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

                tampered = json.loads(json.dumps(payload))
                tampered["telemetry"]["totals"]["battles_started"] += 1
                forged = directory / "forged-telemetry.json"
                forged.write_text(json.dumps(tampered), encoding="utf-8")
                assert invoke([authority, str(forged)]).returncode != 0

                tampered = json.loads(json.dumps(payload))
                tampered["telemetry_history"].pop()
                forged = directory / "truncated-telemetry.json"
                forged.write_text(json.dumps(tampered), encoding="utf-8")
                assert invoke([authority, str(forged)]).returncode != 0

                # Forge the archived combat events and matching live window,
                # leaving all eight action-boundary journals untouched.
                tampered = json.loads(json.dumps(payload))
                archive = tampered["event_history"]
                assert archive, "new campaigns must archive actual combat events"
                first = next(
                    record for record in archive if record["event"]["kind"] != "card_played"
                )
                seq = first["event"]["seq"]
                first["event"]["kind"] = "card_played"
                for event in tampered["battle"]["events"]:
                    if event["seq"] == seq:
                        event["kind"] = "card_played"
                # The ninth domain also holds the active event window. Forge
                # it consistently to exercise the event-vs-telemetry check.
                change = tampered["battle_runtime_history"][-1]["change"]
                assert change["type"] in ("enter", "update"), change
                for event in change["state"]["events"]:
                    if event["seq"] == seq:
                        event["kind"] = "card_played"
                forged = directory / "forged-event-archive.json"
                forged.write_text(json.dumps(tampered), encoding="utf-8")
                rejected = invoke([authority, str(forged)])
                assert rejected.returncode != 0, (
                    "forged combat archive passed eight-domain reconciliation",
                    rejected.stderr,
                )
                assert "event outcomes disagree" in rejected.stderr, rejected.stderr

                # A forged revision cursor must fail even if the final event
                # archive, all eight journal states and Game snapshot agree.
                tampered = json.loads(json.dumps(payload))
                tampered["telemetry_history"][1]["archive_len"] = 0
                forged = directory / "forged-event-boundary.json"
                forged.write_text(json.dumps(tampered), encoding="utf-8")
                rejected = invoke([authority, str(forged)])
                assert rejected.returncode != 0, rejected.stdout
                assert "event revision" in rejected.stderr, rejected.stderr

                # Older V2 saves have no cursor evidence. They retain the
                # existing final-state audit, but cannot claim revision checks.
                tampered = json.loads(json.dumps(payload))
                for event in tampered["telemetry_history"]:
                    event.pop("archive_len", None)
                forged = directory / "legacy-no-boundaries.json"
                forged.write_text(json.dumps(tampered), encoding="utf-8")
                legacy = invoke([authority, str(forged)])
                assert legacy.returncode == 0, legacy.stderr
                legacy_report = json.loads(legacy.stdout)
                assert legacy_report["event_archive_reconciled"] is True
                assert legacy_report["event_boundaries_reconciled"] is False

                # Missing old V2 event archives are readable but not certified.
                tampered = json.loads(json.dumps(payload))
                tampered.pop("event_history")
                forged = directory / "missing-event-archive.json"
                forged.write_text(json.dumps(tampered), encoding="utf-8")
                rejected = invoke([authority, str(forged)])
                assert rejected.returncode != 0
                assert "complete combat event archive" in rejected.stderr

                assert json.loads(save.read_text(encoding="utf-8")) == payload
                print(f"PASS {hero} ascension={ascension}: nine streams/archive/restart/identity and telemetry tamper")
    # Real CLI process: an intermediate revision is forged and a later
    # legitimate action restores the final nine-domain runtime state.
    directory = root / "intermediate-runtime-window"
    journey = invoke(
        [tks, "--plain", "--save-dir", str(directory)],
        stdin="custom guanyu 777 0\\nchoose 1\\nend\\nquit\\n".replace("\\\\n", "\\n"),
    )
    assert journey.returncode == 0, journey.stderr[-1500:]
    save = directory / "save.json"
    assert invoke([authority, str(save)]).returncode == 0
    payload = json.loads(save.read_text(encoding="utf-8"))
    assert payload["revision"] >= 2, payload["revision"]
    tampered = json.loads(json.dumps(payload))
    first = tampered["battle_runtime_history"][1]["change"]
    assert first["type"] == "enter" and first["state"]["events"], first
    first["state"]["events"][0]["source"] += ":forged"
    forged = directory / "forged-intermediate-runtime-window.json"
    forged.write_text(json.dumps(tampered), encoding="utf-8")
    rejected = invoke([authority, str(forged)])
    assert rejected.returncode != 0, rejected.stdout
    assert "event revision 1: battle runtime window differs" in rejected.stderr, rejected.stderr
    assert json.loads(save.read_text(encoding="utf-8")) == payload

    print("PASS 10 cross-platform real-process composite authority scenarios")


if __name__ == "__main__":
    main()
