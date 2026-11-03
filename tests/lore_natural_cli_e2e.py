"""Replay real, unmodified CLI campaigns to guard literary story reachability.

These scripts contain only player commands. No scene injection, flag editing,
fixture save file, or direct Game state construction is permitted. A successful
run proves the listed scenes can occur on the specified seed and build; it does
not prove that every chapter or a full three-act campaign is reachable.
"""
import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
CASES = (
    (37044, 239, 14, ("longzhong", "longzhong_letter", "imperial_jade",
                      "longzhong_conditions", "bowang_orders"),
     ("longzhong_plan", "longzhong_conditions_record", "bowang_orders_record")),
    (46001, 87, 11, ("longzhong", "borrow_arrows", "redcliff"),
     ("longzhong_plan", "mist_arrows", "redcliff_wind")),
    # PR #226: genuine floor-12 Fengchu -> Leiyang handoff, then a 21-floor win.
    # Earlier Longzhong conditions are declined, so this is one feasible branch,
    # not proof that every responsibility-preserving route is reachable.
    (61068, 225, 21, ("longzhong", "longzhong_letter",
                       "longzhong_conditions", "borrow_arrows", "redcliff",
                       "fengchu", "leiyang_review", "jiangwei"),
     ("longzhong_plan", "mist_arrows", "redcliff_wind",
      "phoenix_letter", "leiyang_case_record")),
)


def replay(binary: pathlib.Path, seed: int, expected_steps: int,
           expected_floor: int, expected_scenes: tuple[str, ...],
           expected_flags: tuple[str, ...]) -> str:
    commands = (HERE / "fixtures" / f"lore_seed{seed}.txt").read_text(encoding="utf-8")
    lines = commands.splitlines()
    assert lines[0] == f"new zhuge {seed} 0"
    assert lines[-3:] == ["verify", "lore", "quit"]
    assert all(line.split()[0] in {"new", "choose", "play", "use", "end", "verify", "lore", "quit"}
               for line in lines), "only genuine player commands are allowed"
    with tempfile.TemporaryDirectory(prefix=f"tks-lore-{seed}-") as directory:
        result = subprocess.run(
            [str(binary), "--plain", "--save-dir", directory], input=commands,
            text=True, encoding="utf-8", stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, timeout=75, check=False,
        )
        assert result.returncode == 0, (seed, result.returncode, result.stdout[-2000:])
        marker = f"全战役操作回放校验通过，共{expected_steps}步"
        assert marker in result.stdout, (seed, "CLI verify failed", result.stdout[-2000:])
        saved = json.loads((pathlib.Path(directory) / "save.json").read_text(encoding="utf-8"))
        assert saved["seed"] == seed and saved["hero"] == "zhuge_liang"
        expected_stage = "won" if seed == 61068 else "lost"
        assert saved["floor"] == expected_floor and saved["stage"] == expected_stage
        assert len(saved["action_history"]) == expected_steps
        assert tuple(saved["lore_seen"]) == expected_scenes
        assert all(flag in saved["lore_flags"] for flag in expected_flags)
        if seed == 61068:
            assert saved["hp"] == 32
            assert "longzhong_conditions_record" not in saved["lore_flags"]
            # Confirm both independent choices occurred at floor 12. Merely
            # seeing two scene IDs in the final save would be insufficient.
            visits = [(entry["state"]["floor"], entry["state"]["lore_current"])
                      for entry in saved["campaign_history"]
                      if isinstance(entry.get("state"), dict)
                      and entry["state"].get("lore_current") in
                      ("fengchu", "leiyang_review")]
            assert visits == [(12, "fengchu"), (12, "leiyang_review")], visits
        # No manually injected scene/flag: the only inputs are the CLI script above.
        serialized = json.dumps(saved["action_history"], sort_keys=True,
                                ensure_ascii=False, separators=(",", ":"))
        digest = hashlib.sha256(serialized.encode("utf-8")).hexdigest()
        if seed == 61068:
            assert digest == "40c263596dfc69a9cca5cae0dfc6c528a79172d77ab205ae188e2c01d24fd41d"
        return digest


def replay_yizhou_alternative(binary: pathlib.Path) -> None:
    """Find the real floor-15 Story in the existing winning seed, then opt in.

    Every input is a player command. The prefix is discovered by replaying
    shorter command sequences, never by editing save.json or injecting a scene.
    """
    original = (HERE / "fixtures" / "lore_seed61068.txt").read_text(encoding="utf-8").splitlines()
    assert original[-3:] == ["verify", "lore", "quit"]
    campaign = original[:-3]

    def run(commands: list[str]) -> tuple[dict, str]:
        with tempfile.TemporaryDirectory(prefix="tks-yizhou-natural-") as directory:
            result = subprocess.run(
                [str(binary), "--plain", "--save-dir", directory],
                input="\n".join(commands) + "\n", text=True, encoding="utf-8",
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                timeout=75, check=False,
            )
            assert result.returncode == 0, (result.returncode, result.stdout[-2000:])
            saved = json.loads((pathlib.Path(directory) / "save.json").read_text(encoding="utf-8"))
            return saved, result.stdout

    low, high = 1, len(campaign)
    while low < high:
        middle = (low + high) // 2
        saved, _ = run(campaign[:middle] + ["quit"])
        if saved["floor"] >= 15:
            high = middle
        else:
            low = middle + 1
    prefix = campaign[:low]
    reached, _ = run(prefix + ["quit"])
    assert (reached["floor"], reached["stage"], reached["lore_current"]) == (
        15, "story", "jiangwei"
    ), (low, reached["floor"], reached["stage"], reached["lore_current"])
    assert "leiyang_case_record" in reached["lore_flags"]
    assert "yizhou_map_audit" not in reached["lore_seen"]

    script = prefix + ["choose 4", "choose 1", "verify", "lore", "quit"]
    histories = []
    for _ in range(2):
        saved, output = run(script)
        assert "全战役操作回放校验通过" in output, output[-2000:]
        assert (saved["floor"], saved["stage"]) == (15, "map")
        assert saved["lore_seen"][-1] == "yizhou_map_audit"
        assert "yizhou_map_record" in saved["lore_flags"]
        assert "jiangwei" not in saved["lore_seen"]
        assert "successor_scroll" not in saved["lore_flags"]
        histories.append(hashlib.sha256(json.dumps(
            saved["action_history"], sort_keys=True, ensure_ascii=False,
            separators=(",", ":")
        ).encode("utf-8")).hexdigest())
    assert histories[0] == histories[1], histories
    print(f"PASS seed=61068 hero=zhuge difficulty=0 floor=15 "
          f"scenes=yizhou_map_audit optional_choice=4+1 "
          f"history_sha256={histories[0]} (2 fresh natural campaigns)")

    # The same genuine third-act Story entry can carry three more independent
    # chapter decisions. The four successive 'choose 4' commands explicitly
    # retain the current record and open the next; the final 'choose 1' exits.
    # No map mutation, scene injection, flag editing, or synthetic Game state.
    script = prefix + ["choose 4", "choose 4", "choose 4", "choose 4",
                       "choose 1", "verify", "lore", "quit"]
    chain = ("yizhou_map_audit", "fu_pass_review",
             "luofeng_mourning", "jinyan_bridge_review")
    flags = ("yizhou_map_record", "fu_pass_record",
             "luofeng_mourning_record", "jinyan_bridge_record")
    chain_hashes = []
    for _ in range(2):
        saved, output = run(script)
        assert "全战役操作回放校验通过" in output, output[-2000:]
        assert (saved["floor"], saved["stage"]) == (15, "map")
        assert tuple(saved["lore_seen"][-4:]) == chain, saved["lore_seen"]
        assert all(flag in saved["lore_flags"] for flag in flags)
        assert "chengdu_land_record" not in saved["lore_flags"]
        assert "jiangwei" not in saved["lore_seen"]
        visits = [(entry["state"]["floor"], entry["state"]["lore_current"])
                  for entry in saved["campaign_history"]
                  if isinstance(entry.get("state"), dict)
                  and entry["state"].get("lore_current") in chain]
        assert visits == [(15, scene) for scene in chain], visits
        chain_hashes.append(hashlib.sha256(json.dumps(
            saved["action_history"], sort_keys=True, ensure_ascii=False,
            separators=(",", ":")
        ).encode("utf-8")).hexdigest())
    assert chain_hashes[0] == chain_hashes[1], chain_hashes
    print(f"PASS seed=61068 hero=zhuge difficulty=0 floor=15 "
          f"scenes={','.join(chain)} independent_choices=4+4+4+4+1 "
          f"history_sha256={chain_hashes[0]} (2 fresh natural campaigns)")


    # Two separate CLI processes, sharing the same untouched save directory.
    with tempfile.TemporaryDirectory(prefix="tks-yizhou-restart-") as directory:
        commands = prefix + ["choose 4", "choose 4", "choose 4",
                             "choose 4", "choose 1", "save", "quit"]
        first = subprocess.run(
            [str(binary), "--plain", "--save-dir", directory],
            input="\n".join(commands) + "\n", text=True, encoding="utf-8",
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            timeout=75, check=False,
        )
        assert first.returncode == 0, first.stdout[-2000:]
        save_path = pathlib.Path(directory) / "save.json"
        before = json.loads(save_path.read_text(encoding="utf-8"))
        assert (before["floor"], before["stage"]) == (15, "map")
        assert tuple(before["lore_seen"][-4:]) == chain
        assert all(flag in before["lore_flags"] for flag in flags)
        second = subprocess.run(
            [str(binary), "--plain", "--save-dir", directory],
            input="load\nverify\nlore\nquit\n", text=True, encoding="utf-8",
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            timeout=75, check=False,
        )
        assert second.returncode == 0, second.stdout[-2000:]
        assert "读取进度成功" in second.stdout, second.stdout[-2000:]
        assert "全战役操作回放校验通过" in second.stdout, second.stdout[-2000:]
        after = json.loads(save_path.read_text(encoding="utf-8"))
        for field in ("seed", "hero", "floor", "stage", "lore_seen",
                      "lore_flags", "action_history", "campaign_history"):
            assert after[field] == before[field], field
        assert "chengdu_land_record" not in after["lore_flags"]
        print("PASS seed=61068 hero=zhuge difficulty=0 floor=15 "
              "Yizhou responsibility chain survives CLI restart+load+verify; "
              "Chengdu successor remains unverified")


def main() -> None:
    binary = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/tks").resolve()
    for seed, steps, floor, scenes, flags in CASES:
        first = replay(binary, seed, steps, floor, scenes, flags)
        second = replay(binary, seed, steps, floor, scenes, flags)
        assert first == second, f"seed {seed}: replay histories diverged"
        print(f"PASS seed={seed} hero=zhuge difficulty=0 floor={floor} "
              f"actions={steps} scenes={','.join(scenes)} history_sha256={first}")
    replay_yizhou_alternative(binary)
    print("Natural CLI story reachability: 3 seeds, 10 fresh campaigns plus prefix discovery, deterministic.")


if __name__ == "__main__":
    main()
