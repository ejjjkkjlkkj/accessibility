# Verification de l'agent contre les specifications (PDF extraits en texte)

Sources : `<DOSSIER_EXTRAITS_PDF>\*.txt` (extraits de `<DOSSIER_SPECS_PDF>\*.pdf`).

## xHCI 1.2c (USB_xHCI)
- PORTSC : PR bit 4 (RW1S), PED bit 1 (RW1CS, ecrire 1 desactive le port), CCS bit 0, vitesse bits 13:10,
  PLS bits 8:5, PP bit 9, bits de changement RW1C 17..23 (CSC, PEC, WRC, OCC, PRC, PLC, CEC).
  Le code ecrit PR en gardant les bits de changement a 0 et PED a 0 : correct.
- TRB : Normal 1, Setup 2, Data 3, Status 4, Link 6, Enable Slot 9, Address Device 11,
  Configure Endpoint 12, Transfer Event 32, Command Completion 33. Le code : correct.
- Sémaphores USB legacy : BIOS bit 16, OS bit 24, USBLEGCTLSTS = registre +4. Le code : correct.
- Intervalle interruption pleine/basse vitesse : floor(log2(bInterval)) + 3. Le code : correct.
- Scratchpad : presents sur le PC (2 pages, lu dans le rapport). Le code les alloue : corrige.
- Contexte emplacement : Context Entries bits 31:27, Speed 23:20, Root Hub Port 23:16 (adresse 0x4). A verifier par lecture de la section 6.2.2.
- Contexte endpoint : EP Type bits 5:3, CErr bits 2:1, MPS bits 31:16, Interval bits 23:16.
- ERDP : bit EHB = bit 3 ; le code l'ecrit avec l'adresse courante + 1 : correct.

## NVMe 2.4 (SMART / Health Information Log, figure 213)
Decalages verifies dans `usb_r0` / `main.rs` (phase 13) :
- CTEMP en Kelvin (octets 1-2) : code soustrait 273 : correct.
- Critical Warning (octet 0), Available Spare (octet 3) : correct.
- Data Units Read 32, Data Units Written 48, Host Read Cmd 64, Host Write Cmd 80,
  Power Cycles 112, Power On Hours 128, Unsafe Shutdowns 144, Media Errors 160,
  Number of Error Log Entries 176 : correct.

## HDA 1.0 (high-definition-audio-specification)
- Registres : ICOI 60h, ICII 64h, ICIS 68h (ICB bit 0, IRV bit 1), GCTL CRST bit 0 : correct.
- Parametres GET_PARAMETER : 04h (Subordinate Node Count), 05h (Function Group Type), 09h (Audio Widget Caps) : correct.
- Verbe 0xF1C (Get Configuration Default) : utilise pour les broches ; a verifier contre la section 7.3.3.31.

## Non verifie (a faire)
- Registres exacts des contextes xHCI (section 6.2) : lire les tableaux de bits.
- Format des Event Ring Segment Table (ERST) : section 6.5.
- Protocole HID boot et format des rapports clavier : pas dans les PDF fournis (documentation USB-IF HID 1.11, absente).
- Hubs USB : spec de classe hub absente.
- Tables UEFI utilisees (GOP, ExitBootServices, ConIn) : UEFI 2.11 disponible, a relire pour les signatures.

## Corrections (2026-11-12)
- xHCI HCSPARAMS2 (5.3.3) : Max Scratchpad Buffers = (Hi bits 25:21 << 5) | Lo bits 31:27. Le code inversait Hi et Lo : corrige dans `usb_r0.rs`.
- xHCI ERST (5.3.1 / 6.5) et IR0 (ERSTSZ +08, ERSTBA +10, ERDP +18) : verifies, corrects.
- HID boot (USB HID 1.11, App. B) : SET_PROTOCOL 0x21/0x0B, wValue 0 (boot), wIndex = interface ; Shift = bits 1 et 5 (masque 0x22) : verifies, corrects.
- HDA 1.0 7.3.4.3 (Subordinate Node Count) : bits 7:0 = noeud de depart, bits 23:16 = nombre. Le code inversait les deux (root et sous-noeuds) : corrige dans `usb_r0.rs` (arbre HDA).
- NVMe 2.4 : CC (IOCQES 23:20, IOSQES 19:16, EN), AQA, doorbells (stride DSTRD), Identify (CNS 1 et 0), LBAF/LBADS, Create IO CQ/SQ, Read, phase tag DW3 bit 16 : verifies, corrects, aucune modification.
- Hub USB 2.0 ch.11 + xHCI 6.2.2 (contexte emplacement) : GET_DESCRIPTOR hub (0xA0, 0x2900), GET_STATUS (0xA3), SET/CLEAR_FEATURE, bits wPortStatus (bit 9 basse vitesse, bit 10 haute vitesse), DW0/1/2 du contexte : verifies, corrects, aucune modification.
- AMD APM vol.2 rev 3.43 (Table A-1, A-8) : SYSCFG C001_0010, HWCR C001_0015, SMBASE C001_0111, SMM_ADDR C001_0112, SMM_MASK C001_0113, VM_CR C001_0114 verifies. C001_001F n'est pas dans l'APM 3.43 : conserve, marque "non documente" dans le rapport. Noms ajoutes aux lignes MSR du rapport.
- AMD APM vol.3 Appendix E (CPUID Fn0000_0001_EAX) : famille/modele/stepping decodes et compares a la PPR 56569 (Family 19h Model 51h A1). Ligne ajoutee au rapport phase 17.
- AMD PPR 56569 : identifiants PCI 1638/1637 (GPU et HDA d'affichage), 790B/790E (SMBus/LPC), 15E2/15E3/1639 : confirmes presents dans la PPR.
- AMD APM vol.2 rev 3.43 : SMBASE/SMM_ADDR/SMM_MASK = C001_0111/0112/0113 (correction des libelles).
- Sonde CPU intégrée (src/cpu_sonde.rs, appelée en phase 17) : CPUID seul, aucun MSR. Feuilles 0, 1, 4, 7, B, 8000_0000–8000_0008, 8000_001F ; bits de fonctionnalités d'après APM vol. 3 annexe D/E ; XCR0 via XGETBV seulement si OSXSAVE.
- Hachage complet du disque déplacé après la phase 18 (fonction hash_disque_entier), pour qu'un blocage ne masque pas les autres sondes.
- PCI : toutes les BAR (type, adresse, 64 bits, prefetch), capacités standard et étendues décodées (decrire_pci). Lecture seule, aucune écriture de BAR (pas de sondage de taille).
- SMBIOS : BIOS, système, carte mère, processeur, tableau et modules mémoire décodés (decrire_smbios). Numéros de série et UUID non lus.
- À faire : décodage AML de la DSDT/SSDT, SRAT/SLIT. Remarque : la phase NVMe affiche encore le numéro de série du contrôleur (ligne Identify), contrairement à la règle du README : à corriger.
- CR0 et CR4 décodés bit par bit (APM vol. 2 rév. 3.43, §3.1.1 et §3.1.3). Source vol. 1 : vue d'ensemble seulement, les bits sont dans vol. 2.
