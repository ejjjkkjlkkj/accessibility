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
| Ada | components/core-ada/screenreader_cursor.ads and .adb | GNAT assertions |
| Zig | components/core-zig/src/screenreader_cursor.zig | zig test |
| Nim | components/core-nim/src/screenreader_cursor.nim | nim c -r |
| Swift | components/core-swift/Sources/AccessibilityCore/ScreenReaderCursor.swift | swift test |

## IA universelle partagée — intégration optionnelle

Le modèle cible est un modèle commun aux langues naturelles et aux langages de programmation, maintenu dans `ejjjkkjlkkj/ia` (architecture : `UNIVERSAL-MODEL-ARCHITECTURE.md`). Ce dépôt d'accessibilité fournit les interfaces et tests d'intégration; il ne doit pas créer un modèle différent par langage ni dépendre d'un service cloud.

- L'IA ne remplace pas le noyau déterministe de navigation, l'arbre sémantique ou les API d'accessibilité natives.
- Les sorties du modèle sont non fiables : aucune action UI, commande système ou action firmware ne s'exécute sans validation déterministe et autorisation explicite.
- UEFI/bare metal : chargement seulement si la mémoire, le runtime et les pilotes sont disponibles; le chemin de démarrage et les annonces minimales doivent fonctionner sans IA.
- Les tests doivent séparer la couverture UTF-8, l'interopérabilité du checkpoint, la qualité par langue/tâche et la validation matérielle.
- La baseline bigramme UTF-8 actuelle est expérimentale; elle ne prouve ni compréhension sémantique ni aptitude à guider un utilisateur aveugle.

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
