# Cartographie multi-langage : Screen Reader, UEFI et OS

## Objectif

Définir une piste de travail pour chaque ID du catalogue commun IA. Cette cartographie est une feuille de route, pas une déclaration d'implémentation achevée. Tous les statuts commencent à NOT_VALIDATED tant qu'une preuve spécifique n'est pas attachée.

## Architecture de référence

1. **Noyau commun** : événements d'entrée, état du focus, arbre sémantique, commandes clavier, gestion des erreurs et journal de preuve.
2. **Screen reader hébergé** : adaptateur vers les API natives d'accessibilité du système (par exemple UI Automation sur Windows); parole, braille, navigation et tests clavier.
3. **Screen reader UEFI / bare metal** : console texte et événements firmware disponibles, GOP pour l'affichage, clavier selon protocoles exposés, synthèse vocale et audio uniquement lorsque le chemin matériel est démontré. Les API UEFI ne fournissent pas universellement un pilote audio ou une API d'accessibilité.
4. **Couche OS** : adaptateurs distincts pour Windows, Linux et autres cibles effectivement supportées; ne pas confondre un binaire UEFI avec un pilote ou une application OS.
5. **IA locale** : optionnelle, hors du chemin critique de démarrage; un modèle externe ou un service cloud ne constitue pas le screen reader lui-même.

## Matrice des pistes

| ID | Langage | Screen reader | UEFI / bare metal | OS | Statut initial |
|---|---|---|---|---|---|
| c | C | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| cpp | C++ | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| rust | Rust | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| zig | Zig | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| ada | Ada | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| asm-x86_64 | x86-64 Assembly | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| asm-aarch64 | AArch64 Assembly | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| asm-riscv64 | RISC-V Assembly | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| go | Go | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| d | D | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| nim | Nim | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| swift | Swift | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| pascal | Free Pascal / Object Pascal | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| odin | Odin | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| fortran | Fortran | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| csharp | C# | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| objc | Objective-C | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| v | V | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| crystal | Crystal | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| modula2 | Modula-2 | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| cobol | COBOL | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| pli | PL/I | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| java | Java | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| kotlin | Kotlin | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| python | Python | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| javascript | JavaScript | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| typescript | TypeScript | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| scratch | Scratch | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| u | U (ulanguage.org) | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| nexora | NEXORA (langage universel expérimental) | NOT_VALIDATED — aucun screen reader écrit en NEXORA validé | NOT_VALIDATED — aucun compilateur/backend UEFI ni runtime freestanding validé | NOT_VALIDATED — aucun backend OS ni adaptateur natif validé | PARTIAL — interpréteur de référence limité seulement |
| julia | Julia | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| mojo | Mojo | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| chapel | Chapel | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| r | R | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| apl | APL | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| j | J | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| k | K | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| q | q | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |
| lua | Lua | À ÉVALUER — composants et adaptateurs selon runtime | À ÉVALUER — seulement si cible/compiler et interfaces firmware disponibles | À ÉVALUER — API natives et dépendances déclarées | NOT_VALIDATED |

### Piste NEXORA — limites actuelles

NEXORA est distinct de U (ulanguage.org). Le dépôt IA contient un prototype d’interpréteur limité; cela ne prouve pas l’existence d’un compilateur, d’un IR typé stable, d’un binaire natif, d’un screen reader, d’un backend OS ou d’un exécutable UEFI. Les trois axes Screen Reader, UEFI/bare metal et OS restent NOT_VALIDATED jusqu’à des tests spécifiques reproductibles.

## Règles propres aux cibles

- **C, C++, Rust, Zig, Ada, D et assembleurs** : candidats bas niveau selon compilateur, ABI, pilotes et bibliothèques réellement disponibles; ne pas présumer qu'un langage suffit à lui seul.
- **Go, Nim, Swift, Pascal, Odin, Fortran, C#, Objective-C et langages JVM** : capacité OS selon runtime/cible; la prise en charge UEFI/freestanding doit être prouvée séparément.
- **Python, JavaScript, TypeScript, R, Julia, Mojo, Chapel, Lua, APL, J, K, q et autres langages hébergés/expérimentaux** : prototypes et composants OS selon runtime; aucune prétention bare metal sans toolchain freestanding et preuve d'exécution.
- **Assembleurs x86-64, AArch64 et RISC-V** : noyaux/entrées bas niveau; ils ne remplacent pas à eux seuls un lecteur d'écran complet.
- **Scratch** : piste pédagogique, pas une cible de release UEFI/OS.
- **U** : valider précisément le compilateur et la chaîne U → C avant d'affirmer une capacité native.

## Preuves de release

### Screen Reader
- Tests clavier sans souris; focus observable et cohérent.
- Rôles, noms, états, changements et annonces vérifiés avec une application de test.
- Tests NVDA/JAWS/Narrator/Orca quand la cible le permet; consigner les versions.
- Parole et braille testés réellement, avec logs et cas d'erreur.

### UEFI / bare metal
- Compilation EFI reproductible, architecture et hash consignés.
- Boot QEMU explicitement étiqueté SIMULATION.
- Validation sur matériel physique séparée, avec matériel, firmware, méthode, logs et résultat.
- Aucune preuve audio physique basée uniquement sur l'écriture de registres ou un buffer PCM.

### OS
- Compilation et tests sur chaque OS revendiqué.
- API d'accessibilité et périphériques testés sur la cible; pas de réussite extrapolée depuis un autre OS.
- Package de release et procédure de retour arrière documentés.

## Statuts obligatoires

- NOT_IMPLEMENTED : aucun composant vérifiable.
- PARTIAL : composants présents, critères incomplets.
- BLOCKED : blocage reproductible documenté.
- NOT_VALIDATED : preuve insuffisante ou tests non exécutés.
- PASS : critères de la cible satisfaits et artefacts/logs disponibles.

Un test unitaire ou une simulation ne prouve pas une validation matérielle. La cible de fiabilité du projet est 99,99 %, mais elle ne peut être revendiquée sans une méthode statistique et des données d'essai adaptées.


## NEXORA accessibility interface

The proposed language-neutral node/event/command contract is defined in `projects/nexora/accessibility-contract.json` and explained in `docs/NEXORA-ACCESSIBILITY-CONTRACT.md`. It is a design boundary only; native adapters and end-to-end behavior remain NOT_VALIDATED.
