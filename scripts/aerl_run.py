#!/usr/bin/env python3
"""Run one command and append a privacy-minimized outcome to the AERL ledger."""
from __future__ import annotations

import argparse
import hashlib
import os
import subprocess
import sys
import time
from pathlib import Path

from aerl import AerlError, append_event

MAX_COMMAND_ARGS = 256


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--file", required=True, type=Path, help="shared AERL ledger")
    parser.add_argument("--actor", default="CI")
    parser.add_argument("--kind", required=True, help="stable test/workflow event identifier")
    parser.add_argument("--timeout-seconds", type=int, default=300, help="command timeout in seconds; 0 explicitly disables the timeout (default: 300)")
    parser.add_argument("command", nargs=argparse.REMAINDER, help="command after --")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    command = list(args.command)
    if command and command[0] == "--":
        command = command[1:]
    if not command or len(command) > MAX_COMMAND_ARGS or any(not isinstance(x, str) for x in command):
        print("FAIL: provide 1..256 command arguments after --", file=sys.stderr)
        return 2
    if args.timeout_seconds < 0 or args.timeout_seconds > 86400:
        print("FAIL: timeout must be between 0 and 86400 seconds", file=sys.stderr)
        return 2
    # Never persist command text: it may contain credentials or user data.
    command_hash = hashlib.sha256(b"\0".join(x.encode("utf-8", "surrogatepass") for x in command)).hexdigest()
    started = time.monotonic()
    exit_code = 127
    outcome = "FAIL"
    launch_error = None
    try:
        completed = subprocess.run(
            command, check=False, timeout=args.timeout_seconds or None,
            stdin=None, stdout=None, stderr=None,
        )
        exit_code = completed.returncode
        outcome = "PASS" if exit_code == 0 else "FAIL"
    except subprocess.TimeoutExpired:
        exit_code = 124
        outcome = "BLOCKED"
        launch_error = "timeout"
    except OSError as exc:
        launch_error = type(exc).__name__
        print(f"FAIL: unable to launch command ({launch_error})", file=sys.stderr)
    elapsed_ms = min(2**31 - 1, max(0, int((time.monotonic() - started) * 1000)))
    payload = {"command_sha256": command_hash, "argument_count": len(command),
               "exit_code": exit_code, "elapsed_ms": elapsed_ms}
    # Correlate CI evidence without copying the full environment (which may contain secrets).
    ci = {}
    for env_name, field_name in (
        ("GITHUB_SHA", "commit_sha"), ("GITHUB_RUN_ID", "run_id"),
        ("GITHUB_RUN_ATTEMPT", "run_attempt"), ("GITHUB_JOB", "job"),
        ("GITHUB_WORKFLOW", "workflow"), ("RUNNER_OS", "runner_os"),
        ("GITHUB_REPOSITORY", "repository"),
    ):
        value = os.environ.get(env_name)
        if value and len(value) <= 160 and "\\x00" not in value:
            ci[field_name] = value
    if ci:
        payload["ci"] = ci
    if launch_error:
        payload["launch_error_type"] = launch_error
    try:
        sequence, head = append_event(args.file, args.actor, args.kind, outcome, payload)
    except (AerlError, OSError, ValueError) as exc:
        print(f"FAIL: command ran but AERL recording failed: {exc}", file=sys.stderr)
        return 125
    print(f"AERL: seq={sequence} kind={args.kind} status={outcome} exit_code={exit_code} head={head}")
    return exit_code if exit_code != 0 else 0


if __name__ == "__main__":
    raise SystemExit(main())
