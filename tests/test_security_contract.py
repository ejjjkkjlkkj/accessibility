import json
import subprocess
import sys
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


if __name__ == "__main__":
    unittest.main()
