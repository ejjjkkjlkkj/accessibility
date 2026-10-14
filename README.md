# Accessibilité — Screen reader, UEFI et OS

Dépôt de référence pour cartographier et développer les composants d'accessibilité par langage, depuis les couches bas niveau jusqu'aux applications système.

## Axes de travail

- **Screen reader** : navigation clavier, focus, rôles/états accessibles, événements UIA ou équivalents, braille et synthèse vocale.
- **UEFI / bare metal** : démarrage et interfaces firmware, clavier/console, GOP, audio et limites matérielles documentées.
- **OS** : intégration par système, API d'accessibilité, gestion audio, pilotes et installation.
- **IA locale** : intégration facultative d'un modèle local; aucune clé cloud obligatoire.
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
