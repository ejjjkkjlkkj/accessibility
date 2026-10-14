# Recherche : potentiel des langages pour l’accessibilité et UEFI

Date de recherche : 2026-10-14  
Statut : étude technique et feuille de route, pas une déclaration de conformité ni une preuve matérielle.

## Résumé exécutif

Les huit langages du dépôt peuvent contribuer à une plateforme d’accessibilité, mais ils ne sont pas interchangeables. La stratégie recommandée est de définir un contrat d’événement stable, de conserver une ABI C minimale, de mettre les composants pré-OS sensibles dans une base Rust `no_std` (et éventuellement C), puis d’utiliser chaque autre langage là où son écosystème apporte une valeur réelle.

**Important :** UEFI fournit des protocoles d’entrée/sortie, de texte et de graphisme, mais il ne garantit pas qu’un PC possède une synthèse vocale, un pilote audio pré-OS, un lecteur d’écran ou une API sémantique d’interface universelle. Le projet devra donc distinguer les fonctions natives disponibles, les fonctions optionnelles et les solutions de repli. Une voix intégrée et un son audible avant le démarrage de l’OS exigent une chaîne audio réellement prise en charge et testée sur le matériel ciblé.

## Capacités à viser

### A. Accessibilité UEFI / pré-démarrage

1. **Console texte accessible**
   - Lire les caractères reçus via `EFI_SIMPLE_TEXT_INPUT_PROTOCOL` et `EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL`.
   - Intercepter les événements clavier disponibles et fournir des raccourcis cohérents.
   - Annoncer les changements de ligne, erreurs, choix sélectionnés et demandes de confirmation.
   - Garder une navigation clavier complète, avec focus explicite et retour à l’écran précédent.
   - Limite : le texte de la console ne garantit pas l’accès aux contrôles graphiques propriétaires du firmware.

2. **Rendu graphique et loupe**
   - Utiliser `EFI_GRAPHICS_OUTPUT_PROTOCOL` pour lire les modes et dessiner un affichage de repli.
   - Ajouter contraste élevé, inversion, grande police bitmap, curseur visible et zones de focus.
   - Utiliser l’OCR seulement en dernier recours, avec indication claire de son incertitude.
   - Limite : GOP fournit des pixels, pas l’arbre sémantique des menus BIOS/UEFI.

3. **Interface HII / configuration**
   - Étudier les protocoles HII pour les chaînes, polices, formulaires et abstractions de saisie de configuration.
   - Construire un lecteur de formulaires HII si l’environnement et les licences permettent cette implémentation.
   - Exposer le nom du champ, la valeur, les contraintes, les options et les erreurs de validation.
   - Limite : toutes les interfaces de firmware ne sont pas nécessairement exposées comme formulaires HII standards.

4. **Synthèse vocale et audio**
   - Définir une interface `SpeechBackend` avec les états `available`, `unavailable`, `error`, `speaking`.
   - Priorité : sortie audio effectivement supportée par le firmware ou par un pilote pré-OS spécifique à la machine.
   - Prévoir une sortie de secours : console texte, série/console distante si configurée, codes sonores uniquement si le matériel les prend en charge.
   - Ne pas présenter un buffer PCM, une écriture de registres ou un test QEMU comme preuve d’un son physique.
   - Les contrôleurs HDA/USB audio nécessitent une découverte de périphérique, une configuration et des tests matériels dédiés; le support générique n’est pas garanti.

5. **Clavier, pointage et alternatives**
   - Utiliser les protocoles d’entrée standard disponibles et détecter leurs capacités.
   - Fournir navigation sans souris, répétition contrôlée, raccourcis découvrables et confirmation des actions destructrices.
   - Ne pas supposer qu’un écran tactile, une souris ou un périphérique braille est disponible avant l’OS.

6. **Transfert vers le système d’exploitation**
   - Fournir un démarrage accessible sans modifier la NVRAM, Secure Boot ou les paramètres firmware de l’utilisateur.
   - Transmettre un rapport de capacités et de limitations au chargeur ou à l’OS si un canal défini existe.
   - Tester les limites de durée de vie des services UEFI : les Boot Services cessent après `ExitBootServices()`; ne pas les appeler ensuite.
   - Maintenir une voie de retour sûre et un mode diagnostic en lecture seule.

### B. Accessibilité du système d’exploitation et des applications

- **Windows :** client UI Automation, prise en charge MSAA/IAccessible pour compatibilité, rôles/états/valeurs/relations/focus/événements, tests clavier et automatisation. Utiliser les API officielles plutôt que l’OCR en première intention.
- **Linux :** intégration AT-SPI, tests avec Orca, navigation clavier, annonces d’événements, applications GTK/Qt et terminal.
- **Apple :** intégration aux API d’accessibilité natives et tests VoiceOver; Swift est particulièrement adapté à cette couche.
- **Web :** HTML sémantique d’abord, WAI-ARIA seulement quand nécessaire, navigation clavier et validation WCAG 2.2.
- **Braille et basse vision :** modèle commun pour cellules braille, périphériques, zoom, contraste, espacement, réduction des animations et préférences utilisateur.
- **Outils de développement :** analyse statique, audits automatisés, rapports lisibles par lecteurs d’écran, tests de régression et reproduction des erreurs.
- **IA/OCR :** description d’images et compréhension de captures en mode complémentaire, avec provenance, confiance, confidentialité locale et absence d’action automatique dangereuse.

## Répartition recommandée par langage

| Langage | Où il peut apporter le plus | Usages réalistes | Réserves |
|---|---|---|---|
| **C11** | ABI, protocoles UEFI, pilotes et interopérabilité bas niveau | Structures stables, appels firmware, bindings aux SDK C, tests de layout ABI | Gestion manuelle de mémoire; vérifier les tailles, alignements, conventions d’appel et durées de vie |
| **Rust** | Cœur système et pré-OS | Modèle d’événements, parsing sans allocation, machine à états, composants `no_std`, application UEFI avec crate adaptée | Le code unsafe et les bindings firmware restent à auditer; tests unitaires ≠ test matériel |
| **Go** | Outils hôte et orchestration | Catalogue de ressources, audits, validation, rapports, CLI, serveurs de test | Runtime et collecte mémoire : pas le choix par défaut pour une application UEFI bare-metal |
| **D** | Outils natifs et composants système si le runtime est maîtrisé | Utilitaires rapides, interopérabilité C, analyseurs et outils de diagnostic | Le support freestanding dépend du compilateur, de la bibliothèque et de la cible; ne pas supposer une compatibilité UEFI directe |
| **Ada** | Logique contractuelle et composants critiques | Modèles d’état, invariants, contrôles de bornes, contrats et assertions | Étudier SPARK pour les preuves formelles; la présence d’Ada ou de tests ne constitue pas une certification |
| **Zig** | Composants systèmes explicites | Allocation contrôlée, bindings C, outils de bas niveau, composants freestanding après validation de cible | Version du compilateur et cible doivent être épinglées; tester ABI et bibliothèque disponible |
| **Nim** | Outils et logique de haut niveau compilés en natif | Générateurs de rapports, analyseurs, utilitaires d’audit, prototypes natifs | Le runtime et la gestion mémoire dépendent de la configuration; pas de présomption de compatibilité firmware |
| **Swift** | Applications, services et couche d’accessibilité Apple | API macOS/iOS, interfaces utilisateur, clients d’accessibilité et tests | Embedded Swift est un sous-ensemble expérimental selon la documentation consultée; ne pas le considérer comme chemin UEFI de production sans preuve |

## Architecture proposée

### Contrat d’événement partagé

Définir une version d’ABI indépendante du langage avec au minimum :

- `abi_version`, identifiant d’événement, horodatage monotone si disponible;
- source : firmware, clavier, UI Automation, AT-SPI, application ou synthèse vocale;
- rôle, nom accessible, valeur, description, état activé/désactivé/sélectionné/indisponible;
- relations parent/enfant, ordre de navigation et indication du focus;
- actions possibles, résultat et erreur structurée;
- langue, direction du texte et provenance;
- niveau de confiance pour toute donnée inférée par OCR/IA;
- taille maximale, limites explicites, traitement des chaînes Unicode et compatibilité de version.

Ne pas utiliser une seule structure fixe de nom court comme modèle universel pour des interfaces modernes : prévoir des chaînes UTF-8 bornées ou des références à des blocs de données avec durée de vie définie. Toute représentation C partagée doit spécifier alignement, taille, encodage, ownership et règles de compatibilité.

### Couches

1. **Platform adapters** : UEFI, Windows UI Automation/MSAA, Linux AT-SPI, Apple Accessibility, web.
2. **Normalized event model** : événements, rôles, états, focus, actions, erreurs et provenance.
3. **Navigation engine** : parcours par rôle, landmarks, éléments interactifs, formulaire, dialogue et historique.
4. **Output backends** : synthèse vocale, braille, affichage visuel, journal texte et sorties de secours.
5. **Input backends** : clavier, braille, commutateurs et dispositifs pris en charge.
6. **Accessibility test engine** : tests clavier, éléments sans nom, focus perdu, annonces manquantes, états incohérents, contraste et régression.
7. **Local intelligence** : OCR/vision/LLM local facultatif; aucune dépendance cloud obligatoire et aucune action destructive autonome.

## Feuille de route priorisée

### P0 — rendre le socle testable
- Épingler les versions des compilateurs et toolchains.
- Corriger et exécuter tous les tests CI des huit langages.
- Définir un schéma d’événement versionné unique, avec tests de compatibilité croisée C/Rust/Go.
- Ajouter des fixtures d’événements : bouton, champ de texte, menu, dialogue, erreur, focus et élément indisponible.
- Ajouter fuzzing des parseurs, tests de bornes, Unicode, tailles nulles/excessives et entrées malformées.
- Étiqueter chaque fonctionnalité : implémentée, simulée, testée en CI, testée sous QEMU, validée sur matériel.

### P1 — prototype UEFI non destructif
- Construire une application UEFI de diagnostic qui lit les protocoles disponibles et les affiche vocalement seulement si un backend audio réellement supporté existe.
- Implémenter navigation clavier, annonces de focus et de résultats, affichage à contraste élevé et journalisation série optionnelle.
- Examiner GOP, Console Input Ex, HII, Device Path, Loaded Image et fichiers UEFI, sans modifier NVRAM, Secure Boot, partitions ou paramètres firmware.
- Ajouter des tests avec firmware virtuel/QEMU, explicitement marqués comme simulation.

### P2 — preuve matérielle
- Définir une matrice par machine : firmware, mode UEFI, GOP, clavier, ports, contrôleur audio, pilote disponible, sortie physique.
- Démarrer depuis un support de test dédié après autorisation explicite; ne jamais écrire dans le firmware ou la NVRAM par défaut.
- Vérifier son audible avec observation externe, la récupération clavier, les erreurs, le retour au menu et le démarrage normal après test.
- Conserver logs, versions, hash des binaires, conditions de test et résultats PASS/FAIL/PARTIAL/BLOCKED.
- Tester les échecs et la récupération avant d’envisager un usage quotidien.

### P3 — lecteurs d’écran du système
- Implémenter un premier adaptateur Windows UI Automation avec C++/C ABI ou Rust bindings soigneusement audités.
- Ajouter un adaptateur Linux AT-SPI et valider avec Orca.
- Ajouter une couche Apple native en Swift et tester avec VoiceOver.
- Tester avec NVDA, JAWS, Narrator et Orca sur une matrice d’applications réelle.

### P4 — qualité et conformité
- Établir une matrice WCAG 2.2 pour le Web, EN 301 549 pour les produits concernés et critères d’accessibilité applicables à chaque plateforme.
- Mesurer la complétude des rôles/états, la latence des annonces, la perte de focus, la réussite clavier et les faux positifs d’audit.
- Inclure des tests utilisateurs rémunérés ou encadrés avec personnes aveugles et malvoyantes.
- Publier les limites connues et les résultats reproductibles; ne jamais déclarer « conforme » sur la base d’un seul outil automatique.

## Risques et points de contrôle

- **Audio pré-OS :** ne pas promettre une synthèse vocale universelle sans pilote et sortie réellement testés.
- **UEFI :** une console texte n’est pas un arbre d’accessibilité de tous les menus de configuration.
- **ABI :** les types, tailles et encodages doivent être identiques à travers les langages; les chaînes de chaque langage ne sont pas interchangeables en mémoire.
- **Sécurité :** pas de modification automatique de NVRAM, Secure Boot, clés, partitions ou firmware; pas de désactivation de protections.
- **IA :** les descriptions générées peuvent être erronées; afficher l’incertitude et ne pas exécuter d’actions sensibles sur la seule base d’une inférence.
- **Validation :** tests CI et QEMU démontrent certains comportements logiciels, pas l’audibilité, l’accessibilité réelle, la compatibilité de chaque firmware ou une fiabilité de 99,99 %.

## Sources techniques

- UEFI Forum, spécifications (dont UEFI 2.11, ACPI et documents associés) : https://uefi.org/specifications
- UEFI 2.10, protocoles de console et GOP : https://uefi.org/specs/UEFI/2.10/12_Protocols_Console_Support.html
- UEFI 2.10, Human Interface Infrastructure : https://uefi.org/specs/UEFI/2.10/33_Human_Interface_Infrastructure.html
- Rust UEFI crate et exemple `no_std` : https://docs.rs/uefi/latest/uefi/
- Microsoft UI Automation : https://learn.microsoft.com/en-us/windows/win32/winauto/entry-uiauto-win32
- W3C WCAG : https://www.w3.org/WAI/standards-guidelines/wcag/
- W3C WAI-ARIA : https://www.w3.org/WAI/standards-guidelines/aria/
- D interopérabilité C : https://dlang.org/spec/interfaceToC.html
- Ada/SPARK et vérification formelle : https://www.adacore.com/languages/spark
- Swift Embedded documentation : https://docs.swift.org/latest/documentation/embeddedswift/
- Swift Embedded overview : https://www.swift.org/get-started/embedded/

## État de preuve

- Recherche et architecture proposées : **DOCUMENTÉES**.
- Compatibilité UEFI de D, Ada, Zig, Nim et Swift pour la cible précise : **NON VALIDÉE**.
- Audio vocal audible avant OS sur une machine donnée : **NON VALIDÉ**.
- Conformité WCAG/EN 301 549 : **NON VALIDÉE**.
- Validation matérielle : **NON VALIDÉE**.
