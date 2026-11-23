# Verification code vs specs : xHCI, HDA, UEFI

Perimetre : `<DEPOT_LOCAL>\hwa2_agent\src\usb_r0.rs`, `main.rs` (zones lues), `cpu_sonde.rs`.
Specs : `<DEPOT_LOCAL>\amd\uefi_a_z\docs\specs\` (USB_xHCI_1.2c.txt, HDA_spec_codex.txt, UEFI_Spec_Final_2_11.txt, AMD64_APM_vol3_r339_pdf.txt).
Aucune modification de code, aucune ecriture sur D:\, aucun acces disque/firmware. NVMe non touche.

Notation : "spec xHCI l.N" = ligne N de USB_xHCI_1.2c.txt ; "spec HDA l.N" = HDA_spec_codex.txt ; "spec UEFI l.N" = UEFI_Spec_Final_2_11.txt.

---

## Etat des corrections (mis a jour le 2026-11-14, code relu sur la branche claude/changes)

Ce tableau remplace les "corrections proposees" ci-dessous, dont plusieurs etaient deja appliquees dans le code
au moment de cette relecture (le document ne refletait plus le code).

| Ref | Constat | Etat dans le code |
|---|---|---|
| C1 | HDA Subordinate Node Count inverse | Corrige (usb_r0 `hda_tree` : start = bits 23:16, count = bits 7:0) |
| C2 | HDA IRV jamais efface | Corrige (`hda_cmd` ecrit 0x0002 sur ICS avant chaque ICW) |
| C3 | HDA pas de delai apres CRST=1 | Corrige partiellement : delai TSC de 1 ms ; pas de signal d'erreur si CRST reste a 0 |
| C4 | xHCI CNR non attendu | Corrige (attente CNR=0 avant les ecritures operationnelles) |
| C5 | xHC bus master non pose par le pilote | Ouvert : le pilote suppose que le firmware l'a fait (constate : le xHC repond) |
| C6 | Scratchpad chevauche les zones DMA | Corrige : refus si plus de 5 pages scratchpad |
| C7 | Anneau de commandes sans Link TRB | Corrige (`usb_enum::next_cmd_index` : Link TRB en 255, bascule du PCS) |
| C8 | ExitBootServices jamais appele | Fait dans le code (2026-11-14) : sortie des services UEFI apres les decouvertes, allocateur statique (`src/arena.rs`), rapport ecrit juste avant. **A verifier sur le PC.** |
| C9 | Handoff USB Legacy incomplet | Corrige (OS owned pose ; BIOS force a 0 apres delai, signale dans le rapport) |
| C10 | PAGESIZE non lu | Corrige (bit 0 verifie, erreur sinon) |
| N2 | Ports > 16 ignores | Corrige : lecture jusqu'a 32 ports (`MAX_PORTS_LUS`) |

**C8 (ExitBootServices) : mise en oeuvre.** Avant, l'allocateur UEFI rendait null et appelait les services de
demarrage a chaque liberation : impossible apres la sortie. Maintenant :
1. `src/arena.rs` remplace l'allocateur global par une zone statique de 192 Mio qui ne fait qu'avancer.
2. Le rapport est ecrit sur la cle, puis le systeme de fichiers est oublie (`mem::forget`), ainsi que le resultat
   de `exit_boot_services` ; aucun protocole n'est ferme apres la sortie.
3. Apres la sortie : xHCI, HDA, NVMe, clavier et voix utilisent uniquement des acces materiels directs. Les lignes
   du rapport restent a l'ecran et sont lues a voix haute, mais **ne sont plus ecrites sur la cle**.
4. Fin de session : `runtime::reset` (redemarrage a froid) au lieu de rendre la main.
Risque : un panic apres la sortie ne peut plus afficher le message par la console UEFI. A tester sur le PC ; les
effets sur le firmware USB ne sont pas verifiables sans materiel.

**Audio (hda_play.rs)** : interpolation 16 -> 48 kHz par sinc fenetree a 8 prises (coefficients Q15 en tete du
fichier, somme 32768 par phase) au lieu d'une interpolation lineaire. Pas encore verifie a l'oreille.

## 0. Couverture reelle (lecture partielle, a lire avant tout)

| Document | Lignes totales | Lignes lues (Read) | Lecture ciblee (grep/sed) | Non couvert |
|---|---|---|---|---|
| USB_xHCI_1.2c.txt | 27 951 | environ 2 750 (blocs de 100, plus petits blocs) | sections 4.2, 4.3, 4.9.3-4.9.4, 4.20, 4.21.2, 4.22.1, 5.x registres, 6.2.x TRB, 6.5, 6.6, 7.1, Table 6-90, 6-91 | 5.2 (PCI config), 6.2 (contextes, utilise par usb_enum/usb_hid), 6.4.1 (TRB transfert), 4.19 (root hub), 7.2 (Supported Protocol) |
| HDA_spec_codex.txt | 8 886 | environ 450 | 3.3.7, 3.3.9, 3.4.1-3.4.3, 4.3, 7.3.3.1, 7.3.3.31, 7.3.4.3, 7.3.4.4, 7.3.4.6 | CORB/RIRB, widgets audio hors pin, autres verbes et parametres, hda_play.rs |
| UEFI_Spec_Final_2_11.txt | 99 708 | 40 (7.4.6 ExitBootServices, l.14088-14127) | Pci.Read (l.30310-30330) | Lecture ligne par ligne NON faite : Block IO (13.9), PCI root bridge (14.2), GOP (12.9), USB IO, Boot Services generaux |
| AMD64_APM_vol3_r339 | 42 914 | 0 | 1 point (TBM, l.7107/7163) | Table des bits CPUID de cpu_sonde.rs non verifiee bit a bit |

Le code relu : `usb_r0.rs` (567 lignes, integralite), `cpu_sonde.rs` (277 lignes, integralite), `main.rs` l.1425-1466 (entree zone R0), l.1466-1585 (scan PCI xHCI/NVMe/HDA), l.1585-1714 (appel xHCI et HDA), plus grep sur le reste. Non relus : reste de main.rs, `usb_enum.rs` (seulement `next_cmd_index`), `usb_hid.rs`, `usb_hub.rs`, `hda_play.rs`, `nvme.rs`.

---

## 1. Ecarts critiques (classes par impact)

### C1 - HDA : champs Subordinate Node Count inverses (critique)
- Code : `usb_r0.rs` l.531-532 : `fg_start = root & 0xFF` ; `fg_count = (root >> 16) & 0xFF`. Le commentaire l.530 ("bits 7:0 = noeud de depart, bits 23:16 = nombre") est faux.
- Spec : HDA l.7893-7898, Figure 83 : bits 23:16 = Starting Node Number, bits 7:0 = Total Number of Nodes.
- Effet : groupes de fonctions et widgets enumeres sur les mauvais noeuds des que start != count.
- Correction proposee : `fg_start = (root >> 16) & 0xFF; fg_count = root & 0xFF;` et corriger le commentaire l.530.

### C2 - HDA : bit IRV (ICS bit 1) jamais efface, reponses periment (critique)
- Code : `usb_r0.rs` l.277 (`write_volatile(bar+0x68, 0x0001)`), l.305 (idem dans `hda_cmd`), attente IRV l.279 et l.307 (`& 0x0002`). Aucun effacement de IRV avant une nouvelle commande.
- Spec : HDA l.2069-2073 : IRV est RW1C ; "Software must clear this bit before issuing a new command by writing a one to it". HDA l.2087-2096 : etapes 2 a 5 (ICB puis IRV puis lecture IRR puis effacement IRV).
- Effet : apres la premiere reponse, IRV reste a 1 ; la boucle d'attente sort immediatement et IRR contient la reponse de la commande precedente. Tout l'arbre HDA et la lecture du fournisseur sont alors faux. A confirmer sur materiel (la spec ne dit pas que le controleur efface IRV seul).
- Correction proposee : avant d'ecrire ICW, ecrire `0x0002` sur ICS (efface IRV), attendre ICB=0, ecrire ICW, puis ecrire `0x0001` sur ICS, puis attendre IRV=1. Meme correction dans `hda_probe` (l.277) et `hda_cmd` (l.305).

### C3 - HDA : pas de delai de 521 us apres CRST=1 (critique)
- Code : `usb_r0.rs` l.245-256 : apres CRST=1, boucle de test de CRST sans delai ; l.260 lit STATESTS immediatement. Si l'attente expire (l.249-253), la fonction continue sans erreur.
- Spec : HDA l.2387-2390 (section 4.3 Codec Discovery) : "software must wait at least 521 us (25 frames) after reading CRST as a 1" avant de considerer les codecs enumeres. Spec HDA 3.3.7 l.1197-1199 : CRST lu a 1 avant tout acces aux registres.
- Effet : STATESTS peut etre a 0 (codec_found = false) alors qu'un codec existe.
- Correction proposee : attente >= 521 us (boucle calibree ou TSC) apres CRST=1 ; renvoyer une erreur si CRST ne passe pas a 1.

### C4 - xHCI : CNR non attendu avant la premiere ecriture operationnelle (critique)
- Code : `usb_r0.rs` l.110 : `wr32(op, rd32(op) & !1)` (ecriture USBCMD) avant toute attente de CNR. CNR n'est teste qu'apres HCRST (l.121).
- Spec : xHCI l.1767 : "After Chip Hardware Reset wait until CNR is 0 before writing any xHC Operational or Runtime registers". Spec xHCI l.12706-12714 (USBSTS bit 11, CNR) : "Software shall not write any Doorbell or Operational register ... until CNR = 0".
- Correction proposee : lire USBSTS (op+4) et attendre CNR=0 (bit 11) avant la ligne 110, avec delai borne.

### C5 - xHCI : bus master et acces memoire non actives pour le controleur xHCI (critique si firmware ne l'a pas fait)
- Code : `usb_r0.rs` ne touche jamais au registre Command PCI du xHC. Seule `hda_probe` ecrit le registre 0x04 (l.239-242) pour le HDA. `main.rs` l.1486-1499 ne conserve pas le numero de bus/device/fonction du xHC (seulement la base).
- Spec : xHCI l.9376-9377 (4.21.2) : Bus Master Enable doit etre a 1 pour que le xHC fasse du DMA ; BME a 0 pendant l'execution -> HCE.
- Limite : les bits du registre Command PCI (memory space, bus master) ne sont pas dans les specs fournies (spec PCI non fournie). Verification indirecte uniquement.
- Correction proposee : conserver le BDF du xHC au scan ; avant `xhci_bringup`, lire le registre 0x04 et poser memory space et bus master ; verifier la relecture.

### C6 - xHCI : buffers scratchpad chevauchent les zones DMA des peripheriques (critique si Max Scratchpad >= 6)
- Code : `usb_r0.rs` l.9 (XHCI_MEM, 64 pages), l.14 (`DEV_BASE = 16`), l.136-141 (tableau scratchpad en page 10, buffers en pages 11 a 11+n-1), l.419 (`ZONE_NEXT = 16`).
- Spec : xHCI l.9327-9337 (4.20) : buffers scratchpad alloues par logiciel, adresses dans le tableau pointe par DCBAA[0] ; l.9331 : "System software shall not read or write a Scratchpad buffer". Spec xHCI 6.6 (l.20541-20545) : tableau en entree 0 du DCBAA.
- Effet : pour n >= 6, la page 16 (scratchpad) est aussi la page 0 de la zone du premier peripherique (contexte d'entree). Le code limite n a 16 (l.75), donc chevauchement jusqu'a la page 26.
- Correction proposee : placer les zones peripheriques apres 11+n (ou allouer les scratchpads hors de XHCI_MEM), et verifier n avant tout allocation de zone.

### C7 - xHCI : anneau de commandes sans Link TRB ni bascule de cycle (critique a moyen terme)
- Code : `usb_r0.rs` l.400-406 : TRB ecrit a `cmd + idx*16`, cycle fixe a 1. `usb_enum.rs` l.67-74 : `next_cmd_index` incremente sans borne.
- Spec : xHCI l.4703-4705 (4.9.3) : le producteur bascule le cycle a chaque passage ; l.1271 (4.9.2) : un anneau est forme par un Link TRB en fin d'anneau ; Link TRB = type 6 (Table 6-91, l.20214-20290).
- Effet : apres 256 commandes, l'ecriture sort de la page de commandes et chevauche l'anneau d'evenements (base + 2*4096). Cycle non inverse : le controleur lit des TRB invalides.
- Correction proposee : reserver le dernier TRB de la page comme Link TRB (pointant vers le debut, TC=1), inverser le PCS a chaque passage, et reutiliser l'index modulo 255.

### C8 - UEFI : ExitBootServices jamais appele alors que R0_EBS = true (critique pour le principe R0)
- Code : `main.rs` l.51 (`R0_EBS = true`, commentaire "sortie des services UEFI"), l.1429-1431 (bloc R0 : commentaire "Tout ce qui allouerait ... avant ExitBootServices"). Recherche insensible a la casse de "exit" dans `main.rs` : seule occurrence l.1431 (commentaire). Aucun appel a `boot::exit_boot_services` ou equivalent.
- Spec : UEFI l.14108-14110 (7.4.6) : apres ExitBootServices, "No further calls to boot service functions or EFI device-handle-based protocols may be used", et MapKey doit venir d'un GetMemoryMap appele juste avant.
- Effet : les appels `r.save()` (l.1450, l.1582, puis l.1716-1729) utilisent encore le systeme de fichiers via protocoles UEFI, et le firmware garde la main (son pilote USB peut encore posseder le xHC : voir C9 et spec xHCI l.9380). Le bloc R0 n'est donc pas en R0 reel.
- Correction proposee (a decider par l'auteur, modification de comportement) : GetMemoryMap puis ExitBootServices immediatement avant la phase R0, puis n'appeler que des acces materiels directs ; deplacer `r.save()` avant la sortie.

### C9 - xHCI : handoff USB Legacy (propriete OS) incorrect quand le BIOS ne possede pas le xHC (moyen)
- Code : `usb_r0.rs` l.89-91 : le bit OS Owned (bit 24) n'est pose que si le bit BIOS Owned (bit 16) est a 1.
- Spec : xHCI l.20811-20825 (7.1.1) : le systeme d'exploitation met son semaphore a 1 pour demander la propriete ; la propriete est acquise quand OS=1 et BIOS=0. Spec l.9380 (4.22.1) : un handoff incorrect laisse deux agents croire qu'ils possedent le xHC.
- Correction proposee : toujours poser le bit 24 (32 bits ou acces octet, spec l.20831), puis attendre BIOS=0 avec un delai borne.

### C10 - xHCI : PAGESIZE non lu, 4 KiB suppose (moyen/faible)
- Code : toutes les pages DMA font 4096 octets (`usb_r0.rs` l.5-9). PAGESIZE (op+0x08) n'est jamais lu.
- Spec : xHCI l.12758-12769 (5.4.3) : bit n a 1 => page 2^(n+12) ; l.9316 (4.20) : les buffers scratchpad sont de taille PAGESIZE.
- Correction proposee : lire op+0x08, verifier le bit 0 (4 KiB) et renvoyer une erreur propre sinon.

---

## 2. Autres constats (non critiques)

- N1 (xHCI) `wait_event` ne verifie pas le champ Command TRB Pointer de l'evenement (spec xHCI 6.4.2.2, l.17693-17695). Acceptable tant qu'une seule commande est en vol.
- N2 (xHCI) `xhci_bringup` ignore les ports > 16 (`max_ports.min(16)`, l.178). Spec xHCI l.11413-11415 : MaxPorts jusqu'a 255.
- N3 (xHCI, hors perimetre) HCCPARAMS1.CSZ (bit 2, spec xHCI l.11708-11710 : 64 octets si 1). Non lu dans le code de ce perimetre ; a verifier dans `usb_enum.rs` / `usb_hid.rs` qui manipulent les contextes.
- N4 (xHCI) `IMAN` est ecrit avec `rd32(ir0) | 2` (l.159). IP est RW1C (spec l.14492) : si IP etait a 1 la lecture-modification-ecriture l'effacerait. Faible, IP est a 0 a l'init.
- N5 (xHCI) Interruptions : INTE et IE sont poses sans MSI/MSI-X ; le code scrute l'anneau (polling). Conforme a la note 7 de la spec xHCI l.1796 (interruptions optionnelles).
- N6 (xHCI) Enable Slot : Slot Type = 0 (l.18407 renvoie a 7.2.2.1.4). Spec xHCI l.21641 : le champ Protocol Slot Type des protocoles USB vaut 0. Conforme.
- N7 (xHCI) Ecriture PORTSC : masque des bits RW1C (17-23) et PED (bit 1) conforme (spec xHCI l.13176 PED RW1CS, l.13569 PRC RW1CS, l.13495 CSC RW1CS). PR (bit 4) et PED ne sont jamais poses ensemble (note 82, l.13228). Conforme.
- N8 (HDA) `hda_cmd` attend ICB=0 avant ICW (spec HDA l.2030-2031, l.2079-2083). Conforme. Note : ICB ne se libere que si le CORB est inactif (l.2081) ; le code ne desactive pas un CORB laisse actif par le firmware.
- N9 (HDA) `hda_probe` n'arrete pas la boucle CRST sur erreur et continue (l.249-253), voir C3.

---

## 3. Conformite verifiee (valeurs confirmees par lecture de la spec)

xHCI :
- CAPLENGTH = octet 0 (spec l.11316-11330) : code l.66 OK.
- HCSPARAMS1 : MaxSlots bits 7:0 (l.11398-11404), MaxPorts bits 31:24 (l.11413-11415) : code l.70-71 OK.
- HCSPARAMS2 : Max Scratchpad Hi bits 25:21 (l.11500-11502), Lo bits 31:27 (l.11526-11528), total = Hi<<5 | Lo : code l.74 OK.
- DBOFF bits 31:2 (l.11851-11854) : code l.407 OK (`& !0x3`). RTSOFF bits 31:5 (l.11898-11901) : code l.78 OK (`& !0x1F`).
- USBCMD : R/S bit 0, HCRST bit 1, INTE bit 2 (l.12306-12352) : code l.110, 119, 162 OK.
- USBSTS : HCH bit 0 (l.12583), CNR bit 11 (l.12706) : code l.112, 121 OK.
- CRCR : RCS bit 0 (l.12863-12875), pointeur 64 octets aligne (l.12923) : code l.147 OK (page alignee).
- DCBAAP : 64 octets alignes (l.12957) ; entree 0 = scratchpad (l.9327-9336) : code l.133-144 OK.
- CONFIG a op+0x38, MaxSlotsEn bits 7:0 (l.12993, l.13031) : code l.128 OK.
- PORTSC : registres 16 octets par port (l.12214 : groupes consecutifs), PRC bit 21, PED bit 1, PR bit 4 : code l.372, 379, 381, 387-388 OK.
- Runtime : IR0 = RTSOFF+0x20 (l.14468), IMAN +0 (IE bit 1, l.14502), IMOD +4 (l.14526), ERSTSZ +8 (l.14578), ERSTBA +0x10 (l.14600), ERDP +0x18 (l.14634), EHB bit 3 (l.14656) : code l.153-159 OK.
- Doorbell 0 : DB Target 0 = commande (l.14770-14776) : code l.408 OK.
- ERST : base 63:6 (DW0/DW1), taille 15:0 a l'offset 08 (l.20496-20529) : code l.150-152 OK.
- Evenement Command Completion : type 33 (Table 6-91, l.20214-20290), code de fin bits 31:24 du DW2, Slot ID bits 31:24 du DW3 (l.17697-17776) : code l.493-497 OK.
- Commande Enable Slot : type 9 bits 15:10, C bit 0 (l.18384-18403) : code l.406 OK.
- Code de succes = 1 (Table 6-90, l.19826-19836) : utilise implicitement l.410 OK.
- Ordre d'initialisation (spec l.1765-1812) : BIOS handoff, arret, HCRST, MaxSlotsEn, DCBAAP, scratchpad (avant R/S, l.9319), CRCR, ERSTSZ, ERDP, ERSTBA, IMAN.IE, INTE+R/S : code l.63-169 conforme, sauf C4 (CNR).

HDA :
- GCTL CRST bit 0 (l.1185-1197) : code l.245-247 OK.
- STATESTS a 0x0E, 15 bits SDIN (l.1231-1239) : code l.260-261 OK.
- ICOI 0x60 (ICW, l.2023-2031), IRI 0x64 (IRR, l.2032-2046), ICS 0x68 : ICB bit 0, IRV bit 1 (l.2057-2094) : adresses et bits OK, effacement IRV KO (C2).
- Verbe : format GET_PARAMETER F00h + id (l.5425-5427) : code l.275 OK.
- Subordinate Node Count : fields inverses (C1).
- Function Group Type bits 7:0, 01h = audio (l.7915-7928) : code l.537 OK.
- Audio Widget Capabilities : Type bits 23:20 (l.7970), table 138 (l.7982-7993, pin = 4h) : code l.549-550 OK.
- Configuration Default verbe F1Ch (l.7014-7015) : code l.552 OK ; Port Connectivity bits 31:30 et Default Device bits 23:20 (l.7022-7024) : code l.553-554 OK.

UEFI :
- ExitBootServices : voir C8.
- Pci.Read : signature et largeur de type UINT8 (l.30310-30330, lecture partielle). Le code lit `a.with_register(0)` avec un tampon de 64 octets (`main.rs` l.1485) et 4096 octets pour l'etendu (l.875) : non verifie en detail (spec 14.2.8 non lue).

---

## 4. Ce qui reste non verifie

- Lecture ligne par ligne de UEFI_Spec_Final_2_11.txt : non faite (99 708 lignes). Seul 7.4.6 est lu.
- Lecture complete de USB_xHCI_1.2c.txt : environ 10 % lu en blocs, le reste par recherche ciblee.
- HDA : environ 5 % lu en blocs. CORB/RIRB et l'essentiel de hda_play.rs non verifies.
- cpu_sonde.rs : positions de bits de la table FLAGS non verifiees contre AMD APM vol 3, sauf TBM (Fn8000_0001_ECX bit 21, l.7107 et 7163 de l'APM).
- Bits du registre Command PCI (memory space, bus master, l.0x04) : spec PCI non fournie.
- Codes de dimension fixes du pilote (BAR 64 bits, VID/DID 1022:15E3) : non verifies (hors specs fournies ou specs non lues).
- Tous les constats sont des lectures de code et de spec ; aucun n'a ete reproduit sur materiel.

## 5. Corrections proposees (ordre de priorite, sans application)

1. C1 : inverser start/count dans `hda_tree` (usb_r0.rs l.531-532).
2. C2 : effacer IRV (ecrire 0x0002 sur ICS) avant chaque ICW, dans `hda_probe` (l.277) et `hda_cmd` (l.305).
3. C4 : attendre CNR=0 avant l.110 de usb_r0.rs.
4. C3 : delai >= 521 us apres CRST=1, erreur si CRST non lu.
5. C6 : deplacer les zones DMA apres les buffers scratchpad, ou verifier n < 6.
6. C5 : activer memory space et bus master du xHC avant `xhci_bringup`.
7. C7 : Link TRB et bascule du cycle sur l'anneau de commandes.
8. C8 : ExitBootServices explicite avant la phase R0 (decision de l'auteur).
9. C9 : toujours poser OS Owned, puis attendre BIOS Owned = 0 avec delai.
10. C10 : verifier PAGESIZE bit 0.
