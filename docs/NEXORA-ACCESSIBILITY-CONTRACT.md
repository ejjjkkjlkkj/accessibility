# NEXORA accessibility integration contract

Status: **CONTRACT_ONLY_NOT_IMPLEMENTED**. This file defines the boundary expected of a future NEXORA runtime; it does not implement a screen reader or native platform adapter.

Machine-readable contract: [projects/nexora/accessibility-contract.json](../projects/nexora/accessibility-contract.json).

## Architecture

1. Native adapters obtain the platform's accessible tree and events (for example Windows UI Automation or Linux AT-SPI).
2. Adapters normalize nodes/events into the language-neutral contract.
3. The deterministic navigation core handles focus movement and reading commands.
4. Speech and braille are separate output adapters.
5. NEXORA AI integration, when available, is optional and cannot issue privileged actions directly.

## Mandatory safety behavior

- Password values are never announced.
- Unknown roles/states degrade to safe generic output.
- Node IDs and event sequence numbers are checked; stale or malformed events are rejected.
- Actions such as activation, writing values, process launch, network, firmware, NVRAM, reboot, or firmware flashing require a specific native adapter and explicit authorization; they are not enabled by this contract alone.
- Basic navigation must work without an AI model or cloud service.
- Screen-reader, OS integration, and UEFI/bare-metal status are tracked separately.

## Validation required before PASS

- Contract schema and bounds tests.
- Native adapter contract tests on the target OS.
- Keyboard-only end-to-end tests in real applications.
- Speech and braille output tests with logs.
- Security tests for password suppression, stale events, malicious model output, and denied actions.
- Separate physical-device evidence for UEFI/hardware features; QEMU is simulation only.

No NEXORA screen-reader functionality is claimed as implemented by adding this contract.
