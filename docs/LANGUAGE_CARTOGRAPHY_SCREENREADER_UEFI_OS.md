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
| c | C | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| cpp | C++ | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| rust | Rust | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| zig | Zig | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| ada | Ada | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| asm-x86_64 | x86-64 Assembly | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| asm-aarch64 | AArch64 Assembly | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| asm-riscv64 | RISC-V Assembly | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| go | Go | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| d | D | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| nim | Nim | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| swift | Swift | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| pascal | Free Pascal / Object Pascal | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| odin | Odin | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| fortran | Fortran | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| csharp | C# | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| objc | Objective-C | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| v | V | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| crystal | Crystal | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| modula2 | Modula-2 | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| cobol | COBOL | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| pli | PL/I | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| java | Java | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| kotlin | Kotlin | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| python | Python | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| javascript | JavaScript | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| typescript | TypeScript | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| scratch | Scratch | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| u | U (ulanguage.org) | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| julia | Julia | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| mojo | Mojo | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| chapel | Chapel | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| r | R | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| apl | APL | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| j | J | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| k | K | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| q | q | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |
| lua | Lua | PARTIAL — composants et adaptateurs selon runtime | PARTIAL — seulement si cible/compiler et interfaces firmware disponibles | PARTIAL — API natives et dépendances déclarées | NOT_VALIDATED |

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
