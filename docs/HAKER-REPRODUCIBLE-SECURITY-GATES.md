# HAKER — gates de sécurité reproductibles

Statut : **PLAN DE CONTRÔLE — NON EXÉCUTÉ PAR CE DOCUMENT**. Ce fichier ne constitue ni un audit réussi ni une preuve de sécurité.

## Périmètre

Tests limités au code, aux dépendances et aux artefacts de ce dépôt, dans des environnements contrôlés. Aucun scan de tiers, exploitation externe, collecte d'identifiants, action firmware/NVRAM, flash BIOS ou redémarrage.

## Portes obligatoires

1. **Baseline** — enregistrer commit, version OS/runtime, commande exacte, code retour et empreintes SHA-256 des rapports.
2. **Analyse statique** — Python : règles SEC007 (désérialisation), SEC008 (MD5/SHA-1 à examiner), SEC009 (tempfile.mktemp) ; Rust : examiner chaque bloc `unsafe`, les conversions d'entiers, tailles de buffers, pointeurs et accès MMIO.
3. **Tests adversariaux bornés** — entrées malformées, vides, surdimensionnées et limites numériques sur les parseurs du dépôt. Limiter temps, mémoire et taille d'entrée ; enregistrer toute entrée qui reproduit un défaut.
4. **Fuzzing** — harness par parseur avec graine et budget explicites ; conserver les cas minimaux de reproduction. Un crash du harness doit être distingué d'une vulnérabilité exploitable.
5. **Dépendances** — verrouiller les versions, vérifier licences et provenance, relever les avis connus et produire un inventaire SBOM lorsque l'outillage disponible le permet.
6. **Secrets** — scanner le diff et les journaux ; masquer les valeurs sensibles dans les rapports. Les tests doivent vérifier le masquage, pas imprimer les secrets.
7. **Non-régression** — toute anomalie confirmée possède un test qui échoue avant correction et passe après correction.
8. **Artefacts** — vérifier provenance, taille, format et SHA-256 des binaires produits ; ne pas assimiler compilation ou QEMU à une validation physique.
9. **Revue indépendante** — relecture des changements de sécurité et vérification des faux positifs avant de modifier le statut d'une alerte.

## Format minimal d'une preuve

- ID de contrôle et commit testé ;
- environnement et versions ;
- commande exacte et code retour ;
- résultat brut expurgé de tout secret ;
- empreinte SHA-256 du rapport ;
- statut : PASS, FAIL, PARTIAL, BLOCKED ou NOT_VALIDATED ;
- limites, faux positifs connus et étape suivante.

## Règles de statut

- **PASS** : commande reproductible exécutée, résultat attendu et preuve conservée.
- **FAIL** : échec observé et reproductible.
- **PARTIAL** : une partie seulement de la couverture est exécutée.
- **BLOCKED** : prérequis indisponible, identifié explicitement.
- **NOT_VALIDATED** : aucune preuve suffisante disponible.

Une détection heuristique reste une alerte à examiner ; elle ne devient une vulnérabilité confirmée qu'après triage et reproduction. Les portes doivent être ajoutées à la CI après vérification des commandes et dépendances réellement présentes dans le dépôt.
