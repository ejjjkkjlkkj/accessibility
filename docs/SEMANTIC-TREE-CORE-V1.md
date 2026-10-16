# Semantic accessibility tree core v1

## Status

- Rust `no_std` semantic node model: **IMPLEMENTED**.
- Structural validation and bounded focus traversal: **IMPLEMENTED; CI validation pending for this commit**.
- Windows UI Automation, Linux AT-SPI, macOS Accessibility API, Android Accessibility, screen capture, keyboard hooks, speech and braille adapters: **NOT IMPLEMENTED by this module**.
- UEFI/physical firmware behavior: **NOT VALIDATED**.

## Data model

`crates/accessibility-core/src/tree.rs` defines a borrowed `AccessibleNode` with a stable numeric ID, optional parent ID, canonical role, state flags and UTF-8 name. It uses no heap allocation and no operating-system API, so it can be reused as a platform-neutral core. A caller must supply the node slice.

State bits currently defined:
- bit 0: focused
- bit 1: focusable
- bit 2: enabled
- bit 3: disabled

A node is eligible for keyboard focus traversal only when both `focusable` and `enabled` are set and `disabled` is not set. Unknown state bits and simultaneous `enabled`/`disabled` are rejected; `focused` requires a focusable, enabled, non-disabled node, and at most one node may be focused. This is a logical rule; it does not move the real operating-system focus.

## Validation guarantees

`validate_tree` rejects an empty slice, more than 512 nodes, duplicate IDs, anything other than exactly one root, self-parenting, missing parents, ancestry cycles, parent chains deeper than 64 links, names longer than 4096 UTF-8 bytes, and names containing an embedded NUL. Validation is allocation-free and iterative. Nodes may appear in any slice order; the validator does not impose parent-before-child ordering. These fixed bounds limit work on untrusted input.

`next_focusable` and `previous_focusable` traverse in slice order without wrapping. They return an ID only; they do not dispatch input, announce a name, or modify focus state. Callers must validate untrusted node slices before traversal.

## Verification

The Rust unit tests cover valid tree traversal, disabled-node exclusion, navigation boundaries, unknown cursor IDs, duplicate IDs, missing parents, cycles, empty trees, oversized and NUL-containing names, invalid/unknown state flags, multiple focused nodes, and node-count/depth limits. Hosted CI must pass for the exact commit before these tests are marked PASS.

## Next required implementation

1. Add bounded event sequencing and change notifications to the platform-neutral core.
2. Implement and test OS-specific adapters independently (Windows UIA; Linux AT-SPI; macOS Accessibility API).
3. Add speech/braille output behind explicit interfaces.
4. Validate with real applications and screen readers; keep emulator tests separate from physical UEFI proof.
