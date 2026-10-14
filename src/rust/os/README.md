# Rust operating-system / kernel track

Status: **NOT_IMPLEMENTED**.

## Proposed first target

A freestanding x86_64 kernel using `no_std`, a pinned boot protocol, and a documented linker layout. The portable screen-reader crate is not itself a kernel and must not be treated as one.

## Required implementation gates

1. Pin target, Rust toolchain, boot protocol, linker script, panic handler, and allocator strategy.
2. Build a minimal entry point with serial diagnostics and inspect the kernel artifact.
3. Implement memory initialization, exception handling, interrupt controllers, and timer.
4. Add keyboard and display drivers, then expose a semantic focus/event API.
5. Keep speech independent of focus acquisition; audio is BLOCKED until a specific driver and output path are validated.
6. Separate unit tests, QEMU/OVMF or kernel-emulator boot tests, and physical hardware evidence.

No OS boot, screen-reader usability, or physical audio is currently validated.
