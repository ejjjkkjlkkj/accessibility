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

## OS complet et chaîne accessible

Le plan [OS complet, chaîne logicielle accessible et validation HAKER](docs/OS-COMPLETE-ACCESSIBLE-CHAIN.md) couvre du matériel/UEFI jusqu'au noyau, pilotes, services, accessibilité, interface, applications, SDK, IA locale, installation et mises à jour. Il définit les portes de validation HAKER et sépare explicitement simulation, preuves logicielles, tests humains et validation matérielle. C'est une architecture cible, pas une affirmation que l'OS ou HAKER est déjà complet.

## Audit des éléments manquants

Le document [Audit des manques de l'OS complet accessible et de HAKER](docs/OS-COMPLETE-GAP-AUDIT.md) recense les écarts de couverture, les priorités P0/P1/P2, les critères d'acceptation et les références primaires (NIST, UEFI/ACPI, noyau Linux, W3C). C'est un audit documentaire : il ne certifie pas l'implémentation ni l'exhaustivité du projet.

## Réduction des ports réseau exposés

La politique [Réduction de la surface réseau](docs/NETWORK-PORT-MINIMIZATION.md) exige un refus par défaut, des exceptions minimales déclarées, un inventaire des sockets TCP/UDP IPv4/IPv6 et une comparaison entre ports attendus et réellement en écoute. Le document est une spécification : l'application automatique des règles et les validations natives restent **NOT_IMPLEMENTED / NOT_VALIDATED** jusqu'à preuve reproductible.

## Réseau discret et modèle de menace

[Conception réseau discrète et modèle de menace](docs/NETWORK-STEALTH-THREAT-MODEL.md) précise ce que « réseau invisible » peut réellement signifier, les surfaces d'attaque à examiner, les défenses en couches, les tests HAKER autorisés et les portes de validation. L'invisibilité absolue n'est pas garantie ; l'objectif est de réduire l'exposition, les communications non déclarées et les surprises, avec preuves par environnement.


## Objectif zéro faille connue et réduction du temps perdu

Les [portes de validation sécurité et fiabilité](docs/ZERO-KNOWN-VULNERABILITIES-AND-RELEASE-GATES.md) définissent les contrôles attendus pour le réseau, les logiciels/dépendances, les pilotes, le noyau, la récupération et l'accessibilité, ainsi que les critères qui bloquent une livraison. Il s'agit d'un objectif de **zéro vulnérabilité critique/haute connue non résolue**, pas d'une promesse impossible de zéro faille absolue. À ce stade, le document est une spécification : les mécanismes doivent encore être implémentés et validés.

## Deuxième passe — audit de complétude

L’[audit de deuxième passe de complétude, sécurité, chaîne d’approvisionnement, matériel et accessibilité](docs/OS-COMPLETENESS-SECOND-PASS-AUDIT.md) recense les lacunes encore à traiter et leur priorité. Verdict documentaire : **PARTIAL** ; HAKER et ZERO ne sont pas déclarés fonctionnels, et la validation matérielle reste séparée.
