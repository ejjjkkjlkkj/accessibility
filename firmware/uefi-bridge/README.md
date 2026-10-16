# Native UEFI PCI inventory application

This EDK II module is the first concrete firmware-side adapter. It locates handles that expose `EFI_PCI_IO_PROTOCOL`, reads PCI vendor/device identifiers and class bytes through that protocol, and prints a bounded inventory to the UEFI text console.

## Safety boundary

- Read-only PCI configuration reads through `EFI_PCI_IO_PROTOCOL`.
- No raw ECAM address dereferencing, MMIO mapping, I/O-port access, DMA, firmware-variable writes, boot-order changes, or reboot requests.
- Reported records are **firmware-reported**, not cryptographic attestations and not independent physical-proof evidence.
- Console output is not spoken output. This module does not yet provide accessible audio or braille output.
- The compile workflow targets the pinned EDK II commit `2970e5699ba6267f3384ffab20f96647578aebc8` (`edk2-stable202608`) and uploads the EFI binary as a review artifact when the build succeeds. A successful compile still does **not** prove execution on the target machine.

## Build integration

1. Use an EDK II workspace with `MdePkg` available.
2. Add `firmware/uefi-bridge/HardwareBridge.inf` to the `[Components]` section of the platform DSC you intend to build.
3. Build that platform for X64 using your configured EDK II toolchain.
4. Inspect the produced EFI application and its hash before any execution.
5. Only run it through an already-authorized UEFI Shell or approved firmware workflow. Do not change Secure Boot, boot order, NVRAM, or firmware settings just to run this test.

The host-side Rust contracts remain a separate no-std foundation. This application is not yet connected to those contracts, a native screen-reader runtime, speech output, or braille. No physical UEFI run is claimed.
