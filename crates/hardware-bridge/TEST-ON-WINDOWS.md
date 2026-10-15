# Test the physical-inventory bridge on Windows

## Scope and evidence boundary

This is a first runnable host-side adapter for the accessibility hardware bridge. It queries Windows Plug and Play metadata through CIM in read-only mode. It does **not** directly enumerate PCI configuration space, map ECAM/MMIO, access I/O ports, perform DMA, change firmware, or reboot the PC.

**Evidence label:** `OS_REPORTED_NOT_PHYSICALALLY_VERIFIED`. Windows-reported devices are useful for testing the inventory pipeline, but are not proof that each device was independently observed on the physical bus. Virtual and software-defined devices may appear in the OS inventory.

## Run

Open PowerShell (standard user is sufficient), then run from the repository root:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\hardware-bridge-inventory.ps1 -SelfTest
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\hardware-bridge-inventory.ps1
```

The JSON report is written by default to `%TEMP%\accessibility-hardware-inventory.json`. To choose another path:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\tools\hardware-bridge-inventory.ps1 -OutputPath "$env:TEMP\bridge-test.json"
```

Expected self-test result: `SELFTEST PASS`. The inventory run prints the number of Windows PnP records and the report path. Exit code is non-zero if collection fails.

## Validate the JSON

```powershell
$r = Get-Content "$env:TEMP\accessibility-hardware-inventory.json" -Raw | ConvertFrom-Json
$r | Select-Object SchemaVersion, ReadOnly, EvidenceLevel, DeviceCount
$r.Devices | Select-Object Name, PnpClass, PnpDeviceId, Status, ConfigManagerErrorCode | Format-Table -AutoSize
```

## Status

- Implemented: read-only Windows PnP metadata collector, JSON output, basic self-test.
- Not validated until executed: actual output on the target PC.
- Not implemented here: native UEFI PCI protocol adapter, ACPI table acquisition from firmware, direct PCI configuration access, and hardware-backed screen-reader event delivery.
