# Go operating-system / kernel track

Status: **NOT_IMPLEMENTED**.

## Feasibility boundary

Go's standard runtime, garbage collector, scheduler, and assumptions about a hosted environment are not automatically suitable for a freestanding kernel. The initial feasibility gate is a minimal runtime-free entry point, not a conventional Go executable.

## Required implementation gates

1. Pin compiler/version and prove the chosen target can emit a freestanding kernel artifact.
2. Document which runtime components are excluded, replaced, or initialized.
3. Establish boot entry, stack, serial output, memory initialization, exceptions, interrupts, and timer.
4. Implement keyboard/display drivers and a semantic accessibility event service.
5. Do not claim speech until an explicit audio driver works; retain core keyboard navigation if audio is unavailable.
6. Record emulator and physical results independently.

If a minimal runtime-free kernel cannot be demonstrated, mark the feasibility result BLOCKED with build evidence.
