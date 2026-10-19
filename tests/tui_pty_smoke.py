#!/usr/bin/env python3
"""Exercise the actual --tui program inside a controlling POSIX pseudo-terminal.

Unlike stdout snapshots or unit-only keyboard menu checks, this verifies the
real terminal raw-mode path, arrow-key decoding, persistence, and checked quit.
Uses only the Python standard library on macOS and Linux.
"""
from __future__ import annotations

import errno
import fcntl
import json
import os
import pathlib
import pty
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: python3 tests/tui_pty_smoke.py <path-to-tks>")

    executable = str(pathlib.Path(sys.argv[1]).resolve())
    with tempfile.TemporaryDirectory(prefix="tks-real-pty-") as tmp:
        save_dir = pathlib.Path(tmp)
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))

        def become_tty_leader() -> None:
            os.setsid()
            fcntl.ioctl(0, termios.TIOCSCTTY, 0)

        process = subprocess.Popen(
            [executable, "--tui", "--save-dir", str(save_dir)],
            stdin=slave, stdout=slave, stderr=slave,
            preexec_fn=become_tty_leader,
            env={**os.environ, "TERM": "xterm-256color"},
        )
        os.close(slave)
        output = bytearray()

        def pump(timeout: float = 0.1) -> None:
            readable, _, _ = select.select([master], [], [], timeout)
            if readable:
                try:
                    data = os.read(master, 65536)
                except OSError as error:
                    if error.errno != errno.EIO:
                        raise
                    return
                output.extend(data)
                if len(output) > 256_000:
                    del output[:-128_000]

        def current_snapshot() -> dict:
            try:
                return json.loads((save_dir / "save.json").read_text(encoding="utf-8"))
            except (OSError, ValueError):
                return {}

        def await_condition(predicate, description: str, timeout: float = 25.0) -> None:
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                pump()
                if predicate():
                    return
                if process.poll() is not None:
                    raise AssertionError(
                        f"{description}: TUI exited early ({process.returncode}); "
                        f"terminal output tail={output[-4000:]!r}"
                    )
            raise AssertionError(
                f"timeout waiting for {description}; terminal output tail={output[-4000:]!r}"
            )

        try:
            # A pipe-only fallback would not emit the full-screen TUI clear sequence.
            await_condition(lambda: b"\x1b[2J\x1b[H" in output, "first full-screen TUI render")
            # Arrow down moves selection from Guan Yu to Zhuge Liang; Enter starts.
            os.write(master, b"\x1b[B\r")
            await_condition(
                lambda: current_snapshot().get("hero") == "zhuge_liang"
                and current_snapshot().get("stage") == "map",
                "arrow-selected Zhuge Liang campaign saved",
            )
            first = current_snapshot()
            assert first["revision"] == 0, first
            # The number key must execute the first reachable map option.
            os.write(master, b"1")
            await_condition(
                lambda: current_snapshot().get("floor", 0) == 1
                and current_snapshot().get("revision", 0) >= 1,
                "numeric menu choice committed a floor",
            )
            played = current_snapshot()
            assert played["hero"] == "zhuge_liang"
            # Exercise the raw-mode colon command input, then checked quit.
            os.write(master, b":help\r")
            await_condition(
                lambda: output.count(b"\x1b[2J\x1b[H") >= 4,
                "TUI redrew after advanced command",
            )
            os.write(master, b"q")
            try:
                code = process.wait(timeout=15)
            except subprocess.TimeoutExpired as error:
                raise AssertionError("checked q shortcut did not terminate TUI") from error
            pump(0.0)
            assert code == 0, f"TUI returned {code}; tail={output[-4000:]!r}"
            saved = current_snapshot()
            assert saved["revision"] == played["revision"], (played, saved)
            assert saved["floor"] == 1 and saved["hero"] == "zhuge_liang"
            assert (save_dir / "chronicle.json").exists(), "new run ledger missing"
            print("Real controlling-PTY TUI: arrow, Enter, numeric route, colon help, checked q, durable save passed")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=10)
            os.close(master)


if __name__ == "__main__":
    main()
