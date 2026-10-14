# Exigences initiales de ZERO

**Statut : PROPOSÉES — aucune n'est encore validée par une implémentation.**

Chaque exigence doit devenir une spécification précise, un test automatisé et un résultat archivé.

## Sûreté et sémantique
- Z-SEM-001 : spécifier le résultat de chaque opération arithmétique, y compris dépassements et conversions.
- Z-MEM-001 : dans le sous-ensemble sûr, interdire les lectures de valeurs non initialisées, les références pendantes et les accès mémoire hors limites.
- Z-MEM-002 : isoler les opérations pointeurs/ABI/registres/assembleur dans un sous-ensemble explicitement marqué.
- Z-CON-001 : spécifier les règles de concurrence et détecter ou interdire les courses de données dans le sous-ensemble sûr.
- Z-ERR-001 : documenter les erreurs, leur propagation et les conditions de terminaison/panique.

## Contrôle et portabilité
- Z-RT-001 : proposer un profil sans runtime obligatoire et sans allocation implicite.
- Z-UEFI-001 : spécifier un profil UEFI et vérifier les contraintes ABI, format PE/COFF et appels de protocole avec une cible déclarée.
- Z-CPU-001 : définir les cibles initiales et distinguer fonctionnalité du compilateur et disponibilité réelle d'un backend.
- Z-ABI-001 : spécifier une interface C documentée ; ne pas considérer une frontière FFI comme automatiquement sûre.
- Z-REP-001 : fournir versions des outils, options, dépendances, hash des artefacts et commandes pour reproduire une compilation.

## Accessibilité et IA
- Z-A11Y-001 : diagnostics accessibles en texte structuré, avec codes stables et contexte d'erreur.
- Z-AI-001 : définir les types numériques, dispositions mémoire et opérations de tenseurs sans prétendre fournir un accélérateur avant implémentation.
- Z-TEST-001 : tests unitaires, tests de conformité, fuzzing du parseur et tests différentiels pour les backends.

## Critères d'acceptation
Une exigence ne passe à PASS que si :
1. la spécification est écrite ;
2. le code correspondant existe ;
3. les tests sont automatisés et reproductibles ;
4. les journaux et versions d'outils sont conservés ;
5. les limitations et cas non testés sont indiqués.

Pour UEFI : une compilation ou une exécution QEMU ne vaut pas preuve de fonctionnement sur matériel physique. Toute validation matérielle doit être rapportée séparément et ne doit pas impliquer de modification automatique de firmware/NVRAM.
