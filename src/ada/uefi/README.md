# Ada UEFI / pre-OS track

Status: **NOT_IMPLEMENTED**.

## Current evidence and boundary

Requires a GNAT target/runtime profile compatible with EFI and bindings for System Table/Boot Services. Runtime initialization and ABI are not demonstrated.

The intended component is an independent firmware application, not an OS kernel and not yet a full screen reader. It must discover firmware protocols, support keyboard navigation and accessible text/graphics where possible, and never assume UEFI provides generic audio.

## Validation gates

1. Pin the compiler and exact target triple/version.
2. Build a PE/COFF EFI application and verify the artifact type and entry point.
3. Run it in QEMU/OVMF; record emulator configuration and logs.
4. Add GOP and keyboard protocol discovery and robust error reporting.
5. Treat speech/audio as BLOCKED until a concrete hardware path is implemented and physical output is measured.
6. Keep firmware-variable writes, firmware flashing and real-machine reboot out of unattended CI.

A source file or successful host build is not proof of EFI execution. QEMU is not physical-hardware proof.
