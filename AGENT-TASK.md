# Autonomous engineering queue — Accessibility / ZERO

This task file is the persistent work queue for the local Ollama agent team. Work only on the current dedicated feature branch. Each iteration must inspect the relevant implementation and tests, select one small, high-value item, implement it, add regression coverage, run the repository's real test suite, and report evidence through the JSONL log. The runner publishes only test-passing commits to this feature branch; it never merges or releases.

## Priority order

1. **Semantic accessibility core:** inspect `crates/accessibility-core/`; improve tree invariants, accessible roles/states, focus/cursor behavior, event semantics, malformed input handling, and bounded traversal with regression tests.
2. **Go implementation and catalog:** inspect `internal/accessibility/`, `internal/screenreader/`, and `cmd/catalog-validator/`; preserve stable contracts and reject invalid role/resource data predictably.
3. **Cross-language parity:** compare C, Ada, D, Nim, Swift, Zig, Rust, and Go implementations. Fix concrete differences with language-native tests; never mark an uncompiled language as validated.
4. **Resource database and project manifest:** validate schemas, duplicate/invalid entries, generated output determinism, and broken links.
5. **Accessibility behavior:** prioritize keyboard-first navigation, meaningful focus and announcements, non-AI fallback, predictable errors, and compatibility contracts.
6. **Security and release readiness:** add repository-local adversarial regression tests, dependency/secret hygiene checks, reproducible build evidence, and a truthful release-gate checklist.

## Specialist roles

- IMPLEMENTER: one correctness fix or small missing feature.
- TEST_ENGINEER: regression/boundary/property tests; do not weaken tests to get green CI.
- ACCESSIBILITY_REVIEWER: semantic roles/states, focus, keyboard-only operation, assistive-technology behavior, and non-AI fallback.
- SECURITY_RED_TEAM: local-only defensive tests using synthetic inputs; no external targets, credential access, exploit deployment, or destructive actions.
- INDEPENDENT_REVIEWER: check assumptions, scope, determinism, resource limits, evidence, and changed-file diff.

## Hard gates

Never modify `.github/workflows/`, `AGENTS.md`, this queue, secrets/configuration, release artifacts, or the agent runner. No credential access, network beacons, persistence, destructive behavior, arbitrary command execution, firmware/NVRAM/partition operations, or reboot logic. Never present QEMU/simulation as physical proof. Label UEFI, hardware audio, and physical device interactions `NOT_VALIDATED` without actual physical evidence. Do not claim a test passed unless its exit code is zero. If no safe, testable improvement is apparent, return exactly `NO_PATCH`.
