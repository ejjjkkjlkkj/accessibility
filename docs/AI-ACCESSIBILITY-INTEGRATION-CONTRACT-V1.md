# Contrat d'intégration IA ↔ ACCESSIBILITY — v1

**Statut : contrat de coordination, pas preuve d'intégration exécutée.**

Référentiels :
- Modèle IA, tokenizer, format des poids et pipeline d'entraînement : [ejjjkkjlkkj/ia](https://github.com/ejjjkkjlkkj/ia).
- Screen reader, UEFI, adaptateurs OS et journal de preuves AERL : [ejjjkkjlkkj/accessibility](https://github.com/ejjjkkjlkkj/accessibility).

## Règles non négociables

1. Le démarrage UEFI, le focus clavier et la navigation accessible de base restent utilisables sans modèle IA, connexion réseau ou clé API.
2. L'inférence est locale et optionnelle. Aucun contenu utilisateur, secret, texte d'écran ou prompt complet ne doit être écrit dans AERL par défaut.
3. Une intégration doit déclarer explicitement le backend, la version du contrat, les dimensions/tokenizer du modèle, les limites mémoire/temps et le comportement en cas d'échec.
4. Les poids ne sont compatibles que si le hash du manifeste, la version du tokenizer, les formes des tenseurs et les vecteurs de test concordent. Un format commun annoncé ne suffit pas.
5. Le dépôt IA ne déclare pas un modèle entraîné sans artefact vérifiable, configuration, provenance des données, hash, commande reproductible et évaluation tenue à l'écart.
6. Le dépôt ACCESSIBILITY ne déclare pas la voix, l'adaptateur OS ou le fonctionnement matériel comme validés sur la base d'un test simulé.
7. Les deux dépôts gardent des CI séparées et des commits/rapports identifiables. Un succès dans l'un ne valide pas l'autre.

## Événement de preuve commun

Les rapports d'intégration peuvent exposer les champs non sensibles suivants :
- `contract_id`: `ai-accessibility-v1`
- `producer_repo`: nom du dépôt producteur
- `producer_commit`: SHA Git complet
- `component`: identifiant stable du composant
- `status`: `PASS`, `FAIL`, `PARTIAL`, `BLOCKED` ou `NOT_VALIDATED`
- `test_id`: identifiant stable du test
- `artifact_sha256`: SHA-256 d'un artefact de test, si disponible
- `limitations`: liste concise des limites connues

Ne jamais inclure de secret, de données personnelles, de contenu brut d'écran ou de prompt privé. Les rapports doivent rester falsifiables et traçables; un hash seul ne garantit ni la provenance ni l'absence de falsification.

## Porte de validation

Une intégration est `PASS` uniquement après :
1. test de conformité du contrat ;
2. test de chargement d'un artefact dont les empreintes sont vérifiées ;
3. vecteurs d'entrée/sortie reproductibles ;
4. test du mode dégradé sans IA ;
5. contrôle des fuites de données ;
6. rapport CI lié aux commits des deux dépôts.

À défaut, utiliser `PARTIAL` ou `NOT_VALIDATED`. Le présent document ne prétend pas que ces étapes sont déjà satisfaites.
