#!/usr/bin/env python3
"""Create and verify portable AERL checkpoints; store checkpoints independently for real protection."""
from __future__ import annotations

import argparse
import json
import os
import stat
import sys
from pathlib import Path

from aerl import AerlError, HASH_RE, MAX_RECORD_BYTES, canonical_json, reject_duplicate_keys, reject_json_constant, strict_json, verify_file

CHECKPOINT_SCHEMA = "AERL-CHECKPOINT/1"
FIELDS = {"schema", "sequence", "head_hash"}


def read_checkpoint(path: Path) -> dict:
    if path.is_symlink():
        raise AerlError("checkpoint path must not be a symbolic link")
    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NONBLOCK", 0)
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    fd = os.open(path, flags)
    try:
        opened = os.fstat(fd)
        current = path.lstat()
        if not stat.S_ISREG(opened.st_mode) or not stat.S_ISREG(current.st_mode):
            raise AerlError("checkpoint must be a regular file")
        if (opened.st_dev, opened.st_ino) != (current.st_dev, current.st_ino):
            raise AerlError("checkpoint path changed while being opened")
        if opened.st_size > MAX_RECORD_BYTES:
            raise AerlError("checkpoint exceeds size limit")
        raw = os.read(fd, MAX_RECORD_BYTES + 1)
        if len(raw) > MAX_RECORD_BYTES or opened.st_size != len(raw):
            raise AerlError("checkpoint size changed or exceeds size limit")
    finally:
        os.close(fd)
    data = strict_json(raw)
    if not isinstance(data, dict) or set(data) != FIELDS:
        raise AerlError("checkpoint fields do not match the required schema")
    if data["schema"] != CHECKPOINT_SCHEMA:
        raise AerlError("unsupported checkpoint schema")
    sequence = data["sequence"]
    if isinstance(sequence, bool) or not isinstance(sequence, int) or sequence < 0:
        raise AerlError("checkpoint sequence must be a non-negative integer")
    if not isinstance(data["head_hash"], str) or not HASH_RE.fullmatch(data["head_hash"]):
        raise AerlError("checkpoint head_hash must be 64 lowercase hexadecimal characters")
    if canonical_json(data) + b"\\n" != raw:
        raise AerlError("checkpoint is not canonical JSON or lacks its final newline")
    return data


def create_checkpoint(ledger: Path, output: Path) -> dict:
    sequence, head_hash = verify_file(ledger)
    checkpoint = {"schema": CHECKPOINT_SCHEMA, "sequence": sequence, "head_hash": head_hash}
    encoded = canonical_json(checkpoint) + b"\n"
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_BINARY", 0)
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    fd = os.open(output, flags, 0o600)
    try:
        view = memoryview(encoded)
        while view:
            written = os.write(fd, view)
            if written <= 0:
                raise AerlError("short write while creating checkpoint")
            view = view[written:]
        os.fsync(fd)
    finally:
        os.close(fd)
    return checkpoint


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Create or verify a portable AERL checkpoint.")
    sub = parser.add_subparsers(dest="command", required=True)
    create = sub.add_parser("create", help="create a new checkpoint file; refuses to overwrite")
    create.add_argument("--ledger", required=True, type=Path)
    create.add_argument("--out", required=True, type=Path)
    verify = sub.add_parser("verify", help="verify ledger against a saved checkpoint")
    verify.add_argument("--ledger", required=True, type=Path)
    verify.add_argument("--checkpoint", required=True, type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        if args.command == "create":
            checkpoint = create_checkpoint(args.ledger, args.out)
            print(json.dumps({"status": "PASS", "operation": "checkpoint-created", **checkpoint}, sort_keys=True))
        else:
            checkpoint = read_checkpoint(args.checkpoint)
            count, head = verify_file(args.ledger, expected_head=checkpoint["head_hash"],
                                      expected_sequence=checkpoint["sequence"])
            print(json.dumps({"status": "PASS", "operation": "checkpoint-verified",
                              "records": count, "head_hash": head}, sort_keys=True))
        return 0
    except (AerlError, OSError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
