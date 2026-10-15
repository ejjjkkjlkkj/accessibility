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
            r"ACCESSIBILITY_ROLE_([A-Z_]+)\s*=\s*(\d+)", normalizer=str.lower), self.expected)

    def test_rust_role_codes(self):
        self.assertEqual(role_map("crates/accessibility-core/src/lib.rs",
            r"\b([A-Z][A-Za-z]+)\s*=\s*(\d+)", normalizer=snake), self.expected)

    def test_go_role_codes(self):
        self.assertEqual(role_map("internal/accessibility/role.go",
            r"Role([A-Z][A-Za-z]*)\s+Role\s*=\s*(\d+)", normalizer=snake), self.expected)

    def test_d_role_codes(self):
        self.assertEqual(role_map("components/core-d/src/accessibility_event.d",
            r"^\s*([A-Za-z][A-Za-z0-9]*)\s*=\s*(\d+),?$", normalizer=snake), self.expected)

    def test_swift_role_codes(self):
        self.assertEqual(role_map("components/core-swift/Sources/AccessibilityCore/AccessibilityEvent.swift",
            r"case\s+([A-Za-z][A-Za-z0-9]*)\s*=\s*(\d+)", normalizer=snake), self.expected)

    def test_zig_role_codes(self):
        self.assertEqual(role_map("components/core-zig/src/accessibility_event.zig",
            r"^\s*([a-z][a-z_]*)\s*=\s*(\d+),?$", normalizer=str.lower), self.expected)

    def test_nim_role_codes(self):
        self.assertEqual(role_map("components/core-nim/src/accessibility_event.nim",
            r"\ber([A-Z][A-Za-z]+)\s*=\s*(\d+)", normalizer=snake), self.expected)

    def test_ada_role_codes(self):
        found = role_map("components/core-ada/accessibility_event.ads",
            r"([A-Za-z_]+)_Role\s*=>\s*(\d+)", normalizer=lambda value: value.lower())
        self.assertEqual(found, self.expected)


    def test_semantic_state_flag_masks_match_canonical_contract(self):
        contract_path = ROOT / "data" / "accessibility-state-flags-v1.json"
        state_contract = json.loads(contract_path.read_text(encoding="utf-8"))
        expected = {item["name"]: item["mask"] for item in state_contract["flags"]}
        self.assertEqual(state_contract["schema_version"], 1)
        self.assertEqual(state_contract["known_mask"], sum(expected.values()))
        self.assertEqual(expected, {"focused": 1, "focusable": 2, "enabled": 4, "disabled": 8})

        def shifted_map(path, pattern, normalize=str.lower):
            source = (ROOT / path).read_text(encoding="utf-8")
            result = {}
            for name, bit in re.findall(pattern, source, re.MULTILINE):
                key = normalize(name)
                if key in result:
                    self.fail(f"{path}: duplicate state flag {key}")
                result[key] = 1 << int(bit)
            return result

        shift_patterns = {
            "C": ("components/core-c/include/accessibility_event.h",
                  r"#define ACCESSIBILITY_STATE_([A-Z]+) \(UINT64_C\(1\) << (\d+)\)"),
            "Rust": ("crates/accessibility-core/src/tree.rs",
                     r"pub const STATE_([A-Z]+): u64 = 1 << (\d+);"),
            "D": ("components/core-d/src/accessibility_event.d",
                  r"enum ulong STATE_([A-Z]+) = 1UL << (\d+);"),
            "Nim": ("components/core-nim/src/accessibility_event.nim",
                    r"STATE_([A-Z]+)\* = 1'u64 shl (\d+)"),
            "Swift": ("components/core-swift/Sources/AccessibilityCore/AccessibilityEvent.swift",
                      r"public static let ([a-z]+): UInt64 = 1 << (\d+)"),
            "Zig": ("components/core-zig/src/accessibility_event.zig",
                    r"pub const STATE_([A-Z]+): u64 = 1 << (\d+);"),
        }
        for language, (path, pattern) in shift_patterns.items():
            with self.subTest(language=language):
                self.assertEqual(shifted_map(path, pattern), expected, path)

        go_source = (ROOT / "internal/accessibility/role.go").read_text(encoding="utf-8")
        go_block = re.search(
            r"const\s*\(\s*StateFocused StateFlags = 1 << iota(.*?)\)",
            go_source,
            re.DOTALL,
        )
        self.assertIsNotNone(go_block, "Go state flag iota block missing")
        self.assertEqual(
            re.findall(r"^\s*(StateFocusable|StateEnabled|StateDisabled)\s*$", go_block.group(1), re.MULTILINE),
            ["StateFocusable", "StateEnabled", "StateDisabled"],
        )
        self.assertEqual(
            {"focused": 1, "focusable": 2, "enabled": 4, "disabled": 8},
            expected,
        )

        ada_source = (ROOT / "components/core-ada/accessibility_event.ads").read_text(encoding="utf-8")
        ada_map = {
            name.lower(): int(mask)
            for name, mask in re.findall(
                r"State_(Focused|Focusable|Enabled|Disabled)\s*:\s*constant Interfaces.Unsigned_64 := (\d+);",
                ada_source,
            )
        }
        self.assertEqual(ada_map, expected)
        for path, marker in [
            ("components/core-ada/accessibility_event.ads", "State_Known_Mask : constant Interfaces.Unsigned_64 := 15"),
            ("components/core-nim/src/accessibility_event.nim", "STATE_KNOWN_MASK* = 0x0F'u64"),
            ("components/core-zig/src/accessibility_event.zig", "STATE_KNOWN_MASK: u64 = 0x0F"),
            ("components/core-c/include/accessibility_event.h", "ACCESSIBILITY_STATE_KNOWN_MASK UINT64_C(0x0F)"),
        ]:
            self.assertIn(marker, (ROOT / path).read_text(encoding="utf-8"), path)

    def test_semantic_state_flag_contract_is_implemented_across_languages(self):
        implementations = {
            "C": ("components/core-c/include/accessibility_event.h", "ACCESSIBILITY_STATE_FOCUSED", "accessibility_event_init_n"),
            "Rust": ("crates/accessibility-core/src/tree.rs", "STATE_FOCUSED", "state_flags_are_valid"),
            "Go": ("internal/accessibility/role.go", "type Event struct", "func (e Event) Valid()"),
            "D": ("components/core-d/src/accessibility_event.d", "STATE_FOCUSED", "isValidStateFlags"),
            "Ada": ("components/core-ada/accessibility_event.ads", "State_Focused", "State_Flags_Valid"),
            "Nim": ("components/core-nim/src/accessibility_event.nim", "STATE_FOCUSED", "isValidStateFlags"),
            "Swift": ("components/core-swift/Sources/AccessibilityCore/AccessibilityEvent.swift", "AccessibilityState", "stateFlags"),
            "Zig": ("components/core-zig/src/accessibility_event.zig", "STATE_FOCUSED", "stateFlagsAreValid"),
        }
        for language, (path, state_marker, validator_marker) in implementations.items():
            text = (ROOT / path).read_text(encoding="utf-8")
            with self.subTest(language=language):
                self.assertIn(state_marker, text)
                self.assertIn(validator_marker, text)

    def test_shared_name_validation_hooks_exist_across_bindings(self):
        requirements = {
            "C": ("components/core-c/src/accessibility_event.c", [
                "accessibility_utf8_valid", "memchr(name, '\\0', name_length)",
            ]),
            "Rust": ("crates/accessibility-core/src/lib.rs", [
                "self.name.contains('\\0')",
            ]),
            "Go": ("internal/accessibility/role.go", [
                "utf8.ValidString(e.Name)", "strings.ContainsRune(e.Name, 0)",
            ]),
            "D": ("components/core-d/src/accessibility_event.d", [
                "bool isValidName(string name)", "validate(name)", "byteValue == '\\0'",
            ]),
            "Nim": ("components/core-nim/src/accessibility_event.nim", [
                "proc isValidName*", "validateUtf8(name) == -1", "name.find('\\0')",
            ]),
            "Swift": ("components/core-swift/Sources/AccessibilityCore/AccessibilityEvent.swift", [
                'name.contains("\\0")',
            ]),
            "Zig": ("components/core-zig/src/accessibility_event.zig", [
                "utf8ValidateSlice(name)", "byte == 0",
            ]),
        }
        for language, (path, markers) in requirements.items():
            source = (ROOT / path).read_text(encoding="utf-8")
            with self.subTest(language=language):
                for marker in markers:
                    self.assertIn(marker, source, f"{path}: missing {marker}")

    def test_shared_name_limit_is_enforced(self):
        c_header = (ROOT / "components/core-c/include/accessibility_event.h").read_text(encoding="utf-8")
        self.assertIn("ACCESSIBILITY_NAME_CAPACITY 4097u", c_header)
        for path in [
            "components/core-d/src/accessibility_event.d",
            "components/core-nim/src/accessibility_event.nim",
            "components/core-swift/Sources/AccessibilityCore/AccessibilityEvent.swift",
            "components/core-zig/src/accessibility_event.zig",
            "crates/accessibility-core/src/tree.rs",
            "components/core-ada/accessibility_event.adb",
        ]:
            self.assertIn("4096", (ROOT / path).read_text(encoding="utf-8"), path)


if __name__ == "__main__":
    unittest.main()
