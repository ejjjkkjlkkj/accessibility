# VOXA : cahier des charges (2026-10-11)
Statuts : PASS / FAIL / PARTIAL / BLOCKED / NOT_VALIDATED. Rien ci-dessous n'est validé sur matériel réel.

## 1. Principes tirés de l'étude (CARTE-SYSTEMES-ET-LECONS.md, CARTE-SYSTEMES-COMPLEMENTS.md)
| # | Exigence | Origine de la leçon |
|---|---|---|
| P0 | **Le lecteur d'écran est la couche la plus basse** : la voix est initialisée AVANT la mémoire, les pilotes et le reste ; toute panne (démarrage, panique du noyau) est dite avant d'arrêter ; chaque couche supérieure parle via `Voice` | demande de l'utilisateur ; `voxa-kernel::Voice` |
| P1 | La parole démarre sans aucune touche, dès l'écran du menu | téléviseurs (Tizen/webOS), Slint |
| P2 | Arbre sémantique (rôles, états, actions, plages) fourni par le système, pas déduit de pixels | Fuchsia, WebKit AX, TalkBack |
| P3 | File d'annonces à priorités, interruption qui garde les messages importants | Fuchsia (speaker queue) |
| P4 | Chaque application peut parler elle-même, et aussi être lue | Emacspeak |
| P5 | Secours sans voix : bips de rang, texte à l'écran, code d'erreur | Debian (bip GRUB), nos tests |
| P6 | Lecteur = service isolé à privilèges minimaux | résumé GI 2025 |
| P7 | Installation entièrement parlée, sans réglage | Slint ; point faible d'Ubuntu/Fedora |
| P8 | Braille HID (page 0x41) en plus de la parole | Darwin, TalkBack |
| P9 | Touches rémanentes/lentes au niveau du clavier | Darwin IOHIDKeyboardFilter |
| P10 | Tout comportement est vérifiable par test reproductible avec journal | règles de travail |
Licences : code copyleft/APSL étudié, jamais copié ; MIT/Apache/BSD réutilisable avec attribution.

## 2. Exigences par couche (état actuel)
| Couche | Exigence | État |
|---|---|---|
| Démarrage UEFI | menu parlé, confirmations, secours bips | PASS émulation / NOT_VALIDATED matériel |
| Cœur d'accessibilité | arbre, focus, annonces, revue, saisie, recherche, braille (table à faire valider) | PASS tests hôte |
| Audio | pilote HDA Rust, parole française formantique | PASS modèle émulé / NOT_VALIDATED audible |
| Noyau | mémoire, interruptions, tâches, ExitBootServices | non commencé |
| Pilotes | clavier 8042, USB/HID, NVMe, FAT, GOP, réseau | non commencé |
| Applications | éditeur, fichiers, navigateur, courrier | non commencé |
| VOXA AI | moteur d'inférence propre + modèle propre | embryonnaire (phonétisation) |

## 3. Banc d'essai : « notre hyperviseur »
Deux niveaux, à ne pas confondre :
1. **Existant (PASS)** : notre émulateur x86-64 + firmware UEFI simulé + contrôleur HDA modélisé. Il exécute le code rustc, injecte touches, mesure l'audibilité du modèle. Limite : ne reproduit pas un vrai firmware ni un vrai codec.
2. **À construire** : hyperviseur matériel AMD-V (SVM) sur le Ryzen 7 5800H : démarre sur clé, lance l'invité Voxa en virtualisation réelle, journalise sorties/entrées, avec disque virtuel en mémoire. Étapes : (a) détection SVM via CPUID, (b) VMCB + NPT, (c) interception I/O ports, (d) invité minimal, (e) journal série. Risque : un hyperviseur sur matériel peut figer la machine ; il ne s'essaie que depuis la clé, après la validation de l'étape précédente, sans toucher au disque interne. Statut : NOT_STARTED.

## 4. Chemin vers l'ISO finale (portes d'entrée, chacune doit être PASS)
G1 régression émulée verte (fait, 50 PASS) → G2 essai physique de Voxa Live depuis clé par l'utilisateur (PLAN-VALIDATION-ISO.md) → G3 noyau minimal + pilote clavier natif → G4 stockage + FAT → G5 hyperviseur de test → G6 essais à 3 machines et 5 utilisateurs aveugles → G7 ISO finale publiée.
**Ce que je ne fais pas** : écrire sur une clé, toucher au disque interne ou au démarrage de Windows, redémarrer le PC. Écrire l'ISO sur une clé vide est votre action (outil de gravure de votre choix, lettre de lecteur vérifiée) ; la clé sera effacée. Sauvegardez d'abord la clé de récupération BitLocker.
