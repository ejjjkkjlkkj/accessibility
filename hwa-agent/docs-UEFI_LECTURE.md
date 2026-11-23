# Lecture UEFI 2.11 (UEFI_Spec_Final_2_11.txt) — notes pour OS from zero

Source : <DOSSIER_SPECS>\UEFI_Spec_Final_2_11.txt (UEFI Forum, 21 nov 2024).

## Lignes 1-400 — table des matières (chapitres utiles à l'agent)
- 1 Introduction (conventions : structures, protocoles, pseudo-code, nombres).
- 2 Overview : Boot Manager, UEFI images (applications, OS loaders, drivers), Firmware Core (UEFI Services, Runtime Services), conventions d'appel par architecture (IA-32, x64 p.30, AArch32/64, RISC-V p.40, LoongArch p.43), protocoles (p.46), modèle de driver (p.50), requis (p.60).
- 3 Boot Manager : variables Boot####, Load Options (p.71), BootNext/BootOrder (3.3 p.78), Boot via Simple File Protocol (p.84), Load File Protocol / netboot (p.85).
- 4 EFI System Table : EFI_IMAGE_ENTRY_POINT (p.87), EFI_TABLE_HEADER (p.89), EFI_SYSTEM_TABLE (p.90), EFI_BOOT_SERVICES (p.92), EFI_RUNTIME_SERVICES (p.96), configuration tables (p.98 : ACPI, SMBIOS, devicetree, JSON, EFI_RT_PROPERTIES_TABLE p.101, EFI_MEMORY_ATTRIBUTES_TABLE p.102, EFI_CONFORMANCE_PROFILE_TABLE p.103).
- 5 GPT : LBA 0, MBR protectif (p.114), entrées de partitions GPT (p.119). Pertinent pour l'écriture de disque : NE PAS écrire sur clé/disque sans accord explicite.
- 6 BTT (Block Translation Table, NVDIMM) : p.121, non prioritaire.
- 7 Boot Services : événements/TPL (p.136), AllocatePages/FreePages/GetMemoryMap (p.154–156), AllocatePool (p.160), protocoles (InstallProtocolInterface p.166, OpenProtocol p.175, LocateHandleBuffer p.190, LocateProtocol p.193), images (LoadImage p.197, StartImage p.199, ExitBootServices p.203), Stall p.206, CopyMem/SetMem p.206–207, CalculateCrc32 p.209.
- 8 Runtime Services : règles (p.212), variables (GetVariable p.214, SetVariable p.218, authentification EFI_VARIABLE_AUTHENTICATION_2/3 p.224–227), temps (p.230), SetVirtualAddressMap (p.236), ResetSystem (p.239), UpdateCapsule (p.242).

## Points à retenir (provisoires, à compléter en lisant les chapitres)
- Le Boot Services de mémoire (GetMemoryMap + ExitBootServices) est la frontière : après ExitBootServices, plus aucun service de boot ; la carte mémoire doit être relue si la clé est périmée.
- Tout l'OS doit fournir sa propre gestion mémoire après ExitBootServices.
- [AGENT] Le projet hwa2_agent est un binaire UEFI : vérifier la signature d'entrée (EFI_IMAGE_ENTRY_POINT) et l'usage de ExitBootServices.

## Statut
- Lignes 1 à 400 lues (table des matières). Lecture de la suite : corps de la spec, environ 80 000 lignes au total — à continuer par blocs de 400 lignes.

## Lignes 401-800 — table des matières (suite)
- 9 Loaded Image Protocol (p.255) : EFI_LOADED_IMAGE_PROTOCOL, Unload (p.257). À utiliser pour connaître l'image chargée (ImageBase, ImageSize, DeviceHandle, FilePath).
- 10 Device Path (p.259) : nœuds PCI (p.262), Memory Mapped (p.263), ACPI _HID/_ADR (p.264–266), Messaging : ATAPI, SCSI, USB (p.270), SATA (p.271), NVMe namespace (p.288), MAC, IPv4/IPv6 ; Media : Hard Drive (p.295), File Path (p.296), PIWG firmware volume/file (p.297–298), RAM Disk (p.298). Règles de génération (p.300–302).
- 11 Driver Model : Driver Binding (Supported p.337, Start p.341, Stop p.350), Component Name (p.363), Service Binding (p.366), Driver Health (p.382). Pertinent si l'agent devient un driver UEFI.
- 12 Console : Simple Text Input Ex (p.402), Simple Text Input (p.409), Simple Text Output (p.411 : OutputString p.414, QueryMode p.417, SetCursorPosition p.421), Simple Pointer (p.423), Absolute Pointer (p.429), Serial I/O (p.433, SetAttributes p.438), GOP (p.443–450 : Blt p.450), EDID (p.452–454).
- 13 Media Access : Load File (p.458), Simple File System (p.467), File Protocol (p.469 : Open p.471, Read p.474, Write p.475, GetInfo p.483), Disk I/O (p.497), Block I/O (p.506 : ReadBlocks p.509, WriteBlocks p.510, FlushBlocks p.511), Block I/O 2 (p.512), Erase Block (p.529), ATA Pass Thru (p.531), Storage Security (p.545), NVM Express Pass Thru (p.549, PassThru p.551), SD MMC (p.558), RAM Disk (p.564), NVDIMM Label (p.568), UFS (p.583).
- 14 PCI : PCI Root Bridge I/O (p.587), protocole EFI_PCI_ROOT_BRIDGE_IO_PROTOCOL (p.591 : PollMem p.598, Mem.Read/Write p.600, Io p.602, Pci.Read/Write p.603, CopyMem p.604, Map p.606, Unmap p.607, AllocateBuffer p.608).

- [AGENT] Les protocoles Block I/O, NVM Express Pass Thru et PCI Root Bridge I/O sont les équivalents UEFI des chemins que l'agent code en bare-metal (NVMe, PCI). Ces chapitres sont à lire en détail avant toute comparaison avec usb_r0.rs / main.rs.

## Statut
- Lignes 1 à 800 lues (table des matières complète jusqu'au chapitre 14). Corps de la spec : lignes ~800 et au-delà (chapitres 2 à 14 détaillés, puis 15 et plus). Lecture détaillée non encore faite.

## Lignes 801-2300 — table des matières (suite) et listes de tables
- 14.2.16–14.2.19 (attributs, Configuration, device paths des root bridges, p.611–615) ; 14.3 PCI Driver Model (p.617) ; 14.4 EFI PCI I/O Protocol (p.623 : Mem/Io/Pci Read/Write, Map p.641, AllocateBuffer p.643, GetLocation p.646, Attributes p.647, GetBarAttributes/SetBarAttributes p.649–651, PCI Option ROMs p.653, responsabilités bus/device p.656–657, hot-plug p.659).
- 15 SCSI (p.663), 16 iSCSI (p.693), 17 USB : USB2 Host Controller Protocol (p.697 : ControlTransfer p.704, BulkTransfer p.706, SyncInterruptTransfer p.710, IsochronousTransfer p.712, GetRootHubPortStatus p.716) ; USB I/O (p.726) ; USB Function (p.744).
- 18 Debugger support (p.768), 19 Compression (p.795), 20 ACPI Table Protocol (p.814 : InstallAcpiTable p.814), 21 Unicode/Regex (p.817), 22 EBC virtual machine (p.827, ISA p.831–875, EBC protocol p.878).
- 23 Firmware update : Firmware Management Protocol (p.889, SetImage p.897), dependency expressions (p.903), capsules (p.911), ESRT (p.917), JSON capsule (p.920).
- 24 Network : SNP (p.924, Transmit p.941, Receive p.943), PXE Base Code (p.946), BIS (p.984), HTTP Boot (p.1014). 25 MNP (p.1028). 26 Bluetooth (p.1043). 27 VLAN/EAP/Wi-Fi/Supplicant (p.1107). 28 TCP/IP/IPsec/FTP/TLS (p.1169–1309). 29 ARP/DHCP/DNS/HTTP/REST (p.1310–1437). 30 UDP/MTFTP (p.1439).
- Appendices : A GUID/time (p.1970), B Console (p.1972), C Device path examples (p.1978), D Status codes (p.1984), E Universal Network Driver Interface (UNDI) (p.1987–2070), F Simple Pointer (p.2073), G Ext SCSI Pass Thru (p.2074), H/I compression source (p.2077, 2108), J EBC opcodes (p.2126), K (p.2128), L EFI 1.10 changes (p.2129), M language codes (p.2131), N CPER (p.2132–2189), O UEFI ACPI Data Table (p.2190), P HER persistence (p.2192), Q References (p.2194), R Glossary (p.2201).
- [AGENT] Le chapitre 14 (PCI Root Bridge I/O, PCI I/O) et 17 (USB2 HC) correspondent directement aux chemins bare-metal du projet ; à comparer avec src/usb_r0.rs et la lecture PCI faite précédemment.

## Statut
- Lignes 1 à 2300 lues : front matter (table des matières, listes de tables, copyright). Corps de la spec à partir de la ligne ~2300 (chapitre 1 Introduction, p.1).

## Lignes 2301-3200 — listes de tables/figures, puis historique des révisions (2.11 à 2.7A)
- Lignes 2301–2370 : listes des tables et figures (sans contenu technique propre ; utiles pour retrouver une table : par ex. Table 10.x device paths, Table 7.x services, Table 12.x console, Table N.x CPER).
- Lignes 2375–2443 : révision 2.11 (nov 2024) : Memory Attribute pour hot plug (2365), EFI_KMS DeleteKey (2371), Arm alloue les pages dans la plage 48 bits en premier (2384), EFI_RNG décrit DRBG (RNDR Arm) (2386), proxy host URI HTTP Boot (2388), CPER Arm/LoongArch/FRU memory poison, SM2/SM3 (2453), RISC-V AP-TEE (2449), dépréciation CryptoIndications (2480). Mention de Section 9.1 (EFI_LOADED_IMAGE_PROTOCOL) et 2468 (précision sur Unload()).
- Lignes 2445–2710 : historique 2.10A, 2.10, 2.9A (corrections DP NVMe EUI-64, EFI_LOAD_OPTION, REST, Inclusive Terminology §1.1), 2.9 (March 2021 : unaccepted memory type 2134, DTB config table 2129, EVENT_GROUP_BEFORE_EXIT_BOOT_SERVICES, Japanese keyboard keys 14/56).
- Lignes 2711–2900 : historique 2.8/2.8A/2.8B/2.8C et 2.7x (2019–2020) : JSON capsule (1935, 1996), REST/EX, FMP dependency expression (1942), ResetSystem ResetData (1936), DHCP optional HTTP boot (1878), ARM memory types (1897), protective MBR (1901).
- Lignes 2901–3200 : historique 2.7B / 2.7A (2017–2019) : clarifications LoadFile2 (1846), GOP RGB/BGR (1838), NVDIMM label (1817, 1829, 1830), PCI bus mastering (1821), HTTP config (1816), VLAN config (1987), FMP buffer (1869), ACPI table pointer (1906).
- [AGENT] Aucun point ici n'est à implémenter ; à retenir : la version de la spec est 2.11 (notes de conformité si l'agent produit une table « EFI_TABLE_HEADER.Revision »).

## Statut
- Lignes 1 à 3200 lues. Prochaine lecture : ligne 3201 (historique 2.7A puis, à partir de ~3500 environ, le corps du texte à vérifier avec grep « Chapter 1 »).

## Lignes 3201-4100 — historique des révisions 2.7A à 2.5A (2015–2017)
- Erratas notables : 1608 EFI_IFR_NUMERIC (Step), 1669/1675 corrections, 1634 TPL de EFI_SIMPLE_TEXT_INPUT, 1629 GetVariable (runtime), 1612 ResetSystem, 1609 Pkcs7Verify (sécurité), 1557 Secure Boot/variables authentifiées, 1741 la carte mémoire renvoyée par GetMemoryMap peut contenir des valeurs impossibles, 1726 « Host and I/O defense », 1691 SMM Communication ACPI table déprécié.
- Ajouts : HII Popup (1730), BluetoothLE (1716), EFI_HTTP_STATUS 308 (1623), DNS device path (1652), NVDIMM Label/BTT (1744, 1745), UFS DeviceConfig (1762), Regular Expression (1627), RAM disk (1466), SD/eMMC PassThru (1376, 1471, 1493), EraseBlocks (1383), ARM CPER (1357), HII Image Ex/Font Ex (1408, 1409), WiFi MAC Connection II (1467), Reset Notification (1690), Key Management Services (1685), Variables (1672).
- [AGENT] Notes utiles : 1741 (GetMemoryMap peut donner des entrées impossibles : valider les types/adresses avant utilisation) ; 1612 (ResetSystem) ; 1609 (Pkcs7Verify, mise en garde sécurité).

## Statut
- Lignes 1 à 4100 lues. Prochaine : ligne 4101 (fin de l'historique 2.5A/2.5/2.4...).

## Lignes 4101-5000 — historique des révisions 2.5 à 2.3.1D (2012–2015)
- Ajouts 2.5 : NVMe Pass Thru (1199), UFS device path (1200, 1255), SD device path (1254), USB Function I/O (1204), SmartCard edge (1234), SMBIOS3 table GUID (1191), DNS4/DNS6/HTTP/HTTPBoot/TLS/EAP2/Wi-Fi/Bluetooth (1212–1220), ESRT (1090), EFI_HASH2_PROTOCOL (1071), inline crypto (1163), System Prep Applications (1159), IPv6 support from UNDI (1121).
- Erratas 2.4 : NVMe device node (1124, 1031, 1024), ATA Pass Thru (1198), EFI_LOAD_OPTION (1211), GetMemoryMap/ResetSystem.
- Erratas 2.3.1 : timestamp (1002), AllocatePool/AllocatePages return codes (909), Platform Key (874, 955), SMM communication ACPI table (893, 1691), InstallAcpiTable duplicates (959, 1081), Protective MBR boot indicator (865), GPT table (885), Netboot6 (880, 881, 960), HII items.
- [AGENT] Items pertinents : 1081/959 (InstallAcpiTable doublon) ; 885 (structure GPT) ; 865 (MBR protectif) : à vérifier si l'agent écrit ou lit une table GPT / ACPI.

## Statut
- Lignes 1 à 5000 lues. Prochaine : ligne 5001.

## Lignes 5001-5900 — historique des révisions 2.3.1A à 2.3 (2009–2012)
- Versions : 2.3.1 (avril 2011, 687/696 : EFI_SYSTEM_TABLE_REVISION), 2.3.1A/B/C/D (2011–2013), 2.3A/B/C (2009–2010).
- Notables : SMM Communication ACPI table (736), Tape header (735), SetVariable update (711, 724), Authenticated variables (679, 785, 776), Key Management Service (484), Security Protocol HDD (616), UEFI system partition FAT32 alignment (612), PXE IPv6 (538, 664), IPSec tunnel (651), IP stack dynamic media detect (452–454), ACPI_SUPPORT_PROTOCOL FADT/DSDT/FACS (580), Machine hand-off/MP (575), GPT LegacyBIOSBootable attribute (532), GPT partition entry size (534), binary prefix (549), Table ISCSI/CHAP device path (522, 539), IPv4/IPv6 device path gateways (513), ACPI ADR device path representation (537), HII items (531, 545, 547, 600, 634–686).
- [AGENT] Items à vérifier dans le code : 580 (FADT/DSDT/FACS via ACPI_SUPPORT_PROTOCOL), 532/534 (attributs GPT) ; 612 (alignement FAT32 de la partition système EFI) si l'agent écrit une partition ESP.

## Statut
- Lignes 1 à 5900 lues. Prochaine : ligne 5901.

## Lignes 5901-6800 — historique des révisions 2.3 (2009) à 2.1C (2008)
- 2.3 (2009) : Firmware Management Protocol ajouté (326), ARM processor binding (376), Non-removable media boot behavior (395), HII FORM_OPEN/FORM_CLOSE (388), Appendix O renommé « UEFI ACPI Data » (418).
- 2.2A/2.2 (2008–2009) : nombreux HII/errata ; UEFI ATA Pass-Thru (242, 282, 324) ; UEFI User Identification (237) ; GPT entry attributes (198, 288) ; Driver Health (169) ; Floating-point ABI x86/x64/Itanium (157) ; EAP, VLAN, FTP, TCP6, MTFTP6, IPv6 (199–315) ; Capsule reset sans retour à l'appelant (303) ; UpdateCapsule errata (304) ; EFI_SYSTEM_TABLE errata (370).
- 2.1C (2008) : IA-32 et x64 stack alignés sur 16 octets (219) ; ResetSystem (56) ; UpdateCapsule (55) ; runtime memory allocation (281) ; Extended SCSI Pass Thru (206) ; PXE/Platform error record x64 register state (203).
- [AGENT] Point utile : 219 (pile alignée sur 16 octets en IA-32/x64 au point d'appel) — à vérifier dans le point d'entrée du binaire UEFI (hwa2_agent) avant tout appel à un service UEFI.

## Statut
- Lignes 1 à 6800 lues. Prochaine : ligne 6801 (fin de l'historique 2.1C/2.1B, puis début du corps à partir de la ligne 6988).

## Lignes 6801-7590 — fin de l'historique (2.1B/2.1A, 2.0) puis CORPS : chapitre 1 et chapitre 2 (début)
### Historique
- 2.1C à 2.0 : 2.0 = première version (31 janvier 2006) ; 2.1 (23 janvier 2007) ; 2.1A (27 avril 2007). 2.1B–2.1C (2007–2008).

### Chapitre 1 Introduction (lignes 6984–7255)
- UEFI remplace le « PC-AT boot world » : interface table + services boot/runtime + protocoles. Spécification = interface pure (ce que le firmware doit fournir), pas une implémentation.
- Modèle de pilote : pilotes de préboot seulement, pas des pilotes OS hautes performances ; EBC (bytecode) optionnel ; compression possible.
- Organisation : boot manager, system table et partitions, Block Translation Table, boot services, runtime services, protocoles, EBC, firmware update, réseau, Secure Boot, HII, user identification, technologies de sécurité, protocoles divers, annexes.
- Principes : abstraction OS/firmware (les protocoles sont la frontière), pas de pilotes legacy, partition système partagée (ESP), compatibilité PC-AT (même disque pour un OS legacy et UEFI).
- Conventions : little endian obligatoire ; champs réservés = 0 à l'écriture, à ignorer en lecture, à préserver en mise à jour ; nombres 0x…, binaire suffixe b ; préfixes SI (1 KB = 1000) vs binaires (1 KiB = 1024).
- Révisions : une version majeure.mineur = contenu substantiel ; lettre (2.10A) = errata.
- [AGENT] Règle clé pour le projet : toujours passer par les protocoles et tables, ne pas accéder aux régions matérielles sans protocole approprié.

### Chapitre 2 Overview (début, lignes 7258–7590)
- 2.1 Boot Manager : images UEFI = PE32+ avec signature modifiée. Sous-systèmes : 10 = EFI_APPLICATION (mémoire EfiLoaderCode/EfiLoaderData), 11 = BOOT_SERVICE_DRIVER (EfiBootServicesCode/Data), 12 = RUNTIME_DRIVER (EfiRuntimeServicesCode/Data).
- Machine (PE32+ header) : IA32 0x014C, IA64 0x0200, EBC 0x0EBC, x64 0x8664, ARMThumb-mixed 0x01C2, AArch64 0xAA64, RISC-V32 0x5032, RISC-V64 0x5064, RISC-V128 0x5128, LoongArch32 0x6232, LoongArch64 0x6264.
- Une application est déchargée à son retour ; un driver n'est déchargé que s'il retourne une erreur UEFI (EFI_SUCCESS garde le driver).
- LoadImage applique relocation et sections ; le point d'entrée reçoit (ImageHandle, EFI_SYSTEM_TABLE*).
- 2.1.3 OS loader : prend le contrôle via ExitBootServices() ; après cela, tous les boot services disparaissent (mémoire incluse) et l'OS est seul responsable.
- 2.1.4 Driver runtime : passe en adressage virtuel après SetVirtualAddressMap().
- 2.2 Firmware core : Runtime Services (GetTime/SetTime/GetWakeupTime/SetWakeupTime/GetVariable/GetNextVariableName/SetVariable/SetVirtualAddressMap/ConvertPointer/GetNextHighMonotonicCount/ResetSystem/UpdateCapsule/QueryCapsuleCapabilities/QueryVariableInfo) — tous non bloquants, appelables interruptions désactivées.
- Les zones mémoire des runtime services doivent être réservées au système d'exploitation (EfiRuntimeServicesCode/Data).

### Chapitre 2.3 Calling conventions (début)
- Tous les pointeurs de fonction UEFI sont castés en EFIAPI (convention d'appel du compilateur).
- Appelant : pointeurs physiques valides, alignement correct, pas de NULL sauf si autorisé, structure > taille native par valeur interdit.
- Types : BOOLEAN 1 octet (0/1) ; INTN/UINTN taille native ; INT8…UINT64 ; CHAR8 (ISO-Latin-1) ; CHAR16 (UCS-2) ; EFI_GUID 128 bits ; EFI_STATUS = UINTN ; EFI_LBA = UINT64 ; EFI_TPL = UINTN ; EFI_MAC_ADDRESS 32 octets ; EFI_IPv4_ADDRESS 4 octets ; EFI_IPv6_ADDRESS 16 octets.
- 2.3.2 IA-32 : protégé, flat, paging optionnel (PAE recommandé, identité pour la memory map UEFI), pile ≥ 128 KiB alignée 16 octets, CR0.EM=0, CR0.TS=0, FCW=0x027F, MXCSR=0x1F80, DF=0, interruptions possibles (les services ne sont que « polling »). Après ExitBootServices : mémoire runtime doit être préservée ; pile ≥ 4 KiB alignée 16 octets pour runtime.
- ACPI : tables au boot en EfiACPIReclaimMemory (recommandé) ou EfiACPIMemoryNVS ; FACS doit être en EfiACPIMemoryNVS ; EfiACPIReclaimMemory et EfiACPIMemoryNVS alignés 4 KiB et multiples de 4 KiB.
- Tables de configuration (SMBIOS…) : en EfiRuntimeServicesData (recommandé), EfiBootServicesData (SMBIOS seulement pour OS conformes), EfiACPIReclaimMemory ou EfiACPIMemoryNVS.
- 2.3.3 Itanium : physique, pile 128 KiB, backing store 16 KiB, FPSR particulier.
- [AGENT — IMPORTANT] Si le binaire UEFI de hwa2_agent passe à un OS ou appelle ExitBootServices : pile alignée 16, FCW=0x027F, MXCSR=0x1F80, DF clair, régions ACPI en 4 KiB. Le code existant n'a pas encore été vérifié sur ces points.

## Statut
- Lignes 1 à 7590 lues. Prochaine : ligne 7591 (suite 2.3.4 x64 Platforms).

## Lignes 7591-8190 — 2.3.4 x64, 2.3.5 AArch32, 2.3.6 AArch64, 2.3.7 RISC-V, 2.3.8 LoongArch, 2.4 Protocoles, 2.5 Driver Model (début)
### 2.3.4 x64 (lignes 7587–7662) — À RETENIR POUR UN BINAIRE UEFI x64
- Mode long 64 bits, paging actif, mémoire de la carte UEFI identity-mappée (virt = phys) ; régions « unaccepted memory » non définies.
- Pile ≥ 128 KiB, alignée 16 octets ; DF = 0 ; CR0.EM = 0 ; CR0.TS = 0.
- **FCW = 0x037F** (x64 : double-extended, toutes exceptions masquées, arrondi au plus proche). MXCSR (si présent) = 0x1F80.
- Handoff : RCX = EFI_HANDLE, RDX = EFI_SYSTEM_TABLE*, RSP = adresse de retour.
- Convention : arguments 1–4 dans RCX, RDX, R8, R9 ; au-delà sur la pile ; espace shadow (32 octets) à réserver par l'appelant ; retour dans RAX (ou pointeur passé en premier argument si > 64 bits). Volatiles : RAX, RCX, RDX, R8–R11, XMM0–XMM5 ; non volatiles : RBX, RBP, RDI, RSI, R12–R15, XMM6–XMM15.
- Pile alignée 16 avant l'appel. Les flottants/doubles passent en XMM0–XMM3.
- Runtime après ExitBootServices (OS) : pile ≥ 4 KiB, alignée 16 ; mémoire runtime préservée ; SetVirtualAddressMap si relocalisation.
- Régions ACPI : EfiACPIReclaimMemory / EfiACPIMemoryNVS alignées 4 KiB, multiples de 4 KiB ; FACS en EfiACPIMemoryNVS.
- Tables de configuration : EfiRuntimeServicesData (recommandé) ; SMBIOS en EfiBootServicesData seulement pour OS conformes.

### 2.3.5 AArch32 (7666–7727)
- Unaligned accesses, MMU activé, identity mapping ; SCTLR config : I=1, C=1, B=0, TRE=0, AFE=0, M=1 ; DACR = 0x33333333 ; 128 KiB de pile minimum ; interruptions = polling seulement ; 4 KiB pages uniquement ; TTBR0 seul.
- Handoff : R0 = EFI_HANDLE, R1 = EFI_SYSTEM_TABLE*, R14 = adresse de retour.
- Runtime : mode privilégié, régions RUNTIME identity-mappées, pile ≥ 4 KiB.

### 2.3.6 AArch64 (7738–7878)
- 64 bits ; MMU et caches activés ; identity mapping ; 4 KiB granule ; SCTLR_EL1/EL2 : EE=0, I=1, SA=1, C=1, A=0, M=1 ; TCR TG0=00 ; FP activé (FPCR=0, CPACR_EL1.FPEN=11).
- Timer générique : CNTFRQ programmé ; accès EL1/EL0 via CNTHCTL_EL2 (EL1PCTEN, EL1PCEN).
- Pile ≥ 128 KiB (boot) ; alignement 16 octets ; 4 KiB pages ; TTBR0 seul ; aucune adresse hors 48 bits sauf cas exceptionnels.
- Table 2.5 : mappage EFI_MEMORY_UC/WC/WT/WB/UCE → MAIR (Device-nGnRnE 0000 0000, Normal non-cacheable 0100 0100, WT 1011 1011, WB 1111 1111) ; SH=11 pour WT/WB.
- Table 2.6 : EFI_MEMORY_XP → XN (EL2) / UXN+PXN (EL1/0) ; EFI_MEMORY_RO → AP[2]=1.
- Handoff : X0 = EFI_HANDLE, X1 = EFI_SYSTEM_TABLE*, X30 = adresse de retour.
- Runtime : pile ≥ 8 KiB alignée 16 ; EL1 ou EL2 seul et cohérent ; pas d'appels simultanés EL1/EL2.
- Convention : A64 uniquement ; flottants autorisés ; pile 16 octets ; r18 (plate-forme) réservé.

### 2.3.7 RISC-V (7884–7960)
- RV32/RV64/RV128, mode natif uniquement. Extensions requises : A, C, I, M, Zicsr, Zifencei.
- Supervisor little endian ; HS mode si hypervisor présent ; pile ≥ 128 KiB ; timer activé (Sstc si présent).
- Handoff : a0 (x10) = EFI_HANDLE, a1 (x11) = EFI_SYSTEM_TABLE*, ra (x1) = retour.
- Runtime : pile ≥ 8 KiB alignée 16 ; gp (x3) et tp (x4) NON fiables côté firmware (à préserver si modifiés, ne jamais les toucher après ExitBootServices).

### 2.3.8 LoongArch (7961–8004)
- Exécution PLV0 uniquement ; vecteur de reset 0x1C00 0000 ; caches et MMU activés ; I/O en mémoire mappée ; pile ≥ 128 KiB ; timer et stable counter activés.
- Handoff : r4 (a0) = EFI_HANDLE, r5 (a1) = EFI_SYSTEM_TABLE*, r1 (ra) = retour.
- Runtime : pile ≥ 16 KiB ; régions ACPI et runtime alignées 64 KiB.
- Convention : 8 registres d'arguments a0–a7 ; retours a0–a1 ; s0–s9 préservés.

### 2.4 Protocoles (8006–8040)
- Découverte via HandleProtocol() ou OpenProtocol() avec le GUID.
- Les protocoles ne sont pas alloués en mémoire runtime (sauf mention) ; leurs fonctions sont appelables à TPL ≤ TPL_NOTIFY ; non réentrants, non MP-safe sauf mention.
- Table 2.7 : liste complète des protocoles (Loaded Image, Device Path, Driver Binding, Simple Text I/O, Simple Pointer, Serial I/O, Load File/Load File2, Simple File System, File, Disk I/O, Block I/O/2, Unicode Collation, PCI Root Bridge I/O, PCI I/O, USB I/O, SNP, PXE, BIS, Debug Support/Debugport, Decompress, EBC, GOP, NVMe Pass Thru, Ext SCSI Pass Thru, USB2 HC, Authentication Info, Device Path Utilities/To/From Text, EDID, iSCSI, Tape I/O, MNP, ARP, DHCP4, TCP4, IP4, IP4Config2, UDP4, MTFTP4, Hash, SD MMC Pass Thru).

### 2.5 UEFI Driver Model (8157–8200)
- Un driver produit un Driver Binding Protocol sur son image handle ; quand le firmware le connecte, il produit un protocole sur le device handle du contrôleur. Un bus driver crée en plus les handles enfants.
- Arbre : cœurs CPU → chipsets → bus → périphériques (figure desktop/server).
- 2.5.1 : les option ROM legacy sont en 16 bits réel (≤ 1 MiB), la zone 0xC0000–0xDFFFF (128 KiB) est partagée, EBDA non standard ; le driver UEFI doit être en mode plat C portable.
- [AGENT] Si l'agent tourne au niveau UEFI (pas en bare-metal), les règles de pile (16 octets), FCW=0x037F et MXCSR=0x1F80 sont obligatoires ; le code bare-metal actuel (hwa2_agent/src) doit être vérifié.

## Statut
- Lignes 1 à 8190 lues. Prochaine : ligne 8191 (fin de 2.5.1 Legacy option ROM, puis 2.5.2 Driver Initialization).

## Lignes 8191-8790 — 2.5.1 à 2.5.8 (Driver Model), 2.6 (exigences), CHAPITRE 3 Boot Manager (début)
### 2.5 Driver model (suite)
- Un driver ne touche pas le matériel dans son point d'entrée ; il installe Driver Binding (obligatoire), éventuellement Component Name, Driver Diagnostics, Unload() dans Loaded Image, un événement ExitBootServices.
- Host Bus Controllers : créés par le firmware (ex. PCI Root Bridge I/O Protocol), hors du modèle.
- Device driver : Supported() / Start() / Stop() ; utilise OpenProtocol()/CloseProtocol() (BY_DRIVER, un seul consommateur par protocole) ; OpenProtocolInformation() pour l'inventaire.
- Bus driver : crée les handles enfants, installe Device Path (si périphérique physique) et au minimum le protocole I/O du bus ; Bus Specific Driver Override optionnel.
- Platform : ConnectController() / DisconnectController() ; Platform Driver Override a la priorité la plus haute.
- Hot-plug : le bus driver appelle ConnectController() / DisconnectController() ; si Disconnect échoue, retry depuis un timer.
- 2.5.8 EFI_SERVICE_BINDING_PROTOCOL : famille de protocoles (sans GUID propre, un GUID par service) pour les cas où plusieurs consommateurs partagent un service.

### 2.6 Exigences
- 2.6.1 Required (Table 2.8) : System Table, Boot Services, Runtime Services, Loaded Image (+ Device Path), Device Path, Decompress, Device Path Utilities.
- 2.6.2 Platform-specific (1–36) : console (Simple Text In/Out/Ex), HII si configuration, GOP+EDID si graphique, Simple Pointer, Block I/O+Disk I/O+Simple FS+Unicode Collation+MBR/GPT/El Torito pour le boot disque, PXE si réseau, MNP/ARP/DHCP4/TCP4/IP4/UDP4 pour applications réseau, Serial I/O, PCI Root Bridge I/O + PCI I/O, USB2 HC + USB I/O, NVMe Pass Thru (+ Block I/O pour boot NVMe), Ext SCSI Pass Thru, SCSI I/O, iSCSI, Debug Support/Debugport, Platform Driver Override, ATA Pass Thru, Authentication Info, ESP.
- Item 34 : une plate-forme peut renvoyer EFI_UNSUPPORTED pour certains runtime services après ExitBootServices, et doit publier EFI_RT_PROPERTIES_TABLE.
- Item 36 : RISC-V doit implémenter RISCV_EFI_BOOT_PROTOCOL (GetBootHartId).
- Item 27 : Boot####, Driver####, SysPrep#### (variables de chargement) : prise en charge de tous les bits Attributes, Driver#### traités à chaque boot, SysPrep#### avant Boot####.
- 2.6.3 Driver-specific (1–28) : Driver Binding (+ Component Name recommandé), HII Database pour configuration, Driver Diagnostics2, Bus Specific Driver Override pour bus à option ROM, Block I/O + Block I/O2 pour disque, SPC-4/ATA8 security, PCI Root Bridge I/O + PCI I/O, Load File2, Driver Health Protocol, Driver Family Override, Firmware Management, PCI device driver = accès via PCI I/O uniquement (item 25).
- 2.6.5 Cryptographie : SHA-256 minimum (variables authentifiées PKCS#1 v1.5), RSA ≥ 2048 pour EAP-TLS et PK Secure Boot, SHA-1 et MD5 seulement pour compatibilité (SHA-1 pour TPM 1.2, MD5 pour CHAP iSCSI).
- [AGENT] Pour un OS écrit « from zero » : le minimum est Loaded Image + Device Path + Boot/Runtime Services ; PCI I/O et NVMe Pass Thru sont les protocoles à utiliser au lieu d'accès MMIO direct (item 25 : un pilote de périphérique PCI doit passer par PCI I/O pour tous les accès mémoire/I/O/config/DMA).

### CHAPITRE 3 Boot Manager (lignes 8631–8790)
- Le boot manager est une machine à politique pilotée par les variables NVRAM globales : BootOrder, BootNext, DriverOrder, Boot####, Driver####, SysPrep####, OsRecovery####, PlatformRecovery####, Key####.
- 3.1.1 : format #### = 4 chiffres hexadécimaux majuscules. Une liste ordonnée (DriverOrder, BootOrder) référence les variables. Le firmware peut supprimer les variables non référencées ou mal formées.
- 3.1.2 : traitement des drivers avant les boot ; OsIndications bits EFI_OS_INDICATIONS_START_OS_RECOVERY / START_PLATFORM_RECOVERY ; BootNext est supprimé avant le démarrage pour éviter les boucles ; LoadImage supporte Simple FS et Load File ; watchdog 5 min avant StartImage, désactivé si l'image rend la main.
- Boot short-form : USB WWID et USB Class (0xFFFF/0xFF = élément ignoré), Hard Drive (GUID GPT ou signature MBR + numéro de partition ; l'OS doit assurer l'unicité des signatures), File Path media (énumère les médias amovibles puis fixes), URI (LoadFile).
- 3.1.3 EFI_LOAD_OPTION : Attributes (UINT32), FilePathListLength (UINT16), Description (CHAR16 null-terminé), FilePathList (device paths), OptionalData (reste).
  - Bits : LOAD_OPTION_ACTIVE = 0x1 ; FORCE_RECONNECT = 0x2 ; HIDDEN = 0x8 ; CATEGORY = 0x1F00 (BOOT = 0x0000, APP = 0x0100 ; 0x0200–0x1F00 réservés).
  - Driver#### doit être BOOT_SERVICE_DRIVER ou RUNTIME_DRIVER ; SysPrep/Boot/OsRecovery doit être APPLICATION.
- 3.1.4 BootOptionSupport : KEY = 0x1, APP = 0x2, SYSPREP = 0x10, COUNT = 0x300 (nombre max de touches).
- 3.1.6 Key#### : EFI_KEY_OPTION (KeyData, BootOptionCrc, BootOption) pour lancement par touches.

## Statut
- Lignes 1 à 8790 lues. Prochaine : ligne 8791 (suite 3.1.6 Key#### et 3.1.7 System preparation applications, puis 3.2 Boot Manager Policy Protocol).

## Lignes 8791-9390 — 3.1.6 à 3.5 (boot manager, variables globales, recovery), CHAPITRE 4 (System Table)
### 3.1.6–3.2 (8791–8830)
- EFI_KEY_OPTION : KeyData (EFI_BOOT_KEY_DATA : Revision 8 bits, Shift/Control/Alt/Logo/Menu/SysReq, InputKeyCount 2 bits), BootOptionCrc (CRC-32 du Boot####), BootOption, Keys[] (0 à 3 touches).
- Exemples de PackedValue : ALT = 0x00000400 ; CTRL-ALT-P-R = 0x80000600 ; CTRL-F1 = 0x40000200.
- 3.1.7 SysPrep#### exécutés avant Boot#### selon SysPrepOrder ; ne doivent pas appeler ExitBootServices ; EVENT_GROUP_READY_TO_BOOT et AFTER_READY_TO_BOOT notifiés après SysPrep et avant Boot.
- 3.2 Boot Manager Policy Protocol (GUID FEDF8E0C-E147-11E3-9903-B8E8562CBAFA) : ConnectDevicePath(This, DevicePath, Recursive), ConnectDeviceClass(This, Class). Classes : CONSOLE CAB0E94C-…, NETWORK D04159DC-…, STORAGE CD68FE79-…, CONNECT_ALL 113B2126-…

### 3.3 Variables globales (9001–9124)
- GUID EFI_GLOBAL_VARIABLE = 8BE4DF61-93CA-11D2-AA0D-00E098032B8C.
- Attributs : NV (persistant), BS (avant ExitBootServices seulement), RT (avant et après), AT (écriture authentifiée temporelle).
- Variables : AuditMode, Boot####, BootCurrent, BootNext, BootOrder, BootOptionSupport, ConIn/ConOut/ErrOut (+ Dev versions volatiles), Driver####, DriverOrder, Key####, KEK, KEKDefault, PK, PKDefault, dbDefault/dbrDefault/dbtDefault/dbxDefault, DeployedMode, devAuthBoot, devdbDefault, HwErrRecSupport, OsIndications, OsIndicationsSupported, OsRecoveryOrder, PlatformLang, PlatformLangCodes, PlatformRecovery####, SecureBoot, SetupMode, SignatureSupport, SysPrep####, SysPrepOrder, Timeout (UINT16, 0xFFFF = attendre l'utilisateur), VendorKeys.
- SetupMode = 1 quand SetupMode==1, AuditMode==0, DeployedMode==0 (mode configuration des clés).
- SecureBoot : 1 = vérification active pendant le boot en cours, initialisé avant l'authentification, immuable.
- BootCurrent : défini avant EVENT_GROUP_READY_TO_BOOT.

### 3.4 Boot option recovery (9125–9145)
- Ordre : OS recovery (OsRecovery####, GUID de OsRecoveryOrder, signé dbr/KEK/PK) → BootOrder à nouveau → platform recovery (PlatformRecovery####).
- Par défaut : une entrée PlatformRecovery#### doit pointer vers le chemin par défaut des médias amovibles (\EFI\BOOT\BOOT{machine}.EFI).

### 3.5 Mécanismes de boot (9146–9190)
- Simple File System d'abord, Load File ensuite. Chemin par défaut des médias amovibles : \EFI\BOOT\BOOT{machine}.EFI.
- Table 3.4 : BOOTIA32.EFI (0x014C), BOOTx64.EFI (0x8664), BOOTIA64.EFI (0x200), BOOTARM.EFI (0x01C2), BOOTAA64.EFI (0xAA64), BOOTRISCV32.EFI (0x5032), BOOTRISCV64.EFI (0x5064), BOOTRISCV128.EFI (0x5128), BOOTLOONGARCH32.EFI (0x6232), BOOTLOONGARCH64.EFI (0x6264).
- [AGENT] Pour une clé USB UEFI de l'agent : le fichier doit être \EFI\BOOT\BOOTx64.EFI (écrire sur la clé exige accord explicite — règle du projet).

### CHAPITRE 4 — EFI System Table (9192–9390)
- 4.1 Point d'entrée : EFI_IMAGE_ENTRY_POINT(ImageHandle, SystemTable). Applications, OS loaders et drivers ont le même point d'entrée.
- Application : déchargée à la sortie ; OS loader : si ExitBootServices est appelé, EFI ne reprend pas la main avant un reset (ResetSystem) ; driver : retourne EFI_SUCCESS pour rester résident, erreur pour être déchargé.
- Driver Model : l'entrée n'accède jamais au matériel, installe seulement le Driver Binding Protocol sur ImageHandle ; EFI_OUT_OF_RESOURCES si échec.
- 4.2 EFI_TABLE_HEADER : Signature (UINT64), Revision (UINT32 : haut = majeur, bas = mineur en décimal), HeaderSize, CRC32 (CRC-32 calculé avec ce champ à 0 sur HeaderSize octets, polynôme 0x04C11DB7), Reserved = 0.
- Révisions : (2<<16)|(100) = 2.10 ; (2<<16)|(31) = 2.3.1 ; (2<<16)|(110) = 2.11 (valeur de cette spec).
- 4.3 EFI_SYSTEM_TABLE : Hdr, FirmwareVendor (CHAR16*), FirmwareRevision (UINT32), ConsoleInHandle, ConIn, ConsoleOutHandle, ConOut, StandardErrorHandle, StdErr, RuntimeServices, BootServices, NumberOfTableEntries, ConfigurationTable.
- Signature : EFI_SYSTEM_TABLE_SIGNATURE = 0x5453595320494249 (ASCII « IBI SYST » en little-endian).
- Après ExitBootServices, seuls Hdr, FirmwareVendor, FirmwareRevision, RuntimeServices, NumberOfTableEntries et ConfigurationTable restent valides.
- [AGENT — IMPORTANT] Pour un OS qui appelle ExitBootServices : ne plus utiliser ConOut, BootServices, ni les protocoles ; garder seulement RuntimeServices et ConfigurationTable (ACPI RSDP, etc.).

## Statut
- Lignes 1 à 9390 lues. Prochaine : ligne 9391 (suite 4.3.1 : pointeurs ConIn/ConOut/StdErr, puis 4.4 EFI Boot Services Table, 4.5 Runtime Services Table, 4.6 configuration tables).

## Lignes 9391-9990 — 4.3.1 fin, 4.4 Boot Services Table, 4.5 Runtime Services Table, 4.6 Configuration Tables
- 4.4 EFI_BOOT_SERVICES : signature 0x56524553544F4F42 ("BOOTSERV"). Ordre des pointeurs (à respecter pour l'ABI) : Hdr, RaiseTPL, RestoreTPL, AllocatePages, FreePages, GetMemoryMap, AllocatePool, FreePool, CreateEvent, SetTimer, WaitForEvent, SignalEvent, CloseEvent, CheckEvent, InstallProtocolInterface, ReinstallProtocolInterface, UninstallProtocolInterface, HandleProtocol, Reserved (NULL), RegisterProtocolNotify, LocateHandle, LocateDevicePath, InstallConfigurationTable, LoadImage, StartImage, Exit, UnloadImage, ExitBootServices, GetNextMonotonicCount, Stall, SetWatchdogTimer, ConnectController, DisconnectController, OpenProtocol, CloseProtocol, OpenProtocolInformation, ProtocolsPerHandle, LocateHandleBuffer, LocateProtocol, InstallMultipleProtocolInterfaces, UninstallMultipleProtocolInterfaces, CalculateCrc32, CopyMem, SetMem, CreateEventEx.
- Après ExitBootServices, les pointeurs de boot services ne sont plus valides.
- 4.5 EFI_RUNTIME_SERVICES : signature 0x56524553544E5552 ("RUNTSERV"). Ordre : Hdr, GetTime, SetTime, GetWakeupTime, SetWakeupTime, SetVirtualAddressMap, ConvertPointer, GetVariable, GetNextVariableName, SetVariable, GetNextHighMonotonicCount, ResetSystem, UpdateCapsule, QueryCapsuleCapabilities, QueryVariableInfo.
- Table valide après ExitBootServices ; après SetVirtualAddressMap, les pointeurs sont relocalisés.
- 4.6 EFI_CONFIGURATION_TABLE : VendorGuid + VendorTable. GUID uniques, au plus une instance par type.
  - ACPI 2.0+ : EFI_ACPI_TABLE_GUID = 8868E871-E4F1-11D3-BC22-0080C73C8881 (pointe vers RSDP). ACPI 1.0 : EB9D2D30-2D88-11D3-9A16-0090273FC14D. SMBIOS : EB9D2D31-…; SMBIOS3 : F2FD1544-9794-4A2C-992E-E5BBCF20E394 ; SAL : EB9D2D32-… ; MPS : EB9D2D2F-….
  - JSON : EFI_JSON_CONFIG_DATA_TABLE_GUID, EFI_JSON_CAPSULE_DATA_TABLE_GUID, EFI_JSON_CAPSULE_RESULT_TABLE_GUID.
  - Devicetree : EFI_DTB_TABLE_GUID = B1B621D5-F19C-41A5-830B-D9152C69AAE0 (DTB en EfiACPIReclaimMemory).
  - 4.6.2 EFI_RT_PROPERTIES_TABLE (GUID EB66918A-7EEF-402A-842E-931D21C38AE9) : Version = 0x1, Length = 8, RuntimeServicesSupported (bits : GET_TIME 0x1, SET_TIME 0x2, GET_WAKEUP 0x4, SET_WAKEUP 0x8, GET_VARIABLE 0x10, GET_NEXT_VARIABLE_NAME 0x20, SET_VARIABLE 0x40, SET_VIRTUAL_ADDRESS_MAP 0x80, CONVERT_POINTER 0x100, GET_NEXT_HIGH_MONOTONIC_COUNT 0x200, RESET_SYSTEM 0x400, UPDATE_CAPSULE 0x800, QUERY_CAPSULE_CAPABILITIES 0x1000, QUERY_VARIABLE_INFO 0x2000). Même si non supporté, la fonction doit renvoyer EFI_UNSUPPORTED.
  - 4.6.3 EFI_MEMORY_ATTRIBUTES_TABLE (GUID DCFA911D-26EB-469F-A220-38B7DC461220) : Version = 2, NumberOfEntries, DescriptorSize, Flags (bit0 = forward CFG/CET-IBT/BTI), Entry[] (EFI_MEMORY_DESCRIPTOR). Seuls RO/XP/RUNTIME comptent. Régions RUNTIME jamais superposées ; sous-région de GetMemoryMap.
- [AGENT] Pour l'OS « from zero » : après ExitBootServices, lire ACPI via ConfigurationTable (GUID ACPI 2.0), ne pas dépendre de ConOut, et si on utilise les runtime services, tenir compte de RT_PROPERTIES et MEMORY_ATTRIBUTES.

## Statut
- Lignes 1 à 9990 lues. Prochaine : ligne 9991 (table 4.1 et 4.6.4 Conformance Profile, 4.6.5, 4.7 exemples d'entrée, puis chapitre 5 GPT).

## Lignes 9991-10590 — 4.6.4 à 4.7 (exemples d'entrée), CHAPITRE 5 GPT (début)
- 4.6.4 EFI_CONFORMANCE_PROFILES_TABLE (GUID 36122546-F7E7-4C8F-BD9B-EB8525B50C0B) : Version = 1, NumberOfProfiles, ConformanceProfiles[] ; absence = conforme à 2.6 ; profil UEFI spec GUID 523C91AF-A195-4382-818D-295FE4006465.
- 4.6.5 Autres tables : MEMORY_RANGE_CAPSULE, DEBUG_IMAGE_INFO (18.4.3), SYSTEM_RESOURCE (23.4), IMAGE_EXECUTION_INFO (32.5.3.1), User Information Table (36.5), HII export (33.2.11.1).
- 4.7 Exemples : application (gST/gBS/gRT ; ConOut->OutputString ; AllocatePool(EfiBootServicesData) ; GetTime) ; driver qui retourne EFI_SUCCESS reste résident ; driver qui retourne EFI_DEVICE_ERROR est déchargé.
- 4.7.2–4.7.4 : exemple Driver Model : InstallMultipleProtocolInterfaces(&DriverBindingHandle, gEfiDriverBindingProtocolGuid, …) ; version déchargeable : Unload() via LoadedImage->Unload, OpenProtocol(GET_PROTOCOL) ; version multi-instances (handles distincts).
- [AGENT] Modèle minimal pour un binaire UEFI de l'agent : ImageHandle, SystemTable -> BootServices, ConOut ; toujours vérifier les EFI_ERROR.

## CHAPITRE 5 — GUID Partition Table (lignes 10301–10590)
- 5.1 Avantages GPT : LBA 64 bits ; nombre de partitions illimité (en pratique 128 entrées) ; table primaire + secondaire ; CRC32 ; GUID unique par partition ; type GUID ; nom de 36 caractères.
- 5.2 LBA 0 : soit MBR legacy, soit MBR protectif (obligatoire si GPT).
- 5.2.1 MBR legacy (Table 5.1) : BootCode 440 octets (non exécuté par UEFI) ; UniqueMbrSignature à l'offset 440 (écrit par l'OS, jamais par le firmware) ; 2 octets inconnus (444) ; 4 entrées de partition à l'offset 446 (16 octets chacune) ; signature 0xAA55 à l'offset 510.
- Entrée MBR (Table 5.2) : BootIndicator (0x80 = actif), StartingCHS (3), OSType (1), EndingCHS (3), StartingLBA (4), SizeInLBA (4).
- Si OSType = 0xEF (UEFI System Partition) : le firmware doit ajouter le GUID de partition système EFI au handle via InstallProtocolInterface().
- Validation MBR : signature = 0xAA55 ; entrée avec OSType 0 ou SizeInLBA 0 ignorée ; partitions dans le disque et sans chevauchement.
- 5.2.2 OSType : 0xEF = EFI System Partition ; 0xEE = GPT Protective ; autres valeurs = OS legacy.
- 5.2.3 MBR protectif (Table 5.3) : BootCode non utilisé (0), disk signature 0, unused 0, une entrée de partition : BootIndicator 0x00, StartingLBA = 0x000200 ? (table : 0x000200 — noter : valeur indiquée par la table), OSType 0xEE, EndingCHS = 0xFFFFFF si non représentable, StartingLBA = 1 (LBA du header GPT), SizeInLBA = taille du disque - 1 (0xFFFFFFFF si trop grand) ; trois autres entrées à 0 ; signature 0xAA55.
- [AGENT — sécurité] Ne jamais écrire le MBR ni le GPT sur une clé ou un disque interne sans accord explicite (règle du projet). Lire seulement.

## Statut
- Lignes 1 à 10590 lues. Prochaine : ligne 10591 (suite 5.2.3 Figures 5.2–5.3, 5.2.4 Partition Information, 5.3 GPT header et entrées).

## Lignes 10591-11190 — 5.2.4 à 5.3 (GPT), CHAPITRE 6 BTT, début CHAPITRE 7 (Boot Services)
### 5.2.4 : EFI_PARTITION_INFO sur chaque handle EFI_BLOCK_IO.
### 5.3 GPT
- Organisation : LBA 0 = MBR protectif ; LBA 1 = en-tête primaire ; dernière LBA = en-tête de secours. Entrées : tableau primaire après l'en-tête, secours avant l'en-tête de secours.
- En-tête GPT (Table 5.5) : Signature 8 octets = « EFI PART » (0x5452415020494645) ; Revision 0x00010000 ; HeaderSize (≥ 92, ≤ taille du bloc) ; HeaderCRC32 (CRC calculé avec ce champ à 0) ; Reserved (0) ; MyLBA ; AlternateLBA ; FirstUsableLBA ; LastUsableLBA ; DiskGUID (16) ; PartitionEntryLBA ; NumberOfPartitionEntries ; SizeOfPartitionEntry (128 × 2^n) ; PartitionEntryArrayCRC32 (sur NumberOfPartitionEntries × SizeOfPartitionEntry) ; reste à 0.
- Tailles : FirstUsableLBA ≥ 34 si bloc 512 (1 MBR + 1 en-tête + 32 blocs d'entrées) ; ≥ 6 si bloc 4096. Au moins 16 384 octets pour le tableau.
- Validation : signature, CRC d'en-tête, MyLBA = position réelle, CRC du tableau. Si primaire corrompu : utiliser le secours, le restaurer en signalant (confirmation utilisateur recommandée). Mettre à jour secours avant primaire.
- Alignement recommandé : multiple de 2048 LBA (1 MiB).
- Table 5.6 : entrée de partition 128 octets : PartitionTypeGUID (16), UniquePartitionGUID (16), StartingLBA (8), EndingLBA (8), Attributes (8), PartitionName (72 = 36 CHAR16). Entrée à zéro = inutilisée.
- Table 5.7 : GUID EFI System Partition = C12A7328-F81F-11D2-BA4B-00A0C93EC93B ; partition contenant un MBR legacy = 024DEE41-33E7-11D3-9D69-0008C781F39F.
- Table 5.8 Attributes : bit 0 Required Partition ; bit 1 No Block IO Protocol (pas de EFI_BLOCK_IO pour cette partition) ; bit 2 Legacy BIOS Bootable (ignoré par UEFI) ; bits 48–63 réservés au type de partition.
- [AGENT — sécurité] Ne jamais réécrire un GPT existant ; lecture seule sans accord. Pour l'écriture d'une ESP sur clé : accord explicite requis.

### CHAPITRE 6 BTT (Block Translation Table) — résumé
- Concerne les espaces de noms NVDIMM (atomicité de bloc 512 GiB arènes). Structures : Info Block (signature « BTT_ARENA_INFO00 », Major 2, Minor 0, checksum Fletcher64), Map (entrées 32 bits : PostMapLba 30 bits, Error, Zero), Flog (free list + journal, 2 copies Seq0/Seq1), Data Area.
- Non prioritaire pour le projet ; à retenir uniquement si l'agent manipule des NVDIMM.

### CHAPITRE 7 Boot Services — début (11185–11190)
- Deux types de services : boot services (avant ExitBootServices) et runtime services (avant et après).
- Services « globaux » (événements, mémoire, Image, Protocol) et « handle-based » (protocoles dynamiques).
- Le system table fournit le dispatch des boot services et les handles console par défaut.
- ExitBootServices : transfert au système d'exploitation ; ensuite seul le runtime reste.

## Statut
- Lignes 1 à 11190 lues. Prochaine : ligne 11191 (7.1 Event, Timer, Task Priority Services : CreateEvent, CreateEventEx, CloseEvent, SignalEvent, WaitForEvent, CheckEvent, SetTimer, RaiseTPL, RestoreTPL).

## Lignes 11191-11790 — CHAPITRE 7.1 Event, Timer, TPL (CreateEvent, CreateEventEx, CloseEvent)
- Niveaux TPL : TPL_APPLICATION (4, le plus bas, E/S utilisateur, opérations longues), TPL_CALLBACK (8, interruptible seulement sous ce niveau, opérations longues OK), TPL_NOTIFY (16, pas de blocage : code qui se termine et signale un événement pour la suite), TPL_HIGH_LEVEL (31, réservé au firmware, interruptions désactivées).
- RaiseTPL masque les notifications de niveau égal ou inférieur ; RestoreTPL rétablit.
- Table 7.3 (restrictions) : par ex. CreateEvent, CloseEvent (≤ TPL_CALLBACK selon les cas) ; ExitBootServices() à TPL_APPLICATION ; SetTimer < TPL_CALLBACK ; StartImage < TPL_APPLICATION ; WaitForEvent = TPL_APPLICATION ; protocoles réseau TPL_CALLBACK ; Simple Text Input ReadKeyStroke ≤ TPL_CALLBACK.
- CreateEvent Type (constantes) : EVT_TIMER 0x80000000 ; EVT_RUNTIME 0x40000000 (événement en mémoire runtime) ; EVT_NOTIFY_WAIT 0x100 ; EVT_NOTIFY_SIGNAL 0x200 ; EVT_SIGNAL_EXIT_BOOT_SERVICES 0x201 ; EVT_SIGNAL_VIRTUAL_ADDRESS_CHANGE 0x60000202.
- Un événement est « waiting » ou « signaled » ; les notifications sont mises en file FIFO par niveau TPL.
- Les EVT_TIMER ne fonctionnent que pendant les boot services.
- CreateEventEx : groupes d'événements (GUID). Groupes pré-définis : EXIT_BOOT_SERVICES 27ABF055-B1B8-4C26-8048-748F37BAA2DF ; BEFORE_EXIT_BOOT_SERVICES 8BE0E274-3970-4B44-80C5-1AB9502F3BFC ; VIRTUAL_ADDRESS_CHANGE 13FA7698-C831-49C7-87EA-8F43FCC25196 ; MEMORY_MAP_CHANGE 78BEE926-692F-48FD-9EDB-01422EF0D7AB ; READY_TO_BOOT 7CE88FB3-4BD7-4679-87A8-A8D8DEE50D2B ; AFTER_READY_TO_BOOT 3A2A00AD-98B9-4CDF-A478-702777F1C10B ; RESET_SYSTEM 62DA6A56-13FB-485A-A8DA-A3DD7912CB6B.
- Règle clé pour les notifications EXIT_BOOT_SERVICES : pas d'allocation mémoire (elle modifie la carte mémoire), pas de dépendance aux timers (désactivés juste après).
- Groupe de table de configuration : chaque GUID de table définit aussi un groupe ; InstallConfigurationTable() signale le groupe.
- [AGENT] Si l'agent gère des interruptions ou des timers après ExitBootServices, ces événements ne sont plus disponibles : tout doit être refait par l'agent.

## Statut
- Lignes 1 à 11790 lues. Prochaine : ligne 11791 (suite CloseEvent, SignalEvent, WaitForEvent, CheckEvent, SetTimer, RaiseTPL, RestoreTPL, 7.2 Memory services).

## Lignes 11791-12390 — 7.1.4 à 7.2.3 (SignalEvent, WaitForEvent, CheckEvent, SetTimer, RaiseTPL/RestoreTPL, Memory services)
- SignalEvent : utilisable à tout TPL ; événements d'un groupe tous signalés.
- WaitForEvent : TPL_APPLICATION obligatoire (sinon EFI_UNSUPPORTED) ; EVT_NOTIFY_SIGNAL refusé (EFI_INVALID_PARAMETER).
- CheckEvent : EVT_NOTIFY_SIGNAL refusé ; EFI_NOT_READY si non signalé.
- SetTimer : TimerCancel = 0, TimerPeriodic = 1, TimerRelative = 2 ; TriggerTime en unités de 100 ns.
- RaiseTPL / RestoreTPL : TPL_APPLICATION 4, TPL_CALLBACK 8, TPL_NOTIFY 16, TPL_HIGH_LEVEL 31 ; autres valeurs réservées.
- 7.2 Memory : le firmware possède la carte mémoire avant ExitBootServices ; un EFI image n'utilise que la mémoire allouée ; libérer avant de quitter.
- Table 7.9/7.10 (types) avant/après ExitBootServices : EfiReservedMemoryType (non utilisable), EfiLoaderCode/Data (chargeur ou OS après ExitBootServices), EfiBootServicesCode/Data (libérés après ExitBootServices), EfiRuntimeServicesCode/Data (à préserver après), EfiConventionalMemory (libre), EfiUnusableMemory (erreurs), EfiACPIReclaimMemory (tables ACPI, récupérables après activation ACPI), EfiACPIMemoryNVS (à préserver en S1–S4), EfiMemoryMappedIO (ne pas utiliser ; info via ACPI), EfiMemoryMappedIOPortSpace, EfiPalCode (Itanium), EfiPersistentMemory (NV), EfiUnacceptedMemoryType (à accepter avant usage).
- Usage typique : applications et OS loaders → EfiLoaderData ; drivers boot services → EfiBootServicesData ; drivers runtime → EfiRuntimeServicesData (uniquement en boot services). Les types 0x70000000–0x7FFFFFFF = OEM ; 0x80000000–0xFFFFFFFF = OS loaders.
- Ne jamais allouer EfiReservedMemoryType, EfiMemoryMappedIO ni EfiUnacceptedMemoryType.
- AllocatePages Type : AllocateAnyPages (0), AllocateMaxAddress (1), AllocateAddress (2). Pages = 4 KiB.
- GetMemoryMap : MemoryMapSize, MemoryMap, MapKey (à passer à ExitBootServices ; change à chaque changement), DescriptorSize (utiliser pour parcourir), DescriptorVersion = 1.
- EFI_MEMORY_DESCRIPTOR : Type (UINT32), PhysicalStart, VirtualStart, NumberOfPages, Attribute.
- Attributs (bits) : UC 0x1, WC 0x2, WT 0x4, WB 0x8, UCE 0x10, WP 0x1000, RP 0x2000, XP 0x4000, NV 0x8000, MORE_RELIABLE 0x10000, RO 0x20000, SP 0x40000, CPU_CRYPTO 0x80000, HOT_PLUGGABLE 0x100000, RUNTIME 0x8000000000000000, ISA_VALID 0x4000000000000000, ISA_MASK 0x0FFFF00000000000.
- [AGENT — IMPORTANT] Pour ExitBootServices : boucle obligatoire GetMemoryMap → ExitBootServices(MapKey) ; si la clé est périmée, refaire. Après : ne pas toucher aux régions RuntimeServices / ACPI NVS.

## Statut
- Lignes 1 à 12390 lues. Prochaine : ligne 12391 (suite 7.2.3 GetMemoryMap et 7.2.4 AllocatePool, FreePool, puis 7.3 Protocol Handler Services).

## Lignes 12391-12990 — 7.2.4 AllocatePool/FreePool, 7.3 Protocol Handler Services (Install/Uninstall/Reinstall, RegisterProtocolNotify, LocateHandle, HandleProtocol, LocateDevicePath)
- AllocatePool : PoolType ; allocations 8 octets alignées ; pages prises dans EfiConventionalMemory ; FreePool rend la mémoire en EfiConventionalMemory.
- 7.3 modèle : base de données de handles ; chaque handle = liste de paires (GUID, interface) ; un GUID ne peut être installé qu'une fois par handle (EFI_INVALID_PARAMETER sinon).
- InstallProtocolInterface(Handle*, Protocol, InterfaceType = EFI_NATIVE_INTERFACE, Interface) : Handle NULL → nouveau handle.
- UninstallProtocolInterface : tente de déconnecter les pilotes consommateurs (DisconnectController) ; EFI_ACCESS_DENIED si un agent l'utilise encore.
- ReinstallProtocolInterface : Uninstall puis Install puis ConnectController.
- RegisterProtocolNotify(Protocol, Event, Registration) ; LocateHandle(AllHandles / ByRegisterNotify / ByProtocol) ; BUFFER_TOO_SMALL met à jour BufferSize.
- HandleProtocol (ancien) : équivalent à OpenProtocol(BY_HANDLE_PROTOCOL) ; à éviter pour du nouveau code (les agents ne sont pas suivis).
- LocateDevicePath : retourne le handle le plus proche du chemin qui supporte le protocole.
- [AGENT] Pour un driver minimal : utiliser OpenProtocol/CloseProtocol, pas HandleProtocol.

## Statut
- Lignes 1 à 12990 lues. Prochaine : ligne 12991 (suite LocateDevicePath, 7.3.9 OpenProtocol, 7.3.10 CloseProtocol, OpenProtocolInformation, ConnectController, DisconnectController, ProtocolsPerHandle, LocateHandleBuffer, LocateProtocol, InstallMultipleProtocolInterfaces).

## Lignes 12991-13590 — LocateDevicePath, OpenProtocol, CloseProtocol, OpenProtocolInformation, ConnectController, DisconnectController
- LocateDevicePath : avance DevicePath sur les nœuds consommés ; instance multiple = première instance.
- OpenProtocol(Handle, Protocol, *Interface, AgentHandle, ControllerHandle, Attributes). Attributs : BY_HANDLE_PROTOCOL 0x1 ; GET_PROTOCOL 0x2 (pas de fermeture obligatoire) ; TEST_PROTOCOL 0x4 ; BY_CHILD_CONTROLLER 0x8 ; BY_DRIVER 0x10 ; EXCLUSIVE 0x20.
- Codes utiles : EFI_UNSUPPORTED si le protocole n'existe pas ; EFI_ALREADY_STARTED si BY_DRIVER déjà ouvert par le même agent ; EFI_ACCESS_DENIED si un autre pilote tient le protocole en BY_DRIVER / EXCLUSIVE.
- Exemple Driver : OpenProtocol(ctrl, &gEfiXyzIoProtocol, &io, This->DriverBindingHandle, ctrl, BY_DRIVER | EXCLUSIVE).
- CloseProtocol(Handle, Protocol, AgentHandle, ControllerHandle) : EFI_NOT_FOUND si non ouvert par cet agent.
- OpenProtocolInformation : retourne EFI_OPEN_PROTOCOL_INFORMATION_ENTRY[] (AgentHandle, ControllerHandle, Attributes, OpenCount), buffer alloué par AllocatePool, à libérer par l'appelant.
- ConnectController(Controller, DriverImageHandle[] (optionnel), RemainingDevicePath (optionnel), Recursive) : priorité de pilotes : (1) DriverImageHandle explicite ; (2) Platform Driver Override (GetDriver) ; (3) Driver Family Override (tri GetVersion) ; (4) Bus Specific Driver Override ; (5) Driver Binding Search (tri Version). Supported() puis Start() ; recommence à la première priorité après chaque succès.
- EFI_NOT_FOUND si aucun pilote ; EFI_SUCCESS si chemin restant = End Node.
- DisconnectController(Controller, DriverImageHandle optionnel, ChildHandle optionnel) : appelle Stop() ; enfants détruits d'abord si ChildHandle NULL.
- [AGENT] Pour un OS qui se passe du modèle de pilotes UEFI : ces services ne sont utiles qu'avant ExitBootServices ; pas de dépendance après.

## Statut
- Lignes 1 à 13590 lues. Prochaine : ligne 13591 (suite DisconnectController, ProtocolsPerHandle, LocateHandleBuffer (exemple), LocateProtocol, InstallMultipleProtocolInterfaces, UninstallMultipleProtocolInterfaces, 7.4 Image Services, LoadImage, StartImage, UnloadImage, Exit, ExitBootServices).

## Lignes 13591-14190 — ConnectController/DisconnectController (suite), ProtocolsPerHandle, LocateHandleBuffer, LocateProtocol, Install/UninstallMultipleProtocolInterfaces, 7.4 Image Services (LoadImage, StartImage, UnloadImage, Exit, ExitBootServices), 7.5 début
- ProtocolsPerHandle : buffer de GUID alloué par AllocatePool ; libérer avec FreePool.
- LocateHandleBuffer : même logique que LocateHandle (buffer alloué). Exemple complet de parcours de la base de handles (ProtocolsPerHandle + OpenProtocol GET + OpenProtocolInformation).
- LocateProtocol : premier handle exposant le protocole ; EFI_NOT_FOUND sinon.
- InstallMultipleProtocolInterfaces(&Handle, GUID1, If1, GUID2, If2, …, NULL) : rejette un Device Path déjà présent (EFI_ALREADY_STARTED) ; rollback en cas d'erreur. Recommandé à la place de InstallProtocolInterface.
- Table 7.30 : image types — application (EfiLoaderCode/Data, libérée au retour), boot service driver (EfiBootServicesCode/Data, libérée si retour d'erreur), runtime driver (EfiRuntimeServicesCode/Data, à allouer seulement en boot services).
- LoadImage(BootPolicy, ParentImageHandle, DevicePath, SourceBuffer, SourceSize, &ImageHandle) : soit mémoire (SourceBuffer), soit fichier via Simple File System, Load File ou Load File2. Crée EFI_LOADED_IMAGE_PROTOCOL. Peut renvoyer EFI_SECURITY_VIOLATION (Secure Boot) ; installe EFI_HII_PACKAGE_LIST_PROTOCOL si ressource PE 'HII'.
- StartImage(ImageHandle, *ExitDataSize, **ExitData) : une seule fois ; après retour, ExitData à libérer par l'appelant (FreePool).
- UnloadImage : si non démarré → unload direct ; si démarré avec Unload() → appelé ; sinon EFI_UNSUPPORTED.
- EFI_IMAGE_ENTRY_POINT(ImageHandle, SystemTable).
- Exit(ImageHandle, ExitStatus, ExitDataSize, ExitData) : ne doit pas être appelé si des images enfants sont en cours ; le buffer ExitData doit être alloué par AllocatePool.
- ExitBootServices(ImageHandle, MapKey) — RÈGLES IMPORTANTES :
  - MapKey = clé de la carte mémoire la plus récente (GetMemoryMap) ; si incorrecte → EFI_INVALID_PARAMETER et refaire GetMemoryMap + ExitBootServices.
  - Événements signalés avant retour : EFI_EVENT_GROUP_BEFORE_EXIT_BOOT_SERVICES, puis timers désactivés, puis EXIT_BOOT_SERVICES et EVT_SIGNAL_EXIT_BOOT_SERVICES.
  - Après succès : le chargeur possède toute la mémoire ; EfiBootServicesCode/Data utilisables ; plus aucun service boot ni protocole à handle.
  - Champs de la System Table mis à NULL : ConsoleInHandle, ConIn, ConsoleOutHandle, ConOut, StandardErrorHandle, StdErr, BootServices. CRC de la System Table à recalculer.
  - Après le premier ExitBootServices, ne pas appeler de service boot autre qu'allocation mémoire ; l'appel peut être répété.
  - Watchdog désactivé.
- 7.5 Services divers : SetWatchdogTimer (Timeout en secondes, 0 = désactivé, code 0x0000–0xFFFF réservé au firmware) ; le boot manager arme un watchdog de 5 minutes avant StartImage.
- CalculateCrc32 : utilisé pour System Table, Boot Services Table, Runtime Services Table, GPT.
- [AGENT — CRITIQUE] Séquence de sortie d'un OS UEFI : GetMemoryMap → ExitBootServices(MapKey) (en boucle si EFI_INVALID_PARAMETER) → recalculer CRC de la System Table → ne plus toucher aux pointeurs de boot services ni aux protocoles.

## Statut
- Lignes 1 à 14190 lues. Prochaine : ligne 14191 (suite 7.5 Miscellaneous : Stall, CopyMem, SetMem, GetNextMonotonicCount, InstallConfigurationTable, CalculateCrc32 ; puis CHAPITRE 8 Runtime Services).

## Lignes 14191-14790 — fin 7.5 (SetWatchdogTimer, Stall, CopyMem, SetMem, GetNextMonotonicCount, InstallConfigurationTable, CalculateCrc32), CHAPITRE 8 Runtime Services (8.1, 8.2 Variables début)
- SetWatchdogTimer : précision ±1 s ; pas de watchdog après ExitBootServices.
- Stall(µs) : attente active, pas de cession du CPU.
- CopyMem : gère le chevauchement (copie arrière si Destination > Source et Destination < Source+Length) ; SetMem(Buffer, Size, Value) ; réentrant.
- GetNextMonotonicCount : compteur 64 bits, poids faible volatil (remis à 0 à chaque reset), poids fort non volatil.
- InstallConfigurationTable(Guid, Table) : Guid NULL → EFI_INVALID_PARAMETER ; Table NULL = suppression ; liste allouée en EfiRuntimeServicesData.
- CalculateCrc32(Data, DataSize, &Crc32) : CRC-32 standard, DataSize 0 interdit.
- CHAPITRE 8 Runtime : tous les runtime services sont non bloquants et appelables interruptions désactivées.
- 8.1 Règles de réentrance (Table 8.1) : pas d'appel à certains services pendant un appel en cours (ex. SetVariable/GetVariable/QueryVariableInfo ... ; SetVirtualAddressMap interdit pendant toute autre opération).
- 8.1.1 NMI / Machine Check / INIT : après SetVirtualAddressMap, les appels doivent utiliser le mapping virtuel. Les fonctions GetTime, GetVariable, GetNextVariableName, QueryVariableInfo, SetVariable, UpdateCapsule, QueryCapsuleCapabilities, ResetSystem peuvent être appelées même si un appel précédent était bloqué (Table 8.2).
- 8.2 Variables : clé = (nom, VendorGuid) ; persistance requise en général.
  - Attributs (GetVariable/SetVariable) : NON_VOLATILE 0x1 ; BOOTSERVICE_ACCESS 0x2 ; RUNTIME_ACCESS 0x4 ; HARDWARE_ERROR_RECORD 0x8 ; TIME_BASED_AUTHENTICATED_WRITE_ACCESS 0x20 ; APPEND_WRITE 0x40 ; ENHANCED_AUTHENTICATED_ACCESS 0x80.
  - Après ExitBootServices : variables sans RUNTIME_ACCESS → EFI_NOT_FOUND.
  - EFI_BUFFER_TOO_SMALL : DataSize mis à la taille requise ; Attributes renseigné dans les deux cas.
  - GetNextVariableName : parcours ; ne filtre pas par GUID ; commencer avec un nom vide ; EFI_INVALID_PARAMETER si nom/GUID inexistant.
  - SetVariable : DataSize = 0 supprime (sauf attributs authentifiés/APPEND) ; nom vide interdit.
  - EFI_VARIABLE_AUTHENTICATION_2 : TimeStamp (EFI_TIME en GMT) + WIN_CERTIFICATE_UEFI_GUID (PKCS7).
  - EFI_VARIABLE_AUTHENTICATION_3 : Version = 1, Type (1 = TIMESTAMP, 2 = NONCE), MetadataSize, Flags (UPDATE_CERT = 0x1). Séquence décrite au §8.2.5.
- [AGENT] Pour un OS : les variables NV sont rares (espace limité) ; utiliser avec parcimonie. Ne pas supposer que GetNextVariableName soit filtrable.

## Statut
- Lignes 1 à 14790 lues. Prochaine : ligne 14791 (suite 8.2.3 SetVariable, QueryVariableInfo, 8.2.5–8.2.7 authentification et Hardware Error Record, 8.3 Time, 8.4 Virtual Memory, 8.5 Misc/Capsule).

## Lignes 14791-15390 — 8.2.3 SetVariable (règles d'attributs), 8.2.4 QueryVariableInfo, 8.2.5 AUTH_3, 8.2.6 AUTH_2, 8.2.7 Hardware Error Records, 8.3 Time Services, 8.4 Virtual Memory (début)
- SetVariable : règles clés : attributs différents sur une variable existante → EFI_INVALID_PARAMETER (sauf APPEND_WRITE) ; sans attribut d'accès → suppression ; RUNTIME_ACCESS implique BOOTSERVICE_ACCESS ; après ExitBootServices seules les variables RUNTIME+NON_VOLATILE sont modifiables ; TIME_BASED et ENHANCED_AUTHENTICATED sont exclusifs (EFI_INVALID_PARAMETER) ; attributs globaux (Table 3.3) doivent être exacts.
- AUTH_2 (TIME_BASED_AUTHENTICATED_WRITE_ACCESS) : Data = EFI_VARIABLE_AUTHENTICATION_2 (EFI_TIME + WIN_CERTIFICATE_UEFI_GUID PKCS7) || nouvelle valeur ; digest = hash(VariableName, VendorGuid, Attributes, TimeStamp, nouvelle valeur) (sans le NUL final) ; SHA-256/384/512 ; si SetupMode==1 et variable de politique Secure Boot → vérifications 4–5 sautées ; PK/KEK signées par la PK courante ; db/dbt/dbr/dbx signées par KEK ou PK ; OsRecovery* signées par dbr/KEK/PK ; autres = Private Authenticated Variable (EFI_SECURITY_VIOLATION si échec).
- AUTH_3 (ENHANCED_AUTHENTICATED_ACCESS) : Version 1, Type (1 TIMESTAMP / 2 NONCE), MetadataSize, Flags (UPDATE_CERT 0x1). NonceSize > 0 ; nonce tout à 0 = aléatoire généré par la plate-forme.
- QueryVariableInfo(Attributes, MaxStorage, RemainingStorage, MaxVariableSize) : EFI_UNSUPPORTED si attribut non supporté ; après ExitBootServices certains retours peuvent être EFI_INVALID_PARAMETER.
- Hardware Error Records : GUID EFI_HARDWARE_ERROR_VARIABLE = 414E6BDD-E47B-47CC-B244-BB61020CF516 ; noms HwErrRec#### avec attribut NV+BS+RT+HR ; format CPER (annexe N).
- 8.3 Time : EFI_TIME = Year(16), Month, Day, Hour, Minute, Second, Pad1, Nanosecond(32), TimeZone (INT16 en minutes, −1440..1440 ou 2047 = non spécifié), Daylight (bit 0 ADJUST_DAYLIGHT, bit 1 IN_DAYLIGHT), Pad2. EFI_TIME_CAPABILITIES : Resolution, Accuracy (ppm×1e6), SetsToZero.
- GetTime : n'est pas forcément le fuseau du RTC ; TimeZone/Daylight viennent du dernier SetTime.
- SetWakeupTime : résolution 1 s ; alarme latchée jusqu'à acquittement.
- 8.4 SetVirtualAddressMap(MemoryMapSize, DescriptorSize, DescriptorVersion, VirtualMap) : appelé UNE SEULE fois, en mode physique, par le propriétaire de la carte mémoire (le chargeur) ; tous les EVT_SIGNAL_VIRTUAL_ADDRESS_CHANGE signalés avant retour ; adresses virtuelles alignées 4 KiB ; RuntimeServices, FirmwareVendor, ConfigurationTable convertis par ConvertPointer ; recalcul du CRC des tables System et Runtime ; deuxième appel → EFI_UNSUPPORTED.
- [AGENT] Ordre OS : ExitBootServices → SetVirtualAddressMap (si mode virtuel voulu) → ConvertPointer sur les pointeurs de tables → recalcul CRC.

## Statut
- Lignes 1 à 15390 lues. Prochaine : ligne 15391 (suite 8.4.2 ConvertPointer, 8.5 Miscellaneous Runtime : ResetSystem, GetNextHighMonotonicCount, UpdateCapsule, QueryCapsuleCapabilities, puis capsule).

## Lignes 15391-15990 — 8.4.2 ConvertPointer, 8.5 Miscellaneous Runtime (ResetSystem, GetNextHighMonotonicCount, UpdateCapsule, capsules, EFI_MEMORY_RANGE_CAPSULE, QueryCapsuleCapabilities, OsIndications, capsules via fichier ESP, CapsuleReport)
- ConvertPointer(DebugDisposition, **Address) : appelé pendant SetVirtualAddressMap ; EFI_OPTIONAL_PTR = 0x1 autorise NULL.
- ResetSystem(ResetType, ResetStatus, DataSize, ResetData) : EFI_RESET_TYPE = EfiResetCold(0), EfiResetWarm(1), EfiResetShutdown(2), EfiResetPlatformSpecific(3) ; ResetData = chaîne NUL + GUID pour PlatformSpecific. Ne retourne pas.
- GetNextHighMonotonicCount : partie haute 32 bits non volatile, incrémentée à chaque reset, à chaque appel, ou au débordement du compteur bas.
- UpdateCapsule(CapsuleHeaderArray, CapsuleCount, ScatterGatherList) : capsules contiguës en mémoire virtuelle, mais leurs pages physiques peuvent être fragmentées ; EFI_CAPSULE_HEADER = CapsuleGuid, HeaderSize, Flags, CapsuleImageSize.
  - Flags : PERSIST_ACROSS_RESET = 0x10000 ; POPULATE_SYSTEM_TABLE = 0x20000 ; INITIATE_RESET = 0x40000 (ce dernier implique PERSIST).
  - EFI_CAPSULE_BLOCK_DESCRIPTOR : Length (UINT64) + union (DataBlock physique ou ContinuationPointer). Liste terminée par Length = 0 et DataBlock = 0.
  - Chaque capsule commence sur une nouvelle page ; pages pleines sauf la dernière ; pages naturellement alignées ; pas de recouvrement.
  - Après ExitBootServices, l'appelant doit faire la maintenance de cache (ScatterGatherList) avant UpdateCapsule si la MMU est désactivée.
- QueryCapsuleCapabilities(CapsuleHeaderArray, CapsuleCount, *MaximumCapsuleSize, *ResetType) ; EFI_UNSUPPORTED si capsule non supportée.
- EFI_MEMORY_RANGE_CAPSULE (GUID 0DE9F0EC-88B6-428F-977A-258F1D0E5E72) : optionnel ; zones à laisser intactes pendant le reset ; OsRequestedMemoryType ≥ 0x80000000 ; EFI_MEMORY_RANGE_CAPSULE_RESULT publié dans la table.
- OsIndications / OsIndicationsSupported (UINT64) : BOOT_TO_FW_UI 0x1 ; TIMESTAMP_REVOCATION 0x2 ; FILE_CAPSULE_DELIVERY_SUPPORTED 0x4 ; FMP_CAPSULE_SUPPORTED 0x8 ; CAPSULE_RESULT_VAR_SUPPORTED 0x10 ; START_OS_RECOVERY 0x20 ; START_PLATFORM_RECOVERY 0x40 ; JSON_CONFIG_DATA_REFRESH 0x80.
  - START_OS_RECOVERY : le firmware saute BootOrder et passe à la récupération (le bit est effacé par le firmware).
- Capsules par fichier : répertoire \EFI\UpdateCapsule sur l'ESP du périphérique de boot ; un fichier = une capsule contiguë commençant par EFI_CAPSULE_HEADER ; tri alphabétique des fichiers ; fichiers supprimés après traitement.
- Résultat : variables CapsuleNNNN avec GUID EFI_CAPSULE_REPORT_GUID = 39B68C46-F7FB-441B-B6EC-16B0F69821F3 (NV|BS|RT) ; CapsuleMax et CapsuleLast en lecture seule ; EFI_CAPSULE_RESULT_VARIABLE_HEADER = VariableTotalSize, Reserved, CapsuleGuid, CapsuleProcessed (EFI_TIME), CapsuleStatus.
- [AGENT] Pour un OS : ne pas toucher aux variables CapsuleNNNN ; supprimer après lecture ; ne pas supposer l'ordre sans tri.

## Statut
- Lignes 1 à 15990 lues. Prochaine : ligne 15991 (suite 8.5.6.1.3 JSON capsule, Table 8.23, 8.5.6.x, puis 8.5.6.1 fin, chapitre 9 Protocoles — Loaded Image).

## Lignes 15991-16590 — fin 8.5 (JSON capsule, statuts de capsule), CHAPITRE 9 Loaded Image, CHAPITRE 10 Device Path (10.1–10.3.4.3)
- EFI_LOADED_IMAGE_PROTOCOL (GUID 5B1B31A1-9562-11D2-8E3F-00A0C969723B), Revision 0x1000 : Revision, ParentHandle (NULL si lancé par le boot manager), SystemTable, DeviceHandle, FilePath, Reserved, LoadOptionsSize, LoadOptions, ImageBase, ImageSize (UINT64), ImageCodeType, ImageDataType, Unload.
- Unload() : NULL = pas de déchargement ; à enregistrer avant de retourner du point d'entrée.
- EFI_LOADED_IMAGE_DEVICE_PATH_PROTOCOL GUID BC62157E-3E33-4FEC-9920-2D3B36D750DF : même structure que Device Path.
- CHAPITRE 10 Device Path : nœud = Type (1), SubType (1), Length (2, little endian, inclut l'en-tête). Types : 0x01 Hardware, 0x02 ACPI, 0x03 Messaging, 0x04 Media, 0x05 BIOS Boot Spec, 0x7F End (SubType 0x01 = fin d'instance, 0xFF = fin de chemin complet, longueur 4). Nœuds non alignés → copier avant lecture.
- Hardware :
  - PCI (1/1, len 6) : Function (1), Device (1). Doit être précédé d'un nœud ACPI identifiant le root bus PCI.
  - PCCARD (1/2, len 5) : Function.
  - Memory Mapped (1/3, len 24) : MemoryType (4), StartAddress (8), EndAddress (8).
  - Vendor (1/4, len 20+n) : VendorGuid (16) + données.
  - Controller (1/5, len 8) : ControllerNumber (4).
  - BMC (1/6, len 13) : InterfaceType (1 : 0 unknown, 1 KCS, 2 SMIC, 3 BT), BaseAddress (8 ; bit0=1 → espace I/O).
- ACPI :
  - ACPI (2/1, len 12) : _HID (UINT32 compressé EISA), _UID (UINT32). Macro EFI_PNP_ID(ID) = (ID << 16) | 0x41D0 (EISA_PNP_ID). Ne pas comparer directement à la table ACPI.
  - Expanded ACPI (2/2, len ≥ 19) : _HID, _UID, _CID (UINT32) puis _HIDSTR, _UIDSTR, _CIDSTR (chaînes ASCII NUL) ; si une chaîne est non vide elle remplace le champ numérique.
  - ADR (2/3, len ≥ 8) : valeurs _ADR des sorties vidéo (ACPI 3.0 Table B-2), plusieurs possibles.
  - NVDIMM (2/4, len 8) : NFIT Device Handle (4).
- Messaging :
  - ATAPI (3/1, len 8) : PrimarySecondary (1), SlaveMaster (1), LUN (2).
  - SCSI (3/2, len 8) : Target (2), LUN (2).
  - Fibre Channel (3/3, len 24) : Reserved (4), WWN (8), LUN (8).
  - Fibre Channel Ex (3/21, len 24) : WWN 8 octets, LUN 8 octets ; LUN tableau d'octets = UINT64 little endian (ex. {0..7} → 0x0706050403020100) ; affichage SAM-4 : octet 0 à gauche.
- [AGENT] Pour un driver PCI sans UEFI : les chemins PCI(dev,fn) et ACPI(PNP0A03/PNP0A08, uid) servent d'identifiants stables ; à croiser avec le MCFG/ACPI de l'agent.

## Statut
- Lignes 1 à 16590 lues. Prochaine : ligne 16591 (suite 10.3.4.3 Fibre Channel Ex example, 10.3.4.4 1394, USB, SATA, NVMe, etc.).

## Lignes 16591-17190 — exemples Fibre Channel Ex, 1394, USB (types 3/5, exemples PciRoot(0)/PCI(31,2)/USB(0,0)), SATA, USB WWID, Device Logical Unit, USB Class, I2O, MAC, IPv4, IPv6, VLAN, InfiniBand, UART
- 1394 (3/4, len 16) : Reserved (4), GUID 1394 (8).
- USB (3/5, len 6) : ParentPortNumber (1), InterfaceNumber (1). Exemple : PciRoot(0)/PCI(31,2)/USB(0,0).
- SATA (3/18, len 10) : HBA port (2, 0xFFFF réservé), port multiplier (2, 0xFFFF si directement connecté), LUN (2).
- USB WWID (3/16) : InterfaceNumber (2), VendorId (2), ProductId (2), dernier 64 caractères UTF-16 du numéro de série ; l'appariement suit les règles de Load Option Processing (chapitre 3).
- Device Logical Unit (3/17, len 5) : LUN (1).
- USB Class (3/15, len 11) : VendorId (2), ProductId (2), DeviceClass (1), DeviceSubclass (1), DeviceProtocol (1) ; 0xFFFF / 0xFF = joker.
- I2O (3/6, len 8) : TID (4).
- MAC Address (3/11, len 37) : MacAddress (32, complété de zéros), IfType (1) (RFC 3232, ex. 802.3).
- IPv4 (3/12, len 27) : LocalIP (4), RemoteIP (4), LocalPort (2), RemotePort (2), Protocol (2), StaticIPAddress (1 : 0 = DHCP, 1 = statique), Gateway (4), SubnetMask (4). Spécification ancienne 19 octets : vérifier la taille.
- IPv6 (3/13, len 60) : LocalIP (16), RemoteIP (16), LocalPort, RemotePort, Protocol, IPAddressOrigin (1 : 0 manuel, 1 SLAAC, 2 DHCPv6), PrefixLength (1), GatewayIP (16).
- VLAN (3/20, len 6) : VlanId (2, 0-4094).
- InfiniBand (3/9, len 48) : Flags (4) (bit 0 IOC/Service, bit 1 Extend Boot, bit 2 Console, bit 3 Storage, bit 4 Network), PortGID (16), IOC GUID/Service ID (8), TargetPortID (8), DeviceID (8).
- UART (3/14) : voir section suivante.

## Statut
- Lignes 1 à 17190 lues. Prochaine : ligne 17191 (suite UART, UART Flow Control, SAS, iSCSI, NVMe namespace 10.3.4.21, URI, UFS, SD, Bluetooth, Wi-Fi, eMMC, DNS, NVDIMM namespace, REST, NVMe-oF).

## Lignes 17191-17790 — UART (3/14, len 19 : Reserved 4, BaudRate 8, DataBits 1, Parity 1 (0 défaut, 1 none, 2 even, 3 odd, 4 mark, 5 space), StopBits 1 (0 défaut, 1 = 1, 2 = 1.5, 3 = 2)), Vendor messaging (3/10), flux (UART Flow Control : bit 0 matériel, bit 1 Xon/Xoff), SAS (3/10 GUID d487ddb4-008b-11d9-afdc-001083ffca4d), SAS Ex (3/22, 32 octets), iSCSI (3/19) et exemples IPv4/IPv6
- Vendor GUIDs de terminal : PC-ANSI E0C14753-F9BE-11D2-9A0C-0090273FC14D ; VT-100 DFA66065-B419-11D3-9A2D-0090273FC14D ; VT-100+ 7BAEC70B-57E0-4C76-8E87-2F9E280883 43 ; VT-UTF8 AD15A0D6-8BEC-4ACF-A073-D01DE77E2D88.
- UART Flow Control : GUID 37499A9D-542F-4C89-A026-35DA142094E4, Flow_Control_Map (bit 0 matériel, bit 1 Xon/Xoff).
- SAS (3/10 vendor GUID) : SASAddress (8), Lun (8), SAS/SATA Info (2) : octet 40 bits 0–3 (0 aucune info, 1 info, 2 info+1 octet), bits 4–5 type (0 SAS interne, 1 SATA interne, 2 SAS externe, 3 SATA externe), bits 6–7 topologie (0 direct, 1 expander) ; octet 41 = numéro de baie ; RTP (2, port cible relatif, VPD 0x83).
- SAS Ex (3/22, 32 octets) : mêmes champs avec SAS address et LUN en tableaux 8 octets ; LUN {0..7} → UINT64 0x0706050403020100 ; SAM-4 : affichage octet 0 à gauche.
- iSCSI (3/19, longueur 18+n) : Protocol (2, 0 = TCP), LoginOptions (2 : digest d'en-tête bits 0–1, digest de données bits 2–3, AuthMethod bits 10–11 (0 CHAP, 2 None), CHAP_BI/UNI bit 12), LUN (8), TargetPortalGroupTag (2), TargetName (ASCII, jusqu'à 223 octets).
- Exemples iSCSI (IPv4 et IPv6) : combinent ACPI(PNP0A03) / PCI / MAC / IPv4|IPv6 / iSCSI / HD(...).
- [AGENT] À appliquer si l'agent doit démarrer depuis iSCSI : LUN et nom cible en ASCII, pas d'endian swap pour SAS Ex.

## Statut
- Lignes 1 à 17790 lues. Prochaine : ligne 17791 (suite exemples iSCSI IPv6, 10.3.4.21 NVMe namespace, URI, UFS, SD, Bluetooth, Wi-Fi, eMMC, DNS, NVDIMM namespace, REST, NVMe-oF).

## Lignes 17791-18390 — fin exemples iSCSI IPv6 et MBR ; NVMe Namespace (3/23, len 16 : NSID 4, EUI-64 8 ; 0 et 0xFFFFFFFF invalides) ; URI (3/24) ; UFS (3/25, len 6 : Target 1, LUN 1) ; SD (3/26, len 5) ; Bluetooth (3/27, len 10) ; Wi-Fi (3/28, len 36, SSID 32) ; eMMC (3/29, len 5) ; BluetoothLE (3/30, len 11) ; DNS (3/31, IsIPv6 1 + IP) ; NVDIMM Namespace (3/32, UUID 16) ; REST (3/33) ; NVMe-oF (3/34, NIDT 1 + NID 16 + SubsystemNQN UTF-8)
- NVMe Namespace (3/23) : affichage EUI-64 avec octet 7 à gauche (règle d'affichage, stockage little-endian).
- URI (3/24) : longueur 4 + n (RFC 3986).
- UFS (3/25, len 6) : PUN (1, 0 en UFS 2.0), LUN (1 : 0–7 normaux, 81h/D0h/B0h/C4h bien connus avec bit7).
- SD (3/26, len 5) : SlotNumber (1). eMMC (3/29, len 5) : SlotNumber (1).
- Bluetooth (3/27, len 10) : adresse 48 bits (6). BluetoothLE (3/30, len 11) : adresse (6) + AddressType (1 : 0 publique, 1 aléatoire).
- Wi-Fi (3/28, len 36) : SSID (32).
- DNS (3/31, len 5+n) : IsIPv6 (1 : 0 IPv4, 1 IPv6) puis une ou plusieurs adresses EFI_IP_ADDRESS.
- NVDIMM Namespace (3/32, len 20) : UUID (16) du label.
- REST Service (3/33, len 6) : AccessMode 1 (0x01 in-band, 0x02 out-of-band), RestServiceType (1 : 0x01 Redfish, 0x02 OData, 0xFF = vendeur + GUID).
- NVMe-oF (3/34, len 20+n) : NIDT (1 : 1h/2h/3h = EUI-64, NGUID, UUID), NID (16, big endian), SubsystemNQN (UTF-8, ≤ 224 octets).
- Media (4/*) commence en 10.3.5 : Hard Drive (4/1, len 42) : PartitionNumber (4 ; 0 = disque entier), PartitionStart (8, LBA), PartitionSize (8), PartitionSignature (16 : MBR 4 octets ou GUID), PartitionFormat (1 : 0x01 MBR, 0x02 GPT), SignatureType (1 : 0 aucune, 1 MBR 32 bits, 2 GUID).
- CD-ROM (4/2, len 24) : BootEntry (4, 0 = Initial/Default), PartitionStart (8, RBA), PartitionSize (8, en blocs).
- Vendor media (4/3, len 20+n) : GUID + données.
- File Path (4/4, len 4+n) : chaîne NUL ; concaténation de plusieurs nœuds avec séparateurs « \ » ; pas de double séparateur.
- Media Protocol (4/5) : GUID du protocole (16).
- [AGENT — pour le projet] Hard Drive (4/1) : PartitionFormat=GPT, SignatureType=2 (GUID) ; à utiliser pour identifier la partition ESP sans lire d'entrées par ordre physique.

## Statut
- Lignes 1 à 18390 lues. Prochaine : ligne 18391 (suite 10.3.5.5 Media Protocol, PIWG Firmware File/Volume, Relative Offset Range, RAM Disk, 10.3.6 BIOS Boot, 10.4 règles de génération, 10.5 Device Path Utilities, 10.6 Text).

## Lignes 18391-18990 — Media (fin), 10.3.5.5 Media Protocol, PIWG Firmware File/Volume, Relative Offset Range, RAM Disk (GUID types), 10.3.6 BIOS Boot Spec, 10.4 règles de génération, 10.5 Device Path Utilities (GetDevicePathSize, Duplicate, Append*, GetNextDevicePathInstance, CreateDeviceNode, IsMultiInstance), 10.6 affichage texte (début)
- Media Protocol (4/5, len 20) : GUID de protocole.
- PIWG Firmware File (4/6) et Firmware Volume (4/7) : contenu défini par UEFI PI.
- Relative Offset Range (4/8, len 24) : Reserved (4), StartingOffset (8), EndingOffset (8) (inclusif).
- RAM Disk (4/9, len 38) : StartingAddress (8), EndingAddress (8), TypeGUID (16) — EFI_VIRTUAL_DISK_GUID 77AB535A-45FC-624B-5560-F7B281D1F96E ; EFI_VIRTUAL_CD_GUID 3D5ABD30-4175-87CE-6D64-D2ADE523C4BB ; EFI_PERSISTENT_VIRTUAL_DISK_GUID 5CEA02C9-4D07-69D3-269F-4496FBE096F9 ; EFI_PERSISTENT_VIRTUAL_CD_GUID 08018188-42CD-BB48-100F-5387D53DED3D ; Instance (2, défaut 0).
- BIOS Boot Spec (5/1, len 8+n) : DeviceType (2 : 01 floppy, 02 disque dur, 03 CD-ROM, 04 PCMCIA, 05 USB, 06 réseau, 80 BEV, FF inconnu), StatusFlag (2), Description (ASCII). Non compatible Secure Boot.
- 10.4.1 Règles générales : un chemin = nœuds + End ; chaque instance se termine par End Instance (0x01), le dernier par End Entire (0xFF) ; chemin NULL = End seul ; nœuds inconnus sautables grâce à Length.
- 10.4.2 ACPI _HID/_UID : _UID = 0 si non défini dans la table ACPI ; _CRS → chemin ACPI. Table 10.64 : PCI Root Bus (PNP0A03, UID), Floppy (PNP0604, drive select), Keyboard (PNP0301, 0), Serial (PNP0501, COM), Parallel (PNP0401, LPT).
- 10.4.3 ACPI _ADR : bus EISA, IDE, PCI, PCMCIA, SMBus, SATA → chemins Hardware ou Messaging (ATAPI pour IDE, SATA pour SATA).
- 10.4.4 Hardware vs Messaging : Hardware = ressources dans le domaine de cohérence (mémoire, I/O, config PCI) ; Messaging = bus consommant des ressources du domaine (SCSI, Fibre Channel, réseau).
- 10.4.5 Media : chaque partition a son EFI_BLOCK_IO avec le même chemin que le disque plus un nœud Media (partition).
- 10.4.6 : l'extension se fait par GUID vendeur (Hardware/Messaging/Media Vendor), tout type non défini est réservé.
- 10.5 EFI_DEVICE_PATH_UTILITIES_PROTOCOL (GUID 0379BE4E-D706-437D-B037-EDB82FB772A4) : GetDevicePathSize (y compris la fin), DuplicateDevicePath, AppendDevicePath (Src1 + Src2 sans sa fin), AppendDeviceNode, AppendDevicePathInstance (insère End Instance), GetNextDevicePathInstance, CreateDeviceNode (Type, SubType, Length), IsDevicePathMultiInstance. Mémoire allouée par boot services, libérée par l'appelant.
- 10.6 Affichage : format texte standard pour outils et lecture humaine : PciRoot(0) au lieu de Acpi(PNP0A03,0) ; champs par défaut omis ; caractères réservés ' ' | < > pour le shell.
- Table 10.66 : types de paramètres : GUID (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx), Keyword, Integer (décimal ou 0x…), EISAID (3 lettres + 4 hex, ou 0), String, HexDump, IPv4 (a.b.c.d[:port]), IPv6 ([adresse]:port).
- [AGENT] Utile pour afficher les chemins de boot de l'agent dans les journaux (format texte stable).

## Statut
- Lignes 1 à 18990 lues. Prochaine : ligne 18991 (suite 10.6.1.6 Text Device Node Reference : formes textuelles PCI, PcCard, MemoryMapped, Vendor, Acpi, etc.).

## Lignes 18991-19590 — 10.6.1.6 Text Device Node Reference (formes texte canoniques de chaque nœud Hardware/ACPI/Messaging/Media)
- Hardware : VenHw(GUID, data), Ctrl(n), BMC(type, adresse 64 bits), fallback HardwarePath(subtype, data).
- ACPI : Acpi(HID, UID) (HID = EISAID : 3 lettres + 4 hex, ou 0) ; PciRoot(UID) = Acpi(PNP0A03,UID) ; PcieRoot(UID) = PNP0A08 ; Floppy = PNP0604 ; Keyboard = PNP0301 ; Serial = PNP0501 ; ParallelPort = PNP0401 ; AcpiEx(HID,CID,UID,HIDSTR,CIDSTR,UIDSTR) ; AcpiAdr(...) ; NvdimmAcpiAdr(NFIT).
- Messaging : Ata(Controller,Drive,LUN) (Primary/Secondary, Master/Slave) ; Scsi(PUN,LUN) ; Fibre(WWN,LUN) ; FibreEx(WWN,LUN) (octets affichés 0 à gauche) ; I1394(GUID) ; USB(Port,Interface) ; I2O(TID) ; Infiniband(Flags,Guid,ServiceId,TargetId,DeviceId) ; VenMsg(GUID,data) ; VenPcAnsi / VenVt100 / VenVt100Plus / VenUtf8 ; UartFlowCtrl(None|Hardware|XonXoff) ; SAS(Address,LUN,RTP,SASSATA,Location,Connect,DriveBay) ; DebugPort() ; MAC(MacAddr,IfType) ; IPv4(RemoteIp,Protocol,Type,LocalIp,Gateway,Subnet) (Type = DHCP 0 / Static 1) ; IPv6(RemoteIp,Protocol,IPAddressOrigin,LocalIp,Gateway,Subnet) (Static 0, StatelessAutoConfigure 1, StatefulAutoConfigure 2) ; Uart(Baud,DataBits,Parity,StopBits) (défauts 115200, 8, 0 = D/N/E/O/M/S, 0 = D/1/1.5/2) ; UsbClass(VID,PID,Class,SubClass,Protocol) ; variantes UsbAudio, UsbCDCControl, UsbHID, UsbImage, UsbPrinter, UsbMassStorage, UsbHub, UsbCDCData, UsbSmartCard, UsbVideo, UsbDiagnostic, UsbWireless, UsbDeviceFirmwareUpdate, UsbIrdaBridge, UsbTestAndMeasurement ; UsbWwid(VID,PID,InterfaceNumber,"WWID") ; Unit(LUN) ; Sata(HPN,PMPN,LUN) ; iSCSI(TargetName,PortalGroup,LUN,HeaderDigest,DataDigest,Authentication,Protocol) ; Vlan(VlanId) ; SasEx(...) ; NVMe(NSID,EUI) (EUI-64 avec tirets FF-FF-...) ; Uri(...) ; UFS(PUN,LUN) ; SD(SlotNumber) ; Bluetooth(BD_ADDR) ; Wi-Fi(SSID) ; eMMC(SlotNumber) ; BluetoothLE(BD_ADDR,AddressType) ; Dns(...) ; NVDIMM(UUID) ; RestService(...) ; NVMEoF(SubsystemNQN,NID).
- Media : MediaPath(subtype, data) ; HD(Partition,Type,Signature,Start,Size) (Type : MBR=1, GPT=2 par défaut ; Partition 0 = disque entier) ; CDROM(Entry,Start,Size) ; VenMedia(GUID,data) ; chemin de fichier = chaîne ; Media(GUID) ; Offset(Start,End) ; RamDisk(Start,End,Instance,TypeGuid).
- [AGENT] Les formes texte servent aux journaux de l'agent ; éviter de parser des chaînes contenant ( ) , / \ | < > (ambigu).

## Statut
- Lignes 1 à 19590 lues. Prochaine : ligne 19591 (fin 10.6.x : Device Path To/From Text Protocol, grammaire, 10.6.5, 10.7 Tables, 10.8 exemples ; chapitre 11 Driver Model).

## Lignes 19591-20190 — fin 10.6 (formes texte Media/BBS, EFI_DEVICE_PATH_TO_TEXT / FROM_TEXT), CHAPITRE 11 Driver Model : 11.1 Driver Binding (Supported, Start)
- Texte : RamDisk/VirtualDisk/VirtualCD/PersistentVirtualDisk/PersistentVirtualCD ; BBS(Type,Id,Flags) (Floppy 1, HD 2, CDROM 3, PCMCIA 4, USB 5, Network 6).
- EFI_DEVICE_PATH_TO_TEXT_PROTOCOL (GUID 8B843E20-8132-4852-90CC-551A4E4A7F1C) : ConvertDeviceNodeToText(node, DisplayOnly, AllowShortcuts), ConvertDevicePathToText(path, DisplayOnly, AllowShortcuts).
- EFI_DEVICE_PATH_FROM_TEXT_PROTOCOL (GUID 05C99A21-C70F-4AD2-8A5F-35DF3343F51E) : ConvertTextToDeviceNode, ConvertTextToDevicePath.
- 11.1 EFI_DRIVER_BINDING_PROTOCOL (GUID 18A031AB-B443-4D1A-A5C0-0C09261E9F71) : Supported, Start, Stop, Version (UINT32 : 0x0–0xF et 0xFFFFFFF0–0xFFFFFFFF réservés plate-forme/OEM ; 0x10–0xFFFFFFEF IHV ; plus haut = essayé d'abord), ImageHandle, DriverBindingHandle.
- Supported(This, ControllerHandle, RemainingDevicePath) : aucun changement de matériel ; OpenProtocol/CloseProtocol apparié ; EFI_ACCESS_DENIED si un autre agent a un accès exclusif ; EFI_ALREADY_STARTED si déjà géré (un bus peut continuer) ; EFI_UNSUPPORTED sinon.
  - Device driver : n'utilise pas RemainingDevicePath ; ouvre les protocoles en BY_DRIVER (ou BY_DRIVER|EXCLUSIVE), teste, ferme.
  - Bus driver : vérifie le premier nœud de RemainingDevicePath (End ou nœud reconnu).
- Exemple XYZ : OpenProtocol(ctrl, &gEfiXyzIoProtocol, &io, This->DriverBindingHandle, ctrl, BY_DRIVER) puis CloseProtocol.
- 11.1.3 Start : même règles ; ce qui est alloué dans Start doit être libéré dans Stop (AllocatePool/Pages, OpenProtocol, InstallProtocolInterface ↔ FreePool/FreePages, CloseProtocol, UninstallProtocolInterface).
- Start retourne : EFI_SUCCESS ; EFI_DEVICE_ERROR si erreur matérielle sans Driver Health ; EFI_OUT_OF_RESOURCES.
- Si RemainingDevicePath commence par End : le bus n'énumère pas les enfants (mais installe les protocoles d'initialisation).
- [AGENT] Pour un OS écrit sans le modèle UEFI, ces règles ne s'appliquent pas, mais elles décrivent l'ordre minimal : tester sans effet de bord, allouer et libérer symétriquement.

## Statut
- Lignes 1 à 20190 lues. Prochaine : ligne 20191 (suite exemple Start, 11.1.4 Stop, 11.2 Platform Driver Override, 11.3 Bus Specific Driver Override, 11.4 Driver Diagnostics, 11.5 Component Name, 11.6 Service Binding ...).

## Lignes 20191-20790 — 11.1.3 Start (algorithmes device/bus), exemple AbcStart, 11.1.4 Stop (règles NumberOfChildren, algorithmes, AbcStop), 11.2 Platform Driver Override (GetDriver, GetDriverPath, DriverLoaded), 11.3 Bus Specific Driver Override, 11.4 Driver Diagnostics (début)
- Start, algorithme device driver : (1) ignorer RemainingDevicePath ; (2) OpenProtocol BY_DRIVER(|EXCLUSIVE) — même attribut que Supported ; (3) erreur → CloseProtocol et retour ; (4) initialiser ; si non supporté/erreur non réparable → CloseProtocol + EFI_DEVICE_ERROR ; si réparable via Driver Health → produire Driver Health et retourner EFI_SUCCESS ; (5) allouer ; (6) InstallMultipleProtocolInterfaces ; (7) EFI_SUCCESS.
- Start, bus driver (tous les enfants au premier Start) : même début, puis découverte des enfants, un handle par enfant (InstallMultipleProtocolInterfaces + OpenProtocol BY_CHILD_CONTROLLER sur le contrôleur parent), installation éventuelle de protocoles sur le contrôleur.
- Start, bus à création partielle : si RemainingDevicePath non NULL, seul l'enfant désigné est créé ; si End → aucun enfant.
- Exemple AbcStart : AllocatePool(EfiBootServicesData, sizeof) + ZeroMem ; initialiser ; InstallMultipleProtocolInterfaces(&ControllerHandle, gEfiAbcIoProtocolGuid, &AbcIo, NULL) ; ErrorExit : FreePool + CloseProtocol.
- Stop(This, ControllerHandle, NumberOfChildren, ChildHandleBuffer) : NumberOfChildren > 0 → libérer ces enfants (bus) ; = 0 → arrêter le contrôleur ; EFI_DEVICE_ERROR si non arrêtable. Miroir exact de Start (UninstallProtocolInterface, CloseProtocol, FreePool).
- Exemple AbcStop : OpenProtocol GET_PROTOCOL → ABC_IO_PRIVATE_DATA_FROM_THIS → UninstallMultipleProtocolInterfaces → CloseProtocol → FreePool.
- 11.2 EFI_PLATFORM_DRIVER_OVERRIDE_PROTOCOL (GUID 6B30C738-A391-11D4-9A3B-0090273FC14D) : GetDriver(This, ControllerHandle, *DriverImageHandle) (NULL = premier ; EFI_NOT_FOUND = fin ; EFI_INVALID_PARAMETER si handle inconnu), GetDriverPath(This, ControllerHandle, **DriverImagePath), DriverLoaded(This, ControllerHandle, DriverImagePath, DriverImageHandle). Priorité la plus haute pour ConnectController.
- 11.3 EFI_BUS_SPECIFIC_DRIVER_OVERRIDE_PROTOCOL (GUID 3BC1B285-8A15-4A82-AABF-4D7D13FB3265) : GetDriver(This, *DriverImageHandle) ; installé par le bus sur le handle enfant (ex. PCI : pilote dans l'option ROM).
- 11.4 EFI_DRIVER_DIAGNOSTICS2_PROTOCOL (GUID 4D330321-025F-4AAC-90D8-5ED900173B63) : RunDiagnostics, SupportedLanguages (CHAR8*).
- [AGENT] La séquence Supported → Start → (Stop) est le squelette de tout pilote UEFI ; sans UEFI, le pilote NVMe/xHCI de l'agent doit reproduire la même discipline d'ouverture/fermeture des ressources.

## Statut
- Lignes 1 à 20790 lues. Prochaine : ligne 20791 (suite 11.4.2 RunDiagnostics, 11.5 Component Name, 11.6 Service Binding, 11.7 Platform to Driver Configuration, 11.8 Supported EFI Version, 11.9 Driver Family Override, 11.10 Driver Health, 11.11 Adapter Information, 11.12 AIP types).

## Lignes 20791-21390 — 11.4.2 RunDiagnostics (types Standard 0 / Extended 1 / Manufacturing 2 / Cancel 3), 11.5 Component Name2 (GetDriverName, GetControllerName, GUID 6A7A5CFF-E8D9-4F70-BADA-75AB3025CE14), 11.6 Service Binding (CreateChild, DestroyChild, algorithmes, exemple ARP), 11.7 Platform→Driver Configuration (Query, Response)
- RunDiagnostics(This, ControllerHandle, ChildHandle, DiagnosticType, Language, *ErrorType, *BufferSize, **Buffer) : EFI_UNSUPPORTED si langue/type non géré ; EFI_NOT_READY = en cours ; EFI_DEVICE_ERROR si échec ; buffer à FreePool.
- Component Name2 : GetDriverName(Language), GetControllerName(ControllerHandle, ChildHandle, Language) (EFI_UNSUPPORTED si langue absente ou contrôleur non géré) ; langue RFC 4646 (RFC 4647 pour la recherche).
- Service Binding (pas de GUID propre ; le GUID est celui du service, ex. ARP_SERVICE_BINDING, IP4, TCP4…). CreateChild(This, *ChildHandle) : allouer, installer le protocole sur le nouveau handle (ou existant), OpenProtocol BY_CHILD_CONTROLLER sur le protocole parent ; en cas d'échec de ouverture : DestroyChild et retour d'erreur. DestroyChild(This, ChildHandle) : retrouver le protocole (si absent → EFI_SUCCESS) ; éviter la récursion par un drapeau ; CloseProtocol parent ; UninstallMultipleProtocolInterfaces (si échec, rouvrir parent BY_CHILD_CONTROLLER) ; FreePool ; décrémenter le compteur.
- 11.7 EFI_PLATFORM_TO_DRIVER_CONFIGURATION_PROTOCOL (GUID 642CD590-8059-4C0A-A958-C5EC07D23C4B) : Query(This, ControllerHandle, ChildHandle, *Instance, **ParameterTypeGuid, **ParameterBlock, *Size) appelé tôt dans Start ; Response(... ConfigurationAction) apparié ; boucle jusqu'à EFI_NOT_FOUND ; action EfiPlatformConfigurationActionUnsupportedGuid si GUID inconnu.
- [AGENT] Pour un pilote de l'agent, la discipline Service Binding (créer/détruire des enfants proprement) est utile pour les contrôleurs multi-clients (ex. plusieurs namespaces NVMe).

## Statut
- Lignes 1 à 21390 lues. Prochaine : ligne 21391 (suite 11.7.3 Response, 11.8 Supported EFI Version, 11.9 Driver Family Override, 11.10 Driver Health (GetHealthStatus, Repair, algorithmes boot), 11.11 AIP, 11.12 AIP types).

## Lignes 21391-21990 — 11.7.3–11.7.4 (Response, actions de configuration, CLP GUID), 11.8 Supported EFI Version (GUID 5C198761-16A8-4E69-972C-89D67954F81D), 11.9 Driver Family Override (GUID B1EE129E-DA36-4181-91F8-04A4923766A7, ordre de priorité), 11.10 Driver Health (GUID 2A534210-9280-41D8-AE79-CADA01A2B127, états, Repair, algorithmes boot), 11.11 Adapter Information (début, GUID E5DD1403-D622-C24E-8488-C71B17F5E802)
- ConfigurationAction : None 0, StopController 1, RestartController 2, RestartPlatform 3, NvramFailed 4, UnsupportedGuid 5.
- CLP GUID 345ECC0E-00CB-4B75-BB57-1B129C47333E : CLPCommand, CLPReturnString (UTF-8), CLPCmdStatus, CLPErrorValue, CLPMsgCode (bit 15 OEM).
- Supported EFI Version : Length, FirmwareVersion (version UEFI que le pilote respecte) — à poser sur le handle image du pilote.
- Driver Family Override : GetVersion() ; priorité : Context Override > Platform Driver Override > Driver Family Override > Bus Specific > Driver Binding Search.
- Driver Health : GetHealthStatus(This, ControllerHandle, ChildHandle, *HealthStatus, **MessageList, *FormHiiHandle) ; Repair(This, ControllerHandle, ChildHandle, RepairNotify).
  - États : Healthy 0, RepairRequired 1, ConfigurationRequired 2, Failed 3, ReconnectRequired 4, RebootRequired 5.
  - Failed = terminal, non bootable (ainsi que ses enfants). ReconnectRequired → DisconnectController puis ConnectController. RebootRequired → reset après réparations.
  - Messages : EFI_DRIVER_HEALTH_HII_MESSAGE (HiiHandle, StringId, MessageCode 64 bits : 0x1–0x0FFF… réservés UEFI, 0x1000… IHV, 0x8000… plate-forme/OEM).
  - Boot Manager : boucle GetHealthStatus → Repair ou formulaire HII (EFI_FORM_BROWSER2 SendForm, formset EFI_HII_DRIVER_HEALTH_FORMSET_GUID) jusqu'à état terminal.
- Adapter Information : GetInformation, SetInformation, GetSupportedTypes ; non bloquant ; pas de cache.
- [AGENT] Driver Health = mécanisme UEFI de « réparer avant de booter » ; à ne pas confondre avec un état de contrôleur interne.

## Statut
- Lignes 1 à 21990 lues. Prochaine : ligne 21991 (suite 11.11.2 à 11.12 AIP : GET_INFO, SET_INFO, types, Network Media, Network Boot, SAN MAC, IPv6 UNDI, CDAT ; puis 12 Console).

## Lignes 21991-22590 — 11.11 AIP (GetInformation, SetInformation, GetSupportedTypes), 11.12 types (Media State, Network Boot, SAN MAC, UNDI IPv6, Media Type, CDAT), CHAPITRE 12 Console (12.1 Console I/O, Table 12.1 contrôles, 12.2 Simple Text Input Ex)
- AIP : GetInformation(This, InformationType, **Block, *Size) (EFI_UNSUPPORTED si type inconnu, EFI_NOT_FOUND si absent, bloc alloué par le service) ; SetInformation(This, Type, Block, Size) ; GetSupportedTypes(This, **Buffer, *Count).
- Types : MEDIA_STATE D7C74207-A831-4A26-B1F5-D193065CE8B6 (EFI_SUCCESS = média présent, EFI_NOT_READY = rebond, EFI_NO_MEDIA = absent) ; NETWORK_BOOT 1FBD2960-4130-41E5-94AC-D2CF037FB37C (iSCSI IPv4/IPv6, FCoE, offload, MPIO, booléens) ; SAN_MAC_ADDRESS 114DA5EF-2CF1-4E12-9BBB-C470B55205D9 (FPMA) ; UNDI_IPV6_SUPPORT 4BD56BE3-4975-4D8A-A0AD-C405204B5D4D ; MEDIA_TYPE 8484472F-71EC-411A-B39C-62CD94D9916E (1 Ethernet, 2 Wi-Fi) ; CDAT 77AF24D1-B6F0-42B9-83F5-8FE6E83EB6F0 (CdatSize + Cdat[]).
- Console : ConsoleIn = EFI_SIMPLE_TEXT_INPUT_PROTOCOL (caractères UCS-2 + scan codes) ; ConsoleOut = Simple Text Output ; pas de pointeur ni bitmap en entrée.
- Table 12.1 : U+0000 Null (ignoré), U+0008 BS (recule), U+0009 TAB, U+000A LF, U+000D CR (retour début de ligne).
- Scan code = 0x00 → caractère Unicode valide ; sinon touche spéciale (Appendix B).
- 12.2 EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL (GUID DD9E7534-7762-4698-8C14-F58517A625AA) : Reset(This, ExtendedVerification), ReadKeyStrokeEx(This, *KeyData), WaitForKeyEx (événement), SetState(This, *KeyToggleState), RegisterKeyNotify(This, *KeyData, Function, **Handle), UnregisterKeyNotify.
  - EFI_KEY_DATA = Key (scan + Unicode) + KeyState (KeyShiftState 32 bits, KeyToggleState 8 bits).
  - Shift bits : VALID 0x80000000 ; RIGHT_SHIFT 0x1 ; LEFT_SHIFT 0x2 ; RIGHT_CTRL 0x4 ; LEFT_CTRL 0x8 ; RIGHT_ALT 0x10 ; LEFT_ALT 0x20 ; RIGHT_LOGO 0x40 ; LEFT_LOGO 0x80 ; MENU 0x100 ; SYS_REQ 0x200.
  - Toggle bits : VALID 0x80 ; KEY_STATE_EXPOSED 0x40 (touches seules) ; SCROLL_LOCK 0x01 ; NUM_LOCK 0x02 ; CAPS_LOCK 0x04.
  - ReadKeyStrokeEx : EFI_NOT_READY si rien ; EFI_UNSUPPORTED si non géré.
  - RegisterKeyNotify : fonction à TPL ≤ TPL_CALLBACK.
- [AGENT] Pour une console de secours sur l'agent : ConOut->OutputString et scan codes suffisent ; pas besoin d'Ex sauf pour raccourcis Shift.

## Statut
- Lignes 1 à 22590 lues. Prochaine : ligne 22591 (suite 12.2.5–12.2.6 Unregister, 12.3 Simple Text Input, 12.4 Simple Text Output (OutputString, QueryMode, SetMode, SetAttribute, ClearScreen, SetCursorPosition, EnableCursor), 12.5 Simple Pointer).

## Lignes 22591-23190 — 12.2.5–12.2.6 (RegisterKeyNotify, UnregisterKeyNotify), 12.3 Simple Text Input (GUID 387477C1-69C7-11D2-8E39-00A0C9697 23B ; Reset, ReadKeyStroke, WaitForKey ; EFI_INPUT_KEY ScanCode + UnicodeChar), 12.4 Simple Text Output (GUID 387477C2-69C7-11D2-8E39-00A0C969723B) : Reset, OutputString, TestString, QueryMode, SetMode, SetAttribute, ClearScreen, SetCursorPosition, EnableCursor, SIMPLE_TEXT_OUTPUT_MODE
- Simple Text Input : même contenu que Ex sans état (scan + Unicode). EFI_NOT_READY si rien.
- Simple Text Output : Mode (MaxMode, Mode, Attribute, CursorColumn, CursorRow, CursorVisible) ; mode texte 80x25 obligatoire (mode 0), 80x50 = mode 1 (sinon EFI_UNSUPPORTED) ; mode CurrentMode = -1 si non valide à la connexion.
- Caractères de dessin (box-drawing U+2500–U+256C, blocs U+2588/U+2591, formes géométriques U+25B2/25BA/25BC/25C4, flèches U+2191/U+2193) obligatoires.
- OutputString : Table 12.10 : NUL ignoré ; BS → recule si pas au bord gauche ; LF → descend (scroll si en bas, sans mise à jour du curseur) ; CR → début de ligne ; autres → affiché puis curseur avance, retour à la ligne automatique (équivalent CR+LF). Codes : EFI_SUCCESS, EFI_DEVICE_ERROR, EFI_UNSUPPORTED, EFI_WARN_UNKNOWN_GLYPH.
- TestString : EFI_UNSUPPORTED si un caractère n'est pas rendable.
- Reset : curseur (0,0) et écran effacé.
- [AGENT] Pour un diagnostic minimal : OutputString avec CR+LF explicites ; mode 0 garanti.

## Statut
- Lignes 1 à 23190 lues. Prochaine : ligne 23191 (suite 12.4.6 SetMode, 12.4.7 SetAttribute (couleurs), 12.4.8 ClearScreen, 12.4.9 SetCursorPosition, 12.4.10 EnableCursor, 12.5 Simple Pointer, 12.6 device paths, 12.7 Absolute Pointer, 12.8 Serial I/O).

## Lignes 23191-23790 — 12.4.6–12.4.10 (SetMode, SetAttribute : couleurs EFI_BLACK 0x0 … EFI_WHITE 0xF, EFI_BACKGROUND_* 0x00–0x70 ; EFI_TEXT_ATTR(fg,bg) = fg | bg<<4 ; ClearScreen ; SetCursorPosition ; EnableCursor), 12.5 Simple Pointer (GUID 31878C87-0B75-11D5-9A4F-0090273FC14D ; Reset, GetState, WaitForInput ; EFI_SIMPLE_POINTER_MODE, EFI_SIMPLE_POINTER_STATE), 12.6 chemins de périphériques de pointeur (PS/2, série, USB), 12.7 Absolute Pointer (début, GUID 8D59D32B-C655-4AE9-9B15-F25904992A43)
- SetMode : écran effacé, curseur (0,0). SetAttribute : fond/texte ; le masque peut être posé même hors mode valide.
- ClearScreen : curseur (0,0). SetCursorPosition : coordonnée 0 ≤ col < colonnes, 0 ≤ row < lignes du mode courant.
- Simple Pointer : ResolutionX/Y/Z en comptes par mm (0 = axe absent) ; LeftButton/RightButton ; GetState renvoie EFI_NOT_READY si rien n'a changé ; RelativeMovementX/Y/Z signés, à interpréter avec la résolution.
- Chemins : PS/2 = ACPI(PNP0A03,0)/PCI(7,0)/ACPI(PNP0F03,0) ; série = ACPI(PNP0A03,0)/PCI(7,0)/ACPI(PNP0501,0)/UART(1200,N,8,1)/ACPI(PNP0F01,0) ; USB = ACPI(PNP0A03,0)/PCI(7,2)/USB(0,0).
- Absolute Pointer : écrans tactiles ; 1 à 3 axes ; Z optionnel = pression ; contact (touch-active) obligatoire ; bouton secondaire optionnel.
- [AGENT] Les chemins PCI(7,2)/USB(0,0) montrent comment l'agent pourrait nommer un périphérique xHCI en UEFI.

## Statut
- Lignes 1 à 23790 lues. Prochaine : ligne 23791 (suite 12.7.2 Reset/GetState Absolute, 12.8 Serial I/O (GUID, Reset, SetAttributes, SetControl, GetControl, Write, Read), 12.9 GOP (Blt, QueryMode, SetMode), EDID).

## Lignes 23791-24320 — 12.7.1–12.7.3 (Absolute Pointer : Mode AbsoluteMin/Max X,Y,Z, Attributes SupportsAltActive 0x1, SupportsPressureAsZ 0x2 ; State CurrentX/Y/Z, ActiveButtons TouchActive 0x1, AltActive 0x2), 12.8 Serial I/O (GUID BB25CF6F-F1D4-11D2-9A0C-0090273FC1FD ; révision 0x10000 / 1.1 = 0x10001 ; Reset, SetAttributes, SetControl, GetControl, Write, Read ; SERIAL_IO_MODE ; EFI_PARITY_TYPE, EFI_STOP_BITS_TYPE ; device-type GUID terminal 6AD9A60F-5815-4C7C-8A10-5053D2BF7A1B), 12.9 début Graphics Output
- Absolute Pointer : GetState renvoie EFI_NOT_READY si inchangé ; Reset ; WaitForInput.
- Serial I/O : défauts 115200 baud, FIFO 1 octet, timeout 1 000 000 µs, no parity, 8 bits, 1 stop. Contrôle de flux à la charge du logiciel (RTS/DTR par SetControl ; CTS/DSR/RI/CD par GetControl). Pilote en polling : lire assez souvent pour ne pas perdre de caractères.
- Bits de contrôle : CLEAR_TO_SEND 0x10 ; DATA_SET_READY 0x20 ; RING_INDICATE 0x40 ; CARRIER_DETECT 0x80 ; REQUEST_TO_SEND 0x02 ; DATA_TERMINAL_READY 0x01 ; INPUT_BUFFER_EMPTY 0x100 ; OUTPUT_BUFFER_EMPTY 0x200 ; HARDWARE_LOOPBACK 0x1000 ; SOFTWARE_LOOPBACK 0x2000 ; HARDWARE_FLOW_CONTROL 0x4000 ; seuls RTS, DTR, loopbacks et flux matériel sont écrivables.
- Write / Read : BufferSize en entrée/sortie ; EFI_TIMEOUT si délai dépassé ; Read s'arrête à erreur/overrun.
- Identification de périphérique série : pilote plate-forme (Version ≥ 0xFFFFFFF0) renseigne DeviceTypeGuid ; les pilotes de terminal supposent un terminal si révision 1.0 ou DeviceTypeGuid NULL.
- 12.9 Graphics Output (GUID 9042A9DE-23DC-4A38-96FB-7ADED080516A) : QueryMode, SetMode (efface en noir), Blt, Mode. Buffer Blt = pixels 32 bits BGRR (EFI_GRAPHICS_OUTPUT_BLT_PIXEL) ; index = Y × Width + X ; (0,0) en haut à gauche.
- [AGENT] Un accès direct au framebuffer (Mode->FrameBufferBase) est possible après ExitBootServices si le chargeur le transmet ; à noter pour un affichage bare-metal.

## Statut
- Lignes 1 à 24320 lues. Prochaine : ligne 24321 (suite GOP : Blt Operation table, Mode, QueryMode, SetMode, Blt ; EDID ; PCI rules; 13 Media Access).

## Lignes 24321-24920 — 12.9 Graphics Output, 12.9.2 à 12.9.2.7 (EDID, GetEdid), 12.10 Rules for PCI/AGP, début 13 Media Access (Load File)
- GOP : GUID 9042A9DE-23DC-4A38-96FB-7ADED080516A. Membres : QueryMode, SetMode, Blt, Mode (pointeur vers EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE).
- EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE : MaxMode, Mode (0..MaxMode-1), Info, SizeOfInfo, FrameBufferBase, FrameBufferSize (= PixelsPerScanLine × VerticalResolution × PixelElementSize).
- EFI_GRAPHICS_OUTPUT_MODE_INFORMATION : Version (0), HorizontalResolution, VerticalResolution, PixelFormat, PixelInformation (EFI_PIXEL_BITMASK), PixelsPerScanLine.
- Formats EFI_GRAPHICS_PIXEL_FORMAT : PixelRedGreenBlueReserved8BitPerColor (0), PixelBlueGreenRedReserved8BitPerColor (1), PixelBitMask (2), PixelBltOnly (3), PixelFormatMax.
- [AGENT] Règle critique : l'adresse de la pixel (x,y+1) = base + PixelElementSize × PixelsPerScanLine, PAS × HorizontalResolution (padding possible). Utiliser PixelsPerScanLine.
- QueryMode : ModeNumber hors [0, MaxMode-1] → EFI_INVALID_PARAMETER ; renvoie Info alloué par le callee (libérer avec FreePool).
- SetMode : mode non supporté → EFI_UNSUPPORTED ; succès → écran visible effacé en noir.
- Blt : BltBuffer ; opérations EfiBltVideoFill (1 pixel répliqué sur le rectangle, Delta ignoré), EfiBltVideoToBltBuffer, EfiBltBufferToVideo, EfiBltVideoToVideo (pas de buffer, pas de limite de chevauchement). Delta = octets par ligne du buffer quand sous-rectangle. Erreurs : EFI_INVALID_PARAMETER, EFI_DEVICE_ERROR.
- EFI_GRAPHICS_OUTPUT_BLT_PIXEL : Blue, Green, Red, Reserved (4 octets).
- EDID : EFI_EDID_DISCOVERED_PROTOCOL (GUID 1C0C34F6-D380-41FA-A049-8AD06C1A66AA) et EFI_EDID_ACTIVE_PROTOCOL (BD8C1056-9F36-44EC-92A8-A6337F817986) : SizeOfEdid (0 ou ≥128) + Edid. EFI_EDID_OVERRIDE_PROTOCOL (48ECB431-FB72-45C0-A922-F458FE040BD5) : GetEdid ; attributs DONT_OVERRIDE 0x01, ENABLE_HOT_PLUG 0x02.
- 12.10 Rules PCI/AGP : pilote GOP suit le UEFI driver model ; Start() ne doit rien modifier de visible ; crée un child handle par sortie physique (ACPI _ADR), EDID, combinaisons optionnelles ; Stop() défait tout ; GetControllerName requis.
- Modes minimaux : 800×600×32 ou 640×480×32, format RGB ou BGR.
- 13.1 Load File Protocol : GUID 56EC3091-954C-11D2-8E3F-00A0C9693B72 ; LoadFile. Ordre de chargement : d'abord Simple File System, sinon Load File ; le LoadFile implémente sa propre politique de File Path.

## Statut
- Lignes 1 à 24920 lues. Prochaine : ligne 24921 (13.1.1 suite LoadFile, Load File 2 ; 13.2 Simple File System...).

## Lignes 24921-25219 — 13.1.2 LoadFile (suite), 13.2 Load File 2, 13.3 File System Format, 13.3.1 System Partition, 13.3.2 Partition Discovery, 13.3.4 Media Formats, 13.4 Simple File System (début)
- LoadFile : BootPolicy TRUE = boot manager ; FALSE = fichier exact (EFI_NOT_FOUND sinon). PXE + BootPolicy FALSE → EFI_UNSUPPORTED. BufferSize NULL → EFI_INVALID_PARAMETER ; Buffer NULL → renvoie la taille. EFI_BUFFER_TOO_SMALL : BufferSize requis, aucune donnée copiée. EFI_WARN_FILE_SYSTEM : Buffer contient un FS UEFI (disque RAM enregistré).
- Load File 2 : GUID 4006C0C1-FCB3-403E-996D-4A6C8724E06D. Utilisé par LoadImage quand BootOption FALSE et pas de Simple File System ; BootPolicy toujours FALSE.
- 13.3 Format FS : FAT32 pour partition système, FAT12/FAT16 pour médias amovibles ; le firmware doit supporter les trois. Noms LFN sensibles à la casse en stockage, unicité insensible à la casse. Seul le sous-ensemble UCS-2 est reconnu pour le tri.
- Arborescence : \EFI\<vendeur>\...; un seul binaire EFI par architecture et par sous-répertoire vendeur. Médias amovibles : \EFI\BOOT\BOOT{short}.EFI (BOOTX64.EFI pour x64).
- Partition Discovery, ordre : 1) GPT, 2) ISO-9660/El Torito, 3) MBR legacy (LBA 0), 4) rien. Une entrée GPT avec bit 1 d'attribut = 1 ne produit pas de BlockIO logique ; ce bit exclut l'ESP. Le firmware ne exécute pas le code MBR.
- El Torito Platform ID 0xEF = ESP ; Sector Count 0 ou 1 → partition jusqu'à la fin du CD.
- [AGENT] Pour un chargeur bare-metal : détecter l'ESP par GUID (C12A7328-F81F-11D2-BA4B-00A0C93EC93B) et vérifier le FAT ; ne pas se fier au code MBR.
- 13.4 Simple File System : GUID 0964E5B2-6459-11D2-8E39-00A0C9692 3B (vérifier la valeur exacte dans la spec : {0x0964e5b22,...} est une coquille de mise en page, valeur réelle 0x964E5B22). Révision 0x00010000. Membres : Revision, OpenVolume. OpenVolume renvoie la racine ; EFI_MEDIA_CHANGED si média changé ; EFI_UNSUPPORTED si FS différent.

## Statut
- Lignes 1 à 25219 lues. Prochaine : ligne 25220 (13.4.2 suite OpenVolume status codes, 13.5 File Protocol).

## Lignes 25220-25519 — 13.4.2 OpenVolume (codes), 13.5 File Protocol (Revision 0x10000 / 0x20000, membres Open, Close, Delete, Read, Write, GetPosition, SetPosition, GetInfo, SetInfo, Flush, OpenEx, ReadEx, WriteEx, FlushEx), 13.5.2 Open, 13.5.3 Close, 13.5.4 Delete, 13.5.5 Read, 13.5.6 Write (début)
- OpenVolume codes : EFI_SUCCESS, EFI_UNSUPPORTED, EFI_NO_MEDIA, EFI_DEVICE_ERROR, EFI_VOLUME_CORRUPTED, EFI_ACCESS_DENIED, EFI_OUT_OF_RESOURCES, EFI_MEDIA_CHANGED.
- Modes d'ouverture : READ 0x1 ; WRITE 0x2 ; CREATE 0x8000000000000000. Combinaisons valides seulement : R, RW, CRW.
- Attributs : READ_ONLY 0x1, HIDDEN 0x2, SYSTEM 0x4, RESERVED 0x8, DIRECTORY 0x10, ARCHIVE 0x20, VALID_ATTR 0x37.
- Open : chemins "\" (racine du volume), "" séparateur, "." courant, ".." parent (erreur si racine). CREATE : ouvre si existe, crée sinon ; échec si cible existante non-répertoire mais attendue répertoire. Codes : EFI_NOT_FOUND, EFI_WRITE_PROTECTED, EFI_VOLUME_FULL, EFI_ACCESS_DENIED, EFI_OUT_OF_RESOURCES, EFI_INVALID_PARAMETER.
- [AGENT] Règle : tant qu'un handle fichier est ouvert, ne pas accéder directement au BlockIO/DiskIO sous-jacent (incohérence de cache).
- Close : flush des données sales, fermeture ; attend les E/S asynchrones. Retourne toujours EFI_SUCCESS.
- Delete : ferme dans tous les cas ; EFI_WARN_DELETE_FAILURE si suppression impossible (handle fermé quand même).
- Read : fichier normal lit au curseur, tronqué en fin de fichier, curseur avancé. Répertoire : renvoie une entrée EFI_FILE_INFO ; si buffer trop petit → EFI_BUFFER_TOO_SMALL sans avancer, BufferSize = taille requise ; fin de répertoire = lecture de taille 0.
- Write : écrit au curseur, agrandit le fichier ; écriture partielle seulement en cas d'erreur ; écriture sur répertoire ouvert → EFI_UNSUPPORTED.

## Statut
- Lignes 1 à 25519 lues. Prochaine : ligne 25520 (suite Write, SetPosition, GetInfo, SetInfo, Flush, EFI_FILE_INFO).

## Lignes 25520-25819 — 13.5.6 Write (fin), 13.5.7 OpenEx, 13.5.8 ReadEx, 13.5.9 WriteEx, 13.5.10 FlushEx, 13.5.11 SetPosition, 13.5.12 GetPosition, 13.5.13 GetInfo, 13.5.14 SetInfo (début)
- Révision 2 (0x00020000) ajoute OpenEx/ReadEx/WriteEx/FlushEx : EFI_FILE_IO_TOKEN { Event, Status, BufferSize, Buffer }. Event NULL = bloquant ; sinon non-bloquant avec signal à la fin. Si l'appel retourne une erreur, l'Event n'est PAS signalé.
- [AGENT] En non-bloquant, le curseur avance dans l'ordre de soumission ; Status du token lu après signal.
- ReadEx / Write : mêmes règles de curseur qu'en Read/Write. EFI_OUT_OF_RESOURCES si file d'attente pleine.
- FlushEx : flush de toutes les écritures soumises avant.
- SetPosition : absolu uniquement ; seek au-delà de la fin autorisé (écriture agrandit) ; 0xFFFFFFFFFFFFFFFF = fin de fichier ; sur répertoire seule position 0 est admise (réinitialise l'énumération). EFI_UNSUPPORTED si position non nulle sur répertoire.
- GetPosition : EFI_UNSUPPORTED sur répertoire.
- GetInfo : InformationType = GUID (EFI_FILE_INFO, EFI_FILE_SYSTEM_INFO...). EFI_UNSUPPORTED si type inconnu ; EFI_BUFFER_TOO_SMALL avec BufferSize requis.

## Statut
- Lignes 1 à 25819 lues. Prochaine : ligne 25820 (SetInfo suite, Flush, EFI_FILE_INFO, EFI_FILE_SYSTEM_INFO, EFI_FILE_SYSTEM_VOLUME_LABEL).

## Lignes 25820-26119 — 13.5.14 SetInfo (fin), 13.5.15 Flush, 13.5.16 EFI_FILE_INFO, 13.5.17 EFI_FILE_SYSTEM_INFO, 13.5.18 EFI_FILE_SYSTEM_VOLUME_LABEL, 13.6 Tape I/O (début)
- SetInfo : un fichier ouvert en lecture seule peut passer en lecture-écriture seulement via EFI_FILE_INFO_ID en ne modifiant que Attribute. Codes : EFI_ACCESS_DENIED (renommage vers nom existant, changement de DIRECTORY, taille de répertoire), EFI_BAD_BUFFER_SIZE, EFI_VOLUME_FULL, EFI_WRITE_PROTECTED.
- Flush : EFI_SUCCESS, EFI_NO_MEDIA, EFI_DEVICE_ERROR, EFI_VOLUME_CORRUPTED, EFI_WRITE_PROTECTED, EFI_ACCESS_DENIED, EFI_VOLUME_FULL.
- EFI_FILE_INFO_ID GUID 09576E92-6D3F-11D2-8E39-00A0C969723B. Structure : Size, FileSize, PhysicalSize, CreateTime, LastAccessTime, ModificationTime (EFI_TIME chacun), Attribute, FileName[] (nul). Règles SetInfo : FileSize ignoré sur répertoire ; PhysicalSize ignoré ; bit DIRECTORY non modifiable ; zéro dans un champ temps = ignoré.
- EFI_FILE_SYSTEM_INFO_ID GUID 09576E93-6D3F-11D2-8E39-00A0C969723B : Size, ReadOnly (BOOLEAN), VolumeSize, FreeSpace, BlockSize, VolumeLabel[] ; seul VolumeLabel est modifiable (via SetInfo) ; obtenu sur le handle racine.
- EFI_FILE_SYSTEM_VOLUME_LABEL_ID GUID DB47D7D3-FE81-11D3-9A35-00903F3FC14D : VolumeLabel[].
- 13.6 Tape I/O (EFI_TAPE_IO_PROTOCOL GUID 1E93E633-D65A-459E-AB84-93D9EC266D18) : TapeRead, TapeWrite, TapeRewind, TapeSpace, TapeWriteFM, TapeReset. Hors périmètre agent (non prévu pour bare-metal USB/NVMe). Lu pour complétude.
- [AGENT] Taille de bloc tape courante usuelle : 512 à 32768 ; EFI_END_OF_FILE = filemark.

## Statut
- Lignes 1 à 26119 lues. Prochaine : ligne 26120 (suite Tape : TapeSpace, TapeWriteFM, TapeReset, puis 14 Protocols – Miscellaneous).

## Lignes 26120-26419 — 13.6.2.4 TapeRewind (fin), 13.6.2.5 TapeSpace, 13.6.2.6 TapeWriteFM, 13.6.2.7 TapeReset, 13.6.3 Tape Header, 13.7 Disk I/O (début)
- Tape : TapeSpace Direction > 0 avance, < 0 recule ; Type 0 = BLOCK, 1 = FILEMARK. Fin de média → EFI_END_OF_MEDIA. TapeWriteFM Count ; TapeReset(ExtendedVerification) réinitialise le bus parent si TRUE.
- Tape Header : Signature "EFI BOOT" 0x544F4F4220494645, Revision 1, taille 1024, CRC, TapeGUID, TapeType, TapeUnique, BLLocation, BLBlocksize, BLFilesize, OSVersion[40], AppVersion[40], CreationDate[10], CreationTime[10], SystemName[256], TapeTitle[120], pad[468]. Dans les 20 premiers blocs ; little-endian. Hors périmètre.
- 13.7 Disk I/O : GUID CE345171-BA0B-11D2-8E4F-00A0C9693B72 ; Revision 0x00010000 ; ReadDisk, WriteDisk (offset arbitraire, pas aligné sur bloc). Firmware ajoute Disk I/O à chaque Block I/O. Systèmes de fichiers requis : FAT12/16/32, MBR (optionnel mais si présent, un logical device par partition).
- [AGENT] Disk I/O = pont pratique pour lire des offsets arbitraires ; ne pas écrire via Disk I/O sans flush (FlushBlocks du Block I/O).

## Statut
- Lignes 1 à 26419 lues. Prochaine : ligne 26420 (13.7.2 ReadDisk/WriteDisk, puis 13.8 Block I/O...).

## Lignes 26420-26719 — 13.7.2 ReadDisk, 13.7.3 WriteDisk, 13.8 Disk I/O 2 (GUID 151C8EAE-7F2C-472C-9E54-98281 94F6A88, rév 0x20000 ; Cancel, ReadDiskEx, WriteDiskEx, FlushDiskEx ; EFI_DISK_IO2_TOKEN), 13.9.1 Block I/O (début)
- ReadDisk / WriteDisk : MediaId doit correspondre au média courant (sinon EFI_MEDIA_CHANGED) ; tous les octets ou erreur ; EFI_NO_MEDIA ; EFI_INVALID_PARAMETER si adresse hors device. WriteDisk : EFI_WRITE_PROTECTED.
- Disk I/O 2 : Cancel → tokens en vol signalés avec EFI_ABORTED ; après Cancel, libérer tokens/buffers. Token NULL = bloquant. Erreur sur l'appel → Event non signalé. FlushDiskEx : flush vers le périphérique physique.
- [AGENT] Pour un driver bare-metal, préférer Block I/O (LBA + MediaId) à Disk I/O si on lit par secteur ; garder MediaId et tester EFI_MEDIA_CHANGED.
- Block I/O : GUID 964E5B21-6459-11D2-8E39-00A0C969723B ; révisions 0x00020001 (REVISION2) et (2<<16)|31 = 0x0002001F (REVISION3) — lire la suite pour les membres.

## Statut
- Lignes 1 à 26719 lues. Prochaine : ligne 26720 (13.9.1 Block I/O Protocol Interface Structure, Media, ReadBlocks, WriteBlocks, FlushBlocks).

## Lignes 26720-27018 — 13.9.1 Block I/O (structure, EFI_BLOCK_IO_MEDIA), 13.9.2 Reset, 13.9.3 ReadBlocks, 13.9.4 WriteBlocks, 13.9.5 FlushBlocks, 13.10 Block I/O 2 (début)
- Block I/O : GUID 964E5B21-6459-11D2-8E39-00A0C969723B. Membres : Revision, Media*, Reset, ReadBlocks, WriteBlocks, FlushBlocks (optionnel).
- EFI_BLOCK_IO_MEDIA : MediaId (UINT32), RemovableMedia, MediaPresent, LogicalPartition, ReadOnly, WriteCaching, BlockSize, IoAlign, LastBlock ; rév2 : LowestAlignedLba, LogicalBlocksPerPhysicalBlock ; rév3 : OptimalTransferLengthGranularity.
- [AGENT] BlockSize = octets par bloc logique (ATA words 117-118 / SCSI READ CAPACITY 16) ; LastBlock = dernier LBA (nombre de blocs - 1).
- [AGENT] IoAlign : 0 ou 1 = pas de contrainte ; sinon puissance de 2 et adresse du buffer multiple de IoAlign → EFI_INVALID_PARAMETER si non aligné.
- ReadBlocks / WriteBlocks : BufferSize multiple de BlockSize ; EFI_NO_MEDIA et EFI_MEDIA_CHANGED DOIVENT être retournés même si LBA/BufferSize/Buffer invalides (sonde de changement de média). EFI_BAD_BUFFER_SIZE si non multiple. EFI_WRITE_PROTECTED en écriture.
- FlushBlocks : toutes les écritures précédentes doivent être physiquement écrites avant SUCCESS (y compris cache du device).
- Block I/O 2 : GUID A77B2472-E282-4E9F-A245-C2C0E27BBCC1 ; Media, Reset (ResetEx), ReadBlocksEx, WriteBlocksEx, FlushBlocksEx (version asynchrone à token).

## Statut
- Lignes 1 à 27018 lues. Prochaine : ligne 27019 (13.10.2 suite Reset, ReadBlocksEx, WriteBlocksEx, FlushBlocksEx, puis 13.11 SCSI / ATA / NVMe pass-thru).

## Lignes 27019-27318 — 13.10.2 Reset (suite), 13.10.3 ReadBlocksEx, 13.10.4 WriteBlocksEx, 13.10.5 FlushBlocksEx, 13.11 Inline Cryptographic Interface (début)
- Block I/O 2 Reset : abandonne les E/S non-bloquantes en vol avec TransactionStatus = EFI_ABORTED ; tokens/buffers libérables après retour.
- ReadBlocksEx / WriteBlocksEx : token NULL-Event = bloquant ; si erreur à l'appel, Event non signalé. EFI_BAD_BUFFER_SIZE, EFI_INVALID_PARAMETER (LBA ou alignement), EFI_OUT_OF_RESOURCES.
- FlushBlocksEx : même sémantique que FlushBlocks (persistance physique).
- Inline Crypto (ICI) : GUID A00490BA-3F1A-4B4C-AB90-4FA99726A1E8 ; membres Media, Reset, GetCapabilities, SetConfiguration, GetConfiguration, ReadExtended, WriteExtended, FlushBlocks. Usage FDE : ESP laissé en clair, lu via Block I/O standard.
- Algorithmes : AES_XTS (2F87BA6A-5C04-4385-A780-F3BF78A97BEC, IEEE 1619-2007, IV = CryptoBlockNumber), AES_CBC_MICROSOFT_BITLOCKER (689E4C62-70BF-4CF3-88BB-33B318268670, IV = CryptoBlockByteOffset).
- [AGENT] Hors périmètre sauf si disque chiffré BitLocker/FDE ; pour un chargeur bare-metal, ne pas tenter de contourner.

## Statut
- Lignes 1 à 27318 lues. Prochaine : ligne 27319 (suite ICI : GetCapabilities, SetConfiguration, ReadExtended, WriteExtended).

## Lignes 27319-27618 — 13.11.1 suite (IV_INPUT, GetCapabilities structure, Configuration entries), 13.11.2 Reset, 13.11.3 GetCapabilities, 13.11.4 SetConfiguration, 13.11.5 GetConfiguration, 13.11.6 ReadExtended (début)
- EFI_BLOCK_IO_CRYPTO_IV_INPUT : Header commun avec InputSize (taille totale du buffer IV). Capability GUID doit être un seul bit de CryptoBlockSizeBitMask ; bit j = bloc de 2^j octets.
- Capacités : Supported, KeyCount, CapabilityCount, Capabilities[]. INDEX_ANY = 0xFFFFFFFFFFFFFFFF.
- GetCapabilities : EFI_BUFFER_TOO_SMALL avec CapabilityCount requis ; EFI_NO_RESPONSE ; EFI_INVALID_PARAMETER si NULL.
- SetConfiguration : atomique (tout ou rien) ; si entrée déjà occupée par un autre owner → échec, rien modifié ; EFI_OUT_OF_RESOURCES si pas assez d'index libres. Doit être appelé avant tout I/O crypto. Re-run possible (couper les IO concernés pendant changement).
- GetConfiguration : ne renvoie jamais la clé, seulement Index, KeyOwnerGuid, Capability ; index inutilisés → INDEX_ANY.
- [AGENT] Hors périmètre usuel (UFS inline crypto) ; à noter seulement : une clé n'est jamais relue.

## Statut
- Lignes 1 à 27618 lues. Prochaine : ligne 27619 (ReadExtended suite, WriteExtended, FlushBlocks ICI, 13.12 suite protocols media).

## Lignes 27619-27918 — 13.11.6 ReadExtended (fin), 13.11.7 WriteExtended, 13.11.8 FlushBlocks ICI, 13.12 Erase Block Protocol, 13.13.1 ATA Pass Thru (début)
- ReadExtended / WriteExtended : Index optionnel (absent = pas de chiffrement/déchiffrement) ; CryptoIvInput obligatoire si Index. Token = EFI_BLOCK_IO_CRYPTO_TOKEN {Event, TransactionStatus}.
- FlushBlocks ICI : le chiffrement ne se fait jamais au flush ; tout doit être chiffré dans WriteExtended.
- Erase Block : GUID 95A9A93E-A86E-4926-AAEF-9918E772D987, révision (2<<16)|60 ; EraseLengthGranularity ; EraseBlocks(MediaId, LBA, Token, Size). Size multiple de la granularité physique. Après SUCCESS, lectures = valeur initialisée.
- [AGENT] Effacement NVMe/ATA : ne jamais l'utiliser sur le disque interne sans accord explicite (voir règle « rien écrit sur disque »).
- ATA Pass Thru : GUID 1D3DE7F0-0807-424F-AA69-11A54E19A46F. Commandes ATA sur disques ATA ; ATAPI via EXT_SCSI_PASS_THRU.

## Statut
- Lignes 1 à 27918 lues. Prochaine : ligne 27919 (suite ATA Pass Thru : structures, ExecutePassThrough...).

## Lignes 27919-28218 — 13.13.1 ATA Pass Thru (structures), 13.13.2 PassThru (début), EFI_ATA_COMMAND_BLOCK, EFI_ATA_STATUS_BLOCK
- EFI_ATA_PASS_THRU_MODE : Attributes (PHYSICAL 0x1, LOGICAL 0x2, NONBLOCKIO 0x4), IoAlign. Membres : Mode, PassThru, GetNextPort, GetNextDevice, BuildDevicePath, GetDevice, ResetPort, ResetDevice.
- Un contrôleur non-RAID met PHYSICAL+LOGICAL ; RAID : deux interfaces séparées. Ni PHYSICAL ni LOGICAL = configuration illégale.
- PATA mapping : Primary Master = port 0 / PM 0 ; Primary Slave = 0 / 1 ; Secondary Master = 1 / 0 ; Secondary Slave = 1 / 1 (table 13.45 : ligne ambiguë dans le texte, vérifier).
- PassThru(Port, PortMultiplierPort (0xFFFF si pas de PM), Packet, Event) ; Event ignoré si non-bloquant non supporté.
- EFI_ATA_PASS_THRU_COMMAND_PACKET : Asb (sense), Acb (Command Block), Timeout (100 ns, 0 = infini), InDataBuffer / OutDataBuffer, InTransferLength / OutTransferLength (EFI_BAD_BUFFER_SIZE si trop grand), Protocol, Length.
- EFI_ATA_COMMAND_BLOCK : Reserved1[2], AtaCommand, AtaFeatures, AtaSectorNumber, AtaCylinderLow, AtaCylinderHigh, AtaDeviceHead, + Exp (étendus 48 bits), AtaSectorCount, Reserved2[6].
- EFI_ATA_STATUS_BLOCK : Reserved1[2], AtaStatus, AtaError, AtaSectorNumber, AtaCylinderLow, AtaCylinderHigh, AtaDeviceHead, + Exp, Reserved2.
- [AGENT] Interdit : tout ATA destructif (SECURITY ERASE, SANITIZE, SMART WRITE, Format) sans accord explicite (règle du projet, comme NVMe Format/Sanitize).

## Statut
- Lignes 1 à 28218 lues. Prochaine : ligne 28219 (suite PassThru status, GetNextPort, GetNextDevice, BuildDevicePath, GetDevice, ResetPort, ResetDevice).

## Lignes 28219-28518 — EFI_ATA_STATUS_BLOCK (fin), protocoles ATA (EFI_ATA_PASS_THRU_CMD_PROTOCOL), 13.13.2 PassThru (fin), 13.13.3 GetNextPort, 13.13.4 GetNextDevice, 13.13.5 BuildDevicePath, 13.13.6 GetDevice, 13.13.7 ResetPort (début)
- Protocoles ATA : HARDWARE_RESET 0x00, SOFTWARE_RESET 0x01, NON_DATA 0x02, PIO_DATA_IN 0x04, PIO_DATA_OUT 0x05, DMA 0x06, DMA_QUEUED 0x07, DEVICE_DIAGNOSTIC 0x08, DEVICE_RESET 0x09, UDMA_DATA_IN 0x0A, UDMA_DATA_OUT 0x0B, FPDMA 0x0C, RETURN_RESPONSE 0xFF.
- Length : BYTES 0x80 (sinon en blocs) ; MASK 0x70 ; NO_DATA_TRANSFER 0x00 ; FEATURES 0x10 ; SECTOR_COUNT 0x20 ; TPSIU 0x30 ; COUNT 0x0F.
- PassThru codes : SUCCESS, BAD_BUFFER_SIZE (InTransferLength renvoie la taille max), NOT_READY (file pleine, retry), DEVICE_ERROR, INVALID_PARAMETER (Port/PM invalides ou alignement IoAlign), UNSUPPORTED, TIMEOUT (Timeout en 100 ns). Asb à lire après SUCCESS/BAD_BUFFER/DEVICE_ERROR/TIMEOUT.
- [AGENT] Pour PATA/SATA bas niveau : ne jamais émettre HARDWARE_RESET (COMRESET) sur un disque interne sans accord.
- GetNextPort : Port=0xFFFF → premier port ; EFI_NOT_FOUND après le dernier ; EFI_INVALID_PARAMETER si port non retourné avant.
- GetNextDevice : même logique pour PortMultiplierPort ; sans port multiplier : SUCCESS avec 0xFFFF ou NOT_FOUND.
- BuildDevicePath : alloue avec AllocatePool ; appelant libère. EFI_NOT_FOUND si absent.
- GetDevice : DevicePath → Port/PM ; EFI_UNSUPPORTED si type inconnu ; EFI_NOT_FOUND si pas de correspondance.

## Statut
- Lignes 1 à 28518 lues. Prochaine : ligne 28519 (13.13.7 ResetPort suite, ResetDevice, 13.14 Extended SCSI Pass Thru).

## Lignes 28519-28818 — 13.13.7 ResetPort (fin), 13.13.8 ResetDevice, 13.14 Storage Security Command, 13.15 NVM Express Pass Through (début)
- ResetPort : EFI_UNSUPPORTED si pas supporté ; DEVICE_ERROR ; TIMEOUT ; SUCCESS.
- ResetDevice(Port, PM) : mêmes codes + EFI_INVALID_PARAMETER.
- Storage Security Command : GUID C88B0B6D-0DFC-49A7-9CB4-49074B4C3A78 ; ReceiveData(MediaId, Timeout, SecurityProtocolId, SecurityProtocolSpecificData (big-endian), PayloadBufferSize, PayloadBuffer, PayloadTransferSize) ; SendData(...). ATA : TRUSTED RECEIVE/SEND ou Trusted Non-Data si taille 0 ; SCSI : SECURITY PROTOCOL IN/OUT (SPC-4). EFI_WARN_BUFFER_TOO_SMALL si tronqué.
- [AGENT] SendData = commande d'écriture de sécurité (peut modifier le disque) : interdit sur disque interne sans accord explicite (règle projet).
- NVM Express Pass Through : GUID 52C78312-8EDC-4233-98F2-1A1AA5E388A5 ; Mode, PassThru, GetNextNamespace, BuildDevicePath, GetNamespace. Attributs : Attributes, IoAlign, NvmeVersion (format du registre VS).
- [AGENT] Concorde avec mes notes NVMe : CAP, VS, CC, AQA/ASQ/ACQ sont la base du contrôleur ; le pass-through UEFI passe par des commandes admin/IO déjà formées.

## Statut
- Lignes 1 à 28818 lues. Prochaine : ligne 28819 (suite NVMe Pass Through : Related definitions, PassThru, GetNextNamespace, BuildDevicePath, GetNamespace).

## Lignes 28819-29118 — 13.15 NVMe Pass Through (mode, attributs, PassThru, command packet, CDW0, Flags, completion), 13.15.3 GetNextNamespace, 13.15.4 BuildDevicePath, 13.15.5 GetNamespace (début)
- EFI_NVM_EXPRESS_PASS_THRU_MODE : Attributes, IoAlign, NvmeVersion. Attributs : PHYSICAL 0x1, LOGICAL 0x2, NONBLOCKIO 0x4, CMD_SET_NVM 0x8.
- PassThru(This, NamespaceId, Packet, Event) : NamespaceId 0 = contrôleur ; 0xFFFFFFFF = tous les namespaces valides (ne pas confondre avec le champ NSID de la commande).
- EFI_NVM_EXPRESS_PASS_THRU_COMMAND_PACKET : CommandTimeout (100 ns, 0 = infini), TransferBuffer / TransferLength, MetaDataBuffer / MetadataLength, QueueType (0 = Admin SQ, 1 = I/O SQ), NvmeCmd*, NvmeCompletion*.
- EFI_NVM_EXPRESS_COMMAND : Cdw0 (OpCode 8 bits, FusedOperation 2 bits : NORMAL 0, FUSED_FIRST 1, FUSED_SECOND 2), Flags, Nsid, Cdw2, Cdw3, Cdw10..Cdw15. Flags : CDW2_VALID 0x01 ... CDW15_VALID 0x80 (indique quels champs sont valides).
- EFI_NVM_EXPRESS_COMPLETION : DW0..DW3 = CQE brute (8 octets de statut en DW3 : phase tag bit 16 de DW3, SC et SCT, à vérifier dans la base NVMe).
- Codes PassThru : SUCCESS, BAD_BUFFER_SIZE (TransferLength = max), NOT_READY, DEVICE_ERROR, INVALID_PARAMETER (NSID, alignement, QueueType ∉ {0,1}), UNSUPPORTED, TIMEOUT. Après SUCCESS/DEVICE_ERROR/TIMEOUT, lire NvmeCompletion.
- GetNextNamespace : 0xFFFFFFFF = premier ; EFI_NOT_FOUND après le dernier ; EFI_INVALID_PARAMETER si NSID invalide.
- BuildDevicePath : NOT_FOUND si NSID invalide ; AllocatePool à libérer par l'appelant.
- [AGENT] Concordance NVMe : le CQE Phase Tag est le bit 16 de DW3 (voir NVME_PCIE_TRANSPORT_LECTURE.md, section CQ phase). Le pass-through UEFI ne gère pas les doorbells : le pilote firmware le fait.

## Statut
- Lignes 1 à 29118 lues. Prochaine : ligne 29119 (GetNamespace codes, ResetPort/ResetController? et 13.16 Serial/RAM Disk/ etc.).

## Lignes 29119-29418 — 13.15.5 GetNamespace (fin), 13.16 SD/MMC Pass Thru (GUID 716EF0D9-FF83-4F69-81E9-518BD39A8E70 ; PassThru, GetNextSlot, BuildDevicePath, GetSlotNumber, ResetDevice), 13.17 RAM Disk (début)
- GetNamespace : EFI_INVALID_PARAMETER si NULL ; EFI_UNSUPPORTED si type de nœud inconnu ; EFI_NOT_FOUND si pas de correspondance.
- SD/MMC Pass Thru : EFI_SD_MMC_COMMAND_BLOCK (CommandIndex CMDx, CommandArgument, CommandType Bc/Bcr/Ac/Adtc, ResponseType R1..R7 dont R1b, R5b), EFI_SD_MMC_STATUS_BLOCK (Resp0..Resp3), TransactionStatus.
- [AGENT] Données : InDataBuffer ou OutDataBuffer obligatoire pour commande de données ; EFI_NO_MEDIA si pas de carte. Clé-SD bare-metal : hors périmètre principal.
- EFI_RAM_DISK_PROTOCOL GUID AB38A0DF-6873-44A9-87E6-D4EB56148449 ; Register(RamDiskBase, RamDiskSize, RamDiskType, ParentDevicePath, DevicePath) ; Unregister(DevicePath). Le consommateur alloue/libère la mémoire.

## Statut
- Lignes 1 à 29418 lues. Prochaine : ligne 29419 (suite RAM Disk : Register, Unregister, types, 13.18 Partition/Miscellaneous).

## Lignes 29419-29718 — 13.17.2 RAM Disk Register (fin), 13.17.3 Unregister, 13.18 Partition Information Protocol, 13.19 NVDIMM Label Protocol (début), Label Index Block
- RAM Disk Register : RamDiskSize = 0 → EFI_INVALID_PARAMETER ; RamDiskType NULL → EFI_INVALID_PARAMETER ; DevicePath déjà présent → EFI_ALREADY_STARTED. Consommateur alloue/libère la RAM ; producteur installe Device Path + Block I/O.
- Unregister : codes SUCCESS, INVALID_PARAMETER, UNSUPPORTED, NOT_FOUND.
- Partition Information Protocol : GUID 8CF2F62C-BC9B-4821-808D-EC9EC421A1A0, révision 0x00010000 ; Type (PARTITION_TYPE_OTHER 0x00 / MBR 0x01 / GPT 0x02), System (1 = ESP), Info (MBR_PARTITION_RECORD ou EFI_PARTITION_ENTRY). Structure packée (#pragma pack(1)).
- [AGENT] Un utilitaire qui modifie les partitions doit obtenir l'accès exclusif au disque ; sinon redémarrer. Ne pas toucher aux partitions du disque interne sans accord.
- NVDIMM Label Protocol : GUID D40B6B80-97D5-4282-BB1D-223A16918058 ; LabelStorageInformation, LabelStorageRead, LabelStorageWrite. Hors périmètre (NVDIMM).
- Label Index Block : Sig "NAMESPACE_INDEX0" (16 octets), Flags[3] = 0, LabelSize (128 << LabelSize), Seq (2 bits utiles, 00 = invalide), MyOff, MySize (multiple de 256), OtherOff, LabelOff, NSlot, Major, Minor, Checksum, Free[]. Deux index blocks alternés ; si Seq identiques, prendre celui à l'offset le plus haut.

## Statut
- Lignes 1 à 29718 lues. Prochaine : ligne 29719 (suite Label Index Block, Label Definitions, puis 13.20 et suite).

## Lignes 29719-30018 — 13.19 NVDIMM Label (fin : Label Index Block, EFI_NVDIMM_LABEL, SetCookie, 13.19.5 Label Storage Area, règles d'écriture/recovery, renommage), 13.20 UFS Device Config (début)
- Label : Uuid, Name[64], Flags (ROLABEL 0x1, LOCAL 0x2, UPDATING, SPACOOKIE_BOUND 0x10), NLabel, Position, SetCookie, LbaSize, Dpa, RawSize, Slot, Alignment, TypeGuid, AddressAbstractionGuid, SPALocationCookie, Checksum (Fletcher64).
- Règles clés : au moins 2 slots libres pour ajouter un label ; jamais écrire un slot actif ; écriture = slot libre puis bascule de l'Index Block (incrément Seq, écriture dans le bloc inactif).
- Validation : Index Block invalide si Sig, Checksum, MyOff/MySize/OtherOff incohérent, Seq = 0 ; Seq identiques → bloc à l'offset le plus haut.
- Recovery non-local : roll-back si incomplet, roll-forward sinon (flag UPDATING).
- [AGENT] Hors périmètre (NVDIMM / pmem), à noter seulement.
- UFS Device Config : GUID B81BFAB0-0EB3-4CF9-8465-7FA986361664 ; RwUfsDescriptor, RwUfsFlag, RwUfsAttribute. Lecture/écriture de descripteurs/flags/attributs : écriture = commande à risque, interdit sans accord.

## Statut
- Lignes 1 à 30018 lues. Prochaine : ligne 30019 (suite UFS Device Config : fonctions, codes, 13.21 et protocoles suivants).

## Lignes 30019-30318 — 13.20 UFS Device Config (RwUfsDescriptor, RwUfsFlag, RwUfsAttribute : codes), 14 PCI Bus Support, 14.1 PCI Root Bridge I/O (aperçu), 14.2.1 EFI_PCI_ROOT_BRIDGE_IO_PROTOCOL (structure, widths)
- UFS : Read BOOLEAN ; DescId/Index/Selector ; FlagId (1 octet) ; AttrId/Index/Selector + AttrSize. Codes : SUCCESS, INVALID_PARAMETER, DEVICE_ERROR. [AGENT] Écriture = à risque, interdit sans accord.
- 14 PCI Root Bridge : GUID 2F707EBB-4A1A-11D4-9A38-0090273FC14D. Membres : ParentHandle, PollMem, PollIo, Mem (Read/Write), Io (Read/Write), Pci (Read/Write), CopyMem, Map, Unmap, AllocateBuffer, FreeBuffer, Flush, GetAttributes, SetAttributes, Configuration, SegmentNumber.
- Ordre PCI : le protocole respecte les règles PCI ; pas de garantie d'ordre avec mémoire système sans barrière (fence) sur certains x64.
- [AGENT] Le pilote xHCI/NVMe utilise souvent Map/Unmap pour DMA et AllocateBuffer pour les structures communes (CQ/SQ). Ces opérations DOIVENT être appelées avec le bon sens (Read/Write) et Map/Unmap appariés.
- [AGENT] Un bare-metal après ExitBootServices n'a plus ce protocole : il faut accéder à la config PCI directement (ECAM MCFG) et gérer ses propres mappings DMA.
- EFI_PCI_ROOT_BRIDGE_IO_PROTOCOL_WIDTH : Uint8/16/32/64, FifoUint8..64, FillUint8..64 (Fifo = adresse fixe, Fill = remplissage).

## Statut
- Lignes 1 à 30318 lues. Prochaine : ligne 30319 (suite PCI Root Bridge : PollMem, PollIo, Mem.Read/Write, Io, Pci, CopyMem, Map/Unmap, AllocateBuffer, Flush, Attributes, Configuration).

## Lignes 30319-30618 — 14.2.1 fin (IO_MEM, ACCESS, attributs PCI root bridge 0x1..0x40000, OPERATION), 14.2.2 PollMem (début)
- Attributs root bridge : ISA_MOTHERBOARD_IO 0x1 (0x00-0xFF), ISA_IO 0x2 (0x100-0x3FF, décodage 10 bits), VGA_PALETTE_IO 0x4, VGA_MEMORY 0x8 (0xA0000-0xBFFFF), VGA_IO 0x10, IDE_PRIMARY_IO 0x20, IDE_SECONDARY_IO 0x40, MEMORY_WRITE_COMBINE 0x80, MEMORY_CACHED 0x800, MEMORY_DISABLE 0x1000, DUAL_ADDRESS_CYCLE 0x8000 (AllocateBuffer seulement), ISA_IO_16 0x10000, VGA_PALETTE_IO_16 0x20000, VGA_IO_16 0x40000.
- Operations DMA : BusMasterRead/Write/CommonBuffer (32 bits) et *64 (dual address cycle).
- [AGENT] Séquence DMA lecture : Map(BusMasterRead) → programmer DeviceAddress → start → attendre → Unmap.
- [AGENT] DMA écriture : Map → start → attendre → lecture PCI factice pour vider les buffers posted → Flush() → Unmap.
- [AGENT] Common buffer : AllocateBuffer → Map(CommonBuffer) → usage cohérent → Unmap → FreeBuffer. C'est la structure utilisée pour CQ/SQ NVMe et rings xHCI.
- [AGENT] Map peut renvoyer moins d'octets que demandé : boucler.
- PollMem : Address, Mask, Value, Delay (100 ns), Result ; lit jusqu'à (Read & Mask) == Value ou délai expiré (EFI_TIMEOUT).

## Statut
- Lignes 1 à 30618 lues. Prochaine : ligne 30619 (suite PollMem codes, PollIo, Mem.Read/Write, Io, Pci, CopyMem, Map/Unmap détails).

## Lignes 30619-30918 — 14.2.2 PollMem (fin), 14.2.3 PollIo, 14.2.4/5 Mem.Read/Write, 14.2.6/7 Io.Read/Write, 14.2.8/9 Pci.Read/Write (adresse config), 14.2.10 CopyMem (début)
- PollMem/PollIo : au moins un accès même si Delay = 0 ; succès si (Result & Mask) == Value ; EFI_TIMEOUT à expiration ; Width invalide → EFI_INVALID_PARAMETER ; Delay en 100 ns.
- Mem/Io Read/Write : Count opérations ; Width Uint → Address et Buffer avancent ; Fifo → seul Buffer avance (même adresse, ex. FIFO UART/xHCI doorbell) ; Fill → seule Address avance (répète le motif Buffer[0]).
- [AGENT] FIFO : accès répété à la même adresse ; utile pour vider un registre de données (ex. PIO HDA/IDE) mais attention au cache si MEMORY_CACHED.
- Pci.Read/Write : adresse de configuration = Register (byte 0, 8 bits), Function (1), Device (1), Bus (1), ExtendedRegister (4 octets ; si non nul, remplace Register). Format standard config PCI = ECAM / port CF8/CFC.
- [AGENT] Pci.Write est une écriture de configuration : à ne pas utiliser sur le matériel de stockage en dehors d'un besoin explicite.
- Ordre : toutes les transactions de lecture PCI sont complètes avant retour ; écritures suivent les règles PCI sauf si MEMORY_CACHED (règles CPU).
- CopyMem(Width, DestAddress, SrcAddress, Count) : copie dans l'espace mémoire du root bridge (ex. scroll vidéo).

## Statut
- Lignes 1 à 30918 lues. Prochaine : ligne 30919 (suite CopyMem, Map, Unmap, AllocateBuffer, FreeBuffer, Flush, GetAttributes, SetAttributes, Configuration).

## Lignes 30919-31218 — 14.2.10 CopyMem (fin : chevauchement, règle de direction), 14.2.11 Map, 14.2.12 Unmap, 14.2.13 AllocateBuffer, 14.2.14 FreeBuffer, 14.2.15 Flush, 14.2.16 GetAttributes (début)
- CopyMem : si Dest > Src et Dest < Src+taille → copier depuis la fin ; sinon depuis le début.
- Map(Operation, HostAddress, NumberOfBytes (entrée/sortie), DeviceAddress, Mapping) : Map peut mapper MOINS que demandé → boucler Map/Unmap. Codes : SUCCESS, INVALID_PARAMETER (Operation, NULL), UNSUPPORTED (HostAddress non common buffer), DEVICE_ERROR, OUT_OF_RESOURCES.
- [AGENT] Lecture bus-master : données DOIVENT être en mémoire avant Map. Écriture bus-master : données non visibles avant Unmap. Common buffer : seulement mémoire d'AllocateBuffer.
- Unmap(Mapping) : commit des écritures ; DEVICE_ERROR si non commis.
- AllocateBuffer(Type ignoré, MemoryType = EfiBootServicesData ou EfiRuntimeServicesData, Pages, HostAddress, Attributes) : attributs autorisés WRITE_COMBINE 0x80, CACHED 0x800, DUAL_ADDRESS_CYCLE 0x8000 ; autres → EFI_UNSUPPORTED. Device address Map < 4 GiB si DUAL_ADDRESS_CYCLE clair.
- [AGENT] Pour xHCI/NVMe, rester sur allocation sous 4 GiB (DAC clair) quand le contrôleur n'est pas dual-address-cycle capable.
- FreeBuffer(Pages, HostAddress) : EFI_INVALID_PARAMETER si non alloué par AllocateBuffer.
- Flush : PCI posted writes du host bridge vers mémoire ; ne vide PAS les bridges PCI en aval (il faut une lecture PCI du contrôleur avant). EFI_DEVICE_ERROR si échec.
- GetAttributes(Supports, Attributes) : au moins un des deux non NULL, sinon EFI_INVALID_PARAMETER.

## Statut
- Lignes 1 à 31218 lues. Prochaine : ligne 31219 (suite GetAttributes, SetAttributes, Configuration, puis 14.3 PCI Bus Driver / Driver Binding, etc.).

## Lignes 31219-31518 — 14.2.16 GetAttributes (fin), 14.2.17 SetAttributes, 14.2.18 Configuration (QWORD Address Space Descriptor 0x8A, End Tag 0x79), 14.2.19 PCI Root Bridge Device Paths (début)
- SetAttributes : WRITE_COMBINE/CACHED/DISABLE exigent ResourceBase et ResourceLength (sinon EFI_INVALID_PARAMETER) ; plusieurs attributs nécessitant une plage → INVALID_PARAMETER ; non supporté → UNSUPPORTED ; plage peut être élargie (valeurs retournées).
- Configuration : renvoie des descripteurs ACPI : QWORD Address Space (0x8A, longueur 0x2B) avec Resource Type (0 mem, 1 I/O, 2 bus), Address Space Granularity (32 ou 64), Min, Max, Translation Offset, Length ; End Tag 0x79 0x00 (checksum 0 = valide). Le buffer est read-only et ne doit pas être libéré par l'appelant.
- [AGENT] Ce format est le même que les descripteurs _CRS ACPI (voir ACPI_LECTURE.md) : une source pour cross-check du MADT/MCFG/_CRS.
- Device path root bridge : ACPI(PNP0A03, UID) ; _HID/_UID doivent correspondre à l'ACPI. Desktop : UID 0x0000 ; serveur : UID 0..3.
- [AGENT] Pour un agent bare-metal : récupérer la liste des root bridges via ACPI (_HID PNP0A03/PNP0A08 + _CRS) plutôt que le protocole, car après ExitBootServices il n'existe plus.

## Statut
- Lignes 1 à 31518 lues. Prochaine : ligne 31519 (suite Device Paths, puis 14.3 PCI Bus Support / Driver rules).

## Lignes 31519-31818 — 14.2.19 fin (Expanded ACPI device path ACPI(HID,UID,CID)), 14.3 PCI Driver Model (init, Driver Diagnostics, Component Name, Driver Family Override, PCI Bus Driver, Driver Binding Bus/Device, Enumeration, Device Drivers), 14.4 EFI PCI I/O Protocol (début)
- Driver image : LoadImage crée Image Handle (Loaded Image) ; StartImage ; le driver ne touche pas au matériel dans son entry point, il installe Driver Binding sur son Image Handle (+ Component Name, Diagnostics optionnels).
- PCI Bus Driver : Supported() vérifie Device Path + PCI Root Bridge I/O ; Start() : init host, énumération si nécessaire (bus numbers, I/O/Mem/PMem), découverte des devices, désactivation I/O/Mem/BusMaster sur devices legacy (VGA), création d'un handle par device avec Device Path + PCI I/O (+ Bus Specific Driver Override si Option ROM contient des drivers).
- PCI Bus Driver aplatit la hiérarchie (les bridges PCI-PCI n'apparaissent pas).
- Stop() : transactions finies ; uninstall des protocoles des enfants ; libérer ROM copies ; détruire les handles.
- PCI Device Driver : ne crée JAMAIS de handle ; ne modifie PAS BARs ni config de bridge ; enable I/O/Mem/BusMaster via Attributes() et restaure à Stop().
- [AGENT] Règle importante pour notre agent : le firmware a déjà configuré les BAR ; un driver qui les réécrit casse les autres périphériques. Sur bare-metal, c'est nous qui devons faire l'énumération si on ne boot pas via firmware.
- 14.4 PCI I/O : GUID 4CF5B200-68B8-4CA5-9EEC-B23E3F50029A. Membres : PollMem, PollIo, Mem.Read/Write (BAR index), Io.Read/Write (BAR index), Pci.Read/Write, CopyMem, Map, Unmap, AllocateBuffer, FreeBuffer, Flush, GetLocation, Attributes, GetBarAttributes, SetBarAttributes, RomSize, RomImage.
- Widths identiques (Uint/Fifo/Fill). EFI_PCI_IO_PASS_THROUGH_BAR = 0xFF : accès direct sans BAR (adresse brute, à éviter).

## Statut
- Lignes 1 à 31818 lues. Prochaine : ligne 31819 (suite PCI I/O PollMem/PollIo, Mem/Io, Pci.Read/Write BAR, Map, Attributes, GetBarAttributes, SetBarAttributes).

## Lignes 31819-32118 — 14.4.1 EFI_PCI_IO_PROTOCOL (fin : Mem/Io IO_MEM BAR, ACCESS, CONFIG, CONFIG_ACCESS, attributs PCI IO), OPERATION, 14.4.2 PollMem (début)
- PCI I/O : Mem/Io Read/Write prennent BarIndex (0..5 ou 6 ROM) + Offset ; Pci.Read/Write prennent Offset (0..0xFFFFFFFF, adresse dans l'espace config du device).
- Attributs PCI I/O : ISA_MOTHERBOARD_IO 0x1, ISA_IO 0x2, VGA_PALETTE_IO 0x4, VGA_MEMORY 0x8, VGA_IO 0x10, IDE_PRIMARY 0x20, IDE_SECONDARY 0x40, WRITE_COMBINE 0x80, IO 0x100 (décode I/O), MEMORY 0x200 (décode Mem), BUS_MASTER 0x400, MEMORY_CACHED 0x800, MEMORY_DISABLE 0x1000, EMBEDDED_DEVICE 0x2000 (carte-mère vs slot), EMBEDDED_ROM 0x4000, DUAL_ADDRESS_CYCLE 0x8000, ISA_IO_16 0x10000, VGA_PALETTE_IO_16 0x20000, VGA_IO_16 0x40000.
- [AGENT] Pour activer un contrôleur : Attributes(Enable, IO|MEMORY|BUS_MASTER). Pour l'éteindre au Stop : Attributes(Disable) avec l'état initial sauvegardé. C'est la base du bit Command PCI (offset 0x04).
- [AGENT] DMA via PCI I/O : mêmes séquences que root bridge, mais Operation = BusMasterRead/Write/CommonBuffer (sans 64). Pour xHCI/NVMe, DMA 64 bits → DUAL_ADDRESS_CYCLE.
- PCI I/O Operation enum : BusMasterRead, BusMasterWrite, BusMasterCommonBuffer.

## Statut
- Lignes 1 à 32118 lues. Prochaine : ligne 32119 (suite 14.4.2 PollMem, PollIo, Mem.Read/Write, Io, Pci, CopyMem, Map, Unmap, AllocateBuffer, FreeBuffer, Flush, GetLocation, Attributes, GetBarAttributes, SetBarAttributes).

## Lignes 32119-32418 — 14.4.2 PollMem (fin), 14.4.3 PollIo, 14.4.4/5 Mem.Read/Write (BAR), 14.4.6/7 Io.Read/Write (BAR), 14.4.8/9 Pci.Read/Write (config), 14.4.10 CopyMem (début)
- PCI I/O : BarIndex 0..5 ; EFI_PCI_IO_PASS_THROUGH_BAR (0xFF) = adresse brute passée au root bridge.
- Codes : EFI_UNSUPPORTED si BarIndex invalide pour ce contrôleur ou Offset hors BAR (vérifier le BAR dans la config avant).
- Mem/Io : Offset + Width×Count doit tenir dans la BAR ; Fifo = même adresse ; Fill = motif.
- Pci.Read/Write : Offset dans l'espace de config du device ; EFI_UNSUPPORTED si hors en-tête.
- [AGENT] Pour les registres xHCI (BAR0) : Mem.Read32(BarIndex=0, Offset=CAP...) ; pour MSI-X, la table pointe vers une BAR (BIR) — utiliser Mem sur la BAR indiquée.
- CopyMem PCI I/O : même règle de chevauchement que root bridge (direction selon Dest/Src).

## Statut
- Lignes 1 à 32418 lues. Prochaine : ligne 32419 (suite CopyMem, Map, Unmap, AllocateBuffer, FreeBuffer, Flush, GetLocation, Attributes, GetBarAttributes, SetBarAttributes).

## Lignes 32419-32718 — 14.4.10 CopyMem (fin), 14.4.11 Map (PCI I/O : opérations BusMasterRead/Write/CommonBuffer), 14.4.12 Unmap, 14.4.13 AllocateBuffer (attributs WRITE_COMBINE/CACHED seulement), 14.4.14 FreeBuffer, 14.4.15 Flush, 14.4.16 GetLocation (début)
- PCI I/O Map : mêmes règles que root bridge ; mapping partiel possible → boucle.
- AllocateBuffer PCI I/O : attributs légaux WRITE_COMBINE et CACHED uniquement (DUAL_ADDRESS_CYCLE pris de l'état courant des Attributes, pas du paramètre) ; device address < 4 GiB si DAC clair.
- [AGENT] Ne pas passer DUAL_ADDRESS_CYCLE dans AllocateBuffer PCI I/O : il est lu sur l'état courant (Attributes(Enable)).
- Flush PCI I/O : même limite que root bridge (ne vide pas les bridges en aval).
- GetLocation : Segment, Bus, Device, Function ; à utiliser avec Pci.Read du root bridge pour config cycles sur les pairs.

## Statut
- Lignes 1 à 32718 lues. Prochaine : ligne 32719 (suite GetLocation, Attributes, GetBarAttributes, SetBarAttributes, puis 14.5 ...).

## Lignes 32719-33018 — 14.4.16 GetLocation (fin), 14.4.17 Attributes (Get/Set/Enable/Disable/Supported), 14.4.18 GetBarAttributes, 14.4.19 SetBarAttributes, 14.4.20 PCI Device Paths, 14.4.21 PCI Option ROMs (début)
- Attributes(Operation, Attributes, Result) : Get/Supported exigent Result non NULL. Enable/Disable n'agissent que sur les bits à 1. UNSUPPORTED si bit non supporté par le contrôleur ou un bridge parent, ou si un autre contrôleur du même root bridge a déjà pris un attribut ISA/VGA/IDE (conflit).
- [AGENT] Séquence standard pilote : Attributes(Get,Supported) → sauver état initial → Attributes(Enable, IO|MEMORY|BUS_MASTER) → à Stop : Attributes(Set, état initial).
- GetBarAttributes(BarIndex, Supports, Resources) : Resources = descripteurs ACPI (QWORD 0x8A / End Tag 0x79) alloués par AllocatePool, à libérer par FreePool ; Supports et Resources NULL → INVALID_PARAMETER.
- SetBarAttributes(Attributes, BarIndex, Offset*, Length*) : peut élargir la plage ; UNSUPPORTED si attribut ou plage non supporté.
- Device path PCI : ACPI(PNP0A03,UID)/PCI(dev,fn) ; bridges PCI-PCI ajoutent un nœud PCI chacun. Exemple : ACPI(PNP0A03,0)/PCI(5,0)/PCI(7,0).
- [AGENT] Ces device paths servent à identifier les contrôleurs xHCI/NVMe : relire la valeur "Bus/Device/Function" réelle pour cross-check avec le PPR AMD (même format).
- PCI Option ROM : taille ≤ 16 MiB ; images alignées 512 octets, taille multiple de 512 ; PCIR dans les 64 KiB de l'image ; en-tête EFI PCI Expansion ROM pour images EFI.

## Statut
- Lignes 1 à 33018 lues. Prochaine : ligne 33019 (suite PCI Option ROM, puis 15 Protocols – Network / autres selon table des matières).

## Lignes 33019-33318 — 14.4.21 PCI Option ROM (suite : ordre de priorité, layouts), Standard PCI Expansion ROM Header (0x55AA, pointeur PCIR), codes type (0 IA-32, 1 Open Firmware, 2 HP PA-RISC, 3 EFI, 4+ réservés), EFI PCI Expansion ROM Header (0x0EF1, InitSize en 512 o, Subsystem 11/12, Machine 0x8664 x64 / 0xAA64 ARM64 / 0x0EBC EBC, compression 0/1, EFI image offset), 14.4.22 PCI Bus Driver responsabilités, 14.4.23 PCI Device Driver (layout recommandé)
- [AGENT] Ceci est de la lecture de ROM d'option : pour un agent qui ne boote pas par firmware, IGNORER (ne pas exécuter de code d'option ROM).
- Recherche : type code 3 (EFI) + signature 0x0EF1 + Subsystem 11 (Boot Service Driver) ou 12 (Runtime Driver) ; LoadImage via Load File 2 ; device path = contrôleur/Offset(start,end). Bus Specific Driver Override pour l'ordre.
- Machine types : 0x014C IA-32, 0x0200 Itanium, 0x0EBC EBC, 0x8664 X64, 0x01C2 ARM, 0xAA64 ARM64, 0x5032/0x5064 RISC-V 32/64, 0x5128 RISC-V 128, 0x6232/0x6264 LoongArch.
- Layout recommandé : en-tête EFI puis PCIR (signature 'PCIR', Vendor ID, Device ID, Class Code, Code Type 3, indicateur dernière image bit 7).

## Statut
- Lignes 1 à 33318 lues. Prochaine : ligne 33319 (suite layout PCI driver, 14.4.24 Nonvolatile Storage, puis suite).

## Lignes 33319-33618 — 14.4.24 Nonvolatile Storage (pas de service standard), 14.4.25 PCI Hot-Plug (hors préboot), figures 14.15-14.19 (layouts ROM), chapitre 15 SCSI Driver Model (15.1 aperçu, 15.2 SCSI Bus Drivers, 15.2.2 Enumeration, 15.3 SCSI Device Drivers, 15.4 EFI SCSI I/O : GUID 932F47E6-2362-4002-803E-3CD54B138F85, GetDeviceType, GetDeviceLocation, ResetBus (début))
- Nonvolatile PCI : aucun accès standard ; NE PAS écrire dans la ROM d'option ; RomImage est une copie mémoire.
- [AGENT] Règle projet : jamais d'écriture de firmware/option ROM sans accord.
- Hot-plug : hors périmètre préboot (recommandé de ne pas implémenter).
- SCSI Pass Thru : SCSI Pass Thru driver → SCSI Bus (Extended SCSI Pass Thru) → SCSI I/O par device. Usage typique : SMART pendant POST (Mode Sense, Log Sense).
- EFI_SCSI_IO_PROTOCOL membres : GetDeviceType, GetDeviceLocation, ResetBus, ResetDevice, ExecuteScsiCommand, IoAlign.
- Types de device (SPC-4) : 0x00 disque, 0x01 tape, 0x02 imprimante, 0x03 processeur, 0x04 WORM, 0x05 CD/DVD, 0x06 scanner, 0x07 optique, 0x08 changeur, 0x09 communication, 0x0C RAID, 0x0D SES, 0x0E RBC, 0x0F OCRW, 0x10 bridge, 0x11 OSD, 0x1F inconnu.
- GetDeviceLocation(Target*, Lun*) : (Target, Lun) identifie le device sur le canal.
- [AGENT] Hors périmètre (SCSI) sauf futur disque SAS. Lu pour complétude.

## Statut
- Lignes 1 à 33618 lues. Prochaine : ligne 33619 (suite ResetBus, ResetDevice, ExecuteScsiCommand, SCSI Pass Thru, Extended SCSI).

## Lignes 33619-33918 — 15.4.4 ResetBus (fin), 15.4.5 ResetDevice, 15.4.6 ExecuteScsiCommand (EFI_SCSI_IO_SCSI_REQUEST_PACKET, DataDirection 0/1/2, HostAdapterStatus, TargetStatus, codes), 15.5 SCSI Device Paths (début, exemple ACPI/PCI/SCSI)
- ResetBus / ResetDevice : UNSUPPORTED si non supporté ; DEVICE_ERROR ; TIMEOUT.
- ExecuteScsiCommand(Packet, Event) : Timeout 100 ns ; InDataBuffer (READ) et OutDataBuffer (WRITE) : l'un doit être NULL ; Cdb / CdbLength (6/10/12/16 ou variable) ; SenseData / SenseDataLength (en entrée taille, sortie octets écrits).
- Ordre d'inspection après SUCCESS/BAD_BUFFER/DEVICE_ERROR/TIMEOUT : HostAdapterStatus → TargetStatus → SenseDataLength → SenseData.
- HostAdapterStatus : OK 0x00, TIMEOUT_COMMAND 0x09, TIMEOUT 0x0B, MESSAGE_REJECT 0x0D, BUS_RESET 0x0E, PARITY 0x0F, REQUEST_SENSE_FAILED 0x10, SELECTION_TIMEOUT 0x11, DATA_OVERRUN_UNDERRUN 0x12, BUS_FREE 0x13, PHASE_ERROR 0x14, OTHER 0x7F.
- TargetStatus : GOOD 0x00, CHECK_CONDITION 0x02, CONDITION_MET 0x04, BUSY 0x08, INTERMEDIATE 0x10, INTERMEDIATE_CONDITION_MET 0x14, RESERVATION_CONFLICT 0x18, COMMAND_TERMINATED 0x22, QUEUE_FULL 0x28.
- [AGENT] Un device SCSI/SAS/ATAPI peut recevoir des commandes d'écriture (FORMAT UNIT, WRITE, MODE SELECT...) : interdit sur disque interne sans accord explicite.
- SCSI Device Path : ACPI(PNP0A03,0)/PCI(7,0)/SCSI(2,0) (exemple). Hors périmètre.

## Statut
- Lignes 1 à 33918 lues. Prochaine : ligne 33919 (suite SCSI Device Paths : ATAPI, Fibre Channel, InfiniBand ; 15.6 suite).

## Lignes 33919-34218 — 15.5 SCSI Device Paths (fin : SCSI, ATAPI, Fibre Channel, InfiniBand), 15.6 SCSI Pass Thru Device Paths (exemples single/multi-channel, Ctrl node), 15.7 Extended SCSI Pass Thru (GUID 143B7632-B81B-4CB7-ABD3-B625A5B9BFFE, membres PassThru, GetNextTargetLun, BuildDevicePath, GetTargetLun, ResetChannel, ResetTargetLun, GetNextTarget ; Mode : AdapterId, Attributes, IoAlign)
- Chemins types : ACPI(PNP0A03,0)/PCI(7,0)/SCSI(2,0) ; ATA(Primary,Master,0) ; Fibre(WWN,LUN) ; Infiniband(X,Y,Z).
- Extended SCSI Pass Thru attributs : PHYSICAL 0x1, LOGICAL 0x2, NONBLOCKIO 0x4 ; TARGET_MAX_BYTES 0x10.
- [AGENT] Hors périmètre (SCSI/ATAPI/FC/IB) pour l'agent NVMe/USB ; à garder en référence si un disque ATA est testé en lecture seule uniquement.

## Statut
- Lignes 1 à 34218 lues. Prochaine : ligne 34219 (suite EXT SCSI Pass Thru : PassThru, GetNextTargetLun, BuildDevicePath, GetTargetLun, ResetChannel, ResetTargetLun, GetNextTarget).

## Lignes 34219-34518 — 15.7.1 fin (Attributes RAID/PHYSICAL/LOGICAL, Controller node par canal), 15.7.2 PassThru (Target TARGET_MAX_BYTES=16, Lun, Packet ; codes), 15.7.3 GetNextTargetLun, 15.7.4 BuildDevicePath (début)
- Extended SCSI : PassThru(Target[16], Lun, Packet, Event). Sense : recommandé ≥ 252 octets. HostAdapterStatus bit 7 = code vendeur. Périphérique absent mais adresse valide → TIMEOUT + HostAdapterStatus = TIMEOUT_COMMAND (0x09).
- TargetStatus étendu : TASK_SET_FULL 0x28, ACA_ACTIVE 0x30, TASK_ABORTED 0x40.
- GetNextTargetLun : Target = 0xFF×16 → premier ; retour SUCCESS puis NOT_FOUND après le dernier ; INVALID_PARAMETER si cible non retournée.
- [AGENT] Hors périmètre actuel (SCSI).

## Statut
- Lignes 1 à 34518 lues. Prochaine : ligne 34519 (suite BuildDevicePath, GetTargetLun, ResetChannel, ResetTargetLun, GetNextTarget ; 15.8 suite).

## Lignes 34519-34818 — 15.7.4 BuildDevicePath (fin), 15.7.5 GetTargetLun, 15.7.6 ResetChannel, 15.7.7 ResetTargetLun, 15.7.8 GetNextTarget ; 16 Protocols – iSCSI Boot (aperçu, 16.1 couches, 16.2 EFI_ISCSI_INITIATOR_NAME_PROTOCOL GUID 59324945-EC44-4C0D-B1CD-9DB139DF070C)
- BuildDevicePath SCSI : NOT_FOUND si absent, INVALID_PARAMETER si DevicePath NULL, OUT_OF_RESOURCES.
- GetTargetLun : SUCCESS / INVALID_PARAMETER (NULL) / UNSUPPORTED (type non reconnu) / NOT_FOUND.
- ResetChannel / ResetTargetLun : UNSUPPORTED si absent ; INVALID_PARAMETER si Target/Lun hors plage ; DEVICE_ERROR ; TIMEOUT.
- GetNextTarget : 0xFF×16 = premier ; NOT_FOUND après le dernier.
- iSCSI : TCP/IP ; sessions ; boot via NIC (stack réseau UEFI) ou HBA TOE. EFI_ADAPTER_INFORMATION_PROTOCOL avec EFI_ADAPTER_INFO_NETWORK_BOOT pour détection.
- Initiator Name : taille max 224 octets (NUL inclus), chaîne vide interdite, réécriture écrase l'ancienne.
- [AGENT] Hors périmètre (réseau/iSCSI). Lu pour complétude.

## Statut
- Lignes 1 à 34818 lues. Prochaine : ligne 34819 (suite iSCSI : Initiator Name, iSCSI Boot Firmware Table, etc.).

## Lignes 34819-35118 — 16.2 iSCSI Initiator Name (Get/Set, 223 caractères UTF-8 + NUL, codes), CHAPITRE 17 USB SUPPORT : 17.1 USB2 Host Controller Protocol (GUID 3E745226-9818-45B6-A2AC-D7CD0E8BA2BC ; membres GetCapability, Reset, GetState, SetState, ControlTransfer, BulkTransfer, AsyncInterruptTransfer, SyncInterruptTransfer, IsochronousTransfer, AsyncIsochronousTransfer, GetRootHubPortStatus, SetRootHubPortFeature, ClearRootHubPortFeature, MajorRevision, MinorRevision), 17.1.3 GetCapability, 17.1.4 Reset, 17.1.5 GetState (début)
- USB2 HC : vitesses EFI_USB_SPEED_FULL 0x0000 (12 Mb/s), LOW 0x0001 (1.5 Mb/s), HIGH 0x0002 (480 Mb/s), SUPER 0x0003 (4.8 Gb/s).
- GetCapability(MaxSpeed, PortNumber, Is64BitCapable) : Is64BitCapable = TRUE si DMA > 4 GiB possible.
- Reset(Attributes) : GLOBAL 0x1 (reset bus + devices), HOST_CONTROLLER 0x2 (contrôleur seul), GLOBAL_WITH_DEBUG 0x4, HOST_WITH_DEBUG 0x8. Si debug port configuré (xHCI/EHCI), GLOBAL et HOST_CONTROLLER → EFI_ACCESS_DENIED.
- EFI_USB_HC_STATE : Halt, Operational, Suspend. Halt après reset matériel ; Operational requis pour trafic.
- [AGENT] Concordance xHCI : HCH (USBSTS.HCHalted) ↔ EfiUsbHcStateHalt ; RUN/STOP (USBCMD.R/S) ↔ Operational ; Suspend ↔ port suspend. À vérifier contre xHCI 1.2c (USBCMD, USBSTS).
- [AGENT] Projet : clé USB — pour l'agent, l'accès direct xHCI est prévu ; l'USB2 HC Protocol UEFI ne sera pas disponible après ExitBootServices.

## Statut
- Lignes 1 à 35118 lues. Prochaine : ligne 35119 (suite GetState, SetState, ControlTransfer, BulkTransfer, interrupt, isochronous, root hub).

## Lignes 35119-35418 — 17.1.5 GetState (fin : Halt/Suspend, 3 µs sans trafic), 17.1.6 SetState, 17.1.7 ControlTransfer (paramètres, Translator, règles MaximumPacketLength), 17.1.8 BulkTransfer (DataBuffersNumber, DataToggle, EFI_USB_MAX_BULK_BUFFER_NUM), 17.1.9 AsyncInterruptTransfer (début)
- GetState : Halt déclenché aussi par « pas de trafic bus pendant 3 µs » (suspend). SetState : transitions Halt↔Operational↔Suspend ; INVALID_PARAMETER si état inconnu.
- ControlTransfer : 3 types — NoData (Data NULL, DataLength 0) ; DataOut (SETUP puis OUT) ; DataIn (SETUP puis IN). Translator obligatoire pour split transactions (device LS/FS derrière hub HS) ; NULL pour USB1.1 ou HS natif.
- MaximumPacketLength (EP0) : LOW = 8 ; FULL/HIGH = 8/16/32/64 ; SUPER = 512.
- BulkTransfer : FULL → Data[0] seul ; HIGH → plusieurs buffers (≤ EFI_USB_MAX_BULK_BUFFER_NUM) ; direction dans le bit IN/OUT de EndPointAddress ; DataToggle 0 ou 1 (DATA0/DATA1), sortie = toggle du dernier paquet réussi (à mémoriser par endpoint).
- MaxPacket bulk : ≤ 64 FULL, ≤ 512 HIGH, ≤ 1024 SUPER.
- [AGENT] Règle clé : le toggle doit être persisté par endpoint ; un mauvais toggle casse la synchro (perte/duplication de paquets).

## Statut
- Lignes 1 à 35418 lues. Prochaine : ligne 35419 (suite AsyncInterruptTransfer, SyncInterruptTransfer, IsochronousTransfer, AsyncIsochronous, root hub).

## Lignes 35419-35718 — 17.1.9 AsyncInterruptTransfer (fin : IsNewTransfer, PollingInterval 1..255 ms, callback, annulation), 17.1.10 SyncInterruptTransfer, 17.1.11 IsochronousTransfer (EFI_USB_MAX_ISO_BUFFER_NUM 7 / NUM1 2), 17.1.12 AsyncIsochronousTransfer (début)
- AsyncInterrupt : IsNewTransfer TRUE = crée le pipe (un seul par endpoint, sinon INVALID_PARAMETER) ; FALSE = annule. Direction IN obligatoire ; PollingInterval 1..255 ; DataLength ≠ 0 ; DataToggle 0/1.
- [AGENT] Clavier/souris/hub : pattern « submit once, execute periodically » (HID) ; à mapper plus tard si besoin d'entrée bare-metal (xHCI interrupter/ring événement).
- SyncInterrupt : une seule exécution ; MaximumPacketLength ≤ 3072 HS, ≤ 64 FS, ≤ 8 LS.
- Isochronous : FS/HS/SS seulement ; MaxPacket ≤ 1023 FS, ≤ 1024 HS/SS ; FS : Data[0] seul ; HS : jusqu'à 7 buffers ; split FS sur HC HS : 2 buffers. Doit tenir dans une trame USB (1 ms) sinon EFI_TIMEOUT.
- [AGENT] Isochrone = réservation de bande passante : relevant pour audio USB uniquement.

## Statut
- Lignes 1 à 35718 lues. Prochaine : ligne 35719 (suite AsyncIsochronous, GetRootHubPortStatus, SetRootHubPortFeature, ClearRootHubPortFeature, port features).

## Lignes 35719-36017 — 17.1.12 AsyncIsochronousTransfer (fin), 17.1.13 GetRootHubPortStatus (EFI_USB_PORT_STATUS : PortStatus / PortChangeStatus ; bits USB_PORT_STAT_* et *_C_*), 17.1.14 SetRootHubPortFeature (début)
- AsyncIsochronous : retour immédiat ; callback à la fin ; mêmes limites de buffers et MaxPacket que IsochronousTransfer.
- PortStatus bits : CONNECTION 0x0001, ENABLE 0x0002, SUSPEND 0x0004, OVERCURRENT 0x0008, RESET 0x0010, POWER 0x0100, LOW_SPEED 0x0200, HIGH_SPEED 0x0400, SUPER_SPEED 0x0800, OWNER 0x2000.
- PortChangeStatus bits : C_CONNECTION 0x0001, C_ENABLE 0x0002, C_SUSPEND 0x0004, C_OVERCURRENT 0x0008, C_RESET 0x0010.
- Priorité vitesse : SUPER_SPEED (bit 11) > HIGH_SPEED (bit 10) > LOW_SPEED (bit 9) ; si SUPER set, ignorer 9/10 ; si HIGH set, ignorer 9.
- [AGENT] Les bits PORTSC xHCI (CCS, PED, PR, PLS, speed ID) correspondent à ces champs : vérifier dans xHCI 1.2c §5.4.8 avant tout code.
- GetRootHubPortStatus : PortNumber 0-based ; ≥ nombre de ports → EFI_INVALID_PARAMETER.

## Statut
- Lignes 1 à 36017 lues. Prochaine : ligne 36018 (suite SetRootHubPortFeature, ClearRootHubPortFeature, EFI_USB_PORT_FEATURE values, USB I/O Protocol 17.2).

## Lignes 36018-36317 — 17.1.14 SetRootHubPortFeature (fin : EFI_USB_PORT_FEATURE = Enable 1, Suspend 2, Reset 4, Power 8, Owner 13, ConnectChange 16, EnableChange 17, SuspendChange 18, OverCurrentChange 19, ResetChange 20), 17.1.15 ClearRootHubPortFeature, 17.2 USB Driver Model (bus driver, hot-plug, énumération, device driver), 17.2.4/17.2.5 EFI_USB_IO_PROTOCOL (GUID 2B2F68D6-0CD2-44CF-8E8B-BBA20B1B5B75), 17.2.6 UsbControlTransfer (début)
- SetRootHubPortFeature : seuls Enable, Suspend, Reset, Power sont légaux ; sinon INVALID_PARAMETER. Port ≥ nb de ports → INVALID_PARAMETER.
- ClearRootHubPortFeature : Enable (disable), Suspend (resume), Reset (clear RESET), Power (coupe), Owner (rend le port au companion host), change bits (ConnectChange, EnableChange, SuspendChange, OverCurrentChange, ResetChange).
- [AGENT] Concordance xHCI : Reset root hub = PORTSC.PR ; Power = PP ; Owner = non applicable (pas de companion). Clear change bits = écrire 1 dans les bits W1C de PORTSC (CSC, PEC, WRC, OCC, PRC, PLC, CEC).
- USB Driver Model : bus driver consume EFI_USB2_HC_PROTOCOL et crée un handle par interface (USB_IO + Device Path) ; device driver ouvre USB_IO et installe ses protocoles (ex. clavier → SIMPLE_TEXT_INPUT).
- Hot-plug : timer root hub + events hubs ; énumération : reset/enable port → configurer → handle par interface → ConnectController ; déconnexion : DisconnectController (retry si échec).
- USB I/O : UsbControlTransfer, UsbBulkTransfer, UsbAsyncInterruptTransfer, UsbSyncInterruptTransfer, UsbIsochronousTransfer, UsbAsyncIsochronousTransfer, UsbGetDeviceDescriptor, UsbGetConfigDescriptor, UsbGetInterfaceDescriptor, UsbGetEndpointDescriptor, UsbGetStringDescriptor, UsbGetSupportedLanguages, UsbPortReset.

## Statut
- Lignes 1 à 36317 lues. Prochaine : ligne 36318 (suite UsbControlTransfer, UsbBulkTransfer, etc.).

## Lignes 36318-36617 — 17.2.6 UsbControlTransfer (fin : EFI_USB_DATA_DIRECTION, codes EFI_USB_ERR_*, EFI_USB_DEVICE_REQUEST {RequestType, Request, Value, Index, Length}), 17.2.7 UsbBulkTransfer, 17.2.8 UsbAsyncInterruptTransfer (callback, exemple clavier)
- EFI_USB_ERR : NOERROR 0x0000, NOTEXECUTE 0x0001, STALL 0x0002, BUFFER 0x0004, BABBLE 0x0008, NAK 0x0010, CRC 0x0020, TIMEOUT 0x0040, BITSTUFF 0x0080, SYSTEM 0x0100. Résultat détaillé dans Status (UINT32).
- Timeout 0 = attendre indéfiniment jusqu'à SUCCESS/DEVICE_ERROR (ms).
- UsbBulkTransfer : DeviceEndpoint 0x01..0x0F (OUT) ou 0x81..0x8F (IN), MSB = direction ; sinon INVALID_PARAMETER ; doit être un endpoint BULK.
- UsbAsyncInterrupt : PollingInterval 1..255 ms ; callback(Data, DataLength, Context, Status) ; IsNewTransfer FALSE = annule.
- [AGENT] Clé USB de mass-storage : CBW/CSW sur bulk IN/OUT ; le toggle DATA0/DATA1 doit suivre. Écrire sur la clé de l'agent = interdit sans accord (README : « Rien n'est écrit sur une clé sans accord explicite »).
- [AGENT] Ne pas confondre EFI_USB_ERR_STALL avec un vrai échec : un STALL sur bulk impose un Clear Halt (ClearFeature ENDPOINT_HALT).

## Statut
- Lignes 1 à 36617 lues. Prochaine : ligne 36618 (suite exemple clavier, UsbSyncInterruptTransfer, UsbIsochronousTransfer, UsbGet*Descriptor, UsbPortReset, USB Hot-Plug USB I/O).

## Lignes 36618-36917 — exemple clavier (fin), 17.2.9 UsbSyncInterruptTransfer (une seule exécution, Timeout), 17.2.10 UsbIsochronousTransfer (1 trame, découpage si > payload), 17.2.11 UsbAsyncIsochronousTransfer, 17.2.12 UsbGetDeviceDescriptor (EFI_USB_DEVICE_DESCRIPTOR, USB 1.1), 17.2.13 UsbGetConfigDescriptor (config active), 17.2.14 UsbGetInterfaceDescriptor (début)
- Sync interrupt : requête supprimée après un appel, quel que soit le statut.
- Isochronous USB I/O : pas de récupération d'erreur par le bus driver ; découpage en paquets par l'appelant/la fonction si > payload par trame.
- Descripteurs : device (18 octets : BcdUSB, classe/sous-classe/protocole, MaxPacketSize0, VID/PID, BcdDevice, index chaînes, NumConfigurations) ; configuration (TotalLength, NumInterfaces, ConfigurationValue, Attributes, MaxPower) ; NOT_FOUND si non configuré.
- [AGENT] Pour identifier une clé USB : lire VID/PID et classe 0x08 (mass storage), sous-classe 0x06 (SCSI transparent), protocole 0x50 (BOT). Utile pour la lecture seule / détection.

## Statut
- Lignes 1 à 36917 lues. Prochaine : ligne 36918 (suite GetInterfaceDescriptor, GetEndpointDescriptor, GetStringDescriptor, GetSupportedLanguages, UsbPortReset, 17.3 ...).

## Lignes 36918-37217 — 17.2.14 GetInterfaceDescriptor (fin), 17.2.15 GetEndpointDescriptor (index 0..15, EFI_USB_ENDPOINT_DESCRIPTOR), 17.2.16 GetStringDescriptor (LangID, StringID, buffer AllocatePool), 17.2.17 GetSupportedLanguages, 17.2.18 UsbPortReset (interdit sur hub), 17.3 USB Function Protocol (mode device/peripheral ; GUID 32D2963A-FE5D-4F30-B633-6E5DC55803CC ; révision 0x00010001 ; bulk USB 2.0 seulement, config/interface uniques) (début)
- Interface : InterfaceNumber, AlternateSetting, NumEndpoints, InterfaceClass/SubClass/Protocol. Endpoint : EndpointAddress, Attributes, MaxPacketSize, Interval.
- UsbPortReset : re-énumère (adresse + configuration) sans changer la topologie.
- [AGENT] USB Function (mode device) : hors périmètre pour l'agent hôte (clé USB = device, PC = hôte). Ne pas configurer le PC en mode device sans besoin explicite.
- EFI_USBFN_IO_PROTOCOL : DetectPort, ConfigureEnableEndpoints, GetEndpointMaxPacketSize, GetDeviceInfo, GetVendorIdProductId, AbortTransfer, Get/SetEndpointStallState, EventHandler, Transfer, GetMaxTransferSize, Allocate/FreeTransferBuffer, StartController/StopController, Set/GetEndpointPolicy.

## Statut
- Lignes 1 à 37217 lues. Prochaine : ligne 37218 (suite USBFN : DetectPort, Transfer, EventHandler, etc.).

## Lignes 37218-37517 — 17.3.2 DetectPort (EFI_USBFN_PORT_TYPE : Unknown 0, StandardDownstream, ChargingDownstream, DedicatedCharging, InvalidDedicatedCharging ; USB Battery Charging 1.2), 17.3.3 ConfigureEnableEndpoints (DeviceInfo ; ignore bMaxPacketSize0 et wMaxPacketSize), 17.3.4 GetEndpointMaxPacketSize (EFI_USB_ENDPOINT_TYPE Control 0, Bulk 2, Interrupt 3), 17.3.5 GetDeviceInfo (Id : SerialNumber, ManufacturerName, ProductName ; Unicode), 17.3.6 GetVendorIdProductId, 17.3.7 AbortTransfer (début)
- Port types : standard/charging/dedicated = règles de la spec Battery Charging 1.2 ; EfiUsbUnknownPort jamais renvoyé avec succès.
- [AGENT] Hors périmètre (mode device) ; lu pour complétude.
- ConfigureEnableEndpoints : active le port et démarre la réception d'événements USB (le RUN/STOP est mis à 1 ici).

## Statut
- Lignes 1 à 37517 lues. Prochaine : ligne 37518 (suite AbortTransfer, GetEndpointStallState, SetEndpointStallState, EventHandler, Transfer, Allocate/FreeTransferBuffer, Start/Stop, Set/GetEndpointPolicy).

## Lignes 37518-37817 — 17.3.7 AbortTransfer (fin ; EFI_USBFN_ENDPOINT_DIRECTION : HostOut 0 = DeviceRx, HostIn 1 = DeviceTx), 17.3.8 GetEndpointStallState, 17.3.9 SetEndpointStallState, 17.3.10 EventHandler (messages : None 0, SetupPacket, EndpointStatusChangedRx/Tx, BusEvent Detach/Attach/Reset/Suspend/Resume/Speed ; EFI_USBFN_TRANSFER_STATUS ; EFI_USBFN_TRANSFER_RESULT ; EFI_USB_BUS_SPEED Unknown 0 / Low 1 / Full 2 / High 3 / Super 4), 17.3.11 Transfer (début)
- Stall : TRUE = endpoint en stall ; INVALID_PARAMETER si direction incorrecte.
- EventHandler : à appeler en boucle ; payload selon message (udr, utr, ubs). Le buffer de Transfer doit être identique au Buffer du payload.
- [AGENT] Hors périmètre (mode device). Lu pour complétude.

## Statut
- Lignes 1 à 37817 lues. Prochaine : ligne 37818 (suite Transfer, GetMaxTransferSize, AllocateTransferBuffer, FreeTransferBuffer, StartController, StopController, SetEndpointPolicy, GetEndpointPolicy, puis chapitre 18 suite).

## Lignes 37818-38117 — 17.3.11 Transfer (fin : TX/RX, buffer réutilisable seulement après message EndpointStatusChanged), 17.3.12 GetMaxTransferSize, 17.3.13 AllocateTransferBuffer, 17.3.14 FreeTransferBuffer, 17.3.15 StartController (alimente, n'active pas le port), 17.3.16 StopController, 17.3.17 SetEndpointPolicy (début ; EFI_USBFN_POLICY_TYPE : Undefined 0, MaxTransactionSize, ZeroLengthTerminationSupport, ZeroLengthTermination)
- Transfer : retour immédiat ; le buffer doit être libéré seulement après le message de fin de transfert.
- SetEndpointPolicy : n'est appelable qu'avant StartController ou après StopController.
- [AGENT] Mode device, hors périmètre.

## Statut
- Lignes 1 à 38117 lues. Prochaine : ligne 38118 (suite SetEndpointPolicy, GetEndpointPolicy, fin de 17.3, puis 17.4 et chapitre 18 si présent).

## Lignes 38118-38417 — 17.3.17 SetEndpointPolicy (fin : MaxTransactionSize, ZeroLengthTermination : BOOLEAN ; défaut controller ne doit PAS activer ZLP auto), 17.3.18 GetEndpointPolicy, 17.3.19 diagramme ; CHAPITRE 18 DEBUGGER SUPPORT : 18.1 aperçu (agent debug distant, Debug Support + Debug Port), 18.2 EFI Debug Support (GUID 2755590C-6F3C-42FA-9EA4-A3BA543CDA25 ; Isa ; GetMaximumProcessorIndex ; RegisterPeriodicCallback ; RegisterExceptionCallback ; InvalidateInstructionCache) (début)
- Debug Support : callbacks en contexte interruption ; l'agent ne doit appeler AUCUN service EFI depuis l'interruption ; une seule instance par architecture ; InvalidateInstructionCache obligatoire après modification de code (breakpoints logiciels).
- ISA : IA32 0x014C, X64 0x8664, IPF 0x0200, EBC 0x0EBC, ARM 0x1C2, AArch64 0xAA64, RISC-V 0x5032/0x5064/0x5128, LoongArch 0x6232/0x6264.
- [AGENT] Très utile pour un agent bare-metal : le modèle « callback d'exception avec contexte modifiable » et la règle « pas de service EFI en interruption » s'appliquent aussi à nos handlers IDT (voir notes x64 APM vol2 à venir).
- [AGENT] Règle : après patch de code (int3), invalider l'I-cache sur la plage modifiée (x64 : serialisation / cpuid selon APM).

## Statut
- Lignes 1 à 38417 lues. Prochaine : ligne 38418 (suite RegisterPeriodicCallback, RegisterExceptionCallback, EFI_SYSTEM_CONTEXT x64/IA32, InvalidateInstructionCache, Debug Port).

## Lignes 38418-38717 — EFI_SYSTEM_CONTEXT_EBC (R0..R7, Flags, ControlFlags, Ip), EFI_SYSTEM_CONTEXT_RISCV32/64/128 (registres x0..x31 : Zero, Ra, Sp, Gp, Tp, T0-T6, S0FP, S1, A0-A7, S2-S11 ; Ft0-Ft11, Fs0-Fs11, Fa0-Fa7 en UINT128), EFI_SYSTEM_CONTEXT_IA32 (ExceptionData, FxSaveState, Dr0-Dr7, Cr0/Cr2/Cr3/Cr4, Eflags, Ldtr, Tr, Gdtr[2], Idtr[2], Eip, segments, Edi..Eax), EFI_FX_SAVE_STATE_IA32 (FCW/FSW/FTW/Opcode/Eip/Cs/DataOffset/Ds, ST0-7/MM0-7 sur 16 o, XMM0-7 sur 16 o, Reserved11[14*16]) (début de structure x64 non lue ici)
- [AGENT] FXSAVE layout IA-32 : FCW au décalage 0 (même format que la valeur FCW 0x037F citée dans le résumé x64) ; les XMM sont aux décalages 160.. (voir APM vol2/vol3 pour la référence définitive, ne pas se fier à cette note seule).
- [AGENT] Le contexte x64 (EFI_SYSTEM_CONTEXT_X64) est à lire à la suite : il décrit l'état complet à sauver/restaurer dans un handler d'exception bare-metal.

## Statut
- Lignes 1 à 38717 lues. Prochaine : ligne 38718 (suite FX_SAVE_STATE, EFI_SYSTEM_CONTEXT_X64, AArch64, EFI_PERIODIC_CALLBACK details).

## Lignes 38718-39017 — EFI_SYSTEM_CONTEXT_X64 (ExceptionData, FxSaveState, Dr0-3/Dr6/Dr7, Cr0/Cr2/Cr3/Cr4/Cr8, Rflags, Ldtr/Tr, Gdtr[2]/Idtr[2], Rip, segments, Rdi..Rax, R8..R15), EFI_FX_SAVE_STATE_X64 (idem IA-32 mais Rip 64 bits ; Reserved1[8]), contextes IPF (Itanium, hors périmètre), ARM (R0..R12, SP, LR, PC, CPSR, DFSR, DFAR, IFSR), AArch64 (X0..X28, FP, LR, SP, V0..V31, ELR, SPSR, FPSR, ESR, FAR), LoongArch64 (R0..R31, CRMD, PRMD, EUEN, MISC, ECFG, ESTAT, ERA, BADV, BADI) ; 18.2.4 RegisterPeriodicCallback (désenregistrer = NULL ; codes SUCCESS, ALREADY_STARTED, OUT_OF_RESOURCES), 18.2.5 RegisterExceptionCallback (EFI_EXCEPTION_TYPE = INTN ; codes EBC 0..9+ : UNDEFINED 0, DIVIDE_ERROR 1, DEBUG 2, BREAKPOINT 3, OVERFLOW 4, INVALID_OPCODE 5, STACK_FAULT 6, ALIGNMENT_CHECK 7, INSTRUCTION_ENCODING 8, BAD_BREAK 9, SINGLE_STEP …)
- [AGENT] Structure x64 : Rip est placé après FxSaveState/Dr/Cr/Rflags ; l'ordre exact figure dans le texte mais les décalages doivent être vérifiés contre l'APM vol2 avant tout code ASM.
- [AGENT] Règle : le handler doit sauver TOUT le contexte, appliquer les modifications à la fin, restaurer depuis le contexte modifié.
- RegisterPeriodicCallback : si l'interruption sert aussi au timer EFI, appeler le callback AVANT tout traitement EFI et passer le contrôle au handler firmware de façon transparente.

## Statut
- Lignes 1 à 39017 lues. Prochaine : ligne 39018 (suite EXCEPT_EBC_SINGLE_STEP, codes IA32/X64 (#DE, #DB, #BP, #OF, #BR, #UD, #NM, #DF, #TS, #NP, #SS, #GP, #PF, #MF, #AC, #MC, #XF), RegisterExceptionCallback fin, InvalidateInstructionCache, Debug Port).

## Lignes 39018-39317 — 18.2.5 suite (EBC SINGLE_STEP = fin de liste EBC), codes IA-32 et X64 (vecteurs IDT), IPF (hors périmètre), début ARM
- Vecteurs x64/IA-32 (identiques aux numéros IDT) : DIVIDE_ERROR 0 (#DE), DEBUG 1 (#DB), NMI 2, BREAKPOINT 3 (#BP), OVERFLOW 4 (#OF), BOUND 5 (#BR), INVALID_OPCODE 6 (#UD), DOUBLE_FAULT 8 (#DF), INVALID_TSS 10 (#TS), SEG_NOT_PRESENT 11 (#NP), STACK_FAULT 12 (#SS), GP_FAULT 13 (#GP), PAGE_FAULT 14 (#PF), FP_ERROR 16 (#MF x87), ALIGNMENT_CHECK 17 (#AC), MACHINE_CHECK 18 (#MC), SIMD 19 (#XM/#XF). Vecteur 7 (#NM) et 9, 15 non listés.
- [AGENT] Ces numéros sont la référence pour l'IDT de notre agent ; #DF doit utiliser une IST/pile dédiée (APM vol2 à confirmer).
- [AGENT] #PF : l'adresse fautive est dans CR2 (champ Cr2 du contexte) ; le code d'erreur est poussé sur la pile (ExceptionData).
- IPF : hors périmètre (Itanium).
- ARM : hors périmètre.

## Statut
- Lignes 1 à 39317 lues. Prochaine : ligne 39318 (suite contexte ARM, AArch64 si nécessaire, puis InvalidateInstructionCache, Debug Port ...).

## Lignes 39318-39617 — codes ARM, AArch64, RISC-V, LoongArch (hors périmètre, lus pour complétude), 18.2.5 RegisterExceptionCallback (fin : NULL = désenregistrer, pas de chaînage de handlers ; codes SUCCESS, ALREADY_STARTED, OUT_OF_RESOURCES), 18.2.6 InvalidateInstructionCache (début)
- Règle : « No chaining of exception handlers is allowed » — un seul callback par exception et par processeur.
- [AGENT] Pour notre agent x64 : un seul handler par vecteur, pas de chaînage avec le firmware après ExitBootServices.

## Statut
- Lignes 1 à 39617 lues. Prochaine : ligne 39618 (suite InvalidateInstructionCache, 18.3 Debug Port Protocol).

## Lignes 39618-39917 — 18.2.6 InvalidateInstructionCache (fin), 18.3 EFI Debugport Protocol (GUID EBA4E8D2-3858-41EC-A281-2647BA9660D0 ; Reset, Write(Timeout µs), Read(Timeout µs), Poll ; codes), 18.3.7 Debugport Device Path (messaging vendor GUID = GUID du protocole ; ex. PciRoot(0)/Pci(0x1f,0)/ACPI(PNP0501,0)/UART(115200,N,8,1)/DebugPort()), 18.3.8 variable DEBUGPORT (défauts PNP0501 : 115200, 8N1, sans flux), 18.4 Debug Support Table (début : localisation EFI_SYSTEM_TABLE_POINTER sur frontière 4 MiB, CRC32, GUID table image debug 49152E77-1ADA-4764-B7A2-7AFEFED95E8B, UpdateStatus IN_PROGRESS 0x01 / MODIFIED 0x02)
- Debugport : Poll ne retire aucune donnée ; Read stoppe sur timeout/overrun ; aucune fonction ne doit appeler de service EFI (contexte interruption).
- [AGENT] Pour un agent qui expose un port de débogage : Debugport Device Path + variable DEBUGPORT ; hors périmètre si on ne traite pas de série. Utile : défauts UART 115200 8N1.
- [AGENT] Recherche du System Table Pointer : scanner les frontières 4 MiB depuis le haut de la RAM (signature 0x5453595320494249 ; CRC32 du bloc, champ à zéro pendant calcul). Utile pour un agent qui doit retrouver la table système sans firmware.
- [AGENT] La table d'images debug permet de lister les EFI images chargées sans services : utile au diagnostic post-boot.

## Statut
- Lignes 1 à 39917 lues. Prochaine : ligne 39918 (suite EFI_DEBUG_IMAGE_INFO, EFI_DEBUG_IMAGE_INFO_NORMAL, SMM; chapitre 18 fin ; puis chapitre 19+).

## Lignes 39918-40217 — 18.4.3 suite (EFI_DEBUG_IMAGE_INFO_TABLE_HEADER : UpdateStatus, TableSize, EfiDebugImageInfoTable ; type NORMAL 0x01 ; EFI_DEBUG_IMAGE_INFO_NORMAL = LoadedImageProtocolInstance + ImageHandle), CHAPITRE 19 COMPRESSION ALGORITHM (19.1 aperçu LZ77 + Huffman canonique ; 19.2 format : bit order MSB first, en-tête 2×UINT32 little-endian (Compressed Size, Original Size), blocs sans padding, terminateur 0 ; Block Header : Block Size 16 bits, tailles de tableaux Extra(5 bits)/Position(4)/Char&Len, codes de longueur ; Block Body : Char&Len 0..509 (= (Len-3)|0x100), Position 0..8190 préfixée par code de longueur ; 19.3 compresseur (String Info Log en arbres, Output, Block Buffer))
- [AGENT] Décompresseur UEFI : utile seulement si on charge des images compressées (ROM/FV). Pour notre agent : à implémenter SEULEMENT si on lit des images compressées ; sinon ignorer.
- [AGENT] Règle de lecture : Compressed Size = octets de (blocs + bits de remplissage + terminateur). Tous les champs en little-endian.
- Points de vigilance pour un décompresseur : longueur de code ≤ 16 bits ; Block Size = nb caractères + nb pointeurs ; string length ∈ [3, 256] ; distance ∈ [0, 8190].

## Statut
- Lignes 1 à 40217 lues. Prochaine : ligne 40218 (suite 19.3.3 Huffman code generation, puis décompresseur / Appendix H, puis protocol EFI_DECOMPRESS).

## Lignes 40218-40517 — 19.3.3 Huffman (génération d'arbre, ajustement LengthCount ≤ 16 bits, génération des codes canoniques), 19.4 décompresseur (pseudo-code : bloc, Char&Len <256 = octet, sinon longueur + position), 19.5 EFI_DECOMPRESS_PROTOCOL (GUID D8117CFE-94A6-11D4-9A3A-0090273FC14D ; GetInfo ; Decompress ; pas d'allocation mémoire dans le décompresseur), CHAPITRE 20 ACPI PROTOCOLS (20.1 EFI_ACPI_TABLE_PROTOCOL GUID FFE06BDD-6107-46A6-7BB2-5A9C7EC5275C ; InstallAcpiTable ; UninstallAcpiTable ; codes) (début)
- Décompresseur : Original Size = 4 premiers octets ; GetInfo ne valide pas le contenu ; INVALID_PARAMETER si format invalide.
- [AGENT] Décompresser ne doit pas appeler AllocatePool/AllocatePages : Destination et Scratch fournis par l'appelant (bon modèle pour un agent sans allocateur).
- ACPI_TABLE_PROTOCOL : InstallAcpiTable(buffer, size, TableKey) — table SDT ou FACS ; si signature déjà présente, nouvelle table non remplacée (ou ACCESS_DENIED selon plateforme) ; checksum recalculé ; RSDT/XSDT réinstallée dans la table de configuration.
- [AGENT] Cross-check : nos notes ACPI_LECTURE.md — le firmware lié RSDT/XSDT via configuration table ; après ExitBootServices, la table reste accessible par GUID ACPI 2.0 (8868E871-E4F1-11D3-BC22-0080C73C8881) sinon ACPI 1.0.

## Statut
- Lignes 1 à 40517 lues. Prochaine : ligne 40518 (suite UninstallAcpiTable, puis 21 Unicode Collation 2 (1.1+), StrLwr, etc.).

## Lignes 40518-40817 — 20.3 fin, CHAPITRE 21 STRING SERVICES : 21.1 Unicode Collation (GUID A4C751FC-23AE-4C3E-92E9-4964CF63F349, StriColl (0 = égal, <0, >0), MetaiMatch (joker * ? [..] [a-z]), StrLwr, StrUpr, FatToStr, StrToFat (retour TRUE si substitution '_'), SupportedLanguages RFC 4646), 21.2 Regular Expression Protocol (début, GUID B3F79D9A-436C-DC11-B052-CD85DF524CE6, MatchString, GetInfo)
- [AGENT] Comparaison de noms FAT insensible à la casse : StriColl/StrUpr pour rechercher un fichier (ex. \EFI\BOOT\BOOTX64.EFI) — à reproduire dans notre lecteur FAT si besoin.
- [AGENT] Projet : les noms FAT 8.3 sont en majuscules ; LFN sensibles en stockage (voir 13.3.1.2).

## Statut
- Lignes 1 à 40817 lues. Prochaine : ligne 40818 (suite Regular Expression MatchString/GetInfo, puis chapitre 22 suivant).

## Lignes 40818-41116 — 21.2.2 MatchString (EFI_REGEX_CAPTURE {CapturePtr, Length}), 21.2.3 GetInfo (EFI_REGEX_SYNTAX_TYPE = GUID), 21.2.4 GUIDs de syntaxe (POSIX ERE, Perl, ECMA-262, et variantes ASCII) ; CHAPITRE 22 EFI BYTE CODE VM (22.1 objectifs, indépendance processeur, Option ROM EBC, compat legacy, taille illimitée), 22.2 ordre mémoire fort, 22.3 registres VM (R0 pile, R1-R3 préservés, R4-R7 scratch, R7 retour ; FLAGS bit0 C, bit1 SS ; IP bit0 = 0), 22.4 indexation naturelle (encodage 16/32/64 bits, sign + w + c + n), 22.5 opérandes (direct, @Rn indirect, @Rn(+n,+c) indexé, immédiat), 22.6 syntaxe (début)
- [AGENT] EBC = machine virtuelle pour option ROM. Hors périmètre agent (on ne charge pas d'option ROM EBC). Lu pour complétude.
- Règle utile pour tout code bas niveau : alignement des instructions au mot ; bit 0 de l'IP toujours 0.

## Statut
- Lignes 1 à 41116 lues. Prochaine : ligne 41117 (suite syntaxe EBC, jeu d'instructions 22.7+).

## Lignes 41117-41416 — 22.7 encodage (opcode byte : bits 0-5 opcode, bit 6 taille, bit 7 immédiat présent ; operand byte : bits 0-2 Op1, bit 3 Op1 indirect, bits 4-6 Op2, bit 7 Op2 indirect), 22.7.3 immédiats/index, 22.8 jeu d'instructions : ADD (0x0C), AND (0x14), ASHR (0x19), BREAK (codes : 0 runaway, 1 version → R7 = 0x00010000, 3 debug, 4 syscall, 5 create thunk, 6 compiler version), CALL (début)
- [AGENT] Hors périmètre (EBC). Note générale : les opérations 32 bits « clear upper 32 » quand Op1 est direct — à retenir pour nos emulations.

## Statut
- Lignes 1 à 41416 lues. Prochaine : ligne 41417 (suite CALL, CMP, DIV, EXTND, JMP, LOCK, MOV, MOVI, MUL, NEG, NOT, OR, POP, PUSH, RET, SHL, SHR, STORESP, SUB, XOR, XADD, DATA ... ).

## Lignes 41417-41716 — 22.8.5 CALL (fin : CALL32/CALL64/CALLEX, opcode 0x03, pile R0 -= 16 puis retour empilé), 22.8.6 CMP (opcodes 0x05 eq, 0x06 lte, 0x07 gte, 0x08 ulte, 0x09 ugte → Flags.C), 22.8.7 CMPI (0x2D-0x31), 22.8.8 DIV (0x10 ; division par 0 = exception), 22.8.9 DIVU (0x11), 22.8.10 EXTNDB (0x1A) (début)
- Exception division par zéro : Operand 2 = 0 → exception (à traiter explicitement dans un VM/émulateur).
- [AGENT] Hors périmètre EBC.

## Statut
- Lignes 1 à 41716 lues. Prochaine : ligne 41717 (suite EXTNDB, EXTNDD, EXTNDW, JMP, JMP8, LOCK, MOD, MODU, MOV, MOVI, MOVIn, MOVn, MUL, MULU, NEG, NOT, OR, POP, PUSH, RET, SHL, SHR, STORESP, SUB, XOR, XADD, DATA ...).

## Lignes 41717-42015 — EBC : EXTNDB (fin, 0x1A), EXTNDD (0x1C), EXTNDW (0x1B), 22.8.13 JMP (opcode 0x01, JMP32/JMP64, cs/cc ; alignment exception si cible impaire), 22.8.14 JMP8 (0x02, offset signé en mots), 22.8.15 LOADSP (0x29, seul Flags autorisé), 22.8.16 MOD (0x12) (début)
- [AGENT] Hors périmètre EBC.

## Statut
- Lignes 1 à 42015 lues. Prochaine : ligne 42016 (suite MOD, MODU, MOV, MOVI, MOVIn, MOVn, MUL, MULU, NEG, NOT, OR, POP, PUSH, RET, SHL, SHR, STORESP, SUB, XOR, XADD, DATA, EBC appendix).

## Lignes 42016-42315 — EBC : MOD (fin, 0x12), MODU (0x13), MOV (opcodes 0x1D-0x24, 0x28 MOVqq), MOVI (0x37 ; immédiat signé étendu), MOVIn (0x38), MOVn (0x32 MOVnw, 0x33 MOVnd), MOVREL (début)
- [AGENT] Hors périmètre EBC.

## Statut
- Lignes 1 à 42315 lues. Prochaine : ligne 42316 (suite MOVREL, MUL, MULU, NEG, NOT, OR, POP, PUSH, RET, SHL, SHR, STORESP, SUB, XOR, XADD, ... fin 22, chapitre 23 ?).

## Lignes 42316-42615 — EBC : MOVREL (fin, 0x39), MOVsn (0x25 MOVsnw, 0x26 MOVsnd), MUL (0x0E), MULU (0x0F), NEG (0x0B), NOT (0x0A ; début de la fiche NOT)
- [AGENT] Hors périmètre EBC.

## Statut
- Lignes 1 à 42615 lues. Prochaine : ligne 42616 (suite NOT, OR, POP, PUSH, RET, SHL, SHR, STORESP, SUB, XOR, XADD, ... fin 22, chapitre 23).

## Lignes 42616-42915 — EBC : NOT (fin, 0x0A), OR (0x15), POP (0x2C), POPn (0x36), PUSH (0x2B), PUSHn (0x35), RET (opcode 0x04 ; alignement 16 bits de l'adresse de retour sinon exception d'alignement), SHL (0x17) (début)
- [AGENT] Hors périmètre EBC.

## Statut
- Lignes 1 à 42915 lues. Prochaine : ligne 42916 (suite SHL, SHR, STORESP, SUB, XOR, XADD, DATA ; puis fin chapitre 22 ; chapitres 23+).

## Lignes 42916-43215 — EBC : SHL (fin, 0x17), SHR (0x18), STORESP (0x2A), SUB (0x0D), XOR (0x16) ; 22.9 conventions (appels hors VM = CALLEX ; paramètres CDECL ; retour ≤ 8 octets dans R7 ; binaire PE32+ avec section VarBss) ; 22.10 exigences (images PE32+ ; transitions EBC↔natif ; retours ≤ 64 bits ; structures non supportées) ; 22.11 EFI_EBC_PROTOCOL (GUID 13AC6DD1-73D0-11D4-B06B-00AA00BD6DE7 ; CreateThunk, UnloadImage, RegisterICacheFlush, GetVersion) (début)
- [AGENT] Hors périmètre EBC (on ne charge pas d'option ROM EBC).

## Statut
- Lignes 1 à 43215 lues. Prochaine : ligne 43216 (suite CreateThunk, UnloadImage, RegisterICacheFlush, GetVersion ; puis chapitre 23+).

## Lignes 43216-43515 — 22.11.2 CreateThunk (entry point ≥ 2 octets alignés), 22.11.3 UnloadImage, 22.11.4 RegisterICacheFlush (EBC_ICACHE_FLUSH(Start, Length)), 22.11.5 GetVersion ; 22.12 EBC Tools (compilateur C, CALLEX, convention CDECL, retours R7, thunking EBC↔natif, linker IMAGE_FILE_MACHINE_EBC 0x0EBC, subsystems 10/11/12, relocations EBC 0..4, image loader, debug) ; 22.13 exceptions VM (div0, debug break code 3, opcode invalide, stack fault, alignement, encodage, bad break code 0, undefined) (début)
- [AGENT] Hors périmètre EBC.

## Statut
- Lignes 1 à 43515 lues. Prochaine : ligne 43516 (suite 22.13.8 undefined, puis chapitre 23 suivant).

## Lignes 43516-43815 — 22.14 Option ROM Formats (fin), CHAPITRE 23 FIRMWARE UPDATE AND REPORTING : 23.1 Firmware Management Protocol (GUID 86C77A67-0B97-4633-A187-49104D0685C7 ; GetImageInfo (obligatoire), GetImage, SetImage, CheckImage, GetPackageInfo, SetPackageInfo — non supportés → EFI_UNSUPPORTED) ; 23.1.2 GetImageInfo (EFI_FIRMWARE_IMAGE_DESCRIPTOR v3 : ImageIndex, ImageTypeId, ImageId, Version, Size, AttributesSupported/Setting, Compatibilities, LowestSupportedImageVersion, LastAttemptVersion/Status, HardwareInstance, Dependencies ; attributs UPDATABLE 0x1, RESET_REQUIRED 0x2, AUTHENTICATION_REQUIRED 0x4, IN_USE 0x8, UEFI_IMAGE 0x10, DEPENDENCY 0x20) (début)
- [AGENT] Règle projet : jamais de SetImage/mise à jour firmware sans accord explicite. GetImageInfo est en lecture seule : acceptable pour inventaire.
- Compatibilité : ne jamais autoriser une mise à jour qui baisse le niveau de compatibilité (risque de rendre le matériel inopérant).
- Authentification : WIN_CERTIFICATE_UEFI_GUID + MonotonicCount (anti-rejeu), CertType EFI_CERT_TYPE_PKCS7_GUID.

## Statut
- Lignes 1 à 43815 lues. Prochaine : ligne 43816 (suite 23.1.2 : Image Attribute Authentication, Dependency, Figure 23.1 ; 23.1.3 GetImage ...).

## Lignes 43816-44115 — 23.1.3 GetImage (copie de l'image courante ; sauvegarde/rollback ; codes), 23.1.4 SetImage (EFI_FIRMWARE_MANAGEMENT_UPDATE_IMAGE_PROGRESS 1..100 %, VendorCode, AbortReason ; validations : authentification → EFI_SECURITY_VIOLATION, image non supportée → EFI_ABORTED ; codes), 23.1.5 CheckImage (IMAGE_UPDATABLE_VALID 0x1, INVALID 0x2, INVALID_TYPE 0x4, INVALID_OLD 0x8 (downgrade refusé), VALID_WITH_VENDOR_CODE 0x10), 23.1.6 GetPackageInfo (PACKAGE_ATTRIBUTE_VERSION_UPDATABLE 0x1, RESET_REQUIRED 0x2, AUTHENTICATION_REQUIRED 0x4 ; PackageVersion 0xFFFFFFFF non supporté / 0xFFFFFFFE comparer par nom / 0xFFFFFFFD mise à jour en cours), 23.1.7 SetPackageInfo (début)
- [AGENT] Projet : GetImage et CheckImage sont en lecture (acceptables pour inventaire, sans écriture). SetImage/SetPackageInfo = écriture firmware : INTERDIT sans accord explicite.
- [AGENT] Référence utile : CheckImage renvoie INVALID_OLD si downgrade → le firmware refuse le retour arrière de version par défaut.

## Statut
- Lignes 1 à 44115 lues. Prochaine : ligne 44116 (suite SetPackageInfo, Image attributes/auth, 23.2 Capsule, 23.3 ...).

## Lignes 44116-44415 — 23.1.7 SetPackageInfo (fin : PackageVersionName ≤ MaxLen, codes), 23.2 Dependency Expression Instruction Set (FMP depex : notation postfixe ; stack overflow/underflow → FALSE ; opcode invalide → FALSE ; GUID introuvable → FALSE ; types mixtes → FALSE ; END obligatoire), tableau des opcodes (0x00 PUSH_GUID, 0x01 PUSH_VERSION, 0x02 DECLARE_VERSION_NAME, 0x03 AND, 0x04 OR, 0x05 NOT, 0x06 TRUE, 0x07 FALSE, 0x08 EQ, 0x09 GT, 0x0A GTE, 0x0B LT, 0x0C LTE, 0x0D END, 0x0E DECLARE_LENGTH), 23.2.1-23.2.11 détails (début)
- [AGENT] Projet : depex = logique de dépendance entre images firmware ; hors périmètre (pas d'écriture firmware).
- Lecture : GUIDs potentiellement non alignés → lecture octet par octet sur architectures strictes.

## Statut
- Lignes 1 à 44415 lues. Prochaine : ligne 44416 (suite LT, LTE, END, DECLARE_LENGTH, 23.3 Capsule / Firmware update / Reporting).

## Lignes 44416-44715 — 23.2.12 LT, 23.2.13 LTE, 23.2.14 END (dernier opcode), 23.2.15 DECLARE_LENGTH (premier opcode), 23.3 Capsules FMP (GUID 6DCBD5ED-E82D-4C44-BDA1-7194199AD92A ; EFI_FIRMWARE_MANAGEMENT_CAPSULE_HEADER {Version 1, EmbeddedDriverCount, PayloadItemCount, ItemOffsetList} ; EFI_FIRMWARE_MANAGEMENT_CAPSULE_IMAGE_HEADER v3 {Version 3, UpdateImageTypeId, UpdateImageIndex, UpdateImageSize, UpdateVendorCodeSize, UpdateHardwareInstance, ImageCapsuleSupport (AUTH 0x1, DEPENDENCY 0x2)} ; traitement 1-15 : UpdateCapsule, PERSIST_ACROSS_RESET, drivers via LoadImage, correspondance ImageTypeId (+HardwareInstance v2), SetImage, EFI_NOT_READY/EFI_ABORTED ; paquetage packé non aligné), 23.4 EFI System Resource Table (GUID B122A263-3661-4F68-9929-78F8B0D62180 ; FwResourceCount ≠ 0 ; FwResourceCountMax ; FwResourceVersion = 1 ; entrées : FwClass, FwType, FwVersion, LowestSupportedFwVersion, CapsuleFlags, LastAttemptVersion, LastAttemptStatus) (début)
- [AGENT] Projet : aucune capsule ni UpdateCapsule sans accord explicite (écriture firmware). ESRT = lecture seule utile pour inventaire.
- [AGENT] Règle capsule : PERSIST_ACROSS_RESET nécessaire pour déverrouiller l'écriture flash ; traitement avant EVENT_GROUP_READY_TO_BOOT.

## Statut
- Lignes 1 à 44715 lues. Prochaine : ligne 44716 (suite ESRT : Firmware Type Definitions, LastAttemptStatus, 23.5 Capsule Definition / UpdateCapsule / Delivery via file).

## Lignes 44716-45015 — 23.4 ESRT fin (LastAttemptStatus : SUCCESS 0, UNSUCCESSFUL 1, INSUFFICIENT_RESOURCES 2, INCORRECT_VERSION 3, INVALID_FORMAT 4, AUTH_ERROR 5, PWR_EVT_AC 6, PWR_EVT_BATT 7, UNSATISFIED_DEPENDENCIES 8, vendeur 0x1000-0x4000), types FW (UNKNOWN 0, SYSTEMFIRMWARE 1, DEVICEFIRMWARE 2, UEFIDRIVER 3), 23.4.2-23.4.4 (règles de mise à jour ESRT, mapping FMP → ESRT), 23.5 Capsule JSON (GUID 67D6F4CD-D6B8-4573-BF4A-DE5E252D61AE ; EFI_JSON_CAPSULE_HEADER {Version, CapsuleId, PayloadLength, Payload} ; EFI_JSON_CAPSULE_CONFIG_DATA ; traitement 1-9 : table de configuration EFI_JSON_CAPSULE_DATA_TABLE_GUID), CHAPITRE 24 NETWORK (24.1 Simple Network Protocol : GUID A19832B9-AC25-11D3-9A2D-0090273FC14D, Revision 0x10000 ; Start, Stop, Initialize, Reset, Shutdown, ReceiveFilters, StationAddress, Statistics, MCastIpToMac, NvData, GetStatus, Transmit, Receive, WaitForPacket, Mode) (début)
- [AGENT] Projet : pas d'écriture de config/firmware sans accord ; lecture ESRT/config = inventaire.
- [AGENT] Réseau : hors périmètre des cibles USB/NVMe/HDA ; à noter pour une éventuelle sortie réseau d'un agent (SNP = paquets bruts, Initialize alloue les buffers).

## Statut
- Lignes 1 à 45015 lues. Prochaine : ligne 45016 (suite SNP : Start/Stop/Initialize/Reset/Shutdown/Receive/Transmit etc.).

## Lignes 45016-45315 — 24.1 SNP (fin des membres : NvData, GetStatus, Transmit, Receive, WaitForPacket, Mode), 24.1.1 EFI_SIMPLE_NETWORK_MODE (State, HwAddressSize, MediaHeaderSize, MaxPacketSize, NvRamSize/AccessSize (1,2,4,8), ReceiveFilterMask/Setting, MaxMCastFilterCount, MCastFilter[16], CurrentAddress, BroadcastAddress, PermanentAddress, IfType, MacAddressChangeable, MultipleTxSupported, MediaPresentSupported, MediaPresent) ; état Stopped 0 / Started 1 / Initialized 2 ; filtres RECEIVE_UNICAST 0x01, MULTICAST 0x02, BROADCAST 0x04, PROMISCUOUS 0x08, PROMISCUOUS_MULTICAST 0x10 ; 24.1.2 Start, 24.1.3 Stop, 24.1.4 Initialize (buffers Rx/Tx), 24.1.5 Reset (vide files, conserve filtres/adresse) (début)
- Note : le réseau 32 bits au-dessus de 4 GiB est double-bufferisé (perf).
- [AGENT] Projet : hors périmètre réseau.

## Statut
- Lignes 1 à 45315 lues. Prochaine : ligne 45316 (suite Reset codes, Shutdown, ReceiveFilters, StationAddress, Statistics, MCastIPtoMAC, NvData, GetStatus, Transmit, Receive).

## Lignes 45316-45615 — 24.1.5 Reset (codes), 24.1.6 Shutdown (libère buffers, interruptions désactivées), 24.1.7 ReceiveFilters (Enable puis Disable, Disable prioritaire ; ResetMCastFilter ; NOTE : désactiver tout le matériel exige Shutdown + Stop), 24.1.8 StationAddress (Reset/New), 24.1.9 Statistics (EFI_NETWORK_STATISTICS : RxTotalFrames … RxCrcErrorFrames, RxTotalBytes, TxTotalFrames … ; -1 = non disponible) (début)
- [AGENT] Projet : hors périmètre réseau.

## Statut
- Lignes 1 à 45615 lues. Prochaine : ligne 45616 (suite EFI_NETWORK_STATISTICS, MCastIPtoMAC, NvData, GetStatus, Transmit, Receive, WaitForPacket).

## Lignes 45616-45915 — 24.1.9 Statistics (fin : EFI_NETWORK_STATISTICS 25 compteurs ; Reset ; codes), 24.1.10 MCastIPtoMAC, 24.1.11 NvData (Offset/BufferSize multiples de NvRamAccessSize, BufferSize multiple de 2), 24.1.12 GetStatus (IRQ : RECEIVE 0x01, TRANSMIT 0x02, COMMAND 0x04, SOFTWARE 0x08 ; TxBuf recyclé), 24.1.13 Transmit (HeaderSize = MediaHeaderSize ; non-bloquant, EFI_NOT_READY si occupé ; buffer à ne pas modifier avant émission), 24.1.14 Receive (début)
- [AGENT] Projet : hors périmètre réseau. Règle générique intéressante : Transmit non-bloquant ; boucle GetStatus pour recycler les buffers (modèle de ring à recycler, cf. xHCI/NVMe).

## Statut
- Lignes 1 à 45915 lues. Prochaine : ligne 45916 (suite Receive, WaitForPacket, 24.2 PXE, 24.3 BIS, 24.4 HTTP Boot...).

## Lignes 45916-46215 — 24.1.14 Receive (fin : codes, BufferSize entrée/sortie, EFI_NOT_READY si file vide), 24.2 Network Interface Identifier (GUID 1ACED566-76ED-4218-BC81-767F1F977A89, rév 0x00020000 ; Id, ImageAddr, ImageSize, StringId "UNDI", Type, MajorVer, MinorVer, Ipv6Supported, IfNum ; EfiNetworkInterfaceUndi = 1), 24.3 PXE Base Code (GUID 03C4E603-AC28-11D3-9A2D-0090273FC14D, rév 0x10000 ; Start, Stop, Dhcp, Discover, Mtftp, UdpWrite, UdpRead, SetIpFilter, Arp, SetParameters, SetStationIp, SetPackets, Mode ; EFI_PXE_BASE_CODE_MODE : Started, Ipv6Available/Supported, UsingIpv6, BisSupported/Detected, AutoArp, SendGUID, DhcpDiscoverValid, DhcpAckReceived, ProxyOfferReceived, PxeDiscoverValid, PxeReplyReceived, PxeBisReplyReceived, IcmpErrorReceived, TftpErrorReceived, MakeCallbacks, TTL, ...) (début)
- [AGENT] Projet : PXE/réseau hors périmètre ; ne pas tenter de boot réseau.

## Statut
- Lignes 1 à 46215 lues. Prochaine : ligne 46216 (suite PXE Mode : ToS, ..., routes ARP, DHCP packets, Start/Stop/Dhcp/...).

## Lignes 46216-46515 — 24.3.1 EFI_PXE_BASE_CODE_MODE (fin : ToS, StationIp, SubnetMask, paquets cachés DhcpDiscover/DhcpAck/ProxyOffer/PxeDiscover/PxeReply/PxeBisReply, IpFilter, ArpCache[8], RouteTable[8], IcmpError, TftpError ; drapeaux détaillés), types EFI_PXE_BASE_CODE_UDP_PORT (UINT16), EFI_IPv4_ADDRESS (4 o), EFI_IPv6_ADDRESS (16 o), EFI_IP_ADDRESS (union), EFI_MAC_ADDRESS (32 o), 24.3.2 DHCP packet types (DHCPv4 : BOOTP 236 octets + DhcpMagik + options 56 ; DHCPv6 : MessageType:8, TransactionId:24, options 1024 ; EFI_PXE_BASE_CODE_PACKET = union Raw[1472]) (début)
- [AGENT] Projet : PXE hors périmètre.
- [AGENT] Règle : champs réseau en ordre réseau (big-endian).

## Statut
- Lignes 1 à 46515 lues. Prochaine : ligne 46516 (suite ICMP_ERROR, TFTP_ERROR, PXE Base Code functions Start/Stop/Dhcp/Discover/Mtftp/...).

## Lignes 46516-46815 — 24.3.2 fin (ICMP error, TFTP error), 24.3.3 IP Receive Filter (MAX_IPCNT 8 ; FILTER_STATION_IP 0x1, BROADCAST 0x2, PROMISCUOUS 0x4, PROMISCUOUS_MULTICAST 0x8), 24.3.4 ARP entry / route entry, 24.3.5 UDP opflags (ANY_SRC_IP 0x1, ANY_SRC_PORT 0x2, ANY_DEST_IP 0x4, ANY_DEST_PORT 0x8, USE_FILTER 0x10, MAY_FRAGMENT 0x20 ; DEFAULT_TTL 64, DEFAULT_ToS 0), tags PXE (94 = NII, 93 = architecture, 60 = class id), 24.3.6 PXE Start (UseIpv6 ; codes), 24.3.7 PXE Stop (début)
- [AGENT] Projet : PXE hors périmètre.

## Statut
- Lignes 1 à 46815 lues. Prochaine : ligne 46816 (suite Stop codes, Dhcp, Discover, Mtftp, UdpWrite/Read, SetIpFilter, Arp, SetParameters, SetStationIp, SetPackets, 24.4 BIS ...).

## Lignes 46816-47115 — 24.3.8 Dhcp (SortOffers ; ≥31 s ; codes), 24.3.9 Discover (Type/bootstrap, Layer, UseBis, Info : UseMCast/UseBCast/UseUCast/MustUseList/ServerMCastIp/SrvList ; ≥10 s), 24.3.10 Mtftp (TFTP_OPCODE : GET_FILE_SIZE, READ_FILE, WRITE_FILE, READ_DIRECTORY, MTFTP_* ; MTFTP_INFO : MCastIp, CPort, SPort, ListenTimeout, TransmitTimeout ; BlockSize ≥ 512 ; format répertoire "size yyyy-mm-dd hh:mm:ss" UTC), 24.3.11 UdpWrite (début)
- [AGENT] Projet : PXE/TFTP hors périmètre.
- Note générique utile : toujours demander la taille (GET_FILE_SIZE) avant de lire un gros fichier.

## Statut
- Lignes 1 à 47115 lues. Prochaine : ligne 47116 (suite UdpWrite, UdpRead, SetIpFilter, Arp, SetParameters, SetStationIp, SetPackets, 24.4 BIS...).

## Lignes 47116-47415 — 24.3.11 UdpWrite (fin), 24.3.12 UdpRead (tables de filtres : destination IP, port, source IP/port ; BUFFER_TOO_SMALL = paquet perdu), 24.3.13 SetIpFilter (filtre logiciel USE_FILTER ; Dhcp/Discover/Mtftp réinitialisent le filtre), 24.3.14 Arp (IPv4 seulement), 24.3.15 SetParameters (AutoArp, SendGUID, TTL, ToS, MakeCallback), 24.3.16 SetStationIp (début de SetPackets)
- [AGENT] Projet : PXE/UDP hors périmètre.

## Statut
- Lignes 1 à 47415 lues. Prochaine : ligne 47416 (suite SetPackets, puis 24.4 BIS, 24.5 HTTP Boot).

## Lignes 47416-47715 — 24.3.17 SetPackets (fin), 24.3.18 Netboot6 (IPv6 : DHCPv6 SOLICIT/ADVERTISE/REQUEST/REPLY, Proxy DHCP6 port 4011, options BootFileURL 59, BootFileParameters 60, VendorClass 16, ARCH 61, NII 62, DUID-UUID 4), 24.4 PXE Callback (GUID 245DCA21-FB7B-11D3-8F01-00A0C9693B72 ; callback ≤ 5 ms ; CONTINUE/ABORT ; fonctions DHCP/DISCOVER/MTFTP/UDP_WRITE/UDP_READ/ARP/IGMP), 24.5 BIS (GUID 0B64AAB0-5429-11D4-9816-00A0C91FADCF ; Initialize, Shutdown, Free, GetBootObjectAuthorizationCertificate/CheckFlag/UpdateToken, GetSignatureInfo, UpdateBootObjectAuthorization, VerifyBootObject, VerifyObjectWithCredential) (début)
- [AGENT] Projet : réseau/PXE/BIS hors périmètre. Règle générique : un callback de polling ne doit pas dépasser 5 ms.

## Statut
- Lignes 1 à 47715 lues. Prochaine : ligne 47716 (suite BIS : Initialize codes, Shutdown, Free, certificates, VerifyBootObject, 24.6 HTTP Boot ...).

## Lignes 47716-48015 — 24.5.2 BIS Initialize (BIS_APPLICATION_HANDLE opaque, EFI_BIS_VERSION Major/Minor, BIS_VERSION_1 = 1, EFI_BIS_DATA {Length, Data}, codes), 24.5.3 Shutdown, 24.5.4 Free, 24.5.5 GetBootObjectAuthorizationCertificate, 24.5.6 GetBootObjectAuthorizationCheckFlag, 24.5.7 GetBootObjectAuthorizationUpdateToken (anti-rejeu), 24.5.8 GetSignatureInfo (EFI_BIS_SIGNATURE_INFO {CertificateID, AlgorithmID, KeyLength}) (début)
- [AGENT] Projet : BIS/vérification de signature = hors périmètre ; lecture seule d'infos OK.

## Statut
- Lignes 1 à 48015 lues. Prochaine : ligne 48016 (suite GetSignatureInfo, UpdateBootObjectAuthorization, VerifyBootObject, VerifyObjectWithCredential, types BIS_CERT_ID / BIS_ALG_ID, 24.6 HTTP Boot).

## Lignes 48016-48315 — 24.5.8 GetSignatureInfo (fin : BIS_GET_SIGINFO_COUNT, BIS_CERT_ID = 4 premiers octets LE du SHA-1 du certificat, masque 0xFF7F7FFF, IDs réservés DSA/RSA_MD5 ; BIS_ALG_ID : DSA 41, RSA_MD5 42), 24.5.9 UpdateBootObjectAuthorization (RequestCredential = Signed Manifest ; codes incl. EFI_SECURITY_VIOLATION multiples), format manifeste (Manifest-Version 2.0, ManifestPersistentId, Name memory:UpdateRequestParameters, X-Intel-BIS-*, signer info, PKCS#7 .DSA/.RSA), 24.5.10 VerifyBootObject (début : Credentials optionnel si check flag désactivé ; DataObject en mémoire ; IsVerified) — manifeste memory:BootObject
- [AGENT] Projet : vérification/configuration BIS hors périmètre ; aucune écriture de configuration d'autorisation.

## Statut
- Lignes 1 à 48315 lues. Prochaine : ligne 48316 (suite VerifyBootObject (fin), VerifyObjectWithCredential, 24.6 HTTP Boot...).

## Lignes 48316-48615 — 24.5.10 VerifyBootObject (fin : deux cas selon check flag ; security violation), 24.5.11 VerifyObjectWithCredential (SectionName, AuthorityCertificate optionnel, codes), 24.6 DHCP options iSCSI IPv6 (option 59 root path RFC 4173 ; option 60 serveur DHCP RFC 5970), 24.7 HTTP Boot (24.7.1 boot from URL : NBP ; types de tags 0x0F x86, 0x10 x64 ; 24.7.2 concept, usage entreprise) (début)
- [AGENT] Projet : réseau/HTTP Boot hors périmètre.

## Statut
- Lignes 1 à 48615 lues. Prochaine : ligne 48616 (suite HTTP Boot : 24.7.2.x, EFI_HTTP_PROTOCOL, EFI_HTTP_BOOT ...).

## Lignes 48616-48915 — 24.7.2-24.7.10 HTTP Boot (topologie, pile, device paths IPv4/IPv6 + Uri + RamDisk + Dns, priorités DHCP offers, messages DHCPv4/v6, proxy CONNECT), 24.7.11-13 HTTP Boot Callback (GUID BA23B311-343D-11E6-9185-5820B1D65299 ; types Dhcp4/Dhcp6/HttpRequest/HttpResponse/HttpEntityBody/HttpAuthInfo ; ≤ 5 ms ; EFI_SUCCESS continue / EFI_ABORTED), CHAPITRE 25 MANAGED NETWORK (25.1 MNSBP GUID F36FF770-A7E1-42CF-9ED2-56F0F271F44C ; 25.1.2 MNP GUID 7AB33A91-ACE5-4326-B572-E7EE33D39F16 : GetModeData, Configure, McastIpToMac, Groups, Transmit, Receive, Cancel, Poll) (début)
- [AGENT] Projet : réseau hors périmètre.

## Statut
- Lignes 1 à 48915 lues. Prochaine : ligne 48916 (suite MNP : Configure, Transmit, Receive, Poll ...).

## Lignes 48916-49215 — 25.1.2 fin (ordre : Configure, puis Transmit/Receive/Poll), 25.1.3 GetModeData (EFI_MANAGED_NETWORK_CONFIG_DATA : ReceivedQueueTimeoutValue, TransmitQueueTimeoutValue (défaut 10 s), ProtocolTypeFilter, EnableUnicast/Multicast/Broadcast/PromiscuousReceive, FlushQueuesOnReset, EnableReceiveTimestamps, DisableBackgroundPolling), 25.1.4 Configure (NULL = reset ; instance démarrée après Configure), 25.1.5 McastIpToMac, 25.1.6 Groups (JoinFlag, MacAddress ; NULL + FALSE = quitter tous), 25.1.7 Transmit (début)
- [AGENT] Projet : réseau hors périmètre.

## Statut
- Lignes 1 à 49215 lues. Prochaine : ligne 49216 (suite Transmit, Receive, Cancel, Poll, EFI_MANAGED_NETWORK_COMPLETION_TOKEN).

## Lignes 49216-49512 — 25.1.7 Transmit (fin : EFI_MANAGED_NETWORK_COMPLETION_TOKEN {Event EVT_NOTIFY_SIGNAL, Status, Packet (RxData|TxData)}, codes), EFI_MANAGED_NETWORK_RECEIVE_DATA (Timestamp, RecycleEvent, PacketLength, HeaderLength, AddressLength, DataLength, Broadcast/Multicast/Promiscuous flags, ProtocolType, DestinationAddress, SourceAddress, MediaHeader, PacketData), EFI_MANAGED_NETWORK_TRANSMIT_DATA (DestinationAddress, SourceAddress, ProtocolType, DataLength, HeaderLength, FragmentCount, FragmentTable[]), EFI_MANAGED_NETWORK_FRAGMENT_DATA {FragmentLength, FragmentBuffer}, 25.1.8 Receive, 25.1.9 Cancel (début)
- Règle générique utile : RecycleEvent doit être signalé après traitement pour rendre le stockage de la file ; ne plus accéder au paquet après.

## Statut
- Lignes 1 à 49512 lues. Prochaine : ligne 49513 (suite Cancel, Poll, fin MNP ; puis chapitre 26+).

## Lignes 49513-49812 — 25.1.9 Cancel (Token NULL = tout annuler ; EFI_NOT_FOUND si déjà terminé), 25.1.10 Poll (à appeler plus souvent si pertes ; EFI_NOT_READY / EFI_TIMEOUT), CHAPITRE 26 BLUETOOTH : 26.1 EFI Bluetooth HC Protocol (GUID B3930571-BEBA-4FC5-9203-9427242E6A43 ; SendCommand, ReceiveEvent, AsyncReceiveEvent, SendACLData, ReceiveACLData, AsyncReceiveACLData, SendSCOData, ReceiveSCOData, AsyncReceiveSCOData ; 26.1.2-26.1.4 commandes/événements HCI ; codes) (début)
- [AGENT] Projet : Bluetooth hors périmètre.

## Statut
- Lignes 1 à 49812 lues. Prochaine : ligne 49813 (suite ACL/SCO, puis chapitres Bluetooth/ suivants ; OK jusqu'à la fin du chapitre 26 puis appendices).

## Lignes 49813-50112 — 26.1.5-26.1.10 (ACL send/receive/async, SCO send/receive/async : codes, IsNewTransfer, PollingInterval ; SCO non supporté → EFI_UNSUPPORTED), 26.2 EFI Bluetooth Bus : 26.2.1 IO Service Binding (GUID 388278D3-7B85-42F0-ABA9-FB4BFD69F5AB ; CreateChild/DestroyChild), 26.2.2 EFI_BLUETOOTH_IO_PROTOCOL (GUID 467313DE-4E30-43F1-943E-323F8984 5DB5 ; GetDeviceInfo, GetSdpInfo, L2CapRawSend/RawReceive/RawAsyncReceive, L2CapSend/Receive/AsyncReceive, L2CapConnect/Disconnect/RegisterService) (début)
- [AGENT] Projet : Bluetooth hors périmètre.

## Statut
- Lignes 1 à 50112 lues. Prochaine : ligne 50113 (suite Bluetooth IO : fonctions L2CAP et codes, puis chapitres suivants).

## Lignes 50113-50412 — 26.2.2 fin (liste des fonctions L2CAP), 26.2.3 GetDeviceInfo (EFI_BLUETOOTH_DEVICE_INFO : Version, BD_ADDR[6], PageScanRepetitionMode, ClassOfDevice (bitfields FormatType:2, MinorDeviceClass:6, MajorDeviceClass:5, MajorServiceClass:11), ClockOffset, RSSI, ExtendedInquiryResponse[240]), 26.2.4 GetSdpInfo, 26.2.5 L2CapRawSend, 26.2.6 L2CapRawReceive, 26.2.7 L2CapRawAsyncReceive (début)
- [AGENT] Projet : Bluetooth hors périmètre.

## Statut
- Lignes 1 à 50412 lues. Prochaine : ligne 50413 (suite L2CapRawAsyncReceive, L2CapSend, L2CapReceive, L2CapAsyncReceive, L2CapConnect, L2CapDisconnect, L2CapRegisterService).

## Lignes 50413-50712 — 26.2.7 L2CapRawAsyncReceive (fin), 26.2.8 L2CapSend (Handle issu de L2CapConnect), 26.2.9 L2CapReceive, 26.2.10 L2CapAsyncReceive, 26.2.11 L2CapConnect (PSM, MTU, callback, blocage long), 26.2.12 L2CapDisconnect (début de la fiche)
- [AGENT] Projet : Bluetooth hors périmètre.

## Statut
- Lignes 1 à 50712 lues. Prochaine : ligne 50713 (suite L2CapDisconnect, L2CapRegisterService, puis chapitre 27+).

## Lignes 50713-51012 — 26.2.13 L2CapRegisterService (NULL = désenregistrer), 26.3 EFI Bluetooth Configuration (GUID 62960CF3-40FF-4263-A77C-DFDEBD191B4B ; Init, Scan, Connect, Disconnect, GetData, SetData, GetRemoteData, RegisterPinCallback, RegisterGetLinkKeyCallback, RegisterSetLinkKeyCallback, RegisterLinkConnectCompleteCallback), 26.3.2 Init, 26.3.3 Scan (ReScan, ScanType Inquiry/Page, callback ; NULL CallbackInfo = fin), 26.3.4 Connect (BD_ADDR ; crée EFI_BLUETOOTH_IO_PROTOCOL), 26.3.5 Disconnect (Reason) (début)
- [AGENT] Projet : Bluetooth hors périmètre.

## Statut
- Lignes 1 à 51012 lues. Prochaine : ligne 51013 (suite Disconnect, GetData, SetData, GetRemoteData, callbacks PIN/LinkKey, puis chapitres suivants).

## Lignes 51013-51312 — 26.3.5 Disconnect (fin : Reason), 26.3.6 GetData (EFI_BLUETOOTH_CONFIG_DATA_TYPE : DeviceName, ClassOfDevice, RemoteDeviceState, SdpInfo, BDADDR, Discoverable, ControllerStoredPairedDeviceList, AvailableDeviceList, RandomAddress, RSSI, AdvertisementData, IoCapability, OOBDataFlag, KeyType, EncKeySize ; états distants CONNECTED 0x1, PAIRED 0x2 ; LINK_KEY_SIZE 16), 26.3.7 SetData (WRITE_PROTECTED possible), 26.3.8 GetRemoteData, 26.3.9 RegisterPinCallback (types : UserPasskeyNotification, UserConfirmationRequest, OOBDataRequest, PinCodeRequest ; SSP et legacy obligatoires), 26.3.10 RegisterGetLinkKeyCallback (début)
- [AGENT] Projet : Bluetooth hors périmètre. Remarque : les clés de liaison (link keys) sont sensibles : ne jamais les journaliser.

## Statut
- Lignes 1 à 51312 lues. Prochaine : ligne 51313 (suite GetLinkKey/SetLinkKey/LinkConnectComplete callbacks, puis chapitres suivants).

## Lignes 51313-51612 — 26.3.10 fin (GetLinkKey : si non enregistré ou échec → HCI_Link_Key_Request_Negative_Reply), 26.3.11 SetLinkKey (Link_Key_Notification_Event ; stocker en mémoire non volatile sûre), 26.3.12 RegisterLinkConnectCompleteCallback (EFI_BLUETOOTH_CONNECT_COMPLETE_CALLBACK_TYPE : Disconnected, Connected, Authenticated, Encrypted), 26.4 EFI Bluetooth Attribute (GUID 898890E9-84B2-4F3A-8C58-D8577813E0AC ; SendRequest, RegisterForServerNotification, GetServiceInfo, GetDeviceInfo), 26.4.2 SendRequest (début)
- [AGENT] Projet : Bluetooth hors périmètre.

## Statut
- Lignes 1 à 51612 lues. Prochaine : ligne 51613 (suite SendRequest callback, RegisterForServerNotification, GetServiceInfo, GetDeviceInfo, puis chapitres suivants).

## Lignes 51613-51912 — 26.4.2 SendRequest (fin : codes ; Data = opcode ATT + champs ; callback NULL pour COMMAND), 26.4.3 RegisterForServerNotification (EFI_BLUETOOTH_ATTRIBUTE_CALLBACK_PARAMETER {Version, AttributeOpCode, Parameter {Notification|Indication} AttributeHandle}), 26.4.4 GetServiceInfo (EFI_BLUETOOTH_UUID {Length 2/4/16, Uuid16/32/128}, EFI_BLUETOOTH_ATTRIBUTE_HEADER {Type, Length, AttributeHandle, AttributePermission}, types GATT : Primary 0x2800, Secondary 0x2801, Include 0x2802, Characteristic 0x2803, Descriptor 0x2900 ; permissions bitfield : Readable, ReadEncryption/Authentication/Authorization, ReadKeySize:5, Writeable, Write*...), EFI_BLUETOOTH_GATT_PRIMARY_SERVICE_INFO (début)
- [AGENT] Projet : Bluetooth hors périmètre.

## Statut
- Lignes 1 à 51912 lues. Prochaine : ligne 51913 (suite GATT Primary/Secondary/Include/Characteristic info, GetDeviceInfo, puis chapitres suivants).

## Lignes 51913-52212 — 26.4.4 fin (GATT Include/Characteristic/Descriptor infos), 26.4.5 GetDeviceInfo (BLUETOOTH_LE_ADDRESS {Address[6], Type} ; EFI_BLUETOOTH_LE_DEVICE_INFO), 26.4.6 Attribute Service Binding (GUID 5639867A-8C8E-408D-AC2F-4B61BDC0BBBB), 26.5 EFI Bluetooth LE Configuration (GUID 8F76DA58-1F99-4275-A4EC-4756515B1CE8 ; Init, Scan, Connect, Disconnect, GetData, SetData, GetRemoteData, RegisterSmpAuthCallback, SendSmpAuthData, RegisterSmpGet/SetDataCallback, RegisterLinkConnectCompleteCallback), 26.5.2 Init, 26.5.3 Scan (EFI_BLUETOOTH_LE_CONFIG_SCAN_PARAMETER : ScanType, ScanInterval, ScanWindow, ScanningFilterPolicy, AdvertisementFlagFilter) (début)
- [AGENT] Projet : BLE hors périmètre.

## Statut
- Lignes 1 à 52212 lues. Prochaine : ligne 52213 (suite LE Scan callback, Connect, Disconnect, GetData, SetData, SMP callbacks, puis chapitres suivants).

## Lignes 52213-52512 — 26.5.3 fin (EFI_BLUETOOTH_LE_SCAN_CALLBACK_INFORMATION : BDAddr, DirectAddress, RemoteDeviceState, RSSI, AdvertisementData), 26.5.4 Connect (AutoReconnect, DoBonding, EFI_BLUETOOTH_LE_CONFIG_CONNECT_PARAMETER : ConnIntervalMin/Max, ConnLatency, SupervisionTimeout ; ALREADY_STARTED/NOT_FOUND), 26.5.5 Disconnect, 26.5.6 GetData, 26.5.7 SetData, 26.5.8 GetRemoteData (début)
- [AGENT] Projet : BLE hors périmètre.

## Statut
- Lignes 1 à 52512 lues. Prochaine : ligne 52513 (suite GetRemoteData, SMP auth/get/set callbacks, LE connect complete, puis chapitres suivants).

## Lignes 52513-52812 — 26.5.8 fin, 26.5.9 RegisterSmpAuthCallback (EFI_BLUETOOTH_LE_SMP_EVENT_DATA_TYPE : AuthorizationRequest, PasskeyReady, PasskeyRequest, OOBDataRequest, NumericComparison), 26.5.10 SendSmpAuthData, 26.5.11 RegisterSmpGetDataCallback (EFI_BLUETOOTH_LE_SMP_DATA_TYPE : clés locales IR/ER/DHK ; clés pair LTK/IRK/CSRK/Rand/EDIV/SignCounter/KeySize/KeyType ; PeerAddressList), 26.5.12 RegisterSmpSetDataCallback (début)
- [AGENT] Projet : BLE hors périmètre. Clés (IR, ER, LTK, IRK, CSRK) = secrets : ne jamais journaliser ni exposer.

## Statut
- Lignes 1 à 52812 lues. Prochaine : ligne 52813 (suite SetDataCallback, LE connect complete callback, puis chapitres suivants).

## Lignes 52813-53112 — 26.5.12 fin, 26.5.13 RegisterLinkConnectCompleteCallback (LE), fin du chapitre 26 ; CHAPITRE 27 NETWORK VLAN/EAP/WI-FI/SUPPLICANT : 27.1 VLAN Config (GUID 9E23D768-D2F3-4366-9FC3-3A7ABA864374 ; Set, Find, Remove ; VlanId 1-4094 ou 0 = non tagué ; Priority 0-7, défaut 0 ; EFI_VLAN_FIND_DATA {VlanId, Priority}), 27.2 EAP Protocol (début : GUID 5D9F96DB-E731-4CAA-A00D-72E187CD7762 ; SetDesiredAuthMethod, RegisterAuthMethod ; RFC 2284)
- [AGENT] Projet : réseau/VLAN/EAP/Wi-Fi hors périmètre. Note : le texte source contient une ligne de label RST « 2876 .. _network-protocols… » (artefact de mise en page, sans incidence).

## Statut
- Lignes 1 à 53112 lues. Prochaine : ligne 53113 (suite EAP RegisterAuthMethod, Wi-Fi, supplicant...).

## Lignes 53113-53412 — 27.2.2 SetDesiredAuthMethod (EAP_TYPE_TLS = 13, RFC 5216), 27.2.3 RegisterAuthMethod (EFI_EAP_BUILD_RESPONSE_PACKET), 27.2.4-27.2.5 EAPManagement (GUID BB62E663-625D-40B2-A088-BBE83623A245 ; GetSystemConfiguration, SetSystemConfiguration, InitializePort, UserLogon, UserLogoff, GetSupplicantStatus, SetSupplicantConfiguration, GetSupplicantStatistics ; IEEE 802.1X), 27.2.6 GetSystemConfiguration (PAE_SUPPORT_AUTHENTICATOR 0x1, SUPPLICANT 0x2 ; EFI_EAPOL_PORT_INFO), 27.2.7 SetSystemConfiguration, 27.2.8 InitializePort (début)
- [AGENT] Projet : réseau/EAP hors périmètre.

## Statut
- Lignes 1 à 53412 lues. Prochaine : ligne 53413 (suite InitializePort, UserLogon, UserLogoff, Supplicant status/config/stats, puis Wi-Fi/Supplicant).

## Lignes 53413-53712 — 27.2.8 InitializePort (fin), 27.2.9 UserLogon, 27.2.10 UserLogoff, 27.2.11 GetSupplicantStatus (états : Logoff, Disconnected, Connecting, Acquired, Authenticating, Held, Authenticated ; ValidFieldMask AUTH 0x1, HELD 0x2, START 0x4, MAX_START 0x8 ; défauts : authWhile 30 s, heldWhile 60 s, startWhen 30 s, MaxStart 3), 27.2.12 SetSupplicantConfiguration, 27.2.13 GetSupplicantStatistics (compteurs EAPOL), 27.2.14 EAP Management2 (GUID 5E93C847-456D-40B3-A6B4-78B0C9CF7F20 ; GetKey), 27.2.15 GetKey (MSK/EMSK pour dériver PMK), 27.2.16 EAP Configuration (GUID E5B58DBB-7688-44B4-97BF-5F1D4B7CC8DB ; SetData, GetData ; certificats, clés privées) (début)
- [AGENT] Projet : EAP/802.1X/Wi-Fi hors périmètre. Les clés MSK/EMSK sont sensibles : ne pas journaliser.

## Statut
- Lignes 1 à 53712 lues. Prochaine : ligne 53713 (suite EAP Configuration SetData/GetData, puis Wi-Fi/Supplicant/Chapitres suivants).

## Lignes 53713-54012 — 27.2.17 EAP Configuration SetData (EFI_EAP_TYPE : Attribute 0, Identity 1, Notification 2, NAK 3, MD5Challenge 4, OTP 5, GTC 6, EAPTLS 13, EAPSIM 18, TTLS 21, PEAP 25, MSCHAPV2 26, EAP_EXTENSION 33 ; EFI_EAP_CONFIG_DATA_TYPE : CA cert, client cert, clé privée (+ mot de passe), cipher suites, MSCHAPv2 password, 2nd auth method), 27.2.18 GetData (DataSize entrée/sortie), 27.3 EFI Wireless MAC Connection (GUID 0DA55BC9-45F8-4BB4-8719-5224F18A4D45 ; Scan, Associate, Disassociate, Authenticate, Deauthenticate), 27.3.2 Scan (EFI_80211_SCAN_DATA_TOKEN {Event, Status, Data, ResultCode, Result} ; EFI_80211_SCAN_DATA : BSSType, BSSId, SSIdLen, SSId, PassiveMode, ProbeDelay, ChannelList, Min/MaxChannelTime, RequestInformation, SSIDList, AccessNetworkType, VendorSpecificInfo) (début)
- [AGENT] Projet : Wi-Fi/EAP hors périmètre. Mots de passe / clés privées = secrets.

## Statut
- Lignes 1 à 54012 lues. Prochaine : ligne 54013 (suite EFI_80211 scan types, Associate, Disassociate, Authenticate, Deauthenticate, puis chapitre 28+).

## Lignes 54013-54312 — 27.3.2 Scan (fin : EFI_80211_ACC_NET_TYPE 0..5/14/15 ; EFI_80211_BSS_TYPE Infra/IBSS/MBSS/Any ; ELEMENT_REQ/SSID/HEADER ; EFI_80211_SCAN_RESULT_CODE ; EFI_80211_SCAN_RESULT ; EFI_80211_BSS_DESCRIPTION : BSSId, SSId, BeaconPeriod, Timestamp, CapabilityInfo, rates, Country, RSN, RSSI, RCPI, RSNI, membership, ExtCap ; EFI_80211_ELEMENT_COUNTRY / COUNTRY_TRIPLET (Subband {FirstChannelNum, NumOfChannels, MaxTxPowerLevel} / Operating)) (début)
- [AGENT] Projet : Wi-Fi hors périmètre.

## Statut
- Lignes 1 à 54312 lues. Prochaine : ligne 54313 (suite country triplet operating, BSS_DESP_PILOT, EXT_CAP, Associate/Disassociate/Authenticate/Deauthenticate).

## Lignes 54313-54612 — 27.3.2 fin (codes), détails Wi-Fi (COUNTRY_TRIPLET_OPERATE, ELEMENT_RSN / ELEMENT_DATA_RSN, ELEMENT_EXT_CAP, BSS_DESP_PILOT, MULTIPLE_BSSID, SUBELEMENT_INFO), 27.3.3 Associate (EFI_80211_ASSOCIATE_DATA_TOKEN {Event, Status, Data, ResultCode, Result}) (début)
- [AGENT] Projet : Wi-Fi hors périmètre.

## Statut
- Lignes 1 à 54612 lues. Prochaine : ligne 54613 (suite ASSOCIATE_DATA_TOKEN, Associate data/result, Disassociate, Authenticate, Deauthenticate).

## Lignes 54613-54912 — 27.3.3 Associate (fin : EFI_80211_ASSOCIATE_DATA {BSSId, CapabilityInfo, FailureTimeout, ListenInterval, Channels, RSN, ExtCap, VendorSpecificInfo}, EFI_80211_ELEMENT_SUPP_CHANNEL(_TUPLE), codes AssociateSuccess / RefusedReasonUnspecified / CapsMismatch / ExtReason / APOutOfMemory / BasicRatesMismatch / EmergencyServicesNotSupported / RefusedTemporarily, EFI_80211_ASSOCIATE_RESULT {BSSId, CapabilityInfo, AssociationID, RCPI/RSNI, ExtCap, TimeoutInterval, VendorSpecificInfo}, ELEMENT_TIMEOUT_VAL, codes), 27.3.4 Disassociate (EFI_80211_DISASSOCIATE_DATA_TOKEN ; EFI_ACCESS_DENIED, EFI_NOT_READY) (début)
- [AGENT] Projet : Wi-Fi hors périmètre.

## Statut
- Lignes 1 à 54912 lues. Prochaine : ligne 54913 (suite Disassociate data, Authenticate, Deauthenticate ; chapitre 28+).

## Lignes 54913-55212 — 27.3.4 Disassociate (fin : EFI_80211_DISASSOCIATE_DATA {BSSId, ReasonCode (EFI_80211_REASON_CODE 1..8), VendorSpecificInfo}, DisassociateSuccess / InvalidParameters, codes), 27.3.5 Authenticate (EFI_80211_AUTHENTICATE_DATA_TOKEN ; EFI_80211_AUTHENTICATION_TYPE : OpenSystem, SharedKey (WEP, obsolète), FastBSSTransition, SAE ; result codes ; EFI_PROTOCOL_ERROR / EFI_NO_RESPONSE), 27.3.6 Deauthenticate (début)
- [AGENT] Projet : Wi-Fi hors périmètre. SharedKey/WEP = cryptographie faible, à ne pas utiliser.

## Statut
- Lignes 1 à 55212 lues. Prochaine : ligne 55213 (suite Deauthenticate data/codes, puis chapitre 28+).

## Lignes 55213-55512 — 27.3.6 Deauthenticate (fin : EFI_80211_DEAUTHENTICATE_DATA ; codes), 27.4 EFI Wireless MAC Connection II (GUID 1B0FB9BF-699D-4FDD-A7C3-2546681BF63B ; GetNetworks, ConnectNetwork, DisconnectNetwork), 27.4.2 GetNetworks (EFI_80211_GET_NETWORKS_TOKEN ; SSID cachés ; EFI_MAX_SSID_LEN 32 ; EFI_80211_NETWORK_DESCRIPTION {Network, NetworkQuality 0..100} ; EFI_80211_NETWORK {BSSType, SSId, AKMSuite, CipherSuite}; EFI_80211_SUITE_SELECTOR {Oui[3] = 00-0F-AC, SuiteType}) (début)
- [AGENT] Projet : Wi-Fi hors périmètre.

## Statut
- Lignes 1 à 55512 lues. Prochaine : ligne 55513 (suite SUITE_SELECTOR, ConnectNetwork, DisconnectNetwork, puis chapitres suivants).

## Lignes 55513-55812 — 27.4.2 fin (AKM_SUITE_SELECTOR, CIPHER_SUITE_SELECTOR, codes), 27.4.3 ConnectNetwork (EFI_80211_CONNECT_NETWORK_TOKEN/DATA {Network, FailureTimeout} ; ConnectSuccess / Refused / Failed / FailureTimeout / FailedReasonUnspecified ; recommandé ≥ 5 s ; état via EFI_ADAPTER_INFORMATION_PROTOCOL : EFI_SUCCESS connecté / EFI_NO_MEDIA déconnecté), 27.4.4 DisconnectNetwork (EFI_80211_DISCONNECT_NETWORK_TOKEN), 27.5 EFI Supplicant (27.5.1-27.5.2 Service Binding GUID 45BCD98E-59AD-4174-9546-344A07485898 ; 27.5.4 EFI_SUPPLICANT_PROTOCOL GUID 54FCC43E-AA89-4333-9A85-CDEA24051E9E : BuildResponsePacket, ProcessPacket, SetData, GetData ; PSK et EAP 802.1X ; 27.5.5 BuildResponsePacket (début))
- [AGENT] Projet : Wi-Fi/supplicant hors périmètre.

## Statut
- Lignes 1 à 55812 lues. Prochaine : ligne 55813 (suite BuildResponsePacket, ProcessPacket, SetData, GetData du supplicant ; puis chapitres suivants).

## Lignes 55813-56112 — 27.5.5 BuildResponsePacket (fin : NULL = démarrer EAPOL-Start ; codes), 27.5.6 ProcessPacket (EFI_SUPPLICANT_FRAGMENT_DATA, EFI_SUPPLICANT_CRYPT_MODE Encrypt/Decrypt ; EFI_UNSUPPORTED si pas de chiffrement logiciel), 27.5.7 SetData (EFI_SUPPLICANT_DATA_TYPE : AKMSuite, GroupDataCipherSuite, PairwiseCipherSuite, PskPassword (ASCII), TargetSSIDName, StationMac, TargetSSIDMac, PTK, GTK, State, LinkState, KeyRefresh, Supported*, IGTK, PMK ; EFI_80211_LINK_STATE : UnauthenticatedUnassociated, AuthenticatedUnassociated, PendingRSNAuthentication, AuthenticatedAssociated ; EFI_SUPPLICANT_GTK_LIST, EFI_SUPPLICANT_KEY {Key[64], KeyLen, KeyId, KeyType, Addr, Rsc[8], RscLen, IsAuthenticator, CipherSuite, Direction}) (début)
- [AGENT] Projet : supplicant/Wi-Fi hors périmètre. Clés PTK/GTK/PMK/PSK = secrets à ne jamais journaliser.

## Statut
- Lignes 1 à 56112 lues. Prochaine : ligne 56113 (suite EFI_SUPPLICANT_KEY_TYPE / DIRECTION, GetData, puis chapitres suivants).

## Lignes 56113-56412 — 27.5.7 fin (EFI_SUPPLICANT_KEY_TYPE : Group, Pairwise, PeerKey, IGTK ; EFI_SUPPLICANT_KEY_DIRECTION : Receive, Transmit, Both), 27.5.8 GetData (DataSize entrée/sortie ; EFI_NOT_FOUND ; BUFFER_TOO_SMALL), CHAPITRE 28 TCP/IP/IPSEC/FTP/TLS : 28.1 EFI TCPv4 (service binding GUID 00720665-67EB-4A99-BAF7-D3C33A1C7CC9 ; protocole GUID 65530BC7-A359-410F-B010-5AADC7EC2B62 ; GetModeData, Configure, Routes, Connect, Accept, Transmit, Receive, Close, Cancel, Poll ; EFI_TCP4_ACCESS_POINT {UseDefaultAddress, StationAddress, SubnetMask, StationPort, RemoteAddress, RemotePort, ActiveFlag}) (début)
- [AGENT] Projet : TCP/IP hors périmètre. Règle : adresses IPv4 et paquets en ordre réseau (big-endian), autres paramètres en ordre hôte.

## Statut
- Lignes 1 à 56412 lues. Prochaine : ligne 56413 (suite EFI_TCP4_CONFIG_DATA, ConnectionState, Configure, Routes, Connect, Accept, Transmit, Receive, Close, Cancel, Poll).

## Lignes 56413-56712 — 28.1.5 fin (EFI_TCP4_OPTION : ReceiveBufferSize, SendBufferSize, MaxSynBackLog, ConnectionTimeout, DataRetries, FinTimeout, TimeWaitTimeout, KeepAliveProbes/Time/Interval, EnableNagle/TimeStamp/WindowScaling/SelectiveAck/PathMtuDiscovery (RFC 896, 7323, 2018, 1191)), EFI_TCP4_CONFIG_DATA (TypeOfService, TimeToLive, AccessPoint, ControlOption), EFI_TCP4_CONNECTION_STATE (Closed 0, Listen 1, SynSent 2, SynReceived 3, Established 4, FinWait1 5, FinWait2 6, Closing 7, TimeWait 8, CloseWait 9, LastAck 10), 28.1.6 Configure (NULL = reset brutal ; codes), 28.1.7 Routes (SubnetAddress/SubnetMask/GatewayAddress ; route par défaut 0.0.0.0/0.0.0.0 ; direct si gateway 0.0.0.0), 28.1.8 Connect (EFI_TCP4_COMPLETION_TOKEN {Event, Status} ; codes EFI_CONNECTION_RESET/REFUSED/ABORTED/TIMEOUT/NETWORK_UNREACHABLE/HOST_UNREACHABLE) (début)
- [AGENT] Projet : TCP/IP hors périmètre.

## Statut
- Lignes 1 à 56712 lues. Prochaine : ligne 56713 (suite Connect, Accept, Transmit, Receive, Close, Cancel, Poll TCP4).

## Lignes 56713-57012 — 28.1.8 fin (codes Connect), 28.1.9 Accept (EFI_TCP4_LISTEN_TOKEN {CompletionToken, NewChildHandle} ; hérite de la config ; état Established ; codes), 28.1.10 Transmit (EFI_TCP4_IO_TOKEN {CompletionToken, Packet RxData|TxData} ; codes : EFI_CONNECTION_FIN 104, EFI_CONNECTION_RESET 105, EFI_CONNECTION_REFUSED 106 (EFIERR(n)) ; EFI_TCP4_RECEIVE_DATA {UrgentFlag, DataLength, FragmentCount, FragmentTable} ; EFI_TCP4_FRAGMENT_DATA {FragmentLength, FragmentBuffer} ; EFI_TCP4_TRANSMIT_DATA {Push, Urgent, DataLength, FragmentCount, FragmentTable}) (début)
- [AGENT] Projet : TCP/IP hors périmètre.

## Statut
- Lignes 1 à 57012 lues. Prochaine : ligne 57013 (suite TRANSMIT_DATA, Receive, Close, Cancel, Poll TCP4 ; puis IP4/IP6/UDP/ETC).

## Lignes 57013-57312 — 28.1.10 fin (Transmit : codes), 28.1.10.1 Receive (DataLength = taille totale des fragments ; codes ; EFI_CONNECTION_FIN), 28.1.11 Close (EFI_TCP4_CLOSE_TOKEN {CompletionToken, AbortOnClose} : FIN gracieux RFC 793 ou RST rapide ; codes ; EFI_ABORTED si Configure NULL), 28.1.12 Cancel (Token NULL = tout annuler ; EFI_NOT_FOUND), 28.1.13 Poll (appel optionnel, fréquent si pertes), 28.2 EFI TCPv6 (service binding GUID EC20EB79-6C1A-4664-9A0D-D2E4CC16D664 ; protocole GUID 46E44855-BD60-4AB7-AB0D-A679B9447D77 ; mêmes méthodes qu'en TCPv4 ; 28.2.5 GetModeData) (début)
- [AGENT] Projet : TCP/IP hors périmètre.

## Statut
- Lignes 1 à 57312 lues. Prochaine : ligne 57313 (suite TCP6 GetModeData, Configure, Connect, Accept, Transmit, Receive, Close, Cancel, Poll ; puis IP4/IP6/UDP/...).

## Lignes 57313-57612 — 28.2.5 fin (EFI_TCP6_ACCESS_POINT ; StationAddress 0 = choix par IPv6), EFI_TCP6_OPTION (mêmes champs que TCP4), EFI_TCP6_CONFIG_DATA (TrafficClass, HopLimit, AccessPoint, ControlOption), EFI_TCP6_CONNECTION_STATE (0..10 identiques à TCP4), 28.2.6 Configure (codes), 28.2.7 Connect (EFI_TCP6_COMPLETION_TOKEN ; codes incl. PROTOCOL_UNREACHABLE, PORT_UNREACHABLE) (début)
- [AGENT] Projet : TCP/IPv6 hors périmètre.

## Statut
- Lignes 1 à 57612 lues. Prochaine : ligne 57613 (suite Connect TCP6 codes, Accept, Transmit, Receive, Close, Cancel, Poll TCP6 ; IP4/IP6 ...).

## Lignes 57613-57912 — 28.2.7 fin (codes TCP6 Connect, EFI_SECURITY_VIOLATION = politique IPSec), 28.2.8 Accept (EFI_TCP6_LISTEN_TOKEN ; codes), 28.2.9 Transmit (EFI_TCP6_IO_TOKEN ; EFI_TCP6_RECEIVE_DATA / TRANSMIT_DATA / FRAGMENT_DATA identiques à TCP4 ; codes), 28.2.10 Receive (début, mêmes règles que TCP4)
- [AGENT] Projet : TCP/IPv6 hors périmètre.

## Statut
- Lignes 1 à 57912 lues. Prochaine : ligne 57913 (suite TCP6 Receive codes, Close, Cancel, Poll ; puis IP4/IP6/UDP/IPSEC/FTP/TLS/config).

## Lignes 57913-58212 — 28.2.12 Cancel (Token NULL = annule tous les tokens Connect/Accept/Transmit/Receive ; codes EFI_NOT_FOUND, EFI_UNSUPPORTED), 28.2.13 Poll (optionnel, à appeler plus souvent si pertes de paquets ; codes EFI_NOT_READY, EFI_TIMEOUT), 28.3 EFI IPv4 Protocol (présentation : RFC 1122, 4632, 3021 ; adresses en network byte order, autres paramètres en host byte order), 28.3.2 IP4 Service Binding, 28.3.4 EFI_IP4_PROTOCOL (GetModeData, Configure, Groups, Routes, Transmit, Receive, Cancel, Poll), 28.3.5 GetModeData (EFI_IP4_MODE_DATA)
- [AGENT] Projet : réseau hors périmètre. Règle générale notée : adresses réseau en network byte order.

## Statut
- Lignes 1 à 58212 lues. Prochaine : ligne 58213 (suite EFI_IP4_MODE_DATA).

## Lignes 58213-58512 — EFI_IP4_MODE_DATA (suite : IsStarted, MaxPacketSize, IsConfigured, GroupTable, RouteTable, IcmpTypeList), EFI_IP4_CONFIG_DATA (DefaultProtocol, AcceptAnyProtocol/IcmpErrors/Broadcast/Promiscuous, UseDefaultAddress, StationAddress 0.0.0.0 = toutes adresses, SubnetMask, TOS, TTL, DF, RawData, ReceiveTimeout/TransmitTimeout en microsecondes), EFI_IP4_ROUTE_TABLE (masque : Dest & Mask == Subnet -> passerelle ; Gateway 0.0.0.0 = pas de redirection), EFI_IP4_ICMP_TYPE (RFC 792/950), 28.3.6 Configure (codes ; changer adresse/masque/RawData exige Stop ; réinit = cancel des requêtes en attente EFI_ABORTED), 28.3.7 Groups (JoinFlag ; FALSE + NULL = quitter tous), 28.3.8 Routes (début : DeleteRoute, clé = SubnetAddress+SubnetMask)
- [AGENT] Projet : réseau hors périmètre.

## Statut
- Lignes 1 à 58512 lues. Prochaine : ligne 58513 (suite 28.3.8 Routes : GatewayAddress, codes ; 28.3.9 Transmit...).

## Lignes 58513-58812 — 28.3.8 Routes (fin : routes par ET masque ; route défaut 0.0.0.0/0.0.0.0 ; Gateway 0.0.0.0 = nonroute, ARP ; passerelle sur le même sous-réseau ; routes non partagées entre instances), 28.3.9 Transmit (EFI_IP4_COMPLETION_TOKEN : Event NOTIFY_SIGNAL TPL <= CALLBACK ; Status SUCCESS/ABORTED/TIMEOUT/ICMP_ERROR/DEVICE_ERROR/NO_MEDIA ; union RxData/TxData), EFI_IP4_RECEIVE_DATA (TimeStamp, RecycleSignal : après signal, ne plus référencer ; HeaderLength, Header, Options, DataLength, FragmentTable non contigu), EFI_IP4_HEADER (pack(1), champs RFC 791), EFI_IP4_FRAGMENT_DATA (longueur non nulle, buffer non NULL), EFI_IP4_TRANSMIT_DATA (DestinationAddress, OverrideData, options, TotalDataLength, FragmentTable)
- [AGENT] Projet : réseau hors périmètre. Règle générale : un token doit rester valide jusqu'au signal, et RecycleSignal libère la structure.

## Statut
- Lignes 1 à 58812 lues. Prochaine : ligne 58813 (suite EFI_IP4_TRANSMIT_DATA, EFI_IP4_OVERRIDE_DATA, Transmit codes, 28.3.10 Receive...).

## Lignes 58813-59112 — EFI_IP4_OVERRIDE_DATA (SourceAddress, GatewayAddress, Protocol, TOS, TTL, DF ; ne vaut que pour un Transmit), codes Transmit (liste de contrôles : token/Event/TxData/Fragment non NULL, longueurs non nulles, TotalDataLength = somme des fragments, en-tête bien formé si RawData ; BUFFER_TOO_SMALL si > MTU, ou > taille max si DF), 28.3.10 Receive (Event obligatoire ; cache du token ; codes), 28.3.11 Cancel (Token NULL = tout annuler ; NOT_FOUND si déjà terminé, pas de signal), 28.3.12 Poll, 28.4 EFI IPv4 Configuration II Protocol (SetData, GetData, RegisterDataNotify, UnregisterDataNotify), 28.4.2 SetData (non bloquant, EFI_NOT_READY pour un traitement asynchrone, résultat via event + GetData ; types : InterfaceInfo lecture seule, Policy, ManualAddress (seulement si policy Static), Gateway, DnsServer ; données persistantes après reset), EFI_IP4_CONFIG2_INTERFACE_INFO (Name 32 CHAR16, IfType, HwAddress, StationAddress, SubnetMask, RouteTable)
- [AGENT] Projet : réseau hors périmètre. Règle notable : un Set asynchrone doit être suivi d'un GetData après signal.

## Statut
- Lignes 1 à 59112 lues. Prochaine : ligne 59113 (suite EFI_IP4_CONFIG2_INTERFACE_INFO, EFI_IP4_CONFIG2_POLICY, GetData, RegisterDataNotify).

## Lignes 59113-59412 — EFI_IP4_CONFIG2_INTERFACE_INFO (fin : lecture seule, buffer appelant, ne pas libérer RouteTable séparément), EFI_IP4_CONFIG2_POLICY (Static : adresse/passerelle/DNS manuels ; Dhcp : interdit via SetData ; changement Static->Dhcp efface la config manuelle), EFI_IP4_CONFIG2_MANUAL_ADDRESS, codes SetData, 28.4.3 GetData (DataSize en entrée/sortie, EFI_BUFFER_TOO_SMALL donne la taille, EFI_NOT_READY si traitement asynchrone), 28.4.4 RegisterDataNotify, 28.4.5 UnregisterDataNotify, 28.5 EFI IPv6 (ND, MLD, ICMPv6), 28.5.2 IP6 Service Binding, 28.5.4 EFI_IP6_PROTOCOL (GetModeData, Configure, Groups, Routes, Neighbors, Transmit, Receive, Cancel, Poll)
- [AGENT] Projet : réseau hors périmètre.

## Statut
- Lignes 1 à 59412 lues. Prochaine : ligne 59413 (suite EFI_IP6_PROTOCOL : Transmit/Receive/Cancel/Poll et définitions IPv6).

## Lignes 59413-59712 — 28.5.4 fin (Receive/Cancel/Poll), 28.5.5 GetModeData, EFI_IP6_MODE_DATA (AddressList, GroupTable, RouteTable, NeighborCache, PrefixTable, IcmpTypeList : buffers à libérer par l'appelant), EFI_IP6_CONFIG_DATA (DefaultProtocol : valeurs interdites 0,1,2,41,43,44,59,60,124 ; DestinationAddress ; StationAddress 0 = binding auto, sinon doit être une adresse configurée ; TrafficClass, HopLimit, FlowLabel, timeouts en µs), EFI_IP6_ADDRESS_INFO, EFI_IP6_ROUTE_TABLE (Gateway 0 = on-link), EFI_IP6_NEIGHBOR_CACHE, EFI_IP6_NEIGHBOR_STATE (InComplete, Reachable, Stale, Delay, Probe), EFI_IP6_ICMP_TYPE
- [AGENT] Projet : réseau hors périmètre. Règle : adresses en network byte order, autres champs en host byte order (même convention que IPv4).

## Statut
- Lignes 1 à 59712 lues. Prochaine : ligne 59713 (EFI_IP6_ICMP_TYPE suite, 28.5.6 Configure IPv6).

## Lignes 59713-60012 — Définitions ICMPv6 : erreurs (DEST_UNREACHABLE 0x1, PACKET_TOO_BIG 0x2, TIME_EXCEEDED 0x3, PARAMETER_PROBLEM 0x4), informatifs (ECHO 0x80/0x81, LISTENER 0x82/0x83/0x84, ROUTER_SOLICIT 0x85, ROUTER_ADVERTISE 0x86, NEIGHBOR_SOLICIT 0x87, NEIGHBOR_ADVERTISE 0x88, REDIRECT 0x89, LISTENER_REPORT_2 0x8F), codes de DEST_UNREACHABLE (0 no route .. 6 route rejected), TIME_EXCEEDED (0 hop limit, 1 reassemble), PARAMETER_PROBLEM (0 erroné, 1 next header inconnu, 2 option inconnue) ; 28.5.6 Configure IPv6 (StationAddress 0 = binding auto selon RFC ; non nul doit être une adresse configurée ; DefaultProtocol illégal ; réinit = EFI_ABORTED), 28.5.7 Groups (SSM non requis), 28.5.8 Routes (Destination + PrefixLength ; défaut = tous zéros ; table partagée entre instances ; Gateway ne peut être une adresse locale), 28.5.9 Neighbors (Timeout en unités 100 ns, 0 = permanent ; Override ; comportement RFC 4861), 28.5.10 Transmit (début)
- [AGENT] Projet : réseau hors périmètre.

## Statut
- Lignes 1 à 60012 lues. Prochaine : ligne 60013 (suite 28.5.10 Transmit : EFI_IP6_COMPLETION_TOKEN, structures IPv6).

## Lignes 60013-60312 — 28.5.10 Transmit IPv6 : EFI_IP6_COMPLETION_TOKEN (Event NOTIFY_SIGNAL ; Status SUCCESS/ABORTED/TIMEOUT/ICMP_ERROR/DEVICE_ERROR/SECURITY_VIOLATION (IPsec) ; union RxData/TxData), EFI_IP6_RECEIVE_DATA (TimeStamp, RecycleSignal, HeaderLength incluant extensions, Header, DataLength, FragmentTable), EFI_IP6_HEADER (pack(1) : TrafficClass et FlowLabel éclatés sur bits, PayloadLength, NextHeader, HopLimit, Src/Dst), EFI_IP6_FRAGMENT_DATA, EFI_IP6_TRANSMIT_DATA (DestinationAddress 0 = ConfigData, OverrideData, ExtHdrs/ExtHdrsLength/NextHeader, DataLength, FragmentTable), EFI_IP6_OVERRIDE_DATA (Protocol, HopLimit, FlowLabel), codes Transmit (contrôles : ExtHdrs non NULL si longueur non nulle, longueurs de fragments non nulles, DataLength = somme, DestinationAddress cohérente avec Configure)
- [AGENT] Projet : réseau hors périmètre.

## Statut
- Lignes 1 à 60312 lues. Prochaine : ligne 60313 (suite codes Transmit IPv6, 28.5.11 Receive...).

## Lignes 60313-60612 — codes Transmit IPv6 (fin : NOT_READY, BUFFER_TOO_SMALL si DataLength > max décrit par Fragment Offset), 28.5.11 Receive IPv6 (codes), 28.5.12 Cancel IPv6 (EFI_NOT_FOUND si déjà terminé), 28.5.13 Poll IPv6, 28.6 EFI IPv6 Configuration Protocol (SetData, GetData, RegisterDataNotify, UnregisterDataNotify), 28.6.2 SetData (non bloquant ; DAD ; EFI_NOT_READY + event + GetData), types Ip6ConfigDataType (InterfaceInfo lecture seule, AltInterfaceId 64 bits si l'ID link-local est dupliqué, Policy, DupAddrDetectTransmits (0 = pas de DAD), ManualAddress, Gateway, DnsServer), EFI_IP6_CONFIG_INTERFACE_INFO (Name, IfType RFC 3232, HwAddress, AddressInfo, RouteTable : buffer unique fourni par appelant), EFI_IP6_CONFIG_INTERFACE_ID (8 octets), EFI_IP6_CONFIG_POLICY (Manual / Automatic : SLAAC et/ou DHCPv6 stateful ; changement Manual -> Automatic efface la config manuelle)
- [AGENT] Projet : réseau hors périmètre.

## Statut
- Lignes 1 à 60612 lues. Prochaine : ligne 60613 (suite EFI_IP6_CONFIG_DUP_ADDR_DETECT_TRANSMITS, GetData, RegisterDataNotify...).

## Lignes 60613-60912 — EFI_IP6_CONFIG_DUP_ADDR_DETECT_TRANSMITS (défaut 1 ; 0 = pas de DAD), EFI_IP6_CONFIG_MANUAL_ADDRESS (Address, IsAnycast, PrefixLength), codes SetData IPv6, 28.6.3 GetData IPv6 (DataSize entrée/sortie, BUFFER_TOO_SMALL), 28.6.4 RegisterDataNotify IPv6, 28.6.5 UnregisterDataNotify IPv6, 28.7 IPsec (AH RFC 4302, ESP RFC 4303, architecture RFC 4301 ; IPv6 exige IPsec), 28.7.2-28.7.3 EFI_IPSEC_CONFIG_PROTOCOL (SetData, GetData, GetNextSelector, RegisterDataNotify, UnregisterDataNotify), 28.7.4 SetData IPsec (InsertBefore, Selector NULL = flush, Data NULL = suppression d'une entrée), types EFI_IPSEC_CONFIG_DATA_TYPE (SPD, SAD, PAD), EFI_IPSEC_CONFIG_SELECTOR (union SPD/SA/PAD)
- [AGENT] Projet : IPsec et réseau hors périmètre.

## Statut
- Lignes 1 à 60912 lues. Prochaine : ligne 60913 (suite EFI_IPSEC_CONFIG_SELECTOR, SetData IPsec suite...).

## Lignes 60913-61212 — EFI_IPSEC_SPD_SELECTOR (LocalAddress/RemoteAddress listes, NextLayerProtocol 0 = joker, ports et plages ; pour ICMP, Local/RemotePort = type/code), EFI_IP_ADDRESS_INFO, MAX_PEERID_LEN 128, EFI_IPSEC_SPD_DATA (Name, PackageFlag bits 0-4, TrafficDirection, Action, ProcessingPolicy, SaId list), EFI_IPSEC_TRAFFIC_DIR (InBound/OutBound), EFI_IPSEC_ACTION (Discard / Bypass / Protect), EFI_IPSEC_PROCESS_POLICY (ExtSeqNum, SeqOverflow, FragCheck, SaLifetime, Mode, TunnelOption, Proto AH/ESP, AuthAlgoId, EncAlgoId), EFI_IPSEC_SA_LIFETIME (ByteCount, SoftLifetime, HardLifetime en secondes ; 0 = illimité), EFI_IPSEC_MODE (Transport / Tunnel), EFI_IPSEC_TUNNEL_DF_OPTION (ClearDf / SetDf / CopyDf)
- [AGENT] Projet : IPsec hors périmètre.

## Statut
- Lignes 1 à 61212 lues. Prochaine : ligne 61213 (suite EFI_IPSEC_TUNNEL_OPTION, EFI_IPSEC_SA_ID, EFI_IPSEC_SA_DATA...).

## Lignes 61213-61512 — EFI_IPSEC_TUNNEL_OPTION (LocalTunnelAddress, RemoteTunnelAddress, DF), EFI_IPSEC_PROTOCOL_TYPE (AH RFC 4302 / ESP RFC 4303), EFI_IPSEC_SA_ID (Spi, Proto, DestAddress : triplet d'identification), EFI_IPSEC_SA_DATA (Mode, SNCount 64 bits, AntiReplayWindows, AlgoInfo, SaLifetime, PathMTU RFC 4301 §8, SpdSelector, ManualSet), EFI_IPSEC_SA_DATA2 (ajoute TunnelSource/DestinationAddress : à préférer si tunnel), EFI_IPSEC_ALGO_INFO (union AH/ESP), EFI_IPSEC_AH_ALGO_INFO (RFC 4305), EFI_IPSEC_ESP_ALGO_INFO (enc + auth ; combined mode RFC 4309 type AES-CCM), EFI_IPSEC_PAD_ID (PeerIdValid, union IpAddress / PeerId 128 octets : DNS, DN, email RFC 822, Key ID)
- [AGENT] Projet : IPsec hors périmètre. Règle notable : les clés (Key, EncKey, AuthKey) sont des secrets ; ne jamais les journaliser.

## Statut
- Lignes 1 à 61512 lues. Prochaine : ligne 61513 (suite EFI_IPSEC_PAD_DATA...).

## Lignes 61513-61812 — EFI_IPSEC_PAD_DATA (AuthProtocol IKEv1/IKEv2, AuthMethod PreSharedSecret/Certificates, IkeIdFlag, AuthData = secret ou trust anchor, RevocationData CRL/OCSP), codes SetData IPsec, 28.7.5 GetData IPsec (BUFFER_TOO_SMALL met à jour DataSize), 28.7.6 GetNextSelector (énumération ; buffer initial à zéro ; SetData entre deux appels = résultat imprévisible ; NOT_FOUND en fin), 28.7.7 RegisterDataNotify IPsec, 28.7.8 UnregisterDataNotify IPsec, 28.7.9-28.7.10 EFI_IPSEC_PROTOCOL (Process, DisabledEvent, DisabledFlag ; traitement paquet IPv4 et IPv6), 28.7.11 Process (début : EFI_IPSEC_FRAGMENT_DATA)
- [AGENT] Projet : IPsec hors périmètre. Règle : AuthData (pré-shared secret) est un secret, ne jamais l'écrire dans un log ni dans un fichier de notes.

## Statut
- Lignes 1 à 61812 lues. Prochaine : ligne 61813 (suite EFI_IPSEC_FRAGMENT_DATA, Process, codes).

## Lignes 61813-62112 — EFI_IPSEC_FRAGMENT_DATA, 28.7.11 Process (fin : rôle des champs IpHead, LastHead, OptionsBuffer, FragmentTable, TrafficDirection, RecycleSignal ; codes SUCCESS bypass / protect, ACCESS_DENIED discard), 28.7.12-28.7.14 EFI_IPSEC2_PROTOCOL / ProcessExt (gère les extension headers ; OptionsBuffer et FragmentTable en entrée/sortie ; tunnel inbound = en-tête interne ; NOT_READY = négociation IKE lancée et paquet jeté), 28.8 EFI FTPv4 (FTP client, télécommande TELNET, une opération de transfert par instance), 28.8.1 FTP4 Service Binding (GUID, adresses IP network order, ports host order), 28.8.2 EFI_FTP4_PROTOCOL (GetModeData, Connect, Close, Configure, ReadFile, WriteFile, ReadDirectory, Poll), 28.8.3 GetModeData (Username/Password alloués par la fonction : l'appelant libère), 28.8.4 Connect (EFI_FTP4_CONNECTION_TOKEN ; codes réseau ICMP/TIMEOUT/CONNECTION_RESET ; Event NULL = bloquant), 28.8.5 Close (début)
- [AGENT] Projet : FTP, IPsec hors périmètre. Règle : un service qui alloue un buffer retourné le rend à l'appelant (ici Username/Password).

## Statut
- Lignes 1 à 62112 lues. Prochaine : ligne 62113 (suite 28.8.5 Close FTP4, Configure, ReadFile, WriteFile, ReadDirectory, Poll).

## Lignes 62113-62412 — 28.8.5 Close FTP4 (fin : token ; codes), 28.8.6 Configure FTP4 (EFI_FTP4_CONFIG_DATA : Username/Password alloués par GetModeData ; Active vs passive ; UseDefaultSetting ; StationIp/SubnetMask/GatewayIp ; ServerIp ; ServerPort 21 par convention ; AltDataPort 0 = ServerPort-1 ; RepType (nibble bas = type image/EBCDIC/ASCII/local, nibble haut = non-print/Telnet/carriage control) ; FileStruct file/record/page ; TransMode stream/block/compressed ; NULL = reset + flush), 28.8.7 ReadFile (EFI_FTP4_COMMAND_TOKEN : Pathname, DataBufferSize, DataBuffer, DataCallback appelé quand le buffer est plein puis réécriture au début, Context, Status ; codes), EFI_FTP4_DATA_CALLBACK, 28.8.8 WriteFile (DataBufferSize = nb octets à transférer ; callback met DataBufferSize à 0 pour fin ; peut être UNSUPPORTED), 28.8.9 ReadDirectory (début)
- [AGENT] Projet : FTP hors périmètre. Règle : FTP en clair ; le mot de passe est transmis en clair par le protocole.

## Statut
- Lignes 1 à 62412 lues. Prochaine : ligne 62413 (suite 28.8.9 ReadDirectory, Poll FTP4, puis 28.9 TLS...).

## Lignes 62413-62712 — 28.8.9 ReadDirectory FTP4 (fin : liste de fichiers liés au Pathname ; peut être UNSUPPORTED), 28.8.10 Poll FTP4, 28.9 EFI TLS Protocols, 28.9.1 TLS Service Binding (GUID 952cb795-ff36-48cf-a249-4df486d6ab8d ; CreateChild -> EFI_TLS_PROTOCOL + EFI_TLS_CONFIGURATION_PROTOCOL ; DestroyChild après usage), 28.9.2 EFI_TLS_PROTOCOL (SetSessionData, GetSessionData, BuildResponsePacket pour alert/handshake/change_cipher_spec, ProcessPacket pour application_data après connexion), 28.9.3 SetSessionData (toutes données avant BuildResponsePacket), types EFI_TLS_SESSION_DATA_TYPE (Version, ConnectionEnd, CipherList, CompressionMethod, ExtensionData, VerifyMethod, SessionID, SessionState, ClientRandom, ServerRandom, KeyMaterial, VerifyHost), EFI_TLS_VERSION, EFI_TLS_CONNECTION_END (Client/Server), EFI_TLS_CIPHER (RFC 5246), EFI_TLS_COMPRESSION (RFC 3749), EFI_TLS_EXTENSION, EFI_TLS_VERIFY (NONE ou PEER obligatoire ; FAIL_IF_NO_PEER_CERT serveur seulement ; CLIENT_ONCE), EFI_TLS_VERIFY_HOST (Flags + HostName), EFI_TLS_VERIFY_HOST_FLAG (ALWAYS_CHECK_SUBJECT, NO_WILDCARDS, NO_PARTIAL_WILDCARDS, MULTI_LABEL_WILDCARDS, SINGLE_LABEL_SUBDOMAINS)
- [AGENT] Projet : TLS hors périmètre. Règle : VERIFY_NONE désactive l'authentification du serveur ; à éviter.

## Statut
- Lignes 1 à 62712 lues. Prochaine : ligne 62713 (suite TLS : EFI_TLS_VERIFY_HOST_FLAG, GetSessionData, BuildResponsePacket, ProcessPacket...).

## Lignes 62713-63012 — EFI_TLS_VERIFY_HOST_FLAG (fin : NEVER_CHECK_SUBJECT 0x20 ; ALWAYS et NEVER ensemble = INVALID ; NO_WILDCARDS avec NO_PARTIAL ou MULTI_LABEL = INVALID), EFI_TLS_RANDOM (GmtUnixTime + 28 octets), EFI_TLS_MASTER_SECRET (48 octets), EFI_TLS_SESSION_ID (max 32), EFI_TLS_SESSION_STATE (NotStarted -> HandShaking -> DataTransferring -> Closing ; Error via ProcessPacket abort -> BuildResponsePacket(NULL) pour alerte), codes SetSessionData (ACCESS_DENIED pour ClientRandom/ServerRandom/KeyMaterial ; NOT_READY si état non NotStarted), 28.9.4 GetSessionData (DataSize entrée/sortie ; BUFFER_TOO_SMALL donne la taille), 28.9.5 BuildResponsePacket (RequestBuffer NULL = ClientHello ou CloseNotify ou alerte selon état), 28.9.6 ProcessPacket (EFI_TLS_FRAGMENT_DATA, EFI_TLS_CRYPT_MODE Encrypt/Decrypt ; seulement en DataTransferring), 28.9.7 EFI TLS Configuration Protocol (début, GUID 1682fe44-...)
- [AGENT] Projet : TLS hors périmètre. Règle : ne jamais exposer MasterSecret/KeyMaterial dans un log.

## Statut
- Lignes 1 à 63012 lues. Prochaine : ligne 63013 (suite EFI_TLS_CONFIGURATION_PROTOCOL : GetData/SetData, types config).

## Lignes 63013-63097 — 28.9.7 fin (EFI_TLS_CONFIGURATION_PROTOCOL : SetData, GetData), 28.9.8 SetData config TLS (types EFI_TLS_CONFIG_DATA_TYPE : HostPublicCert DER ou PEM, HostPrivateKey, CACertificate, CertRevocationList DER ; codes), 28.9.9 GetData config TLS (codes), fin du chapitre 28 à la ligne 63097.

## Lignes 63098-63312 — CHAPITRE 29 : Network Protocols ARP, DHCP, DNS, HTTP, REST
- 29.1 ARP (RFC 826 et 1122). 29.1.1 ARP Service Binding (GUID f44c00ee-...) : les instances enfants d'un même binding partagent un cache ARP. 29.1.2 EFI_ARP_PROTOCOL (Configure, Add, Find, Delete, Flush, Request, Cancel). Adresses en network byte order ; autres paramètres en host byte order.
- 29.1.3 Configure ARP : EFI_ARP_CONFIG_DATA (SwAddressType 16 bits host order, ex. 0x0800 = IP ; SwAddressLength ; StationAddress network byte order ; EntryTimeOut, RetryCount, RetryTimeOut en unités 100 ns, 0 = implémentation). NULL = reset. Changer le type/adresse d'une instance configurée = erreur.
- 29.1.4 Add ARP : entrées "normal" (SW+HW, clé = SW) ou "deny" (SW ou HW, pas les deux). Overwrite contrôle l'écrasement. TimeoutValue 0 = permanent.
- [AGENT] Projet : ARP, réseau hors périmètre ; mais règle générale : ne pas réparer une entrée ARP via un Add sans overwrite explicite.

## Statut
- Lignes 1 à 63312 lues. Prochaine : ligne 63313 (suite 29.1.5 ARP Find : BySwAddress, AddressBuffer, EntryLength, EntryCount, Entries, Refresh).

## Lignes 63313-63512 — 29.1.5 Find ARP (BySwAddress, AddressBuffer NULL = tout, EntryLength, EntryCount, Entries alloué par la fonction, Refresh ne touche pas les entrées permanentes ; EFI_ARP_FIND_DATA : Size, DenyFlag, StaticFlag, HwAddressType, SwAddressType, HwAddressLength, SwAddressLength), 29.1.6 Delete ARP, 29.1.7 Flush ARP (ne supprime que les dynamiques ajoutées par cette interface), 29.1.8 Request ARP (TargetSwAddress NULL = broadcast HW immédiat ; multicast IP via McastIpToMac ; ResolvedEvent ; TargetHwAddress rempli à zéro si échec ; ACCESS_DENIED si adresse en deny), 29.1.9 Cancel ARP (SW et Event tous deux NULL = tout annuler ; l'un sans l'autre = INVALID), 29.2 EFI DHCPv4 Protocol (intro : configuration IPv4, découverte serveur DHCP et PXE), 29.2.1 DHCP4 Service Binding (GUID 9d9a39d8-...), instance prête après CreateChild, 29.2.2 EFI_DHCP4_PROTOCOL (GetModeData, Configure, Start, RenewRebind, Release, Stop, Build, TransmitReceive, Parse), 29.2.3 GetModeData DHCP4 (début)
- [AGENT] Projet : DHCP/ARP hors périmètre. Règle notable : Cancel avec un seul des deux paramètres NULL est une erreur ; Flush ne touche pas les entrées statiques.

## Statut
- Lignes 1 à 63512 lues. Prochaine : ligne 63513 (suite 29.2.3 GetModeData DHCP4 : EFI_DHCP4_MODE_DATA, structures).

## Lignes 63513-63812 — 29.2.3 GetModeData DHCP4 : EFI_DHCP4_MODE_DATA (State, ConfigData, ClientAddress (0 = acquisition non finie), ClientMacAddress, ServerAddress, RouterAddress, SubnetMask, LeaseTime en secondes (0xFFFFFFFF = infini ; défaut 7 jours), ReplyPacket), EFI_DHCP4_STATE (Stopped 0, Init 1, Selecting 2, Requesting 3, Bound 4, Renewing 5, Rebinding 6, InitReboot 7, Rebooting 8 ; machine RFC 2131 ; partagée par toutes les instances d'un même périphérique), EFI_DHCP4_PACKET (pack(1) : Size, Length, Header, Magik cookie network order, Option), 29.2.4 Configure DHCP4 (appel valide seulement en Stopped, Init, InitReboot, Bound et si aucune autre instance ne l'a configuré ; NULL = reset des données par défaut sans changer d'état ; non-NULL depuis Stopped -> Init ou InitReboot selon ClientAddress/options RFC 2131), EFI_DHCP4_CONFIG_DATA (début : DiscoverTryCount, DiscoverTimeout, RequestTryCount, RequestTimeout, ClientAddress, Dhcp4Callback)
- [AGENT] Projet : DHCP hors périmètre. Règle notable : la machine DHCP est partagée entre instances du même périphérique (concurrence à éviter).

## Statut
- Lignes 1 à 63812 lues. Prochaine : ligne 63813 (suite EFI_DHCP4_CONFIG_DATA : callback, OptionCount, OptionList, codes Configure, Start...).

## Lignes 63813-64112 — EFI_DHCP4_CONFIG_DATA (fin : CallbackContext, OptionCount, OptionList libérable après Configure ; Pad ajoutés auto), 29.2.4 Configure DHCP4 (fin : codes), EFI_DHCP4_CALLBACK (retour SUCCESS = continuer / sélection terminée en Selecting, NOT_READY = attendre d'autres offres, ABORTED = retour Init/InitReboot ; NewPacket ne doit pas être le Packet lui-même), EFI_DHCP4_EVENT (SendDiscover 1, RcvdOffer 2, SelectOffer 3, SendRequest 4, RcvdAck 5, RcvdNak 6, SendDecline 7, BoundCompleted 8, EnterRenewing 9, EnterRebinding 10, AddressLost 11, Fail 12 ; ABORTED sur RcvdAck -> SendDecline ; RcvdNak -> Init toujours), EFI_DHCP4_HEADER (pack(1), réseau : OpCode 1 request / 2 reply, Xid, Seconds, ClientAddr, YourAddr, ServerAddr, GatewayAddr, ClientHwAddr[16], ServerName[64], BootFileName[128]), EFI_DHCP4_PACKET_OPTION (OpCode, Length absent si OpCode 0 ou 255, Data ; accès multi-octets non aligné : prudence), codes Configure, 29.2.5 Start DHCP4 (début : CompletionEvent ; NULL = bloquant jusqu'à Bound ou échec)
- [AGENT] Projet : DHCP hors périmètre. Règle notable : le callback ne doit pas garder le pointeur Packet (copier seulement).

## Statut
- Lignes 1 à 64112 lues. Prochaine : ligne 64113 (suite 29.2.5 Start DHCP4 : description, codes ; 29.2.6 RenewRebind...).

## Lignes 64113-64412 — 29.2.5 Start DHCP4 (fin : n'est appelable qu'en Init ou InitReboot ; succès -> Bound via Selecting/Requesting ; échec -> Init, rappelable ; codes), 29.2.6 RenewRebind (RebindRequest TRUE = broadcast Rebinding ; FALSE = unicast Renewing ; sans réponse avant expiration du bail -> retour Bound, config précédente conservée), 29.2.7 Release (DHCPRELEASE en Bound ; ou 0.0.0.0 en InitReboot ; retour Init ; paquets entrants ignorés), 29.2.8 Stop (de n'importe quel état -> Stopped), 29.2.9 Build (construit un paquet sans changer d'état ; FreePool requis sur NewPacket), 29.2.10 TransmitReceive (début : EFI_DHCP4_TRANSMIT_RECEIVE_TOKEN : Status, CompletionEvent, RemoteAddress, RemotePort 67 par défaut, GatewayAddress, ListenPoints (défaut station + port 68), TimeoutValue (0 invalide), Packet, ResponseCount, ResponseList à libérer par FreePool), EFI_DHCP4_LISTEN_POINT (ListenAddress, SubnetMask, ListenPort)
- [AGENT] Projet : DHCP hors périmètre. Règle : Build et TransmitReceive ne changent pas l'état de la machine DHCP ; Release et Stop le changent.

## Statut
- Lignes 1 à 64412 lues. Prochaine : ligne 64413 (suite codes TransmitReceive, 29.2.11 Parse...).

## Lignes 64413-64712 — 29.2.10 TransmitReceive DHCP4 (codes : NOT_READY si appel précédent non fini, transaction ID en usage = INVALID), 29.2.11 Parse DHCP4 (OptionCount entrée/sortie ; fin/pad non inclus ; RFC 3396 : réassembler les options répétées ; BUFFER_TOO_SMALL si liste trop petite), 29.3 EFI DHCP6 Protocol, 29.3.2 DHCP6 Service Binding (GUID 9fb9a8a1-...), 29.3.4 EFI_DHCP6_PROTOCOL (GetModeData, Configure, Start, InfoRequest, RenewRebind, Decline, Release, Stop, Parse ; adresses en network byte order), 29.3.5 GetModeData DHCP6 (ModeData et ConfigData : libérés par l'appelant), EFI_DHCP6_MODE_DATA (ClientId DUID, Ia), EFI_DHCP6_DUID (Length + octets ; DUID-UUID pour toutes les transactions), EFI_DHCP6_IA (Descriptor, State, ReplyPacket, IaAddressCount, IaAddress ; adresses utilisables seulement en Bound/Renewing/Rebinding), EFI_DHCP6_IA_DESCRIPTOR (Type NA = 3, TA = 4 ; IaId), EFI_DHCP6_STATE (Init 0, Selecting 1, Requesting 2, Declining 3, Confirming 4, Releasing 5, ...)
- [AGENT] Projet : DHCP hors périmètre.

## Statut
- Lignes 1 à 64712 lues. Prochaine : ligne 64713 (suite EFI_DHCP6_STATE, EFI_DHCP6_PACKET, EFI_DHCP6_IA_ADDRESS, Configure DHCP6).

## Lignes 64713-65012 — EFI_DHCP6_STATE (fin : Bound 6, Renewing 7, Rebinding 8 ; Declining = Decline, Confirming = Confirm, Releasing = Release), EFI_DHCP6_IA_ADDRESS (IpAddress, PreferredLifetime, ValidLifetime en secondes), EFI_DHCP6_PACKET (pack(1) : Size, Length, Header, Option ; RFC 3315), EFI_DHCP6_HEADER (TransactionId 24 bits, MessageType 8 bits), codes GetModeData, 29.3.6 Configure DHCP6 (cfg non NULL -> état Init ; NULL -> nettoyage ; pour changer, nettoyer d'abord), EFI_DHCP6_CONFIG_DATA (Dhcp6Callback, CallbackContext, OptionList sans Client Identifier ni IA ; IaDescriptor ; IaInfoEvent : si NULL, Start/RenewRebind/Decline/Release/Stop bloquent ; ReconfigureAccept ; RapidCommit ; SolicitRetransmission), EFI_DHCP6_CALLBACK (SUCCESS continuer / ABORTED -> Init ; NewPacket ne partage pas le buffer de Packet ; l'instance libère Packet), EFI_DHCP6_PACKET_OPTION (pack(1) : OpCode 16 bits, OpLen 16 bits, Data)
- [AGENT] Projet : DHCPv6 hors périmètre.

## Statut
- Lignes 1 à 65012 lues. Prochaine : ligne 65013 (suite EFI_DHCP6_PACKET_OPTION, EFI_DHCP6_EVENT, EFI_DHCP6_RETRANSMISSION, Start DHCP6, InfoRequest...).

## Lignes 65013-65312 — EFI_DHCP6_PACKET_OPTION (fin : OpCode et OpLen en network order), EFI_DHCP6_EVENT (SendSolicit 0, RcvdAdvertise 1, SelectAdvertise 2, SendRequest 3, RcvdReply 4, RcvdReconfigure 5, SendDecline 6, SendConfirm 7, SendRelease 8, SendRenew 9, SendRebind 10), EFI_DHCP6_RETRANSMISSION (Irt, Mrc, Mrt, Mrd ; 0 = pas de limite), codes Configure DHCP6 (IaDescriptor type NA ou TA, IaId unique ; IaInfoEvent et SolicitRetransmission ne peuvent pas être NULL tous deux ; Mrc et Mrd nuls ensemble = INVALID ; ACCESS_DENIED si déjà configurée), 29.3.7 Start DHCP6 (n'est appelable qu'en Init ; IaInfoEvent NULL = bloquant ; échec système -> Init), 29.3.8 InfoRequest (SendClientId, OptionRequest, OptionList, Retransmission, TimeoutEvent, ReplyCallback obligatoire, CallbackContext ; callback : SUCCESS fin, NOT_READY continuer, ABORTED arrêt ; codes), EFI_DHCP6_INFO_CALLBACK, 29.3.9 RenewRebind DHCP6 (Renew -> Renewing, Rebind -> Rebinding ; échec Rebind -> Init et IA inutilisable ; échec Renew -> Bound avec timers qui continuent), 29.3.10 Decline (début)
- [AGENT] Projet : DHCPv6 hors périmètre. Règle notable : un échec de Rebind rend l'IA inutilisable.

## Statut
- Lignes 1 à 65312 lues. Prochaine : ligne 65313 (suite 29.3.10 Decline DHCP6 : codes ; 29.3.11 Release ; Stop ; Parse DHCP6).

## Lignes 65313-65612 — 29.3.10 Decline DHCP6 (AddressCount, Addresses ; ne se fait qu'en Bound ; tout décliné -> Declining -> Init, sinon Bound ; codes), 29.3.11 Release DHCP6 (AddressCount 0 = toutes les adresses de l'IA ; Releasing -> Init si tout libéré ; codes), 29.3.12 Stop DHCP6 (libère toutes les adresses -> Init), 29.3.13 Parse DHCP6 (OptionCount ; OpCode et OpLen network order ; BUFFER_TOO_SMALL), 29.4 EFI DNSv4 Protocol (RFC 1035 obligatoire ; LLMNR RFC 4795 optionnel, DNS prioritaire ; cache DNS commun entre instances ; chaque instance lie à une interface), 29.4.1 DNS4 Service Binding (GUID b625b186-...), 29.4.2 EFI_DNS4_PROTOCOL (GetModeData, Configure, HostNameToIp, IpToHostName, GeneralLookUp, UpdateDnsCache, Poll, Cancel), 29.4.3 GetModeData DNS4 (DnsModeData, DnsServerList et DnsCacheList alloués par le driver, libérés par l'appelant), EFI_DNS4_MODE_DATA, EFI_DNS4_CONFIG_DATA (DnsServerListCount 0 = via DHCP ; EnableDnsCache ; Protocol ; StationIp ; SubnetMask ; LocalPort ; RetryCount ; RetryInterval)
- [AGENT] Projet : DNS/DHCP hors périmètre. Règle notable : DNS a priorité sur LLMNR ; le cache DNS est partagé entre instances.

## Statut
- Lignes 1 à 65612 lues. Prochaine : ligne 65613 (suite EFI_DNS4_CONFIG_DATA, Configure DNS4, HostNameToIp...).

## Lignes 65613-65912 — EFI_DNS4_CONFIG_DATA (fin : EnableDnsCache, Protocol TCP/UDP seulement, StationIp/SubnetMask si pas de défaut, LocalPort 0 = auto, RetryCount, RetryInterval minimum 2 s), EFI_DNS4_CACHE_ENTRY (HostName CHAR16, IpAddress, Timeout ; 0 = permanent), 29.4.4 Configure DNS4 (NULL = reset ; second appel sans reset = ALREADY_STARTED ; ListCount et List incohérents = INVALID ; StationIp unicast requis), 29.4.5 HostNameToIp (FQDN ASCII ; Punycode à la charge de l'appelant ; requête type A ; token : Event NOTIFY_SIGNAL, Status SUCCESS/NOT_FOUND/TIMEOUT/DEVICE_ERROR/NO_MEDIA, RetryCount/RetryInterval 0 = valeur de Configure, union RspData), DNS_HOST_TO_ADDR_DATA (IpCount, IpList), DNS_ADDR_TO_HOST_DATA (HostName : à libérer par l'appelant), DNS_GENERAL_LOOKUP_DATA (RRCount, RRList), DNS_RESOURCE_RECORD (QName, QType, QClass, TTL 0 = non cacheable, DataLength, RData), 29.4.6 IpToHostName (début : résolution inverse PTR)
- [AGENT] Projet : DNS hors périmètre.

## Statut
- Lignes 1 à 65912 lues. Prochaine : ligne 65913 (suite 29.4.6 IpToHostName : codes, 29.4.7 GeneralLookUp, UpdateDnsCache, Poll, Cancel).

## Lignes 65913-66212 — 29.4.6 IpToHostName (fin : PTR, optionnel ; codes ; Token déjà utilisé = INVALID/ALREADY), 29.4.7 GeneralLookUp (QName, QType, QClass, Token ; retourne tous les RR ; optionnel ; codes), 29.4.8 UpdateDnsCache (DeleteFlag, Override, DnsCacheEntry ; Timeout 0 = invalide ; ACCESS_DENIED si existe sans Override), 29.4.9 Poll DNS4, 29.4.10 Cancel DNS4 (Token NULL = tout annuler ; NOT_FOUND si terminé), 29.5 EFI DNSv6 (RFC 3596 ; LLMNR RFC 4795 optionnel ; cache commun), 29.5.2 DNS6 Service Binding (GUID 7f1647c8-...), 29.5.4 EFI_DNS6_PROTOCOL (mêmes 8 services), 29.5.5 GetModeData DNS6 (EFI_DNS6_MODE_DATA : DnsConfigData, DnsServerList IPv6, DnsCacheList)
- [AGENT] Projet : DNS hors périmètre.

## Statut
- Lignes 1 à 66212 lues. Prochaine : ligne 66213 (suite EFI_DNS6_MODE_DATA : EFI_DNS6_CONFIG_DATA, EFI_DNS6_CACHE_ENTRY, Configure DNS6...).

## Lignes 66213-66512 — EFI_DNS6_CONFIG_DATA (IsDnsServerAuto, EnableDnsCache, Protocol TCP/UDP, StationIp 0 = choix du driver IPv6 sinon adresse configurée, LocalPort, DnsServerCount/List, RetryCount, RetryInterval >= 2 s), EFI_DNS6_CACHE_ENTRY (HostName interprété en Unicode, IpAddress IPv6, Timeout), codes GetModeData, 29.5.6 Configure DNS6 (NULL = reset ; second appel sans reset = ALREADY_STARTED), 29.5.7 HostNameToIp DNS6 (même logique que DNS4, requête AAAA ; token réutilisé avant fin = ALREADY_STARTED), EFI_DNS6_COMPLETION_TOKEN (Event obligatoire ; RspData union ; Status ; RetryCount/Interval de Configure), DNS6_HOST_TO_ADDR_DATA (IPv6), DNS6_ADDR_TO_HOST_DATA (à libérer par appelant), DNS6_GENERAL_LOOKUP_DATA, DNS6_RESOURCE_RECORD (début)
- [AGENT] Projet : DNS hors périmètre. Règle notable : un token ne se réutilise pas avant signal ou annulation.

## Statut
- Lignes 1 à 66512 lues. Prochaine : ligne 66513 (suite DNS6_RESOURCE_RECORD, IpToHostName6, GeneralLookUp6, UpdateDnsCache6, Poll6, Cancel6).

## Lignes 66513-66812 — DNS6_RESOURCE_RECORD (fin), 29.5.7 HostNameToIp DNS6 (fin : AAAA ; codes : ALREADY_STARTED si token dans une autre session), 29.5.8 IpToHostName DNS6 (PTR ; optionnel), 29.5.9 GeneralLookUp DNS6 (optionnel), 29.5.10 UpdateDnsCache DNS6, 29.5.11 Poll DNS6, 29.5.12 Cancel DNS6 (NOT_FOUND si terminé), 29.6 EFI HTTP Protocols (HTTP sur TCP ; HTTPSB GUID bdc8e6af-...), 29.6.3 EFI_HTTP_PROTOCOL (GUID 7A59B29B-910B-4171-8242-A85A0DF25B5B ; GetModeData, Configure, Request, Cancel, Response, Poll ; requêtes non bloquantes), 29.6.4 GetModeData HTTP (début)
- [AGENT] Projet : HTTP hors périmètre.

## Statut
- Lignes 1 à 66812 lues. Prochaine : ligne 66813 (suite GetModeData HTTP : EFI_HTTP_CONFIG_DATA, Configure HTTP...).

## Lignes 66813-67112 — 29.6.4 GetModeData HTTP (fin : buffer appelant, IPv6Node recommandé car le plus grand), EFI_HTTP_CONFIG_DATA (HttpVersion, TimeOutMillisec, LocalAddressIsIPv6 : FALSE = DNS4/TCP4, TRUE = DNS6/TCP6 ; AccessPoint union IPv4Node/IPv6Node), EFI_HTTP_VERSION (1.0, 1.1, Unsupported), EFI_HTTPv4_ACCESS_POINT (UseDefaultAddress, LocalAddress, LocalSubnet, LocalPort), EFI_HTTPv6_ACCESS_POINT (LocalAddress, LocalPort), 29.6.5 Configure HTTP (NULL = reset : ferme connexions, annule tokens, vide buffers sans prévenir ; aucune autre fonction avant Configure réussi), 29.6.6 Request HTTP (non bloquant ; EFI_HTTP_TOKEN : Event NOTIFY_SIGNAL TPL <= CALLBACK, Status SUCCESS/HTTP_ERROR (code dans token)/ABORTED/TIMEOUT/DEVICE_ERROR, Message ; EFI_HTTP_MESSAGE : union Request/Response, HeaderCount, Headers (alloué par driver en réponse, libéré par appelant), BodyLength, Body ; envoi par morceaux PUT/POST : Body seul = suite vers dernier hôte ; nouvelle connexion TCP si hôte différent (RFC 2616)), EFI_HTTP_REQUEST_DATA (Method, Url : port 80 ou 443 par défaut ; RFC 3986), EFI_HTTP_CONNECT_REQUEST_DATA (proxy : ProxyUrl pour HttpMethodConnect)
- [AGENT] Projet : HTTP hors périmètre. Règle notable : une requête interrompue doit être annulée explicitement (sinon token reste dans la file).

## Statut
- Lignes 1 à 67112 lues. Prochaine : ligne 67113 (suite EFI_HTTP_CONNECT_REQUEST_DATA, codes Request, EFI_HTTP_RESPONSE_DATA, Cancel, Response, Poll).

## Lignes 67113-67412 — EFI_HTTP_METHOD (Get, Post, Patch, Options, Connect, Head, Put, Delete, Trace), EFI_HTTP_RESPONSE_DATA (StatusCode), EFI_HTTP_HEADER (FieldName, FieldValue : chaînes NUL, RFC 2616 §14), EFI_HTTP_STATUS_CODE (enum complète 100 à 505 + 308 ; 0 = UNSUPPORTED), codes Request, 29.6.7 Cancel HTTP (Token NULL = toutes les requêtes/réponses ; NOT_FOUND si absent), 29.6.8 Response HTTP (token de réception ; Body + BodyLength : copie jusqu'à BodyLength octets et met à jour BodyLength ; réception partielle en morceaux ; ACCESS_DENIED si aucune connexion TCP ouverte vers l'hôte de la réponse), 29.6.9 Poll HTTP, 29.6.9.1 Exemple de client (téléchargement de bundle de pilotes via proxy ; BUFFER_SIZE 1 Mo ; callbacks d'événements ; AllocatePool ; LocateProtocol du service binding HTTP ; début du code)
- [AGENT] Projet : HTTP hors périmètre. Règle notable : le client doit garder une connexion TCP ouverte avant Response, sinon ACCESS_DENIED.

## Statut
- Lignes 1 à 67412 lues. Prochaine : ligne 67413 (suite exemple HTTP : création du child, Configure, Request, Response, gestion proxy).

## Lignes 67413-67700 — Exemple HTTP complet (fin) : CreateChild HTTP, HandleProtocol, Configure (HttpVersion11, TimeOut 0 = défaut, IPv4 UseDefaultAddress TRUE via DHCP), Request GET avec en-tête Host = proxy (le driver ne résout pas le proxy), token avec Event NOTIFY_SIGNAL, boucle Poll + GetTime (timeout de 10 s), Cancel si non terminé ; Response (Headers alloués par driver, Body = buffer, BodyLength = max), lecture du Content-Length, boucle de téléchargement par morceaux (Response avec Headers/Data NULL = copie directe dans Body), 29.6.10-29.6.13 HTTP Utilities (Build : SeedMessage + DeleteList + AppendList -> NewMessage ; Parse : texte brut -> paires clé/valeur ; sans réseau), 29.7 EFI REST Support (REST via HTTP ; EFI_REST protocol UEFI 2.5 léger, EFI_REST_EX UEFI 2.8 ; in-band (BMC/Redfish) et out-of-band ; l'authentification est à la charge du client REST), 29.7.1-29.7.2 scénarios (Redfish), 29.7.3-29.7.4 EFI_REST_PROTOCOL (GUID 0DB48A36-..., début : SendReceive)
- [AGENT] Projet : HTTP/REST hors périmètre. Règle notable : un client HTTP doit faire Cancel quand la requête ne termine pas, et l'authentification REST n'est jamais gérée par la couche EFI.

## Statut
- Lignes 1 à 67700 lues. Prochaine : ligne 67701 (suite EFI_REST_PROTOCOL : SendReceive, fonctions, EFI_REST_EX_PROTOCOL...).

## Lignes 67701-68000 — 29.7.4-29.7.6 EFI_REST_PROTOCOL (GUID 0DB48A36-4E54-EA9C-...; SendReceive : RequestMessage/ResponseMessage EFI_HTTP_MESSAGE ; réponses intermédiaires 1xx gérées par le protocole, non remises à l'appelant ; buffers Response/Headers/Body alloués par la fonction, libérés par l'appelant ; codes SUCCESS/INVALID/DEVICE_ERROR/TIMEOUT ; GetServiceTime optionnel, UNSUPPORTED si absent), 29.7.7-29.7.8 EFI REST EX Service Binding (GUID 456bbe01-99d0-45ea-...; Configure optionnel selon driver), 29.7.9 EFI_REST_EX_PROTOCOL (GUID 55648b91-...; SendReceive, GetServiceTime, GetService, GetModeData, Configure, AsyncSendReceive, EventService), 29.7.10 SendReceive EX (NOT_READY si Configure non fait), 29.7.11 GetService (EFI_REST_EX_SERVICE_INFO versionné : Header Length + Version Major/Minor ; type de service, mode In-band/Out-of-band, VendorRestServiceName GUID, VendorSpecifcData ; device path du service = sous-type 32 du Messaging ; GetService utilisable sans Configure ; mémoire libérée par l'appelant), EFI_REST_EX_SERVICE_INFO_HEADER, EFI_REST_EX_SERVICE_INFO_VER, EFI_REST_EX_SERVICE_INFO union (v1.0 : RestExServiceType, RestServiceAccessMode, VendorRestServiceName, VendorSpecificDataLength, VendorSpecifcData)
- [AGENT] Projet : REST hors périmètre. Règle notable : toute structure versionnée doit être lue via son header Length + Version, jamais en supposant la taille.

## Statut
- Lignes 1 à 68000 lues. Prochaine : ligne 68001 (suite EFI_REST_EX_SERVICE_INFO v1.0, EFI_REST_EX_SERVICE_TYPE, ACCESS_MODE, GetModeData, Configure, AsyncSendReceive, EventService).

## Lignes 68001-68300 — EFI_REST_EX_SERVICE_INFO_V_1_0 (fin : RestExConfigType, RestExConfigDataLength), EFI_REST_EX_SERVICE_TYPE (UNSPECIFIC 1, REDFISH 2, ODATA 3, VENDOR_SPECIFIC 0xFF ; UNSPECIFIC = seul SendReceive, les autres retournent UNSUPPORTED), EFI_REST_EX_SERVICE_ACCESS_MODE (IN_BAND 1 = via BMC/EC ; OUT_OF_BAND 2 = réseau), EFI_REST_EX_CONFIG_TYPE (HTTP 0 ; UNSPECIFIC 1), EFI_REST_EX_HTTP_CONFIG_DATA (HttpConfigData + SendReceiveTimeout ms), 29.7.12 GetModeData REST EX (EFI_REST_EX_CONFIG_DATA = UINT8*, libéré par appelant ; NOT_READY si non configuré), 29.7.13 Configure REST EX (NULL = retour non configuré ; tant que non configuré, toutes les fonctions sauf GetService renvoient NOT_READY ; plusieurs clients peuvent utiliser des ports TCP différents), 29.7.14 AsyncSendReceive (RequestMessage NULL = annulation du token ; timeout en ms, NULL = pas de timeout ; signal EFI_TIMEOUT / ABORTED / DEVICE_ERROR / SUCCESS ; token EFI_REST_EX_TOKEN : Event, Status, ResponseMessage valide seulement si SUCCESS ; buffers libérés par appelant ; UNSPECIFIC -> UNSUPPORTED), 29.7.15 EventService (abonnement à des changements de ressources ; RequestMessage NULL = désabonnement ; UNSUPPORTED si UNSPECIFIC ; l'abonnement est retiré si le service renvoie une erreur)
- [AGENT] Projet : REST hors périmètre. Règle notable : une opération asynchrone doit être annulable explicitement, et le token ne doit être réutilisé qu'après signal ou annulation.

## Statut
- Lignes 1 à 68300 lues. Prochaine : ligne 68301 (suite 29.7.15 EventService : codes ; puis section 29.8 ou chapitre 30 suivant).

## Lignes 68301-68600 — 29.7.15 EventService (fin : codes), 29.7.15.1 Exemple d'usage REST EX (LocateHandleBuffer du service binding REST EX ; pour chaque handle : CreateChild, OpenProtocol REST EX, GetService, choix du service préféré : OUT_OF_BAND + REDFISH + CONFIG_TYPE_HTTP ; Configure ; SendReceive ; DestroyChild en sortie ; note : le code de l'exemple contient une variante de nom de champ, à ne pas copier tel quel), 29.7.15.1.1 AsyncSendReceive HTTP (en-tête Prefer: respond-async RFC 7240 ; 202 Accepted = succès immédiat ; URI Location à interroger par timer ; UNSUPPORTED si service incapable), 29.7.15.1.2 EventService HTTP (201 Created ; Location = abonnement), 29.7.16 EventService (doublon de la section précédente), 29.7.17 EFI REST JSON Structure Protocol (convertisseur JSON <-> structure C ; enregistrement par type/version ; GUID a9a048f6-48a0-4714-...), 29.7.18 Register (JsonStructureSupported, ToStructure, ToJson, DestroyStructure ; EFI_ALREADY_STARTED si même type et révision déjà enregistré ; EFI_REST_JSON_RESOURCE_TYPE_NAMESPACE : ResourceTypeName, Major/Minor/Errata ; EFI_REST_JSON_STRUCTURE_SUPPORTED en liste chaînée), 29.7.19 ToStructure (début : JSON texte -> structure)
- [AGENT] Projet : REST/JSON hors périmètre. Remarque : cette zone de la spécification contient beaucoup de sauts de page vides (pages 1423-1430) et de doublons de section (29.7.15 / 29.7.16) ; ce sont des erreurs de mise en forme, pas des exigences.

## Statut
- Lignes 1 à 68600 lues. Prochaine : ligne 68601 (suite 29.7.19 ToStructure : codes, ToJson, DestroyStructure).

## Lignes 68601-68900 — 29.7.19 ToStructure (JsonRsrcIdentifier NULL = détection automatique par chaque convertisseur, UNSUPPORTED si non supporté ; ResourceTypeName obligatoire si identifiant non NULL ; EFI_NOT_FOUND si aucun convertisseur), EFI_REST_JSON_STRUCTURE_HEADER (JsonRsrcIdentifier + pointeur opaque ; structure définie par le producteur), 29.7.20 ToJson (libération du texte par le consommateur ; NOT_FOUND si pas de convertisseur), 29.7.21 DestroyStructure (le convertisseur libère tout), 29.7.21.1 renvoi au chapitre 31 (Redfish), 29.7.x fin ; CHAPITRE 30 : Network Protocols UDP et MTFTP ; 30.1.1 UDP4 Service Binding (GUID 83f01464-99bd-45e5-...), 30.1.2 EFI_UDP4_PROTOCOL (GUID 3ad9df29-4501-478d-...; GetModeData, Configure, Groups, Routes, Transmit, Receive, Cancel, Poll ; instances indépendantes ; adresses IPv4 et paquets en network byte order, autres en host byte order), 30.1.2.2 GetModeData UDP4 (Udp4ConfigData, Ip4ModeData, MnpConfigData, SnpModeData ; début EFI_UDP4_CONFIG_DATA : AcceptBroadcast, AcceptPromiscuous, AcceptAnyPort, AllowDuplicatePort, TypeOfService, TimeToLive, DoNotFragment, ReceiveTimeout, TransmitTimeout, puis Access Point)
- [AGENT] Projet : UDP/MTFTP hors périmètre (réseau). Règle notable : un ReceiveTimeout/TransmitTimeout à zéro désactive la suppression des paquets.

## Statut
- Lignes 1 à 68900 lues. Prochaine : ligne 68901 (suite EFI_UDP4_CONFIG_DATA : AccessPoint, StationAddress, RemoteAddress, RemotePort, GetModeData UDP4 codes).

## Lignes 68901-69200 — EFI_UDP4_CONFIG_DATA (fin : StationAddress 0.0.0.0 = toutes adresses, SubnetMask, StationPort 0 = port aléatoire, RemoteAddress/RemotePort = connexion par défaut et filtre entrant), 30.1.2.2 GetModeData UDP4 (fin : codes), 30.1.2.3 Configure UDP4 (init / changement / reset ; NULL = flush des files, plus de trafic ; seuls TOS, TTL, DF, ReceiveTimeout, TransmitTimeout peuvent changer sans stop ; AllowDuplicatePort FALSE + port déjà pris = ACCESS_DENIED ; codes), 30.1.2.4 Groups UDP4 (JoinFlag ; FALSE + NULL = tous les groupes), 30.1.2.5 Routes UDP4 (table propre à chaque instance ; copies de la table IP4 CONFIG2 pour les instances en adresse par défaut, écrasées lors de reconfiguration), 30.1.2.6 Transmit UDP4 (début : EFI_UDP4_COMPLETION_TOKEN ; Status SUCCESS / ABORTED / TIMEOUT / NETWORK_UNREACHABLE(100) / HOST_UNREACHABLE(101) / PROTOCOL_UNREACHABLE(102) / PORT_UNREACHABLE(103) / ICMP_ERROR / DEVICE_ERROR / NO_MEDIA ; RxData = NULL dans les cas ICMP ; union RxData/TxData), EFI_UDP4_RECEIVE_DATA (début : TimeStamp, RecycleSignal, UdpSession)
- [AGENT] Projet : UDP hors périmètre. Règle notable : un ReceiveTimeout/TransmitTimeout à zéro désactive la suppression des paquets ; les codes ICMP sont des EFIERR(100..103).

## Statut
- Lignes 1 à 69200 lues. Prochaine : ligne 69201 (suite EFI_UDP4_RECEIVE_DATA : DataLength, FragmentTable, EFI_UDP4_SESSION_DATA, EFI_UDP4_TRANSMIT_DATA, Receive, Cancel, Poll UDP4).

## Lignes 69201-69500 — EFI_UDP4_RECEIVE_DATA (fin : TimeStamp, RecycleSignal à signaler après traitement, UdpSession, DataLength, FragmentTable ; IP et UDP en-têtes inclus si RawData sinon retirés ; un client doit parfois recoller des fragments pour former son propre en-tête), EFI_UDP4_SESSION_DATA (SourceAddress/SourcePort, DestinationAddress/DestinationPort ; 0 = valeur de Configure ; port 0 non lié -> Transmit échoue), EFI_UDP4_FRAGMENT_DATA (évite copies), EFI_UDP4_TRANSMIT_DATA (UdpSessionData, GatewayAddress, DataLength <= taille max UDP, FragmentCount, FragmentTable), codes Transmit (DataLength doit égaler somme ; NO_MAPPING = pas de route ; BAD_BUFFER_SIZE si > max ou > MTU avec DF), 30.1.2.7 Receive UDP4 (asynchrone, Event obligatoire, notification jamais réentrante), 30.1.2.8 Cancel UDP4 (Token NULL = tout ; NOT_FOUND si terminé), 30.1.2.9 Poll UDP4, 30.2 EFI UDPv6 (GUID UDP6 service binding 66ed4721-..., EFI_UDP6_PROTOCOL 4f948815-b4b9-43cb-...; GetModeData, Configure, Groups, Transmit, Receive, Cancel, Poll — pas de Routes ; début)
- [AGENT] Projet : UDP hors périmètre. Règle notable : une notification d'évènement UDP n'est jamais réentrante ; le client doit libérer RecycleSignal après usage.

## Statut
- Lignes 1 à 69500 lues. Prochaine : ligne 69501 (suite EFI_UDP6_PROTOCOL : Cancel, Poll ; EFI_UDP6_CONFIG_DATA ; Transmit/Receive UDP6...).

## Lignes 69501-69800 — 30.2.2.1 EFI_UDP6_PROTOCOL (fin : Cancel, Poll ; description : instance = service lié à un port ou client connecté), 30.2.2.2 GetModeData UDP6 (Udp6ConfigData, Ip6ModeData, MnpConfigData, SnpModeData ; EFI_UDP6_CONFIG_DATA : AcceptPromiscuous, AcceptAnyPort, AllowDuplicatePort, TrafficClass, HopLimit, ReceiveTimeout, TransmitTimeout, StationAddress (0:/128 = binding auto selon algorithme RFC 6724), StationPort, RemoteAddress, RemotePort), 30.2.2.3 Configure UDP6 (seuls TrafficClass, HopLimit, ReceiveTimeout, TransmitTimeout modifiables sans stop ; codes : ACCESS_DENIED port déjà utilisé sans AllowDuplicatePort ; NO_MAPPING si pas d'adresse source), 30.2.2.4 Groups UDP6 (NULL + FALSE = tous les groupes), 30.2.2.5 Transmit UDP6 (début : EFI_UDP6_COMPLETION_TOKEN : Event NOTIFY_SIGNAL, Status, union RxData/TxData)
- [AGENT] Projet : UDP hors périmètre. Règle notable : StationAddress est modifiable seulement pendant la transition stopped->started.

## Statut
- Lignes 1 à 69800 lues. Prochaine : ligne 69801 (suite Transmit UDP6 : Status, EFI_UDP6_RECEIVE_DATA, EFI_UDP6_TRANSMIT_DATA, Receive, Cancel, Poll UDP6).

## Lignes 69801-70100 — 30.2.2.5 Transmit UDP6 (fin : Status : + EFI_SECURITY_VIOLATION IPsec ; codes Transmit : DestinationAddress de la session doit être cohérente avec Configure ; BAD_BUFFER_SIZE si trop grand), EFI_UDP6_RECEIVE_DATA (TimeStamp, RecycleSignal, UdpSession, DataLength, FragmentCount, FragmentTable), EFI_UDP6_SESSION_DATA (Source = non utilisé à l'envoi ; Destination ignorée si zéro à l'envoi), EFI_UDP6_FRAGMENT_DATA, EFI_UDP6_TRANSMIT_DATA (seuls DestinationAddress et DestinationPort surchargeables), 30.2.2.6 Receive UDP6 (asynchrone ; notification non réentrante), 30.2.2.7 Cancel UDP6, 30.2.2.8 Poll UDP6 ; 30.3 EFI MTFTPv4 Protocol (construit sur UDP4 ; client TFTP unicast et/ou multicast ; une opération de transfert par instance ; GUID service binding 02e800be-8f01-4aa6-...; GUID EFI_MTFTP4_PROTOCOL 78247c57-63db-4708-...; début de la structure d'interface)
- [AGENT] Projet : UDP/TFTP hors périmètre. Règle notable : MTFTP = une opération par instance ; deux fichiers en parallèle exigent deux instances.

## Statut
- Lignes 1 à 70100 lues. Prochaine : ligne 70101 (suite EFI_MTFTP4_PROTOCOL : GetModeData, Configure, GetInfo, ParseOptions, ReadFile, WriteFile, ReadDirectory, Poll).

## Lignes 70101-70400 — EFI_MTFTP4_PROTOCOL (liste des 8 fonctions : GetModeData, Configure, GetInfo, ParseOptions, ReadFile, WriteFile (peut être UNSUPPORTED), ReadDirectory (idem), Poll), 30.3.3 GetModeData MTFTP4 (EFI_MTFTP4_MODE_DATA : ConfigData, SupportedOptions, UnsupportedOptions), EFI_MTFTP4_CONFIG_DATA (UseDefaultSetting, StationIp, SubnetMask, LocalPort, GatewayIp, ServerIp, InitialServerPort 69 par défaut, TryCount, TimeoutValue en secondes), 30.3.4 Configure MTFTP4 (NULL = reset et abort de toute opération ; passage d'un serveur à un autre ; codes : passerelle hors sous-réseau = INVALID), 30.3.5 GetInfo MTFTP4 (début : envoie une requête avec options, peut recevoir OACK, ERROR ou ICMP ERROR ; retries seulement si aucun paquet reçu ; EFI_MTFTP4_OVERRIDE_DATA : GatewayIp, ServerIp, ServerPort, TryCount, TimeoutValue ; EFI_MTFTP4_OPTION (début) ; Packet à libérer par l'appelant)
- [AGENT] Projet : TFTP hors périmètre. Règle notable : pas de retry si un paquet d'erreur ou ICMP est reçu.

## Statut
- Lignes 1 à 70400 lues. Prochaine : ligne 70401 (suite EFI_MTFTP4_OPTION, EFI_MTFTP4_PACKET, GetInfo codes, ParseOptions, ReadFile...).

## Lignes 70401-70700 — EFI_MTFTP4_OPTION (OptionStr, ValueStr : chaînes ASCII NUL), EFI_MTFTP4_PACKET (union, pack(1) : OpCode, Rrq/Wrq, Oack, Data, Ack, Data8 et Ack8 = réservés à ignorer, Error), EFI_MTFTP4_REQ_HEADER (OpCode, Filename), EFI_MTFTP4_OACK_HEADER (OpCode + options), EFI_MTFTP4_DATA_HEADER (OpCode, Block 16 bits, Data), EFI_MTFTP4_ACK_HEADER (OpCode, Block), EFI_MTFTP4_DATA8_HEADER / ACK8_HEADER (Block 64 bits, réservés), EFI_MTFTP4_ERROR_HEADER (OpCode, ErrorCode, ErrorMessage) ; tous les paquets sont en network byte order ; utilisés aussi par ReadFile, WriteFile, ReadDirectory et les callbacks de contrôle de paquet ; Table 30.18 décrit les champs.
- [AGENT] Projet : TFTP hors périmètre. Règle notable : les champs Data8/Ack8 doivent être ignorés (réservés).

## Statut
- Lignes 1 à 70700 lues. Prochaine : ligne 70701 (suite Table 30.18 : opcodes, codes d'erreur ; GetInfo codes ; ParseOptions...).

## Lignes 70701-71000 — Suite Table 30.18 (Block, Data, ErrorCode, ErrorMessage ; champs Data8/Ack8 réservés), opcodes MTFTP4 (RRQ 1, WRQ 2, DATA 3, ACK 4, ERROR 5, OACK 6, DIR 7, DATA8 8 et ACK8 9 réservés), codes d'erreur MTFTP4 (NOT_DEFINED 0, FILE_NOT_FOUND 1, ACCESS_VIOLATION 2, DISK_FULL 3, ILLEGAL_OPERATION 4, UNKNOWN_TRANSFER_ID 5, FILE_ALREADY_EXISTS 6, NO_SUCH_USER 7, REQUEST_DENIED 8 = refus lié à la négociation d'options), codes GetInfo (SUCCESS = OACK dans Packet ; ERROR ou ICMP -> Packet NULL pour ICMP, paquet ERROR pour MTFTP ; EFI_TFTP_ERROR ; EFI_PROTOCOL_ERROR ; TIMEOUT si aucune réponse), 30.3.6 ParseOptions MTFTP4 (PacketLen, Packet, OptionCount, OptionList ; PROTOCOL_ERROR si option invalide et liste arrêtée à la dernière valide ; FreePool sur OptionList), 30.3.7 ReadFile MTFTP4 (début : Token : Status, Event, OverrideData, Filename, ModeStr, OptionCount, OptionList, BufferSize, Buffer, Context, CheckPacket ; téléchargement vers un buffer fixe ou une fonction CheckPacket, les deux peuvent être utilisés : CheckPacket appelé en premier)
- [AGENT] Projet : TFTP hors périmètre. Règle notable : un paquet d'erreur MTFTP est remonté à l'appelant, jamais écarté silencieusement.

## Statut
- Lignes 1 à 71000 lues. Prochaine : ligne 71001 (suite EFI_MTFTP4_TOKEN : CheckPacket, EFI_MTFTP4_CHECK_PACKET, codes ReadFile, WriteFile, ReadDirectory, Poll).

## Lignes 71001-71300 — EFI_MTFTP4_TOKEN (fin : TimeoutCallback, PacketNeeded ; champs : Status, Event NOTIFY_SIGNAL TPL <= CALLBACK, OverrideData, Filename, ModeStr 'octet' par défaut, OptionCount/OptionList (options inconnues transmises telles quelles), BufferSize (entrée/sortie), Buffer, Context, CheckPacket), codes ReadFile (SUCCESS, BUFFER_TOO_SMALL, ABORTED, ICMP variés, TIMEOUT, TFTP_ERROR, DEVICE_ERROR, NO_MEDIA), EFI_MTFTP4_CHECK_PACKET (intercepte DATA/DATA8 en ReadFile et OACK/ERROR en ReadFile/WriteFile/ReadDirectory ; tout statut différent de SUCCESS abandonne le transfert), EFI_MTFTP4_TIMEOUT_CALLBACK (appelé à chaque timeout ; statut != SUCCESS abandonne), EFI_MTFTP4_PACKET_NEEDED (WriteFile sans buffer ; Length 0 = fin de session ; statut != SUCCESS = abandon), codes PacketNeeded, 30.3.8 WriteFile MTFTP4 (peut être UNSUPPORTED ; options non supportées ignorées -> TFTP standard ; Buffer et PacketNeeded ne peuvent pas être utilisés ensemble, le callback est ignoré si buffer fourni), 30.3.9 ReadDirectory MTFTP4 (début : liste de fichiers en chaînes ASCII NUL ; avec multicast : champ ip:ip:ip:ip:port ; nom de fichier ; chaîne d'info filesize yyyy-mm-dd hh:mm:ss:ffff en UTC)
- [AGENT] Projet : TFTP hors périmètre. Règle notable : un callback qui renvoie un statut autre que SUCCESS arrête le transfert, il ne faut pas l'ignorer.

## Statut
- Lignes 1 à 71300 lues. Prochaine : ligne 71301 (suite 30.3.9 ReadDirectory codes ; 30.3.10 Poll MTFTP4 ; 30.4 MTFTPv6...).

## Lignes 71301-71600 — 30.3.10 Poll MTFTP4 (codes ; EFI_TIMEOUT possible), 30.4 EFI MTFTPv6 (construit sur UDP6 ; GUID service binding d9760ff3-3cca-4267-...; GUID EFI_MTFTP6_PROTOCOL bf0a78ba-ec29-49cf-...), 30.4.2.1 interface MTFTP6 (GetModeData, Configure, GetInfo, ParseOptions, ReadFile, WriteFile, ReadDirectory, Poll), 30.4.2.2 GetModeData MTFTP6 (EFI_MTFTP6_MODE_DATA : ConfigData, SupportedOptions en lecture seule, ne pas libérer), EFI_MTFTP6_CONFIG_DATA (StationIp, LocalPort, ServerIp, InitialServerPort 69 par défaut, TryCount, TimeoutValue en secondes), 30.4.2.3 Configure MTFTP6 (codes : StationIp doit être une adresse configurée ou zéro ; LocalPort déjà utilisé = INVALID ; ACCESS_DENIED si opération en cours), 30.4.2.4 GetInfo MTFTP6 (début : même logique que MTFTP4 ; EFI_MTFTP_OVERRIDE_DATA IPv6 : ServerIp, ServerPort, TryCount, TimeoutValue)
- [AGENT] Projet : TFTP hors périmètre.

## Statut
- Lignes 1 à 71600 lues. Prochaine : ligne 71601 (suite EFI_MTFTP_OVERRIDE_DATA IPv6, EFI_MTFTP6_OPTION, EFI_MTFTP6_PACKET, GetInfo codes, ParseOptions, ReadFile MTFTP6).

## Lignes 71601-71900 — EFI_MTFTP6_OVERRIDE_DATA (fin : ServerIp, ServerPort, TryCount, TimeoutValue ; 0/zéro = valeur de Configure), EFI_MTFTP6_OPTION (OptionStr, ValueStr), EFI_MTFTP6_PACKET (union pack(1) : identique à MTFTP4 ; Data8/Ack8 réservés), EFI_MTFTP6_REQ_HEADER, OACK_HEADER, DATA_HEADER (Block 16 bits), ACK_HEADER, DATA8_HEADER / ACK8_HEADER (Block 64 bits, réservés), EFI_MTFTP6_ERROR_HEADER (OpCode, ErrorCode, ErrorMessage) ; paquets en network byte order ; utilisés par ReadFile, WriteFile, ReadDirectory et callbacks ; Table 30.30 décrit les champs (début).
- [AGENT] Projet : TFTP hors périmètre. Règle notable : la structure MTFTP6 est identique à MTFTP4 hors adresses IPv6.

## Statut
- Lignes 1 à 71900 lues. Prochaine : ligne 71901 (suite Table 30.30 : opcodes, codes d'erreur MTFTP6 ; GetInfo codes ; ParseOptions ; ReadFile MTFTP6...).

## Lignes 71901-72200 — Suite Table 30.30 (Block, Data, ErrorCode, ErrorMessage des paquets MTFTP6), opcodes MTFTP6 (lignes 71980-71980), Table 30.31 (descriptions), codes d'erreur MTFTP6 (Table 30.32 : NOT_DEFINED 0, FILE_NOT_FOUND 1, ACCESS_VIOLATION 2, DISK_FULL 3, ILLEGAL_OPERATION 4, UNKNOWN_TRANSFER_ID 5, FILE_ALREADY_EXISTS 6, NO_SUCH_USER 7, REQUEST_DENIED 8), codes GetInfo MTFTP6 (SUCCESS OACK ; TFTP_ERROR ; ICMP variés -> Packet NULL ; TIMEOUT), 30.4.2.5 ParseOptions MTFTP6 (PacketLen, Packet, OptionCount, OptionList : pointeurs dans le paquet lui-même ; FreePool de la liste seulement ; PROTOCOL_ERROR à la première option invalide), 30.4.2.6 ReadFile MTFTP6 (début : Token ; même modèle que MTFTP4 : Buffer et/ou CheckPacket, CheckPacket appelé en premier)
- [AGENT] ANOMALIE DU TEXTE DE LA SPEC (ligne 71980) : `EFI_MTFTP6_OPCODE_ACK` est défini à 6 alors que `EFI_MTFTP6_OPCODE_OACK` vaut aussi 6 ; la valeur correcte est ACK = 4 (RFC 1350), OACK = 6 (RFC 2347), comme dans MTFTP4 (ligne 70762). Ne pas coder ACK = 6. Les lignes 71980 utilisent aussi le préfixe EFI_MTFTP4_OPCODE_DATA8 / ACK8 au lieu de MTFTP6 : traiter comme réservés.
- [AGENT] Projet : TFTP hors périmètre.

## Statut
- Lignes 1 à 72200 lues. Prochaine : ligne 72201 (suite EFI_MTFTP6_TOKEN, ReadFile MTFTP6 codes, WriteFile, ReadDirectory, Poll MTFTP6, puis chapitre 31 Redfish).

## Lignes 72201-72500 — EFI_MTFTP6_TOKEN (Status, Event NOTIFY_SIGNAL, OverrideData (non pointeur ici, structure inline), Filename, ModeStr, OptionCount/OptionList, BufferSize, Buffer, Context, CheckPacket, TimeoutCallback, PacketNeeded), codes Status ReadFile MTFTP6 (SUCCESS, OUT_OF_RESOURCES, BUFFER_TOO_SMALL, ABORTED, ICMP, TIMEOUT, TFTP_ERROR, DEVICE_ERROR), EFI_MTFTP6_CHECK_PACKET (même logique que MTFTP4 ; statut != SUCCESS arrête), EFI_MTFTP6_TIMEOUT_CALLBACK (même logique), EFI_MTFTP6_PACKET_NEEDED (Length 0 = fin ; statut != SUCCESS = arrêt), codes ReadFile MTFTP6, 30.4.2.7 WriteFile MTFTP6 (même logique : buffer et callback mutuellement exclusifs, callback ignoré si buffer fourni), 30.4.2.8 ReadDirectory MTFTP6 (début)
- [AGENT] Projet : TFTP hors périmètre. Remarque : dans l'EFI_MTFTP6_TOKEN, OverrideData est une structure, pas un pointeur, contrairement à MTFTP4 (pointeur). À ne pas confondre dans le code.

## Statut
- Lignes 1 à 72500 lues. Prochaine : ligne 72501 (suite 30.4.2.8 ReadDirectory MTFTP6 codes ; 30.4.2.9 Poll MTFTP6 ; puis chapitre 31 Redfish).

## Lignes 72501-72800 — 30.4.2.8 ReadDirectory MTFTP6 (fin : liste en chaînes ASCII NUL ; ip:ip:ip:ip:port si multicast ; info filesize yyyy-mm-dd hh:mm:ss:ffff UTC ; codes), 30.4.2.9 Poll MTFTP6 (codes). FIN DU CHAPITRE 30 (UDP/MTFTP) à la ligne 72547. CHAPITRE 31 : EFI Redfish Service Support. 31.1 EFI Redfish Discover Protocol : rôle (découvrir des services Redfish via SMBIOS type 42 Redfish Host Interface, ou SSDP/UPnP M-SEARCH UDP 1900 ; crée un REST EX par service ; s'appuie sur UDP et REST EX), 31.1.2 driver (Supported : REST EX SB, UDP4 SB ou UDP6 SB ; Start : crée un child par service binding, ouvert BY_DRIVER ; Stop : arrête la découverte et détruit les children), 31.1.3 client (deux scénarios : unicast M-SEARCH vers service SMBIOS 42 ; multicast M-SEARCH via interfaces réseau), 31.1.4 EFI_REDFISH_DISCOVER_PROTOCOL (GUID 5db12509-4550-4347-...; GetNetworkInterfaceList, AcquireRedfishService, AbortAcquireRedfishService, ReleaseRedfishService), 31.1.4.1 GetNetworkInterfaceList (EFI_REDFISH_DISCOVER_NETWORK_INTERFACE : MacAddress, IsIpv6, SubnetId, SubnetPrefixLength, VlanId ; mémoire libérée par appelant)
- 31.1.4.2 AcquireRedfishService : TargetNetworkInterface NULL = toutes ; Flags (EFI_REDFISH_DISCOVER_FLAG) ; Token EFI_REDFISH_DISCOVERED_TOKEN (Signature 'RFTS', DiscoveredList, Event TPL_CALLBACK, Timeout en secondes : 1 à 5 en asynchrone ; 0 = synchrone) ; EFI_REDFISH_DISCOVERED_LIST (NumberOfServiceFound, RedfishInstances, alloués par le protocole, libérés par ReleaseRedfishService) ; drapeaux : HOST_INTERFACE 0x1, SSDP 0x2, SSDP_UDP6 0x4, KEEP_ALIVE 0x8 (M-SEARCH périodique, durée 2^n s avec n entre 3 et 15, 8 s à 2^15 s), RENEW 0x10 (renotifier même les services connus), VALIDATION 0x80000000 (unicast M-SEARCH pour vérifier un service SMBIOS 42), DURATION_MASK 0x0F000000 ; INVALID_PARAMETER si ni HOST_INTERFACE ni SSDP ; UDP enfant détruit après découverte ; REST EX child créé puis fermé après configuration ; le client ouvre le REST EX via RedfishRestExHandle.
- [AGENT] Projet : Redfish hors périmètre (le PC n'est pas un client Redfish). Règle notable : le client doit fermer le REST EX dès qu'il ne communique plus, sinon fuite d'instance.

## Statut
- Lignes 1 à 72800 lues. Prochaine : ligne 72801 (suite 31.1.4.2 : EFI_REDFISH_DISCOVERED_INSTANCE, EFI_REDFISH_DISCOVERED_INFORMATION, ReleaseRedfishService, AbortAcquire...).

## Lignes 72801-73100 — 31.1.4.2 fin : EFI_REDFISH_DISCOVERED_INSTANCE (Status, Information valide seulement si SUCCESS), EFI_REDFISH_DISCOVERED_INFORMATION (RedfishRestExHandle déjà configuré, IsIPv6, RedfishHostIpAddress, RedfishVersion 16 bits : octet haut = majeure, bas = mineure (0x100 = 1.0), Location, Uuid (USN), Os, OsVersion, Product, ProductVersion (issus de l'en-tête Server UPnP), UseHttps), codes AcquireRedfishService (Timeout > 5 s = INVALID ; Token->Event NULL = INVALID ; DiscoveredList non vide en entrée = INVALID), 31.1.4.3 AbortAcquireRedfishService (annule SSDP et unicast UDP, ferme et détruit UDP4/UDP6, ferme REST EX), 31.1.4.4 ReleaseRedfishService (libère la mémoire et détruit les REST EX créés ; la liste peut être quelconque), 31.1.5 exemples (processus de découverte ; configuration des interfaces : statique avec au moins IP et masque, ou DHCP ; l'ifconfig du shell EFI doit être relancé à chaque boot sauf persistance), 31.2 EFI Redfish JSON Structure Converter (un convertisseur par type de ressource Redfish : AccountService.v1_0_0, AttributeRegistry.v1_2_0, EthernetInterface.v1_4_0 ; livré en source ou binaire avec header C ; enregistré via EFI_REST_JSON_STRUCTURE_PROTOCOL ; identifiant de type = Namespace + Datatype ; si JsonRsrcIdentifier non NULL, le convertisseur suit l'identifiant et non "odata.type" ; si NULL, il lit "odata.type" ; tous les membres de structure en pointeurs pour convertir seulement les champs non NULL), 31.2.2 usage du convertisseur (inclure le header C ; NULL = le consommateur doit vérifier l'identifiant renvoyé avant d'utiliser la structure)
- [AGENT] Projet : Redfish hors périmètre. Règle notable : le convertisseur doit suivre l'identifiant fourni et non le JSON reçu, sinon la structure ne correspond pas à ce que le consommateur attend.
- CHAPITRE 32 : SECURE BOOT AND DRIVER SIGNING commence à la ligne 73052. 32.1 Secure Boot : EFI_AUTHENTICATION_INFO_PROTOCOL (GUID 7671d9d0-53db-4173-...; Get, Set ; associe des informations d'authentification à un device path ; un Set écrase l'ancien, qui n'est plus récupérable), 32.1.2 Get (début).

## Statut
- Lignes 1 à 73100 lues. Prochaine : ligne 73101 (suite 32.1.2 Get : codes ; 32.1.3 Set ; autres protocoles secure boot / signature de pilotes).

## Lignes 73101-73400 — 32.1.2 Get authentication (ControllerHandle, Buffer alloué par la fonction, libéré par appelant ; SUCCESS / INVALID / DEVICE_ERROR), 32.1.3 Set (écrase l'entrée existante ; UNSUPPORTED si la politique plateforme l'interdit), 32.1.4-32.1.5 nœuds d'authentification (structure variable : GUID de type 16 octets, Length 2 octets, données ; packed, tout champ à traiter comme non aligné ; traversée possible même si type inconnu grâce au Length), 32.1.6 nœud CHAP RADIUS (GUID d6062b50-15ca-11da-...; adresses RADIUS, NAS, secrets, noms CHAP et Reverse CHAP ; table 32.3 ; longueur totale 58+P+Q+R+S+T), 32.1.7 nœud CHAP local (GUID c280c73e-15ca-11da-...; User Secret/Name, CHAP Secret/Name, Reverse ; table 32.4), 32.2 UEFI Driver Signing (vérifier qu'un exécutable vient d'une source autorisée ; signature numérique : hash puis chiffrement par clé privée ; vérif par clé publique), 32.2.2 signatures embarquées (dans le PE/COFF : 5e entrée du Data Directory = pointeur vers la table des certificats ; normalement ajoutés à la fin du fichier), 32.2.3 digest d'image (renvoi à Microsoft Authenticode), 32.2.4 définitions (32.2.4.1 WIN_CERTIFICATE : dwLength 32 bits, wRevision 16 bits, wCertificateType 16 bits, bCertificate)
- [AGENT] Projet : secure boot touche le binaire agent (hwa2_agent_corrige.efi). Point utile : si le binaire est modifié (ajout de section ou de données), la table de certificats en fin de fichier et l'entrée 5 du Data Directory doivent être recalculées ; un binaire modifié invalide sa signature. Ne pas signer ni modifier la PK/KEK/db sans accord explicite (règle de projet).
- [AGENT] Hors périmètre actuel : Authentication Node CHAP (pas de réseau dans le projet).

## Statut
- Lignes 1 à 73400 lues. Prochaine : ligne 73401 (suite 32.2.4.1 WIN_CERTIFICATE : CERTIFICATE_TYPE, EFI_CERT_BLOCK, WIN_CERTIFICATE_UEFI_GUID, EFI_CERT_TYPE, signatures PKCS7 / X.509 ...).

## Lignes 73401-73700 — 32.2.4.1 WIN_CERTIFICATE (fin : wRevision 0x0200, wCertificateType : PKCS_SIGNED_DATA 0x0002 (Authenticode), EFI_PKCS115 0x0EF0, EFI_GUID 0x0EF1 ; réservé 0x0EF0-0x0EFF ; table 32.5 : quels certificats vérifient quelle entrée de base de signature ; cas PKCS7 : match possible à n'importe quel niveau de la chaîne X.509, et hash du PE possible), 32.2.4.2 WIN_CERTIFICATE_EFI_PKCS1_15 (RSASSA-PKCS1-v1_5 RFC 2437 §8-9 ; HashAlgorithm GUID ; Signature de la taille de la clé : 256 octets pour RSA 2048), 32.2.4.3 WIN_CERTIFICATE_UEFI_GUID (CertType GUID : RSA2048_SHA256 a7717414-c616-4977-... ou PKCS7 4aafd29d-68df-49ee-... ; EFI_CERT_BLOCK_RSA_2048_SHA256 : HashType, PublicKey 256 octets exposant 0x10001, Signature 256 octets PKCS1 v1.5 ; PKCS7 : DER SignedData RFC 2315), 32.3 Key Exchange (PK, KEK, modes : Setup (SetupMode=1), User (SetupMode=0), Audit (AuditMode=1, PK effacé, DeployedMode=0), Deployed (DeployedMode=1 en lecture seule) ; transitions vers Setup/Audit peuvent demander un reset : EFI_WARN_RESET_REQUIRED ; ALREADY_STARTED si transition déjà en file), 32.3.1 enrôlement PK (en user mode, nouveau PK signé par PK courant ; setup -> user immédiat), 32.3.2 effacement PK (signé par PKpriv courant ; SetupMode repasse à 1), 32.3.3 Audit Mode, 32.3.4 Deployed Mode (depuis User par DeployedMode=1 sans reset ; depuis Audit par PK), 32.3.5 enrôlement KEK (APPEND_WRITE ou réécriture complète ; signature PK en user mode), 32.3.6 exigences de stockage (PK : non volatile résistant à la falsification et à l'effacement ; KEK : résistant à la falsification ; attention au management out-of-band), 32.4 passage de clés publiques OS vers firmware (signature : EFI_SIGNATURE_DATA SignatureOwner GUID + données ; EFI_SIGNATURE_LIST : SignatureType, SignatureListSize, SignatureHeaderSize, SignatureSize, puis entrées)
- [AGENT] Projet : Secure Boot : ne rien écrire dans PK/KEK/db/dbx/SetupMode/AuditMode/DeployedMode sans accord explicite (règle projet : pas d'écriture firmware ou NVRAM). Ce passage est de la lecture de spec uniquement.

## Statut
- Lignes 1 à 73700 lues. Prochaine : ligne 73701 (suite EFI_SIGNATURE_LIST, types de signatures EFI_CERT_*, Signature Database, puis variables globales).

## Lignes 73701-74000 — 32.4.1 Signature Database (fin : EFI_SIGNATURE_LIST : SignatureListSize, SignatureHeaderSize, SignatureSize ; chaque signature a SignatureOwner GUID ; la base est une variable UEFI ; liste de types : SHA256 c1c41626-..., RSA2048 3c5766e8-... (module seulement, exposant 0x10001, big-endian), RSA2048_SHA256 e2b36190-..., SHA1 826ca512-..., RSA2048_SHA1 67f8444f-..., X509 a5c059a1-94e4-4aa7-... (un certificat par EFI_SIGNATURE_LIST), SHA224 0b6e5233-..., SHA384 ff3e5307-..., SHA512 093e0fae-..., X509_SHA256 3bd2a492-... (ToBeSignedHash + TimeOfRevocation, 0 = toujours révoqué), X509_SHA384 7076876e-..., X509_SHA512 446dbf63-..., SM3 57347f87-..., X509_SM3 60d807e5-..., EXTERNAL_MANAGEMENT 452e8ced-... (pseudo-signature : n'autorise aucune écriture, sauf gestion hors bande), 32.4.2 Image Execution Information Table (AuditMode 0 : image refusée et décrite dans la table si non autorisée ou interdite ; AuditMode 1 : une entrée par certificat et par EFI_CERT_SHAxxx ; Action : AUTH_UNTESTED 0, AUTH_SIG_FAILED 1, AUTH_SIG_PASSED 2, AUTH_SIG_NOT_FOUND 3, AUTH_SIG_FOUND 4 (dans la liste interdite), POLICY_FAILED 5, INITIALIZED 0x8 ; EFI_IMAGE_EXECUTION_INFO : Action, InfoSize, Name, DevicePath, Signature ; table : NumberOfImages + tableau ; un OS legacy en User Mode échoue avec AUTH_UNTESTED), 32.5 UEFI Image Validation (vue d'ensemble : base autorisée et base interdite consultées par le Boot Manager avant de démarrer une image ; horodatage RFC 3161 si X509_SHA* ; règles de révocation par TimeOfRevocation)
- [AGENT] Projet : écriture bases PK/KEK/db/dbx hors périmètre (règle projet). Point utile pour le binaire agent : si une image non autorisée est refusée, elle apparaît dans la table d'exécution ; la lecture de cette table est de la lecture seule et reste possible.

## Statut
- Lignes 1 à 74000 lues. Prochaine : ligne 74001 (suite 32.5.2 Authorized User : message d'autorisation utilisateur, puis les sections 32.5.x de validation d'image).

## Lignes 74001-74300 — 32.5.2 Authorized User (détient KEKpriv), 32.5.3 Signature Database Update (db, dbx, dbt, dbr = variables authentifiées GUID d719b2cb-3d3a-4596-... ; écriture en user mode avec KEKpriv ou PKpriv, ou en setup mode ; dbDefault/dbxDefault/... fournies par vendeur ; EFI_OUT_OF_RESOURCES = l'agent peut supprimer une entrée qui ne lui appartient pas seulement si elle est à lui ; procédure en 11 étapes : GetVariable, créer ou étendre, signer, SetVariable, gérer OUT_OF_RESOURCES), 32.5.3.1 Image Execution Information Table (informations collectées : hash, nom, device path, action), 32.5.3.2 politique firmware (image en flash, device sécurisé, autre signature), 32.5.3.3 processus d'autorisation en 11 étapes (reset, validation de la base avant images signées, validation d'image, démarrage ou refus, choix Yes/No/Defer, ajout à db, option suivante, table pour l'OS, validation OS, ajout à db), règles de rejet de l'image (dbx : hash SHA256 de l'image ; X509_SHA* du TBS de la chaîne ; X509 avec issuer + serial + TBS ; une seule signature valide suffit dans db si aucune n'est dans dbx), 32.6 Device Authentication (devdb ; variable L"devAuthBoot" ; SPDM GET_CERTIFICATE, CHALLENGE, KEY_EXCHANGE ; en cas d'échec le bus ignore le périphérique ; ne vérifie pas le firmware du périphérique), 32.6.2-32.6.3 (PKpriv pour devdb), 32.7 Code (32.7.1 : GUID db et noms db/dbx/dbt/dbr ; flag APPEND_WRITE obligatoire ; stockage tamper-resistant ; 32.7.2 : GUID b9c2b4f4-bf5f-462d-... nom devdb)
- [AGENT] Projet : aucune écriture dans db/dbx/dbt/dbr/devdb/PK/KEK (règle projet). Point utile : la règle de rejet par dbx (hash SHA256 du binaire) signifie qu'une modification du binaire agent peut changer son hash et donc son admission ; à vérifier avant toute tentative de démarrage.
- CHAPITRE 33 : Human Interface Infrastructure (HII) commence à la ligne 74125. 33.1 Goals (gestion des entrées utilisateur : polices, chaînes, formulaires, IFR, XML pour configuration, support écran local/distant/web), 33.2 design (drivers et applications installent images, polices, chaînes, formulaires dans la base HII ; le navigateur de formulaires les rend et renvoie les changements ; localisation : formats date, nombres, ponctuation ; entrée : clavier et souris ; 33.2.1.x composants).

## Statut
- Lignes 1 à 74300 lues. Prochaine : ligne 74301 (suite 33.2.3 User Input, 33.2.x, puis 33.3 HII Database...).

## Lignes 74301-74600 — 33.2.3 User Input (clavier : touches communes seulement, F1-F12, chiffres, flèches, pavé numérique, Enter/Space/Tab/Esc, modificateurs, NumLock ; mots de passe : seule exception aux touches spécifiques, car jamais affichés), 33.2.4 Keyboard Layout (EFI_KEY, mapping à partir d'ISO 9995 ; six éléments par touche : nom, Unicode, shifted, AltGr, shifted AltGr, modifier ; valeur 0xFFFF = exception ; EFI_LEFT_CONTROL_MODIFIER ; touches mortes EFI_NS_KEY_MODIFIER suivies de EFI_NS_KEY_DEPENDENCY_MODIFIER, définition incomplète = erreur), 33.2.5 Forms (formulaires dans la base HII en IFR ; navigateur : parcourt, collecte, sauve dans variable globale ou privée ; processeurs de formulaires peuvent valider sans UI), 33.2.5.1 Form Sets (GUID d'identification, classes, titre, aide, image ; parent = premier formulaire visible ; DISABLE_IF et SUPPRESS_IF), 33.2.5.2 Forms (identifiant 16 bits, titre, modal, GRAY_OUT_IF, DISABLE_IF évalué à l'init seulement), 33.2.5.3 Statements (texte statique, sous-titre, bouton reset ; visibilité réévaluée à chaque changement de valeur), 33.2.5.4 Questions (identifiant 16 bits, valeurs par défaut et fabrication, stockage, refresh, validation, callback ; valeur Undefined = question masquée ; lecture : EFI_IFR_READ puis stockage ; écriture : EFI_IFR_WRITE puis stockage ; types : action (sans stockage), boolean (1 octet 0/1), date (EFI_HII_DATE, option SetTime), number (1/2/4/8 octets, radix, min, max, skip ; contrôle de débordement par le processeur), set/ordered-list (un octet par conteneur, minuscules pour hex), string (NUL-terminée, PASSWORD masqué), cross-reference (EFI_HII_REF), time (EFI_HII_TIME ; SetTime/SetWakeupTime))
- [AGENT] Projet : HII/interface utilisateur hors périmètre de l'agent (pas de saisie d'écran de configuration, et règle projet : aucune modification de configuration firmware sans accord). Retenu : la gestion du clavier brut se fait au niveau USB/HID, pas ici.

## Statut
- Lignes 1 à 74600 lues. Prochaine : ligne 74601 (suite 33.2.5.5 Options, syntaxe, 33.2.6 ... HII string, fonts, images, IFR).

## Lignes 74601-74900 — 33.2.5.5 Options (fin : visibilité ; SUPPRESS_IF évalué à chaque affichage), 33.2.5.6 Storage (quatre types : Buffer (un champ par offset, via EFI_HII_CONFIG_ACCESS), Name/Value (chaîne par question), EFI Variable (GetVariable/SetVariable sur tout le buffer), Date/Time (GetTime/SetTime, GetWakeupTime/SetWakeupTime) ; sans stockage, la question doit avoir un callback RETRIEVE/CHANGED ou un READ/WRITE IFR ; tableau 33.2 de provenance des informations de stockage), 33.2.5.7 Expressions (RPN, 64 bits ; types Boolean, UINT64, String, Buffer, Undefined ; pas de conversion implicite ; EFI_IFR_RULE 0-255), 33.2.5.8 Defaults (trois magasins standard : Standard 0x0000, Manufacturing 0x0001, Safe 0x0002 ; plages : 0x0000-0x3FFF UEFI, 0x4000-0x7FFF plateforme, 0x8000-0xBFFF matériel, 0xC000-0xFFFF firmware ; priorité de la valeur par défaut : callback, ALTCFG, EFI_IFR_DEFAULT, option par défaut, booléen), 33.2.5.9 Validation (édition, niveau question : longueurs, débordement de Number, INCONSISTENT_IF erreur, WARNING_IF avertissement ; niveau formulaire : NO_SUBMIT_IF), 33.2.5.10 Forms Processing (erreurs : opcode inconnu = ignoré ; opcode mal formé = formulaire désactivé ; opcode étendu = ignorer les données en trop ; formset mal formé = ignoré en entier ; bits réservés ignorés), 33.2.5.11 Forms Editing (verrouillage IFR_LOCKED irréversible, renumérotation lors d'un déplacement), 33.2.5.12 privilèges de sécurité (EFI_IFR_SECURITY ; EFI_USER_PROFILE_CHANGED_EVENT_GUID), 33.2.6 Strings (UCS-2 ; langues RFC 4646/4647 ; polices indicatives ; zone privée Unicode interdite ; surrogates interdits)
- [AGENT] Projet : HII hors périmètre. Remarque : la règle "malformé = désactivé entier" est pertinente pour tout parseur de structures binaires que l'agent pourrait écrire : rejeter la structure entière, pas la deviner.

## Statut
- Lignes 1 à 74900 lues. Prochaine : ligne 74901 (suite 33.2.6.2.4 Common Control Codes, fonts, images, 33.3 HII Database Protocol...).

## Lignes 74901-75200 — 33.2.6.2.4 Common Control Codes (table 33.3 : Font Family Select 0x7F 0x00 / 0xF7zz ; Font Size Select 0x7F 0x01 / 0xF8zz en demi-points ; Bold/Italic/Underline/Emboss/Shadow/DblUnderline ON/OFF = 0x20..0x2B / 0xF620..0xF62B), 33.2.6.2.5 Line Breaks (espaces = points de coupure ; NBSP 00A0 et NNBSP 202F = espace sans coupure ; ZWSP 200B ; tirets 058A 2010 2012 2013 ...; EM DASH 2014 ; NBHY 2011 non coupant ; coupures obligatoires 000A 000C 000D 2028 2029 ; table 33.10), 33.2.7 Fonts (police 'system' obligatoire : fixe, 8x19 ou 16x19 ; Simplified Font Package ou Font Package ; attributs : nom ('sysdefault' pour la police standard), taille (hauteur de cellule en points, 19 pour la standard), style ; EFI_HII_FONT_GLYPH_GENERATOR_PROTOCOL pour générer les glyphes manquants), 33.2.7.3 fixed font (baseline, cap_height, x_height, descender, ascender ; 16x19 stocké colonne gauche puis droite ; table 33.11 : 8x19 baseline 15 / cap 12 / x 8 / desc 3 / asc 4 ; 16x19 : 14 / 11 / 7 / 4 / 4), 33.2.7.4 proportional (Width, Height, Offset_X, Offset_Y, Advance ; baseline = max(Height + Offset_Y) ; origine suivante = Origin_X + Advance), 33.2.8 Images (1, 4, 8, 24, 32 bits ; B,G,R ; 32 bits aligné DWORD ; conversion palette ; écran non TrueColor : palette globale 256 entrées, composantes 6 bits sur certains matériels), 33.2.9 HII Database (dépôt de formulaires, chaînes, images, polices), 33.2.10 Forms Browser (navigation, formulaires actifs/inactifs ; FORM_OPEN/FORM_CLOSE ; Exit/Discard All = EFI_BROWSER_ACTION_REQUEST_EXIT)
- [AGENT] Projet : HII/polices/images hors périmètre de l'agent. Aucune action.

## Statut
- Lignes 1 à 75200 lues. Prochaine : ligne 75201 (suite 33.2.10.1 : comportement du navigateur, sauvegarde, 33.3 HII Protocols...).

## Lignes 75201-75500 — 33.2.10.1 Forms Browser (fin : Exit/Submit All ; Default ; comportements actif/sélectionné/question : Change, Commit, Discard ; un formulaire est actif entre FORM_OPEN et FORM_CLOSE ; questions modifiées si stockage existant et valeur différente), 33.2.11 Configuration Settings (sauvegarde seulement sur demande explicite, sauf batch, mot de passe, date/heure ; drivers lisent les réglages au boot ; 33.2.11.1 OS Runtime : ExportPackageLists + ExportConfig, pointeurs dans la table de configuration UEFI, mise à jour par variables (SetVariable) ou par UpdateCapsule), 33.2.11.2 UEFI Configuration Language (tables de correspondance pour un consommateur standard), 33.2.12 Form Callback, 33.2.13 Driver Model Interaction (Configuration Access Protocol fourni par le contrôleur ; Supported/Start/Stop), 33.2.14-33.2.15 Standards Map Forms (correspondance entre questions HII et autres méthodes : DMTF CLP, WMI, SNMP ; trois cas : Value Shift, One-To-Many, Many-To-One ; équations avec EFI_IFR_GET/READ/WRITE/SET/MAP ; table de vérité 33.12-33.13), 33.3 Code Definitions (formats binaires des paquets : Font, Simplified Font, String, Image, Device Path, Keyboard Layout, GUID, Forms), 33.3.1 EFI_HII_PACKAGE_HEADER (Length 24 bits, Type 8 bits, Data ; liste terminée par EFI_HII_PACKAGE_END ; TYPE_GUID = paquets fournisseur ; types ALL 0x00, GUID 0x01, FORMS ...)
- [AGENT] Projet : HII hors périmètre. Remarque retenue : un en-tête de paquet ne mesure que 24 bits de longueur (max 16 Mo) ; l'agent ne doit pas supposer une longueur plus grande.

## Statut
- Lignes 1 à 75500 lues. Prochaine : ligne 75501 (suite types de paquets HII : FORMS, STRINGS, FONTS, IMAGES, SIMPLE_FONTS, DEVICE_PATH, KEYBOARD_LAYOUT ...).

## Lignes 75501-75800 — Table 33.15 types de paquets HII (FORMS 0x02, STRINGS 0x04, FONTS 0x05, IMAGES 0x06, SIMPLE_FONTS 0x07, DEVICE_PATH 0x08, KEYBOARD_LAYOUT 0x09, ANIMATIONS 0x0A, END 0xDF, réservés système 0xE0-0xFF ; ALL 0x00 = pseudo-type à l'export ; GUID 0x01 = format défini par GUID qui suit l'en-tête), 33.3.1.2 EFI_HII_PACKAGE_LIST_HEADER (PackageListGuid, PackageLength incluant l'en-tête ; même GUID = mêmes données ; une nouvelle version = nouveau GUID), 33.3.2 Simplified Font Package (8x19 narrow et 16x19 wide ; 33.3.2.1 EFI_HII_SIMPLE_FONT_PACKAGE_HDR : NumberOfNarrowGlyphs, NumberOfWideGlyphs, tableaux triés par code Unicode pour recherche binaire ; offset WideGlyphs = offset NarrowGlyphs + taille * nombre), 33.3.2.2 EFI_NARROW_GLYPH (UnicodeWeight CHAR16, Attributes : NON_SPACING 0x01, WIDE 0x02 ; EFI_GLYPH_HEIGHT 19, WIDTH 8 ; GlyphCol1 en colonnes), 33.3.2.3 EFI_WIDE_GLYPH (GlyphCol1 + GlyphCol2, Pad[3] à zéro, taille = 2 x narrow), 33.3.3 Font Package (en-tête fixe : Header, HdrSize, GlyphBlockOffset, Cell (largeur/hauteur max, OffsetX/OffsetY, AdvanceX), FontStyle (BOLD 0x1, ITALIC 0x2, EMBOSS 0x10000, OUTLINE 0x20000, SHADOW 0x40000, UNDERLINE 0x80000, DBL_UNDER 0x100000), FontFamily NUL ; 33.3.3.2 blocs de glyphes : 1 octet de type puis corps ; traités dans l'ordre)
- [AGENT] Projet : HII hors périmètre. Remarque : les blocs de glyphes doivent être traités dans l'ordre ; tableaux triés pour recherche binaire (un parseur qui suppose l'ordre doit vérifier).

## Statut
- Lignes 1 à 75800 lues. Prochaine : ligne 75801 (suite types de blocs de glyphes, EFI_HII_GLYPH_INFO, EFI_HII_FONT_STYLE, Glyph Block types ...).

## Lignes 75801-76100 — 33.3.3.2 Glyph information (blocs de glyphes ; CharValueCurrent démarre à 1 ; bloc de type inconnu = ignorer s'il a une longueur, sinon arrêt ; table des types : END 0x00, GLYPH 0x10, GLYPHS 0x11, GLYPH_DEFAULT 0x12, GLYPHS_DEFAULT 0x13, GLYPH_VARIABILITY 0x14, DUPLICATE 0x20, SKIP2 0x21, SKIP1 0x22, DEFAULTS 0x23, EXT1 0x30, EXT2 0x31, EXT4 0x32), EFI_HII_GLYPH_INFO (Width, Height, OffsetX, OffsetY, AdvanceX : UINT16 pour taille, INT16 pour décalages), 33.3.3.2.1 DEFAULTS (change la cellule par défaut jusqu'au prochain DEFAULTS ; avant le premier, cellule de l'en-tête), 33.3.3.2.2 DUPLICATE (même glyphe pour CharValue ; incrémente CharValueCurrent), 33.3.3.2.3 END (glyphes >= CharValueCurrent sont vides), 33.3.3.2.4 EXT1/EXT2/EXT4 (blocs d'extension avec longueur pour pouvoir les sauter), 33.3.3.2.5 GLYPH (bitmap d'un glyphe ; taille = ((Width+7)/8) * Height octets ; bits gauche-droite, haut-bas ; incrémente CharValueCurrent)
- [AGENT] Projet : HII hors périmètre. Anomalie de spec notée : la description du bloc GLYPH cite un membre `GlyphCount` absent de la structure ; à ignorer.
- [AGENT] Règle générale retenue pour tout parseur de blocs : un type inconnu doit être sauté via sa longueur, sans quoi arrêter le parsing.

## Statut
- Lignes 1 à 76100 lues. Prochaine : ligne 76101 (suite 33.3.3.2.6 GLYPHS, GLYPH_DEFAULT, GLYPHS_DEFAULT, VARIABILITY, puis Font Package fin, Image Package...).

## Lignes 76101-76400 — 33.3.3.2.6 GLYPHS (Count glyphes à cellule identique, bitmaps de même taille ; CharValueCurrent += Count), 33.3.3.2.7 GLYPH_DEFAULT (cellule par défaut de l'en-tête ou du dernier DEFAULTS), 33.3.3.2.8 GLYPHS_DEFAULT (Count UINT16 + bitmaps), 33.3.3.2.9 SKIP2 (UINT16) / SKIP1 (UINT8) : incrémentent CharValueCurrent sans glyphe, 33.3.3.2.10 GLYPH_VARIABILITY (0x14 : GlyphPackInBits 1, 2, 4, 8, 16, 24 ou 32 bits par pixel ; format décodé uniquement par EFI_HII_FONT_GLYPH_GENERATOR_PROTOCOL ; la longueur du bloc dépend de ce champ), 33.3.4 Device Path Package (chemin du handle driver, créé par NewPackageList), 33.3.5 GUID Package (type 0x01 : Guid puis données libres), 33.3.6 String Package (en-tête : HdrSize, StringInfoOffset, LanguageWindow[16] CHAR16, LanguageName (StringId), Language ASCII ; UEFI_CONFIG_LANG "x-UEFI" et "x-i-UEFI" réservés à la configuration), 33.3.6.2 String Information (blocs traités dans l'ordre à partir de StringIdCurrent = 1 ; types : string, duplicate, font, skip ; fin par SIBT_END 0x00 ; SIBT_STRING_SCSU 0x10 ...)
- [AGENT] Projet : HII hors périmètre. Remarque : GLYPH_VARIABILITY ne se décode pas sans le générateur : un parseur qui n'a pas le générateur doit sauter le bloc par sa longueur ou arrêter.

## Statut
- Lignes 1 à 76400 lues. Prochaine : ligne 76401 (suite types SIBT : SCSU, SCSU_FONT, UCS2, FONT, DUPLICATE, SKIP ; String Package fin ; Image Package...).

## Lignes 76401-76700 — Table 33.17 types SIBT (suite : STRING_SCSU_FONT 0x11, STRINGS_SCSU 0x12, STRINGS_SCSU_FONT 0x13, STRING_UCS2 0x14, STRING_UCS2_FONT 0x15, STRINGS_UCS2 0x16, STRINGS_UCS2_FONT 0x17, DUPLICATE 0x20, SKIP2 0x21, SKIP1 0x22, EXT1 0x30, EXT2 0x31, EXT4 0x32, FONT 0x40), 33.3.6.2.1 SIBT_DUPLICATE (StringId d'une chaîne au texte identique ; incrémente StringIdCurrent), 33.3.6.2.2 SIBT_END (chaînes >= StringIdCurrent vides ; le texte de la spec mentionne BlockType2 = FONT pour END : erreur de recopie), 33.3.6.2.3 EXT1/EXT2/EXT4 (longueur pour sauter), 33.3.6.2.4 SIBT_FONT (FontId 8 bits unique dans le paquet, FontSize en pixels, FontStyle, FontName CHAR16 NUL ; FontId 0 = police par défaut), 33.3.6.2.5 SKIP1 (UINT8), 33.3.6.2.6 SKIP2 (UINT16), 33.3.6.2.7 SIBT_STRING_SCSU (début : chaîne compressée SCSU, décrite dans l'algorithme de décodage des figures 33.40-33.42)
- [AGENT] Projet : HII hors périmètre. Anomalie de spec notée : la description de SIBT_END référence Header.BlockType2 = EFI_HII_SIBT_FONT (copie erronée, END n'a pas d'extension).

## Statut
- Lignes 1 à 76700 lues. Prochaine : ligne 76701 (suite SIBT_STRING_SCSU, SCSU, UCS2 strings, Image Package, Keyboard Layout Package, Forms Package...).

## Lignes 76701-77000 — 33.3.6.2.7 fin : SIBT_STRING_SCSU (texte NUL en SCSU, police par défaut), 33.3.6.2.8 SCSU_FONT (FontIdentifier 0 ou ID de SIBT_FONT ; écarts = code de contrôle explicite), 33.3.6.2.9 STRINGS_SCSU (StringCount UINT16 + chaînes NUL ; ids séquentiels), 33.3.6.2.10 STRINGS_SCSU_FONT, 33.3.6.2.11 STRING_UCS2 (CHAR16 NUL), 33.3.6.2.12 STRING_UCS2_FONT, 33.3.6.2.13 STRINGS_UCS2 (StringCount + chaînes), 33.3.6.2.14 STRINGS_UCS2_FONT, 33.3.6.3 Encodage (SCSU étendu : 0x7F introduit les codes de contrôle ; 0x7F s'encode 0x01 0x7F ; fenêtres par défaut dans l'en-tête ; caractères 0xF000-0xFCFF rejetés ; UCS-2 : même rejet), 33.3.7 Image Package (début : en-tête ImageInfoOffset et PaletteInfoOffset ; 0 = absent ; 33.3.7.2 blocs d'image traités dans l'ordre)
- [AGENT] Projet : HII hors périmètre. Règle notable : rejeter les caractères 0xF000-0xFCFF (zone d'usage privé, interdite par UEFI).

## Statut
- Lignes 1 à 77000 lues. Prochaine : ligne 77001 (suite Image Package : table des types de blocs image, EFI_HII_IMAGE_BLOCK, palettes ; puis Keyboard Layout Package, Forms Package...).

## Lignes 77001-77300 — Table 33.18 types de blocs image (END 0x00, IMAGE_1BIT 0x10, 1BIT_TRANS 0x11, 4BIT 0x12, 4BIT_TRANS 0x13, 8BIT 0x14, 8BIT_TRANS 0x15, 24BIT 0x16, 24BIT_TRANS 0x17, JPEG 0x18, PNG 0x19, DUPLICATE 0x20, SKIP2 0x21, SKIP1 0x22, EXT1 0x30, EXT2 0x31, EXT4 0x32) ; ImageIdCurrent démarre à 1 ; bloc inconnu = sauter ou arrêter ; 33.3.7.2.1 IIBT_END (images >= ImageIdCurrent vides ; la description cite BlockType2, copie erronée), 33.3.7.2.2 EXT1/2/4 (Length inclut l'en-tête), 33.3.7.2.3 IMAGE_1BIT (Width, Height, Bitmap ((Width+7)/8)*Height octets, bit de poids fort = pixel en haut à gauche ; PaletteIndex ; exemple donné), 33.3.7.2.4 IMAGE_1BIT_TRANS (pixel 0 = transparent, 1 = couleur de palette), 33.3.7.2.5 IMAGE_24BIT (Width, Height, pixels EFI_HII_RGB_PIXEL b,g,r ; taille = Width*3*Height), 33.3.7.2.6 24BIT_TRANS (début)
- [AGENT] Projet : HII/images hors périmètre. Règle notable : taille de bitmap à recalculer depuis Width/Height (avec arrondi à l'octet pour 1 bit) avant toute lecture.

## Statut
- Lignes 1 à 77300 lues. Prochaine : ligne 77301 (suite IMAGE_24BIT_TRANS, 4BIT, 8BIT, JPEG, PNG, DUPLICATE, SKIP, palette info...).

## Lignes 77301-77600 — 33.3.7.2.6 IMAGE_24BIT_TRANS (0x17 : pixel 0x00 0x00 0x00 = transparent ; pour un vrai noir, utiliser 0x00 0x00 0x01), 33.3.7.2.7 IMAGE_4BIT (0x12 : PaletteIndex, bitmap (Width+1)/2 * Height octets ; nibble de poids fort = pixel en haut à gauche), 33.3.7.2.8 4BIT_TRANS (0x13 : 0 = transparent), 33.3.7.2.9 IMAGE_8BIT (0x14 : Width*Height octets, premier octet = pixel en haut à gauche), 33.3.7.2.10 8BIT_TRANS (0x15 : 0 = transparent), 33.3.7.2.x DUPLICATE (0x20 : ImageId déjà défini, ImageIdCurrent += 1), 33.3.7.2.11 JPEG (0x18 : Size UINT32 puis données JPEG avec signature JFIF à l'offset 6 ; décodeur requis : JPEG baseline couleur RGB, 1:1:1 et 4:1:1 ; pas de composante gris), 33.3.7.2.12 SKIP1 (0x22, UINT8), 33.3.7.2.13 SKIP2 (0x21, UINT16)
- [AGENT] Projet : HII/images hors périmètre. Anomalies de spec notées : artefact "%%%%%" après le titre EFI_HII_IIBT_DUPLICATE ; la section DUPLICATE parle de ImageValueCurrent au lieu de ImageIdCurrent ; la structure 4BIT/8BIT déclare EFI_HII-IIBT (tiret) : coquille.

## Statut
- Lignes 1 à 77600 lues. Prochaine : ligne 77601 (suite SKIP2 IIBT, palettes EFI_HII_IMAGE_PALETTE_INFO, puis Keyboard Layout Package et Forms Package).

## Lignes 77601-77900 — 33.3.7.2.14 PNG (0x19 : Size UINT32 + données PNG brutes), 33.3.7.3 Palette Information (EFI_HII_IMAGE_PALETTE_INFO_HEADER : PaletteCount UINT16 ; palettes numérotées 1..n ; chaque palette : PaletteSize + tableau EFI_HII_RGB_PIXEL b,g,r ; entrée 0 = couleur 0 de l'image ; palette minimale, index contigus à partir de 0), 33.3.8 Forms Package (IFR : suite d'opcodes ; en-tête EFI_IFR_OP_HEADER ; 33.3.8.1 binaire : OpCode 8 bits, Length 7 bits (2 à 127 octets y compris en-tête), Scope 1 bit, END ferme la portée ; 33.3.8.2.1 EFI_IFR_OP_HEADER ; types EFI_QUESTION_ID, IMAGE_ID, STRING_ID, FORM_ID, VARSTORE_ID, ANIMATION_ID = UINT16 ; 33.3.8.2.2 EFI_IFR_QUESTION_HEADER (QuestionId 0 réservé, VarStoreId 0 = pas de stockage, VarStoreInfo = VarOffset ou VarName selon stockage ; flags : READ_ONLY 0x01, CALLBACK 0x04, RESET_REQUIRED 0x10, REST_STYLE 0x20, RECONNECT_REQUIRED 0x40, OPTIONS_ONLY 0x80) ; 33.3.8.2.3 EFI_IFR_STATEMENT_HEADER (Prompt, Help ; 0 = absent) ; 33.3.8.3 table des opcodes IFR (FORM 0x01, SUBTITLE 0x02, TEXT 0x03, IMAGE 0x04, ONE_OF 0x05, CHECKBOX 0x06, NUMERIC 0x07, PASSWORD 0x08, ONE_OF_OPTION 0x09, SUPPRESS_IF 0x0A, LOCKED 0x0B, ACTION 0x0C, RESET_BUTTON 0x0D, FORM_SET 0x0E, REF 0x0F, NO_SUBMIT_IF 0x10, INCONSISTENT_IF 0x11, EQ_ID_VAL 0x12, EQ_ID_ID 0x13, EQ_ID_VAL_LIST 0x14, AND 0x15, OR 0x16, NOT 0x17, RULE 0x18, GRAY_OUT_IF 0x19, DATE 0x1A, TIME 0x1B, STRING 0x1C, REFRESH 0x1D, DISABLE_IF 0x1E, ANIMATION 0x1F, TO_LOWER 0x20, TO_UPPER 0x21, MAP 0x22 (le texte liste un ordre différent : ne pas se fier à la numérotation de la liste, consulter la table 33.20 avant codage), ORDERED_LIST, VARSTORE, VARSTORE_NAME_VALUE, VARSTORE_EFI, VARSTORE_DEVICE, VERSION, END, MATCH, GET, SET, READ, WRITE, EQUAL, NOT_EQUAL, GREATER_THAN)
- [AGENT] Projet : HII/IFR hors périmètre. Remarque : la table 33.20 donne des valeurs d'opcodes qui ne suivent pas toujours l'ordre de la liste de noms (les numéros sont en mise en page éclatée). Pour tout codage futur, relire les valeurs dans un en-tête officiel EDK2 et non dans cette liste.

## Statut
- Lignes 1 à 77900 lues. Prochaine : ligne 77901 (suite table 33.20 des opcodes IFR avec valeurs 0x23 et suivantes, puis les définitions des opcodes).

## Lignes 77901-78200 — Suite table 33.20 (opcodes 0x23-0x64 : FORM_SET 0x0E ... ; ADD 0x3A ; BITWISE_AND 0x35, BITWISE_OR 0x36, BITWISE_NOT 0x37 ; CATENATE 0x5E ; MATCH2 0x64 ; WARNING_IF 0x63 ; GUID 0x5F ; SECURITY 0x60 ; MODAL_TAG 0x61 ; REFRESH_ID 0x62 ; VALUE 0x58, DEFAULT 0x59, DEFAULTSTORE 0x5A, FORM_MAP 0x5B, SPAN 0x5C? et THIS 0x5D?, les numéros exacts sont en table 33.20), 33.3.8.3.1 ACTION (0x0C : Question + QuestionConfig ; config vide = callback seul ; read-only = non sélectionnable), 33.3.8.3.2 ANIMATION (0x1F : Id), 33.3.8.3.3 ADD (0x3A : dépile droite puis gauche ; non entiers = Undefined ; étend en 64 bits ; overflow ignoré), 33.3.8.3.4 AND (0x15 : booléens ; non booléens = Undefined), 33.3.8.3.5 BITWISE_AND (0x35), 33.3.8.3.6 BITWISE_NOT (0x37), 33.3.8.3.7 BITWISE_OR (0x36), 33.3.8.3.8 CATENATE (0x5E : deux chaînes sans NUL final, ou deux buffers ; types différents = Undefined), 33.3.8.3.9 CHECKBOX (0x06 : Question + Flags UINT8)
- [AGENT] Projet : IFR hors périmètre. Règle notable pour un interpréteur d'expressions : arithmétique en 64 bits, débordement ignoré (pas d'erreur), types incompatibles = Undefined et non erreur silencieuse.

## Statut
- Lignes 1 à 78200 lues. Prochaine : ligne 78201 (suite 33.3.8.3.9 CHECKBOX, puis DATE, DEFAULT, DEFAULTSTORE, DISABLE_IF, DIVIDE, DUP, ...).

## Lignes 78201-78500 — 33.3.8.3.9 CHECKBOX (fin : Flags, EFI_IFR_CHECKBOX_DEFAULT 0x01, DEFAULT_MFG 0x02 ; défauts par priorité Flags < OPTION < DEFAULT), 33.3.8.3.10 CONDITIONAL (0x50 : dépile droite, milieu, gauche ; gauche non booléen = Undefined ; gauche TRUE -> droite, sinon milieu), 33.3.8.3.11 DATE (0x1A : Flags, EFI_QF_DATE_YEAR_SUPPRESS 0x01, MONTH 0x02, DAY 0x04, STORAGE 0x30 avec NORMAL 0x00, TIME 0x10, WAKEUP 0x20), 33.3.8.3.12 DEFAULT (0x5B selon la section ; DEFAULT et DEFAULT_2 : DefaultId, Type, Value ou VALUE imbriquée), 33.3.8.3.13 DEFAULTSTORE (0x5C : DefaultName string, DefaultId ; doit précéder toute référence DEFAULT), 33.3.8.3.14 DISABLE_IF (0x1E : expression en premier objet imbriqué ; sous FORM_SET = constantes seulement ; sous FORM = peut lire les questions du même formulaire), 33.3.8.3.15 DIVIDE (0x3D : diviseur 0 = Undefined ; 64 bits), 33.3.8.3.16 DUP (0x57 : duplique le sommet de pile ; optimisation des outils), 33.3.8.3.17 END (0x29 : fin de portée), 33.3.8.3.18 EQUAL (0x2F : chaînes, booléens, entiers ; types différents = Undefined ; chaînes comparées lexicographiquement), 33.3.8.3.19 EQ_ID_ID (0x13 : deux QuestionId ; non comparables = Undefined), 33.3.8.3.20 EQ_ID_VAL_LIST (0x14, début)
- [AGENT] Projet : IFR hors périmètre. Anomalie de spec notée : les valeurs d'opcodes des sections (DEFAULT 0x5B, DEFAULTSTORE 0x5C) ne concordent pas avec la table 33.20 (qui les situe autour de 0x59-0x5A) ; ne pas coder ces valeurs sans les vérifier dans un en-tête officiel. Règle utile : une division par zéro produit Undefined, pas une faute silencieuse.

## Statut
- Lignes 1 à 78500 lues. Prochaine : ligne 78501 (suite EQ_ID_VAL_LIST, EQ_ID_VAL, EXTRA opcodes, FALSE, FIND, FORM, FORM_MAP, FORM_SET, FORM_SET... 33.3.8.3.x).

## Lignes 78501-78800 — 33.3.8.3.20 EQ_ID_VAL_LIST (fin : ListLength + ValueList, non convertible = Undefined), 33.3.8.3.21 EQ_ID_VAL (0x12 : QuestionId + UINT16 Value), 33.3.8.3.22 FALSE (0x47), 33.3.8.3.23 FIND (0x4C : Format 0x00 sensible à la casse, 0x01 insensible ; dépile droite (index), milieu (motif), gauche (chaîne) ; index >= longueur ou non trouvé = 0xFFFFFFFFFFFFFFFF), 33.3.8.3.24 FORM (0x01 : FormId, FormTitle), 33.3.8.3.25 FORM_MAP (0x5D : FormId + liste Methods (MethodTitle StringId + MethodIdentifier GUID) ; GUID EFI_HII_STANDARD_FORM_GUID 3bd2f4ec-e524-46e4-... = équivalent à FORM), 33.3.8.3.26 FORM_SET (0x0E : Guid, FormSetTitle, Help, Flags (bits 0-1 = nombre de ClassGuid 0 à 4 ; bits 2-7 réservés à 0), ClassGuid[]), 33.3.8.3.27 GET (0x2B : VarStoreId, VarStoreInfo (VarOffset ou VarName), VarStoreType ; pousse la valeur ou Undefined)
- [AGENT] Projet : IFR/HII hors périmètre. Règle utile : une recherche sans résultat renvoie un sentinelle explicite (0xFFFFFFFFFFFFFFFF), pas une valeur plausible comme 0.

## Statut
- Lignes 1 à 78800 lues. Prochaine : ligne 78801 (suite GET VarStoreType table, GRAY_OUT_IF, GREATER_EQUAL, GREATER_THAN, GUID, IMAGE, INCONSISTENT_IF, LENGTH, LESS_EQUAL...).

## Lignes 78801-79100 — Table 33.21 VarStoreType (NUM_SIZE_8/16/32/64, BOOLEAN, TIME, DATE, STRING, OTHER, ACTION, UNDEFINED, BUFFER, REF), 33.3.8.3.28 GRAY_OUT_IF (0x19 : nidifié, expression en premier objet, END si plusieurs opcodes ; non sélectionnable et visuellement différent), 33.3.8.3.29 GREATER_EQUAL (0x32), 33.3.8.3.30 GREATER_THAN (0x31) : chaînes, booléens, entiers ; types différents = Undefined, 33.3.8.3.31 GUID (0x5F : opcode extensible ; outil qui ne le comprend pas doit le sauter), 33.3.8.3.32 IMAGE (0x04 : Id image HII), 33.3.8.3.33 INCONSISTENT_IF (0x011 : Error StringId ; évalué à chaque passage de question ; l'utilisateur ne doit pas pouvoir soumettre), 33.3.8.3.34 LENGTH (0x56 : buffer en octets, chaîne en caractères, sinon Undefined), 33.3.8.3.35 LESS_EQUAL (0x34), 33.3.8.3.36 LESS_THAN (0x33), 33.3.8.3.37 LOCKED (0x0B), 33.3.8.3.38 MAP (0x22 : pop une valeur, compare aux match, pousse le return correspondant, sinon Undefined), 33.3.8.3.39 MATCH (0x2A : motif via EFI_UNICODE_COLLATION2_PROTOCOL.MetaiMatch ; le protocole doit être présent), 33.3.8.3.40 MID (0x4B : chaîne ou buffer ; index 0-based ; longueur 0 ou index hors borne = vide ; extraction tronquée à la fin), 33.3.8.3.41 MODAL_TAG (0x61 : formulaire modal ; désactivé seulement par navigation ou callback RESET/SUBMIT/EXIT/FORM_SUBMIT_EXIT/FORM_DISCARD_EXIT)
- [AGENT] Projet : IFR hors périmètre. Remarque : MID a des règles de bornes précises (tronquer, vide si hors borne) à reproduire à l'identique si un jour un parseur est écrit.

## Statut
- Lignes 1 à 79100 lues. Prochaine : ligne 79101 (suite MODAL_TAG, NOT, NOT_EQUAL, NUMERIC, ONE, ONE_OF, ONE_OF_OPTION, ONES, ORDERED_LIST, OR, PASSWORD, QUESTION_REF ...).

## Lignes 79101-79400 — 33.3.8.3.41 MODAL_TAG (fin : un formulaire modal ne se ferme pas par Navigate Forms, Discard All, Submit All, ni Reset Platform hors callback), 33.3.8.3.42 MODULO (0x3E : diviseur 0 = Undefined ; reste), 33.3.8.3.43 MULTIPLY (0x3C : 64 bits, bits faibles), 33.3.8.3.44 NOT (0x17 : booléen, non booléen = Undefined), 33.3.8.3.45 NOT_EQUAL (0x30 : mêmes règles que EQUAL), 33.3.8.3.46 NO_SUBMIT_IF (0x10 : Error StringId ; bloque la soumission si vrai), 33.3.8.3.47 NUMERIC (0x07 : Question, Flags (NUMERIC_SIZE masque 0x03 : 1 octet = 0x00, 2 = 0x01, 4 = 0x02, 8 = 0x03 ; DISPLAY 0x30 : INT_DEC 0x00, UINT_DEC 0x10, UINT_HEX 0x20), MinValue, MaxValue, Step (Step 0 = saisie libre) ; avec INT_DEC le contrôle min/max se fait en signé alors que les expressions restent non signées), 33.3.8.3.48 ONE (0x53), 33.3.8.3.49 ONES (0x54 : 0xFFFFFFFFFFFFFFFF), 33.3.8.3.50 ONE_OF (0x05 : même structure que NUMERIC, équivalent), 33.3.8.3.51 ONE_OF_OPTION (0x09 : Option StringId, Flags, Type, Value)
- [AGENT] Projet : IFR hors périmètre. Règle utile : quand un type signé est affiché mais que les expressions sont non signées, la validation doit suivre le drapeau d'affichage (signé) et non la représentation interne.

## Statut
- Lignes 1 à 79400 lues. Prochaine : ligne 79401 (suite ONE_OF_OPTION : Flags/Type/Value, OR, ORDERED_LIST, PASSWORD, QUESTION_REF1/2/3, REF, REFRESH, REFRESH_ID, RULE, RULE_REF, SECURITY, SET, SPAN, STRING, STRING_REF...).

## Lignes 79401-79700 — 33.3.8.3.51 ONE_OF_OPTION (fin : EFI_IFR_TYPE_VALUE : NUM_SIZE_8 0x00, 16 0x01, 32 0x02, 64 0x03, BOOLEAN 0x04, TIME 0x05, DATE 0x06, STRING 0x07, OTHER 0x08 (valeur par VALUE imbriquée), UNDEFINED 0x09, ACTION 0x0A, BUFFER 0x0B, REF 0x0C ; EFI_HII_TIME (H,M,S), EFI_HII_DATE (Year 16 bits, Month, Day), EFI_HII_REF (QuestionId, FormId, FormSetGuid, DevicePath) ; OPTION_DEFAULT 0x10, OPTION_DEFAULT_MFG 0x20), 33.3.8.3.52 OR (0x16 : non booléen = Undefined), 33.3.8.3.53 ORDERED_LIST (0x23 : MaxContainers, Flags ; UNIQUE_SET 0x01, NO_EMPTY_SET 0x02 ; valeur 0 = slot vide, donc les options valides ne doivent pas valoir 0), 33.3.8.3.54 PASSWORD (0x08 : MinSize, MaxSize ; deux modes : sans FLAG_CALLBACK le navigateur gère les comparaisons ; avec CALLBACK, poignée de main avec le driver ; organigrammes 33.48-33.49), 33.3.8.3.55 QUESTION_REF1 (0x40 : QuestionId ; Undefined si absente), 33.3.8.3.56 QUESTION_REF2 (0x41 : dépile un entier ; > 0xFFFF = Undefined), 33.3.8.3.57 QUESTION_REF3 (0x51 et variantes _2 et _3 avec DevicePath StringId et Guid du formset ; formset Nil avec plusieurs formsets sur le même device path = Undefined)
- [AGENT] Projet : HII hors périmètre. Remarque : la valeur 0 sert de marqueur de slot vide dans les listes ordonnées : un parseur doit traiter 0 comme 'vide' et non comme une valeur.

## Statut
- Lignes 1 à 79700 lues. Prochaine : ligne 79701 (suite EFI_IFR_READ, REF, REFRESH, REFRESH_ID, RULE, RULE_REF, SECURITY, SET, SPAN, STRING, STRING_REF1/2, SUBTITLE, SUPPRESS_IF, TEXT, THIS, TO_BOOLEAN, TO_LOWER, TO_STRING, TO_UINT, TO_UPPER, TOKEN, TRUE, UINT8..64, UNDEFINED, VALUE, VARSTORE..., VERSION, WARNING_IF, WRITE, ZERO...).

## Lignes 79701-80000 — 33.3.8.3.58 READ (0x2D : lit la valeur du stockage puis évalue l'expression ; lien vide si FormId et QuestionId nuls), 33.3.8.3.59 REF (0x0F : lien vers formulaire ou question ; variantes REF2 (QuestionId), REF3 (+ FormSetId GUID), REF4 (+ DevicePath StringId), REF5 (sans cible, cible via VALUE imbriquée) ; FormSetId nul = formset courant), 33.3.8.3.60 REFRESH (0x1D : RefreshInterval secondes ; 0 = pas de rafraîchissement auto), 33.3.8.3.61 REFRESH_ID (0x62 : RefreshEventGroupId GUID ; groupe d'événements créé à l'ouverture du formset et fermé à sa fermeture ; signal = rafraîchissement de la question ou du formulaire entier), 33.3.8.3.62 RESET_BUTTON (0x0D : Statement + DefaultId ; remet les défauts du formulaire ; FLAG_CALLBACK => callback du formset), 33.3.8.3.63 RULE (0x18 : RuleId 8 bits unique dans le formulaire ; doublon = formulaire invalide), 33.3.8.3.64 RULE_REF (0x3F : évalue la règle ; absente = Undefined), 33.3.8.3.65 SECURITY (0x60 : Permissions GUID ; pousse TRUE/FALSE selon le profil utilisateur), 33.3.8.3.66 SET (0x2C : VarStoreId, VarStoreInfo, VarStoreType ; début)
- [AGENT] Projet : HII hors périmètre. Règle utile : une règle en double ne doit pas être choisie silencieusement : le formulaire entier est marqué invalide.

## Statut
- Lignes 1 à 80000 lues. Prochaine : ligne 80001 (suite SET : VarStoreType, SETs, SPAN, STRING, STRING_REF1/2, SUBTITLE, SUPPRESS_IF, TEXT, THIS, TO_BOOLEAN, TO_LOWER, TO_STRING, TO_UINT, TO_UPPER, TOKEN, TRUE, UINT8/16/32/64, UNDEFINED, VALUE, VARSTORE*, VERSION, WARNING_IF, WRITE, ZERO).

## Lignes 80001-80300 — 33.3.8.3.66 SET (fin : dépile la valeur, l'écrit dans le stockage ; pousse TRUE si réussi, FALSE sinon), 33.3.8.3.67 SHIFT_LEFT (0x38 : décalage par nombre de bits ; non entiers = Undefined), 33.3.8.3.68 SHIFT_RIGHT (0x39), 33.3.8.3.69 SPAN (0x59 : Flags FIRST_MATCHING 0x00 / FIRST_NON_MATCHING 0x01 ; cherche le premier caractère dans des plages), 33.3.8.3.70 STRING (0x1C : MinSize, MaxSize UINT8, Flags STRING_MULTI_LINE 0x01 = indice seulement), 33.3.8.3.71 STRING_REF1 (0x4E : StringId ; chaîne vide si absente), 33.3.8.3.72 STRING_REF2 (0x4F : dépile un identifiant, > 0xFFFF = Undefined, chaîne vide si absente), 33.3.8.3.73 SUBTITLE (0x02 : Flags, FLAGS_HORIZONTAL 0x01 = indice seulement), 33.3.8.3.74 SUBTRACT (0x3B : soustraction 64 bits, bits faibles), 33.3.8.3.75 SUPPRESS_IF (0x0A : masque les objets imbriqués si vrai ; formulaire réévalué à chaque changement), 33.3.8.3.76 TEXT (0x03 : Statement, TextTwo ; début)
- [AGENT] Projet : HII hors périmètre. Remarque : SUBTRACT non signé sur 64 bits : une soustraction négative donne un grand entier (repli modulo 2^64), comportement à noter si un jour un interpréteur est écrit.

## Statut
- Lignes 1 à 80300 lues. Prochaine : ligne 80301 (suite TEXT, THIS, TO_BOOLEAN, TO_LOWER, TO_STRING, TO_UINT, TO_UPPER, TOKEN, TRUE, UINT8..64, UNDEFINED, VALUE, VARSTORE*, VERSION, WARNING_IF, WRITE, ZERO).

## Lignes 80301-80600 — 33.3.8.3.76 TEXT (fin : TextTwo), 33.3.8.3.77 THIS (0x58 : pousse la valeur de la question courante), 33.3.8.3.78 TIME (0x1B : Flags : HOUR_SUPPRESS 0x01, MINUTE 0x02, SECOND 0x04, STORAGE 0x30 (NORMAL 0x00, TIME 0x10, WAKEUP 0x20)), 33.3.8.3.79 TOKEN (0x4D : chaîne, délimiteurs, index 0-based ; index 0 = à gauche du premier délimiteur ; fin de chaîne = dernier délimiteur ; absent = vide), 33.3.8.3.80 TO_BOOLEAN (0x4A : entier 0 = FALSE, sinon TRUE ; chaîne "true"/"false" insensible à la casse, sinon Undefined ; buffer tout à zéro = FALSE), 33.3.8.3.81 TO_LOWER (0x20 : StrLwr de UNICODE_COLLATION2), 33.3.8.3.82 TO_STRING (0x49 : Format 0 décimal non signé, 1 décimal signé, 2 hex minuscule, 3 hex majuscule ; buffer : Format 0 = ASCII, 8 = UCS-2 ; s'arrête au NUL), 33.3.8.3.83 TO_UINT (0x48 : booléen 1/0 ; chaîne : espaces ignorés, préfixe 0x/0X = hex, '-' = entier signé ; buffer > 8 octets = Undefined, sinon étendu à 64 bits), 33.3.8.3.84 TO_UPPER (0x21 : StrUpr), 33.3.8.3.85 TRUE (0x46), 33.3.8.3.86 UINT8/16/32/64 (0x42, 0x43, 0x44, 0x45 : valeur étendue à 64 bits)
- [AGENT] Projet : HII hors périmètre. Règle utile : la conversion chaîne -> entier accepte '-' pour un signé mais le résultat reste non signé 64 bits ; conversion de buffer > 8 octets = Undefined (ne pas tronquer).

## Statut
- Lignes 1 à 80600 lues. Prochaine : ligne 80601 (suite UNDEFINED (0x53? voir table), VALUE, VARSTORE, VARSTORE_DEVICE, VARSTORE_EFI, VARSTORE_NAME_VALUE, VERSION, WARNING_IF, WRITE, ZERO, puis fin 33.3.8.3 et 33.3.9 Keyboard Layout Package...).

## Lignes 80601-80900 — 33.3.8.3.87 UNDEFINED (0x55 : pousse Undefined), 33.3.8.3.88 VALUE (0x5A : valeur sans stockage ; question en lecture seule), 33.3.8.3.89 VARSTORE (0x24 : Guid, VarStoreId (0 invalide), Size, Name ASCII ; stockage tampon linéaire), 33.3.8.3.90 VARSTORE_NAME_VALUE (0x25 : VarStoreId, Guid), 33.3.8.3.91 VARSTORE_EFI (0x26 : VarStoreId, Guid, Attributes UINT32, Size, Name ; GetVariable/SetVariable avec nom CHAR16), 33.3.8.3.92 VARSTORE_DEVICE (0x27 : DevicePath StringId ; si dans une question, prime sur le handle de NewPackageList), 33.3.8.3.93 VERSION (0x28 : 16 bits = majeure [15:8], dizaines [7:4], unités [3:0] de la révision de la table système), 33.3.8.3.94 WRITE (0x2E : évalué après avoir positionné THIS, avant écriture), 33.3.8.3.95 ZERO (0x52), 33.3.8.3.96 WARNING_IF (0x063 : Warning StringId, TimeOut secondes ; 0 = jusqu'à acquittement ; l'utilisateur peut soumettre après acquittement, contrairement à INCONSISTENT_IF), 33.3.8.3.97 MATCH2 (début : expression régulière)
- [AGENT] Projet : HII/IFR hors périmètre. Remarque : la même question peut lire une valeur via plusieurs stockages (VARSTORE, VARSTORE_EFI, VARSTORE_NAME_VALUE) : un seul identifiant de stockage doit être choisi et documenté.
- [AGENT] Anomalie de spec notée : EFI_IFR_WARNING_IF_OP est écrit 0x063 (octal) alors que tous les autres opcodes sont en hexadécimal ; la valeur hexadécimale est 0x63 (cohérent avec la table 33.20).

## Statut
- Lignes 1 à 80900 lues. Prochaine : ligne 80901 (suite MATCH2, puis 33.3.8.4 fin IFR, 33.3.9 Keyboard Layout Package, 33.3.10 Animation Package...).

## Lignes 80901-81200 — 33.3.8.3.97 MATCH2 (0x64 : SyntaxType GUID ; EFI_REGULAR_EXPRESSION_PROTOCOL : GetInfo / RegExSyntaxTypeList ; type absent = Undefined ; note : un pilote qui publie des formulaires doit vérifier les types à l'initialisation et peut installer sa propre instance), 33.3.9 Keyboard Package (EFI_HII_KEYBOARD_PACKAGE_HDR : en-tête + LayoutCount + Layout[]), 33.3.10 Animations Package (33.3.10.1 en-tête : AnimationInfoOffset ; 0 = pas de séquence ; 33.3.10.2 blocs : END 0x00, OVERLAY_IMAGES 0x10 (une fois, image superposée), CLEAR_IMAGES 0x11 (efface la fenêtre logique avant la suivante), RESTORE_SCRN 0x12 (restaure), OVERLAY_IMAGES_LOOP 0x18, CLEAR_IMAGES_LOOP 0x19, RESTORE_SCRN_LOOP 0x1A (en boucle), DUPLICATE 0x20, SKIP2 0x21, SKIP1 0x22, EXT1/2/4 0x30/0x31/0x32), 33.3.10.2.2 EXT (longueur explicite), 33.3.10.2.3 OVERLAY_IMAGES (DftImageId (0 = aucune image), Width, Height (fenêtre logique), CellCount, AnimationCell[]), EFI_HII_ANIMATION_CELL (OffsetX, OffsetY, ImageId, Delay en millisecondes ; 0 = pas de délai), 33.3.10.2.4 CLEAR_IMAGES (début : + BackgndColor EFI_HII_RGB_PIXEL)
- [AGENT] Projet : HII/animations hors périmètre. Règle utile : les images sont découpées dans la fenêtre logique ; un parseur doit borner chaque cellule à la fenêtre au lieu de dépasser.

## Statut
- Lignes 1 à 81200 lues. Prochaine : ligne 81201 (suite CLEAR_IMAGES, RESTORE_SCRN, LOOP, DUPLICATE, SKIP, puis fin des animations et 33.4 HII Protocols).

## Lignes 81201-81479 — 33.3.10.2.4 CLEAR_IMAGES (fin : BackgndColor, la fenêtre est effacée avant chaque image et avant DftImageId), 33.3.10.2.5 RESTORE_SCRN (0x12 : sauvegarde de la fenêtre avant la première image, restauration avant chaque suivante ; sans mémoire = CLEAR_IMAGES en noir), 33.3.10.2.6 OVERLAY_IMAGES_LOOP (0x18 : répète en boucle, superposition), 33.3.10.2.7 CLEAR_IMAGES_LOOP (0x19 : boucle + effacement), 33.3.10.2.8 RESTORE_SCRN_LOOP (0x1A : boucle + restauration ; anomalie de spec : la section utilise Length dans Members alors que la structure n'a pas de champ Length), 33.3.10.2.9 DUPLICATE (AnimationId de même séquence), 33.3.10.2.10 SKIP1 (UINT8), 33.3.10.2.11 SKIP2 (UINT16) ; fin du chapitre 33 à 81479. CHAPITRE 34 : HII PROTOCOLS commence à 81479. 34.1 Font Protocol : EFI_HII_FONT_PROTOCOL (GUID e9ca4775-8657-47fc-...) ; interfaces de conversion chaîne vers image et rendu.
- [AGENT] Projet : HII/animations hors périmètre. Remarque : la ligne Members de EFI_HII_AIBT_RESTORE_SCRN_LOOP est une coquille (mention de Length absente de la structure) ; ne pas coder un champ Length qui n'existe pas.

## Statut
- Lignes 1 à 81500 lues. Prochaine : ligne 81501 (suite 34.1 Font Protocol : EFI_HII_FONT_PROTOCOL.StringToImage, StringIdToImage, GetGlyph, GetFontInfo ...).

## Lignes 81501-81800 — 34.1 EFI_HII_FONT_PROTOCOL (GUID e9ca4775-8657-47fc-...; StringToImage, StringIdToImage, GetGlyph, GetFontInfo), 34.1.2 StringToImage (Flags EFI_HII_OUT_FLAG_CLIP 0x01, WRAP 0x02, CLIP_CLEAN_Y 0x04, CLIP_CLEAN_X 0x08, TRANSPARENT 0x10, IGNORE_IF_NO_GLYPH 0x20, IGNORE_LINE_BREAK 0x40, DIRECT_TO_SCREEN 0x80 ; Blt non NULL = dessin sur image existante avec CLIP implicite ; Blt NULL = allocation par la fonction, libérée par appelant ; RowInfoArray : StartIndex, EndIndex, LineHeight, LineWidth, BaselineOffset ; ColumnInfoArray : ~0 pour non imprimables ; WRAP ne se combine pas avec CLIP_CLEAN_X ; TRANSPARENT interdit si Blt NULL ; glyphe inconnu remplacé par U+FFFD sauf IGNORE_IF_NO_GLYPH), 34.1.3 StringIdToImage (même logique avec PackageList + StringId + Language ; NOT_FOUND si l'identifiant ou la liste absente), 34.1.4 GetGlyph (Char -> bitmap ; WARN_UNKNOWN_GLYPH si remplacé par U+FFFD ; Blt doit valoir NULL à l'entrée), 34.1.5 GetFontInfo (FontHandle itérateur ; NULL = premier ; StringInfoOut alloué par AllocatePool ; NOT_FOUND si aucune police), 34.2 EFI HII Font Ex (GUID 849e6875-db35-4df8-...; étend les appels précédents : si un glyphe manque, utilise la première instance EFI_HII_FONT_GLYPH_GENERATOR_PROTOCOL qui supporte la police demandée ; GetGlyphInfo ajouté), 34.2.2 StringToImageEx, 34.2.3 StringIdToImageEx (début)
- [AGENT] Projet : HII/rendu de polices hors périmètre. Règle utile : un ColumnInfoArray plus court que la chaîne ou un Blt NULL avec TRANSPARENT sont des erreurs de paramètre à rejeter, pas à corriger en silence.

## Statut
- Lignes 1 à 81800 lues. Prochaine : ligne 81801 (suite StringIdToImageEx, GetGlyphEx, GetFontInfoEx, GetGlyphInfo, puis 34.3 String protocol...).

## Lignes 81801-82100 — 34.2.3 StringIdToImageEx (fin), 34.2.4 GetGlyphEx, 34.2.5 GetFontInfoEx (même logique que la version non Ex), 34.2.6 GetGlyphInfo (géométrie seule, sans bitmap ; UNSUPPORTED si aucune source ; NOT_FOUND ; NULL = INVALID), 34.2.7.1 EFI_FONT_DISPLAY_INFO (ForegroundColor, BackgroundColor, FontInfoMask, FontInfo ; masques : SYS_FONT 0x01, SYS_SIZE 0x02, SYS_STYLE 0x04, SYS_FORE_COLOR 0x10, SYS_BACK_COLOR 0x20, RESIZE 0x1000, RESTYLE 0x2000, ANY_FONT 0x10000, ANY_SIZE 0x20000, ANY_STYLE 0x40000 ; combinaisons interdites : SYS_FONT/ANY_FONT, SYS_SIZE/ANY_SIZE ou RESIZE, SYS_STYLE/ANY_STYLE ou RESTYLE), 34.2.7.2 EFI_IMAGE_OUTPUT (Width, Height, Bitmap ou Screen = union), 34.3 EFI_HII_STRING_PROTOCOL (GUID 0fd96974-23aa-4cdc-...; NewString, GetString, SetString, GetLanguages, GetSecondaryLanguages), 34.3.2 NewString (PackageList, Language RFC 4646, LanguageName facultatif, String, StringFontInfo ; StringId unique dans la liste et réservé à toutes les langues ; EFI_FONT_INFO : FontStyle, FontSize en pixels, FontName), 34.3.3 GetString (langue : RFC 4647 pour la recherche ; NOT_FOUND si id absent ; INVALID_LANGUAGE si id présent sans cette langue ; BUFFER_TOO_SMALL met à jour StringSize ; StringFontInfo alloué par la fonction et libéré par appelant)
- [AGENT] Projet : HII hors périmètre. Règle utile : distinguer 'identifiant absent' (NOT_FOUND) de 'langue absente' (INVALID_LANGUAGE) ; un appelant doit traiter les deux cas séparément, sans fallback silencieux.

## Statut
- Lignes 1 à 82100 lues. Prochaine : ligne 82101 (suite GetString, SetString, GetLanguages, GetSecondaryLanguages, puis 34.4 Image protocol...).

## Lignes 82101-82400 — 34.3.4 SetString (StringFontInfo NULL = police inchangée ; on ne peut pas changer la police sans changer le texte), 34.3.5 GetLanguages (liste ASCII des langues ; BUFFER_TOO_SMALL met à jour la taille), 34.3.6 GetSecondaryLanguages (PrimaryLanguage absente = INVALID_LANGUAGE ; liste NULL si aucune), 34.4 EFI_HII_IMAGE_PROTOCOL (GUID 31a6406a-6bdf-4e46-...; NewImage, GetImage, SetImage, DrawImage, DrawImageId), 34.4.2 NewImage (EFI_IMAGE_ID UINT16 unique dans la liste ; EFI_IMAGE_INPUT : Flags (EFI_IMAGE_TRANSPARENT 0x01), Width, Height, Bitmap en EFI_GRAPHICS_OUTPUT_BLT_PIXEL, taille = Width*Height*4), 34.4.3 GetImage (le bitmap retourné ne doit pas être libéré ni modifié par l'appelant), 34.4.4 SetImage (NOT_FOUND ou INVALID si Image NULL), 34.4.5 DrawImage (EFI_HII_DRAW_FLAG_CLIP ; FORCE_TRANS ; FORCE_OPAQUE ; DEFAULT suit EFI_IMAGE_TRANSPARENT ; pas de transparence si Blt NULL), 34.4.6 DrawImageId (même logique par identifiant)
- [AGENT] Projet : HII/images hors périmètre. Règle utile : un bitmap retourné par GetImage est en lecture seule pour l'appelant ; toute modification doit passer par SetImage (copie).

## Statut
- Lignes 1 à 82400 lues. Prochaine : ligne 82401 (suite DrawImageId, EFI_HII_DRAW_FLAGS, puis 34.5 Database protocol...).

## Lignes 82401-82700 — 34.4.6 DrawImageId (fin : EFI_HII_DRAW_FLAGS : CLIP 0x01, TRANSPARENT 0x30, DEFAULT 0x00, FORCE_TRANS 0x10, FORCE_OPAQUE 0x20, DIRECT_TO_SCREEN 0x80), 34.5 EFI HII Image Ex (GUID 1a1241e6-8f19-41a9-...; NewImageEx, GetImageEx, SetImageEx, DrawImageEx, DrawImageIdEx, GetImageInfo ; les variantes Ex délèguent aux fonctions de base et ajoutent le décodage par EFI_HII_IMAGE_DECODER_PROTOCOL si le format n'est pas géré), 34.5.7 GetImageInfo (renvoie seulement Width/Height ; Bitmap toujours NULL ; UNSUPPORTED si format inconnu ; NOT_FOUND si absente ; ImageId 0 = INVALID), 34.6 EFI HII Image Decoder Protocol (décodeurs pour les formats JPEG et PNG ; un décodeur peut annoncer d'autres formats ; le décodeur vérifie la signature avant traitement ; GUID 9E66F251-727C-418C-...; GetImageDecoderName, GetImageInfo, DecodeImage)
- [AGENT] Projet : HII/images hors périmètre. Règle utile : un décodeur d'image doit vérifier la signature du format avant de traiter les octets (les données viennent potentiellement de l'extérieur).

## Statut
- Lignes 1 à 82700 lues. Prochaine : ligne 82701 (suite DecodeImage : signature, codes, puis 34.7 Database protocol...).

## Lignes 82701-83000 — 34.6.1 DecodeImage (fin : codes : UNSUPPORTED si format non géré ; OUT_OF_RESOURCE ; INVALID si Image NULL ou taille 0), 34.6.2 GetImageDecoderName (DecoderName tableau de GUID, NumberOfDecoderName ; GUID JPEG efefd093-0d9b-46eb-... ; GUID PNG af060190-5e3a-4025-... ; UNSUPPORTED si aucun décodeur), 34.6.3 GetImageInfo (EFI_HII_IMAGE_DECODER_IMAGE_INFO : vérifie la signature d'abord, UNSUPPORTED si non géré, INVALID si image corrompue ; l'appelant libère ImageInfo), types : EFI_HII_IMAGE_DECODER_COLOR_TYPE (RGB 0, RGBA 1, CMYK 2, UNKNOWN 0xFF), EFI_HII_IMAGE_DECODER_IMAGE_INFO_HEADER (DecoderName, ImageInfoSize, ImageWidth, ImageHeight, ColorType, ColorDepthInBits), JPEG_INFO (ScanType : PROGRESSIVE 0x01, INTERLACED 0x02 ; Reserved 64 bits), PNG_INFO (Channels), OTHER_INFO (ImageExtension CHAR16 ; ex. "GIF", "TIFF"), 34.6.4 Decode (Bitmap NULL = le décodeur alloue ; Transparent FALSE pour désactiver la transparence et accélérer ; fond noir 0x00000000 si pas de Bitmap), 34.7 Font Glyph Generator (GUID f7102853-7787-4dc2-...; GenerateGlyph, GenerateGlyphImage ; générateur de glyphes pour une famille de polices, appelé quand le glyphe manque dans la base ; exemple cité : police "Times new roman")
- [AGENT] Projet : HII/images hors périmètre. Remarque : ImageInfoSize et ImageExtension sont des tailles variables : toute lecture doit se faire à partir de ImageInfoSize, pas d'une taille fixe supposée.

## Statut
- Lignes 1 à 83000 lues. Prochaine : ligne 83001 (suite 34.7.2 GenerateGlyph : codes, GenerateGlyphImage...).

## Lignes 83001-83300 — 34.7.2 GenerateGlyph (GlyphBlock = EFI_HII_GIBT_VARIABILITY_BLOCK ; codes : INVALID si FontInfo ou GlyphBlock NULL, UNSUPPORTED si nom/style/taille non gérés), 34.7.3 GenerateGlyphImage (Cell, GlyphBuffer = BitmapData du bloc, FontInfo, Image, BltX/BltY, Transparent ; codes), 34.8 EFI_HII_DATABASE_PROTOCOL (GUID ef9fc172-a1b2-4693-...; NewPackageList, RemovePackageList, UpdatePackageList, ListPackageLists, ExportPackageLists, RegisterPackageNotify, UnregisterPackageNotify, FindKeyboardLayouts, GetKeyboardLayout, SetKeyboardLayout, GetPackageListHandle), 34.8.2 NewPackageList (un DriverHandle avec EFI_DEVICE_PATH_PROTOCOL ajoute un paquet device path ; chaque appel doit être suivi de RemovePackageList ; EFI_HII_HANDLE = void*), 34.8.3 RemovePackageList (NOT_FOUND si absent ; notifications REMOVE_PACK avant suppression), 34.8.4 UpdatePackageList (types collectés, anciens paquets de même type retirés, nouveaux ajoutés ; notifications REMOVE_PACK puis ADD_PACK), 34.8.5 ListPackageLists (PackageType ou ALL ; PackageGuid obligatoire seulement si type GUID et interdit sinon ; BUFFER_TOO_SMALL met à jour la taille), 34.8.6 ExportPackageLists (début : Handle ou NULL = tout ; BufferSize en entrée/sortie)
- [AGENT] Projet : HII hors périmètre. Règle utile : un ajout de paquets doit toujours avoir sa suppression symétrique (sinon fuite dans la base), règle à reprendre pour tout gestionnaire de ressources que l'agent écrirait.

## Statut
- Lignes 1 à 83300 lues. Prochaine : ligne 83301 (suite ExportPackageLists, RegisterPackageNotify, FindKeyboardLayouts, GetKeyboardLayout, SetKeyboardLayout, GetPackageListHandle).

## Lignes 83301-83600 — 34.8.6 ExportPackageLists (fin : EXPORT_PACK notif ; UpdatePackageList possible depuis le callback ; BUFFER_TOO_SMALL donne la taille), 34.8.7 RegisterPackageNotify (PackageType, PackageGuid obligatoire seulement si GUID ; NotifyType ; NotifyHandle à désenregistrer ; NULL GUID = toutes les paquets GUID), 34.8.8 UnregisterPackageNotify (NOT_FOUND si handle inconnu), 34.8.9 FindKeyboardLayouts (GUID de chaque disposition ; BUFFER_TOO_SMALL), 34.8.10 GetKeyboardLayout (KeyGuid NULL = courante ; EFI_HII_KEYBOARD_LAYOUT : LayoutLength, Guid, LayoutDescriptorStringOffset, DescriptorCount, Descriptors[] ; EFI_DESCRIPTION_STRING : Language RFC 4646, espace séparateur, DescriptionString NUL ; EFI_DESCRIPTION_STRING_BUNDLE : DescriptionCount + chaînes ; EFI_KEY_DESCRIPTOR : Key, Unicode, ShiftedUnicode, AltGrUnicode, ShiftedAltGrUnicode, Modifier, AffectedAttribute)
- [AGENT] Projet : HII/clavier hors périmètre de l'agent (la gestion clavier brut passe par USB HID et xHCI, pas par HII). Remarque : une disposition clavier est une table de correspondance scancode -> caractère ; ne pas la charger comme une vérité du matériel.
- [AGENT] Anomalie de spec notée : l'exemple "2en-US English Keyboard<null>es-ES Keyboard en" contient des fragments incohérents (ingles<null> hors exemple) ; le nombre de chaînes DescriptionCount fait foi.

## Statut
- Lignes 1 à 83600 lues. Prochaine : ligne 83601 (suite EFI_KEY_DESCRIPTOR AffectedAttribute, SetKeyboardLayout, GetPackageListHandle, puis 34.9 et chapitre suivant).

## Lignes 83601-83900 — EFI_KEY_DESCRIPTOR (fin : AffectedAttribute : AFFECTED_BY_STANDARD_SHIFT 0x0001, CAPS_LOCK 0x0002, NUM_LOCK 0x0004), enum EFI_KEY (EfiKeyLCtrl = 0 ... EfiKeyIntl9 ; figure 34.3 : EfiKeyFour = chiffre 4 du pavé numérique), modificateurs (NULL 0x0000, LEFT_CONTROL 0x0001, RIGHT_CONTROL 0x0002, LEFT_ALT 0x0003, RIGHT_ALT 0x0004, ALT_GR 0x0005, INSERT 0x0006, DELETE 0x0007, PAGE_DOWN 0x0008, PAGE_UP 0x0009, HOME 0x000A, END 0x000B, LEFT_SHIFT 0x000C, RIGHT_SHIFT 0x000D, CAPS_LOCK 0x000E, NUM_LOCK 0x000F, flèches 0x0010-0x0013, NS_KEY 0x0014, NS_KEY_DEPENDENCY 0x0015, touches de fonction 1-12 = 0x0016-0x0021, PRINT 0x0022, SYS_REQUEST 0x0023, SCROLL_LOCK 0x0024, PAUSE 0x0025, BREAK 0x0026, LEFT_LOGO 0x0027, RIGHT_LOGO 0x0028, MENU 0x0029), 34.8.10 GetKeyboardLayout (fin : NOT_FOUND, BUFFER_TOO_SMALL, INVALID), 34.8.11 SetKeyboardLayout (KeyGuid ; GUID de l'événement EFI_HII_SET_KEYBOARD_LAYOUT_EVENT_GUID 14982a4f-b0ed-45b8-... signalé ; NOT_FOUND = action par défaut ; NULL = INVALID), 34.8.12 GetPackageListHandle (EFI_HII_HANDLE -> DriverHandle enregistré via NewPackageList)
- [AGENT] Projet : HII clavier hors périmètre. Remarque : les constantes EFI_KEY et modificateurs sont des tables d'interprétation et ne reflètent pas un scancode matériel ; ne pas les confondre avec les codes USB HID.

## Statut
- Lignes 1 à 83900 lues. Prochaine : ligne 83901 (suite 34.8.12 codes, puis 34.9 / chapitre 35 Human Interface (HII) Config Access, etc.).

## Lignes 83901-84200 — 34.8.12 GetPackageListHandle (fin : codes), 34.8.13 Database Structures (34.8.13.1 EFI_HII_DATABASE_NOTIFY : PackageType, PackageGuid, Package, Handle, NotifyType ; ce qu'ils désignent selon NEW_PACK, REMOVE_PACK, EXPORT_PACK, ADD_PACK), 34.8.14 EFI_HII_DATABASE_NOTIFY_TYPE (NEW_PACK 0x1, REMOVE_PACK 0x2, EXPORT_PACK 0x4, ADD_PACK 0x8), CHAPITRE 35 : HII CONFIGURATION PROCESSING AND BROWSER PROTOCOL (35.1 : chaînes name/value séparées par '=' et '&', UCS-2 ; flux : pilotes -> base HII -> applications de configuration, puis retour ; EFI HII Configuration Routing Protocol = seule interface appelée par l'extérieur ; EFI HII Configuration Access Protocol = côté pilote), 35.2 Configuration Strings (format HTTP sans % ; grammaire : GUID=, NAME=, PATH= (device path en hexa %02x), ALTCFG=, OFFSET=, WIDTH=, VALUE=, Label C, String UCS-2 ; mots réservés GUID NAME PATH ALTCFG OFFSET WIDTH VALUE ; NAMESPACE, KEYWORD, filtres DataFilter Buffer/Numeric(:1 :2 :4 :8)/String/Boolean/Date/Time, UsageFilter ReadOnly/ReadWrite, KEYWORDINFO, DISPLAYNAME, DATATYPE, MAX/MIN/STEP, OPTIONVALUE/OPTIONSTRING, STANDARDDEFAULT/MFGDEFAULT/SAFEDEFAULT ; &READONLY marque un mot-clé en lecture seule ; 35.2.1.6 exemples : KEYWORD=BIOSVendor ; réponse x-UEFI-ns&KEYWORD=BIOSVendor&VALUE=AcmeBIOSCompany)
- [AGENT] Projet : configuration firmware hors périmètre, et écriture de configuration interdite sans accord (règle projet). Retenu : toute chaîne de configuration est une donnée non fiable à valider avant usage.

## Statut
- Lignes 1 à 84200 lues. Prochaine : ligne 84201 (suite 35.2.1.6.3 exemples et 35.3 Routing / ConfigRouting protocol...).

## Lignes 84201-84500 — 35.2.1.6.4 exemple KEYWORDINFO=All (réponse : DataType, MAX, MIN, STANDARDDEFAULT, MFGDEFAULT), 35.2.2 String Types (six types : ConfigRequest (routage + noms, vers pilote), ConfigAltResp (courant puis alternatives ALTCFG=DefaultId, du pilote), ConfigResp (valeurs à appliquer, uniquement au pilote concerné : le pilote rejette les autres), MultiConfigRequest, MultiConfigAltResp, MultiConfigResp), 35.3 EFI Configuration Keyword Handler (GUID 0a8badd5-03b8-4d19-...; SetData, GetData ; erreurs KEYWORD_HANDLER_NO_ERROR 0, NAMESPACE_ID_NOT_FOUND 0x1, MALFORMED_STRING 0x2, KEYWORD_NOT_FOUND 0x4, INCOMPATIBLE_VALUE_DETECTED 0x8, ACCESS_NOT_PERMITTED 0x10, UNDEFINED 0x80000000 ; SetData : si un élément échoue, aucun stockage ne doit être modifié (tous les tests avant écriture) ; EFI_DEVICE_ERROR = état indéfini), 35.3.3 GetData (NamespaceId NULL = tous les espaces x-UEFI- ; Results alloué par la fonction, libéré par appelant), 35.4 EFI HII Configuration Routing Protocol (GUID 587e72d7-cc50-4f79-...; un seul exemplaire ; ExtractConfig, ExportConfig, RouteConfig, BlockToConfig, ConfigToBlock, GetAltConfig), 35.4.2 ExtractConfig (exemple : GUID=...&PATH=...&Fred&George -> valeurs ; alternatives ALTCFG=... après la configuration courante ; codes : NOT_FOUND si aucun pilote ne correspond, Progress pointe sur le G de GUID), 35.4.3 ExportConfig (tout le contenu HII ; Results NULL = INVALID)
- [AGENT] Projet : configuration firmware hors périmètre, aucune écriture. Règle utile à retenir : une écriture groupée doit tout valider avant de modifier quoi que ce soit (atomicité par validation préalable).

## Statut
- Lignes 1 à 84500 lues. Prochaine : ligne 84501 (suite 35.4.4 RouteConfig, 35.4.5 BlockToConfig, ConfigToBlock, GetAltConfig, puis 35.5 EFI HII Configuration Access Protocol...).

## Lignes 84501-84800 — 35.4.4 RouteConfig (Configuration en MultiConfigResp ; Progress = '&' fautif ; NOT_FOUND si cible absente ; ACCESS_DENIED si politique), 35.4.5 BlockToConfig (lit des octets via <BlockName> OFFSET=n&WIDTH=m, format %x ; échec si offset+width >= BlockSize), 35.4.6 ConfigToBlock (écrit VALUE dans le bloc ; exemple : bloc 00 01 02 03 04 05 + OFFSET=3WIDTH=1&VALUE=7&OFFSET=0&WIDTH=2&VALUE=AA55 -> 55 AA 02 07 04 05 ; BufferSize mis à jour si trop petit ; seuls les octets référencés sont modifiés), 35.4.7 GetAltCfg (extrait une portion par GUID, NAME, PATH, ALTCFG ; NULL = tout ; buffer alloué, libéré par appelant), 35.5 EFI HII Configuration Access Protocol (GUID 330d4706-f2a0-4e4f-...; ExtractConfig, RouteConfig, Callback ; le pilote publie ce protocole ; règle : la sortie ne dépend pas du contexte du navigateur ; 35.5.2 ExtractConfig (Request NULL = tous les réglages ; ALTCFG après la configuration courante ; codes), 35.5.3 RouteConfig (applique les paires name/value ; ConfigToBlock utile), 35.5.4 Callback (début : Action, QuestionId, Type, Value, ActionRequest)
- [AGENT] Projet : aucune écriture de configuration. Remarque : la règle "un bloc n'est modifié qu'aux octets référencés" est utile comme invariant pour toute écriture partielle que l'agent ferait un jour (ne toucher que les octets demandés).

## Statut
- Lignes 1 à 84800 lues. Prochaine : ligne 84801 (suite 35.5.4 CallBack : codes, actions EFI_BROWSER_ACTION_* ...).

## Lignes 84801-85100 — 35.5.4 CallBack (Action : CHANGING 0, CHANGED 1, RETRIEVE 2, FORM_OPEN 3, FORM_CLOSE 4, SUBMITTED 5, DEFAULT_STANDARD 0x1000, DEFAULT_MANUFACTURING 0x1001, DEFAULT_SAFE 0x1002, DEFAULT_PLATFORM 0x2000 (plage de 4096), DEFAULT_HARDWARE 0x3000, DEFAULT_FIRMWARE 0x4000 ; comportements par type de question (tableau 35.11) : CHANGING peut être refusé (UNSUPPORTED = valeur du navigateur ; SUCCESS = valeur du pilote ; autre erreur = valeur non modifiée) ; RETRIEVE : erreur = valeur d'origine ; FORM_OPEN : appelé avant lecture, BrowserCallback interdit ; ActionRequest : NONE 0, RESET 1, SUBMIT 2, EXIT 3, FORM_SUBMIT_EXIT 4, FORM_DISCARD_EXIT 5, FORM_APPLY 6, FORM_DISCARD 7, RECONNECT 8 (DisconnectController puis ConnectController sur le handle du protocole à la sortie), QUESTION_APPLY 9 ; retourner UNSUPPORTED pour toute action non gérée), 35.6 Form Browser Protocol (GUID b9d4c360-bcfb-4f9b-...; SendForm, BrowserCallback), 35.6.2 SendForm (Handles, HandleCount, FormsetGuid NULL = classe plateforme 93039971-8545-4b04-..., FormId 0 = premier formulaire actif, ScreenDimensions en caractères (8 px x 19 px en graphique), ActionRequested ; classes : PLATFORM_SETUP, DRIVER_HEALTH f22fc20c-..., USER_CREDENTIAL 337f4407-..., REST_STYLE 790217bd-...), 35.6.3 BrowserCallback (début : ResultsDataSize, ResultsData, RetrieveData, VariableGuid, VariableName)
- [AGENT] Projet : configuration firmware et navigateur hors périmètre ; aucune action de RECONNECT ou RESET ne doit être déclenchée par l'agent (règle projet : pas de modification système sans accord). Remarque : un callback qui ne gère pas une action doit renvoyer UNSUPPORTED, pas SUCCESS.

## Statut
- Lignes 1 à 85100 lues. Prochaine : ligne 85101 (suite BrowserCallback : codes, fin du chapitre 35, puis chapitre 36 suivant).

## Lignes 85101-85400 — 35.6.3 BrowserCallback (fin : ResultsDataSize entrée/sortie, RetrieveData TRUE = lire l'état non validé, FALSE = l'écrire ; codes SUCCESS, BUFFER_TOO_SMALL, UNSUPPORTED), 35.7 HII Popup Protocol (GUID 4311edc0-6054-46d4-...; CreatePopup ; Revision 1 ; styles Info/Warning/Error ; types Ok, OkCancel, YesNo, YesNoCancel ; sélections Ok, Cancel, Yes, No ; fenêtre modale), CHAPITRE 36 : USER IDENTIFICATION (36.1 : profils utilisateur (identifiant persistant, politique booléenne AND/OR/NOT sur classes ou fournisseurs de credentials, privilèges, infos publiques/privées/protégées), base de profils (non volatile, protégée ; ouverte ou fermée), politiques CredentialClass(Password) || CredentialClass(Fingerprint) etc., fournisseurs de credentials (EFI_USER_CREDENTIAL2_PROTOCOL, formulaires HII, EFI_HII_CONFIG_ACCESS), sécurité : les pilotes chargés avant le gestionnaire doivent être vérifiés ; fermeture de la base à EXIT_BOOT_SERVICES ou READY_TO_BOOT ; 36.1.5 exécution différée (EFI_DEFERRED_IMAGE_LOAD_PROTOCOL) ; 36.2 processus d'identification en 10 étapes), 36.3 Code (36.3.1 EFI_USER_MANAGER_PROTOCOL GUID 6fd5b00c-d426-4283-... : Create, Delete, GetNext, Current, Identify, Find, Notify, GetInfo)
- [AGENT] Projet : identification utilisateur hors périmètre (aucune authentification à réaliser, aucun credential ne doit être saisi ni stocké). Remarque retenue : le principe "les pilotes non vérifiés ne doivent pas s'exécuter pendant une décision de politique" s'applique au chargement de l'agent lui-même : ne charger que les binaires dont la signature est vérifiée.

## Statut
- Lignes 1 à 85400 lues. Prochaine : ligne 85401 (suite EFI_USER_MANAGER_PROTOCOL : fonctions Create, Delete, GetNext, Current, Identify, Find, Notify, GetInfo...).

## Lignes 85401-85700 — EFI_USER_MANAGER_PROTOCOL (fin : SetInfo, DeleteInfo, GetNextInfo ; description de chaque fonction), 36.3.1.2 Create (ACCESS_DENIED si profil courant ne le permet pas ; UNSUPPORTED ; handle valide seulement pendant ce démarrage), 36.3.1.3 Delete (refusé si un seul profil ; ACCESS_DENIED / UNSUPPORTED / INVALID), 36.3.1.4 GetNext (User NULL = premier ; au moins un profil existe toujours), 36.3.1.5 Current, 36.3.1.6 Identify (TPL_APPLICATION seulement ; événement EFI_EVENT_GROUP_USER_PROFILE_CHANGED 0xbaf1e6de-...), 36.3.1.7 Find (recherche dans les records d'information, InfoType + données partielles), 36.3.1.8 Notify (credential provider notifie un changement ; NOT_READY si non sélectionné ; UNSUPPORTED si asynchrone non géré), 36.3.1.9 GetInfo (EFI_USER_INFO : Credential GUID ou Nil, InfoType, Reserved1 = 0, InfoAttribs, InfoSize ; EFI_USER_INFO_ATTRIBS : STORAGE mask 0x000F (VOLATILE 0x0000, CREDENTIAL_NV 0x0001, PLATFORM_NV 0x0002), ACCESS mask 0x0070 (PUBLIC 0x0010, PRIVATE 0x0020, PROTECTED 0x0030), EXCLUSIVE 0x0080 ; ACCESS_DENIED si privé ou protégé non autorisé)
- [AGENT] Projet : identification utilisateur hors périmètre. Remarque : un attribut de confidentialité qui refuse l'accès doit renvoyer ACCESS_DENIED et ne jamais révéler la valeur ; pas de contournement.

## Statut
- Lignes 1 à 85700 lues. Prochaine : ligne 85701 (suite 36.3.1.9 GetInfo, Info records, EFI_USER_CREDENTIAL2_PROTOCOL, puis 36.3.2...).

## Lignes 85701-86000 — EFI_USER_INFO_STORAGE/ACCESS (fin : EXCLUSIVE 0x0080 = un seul record de ce type), 36.3.1.10 SetInfo (UserInfo NULL = ajout, sinon remplacement ; EXCLUSIVE déjà présent = ACCESS_DENIED ; SECURITY_VIOLATION ; l'identity policy doit rester synchrone avec les fournisseurs), 36.3.1.11 DeleteInfo (NOT_FOUND / ACCESS_DENIED), 36.3.1.12 GetNextInfo (fin : NOT_FOUND = fin d'énumération), 36.3.2 Credential Provider Protocols : 36.3.2.1 EFI_USER_CREDENTIAL2_PROTOCOL (GUID e98adb03-b8b9-4af8-...; Identifier, Type, Enroll, Form, Tile, Title, User, Select, Deselect, Default, GetInfo, GetNextInfo, Capabilities, Delete), classes : UNKNOWN 5cf32e68-..., PASSWORD f8e5058c-..., SMART_CARD 5f03ba33-..., FINGERPRINT 32cba21f-..., HANDPRINT 5917ef16-..., SECURE_CARD 8a6b4a83-... ; EFI_CREDENTIAL_CAPABILITIES (UINT64 ; ENROLL 0x1), 36.3.2.2 Enroll (ACCESS_DENIED / UNSUPPORTED / DEVICE_ERROR / INVALID), 36.3.2.3 Form (renvoie HII handle, FormSetId, FormId du premier formulaire actif de la classe EFI_HII_USER_CREDENTIAL_FORMSET_GUID ; NOT_FOUND si aucun formulaire requis)
- [AGENT] Projet : identifiants et enrôlement utilisateur hors périmètre. Aucun credential ne doit être saisi ni enregistré par l'agent (règle projet : pas de mots de passe ni de jetons). Remarque : un fournisseur qui ne propose pas d'enrôlement en pré-OS doit renvoyer UNSUPPORTED et non réussir à moitié.

## Statut
- Lignes 1 à 86000 lues. Prochaine : ligne 86001 (suite 36.3.2.4 Tile, Title, User, Select, Deselect, Default, GetInfo, GetNextInfo, Delete, puis User Information Table, Deferred Image Load...).

## Lignes 86001-86300 — 36.3.2.4 Tile (bitmap facultative, EFI_NOT_FOUND si absente ; Width/Height NULL = pas d'info), 36.3.2.5 Title (chaîne HII facultative), 36.3.2.6 User (cinq réponses : NOT_READY si aucun profil identifiable ; ACCESS_DENIED si verrouillé ; SUCCESS avec l'identifiant ; INVALID si Identifier NULL ; NOT_FOUND si profil inconnu), 36.3.2.7 Select (EFI_CREDENTIAL_LOGON_FLAG_AUTO 0x1 = essayer tout de suite ; DEFAULT 0x2 = fournisseur par défaut, choix d'implémentation si plusieurs), 36.3.2.8 Deselect, 36.3.2.9 Default (retourne les flags de logon par défaut), 36.3.2.10 GetInfo (BUFFER_TOO_SMALL donne la taille requise), 36.3.2.11 GetNextInfo, 36.3.2.12 Delete (ACCESS_DENIED / UNSUPPORTED / DEVICE_ERROR / INVALID), 36.3.3 Deferred Image Load Protocol (GUID 15853d7c-3ddf-43e0-...; GetImageInfo ; liste des images dont le chargement a été refusé pour raisons de sécurité ; réévaluées à chaque changement d'identité)
- [AGENT] Projet : hors périmètre. Remarque retenue pour l'agent : un chargement d'image différé pour raison de sécurité doit être consigné (liste d'images refusées) plutôt qu'ignoré silencieusement.

## Statut
- Lignes 1 à 86300 lues. Prochaine : ligne 86301 (suite EFI_DEFERRED_IMAGE_LOAD_PROTOCOL.GetImageInfo, EFI_DEFERRED_IMAGE_INFO, puis fin du chapitre 36 et chapitre 37 suivant).

## Lignes 86301-86600 — 36.3.3.2 DeferredImage GetImageInfo (ImageIndex 0-based, ImageDevicePath non libérable, Image NULL possible si la décision vient de l'emplacement, ImageSize, BootOption ; NOT_FOUND = fin d'énumération), 36.4 User Information (types d'enregistrements : EMPTY 0x00, NAME 0x01, CREATE_DATE 0x02, USAGE_DATE 0x03, USAGE_COUNT 0x04, IDENTIFIER 0x05 (16 octets, exactement un par profil), CREDENTIAL_TYPE 0x06, CREDENTIAL_TYPE_NAME 0x07, CREDENTIAL_PROVIDER 0x08, CREDENTIAL_PROVIDER_NAME 0x09, PKCS11 0x0A, CBEFF 0x0B, FAR 0x0C, RETRY 0x0D, ACCESS_POLICY 0x0E, IDENTITY_POLICY 0x0F, GUID 0xFF), 36.4.1 ACCESS_POLICY (EFI_USER_INFO_ACCESS_CONTROL : Type, Size ; FORBID_LOAD 0x1 et PERMIT_LOAD 0x2 (PERMIT prime sur FORBID) ; ENROLL_SELF 0x3, ENROLL_OTHERS 0x4, MANAGE 0x5, SETUP 0x6 (GUID ADMIN 85b75607-..., NORMAL 1db29ae0-..., RESTRICTED bdb38125-...), FORBID_CONNECT 0x7 et PERMIT_CONNECT 0x8 (sur ConnectController), BOOT_ORDER 0x9 (INSERT 0x0 / APPEND 0x1 / REPLACE 0x2, masque 0x000F ; NODEFAULT 0x10 ; exclusif)), 36.4.2 CBEFF (ISO 19785), 36.4.3 CREATE_DATE (EFI_TIME), 36.4.4-36.4.8 fournisseur, types et noms (CHAR16), GUID record 0xFF, 36.4.9 FAR (pourcentage 0 à 100), 36.4.10 IDENTIFIER (16 octets), 36.4.11 IDENTITY_POLICY (expression RPN : FALSE 0x00, TRUE 0x01, CREDENTIAL_TYPE 0x02, CREDENTIAL_PROVIDER 0x03, NOT 0x10, AND 0x11, OR 0x12 ; sans profil = FALSE)
- [AGENT] Projet : identité et politique utilisateur hors périmètre. Remarque : une politique d'accès qui contrôle le chargement d'exécutables (FORBID_LOAD / PERMIT_LOAD) est une règle de sécurité à respecter ; l'agent ne doit contourner aucune de ces règles.

## Statut
- Lignes 1 à 86600 lues. Prochaine : ligne 86601 (suite 36.4.11 IDENTITY_POLICY : table des types, 36.4.12 et suivants, puis chapitre 37 suivant).

## Lignes 86601-86900 — Table 36.17 (fin : IDENTITY_POLICY : CREDENTIAL_TYPE 0x02 (GUID suit), CREDENTIAL_PROVIDER 0x03 (GUID suit), NOT 0x10, AND 0x11, OR 0x12), 36.4.12 NAME (0x01), 36.4.13 PKCS11 (0x0A), 36.4.14 RETRY (0x0D, UINT8), 36.4.15 USAGE_DATE (0x03), 36.4.16 USAGE_COUNT (0x04, UINT64), 36.5 User Information Table (EFI_USER_INFO_TABLE : Size UINT64 total), CHAPITRE 37 : SECURE TECHNOLOGIES (37.1 Hash : SHA-1, SHA-256, SHA-224, SHA-384, SHA-512, MD5 ; EFI_HASH_SERVICE_BINDING 42881c98-a4f3-44b0-... ; EFI_HASH_PROTOCOL c5184932-dba5-46db-... : GetHashSize, Hash ; Hash(Extend, Message, MessageSize (multiple du bloc pour NOPAD), Hash en entrée/sortie) ; NOPAD : pas de padding par l'implémentation, le dernier bloc doit être paddé par l'appelant ; EFI_HASH_OUTPUT union ; GUIDs NOPAD SHA1 24c5dc2f-53e2-40ca-... et SHA256 8628752a-6cb7-4814-...), 37.2 Hash2 (EFI_HASH2_SERVICE_BINDING da836f8d-217f-4ca0-... ; EFI_HASH2_PROTOCOL 55b1d734-c5e1-49db-... : GetHashSize, Hash, HashInit, HashUpdate, HashFinal ; padding et finalisation faits par le pilote)
- [AGENT] Projet : hachage pertinent pour les empreintes des binaires (ex. vérification d'intégrité d'un .efi avant exécution, comparaison avec un hash de référence). Règle utile : pour un algorithme NOPAD, un message dont la longueur n'est pas un multiple du bloc est une erreur, pas une troncature silencieuse.

## Statut
- Lignes 1 à 86900 lues. Prochaine : ligne 86901 (suite 37.2.2.2 GetHashSize Hash2, HashInit, HashUpdate, HashFinal, puis 37.3 Signature / Hash2 codes...).

## Lignes 86901-87200 — 37.2.2.1 Hash2 (Hash = empreinte finale en un appel, padding ajouté ; HashInit/HashUpdate/HashFinal = multi-parties ; un Hash() pendant un HashInit annule la session en cours ; NOPAD SHA1/SHA256 non compatibles Hash2 = UNSUPPORTED ; tableau 37.2 : Hash("ABCDEF") = HashInit + HashUpdate("ABCDEF") + HashFinal = HashInit + HashUpdate("ABC") + HashUpdate("DE") + HashUpdate("F") + HashFinal, résultats identiques), 37.2.2.2 GetHashSize Hash2, 37.2.2.3 Hash Hash2 (MessageSize n'a pas de contrainte de bloc ; INVALID si Hash NULL ; OUT_OF_RESOURCES si taille > maximum plateforme), 37.2.2.4 HashInit (ALREADY_STARTED si une session est déjà ouverte sur l'instance), 37.2.2.5 HashUpdate (zéro octet = SUCCESS sans effet ; NOT_READY si pas de HashInit), 37.2.2.6 HashFinal (NOT_READY si pas de HashInit ou pas de HashUpdate ; après HashFinal, plus de mise à jour), table 37.8 : GUID SHA1 2ae9d80f-3fb2-4095-..., SHA224 8df01a06-9bd5-4bf7-..., SHA256 51aa59de-fdf2-4ea3-..., SHA384 efa96432-de33-4dd2-..., SHA512 caa4381e-750c-4770-..., MD5 af7c79c-65b5-4319-... (rétrocompatibilité seulement), 37.2.3.1 EFI_HASH2_OUTPUT (union MD5 16, SHA1 20, SHA224 28, SHA256 32, SHA384 48, SHA512 64 octets), 37.3 Key Management Service (EFI_KMS_PROTOCOL GUID EC3A978D-7C4E-48FA-... ; types de données NONE 0, BINARY 1, ASCII 2, UNICODE 4, UTF8 8 (combinables) ; service de clés : stockage, génération, éventuellement serveur ou HSM ; authentification et contrôle d'accès hors protocole ; début : GetServiceStatus)
- [AGENT] Projet : hachage utile pour vérifier une empreinte de binaire ; éviter SHA-1 et MD5 pour toute décision de sécurité nouvelle (le texte le dit déjà : rétrocompatibilité). Règle à retenir : le hash d'un message dans une session multi-parties ne doit pas être réutilisé après HashFinal.

## Statut
- Lignes 1 à 87200 lues. Prochaine : ligne 87201 (suite KMS : GetServiceStatus, Start..., Key ops, puis fin du chapitre 37 et chapitres suivants).

## Lignes 87201-87500 — 37.3.1 EFI_KMS_PROTOCOL (fin de la liste des fonctions : RegisterClient, CreateKey, GetKey, AddKey, DeleteKey, GetKeyAttributes, AddKeyAttributes, DeleteKeyAttributes, GetKeyByAttributes ; GetServiceStatus seule fonction valide avant ServiceAvailable ; AddKey remplace une clé de même identifiant) ; champs : ProtocolVersion = 0x00020040 (version initiale), ServiceId (GUID optionnel), ServiceName, ServiceVersion, ServiceAvailable, ClientIdSupported/Required/MaxSize, ClientNameStringTypes/Required/MaxCount, ClientDataSupported/MaxSize (tous à 1 si inconnu), KeyIdVariableLenSupported, KeyIdMaxSize (clés plus courtes complétées à droite par des espaces), KeyFormatsCount/KeyFormats (au moins un format ; même algorithme de hachage contigu, tailles croissantes ; un GUID inconnu = UNSUPPORTED), KeyAttributesSupported, KeyAttributeIdStringTypes (BINARY interdit), KeyAttributeIdMaxCount, KeyAttributesCount/KeyAttributes ; EFI_KMS_CLIENT_INFO (ClientIdSize, ClientId, ClientNameType, ClientNameCount, ClientName) ; GUID de formats génériques 128 à 3072 bits et DYNAMIC (EFI_KMS_FORMAT_GENERIC_DYNAMIC : KeySize UINT32 + KeyData), hash (MD2, MDC2, MD4, MDC4, MD5, MD5SHA 128 bits ; SHA1_160 ; SHA256_256 ; SHA512_512), chiffrement (AESXTS 128/256, AESCBC 128/256, RSASHA1 1024/2048, RSASHA256 2048/3072) ; tableau 37.9 : AES-XTS 256 bits (deux champs de 128) etc., RSA hash SHA1 ou SHA256
- [AGENT] Projet : KMS hors périmètre. Remarque : les tables de formats de clés sont des identificateurs à comparer exactement (GUID), jamais par tailles seules.
- [AGENT] Anomalie de spec notée : la liste des GUID RSASHA256_3072 est en partie illisible (texte mal extrait) ; et le nom EFI_KMS_FORMAT_SHA512 512_GUID contient un espace ; pour toute valeur, vérifier dans un en-tête officiel.

## Statut
- Lignes 1 à 87500 lues. Prochaine : ligne 87501 (suite EFI_KMS_KEY_DESCRIPTOR et types, puis fonctions KMS).

## Lignes 87501-87800 — EFI_KMS_KEY_DESCRIPTOR (fin : KeyStatus : SUCCESS, WARN_STALE_DATA (paramètres hors politique, à remplacer), COMPROMISED_DATA (à remplacer), UNSUPPORTED, OUT_OF_RESOURCES, TIMEOUT, DEVICE_ERROR, INVALID_PARAMETER, NOT_FOUND ; KeyIdentifierType (pas BINARY), KeyIdentifierSize 0-255, KeyFormat GUID, KeyValue NULL = pas de clé), EFI_KMS_ATTRIBUTE types (NONE 0x00, INTEGER 0x01, LONG_INTEGER 0x02, BIG_INTEGER 0x03, ENUMERATION 0x04, BOOLEAN 0x05, BYTE_STRING 0x06, TEXT_STRING 0x07, DATE_TIME 0x08, INTERVAL 0x09, STRUCTURE 0x0A, DYNAMIC 0x0B), EFI_KMS_DYNAMIC_ATTRIBUTE (FieldCount + Field[]), EFI_KMS_DYNAMIC_FIELD (Tag, Type, Length, KeyAttributeData), EFI_KMS_KEY_ATTRIBUTE (identifiant, KeyAttributeInstance 0xFFFF = joker en entrée, KeyAttributeType, KeyAttributeValueSize 0 = pas de valeur (pointeur NULL), KeyAttributeStatus), codes d'attribut, description générale (client identifié par EFI_KMS_CLIENT_INFO si requis ; passage de données arbitraires à l'appelant ; usage à limiter), 37.3.2 GetServiceStatus (SUCCESS, NOT_READY, NO_MAPPING, NO_RESPONSE, DEVICE_ERROR, INVALID ; seule fonction valide avant disponibilité ; initie la connexion si besoin), 37.3.2.1 RegisterClient (ClientDataSize NULL = ClientData ignoré ; 0 = rien envoyé mais données reçues possibles ; ClientData libéré par l'appelant ; si l'identifiant fourni est rejeté, le KMS peut en fournir un autre ; NULL dans la structure = demande d'identifiant généré ; codes), 37.3.2.2 CreateKey (génération optionnelle ; KeyDescriptors : Identifier optionnel ; KeyFormat de la première description = défaut pour les suivantes avec GUID NULL ; mémoire identifiant et valeur en BOOT_SERVICES_DATA, libérée par l'appelant ; KeyStatus par clé ; codes)
- [AGENT] Projet : KMS hors périmètre ; aucune génération ou import de clé ne doit être lancé par l'agent. Règle utile : le statut global d'une opération groupée peut différer du statut par élément ; toujours examiner chaque KeyStatus.

## Statut
- Lignes 1 à 87800 lues. Prochaine : ligne 87801 (suite CreateKey codes, GetKey, AddKey, DeleteKey, GetKeyAttributes, AddKeyAttributes, DeleteKeyAttributes, GetKeyByAttributes).

## Lignes 87801-88100 — 37.3.2.3 GetKey (obligatoire dans toute instance KMS ; identifiant sur entrée, format et valeur en sortie ; mémoire BOOT_SERVICES_DATA libérée par appelant ; si plusieurs clés pour un identifiant et tampon trop petit, KeyDescriptorCount indique le nombre traité ; codes), 37.3.2.4 AddKey (identifiant par défaut de la première description ; 0 = remplacé par le défaut ; optionnel ; codes), 37.3.2.5 DeleteKey (optionnel ; identifiant seul dans la description, champs autres à NULL), 37.3.2.6 GetKeyAttributes (KeyAttributesCount = 0 si aucun ; champs d'entrée à NULL ; codes : NOT_FOUND = identifiant inconnu, count 0), 37.3.2.7 AddKeyAttributes (optionnel ; UNSUPPORTED = gestion des attributs faite autrement ; codes), 37.3.2.8 DeleteKeyAttributes (début : KeyAttributesCount en entrée >= 1)
- [AGENT] Projet : KMS hors périmètre. Remarque : les opérations de suppression de clé ou d'attribut sont destructrices ; à ne jamais lancer sans accord explicite (règle projet, opérations irréversibles).

## Statut
- Lignes 1 à 88100 lues. Prochaine : ligne 88101 (suite DeleteKeyAttributes codes, GetKeyByAttributes, puis chapitre 38 ou suivant).

## Lignes 88101-88400 — 37.3.2.8 DeleteKeyAttributes (fin : codes), 37.3.2.9 GetKeyByAttributes (retourne les clés qui correspondent à tous les attributs ; KeyAttributeInstance 0xFFFF = toutes les instances ; KeyDescriptors en sortie = mémoire BOOT_SERVICES_DATA ; codes), 37.4 PKCS7 Verify Protocol (EFI_PKCS7_VERIFY_PROTOCOL GUID 47889fb2-d671-4fab-... ; RFC 2315 ; DER uniquement ; formats : ContentInfo/SignedData embarqué ou signature détachée ; VerifyBuffer, VerifySignature), 37.4.2 VerifyBuffer (AllowedDb obligatoire : succès si le signataire y figure et pas dans RevokedDb ; RevokedDb optionnel : bloque aussi un hash de contenu révoqué ; TimeStampDb : autorise un signataire révoqué si l'horodatage précède la révocation ; Content copié seulement si fourni, BUFFER_TOO_SMALL donne la taille ; SECURITY_VIOLATION si signataire révoqué ou absent de AllowedDb ; COMPROMISED_DATA si hash calculé différent du hash signé ; un match peut se faire à n'importe quel niveau de la chaîne), 37.4.2.1 VerifySignature (hash fourni par l'appelant ; ses données embarquées sont ignorées ; taille du hash doit égaler celle signée ; NOTE : vérifier chaque algorithme de hash car une révocation avec un autre algorithme pourrait échapper au contrôle)
- [AGENT] Projet : vérification de signature PKCS7 pertinente pour le binaire agent (signature Authenticode / PKCS7 d'une image). Règles retenues : AllowedDb requis (une absence = refus), une révocation l'emporte, et tester tous les hashs supportés si la révocation peut utiliser un autre algorithme. Aucune base de confiance ne doit être modifiée par l'agent (règle projet).
- Sections 37.5 RNG (début) : EFI_RNG_PROTOCOL GUID 3152bca5-eade-433d-... ; GetInfo (liste d'algorithmes, le premier = défaut) ; GetRNG (début de description).

## Statut
- Lignes 1 à 88400 lues. Prochaine : ligne 88401 (suite 37.5.3 GetRNG : paramètres, codes, puis 37.6 et suivants...).

## Lignes 88401-88700 — 37.5.3 GetRNG (RNGAlgorithm NULL = défaut ; RNGValueLength octets exactement ; pas de réutilisation d'octets entre appels ; codes : NOT_READY si entropie insuffisante ; INVALID si buffer NULL ou longueur 0), 37.5.4 Algorithmes (SP800_90_HASH_256 a7af67cb-..., HMAC_256 c5149b43-..., CTR_256 44f0de6e-..., X9_31_3DES 63c4785a-..., X9_31_AES acd03321-..., RAW e43176d7-... (entropie brute, sans DRBG), ARM_RNDR 43d2fde3-... ; DRBG sur une source brute : niveau de sécurité d'au moins 256 bits), 37.5.5 références NIST SP 800-90 et X9.31, 37.6 Smart Card Reader and Smart Card Edge Protocol (basé sur PC/SC partie 5 ; pas de gestionnaire de ressources ; pas de partage de carte ; énumération par OpenProtocol/SCardStatus ; GUID 2a4d1adf-21dc-4b81-...), 37.6.1 structure (SCardConnect, SCardDisconnect, SCardStatus, SCardTransmit, SCardControl, SCardGetAttrib), 37.6.2 SCardConnect (AccessMode SCARD_AM_READER 0x1 / CARD 0x2 ; CardAction NORESET 0, COLDRESET 1, WARMRESET 2, UNPOWER 3, EJECT 4 ; protocoles UNDEFINED 0, T0 0x1, T1 0x2, RAW 0x4 ; accès lecteur = PreferredProtocols UNDEFINED et NORESET obligatoires ; codes : INVALID, NOT_READY (pas d'ATR), UNSUPPORTED, NO_MEDIA, ACCESS_DENIED (verrou déjà pris), DEVICE_ERROR), 37.6.3 SCardDisconnect (libère le verrou ; succès si carte retirée ou pas d'appel préalable ; EJECT non supporté = UNSUPPORTED), 37.6.4 SCardStatus (ReaderName, State SCARD_UNKNOWN 0 / ABSENT 1 / INACTIVE 2 / ACTIVE 3, CardProtocol, Atr (33 octets usuels), tailles entrée/sortie ; NULL = non rempli sans erreur ; contenu invalide si pas SUCCESS)
- [AGENT] Projet : lecteurs de cartes à puce hors périmètre (aucune carte ni PIN ne doit être utilisé par l'agent). Règle retenue : une commande de déconnexion ou d'éjection physique n'est jamais lancée sans accord.

## Statut
- Lignes 1 à 88700 lues. Prochaine : ligne 88701 (suite 37.6.5 SCardTransmit : CAPDU/RAPDU, codes, 37.6.6 SCardControl, SCardGetAttrib...).

## Lignes 88701-89000 — 37.6.5 SCardTransmit (CAPDU -> RAPDU ; CAPDU NULL ou longueur 0 = INVALID ; RAPDULength entrée = taille max, sortie = taille réelle ; BUFFER_TOO_SMALL donne la taille requise ; NO_MEDIA, NOT_READY (carte non alimentée), PROTOCOL_ERROR, TIMEOUT, ACCESS_DENIED si une communication est déjà en cours ; SCardConnect requis avant), 37.6.6 SCardControl (ControlCode propre au fournisseur ; seul code standard : CM_IOCTL_GET_FEATURE_REQUEST = 0x42000000 + 3400 (PC/SC partie 10) ; InBuffer/OutBuffer facultatifs selon le code ; codes multiples), 37.6.7 SCardGetAttrib (Attrib = identifiant PC/SC partie 3 ; OutBuffer NULL ou longueur 0 = INVALID ; BUFFER_TOO_SMALL met à jour la taille), 37.6.8 Smart Card Edge Protocol (GUID d317f29b-a325-4712-... ; interface PKI pour l'authentification, la signature et le déchiffrement pendant le boot ; pas d'administration ni de provisioning de carte ; scrutation de l'insertion et du retrait ; GetContext, Connect, Disconnect, GetCsn, GetReaderName, VerifyPin, GetPinRemaining, GetData, GetCredential, SignData, DecryptData, BuildDHAgreement ; 37.6.8.2 GetContext (AID max 16 octets, CSN 16 octets, version SMART_CARD_EDGE_PROTOCOL_VERSION_1 = 0x100 ; table tronquée si trop petite ; le retrait met à jour la liste immédiatement, même si une connexion est ouverte)
- [AGENT] Projet : cartes à puce et PIN hors périmètre. Règle absolue : l'agent ne vérifie jamais de PIN, ne signe ni ne déchiffre rien avec une carte. Aucune saisie de code secret.

## Statut
- Lignes 1 à 89000 lues. Prochaine : ligne 89001 (suite 37.6.8.3 Connect, Disconnect, GetCsn, VerifyPin, SignData, DecryptData... puis chapitre suivant).

## Lignes 89001-89300 — 37.6.8.3 Connect (ScardCsn NULL = premier carte supportée ; SCardHandle reste valide après retrait mais les fonctions échouent jusqu'à Disconnect ; NO_MEDIA si aucune carte supportée), 37.6.8.4 Disconnect (succès même si carte retirée), 37.6.8.5 GetCsn (16 octets), 37.6.8.6 GetReaderName (ReaderNameLength en entrée = taille max), 37.6.8.7 VerifyPin (PIN en clair côté protocole ; PinCode NULL + PinSize négatif = test de support Secure PIN Entry PCSC V2 ; PinResult TRUE si le PIN est accepté ; RemainingAttempts), 37.6.8.8 GetPinRemaining, 37.6.8.9 GetData (DataId = GUID partagé entre pilote et application ; DataSize en entrée/sortie ; ACCESS_DENIED si PIN non vérifié), 37.6.8.10 GetCredentials (liste TLV : en-tête 0x0000 nombre de conteneurs ; 0x0001 certificat (L sur 2 octets big-endian) ; 0x0002 index de clé ; 0x0003 type de clé (0x01 RSA déchiffrement, 0x02 RSA signature, 0x03-0x05 ECDSA 256/384/521, 0x06-0x08 ECDH 256/384/521) ; 0x0004 taille de clé en bits (L 2 octets))
- [AGENT] Projet : aucune vérification de PIN, aucun accès aux credentials ni aux certificats de carte. L'agent ne doit pas non plus demander ni stocker un PIN (règle absolue : identifiants jamais saisis, jamais écrits). Ce passage est lecture de spec seulement.
- [AGENT] Note : GetPinRemaining et VerifyPin renvoient ACCESS_DENIED/NO_MEDIA selon la présence de la carte ; ne pas réessayer en boucle un PIN (risque de blocage de carte).

## Statut
- Lignes 1 à 89300 lues. Prochaine : ligne 89301 (suite 37.6.8.10 codes, 37.6.8.11 SignData, DecryptData, BuildDHAgreement, puis chapitre 37 fin et chapitre 38+).

## Lignes 89301-89600 — 37.6.8.10 GetCredentials (fin : CredentialList NULL si taille non nulle = INVALID ; BUFFER_TOO_SMALL), 37.6.8.11 SignData (signe un hash déjà calculé ; KeyType doit correspondre à KeyId ; HashAlgorithm SHA1/256/384/512 ; PaddingMethod RSASSA_PKCS1V1P5 9317ec24-... ou RSASSA_PSS 7b2349e0-... ; SignatureData dimensionné par type et taille de clé ; PIN non vérifié = ACCESS_DENIED), 37.6.8.12 DecryptData (clé de type SC_EDGE_RSA_EXCHANGE uniquement ; PaddingMethod NONE 3629ddb1-..., RSAES_PKCS1V1P5 e1c1d0a9-..., RSAES_OAEP c1e63ac4-... ; PlaintextSize en entrée/sortie ; BUFFER_TOO_SMALL), 37.6.8.13 BuildDHAgreement (clés ECDH 256/384/521 uniquement ; Qx et Qy big-endian ; l'accord doit être dérivé selon un processus convenu entre application et carte), 37.7 Memory Protection (EFI_MEMORY_ATTRIBUTE_PROTOCOL GUID f4560cf6-40ec-4b4a-...; GetMemoryAttributes, SetMemoryAttributes, ClearMemoryAttributes ; abstrait les attributs mémoire d'une région ; début 37.7.1.1 GetMemoryAttributes)
- [AGENT] Projet : signature et déchiffrement par carte hors périmètre ; aucune clé de carte utilisée. Remarque utile pour l'agent : les protections mémoire (W^X, NX) passent par ce protocole ; toute modification d'attribut mémoire est une action système à ne pas faire sans accord.

## Statut
- Lignes 1 à 89600 lues. Prochaine : ligne 89601 (suite 37.7.1.1 GetMemoryAttributes, SetMemoryAttributes, ClearMemoryAttributes).

## Lignes 89601-89900 — 37.7.1.1 GetMemoryAttributes (BaseAddress, Length, Attributes ; attributs valides en lecture : EFI_MEMORY_R, EFI_MEMORY_XP, EFI_MEMORY_RO ; attributs hétérogènes dans la région = NO_MAPPING), 37.7.1.2 SetMemoryAttributes (ajoute seulement les bits à 1 ; valides : EFI_MEMORY_RP, XP, RO ; codes : INVALID si longueur 0 ou combinaison illégale ; UNSUPPORTED ; ACCESS_DENIED si la région est contrôlée par le firmware système), 37.7.1.3 ClearMemoryAttributes (retire seulement les bits à 1 ; mêmes codes), CHAPITRE 38 : CONFIDENTIAL COMPUTING (début : firmware virtuel avec TEE matérielle (ex. Intel TDX) ; si un TPM virtuel est présent, EFI_TCG2_PROTOCOL et event log TCG ; sinon interfaces CC), 38.1 Virtual Platform CC Event Log (format TCG PFP EFI_TCG2_EVENT_LOG_FORMAT_TCG_2 ; index = registre de mesure CC propre au fournisseur), 38.2 EFI_CC_MEASUREMENT_PROTOCOL (GUID 96751a3d-72f4-41a6-...; GetCapability, GetEventLog, HashLogExtendEvent, MapPcrToMrIndex), 38.2.2 GetCapability (structure EFI_CC_BOOT_SERVICE_CAPABILITY : Size, StructureVersion (1.0), ProtocolVersion (1.0), HashAlgorithmBitmap (SHA384 0x00000004), SupportedEventLogs, CcType (Type, SubType ; NONE 0)), 38.2.3 GetEventLog (EventLogFormat, EventLogLocation, EventLogLastEntry, EventLogTruncated = TRUE si des entrées ont été perdues faute de place)
- [AGENT] Projet : hors périmètre (pas de TEE, pas de TPM virtuel dans le PC). Remarque : EventLogTruncated = TRUE signifie une piste d'audit incomplète ; ne pas la considérer comme complète.
- [AGENT] Point de vérification à retenir : SetMemoryAttributes ne peut pas retirer des bits ni les ajouter sans règle de combinaison ; toute modification d'attribut mémoire est une opération sensible à ne pas lancer sans accord.

## Statut
- Lignes 1 à 89900 lues. Prochaine : ligne 89901 (suite 38.2.3 GetEventLog codes, 38.2.4 HashLogExtendEvent, MapPcrToMrIndex, 38.3..., 38.4 CC types...).

## Lignes 89901-90200 — 38.2.3 GetEventLog (fin : EVENT_LOG_FORMAT_TCG_2 0x00000002), 38.2.4 HashLogExtendEvent (Flags : EXTEND_ONLY 0x1 = étendre sans journaliser ; PE_COFF_IMAGE 0x10 ; DataToHash physique ; EFI_CC_EVENT = Size + EFI_CC_EVENT_HEADER (HeaderSize, HeaderVersion = 1, MrIndex, EventType) + Event ; codes : VOLUME_FULL si journal plein, l'extension a eu lieu malgré tout), 38.2.5 MapPcrToMrIndex (correspondance PCR TPM -> MR CC, spécifique au fournisseur), 38.3 EFI CC Final Events Table (GUID dd4a4648-2de7-4665-... ; table des événements créés après GetEventLog ; Version = 1 ; NumberOfEvents ; CC_EVENT au format TPML_DIGEST_VALUES comme TCG_PCR_EVENT2), 38.4 Vendor Specific (38.4.1 Intel TDX : PCR 0 -> MR 0 MRTD ; PCR 1,7 -> MR 1 RTMR[0] ; PCR 2~6 -> MR 2 RTMR[1] ; PCR 8~15 -> MR 3 RTMR[2] ; CcType TDX = 2 ; 38.4.2 RISC-V AP-TEE : PCR 0~16 et 23 -> même indice MR, CcType = 3), CHAPITRE 39 : MISCELLANEOUS PROTOCOLS (39.1 EFI Timestamp Protocol, GUID afbfde41-2e6e-4262-...; GetTimestamp (compteur 64 bits libre, qui revient à zéro), GetProperties)
- [AGENT] Projet : les protocoles CC et TEE sont hors périmètre. Remarque : un mapping PCR -> MR est spécifique au fournisseur ; ne pas le réutiliser entre plateformes sans documentation.

## Statut
- Lignes 1 à 90200 lues. Prochaine : ligne 90201 (suite 39.1.3 GetProperties Timestamp, puis 39.2 et chapitres suivants jusqu'à 99708).

## Lignes 90201-90500 — 39.1.2 GetTimestamp (fin : compteur 64 bits libre, valeur pouvant être identique entre deux appels, ne recule pas sauf rollover ; fréquence constante même avec gestion d'énergie ; la quantification des mises à jour peut varier), 39.1.3 GetProperties (EFI_TIMESTAMP_PROPERTIES : Frequency en Hz, EndValue = valeur juste avant rollover ; EFI_DEVICE_ERROR, INVALID si pointeur NULL), 39.2 Reset Notification Protocol (GUID 9da34ae0-eaf9-4bbf-...; RegisterResetNotify, UnRegisterResetNotify ; 39.2.2 : le callback ne doit pas lui-même réinitialiser la plateforme ; traité seulement si ResetSystem est appelé avant ExitBootServices ; codes SUCCESS, INVALID, OUT_OF_RESOURCES, ALREADY_STARTED), 39.2.3 UnregisterResetNotify (INVALID si non enregistré), APPENDICE A : GUID et formats temporels (GUID RFC 4122 ; champs TimeLow, TimeMid, TimeHighAndVersion en little-endian ; horodatage 60 bits de compte de 100 ns depuis le 15 octobre 1582 ; format texte aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee ; tableau A.2 de correspondance octets/Data1-4), APPENDICE B : CONSOLE (B.1 correspondances des codes de scan EFI_SIMPLE_TEXT_INPUT_PROTOCOL et EX : séquences ANSI X3.64/VT200, VT100+, codes HID USB et AT 101/102 pour les touches Null, flèches, Home/End, Insert/Delete, PgUp/PgDn, F1-F10 ... ; tables B.1 et B.2 ; séquences CSI = 0x9B en 8 bits ou ESC [ en 7 bits)
- [AGENT] Projet : le ResetNotification est un mécanisme que l'agent ne doit jamais utiliser pour provoquer un reset (règle projet : pas de reset sans accord). Remarque pour les GUID : TimeLow/TimeMid/TimeHighAndVersion sont little-endian dans le binaire mais se lisent dans l'ordre texte ; erreur fréquente à éviter dans tout code qui parse des GUID.
- [AGENT] Point utile : les tables de correspondance clavier de l'annexe B sont des mappings de terminal ; ne pas les confondre avec les codes USB HID bruts lus par xHCI.

## Statut
- Lignes 1 à 90500 lues. Prochaine : ligne 90501 (suite annexe B table B.2, puis annexes suivantes du spec jusqu'à 99708).

## Lignes 90501-90800 — Table B.2 (fin : codes Mute 0x7F, Volume Up 0x80, Volume Down 0x81, Brightness Up 0x100, Brightness Down 0x101, Suspend 0x102, Hibernate 0x103, Toggle Display 0x104, Recovery 0x105, Eject 0x106, OEM reserved 0x80-0xFF selon le texte, 0xFFFF = fin de table ; colonnes ANSI/VT et HID/AT 101), B.2 EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL pour PC ANSI et ANSI X3.64 : table B.3 (séquences : ESC[2J effacer écran, ESC[0m normal, ESC[1m gras, ESC[7m inversé, ESC[30m à 37m couleur texte, ESC[40m à 47m couleur fond, ESC[=3h mode 80x25 couleur, ESC[row;colH positionnement curseur, en chiffres ASCII) ; table B.4 : exemple de disposition clavier (EFI_KEY, codes HID USB 0x04-0xE7, positions AT-101 ; touches étendues avec astérisque ; LeftControl 0xE0 (58), LeftShift 0xE1 (44), LeftAlt 0xE2 (60), LeftLogo 0xE3 (127), RightControl 0xE4 (64), etc.)
- [AGENT] Projet : tables de terminal et disposition clavier, lecture seule. Remarque : des codes de scan (AT, HID) et des codes EFI_KEY sont différents d'un système à l'autre ; ne pas les mélanger dans un parseur d'entrée.
- [AGENT] Anomalie de spec notée : dans la table B.4, la ligne "EfiKeyPeriod" et "Keypad . and Delete" n'ont pas le même nombre de colonnes que l'entête (extraction PDF) ; les valeurs HID prévalent sur les colonnes décalées.

## Statut
- Lignes 1 à 90800 lues. Prochaine : ligne 90801 (suite table B.4, puis annexes C et suivantes jusqu'à 99708).

## Lignes 90801-91100 — Table B.4 (fin : Non-US #/~ ; Non-US \ ; Non-US ¥ et | ; astérisque = touche non imprimable), ANNEXE C : DEVICE PATH EXAMPLES (C.1 machine exemple : chipset mémoire, root PCI host bridge (PCI0, _UID 0), second root PCI bridge (PCI1, _UID 1), USB/ATA66/AC'97 dans le root bridge, LPC vers SIO (ISA0)), C.2 Legacy floppy (chemin : ACPI _HID PNP0A03 _UID 0 ; PCI to ISA bridge (device/function) ; ACPI PNP0303 _UID 0 pour FLPY ; End : octets 0x02 0x01 0x0C 0x41D0 0x0A03 0x0000 0x01 ... 0xFF 0x04), C.3 IDE disk (ACPI PNP0A03 ; PCI device path de l'IDE ; ATAPI messaging (Primary 0/Secondary 1, Master 0/Slave 1, LUN) ; table C.2), C.4 Secondary root PCI bus (PNP0A03 _UID 1 ; PCI bridge puis device PCI ; table C.3), C.5 termes ACPI (_ADR, _CRS, _HID compressé EISA 32 bits, _UID unique par _HID ; noms ACPI 4 caractères), C.6 device path vu comme espace de noms (accès via DEVICE_PATH_INTERFACE depuis le handle ; tous les nœuds n'ont pas de handle) ; ANNEXE D : STATUS CODES (D.1 plages : 0x0-0x1FFF... bit haut 0 = succès/avertissement ; bit haut 1 = erreur ; bit 30 = réservé OEM ; plages 64 bits : 0x8000... etc. ; D.2 succès : EFI_SUCCESS = 0 ; D.3 erreurs : LOAD_ERROR 1, INVALID_PARAMETER 2, UNSUPPORTED 3, BAD_BUFFER_SIZE 4, BUFFER_TOO_SMALL 5 (taille requise renvoyée), NOT_READY 6, DEVICE_ERROR 7, WRITE_PROTECTED 8, OUT_OF_RESOURCES 9, VOLUME_CORRUPTED 10 ...)
- [AGENT] Projet : chemins d'appareil ACPI/PCI/ATA ci-dessus sont la base de la correspondance entre le matériel observé (fw_deep_acpi.txt) et les noms ACPI ; utile pour recouper MADT/FADT/_CRS/_UPC de ACPI_LECTURE.md. Règle à retenir : EFI_BUFFER_TOO_SMALL renseigne la taille requise, ne pas le traiter comme une erreur fatale.

## Statut
- Lignes 1 à 91100 lues. Prochaine : ligne 91101 (suite D.3 erreurs, D.4 warnings, annexes E et suivantes jusqu'à 99708).

## Lignes 91101-91400 — D.3 erreurs (fin : NO_RESPONSE 16, NO_MAPPING 17, TIMEOUT 18, NOT_STARTED 19, ALREADY_STARTED 20, ABORTED 21, ICMP_ERROR 22, TFTP_ERROR 23, PROTOCOL_ERROR 24, INCOMPATIBLE_VERSION 25, SECURITY_VIOLATION 26, CRC_ERROR 27, END_OF_MEDIA 28, END_OF_FILE 31, INVALID_LANGUAGE 32, COMPROMISED_DATA 33, IP_ADDRESS_CONFLICT 34, HTTP_ERROR 35 ; VOLUME_FULL 11, NO_MEDIA 12, MEDIA_CHANGED 13, NOT_FOUND 14, ACCESS_DENIED 15), D.4 avertissements (WARN_UNKNOWN_GLYPH 1, WARN_DELETE_FAILURE 2, WARN_WRITE_FAILURE 3, WARN_BUFFER_TOO_SMALL 4 (données tronquées), WARN_STALE_DATA 5, WARN_FILE_SYSTEM 6 (système de fichiers UEFI conforme), WARN_RESET_REQUIRED 7 (opération poursuivie après reset)), ANNEXE E : UNIVERSAL NETWORK DRIVER INTERFACES (E.1 : UNDI H/W et S/W pour écrire un pilote réseau ; PXE, BC (BaseCode), NBP, NIC, LOM ; E.1.2 liste des normes : ARP, BOOTP, DHCP, ICMP, IGMP, IP, MTFTP, PCI, PnP, PXE 2.1, RFC, TCP)
- [AGENT] Projet : PXE/UNDI hors périmètre (le projet n'utilise pas le réseau de démarrage). Remarque utile : les codes d'erreur WARN_* ne sont pas des échecs ; un code WARN_BUFFER_TOO_SMALL signale des données tronquées, à ne pas ignorer.

## Statut
- Lignes 1 à 91400 lues. Prochaine : ligne 91401 (suite annexe E : E.2 H/W UNDI, E.3 S/W UNDI, E.4... jusqu'à 99708).

## Lignes 91401-91700 — Annexe E (UNDI) : E.1.3 piles réseau OS (Custom / S/W UNDI / H/W UNDI, tableau pour/contre), E.2 vue d'ensemble (3 changements vs PXE 2.1 16 bits), E.2.1 structure !PXE (signature, Len, checksum, Rev, IFcnt, Major/Minor, bits d'implémentation, status, command), E.2.2 format CDB (OpCode, OpFlags, CPBsize, DBsize, CPBaddr, DBaddr, StatCode, StatFlags, IFnum, Control), E.3 définitions C (macros d'ordre d'octets, SWAP, PXE_BUSTYPE)
- [AGENT] Hors périmètre direct (PXE/UNDI non utilisé). Règles à retenir si un jour pertinent : la CDB doit être entièrement initialisée à zéro pour les champs non utilisés et les champs StatCode/StatFlags ; une CDB mal formée n'est pas exécutée (INVALID_CDB) ; ne jamais réutiliser une structure de commande avant completion.

## Statut
- Lignes 1 à 91700 lues. Prochaine : ligne 91701.

## Lignes 91701-92000 — Annexe E.3 suite : types portables (PXE_UINT8/16/32/64, ordre little-endian, PXE_BOOL), PXE_OPCODE (GET_STATE 0x0000 ... RECEIVE 0x0011, LAST_VALID 0x0011), PXE_OPFLAGS (par commande : INITIALIZE, RESET, INTERRUPT_ENABLES, RECEIVE_FILTERS, STATION_ADDRESS, STATISTICS, MCAST_IP_TO_MAC, NVDATA, GET_STATUS, FILL_HEADER, TRANSMIT), PXE_STATFLAGS (COMMAND_COMPLETE 0xC000, COMMAND_FAILED 0x8000, COMMAND_QUEUED 0x4000 ; par commande), PXE_STATCODE (SUCCESS 0x0000, INVALID_CDB 0x0001, INVALID_CPB 0x0002, BUSY 0x0003, QUEUE_FULL 0x0004, ALREADY_STARTED 0x0005, NOT_STARTED 0x0006, DEVICE_FAILURE 0x000A, NVDATA_FAILURE 0x000B, UNSUPPORTED 0x000C, BUFFER_FULL 0x000D, INVALID_PARAMETER 0x000E, NO_DATA 0x0013 ...)
- [AGENT] Hors périmètre (PXE/UNDI non utilisé). Point de cohérence : le modèle "CDB + StatCode/StatFlags à zéro avant envoi, puis attendre COMMAND_COMPLETE ou COMMAND_FAILED" est le même principe que la file de commandes NVMe/xHCI : ne pas réutiliser un slot tant que l'état final n'est pas observé.

## Statut
- Lignes 1 à 92000 lues. Prochaine : ligne 92001.

## Lignes 92001-92300 — Annexe E.3.4 suite : PXE_IFNUM, PXE_CONTROL (QUEUE_IF_BUSY 0x0002, LINK 0x0001), PXE_FRAME_TYPE, types IPv4/IPv6/MAC (big-endian), PXE_IFTYPE (Ethernet 0x01), PXE_MEDIA_PROTOCOL ; E.3.5 structures packées : PXE_HW_UNDI (signature "!PXE", Len, Fudge, Rev 0x02, MajorVer 0x03, MinorVer 0x01, bits Implementation, port d'état/commande H/W : PXE_HWSTAT_*, PXE_HWCMD_*), PXE_SW_UNDI (EntryPoint 64 bits, BusType "PCIR"...), PXE_UNDI (union), constantes PXE_ROMID_IMP_*, début de PXE_CDB (OpCode, OpFlags, CPBsize, DBsize, CPBaddr, DBaddr, StatCode, StatFlags, IFnum, Control)
- [AGENT] Hors périmètre (PXE/UNDI). Points de vigilance génériques : structures marquées #pragma pack(1) (pas de padding : utiliser des tailles fixes dans tout parseur) ; PXE_IPV4/IPV6/MAC en big-endian alors que le reste est little-endian (ne pas mélanger l'ordre d'octets) ; les champs réservés doivent être à zéro sinon la commande est refusée.

## Statut
- Lignes 1 à 92300 lues. Prochaine : ligne 92301.

## Lignes 92301-92600 — E.3.5.5 PXE_IP_ADDR (union IPv4/IPv6, big-endian), E.3.5.6 PXE_DEVICE (BusType, Bus, Device, Function : PCI/PCC) ; E.4 commandes UNDI : toutes même format CDB ; règle : ne lire ni modifier CDB/CPB/DB tant que COMMAND_COMPLETE ou COMMAND_FAILED n'est pas positionné ; états UNDI : Stopped (accepte Get State, Start), Started (Get State, Stop, Get Init Info, Initialize), Initialized (tout sauf Start, Stop, Initialize) ; E.4.1 chaînage (CDB consécutives, bit LINK sur toutes sauf la dernière, arrêt à la première erreur) et file d'attente (PXE_CONTROL_QUEUE_IF_BUSY, FIFO) ; E.4.2 Get State (statut COMMAND_COMPLETE/FAILED/QUEUED, état STOPPED/STARTED/INITIALIZED) ; E.4.3 Start : CPB de callbacks (Delay, Block, Virt2Phys, Mem_IO, Map_Mem, UnMap_Mem, Sync_Mem, UniqueId) pour S/W UNDI 3.0 et 3.1 ; directions TO_AND_FROM_DEVICE 0, FROM_DEVICE 1, TO_DEVICE 2 ; E.4.4 Stop (NOT_STARTED, NOT_SHUTDOWN).
- [AGENT] Hors périmètre (PXE). Règles génériques retenues : ne pas réutiliser une structure de commande avant son état final ; le chaînage s'arrête à la première erreur (à reproduire si l'agent enchaîne des commandes) ; les adresses de DMA passées au matériel doivent être des adresses périphérique, non virtuelles (même vigilance que pour xHCI/NVMe : adresses physiques).

## Statut
- Lignes 1 à 92600 lues. Prochaine : ligne 92601.

## Lignes 92601-92900 — E.4.5 Get Init Info (DB : MemoryRequired, FrameDataLen, LinkSpeeds[4], NvCount, NvWidth, MediaHeaderLen, HWaddrLen, MCastFilterCnt, TxBuf/RxBuf, IFtype, duplex, loopback) ; E.4.6 Get Config Info (DB PCI/PCC : BusType, Bus, Device, Function, copie de l'espace de configuration PCI 256 octets) ; E.4.7 Initialize (CPB : MemoryAddr, MemoryLength (adresse physique contiguë non swappable), LinkSpeed, buffers, DuplexMode, LoopBackMode ; réinitialisation 4 à 20 secondes ; COMMAND_COMPLETE non positionné avant que le NIC soit prêt ; DB : MemoryUsed, tailles de buffers réelles) ; E.4.8 Reset (vide files TX/RX, efface interruptions, filtres conservés sauf OpFlags) ; E.4.9 Shutdown (désactive filtres/interruptions, libère/réaffecte la mémoire, passe de Initialized à Started).
- [AGENT] Hors périmètre (PXE). Règles génériques : la mémoire passée au périphérique doit être contiguë, physique et non swappable (même principe que les buffers DMA xHCI/NVMe) ; une initialisation peut durer 20 s : prévoir un délai d'attente dans tout pilote, ne pas considérer la commande comme finie avant COMMAND_COMPLETE ; un état "pas de média" n'est pas une erreur.

## Statut
- Lignes 1 à 92900 lues. Prochaine : ligne 92901.

## Lignes 92901-93200 — E.4.10 Interrupt Enables (lecture / activation / désactivation RECEIVE, TRANSMIT, COMMAND, SOFTWARE ; SOFTWARE génère une interruption sans l'activer ; StatFlags peuvent différer de la demande), E.4.11 Receive Filters (UNICAST, BROADCAST, FILTERED_MULTICAST, PROMISCUOUS, ALL_MULTICAST ; liste MAC multicast ; un filtre non supporté est promu au niveau plus permissif), E.4.12 Station Address (READ / WRITE / RESET ; StationAddr, BroadcastAddr, PermanentAddr), E.4.13 Statistics (RX/TX counters ; Supported bitmap ; UNSUPPORTED si non implémenté), E.4.14 MCast IP to MAC (IPv4/IPv6), E.4.15 NvData (début : sparse ou bulk, selon !PXE.Implementation).
- [AGENT] Hors périmètre (PXE). Règles génériques : une demande de filtre "promue" signifie que le résultat réel doit être relu dans StatFlags et non supposé ; ne pas écrire dans une mémoire non-volatile (NvData) sans accord explicite (même règle que le contrat du projet : aucune écriture persistante sans accord) ; un état de statistiques non supporté est rempli à zéro, donc ne pas le lire comme "0 erreur".

## Statut
- Lignes 1 à 93200 lues. Prochaine : ligne 93201.

## Lignes 93201-93500 — E.4.15 NvData (suite : CPB sparse = liste (Addr, Data Byte/Word/Dword) ; CPB bulk = toute la mémoire NIC ; DB lecture), E.4.16 Get Status (OpFlags : INTERRUPT_STATUS efface les interruptions en lisant ; GET_TRANSMITTED_BUFFERS libère les buffers TX à recycler ; MEDIA_STATUS ; StatFlags RECEIVE/TRANSMIT/COMMAND/SOFTWARE ; TXBUF_QUEUE_EMPTY ; NO_TXBUFS_WRITTEN ; NO_MEDIA ; DB : RxFrameLen, reserved, xBuffer[n] 64 bits), E.4.17 Fill Header (WHOLE ou FRAGMENTED ; CPB : SrcAddr, DestAddr, MediaHeader ou FragDesc, PacketLen, Protocol 0x0800 IP / 0x0806 ARP, MediaHeaderLen), E.4.18 Transmit (buffers verrouillés : le pilote perd la propriété jusqu'au recyclage ; TX complete ; CPB Whole/Fragmented ; BLOCK / DONT_BLOCK pour S/W UNDI seulement ; BUFFER_FULL si file pleine ; attendre GET_TRANSMITTED_BUFFERS pour réutiliser), E.4.19 Receive (début : BufferAddr ; un buffer trop petit tronque la trame et la trame est perdue).
- [AGENT] Hors périmètre (PXE). Règles génériques : un buffer TX ne doit être ni libéré ni relocalisé tant que le périphérique ne l'a pas rendu (règle à retenir pour les anneaux TX NVMe/xHCI/UAS) ; un buffer RX trop petit ne donne pas une trame partielle réutilisable : la trame est perdue (à vérifier dans le code de lecture de disque/ring si la taille du buffer est sous-estimée) ; l'état BUFFER_FULL demande d'attendre et de vider, pas de renvoyer en boucle serrée.

## Statut
- Lignes 1 à 93500 lues. Prochaine : ligne 93501.

## Lignes 93501-93800 — E.4.19 Receive (fin : CPB BufferAddr/BufferLen/reserved ; états COMMAND_COMPLETE, NO_DATA si file vide ; DB : SrcAddr, DestAddr, FrameLen (peut dépasser BufferLen : c'est le moyen de détecter une perte), Protocol, MediaHeaderLen, Type, reserved[7]) ; E.4.20 clarifications PXE 2.1 (timeouts DHCP 4/8/16/32 s répétés, siaddr avant option 54, options PXE 6/8/9/12/71/43 rendues facultatives selon les cas) ; APPENDIX F : Simple Pointer Protocol (axes X/Y/Z, double-clic 0,5 s, vitesse en comptes/mm, 2 % de l'écran par mm) ; APPENDIX G : exemple EFI_EXT_SCSI_PASS_THRU (LocateHandle, HandleProtocol, GetNextTargetLun, PassThru bloquant ou non) ; APPENDIX H : début du code source de compression (Intel, LZ77 + Huffman, constantes WNDBIT 13, MAXMATCH 256, CRC 0xA001).
- [AGENT] Pertinent : (1) Receive : FrameLen > BufferLen = perte, à vérifier comme erreur ; (2) Simple Pointer : pas utile au projet ; (3) EXT_SCSI_PASS_THRU : rappel du projet : commandes SCSI destructives interdites sans accord explicite (GetNextTargetLun et PassThru en lecture seule seulement) ; (4) Appendix H : code de compression Intel hors périmètre, à survoler seulement.

## Statut
- Lignes 1 à 93800 lues. Prochaine : ligne 93801.

## Lignes 93801-94100 — APPENDIX H (suite) : constantes de compression (CBIT 9, NP WNDBIT+1, PBIT 4, NT CODE_BIT+3, TBIT 5), prototypes des fonctions internes (Encode, CountTFreq, WritePTLen, WriteCLen, EncodeC/EncodeP, SendBlock, Output, HufEncodeStart/End, MakeCrcTable, PutBits, FreadCrc, InitPutBits, CountLen, MakeLen, DownHeap, MakeCode, MakeTree), variables globales ; fonction Compress (entrée SrcBuffer/SrcSize, sortie DstBuffer/DstSize, retour EFI_BUFFER_TOO_SMALL avec la taille nécessaire, EFI_SUCCESS) ; PutDword (écriture little-endian avec contrôle de borne à chaque octet), AllocateMemory (malloc pour mText, mLevel, mChildCount, mPosition, mParent, mPrev, mNext, tampon mBuf réduit de 10 % si échec, minimum 4 Ko), FreeMemory.
- [AGENT] Hors périmètre (code de compression d'exemple, non nécessaire au projet). Seul point réutilisable : le contrôle de borne à chaque écriture (PutDword) et la gestion d'échec d'allocation avec repli ; à appliquer si l'agent écrit un parseur ou un encodeur.

## Statut
- Lignes 1 à 94100 lues. Prochaine : ligne 94101.

## Lignes 94101-94400 — APPENDIX H (suite) : InitSlide, Child, MakeChild, Split, InsertNode, DeleteNode, GetNextMatch (arbre de correspondance LZ77 : fenêtre WNDSIZ, Child/MakeChild/Split), Encode (boucle principale : Output caractère ou couple longueur/position), CountTFreq (fréquences du jeu Extra), WritePTLen et WriteCLen (écriture des longueurs de codes Huffman, symboles spéciaux 0/1/2 et CBIT), code de compression d'exemple Intel.
- [AGENT] Hors périmètre (compression d'exemple, non utilisée par le projet). Aucune règle à retenir au-delà de ce qui est déjà noté. Attention : ce code d'exemple contient des défauts de transcription dans l'extraction (formules tronquées, commentaires perdus) : ne pas l'utiliser comme référence.

## Statut
- Lignes 1 à 94400 lues. Prochaine : ligne 94401.

## Lignes 94401-94700 — APPENDIX H fin (EncodeC, EncodeP, SendBlock, Output, HufEncodeStart/End, MakeCrcTable, PutBits, FreadCrc, InitPutBits, CountLen, MakeLen, DownHeap, MakeCode, MakeTree : construction Huffman, longueurs de codes limitées à 16 bits) ; APPENDIX I : début du décompresseur Intel (constantes BITBUFSIZ 16, WNDBIT 13, WNDSIZ 8192, MAXMATCH 256, THRESHOLD 3, BAD_TABLE -1 ; structure SCRATCH_DATA avec mSrcBase, mDstBase, compteurs, tables mLeft/mRight/mCLen/mPTLen/mCTable/mPTTable, mBadTableFlag ; prototype FillBuf).
- [AGENT] Hors périmètre (compression/décompression d'exemple, non utilisée par le projet). Seule remarque : le décompresseur lit des données non fiables ; s'il est repris, toute longueur ou position lue doit être bornée (mSrcBase/mDstBase) et une table invalide doit donner BAD_TABLE, pas un accès hors limites.

## Statut
- Lignes 1 à 94700 lues. Prochaine : ligne 94701.

## Lignes 94701-95000 — APPENDIX I (suite) : protocole EFI_DECOMPRESS_PROTOCOL : GetInfo (ScratchSize = taille SCRATCH_DATA ; DstSize lu sur 4 octets little-endian ; SrcSize < 8 = EFI_INVALID_PARAMETER) ; Decompress (vérifie ScratchSize, CompSize + 8 <= SrcSize, met à zéro le scratch, boucle par fenêtres WNDSIZ, écriture bornée à DstSize sinon EFI_INVALID_PARAMETER, mBadTableFlag -> EFI_INVALID_PARAMETER) ; FillBuf, GetBits, MakeTable (vérifie BitLen <= 16, Start[17] != 0 => BAD_TABLE, débordement de table => BAD_TABLE).
- [AGENT] Pertinent comme modèle de validation : le décompresseur de référence contrôle toutes les tailles (source, scratch, destination) avant d'écrire et renvoie EFI_INVALID_PARAMETER sur données corrompues. Règle à reprendre pour tout parseur de données externes (firmware, disque, réseau) : valider taille, bornes et table avant usage, ne jamais écrire au-delà de la destination.

## Statut
- Lignes 1 à 95000 lues. Prochaine : ligne 95001.

## Lignes 95001-95300 — APPENDIX I fin (MakeTable suite, DecodeP, ReadPTLen, ReadCLen, DecodeC, Decode : décodage Huffman + LZ77 ; BAD_TABLE renvoyé sur table invalide, longueur de bloc nulle relue, position hors fenêtre masquée par (WNDSIZ - 1) ; contrôle mOrigSize pendant la copie).
APPENDIX J (début) : titre "EFI BYTE CODE VIRTUAL MACHINE OPCODE LIST" (EBC, machine virtuelle de code octet UEFI).
- [AGENT] Décompresseur : même remarque que précédemment, les données compressées sont une entrée non fiable ; la référence les valide (BAD_TABLE, bornes de fenêtre). Hors périmètre pour le projet (pas d'EBC).

## Statut
- Lignes 1 à 95300 lues. Prochaine : ligne 95301.

## Lignes 95301-95600 — APPENDIX J (table des opcodes EBC 0x00-0x3F, machine virtuelle de code octet ; appendice redigé en 2.6) ; APPENDIX K (liste alphabétique, page de titre seule) ; APPENDIX L : changements de noms depuis EFI 1.10 (tables L.1 noms de protocoles et GUID, L.2 identifiants de révision, TPL) ; APPENDIX M : codes de langue RFC 4646 (zh-Hans, zh-Hant, alias zh-chs / zh-cht ; tableaux de langues séparés par ';') ; APPENDIX N : CPER (Common Platform Error Record) : N.1 introduction, N.2 format (en-tête : signature "CPER", révision, Section Count, Error Severity 0 recoverable / 1 fatal / 2 corrected / 3 informational, Validation Bits, Record Length, Timestamp BCD, Platform ID, Partition ID, Creator ID, Notification Type GUIDs : CMC, CPE, MCE, PCIe, INIT, NMI, Boot, DMAr, SEA, SEI...).
- [AGENT] Pertinent : CPER est le format standard d'erreurs matérielles (PCIe, MCE, etc.). Un agent qui journalise des erreurs doit respecter l'en-tête (signature, longueur, sévérité) ; sévérité "Fatal" = à signaler sans tenter de continuer. Les appendices J, K, L, M sont hors périmètre.

## Statut
- Lignes 1 à 95600 lues. Prochaine : ligne 95601.

## Lignes 95601-95900 — N.2 CPER suite : en-tête Record ID / Flags (RECOVERED 1, PREVERR 2, SIMULATED 4) / Persistence Information ; N.2.1.1 types de notification (MCE, CMC, CPE, NMI, PCIe, INIT, BOOT, DMAr, SEA, SEI, PEI, CXL Component) ; N.2.1.2 Error Status (Error_Type, Address/Control/Data, Responder/Requester, First Error, Overflow) et Error Types (ERR_INTERNAL, ERR_BUS, ERR_MEM, ERR_TLB, ERR_CACHE, ERR_FUNCTION, ERR_SELFTEST, ERR_FLOW, ERR_MAP, ERR_IMPROPER, ERR_UNIMPL, ERR_LOL, ERR_RESPONSE, ERR_PARITY, ERR_PROTOCOL, ERR_ERROR, ERR_TIMEOUT, ERR_POISONED) ; N.2.2 Section Descriptor (offset, longueur, révision, validation FRUId/FRUString, flags Primary, Containment Warning, Reset, Threshold exceeded, Resource not accessible, Latent error, Propagated, Overflow ; Section Type GUID : Processor Generic, Processor Specific IA32/X64/IPF/ARM, Platform Memory, PCIe, Firmware Error Record Reference, PCI/PCI-X Bus, PCI Component, DMAr Generic/VT-d/IOMMU, CXL ; FRU Id, Section Severity, FRU Text) ; N.2.3 sections non standard ; N.2.4 sections processeur, N.2.4.1 Generic Processor Error Section (Validation Bits : Type, ISA, Error Type, Operation, Flags, Level, CPU Version, Brand String, CPU Id, Target Address, Requester/Responder Identifier, Instruction IP).
- [AGENT] Pertinent pour journalisation d'erreurs : une section avec le bit "Resource not accessible" ou "Overflow" signale que des champs peuvent être invalides ou que des enregistrements ont été perdus, à reporter tel quel. Un enregistrement CPER doit au minimum contenir une section ; la sévérité Fatal (1) ne doit pas être ignorée. Ne rien écrire en NVRAM pour journaliser sans accord (règle du projet).

## Statut
- Lignes 1 à 95900 lues. Prochaine : ligne 95901.

## Lignes 95901-96200 — N.2.4.1 Generic Processor Error Section (fin : Processor ID = APIC ID / MPIDR_EL1, Target/Requestor/Responder Identifier, Instruction IP) ; N.2.4.2 IA32/X64 Processor Error Section (Local APIC_ID, CPUID Info 48 octets, nombre d'entrées PROC_ERR_INFO_NUM 0-63, PROC_CONTEXT_INFO_NUM) ; N.2.4.2.1 structure d'information d'erreur (GUID de type : Cache Check, TLB Check, Bus Check, MS Check ; Check Info, Target/Requestor/Responder, Instruction Pointer) ; N.2.4.2.2 Cache Check (Transaction Type 0 instruction / 1 donnée / 2 générique ; Operation 0-8 dont data read, data write, instruction fetch, prefetch, eviction, snoop ; Level ; Processor Context Corrupt ; Uncorrected 0 corrigée / 1 non corrigée ; Precise IP ; Restartable IP ; Overflow) ; N.2.4.2.3 TLB Check (mêmes bits, TLB Level) ; N.2.4.2.4 Bus Check (début).
- [AGENT] Pertinent pour interpréter une erreur PCIe/CPU : "Processor Context Corrupt = 1" ou "Uncorrected = 1" doivent faire passer l'agent en mode arrêt sûr, pas en reprise silencieuse ; "Restartable IP = 0" signifie que la reprise n'est pas fiable. Un "Overflow = 1" indique des erreurs perdues : le journal est incomplet.

## Statut
- Lignes 1 à 96200 lues. Prochaine : ligne 96201.

## Lignes 96201-96500 — N.2.4.2.4 Bus Check (fin : Participation Type 0-3, Time Out, Address Space 0 mémoire / 2 E/S / 3 autre) ; N.2.4.2.5 MS Check (Error Type 0 no error, 1 non classé, 2 erreur ROM microcode, 3 externe, 4 FRC, 5 interne ; Processor Context Corrupt, Uncorrected, Precise IP, Restartable IP, Overflow) ; N.2.4.2.6 Processor Context (types 0 à 7 : données non classées, MSR, contexte 32 bits, contexte 64 bits, FXSAVE, DR0-DR7 32/64 bits, registres mappés en mémoire ; tableau des registres IA32 et X64 : EAX..GDTR, IDTR, LDTR, TR, RAX..TR, CR8, RFLAGS) ; N.2.4.3 IA64 renvoyé à SAL ; N.2.4.4 ARM Processor Error Section (type GUID, ERR_INFO_NUM >= 1, CONTEXT_INFO_NUM, Error affinity level 0-3, MPIDR_EL1, MIDR_EL1, Running State, PSCI State, contexte processeur optionnel, info vendeur).
- [AGENT] Pertinent : les contextes de registres peuvent contenir des données sensibles (contexte d'exécution en mode secure) ; ne pas les journaliser ou transmettre sans accord, et ne jamais fournir de contexte d'un niveau secure à un agent non sécurisé (remarque explicite de la spec ARM). Hors périmètre ARM.

## Statut
- Lignes 1 à 96500 lues. Prochaine : ligne 96501.

## Lignes 96501-96800 — N.2.4.4.1 ARM Processor Error Information (Version, Length 32, Type Cache/TLB/Bus/Micro-architectural, Multiple Error count, Flags First/Last/Propagated/Overflow, Virtual Fault Address, Physical Fault Address) ; ARM Cache Error Structure, ARM TLB Error Structure, ARM Bus Error Structure (Participation, Time Out, Address Space, Memory Attributes, Access Mode secure/normal) ; N.2.4.4.1.1 structure micro-architecturale ARM propre au fournisseur ; N.2.4.4.2 ARM Processor Context Information : Header (types 0 à 8 : GPR AArch32/AArch64, registres EL1/EL2/EL3, secure, système), recommandations de contenu selon le niveau de l'agent consommateur (secure world : données sensibles à ne pas exposer ; EL2 : registres visibles en EL2 seulement ; EL1 : registres EL1 seulement).
- [AGENT] Règle reprise : le contexte d'erreur ne doit pas être exposé à un agent moins privilégié que le niveau où l'erreur s'est produite ; pour cet agent (UEFI, ring 0 en pratique), ne remonter que ce qui est nécessaire au diagnostic. Hors périmètre ARM.

## Statut
- Lignes 1 à 96800 lues. Prochaine : ligne 96801.

## Lignes 96801-97100 — N.2.4.4.2 ARM contexte (tables de registres) : Type 0 ARMv8 AArch32 GPRs (R0-R15, SP R13, LR R14, PC R15) ; Type 1 registres système EL1 AArch32 (DFAR, DFSR, IFAR, ISR, MAIR0/1, MIDR, MPIDR, NMRR, PRRR, SCTLR, SPSR et variantes, TPIDR*, TTBCR, TTBR0/1, DACR) ; Type 2 registres système EL2 AArch32 (ELR_hyp, HAMAIR0/1, HCR, HCR2, HDFAR, HIFAR, HPFAR, HSR, HTCR, HTPIDR, HTTBR, SPSR_hyp, VTCR, VTTBR, DACR32_EL2) ; Type 3 registres secure AArch32 (SCTLR (S), SPSR_mon) ; Type 4 GPR AArch64 (X0 à X30, 8 octets chacun).
- [AGENT] Hors périmètre (ARM). Règle reprise : tous les tableaux de registres sont de taille fixe et connue ; vérifier la longueur déclarée avant lecture. Rien à implémenter pour le projet x86-64.

## Statut
- Lignes 1 à 97100 lues. Prochaine : ligne 97101.

## Lignes 97101-97400 — N.2.4.4.2 ARM suite : Type 4 GPR AArch64 (X0 à X30, SP) ; Type 5 registres système EL1 AArch64 (ELR_EL1, ESR_EL1, FAR_EL1, ISR_EL1, MAIR_EL1, MIDR_EL1, MPIDR_EL1, SCTLR_EL1, SP_EL0, SP_EL1, SPSR_EL1, TCR_EL1, TPIDR_EL0/EL1, TPIDRRO_EL0, TTBR0/1_EL1) ; Type 6 registres système EL2 AArch64 (début, décalages 0 à 56).
- [AGENT] Hors périmètre (ARM). Pas d'action pour le projet. Les tableaux de registres sont à décalage fixe de 8 octets en AArch64 : un parseur doit rejeter une longueur qui n'est pas multiple de 8.

## Statut
- Lignes 1 à 97400 lues. Prochaine : ligne 97401.

## Lignes 97401-97700 — N.2.4.4.2 ARM : fin Type 6 (ELR_EL2, ESR_EL2, FAR_EL2, HACR_EL2, HCR_EL2, HPFAR_EL2, MAIR_EL2, SCTLR_EL2, SP_EL2, SPSR_EL2, TCR_EL2, TPIDR_EL2, TTBR0_EL2, VTCR_EL2, VTTBR_EL2) ; Type 7 registres EL3 (ELR_EL3 ... TTBR0_EL3) ; Type 8 registres système divers (encodage MRS : Op2, CRm, CRn, Op1, O0 ; valeur lue) ; Type 9 registres de base de tables 128 bits (TTBR0/1 EL1/EL2/EL3, VTTBR_EL2 ; registre invalide = 128 bits à 1) ; N.2.5 Memory Error Section (GUID A5BC1114... ; Validation Bits 0-21 : Error Status, Physical Address, Mask, Node, Card, Module, Bank, Device, Row, Column, Bit Position, Requestor/Responder/Target ID, Memory Error Type, Rank, Card/Module Handle, Extended Row, Bank Group/Address, Chip ID ; champs : Error Status, Physical Address (et Mask), Node, Card, Module, Bank (Bank Address + Bank Group), Device, Row, Column, Bit Position, Requestor/Responder/Target ID, Memory Error Type 0-15 : Unknown, No error, Single-bit ECC, Multi-bit ECC, ChipKill, Master/Target abort, Parity, Watchdog timeout, Invalid address, Mirror broken, Memory Sparing, Scrub corrected/uncorrected, Map-out).
- [AGENT] Pertinent si l'agent journalise des erreurs mémoire DRAM : "Single-bit ECC" est une correction, "Multi-bit ECC" est une perte de données à signaler ; "Memory Sparing" et "Map-out" changent la topologie mémoire, à ne pas ignorer. Pas d'action immédiate pour le projet.

## Statut
- Lignes 1 à 97700 lues. Prochaine : ligne 97701.

## Lignes 97701-98000 — N.2.5 Memory Error Section (fin : Extended Rank Number, Card Handle, Module Handle SMBIOS Type 16/17) ; N.2.6 Memory Error Section 2 (GUID 61EC04FC... ; champs élargis : Row 4 octets, Column 4, Rank 4, Bit Position 4, Chip Identification, Memory Error Type 0-15 et 16-255 réservés, Status bit 0 = 0 corrigée / 1 non corrigée, Requestor/Responder/Target ID 8 octets, Card/Module Handle 4 octets) ; N.2.7 PCI Express Error Section (GUID D995E954... ; Validation Bits 0-9 : Port Type, Version, Command Status, Device ID (config space), Device Serial Number, Bridge Control Status, Capability Structure Status, AER Info, Device ID RCRB, RCRB High Address ; Port Type 0 Endpoint, 1 Legacy Endpoint, 4 Root Port, 5 Upstream Switch, 6 Downstream Switch, 7 PCIe-to-PCI bridge, 8 PCI-to-PCIe bridge, 9 RCiEP, 10 RCEC ; Version BCD ; Command/Status PCI).
- [AGENT] TRÈS PERTINENT pour l'agent : Port Type 4 (Root Port) et Command/Status PCI sont les champs à relire lors d'une erreur PCIe ; le registre Command ne doit pas être réécrit par l'agent pour "corriger" une erreur sans accord, et les BAR ne doivent pas être reprogrammés (règle du projet déjà notée). Concordance : même vocabulaire que AER (Advanced Error Reporting) du PCIe.

## Statut
- Lignes 1 à 98000 lues. Prochaine : ligne 98001.

## Lignes 98001-98300 — N.2.7 PCIe (fin : RCRB High Address, Device ID, Device Serial Number, Bridge Control Status, Capability Structure 60 octets PCIe 1.1 / 2.0, AER Info 96 octets) ; N.2.8 PCI/PCI-X Bus Error Section (GUID C5753963... ; Error Type : 0 inconnu, 1 Data Parity, 2 System Error, 3 Master Abort, 4 Bus Timeout / No DEVSEL#, 5 Master Data Parity, 6 Address Parity, 7 Command Parity ; Bus Id, Bus Address, Bus Data, Bus Command (bit 0 = PCI-X), Requestor/Completer/Target ID) ; N.2.9 PCI/PCI-X Component Error Section (GUID EB5E4685... ; Id Info, Memory Number et IO Number, paires adresse/donnée 2 à 8 octets) ; N.2.10 Firmware Error Record Reference (type 0 IPF SAL, 1 SOC réservé CrashLog, 2 SOC ; Record Identifier, GUID) ; N.2.11 DMAr : début (section générique GUID 5B51FEF7... : Requester-ID, Segment Number).
- [AGENT] TRÈS PERTINENT : PCI Bus Error Type 0x04 "Bus Timeout / No DEVSEL#" et 0x03 "Master Abort" = la lecture d'un périphérique absent ou mal configuré (ex. BAR mal mappé) ; l'agent doit traiter un tel retour comme une erreur et non comme des données valides (0xFFFFFFFF). La même règle s'applique à ce que lit le projet sur les BAR et l'espace de configuration PCI.

## Statut
- Lignes 1 à 98300 lues. Prochaine : ligne 98301.

## Lignes 98301-98600 — N.2.11.1 DMAr Generic (fin : Fault Reason 1h à Ah, Access Type 0 écriture / 1 lecture, Address Type 0 non traduit / 1 traduction, Architecture 1 VT-d / 2 IOMMU, Device Address 64 bits) ; N.2.11.2 Intel VT-d specific (Version, Revision, OemId, Capability, Extended Capability, Global Command, Global Status, Fault Status, Fault Record, Root Entry, Context Entry, PTE niveaux 1 à 6) ; N.2.11.3 IOMMU AMD specific (Revision, Control, Status, Event Log Entry, Device Table Entry, PTE niveaux 1 à 6) ; N.2.12 CCIX PER Log Error Section (début).
- [AGENT] TRÈS PERTINENT (projet AMD x86-64 avec IOMMU AMD) : la section IOMMU AMD renvoie le Control et Status du registre IOMMU et une entrée d'événement : c'est la base pour lire les erreurs DMA sans les réinitialiser. Ne pas modifier les registres IOMMU (Control, Command) sans accord, car une erreur DMA peut être le signe d'un périphérique réellement hostile ou d'une table mal construite. Concordance avec APM vol2/PPR à vérifier quand ces documents seront lus.

## Statut
- Lignes 1 à 98600 lues. Prochaine : ligne 98601.

## Lignes 98601-98900 — N.2.12 CCIX PER Log (fin : Source ID, Port ID, PER Log en DWORDs) ; N.2.13 CXL Protocol Error Section (GUID 80B9EFB4... ; CXL Agent Type 0 à 7 : RCD RCiEP, RCH Downstream Port RCRB, Endpoint, Logical Device, Fabric-managed, Root Port, Downstream/Upstream Switch Port ; Agent Address, Device ID, Serial Number, Capability Structure 60 octets, CXL DVSEC, CXL Error Log) ; N.2.14 CXL Component Events (renvoie à CXL 2.0) ; N.2.15 FRU Memory Poison Section (GUID 5E4706C1... : checksum de la section à somme nulle ; FRU Architecture x86 CPUID_Fn00000001_EAX ; FRU Identifier PPIN ; descripteurs de poison de 32 octets : timestamp, Hardware Identifier MCA_IPID, Address MCA_ADDR).
- [AGENT] Règle générale : une section dont le checksum ne donne pas zéro doit être rejetée. Une page marquée "poison" (FRU Memory Poison) ne doit jamais être réutilisée par l'agent ni par un chargeur d'image : l'agent doit éviter cette adresse physique dans toute allocation ou tampon DMA.

## Statut
- Lignes 1 à 98900 lues. Prochaine : ligne 98901.

## Lignes 98901-99200 — N.2.15 FRU Memory Poison (fin : adresse Address Type / Address Value) ; APPENDIX O : UEFI ACPI DATA TABLE (en-tête standard ACPI avec signature "UEFI", Length, Revision, Checksum à somme nulle, OEMID, OEM Table ID, Creator ID, DataOffset, GUID identifiant ; O.1 deux méthodes d'invocation SMM : registre d'invocation (GAS, prioritaire) ou numéro SW SMI x86 avec tampon EFI_SMM_COMMUNICATE_HEADER, pointeur de tampon à zéro (opération atomique, CMPXCHG) puis remis à zéro) ; APPENDIX P : persistance des enregistrements d'erreurs matérielles (variable HwErrRecSupport ; QueryVariableInfo avec bit HR ; SetVariable "HwErrRecNNNN" avec GUID EFI_HARDWARE_ERROR_VARIABLE ; EFI_OUT_OF_RESOURCES = espace insuffisant ; écriture par l'OS seulement après ExitBootServices ; effacement par taille 0 ; CreatorId) ; APPENDIX Q : références (début : 802.1X, AMD64 APM, ACPI 3.0/4.0, BIOS Boot, DHCP/BOOTP, CXL, EAP-TTLS, PCI, PCIe 2.1, TLS, UDP, TCP, TFTP, FAT32, PE/COFF, Authenticode, UDF, PC/SC, RFC 1700, 1994, 2131, 2132, 2236, 2246, 2315, 2347...).
- [AGENT] TRÈS PERTINENT : (1) La table "UEFI" (Appendice O) est un moyen de créer une table ACPI sans collision de namespace : à retenir si le projet ajoute une table ACPI personnalisée. (2) Pour le SMM : écriture de la commande SW SMI et du pointeur de tampon uniquement avec accord explicite (ce sont des chemins sensibles). (3) Appendice P : l'agent ne doit écrire dans les variables HwErrRec qu'après ExitBootServices et uniquement pour ses propres enregistrements (CreatorId) ; toute écriture de variable persistante reste soumise à l'accord du projet (NVRAM). (4) Les références Q sont une liste bibliographique.

## Statut
- Lignes 1 à 99200 lues. Prochaine : ligne 99201.

## Lignes 99201-99500 — Q.1 références (suite : RFC 2348, 2349 TFTP ; 2407-2409 IPsec/IKE ; 2454, 2460, 2463 IPv6 ; 2759, 2818 HTTPS, 3004, 3232 (remplace RFC 1700), 3315 DHCPv6, 3396, 3513, 3596, 3617, 3646, 3720 iSCSI, 3748 EAP, 3986 URI, 4173, 4301, 4303, 4346, 4347 DTLS, 4578 PXE options, 5216, 5246 TLS 1.2, 5247, 5281, 5970, 6101 SSL 3.0, 6347 DTLS 1.2 ; SMBIOS 2.6.1 ; USB 2.0, USB 2.0 legacy 0.9, USB BC ; Unicode 5.2 ; ouvrages généraux sur l'IHM et les langages) ; Q.2 prérequis : ACPI (interface principale OS, minimiser les appels runtime vers le firmware) ; Q.2.2 Itanium (SAL, IA64 SDM vol 1 à 3).
APPENDIX R (début, glossaire) : _ADR, _CRS (ressources courantes, requis pour les périphériques non énumérés), _HID (ID PnP compressé 32 bits, requis pour les périphériques énumérés par ACPI), _UID (unique pour un même _HID), ACPI Device Path, Alt-GR, Base Code (PXE), BIOS Boot Specification Device Path, BPB, Block I/O, Block Size (>= 512), Boot Device, Boot Manager, Boot Services Table/Time, Coherency Domain (mémoire + E/S vue par un processeur), ConsoleIn/Out, Device Handle, Device Path (6 types), Device Path Instance / Node, Disconnected (Forms Processor), EBC, EFI File, FAT/FAT12/16/32 (28 bits utiles), GUID Partition Table (GPT, Header LBA 1, backup en dernière LBA), GUID, HII, HTML, IFR, Image Handle, Image Handoff State, Little/Big Endian, LBA (premier LBA = 0), LOM, LFN, MBR (LBA 0), MBR boot code, Media Device Path, MCA.
- [AGENT] Pertinent pour le projet : (1) Le _CRS et le _HID/_UID : l'agent doit lire les ressources via _CRS et non en supposant des adresses fixes (cohérent avec la règle "ne pas reprogrammer les BAR"). (2) Un GPT valide a un header primaire en LBA 1 et un header de secours en dernière LBA ; un disque dont les deux headers diffèrent est corrompu : ne rien écrire sur ce disque sans accord. (3) Un FAT32 n'utilise que 28 bits des numéros de cluster : ne pas traiter les 4 bits de poids fort comme valides. (4) Little Endian : toute structure du projet est en little-endian sauf champs réseau (big-endian).

## Statut
- Lignes 1 à 99500 lues. Prochaine : ligne 99501.

## Lignes 99501-99708 — fin du glossaire (Memory Allocation, Memory Map, Memory Type, Messaging Device Path, Miscellaneous Service, MTFTP, Namespace, Native Code, NBP, NIC, Non-spacing key, NV, Package/Package List, Page Memory, Partition Discovery, PCI Bus Driver, PCI Bus / Configuration Space (ports 0xCF8/0xCFC sur PC-AT), PCI Controller, PCI Device Driver, PCI Device/Function (max 8 fonctions), PCI Enumeration, PCI Host Bus Controller, PCI I/O Protocol, PCI Option ROM, PCI Root Bridge, PCI Segment (_SEG, SAL_PCI_CONFIG_READ/WRITE), Pool Memory, PXE, Protocol, Protocol Handler Services, Question, ROM, Reset, Runtime Services, SAL, scan code, Serial Protocol, SGML, shifted Unicode, Simple File System, Simple Input, Simple Network, Simple Text Output, SKU, SMBIOS, SNIA RAID, SSL, StandardError, Status Codes, string UCS-2, Submit, System Table, Target, TPL, TFTP, Time/Timer/Virtual Memory/Watchdog Services, TLS, UEFI Application / Boot Service Driver / Driver / OS Loader / Runtime Driver, **Unaccepted Memory (AMD SEV-SNP : acceptation de mémoire avant usage)**, UNDI, Unicode Collation, Unicode, USB Bus/Controller/Device/Enumeration/Host Controller (UHCI/OHCI pour USB 1.1)/Hub/I/O Protocol/Interface, Variable Services, VM, Watchdog Timer, WfM, x64, XHTML, XML).
FIN DE LECTURE DE UEFI 2.11 (lignes 1 à 99708, 99708 lignes lues en totalité).
- [AGENT] Points clés retenus de la fin du document : (1) Unaccepted Memory (AMD SEV-SNP) : la mémoire doit être acceptée avant usage par l'invité ; à prendre en compte si l'agent tourne dans une VM confidentielle. (2) PCI Enumeration : c'est le firmware qui attribue les bus, IRQ, E/S, mémoire ; l'agent ne doit pas réattribuer ces ressources. (3) PCI Segment : au-delà de 256 bus, il faut lire le segment (_SEG) ; le projet doit donc utiliser MCFG et non le seul port 0xCF8/0xCFC si le système a plusieurs segments. (4) Le watchdog UEFI doit être désactivé ou géré explicitement avant de rendre la main à l'OS ou à l'agent (sinon reset après 5 min).

## Statut
- UEFI 2.11 : LU EN ENTIER (lignes 1 à 99708). Prochaine étape : UEFI PI (UEFI_PI_Spec__1_.txt), ligne 1.

# UEFI PI 1.10 (UEFI_PI_Spec__1_.txt, 43 663 lignes)

## Lignes 1-200 — Page de titre (UEFI Platform Initialization Specification, Release 1.10, juillet 2026) et table des matières : Partie I PEI (Pre-EFI Initialization) : services PEI (InstallPpi, ReinstallPpi, LocatePpi, NotifyPpi, GetBootMode/SetBootMode, GetHobList/CreateHob, FfsFindNextVolume/File/SectionData, FfsGetFileInfo, FfsGetVolumeInfo, RegisterForShadow, InstallPeiMemory, AllocatePages/Pool, CopyMem, FreePages, SetMem, ReportStatusCode, ResetSystem, I/O et PCI), PEI Foundation (entrée, dispatcher, dépendances DEPEX en notation polonaise inverse, PEIM a priori), PPI architecturales (Master Boot Mode, DXE IPL, Memory Discovered ; optionnelles : Recovery, End of PEI, PEI Reset/Reset2, Status Code, Security, Temporary RAM, CORE_FV_LOCATION), PEIM (structure, descripteurs), PPI supplémentaires (PCI Configuration, Stall, Variable Services, DELAYED DISPATCH, SEC Platform Information, Loaded Image, Recovery, Vector Handoff, CPU I/O, Capsule, Graphics), handoff PEI->DXE par liste HOB, chemins de boot (G0->S0, S-states, recovery), mémoire physique avant/après mémoire permanente, Itanium, phase SEC (responsabilités : reset, RAM temporaire, racine de confiance, HOB), grammaire DEPEX, image TE.
- [AGENT] Retenu : le projet bare-metal n'est pas une phase PEI ; mais la structure de la liste HOB et le handoff vers DXE sont utiles si l'agent lit les HOB fournies par le firmware. Ne pas écrire de PPI ni de HOB sans accord (chemin de boot sensible). Le "Health Flag" (SEC) indique l'état d'auto-test : à lire.

## Statut
- UEFI PI : lignes 1 à 200 lues (table des matières). Prochaine : ligne 201.

## Lignes 201-400 — Table des matières (suite) : I-20 création d'image TE, I-21 chargement TE (XIP, relocalisée, PIC) ; Partie II DXE CIS : II-2 vue d'ensemble (System Table, Boot Services, Runtime Services, DXE Services Table, DXE Foundation, dispatcher, drivers, protocoles architecturaux, protocole Runtime), II-3 Boot Manager, II-4 tables système et exemples d'entrée (application UEFI, drivers non-UEFI résidents/non-résidents, driver UEFI, déchargeable, multi-instances), II-5 Boot Services (événements, groupes prédéfinis, ajouts à LoadImage), II-6 capacités runtime (Status Code runtime), II-7 DXE Services (GCD mémoire/E/S, Dispatch, Schedule, Trust), II-8 Device Path (Firmware Volume / Firmware File), II-9 DXE Foundation (liste HOB, composants requis, DXE_ENTRY_POINT, dépendances, traduction HOB : PHIT, CPU, Resource Descriptor, Firmware Volume, Memory Allocation, GUID Extension), II-10 dispatcher (a priori, DEPEX : BEFORE, AFTER, PUSH, AND, OR, NOT, TRUE, FALSE, END, SOR ; machine d'état ; exemples d'ordre ; considérations de sécurité), II-11 drivers DXE (classes : précoces, modèle UEFI driver), II-12 protocoles architecturaux (BDS, CPU : FlushDataCache, Enable/DisableInterrupt, GetInterruptState, Init, RegisterInterruptHandler, GetTimerValue, SetMemoryAttributes ; Metronome, Monotonic Counter, Real Time Clock, Reset, Runtime, Security/Security2, Timer, Variable, Variable Write, Capsule, Watchdog), II-13 DXE Boot Services Protocol (MP Services : GetNumberOfProcessors, GetProcessorInfo, StartupAllAPs, StartupThisAP, SwitchBSP, EnableDisableAP, WhoAmI), II-14 Status Code Runtime, II-15 grammaire DEPEX, II-16 codes d'erreur, II-17 GUID (DXE Services Table, HOB List) ; Partie III éléments partagés : III-2 stockage firmware (volumes, système de fichiers FFS, fichiers, sections, états), III-3 définitions (PEI FV Info PPI, Load File, Guided Section Extraction, Decompress ; DXE FV2, FV Block2, Guided Section Extraction ; SMM FV), III-4 conception des HOB (liste, règles de construction), III-5 définitions HOB (générique, PHIT, Memory Allocation, BSP stack/BSPSTORE, Module, Resource Descriptor (et v2), GUID Extension, Firmware Volume 1-3, CPU, Memory Pool, UEFI Capsule, Unused, End of HOB list, SMRAM).
- [AGENT] Retenu pour le projet : (1) La liste HOB est la carte de mémoire fournie par le firmware : le projet doit lire les HOB Resource Descriptor pour connaître les régions utilisables, et ne rien allouer dans une région non déclarée. (2) Le protocole MP Services : StartupThisAP / StartupAllAPs sont l'API officielle pour démarrer des cœurs : l'agent ne doit pas envoyer d'INIT/SIPI manuellement sans accord. (3) CPU Arch : SetMemoryAttributes est une opération sensible (cf. notes UEFI 2.11). (4) Les dépendances DEPEX définissent l'ordre d'exécution : un driver ne doit pas supposer un ordre hors DEPEX.

## Statut
- UEFI PI : lignes 1 à 400 lues (TOC). Prochaine : ligne 401.

## Lignes 401-520 — Table des matières (suite) : III-6 codes de statut (classes de progression/erreur, instance, classes matérielles : Computing Unit, User-Accessible Peripheral, I/O Bus ; classes logicielles : Host, sous-classes, RT) ; III-7 routeurs ReportStatusCode (protocole et PPI) ; III-8 PCD (Platform Configuration Database : protocoles et PPI) ; Partie IV Management Mode (MM) : IV-1 vue d'ensemble (MM traditionnel vs MM standalone, initialisation SEC/PEI/DXE, entrée/sortie MM, pilotes MM, SOR/DEPEX, MMI handlers, services MM), IV-2 point d'entrée MM, IV-3 MMST (MmAllocatePool/Pages, MmStartupThisAp, MmInstall/Handle/Locate Protocol, MmiManage, MmiHandlerRegister/UnRegister), IV-4 protocoles MM (Status Code, CPU Save State Read/WriteSaveState, CPU I/O, PCI Root Bridge I/O, Ready To Lock, MP Protocol (DispatchProcedure, BroadcastProcedure, ...), Configuration, End of PEI, UEFI Ready, Ready To Boot, Exit Boot Services, Security Architecture, End of DXE, Handler State Notification), IV-5 protocoles UEFI MM (MM Base InMm/GetMmstLocation, MM Access Open/Close/Lock/GetCapabilities, MM Control Trigger/Clear, MM Configuration, DXE MM Ready to Lock, MM Communication 1/2/3 : Communicate), IV-6 PPI PEI MM (Access, Control, Configuration, Communication 1/3).
- [AGENT] Très pertinent pour la sécurité : (1) SMM / MM : le projet ne doit ni ouvrir ni fermer une zone MM (Open/Close/Lock) ni déclencher un MMI (Trigger) sans accord explicite. (2) Un MM Communication buffer est une entrée non fiable pour le code en MM : toute donnée reçue en MM doit être validée (taille, bornes). (3) Save State : lire le contexte CPU sauvegardé est une fuite potentielle de données : ne rien exposer hors du projet. (4) MM Ready To Lock : après ce point, les zones MM sont verrouillées, ne pas tenter de les écrire.

## Statut
- UEFI PI : lignes 1 à 520 lues (TOC). Prochaine : ligne 521.

## Lignes 521-640 — Table des matières (suite) : IV-7 protocoles de dispatch MM enfants (Software, Sx, Periodic Timer, USB, GPI, Standby Button, Power Button, IO Trap), IV-7.10 HOB EFI_PEI_MM_CORE_GUID ; IV-8 interactions MM avec DXE et PEI (méthodes de communication logicielle MMI n°1 et n°2) ; IV-9 notes sur les types de fichiers MM (EFI_FV_FILETYPE_MM, COMBINED_MM_DXE, MM_STANDALONE) et section MM_DEPEX ; IV-10 gestion MCA/INIT/PMI (handlers et protocole EFI_SAL_MCA_INIT_PMI) ; IV-11 Extended SAL (services I/O, Stall, RTC, Reset, PCI, Cache, PAL, Status Code, compteur monotone, variables, FV block, MCA log, base, MP, MCA) ; IV-12 pile SPI SMM (flash legacy, NOR, configuration, contrôleur, bloc map) ; IV-13 types de compatibilité SMM (SMM_BASE2, capabilities, inside-out, contextes SW/périodique, types IO) ; Partie V Standards : V-2/V-3/V-4/V-5 SMBus Host Controller et PPI SMBus (Execute, ArpDevice, GetArpMap, Notify), V-6 SMBIOS (Add, UpdateString, Remove, GetNext), V-7 IDE controller (init, phases, SubmitData, Disqualify, Calculate, SetTiming, DISK_INFO : Inquiry, Identify, SenseData, WhichIde), V-8 reprise S3 (Boot Script : opcodes IO, MEM, PCI CONFIG, SMBus, STALL, DISPATCH, INFORMATION, Save State Insert/Label/Compare, S3 SMM Save State), V-9 ACPI SDT Protocol (GetAcpiTable, RegisterNotify, Open/OpenSdt/Close/GetChild/GetOption/SetOption/FindPath), V-10 PCI Host Bridge (Resource Allocation Protocol : NotifyPhase, GetNextRootBridge, GetAllocAttributes, StartBusEnumeration, SetBusNumbers, SubmitResources, GetProposedResources, PreprocessController, End of PCI Enumeration ; exemples d'architectures : 1 root bridge, 4 root bridges, 2 segments PCI, 2 bus hosts ; aliasing ISA), V-11 PCI Platform (PlatformNotify, PlatformPrepController, GetPlatformPolicy, GetPciRom, PCI Override, Incompatible PCI Device Support), V-12 Hot Plug PCI Init Protocol.
- [AGENT] Très pertinent : (1) S3 Boot Script : le projet ne doit ni écrire ni rejouer de script S3 (accès I/O, PCI, mémoire à la reprise) sans accord ; (2) V-10.6 : la programmation des registres PCI standard est faite par l'allocateur de ressources PCI, non par un pilote : l'agent ne doit pas reprogrammer BAR/bus (règle déjà notée) ; (3) V-10.4.4 : un système avec 2 segments PCI nécessite la lecture du segment (cohérent avec la note UEFI 2.11 sur _SEG) ; (4) MM : les dispatchers MM sont des points d'entrée à surface d'attaque élevée.

## Statut
- UEFI PI : lignes 1 à 640 lues (TOC, parties I à V.12). Prochaine : ligne 641 (V-12 et suite de la TOC, puis texte normatif à partir de ~ligne 700 à vérifier).

## Lignes 641-760 — Table des matières (fin) : V-12.6 à V-12.8 Hot Plug (GetRootHpcList, InitializeRootHpc, GetResourcePadding, Hot Plug Request Notify) ; V-13 Super I/O (RegisterAccess, GetResources, SetResources, PossibleResources, Modify) ; V-14 interactions Super I/O et ISA (PPI EFI_SIO, EFI_ISA_HC_PPI/PROTOCOL, service binding, EFI_SIO_CONTROL) ; V-15 CPU I/O Protocol 2 (Mem.Read/Write, Io.Read/Write) ; V-16 Legacy Region 2 (Decode, Lock, BootLock, UnLock, GetInfo) ; V-17 pile I2C (bus, design, hot plug ; I2C Master : SetBusFrequency, Reset, StartRequest ; I2C Host : QueueRequest ; I2C I/O : GetDeviceInfo, QueueRequest ; Bus Configuration Management : EnableI2cBusConfiguration, I2cDeviceReset ; Enumerate : Enumerate, GetBusFrequency ; équivalents PEI).
- [AGENT] Pour le projet : (1) CPU I/O et Legacy Region : accès E/S et mémoire legacy passent par des protocoles, ne pas les contourner sans accord. (2) I2C : toute transaction vers des périphériques de la carte (EC, capteurs) est à traiter en lecture seule sauf accord. (3) Super I/O : reconfigurer les ressources de la Super I/O change les ports legacy, donc à éviter.

## Statut
- UEFI PI : lignes 1 à 760 lues (TOC terminée). Prochaine : ligne 761.

## Lignes 761-960 — TOC suite : V-18 pile SPI (bus, couches Application/Peripheral/IO/Bus/Host Controller ; EFI_SPI_CONFIGURATION, CHIP_SELECT, PART, PERIPHERAL, CLOCK, BUS, NOR_FLASH : GetFlashId, ReadData, LfReadData, ReadStatus, WriteStatus, WriteData, Erase ; LEGACY_SPI_FLASH : BiosBaseAddress, ClearSpiProtect, IsRangeProtected, ProtectNextRange, LockController ; SPI_IO : Transaction, UpdateSpiPeripheral ; SPI_HC : ChipSelect, Clock, Transaction ; LEGACY_SPI_CONTROLLER ; NOR_FLASH_BLOCK_MAP) ; V-19 codes d'erreur ; Index ; page de licence (aucune exigence technique) ; "Specification Organization" (volumes I à V) ; historique des révisions 1.0 à 1.10 (listes de tickets, sans contenu normatif).
- [AGENT] Pertinent : (1) SPI NOR flash : écrire ou effacer la flash du firmware est destructif ; le projet n'y touche pas sans accord explicite (règle UEFI/firmware déjà notée). (2) LockController / ClearSpiProtect : désactiver la protection SPI est une opération de sécurité à ne jamais faire sans accord. (3) Le document est sous licence UEFI Forum : ne pas reproduire de passages au-delà de la citation courte.

## Statut
- UEFI PI : lignes 1 à 960 lues. Prochaine : ligne 961 (début de la Partie I : Pre-EFI Initialization Core Interface, contenu normatif).

## Lignes 961-1160 — Listes de figures/tableaux Partie I ; Chapitre 1 principe de terminologie inclusive ; Chapitre 2 organisation du CIS PEI (table des sections : services PEI, PEI Foundation, PEIMs, PPI architecturales et supplémentaires, handoff PEI→DXE, chemins de boot, mémoire physique PEI, SEC, grammaire DEPEX, image TE) ; Chapitre 3 conventions (little-endian obligatoire ; champs réservés : initialiser à zéro, ignorer en lecture, préserver en mise à jour ; formats de description des structures, procédures, instructions DEPEX, PPI) ; Chapitre 4 exigences (une implémentation conforme doit respecter intégralement toute fonction non marquée « optional » ; un composant modulaire non conforme s'il dépend d'une interface hors des spécifications PI) ; Chapitre 5 notations (binaire 0101b, hexadécimal 0xFA23, préfixes SI et binaires KiB/MiB) ; Chapitre 6 vue d'ensemble PEI (invoqué après SEC, pilotes PEIMs, initialisation d'une RAM linéaire, HOB mémoire et FV, passage à DXE ; PEI doit rester minimal : reprise sur crash et reprise S3 rapide) ; 6.2 à 6.8 (PEI Foundation, dispatcher : évaluation des DEPEX de PPI, fin quand plus aucune transition FALSE→TRUE, puis appel de DXE IPL PPI ; PEIMs ; PPI : EFI_PEI_PPI_DESCRIPTOR = paire GUID/pointeur ; architecturales vs supplémentaires ; firmware volumes multiples) ; Chapitre 7 PEI Services Table (EFI_TABLE_HEADER, CRC32 à zéro, pointeur passé à chaque PEIM).
- [AGENT] Retenu : (1) little-endian partout sauf cas réseau (cohérent avec les notes UEFI 2.11). (2) Champs réservés : à préserver en écriture (règle de mise à jour de structure) : l'agent ne doit pas remettre à zéro un champ réservé qu'il ne comprend pas. (3) Phase PEI : le firmware reste sur cache processeur avant mémoire permanente : ne pas supposer de RAM disponible tant que le firmware n'a pas signalé la mémoire.

## Statut
- UEFI PI : lignes 1 à 1160 lues. Prochaine : ligne 1161 (Chapitre 7, PEI Services Table, suite).

## Lignes 1161-1360 — I-7.2 PEI Services Table : PI_SPECIFICATION_VERSION = (1<<16)|100 ; PEI_SERVICES_SIGNATURE 0x5652455320494550 ("PEI SERVIR" en ordre octets) ; structure EFI_PEI_SERVICES (Hdr, InstallPpi, ReInstallPpi, LocatePpi, NotifyPpi, GetBootMode, SetBootMode, GetHobList, CreateHob, FfsFindNextVolume/File/SectionData, InstallPeiMemory, AllocatePages/Pool, CopyMem, SetMem, ReportStatusCode, ResetSystem, CpuIo, PciCfg, FfsFindFileByName, FfsGetFileInfo/2, FfsGetVolumeInfo, RegisterForShadow, FindSectionData3, ResetSystem2, FreePages). I-8 Services PEI : tableau des services ; 8.2 services PPI (InstallPpi : liste terminée par TERMINATE_LIST, au moins un descripteur, codes EFI_INVALID_PARAMETER si liste NULL ou descripteur sans bit PPI, EFI_OUT_OF_RESOURCES si base pleine ; descripteurs NOTIFY_DISPATCH et NOTIFY_CALLBACK ; la liste doit rester en mémoire valide car le pointeur n'est pas copié).
- [AGENT] Retenu : (1) Les services PEI ne sont pas soumis aux restrictions TPL (contrairement aux Boot Services) : sans rapport direct avec l'agent. (2) Un service non encore publié renvoie EFI_NOT_AVAILABLE_YET : l'agent doit gérer ce code sans boucle infinie. (3) Une liste de descripteurs passée par pointeur doit rester valide : règle générale de durée de vie des tampons à respecter dans le projet (ne pas passer l'adresse d'une pile éphémère).

## Statut
- UEFI PI : lignes 1 à 1360 lues. Prochaine : ligne 1361 (I-8.2.2 ReinstallPpi, suite).

## Lignes 1361-1560 — I-8.2.2 ReinstallPpi (OldPpi, NewPpi ; EFI_NOT_FOUND si absent) ; I-8.2.3 LocatePpi (Guid, Instance commençant à 0, boucle jusqu'à EFI_NOT_FOUND ; contrairement à HandleProtocol, pas d'espace de noms de handle) ; I-8.2.4 NotifyPpi (notification déclenchée aussi pour les PPI déjà installés ; EFI_PEI_PPI_DESCRIPTOR_NOTIFY_TYPES) ; I-8.3 Boot Mode (EFI_BOOT_MODE 32 bits : 0x00 full config, 0x01 minimal, 0x02 pas de changement de config, 0x03 full + diagnostics, 0x04 défauts, 0x05 S4 resume, 0x06 S5 resume, 0x07 mode fabrication, 0x10 S2, 0x11 S3 resume, 0x12 flash update, 0x20 recovery ; 0x21-0x0F.. réservé ; SetBootMode modifiable à tout moment en PEI).
- [AGENT] Retenu : (1) BOOT_ON_S3_RESUME (0x11) : le chemin de reprise S3 ne doit pas être modifié par l'agent sans accord (sensible). (2) BOOT_IN_RECOVERY_MODE (0x20) : un agent qui détecte ce mode ne doit pas écrire la flash. (3) Le mode fabrication (0x07) n'est pas un mode de production normal : à signaler si rencontré.

## Statut
- UEFI PI : lignes 1 à 1560 lues. Prochaine : ligne 1561 (suite de SetBootMode, puis GetHobList/CreateHob).

## Lignes 1561-1760 — I-8.4 HOB Services : GetHobList (EFI_NOT_AVAILABLE_YET tant que la liste n'est pas publiée) ; CreateHob (Type, Length arrondi à 8 octets, aucune vérification de Length, EFI_OUT_OF_RESOURCES si plein ; PHIT HOB aligné 8) ; I-8.5 Firmware Volume Services : FfsFindNextVolume (Instance 0 = Boot Firmware Volume ; EFI_PEI_FV_HANDLE unique), FfsFindNextFile (SearchType EFI_FV_FILETYPE_ALL = sans filtre ; FileHandle NULL = début ; EFI_NOT_FOUND si checksum d'en-tête non nul) ; I-8.5.3 FfsFindSectionData (début : SectionType, FileHandle).
- [AGENT] Retenu : (1) Une longueur de HOB non vérifiée par le service : tout lecteur de HOB doit borner la longueur lue avant d'accéder au contenu (même principe que les parseurs de firmware déjà notés). (2) Un en-tête FFS avec checksum non nul doit être rejeté : appliquer la même vérification à toute structure lue du firmware. (3) Les HOB créés avant la mémoire permanente sont dans la RAM temporaire : ne pas en conserver de pointeur au-delà de la phase.

## Statut
- UEFI PI : lignes 1 à 1760 lues. Prochaine : ligne 1761 (FfsFindSectionData, suite).

## Lignes 1761-1960 — I-8.5.3 FfsFindSectionData (fin) : recherche dans les sections d'encapsulation (compressées, GUIDées) mais pas une section GUIDée dans une GUIDée ; pas de recherche dans les sections à extraire si la mémoire est absente ; I-8.5.4 FfsFindSectionData3 (SectionInstance, AuthenticationStatus) ; I-8.5.5 FfsFindFileByName (FileName, VolumeHandle ; EFI_INVALID_PARAMETER si NULL) ; I-8.5.6 FfsGetFileInfo (EFI_FV_FILE_INFO : FileName, FileType, FileAttributes, Buffer valide seulement si EFI_FV_FILE_ATTRIB_MEMORY_MAPPED, BufferSize) ; I-8.5.7 FfsGetFileInfo2 (début).
- [AGENT] Retenu : (1) Le champ AuthenticationStatus renseigne si une section a été authentifiée : l'agent doit refuser de charger une section non authentifiée quand la politique l'exige (Secure Boot). (2) Buffer invalide sans attribut memory-mapped : ne pas déréférencer. (3) Recherche par nom : la correspondance peut être ambiguë si plusieurs volumes contiennent le même nom : vérifier le volume.

## Statut
- UEFI PI : lignes 1 à 1960 lues. Prochaine : ligne 1961 (FfsGetFileInfo2 suite, FfsGetVolumeInfo, RegisterForShadow).

## Lignes 1961-2160 — I-8.5.7 FfsGetFileInfo2 (EFI_FV_FILE_INFO2 : + AuthenticationStatus) ; I-8.5.8 FfsGetVolumeInfo (EFI_FV_INFO : FvAttributes, FvFormat, FvName, FvStart valide si EFI_FVB_MEMORY_MAPPED, FvSize ; EFI_INVALID_PARAMETER si volume absent ou pointeur NULL) ; I-8.5.9 RegisterForShadow (un PEIM est initialisé deux fois : EFI_SUCCESS puis EFI_ALREADY_STARTED ; peut être initialisé après la mémoire permanente selon l'ordre de dispatch) ; I-8.6 PEI Memory Services ; I-8.6.1 InstallPeiMemory (MemoryBegin, MemoryLength : une seule plage contiguë, crée la HOB mémoire ; appels suivants ignorés après un succès ; le PEI Foundation publie ensuite EFI_PEI_PERMANENT_MEMORY_INSTALLED_PPI) ; I-8.6.2 AllocatePages (début).
- [AGENT] Retenu : (1) Une seule plage de mémoire permanente est annoncée au départ : le reste doit venir des HOB mémoire complètes ; l'agent ne doit pas supposer qu'une adresse en dehors de la HOB est utilisable. (2) RegisterForShadow : un composant initialisé deux fois peut avoir des effets de bord (double écriture) : à surveiller en cas d'initialisation de périphérique. (3) FvSize/FvStart : même règle de bornage que pour les autres lectures de firmware.

## Statut
- UEFI PI : lignes 1 à 2160 lues. Prochaine : ligne 2161 (AllocatePages suite, AllocatePool, CopyMem, FreePages, SetMem).

## Lignes 2161-2360 — I-8.6.2 AllocatePages (MemoryType restreint à : LoaderCode/Data, RuntimeServicesCode/Data, BootServicesCode/Data, ACPIReclaimMemory, ReservedMemoryType, ACPIMemoryNVS ; Pages en 4 KiB ; crée une HOB Memory Allocation ; avant InstallPeiMemory = tas, après = région fournie au mieux ; EFI_INVALID_PARAMETER si type illégal) ; I-8.6.3 AllocatePool (multiple de 8 octets, EFI_OUT_OF_RESOURCES) ; I-8.6.4 CopyMem (sans code retour) ; I-8.6.5 FreePages (adresse alignée page, EFI_NOT_FOUND si non alloué par AllocatePages) ; I-8.6.6 SetMem ; I-8.7 Status Code (EFI_NOT_AVAILABLE_YET tant qu'aucun PEIM ne publie le service ; PPI EFI_PEI_PROGRESS_CODE_PPI) ; I-8.7.1 ReportStatusCode (début).
- [AGENT] Retenu : (1) Types de mémoire autorisés : seul ACPIMemoryNVS et ReservedMemoryType sont réservés au système ; un allocataire ne doit pas utiliser ces types pour du tampon de travail. (2) FreePages ne doit libérer que ce qu'il a alloué (EFI_NOT_FOUND sinon) : même règle de propriété des tampons que pour les anneaux DMA. (3) CopyMem/SetMem sans contrôle de bornes : toute longueur doit être validée par l'appelant.

## Statut
- UEFI PI : lignes 1 à 2360 lues. Prochaine : ligne 2361 (ReportStatusCode suite, ResetSystem, I/O et PCI).

## Lignes 2361-2560 — I-8.7.1 ReportStatusCode (paramètres Type, Value, Instance (0 = non applicable, numérotation à partir de 1), CallerId et Data optionnels ; description : point d'entrée réservé aux PEIM, ne doit utiliser aucun service PEI ni PPI d'autres modules, ne rien supposer sur l'allocation mémoire, ne travailler que sur sa pile locale, prudence sur E/S et MMIO car la propriété des ressources est indéterminée pendant la « période de blackout » PEI→DXE ; types EFI_PROGRESS_CODE 1, EFI_ERROR_CODE 2, EFI_DEBUG_CODE 3 ; sévérités MINOR 0x40000000, MAJOR 0x80000000, UNRECOVERED 0x90000000, UNCONTAINED 0xA0000000 ; masques classe 0xFF000000, sous-classe 0x00FF0000, opération 0x0000FFFF ; EFI_STATUS_CODE_DATA : HeaderSize, Size, Type GUID ; EFI_NOT_AVAILABLE_YET si pas de fournisseur) ; I-8.8 Reset Services ; I-8.8.1 ResetSystem (début).
- [AGENT] TRÈS PERTINENT : (1) Une erreur "UNCONTAINED" (0xA0000000) signifie que la donnée défectueuse a pu se propager : l'agent doit la traiter comme non fiable et ne pas la réutiliser. (2) La période de blackout PEI→DXE : l'agent ne doit pas toucher aux E/S ni au MMIO pendant la transition, car la propriété des ressources est indéterminée. (3) Le service de rapport d'erreurs ne doit pas dépendre d'allocation : à reprendre pour le journal d'erreur de l'agent (pile locale, pas de tas).

## Statut
- UEFI PI : lignes 1 à 2560 lues. Prochaine : ligne 2561 (ResetSystem suite, I/O et PCI Services).

## Lignes 2561-2760 — I-8.8.1 ResetSystem (réinitialise toute la plateforme, processeurs et périphériques ; cas d'usage : changer la fréquence CPU, redémarrer pour finir l'init du chipset, réponse à une erreur catastrophique ; EFI_NOT_AVAILABLE_YET tant que non installé) ; I-8.9 CPU I/O et PCI Config publiés par le PEI Foundation ; CHAPITRE 9 PEI FOUNDATION : I-9.1 (binaire EFI_FV_FILETYPE_PEI_CORE : section d'authentification + code PE32+ ou brut ; découvert et authentifié par SEC ; prérequis : mode d'exécution et accès au FV contenant le PEI Foundation), I-9.1.2 modes par architecture (IA-32 « flat 32 » ring 0 ; Itanium SAL_E/PAL-A/PAL-B), I-9.1.2.3 accès au BFV : le BFV ne peut être que de type EFI_FIRMWARE_FILE_SYSTEM2_GUID ; le PEI Foundation et les PEIM de reprise doivent être dans un FV non modifiable ou une mise à jour tolérante aux pannes (ancienne ou nouvelle version toujours valide) ; I-9.2 entrée PEI (EFI_SEC_PEI_HAND_OFF : DataSize, BootFirmwareVolumeBase/Size, TemporaryRamBase/Size, PeiTemporaryRamBase/Size, StackBase/Size ; la zone PEI RAM ne doit pas sortir de la RAM temporaire ni chevaucher la pile ; PPI SEC : EFI_SEC_PLATFORM_INFORMATION_PPI, EFI_SEC_HOB_DATA_PPI) ; I-9.3 convention d'appel ; I-9.4 récupération du pointeur EFI_PEI_SERVICES (x86 : 4 octets avant la base IDT, lue par SIDT ; règles IDT : la SEC initialise l'IDT via LIDT et garde les 4 octets précédents en mémoire temporaire).
- [AGENT] Retenu : (1) La BFV doit être authentifiée ; le projet ne doit pas écrire la BFV ni modifier son FV sans accord (chaîne de confiance). (2) Le pointeur de services PEI est stocké dans les 4 octets avant l'IDT : hors périmètre direct mais à savoir si l'agent lit l'IDT. (3) Le mode d'exécution de départ est ring 0 à plat : l'agent UEFI fonctionne dans les mêmes conditions. (4) Les zones de RAM temporaire ne doivent pas se chevaucher avec la pile : vérifier cette règle si l'agent alloue sa propre pile.

## Statut
- UEFI PI : lignes 1 à 2760 lues. Prochaine : ligne 2761 (I-9.4.1.1 règles IDT, puis I-9.4.2 x64).

## Lignes 2761-2960 — I-9.4.1.1 / I-9.4.2.1 règles IDT x86 et x64 (SEC initialise IDT via LIDT avec le champ de 4 ou 8 octets précédant la base en mémoire temporaire ; un PEIM qui réinstalle l'IDT doit recopier ce champ) ; I-9.4.3 IPF (ar.kr7) ; I-9.4.4 ARM (TPIDRURW) et vecteurs ARM (LDR pc, [pc,#0x20]) ; I-9.4.5 AArch64 (TPIDR_EL0) ; I-9.4.6 RISC-V (SSCRATCH) ; I-9.4.7 LoongArch (CSR KS0) ; I-9.5 dispatcher PEI (examine EFI_FV_FILETYPE_PEIM et COMBINED_PEIM_DRIVER, DEPEX et a priori) ; I-9.6 ordre : ordre faible, a priori file (GUID 1b45cc0a-156a-428a-af62-49864da0e6e6, RAW, DEPEX ignorées pour les PEIM listés, authentification via SECURITY2 si présent) ; I-9.6.4 FV imbriqués (EFI_SECTION_PEI_DEPEX) ; I-9.7 DEPEX : pile polonaise postfixée, opcodes 8 bits, dépassement ou sous-dépassement de pile = FALSE, opcode invalide = FALSE, END absent = FALSE, limite 256 termes ; table des opcodes ; PUSH <GUID> (0x02, GUID 16 octets, LocatePpi), AND (0x03).
- [AGENT] Retenu : (1) Une expression mal formée (END absent, pile en débordement, opcode inconnu) doit être évaluée à FALSE : règle de sûreté à reprendre dans tout interpréteur de conditions du projet (ne jamais autoriser une condition indéfinie à valoir TRUE). (2) Les GUID dans la DEPEX peuvent être non alignés : accès octet par octet sur les architectures stricte. (3) IDT : le champ précédant la base IDT contient un pointeur : ne pas écraser cette zone si l'agent réinstalle l'IDT (cohérent avec la règle du projet sur les tables).

## Statut
- UEFI PI : lignes 1 à 2960 lues. Prochaine : ligne 2961 (suite AND/OR/NOT/TRUE/FALSE/END/SOR).

## Lignes 2961-3160 — I-9.7.1.1 opcodes DEPEX : OR 0x04, NOT 0x05, TRUE 0x06, FALSE 0x07, END 0x08 (dernier opcode obligatoire) ; I-9.7.2 DEPEX vide = TRUE ; I-9.7.3 section absente = DEPEX vide ; I-9.7.4 BNF RPN ; I-9.8 algorithme de dispatch : balayage répété jusqu'à une passe sans dispatch ; exposition de FV (EFI_PEI_FIRMWARE_VOLUME_INFO_PPI, HOB FV, FV ignoré si format non supporté) ; exigences : ordre faible, pas de boucle infinie (cycle AcXpY / BcYpX irrésoluble), économie de registres, ordre CBDA sur l'exemple AcQpZ BcLpR CpL DcRpQ ; mémoire : pile et tas temporaires avant InstallPeiMemory, aucune écriture au-delà avant ; après installation copie de la pile, du tas, de la liste HOB et des FV vers la mémoire permanente avec correction des pointeurs PPI ; DXE IPL PPI appelé après le dispatch ; entrée PEIM : pointeur indirect vers les services PEI et handle du fichier ; rapport de la position du cœur PEI via EFI_PEI_LOADED_IMAGE_PPI ; exemple de dispatch (carte DispatchedBitMap).
- [AGENT] Retenu : (1) Un cycle de dépendances est un blocage : l'agent doit détecter une attente circulaire et s'arrêter (pas de boucle infinie) : même principe que les timeouts de file. (2) Une expression DEPEX non terminée par END est FALSE : cohérent avec la règle de sûreté notée plus haut. (3) La copie vers la mémoire permanente corrige les pointeurs : tout pointeur gardé par l'agent avant une migration de mémoire doit être recalculé (ne pas garder d'adresses de la RAM temporaire). (4) Le DXE IPL doit trouver son PPI : l'échec est une erreur, pas un cas silencieux.

## Statut
- UEFI PI : lignes 1 à 3160 lues. Prochaine : ligne 3161 (suite I-9.8.4 mémoire et dispatch, I-9.8.5 PEIM dispatching, I-9.8.6 authentification PEIM).

## Lignes 3161-3360 — I-9.8.5 dispatch d'un PEIM (1 : EFI_PEI_LOAD_FILE_PPI ; 2 : sinon PE32+/TE XIP ; 3 : EFI_PEI_SECURITY2_PPI, sauf EFI_SECURITY_VIOLATION qui marque le PEIM comme dispatché sans l'exécuter ; 4 : appel du point d'entrée ; 5 : marquage) ; I-9.8.6 authentification (3 méthodes : section GUIDée via GUIDED_SECTION_EXTRACTION_PPI, LOAD_FILE_PPI, signature numérique UEFI 2.0 ; résultat transmis à SECURITY2) ; I-10 PPI architecturales : I-10.2.1 Master Boot Mode PPI (GUID 7408d748...; racine des DEPEX des PEIM qui modifient le matériel), I-10.2.2 DXE IPL PPI (GUID ae8ce5d-e448-4437-a8d7-ebf5f194f731 ; dernière PPI exécutée ; état attendu : un seul thread, interruptions désactivées, mode PEI ; ne doit jamais revenir ; peut lancer DXE, recovery, S3 ; en S3 doit fonctionner sans mémoire permanente) ; EFI_PEI_HOB_POINTERS (union des types HOB) ; I-10.2.3 Permanent Memory Installed PPI (GUID f894643d-c449-42d1-8ea8-85bdd8c65bde) ; I-10.3.1 Recovery Mode PPI (GUID 17ee496a-d8e4-4b9a-94d1-ce8272308 50 ; signal sans interface) ; I-10.3.2 End of PEI PPI (GUID 605EA650-C65C-42e1-BA80-91A52AB618C6 ; installée à la fin de PEI, y compris en S3 juste avant le vecteur de réveil OS) ; I-10.3.3 PEI Reset PPI (GUID ef398d58-9dfd-4103-bf94-78c6f4fe712f) ; I-10.3.4 PEI Reset2 PPI (GUID 6cc45765-cce4-42fd-bc56-011aaac6c9a8).
- [AGENT] Retenu : (1) EFI_SECURITY_VIOLATION : un module qui échoue l'authentification ne doit jamais être exécuté : l'agent applique la même règle à tout code chargé. (2) Le DXE IPL est dernier : tout état de PEI doit être terminé avant. (3) End of PEI : moment où la mémoire passe à DXE : l'agent ne doit pas utiliser de mémoire PEI après cette PPI. (4) Un reset depuis PEI passe par PPI : pour l'agent, un reset est une opération destructive à ne pas déclencher sans accord.

## Statut
- UEFI PI : lignes 1 à 3360 lues. Prochaine : ligne 3361 (suite Reset2, Status Code PPI, Security PPI, Temporary RAM PPI, CORE_FV_LOCATION).

## Lignes 3361-3560 — I-10.3.4 Reset2 (ResetType : EfiResetCold 0, EfiResetWarm 1, EfiResetShutdown 2, EfiResetPlatformSpecific 3 ; ResetStatus ; ResetData = chaîne NUL + données binaires ; Cold = cycle complet ; Warm = processeurs réinitialisés, cycles en cours non corrompus, sinon Cold ; Shutdown = état ACPI G2/S5 ou G3 ; PlatformSpecific = GUID après la chaîne ; ResetSystem ne revient pas) ; I-10.3.5 Progress Code PPI (GUID 229832d3-7a30-4b36-b827-f40cb7d45436, singleton, notifie à l'installation) ; I-10.3.6 Security PPI (GUID dcd0be23-9586-40f4-b643-06522ced4ede ; AuthenticationState : AuthenticationStatus, FvHandle, FileHandle, DeferExecution ; absent = PEIM considérés comme de confiance ; politique laissée à la plateforme, peut bloquer ou simplement journaliser ; EFI_SECURITY_VIOLATION = objet non fiable) ; I-10.3.7 Temporary RAM Support PPI (GUID dbe23aa9-a345-4b97-85b6-b226f1617389 ; migration de la RAM temporaire vers la RAM permanente, TEMPORARY_RAM_MIGRATION).
- [AGENT] TRÈS PERTINENT : (1) Politique de reset : Cold et Shutdown coupent l'alimentation ou l'équivalent : toute écriture non finalisée est perdue ; l'agent ne doit pas déclencher un reset sans accord explicite. (2) Un PEIM dont l'authentification échoue peut être journalisé ou bloqué : l'agent doit de même refuser de charger un module non authentifié. (3) La migration RAM temporaire -> permanente exige que les pointeurs soient recalculés (règle déjà notée).

## Statut
- UEFI PI : lignes 1 à 3560 lues. Prochaine : ligne 3561 (suite Temporary RAM Support PPI, Temporary RAM Done, CORE_FV_LOCATION, puis chapitre I-11 PEIM).

## Lignes 3561-3760 — I-10.3.7.2 TemporaryRamMigration (TemporaryMemoryBase, PermanentMemoryBase, CopySize ; appelé après InstallPeiMemory et avant EFI_PEI_PERMANENT_MEMORY_INSTALLED_PPI ; la zone permanente doit tenir dans la mémoire confiée à InstallPeiMemory ; si la PPI est absente, le PEI Foundation copie directement et bascule la pile ; EFI_INVALID_PARAMETER si la destination chevauche la source) ; I-10.3.8 Temporary RAM Done PPI (GUID ceab683c-ec56-4a2d-a906-4053fa4e9c16 ; TemporaryRamDone désactive la RAM temporaire après la migration ; EFI_DEVICE_ERROR si impossible) ; I-10.3.9 EFI_PEI_CORE_FV_LOCATION_PPI (GUID 52888eae-5b10-47d0-a87f-b822aba0caf4 ; indique le FV contenant le PEI Foundation ; sinon hypothèse BFV) ; CHAPITRE 11 PEIMS : I-11.1 introduction ; I-11.2 structure (en-tête, code XIP, relocations optionnelles, authentification) ; I-11.2.2 code position-dépendant (PE32 lié à l'adresse zéro, relocations défaites pour vérifier la signature) et indépendant (point d'entrée en tête de section, cas Itanium) ; I-11.2.3 authentification via section GUIDée (hash, checksum ou CRC) ; I-11.3.1 EFI_PEIM_ENTRY_POINT2 (appelé une fois ; l'argument 2 indique PEI ou DXE selon la signature : EFI_SYSTEM_TABLE_SIGNATURE ou PEI_SERVICES_SIGNATURE) ; I-11.4.1 descripteurs PEIM (drapeaux, pointeur GUID, pointeur de données ; double rôle : interface pour PEIM et callback pour le Foundation).
- [AGENT] Retenu : (1) Un module PE/COFF lié à l'adresse zéro doit être recalculé avant vérification : la vérification d'intégrité d'un firmware doit annuler les relocations, sinon elle échoue à tort ou réussit à tort. (2) Un même binaire peut être PEI et DXE : le code doit tester la signature de la table reçue avant d'utiliser ses services (même principe que la vérification d'un en-tête avant lecture). (3) La migration de RAM temporaire ne doit jamais faire chevaucher source et destination.

## Statut
- UEFI PI : lignes 1 à 3760 lues. Prochaine : ligne 3761 (I-11.4.2 EFI_PEI_DESCRIPTOR, suite).

## Lignes 3761-3960 — I-11.4.2 EFI_PEI_DESCRIPTOR (union notify/PPI) ; I-11.4.3 EFI_PEI_NOTIFY_DESCRIPTOR (Flags, Guid, Notify ; callback immédiat vs dispatch différé après retour du PEIM : pile minimale) ; I-11.4.4 EFI_PEI_PPI_DESCRIPTOR (Flags : PIC 0x01, PPI 0x10, NOTIFY_CALLBACK 0x20, NOTIFY_DISPATCH 0x40, NOTIFY_TYPES 0x60, TERMINATE_LIST 0x80000000) ; I-11.5 communication entre PEIM : base PPI opaque, LocatePpi/InstallPpi/NotifyPpi, DEPEX évaluée par présence des GUID ; deux façons d'obtenir une PPI (DEPEX ou notification) ; une PPI ne peut pas être désinstallée (on la réinstalle en version qui renvoie un échec) ; danger de mise en cache d'une PPI qui peut être réinstallée (données périmées après shadowing) ; PPI Stall XIP et version en mémoire séparées ; CHAPITRE 12 PPI supplémentaires : I-12.1 (requises : CPU I/O, PCI Configuration, Stall, PEI Variable Services ; optionnelle : SEC Platform Information).
- [AGENT] Retenu : (1) Une PPI ne peut pas être retirée : une ressource exposée reste accessible ; l'agent ne doit pas supposer qu'une fonction absente l'est réellement. (2) La mise en cache d'un pointeur de service devient fausse si la PPI est réinstallée : l'agent doit relire la PPI après chaque changement de phase ou de mémoire. (3) Les flags de notification ont un compromis pile/délai : à garder en tête si l'agent utilise un callback imbriqué.

## Statut
- UEFI PI : lignes 1 à 3960 lues. Prochaine : ligne 3961 (I-12.2.1 PCI Configuration PPI).

## Lignes 3961-4160 — I-12.2.1 PCI Configuration PPI (EFI_PEI_PCI_CFG2_PPI, GUID 057a449a-1fdc-4c06-bfc9-f53f6a99bb92 ; une instance par segment PCI ; segment 0 pointé par la table PEI Services ; recherche des autres segments par LocatePpi et comparaison du champ Segment) ; Read/Write/Modify (Modify = lecture + modification + écriture) ; largeurs 1, 2, 4, 8 octets ; adresse EFI_PEI_PCI_CFG_PPI_PCI_ADDRESS (Register 8 bits, Function 0-7, Device 0-31, Bus 0-255, ExtendedRegister 32 bits si Register >= 256) ; macro EFI_PEI_PCI_CFG_ADDRESS(bus,dev,func,reg) ; codes EFI_SUCCESS, EFI_DEVICE_ERROR, EFI_DEVICE_NOT_READY.
- [AGENT] TRÈS PERTINENT (projet PCI) : (1) Un accès config PCI retourne EFI_DEVICE_ERROR : l'agent doit traiter cela comme une erreur, pas comme une valeur. (2) La méthode Modify fait une lecture-puis-écriture : sur un registre de commande ou d'état, elle peut déclencher un effet ; à éviter sans accord. (3) Les segments au-delà de 0 doivent être trouvés par leur champ Segment, pas supposés : cohérent avec la note MCFG/_SEG déjà relevée.

## Statut
- UEFI PI : lignes 1 à 4160 lues. Prochaine : ligne 4161 (suite I-12.2.1.3 Write, Modify, puis Stall PPI).

## Lignes 4161-4360 — I-12.2.1.3 Write et I-12.2.1.4 Modify (SetBits OR, ClearBits AND-NOT, tailles selon Width) ; I-12.2.2 Stall PPI (GUID 1f4c6f90-b06b-48d8-a201-bae5f1cd7d56 ; Resolution en microsecondes ; Stall bloquant pendant au moins le délai demandé) ; I-12.2.3 Variable Services PPI en lecture seule (EFI_PEI_READ_ONLY_VARIABLE2_PPI, GUID 2ab86ef5-ecb5-4134-b556-3854ca1fe1b4 ; GetVariable : VariableName, VariableGuid, Attributes, DataSize, Data ; EFI_BUFFER_TOO_SMALL avec la taille requise ; NextVariableName : parcours avec chaîne vide au départ, EFI_NOT_FOUND en fin de liste) ; I-12.2.4 EFI DELAYED DISPATCH PPI (GUID 869c711d-649c-44fe-8b9e-2cbb2911c3e6, service de temporisation en PEI).
- [AGENT] Retenu : (1) Variables en PEI : lecture seule uniquement ; l'agent ne doit pas écrire dans le stockage de variables (NVRAM) sans accord. (2) GetVariable : toujours vérifier EFI_BUFFER_TOO_SMALL et réallouer la taille indiquée, pas une taille fixe. (3) Le Stall est bloquant : à ne pas utiliser dans une boucle de service sans borne.

## Statut
- UEFI PI : lignes 1 à 4360 lues. Prochaine : ligne 4361 (suite DELAYED DISPATCH PPI).

## Lignes 4361-4560 — I-12.2.4 DELAYED DISPATCH PPI (Register : Function, Context 64 bits, DelayedGroupId optionnel, Delay en microsecondes ; delay 0 = exécution immédiate ; la fonction met à jour Context et NewDelay : NewDelay 0 désinscrit, sinon rappel après ce délai ; WaitOnEvent : met en pause le composant jusqu'à ce que tout le groupe soit terminé ; EFI_OUT_OF_RESOURCES si plus d'entrées) ; I-12.3 PPI optionnelles : I-12.3.1 SEC Platform Information PPI (GUID 6f8c2b35-fef4-448d-8256-e11b19d61077 ; PlatformInformation : StructureSize et EFI_SEC_PLATFORM_INFORMATION_RECORD ; union IA32/X64/Itanium HANDOFF_STATUS ; état de santé du processeur au reset (BIST sur IA-32, PAL-A sur Itanium) ; le BSP peut être choisi autrement si l'état est mauvais).
- [AGENT] Retenu : (1) Delayed dispatch : remplace les boucles d'attente actives par des rappels : pour l'agent, une attente doit être bornée ou rappelée, pas une boucle tournante sans fin. (2) Le Health Flag indique l'état du BSP au reset : à lire et à journaliser si le projet tourne sur le BSP. (3) Choix d'un autre BSP en cas d'erreur grave : opération de topologie à ne pas faire sans accord.

## Statut
- UEFI PI : lignes 1 à 4560 lues. Prochaine : ligne 4561 (suite HANDOFF_STATUS/Health Flags, SEC Platform Information 2, Loaded Image PPI).

## Lignes 4561-4760 — EFI_HEALTH_FLAGS (champs de bits Status, Tested, Reserved1, VirtualMemoryUnavailable, Ia32ExecutionUnavailable, FloatingPointUnavailable, MiscFeaturesUnavailable, Reserved2) ; IA32_HANDOFF_STATUS et X64_HANDOFF_STATUS = EFI_HEALTH_FLAGS ; ARM : pas d'instance ; ITANIUM_HANDOFF_STATUS (BootPhase : 0 = RESET/Normal, 3 = Recovery_Check ; FWStatus ; ProcId ; IdMask ; EidMask ; PalCallAddress ; PalSpecialAddress ; SelfTestStatus ; SelfTestControl ; MemoryBufferRequired) ; I-12.3.2 SEC Platform Information 2 PPI (GUID 9e9f374b-8f16-4230-9824-5846ee766a97 ; multi-CPU : EFI_SEC_PLATFORM_INFORMATION_CPU = CpuLocation (APIC ID) + record, NumberOfCpus ; EFI_BUFFER_TOO_SMALL avec la taille requise) ; I-12.3.3 Loaded Image PPI (début).
- [AGENT] TRÈS PERTINENT (projet AMD x86-64) : (1) Les bits Ia32ExecutionUnavailable, FloatingPointUnavailable et VirtualMemoryUnavailable indiquent un défaut matériel au reset : l'agent doit les relire et refuser de démarrer si un bit critique est levé. (2) Tested = 0 : le BIST n'a pas été exécuté : ne pas confondre avec un succès. (3) Le CPU Location est l'APIC ID : cohérent avec les notes MADT/ACPI déjà faites.

## Statut
- UEFI PI : lignes 1 à 4760 lues. Prochaine : ligne 4761 (suite Loaded Image PPI, SEC HOB, Recovery, Recovery Block I/O2, Vector Handoff, CPU I/O, Capsule, Graphics).

## Lignes 4761-4960 — I-12.3.3 Loaded Image PPI (GUID c1fcd448-6300-4458-b864-28df015364bc ; ImageAddress, ImageSize, FileHandle ; installée après chargement et contrôles de sécurité ; un même PEIM peut être initialisé deux fois) ; I-12.3.4 SEC HOB Data PPI (GUID 3ebdaf20-6667-40d8-b4ee-f5999ac1b71f ; GetHobs renvoie des HOB terminés par END_OF_HOB_LIST, sans PHIT, alignés 8 octets ; EFI_UNSUPPORTED si aucune) ; I-12.3.5 Recovery (table : Recovery Module PPI, Device Recovery Module PPI, Device Recovery Block I/O PPI) ; I-12.3.5.1 Recovery Module PPI (GUID FB6D9542-612D-4f45-872F-5CFF52E93DCF ; LoadRecoveryCapsule ; étapes : compter les capsules, allouer MaxRecoveryCapsuleSize ou plus, charger, contrôler la sécurité, journaliser les échecs dans une HOB, décomposer la capsule, invalider les entrées HOB de FV mises à jour pour qu'aucun pilote suspect ne s'exécute, mettre à jour la HOB, revenir au dispatcher) ; LoadRecoveryCapsule (EFI_SUCCESS, EFI_DEVICE_ERROR, EFI_NOT_FOUND) ; I-12.3.5.2 Device Recovery Module PPI (GUID 0DE2CE25-446A-45a7-BFC9-37DA26344B37 ; GetNumberRecoveryCapsules, GetRecoveryCapsuleInfo, LoadRecoveryCapsule ; capsule 1 = priorité la plus haute).
- [AGENT] TRÈS PERTINENT : (1) La récupération efface les entrées HOB des FV mis à jour pour empêcher l'exécution de pilotes douteux : l'agent doit rejeter tout FV qui n'est pas dans la liste de confiance courante. (2) Une capsule dont la sécurité échoue doit être journalisée et ignorée, pas exécutée. (3) Aucune écriture de firmware, de capsule ou de flash de récupération sans accord explicite (règle du projet). Le Recovery Block I/O est cité plus bas dans le document.

## Statut
- UEFI PI : lignes 1 à 4960 lues. Prochaine : ligne 4961 (GetNumberRecoveryCapsules suite, GetRecoveryCapsuleInfo, LoadRecoveryCapsule, Recovery Block I/O).

## Lignes 4961-5160 — I-12.3.5.2.2 GetNumberRecoveryCapsules (nombre de capsules ; entrée 1 = priorité la plus haute) ; I-12.3.5.2.3 GetRecoveryCapsuleInfo (CapsuleInstance entre 1 et N, Size, CapsuleType GUID défini par l'implémentation) ; I-12.3.5.2.4 LoadRecoveryCapsule (Buffer fourni par l'appelant) ; I-12.3.5.3 Device Recovery Block I/O PPI (lecteurs legacy floppy, CD-ROM, IDE ; GUID 695d8aa1-42ee-4c46-805c-6ea6bce799e3) ; GetNumberOfBlockDevices (0 si aucun périphérique) ; GetBlockDeviceMediaInfo (début).
- [AGENT] Retenu : (1) Les capsules de récupération sont listées par priorité : l'agent ne doit pas réordonner ni sauter une capsule sans journal. (2) Le Block I/O de récupération lit des disques : cohérent avec la règle du projet (aucune écriture sur clé ou disque interne sans accord). (3) Une capsule dont la taille dépasse le tampon fourni ne doit pas être tronquée silencieusement.

## Statut
- UEFI PI : lignes 1 à 5160 lues. Prochaine : ligne 5161 (GetBlockDeviceMediaInfo suite, ReadBlocks).

## Lignes 5161-5360 — I-12.3.5.3.3 GetBlockDeviceMediaInfo (DeviceIndex 1..N ; EFI_PEI_BLOCK_IO_MEDIA : DevType, MediaPresent, LastBlock, BlockSize ; types : LegacyFloppy 0, IdeCDROM 1, IdeLS120 2, UsbMassStorage 3, SD 4, EMMC 5, UfsDevice 6) ; I-12.3.5.3.4 ReadBlocks (StartLBA 64 bits, BufferSize multiple de la taille de bloc, EFI_NO_MEDIA, EFI_BAD_BUFFER_SIZE, lecture complète ou erreur) ; I-12.3.6 Recovery Block IO2 PPI (GUID 26cc0fad-beb3-478a-91b2-0c188f726198, Revision 0x00010000, mêmes trois fonctions avec variantes 2).
- [AGENT] Retenu : (1) Lecture bloc : toute lecture doit vérifier que BufferSize est multiple de la taille de bloc ; un tampon mal aligné doit être refusé (EFI_BAD_BUFFER_SIZE), pas corrigé en silence. (2) L'ordre de priorité des périphériques de récupération est une politique : ne pas le modifier sans accord. (3) Les types USB et SD/eMMC/UFS montrent que la récupération peut lire sur ces supports : mêmes règles de lecture seule que pour le projet.

## Statut
- UEFI PI : lignes 1 à 5360 lues. Prochaine : ligne 5361 (suite Recovery Block IO2, puis EFI PEI Vector Handoff Info PPI).

## Lignes 5361-5560 — I-12.3.6.2/3/4 Recovery Block IO2 (GetNumberOfBlockDevices2, GetBlockDeviceMediaInfo2 : EFI_PEI_BLOCK_IO2_MEDIA = InterfaceType (sous-type Messaging Device Path, ex. 5 pour USB), RemovableMedia, MediaPresent, ReadOnly, BlockSize (UINT32), LastBlock ; ReadBlocks2 : mêmes codes d'erreur) ; I-12.3.7 EFI_PEI_VECTOR_HANDOFF_INFO_PPI (GUID 3cd652b4-6d33-4dce-89db-83df9766fcca ; tableau de vecteurs établis en SEC, terminé par EFI_VECTOR_HANDOFF_LAST_ENTRY ; attributs : DO_NOT_HOOK (gestionnaire à conserver), HOOK_BEFORE (nouveaux gestionnaires avant l'existant), HOOK_AFTER (après) ; table système EFI_VECTOR_HANDOFF_TABLE_GUID 996ec11c-5397-4e73-b58f-827e52906def, en mémoire EfiBootServicesData).
- [AGENT] TRÈS PERTINENT (IDT/exceptions) : (1) Un vecteur marqué DO_NOT_HOOK ne doit jamais être remplacé par l'agent : c'est une règle explicite à respecter dans tout code qui installe ses propres gestionnaires d'interruption. (2) HOOK_BEFORE / HOOK_AFTER imposent un ordre d'appel : l'agent qui s'enchaîne sur un vecteur doit respecter cet ordre. (3) InterfaceType et ReadOnly : un périphérique marqué ReadOnly ne doit recevoir aucune écriture, même par erreur de logique.

## Statut
- UEFI PI : lignes 1 à 5560 lues. Prochaine : ligne 5561 (suite de la table vecteurs, EFI PEI CPU I/O, Capsule, Graphics).

## Lignes 5561-5760 — I-12.3.7 suite : EFI_VECTOR_HANDOFF_INFO (VectorNumber, Attribute, Owner GUID — unique propriétaire si DO_NOT_HOOK) ; I-12.3.8 CPU I/O PPI (GUID indicateur e6af1f7b-fc3f-46da-a828-a3b457a44282 sans données ; sinon fonctions factices renvoyant EFI_NOT_YET_AVAILABLE ; structure : Mem, Io (chacun Read/Write), IoRead8/16/32/64, IoWrite8/16/32/64, MemRead8/16/32/64, MemWrite8/16/32/64 ; largeurs incl. FIFO et FILL ; I-12.3.8.2 Mem() (Address, Count, Buffer) ; I-12.3.8.3 Io() ; I-12.3.8.4 IoRead8 (début).
- [AGENT] TRÈS PERTINENT (E/S et MMIO) : (1) Les accès FIFO répètent la même adresse (lecture d'un port de données) : une erreur de largeur ou de compte envoie des accès parasites au périphérique. (2) Les accès mémoire et E/S du CPU ne doivent pas être utilisés pour écrire dans un périphérique sans accord (règle du projet). (3) EFI_NOT_YET_AVAILABLE : l'agent doit attendre la PPI, pas supposer un accès direct.

## Statut
- UEFI PI : lignes 1 à 5760 lues. Prochaine : ligne 5761 (suite IoRead8 à MemWrite64, puis Capsule PPI).

## Lignes 5761-5960 — I-12.3.8.4 à I-12.3.8.9 : IoRead8/16/32/64 (lecture d'E/S à une adresse physique, valeur renvoyée), IoWrite8/16 (écriture ; retour VOID) ; I-12.3.8.10 IoWrite32 (début).
- [AGENT] Retenu : ces fonctions ne renvoient aucun code d'erreur (lecture de valeur sans statut) : un port absent ou mal décodé renvoie une valeur quelconque (souvent 0xFF) qu'il faut recouper avec d'autres sources avant de l'interpréter. Écritures E/S : même règle de sécurité que ci-dessus (aucune sans accord).

## Statut
- UEFI PI : lignes 1 à 5960 lues. Prochaine : ligne 5961 (IoWrite32, IoWrite64, MemRead/MemWrite).

## Lignes 5961-6160 — I-12.3.8.10/11 IoWrite32 et IoWrite64 (VOID, Address 64 bits, Data 32/64 bits) ; I-12.3.8.12 à 15 MemRead8/16/32/64 (lecture d'un registre mémoire, valeur renvoyée sans statut) ; I-12.3.8.16 MemWrite8 et I-12.3.8.17 MemWrite16 (début).
- [AGENT] Retenu : (1) Lectures MMIO sans statut : une adresse non mappée ou un périphérique absent renvoie une valeur sans erreur ; à recouper. (2) Ces accès sont à la fois lecture et écriture de registres matériels : ne pas écrire dans une zone MMIO du projet sans accord. (3) Pour les registres 64 bits, certains matériels imposent des accès 32 bits : utiliser la largeur documentée du périphérique (lien avec les notes NVMe/xHCI).

## Statut
- UEFI PI : lignes 1 à 6160 lues. Prochaine : ligne 6161 (MemWrite16 suite, MemWrite32, MemWrite64, puis Legacy Region ou Capsule PPI).

## Lignes 6161-6360 — I-12.3.8.18/19 MemWrite32 et MemWrite64 ; attributs des vecteurs : DO_NOT_HOOK 0x00000000, HOOK_BEFORE 0x00000001, HOOK_AFTER 0x00000002, LAST_ENTRY 0x80000000 ; I-12.3.9 EFI_PEI_CAPSULE_PPI (GUID 3acf33ee-d892-40f4-a2fc-3854d2e1323d ; Coalesce (regroupe les fragments de capsule en mémoire contiguë, EFI_BUFFER_TOO_SMALL si tampon insuffisant), CheckCapsuleUpdate (EFI_SUCCESS si capsule présente, sinon EFI_NOT_FOUND), CreateState (appelé quand la mémoire est disponible : crée la HOB de capsule) ; I-12.3.9.2 Coalesce (début, MemoryBase et MemorySize).
- [AGENT] Retenu : (1) Une capsule est une mise à jour de firmware : l'agent ne doit ni la créer ni la traiter sans accord explicite. (2) Coalesce copie des fragments : vérifier que la taille de destination suffit avant copie, et ne pas réutiliser la zone si l'opération échoue (EFI_BUFFER_TOO_SMALL). (3) Les codes de vecteur ci-dessus confirment les attributs notés plus haut.

## Statut
- UEFI PI : lignes 1 à 6360 lues. Prochaine : ligne 6361 (suite Coalesce, CreateState, puis Graphics PPI).

## Lignes 6361-6560 — I-12.3.9.4 CapsuleCreateState (CapsuleBase et CapsuleSize issus de Coalesce ; EFI_VOLUME_CORRUPTED si la base n'est pas une capsule regroupée ; EFI_SUCCESS avec HOB créée) ; I-12.3.9.5 EFI_PEI_MP_SERVICES_PPI (GUID ee16160a-e8be-47a6-820a-c6900db0250a ; GetNumberOfProcessors, GetProcessorInfo, StartupAllAPs, StartupThisAP, SwitchBSP, EnableDisableAP, WhoAmI ; avant END_OF_PEI les AP doivent rester en état d'attente ; après END_OF_PEI ils passent en état compatible OS ; SwitchBSP et EnableDisableAP peuvent ne plus être disponibles après END_OF_PEI) ; I-12.3.9.6 GetNumberOfProcessors (total incluant BSP et AP désactivés, nombre activé ; EFI_DEVICE_ERROR si appelé depuis un AP) ; I-12.3.9.7 GetProcessorInfo (ProcessorNumber ; EFI_NOT_FOUND ; BSP seulement) ; I-12.3.9.8 StartupAllAPs (Procedure, SingleThread, TimeoutInMicroseconds (0 = infini), ProcedureArgument ; EFI_TIMEOUT en mode bloquant, les AP en dépassement sont arrêtés).
- [AGENT] TRÈS PERTINENT (cœurs multiples) : (1) Les AP doivent rester inactifs avant END_OF_PEI et dans l'état OS compatible après : l'agent ne doit pas réveiller un AP hors de ce cadre. (2) StartupAllAPs/StartupThisAP sont l'API officielle pour exécuter du code sur un AP : c'est ce que le projet doit utiliser plutôt que d'envoyer INIT/SIPI directement (règle déjà notée). (3) Un timeout 0 signifie attente infinie : toujours fournir une valeur finie dans le projet. (4) SwitchBSP change le cœur de démarrage : opération sensible, à ne pas faire sans accord.

## Statut
- UEFI PI : lignes 1 à 6560 lues. Prochaine : ligne 6561 (suite StartupAllAPs, StartupThisAP, SwitchBSP, EnableDisableAP, WhoAmI).

## Lignes 6561-6760 — I-12.3.9.8 StartupAllAPs (suite : EFI_NOT_READY immédiat si un AP est occupé, aucun AP ne démarre ; SingleThread = un par un dans l'ordre croissant, sinon simultané ; le fournisseur ne garantit pas que Procedure est MP-safe : tâches parallèles limitées ; PEI services et PPI interdits aux AP sauf mention ; codes EFI_NOT_STARTED, EFI_NOT_READY, EFI_TIMEOUT) ; I-12.3.9.9 StartupThisAP (un AP précis, bloquant ; EFI_INVALID_PARAMETER si BSP ou AP désactivé) ; I-12.3.9.10 SwitchBSP (ProcessorNumber, EnableOldBSP ; EFI_UNSUPPORTED si impossible ; seul le BSP courant peut l'appeler) ; EnableDisableAP (EnableAP, HealthFlag optionnel ; un AP réactivé reçoit une initialisation complète compatible OS ; EFI_UNSUPPORTED si impossible).
- [AGENT] Retenu : (1) Procedure MP-safe non garantie : le code exécuté sur un AP doit être limité à une tâche indépendante, sans accès partagé non protégé. (2) EFI_NOT_READY : un AP occupé fait échouer tout le lot : prévoir un nouvel essai ou une file. (3) EnableDisableAP : désactiver un cœur est une opération de configuration ; ne pas le faire sans accord. (4) L'état de santé (HealthFlag) doit être lu, pas ignoré.

## Statut
- UEFI PI : lignes 1 à 6760 lues. Prochaine : ligne 6761 (suite EnableDisableAP, WhoAmI, puis Graphics PPI).

## Lignes 6761-6960 — I-12.3.9.10 suite EnableDisableAP (codes : EFI_UNSUPPORTED si opération impossible avant retour, EFI_INVALID_PARAMETER si ProcessorNumber est le BSP) ; I-12.3.9.11 WhoAmI (renvoie le numéro logique du processeur appelant ; accessible au BSP et aux AP ; EFI_INVALID_PARAMETER si NULL) ; I-12.3.9.12 EFI_PEI_MP_SERVICES2_PPI (GUID 5cb9cb3d-31a4-480c-9498-29d2b9bacfba ; même jeu de services + StartupAllCPUs ; même règle d'état AP avant/après END_OF_PEI) ; I-12.3.9.13 GetNumberOfProcessors v2 (sans paramètre PeiServices) ; I-12.3.9.14 GetProcessorInfo v2 ; I-12.3.9.15 StartupAllAPs v2 (début).
- [AGENT] Retenu : (1) WhoAmI doit être appelé pour savoir sur quel cœur on exécute : ne pas supposer que le cœur 0 est le BSP après un SwitchBSP. (2) Version 2 du PPI : l'agent doit vérifier la présence de la PPI avant de l'utiliser (EFI_NOT_AVAILABLE_YET sinon) et choisir la version disponible.

## Statut
- UEFI PI : lignes 1 à 6960 lues. Prochaine : ligne 6961 (suite StartupAllAPs v2, StartupThisAP v2, SwitchBSP v2, EnableDisableAP v2, StartupAllCPUs).

## Lignes 6961-7160 — I-12.3.9.15 StartupAllAPs v2 (mêmes règles que v1 : SingleThread, timeout 0 = infini, EFI_NOT_READY si AP occupé, Procedure non garantie MP-safe) ; I-12.3.9.16 StartupThisAP v2 (EFI_INVALID_PARAMETER si BSP ou AP désactivé) ; I-12.3.9.17 SwitchBSP v2 (EnableOldBSP ; EFI_NOT_READY si l'AP est occupé) ; EnableDisableAP v2 (EnableAP, HealthFlag ; même règle EFI_UNSUPPORTED).
- [AGENT] Même conclusion que la v1 : les règles d'appel, de MP-safety et de timeout sont identiques ; l'agent doit traiter EFI_NOT_READY et EFI_TIMEOUT comme des échecs explicites. Ne pas exécuter de Procedure non MP-safe en parallèle.

## Statut
- UEFI PI : lignes 1 à 7160 lues. Prochaine : ligne 7161 (suite EnableDisableAP v2, StartupAllCPUs, puis chapitre Graphics PPI).

## Lignes 7161-7360 — EnableDisableAP v2 (fin) ; I-12.3.9.18 WhoAmI v2 (sans PeiServices) ; I-12.3.9.19 StartupAllCPUs (exécute sur tous les CPU activés, BSP inclus ; EFI_NOT_READY si un CPU est occupé, aucun n'est lancé ; mode non bloquant : envoyé à tous sans attente) ; I-12.3.9.20 Random Number Generator PPI (GUID eaed0a7e-1a70-4c2b-8558-371774 56d806 ; GetInfo (liste d'algorithmes, premier = défaut, au moins un) ; GetRNG (RNGAlgorithm optionnel, RNGValueLength octets exacts, jamais de réutilisation d'octets entre appels ; EFI_NOT_READY, EFI_UNSUPPORTED, EFI_DEVICE_ERROR, EFI_INVALID_PARAMETER) ; un DRBG alimenté par une source brute doit offrir au moins 256 bits de sécurité.
- [AGENT] TRÈS PERTINENT (aléa) : (1) Toute clé ou nonce de l'agent doit provenir de GetRNG, jamais d'un compteur ou d'un horodatage seul. (2) Vérifier que GetRNG a bien renvoyé le nombre d'octets demandé (RNGValueLength exact). (3) Un EFI_DEVICE_ERROR ou EFI_NOT_READY ne doit pas être remplacé par une valeur de repli prévisible : il faut échouer.

## Statut
- UEFI PI : lignes 1 à 7360 lues. Prochaine : ligne 7361 (suite RNG GetRNG, puis Graphics PPI).

## Lignes 7361-7560 — I-12.3.9.20.1.2 GetRNG (fin des codes) ; I-12.3.9.20.2/3 définitions d'algorithmes (GUID UEFI 2.10 section 37.5.4 ; références NIST SP 800-90 DRBG, ANSI X9.31) ; I-12.4 Graphics PEIM : PPI EFI_PEI_GRAPHICS_PPI (GUID 6ecd1463-4a4a-461b-af5f-5a33e3b2162b ; GraphicsPpiInit, GraphicsPpiGetMode) ; GraphicsPpiInit (prérequis : mémoire initialisée ; phases dans l'ordre ; EFI_NOT_READY = réessayer plus tard) ; GraphicsPpiGetMode (Fig. 12.1 ; EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE : MaxMode, Mode, Info, SizeOfInfo, FrameBufferBase, FrameBufferSize ; EFI_GRAPHICS_OUTPUT_MODE_INFORMATION : Version, résolutions, PixelFormat (RGB 8 bits/couleur, BGR, PixelBitMask, PixelBltOnly), PixelInformation, PixelsPerScanLine ; pixel 32 bits, octet 3 réservé à 0) ; I-12.4.4 EFI_PEI_GRAPHICS_INFO_HOB (GUID 39f62cce-6825-4669-bb56-541aba753a07 : FrameBufferBase, FrameBufferSize, GraphicsMode) ; EFI_PEI_GRAPHICS_DEVICE_INFO_HOB (GUID e5cb2ac9-d35d-4430-936e-1de332478de7 : VendorId, DeviceId, SubsystemVendorId, SubsystemId, RevisionId...).
- [AGENT] TRÈS PERTINENT (framebuffer) : (1) Le pas entre lignes est PixelsPerScanLine, pas HorizontalResolution : une erreur de calcul décale l'affichage (déjà noté dans les conventions du projet). (2) PixelBltOnly : pas de framebuffer linéaire, ne pas écrire directement en mémoire. (3) Les HOB graphiques donnent l'adresse du framebuffer : l'agent doit la lire dans la HOB et non la coder en dur. (4) Le pixel a l'octet 3 réservé : ne pas s'en servir pour de la donnée.

## Statut
- UEFI PI : lignes 1 à 7560 lues. Prochaine : ligne 7561 (suite EFI_PEI_GRAPHICS_DEVICE_INFO_HOB, puis chapitre I-13 handoff PEI→DXE).

## Lignes 7561-7760 — I-12.4.4.2 EFI_PEI_GRAPHICS_DEVICE_INFO_HOB (BarIndex : BAR MMIO contenant le framebuffer, BAR 0 par défaut ; ID à MAX_UINT16 ou RevisionId à MAX_UINT8 = champ ignoré) ; le framebuffer peut être une adresse temporaire réaffectée par l'énumération PCI : la phase post-PEI doit recalculer son adresse après énumération ; CHAPITRE 13 handoff PEI→DXE : DXE IPL PPI localisé par GUID ; liste HOB transmise ; HOB requises (PHIT, Resource Descriptor mémoire et firmware, BSP Stack, BSPStore Itanium seulement, Firmware Volume, Memory Allocation Module) ; I-13.4 état du processeur au handoff (x86 : mode plat 32 bits ; Itanium : mode physique ; XScale : superviseur, mapping 1:1) ; CHAPITRE 14 chemins de boot : I-14.1 (PEIM modifient le mode via SetBootMode, l'ordre de dispatch ne dépend pas du mode) ; I-14.2 ordre de phases SEC, PEI, DXE, BDS, Runtime, Afterlife ; I-14.2.1 reset (PAL-A/PAL-B Itanium ; reset non-POR pour changer la fréquence, finir l'init chipset, réponse à une erreur grave, CVDR) ; I-14.3 chemins normaux (G0->S0 full, minimal, pas de changement, diagnostics, défauts ; S3 : table de sauvegarde matérielle dans la zone réservée E820 type 4 ; S4 ; S5) ; I-14.4 récupération (découverte via cavalier par exemple ; PEI peut transiter vers DXE).
- [AGENT] TRÈS PERTINENT : (1) Ordre de priorité des modes de boot : RECOVERY > FLASH_UPDATE > S3_RESUME > MINIMAL > FULL > NO_CHANGE > DIAGNOSTICS > DEFAULTS > S4 > S5 > S2 ; un PEIM ne doit pas revenir sur le mode recovery une fois posé. (2) La table de sauvegarde S3 contient des écritures matérielles à rejouer : l'agent ne doit pas la modifier ni la rejouer sans accord. (3) Le framebuffer peut bouger après énumération PCI : l'agent doit relire son adresse après le handoff, pas garder celle de la HOB.

## Statut
- UEFI PI : lignes 1 à 7760 lues. Prochaine : ligne 7761 (suite I-14.9.4 récupération DXE, puis chapitre I-15 mémoire physique PEI).

## Lignes 7761-7960 — I-14.9.4 Recovery DXE : le DXE IPL PEIM est toujours invoqué ; en recovery il appelle Recovery Module PPI (LoadRecoveryCapsule) ; séquence : compter les capsules par périphérique, décider de l'ordre, charger, vérifier la sécurité (échec = HOB d'erreur, passage au suivant), décomposer, invalider les FV modifiables ; tableau des fonctions Device Recovery Module (GetNumberRecoveryCapsules, GetRecoveryCapsuleInfo, LoadRecoveryCapsule) et Block I/O (GetNumberOfBlockDevices, GetBlockDeviceMediaInfo, ReadBlocks) ; I-15 mémoire physique PEI : avant mémoire permanente, rien n'est supposé présent ; InstallPeiMemory crée les HOB Resource Descriptor ; allocations hors zone via HOB GUID Extension ; après : allocation via HOB GUID (ne doit pas chevaucher la zone PHIT EfiMemoryBottom/Top) ou AllocatePages ; I-16 Itanium (recovery check call, INIT et MCHK, min-state save area, EFI_PEI_MIN_STATE_DATA avec pointeurs et checksums des handlers INIT/MCHK du système d'exploitation) ; I-17 phase SEC (gestion de tous les reset, mémoire temporaire, racine de confiance, handoff) — début de la section.
- [AGENT] TRÈS PERTINENT : (1) Le mode recovery est posé par un PEIM après détection de corruption et ne doit plus être changé : un agent qui détecte une corruption doit poser recovery et s'arrêter, pas réécrire de firmware (règle du projet). (2) Les allocations après mémoire permanente ne doivent pas chevaucher la zone PHIT : l'agent doit vérifier ses tampons contre les bornes PHIT. (3) Les checksums des handlers INIT/MCHK doivent être vérifiés avant appel (même principe : vérifier l'intégrité avant exécution). (4) SEC : la racine de confiance est la première étape : cohérent avec les notes précédentes.

## Statut
- UEFI PI : lignes 1 à 7960 lues. Prochaine : ligne 7961 (suite SEC : traitement des resets, mémoire temporaire, racine de confiance, SEC Platform Information).

## Lignes 7961-8160 — I-17.2 SEC : traitement de tous les événements de restart (power-on, reset, exceptions) ; mémoire temporaire en « no eviction mode » (cache comme RAM linéaire : tout accès doit être un hit, une éviction vers une RAM non configurée peut faire planter la plateforme) ; racine de confiance (authentifier PEI Foundation depuis SEC) ; I-17.2.4 handoff : état plateforme, BFV, RAM temporaire, pile, HOB optionnelles ; EFI_PEI_CORE_FV_LOCATION_PPI remplace la BFV si présent ; I-17.3 SEC Platform Information PPI ; I-17.4 SEC HOB Data PPI (installées avant tout PEIM) ; I-17.5 Health Flag : Tested (Te) seul bit commun IA-32/Itanium ; bits Vm, Ia, Fp, Mf ; I-17.5.1 Self-Test State (0 Catastrophic, 1 Healthy, 2 Performance Restricted, 3 Functionally Restricted ; états fonctionnellement restreints : au moins une unité mémoire, ALU, décaleur et branchement doivent fonctionner) ; I-17.6.1 IA-32 : localiser PEI, passer le contrôle avec l'état de handoff, initialiser la mémoire cache utilisable en pile (Fig. 17.2) ; I-17.6.2 Itanium : PAL-A et PAL-B authentifiés par le microcode ; CHAPITRE 18 grammaire DEPEX : BNF (<depex>, <bool> AND/OR, <term> NOT, <factor> TRUE/FALSE/GUID/END, GUID en 8-4-4-8 octets, hexadécimal 0x) ; exemples binaires (PUSH 0x02 + GUID 16 octets, AND 0x03, END 0x08) ; CHAPITRE 19 TE Image (début : format réduit, en-têtes COFF et optionnels utiles uniquement).
- [AGENT] Retenu : (1) « no eviction mode » : une écriture dans une zone non encore configurée peut faire planter la plateforme : l'agent ne doit pas toucher à une mémoire non déclarée, même lors d'un test. (2) Health Flag : Tested = 0 signifie aucun test effectué, les autres champs ne valent rien : à ne pas lire comme un succès. (3) Functionally Restricted : un processeur restreint doit être signalé ; ne pas poursuivre sans avertissement. (4) La grammaire DEPEX est une entrée à valider (BNF strict) : même principe de rejet d'entrée mal formée.

## Statut
- UEFI PI : lignes 1 à 8160 lues. Prochaine : ligne 8161 (suite Table 19.1 COFF, Table 19.2 optionnel, en-tête TE, relocations).

## Lignes 8161-8360 — Tables 19.1/19.2 (champs COFF et optionnels requis pour TE : Machine, NumberOfSections, AddressOfEntryPoint, BaseOfCode, ImageBase, Subsystem) ; I-19.2.1 en-tête TE (Signature 0x5A56 "VZ", Machine, NumberOfSections (1 octet), Subsystem (1 octet), StrippedSize, AddressOfEntryPoint, BaseOfCode, ImageBase, DataDirectory[2] : relocations et debug ; StrippedSize = en-têtes retirés, delta de taille = StrippedSize - sizeof(EFI_TE_IMAGE_HEADER)) ; CHAPITRE 20 création TE (outil : lire PE/COFF, remplir l'en-tête, écrire, refuser si un champ ne tient pas) ; I-20.3 relocations non modifiées à la conversion ; CHAPITRE 21 chargement TE (XIP : entrée = image + taille en-tête + AddressOfEntryPoint - StrippedSize ; relocations ; PIC si relocation size = 0) ; PARTIE II DXE CIS : II-1 introduction (UEFI 2.0 avec extensions, machine préparée pour les phases suivantes ; cibles : IHV/OEM et développeurs BIOS) ; conventions II-1.4 (little-endian, réservés initialisés à zéro, préservés en mise à jour) ; tables des figures et tableaux de la partie II.
- [AGENT] Retenu : (1) Un en-tête TE mal converti ou un StrippedSize incohérent décale l'entrée : l'agent doit vérifier la signature VZ et les bornes avant de sauter à un point d'entrée. (2) Image PIC : elle peut être exécutée depuis la flash sans relocation ; un chargeur qui applique des relocations à une image PIC altère le code. (3) Les champs réservés doivent être préservés en mise à jour : règle déjà relevée au chapitre 3.

## Statut
- UEFI PI : lignes 1 à 8360 lues. Prochaine : ligne 8361 (Part II, II-1.4.6 typographie puis II-2 vue d'ensemble DXE).

## Lignes 8361-8560 — II-1.5 exigences (une implémentation partielle est non conforme ; un composant dépendant d'interfaces hors PI ne l'est pas) ; II-1.6 notations (identiques à la partie I) ; CHAPITRE 2 vue d'ensemble DXE : phase où l'initialisation système est faite ; entrée = liste HOB ; trois composants : Foundation (Boot/Runtime/DXE Services), Dispatcher, Drivers ; le Foundation ne persiste pas dans le runtime, seules les structures runtime et les drivers runtime y restent ; DXE n'exige pas PEI, seule une liste HOB valide est requise ; II-2.2 UEFI System Table (pointeurs vers Boot/Runtime Services, consoles, table de configuration avec GUID : DXE Services, HOB, ACPI, SMBIOS, SAL) ; Boot Services (TPL, mémoire, événements et timers, handles/protocoles, images PE/COFF, drivers connect/disconnect ; connexion minimale = réduction du temps de boot) ; Runtime Services (variables, horloge, reset, mappage virtuel une seule fois en mode physique) ; DXE Services (GCD, dispatcher) ; II-2.4 dispatcher (DEPEX + a priori ; BDS après épuisement) ; II-2.6 protocoles architecturaux (Security, CPU, Metronome, Timer, BDS, Watchdog, Runtime, Variable, Variable Write, Monotonic Counter, Reset, Real Time Clock, Capsule) ; II-2.7 Status Code Runtime Protocol ; CHAPITRE 3 Boot Manager (BDS après les drivers ; chemin court commençant par un FV, GUID comparé au nom du FV ; BDS : consoles, drivers, boot ; re-invocation du dispatcher si bloqué).
- [AGENT] Retenu : (1) Les services de Boot ne persistent pas dans le runtime : l'agent qui s'exécute après ExitBootServices ne doit plus appeler de service Boot (règle à retenir pour toute phase OS-runtime). (2) Le GCD décrit toute la mémoire : allouer en dehors d'une région déclarée est une erreur. (3) Chemin court de FV : le GUID est comparé au GUID de nom du FV : vérifier l'identité avant d'utiliser. (4) SetVirtualAddressMap ne peut être appelé qu'une fois : à ne pas répéter. (5) Le watchdog du firmware doit être géré avant de passer la main (déjà noté).

## Statut
- UEFI PI : lignes 1 à 8560 lues. Prochaine : ligne 8561 (CHAPITRE 4 UEFI System Table : DXE_SERVICES, suite).

## Lignes 8561-8760 — II-4.1.1 DXE_SERVICES (signature 0x565245535f455844 "DXE_SERVEV" en ordre octets, révision = PI_SPECIFICATION_VERSION ; fonctions : AddMemorySpace, AllocateMemorySpace, FreeMemorySpace, RemoveMemorySpace, GetMemorySpaceDescriptor, SetMemorySpaceAttributes, GetMemorySpaceMap, AddIoSpace, AllocateIoSpace, FreeIoSpace, RemoveIoSpace, GetIoSpaceDescriptor, GetIoSpaceMap, Dispatch, Schedule (efface le flag SOR), Trust (passe un fichier de non-confiance à confiance), ProcessFirmwareVolume (FV en mémoire d'une capsule), SetMemorySpaceCapabilities) ; II-4.2 exemples d'entrée : application UEFI (recherche de la DXE Services Table dans la table de configuration par GUID, affichage via ConOut, allocation de pool en EfiBootServicesData).
- [AGENT] Retenu : (1) Trust : promouvoir un fichier vers la confiance est une décision de sécurité : l'agent ne doit jamais le faire sans accord. (2) SetMemorySpaceAttributes modifie les attributs de mémoire (exécution, écriture) : opération sensible, à ne pas appeler sans accord. (3) Schedule : efface le drapeau SOR qui planifie un driver : un changement d'ordre de chargement. (4) La découverte de DXE Services passe par la table de configuration et un GUID : vérifier le GUID avant d'utiliser le pointeur.

## Statut
- UEFI PI : lignes 1 à 8760 lues. Prochaine : ligne 8761 (suite exemple application, exemples de drivers non-UEFI et UEFI driver model).

## Lignes 8761-8960 — II-4.2.1 suite (GetTime, FreePool, GetMemorySpaceMap, FreePool : rappel que chaque allocation de tampon doit être libérée) ; II-4.2.2 driver non-UEFI-Driver-Model résident (renvoie EFI_SUCCESS pour rester en mémoire ; EFI_REQUEST_UNLOAD_IMAGE si DXE Services absents) ; II-4.2.3 non résident (renvoie EFI_REQUEST_UNLOAD_IMAGE) ; II-4.2.4 UEFI Driver Model (EFI_DRIVER_BINDING_PROTOCOL : Supported, Start, Stop, version ; installé sur le handle image via InstallMultipleProtocolInterfaces) ; II-4.2.5 Driver Model déchargeable (ouverture de EFI_LOADED_IMAGE_PROTOCOL, champ Unload = AbcUnload ; AbcUnload refuse si le driver gère encore des contrôleurs, et doit désinstaller protocoles et libérer mémoire de l'entrée).
- [AGENT] Retenu : (1) Un driver qui reste résident (EFI_SUCCESS) occupe la mémoire de boot services : l'agent qui se charge comme driver doit le signaler explicitement. (2) Un déchargement doit libérer tout ce qu'il a alloué ou installé : aucune fuite de protocole ou de pool. (3) Un driver qui gère des contrôleurs ne doit pas être déchargé : vérifier ce compteur avant tout unload.

## Statut
- UEFI PI : lignes 1 à 8960 lues. Prochaine : ligne 8961 (suite AbcUnload, UEFI Driver Model multi-instances, puis II-5 Boot Services).

## Lignes 8961-9160 — II-4.2.5 suite AbcUnload (EFI_ACCESS_DENIED si Count > 0 contrôleurs gérés ; désinstallation du Driver Binding) ; II-4.2.6 Driver Model à trois instances (un Driver Binding sur le handle image, deux sur de nouveaux handles) ; CHAPITRE 5 Boot Services : II-5.1.1 EFI_EVENT_LEGACY_BOOT_GUID (notifié juste avant INT19h) ; II-5.1.2 groupes d'événements : EFI_EVENT_GROUP_DXE_DISPATCH_GUID (un tour de dispatch DXE fini) ; EFI_END_OF_DXE_EVENT_GROUP_GUID (avant tout driver tiers : deux étapes : signal End of DXE puis installation du DXE SMM Ready to Lock) ; II-5.1.3 LoadImage étendu (BootPolicy, ParentImageHandle, FilePath, SourceBuffer, SourceSize ; relocalise toujours ; Security Architectural Protocol : EFI_SUCCESS normal, EFI_SECURITY_VIOLATION = chargé mais ne pas démarrer, EFI_ACCESS_DENIED = image déchargée et refusée) ; sous-systèmes : EFI_APPLICATION (10, LoaderCode, libérée à la sortie), BOOT_SERVICE_DRIVER (11, BootServicesCode, libérée si erreur), RUNTIME_DRIVER (12, RuntimeServicesCode, converti par SetVirtualAddressMap), SAL_RUNTIME_DRIVER (13, non converti).
- [AGENT] TRÈS PERTINENT (chaîne de confiance) : (1) EFI_SECURITY_VIOLATION : l'image est chargée mais ne doit pas démarrer : l'agent ne doit donc jamais exécuter un binaire qui a reçu ce code. (2) EFI_ACCESS_DENIED : image à ne jamais faire confiance, déchargée. (3) Le groupe "End of DXE" marque la fin du contrôle plateforme : l'agent, s'il est chargé après, doit considérer les ressources verrouillées. (4) Un runtime driver non converti (SAL_RUNTIME) garde ses adresses physiques après SetVirtualAddressMap : vérifier le type avant toute conversion d'adresse.

## Statut
- UEFI PI : lignes 1 à 9160 lues. Prochaine : ligne 9161 (suite LoadImage et sous-systèmes, puis II-5.2 Boot Services : tables de services).

## Lignes 9161-9360 — Table 5.2 codes LoadImage (EFI_SUCCESS, EFI_SECURITY_VIOLATION, EFI_ACCESS_DENIED, EFI_NOT_FOUND, EFI_INVALID_PARAMETER, EFI_UNSUPPORTED, EFI_OUT_OF_RESOURCES, EFI_LOAD_ERROR, EFI_DEVICE_ERROR) ; CHAPITRE 6 runtime : Status Code Runtime Protocol (ReportStatusCode utilisable au boot et au runtime) ; CHAPITRE 7 DXE Services : GCD mémoire (carte d'état initialisée depuis la liste HOB, jusqu'à 64 lignes d'adresse, états : inexistante, système, MMIO, réservée ; fusion des régions adjacentes semblables) ; GCD E/S (mêmes principes, états : inexistante, E/S, réservée) ; tableau 7.1 (services de type boot : AddMemorySpace, AllocateMemorySpace, FreeMemorySpace, RemoveMemorySpace, GetMemorySpaceDescriptor, SetMemorySpaceAttributes, SetMemorySpaceCapabilities, GetMemorySpaceMap, Add/Allocate/Free/Remove/GetIoSpaceDescriptor, GetIoSpaceMap) ; II-7.2.4.1 AddMemorySpace (début).
- [AGENT] TRÈS PERTINENT (carte mémoire) : (1) La carte GCD vient de la liste HOB : toute région non déclarée n'est pas disponible à l'allocation. (2) Les états "mémoire MMIO" et "réservée" ne doivent pas servir de RAM ordinaire : le projet ne doit pas allouer ni écrire dans une région marquée MMIO ou réservée sans accord. (3) La fusion des régions adjacentes change la forme de la carte : toujours relire la carte au lieu de la supposer constante.

## Statut
- UEFI PI : lignes 1 à 9360 lues. Prochaine : ligne 9361 (suite AddMemorySpace et autres services GCD).

## Lignes 9361-9560 — II-7.2.4.1 AddMemorySpace (types autorisés : Reserved, SystemMemory, Persistent, MoreReliable, Unaccepted, MemoryMappedIo ; Capabilities = attributs supportés ; EFI_INVALID_PARAMETER si Length 0 ou type invalide ; EFI_UNSUPPORTED si le processeur ne gère pas une partie ; EFI_ACCESS_DENIED si conflit ou déjà alloué ; les régions SystemMemory alignées sur 4 KiB peuvent être converties automatiquement en allouées ; types EfiGcdMemoryType : NonExistent, Reserved, SystemMemory, MemoryMappedIo, Persistent (non volatile octet-adressable), MoreReliable, Unaccepted (doit être accepté avant usage)) ; II-7.2.4.2 AllocateMemorySpace (cinq modes de recherche : AnySearchBottomUp, MaxAddressSearchBottomUp, Address, AnySearchTopDown, MaxAddressSearchTopDown ; ImageHandle et DeviceHandle identifient le demandeur ; EFI_NOT_FOUND si aucune plage) ; II-7.2.4.3 FreeMemorySpace (début).
- [AGENT] TRÈS PERTINENT : (1) Unaccepted : une région non acceptée doit être acceptée avant usage (cf. notes sur AMD SEV-SNP dans UEFI 2.11) : l'agent ne doit jamais écrire dans une région Unaccepted. (2) AllocateMemorySpace indique le propriétaire par ImageHandle : l'agent doit enregistrer son ImageHandle pour pouvoir libérer ses allocations. (3) FreeMemorySpace ne doit libérer que ce qu'on a alloué : même règle de propriété que précédemment.

## Statut
- UEFI PI : lignes 1 à 9560 lues. Prochaine : ligne 9561 (suite FreeMemorySpace, RemoveMemorySpace, GetMemorySpaceDescriptor, SetMemorySpaceAttributes).

## Lignes 9561-9760 — II-7.2.4.3 FreeMemorySpace (fin : EFI_NOT_FOUND si non alloué par AllocateMemorySpace) ; II-7.2.4.4 RemoveMemorySpace (ramène à NonExistent ; EFI_ACCESS_DENIED si alloué ; EFI_NOT_FOUND si non ajouté) ; II-7.2.4.5 GetMemorySpaceDescriptor (EFI_GCD_MEMORY_SPACE_DESCRIPTOR : BaseAddress, Length, Capabilities, Attributes, GcdMemoryType, ImageHandle (NULL = non alloué), DeviceHandle ; EFI_NOT_AVAILABLE_YET si CPU Architectural Protocol absent) ; II-7.2.4.6 SetMemorySpaceAttributes (début : Attributes doit être un sous-ensemble strict des Capabilities, EFI_UNSUPPORTED sinon).
- [AGENT] TRÈS PERTINENT : (1) Un descripteur avec ImageHandle NULL est libre : ne pas écrire dans une région qui a un ImageHandle non NULL sans l'avoir allouée soi-même. (2) SetMemorySpaceAttributes modifie les droits d'exécution et d'écriture : une région de code ne doit jamais devenir inscriptible et exécutable en même temps ; l'agent ne doit pas appeler cette fonction sans accord. (3) RemoveMemorySpace retire une région de la carte : à éviter sans accord.

## Statut
- UEFI PI : lignes 1 à 9760 lues. Prochaine : ligne 9761 (suite SetMemorySpaceAttributes, GetMemorySpaceMap, Add/Allocate/Free/Remove/Get IO space).

## Lignes 9761-9960 — II-7.2.4.6 SetMemorySpaceAttributes (fin : EFI_ACCESS_DENIED si la politique système l'interdit) ; II-7.2.4.7 SetMemorySpaceCapabilities (les capacités doivent inclure les attributs en cours, sinon EFI_UNSUPPORTED) ; II-7.2.4.8 GetMemorySpaceMap (tableau alloué par AllocatePool, à libérer par l'appelant ; EFI_INVALID_PARAMETER si NULL) ; II-7.2.4.9 AddIoSpace (types : Reserved, Io ; Length 0 invalide ; conflit = EFI_ACCESS_DENIED) ; EFI_GCD_IO_TYPE : NonExistent, Reserved (décodée mais non utilisable pour périphériques), Io (ports utilisables) ; II-7.2.4.10 AllocateIoSpace (début, mêmes modes que la mémoire).
- [AGENT] Retenu : (1) La carte mémoire renvoyée est un tampon alloué à libérer : ne pas oublier FreePool (règle de propriété). (2) Un port I/O marqué Reserved n'est pas utilisable pour un périphérique : l'agent ne doit pas l'utiliser malgré son adresse connue. (3) Les capacités ne peuvent pas être réduites en dessous des attributs courants : une modification doit suivre l'ordre attributs puis capacités.

## Statut
- UEFI PI : lignes 1 à 9960 lues. Prochaine : ligne 9961 (suite AllocateIoSpace, FreeIoSpace, RemoveIoSpace, GetIoSpaceDescriptor, GetIoSpaceMap).

## Lignes 9961-10160 — II-7.2.4.10 AllocateIoSpace (fin : mêmes modes de recherche que la mémoire, ImageHandle obligatoire, EFI_NOT_FOUND si aucune plage) ; II-7.2.4.11 FreeIoSpace (EFI_NOT_FOUND si non alloué par AllocateIoSpace) ; II-7.2.4.12 RemoveIoSpace (ramène à NonExistent ; EFI_ACCESS_DENIED si alloué) ; II-7.2.4.13 GetIoSpaceDescriptor (EFI_GCD_IO_SPACE_DESCRIPTOR : BaseAddress, Length, GcdIoType, ImageHandle, DeviceHandle).
- [AGENT] Retenu : (1) Un port E/S ne doit être utilisé qu'après AllocateIoSpace avec son ImageHandle : un port non alloué peut appartenir à un autre composant. (2) RemoveIoSpace et FreeIoSpace sont destructifs pour les autres : ne pas les appeler sans accord. (3) Vérifier GcdIoType = Io avant tout accès (Reserved n'est pas utilisable).

## Statut
- UEFI PI : lignes 1 à 10160 lues. Prochaine : ligne 10161 (suite GetIoSpaceDescriptor, GetIoSpaceMap, puis II-7.3 Dispatcher Services).

## Lignes 10161-10360 — II-7.2.4.14 GetIoSpaceMap (tableau alloué par AllocatePool, libération par l'appelant) ; II-7.3 Dispatcher Services : Dispatch (charge et exécute les drivers dont les DEPEX sont vraies, après authentification par le Security Architectural Protocol ; EFI_SUCCESS si au moins un, EFI_NOT_FOUND sinon, EFI_ALREADY_STARTED si appel récursif) ; II-7.3.2 Schedule (efface le flag SOR d'un driver ; EFI_NOT_FOUND si absent ou sans SOR ; n'invoque pas le dispatcher) ; II-7.3.3 Trust (promeut un fichier non-fiable en fiable ; seul le SAP peut le mettre en non-fiable ; EFI_NOT_FOUND sinon) ; ProcessFirmwareVolume (crée un handle FV pour un FV en mémoire : EFI_FIRMWARE_VOLUME2_PROTOCOL + EFI_DEVICE_PATH ; EFI_VOLUME_CORRUPTED si invalide).
- [AGENT] TRÈS PERTINENT : (1) Trust et Schedule modifient ce qui est exécuté : décisions de sécurité et d'ordre, à ne jamais prendre sans accord. (2) ProcessFirmwareVolume : un FV provenant d'une capsule ou d'une zone mémoire doit être validé (EFI_VOLUME_CORRUPTED) avant d'être exposé. (3) Dispatch renvoie EFI_NOT_FOUND quand rien ne s'exécute : ce n'est pas une erreur fatale, mais l'appelant doit le savoir.

## Statut
- UEFI PI : lignes 1 à 10360 lues. Prochaine : ligne 10361 (CHAPITRE 8 Device Path : Firmware Volume Media Device Path, Firmware File Media Device Path).

## Lignes 10361-10560 — II-8 Device Path : Firmware Volume Media Device Path (type 4, sous-type 7, longueur 20, GUID du nom de FV ; texte "Fv(guid)") ; Firmware File Media Device Path (type 4, sous-type 6, longueur 20, GUID du fichier ; texte "FvFile(guid)") ; CHAPITRE 9 DXE Foundation : II-9.1 (portable : dépend uniquement de la liste HOB, pas d'adresse codée en dur, pas de code CPU/chipset/plateforme ; GetMemoryMap doit inclure les régions Reserved et MMIO de la GCD) ; II-9.2 liste HOB en lecture seule (PHIT en premier, puis entrées dans n'importe quel ordre, HOB GUIDées non consommées par le Foundation, entrées en position indépendante) ; II-9.3 structures (System Table, Boot/Runtime Services, DXE Services, tables de configuration : DXE Services, HOB, ACPI, SMBIOS, SAL ; à la sortie vers runtime seuls restent System Table, Runtime Services et tables de configuration) ; II-9.4 composants requis (Boot Services, DXE Services, parseur HOB, dispatcher avec évaluateur DEPEX, driver FV2, EFI_DECOMPRESS_PROTOCOL, StartImage/Exit avec SetJump/LongJump, chargeur PE/COFF PE32+, cache d'instructions si besoin, changement de pile) ; II-9.5 tâches avant le dispatcher (initialiser GCD et carte UEFI, tables Boot/DXE Services allouées puis remplies, Runtime Services en EFI_RUNTIME_SERVICES_MEMORY, System Table, handle de l'image Foundation, relocation de la HOB si besoin, notifications sur chaque protocole architectural, CRC32 des tables, FV handles) ; si un protocole architectural manque le Foundation s'arrête ; II-9.6 DXE_ENTRY_POINT (HobStart seul paramètre, lecture seule ; au moins PHIT, CPU, mémoire système, un FV ; un seul cœur, interruptions désactivées).
- [AGENT] TRÈS PERTINENT (pour le projet) : (1) La liste HOB est en lecture seule : l'agent ne doit jamais la modifier ; il la lit et ne copie que ce qui est nécessaire. (2) À la sortie vers runtime, seuls la System Table, les Runtime Services et les tables de configuration restent : l'agent doit ne garder aucun pointeur vers Boot Services ou DXE Services après ExitBootServices. (3) Le Foundation doit s'arrêter si un protocole architectural manque : ne pas démarrer l'OS dans un état non défini. (4) Le CRC32 des tables doit être recalculé après modification : toute modification de ces tables par l'agent invalide leur CRC.

## Statut
- UEFI PI : lignes 1 à 10560 lues. Prochaine : ligne 10561 (suite Table 9.1, dépendances des services Boot, DXE Services et HOB Translations).

## Lignes 10561-10760 — II-9.7 dépendances (SetTimer, RaiseTPL/RestoreTPL, SetWatchdogTimer, Stall, GetNextMonotonicCount, CalculateCrc32, GetMemoryMap : régions Reserved, PersistentMemory et MMIO avec attribut RUNTIME inclus dans la carte UEFI) ; II-9.7.2 Runtime Services (GetVariable, SetVariable : lecture seule puis écriture volatile ; écriture non-volatile = EFI_NOT_AVAILABLE_YET tant que le protocole d'écriture n'est pas là ; GetTime/SetTime/Wakeup via RTC ; SetVirtualAddressMap, ConvertPointer, ResetSystem via Runtime / Reset) ; II-9.7.3 DXE Services (GetMemorySpaceDescriptor : attribut valide seulement après CPU Arch ; SetMemorySpaceAttributes via CPU SetMemoryAttributes) ; II-9.8 traduction HOB : PHIT (région de mémoire testée, EfiFreeMemoryBottom = fin de la liste HOB, mémoire libre utilisable tôt) ; CPU HOB (SizeOfMemorySpaceMap et SizeOfIoSpaceMap : carte initiale NonExistent de 0 à 2^n) ; Resource Descriptor HOB (table 9.4 : System Memory, MMIO, Firmware Device, Reserved, Unaccepted, E/S ; ResourceAttribute donne les capacités de cache) ; Firmware Volume HOB (EFI_HOB_FIRMWARE_VOLUME et VOLUME2 ; handle créé, FV2 installé) ; Memory Allocation HOB (mise à jour de la GCD et de la carte UEFI) ; GUID Extension HOB (optionnelles) ; CHAPITRE 10 DXE Dispatcher : II-10.2 exigences (ordre fixe par a priori, ordre par DEPEX, "emergency patch" avant/après un driver, politique de sécurité via Security Architectural Protocol à chaque FV et chaque driver) ; II-10.3 a priori : GUID EFI_APRIORI_GUID fc510ee7-ffdc-11d4-bd41-0080c73c8881 ; liste de GUID packés ; DEPEX ignorées pour ces drivers ; un GUID introuvable est sauté ; un reste incomplet ignore le dernier GUID ; II-10.4 FV dans un FV (EFI_SECTION_DXE_DEPEX ; ignoré si déjà décrit par un HOB FV2).
- [AGENT] TRÈS PERTINENT : (1) PHIT : la mémoire libre annoncée est utilisable tôt ; l'agent doit cependant la considérer comme transitoire et ne pas la garder au-delà de la fin de DXE. (2) Unaccepted : même règle que plus haut, accepter avant usage. (3) Réglage des variables : écrire en NVRAM tant que le protocole d'écriture n'est pas installé renvoie EFI_NOT_AVAILABLE_YET : l'agent doit réessayer plus tard, pas forcer. (4) La liste a priori est une liste d'ordre : une entrée corrompue ou une fin tronquée doit être journalisée, pas ignorée en silence (règle de validation des entrées).

## Statut
- UEFI PI : lignes 1 à 10760 lues. Prochaine : ligne 10761 (suite II-10.4 FV dans un FV, machine d'état du dispatcher, exemples d'ordre, sécurité).

## Lignes 10761-10960 — II-10.6 DEPEX DXE (pile polonaise postfixée, même règle FALSE : dépassement/sous-dépassement, opcode invalide, END absent, instruction qui dépasse la section) ; II-10.7 opcodes DXE : BEFORE 0x00 et AFTER 0x01 (GUID de fichier, seul opcode de l'expression : sinon FALSE ; place le driver juste avant/après le driver cité ; "emergency patch") ; PUSH 0x02 (GUID de protocole, test par LocateProtocol) ; AND 0x03, OR 0x04, NOT 0x05, TRUE 0x06, FALSE 0x07 ; END 0x08 ; SOR 0x09 (Schedule On Request, voir suite).
- [AGENT] Retenu : (1) BEFORE/AFTER doivent être seuls dans l'expression : une expression qui les mélange est FALSE, donc jamais exécutée : règle de sûreté à reprendre. (2) Une instruction tronquée à la fin de la section est FALSE. (3) Les GUID non alignés : lecture octet par octet sur les architectures sensibles.

## Statut
- UEFI PI : lignes 1 à 10960 lues. Prochaine : ligne 10961 (II-10.7.8 FALSE, END, SOR, II-10.8 expressions vides, RPN, machine d'état du dispatcher DXE).

## Lignes 10961-11160 — II-10.7.8 FALSE ; II-10.7.9 END (dernier opcode) ; II-10.7.10 SOR 0x09 (NOP pour l'évaluateur ; drapeau de planification à l'appel de Schedule ; SOR doit être le premier opcode et suivi d'une expression valide, sinon FALSE) ; II-10.8 DEPEX sans dépendance = TRUE END (2 octets) ; II-10.9 DEPEX vide : implique l'AND de tous les protocoles architecturaux (BDS, CPU, Metronome, Monotonic Counter, RTC, Reset, Runtime, Security, Timer, Variable, Variable Write, Watchdog), soit environ 216 octets ; un driver sans section DEPEX n'est chargé qu'une fois tous les protocoles architecturaux installés ; exemple d'encodage PUSH (0x02 + GUID 16 octets).
- [AGENT] Retenu : (1) SOR : le driver reste en attente jusqu'à Schedule ; l'agent ne doit pas supposer qu'un driver SOR a été exécuté. (2) Un driver sans DEPEX n'est lancé qu'après tous les protocoles architecturaux : l'agent, s'il est un driver DXE, doit déclarer sa DEPEX pour contrôler le moment de son exécution.

## Statut
- UEFI PI : lignes 1 à 11160 lues. Prochaine : ligne 11161 (suite exemple DEPEX encodé, II-10.10 RPN, II-10.11 machine d'état du dispatcher).

## Lignes 11161-11360 — Exemple DEPEX encodé (suite de PUSH/AND pour les 12 protocoles architecturaux) ; II-10.10 RPN (statement : SOR expr END | BEFORE guid END | AFTER guid END | expr END) ; II-10.11 machine d'état du dispatcher DXE : Undiscovered -> Discovered -> (SOR ? Unrequested : Dependent) ; Unrequested ne passe à Dependent que par Schedule ; Dependent -> Scheduled quand DEPEX vraie ; file Scheduled : contrôle SAP à la tête de file (Untrusted si non conforme, Never Trusted si jamais exécutable ; Untrusted repasse Scheduled par Trust) ; LoadImage puis StartImage -> Initializing ; Exit -> Initialized (état terminal) ; réévaluation des Dependent après chaque Initialized ; a priori : Scheduled immédiatement ; II-10.12 exemples d'ordre : Security, Runtime, Variable d'abord (a priori), puis BDS et CPU (DEPEX TRUE), puis Timer/Metronome/Reset (dépendent du CPU) ; 30 ordres valides, un dispatcher doit être cohérent d'un démarrage à l'autre.
- [AGENT] TRÈS PERTINENT : (1) L'état Never Trusted est définitif : un driver ainsi marqué ne doit jamais être exécuté. (2) Untrusted ne repasse à Scheduled que par Trust explicite. (3) Pour l'agent : l'ordre de démarrage peut varier entre configurations ; ne jamais supposer un ordre fixe entre drivers qui n'ont pas de dépendance commune déclarée.

## Statut
- UEFI PI : lignes 1 à 11360 lues. Prochaine : ligne 11361 (suite ordres DXE, II-10.13 considérations de sécurité, puis II-11 drivers DXE).

## Lignes 11361-11560 — Table 10.12 (suite des 30 ordres valides : Security, Runtime, Variable, puis CPU, puis permutations de BDS, Timer, Metronome, Reset ; tableau purement illustratif, aucune règle normative supplémentaire).
- [AGENT] Pas de règle nouvelle. Confirme : l'ordre entre drivers sans dépendance commune est libre, donc un driver de l'agent ne doit pas dépendre d'un ordre particulier entre Timer, Metronome et Reset.

## Statut
- UEFI PI : lignes 1 à 11560 lues. Prochaine : ligne 11561 (suite tableau des ordres, II-10.13 considérations de sécurité, II-11 drivers DXE).

## Lignes 11561-11760 — Table 10.12 fin ; II-10.13 sécurité (le SAP n'est utilisable qu'après son installation ; a priori pour l'avancer ; SAP évalue FV puis chaque driver : action plateforme = ignorer le FV, Untrusted, Never Trusted, ou journaliser) ; CHAPITRE 11 DXE drivers : II-11.2.1 drivers précoces (initialisation CPU/chipset, produisent les protocoles architecturaux) ; II-11.2.2 drivers UEFI Driver Model (ne touchent pas au matériel à l'init, enregistrent un Driver Binding, déclenchés par BDS) ; II-11.2.3 boot service drivers (libérés à ExitBootServices) vs runtime drivers (services persistants) ; les runtime ne peuvent plus appeler Boot Services ou DXE Services après ExitBootServices ; CHAPITRE 12 DXE Architectural Protocols : introduction (le Foundation lit une liste HOB et une FV en lecture seule ; mode plat physique, un cœur, interruptions désactivées ; System Table, Boot, Runtime, DXE Services ; Foundation et protocoles de boot libérés au runtime ; SetVirtualAddressMap via Runtime Architectural Protocol) ; II-12.2 BDS Architectural (EFI_BDS_ARCH_PROTOCOL, GUID 665e3ff6-46cc-11d4-9a38-0090273fc14d ; Entry : ne doit normalement pas revenir ; si retour, redémarrer le dispatcher ; échec deux fois de suite = périphérique ignoré) ; II-12.3 CPU Architectural (GUID 26baccb1-6f42-11d4-bce7-0080c73c8881 ; FlushDataCache (DMA), EnableInterrupt/DisableInterrupt (RaiseTPL/RestoreTPL), GetInterruptState, Init (INIT, EFI_UNSUPPORTED si impossible), RegisterInterruptHandler, GetTimerValue, SetMemoryAttributes, NumberOfTimers et DmaBufferAlignment en lecture seule ; le driver CPU doit initialiser les attributs de la carte GCD via SetMemorySpaceAttributes à leurs valeurs courantes).
- [AGENT] TRÈS PERTINENT (projet bare-metal) : (1) DmaBufferAlignment : les tampons DMA doivent être alignés sur la plus grande ligne de cache, pour qu'aucun tampon ne partage une ligne de cache avec un autre (cohérent avec les notes NVMe/xHCI). (2) Les attributs de mémoire sont initialisés par le driver CPU : l'agent ne doit pas les changer hors du chemin officiel. (3) Un runtime driver ne doit pas utiliser de service Boot après ExitBootServices : règle déjà retenue. (4) Init (INIT du processeur) est une opération de reset partiel : à ne pas déclencher sans accord.

## Statut
- UEFI PI : lignes 1 à 11760 lues. Prochaine : ligne 11761 (suite CPU Architectural : FlushDataCache, EnableInterrupt, DisableInterrupt, GetInterruptState, Init, RegisterInterruptHandler, GetTimerValue, SetMemoryAttributes).

## Lignes 11761-11960 — II-12.3.2 FlushDataCache (Start, Length : la ligne de cache est élargie au début et à la fin ; types EfiCpuFlushTypeWriteBackInvalidate (obligatoire), WriteBack, Invalidate ; EFI_UNSUPPORTED, EFI_DEVICE_ERROR) ; II-12.3.3 EnableInterrupt et II-12.3.4 DisableInterrupt (EFI_DEVICE_ERROR si échec) ; II-12.3.5 GetInterruptState (State TRUE si interruptions actives ; EFI_INVALID_PARAMETER si NULL) ; II-12.3.6 Init (EfiCpuInit ; le processeur redémarre, pas de retour ; EFI_UNSUPPORTED si impossible) ; II-12.3.7 RegisterInterruptHandler (InterruptType ; handler NULL = désinstallation ; EFI_ALREADY_STARTED si déjà installé, EFI_INVALID_PARAMETER si NULL et non installé, EFI_UNSUPPORTED ; pas de chaînage des gestionnaires).
- [AGENT] TRÈS PERTINENT : (1) FlushDataCache : la plage est arrondie aux lignes de cache, donc il faut flusher la mémoire partagée avec un DMA avant de la lire côté CPU, et le faire sur des limites alignées (cohérent avec DmaBufferAlignment). (2) Init : reset processeur, à éviter sans accord. (3) RegisterInterruptHandler : un vecteur déjà pris renvoie EFI_ALREADY_STARTED et aucun chaînage n'est possible : l'agent doit détecter ce cas et ne pas écraser le gestionnaire existant (règle DO_NOT_HOOK déjà notée). (4) Désactiver puis réactiver les interruptions doit respecter l'état antérieur (GetInterruptState).

## Statut
- UEFI PI : lignes 1 à 11960 lues. Prochaine : ligne 11961 (suite RegisterInterruptHandler, GetTimerValue, SetMemoryAttributes, Metronome).

## Lignes 11961-12160 — II-12.3.7 fin (InterruptType, SystemContext ; codes) ; II-12.3.8 GetTimerValue (TimerIndex 0..NumberOfTimers-1, TimerValue, TimerPeriod en femtosecondes optionnel ; 0 si fréquence non prévisible ; EFI_UNSUPPORTED si pas de timer lisible) ; II-12.3.9 SetMemoryAttributes (BaseAddress, Length, Attributes ; EFI_INVALID_PARAMETER pour combinaison interdite, ex. UC et WT ensemble ; EFI_ACCESS_DENIED si politique ; EFI_UNSUPPORTED) ; II-12.4 Metronome (GUID 26baccb2-...; WaitForTick(TickNumber) ; TickPeriod en 100 ns, au plus 200 µs ; EFI_TIMEOUT ; ne garantit pas l'absence de tick manqué) ; II-12.5 Monotonic Counter (GUID 1da97072-bddc-4b30-99f1-72a0b56fff2a ; driver runtime ; initialise GetNextMonotonicCount et GetNextHighMonotonicCount) ; II-12.6 RTC (GUID 27cfac87-46cc-11d4-9a38-0090273fc14d ; GetTime, SetTime, GetWakeupTime, SetWakeupTime ; driver runtime) ; II-12.7 Reset (GUID 27cfac88-46cc-11d4-9a38-0090273fc14d ; ResetSystem ; driver runtime).
- [AGENT] Retenu : (1) SetMemoryAttributes : une combinaison de caching incompatible renvoie EFI_INVALID_PARAMETER : l'agent doit valider ses combinaisons avant appel (UC et WT ne vont pas ensemble). (2) Le compteur monotone doit être strictement croissant : utilisable comme horodatage, mais un reset peut le remettre en cause selon la plateforme (à vérifier). (3) WaitForTick peut manquer un tick : ne pas l'utiliser pour mesurer une durée précise sans marge.

## Statut
- UEFI PI : lignes 1 à 12160 lues. Prochaine : ligne 12161 (II-12.8 Runtime Architectural, II-12.9 Security, II-12.10 Timer, II-12.11 Variable).

## Lignes 12161-12360 — II-12.8 Runtime Architectural (GUID b7dfb4e1-052f-449f-87be-9818fc91b733 ; ImageHead et EventHead (listes des images et événements runtime à reloger) ; MemoryDescriptorSize et Version, MemoryMapSize, MemoryMapPhysical (tampon runtime alloué par le Foundation), MemoryMapVirtual (mis à jour après SetVirtualAddressMap), VirtualMode, AtRuntime ; EFI_RUNTIME_IMAGE_ENTRY (ImageBase, ImageSize, RelocationData, Handle, Link) : la relocation virtuelle ne doit pas écraser les données modifiées à l'exécution ; EFI_RUNTIME_EVENT_ENTRY (Type, NotifyTpl, NotifyFunction, NotifyContext, Event doit être en mémoire runtime ; note : le type EFI_EVENT* est une erreur historique) ; le driver installe le protocole avec un pointeur d'interface après avoir initialisé SetVirtualAddressMap, ConvertPointer et CalculateCrc32 ; II-12.9 Security Architectural : II-12.9.1 Security Architectural (GUID a46423e3-4617-49f1-b9ff-d1bfa9115839 ; FileAuthenticationState ; sert au contrôle des sections authentifiées ; EFI_SECURITY_POLICY_PROTOCOL_GUID 78e4d245-cd4d-4a05-a2ba-4743e86cfcab : présence = tenter l'authentification des sections AuthenticationSection).
- [AGENT] TRÈS PERTINENT : (1) EFI_RUNTIME_EVENT_ENTRY.Event doit être en mémoire runtime : un événement stocké en mémoire boot services disparaîtra après ExitBootServices. (2) RelocationData : une écriture de données globales modifiées par le code doit être corrigée par le code lui-même lors du passage en virtuel : oubli fréquent, à vérifier pour tout driver runtime de l'agent. (3) La présence de EFI_SECURITY_POLICY_PROTOCOL_GUID impose la vérification des sections authentifiées : à respecter.

## Statut
- UEFI PI : lignes 1 à 12360 lues. Prochaine : ligne 12361 (suite FileAuthenticationState, Security2, Timer, Variable).

## Lignes 12361-12560 — II-12.9.1.2 FileAuthenticationState (AuthenticationStatus, File ; EFI_SUCCESS = utilisable ; EFI_ACCESS_DENIED = jamais utilisable ; EFI_SECURITY_VIOLATION = non utilisable maintenant, placer en Untrusted ; EFI_INVALID_PARAMETER si File NULL) ; II-12.9.2 Security2 Architectural (GUID 94ab2f58-1438-4ef1-9152-18941a3a0e68 ; FileAuthentication ; publié avant Security et par le même driver ; LoadImage l'appelle sur chaque image, Security seulement après Security2 et seulement pour les images lues par FV) ; II-12.9.2.1 FileAuthentication (DevicePath, FileBuffer, FileSize, BootPolicy ; mesure TCG + vérification Secure Boot + identité ; DevicePath NULL = origine inconnue, politique la plus stricte ; FileBuffer NULL = test de connexion d'un périphérique ; codes : SUCCESS, SECURITY_VIOLATION (Untrusted ou Deferred), ACCESS_DENIED) ; II-12.10 Timer Architectural (GUID 26baccb3-6f42-11d4-bce7-0080c73c8881 ; RegisterHandler (NotifyFunction s'exécute à EFI_TPL_HIGH_LEVEL ; NULL désinscrit ; EFI_ALREADY_STARTED, EFI_INVALID_PARAMETER, EFI_DEVICE_ERROR) ; SetTimerPeriod (100 ns, optionnel) ; GetTimerPeriod ; GenerateSoftInterrupt).
- [AGENT] TRÈS PERTINENT (Secure Boot) : (1) Security2 est appelé sur chaque image chargée : l'agent, s'il est chargé, doit lui-même passer par ce contrôle. (2) EFI_SECURITY_VIOLATION = placé en Untrusted : le chargement n'est pas exécuté ; EFI_ACCESS_DENIED = jamais exécuté. (3) Le gestionnaire de Timer tourne à HIGH_LEVEL : il ne doit faire que le strict nécessaire, aucune allocation ni E/S lente (règle de sécurité des interruptions déjà notée pour le débogage).

## Statut
- UEFI PI : lignes 1 à 12560 lues. Prochaine : ligne 12561 (suite Timer : SetTimerPeriod, GetTimerPeriod, GenerateSoftInterrupt, Variable Architectural).

## Lignes 12561-12760 — II-12.10 Timer (EFI_TIMER_NOTIFY : Time en 100 ns, peut détecter les interruptions manquées ; codes ; SetTimerPeriod 0 = désactivé, pas de masquage CPU ; GetTimerPeriod 0 = désactivé ; GenerateSoftInterrupt, EFI_UNSUPPORTED possible) ; II-12.11 Variable Architectural (GUID 1e5668e2-8481-11d4-bcf1-0080c73c8881 ; GetVariable, GetNextVariableName, SetVariable (volatile seulement), QueryVariableInfo ; écriture non-volatile = protocole d'écriture) ; II-12.12 Variable Write Architectural (GUID 6441f818-6362-4e44-b570-7dba31dd2453 ; met à jour SetVariable) ; II-12.13 Capsule Architectural (GUID 5053697e-2cbc-4819-90d9-0580deee5754 ; CapsuleUpdate, QueryCapsuleCapabilities) ; II-12.14 Watchdog Timer Architectural (GUID 665e3ff5-46cc-11d4-9a38-0090273fc14d ; RegisterHandler (appelé avant ResetSystem par défaut) ; SetTimerPeriod, GetTimerPeriod en 100 ns ; implémenté par timer logiciel ou matériel ; sans handler, reset système).
- [AGENT] TRÈS PERTINENT : (1) Le watchdog réinitialise la machine si rien ne le réarme : l'agent doit le désactiver ou le réarmer explicitement avant d'attendre longtemps (sinon reset inattendu). (2) Variables volatiles seules sans protocole d'écriture : les variables non-volatiles ne doivent pas être écrites par l'agent sans accord (règle NVRAM). (3) CapsuleUpdate : mise à jour firmware, ne pas déclencher sans accord.

## Statut
- UEFI PI : lignes 1 à 12760 lues. Prochaine : ligne 12761 (suite Watchdog, DXE Boot Services Protocol, MP Services, Status Code runtime, puis II-14 etc.).

## Lignes 12761-12960 — II-12.14.1 suite (Time = temps écoulé depuis armement, notification si > TimerPeriod ; codes) ; II-12.14.3 SetTimerPeriod (0 = désactivé) ; II-12.14.4 GetTimerPeriod (EFI_INVALID_PARAMETER si NULL) ; CHAPITRE 13 DXE Boot Services Protocol (MP Services) : II-13.3 vue d'ensemble (obligatoire dès qu'il y a plus d'un cœur ; disponible seulement au boot ; utilisable pour MTRR, ACPI, test mémoire parallèle, diagnostic) ; II-13.4.1 EFI_MP_SERVICES_PROTOCOL (GUID 3fdda605-a76e-4f46-ad29-12f4531b3d08 ; sept services ; idle des AP avant EFI_EVENT_LEGACY_BOOT_GUID ou EFI_EVENT_GROUP_EXIT_BOOT_SERVICES ; état compatible OS après ; les requêtes non bloquantes doivent être terminées avant ces événements ; ordre d'exécution des notifications d'un même groupe non déterministe) ; II-13.4.2 GetNumberOfProcessors (BSP seulement, EFI_DEVICE_ERROR si AP).
- [AGENT] TRÈS PERTINENT : (1) Les requêtes non bloquantes sur les AP doivent être terminées avant ExitBootServices : l'agent qui lance un travail sur un AP en non bloquant doit l'attendre, sinon le travail peut se poursuivre pendant l'OS. (2) Le notifier de sortie ne garantit pas l'ordre : ne pas s'appuyer sur ce qu'un autre driver a fait avant. (3) Le service ne fonctionne que sur le BSP pour la plupart des appels : le code de l'agent doit vérifier qu'il est sur le BSP (WhoAmI) avant d'appeler GetNumberOfProcessors.

## Statut
- UEFI PI : lignes 1 à 12960 lues. Prochaine : ligne 12961 (suite GetProcessorInfo, StartupAllAPs, StartupThisAP, SwitchBSP, EnableDisableAP, WhoAmI du protocole DXE).

## Lignes 12961-13160 — II-13.4.3 GetProcessorInfo (ProcessorNumber 0..N-1, buffer fourni par l'appelant ; BSP seulement) ; EFI_PROCESSOR_INFORMATION (ProcessorId 64 bits ; StatusFlag ; Location ; ExtendedInformation) ; StatusFlag : bit 0x1 = BSP, 0x2 = activé, 0x4 = santé ; tableau des 8 combinaisons (BSP désactivé = invalide ; BSP sain et activé = 0x7) ; EFI_CPU_PHYSICAL_LOCATION (Package, Core, Thread) ; CPU_V2_EXTENDED_TOPOLOGY : EFI_CPU_PHYSICAL_LOCATION2 (Package, Module, Tile, Die, Core, Thread).
- [AGENT] TRÈS PERTINENT (topologie CPU, cohérent avec les notes MADT/APIC) : (1) ProcessorId est l'identifiant matériel unique : l'agent doit le comparer à l'APIC ID de la MADT avant d'associer un cœur à une entrée. (2) Un cœur "non sain" (bit santé à 0) ne doit pas recevoir de travail. (3) Un BSP désactivé est une combinaison invalide : toute lecture de ce cas indique un état corrompu, à signaler.

## Statut
- UEFI PI : lignes 1 à 13160 lues. Prochaine : ligne 13161 (suite EFI_CPU_PHYSICAL_LOCATION2, StartupAllAPs, StartupThisAP, SwitchBSP, EnableDisableAP, WhoAmI du protocole DXE).

## Lignes 13161-13360 — EFI_CPU_PHYSICAL_LOCATION2 (fin) ; II-13.4.4 StartupAllAPs du protocole DXE (WaitEvent NULL = bloquant, sinon non bloquant avec événement signalé ; FailedCpuList alloué par le service, à libérer par l'appelant ; END_OF_CPU_LIST = 0xFFFFFFFF ; EFI_UNSUPPORTED si non bloquant après EFI_EVENT_GROUP_READY_TO_BOOT ; EFI_NOT_READY si un AP est occupé ; EFI_ACCESS et services EFI interdits aux AP sauf mention ; Procedure non MP-safe garantie) ; procédure de non-bloquant en 4 étapes (CreateEvent, StartupAllAPs, CheckEvent/WaitForEvent, examen de FailedCpuList) ; II-13.4.5 StartupThisAP (WaitEvent, Timeout, Finished en sortie ; même principe).
- [AGENT] TRÈS PERTINENT : (1) FailedCpuList est un tampon alloué par le service : libération obligatoire, et la liste se termine par 0xFFFFFFFF : l'agent doit lire jusqu'à ce marqueur. (2) Après ReadyToBoot, aucune requête non bloquante : tout travail de l'agent sur les AP doit être bloquant à partir de ce moment. (3) Une Procedure qui dépasse le timeout est arrêtée au milieu : elle ne doit pas laisser de ressource partagée dans un état incohérent (ex. verrou non relâché).

## Statut
- UEFI PI : lignes 1 à 13360 lues. Prochaine : ligne 13361 (suite StartupThisAP, SwitchBSP, EnableDisableAP, WhoAmI du protocole DXE MP).

## Lignes 13361-13560 — II-13.4.5 StartupThisAP (suite : Finished = TRUE/FALSE en non bloquant ; EFI_UNSUPPORTED après ReadyToBoot en non bloquant ; EFI_TIMEOUT ; EFI_NOT_FOUND ; EFI_INVALID_PARAMETER si BSP ou AP désactivé ou Procedure NULL) ; II-13.4.6 SwitchBSP (EFI_UNSUPPORTED après ReadyToBoot ; EFI_NOT_READY si l'AP est occupé) ; II-13.4.7 EnableDisableAP (HealthFlag, EFI_UNSUPPORTED après ReadyToBoot) ; II-13.4.8 WhoAmI (BSP et AP) ; CHAPITRE 14 DXE runtime protocols : II-14.2 Status Code Runtime Protocol (GUID d2b2b828-0826-48a7-b3df-983c006024f0 ; ReportStatusCode utilisable en runtime ; produit par un driver runtime).
- [AGENT] Retenu : (1) Après ReadyToBoot : SwitchBSP, EnableDisableAP et les requêtes non bloquantes sont refusés : l'agent doit pouvoir s'adapter à EFI_UNSUPPORTED et ne pas réessayer en boucle. (2) WhoAmI est autorisé sur les AP : un AP peut donc savoir qui il est sans passer par le BSP. (3) Le rapport d'état en runtime est disponible : le projet peut l'utiliser pour journaliser après ExitBootServices (sans appeler de service Boot).

## Statut
- UEFI PI : lignes 1 à 13560 lues. Prochaine : ligne 13561 (suite ReportStatusCode runtime, puis CHAPITRE 15 DEPEX grammar, codes d'erreur, GUID).

## Lignes 13561-13760 — II-14.2.1 EFI_STATUS_CODE_PROTOCOL (ReportStatusCode(Type, Value, Instance, CallerId optionnel, Data optionnel) ; aucune disposition garantie ; peut être appelé à TPL HIGH : interdiction d'appeler tout service non garanti à ce niveau (y compris allocation) ; SignalEvent autorisé ; le driver de protocole évalue la vraie sévérité ; Data traité en lecture seule, recopié dans un tampon local avant toute action (au moins 256 octets, perte possible) ; CallerId doit pointer vers un GUID constant ; mêmes masques et codes de type/sévérité que la partie I ; Table 14.1 : EFI_SUCCESS ou EFI_DEVICE_ERROR) ; CHAPITRE 15 grammaire DEPEX (BNF : depex = BEFORE guid | AFTER guid | SOR bool | bool ; bool AND/OR term ; term NOT factor ; factor = bool | TRUE | FALSE ...).
- [AGENT] TRÈS PERTINENT : (1) ReportStatusCode peut être appelé à TPL HIGH : le code de l'agent qui journalise doit n'utiliser aucune allocation ni protocole : tampon statique ou pile. (2) Un tampon de 256 octets peut tronquer les données : l'agent doit tronquer explicitement et le signaler, pas perdre silencieusement. (3) Comportement indéfini si tous les tampons locaux sont pris : l'agent ne doit pas compter sur un rapport d'erreur garanti.

## Statut
- UEFI PI : lignes 1 à 13760 lues. Prochaine : ligne 13761 (suite grammaire DEPEX : exemples, codes d'erreur, GUID de la partie II).

## Lignes 13761-13960 — II-15.2 grammaire DEPEX (fin : <guid> au format 8-4-4-8 octets en hexadécimal 0x ; factor = GUID | END) ; II-15.3 exemples : (1) "EFI_CPU_IO_PROTOCOL_GUID AND EFI_CPU_ARCH_PROTOCOL_GUID END" -> PUSH, PUSH, AND, END ; (2) "AFTER (guid) END" -> 0x01 + GUID + END ; (3) "SOR guid END" -> 0x09, PUSH, GUID, END.
- [AGENT] Retenu : la grammaire ne parle que de la forme de la source : une expression qui combine BEFORE/AFTER avec d'autres opérateurs n'est pas dans la grammaire et doit être refusée par le compilateur (règle déjà relevée à la section II-10). Un GUID mal formé (hexadécimal invalide, nombre de champs) doit être rejeté, pas tronqué.

## Statut
- UEFI PI : lignes 1 à 13960 lues. Prochaine : ligne 13961 (suite exemples DEPEX, puis Appendice II-16 codes d'erreur, II-17 GUID).

## Lignes 13961-14160 — II-15.3 fin (exemple SOR) ; CHAPITRE 16 Appendice codes d'erreur (DXE_ERROR(a) = MAX_BIT|MAX_BIT>>2|a ; EFI_REQUEST_UNLOAD_IMAGE = DXE_ERROR(1) : un driver le renvoie pour être déchargé ; EFI_NOT_AVAILABLE_YET = DXE_ERROR(2) : capacité non encore installée) ; CHAPITRE 17 Appendice GUID (DXE Services Table GUID 05ad34ba-6f02-4214-952e-4da0398e2bb9, stockée en EfiBootServicesData ; HOB List GUID 7739f24c-93d7-11d4-9a3a-0090273fc14d, stockée en EfiBootServicesData) ; PARTIE III Shared Architectural Elements : listes de figures et tableaux (FFS, sections, HOB, Status Codes, classes de codes) ; CHAPITRE 1 introduction (volume III : stockage firmware PI (FV, FFS, fichiers, PPI, protocoles DXE) et HOB ; cibles IHV/OEM et développeurs ; conventions identiques ; little-endian ; champs réservés à zéro, préservés en mise à jour) ; CHAPITRE 2 conception du stockage : 2.1.1 périphériques flash (effacement par secteur, écriture bit à bit de 1 vers 0 seulement, verrouillage lecture/écriture possible, opérations lentes) ; 2.1.2 firmware volumes (nom GUID, taille, format, memory-mapped, sticky write, polarité d'effacement, alignement, lecture/écriture/verrou) ; 2.1.3 système de fichiers firmware (GUID unique par FFS, sert à associer un driver au FV).
- [AGENT] Retenu : (1) Flash : un bit ne peut passer de 0 à 1 qu'en effaçant tout le secteur : le projet ne doit pas écrire un secteur de firmware sans l'effacer ; toute écriture sur flash est une opération destructive à exclure sans accord (règle du projet). (2) FV non memory-mapped : l'agent ne doit pas supposer qu'il peut lire son contenu directement. (3) Le GUID d'un FFS est l'identité du module : vérifier le GUID avant de l'associer à un driver.

## Statut
- UEFI PI : lignes 1 à 14160 lues. Prochaine : ligne 14161 (III-2.1.4 fichiers firmware et sections, suite du chapitre 2).

## Lignes 14161-14360 — III-2.1.4 fichiers firmware (nom GUID unique par FV ; type : Normal 0x00-0xBF, OEM 0xC0-0xDF, Debug 0xE0-0xEF, FV-spécifique 0xF0-0xFF ; alignement ; taille ; PEI : FfsFindNextFile/FindFileByName/GetFileInfo ; DXE : EFI_FIRMWARE_VOLUME2_PROTOCOL ReadFile/ReadSection/WriteFile/GetNextFile) ; III-2.1.4.1 types : RAW 0x01, FREEFORM 0x02, SECURITY_CORE 0x03, PEI_CORE 0x04, DXE_CORE 0x05, PEIM 0x06, DRIVER 0x07, COMBINED_PEIM_DRIVER 0x08, APPLICATION 0x09, MM 0x0A, FIRMWARE_VOLUME_IMAGE 0x0B, COMBINED_MM_DXE 0x0C, MM_CORE 0x0D, MM_STANDALONE 0x0E, MM_CORE_STANDALONE 0x0F, FFS_PAD 0xF0 ; règles de construction de chaque type (un seul PE32/PIC/TE pour PEIM, au plus un DXE_DEPEX et un PEI_DEPEX pour les combinés, PE32 requis pour DRIVER/APPLICATION, FREEFORM avec sous-type GUID unique) ; pad : état 00000111b, ne doit jamais recevoir de données ; III-2.1.5 sections (encapsulation vs feuille ; arbre) ; Table 2.2 types de section : COMPRESSION 0x01, GUID_DEFINED 0x02, DISPOSABLE 0x03, PE32 0x10, PIC 0x11, TE 0x12, DXE_DEPEX 0x13, VERSION 0x14, USER_INTERFACE 0x15, COMPATIBILITY16 0x16, FIRMWARE_VOLUME_IMAGE 0x17, FREEFORM_SUBTYPE_GUID 0x18, RAW 0x19, PEI_DEPEX 0x1B, MM_DEPEX 0x1C ; III-2.2 format binaire standard (trois niveaux : volume, système de fichiers, fichier ; en-tête de FV avec GUID du format FFS).
- [AGENT] Retenu : (1) Pour le projet : un PEIM ou un driver doit contenir exactement la section exécutable attendue ; si plusieurs PE32 sont présents, l'algorithme de sélection est défini par LoadImage : l'agent qui charge un module doit appliquer la même règle et refuser l'ambigu. (2) EFI_SECTION_GUID_DEFINED peut contenir de la donnée d'authentification : ne jamais la sauter sans vérifier. (3) Un fichier pad doit rester vide : ne jamais y écrire.

## Statut
- UEFI PI : lignes 1 à 14360 lues. Prochaine : ligne 14361 (III-2.2 format du volume, en-tête, système de fichiers, fichier).

## Lignes 14361-14560 — III-2.2.1 FV : en-tête EFI_FIRMWARE_VOLUME_HEADER, données décrites par un GUID FFS (FILE_SYSTEM2 ou FILE_SYSTEM3 ; FFS3 pour fichiers > 16 Mo) ; III-2.2.2 FFS : système de fichiers plat sans répertoire (il faut parcourir tout le volume pour lister) ; VTF (Volume Top File, GUID EFI_FFS_VOLUME_TOP_FILE_GUID, doit finir au dernier octet du volume ; pad inséré automatiquement) ; III-2.2.3 fichier FFS (en-tête aligné 8 octets ; fichier sans donnée = 24 octets ; FILE_HEADER2 au-delà de 16 Mo) ; III-2.2.4 sections (en-tête aligné 4 octets, taille sur 24 bits, 0xFFFFFF = taille sur 32 bits suivante, remplissage à zéro) ; III-2.2.5 initialisation FFS : FvCheck (en-tête FV à somme nulle, système de fichiers FFS2/FFS3 sinon non FFS, vérification de chaque fichier, puis espace libre ; échec = volume corrompu et reprise sur crash) ; Exists (en-tête non effacé) ; GetFileState (bit de poids fort de State) ; FileCheck par état : CONSTRUCTION, VALID, DATA_VALID, MARKED_FOR_UPDATE, DELETED, HEADER_INVALID ; DuplicateFileExists (même GUID de fichier, sauf pad) ; CopyFile (sticky write) ; III-2.2.6 accès aux fichiers avant le driver FFS en écriture : SEC/PEI/DXE précoce ne réparent pas ; un fichier MARKED_FOR_UPDATE avec un doublon DATA_VALID : utiliser le doublon (EarlyFfsUpdateCheck) ; recovery autorisé si incohérence.
- [AGENT] TRÈS PERTINENT : (1) Un FV avec somme d'en-tête non nulle ou système de fichiers inconnu n'est pas utilisable : l'agent doit le refuser, pas tenter de le réparer. (2) Une incohérence non réparable déclenche une reprise : le projet ne doit ni réécrire ni "corriger" un FV sans accord explicite. (3) La règle MARKED_FOR_UPDATE + doublon DATA_VALID est la seule réparation permise en lecture : à reprendre pour toute lecture de module par l'agent.

## Statut
- UEFI PI : lignes 1 à 14560 lues. Prochaine : ligne 14561 (suite EarlyFfsUpdateCheck, III-2.2.7 intégrité des fichiers et transitions d'état).

## Lignes 14561-14760 — III-2.2.7 intégrité (trois familles de pannes : générale (bit altéré), effacement interrompu, écriture interrompue ; les bits State seuls ne suffisent pas : IntegrityCheck du header et du fichier) ; III-2.2.8 transitions d'état (seul le bit de poids fort TRUE a un sens ; chaque transition atomique) : création (CONSTRUCTION 00000001b, puis HEADER_VALID 00000011b après les champs Name/Type/Attributes/Size figés, puis DATA_VALID 00000111b) ; suppression (DELETED, 0001xx11b) ; mise à jour (MARKED_FOR_UPDATE 00001111b sur l'ancien, nouveau fichier créé, ancien supprimé) ; fichier > 16 Mo : Size à zéro et ExtendedSize ; III-3 définitions : EFI_FIRMWARE_VOLUME_HEADER (ZeroVector 16 octets, FileSystemGuid, FvLength, Signature "_FVH", Attributes EFI_FVB_ATTRIBUTES_2, HeaderLength, Checksum (somme nulle), ExtHeaderOffset, Reserved, Revision = 2, BlockMap terminé par {0,0}) ; attributs FVB2 (READ_DISABLED_CAP 0x1, READ_ENABLED_CAP 0x2, READ_STATUS 0x4, WRITE_DISABLED_CAP 0x8, WRITE_ENABLED_CAP 0x10, WRITE_STATUS 0x20, LOCK_CAP 0x40, LOCK_STATUS 0x80, STICKY_WRITE 0x200, MEMORY_MAPPED 0x400, ERASE_POLARITY 0x800, READ_LOCK_CAP 0x1000, READ_LOCK_STATUS 0x2000, WRITE_LOCK_CAP 0x4000, WRITE_LOCK_STATUS 0x8000).
- [AGENT] Retenu : (1) Une mise à jour de fichier est atomique par construction : un module en MARKED_FOR_UPDATE avec son doublon valide n'est pas exécuté deux fois. (2) L'en-tête FV a une somme nulle et un Signature "_FVH" : vérifier les deux avant de parser le volume. (3) ZeroVector : les 16 premiers octets sont réservés au vecteur de reset en adresse 0 : ne jamais y écrire de donnée du projet. (4) ERASE_POLARITY inverse le sens de tous les bits State : toujours la lire avant d'interpréter un état de fichier.

## Statut
- UEFI PI : lignes 1 à 14760 lues. Prochaine : ligne 14761 (suite EFI_FIRMWARE_VOLUME_HEADER : EXT_HEADER, FFS file header, EFI_FFS_FILE_HEADER, sections).

## Lignes 14761-14960 — Alignements FVB2 (1 octet à 2 Go, codes 0x00000000 à 0x001F0000, EFI_FVB2_WEAK_ALIGNMENT 0x80000000 ; EFI_FVB2_ALIGNMENT_MASK 0x001F0000) ; description des attributs (lecture/écriture désactivables, verrous, STICKY_WRITE : effacement de bloc obligatoire pour revenir de 1 à 0, MEMORY_MAPPED, ERASE_POLARITY, alignement, WEAK_ALIGNMENT) ; EFI_FV_BLOCK_MAP (NumBlocks, Length) ; EFI_FIRMWARE_VOLUME_EXT_HEADER (FvName, ExtHeaderSize) ; extensions : OEM_TYPE 0x01 (TypeMask pour les types 0xC0..0xDF, tableau de GUID), GUID_TYPE 0x02 (FormatType + données), USED_SIZE 0x03 (UsedSize : le reste doit valoir la polarité d'effacement) ; III-3.2.1.2 FV signés (EFI_FIRMWARE_CONTENTS_SIGNED_GUID, WIN_CERTIFICATE_UEFI_GUID avec PKCS7, RSA2048-SHA256, RSA3072-SHA384, RSA4096-SHA512, RSAPSS3072/4096, ECDSA P-256/P-384) ; structures EFI_CERT_BLOCK_RSA_3072_SHA384 (exposant 0x10001, PKCS1 v1.5), RSA_4096_SHA512, ECDSA P-256 (64 octets clé, 64 octets signature, big-endian), ECDSA P-384 (96 + 96) ; III-3.2.2 FFS : FILE_SYSTEM2_GUID 8c8ce578-8a3d-4f1c-9935-896185c32dd3 ; FILE_SYSTEM3_GUID 5473c07a-3dcb-4dca-bd6f-1e9689e7349a (fichiers >= 16 Mo) ; VTF GUID 1ba0062e-c779-4582-8566-336ae8f78f09 ; III-3.2.3 en-tête de fichier FFS (8 octets alignés, début de la section sur III-3.2.3.1).
- [AGENT] TRÈS PERTINENT (chaîne de confiance) : (1) Un FV signé doit être vérifié avant tout usage : signature cryptographique sur tout le volume. (2) Les tailles des clés et signatures sont fixes : un blob de taille différente est invalide et doit être refusé. (3) Les signatures ECDSA sont en big-endian alors que le reste du format est en little-endian : ne pas inverser par erreur. (4) Ne pas accepter un FFS3 comme FFS2 : les fichiers de 16 Mo et plus n'ont pas le même en-tête.

## Statut
- UEFI PI : lignes 1 à 14960 lues. Prochaine : ligne 14961 (suite EFI_FFS_FILE_HEADER, IntegrityCheck, Type, Attributes, State, Size).

## Lignes 14961-15160 — III-3.2.3.1 EFI_FFS_FILE_HEADER (Name 16 octets, IntegrityCheck 2 octets, Type, Attributes, Size sur 3 octets, State) et FILE_HEADER2 (+ExtendedSize 64 bits quand FFS_ATTRIB_LARGE_FILE) ; taille du fichier = Size - taille de l'en-tête ; fichier vide = 24 octets ; fichier suivant à l'offset aligné 8 octets ; EFI_FFS_INTEGRITY_CHECK (Header : somme de l'en-tête à zéro avec State et File à zéro ; File : somme des données si FFS_ATTRIB_CHECKSUM, sinon 0xAA) ; EFI_FV_FILETYPE (UINT8) ; attributs FFS (LARGE_FILE 0x01, FIXED 0x04, DATA_ALIGNMENT 0x38, CHECKSUM 0x40) ; Table 3.3 alignements FFS (1 o à 16 Mio, codes d'alignement 0 à 7 et DATA_ALIGNMENT2) ; EFI_FFS_FILE_STATE (CONSTRUCTION 0x01, HEADER_VALID 0x02, DATA_VALID 0x04, MARKED_FOR_UPDATE 0x08, DELETED 0x10, HEADER_INVALID 0x20 ; autres bits = polarité d'effacement) ; III-3.2.4 sections : EFI_COMMON_SECTION_HEADER (Size 3 octets, Type ; ExtendedSize 32 bits si Size = 0xFFFFFF ; un en-tête de 4 octets pour une section vide ; section très grande seulement dans un fichier FILE_HEADER2) ; EFI_SECTION_TYPE (UINT8).
- [AGENT] Retenu : (1) Le checksum d'en-tête doit être vérifié avant d'utiliser Size ou Type : un en-tête corrompu peut donner une longueur fausse et faire lire hors du fichier. (2) FFS_ATTRIB_CHECKSUM non levé = champ File fixé à 0xAA : ne pas traiter 0xAA comme une somme réelle. (3) Un alignement de fichier impose souvent un pad avant : l'agent ne doit pas insérer de pad sans respecter l'alignement déclaré.

## Statut
- UEFI PI : lignes 1 à 15160 lues. Prochaine : ligne 15161 (suite types de section, section de chaque type : COMPRESSION, GUID_DEFINED, PE32, DEPEX, VERSION, USER_INTERFACE, etc.).

## Lignes 15161-15360 — Valeurs EFI_SECTION_TYPE (ALL 0x00 joker ; encapsulation COMPRESSION 0x01, GUID_DEFINED 0x02, DISPOSABLE 0x03 ; feuilles PE32 0x10, PIC 0x11, TE 0x12, DXE_DEPEX 0x13, VERSION 0x14, USER_INTERFACE 0x15, COMPATIBILITY16 0x16, FIRMWARE_VOLUME_IMAGE 0x17, FREEFORM_SUBTYPE_GUID 0x18, RAW 0x19, PEI_DEPEX 0x1B, MM_DEPEX 0x1C) ; III-3.2.5 : COMPATIBILITY16 (exécutable IA-32 16 bits) ; COMPRESSION (UncompressedLength, CompressionType : NOT_COMPRESSED 0x00, STANDARD 0x01 = algorithme UEFI 2.0 ; EFI_COMPRESSION_SECTION2 si >= 16 Mo) ; DISPOSABLE (contenu jetable : debug, instructions d'intégration) ; DXE_DEPEX (expression du dispatcher DXE) ; FIRMWARE_VOLUME_IMAGE (FV complet en feuille) ; FREEFORM_SUBTYPE_GUID (début : GUID de sous-type dans l'en-tête).
- [AGENT] Retenu : (1) Une section DISPOSABLE peut être supprimée sans perte fonctionnelle : l'agent ne doit pas s'appuyer sur son contenu pour l'exécution. (2) CompressionType inconnu : refuser la section, ne pas la passer telle quelle. (3) Une section FIRMWARE_VOLUME_IMAGE est un FV complet : le même contrôle de signature et d'en-tête s'applique (cf. lignes précédentes).

## Statut
- UEFI PI : lignes 1 à 15360 lues. Prochaine : ligne 15361 (suite FREEFORM_SUBTYPE_GUID, GUID_DEFINED, PE32, PIC, TE, VERSION, USER_INTERFACE, RAW, PEI_DEPEX, MM_DEPEX).

## Lignes 15361-15560 — FREEFORM_SUBTYPE_GUID (fin : GUID de sous-type défini par le créateur) ; III-3.2.5.7 GUID_DEFINED (SectionDefinitionGuid, DataOffset, Attributes, GuidSpecificHeaderFields ; bits : PROCESSING_REQUIRED 0x01 = traitement nécessaire (chiffrement, compression), AUTH_STATUS_VALID 0x02 = statut d'authentification à reporter ; GUID du signé f9d89e8-9259-4f76-a5af-0c8 9e34023df ; SignatureInfo = WIN_CERTIFICATE_UEFI_GUID) ; III-3.2.5.8 PE32 (PE32+ complet) ; III-3.2.5.9 PEI_DEPEX ; III-3.2.5.10 PIC (image sans relocation) ; III-3.2.5.11 RAW ; III-3.2.5.12 MM_DEPEX (évaluée avant appel en MMRAM, peut référencer protocoles UEFI ou MM) ; III-3.2.5.13 TE (seulement pour PEI Foundation et PEIM) ; III-3.2.5.14 USER_INTERFACE (nom lisible Unicode, au plus une fois par fichier) ; III-3.2.5.15 VERSION (numéro de build + chaîne).
- [AGENT] TRÈS PERTINENT : (1) Une section GUID_DEFINED sans extracteur installé et PROCESSING_REQUIRED levé ne peut pas être lue : l'agent doit la refuser, pas renvoyer les octets bruts. (2) AUTH_STATUS_VALID : le statut reporté doit être relu pour décider d'exécuter ou non. (3) Une section TE n'existe que pour PEI : un chargeur DXE qui rencontre TE doit le refuser. (4) PIC : exécutable sans relocation depuis la flash, donc le code ne doit pas contenir d'adresse absolue.

## Statut
- UEFI PI : lignes 1 à 15560 lues. Prochaine : ligne 15561 (suite VERSION, puis III-3.3 PEI : FV info PPI, FV PPI, Load File PPI, Guided Section Extraction PPI, Decompress PPI).

## Lignes 15561-15760 — III-3.2.5.15 VERSION (fin : BuildNumber UINT16 croissant, VersionString, une seule section par fichier) ; III-3.3 PEI : III-3.3.1 EFI_PEI_FIRMWARE_VOLUME_INFO_PPI (GUID 49edb1c1-bf21-4761-bb12-eb0031aabb39 ; FvFormat, FvInfo, FvInfoSize, ParentFvName, ParentFileName) ; III-3.3.2 FV_INFO2 (GUID ea7ca24b-ded5-4dad-a389-bf827e8f9b38 ; + AuthenticationStatus) ; III-3.3.3 EFI_PEI_FIRMWARE_VOLUME_PPI (GUID = GUID du format FV ; ProcessVolume, FindFileByType, FindFileByName, GetFileInfo, GetVolumeInfo, FindSectionByType, GetFileInfo2, FindSectionByType2 ; Signature "PFVP" ; Revision 0x00010030) ; ProcessVolume (Buffer, BufferSize, FvHandle unique).
- [AGENT] Retenu : (1) Un FV exposé en PEI porte son statut d'authentification (INFO2) : l'agent doit relire ce statut avant d'utiliser le FV. (2) Le nombre de version (BuildNumber) n'est pas une garantie d'intégrité : à ne pas confondre avec une vérification. (3) La Signature "PFVP" permet de vérifier le type de l'interface : l'agent doit comparer la signature avant d'appeler les fonctions.

## Statut
- UEFI PI : lignes 1 à 15760 lues. Prochaine : ligne 15761 (suite ProcessVolume, FindFileByType, FindFileByName, GetFileInfo, GetVolumeInfo, FindSectionByType...).

## Lignes 15761-15960 — III-3.3.3.2 ProcessVolume (début de règle : Buffer/BufferSize = début et taille du FV si memory-mapped ; sinon buffer d'information ; EFI_VOLUME_CORRUPTED si corrompu) ; III-3.3.3.3 FindFileByType (SearchType, FvHandle, FileHandle NULL = début ; EFI_NOT_FOUND) ; III-3.3.3.4 FindFileByName (FileName, FvHandle NULL = tous les volumes ; EFI_INVALID_PARAMETER si NULL) ; III-3.3.3.5 GetFileInfo (EFI_INVALID_PARAMETER si handle invalide) ; III-3.3.3.6 GetFileInfo2 ; III-3.3.3.7 GetVolumeInfo (EFI_INVALID_PARAMETER si handle invalide) ; III-3.3.3.8 FindSectionByType (début : SearchType, FileHandle).
- [AGENT] Retenu : (1) ProcessVolume renvoie EFI_VOLUME_CORRUPTED : l'agent doit alors cesser d'utiliser ce FV, pas poursuivre avec des données partielles. (2) FindFileByName avec FvHandle NULL peut trouver un fichier dans un autre volume que prévu : vérifier le FvHandle renvoyé avant d'accepter le fichier. (3) Les handles de fichiers sont uniques dans le système : ne pas les réutiliser entre phases.

## Statut
- UEFI PI : lignes 1 à 15960 lues. Prochaine : ligne 15961 (suite FindSectionByType, FindSectionByType2, puis III-3.3.4 Load File PPI).

## Lignes 15961-16160 — III-3.3.3.8 fin ; III-3.3.3.9 FindSectionByType2 (SearchInstance, AuthenticationStatus) ; III-3.3.4 Load File PPI (GUID b9e0abfe-5979-4914-977f-6dee78c278a6 ; LoadFile(FileHandle, ImageAddress, ImageSize, EntryPoint, AuthenticationState) ; charge et relocalise ; doit au minimum supporter PE/COFF et TE ; EFI_NOT_SUPPORTED si copie nécessaire sans mémoire permanente ; EFI_WARN_BUFFER_TOO_SMALL = image XIP exécutable quand même ; EFI_OUT_OF_RESOURCES pour image non XIP ; attendre EFI_PEI_PERMANENT_MEMORY_INSTALLED_PPI et Load File avant exécution depuis la mémoire) ; III-3.3.5 Guided Section Extraction PPI (ExtractSection(InputSection, OutputBuffer, OutputSize, AuthenticationStatus) ; OutputBuffer alloué en mémoire permanente PEI ; si PROCESSING_REQUIRED est à zéro, pointe simplement sur le contenu ; AuthenticationStatus à zéro si AUTH_STATUS_VALID absent).
- [AGENT] TRÈS PERTINENT : (1) Load File supporte les XIP sans copie : une image XIP n'est pas relocalisée, donc son contenu en flash est exécuté tel quel ; l'agent ne doit pas tenter de la modifier. (2) EFI_WARN_BUFFER_TOO_SMALL n'est pas une erreur : l'image est utilisable, mais l'agent ne doit pas l'ignorer si la mémoire est insuffisante pour son propre tampon. (3) Le statut d'authentification de l'extraction doit être relu : une section non authentifiée ne doit pas être utilisée quand la politique l'interdit.

## Statut
- UEFI PI : lignes 1 à 16160 lues. Prochaine : ligne 16161 (suite Guided Section Extraction, puis III-3.3.6 Decompress PPI, III-3.4 DXE FV2 et Block2, Guided Section Extraction protocol).

## Lignes 16161-16360 — III-3.3.5 suite : statut d'authentification (AUTH_STATUS_VALID : bits PLATFORM_OVERRIDE 0x01, IMAGE_SIGNED 0x02, NOT_TESTED 0x04, TEST_FAILED 0x08 ; combinaisons possibles : non signée, override, signée et testée réussie, signée non testée, signée et échouée ; deux couches d'encapsulation peuvent s'agréger par OU) ; codes ExtractSection (SUCCESS, OUT_OF_RESOURCES, INVALID_PARAMETER si GUID ne correspond pas) ; III-3.3.6 Decompress PPI (GUID 1a36e4e7-fab6-476a-8e75-695a0576fdd7 ; Decompress(InputSection, OutputBuffer, OutputSize) ; sans elle, les sections compressées sont ignorées ; EFI_UNSUPPORTED si algorithme inconnu) ; III-3.4 DXE : III-3.4.1 EFI_FIRMWARE_VOLUME2_PROTOCOL (GUID 220e73b6-6bdb-4413-8405-b974b108619a ; GetVolumeAttributes, SetVolumeAttributes, ReadFile, ReadSection, WriteFile, GetNextFile, KeySize, ParentHandle, GetInfo, SetInfo ; un FV non visible en DXE n'a pas de protocole ; abstraction : flash ou fichier).
- [AGENT] TRÈS PERTINENT : (1) AUTH_STATUS_NOT_TESTED : une signature existe mais n'a pas été vérifiée : l'agent ne doit pas l'accepter comme signée. (2) TEST_FAILED = à rejeter. (3) L'agrégation par OU de deux couches peut faire apparaître TEST_FAILED sans qu'une seule couche ne l'ait déclaré : l'agent doit regarder le résultat agrégé, pas une couche isolée. (4) WriteFile et SetVolumeAttributes du protocole FV2 modifient le firmware : à ne jamais appeler sans accord.

## Statut
- UEFI PI : lignes 1 à 16360 lues. Prochaine : ligne 16361 (suite GetVolumeAttributes, SetVolumeAttributes, ReadFile, ReadSection, WriteFile, GetNextFile).

## Lignes 16361-16560 — III-3.4.1.2 GetVolumeAttributes (EFI_FV_ATTRIBUTES 64 bits : READ_DISABLE_CAP 0x1, READ_ENABLE_CAP 0x2, READ_STATUS 0x4, WRITE_DISABLE_CAP 0x8, WRITE_ENABLE_CAP 0x10, WRITE_STATUS 0x20, LOCK_CAP 0x40, LOCK_STATUS 0x80, WRITE_POLICY_RELIABLE 0x100, READ/WRITE_LOCK_CAP/STATUS 0x1000-0x8000, ALIGNMENT 0x1F0000 ; appelable seulement à TPL ≤ NOTIFY) ; III-3.4.1.3 SetVolumeAttributes (seuls READ_STATUS, WRITE_STATUS, LOCK_STATUS modifiables et seulement si la capacité existe ; LOCK_STATUS verrouille les attributs sans affecter lecture/écriture ; EFI_ACCESS_DENIED si verrouillé ; EFI_INVALID_PARAMETER si la capacité manque) ; III-3.4.1.4 ReadFile (NameGuid, Buffer (NULL = seulement taille, type, attributs), BufferSize, FoundType, FileAttributes, AuthenticationStatus ; un seul fichier valide par GUID dans un FV).
- [AGENT] TRÈS PERTINENT : (1) SetVolumeAttributes permet de désactiver l'écriture ou la lecture du firmware : opération à ne pas faire sans accord, et un LOCK_STATUS irréversible jusqu'au reset bloque toute modification future. (2) ReadFile avec Buffer NULL donne la taille sans lire le contenu : utiliser ce mode pour dimensionner un tampon, puis relire, sans supposer que la taille n'a pas changé. (3) WRITE_POLICY_RELIABLE : seule une écriture fiable peut être utilisée pour mettre à jour un composant critique.

## Statut
- UEFI PI : lignes 1 à 16560 lues. Prochaine : ligne 16561 (suite ReadFile, ReadSection, WriteFile, GetNextFile, GetInfo, SetInfo du protocole FV2).

## Lignes 16561-16760 — III-3.4.1.4 ReadFile (suite : Buffer caller-allocated, EFI_WARN_BUFFER_TOO_SMALL avec taille requise ; Buffer NULL = allocation par ReadFile en EfiBootServicesData ; callable à TPL ≤ NOTIFY ; EFI_FV_FILE_ATTRIBUTES : ALIGNMENT 0x1F, FIXED 0x100, MEMORY_MAPPED 0x200 ; Buffer invalide si non memory-mapped ; Table 3.19 alignements) ; III-3.4.1.5 ReadSection (NameGuid, SectionType, SectionInstance (base 0), Buffer, BufferSize, AuthenticationStatus ; recherche en profondeur d'abord, gauche à droite ; EFI_PROTOCOL_ERROR si l'extracteur GUIDé requis manque) ; III-3.4.1.6 WriteFile (NumberOfFiles, WritePolicy, FileData[] ; EFI_FV_UNRELIABLE_WRITE 0 = aucune garantie, possibles pertes de tous les fichiers ; EFI_FV_RELIABLE_WRITE 1 = tous écrits ou aucun après reprise ; taille 0 = suppression d'un fichier, une à la fois, interdite dans un lot).
- [AGENT] TRÈS PERTINENT : (1) ReadSection EFI_PROTOCOL_ERROR : une section peut exister mais ne pas être lisible sans un extracteur : l'agent doit le signaler et ne pas considérer la section comme absente. (2) WriteFile : écriture non fiable possible de tous les fichiers : interdite au projet sans accord explicite. (3) WriteFile avec taille 0 supprime un fichier : ne pas l'utiliser pour nettoyer sans accord.

## Statut
- UEFI PI : lignes 1 à 16760 lues. Prochaine : ligne 16761 (suite EFI_FV_WRITE_FILE_DATA, GetNextFile, GetInfo, SetInfo du protocole FV2).

## Lignes 16761-16960 — III-3.4.1.6 WriteFile (EFI_FV_WRITE_FILE_DATA : NameGuid, Type, FileAttributes, Buffer, BufferSize ; codes : EFI_WRITE_PROTECTED, EFI_NOT_FOUND (suppression absente), EFI_VOLUME_FULL implicite, EFI_INVALID_PARAMETER pour politique ou type inconnu ; sauf EFI_DEVICE_ERROR, aucun code d'erreur ne modifie le volume ; EFI_DEVICE_ERROR = volume possiblement corrompu, réparation nécessaire) ; III-3.4.1.7 GetNextFile (Key 0 = recommencer, FileType filtre, NameGuid, Attributes, Size ; Key opaque à ne pas modifier ; EFI_NOT_FOUND en fin) ; III-3.4.1.8 GetInfo (InformationType GUID, BufferSize, Buffer ; EFI_UNSUPPORTED si type inconnu ; EFI_VOLUME_CORRUPTED) ; III-3.4.1.9 SetInfo (EFI_WRITE_PROTECTED, EFI_VOLUME_FULL, EFI_BAD_BUFFER_SIZE) ; III-3.4.2 Firmware Volume Block2 Protocol (optionnel, GUID 8f644fa9-e850-4db1-9ce2-0b44698e8da4 ; contrôle des périphériques firmware en blocs ; début : GetAttributes, SetAttributes).
- [AGENT] TRÈS PERTINENT : (1) EFI_DEVICE_ERROR pendant WriteFile : le firmware peut être corrompu : l'agent doit le journaliser comme erreur critique, ne pas réécrire pour "réparer" sans accord. (2) Key de GetNextFile est opaque : ne pas l'interpréter ni la modifier. (3) FVB2 : contrôle direct des blocs flash : une opération destructive à exclure sans accord.

## Statut
- UEFI PI : lignes 1 à 16960 lues. Prochaine : ligne 16961 (suite FVB2 : SetAttributes, EraseBlocks, Read, Write, GetPhysicalAddress, GetBlockSize, GetVolumeAttributes..).

## Lignes 16961-17160 — III-3.4.2.1 FVB2 (structure : GetAttributes, SetAttributes, GetPhysicalAddress, GetBlockSize, Read, Write, EraseBlocks, ParentHandle ; description : bas niveau, l'accès fichier passe par le protocole FV2 ; seul le driver FS se lie à FVB2) ; III-3.4.2.2 GetAttributes ; III-3.4.2.3 SetAttributes (EFI_INVALID_PARAMETER si en conflit avec les capacités) ; III-3.4.2.4 GetPhysicalAddress (EFI_UNSUPPORTED si non memory-mapped) ; III-3.4.2.5 GetBlockSize (Lba, BlockSize, NumberOfBlocks ; EFI_INVALID_PARAMETER si hors plage ; sert à lire la carte de blocs) ; III-3.4.2.6 Read (Lba, Offset, NumBytes, Buffer ; EFI_ACCESS_DENIED si lecture désactivée, sans modifier le tampon ; ne doit jamais franchir une frontière de bloc : NumBytes = octets réellement lus, lecture partielle possible ; EFI_BAD_BUFFER_SIZE).
- [AGENT] TRÈS PERTINENT : (1) Read ne franchit pas une frontière de bloc : une lecture de N octets peut ne renvoyer qu'une partie ; l'agent doit boucler sur NumBytes et ne jamais supposer une lecture complète. (2) Le tampon n'est pas modifié si la lecture est refusée (ACCESS_DENIED) : ne pas interpréter son contenu comme valide. (3) GetPhysicalAddress EFI_UNSUPPORTED : un FV non memory-mapped doit être lu par Read, pas par pointeur direct.

## Statut
- UEFI PI : lignes 1 à 17160 lues. Prochaine : ligne 17161 (suite Read, Write, EraseBlocks, puis III-3.5 SMM Firmware Volume).

## Lignes 17161-17360 — III-3.4.2.6 Read (fin : EFI_DEVICE_ERROR si bloc défaillant) ; III-3.4.2.7 Write (si sticky write : bits de la plage doivent déjà être en polarité d'effacement, sinon résultat imprévisible ; sinon EraseBlocks d'abord ; EFI_ACCESS_DENIED si WriteDisabled sans modifier ; franchissement de frontière = écriture partielle ; tout doit être vidé en matériel avant retour) ; III-3.4.2.8 EraseBlocks (liste variable de tuples (LBA, nombre), terminée par EFI_LBA_LIST_TERMINATOR 0xFFFFFFFFFFFFFFFF ; toute la liste vérifiée avant d'effacer ; EFI_INVALID_PARAMETER si LBA hors volume ; EFI_DEVICE_ERROR peut laisser un bloc partiellement effacé) ; III-3.4.3 Guided Section Extraction Protocol (GUID propre à l'extracteur ; ExtractSection : OutputBuffer alloué en pool boot services, libéré par l'appelant ; statut dans bits 19:16 et 3:0 identiques ; EFI_SECURITY_POLICY_PROTOCOL_GUID présent = contournement possible de l'authentification complète avec EFI_AUTH_STATUS_PLATFORM_OVERRIDE) ; III-3.5 SMM : III-3.5.1 SMM FV protocol (GUID 19e9da84-072b-4274-b32e-0c0802e717a5, même API que FV2) ; III-3.5.2 SMM FVB protocol (GUID d326d041-bd31-4c01-b5a8-628be87f0653, même API que FVB2).
- [AGENT] TRÈS PERTINENT : (1) EFI_DEVICE_ERROR pendant EraseBlocks : le firmware peut être partiellement effacé, donc inutilisable : signaler, ne pas réutiliser. (2) EFI_SECURITY_POLICY_PROTOCOL permet de sauter l'authentification : l'agent ne doit jamais s'appuyer sur PLATFORM_OVERRIDE comme preuve d'authenticité. (3) Le protocole SMM reprend l'API DXE : mêmes précautions, en plus de la sensibilité SMM (cf. notes MM).

## Statut
- UEFI PI : lignes 1 à 17360 lues. Prochaine : ligne 17361 (CHAPITRE 4 de la Partie III : Hand-Off Blocks).

## Lignes 17361-17560 — CHAPITRE 4 HOB design : III-4.1 termes (producteur / consommateur de HOB ; PEI et DXE ne sont que des implémentations possibles) ; III-4.2 vue d'ensemble (HOB = conteneur de données séquentiel en mémoire disponible) ; III-4.4 liste HOB : premier = PHIT, dernier = End of HOB List ; seuls les producteurs modifient, la liste est en lecture seule côté consommateur ; un reset demandé par le consommateur ne doit pas modifier le mode de boot dans la HOB ; III-4.5.1 construction initiale : PHIT, allocation de pile BSP (ou BSPStore Itanium), resource descriptor de la zone du producteur (présent, initialisé, testé) ; III-4.5.2 règles : (1) chaque HOB commence par un en-tête générique ; (2) données de boot services valables seulement jusqu'à la fin de la phase consommatrice ; (3) relocation possible, aucun pointeur vers la liste ; (4) multiples de 8 octets ; (5) PHIT aligné 8 ; (6) ajout en fin, seulement côté producteur ; (7) jamais de suppression ; III-4.5.3 ajout (taille, contrôle NewHobSize <= EfiFreeMemoryTop - EfiFreeMemoryBottom, construction à EfiFreeMemoryBottom, puis avancer) ; III-5 définitions : HobType 0x0001 HANDOFF, 0x0002 MEMORY_ALLOCATION, 0x0003 RESOURCE_DESCRIPTOR, 0x0004 GUID_EXTENSION, 0x0005 FV, 0x0006 CPU, 0x0007 MEMORY_POOL, 0x0009 FV2, 0x000A UNUSED (LOAD_PEIM), 0x000B UEFI_CAPSULE, 0x000C FV3, 0x000D RESOURCE_DESCRIPTOR2, 0xFFFE UNUSED, 0xFFFF END_OF_HOB_LIST ; EFI_HOB_GENERIC_HEADER (HobType 16 bits, HobLength 16 bits, Reserved 32 bits = 0) ; PHIT (Version 0x000a, BootMode, EfiMemoryTop, EfiMemoryBottom, EfiFreeMemoryTop, EfiFreeMemoryBottom, EfiEndOfHobList ; adresses en 4 KiB pour Top/FreeTop).
- [AGENT] TRÈS PERTINENT pour le projet : (1) Le contrôle NewHobSize <= Free est la seule vérification : la liste HOB n'est pas bornée autrement : l'agent qui lit une HOB doit contrôler HobLength (multiple de 8, ≥ 8, sans dépasser la fin) avant de sauter. (2) La règle "HOB sans pointeur" : une HOB est copiable sans correction : ne jamais stocker de pointeur vers la liste HOB sans recalcul après relocation. (3) PHIT : EfiMemoryBottom..EfiMemoryTop = zone utilisée par le producteur, EfiFreeMemoryBottom..Top = libre ; toute écriture hors de la zone libre est une erreur.

## Statut
- UEFI PI : lignes 1 à 17560 lues. Prochaine : ligne 17561 (suite EFI_HOB_MEMORY_ALLOCATION, en-tête d'allocation, STACK, BSP_STORE, MODULE).

## Lignes 17561-17760 — III-5.4.1 Memory Allocation HOB (GUID ou Name ; EFI_HOB_MEMORY_ALLOCATION_HEADER : Name, MemoryBaseAddress, MemoryLength, MemoryType 32 bits, Reserved 4 octets ; le consommateur ne lit que base/longueur/type, il ne déplace pas la donnée sauf s'il connaît le Name ; deux composants ne doivent pas modifier la même zone ; III-5.4.2 BSP Stack HOB (GUID 4ed4bf27-4092-42e9-807d-527b1d00c9bd ; décrit la pile du BSP ; doit être valide au handoff ; le consommateur peut la déplacer) ; III-5.4.3 BSPSTORE HOB (Itanium seulement, GUID 564b33cd-c92a-4593-90bf-2473e43c6322) ; III-5.4.4 Memory Allocation Module HOB (GUID f8e21975-0899-4f58-a4be-5525a9c6d77a ; ModuleName, EntryPoint = adresse du FV memory-mapped contenant le fichier du consommateur ; sert à créer le handle image du consommateur) ; III-5.5 Resource Descriptor HOB (début : Owner GUID, ResourceType, ResourceAttribute, PhysicalStart, ResourceLength).
- [AGENT] TRÈS PERTINENT : (1) La HOB Memory Allocation déclare qu'une zone est déjà utilisée : l'agent ne doit pas allouer dans une zone ainsi déclarée, même si elle semble libre. (2) La pile BSP décrite doit être exacte au handoff : une pile qui n'est pas mise à jour après réallocation pointe vers une zone libre : à vérifier avant d'utiliser la pile du projet. (3) Le champ Owner des resource descriptors est ignoré du consommateur : ne pas s'y fier pour décider de la propriété.

## Statut
- UEFI PI : lignes 1 à 17760 lues. Prochaine : ligne 17761 (suite Resource Descriptor HOB : ResourceType, ResourceAttribute, types de ressources).

## Lignes 17761-17960 — Resource Descriptor HOB (fin : ResourceLength ; description : déclare ce qui existe sur le bus hôte, pas l'usage ; le consommateur construit la carte GCD ; les ressources fixes (ex. RTC 70h/71h) peuvent être allouées plus tard par un driver chipset ; Owner ignoré par la GCD ; un consommateur qui prend une ressource déjà initialisée doit la désallouer avant) ; EFI_RESOURCE_TYPE (SYSTEM_MEMORY 0, MEMORY_MAPPED_IO 1, IO 2, FIRMWARE_DEVICE 3, MEMORY_MAPPED_IO_PORT 4, MEMORY_RESERVED 5, IO_RESERVED 6, MEMORY_UNACCEPTED 7 ; valeur ≥ 8 = illégale) ; EFI_RESOURCE_ATTRIBUTE_TYPE : attributs PRESENT 0x1, INITIALIZED 0x2, TESTED 0x4, ECC SINGLE 0x8, MULTIPLE 0x10, ECC_RESERVED 0x20/0x40, READ_PROTECTED 0x80, WRITE_PROTECTED 0x100 (cacheabilité depuis PI 1.4, use READ_ONLY_PROTECTED), EXECUTION_PROTECTED 0x200, PERSISTENT 0x800000, MORE_RELIABLE 0x2000000 ; capacités : UNCACHEABLE 0x400, WRITE_COMBINEABLE 0x800, WRITE_THROUGH 0x1000, WRITE_BACK 0x2000, 16/32/64_BIT_IO 0x4000/0x8000/0x10000, UNCACHED_EXPORTED 0x20000, READ_ONLY_PROTECTED 0x40000, protégeables (READ 0x100000, WRITE 0x200000, EXECUTION 0x400000, PERSISTABLE 0x1000000, READ_ONLY 0x80000), ENCRYPTED 0x4000000, SPECIAL_PURPOSE 0x8000000, HOT_PLUGGABLE 0x10000000.
- [AGENT] TRÈS PERTINENT (projet) : (1) ResourceType >= 8 : la HOB est illégale : rejeter, ne pas deviner. (2) EFI_RESOURCE_ATTRIBUTE_TESTED absent : la mémoire n'a pas été testée : ne pas l'utiliser pour des données du projet. (3) ENCRYPTED et UNACCEPTED (lien SEV-SNP déjà noté) : une région UNACCEPTED doit être acceptée avant tout usage. (4) WRITE_PROTECTED ≠ cacheabilité depuis PI 1.4 : ne pas interpréter l'attribut 0x100 comme protection en écriture sur des fichiers d'anciennes versions.

## Statut
- UEFI PI : lignes 1 à 17960 lues. Prochaine : ligne 17961 (suite Resource Descriptor HOB et EFI_HOB_RESOURCE_DESCRIPTOR2, GUID Extension HOB, FV HOB, CPU HOB, Memory Pool HOB, Capsule HOB, End of HOB List HOB).

## Lignes 17961-18160 — Attributs : CRYPTO (capable de protection par cryptographie CPU), SPM (Specific-Purpose Memory, réservée à un usage ; l'OS évite d'y mettre son code) , MORE_RELIABLE, HOT_PLUGGABLE (retirable à l'exécution) ; Table 5.2 (types producteur : SystemMemory, MMIO, I/O et leurs attributs admis) ; III-5.5.2 EFI_HOB_RESOURCE_DESCRIPTOR2 (Owner, ResourceType, ResourceCapabilities, PhysicalStart, ResourceLength, ResourceMemoryAttributes = attributs mémoire à mapper, selon GetMemoryMap) ; III-5.6 GUID Extension HOB (Header, Name ; données propres au GUID ; le consommateur doit connaître le GUID pour interpréter, contrat producteur/consommateur) ; III-5.7 FV HOB (EFI_HOB_FIRMWARE_VOLUME : BaseAddress memory-mapped, Length ; le consommateur lit les FV pour trouver des drivers et le fichier de handoff) ; III-5.7.2 FV2 (BaseAddress, Length, FvName, FileName : FV extrait d'un fichier, évite de le retraiter) ; III-5.7.3 FV3 (début : authentification).
- [AGENT] TRÈS PERTINENT : (1) SPM = mémoire réservée à un autre usage : ne pas l'utiliser comme RAM de travail. (2) HOT_PLUGGABLE : une région retirable ne doit pas porter de données persistantes du projet. (3) GUID Extension HOB : la HOB n'a de sens que pour le GUID qui la définit : l'agent ne doit interpréter qu'une HOB dont le GUID est connu, sinon l'ignorer. (4) FV HOB : BaseAddress et Length sont des bornes à vérifier avant tout accès, comme pour toute structure lue.

## Statut
- UEFI PI : lignes 1 à 18160 lues. Prochaine : ligne 18161 (suite EFI_HOB_FIRMWARE_VOLUME3 : authentification, puis CPU HOB, Memory Pool HOB, UEFI Capsule HOB, Unused HOB, End of HOB List, SMRAM HOB).

## Lignes 18161-18360 — III-5.7.3 FV3 (BaseAddress, Length, AuthenticationStatus, ExtractedFv BOOLEAN, FvName et FileName valides seulement si ExtractedFv ; le consommateur doit fournir l'authentification correspondant à AuthenticationStatus) ; III-5.8 CPU HOB (SizeOfMemorySpace, SizeOfIoSpace en bits d'adressage, Reserved 6 octets à zéro ; sert à initialiser la GCD) ; III-5.9 Memory Pool HOB (tampon libre pour le producteur, ignorable par le consommateur) ; III-5.10 UEFI Capsule HOB (BaseAddress, Length ; créée après UpdateCapsule avec CAPSULE_FLAGS_POPULATE_SYSTEM_TABLE) ; III-5.11 Unused HOB (0xFFFE, à ignorer) ; III-5.12 End of HOB List (0xFFFF, obligatoire en dernier, sinon liste mal formée) ; III-5.13 SMRAM HOB (GUID 6dadf1d1-d4cc-4910-bb6e-82b1fd80ff3d ; NumberOfSmmReservedRegions, Descriptor[] = régions candidates SMRAM) ; début CHAPITRE 6 Status Codes (architecture de codes de statut, classes/sous-classes/opérations).
- [AGENT] TRÈS PERTINENT (projet) : (1) Une liste HOB sans End of HOB List est mal formée : le lecteur doit détecter la fin par la longueur et le type END, pas par la seule taille de la HOB. (2) CPU HOB : SizeOfMemorySpace/SizeOfIoSpace bornent les adresses valables : toute adresse hors de ces bornes est invalide, à rejeter. (3) SMRAM : régions réservées au mode SMM : l'agent ne doit ni les allouer ni y écrire (cohérent avec les notes MM). (4) Un Unused HOB doit être sauté, sans être interprété.

## Statut
- UEFI PI : lignes 1 à 18360 lues. Prochaine : ligne 18361 (CHAPITRE 6 Status Codes : termes, classes Computing Unit, User-Accessible Peripheral, I/O Bus, Software).

## Lignes 18361-18560 — III-6.1.1 organisation (overview, classes, définitions de code) ; III-6.2 termes (debug code, error code, progress code, status code, status code driver = producteur de EFI_STATUS_CODE_PROTOCOL) ; III-6.3 types : pour chaque couple classe/sous-classe trois ensembles d'opérations (progress, error, debug) ; valeurs 0x0000-0x0FFF communes, 0x1000-0x7FFF réservées par la spec, 0x8000-0xFFFF OEM ; III-6.3.1 classes (Computing Unit, User-Accessible Peripheral, I/O Bus = matériel ; Host Software = logiciel) ; III-6.3.2 numéros d'instance (0 à N-1 ; 0xFFFFFFFF = non disponible) ; numérotation cohérente avec ACPI ; III-6.4.1 Computing Unit (progress : INIT_BEGIN, INIT_END ; error : NON_SPECIFIC, DISABLED, NOT_SUPPORTED, NOT_DETECTED, NOT_CONFIGURED ; sous-classes Unspecified, Host Processor, Firmware, IO processor, Cache, Memory, Chipset, Manageability) ; Host Processor (instance = numéro de handle MP Services ; progress POWER_ON_INIT, CACHE_INIT, RAM_INIT, MEMORY_CONTROLLER_INIT, IO_INIT, BSP_SELECT, BSP_RESELECT, AP_INIT, SMM_INIT ; erreurs INVALID_TYPE, INVALID_SPEED, MISMATCH, TIMER_EXPIRED).
- [AGENT] Retenu : (1) Les codes 0x8000-0xFFFF sont OEM : l'agent ne doit pas les interpréter sans connaître le fournisseur. (2) 0xFFFFFFFF = instance inconnue : ne pas l'utiliser comme indice de cœur. (3) Le numéro d'instance d'un processeur doit correspondre à l'ordre ACPI : cohérent avec les notes MADT (APIC ID).

## Statut
- UEFI PI : lignes 1 à 18560 lues. Prochaine : ligne 18561 (suite Host Processor, Firmware, IO Processor, Cache, Memory, Chipset, Manageability, puis User-Accessible Peripheral).

## Lignes 18561-18760 — Host Processor (suite : BSP_SELECT, BSP_RESELECT, AP_INIT par le BSP courant, SMM_INIT ; erreurs : BIST, IERR, THERMAL, LOW/HIGH_VOLTAGE, CACHE, MICROCODE_UPDATE, CORRECTABLE, UNCORRECTABLE, NO_MICROCODE_UPDATE) ; Firmware Processor (erreurs HARD_FAIL, SOFT_FAIL, COMM_ERROR) ; I/O Processor = même que Host ; Cache (PRESENCE_DETECT, CONFIGURATION ; erreurs INVALID_TYPE, INVALID_SPEED, INVALID_SIZE, MISMATCH) ; Memory (progress : SPD_READ, PRESENCE_DETECT, TIMING, CONFIGURING, OPTIMIZING, INIT, TEST ; erreurs : INVALID_TYPE, INVALID_SPEED, CORRECTABLE, UNCORRECTABLE (dont échecs de comparaison en test), SPD_FAIL, INVALID_SIZE, MISMATCH, S3_RESUME_FAIL, UPDATE_FAIL, NONE_DETECTED, NONE_USEFUL ; instance = numéro de DIMM).
- [AGENT] Retenu : (1) Un BIST réussi n'exclut pas une erreur CORRECTABLE : à remonter. (2) MICROCODE_UPDATE failed ou NO_MICROCODE_UPDATE : le cœur peut exécuter une microarchitecture non corrigée : à signaler. (3) S3_RESUME_FAIL : reprise S3 échouée, à traiter comme un événement grave (cohérent avec les notes de reprise). (4) UNCORRECTABLE mémoire : les données de l'agent dans ce DIMM ne sont pas fiables.

## Statut
- UEFI PI : lignes 1 à 18760 lues. Prochaine : ligne 18761 (suite Memory subclass, Chipset, Manageability, User-Accessible Peripheral Class, puis I/O Bus Class).

## Lignes 18761-18960 — Memory (fin : Related Definitions) ; Chipset (progress : PEI CAR SB/NB init, PEI MEM SB/NB init, DXE HB init, DXE NB init, DXE NB SMM init, DXE SB RT init, DXE SB init, DXE SB SMM init, DXE SB devices init ; erreurs : BAD_BATTERY, DXE NB/SB ERROR, INTRUDER_DETECT = accès physique aux composants internes) ; Manageability (erreurs REDFISH_COMMUNICATION_ERROR, REDFISH_HOST_INTERFACE_ERROR, REDFISH_BOOTSTRAP_CREDENTIAL_ERROR) ; III-6.4.2 User-Accessible Peripheral (instance 0 = inconnue ; progress INIT, RESET, DISABLE, PRESENCE_DETECT, ENABLE, RECONFIG, DETECTED, REMOVED ; erreurs NON_SPECIFIC, DISABLED, NOT_SUPPORTED, NOT_DETECTED, NOT_CONFIGURED, INTERFACE_ERROR, CONTROLLER_ERROR, INPUT_ERROR, OUTPUT_ERROR, RESOURCE_CONFLICT) ; sous-classes : Unspecified, Keyboard, Mouse, Local console, Remote console, Serial port, Parallel port, Fixed media.
- [AGENT] Retenu : (1) INTRUDER_DETECT : signal d'intrusion physique à journaliser et à remonter, et l'agent doit éviter les opérations sensibles si ce signal est actif. (2) REDFISH_BOOTSTRAP_CREDENTIAL_ERROR : des identifiants de démarrage non obtenus : ne jamais utiliser de repli par défaut sans accord. (3) RESOURCE_CONFLICT : deux composants réclament la même ressource : l'agent ne doit pas forcer l'accès.

## Statut
- UEFI PI : lignes 1 à 18960 lues. Prochaine : ligne 18961 (suite sous-classes User-Accessible Peripheral : Keyboard, Mouse, consoles, ports, Fixed media, Removable media, Audio, LCD, Network).

## Lignes 18961-19160 — Sous-classes Peripheral (Removable media, Audio input/output, LCD, Network, Docking station, TPM 0x80-0xFF réservé OEM) ; Unspecified (progress/error réservés) ; Keyboard (progress CLEAR_BUFFER, SELF_TEST ; erreurs LOCKED, STUCK_KEY, BUFFER_FULL ; ExtendedData = chemin du périphérique) ; Mouse (SELF_TEST ; LOCKED) ; Local Console (GPU/VGA directs, rien de défini) ; Remote Console (série/LAN) ; Serial Port (CLEAR_BUFFER ; ExtendedData = device handle) ; Parallel Port (début).
- [AGENT] Retenu : (1) STUCK_KEY et LOCKED : une entrée clavier verrouillée ou collée ne doit pas être prise comme une saisie utilisateur par le projet. (2) Les consoles distantes (série, LAN) exposent la session à un tiers : à ne pas utiliser pour des données sensibles sans accord. (3) Le TPM est une sous-classe dédiée : tout accès TPM passe par cette classe et doit être journalisé.

## Statut
- UEFI PI : lignes 1 à 19160 lues. Prochaine : ligne 19161 (suite Parallel Port, Fixed media, Removable media, Audio, LCD, Network, Docking, TPM).

## Lignes 19161-19360 — Sous-classes Peripheral (fin) : Parallel Port, Fixed Media (périphériques BLOCK_IO durs), Removable Media (floppy, LS-120), Audio Input, Audio Output, LCD Device, Network Device (pas de progress/erreur spécifiques, réservés) ; III-6.4.3 I/O Bus Class (instance ignorée, ExtendedData = chemin du contrôleur hôte, par ex. root bridge PCI) ; progress : INIT, RESET, DISABLE, DETECT, ENABLE, RECONFIG, HOTPLUG (0x0007 pour INIT, 0x1000-0x7FFF réservé, 0x8000-0xFFFF OEM) ; erreurs : NON_SPECIFIC, DISABLED, NOT_SUPPORTED, NOT_DETECTED, NOT_CONFIGURED, INTERFACE_ERROR, CONTROLLER_ERROR, READ_ERROR, WRITE_ERROR, RESOURCE_CONFLICT.
- [AGENT] TRÈS PERTINENT (projet, bus PCI/USB/NVMe) : (1) RESET du bus réinitialise TOUS les périphériques du bus : opération destructive, à ne pas déclencher sans accord (les disques du bus perdent leur état). (2) READ_ERROR / WRITE_ERROR d'un bus : à remonter comme erreur matérielle (cohérent avec les codes NVMe déjà notés). (3) HOTPLUG : un périphérique ajouté à chaud doit être réinitialisé par le firmware avant usage. (4) RESOURCE_CONFLICT : ne pas forcer d'adresse BAR (règle du projet déjà notée).

## Statut
- UEFI PI : lignes 1 à 19360 lues. Prochaine : ligne 19361 (sous-classes I/O Bus : Unspecified, PCI, USB, IBA, AGP, PCCard, LPC, SCSI, ATA/ATAPI/SATA, FC, IP network, SMBus, I2C).

## Lignes 19361-19560 — Fin des codes erreur I/O Bus (Extended Data, descriptions), puis III-6.4.3.4 Subclasses : tableau 6.31 (liste des sous-classes I/O Bus : Unspecified, PCI, USB, IBA, AGP, PC Card, LPC, SCSI, ATA/ATAPI, SATA, FC, IP network, SMBus, I2C ; 0x0D-0x7F réservé, 0x80-0xFF OEM). Sous-classe Unspecified : pas de codes spécifiques (plages 0x1000-0x7FFF réservées). Sous-classe PCI : progress EFI_IOB_PCI_BUS_ENUM (énumération des bus sous un root bridge), EFI_IOB_PCI_RES_ALLOC (allocation des ressources aux devices sous un host bridge), EFI_IOB_PCI_HPC_INIT (init du contrôleur hotplug PCI) ; erreurs EFI_IOB_PCI_EC_PERR (parity error PCI) et EFI_IOB_PCI_EC_SERR (system error PCI), extended data = chemin device du root bridge / du contrôleur fautif. Sous-classes USB, IBA, AGP, PC Card, LPC : aucun code spécifique (réservés).
- [AGENT] PERTINENT (PCI/USB/NVMe du projet) : PERR et SERR sont des erreurs PCI à remonter avec le chemin du device fautif (utile pour diagnostic ; ne pas les masquer). La sous-classe PCI est la seule qui précise des codes ; pour USB/xHCI, rien de normatif à ce niveau dans ce document (voir xHCI et USB spec).

## Statut
- UEFI PI : lignes 1 à 19560 lues ; notes jusqu'à 19560 ajoutées ci-dessus pour 19161-19560. Prochaine : ligne 19561 (LPC, SCSI, ATA/ATAPI, SATA, FC, IP network, SMBus, I2C).

## Lignes 19561-20460 — Fin des sous-classes I/O Bus : LPC, SCSI, FC, IP Network, SMBus, I2C (aucun code propre, plages 0x1000-0x7FFF réservées) ; ATA/ATAPI/SATA (progress SMART_ENABLE, SMART_DISABLE, SMART_OVERTHRESHOLD, SMART_UNDERTHRESHOLD ; erreurs SMART_NOTSUPPORTED, SMART_DISABLED). III-6.5 Host Software Class (instance non utilisée) ; progress communs : INIT, LOAD (LoadImage), INIT_BEGIN (StartImage), INIT_END, AUTHENTICATE_BEGIN/END, INPUT_WAIT, USER_SETUP ; erreurs communes : NON_SPECIFIC, LOAD_ERROR, INVALID_PARAMETER, UNSUPPORTED, INVALID_BUFFER, OUT_OF_RESOURCES, ABORTED, ILLEGAL_SOFTWARE_STATE (données = EFI_DEBUG_ASSERT_DATA), ILLEGAL_HARDWARE_STATE, START_ERROR, BAD_DATE_TIME, CFG_INVALID, CFG_CLR_REQUEST, CFG_DEFAULT, PWD_INVALID, PWD_CLR_REQUEST, PWD_CLEARED, EVENT_LOG_FULL, WRITE_PROTECTED, FV_CORRUPTED, INCONSISTENT_MEMORY_MAP (le système redémarre) ; 0x0100-0x03FF = exceptions EBC / IA-32 / Itanium. Sous-classes logicielles (table 6.47) : Unspecified, SEC, PEI_CORE, PEI_MODULE, DXE_CORE, DXE_BS_DRIVER, DXE_RT_DRIVER, SMM_DRIVER, EFI_APPLICATION, EFI_OS_LOADER, EFI_RT, exceptions EBC/IA32/IPF/X64/ARM/AARCH64/RISCV/LOONGARCH, PEI_SERVICE, EFI_BOOT_SERVICE, EFI_RUNTIME_SERVICE, EFI_DXE_SERVICE. SEC : ENTRY_POINT, HANDOFF_TO_NEXT (services de code d'état souvent absents en SEC). PEI Foundation : ENTRY_POINT, HANDOFF_TO_NEXT, RETURN_TO_LAST ; erreurs DXE_CORRUPT, DXEIPL_NOT_FOUND, MEMORY_NOT_INSTALLED. PEI Module : RECOVERY_BEGIN, CAPSULE_LOAD, CAPSULE_START, RECOVERY_USER, RECOVERY_AUTO, S3_BOOT_SCRIPT, OS_WAKE, S3_STARTED ; erreurs NO_RECOVERY_CAPSULE, INVALID_CAPSULE_DESCRIPTOR, S3_RESUME_PPI_NOT_FOUND, S3_BOOT_SCRIPT_ERROR, S3_OS_WAKE_ERROR, S3_RESUME_FAILED, RECOVERY_PPI_NOT_FOUND, RECOVERY_FAILED, S3_RESUME_ERROR, INVALID_CAPSULE. DXE Foundation : ENTRY_POINT, HANDOFF_TO_NEXT, RETURN_TO_LAST, START_DRIVER, ARCH_READY ; erreur NO_ARCH (protocoles architecturaux manquants en fin de dispatch). DXE BS Driver : LEGACY_OPROM_INIT, READY_TO_BOOT_EVENT, LEGACY_BOOT_EVENT, EXIT_BOOT_SERVICES_EVENT, VIRTUAL_ADDRESS_CHANGE_EVENT, VARIABLE_SERVICES_INIT, VARIABLE_RECLAIM, CONFIG_RESET, CSM_INIT, BOOT_OPTION_COMPLETE ; erreurs LEGACY_OPROM_NO_SPACE, INVALID_PASSWORD, BOOT_OPTION_LOAD_ERROR, BOOT_OPTION_FAILED, INVALID_IDE_PASSWORD ; ATTEMPT_BOOT_ORDER_EVENT. DXE RT Driver, SMM Driver, EFI Application, OS Loader : rien de propre. Runtime : ENTRY_POINT, RETURN_TO_LAST, HANDOFF_TO_NEXT. PEI Services : un code par service (INSTALL_PPI … FFS_REGISTER_FOR_SHADOW) ; erreurs RESET_NOT_AVAILABLE, MEMORY_INSTALLED_TWICE. Boot Services : un code par service (RAISE_TPL … CREATE_EVENT_EX) ; RAISE_TPL, RESTORE_TPL et SIGNAL_EVENT sont des codes invalides car le pilote de codes d'état utilise lui-même ces services. Début Runtime Services subclass.
- [AGENT] PERTINENT (chargeur de l'OS) : (1) Notre chargeur relève de la sous-classe EFI_SOFTWARE_EFI_OS_LOADER ; elle n'a aucun code propre, donc seuls les codes communs Host Software s'appliquent. (2) EXIT_BOOT_SERVICES_EVENT et VIRTUAL_ADDRESS_CHANGE_EVENT sont les deux jalons que le firmware signale au passage vers l'OS : utiles pour dater un blocage. (3) BOOT_OPTION_LOAD_ERROR / BOOT_OPTION_FAILED : c'est ce que le firmware émet si notre BOOTX64.EFI ne se charge pas ou rend une erreur. (4) FV_CORRUPTED, INCONSISTENT_MEMORY_MAP, S3_RESUME_FAILED : événements graves à remonter tels quels. (5) VARIABLE_RECLAIM : le firmware réécrit le magasin de variables ; ne pas couper l'alimentation à ce moment (règle NVRAM déjà notée). (6) SMART over threshold côté ATA : sans objet pour le NVMe du poste, à noter seulement.

## Statut
- UEFI PI : lignes 1 à 20460 lues. Prochaine : ligne 20461 (Runtime Services subclass, DXE Services subclass, puis définitions de code).

## Lignes 20461-21160 — Runtime Services subclass (un code par service : GET_TIME … QUERY_VARIABLE_INFO) ; DXE Services subclass (ADD/ALLOCATE/FREE/REMOVE_MEMORY_SPACE, GET_MEMORY_SPACE_DESCRIPTOR, SET_MEMORY_SPACE_ATTRIBUTES, GET_MEMORY_SPACE_MAP, mêmes pour IO_SPACE, DISPATCH, SCHEDULE, TRUST, PROCESS_FIRMWARE_VOLUME). III-6.6 Code Definitions : EFI_STATUS_CODE_DATA { UINT16 HeaderSize ; UINT16 Size ; EFI_GUID Type } ; EFI_STATUS_CODE_DATA_TYPE_STRING_GUID = 92D11080-496F-4D95-BE7E-037488382B0A (chaîne ASCII, Unicode ou jeton HII ; le handle HII change d'un démarrage à l'autre) ; EFI_STATUS_CODE_SPECIFIC_DATA_GUID = 335984bd-e805-409a-b8f8-d27ece5ff7a6 (format dépendant de la valeur du code ; interdit pour les plages OEM). Partition des codes d'opération : 0x0000-0x0FFF communs à la classe, EFI_SUBCLASS_SPECIFIC = 0x1000, EFI_OEM_SPECIFIC = 0x8000 ; un seul code debug : EFI_DC_UNSPECIFIED = 0. Données étendues communes : EFI_DEVICE_PATH_EXTENDED_DATA (le chemin est inclus, pas un pointeur ; Size = 0 pour un périphérique virtuel ; préféré aux handles car les handles changent à chaque démarrage), EFI_DEVICE_HANDLE_EXTENDED_DATA (handle valable seulement au moment du rapport), EFI_RESOURCE_ALLOC_FAILURE_ERROR_DATA (Bar, DevicePathSize, ReqResSize, AllocResSize, puis chemin, ressources demandées et allouées au format descripteur ACPI 2.0). III-6.7 Classes : EFI_COMPUTING_UNIT = 0x00000000, EFI_PERIPHERAL = 0x01000000, EFI_IO_BUS = 0x02000000, EFI_SOFTWARE = 0x03000000 (4-127 réservé, 128-255 OEM). Computing Unit : sous-classes UNSPECIFIED 0x00000, HOST_PROCESSOR 0x10000, FIRMWARE_PROCESSOR 0x20000, IO_PROCESSOR 0x30000, CACHE 0x40000, MEMORY 0x50000, CHIPSET 0x60000, MANAGEABILITY 0x70000 ; progress communs INIT_BEGIN 0, INIT_END 1 ; valeurs numériques des codes Host Processor (POWER_ON_INIT 0x1000 … SMM_INIT 0x1008), Cache, Memory (SPD_READ 0x1000 … TEST 0x1006), Chipset (0x1000 … 0x100A) ; erreurs communes NON_SPECIFIC 0 … NOT_CONFIGURED 4 ; erreurs Host Processor 0x1000 (INVALID_TYPE) … 0x100D (NO_MICROCODE_UPDATE), Firmware Processor, Cache, Memory (0x1000 … 0x100A NONE_USEFUL), Chipset (BAD_BATTERY, DXE_NB_ERROR, DXE_SB_ERROR, INTRUDER_DETECT). Données étendues Host Processor : VOLTAGE_ERROR_DATA (Voltage, Threshold : sous le seuil = tension trop basse, au-dessus = trop haute), MICROCODE_UPDATE_ERROR_DATA (Version), TIMER_EXPIRED_ERROR_DATA (TimerLimit en secondes).
- [AGENT] PERTINENT : (1) Un code d'état complet = classe (octet haut) | sous-classe (octet suivant) | opération (16 bits bas) : c'est la règle de décodage à appliquer si l'agent lit un journal de codes du firmware. (2) Les handles ne doivent jamais être conservés d'un démarrage à l'autre ; garder les chemins de périphérique (règle déjà notée, confirmée ici). (3) EFI_RESOURCE_ALLOC_FAILURE_ERROR_DATA donne le BAR et les ressources demandées et obtenues : c'est la donnée à lire si un périphérique PCI (NVMe, xHCI, Wi-Fi) n'a pas reçu ses ressources ; l'agent ne réattribue rien lui-même. (4) SET_VARIABLE, RESET_SYSTEM, UPDATE_CAPSULE restent des écritures soumises à accord explicite.

## Statut
- UEFI PI : lignes 1 à 21160 lues. Prochaine : ligne 21161 (suite des données étendues Computing Unit, puis classes Peripheral, I/O Bus, Software : définitions).

## Lignes 21161-21960 — Données étendues Computing Unit (fin) : EFI_HOST_PROCESSOR_MISMATCH_ERROR_DATA (Instance du second processeur, Attributes en champ de bits : SPEED 0x0001, FSB_SPEED 0x0002, FAMILY 0x0004, MODEL 0x0008, STEPPING 0x0010, CACHE_SIZE 0x0020, OEM1-4 0x1000-0x8000) ; THERMAL_ERROR_DATA (Temperature, Threshold) ; EFI_CACHE_INIT_DATA (Level à partir de 1, Type : DataOnly, InstrOnly, Both, Unspecified) ; CPU_DISABLED_ERROR_DATA (Cause : INTERNAL_ERROR 0x0001, THERMAL_ERROR 0x0002, SELFTEST_FAILURE 0x0004, PREBOOT_TIMEOUT 0x0008, FAILED_TO_START 0x0010, CONFIG_ERROR 0x0020, USER_SELECTION 0x0080, BY_ASSOCIATION 0x0100, UNSPECIFIED 0x8000 ; SoftwareDisabled TRUE = absent des tables ACPI mais répond encore aux IPI, FALSE = désactivé matériellement, invisible) ; EFI_MEMORY_EXTENDED_ERROR_DATA (Granularity : OTHER 1, UNKNOWN 2, DEVICE 3, PARTITION 4 ; Operation : OTHER 1, UNKNOWN 2, READ 3, WRITE 4, PARTIAL_WRITE 5 ; Syndrome ; Address physique ; Resolution en octets) ; EFI_STATUS_CODE_DIMM_NUMBER (Array, Device ; 0xfffe = plusieurs, 0xffff = tous) ; MEMORY_MODULE_MISMATCH_ERROR_DATA ; EFI_MEMORY_RANGE_EXTENDED_DATA (Start, Length). III-6.7.2 Peripheral : sous-classes KEYBOARD 0x10000, MOUSE 0x20000, LOCAL_CONSOLE 0x30000, REMOTE_CONSOLE 0x40000, SERIAL_PORT 0x50000, PARALLEL_PORT 0x60000, FIXED_MEDIA 0x70000, REMOVABLE_MEDIA 0x80000, AUDIO_INPUT 0x90000, AUDIO_OUTPUT 0xA0000, LCD_DEVICE 0xB0000, NETWORK 0xC0000, DOCKING 0xD0000, TPM 0xE0000 (toutes | 0x01000000) ; progress INIT 0, RESET 1, DISABLE 2, PRESENCE_DETECT 3, ENABLE 4, RECONFIG 5, DETECTED 6, REMOVED 7 ; clavier CLEAR_BUFFER 0x1000, SELF_TEST 0x1001 ; erreurs NON_SPECIFIC 0 … RESOURCE_CONFLICT 9 ; clavier LOCKED 0x1000, STUCK_KEY 0x1001, BUFFER_FULL 0x1002. III-6.7.3 I/O Bus : sous-classes PCI 0x10000, USB 0x20000, IBA 0x30000, AGP 0x40000, PC_CARD 0x50000, LPC 0x60000, SCSI 0x70000, ATA_ATAPI 0x80000, FC 0x90000, IP_NETWORK 0xA0000, SMBUS 0xB0000, I2C 0xC0000 (toutes | 0x02000000) ; progress INIT 0, RESET 1, DISABLE 2, DETECT 3, ENABLE 4, RECONFIG 5, HOTPLUG 6 ; PCI BUS_ENUM 0x1000, RES_ALLOC 0x1001, HPC_INIT 0x1002 ; ATA SMART 0x1000-0x1003 ; erreurs NON_SPECIFIC 0 … WRITE_ERROR 8.
- [AGENT] Retenu : (1) Écart avec le tableau lu plus haut : le tableau 6.29 donnait 0x0007 comme borne après HOTPLUG, les définitions donnent INIT = 0 … HOTPLUG = 6 ; ce sont les définitions qui font foi. (2) Un cœur « SoftwareDisabled » répond encore aux IPI : l'agent ne doit pas réveiller un cœur absent de la MADT. (3) Une erreur mémoire donne l'adresse physique et la résolution : c'est la plage à exclure de l'allocateur de l'OS. (4) Les valeurs PCI (0x02011000 = énumération, 0x02011001 = allocation des ressources) servent à repérer dans un journal le moment où le firmware configure NVMe, xHCI et la carte Wi-Fi.

## Statut
- UEFI PI : lignes 1 à 21960 lues. Prochaine : ligne 21961 (fin des erreurs I/O Bus, puis définitions de la classe Software).

## Lignes 21961-22860 — Fin I/O Bus : RESOURCE_CONFLICT 9, PCI PERR 0x1000, SERR 0x1001, ATA SMART_NOTSUPPORTED 0x1000, SMART_DISABLED 0x1001 ; données étendues I/O Bus = DEVICE_PATH, DEVICE_HANDLE, RESOURCE_ALLOC_FAILURE. III-6.7.4 Software : valeurs des sous-classes (| 0x03000000) : SEC 0x10000, PEI_CORE 0x20000, PEI_MODULE 0x30000, DXE_CORE 0x40000, DXE_BS_DRIVER 0x50000, DXE_RT_DRIVER 0x60000, SMM_DRIVER 0x70000, EFI_APPLICATION 0x80000, EFI_OS_LOADER 0x90000, RT 0xA0000, AL 0xB0000, EBC_EXCEPTION 0xC0000, IA32_EXCEPTION 0xD0000, IPF_EXCEPTION 0xE0000, PEI_SERVICE 0xF0000, EFI_BOOT_SERVICE 0x100000, EFI_RUNTIME_SERVICE 0x110000, EFI_DXE_SERVICE 0x120000, X64_EXCEPTION 0x130000, ARM 0x140000, AARCH64 0x150000, LOONGARCH 0x160000. Progress communs : INIT 0, LOAD 1, INIT_BEGIN 2, INIT_END 3, AUTHENTICATE_BEGIN 4, AUTHENTICATE_END 5, INPUT_WAIT 6, USER_SETUP 7. Valeurs par sous-classe : SEC (0x1000-0x1001), PEI Core (0x1000-0x1002), PEI Module (0x1000-0x1007), DXE Core (ENTRY_POINT 0x1000 … ARCH_READY 0x1004), DXE BS Driver (LEGACY_OPROM_INIT 0x1000, READY_TO_BOOT_EVENT 0x1001, LEGACY_BOOT_EVENT 0x1002, EXIT_BOOT_SERVICES_EVENT 0x1003, VIRTUAL_ADDRESS_CHANGE_EVENT 0x1004, VARIABLE_SERVICES_INIT 0x1005, VARIABLE_RECLAIM 0x1006, ATTEMPT_BOOT_ORDER_EVENT 0x1007, CONFIG_RESET 0x1008, CSM_INIT 0x1009, BOOT_OPTION_COMPLETE 0x100A), RT (0x1000-0x1002), PEI Services (0x1000-0x1016, trou 0x1011-0x1012), Boot Services (RAISE_TPL 0x1000 … EXIT_BOOT_SERVICES 0x1019 … CREATE_EVENT_EX 0x102A), Runtime Services (GET_TIME 0x1000 … SET_VARIABLE 0x1008, RESET_SYSTEM 0x100A, UPDATE_CAPSULE 0x100B, QUERY_VARIABLE_INFO 0x100D), DXE Services (0x1000-0x1010), DXE RT Driver (S0 0x1000 … S5 0x1005). Erreurs communes : NON_SPECIFIC 0 … INCONSISTENT_MEMORY_MAP 0x14 ; PEI Core, PEI Module (0x1000-0x1009), DXE Core NO_ARCH 0x1000, DXE BS (0x1000-0x1004). Exceptions : les codes EBC, IA32, IPF, X64 (DIVIDE_ERROR, DEBUG, NMI, BREAKPOINT, OVERFLOW, BOUND, INVALID_OPCODE, DOUBLE_FAULT, INVALID_TSS, SEG_NOT_PRESENT, STACK_FAULT, GP_FAULT, PAGE_FAULT, FP_ERROR, ALIGNMENT_CHECK, MACHINE_CHECK, SIMD), ARM, AArch64, RISC-V, LoongArch reprennent les constantes EXCEPT_* du Debug Support Protocol. Données étendues : EFI_DEBUG_ASSERT_DATA (LineNumber, FileNameSize, FileName), EFI_STATUS_CODE_EXCEP_EXTENDED_DATA (union des contextes processeur, dont EFI_SYSTEM_CONTEXT_X64), EFI_STATUS_CODE_START_EXTENDED_DATA (ControllerHandle, DriverBindingHandle, DevicePathSize, RemainingDevicePath), EFI_LEGACY_OPROM_EXTENDED_DATA (DeviceHandle, RomImageBase), EFI_RETURN_STATUS_EXTENDED_DATA (ReturnStatus). Début du chapitre III-7 Report Status Code Routers (un pilote générique par phase ; il consomme le protocole Report Status Code et produit le Handler PPI/Protocol ; chaque consommateur enregistre un rappel).
- [AGENT] PERTINENT : (1) Code complet de notre cas : chargeur OS = 0x03090000 | opération ; sortie des services de démarrage signalée par le firmware = 0x03051003 (EXIT_BOOT_SERVICES_EVENT) ; appel du service lui-même = 0x03101019. (2) Une exception x64 survenue dans le firmware est rapportée en classe 0x03130000 avec le contexte EFI_SYSTEM_CONTEXT_X64 : même numérotation de vecteurs que celle du CPU (0 = #DE … 14 = #PF, 18 = #MC, 19 = #XM), cohérente avec APM vol. 2. (3) Coquilles du texte source à ne pas recopier : CAPSULE_START a la même valeur que CAPSULE_LOAD (0x1001) et deux parenthèses manquent ; se fier aux en-têtes de référence et non à ce texte pour ces deux lignes. (4) Les codes Sx (S0-S5) du pilote runtime indiquent la transition de veille demandée : utile pour tracer un arrêt ou une mise en veille.

## Statut
- UEFI PI : lignes 1 à 22860 lues. Prochaine : ligne 22861 (III-7 Report Status Code Routers : PPI et protocoles Handler).

## Lignes 22861-23760 — III-7.2 Routeurs de codes d'état : EFI_RSC_HANDLER_PROTOCOL (GUID 86212936-0e76-41c8-a03a-2af2fc1c39e2 ; Register(Callback, Tpl), Unregister(Callback)) ; le rappel reçoit (CodeType, Value, Instance, CallerId, Data) ; il est appelé au TPL enregistré (TPL_HIGH s'il n'a pas de contrainte) ; il doit être désenregistré avant libération et à ExitBootServices, sauf s'il est runtime : pilote de type runtime, pas de désenregistrement, aucun appel aux services de démarrage. Retours : SUCCESS, INVALID_PARAMETER (NULL), OUT_OF_RESOURCES, ALREADY_STARTED ; Unregister : NOT_FOUND. EFI_PEI_RSC_HANDLER_PPI (GUID 0065d394-9951-4144-82a3-0afc8579c251), même principe, rappel avec PeiServices. EFI_SMM_RSC_HANDLER_PROTOCOL (GUID 2ff29fa7-5e80-4ed9-b380-017d3c554ff4), même principe en SMM. Chapitre III-8 PCD : EFI_PCD_PROTOCOL (GUID 13a3f0f6-264a-3ef0-f2e0-dec512342f34) : SetSku, Get8/16/32/64, GetPtr, GetBool, GetSize, Set8/16/32/64, SetPtr, SetBool, CallbackOnSet, CancelCallback, GetNextToken, GetNextTokenSpace ; appelant au TPL_APPLICATION ; base de réglages de la plateforme, quel que soit le support (données en lecture seule ou variable EFI). SetSku : SKU 0 = défaut ; jeton non « SKU enabled » = valeur unique ; SKU invalide = repli sur le défaut ; défaut sans valeur = résultat imprévisible. Get* : jeton invalide = résultat imprévisible ; GetPtr : aucune hypothèse sur l'alignement du pointeur. Set* : contrôle de taille, retours SUCCESS, INVALID_PARAMETER (taille incompatible), NOT_FOUND. Début SetPtr.
- [AGENT] Retenu : (1) Un rappel de codes d'état serait le moyen propre, pour une application UEFI de diagnostic, de recevoir les codes du firmware pendant les services de démarrage ; il faut le désenregistrer avant ExitBootServices. Lecture seule, sans risque. (2) PCD : les Get* sont des lectures ; les Set* modifient la configuration de la plateforme et peuvent finir dans une variable EFI, donc écriture NVRAM possible : interdit sans accord explicite. (3) Un jeton PCD invalide donne un résultat imprévisible et non une erreur : ne jamais lire un jeton deviné, énumérer d'abord avec GetNextTokenSpace / GetNextToken.

## Statut
- UEFI PI : lignes 1 à 23760 lues. Prochaine : ligne 23761 (PCD : SetPtr, SetBool, CallbackOnSet, GetNextToken, puis Get Info Protocol et PPI PCD).

## Lignes 23761-25160 — Fin PCD (protocole) : SetPtr (SizeOfValue renvoie la taille maximale si trop grand), SetBool, CallbackOnSet(Guid optionnel, CallBackToken, CallBackFunction) ; le rappel reçoit (Guid, CallBackToken, TokenData, TokenDataSize) ; CancelCallback ; GetNextToken (partir de 0 ; liste creuse) ; GetNextTokenSpace (*Guid NULL = premier espace). EFI_GET_PCD_INFO_PROTOCOL (GUID fd0f4478-0efd-461d-ba2d-e58c45fd5f5e) : GetInfo → EFI_PCD_INFO { PcdType (8, 16, 32, 64, BOOL, PTR), PcdSize, PcdName } ; GetSku (0 si non défini). III-8.2 EFI_PEI_PCD_PPI (GUID 01f34d25-4de2-23ad-3ff3-36353ff323f1) : mêmes fonctions qu'en DXE ; EFI_GET_PCD_INFO_PPI (GUID a60c6b59-e459-425d-9c69-0bcc9cb27d81). FIN DE LA PARTIE III (ligne 24925). PARTIE IV — Management Mode Core Interface. IV-1 : définitions (MM = environnement isolé, SMM sur x86, TrustZone sur ARM ; MMI = instruction ou interruption d'entrée ; MMRAM = mémoire réservée, cachée puis verrouillée, non rouvrable avant le reset suivant ; MMST = table système MM ; RSM = sortie). Trois volets : MM Dispatch, MM Initialization, MMI Management. Initialisation en mode traditionnel : en SEC, en PEI (HOB EFI_PEI_MM_CORE_GUID avec le drapeau MM_CORE_LOADED) ou en DXE (EFI_MM_ACCESS_PROTOCOL décrit les régions MMRAM ; EFI_MM_CONTROL_PROTOCOL génère des MMI synchrones ; EFI_MM_CONFIGURATION_PROTOCOL donne les zones à exclure et RegisterMmEntry ; l'IPL crée le tas MMRAM, charge la MM Foundation qui produit la MMST, publie EFI_MM_BASE_PROTOCOL, puis Communicate() transmet le pointeur de la System Table ; EFI_DXE_MM_READY_TO_LOCK_PROTOCOL avant les options de démarrage, puis fermeture et verrouillage facultatifs de la MMRAM). Mode Standalone : initialisation en PEI (PPI MM_ACCESS, MM_CONTROL, MM_CONFIGURATION, MM_COMMUNICATION3) ou en SEC ; la MM Foundation doit avoir la convention d'appel de la phase DXE (64 bits si DXE est 64 bits). IV-1.6 Entrée et sortie : le vecteur d'entrée sauve l'état CPU, choisit un seul CPU pour entrer dans la Foundation, qui met à jour la MMST, appelle les handlers racine par MmiManage(NULL) et revient ; puis restauration et reprise. Pilotes : MM (type de fichier EFI_FV_FILETYPE_MM, lancé une fois en MMRAM, DEPEX MM), combinés MM/DXE (EFI_FV_FILETYPE_COMBINED_MM_DXE, lancés deux fois), Standalone (EFI_FV_FILETYPE_MM_STANDALONE, point d'entrée MM_IMAGE_ENTRY_POINT(ImageHandle, MmSystemTable)) ; pas de SOR pour les pilotes MM. Pendant l'initialisation un pilote MM traditionnel voit UEFI et MM (sauf Exit et ExitBootServices) ; à l'exécution il ne voit que les protocoles MM, ne doit plus toucher à la System Table ni aux services, doit avoir fermé ses événements. Handlers MMI : racine (HandlerType NULL ; détecte, efface, désactive les sources), contrôleur enfant (GUID de la source parente), feuille ; EFI_INTERRUPT_PENDING = source non calmée. Services MMST : E/S bloquantes minimales, allocateur en MMRAM, petite base de protocoles. IV-2.1 EFI_MM_ENTRY_POINT(MmEntryContext) ; EFI_MM_ENTRY_CONTEXT { MmStartupThisAp, CurrentlyExecutingCpu, NumberOfCpus, CpuSaveStateSize, CpuSaveState }.
- [AGENT] TRÈS PERTINENT (l'OS tourne sous le SMM du firmware) : (1) La MMRAM (TSEG) est fermée et verrouillée avant le démarrage de l'OS ; notre noyau ne peut ni la lire ni la modifier et ne doit pas essayer : c'est une zone à exclure de la carte mémoire, déjà marquée réservée. (2) Une SMI peut interrompre l'OS à tout moment, sur tous les cœurs, de façon invisible : tout délai mesuré par l'agent peut contenir du temps SMM ; ne pas conclure à un défaut matériel sur un seul écart de temps. (3) Générer une MMI logicielle (écriture sur le port de commande SMI) lance du code firmware avec tous les droits : interdit sans accord explicite. (4) L'état CPU sauvé à l'entrée SMM se trouve dans la zone de sauvegarde décrite dans APM vol. 2 (chapitre SMM), à relire à ce moment-là.

## Statut
- UEFI PI : lignes 1 à 25160 lues (partie III terminée, partie IV commencée). Prochaine : ligne 25161 (MM Foundation Entry Point, MMST).

## Lignes 25161-26460 — IV-2 (fin) : CurrentlyExecutingCpu, NumberOfCpus (compte les processeurs opérationnels, pas ceux entrés en MM), tableaux CpuSaveStateSize et CpuSaveState ; MM_FOUNDATION_ENTRY_POINT(HobStart) pour une Foundation autonome (HOB minimum : PHIT, CPU, description de la MMRAM, au moins un volume firmware). IV-3 MMST : signature 'SMST', EFI_MM_SYSTEM_TABLE { Hdr (CRC32 à zéro), MmFirmwareVendor, MmFirmwareRevision, MmInstallConfigurationTable, MmIo, MmAllocatePool, MmFreePool, MmAllocatePages, MmFreePages, MmStartupThisAp, CurrentlyExecutingCpu, NumberOfCpus, CpuSaveStateSize, CpuSaveState, NumberOfTableEntries, MmConfigurationTable, MmInstallProtocolInterface, MmUninstallProtocolInterface, MmHandleProtocol, MmRegisterProtocolNotify, MmLocateHandle, MmLocateProtocol, MmiManage, MmiHandlerRegister, MmiHandlerUnRegister } ; les champs CPU ne sont définis que pendant le traitement d'une MMI ; les allocations MMRAM utilisent EfiRuntimeServicesCode ou EfiRuntimeServicesData, alignées sur 8 octets ; services UEFI et services MM ne se mélangent pas (ni mémoire ni protocoles). MmInstallConfigurationTable : quatre cas (ajout, NOT_FOUND, mise à jour, retrait). MmStartupThisAp(Procedure, CpuNumber, ProcArguments) : un AP sain, activé, non occupé. MmRegisterProtocolNotify(Protocol, Function, Registration) : pas d'événement ; Function NULL = décrochage. MmiManage(HandlerType, Context, CommBuffer, CommBufferSize) : NULL = handlers racine avec EFI_MM_ENTRY_CONTEXT ; retours SUCCESS, WARN_INTERRUPT_SOURCE_PENDING, INTERRUPT_PENDING, NOT_FOUND. MmiHandlerRegister(Handler, HandlerType, DispatchHandle) ; le handler reçoit (DispatchHandle, Context, CommBuffer, CommBufferSize) ; CommBuffer = tampon contigu, adresse physique, venant du monde non-MM ; retours du handler : SUCCESS (plus aucun handler appelé), WARN_INTERRUPT_SOURCE_QUIESCED, WARN_INTERRUPT_SOURCE_PENDING, INTERRUPT_PENDING. IV-4 Protocoles MM (modèle « rien de partagé » : sauver et restaurer toute ressource partagée avec l'OS au premier plan) : EFI_MM_STATUS_CODE_PROTOCOL (GUID 6afd2b77-98c1-4acd-a6f9-8a9439de0fb1 ; copie atomique de Data dans un tampon local d'au moins 256 octets, réentrant) ; EFI_MM_CPU_PROTOCOL (GUID eb346b97-975f-4a9f-8b22-f8e92bb3d569 ; ReadSaveState / WriteSaveState(Width, Register, CpuIndex, Buffer)) ; registres x64 : GDTBASE 4 … LDTINFO 10, ES 20 … GS 25, LDTR_SEL 26, TR_SEL 27, DR7 28, DR6 29, R8-R15 30-37, RAX 38, RBX 39, RCX 40, RDX 41, RSP 42, RBP 43, RSI 44, RDI 45, RIP 46, RFLAGS 51, CR0 52, CR3 53, CR4 54, FCW 256 … XMM15 287 ; pseudo-registres IO 512, LMA 513 (32 ou 64, un octet, lecture seule), PROCESSOR_ID 514 (64 bits, lecture seule) ; registres ARM/AArch64 à partir de 1024 (hors périmètre). EFI_MM_SAVE_STATE_IO_INFO { IoData, IoPort, IoWidth (UINT8 0 … UINT64 3), IoType (INPUT 1, OUTPUT 2, STRING 4, REP_PREFIX 8) } ; NOT_FOUND s'il n'y avait pas d'E/S. EFI_MM_CPU_IO_PROTOCOL (GUID 3242a9d8-ce70-4aa0-955d-5e7b140de4d2 ; Mem et Io, Read/Write(Width, Address, Count, Buffer), exécutées telles quelles). EFI_MM_PCI_ROOT_BRIDGE_IO_PROTOCOL (GUID 8bc1714d-ffcb-41c3-89dc-6c74d06d98ea ; Map, Unmap, Flush, AllocateBuffer, FreeBuffer, SetAttributes, Configuration peuvent rendre UNSUPPORTED). EFI_MM_READY_TO_LOCK_PROTOCOL (GUID 47b7fa8c-f4bd-4af6-8200-333086f0d2c8 ; obligatoire, interface NULL, installé après End of DXE côté MM). EFI_MM_MP_PROTOCOL (GUID 5D5450D7-990C-4180-A803-8E63F0608307 ; Revision 0 ; Attributes bit 0 = délais gérés ; GetNumberOfProcessors, DispatchProcedure, BroadcastProcedure, SetStartupProcedure, CheckOnProcedure, WaitForProcedure ; jeton NULL = bloquant ; délai 0 = infini ; NOT_STARTED pour un processeur exclu de la diffusion). EFI_MM_CONFIGURATION_PROTOCOL (GUID 26eeb3de-b689-492e-80f0-be8bd7da4ba7 ; RegisterMmFoundationEntry).
- [AGENT] PERTINENT : (1) Le pseudo-registre IO décrit l'accès au port qui a déclenché la SMI : c'est ainsi que le firmware reconnaît une commande écrite sur le port SMI (port et valeur lus dans l'état sauvé) ; vu de l'OS, une écriture sur ce port est donc un appel au firmware, à ne pas faire sans accord. (2) Le handler SMM peut modifier l'état sauvé (WriteSaveState) : au retour, les registres de l'OS peuvent avoir changé (RAX comme valeur de retour) ; c'est normal pour un appel SMI logiciel. (3) SetStartupProcedure : le firmware peut exécuter du code sur un cœur qui sort d'un état profond avant de le rendre à l'OS ; l'OS ne doit pas supposer qu'un cœur réveillé est dans l'état où il l'a laissé (MSR à reprogrammer, déjà noté pour S3). (4) CommBuffer est une adresse physique fournie par le monde normal : côté OS, ce tampon doit être dans une zone déclarée au firmware (voir plus loin MM Communication).

## Statut
- UEFI PI : lignes 1 à 26460 lues. Prochaine : ligne 26461 (fin MM Configuration, End of DXE, protocoles MM Child Dispatch).

## Lignes 26461-27760 — Protocoles MM de jalon (interface NULL) : EFI_MM_END_OF_PEI_PROTOCOL (f33e1bf3-980b-4bfb-a29a-b29c86453732), EFI_MM_UEFI_READY_PROTOCOL (c63a953b-73b0-482f-8da6-766566f65a82), EFI_MM_READY_TO_BOOT_PROTOCOL (6e057ecf-fa99-4f39-95bc-59f9921d17e4), EFI_MM_EXIT_BOOT_SERVICES_PROTOCOL (296eb418-c4c8-4e05-ab59-39e8af56f00a), EFI_MM_END_OF_DXE_PROTOCOL (24e70042-d5c5-4260-8c39-0ad3aa32e93d ; obligatoire, installé avant MM Ready to Lock) ; EFI_MM_SECURITY_ARCH_PROTOCOL (b48e70a3-476f-486d-b9c0-c2d0f8b944d9 ; même usage que le protocole de sécurité DXE) ; EFI_MM_HANDLER_STATE_NOTIFICATION_PROTOCOL (30C8340F-4C30-41D9-BFAE-444ACB2C1F76 ; notifie chaque enregistrement ou retrait de handler : HandlerRegistered / HandlerUnregistered). IV-5 Protocoles UEFI : EFI_MM_BASE_PROTOCOL (f4ccbfb7-f6e0-47fd-9dd4-10a8f150c191 ; InMm(InMmram), GetMmstLocation(Mmst) ; UNSUPPORTED hors MM) ; EFI_MM_ACCESS_PROTOCOL (c2702b74-800c-4131-8746-8fb5b89ce4ac ; Open, Close, Lock (écriture unique), GetCapabilities(MmramMapSize, MmramMap), LockState, OpenState) ; EFI_MMRAM_DESCRIPTOR { PhysicalStart, CpuStart, PhysicalSize, RegionState } ; états OPEN 0x1, CLOSED 0x2, LOCKED 0x4, CACHEABLE 0x8, ALLOCATED 0x10, NEEDS_TESTING 0x20, NEEDS_ECC_INITIALIZATION 0x40 ; PhysicalStart sert tant que la MMRAM est ouverte, CpuStart une fois fermée ; T-SEG = mémoire en haut de la DRAM. EFI_MM_CONTROL_PROTOCOL (843dc720-ab1e-42cb-9357-8a0078f3561b ; pilote runtime ; Trigger(CommandPort, DataPort, Periodic, ActivationInterval), Clear(Periodic), MinimumTriggerPeriod en pas de 10 ns) ; CommandPort = SwMmiInputValue du Software Dispatch, DataPort = octet de données. EFI_MM_CONFIGURATION_PROTOCOL (26eeb3de-… ; MmramReservedRegions = tableau terminé par une taille 0 ; RegisterMmEntry). EFI_DXE_MM_READY_TO_LOCK_PROTOCOL (60ff8964-e906-41d0-afed-f241e974e08e ; obligatoire, publié juste après End of DXE : passage du monde « constructeur seul » au monde des modules tiers ; c'est là que la plateforme verrouille la MMRAM). EFI_MM_COMMUNICATION_PROTOCOL (c68ed8e2-9dc6-4cbd-9d94-db65acc5c332 ; Communicate(CommBuffer, CommSize)) : service runtime ; tampon contigu, adresse PHYSIQUE (les handlers MM tournent en mode physique) ; commence toujours par EFI_MM_COMMUNICATE_HEADER { HeaderGuid, MessageLength, Data[] } ; non réentrant ; retours SUCCESS, INVALID_PARAMETER, BAD_BUFFER_SIZE (taille maximale renvoyée), ACCESS_DENIED (tampon dans une plage inaccessible à MM). EFI_MM_INITIALIZATION_HEADER { Header, SystemTable } avec EFI_MM_INITIALIZATION_GUID 99be0d8f-3548-48aa-b577-fcfba56a67f7. EFI_MM_COMMUNICATION2_PROTOCOL (378daedc-f06b-4446-8314-40ab933c87a3 ; adresses physique ET virtuelle). EFI_MM_COMMUNICATION3_PROTOCOL (f7234a14-0df2-46c0-ad28-90e6b883a72f) avec EFI_MM_COMMUNICATE_HEADER_V3 { HeaderGuid = 68e8c853-2ba9-4dd7-9ac0-91e16155c935, BufferSize, Reserved, MessageGuid, MessageSize, MessageData[] }. IV-6 PPI PEI : EFI_PEI_MM_ACCESS_PPI (268f33a9-cccd-48be-8817-86053ac32ed6 ; Open/Close/Lock par DescriptorIndex, GetCapabilities), EFI_PEI_MM_CONTROL_PPI (61c68702-4d7e-4f43-8def-a74305ce74c5 ; Trigger(ArgumentBuffer, …), Clear), EFI_PEI_MM_CONFIGURATION_PPI (0c109319-c149-450e-a3e3-b9badd9dc3a4), EFI_PEI_MM_COMMUNICATION_PPI (ae933e1c-cc47-4e38-8f0e-e2f61d2605df ; appelant en mappage 1:1).
- [AGENT] TRÈS PERTINENT : (1) Le service Communicate reste utilisable par l'OS après ExitBootServices ; c'est par là que passent, sur beaucoup de firmwares, les écritures de variables UEFI. Si notre OS appelle SetVariable en runtime, il déclenche donc une SMI derrière le rideau : écriture NVRAM, soumise à accord explicite. (2) Le tampon doit être une adresse physique contiguë située dans une zone que le firmware accepte (runtime ou réservée), sinon ACCESS_DENIED. (3) EFI_MM_ACCESS.Lock est à écriture unique et fait avant le lancement de notre chargeur : LockState est vrai quand nous arrivons ; aucune tentative d'ouverture à faire. (4) La période minimale de déclenchement est en pas de 10 ns : unité à ne pas confondre avec les 100 ns des temporisateurs UEFI.

## Statut
- UEFI PI : lignes 1 à 27760 lues. Prochaine : ligne 27761 (fin MM Communication PPI, puis protocoles MM Child Dispatch : SW, Sx, Periodic Timer, USB, GPI, Standby Button, Power Button, IO Trap).

## Lignes 27761-29160 — EFI_PEI_MM_COMMUNICATION3_PPI (e70febf6-13ef-4a21-899e-b236f825ffc9 ; tampon commençant par l'en-tête V3). IV-7 Protocoles MM Child Dispatch (utilisables seulement depuis la MMRAM ; chaque instance porte un chemin de périphérique) : EFI_MM_SW_DISPATCH_PROTOCOL (18a3c6dc-5eea-48c8-a1c1-b53389f98999 ; Register(DispatchFunction, RegisterContext{SwMmiInputValue}, DispatchHandle), UnRegister, MaximumSwiValue ; valeur (UINTN)-1 = attribution automatique ; le handler reçoit EFI_MM_SW_CONTEXT { SwMmiCpuIndex, CommandPort, DataPort }) ; EFI_MM_SX_DISPATCH_PROTOCOL (456d2859-a84b-4e47-a2ee-3276d886997d ; contexte { Type : SxS0 … SxS5 ; Phase : SxEntry, SxExit }) ; EFI_MM_PERIODIC_TIMER_DISPATCH_PROTOCOL (4cec368e-8e8e-4d71-8be1-958c45fc8a53 ; contexte { Period, MmiTickInterval } en unités de 100 ns ; exemple : puce à 64 ms ou 2 s, période voulue 3 s → Period 4 s avec tick 2 s, ou Period 3 s avec tick 64 ms (rappel toutes les 47 MMI, au prix de plus de MMI) ; EFI_MM_PERIODIC_TIMER_CONTEXT { ElapsedTime } ; GetNextShorterInterval du plus long au plus court) ; EFI_MM_USB_DISPATCH_PROTOCOL (ee9b8d90-c5a6-40a2-bde2-52558d33cca1 ; contexte { Type : UsbLegacy (émulation, piège sur les registres du contrôleur clavier PS/2) ou UsbWake ; Device = chemin du contrôleur hôte }) ; EFI_MM_GPI_DISPATCH_PROTOCOL (25566b03-b577-4cbf-958c-ed663ea24380 ; GpiNum ; NumSupportedGpis) ; EFI_MM_STANDBY_BUTTON_DISPATCH_PROTOCOL (7300c4a1-43f2-4017-a51b-c81a7f40585b ; Phase Entry/Exit) ; EFI_MM_POWER_BUTTON_DISPATCH_PROTOCOL (1b1183fa-1823-46a7-8872-9c578755409d ; Phase Entry/Exit) ; EFI_MM_IO_TRAP_DISPATCH_PROTOCOL (58dc368d-7bfa-4e77-abbc-0e29418df930 ; contexte { Address, Length, Type : WriteTrap, ReadTrap, ReadWriteTrap } ; Address 0 = plage allouée ; EFI_MM_IO_TRAP_CONTEXT { WriteData } ; ressource matérielle limitée). HOB EFI_PEI_MM_CORE_GUID (8d1b3618-111b-4cba-b79a-55b32f60f029 ; Flags bit 0 = MM Core chargé). IV-8 Interactions : méthode 1 = SetVariable runtime réalisé par Communicate(), le code MM sérialise l'écriture flash demandée par l'OS et ses propres écritures (journal d'erreurs) ; méthode 2 = SMI logicielle décrite par la table ACPI de communication MM (annexe O de la spec UEFI), utilisable par l'OS ; en PEI, exemple LockBox pour la reprise S3. IV-9 Types de fichier : EFI_FV_FILETYPE_MM 0x0A, COMBINED_MM_DXE 0x0C, MM_STANDALONE 0x0E ; section EFI_SECTION_MM_DEPEX 0x1C ; chaque fichier : au moins une section PE32, au plus une VERSION, au plus une MM_DEPEX. IV-10 MCA / INIT / PMI : entièrement Itanium (PAL, SAL, SALE_ENTRY, min-state, MCA_INIT_PMI_PER_PROCESSOR_DATA à min-state + 4 K, handlers EFI_SAL_MCA_HANDLER, EFI_SAL_INIT_HANDLER, SAL_PMI_HANDLER, protocole EFI_SAL_MCA_INIT_PMI_PROTOCOL b60dc6e8-3b6f-11d5-af09-00a0c944a05b) : lu, hors périmètre (processeur AMD64).
- [AGENT] TRÈS PERTINENT (clavier USB, bouton d'alimentation, veille) : (1) UsbLegacy : tant que l'émulation héritée du firmware est active, des SMI interceptent les ports 0x60/0x64 et le firmware pilote lui-même le contrôleur USB ; notre pilote xHCI doit d'abord prendre la main par la procédure « BIOS/OS handoff » de la capacité USB Legacy Support (déjà notée dans la lecture xHCI) avant de toucher au contrôleur. (2) Bouton d'alimentation et bouton de veille passent d'abord par une SMI ; ce que l'OS reçoit ensuite vient de l'ACPI (événement à bouton fixe ou GPE), pas directement du bouton. (3) Entrée en S3/S4/S5 : l'écriture de SLP_TYP|SLP_EN dans PM1 déclenche une SMI Sx avant l'extinction ; un blocage à l'arrêt peut donc venir d'un handler firmware. (4) Piège d'E/S : un port lu ou écrit par l'OS peut être intercepté par le firmware ; ne pas sonder des ports au hasard (règle déjà notée, confirmée). (5) Les variables UEFI écrites en runtime passent par ce chemin et sont sérialisées par le code MM : aucune écriture parallèle directe dans la flash.

## Statut
- UEFI PI : lignes 1 à 29160 lues. Prochaine : ligne 29161 (fin MCA/INIT/PMI Itanium, Extended SAL, puis partie V Standards).

## Lignes 29161-30560 — Fin du protocole MCA/INIT/PMI Itanium (RegisterMcaHandler, RegisterInitHandler, RegisterPmiHandler avec MakeFirst / MakeLast). Chapitre IV-11 Extended SAL Services (Itanium uniquement) : convention d'appel SAL, appels en mode physique avant SetVirtualAddressMap puis physique ou virtuel après ; pilotes de type DXE_SAL_DRIVER ; EXTENDED_SAL_BOOT_SERVICE_PROTOCOL (de0ee9a4-3c7a-44f2-b78b-e3ccd69c3af7 : AddSalSystemTableInfo, AddSalSystemTableEntry, RegisterExtendedSalProc, ExtendedSalProc toujours disponible) ; procédures nommées par (GUID de classe, FunctionId) ; retours EFI_SAL_NOT_IMPLEMENTED, EFI_SAL_VIRTUAL_ADDRESS_ERROR. Classes : Base I/O (IoRead, IoWrite, MemRead, MemWrite = équivalents du CPU I/O PPI), Stall (équivalent de Stall), Real Time Clock (GetTime, SetTime, GetWakeupTime, SetWakeupTime), Reset (ResetSystem), puis Status Code, Monotonic Counter, Variable, Base, Cache, PAL, PCI, MCA Log, FVB, MP, MCA.
- [AGENT] Hors périmètre : tout ce bloc ne concerne que l'Itanium (PAL/SAL). Rien à retenir pour un processeur AMD64, sinon que SetTime, SetWakeupTime et ResetSystem restent des opérations à effet (écriture RTC, redémarrage) soumises à accord, comme leurs équivalents UEFI.

## Statut
- UEFI PI : lignes 1 à 30560 lues. Prochaine : ligne 30561 (suite Extended SAL : Reset, PCI, Cache, PAL, Status Code, Monotonic Counter, Variable, MP, MCA).

## Lignes 30561-33110 — Suite Extended SAL (Itanium uniquement). Classe PCI (a46b1a31-ad66-4905-92f6-2b4659dc3063 : SalPciConfigRead, SalPciConfigWrite = SAL_PCI_CONFIG_READ/WRITE avec adresse, taille, valeur, type d'adresse). Classe Cache (0edc9494-2743-4ba5-8818-0aef5213f188 : CacheInit, CacheFlush(i_or_d)). Classe PAL (e1cd9d21-0fc2-438d-9703-04e66d961e57 : PalProc = enveloppe C des appels PAL, statique ou empilé, copie mémoire si le PAL est recopié ; SetNewPalEntry / GetNewPalEntry physique ou virtuel ; UpdatePal). Classe Status Code (00dbd91d-55e9-420f-9639-5e9f8437b44f : ReportStatusCode). Classe Monotonic Counter (899afd18-75e8-408b-a41a-6e2e7ecdf454 : GetNextHighMonotonicCount). Classe Variable (4ecb6c53-c641-4370-8cb2-3b0e496e8378 : GetVariable, GetNextVariableName, SetVariable, QueryVariableInfo, mêmes prototypes que les services runtime). Classe Firmware Volume Block (a2271df1-bcbb-4f1d-98a9-06bc172f071a : Read, Write, EraseBlock (un seul bloc par appel), GetAttributes, SetAttributes, GetPhysicalAddress, GetBlockSize, EraseCustomBlockRange (effacement partiel des blocs de début et de fin)). Classe MCA Log (cb3fd86e-38a3-4c03-9a5c-90cfa3a2ab7a : GetStateInfo, GetStateInfoSize, ClearStateInfo, GetStateBuffer (adresse dans r9, taille dans r10), SaveStateBuffer vers le stockage non volatil). Classe Base (d9e9fa06-0fe0-41c3-96fb-83425a3394f8 : SetVectors, McRendez, McSetParams, GetVectors (0 = MCA, 1 = BSP INIT, 2 = BOOT_RENDEZ, 3 = AP INIT), McGetParams, McGetMcParams, GetMcCheckinFlags, GetPlatformBaseFreq, RegisterPhysicalAddr). Toutes ces procédures prennent (FunctionId, Arg2..Arg8, VirtualMode, ModuleGlobal) et rendent EFI_SAL_VIRTUAL_ADDRESS_ERROR si elles sont appelées en mode virtuel avant conversion des adresses.
- [AGENT] Hors périmètre (Itanium). Seule idée transposable : les services Write, EraseBlock, SetAttributes d'un volume firmware et SetVariable sont des écritures dans la flash du firmware ; sur notre machine leurs équivalents (protocole FVB, SetVariable) restent interdits sans accord explicite.

## Statut
- UEFI PI : lignes 1 à 33110 lues. Prochaine : ligne 33111 (fin Extended SAL : classes Base (fin), MP, MCA ; puis partie V à la ligne 34467).

## Lignes 33111-33760 — Fin de la classe Base Extended SAL (McGetParams : 1 = interruption de rendez-vous, 2 = réveil, 3 = vecteur d'erreur plateforme corrigée ; McGetMcParams : rz_always, time_out, binit_escalate ; GetMcCheckinFlags ; GetPlatformBaseFreq ; RegisterPhysicalAddr). Classe MP (697d81a2-cf18-4dc0-9e0d-06113b618a3f : AddCpuData (Global ID, drapeau activé, compatibilité PAL, Platform ID), RemoveCpuData, ModifyCpuData, GetCpuDataById, GetCpuDataByIndex (début), SendIpi, WhoAmI, NumProcessors, SetMinState, GetMinState, PhysicalIdInfo) ; retours EFI_SAL_SUCCESS, EFI_SAL_NOT_ENOUGH_SCRATCH, EFI_SAL_NO_INFORMATION.
- [AGENT] Hors périmètre (Itanium), rien à retenir pour le projet.

## Statut
- UEFI PI : lignes 1 à 33760 lues (sur 43 663). Prochaine : ligne 33761 (fin de la classe MP et classe MCA Extended SAL jusqu'à la ligne 34466 ; puis PARTIE V Standards à partir de 34467 : SMBus, SMBIOS, IDE, S3, ACPI SDT, PCI Host Bridge, PCI Platform, Hot Plug, Super I/O, CPU I/O, Legacy Region, I2C, SPI, codes d'erreur).

## Lignes 33761-34466 — Fin Extended SAL (Itanium) : classe MP (GetCpuDataByIndex, WhoAmI, NumProcessors : maximum dans r9, total dans r10, activés dans r11 ; SetMinState, GetMinState, PhysicalIdInfo) ; classe MCA (2a591128-6cc7-42b1-8af0-58933b682dbb : McaGetStateInfo, McaRegisterCpu). IV-12 Pile SPI en SMM : même conception qu'en DXE (voir partie V) mais un SEUL contrôleur SPI hôte en SMM, car SMM n'a pas de chemins de périphérique ; sert surtout à la flash SPI ; GUID propres à SMM : LEGACY_SPI_SMM_FLASH 5e3848d4-0db5-4fc0-9729-3f353d4f879f, SPI_SMM_NOR_FLASH aab18f19-fe14-4666-8604-87ff6d662c9a, SPI_SMM_CONFIGURATION 995c6eca-171b-45fd-a3aa-fd4c9c9def59, SPI_SMM_HC e9f02217-2093-4470-8a54-5c2cffe73ecb, LEGACY_SPI_SMM_CONTROLLER 62331b78-d8d0-4c8c-8ccb-d27dfe32db9b, SPI_SMM_NOR_FLASH_BLOCK_MAP ADB71277-4754-4FFE-9836-0623BFA9DF77. IV-13 Annexe de compatibilité : jusqu'à PI 1.4 tout s'appelait SMM ; table d'alias EFI_SMM_* → EFI_MM_* (EFI_SMM_BASE2_PROTOCOL, EFI_SMM_ACCESS2_PROTOCOL, EFI_SMM_CONTROL2_PROTOCOL, EFI_SMM_SW_DISPATCH2_PROTOCOL, EFI_SMM_SYSTEM_TABLE2, SwSmiInputValue, SmiTickInterval, etc.), structures identiques. FIN DE LA PARTIE IV.
- [AGENT] PERTINENT : (1) Dans un firmware réel (dump BIOS de la machine), les modules portent encore les anciens noms SMM et les mêmes GUID : pour reconnaître un module dans le dump, chercher les GUID notés plus haut, pas les noms. (2) La flash SPI du firmware est pilotée depuis SMM par un contrôleur unique : l'OS ne doit pas piloter ce contrôleur SPI en parallèle (collision avec le firmware) ; lecture du BIOS seulement par la fenêtre mémoire déjà utilisée, aucune écriture.

## Statut
- UEFI PI : lignes 1 à 34466 lues (parties I à IV terminées). Prochaine : ligne 34467 (PARTIE V Standards : SMBus).

## Lignes 34467-35316 — PARTIE V Standards. V-1 Introduction et glossaire : bus PCI = jusqu'à 32 périphériques ; périphérique = jusqu'à 8 fonctions ; segment PCI = jusqu'à 256 bus partageant le même espace de configuration (_SEG, 0 si unique) ; espace de configuration par les ports 0xCF8/0xCFC, PCI Express par une fenêtre mémoire ; host bridge (abstraction logicielle) / root bridge (bus racine, objet ACPI PNP0A03) ; énumération PCI = numéros de bus, interruptions, ressources E/S, mémoire, mémoire préchargeable, faite une seule fois par démarrage ; « périphérique PCI incompatible » = exigences spéciales de BAR ; HPC / PHPC / SHPC (hot plug), root HPC (réinitialisé seulement au reset système), bourrage de ressources (resource padding) pour l'ajout à chaud ; PEC = code d'erreur de paquet SMBus ; UDID = identifiant 128 bits pour l'ARP SMBus. Conventions : petit-boutiste, champs réservés à zéro en écriture, ignorés en lecture, conservés en mise à jour. V-2/V-3 SMBus : EFI_SMBUS_HC_PROTOCOL (e49d33ed-513d-4634-b698-6f55aa751c1b) : Execute(SlaveAddress, Command, Operation, PecCheck, Length, Buffer), ArpDevice, GetArpMap, Notify ; un pilote peut ne fournir qu'Execute (les trois autres rendent UNSUPPORTED) ; SMBus = lent et non énumérable, pas de périphériques de démarrage ; retours CRC_ERROR (PEC faux), TIMEOUT, DEVICE_ERROR (collision, commande illégale, cycle non réclamé), BUFFER_TOO_SMALL. V-4/V-5 PPI : EFI_PEI_SMBUS2_PPI (9ca93627-b65b-4324-a202-c0b461764543, avec Identifier) ; EFI_SMBUS_DEVICE_ADDRESS = 7 bits ; opérations : QuickRead, QuickWrite, ReceiveByte, SendByte, ReadByte, WriteByte, ReadWord, WriteWord, ReadBlock, WriteBlock, ProcessCall, BWBRProcessCall ; EFI_SMBUS_UDID { VendorSpecificId, SubsystemDeviceId, SubsystemVendorId, Interface, DeviceId, VendorId, VendorRevision, DeviceCapabilities } ; EFI_SMBUS_DEVICE_MAP { adresse, UDID }. V-6 SMBIOS : EFI_SMBIOS_PROTOCOL (03583ff6-cb36-4940-947e-b9b39f4afaf7) : Add, UpdateString, Remove, GetNext, MajorVersion, MinorVersion ; TPL < TPL_NOTIFY ; handle 0xFFFE = attribution automatique / début et fin de parcours ; en-tête { Type, Length, Handle } ; zone formatée suivie de chaînes terminées par un double zéro ; types 0 (BIOS) à 42, 126 inactif, 127 fin de table, 128-255 OEM ; types utiles : 1 système, 2 carte mère, 4 processeur, 7 cache, 16/17 mémoire, 22 batterie. V-7 Contrôleur IDE (début) : EFI_IDE_CONTROLLER_INIT_PROTOCOL abstrait les registres de timing propres au fabricant ; PATA = 2 canaux × 2 périphériques ; SATA en émulation ou en AHCI (jusqu'à 32 ports, multiplicateur de port = jusqu'à 16 périphériques par port) ; adresse (Channel, Device) ; mode natif ou compatibilité = bit 0 du registre 0x09 (Programming Interface), base du bloc de commandes = BAR0 avec le bit 0 effacé.
- [AGENT] PERTINENT : (1) SMBIOS : la table est accessible à l'OS par la table de configuration UEFI (GUID SMBIOS / SMBIOS3) ; lecture seule, c'est la source propre du modèle, du numéro de série, des barrettes mémoire et de la batterie. L'agent n'utilise que GetNext ; Add, UpdateString, Remove modifient la table du firmware : interdits. (2) SMBus : c'est par là que se lisent les SPD des barrettes et certains capteurs ; un WriteByte au mauvais esclave peut corrompre une EEPROM SPD : aucune écriture SMBus sans accord explicite, et pas de balayage d'adresses en écriture (QuickWrite). (3) La machine n'a pas de disque IDE/SATA utilisé (NVMe) : chapitre IDE à lire mais sans conséquence directe. (4) Règle « champs réservés » identique à celle des autres spécifications : conserver à la mise à jour.

## Statut
- UEFI PI : lignes 1 à 35316 lues. Prochaine : ligne 35317 (suite contrôleur IDE, puis S3 à la ligne 35871).

## Lignes 35317-36156 — V-7 IDE (fin) : séquence d'énumération d'un canal : NotifyPhase(BeforeChannelEnumeration), GetChannelInfo(Enabled, MaxDevices), BeforeChannelReset, reset, AfterChannelReset, BeforeDevicePresenceDetection, détection, AfterDevicePresenceDetection, ResetMode ; SubmitData (IDENTIFY, NULL si absent, une seule fois par (Channel, Device)), DisqualifyMode, CalculateMode (tampon alloué par l'appelé), SET_FEATURES côté périphérique (en cas d'erreur le mode est disqualifié et on recalcule), SetTiming (une fois), AfterChannelEnumeration, puis BLOCK_IO. EFI_IDE_CONTROLLER_INIT_PROTOCOL (a1e37052-80d9-4e65-a317-3e9a55c43ec9 : GetChannelInfo, NotifyPhase, SubmitData, DisqualifyMode, CalculateMode, SetTiming, EnumAll, ChannelCount) ; EFI_ATA_IDENTIFY_DATA et EFI_ATAPI_IDENTIFY_DATA (ATA/ATAPI-8, 256 mots ; n° de série mots 10-19, firmware 23-26, modèle 27-46, LBA48 mots 100-103 ; bit 15 de config = ATAPI) ; EFI_ATA_COLLECTIVE_MODE { PioMode, SingleWordDmaMode, MultiWordDmaMode, UdmaMode, ExtModeCount, ExtMode[] } ; SATA : AUTO 0, GEN1 1, GEN2 2. EFI_DISK_INFO_PROTOCOL (d432a67f-14dc-484b-b3bb-3f0291849327 : Interface, Inquiry, Identify, SenseData, WhichIde) ; GUID d'interface : IDE 5e948fe3-26d3-42b5-af17-610287188dec, SCSI 08f74baa-ea36-41d9-9521-21a70f8780bc, USB cb871572-c11a-47b5-b492-675eafa77727, AHCI 9e498932-4abc-45af-a34d-0247787be7c6, NVMe 3ab14680-5d3f-4a4d-bcdc-cc380018c7f7, UFS 4b3029cc-6b98-47fb-bc96-76dcb80441f0, SD/MMC 8deec992-d39c-4a5c-ab6b-986e14242b9d. V-8 Reprise S3 : chemin de démarrage spécial ; SEC inchangé, PEI rejoué (PEIM « conscients du chemin »), DXE NON rejoué : remplacé par un script de démarrage enregistré par les pilotes DXE dans la mémoire NVS et rejoué par un moteur en PEI ; le firmware ne restaure que le chipset et le processeur (SMM, microcode, init propre au processeur, caches) ; le DXE IPL passe la main au S3 Resume PPI, qui rejoue le script puis saute au vecteur de réveil de l'OS (passage en mode réel si l'OS n'a qu'un vecteur mode réel) ; s'il revient, la reprise a échoué. EFI_PEI_S3_RESUME2_PPI (6d582dbc-db85-4514-8fcc-5adf6227b147 ; retours ABORTED, NOT_FOUND). EFI_S3_SAVE_STATE_PROTOCOL (e857caf6-c046-45dc-be3f-ee0765fba887 : Write, Insert, Label, Compare) ; opcodes : IO_WRITE 0x00, IO_READ_WRITE 0x01, MEM_WRITE 0x02, MEM_READ_WRITE 0x03, PCI_CONFIG_WRITE 0x04, PCI_CONFIG_READ_WRITE 0x05, SMBUS_EXECUTE 0x06, STALL 0x07, DISPATCH 0x08, DISPATCH_2 0x09, INFORMATION 0x0A, PCI_CONFIG2_WRITE 0x0B, PCI_CONFIG2_READ_WRITE 0x0C, IO_POLL 0x0D, MEM_POLL 0x0E, PCI_CONFIG_POLL 0x0F, PCI_CONFIG2_POLL 0x10 ; 0x80-0xFE réservés à l'implémentation ; largeurs Uint8..64, Fifo (adresse fixe), Fill (valeur fixe).
- [AGENT] TRÈS PERTINENT (veille S3 de l'OS) : (1) Au réveil S3 le firmware ne restaure QUE chipset et processeur ; tout le reste (contrôleur NVMe, xHCI, HDA, carte Wi-Fi, GPU, APIC locaux, MSR de l'OS, tables de pages) est à remettre en état par l'OS lui-même. (2) L'OS doit donner un vecteur de réveil dans la FACS avant d'entrer en S3 (déjà noté dans la lecture ACPI) ; si seul le vecteur 16 bits est fourni, le firmware saute en mode réel. (3) Le protocole DISK_INFO avec le GUID NVMe donne les données Identify du contrôleur pendant les services de démarrage : lecture seule, pratique pour confirmer le disque avant ExitBootServices. (4) Le script S3 est figé avant le démarrage de l'OS : l'OS ne peut rien y ajouter.

## Statut
- UEFI PI : lignes 1 à 36156 lues. Prochaine : ligne 36157 (opcodes du script S3 un par un, Insert, Label, Compare, S3 SMM Save State).

## Lignes 36157-36956 — V-8.7 Opcodes du script S3, un par un. IO_READ_WRITE / MEM_READ_WRITE / PCI_CONFIG_READ_WRITE (Width, Address, Data, DataMask) : au rejeu, lire, ET avec DataMask, OU avec Data, réécrire. IO_POLL / MEM_POLL / PCI_CONFIG_POLL (…, Delay en unités de 100 ns) : relire jusqu'à ce que (valeur & DataMask) == Data ou que le délai expire ; au moins un accès est toujours fait. MEM_WRITE / PCI_CONFIG_WRITE (Width, Address, Count, Buffer) avec les trois familles de largeur (normale : adresse et tampon avancent ; Fifo : adresse fixe ; Fill : valeur fixe). PCI_CONFIG : adresse au format du Root Bridge I/O ; plusieurs registres d'une même fonction en un seul enregistrement ; dépasser l'espace de configuration de la fonction = comportement indéfini ; pour PCI Express on peut partir d'un décalage 8 bits et continuer dans l'espace étendu. Variantes PCI_CONFIG2_* avec un paramètre Segment. SMBUS_EXECUTE (SlaveAddress, Command, Operation, PecCheck, Length, Buffer). STALL (durée en microsecondes). DISPATCH (EntryPoint) et DISPATCH_2 (EntryPoint, Context) : saut vers du code arbitraire puis retour au script ; le code et le contexte doivent être en mémoire EfiRuntimeServicesCode, EfiRuntimeServicesData ou EfiACPIMemoryNVS, convention d'appel DXE. INFORMATION : sans effet, pour le débogage. Insert(BeforeOrAfter, Position, OpCode, …) ; Label(BeforeOrAfter, CreateIfNotFound, Position, Label) ; Compare(Position1, Position2, RelativePosition : -1, 0, 1). V-8.8 EFI_S3_SMM_SAVE_STATE_PROTOCOL (320afe62-e593-49cb-a9f1-d4c2f4af014c) : mêmes fonctions, côté SMM, appelable en runtime (OUT_OF_RESOURCES possible).
- [AGENT] TRÈS PERTINENT (sécurité de la veille) : (1) Le script S3 contient des sauts vers du code (DISPATCH) logé en mémoire ACPI NVS ou runtime, exécuté au réveil avec tous les droits, avant l'OS. Ces zones sont dans la carte mémoire que reçoit l'OS : notre noyau ne doit JAMAIS y écrire ni les allouer (types EfiACPIMemoryNVS, EfiRuntimeServicesCode/Data, réservé), sinon la reprise S3 exécute des données corrompues. (2) Coquilles du texte source : dans les sections IO_READ_WRITE et IO_POLL les noms d'opcode cités dans « OpCode » sont intervertis ; se fier au titre de section. (3) Unités à ne pas confondre : POLL en 100 ns, STALL en microsecondes.

## Statut
- UEFI PI : lignes 1 à 36956 lues. Prochaine : ligne 36957 (V-9 ACPI System Description Table Protocol).

## Lignes 36957-37756 — V-9 EFI_ACPI_SDT_PROTOCOL (eb97088e-cfdf-49c6-be4b-d906a5b20e86) : AcpiVersion (bits NONE, 1.0B, 2.0, 3.0, 4.0, 5.0), GetAcpiTable(Index, Table, Version, TableKey) (RSDP, RSDT, XSDT ne sont pas dans la liste), RegisterNotify, Open, OpenSdt, Close (recalcule la somme de contrôle si modifié), GetChild, GetOption(Handle, Index, DataType, Data, DataSize), SetOption (BAD_BUFFER_SIZE si la donnée ne tient pas), FindPath ; en-tête EFI_ACPI_SDT_HEADER { Signature, Length, Revision, Checksum, OemId[6], OemTableId[8], OemRevision, CreatorId, CreatorRevision } ; types de données NONE 0, OPCODE 1, NAME_STRING 2, OP 3, UINT 4, STRING 5, CHILD 6 ; table 9.1 des opcodes AML et de leurs options (0x00 Zero, 0x01 One, 0x06 Alias, 0x08 Name, 0x0A Byte, 0x0B Word, 0x0C DWord, 0x0D String, 0x0E QWord, 0x10 Scope, 0x11 Buffer, 0x12 Package, 0x14 Method, 0x60-0x67 Local0-7, 0x68-0x6E Arg0-6, 0x70 Store … 0xA0 If, 0xA1 Else, 0xA2 While, 0xA4 Return, 0xFF Ones, préfixe 0x5B : 0x5B01 Mutex, 0x5B80 OperationRegion, 0x5B81 Field, 0x5B82 Device, 0x5B83 Processor, 0x5B84 PowerResource, 0x5B85 ThermalZone, 0x5B86 IndexField, 0x5B87 BankField). V-10 PCI Host Bridge : EFI_PCI_HOST_BRIDGE_RESOURCE_ALLOCATION_PROTOCOL, propre au chipset, obligatoire avec EFI_PCI_ROOT_BRIDGE_IO_PROTOCOL dès qu'il y a du PCI ; exemples : bureau à 1 root bridge (tout le segment 0, tout l'espace E/S, toute la mémoire au-dessus de la RAM), serveur à 4 root bridges (mêmes espaces partagés), 2 segments (512 bus), 2 host bridges (espaces séparés) ; alias ISA : si réservés, le host bridge alloue 4 fois l'E/S demandée et positionne _RNG. Répartition des registres (type 0) : bits I/O, Memory, Bus Master du registre de commande = pilote de bus PCI à la demande du pilote de périphérique ; SERR, PERR, MWI = code chipset ; taille de ligne de cache = code chipset ; latency timer = pilote de bus (0x20, PCI-X 0x40) ; BAR = pilote de bus ; interrupt line = non touché ; type 1 (ponts) : fenêtres bus, E/S, mémoire = pilote de bus ; Secondary Bus Reset permis pendant l'énumération. Algorithme d'énumération : NotifyPhase(BeginEnumeration), BeginBusAllocation, pour chaque root bridge StartBusEnumeration, GetAllocAttributes, balayage (ponts : numéros de bus ; périphériques : collecte des besoins ; réserve pour la plus grande ROM d'extension), SetBusNumbers, EndBusAllocation, BeginResourceAllocation, SubmitResources, AllocateResources (en cas d'échec : GetProposedResources, FreeResources, resoumettre), SetResources (décodage programmé), puis Configuration() et programmation des BAR, création des handles PCI_IO, EndResourceAllocation ; l'ordre des appels ne peut pas changer.
- [AGENT] TRÈS PERTINENT (PCI de l'OS, en l'absence de spécification PCI dans le dossier) : (1) Quand notre OS démarre, le firmware a DÉJÀ numéroté les bus et programmé les BAR et les fenêtres des ponts : l'OS lit ces valeurs et ne les réattribue pas (règle du projet confirmée par la répartition des rôles). (2) Ce que l'OS fait lui-même par périphérique : positionner Memory Space et Bus Master dans le registre de commande au moment de piloter le périphérique (NVMe, xHCI, HDA, Wi-Fi), rien d'autre. (3) Interrupt Line n'est pas fiable (« non touché ») : utiliser MSI/MSI-X ou le routage ACPI (_PRT). (4) Secondary Bus Reset réinitialise tout ce qui est derrière un pont : interdit sans accord explicite. (5) Le protocole ACPI SDT permet de MODIFIER l'AML (SetOption) : l'agent n'utilise que la lecture ; les tables ACPI se lisent de toute façon par le RSDP de la table de configuration.

## Statut
- UEFI PI : lignes 1 à 37756 lues. Prochaine : ligne 37757 (fin de l'exemple, définitions du protocole PCI Host Bridge Resource Allocation).

## Lignes 37757-38556 — V-10.7 (suite) : même processus redit avec les notifications aux pilotes plateforme (EFI_PCI_PLATFORM_PROTOCOL puis EFI_PCI_OVERRIDE_PROTOCOL avant et après le host bridge : BeforePciHostBridge / AfterPciHostBridge), fin par EndEnumeration puis installation du protocole EFI_PCI_ENUMERATION_COMPLETE_GUID (30cfe3e7-3de1-4586-be20-deaba1b3b793, interface NULL) ; mise en place d'un périphérique : BeforeResourceCollection, collecte des besoins, EFI_INCOMPATIBLE_PCI_DEVICE_SUPPORT_PROTOCOL (jeu de ressources de remplacement), GetPciRom plateforme puis override, sinon ROM d'option du périphérique, extraction de l'image UEFI. V-10.8 EFI_PCI_HOST_BRIDGE_RESOURCE_ALLOCATION_PROTOCOL (CF8034BE-6768-4d8b-B739-7CCE683A9FBE) : NotifyPhase, GetNextRootBridge, GetAllocAttributes, StartBusEnumeration, SetBusNumbers, SubmitResources, GetProposedResources, PreprocessController ; pas de chemin de périphérique sur le handle du host bridge (abstraction sans équivalent ACPI). Ressources décrites en descripteurs ACPI : QWORD Address Space Descriptor (octet 0 = 0x8A, longueur 0x2B ; type 0 mémoire, 1 E/S, 2 bus ; drapeaux généraux _MAF bit 3, _MIF bit 2 ; granularité 32 ou 64 ; minimum = base allouée ; maximum = alignement de la forme 2^n − 1 (interprétation propre à PI) ; décalage de traduction = état d'allocation ; longueur) puis End Tag (0x79, somme 0 = ignorée) ; E/S : bit 0 _RNG = limiter aux plages non ISA (n100-n3FF, n500-n7FF, n900-nBFF, nD00-nFFF) ; mémoire : bits 2:1 _MEM, 0 = non préchargeable, 3 = préchargeable. Phases : BeginEnumeration (remet à zéro les fenêtres ; pas de redémarrage après une autre notification), BeginBusAllocation, EndBusAllocation, BeginResourceAllocation, AllocateResources (peut rendre OUT_OF_RESOURCES ; fenêtres parfois plus grandes par bourrage, parfois plus petites), SetResources (matériel programmé, plus de réattribution possible), FreeResources, EndResourceAllocation, EndEnumeration ; retours NOT_READY, DEVICE_ERROR. Attributs : COMBINE_MEM_PMEM = 1 (pas de fenêtre préchargeable séparée), MEM64_DECODE = 2 (fenêtres 64 bits). StartBusEnumeration rend la plage de bus à balayer (minimum, longueur) ; SetBusNumbers la programme (paramètres valides mais faux = panne possible). EFI_RESOURCE_ALLOCATION_STATUS : SATISFIED 0, NOT_SATISFIED −1, autre = quantité manquante. PreprocessController(RootBridgeHandle, PciAddress, Phase) : BeforeChildBusEnumeration (ponts seulement, numéros de bus déjà valides, peut réinitialiser le bus secondaire et régler vitesse/mode) et BeforeResourceCollection (avant la mesure des BAR, pour chaque fonction) ; adresse PCI { Register, Function, Device, Bus, ExtendedRegister }. V-11 PCI Platform : introduction (PLATFORM, OVERRIDE, INCOMPATIBLE_PCI_DEVICE_SUPPORT).
- [AGENT] PERTINENT : (1) Format de l'adresse PCI firmware (registre, fonction, périphérique, bus, registre étendu) : identique à celui du Root Bridge I/O ; utile si l'agent lit la configuration PCI par ce protocole avant ExitBootServices. (2) La fenêtre mémoire d'un root bridge peut être en 64 bits : les BAR du NVMe ou de la carte Wi-Fi peuvent se trouver au-dessus de 4 Go ; le noyau doit donc mapper des adresses MMIO 64 bits. (3) Après SetResources le firmware lui-même ne réattribue plus : raison de plus pour ne pas déplacer un BAR depuis l'OS. (4) Descripteur QWORD 0x8A + End Tag 0x79 : même encodage que les _CRS ACPI déjà notés, mais deux champs ont un sens différent ici (alignement, état).

## Statut
- UEFI PI : lignes 1 à 38556 lues. Prochaine : ligne 38557 (V-11 PCI Platform Protocol, Override, Incompatible PCI Device, puis Hot Plug à 39015).

## Lignes 38557-39016 — V-11 PCI Platform. EFI_PCI_PLATFORM_PROTOCOL (07d75280-27d4-4d69-90d0-5643e238b341 ; optionnel, une seule instance) : PlatformNotify(HostBridge, Phase, ExecPhase) appelé deux fois par phase (BeforePciHostBridge = ChipsetEntry = 0, AfterPciHostBridge = ChipsetExit = 1), PlatformPrepController (par contrôleur, deux fois ; sert à programmer les identifiants de sous-système, les contournements matériels, les registres cachés qui fixent la taille des BAR, la vitesse des bus), GetPlatformPolicy (masque : RESERVE_NONE_IO_ALIAS 0x0000, RESERVE_ISA_IO_ALIAS 0x0001, RESERVE_ISA_IO_NO_ALIAS 0x0002, RESERVE_VGA_IO_ALIAS 0x0004, RESERVE_VGA_IO_NO_ALIAS 0x0008 ; par défaut le pilote de bus réserve les alias ISA et VGA), GetPciRom(PciHandle, RomImage, RomSize) : ROM d'option rangée par la plateforme (contrôleurs embarqués) ou remplaçant celle d'une carte ; image au format PCI 2.2, pouvant contenir du code hérité et du code UEFI. EFI_PCI_OVERRIDE_PROTOCOL (b5b35764-460c-4a06-99fc-77a17c1b5ceb) : même structure, appelé si le protocole plateforme est absent ou en erreur. EFI_INCOMPATIBLE_PCI_DEVICE_SUPPORT_PROTOCOL (eb23f55a-7863-4ac2-8d3d-956535de0375) : CheckDevice(VendorId, DeviceId, RevisionId, SubsystemVendorId, SubsystemDeviceId, Configuration) ; (UINTN)−1 = champ ignoré ; rend toujours SUCCESS, Configuration NULL si le périphérique est normal ; corrige longueur, alignement ou adresse fixe quand le BAR ment ; descripteur QWORD : maximum / décalage = index de BAR 0 à 6 (6 = BAR de la ROM d'option, −1 = tous sauf la ROM), granularité 32 = sous 4 Go, 64 = au-dessus, longueur 0 = garder celle du BAR.
- [AGENT] Retenu : (1) Certains périphériques ont un BAR qui ne dit pas la vérité (longueur ou alignement) et le firmware le corrige par une table interne : raison supplémentaire de reprendre les BAR tels que le firmware les a laissés, sans les re-mesurer en écrivant 0xFFFFFFFF (ce qui coupe brièvement le décodage). Si une mesure de taille est indispensable, la faire Memory Space désactivé et restaurer la valeur. (2) La ROM d'option d'un contrôleur embarqué (carte Wi-Fi, GPU intégré) peut ne pas être dans le périphérique mais dans la flash de la plateforme : ne pas conclure qu'il n'y en a pas parce que le BAR ROM est vide.

## Statut
- UEFI PI : lignes 1 à 39016 lues. Prochaine : ligne 39017 (V-12 Hot Plug PCI).

## Lignes 39017-39436 — V-12 Hot Plug PCI. Contrôleurs de branchement à chaud (HPC) : contrôleur PCI Hot Plug, pont PCI-CardBus, port racine ou port aval PCI Express. HPC racine = réinitialisé seulement au reset système, à initialiser par le firmware ; chaque HPC peut demander un bourrage de ressources (bus, E/S, mémoire) pris en compte à l'énumération ; allumer un bus SHPC peut prendre jusqu'à 15 s, d'où l'initialisation en parallèle ; plusieurs passes d'énumération. EFI_PCI_HOT_PLUG_INIT_PROTOCOL (aa0e8bc1-dabc-46b0-a844-37b8169b2bea ; une seule instance) : GetRootHpcList(HpcCount, HpcList de { HpcDevicePath, HpbDevicePath }), InitializeRootHpc(HpcDevicePath, HpcPciAddress, Event facultatif, HpcState), GetResourcePadding(…, Padding en descripteurs ACPI, Attributes) ; EFI_HPC_STATE : 0 non initialisé, INITIALIZED 0x01 (initialisé mais désactivé, pas de bourrage), INITIALIZED|ENABLED 0x03 (bourrage requis) ; attributs de bourrage : EfiPaddingPciBus (au niveau du bus, visible dans les fenêtres du pont) ou EfiPaddingPciRootBridge (au niveau du root bridge, impose de reprogrammer les ponts lors d'un ajout à chaud) ; un emplacement peut rester désactivé (défaut d'alimentation, type de bus, budget de puissance). EFI_PCI_HOTPLUG_REQUEST_PROTOCOL (19cb87ab-2cb9-4665-8360-ddcf6054f79d) : Notify(Operation Add / Remove, Controller, RemainingDevicePath, NumberOfChildren, ChildHandleBuffer). Ce protocole ne gère pas le branchement à chaud pendant le pré-démarrage lui-même.
- [AGENT] Retenu : (1) Sur un portable, le M.2 du NVMe et celui de la carte Wi-Fi sont derrière des ports racine PCI Express ; les fenêtres de ces ponts peuvent être plus larges que nécessaire (bourrage) : c'est normal, ne pas les « corriger ». (2) Le branchement à chaud en fonctionnement relève ensuite de l'OS (capacité PCI Express Slot, ACPI) : hors de la première version de l'OS ; un périphérique absent au démarrage n'est simplement pas géré.

## Statut
- UEFI PI : lignes 1 à 39436 lues. Prochaine : ligne 39437 (V-13 Super I/O Protocol, puis V-14 CPU I/O / I2C).

## Lignes 39437-39856 — V-13 EFI_SIO_PROTOCOL (215fdd18-bd50-4feb-890b-58ca0b4739e9), un par périphérique logique du Super I/O : RegisterAccess(Write, ExitCfgMode, Register, Value) (registres internes par les ports indexés 0x2E/0x2F, pas les E/S des périphériques logiques), GetResources (descripteurs ACPI petits ou grands, terminés par End tag), SetResources (ACCESS_DENIED si ressources occupées), PossibleResources, Modify (table { Register, AndMask, OrMask }) ; en-têtes ACPI_SMALL_RESOURCE_HEADER (Length 3 bits, Name 4 bits, Type 1 bit) et ACPI_LARGE_RESOURCE_HEADER (Name 7 bits, Type, Length 16 bits). V-14 Interactions Super I/O / contrôleur hôte ISA : Super I/O = composant sur bus ISA/LPC offrant clavier, souris, série, parallèle, disquette ; pilote Super I/O (DXE et PEIM), pilote de bus ISA (Service Binding), pilote de contrôleur hôte ISA (pont PCI-ISA ou PCI-LPC, nœud ACPI PNP0A05 décodage soustractif ou PNP0A06 décodage positif, ouvre et ferme les fenêtres E/S), pilotes logiques (PNP0303 clavier, PNP0501 série…) ; configuration tirée de la PCD. EFI_SIO_PPI (23a464ad-cb83-48b8-94ab-1a6fefcfe522 : Read, Write, Modify, SioGuid, Info) ; EFI_SIO_INFO { Hid, Uid, Ldn } ; HID : KBC PNP0303, LPT PNP0400, COM PNP0500, FDC PNP0700, MIDI PNPB006, GAME PNPB02F, END PNP0000.
- [AGENT] Retenu : sur le portable il n'y a pas de Super I/O classique, mais un contrôleur embarqué (EC) derrière le pont LPC ; le clavier interne est un périphérique PNP0303 (ports 0x60/0x64) déclaré dans l'ACPI. Les ports de configuration 0x2E/0x2F et 0x4E/0x4F ne doivent pas être sondés en écriture (séquence d'entrée en mode configuration = écriture à effet) sans accord.

## Statut
- UEFI PI : lignes 1 à 39856 lues. Prochaine : ligne 39857 (suite EFI_SIO_PPI, ISA HC, puis CPU I/O à 40281, Legacy Region, I2C, SPI).

## Lignes 39857-40286 — EFI_SIO_PPI (fin) : Read / Write(ExitCfgMode, Register, IoData), Register = (numéro de périphérique logique << 8) | registre, EFI_SIO_LDN_GLOBAL = 0xFF ; retours TIMEOUT, DEVICE_ERROR ; Modify par table. EFI_ISA_HC_PPI (8d48bd70-c8a3-4c06-901b-747946aac358 : Version, Address du pont ISA/LPC [fonction bits 0-2, périphérique 3-7, bus 8-15, segment 16-23, type de bus 24-31], OpenIoAperture(IoAddress, IoLength, IoApertureHandle), CloseIoAperture ; fenêtres comptées par référence). EFI_ISA_HC_PROTOCOL (bcdaf080-1bde-4e22-ae6a-43541e128ec4), même chose en DXE. EFI_ISA_HC_SERVICE_BINDING_PROTOCOL (fad7933a-6c21-4234-a434-0a8a0d2b0781). EFI_SIO_CONTROL_PROTOCOL (b91978df-9fc1-427d-bb05-4c828455ca27 : EnableDevice, DisableDevice ; ALREADY_STARTED si déjà dans l'état demandé). Début du chapitre V-15 CPU I/O Protocol (abstraction des E/S et de la mémoire du processeur pour le pilote de root bridge ; services de démarrage seulement).
- [AGENT] Retenu : sur un pont LPC à décodage positif, un port d'E/S n'atteint un périphérique LPC que si la fenêtre correspondante a été ouverte dans le pont par le firmware. Un port qui « ne répond pas » (0xFF) peut simplement ne pas être décodé : ce n'est pas une preuve d'absence, et l'OS ne modifie pas les fenêtres du pont LPC (registres chipset, voir PPR).

## Statut
- UEFI PI : lignes 1 à 40286 lues. Prochaine : ligne 40287 (V-15 CPU I/O, V-16 Legacy Region, V-17 I2C).

## Lignes 40287-41036 — V-15 EFI_CPU_IO2_PROTOCOL (ad61f191-ae5f-4c0e-b9fa-e869d288c64f) : Mem.Read / Mem.Write / Io.Read / Io.Write(Width, Address, Count, Buffer) ; une instance par système, sans chemin de périphérique, parent des root bridges ; réentrant ; largeurs Uint8..64, Fifo (adresse fixe), Fill (valeur fixe) ; accès faits tels quels ; UNSUPPORTED si la plage est invalide. V-16 EFI_LEGACY_REGION2_PROTOCOL (70101eaf-0085-440c-b356-8ee36fef24f0) : contrôle la zone 0xC0000-0xFFFFF (recopie des ROM d'option et du code 16 bits) : Decode(Start, Length, Granularity, On) lecture, Lock (interdit l'écriture), BootLock (fige les attributs jusqu'au prochain cycle d'alimentation), UnLock, GetInfo (descripteurs { Start, Length, Attribute, Granularity } ; attributs Decoded, NotDecoded, WriteEnabled, WriteDisabled, BootLocked, NotLocked). V-17 Pile I2C : bus à maître unique ; « configurations de bus » numérotées 0 à N−1 = état des commutateurs et multiplexeurs (un commutateur raccourcit le bus pour une fréquence plus haute, un multiplexeur crée des espaces d'adresses séparés) ; la plateforme doit laisser le bus dans la configuration compatible avec l'OS à ExitBootServices, car certains OS interdisent de changer commutateurs et multiplexeurs. Couches : pilote tiers de puce (identifié par un GUID fabricant, adressage relatif par index dans un tableau d'adresses esclaves, HardwareRevision conseillé égal à _HRV), EFI_I2C_IO_PROTOCOL (par périphérique), EFI_I2C_HOST_PROTOCOL (file FIFO, une seule requête à la fois vers le maître, active la bonne configuration), EFI_I2C_MASTER_PROTOCOL (propre au contrôleur ; a besoin de l'adresse du contrôleur et de sa fréquence d'horloge d'entrée), code plateforme (EFI_I2C_ENUMERATE_PROTOCOL et EFI_I2C_BUS_CONFIGURATION_MANAGEMENT_PROTOCOL) ; périphérique adressé par le couple configuration:adresse esclave ; SMBus peut être bâti sur un contrôleur I2C (une seule configuration) ; course possible entre pilote SMBus et code plateforme, réglée par le numéro de version du Driver Binding (plateforme 0xfffffff0-0xffffffff, SMBus 0x10-0xffffffef) ; comparaison avec PCI (GUID au lieu de Vendor/Device ID, index d'adresses au lieu de BAR) ; branchement à chaud géré par le code plateforme. Début des définitions : EFI_I2C_MASTER_PROTOCOL (cd72881f-45b5-4feb-98c8-313da8117462).
- [AGENT] PERTINENT (pavé tactile et capteurs du portable, souvent en I2C-HID) : (1) Les périphériques I2C ne sont pas énumérables : leur liste, leur adresse esclave et leur contrôleur viennent de l'ACPI (ressource I2cSerialBus dans _CRS, déjà notée dans la lecture ACPI), pas d'un balayage. (2) Un balayage d'adresses I2C en écriture est interdit sans accord (même raison que pour SMBus). (3) Zone 0xC0000-0xFFFFF : peut être verrouillée en écriture, voire figée (BootLock) ; l'OS la traite comme réservée et n'y écrit pas. (4) Le protocole CPU I/O est un moyen sûr de lire un port ou un registre MMIO pendant les services de démarrage, avec contrôle de plage par le firmware.

## Statut
- UEFI PI : lignes 1 à 41036 lues. Prochaine : ligne 41037 (définitions I2C : Master, Host, IO, Enumerate, Bus Configuration Management, PPI ; puis SPI à 42510).

## Lignes 41037-41836 — Définitions I2C (DXE). EFI_I2C_MASTER_PROTOCOL : SetBusFrequency(BusClockHertz : fréquence demandée, rend la fréquence réelle, toujours arrondie vers le BAS, UNSUPPORTED s'il n'y en a pas de plus basse), Reset (suivi obligatoirement de SetBusFrequency), StartRequest(SlaveAddress, RequestPacket, Event, I2cStatus) ; une seule transaction à la fois, pas de file ; adresse 7 bits (bits 0-6) ou 10 bits avec I2C_ADDRESSING_10_BIT = 0x80000000 ; EFI_I2C_REQUEST_PACKET { OperationCount, Operation[] } : start, opérations séparées par des start répétés, stop final ; EFI_I2C_OPERATION { Flags, LengthInBytes, Buffer } ; drapeaux : I2C_FLAG_READ 0x1, SMBUS_OPERATION 0x10000, SMBUS_BLOCK 0x20000, SMBUS_PROCESS_CALL 0x40000, SMBUS_PEC 0x80000 ; table de correspondance des opérations SMBus (QuickRead/QuickWrite = longueur 0 ; ReadDataByte = 2 opérations : écriture d'un octet de commande puis lecture ; ReadBlock = lecture de 33 octets au plus, le premier donne la longueur) ; longueur 0 = « ping » d'adresse, que certains contrôleurs refusent ; EFI_I2C_CONTROLLER_CAPABILITIES { StructureSizeInBytes, MaximumReceiveBytes, MaximumTransmitBytes, MaximumTotalBytes } ; retours ALREADY_STARTED (occupé), DEVICE_ERROR (NACK pendant la transaction), NO_RESPONSE (aucune réponse à l'adresse), NOT_FOUND (bit réservé dans l'adresse), BAD_BUFFER_SIZE. TPL ≤ NOTIFY, synchrone ≤ CALLBACK. EFI_I2C_HOST_PROTOCOL (a5aab9e3-c727-48cd-8bbf-427233854948 : QueueRequest(I2cBusConfiguration, SlaveAddress, Event, RequestPacket, I2cStatus), NO_MAPPING si configuration invalide). EFI_I2C_IO_PROTOCOL (b60a3e6b-18c4-46e5-a29a-c9a10665a28e : QueueRequest(SlaveAddressIndex, …), DeviceGuid, DeviceIndex, HardwareRevision, capacités). EFI_I2C_BUS_CONFIGURATION_MANAGEMENT_PROTOCOL (55b71fb5-17c6-410e-b5bd-5fa2e3d4466b : EnableI2cBusConfiguration ; facultatif ; sans lui le protocole hôte ne démarre pas et le maître peut servir au SMBus ; à ExitBootServices la plateforme laisse la configuration que l'OS devra utiliser). EFI_I2C_ENUMERATE_PROTOCOL (da8cd7c4-1c00-49e2-803e-5214e701894c : Enumerate, GetBusFrequency) ; EFI_I2C_DEVICE { DeviceGuid, DeviceIndex, HardwareRevision, I2cBusConfiguration, SlaveAddressCount, SlaveAddressArray }.
- [AGENT] PERTINENT (futur pilote I2C de l'OS pour le pavé tactile) : (1) Forme d'une lecture de registre I2C : une transaction à deux opérations (écriture de l'adresse de registre, start répété, lecture), jamais deux transactions séparées. (2) Fréquence toujours arrondie vers le bas pour ne dépasser la limite d'aucun périphérique du bus. (3) Distinguer « pas de réponse à l'adresse » (périphérique absent) de « NACK en cours de transaction » (erreur) quand le contrôleur le permet. (4) QuickWrite est une écriture : ne pas s'en servir pour détecter des périphériques.

## Statut
- UEFI PI : lignes 1 à 41836 lues. Prochaine : ligne 41837 (fin I2C Enumerate, PPI I2C, puis V-18 SPI à 42510 et V-19 codes d'erreur à 43602).

## Lignes 41837-42516 — Fin I2C DXE (Enumerate : NULL pour commencer ; GetBusFrequency, pour le diagnostic). V-17.3 I2C en PEI (sous-ensemble) : EFI_PEI_I2C_MASTER_PPI (b3bfab9b-9f9c-4e8b-ad37-7f8c51fc6280 : SetBusFrequency, Reset, StartRequest synchrone, capacités, Identifier) ; EFI_PEI_I2C_HOST (3a12e52d-3bd2-482c-a680-0feb619aebef : StartRequest(I2cBusConfiguration, SlaveAddress, RequestPacket), ControllerNumber ; réservé au pilote de bus) ; GUID EFI_PEI_I2C_BUS_CONFIGURED (9eade134-6bb1-421d-acaf-590a5d2ea63a) installé par la plateforme pour débloquer les dépendances ; EFI_PEI_I2C_IO (42179ed0-2fa2-47c0-857e-8bc01881ea97 : GetDeviceInfo, GetDeviceInfoIdList, StartRequest(SlaveAddressIndex, …), I2cDevice, capacités) ; EFI_PEI_I2C_BUS_CONFIGURATION_MANAGEMENT (e721de6f-145b-4532-bd78-9b4095c74697 : EnableI2cBusConfiguration, I2cDeviceReset appelé après un TIMEOUT pour débloquer un périphérique qui tient SCL ou SDA à l'état bas ; NO_MEDIA = plusieurs périphériques réinitialisés, état perdu, la file de requêtes doit être rendue en erreur) ; EFI_PEI_I2C_ENUMERATE (be83f6f4-e286-4e70-b451-1a2e42df3103). Retours supplémentaires en PEI : TIMEOUT, NO_MEDIA. Début du chapitre V-18 SPI : lignes d'horloge, de sortie et d'entrée partagées, une ligne de sélection (chip select) par périphérique, fréquence maximale, polarité et phase d'horloge propres à chaque périphérique.
- [AGENT] Retenu : un bus I2C peut rester bloqué par un périphérique qui tient une ligne à l'état bas ; le remède est une réinitialisation propre à la plateforme, qui peut faire perdre l'état d'autres périphériques : à prévoir comme erreur rare dans le pilote I2C de l'OS, sans relance automatique en boucle.

## Statut
- UEFI PI : lignes 1 à 42516 lues. Prochaine : ligne 42517 (V-18 SPI, puis V-19 codes d'erreur à 43602, fin à 43663).

## Lignes 42517-43663 — V-18 Pile SPI (fin du document). Couches : application, pilote de puce (périphérique), bus SPI (énumération, transactions, gestion du contrôleur), contrôleur hôte, carte (description des bus et des puces, horloge et chip select de remplacement). Particularité : EFI_SPI_IO_PROTOCOL est publié sous le GUID du pilote de puce (SpiPeripheralDriverGuid), pas sous un GUID générique, car SPI n'a aucun moyen matériel commun d'identifier une puce ; toutes les transactions sont synchrones. Transaction : régler horloge, polarité et phase ; activer le chip select ; transférer ; relâcher le chip select ; fréquence = la plus basse entre contrôleur, puce et ClockHz demandé. Types : FULL_DUPLEX 0, WRITE_ONLY, READ_ONLY, WRITE_THEN_READ (commande + adresse puis lecture : cas des flash NOR) ; si le contrôleur ne sait pas faire le demi-duplex, la couche bus convertit en duplex intégral. Contrôleur de flash SPI « hérité » : trames de 8 bits seulement, horloge fixe, pas de duplex intégral ni de lecture seule, lecture 64 octets au plus, écriture 67 octets au plus, tables de préfixes et d'opcodes ; sécurités : adresse de base du BIOS, registres de plages protégées, verrou de configuration. Structures : EFI_SPI_CONFIGURATION_PROTOCOL (85a6d3e6-b65b-4afc-b38f-c6d54af6ddc8 : BusCount, Buslist), EFI_SPI_PART { Vendor, PartNumber, MinClockHz, MaxClockHz, ChipSelectPolarity }, EFI_SPI_PERIPHERAL { suivant, FriendlyName, SpiPeripheralDriverGuid, SpiPart, MaxClockHz, ClockPolarity, ClockPhase, Attributes (bus de données 2/4/8 bits : 0x1, 0x2, 0x4), ConfigurationData, SpiBus, ChipSelect, ChipSelectParameter }, EFI_SPI_BUS { FriendlyName, Peripherallist, ControllerPath, Clock, ClockParameter }. EFI_SPI_NOR_FLASH_PROTOCOL (b57ec3fe-f833-4ba6-8578-2a7d6a87444b : SpiPeripheral, FlashSize, DeviceId[3], EraseBlockBytes, GetFlashId (3 octets fabricant + périphérique), ReadData, LfReadData (lecture basse fréquence), ReadStatus, WriteStatus, WriteData, Erase par blocs de 4 Kio). EFI_LEGACY_SPI_FLASH_PROTOCOL (f01bed57-04bc-4f3f-9660-d6f2ea228259) = le précédent + BiosBaseAddress, ClearSpiProtect, IsRangeProtected, ProtectNextRange, LockController. EFI_SPI_IO_PROTOCOL { SpiPeripheral, OriginalSpiPeripheral, FrameSizeSupportMask (bit N−1 = trame de N bits, 8 bits obligatoire), MaximumTransferBytes, Attributes (dont TRANSFER_SIZE_INCLUDES_OPCODE 0x8 et _ADDRESS 0x10), LegacySpiProtocol, Transaction, UpdateSpiPeripheral (puces sur support) } ; EFI_SPI_BUS_TRANSACTION { SpiPeripheral, TransactionType, DebugTransaction, BusWidth 1/2/4, FrameSize 1-32, WriteBytes, WriteBuffer, ReadBytes, ReadBuffer }. EFI_SPI_HC_PROTOCOL (c74e5db2-fa96-4ae2-b399-15977fe3002d : Attributes, FrameSizeSupportMask, MaximumTransferBytes, ChipSelect, Clock (fréquence ≤ demandée, 0 = arrêt), Transaction) ; duplex intégral et bus 1 bit obligatoires. EFI_LEGACY_SPI_CONTROLLER_PROTOCOL (39136fc7-1a11-49de-bf35-0e78ddb524fc : MaximumOffset, MaximumRangeBytes, RangeRegisterCount, EraseBlockOpcode, WriteStatusPrefix, BiosBaseAddress, ClearSpiProtect, IsRangeProtected, ProtectNextRange, LockController ; après verrouillage, tables de préfixes et d'opcodes, adresse de base et plages protégées ne changent plus : ACCESS_ERROR). EFI_SPI_NOR_FLASH_BLOCK_MAP_PROTOCOL (EEFF0EF1-6A4C-4B4B-9F84-59045C634BC3 ; tableau { RegionAddress, RegionSize, BlockSize } terminé par une entrée nulle, du bas vers le haut). V-19 Annexe, codes d'erreur 64 bits : EFI_INTERRUPT_PENDING = 0xA000000000000000, EFI_WARN_INTERRUPT_SOURCE_PENDING = 0x2000000000000000, EFI_WARN_INTERRUPT_SOURCE_QUIESCED = 0x2000000000000001. Index alphabétique final (lignes 43619-43663).
- [AGENT] TRÈS PERTINENT (flash du BIOS, règle « ne pas toucher au firmware ») : (1) Le firmware fixe l'adresse de base du BIOS, protège des plages en écriture et en effacement, puis VERROUILLE le contrôleur SPI avant de passer la main au chargeur de l'OS : sur une machine bien configurée, l'OS ne peut plus ni écrire ni effacer les plages protégées, et il ne doit pas essayer. (2) Seules les commandes inscrites dans la table d'opcodes avant le verrouillage restent utilisables. (3) Lecture de l'identifiant de la puce (3 octets) et lecture des données : opérations sans effet, cohérentes avec l'outil de vidage en lecture seule déjà fait. WriteData, WriteStatus et Erase restent interdits sans accord explicite. (4) Coquille du texte source : le GUID du contrôleur SPI hérité est imprimé « 0xlall » pour 0x1a11.

## Statut
- UEFI PI : LU EN ENTIER, lignes 1 à 43663 (fin du fichier), le 2026-11-12. Documents suivants dans l'ordre prévu : AMD APM vol. 2, APM vol. 3, PPR AMD 56569.
