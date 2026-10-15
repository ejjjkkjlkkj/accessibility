#!/usr/bin/env python3
"""Offline heuristic security review; findings are leads, not confirmed vulnerabilities."""
from __future__ import annotations
import argparse, json, os, re, stat, sys, tempfile
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
def iter_files(root, exclude_paths=()):
    """Yield eligible files in stable order without descending into excluded trees."""
    excluded = {Path(item).resolve() for item in exclude_paths}
    for current, dirs, names in os.walk(root, topdown=True, followlinks=False):
        current_path = Path(current)
        dirs[:] = sorted(
            name for name in dirs
            if name not in SKIP_DIRS
            and not (current_path / name).is_symlink()
        )
        for name in sorted(names):
            path = current_path / name
            try:
                if path.is_symlink() or not path.is_file():
                    continue
                if path.suffix.lower() not in TEXT_SUFFIXES and path.name not in {"Dockerfile", "Makefile"}:
                    continue
                if path.resolve() in excluded:
                    continue
            except OSError:
                continue
            yield path


def read_bounded_text(path):
    """Read a regular file with a hard byte cap; refuse final-component symlinks where supported."""
    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0)
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    descriptor = os.open(path, flags)
    try:
        with os.fdopen(descriptor, "rb") as stream:
            descriptor = -1
            metadata = os.fstat(stream.fileno())
            if not stat.S_ISREG(metadata.st_mode) or metadata.st_size > MAX_FILE_BYTES:
                return None
            raw = stream.read(MAX_FILE_BYTES + 1)
            if len(raw) > MAX_FILE_BYTES:
                return None
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    return raw.decode("utf-8")


def audit(root, exclude_paths=()):
    findings, unreadable, scanned = [], [], 0
    for path in iter_files(root, exclude_paths=exclude_paths):
        rel = path.relative_to(root).as_posix()
        try:
            content = read_bounded_text(path)
        except (OSError, UnicodeDecodeError):
            unreadable.append(rel)
            continue
        if content is None:
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
    return {"schema_version":"1.0", "audit_type":"OFFLINE_HEURISTIC_STATIC_REVIEW", "root":".", "files_scanned":scanned, "unreadable_files":unreadable, "finding_count":len(findings), "severity_counts":counts, "findings":findings, "limitations":["Heuristic false positives and false negatives are possible.", "Findings require human reproduction and validation.", "No AI agent is launched and inspected code is not executed.", "A clean report does not prove absence of vulnerabilities.", "Hardware, firmware, runtime, and physical-device behavior are not validated."]}

def is_previous_report(path):
    """Permit refreshing an existing output only when it is already a HAKER report."""
    if path.suffix.lower() != ".json" or path.is_symlink() or not path.is_file():
        return False
    try:
        previous = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return False
    return (
        isinstance(previous, dict)
        and previous.get("schema_version") == "1.0"
        and previous.get("audit_type") == "OFFLINE_HEURISTIC_STATIC_REVIEW"
    )


def write_report_atomically(path, rendered):
    """Write through a same-directory temporary file so a symlink target is never followed."""
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.is_symlink():
        raise OSError("output path must not be a symbolic link")
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=".haker-report-", suffix=".tmp", dir=path.parent
    )
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            descriptor = -1
            stream.write(rendered)
        os.replace(temporary_name, path)
        temporary_name = ""
    finally:
        if descriptor >= 0:
            os.close(descriptor)
        if temporary_name and os.path.exists(temporary_name):
            os.unlink(temporary_name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--json-out", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    if not root.is_dir():
        print("FAIL: root is not a directory", file=sys.stderr)
        return 2

    output_path = args.json_out
    excluded_paths = ()
    if output_path is not None:
        try:
            if output_path.is_symlink():
                print("FAIL: report output must not be a symbolic link", file=sys.stderr)
                return 2
            if output_path.exists() and not is_previous_report(output_path):
                print(
                    "FAIL: refusing to overwrite an existing file unless it is a prior HAKER JSON report",
                    file=sys.stderr,
                )
                return 2
            excluded_paths = (output_path.resolve(),)
        except OSError as exc:
            print(f"FAIL: cannot validate report output: {exc}", file=sys.stderr)
            return 2

    report = audit(root, exclude_paths=excluded_paths)
    rendered = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    if output_path is not None:
        try:
            write_report_atomically(output_path, rendered)
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
