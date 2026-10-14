# Zig operating-system / kernel track

Status: **NOT_IMPLEMENTED**.

## Proposed first target

A freestanding x86_64 kernel with a pinned Zig version, explicit target, linker script, and boot protocol.

## Required implementation gates

1. Build a minimal entry point and serial diagnostics; inspect the artifact and linked symbols.
2. Initialize memory, exceptions, interrupt controllers, and timer.
3. Add bounded keyboard input and framebuffer/display drivers.
4. Integrate semantic focus tracking and keyboard navigation before optional graphics or AI.
5. Keep speech optional until a concrete audio driver is validated; never infer audio from a successful build.
6. Run emulator tests and record physical-hardware tests separately.

Zig's freestanding capabilities are a feasibility advantage, not proof of a working OS.
