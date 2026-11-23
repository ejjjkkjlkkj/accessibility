# Informations manquantes pour remplacer Windows

Document de travail. Lecture seule : rien ici n'implique d'écrire sur le disque interne, la NVRAM, le firmware ou le CPU.

## 1. Ce que l'agent relève déjà (rapport `RAPPORT_UNIQUE.TXT`)

| Domaine | Couvert | Phase |
|---|---|---|
| Tables ACPI (29 SDT + RSDP) | oui, en brut | 2 |
| Carte mémoire UEFI | oui | 3 |
| Menus HII (BIOS) | oui, export complet | 4 |
| Disques (BlockIO, GPT, MBR) | oui | 5 |
| Protocoles UEFI (handles) | oui | 8 |
| PCI : 256 octets par fonction | oui | 7, 15b |
| NVRAM (variables) | oui | 9 |
| SMBIOS | oui | 11 |
| TPM : journal de mesures | oui | 12 |
| NVMe : identification, SMART, secteur, table GPT | oui | 13, GPT |
| USB : descripteurs | oui | 14 |
| Écran : EDID, modes GOP | oui | 15a, 15 |
| Fichiers de chaque partition (FAT via firmware) | oui | 16 |
| CPUID et registres de contrôle | oui | 17 |
| Sortie des services UEFI, xHCI, HDA, NVMe, clavier | oui, avec écritures | R0 |

## 2. Ce qui manque pour remplacer Windows

### Stockage
- **NTFS (partition C:)** : aucune lecture par l'agent. Il faut lire le MFT et les fichiers de démarrage. Lecture seule.
- **Table GPT complète** : seule la liste des partitions est lue. Les types et GUID des partitions Windows restent à confronter à `Get-Partition` sous Windows.
- **Santé NVMe** : le SMART est lu, mais les journaux d'erreurs (log pages 01h et 03h) ne le sont pas.

### Firmware et chipset
- **DSDT** : le bytecode n'est pas décompilé (il faut `iasl`, absent). Les méthodes `_PRT`, `_CRS`, `_DSM` ne sont donc pas lues.
- **Volumes FV de la SPI** : le dump de 16 Mo n'est pas analysé par l'agent (volumes FFS, modules, compression LZMA).
- **Chipset AMD** : les registres SMBus, LPC et l'état de la fenêtre SPI ne sont pas relevés.

### Processeur
- **Révision de microcode** : pas relevée (lecture par MSR, qui a déjà bloqué la machine).
- **Topologie** : CPUID sous-feuilles 0Bh (cœurs et threads) à vérifier.

### Périphériques indispensables à un système sans Windows
- **Clavier** : xHCI initialisé en R0, mais l'énumération des périphériques n'est pas confirmée sur cette machine.
- **Affichage** : GOP en mode texte uniquement. Pas de pilote GPU AMD (`1002:1638`).
- **Son** : HDA, test de lecture seulement, pas de pilote complet (`1022:15E3`, codec ALC256 signalé).
- **Réseau** : Wi-Fi MediaTek MT7921 (`14C3:7961`) : firmware propriétaire, aucune spec publique.
- **Énergie** : état S3, gestion de l'alimentation, aucune donnée relevée.

## 3. Ordre suggéré (lecture seule, à valider sur la machine)

1. Lire les volumes NTFS du NVMe (MFT, `$Boot`), sans écriture.
2. Relever la table GPT complète avec les GUID de type, à comparer à `Get-Partition`.
3. Décompiler la DSDT avec `iasl` sur l'hôte (hors machine), puis relever les méthodes `_PRT` et `_CRS`.
4. Relever les pages de journal NVMe 01h et 03h.
5. Relever les sous-feuilles CPUID 0Bh pour la topologie.

## 4. Ce qui reste hors du périmètre de cet agent

- Écriture sur le disque interne, la NVRAM, la SPI, le microcode ou la SMM.
- Installation persistante dans le firmware, le CPU ou le chipset.
- Modification de la table de partitions Windows.

Ces actions sont exclues parce qu'elles peuvent rendre la machine inutilisable et qu'elles sortent du cadre de lecture seule de ce projet.
