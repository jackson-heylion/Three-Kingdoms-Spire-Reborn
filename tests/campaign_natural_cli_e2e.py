#!/usr/bin/env python3
"""Real-binary CLI E2E for expanded 40-floor campaigns, causality & recovery.

Every input goes strictly through `tks --plain --save-dir <dir>` player
commands (`campaign`, `choose`, `play`, `end`, `assault`, `use`, etc.). Zero
save editing or injected scene/flag manipulation is used for gameplay paths.
"""
from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
FIXTURE = HERE / "fixtures" / "campaign_guanyu_247510_warlords.txt"
ALLOWED_CMDS = {
    "campaign", "choose", "play", "end", "assault", "use",
    "upgrade", "remove", "discard", "enchant", "verify",
    "history-audit", "verify-events", "quit",
}


def run_cli(binary: pathlib.Path, directory: pathlib.Path, commands: list[str], *, expect_save: bool = True) -> tuple[str, dict | None]:
    payload = "\n".join(commands) + "\n"
    res = subprocess.run(
        [str(binary), "--plain", "--save-dir", str(directory)],
        input=payload,
        text=True,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=300,
        check=False,
    )
    assert res.returncode == 0, (res.returncode, res.stdout[-2500:])
    save_file = directory / "save.json"
    if not expect_save:
        return res.stdout, None
    saved = json.loads(save_file.read_text(encoding="utf-8"))
    return res.stdout, saved


def extract_history_audit_json(output: str) -> dict:
    marker = " 提示：{"
    idx = output.rfind(marker)
    assert idx >= 0, f"missing history-audit JSON output in: {output[-2000:]}"
    decoder = json.JSONDecoder()
    obj, _ = decoder.raw_decode(output[idx + len(" 提示："):])
    return obj


def verify_full_40_floor_cli(binary: pathlib.Path) -> list[str]:
    lines = [line.strip() for line in FIXTURE.read_text(encoding="utf-8").splitlines() if line.strip()]
    assert lines[0] == "campaign guanyu 247510 warlords"
    assert lines[-4:] == ["verify", "history-audit json", "verify-events", "quit"]
    assert all(line.split()[0] in ALLOWED_CMDS for line in lines), "unexpected non-player command"

    with tempfile.TemporaryDirectory(prefix="tks-exp-cli-40-") as td:
        d = pathlib.Path(td)
        out, saved = run_cli(binary, d, lines)
        assert saved is not None
        assert saved["hero"] == "guan_yu" and saved["difficulty"] == "warlords"
        assert saved["expanded_rules_revision"] == 1
        assert saved["floor"] == 40 and saved["stage"] == "won", (saved["floor"], saved["stage"])
        steps = len(saved["action_history"])
        assert steps == len(lines) - 5, (steps, len(lines))
        assert f"全战役操作回放校验通过，共{steps}步" in out, out[-2000:]
        assert "事件源重建核对通过" in out, out[-2000:]
        audit = extract_history_audit_json(out)
        assert audit["schema_version"] == 2 and audit["complete_campaign"] is True
        assert audit["covered_lower_bound"] >= 12, audit
        assert audit["battle_effect"] >= 6 and audit["cross_act_effect"] >= 2, audit
        assert "run_id" not in audit

        # Cross-process read-only restart must preserve exact JSON and identical audit report.
        before = (d / "save.json").read_text(encoding="utf-8")
        out2, _ = run_cli(binary, d, ["status", "history-audit json", "verify", "verify-events", "quit"])
        assert "读取进度成功" in out2 and "新制【群雄逐鹿】" in out2
        assert "全战役操作回放校验通过" in out2 and "错误：" not in out2
        assert extract_history_audit_json(out2) == audit
        assert (d / "save.json").read_text(encoding="utf-8") == before
        digest = hashlib.sha256(
            json.dumps(saved["action_history"], sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()
        ).hexdigest()
        print(
            f"PASS 40-floor expanded CLI win: seed=247510 warlords steps={steps} "
            f"density={audit['covered_lower_bound']} battle={audit['battle_effect']} "
            f"cross_act={audit['cross_act_effect']} sha256={digest}"
        )
    return lines[:-4]


def verify_counterfactual_story_branch(binary: pathlib.Path, campaign_cmds: list[str]) -> None:
    # Find the earliest Story node carrying an authored Beat in the genuine playthrough via linear prefix scan.
    story_prefix = None
    for idx in range(2, min(len(campaign_cmds), 120)):
        if campaign_cmds[idx].startswith("choose "):
            with tempfile.TemporaryDirectory(prefix="tks-cf-probe-") as td:
                _, s = run_cli(binary, pathlib.Path(td), campaign_cmds[:idx] + ["quit"])
                if s is not None and s["stage"] == "story" and s.get("lore_current"):
                    story_prefix = (campaign_cmds[:idx], s["lore_current"])
                    break
    assert story_prefix is not None, "could not find genuine Act 1 Story beat in CLI script"
    prefix, beat_id = story_prefix

    def branch_run(choice: str) -> dict:
        with tempfile.TemporaryDirectory(prefix=f"tks-cf-{choice}-") as td:
            out, s = run_cli(binary, pathlib.Path(td), prefix + [f"choose {choice}", "verify", "quit"])
            assert "全战役操作回放校验通过" in out and "错误：" not in out, out[-1500:]
            assert s is not None
            return s

    vow_save = branch_run("1")
    bargain_save = branch_run("2")
    assert vow_save["stage"] == "map" and bargain_save["stage"] == "map"
    assert len(vow_save["lore_flags"]) > len(bargain_save["lore_flags"]), (
        beat_id, vow_save["lore_flags"], bargain_save["lore_flags"]
    )
    assert vow_save["action_history"][:-1] == bargain_save["action_history"][:-1]
    print(f"PASS counterfactual expanded CLI branch on beat={beat_id}: vow vs bargain diverged deterministically")


def verify_corrupt_primary_recovery_and_tier_guard(binary: pathlib.Path) -> None:
    with tempfile.TemporaryDirectory(prefix="tks-exp-badtier-") as bad_td:
        bad_dir = pathlib.Path(bad_td)
        out_bad, _ = run_cli(binary, bad_dir, ["campaign cao 999 legend", "quit"], expect_save=False)
        assert "新制难度仅支持 warlords、conqueror、destiny" in out_bad
        assert not (bad_dir / "save.json").exists(), "invalid tier must never create save.json"

    with tempfile.TemporaryDirectory(prefix="tks-exp-recover-") as td:
        d = pathlib.Path(td)
        out, saved = run_cli(binary, d, ["campaign cao 999 conqueror", "choose 1", "end", "save", "quit"])
        assert "错误：" not in out, out[-1500:]
        assert saved is not None and saved["difficulty"] == "conqueror" and saved["expanded_rules_revision"] == 1
        assert saved["revision"] == 2
        good_json = (d / "save.json").read_text(encoding="utf-8")
        assert (d / "save.bak").is_file()

        # Corrupt primary save.json (simulated crash during overwrite) and restart process.
        (d / "save.json").write_text("{corrupted-expanded-primary", encoding="utf-8")
        out_rec, rec = run_cli(binary, d, ["status", "verify", "verify-events", "quit"])
        assert "全战役操作回放校验通过" in out_rec and "事件源重建核对通过" in out_rec
        assert "已从上一次有效" in out_rec and "错误：" not in out_rec, out_rec[-1500:]
        assert rec == json.loads(good_json), "backup restore did not preserve exact expanded state"
        assert (d / "save.corrupt").read_text(encoding="utf-8") == "{corrupted-expanded-primary"
        assert (d / "save.bak").is_file() and json.loads((d / "save.bak").read_text(encoding="utf-8")) == json.loads(good_json)
        assert not list(d.glob("*.tmp.*"))
        print("PASS expanded corrupt-primary recovery from save.bak + invalid tier atomic rejection")


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: python3 tests/campaign_natural_cli_e2e.py <path-to-tks>")
    binary = pathlib.Path(sys.argv[1]).resolve()
    if not binary.is_file():
        raise SystemExit(f"binary not found: {binary}")
    cmds = verify_full_40_floor_cli(binary)
    verify_counterfactual_story_branch(binary, cmds)
    verify_corrupt_primary_recovery_and_tier_guard(binary)
    print("PASS expanded natural 40-floor CLI, counterfactual divergence & backup recovery")


if __name__ == "__main__":
    main()
