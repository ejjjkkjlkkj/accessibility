# AERL — Accessibility Evidence & Reliability Log

**État : prototype local; tests en CI, aucune preuve matérielle. Ce n'est pas un remplacement général de syslog.**

AERL ajoute au dépôt ACCESSIBILITY un format de journal d'audit structuré et vérifiable, sans dépendance Python tierce, sans réseau et sans service cloud obligatoire. Il vise à conserver des preuves de tests, de validations HAKER et d'événements d'accessibilité avec une séquence et une chaîne de hashes SHA-256.

## Pourquoi ce n'est pas « mieux que syslog » dans tous les cas

Syslog est un standard éprouvé pour transporter et centraliser des événements système. AERL cible un problème plus restreint : l'intégrité vérifiable d'un journal de preuves local. Il ne remplace pas les journaux du noyau, les collecteurs distants, la rotation, les alertes ni les fonctions de transport de syslog ou systemd-journald.

## Format et garanties

Chaque ligne JSON UTF-8 contient exactement : version du schéma, séquence, horodatage UTC, acteur, type d'événement, statut, données structurées, hash précédent et hash courant. Les propriétés et nombres JSON ambigus sont refusés. Le hash est SHA-256 du JSON canonique de tous les champs sauf le hash courant.

Le lecteur vérifie :
- que les lignes sont terminées, bornées et au format attendu ;
- que la séquence commence à 1 et reste continue ;
- que le hash précédent correspond à l'enregistrement précédent ;
- que le hash courant correspond au contenu de l'enregistrement ;
- que les chemins sont des fichiers réguliers, pas des liens symboliques ou des fichiers spéciaux ;
- que les écritures passent par un verrou de fichier et un flush durable (fsync).

Les valeurs de statut possibles sont INFO, PASS, FAIL, PARTIAL, BLOCKED, NOT_VALIDATED et UNPROVEN.

## Profil bas niveau Rust no_std

Le noyau Rust binaire AERL-BIN/1 se trouve dans crates/aerl-core. Il utilise un tampon fourni par l'appelant, une trame binaire à longueur déterminée, une séquence, un hash précédent et SHA-256 via la bibliothèque RustCrypto sha2. Il n'a pas de système de fichiers, de réseau, d'allocation dynamique ni d'accès à l'horloge. Le timestamp est fourni par l'adaptateur; 0 signifie inconnu/non fiable.

Le profil CLI Python est AERL-JSONL/1. Les profils JSONL et BIN ne sont pas interchangeables; aucun convertisseur entre les deux n'est revendiqué. Le noyau binaire accepte un payload opaque borné: c'est l'adaptateur qui doit sérialiser les données et ne jamais transmettre de secrets, de texte utilisateur ni de données personnelles inutiles.

Validation du noyau :

    cargo test --manifest-path crates/aerl-core/Cargo.toml

Contrôle de compilation croisée, sans démarrage firmware :

    rustup target add x86_64-unknown-uefi
    cargo check --target x86_64-unknown-uefi --manifest-path crates/aerl-core/Cargo.toml

Ce contrôle ne valide pas le fonctionnement sur un PC physique. Le stockage, la sérialisation du payload, l'accès concurrent au tampon et l'ancrage externe du checkpoint restent du ressort des adaptateurs.

## Console unifiée pour les journaux du dépôt

`scripts/aerl_tool.py` devient la console unifiée pour initialiser, alimenter, collecter, vérifier, résumer, rechercher et créer/vérifier un checkpoint. La collecte automatique est volontairement bornée : elle ne parcourt que les rapports machine aux noms autorisés (par exemple `haker-audit-report.json`, `junit*.xml`, `test-results.json`). Elle n'enregistre ni le contenu brut ni les messages d'erreur des rapports : uniquement chemin relatif, SHA-256, taille et compteurs/status autorisés. Les sources inconnues sont ignorées; les doublons de même chemin + même hash sont ignorés. Les nouveaux contenus d'un même chemin sont collectés à nouveau. La collecte refuse les liens symboliques, fichiers spéciaux, rapports de plus de 2 Mio et XML invalides. Un refus rend la collecte PARTIAL et la commande échoue (code 1).

    python scripts/aerl_tool.py init --file .aerl/events.aerl
    python scripts/aerl_tool.py append --file .aerl/events.aerl --actor QA --kind accessibility.keyboard --status PASS --payload '{"checks":12}'
    python scripts/aerl_tool.py collect --root . --file .aerl/events.aerl
    python scripts/aerl_tool.py import-ledger --source matrix-linux/events.aerl --file .aerl/events.aerl --source-label linux-py313
    python scripts/aerl_run.py --file .aerl/events.aerl --kind test.unit -- python -m unittest discover -s tests -p "test_aerl_tool.py" -v
    python scripts/aerl_tool.py verify --file .aerl/events.aerl
    python scripts/aerl_tool.py summary --file .aerl/events.aerl --out summary.json
    python scripts/aerl_tool.py search --file .aerl/events.aerl --status FAIL --limit 100
    python scripts/aerl_tool.py search --file .aerl/events.aerl --commit-sha COMMIT_SHA --run-id RUN_ID --limit 100
    python scripts/aerl_tool.py checkpoint-create --file .aerl/events.aerl --out checkpoint.json
    python scripts/aerl_tool.py checkpoint-verify --file .aerl/events.aerl --checkpoint checkpoint.json
    python -m unittest discover -s tests -p "test_aerl_tool.py" -v

Le lanceur `scripts/aerl_run.py` exécute une commande sans masquer sa sortie, puis inscrit son code de retour, sa durée, le nombre d'arguments et l'empreinte SHA-256 de la ligne de commande. Il ne journalise jamais la ligne de commande elle-même, qui pourrait contenir des secrets. Si le journal ne peut pas être écrit, il renvoie 125 même si la commande cible a réussi : aucune réussite n'est déclarée sans enregistrement de preuve. La matrice HAKER utilise ce lanceur pour les tests et finalise le journal même lorsqu'une étape échoue. Le job d'agrégation vérifie chaque journal de matrice source, importe chaque événement avec son hash et sa provenance, déduplique les imports répétés, puis produit un journal maître `aerl-master-evidence`. Le hash source et le hash de chaque enregistrement sont conservés; le timestamp source n'est pas réutilisé comme timestamp maître. Le journal maître est un artefact de cette exécution CI, pas encore un stockage permanent inter-exécutions.\n\nLes rapports utilisent le schéma `AERL-REPORT/1`. Les sorties de rapport refusent l’écrasement de fichiers existants. La collecte est un point de départ borné, pas une capture exhaustive de chaque événement du dépôt; les producteurs de tests et d'agents doivent encore émettre des rapports structurés pour être collectés.

## Exemple Windows PowerShell

    New-Item -ItemType Directory -Force "$env:LOCALAPPDATA\Accessibility\logs" | Out-Null
    $ledger = "$env:LOCALAPPDATA\Accessibility\logs\events.aerl"
    python scripts/aerl.py append --file $ledger --actor QA --kind ci.haker.completed --status PASS --payload '{"run":"local-001","tests":12}'
    python scripts/aerl.py verify --file $ledger

La commande affiche la tête SHA-256. Pour créer un checkpoint JSON portable sans écraser un checkpoint existant :

    python scripts/aerl_checkpoint.py create --ledger $ledger --out checkpoint.json
    python scripts/aerl_checkpoint.py verify --ledger $ledger --checkpoint checkpoint.json

Le checkpoint contient le schéma, le numéro de séquence et le hash de tête. Conserver ce fichier dans un domaine de confiance distinct (par exemple un artefact de release signé, un stockage WORM ou un contrôleur de confiance indépendant). Si le checkpoint est stocké à côté du journal et modifiable par le même acteur, il n’apporte pas de protection indépendante contre la réécriture simultanée des deux fichiers. Vérifier manuellement reste possible avec :

    python scripts/aerl.py verify --file $ledger --expected-head HASH_DE_REFERENCE --expected-sequence 1

Sans un checkpoint gardé séparément, une personne qui peut réécrire l'intégralité du fichier peut recalculer la chaîne. Le hash-chain détecte les modifications internes, les réordonnancements et les suppressions au milieu, mais ne prouve pas l'authenticité de l'auteur et ne garantit pas, seul, la détection de la suppression de la fin du journal. Le checkpoint externe répond à ce dernier cas s'il reste fiable.

## Limites et hygiène des données

- Il n'y a ni signature par clé publique, ni chiffrement du contenu, ni export réseau, ni collecteur distant intégré.
- La limite du journal est 64 Mio; chaque enregistrement est limité à 32 Kio et le payload à 16 Kio. AERL refuse d'écrire au-delà de ces limites. La rotation est volontairement manuelle pour ne pas casser la chaîne.
- Les clés de payload dont le nom ressemble à un secret ou à une donnée personnelle (token, mot de passe, clé privée, cookie, email, téléphone, etc.) sont refusées. Cela ne détecte pas les secrets dissimulés dans une valeur arbitraire : journaliser uniquement des métadonnées et jamais le texte utilisateur, les mots de passe ou les clés.
- Le verrou est coopératif; un acteur qui peut modifier directement le fichier ou tuer le processus peut toujours perturber le journal. Des checkpoints externes et des permissions strictes restent nécessaires.
- Horodatages et événements proviennent de l'hôte qui écrit le journal; l'heure n'est pas signée ni attestée matériellement.
- Aucune validation de matériel physique, de firmware ou d'absence générale de vulnérabilités n'est revendiquée.

## Validation reproductible

    python -m unittest discover -s tests -p "test_aerl.py" -v
    python -m unittest discover -s tests -p "test_aerl_checkpoint.py" -v
    python -m unittest discover -s tests -p "test_aerl_tool.py" -v
    python -m unittest discover -s tests -p "test_aerl_run.py" -v
    python -m py_compile scripts/aerl.py scripts/aerl_checkpoint.py scripts/aerl_tool.py scripts/aerl_run.py tests/test_aerl.py tests/test_aerl_checkpoint.py tests/test_aerl_tool.py tests/test_aerl_run.py

AERL n'accède pas au réseau et les tests n'attaquent que des fichiers temporaires locaux.


## Format binaire AERL-BIN/1

Le noyau Rust utilise un format binaire distinct du JSONL Python. Le contrat byte-à-byte, les offsets, l'ordre little-endian, la couverture SHA-256 et un vecteur connu sont décrits dans [AERL-BIN-FORMAT.md](AERL-BIN-FORMAT.md).


## Journalisation sémantique d'accessibilité

Le noyau Rust expose append_accessible_event pour enregistrer un rôle, des états et un identifiant source limité à la session. Le nom accessible n'est pas écrit dans le payload par défaut afin d'éviter de journaliser du texte utilisateur ou des libellés contenant des informations personnelles. Le timestamp est fourni par l'adaptateur et n'est pas attesté.
