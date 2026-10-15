# AERL-BIN/1 — format binaire de journal de preuves

**État : contrat format version 1, implémentation Rust no_std en cours de validation.**

AERL-BIN/1 est un format binaire borné utilisé par le crate Rust crates/aerl-core. Il n'est pas compatible octet-à-octet avec le profil texte AERL-JSONL/1 du CLI Python. Aucun convertisseur n'est fourni.

## Disposition d'une trame

Tous les entiers multi-octets sont little-endian. Les offsets sont relatifs au début de chaque trame. Les chaînes acteur et type sont des octets UTF-8, sans terminaison NUL.

| Offset | Longueur | Champ | Règle |
|---:|---:|---|---|
| 0 | 4 | Magic | ASCII AERL |
| 4 | 1 | Version | 0x01 |
| 5 | 8 | Séquence | u64, commence à 1 et augmente exactement de 1 |
| 13 | 8 | Horodatage UTC millisecondes Unix | u64 fourni par l'adaptateur; non authentifié |
| 21 | 1 | Statut | 0=INFO, 1=PASS, 2=FAIL, 3=PARTIAL, 4=BLOCKED, 5=NOT_VALIDATED, 6=UNPROVEN |
| 22 | 1 | Longueur acteur | 1 à 63 octets |
| 23 | 1 | Longueur type | 1 à 80 octets |
| 24 | 2 | Longueur payload | 0 à 4096 octets |
| 26 | 32 | Hash précédent | 32 octets zéro sur la première trame; sinon hash courant de la trame précédente |
| 58 | variable | Corps | acteur UTF-8, type UTF-8, payload opaque, dans cet ordre |
| après le corps | 32 | Hash courant | SHA-256 de tous les octets de la trame à partir de Magic jusqu'à la fin du payload; les 32 octets du hash courant sont exclus du calcul |

La taille d'une trame est 90 + longueur_acteur + longueur_type + longueur_payload. La taille totale maximale vérifiée est 64 Mio. Le noyau rejette les versions inconnues, longueurs hors limites, séquences incorrectes, chaînes non UTF-8, statuts invalides, liens cassés et hashes incorrects.

## Vecteur de test indépendant

Événement d'entrée :
- timestamp : 0
- acteur : QA
- type : test
- statut : PASS (1)
- payload : les deux octets ASCII {}

La trame complète (98 octets, en hexadécimal) est :

    4145524c0101000000000000000000000000000000010204020000000000000000000000000000000000000000000000000000000000000000005141746573747b7d387cec86f9cd329ef8c20f7a9ddc0fda8eafa4639d9d1436e6abcfec8491645f

Hash courant attendu :

    387cec86f9cd329ef8c20f7a9ddc0fda8eafa4639d9d1436e6abcfec8491645f

Le test Rust binary_format_matches_published_golden_vector doit produire cette tête exacte. Les implémentations d'autres langages peuvent utiliser ce vecteur avant d'être reliées à la même interface binaire.


## Payload sémantique d'accessibilité ACCE/1

La fonction Rust append_accessible_event accepte un AccessibleEvent validé provenant de accessibility-core et crée un payload de 25 octets :

| Offset du payload | Longueur | Champ |
|---:|---:|---|
| 0 | 4 | Magic ASCII ACCE |
| 4 | 1 | Version du payload, 0x01 |
| 5 | 4 | Code de rôle partagé u32 little-endian |
| 9 | 8 | Drapeaux d'état u64 little-endian |
| 17 | 8 | Identifiant source u64 little-endian |

Le type d'événement AERL est accessibility.semantic. Le nom accessible n'est volontairement pas écrit : les libellés d'interface peuvent contenir des informations privées. L'identifiant source peut lui-même permettre une corrélation; l'adaptateur doit utiliser un identifiant éphémère limité à la session plutôt qu'un identifiant global personnel.

La fonction refuse un événement sémantique invalide avant toute écriture. Cela relie le contrat logique rôles/états au journal vérifiable sans enregistrer le texte de l'interface.

## Limites de sécurité

- SHA-256 chaîné détecte les altérations de trames. Ce n'est ni une signature, ni un MAC; un attaquant qui peut réécrire le journal entier peut recalculer tous les hashes.
- La suppression de la fin du journal n'est détectable que si une tête et une séquence attendues sont conservées séparément dans un domaine de confiance indépendant.
- Le timestamp et l'identité d'acteur sont fournis par l'appelant; le format ne les authentifie pas.
- Le payload est opaque. L'adaptateur doit retirer secrets, identifiants personnels, mots de passe, contenu utilisateur et toute donnée qui n'est pas nécessaire.
- Le crate est no_std et interdit unsafe, mais il ne gère ni la persistance, ni le verrouillage des périphériques, ni les interruptions, ni la récupération de puissance, ni l'accès firmware.
- Une compilation pour x86_64-unknown-uefi ne prouve pas le démarrage sur un PC réel ou le fonctionnement du matériel.
