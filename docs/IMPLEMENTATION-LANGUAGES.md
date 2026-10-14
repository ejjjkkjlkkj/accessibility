# Implementation languages: Go, C, and Rust

The accessibility project covers the full computing path: firmware and pre-boot, boot chain, kernel and drivers, operating systems, desktop, assistive technology, applications, web, deployment and recovery. No single language is expected to own every layer.

## Responsibilities

| Language | Primary responsibility | Initial implementation | Guardrails |
|---|---|---|---|
| C11 | Small, portable ABI and low-level adapters where hardware/OS interfaces require C | `components/core-c/`: versioned accessibility event structure and initialization routine | Keep interfaces narrow; validate pointers and bounds; add platform-specific drivers only with hardware evidence |
| Rust (`no_std` where appropriate) | Shared typed core, safety-critical parsing/state logic, firmware/pre-boot components | `crates/accessibility-core/`: allocation-free event data model | No claim of firmware execution or physical hardware validation from unit tests |
| Go | Catalog ingestion, audits, orchestration, report generation and host-side tooling | `cmd/catalog-validator/`: standard-library CSV schema validator | Network checks remain read-only; report uncertainty rather than treating a timeout as proof a domain is dead |

## Interoperability direction

1. Keep the C ABI versioned and documented before binding Rust or Go to it.
2. Keep platform-specific operating-system accessibility APIs behind adapters; do not pretend one event schema replaces UI Automation, MSAA/IAccessible, AT-SPI, browser accessibility trees, or firmware-specific interfaces.
3. Treat names, roles, states, focus, actions and errors as structured data. OCR is a fallback, not the primary semantic interface.
4. Use Go for repository/catalog operations, not as a replacement for kernel or firmware code.
5. Use Rust `no_std` only for components that genuinely need it; host-side tools may use the standard library.
6. Preserve evidence labels: PASS, FAIL, PARTIAL, BLOCKED, NOT VALIDATED. QEMU success is not physical hardware proof.

## Validation

The polyglot CI workflow runs Go unit tests and catalog validation, CMake build/tests, and Rust tests. CI is software validation only; it does not establish screen-reader usability, accessibility conformance, or physical hardware behavior.
