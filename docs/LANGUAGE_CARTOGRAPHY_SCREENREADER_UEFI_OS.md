# Language cartography: screen reader, UEFI and OS

## Purpose

This document maps candidate languages to three independent deliverables: a native screen reader, a UEFI/pre-OS application, and an independently bootable OS/kernel. It is a feasibility and execution map, not evidence that the components already exist.

## Rules that apply to every language

- Keep screen reader, UEFI app and kernel in separate source trees, build jobs and release gates.
- A native executable is not automatically UEFI-compatible, freestanding, or a bootable OS.
- Mark each result PASS, FAIL, PARTIAL, BLOCKED or NOT_VALIDATED. Never infer PASS from a source file existing.
- Every PASS must record source commit, compiler/toolchain version, exact command, exit code, test logs and artifact SHA-256.
- QEMU results must be labelled EMULATOR; they do not prove physical hardware support.
- No firmware flashing, NVRAM writes, unattended physical reboot, or changes to the user's Windows disk.
- Accessibility must use semantic OS APIs where available; OCR is fallback only. Audio is not considered working until a real backend/device path is demonstrated.
- Do not force every language into every target. Where runtime, compiler, ABI or SDK requirements cannot be met, record BLOCKED with evidence and preserve a stable C ABI boundary where appropriate.

## Shared architecture

### Screen-reader layer

1. Language-neutral event contract: semantic node, accessible name, role, state, value, hierarchy, focus, live-region change, alert, keyboard input, speech request, cancellation and backend error.
2. Bounded event/speech queues, deterministic ordering, cancellation, duplicate suppression and overflow recovery.
3. Platform adapters:
   - Windows: UI Automation and a supported speech API.
   - Linux: AT-SPI2 and a supported local speech API/service.
   - Apple: Accessibility APIs and supported speech APIs.
4. Keyboard-first navigation and focus reporting; configurable voice, rate, volume and output device where supported.
5. Optional Braille and magnification adapters; unsupported targets must be explicitly documented.
6. Shared conformance vectors run against each language's portable core and platform adapter.

### UEFI/pre-OS layer

1. Produce a valid PE/COFF EFI application for a named architecture and ABI.
2. Discover console input/output and GOP where available; handle missing protocols and errors.
3. Keep pre-boot interaction useful even without sound: keyboard navigation, readable console output, structured status, and a documented handoff path.
4. No universal UEFI audio capability is assumed. Audio stays BLOCKED until a device-specific driver/path is implemented and tested.
5. Separate emulator validation from physical validation.

### OS/kernel layer

1. Specify architecture, boot protocol, linker layout, entry point and image format.
2. Implement early serial diagnostics, firmware memory-map ingestion, allocator, exceptions/interrupts and a documented scheduler baseline.
3. Add device discovery, keyboard/input and display drivers, then a stable event model.
4. Integrate accessibility events before claiming a complete graphical system.
5. Define audio hardware support and a real speech path; a synthesized PCM buffer alone is not proof of sound output.
6. Build a reproducible boot image and test host logic, emulator boot and physical boot separately.

## Language-by-language execution map

### Progress update — 14 full language records

The repository now includes initial portable-core scaffolding for C++, Free Pascal, Odin, Fortran, Objective-C and C#; C# has a hosted core and smoke-test executable. UEFI and kernel status for these additions remains NOT_VALIDATED, NOT_IMPLEMENTED or BLOCKED as stated in `projects/manifest.json`. CI jobs were added for C++ and Free Pascal, and Fortran. Their results are pending; no PASS is claimed. Objective-C is scoped to Apple-specific adapter work, not a shared bare-metal kernel.

### Existing eight priority tracks

#### C
- Screen reader: native OS adapters, semantic event core, speech backend, queue and failure tests.
- UEFI: EDK2 or equivalent PE/COFF build; console input/output and GOP.
- OS: freestanding compiler, linker script, boot protocol, memory/interrupt baseline.
- Gate: validate ABI, memory safety boundaries and reproducible artifacts.

#### Rust
- Screen reader: portable core plus platform bindings and speech integration.
- UEFI: `x86_64-unknown-uefi`, EFI entry/ABI, panic strategy and allocation policy.
- OS: `no_std`, target, linker, allocator and boot handoff.
- Gate: ensure all unsafe/MMIO code is isolated and tested; target builds must be reproducible.

#### Go
- Screen reader: native platform APIs through supported bindings; audit runtime, GC and speech dependencies.
- UEFI: demonstrate a reliable EFI ABI and runtime startup path before implementation is counted.
- OS: runtime/GC and scheduler/startup dependencies are a major feasibility gate.
- Gate: if freestanding startup/runtime cannot be demonstrated, mark UEFI/kernel BLOCKED rather than create a misleading stub.

#### D
- Screen reader: platform bindings, speech backend and runtime/GC audit.
- UEFI: prove compiler/runtime profile, EFI ABI and PE/COFF output.
- OS: prove freestanding profile, startup and runtime requirements.
- Gate: no kernel PASS until runtime and linker behavior are demonstrated.

#### Ada
- Screen reader: OS bindings and local speech integration.
- UEFI: validate GNAT target/runtime profile and elaboration/startup.
- OS: runtime profile, startup, linker and interrupt integration.
- Gate: preserve reproducible compiler/runtime configuration and document unavailable targets.

#### Zig
- Screen reader: native API bindings and speech backend.
- UEFI: target support, EFI ABI and linker output.
- OS: freestanding target, linker, boot protocol and allocator.
- Gate: pin compiler version and run the same shared behavioral test vectors.

#### Nim
- Screen reader: generated C and platform bindings; inspect generated code and runtime dependencies.
- UEFI: generated C toolchain, startup and runtime audit.
- OS: prove generated startup/runtime can operate freestanding.
- Gate: include generated-source and compiler-version evidence in artifacts.

#### Swift
- Screen reader: use supported platform SDKs and speech APIs; support differs by OS.
- UEFI: prove target/ABI and Swift runtime availability before coding beyond a feasibility prototype.
- OS: standard-library, runtime and freestanding feasibility must be demonstrated.
- Gate: unsupported compiler/runtime combinations remain BLOCKED.

### Additional candidates to evaluate

These candidates expand the language survey. They must first pass a toolchain/ABI feasibility spike; they are not automatically part of the release count.

#### C++
- Screen reader: high priority for Windows/Linux native adapters, system integration and speech backends.
- UEFI: feasible with a suitable freestanding C++ toolchain and disciplined runtime use.
- OS: feasible for kernel components with exceptions, RTTI, constructors and standard-library dependencies deliberately controlled.
- Priority: HIGH; candidate for a ninth full track after a reproducible toolchain check.

#### Assembly
- x86-64: boot entry, interrupt stubs, context switching and CPU-specific routines.
- AArch64/ARM64: equivalent routines for ARM systems.
- RISC-V: boot and trap/interrupt routines for RISC-V targets.
- Screen reader: not a practical standalone application language; use for narrowly scoped architecture-specific primitives.
- UEFI: only where architecture and firmware ABI match.
- OS: complementary to C/Rust/Ada/Zig; not a replacement for a complete systems language.
- Priority: HIGH as an architecture support layer, not as three independent complete screen readers.

#### Free Pascal
- Screen reader: native tooling and platform adapters if APIs and speech libraries are available.
- UEFI: first prove compiler output, PE/COFF format and entry ABI.
- OS: first prove freestanding startup, linker and runtime dependencies.
- Priority: MEDIUM; feasibility spike before creating full tracks.

#### Odin
- Screen reader: investigate native OS bindings and speech integrations.
- UEFI: verify target support, ABI, linker and runtime assumptions experimentally.
- OS: verify freestanding startup and memory/runtime dependencies.
- Priority: MEDIUM/EXPERIMENTAL; no UEFI/kernel claim without artifacts.

#### V
- Screen reader: evaluate generated C, native API bindings and speech dependencies.
- UEFI: generated C toolchain and runtime audit required.
- OS: generated startup and runtime must be proven freestanding.
- Priority: EXPERIMENTAL.

#### Objective-C
- Screen reader: targeted Apple Accessibility and speech adapter; not a cross-platform core replacement.
- UEFI: low priority; runtime and object model make it an unsuitable default for a common EFI implementation.
- OS: low priority for a portable freestanding kernel.
- Priority: HIGH only for Apple-specific adapter work.

#### Fortran
- Screen reader: low priority; could support numerical/ML subroutines, not the main event/focus layer.
- UEFI: toolchain/runtime and ABI would need proof.
- OS: low priority; runtime and I/O assumptions need a freestanding audit.
- Priority: LOW for the core, possible specialized numeric library.

#### Crystal
- Screen reader: possible native utility/backend experiments; runtime and GC must be present and audited.
- UEFI: not a default candidate unless a minimal runtime and EFI ABI are proven.
- OS: low priority because runtime/GC requirements are a substantial barrier.
- Priority: LOW/EXPERIMENTAL.

#### Modula-2
- Screen reader: only if a maintained compiler and required OS bindings are available.
- UEFI: compiler target and PE/COFF ABI must be demonstrated.
- OS: freestanding toolchain, linker and boot path must be demonstrated.
- Priority: LOW; compiler/toolchain availability is the first gate.

#### COBOL and PL/I
- Screen reader: not recommended for the semantic event/focus engine; possible only for peripheral tooling where useful.
- UEFI: not recommended unless a concrete supported freestanding target exists.
- OS: not recommended as a primary kernel language.
- Priority: LOW; do not create placeholder tracks merely to inflate language count.

## Recommended implementation order

1. Stabilize and test the existing eight portable cores using shared event and queue test vectors.
2. Build one real native screen-reader vertical slice on Windows: UI Automation + keyboard focus/navigation + connected speech backend + accessibility tests.
3. Add Linux AT-SPI2 and then Apple adapters when the required SDK/toolchain is available.
4. Validate C UEFI, then Rust UEFI, with reproducible PE/COFF artifacts and explicit emulator evidence.
5. Create one small bootable kernel milestone first (serial output, memory map, interrupts); use it as the reference design for other feasible language tracks.
6. Add C++ as the first extra full language only after its native reader, EFI and kernel gates are individually evidenced.
7. Add assembly per CPU architecture as support modules.
8. Run feasibility spikes for Free Pascal and Odin. Promote only candidates that pass build, ABI and runtime gates.
9. Keep Objective-C scoped to Apple accessibility. Keep V, Crystal, Fortran and Modula-2 experimental until a concrete need and reliable toolchain are established.
10. Produce a release evidence report listing all language × component combinations, with unsupported combinations clearly BLOCKED.

## Release status vocabulary

- PASS: all applicable requirements have reproducible evidence.
- PARTIAL: some implementation exists, but required integration or tests are missing.
- BLOCKED: a documented dependency, ABI, SDK, runtime or hardware limitation prevents progress.
- FAIL: a check was run and failed.
- NOT_VALIDATED: no adequate evidence has been collected yet.

## Current baseline

The eight existing language cores remain PARTIAL until real native accessibility adapters and speech backends are connected. C and Rust UEFI sources must be treated as prototypes until EFI build and runtime evidence is current. Do not mark any of the eight OS/kernel tracks as complete without a reproducible boot image and recorded boot tests. Additional candidates above are mapped for feasibility, not claimed implemented.


## Latest expansion status

The project manifest has been expanded to 14 language tracks (42 language/component records). C++ and Free Pascal have portable core test programs and CI jobs; Fortran has a portable core test program and CI job. Odin and Objective-C source has been added but is not yet compiler-validated. C# has a CI smoke-test job pending results. UEFI and kernel work for all new tracks remains unproven; see the manifest and CI runs. Architecture assembly is tracked as support code, not a standalone complete screen reader.


C# addition: the hosted .NET core has announcement and bounded FIFO queue smoke checks. Its UEFI and freestanding kernel tracks are explicitly BLOCKED until a compatible runtime/ABI is proven; .NET Native AOT alone is not UEFI or kernel proof.
