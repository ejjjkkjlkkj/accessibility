# Native UEFI PCI inventory application

This EDK II module is the first concrete firmware-side adapter. It locates handles that expose `EFI_PCI_IO_PROTOCOL`, reads PCI vendor/device identifiers and class bytes through that protocol, and prints a bounded inventory to the UEFI text console.

## Safety boundary

- Read-only PCI configuration reads through `EFI_PCI_IO_PROTOCOL`.
- No raw ECAM address dereferencing, MMIO mapping, I/O-port access, DMA, firmware-variable writes, boot-order changes, or reboot requests.
- Reported records are **firmware-reported**, not cryptographic attestations and not independent physical-proof evidence.
- Console output is not spoken output. This module does not yet provide accessible audio or braille output.
- The code has been added to source control but has **not yet been compiled against an EDK II toolchain or run on the target machine**.

## Build integration

1. Use an EDK II workspace with `MdePkg` available.
2. Add `firmware/uefi-bridge/HardwareBridge.inf` to the `[Components]` section of the platform DSC you intend to build.
3. Build that platform for X64 using your configured EDK II toolchain.
4. Inspect the produced EFI application and its hash before any execution.
5. Only run it through an already-authorized UEFI Shell or approved firmware workflow. Do not change Secure Boot, boot order, NVRAM, or firmware settings just to run this test.

The host-side Rust contracts remain the testable, no-std foundation. A future integration step will normalize this application's firmware inventory into those contracts and forward sanitized events to the accessibility event queue.
