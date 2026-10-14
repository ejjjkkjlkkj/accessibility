# Implementation matrix

Status reflects the strongest current evidence. All eight portable screen-reader cores contain announcement formatting, bounded event queues, and bounded cursor-navigation implementations with regression tests for empty, single-item, and multi-item lists. The expanded tests have been committed, but CI results must be checked per language before claiming validation. They remain **PARTIAL** because native accessibility API adapters and speech/audio are not connected. Kernel sketches and UEFI prototypes are not complete implementations.

| Language | Native screen reader | UEFI / pre-OS app | OS / kernel track | Key technical boundary |
|---|---|---|---|---|
| C | PARTIAL | NOT_VALIDATED (interactive arrow-key text menu source; build/run unproven) | NOT_IMPLEMENTED | EDK2 build/run and bootable kernel linker/boot protocol remain unproven |
| Rust | PARTIAL | NOT_VALIDATED (console-only source; CI build result unconfirmed) | NOT_IMPLEMENTED | UEFI build result, kernel boot contract, allocator and drivers remain unproven |
| Go | PARTIAL | NOT_IMPLEMENTED | NOT_IMPLEMENTED | Standard runtime/GC complicates EFI and freestanding kernel use |
| D | PARTIAL | NOT_IMPLEMENTED | NOT_IMPLEMENTED | Compiler runtime, GC and target ABI must be audited |
| Ada | PARTIAL | NOT_IMPLEMENTED | NOT_IMPLEMENTED | GNAT runtime profile, elaboration and EFI ABI must be proven |
| Zig | PARTIAL | NOT_IMPLEMENTED | NOT_IMPLEMENTED | UEFI target and kernel linker/boot configuration still need build evidence |
| Nim | PARTIAL | NOT_IMPLEMENTED | NOT_IMPLEMENTED | Generated C and Nim runtime dependencies must be audited |
| Swift | PARTIAL | NOT_IMPLEMENTED | NOT_IMPLEMENTED | EFI target, Swift runtime and freestanding support remain unproven |

## Required capabilities per screen reader

- Keyboard-first navigation and predictable focus tracking.
- Accessible names, roles, states, values, and change notifications from the target platform's accessibility APIs.
- Speech output through a clearly identified platform backend.
- Braille and magnification integration where the platform supports them.
- Language selection, speech rate/volume controls, interrupt/stop speech, and recovery from audio failure.
- Automated tests for navigation, focus, event ordering, and speech queue behavior.
- AI is an optional local subsystem, never a prerequisite for core navigation, and must not block immediate speech.
- No OCR-only approach where semantic accessibility APIs are available.

## Required capabilities per UEFI track

- Separate UEFI application from an actual standalone OS kernel.
- Validate PE/COFF target, EFI entry point, protocol discovery, GOP handling, keyboard input protocol, memory-map handling, and exit/boot-service constraints.
- The C prototype now navigates a small firmware text menu with Up/Down, Enter and Escape. This is keyboard navigation only, not speech or a semantic screen reader.
- Audio must be treated as BLOCKED until a specific device/driver path and physical output are demonstrated; UEFI does not guarantee a generic audio protocol.
- Never modify firmware variables, flash firmware, or reboot a real machine as part of unattended CI.
- QEMU results are emulator-only evidence.

## Required capabilities per OS track

- Document architecture, bootloader/entry point, memory management, interrupt handling, scheduler, device model, and driver plan.
- Screen reader must be integrated at the system level; do not assume a graphical desktop exists.
- Identify a concrete supported hardware target and boot-test strategy.
- Separate host-side unit tests, emulator boot tests, and physical hardware validation.
- Kernel entry sketches are not bootable OS implementations until a linker layout, boot protocol and runtime requirements are validated.

## Evidence format

For each test record: component, source revision, command, compiler/toolchain version, environment (host/emulator/physical), result, logs/artifacts, and remaining limitations. A compile-only check is not a runtime test; a runtime emulator test is not physical validation.
