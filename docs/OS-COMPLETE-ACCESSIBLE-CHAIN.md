# OS complet, chaîne logicielle accessible et validation HAKER

**État global : ARCHITECTURE PROPOSÉE — OS complet non implémenté/non validé par ce document.**  
Ce plan définit une chaîne de réalisation mesurable. Une couche n'est « fonctionnelle » que lorsque son code, ses tests, ses dépendances et ses preuves sont disponibles. Un prototype, une compilation ou une VM ne prouve pas le fonctionnement sur matériel physique.

## Objectif

Construire un système d'exploitation complet et une chaîne logicielle cohérente, accessible dès le démarrage, utilisable au clavier et avec lecteur d'écran, braille et voix, sans rendre le fonctionnement essentiel dépendant du cloud ou d'un modèle d'IA. ZERO est destiné aux composants de production ; HAKER est destiné à les mettre à l'épreuve, trouver les défauts dans un périmètre autorisé, produire des preuves et vérifier les corrections.

« Complet » signifie couvrir toutes les couches listées ci-dessous, pas prétendre que tout est déjà développé ni que toute combinaison matérielle est compatible.

## Architecture de bout en bout

### L0 — Matériel, firmware et démarrage
- CPU/architecture, RAM, interruptions, temporisateurs et gestion de l'énergie.
- Découverte et description des périphériques (PCI/PCIe, ACPI, SMBIOS, USB, HID, audio, stockage, réseau), avec séparation nette entre capacité détectée et capacité testée.
- UEFI/firmware : démarrage, console texte, GOP/framebuffer, chargement et vérification des artefacts.
- Chemin de secours accessible : messages de démarrage textuels, clavier, diagnostic vocal lorsque l'audio est effectivement disponible.
- HAKER : parseurs de structures, entrées tronquées/malformées, limites, états incohérents, provenance et intégrité des artefacts. Les écritures firmware, NVRAM, flash et redémarrages sont exclus des tests automatiques par défaut.

### L1 — Bootloader et transfert au noyau
- Vérification des formats, signatures et versions ; politique explicite en cas de vérification échouée.
- Carte mémoire, paramètres de démarrage, transfert de contrôle et journal de démarrage reproductible.
- Retour arrière et récupération documentés ; pas de mise à jour silencieuse non validée.
- HAKER : tests négatifs des images et métadonnées, tailles limites, intégrité, séquence de démarrage et récupération en image jetable.

### L2 — Noyau
- Gestion mémoire virtuelle/physique, allocateur, isolation des espaces d'adressage et protections.
- Ordonnanceur, synchronisation, interruptions, temporisateurs, IPC, gestion des erreurs et arrêt contrôlé.
- Modèle de capacités/permissions, séparation noyau-utilisateur et journal d'audit.
- Interfaces stables, versionnées et documentées ; aucune frontière ABI n'est considérée sûre par défaut.
- HAKER : invariants, tests de propriétés, fuzzing des interfaces, débordements, erreurs de concurrence, épuisement de ressources borné et tests de récupération.

### L3 — Pilotes et périphériques
- Clavier/HID, braille, affichage, audio, stockage, USB, réseau, Bluetooth et gestion d'énergie.
- Pilotes isolés lorsque possible, permissions minimales, temporisations, annulation, déconnexion/reconnexion et repli sûr.
- Contrat commun pour périphériques physiques et virtuels, avec identifiants de provenance et statut explicite.
- HAKER : périphériques simulés et fixtures en CI ; tests matériels séparés, ciblés et autorisés. Un pilote simulé ne vaut pas validation physique.

### L4 — Services système
- Système de fichiers, comptes et identités, permissions, configuration, horloge, journalisation, réseau, IPC, installation, mises à jour et récupération.
- Gestion des secrets, stockage chiffré si prévu, mises à jour signées, manifeste logiciel et provenance des builds.
- Services sans interface graphique utilisables par commandes accessibles.
- HAKER : contrôles d'autorisation, chemins et noms malformés, erreurs de permissions, corruption, reprise après panne, intégrité des paquets et tests de mise à jour/retour arrière sur images jetables.

### L5 — Modèle d'accessibilité universel
- Arbre sémantique avec rôles, noms, états, valeurs, relations, actions, focus, sélection et événements.
- Contrat versionné partagé entre composants, avec adaptateurs propres à chaque backend et langage.
- Navigation clavier complète, raccourcis configurables, focus visible et logique, ordre de lecture, annonces non redondantes, historique/navigation et gestion des dialogues.
- API utilisable sans OCR comme mécanisme primaire. OCR/vision est un complément pour les surfaces qui n'exposent pas de sémantique.
- Braille et synthèse vocale derrière des interfaces testables, avec détection de disponibilité réelle du périphérique.
- Chemin de secours indépendant de l'IA, du réseau et d'une application de lecteur d'écran tierce.
- HAKER : tests d'accessibilité automatisables, événements perdus/dupliqués, focus incohérent, rôles manquants, navigation clavier, localisation, sorties structurées et tests avec lecteurs d'écran. Les tests automatisés ne remplacent pas la validation par des utilisateurs concernés.

### L6 — Interface système et environnement de bureau
- Interface graphique et/ou textuelle, gestionnaire de fenêtres, terminal, paramètres, notifications, presse-papiers, dialogues et gestion des sessions.
- Toutes les fonctions critiques doivent avoir un parcours clavier et une représentation sémantique.
- Première configuration, connexion, réseau, mises à jour et récupération accessibles sans aide visuelle obligatoire.
- Mode texte de secours exploitable avec lecteur d'écran dès que la pile audio est disponible.
- HAKER : parcours end-to-end, erreurs de focus, dialogues bloquants, états de chargement, régression de navigation et récupération après plantage.

### L7 — Applications et outils intégrés
- Gestionnaire de fichiers, terminal, éditeur, paramètres, explorateur de processus, installateur, navigateur selon périmètre, outils réseau, aide, mises à jour et outils de diagnostic.
- API documentées et stables ; permissions et capacités minimales ; données exportables dans des formats ouverts.
- Chaque application déclare son niveau de conformité, ses raccourcis, ses rôles/états et ses limites connues.
- HAKER : tests unitaires, intégration, fuzzing des formats importés, tests de permissions, tests de non-régression et parcours accessibles. Les tests contre des services tiers sont exclus sans autorisation explicite.

### L8 — Langages, SDK et chaîne de construction
- ZERO : grammaire, sémantique, types, diagnostics, bibliothèques, compilateur, linker/format objet, débogage, paquets et profils hosted/freestanding/UEFI/kernel.
- Interopérabilité contrôlée avec C, Rust, Go et autres langages via des ABI documentées ; chaque implémentation reste testée indépendamment.
- Builds déterministes quand possible, verrouillage des dépendances, SBOM, provenance, signatures et vérification des artefacts.
- Outils développeur accessibles : compilateur en CLI, diagnostics structurés, débogueur, profiler, test runner, documentation et SDK.
- HAKER : attaques de parseur/compilateur, tests différentiels, génération de cas, corpus de régression, analyse statique et vérification de la chaîne de dépendances.

### L9 — IA locale et fonctions assistées
- IA facultative pour aide contextuelle, OCR complémentaire, résumé et assistance aux développeurs.
- Les fonctions essentielles de démarrage, navigation, clavier et sortie de secours restent disponibles sans modèle, cloud ou connexion.
- Provenance/version du modèle, permissions d'accès aux données, limites de ressources et commandes d'arrêt explicites.
- HAKER : entrées adversariales synthétiques, tests de confidentialité, fuite entre contextes, résultats trompeurs, résistance aux instructions non fiables et vérification que l'IA ne peut pas contourner les permissions du système.

### L10 — Installation, mises à jour et cycle de vie
- Image d'installation accessible, partitionnement avec confirmation explicite, détection de compatibilité, installation, désinstallation, mise à jour, restauration et effacement documenté.
- Aucune opération destructive sans confirmation explicite ; prévisualisation des changements et journaux récupérables.
- Matrice matérielle testée, liste de limitations et procédure de retour à un état fonctionnel.
- HAKER : tests sur images jetables, interruptions d'installation, corruption, signatures invalides, retour arrière et récupération. Aucune modification du PC réel n'est déclenchée automatiquement par la CI.

## Boucle HAKER obligatoire pour chaque composant

1. Définir exigence, menace, actif, périmètre, préconditions et oracle de résultat.
2. Écrire les tests normaux et négatifs avant de déclarer le composant prêt.
3. Exécuter compilation, analyse statique, tests unitaires et intégration.
4. Attaquer le composant avec HAKER dans un bac à sable local borné ou un laboratoire explicitement autorisé.
5. Capturer version, commit, environnement, graine, commande, code retour, entrée minimisée, sortie, journaux et empreintes des artefacts.
6. Sur échec : FAIL, reproduire, corriger, ajouter une régression et rejouer les tests affectés.
7. Sur test impossible : BLOCKED ; sur couverture partielle : PARTIAL ; sans preuve suffisante : UNPROVEN ; non lancé : NOT_RUN.
8. Bloquer l'acceptation si un test obligatoire échoue, est ignoré ou n'a pas été exécuté.
9. Après correction, relancer la suite complète pertinente et comparer les résultats.

HAKER ne doit pas être limité à une liste de techniques : l'objectif est un moteur exécutable, un langage versionné, des adaptateurs et des rapports vérifiables. Tant que ces éléments ne sont pas implémentés, le statut reste NOT_IMPLEMENTED.

## Portes de validation et preuves

- **G0 — contrat** : architecture, menaces, exigences et critères mesurables.
- **G1 — composant** : tests unitaires, tests négatifs, analyse et artefacts.
- **G2 — intégration** : contrats inter-couches et tests de bout en bout.
- **G3 — attaque HAKER** : corpus exécuté, régressions et rapports.
- **G4 — chaîne de livraison** : SBOM, provenance, signature et reproductibilité selon capacité.
- **G5 — simulation** : QEMU/VM et périphériques virtuels, rapportés uniquement comme simulation.
- **G6 — matériel physique** : appareil, firmware, périphériques, versions et protocole consignés ; aucune extrapolation depuis la simulation.
- **G7 — accessibilité humaine** : essais clavier/lecteur d'écran/braille/voix et résultats des utilisateurs, sans les remplacer par un score automatisé.

Chaque porte indique PASS, FAIL, PARTIAL, BLOCKED, UNPROVEN ou NOT_RUN. Aucun score global « sécurisé » ne masque un sous-système non testé.

## Ordre de réalisation pragmatique

1. Stabiliser les contrats sémantiques d'accessibilité, les formats de preuve et les exigences de sécurité.
2. Construire un exécuteur HAKER local minimal avec catalogue, tests déterministes, rapports JSON/texte et codes de sortie fiables.
3. Faire exécuter HAKER dans la CI sur les composants existants et empêcher un PASS si les tests sont absents/ignorés.
4. Créer un vertical slice boot → noyau minimal → clavier → arbre sémantique → navigation accessible → sortie vocale, en séparant simulation et matériel.
5. Ajouter mémoire, processus, pilotes, services, stockage, installation et applications par couches, avec attaques et régressions à chaque étape.
6. Implémenter ZERO progressivement, d'abord une tranche compilateur/testée, puis les profils système ; ne pas déclarer le langage complet avant.
7. Élargir la matrice de matériel réel uniquement avec essais identifiés et autorisés.

## Statut actuel

- Architecture de chaîne complète : **documentée par ce plan**, pas implémentée.
- ZERO compilateur/langage complet : **NOT_IMPLEMENTED** selon les spécifications existantes.
- HAKER exécuteur/langage complet : **NOT_IMPLEMENTED** selon les spécifications existantes.
- OS complet issu de ce plan : **NOT_IMPLEMENTED / NOT_VALIDATED**.
- Accessibilité physique de bout en bout : **UNPROVEN** tant que les essais matériels et humains ne sont pas réalisés.
