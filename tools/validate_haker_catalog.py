"""Structural validator for the HAKER case catalogue; does not execute security tests."""
from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

DEFAULT_PATH = Path("ZERO/HAKER-CASES.json")
ALLOWED_CASE_STATUSES = {
    "NOT_IMPLEMENTED", "NOT_RUN", "PASS", "FAIL", "PARTIAL", "BLOCKED",
    "UNPROVEN", "SIMULATION_ONLY",
}
REQUIRED_CASE_FIELDS = {"id", "area", "attack", "expected", "status"}


def validate_catalog(data: Any) -> dict[str, int]:
    if not isinstance(data, dict):
        raise ValueError("catalog root must be a JSON object")
    if data.get("schema_version") != 1:
        raise ValueError("schema_version must be 1")
    if data.get("project") != "HAKER":
        raise ValueError("project must be HAKER")
    policy = data.get("policy")
    if not isinstance(policy, dict):
        raise ValueError("policy must be an object")
    if policy.get("default_scope") != "local_repository_and_synthetic_fixtures":
        raise ValueError("default_scope must restrict execution to local repository and synthetic fixtures")
    if policy.get("network_tests_default") is not False:
        raise ValueError("network_tests_default must be false")
    if policy.get("destructive_tests_default") is not False:
        raise ValueError("destructive_tests_default must be false")
    if policy.get("require_explicit_scope_for_external_targets") is not True:
        raise ValueError("external targets must require explicit scope")
    cases = data.get("cases")
    if not isinstance(cases, list) or not cases:
        raise ValueError("cases must be a non-empty array")

    seen: set[str] = set()
    counts: dict[str, int] = {}
    for index, case in enumerate(cases):
        if not isinstance(case, dict):
            raise ValueError(f"cases[{index}] must be an object")
        missing = REQUIRED_CASE_FIELDS - case.keys()
        if missing:
            raise ValueError(f"cases[{index}] missing fields: {', '.join(sorted(missing))}")
        for field in REQUIRED_CASE_FIELDS - {"status"}:
            if not isinstance(case[field], str) or not case[field].strip():
                raise ValueError(f"cases[{index}].{field} must be a non-empty string")
        case_id = case["id"]
        if case_id in seen:
            raise ValueError(f"duplicate case id: {case_id}")
        seen.add(case_id)
        status = case["status"]
        if status not in ALLOWED_CASE_STATUSES:
            raise ValueError(f"cases[{index}].status is unsupported: {status}")
        counts[status] = counts.get(status, 0) + 1

    return counts


def main() -> int:
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PATH
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        counts = validate_catalog(data)
    except (OSError, UnicodeError, json.JSONDecodeError, ValueError) as exc:
        print(f"FAIL: HAKER catalogue validation: {exc}", file=sys.stderr)
        return 1

    total = sum(counts.values())
    print(f"PASS: HAKER catalogue structure and scope policy; cases={total}")
    for status in sorted(counts):
        print(f"CASE_STATUS {status}={counts[status]}")
    if counts.get("NOT_IMPLEMENTED", 0) or counts.get("NOT_RUN", 0):
        print("LIMITATION: catalogue validation is not execution of attack tests.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
