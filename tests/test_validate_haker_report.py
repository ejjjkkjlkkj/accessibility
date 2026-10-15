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

    def test_rejects_boolean_severity_count(self):
        data = report()
        data["severity_counts"]["HIGH"] = True
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("non-negative integer", result.stderr)

    def test_rejects_path_traversal(self):
        data = report()
        data["findings"][0]["path"] = "../private.txt"
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("relative and confined", result.stderr)

    def test_rejects_windows_absolute_finding_path(self):
        data = report()
        data["findings"][0]["path"] = r"C:\runner\private.txt"
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("relative and confined", result.stderr)

    def test_rejects_windows_drive_relative_path(self):
        data = report()
        data["findings"][0]["path"] = r"C:private.txt"
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("relative and confined", result.stderr)

    def test_rejects_unreadable_path_traversal(self):
        data = report()
        data["unreadable_files"] = ["../../outside.txt"]
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("unreadable_files[0]", result.stderr)

    def test_rejects_nul_in_unreadable_path(self):
        data = report()
        data["unreadable_files"] = ["sample.py\x00private"]
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("unreadable_files[0]", result.stderr)

    def test_rejects_absolute_root(self):
        data = report()
        data["root"] = "/private/runner/work"
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("stable relative marker", result.stderr)

    def test_rejects_boolean_line_number(self):
        data = report()
        data["findings"][0]["line"] = True
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("line must be a positive integer", result.stderr)

    def test_rejects_unknown_severity(self):
        data = report()
        data["findings"][0]["severity"] = "URGENT"
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("invalid severity", result.stderr)

    def test_rejects_dot_as_finding_path(self):
        data = report()
        data["findings"][0]["path"] = "."
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("relative and confined", result.stderr)

    def test_rejects_nul_in_finding_path(self):
        data = report()
        data["findings"][0]["path"] = "sample.py\x00private"
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("relative and confined", result.stderr)

    def test_rejects_unhashable_severity_without_crashing(self):
        data = report()
        data["findings"][0]["severity"] = ["HIGH"]
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("invalid severity", result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_rejects_empty_finding_path(self):
        data = report()
        data["findings"][0]["path"] = ""
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("relative and confined", result.stderr)

    def test_rejects_non_list_findings(self):
        data = report()
        data["findings"] = {}
        data["finding_count"] = 0
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("findings must be a list", result.stderr)

    def test_rejects_non_string_limitations(self):
        data = report()
        data["limitations"] = ["heuristic only", 7]
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("limitations must be a list of strings", result.stderr)

    def test_rejects_empty_finding_evidence(self):
        data = report()
        data["findings"][0]["evidence"] = " "
        result = self.run_validator(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("evidence must be a non-empty string", result.stderr)


if __name__ == "__main__":
    unittest.main()
