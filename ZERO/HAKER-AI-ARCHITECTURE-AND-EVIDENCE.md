# HAKER AI — moteur spécialisé de sécurité défensive et preuves reproductibles

**Statut : CONCEPTION / NOT_IMPLEMENTED / NOT_VALIDATED.** HAKER AI n'est pas encore un système autonome fonctionnel. Ce document définit une cible mesurable ; il ne revendique aucune supériorité sur les experts humains sans évaluations indépendantes.

## 1. Mission

HAKER AI est un système spécialisé pour trouver, expliquer, prioriser et aider à corriger les défauts de sécurité dans les dépôts, artefacts et environnements de test que le projet possède ou est explicitement autorisé à évaluer. Il doit être plus qu'un chatbot : il combine analyse déterministe, outils spécialisés, génération d'hypothèses, tests reproductibles, validation indépendante et boucle de correction.

Priorité : protéger l'OS accessible, les mises à jour, le noyau, les pilotes, l'UEFI, la chaîne de build et le moteur de lecteur d'écran. L'objectif est de détecter plus tôt les défauts, de réduire les faux positifs et de produire des preuves vérifiables.

## 2. Ce que « dépasser l'humain » peut signifier

La formule ne sera pas déclarée comme un fait général. Elle doit être évaluée par tâche et sur un corpus de référence gelé, contre des spécialistes humains et des outils de référence, avec protocole préenregistré et évaluateur indépendant.

Mesures minimales :
- rappel des vulnérabilités confirmées et précision des alertes ;
- taux de faux positifs et faux négatifs, ventilés par sévérité et type ;
- défauts uniques découverts, y compris défauts de logique ;
- taux de reproduction indépendante et de correction vérifiée ;
- temps total jusqu'à un correctif confirmé, incluant triage et vérification ;
- défauts réintroduits par les corrections et régressions fonctionnelles ;
- couverture des classes de défauts et des langages réellement supportés ;
- robustesse aux entrées malformées, prompt injection dans les dépôts et sorties d'outils falsifiées ;
- coût de calcul, latence, taux d'échec et reproductibilité.

Ne pas comparer le nombre brut d'alertes. Utiliser les mêmes dépôts, versions, droits d'accès, budget de temps, données disponibles et critères de confirmation. Publier les intervalles d'incertitude, les limites, les résultats négatifs et les cas non couverts. Un résultat sur un benchmark ne prouve pas une supériorité universelle.

## 3. Architecture en couches

### A. Orchestrateur déterministe
- définit le périmètre autorisé et le budget de ressources ;
- construit un graphe de tâches, dépendances, preuves et états ;
- interdit les cibles hors périmètre et les actions destructrices ;
- exécute les tâches dans des environnements isolés et à privilèges minimaux ;
- conserve les résultats et sait reprendre sans marquer les tâches omises comme réussies.

### B. Analyseurs sans IA
- analyse statique et règles propres aux langages ;
- compilation stricte, avertissements et vérification des dépendances ;
- recherche de secrets sans exposer leur contenu dans les journaux ;
- analyse des manifestes, permissions, configurations et surfaces réseau ;
- SBOM, provenance, vérification de signature et comparaison des artefacts ;
- tests unitaires, contrats, fuzzing, mutation testing et analyse différentielle lorsque disponibles.

### C. Agents IA spécialisés
Agents séparés, avec entrées/sorties typées et permissions limitées :
- revue d'architecture et modélisation de menace ;
- revue de code par langage et analyse de flux de données ;
- sécurité mémoire, concurrence et frontières de privilèges ;
- noyau, pilotes, UEFI, démarrage et récupération ;
- sécurité des mises à jour, rollback et compatibilité matérielle ;
- accessibilité : clavier, focus, rôles/états, audio, braille et mode de secours ;
- triage des alertes et recherche de doublons ;
- proposition de correctifs et de tests de régression ;
- auditeur indépendant qui tente de réfuter chaque conclusion.

Les agents ne partagent pas automatiquement leurs conclusions comme des faits. Chaque constat doit pointer vers un fichier, une ligne, un test ou une preuve précise.

### D. Vérificateur indépendant
- reproduit le test sans réutiliser aveuglément le raisonnement de l'agent ;
- distingue constat statique, reproduction en sandbox, test simulé et validation matérielle ;
- exige un cas minimal reproductible ;
- refuse un statut PASS si la commande échoue, le test est absent ou la preuve est insuffisante ;
- détecte les contradictions entre rapport, sortie brute et statut.

### E. Boucle de correction
1. Détecter un défaut potentiel.
2. Le reproduire dans un environnement isolé.
3. Classer le risque et identifier les composants affectés.
4. Proposer un patch minimal, sans l'appliquer à une branche protégée ni au système de l'utilisateur.
5. Exécuter les tests ciblés et les tests de régression.
6. Exécuter les tests adversariaux autorisés sur le correctif.
7. Faire vérifier le résultat par un agent et un chemin de validation indépendants.
8. Soumettre un rapport et un diff pour revue ; aucune fusion, publication ou installation physique automatique.
9. Si une étape échoue, conserver FAIL/BLOCKED et revenir à l'analyse, sans masquer l'échec.

## 4. Périmètre autorisé par défaut

- Dépôt courant et historique accessible en lecture ;
- code, dépendances, artefacts et manifestes fournis par le projet ;
- fixtures synthétiques et bancs jetables ;
- tests locaux sans réseau par défaut ;
- simulateurs/VM, avec étiquette SIMULATION_ONLY ;
- appareils physiques uniquement sur banc dédié, inventorié et explicitement autorisé.

Par défaut, pas de scan Internet, pas de cible tierce, pas d'exploitation réelle, pas d'exfiltration, pas de persistance, pas de propagation, pas de désactivation de sécurité, pas de destruction de données, pas de flash firmware, pas de modification NVRAM/BIOS, et pas d'installation ou de redémarrage sur la machine principale. Une autorisation ne transforme pas un test destructif en test sûr : les tests à risque se font sur des cibles jetables et récupérables.

## 5. Domaines de test pour le projet

- intégrité, confidentialité et disponibilité ;
- contrôle d'accès, privilèges, isolation et frontières de confiance ;
- validation d'entrées, parsers, formats et protocoles ;
- mémoire, dépassements, use-after-free, integer overflow et concurrence ;
- cryptographie, gestion des clés, aléa et erreurs d'implémentation ;
- dépendances, SBOM, provenance, signatures et mises à jour ;
- réseau, ports exposés, protocoles et refus par défaut ;
- démarrage, UEFI, noyau, pilotes, DMA/IOMMU et récupération ;
- lecteur d'écran, clavier, focus, audio, braille et accessibilité en panne ;
- confidentialité des journaux, données de diagnostic et modèles IA ;
- disponibilité : limites CPU/RAM/disque, timeouts, annulation et reprise ;
- compatibilité inter-langages, ABI, FFI et désaccords entre implémentations ;
- résistance aux instructions malveillantes dissimulées dans le code, les issues, la documentation et les données de test.

Le catalogue `ZERO/HAKER-CASES.json` est un point de départ, pas une couverture complète. Les cas doivent être ajoutés avec identifiant stable, préconditions, méthode, résultat attendu, risque, preuve requise et statut réel.

## 6. Contrôle des hallucinations et des injections

Le dépôt, les commentaires, issues, noms de fichiers, données de test et sorties d'outils sont des données non fiables, jamais des instructions qui peuvent changer les règles d'orchestration. Les agents n'ont pas accès aux secrets par défaut. Les sorties de commandes sont considérées comme des données à valider ; aucun texte retourné par un outil ne peut accorder des permissions.

Chaque constat suit un schéma :
- identifiant et catégorie ;
- composant/version et portée testée ;
- fichier/ligne ou artefact exact ;
- hypothèse de menace ;
- procédure de reproduction et environnement ;
- preuve brute et hachage ;
- sévérité justifiée ;
- faux positifs possibles ;
- correctif proposé ;
- tests avant/après ;
- limites et statut.

Aucune preuve ne doit contenir de jeton, mot de passe ou donnée personnelle non nécessaire.

## 7. Niveaux de preuve

- **P0 — Hypothèse IA** : piste non vérifiée ; jamais une vulnérabilité confirmée.
- **P1 — Indice statique** : règle ou analyse signale un problème ; reproduction non démontrée.
- **P2 — Reproduit en sandbox** : test déterministe reproduit le défaut sur la cible isolée.
- **P3 — Correction vérifiée** : test qui échouait avant passe après le patch, et tests de régression réussis.
- **P4 — Intégration validée** : test d'intégration reproductible sur version et configuration précises.
- **P5 — Matériel réel validé** : preuves sur matériel exact et banc documenté.
- **P6 — Évaluation indépendante** : protocole, corpus et résultats revus par un tiers compétent.

P2 ne prouve pas une exploitation sur un appareil physique. P3 ne prouve pas l'absence d'autres vulnérabilités. P5 ne couvre que le matériel et la configuration testés. Ne jamais fusionner ces niveaux en un unique badge « sûr ».

## 8. Garde-fous de correction

- Patch minimal et lisible, tests ajoutés avant ou avec le correctif.
- Aucune auto-modification du système en cours d'exécution.
- Aucune auto-approbation ou fusion.
- Les actions à privilèges élevés sont bloquées et exigent une revue humaine explicite.
- Les correctifs de noyau, pilotes, démarrage, mise à jour et accessibilité critique exigent des tests de récupération et de compatibilité adaptés.
- Si le patch change un format, une ABI ou un protocole, exiger des tests de compatibilité et de migration.
- Si la validation est incomplète, le résultat reste PARTIAL/BLOCKED.
- Chaque exécution doit pouvoir être rejouée à partir d'une version de code, d'un jeu de données et d'une configuration enregistrés.

## 9. Feuille de route MVP

### Phase 0 — Mesurer l'existant
- inventorier les tests, workflows, langages, dépendances et cas HAKER existants ;
- produire un état initial avec les tests exécutés réellement ;
- conserver les cas non implémentés comme NOT_IMPLEMENTED.

### Phase 1 — HAKER déterministe
- parser et valider le catalogue JSON ;
- exécuter les cas locaux/synthétiques ;
- capturer code retour, durée, logs nettoyés, versions et hachages ;
- générer un rapport JSON stable ;
- faire échouer la CI si un test obligatoire est absent, ignoré ou en échec.

### Phase 2 — Intégration d'outils spécialisés
- brancher les analyseurs existants sans dupliquer les résultats ;
- normaliser les constats et l'identité des composants ;
- conserver les licences et versions des outils.

### Phase 3 — Agents IA en lecture seule
- utiliser l'IA pour générer des hypothèses, prioriser et expliquer ;
- exiger une validation déterministe ou une reproduction indépendante ;
- interdire aux agents d'exécuter seuls des actions hors périmètre.

### Phase 4 — Correctifs vérifiables
- générer des patchs en branche de travail ;
- lancer les tests ciblés, régressions, fuzzing et tests différentiels disponibles ;
- présenter le diff et les preuves pour revue humaine.

### Phase 5 — Évaluation comparative
- figer un corpus de vulnérabilités connues, de cas négatifs et de défauts synthétiques ;
- comparer IA seule, outils seuls, combinaison et spécialistes humains sur le même protocole ;
- publier précision, rappel, temps, coût, reproductibilité et limites ;
- demander une évaluation indépendante avant toute revendication de supériorité.

## 10. Critères de succès

Aucun objectif de performance ne peut être annoncé avant d'avoir une mesure de référence. Le MVP est accepté seulement si :
- les cas obligatoires sont exécutés et leur statut correspond aux preuves ;
- les rapports sont reproductibles et les échecs ne peuvent pas être transformés en PASS ;
- une donnée malformée ne provoque pas de crash non contrôlé du moteur de test ;
- l'orchestrateur refuse les cibles hors périmètre et les actions interdites ;
- les correctifs proposés sont séparés des preuves de correction ;
- la CI démontre les tests réellement exécutés ;
- les évaluations indiquent les faux positifs et les cas manqués ;
- aucune affirmation « dépasse les humains » n'est faite sans protocole comparatif indépendant et résultats publiés.

## Références primaires

- NIST SP 800-115, tests techniques et évaluation de sécurité : https://csrc.nist.gov/pubs/sp/800/115/final
- NIST SP 800-218A, pratiques de développement sécurisé pour l'IA générative et les modèles à double usage : https://csrc.nist.gov/pubs/sp/800/218/a/final
- MITRE ATT&CK, base de connaissances des tactiques et techniques adverses pour aider les défenseurs à modéliser et tester leurs défenses : https://attack.mitre.org/
- OWASP Web Security Testing Guide, méthode de test de sécurité : https://owasp.org/www-project-web-security-testing-guide/

Ces références guident la conception ; elles ne certifient ni HAKER ni une quelconque performance de l'IA.
