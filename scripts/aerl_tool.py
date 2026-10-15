#!/usr/bin/env python3
"""AERL repository log console: verify, summarize, and query a local AERL ledger."""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path

from aerl import AerlError, MAX_LEDGER_BYTES, _open_regular, _read_locked_fd, strict_json, verify_bytes

MAX_QUERY = 10_000


def load_records(path: Path) -> tuple[list[dict], str]:
    fd = _open_regular(path, 0)
    try:
        raw = _read_locked_fd(fd)
    finally:
        import os
        os.close(fd)
    count, head = verify_bytes(raw)
    records = []
    if raw:
        for row in raw[:-1].split(b"\n"):
            records.append(strict_json(row))
    if len(records) != count:
        raise AerlError("internal record count mismatch")
    return records, head


def summary(records: list[dict], head: str, path: Path) -> dict:
    statuses = Counter(r["status"] for r in records)
    kinds = Counter(r["kind"] for r in records)
    actors = Counter(r["actor"] for r in records)
    failures = [r for r in records if r["status"] in {"FAIL", "BLOCKED", "UNPROVEN", "NOT_VALIDATED"}]
    return {
        "schema": "AERL-REPORT/1",
        "status": "PASS",
        "operation": "summary",
        "ledger": str(path),
        "records": len(records),
        "head_hash": head,
        "status_counts": dict(sorted(statuses.items())),
        "event_kind_counts": dict(sorted(kinds.items())),
        "actor_counts": dict(sorted(actors.items())),
        "attention_records": [
            {"seq": r["seq"], "ts": r["ts"], "actor": r["actor"],
             "kind": r["kind"], "status": r["status"], "hash": r["hash"]}
            for r in failures[-100:]
        ],
        "limitations": [
            "A valid hash chain is not proof that events are truthful.",
            "Whole-ledger rewrite or tail truncation requires an independently protected checkpoint.",
            "No hardware, firmware, or physical-device state is inferred by this report.",
        ],
    }


def parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="AERL unified local log tool (offline; standard library only).")
    sub = p.add_subparsers(dest="command", required=True)
    for name, help_text in (
        ("verify", "verify the complete hash chain"),
        ("summary", "produce machine-readable repository log summary"),
    ):
        q = sub.add_parser(name, help=help_text)
        q.add_argument("--file", required=True, type=Path)
        q.add_argument("--out", type=Path, help="write JSON report to a new file; refuses overwrite")
    search = sub.add_parser("search", help="filter verified events by metadata")
    search.add_argument("--file", required=True, type=Path)
    search.add_argument("--kind")
    search.add_argument("--actor")
    search.add_argument("--status", choices=["INFO", "PASS", "FAIL", "PARTIAL", "BLOCKED", "NOT_VALIDATED", "UNPROVEN"])
    search.add_argument("--limit", type=int, default=100)
    search.add_argument("--out", type=Path, help="write JSON results to a new file; refuses overwrite")
    return p


def emit(value: dict | list, out: Path | None) -> None:
    encoded = (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n")
    if out is None:
        print(encoded, end="")
        return
    flags = __import__("os").O_WRONLY | __import__("os").O_CREAT | __import__("os").O_EXCL | getattr(__import__("os"), "O_BINARY", 0)
    if hasattr(__import__("os"), "O_NOFOLLOW"):
        flags |= __import__("os").O_NOFOLLOW
    fd = __import__("os").open(out, flags, 0o600)
    try:
        data = encoded.encode("utf-8")
        offset = 0
        while offset < len(data):
            n = __import__("os").write(fd, data[offset:])
            if n <= 0:
                raise AerlError("short write while writing report")
            offset += n
        __import__("os").fsync(fd)
    finally:
        __import__("os").close(fd)
    print(json.dumps({"status": "PASS", "report_path": str(out)}, sort_keys=True))


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        records, head = load_records(args.file)
        if args.command == "verify":
            emit({"schema": "AERL-REPORT/1", "status": "PASS", "operation": "verify",
                  "ledger": str(args.file), "records": len(records), "head_hash": head,
                  "chain_integrity": "VERIFIED", "external_anchor": "NOT_CHECKED"}, args.out)
            return 0
        if args.command == "summary":
            emit(summary(records, head, args.file), args.out)
            return 0
        if isinstance(args.limit, bool) or not 1 <= args.limit <= MAX_QUERY:
            raise AerlError(f"limit must be between 1 and {MAX_QUERY}")
        selected = [r for r in records
                    if (args.kind is None or r["kind"] == args.kind)
                    and (args.actor is None or r["actor"] == args.actor)
                    and (args.status is None or r["status"] == args.status)]
        selected = selected[-args.limit:]
        emit({"schema": "AERL-REPORT/1", "status": "PASS", "operation": "search",
              "matches": len(selected), "head_hash": head,
              "filters": {"kind": args.kind, "actor": args.actor, "status": args.status},
              "events": selected}, args.out)
        return 0
    except (AerlError, OSError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
