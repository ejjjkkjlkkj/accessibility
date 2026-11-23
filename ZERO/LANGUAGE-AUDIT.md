# Audit des langages existants pour concevoir ZERO

**Statut global : PARTIAL — plan et hypothèses de départ seulement.** Aucun audit exhaustif de tous les langages existants n'est terminé. Les lignes ci-dessous sont des axes de recherche, pas des verdicts définitifs.

## Méthode obligatoire
Pour chaque langage/version, collecter :
- spécification normative et version du compilateur ;
- comportement mémoire, arithmétique et concurrence ;
- modèle d'erreurs, exceptions/panics et récupération ;
- allocations, runtime/GC, dépendances système et possibilités freestanding ;
- compilation croisée, architectures et ABI ;
- métaprogrammation, génériques, modularité et temps de compilation ;
- bibliothèques, gestion des paquets, chaîne de compilation et reproductibilité ;
- tests de conformité, analyse statique, fuzzing et preuves de sûreté disponibles ;
- accessibilité des outils et qualité des diagnostics ;
- benchmarks mesurés avec matériel, versions, commandes et données publiés.

Classer chaque observation :
- FACT : confirmé par une source primaire/versionnée ;
- TRADEOFF : compromis intentionnel, pas nécessairement défaut ;
- ECOSYSTEM : limitation d'outils ou bibliothèques, pas du langage lui-même ;
- HYPOTHESIS : à tester ;
- NOT_CHECKED : pas encore audité.

Une affirmation « ZERO est meilleur » exige un test comparatif reproductible. Ne pas comparer des benchmarks non équivalents.

## Première cartographie par familles

### Systèmes et bas niveau
- C — à examiner : comportement indéfini ou dépendant de l'implémentation dans certains cas, gestion manuelle de la mémoire, vérification limitée par le langage. Atouts à préserver : ABI, portabilité, contrôle bas niveau et écosystème.
- C++ — à examiner : complexité de la norme, interactions entre fonctionnalités, durée de vie et comportement indéfini. Atouts : abstractions, templates, performance et vaste écosystème.
- Rust — à examiner : coût d'apprentissage du système d'emprunts, contraintes d'interopérabilité et blocs unsafe ; la sûreté du code sûr ne prouve pas celle des interfaces et composants unsafe.
- Zig — à examiner : maturité/version de l'outil, stabilité des interfaces, couverture des cibles et bibliothèques. Ne pas traiter une fonctionnalité manquante dans une version comme une limite définitive.
- Ada/SPARK — à examiner : disponibilité des outils et bibliothèques selon les cibles, coût d'apprentissage et intégration aux écosystèmes existants. Atouts : typage et vérification pour systèmes critiques.
- D, Nim, Odin, V, Crystal, Modula-2, Pascal, Fortran, COBOL, PL/I, assembleurs x86-64/AArch64/RISC-V — auditer séparément sémantique mémoire, runtime, compilation croisée, maturité, ABI, outils de vérification et usage réel freestanding. Ne pas supposer qu'ils partagent les mêmes limites.

### Langages applicatifs et runtimes
- Go — vérifier GC, coût et déterminisme des pauses selon le cas d'usage, taille du runtime, interopérabilité bas niveau et limites du profil freestanding.
- Java/Kotlin, C#/.NET, Swift, Objective-C — examiner dépendance aux runtimes ou plateformes selon la cible, modèle de mémoire, ABI, compilation native, démarrage, empreinte et interopérabilité. Ces contraintes varient par implémentation.
- Python, JavaScript, TypeScript, Ruby, Lua, R, Julia, APL, J, K, q — examiner runtime, gestion dynamique des types, déterminisme des performances, dépendances de bibliothèques et disponibilité d'un chemin natif/freestanding. Les résultats doivent être spécifiques à chaque implémentation.
- Scratch — traiter comme environnement visuel/pédagogique ; auditer son modèle d'exécution et ses objectifs sans lui appliquer arbitrairement les critères d'un langage noyau.
- Julia, Mojo, Chapel — examiner maturité, compatibilité, modèle parallèle, compilation, déploiement et dépendances de runtime avec versions exactes.
- U et autres langages spécialisés ou expérimentaux — confirmer identité, spécification, état de l'implémentation et licence avant de tirer des conclusions.

Cette liste reprend les pistes actuellement suivies dans les projets, mais ne constitue pas une liste exhaustive de tous les langages du monde.

## Axes transversaux à comparer
1. Sécurité mémoire et sûreté des conversions.
2. Sémantique définie et déterminisme.
3. Temps réel, latence et consommation mémoire.
4. Concurrence et parallélisme.
5. Contrôle du runtime et compilation freestanding.
6. Portabilité CPU/UEFI/OS et ABI.
7. Génériques, macros, métaprogrammation et complexité.
8. Diagnostics, débogage, tests, fuzzing et analyse formelle.
9. Bibliothèques, paquets, licences et chaîne d'approvisionnement.
10. Accessibilité des outils de développement.
11. IA : opérations numériques, SIMD/GPU, gestion mémoire et reproductibilité.
12. Interopérabilité et migration du code existant.

## Exigences qui pourraient en découler pour ZERO
À valider, pas à déclarer acquises :
- architecture du jeu d'instructions (ZERO-ISA 64-bit Tier-0) indépendante de tout langage ou OS hôte ;
- arithmétique définie et conversions explicites ;
- valeurs initialisées garanties dans le sous-ensemble sûr ;
- vérification statique de durée de vie et d'accès mémoire ;
- limites explicites pour unsafe/raw et ABI ;
- allocation et effets de runtime déclarés dans les profils ;
- concurrence avec règles de sûreté spécifiées ;
- sous-ensemble freestanding sans GC ni allocation implicite ;
- diagnostics structurés accessibles ;
- compilation croisée et tests différentiels automatisés ;
- spécification versionnée et suite publique de conformité.

## Sources primaires de départ
- Rust Reference, comportements indéfinis : https://doc.rust-lang.org/reference/behavior-considered-undefined.html
- Rust Reference, opérations unsafe : https://doc.rust-lang.org/reference/unsafety.html
- Go FAQ, motivations et compromis de conception : https://go.dev/doc/faq
- LLVM, infrastructure de compilation : https://llvm.org/
- WebAssembly, objectifs de conception : https://webassembly.org/docs/high-level-goals/

Chaque entrée devra ensuite être enrichie de sources primaires propres à la version examinée et de tests reproductibles.
