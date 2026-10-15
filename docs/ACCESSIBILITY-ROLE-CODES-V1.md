# Accessibility role codes — contract v1

Status: **IMPLEMENTED AS A CROSS-LANGUAGE ENUMERATION CONTRACT; OS ADAPTERS NOT IMPLEMENTED BY THIS DOCUMENT.**

This file is the canonical mapping for the logical role code shared by the C, Rust, Go, D, Ada, Nim, Swift, and Zig prototypes. Names are descriptive labels; the numeric values are the stable wire-level role identifiers.

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

Rules:
- Numeric values are explicit; never rely on compiler-assigned enum ordinals.
- unknown (0) is a valid fallback for unrecognized platform roles, not proof that an element is accessible.
- The logical role code is not, by itself, a complete binary ABI. String encoding, struct layout, ownership, state flags, event sequencing, and platform handles need explicit wire/FFI contracts. The Rust-only state-flag prototype is documented in `ACCESSIBILITY-STATE-FLAGS-V1.md`; it is not yet implemented consistently across all language bindings and is not a completed shared ABI.
- Cross-language validation must check every code above, including boundary values 0 and 17.
- Names transported across a future wire boundary must be UTF-8 and no longer than 4096 bytes. Implementations must reject oversize input rather than silently truncate it.
- Event names must also reject malformed UTF-8 and embedded NUL. The C length-aware `accessibility_event_init_n` enforces this explicitly; its legacy NUL-terminated wrapper follows the normal C-string boundary and cannot inspect bytes past the first NUL. Ada's current validator receives only a length, so it cannot validate the name bytes and remains a partial binding.
- Password values must not be announced. This role map does not authorize activation, editing, firmware access, network access, or other side effects.

Release gate: each language implementation must have tests asserting the same role-to-code mapping; native UI integration and screen-reader behavior require separate tests.
