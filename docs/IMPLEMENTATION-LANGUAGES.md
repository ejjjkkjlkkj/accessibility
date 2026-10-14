# Implementation languages: Go, C, Rust, D, Ada, Zig, Nim, and Swift

The accessibility project covers the full computing path: firmware and pre-boot, boot chain, kernel and drivers, operating systems, desktop, assistive technology, applications, web, deployment and recovery. No single language is expected to own every layer.

## Responsibilities

| Language | Primary responsibility | Initial implementation | Guardrails |
|---|---|---|---|
| C11 | Portable ABI and low-level adapters where hardware/OS interfaces require C | `components/core-c/`: versioned event structure and initialization routine | Validate pointers and bounds; platform drivers require hardware evidence |
| Rust | Shared typed core, safety-sensitive parsing/state logic, firmware/pre-boot components | `crates/accessibility-core/`: allocation-free event model with `no_std` | Unit tests do not prove firmware execution or physical behavior |
| Go | Catalog ingestion, audits, orchestration, reports and host tooling | `cmd/catalog-validator/`: CSV schema validator | Network errors are not proof a resource is unavailable |
| D | Systems utilities and performance-oriented host components where D's runtime/toolchain is acceptable | `components/core-d/`: event contract and unit tests | Do not assume a D runtime is suitable for firmware or kernels |
| Ada | Strongly typed components and contracts for safety-oriented logic | `components/core-ada/`: event validation package and assertions | This starter is not certified, safety-qualified, or a complete SPARK proof |
| Zig | Small systems components and explicit memory/layout work | `components/core-zig/`: event model and tests | Compiler-version-sensitive; no hardware claims without target tests |
| Nim | Readable tooling and native utilities | `components/core-nim/`: event validation and executable assertions | This starter uses the Nim runtime; it is not a `no_std` firmware component |
| Swift | Accessibility-aware application and service layers, especially Apple platforms | `components/core-swift/`: Swift package and XCTest coverage | Linux CI does not validate VoiceOver or Apple-platform integration |

## Interoperability direction

1. Keep the C ABI versioned and documented before binding other languages to it.
2. Keep platform-specific accessibility APIs behind adapters; a shared event model does not replace UI Automation, MSAA/IAccessible, AT-SPI, browser accessibility trees, or firmware interfaces.
3. Treat names, roles, states, focus, actions and errors as structured data. OCR is a fallback, not the primary semantic interface.
4. Select languages by target constraints: runtime availability, memory allocation, binary interface, licensing, compiler support and maintainability.
5. Use Rust `no_std` only where necessary. Do not assume Go, Swift, Nim or a D runtime belongs in firmware/kernel paths.
6. Preserve evidence labels: PASS, FAIL, PARTIAL, BLOCKED, NOT VALIDATED. QEMU success is not physical hardware proof.

## Validation

The `polyglot-ci.yml` workflow is configured to test Go, C11, Rust, D, Ada, Zig, Nim and Swift starter components. Until a workflow run is observed, these checks are **NOT VALIDATED**. Even a green CI run is software validation only; it does not establish screen-reader usability, accessibility conformance, certified safety, or physical hardware behavior.
