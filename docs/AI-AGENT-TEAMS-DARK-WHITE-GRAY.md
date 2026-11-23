# Équipe d'agents IA — Dark, White et Gray

Statut global : **PARTIAL / NOT_VALIDATED**. Ce contrat définit l'équipe cible et ses garde-fous. Il ne prouve pas que des agents autonomes sont déjà déployés, connectés ou en exécution.

## Objectif

Organiser les agents IA du projet Accessibility en trois équipes complémentaires. Elles travaillent sur les mêmes composants — matériel physique, pont matériel, UEFI/firmware, noyau/runtime, systèmes d'exploitation, lecteur d'écran, accessibilité clavier/voix/braille, réseau, chaîne d'approvisionnement et CI — mais avec des missions et des permissions séparées.

- **Dark Team** : recherche adversariale et Red Team autorisée. Cherche les défauts, construit des reproductions minimales et des tests négatifs dans des cibles explicitement autorisées.
- **White Team** : Blue Team et ingénierie défensive. Corrige les défauts, ajoute protections, détection, journaux minimaux, tests de régression et procédures de récupération.
- **Gray Team** : validation indépendante, corrélation et arbitrage. Vérifie les preuves des deux autres équipes, reproduit les résultats dans un environnement autorisé, évalue l'impact et refuse les conclusions non prouvées.

Le nom d'une équipe ne change pas les règles de sécurité : aucune action offensive non autorisée. Le Gray Team ne remplace pas l'approbation humaine pour les opérations à risque.

## Agents définis

| ID | Équipe | Rôle | Sortie principale |
|---|---|---|---|
| ORCH-01 | Gray | Orchestrateur et répartiteur | Tâches bornées, dépendances et file de travail |
| DARK-01 | Dark | Auditeur Red Team | Scénarios d'abus autorisés, preuves reproductibles |
| DARK-02 | Dark | Fuzzing et tests négatifs | Cas limites, entrées malformées et régressions minimales |
| DARK-03 | Dark | Chercheur de surface d'attaque | Inventaire de surfaces et provenance des sources |
| WHITE-01 | White | Ingénieur de correction | Correctifs ciblés et tests de régression |
| WHITE-02 | White | Défense système / matériel | Contrôles d'accès, validation d'entrées et état sûr |
| WHITE-03 | White | Supply chain et CI | Inventaire dépendances, SBOM, provenance et permissions CI |
| WHITE-04 | White | Accessibilité sécurisée | Tests clavier, lecteur d'écran, focus, voix et braille |
| GRAY-01 | Gray | Vérificateur de preuves | Reproduction, logs, empreintes et statut exact |
| GRAY-02 | Gray | Analyse de risque et triage | Sévérité, confiance, portée et priorité de correction |
| GRAY-03 | Gray | Auditeur de couverture | Écarts entre exigences, modules, tests et preuves |
| GRAY-04 | Gray | Relecteur de release | Décision de gate avec blocages et limites explicites |

## Cycle de travail

1. **Gray / ORCH-01** crée une tâche avec identifiant, cible autorisée, limites, critères d'acceptation et date de révision.
2. **Dark** inspecte le code et les interfaces, puis produit un scénario reproductible non destructif. Les tests actifs sont limités aux fixtures, laboratoires isolés ou cibles dont l'autorisation est documentée.
3. **White** traite le constat dans une branche de travail distincte et ajoute un test de régression.
4. **Gray** vérifie le diff, relance ou demande l'exécution exacte des tests, examine les logs et classe le résultat.
5. L'orchestrateur met à jour le manifeste de tâches. Aucun agent ne fusionne sur la branche principale, ne publie une release ou n'effectue une opération firmware sans autorisation explicite.

Un agent ne doit pas valider seul son propre correctif. Les constats Dark et les corrections White exigent une vérification Gray indépendante.

## Matrice de permissions

| Action | Dark | White | Gray |
|---|---|---|---|
| Lire le code et les journaux autorisés | Oui | Oui | Oui |
| Recherche de sources publiques et analyse statique | Oui | Oui | Oui |
| Tests unitaires / fixtures locales | Oui | Oui | Oui |
| Fuzzing non destructif sur cible isolée | Oui, borné | Oui | Oui |
| Modifier le code dans une branche dédiée | Tests et reproductions uniquement | Correctifs | Outils de validation uniquement |
| Scanner une cible externe | Non sans autorisation explicite et portée enregistrée | Non sans autorisation explicite | Non sans autorisation explicite |
| Vol de secrets, persistance furtive, exfiltration | Interdit | Interdit | Interdit |
| Écrire firmware/NVRAM, flasher BIOS ou redémarrer le PC | Interdit par défaut | Interdit par défaut | Interdit par défaut |
| Accès DMA/MMIO/ports bruts ou modification de périphérique | Interdit par défaut; revue humaine et dispositif de labo requis | Interdit par défaut | Vérification en lecture seule par défaut |
| Lire ou publier des secrets CI | Interdit | Interdit | Interdit |
| Fusionner, publier, accepter un risque critique | Approbation humaine requise | Approbation humaine requise | Approbation humaine requise |

## Contrat d'une tâche agent

Chaque tâche doit inclure :
- task_id, agent_id, team, repository, branch et commit de base ;
- cible autorisée et limites d'action ;
- hypothèse testée, étapes reproductibles et résultat attendu ;
- références aux fichiers modifiés, tests et logs ;
- statut parmi PASS, FAIL, PARTIAL, BLOCKED, NOT_VALIDATED ;
- classe de preuve : STATIC_ANALYSIS, UNIT_TEST, HOSTED_CI, LOCAL_RUNNER, SIMULATION, PHYSICAL_HARDWARE ;
- risque, confiance, limitations, recommandation et relecteur indépendant.

Les secrets et données personnelles doivent être exclus des prompts, logs et rapports, sauf besoin justifié et accès autorisé. Le résultat d'un modèle est une proposition non fiable jusqu'à validation déterministe.

## Priorités du projet

1. Auditer le code existant et les permissions des workflows avant d'activer une autonomie.
2. Tester les parseurs de données matérielles et ACPI avec fixtures malformées, limites et fuzzing borné.
3. Renforcer la sécurité du pont matériel en lecture seule et la séparation entre inventaire OS et preuve physique.
4. Auditer le pipeline UEFI, les dépendances, les actions CI et le runner local.
5. Tester les événements de focus, rôles/états, entrées clavier, sorties vocales et braille.
6. Produire des rapports de sécurité accessibles au clavier et aux lecteurs d'écran.
7. Ajouter ensuite des agents d'exécution connectés, chacun avec permissions minimales et outils explicitement autorisés.

## Conditions d'activation

La configuration du manifeste n'est pas à elle seule un runtime d'agents. Avant de déclarer une équipe opérationnelle, il faut relier chaque agent à un moteur et à des outils réels, tester ses permissions, conserver les logs et prouver qu'il respecte la matrice. En l'absence de ces preuves, le statut reste **NOT_VALIDATED**.
