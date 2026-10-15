import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "validate_haker_report.py"


def report():
    return {
        "schema_version": "1.0",
        "audit_type": "OFFLINE_HEURISTIC_STATIC_REVIEW",
        "root": ".",
        "files_scanned": 1,
        "unreadable_files": [],
        "finding_count": 1,
        "severity_counts": {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 0, "LOW": 0, "INFO": 0},
        "findings": [{
            "id": "SEC001", "severity": "HIGH", "title": "private-key marker",
            "path": "sample.py", "line": 1, "evidence": "marker",
            "status": "UNCONFIRMED_REVIEW_REQUIRED"
        }],
        "limitations": ["Heuristic review only."]
    }


class HakerReportValidatorTests(unittest.TestCase):
    def run_validator(self, data):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "report.json"
            path.write_text(json.dumps(data), encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(VALIDATOR), str(path)],
                text=True, capture_output=True, check=False, timeout=10
            )

    def test_accepts_valid_report(self):
        result = self.run_validator(report())
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("PASS:", result.stdout)

    def test_rejects_inconsistent_severity_counts(self):
        data = report()
        data["severity_counts"]["HIGH"] = 0
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("severity count mismatch", result.stderr)

    def test_rejects_path_traversal(self):
        data = report()
        data["findings"][0]["path"] = "../outside.txt"
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("relative and confined", result.stderr)

    def test_rejects_absolute_root(self):
        data = report()
        data["root"] = "/private/runner/work"
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("stable relative marker", result.stderr)


if __name__ == "__main__":
    unittest.main()
