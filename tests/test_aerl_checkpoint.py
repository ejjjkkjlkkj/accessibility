import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import aerl  # noqa: E402
import aerl_checkpoint  # noqa: E402


class AerlCheckpointTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.ledger = self.root / "events.aerl"
        self.checkpoint = self.root / "checkpoint.json"
        aerl.append_event(self.ledger, "QA", "test.completed", "PASS", {"case": "baseline"})

    def run_cli(self, *args):
        return subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "aerl_checkpoint.py"), *map(str, args)],
            text=True, capture_output=True, check=False, timeout=10,
        )

    def test_create_and_verify_checkpoint(self):
        created = self.run_cli("create", "--ledger", self.ledger, "--out", self.checkpoint)
        self.assertEqual(created.returncode, 0, created.stdout + created.stderr)
        data = json.loads(self.checkpoint.read_text(encoding="utf-8"))
        self.assertEqual(data["sequence"], 1)
        verified = self.run_cli("verify", "--ledger", self.ledger, "--checkpoint", self.checkpoint)
        self.assertEqual(verified.returncode, 0, verified.stdout + verified.stderr)
        self.assertIn("checkpoint-verified", verified.stdout)

    def test_checkpoint_refuses_overwrite(self):
        self.checkpoint.write_text("keep", encoding="utf-8")
        result = self.run_cli("create", "--ledger", self.ledger, "--out", self.checkpoint)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(self.checkpoint.read_text(encoding="utf-8"), "keep")

    def test_checkpoint_detects_tail_truncation(self):
        aerl.append_event(self.ledger, "QA", "test.second", "PASS", {})
        self.assertEqual(self.run_cli("create", "--ledger", self.ledger, "--out", self.checkpoint).returncode, 0)
        rows = self.ledger.read_bytes().splitlines(keepends=True)
        self.ledger.write_bytes(rows[0])
        result = self.run_cli("verify", "--ledger", self.ledger, "--checkpoint", self.checkpoint)
        self.assertEqual(result.returncode, 2)
        self.assertIn("checkpoint", result.stderr)

    def test_rejects_unknown_checkpoint_fields(self):
        self.checkpoint.write_text(json.dumps({
            "schema": aerl_checkpoint.CHECKPOINT_SCHEMA, "sequence": 1,
            "head_hash": aerl.verify_file(self.ledger)[1], "extra": True
        }, sort_keys=True, separators=(",", ":")), encoding="utf-8")
        with self.assertRaisesRegex(aerl.AerlError, "required schema"):
            aerl_checkpoint.read_checkpoint(self.checkpoint)

    def test_rejects_symlink_checkpoint(self):
        try:
            self.checkpoint.symlink_to(self.ledger)
        except (OSError, NotImplementedError):
            self.skipTest("file symlinks unavailable")
        with self.assertRaisesRegex(aerl.AerlError, "symbolic link"):
            aerl_checkpoint.read_checkpoint(self.checkpoint)

    def test_rejects_invalid_checkpoint_hash(self):
        self.checkpoint.write_text(
            '{"head_hash":"bad","schema":"AERL-CHECKPOINT/1","sequence":1}',
            encoding="utf-8",
        )
        with self.assertRaisesRegex(aerl.AerlError, "64 lowercase hexadecimal"):
            aerl_checkpoint.read_checkpoint(self.checkpoint)


if __name__ == "__main__":
    unittest.main()
