import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import aerl
import aerl_tool


class AerlRunTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.ledger = Path(self.temp.name) / "events.aerl"

    def run_cli(self, *args, env=None):
        return subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "aerl_run.py"), "--file", str(self.ledger),
             "--kind", "test.wrapper", *args],
            text=True, capture_output=True, check=False, timeout=10, env=env,
        )

    def test_records_pass_without_recording_command_text(self):
        result = self.run_cli("--", sys.executable, "-c", "print('ok')")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        records, _ = aerl_tool.load_records(self.ledger)
        self.assertEqual(len(records), 1)
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

    def test_timeout_is_recorded_as_blocked_and_returns_124(self):
        result = self.run_cli(
            "--timeout-seconds", "1", "--", sys.executable, "-c",
            "import time; time.sleep(5)",
        )
        self.assertEqual(result.returncode, 124, result.stdout + result.stderr)
        records, _ = aerl_tool.load_records(self.ledger)
        self.assertEqual(len(records), 1)
        self.assertEqual(records[0]["status"], "BLOCKED")
        self.assertEqual(records[0]["payload"]["exit_code"], 124)
        self.assertEqual(records[0]["payload"]["launch_error_type"], "timeout")

    def test_launch_failure_is_recorded_without_leaking_command(self):
        result = self.run_cli("--", str(Path(self.ledger.parent) / "missing-executable"))
        self.assertEqual(result.returncode, 127)
        records, _ = aerl_tool.load_records(self.ledger)
        self.assertEqual(records[0]["status"], "FAIL")
        self.assertEqual(records[0]["payload"]["launch_error_type"], "FileNotFoundError")
        self.assertNotIn("missing-executable", json.dumps(records[0]))

    def test_ci_correlation_metadata_is_recorded_without_environment_dump(self):
        env = dict(os.environ)
        env.update({
            "GITHUB_SHA": "abc123",
            "GITHUB_RUN_ID": "98765",
            "GITHUB_JOB": "unit-tests",
            "SECRET_TOKEN": "must-not-be-recorded",
        })
        result = self.run_cli("--", sys.executable, "-c", "pass", env=env)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        records, _ = aerl_tool.load_records(self.ledger)
        ci = records[0]["payload"]["ci"]
        self.assertEqual(ci["commit_sha"], "abc123")
        self.assertEqual(ci["run_id"], "98765")
        self.assertEqual(ci["job"], "unit-tests")
        self.assertNotIn("SECRET_TOKEN", json.dumps(records[0]))
        self.assertNotIn("must-not-be-recorded", json.dumps(records[0]))

    def test_default_command_timeout_is_bounded(self):
        from aerl_run import build_parser

        args = build_parser().parse_args([
            "--file", str(self.ledger),
            "--kind", "test.timeout",
            "--", sys.executable, "-c", "pass",
        ])
        self.assertEqual(args.timeout_seconds, 300)

    def test_rejects_invalid_timeout(self):
        result = self.run_cli("--timeout-seconds", "86401", "--", sys.executable, "-c", "pass")
        self.assertEqual(result.returncode, 2)
        self.assertFalse(self.ledger.exists())


if __name__ == "__main__":
    unittest.main()
