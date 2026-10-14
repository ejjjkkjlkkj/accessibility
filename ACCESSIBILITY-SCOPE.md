# Accessibility — Unified Scope

## Mission

Make accessibility a cross-layer property of the entire computing experience: firmware and pre-boot setup, bootloader, operating system, drivers, desktop, applications, web, development tools, installers, recovery environments, and connected services. UEFI/BIOS is one layer of the mission, not the whole project.

## Core objective

A blind, low-vision, keyboard-only, or otherwise disabled user should be able to install, boot, configure, operate, recover, and update a system as independently as possible. Accessibility must not depend solely on a particular desktop screen reader or on visual OCR.

## Scope map

| Layer | Areas to research and validate | Accessibility expectations |
|---|---|---|
| Firmware / pre-boot | UEFI, BIOS setup, UEFI HII forms, boot menu, Secure Boot dialogs, firmware updates and recovery | Keyboard-only operation, meaningful focus, programmatic labels/states where interfaces permit, speech/audio or accessible external console path, no silent destructive actions |
| Boot chain | Boot managers, bootloaders, kernel handoff, recovery and diagnostics | Readable status/errors, keyboard control, audible or alternative accessible feedback, timeout control |
| Hardware interfaces | ACPI, SMBIOS, PCI/PCIe, USB, HID, audio, display/GOP, storage, network, TPM | Stable discovery and accessible status/error reporting; hardware-specific limits documented |
| Operating systems | Windows, Linux, BSD and other relevant OS families | Native accessibility APIs, accessible setup/OOBE, login, recovery, update and security dialogs |
| Kernel and drivers | Input, display, audio, USB/HID, graphics, accessibility API bridges | Reliable events, focus/state propagation, diagnostics and failure reporting |
| Desktop / shell | File managers, settings, taskbars, notifications, system menus | Keyboard-first interaction, logical focus, names/roles/states, scaling, contrast, speech/Braille compatibility |
| Assistive technology | NVDA, JAWS, Narrator, Orca, magnification, Braille, speech input, switch access | Interoperability, documented APIs, regression testing and no unnecessary AT-specific assumptions |
| Applications | Native, Electron, Java, .NET, Qt, GTK, office, developer tools and installers | Semantic controls, keyboard access, focus management, accessible errors and progress |
| Web | Browsers, web apps, HTML, ARIA, WCAG, PDFs and media | Standards-based semantics, keyboard access, captions, structure, zoom and AT interoperability |
| Development / deployment | IDEs, terminals, build systems, CI/CD, installers, packaging, update channels | Accessible workflows, machine-readable logs, reproducible builds, accessible documentation |
| Security / recovery | Authentication, Secure Boot, encryption prompts, firmware updates, diagnostics and recovery keys | Accessible security prompts without weakening security; accessible recovery procedures |
| AI / automation | Local speech, language models, vision/OCR as fallback, agent workflows | Local-first options, privacy, transparent uncertainty, keyboard control and nonvisual confirmations |
| Standards / policy | WCAG, WAI-ARIA, EN 301 549, applicable regulations and platform accessibility specifications | Map each requirement to implementation evidence and test results |
| Domain and source ecosystem | Official standards bodies, OS vendors, firmware vendors, open-source projects, accessibility communities, issue trackers and documentation | Verify domain ownership/affiliation, authoritative status, licensing, security advisories and accessibility resources |

## Research workflow

1. Enumerate relevant technologies, standards bodies, vendors, projects, repositories, documentation portals, test tools and accessibility communities across all layers.
2. Separate official domains from third-party resources; do not infer legitimacy from a domain name alone.
3. Record each resource with canonical domain, organization/project, scope, source URL, evidence, date checked, official-status confidence, accessibility relevance, license and security contact/advisory page where applicable.
4. Map accessibility requirements to concrete interfaces, components, test cases and evidence.
5. Run read-only audits first. Do not flash firmware, alter NVRAM, change security settings, publish releases or merge changes without explicit authorization.
6. Report outcomes as PASS, FAIL, PARTIAL, BLOCKED or NOT VALIDATED. Never treat an emulator test as proof of physical hardware behavior.
7. Maintain separate evidence for keyboard-only operation, screen-reader speech, Braille, magnification, contrast, cognitive accessibility and alternative input.
8. Prioritize barriers that prevent independent installation, boot, login, security configuration, recovery or system updates.

## Baseline standards and sources

- UEFI Forum specifications: https://uefi.org/specifications
- TianoCore / EDK II: https://www.tianocore.org/
- coreboot documentation: https://doc.coreboot.org/
- W3C WCAG overview: https://www.w3.org/WAI/standards-guidelines/wcag/
- W3C WAI-ARIA overview: https://www.w3.org/WAI/standards-guidelines/aria/
- U.S. Access Board ICT standards: https://www.access-board.gov/ict/
- French RGAA reference resources: https://accessibilite.numerique.gouv.fr/ressources/references/
- NIST platform firmware resiliency: https://csrc.nist.gov/pubs/sp/800/193/final
- CHIPSEC documentation: https://chipsec.github.io/

## Current status

- Scope defined at architecture level.
- This file is a research and planning baseline, not proof of conformance.
- A complete worldwide enumeration of all relevant domains and projects remains NOT VALIDATED.
- Actual accessibility requires testing on each supported firmware, OS, hardware platform, application and assistive-technology combination.
