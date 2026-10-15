#!/usr/bin/env python3
"""AERL repository log console: verify, summarize, query, and collect safe report metadata."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import stat
import sys
import xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path

from aerl import AerlError, MAX_LEDGER_BYTES, MAX_PAYLOAD_BYTES, _open_regular, _read_locked_fd, append_event, strict_json, verify_bytes
import aerl_checkpoint

MAX_QUERY = 10_000
MAX_SOURCE_BYTES = 2 * 1024 * 1024
SKIP_DIRS = {".git", ".hg", ".svn", ".venv", "venv", "node_modules", "__pycache__", "target", "dist", "build"}
REPORT_NAMES = {
    "haker-audit-report.json", "accessibility-report.json", "test-results.json",
    "pytest-report.json", "coverage-summary.json", "aerl-report.json",
}
REPORT_SUFFIXES = (".junit.xml", "-junit.xml", "_junit.xml")
ATTENTION = {"FAIL", "BLOCKED", "UNPROVEN", "NOT_VALIDATED"}


def load_records(path: Path) -> tuple[list[dict], str]:
    fd = _open_regular(path, os.O_RDONLY)
    try:
        raw = _read_locked_fd(fd)
    finally:
        os.close(fd)
    count, head = verify_bytes(raw)
    records = [strict_json(row) for row in raw[:-1].split(b"\n")] if raw else []
    if len(records) != count:
        raise AerlError("internal record count mismatch")
    return records, head


def summary(records: list[dict], head: str, path: Path) -> dict:
    statuses = Counter(r["status"] for r in records)
    kinds = Counter(r["kind"] for r in records)
    actors = Counter(r["actor"] for r in records)
    failures = [r for r in records if r["status"] in ATTENTION]
    return {
        "schema": "AERL-REPORT/1", "status": "PASS", "operation": "summary",
        "ledger": str(path), "records": len(records), "head_hash": head,
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


def search_records(records: list[dict], *, kind: str | None = None,
                  actor: str | None = None, status: str | None = None,
                  commit_sha: str | None = None, run_id: str | None = None,
                  job: str | None = None) -> list[dict]:
    selected = []
    for record in records:
        payload = record.get("payload", {})
        ci = payload.get("ci", {}) if isinstance(payload, dict) else {}
        if kind is not None and record["kind"] != kind:
            continue
        if actor is not None and record["actor"] != actor:
            continue
        if status is not None and record["status"] != status:
            continue
        if commit_sha is not None and ci.get("commit_sha") != commit_sha:
            continue
        if run_id is not None and ci.get("run_id") != run_id:
            continue
        if job is not None and ci.get("job") != job:
            continue
        selected.append(record)
    return selected


def _report_candidate(path: Path) -> bool:
    name = path.name.lower()
    return name in REPORT_NAMES or name.endswith(REPORT_SUFFIXES) or (
        name.startswith("junit") and name.endswith(".xml")
    )


def _safe_source(root: Path, path: Path) -> tuple[bytes, str]:
    if path.is_symlink():
        raise AerlError("report source must not be a symbolic link")
    resolved_root = root.resolve()
    resolved = path.resolve()
    if resolved_root not in resolved.parents:
        raise AerlError("report source escaped scan root")
    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NONBLOCK", 0)
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    fd = os.open(path, flags)
    try:
        before = os.fstat(fd)
        current = path.lstat()
        if not stat.S_ISREG(before.st_mode) or not stat.S_ISREG(current.st_mode):
            raise AerlError("report source must be a regular file")
        if (before.st_dev, before.st_ino) != (current.st_dev, current.st_ino):
            raise AerlError("report source path changed while opening")
        if before.st_size > MAX_SOURCE_BYTES:
            raise AerlError("report source exceeds 2 MiB limit")
        chunks = []
        remaining = MAX_SOURCE_BYTES + 1
        while remaining:
            chunk = os.read(fd, min(65536, remaining))
            if not chunk:
                break
            chunks.append(chunk)
            remaining -= len(chunk)
        raw = b"".join(chunks)
        after = os.fstat(fd)
        current_after = path.lstat()
        if len(raw) > MAX_SOURCE_BYTES:
            raise AerlError("report source exceeds 2 MiB limit")
        if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (
            after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns
        ):
            raise AerlError("report source changed while being read")
        if (after.st_dev, after.st_ino) != (current_after.st_dev, current_after.st_ino):
            raise AerlError("report source path was replaced while being read")
        if len(raw) != after.st_size:
            raise AerlError("report source size changed while being read")
    finally:
        os.close(fd)
    return raw, hashlib.sha256(raw).hexdigest()


def _extract_report(path: Path, raw: bytes) -> tuple[str, str, dict]:
    name = path.name.lower()
    if name.endswith(".xml"):
        try:
            root = ET.fromstring(raw)
        except (ET.ParseError, ValueError) as exc:
            raise AerlError(f"invalid JUnit XML report: {exc}") from exc
        suites = [root] if root.tag == "testsuite" else list(root.iter("testsuite"))
        tests = failures = errors = skipped = 0
        for suite in suites:
            for key, total in (("tests", "tests"), ("failures", "failures"),
                               ("errors", "errors"), ("skipped", "skipped")):
                value = suite.attrib.get(key, "0")
                try:
                    number = int(value)
                except (TypeError, ValueError) as exc:
                    raise AerlError(f"invalid JUnit {key} count") from exc
                if number < 0 or number > 10_000_000:
                    raise AerlError(f"JUnit {key} count outside accepted range")
                if total == "tests": tests += number
                elif total == "failures": failures += number
                elif total == "errors": errors += number
                else: skipped += number
        status = "FAIL" if failures or errors else "PASS"
        return "report.junit", status, {"format": "junit-xml", "tests": tests,
                                        "failures": failures, "errors": errors, "skipped": skipped}
    try:
        value = strict_json(raw)
    except AerlError as exc:
        raise AerlError(f"invalid JSON report: {exc}") from exc
    if not isinstance(value, dict):
        raise AerlError("JSON report root must be an object")
    # HAKER's offline heuristic report intentionally remains UNPROVEN: its findings
    # require independent reproduction. Copy only aggregate counters, never evidence text.
    if value.get("audit_type") == "OFFLINE_HEURISTIC_STATIC_REVIEW" and value.get("schema_version") == "1.0":
        counts = {}
        for key in ("files_scanned", "unreadable_count", "finding_count"):
            item = value.get(key)
            if isinstance(item, int) and not isinstance(item, bool) and 0 <= item <= 10_000_000:
                counts[key] = item
        unreadable = value.get("unreadable_files")
        if isinstance(unreadable, list):
            counts["unreadable_count"] = len(unreadable)
        severity = value.get("severity_counts")
        if isinstance(severity, dict):
            for key in ("CRITICAL", "HIGH", "MEDIUM", "LOW", "INFO"):
                item = severity.get(key)
                if isinstance(item, int) and not isinstance(item, bool) and 0 <= item <= 10_000_000:
                    counts["severity_" + key.lower()] = item
        return "report.haker.audit", "UNPROVEN", {"format": "haker-audit-json",
                "reported_status": "UNPROVEN_REQUIRES_REVIEW", "counts": counts}
    raw_status = value.get("status", value.get("result", value.get("conclusion", "UNPROVEN")))
    if not isinstance(raw_status, str):
        raw_status = "UNPROVEN"
    mapped = raw_status.upper()
    status = mapped if mapped in {"PASS", "FAIL", "PARTIAL", "BLOCKED", "NOT_VALIDATED", "UNPROVEN", "INFO"} else "UNPROVEN"
    counts = {}
    for key in ("tests", "passed", "failed", "errors", "skipped", "findings", "issues"):
        item = value.get(key)
        if isinstance(item, int) and not isinstance(item, bool) and 0 <= item <= 10_000_000:
            counts[key] = item
    return "report.json", status, {"format": "json", "reported_status": raw_status[:80],
                                   "counts": counts}


def collect(root: Path, ledger: Path, actor: str = "AERL-COLLECTOR") -> dict:
    if root.is_symlink() or not root.is_dir():
        raise AerlError("scan root must be an existing directory and not a symlink")
    root = root.resolve()
    ledger_resolved = ledger.resolve()
    existing, _ = load_records(ledger) if ledger.exists() else ([], "0" * 64)
    seen = {(r["payload"].get("source_path"), r["payload"].get("source_sha256"))
            for r in existing if isinstance(r.get("payload"), dict)}
    scanned = appended = duplicates = refused = 0
    errors = []
    for base, dirs, files in os.walk(root, followlinks=False):
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS and not (Path(base) / d).is_symlink())
        for filename in sorted(files):
            source = Path(base) / filename
            if not _report_candidate(source) or source.resolve() == ledger_resolved:
                continue
            scanned += 1
            relative = source.relative_to(root).as_posix()
            try:
                raw, digest = _safe_source(root, source)
                if (relative, digest) in seen:
                    duplicates += 1
                    continue
                kind, status, extracted = _extract_report(source, raw)
                payload = {"source_path": relative, "source_sha256": digest,
                           "source_bytes": len(raw), **extracted}
                append_event(ledger, actor, kind, status, payload)
                seen.add((relative, digest))
                appended += 1
            except (AerlError, OSError, ValueError, ET.ParseError) as exc:
                refused += 1
                errors.append({"source_path": relative, "reason": str(exc)[:240]})
    return {"schema": "AERL-REPORT/1", "status": "PARTIAL" if refused else "PASS",
            "operation": "collect", "root": str(root), "ledger": str(ledger),
            "sources_scanned": scanned, "events_appended": appended,
            "duplicates_skipped": duplicates, "sources_refused": refused,
            "refusals": errors[:100],
            "limitations": ["Only allowlisted machine-readable report filenames are collected.",
                            "Only report metadata and hashes are recorded; raw report contents are not copied.",
                            "Collection does not discover every repository event or prove report truthfulness."]}


def parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="AERL unified local log tool (offline; standard library only).")
    sub = p.add_subparsers(dest="command", required=True)
    init = sub.add_parser("init", help="create an empty ledger")
    init.add_argument("--file", required=True, type=Path)
    append = sub.add_parser("append", help="append one structured event")
    append.add_argument("--file", required=True, type=Path)
    append.add_argument("--actor", required=True)
    append.add_argument("--kind", required=True)
    append.add_argument("--status", required=True, choices=["INFO", "PASS", "FAIL", "PARTIAL", "BLOCKED", "NOT_VALIDATED", "UNPROVEN"])
    append.add_argument("--payload", default="{}", help="JSON metadata object; sensitive-looking keys are refused")
    for name, help_text in (("verify", "verify the complete hash chain"),
                            ("summary", "produce machine-readable log summary")):
        q = sub.add_parser(name, help=help_text)
        q.add_argument("--file", required=True, type=Path)
        q.add_argument("--out", type=Path, help="write JSON report to a new file; refuses overwrite")
    search = sub.add_parser("search", help="filter verified events")
    search.add_argument("--file", required=True, type=Path)
    search.add_argument("--kind")
    search.add_argument("--actor")
    search.add_argument("--status", choices=["INFO", "PASS", "FAIL", "PARTIAL", "BLOCKED", "NOT_VALIDATED", "UNPROVEN"])
    search.add_argument("--commit-sha", help="filter on payload.ci.commit_sha")
    search.add_argument("--run-id", help="filter on payload.ci.run_id")
    search.add_argument("--job", help="filter on payload.ci.job")
    search.add_argument("--limit", type=int, default=100)
    search.add_argument("--out", type=Path, help="write JSON results to a new file; refuses overwrite")
    collect_parser = sub.add_parser("collect", help="collect metadata from allowlisted repository reports")
    collect_parser.add_argument("--root", type=Path, default=Path("."))
    collect_parser.add_argument("--file", required=True, type=Path, help="shared AERL ledger")
    collect_parser.add_argument("--actor", default="AERL-COLLECTOR")
    cp_create = sub.add_parser("checkpoint-create", help="create a checkpoint without overwriting")
    cp_create.add_argument("--file", required=True, type=Path)
    cp_create.add_argument("--out", required=True, type=Path)
    cp_verify = sub.add_parser("checkpoint-verify", help="verify against a saved checkpoint")
    cp_verify.add_argument("--file", required=True, type=Path)
    cp_verify.add_argument("--checkpoint", required=True, type=Path)
    return p


def emit(value: dict | list, out: Path | None = None) -> None:
    encoded = json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
    if out is None:
        print(encoded, end="")
        return
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_BINARY", 0)
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    fd = os.open(out, flags, 0o600)
    try:
        data = encoded.encode("utf-8")
        view = memoryview(data)
        while view:
            written = os.write(fd, view)
            if written <= 0:
                raise AerlError("short write while writing report")
            view = view[written:]
        os.fsync(fd)
    finally:
        os.close(fd)
    print(json.dumps({"status": "PASS", "report_path": str(out)}, sort_keys=True))


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        if args.command == "init":
            fd = os.open(args.file, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_BINARY", 0), 0o600)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
            emit({"schema": "AERL-REPORT/1", "status": "PASS", "operation": "init",
                  "ledger": str(args.file), "records": 0})
            return 0
        if args.command == "append":
            payload_raw = args.payload.encode("utf-8")
            if len(payload_raw) > MAX_PAYLOAD_BYTES:
                raise AerlError(f"payload exceeds {MAX_PAYLOAD_BYTES} bytes")
            seq, head = append_event(args.file, args.actor, args.kind, args.status, strict_json(payload_raw))
            emit({"schema": "AERL-REPORT/1", "status": "PASS", "operation": "append",
                  "sequence": seq, "head_hash": head, "external_anchor": "NOT_PROVIDED"})
            return 0
        if args.command == "collect":
            report = collect(args.root, args.file, args.actor)
            emit(report)
            return 1 if report["status"] == "PARTIAL" else 0
        if args.command == "checkpoint-create":
            value = aerl_checkpoint.create_checkpoint(args.file, args.out)
            emit({"schema": "AERL-REPORT/1", "status": "PASS", "operation": "checkpoint-created", **value})
            return 0
        if args.command == "checkpoint-verify":
            checkpoint = aerl_checkpoint.read_checkpoint(args.checkpoint)
            count, head = __import__("aerl").verify_file(
                args.file, expected_head=checkpoint["head_hash"], expected_sequence=checkpoint["sequence"])
            emit({"schema": "AERL-REPORT/1", "status": "PASS", "operation": "checkpoint-verified",
                  "records": count, "head_hash": head})
            return 0
        records, head = load_records(args.file)
        if args.command == "verify":
            emit({"schema": "AERL-REPORT/1", "status": "PASS", "operation": "verify",
                  "ledger": str(args.file), "records": len(records), "head_hash": head,
                  "chain_integrity": "VERIFIED", "external_anchor": "NOT_CHECKED"}, args.out)
        elif args.command == "summary":
            emit(summary(records, head, args.file), args.out)
        else:
            if isinstance(args.limit, bool) or not 1 <= args.limit <= MAX_QUERY:
                raise AerlError(f"limit must be between 1 and {MAX_QUERY}")
            selected = search_records(records, kind=args.kind, actor=args.actor,
                                       status=args.status, commit_sha=args.commit_sha,
                                       run_id=args.run_id, job=args.job)
            emit({"schema": "AERL-REPORT/1", "status": "PASS", "operation": "search",
                  "matches": len(selected), "head_hash": head,
                  "filters": {"kind": args.kind, "actor": args.actor, "status": args.status,
                              "commit_sha": args.commit_sha, "run_id": args.run_id, "job": args.job},
                  "events": selected[-args.limit:]}, args.out)
        return 0
    except (AerlError, OSError, ValueError, ET.ParseError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
