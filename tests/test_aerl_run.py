import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import aerl
import aerl_run


class AerlRunTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.ledger = Path(self.temp.name) / "events.aerl"

    def run_cli(self, *args):
        return subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "aerl_run.py"), "--file", str(self.ledger),
             "--kind", "test.wrapper", *args],
            text=True, capture_output=True, check=False, timeout=10,
        )

    def test_records_pass_without_recording_command_text(self):
        result = self.run_cli("--", sys.executable, "-c", "print('ok')")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        records, _ = aerl_run.load_records(self.ledger) if hasattr(aerl_run, "load_records") else ([], "")
        if not records:
            raw = self.ledger.read_bytes()
            count, _ = aerl.verify_bytes(raw)
            records = [aerl.strict_json(line) for line in raw[:-1].split(b"\\n")]
            self.assertEqual(count, 1)
        self.assertEqual(records[0]["status"], "PASS")
        self.assertEqual(records[0]["payload"]["exit_code"], 0)
        self.assertNotIn("print('ok')", json.dumps(records[0]))

    def test_records_failure_and_preserves_exit_code(self):
        result = self.run_cli("--", sys.executable, "-c", "raise SystemExit(7)")
        self.assertEqual(result.returncode, 7)
        raw = self.ledger.read_bytes()
        aerl.verify_bytes(raw)
        record = aerl.strict_json(raw[:-1])
        self.assertEqual(record["status"], "FAIL")
        self.assertEqual(record["payload"]["exit_code"], 7)

    def test_refuses_missing_command(self):
        result = self.run_cli()
        self.assertEqual(result.returncode, 2)
        self.assertFalse(self.ledger.exists())

    def test_rejects_invalid_timeout(self):
        result = self.run_cli("--timeout-seconds", "86401", "--", sys.executable, "-c", "pass")
        self.assertEqual(result.returncode, 2)
        self.assertFalse(self.ledger.exists())


if __name__ == "__main__":
    unittest.main()
