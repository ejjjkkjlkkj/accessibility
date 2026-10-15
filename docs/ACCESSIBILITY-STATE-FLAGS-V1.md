# Accessibility semantic state flags — prototype v1

**Status:** implemented in the Rust `no_std` semantic core only; cross-language parity, native OS adapters, and physical-hardware behavior are **NOT VALIDATED**.

This document records the state semantics currently enforced by `crates/accessibility-core`. It does not claim a complete C ABI, firmware interface, or cross-language state contract.

## Bit assignments

| Bit | Constant | Meaning |
|---:|---|---|
| 0 | `STATE_FOCUSED` | The node currently owns logical keyboard focus. |
| 1 | `STATE_FOCUSABLE` | The node may receive keyboard focus. |
| 2 | `STATE_ENABLED` | The node is enabled for interaction. |
| 3 | `STATE_DISABLED` | The node is disabled for interaction. |

Bits 4–63 are reserved in this prototype and are rejected, not silently ignored.

## Validation rules

- `STATE_ENABLED` and `STATE_DISABLED` may not be set together.
- `STATE_FOCUSED` requires both `STATE_FOCUSABLE` and `STATE_ENABLED`, and forbids `STATE_DISABLED`.
- A validated semantic tree may contain at most one focused node.
- A focus target must be focusable, enabled, and not disabled.
- Navigation is bounded by the supplied node slice and does not wrap around.
- Tree validation rejects duplicate IDs, missing parents, multiple roots, cycles, names exceeding 4096 UTF-8 bytes, and invalid state combinations.

## Safety and implementation boundary

These flags describe semantic state only. They do not authorize input injection, activation, text entry, network access, firmware changes, or speech output. Callers must validate externally supplied trees before navigation.

The current Rust implementation uses no heap allocation in its semantic-tree validation and navigation routines, and the crate forbids `unsafe` code. This is source-level scope, not proof of timing behavior on every target. The remaining parent-chain verification cost is not yet certified for arbitrarily large trees.

## Required next validation

Before treating these bits as a cross-language wire contract, each C, D, Ada, Go, Nim, Swift, and Zig binding must implement identical constants and invalid-state tests; the shared contract validator must check parity. Native accessibility adapters and physical-device behavior remain separate release gates.
