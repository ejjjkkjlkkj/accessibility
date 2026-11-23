# Lecture ACPI 6.5 (ACPI_Spec_6_5_Aug29.txt, 57 717 lignes) — notes de l'agent

## Carte du document (table des matières, lignes du .txt)
- 5.2.5 RSDP : 6823 ; 5.2.6 en-tête de table : 6909 ; 5.2.7 RSDT : 7090 ; 5.2.8 XSDT : 7185 ; 5.2.9 FADT : 7281 (9.1 PM profile 8902, 9.2-9.3 flags 8918-9035) ; 5.2.11 DSDT/SSDT : 9526 ; 5.2.12 MADT : 9714 (LAPIC 10096, IOAPIC 10204, ISO 10261, x2APIC 10797) ; 5.2.13 GSI : 12165 ; 5.2.16 SRAT : 12410 ; 5.2.17 SLIT : 13272 ; 5.2.23 BGRT : 16141 ; 5.2.24 FPDT : 16276 ; 5.2.30 PPTT : 20956 ; 5.3 Namespace : 22908 ; 8.4 processeurs : 32912 ; 9.12 _UPC : voir table des matières (p.569) ; 6.1.8 _PLD : p.328.
- Tables utiles à l'agent : RSDP, XSDT/RSDT, FADT, DSDT, MADT (IOAPIC, LAPIC, x2APIC), MCFG (réservée ACPI, lignes à localiser), SRAT/SLIT (topologie NUMA), PPTT (topologie CPU), HPET, _UPC/_PLD (ports USB).

## 5.2.5 RSDP (lignes 6823-6908)
- Recherche IA-PC : premier 1 KiB de l'EBDA (40:0Eh) et 0E0000h-0FFFFFh, sur frontière 16 octets.
- UEFI : GUID ACPI 2.0+ = 8868e871-e4f1-11d3-bc22-0080c73c8881 ; ACPI 1.0 = eb9d2d30-2d88-11d3-9a16-0090273fc14d. Chercher d'abord le GUID 2.0, sinon 1.0. Doit être lu AVANT ExitBootServices.
- Structure : signature "RSD PTR " (espace final), checksum sur 20 octets (somme = 0), OEMID 6, Revision (2 = ACPI 2+), RsdtAddress (4), Length (4, rév.2+), XsdtAddress (8, rév.2+), ExtendedChecksum (1, somme sur toute la longueur), 3 octets réservés.
- Règle OSPM : utiliser XSDT si présent (5.2.8).

## 5.2.6 En-tête de table (lignes 6909-7089)
- 36 octets : Signature(4) @0, Length(4) @4, Revision(1) @8, Checksum(1) @9 (somme de la table = 0), OEMID(6) @10, OEM Table ID(8) @16, OEM Revision(4) @24, Creator ID(4) @28, Creator Revision(4) @32.
- Si la signature n'est pas dans la liste (5.5/5.6), OSPM ignore la table.
- Signatures définies par ACPI (5.5) : APIC, BERT, BGRT, CCEL, CPEP, DSDT, ECDT, EINJ, ERST, FACP, FACS, FPDT, GTDT, HEST, MISC, MSCT, MPST, NFIT, OEMx, PCCT, PHAT, PMTT, PPTT, PSDT, RASF, RAS2, RSDT, SBST, SDEV, SLIT, SRAT, SSDT, SVKL, XSDT.
- Réservées ACPI (5.6) incluent MCFG, HPET, DMAR, IVRS, LPIT, IORT, WDAT, TPM2, UEFI, SPCR, etc.

## 5.2.7 RSDT (7090-7184) / 5.2.8 XSDT (7185-7280)
- RSDT : signature 'RSDT', tableau 32 bits, entrées à partir de l'offset 36 (4*n).
- XSDT : signature 'XSDT', tableau 64 bits à partir de l'offset 36 (8*n). L'OS doit utiliser XSDT si présent.
- Le champ TableID doit correspondre à l'OEM Table ID de la FADT.

## 5.2.9 FADT (7281 ...) — lu jusqu'à offset 40
- Signature 'FACP' (nom historique). Major version = Revision (offset 8). Checksum offset 9.
- FIRMWARE_CTRL @36 (32 bits, adresse FACS) ; DSDT @40 (32 bits). Si X_FIRMWARE_CTRL / X_DSDT (64 bits, plus loin dans la table) sont non nuls et utilisables, ils priment sur les champs 32 bits (règle générale p.7283-7284).
- Si flag HW_REDUCED_ACPI : ignorer les champs 46-108 et 148-232 et certains bits de flags (1,2,3,7,8,13,14,16,17).
- Vérification agent : vérifier que le code lit X_DSDT en priorité (offset 140) et ne prend le DSDT 32 bits qu'en repli.

## 5.2.9 FADT — suite (lignes 7423-7872, offsets 44-100)
- 44 : INT_MODEL (ACPI 1.0 seulement ; 0 attendu).
- 45 Preferred_PM_Profile : 0 Unspecified, 1 Desktop, 2 Mobile, 3 Workstation, 4 Enterprise Server, 5 SOHO Server, 6 Appliance PC, 7 Performance Server, 8 Tablet, >8 réservé.
- 46 SCI_INT (16 bits) : vecteur 8259 ou GSI du SCI (partageable, level, active low).
- 48 SMI_CMD (32), 52 ACPI_ENABLE (8), 53 ACPI_DISABLE (8), 54 S4BIOS_REQ (8), 55 PSTATE_CNT (8).
- 56 PM1a_EVT_BLK, 60 PM1b_EVT_BLK, 64 PM1a_CNT_BLK, 68 PM1b_CNT_BLK, 72 PM2_CNT_BLK (32 bits, adresses port). Si X_xxx 64 bits non nul utilisable → prioritaire.
- 76 PM_TMR_BLK (4 octets ; PM timer), 80 GPE0_BLK, 84 GPE1_BLK.
- 88 PM1_EVT_LEN (≥4), 89 PM1_CNT_LEN (≥2), 90 PM2_CNT_LEN, 91 PM_TMR_LEN (=4 si timer), 92 GPE0_BLK_LEN, 93 GPE1_BLK_LEN (multiples de 2), 94 GPE1_BASE, 95 CST_CNT, 96 P_LVL2_LAT (µs, >100 = pas de C2), 98 P_LVL3_LAT (µs, >1000 = pas de C3).
- 100 FLUSH_SIZE, FLUSH_STRIDE, DUTY_OFFSET, DUTY_WIDTH, DAY_ALRM ... (suite non lue).
- Vérification agent : l'agent ne doit pas écrire dans ces ports (SMI_CMD, ACPI_ENABLE) sans accord explicite ; lecture seule du PM_TMR possible.

## 5.2.9 FADT suite — lignes 7873-8470
- 104/105 : DUTY_OFFSET (index dans P_CNT), DUTY_WIDTH (0 = non supporté) ; fréquence = BF × DC/2^DUTY_WIDTH si THTL_EN=1.
- 106 DAY_ALRM, 107 MON_ALRM, 108 CENTURY : index CMOS RTC (0 = non supporté ; MON_ALRM exige DAY_ALRM).
- 109 IAPC_BOOT_ARCH (§5.2.9.3). 111 : doit être 0. 112 Flags fixes (Table 5.10).
- 116 RESET_REG (Generic Address : espace I/O, mémoire, PCI bus0 ; Register_Bit_Width = 8, Bit_Offset = 0). 128 RESET_VALUE (valeur à écrire).
- 129 ARM_BOOT_ARCH. 131 Minor version (bits 3:0) et errata (bits 7:4). 132 X_FIRMWARE_CTRL (64 bits, prioritaire si non nul).
- 140 X_DSDT (64 bits, prioritaire sur DSDT 32 bits @40 si non nul).
- 148 X_PM1a_EVT_BLK, 160 X_PM1b_EVT_BLK, 172 X_PM1a_CNT_BLK, 184 X_PM1b_CNT_BLK, 196 X_PM2_CNT_BLK, 208 X_PM_TMR_BLK, 220 X_GPE0_BLK, 232 X_GPE1_BLK (Generic Address ; prioritaires sur les 32 bits s'ils sont non nuls).
- 244 SLEEP_CONTROL_REG, 256 SLEEP_STATUS_REG (Generic Address ; I/O, mémoire ou PCI bus0 ; largeur 8, offset 0) — §4.8.3.7.
- 268 Hypervisor Vendor Identity (8 octets ; zéro si pas d'hyperviseur).
- Table 5.10 : Flags fixes. Bit 0 WBINVD (fonction correcte) ; bit 1 WBINVD_FLUSH (caches maintenus cohérents sans invalidation) ; bit 2 PROC_C1 (C1 sur tous les processeurs) ; bit 3 P_LVL2_UP (C2 UP/MP) ; bit 4 PWR_BUTTON (0 = bouton géré par ACPI, 1 = pas de bouton) ; bit 5 SLP_BUTTON ; bit 6 FIX_RTC ; bit 7 RTC_S4 ; bit 8 TMR_VAL_EXT (timer 32 bits) ; bit 9 DCK_CAP ; bit 10 RESET_REG_SUP ; bit 11 SEALED_CASE ; bit 12 HEADLESS (suite lue ensuite).

## FADT Flags (suite) lignes 8473-8900 et §5.2.9.1-5.2.9.2
- Flags : bit 12 HEADLESS (suite), 13 CPU_SW_SLP (instruction native après écriture SLP_TYPx), 14 PCI_EXP_WAK (bit PCIEXP_WAK dans PM1), 15 USE_PLATFORM_CLOCK (timer plateforme pour compteurs monotones ; HPET puis PM timer), 16 S4_RTC_STS_VALID, 17 REMOTE_POWER_ON_CAPABLE (GPE armés avant S5), 18 FORCE_APIC_CLUSTER_MODEL, 19 FORCE_APIC_PHYSICAL_DESTINATION_MODE, 20 HW_REDUCED_ACPI, 21 LOW_POWER_S0_IDLE_CAPABLE (économies S0 ≥ S3), 22 PERSISTENT_CPU_CACHES (00b non rapporté, 01b non persistant, 10b persistant, 11b réservé), bits 23 réservé.
- §5.2.9.1 profils : Desktop, Mobile, Workstation, Enterprise Server, SOHO Server, Appliance PC, Performance Server, Tablet (texte de référence).
- §5.2.9.3 IAPC_BOOT_ARCH (offset 109, 2 octets) : bit0 LEGACY_DEVICES (dispositifs visibles sur LPC/ISA), bit1 8042 (contrôleur clavier ports 60/64), bit2 VGA_NOT_PRESENT (ne pas sonder A0000h-BFFFFh, 3B0-3BB, 3C0-3DF), bit3 MSI_NOT_SUPPORTED, bit4 PCIE_ASPM_CONTROLS (ne pas activer ASPM), bit5 CMOS_RTC_NOT_PRESENT (utiliser Time and Alarm 9.17), bits 15:6 réservés = 0.
- §5.2.9.4 ARM boot flags (non applicable x86) : PSCI_COMPLIANT bit0, PSCI_USE_HVC bit1.
- Note agent : si IAPC_BOOT_ARCH bit2 = 1, l'agent ne doit pas sonder le VGA (ce qui n'est pas fait actuellement à vérifier) ; si bit1 = 0, pas de contrôleur 8042 supposé.

## 5.2.10 FACS (lignes 9078-9430)
- 'FACS' ; longueur ≥ 64 ; alignée 64 octets ; mémoire NON rapportée comme RAM utilisable (E820 = Reserved).
- Offset 8 : Hardware Signature (doit changer si les tables ACPI changent). Offset 12 : Firmware Waking Vector 32 bits (superseded). Offset 16 : Global Lock (32 bits). Offset 20 : Flags (S4BIOS_F bit0, 64BIT_WAKE_SUPPORTED_F bit1). Offset 24 : X_Firmware_Waking_Vector (64 bits, prioritaire). Offset 32 : Version (3). Offset 36 : OSPM Flags (64BIT_WAKE_F bit0).
- Global Lock (§5.2.10.1) : bit0 pending, bit1 owned ; acquisition par cmpxchg ; libération avec signal GBL_RLS/BIOS_RLS si pending. Logique à utiliser si l'agent touche des registres partagés SMI/OS (ne pas le faire sans besoin).
- Pour l'agent : FACS lue en lecture seule uniquement ; ne pas écrire le Waking Vector.

## 5.2.11 Définitions de blocs (lignes 9526-9545)
- Un bloc = en-tête + AML. DSDT chargée en premier (pointeur FADT DSDT ou X_DSDT), SSDT ensuite dans l'ordre RSDT/XSDT.
- Champ Revision de l'en-tête DSDT/SSDT : valeur ≥ 2 → entiers 64 bits dans l'interpréteur AML ; < 2 → 32 bits.

## 5.2.11.1 DSDT (9543-9624) — en-tête 36 octets + AML
- 'DSDT' ; Length @4 ; Revision @8 (≥2 = entiers 64 bits) ; Checksum @9 ; OEMID @10 ; OEM Table ID @16 ; OEM Rev @24 ; Creator ID @28 ; Creator Rev @32 ; AML @36.

## 5.2.11.2 SSDT (9625-) — même en-tête, signature 'SSDT' ; ne peut qu'ajouter, pas écraser.

## 5.2.11.3 PSDT (9712) : traitée comme SSDT.

## 5.2.12 MADT (9714-10094)
- Signature 'APIC' ; en-tête 36 o ; @36 Local Interrupt Controller Address (32 bits, adresse LAPIC des processeurs) ; @40 Flags (bit0 PCAT_COMPAT = double 8259 présent → masquer les vecteurs 8259 avant d'activer l'APIC) ; @44 liste de structures d'interruption (Type 1 octet, Length 1 octet).
- Types (Table 5.21) : 0 Processor Local APIC (§5.2.12.2) ; 1 I/O APIC (§5.2.12.3) ; 2 Interrupt Source Override (§5.2.12.5) ; 3 NMI Source ; 4 Local APIC NMI ; 5 Local APIC Address Override ; 6 I/O SAPIC ; 7 Local SAPIC ; 8 Platform Interrupt Sources ; 9 Processor Local x2APIC ; 0xA Local x2APIC NMI ; 0xB GICC ; 0xC GICD ; 0xD GIC MSI Frame ; 0xE GICR ; 0xF GIC ITS ; 0x10 Multiprocessor Wakeup ; 0x11 CORE PIC ; 0x12 LIO PIC ; 0x13 HT PIC ; 0x14 EIO PIC ; 0x15 MSI PIC ; 0x16 BIO PIC ; 0x17 LPC PIC ; 0x18-0x7F réservés (OSPM ignore) ; 0x80-0xFF réservés OEM.
- Ordre : initialiser les processeurs dans l'ordre MADT ; le BSP doit être la 1re entrée ; les premiers threads avant les seconds threads.
- Processor Local APIC (type 0, longueur 8) : @2 ACPI Processor UID (1 o), @3 APIC ID (1 o), @4 Flags (4 o) : bit0 Enabled, bit1 Online Capable, bits 31:2 = 0.
- Si ni Enabled ni Online Capable : processeur inutilisable.
- I/O APIC (type 1, longueur 12) : @2 I/O APIC ID (1), @3 réservé, @4 adresse I/O APIC 32 bits, @8 Global System Interrupt Base (32 bits). Nombre d'entrées = Max Redir Entry du registre.
- Double 8259 + APIC : GSI 0-15 = IRQ 0-15 sauf overrides ; GSI > 15 ignorés en mode 8259.
- Interrupt Source Override (type 2) : ISA IRQ → GSI (ex. PIT IRQ0 → GSI 2 ; cf. §5.2.12.5 et Flags).
- Agent : la MADT permet de savoir le nombre de processeurs et l'adresse LAPIC/I/O APIC — à lire seulement, aucun contrôle des interruptions sans besoin.

## MADT suite (10273-10870)
- Interrupt Source Override (type 2, longueur 10) : @2 Bus (0 = ISA), @3 Source (IRQ), @4 GSI (32 bits), @8 Flags MPS INTI (2 bits polarité : 00 bus, 01 haut, 11 bas ; 2 bits déclenchement : 00 bus, 01 front, 11 niveau). Doit être présent pour SCI si SCI non identité.
- NMI Source (type 3, longueur 8) : @2 Flags, @4 GSI.
- Local APIC NMI (type 4, longueur 6) : @2 ACPI Processor UID (0xFF = tous), @3 Flags, @5 LINTn.
- Local APIC Address Override (type 5, longueur 12) : @4 réservé 16 bits, @8 adresse 64 bits LAPIC (un seul, remplace l'adresse 32 bits de l'en-tête).
- I/O SAPIC (type 6, longueur 16) : @2 ID, @4 GSI base, @8 adresse 64 bits.
- Local SAPIC (type 7, longueur ≥ 16) : ID, EID, UID (valeur ou chaîne). Itanium — non concerné.
- Platform Interrupt Source (type 8, longueur 16) : Flags (bits 1:0 type: 1 PMI, 2 INIT, 3 CPEI), Processor ID/EID, vecteur, GSI, flags CPEI-Processor-Override (bit0). Non concerné x86 hors cas CPEI.
- Processor Local x2APIC (type 9, longueur 16) : @4 x2APIC ID (32 bits), @8 Flags (même format que LAPIC), @12 ACPI Processor UID (32 bits). Obligatoire pour les processeurs d'ID ≥ 255. Compatibilité : processeurs ID < 255 déclarés en Processor() / structure LAPIC.
- Local x2APIC NMI (type 0xA, longueur ≥ 12) : pour x2APIC ID ≥ 255 ; Local APIC NMI pour ID < 255 (cf. §5.2.12.13).
- Agent : nombre de cœurs = nombre d'entrées LAPIC/x2APIC avec bit Enabled (bit0 de Flags) ou Online Capable.

## MADT suite ARM/GIC (10873-11472) — non pertinent x86 (résumé)
- Local x2APIC NMI (type 0xA, longueur 12) : @2 Flags, @4 UID 32 bits (0xFFFFFFFF = tous), @8 LINTn.
- GICC (type 0xB, longueur 82) : CPU Interface Number, UID, Flags (bit0 Enabled, bit1 Performance Interrupt Mode, bit2 VGIC maintenance mode, bit3 Online Capable), adresses GICC/GICV/GICH, GSIV maintenance, GICR base, MPIDR, efficacité, SPE/TRBE GSIV.
- GICD (type 0xC, longueur 24) : GIC version (0 = pas spécifié, 1..4 = GICv1..v4), adresse base 64 bits, system vector base.
- GIC MSI Frame (type 0xD, longueur 24) : ID, adresse 64 bits, flags (bit0 SPI Count/Base select), SPI count/base.
- GICR (type 0xE, longueur 16) : base 64 bits + longueur de la zone de découverte.
- ITS, Multiprocessor Wakeup, CORE PIC, LIO PIC, HT PIC, EIO PIC, MSI PIC, BIO PIC, LPC PIC : structures d'autres architectures (LoongArch, etc.), non pertinentes pour l'agent x86.

## MADT suite (11473-12070) — structures non x86 (résumé)
- ITS (0xF, longueur 20) : ID, adresse 64 bits.
- Multiprocessor Wakeup (0x10, longueur 16) : version mailbox (0), adresse mailbox (4 KiB alignée, ACPINvs). Mailbox : Command (0 Noop, 1 Wakeup, autres réservés), ApicId, WakeupVector (64 bits, long mode, identité, page unique), OS section 2 KiB, firmware 2 KiB. Pertinent si l'agent réveille des AP par mailbox (non le cas actuel).
- CORE PIC (0x11), LIO PIC (0x12), HT PIC (0x13), EIO PIC (0x14), MSI PIC (0x15), BIO PIC (0x16), LPC PIC (0x17) : architecture LoongArch / Loongson, non pertinentes x86.

## Suite MADT / 5.2.13-5.2.16 (12071-12670)
- LPC PIC (0x17) : LoongArch, non pertinent.
- §5.2.13 GSI : modèle APIC : chaque I/O APIC couvre GSI base .. base + (Max Redirection Entry), lu dans le registre I/O APIC ; modèle 8259 : GSI 0-15 = IRQ ISA.
- §5.2.14 SBST (Smart Battery, 'SBST') : Warning energy @36, Low @40, Critical @44 (mWh). Non prioritaire agent.
- §5.2.15 ECDT ('ECDT', longueur variable) : EC_CONTROL (GAS @36 pointeur commande/état), EC_DATA (GAS @48), UID @60 (4), GPE_BIT @64 (1), EC_ID @65 (chaîne ASCII nulle, ex "\_SB.PCI0.ISA.EC"). ACPI 1.0 ignore la table. Méthode _REG/ECAV pour détecter les régions EC disponibles.
- §5.2.16 SRAT ('SRAT') : en-tête 36 o ; @36 Revision (=1), @40 réservé 8 octets ; liste à partir de @48. Doit contenir tous les processeurs démarrés au boot (APIC/x2APIC ID ou GICC UID). Sous-structures :
  - Processor Local APIC/SAPIC Affinity (type 0, longueur 16) : @2 Proximity Domain [7:0], @3 APIC ID, @4 Flags (bit0 Enabled), @8 SAPIC EID, @9 Proximity Domain [31:8] (24 bits), @12 Clock Domain (32 bits).
  - Memory Affinity (type 1) : voir §5.2.16.2 (suite).
  - Processor Local x2APIC Affinity (type 2), GICC Affinity (type 3), ITS Affinity (type 4), Generic Initiator (type 5), Generic Port (type 6) : §5.2.16.3-7.
- Agent : la SRAT n'est utile que pour la topologie NUMA (Ryzen mono-nœud ; à confirmer dans l'instance).

## SRAT suite (12671-13265)
- Memory Affinity (type 1, longueur 40) : @2 Proximity Domain (32), @8 Base Low, @12 Base High, @16 Length Low, @20 Length High, @28 Flags : bit0 Enabled, bit1 Hot Pluggable, bit2 Non-Volatile ; bits 31:3 = 0.
- x2APIC Affinity (type 2, longueur 24) : @4 Proximity Domain (32), @8 x2APIC ID (32), @12 Flags (bit0 Enabled), @16 Clock Domain (32).
- GICC Affinity (type 3, longueur 18) : @2 PD (32), @6 ACPI Processor UID (32), @10 Flags (bit0 Enabled), @14 Clock Domain.
- ITS Affinity (type 4, longueur 12) : @2 PD (32), @8 ITS ID (32). ARM.
- Generic Initiator Affinity (type 5, longueur 32) : @3 Device Handle Type (0 ACPI : _HID 8 o @0, _UID 4 o @8 ; 1 PCI : Segment 2 o, BDF 2 o), @4 PD (32), @8 Device Handle (16), @24 Flags (bit0 Enabled). Optionnel, à déclarer via _OSC.
- Generic Port Affinity (type 6, longueur 32) : mêmes champs (PD pour HMAT, Device Handle, Flags bit0 Enabled, bit1 Architectural transactions). CXL.
- Agent : seule utilité pratique = topologie NUMA ; non critique pour le boot UEFI de l'agent.

## 5.2.17 SLIT (13272-13398)
- 'SLIT' ; @36 Number of System Localities (N, 64 bits) ; matrice N×N d'octets à partir de @44, entrée [i][j] = distance i→j. Diagonale = 10. Valeur 255 = inaccessible. 0-9 réservés (hors diagonale).

## 5.2.18 CPEP (13399-13520)
- 'CPEP' ; @36 réservé ; liste à partir de @44 de structures (type 0 APIC/SAPIC : @2 Processor ID, @3 EID, @4 intervalle de scrutation en ms).

## 5.2.19 MSCT (13572-13730)
- 'MSCT' ; @36 OffsetProxDomInfo (32) ; @40 Max Proximity Domains − 1 ; @44 Max Clock Domains − 1 ; @48 Max Physical Address (64).
- Maximum Proximity Domain Info (révision 1, longueur 22) : @2 PD low (32), @6 PD high (32), @10 Max Processor Capacity (32), @14 Max Memory Capacity (64).

## 5.2.20 RASF (13810-) — en-tête standard, 'RASF'. Lu jusqu'à offset 36 ; table suite non lue ici.

## 5.2.20 RASF (13871-14204) — résumé (non pertinent agent)
- @36 identifiant du canal PCC (sous-canal). Mémoire partagée PCC : signature 0x52415346 ('RASF') @0, Command @4 (0 réservé, 1 Execute), Status @6, Version @8 (minor, major), RAS Capabilities @10 (bitmap : bit0 patrol scrub matériel, bit1 patrol scrub exposé logiciel), Set RAS Capabilities @26, nb de blocs paramètres @42, Status @44 (0 Success, 1 Not Valid, 2 Not Supported, 3 Busy, 4 Failed, 5 Aborted, 6 Invalid Data), blocs paramètres @48.
- Bloc PATROL_SCRUB (§5.2.20.5) : Type 0x0000, Version, Length, Command (1 GET, 2 START, 3 STOP), adresse demandée (base+taille 16 o), plage réelle, flags (bit0 déjà actif ; bits 3:1 vitesse Slow 000 / Medium 100 / Fast 111).

## 5.2.21 RAS2 (14205-14464) — résumé (non pertinent agent)
- 'RAS2' ; @36 réservé (2) ; @38 nombre de descripteurs PCC (2) ; @40 liste N×8 : Identifier de canal PCC (1 o @0), réservé @1, Feature Type (1 o @2 : 0x00 mémoire, 0x80-0xFF vendeur), Instance (4 o @4).
- Canal PCC : signature 0x50434300 | ID de sous-espace, command, status, version, RAS features (16 o), Set RAS capabilities (16 o), nombre de blocs paramètres (2 o), Status (4 o).

## RAS2 Memory (14471-14781) — résumé
- Feature type 0 (mémoire) ; bitmap : bit0 PATROL_SCRUB matériel, bit1 LA2PA (traduction adresse logique→physique). Paramètres (blocs 48+) : commande GET/START/STOP, plage demandée/réelle, flags (bit0 actif), scrub rates (min/max). Non pertinent agent.

## 5.2.22 MPST (14783-15070+) — résumé
- 'MPST' ; @36 Identifiant canal PCC ; @40 nombre de nœuds de puissance mémoire ; @42 nombre de structures caractéristiques. Commandes PCC : 1 GET état, 2 SET état, 3 puissance moyenne, 4 énergie consommée. Non pertinent agent (mémoire serveur).

## MPST suite (15071-15670) — résumé (non pertinent agent)
- Command status (bits) : 0 complete, 1 SCI doorbell, 2 erreur, 3 SCI platform, 4 réservé. Codes d'état 0-6 comme RASF.
- Séquences SET/GET : écrire nœud, état, commande SET(2)/GET(1) puis PCC Execute, doorbell, attendre SCI.
- Memory Power Node (flags : bit0 enabled, bit1 power managed, bit2 hot pluggable) ; Power State Characteristics (flags bit0 contenu préservé, bit1 entrée autonome/explicite, bit2 sortie explicite). MPS0 = actif, MPS1..n = états basse consommation.

## MPST fin / PMTT (15671-16140)
- MPST : état d'exit mesuré, notes NUMA/hot plug (MEM0 PNP0C80 via _CRS), OS allocation (mémoire non power-managed pour données non migrables). Non pertinent agent.
- PMTT ('PMTT', révision courante 2 ; rév. 1 dépréciée 6.4) : @36 nombre de devices top-level ; structure commune (type @0 : 0 Socket, 1 Memory Controller, 2 DIMM, 0xFF vendeur ; longueur @2 ; flags @4 : bit0 top-level aggregator, bit1 physique/logique, bits 3:2 volatile/non-volatile ; @8 nombre de devices enfants). Socket (type 0) : ID @12 (16). Memory Controller (1) : ID @12. DIMM (2) : SMBIOS handle @12 (0xFFFFFFFF = invalide). Vendor (0xFF) : UUID @12, données @28.
- Non prioritaire agent (topologie mémoire).

## 5.2.23 BGRT ('BGRT', 16141-16270)
- Table UEFI uniquement. @36 Version (2, doit être 1) ; @38 Status (1 o) : bit0 Displayed, bits 2:1 rotation (00 0°, 01 90°, 10 180°, 11 270°), bits 7:3 = 0 ; @39 Image Type (0 = bitmap) ; @40 Image Address (64 bits, en EfiBootServicesData ; bitmap 24 bits 0xRRGGBB ou 32 bits) ; @48 Image Offset X (32) ; @52 Image Offset Y (32).
- Remarque : doit passer Displayed à 0 si le démarrage est interrompu (appui touche).
- Agent : lecture seule ; pertinent pour détecter l'image du firmware.

## 5.2.24 FPDT ('FPDT', 16276-16870) — résumé utile
- En-tête standard ; @36 liste d'enregistrements (sous-en-tête : type 2 octets, longueur 1, révision 1 ; @4 données). Horodatages en ns depuis le reset (précision ±10 %).
- Types : 0x0000 pointeur FBPT (Firmware Basic Boot Performance), 0x0001 pointeur S3PT, 0x0002.. réservés.
- Pointeur FBPT (longueur 16, révision 1) : @8 adresse 64 bits de la table FBPT (zone ACPI Reserved).
- Table FBPT ('FBPT', @4 longueur) ; enregistrement type 2 (longueur 48, révision 2) : @8 Reset End, @16 OS Loader LoadImage Start, @24 OS Loader StartImage Start, @32 ExitBootServices Entry, @40 ExitBootServices Exit (tous 64 bits).
- Pointeur S3PT (type 1) @8 adresse S3PT. Table 'S3PT' : enregistrement type 0 (longueur 24, révision 1) : @4 nombre de reprises S3, @8 FullResume, @16 AverageResume (64 bits).
- Agent : utile pour mesurer le temps passé dans le firmware. Lecture seule.

## FPDT S3 suite (16871-16940)
- Basic S3 Suspend (type 1, longueur 20) : @4 SuspendStart, @12 SuspendEnd (timer, 32 bits chacun ; le texte donne les offsets 4 et 12 pour les deux champs).

## 5.2.25 GTDT (16941-17470) — ARM uniquement (résumé)
- 'GTDT' ; @36 adresse CntControlBase (64) ; @48 GSIV/flags par timer (Secure EL1, Non-Secure EL1, Virtual EL1, EL2), @80 CntReadBase (64), @88 nombre de Platform Timers, @92 offset, @96 Virtual EL2 GSIV.
- Flags : bit0 mode (1 front), bit1 polarité (1 actif bas), bit2 always-on.
- Platform Timers : type 0 GT Block (CntCtlBase, nombre de timers ≤ 8), type 1 Arm Generic Watchdog. Non pertinent x86.

## GTDT Watchdog / Common flags (17471-17547) — ARM, résumé
- Flags timer : bit0 mode, bit1 polarité, bit2 secure (watchdog), bit2 always-on (common). Arm Generic Watchdog (type 1, longueur 28) : RefreshFrame @4, ControlFrame @12 (64 bits), GSIV @20, flags @24.

## 5.2.26 NFIT ('NFIT', 17688-18070) — NVDIMM (résumé, non pertinent agent)
- En-tête 36 o ; @36 structures (type 2 o, longueur 2 o). Types : 0 SPA Range (GUID type @16, base @32, longueur @40, attributs mémoire @48 : UC 1, WC 2, WT 4, WB 8, UCE 0x10, WP 0x1000, RP 0x2000, XP 0x4000, NV 0x8000, MORE_RELIABLE 0x10000, RO 0x20000, SP 0x40000), 1 NVDIMM Region Mapping, 2 Interleave, 3 SMBIOS Management, 4 Control Region, 5 Block Data Window, 6 Flush Hint, 7 Platform Capabilities.
- GUID PM Region = 66F0D379-B4F3-4074-AC43-0D3318B78CDB ; Control = 92F701F6-13B4-405D-910B-299367E8234C ; Block Data Window = 91AF0530-5D86-470E-A6B0-0A2DB9408249 (autres GUIDs = disques RAM virtuels).
- Non pertinent : pas de NVDIMM dans l'agent.

## NFIT suite (18071-18670) — non pertinent agent (résumé)
- NVDIMM Region Mapping (type 1, longueur 48) : NFIT Device Handle @4, NVDIMM Physical ID @8, Region ID @10, SPA index @12, Control index @14, Region size @16, Offset @24, Base @32, Interleave index @40, ways @42, state flags @44 (bit0 SAVE échoué, bit1 RESTORE échoué, bit2 flush échoué, bit3 non armé, bit4 SMART avant handoff, bit5 notifs SMART, bit6 non mappé).
- Interleave (type 2) : index @4, lignes @8, taille de ligne @12, offsets de ligne @16.
- SMBIOS Management (type 3) ; NVDIMM Control Region (type 4, 32 ou 80 o) : index @4, vendeur @6, device @8, révision @10, sous-systèmes, date, numéro de série @24.

## NFIT fin / SDEV (18671-19270) — résumé
- Control Region suite : Region format interface code @28, nombre de Block Control Windows @30, offsets/tailles de registres @32-@72, flag buffered @72. Block Data Window (type 5) : Control index @4, nombre @6, offset @8, taille @16, capacité @24, adresse premier bloc @32.
- Flush Hint (type 6) : device handle @4, nombre m @8, adresses 64 bits @16+. Écrire pour forcer la persistance (non pertinent agent).
- Platform Capabilities (type 7, longueur 16) : highest valid bit @4 ; capacités @8 (bit0 cache CPU flushée sur perte de puissance, bit1 contrôleur mémoire flushé, bit2 miroir PM).
- Représentation identifiant NVDIMM : format "%02x%02x-%02x-%02x%02x-%02x%02x%02x%02x".

## 5.2.27 SDEV ('SDEV', 19102-19270) — dispositifs sécurisés (résumé)
- En-tête standard ; @36 liste de structures. Type 0 = dispositif namespace ACPI (@0 type, @1 flags : bit0 autoriser le transfert à l'OS non sécurisé, bit1 composants d'accès sécurisé présents ; @2 longueur ; @4 offset du nom ACPI ASCII ; @6 longueur du nom). Type 1 = dispositif PCIe endpoint. Autres = réservés (ignorer via Length).
- Non pertinent agent (pas de secure OS).

## SDEV suite (19271-19787) — résumé
- Secure Access Components : Identification (type 0 : hardware ID PNP/ACPI "AAA####"/"NNNN####", subsystem ID, revision, classe PCI), Memory (type 1 : adresse 64 bits + longueur 64 bits).
- PCIe Endpoint Device-based (type 1) : segment PCI @4, bus de départ @6, chemin PCI (device/function pairs) @8/@10, données vendeur @12/@14.

## 5.2.28 HMAT ('HMAT', 19788-19870+) — en-tête : @36 réservé ; révision 2 ; structures : Memory Proximity Domain Attributes (5.2.28.3), Latence/bande passante (5.2.28.4), Memory Side Cache (5.2.28.5). Évaluée au boot uniquement ; _HMA pour changements. Non critique agent.

## HMAT suite (19871-20470) — résumé
- Memory Proximity Domain Attributes (type 0, longueur 40) : flags @8 (bit0 initiator PD valide), initiator PD @12, memory PD @16 (32 bits chacun).
- System Locality Latency/Bandwidth (type 1) : @8 flags (bits 3:0 hiérarchie : 0 mémoire, 1-3 cache niveau n ; bits 5:4 attributs : 0x10 taille de transfert minimale, 0x20 non séquentiel), @9 type de donnée (0 latence accès, 1 lecture, 2 écriture, 3 bande passante accès, 4 lecture, 5 écriture), @10 MinTransferSize (exposant biaisé), @12 nombre d'initiateurs s (32), @16 nombre de cibles t, @24 Entry Base Unit (latence en ps, BW en MB/s, non nul), @32 liste initiateurs puis cibles puis matrice s×t de valeurs 16 bits (0xFFFF = injoignable). Préférer HMAT au SLIT si les deux existent.
- Memory Side Cache Information (type 2, longueur 4+) : @8 PD mémoire, @16 taille du cache (64), @24 attributs (bits 3:0 nombre de niveaux, bits 7:4 niveau décrit).
- Non critique agent.

## HMAT Memory Side Cache suite / PDTT (20471-21069)
- Memory Side Cache (type 2) : @8 PD mémoire ; @16 taille ; @24 attributs : 3:0 niveaux, 7:4 niveau décrit, 11:8 associativité (1 direct, 2 complexe), 15:12 politique (1 WB, 2 WT), 31:16 taille de ligne. @28 nombre de handles SMBIOS n (16) ; @32 handles (2×n).

## 5.2.29 PDTT ('PDTT', 20529-20955) — résumé
- @36 Trigger Count (1 o) ; @40 offset du tableau d'identifiants (2 o chacun) ; chaque identifiant : bits 7:0 index de sous-canal PCC (dans PCCT), bit 8 Run-time (0 seulement crash fatal), bit 9 Wait for Completion, bit 10 Trigger Order (0 avant crash dump, 1 après).
- Commandes PCC : 0x00 doorbell seul, 0x01 avec commande spécifique vendeur. Attente par bit0 du status.
- Pour l'agent : non pertinent (déclencheur de débogage plateforme lors d'un crash OS).

## 5.2.30 PPTT ('PPTT', 20956-21069+)
- En-tête 36 o ; corps = structures de topologie : Type 0 nœud processeur (@... : hiérarchie, parent, flags, ID, ressources privées), Type 1 cache (niveau, taille, associativité, ligne, politique...).
- Une structure Type 0 par processeur présenté dans la MADT et par groupe partageant une ressource ; chaque package a un nœud. Racine = parent 0.
- Pertinent agent : topologie CPU (cœurs/SMT, caches L1/L2/L3) ; à compléter au besoin.

## PPTT suite (21070-21545)
- Processor Hierarchy Node (type 0) : longueur @1, réservé @2, flags @4 (bit0 frontière de package physique, bit1 ACPI Processor ID valide, bit2 partage d'unités fonctionnelles (leaf), bit3 leaf, bit4 implémentation identique des enfants, bits 31:5 = 0), parent @8 (offset depuis le début de la PPTT, 0 = racine), ACPI Processor ID @12 (= MADT ou _UID), nombre de ressources privées @16, liste @20 (offsets de structures cache).
- Note : un thread SMT doit être un nœud dans son propre groupe de cœur ; les processeurs désactivés dans MADT rendent leurs nœuds désactivés.
- Cache Type (type 1, longueur 28) : @2 réservé, @4 flags (bit0 taille valide, bit1 nombre de sets, bit2 associativité, bit3 allocation, bit4 type, bit5 politique d'écriture, bit6 taille de ligne, bit7 Cache ID valide), @8 offset du cache de niveau suivant (0 = dernier), @12 taille (32), @16 nombre de sets, @20 associativité (ways), @21 allocation (bits 1:0 : 0 read, 1 write, 2/3 both), @21 bits 3:2 type (0 data, 1 instruction, 2/3 unifié), bit 4 politique (0 WB, 1 WT), @22 taille de ligne (16), @24 Cache ID (32, non nul).
- Structures lues jusqu'à 21545. PHAT à suivre.

## 5.2.31 PHAT ('PHAT', 21546-21670+)
- En-tête 36 o ; révision 1 ; @36 liste d'enregistrements (type @0 2 o, longueur @2 2 o, révision @4 ...). Télémétrie de santé plateforme (versions firmware pré-OS, état des pilotes firmware). Pas d'action OSPM attendue.

## PHAT suite (21670-22231)
- Enregistrement : type @0 (2), longueur @2 (2), révision @4 (1), données @5.
- Type 0x0001 Firmware Version Data : count @8, éléments @12 (GUID 16 + version 64 + Vendor ID 32, ID 0xFFFF = non défini).
- Type 0x0002 Firmware Health Data : GUID @4 (16), offset données @24, device path @28, état @7 (0 erreurs, 1 OK, 2 inconnu, 3 avertissement).
- Reset Reason Health Record (GUID 7a014ce2-f263-4b77-b88a-e6336b782c14) : source supportée @... (bits : 0 inconnu, 1 hardware, 2 firmware, 3 logiciel, 4 superviseur), sous-source, raison (0 UNKNOWN, 1 COLD BOOT, 2 COLD RESET, 3 WARM RESET, 4 UPDATE, 32 UNEXPECTED, 33 FAULT, 34 TIMEOUT, 35 THERMAL, 36 POWER LOSS, 37 POWER BUTTON), nombre d'entrées vendeur.
- Pertinent agent : lecture seule de la cause du dernier reset (utile à consigner dans le rapport).

## 5.2.32 VIOT ('VIOT', 22232-22231+) — IOMMU para-virtualisé (virtio-iommu), non pertinent agent. En-tête standard ; @36 Node Count (32), @40 Node Offset (16).

## VIOT suite (22271-22570) — non pertinent agent (résumé)
- Nœuds alignés 8 o ; @38 offset du premier nœud ; types : 1 PCI Range (longueur 24 : endpoint start @4, segments @8/@10, BDF @12/@14, offset suivant @16), 2 MMIO Endpoint (24 : endpoint ID @4, adresse @8, sortie @16), 3 virtio-pci IOMMU (16 : segment @4, BDF @6), 4 virtio-mmio IOMMU (16 : base @8). ID endpoint = ((seg - segStart) << 16) + BDF - BDFStart + endpointStart.

## 5.2.33 MISC ('MISC', 22571-22682) : table générique d'entrées GUID. En-tête standard puis entrées (GUID 16 o @0, longueur @16, révision @20, Producer ID @24 ASCII, données @28).

## 5.2.34 CCEL ('CCEL', 22683-22803) : journal d'événements confidential computing (AMD SEV = type 1, Intel TDX = type 2) ; @36 type CC ; @37 sous-type ; @40 LAML (64) ; @48 LASA (64) ; la zone du journal = LASA..LASA+LAML-1. Non pertinent agent.

## 5.2.35 SKVL ('SKVL', 22804-) : table des clés de volume de stockage chiffré ; @36 nombre de clés C ; entrées de 16 o chacune (Key Type, Length, ...). Non pertinent agent.

## SKVL suite (22871-22907)
- Clé : type @0 (2 o, 0 = clé principale), format @2 (2 o, 0 raw), taille @4 (4), adresse GPA @8 (64 ; en ACPI Reserved).

## 5.3 ACPI Namespace (22908-22974)
- Espace de noms unique pour tous les Definition Blocks ; collision à la charge = fatale.
- Noms = 4 caractères fixes ; 1er : A-Z, _ ; suivants : A-Z 0-9 _ ; complétion par '_' ; noms commençant par '_' réservés ; '\' = racine, '^' = parent.
- Recherche : nom simple = remontée jusqu'à la racine ; chemins multi-segments = absolus (pas de recherche).
- Racines : \_GPE (événements généraux), \_PR (processeurs ACPI 1.0, déprécié), \_SB (bus/périphériques), \_SI (indicateurs système), \_TZ (zones thermiques ACPI 1.0).
- Objets : global sauf locaux ; révision d'objet : OSPM ignore les champs au-delà de sa révision.

## 5.4 Définition Block Encoding (22975-23037)
- AML : flux d'octets ; PkgLength sur 1-4 octets (2 bits de tête = nombre d'octets suivants). Limites : 0x3F (1 o), 0x0FFF (2), 0x0FFFFF (3), 0xFFFFFFFF (4).
- Chargement : le premier objet doit être une méthode nommée (initialisation). Les objets non nommés hors racine ; code exécutable au niveau du Definition Block autorisé (configuration dynamique).

## 5.5 Méthodes et ASL (23038-23175)
- Objet = Type + liste fixe + liste variable. ASL déclaratif.
- Méthodes : jusqu'à 7 arguments (passage par référence constante ; ArgX non modifiable sauf via RefOf). 8 variables locales (LocalX). Objets créés dans une méthode = dynamiques (détruits en sortie) ; ceux créés au chargement = statiques.
- Non-préemptif mais peut bloquer. Accès exclusif garanti aux objets globaux tant que la méthode ne bloque pas.

## 5.5.2.4 Régions d'opération (23176-23260)
- Champs (Field) dans les OperationRegion ; les méthodes ne doivent pas accéder directement aux registres ACPI (GPEx_BLK, PM1…) hors régions autorisées.
- Table 5.182 : 0 SystemMemory, 1 SystemIO, 2 PCI_Config, 3 EmbeddedControl, 4 SMBus, 5 SystemCMOS, 6 PciBarTarget, 7 IPMI, 8 GeneralPurposeIO, 9 GenericSerialBus, 0x0A PCC, 0x0B PlatformRtMechanism, 0x7F FFixedHW (FFH), 0x80-0xFF OEM.
- SystemCMOS : PNP0B00/B01/B02 ; octets d'heure/date en lecture seule.
- PciBarTarget : OperationRegion(PBAR, PciBarTarget, 0x10, 0x4) ; uniquement pour PCI header type 0 ; un OS ne charge pas de pilote natif sur la fonction.

## 5.5.2.4.5 IPMI (23264-23376)
- HID "IPI0001", _IFT (type d'interface), région IPMI par fonction réseau (adresse = fonction<<8 | offset commande), longueur 256. Tampon 66 octets : Status (0), Length (1), Data (2..65). Codes de statut : 00h OK, 07h erreur inconnue, 10h timeout. Non pertinent agent.

## 5.5.2.4.6 GeneralPurposeIO (23454-23469)
- RegionSpace 0x08 ; Offset ignoré ; Length = nombre de broches GPIO arrondi à l'octet ; déclaré dans le périphérique contrôleur GPIO.

## GPIO / GenericSerialBus (23471-24070) — résumé
- GPIO fields : ByteAcc, NoLock sur HW-reduced ; Connection GpioIo(...) par broche ; _DEP pour la dépendance au contrôleur.
- GenericSerialBus (région 0x09, longueur 0x100 commandes) : champs BufferAcc avec AccessAs(protocole) : AttribQuick 0x02, SendReceive 0x04, Byte 0x06, Word 0x08, Block 0x0A, Bytes 0x0B, ProcessCall 0x0C, BlockProcessCall 0x0D, RawBytes 0x0E, RawProcessBytes 0x0F. Tampon : Status @0, Length @1, Data @2. Connexion par ressource I2C/SPI/UART (I2CSerialBusV2 etc.).
- Non pertinent agent (périphériques I2C/SPI via AML).

## Suite 5.5.2.4.7 - 5.5.2.4.8 (24071-24570) — résumé
- SPB : BLOCK PROCESS CALL, BytesRaw… : tampons de longueur variable ; Raw ignore l'offset de commande.
- PCC OperationRegion (région 0x0A) : OperationRegion(NOM, PCC, subspace-id, longueur) ; champs ByteAcc/NoLock ; ordre d'écriture : payload puis CMD en dernier (le déclenchement se fait sur écriture de CMD). Types de sous-canal 0-2 (CMD 16, STAT 16, DATA), type 3 (FLGS, LEN, CMD 32, DATA). _REG Arg0 = 0x030A pour signaler le support du type 3.
- Exemple : PMT0 (ACPI000D, mesure de puissance).

## 5.6 Modèle d'événements ACPI (24572-24670+)
- Composants : OSPM, FADT (blocs fixes), PM1x_STS/EN, GPE0/1_BLK, SCI (level, partageable), modèle GPE AML, périphériques événementiels, EC.
- Événements fixes (Table 5.186) : timer carry (TMR_STS), bouton power, bouton sleep, RTC alarm, wake status, requête bus master ... Un bouton peut être un périphérique ACPI (méthode) au lieu d'un bit fixe.
- Les GPE (jusqu'à 256 bits STS/EN) : événements généraux, méthodes _Exx/_Lxx/_Qxx.

## 5.6.4 GPE (24677-24760) — points clés
- OSPM : désactive la source, efface le statut (edge), dispatch, réactive. Registres GPEx_STS : latching, RW1C ; EN : RW.
- Méthodes : \_GPE._Exx (edge) / _Lxx (level) avec xx = index GPE (GPE0 : 0 .. GPE0_BLK_LEN/2-1 ; GPE1 : GPE1_BASE ..). _Qxx = requête EC ou alarme SMBus (adresse 7 bits).
- _Wxx : déterminer la source de réveil après _SWS.
- Réveil : PNP0C0C/0D/0E et PNP0A03 peuvent partager un GPE (wake + runtime) ; sinon bits séparés.
- _PRW : index du bit GPE de réveil ; _PSW pour activer.

## 5.6.5 GPIO-signalés (24760-24847)
- _AEI (sous le contrôleur GPIO) : GpioInt par pin ; méthode _EVT(Arg0 = numéro de pin) pour les événements >255 ; _Exx/_Lxx prioritaires pour <255.

## 5.6.6 Notify (24848-25005) — valeurs communes
- 0 Bus Check, 1 Device Check, 2 Device Wake, 3 Eject Request, 4 Device Check Light, 5 Frequency Mismatch, 6 Bus Mode Mismatch, 7 Power Fault, 8 Capabilities Check (_OSC), 9 Device _PLD Check, 0xB System Locality Info Update, 0xD System Resource Affinity Update, 0xE Heterogeneous Memory Update, 0xF Error Disconnect Recover ; 0x80 Graceful Shutdown (sur \_SB).
- Spécifiques : Battery 0x80-0x82 ; Thermal Zone 0x80-0x83 ; Power Button 0x80 ; Sleep Button 0x80 ; Lid 0x80 ; Processor 0x80 Performance, 0x81 C States, 0x82 Throttling, 0x83 CPPC Guaranteed, 0x84 Min Excursion, 0x85 Highest Perf ; Power Meter 0x80-0x85 ; Fan 0x80 Low speed ; NVDIMM ; ALS ; UPD 0x80 ; Memory 0x80-0x81 bande passante.
- Agent : seul Device Check/Bus Check pertinent si l'agent observe un changement USB.

## 5.6.7 Périphériques ACPI (25006-25050) — table ACPI Device IDs
- PNP0C08 core ACPI ; PNP0A05 container générique ; PNP0A06 container étendu ; PNP0C09 EC ; PNP0C0A battery ; PNP0C0B fan ; PNP0C0C power button ; PNP0C0D lid ; PNP0C0E sleep button ; PNP0C0F PCI interrupt link ; PNP0C80 memory device ; ACPI0003 power source ; ACPI0004 module ; ACPI0005 SMBus 2.0 ; ACPI0006 GPE block ; ACPI0007 processor (Device) ; ACPI0008 ALS ; ACPI0009 IOxAPIC ; ACPI000A I/O APIC ; ACPI000B I/O SAPIC ; ACPI000C processor aggregator ; ACPI000D power meter ; ACPI000E time/alarm ; ACPI000F user presence ; ACPI0010 processor container ; ACPI0011 generic buttons ; ACPI0012 NVDIMM root ; ACPI0013 GED ; ACPI0015 ... ; ACPI0018 ... (CXL, audio, USB4 selon table).

## 5.6.8 Noms prédéfinis (25051-25165) — index (noms les plus utiles pour l'agent)
- _ADR, _HID, _CID, _CLS, _UID, _STA, _CRS/_PRS/_SRS (ressources), _PLD, _UPC (ports USB), _SUB, _HRV, _DSD (données de périphérique), _DSM, _OSC, _OSI, _DEP, _PRT (routage PCI IRQ), _HPP/_HPX (hot-plug PCI), _MAT, _PXM (proximité NUMA), _SLI, _HMA, _CCA, _DCK, _EJx, _LCK, _OST, _RST, _PRW, _REG, _REV, _SWS, _Wxx, _CDM, _CPC, _CST, _PSS, _PCT, _TSS, _TSD etc.
- _OSI : up-call du firmware vers l'OS ; _REV = révision de la spécification implémentée ; _OS = nom de l'OS.

## 5.7 Objets prédéfinis (25247-25270)
- \_GL (mutex Global Lock), \_OS (chaîne nom OS), \_OSI (up-call, réponse de l'OS), \_REV.
- \_GL : acquérir ce mutex acquiert aussi le Global Lock de FACS.

## 6.1.1-6.1.7 (25422-25660) — identification
- _ADR : adresse sur bus parent ; ADR valide seulement dans Augmented Device Descriptor ; table d'encodages (EISA slot, floppy, I3C, IDE, HDA codec/nœud, PCI device.function, USB port, SATA port, SDIO, NVDIMM handle).
- _CID : liste ID compatibles (ordre de préférence) ; formats PCI\CC_ccss, PCI\VEN_vvvv&DEV_dddd... ; _CID seulement avec _HID.
- _CLS : Package(base, sous-classe, interface de programmation) — ex. 01/06/01 = SATA AHCI.
- _DDN : nom DOS (COM1). _HID : PNP "AAA####" ou ACPI "NNNN####" ; "ACPI" réservé, 4 hex = IDs PCI. Un objet device a _HID ou _ADR, jamais les deux.
- _HRV : entier de révision matérielle. _MLS : package de (langue RFC3066, Unicode).
- Contrôle : méthodes _HID/_ADR ... doivent fonctionner sans opregion ; retourner la valeur de boot si indisponibles.

## 6.1.8 _PLD (25660-26020)
- Retourne un Package de buffers ; buffer 0 = 2 DWORD (révision 2) : bit0 ignore color, couleur RGB 24 bits (bits 8..31), largeur (mm, 16 bits), hauteur (16), User Visible (bit 64? : bit "User Visible"), Dock, Lid, Panel (0 Top, 1 Bottom, 2 Left, 3 Right, 4 Front, 5 Back, 6 Unknown), Vertical Position (0 haut, 1 centre, 2 bas), Horizontal Position (0 gauche, 1 centre, 2 droite), Shape (0 round, 1 oval, 2 square, 3 vertical rect, 4 horizontal rect, 5/6 trapèze, 7 unknown, 8 chamfered), Group Orientation, Group Token (16 bits), Group Position, Bay, Ejectable, OSPM Ejection Required, Cabinet Number, Card Cage Number, Reference, Rotation (0-7 pas de 45°).
- Notify 0x09 (Device _PLD Check) = réévaluer _PLD si la connexion change (dock/replicateur).
- Note agent : _PLD décrit la position physique (connecteurs), utile pour associer un port xHCI à un connecteur visible (complète _UPC).

## 6.1.8 _PLD suite (26021-26170) — Order (bits 18?/Order 3 bits), Vertical/Horizontal Offset (0.1 mm, 0xFFFFFFFF = absent) ; buffers 1..N = paires {GUID, données} OEM. Exemple "PLD Back Panel" (C1, C2 = USB ports).

## PLAN DE LECTURE RESTANT (fait le 2026-11-12)
- Fait : ACPI 1.-6.1.8 partiel, lignes 1 à 26170 sur 57 717 (~45 %).
- Reste ACPI : fin 6.1.8 (26170) ; 6.2 _CDM/_CRS/_DSD/_HPX/_OSC (≈ 26400-27900) ; 6.3 _EJx/_OST ; 6.4 ressources (IRQ, I/O, Memory, Extended Interrupt, GPIO/SPB) ; 6.5 divers (_DEP, _FIT, _CBR) ; 7 alimentation (_PSx, _Sx, _WAK, _PTS) ; 8 processeurs (C-states, _CST, _LPI, P-states _PSS/_CPC, _TSS) ; 9 périphériques (_UPC §9.12 ~ 569 p., _DSM, GED, RTC/TAD) ; 10 batteries/alim ; 11 thermique ; 12 EC ; annexes.
- Documents utiles trouvés (<DEPOT_LOCAL>\docs) :
  - specs : UEFI_Spec_Final_2_11.txt (+ UEFI_PI_Spec), USB2_spec_et_ECN.txt, USB32_spec_et_ECN.txt, NVMe_PCIe_Transport_1.4.txt, AMD_56569_PPR.txt, APM vol1/2/3, HDA_spec_codex.txt.
  - projet : UEFI_PILE.md, RAPPORT_*.md, amd_verifie.json, INSTRUCTIONS_CPU.md, README.md.
  - telechargements : PHYSICAL-BOOT-RESULT.txt, physical-amd-hil-evidence-20260925.zip (inventaire matériel hardware-inventory.json), usb_20/usb_32 zips.
  - Index : docs\INDEX.md.
- Pertinence agent : ACPI (tables + _UPC/_PLD + _CRS ressources), UEFI 2.11 (GetMemoryMap, ConfigTable, services), USB2/USB3.2 (ports, PSI), PPR (processeur).

## Suite 6.1.8 - 6.2.6 (26171-26770) — résumé
- _PLD (suite) : exemple de rendu (C1..C16, PCI 1-7, SATA, 1394, coax) ; origine = coin bas-gauche du panneau ; offsets positifs = droite/haut.
- _SUB : chaîne PNP/ACPI ID (ex. "MSFT3000"). _STR : chaîne Unicode descriptive. _SUN : numéro unique par type d'emplacement. _UID : entier ou chaîne, unique parmi les mêmes _HID/_CID ; entier obligatoire pour ACPI0007 (lien avec MADT/PPTT).
- 6.2 : objets de configuration. _PRS (ressources possibles), _CRS (ressources courantes, obligatoire, buffer de descripteurs ; désactivé → buffer valide mais ignoré), _SRS (écriture), _DIS (désactive, bit Enabled de _STA = 0), _DMA (fenêtres DMA côté enfant d'un bus), _PRT (routage PCI), _PXM, _SLI, _CDM (domaine d'horloge), _CCA (cohérence cache), _DSD (données par UUID ; un UUID ne doit pas apparaître deux fois), _FIX (PNP0C20 SMI_CMD, PNP0C21/22 PM1a/b EVT, PNP0C23/24 PM1a/b CNT, PNP0C25 PM2_CNT, PNP0C26 PM_TMR, PNP0C27/28 GPE0/1, PNP0B00/01/02 FIXED_RTC).
- _DMA : sur un bus, décrit les plages utilisables par les bus-masters enfants. À relire si l'agent configure un contrôleur PCI.

## Suite 6.2.7 - 6.2.9.4 (26771-27370) — résumé
- _GSB : base GSI d'un I/O APIC (64 bits), obligatoire si l'I/O APIC n'est pas dans la MADT ou est hot-plug ; évalué avant _MAT.
- _HPP (table 6.7) : cache-line (DWORDs), latency timer (cycles PCI), SERR, PERR ; applique à tous les bus enfants jusqu'au prochain _HPP. Remplacé par _HPX si les deux existent.
- Exemples Hot Plug (P2P2, slots S1/S2, méthodes _EJ0) — lecture seule.
- _HPX (6.2.9) : Package de records : Type 0 PCI (rév 1 : cacheline, latency, SERR, PERR), Type 1 PCI-X (max memory read byte count, outstanding split transactions), Type 2 PCIe AER (masques AND/OR par registre : uncorrectable mask/severity, correctable mask, ACS/AER control, device control, link control, secondary uncorrectable severity/mask), Type 3 PCIe Descriptor (liste de Register Descriptors : Device/Port Type mask — bit0 Endpoint, 1 Legacy EP, 2 RCiEP, 3 RCEC, 4 Root Port, 5 Switch upstream, 6 Switch downstream, 7 PCIe→PCI bridge, 8 PCI→PCIe bridge ; Function Type mask — bit0 non-SR-IOV, 1 PF, 2 VF ; Configuration Space Location 0-4 ; Match Register Offset/Value et Write Register Offset/Masks).
- Algorithme : lire valeur, AND masque, OR masque, écrire. RsvdP = AND 1 / OR 0.
- Agent : utile seulement si l'agent reconfigure des périphériques PCI hot-plug (non prévu). Rappel : OSPM peut écraser ces valeurs si l'OS a pris AER via _OSC.

## Suite 6.2.9.4 - 6.2.13.1 (27371-27970) — résumé
- Type 3 (suite) : Version (bit4 = "≥", bits 3:0 = version ; PM : 3 bits), PCIe Vendor ID, VSEC/DVSEC ID et Rev, masques Match et Write (AND/OR), offset alignés DWORD.
- Exemple _HPX (CTO timeouts, AER) — lecture seule.
- _MAT (6.2.10) : buffer d'entrées MADT, sous un I/O APIC ou un processeur ; seuls les types marqués « yes » du tableau 5.21 sont traités.
- _OSC (6.2.11) : Arg0 UUID, Arg1 révision, Arg2 nombre de DWORD, Arg3 buffer. DWORD 1 : bit0 Query, bits 1-3 à 0 ; retour : bit1 échec, bit2 UUID inconnu, bit3 révision inconnue, bit4 capacités masquées. Règle : l'OS ne choisit pas son pilote avec _OSC (utiliser _HID/_CID).
  - Évaluer _OSC à l'init du pilote natif, après S4 ; le firmware ne peut pas retirer une capacité accordée.
- _OSC plateforme (UUID 0811B06E-4A27-44F9-8D60-3CBBC22E7B48, rév. 1, 2 DWORD) : DWORD 2 bits : 0 Processor Aggregator, 1 _PPC _OST, 2 _PR3, 3 insertion/éjection _OST, 4 APEI, 5 CPPC, 6 CPPC 2, 7 Low Power Idle plateforme, 8 Low Power Idle OS, 9 _TFP, 10 >16 P-states, 11 GED, 12 Diverse CPPC, 13 ResourceSource, 14 CPPC Flex addr. space, 15 GHES_ASSIST, 16 multi-PCC CPPC, 17 Generic Initiator, 18 USB4 natif, 19 Battery Charge Limiting, 20 PCI BAR Target GAS, 21 PRM, 22 FFixedHW.
- _OSC USB4 (UUID 23A0D13A-26AB-486C-9C5F-0FFA525A575A, 3 DWORD) : Support 5:0 version USB4 (0 = 1.0), Control : bit0 USB tunneling, bit1 DisplayPort tunneling, bit2 PCIe tunneling, bit3 inter-domain USB4.
- _PRS (6.2.12) : possibilités de ressources ; ne doit pas accéder à une région non déclarée par _REG.
- _PRT (6.2.13) : mapping pin PCI → source (PCI Interrupt Link via _PRS/_CRS/_SRS/_DIS, ou GSI fixe si Source = 0). Adresse 0xFFFF = toutes les fonctions. Exemple avec LNKA..LNKD (PNP0C0F).
- Agent : la PRT et le routage PCI deviennent utiles si l'agent cartographie les IRQ des contrôleurs xHCI et NVMe.

## Suite 6.2.13.1 - 6.3.2 (27971-28570) — résumé
- _PRT exemple (LNKA..LNKD, slots 1-2, vidéo) ; adresse 0x0004FFFF = slot 1 toutes fonctions.
- _PXM (6.2.14) : entier = domaine de proximité (SRAT) ; enfants héritent sauf _PXM propre. Pas d'hypothèse de distance entre domaines.
- _SLI (6.2.15) : buffer de distances pour une localité (mise à jour runtime) ; (i,i) = 10. Exemple 4 nœuds (matrice 10/15/20/18…) et 5 nœuds.
- _SRS (6.2.16) : écrit les ressources avec le même format et la même longueur que _CRS ; prend effet avant retour ; ne désactive pas (utiliser _DIS). Pas de région non déclarée via _REG.
- _CCA (6.2.17) : 1 = cohérence cache matérielle, 0 = non (OS doit flusher) ; hérité par les descendants ; sur Intel si absent = cohérent. Exemple XHCI/PCI0/SDHC/DMAC.
- _HMA (6.2.18) : renvoie tout le HMAT (remplace).
- 6.3 Insertion/éjection : étapes Notify 0 (Bus Check), 1 (Device Check), 3 (Eject Request) ; _LCK, _EJx (hot _EJ0, warm _EJ1..4 max 2), _STA après éjection (3 = présent mais défaillant).
- _EDL (6.3.1) : liste des périphériques dépendants à préparer avant éjection (dock). _EJD (6.3.2) : nom du périphérique dont dépend celui-ci ; évalué une fois au chargement.
- Agent : rien à faire en éjection ; _CCA utile pour savoir si les contrôleurs DMA (xHCI, NVMe) sont cohérents.

## Suite 6.3.2 - 6.4.2.7 (28571-29170) — résumé
- _EJD / _EDL : exemple DOCK1/DOCK2 (CB2, IDE2) ; ACPI 2.0+ utilise _EDL.
- _EJx (6.3.3) : arg 1 = éjection à chaud (EJ0 : éjection immédiate, retour après fin) ; EJ1..EJ4 = préparation pour Sx, éjection après SLP_EN. Peut avoir EJ0 + EJ4.
- _LCK (6.3.4) : arg 1 verrouille, 0 déverrouille. Un périphérique a soit _LCK soit _EJx.
- _OST (6.3.5) : arg0 = source event (≤0xFF = valeur Notify ; 0x100 arrêt OS ; 0x103 éjection ; 0x200 insertion ; 0xFFFFFFFF inconnu), arg1 = code de statut (0 succès, 1 échec, 2 non reconnu ; 0x80 refusé, 0x81 en cours (heartbeat toutes les 10 s, timeout 60 s), 0x82 terminé, 0x83 non supporté ; 0x80-0x84 pour éjection, 0x84 en attente ; 0x80-0x83 insertion/ressources). Error Disconnect Recover : 0x80 succès, 0x81 non récupéré.
- _RMV (6.3.6) : 1 = retirable sans action ACPI.
- _STA (6.3.7) : bit0 présent, bit1 activé et décode ses ressources, bit2 affiché UI, bit3 fonctionnel, bit4 batterie (seulement PNP0C0A). Présent sur bus énumérable ⇒ _STA ≠ 0. Sans _STA = toutes les bits à 1. Lecture seule.
- 6.4 Ressources : tag small item = 1 octet : bit7 = 0, bits 6:3 nom, bits 2:0 longueur.
  - 6.4.2.1 IRQ (0x22/0x23, mask 16 bits, byte 3 optionnel : bit5 wake, bit4 shared, bit3 polarité bas, bit0 mode edge). Par défaut edge haut exclusif.
  - 6.4.2.2 DMA (0x2A : mask canaux 0-7, byte 2 : type, bus-master, taille).
  - 6.4.2.3 Start Dependent Function (0x30/0x31, priorités compatibilité bits 1:0, performance bits 3:2) ; 6.4.2.4 End Dependent (0x38).
  - 6.4.2.5 I/O Port (0x47, _DEC bit0 16-bit, _MIN, _MAX, _ALN, _LEN).
  - 6.4.2.6 Fixed I/O (0x4B, 10 bits, _BAS, _LEN).
  - 6.4.2.7 Fixed DMA (0x4A, 5 octets, canal/requête, largeur bus ; _CRS seulement).
- Agent : utile pour décoder les _CRS des périphériques (xHCI, NVMe, HDA).

## Suite 6.4.2.8 - 6.4.3.5.2 (29171-29770) — résumé
- 6.4.2.8 Vendor Short (0x71-0x77). 6.4.2.9 End Tag (0x79 + checksum).
- 6.4.3 Large : tag bit7=1, nom bits 6:0, longueur 2 octets.
  - 6.4.3.1 Memory24 (0x81, longueur 9) : _RW bit0, _MIN/_MAX (bits 23:8), _ALN (0 = 64 KB), _LEN en blocs de 256 o. Pas mélangé avec 32 bits.
  - 6.4.3.2 Vendor Long (0x84) : UUID + vendor data.
  - 6.4.3.3 Memory32 (0x85, longueur 17) : _RW, _MIN, _MAX 32 bits, _ALN, _LEN (octets).
  - 6.4.3.4 Memory32Fixed (0x86, longueur 9) : _RW, _BAS, _LEN.
  - 6.4.3.5 Address Space (WORD 0x88, DWORD 0x87, QWORD 0x8A, Extended 0x8B) : type ressource 0 mémoire, 1 I/O, 2 bus, 192-255 vendeur ; flags _MAF bit3, _MIF bit2, _DEC bit1 ; _GRA (granularité, puissance de 2 moins 1), _MIN, _MAX, _TRA (offset de traduction, 0 pour enfants), _LEN, Resource Source (optionnel).
  - Tableau 6.44 : combinaisons valides _LEN/_MIF/_MAF/_GRA pour _PRS (variable/fixe, emplacement variable/fixe).
- Agent : décodeur de _CRS = lire tag, longueur, puis champs ci-dessus (QWORD IO/MEM pour MCFG, xHCI BAR, NVMe BAR).

## Suite 6.4.3.5.2 - 6.4.3.8.2.1 (29771-30370) — résumé
- WORD (0x88, longueur ≥13, champs 16 bits) ; EXTENDED (0x8B, longueur 53, _TRA 64 bits, _ATT attribut mémoire UEFI (table GetMemoryMap) ; flags : bit0 consommateur/producteur, Revision ID = 1).
- Mémoire flags (Table 6.49) : bits 5 _TTP, 4:3 _MTP (0 Memory, 1 Reserved, 2 ACPI, 3 NVS), 2:1 _MEM (0 non-cacheable, 3 prefetchable), 0 _RW. I/O flags (6.50) : _TRS sparse, _TTP, _RNG. Bus number (6.51).
- 6.4.3.6 Extended Interrupt (0x89) : longueur variable ; byte 3 flags (bit4 wake, bit3 shared, bit2 polarité, bit1 mode, bit0 consommateur) ; nombre d'INT (=1 dans _CRS/_SRS) ; GSI sur 32 bits ; ResourceSource optionnel. Sur 8259 : seules les GSI 0-15.
- 6.4.3.7 Generic Register (0x82, longueur 12) : _ASI (espace : 0 mémoire, 1 I/O, 2 PCI config, 3 EC, 4 SMBus, 5 CMOS, 6 PciBarTarget, 7 IPMI, 8 GPIO, 9 SPB, 0xA PCC, 0x7F FFH), _RBW (largeur bits), _RBO (décalage bits), _ASZ (0 indéf., 1 octet, 2 mot, 3 dword, 4 qword), _ADR 64 bits.
- 6.4.3.8.1 GPIO Connection (0x8C, longueur ≥ 22 + nom) : revision 1 ; type 0 Interrupt, 1 IO ; flags ; Interrupt flags (wake, shared, polarité 0 haut/1 bas/2 both, mode level/edge) ; IO flags (_SHR, _IOR 0-3) ; _PPI (pull) ; _DRS, _DBT ; table de pins (offsets) ; _PIN (0xFFFF = aucun) ; nom de contrôleur ; données vendeur _VEN.
- 6.4.3.8.2 GenericSerialBus (0x8E, revision 2) : type 1 I2C, 2 SPI, 3 UART, 4 CSI-2, 0xC0+ vendeur ; flags (bit0 esclave/maître, bit1 consommateur, bit2 partagé) ; données spécifiques (I2C : adresse, vitesse...). Non pertinent agent (périphériques SPB).

## Suite 6.4.3.8.2.1 - 6.4.3.11 (30371-30970) — résumé (périphériques SPB et GPIO)
- I2C (revision 2) : flags, _MOD (10 bits), _SPE (vitesse Hz 32 bits), _ADR (7 ou 10 bits), _LVR (I3C), Master instance.
- SPI (revision 1) : _DPL polarité sélection, _MOD 3/4 fils, _LEN (bits par transfert), _PHA, _POL, _ADR (sélection puce), _SPE.
- UART (revision 1) : _END endianness, _LEN 5..9 bits, _STB stop, _FLC contrôle de flux, _SPE, _RXL/_TXL (FIFO), _PAR, _LIN (RTS/CTS/DTR/DSR/RI/DCD).
- CSI-2 (revision 1) : instance de port distant, _PRT, _PHY (0 C-PHY, 1 D-PHY).
- Pin Function (0x8D) : revision 1, _SHR, pull, numéro de fonction, table de pins, source, données vendeur, _PIN 16 bits.
- Pin Configuration (0x8F) : revision 1, _TYP (type), _VAL 32 bits, table, source, _PIN.
- Pin Group (0x90) : revision 1, consommateur/producteur ; table de pins ; nom du groupe.
- Agent : non pertinent pour le boot ; utile si l'agent décode les périphériques GPIO/I2C (tactile, clavier) plus tard.

## Suite 6.4.3.11 - 6.5.6 (30971-31570) — résumé
- Pin Group Function (0x91), Pin Group Configuration (0x92), Clock Input (0x93 : _FQD diviseur ≠ 0, _FQN numérateur 32 bits, échelle bits 3:1 Hz/kHz/MHz, bit0 variable/fixe, Clock Source) — non pertinent agent.
- 6.5 Autres objets : _BBN (bus de base du host bridge PCI, bits 7:0), _SEG (segment PCI, bits 15:0 ; défaut 0), _BDN, _DCK (1 = dock, 0 = undock), _INI, _REG (Arg0 = espace + qualifieur, Arg1 1 connecter / 0 déconnecter), _DEP, _FIT, _GLK, _LSI/_LSR/_LSW (NVDIMM).
- _INI (6.5.1) : table 6.67 : present=0 → ne pas exécuter, parcourir enfants seulement si fonctionnel ; present=1 → exécuter _INI puis enfants.
- _REG : les régions PCI_Config (bus avec _BBN), SystemIO et SystemMemory (System Address Map) sont toujours accessibles ; EC avant tout méthode via ECDT.
- Agent : _BBN et _SEG utiles pour les xHCI/NVMe multi-racines ; _REG et _INI à considérer si l'agent exécute l'AML.

## Suite 6.5.7 - 7.3.13 (31571-32170) — résumé
- _GLK (6.5.7) : 1 = acquérir le Global Lock pour accéder au périphérique (pour cohabiter avec le code SMM). Absent = non requis.
- _DEP (6.5.8) : ordre de démarrage (SPB), indicatif seulement.
- _FIT (6.5.9) : buffer de structures NFIT (NVDIMM, hot plug).
- _LSI/_LSR/_LSW (6.5.10) : labels NVDIMM (Status, taille, max transfert ; offsets/longueurs ; codes 0 OK, 1 échec, 2 params invalides, 3 verrouillé, 4 HW).
- _CBR (6.5.11) : registres host bridge CXL (version 0 = CXL 1.1 RCRB 8 KB, 1 = CXL 2.0 CHBCR 64 KB).
- 7.1 : table des fonctions de gestion d'énergie (Table 7.1) ; ressources d'alimentation, états système S0-S5, _PTS/_TTS/_WAK, _SLP_TYP/SLP_EN (PM1 Control) si _S5 existe.
- 7.2 PowerResource(nom, systemlevel, resourceorder) : _ON, _OFF, _STA (0/1), _RST. Ordre : niveaux croissants à l'activation, décroissants à la désactivation. Passive power resources (sans _ON/_OFF/_STA) pour les LPI (voir _RDI).
- 7.3 Objets périphériques : _PS0..3, _PSC (0 D0, 1 D1, 2 D2, 3 D3), _PSE, _PR0..3 (paquets de références), _PRE (enumeration D3cold), _PRW (wake : EventInfo = entier FADT GPE, paquet bloc GPE, paquet GPIO _AEI, ou paquet GED _CRS ; plus liste de ressources), _DSW (3 arguments : enable, état système, état D : 0 D0 ; 1 D0/D1 ; 2 ... ; 3 ...), _PSW (déprécié), _IRC, _S0D..._S4D, _S0W..._S4W, _RST, _PRR, _DSC.
- Règles : un périphérique ne peut être plus haut en D-state que son parent ; _PS0/_PR0 et _PS3/_PR3 doivent aller ensemble ; si _PSx et _PRx coexistent, mêmes états.
- Agent : utile pour savoir si un contrôleur (xHCI, NVMe) se met en D3 ; l'agent ne doit pas exécuter ces méthodes sans besoin.

## Suite 32171-32770 — résumé
- _PRW : retourne Package { EventInfo, DeepestSleepState, PowerResource[0..n] }. EventInfo = entier (bit index dans GPE enable FADT) ou Package { Device ref, Index } (GPE block device, GPIO _AEI, GED _CRS). Doit retourner la même donnée à chaque évaluation.
- Séquence de wake : (1) mettre ON les power resources ; (2) exécuter _DSW ; (3) interruptions désactivées ; (4) état de sommeil <= DeepestSleepState ; (5) activer les bits GPE.
- _PSW déprécié (ACPI 3.0) : utiliser _DSW s'il existe.
- _IRC : pas d'arguments ; présence seulement = appel D0 à forte intensité d'appel ; l'OS sérialise.
- _S1D.._S4D : D-state le plus superficiel en S1..S4 ; _S0W..._S4W : D-state le plus profond permettant le wake (3=D3 ou D3hot selon _PR3, 4=D3cold si _PR3). _SxW >= _SxD.
- Tables 7.6-7.9 : combinaisons _SxD/_PRW/_SxW -> D-state résultant.
- _RST : reset fonctionnel du périphérique sans le retirer du bus (ne pas déclencher re-énumération USB). _PRR : référence à une power resource de reset implémentant _RST.
- _DSC : D-state le plus profond autorisé pendant _CRS/_PRS/_SRS sans passer en D0. Pas D3cold sauf si _PR3.
- 7.4 Méthodes système : _PTS(Sx) avant sommeil ; _Sx package : PM1a/PM1b SLP_TYP (Table 7.11) ; _TTS(Sx) avant notification drivers, et _TTS(0) à la reprise ; _WAK(Sx) après réveil (émet des Notify de vérification).
- Séquence SetSleepingState : sauver contexte, mettre resume vector FACS, effacer WAK_STS (écrire 1 pour effacer), écrire SLP_TYP puis SLP_EN ; sur HW-reduced : écrire SLEEP_CONTROL_REG puis attendre WAK_STS.
- 7.4.2.1 S0 : états D0/D1/D2/D3hot/D3 par device. S1 : CPU non exécuté, contexte gardé, caches flushés. S2 : contexte CPU perdu, DRAM gardée. S3 : idem S2 fonctionnellement. S4 : DRAM perdue (hibernation OS ou S4BIOS optionnel), tous devices D3. S5 : soft off, pas de contexte, wake events non désactivés (Remote Start), OS doit offrir désactivation des wakes sauf bouton power.
- _SWS (7.4.3) : source du réveil ; sous _GPE (index GPE), sous GPE block device (index relatif), sous _SB (index bit PM1 status) ; -1 si inconnu. Doit être persistant pendant S0. Suivi de _Wxx.
- Agent : pour un OS perso, S0 seul suffit au départ ; S3/S4 = gestion complexe, à ne pas implémenter en premier. Ne pas utiliser _WAK/_PTS dans l'agent sans besoin.

## Suite 32771-33370 — résumé
- _WAK(Sx) retourne Package { status, S-state alimentation } : bits 0x1 = échec manque d'alim, 0x2 = échec thermique.
- 7.5 ordre OSPM : _TTS(Sx) -> notif drivers -> _PTS -> écriture vecteur de sommeil -> réveil -> _WAK -> notif drivers -> _TTS(0).
- 8.1 États C : C0 exécution (throttling via P_CNT duty offset/width ; duty 0 réservé) ; C1 obligatoire (HLT) ; C2 (P_LVL2, latence dans FADT) ; C3 (P_LVL3, BM_STS, ARB_DIS, BM_RLD, flush caches ou WBINVD). C4..Cn via _CST.
- 8.2 Flush caches : WBINVD (flag FADT), FLUSH_SIZE/FLUSH_STRIDE pour flush manuel.
- 8.3 Dépendances de domaines (coordination SW_ALL 0xFC, SW_ANY 0xFD, HW_ALL 0xFE) via _CSD.
- 8.4 Processeur : device ACPI0007 sous \_SB ; _UID relie au MADT (APIC). Container ACPI0010 (CID PNP0A05), _UID unique, _LPI/_RDI/_STA optionnels.
- _CST : Package { Count, CStates[] } ; chaque CState = { Register (ResourceTemplate), Type (1,2,3), Latency µs, Power mW }. Le platform expose _CST pour tous ses CPU ou aucun. Notify 0x81 = réévaluer _CST et _CSD.
- _CSD : CStateDependency = {NumEntries=6, Revision=0, Domain, CoordType, NumProcessors, Index}.
- LPI (ACPI 6.0) : états hiérarchiques ; composite state ; _RDI contraint l'usage LPI selon les devices. Table 8.5 exemple de combinaisons valides.
- Agent : pour la première version, un seul état C1 (HLT) suffit ; _CST/_LPI plus tard. Vérifier que MADT ACPI0007 UID correspond aux LAPIC.

## Suite 33371-33970 — résumé (8.4.3 LPI)
- 8.4.3.2 Coordination d'idle : Platform Coordinated (la plateforme agrège les votes de chaque coeur pour chaque noeud ; prend l'état le moins profond voté) ou OS Initiated (OSPM ne demande un état de noeud que quand le dernier coeur s'endort ; requêtes ordonnées, champ hiérarchie obligatoire pour éviter les races). Autopromotion possible.
- _LPI (8.4.3.3) : retourne { Revision (0), LevelID (QWORD), Count, LPI[1..N] }. Chaque LPI = { MinResidency µs, WorstWakeLatency µs, Flags (bit0 enabled), ArchContextLostFlags, ResidencyCounterFrequency, EnabledParentState (1-based, 0 = pas d'état local), EntryMethod (ResourceDescriptor Register ou entier), ResidencyCounterRegister, UsageCounterRegister, StateName }.
- LPI0 = running ; EPS = index 1-based dans les _LPI du parent. Désactiver un état : bit 0 à 0 sans renuméroter.
- Si un container a _LPI, tous ses enfants doivent en avoir.
- Agent : pas nécessaire au début ; notes pour un scheduler idle futur.

## Suite 33971-34570 — résumé (8.4.3.3.3 à 8.4.3.4 _RDI)
- Ordre des LPI : du plus superficiel (haute puissance) au plus profond. WCWL = latence pire cas du réveil (addition sur niveaux). MR indépendant par niveau.
- Composition de l'entry method : Reg = EntryMethod du coeur ; pour chaque niveau parent, si Integer -> addition ; si Register -> remplace la base. Mode OS Initiated : ajouter LevelID du dernier coeur.
- Integer 0 comme entry = "autopromotable".
- Exemple FFH (Functional Fixed Hardware) : Register(FFH, 0x20, 0x00, 0x..., 0x3).
- Compteurs Residency/Usage optionnels (NULL register = SystemMemory 0,0,0,0,0).
- Wake depuis LPI : _RDI liste, par LPI, les power resources devant rester ON ; un device wake (via _PRx/_PRW) bloque les LPI qui coupent ses ressources.
- Default idle state : état le plus superficiel de chaque feuille, toujours disponible.
- _RDI seulement à la racine (container unique couvrant tous les CPU).
- Agent : aucun impact immédiat ; à garder pour le scheduler idle multi-coeur plus tard.

## Suite 34571-35170 — résumé (8.4.3.4 à 8.4.5.2)
- _RDI : Package { Revision=0, RDI[1..N] } ; RDI[n] = liste de refs aux power resources devant être OFF pour entrer le n-ième état LPI système (même index). Passive resources : compteur de référence = 0.
- OS Initiated : OSPM garantit OFF avant de demander l'état dépendant ; Platform Coordinated : la plateforme rétrograde.
- 8.4.3.5 : _CST et _LPI peuvent coexister ; OSPM préfère _LPI ; pas de mélange.
- 8.4.4 Throttling T-state : soit P_CNT (P_BLK) soit _PTC + _TSS + _TPC (les trois obligatoires ensemble). _PTC : Package {ControlRegister, StatusRegister} (Buffer resource). _TSS : liste de { Percent (1-100 ; 100 obligatoire), Power mW, Latency µs, Control value, Status value }, triée décroissante en puissance. _TPC : entier = premier état disponible ; Notify 0x82 = réévaluer. _TDL : limite de profondeur. _TSD : dépendance T-state (NumEntries=5, Rev=0, Domain, CoordType, NumProcs).
- 8.4.5 Performance (P-states) : _PCT (Control/Status registers), _PSS (liste décroissante : CoreFrequency MHz, Power mW, Latency µs, BusMasterLatency, Control, Status), _PPC (index du premier P-state permis). Tous les CPU ont le même nombre de P-states.
- Agent : pour un premier OS, ignorer T-state et P-state ; à traiter après MADT/FADT.

## Suite 35171-35770 — résumé (8.4.5.3 à 8.4.6.1 _CPC)
- _PPC : entier = premier _PSS utilisable ; Notify 0x80 = réévaluer _PPC/_PDL ; _OST(0x80, 0 succès / 1 échec) après évaluation.
- Exemple _PSS : {Freq MHz, Power mW, Latence µs, BusMasterLatency µs, Control, Status}. Triés décroissants en puissance.
- _PSD : {NumEntries=5, Rev=0, Domain, CoordType (0xFC ALL / 0xFD ANY / 0xFE HW_ALL), NumProcs}.
- _PDL : limite P-state pour contrôle thermique passif (Notify 0x80).
- 8.4.6 CPPC (Collaborative Processor Performance Control) : échelle abstraite fournie par la plateforme ; _CPC (Rev=3, NumEntries=23) : HighestPerformance, NominalPerformance, LowestNonlinearPerformance, LowestPerformance, registres GuaranteedPerformance, DesiredPerformance, Minimum/MaximumPerformance, PerformanceReductionTolerance, TimeWindow, CounterWraparoundTime, ReferencePerformanceCounter, DeliveredPerformanceCounter, PerformanceLimited, CPPCEnable, AutonomousSelectionEnable, AutonomousActivityWindow, EnergyPerformancePreference, ReferencePerformance, LowestFrequency, NominalFrequency. Champs non supportés : Integer 0 ou NULL register.
- Le CPPC passe par PCC ou registres FFH/SystemMemory.
- Agent : ignorer pour l'instant ; intéressant pour un futur governor AMD (CPPC est supporté par Ryzen).

## Suite 35771-36370 — résumé (8.4.6 CPPC détaillé, 8.4.7, 8.5, chapitre 9)
- CPPC : Highest / Nominal / Lowest Nonlinear / Lowest Performance = échelle abstraite (pas des MHz). Guaranteed Performance (Notify 0x83, >= 1 s entre changements). Desired (0 = autonomous). Minimum/Maximum (Notify 0x84 si excursion). Performance Reduction Tolerance. Time Window (ms). Counters Reference / Delivered. Performance Limited (bit0 Desired_Excursion, bit1 Minimum_Excursion, sticky, écrire 0 pour effacer). CPPC Enable. Autonomous Selection Enable. Activity Window (10 bits : 7 mantisse + 3 exposant, base 1 µs). EPP (0 perf ... 0xFF économie).
- Notifications CPPC : 0x80/0x82 remplacent PSS/TSS ; 0x85 = Highest Performance (si buffer) ; 0x83 = Guaranteed ; 0x84 = Minimum excursion.
- Les registres PCC : commandes 0x00 lecture, 0x01 écriture, 0x02-0xFF réservé. Toutes les registres PCC du même domaine dans le même subspace.
- 8.4.7 _PPE : intervalle de poll (ms) pour erreurs corrigées ; 0 = pas de poll. Complète CPEP.
- 8.5 Processor Aggregator (ACPI000C) : _PUR (Notify 0x80) = NumProcessors à idler (Revision 1). _OST : arg2 = buffer 4 octets (nombre idle effectif). Ne jamais idler le BSP.
- Chapitre 9 :
  - 9.1 _DSM : Arg0 UUID (16 octets), Arg1 Revision, Arg2 Function Index (0 = query : buffer bit 0 = autres fonctions supportées), Arg3 Package. Règle : chaque révision supporte les fonctions des révisions précédentes. Pas de collision de noms.
  - 9.2 _SI : _SST (0 off, 1 working, 2 waking, 3 sleeping S1-S3, 4 sleeping avec contexte sauvé NV), _MSG (nombre de messages), _BLT (warning, low, wake thresholds).
- Agent : _DSM est un bon modèle pour une API de plateforme à UUID.

## Suite 36371-37970 — résumé (9.2.3 _BLT à 9.12.3 USB-C)
- _BLT : seuils batterie (mWh/mAh) warning/low/wake, ignorables par la plateforme.
- 9.3 ALS (ACPI0008) : _ALI (lux ; 0 = trop bas, -1 = trop haut), _ALT (K), _ALC (chromaticité x/y en 10^-4, x dans le mot haut), _ALR (paires {ajustement % (100 = 0%), lux} croissantes), _ALP (dixièmes de s, 0 = pas de polling). Notify 0x80 (lux +10%), 0x81 (couleur), 0x82 (réponse changée).
- 9.4 Lid (PNP0C0D) : _LID (0 fermé, non-zéro ouvert) ; Notify 0x80 au changement.
- 9.5 Boutons power/sleep : Notify 0x80 = bouton en S0 ; Notify 0x02 = réveil après que l'OS est réveillé (ne pas envoyer 0x80 dans ce cas).
- 9.6 Generic container PNP0A05/PNP0A06.
- 9.7 ATA : _GTF (56 bits par commande ; reg 1F1..1F7), _GTM/_STM (IDE, timings ns, flags UltraDMA), _SDD (SATA identify). Séquence : _SDD avant _GTF sur SATA.
- 9.8 Floppy : _FDE (4 drives + tape presence), _FDI, _FDM.
- 9.9 GPE block device ACPI0006 : _CRS (une plage IO/mémoire contenant GPE status + enable, chacun = moitié), _Lxx _Exx _Wxx enfants ; ACPI0006 sans méthodes = bloc FADT GPE0/GPE1 (premier trouvé).
- 9.10 Module device ACPI0004 : _CRS optionnel ; obligatoire si host bridge PCI dedans. Exemple PNP0C80 (mémoire).
- 9.11 Memory device PNP0C80 : _CRS obligatoire, _PRS/_SRS/_DIS optionnels, _STA (0 absent, 0x0D désactivé, 0x0F OK). _MBM (bandwidth), _MSM, _OSC UUID 03B19910-F473-11DD-87AF-0800200C9A66.
- 9.12 _UPC USB : Package { Connectable (0/1), Type (0x00 A, 0x03 SS-A, 0x08 USB-C USB2 only, 0x09 USB-C USB2+SS avec switch, 0x0A USB-C sans switch, 0xFF propriétaire), Reserved0 (USB-C : retimer count bits 1:0, bit2 PCIe tunnel, bit3 DP AltMode, bit4 USB4, bit5 TBT3), Reserved1 }.
  - Port visible (_PLD) + connectable : utilisateur peut brancher. Non visible + connectable : câblé. Non connectable : non utilisé.
  - Exemple : _PLD Revision 2 avec couleur ignorée, user visible, panel, shape, ejectable.
  - USB 2 avec companion UHCI/OHCI : _UPC/_PLD sous le EHCI aussi.
  - 9.12.3 USB-C : _PLD identique pour tous les contrôleurs (USB, DP, PCIe) routés vers le même connecteur.
- Pertinence agent : pour détecter les ports xHCI (RHUB/PRTn, _ADR = n), _UPC indique si le port est branché ; utile pour l'énumération USB (usb_enum.rs) — à comparer au code.

## Suite 37971-39570 — résumé (9.12.3 fin à 9.19.7.6 NVDIMM)
- Exemples USB-C : PLDS (PNP0A05) contient PLD0/PLD1/PLDP ; UPCS contient UPC0/UPC1 ; ports xHC (PRT1..6) et PRP0/PRP1 (PCIe root ports USB4) référencent les mêmes _PLD/_UPC.
- 9.12.3 : _PLD identique pour toutes les voies (USB, DP, PCIe) sur le même connecteur.
- 9.13 _PDO (USB PD) : Package { Revision (0x0301 = PD 3.1), Flags (bits 2:0 preferred role ; bit3 PPS), SourcePDOList, SinkPDOList }. PDO = DWORD (encodage PD).
- 9.14 RTC/CMOS : PNP0B00 (64 octets, siècle <= 63), PNP0B01 (PIIX4, 256 octets, CENTURY FADT 0..255), PNP0B02 (Dallas). OperationRegion SystemCMOS.
- 9.15 UPD (ACPI000F) : _UPD (0 absent, 1 présent, 0xFF inconnu), _UPP (dixièmes de s), Notify 0x80.
- 9.16 I/O APIC : ACPI000A (APIC), ACPI000B (SAPIC), ACPI0009 (les deux). _GSB + _CRS (une ressource mémoire = base registres). Doit correspondre au MADT à l'amorçage, puis MADT ignoré après _CRS/_GSB.
- 9.17 Time and Alarm (ACPI000E) : _GCP (bits : 0 AC wake, 1 DC wake, 2 GRT/SRT, 3 ms accuracy, 4 _GWS S4/S5, 5..8 S4/S5 AC/DC) ; _GRT/_SRT (buffer Year WORD, Month, Day, Hour, Minute, Second, Valid, ms WORD, TimeZone WORD (minutes UTC - local, 2047 = inconnu), Daylight BYTE (bit0 affecté par DST, bit1 ajusté)) ; _STV/_TIV (secondes, 0xFFFFFFFF désactivé) ; _STP/_TIP (politique expirée : 0 instant, n s, 0xFFFFFFFF jamais) ; _GWS/_CWS (bit0 expiré, bit1 wake). Notify 0x2 au réveil. Même source de temps que UEFI.
- 9.18 Generic Buttons (ACPI0011) : _CRS = GpioInt/Interrupt edge-only ; _DSD UUID FA6BD625-9CE8-470D-A2C7-B3CA36C4282E = descripteurs HID (collection + usage page/usage). Polarité : OSC/MC/RTC/OOC selon usage ; bouton power = ActiveBoth obligatoire ; Override power 4.8.2.2.1.3.
- 9.19 NVDIMM : root ACPI0012 sous \_SB (_FIT, _STA, _DSM), NVDIMM device sous root avec _ADR = NFIT Device Handle (bit31 = format), _LSI/_LSR/_LSW (labels). Hot-plug : Notify 0x80 root, Device Check sur MEM0 (PNP0C80).
  - _DSM UUID 2f10e7a4-9e91-11e4-89d3-123b93f75cba rev 1 ; fonctions : 1 Query ARS Capabilities, 2 Start ARS, 3 Query ARS Status, 4 Clear Uncorrectable, 5 Translate SPA, 7 ARS Error Inject, etc. Notify 0x81 (uncorrectable non consommée), 0x82 (ARS stopped).
- Agent : pour un OS perso, _UPC et _PLD utiles pour connaître les ports USB ; le RTC PNP0B00 (port 0x70/0x71) est accessible directement ; Time and Alarm optionnel ; NVDIMM hors périmètre pour l'instant.

## Suite 39571-41170 — résumé (9.19.7.6 Query ARS Status à 10.2.2.1 _BCT)
- NVDIMM ARS : Query ARS Status (status ARS complete/in progress/no ARS/stopped), Start ARS (Flags bit0 priorité, bit1 récupérer données existantes), Clear Uncorrectable (SPA base alignée sur unit size), Translate SPA (SPA -> NFIT handle + DPA), ARS Error Inject (options bit0 notification, bit1 force overflow, bit2 persistent), ARS Error Inject Clear, Status Query, Capabilities. Une seule ARS à la fois dans le système.
- NVDIMM device : _NCH (santé : MAINTENANCE_NEEDED, PERFORMANCE_DEGRADED, WRITE PERSISTENCY LOSS..., ALL DATA LOSS...), _NBS (Data Loss Count monotone), _NIC/_NIH/_NIG (injection santé à des fins de test).
- Chapitre 10 Power source & battery :
  - 10.1 Smart Battery : SMBus adresses 0x8 charger, 0x9 charger+SM/selector, 0xA System Manager, 0xB batterie ; ACPI0002 avec _SBS (0..4) ; EC PNP0C09 + SMB-HC ACPI0001 avec _EC (offset/query). Alarm Notify SMBus via EC SCI.
  - 10.2 Control Method Battery PNP0C0A : Notify 0x80 (état/trip), 0x81 (info statique), 0x82 (_BMD flags), 0x83 (_BPT seuil). Battery bay toujours présente (_STA bits 0-3, bit4 présence).
  - Méthodes batterie : _BCT (temps charge), _BIF/_BIX (info), _BMA/_BMC/_BMD/_BMS (calibration/contrôle), _BPC/_BPS/_BPT (puissance), _BST (état courant), _BTH/_BTM (thermique et runtime), _BTP (trip point), _OSC, _PCL (liste devices alimentés), _STA.
  - _BCT : 0 = niveau invalide, 1..0xFFFFFFFE secondes, 0xFFFFFFFF inconnu.
- Pertinence agent : faible. Batteries et EC non prioritaires pour un OS de test ; à noter pour un driver futur (EC sur 0x62/0x66).

## Suite 41171-42770 — résumé (10.2.2.2 _BIF à 11.4.6 _CR3/_DTI)
- Batterie : _BIF (déprécié), _BIX rev1 (Cycle Count, Measurement Accuracy, sampling/averaging min/max, granularités, Battery Swapping Capability bit0 cold, bit4 hot) ; _BMA/_BMS (intervalles) ; _BMC bits (0 calibration, 1 charge off, 2 décharge sur AC, 3 suspension charge limiting) ; _BMD Status (bits 0..6) et Capabilities (bits 0..5) + Recalibrate Count/Quick/Slow Time ; _BPC (Power Threshold Support 0..3) ; _BPS (Instantaneous/Sustainable peak level + period, ms) ; _BPT (ID 0 clear, 1 instantané, 2 soutenu) ; _BST (Battery State bits 0 décharge, 1 charge, 2 critique, 3 Charge Limiting ; Present Rate ; Remaining Capacity ; Present Voltage) ; _BTH (throttle % 0-100) ; _BTM (runtime) ; _BTP (trip) ; _OSC UUID F18FC78B-0F15-4978-B793-53F833A1D35B (bit0 granularity, bit1 wake low, bit2 power threshold).
- Notify batterie : 0x80 état/trip, 0x81 info, 0x82 flags _BMD, 0x83 seuil de puissance.
- 10.3 AC adapters ACPI0003 : _PSR (0 off-line, 1 on-line), _PCL, _PIF (type redondant/partagé, Max Output/Input), _PRL (redondance). Notify 0 = Bus Check sur réévaluation _PIF.
- 10.4 Power meters ACPI000D : _PMC (capacités, unité mW, type entrée/sortie, précision, sampling, averaging min/max, hystérésis, limite HW), _PTP (trip points haut/bas), _PMM (mesure RMS), _PAI/_GAI (intervalle), _SHL/_GHL (limite matérielle), _PMD (devices mesurés, \_SB pour le système entier). Notify 0x80 (_PMC), 0x81 (_PTP), 0x82 (limite), 0x83 (limite appliquée), 0x84 (averaging).
- 10.5 WPC ACPI0014 : _WPC (0 proximité, 1 interférence, 2 changement de bande), _WPP. Notify 0x80 proximité, 0x81 Wi-Fi/BT, 0x86/0x91 autres.
- Chapitre 11 Thermal :
  - Politiques : active (_ACx + _ALx, fans), passive (_PSV + _PSL throttle CPU, algorithme ΔP = TC1*(Tn-Tn-1)+TC2*(Tn-Tt)), critique (_CRT -> arrêt immédiat ; _HOT / _CR3 -> S4 ou veille rapide).
  - Températures en dixièmes de Kelvin (pas de Celsius) : 300.0 K = 3000.
  - Notify thermal zone : 0x80 température, 0x81 seuils, 0x82 listes devices, 0x83 relations (_TRT/_ART).
  - _SCP pour choisir performance (active) vs silence (passive).
  - _TZP (polling) recommandé fortement déconseillé.
  - Fan PNP0C0B : _FIF, _FPS (Control, TripPoint, Speed rpm, NoiseLevel, Power), _FSL (Level 0-100 si fine grain sinon Control), _FST (Control, Speed). Tous requis si fan avancé. Notify 0x80 = relire _FST.
  - _ART : Revision 0 ; SourceDevice, TargetDevice, Weight, AC0..AC9MaxLevel (0xFFFFFFFF = non).
- Agent : hors périmètre immédiat (pas de thermal zone à implémenter pour l'instant). Les seuils en dixièmes de K sont utiles pour lire la température CPU via _TMP si besoin.

## Suite 42771-44370 — résumé (11.4.7 _HOT à chap. 13 SMBus)
- _HOT (critique S4), _MTL (% minimum), _NTT (seuil de notification device), _PSL, _PSV (en dixièmes de K), _RTV (absolu/relatif ; zéro = absolu), _SCP (Arg0 mode 0 active / 1 passive ; Arg1 acoustic 1..5 ; Arg2 power 1..5), _CR3 (veille basse puissance plus rapide que S4), _DTI (notif capteur device), _STR (nom de zone), _TC1/_TC2 (constantes), _TFP (ms, prime sur _TSP), _TMP, _TPT (déprécié -> _DTI), _TRT (Influence, SamplingPeriod en dixièmes de s), _TSN, _TSP (dixièmes de s), _TST, _TZD, _TZM, _TZP (0 = pas de poll).
- Exemples de zone thermique : Fan PNP0C0B, _AL0/_AL1, _PSL = CPU, _HOT/_CRT, _SCP.
- 11.6 : règles (au moins un _TMP ; _ACx -> _ALx ; _PSV -> _PSL ou _TZD ; _HOT -> S4 supporté).
- Chapitre 12 Embedded Controller (PNP0C09) :
  - Ports : données (EC_DATA) et status/command (EC_SC). Bits EC_SC : bit0 OBF, bit1 IBF, bit2 CMD, bit3 ignoré, bit4 BURST, bit5 SCI_EVT, bit6 SMI_EVT, bit7 IGN.
  - Commandes : 0x80 RD_EC (adresse), 0x81 WR_EC (adresse, donnée), 0x82 BE_EC (burst ; ack 0x90), 0x83 BD_EC, 0x84 QR_EC (retourne valeur de requête ; 0 = pas d'événement).
  - Protocole : écrire commande -> IBF=1 -> lire status ; OBF signale donnée prête.
  - Burst timeouts : 400 µs premier accès, 50 µs suivants, 1 ms total.
  - Ressources CRS : ports 0x62 (données) et 0x66 (status/command) ; _GPE = bit SCI dans GPEx_STS (ou package {GPE block device, bit}).
  - _DSM EC UUID ecc0d5e9-3ee7-4f53-8c8f-766b839dddce, fonction 1 = taille minimale burst accordée par le firmware (0 ou 0xFF réservé).
  - EC-SMB-HC : registres SMB_PRTCL(0x00 base), SMB_STS, SMB_ADDR, SMB_CMD, SMB_DATA[0..31], SMB_BCNT, SMB_ALRM_ADDR, SMB_ALRM_DATA[0..1]. Protocoles 0x02..0x0D ; bit7 = PEC.
  - Codes de statut SMB : 00h OK, 07h unknown, 10h NACK, 11h device error, 12h command denied, 13h unknown, 17h access denied, 18h timeout, 19h unsupported, 1Ah busy, 1Fh PEC error.
- Chapitre 13 SMBus : protocoles 0x02 Quick, 0x04 SendReceive, 0x06 Byte, 0x08 Word, 0x0A Block, 0x0C ProcessCall, 0x0D BlockProcessCall ; bit 7 PEC ; bit 0 = read/write selon l'opération. Adresse 7 bits non décalée dans ASL (décaler de 1 pour le registre EC). _ADR pour adresse SMBus 1.0 ; ACPI0005 = SMBus 2.0 avec énumération.
- Agent : pour le moment inutile. Le port EC 0x62/0x66 est utile si l'agent veut lire l'état d'un EC (batterie/thermique) — à éviter en écriture sans accord.

## Suite 44371-45970 — résumé (13.2.3 SMBus OperationRegion à 14.2 PCC)
- 13.2.3 OperationRegion(name, SMBus (0x04), Offset = adresse<<8 | cmd_offset, Length 0x100 - offset). Exemple : 0x0B00 = batterie 0x0B.
- 13.2.4 Field BufferAcc, LockRule (Lock si firmware accède au SMBus), AccessAs(BufferAcc, protocole) comme premier élément ; un champ = un registre virtuel 8 bits à l'offset = command.
- 13.2.5 Buffer SMBus 34 octets : Status(0), Length(1), Data(2..33). Store(champ, BUFF) déclenche la transaction ; Store(BUFF, champ) pour écriture ; double Store pour process call.
- 13.3 Exemples : Quick (0x02/0x03 sans données), Send/Receive Byte, Read/Write Byte, Word, Block (0..32 octets), Process Call, Block Process Call (max 32 o agrégé).
- Agent : rien à implémenter maintenant.

## Chapitre 14 PCC (Platform Communications Channel)
- PCCT (signature 'PCCT') : Flags bit0 = interruption plateforme possible (doit être 0 bits reserved) ; liste de sous-espaces (max 256), ID = index.
- Types de sous-espace (champ type dans chaque structure) :
  - 0 : Generic Communications (base mémoire partagée, longueur > 8, doorbell GAS, Preserve/Write masks, Nominal Latency µs, Max Periodic Access Rate cmd/min, Min Request Turnaround µs). Seul System I/O / System Memory pour le doorbell.
  - 1 : HW-Reduced (GSIV, flags level/edge, polarity), doorbell GAS avec Functional Fixed HW autorisé ; pas de level-triggered sans ack.
  - 2 : HW-Reduced avec Platform Interrupt Ack register (read-modify-write).
  - 3 : Initiator (OSPM -> plateforme) avec command complete check/update/error status registers ; 4 : Responder (plateforme -> OSPM notification, doorbell optionnel).
  - 5 : HW Registers-based (version 1, Command Complete Check mask, Error Status mask).
- Shared memory : Signature, Command, Status, Communication Space ; Generic : signature 0x50434300 ~ 'PCC' + subspace ID (vérifier table 14.9 avant utilisation).
- Relevance : CPPC (_CPC) utilise PCC ; sinon rien pour l'agent.
- Agent : hors périmètre immédiat.

## Suite 45971-47570 — résumé (14.2 PCC fin, chap. 15 System Address Map, chap. 16 Waking/Sleeping, chap. 17 NUMA, début chap. 18 APEI)
- PCC : signature = 0x50434300 | subspace ID. Command 16 bits (code 0..7, bit15 notify on completion). Status bits : 0 command complete, 1 platform interrupt, 2 error, 3 platform notification. Type 3/4 avec Command complete check/update masks, Error status mask. Protocole doorbell : vérifier command complete -> écrire message -> effacer command complete -> ring doorbell (RMW avec preserve/write mask). Accès au status : interlocked pour types 0-2.
- 15 Address map : Table 15.1 types (1 Memory, 2 Reserved, 3 ACPI Reclaim, 4 ACPI NVS, 5 Unusable, 6 Disabled, 7 Persistent, 8 Unaccepted, autres réservés). E820 = Base64 + Length64 + Type32 + Extended attributes (bit0 = 1 obligatoire, bit3 error log). Mapping UEFI -> ACPI : LoaderCode/Data, BootServicesCode/Data, ConventionalMemory -> Memory ; RuntimeServices, MMIO, PalCode, Reserved, Unusable -> Reserved ; ACPIReclaim -> ACPI ; ACPINVS -> NVS ; Persistent -> Persistent. Exemple 15.5 : 639 KB base, LFB 12 MiB, hole 8-16 MiB, APIC FEC00000/FEE00000.
  - Pour l'agent : GetMemoryMap() est à consommer avant ExitBootServices ; ensuite seule l'ACPI l'indique. Ne jamais utiliser Reserved / NVS / Persistent comme RAM.
- 16 Sleep : S1 (CPU caches flush, WBINVD), S2 (CPU reset, vecteur de reset), S3 (RAM auto-refresh, boot vector + waking vector), S4 (OS-initiated ou S4BIOS ; RAM sauvée), S5 (soft off). Séquence : _TTS -> _PTS -> écrire SLP_TYPa/b + SLP_EN -> WAK_STS.
  - Sortie : WAK_STS, _WAK, S4BIOS_F dans FACS, hardware signature FACS.
  - Initialisation : firmware réinitialise minimum CPU/MC/cache, saute au waking vector (FACS Firmware_Waking_Vector ou X_Firmware_Waking_Vector). SCI_EN doit être posé. ACPI_ENABLE via SMI_CMD si SCI_EN = 0 au démarrage.
  - 16.3.2 Memory : ACPI Reclaim (au-dessus de 8 MB), NVS (sauvegardé à S4).
  - 16.3.4 Sortie du mode ACPI : ACPI_DISABLE sur SMI_CMD.
- 17 NUMA : nodes = module devices ; SRAT (boot, Proximity Domain), _PXM (runtime), SLIT (distances, _SLI), HMAT (_HMA, latence et bande passante). Notifications 0x0D (affinité), 0xB (localités), 0xE (HMAT). Proximity Domains : valeurs compactes 0..N-1.
- 18 APEI : quatre tables ERST, BERT, HEST, EINJ. Corrected/uncorrected (fatal/non-fatal). BERT = erreurs d'un boot précédent (CPER). HEST = sources d'erreur (type 0 IA-32 MCE, type 1 IA-32 CMC, type 2 ... ).
- Agent : ne pas implémenter le sommeil à ce stade. Sur UEFI, GetMemoryMap et la table de mapping sont la référence pour la carte mémoire de l'agent.

## Suite 47571-48770 — résumé (18.3 HEST : types 0..11, GHES, PNP0C33)
- HEST types : 0 IA-32 MCE (MCA banks : MSR control/status/addr/misc), 1 IA-32 CMC, 2 NMI, 6 PCIe Root Port AER (GLOBAL flag bit1), 7 PCIe Endpoint AER, 8 PCIe/PCI-X bridge AER, 9 Generic Hardware Error Source (GHES), 10 GHESv2 (Read Ack Register : write ((X & Preserve) | Write)), 11 IA-32 Deferred MC. Types 3-5 réservés. Types >= 12 : header commun (Type 2 octets, Length 2 octets, Source ID…).
- Flags communs : bit0 FIRMWARE_FIRST, bit2 GHES_ASSIST.
- Generic Error Status Block : Block Status (bits : 0 uncorrectable valid, 1 correctable valid, 2/3 multiples, 13:4 nombre d'entrées), Raw Data Offset/Length, Data Length, Error Severity (0 recoverable, 1 fatal, 2 corrected, 3 none). Entrée = section type CPER, severity, revision 0x300, validation bits, FRU id/text, timestamp BCD.
- Notification (Table 18.14) : type 0 polled, 1 external IRQ, 2 local IRQ, 3 SCI, 4 NMI, 5 CMCI, 6 MCE, 7 GPIO-signal, 8 ARMv8 SEA, 9 SEI, 10 GSIV, 11 SDEI. Champs de seuil (poll interval, switch-to-polling threshold).
- Event notification : device PNP0C33 ; GPE _L08 ou _EVT (HW-reduced) -> Notify(error_device, 0x80).
- 18.4 Firmware First : NMI pour erreurs non corrigées ; le firmware remplit le bloc, active le bit block status, génère l'NMI.
- 18.5 ERST : Serialization Action Table ; Actions 0x0 BEGIN_WRITE, 0x1 BEGIN_READ, 0x2.. (END, EXECUTE, CLEAR, GET_ERROR_RECORD_ID...). Les enregistrements sont au format CPER.
- Agent : APEI non prioritaire. Utile seulement si le firmware publie une HEST à lire (lecture seule, pas d'écriture sans accord).

## Suite 48771-50370 — résumé (18.5 ERST complet, 18.6 EINJ)
- ERST : actions 0x0 BEGIN_WRITE, 0x1 BEGIN_READ, 0x2 BEGIN_CLEAR, 0x3 END, 0x4 SET_RECORD_OFFSET, 0x5 EXECUTE, 0x6 CHECK_BUSY, 0x7 GET_COMMAND_STATUS (00 OK, 01 pas assez de place, 02 HW absent, 03 échec, 04 store vide, 05 record absent), 0x8 GET/SET_RECORD_IDENTIFIER, 0x9 GET_RECORD_COUNT, 0xA BEGIN_DUMMY_WRITE, 0xF/0x10/0x11 error log address range et attributs, 0x12 timings.
- Instructions ERST (Table 18.21) : READ_REGISTER, READ_REGISTER_VALUE, WRITE_REGISTER(_VALUE), NOOP, LOAD/STORE VAR1/VAR2, ADD/SUBTRACT, STALL, STALL_WHILE_TRUE, SKIP_NEXT_IF_TRUE, GOTO, SET_SRC/DST_ADDRESS_BASE, MOVE_DATA. Flag PRESERVE_REGISTER (bit0).
- Séquence écriture : signature ER dans record -> BEGIN_WRITE -> SET_RECORD_OFFSET -> EXECUTE -> CHECK_BUSY jusqu'à 0 -> GET_COMMAND_STATUS -> END.
- Lecture : BEGIN_READ -> SET_RECORD_IDENTIFIER -> EXECUTE -> ... ; clear : BEGIN_CLEAR -> SET_ID (≠0) -> EXECUTE -> END.
- 18.5.2.4 : les modes NVRAM, RAM volatile + SMM/UEFI runtime, service processor, copie réseau.
- 18.6 EINJ : table de type 'EINJ'. Actions : BEGIN, GET_TRIGGER_ERROR_ACTION_TABLE, SET_ERROR_TYPE (bitmap, Table 18.30 : bits 0-2 processor CE/UCNF/F, 3-5 memory, 6-8 PCIe, 9-11 platform, 12-14 CXL.cache, 15-17 CXL.mem, bit30 EINJv2, bit31 vendor), GET_ERROR_TYPE, END, EXECUTE, CHECK_BUSY (bit0), GET_COMMAND_STATUS (0 OK, 1 unknown, 2 failure, 3 invalid access), SET_ERROR_TYPE_WITH_ADDRESS (prioritaire), GET_EXECUTE_TIMINGS, EINJV2_SET/GET_ERROR_TYPE.
- Structure SET_ERROR_TYPE_WITH_ADDRESS : Error Type, Vendor offset, Flags (bit0 processor ID, bit1 mémoire + masque, bit2 PCIe SBDF), Processor ID, Memory Address, Mask, SBDF.
- Agent : EINJ = outil de test d'erreurs ; NE PAS injecter d'erreur sans accord explicite (risque matériel). Lecture seule uniquement.

## Suite 50371-51970 — résumé (18.6.4 à chap. 19 ASL : grammaire et concepts)
- EINJ v2 : bit30 = EINJv2 ; SET_ERROR_TYPE data (Table 18.34) : Error Type, Vendor code, Flags (bit0 address, bit1 range, bit2 severity, bit3 composants), Length, Severity, Address, Range, nombre de composants. Component ID : processor = APIC ID / ACPI UID ; memory = device ID ; PCIe = SBDF.
- Trigger action table (18.6.5) : GET_TRIGGER_ERROR_ACTION_TABLE -> séquence d'instructions à exécuter ensuite pour déclencher l'erreur ; table vide = erreur consommée à l'injection.
- 18.6.6 : séquence complète BEGIN -> GET_ERROR_TYPE -> SET -> EXECUTE -> CHECK_BUSY -> GET_STATUS -> GET_TRIGGER -> execute -> END.
- 18.7 GHES_ASSIST : index = CPU * banks + bank ; GHES_ASSIST nécessite _OSC bit.
- Chap. 19 ASL :
  - Opérateurs symboliques (19.1) : Z = X + Y équivaut à Add ; X++ ; compound ; X[Y] équivaut à Index.
  - Grammaire (19.2) : Object := ObjectType FixedList VariableList ; noms 4 caractères max pour NameSeg ; ASCII ; Escape sequences ; "\_SB" pour chemins dans les chaînes.
  - Termes importants : Method, Device, Name, OperationRegion (RegionSpace : SystemIO, SystemMemory, PCI_Config, EmbeddedControl, SMBus, SystemCMOS, PciBarTarget, IPMI, GeneralPurposeIO, GenericSerialBus, PCC, PRM, FFixedHW), Field/BankField/IndexField, Mutex, Event, PowerResource, ThermalZone, Processor, Notify, Store/CopyObject, Concatenate, Match, Index, Mid, ToBuffer/ToString/ToInteger, ToUUID, ToPLD, Unicode, Printf, Fprintf.
  - Ressources : ResourceTemplate et macros (IRQ, DMA, IO, FixedIO, Memory24/32, Memory32Fixed, WordBusNumber, WordIO, DWordIO, QWordMemory, ExtendedIO, ExtendedMemory, GpioInt, GpioIO, I2CSerialBusV2, SPISerialBusV2, UARTSerialBusV2, Interrupt, Register, PinConfig, PinFunction, PinGroup, PinGroupConfig, PinGroupFunction, StartDependentFn, EndDependentFn, VendorLong/Short, DMA, FixedDMA, DWordSpace, WordSpace, QWordSpace, ExtendedSpace).
  - Conversions implicites (19.3.5) : table 19.7 ; Store vs CopyObject (19.3.5.8) : Store convertit vers le type fixe de la cible ; LocalX/ArgX sans conversion.
  - Macros : EISAID (PNP0303 -> 0x0303D041 ; encodage 5 bits par lettre), For, Printf, ToPLD, ToUUID, Unicode.
- Agent : les tables de conversion sont utiles pour un interpréteur AML futur (non prioritaire).

## Suite 51971-53570 — résumé (19.3.5.8 Store/Copy, 19.4-19.6 référence des opérateurs ASL, A à Function)
- Règles ArgX/LocalX/Named : Store vers Named applique conversion implicite au type existant ; CopyObject non. Store vers Local/Arg : pas de conversion. RefOf(ArgX) donne la référence.
- Référence opérateurs (19.6) : AccessAs, Acquire (timeout 0xFFFF = infini ; True = expiré), Add/And/Alias (SourceObject doit exister avant), ArgX, BankField (BankValue écrit avant accès), Break (sort du While/Switch), BreakPoint (debugger), Buffer (Initializer), Case, Concatenate (table 19.30/19.31 : types, "[Package]" etc.), ConcatenateResTemplate, CondRefOf (ne fatal pas), Connection (GenericSerialBus/GPIO), Continue, CopyObject (ACPI 2.0), CreateBit/Byte/Word/DWord/QWord/Field, CSI2Bus, DataTableRegion (SystemMemory, table entière), Debug (write-only), Decrement, Default, DefinitionBlock (revision<2 = int 32 bits ; >=2 = 64 bits), DerefOf, Device (exemple IDE), Divide (zéro = fatal), DMA (TypeF/TypeA/TypeB/Compatibility, Transfer8/16/8_16, BusMaster), DWordIO / DWordMemory / DWordSpace (bits _MIF _MAF _DEC _RNG _GRA _MIN _MAX _TRA _LEN _TTP _TRS ; _MEM Cacheable=1/WC=2/Prefetch=3/NonCache=0 ; _MTP Memory=0/Reserved=1/ACPI=2/NVS=3 ; _RW ReadWrite=1), EISAID, Else/ElseIf, EndDependentFn, Event, ExtendedIO/Memory/Space (64 bits), External (ObjectType, ReturnType, ParameterTypes), Fatal (Type, Code, Arg), Field (AccessType, LockRule, UpdateRule ; table 19.34 : SystemIO/SystemMemory/PCI_Config etc. → permis), FindSetLeftBit (1-based, 0 = aucun), FindSetRightBit, FixedDMA, FixedIO, Fprintf, FromBCD, Function (= Method NotSerialized).
- Pertinent pour l'agent : FixedIO/IO/Memory32Fixed pour décrire les ressources (ex : UART COM 0x3F8), DMA rarement utile ; FindSetRightBit/LeftBit utiles pour décoder des masques _CRS.
- Aucun changement de code ; notes de référence uniquement.

## Suite 53571-55170 — résumé (référence ASL 19.6 : Function à Unicode)
- Function = Method NotSerialized ; GpioInt (Edge/Level, ActiveHigh/Low/Both, Shared/Exclusive/*Wake, PinConfig, debounce en centièmes de ms, ResourceSource, pin list) ; GpioIo (drive strength en centièmes de mA, IoRestriction) ; I2CSerialBusV2 (ADR, SLV, SPE, MOD 7/10 bits, Shared) ; If/Include/Increment/Index (Index sur Buffer/String = BufferField 8 bits ; Index sur Package → DeRefOf) ; IndexField (index aligné sur AccessType) ; Interrupt (ResourceConsumer/Producer, GSIV, ResourceSource pour interruption secondaire) ; IO (Decode16/Decode10, alignement, Len 8 bits) ; IRQ (0..15) ; IRQNoFlags ; LAnd/LEqual/LGreater/LLess/LNot... ; Load/LoadTable (SSDT chargée à l'exécution, pas via "SSDT" dans la DSDT) ; Localx ; Match (MTR/MEQ/MLE/MLT/MGE/MGT) ; Memory24 (déprécié) / Memory32 / Memory32Fixed ; Method (NumArgs ≤ 7, Serialized/NotSerialized, SyncLevel 0..15) ; Mid ; Mod (zéro = fatal) ; Multiply ; Mutex ; Name ; NAnd/NOr/Not ; Notify (ObjectType thermal/processor/device) ; Offset (en octets) ; ObjectType (table 19.36 : 0 Uninit, 1 Int, 2 Str, 3 Buf, 4 Pkg, 5 FieldUnit, 6 Device, 7 Event, 8 Method, 9 Mutex, 10 OpRegion, 11 PowerRes, 12 Processor, 13 ThermalZone, 14 BufferField, 16 Debug) ; One/Ones (Ones = 0xFFFFFFFF si DSDT rev<2) ; OperationRegion (Offset/Length en octets ; région dynamique déconseillée en SystemIO/SystemMemory) ; Or ; Package (NumElements, éléments résolus compile/runtime, VarPackageOp si >255) ; PinConfig (Bias Pull-up/-down, Drive Open-Drain/Source/Push-Pull, Drive Strength mA, Slew Rate, Input Debounce, Schmitt Trigger, 0x80-0xFF vendeur) ; PinFunction ; PinGroup ; PinGroupConfig ; PinGroupFunction ; PowerResource ; Printf (Debug) ; QWordIO/QWordMemory/QWordSpace ; RawDataBuffer ; RefOf (fatal si absent) ; Register (AddressSpaceKeyword, RBW, RBO, RegisterAddress, AccessSize 0-4 : 0 undefined, 1 byte, 2 word, 3 dword, 4 qword) ; Release/Reset/Return/Revision/Scope (Scope doit viser un objet scope existant) ; ShiftLeft/ShiftRight/Signal/SizeOf (objet ref -> taille de la cible) ; Sleep (yield, ms) ; SPISerialBusV2 ; Stall (µs, max 100 µs) ; StartDependentFn (CompatPriority/PerfPriority 0 Good, 1 Acceptable, 2 Sub-optimal) / NoPri ; Store (conversion selon Destination) ; Subtract ; Switch/Case/Default (ACPI 2.0, compile en While(1)) ; ThermalZone ; Timer (100 ns) ; ToBCD/ToBuffer/ToDecimalString/ToHexString/ToInteger/ToPLD/ToString/ToUUID (format aabbccdd-eeff-gghh-iijj-kkllmmnnoopp, ordre byte inverse dans les 3 premiers groupes) ; UARTSerialBusV2 (BaudRate, Bits 5/6/7/8/9, Stop 0/1/1.5/2, LinesInUse bits RTS CTS DTR DSR RI DCD, BigEndian, Parity Space/Mark/Odd/Even/None, FlowControl HW/XON/None, RX/TX buffer sizes) ; Unicode (UTF-16 + null 16 bits).
- Pertinence agent : UARTSerialBusV2 / I2CSerialBusV2 / SPISerialBusV2 décrivent les bus d'un périphérique (utile pour un driver I2C/SPI futur). Rien à implémenter maintenant.

## Suite 55171-56770 — résumé (fin 19.6 ; chap. 20 AML ; chap. 21 TDL ; annexe A début)
- VendorLong (≤ 65533 o) / VendorShort (≤ 7 o) ; Wait (retour True = timeout) ; While ; WordBusNumber / WordIO / WordSpace (16 bits) ; XOr ; Zero ; ClockInput (FrequencyNumerator/Divisor, Scale Hz/kHz/MHz, Fixed/Variable, ResourceSource/Index).
- Chap. 20 AML : grammaire binaire. Header : Signature(4) Length(4) Revision(1) Checksum(1) OEMID(6) OEMTableID(8) OEMRev(4) CreatorID(4) CreatorRev(4). NameSeg 4 chars avec '_' de remplissage ; DualNamePrefix 0x2E ; MultiNamePrefix 0x2F + count. PkgLength : 1 à 4 octets (bits 7-6 = nb d'octets suivants ; 6 bits si un octet).
  - Opcodes clés : ZeroOp 0x00, OneOp 0x01, AliasOp 0x06, NameOp 0x08, BytePrefix 0x0A, WordPrefix 0x0B, DWordPrefix 0x0C, StringPrefix 0x0D, QWordPrefix 0x0E, ScopeOp 0x10, BufferOp 0x11, PackageOp 0x12, VarPackageOp 0x13, MethodOp 0x14, ExternalOp 0x15, LocalN 0x60-0x67, ArgN 0x68-0x6E, StoreOp 0x70, RefOfOp 0x71, AddOp 0x72, ConcatOp 0x73, SubtractOp 0x74, IncrementOp 0x75, DecrementOp 0x76, MultiplyOp 0x77, DivideOp 0x78, ShiftLeft 0x79, ShiftRight 0x7A, AndOp 0x7B, NandOp 0x7C, OrOp 0x7D, NorOp 0x7E, XorOp 0x7F, NotOp 0x80, FindSetLeftBit 0x81, FindSetRightBit 0x82, DerefOf 0x83, ConcatRes 0x84, ModOp 0x85, NotifyOp 0x86, SizeOf 0x87, IndexOp 0x88, MatchOp 0x89, CreateDWordField 0x8A, CreateWordField 0x8B, CreateByteField 0x8C, CreateBitField 0x8D, ObjectType 0x8E, CreateQWordField 0x8F, LAnd 0x90, LOr 0x91, LNot 0x92, LEqual 0x93, LGreater 0x94, LLess 0x95, ToBuffer 0x96, ToDecimalString 0x97, ToHexString 0x98, ToInteger 0x99, ToString 0x9C, CopyObject 0x9D, Mid 0x9E, Continue 0x9F, If 0xA0, Else 0xA1, While 0xA2, Noop 0xA3, Return 0xA4, Break 0xA5, BreakPoint 0xCC, One 0xFF (Ones).
  - Ext opcodes 0x5B xx : Mutex 0x01, Event 0x02, CondRefOf 0x12, CreateField 0x13, LoadTable 0x1F, Load 0x20, Stall 0x21, Sleep 0x22, Acquire 0x23, Signal 0x24, Wait 0x25, Reset 0x26, Release 0x27, FromBCD 0x28, ToBCD 0x29, Revision 0x30, Debug 0x31, Fatal 0x32, Timer 0x33 (en fait 0x5B 0x33), OpRegion 0x80, Field 0x81, Device 0x82, Processor 0x83 (déprécié), PowerRes 0x84, ThermalZone 0x85, IndexField 0x86, BankField 0x87, DataRegion 0x88.
  - FieldFlags : bits 0-3 AccessType (0 Any,1 Byte,2 Word,3 DWord,4 QWord,5 Buffer), bit 4 Lock, bits 5-6 UpdateRule.
  - OpRegion RegionSpace : 0 SystemMemory, 1 SystemIO, 2 PCI_Config, 3 EC, 4 SMBus, 5 SystemCMOS, 6 PciBarTarget, 7 IPMI, 8 GPIO, 9 GenericSerialBus, 0xA PCC, 0x80-0xFF OEM.
  - MethodFlags : bits 0-2 ArgCount, bit 3 Serialized, bits 4-7 SyncLevel.
  - Ces tables sont la référence pour un désassembleur AML (futur outil d'analyse des tables DSDT/SSDT extraites).
- Chap. 21 TDL (Table Definition Language) : format texte pour produire des tables de données (FADT, MADT, ...) ; champs `nom : valeur` ; types uint8..uint64, uint24/40/48/56, string, unicode, buffer, guid, label ($ref). Le compilateur calcule longueurs et checksums. Exemple ECDT, exemple table 'OEMZ'.
- Annexe A (device classes, début) : D0..D3 ; D3 = état hors-alimentation, contexte perdu ; A.2.1 bus ; A.2.2 DPMS (VESA) ; A.2.3 PCMCIA ; A.2.4 PCI PM (PMCSR) ; A.2.5 USB PM (SET_FEATURE, GET_STATUS, remote wakeup) ; A.3 device par défaut : D0 et D3 obligatoires. A.4 audio ; A.5 COM port (D0/D3, <1 s retour) ; A.6 display (CRT DPMS, panel interne, HDTV, GPU) : D0 obligatoire, D3 obligatoire, D1/D2 optionnels.
- Pertinence agent : appendix A utile pour décider des états D d'un contrôleur (xHCI, NVMe, HDA) à la mise en veille. Non implémenté.

## Suite 56771-57717 (fin du document) — résumé
- Annexe A (fin) : display (CRT DPMS D0/D2/D3 ; panel interne D0/D3 ; GPU D0/D3 ; D1/D2 optionnels), input (D0/D3 obligatoires), modem (D0/D2/D3 avec D2 optionnel), réseau (D0/D3), PC Card (D0/D3 + D1/D2 optionnels ; parent = max de ses enfants), stockage (D0/D3 ; D1 optionnel pour disque ; IDE channel D0/D3). Politique : D3 si inactif ; D0 à l'usage.
- Annexe B Video Extensions : _DOS (bits 1:0 mode de bascule, bit2 contrôle luminosité AC/DC), _DOD (liste ID de sorties, attributs bits 15:0 ID, 3:0 index, 7:4 port, 11:8 type 0 autre 1 VGA 2 TV 3 DVI externe 4 LCD interne), _ROM (buffer ≤ 4 KB), _GPD / _SPD / _VPO (post VGA device), _ADR (ID unique), _BCL (liste niveaux luminosité 0-100), _BCM, _BQC, _DDC (EDID 128/256/384/512), _DCS (bits 0 présent, 1 activé, 2 prêt, 3 non défectueux, 4 attaché), _DGS, _DSS (bits 0 état, 30/31 commit). Notify 0x80..0x84 (cycle sortie, changement, hotkeys), 0x85..0x89 (luminosité).
- Annexe C : contenu déprécié (_PDC -> _OSC, _GTS/_BFS, _TPT -> _DTI, Processor statement). Signatures réservées CDIT / CRAT.
- Index : fin du document.

## Statut global de la lecture ACPI 6.5
- Lecture ligne-à-ligne terminée : lignes 1 à 57 717 (100 %).
- Ce fichier contient les résumés par tranches ; les tables de référence (AML opcodes, PinConfig, EINJ, ERST, HEST, TDL) sont notées ici pour usage ultérieur.
- Points à recouper avec le code de l'agent (non faits dans cette passe) : MADT (LAPIC/x2APIC/IOAPIC), FADT (flags, PM blocks, X_ fields), _CRS (décodage des descripteurs), _UPC/_PLD (ports USB), HPET/MCFG (non lus en détail dans cette passe si applicable).
