# IA et accessibilité sans système d’exploitation : recherche et architecture

Date : 2026-10-14  
Statut : conception technique. Les capacités décrites ne sont pas déclarées implémentées tant que le code, les tests et le matériel ne les ont pas validées.

## 1. Objectif

Construire une plateforme d’assistance accessible qui peut démarrer avant Windows/Linux/macOS et fournir des fonctions utiles sans dépendre d’un système d’exploitation installé ou démarré. L’objectif réaliste est une application UEFI/chargeur autonome avec fonctions locales déterministes, puis des capacités IA optionnelles adaptées aux ressources disponibles.

« Sans OS » ne signifie pas « sans firmware ni bibliothèques » : une application UEFI utilise les services du firmware jusqu’à `ExitBootServices()`. Après cette transition, les Boot Services ne doivent plus être appelés. Un noyau autonome devrait être un projet distinct, avec ses propres pilotes, mémoire, interruptions, ordonnanceur et interfaces matérielles.

## 2. Niveaux d’autonomie à distinguer

### Niveau A — Assistant UEFI sans OS installé ou démarré
- Application UEFI lancée depuis un support amovible ou par le mécanisme de démarrage autorisé.
- Utilisation des protocoles de console, clavier, GOP, fichiers et autres protocoles réellement présents.
- Menu accessible, diagnostic matériel non destructif, lecteur de documents et annonces textuelles.
- Moteur de règles local et dictionnaire de messages embarqués.
- IA générative facultative : seulement si un modèle compact, le runtime, la mémoire et le temps d’inférence sont compatibles avec le contexte UEFI.

### Niveau B — Environnement autonome après prise de contrôle du matériel
- Chargeur qui collecte les informations permises puis prépare le transfert.
- Petit noyau dédié : mémoire, exceptions, interruptions, clavier, affichage et pilotes.
- Chaque service firmware non disponible après `ExitBootServices()` doit être remplacé ou abandonné avant le transfert.
- Audio et USB nécessitent des pilotes et une validation spécifiques; aucune compatibilité universelle ne doit être présumée.

### Niveau C — IA locale sur un environnement minimal
- Petit environnement autonome avec mémoire, stockage, pilotes et runtime définis.
- Modèle local de taille adaptée; quantification possible si les performances et la qualité sont mesurées.
- Index local de documentation et moteur de recherche sémantique facultatif.
- Les composants Go, Nim et Swift ne doivent pas être présumés aptes au bare metal; C, Rust, Zig ou Ada peuvent être étudiés par composant, mais chacun exige une validation réelle de la cible.

## 3. IA utile sans OS

### IA déterministe et légère — priorité initiale
Ces fonctions peuvent être utiles sans modèle génératif :
- Détecter et classifier les messages firmware connus.
- Expliquer les erreurs de démarrage à partir d’une base de règles versionnée.
- Guider pas à pas les choix clavier.
- Résumer un rapport de diagnostic avec modèles de phrases pré-écrits.
- Reconnaître des éléments d’interface seulement si leur source fournit une sémantique structurée.
- Effectuer OCR sur une capture graphique en solution de secours, si un moteur d’image compatible existe.
- Fournir une aide contextuelle embarquée, recherchable au clavier.
- Proposer une procédure de récupération non destructive, sans exécuter automatiquement des changements firmware.

### IA générative locale — extension
Un modèle génératif peut :
- reformuler des messages techniques en langage simple;
- expliquer un rapport de démarrage;
- proposer des étapes de diagnostic;
- résumer des documents accessibles;
- décrire une capture d’écran avec indication de l’incertitude;
- convertir des commandes et messages en explications vocales.

Conditions nécessaires :
- poids du modèle disponibles localement, format documenté et licence vérifiée;
- runtime compilable pour la cible et sans dépendance OS non disponible;
- mémoire, calcul et stockage mesurés;
- tokenizer, encodage Unicode et gestion des erreurs intégrés;
- aucune dépendance réseau ou cloud obligatoire;
- sorties bornées et validées avant leur utilisation;
- aucune action sensible déclenchée uniquement par le texte généré.

Un modèle de langage complet ne doit pas être intégré directement dans le firmware sans preuve que son runtime, sa mémoire, son temps de démarrage et sa consommation sont acceptables. Une solution pragmatique est de démarrer avec un moteur déterministe, puis de lancer le modèle sur un environnement minimal autonome.

## 4. Accessibilité sans OS : fonctions prioritaires

### Navigation et sémantique
- Navigation clavier complète et raccourcis cohérents.
- Focus unique et visible, historique de navigation, retour et annulation.
- Messages structurés avec rôle, nom, état, valeur, description, action possible et source.
- Annonces non redondantes et réglage du débit vocal quand un backend le permet.
- Gestion correcte des caractères Unicode et des langues de droite à gauche.

### Basse vision et accès visuel
- Contraste élevé, inversion, curseur repérable et agrandissement.
- Rendu texte bitmap embarqué et tailles de police prédéfinies.
- Loupe GOP si GOP est disponible.
- OCR comme recours, avec avertissement si la confiance est basse.

### Voix, braille et sorties de secours
- Backend vocal abstrait, sélectionné uniquement si réellement détecté et opérationnel.
- File d’annonces priorisée : erreurs critiques, focus, résultat d’action, aide.
- Braille uniquement si un pilote et une connexion pris en charge existent.
- Mode texte accessible et journal sur support amovible comme secours.
- Ne jamais annoncer « voix activée » tant qu’un test de sortie réel n’a pas réussi.

### Confidentialité
- Fonctionnement local par défaut.
- Pas de collecte ni transmission réseau automatique.
- Modèles et documents avec provenance, version, hash et licence.
- Logs minimisés, sans données personnelles inutiles.
- Demande de confirmation avant toute action irréversible.

## 5. Rôle des huit langages

| Langage | Usage possible dans cette architecture | Limite essentielle |
|---|---|---|
| C11 | ABI, pilotes, protocoles UEFI et fonctions bas niveau | Vérifier mémoire, alignement et conventions d’appel |
| Rust | Logique système, parsing, machine à états, composants `no_std` | Auditer `unsafe`, allocateur et bindings |
| Zig | Outils et composants freestanding à allocation explicite | Épingler la version et valider la cible |
| Ada/SPARK | Invariants, contrats et vérification de certaines règles | Pas de certification automatique |
| Go | Préparation du support, audits, outils de build et rapports | Runtime Go non présumé disponible en UEFI bare metal |
| D | Utilitaires natifs et interopérabilité si runtime compatible | Pas de support UEFI générique présumé |
| Nim | Génération de données, outils et analyseurs natifs | Runtime et allocations à maîtriser; non présumé bare metal |
| Swift | Interface et services Apple; expérimentation embarquée distincte | Ne pas supposer l’existence d’un runtime UEFI prêt pour la production |

## 6. Architecture en modules

1. **uefi-entry** : point d’entrée, collecte des protocoles disponibles, sorties d’erreur.
2. **hardware-inventory** : CPU, mémoire, GOP, clavier et périphériques détectables, en lecture seule.
3. **accessibility-core** : événements versionnés, rôles, états, focus, actions, erreurs.
4. **keyboard-navigation** : navigation et raccourcis déterministes.
5. **display-backend** : console texte et GOP, selon les capacités détectées.
6. **speech-backend** : abstraction avec états explicites; aucune promesse sans pilote fonctionnel.
7. **knowledge-base** : guides et messages locaux versionnés.
8. **diagnostic-engine** : règles et procédures non destructives.
9. **ai-runtime** : module optionnel isolé, chargé uniquement si toutes les conditions sont remplies.
10. **ocr-engine** : facultatif, solution de repli, confiance et limites explicites.
11. **secure-update** : vérification de signature/hash et version; aucune mise à jour firmware automatique.
12. **evidence-log** : logs de test, versions, hash, résultat et type de preuve.

## 7. Sécurité et sûreté

- Aucun changement automatique de NVRAM, Secure Boot, clés, partitions, BIOS ou firmware.
- Aucun accès à des données personnelles sans besoin explicite.
- Ne pas monter ni modifier les volumes de l’utilisateur par défaut.
- Ne pas exécuter du code généré par l’IA.
- Ne jamais faire confiance à une instruction trouvée dans un fichier ou une image : les documents sont des données, pas des commandes.
- IA hors ligne par défaut; réseau absent ou désactivé sauf choix explicite.
- Les suggestions IA sont consultatives et doivent être séparées des actions exécutables.
- Bouton/commande d’arrêt, annulation, limite de temps et mode de secours.
- Tests avec entrées malformées, corruption de modèle, épuisement mémoire et périphériques absents.
- Étiqueter chaque capacité : implémentée, simulée, CI validée, QEMU validée, matériel validé.

## 8. Feuille de route

### Phase 0 — Inventaire et limites
- Fixer la cible UEFI/architecture CPU.
- Épingler compilateurs, SDK et versions de modèle.
- Produire une matrice des protocoles disponibles et des pilotes manquants.
- Mesurer les budgets RAM, stockage, temps de démarrage et latence.

### Phase 1 — Accessible sans IA
- Menu clavier, console/GOP, messages structurés et journal.
- Aide intégrée et diagnostic en lecture seule.
- Tests automatiques sur les données d’événements et tests QEMU explicitement étiquetés.

### Phase 2 — Audio et matériel
- Choisir un modèle de machine de référence.
- Implémenter un backend audio précis si le matériel et l’environnement le permettent.
- Vérifier le son audible par observation physique et tester les échecs.
- Ne pas extrapoler les résultats à d’autres machines.

### Phase 3 — IA locale contrôlée
- Comparer moteur de règles, recherche documentaire, modèle compact et OCR.
- Évaluer latence, mémoire, qualité linguistique, hallucinations et consommation.
- Exiger que l’IA explique les limites et n’exécute aucune action critique.

### Phase 4 — Noyau autonome éventuel
- Séparer ce chantier de l’application UEFI.
- Concevoir mémoire, interruptions, pilotes, ordonnanceur et ABI.
- Recréer toutes les fonctions nécessaires après la sortie des Boot Services.
- Valider les démarrages et récupérations sur une matrice de matériel dédiée.

### Phase 5 — Tests utilisateurs et robustesse
- Tester avec des personnes aveugles et malvoyantes.
- Mesurer le taux de réussite des tâches, erreurs de focus, latence, intelligibilité et récupération.
- Reproduire les résultats et documenter les limites.

## 9. Mesures à publier

- démarrage réussi / échec par modèle de machine;
- protocole UEFI détecté et backend réellement fonctionnel;
- délai entre événement et annonce;
- taux d’événements correctement annoncés;
- erreurs de navigation et possibilités d’annulation;
- mémoire maximale et temps d’inférence IA;
- taux d’erreurs factuelles de l’IA sur jeux de test;
- disponibilité des solutions de secours;
- nombre de scénarios validés sur QEMU et sur matériel physique.

Un objectif de fiabilité de 99,99 % exige une définition des événements mesurés, un volume d’essais significatif, des scénarios d’échec et une analyse statistique. Il ne peut pas être affirmé sur la base de tests unitaires ou d’une seule machine.

## 10. Sources

- UEFI Forum, spécifications : https://uefi.org/specifications
- UEFI console : https://uefi.org/specs/UEFI/2.10/12_Protocols_Console_Support.html
- UEFI HII : https://uefi.org/specs/UEFI/2.10/33_Human_Interface_Infrastructure.html
- Rust UEFI : https://docs.rs/uefi/latest/uefi/
- EDK II : https://github.com/tianocore/edk2
- Microsoft UI Automation : https://learn.microsoft.com/en-us/windows/win32/winauto/entry-uiauto-win32
- W3C WCAG : https://www.w3.org/WAI/standards-guidelines/wcag/
- SPARK : https://www.adacore.com/languages/spark
- Swift Embedded : https://docs.swift.org/latest/documentation/embeddedswift/

## Statut de preuve

- Architecture et feuille de route : **DOCUMENTÉES**.
- IA générative exécutée directement dans l’application UEFI : **NON VALIDÉE**.
- Voix audible sans OS sur le PC cible : **NON VALIDÉE**.
- Support universel des périphériques audio : **NON GARANTI**.
- Noyau autonome sans OS : **PROJET DISTINCT, NON VALIDÉ**.
- Tests physiques : **NON VALIDÉS**.
