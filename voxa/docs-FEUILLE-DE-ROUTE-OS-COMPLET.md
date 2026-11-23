# Voxa OS : de ce qui existe aujourd'hui à un système complet (feuille de route honnête, 2026-10-11)
Décision de l'utilisateur : tout maison (noyau Rust propre). « Finir aujourd'hui » est réalisable pour **un jalon**, pas pour un système complet : un système complet (noyau, pilotes, fichiers, réseau, applications, mises à jour) représente des mois à des années de travail et des essais sur matériel.

## Jalon livrable aujourd'hui : « Voxa Live v6 »
Application UEFI parlante, sans installation, qui dit la machine : menu à 8 éléments, saisie/recherche de texte, infos SMBIOS, **mémoire utilisable (carte mémoire UEFI) et périphériques PCI par classe**, voix française (chaîne maison phonétisation → synthèse → HDA) avec secours par bips. ISO candidate `out/voxa-live-rs-v6-candidate.iso`, structure vérifiée par deux lecteurs indépendants. **Démarrage réel : non testé (UNPROVEN).** C'est un démonstrateur de démarrage, pas un système d'exploitation.

## Ce qui sépare ce jalon d'un système complet (dans l'ordre de dépendance)
| Étape | Contenu | État |
|---|---|---|
| 0 Preuve matérielle | essai de la clé par l'utilisateur sur la machine : écran, bip, voix, touches ; sans cela rien n'est prouvé | à faire par l'utilisateur |
| 1 Audio réel | pilote HDA face au codec Realtek ALC256 (nœuds, amplificateurs, casque) | modèle émulé seulement |
| 2 Noyau | ExitBootServices, mémoire virtuelle (tables de pages), interruptions (IDT, APIC), minuterie (HPET), tâches/ordonnanceur | allocateur de pages + carte mémoire seulement |
| 3 Entrées | clavier natif 8042, USB xHCI complet (anneaux, énumération, HID, braille), pavé tactile I2C-HID | décodeurs simulés |
| 4 Stockage et fichiers | NVMe, GPT, système de fichiers (FAT puis un système propre avec journal), cache | lecture FAT dans les outils seulement |
| 5 Réseau | Wi-Fi MT7921 (micrologiciel propriétaire) ou Ethernet/USB, TCP/IP, TLS | rien |
| 6 Graphique | pilote GPU AMD ou GOP agrandi, rendu texte et polices | GOP minimal |
| 7 Accessibilité de bout en bout | arbre sémantique pour toutes les applications, API d'accessibilité, braille, touches rémanentes | cœur testé, pas d'applications |
| 8 Applications | éditeur, explorateur de fichiers, navigateur (le plus lourd), courrier, bureautique | rien |
| 9 Système | installation, mises à jour sécurisées, récupération, sécurité (isolation, signature) | rien |
| 10 Hyperviseur de test | AMD-V (SVM, NPT) pour tester l'OS sur cette machine | faisable (SVM/NPT activés), rien d'écrit |
Les étapes 0 à 3 sont le chemin critique de l'accessibilité : sans elles, pas de voix ni d'entrée fiable sur le vrai matériel.

## Règle de sortie
Aucune étape n'est dite « faite » sans test automatisé reproductible, et aucune n'est dite « validée » sans essai sur le matériel réel par l'utilisateur.
