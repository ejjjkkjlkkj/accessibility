# Audit critique — angles morts critiques à éliminer

**Revue : 2026-10-15**  
**Branche :** `feature/zero-haker-attack-first`  
**Base observée :** arbre Git de la branche, 113 entrées visibles au moment de la revue.  
**Statut : PARTIAL — inventaire supplémentaire de risques, non une certification de complétude.**

Cette troisième passe cherche les angles morts qui survivent souvent aux listes classiques de vulnérabilités. Elle complète les audits précédents ; elle ne les remplace pas. Une absence dans l'arbre examiné est notée « non repéré », pas « inexistant dans tout l'écosystème ».

## 1. État de couverture du dépôt observé

- **Présent :** documentation d'architecture, cartographie de langages, cœur d'accessibilité dans plusieurs langages, catalogue HAKER, matrice de menaces, documents de portes de validation.
- **Non repéré dans l'arbre examiné :** workflow dédié CodeQL/SAST identifiable, configuration Dependabot identifiable, workflow SBOM/provenance/signature identifiable, politique de divulgation de vulnérabilités identifiable, dossier de conformité réglementaire identifiable, moteur HAKER exécutable identifiable.
- **Important :** une recherche par nom de fichier n'est pas une preuve suffisante de l'absence de contrôles ; il faut aussi examiner le contenu des workflows et les résultats de CI. Les éléments non repérés sont des points de vérification prioritaires, pas une déclaration définitive d'absence.

## 2. Angles morts P0 — confiance, sécurité et livraison

### A. Menaces contre la chaîne de développement elle-même
- PR provenant de forks, scripts exécutés sur code non fiable, injection de commandes par noms de branches, titres, issues, logs ou artefacts.
- Exposition des secrets, permissions excessives des jetons, dépendance compromise, action GitHub substituée, runner persistant contaminé, cache de build empoisonné.
- Attestations et signatures vérifiées à la consommation, et non simplement générées ; séparation entre identité de l'auteur, provenance du build et identité du signataire.
- Environnements de release isolés, approbation humaine des publications, provenance de chaque binaire/ISO, traçabilité de l'outil de compilation.
- Test régulier des procédures de restauration des secrets et de compromission d'un compte mainteneur.

### B. Dépendances et maintenance à long terme
- Inventaire des dépendances directes, transitives, système, compilateur, linker, firmware, actions CI, outils téléchargés et modèles IA.
- Détection des paquets abandonnés, typosquatting, mainteneurs compromis, licences incompatibles, dépendances sans source reproductible et mises à jour non signées.
- Politique de support, versions minimales, échéances de correction, remplacement d'une dépendance abandonnée et reconstruction des versions anciennes.
- Archivage des sources, toolchains et instructions nécessaires pour reproduire ou reconstruire un artefact à long terme.

### C. Modèle de sécurité et risques résiduels
- Registre des actifs, adversaires, préconditions, impacts, contrôles, tests, risques résiduels, propriétaires et dates de réexamen.
- Classement explicite des risques de confidentialité, intégrité, disponibilité, authenticité, provenance, sûreté physique et accessibilité.
- Scénarios de compromission totale d'une couche et de défaillance d'un contrôle ; ne pas compter deux contrôles dépendant de la même racine de confiance comme indépendants.
- Politique d'exception : justification, durée, approbateur, périmètre et date d'expiration. Aucune exception permanente et silencieuse.

### D. Disponibilité et refus de service
- Épuisement CPU/RAM/stockage, allocations non bornées, files d'événements infinies, journaux saturant le disque, tempêtes d'interruptions, périphérique bloqué, boucle de redémarrage et dégradation progressive.
- Watchdogs, quotas, priorités, délais, annulation, back-pressure, limites de journaux et mode dégradé.
- Garantir que le lecteur d'écran conserve un chemin prioritaire de base même si l'IA, le réseau, un service ou un pilote secondaire est bloqué.
- Tester la récupération après arrêt brutal et la disponibilité du chemin de secours sans cloud.

## 3. Angles morts P1 — matériel et démarrage

- Chaîne d'approvisionnement matérielle, composants contrefaits, provenance des firmwares et limites des mises à jour fournisseur.
- Démarrage depuis médias externes, ordre de démarrage, clés Secure Boot, révocation DB/DBX, clés de plateforme, option ROM et rollback.
- Différence entre mesure, attestation et prévention ; définir ce qui est réellement vérifié et qui fait confiance au résultat.
- SMM/SMI, DMA et périphériques capables d'accéder à la mémoire, IOMMU et configuration effective ; une option activée dans l'interface firmware n'est pas une preuve de protection effective.
- TPM : présence, état, propriétaire, PCR, politique, récupération et conséquences d'un effacement ; aucun effacement ou changement de configuration automatique.
- Microcode, erreurs matérielles corrigibles/non corrigibles, mémoire ECC si disponible, surchauffe, batterie défaillante et perte d'alimentation.
- Périphériques malveillants ou malformés, HID se faisant passer pour un clavier, USB/PCIe hot-plug, firmwares périphériques et réinitialisation contrôlée.
- Canaux auxiliaires : timing, consommation, cache, rayonnement et observation physique. Documenter les limites ; ne pas promettre leur élimination absolue.
- Définir le protocole de test physique, l'identification exacte de la machine, les versions, l'opérateur, les commandes, les preuves et le plan de restauration avant les essais.

## 4. Angles morts P1 — accessibilité essentielle

- L'accessibilité doit survivre aux erreurs de démarrage, au mode sans échec, à la récupération, au verrouillage, à l'authentification forte, au changement de mot de passe et aux mises à jour.
- Chemin de parole minimal indépendant de l'IA et du réseau ; comportement explicite lorsque le contrôleur audio ou le périphérique braille ne fonctionne pas.
- Aucune annonce secrète ou sensible sur haut-parleur sans choix utilisateur ; contrôler confidentialité, volume, casque et changements de sortie audio.
- Éviter les annonces en boucle, l'interruption permanente de la parole, les événements perdus, les changements de focus silencieux et les alertes qui masquent la navigation.
- Contrôles de temporisation accessibles, récupération après délai, authentification sans dépendre d'un puzzle visuel, alternative au glisser-déposer et aux gestes complexes.
- Test de langues et écritures : RTL, IME, accents, caractères combinés, noms non latins, ponctuation, lecture des chemins et des erreurs, prononciation des codes et numéros.
- Test de zoom élevé, contraste, réduction du mouvement, sous-titres/transcriptions, couleurs non seules et compatibilité avec paramètres d'accessibilité du système.
- Vérifier les états de sécurité dans le modèle sémantique : avertissements, permissions, provenance, confirmation de transaction et erreurs doivent être annoncés sans rendre les actions dangereuses plus faciles.
- Inclure les utilisateurs aveugles et malvoyants dans les tests de parcours réels ; les audits automatiques ne peuvent pas valider seuls la qualité de lecture ou l'autonomie.

## 5. Angles morts IA locale et agents

- Menaces d'injection de prompt indirecte via pages, documents, dépôts, issues, OCR, sorties d'outils et fichiers locaux.
- L'IA ne doit pas pouvoir étendre ses permissions par le contenu qu'elle lit ; séparer planification, autorisation et exécution.
- Outils par capacités, liste d'actions permises, confirmation pour opérations destructrices, journal d'audit, limites de ressources et bouton d'arrêt fiable.
- Empoisonnement de données/modèles, modèles ou poids substitués, licences, provenance, fichiers de modèle malformés, chargement de code natif et exécution arbitraire.
- Fuites par prompts, historique, cache, télémétrie et synthèse vocale ; définir la politique de rétention et les frontières de confiance.
- Définir une politique pour les hallucinations : aucune réponse IA ne doit être considérée comme preuve de sécurité, de conformité ou d'exécution d'un test.
- Tester les erreurs de reconnaissance vocale/OCR, les faux positifs et négatifs, les ambiguïtés et les actions sensibles. Toute action irréversible doit rester confirmée par une voie accessible.
- Le chemin de navigation, de démarrage et de secours doit fonctionner sans modèle IA chargé.

## 6. Données, vie privée et cryptographie

- Classification des données, minimisation, permissions de lecture, rétention, suppression vérifiable, export, sauvegarde et chiffrement.
- Gestion de clés et secrets : génération, stockage, permissions, rotation, révocation, récupération, séparation des rôles et protection des sauvegardes.
- Choix cryptographiques documentés, bibliothèques éprouvées, génération aléatoire sûre, validation des certificats, expiration, révocation et migration cryptographique.
- Préparation à la transition post-quantique selon les standards et profils pertinents ; ne pas inventer d'algorithme propriétaire.
- Tests de corruption, restauration, rotation de clés, perte de clé, certificat invalide, horloge fausse, certificat expiré et révocation indisponible.
- Journaux minimisés : aucun mot de passe, jeton, clé privée, contenu sensible de l'utilisateur ou secret d'authentification dans les logs et rapports.

## 7. Tests et qualité des preuves

- Distinguer les tests unitaires, propriétés, fuzzing, tests différentiels, intégration, systèmes, compatibilité, sécurité, tests utilisateurs et validation physique.
- Ajouter des tests de mutation pour vérifier que la suite détecte réellement les défauts injectés ; une suite verte peut être inefficace.
- Contrôler le déterminisme, les graines, les délais, les retries, les tests instables, la couverture des branches et les cas ignorés.
- Chaque résultat conserve commit, artefact, OS, architecture, outil/version, configuration, commande, code retour, logs et hash.
- Les tests bloqués ou non exécutés restent visibles ; les rapports ne doivent jamais transformer une absence de test en PASS.
- Vérifier l'intégrité des rapports et protéger les preuves contre la modification après exécution.
- Conserver les reproductions minimales des défauts, les régressions et les avis de sécurité sans publier prématurément des détails exploitables.

## 8. Exploitation, incident et cycle de vie

- Procédure de réponse à incident : détection, confinement, préservation des preuves, analyse, correctif, communication, restauration et retour d'expérience.
- Politique de divulgation coordonnée, canal de signalement, délais de triage, avis de sécurité, identifiants de vulnérabilité si appropriés et coordination avec les fournisseurs.
- Mises à jour hors ligne, mises à jour interrompues, miroir compromis, expiration des certificats, panne de serveur de mise à jour et retour arrière sécurisé.
- Support à long terme, fin de support, migration, effacement sûr, recyclage et transfert de propriété de la machine.
- Procédure de perte/vol de l'appareil, révocation des secrets, protection des sauvegardes et récupération accessible.
- Documentation opérationnelle accessible : diagnostic, journal d'incident, collecte des preuves et restauration sans commande nécessitant la vue.

## 9. Conformité, licences et droits

- Matrice des exigences légales applicables selon les utilisateurs, pays, usage, distribution et statut du produit ; ne pas déclarer une conformité légale par simple présence d'une norme.
- Accessibilité : WCAG2ICT pour appliquer avec discernement WCAG au logiciel non web, EN 301 549, RGAA et exigences pertinentes selon le périmètre.
- Licences et attributions des composants, modèles, voix, polices, datasets, SDK, firmware et documentation.
- Protection des données et vie privée, gestion des données sensibles et politique de télémétrie selon le contexte applicable.
- Traçabilité des composants tiers et conditions de redistribution ; interdiction de supprimer des notices ou d'intégrer des composants dont la licence n'est pas compatible.

## 10. Sources primaires à utiliser comme références, pas comme preuve automatique

- NIST CSF 2.0 : https://www.nist.gov/cyberframework
- NIST SP 800-193 : https://csrc.nist.gov/pubs/sp/800/193/final
- NIST SSDF : https://csrc.nist.gov/pubs/sp/800/218/final
- NIST SBOM : https://www.nist.gov/itl/executive-order-14028-improving-nations-cybersecurity/software-supply-chain-security-guidance-20
- CISA KEV : https://www.cisa.gov/known-exploited-vulnerabilities-catalog
- SLSA v1.2 : https://slsa.dev/spec/v1.2/
- W3C WCAG2ICT : https://www.w3.org/WAI/standards-guidelines/wcag/non-web-ict/
- W3C WCAG : https://www.w3.org/WAI/standards-guidelines/wcag/
- UEFI Forum : https://uefi.org/specifications

## 11. Ordre d'exécution — ne pas tout déclarer terminé d'un coup

1. **P0 :** ajouter un registre de risques et d'exigences avec propriétaire, statut, preuve et échéance.
2. **P0 :** auditer les workflows CI et leurs permissions ; ajouter les contrôles de sécurité réels manquants après vérification.
3. **P0 :** implémenter le moteur HAKER et prouver qu'il échoue sur un défaut de test volontaire contrôlé.
4. **P0 :** générer puis vérifier SBOM, provenance et signatures des artefacts.
5. **P1 :** tests de résilience, récupération, mises à jour et disponibilité.
6. **P1 :** tests IA/agents isolés et permissions par capacités.
7. **P1 :** adaptateurs d'accessibilité natifs et essais avec lecteurs d'écran et utilisateurs.
8. **P1 :** banc physique documenté et porte G8, sans action persistante non autorisée.
9. **P2 :** audit indépendant, analyse des risques résiduels et réexamen périodique.

## 12. Critère de vérité

Un dépôt documentaire ne peut pas démontrer « rien oublié ». Le résultat honnête est un registre vivant d'exigences et de risques, avec couverture mesurée, preuves reproductibles, limites explicites et revue indépendante.

**Statut : PARTIAL / NOT_CERTIFIED.** Cette revue ajoute des angles morts à vérifier. Elle ne prouve pas que chaque risque est couvert, que le logiciel est sûr ou que le matériel est validé.
