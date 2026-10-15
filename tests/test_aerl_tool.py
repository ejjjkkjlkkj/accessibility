import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import aerl
import aerl_tool


class AerlToolTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.ledger = self.root / "events.aerl"
        aerl.append_event(self.ledger, "CI", "test.completed", "PASS", {"suite": "unit"})
        aerl.append_event(self.ledger, "HAKER", "security.scan", "FAIL", {"finding_id": "F-001"})
        aerl.append_event(self.ledger, "CI", "test.completed", "PASS", {"suite": "integration"})

    def test_summary_counts_and_attention_records(self):
        records, head = aerl_tool.load_records(self.ledger)
        report = aerl_tool.summary(records, head, self.ledger)
        self.assertEqual(report["records"], 3)
        self.assertEqual(report["status_counts"]["PASS"], 2)
        self.assertEqual(report["status_counts"]["FAIL"], 1)
        self.assertEqual(report["attention_records"][0]["kind"], "security.scan")

    def test_search_filters_only_verified_records(self):
        records, _ = aerl_tool.load_records(self.ledger)
        matches = [r for r in records if r["actor"] == "CI" and r["kind"] == "test.completed"]
        self.assertEqual(len(matches), 2)

    def test_corrupt_ledger_is_rejected_before_reporting(self):
        self.ledger.write_bytes(self.ledger.read_bytes().replace(b'"FAIL"', b'"PASS"', 1))
        with self.assertRaisesRegex(aerl.AerlError, "hash mismatch"):
            aerl_tool.load_records(self.ledger)

    def test_report_output_refuses_overwrite(self):
        out = self.root / "report.json"
        out.write_text("keep", encoding="utf-8")
        with self.assertRaises(FileExistsError):
            aerl_tool.emit({"status": "PASS"}, out)
        self.assertEqual(out.read_text(encoding="utf-8"), "keep")


if __name__ == "__main__":
    unittest.main()
