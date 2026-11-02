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
        assert saved["floor"] == expected_floor and saved["stage"] == "lost"
        assert len(saved["action_history"]) == expected_steps
        assert tuple(saved["lore_seen"]) == expected_scenes
        assert all(flag in saved["lore_flags"] for flag in expected_flags)
        # No manually injected scene/flag: the only inputs are the CLI script above.
        serialized = json.dumps(saved["action_history"], sort_keys=True,
                                ensure_ascii=False, separators=(",", ":"))
        return hashlib.sha256(serialized.encode("utf-8")).hexdigest()


def main() -> None:
    binary = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/tks").resolve()
    for seed, steps, floor, scenes, flags in CASES:
        first = replay(binary, seed, steps, floor, scenes, flags)
        second = replay(binary, seed, steps, floor, scenes, flags)
        assert first == second, f"seed {seed}: replay histories diverged"
        print(f"PASS seed={seed} hero=zhuge difficulty=0 floor={floor} "
              f"actions={steps} scenes={','.join(scenes)} history_sha256={first}")
    print("Natural CLI story reachability: 2 seeds, 4 fresh campaigns, deterministic.")


if __name__ == "__main__":
    main()
