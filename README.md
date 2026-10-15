# Accessibilité — Screen reader, UEFI et OS

Dépôt de référence pour cartographier et développer les composants d'accessibilité par langage, depuis les couches bas niveau jusqu'aux applications système.

## Axes de travail

- **Screen reader** : navigation clavier, focus, rôles/états accessibles, événements UIA ou équivalents, braille et synthèse vocale.
- **UEFI / bare metal** : démarrage et interfaces firmware, clavier/console, GOP, audio et limites matérielles documentées.
- **OS** : intégration par système, API d'accessibilité, gestion audio, pilotes et installation.
- **IA locale universelle** : intégration facultative du modèle partagé défini dans `ejjjkkjlkkj/ia`; mêmes poids/contrat pour les langues naturelles et le code, avec adaptateurs par runtime. Le screen reader, la navigation de base et le démarrage UEFI ne doivent pas dépendre du modèle. Aucune clé cloud obligatoire.
- **Multi-langage** : chaque piste indique ce qui est implémenté, compilé, testé et validé sur matériel réel.

## Règles de preuve

- PASS exige une commande reproductible, des logs et des résultats vérifiables.
- PARTIAL, BLOCKED et NOT_VALIDATED restent distincts; une documentation seule ne prouve pas une fonctionnalité.
- QEMU et les autres simulateurs ne constituent pas une preuve matérielle physique.
- Aucune action firmware/NVRAM, aucun flash BIOS et aucun redémarrage automatique ne sont déclenchés par la CI.
- Les workflows sont limités aux validations statiques et aux tests reproductibles en environnement hébergé; les preuves matérielles restent séparées.

## Catalogue et feuille de route

Les documents canoniques maintenus dans docs/ sont la cartographie des langages et les portes de release Screen Reader / UEFI / OS. Les IDs de langage doivent rester synchronisés avec ejjjkkjlkkj/ia; les statuts d'accessibilité et d'IA sont indépendants.

## CI

GitHub Actions vérifie les fichiers de documentation, la syntaxe JSON des manifestes présents et les liens Markdown locaux. Cette CI ne prétend pas prouver le fonctionnement d'un screen reader sur matériel réel.


## Shared role identifiers\n\nThe numeric role mapping for C, Rust, Go, D, Ada, Nim, Swift and Zig is defined in [Accessibility role codes v1](docs/ACCESSIBILITY-ROLE-CODES-V1.md). This is a logical role-code contract, not a claim that all binary layouts or platform adapters are already interchangeable.\n\n## NEXORA integration

The accessibility boundary for the experimental NEXORA language is documented in [the NEXORA accessibility contract](docs/NEXORA-ACCESSIBILITY-CONTRACT.md) and its machine-readable contract at `projects/nexora/accessibility-contract.json`. This is a contract only: it does not claim a working NEXORA screen reader, OS backend, or UEFI runtime.

## Architecture rules

- [Independent implementation tracks for every language and the separate NEXORA universal language](docs/ARCHITECTURE-INDEPENDENT-LANGUAGE-TRACKS-AND-NEXORA.md)


## Dark Team — threat research and security engineering

The Dark Team workstreams cover authorized Red Team testing, Blue Team detection and response, physical hardware/silicon, UEFI/firmware, memory safety, screen-reader attack surfaces, Surface/Deep/Dark Web research, overlay networks, supply-chain provenance, forensics, cross-platform validation and accessible security tools. Scope, safety constraints, evidence labels and implementation gates are defined in [the Dark Team architecture](docs/DARK-TEAM-ARCHITECTURE-AND-IMPLEMENTATION.md); its current machine-readable workstream inventory is [projects/dark-team/manifest.json](projects/dark-team/manifest.json). These are plans and inventory, not claims that all modules are already implemented.
