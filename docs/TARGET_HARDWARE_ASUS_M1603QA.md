# Physical Target Hardware Profile — ASUSTeK VivoBook M1603QA

**Status: TARGET_HARDWARE_SPECIFIED / NOT_PHYSICALLY_TESTED_IN_CI**

This document specifies the target physical PC hardware configuration for bare-metal ZERO OS, UEFI boot, screen reader, and hardware accessibility verification.

---

## 1. Physical System Profile

- **Manufacturer**: ASUSTeK COMPUTER INC.
- **Model / SKU**: VivoBook_ASUSLaptop M1603QA (x64 PC)
- **Baseboard**: ASUSTeK M1603QA v1.0
- **CPU**: AMD Ryzen 7 5800H with Radeon Graphics (Zen 3, 8 Cores / 16 Threads, 3.20 GHz)
- **Installed Memory**: 40.0 GB DDR4
- **Firmware**: American Megatrends International (AMI) UEFI v3.3 (Build M1603QA.308, 22/05/2023)
- **Boot Mode**: UEFI Native
- **Secure Boot State**: Disabled (Désactivé) — Allows unsigned bare-metal ZERO OS kernel & EFI execution
- **Display / Graphics**: AMD Radeon Graphics (Cezanne / Zen 3 iGPU) via UEFI Graphics Output Protocol (GOP)
- **Audio Controller**: AMD / Realtek High Definition Audio Controller (Azalia / HD Audio MMIO)
- **Input Devices**: USB / I2C / HID Keyboard & Touchpad

---

## 2. Bare-Metal ZERO OS Target Mapping

1. **UEFI Boot Phase**:
   - Binary Target: `EFI/BOOT/BOOTX64.EFI` (PE/COFF 64-bit x86_64).
   - Protocol Dependencies: `EFI_GRAPHICS_OUTPUT_PROTOCOL` (GOP), `EFI_SIMPLE_TEXT_INPUT_PROTOCOL` (ConIn/Keyboard).
   - Execution Entry: Boot directly into ZERO OS Kernel without Windows/Linux host runtime.

2. **CPU Architecture & ISA Execution**:
   - Instruction Set: x86_64 Long Mode (64-bit native).
   - ZERO-ISA 64-bit Translation: 64-bit GPRs (`r0-r31`), 512-bit SIMD registers (`t0-t31`) utilizing AVX2/FMA instructions on AMD Zen 3 cores for AI Tensor acceleration (`TMATMUL`, `TVGELU`, `TVATTEN`).

3. **Hardware Accessibility Channel (HAC)**:
   - Text Output: GOP Framebuffer accessible console.
   - Audio Pipeline: HD Audio Controller MMIO base register direct mapping for PCM audio announcements.
   - Keyboard Navigation: HID / Scancode 0x0F (Tab) focus cycle across semantic nodes.

---

## 3. Physical Validation Rules & Gate Status

- **CI Emulation / Build Gate**: `PASS` (C freestanding build & tests succeed).
- **Physical Hardware Execution Gate**: `NOT_VALIDATED` (Requires physical USB flash drive boot on the target ASUS M1603QA PC).
- **Rule**: Do not present unit test or emulator results as physical hardware proof.
