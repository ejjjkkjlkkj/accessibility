# Accessibility: 8 language-specific stacks

Goal: build **eight independent implementations**, one each in C, Rust, Go, D, Ada, Zig, Nim and Swift. Each language project has three distinct deliverables:

1. A native screen reader for supported operating systems.
2. A UEFI application / pre-OS accessibility environment, with bare-metal work explicitly separated from firmware applications.
3. An operating-system research track with a kernel and accessibility integrated into the system design.

This repository currently contains the project definition and validation scaffolding. **It does not yet contain eight working screen readers, eight UEFI implementations, or eight operating systems.** Progress must be recorded per component and supported by build/test evidence.

## Language tracks

See [the implementation matrix](docs/IMPLEMENTATION_MATRIX.md) and machine-readable [project manifest](projects/manifest.json).

## Evidence rules

- `PASS`: the named test ran and passed; include the command and environment.
- `FAIL`: a test ran and failed; retain its useful output.
- `PARTIAL`: only part of the requirement is implemented or verified.
- `BLOCKED`: a named dependency or hardware constraint prevents validation.
- `NOT_VALIDATED`: no adequate evidence yet.
- QEMU or another emulator is not proof of physical hardware operation.
- Never claim audio output, firmware compatibility, accessibility coverage, or OS boot without evidence specific to that claim.
- Keep native OS, UEFI application, and bare-metal OS results separate.

## Initial scope

Start with an auditable common screen-reader contract, then implement and test each language track independently. UEFI applications and full OS kernels have different runtime and toolchain requirements; a successful native build must not be presented as proof of UEFI or kernel support.
