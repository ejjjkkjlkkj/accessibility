# Autonomous Engineering Contract — Accessibility / Bare-Metal OS

This file is the task contract for any coding agent operating in this repository. It does not itself start an agent or grant execution credentials.

## Mission
Drive the accessibility stack and standalone OS toward a reproducible release that can boot and operate on the target physical PC without requiring Windows or Linux at runtime. Host OS tools may be used for development, but the final product must not depend on a host OS at runtime.

## Required work loop
1. Inspect the current branch, working tree, open failures, tests, and latest CI evidence before editing.
2. Pick the highest-impact, smallest independently verifiable blocker.
3. Implement real functionality and regression tests; do not substitute a mock for hardware behavior without labeling it.
4. Run focused tests, then the relevant full suite, formatting, static analysis, and reproducible-build checks.
5. For each capability, label evidence PASS, FAIL, PARTIAL, BLOCKED, or NOT_VALIDATED and attach exact commands/logs/artifact hashes.
6. Repeat while the execution budget remains. Record the next concrete task and blocker when execution stops.

## Product requirements
- Keep the boot path, keyboard navigation, accessible semantic output, and basic speech usable without the optional AI model or cloud credentials.
- Build and test the UEFI/bare-metal path, screen-reader navigation, focus/role/state semantics, speech pipeline, device discovery, and OS/runtime components in separate verifiable layers.
- Maintain explicit per-language implementation status; a directory or interface is not proof of a complete port.
- Keep AI integration behind a versioned contract with `ejjjkkjlkkj/ia`; validate malformed input, limits, timeouts, and graceful fallback.
- Prioritize defensive security: fuzz parsers and protocol boundaries, test privilege boundaries, dependency provenance, and fail-safe recovery only in owned/authorized test targets.
- Never claim physical audio, keyboard, firmware, or boot success from unit tests or QEMU alone. Physical evidence must identify the actual hardware, firmware mode, build hash, test procedure, and observed result.

## Release gates
- Required CI and regression tests pass from a clean checkout.
- Reproducible build produces versioned artifacts with SHA-256 hashes and build logs.
- Boot, keyboard, accessible navigation, speech output, recovery, and installation have explicit evidence and known limitations.
- Physical-PC validation is a separate mandatory gate for hardware claims; simulator results remain SIMULATED.
- No final OS release is declared while a required gate is BLOCKED, PARTIAL, or NOT_VALIDATED.

## Permissions and integrity
- No automatic BIOS/UEFI firmware flashing, NVRAM changes, partition changes, or reboot.
- Do not expose secrets, weaken security checks to make CI green, fabricate test output, or silently skip failing tests.
- Do not force-push, delete branches, publish a release, or merge unreviewed changes automatically.
- Preserve unrelated user work and prefer focused, reviewable changes.
