# Implementation matrix

Status reflects the strongest current evidence. The C and Rust UEFI prototypes are source-level console applications only; neither is yet claimed to compile, boot, navigate firmware menus, or produce audio. This matrix is a work ledger, not a claim that the components already work.

| Language | Native screen reader | UEFI / pre-OS app | OS / kernel track | Key technical boundary |
|---|---|---|---|---|
| C | PARTIAL | NOT_VALIDATED (console-only source) | NOT_IMPLEMENTED |
| Rust | PARTIAL | NOT_VALIDATED (console-only source; CI pending) | NOT_IMPLEMENTED |
| Go | PARTIAL | NOT_IMPLEMENTED (runtime/EFI ABI gate) | NOT_IMPLEMENTED |
| D | PARTIAL | NOT_IMPLEMENTED (runtime/EFI ABI gate) | NOT_IMPLEMENTED |
| Ada | PARTIAL | NOT_IMPLEMENTED (GNAT runtime/EFI ABI gate) | NOT_IMPLEMENTED |
| Zig | PARTIAL | NOT_IMPLEMENTED (EFI target/build not yet proven) | NOT_IMPLEMENTED |
| Nim | PARTIAL | NOT_IMPLEMENTED (generated C/runtime gate) | NOT_IMPLEMENTED |
| Swift | PARTIAL | NOT_IMPLEMENTED (runtime/EFI target gate) | NOT_IMPLEMENTED |

## Required capabilities per screen reader

- Keyboard-first navigation and predictable focus tracking.
- Accessible names, roles, states, values, and change notifications from the target platform's accessibility APIs.
- Speech output through a clearly identified platform backend.
- Braille and magnification integration where the platform supports them.
- Language selection, speech rate/volume controls, interrupt/stop speech, and recovery from audio failure.
- Automated tests for navigation, focus, event ordering, and speech queue behavior.
- IA is an optional local subsystem, never a prerequisite for core navigation, and must not block immediate speech.
- No OCR-only approach where semantic accessibility APIs are available.

## Required capabilities per UEFI track

- Separate UEFI application from an actual standalone OS kernel.
- Validate PE/COFF target, EFI entry point, protocol discovery, GOP handling, keyboard input protocol, memory-map handling, and exit/boot-service constraints.
- Audio must be treated as BLOCKED until a specific device/driver path and physical output are demonstrated; UEFI does not guarantee a generic audio protocol.
- Never modify firmware variables, flash firmware, or reboot a real machine as part of unattended CI.
- QEMU results are emulator-only evidence.

## Required capabilities per OS track

- Document architecture, bootloader/entry point, memory management, interrupt handling, scheduler, device model, and driver plan.
- Screen reader must be integrated at the system level; do not assume a graphical desktop exists.
- Identify a concrete supported hardware target and boot-test strategy.
- Separate host-side unit tests, emulator boot tests, and physical hardware validation.

## Evidence format

For each test record: component, source revision, command, compiler/toolchain version, environment (host/emulator/physical), result, logs/artifacts, and remaining limitations. A compile-only check is not a runtime test; a runtime emulator test is not physical validation.
