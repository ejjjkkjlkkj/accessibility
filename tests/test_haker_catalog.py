import copy
import json
import unittest
from pathlib import Path

from tools.validate_haker_catalog import validate_catalog

CATALOG_PATH = Path(__file__).resolve().parents[1] / "ZERO" / "HAKER-CASES.json"


class HakerCatalogTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))

    def test_current_catalog_has_valid_structure(self):
        counts = validate_catalog(self.catalog)
        self.assertEqual(sum(counts.values()), len(self.catalog["cases"]))

    def test_scope_defaults_are_restricted(self):
        for key, bad_value in (
            ("network_tests_default", True),
            ("destructive_tests_default", True),
            ("require_explicit_scope_for_external_targets", False),
        ):
            candidate = copy.deepcopy(self.catalog)
            candidate["policy"][key] = bad_value
            with self.subTest(policy=key), self.assertRaises(ValueError):
                validate_catalog(candidate)

    def test_duplicate_ids_are_rejected(self):
        candidate = copy.deepcopy(self.catalog)
        candidate["cases"].append(copy.deepcopy(candidate["cases"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate case id"):
            validate_catalog(candidate)

    def test_missing_case_field_is_rejected(self):
        candidate = copy.deepcopy(self.catalog)
        del candidate["cases"][0]["expected"]
        with self.assertRaisesRegex(ValueError, "missing fields"):
            validate_catalog(candidate)

    def test_unknown_status_is_rejected(self):
        candidate = copy.deepcopy(self.catalog)
        candidate["cases"][0]["status"] = "MAGIC_PASS"
        with self.assertRaisesRegex(ValueError, "unsupported"):
            validate_catalog(candidate)

    def test_empty_catalog_is_rejected(self):
        candidate = copy.deepcopy(self.catalog)
        candidate["cases"] = []
        with self.assertRaisesRegex(ValueError, "non-empty"):
            validate_catalog(candidate)


if __name__ == "__main__":
    unittest.main()
