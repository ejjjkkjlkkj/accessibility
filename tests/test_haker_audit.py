import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCANNER = ROOT / "scripts" / "haker_audit.py"


class HakerAuditTests(unittest.TestCase):
    def run_scanner(self, root):
        return subprocess.run(
            [sys.executable, str(SCANNER), "--root", str(root)],
            text=True, capture_output=True, check=False, timeout=10,
        )

    def test_detects_private_key_marker_without_executing_content(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "sample.py").write_text(
                "# SECURITY_SCANNER_TEST_FIXTURE\n"
                "PRIVATE = '-----BEGIN RSA PRIVATE KEY-----'\n",
                encoding="utf-8",
            )
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("SEC001", result.stdout)

    def test_clean_input_reports_limitations(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "clean.py").write_text("value = 42\n", encoding="utf-8")
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("FINDINGS: 0", result.stdout)
            self.assertIn("not a security certification", result.stdout)

    def test_json_report_is_machine_readable(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "clean.py").write_text("value = 42\n", encoding="utf-8")
            report = root / "out" / "report.json"
            result = subprocess.run(
                [sys.executable, str(SCANNER), "--root", str(root), "--json-out", str(report)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            data = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(data["audit_type"], "OFFLINE_HEURISTIC_STATIC_REVIEW")
            self.assertIn("limitations", data)


if __name__ == "__main__":
    unittest.main()
