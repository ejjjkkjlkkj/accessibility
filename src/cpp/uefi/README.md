# C++ UEFI application — feasibility gate

Status: NOT_VALIDATED. This directory deliberately does not claim an EFI executable exists.

Next implementation gate:
1. Pin an EDK2 or cross-compiler toolchain and target architecture.
2. Build a PE/COFF EFI application with the firmware ABI and entry point.
3. Enumerate ConsoleIn/ConsoleOut and GOP, handle absent protocols, and test in OVMF/QEMU.
4. Record exact commands, toolchain version, EFI artifact SHA-256 and emulator logs.
5. Keep audio BLOCKED until a device-specific path is demonstrated. Do not write NVRAM or reboot physical hardware from CI.
