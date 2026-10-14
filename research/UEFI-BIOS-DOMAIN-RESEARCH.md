# UEFI / BIOS / Firmware — Domain Research

**Status:** Initial research baseline  
**Repository:** `ejjjkkjlkkj/accessibility`  
**Method:** Public-web discovery; official sources prioritized. This is a curated starting inventory, **not** proof that every registered domain or TLD has been enumerated. Domain ownership, DNS, TLS, redirects, and current registration must be checked separately before classifying any domain.

## 1. Standards and specifications

| Domain | Resource | Purpose / notes |
|---|---|---|
| https://uefi.org/ | UEFI Forum | Main standards body and specifications portal. |
| https://uefi.org/specifications | Specifications | UEFI, ACPI, PI, UEFI Shell and related specifications. The page lists UEFI 2.11 and ACPI 6.6; verify revision status when using a spec. |
| https://uefi.org/specsandtesttools | Specifications and test tools | Standards and UEFI self-certification testing resources. |
| https://uefi.org/specs/access | Specs and tools access | Official access page; read licensing terms before redistribution or implementation. |
| https://uefi.org/sites/default/files/resources/UEFI_Spec_Final_2.11.pdf | UEFI 2.11 PDF | Primary reference for UEFI interfaces. |
| https://uefi.org/ | ACPI specification entry point | ACPI is maintained/published through the UEFI Forum ecosystem. |

## 2. Open firmware implementations and development

| Domain | Resource | Purpose / notes |
|---|---|---|
| https://www.tianocore.org/ | TianoCore | Community and documentation for EDK II, an open-source UEFI implementation/development environment. |
| https://github.com/tianocore/edk2 | EDK II source | Firmware development framework; platform hardware support varies. |
| https://www.coreboot.org/ | coreboot | Open-source firmware focused on hardware initialization; not synonymous with UEFI. |
| https://doc.coreboot.org/ | coreboot documentation | Architecture, supported platforms, payloads and firmware documentation. |
| https://github.com/tianocore/edk2-platforms | EDK II platform packages | Platform-specific EDK II components. |
| https://github.com/qemu/qemu | QEMU | Emulator/virtualizer useful for simulated firmware testing; QEMU results do not prove physical-hardware behavior. |

## 3. Firmware security and assessment

| Domain | Resource | Purpose / notes |
|---|---|---|
| https://csrc.nist.gov/ | NIST Computer Security Resource Center | Security standards and firmware guidance. |
| https://csrc.nist.gov/pubs/sp/800/193/final | NIST SP 800-193 | Platform firmware resiliency: protect, detect, and recover. |
| https://csrc.nist.gov/pubs/sp/800/147/final | NIST SP 800-147 | BIOS protection guidelines for client systems. |
| https://csrc.nist.gov/pubs/sp/800/147/b/final | NIST SP 800-147B | BIOS protection guidance for servers. |
| https://chipsec.github.io/ | CHIPSEC documentation | Platform and firmware security assessment toolkit. Its documentation warns to use it on test systems only; do not run invasive modules on production hardware. |
| https://nvd.nist.gov/ | National Vulnerability Database | CVE records and vulnerability references; vendor advisories should also be checked. |
| https://www.cisa.gov/known-exploited-vulnerabilities-catalog | CISA KEV catalog | Known-exploited vulnerability cross-check. |

## 4. OS and platform integration references

| Domain | Resource | Purpose / notes |
|---|---|---|
| https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/oem-uefi | Microsoft UEFI requirements | Windows platform and OEM firmware requirements. |
| https://www.intel.com/content/www/us/en/developer/articles/tool/unified-extensible-firmware-interface.html | Intel UEFI resources | UEFI training and technical materials. |
| https://www.amd.com/ | AMD | Processor/platform documentation and OEM support entry point; specific firmware resources vary by product. |

## 5. OEM and firmware-vendor domain leads

These are vendor entry points, **not** confirmation that firmware binaries are openly available for every model:

- https://www.asus.com/
- https://www.dell.com/
- https://support.hp.com/
- https://support.lenovo.com/
- https://www.acer.com/
- https://www.msi.com/
- https://www.gigabyte.com/
- https://www.ami.com/ (AMI firmware vendor)
- https://www.insyde.com/ (Insyde firmware vendor)
- https://www.phoenix.com/ (Phoenix Technologies)

For each OEM, research must record exact model, board/platform ID, firmware version, release date, advisory, signature/hash where published, and supported update/recovery method. Never flash firmware based only on a domain name or a similar model number.

## 6. Research taxonomy to expand

The next discovery passes should cover:

1. Standards: UEFI, PI, ACPI, SMBIOS/DMTF, PCI/PCIe, USB, NVMe, TPM/TCG, Secure Boot, measured boot.
2. Implementations: EDK II, coreboot, oreboot, LinuxBoot, U-Boot, OpenSBI, Slim Bootloader, vendor firmware stacks.
3. Tools: firmware image parsing, SPI/flash analysis, UEFI conformance tests, static analysis, SBOM and reproducible-build tooling.
4. Security: NIST SP 800-193, Secure Boot revocation/DBX, capsule updates, rollback protection, recovery, CVEs and vendor advisories.
5. Architectures: x86/x64, ARM/AArch64, RISC-V, servers, laptops, embedded and virtual platforms.
6. Accessibility: pre-boot keyboard navigation, screen-reader/voice output, serial/debug access, UEFI HII forms and firmware setup accessibility.
7. Domains and trust: authoritative source, ownership, DNS/TLS status, redirect target, archive evidence, official contact/security policy, and whether a domain is actually affiliated with the project/vendor.

## 7. Evidence and safety rules

- Label every record **VERIFIED OFFICIAL**, **RELEVANT THIRD-PARTY**, **UNVERIFIED**, or **SUSPICIOUS** with evidence and date checked.
- Do not infer that `uefi.<TLD>` domains are official merely because the name matches. Verify ownership and links from the UEFI Forum.
- Keep domain discovery separate from vulnerability testing and firmware modification.
- Prefer read-only research and downloads from official vendor/project pages.
- QEMU/OVMF or another emulator validates a simulated environment only; physical-device validation remains a separate requirement.
- Do not modify BIOS/UEFI settings, NVRAM, SPI flash, or perform firmware updates as part of this research.

## Initial source evidence

- UEFI Forum specifications portal: https://uefi.org/specifications
- UEFI Forum specifications and tools: https://uefi.org/specsandtesttools
- TianoCore overview: https://www.tianocore.org/
- coreboot FAQ explaining the distinction between UEFI and coreboot: https://doc.coreboot.org/getting_started/faq.html
- NIST platform firmware resiliency guidance: https://csrc.nist.gov/pubs/sp/800/193/final
- CHIPSEC documentation and safety warning: https://chipsec.github.io/
