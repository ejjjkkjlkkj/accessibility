#!/usr/bin/env python3
"""Validate the stable machine-readable contract emitted by haker_audit.py."""
from __future__ import annotations

import json
import sys
from pathlib import Path, PureWindowsPath

SEVERITIES = {"CRITICAL", "HIGH", "MEDIUM", "LOW", "INFO"}
REQUIRED = {
    "schema_version", "audit_type", "root", "files_scanned", "unreadable_files",
    "finding_count", "severity_counts", "findings", "limitations",
}


def validate(report: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(report, dict):
        return ["report must be a JSON object"]
    missing = sorted(REQUIRED - report.keys())
    if missing:
        errors.append("missing fields: " + ", ".join(missing))
        return errors
    if report["schema_version"] != "1.0" or report["audit_type"] != "OFFLINE_HEURISTIC_STATIC_REVIEW":
        errors.append("unsupported schema_version or audit_type")
    if report["root"] != ".":
        errors.append("root must be the stable relative marker '.'")
    if not isinstance(report["files_scanned"], int) or isinstance(report["files_scanned"], bool) or report["files_scanned"] < 0:
        errors.append("files_scanned must be a non-negative integer")
    if not isinstance(report["unreadable_files"], list) or not all(isinstance(p, str) for p in report["unreadable_files"]):
        errors.append("unreadable_files must be a list of relative path strings")
    if not isinstance(report["findings"], list):
        errors.append("findings must be a list")
        findings = []
    else:
        findings = report["findings"]
    if not isinstance(report["finding_count"], int) or isinstance(report["finding_count"], bool) or report["finding_count"] != len(findings):
        errors.append("finding_count does not match findings length")
    counts = report["severity_counts"]
    if not isinstance(counts, dict) or set(counts) != SEVERITIES:
        errors.append("severity_counts must contain exactly " + ", ".join(sorted(SEVERITIES)))
    else:
        for severity in SEVERITIES:
            expected = sum(1 for f in findings if isinstance(f, dict) and f.get("severity") == severity)
            if counts[severity] != expected:
                errors.append(f"severity count mismatch for {severity}: expected {expected}")
    for index, finding in enumerate(findings):
        if not isinstance(finding, dict):
            errors.append(f"finding {index} must be an object")
            continue
        for key in ("id", "severity", "title", "path", "line", "evidence", "status"):
            if key not in finding:
                errors.append(f"finding {index} missing {key}")
        if finding.get("severity") not in SEVERITIES:
            errors.append(f"finding {index} has invalid severity")
        path = finding.get("path")
        if isinstance(path, str) and (Path(path).is_absolute() or PureWindowsPath(path).is_absolute() or ".." in Path(path).parts or ".." in PureWindowsPath(path).parts):
            errors.append(f"finding {index} path must be relative and confined")
        if not isinstance(finding.get("line"), int) or isinstance(finding.get("line"), bool) or finding.get("line", 0) < 1:
            errors.append(f"finding {index} line must be a positive integer")
        if finding.get("status") != "UNCONFIRMED_REVIEW_REQUIRED":
            errors.append(f"finding {index} has unexpected validation status")
    if not isinstance(report["limitations"], list) or not all(isinstance(item, str) for item in report["limitations"]):
        errors.append("limitations must be a list of strings")
    return errors


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: validate_haker_report.py REPORT.json", file=sys.stderr)
        return 2
    try:
        report = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        print(f"FAIL: cannot read valid JSON report: {exc}", file=sys.stderr)
        return 2
    errors = validate(report)
    if errors:
        for error in errors:
            print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("PASS: HAKER report schema, confinement, and severity counts validated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
