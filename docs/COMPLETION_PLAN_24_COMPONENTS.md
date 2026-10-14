# Completion plan: eight screen readers, eight UEFI applications, eight OS tracks

This plan is the execution contract for the language/component matrix currently recorded in `projects/manifest.json`. It does not mark any component complete by documentation alone.

## Scope

Language tracks currently include C, Rust, Go, D, Ada, Zig, Nim, Swift, C++, Free Pascal, Odin, Fortran, Objective-C and C#. Each track has three separate deliverables, with infeasible combinations explicitly BLOCKED rather than silently counted as complete:

1. **Native screen reader** — platform accessibility API adapter, semantic event model, focus/navigation, speech output, keyboard controls, recovery, and tests.
2. **UEFI/pre-OS application** — EFI application that reads firmware-provided input and display protocols and announces accessible setup/navigation information. Audio remains BLOCKED until a real device-specific path is demonstrated.
3. **Operating system/kernel** — independently bootable kernel/system track with documented boot protocol, memory model, interrupt handling, scheduling, device drivers, and system-level accessibility design.

## Definition of done: native screen reader

A track cannot be marked PASS until evidence exists for all applicable items:

- [ ] Platform adapter obtains semantic accessible name, role, state, value, hierarchy, focus and live changes. OCR is fallback only.
- [ ] Keyboard-first navigation and focus-change announcements work without a mouse.
- [ ] Speech backend is connected and can start, stop, interrupt and recover from failure.
- [ ] Speech queue is bounded; overflow, cancellation and event ordering are tested.
- [ ] Language, rate, volume and output-device controls are tested where supported.
- [ ] Braille and magnification integration is implemented or explicitly marked unsupported for the target.
- [ ] Automated tests include missing names, stale focus, rapid focus changes, duplicate events, queue saturation and backend failure.
- [ ] Build/test command, compiler version, OS, logs and artifact hashes are recorded.

## Definition of done: UEFI

- [ ] Build a valid PE/COFF EFI application for the specified architecture.
- [ ] Validate EFI entry point, protocol discovery, console input/output, GOP when available, and error paths.
- [ ] Avoid allocation and Boot Services after ExitBootServices; only an OS loader/kernel path may perform that transition.
- [ ] Do not claim generic audio support: UEFI has no universally available audio protocol.
- [ ] Run an emulator test and label it EMULATOR evidence; physical hardware validation is separate.
- [ ] Never change firmware variables, flash firmware or reboot physical hardware in unattended CI.

## Definition of done: OS/kernel

- [ ] Specify boot protocol, linker layout, entry point and architecture.
- [ ] Implement early serial diagnostics, memory map ingestion, allocator strategy, exception/interrupt handling and a minimal scheduler or documented single-task baseline.
- [ ] Define device discovery and driver boundaries.
- [ ] Integrate accessibility into system events and input/focus before assuming a graphical desktop exists.
- [ ] Build a reproducible boot image and pass host tests plus emulator boot tests.
- [ ] Record physical boot separately; emulator success is not physical proof.

## Implementation sequence

### Phase 0 — evidence and interfaces
1. Validate the language/component manifest and enforce allowed status labels.
2. Freeze a language-neutral event contract: focus, semantic node, value/state change, alert, input, speech request, cancellation and backend error.
3. Keep native, UEFI and kernel artifacts in separate directories and CI jobs.

### Phase 1 — finish portable screen-reader cores
1. Run and fix unit tests for all eight languages.
2. Add shared behavioral test vectors for event ordering, bounded queues and announcement formatting.
3. Keep all eight cores PARTIAL until a real OS accessibility adapter and speech backend are connected.

### Phase 2 — native operating-system adapters
Implement and validate adapters per OS rather than pretending one API works everywhere:
- Windows: UI Automation; speech backend through a supported OS API.
- Linux: AT-SPI2; speech backend through a supported local service/API.
- Apple platforms: Accessibility APIs and supported speech APIs, subject to toolchain/platform availability.
A language/OS combination is marked unsupported or BLOCKED when the necessary SDK, runtime or hardware is unavailable; it is never silently counted as complete.

### Phase 3 — UEFI applications
Implement one EFI app per language only where a reproducible compiler/runtime path exists. Prefer freestanding or minimal-runtime configurations. If a language cannot reliably target EFI, record the blocker and retain a documented C ABI boundary rather than claiming a native implementation.

### Phase 4 — OS/kernel tracks
Develop independent kernels incrementally, starting with bootable serial-output milestones. Do not equate a kernel entry stub with an OS. Establish boot, memory, interrupts, input, display and speech/device strategy before claiming integrated accessibility.

### Phase 5 — integration and release
- Run CI across all available toolchains.
- Publish a language-by-component evidence report with source revision, commands, versions, environment, result and artifacts.
- Require all critical accessibility tests to pass; list unsupported combinations explicitly.
- Do not merge or publish a release solely because documentation or host unit tests pass.

## Language-specific feasibility gates

| Language | Native reader | UEFI gate | Kernel gate |
|---|---|---|---|
| C | OS SDK, accessibility API and speech backend | EDK2/gnu-efi target and ABI | freestanding compiler, linker script, boot protocol |
| Rust | target-specific accessibility crates/API bindings and speech backend | `x86_64-unknown-uefi`, EFI ABI and panic/allocation policy | `no_std`, target, linker and allocator |
| Go | OS APIs via supported bindings; runtime and GC remain present | runtime/GC and EFI ABI must be demonstrated | runtime/GC and startup dependencies must be resolved |
| D | OS bindings plus runtime and GC audit | compiler/runtime profile and EFI ABI | freestanding profile and runtime audit |
| Ada | OS bindings and speech integration | GNAT target/runtime profile and elaboration | runtime profile, startup and linker |
| Zig | OS API bindings and speech backend | target support, EFI ABI and linker | target, linker, boot protocol and allocator |
| Nim | generated C/API bindings; audit generated code and runtime | generated C toolchain and runtime audit | generated startup/runtime and freestanding audit |
| Swift | platform SDK and speech APIs; platform support varies | Swift runtime and EFI target feasibility must be proven | runtime, standard library and freestanding feasibility must be proven |

## Current truthful baseline

The portable cores are PARTIAL. The existing C UEFI source is a console prototype; Rust UEFI is not validated. Other UEFI implementations and all eight complete OS kernels remain unimplemented. Native platform accessibility adapters and connected speech output remain unimplemented. No physical audio or hardware result is implied by this plan.
