# Execution plan: complete screen reader, UEFI and OS tracks

Status: planning baseline; this document is not implementation or validation evidence.
Branch: work/8-language-screenreader-uefi-os
Language catalogue: `projects/manifest.json`; implementation/component matrix: `projects/multilanguage-component-manifest.json`.

## Objective and non-negotiable release gates

For every feasible language track, deliver three separately versioned products:
1. A native screen reader integrated with the target OS accessibility API and a real speech backend.
2. A UEFI application with keyboard-first pre-boot interaction. Audio is optional and remains blocked until a device-specific path is proven.
3. A separately bootable OS/kernel with its own boot protocol, memory, interrupt, input, display and accessibility design.

Do not label a track complete because a source file exists or compilation succeeds. Every PASS needs a commit SHA, exact command, toolchain version, exit code, logs, artifact SHA-256 and environment. Keep HOST, EMULATOR and PHYSICAL evidence separate. No firmware writes, NVRAM mutation, disk changes, or unattended physical reboot.

## Common screen-reader conformance contract

Every portable core and native adapter must implement the same versioned test vectors:
- Accessible node: stable identifier, name, role, state flags, value, parent/child relation and source.
- Events: focus gained/lost, name/role/state/value change, live-region update, alert, selection, text caret and adapter/backend error.
- Navigation: next/previous item, parent/child, first/last, landmark/heading where supported, and predictable focus recovery.
- Speech queue: bounded capacity, deterministic order, duplicate coalescing, interruption/cancel, priority alerts, overflow recovery and backend failure handling.
- Controls: speech on/off, stop, rate, volume, voice/language selection, and output device where the platform supports it.
- Reliability: empty and malformed event handling, stale node, disappearing window, focus loops, queue saturation, unavailable speech service, and adapter reconnection.
- Accessibility: keyboard-only operation; no OCR as primary source when semantic APIs are available. Braille/magnification are separate optional adapters.
- AI: optional local assistance only; it must never block core focus reporting or navigation.

A portable formatter or FIFO is a component, not a complete screen reader. Required native vertical slices are Windows UI Automation + speech backend first, Linux AT-SPI2 + local speech next, and Apple Accessibility + supported speech APIs when built on a supported Apple SDK.

## UEFI conformance contract

Each claimed EFI app must:
- Build as PE/COFF for an explicit machine architecture and conform to the EFI entry ABI.
- Discover and validate ConsoleIn/ConsoleOut and GOP; report absent protocols safely.
- Support keyboard navigation and clear text status without requiring audio.
- Validate key handling, bounds, malformed protocol results, allocation failures, and exit paths.
- Provide artifact hash, compiler/toolchain versions, build command, and emulator log.
- Treat firmware-specific audio as BLOCKED until a named driver/device route and actual output are demonstrated.
- Separate application handoff from kernel boot. An EFI app is not an OS.

## OS/kernel conformance contract

Each kernel track must state CPU architecture, bootloader/protocol, image format, linker layout, entry point, memory-map contract and supported target. Required staged milestones:
1. Reproducible freestanding image with serial diagnostics.
2. Firmware memory-map ingestion and allocator with boundary tests.
3. Exception/interrupt entry and timer/interrupt test.
4. Keyboard/input path and stable event contract.
5. Display path and accessible text/semantic event model.
6. Screen-reader integration and real speech path for a declared hardware target.
7. Reproducible boot image and emulator boot evidence; physical validation separately recorded.

A kernel entry sketch, linker script alone, PCM buffer, or emulator boot alone is not a complete accessible OS.

## Language track decisions

### Core tracks: C, Rust, Go, D, Ada, Zig, Nim, Swift
Keep all eight tracks. For each, finish native reader, UEFI application and kernel independently. First establish common conformance vectors and CI; then use C and Rust as initial reference vertical slices. Do not copy status across languages: each track needs its own build and runtime evidence.
- C: EDK2/PE-COFF is the initial UEFI reference; native Windows UIA/speech adapter is first reader integration.
- Rust: `no_std` EFI target and separately configured kernel target; isolate and review unsafe/MMIO code.
- Go, D, Nim and Swift: prove runtime, startup, allocator/GC and ABI constraints before calling a target freestanding. Keep unsupported targets BLOCKED rather than using hosted stubs.
- Ada: document GNAT runtime profile, elaboration/startup, EFI ABI and interrupt model.
- Zig: pin compiler version and test target/ABI/linker output explicitly.

### Additional full-track candidates
- C++: first additional candidate for native adapters, freestanding EFI and kernel work; control exceptions, RTTI, static constructors and standard-library use in freestanding targets.
- Free Pascal and Odin: run toolchain/ABI/runtime feasibility spikes first; promote only after reproducible native core tests and target output are proven.
- C#: prioritize hosted native screen-reader integration; UEFI/kernel remain BLOCKED unless a genuinely compatible freestanding runtime and EFI/boot ABI are demonstrated.
- Objective-C: native Apple accessibility adapter track; do not claim a generic EFI app or portable kernel without a proven freestanding runtime.
- V and Crystal: experimental until generated/runtime dependencies, native API access and toolchains are verified.
- Fortran: potential numerical/ML helper libraries, not the default semantic focus/event engine; runtime/ABI gates apply.
- Modula-2: toolchain availability and target ABI are first gates.
- COBOL and PL/I: do not create placeholder kernel/UEFI implementations merely to increase the language count; evaluate only for a concrete peripheral use case.

### Assembly support tracks
x86-64, AArch64/ARM64 and RISC-V assembly are architecture-specific support modules for entry stubs, exception/interrupt entry and context-switch primitives. They are not independent complete screen-reader or OS-language tracks. Validate against the matching architecture and ABI.

## CI and release evidence

For each language/component pair, CI should emit a machine-readable record with:
- language, component, source commit, target triple/architecture;
- compiler/toolchain version and exact command;
- exit code, test count, log/artifact references and SHA-256;
- environment = HOST, EMULATOR or PHYSICAL;
- status = PASS, FAIL, PARTIAL, BLOCKED or NOT_VALIDATED;
- explicit unimplemented requirements and next gate.

CI must fail if a record claims PASS without required evidence fields. CI may mark a component BLOCKED only with a specific technical reason and a documented feasibility test. Do not fabricate tests or results.

## Execution order

1. Audit the current 14-track manifest and ensure every status matches actual source/tests and latest CI results.
2. Add shared screen-reader event/queue/navigation test vectors and run them against each existing portable core.
3. Implement and validate one end-to-end Windows screen-reader vertical slice: UI Automation events, keyboard/focus navigation, connected speech, stop/cancel, and regression tests.
4. Implement Linux and Apple adapters in supported environments; label unavailable SDK/toolchains as BLOCKED.
5. Build and test C UEFI, then Rust UEFI, with explicit emulator evidence.
6. Establish one bootable kernel reference milestone, starting with serial output and memory-map handling; do not copy PASS to other languages.
7. Expand C++ as the first extra full track. Run Free Pascal/Odin feasibility spikes before promoting them.
8. Build per-architecture assembly support modules and test them only on matching targets.
9. Run feasibility gates for every remaining candidate; retain unsupported candidates in the cartography without pretending they are complete products.
10. Publish a release evidence report only when each claim links to its reproducible artifacts. Keep all incomplete products marked PARTIAL/BLOCKED/NOT_VALIDATED.

## Current truthful state

The current repository manifest describes early portable cores and prototypes. Native platform adapters, connected speech, validated UEFI apps and bootable accessible kernels are not complete across all languages. This plan does not change those statuses. The user's physical PC must not be treated as tested until a separately approved physical validation procedure is actually executed and recorded.


## Expanded execution scope — shared language catalog

The execution scope is now the feasibility-driven language catalog in `LANGUAGE_CARTOGRAPHY_SCREENREADER_UEFI_OS.md`, not an arbitrary fixed number. For every eligible systems language, maintain three independent deliverables and release gates: complete native screen reader, complete UEFI/pre-OS application, and independently bootable OS/kernel. For hosted-only languages, complete the native screen reader and formally mark UEFI/kernel BLOCKED when freestanding support is not technically proven; do not fake those targets.

### Work packages and acceptance criteria

1. **Catalog and inventory:** assign stable IDs to every language, record compiler/runtime versions and source paths, and reconcile each status against CI evidence.
2. **Shared reader contract:** define versioned language-neutral semantic nodes, focus and value events, navigation, bounded speech queue, cancellation, priority alerts, settings, braille/magnification adapters, failure recovery and conformance vectors.
3. **Native reader vertical slices:** implement Windows UI Automation + real speech first, Linux AT-SPI2 + local speech next, and Apple Accessibility + supported speech APIs on Apple SDK runners. Use semantic APIs before OCR. Keep AI optional and non-blocking.
4. **UEFI vertical slices:** prove C/EDK2 then Rust EFI builds; add explicit build and emulator tests; then implement each feasible language independently. Verify PE/COFF, EFI ABI, protocol discovery, keyboard/GOP, errors, bounds and artifact SHA-256. Firmware audio stays BLOCKED absent a proven device-specific route.
5. **Kernel vertical slice:** prove a reproducible freestanding reference boot image, serial output and firmware memory-map ingestion; then allocator, exceptions/interrupts, timer, keyboard, display, event model and speech/audio hardware route. Port each layer only where the language/toolchain is genuinely freestanding.
6. **Cross-language parity:** execute shared event/navigation vectors in each implementation, add property/fuzz tests and compare outputs. One language passing never promotes another.
7. **AI repository handoff:** mirror the stable language IDs and feasibility notes into `ejjjkkjlkkj/ia`. Start with a local shared base model and language-specific parser/tool/retrieval/evaluation packages. Fine-tuning is an optional evidence-gated experiment; do not claim independent models or training unless artifacts and benchmark results exist.
8. **Release evidence:** produce a machine-readable record per language × component with commit SHA, toolchain, exact command, exit code, tests, artifact hash, environment (HOST/EMULATOR/PHYSICAL), status, blockers and next gate. CI rejects unsupported PASS claims.

No merge, release, physical boot, firmware mutation, disk mutation or unattended reboot is authorized by this plan. CI and emulator outcomes are not physical proof.
