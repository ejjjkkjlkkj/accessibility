# Development roadmap

## Milestone 0 — project integrity
- [x] Define eight language tracks and 24 component IDs.
- [x] Establish evidence labels and distinguish emulation from physical validation.
- [x] Add initial focus-announcement cores in all eight languages.
- [x] Add starter tests for C, Go, and Rust.
- [x] Add an EDK2 C UEFI console-only entry-point prototype.
- [x] Execute initial CI: manifest validation and all eight portable-core smoke-test jobs passed on the initial revision.\n- [ ] Validate the newly added Rust UEFI build job; result pending.

## Milestone 1 — portable core quality
- [ ] Add tests for D, Ada, Zig, Nim, and Swift.
- [ ] Normalize announcement formatting across all eight implementations.
- [x] Extend C core tests for empty/unfocused output, disabled state, truncation reporting, invalid inputs, queue FIFO/copy semantics, capacity, and wrap-around.\n- [x] Add bounded event queues to C, Rust, Go, D, Ada, Zig, Nim, and Swift source tracks.\n- [ ] Confirm all eight languages' updated edge-case tests in CI and repair actual failures.
- [ ] Keep core formatting independent of native accessibility APIs and speech output.

## Milestone 2 — native screen readers
For each language independently:
- [ ] Implement one platform accessibility adapter at a time.
- [ ] Read semantic role, accessible name, state, value, and focus events from native APIs.
- [ ] Implement a cancellable speech queue and an explicit audio backend.
- [ ] Add keyboard-only acceptance tests; OCR remains a fallback rather than the semantic source.
- [ ] Document unsupported platform APIs and dependencies.

## Milestone 3 — UEFI application
For each language:
- [ ] Prove minimal C and Rust EFI applications build for x86_64; Rust CI build job added, result pending.\n- [ ] Start and verify each artifact in QEMU (emulator-only evidence).
- [ ] Add GOP text/graphics support and keyboard input with protocol discovery.
- [ ] Add firmware-menu navigation only where the firmware exposes usable interfaces.
- [ ] Do not assume firmware provides generic audio.
- [ ] Track physical-hardware results separately.

## Milestone 4 — independent OS/kernel
For each language:
- [ ] Document the boot protocol and minimal supported hardware.
- [x] Add explicitly unvalidated, language-specific kernel entry sketches and validation notes for all eight languages.\n- [ ] Build a freestanding kernel entry point and linker layout for a concrete supported target.
- [ ] Add serial diagnostics, memory initialization, exception/interrupt handling, and timer.
- [ ] Add keyboard, display, and audio driver plans with hardware evidence.
- [ ] Integrate the accessibility service before optional desktop layers.
- [ ] Maintain a distinct kernel artifact and test record.

## Milestone 5 — accessibility and reliability
- [ ] Run structured keyboard navigation tests.
- [ ] Test speech interruption, device loss, focus storms, and recovery.
- [ ] Measure announcement latency and memory usage.
- [ ] Add local AI only as an optional assistive feature; core navigation must work offline without it.
- [ ] Perform emulator and physical-hardware validation separately.
