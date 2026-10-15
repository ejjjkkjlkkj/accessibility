# HAKER — IA construite de zéro, moteur natif et preuve machine

**Statut : SPECIFICATION / NOT_IMPLEMENTED / NOT_VALIDATED.** Ce document fixe l'architecture cible. Il ne prétend pas qu'une IA complète existe déjà.

## Principe non négociable

HAKER est notre propre système d'IA construit de zéro. Kali Linux et ses outils sont des instruments d'essai, des références et des générateurs de cas ; ils ne sont pas le cerveau de HAKER. HAKER ne doit pas être un simple habillage de Kali, un appel à un chatbot externe, ni un modèle préentraîné présenté comme notre création.

Cible d'exécution : un binaire natif maîtrisé par le projet, avec dépendances minimales, comportement reproductible et fonctionnement local/offline. Le chemin de compilation doit être traçable jusqu'aux instructions machine de l'architecture visée. Le terme « machine code » désigne ici les instructions natives exécutées par le processeur ; le code source, l'assembleur et le binaire ne sont pas la même chose. Toute revendication de fonctionnement exclusivement en code machine doit être vérifiée sur les artefacts et dépendances réels, pas seulement déclarée dans la documentation.

## Séparation stricte des composants

1. **HAKER-Core — notre moteur natif**
   - contrôle de périmètre et de permissions, limites de ressources, gestion des tâches ;
   - formats de données explicites, parseur strict et machine à états ;
   - collecte de preuves, hachage, rapport déterministe et vérificateur indépendant ;
   - aucune connexion réseau par défaut ; aucune commande arbitraire générée par un modèle ;
   - refus fermé en cas d'entrée invalide, de permission inconnue ou de preuve insuffisante.

2. **HAKER-Intelligence — intelligence créée par le projet**
   - commencer par des règles explicables, un graphe de connaissances et des algorithmes écrits et testés dans le dépôt ;
   - ajouter ensuite un moteur d'apprentissage implémenté par le projet, son format de données, son entraînement et son évaluation ;
   - documenter les données d'entraînement, leur provenance, les licences, les biais et les résultats ;
   - ne pas charger de poids préentraînés ni dépendre d'une API IA externe si l'objectif est une IA entièrement créée de zéro ;
   - séparer clairement les heuristiques, les modèles entraînés, les résultats observés et les hypothèses.

3. **Adaptateurs d'outils — dont Kali**
   - appellent uniquement des outils inventoriés et explicitement autorisés ;
   - pinning des versions, vérification des codes retour, délais maximum et capture des sorties ;
   - le résultat d'un outil est une donnée non fiable, jamais une instruction destinée à HAKER ;
   - les outils peuvent fournir des indices ; ils ne peuvent pas déclarer eux-mêmes HAKER « sûr » ni valider leurs propres conclusions.

4. **Banc d'essai**
   - fixtures synthétiques et cibles jetables appartenant au projet ;
   - réseau désactivé par défaut, aucun balayage de tiers et aucune exploitation hors périmètre ;
   - tests séparés pour le moteur, chaque adaptateur et le résultat combiné ;
   - QEMU/VM = SIMULATION_ONLY ; aucune simulation ne vaut preuve matérielle.

## Parc de tests Kali

Kali est utile parce qu'il rassemble des outils de sécurité, mais la présence d'un outil ne prouve ni sa bonne installation ni la validité de son résultat. Construire un inventaire versionné comportant, pour chaque outil : nom, version, licence, source, empreinte du binaire ou paquet quand disponible, rôle, prérequis, tests autorisés, limites, code retour attendu et format de preuve.

Ne pas installer toute la distribution dans le binaire HAKER. L'environnement Kali est un banc d'essai isolé. HAKER-Core doit pouvoir analyser un jeu de preuves enregistré sans Kali ni accès réseau, afin de vérifier que le rapport est reproductible.

## Progression de réalisation

### Étape 0 — Contrat natif et inventaire
- définir les architectures CPU prises en charge et la cible binaire ;
- inventorier les langages, compilateurs, linkers, bibliothèques et composants réellement nécessaires ;
- enregistrer la chaîne de build, les versions, les options, le hash du binaire et les dépendances ;
- établir une base de tests sans prétendre que l'IA est déjà implémentée.

### Étape 1 — Noyau déterministe minimal
- lire le catalogue de cas strictement validé ;
- exécuter uniquement les tests synthétiques autorisés ;
- produire un rapport JSON versionné et stable ;
- vérifier les statuts à partir des codes retour et des preuves, jamais à partir d'un texte d'IA ;
- tester les entrées malformées, les doublons, les limites de ressources et les interruptions.

### Étape 2 — Premier raisonneur propre au projet
- créer des représentations structurées de composants, menaces, préconditions, observations et preuves ;
- implémenter des règles et algorithmes explicables ;
- mesurer précision, rappel et faux positifs sur un corpus gelé ;
- conserver les conclusions non vérifiées au statut HYPOTHESIS / NOT_VALIDATED.

### Étape 3 — Apprentissage de zéro
- spécifier le modèle, les tenseurs, les opérations numériques, la fonction de perte, l'optimisation et le format des poids ;
- implémenter et tester ces composants dans le dépôt ;
- produire les poids à partir de données dont la provenance est documentée, sans importer de poids préentraînés ;
- ajouter des tests numériques, de déterminisme, de sauvegarde/reprise et de résistance aux entrées malformées ;
- mesurer si l'apprentissage améliore réellement le raisonneur de base.

### Étape 4 — Adaptateurs Kali confinés
- activer les outils un par un, avec versions et cas de test ;
- analyser les sorties dans des parseurs dédiés, sans shell construit à partir de données non fiables ;
- comparer les constatations aux fixtures connues et aux résultats indépendants ;
- garder chaque résultat relié à sa preuve brute et son empreinte.

### Étape 5 — Évaluation indépendante
- comparer HAKER seul, les outils seuls et HAKER avec outils ;
- utiliser le même corpus, les mêmes droits et les mêmes limites de temps ;
- publier les cas manqués, faux positifs, résultats négatifs et limites ;
- toute affirmation de niveau expert ou supérieur exige un protocole comparatif reproductible et une revue indépendante.

## Statuts de preuve

- **PASS** : contrôle réellement exécuté et preuve requise vérifiée.
- **FAIL** : test exécuté et échec confirmé.
- **PARTIAL** : une partie seulement a été vérifiée.
- **BLOCKED** : prérequis manquant ou action interdite.
- **NOT_IMPLEMENTED** : composant absent.
- **NOT_VALIDATED** : composant ou résultat non vérifié.
- **SIMULATION_ONLY** : résultat provenant d'une simulation ou VM.

Un fichier de conception, un commit, une compilation réussie ou un workflow configuré ne prouve pas à lui seul que HAKER est une IA fonctionnelle. Les tests doivent être exécutés et leurs résultats examinés avant de passer à PASS.

## Garde-fous

- cible par défaut : ce dépôt, ses artefacts de test et les laboratoires jetables explicitement autorisés ;
- les scénarios peuvent simuler l'exfiltration, la propagation ou la persistance pour tester la détection, mais ne doivent pas transmettre de vraies données, se propager hors du laboratoire ni installer de persistance cachée sur des systèmes réels ;
- aucune cible tierce et aucun élargissement automatique du périmètre ;
- les actions destructives sont réservées aux copies jetables avec restauration vérifiée ;
- aucune modification firmware/NVRAM/BIOS, aucun flash et aucune action sur la machine principale sans autorisation explicite et revue humaine ;
- aucun secret dans les journaux ;
- aucune fusion, publication ou installation automatique ;
- séparation entre génération d'hypothèses, exécution du test et vérification indépendante des preuves.

## Critères d'acceptation du premier jalon

- le binaire natif et ses dépendances sont identifiés et hachés ;
- le test de démarrage et les tests de cas synthétiques sont exécutés en CI ;
- le rapport est stable, reproductible et lié aux preuves ;
- des tests négatifs démontrent que les entrées invalides et les résultats falsifiés ne produisent pas PASS ;
- le banc Kali est distinct du cœur HAKER ;
- aucune revendication ne dépasse le niveau de preuve effectivement obtenu.
