# Dark Team — architecture and implementation plan

Status: **PARTIAL / NOT VALIDATED**. This document defines scope and gates; it is not evidence that the listed capabilities have been implemented or tested.

## Mission

The Dark Team is the project's combined threat-research, adversarial-testing and defensive-engineering track. It supports the accessibility stack from physical hardware and firmware through kernel/runtime, operating systems, screen readers, applications and network services. “Dark” includes research into Surface Web, Deep Web, Dark Web and overlay networks; it does not mean unauthorized access or deployment of offensive tools against third parties.

The project must build its own integration and accessibility technology where feasible, while reusing standards and auditing existing open-source components. Do not confuse research/inventory with implementation, simulation with physical proof, or a test plan with a passed test.

## Workstreams

| ID | Workstream | Required outputs | Initial status |
|---|---|---|---|
| DT-01 | Threat model and asset inventory | Assets, trust boundaries, abuse cases, owners, evidence and residual risks | NOT_VALIDATED |
| DT-02 | Authorized Red Team | Isolated test fixtures, negative tests, fuzz targets, reproducible findings and safe regression cases | NOT_VALIDATED |
| DT-03 | Blue Team and response | Detection rules, audit events, containment guidance, recovery tests and false-positive review | NOT_VALIDATED |
| DT-04 | Hardware and silicon boundary | Read-only PCI/device inventory; DMA/IOMMU, MMIO, interrupt and bus threat models; explicit privilege boundaries | PARTIAL |
| DT-05 | UEFI and firmware | Image/measurement provenance, Secure Boot verification where supported, ACPI parsing bounds, firmware update/rollback threat model | NOT_VALIDATED |
| DT-06 | Core/runtime and memory | Rust no_std boundary review, unsafe-code inventory, bounds/overflow checks, parser hardening and panic/failure policy | NOT_VALIDATED |
| DT-07 | Accessibility attack surface | UIA/AT-SPI/event spoofing, focus hijack, keyboard injection, speech-output privacy, braille routing and screen-reader API abuse tests | NOT_VALIDATED |
| DT-08 | Network and overlays | Protocol threat models for DNS/PKI, IP/link layers, Tor, I2P, IPFS, Hyphanet/Freenet, GNUnet and mesh/overlay systems; privacy and accessibility review | NOT_VALIDATED |
| DT-09 | Surface/Deep/Dark Web research | Public-source research, source provenance, confidence labels, safe collection and lawful handling of findings | NOT_VALIDATED |
| DT-10 | Supply chain and provenance | Dependency inventory/SBOM, license and vulnerability checks, pinned actions, artifact hashes and build provenance | NOT_VALIDATED |
| DT-11 | Digital forensics and CTI | Evidence handling, timestamps, hashes, indicators with source/confidence, retention and privacy controls | NOT_VALIDATED |
| DT-12 | Cross-platform labs | Reproducible tests for Windows, Linux and other declared targets; separate simulator, hosted-runner and physical-hardware results | PARTIAL |
| DT-13 | Accessibility of security tools | Keyboard-only operation, accessible status/state/focus, screen-reader smoke tests and accessible logs/reports | NOT_VALIDATED |
| DT-14 | Governance and release gates | Scope authorization, least privilege, secret handling, report templates, risk acceptance and release blockers | PARTIAL |

## Layer coverage

Coverage must be tracked across all applicable layers, not only applications:

1. Physical device, silicon, buses and peripherals.
2. Boot ROM / UEFI / firmware / ACPI / Secure Boot.
3. Memory, interrupts, DMA/IOMMU and device access.
4. Kernel, drivers, system services and runtime.
5. Accessibility APIs, semantic trees, focus, keyboard, speech and braille.
6. Applications, browser/document engines and user data.
7. Network stack, DNS/PKI, public web and private/restricted/overlay networks.
8. Build tools, source control, CI runners, dependencies, signing and artifact distribution.
9. Research/forensics data collection, storage, access, retention and provenance.

A layer is not covered just because it appears in this table. Every layer needs testable requirements, an implementation reference, tests, evidence and known gaps.

## Operating rules

- Test only systems owned by the project or explicitly authorized for the stated scope. Offensive tests run in local fixtures, dedicated labs or approved targets.
- No credential theft, persistence, stealth, destructive payloads, unauthorized scanning, exploitation of third parties, or exfiltration. Research findings should be converted into safe reproductions and defensive regressions.
- Do not run untrusted pull-request code on a privileged self-hosted runner. Do not expose secrets to untrusted workflows. Prefer hosted runners for untrusted contributions and isolate local runners.
- Default to read-only hardware discovery. No BIOS flashing, firmware writes, NVRAM mutation, device writes, or automatic reboot.
- No QEMU, Hyper-V, VirtualBox or other emulator result may be reported as physical-hardware proof. If a simulator is used for a separate test, label it SIMULATION.
- Do not claim that the project has implemented Tor/I2P/IPFS or a hardware security feature merely because its threat model or adapter contract exists.
- Minimize personal data and sensitive telemetry. Redact secrets, identifiers and unrelated device details from reports.
- AI/model output is untrusted input: it must not directly issue privileged hardware, firmware, network or system actions. Enforce authorization and validation in deterministic code.
- Keep source URL, retrieval time, hash, license/usage basis, confidence and reviewer for external research. Do not download or execute untrusted artifacts simply because they were discovered during research.

## Evidence contract

Every finding and test record must include:
- unique ID and workstream;
- target, version/commit and exact scope;
- preconditions and reproducible steps;
- expected and actual result;
- logs/artifact reference and SHA-256 where relevant;
- evidence class: STATIC_ANALYSIS, UNIT_TEST, HOSTED_CI, LOCAL_RUNNER, SIMULATION, or PHYSICAL_HARDWARE;
- status: PASS, FAIL, PARTIAL, BLOCKED, or NOT_VALIDATED;
- limitations, severity, confidence, remediation and retest result.

PASS applies only to the stated test and target. A successful build is not proof of runtime behavior. OS-reported hardware inventory is not physical verification. A test that did not run is NOT_VALIDATED, not PASS.

## Implementation sequence

### Gate 0 — Establish scope and safety
1. Map repository modules, workflows, permissions, secrets and runner labels.
2. Identify exposed attack surfaces and trust boundaries.
3. Record threat scenarios and acceptance criteria.
4. Confirm no workflow routes untrusted pull requests to the self-hosted runner.

### Gate 1 — Safe automated baseline
1. Run existing tests and preserve their real output.
2. Add dependency/SBOM and static-analysis checks appropriate to each language.
3. Add parser boundary tests and fuzz targets for untrusted binary structures such as ACPI/PCI metadata.
4. Add negative tests for permission denial, absent devices, malformed events and invalid model output.
5. Pin third-party actions to reviewed immutable commit SHAs before treating CI as hardened.

### Gate 2 — Defensive instrumentation
1. Define a minimal structured audit-event schema with no secrets by default.
2. Add integrity/provenance checks for generated artifacts.
3. Define detection and response tests for tampering, malformed input and unexpected device state.
4. Test that logging failures do not disclose secrets or silently turn an unsafe operation into success.

### Gate 3 — Accessibility and physical evidence
1. Run keyboard-only and screen-reader tests for security reports and tools.
2. Run Windows/Linux CI and the controlled local runner only for trusted workflows.
3. Perform read-only physical inventory and label each source's trust level.
4. Validate hardware-specific behavior manually with recorded device/firmware details; do not infer it from CI or emulation.

### Gate 4 — Release decision
Release each module independently. Required: passing module tests, reviewed findings, provenance, reproducible artifacts, documented gaps and no unresolved critical/high issue without an explicitly recorded risk decision.

## First concrete deliverables

- This scope and evidence contract.
- A machine-readable workstream manifest in projects/dark-team/manifest.json.
- CI validation of the manifest and Markdown links.
- Follow-up implementation tasks for parser fuzzing, dependency analysis, artifact provenance and accessible security-report output.

The workstream manifest is an inventory of intended coverage, not a claim that those workstreams are implemented.
