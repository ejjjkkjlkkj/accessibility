# Accessibility semantic state flags — prototype v1

**Implementation status:** validators and constants are now present in Rust, C, D, Ada, Go, Nim, Swift, and Zig source prototypes. Cross-language behavior is still **NOT VALIDATED** until native language jobs complete; physical-hardware behavior and native OS accessibility adapters are also **NOT VALIDATED**.

This document records the shared semantics under test. It does not claim a complete binary ABI, firmware interface, or native OS accessibility adapter.

## Bit assignments

| Bit | Constant | Meaning |
|---:|---|---|
| 0 | `STATE_FOCUSED` | The node currently owns logical keyboard focus. |
| 1 | `STATE_FOCUSABLE` | The node may receive keyboard focus. |
| 2 | `STATE_ENABLED` | The node is enabled for interaction. |
| 3 | `STATE_DISABLED` | The node is disabled for interaction. |

Language-specific constant naming follows each language's conventions. Bits 4–63 are reserved and rejected, not silently ignored.

## Validation rules

- `STATE_ENABLED` and `STATE_DISABLED` may not be set together.
- `STATE_FOCUSED` requires both `STATE_FOCUSABLE` and `STATE_ENABLED`, and forbids `STATE_DISABLED`.
- A validated semantic tree may contain at most one focused node.
- A focus target must be focusable, enabled, and not disabled.
- Navigation is bounded by the supplied node slice and does not wrap around.
- Rust tree validation rejects duplicate IDs, missing parents, multiple roots, cycles, names exceeding 4096 UTF-8 bytes, and invalid state combinations.

## Safety and implementation boundary

These flags describe semantic state only. They do not authorize input injection, activation, text entry, network access, firmware changes, or speech output. Callers must validate externally supplied trees before navigation.

The Rust implementation uses no heap allocation in its semantic-tree validation and navigation routines, and the crate forbids `unsafe` code. This is source-level scope, not proof of timing behavior on every target. The remaining parent-chain verification cost is not yet certified for arbitrarily large trees.

## Required next validation

Run unit tests and native builds for every language binding on the supported host platforms. The role code remains separate from a complete binary ABI: struct layout, UTF-8 handling, ownership, event sequencing, and platform handles need explicit contracts. Native accessibility adapters and physical-device behavior remain separate release gates.
