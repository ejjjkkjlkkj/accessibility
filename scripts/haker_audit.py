#!/usr/bin/env python3
"""Offline heuristic security review; findings are leads, not confirmed vulnerabilities."""
from __future__ import annotations
import argparse, json, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP_DIRS = {".git", ".venv", "venv", "node_modules", "target", "dist", "build", "__pycache__", ".mypy_cache", ".pytest_cache"}
TEXT_SUFFIXES = {".py", ".rs", ".c", ".h", ".cpp", ".hpp", ".cc", ".go", ".js", ".ts", ".json", ".toml", ".yaml", ".yml", ".md", ".ps1", ".sh", ".bat", ".cmd", ".ini", ".cfg", ".conf", ".xml", ".html", ".txt", ".lock"}
MAX_FILE_BYTES = 2 * 1024 * 1024
RULES = [
    ("SEC001", "HIGH", re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----"), "Private-key material marker"),
    ("SEC002", "HIGH", re.compile(r"""(?i)\b(?:api[_-]?key|access[_-]?token|client[_-]?secret|password)\s*[:=]\s*['"][^'"]{12,}['"]"""), "Possible hard-coded credential assignment"),
    ("SEC003", "MEDIUM", re.compile(r"(?i)\b(?:verify=False|CERT_NONE|check_hostname\s*=\s*False)\b"), "TLS certificate verification may be disabled"),
    ("SEC004", "MEDIUM", re.compile(r"(?i)\b(?:shell\s*=\s*True|os\.system\s*\(|eval\s*\(|exec\s*\()"), "Dynamic execution or shell invocation requires review"),
    ("SEC005", "MEDIUM", re.compile(r"(?i)\bunsafe\s*\{"), "Rust unsafe block requires manual safety review"),
    ("SEC006", "LOW", re.compile(r"(?i)\b(?:TODO|FIXME|XXX)\b.{0,100}(?:security|auth|overflow|unsafe|validation|permission)"), "Security-related unresolved marker"),
]
def iter_files(root):
    for path in sorted(root.rglob("*")):
        try:
            if path.is_symlink() or not path.is_file():
                continue
            if any(part in SKIP_DIRS for part in path.relative_to(root).parts):
                continue
            if path.suffix.lower() not in TEXT_SUFFIXES and path.name not in {"Dockerfile", "Makefile"}:
                continue
            if path.stat().st_size > MAX_FILE_BYTES:
                continue
        except OSError:
            continue
        yield path

def audit(root):
    findings, unreadable, scanned = [], [], 0
    for path in iter_files(root):
        rel = path.relative_to(root).as_posix()
        try:
            content = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            unreadable.append(rel)
            continue
        scanned += 1
        in_rule_definitions = False
        for number, line in enumerate(content.splitlines(), 1):
            stripped = line.strip()
            if stripped.startswith("RULES = ["):
                in_rule_definitions = True
                continue
            if in_rule_definitions:
                if stripped == "]":
                    in_rule_definitions = False
                continue
            if "SECURITY_SCANNER_TEST_FIXTURE" in line:
                continue
            for rule_id, severity, pattern, title in RULES:
                if pattern.search(line):
                    findings.append({"id": rule_id, "severity": severity, "title": title, "path": rel, "line": number, "evidence": line.strip()[:240], "status": "UNCONFIRMED_REVIEW_REQUIRED"})
    counts = {s: sum(f["severity"] == s for f in findings) for s in ("CRITICAL", "HIGH", "MEDIUM", "LOW", "INFO")}
    return {"schema_version":"1.0", "audit_type":"OFFLINE_HEURISTIC_STATIC_REVIEW", "root":str(root), "files_scanned":scanned, "unreadable_files":unreadable, "finding_count":len(findings), "severity_counts":counts, "findings":findings, "limitations":["Heuristic false positives and false negatives are possible.", "Findings require human reproduction and validation.", "No AI agent is launched and inspected code is not executed.", "A clean report does not prove absence of vulnerabilities.", "Hardware, firmware, runtime, and physical-device behavior are not validated."]}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--json-out", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    if not root.is_dir():
        print("FAIL: root is not a directory", file=sys.stderr)
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
    print("SEVERITY: " + ", ".join(f"{k}={v}" for k,v in report["severity_counts"].items()))
    print("LIMIT: heuristic static review only; not a security certification.")
    for f in report["findings"]:
        print(f"{f['severity']} {f['id']} {f['path']}:{f['line']} {f['title']}")
    return 0
if __name__ == "__main__":
    raise SystemExit(main())
