# Development roadmap

## Milestone 0 — project integrity
- [x] Define eight language tracks and 24 component IDs.
- [x] Establish evidence labels and distinguish emulation from physical validation.
- [x] Add portable focus-announcement cores in C, Rust, Go, D, Ada, Zig, Nim, and Swift.
- [x] Add starter unit tests for all eight portable cores.
- [x] Add the C EDK2 and Rust UEFI console-only prototypes.
- [x] Add language-specific UEFI feasibility notes for all eight tracks.
- [x] Add language-specific OS/kernel entry sketches and validation notes for all eight tracks.
- [x] Add GitHub Actions smoke tests for the eight cores and Rust UEFI build.
- [ ] Confirm the current CI run; never infer PASS from a queued or pending job.

## Milestone 1 — portable core quality
- [x] Add bounded FIFO event queues to all eight language cores.
- [x] Test focus announcement, unfocused silence, value/state formatting, queue bounds, and FIFO snapshot behavior across the language tracks.
- [x] Add circular-queue wraparound FIFO tests to Rust, Go, D, Ada, Zig, Nim, and Swift; C already has wraparound coverage.
- [ ] Confirm all updated edge-case tests in CI and repair any actual failures.
- [ ] Normalize byte limits, overflow reporting, and announcement formatting across all eight implementations.
- [ ] Keep core formatting independent of native accessibility APIs and speech output.

## Milestone 2 — native screen readers
For each language independently:
- [ ] Implement one platform accessibility adapter at a time.
- [ ] Read semantic role, accessible name, state, value, and focus events from native APIs.
- [ ] Implement a cancellable speech queue and an explicit audio backend.
- [ ] Add keyboard-only acceptance tests; OCR remains a fallback rather than the semantic source.
- [ ] Document unsupported platform APIs and dependencies.
- [ ] Validate accessibility behavior on a real supported OS; host core tests are not native screen-reader validation.

## Milestone 3 — UEFI application
For each language:
- [ ] Prove a minimal EFI application build for a pinned x86_64 toolchain.
- [ ] Inspect PE/COFF machine type, subsystem, entry point, and imports.
- [ ] Run viable EFI artifacts in QEMU/OVMF and retain logs as emulator-only evidence.
- [ ] Add GOP and keyboard protocol discovery with robust missing-protocol handling.
- [ ] Add firmware-menu navigation only where firmware exposes usable interfaces.
- [ ] Do not assume firmware provides generic audio; physical output needs separate proof.
- [ ] Keep firmware-variable writes, firmware flashing, and real-machine reboot out of unattended CI.

## Milestone 4 — independent OS/kernel
For each language:
- [x] Add explicitly unvalidated, language-specific kernel entry sketches and feasibility notes for all eight tracks.
- [ ] Pin a boot protocol, target architecture, toolchain/runtime profile, and linker layout.
- [ ] Build a freestanding kernel entry point for a concrete supported target.
- [ ] Add serial diagnostics, memory initialization, exception/interrupt handling, and timer.
- [ ] Add keyboard and display drivers; plan audio only against a concrete device path.
- [ ] Integrate accessibility before optional desktop layers.
- [ ] Maintain a distinct kernel artifact and test record.
- [ ] Prove emulator boot separately from physical hardware boot.

## Milestone 5 — accessibility and reliability
- [ ] Run structured keyboard-only navigation tests.
- [ ] Test speech interruption, device loss, focus storms, queue saturation, and recovery.
- [ ] Measure announcement latency and memory use.
- [ ] Add local AI only as an optional assistive feature; core navigation must work offline without it.
- [ ] Validate on supported physical hardware and record limitations without extrapolating from QEMU.
