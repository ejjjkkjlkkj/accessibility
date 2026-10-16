# Release gates for PC testing

This file defines what may be delivered for the user's Windows PC. A release is a tested artifact, not a roadmap, source-only prototype, or green documentation check.

## Release channels

1. **Native screen-reader release**: runnable host application for each supported OS/language combination. It must obtain real semantic accessibility data from the OS and deliver speech through a connected, tested backend.
2. **UEFI release**: signed or clearly unsigned EFI application package, with architecture and firmware requirements documented. It must never be described as a standalone OS.
3. **OS/kernel release**: bootable image with a documented boot path and supported target. A kernel stub or ISO containing only an EFI app is not an OS release.

## Release artifacts required

For every release candidate, publish:
- version, source commit, target architecture, toolchain versions and build commands;
- artifact names, sizes and SHA-256 hashes;
- automated test results and logs;
- supported/unsupported feature matrix;
- known issues, recovery instructions and rollback instructions;
- emulator results separated from physical-machine results.

## Windows PC test path

The initial physical target is x86_64 UEFI on the user's ASUS VivoBook. That target is not assumed compatible merely because the architecture matches.

Safe staged validation:
1. Host-side unit tests and static checks.
2. Reproducible EFI build and PE/COFF inspection.
3. QEMU/OVMF test, explicitly labeled emulator-only.
4. Create a test USB image only after the prior gates pass. Keep the user's current Windows disk and firmware settings untouched.
5. Physical boot test only with explicit user approval, a verified recovery path and a backup. Never automate a firmware change, flash, or reboot.

## Hard release gates

- No fabricated PASS status or test output.
- No claim of audible speech until audio is heard or captured on the stated target.
- No claim of UEFI compatibility from a host build alone.
- No claim of an OS release from a UEFI application or kernel entry stub.
- Do not package an unbootable image as a release.
- Do not overwrite the user's Windows ISO, boot files, EFI System Partition, or NVRAM entries.
- Screen-reader release requires real semantic focus/navigation plus connected speech; a text-formatting core alone is not releasable as a complete screen reader.

## Current gate status

The repository's current eight portable cores are PARTIAL. The C UEFI prototype and Rust UEFI path are not validated as complete UEFI releases. Seven other UEFI tracks are not implemented, and no complete bootable OS is established for any of the eight languages. Therefore there is currently no verified 24-component release and no artifact that can honestly be presented as the finished product for physical testing.
