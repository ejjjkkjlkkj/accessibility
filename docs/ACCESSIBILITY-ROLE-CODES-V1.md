# Accessibility role codes — contract v1

Status: **IMPLEMENTED AS A CROSS-LANGUAGE ENUMERATION CONTRACT; OS ADAPTERS NOT IMPLEMENTED BY THIS DOCUMENT.**

This file is the canonical mapping for the logical role code shared by the C, Rust, Go, D, Ada, Nim, Swift, and Zig prototypes. Names are descriptive labels; the numeric values are the stable logical role identifiers.

| Code | Role |
|---:|---|
| 0 | unknown |
| 1 | application |
| 2 | window |
| 3 | group |
| 4 | text |
| 5 | button |
| 6 | checkbox |
| 7 | radio |
| 8 | textbox |
| 9 | link |
| 10 | heading |
| 11 | list |
| 12 | list_item |
| 13 | table |
| 14 | row |
| 15 | cell |
| 16 | menu |
| 17 | menu_item |

## Logical contract and wire-format limits

- Numeric values are explicit; never rely on compiler-assigned enum ordinals.
- `unknown` (0) is a valid fallback for unrecognized platform roles, not proof that an element is accessible.
- The logical role code is not, by itself, a complete binary ABI. String encoding, struct layout, ownership, state flags, event sequencing, and platform handles need their own explicit wire/FFI contracts.
- Cross-language validation must check every code above, including boundary values 0 and 17.
- Names transported across a wire boundary must be valid UTF-8 and no longer than 4096 bytes. Implementations must reject oversized or malformed input rather than silently truncate it.
- The C `accessibility_event_init` contract returns `-1` for null arguments, `-2` for an invalid role, `-3` for a name that exceeds the fixed capacity or has no NUL terminator within it, and `-4` for malformed UTF-8.
- The no_std Rust semantic-tree validator currently accepts at most 512 nodes and at most 64 parent links between a node and its root. These are deliberate resource bounds for externally supplied trees; exceeding either limit is invalid.
- Password values must not be announced. This role map does not authorize activation, editing, firmware access, network access, or other side effects.

## Validation boundary

A role map and a valid tree are necessary data-model checks, not a complete screen reader. Release gates must separately test keyboard/input adapters, focus/state correctness, speech output, native UI integration, privacy handling, and hardware compatibility. QEMU or another emulator can establish simulation evidence only; it does not establish physical hardware success.
