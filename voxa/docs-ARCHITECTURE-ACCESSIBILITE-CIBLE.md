# Architecture cible : rendre accessible chaque étape de l'usage d'un ordinateur sans vision
État des lieux honnête au 2026-10-11. **Voxa Live est une application UEFI, pas un système d'exploitation.** Aucun composant ci-dessous n'est validé sur la machine physique ni avec des utilisateurs aveugles. Statuts : PASS, FAIL, PARTIAL, BLOCKED, NOT_VALIDATED ; « NOT_IMPLEMENTED » est utilisé pour dire qu'il n'existe pas de code.

## Couches (ce qui existe et où)
| Couche | Contenu | Où | État |
|---|---|---|---|
| Cœur sémantique | arbre (rôle, nom, valeur, états, actions), focus, file d'annonces à priorités, suivi des changements | `silicon-core/` (Rust `no_std`) | PASS (22 tests hôte) ; NOT_VALIDATED sur cible ; **non relié** à Voxa Live |
| Adaptateur de menu | menu de démarrage branché sur le cœur via une interface « plateforme » | `silicon-core/src/menu.rs` | PASS (tests hôte, plateforme simulée) |
| Application UEFI Voxa Live | menu, pages d'informations, revue ligne par ligne, épellation, confirmations, secours sonore | `exemples/live3.vx` (langage Voxa) | PASS en émulation (tests/nav.sh) ; NOT_VALIDATED sur matériel |
| Parole | texte → phonétique (`phon.vx`) → voix à formants → HD Audio | `bib/phon.vx`, `bib/voix.vx`, `bib/hda.vx` | PARTIAL (voir RAPPORT-PHONETISATION-v0.md) ; matériel NOT_VALIDATED |
| Pilotes | écran GOP, clavier 8042 + clavier UEFI, haut-parleur système, HD Audio, lecture PCI/SMBIOS/CPUID | `bib/*.vx` | PASS en émulation ; NOT_VALIDATED sur matériel |
| Moteur d'inférence | produit int8, requantification, erreurs explicites | `voxa-neural/` | PASS (9 tests) ; aucun modèle |
| Adaptateur Windows | UI Automation → parole (facultatif) | `src/bin/lecteur.rs` | PARTIAL (autotest 27/28 plus tôt, modes récents non testés) |
| Noyau, mémoire, tâches, fichiers, sécurité | — | — | NOT_IMPLEMENTED |

## Chaque situation d'usage : état réel, dépendances, tests requis, obstacles
| Situation | État réel | Dépendances manquantes | Tests requis | Obstacle principal |
|---|---|---|---|---|
| Démarrage et réglages du firmware (BIOS/UEFI setup) | **NOT_IMPLEMENTED.** Voxa Live n'accède pas aux menus du firmware ; il n'est accessible que si l'utilisateur sait lancer la clé | aucune API d'accessibilité standard dans le firmware ; ne pas modifier le firmware ni la NVRAM | démarrage physique, annonce vocale < N secondes | contrainte de sécurité ; rien d'universel à réutiliser |
| Menu de démarrage de Voxa | PARTIAL : menu parlé, secours sonore, confirmations | essai physique, pilote réel Realtek | scénarios sans écran avec utilisateurs aveugles | audio réel non prouvé |
| Applications natives | NOT_IMPLEMENTED (une seule application de démonstration) | contrat de l'arbre sémantique appliqué par chaque application ; SDK d'application | tâches complètes, p. ex. écrire et enregistrer un texte | aucun noyau pour héberger des applications |
| Système de fichiers | NOT_IMPLEMENTED (lecture FAT réalisée seulement par l'outil de fabrication d'ISO) | pilote de stockage (NVMe), système de fichiers, annonces d'état | copier / renommer / chercher sans vision | NVMe + FS à écrire, risque de perte de données |
| Fenêtres et interfaces graphiques | NOT_IMPLEMENTED côté Voxa ; sous Windows seul l'adaptateur UI Automation existe | gestionnaire de fenêtres sémantique | navigation fenêtre/menu/dialogue | pas de pile graphique |
| Terminal et outils de développement | NOT_IMPLEMENTED | console accessible, éditeur, compilateur dans Voxa OS | écrire, compiler, lire les erreurs au clavier | dépend d'un OS |
| Périphériques et réseau | PARTIAL : inventaire PCI/SMBIOS/CPUID lu et dit ; pavé tactile décodé ; réseau absent | pilotes I2C-HID, xHCI, braille HID (HUTRR78), Wi-Fi MediaTek, pile TCP/IP | essais avec matériel ; afficheur braille indisponible ici | pas de matériel braille ; pilotes volumineux |
| Messages d'erreur et notifications | PARTIAL : erreurs audio annoncées (`speech_failed`, bips-codes, ligne d'état), confirmation des actions sensibles | file de notifications système, journal | injection de pannes, vérification de l'annonce | seulement au niveau d'une application |
| Installation, mise à jour, récupération | NOT_IMPLEMENTED | chargeur, partitions, mise à jour atomique, retour arrière, console de récupération | restauration après panne simulée | interdit de formater ou d'écrire dans la NVRAM sans autorisation |

## Priorités retenues (ordre d'exécution)
1. Parole autonome et fiable : secours sonore sans voix (**fait**, testé en émulation), annonce des échecs, mesure de latence au démarrage (à faire sur matériel).
2. Navigation clavier complète : chiffres, Début/Fin, Échap, confirmations (**fait**, testé en émulation) ; reste : recherche, saisie de texte.
3. Restitution structurée des informations système : pages Matériel / Processeur et mémoire / PCI lues ligne par ligne (**fait**, journal d'annonces vérifié) ; reste : lecture par tableau et par champ.
4. Architecture indépendante : cœur Rust `no_std` testé ; reste : l'adapter à Voxa Live, puis à un noyau.
5. Fondations de Voxa OS : étude préalable (carte mémoire UEFI, `ExitBootServices`, pilotes sans le firmware) ; aucune ligne de noyau écrite.
6. Préparation du matériel réel : essais physiques à faire par l'utilisateur, ISO dédiée à ses diagnostics.

## Preuves reproductibles actuelles
`bash tests/regress.sh target2/release` (hôte + émulation) ; `bash tests/nav.sh target2/release` (navigation et secours) ; `tests/eval-phon.sh <lexique>` (phonétisation). Aucune ne remplace un essai physique.
