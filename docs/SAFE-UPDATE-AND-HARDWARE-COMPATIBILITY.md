# Politique de mise à jour sûre — OS, pilotes, UEFI et matériel réel

**Statut actuel : SPECIFICATION / NOT_IMPLEMENTED / NOT_VALIDATED.** Ce document définit des exigences ; il ne prouve pas qu'un mécanisme de mise à jour est déjà implémenté ni sûr sur un appareil physique.

## 1. Objectif et limite

Une mise à jour ne doit être proposée qu'après vérification de son intégrité, de son authenticité, de sa compatibilité, de sa récupérabilité et de ses résultats de test. Le but est de réduire fortement le risque de rendre le système ou un périphérique inutilisable. Aucune procédure ne peut garantir qu'aucune panne ne surviendra sur tout matériel : les preuves sont limitées aux versions, configurations et appareils réellement testés.

**Interdiction par défaut :** une mise à jour logicielle ordinaire ne modifie jamais BIOS/UEFI, firmware de contrôleur, microcode, NVRAM, Secure Boot, partitions de démarrage ou paramètres matériels. Ces éléments nécessitent une voie séparée, une justification explicite, des mécanismes de récupération propres à la plateforme et un consentement spécifique. Aucun flash firmware, changement NVRAM, redémarrage ou écriture sur l'appareil de l'utilisateur n'est autorisé par la CI.

## 2. Classes de mise à jour isolées

Chaque artefact déclare une classe et une portée. Les classes ne peuvent pas être mélangées implicitement dans un même paquet :

- **A — Données et contenu** : langues, voix, documentation, ressources non exécutables.
- **B — Applications et services en espace utilisateur**.
- **C — Cœur d'accessibilité** : navigation clavier, focus, arbre sémantique, lecteur d'écran, braille, audio et mécanismes de secours.
- **D — Noyau et pilotes** : modules, pilotes PCI/USB/HID, stockage, réseau, GPU, audio, ACPI et gestion d'énergie.
- **E — Démarrage/chargeur/partition système**.
- **F — Firmware et microcode** : catégorie critique, désactivée par défaut et hors du flux normal de mise à jour.

Chaque classe doit avoir des prérequis, un ordre d'installation, une stratégie de retour arrière et des preuves de test dédiées. Un composant qui n'est pas nécessaire à une mise à jour ne doit pas être remplacé.

## 3. Chaîne de confiance du paquet

Avant toute installation :

1. Télécharger dans une zone temporaire isolée, avec limites strictes de taille, de durée et d'espace disque.
2. Vérifier la signature du manifeste avec des clés de confiance ; refuser les signatures invalides, absentes, expirées ou révoquées.
3. Vérifier les hachages de tous les fichiers, la taille, le nombre de fichiers, les chemins et l'absence de traversée de chemins, de liens dangereux, de fichiers dupliqués ou d'écrasements hors destination.
4. Vérifier l'identité du produit, l'architecture CPU, les versions minimales/maximales, l'ABI, les dépendances et les révisions compatibles.
5. Refuser les versions anciennes, les métadonnées expirées, les versions incohérentes et les assemblages incompatibles, sauf procédure de récupération signée explicitement autorisée.
6. Vérifier la provenance du build, le commit source, les dépendances verrouillées, la SBOM, les résultats CI et les attestations requises.
7. Si une vérification est impossible, le paquet est REJECTED, jamais « accepté avec avertissement ».

La signature ne prouve pas l'absence de bogue : elle prouve seulement que l'artefact correspond à une identité de signature reconnue.

## 4. Prévention des régressions avant publication

Toute modification doit déclencher les tests pertinents, avec résultats conservés :

- tests unitaires, d'intégration, de contrat et de compatibilité ABI/API ;
- tests de non-régression sur les versions précédentes supportées ;
- tests différentiels entre l'ancienne et la nouvelle version ;
- tests de fuzzing et de robustesse des parseurs/manifeste, lorsque disponibles ;
- tests de limites mémoire, tailles invalides, délais, erreurs d'E/S et données tronquées ;
- tests d'installation, mise à niveau, interruption, reprise et désinstallation dans un environnement jetable ;
- tests de démarrage, récupération et retour à la version connue comme fonctionnelle ;
- tests de charge, d'espace disque insuffisant, d'arrêt imprévu et de perte réseau simulée ;
- vérification des dépendances et de la chaîne d'approvisionnement ;
- vérification de l'accessibilité avec clavier seul et lecteur d'écran, y compris dialogues d'erreur, progression, consentement et mode de secours.

Tout échec critique, toute corruption de données, toute perte d'accès au démarrage, au clavier, à l'affichage, au stockage, au réseau nécessaire à la récupération ou à la sortie audio accessible bloque la publication.

## 5. Déploiement progressif et arrêt automatique

1. Construire une version immuable, identifiée par version et hachage.
2. La tester d'abord en CI et en environnement jetable ; la simulation est étiquetée SIMULATION_ONLY, jamais preuve matérielle.
3. La publier en canal interne/canari, puis sur un groupe limité d'appareils de test autorisés.
4. Comparer les résultats avant/après : échecs de démarrage, erreurs pilotes, plantages, pertes de périphériques, régressions d'accessibilité, latence audio, consommation et erreurs de récupération.
5. Définir à l'avance des seuils d'arrêt et des critères d'annulation. Une régression grave suspend la diffusion et bloque les appareils/configurations affectés.
6. Élargir le déploiement uniquement après une fenêtre d'observation définie et des preuves enregistrées.
7. Tenir un registre des appareils, versions, pilotes, résultats, anomalies, décisions et approbations. Collecte minimale et respectueuse de la vie privée.

L'absence de rapport d'erreur n'est pas une preuve de succès. Les métriques doivent être interprétées avec des tests reproductibles et des limites de couverture documentées.

## 6. Installation atomique et récupération

- Ne jamais écraser la version active fichier par fichier sans transaction ou mécanisme équivalent.
- Préparer la nouvelle version dans un emplacement inactif ; vérifier les hachages après écriture.
- Garder une version connue comme fonctionnelle, son manifeste signé et une méthode de récupération indépendante du composant mis à jour.
- Activer la nouvelle version seulement après validation de l'intégrité de tous les composants du lot.
- Employer un mécanisme de démarrage à deux emplacements (A/B) ou équivalent lorsque la plateforme le permet ; ne pas prétendre qu'il existe si le partitionnement ne le supporte pas.
- Après redémarrage volontaire et autorisé, vérifier les critères de santé avant de valider la mise à jour.
- En cas d'échec, revenir automatiquement à la version précédente quand c'est techniquement sûr, ou démarrer un environnement de récupération minimal accessible.
- Ne jamais supprimer l'ancienne version ni sa clé de récupération avant la fin de la fenêtre de validation.
- Préserver les données utilisateur ; sauvegarde et restauration doivent être testées séparément.
- Si la récupération est impossible ou incertaine, ne pas installer.

Le retour arrière logiciel ne doit pas contourner les protections anti-retour de sécurité. Une restauration d'urgence doit être authentifiée, journalisée et limitée à une version explicitement autorisée.

## 7. Compatibilité matérielle obligatoire

Une matrice doit relier chaque version aux références exactes : modèle de machine/carte mère et révision, CPU, firmware actuel, contrôleurs, identifiants PCI/USB, versions et paramètres de pilotes, stockage, GPU, Wi-Fi/Ethernet, Bluetooth, audio, clavier, touchpad, écran, batterie/ACPI, TPM/IOMMU et configuration de démarrage.

Pour chaque configuration revendiquée comme supportée, tester au minimum :

- démarrage à froid, redémarrage manuel autorisé, arrêt et reprise de veille ;
- clavier et navigation accessibles avant et après connexion ;
- lecteur d'écran, synthèse vocale et périphérique audio physique ;
- affichage, changement de résolution et GPU ;
- lecture/écriture du stockage, intégrité du système de fichiers et récupération ;
- réseau filaire/sans-fil et reconnexion ;
- USB/HID, touchpad, Bluetooth et périphériques annoncés comme supportés ;
- batterie, alimentation et états de veille lorsque pertinents ;
- installation, mise à niveau, interruption et retour arrière ;
- démarrage/récupération après panne logicielle contrôlée dans un banc autorisé.

Les essais sur un modèle proche, une machine virtuelle ou QEMU ne valent pas validation de l'appareil exact. Enregistrer les différences de révision matérielle, firmware et configuration. Une configuration non testée reste NOT_VALIDATED et ne doit pas recevoir automatiquement une mise à jour risquée.

## 8. Firmware, UEFI, microcode et contrôleurs

- Séparer les mises à jour firmware des mises à jour OS/pilotes.
- Vérifier l'autorité du fabricant, le modèle et la révision exacts, les prérequis, l'alimentation et la procédure de récupération officielle.
- Exiger une protection contre la perte d'alimentation, une image de récupération vérifiée et une méthode de restauration réellement disponible sur la plateforme.
- Ne jamais supposer que la récupération est possible si la mise à jour détruit le bloc de démarrage, la clé de récupération ou un contrôleur essentiel.
- Ne pas flasher un firmware tiers, générique ou seulement « similaire ».
- Pas de mise à jour firmware automatisée sur le PC de l'utilisateur par la CI ou ce dépôt.
- Si le fabricant ne fournit pas de récupération fiable ou si le modèle exact n'est pas confirmé : BLOCKED.

Référence de conception : NIST SP 800-193 traite de la protection, de la détection et de la récupération du firmware de plateforme. Cette référence ne certifie pas l'implémentation du projet.

## 9. Lecteur d'écran : service essentiel

- Garder une version précédente du moteur vocal et des composants essentiels.
- Tester focus, rôles/états, annonces d'erreur, raccourcis clavier, langues, braille et sortie audio.
- Fournir une progression et une annulation annoncées vocalement et accessibles au clavier.
- Ne pas couper le moteur fonctionnel avant que le nouveau soit prêt.
- Si la synthèse vocale ou le pilote audio échoue, conserver une voie de secours accessible (console/clavier, braille ou autre backend validé) lorsque le matériel le permet.
- Ne pas enfermer l'utilisateur dans une fenêtre de mise à jour inaccessible.
- Distinguer les tests automatisés des essais utilisateurs assistés.

## 10. Tests de panne en banc isolé

Injecter de façon contrôlée et non destructive : réseau perdu, paquet tronqué, signature invalide, espace disque insuffisant, écriture interrompue, arrêt brutal simulé, manifeste expiré, downgrade, composants incompatibles, pilote manquant, périphérique absent, démarrage échoué, synthèse vocale indisponible et récupération échouée. Vérifier que l'installation s'arrête proprement, que l'état antérieur reste utilisable et que le rapport explique l'échec.

Ne jamais simuler une coupure d'alimentation ou une panne firmware sur la machine principale de l'utilisateur. Les tests destructifs se font uniquement sur un banc jetable dédié et récupérable.

## 11. Portes de livraison

- **U0 — Identité et intégrité** : signature, hachages, provenance et manifeste validés.
- **U1 — Compatibilité** : modèle, architecture, ABI, dépendances et firmware connus.
- **U2 — CI** : tests pertinents réussis ; aucune erreur critique/haute non résolue.
- **U3 — Installation** : transaction, interruption et reprise testées.
- **U4 — Récupération** : retour arrière et environnement de secours démontrés.
- **U5 — Accessibilité** : parcours clavier/lecteur d'écran et secours validés.
- **U6 — Canari** : déploiement limité, seuils et arrêt automatique vérifiés.
- **U7 — Matériel réel** : preuves sur chaque configuration revendiquée comme validée.
- **U8 — Firmware** : procédure séparée, modèle exact, récupération et approbation spécifique.

Toute porte non franchie bloque la classe de mise à jour concernée. Un succès QEMU ne fait jamais passer U7. Une mise à jour logicielle peut être validée pour une plateforme sans que la mise à jour firmware le soit.

## 12. Preuves et statut

Chaque exécution produit un dossier immuable contenant version source, commit, hachages, SBOM, attestations, signatures, matrice de compatibilité, commandes, logs, résultats, modèle/révision matériel, anomalies, décision et limites connues. Ne pas publier de données personnelles ni de secrets dans les logs.

Statuts autorisés : PASS (preuves satisfaisant les critères exacts), FAIL, BLOCKED, PARTIAL, SIMULATION_ONLY, NOT_IMPLEMENTED, NOT_VALIDATED. Toute affirmation précise son périmètre. Aucun badge global « matériel compatible » ne peut être déduit d'une compilation ou de tests virtuels.

## Références de conception

- NIST SP 800-193 — résilience du firmware de plateforme : https://csrc.nist.gov/pubs/sp/800/193/final
- Microsoft — blocage des mises à jour connues pour causer des problèmes de compatibilité : https://learn.microsoft.com/en-us/windows/deployment/update/safeguard-holds
- Uptane — menaces de retour arrière, paquets incohérents et interruptions : https://uptane.org/docs/2.0.0/standard/uptane-standard

Ces références sont des bases de conception ; leur présence ne prouve pas la conformité du dépôt.
