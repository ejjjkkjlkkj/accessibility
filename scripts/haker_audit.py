#!/usr/bin/env python3
"""Offline, heuristic security audit for repository source and metadata.

Findings are review leads, not confirmed vulnerabilities. No network access,
code execution, file modification, or firmware interaction is performed.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP_DIRS = {".git", ".venv", "venv", "node_modules", "target", "dist", "build", "__pycache__"}
TEXT_SUFFIXES = {
    ".py", ".rs", ".c", ".h", ".cpp", ".hpp", ".cc", ".go", ".js", ".ts",
    ".json", ".toml", ".yaml", ".yml", ".md", ".ps1", ".sh", ".bat", ".cmd",
    ".ini", ".cfg", ".conf", ".xml", ".html", ".txt", ".lock",
}
MAX_FILE_BYTES = 2 * 1024 * 1024

RULES = [
    ("SEC001", "HIGH", re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----"), "Private-key material marker"),
    ("SEC002", "HIGH", re.compile(r"(?i)\b(?:api[_-]?key|access[_-]?token|client[_-]?secret|password)\s*[:=]\s*['\"][^'\"]{12,}['\"]"), "Possible hard-coded credential assignment"),
    ("SEC003", "MEDIUM", re.compile(r"(?i)\b(?:verify=False|CERT_NONE|check_hostname\s*=\s*False)\b"), "TLS certificate verification may be disabled"),
    ("SEC004", "MEDIUM", re.compile(r"(?i)\b(?:shell\s*=\s*True|os\.system\s*\(|eval\s*\(|exec\s*\()"), "Dynamic execution or shell invocation requires review"),
    ("SEC005", "MEDIUM", re.compile(r"(?i)\bunsafe\s*\{"), "Rust unsafe block requires manual safety review"),
    ("SEC006", "LOW", re.compile(r"(?i)\b(?:TODO|FIXME|XXX)\b.{0,100}(?:security|auth|overflow|unsafe|validation|permission)"), "Security-related unresolved marker"),
]
SECRET_ALLOWLIST = {
    "scripts/validate_security_contract.py",
}
IGNORED_LINE_MARKERS = ("SECURITY_SCANNER_TEST_FIXTURE",)


def iter_files(root: Path):
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        rel_parts = path.relative_to(root).parts
        if any(part in SKIP_DIRS for part in rel_parts):
            continue
        if path.suffix.lower() not in TEXT_SUFFIXES and path.name not in {"Dockerfile", "Makefile"}:
            continue
        try:
            if path.stat().st_size > MAX_FILE_BYTES:
                continue
        except OSError:
            continue
        yield path


def audit(root: Path) -> dict:
    findings = []
    files_scanned = 0
    unreadable = []
    for path in iter_files(root):
        rel = path.relative_to(root).as_posix()
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            unreadable.append(rel)
            continue
        files_scanned += 1
        for line_no, line in enumerate(text.splitlines(), 1):
            if any(marker in line for marker in IGNORED_LINE_MARKERS):
                continue
            for rule_id, severity, pattern, title in RULES:
                if pattern.search(line):
                    if rule_id == "SEC002" and rel in SECRET_ALLOWLIST:
                        continue
                    findings.append({
                        "id": rule_id,
                        "severity": severity,
                        "title": title,
                        "path": rel,
                        "line": line_no,
                        "evidence": line.strip()[:240],
                        "status": "UNCONFIRMED_REVIEW_REQUIRED",
                    })
    return {
        "schema_version": "1.0",
        "audit_type": "OFFLINE_HEURISTIC_STATIC_REVIEW",
        "root": str(root),
        "files_scanned": files_scanned,
        "unreadable_files": unreadable,
        "finding_count": len(findings),
        "findings": findings,
        "limitations": [
            "Heuristic findings may be false positives or false negatives.",
            "No agent is launched and no code is executed.",
            "A clean report does not prove absence of vulnerabilities.",
            "Hardware, firmware, runtime, and physical-device behavior are not validated.",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT, help="Repository root to inspect")
    parser.add_argument("--json-out", type=Path, help="Optional JSON report path")
    args = parser.parse_args()
    root = args.root.resolve()
    if not root.is_dir():
        print(f"FAIL: root is not a directory: {root}", file=sys.stderr)
        return 2
    report = audit(root)
    rendered = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    if args.json_out:
        try:
            args.json_out.parent.mkdir(parents=True, exist_ok=True)
            args.json_out.write_text(rendered, encoding="utf-8")
        except OSError as exc:
            print(f"FAIL: cannot write report: {exc}", file=sys.stderr)
            return 2
    print(f"PASS: scanned {report['files_scanned']} text files")
    print(f"FINDINGS: {report['finding_count']} unconfirmed review leads")
    print("LIMIT: heuristic static review only; not a security certification.")
    if report["finding_count"]:
        for finding in report["findings"]:
            print(f"{finding['severity']} {finding['id']} {finding['path']}:{finding['line']} {finding['title']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
