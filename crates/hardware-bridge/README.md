# Accessibility Hardware Bridge

A small `no_std` Rust foundation for a hardware-facing accessibility architecture. It is an inventory and event contract, not a hypervisor or device emulator.

## Implemented in this crate

- Typed PCI addresses and device descriptors with bounds and identifier checks.
- Resource-window validation using checked arithmetic and domain-aware overlap checks.
- Bounded ACPI parsers for RSDP, SDT, RSDT/XSDT and MCFG structures, including length, checksum, reserved-byte, bus-range and ECAM-alignment checks.
- A `PciConfigReader` contract for a future authorized UEFI PCI I/O Protocol adapter, plus bounded bus/device/function enumeration logic over that contract.
- Deny-by-default resource policy evaluator: verifies resource kind, declared-window containment, range validity, per-kind read/write allowlists and maximum request size. A policy decision is metadata only, not an OS/firmware I/O capability.
- Explicit provenance records that distinguish ACPI bytes, firmware-reported devices, OS-reported inventory, synthetic fixtures and manual observation. The provenance log is not a cryptographic attestation.
- Fixed-capacity, allocation-free accessibility event FIFO with explicit backpressure. Producers cannot silently overwrite queued events.
- Read-only Windows Plug and Play inventory probe at `tools/hardware-bridge-inventory.ps1`; JSON output defaults to the system temporary directory. It labels data as OS-reported, not physically verified.
- GitHub Actions checks for Rust unit tests, a release `no_std` library build on Linux and Windows, and a Windows PowerShell self-test.

## Safety and evidence boundary

This branch does **not** directly map ECAM/MMIO, perform port I/O or DMA, register interrupts, drive audio, mutate firmware/NVRAM, or reboot the PC. The UEFI PCI reader is an interface contract; the native firmware protocol binding is not implemented yet. ACPI parsing only accepts byte slices supplied by a caller and does not locate or map firmware tables itself.

Windows PnP inventory is useful for exercising the pipeline but can include virtual and software-defined devices. It is not independent proof of physical bus presence. Passing unit tests or CI is software validation only, not physical-hardware proof.

## Run the Windows inventory probe

From the repository root in PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\hardware-bridge-inventory.ps1 -SelfTest
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\hardware-bridge-inventory.ps1
```

Default report: `%TEMP%\accessibility-hardware-inventory.json`.

## Validation labels

- **Implemented:** source code and contract tests exist.
- **CI PASS:** only when the current commit's required jobs report success.
- **NOT_VALIDATED:** not yet run on the target physical machine.
- **NOT_IMPLEMENTED:** native UEFI protocol binding, physical PCI probing, hardware-backed accessibility output and resource authorization remain future work.

Do not label QEMU or another simulated environment as physical validation.
