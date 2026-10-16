# C operating-system / kernel track

Status: **NOT_IMPLEMENTED**.

## Proposed first target

x86_64, booted through a documented boot protocol, with serial diagnostics first. Keep the kernel freestanding: no hosted C library, process environment, or implicit OS services.

## Required implementation gates

1. Pin compiler, linker, target triple, linker script, boot protocol, and artifact format.
2. Add entry point, stack setup, serial output, and a reproducible build.
3. Initialize physical/virtual memory, exceptions, interrupt controllers, and timer.
4. Add keyboard and framebuffer drivers with explicit ownership and bounds checks.
5. Integrate a semantic focus/event service before any graphical shell; speech remains BLOCKED until a concrete audio device driver is validated.
6. Test host unit tests, emulator boot, and physical hardware separately.

A successful host build is not a boot test. QEMU evidence is emulator-only; no physical boot or audio is claimed.
