# AERL — Accessibility Evidence & Reliability Log

**État : prototype local, chaîne d'intégrité testée par CI; pas un remplacement général de syslog.**

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
    python -m py_compile scripts/aerl.py scripts/aerl_checkpoint.py tests/test_aerl.py tests/test_aerl_checkpoint.py
    python -m py_compile scripts/aerl.py tests/test_aerl.py

AERL n'accède pas au réseau et les tests n'attaquent que des fichiers temporaires locaux.
