# Project completion and release gates

## Mission

Deliver the accessibility stack and, if required by the project scope, a project-owned bootable operating system/runtime that can start directly on supported physical hardware without depending on Windows or Linux at runtime. Work iteratively until every release gate is evidenced as PASS or clearly BLOCKED with a concrete remediation. Do not claim the OS is finished while required hardware paths are simulated.

## Autonomous work loop

For each iteration:
1. Inspect repository state, build scripts, issues, TODO/FIXME markers, tests, and latest CI results.
2. Pick the highest-impact independently verifiable defect or missing capability.
3. Implement a small change with regression tests.
4. Run focused tests, then all relevant tests, formatters, linters, static analysis, and build checks available.
5. Fix failures without weakening tests or hiding diagnostics.
6. Review diffs, dependency changes, permissions, secrets, generated artifacts, and hardware side effects.
7. Commit focused changes on the feature branch and record exact commands, exit codes, evidence, and limitations.
8. Repeat until no safe, actionable task remains. Continue independent tasks when another task is BLOCKED.

Never report tests as passed unless they were executed and the output supports the claim.

## Technical priorities

1. Keyboard-first and screen-reader-first interaction: focus order, accessible roles/names/states, predictable announcements, navigation, settings, errors, and recovery.
2. Real adapters and clear interfaces for platform APIs; simulated adapters must be explicitly marked and cannot count as hardware proof.
3. UEFI boot path, memory map, allocation, timers, input, display/framebuffer, audio, storage, PCI/ACPI discovery, and error handling, each with tests and evidence.
4. Built-in voice and accessibility core with local inference where feasible; no mandatory Windows, Linux, NVDA, JAWS, cloud service, or hosted API dependency in the final bare-metal runtime.
5. Security engineering: threat model, attack-surface inventory, fuzzing, malformed-input tests, dependency review, privilege boundaries, and defensive checks. Red-team tests must use synthetic fixtures, local labs, or explicitly authorized targets.
6. Reproducible builds, signed/checksummed artifacts where feasible, installation/recovery documentation, and hardware validation reports.

## Required evidence separation

- UNIT: unit tests only.
- CI: code ran on a CI runner.
- EMULATOR: ran in an emulator; useful but not physical-hardware proof.
- PHYSICAL: executed on named physical hardware with firmware settings, device identifiers, exact steps, logs, and result.
- NOT_VALIDATED: no direct evidence yet.
- BLOCKED: a concrete missing prerequisite prevents the test.

QEMU/emulator success is not proof of physical hardware support. A generated PCM buffer is not proof of audible output. Register writes or mocked device responses are not proof of functioning hardware.

## Final OS and release gates

- [ ] A clean checkout builds from documented source and pinned toolchain/dependencies.
- [ ] Bootable artifact has recorded SHA-256, version, source commit, build log, and release notes.
- [ ] UEFI boot succeeds on each declared supported physical machine; emulator results are separately labelled.
- [ ] Keyboard input, display output, audio output, timers, memory allocation, and error recovery have direct evidence for every claimed device.
- [ ] The accessible interaction path is usable without Windows/Linux or an external screen reader at runtime, where this is the stated target.
- [ ] No required runtime component silently falls back to a hosted API or another operating system.
- [ ] Safety checks cover malformed firmware tables, invalid device data, bounds, allocation failures, and unavailable hardware.
- [ ] Tests, static analysis, dependency/license review, secret scan, and diff review are complete.
- [ ] Installation, boot-media creation, recovery, known limitations, and supported hardware are documented.
- [ ] Final artifact is cleanly rebuilt and smoke-tested; evidence is attached to the release candidate.

## Host OS versus target runtime

A host OS may be used as a development tool to compile, inspect, or package the project. The requirement “no Windows/Linux” applies to the final runtime only if the project can boot and provide its promised functionality directly on supported hardware. State all unavoidable firmware, UEFI, CPU, device, compiler, and third-party dependencies; do not claim “zero dependency” if any remain.

## Safety and change control

Do not flash BIOS/firmware, write NVRAM, reboot the user's machine, alter secure-boot settings, or perform destructive storage/device operations without separate explicit authorization. No automatic merge to the default branch or public release without authorization. Red-team activity is limited to owned, synthetic, or explicitly authorized targets; do not build unauthorized intrusion, stealth, persistence, or credential-theft features.

## Evidence labels

- PASS: executed successfully with exact command/result.
- FAIL: executed and failed; preserve details.
- PARTIAL: some criteria passed.
- BLOCKED: concrete missing prerequisite.
- NOT_VALIDATED: no direct evidence yet.
