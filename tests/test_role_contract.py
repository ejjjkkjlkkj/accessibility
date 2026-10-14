import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CANONICAL = ROOT / "data" / "accessibility-role-codes-v1.json"


def snake(name):
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


def role_map(path, pattern, group_name=1, group_code=2, normalizer=snake):
    text = (ROOT / path).read_text(encoding="utf-8")
    found = {}
    for match in re.finditer(pattern, text, re.MULTILINE):
        name = normalizer(match.group(group_name))
        code = int(match.group(group_code))
        if name in found:
            raise AssertionError(f"{path}: duplicate role {name}")
        found[name] = code
    return found


class AccessibilityRoleContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        data = json.loads(CANONICAL.read_text(encoding="utf-8"))
        cls.expected = {entry["name"]: entry["code"] for entry in data["roles"]}
        cls.limit = data["maximum_name_utf8_bytes"]
        cls.contract = json.loads((ROOT / "projects/nexora/accessibility-contract.json").read_text(encoding="utf-8"))

    def test_canonical_contract_is_sequential_and_unique(self):
        self.assertEqual(list(self.expected.values()), list(range(18)))
        self.assertEqual(len(self.expected), len(set(self.expected.values())))
        self.assertEqual(self.contract["node"]["role_codes"], self.expected)
        self.assertEqual(self.contract["limits"]["maximum_name_utf8_bytes"], self.limit)

    def test_c_role_codes(self):
        self.assertEqual(role_map("components/core-c/include/accessibility_event.h",
            r"ACCESSIBILITY_ROLE_([A-Z_]+)\\s*=\\s*(\\d+)", normalizer=str.lower), self.expected)

    def test_rust_role_codes(self):
        self.assertEqual(role_map("crates/accessibility-core/src/lib.rs",
            r"\\b([A-Z][A-Za-z]+)\\s*=\\s*(\\d+)", normalizer=snake), self.expected)

    def test_go_role_codes(self):
        self.assertEqual(role_map("internal/accessibility/role.go",
            r"Role([A-Z][A-Za-z]*)\\s+Role\\s*=\\s*(\\d+)", normalizer=snake), self.expected)

    def test_d_role_codes(self):
        self.assertEqual(role_map("components/core-d/src/accessibility_event.d",
            r"^\\s*([A-Za-z][A-Za-z0-9]*)\\s*=\\s*(\\d+),?$", normalizer=snake), self.expected)

    def test_swift_role_codes(self):
        self.assertEqual(role_map("components/core-swift/Sources/AccessibilityCore/AccessibilityEvent.swift",
            r"case\\s+([A-Za-z][A-Za-z0-9]*)\\s*=\\s*(\\d+)", normalizer=snake), self.expected)

    def test_zig_role_codes(self):
        self.assertEqual(role_map("components/core-zig/src/accessibility_event.zig",
            r"^\\s*([a-z][a-z_]*)\\s*=\\s*(\\d+),?$", normalizer=str.lower), self.expected)

    def test_nim_role_codes(self):
        self.assertEqual(role_map("components/core-nim/src/accessibility_event.nim",
            r"\\ber([A-Z][A-Za-z]+)\\s*=\\s*(\\d+)", normalizer=snake), self.expected)

    def test_ada_role_codes(self):
        found = role_map("components/core-ada/accessibility_event.ads",
            r"([A-Za-z_]+)_Role\\s*=>\\s*(\\d+)", normalizer=lambda value: snake(value))
        self.assertEqual(found, self.expected)

    def test_shared_name_limit_is_enforced(self):
        c_header = (ROOT / "components/core-c/include/accessibility_event.h").read_text(encoding="utf-8")
        self.assertIn("ACCESSIBILITY_NAME_CAPACITY 4097u", c_header)
        for path in [
            "components/core-d/src/accessibility_event.d",
            "components/core-nim/src/accessibility_event.nim",
            "components/core-swift/Sources/AccessibilityCore/AccessibilityEvent.swift",
            "components/core-zig/src/accessibility_event.zig",
            "crates/accessibility-core/src/lib.rs",
        ]:
            self.assertIn("4096", (ROOT / path).read_text(encoding="utf-8"), path)


if __name__ == "__main__":
    unittest.main()
