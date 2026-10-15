# ZERO — langage de programmation multi-cible

**État : PROPOSITION / NON IMPLÉMENTÉ.** Ce dossier définit le projet ; il ne prétend pas qu'un compilateur ou un langage fonctionnel existe déjà.

## Objectif
Concevoir un langage source universel visant les applications, les systèmes, l'UEFI, les systèmes d'exploitation, l'accessibilité, les calculs d'IA et les appareils physiques/virtuels, en étudiant systématiquement les compromis des langages existants.

ZERO ne doit pas être déclaré « meilleur » sur la base d'intentions. Chaque avantage devra être traduit en exigence, implémentation et test reproductible.

## Principes à évaluer
1. Sécurité mémoire par défaut, avec opérations bas niveau explicitement marquées et vérifiables.
2. Comportement défini : débordements, conversions, concurrence, initialisation, erreurs et ABI spécifiés.
3. Contrôle explicite des allocations, du temps d'exécution et des dépendances ; profil sans allocation dynamique pour UEFI/noyau.
4. Modèle de concurrence avec règles vérifiables et absence de courses de données dans le sous-ensemble sûr.
5. Interopérabilité ABI/C documentée, sans rendre ces interfaces automatiquement sûres.
6. Compilation native multi-architecture ; aucune dépendance obligatoire à une VM, au réseau ou au cloud.
7. Profils distincts : hosted, freestanding, UEFI, kernel et numeric/AI. Chaque profil indique ses API et restrictions.
8. Gestion des erreurs explicite, diagnostics accessibles et reproductibles.
9. Génération déterministe possible, builds reproductibles, manifeste de dépendances et chaîne d'outils auditable.
10. Accessibilité : diagnostics lisibles par lecteurs d'écran, commandes clavier, sorties structurées et outils sans interface graphique.
11. Cybersécurité dès la première ligne de code, grâce à HAKER : langage et outils de tests adversariaux qui attaquent les composants en développement, capturent les défauts et imposent les corrections/régressions avant acceptation.

## Développement attaque-first : ZERO + HAKER
- ZERO est le langage universel destiné à produire les composants.
- HAKER est le langage et l'atelier de test offensif contrôlé : fuzzing, tests de propriétés, cas adversariaux, tests différentiels, analyse de sécurité et preuves.
- Pour chaque changement : coder → tester → attaquer dans un périmètre isolé → enregistrer le résultat → corriger tout échec → rejouer les régressions.
- Un test absent, ignoré ou bloqué ne peut jamais compter comme PASS.
- Le catalogue initial et les règles de HAKER sont décrits dans [HAKER.md](HAKER.md) et [HAKER-CASES.json](HAKER-CASES.json). Leur présence ne signifie pas que HAKER est déjà implémenté.

## Démarche ZERO
- Établir l'inventaire et la matrice d'audit des langages et familles.
- Vérifier chaque constat dans les spécifications et documentations primaires ; distinguer défaut avéré, compromis volontaire, limite d'écosystème et hypothèse.
- Transformer les constats confirmés en exigences testables pour ZERO.
- Écrire la grammaire, la sémantique, le modèle mémoire, le système de types et les profils de compilation.
- Implémenter d'abord un prototype minimal et une suite de tests ; ajouter les backends ensuite.
- Publier les résultats avec statuts PASS, FAIL, PARTIAL, BLOCKED, UNPROVEN ou NOT_RUN.

## Fichiers
- [LANGUAGE-AUDIT.md](LANGUAGE-AUDIT.md) : périmètre, grille d'audit et familles de langages.
- [REQUIREMENTS.md](REQUIREMENTS.md) : exigences initiales et critères d'acceptation.
- [HAKER.md](HAKER.md) : conception du langage/outillage HAKER et boucle attaque-correction.
- [HAKER-CASES.json](HAKER-CASES.json) : catalogue initial de cas adversariaux, encore non exécutés.

## Limites connues
Un langage ne peut pas supprimer les contraintes du matériel, les défauts des compilateurs, les vulnérabilités des bibliothèques, les limites des systèmes d'exploitation ni les coûts des compromis. ZERO doit rendre ces limites explicites, les réduire là où c'est démontrable et ne jamais promettre une supériorité universelle sans benchmarks indépendants. Un test en simulation ne prouve pas le fonctionnement physique.
