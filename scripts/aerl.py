#!/usr/bin/env python3
"""AERL: local, hash-chained audit evidence log. No network or third-party dependencies."""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import hmac
import json
import math
import os
import re
import stat
import sys
from contextlib import contextmanager
from pathlib import Path

SCHEMA = "AERL/1"
ZERO_HASH = "0" * 64
MAX_LEDGER_BYTES = 64 * 1024 * 1024
MAX_RECORD_BYTES = 32 * 1024
MAX_PAYLOAD_BYTES = 16 * 1024
RECORD_FIELDS = {
    "schema", "seq", "ts", "actor", "kind", "status", "payload", "prev", "hash"
}
STATUSES = {"INFO", "PASS", "FAIL", "PARTIAL", "BLOCKED", "NOT_VALIDATED", "UNPROVEN"}
ACTOR_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:@/-]{0,63}$")
KIND_RE = re.compile(r"^[A-Za-z][A-Za-z0-9_.:-]{0,79}$")
HASH_RE = re.compile(r"^[0-9a-f]{64}$")
SENSITIVE_KEY_PARTS = (
    "password", "passwd", "secret", "token", "authorization", "cookie",
    "credential", "privatekey", "apikey", "accesskey", "refresh_token",
    "email", "phone", "telephone", "recoveryphrase", "seedphrase",
)


class AerlError(Exception):
    """Expected refusal due to malformed, oversized, or unsafe evidence input."""


def canonical_json(value: object) -> bytes:
    try:
        return json.dumps(
            value, ensure_ascii=False, sort_keys=True, separators=(",", ":"),
            allow_nan=False,
        ).encode("utf-8")
    except (TypeError, ValueError, UnicodeEncodeError, RecursionError) as exc:
        raise AerlError(f"value is not canonical JSON: {exc}") from exc


def reject_duplicate_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise AerlError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def reject_json_constant(value: str):
    raise AerlError(f"non-standard JSON constant: {value}")


def strict_json(raw: bytes):
    try:
        return json.loads(
            raw.decode("utf-8"),
            object_pairs_hook=reject_duplicate_keys,
            parse_constant=reject_json_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, RecursionError) as exc:
        raise AerlError(f"invalid JSON/UTF-8: {exc}") from exc


def validate_payload(payload: object) -> None:
    if not isinstance(payload, dict):
        raise AerlError("payload must be a JSON object")

    def visit(value, depth: int) -> None:
        if depth > 12:
            raise AerlError("payload nesting exceeds 12 levels")
        if isinstance(value, dict):
            if len(value) > 128:
                raise AerlError("payload object exceeds 128 keys")
            for key, child in value.items():
                if not isinstance(key, str) or not key or len(key) > 80 or "\x00" in key:
                    raise AerlError("payload keys must be non-empty strings of at most 80 characters")
                normalized = re.sub(r"[^a-z0-9]+", "_", key.lower()).strip("_")
                compact = normalized.replace("_", "")
                if any(part.replace("_", "") in compact for part in SENSITIVE_KEY_PARTS):
                    raise AerlError(f"sensitive-looking payload key refused: {key}")
                visit(child, depth + 1)
        elif isinstance(value, list):
            if len(value) > 256:
                raise AerlError("payload list exceeds 256 items")
            for child in value:
                visit(child, depth + 1)
        elif isinstance(value, str):
            if len(value) > 4096 or "\x00" in value:
                raise AerlError("payload strings must be at most 4096 characters and contain no NUL")
        elif isinstance(value, bool) or value is None:
            return
        elif isinstance(value, int):
            if not -(2**63) <= value <= (2**63 - 1):
                raise AerlError("payload integer is outside signed 64-bit range")
        elif isinstance(value, float):
            if not math.isfinite(value):
                raise AerlError("payload numbers must be finite")
        else:
            raise AerlError(f"unsupported payload value type: {type(value).__name__}")

    visit(payload, 0)
    if len(canonical_json(payload)) > MAX_PAYLOAD_BYTES:
        raise AerlError(f"payload exceeds {MAX_PAYLOAD_BYTES} bytes")


def _valid_utc_timestamp(value: object) -> bool:
    if not isinstance(value, str) or not value.endswith("Z") or len(value) > 40:
        return False
    try:
        parsed = dt.datetime.fromisoformat(value[:-1] + "+00:00")
    except ValueError:
        return False
    return parsed.tzinfo is not None and parsed.utcoffset() == dt.timedelta(0)


def _hash_record(unsigned: dict) -> str:
    return hashlib.sha256(canonical_json(unsigned)).hexdigest()


def _validate_record(record: object, expected_seq: int, previous_hash: str) -> str:
    if not isinstance(record, dict):
        raise AerlError(f"record {expected_seq} is not a JSON object")
    if set(record) != RECORD_FIELDS:
        missing = sorted(RECORD_FIELDS - record.keys())
        extra = sorted(record.keys() - RECORD_FIELDS)
        raise AerlError(f"record {expected_seq} fields differ; missing={missing}, extra={extra}")
    if record["schema"] != SCHEMA:
        raise AerlError(f"record {expected_seq} has an unsupported schema")
    seq = record["seq"]
    if isinstance(seq, bool) or not isinstance(seq, int) or seq != expected_seq:
        raise AerlError(f"record sequence mismatch: expected {expected_seq}")
    if not _valid_utc_timestamp(record["ts"]):
        raise AerlError(f"record {expected_seq} timestamp must be ISO-8601 UTC ending in Z")
    if not isinstance(record["actor"], str) or not ACTOR_RE.fullmatch(record["actor"]):
        raise AerlError(f"record {expected_seq} has an invalid actor")
    if not isinstance(record["kind"], str) or not KIND_RE.fullmatch(record["kind"]):
        raise AerlError(f"record {expected_seq} has an invalid event kind")
    if not isinstance(record["status"], str) or record["status"] not in STATUSES:
        raise AerlError(f"record {expected_seq} has an invalid status")
    validate_payload(record["payload"])
    prev = record["prev"]
    if not isinstance(prev, str) or not HASH_RE.fullmatch(prev) or prev != previous_hash:
        raise AerlError(f"record {expected_seq} previous-hash link is invalid")
    actual_hash = record["hash"]
    if not isinstance(actual_hash, str) or not HASH_RE.fullmatch(actual_hash):
        raise AerlError(f"record {expected_seq} hash is malformed")
    unsigned = {key: value for key, value in record.items() if key != "hash"}
    calculated = _hash_record(unsigned)
    if not hmac.compare_digest(actual_hash, calculated):
        raise AerlError(f"record {expected_seq} hash mismatch")
    if len(canonical_json(record)) > MAX_RECORD_BYTES:
        raise AerlError(f"record {expected_seq} exceeds {MAX_RECORD_BYTES} bytes")
    return actual_hash


def verify_bytes(data: bytes, expected_head: str | None = None,
                 expected_sequence: int | None = None) -> tuple[int, str]:
    if len(data) > MAX_LEDGER_BYTES:
        raise AerlError(f"ledger exceeds {MAX_LEDGER_BYTES} bytes")
    if not data:
        count, head = 0, ZERO_HASH
    else:
        if not data.endswith(b"\n"):
            raise AerlError("ledger has a partial final record (missing newline)")
        rows = data[:-1].split(b"\n")
        previous_hash = ZERO_HASH
        for expected_seq, row in enumerate(rows, 1):
            if not row:
                raise AerlError(f"empty ledger row at sequence {expected_seq}")
            if len(row) > MAX_RECORD_BYTES:
                raise AerlError(f"ledger row {expected_seq} exceeds {MAX_RECORD_BYTES} bytes")
            record = strict_json(row)
            if canonical_json(record) != row:
                raise AerlError(f"record {expected_seq} is not canonical JSON")
            previous_hash = _validate_record(record, expected_seq, previous_hash)
        count, head = len(rows), previous_hash
    if expected_head is not None:
        if not isinstance(expected_head, str) or not HASH_RE.fullmatch(expected_head):
            raise AerlError("expected head must be 64 lowercase hexadecimal characters")
        if not hmac.compare_digest(head, expected_head):
            raise AerlError("ledger head differs from the externally supplied checkpoint")
    if expected_sequence is not None:
        if isinstance(expected_sequence, bool) or not isinstance(expected_sequence, int) or expected_sequence < 0 or count != expected_sequence:
            raise AerlError("ledger sequence differs from the externally supplied checkpoint")
    return count, head


def _open_regular(path: Path, flags: int, create: bool = False) -> int:
    if path.is_symlink():
        raise AerlError("ledger path must not be a symbolic link")
    if create and not path.parent.is_dir():
        raise AerlError("parent directory must already exist")
    flags |= getattr(os, "O_BINARY", 0) | getattr(os, "O_NONBLOCK", 0)
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    fd = os.open(path, flags, 0o600)
    try:
        opened = os.fstat(fd)
        current = path.lstat()
        if not stat.S_ISREG(opened.st_mode) or not stat.S_ISREG(current.st_mode):
            raise AerlError("ledger input must be a regular file")
        if (opened.st_dev, opened.st_ino) != (current.st_dev, current.st_ino):
            raise AerlError("ledger path changed while it was being opened")
        if opened.st_size > MAX_LEDGER_BYTES:
            raise AerlError(f"ledger exceeds {MAX_LEDGER_BYTES} bytes")
        return fd
    except Exception:
        os.close(fd)
        raise


@contextmanager
def _locked(fd: int):
    if os.name == "nt":
        import msvcrt
        os.lseek(fd, 0, os.SEEK_SET)
        msvcrt.locking(fd, msvcrt.LK_LOCK, 1)
        try:
            yield
        finally:
            os.lseek(fd, 0, os.SEEK_SET)
            msvcrt.locking(fd, msvcrt.LK_UNLCK, 1)
    else:
        import fcntl
        fcntl.flock(fd, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(fd, fcntl.LOCK_UN)


def _read_fd(fd: int) -> bytes:
    before = os.fstat(fd)
    if before.st_size > MAX_LEDGER_BYTES:
        raise AerlError(f"ledger exceeds {MAX_LEDGER_BYTES} bytes")
    os.lseek(fd, 0, os.SEEK_SET)
    chunks = []
    remaining = MAX_LEDGER_BYTES + 1
    while remaining:
        chunk = os.read(fd, min(1024 * 1024, remaining))
        if not chunk:
            break
        chunks.append(chunk)
        remaining -= len(chunk)
    data = b"".join(chunks)
    after = os.fstat(fd)
    if len(data) > MAX_LEDGER_BYTES:
        raise AerlError(f"ledger exceeds {MAX_LEDGER_BYTES} bytes")
    if before.st_size != after.st_size or len(data) != after.st_size:
        raise AerlError("ledger changed while being read")
    return data


def _read_locked_fd(fd: int) -> bytes:
    with _locked(fd):
        return _read_fd(fd)


def verify_file(path: Path, expected_head: str | None = None,
                expected_sequence: int | None = None) -> tuple[int, str]:
    fd = _open_regular(path, os.O_RDONLY)
    try:
        data = _read_locked_fd(fd)
    finally:
        os.close(fd)
    return verify_bytes(data, expected_head, expected_sequence)


def append_event(path: Path, actor: str, kind: str, status: str,
                 payload: object) -> tuple[int, str]:
    if not isinstance(actor, str) or not ACTOR_RE.fullmatch(actor):
        raise AerlError("actor must match the documented identifier format")
    if not isinstance(kind, str) or not KIND_RE.fullmatch(kind):
        raise AerlError("kind must match the documented event identifier format")
    if not isinstance(status, str) or status not in STATUSES:
        raise AerlError("status is not in the allowed status set")
    validate_payload(payload)
    flags = os.O_RDWR | os.O_CREAT | os.O_APPEND
    fd = _open_regular(path, flags, create=True)
    try:
        with _locked(fd):
            existing = _read_fd(fd)
            count, previous_hash = verify_bytes(existing)
            record = {
                "schema": SCHEMA,
                "seq": count + 1,
                "ts": dt.datetime.now(dt.timezone.utc).isoformat(timespec="microseconds").replace("+00:00", "Z"),
                "actor": actor,
                "kind": kind,
                "status": status,
                "payload": payload,
                "prev": previous_hash,
            }
            record["hash"] = _hash_record(record)
            encoded = canonical_json(record)
            if len(encoded) > MAX_RECORD_BYTES:
                raise AerlError(f"record exceeds {MAX_RECORD_BYTES} bytes")
            line = encoded + b"\n"
            if len(existing) + len(line) > MAX_LEDGER_BYTES:
                raise AerlError(f"append would exceed {MAX_LEDGER_BYTES} bytes; export and externally anchor a checkpoint first")
            os.lseek(fd, 0, os.SEEK_END)
            view = memoryview(line)
            while view:
                written = os.write(fd, view)
                if written <= 0:
                    raise AerlError("short write while appending ledger record")
                view = view[written:]
            os.fsync(fd)
            return record["seq"], record["hash"]
    finally:
        os.close(fd)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="AERL: local hash-chained evidence log; not a general syslog replacement."
    )
    sub = parser.add_subparsers(dest="command", required=True)
    append = sub.add_parser("append", help="append one structured event")
    append.add_argument("--file", required=True, type=Path)
    append.add_argument("--actor", required=True)
    append.add_argument("--kind", required=True)
    append.add_argument("--status", required=True, choices=sorted(STATUSES))
    append.add_argument("--payload", default="{}", help="JSON metadata object; sensitive-looking keys are refused")
    verify = sub.add_parser("verify", help="verify the ledger and optional external checkpoint")
    verify.add_argument("--file", required=True, type=Path)
    verify.add_argument("--expected-head")
    verify.add_argument("--expected-sequence", type=int)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        if args.command == "append":
            payload_raw = args.payload.encode("utf-8")
            if len(payload_raw) > MAX_PAYLOAD_BYTES:
                raise AerlError(f"payload exceeds {MAX_PAYLOAD_BYTES} bytes")
            payload = strict_json(payload_raw)
            seq, head = append_event(args.file, args.actor, args.kind, args.status, payload)
            print(json.dumps({
                "status": "PASS",
                "operation": "append",
                "sequence": seq,
                "head_hash": head,
                "external_anchor": "NOT_PROVIDED",
            }, sort_keys=True))
            return 0
        count, head = verify_file(args.file, args.expected_head, args.expected_sequence)
        print(json.dumps({
            "status": "PASS",
            "operation": "verify",
            "records": count,
            "head_hash": head,
            "external_anchor": "VERIFIED" if args.expected_head is not None or args.expected_sequence is not None else "NOT_PROVIDED",
        }, sort_keys=True))
        if args.expected_head is None and args.expected_sequence is None:
            print("LIMIT: chain consistency verified; truncation/rewrite of the whole ledger needs an independently stored checkpoint to detect.", file=sys.stderr)
        return 0
    except (AerlError, OSError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
