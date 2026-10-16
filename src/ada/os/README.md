# Ada operating-system / kernel track

Status: **NOT_IMPLEMENTED**.

## Feasibility boundary

Select a GNAT target and runtime profile explicitly. Kernel work must not accidentally link hosted services or assume a full Ada runtime is available after boot.

## Required implementation gates

1. Pin compiler, target, runtime profile, boot protocol, and linker layout.
2. Build a minimal entry point with serial diagnostics and inspect linked symbols.
3. Initialize memory, exceptions, interrupt handling, and timer with a documented runtime strategy.
4. Add keyboard/display drivers and a semantic accessibility event interface.
5. Preserve keyboard-first navigation if speech is unavailable; audio is BLOCKED pending a real driver and measured output.
6. Validate unit tests, emulator boot, and physical hardware independently.

Ada language support alone is not evidence that an Ada kernel can boot.
