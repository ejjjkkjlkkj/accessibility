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


    def test_collect_junit_metadata_and_skip_duplicate(self):
        reports = self.root / "reports"
        reports.mkdir()
        report = reports / "junit-results.xml"
        report.write_text(
            '<testsuites><testsuite tests="4" failures="1" errors="0" skipped="1"/></testsuites>',
            encoding="utf-8",
        )
        first = aerl_tool.collect(self.root, self.ledger)
        self.assertEqual(first["events_appended"], 1)
        self.assertEqual(first["status"], "PASS")
        records, _ = aerl_tool.load_records(self.ledger)
        collected = records[-1]
        self.assertEqual(collected["status"], "FAIL")
        self.assertEqual(collected["payload"]["tests"], 4)
        self.assertEqual(collected["payload"]["failures"], 1)
        self.assertNotIn("raw", collected["payload"])
        second = aerl_tool.collect(self.root, self.ledger)
        self.assertEqual(second["events_appended"], 0)
        self.assertEqual(second["duplicates_skipped"], 1)

    def test_collect_allowlists_report_names_and_hashes_content(self):
        (self.root / "ordinary.json").write_text('{"status":"FAIL"}', encoding="utf-8")
        report = self.root / "haker-audit-report.json"
        report.write_text('{"status":"PASS","tests":7,"failed":0,"token_value":"not-copied"}', encoding="utf-8")
        result = aerl_tool.collect(self.root, self.ledger)
        self.assertEqual(result["events_appended"], 1)
        records, _ = aerl_tool.load_records(self.ledger)
        payload = records[-1]["payload"]
        self.assertEqual(payload["counts"]["tests"], 7)
        self.assertNotIn("token_value", payload)
        self.assertEqual(len(payload["source_sha256"]), 64)
        self.assertFalse(any("ordinary.json" == r["payload"].get("source_path") for r in records))

    def test_unified_cli_initializes_and_appends(self):
        ledger = self.root / "new.aerl"
        init = aerl_tool.main(["init", "--file", str(ledger)])
        self.assertEqual(init, 0)
        append = aerl_tool.main([
            "append", "--file", str(ledger), "--actor", "QA",
            "--kind", "manual.check", "--status", "PASS", "--payload", '{"suite":"smoke"}',
        ])
        self.assertEqual(append, 0)
        records, _ = aerl_tool.load_records(ledger)
        self.assertEqual(len(records), 1)


    def test_haker_heuristic_report_is_never_promoted_to_pass_or_copy_evidence(self):
        report = self.root / "haker-audit-report.json"
        report.write_text(json.dumps({
            "schema_version": "1.0",
            "audit_type": "OFFLINE_HEURISTIC_STATIC_REVIEW",
            "files_scanned": 23,
            "unreadable_files": [],
            "finding_count": 1,
            "severity_counts": {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 0, "LOW": 0, "INFO": 0},
            "findings": [{"evidence": "SENSITIVE-RAW-LINE-MUST-NOT-BE-COPIED"}],
        }), encoding="utf-8")
        result = aerl_tool.collect(self.root, self.ledger)
        self.assertEqual(result["events_appended"], 1)
        records, _ = aerl_tool.load_records(self.ledger)
        collected = records[-1]
        self.assertEqual(collected["kind"], "report.haker.audit")
        self.assertEqual(collected["status"], "UNPROVEN")
        self.assertEqual(collected["payload"]["counts"]["finding_count"], 1)
        self.assertEqual(collected["payload"]["counts"]["severity_high"], 1)
        self.assertNotIn("SENSITIVE-RAW-LINE-MUST-NOT-BE-COPIED", json.dumps(collected))

    def test_collect_refuses_symlinked_report(self):
        target = self.root / "real.json"
        target.write_text('{"status":"PASS"}', encoding="utf-8")
        link = self.root / "junit-results.xml"
        try:
            link.symlink_to(target)
        except (OSError, NotImplementedError):
            self.skipTest("file symlinks unavailable")
        result = aerl_tool.collect(self.root, self.ledger)
        self.assertEqual(result["status"], "PARTIAL")
        self.assertEqual(result["sources_refused"], 1)
        self.assertEqual(len(aerl_tool.load_records(self.ledger)[0]), 3)


if __name__ == "__main__":
    unittest.main()
