# Analyse et Dissémination Spécifique du Firmware BIOS AMI UEFI ASUS M1603QA (v308)

**Statut: FIRMWARE_ANALYSIS_SPECIFIED / BARE_METAL_TARGET_COMPATIBLE**

Ce document consigne l'analyse technique complète de la structure du BIOS / Firmware UEFI d'origine de votre **ASUS VivoBook M1603QA** (American Megatrends International LLC, Build M1603QA.308, SMBIOS v3.3).

---

## 1. Structure Interne du Volume de Firmware AMI UEFI

1. **Volume de Flash SPI (SPI ROM - 16 Mo / 128 Mb)** :
   - **NVRAM Volume** : Stocke les variables EFI non volatiles (BootOrder, BootNext, SecureBoot, Setup).
   - **FV_MAIN (Main Firmware Volume)** : Fichiers FFS (Firmware File System) contenant les pilotes DXE (Driver Execution Environment).
   - **FV_RECOVERY (Recovery Firmware Volume)** : Routine de secours en cas d'échec de démarrage.

2. **Pilotes DXE Majeurs & Contrôleurs Matériels ASUS M1603QA** :
   - `PciHostBridgeDxe` : Énumération et gestion des bus PCIe (Processeur AMD Ryzen 7 5800H Cezanne).
   - `AmdGpuDxe` / `VgaClassDxe` : Initialisation de la carte graphique intégrée AMD Radeon Graphics et mise à disposition du Framebuffer VRAM via `EFI_GRAPHICS_OUTPUT_PROTOCOL` (GOP).
   - `HdaDxe` : Initialisation du contrôleur Azalia / Intel High Definition Audio (MMIO).
   - `ConInDxe` / `ConOutDxe` : Routage des entrées clavier HID et sortie console.

---

## 2. Intégration Native avec ZERO OS Bare-Metal

1. **Transition depuis UEFI DXE vers ZERO OS Kernel** :
   - Le chargeur de démarrage `EFI/BOOT/BOOTX64.EFI` de ZERO OS est chargé par le gestionnaire de démarrage UEFI (`BootManagerDxe`).
   - `BOOTX64.EFI` consomme les protocoles `gST->ConOut` (Text Display) et GOP (Framebuffer VRAM 1920x1080), puis appelle directement le point d'entrée du noyau OS ZERO (`zero_os_main`).
2. **Exécution Sans Risque** :
   - Le noyau ZERO OS s'exécute entièrement en RAM (40 Go disponibles) de manière isolée sans altérer le volume SPI ROM, la NVRAM ni la partition Windows 11.
