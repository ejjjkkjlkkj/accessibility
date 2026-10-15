# Objectif zéro faille connue et zéro surprise opérationnelle

**Statut : SPECIFICATION — NOT_IMPLEMENTED / NOT_VALIDATED tant que les contrôles correspondants ne sont pas codés, exécutés et archivés.**

## 1. Objectif réaliste

Le projet vise **zéro vulnérabilité connue non acceptée à la livraison**, zéro régression connue non corrigée et aucune affirmation PASS sans preuve. Aucun logiciel non trivial ne peut garantir mathématiquement « zéro faille » pour tous les usages, matériels et adversaires. Les risques résiduels doivent donc être visibles, bornés, attribués et réévalués.

Une faille réseau, logicielle, de pilote ou du noyau peut faire perdre du temps, interrompre le travail, corrompre des données ou rendre l'accessibilité indisponible. Le système doit prévenir ces défauts tôt, détecter les écarts automatiquement et réduire le temps de diagnostic/récupération.

## 2. Invariants non négociables

1. **Fail closed** : si une vérification de sécurité requise échoue, est absente, expirée ou n'a pas été exécutée, le résultat est FAIL/BLOCKED/NOT_RUN, jamais PASS.
2. **Pas de fonctionnalité cachée** : aucun service réseau, destination sortante, collecte, mise à jour ou privilège sans déclaration explicite et justification.
3. **Privilèges minimaux** : composants séparés, permissions minimales, frontières de confiance testées et interface d'administration explicite.
4. **Contrats versionnés** : ABI, formats binaires, protocoles, événements d'accessibilité, API pilotes et formats de données ont versions, limites, règles d'erreur et tests de compatibilité.
5. **Entrées non fiables** : toute entrée externe ou inter-composants est validée, bornée et testée (tailles, encodage, états invalides, timeouts, concurrence et erreurs).
6. **Provenance vérifiable** : versions et empreintes des dépendances, SBOM, sources, outils de build, artefacts et signatures sont enregistrés selon le niveau de release.
7. **Récupération prévue** : échec de pilote, mise à jour, réseau ou service ne doit pas bloquer le démarrage de récupération ni détruire les données sans avertissement et mécanisme vérifié.
8. **Accessibilité essentielle locale** : commandes de secours, diagnostic, installation et récupération doivent rester utilisables au clavier et par synthèse/braille selon le profil ; aucune dépendance obligatoire au cloud.
9. **Temps de réponse mesuré** : chaque composant critique a des budgets de démarrage, réponse, mémoire, CPU, latence audio et récupération. Dépasser un budget est un échec de test, pas un avertissement ignoré.
10. **Preuves séparées** : analyse statique, tests locaux, CI, VM/QEMU, matériel réel et validation par utilisateurs sont des niveaux différents.

## 3. Couverture obligatoire par couche

### Réseau
- Refus entrant par défaut ; inventaire des sockets TCP/UDP IPv4/IPv6, interfaces, routes, DNS et destinations sortantes.
- Comparaison automatique entre manifeste attendu et état observé ; tout port/service/destination inexpliqué bloque la validation.
- Tests locaux autorisés pour certificats invalides, usurpation/rejeu, messages malformés, fuite DNS, timeouts, reconnexion, panne du serveur et épuisement borné des ressources.
- Vérifier que le filtrage n'empêche pas DHCP, DNS, IPv6, mises à jour, VPN, assistance ou récupération nécessaires.
- Vérifier hors ligne et en réseau dégradé ; aucune télémétrie non déclarée.

### Logiciel et dépendances
- Analyse statique adaptée au langage, avertissements pertinents traités, tests unitaires/intégration, couverture des branches de sécurité critiques.
- Fuzzing des parseurs et formats ; sanitizers lorsque disponibles ; tests de propriétés, limites, erreurs et régression.
- Dépendances verrouillées, inventoriées, contrôlées contre les avis de sécurité et mises à jour de façon traçable.
- Builds reproductibles lorsque possible, artefacts hachés, SBOM et provenance de build.
- Pas de secret dans le dépôt, les logs, les rapports ou les artefacts.

### Pilotes et matériel
- Contrat par pilote : matériel/ID pris en charge, DMA, interruptions, mémoire, concurrence, privilèges, limites et comportement en cas d'erreur.
- Isolation et IOMMU/DMA quand pris en charge ; aucun accès mémoire ou périphérique sans autorisation.
- Tests de données invalides, déconnexion, reset, timeout, manque de mémoire, panne et reprise.
- Matrice explicite des combinaisons matériel/firmware/pilote. Simulation et tests physiques rapportés séparément ; un test simulé ne prouve pas le matériel.

### Noyau, démarrage et services
- Vérification de la chaîne de démarrage, des signatures/provenances et des politiques de retour arrière.
- Isolation entre noyau, pilotes, services et applications ; droits minimaux et transitions de privilèges auditées.
- Tests de corruption, crash, redémarrage d'un service, stockage plein, panne d'alimentation simulée et restauration.
- Le diagnostic et le mode récupération doivent être accessibles même si l'interface principale, un pilote ou le réseau échoue.

### Accessibilité et interaction
- Tester clavier, focus, rôles/états, annonces, braille, synthèse vocale, langue, contraste et parcours de récupération.
- La sécurité ne doit pas retirer silencieusement le focus, masquer une erreur ou rendre un dialogue impossible à utiliser sans souris.
- Le lecteur d'écran de base ne dépend pas d'une IA distante ; les erreurs critiques ont une annonce accessible et une sortie de secours.

## 4. Portes de validation de livraison

- **G0 — Contrat** : exigences, menace, interfaces, budgets de performance et critères d'acceptation documentés.
- **G1 — Provenance** : dépendances inventoriées, versions verrouillées, artefacts et sources identifiables.
- **G2 — Qualité** : formatage, compilation, analyse statique et tests pertinents exécutés ; résultat archivé.
- **G3 — Adversarial** : fuzzing et cas HAKER pertinents exécutés dans un laboratoire possédé/autorisé ; aucune vulnérabilité critique/haute ouverte.
- **G4 — Intégration** : contrats inter-couches et compatibilité vérifiés ; aucun port, privilège ou flux non déclaré.
- **G5 — Résilience** : panne, reprise, rollback et mode hors ligne testés.
- **G6 — Accessibilité** : parcours critiques au clavier et avec technologies d'assistance testés ; défaut bloquant non résolu = blocage.
- **G7 — Release** : rapports, SBOM/provenance, limites connues, preuves par environnement et procédure de récupération publiés.

Une porte non applicable doit être marquée NOT_APPLICABLE avec justification. Une porte non exécutée est NOT_RUN. Une réussite en CI n'implique pas une réussite sur matériel physique.

## 5. Réduire le temps perdu

- Reproduire chaque défaut sous forme de test de régression avant de corriger ; conserver le test ensuite.
- Donner à chaque échec un identifiant, une couche, une gravité, des étapes minimales, des logs accessibles et un responsable.
- Utiliser des tests rapides sur chaque changement, puis tests d'intégration, fuzzing prolongé et matrices multi-OS/matériel à des étapes adaptées.
- Mettre en cache uniquement des dépendances identifiées et vérifiées ; ne jamais remplacer une validation par un cache.
- Éviter les builds complets inutiles, mais ne pas ignorer un test requis : indiquer explicitement le niveau de preuve manquant.
- Définir des SLO mesurables pour démarrage, réponse, stabilité, accessibilité et récupération ; suivre les régressions dans le temps.
- Pour toute modification réseau, pilote, noyau, mise à jour ou accessibilité : exécuter les tests ciblés et les tests de régression des frontières touchées.

## 6. Règle de gravité et de blocage

- **CRITICAL/HIGH ouverts** : release bloquée, sauf décision de risque documentée et explicitement autorisée par les responsables du projet ; aucune exception silencieuse.
- **MEDIUM/LOW** : triage avec propriétaire, justification, échéance et mesure compensatoire ; les risques touchant confidentialité, intégrité, disponibilité ou accessibilité peuvent devenir bloquants selon l'impact.
- **Régression, provenance inconnue, contrôle non exécuté, rapport manquant ou résultat contradictoire** : NOT_VALIDATED/BLOCKED.
- Un score « 0 faille » n'est jamais calculé à partir de l'absence d'alertes seulement. Il faut documenter la couverture, les outils, les limites et les tests réellement exécutés.

## 7. État actuel

- **PASS documentaire** : les exigences ci-dessus sont définies dans ce document.
- **NOT_IMPLEMENTED** : cette spécification ne crée pas à elle seule les analyseurs, le moteur HAKER, l'isolation des pilotes, le contrôle réseau ni les mécanismes de récupération.
- **NOT_VALIDATED** : aucun résultat de ce document ne prouve l'absence de vulnérabilités, la performance sur PC physique ou l'accessibilité complète.
- Prochaine étape technique : implémenter un moteur HAKER minimal qui exécute des cas déterministes, refuse les faux PASS et produit des rapports JSON/textes avec codes de sortie stables ; ensuite brancher les contrôles réseau et la CI.

## Références de base

- NIST Cybersecurity Framework 2.0
- NIST SP 800-218, Secure Software Development Framework (SSDF)
- NIST SP 800-193, Platform Firmware Resiliency Guidelines
- NIST SP 800-207, Zero Trust Architecture
- UEFI Specification et ACPI Specification
- W3C WCAG 2.2 et WAI-ARIA 1.2

Ces références orientent la conception ; leur citation ne constitue pas une certification de conformité.
