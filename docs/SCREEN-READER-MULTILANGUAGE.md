# Screen-reader core across eight languages

Status: **PARTIAL implementation; platform integration NOT VALIDATED**.

This slice implements the same bounded navigation state machine in Go, C11, Rust, D, Ada, Zig, Nim and Swift. It deliberately separates navigation policy from operating-system accessibility APIs and speech output.

## Shared behavior

- First selects item index 0 when the item count is nonzero.
- Next advances one index, or reports no result at the end.
- Previous moves back one index, or reports no result at the beginning.
- Empty lists have no current item.
- Navigation never wraps implicitly.
- Indexes are zero-based and bounded by the caller-supplied item count.

The core only navigates a count supplied by an adapter. It does not discover accessible UI elements, query focus, invoke controls, read protected content, render braille, or speak. It does not yet provide a complete screen reader.

## Implementations

| Language | Location | Validation configured |
|---|---|---|
| Go | internal/screenreader/ | go test ./... |
| C11 | components/core-c/ | CMake + CTest |
| Rust (no_std) | crates/accessibility-core/src/screenreader.rs | cargo test |
| D | components/core-d/src/screenreader_cursor.d | dub test |
| Ada | components/core-ada/ | GNAT assertions (to be wired into CI) |
| Zig | components/core-zig/src/screenreader_cursor.zig | zig test |
| Nim | components/core-nim/src/screenreader_cursor.nim | nim c -r |
| Swift | components/core-swift/Sources/AccessibilityCore/ScreenReaderCursor.swift | swift test |

## Next required components

1. Semantic node model: role, accessible name, states, value, bounds, parent/children, stable identity.
2. Event intake: focus, structure, text, selection, live-region and window events; deduplication/coalescing.
3. Platform adapters: Windows UI Automation/MSAA; Linux AT-SPI; macOS/iOS Accessibility; browser accessibility tree; UEFI HII/keyboard/GOP.
4. Navigation modes: focus, object, headings/landmarks, links, form controls, text caret/ranges and tables.
5. Output adapters: speech, braille and accessible visual feedback, with interruption and verbosity policies.
6. Privacy/security: never announce password values or other protected text; no arbitrary action execution; bounded strings and explicit action permissions.
7. Tests: deterministic unit tests, adapter contract tests, real applications, NVDA/JAWS/VoiceOver/Orca interoperability, keyboard-only flows and hardware testing.

## Evidence rules

- Unit tests validate only the state machine.
- CI is NOT VALIDATED until an actual workflow run and logs are inspected.
- QEMU tests are not physical hardware proof.
- UEFI pre-OS speech is hardware/firmware-dependent and remains NOT VALIDATED.
- No claim of WCAG conformance or screen-reader usability is made by this cursor implementation.
