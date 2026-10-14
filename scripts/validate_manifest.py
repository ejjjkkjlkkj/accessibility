#!/usr/bin/env python3
"""Validate the language-track / component project manifest.

This checks bookkeeping only. It does not build code or prove runtime,
accessibility, UEFI, OS boot, audio, emulator, or physical-hardware behavior.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

EXPECTED_LANGUAGES = {"C", "Rust", "Go", "D", "Ada", "Zig", "Nim", "Swift", "C++", "Free Pascal", "Odin", "Fortran", "Objective-C"}
EXPECTED_COMPONENTS = {"screen_reader", "uefi_pre_os", "operating_system_kernel"}
ALLOWED_STATUSES = {
    "NOT_IMPLEMENTED", "NOT_VALIDATED", "PARTIAL", "BLOCKED",
    "PASS", "FAIL", "UNPROVEN",
}


def main() -> int:
    manifest_path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("projects/manifest.json")
    try:
        data = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"FAIL: cannot read manifest {manifest_path}: {exc}")
        return 2

    errors: list[str] = []
    languages = data.get("languages")
    if not isinstance(languages, list):
        print("FAIL: 'languages' must be a list")
        return 2

    names = [item.get("language") for item in languages if isinstance(item, dict)]
    if len(names) != len(languages):
        errors.append("every language entry must be an object")
    if len(names) != len(set(names)):
        errors.append("duplicate language entries found")
    if set(names) != EXPECTED_LANGUAGES:
        errors.append(
            f"language set mismatch: missing={sorted(EXPECTED_LANGUAGES - set(names))}; "
            f"unexpected={sorted(set(names) - EXPECTED_LANGUAGES)}"
        )

    count = 0
    for entry in languages:
        if not isinstance(entry, dict):
            continue
        language = entry.get("language", "<unknown>")
        components = entry.get("components")
        if not isinstance(components, dict):
            errors.append(f"{language}: components must be an object")
            continue
        if set(components) != EXPECTED_COMPONENTS:
            errors.append(
                f"{language}: component set mismatch; "
                f"missing={sorted(EXPECTED_COMPONENTS - set(components))}; "
                f"unexpected={sorted(set(components) - EXPECTED_COMPONENTS)}"
            )
        for component_name, component in components.items():
            count += 1
            if not isinstance(component, dict):
                errors.append(f"{language}/{component_name}: component must be an object")
                continue
            status = component.get("status")
            evidence = component.get("evidence")
            if status not in ALLOWED_STATUSES:
                errors.append(f"{language}/{component_name}: invalid status {status!r}")
            if not isinstance(evidence, str) or not evidence.strip():
                errors.append(f"{language}/{component_name}: evidence/limitation text is required")
            if status == "PASS" and (
                not isinstance(evidence, str)
                or not all(token in evidence.lower() for token in ("command", "environment"))
            ):
                errors.append(
                    f"{language}/{component_name}: PASS evidence must identify command and environment"
                )

    if count != len(EXPECTED_LANGUAGES) * len(EXPECTED_COMPONENTS):
        errors.append(f"expected {len(EXPECTED_LANGUAGES) * len(EXPECTED_COMPONENTS)} component records; found {count}")

    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        print(f"MANIFEST_VALIDATION=FAIL errors={len(errors)} component_records={count}")
        return 1

    print(f"PASS: {len(EXPECTED_LANGUAGES)} language tracks and {count} component records are structurally valid.")
    print("LIMITATION: manifest structure only; no build, runtime, UEFI, OS, speech, or hardware proof.")
    print(f"MANIFEST_VALIDATION=PASS errors=0 component_records={count}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
