# Swift operating-system / kernel track

Status: **NOT_IMPLEMENTED**.

## Feasibility boundary

Swift must not be assumed to be a viable freestanding kernel language without a pinned toolchain that can build the required target and runtime subset. Standard-library and runtime dependencies need explicit evidence.

## Required implementation gates

1. Identify a compiler/target and document runtime, allocator, initialization, and ABI requirements.
2. Prove a minimal freestanding entry point and inspect unresolved symbols and linked dependencies.
3. If feasible, add serial diagnostics, memory initialization, exception/interrupt handling, and timer.
4. Add keyboard/display drivers and a semantic accessibility event service.
5. Keep keyboard navigation independent of speech; audio is BLOCKED until a concrete driver and output are demonstrated.
6. If toolchain feasibility fails, retain reproducible failure evidence and mark the track BLOCKED instead of claiming kernel support.

No Swift kernel build, boot, or screen-reader operation is currently validated.
