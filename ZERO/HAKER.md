# HAKER — langage et outillage de validation offensive contrôlée

**État : SPECIFICATION INITIALE — PAS ENCORE IMPLEMENTÉ NI VALIDÉ.**

HAKER est le nom du langage et de l'ensemble d'outils offensifs défensifs du projet. Il est développé dès le début en parallèle de ZERO : on ne reporte pas la sécurité à la fin.

HAKER sert à éprouver ZERO, ses compilateurs, bibliothèques, protocoles, composants d'accessibilité, périphériques physiques/virtuels, profils UEFI/OS et adaptateurs PC/mobile. Il doit aussi pouvoir tester des projets externes uniquement dans un périmètre autorisé. HAKER ne remplace pas ZERO : ZERO est le langage universel de production ; HAKER est le langage et l'atelier d'attaque, de vérification et de preuve.

## Boucle obligatoire : coder → attaquer → corriger → rejouer

Pour chaque changement de code :

1. Définir le comportement attendu et les actifs à protéger.
2. Ajouter au moins un cas de test normal et les cas adversariaux pertinents.
3. Compiler/analyser le changement et lancer les tests existants.
4. Lancer HAKER contre le composant dans un bac à sable local et borné.
5. Enregistrer chaque résultat avec version, commande, configuration, entrée, sortie, code de retour et empreinte des artefacts.
6. Si un défaut est reproduit : statut **FAIL**, créer un cas de régression, corriger et rejouer toute la suite concernée.
7. Si le test ne peut pas s'exécuter : **BLOCKED**, jamais PASS.
8. Si la couverture est incomplète : **PARTIAL** ou **UNPROVEN**, jamais « sécurisé ».
9. N'accepter le changement que lorsque les portes obligatoires passent ; les exclusions et limites restent publiées.

Une attaque qui ne trouve rien prouve uniquement que les tests exécutés n'ont pas trouvé de défaut dans leur périmètre.

## Portée technique à construire

### Langage HAKER
- Grammaire versionnée et format de programme documenté.
- Cas d'attaque déterministes, paramètres bornés, générateurs de données malformées.
- Assertions, invariants, préconditions/postconditions et oracles de résultat.
- Reproductibilité par graine enregistrée ; réduction des cas qui échouent (shrinking).
- Sorties structurées accessibles au clavier et aux lecteurs d'écran, codes de résultats stables.
- Exécution locale sans compte, réseau ou cloud obligatoire.
- Capacités explicites : aucun accès réseau, fichier, processus ou périphérique sans permission accordée dans le profil d'exécution.
- Format de rapport machine-readable et humain, avec hashes, versions et limites.
- À terme, compilation native multi-architecture et bootstrap/autohébergement progressif. Un bootstrap initial peut utiliser des outils existants ; ce n'est pas encore l'indépendance totale.

### Moteurs et outils HAKER
1. Tests unitaires et de propriétés.
2. Fuzzing des parseurs, compilateurs, formats, protocoles et API.
3. Tests différentiels entre compilateurs, backends et versions.
4. Analyse statique, vérification de dépendances et détection de secrets.
5. Tests mémoire, arithmétique, concurrence, erreurs et limites de ressources.
6. Tests de protocoles USB/Bluetooth et de messages, d'abord sur données synthétiques et périphériques virtuels.
7. Analyse des artefacts, de leur provenance, de leur intégrité et de leur reproductibilité.
8. Simulation d'intrusion dans un environnement isolé et autorisé.
9. Tests de récupération : détection, confinement, journalisation et restauration.
10. Adaptateurs de validation par plateforme ; aucune compatibilité PC/iPhone/UEFI n'est supposée sans test propre à la plateforme.

## Ordre de développement

- **H0 — contrat** : définir la portée, les statuts, les rapports et les règles de sécurité.
- **H1 — harnais minimal** : charge les cas, exécute des tests locaux déterministes, collecte les preuves et retourne un code d'échec fiable.
- **H2 — attaque du code ZERO** : parseur, sémantique, arithmétique, mémoire, concurrence et diagnostics.
- **H3 — fuzzing et régression** : corpus conservé, graines reproductibles, réduction et replay automatique.
- **H4 — chaîne d'approvisionnement** : dépendances, secrets, SBOM, provenance, signatures et builds reproductibles.
- **H5 — profils système** : UEFI, noyau, périphériques virtuels, protocoles et adaptateurs d'OS.
- **H6 — validation matérielle** : appareils physiques identifiés, tests autorisés, journaux et limitations ; QEMU ne vaut pas preuve matérielle.
- **H7 — autohébergement** : réimplémenter progressivement les composants HAKER avec ZERO/HAKER et prouver l'équivalence par tests différentiels.

Les phases ne sont pas considérées terminées parce qu'un document existe : chaque phase exige code exécutable, tests automatisés, preuves conservées et limites explicites.

## Portée autorisée et sûreté

Par défaut, HAKER teste le dépôt courant, des fixtures synthétiques, des processus locaux jetables et des machines virtuelles explicitement préparées. Les tests réseau, destructifs, de persistance, d'exfiltration, de firmware ou sur des systèmes tiers sont désactivés par défaut et exigent une autorisation explicite, un périmètre précis et des garde-fous. Pas de balayage de tiers, vol de secrets, propagation, contournement d'accès ni action destructive automatique.

## Statuts obligatoires

- **PASS** : les tests exigés ont réellement été exécutés et les preuves sont présentes.
- **FAIL** : défaut ou invariant violé confirmé.
- **PARTIAL** : une partie seulement de la couverture est exécutée.
- **BLOCKED** : prérequis ou environnement indisponible.
- **UNPROVEN** : absence de preuve suffisante.
- **NOT_RUN** : test non exécuté.

Un job ignoré, un outil absent, un test simulé ou une compilation seule ne peut pas produire PASS pour une propriété non testée.

## Définition minimale de « fiable »

La fiabilité est mesurée par exigences et périmètres, pas par une promesse de sécurité absolue. Une livraison doit inclure : matrice de couverture, tests positifs/négatifs, corpus de régressions, résultats CI, versions d'outils, hashes, dépendances/provenance, vulnérabilités ouvertes, limites et liste des validations physiques réellement réalisées.
