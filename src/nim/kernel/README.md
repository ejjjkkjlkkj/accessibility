# Nim kernel track

Status: **NOT_IMPLEMENTED / NOT_VALIDATED**. This file records an independent kernel track, not a working operating system.

## Current source boundary

Exported procedure sketch only. Generated C, Nim runtime and initialization dependencies have not been eliminated; not a kernel binary.

## Required next evidence

1. Exact compiler version and target triple.
2. Reproducible freestanding object/link command and artifact hash.
3. Boot protocol and linker layout.
4. Memory map, exception/interrupt setup, timer and keyboard/display drivers.
5. Emulator boot evidence, followed separately by physical hardware validation.
6. Accessible input/navigation and speech/audio path; no audio success may be inferred from a buffer or console output.

QEMU is emulator evidence, not physical hardware proof.
