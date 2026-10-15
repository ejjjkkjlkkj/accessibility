# Accessibility Hardware Bridge

A small `no_std` Rust contract for the accessibility repository's hardware-facing architecture.

## Current implementation

- Typed PCI address metadata with bounds checks.
- Device descriptors distinguishing physical inventory, firmware-reported, and software-defined origins.
- Resource-window validation using checked arithmetic and half-open ranges.
- Resource overlap detection.
- A capability-free inventory entry for sharing hardware metadata with higher accessibility layers.

## Explicit non-goals in this increment

This crate does **not** probe PCI configuration space, map MMIO, access I/O ports, configure DMA, register interrupts, emulate PCI devices, drive audio, or change firmware/NVRAM. It contains no unsafe code and no operating-system dependency. Describing a resource does not authorize access to it.

## Validation status

- Unit tests are provided for metadata validation, PCI bounds, malformed identifiers, range overflow, overlap, and adjacency.
- These tests validate the data model only.
- Physical hardware discovery and I/O are **NOT_VALIDATED** until a separate platform adapter is implemented and tested on the target machine.
- Emulator results, if later added, must not be reported as physical-hardware proof.

## Next engineering increments

1. Add read-only discovery adapters for UEFI PCI I/O and ACPI/SMBIOS inventory, each behind an explicit platform feature.
2. Normalize discovered devices into these descriptors while preserving provenance and per-field confidence.
3. Add a resource authorization layer separate from inventory metadata.
4. Add accessibility event translation so device status, errors, and changes can be spoken or brailled by the screen-reader runtime.
5. Validate build/test matrices in CI; reserve physical proof for a documented run on real hardware.
