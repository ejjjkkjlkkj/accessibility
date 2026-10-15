import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import aerl  # noqa: E402


class AerlTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.ledger = self.root / "events.aerl"

    def run_cli(self, *args, timeout=10):
        return subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "aerl.py"), *map(str, args)],
            text=True, capture_output=True, check=False, timeout=timeout,
        )

    def append(self, actor="QA", kind="test.completed", status="PASS", payload=None):
        return aerl.append_event(
            self.ledger, actor, kind, status, {} if payload is None else payload
        )

    def test_empty_ledger_has_stable_genesis(self):
        self.ledger.write_bytes(b"")
        self.assertEqual(aerl.verify_file(self.ledger), (0, aerl.ZERO_HASH))

    def test_appended_records_verify_as_hash_chain(self):
        seq1, head1 = self.append(payload={"check": "unit", "attempt": 1})
        seq2, head2 = self.append(payload={"check": "ci", "attempt": 2})
        self.assertEqual((seq1, seq2), (1, 2))
        self.assertNotEqual(head1, head2)
        self.assertEqual(aerl.verify_file(self.ledger), (2, head2))

    def test_tampered_payload_is_detected(self):
        self.append(payload={"check": "unit"})
        raw = self.ledger.read_bytes().replace(b'"unit"', b'"evil"', 1)
        self.ledger.write_bytes(raw)
        with self.assertRaisesRegex(aerl.AerlError, "hash mismatch"):
            aerl.verify_file(self.ledger)

    def test_deletion_or_reordering_breaks_chain(self):
        self.append(payload={"check": "first"})
        self.append(payload={"check": "second"})
        rows = self.ledger.read_bytes().splitlines()
        self.ledger.write_bytes(rows[1] + b"\n" + rows[0] + b"\n")
        with self.assertRaises(aerl.AerlError):
            aerl.verify_file(self.ledger)

    def test_external_checkpoint_detects_tail_truncation(self):
        self.append(payload={"check": "first"})
        _, checkpoint = self.append(payload={"check": "second"})
        rows = self.ledger.read_bytes().splitlines(keepends=True)
        self.ledger.write_bytes(rows[0])
        with self.assertRaisesRegex(aerl.AerlError, "checkpoint"):
            aerl.verify_file(self.ledger, expected_head=checkpoint, expected_sequence=2)

    def test_partial_final_record_is_rejected(self):
        self.append()
        self.ledger.write_bytes(self.ledger.read_bytes().rstrip(b"\n"))
        with self.assertRaisesRegex(aerl.AerlError, "partial final record"):
            aerl.verify_file(self.ledger)

    def test_noncanonical_json_is_rejected(self):
        self.append()
        self.ledger.write_bytes(self.ledger.read_bytes().replace(b',"seq":', b', "seq":', 1))
        with self.assertRaisesRegex(aerl.AerlError, "not canonical JSON"):
            aerl.verify_file(self.ledger)

    def test_append_fails_closed_on_corrupt_ledger_and_preserves_bytes(self):
        self.append()
        original = self.ledger.read_bytes().replace(b'"PASS"', b'"FAIL"', 1)
        self.ledger.write_bytes(original)
        with self.assertRaises(aerl.AerlError):
            self.append(kind="security.recheck")
        self.assertEqual(self.ledger.read_bytes(), original)

    def test_sensitive_payload_keys_are_refused_before_file_creation(self):
        for key in ("api_key", "private_key", "email"):
            with self.subTest(key=key):
                with self.assertRaisesRegex(aerl.AerlError, "sensitive-looking"):
                    self.append(payload={key: "must-not-be-logged"})
                self.assertFalse(self.ledger.exists())

    def test_nonstandard_number_and_duplicate_json_keys_are_refused(self):
        result = self.run_cli(
            "append", "--file", self.ledger, "--actor", "QA",
            "--kind", "test.completed", "--status", "PASS",
            "--payload", '{"value":NaN}',
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("non-standard JSON constant", result.stderr)
        duplicate = self.run_cli(
            "append", "--file", self.ledger, "--actor", "QA",
            "--kind", "test.completed", "--status", "PASS",
            "--payload", '{"value":1,"value":2}',
        )
        self.assertEqual(duplicate.returncode, 2)
        self.assertIn("duplicate JSON key", duplicate.stderr)
        self.assertFalse(self.ledger.exists())

    def test_symlink_input_is_refused(self):
        self.append()
        link = self.root / "link.aerl"
        try:
            link.symlink_to(self.ledger)
        except (OSError, NotImplementedError):
            self.skipTest("file symlinks unavailable")
        result = self.run_cli("verify", "--file", link)
        self.assertEqual(result.returncode, 2)
        self.assertIn("symbolic link", result.stderr)

    def test_fifo_input_is_refused_without_blocking(self):
        if not hasattr(os, "mkfifo"):
            self.skipTest("named pipes unavailable")
        fifo = self.root / "events.fifo"
        os.mkfifo(fifo)
        result = self.run_cli("verify", "--file", fifo, timeout=3)
        self.assertEqual(result.returncode, 2)
        self.assertIn("regular file", result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_cli_checkpoint_round_trip(self):
        first = self.run_cli(
            "append", "--file", self.ledger, "--actor", "WHITE",
            "--kind", "remediation.complete", "--status", "PASS",
            "--payload", '{"test":"regression"}',
        )
        self.assertEqual(first.returncode, 0, first.stdout + first.stderr)
        record = json.loads(first.stdout)
        verified = self.run_cli(
            "verify", "--file", self.ledger,
            "--expected-head", record["head_hash"], "--expected-sequence", "1",
        )
        self.assertEqual(verified.returncode, 0, verified.stdout + verified.stderr)
        self.assertEqual(json.loads(verified.stdout)["external_anchor"], "VERIFIED")

    def test_invalid_expected_checkpoint_is_rejected(self):
        self.append()
        result = self.run_cli("verify", "--file", self.ledger, "--expected-head", "0" * 63)
        self.assertEqual(result.returncode, 2)
        self.assertIn("64 lowercase hexadecimal", result.stderr)

    def test_oversized_payload_is_refused(self):
        with self.assertRaises(aerl.AerlError):
            self.append(payload={"blob": "x" * (aerl.MAX_PAYLOAD_BYTES + 1)})
        self.assertFalse(self.ledger.exists())


if __name__ == "__main__":
    unittest.main()
