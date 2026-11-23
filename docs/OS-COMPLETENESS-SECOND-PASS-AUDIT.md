# Audit de couverture — éléments potentiellement oubliés

**Date de revue : 2026-10-15**  
**Branche examinée :** `feature/zero-haker-attack-first`  
**Portée :** inspection documentaire et de l'arborescence GitHub, pas audit dynamique ni certification.  
**Verdict global : PARTIAL — plusieurs domaines sont décrits, mais de nombreux contrôles et composants ne sont pas implémentés ou prouvés.**

Ce document complète `OS-COMPLETE-GAP-AUDIT.md` et `ZERO-KNOWN-VULNERABILITIES-AND-RELEASE-GATES.md`. Il ne remplace pas les portes de validation existantes.

## 1. Constats vérifiables dans le dépôt

| Domaine | État observé | Conclusion |
|---|---|---|
| Architecture OS/accessibilité complète | Documents de conception présents | PARTIAL : architecture cible, pas OS complet |
| HAKER | Spécification et catalogue de cas présents | NOT_IMPLEMENTED : pas de moteur HAKER identifié dans l'arborescence examinée |
| ZERO, langage universel | Exigences et contrats présents | NOT_IMPLEMENTED : pas de compilateur ZERO complet identifié |
| UEFI/bare metal | Décrit dans les plans | NOT_VALIDATED : aucun chemin UEFI exécutable identifié dans ce dépôt pendant cette revue |
| Noyau et pilotes | Planifiés dans l'architecture | NOT_IMPLEMENTED/NOT_VALIDATED pour l'OS complet |
| Cœur d'accessibilité multi-langage | C, Rust, Go, D, Ada, Nim, Swift et Zig ont des composants de base | PARTIAL : événements/cursor/cœur sémantique ne démontrent pas une parité complète ni des adaptateurs natifs par OS |
| Tests CI | Plusieurs workflows présents | PARTIAL : certains workflows sont manuels ou ne valident que documentation/manifests ; un workflow vert ne prouverait pas l'accessibilité physique |
| Validation matérielle | Porte G8 documentée | NOT_VALIDATED : aucune preuve de banc physique dans le dépôt examiné |
| Chaîne d'approvisionnement | Les documents parlent de provenance/SBOM/signatures | NOT_VALIDATED : aucun pipeline complet de preuve SBOM, provenance signée et vérification d'artefacts n'a été confirmé par cette inspection |

Les statuts sont des constats de cette inspection. « Non identifié » ne signifie pas qu'un élément ne peut exister dans une branche ou un système externe non examiné.

## 2. Lacunes à traiter — priorité P0

### P0-A — Transformer HAKER en outil exécutable
- Charger et valider le schéma du catalogue ; rejeter les IDs dupliqués, champs inconnus critiques, chemins hors périmètre et paramètres non bornés.
- Exécuter des tests déterministes sur fixtures locales sans réseau par défaut.
- Produire JSON + rapport texte accessible, version du moteur, commande, horodatage, hash d'entrée/artefact, code de sortie stable et statut individuel.
- Statuts obligatoires : PASS uniquement si le test a réellement été exécuté et son oracle satisfait ; FAIL si échec reproduit ; BLOCKED si prérequis manquant ; PARTIAL/UNPROVEN si couverture incomplète.
- Ajouter des tests du moteur lui-même : catalogue invalide, rapport tronqué, interruption, délai dépassé, sortie non déterministe, chemin malveillant et erreur d'écriture.
- Brancher HAKER à la CI après validation locale. Tant que ce moteur n'existe pas, les cas de catalogue ne sont pas des tests exécutés.

### P0-B — Verrouiller la chaîne de build et de livraison
- Générer un SBOM machine-readable par artefact (format SPDX ou CycloneDX), incluant les dépendances transitives et les outils de build.
- Scanner les dépendances, les avis CVE et le catalogue CISA KEV ; enregistrer date, version des sources, exceptions justifiées et délai de correction. Un scan sans résultat ne prouve pas l'absence de vulnérabilité.
- Ajouter détection de secrets, analyse statique, analyse des licences, analyse de binaires/artefacts et vérification des fichiers de configuration CI.
- Épingler les actions GitHub à des SHA de commit vérifiés, au lieu de dépendre uniquement de tags mouvants ; mettre à jour les versions de manière contrôlée.
- Produire provenance de build et attestations vérifiables ; signer les artefacts et vérifier les signatures avant installation/mise à jour.
- Définir builds reproductibles, génération de hashes, conservation des journaux, politique de rétention et comparaison indépendante des artefacts.
- Protéger les branches, exiger les contrôles requis et limiter les permissions des workflows ; ne jamais donner `write` par défaut aux jobs de test.

### P0-C — Définir la racine de confiance et la récupération
- Modèle de clés : génération, stockage, séparation des rôles, rotation, révocation, expiration et récupération si une clé est compromise.
- Secure Boot, démarrage mesuré, TPM et attestation sont des fonctions différentes : documenter précisément ce qui est utilisé et ce qui n'est pas disponible sur un appareil.
- Bloquer le downgrade vers des versions vulnérables ; définir comment restaurer une version connue saine sans ouvrir une voie de retour arrière exploitable.
- Tester la récupération après image corrompue, mise à jour interrompue, stockage plein, perte de courant simulée, configuration invalide et échec du composant d'accessibilité.
- Prévoir une procédure de révocation d'urgence et une chaîne de récupération accessible, utilisable hors ligne.

### P0-D — Définir des contrats de sécurité vérifiables entre couches
- Frontières explicites noyau/utilisateur, pilotes/services, processus, IPC, plugins, extensions, modèles IA et adaptateurs de langage.
- Permissions minimales par capacité ; refus par défaut ; authentification et autorisation vérifiées côté serveur/service, pas seulement dans l'interface.
- Validation des entrées, limites de taille/temps/mémoire, annulation, quotas, isolation, erreurs sûres et journalisation sans secrets.
- Définir les invariants mémoire, concurrence, arithmétique, ABI, DMA/IOMMU et accès MMIO. Ne pas confondre détection d'un périphérique et pilote sûr/fonctionnel.
- Modèle de menace par actif, adversaire, frontière de confiance, précondition, impact, contrôle, test et risque résiduel.

## 3. Lacunes P1 — système et matériel

### Matériel, firmware et démarrage
- Inventaire par modèle et révision : CPU/microcode, chipset, UEFI, contrôleurs, versions de firmware, pilotes et périphériques.
- Chaîne d'approvisionnement firmware, vulnérabilités connues du fabricant, protection/récupération firmware et politique de mise à jour.
- DMA/IOMMU, SMM/SMI, TPM, Secure Boot, measured boot, options ROM, Thunderbolt/PCIe, USB, contrôleurs intégrés et périphériques hot-plug.
- Gestion des interruptions, temporisateurs, APIC, ACPI, mémoire physique, cache/cohérence, états de veille, thermique, batterie et reprise.
- Tests physiques séparés pour clavier, affichage, audio réel, braille, stockage, Wi-Fi/Ethernet, Bluetooth, veille/reprise et récupération. Aucun résultat QEMU/VM ne peut remplir cette porte.

### Noyau, pilotes et services
- Allocateur, mémoire virtuelle, isolation, ordonnanceur, synchronisation, IPC, système de fichiers, gestion des erreurs, crash dump et watchdog.
- Pilotes isolés quand possible ; délais, annulation, réinitialisation, déconnexion/reconnexion et repli sans blocage du lecteur d'écran.
- Réseau IPv4/IPv6, DNS, TLS, certificats, pare-feu, routage, Wi-Fi, Bluetooth, VPN/proxy, filtrage egress, ports attendus/réels et fonctionnement hors ligne.
- Comptes, sessions, permissions, secrets, chiffrement, sauvegarde, restauration, mises à jour, rollback protégé, horloge et journal d'audit.
- Mesures de performance et de sûreté sous charge, manque de mémoire, disque plein, périphérique lent et fonctionnement prolongé.

### Confidentialité et intégrité
- Minimisation des données, consentement explicite, politique de télémétrie, stockage local, effacement, export et rétention.
- Définir les données que la synthèse vocale, le braille, les journaux et l'IA locale peuvent lire ; protéger le contenu sensible et les annonces sur haut-parleur.
- Aucun accès cloud obligatoire pour les fonctions d'accessibilité essentielles ; si des services distants existent, définir données envoyées, chiffrement, identité du service, durée de conservation et désactivation.
- Protection contre confusion de provenance, substitution d'artefacts, dépendances compromises, attaques de build, empoisonnement de modèles et extensions non fiables.
- Plan de réponse aux vulnérabilités : canal de signalement, triage, identifiant de suivi, avis de sécurité, correctif, divulgation coordonnée et politique de fin de support.

## 4. Lacunes P1 — accessibilité de bout en bout

- Pré-OS : firmware/setup, sélection de démarrage, Secure Boot, chiffrement, récupération et mises à jour firmware ; fournir un chemin accessible réellement disponible, ou déclarer clairement la limite matérielle.
- Modèle sémantique : noms, rôles, états, valeurs, relations, actions, événements, focus, sélection, ordre de lecture et annonce des changements ; versionnement et compatibilité ABI.
- Interaction : clavier seul, raccourcis configurables, absence de piège clavier, focus logique/visible, temporisations ajustables, dialogues, menus, notifications, glisser-déposer avec alternative.
- Sorties : voix, braille, grossissement, contraste, tailles, couleurs non seules, réduction des animations, sous-titres, alertes visuelles/auditives et contrôle du volume.
- Entrées alternatives : dictée, commandes vocales, commutateurs, HID spécialisés et navigation sans geste fin.
- Internationalisation : langues naturelles, scripts RTL, composition IME, noms prononcés, chiffres, dates, ponctuation, césure, changement de langue en cours de lecture et voix hors ligne.
- Interopérabilité native : Windows UI Automation/MSAA, macOS Accessibility API, Linux AT-SPI/AT-SPI2, Android AccessibilityNodeInfo/services, et équivalents pertinents. Un contrat commun ne remplace pas chaque adaptateur natif.
- Applications : login, gestionnaire de fichiers, paramètres, navigateur, terminal, bureau, installateur, mise à jour, gestionnaire de tâches, éditeur, outils de développement, gestion de clés et environnement de secours.
- Validation : tests automatiques + essais manuels NVDA, JAWS, Narrator, Orca, braille et grossissement sur versions/architectures définies ; tests avec utilisateurs concernés ; consigner les régressions et limites.
- IA/OCR : uniquement comme complément lorsque les sémantiques manquent ; indiquer l'incertitude, éviter les actions irréversibles autonomes, confirmer les actions sensibles et offrir une voie sans IA.

Les WCAG couvrent le contenu web ; elles ne suffisent pas à elles seules pour certifier l'accessibilité d'un OS, d'un firmware ou d'un lecteur d'écran natif.

## 5. Langages, compilateurs et projet ZERO

- Matrice par langage : spécification, compilateur, parseur, analyse sémantique, IR, optimisations, backend, linker, runtime, FFI/ABI, débogueur, paquets, documentation et tests de conformité.
- Distinguer chaque langage existant des contrats communs et du langage expérimental NEXORA/ZERO ; ne pas déclarer une langue « prise en charge » parce qu'un fichier de démonstration existe.
- Conformité différentielle entre compilateurs/backends ; fuzzing des parseurs et fichiers objets ; tests de débordement, UB, mémoire, concurrence et exceptions.
- Cibles CPU/OS/ABI déclarées, options de compilation reproductibles, versions des outils, dépendances, licences et statut du support.
- Bootstrap et autohébergement par étapes avec comparaison des binaires, provenance du compilateur de départ et procédure de récupération si le compilateur est compromis.
- Documentation et diagnostics accessibles, messages localisés, codes d'erreur stables et compatibilité des interfaces entre langages.

## 6. CI/CD et gouvernance du dépôt

- Vérifier que chaque workflow est déclenché comme prévu sur push, PR, planification et lancement manuel ; contrôler les chemins filtrés pour éviter qu'un changement critique échappe aux tests.
- Séparer tests de documentation, compilation, tests unitaires, sécurité, intégration, simulation et tests physiques. Les badges ne doivent pas fusionner ces niveaux de preuve.
- Matrice multi-OS/architecture réaliste : ne pas utiliser un runner comme preuve de compatibilité d'une autre architecture ; archiver OS, architecture, outil, commande, code retour et artefacts.
- Ne pas supposer que les versions d'actions, runners, compilateurs ou images sont disponibles ou sûres sans vérifier leur état réel.
- Tester la CI elle-même : permissions minimales, injection par données/PR, secrets exposés, artefacts non fiables, scripts tiers, dépendances des workflows et règles de publication.
- Définir la politique de revue, CODEOWNERS, provenance des contributions, signatures/attestations, protection de branche, gestion des issues de sécurité et procédure de rollback de release.
- Vérifier les liens externes, licences, attributions, notices, fichiers de contribution et instructions de reproduction.

## 7. Sources de référence à intégrer au registre de conformité

- NIST CSF 2.0 : gouvernance, identification, protection, détection, réponse et récupération.
- NIST SP 800-218 SSDF 1.1 ; surveiller SP 800-218 Rev. 1 / SSDF 1.2 en tant que projet de révision tant que son statut reste draft.
- NIST SP 800-193 : résilience du firmware — protection, détection et récupération.
- NIST SP 800-207 : principes Zero Trust, sans le confondre avec un produit ou un simple pare-feu.
- CISA Known Exploited Vulnerabilities (KEV) : priorité aux vulnérabilités connues comme exploitées.
- UEFI Forum, ACPI, PCI-SIG et USB-IF : spécifications et limites de plateforme.
- W3C WCAG 2.2 et WAI-ARIA 1.2 : contenu web et sémantique applicable ; à compléter par les API d'accessibilité natives.
- EN 301 549 et RGAA : exigences applicables selon le contexte juridique et le produit.
- SPDX ou CycloneDX : nomenclature logicielle ; SLSA/in-toto ou équivalent : provenance/attestations de build ; signatures vérifiables des artefacts.

Références officielles :
- https://www.nist.gov/publications/nist-cybersecurity-framework-csf-20
- https://csrc.nist.gov/pubs/sp/800/218/final
- https://csrc.nist.gov/projects/ssdf/publications
- https://www.nist.gov/publications/platform-firmware-resiliency-guidelines
- https://www.nist.gov/publications/zero-trust-architecture
- https://www.cisa.gov/known-exploited-vulnerabilities-catalog
- https://uefi.org/specifications
- https://www.w3.org/TR/WCAG22/
- https://www.w3.org/TR/wai-aria-1.2/
- https://www.access-board.gov/ict/
- https://accessibilite.numerique.gouv.fr/ressources/references/

## 8. Ordre d'exécution recommandé

1. **P0 :** implémenter le moteur HAKER minimal et ses tests ; le brancher sur le catalogue existant.
2. **P0 :** ajouter les contrôles CI réels (dépendances, secrets, SAST, SBOM, provenance/signature) avec rapports archivés et statuts sans faux PASS.
3. **P0 :** construire un registre d'exigences → composant → test → preuve → limitation, puis bloquer les releases lorsque les portes obligatoires manquent.
4. **P1 :** mettre en place les contrats d'isolation/permissions et les tests d'intégrité/mise à jour/récupération.
5. **P1 :** développer les adaptateurs d'accessibilité natifs et les parcours clavier/voix/braille de bout en bout.
6. **P1 :** produire une matrice matérielle, puis effectuer des tests physiques non destructifs autorisés avec preuves horodatées.
7. **P2 :** étendre langages/backends, optimisation, IA locale et autohébergement uniquement après que les contrats et tests de base soient reproductibles.

## 9. Critère de clôture de cet audit

Cet audit documentaire ne peut être marqué « complet » qu'après :
- revue de chaque exigence par un responsable ;
- liens entre chaque exigence, code, test et artefact de preuve ;
- analyse des exclusions et des risques résiduels ;
- vérification indépendante des contrôles CI ;
- validation des composants sur les plateformes déclarées ;
- essais utilisateurs d'accessibilité ;
- porte G8 validée sur chaque modèle matériel revendiqué.

**Statut final : PARTIAL.** Cette revue ajoute une liste de contrôle ; elle ne prouve ni l'absence de vulnérabilités ni la complétude de l'OS.
