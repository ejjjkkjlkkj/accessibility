import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts" / "validate_security_contract.py"
MANIFEST = ROOT / "agents" / "manifest.json"


class SecurityContractTests(unittest.TestCase):
    def test_manifest_validates(self):
        result = subprocess.run(
            [sys.executable, str(VALIDATOR)],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
            timeout=10,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("PASS: valid contract", result.stdout)
        self.assertIn("does not run agents", result.stdout)

    def test_agent_ids_are_unique(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        ids = [agent["id"] for agent in data["agents"]]
        self.assertEqual(len(ids), len(set(ids)))

    def test_offensive_scope_is_explicitly_authorized(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        rules = data["operating_rules"]
        self.assertEqual(rules["external_targets"], "PROHIBITED")
        self.assertEqual(rules["destructive_actions"], "PROHIBITED")

    def test_simulation_is_not_physical_proof(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.assertIn("physical_hardware_proof", data["release_gate"])
        self.assertIn("not", data["operating_rules"]["no_false_claims"].lower())

    def run_validator_with_data(self, data):
        return self.run_validator_with_raw(json.dumps(data))

    def run_validator_with_raw(self, raw):
        with tempfile.TemporaryDirectory() as temp:
            manifest = Path(temp) / "manifest.json"
            manifest.write_text(raw, encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(VALIDATOR), "--manifest", str(manifest)],
                text=True, capture_output=True, check=False, timeout=10,
            )

    def test_rejects_duplicate_json_object_keys(self):
        raw = MANIFEST.read_text(encoding="utf-8")
        raw = raw.replace(
            '"schema_version": "1.0.0",',
            '"schema_version": "1.0.0", "schema_version": "1.0.0",',
            1,
        )
        result = self.run_validator_with_raw(raw)
        self.assertEqual(result.returncode, 1)
        self.assertIn("duplicate JSON object key: schema_version", result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_rejects_nonstandard_json_constants(self):
        raw = MANIFEST.read_text(encoding="utf-8").replace(
            '"schema_version": "1.0.0"',
            '"schema_version": NaN',
            1,
        )
        result = self.run_validator_with_raw(raw)
        self.assertEqual(result.returncode, 1)
        self.assertIn("non-standard JSON constant: NaN", result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_rejects_oversized_manifest(self):
        with tempfile.TemporaryDirectory() as temp:
            manifest = Path(temp) / "manifest.json"
            manifest.write_bytes(b" " * (1024 * 1024 + 1))
            result = subprocess.run(
                [sys.executable, str(VALIDATOR), "--manifest", str(manifest)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 1)
            self.assertIn("manifest exceeds 1048576 bytes", result.stderr)
            self.assertNotIn("Traceback", result.stderr)

    def test_rejects_manifest_symlink(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            target = root / "real.json"
            target.write_text(MANIFEST.read_text(encoding="utf-8"), encoding="utf-8")
            link = root / "linked.json"
            try:
                link.symlink_to(target)
            except (OSError, NotImplementedError):
                self.skipTest("file symlinks unavailable")
            result = subprocess.run(
                [sys.executable, str(VALIDATOR), "--manifest", str(link)],
                text=True, capture_output=True, check=False, timeout=10,
            )
            self.assertEqual(result.returncode, 1)
            self.assertIn("manifest must not be a symbolic link", result.stderr)
            self.assertNotIn("Traceback", result.stderr)

    def test_rejects_non_object_manifest_without_traceback(self):
        result = self.run_validator_with_data([])
        self.assertEqual(result.returncode, 1)
        self.assertIn("manifest root must be a JSON object", result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_rejects_unhashable_agent_id_without_traceback(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        data["agents"][0]["id"] = ["DARK-LEAD"]
        result = self.run_validator_with_data(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("id must be a non-empty string", result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_rejects_non_string_capability(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        data["agents"][0]["capabilities"][0] = 17
        result = self.run_validator_with_data(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("capabilities entries must be non-empty strings", result.stderr)

    def test_rejects_unhashable_release_status_without_traceback(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        data["release_gate"]["required_statuses"] = [["PASS"]]
        result = self.run_validator_with_data(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("required_statuses must be an array of strings", result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_rejects_boolean_runtime_limit(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        data["agents"][3]["limits"]["max_runtime_seconds"] = True
        result = self.run_validator_with_data(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("max_runtime_seconds must be a positive integer", result.stderr)

    def test_rejects_missing_required_agent_role(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        data["agents"] = [agent for agent in data["agents"] if agent["id"] != "DARK-FUZZ"]
        result = self.run_validator_with_data(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("missing required agent roles: DARK-FUZZ", result.stderr)

    def test_rejects_removal_of_mandatory_release_blocker(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        data["release_gate"]["block_on"].remove("failed_ci")
        result = self.run_validator_with_data(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("missing mandatory blockers: failed_ci", result.stderr)

    def test_rejects_removal_of_mandatory_pass_evidence(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        data["release_gate"]["pass_requires"].remove("recorded_exit_code")
        result = self.run_validator_with_data(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("missing mandatory evidence: recorded_exit_code", result.stderr)

    def test_rejects_network_enabled_fuzz_role(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        data["agents"][3]["limits"]["network"] = True
        result = self.run_validator_with_data(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("DARK-FUZZ limits.network must be false", result.stderr)

    def test_rejects_fuzz_runtime_over_limit(self):
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
        data["agents"][3]["limits"]["max_runtime_seconds"] = 121
        result = self.run_validator_with_data(data)
        self.assertEqual(result.returncode, 1)
        self.assertIn("must be between 1 and 120", result.stderr)


if __name__ == "__main__":
    unittest.main()
