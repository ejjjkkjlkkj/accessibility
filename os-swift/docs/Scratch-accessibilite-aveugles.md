# Scratch et l'accessibilité pour les personnes aveugles

Date de la recherche : 2026-11-20. Sources lues en résumé ou en extrait ; le texte complet n'a pas toujours été lu. Les points non vérifiés sont marqués.

## Constat

- Scratch reste décrit comme peu accessible aux personnes aveugles. Les espaces de travail et les boîtes à outils des environnements en blocs sont en grande partie inutilisables avec un lecteur d'écran.
- Les travaux récents portent sur la navigation au clavier et la saisie vocale.

## Travaux récents

- **Revue systématique (2026)** de 22 études évaluées par des pairs sur l'accessibilité des environnements en blocs (Blockly, Scratch, App Inventor), publiée dans *Universal Access in the Information Society*. Solutions recensées : navigation au clavier, interaction vocale, intégration aux lecteurs d'écran, modèles de navigation dans le programme.
  Source : https://link.springer.com/article/10.1007/s10209-026-01351-6
- **Préprint arXiv (2026)** : interface vocale pour Scratch, destinée aux enfants ayant un handicap moteur. Les auteurs revendiquent 15 critères sur 28 remplis en totalité. Compatibilité avec les lecteurs d'écran : non testée sérieusement, selon les auteurs.
  Source : https://arxiv.org/html/2603.28246v1

## Travaux antérieurs

- **Neuf participants malvoyants** : les programmeurs aveugles utilisent des stratégies proches de celles des voyants pour comprendre le code, malgré les contraintes du lecteur d'écran.
  Source : https://dl.acm.org/doi/10.1145/3507469
- **Blocks4All** : comparaison de neuf environnements en blocs ; l'espace de travail et la boîte à outils de Scratch sont en grande partie inutilisables avec un lecteur d'écran. Proposition : un robot parlant comme sortie accessible.
  Source : https://par.nsf.gov/servlets/purl/10063855
- **Interface en lignes (WCSE 2024)** : navigation dans les blocs avec les flèches et retours du lecteur d'écran, pour réduire la charge de mémorisation de la structure.
  Source : https://www.wcse.org/WCSE_2024/010.pdf
- **Édition au clavier et lecteur d'écran** : les personnes aveugles peuvent créer et modifier des programmes en blocs de façon efficace.
  Source : https://par.nsf.gov/servlets/purl/10302001

## Déclarations non vérifiées

- Une annonce de l'équipe Scratch évoque la navigation au clavier dans l'éditeur et une meilleure compatibilité avec les lecteurs d'écran. Non vérifiée sur le site officiel.

## Lacunes

- Pas d'étude de 2025-2026 où des participants aveugles utilisent Scratch lui-même.
- Le préprint de 2026 ne teste pas les lecteurs d'écran.

## Ce que ça change pour notre projet

- Les pistes validées par les études sont la navigation au clavier, la sortie vocale et une structure de blocs lisible par un lecteur d'écran. Notre noyau d'accessibilité suit déjà ces principes en dehors de Scratch.
- Une interface en lignes, plutôt qu'en blocs graphiques, est la piste la plus solide dans la littérature pour un utilisateur aveugle.
