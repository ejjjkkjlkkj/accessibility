# Audit des manques — OS complet, accessibilité et HAKER

**Date de recherche : 15 octobre 2026**  
**Statut : AUDIT DOCUMENTAIRE — les points ci-dessous sont des écarts de couverture du plan, pas la preuve qu'aucun code correspondant n'existe dans tout le dépôt.**  
**Règle :** « présent dans un document » ≠ implémenté ; « compilé » ≠ testé ; « testé en VM » ≠ validé sur matériel physique.

## Résumé

Le plan `OS-COMPLETE-ACCESSIBLE-CHAIN.md` donne une bonne vue des couches, mais il ne suffit pas encore à définir un OS complet et livrable. Il manque notamment : un contrat d'architecture et de compatibilité précis, un modèle de menace global, les fonctions de sécurité opérationnelles, les services de réseau et de données détaillés, l'installation/récupération testables, les critères d'accessibilité mesurables, les contrats de pilote, et surtout un moteur HAKER exécutable intégré à la CI.

La documentation existante `ACCESSIBILITY-SCOPE.md` couvre déjà plusieurs domaines (firmware, systèmes, lecteurs d'écran, applications, web et standards). Ce document doit être fusionné dans la matrice de traçabilité du projet plutôt que recréé en parallèle.

## Écarts à traiter

### G-01 — Cibles matérielles et contrats d'architecture
- **Manque :** architecture CPU initiale, modes de processeur, ABI, convention d'appel, modèle mémoire, formats d'objets, cibles et limites de compatibilité.
- **À ajouter :** une matrice explicite `architecture × firmware × matériel × fonctionnalités`, avec support déclaré et preuves distinctes.
- **HAKER :** tests des en-têtes/formats, tailles limites, débordements, métadonnées incohérentes et incompatibilités ABI.
- **Acceptation :** chaque cible a un contrat versionné, un artefact reproductible et des tests ; les cibles non prises en charge sont déclarées.

### G-02 — Modèle de menace, confiance et sécurité du noyau
- **Manque :** acteurs adverses, frontières de confiance, actifs, chemins d'attaque, hypothèses matérielles, privilèges, et réponse aux compromissions.
- **À ajouter :** threat model couvrant démarrage, noyau, pilotes, services, applications, mises à jour, chaînes de build et composants d'accessibilité.
- **HAKER :** tests par frontière de confiance, privilèges minimaux, isolation, escalade de privilèges en laboratoire, corruption d'état et régression.
- **Référence :** NIST CSF 2.0 structure la gestion du risque autour de Govern, Identify, Protect, Detect, Respond et Recover ; la documentation du noyau Linux souligne qu'il faut expliciter les limites de responsabilité et les hypothèses matérielles.

### G-03 — Chaîne de confiance firmware et récupération
- **Manque :** racine de confiance, clés et rotation, révocation, démarrage mesuré, politique d'échec, protection contre le retour à une version vulnérable et récupération après corruption.
- **À ajouter :** conception de signature/vérification, gestion du cycle de vie des clés, journaux de vérification et scénario de restauration ; ne pas confondre Secure Boot, measured boot et chiffrement du disque.
- **HAKER :** signatures invalides, artefacts modifiés, downgrade, clé expirée/révoquée, coupure d'alimentation simulée et retour arrière en image jetable.
- **Référence :** NIST SP 800-193 décrit la protection, la détection des modifications non autorisées et la récupération du firmware.

### G-04 — Identités, autorisations et secrets
- **Manque :** comptes locaux, rôles, élévation explicite, gestion de session, verrouillage, récupération de compte, stockage des secrets et audit.
- **À ajouter :** modèle de capacités/permissions, séparation utilisateur/service, consentement clair et authentification accessible sans diminuer la sécurité.
- **HAKER :** accès croisé entre utilisateurs, permissions trop larges, réutilisation de session, secrets dans les logs, contournement d'autorisations et réinitialisation de compte.
- **Acceptation :** chaque service décrit qui peut appeler chaque action et quelles données sont accessibles.

### G-05 — Réseau et communications
- **Manque :** pile réseau complète, configuration hors ligne, DNS, routage, pare-feu, certificats, gestion proxy, Wi-Fi, Bluetooth, VPN et diagnostic accessible.
- **À ajouter :** politique réseau par défaut, permissions par application/service, validation des certificats, gestion des erreurs sans fuite d'informations et mode hors ligne utilisable.
- **HAKER :** tests sur simulateurs/serveurs locaux autorisés, messages malformés, certificats invalides, résolution incohérente, reconnexion, fuite de données et épuisement borné de ressources.
- **Limite :** aucun scan ou test intrusif sur un tiers sans autorisation explicite.

### G-06 — Données, stockage et résilience
- **Manque :** spécification du système de fichiers, intégrité, quotas, permissions, chiffrement, montage, sauvegarde, restauration et récupération après panne.
- **À ajouter :** stratégie de sauvegarde et restauration testée, formats ouverts d'export, vérification d'intégrité et comportement en cas de disque plein/corrompu.
- **HAKER :** fichiers tronqués, noms/path malformés, liens/path traversal, crash pendant écriture, manque d'espace et restauration d'une sauvegarde corrompue.

### G-07 — Ordonnanceur, énergie et fonctionnement prolongé
- **Manque :** limites CPU/mémoire/IO, priorités, starvation, veille/réveil, batterie, thermique, redémarrage de service et fonctionnement prolongé.
- **À ajouter :** budgets de ressources, politiques de priorité et comportements sûrs lorsque la machine manque de ressources.
- **HAKER :** stress borné, famine, concurrence, veille/réveil simulés, déconnexion d'appareil et tests de stabilité longue durée.
- **Acceptation :** les échecs et dégradations sont annoncés de manière accessible et ne bloquent pas indéfiniment la navigation.

### G-08 — Pilotes et isolation
- **Manque :** contrat de cycle de vie des pilotes, signature/provenance, permissions DMA, isolation, timeout, reset, hot-plug et stratégie lorsqu'un pilote plante.
- **À ajouter :** modèle de capacités des pilotes, gestion des interruptions et buffers, limites DMA/IOMMU lorsque disponibles, et matrice des appareils effectivement testés.
- **HAKER :** pilote virtuel fautif, réponses retardées, événements répétés, buffers incorrects et retrait/réapparition ; essais physiques séparés.
- **Acceptation :** un pilote en panne ne doit pas rendre silencieusement le lecteur d'écran inutilisable sans diagnostic ni chemin de secours.

### G-09 — Accessibilité pré-OS et disponibilité de la voix
- **Manque :** preuve qu'un utilisateur aveugle peut effectuer seul la configuration firmware, choisir un démarrage, installer, s'authentifier, récupérer et mettre à jour sur les appareils ciblés. L'UEFI ne garantit pas à lui seul une sortie audio universelle.
- **À ajouter :** chemin accessible de secours explicite : console texte, clavier, braille ou équipement externe compatible selon capacités, et annonce honnête lorsque l'audio n'est pas disponible.
- **HAKER :** perte de focus, clavier absent, voix indisponible, périphérique audio tardif, messages de démarrage tronqués, dialogues de sécurité et parcours de récupération.
- **Acceptation :** les essais utilisateurs et matériels sont consignés par modèle/firmware ; la simulation est étiquetée simulation.

### G-10 — Contrat sémantique d'accessibilité
- **Manque :** schéma versionné complet des nœuds, relations, noms, états, actions, événements, sélection, texte, tableaux, documents, notifications, focus, erreurs et compatibilité descendante.
- **À ajouter :** correspondance documentée entre le contrat canonique et les API natives de Windows, Linux/BSD, macOS, web et UEFI ; prévoir les fonctionnalités non représentables.
- **HAKER :** tests différentiels d'adaptateurs, événements perdus/dupliqués, focus incohérent, rôle/nom absent, annonce en double et régression de navigation.
- **Références :** WCAG 2.2 pour le contenu web et WAI-ARIA 1.2 pour les rôles/états/propriétés web sont des références utiles, mais ne constituent pas à elles seules une norme complète d'accessibilité pour un noyau ou un OS natif.

### G-11 — Langues, voix, braille et modes d'interaction
- **Manque :** localisation des messages, prononciation, langues RTL, changement de langue, ponctuation, abréviations, braille français et international, commande vocale, accès switch et options de réduction de charge cognitive.
- **À ajouter :** contrat de langue et de sortie ; voix et modèles remplaçables ; navigation essentielle disponible sans IA, réseau ou voix propriétaire.
- **HAKER :** textes Unicode malformés, changement de langue en cours de session, textes mixtes RTL/LTR, ponctuation, annonces interrompues et repli sans synthèse.
- **Acceptation :** tests de langue et d'interaction séparés ; ne pas réduire l'accessibilité à un seul lecteur d'écran ou à l'OCR.

### G-12 — Applications, compatibilité et distribution
- **Manque :** gestionnaire de paquets, dépôts, signatures, dépendances, sandbox applicative, compatibilité ABI, API publique, documentation, SDK et politique de fin de vie.
- **À ajouter :** format de paquet et manifeste, permissions déclarées, canaux stable/test, désinstallation propre, compatibilité et migration de données.
- **HAKER :** paquet malformé, dépendance substituée, signature invalide, installation interrompue, désinstallation partielle, rollback et migration répétée.
- **Acceptation :** provenance des composants et licences disponibles ; les composants tiers ont une version et une politique de mise à jour identifiées.

### G-13 — Chaîne de construction et réponse aux vulnérabilités
- **Manque :** SBOM, provenance, builds reproductibles, signatures de publication, gestion des avis de sécurité, délais de correction, disclosure et suivi des vulnérabilités.
- **À ajouter :** processus de release, inventaire des dépendances, attestations de build, registre des défauts et politique de réponse.
- **HAKER :** dépendances connues vulnérables, fichiers de lock incohérents, artefacts non reproductibles, provenance manquante et détection de secrets.
- **Référence :** NIST SSDF SP 800-218 ; sa révision 1.2 publiée en décembre 2025 est un projet public (IPD), à ne pas présenter comme une version finale.

### G-14 — HAKER exécutable et preuves anti-faux-positifs
- **Manque critique :** HAKER est actuellement spécifié comme projet, mais son moteur complet n'est pas encore implémenté/validé. Le plan ne définit pas encore un format d'exécution testé de bout en bout.
- **À ajouter en priorité :** parseur de catalogue strict, schéma versionné, exécution déterministe locale, limites de temps/mémoire, rapports JSON et texte, codes de sortie stables, hash des artefacts, rejouabilité, tests unitaires et tests négatifs du moteur.
- **Anti-faux-PASS :** test absent, ignoré, bloqué, malformé ou non exécuté doit faire échouer la porte ou produire un statut non-PASS ; les résultats simulés ne sont pas des preuves physiques.
- **Acceptation :** inclure des tests de test-runner qui démontrent qu'un test échoué, manquant ou ignoré ne peut pas être annoncé PASS.

### G-15 — Tests de release, qualité et exploitation
- **Manque :** matrice de compatibilité, critères de performance, latence de parole, disponibilité, restauration, accessibilité des journaux, télémétrie respectueuse de la vie privée et critères de fin de support.
- **À ajouter :** budgets de performance mesurables, mesures de latence et de ressources, tests de non-régression, diagnostics exportables par l'utilisateur et politique de collecte opt-in.
- **HAKER :** tests de charge bornés, erreurs intermittentes, corruption de logs, arrêt d'un service, reprise et vérification qu'aucun secret n'est inclus dans les rapports.

### G-16 — Gouvernance, vie privée et limites de la confiance
- **Manque :** classification des données, minimisation, permissions des agents/IA, journalisation, rétention, effacement, audit des décisions automatisées et gestion des vulnérabilités.
- **À ajouter :** aucune action privilégiée directement à partir d'une sortie IA ; confirmation pour actions destructives ; limites explicites et journaux accessibles.
- **HAKER :** prompt injection sur données synthétiques, fuite inter-session, action au-delà des permissions, divulgation dans les logs et tentative d'exfiltration simulée en environnement isolé.

## Ordre de priorité recommandé

1. **P0 — preuves et HAKER :** implémenter le moteur minimal, ses tests négatifs et son intégration CI. Sans cela, les affirmations de sécurité ne sont pas vérifiables.
2. **P0 — contrats :** choisir les premières cibles matérielles/CPU, définir le modèle de menace, les contrats d'accessibilité et les formats de rapports.
3. **P1 — première tranche utilisable :** démarrage, noyau minimal, clavier, arbre sémantique, navigation et une sortie accessible réellement disponible ; documenter les limites de l'audio UEFI.
4. **P1 — sécurité système :** identités, permissions, IPC, stockage, chaîne de confiance, mises à jour et récupération.
5. **P2 — réseau, pilotes et applications :** implémenter progressivement les services et contrats avec HAKER à chaque étape.
6. **P2 — validation :** CI multi-OS pour les tests logiciels, VM pour la simulation, puis appareils physiques et essais utilisateurs pour les preuves correspondantes.

## Références primaires consultées

- [NIST Cybersecurity Framework 2.0](https://www.nist.gov/news-events/news/2024/02/nist-releases-version-20-landmark-cybersecurity-framework)
- [NIST SP 800-218 — Secure Software Development Framework 1.1](https://csrc.nist.gov/pubs/sp/800/218/final)
- [NIST SP 800-218 Rev. 1 / 1.2 — projet public de décembre 2025](https://csrc.nist.gov/pubs/sp/800/218/r1/ipd)
- [NIST SP 800-193 — Platform Firmware Resiliency Guidelines](https://csrc.nist.gov/pubs/sp/800/193/final)
- [Spécifications UEFI et ACPI](https://uefi.org/specifications)
- [Documentation du modèle de menace du noyau Linux](https://www.kernel.org/doc/html/latest/process/threat-model.html)
- [Documentation de l'auto-protection du noyau Linux](https://docs.kernel.org/security/self-protection.html)
- [W3C WCAG 2.2](https://www.w3.org/WAI/news/2023-10-05/wcag22rec/)
- [W3C WAI-ARIA 1.2](https://www.w3.org/TR/wai-aria-1.2/)

Ces références servent à bâtir une matrice de conformité ; elles ne prouvent pas que le projet les satisfait déjà.
