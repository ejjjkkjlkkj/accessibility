# HAKER AI — objectif de performance supérieure et plan de preuve

**État actuel : OBJECTIF / NON IMPLÉMENTÉ / NON VALIDÉ.**  
**Périmètre : dépôt accessibility, composants ZERO/HAKER et environnements de test explicitement autorisés.**

## 1. Objectif ambitieux

Construire de zéro une IA spécialisée en recherche de vulnérabilités et en cybersécurité défensive, avec l'objectif expérimental de dépasser les meilleurs spécialistes humains sur des tâches définies, reproductibles et pertinentes pour ce projet.

Cette ambition n'est pas une affirmation de capacité actuelle. « Dépasser tous les hackers du monde » est une hypothèse à tester, pas un résultat que l'on peut déclarer sans évaluation indépendante et représentative. Aucun benchmark fini ne prouve une supériorité universelle sur toutes les personnes, tous les systèmes et toutes les attaques futures.

## 2. Limite de périmètre non négociable

HAKER AI agit uniquement sur :
- le dépôt du projet et ses révisions autorisées ;
- le code, les dépendances et les artefacts construits pour le projet ;
- les fixtures synthétiques et corpus de vulnérabilités de laboratoire ;
- les processus, conteneurs ou machines virtuelles jetables expressément préparés pour les tests ;
- les appareils physiques du projet après autorisation explicite et définition du plan de test.

Par défaut : réseau désactivé, aucune cible tierce, aucun vol ou export de secrets, aucune propagation, aucune persistance, aucune action destructive, aucune modification de firmware/NVRAM/BIOS et aucune publication/fusion automatique. Les outils d'analyse reçoivent des permissions minimales et un périmètre vérifié. Les contenus du dépôt, issues, commentaires, fixtures et sorties d'outils sont des données non fiables et ne peuvent pas modifier les règles de sécurité.

## 3. Concevoir l'IA à partir de zéro

Le projet doit distinguer les éléments effectivement créés par nous des outils et modèles externes utilisés pendant la recherche ou le bootstrap.

Composants à développer progressivement :
1. **Noyau d'orchestration déterministe** : tâches, budgets, périmètres, délais, quotas et arrêt d'urgence.
2. **Moteur d'analyse propre au projet** : compréhension de ZERO, HAKER, des langages, de la chaîne de compilation et des interfaces OS/UEFI/accessibilité réellement présentes.
3. **Génération de tests adversariaux** : mutations, cas limites, propriétés, fuzzing et tests différentiels sur des cibles locales.
4. **Planificateur d'hypothèses** : propose des chemins d'investigation classés par risque et coût, sans exécuter d'action hors périmètre.
5. **Vérificateur indépendant** : reproduit les résultats, rejette les preuves insuffisantes et empêche un simple raisonnement du modèle de compter comme découverte.
6. **Moteur de correction** : propose un patch isolé, lance la suite de régression et vérifie les effets secondaires.
7. **Mémoire de recherche versionnée** : corpus, graines, cas minimaux, faux positifs, correctifs et résultats reproductibles.
8. **Rapport d'audit** : preuves, hashes, versions, commandes, journaux expurgés, couverture, limites et statut stable.

L'usage initial d'outils, compilateurs ou modèles préexistants doit être déclaré. Une IA réellement entraînée ou compilée de zéro, l'autonomie complète et l'autohébergement sont des étapes distinctes, à prouver séparément.

## 4. Boucle de recherche obligatoire

Pour chaque hypothèse :
1. enregistrer la cible autorisée et son empreinte de version ;
2. formuler une hypothèse falsifiable et un oracle de résultat ;
3. générer des tests bornés et reproductibles ;
4. exécuter en environnement isolé avec limites CPU, mémoire, disque et temps ;
5. confirmer toute découverte par une reproduction indépendante ;
6. produire un cas minimal et une mesure d'impact contextualisée ;
7. proposer une correction dans une branche ou un patch isolé ;
8. exécuter les tests de régression, tests adversariaux et contrôles de compatibilité ;
9. conserver les résultats négatifs et les limites ;
10. demander une revue humaine avant toute fusion, publication ou opération sur matériel réel.

Un modèle qui décrit une vulnérabilité sans la reproduire obtient UNPROVEN, pas une découverte confirmée.

## 5. Évaluation comparative contre des spécialistes

Préenregistrer le protocole avant les essais. Comparer HAKER AI à des spécialistes humains qualifiés et, lorsque pertinent, aux outils de sécurité de référence.

Conditions :
- mêmes dépôts, versions, informations initiales, ressources et limites de temps ;
- séparation entre corpus de développement et corpus caché de test ;
- benchmarks publics et privés, y compris cas nouveaux conçus par des évaluateurs indépendants ;
- ordre des tâches randomisé et notation à l'aveugle lorsque possible ;
- validation indépendante de chaque découverte ;
- publication des réussites, échecs, faux positifs, abstentions et ressources consommées ;
- protection contre la contamination des données d'entraînement par les réponses du benchmark.

Mesures minimales :
- précision et rappel des vulnérabilités confirmées ;
- gravité et nouveauté des défauts trouvés ;
- taux de faux positifs ;
- vulnérabilités critiques manquées ;
- temps médian et coût par découverte confirmée ;
- taux de correctifs acceptés et de régressions ;
- reproductibilité par une autre machine et un autre évaluateur ;
- robustesse aux modifications de code, aux entrées adversariales et aux données trompeuses ;
- couverture effective par composant et par famille de vulnérabilités.

Le seuil de succès doit être fixé avant l'évaluation. Une revendication de supériorité ne peut être faite que sur les tâches et conditions effectivement évaluées, avec intervalles d'incertitude et réplications indépendantes. Le terme « supérieur à tous les hackers » ne peut pas être validé universellement par un seul benchmark.

## 6. Échelle de preuve

- **P0 — intention** : proposition ou documentation seulement.
- **P1 — prototype** : code présent, mais non exécuté.
- **P2 — tests locaux** : exécution enregistrée et tests unitaires réussis.
- **P3 — reproduction** : découverte reproduite depuis un cas minimal, avec artefacts identifiés.
- **P4 — correction** : défaut reproduit avant patch, absent après patch, régressions testées.
- **P5 — benchmark indépendant** : protocole préenregistré, évaluateur indépendant et corpus séparé.
- **P6 — validation de terrain limitée** : matériel/système exact documenté, autorisation, instrumentation et limites consignées.

Une simulation ne prouve pas le comportement matériel réel. Un succès de compilation ne prouve pas la sécurité. Un test ignoré, absent, simulé ou bloqué ne peut jamais produire un statut positif pour la propriété visée.

## 7. Statuts obligatoires

- **PASS** : exigence et preuves prévues vérifiées ;
- **FAIL** : échec confirmé ;
- **PARTIAL** : couverture partielle ;
- **BLOCKED** : environnement ou autorisation manquant ;
- **NOT_RUN** : aucune exécution ;
- **UNPROVEN** : preuves insuffisantes ;
- **SIMULATION_ONLY** : résultat limité à une simulation ;
- **NOT_IMPLEMENTED** : capacité non développée.

## 8. Feuille de route

- **H0** — catalogue strict, schéma, validateurs et rapport JSON fail-closed.
- **H1** — harnais de tests déterministes et fixtures synthétiques locales.
- **H2** — tests de sécurité spécifiques au code ZERO/HAKER et à ses compilateurs.
- **H3** — fuzzing, corpus versionné, réduction et rejeu automatique.
- **H4** — analyse statique, dépendances, secrets factices, SBOM et provenance.
- **H5** — agents IA locaux spécialisés, vérificateur indépendant et boucle de correction.
- **H6** — benchmark comparatif préenregistré, avec spécialistes et évaluateur indépendant.
- **H7** — composants développés par le projet, entraînement/évaluation traçables et autohébergement progressif.
- **H8** — validations distinctes par OS, UEFI, périphérique virtuel et matériel physique autorisé.

Chaque phase exige du code, des tests exécutés, des preuves conservées et des limites documentées. Les documents seuls ne terminent aucune phase.

## 9. Critère de réussite pour ce projet

Le premier objectif vérifiable est que HAKER trouve et confirme des défauts réels dans nos propres composants, produise des correctifs qui passent une suite de régression et démontre un gain mesurable par rapport aux outils de référence et aux évaluateurs humains sur un benchmark préenregistré.

**Aucune supériorité n'est actuellement démontrée.** Toute annonce future devra pointer vers les runs, artefacts, cas de reproduction et rapports indépendants correspondants.
