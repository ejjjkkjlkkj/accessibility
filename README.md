# Accessibility — multi-language screen reader, UEFI and OS engineering

Goal: build the widest technically feasible catalog of independent language tracks. Each eligible track has three separately validated deliverables:

1. A complete native screen reader integrated with the supported OS accessibility APIs and a real speech backend.
2. A UEFI/pre-OS application with EFI ABI/PE-COFF validation, keyboard interaction, GOP support where available, robust error handling and reproducible artifacts.
3. A separately bootable OS/kernel with a declared architecture and boot protocol, memory management, interrupts, input/display drivers and an accessibility/speech path for declared hardware.

The repository contains early portable cores, prototypes and planning/CI scaffolding. **It does not yet contain a complete screen reader, UEFI implementation or OS for every listed language.** Status is tracked per component and requires evidence.

## Canonical planning documents

- [Implementation matrix](docs/IMPLEMENTATION_MATRIX.md)
- [Language cartography and feasibility tiers](docs/LANGUAGE_CARTOGRAPHY_SCREENREADER_UEFI_OS.md)
- [Execution plan and release gates](docs/EXECUTION_PLAN_ALL_LANGUAGE_RELEASE_GATES.md)
- [Machine-readable project manifest](projects/manifest.json)

## AI project synchronization

The same stable language catalog is being used by [the local-first AI repository](https://github.com/ejjjkkjlkkj/ia), with a per-language toolchain, parser, retrieval and evaluation package. Language-specific fine-tuning is optional and must be evidenced; a separate foundation model is not claimed just because a language entry exists.

## Evidence rules

- `PASS`: the named test ran and passed; include commit, command, toolchain, environment and artifact/hash.
- `FAIL`: a test ran and failed; retain useful output.
- `PARTIAL`: implementation or validation is incomplete.
- `BLOCKED`: a concrete runtime, ABI, SDK, hardware or toolchain constraint prevents the target.
- `NOT_VALIDATED`: no adequate evidence yet.
- `NOT_IMPLEMENTED`: no implementation exists for the component.
- QEMU/emulator results are not proof of physical hardware operation.
- A portable formatter is not a complete screen reader; a PCM buffer is not proof of audio; an EFI app is not an OS.
- Keep host, emulator and physical validation separate.
- No firmware/NVRAM mutation, firmware flashing, changes to the user's Windows disk, or unattended physical reboot.

## Execution

Start with reference vertical slices in C and Rust, then advance each language independently. Add feasibility candidates only when compiler, runtime and target evidence exists. Hosted languages can deliver native screen-reader applications where appropriate; do not fake freestanding UEFI or kernel support when runtime/ABI requirements cannot be met.
