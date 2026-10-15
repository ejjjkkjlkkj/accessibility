# HAKER — Référentiel offensif/défensif et plan d'exécution

**Statut : PLAN DE CONTRÔLES.** Ce document référence des méthodes et définit des contrôles à implémenter. Il ne prouve pas que tous les outils sont intégrés, que tous les tests s'exécutent, ni que le dépôt est exempt de vulnérabilités.

## Objectif

Construire une chaîne de sécurité reproductible pour le dépôt ACCESSIBILITY : rechercher les défauts, les reproduire dans un périmètre autorisé, corriger leur cause, ajouter une régression et demander une vérification indépendante. L'objectif est une qualité de référence, pas une promesse d'absence absolue de faille.

## Références primaires

- MITRE CWE Top 25 : https://cwe.mitre.org/top25/ — classer les faiblesses par impact et prévalence.
- NIST SSDF SP 800-218 : https://csrc.nist.gov/pubs/sp/800/218/final — intégrer la sécurité à tout le cycle de développement.
- NIST SSDF publications : https://csrc.nist.gov/projects/ssdf/publications — suivre les versions finales et les brouillons séparément.
- OWASP Web Security Testing Guide : https://github.com/OWASP/wstg/ — méthode et scénarios de tests de sécurité.
- Google OSS-Fuzz : https://github.com/google/oss-fuzz — modèle de fuzzing continu, corpus et régressions.
- AFL++ : https://github.com/AFLplusplus/AFLplusplus — moteur de fuzzing guidé par la couverture.
- ProjectDiscovery Nuclei templates : https://github.com/projectdiscovery/nuclei-templates — exemples de contrôles déclaratifs; ne lancer que contre les cibles autorisées.
- GitHub CodeQL : https://codeql.github.com/ — analyse sémantique statique et requêtes de sécurité.
- OSV Scanner : https://github.com/google/osv-scanner — détection de dépendances affectées par des avis connus.
- Syft : https://github.com/anchore/syft — génération de SBOM.
- Grype : https://github.com/anchore/grype — analyse de vulnérabilités de composants et SBOM.
- MITRE ATT&CK : https://attack.mitre.org/ — vocabulaire de techniques adverses pour scénarios de détection; ce n'est pas une preuve de vulnérabilité.
- RustSec Advisory Database : https://github.com/RustSec/advisory-db — avis de sécurité liés à l'écosystème Rust.
- cargo-audit : https://github.com/rustsec/rustsec — audit des dépendances Cargo.

Les outils, licences, versions et conditions d'utilisation doivent être vérifiés avant adoption. Ne pas importer un scanner ni une liste de règles sans vérifier ses faux positifs, son périmètre et ses exigences réseau.

## Périmètre d'autorisation

Autorisé par défaut :
- le code de ce dépôt et les artefacts construits par sa CI;
- les entrées malformées générées localement et les fixtures dédiées;
- les tests unitaires, intégration, fuzzing et analyse statique avec temps, mémoire et taille d'entrée bornés;
- les systèmes explicitement autorisés et isolés.

Interdit sans autorisation humaine distincte : viser des tiers, scanner des réseaux externes, voler ou tester des identifiants réels, exfiltrer des données, établir une persistance, détruire des données, contourner des protections sur un système tiers, modifier firmware/NVRAM/BIOS ou redémarrer la machine. La CI ne doit pas exécuter de tels actes.

## Plan de contrôle par couche

### 1. Python, scripts, orchestrateur et rapports
- Tester les limites de taille, profondeur JSON, chemins POSIX/Windows, symlinks, permissions, erreurs I/O, délais et codes de sortie.
- Vérifier les appels de sous-processus, la validation des arguments, l'injection de commande, le traitement d'archives et la désérialisation.
- Vérifier que les rapports ne recopient pas secrets, tokens, clés privées, données personnelles ou contenu arbitraire sensible.
- Ajouter un test de régression pour chaque défaut reproduit.

### 2. Rust / no_std / UEFI
- Tester les parsers et structures avec entrées tronquées, tailles extrêmes, entiers limites, longueurs incohérentes et états invalides.
- Vérifier les calculs de taille, index, conversions, alignement, overflow, limites de buffers et comportement en cas d'allocation impossible.
- Examiner unsafe blocks, invariants de pointeurs, aliasing, durée de vie et accès MMIO.
- Tester les erreurs et modes dégradés sans supposer que le matériel est présent.
- Distinguer strictement compilation, tests hôte, émulation et essais physiques autorisés.

### 3. Chaîne UEFI / artefacts de démarrage
- Inventorier format PE/COFF, architecture, sections, taille, hash et provenance de chaque artefact.
- Vérifier la gestion des erreurs de chargement, des entrées invalides et des dépendances firmware attendues.
- Ne jamais présenter une image construite ou un test d'émulation comme preuve de démarrage physique.
- Toute manipulation firmware/NVRAM/BIOS est hors CI et nécessite une autorisation humaine explicite.

### 4. Dépendances et chaîne d'approvisionnement
- Générer une SBOM pour les composants effectivement construits.
- Vérifier les avis de sécurité, versions, licences, provenance et hashes.
- Épingler les actions CI à des références immuables lorsque possible; limiter les permissions au minimum.
- Ne pas afficher de secrets dans les logs. Séparer les artefacts de test des artefacts de livraison.

### 5. Tests offensifs et fuzzing
- Commencer par des harnesses déterministes et fixtures minimales.
- Imposer limites de temps, mémoire, nombre d'itérations et espace disque; aucun accès réseau par défaut.
- Enregistrer le hash de l'entrée, la version du harness, le commit, le moteur et le résultat.
- Réduire chaque crash à un cas minimal, vérifier qu'il ne contient pas de données sensibles, puis l'ajouter au corpus de régression.
- Une absence de crash n'est pas une preuve d'absence de vulnérabilité.

### 6. Défense, journalisation et preuves AERL
- Chaque finding : identifiant stable, sévérité motivée, commit, composant/chemin, reproduction, résultat attendu/réel, hash d'artefact, correction, test de régression, retest et limites.
- États autorisés : PASS, FAIL, PARTIAL, BLOCKED, NOT_VALIDATED, UNPROVEN.
- Ne jamais déclarer une faille corrigée uniquement sur la base d'un patch non testé.
- Une chaîne de hash détecte certaines altérations, mais pas nécessairement le remplacement complet du journal ni la suppression de sa fin. Utiliser des checkpoints indépendamment protégés si l'on veut détecter ces scénarios.
- AERL ne doit jamais contenir de tokens, secrets, captures d'écran privées ou données brutes non nécessaires.

## Portes de validation avant livraison

1. Tests unitaires et de régression passés sur les environnements annoncés.
2. Analyse statique et dépendances exécutées, avec versions et résultats conservés.
3. Fuzzing borné des parseurs prioritaires; corpus de régression versionné.
4. Aucun finding CRITICAL/HIGH non résolu sans décision explicite et documentée.
5. Contrôle indépendant des corrections critiques/élevées.
6. Rapport AERL cohérent et vérifiable, sans fuite de secrets.
7. Hash et provenance des artefacts consignés.
8. Validation physique reportée séparément; CI/émulation ne vaut pas validation matérielle.

## Tableau de maturité

| Capacité | État de départ | Critère pour passer à PASS |
|---|---|---|
| Scanner HAKER heuristique | PARTIAL | Tests multi-OS réussis, faux positifs documentés et rapport validé |
| Tests de sécurité Python | PARTIAL | Suites reproductibles et régressions prouvées |
| Analyse Rust/UEFI | NOT_VALIDATED comme audit exhaustif | Couverture de composants et contrôles unsafe documentés |
| Fuzzing continu | NOT_VALIDATED | Harnesses bornés, corpus, résultats et artefacts de régression |
| SBOM et vulnérabilités de dépendances | NOT_VALIDATED | SBOM produite et scan versionné sur le commit testé |
| Vérification indépendante | NOT_VALIDATED | Second contrôle traçable sur les corrections critiques/élevées |
| Matériel physique | NOT_VALIDATED | Protocole, matériel, version et preuves de test explicitement consignés |
| Agents IA autonomes | CONTRACT_ONLY_NOT_AUTONOMOUS tant que non exécutés | Exécution réelle, limites d'outils, journaux et évaluation reproductible |

## Règle de publication

Ne jamais utiliser les expressions « zéro faille », « incassable » ou « tout est testé » sans périmètre et preuves suffisantes. Publier les limites, versions, commandes, commits et statuts. L'objectif 99,99 % est une cible de fiabilité, pas une mesure atteinte sans données.
