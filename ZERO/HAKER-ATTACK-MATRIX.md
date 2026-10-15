# HAKER — cartographie des familles de techniques d'attaque

**État : PLAN DE COUVERTURE — les techniques listées ne sont pas toutes implémentées.**
**Périmètre : systèmes possédés, environnements de laboratoire ou autorisation explicite.** Les catégories dangereuses sont validées par fixtures, simulations ou laboratoires isolés, jamais par ciblage non autorisé.

HAKER doit couvrir progressivement toutes les grandes familles connues, avec un registre versionné. « Toutes » signifie une cartographie vivante, étendue à chaque nouvelle technique publiée ; aucune liste finie ne peut garantir de connaître toutes les attaques futures.

## 1. Logiciels, langages et compilateurs
- Analyse statique et analyse de flux de données.
- Tests unitaires, propriétés, invariants, fuzzing mutationnel et génératif, fuzzing guidé par couverture.
- Tests différentiels et métamorphiques entre compilateurs, versions, architectures et optimiseurs.
- Entrées tronquées/malformées, ambiguïtés de grammaire, profondeur excessive, dépassements de limites et consommation de ressources.
- Défauts mémoire : hors limites, use-after-free, double libération, références invalides, valeurs non initialisées, aliasing incorrect.
- Arithmétique : débordements, troncatures, conversions, précision flottante et hypothèses d'overflow.
- Concurrence : courses, interblocages, famine, ordonnancement, atomiques et défauts de synchronisation.
- Exceptions/panics, erreurs ignorées, états partiellement modifiés et échec de récupération.
- Vérification des invariants, analyse symbolique et exécution concolique quand les outils le permettent.
- Tests de désassemblage, ABI, conventions d'appel, format binaire, liens et chargement.

## 2. Applications, API et données
- Injections : SQL/NoSQL, commandes, expressions, modèles, requêtes et interpréteurs, dans des bancs d'essai.
- XSS, CSRF, confusion de contenu, traversée de chemins, inclusion de fichiers et validation de chemin.
- Désérialisation non sûre, parser différentiels, validation de schéma et confusion de types.
- Contrôles d'accès manquants, autorisation au niveau objet/fonction, séparation des rôles et défauts multi-tenant.
- Authentification, sessions, expiration, réinitialisation, MFA et récupération de compte.
- SSRF, redirections, proxy/confiance d'en-têtes, CORS et politiques de contenu, uniquement sur applications de test.
- Limites de requêtes, pagination, concurrence, idempotence, replay, états incohérents et abus de logique métier.
- Fuites d'informations dans erreurs, journaux, métadonnées, caches et réponses.
- Mauvaises configurations, valeurs par défaut, secrets factices, permissions et interfaces de debug.

## 3. Réseaux et protocoles
- Analyse des protocoles et machines à états, conformité aux spécifications, fuzzing des champs et des longueurs.
- Trames invalides, fragmentation, réassemblage, ordre inattendu, délais, pertes, duplication et reconnexion.
- Authentification des pairs, négociation, downgrade, rejeu, usurpation dans un laboratoire et vérification des certificats.
- TLS et configuration cryptographique : validation de certificats, versions, suites, renouvellement et erreurs de configuration.
- Résilience aux floods uniquement par tests de charge bornés et autorisés, avec arrêt automatique.
- DNS, DHCP, IPv4/IPv6, routage, proxy, WebSocket, HTTP et autres protocoles selon adaptateurs futurs.
- USB, Bluetooth/BLE, HID, audio et protocoles d'accessoires, d'abord sur périphériques synthétiques/virtuels puis matériel de laboratoire.

## 4. Identité, postes et systèmes d'exploitation
- Permissions, élévation de privilèges, frontières de processus, IPC et confusion de contexte.
- Durcissement et configuration : services inutiles, droits excessifs, stockage de secrets, journalisation et mises à jour.
- Isolement processus, conteneurs, VM, sandbox, appels système et interfaces de noyau.
- Persistance et démarrage sécurisé : tests de détection et d'intégrité dans des images jetables ; aucune modification automatique de firmware/NVRAM.
- Gestion de clés, jetons, certificats, stockage local, effacement et permissions d'applications.
- Windows, Linux, macOS, Android, iOS et UEFI : adaptateurs distincts, respect des contrôles de plateforme et des interfaces officiellement autorisées.
- Tests de restauration, mises à jour, rollback, mode dégradé et perte d'alimentation simulée.

## 5. Firmware, matériel et appareils
- Validation des entrées firmware, tables ACPI/SMBIOS synthétiques, parsers PE/COFF/ELF et images de démarrage.
- États de périphériques, registres simulés, DMA/MMIO modélisés, erreurs d'initialisation et timeouts.
- Protocoles de périphériques, limites de buffers, identifiants, descripteurs et transitions de connexion.
- Secure Boot, signatures, mesure d'intégrité et chaîne de confiance : vérification hors ligne et laboratoire.
- Défaillances matérielles simulées, déconnexion, latence, corruption de données et récupération.
- Side channels et canaux auxiliaires : inventaire des risques, tests comparatifs contrôlés et mesures défensives ; pas d'extraction de secrets de tiers.
- Toute validation physique consigne modèle exact, révision, firmware, conditions, instrumentations et limites. QEMU/simulateur n'est pas une preuve matérielle.

## 6. Cryptographie et secrets
- Vérification de l'usage des primitives, modes, nonces, aléas, dérivation de clés, rotation et séparation des clés.
- Tests de vecteurs connus, propriétés, interopérabilité et cas limites.
- Vérification des protocoles de signature, vérification, certificats et formats de clés.
- Détection de secrets accidentellement commis et contrôle de leur exposition avec faux secrets de test.
- Analyse cryptographique avancée uniquement dans un cadre de recherche légal ; ne jamais prétendre « casser » une primitive sans preuve reproductible.
- Vérification des dépendances cryptographiques, de la provenance et des options de compilation.

## 7. Chaîne d'approvisionnement et builds
- Dépendances directes/transitives, licences, versions verrouillées, typosquatting simulé et dépendances abandonnées.
- SBOM, provenance, signatures, attestations, hashes, builds reproductibles et vérification des artefacts.
- Compromission simulée de paquet ou d'étape CI avec fixtures ; moindre privilège des workflows et protection des secrets.
- Actions CI épinglées, permissions minimales, isolation des runners, artefacts non fiables et prévention des fuites.
- Comparaison des artefacts produits sur plusieurs OS/architectures et vérification de la chaîne de compilation.

## 8. Disponibilité, résilience et récupération
- Limites CPU/mémoire/disque, boucles, profondeur, files, timeouts et saturation bornée.
- Déni de service algorithmique testé avec quotas, watchdog et arrêt d'urgence.
- Pannes partielles, pertes de dépendance, réseau coupé, disque plein et journaux indisponibles simulés.
- Sauvegarde, restauration, rotation de clés, reprise, intégrité après incident et preuve de non-régression.
- Aucun test de charge ne doit viser un service tiers ni dépasser le budget fixé.

## 9. IA, modèles et données
- Entrées adversariales synthétiques, robustesse des pré/post-traitements et limites de taille.
- Fuites de données, permissions d'outils, isolation des fichiers, contrôle des sorties et exécution de code généré.
- Injection de prompt évaluée dans un bac à sable ; tests de séparation entre données non fiables et instructions.
- Empoisonnement de données/modèles simulé avec jeux de test ; provenance, versionnage et reproductibilité.
- Évaluations de confidentialité, biais et robustesse selon des critères documentés.
- Outils autonomes limités par capacités, budget, confirmation humaine et journalisation ; pas d'accès arbitraire à la machine.

## 10. Accessibilité et interface humaine
- Parcours clavier, focus, rôles/états accessibles, UI Automation et changements de contexte.
- Tests NVDA, JAWS, Narrator, Orca et interfaces natives selon la plateforme.
- Détection des annonces manquantes, focus perdu, pièges clavier, ordre de lecture et messages d'erreur silencieux.
- Tests sans OCR comme dépendance primaire quand une API sémantique est disponible.
- Vérification que les alertes de sécurité sont compréhensibles par lecteur d'écran et utilisables sans souris.
- Tests de sécurité qui ne dégradent pas l'accès d'urgence ou les fonctions d'assistance.

## 11. Ingénierie sociale — simulations défensives seulement
- Exercices fictifs de phishing, usurpation de marque et demandes d'autorisation, sur comptes de test consentants.
- Mesure de détection, signalement et procédures de réponse.
- Pas de campagnes trompeuses contre des personnes non consentantes, de collecte réelle de mots de passe ou de réutilisation de données personnelles.

## 12. Modélisation des menaces et tests de défense
- STRIDE, arbres d'attaque, chemins d'attaque et analyse des frontières de confiance.
- Purple-team : associer chaque technique simulée à une détection, une alerte, un contrôle préventif et une procédure de récupération.
- Tests de détection, faux positifs/négatifs, couverture des journaux et synchronisation temporelle.
- Vérification des contrôles de défense en profondeur et des scénarios d'échec.
- Réévaluation des risques après chaque correction.

## 13. Fiche obligatoire par technique

Chaque entrée du registre doit fournir :
- identifiant stable, nom, famille et description ;
- actif ciblé et préconditions ;
- environnement et périmètre autorisé ;
- danger, impacts possibles et garde-fous ;
- fixtures/cas synthétiques, oracle attendu et critère d'arrêt ;
- exécution déterministe et procédure de reproduction ;
- résultat de l'attaque et preuve de détection/blocage ;
- correctif, test de régression et limites restantes ;
- références documentaires versionnées ;
- état : PLANNED, IMPLEMENTED, TESTED, PARTIAL, BLOCKED, NOT_APPLICABLE.

## 14. Priorisation
Priorité initiale : parseur/compilateur ZERO, sécurité mémoire et arithmétique, protocoles et validation des entrées, modèle de permissions, fuzzing avec régression, chaîne d'approvisionnement, puis adaptateurs OS/UEFI et tests matériels. La cartographie doit continuer à évoluer et être revue à chaque version.

**Important :** ce document est une cartographie à implémenter. Il ne prouve ni couverture complète, ni exécution réelle, ni absence de vulnérabilité.
