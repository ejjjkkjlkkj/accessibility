# Portes d'exécution et de release — tous les langages

## Ordre de travail

1. Synchroniser les IDs de ce dépôt avec LANGUAGE-CATALOG.md dans ejjjkkjlkkj/ia.
2. Pour chaque langage, enregistrer compiler/runtime/version, cibles, dépendances, licence et limites.
3. Construire les composants screen reader hébergés : focus, navigation clavier, arbre sémantique, annonces et tests.
4. Séparer les adaptateurs UEFI des adaptateurs OS. Ne pas annoncer une cible bare metal à partir d'une compilation hébergée.
5. Produire un test minimal reproductible par piste, puis ajouter les tests de non-régression.
6. Générer les artefacts de release uniquement lorsque les gates correspondantes passent.
7. Mesurer les résultats sur matériel réel séparément de QEMU/simulateurs.

## Gates indépendantes

- G0 — Catalogue : ID présent dans les deux dépôts; aucune piste oubliée.
- G1 — Toolchain : compilateur/runtime identifiés et versionnés.
- G2 — Screen reader : navigation clavier, focus, sémantique, annonces et erreurs testés.
- G3 — UEFI : artefact EFI valide, boot de simulation identifié et preuve physique séparée.
- G4 — OS : intégration, tests sur la cible et packaging vérifiés.
- G5 — Release : logs, versions, hashes, tests, limites connues et procédure de récupération publiés.

## Conditions de PASS

Chaque gate PASS doit inclure les commandes exactes, les logs non tronqués, les versions d'outils, le hash des artefacts et le périmètre matériel/logiciel. Les échecs restent visibles. Un composant simulé, un stub, un buffer PCM ou un registre écrit ne suffit pas à prouver une fonction physique.

Aucune CI hébergée ne doit déclencher de flash firmware, d'écriture NVRAM, de reboot physique ou d'action destructive.
