# Audit transversal du dépôt accessibility — 2026-10-15

## Périmètre et méthode

- Dépôt : `ejjjkkjlkkj/accessibility`.
- Référence initiale auditée : branche `feature/zero-haker-attack-first`, commit `bae51667f0d540217fd10a22c9660ff19ca39748`.
- Arbre Git récursif complet retourné sans troncature : 121 entrées, 84 fichiers, 37 répertoires, 345 814 octets déclarés pour les blobs.
- Périmètre examiné : workflows GitHub Actions, code et tests C/Rust/Go/D/Ada/Nim/Swift/Zig, catalogue de ressources, manifeste des langages, contrats JSON, documentation d'architecture, exigences HAKER/ZERO, recherche UEFI/accessibilité, audits de risques et statuts de preuve.
- Type de contrôle : revue statique et vérification des runs GitHub Actions disponibles. Aucun test matériel physique n'a été exécuté dans le cadre de cet audit.

## Verdict

**PARTIAL / NOT_CERTIFIED.** Le dépôt contient un contrat de rôles sémantiques commun, un noyau Rust `no_std` limité, des prototypes multi-langages, des validateurs de catalogue et des workflows de tests. Il ne contient pas encore un OS accessible complet, un lecteur d'écran complet, un runtime HAKER autonome, une IA HAKER entraînée de zéro, un compilateur ZERO complet ni une chaîne de build démontrée exclusivement native/code machine.

## Résultats prioritaires

### A0 — CI avec écriture automatique sur main

**Constat initial :** `.github/workflows/resource-database.yml` utilisait `contents: write`, se déclenchait selon un planning ou un lancement manuel, puis poussait automatiquement des changements vers `main`. Cela contredisait la règle de ne pas modifier automatiquement la branche principale.

**Correction appliquée sur cette branche :** permissions réduites à `contents: read`; étape de commit/push automatique supprimée. Le workflow peut produire et téléverser un artefact, mais ne pousse plus la base vers `main`.

**Preuve de changement :** commit `89e065a5dd4da703777cb81cc8631d9cdb40501b`.
**Statut :** correction enregistrée sur la branche de travail; exécution CI de ce changement à confirmer séparément.

### A1 — HAKER n'est pas encore une IA exécutable

**Constat :** `ZERO/HAKER-CASES.json` contient huit cas de test marqués `NOT_IMPLEMENTED`. `tools/validate_haker_catalog.py` valide la structure et la politique du catalogue; il n'exécute pas ces cas. Le job Kali valide ce catalogue et lance les tests unitaires existants, les tests Go et le validateur CSV. Il n'exécute pas un moteur d'attaque HAKER propre au projet.

**Conséquence :** un job vert prouve les contrôles effectivement exécutés dans ce job, pas que HAKER est une IA fonctionnelle ou qu'il a trouvé des vulnérabilités.

**Statut : NOT_IMPLEMENTED** pour le moteur HAKER complet, son raisonneur, son moteur d'apprentissage de zéro et son rapport de preuves d'exécution.

### A2 — « Machine code » : pas encore démontré pour HAKER

Les sources C, Rust, Go, D, Ada, Nim, Swift et Zig sont compilables selon les workflows configurés, mais cela ne constitue pas une IA créée de zéro. Aucun code assembleur HAKER, compilateur HAKER/ZERO, pipeline d'entraînement propre, poids produits par le projet ou binaire HAKER natif avec inventaire de dépendances et provenance n'est présent dans l'arbre audité.

**Statut : NOT_IMPLEMENTED / NOT_VALIDATED.** L'objectif de code natif doit être démontré par artefacts, désassemblage, chaîne de build, versions, dépendances et hashes. Une compilation native ne prouve pas à elle seule que l'intelligence ou les données ont été créées de zéro.

### A3 — Le noyau d'accessibilité est un prototype de logique, pas un lecteur d'écran OS

Le crate Rust `crates/accessibility-core` fournit des structures d'événements, une navigation et des primitives d'arbre sémantique sans allocation. Les autres langages contiennent surtout des enums, validateurs et curseurs de navigation. Les sources indiquent explicitement qu'elles ne réalisent pas les requêtes d'accessibilité OS ni la sortie vocale.

Manquent notamment : adaptateurs natifs Windows UI Automation, Linux AT-SPI et plateformes Apple, événements temps réel, gestion complète des actions et du focus, sortie vocale/braille, intégration périphériques et tests utilisateurs de bout en bout.

**Statut : PARTIAL** pour le noyau sémantique; **NOT_IMPLEMENTED / NOT_VALIDATED** pour l'expérience complète de lecteur d'écran.

### A4 — Les contrats logiques ne sont pas encore une ABI binaire commune

Les codes de rôles 0 à 17 sont synchronisés par tests. Le contrat de rôle précise qu'il ne définit pas à lui seul une ABI complète. Les modèles d'événements diffèrent entre les langages, notamment sur les champs `state_flags`, `source_id`, l'encodage, la propriété des chaînes, la disposition mémoire et les séquences d'événements.

**Action nécessaire :** définir un format de sérialisation ou ABI versionné indépendant du langage, avec tailles, alignements, endianness, chaînes UTF-8 bornées, règles de propriété, identifiants, séquences et vecteurs de test partagés.

### A5 — Durcissement de la validation C

`components/core-c/src/accessibility_event.c` utilise `strlen(name)` avant la copie bornée. Cela suppose que `name` est une chaîne C valide et terminée par NUL; une entrée non terminée peut entraîner une lecture au-delà de la zone prévue. Le contrat d'API doit expliciter la précondition ou évoluer vers une entrée pointeur-longueur et une vérification bornée.

**Statut : PARTIAL — revue statique; aucun crash n'a été reproduit dans cet audit.**

### A6 — Contrôles de sécurité du catalogue de ressources

`tools/build_resource_db.py` suit les redirections HTTP(S) et vérifie des URLs issues du CSV. La portée actuelle est un catalogue de ressources, mais les URL restent des données modifiables. Pour une CI exposée à des contributions non fiables, il faut interdire les destinations loopback, privées, link-local et metadata, revalider chaque redirection, borner le nombre de redirections et éviter les destinations internes. Les tests existants couvrent certaines erreurs HTTP, pas cette politique de destination.

**Statut : PARTIAL — amélioration défensive requise avant de considérer le vérificateur d'URL sûr pour des données arbitraires.**

### A7 — Dépendances des workflows

Plusieurs workflows référencent des actions par tags de version (par exemple `actions/checkout@v6`, `actions/setup-python@v6`), et non par SHA de commit immuable. Pour renforcer la chaîne d'approvisionnement, épingler les actions externes par SHA complet, documenter l'éditeur et gérer les mises à jour par PR avec vérification de provenance. Les permissions doivent rester minimales.

**Statut : PARTIAL — les workflows de test fonctionnent, mais l'épinglage immuable n'est pas généralisé.**

### A8 — Le workflow multi-OS documentaire n'est pas une validation de code multi-OS

`.github/workflows/multi-os-validation.yml` vérifie JSON, la synchronisation du catalogue de langages et l'existence de documents sur plusieurs runners/versions Python. Ce job ne compile pas les composants de lecteur d'écran sur toutes ces cibles. Les compilations/tests de code sont réalisés par d'autres workflows, avec des matrices et couvertures distinctes.

**Statut : PASS pour les contrôles documentaires observés; NOT_VALIDATED pour toute conclusion de couverture complète du code multi-OS.**

### A9 — README mal formé

Le README contenait les caractères littéraux `\\n` autour de la section « Shared role identifiers », ce qui collait les titres et les paragraphes dans le même texte Markdown.

**Correction appliquée :** retours à la ligne Markdown réels.
**Preuve de changement :** commit `b18e8392faff2fc1feaaace7a70fb9f709079a81`.
**Statut :** correction enregistrée sur cette branche; contrôle CI du changement à confirmer séparément.

## Runs observés pour la référence initiale

Sur le commit `bae51667f0d540217fd10a22c9660ff19ca39748`, les runs suivants ont été retournés comme terminés avec succès :

- HAKER proof harness : https://github.com/ejjjkkjlkkj/accessibility/actions/runs/38719978430
- Polyglot functional CI : https://github.com/ejjjkkjlkkj/accessibility/actions/runs/38719978766
- Multi-OS accessibility validation : https://github.com/ejjjkkjlkkj/accessibility/actions/runs/38719978069
- Accessibility and ZERO CI : https://github.com/ejjjkkjlkkj/accessibility/actions/runs/38719978504

Le job `HAKER harness / Kali Linux container` a terminé avec succès. Son succès est limité aux étapes exécutées dans le conteneur Kali : validation du catalogue, tests unitaires du dépôt, tests Go et validation du catalogue CSV. Il ne prouve ni l'exécution de tous les outils Kali, ni l'existence du moteur HAKER, ni une validation matérielle.

## Ordre de travail recommandé

1. **A0 — Vérifier le changement de workflow et les protections CI** : confirmer que le workflow ressource n'a plus de chemin d'écriture vers `main`.
2. **A1 — Construire un moteur HAKER déterministe natif minimal** : exécuteur de cas synthétiques, périmètre fail-closed, limites de ressources, code retour, preuves brutes hachées et rapport JSON versionné.
3. **A2 — Ajouter des tests négatifs au vérificateur** : preuve absente, test omis, statut falsifié, entrée malformée, limite de temps et cible hors périmètre doivent échouer ou rester BLOCKED.
4. **A3 — Formaliser l'ABI** : spécification, sérialisation, vecteurs communs et tests de compatibilité multi-langages.
5. **A4 — Corriger les limites de sécurité** : chaînes C pointeur-longueur, contrôle des redirections/URL, limites de taille et épinglage SHA des actions.
6. **A5 — Intégrer les outils Kali un par un** : versions et licences, adaptateurs typés, sandbox, fixtures et validation indépendante.
7. **A6 — Développer l'intelligence propre au projet** : raisonneur explicable, corpus documenté, puis moteur d'apprentissage implémenté et entraîné à partir de zéro, sans poids préentraînés si cette exigence reste absolue.
8. **A7 — Démontrer la chaîne native** : compiler, hacher et inspecter les binaires; enregistrer architecture cible, compilateur, options, dépendances, SBOM/provenance et résultats reproductibles.
9. **A8 — Valider l'accessibilité de bout en bout** : adaptateurs OS, clavier/focus, rôles/états, parole/braille et tests avec lecteurs d'écran.
10. **A9 — Séparer simulation et matériel** : QEMU/VM reste SIMULATION_ONLY; tout test physique exige un banc dédié, une autorisation et des preuves spécifiques.

## Règles de statut

- **PASS** : exigence effectivement testée avec toutes les preuves prévues.
- **FAIL** : échec reproduit.
- **PARTIAL** : couverture limitée.
- **BLOCKED** : prérequis ou autorisation manquant.
- **NOT_IMPLEMENTED** : code absent.
- **NOT_VALIDATED** : preuves manquantes.
- **SIMULATION_ONLY** : résultat de simulation uniquement.

Ce document est un audit statique daté, pas une certification, pas une preuve de zéro vulnérabilité et pas une déclaration que le dépôt est complet.
