# NVMe 2.4 - notes de lecture ligne par ligne (NVMe_Base_Spec_2.4.txt, 37300 lignes)

Chaque bloc de 250 lignes est noté ci-dessous.

## Lignes 1-100 (page de garde, avis légal, table des matières)
- NVMe Base Spec rev 2.4, 31 juillet 2026 ; intègre 2.3, TP4176 ... ECN141.
- Avis d'obsolescence (août 2027) : Predictable Latency Mode (§8.1.21) et NVM Sets (§3.2.2).
- Plan plan : §1 intro/définitions ; §2 théorie ; §3 architecture (contrôleurs, propriétés §3.1.4 p.52, sous-systèmes, namespaces §3.2, queues §3.5, init §3.6 p.105, shutdown §3.6, resets §3.7 p.120) ; §4 structures (SQE p.139, CQE p.144, status p.145, phase tag p.155).
- Pertinence agent : Identify, CC/CSTS, AQA, doorbells, Create IO CQ/SQ, Read, phase tag, reset/init.

## Lignes 101-400 (TOC détaillée + liste des figures + début §1)
- §4 structures : SQE p.139 (CDW0 fig.92, format commun fig.93), CQE p.144 (DW2/DW3, status fig.101), Status code type fig.102 ; phase tag p.155 (fig.109) ; PRP fig.110-113 ; SGL p.159-166 (fig.114-124).
- §5 Admin : Identify p.334 (CNS fig.336, Identify Controller fig.338, Namespace fig.346) ; Get Features p.209 ; Set Features p.454 ; Get Log Page p.212 (SMART fig.213) ; Create I/O CQ p.524 (fig.571-574) ; Create I/O SQ p.525 (fig.575-579) ; Delete I/O CQ/SQ p.527-528 ; Format NVM p.206 ; Firmware Commit p.202 ; Keep Alive p.403.
- §3.6 Init : CC/CSTS/AQA/ASQ/ACQ/CAP dans propriétés (fig.36-46 : CAP p.55, CC p.60, CSTS p.63, AQA p.66, ASQ p.66, ACQ p.67).
- §1.4 : conventions 0-based ; little endian par défaut ; ASCII/UTF-8 et padding par espaces ; hex suffix h, binaire b.
- §1.5 : Admin Queue = SQ/CQ d'identifiant 0 ; Controller Reset = CC.EN 1->0 ; command submission (PCIe) = doorbell tail ; command completion = CQE postée.

## Lignes 401-600 (§1.5 définitions suite, §1.6-1.8, références, §2 Theory of Operation début)
- Idempotent command : §9.6.3.1. Identify Controller = CNS 01h et CNS 06h ; Identify Namespace = CNS 00h/09h/11h (+08h/1Fh indep., 05h/0Ah/1Bh I/O set). Ces CNS sont à vérifier dans le code (§5.2.14).
- Namespace privée/partagée : NMIC dans Identify Namespace (fig.346). Controller statique/dynamique (1.5.29/1.5.100).
- Firmware slots : jusqu'à 7 images par domaine (1.5.44). Admin Queue = SQ/CQ 0 uniquement, Fabrics commands passent même si CC.EN=0.
- §2 : jusqu'à 65 535 I/O queues, 65 535 commandes par queue ; ≤1 écriture MMIO (doorbell) sur le chemin de soumission ; 4 KiB read tient dans le SQE de 64 B.
- Transport PCIe = modèle memory-based (SQ/CQ en mémoire hôte). Admin Command Set : seules commandes Admin/Fabrics sur la SQ admin.
- Références : NVMe NVM Command Set 1.3, NVMe PCIe Transport 1.4, UEFI 2.10 (réf. citée ; la spec UEFI fournie est 2.11), ACPI 6.5, PCIe 6.2.

## Lignes 601-1000 (§2.1-2.3 : SQ/CQ, mapping, §2.2 Fabrics, §2.3 storage model, exemples)
- SQ : buffer circulaire, slots fixes, 64 octets par commande ; doorbell SQ Tail mis à jour par l'hôte ; contrôleur lit dans l'ordre mais exécute hors ordre. PRP (max 2 entrées) ou SGL (1 segment) ; liste PRP si plus de 2 entrées.
- CQ : buffer circulaire ; plusieurs SQ peuvent pointer sur une même CQ ; CQ Head mis à jour par l'hôte ; bit Phase Tag (P) indique entrée nouvelle sans lire de registre (§4.2.4).
- Fabrics §2.2 : 1 SQ <-> 1 CQ ; pas d'interruption (rôle HBA) ; SGL obligatoires pour Admin/I/O/Fabrics ; pas de PRP ; pas de flow control CQ.
- Capsules : commande ≥64B ; réponse ≥16B. Discovery Service = sous-système avec controllers Discovery seulement.
- Namespace : lié à exactement un I/O Command Set fixé à la création. Hiérarchie : Subsystem > Domain > Endurance Group > NVM Set / Reclaim Group > Namespace.
- Pour l'agent (PCIe) : admin uses PRP, SGL non supporté pour l'admin commands ; données via PRP1/PRP2.

## Lignes 1001-1300 (§2.4 multi-path/sharing, §3.1 modèle statique/dynamique, types de contrôleurs, Discovery, CNTLID FFF0h-FFFFh, Figure 28 Admin command support)
- Namespace privée vs partagée ; NSID identique pour un namespace partagé sur tous les controllers ; Identify Namespace identique.
- §3.1 : PCIe = modèle statique uniquement. Contrôleur I/O / Administratif (pas d'I/O queue, pas de namespace) / Discovery (pas d'I/O, Keep Alive interdit, pas de Disconnect).
- CNTLID : FFFEh = any static, FFFFh = any dynamic ; FFF0h-FFFCh réservé ; FFFDh réservé (dispersed).
- Admin command support (fig.28) : Delete/Create I/O SQ/CQ, Get Log Page, Identify, Abort, Set/Get Features, AER, Namespace Management, Firmware Commit/Download, Device Self-test, Namespace Attachment, Keep Alive (opcodes 00h,01h,02h,04h,05h,06h,08h,09h,0Ah,0Ch,0Dh,10h,11h,14h,15h,18h) ; colonnes I/O/Admin/Discovery M/P.
- Pour le code : la liste Admin de l'agent doit contenir au minimum Identify (06h), Get Log Page (02h), Get/Set Features (0Ah/09h), Create/Delete I/O CQ/SQ (05h/01h/04h/00h) ; à vérifier.

## Lignes 1301-1600 (fig.28 suite : opcodes Admin et support I/O/Admin/Discovery)
- Opcodes Admin : Directive Send 19h, Directive Receive 1Ah, Virtualization Mgmt 1Ch, NVMe-MI Send 1Dh, NVMe-MI Receive 1Eh, Capacity Mgmt 20h, Discovery Info Mgmt 21h, Fabric Zoning Receive 22h, Lockdown 24h, FZ Lookup 25h, Clear Exported NVM Resource 28h, FZ Send 29h, Manage Exported NVM Subsystem Receive 2Ah / Send 2Dh, Manage Exported Namespace 31h, Manage Exported Port 35h, Cross-Controller Reset 38h, Send Discovery Log Page 39h, Track Send 3Dh, Track Receive 3Eh, Migration Send 41h, Migration Receive 42h.
- Références de section fig.28 : Create I/O CQ 5.3.1, Create I/O SQ 5.3.2, Delete I/O CQ 5.3.3, Delete I/O SQ 5.3.4, Get Features 5.2.12, Get Log Page 5.2.13, Identify 5.2.14, Set Features 5.2.30, Abort 5.2.1, AER 5.2.2, Keep Alive 5.2.15, Firmware Image Download 5.2.10, Firmware Commit 5.2.9, Device Self-test 5.2.6, Namespace Attachment 5.2.24, Namespace Management 5.2.25.
- Pour l'agent : la liste de commandes admin utilisées par main.rs est à comparer (Identify, Get Log Page, Create/Delete IO queues, Set Features si utilisé).

## Lignes 1601-1900 (fig.28 fin, notes 1-11, fig.29 Fabrics, fig.30 I/O commons)
- Opcodes Admin : Controller Data Queue 45h, Doorbell Buffer Config 7Ch, Format NVM 80h, Security Send 81h, Security Receive 82h, Sanitize 84h, Sanitize Namespace 8Ch, I/O-specific 85h/86h/88h/89h. Format NVM et Sanitize sont listés comme optionnels.
- Notes clés : M pour memory-based (Doorbell Buffer Config, Note 9) ; Keep Alive obligatoire seulement si Keep Alive feature ; Discovery controllers CDC : DIM/FZ commands obligatoires.
- Fabrics (fig.29) : Property Set 00h (M), Connect 01h (M), Property Get 04h via type 7Fh (M), Auth Send 05h / Auth Recv 06h (O), Disconnect 08h (O I/O, P Admin/Disc) ; vendor 7Fh C0h-FFh. NOTE : pour PCIe toutes les commandes Fabrics sont interdites.
- I/O commons (fig.30) : Flush 00h, Reservation Register 0Dh, Reservation Report 0Eh, Reservation Acquire 11h, I/O Mgmt Receive 12h, Reservation Release 15h, Cancel 18h, I/O Mgmt Send 1Dh.

## Lignes 1901-2200 (fig.30 fin, fig.31 Log Page Support Requirements)
- Notes fig.30 : Reservations (note 2) mandatoires si supportées (Identify Controller) ; FDP (note 3, bit FDPS fig.338).
- fig.31 (Log pages) : la table PDF est déformée à l'extraction ; la liste des identifiants lus : 01h Error Information, 02h SMART/Health (controller + namespace), 03h Firmware Slot, 04h Changed Namespace List, 05h Commands Supported and Effects, 06h Device Self-test, 07h Telemetry Host-Initiated, 08h Telemetry Controller-Initiated, 09h Endurance Group Information, 0Ah Predictable Latency Per NVM Set, 0Bh Predictable Latency Event Aggregate, 0Ch ANA, 0Dh Persistent Event, 0Eh LBA Status Information, 0Fh Endurance Group Event Aggregate, 10h Media Unit Status, 11h Supported Capacity Configuration List, 12h Feature Identifiers Supported and Effects, 13h NVMe-MI Commands Supported and Effects, 14h Command and Feature Lockdown, 15h Boot Partition, 16h Rotational Media, 17h Dispersed Namespace Participating NVM Subsystems, 18h Management Address List, 19h Physical Interface Receiver Eye Opening, 1Ah Reachability Groups, 1Bh Reachability Associations, 1Ch Allocated Namespace List (Changed?), 1Dh Device Personalities, 1Eh Cross-Controller Reset, 1Fh Lost Host Communication, 20h FDP Configurations, 21h Reclaim Unit Handle Usage, 22h FDP Statistics, 23h FDP Events, 24h Manufacturer Default Configuration Status, 25h Power Measurement, 27h Voltage Measurement, 28h Rate Limiting, 70h Discovery, 71h Host Discovery, 72h AVE Discovery, 73h Pull Model DDC Request, 7Fh Sanitize Namespace Status List, 80h Reservation Notification, 81h Sanitize Status, 82h Program List, 83h Downloadable Program Types List, 84h Memory Range Set List, BFh Changed Zone List.
- ATTENTION : les numéros exacts 00h-14h du fig.31 sont extraits avec décalage ; à confirmer avec la section 5.2.13 (Get Log Page, LID) avant tout code.
- Pour l'agent : SMART (02h) et Error Information (01h) sont M ; c'est une cible de vérification (SMART Unsafe Shutdowns 1324 déjà présent dans le rapport).

## Lignes 2201-2500 (fin fig.31 avec sections 5.2.13.1.x / 5.2.13.3.x, notes 1-13, début fig.32 Feature Support)
- Sections Log Page : 5.2.13.1.1 à 5.2.13.1.38 (une par LID) ; 5.2.13.3.x = log pages Discovery (fabrics).
- fig.32 Feature IDs (ordre lu, fiable) : 01h Arbitration (M I/O, ref 5.2.30.1.1), 02h Power Management (M), 03h LBA Range Type (NVM CS), 04h Temperature Threshold (M), 05h Error Recovery (NVM CS), 06h Volatile Write Cache (M), 07h Number of Queues (M), 08h Interrupt Coalescing, 09h Interrupt Vector Configuration, 0Ah Write Atomicity Normal, 0Bh Asynchronous Event Configuration (M), 0Ch Autonomous Power State Transition, 0Dh Host Memory Buffer, 0Eh Timestamp, 0Fh Keep Alive Timer, 10h Host Controlled Thermal Management, 11h Non-Operational Power State Config, 12h Read Recovery Level Config.
- Règle : si une feature est supportée, Set Features et Get Features doivent l'être (fig.32 intro).
- Notes Log Page : note 2 Fixed Capacity Management ; note 4 transport-specific ; note 6 Reachability ; note 9 Set Features optionnel si absent.

## Lignes 2501-2800 (fig.32 suite)
- Feature IDs : 13h Predictable Latency Mode Config, 14h PLM Window, 15h LBA Status Information Report Interval, 16h Host Behavior Support, 17h Sanitize Config, 18h Endurance Group Event Config, 19h I/O Command Set Profile, 1Ah Spinup Control, 1Bh Power Loss Signaling Config, 1Ch Performance Characteristics, 1Dh Flexible Data Placement, 1Eh FDP Events, 1Fh Namespace Admin Label, 20h Key Value Config, 21h Controller Data Queue, 22h Configurable Device Personality, 23h Power Limit, 24h Power Threshold, 25h Power Measurement, 26h Voltage Threshold, 27h Voltage Measurement, 28h Rate Limiting, 78h Embedded Management Controller Address, 79h Host Management Agent Address, 7Dh Enhanced Controller Metadata, 7Eh Controller Metadata, 7Fh Namespace Metadata, 80h Software Progress Marker, 81h Host Identifier, 82h Reservation Notification Mask, 83h Reservation Persistence, 84h Namespace Write Protection Config.
- Les features 0Ah (Write Atomicity), 0Bh (AEC) etc. sont cohérentes avec Get/Set Features ; la numérotation 13h+ sert aux features Set/Get Features plus récentes.
- Pour l'agent : seuls les features utilisés par le code (Number of Queues 07h, Arbitration 01h) comptent pour le pilote. Les autres sont informatifs.

## Lignes 2801-3100 (fig.32 fin : Boot Partition WP 85h ; notes ; §3.1.4 carte des propriétés)
- Boot Partition Write Protection Config = feature 85h. Notes fig.32 : note 2 (Set Features feature Power/Sanitize etc. PCIe), note 3/4 réservations, note 8 Telemetry/Firmware Commit/SMART Critical Warnings.
- §3.1.4 : propriété = dword ou qword ; accès à la largeur indiquée ; réservés en lecture seule à 0. Memory-based : registres PCIe (access via BAR0). Message-based : Property Get/Set.
- CARTE PCIe (fig.33/34) : CAP 0h (8 o), VS 8h, INTMS Ch, INTMC 10h, CC 14h, réservé 18h, CSTS 1Ch, NSSR 20h, AQA 24h, ASQ 28h (8 o), ACQ 30h (8 o), CMBLOC 38h, CMBSZ 3Ch, BPINFO 40h, BPRSEL 44h, BPMBL 48h (8 o), CMBMSC 50h (8 o), CMBSTS 58h, CMBEBS 5Ch, CMBSWTP 60h, NSSD 64h, CRTO 68h, PMRCAP E00h, PMRCTL E04h, PMRSTS E08h, PMREBS E0Ch, PMRSWTP E10h, PMRMSCL E14h, PMRMSCU E18h, PMR réservé E1Ch, doorbells à 1000h.
- Doorbell PCIe : offset 1000h + ((2y) × (4 << CAP.DSTRD)) (y = QID, 0 = admin).
- Message-based : 1000h-12FFh réservé Fabrics ; 1300h+ vendor.
- Pour l'agent : les offsets 0x00..0x28 doivent correspondre au code ; à vérifier au moment de la comparaison.

## Lignes 3101-3400 (§3.1.4 conventions ; CAP ; VS ; INTMS/INTMC ; CC début)
- CAP (fig.36, 64 bits, offset 0h) : NSSES bit 63? (réservé 63:62 puis NSSES bit 61 selon extraction ; à confirmer), CRMS 60:59, CRIMS bit 58, CRWMS bit 57, NSSS bit 56, CMBS bit 55? (extraction 55:52 et 51:48 contiennent MPSMAX/MPSMIN). Champs sûrs : MPSMAX 55:52, MPSMIN 51:48, CPS 47:46, BPS bit 45, CSS 44:37 (NCSS bit 37 ; IOCSS bit 43 ; NOIOCSS bit 44), NSSRS bit 36, DSTRD 35:32, TO 31:24 (unité 500 ms), reserved 23:19, AMS 18:17 (bit 17 = WRRUPC, bit 18 = VS), CQR bit 16, MQES 15:0 (0-based).
- NOTE : l'extraction a mélangé certaines positions de bits des champs 63:52 ; les positions 35:32, 31:24, 15:0 sont fiables. Vérifier CRMS/CRWMS/CRIMS et CMBS/PMRS dans le PDF original avant d'écrire le code qui les lit.
- VS (offset 8h) : MJR 31:16, MNR 15:8, TER 7:0 ; NVMe 2.4 => 2.4.0 (MJR=2, MNR=4, TER=0).
- INTMS (Ch) RWS ; INTMC (10h) RWC ; ces registres ne doivent pas être utilisés en MSI-X (accès non défini).
- CC (14h) : RO reserved 31:25 ; CRIME bit 24 (RW si CAP.CRMS=11b) ; IOCQES 23:20 ; IOSQES 19:16 ; plus SHN/AMS/MPS/CSS/EN (non lus encore).
- Règle : CC.AMS, CC.MPS, CC.CSS, IOCQES, IOSQES doivent être valides AVANT CC.EN=1 ; sinon Create IO queue échoue avec Invalid Queue Size.
- Point de contrôle agent : timeout d'attente CSTS.RDY = CAP.TO × 500 ms ; vérifier le code (si utilisé).

## Lignes 3401-3700 (CC complet ; CSTS)
- CC : IOCQES 23:20 (2^n octets) ; IOSQES 19:16 ; CRIME bit 24 ; SHN 15:14 (00 none, 01 normal, 10 abrupt) ; AMS 13:11 (000 RR, 001 WRR+Urgent) ; MPS 10:7 (taille page = 2^(12+MPS), modifiable seulement si EN=0) ; CSS 6:4 (000 NVM CS, 110 all I/O CS, 111 admin only) ; EN bit 0.
- EN 1->0 = Controller Reset : supprime toutes les I/O SQ/CQ, réinitialise les Admin queues, n'affecte PAS AQA/ASQ/ACQ ni l'état PCIe.
- AQA/ASQ/ACQ modifiables seulement avec CC.EN=0.
- CSTS (1Ch) : ST bit 6 ; PP bit 5 ; NSSRO bit 4 ; SHST 3:2 (00 normal, 01 en cours, 10 terminé) ; CFS bit 1 (fatal) ; RDY bit 0 (1 = prêt, à lire après EN=1 avant toute commande).
- NSSRO : bit 4, reflète NSSR.
- Point agent : après EN=1 attendre RDY=1 (borné par CAP.TO) avant de soumettre ; si CFS=1, pas de reprise sans reset.

## Lignes 3701-4000 (CSTS fin ; NSSR ; AQA ; ASQ ; ACQ ; CMBLOC/CMBSZ/CMBMSC/CMBSTS ; BPINFO/BPRSEL/BPMBL)
- NSSR (20h) : écrire 4E564D65h ("NVMe") déclenche NVM Subsystem Reset ; autre valeur = sans effet ; lecture = 0.
- AQA (24h) : ACQS bits 27:16 (0-based, 2..4096 entrées) ; ASQS bits 11:0 (0-based, 2..4096). Valeur 0 à l'enable = comportement indéfini. Non réinitialisé par Controller Reset.
- ASQ (28h) : ASQB bits 63:12 (adresse physique alignée sur la page CC.MPS) ; bits 11:0 réservés.
- ACQ (30h) : ACQB bits 63:12, aligné page ; toujours associée au vecteur d'interruption 0.
- CMBLOC (38h) : OFST 31:12, CQDA bit 9?, CDMMMS, CDPCILS, CDPMLS, CQPDS, CQMMS, BIR 2:0 (BAR 000b/010b/011b/100b/101b valides, alignée 4 KiB).
- CMBSZ (3Ch) : SZ 31:12, SZU 11:8 (0h=4 KiB, 1h=64 KiB, 2h=1 MiB, 3h=16 MiB, 4h=256 MiB, 5h=4 GiB, 6h=64 GiB), WDS, RDS, LISTS, CQS, SQS.
- BPINFO (40h) : ABPID bit 31, BRS 25:24, BPSZ 14:0 (en 128 KiB), 0 si non supporté.
- BPRSEL (44h) : BPID bit 31, BPROF 30:10 (en 4 KiB), BPRSZ 9:0 (en 4 KiB).
- BPMBL (48h) : base 63:12.
- CMBMSC (50h) : CBA 63:12, CMSE bit 1, CRE bit 0 (CMBSZ/CMBLOC activés seulement si CRE=1).
- CMBSTS (58h) : bit 0 (CBAI ou équivalent ; lecture 0b) — extraction partielle.
- Pour l'agent : aucune CMB/BP n'est utilisée actuellement ; à noter si l'on veut énumérer CMB.

## Lignes 4001-4300 (CMBSTS CBAI ; CMBEBS ; CMBSWTP ; NSSD ; CRTO ; PMRCAP/PMRCTL/PMRSTS)
- CMBSTS bit 1 = CBAI (base invalide). CMBEBS/CMBSWTP : informations facultatives (0 = non disponible).
- NSSD (64h) : écrire 4E726D6Ch ("Nrml") = NVM Subsystem Shutdown normal ; 41627074h ("Abpt") = abrupt ; autre = sans effet.
- CRTO (68h) : CRIMT bits 31:16 (unités 500 ms, valide si CRMS.CRIMS=1) ; CRWMT bits 15:0 (unités 500 ms, ≥ CRIMT).
- PMRCAP (E00h) : CMSS bit 24, PMRTO bits 23:16, PMRWBM 13:10, PMRTU 9:8, BIR 7:5 (010b/011b/100b/101b valides), WDS 4, RDS 3, bits 2:0 reserved? (extraction : BIR 7:5, WDS bit 4 ; RDS bit 3 ; réservé 2:0 ; à confirmer).
- PMRCTL (E04h) : EN bit 0.
- PMRSTS (E08h) : CBAI bit 12, HSTS 11:9 (000 normal, 001 restore error, 010 read-only, 011 unreliable), NRDY bit 8, ERR 7:0.
- Pour l'agent : PMR non utilisé. Les champs CRTO/CSTS sont utiles pour le délai d'attente RDY.

## Lignes 4301-4600 (PMR suite ; NSSC/NSSD ; §3.2 NVM Subsystem Entities : namespaces, NSID, NVM Sets, Endurance Groups, FDP début)
- PMRMSCL/PMRMSCU : CBA 52 bits, CMSE bit 1, CRE? (lecture seule). PMREBS/PMRSWTP : informations facultatives.
- §3.2.1 NSID : valide = 1..NN (0 invalide) ; FFFFFFFFh = broadcast. Un NSID alloué peut être actif ou inactif pour un contrôleur. NSID inactif dans commande -> Invalid Field in Command ; NSID invalide -> Invalid Namespace or Format.
- Identify CNS pour l'énumération : CNS 00h (namespace), CNS 02h (liste active, 1024 max, à répéter), CNS 10h (liste allouée, 1024 max). Identifier les NSIDs actifs en appelant Identify CNS 00h pour chaque NSID ou CNS 02h (méthode 2).
- NSIDs uniques dans le sous-système si gestion Namespace/ANA/NVM Set/Sanitize Namespace.
- §3.2.2 NVM Set : identifiant 16 bits, 0 réservé ; Identify CNS 04h = NVM Set List ; un namespace est dans un seul NVM Set. Si NVM Sets non supportés, le champ NVM Set Identifier est 0h.
- §3.2.3 Endurance Group : identifiant 16 bits, 0 réservé ; Endurance Group Information log page (5.2.13.1.10).
- §3.2.4 FDP : Reclaim Groups et Reclaim Unit Handles, uniquement si FDP activé.
- Pour l'agent : énumération namespace = Identify CNS 02h puis CNS 00h par NSID actif ; la boucle NSID 1..NN doit ignorer les NSIDs non alloués (structure vide).

## Lignes 4601-4900 (§3.2.4 FDP suite ; §3.2.5 Domaines ; §3.3 Queue models ; §3.3.1 PCIe : setup, usage, Empty/Full)
- §3.2.5 : un sous-système peut avoir plusieurs domaines ; Identify Controller Domain Identifier non nul si multi-domaine ; MDS bit CTRATT.
- §3.3.1.1 SÉQUENCE D'INITIALISATION (référence pour le code) :
  1. Configurer AQA, ASQ, ACQ ;
  2. Configurer CC.IOSQES et CC.IOCQES ;
  3. Set Features (Number of Queues, FID 07h) pour demander le nombre de SQ/CQ (le CQE indique le nombre alloué) ;
  4. Lire CAP.MQES et CAP.CQR ;
  5. Create I/O CQ (opcode 05h) ;
  6. Create I/O SQ (opcode 01h).
  Règle d'ordre : CQ créée AVANT les SQ associées ; SQ supprimée AVANT la CQ.
- Queue : Empty si Head == Tail ; Full si Head == Tail+1 (modulo) ; capacité utile = taille - 1.
- Doorbell écrit avec valeur invalide + AER en attente => AER avec "Invalid Doorbell Write Value" ; queue doit être supprimée puis recréée.
- Phase Tag : un CQE est nouveau quand le contrôleur inverse le bit P.
- Cancel : Multiple Command Cancel, NSID FFFFFFFFh pour annuler toutes les commandes d'une SQ (§7.1), ou Delete I/O SQ.
- Pour l'agent : vérifier que le code crée d'abord la CQ puis la SQ, et compte la capacité utile (size-1).

## Lignes 4901-5200 (§3.3.2 Fabrics queues, capsules, Connect, Disconnect, flow control SQHD ; §3.3.3 Queueing attributes ; §3.4 Command processing début)
- Fabrics : création de queues par Connect ; ASQ/ACQ/AQA non utilisés ; Create/Delete I/O CQ/SQ non supportés ; Disconnect pour supprimer une I/O queue.
- Taille de queue : minimum 2 emplacements ; max I/O = 65536 (limité par CAP.MQES) ; max Admin = 4096 ; un emplacement perdu (head/tail).
- QID : 1..65535 pour I/O ; 0 = admin.
- §3.3.3 : priorité de queue utilisée seulement si WRRUPC supporté (sinon ignorée).
- §3.4 : commandes soumises seulement quand CSTS.RDY=1 ET les queues I/O sont créées.
- §3.4.1 : pas d'ordre entre commandes sauf fused ; hôte doit gérer les dépendances LBA.
- §3.4.2 : Fused : bit FUSE de CDW0 (01b premier, 10b second), commandes adjacentes dans la même SQ, un seul doorbell. Erreur : Command Aborted due to Missing Fused Command.
- §3.4.4 : arbitration round-robin obligatoire ; CAP.AMS : WRRUPC optionnel. Arbitration Burst = commandes max par SQ par tour.
- Pour l'agent : aucun fused, donc ignorer ; l'ordre des 2 commandes admin de création d'IO queue doit être CQ puis SQ.

## Lignes 5201-5500 (WRR urgent arbitration ; §3.4.5 outstanding ; §3.5.1 INITIALISATION PCIe ; §3.5.2 Fabrics ; §3.5.3-3.5.4 ready modes)
- §3.5.1 SÉQUENCE D'INITIALISATION PCIe (référence contrôle code) :
  1. Attendre CSTS.RDY = 0 (reset précédent terminé) ;
  2. Configurer AQA, ASQ, ACQ ;
  3. Choisir CC.CSS : si CAP.CSS.NOIOCSS=1 -> 111b ; sinon si IOCSS=1 -> 110b ; sinon si NCSS=1 -> 000b ;
  4. Configurer CC.AMS et CC.MPS ;
  5. CC.EN = 1 ;
  6. Attendre CSTS.RDY = 1 ;
  7. Identify CNS 01h (Controller) ;
  8. Si IOCSS : Identify CNS 1Ch puis Set Features FID 19h (I/O CS Profile) ; pour chaque CS activé : Identify CNS 07h (active NSID list avec CSI), puis par NSID : CNS 00h (namespace NVM) + CNS spécifiques ;
  9. Set Features FID 07h (Number of Queues) pour obtenir le nombre ; configurer MSI/MSI-X ;
  10. Create I/O CQ (opcode 05h) ;
  11. Create I/O SQ (opcode 01h) ;
  12. Set Features (AEN config FID 0Bh) + AER (opcode 0Ch) possibles une fois RDY=1.
- Remarque : les étapes 9-12 et le Identify CNS 1Ch/07h sont des étapes que le code de l'agent doit suivre ou justifier leur absence.
- §3.5.3 : CAP.CRMS : CRWMS (bit 0 de CRMS) doit être 1 pour contrôleurs ≥2.0 ; CRIMS optionnel ; CC.CRIME lit/écrit seulement si CRMS=11b.
- §3.5.4 : timeouts CAP.TO (500 ms), CRTO.CRWMT/CRIMT ; Admin commands autorisées à retourner Admin Command Media Not Ready (fig.84 : Capacity Mgmt, Device Self-test, Firmware Commit/Download, Get LBA Status ; Get Log Page pour une liste de LIDs ; Namespace Attachment/Management, Format NVM, Sanitize…).
- Pour l'agent : les codes NVMe Admin Media Not Ready doivent être traités par re-essai (pas comme erreur fatale).

## Lignes 5501-5800 (§3.5.4 timeouts fin ; §3.6 Shutdown ; §3.6.1 PCIe shutdown ; §3.6.2 ; §3.6.3 NVM Subsystem Shutdown ; §3.7 Resets début)
- Figure 85 : état média selon CC.EN, CSTS.RDY, CSTS.SHST.
- §3.6.1 Shutdown normal PCIe : (1) si EN=1 : arrêter I/O, Delete I/O SQ (opcode 00h), Delete I/O CQ (opcode 04h) ; (2) CC.SHN = 01b ; attendre CSTS.SHST = 10b (et ST=0).
  Shutdown abrupt : CC.SHN = 10b, attendre SHST = 10b.
  Délai : RTD3 Entry Latency (Identify) ; si 0 -> attendre 1 s minimum.
- Après shutdown : redémarrer nécessite CLR (EN 1->0) ou écrire CC.EN=1 avec SHN=00b, puis init §3.5.
- §3.6.3 NVM Subsystem Shutdown : NSSD (64h) = "Nrml" (4E726D6Ch) normal ou "Abpt" (41627074h) abrupt ; CAP.NSSS requis ; NSSES change le comportement du Controller Reset ; délai NSS Latency Identify (0 -> 30 s).
- §3.7.1 NVM Subsystem Reset : déclenché par alimentation, écriture NSSR = 4E564D65h ("NVMe") (CAP.NSSRS), NVMe-MI, ou événement vendor ; CSTS.NSSRO indique la cause (alimentation).
- §3.7.2 Controller Level Reset (CLR) : déclenché par NSSR, Controller Reset (CC.EN 1->0), Cross-Controller Reset, reset transport ; efface toutes les SQ/CQ I/O, réinitialise Admin queues, CSTS.RDY -> 0 ; ne touche pas AQA/ASQ/ACQ (§3.7.4 sur Admin queue reset) ni CSTS.ST/SHST si shutdown en cours.
- Pour l'agent : après ExitBootServices/reset, RDY doit être 0 avant init (étape 1 de §3.5.1).

## Lignes 5801-6100 (§3.7.2 fin ; Controller Reset ; §3.7.3 queue reset I/O ; §3.7.4 Admin queue reset ; §3.8 capacité ; §3.9 Keep Alive début)
- CLR PCIe : Admin Queue : doorbells remis à 0 (SQ tail, CQ head) ; l'hôte DOIT mettre P=0 dans toutes les entrées de l'ACQ avant EN=1 (§4.2.4) -> l'agent doit effacer l'ACQ (zéro) avant d'activer le contrôleur.
- Après un Controller Reset (EN 1->0) : AQA/ASQ/ACQ et CMBMSC conservés (pas remis à zéro) ; PMR properties conservées.
- §3.7.3 : reset de queue I/O = Delete puis Create (ne pas oublier de supprimer les SQ avant la CQ).
- §3.8 : capacité NVM Sets/EG/domain ; MEGCAP, TNVMCAP, UNVMCAP ; Capacity Management (8.1.4) ; non nécessaires à l'agent.
- §3.9 Keep Alive : KATO (Set Features FID 0Fh) ; KATT = keep alive timeout ; l'hôte envoie Keep Alive (opcode 18h) à KATT/2 ; Keep Alive Timeout => nettoyage §3.9.5.
- Pour l'agent : Keep Alive n'est pas implémenté (acceptable, mais à noter : si KAS non nul et la fonctionnalité activée, le contrôleur peut déclencher un timeout après une longue pause sans envoyer Keep Alive).

## Lignes 6101-6400 (Keep Alive fin ; §3.10 Privileged ; §3.11 Firmware update ; §4.1 SQE : CDW0 / NSID / MPTR / DPTR / PRP ; §4.2 CQE début)
- §3.9.5 Keep Alive timeout : Error Log entry "Keep Alive Timeout Expired", arrêt, CSTS.CFS=1.
- §3.10 : commandes privilégiées (Set Features 09h,17h,22h,28h ; Namespace Mgmt 0Dh ; Attachment 15h ; Virt. 1Ch ; Capacity 20h ; etc.).
- §3.11 Firmware : Download (opcode 11h) -> Commit (opcode 10h, Commit Action 011b = activation sans reset, 010b = CLR, 001b/000b…) ; reset/CLR puis ré-initialisation §3.5.
- §4.1.1 SQE (64 octets) :
  CDW0 (octets 0-3) : CID bits 31:16 ; PSDT bits 15:14 (00 PRP, 01 SGL MPTR contig, 10 SGL segment) ; FUSE bits 9:8 ; OPC bits 7:0 (FN 7:2 , DTD 1:0 ; PCIe admin = PRP obligatoire).
  NSID (octets 4-7) ; CDW2/3 (8-15) ; MPTR (16-23) ; DPTR (24-39) = PRP1 (24-31), PRP2 (32-39) ; CDW10 (40-43) ; CDW11 (44-47) ; CDW12..15 (48-63).
  NSID FFFFFFFFh = broadcast (si commande le supporte).
  PRP2 : si transfert ≤1 page => réservé ; si 2 pages => page base ; si >2 pages => pointeur PRP list.
- §4.1.2 Fabrics SQE : OPC = 7Fh ; FCTYPE à l'octet 4 ; PSDT = 10b (SGL) pour données.
- §4.2.1 CQE 16 octets : DW0 cmd-specific ; DW1 réservé/cmd ; DW2 : SQID bits 31:16, SQHD bits 15:0 ; DW3 : Status 31:17 (dont bit 16 = P phase tag), CID 15:0. Phase Tag P = bit 16 de DW3 (vérifier au §4.2.4).
- Pour l'agent : vérifier que les offsets PRP1/PRP2 (24/32) et CDW10-15 (40..63) sont exacts dans main.rs. CDW0 : PSDT=0, FUSE=0.

## Lignes 6401-6700 (CQE DW3 ; Fabrics CQE ; §4.2.3 Status Field ; codes génériques ; codes spécifiques Admin)
- CQE DW3 : Status 31:17 ; P (phase tag) bit 16 ; CID 15:0. (Fabrics : P réservé.)
- Status Field (CQE bits 31:17 de DW3) : DNR bit 31 ; M bit 30 ; CRD 29:28 ; SCT 27:25 ; SC 24:17.
- SCT : 0h Generic ; 1h Command Specific ; 2h Media/Data integrity ; 3h Path related ; 7h Vendor.
- Generic (SCT=0) : 00h Success ; 01h Invalid Opcode ; 02h Invalid Field in Command ; 03h Command ID Conflict ; 04h Data Transfer Error ; 05h Power Loss Notification ; 06h Internal Error ; 07h Command Abort Requested ; 08h SQ Deletion abort ; 09h Failed Fused ; 0Ah Missing Fused ; 0Bh Invalid Namespace or Format ; 0Ch Command Sequence Error ; 0Dh Invalid SGL Segment Descriptor ; 0Eh Invalid Number of SGL Descriptors ; 0Fh Data SGL Length Invalid ; 10h Metadata SGL Length Invalid ; 11h SGL Descriptor Type Invalid ; 12h Invalid Use of CMB ; 13h PRP Offset Invalid ; 14h Atomic Write Unit Exceeded ; 15h Operation Denied ; 16h SGL Offset Invalid ; 18h Host Identifier Inconsistent ; 19h Keep Alive Timer Expired ; 1Ah Keep Alive Timeout Invalid ; 1Bh Aborted (Preempt and Abort) ; 1Ch Sanitize Failed ; 1Dh Sanitize In Progress ; 1Eh SGL Data Block Granularity Invalid ; 1Fh Command Not Supported for Queue in CMB ; 20h Namespace Write Protected ; 21h Command Interrupted ; 22h Transient Transport Error ; 23h Command Prohibited by Lockdown ; 24h Admin Command Media Not Ready ; 25h Invalid Key Tag ; 26h Host Dispersed NS Support Not Enabled ; 27h Host Identifier Not Initialized / Incorrect Key ; 28h FDP Disabled ; 29h Invalid Placement Handle List (lu 2Ah dans table : à vérifier) ; 2Bh Sanitize Namespace Failed ; 2Ch Sanitize Namespace In Progress ; 2Dh Failed to Restore Configuration ; 80h LBA Out of Range ; 81h Capacity Exceeded ; 82h Namespace Not Ready ; 83h Reservation Conflict ; 84h Format In Progress ; 85h-89h KV specifics.
- Command Specific (SCT=1) Admin : 00h Completion Queue Invalid ; 01h Invalid Queue Identifier ; 02h Invalid Queue Size ; 03h Abort Command Limit Exceeded ; 05h AER Limit Exceeded ; 06h Invalid Firmware Slot ; 07h Invalid Firmware Image ; 08h Invalid Interrupt Vector ; 09h Invalid Log Page ; 0Ah Invalid Format ; 0Bh Firmware Activation Requires Conventional Reset ; 0Ch Invalid Queue Deletion ; 0Dh Feature Identifier Not Saveable ; 0Eh Feature Not Changeable ; 0Fh Feature Not Namespace Specific ; 10h FW Activation Requires NVM Subsystem Reset ; 11h FW Activation Requires CLR ; 12h Maximum Time Violation ; 13h FW Activation Prohibited ; 14h Overlapping Range ; 15h Namespace Insufficient Capacity ; 16h Namespace Identifier Unavailable ; 18h Namespace Already Attached ; 19h Namespace Is Private ; 1Ah Namespace Not Attached ; 1Bh Thin Provisioning Not Supported ; 1Ch Controller List Invalid ; 1Dh Device Self-test In Progress ; 1Eh Boot Partition Write Prohibited ; 1Fh Invalid Controller Identifier ; 20h Invalid Secondary Controller State ; 21h Invalid Number of Controller Resources ; 22h Invalid Resource Identifier ; 23h-25h (PM / ANA Group / Attach).
- ATTENTION : la numérotation Command-Specific extraite est partielle (certaines valeurs sont décalées par l'extraction PDF) ; confirmer 00h/01h/02h (les seules utilisées par la création des queues) dans le PDF avant de coder.
- Pour l'agent : codes utiles = Create I/O CQ/SQ (00h, 01h, 02h), Admin Media Not Ready (24h, à réessayer), Namespace Not Ready (82h).

## Lignes 6701-7000 (fin codes Command Specific Admin ; codes I/O CS 80h-BFh ; codes Fabrics 80h-B0h ; Media/Data integrity 80h-89h)
- Command Specific Admin (suite) : 26h Insufficient Capacity ; 27h NS Attachment Limit ; 28h Prohibition not supported ; 29h I/O CS not supported ; 2Ah I/O CS not enabled ; 2Bh CS combination rejected ; 2Ch Invalid I/O CS ; 2Dh Identifier Unavailable ; 2Eh NS Is Dispersed ; 2Fh-44h (discovery, zoning, exported, cross-controller reset, etc.) ; 70h-7Fh Directive-specific ; 80h-BFh I/O CS ou Fabrics ; C0h-FFh vendor.
- I/O CS-specific (80h-BFh, NVM/ZNS) : 80h Conflicting Attributes ; 81h Invalid Protection Information ; 82h Attempted Write to Read-Only Range ; 83h Command Size Limit Exceeded ; 84h Invalid Command ID ; ... ; 9Ch Zone Invalid Write etc. (ZNS) ; liste complète à partir de fig.105.
- Fabrics-specific (80h-B0h) : 80h Incompatible Format ; 81h Controller Busy ; 82h Connect Invalid Parameters ; 83h Connect Restart Discovery ; 84h Connect Invalid Host ; 85h Invalid Queue Type ; 90h Discover Restart ; 91h Authentication Required ; B0h-BFh transport-specific.
- Media/Data integrity (SCT=2) : 80h Write Fault ; 81h Unrecovered Read Error ; 82h E2E Guard Check ; 83h E2E App Tag ; 84h E2E Ref Tag ; 85h Compare Failure ; 86h Access Denied ; 87h Deallocated or Unwritten Logical Block ; 88h E2E Storage Tag Check.
- Pour l'agent : seules les erreurs NVM Read/Identify/Create IO queues sont pertinentes. Access Denied (86h) et Unrecovered Read Error (81h) doivent être remontées clairement dans le rapport.

## Lignes 7001-7300 (Path related status ; §4.2.4 PHASE TAG ; §4.3.1 PRP entries et listes)
- Path Related (SCT=3) : 00h Internal Path Error ; 01h ANA Persistent Loss ; 02h ANA Inaccessible ; 03h ANA Transition ; 60h Controller Pathing Error ; 70h Host Pathing Error ; 71h Command Aborted By Host.
- §4.2.4 PHASE TAG (PCIe) : host initialise P=0 dans toute la CQ AVANT CC.EN=1 (Admin) ou avant Create I/O CQ (I/O). Le contrôleur écrit P=1 la première fois, inverse à chaque tour (wrap). Ce bit est la seule façon de voir qu'une entrée est nouvelle.
  Admin CQ : doit être remise à zéro à chaque reset Admin (§3.7.4).
- §4.3.1 PRP : entrée = adresse 64 bits page-alignée + offset ; offset doit être dword-aligné (bits 1:0 = 0). PRP1 peut avoir un offset non nul ; PRP2 et entrées suivantes doivent avoir offset = 0.
  Liste PRP : une page ; la dernière entrée peut pointer vers la page suivante de la liste.
- Pour l'agent : vérifier (a) que la CQ est mise à zéro avant création ; (b) que le phase-check utilise bit 16 de DW3 ; (c) que PRP2 et entrées de liste sont alignées.

## Lignes 7301-7600 (§4.3.1 fin PRP ; §4.3.2 SGL : descripteurs, identifiant, types, exemple)
- PRP : les entrées de liste sont minimales et compactées ; liste suivante page-alignée ; nombre d'entrées implicite par longueur/page.
- SGL : descripteur de 16 octets ; SGLID octet 15 : SGLDT bits 7:4 (0h Data Block, 1h Bit Bucket, 2h Segment, 3h Last Segment, 4h Keyed, 5h Transport, Fh vendor), SGLDST bits 3:0 (0h adresse, 1h offset...).
- Un descripteur tout à zéro = Data Block NULL.
- Segment/Last Segment : LEN non nul et multiple de 16 ; Data Block de longueur 0 valide.
- Pour l'agent : PCIe admin = PRP seulement (pas de SGL) ; le code ne doit pas construire de SGL pour l'admin.
- Pas de nouvelle action pour l'agent actuellement ; SGL utile seulement pour Fabrics.

## Lignes 7601-7900 (§4.3.3 MR ; §4.4 Feature Values : défaut/sauvegardé/courant, portée NSID ; §4.5 identifiants : VID/SSVID, SN/MN, OUI, EUI64)
- §4.4 Features : chaque feature a valeur par défaut / sauvegardée / courante si SSFS=1 (ONCS). Set Features met la valeur courante (ou sauvegardée si Save=1). Get Features NSID : feature de contrôleur -> NSID 0 ou FFFFFFFFh ; feature de namespace -> NSID actif, FFFFFFFFh seulement si MDS=0.
- Une feature non supportée -> Invalid Field in Command.
- §4.5.1 VID (octets 1:0) et SSVID (3:2) : petit-boutiste (little endian).
- §4.5.2 SN (octets 23:04) et MN (63:24) : ASCII, padding espaces.  => le numéro de série est une donnée sensible : le README de la clé interdit de l'afficher ; le code actuel l'affiche (point à corriger, en attente d'accord).
- §4.5.3 OUI (octets 75:73) little endian.
- §4.5.4 EUI64 (octets 120-127) big endian, format MA-L (OUI + extension).
- Pour l'agent : Identify Controller : VID@0, SSVID@2, SN@4..23, MN@24..63, FR@64..71, RAB@72, IEEE@73..75, CMIC@76, MDTS@77, CNTLID@78..79, ...  (à confirmer en §5.2.14 lors de la lecture de la section Identify).

## Lignes 7901-8200 (fin §4.5 : EUI64/NGUID ; §4.5.6 UUID ; §4.6 listes ; §4.7 NQN ; §4.8 UTF-8)
- NGUID octets 119:104 (big endian), format NAA=6 ; EUI64 octets 127:120 ; OUI MA-L.
- Liste de contrôleurs (4.6.1) : NUMCIDS sur 2 octets, puis CNTLIDs ; max 2047. Liste de NSIDs (4.6.2) : N×4 octets, triés croissant, 0 = vide.
- NQN (4.7) : chaîne UTF-8 terminée par NUL, max 223 octets ; format "nqn.2014-08.org.nvmexpress:uuid:..." ; SUBNQN identifie le sous-système (Identify Controller).
- Unicité namespace : EUI64 ou NGUID ou UUID ; UIDREUSE dans NSFEAT.
- §4.8 : chaînes UTF-8 traitées en binaire, sans normalisation côté contrôleur.
- Fin du chapitre 4. Prochaine étape : chapitre 5 Admin Command Set (Identify à partir de §5.2.14, Get Log Page §5.2.13, Create/Delete I/O queues §5.3.1-5.3.4, Set/Get Features §5.2.12/5.2.30, Abort §5.2.1, AER §5.2.2).

## Lignes 8201-8700 (§5 Admin Command Set : intro, fig.143 opcodes Admin complets, notes 1-11)
- §5 : Admin Command Set = commandes soumises à la SQ admin ; §5.1.2 commun ; §5.3 PCIe ; §5.4 Fabrics. Les commandes admin ne doivent pas être bloquées par l'état des I/O queues.
- Opcodes Admin (fig.143, sélection utile) : 00h Delete I/O SQ ; 01h Create I/O SQ ; 02h Get Log Page ; 04h Delete I/O CQ ; 05h Create I/O CQ ; 06h Identify ; 08h Abort ; 09h Set Features ; 0Ah Get Features ; 0Ch AER ; 0Dh Namespace Mgmt ; 10h Firmware Commit ; 11h Firmware Image Download ; 14h Device Self-test ; 15h Namespace Attachment ; 18h Keep Alive ; 19h Directive Send ; 1Ah Directive Receive ; 1Ch Virtualization Mgmt ; 1Dh NVMe-MI Send ; 1Eh NVMe-MI Receive ; 20h Capacity Mgmt ; 21h Discovery Info Mgmt ; 22h FZ Receive ; 24h Lockdown ; 25h FZ Lookup ; 28h Clear Exported NVM Resource ; 29h FZ Send ; 2Ah / 2Dh Manage Exported NVM Subsystem Receive/Send ; 31h Manage Exported Namespace ; 35h Manage Exported Port ; 38h Cross-Controller Reset ; 39h Send Discovery Log Page ; 3Dh/3Eh Track Send/Receive ; 41h/42h Migration Send/Receive ; 45h Controller Data Queue ; 7Ch Doorbell Buffer Config ; 7Fh Fabrics ; 80h Format NVM ; 81h Security Send ; 82h Security Receive ; 84h Sanitize ; 85h Load Program (CP) ; 86h Program Activation Mgmt (CP) ; 88h Get LBA Status ; 89h Memory Range Set Mgmt (CP) ; 8Ch Sanitize Namespace ; C0h-FFh vendor.
- Format NVM, Security, Sanitize, Load Program, etc. NSID : voir notes de la figure.
- Pour l'agent : les opcodes effectivement envoyés par le code (Identify 06h, Create CQ 05h, Create SQ 01h, Set/Get Features, Get Log Page 02h) doivent correspondre à ce tableau. (À vérifier dans main.rs.)

## Lignes 8701-9000 (§5.1 restrictions sanitize/format ; §5.2.1 Abort ; §5.2.2 AER : types, LID, AEI, codes)
- §5.1 : pendant sanitize/format, seules commandes listées sont admises (Abort, AER, Create/Delete I/O queues, Get Features, Identify, Get Log Page, etc.). Erreurs : Format In Progress, Command Sequence Error, Sanitize Namespace In Progress.
- §5.2.1 Abort : CDW10 = CID bits 31:16, SQID bits 15:0 ; CQE DW0 bit 0 = IANP (1 = pas d'abort immédiat) ; Abort Command Limit (Identify ACL) ; erreur 03h = Abort Command Limit Exceeded.
- §5.2.2 AER : CQE DW0 : LID bits 23:16, AEI bits 15:8, AET bits 2:0 (000 Error, 001 SMART/Health, 010 Notice, 011 Immediate, 100 One-shot, 110 I/O CS specific, 111 Vendor) ; DW1 = EVNTSP. Un événement se réarme par lecture du log (Get Log Page avec RAE=0). Reset => AER en attente abandonnées sans CQE. Erreur de limite : 05h.
- Error status (fig.153) : 00h Write to Invalid Doorbell ; 01h Invalid Doorbell Write Value ; 02h Diagnostic Failure ; 03h Persistent Internal Error ; 04h Transient Internal Error ; 05h Firmware Image Load Error.
- Pour l'agent : l'AER n'est pas utilisée (c'est une recommandation d'init). Les codes d'erreur ci-dessus permettent de décoder une notification si l'agent lance l'AER.

## Lignes 9001-9300 (AEI : Error/SMART/Notice/Immediate/One-shot ; §5.2.3 Capacity Management ; §5.2.4 Clear Exported)
- Notice (AET=010b) AEI : 00h Attached NS Attribute Changed (log 04h), 01h FW Activation Starting (log 03h), 02h Telemetry Log Changed (07h), 03h ANA change (0Ch), 04h Predictable Latency Event Aggregate, 05h LBA status alert, 06h EG Event Aggregate, 07h Reachability Group change, 08h Reachability Association change, 09h Allocated NS Attribute Changed (log 05h?), F0h-F6h discovery / cross-controller / lost host.
- One-shot (AET=100b) : 00h Reservation Log Page, 01h Sanitize Completed, 02h Sanitize Completed With Unexpected Deallocation, 03h Sanitize Entered Media Verification.
- Immediate (AET=011b) : 00h NVM Subsystem Normal Shutdown, 01h Temperature Threshold Hysteresis Recovery.
- SMART (AET=001b) : 00h NVM subsystem reliability, 01h Temperature threshold, 02h Spare below threshold.
- §5.2.3 Capacity Mgmt : CDW10 OPER bits 3:0 (0 select config, 1 create EG, 2 delete EG, 3 create NVM set, 4 delete NVM set, 5 restore default) ; ELID bits 31:16 ; CAPL DW11, CAPU DW12 ; CQE DW0 = CELID.
- Pour l'agent : hors périmètre (contrôle de capacité NVM Sets/EG non nécessaire pour l'agent).

## Lignes 9301-9600 (§5.2.5 Controller Data Queue ; §5.2.6 Device Self-test ; §5.2.7 Directive Receive ; §5.2.8 Directive Send début)
- CDQ : SEL bits 7:0 de CDW10 (0 create, 1 delete). Create : PRP1, CDW11 PC bit 0, CDW12 taille en dwords ; QT bits 7:0 de CDW10 MOS ; CQE DW0 = CDQID. Non utilisé par l'agent.
- Device Self-test (opcode 14h) : CDW10 STC bits 3:0 (1 short, 2 extended, 3 host-initiated refresh, E vendor, F abort) ; NSID 0 = contrôleur seul, FFFFFFFFh = tous les namespaces. Erreur 1Dh = Self-test in Progress.
- Directive Receive (1Ah) / Send (19h) : NUMD (0-based) en CDW10 ; CDW11 : DOPER 7:0, DTYPE 15:8, DSPEC 31:16 ; PRP/DPTR.
- Pour l'agent : pas d'utilisation (hors périmètre).

## Lignes 9601-9900 (§5.2.8 fin ; §5.2.9 Firmware Commit ; §5.2.10 Firmware Image Download ; §5.2.11 Format NVM)
- Firmware Commit (opcode 10h) CDW10 : BPID bit 31, CA bits 5:3 (000 remplace sans activer ; 001 activer au prochain CLR ; 010 activer l'image existante au prochain CLR ; 011 activer immédiatement ; 110 remplace Boot Partition ; 111 marque BP active), FS bits 2:0 (0 = contrôleur choisit, slot 1..7).
- Codes : 06h Invalid Firmware Slot ; 07h Invalid Firmware Image ; 0Bh Requires Conventional Reset ; 10h Requires NVM Subsystem Reset ; 11h Requires CLR ; 12h Max time violation ; 13h Activation Prohibited ; 14h Overlapping Range ; 1Eh BP write prohibited ; 3Dh Manufacturing default personality required.
- Firmware Image Download (opcode 11h) : NUMD (CDW10, 0-based, dwords), OFST (CDW11, en dwords) ; FWUG granularity ; code 14h Overlapping Range ; un Commit est requis avant nouveau download.
- Format NVM (opcode 80h) : CDW10 : LBAFU bits 13:12?/LBAFL (format index), MSET, PI, SES bits 11:9 (000 none, 001 user data erase, 010 crypto erase), FNS/SENS selon FNA ; NSID FFFFFFFFh = broadcast (sauf FNVMBS). Format détruit toutes les données : ne JAMAIS l'envoyer sans accord explicite.
- Pour l'agent : Firmware Commit/Download/Format ne doivent pas être exécutés (écriture firmware/données) sans accord ; à signaler dans le rapport comme interdits.

## Lignes 9901-10200 (Format NVM suite ; §5.2.12 Get Features : SEL, FID, CDW14 UIDX ; table des FIDs et renvois §5.2.30)
- Format NVM : SES 11:9 (001b user data erase, 010b crypto erase) ; LBAFL 3:0 ; code 0Ah = Invalid Format.
- Get Features (opcode 0Ah) : CDW10 : SEL bits 10:8 (000 current, 001 default, 010 saved, 011 supported capabilities), FID bits 7:0 ; CDW14 UIDX bits 6:0 ; DPTR utilisé selon FID.
- Table FIDs (confirmée) : 01h Arbitration (5.2.30.1.1), 02h Power Management, 04h Temperature Threshold, 06h Volatile Write Cache, 07h Number of Queues, 08h Interrupt Coalescing, 09h Interrupt Vector Config, 0Bh AEC, 0Ch APST, 0Dh HMB, 0Eh Timestamp, 0Fh Keep Alive Timer, 10h HCTM, 11h NOPPC, 12h RRL, 13h PLM Config, 14h PLM Window, 16h Host Behavior Support, 17h Sanitize Config, 18h EG Event Config, 19h I/O CS Profile, 1Ah Spinup Control, 1Bh Power Loss Signaling, 1Dh FDP, 1Eh FDP Events, 1Fh NS Admin Label, 21h CDQ, 22h CDP, 23h Power Limit, 24h Power Threshold, 25h Power Measurement, 26h Voltage Threshold, 27h Voltage Measurement, 78h EMC Address, 79h HMA Address.
- Pour l'agent : FIDs utilisés par l'init PCIe = 07h (Number of Queues) et 19h (I/O CS Profile) ; 0Bh pour AER si utilisé ; 0Fh pour Keep Alive.

## Lignes 10201-10500 (fin FIDs ; §5.2.12 Get Features suite : Select 3 = capacités, CQE DW0 ; §5.2.13 Get Log Page : CDW10-14, LID, scopes)
- Get Features Select=011b : CQE DW0 : CHANG bit 2, NSSPEC bit 1, SVBL bit 0. Code 1Fh Invalid Controller Identifier.
- Get Log Page (opcode 02h) : DPTR ; CDW10 : NUMDL bits 31:16 (0-based combiné avec NUMDU), RAE bit 15 (retain AE), LSP bits 14:8, LID bits 7:0 ; CDW11 : LSI 31:16, NUMDU 15:0 ; CDW12/13 : LPOL/LPOU (offset en octets, dword-aligné ; OT bit 23 de CDW14) ; CDW14 : CSI 31:24, OT bit 23, UIDX 6:0.
- Sur fig.209 (LIDs confirmées par le tableau du PDF) : 00h Supported Log Pages (Controller) ; 01h Error Information ; 02h SMART/Health (Controller/Namespace) ; 03h Firmware Slot ; 04h Changed Attached Namespace List ; 05h Commands Supported and Effects ; 06h Device Self-test ; 07h Telemetry Host-Initiated ; 08h Telemetry Controller-Initiated ; 09h Endurance Group Information ; 0Ah Predictable Latency Per NVM Set ; 0Bh Predictable Latency Event Aggregate ; 0Ch Asymmetric Namespace Access ; 0Dh Persistent Event Log ; 0Eh-... (NVM CS) ; 0Fh Endurance Group Event Aggregate ; 10h Media Unit Status ; 11h Supported Capacity Configuration List ; 12h Feature Identifiers Supported and Effects ; 13h NVMe-MI Commands Supported and Effects ; 14h Command and Feature Lockdown.
- CORRECTION : la liste des LIDs notée à partir de la figure 31 (plus haut) était décalée ; la table ci-dessus fait foi. À corriger dans les notes précédentes lors de la comparaison au code.
- Pour l'agent : SMART (02h) et Error Information (01h) sont les LIDs à lire (RAE=0), CDW10 : NUMDL = (dwords-1), LID = 02h.

## Lignes 10501-10800 (suite fig.209 : LIDs 15h-84h et notes)
- LIDs confirmées (table) : 15h Boot Partition ; 16h Rotational Media Information ; 17h Dispersed Namespace Participating NVM Subsystems ; 18h Management Address List ; 19h Physical Interface Receiver Eye Opening (renvoi transport) ; 1Ah Reachability Groups ; 1Bh Reachability Associations ; 1Ch Changed Allocated Namespace List ; 1Dh Device Personalities ; 1Eh Cross-Controller Reset ; 1Fh Lost Host Communication ; 20h FDP Configurations ; 21h Reclaim Unit Handle Usage ; 22h FDP Statistics ; 23h FDP Events ; 24h Manufacturer Default Configuration Status ; 25h Power Measurement ; 26h Reserved ; 27h Reserved? ; 28h Voltage Measurement ; 29h-6Fh réservés ; 70h Discovery ; 71h Host Discovery ; 72h AVE Discovery ; 73h Pull Model DDC Request ; 74h-7Eh réservés ; 7Fh Sanitize Namespace Status List ; 80h Reservation Notification ; 81h Sanitize Status ; 82h-BEh I/O CS ; BFh ZNS ; C0h-FFh vendor.
- CORRECTION par rapport aux notes de fig.31 : les LIDs 15h..84h ci-dessus remplacent la liste donnée plus haut ; l'ancienne liste était décalée (ex. 17h dispersed, pas 17h "Dispersed" ? à vérifier : table 1Dh device personalities et 1Eh cross-controller reset sont confirmés).
- Pour l'agent : aucune LID autre que 01h/02h/00h n'est lue par le code actuel.

## Lignes 10801-11100 (§5.2.13.1.1 Supported Log Pages ; §5.2.13.1.2 Error Information ; §5.2.13.1.3 SMART ; §5.2.13.1.4 FW Slot ; §5.2.13.1.5 Changed NS ; §5.2.13.1.6 Commands Supported début)
- LID 00h : 1024 entrées de 4 octets (LIDS 0..255) ; LSUPP bit 0, IOS bit 1, LIDSP bits 31:16.
- LID 01h Error Information : entrées de 64 octets, les plus récentes en premier ; ECNT 0..7, SQID 9:8? (octets 8-9), CID (10-11), STS (12-13 : P bit 0, STATUS bits 15:1), PEL (14-15), LBA (16-23), NSID (24-27), VSIA (28), TRTYPE (29), CSI (30), OPC (31), CSINFO (32-39), TTSI (40-41), LPVER (63 = 1h).
- LID 02h SMART/Health (512 octets) — offsets (confirmés par le tableau du PDF) : 0 Critical Warning ; 1-2 Composite Temperature (Kelvin) ; 3 Available Spare ; 4 Available Spare Threshold ; 5 Percentage Used ; 6 EGCWS ; 7 ? (Informative Warning/Vlt) ; 32-47 Data Units Read ; 48-63 Data Units Written ; 64-79 Host Read Commands ; 80-95 Host Write Commands ; 96-111 Controller Busy Time (minutes) ; 112-127 Power Cycles ; 128-143 Power On Hours ; 144-159 UNEXPECTED POWER LOSSES (anciennement "Unsafe Shutdowns") ; 160-175 Media and Data Integrity Errors ; 176-191 Number of Error Info Log Entries ; 192-195 WCTT ; 196-199 CCTT ; 200-239 Temperature sensors / TMT ; 240-243 IPM ; 244-511 réservé.
  => 1324 = compteur UPL (octets 144..159 en little endian) : c'est la valeur à citer dans le rapport.
- LID 03h FW Slot : AFI byte 0 (CAFS 2:0 ; NAFS 6:4) ; FRS1..FRS7 chacune 8 octets à partir de l'octet 8.
- LID 04h Changed NS list : liste de 1024 NSIDs max ; FFFFFFFFh en tête si dépassement.
- LID 05h Commands Supported and Effects : 4096 octets ; ACS0..255 (1024 premiers octets = admin) ; IOCS0..255 ensuite.
- Pour l'agent : vérifier les offsets SMART utilisés dans main.rs (UPL 144, Temp 1-2, spare 3, PUSED 5, DUR/DUW, CBT 96, POH 128) ; les dates/heures sont en minutes/heures.

## Lignes 11101-11400 (Commands Supported and Effects data structure ; §5.2.13.1.7 Device Self-test ; §5.2.13.1.8 Telemetry Host-Initiated début)
- Commands Supported & Effects (4 octets par opcode) : CSUPP bit 0, LBCC 1, NCC 2, NIC 3, CCC 4, CSER 15:14 (bits 15:14 de la structure), CSE 2:0 ... (voir fig.217 : CSE bits 2:0? ; CSP bits 11:? ; USS bit 19? ; à ne pas coder sans revérification).
- Device Self-test log (06h, 564 octets) : byte 0 CDSTO (DSTOS 3:0 ; 0 none, 1 short, 2 extended, 3 refresh, E vendor), byte 1 CDSTC (DSTCS 6:0 %), bytes 4..563 = 20 entrées de 28 octets (RDS1 = la plus récente) ; DSTS bits 7:4 DSTC, 3:0 DSTR (0 OK, 1 aborted by Device Self-test, 2 aborted by CLR, 3 aborted namespace removed, 4 Format NVM, 5 fatal, 6 failed segment unknown, 7 failed segment numbered, 8 unknown abort, 9 sanitize abort, Fh empty).
- Telemetry Host-Initiated (07h) : header 512 octets ; blocs de données 512 octets ; Log Specific Parameter CTHID bit 8, MCDA 14:12? ; Get Log Page doit être multiple de 512.
- Pour l'agent : non utilisé.

## Lignes 11401-11700 (Telemetry Host/Controller-Initiated ; EG Information 09h ; PL per NVM Set 0Ah ; PL Event Aggregate 0Bh ; ANA 0Ch début)
- Telemetry (07h/08h) : LID en octet 0 ; IEEE OUI ; THDA1..4LB ; THS / TCS (01h controller, 02h NVM subsystem) ; THDGN ; TCDA ; RID ; blocs de 512 octets. Non utilisé.
- EG Information (09h, 512 octets) : EGCW octet 0 ; EGFEAT octet 1 ; AVSP 2 ; AVSPT 3 ; PUSED 4 ; DID 5 ; EE 7:6 (?) ; DUR 63:48 ; DUW 79:64 ; MUW 95:80 ; HRC 111:96 ; HWC 127:112 ; MDIE 143:128 ; NEILE 159:144 ; TEGCAP 175:160 ; UEGCAP 191:176. LSI = ENDGID (CDW11 bits 15:0).
- PL per NVM Set (0Ah) et PL Event Aggregate (0Bh) : Predictable Latency Mode (obsolète en 2027). Non utilisé.
- ANA (0Ch) : CHGC 63:0, header 16 octets ; RGO bit 8 de LSP ; descripteurs ANA triés par ANAGRPID. Non utilisé.
- Pour l'agent : hors périmètre.

## Lignes 11701-12000 (ANA fin ; §5.2.13.1.14 Persistent Event Log : header, contexte, ACT, bitmap événements)
- ANA descriptor : AGID 3:0 ; NNV 7:4? (octets 15:8) ; CHGC (16-23 …) ; ANAS 4 bits (01 optimized, 02 non-optimized, 03 inaccessible, 04 persistent loss, 0Fh change). Non utilisé.
- Persistent Event Log (0Dh) : Action (LSP bits 9:8 : 00 read, 01 establish+read, 10 release, 11 establish + header 512 o) ; header : LID 0 ; TNEV 3:1? ; TLL 7:4 ; LREV ; LHL ; TSTMP ; POH ; PWRCC ; VID ; SSVID ; SN (bytes 23:04 ...) ; MN ; SUBNQN ; GNUM ; RCI ; bitmap SEB (bit 0 SHLSES, 1 FCES, 2 TCES, 3 PRES, 4 NSHEES, 5 CNES, 6 FNSES, 7 FNCES, 8 SSES, 9 SCES, 10 SFES, 11 TLCES, 12 TEES, 13 SMVES, 14 CDPCES, bit 222 = vendor). Non utilisé.
- Pour l'agent : les champs SN/MN sont répétés dans le log ; à traiter comme sensibles (voir README).

## Lignes 12001-12300 (Persistent Event : format d'événement, bits ECRH, types d'événements 01h-DFh, événements 01h-03h)
- Event header : ET (octet 0), ETR (1), EHL (2) ; EHAI (3), CNTLID (5:4), ETSTP (13:6), PELPID (15:14), VSIL (19:16?)…, EL (23:22) ; longueur totale = EL + EHL + 3.
- LID 0Dh spécifique : bit 0 ECRH (établir contexte + 512 o header) ; le log doit être supporté si NVMe 2.0+.
- Types d'événements (fig.236) : 01h SMART snapshot (M pour PCIe) ; 02h Firmware Commit (M) ; 03h Timestamp change (M) ; 04h Power-on/Reset (M) ; 05h NVM Subsystem HW error (M) ; 06h Change NS, 07h/08h Format NVM start/completion, 09h/0Ah Sanitize start/completion (M si commande) ; 0Bh Set Feature ; 0Ch Telemetry log created ; 0Dh Thermal excursion ; 0Eh Sanitize MV ; 0Fh CDP change ; 10h Exported Subsystem change ; DEh vendor ; DFh TCG.
- Pour l'agent : non utilisé. Évènement 05h (NSHE) pourrait servir de trace pour erreurs matérielles (à considérer plus tard).

## Lignes 12301-12600 (Persistent Event : TS change 03h ; Power-on/Reset 04h ; NVM Subsystem HW Error 05h (codes) ; Change NS 06h)
- 04h Power-on/Reset : FREV, RIL (liste de Controller Reset Information : CNTLID, FA, OIP bit 0 RDNF, CPWRC, POM, CTSTP).
- 05h NSHE (rev 02h) : NSHEEC bits 15:0 ; codes (fig.243) : 01h PCIe correctable ; 02h PCIe uncorrectable non fatal ; 03h PCIe uncorrectable fatal ; 04h PCIe link status change ; 05h PCIe link not active ; 06h critical warning condition ; 07h EG critical warning ; 08h Unexpected Power Loss (AHEI = UPL value at event) ; 09h CSTS.CFS set ; 0Ah Media and Data Integrity Status ; 0Bh Controller Ready Timeout Exceeded.
  Pour l'agent : la survenue d'une perte de puissance inattendue (code 08h) doit être lue dans UPL (SMART 144..159).
- 06h Change Namespace : NMCDW10, NSZE, NCAP, FLBAS, DPS, NMIC, ANAGRPID, NVMSETID, ENDGID.
- Pour l'agent : non utilisé.

## Lignes 12601-12900 (fin Change NS ; 07h/08h Format NVM ; 09h/0Ah Sanitize ; 0Bh Set Feature ; table PE logging des features)
- Format NVM Start (07h, rev 01) : NSID, FNA, FMCDW10. Format NVM Completion (08h, rev 02) : NSID, SFPI, FNVMS (INCPLTF bit 1, FNVME bit 0), CINFO, INFO (STATUS 15:1, P bit 0).
- Sanitize Start (09h, rev 02) : SANICAP, SCDW10, SCDW11, NSID (FFFFFFFFh si NVM subsystem). Sanitize Completion (0Ah, rev 02) : SPROG, SSTAT, CINFO, NSID.
- Set Feature event (0Bh, rev 01) : déclenché par Set Features réussi pour FIDs listés.
- Pour l'agent : aucun Format/Sanitize ne doit être lancé (voir interdiction de l'écriture disque).

## Lignes 12901-13200 (table PE logging des features suite ; fin de la table 5.2.13.1.14)
- Table de journalisation PE des features (I/O / Admin / Discovery) : entrée par FID : 0Ah Arbitration/WRR..., 0Bh AEC (NR), 0Ch APST, 0Dh HMB, 0Eh Timestamp (P), 0Fh KA timer, 10h HCTM, 11h NOPPC, 12h RRL, 13h PLM config, 14h PLM window, 15h LBA SI interval, 16h Host Behavior, 17h Sanitize Config, 18h EG event cfg, 19h I/O CS profile, 1Ah Spinup, 1Bh PLS config, 1Dh FDP, 1Eh FDP events, 1Fh NS Admin Label, 20h KV config, 21h CDQ, 22h CDP, 23h Power Limit, 24h Power Threshold, 25h Power Measurement, 26h Voltage Threshold, 27h Voltage Measurement, 78h EMC address, 79h HMA address, 7Dh-7Fh metadata, 80h SW progress marker (NR), 81h Host Identifier, 82h RNM, 83h Reservation Persistence, 84h NS write protection, 85h BP write protection.
- NR = non recorded (non enregistré) ; P = prohibited ; O = optional.
- Pour l'agent : rien à faire.

## Lignes 13201-13500 (Set Feature event 0Bh : format ; Telemetry Created 0Ch ; Thermal Excursion 0Dh ; Sanitize MV 0Eh ; CDP change 0Fh ; Exported change 10h ; début Vendor 0DEh)
- Set Feature Event Data : SFEL bits 31:16 (MBC) / 2:0 (DWC) ; LCCDW0 bit 3 ; CDWS ; MBUF ; CCDW0.
- Telemetry Log Created (0Ch) : 512 octets = copie de l'en-tête du log télémétrie.
- Thermal Excursion (0Dh) : OTMP (octet 0), THRESH (octet 1, valeurs 1h..89h : WCTEMP, CCTEMP, TMT1, TMT2, self-throttle, normal, under-temp).
- Sanitize Media Verification (0Eh) : pas de données.
- CDP Change (0Fh) : PS octet 0 (CDPRFS bit 1, CDPCE bit 0), PERID octets 11:2 ? (extraction), PED variable.
- Exported NVM Subsystem Change (10h) : ESUBID 1:0, OTYP octet 2 (00h..09h), EEID, UEID.
- Pour l'agent : rien à faire.

## Lignes 13501-13800 (Vendor Specific event DEh ; TCG DFh ; §5.2.13.1.15 EG Event Aggregate 0Fh ; §5.2.13.1.16 Media Unit Status 10h ; §5.2.13.1.17 Supported Capacity Config 11h)
- Vendor event : VSEC/VSEDT/UIDX/VSEDL ; types VSEDT 01h name ASCII, 02h ASCII, 03h binary, 04h signed int 64.
- TCG DFh : réservé TCG.
- EG Event Aggregate 0Fh : NUMENT (1..), liste d'EGID triée.
- Media Unit Status 10h : NMU (1..), CCHANS (ouverture), SELC ; descripteur : MUID, DID, ENDGID, NVMSETID, CAF, AVSP, PUSED, MUCS, CIO...
- Supported Capacity Configuration List 11h : SCCN ; descripteur : CCID, DID, EGCN ; EG config : ENDGID, CADJF, TEGCAP, SEGCAP, EE, EGSETS...
- Pour l'agent : hors périmètre (capacité/endurance).

## Lignes 13801-14100 (Supported Capacity suite ; §5.2.13.1.18 FID Supported & Effects 12h ; §5.2.13.1.19 NVMe-MI 13h ; §5.2.13.1.20 Lockdown 14h)
- Capacity config descriptors : CHID, CHMUS, MUID, MUDL (non utilisé).
- 12h FID Supported & Effects : 4 octets par FID (256 entrées) ; FSUPP bit 0, FSP bits 19:0? (scope), USS, etc.
- 13h NVMe-MI commands : idem par opcode MI ; CSUPP bit 0.
- 14h Lockdown : LSP bit 14 ELPF, bits 13:12 CNTTS, 11:8 SCP ; Log Specific Identifier = CNTLID.
- Pour l'agent : FID 07h (Number of Queues) supporté ? à vérifier dans la table 12h si le code la lit.

## Lignes 14401-14600 (Rotational media fin ; 17h Dispersed NS participating NQNs ; 18h Management Address List ; 1Ah Reachability Groups ; 1Bh Reachability Associations début)
- 16h : SPINC, FSPINC, LDC, FLDC (media rotatif, non utilisé).
- 17h : GENCTR, NUMPSUB, liste NQN de 256 o chacune ; NSID 0/FFFFFFFFh -> Invalid Namespace ; non-dispersed -> Invalid Field.
- 18h : Management Address List, 8 descripteurs max (MAT, MADRS URI UTF-8).
- 1Ah / 1Bh : Reachability Groups / Associations : entêtes CHNGC + NRGD/NRAD, RGO/RAO bits 8 ; non utilisé.
- Pour l'agent : hors périmètre.

## Lignes 14601-14800 (Reachability Association suite ; 1Ch Changed Allocated NS ; 1Dh Device Personalities ; 20h FDP Configurations ; Placement Identifier)
- 1Ch : liste de NSIDs (max 1024, FFFFFFFFh en tête si dépassement).
- 1Dh Device Personalities : NUMP (0-based), CDPLPV, DPLPHL=6, CDPLPS ; descripteur PPS=5 : MRSTT, AUS, PKAS/PCAS, PSCUDE.
- 20h FDP Configurations : NUMFDPC (0-based), VER, SZE ; FDP config : DSZE, FDPA (FDPCV bit 7, FDPVWC bit 4, RGIF 3:0?), VSS, NRG, NRUH, MAXPIDS, NNS, RUNS, ERUTL ; RUH descriptor : RUHT (1h initially isolated, 2h persistently isolated).
- Placement Identifier : PHNDL bits 15:0 (sans RGIF) ; avec RGIF : bits du RG en haut.
- Pour l'agent : FDP/CDP hors périmètre.

## Lignes 14801-15000 (Placement Id avec RGIF ; 21h RUH Usage ; 22h FDP Statistics ; 23h FDP Events : format)
- 21h RUH Usage : NRUH, RUHA (1h Host specified, 2h controller specified, au plus une entrée 2h).
- 22h FDP Statistics : HBMW, MBMW, MBE (16 o chacun) ; reset quand FDP change.
- 23h FDP Events : FDPET bit 8 de LSP ; header NUMFDPE ; événements de 32 o : ETYP (0h no RU fully written, 1h RU time limit, 2h CLR modified RUH, 3h invalid PID, 80h media reallocated, 81h implicit modified RUH), FDPEF, PID, ETMSP, NSID, RGICD.
- Pour l'agent : hors périmètre.

## Lignes 15001-15200 (FDP events suite ; 24h Manufacturer Default ; 25h Power Measurement)
- FDP events 0h..3h (host) et 80h/81h (contrôleur) : sémantique ; LV/NSIDV/PIV bits.
- 24h MDCS : bit 2 DCCS, bit 1 DNCS, bit 0 DSCS.
- 25h Power Measurement : PMGN, PMA (PMT, PHDO, MIPWRTS, EPF, NCPDF, PME bit 0), SZE, PMC, NPHD, SMTR, SMTS, PHDS, PHBS (=250 mW), NPHDS, VSS, PHDOC, AIPWR, MIPWR, MIPWRT, IPWRPE, histogramme. Non utilisé.
- Pour l'agent : hors périmètre.

## Lignes 15201-15400 (Power Histogram ; 27h Voltage Measurement ; 7Fh Sanitize Namespace Status List début)
- Histogramme : PHBC (count), PHBLT (lower threshold, PWRS/PWRV) ; bins de 250 mW.
- 27h Voltage Measurement : VMGN, VMA (VMC bit 4, VSM 3:2, IVOLTS bit 1, VME bit 0), VSI (VSSS, VSSL, VSSV en 0.05 V), VMLSZ, NVMDS, NVMDE, SVMTR, SVMTS, CNTLID, VLVSS ; OVOL/OVC/OVLT/OVTS/IOVPE ; UVOL/UVC/UVLT/UVTS/IUVPE ; descripteurs IVOLM (VOLV 13:0, NCVM bit 14). Non utilisé.
- 7Fh Sanitize Namespace Status List : NLT (LSP bit 8) : in progress / completed.
- Pour l'agent : hors périmètre.

## Lignes 15401-15600 (7Fh suite ; 80h Reservation Notification ; 81h Sanitize Status : SPROG, SSTAT, SOS, ETO/ETBE/ETCE/ETODMM)
- 7Fh : GENCTR, NUMNSID, liste NSID ; Controller Pathing Error si données invalides.
- 80h Reservation Notification : LPC 63:0 ; RNLPT (1h Registration Preempted, 2h Reservation Released, 3h Reservation Preempted) ; NALP ; NSID.
- 81h Sanitize Status (512 o) : SPROG (bytes 1:0, fraction /65536, FFFFh si non applicable), SSTAT (bytes 3:2 : PRGD bit 15, NDE bit 10, MVCNCLD bit 9, GDE bit 8, OPC 7:3 bytes, SOS 2:0 : 000 never, 001 sanitized, 010 sanitizing, 011 failed, 100 unexpected deallocate), SCDW10 (bytes 7:4?) ; ETO/ETBE/ETCE/ETODMM (secondes).
- Pour l'agent : pas de sanitize ; log lu seulement si nécessaire. Note : un sanitize/format ne doit jamais être lancé sans accord.

## Lignes 15601-15800 (81h Sanitize Status suite : SSI, SANS, MNSOIP, STNSID ; Memory-based : aucun LID ; 1Eh Cross-Controller Reset début)
- SSI (octet 36) : FAILS bits 7:4, SANS bits 3:0 (0 Idle, 1 Restricted Processing, 2 Restricted Failure, 3 Unrestricted Processing, 4 Unrestricted Failure, 5 Media Verification, 6 Post-Verification Deallocation).
- PCIe : aucun LID spécifique au transport mémoire (confirmé).
- 1Eh Cross-Controller Reset : NE 15:0 ; entrée 16 o ? : ICID, CIU, ACID, CCRS (00h In progress, 01h success, 02h failed), CCRF (RETRY bits 3:2, CLRI bit 1 ?) ; RMC bit 8 de LSP. Fabrics seulement.
- Pour l'agent : hors périmètre (PCIe).

## Lignes 15801-16000 (1Eh suite V/CIU/CIRN ; 1Fh Lost Host Communication ; 70h Discovery : LSP, types de transport, adresse famille, subtype, TREQ)
- 1Eh : bit V (validated), CCRS, CCRF ; CIU et CIRN doivent correspondre à Identify.
- 1Fh : NE ; entrée : CNTLID, LC (count), CIU ; max 511 entrées ; effacé par RAE=0 ou CLR.
- 70h Discovery (Discovery controllers seulement) : LSP bits 8..14 (EXTDLPE bit 8, PLEO bit 9, ALLSUBE bit 10) ; LID-specific bits ALLSUBES bit 2, PLEOS bit 1, EXTDLPES bit 0.
  Entrée Discovery (1024 o ? : TRTYPE 0 (1 RDMA, 2 FC, 3 TCP, 254 intra-host), ADRFAM 0 (1 IPv4, 2 IPv6, 3 IB, 4 FC), SUBTYPE 0 (1 referral, 2 NVM subsystem, 3 current discovery), TREQ (TASC 5:4, ZHIDS bit 3, SQFCD bit 2, TSC 1:0), PORTID, CNTLID, ASQSZ, DUPRETINFO, ...).
- Pour l'agent : Discovery = Fabrics uniquement, hors PCIe.

## Lignes 16001-16200 (70h Discovery entrée : CNTLID, ASQSZ (min 32), EFLAGS, TRSVCID, SUBNQN, TRADDR, TSAS ; Extended ; 70h page ; 71h Host Discovery ; 72h AVE début)
- Entrée Discovery 1024 o (TEL=1024 hors extended) : CNTLID bytes 7:6? ; ASQSZ 9:8? ; EFLAGS 11:10 (NCC bit 2, EPCSD bit 1, DUPRETINFO bit 0) ; TRSVCID 63:32 ; SUBNQN 255:64 ; TRADDR 511:256 ; TSAS 767:512 ; octets 1023:768 réservés.
- Discovery Log Page header : GENCTR, NUMREC, RECFMT, DLPF (ALLSUB bit 2, PORTLCL bit 1, EXTEND bit 0), TDLPL.
- 71h Host Discovery : entrée host (HOSTNQN, TRADDR, TSAS, TEL, NUMEXAT >=1 avec Host ID) ; HDLPF ALLHOST bit 0 ; LSP ALLHOSTE bit 8.
- 72h AVE : entête GENCTR, NUMREC, RECFMT, TADLPL ; entrées AVE.
- Pour l'agent : Discovery/Host/AVE = Fabrics, hors périmètre.

## Lignes 16201-16400 (fin 72h AVE ; 73h Pull Model DDC ; §5.2.14 IDENTIFY : vue d'ensemble, CDW10/11/14, CNS table)
- 72h AVE entrée : TEL, AVENQN (224 o), NUMATR ; AVE transport record : AVEADRFAM, AVETRSVCID, AVETRADDR (IPv4/IPv6). Hors périmètre.
- 73h : ORI (1 GAZ, 2 AAZ, 3 RAZ, 4 DLPR), TPDRPL. Hors périmètre.
- 331 : Get Log Page status : 09h Invalid Log Page ; 1Fh Invalid Controller ID ; 29h I/O CS not supported.
- IDENTIFY (opcode 06h) :
  DPTR : PRP1 (4096 o, pas de liste PRP) ; CDW10 bits 31:16 CNTID (controller identifier), bits 7:0 CNS ; CDW11 bits 31:24 CSI, 15:0 CNSSID ; CDW14 bits 6:0 UIDX.
  CNS (fig.336) : 00h Identify Namespace (NSID) ; 01h Identify Controller ; 02h Active Namespace ID list ; 03h Namespace Identification Descriptor list (NSID) ; 04h NVM Set list (NVMSETID) ; 05h I/O CS Namespace (NVM CS) ; 06h I/O CS Controller ; 07h I/O CS Active NS list ; 08h I/O CS independent NS ; 09h ... ; 10h Allocated NSID list ; 11h Allocated NS (NSID).
  Controller non supporté -> Invalid Field in Command.
- Pour l'agent : Identify CNS 01h (contrôleur, 4096 o) et CNS 02h (liste NSIDs) sont les seuls requis par l'init. Vérifier dans main.rs.

## Lignes 16401-16600 (fin table CNS : 0Eh..22h ; CNS 14h Primary Controller Caps, 15h Secondary list, 16h NS Granularity, 17h UUID list, 18h Domain list, 19h EG list, 1Ah/1Bh I/O CS allocated, 1Ch I/O CS list, 1Dh Underlying NS, 1Eh Ports, 1Fh indep allocated NS, 20h Controller state formats, 21h Underlying Controller List, 22h Exported UUID)
- CNS table (sélection utile) : 00h NS ; 01h Controller ; 02h Active NSID ; 03h NS ID descriptors ; 04h NVM Set List ; 05h I/O CS NS ; 06h I/O CS Controller ; 07h I/O CS active NSID ; 08h I/O CS indep NS ; 10h allocated NSID ; 11h allocated NS ; 14h primary controller caps ; 15h secondary list ; 16h NS granularity ; 17h UUID list ; 18h Domain list ; 19h EG list ; 1Ch I/O CS list ; 1Dh underlying NS ; 1Eh ports list ; 1Fh indep allocated NS ; 20h controller state formats ; 21h underlying controllers ; 22h exported UUID template.
- Pour l'agent : seuls CNS 01h, 02h, 00h, 03h sont pertinents pour le PCIe de base.

## Lignes 16601-16800 (notes table CNS ; CSI table fig.337 ; 5.2.14.2.1 IDENTIFY CONTROLLER début)
- CSI : 00h NVM CS, 01h Key Value, 02h ZNS, 03h Subsystem Local Memory, 04h Computational Programs, 05h-2Fh réservé, 30h-3Fh réservé, 40h-FFh vendor.
- Identify Controller (CNS 01h, 4096 o), offsets confirmés (fig.338) :
  VID 01:00 (M, little endian) ; SSVID 03:02 ; SN 23:04 (ASCII, espaces) ; MN 63:24 ; FR 71:64 (ASCII) ; RAB 72 (2^n commandes) ; IEEE 75:73 (OUI) ; (suite à lire : CMIC, MDTS, CNTLID, VER, ...).
- Pour l'agent : la comparaison des offsets dans main.rs (SN/MN/FR) est à faire.

## Lignes 16801-17000 (Identify Controller suite : CMIC 76 ; MDTS 77 ; CNTLID 79:78 ; VER 83:80 ; RTD3R 87:84 ; RTD3E 91:88 ; OAES 95:92 ; CTRATT 99:96 début)
- CMIC (octet 76) : ANARS bit 3, FT bit 2, MCTRS bit 1, MPORTS bit 0.
- MDTS (octet 77) : 2^n × CAP.MPSMIN (0 = pas de limite). Non vérifié par main.rs.
- CNTLID octets 79:78 ; VER octets 83:80 (doit être non nul) ; RTD3R 87:84 ; RTD3E 91:88 (µs ; 0 = non reporté → attendre 1 s) ; OAES 95:92 (DLPCN bit 31, ZDCN bit 27, RLCC 22, LHCN 21 ...).
- CTRATT (octets 99:96) : IIELLSS 24:23 ; bits suivants (MDS, MEM, NVM sets, endurance groups…).
- Pour l'agent : VER / CNTLID / MDTS / CMIC sont informatifs. Ne pas écrire de valeur par défaut sans lecture.

## Lignes 17001-17200 (CTRATT suite : IIELLSS, VMS 22, PMS 21, PLS 20, FDPS 19, RHII 18, HMBR 17, MEM 16, ELBAS 15, DNVMS 14, DEG 13, VCM 12, FCM 11, MDS 10, ULIST 9, SQA 8, NG 7, TBKAS 6, PLM 5, EGS 4, RRLVLS 3, NSETS 2, NOPSPM 1, HIDS 0 ; RRLS 101:100)
- CTRATT (octets 99:96) : confirmé. MDS bit 10 : multi-domaine. HIDS bit 0 (128-bit Host ID). TBKAS bit 6 (keep alive trafic).
- RRLS octets 101:100 : RRL0..15.
- Pour l'agent : l'octet CTRATT est informatif mais utile pour journaliser le type de contrôleur.

## Lignes 17201-17400 (Identify Controller : BPCAP 102 ; NSSL 107:104 ; PLSI 109:108 ; CNTRLTYPE 111 ; FGUID 127:112 ; CRDT1/2/3 ; CRCAP 129:128 début)
- BPCAP octet 102 (bit 1 SFBPWPS? ; RPMBBPWPS bits 1:0 de l'octet 102 selon extraction ; à confirmer).
- CHSI octet 103 (CHS bit 0).
- NSSL octets 107:104 (µs, 0 = non reporté → attendre 30 s).
- PLSI octets 109:108 (PLSFQ bit 1, PLSEPF bit 0).
- CNTRLTYPE octet 111 : 0 non rapporté, 1 I/O, 2 Discovery, 3 Admin.
- FGUID octets 127:112 (big endian, EUI-64).
- CRDT1 octets 129:128? ; CRDT2 131:130 ; CRDT3 133:132 (unités de 100 ms).
- Pour l'agent : CNTRLTYPE doit valoir 1 pour un contrôleur I/O. À vérifier.

## Lignes 17401-17600 (CRCAP ; CIU 135? ; CIRN ; NVMSR 253 ; VWCI 254 ; MEC 255 ; OACS 257:256 : bits CCFLS 13, ENSS 12, HMLMS 11, CFLS 10, GLSS 9, DBCS 8, VMS 7, NSRS 6, ...)
- CIU : octet 134 (persiste aux resets, non nul). CIRN : octets 143:136 (non nul, change à chaque CLR sauf power).
- NVMSR (253) : NVMEE bit 1, NVMESD bit 0. VWCI (254). MEC (255) : PCIEME bit 1, TWPME bit 0.
- OACS (257:256) : Security bit 0 ?, Format NVM bit 1, Firmware bit 2, NS Mgmt bit 3, Self-test bit 4, Directives bit 5, NVMe-MI bit 6 (NSRS), Virt bit 7 (VMS), DBCS bit 8, GLSS bit 9, CFLS bit 10, HMLMS bit 11, ENSS 12, CCFLS 13 (ces numéros sont confirmés par les descriptions du PDF, la correspondance exacte OACS pour bits 0-5 est à confirmer dans la section 3.1.3 / fig.338 complet).
- Pour l'agent : OACS sert à savoir si Format/Firmware sont supportés (ne pas les utiliser de toute façon).

## Lignes 17601-17800 (OACS bits 5:0 ; ACL octet 258 ; AERL 259 ; FRMW 260 ; LPA 261 ; ELPE 263:262 ; NPSS 264 ; AVSCC 265 ; APSTA 266)
- OACS bits 5:0 (confirmés) : DIRS bit 5, DSTS bit 4, NMS bit 3, FWDS bit 2, FNVMS bit 1, SSRS bit 0.
- ACL octet 258 (0-based, min 4 recommandé) ; AERL octet 259 (0-based, min 4).
- FRMW octet 260 : SMUD bit 5, FAWR bit 4, NOFS bits 3:1 (1..7), FFSRO bit 0.
- LPA octet 261 : DA4S bit 6, MLPS bit 5, PES bit 4, TS bit 3, LPEDS bit 2, CSES bit 1, SMARTS bit 0.
- ELPE octets 263:262 (0-based) ; NPSS octet 264 (0-based) ; AVSCC octet 265 (VSCF bit 0) ; APSTA octet 266 (APTS bit 0).
- Pour l'agent : SMARTS=1 requis pour lire SMART (LID 02h). Vérifier que le code ne suppose pas un LID absent.

## Lignes 17801-18000 (Identify Controller : WCTEMP 269:268 (rec 0157h) ; CCTEMP 271:270 ; MTFA 275:272 ; HMPRE 279:276 ; HMMIN 283:280 ; TNVMCAP 295:280?/296 ; UNVMCAP 311:296 ; RPMBS 315:312 ; DSTO 318 ; FWUG 319 ; KAS 321:320? ; HCTMA 323:322 ; MNTMT 325:324 ; MXTMT 327:326 ; SANICAP 331:328)
- Table octets (confirmé) : WCTEMP 269:268 ; CCTEMP 271:270 ; MTFA 275:272 (100 ms) ; HMPRE 279:276 (4 KiB) ; HMMIN 283:280 ; TNVMCAP 295:284?/296 (extraction ; 16 o) ; UNVMCAP 311:296 (16 o) ; RPMBS 315:312 ; EDSTT 317:316? (1 min) ; ... DSTO 318 ; FWUG 319 (4 KiB) ; KAS 321:320 (100 ms) ; HCTMA 323:322 ; MNTMT 325:324 ; MXTMT 327:326 ; (331:328 SANICAP).
- Pour l'agent : FWUG/KAS/WCTEMP/CCTEMP informatifs. Ne rien écrire.

## Lignes 18001-18200 (SANICAP 331:328 ; NODMMAS bits 31:30 ; NDI 29 ; SPRRS 5 ; NVERS 4 ; VERS 3 ; OWS 2 ; BES 1 ; CES 0 ; HMMINDS 335:332 ; HMMAXD 337:336 ; NSETIDMAX 339:338 ; ENDGIDMAX 341:340 ; ANATT 342 ; ANACAP 343 ; ANAGRPMAX 347:344)
- SANICAP (octets 331:328) : NODMMAS 31:30, NDI 29, SPRRS 5, NVERS 4, VERS 3, OWS 2, BES 1, CES 0.
- HMMINDS 335:332 (4 KiB) ; HMMAXD 337:336 ; NSETIDMAX 339:338 ; ENDGIDMAX 341:340 ; ANATT 342 (s) ; ANACAP 343 (ANAGIDS 7, ANAGIDLWAS 6, RANACS 4, RANAPLS 3, RANAIS 2, RANANOS 1, RANAOS 0) ; ANAGRPMAX 347:344.
- Pour l'agent : SANICAP sert seulement à savoir si Sanitize existe ; ne pas exécuter.

## Lignes 18201-18400 (ANAGRPMAX suite ; NANAGRPID 351:348 ; PELS 355:352 ; DID 357:356 ; KPIOC 358 ; MPTFAWR 361:360 ; RMDCA 362 ; MEGCAP 383:368? ; TMPTHHA 384 ; MUPA 385 ; CQT 387:386 ; CDPA 388 ; CDPALG 389:388)
- NANAGRPID 351:348 ; PELS 355:352 (64 KiB) ; DID 357:356 (non nul si MDS) ; KPIOC 358 (KPIOS bit 0) ; MPTFAWR 361:360 (100 ms).
- CQT 387:386 (ms, temps de quiesce après Keep Alive Timeout ou perte de communication) ; CDPA 388.
- Pour l'agent : CQT pertinent si Keep Alive utilisé (non implémenté).

## Lignes 18401-18600 (CDPALG suite ; MUP 391:390 ; IPMSR 393:392 ; SRS/SRV 395:394 ; MSMT 397:396 ; MNENS 399:398 ; ENSA 404 ; ENDSFS 405 ; VSEN1-4 ; MSVMT ; 511:424 réservé)
- MUP 391:390, IPMSR 393:392 (ms), MSMT 397:396. ENSA 404, ENDSFS 405.
- Voltage sensors VSEN1-4 (octets 409:406 ... 421:418) puis MSVMT 423:422.
- Pour l'agent : hors périmètre (power/voltage/exported).

## Lignes 18601-18800 (IDENTIFY CONTROLLER : SQES 512 ; CQES 513 ; MAXCMD 515:514 ; NN 519:516 ; ONCS 521:520 (bits)
- SQES octet 512 : MAXSQES bits 7:4 (recommandé 6), MINSQES bits 3:0 (obligatoire = 6 = 64 o).
- CQES octet 513 : MAXCQES bits 7:4 (recommandé 4), MINCQES bits 3:0 (obligatoire = 4 = 16 o).
- MAXCMD octets 515:514 (NVMe-oF obligatoire, PCIe optionnel).
- NN octets 519:516 (NSID max valide, 0 non valide).
- ONCS octets 521:520 : NSZS 12, MAXWZD 11, NVMAFC 10, NVMCSA 9, NVMCPYS 8, NVMVFYS 7, TSS 6, RESERVS 5, SSFS 4, NVMWZSV 3, NVMDSMSV 2, NVMWUSV 1, NVMCMPS 0.
- Pour l'agent : CRITIQUE : CC.IOSQES doit valoir 6 et CC.IOCQES 4 (= valeurs minimales requises) ; vérifier dans main.rs. Pour la lecture des namespaces, NN est la borne ; la vérification de SSFS etc. est informative.

## Lignes 18801-19000 (Identify Controller : FUSES 523:522 (FCWS bit 0) ; FNA 524 (FNVMBS 3, CRYES 2, SENS 1, FNS 0) ; VWC 525 (FB 2:1, VWCP 0) ; AWUN 527:526 ; AWUPF 529:528 ; ICSVSCC 530)
- FNA octet 524 : FNVMBS bit 3 (Format broadcast non supporté si 1), CRYES bit 2, SENS bit 1, FNS bit 0.
- VWC octet 525 : FB bits 2:1 (flush NSID FFFFFFFFh), VWCP bit 0.
- AWUN 527:526 et AWUPF 529:528 (écriture atomique, unités de blocs logiques).
- ICSVSCC octet 530.
- Pour l'agent : informatif.

## Lignes 19001-19200 (ICSVSCC ; NWPC 531 ; ACWU 533:532 ; CDFS 535:534 ; SGLS 539:536 (SGLS 01:00 = 01b supporté sans alignement) ; MNAN 543:540? ; MAXDNA ; MAXCNA ; OAQD ; RHIRI)
- SGLS octets 539:536 : bits 1:0 = support (00 non, 01 oui, 10 dword aligned), bit 2 SBBDS? (non vérifié précisément), SAOS, MSDS, LLDTS, etc.
- NWPC octet 531 (PWPS bit 2, WPUPCS bit 1, NWPWPS bit 0).
- CDFS 535:534 (CDF0S..CDF4S bits 0..4).
- MNAN octets 543:540 (nombre max de namespaces, 0 = <= NN) ; MAXDNA ; MAXCNA ; OAQD ; RHIRI.
- Pour l'agent : SGLS n'est pas utilisé (PCIe admin = PRP).

## Lignes 19201-19400 (Identify Controller : HIRT 569 ; CMMRTD 571:570 ; NMMRTD 573:572 ; MINMRTG 574 ; MAXMRTG 575 ; TRATTR 576 ; MCUDMQ/MNSUDMQ/MCMR/NMCMR/MCDQPC ; SUBNQN 1023:768 ; IOCCSZ/IORCSZ/ICDOFF/FCATT/MSDBD)
- TRATTR octet 576 (MRTLL bit 2, TUDCS bit 1, THMCS bit 0).
- SUBNQN octets 1023:768 (UTF-8 NUL-terminated, obligatoire si revision >=1.2.1).
- Fabrics : IOCCSZ, IORCSZ, ICDOFF, FCATT (NZNSETIDS 1, DCMS 0), MSDBD. Non PCIe.
- Pour l'agent : SUBNQN est une donnée identifiante (ne pas afficher intégralement, voir README de la clé).

## Lignes 19401-19600 (Identify Controller suite : MSDBD 1805:1804? ; OFCS 1805 ; DCTYPE 1806 ; CCRL 1807 ; 2047:1808 réservé ; PSDn 2079:2048 (PSD0 à 2079:2048? ) ; VS 4095:3072 ; Voltage Sensor struct 339 ; Power State Descriptor 340)
- Power State Descriptor (32 octets, PSD0 à offset 2048) : MP (Maximum Power), MXPS, NOPS, ENTLAT, EXLAT, RRT/RRL, RWT/RWL, IDLP, ACTP, APS, etc. (noter : l'offset exact de chaque champ PSD est à confirmer dans fig.340).
- DCTYPE octet 1806 (0 non reporté, 1 DDC, 2 CDC).
- CCRL octet 1807.
- Note 5 : SN et MN doivent être non nuls ensemble pour un Discovery.
- Voltage sensor (fig.339) : IVMSR 31:16 ; VSRS 15:8 ; VSRV 7:0 ; VOLSS 15:14 ; PIT 13:12 PISL, 11:0 PISV (x0,05 V).
- Pour l'agent : Power State Descriptor 0 (PSD0) est mandatory : utile pour connaître la puissance max (Maximum Power).

## Lignes 19601-19800 (Power State Descriptor suite : EPFVT, FQVT, EPFRT, APS, APW, ACTP, IPS, IDLP, RWL/RWT/RRL/RRT, EXLAT, ENLAT, NOPS, MXPS, MP ; 5.2.14.2.2 CNS 02h ; 5.2.14.2.3 CNS 03h ; 5.2.14.2.4 CNS 04h)
- PSD (32 octets) : MP 15:0 (MXPS bit 24?), NOPS bit 25?, ENLAT 31:..., EXLAT ... (positions exactes à confirmer ; fig.340 : MP bits 15:0 du dword 0 ; MXPS bit 16...). 
- Time scale (fig.341) : 0h=1 µs, 1h=10 µs, 2h=100 µs, 3h=1 ms, 4h=10 ms, 5h=100 ms, 6h=1 s, 7h=10 s, 8h=100 s, 9h=1000 s, Ah=10000 s, Bh=100000 s, Ch=1000000 s.
- CNS 02h (Active NSID list) : NSID = valeur de départ (exclusive, 0 = à partir de 1) ; retourne jusqu'à 1024 NSIDs triés ; NSID FFFFFFFEh/FFFFFFFFh -> Invalid Namespace or Format.
- CNS 03h (NS ID descriptor list) : chaque descripteur : NIDT octet 0 (1 EUI64 8 o, 2 NGUID 16 o, 3 UUID 16 o, 4 CSI 1 o), NIDL octet 1, NID à partir de l'octet 4 ; NIDL 0 = fin de liste. Un namespace doit avoir au moins NIDT 1,2 ou 3, et NIDT 4 si CAP.CSS.IOCSS=1.
- CNS 04h (NVM Set List) : NUMENT (max 31), entrées de 128 o : NSETID, ENDGID, R4KRT, OWS, TNVMSC, UNVMSC.
- Pour l'agent : CNS 02h et CNS 03h et CNS 04h ; CNS 02h est requis pour énumérer les namespaces actifs.

## Lignes 19801-20000 (CNS 05h/06h/07h/08h ; Identify Namespace I/O CS indep. : NSFEAT 0 ; NMIC 3? ; RESCAP ; FPI 3)
- CNS 05h : Identify NS I/O CS-specific (CSI) ; NSID inactif -> données nulles. CNS 06h : Identify controller I/O CS-specific. CNS 07h : active NSID list par CSI (1024 max). CNS 08h : Identify NS I/O CS indépendant.
- Identify Namespace indep. (CNS 08h) : NSFEAT octet 24 (VWCNP bit 5, RMEDIA bit 4, UIDREUSE bit 3, bits 2:0 reserved) ; NMIC octet 25 (DISNS bit 1, SHRNS bit 0) ; RESCAP octets 27:26? (IEKS 7, EAARS 6, WEARS 5, EAROS 4, WEROS 3, EAS 2, WES 1, PTPLS 0) ; FPI octet 28 (FPIS bit 7, RFNVM 6:0).
- Note : les offsets (24..28) sont ceux de la fig.346 (extraction) à confirmer ; le champ NSFEAT est à l'octet 24 et NMIC à 25 pour la table standard NVMe (vérifier).
- Pour l'agent : Identify NS pour chaque NSID actif (CNS 00h, NVM CS) est nécessaire pour la taille du disque (NSZE/NCAP/LBA format). À vérifier dans main.rs.

## Lignes 20001-20200 (fin Identify NS indep. : ANAGRPID 7:4 ; NSATTR 8 ; NVMSETID 11:10 ; ENDGID 13:12 ; NSTAT 14 ; KPIOS 15 ; MAXKT 17:16 ; RGRPID 23:20 ; 5.2.14.2.9 CNS 10h ; CNS 11h ; CNS 12h ; CNS 13h ; CNS 16h ; CNS 17h UUID list)
- Identify NS indep. (suite) : ANAGRPID 7:4 ; NSATTR bit 0 CWP (octet 8) ; NVMSETID 11:10 ; ENDGID 13:12 ; NSTAT 14 (IOI 2:1, NRDY bit 0) ; KPIOS 15 ; MAXKT 17:16 ; RGRPID 23:20 ; taille 4096 o.
- CNS 10h : liste NSIDs alloués (1024 max). CNS 11h : Identify NS (NVM CS) d'un NSID alloué. CNS 12h : contrôleurs ayant le NS attaché. CNS 13h : liste des contrôleurs I/O. CNS 16h : NS granularity. CNS 17h : liste UUID.
- Pour l'agent : NRDY (NSTAT bit 0) doit valoir 1 pour que le namespace soit lisible ; à vérifier dans le code.

## Lignes 20201-20400 (UUID list ; CNS 18h Domain list ; CNS 19h EG list ; CNS 1Ah/1Bh I/O CS allocated ; CNS 1Ch I/O CS list ; CNS 1Dh underlying ; CNS 1Fh ; CNS 20h ; CNS 22h)
- CNS 1Ch : IOCSC0..511 (8 o chacun) ; vecteur : bit 0 NVMCS, 1 KVCS, 2 ZNSCS, 3 SLMCS, 4 CPNCS. Profil I/O CS sélectionné par FID 19h (Set Features).
- CNS 18h Domain list (NUMENT max 31), CNS 19h EG list (NEGIDS max 2047), CNS 1Dh underlying NS list, CNS 20h controller state formats, CNS 22h template UUIDs : non utilisés.
- Pour l'agent : CNS 1Ch est nécessaire si CAP.CSS.IOCSS=1 (étape 8 de §3.5.1) ; vérifier dans main.rs si CNS 1Ch est lu, sinon signaler l'écart.

## Lignes 20401-20600 (CNS 14h/15h/21h Primary/Secondary ; CNS 1Eh Ports ; §5.2.15 Keep Alive ; §5.2.16 Lockdown début)
- CNS 14h Primary Controller Caps : CNTLID 1:0, PORTID 3:2, CRT 4 (VIRS bit 1, VQRS bit 0), VQFRT, VQRFA ... (VI/VQ resources) : virtualisation, non utilisé.
- CNS 15h Secondary list (127 max) : entrée 32 o : SCID 1:0, PCID 3:2, SCS 4 (OLS bit 0), VFN 7:5?, NVQ, NVI.
- Keep Alive (opcode 18h) : sans paramètres ; restart du timer si actif.
- Lockdown (opcode 24h) : CDW10 : CSEL 19:16, OFI 15:8, PRHBT bit 7?, IFC 6:5, SCP 4:0? (valeurs à confirmer dans fig.365), CDW14 : CSS 31:16, UIDX 6:0.
- Pour l'agent : Keep Alive non implémenté (acceptable, pas requis par PCIe si Keep Alive Timer non activé).

## Lignes 20601-20800 (fin Lockdown ; §5.2.17 Manage Exported Namespace (31h) ; §5.2.18 Manage Exported NVM Subsystem Receive (2Ah) : SEL)
- Lockdown : codes 1Fh Invalid Controller ID ; 28h Prohibition of Command Execution Not Supported.
- Manage Exported Namespace (31h) : SEL bits 7:0 de CDW10 (01h Associate, 02h Disassociate) ; CDW14 : ESUBID 31:16, ESUBIDV bit 7. Non supporté pour Exported NVM Subsystems ; hors périmètre agent.
- Manage Exported NVM Subsystem Receive (2Ah) : CDW10 SEL (00h create, 01h list subsystems, 02h list namespaces, 03h list controllers (Fabrics seulement), 04h list ports (memory interdit), 05h get config state). Hors périmètre.
- Pour l'agent : aucun de ces commandes ne doit être envoyé.

## Lignes 20801-21000 (Manage Exported NVM Subsystem Receive suite : OL/OU/NUMDL ; Create/List Exported Subsystems/Namespaces/Controllers/Ports ; structures)
- Hors périmètre de l'agent (fonctions d'exportation NVMe-oF/Exported subsystems). Notes minimales : Create Exported (opcode 2Ah, op 00h) : ESUBNQN 255:0 ; ESUBID ; TR bit 31 (CDW11) ; ENSTI bits 30:8? ; MNENS ; RA bit 8.
- Lists : 01h subsystems (260 o/entrée), 02h namespaces (8 o), 03h controllers (4 o), 04h ports (40 o).
- Pour l'agent : aucun de ces commandes ne doit être lancé.

## Lignes 21001-21200 (Manage Exported NVM Subsystem Receive fin ; §5.2.19 Manage Exported NVM Subsystem Send (2Dh) : SEL ; Delete ; Change Access Mode ; Grant Host Access début)
- Send (2Dh) : CDW10 SEL (01h Delete, 02h Change Access Mode, 03h Grant Host Access, 04h Revoke, 05h Associate Controllers, 06h Set Exported Config). CDW14 ESUBID 31:16, ESUBIDV bit 7.
- Hors périmètre de l'agent (exports NVMe-oF).
- Pour l'agent : non utilisé.

## Lignes 21201-21400 (Send : Grant/Revoke Host Access ; Associate Controllers ; Set Exported Config ; §5.2.20 Migration Receive (42h) : SEL ; Get Controller State début)
- Hors périmètre de l'agent (migration, exports).
- Migration Receive (42h) : CDW10 SEL (00h Get Controller State, 01h Get Exported NVM Subsystem State) ; CDW12/13 OL/OU (dword-aligned) ; CDW15 NUMDL.
- Pour l'agent : non utilisé.

## Lignes 21401-21600 (Migration Receive suite ; §5.2.21 Migration Send (41h) : SEL, SEQIND ; Suspend)
- Migration/Suspend : hors périmètre agent.
- Migration Send CDW10 SEL (00h Suspend, 01h Resume, 02h Set Controller State, 03h Set Exported State).
- Pour l'agent : non utilisé.

## Lignes 21601-21800 (Resume ; Set Controller State ; structures Controller State / NVMe Controller State : AMA, AQA/ASQ/ACQ, INTM, CC, CSTS, NIOSQ/NIOCQ, listes SQ/CQ, AER)
- Structures migration : hors périmètre agent, mais donnent la liste exacte des champs de contrôleur à sauvegarder : AQA, ASQ, ACQ, INTM, CC, CSTS, ASQHP/TP, ACQHP/TP, S0PT (phase tag).
- Pour l'agent : rien à faire.

## Lignes 21801-22000 (Pending events ; SQ/CQ state structures : IOSQPRP1 ; IOSQQSIZE ; IOSQQID ; IOSQCQID ; IOSQA (PC bit 0) ; IOCQA (IEN bit 1, PC bit 0, IV 31:16, S0PT bit 0?) ; §5.2.21.2.4 Set Exported ; §5.2.22/23 NVMe-MI ; §5.2.24 Namespace Attachment début)
- Structures d'état de queue (migration) : indiquent les champs du Create I/O SQ/CQ : PRP1, QSIZE, QID, CQID, QPRIO, PC ; IEN, IV, PC (CQ).
- §5.2.22/23 : NVMe-MI renvoyé au document MI.
- §5.2.24 Namespace Attachment (opcode 15h) : CDW10 SEL (0 controller attach, 1 controller detach) ; DPTR : Controller List 4096 o ; attach/detach persistants.
- Pour l'agent : rien à faire ici (l'agent ne fait pas d'attachement).

## Lignes 22001-22200 (fin Namespace Attachment ; §5.2.25 Namespace Management (0Dh) ; §5.2.26 Sanitize (84h) début)
- Namespace Attachment (15h) : SEL CDW10 bits 3:0 (0 attach, 1 detach) ; codes 18h NS déjà attaché, 19h NS privé, 1Ah non attaché, 1Ch Controller List invalide, 25h ANA, 27h limite, 29h CS non supporté, 2Ah CS non activé.
- Namespace Management (0Dh) : CDW10 SEL bits 3:0 (0 create, 1 delete, 2 restore) ; CDW11 CSI 31:24 ; create : données 4096 o ; NSID FFFFFFFFh delete = tous. Codes : 0Ah Invalid Format ; 15h Insufficient capacity ; 16h NS ID unavailable ; 1Bh thin provisioning ; 24h ANA ; 29h CS non supporté.
- Sanitize (84h) : CDW10 SANACT, PREQ bit 11?, EMVS bit 9? ; efface les données ; refuser toute exécution sans accord explicite.
- Pour l'agent : Namespace Management, Format, Sanitize et Attachment ne doivent PAS être exécutés (écrivent sur le disque).

## Lignes 22201-22400 (Sanitize CDW10/11 ; SANACT ; codes ; §5.2.27 Sanitize Namespace (45h? 8Ch) ; §5.2.28 Security Receive (82h) ; SECP)
- Sanitize (84h) CDW10 : SANACT bits 2:0 (001 Exit Failure Mode, 010 Block Erase, 011 Overwrite, 100 Crypto Erase, 101 Exit Media Verification) ; AUSE bit 3 ; OWPASS 7:4 ; OIPBP 8? ; NDAS 9 ; PREQ 10? ; EMVS 11? (confirmé 22199) ; CDW11 OVRPAT. Codes : 0Bh Firmware activation... ; 10h/11h ; 23h Sanitize prohibited PMR ; 39h Controller Suspended.
- Sanitize Namespace (8Ch) : CDW10 SANACT bits 2:0 (100 crypto erase, 101 exit). NSID FFFFFFFFh/0 -> Invalid Namespace. Codes 12h MNSOIP exceeded.
- Security Receive (82h) : CDW10 SECP 31:24, SPSP1 23:16, SPSP0 15:8, NSSF 7:0 (EAh : 0001h RPMB, 0002h CDP) ; CDW11 AL.
- Pour l'agent : sanitize et security interdits (écriture/effacement). Security Receive en lecture seule pourrait servir à l'inventaire (non prévu).

## Lignes 22401-22600 (fin Security Send ; §5.2.30 SET FEATURES : DPTR ; CDW10 SV bit 31, FID bits 7:0 ; CDW14 UIDX ; table des FIDs persistance/données)
- Set Features (opcode 09h) : CDW10 SV bit 31 (save), FID 7:0 ; CDW14 UIDX 6:0. Code Feature Identifier Not Saveable si SV=1 non supporté.
- Table FIDs (fig.466) : 01h Arbitration (Controller, pas de persistance), 02h Power Management (Controller), 03h LBA Range (NVM CS), 04h Temp threshold, 05h Error recovery (NVM CS), 06h Volatile Write Cache, 07h Number of Queues, 08h Interrupt Coalescing, 09h Interrupt Vector Config, 0Ah Write Atomicity, 0Bh AEC (Controller, Yes/data?), 0Ch APST (données), 0Dh HMB, 0Eh Timestamp, 0Fh Keep Alive Timer, 10h HCTM, 11h NOPPC, 12h RRL, 13h-1Dh (PLM, Host behavior, sanitize, EG event, I/O CS profile, spinup, PLS, FDP, FDP events), 1Fh NS admin label, 21h CDQ, 22h CDP, 23h power limit, 24h-28h power/voltage, 78h-7Dh management addresses / metadata ; 29h-77h réservés ; 7Eh-7Ch réservés ; 84h NS write protection ; 85h BP write protection.
- Pour l'agent : Set Features FID 07h (Number of Queues, NSID 0, Controller scope) est l'étape 3 de §3.5.1 ; FID 19h (I/O CS profile) est nécessaire si CSS IOCSS.

## Lignes 22601-22800 (suite table FIDs : 7Dh-85h ; notes 1-8 : colonnes persistance/données/portée)
- Notes confirmées : note 2 (colonne 'current setting' seulement si non saveable) ; note 3 (HMB non persistante) ; note 4 (Set Features sans buffer, Get avec buffer) ; note 6 (scope dans FID Supported & Effects) ; note 7 (puissance par domaine) ; note 8 (persistance définie par la feature).
- FID 84h/85h : NS write protection (namespace), BP write protection (controller). FID 86h-BFh réservés, C0h-FFh vendor.
- Pour l'agent : ne pas envoyer de Set Features autres que 07h/19h, et uniquement en connaissance de la portée (NSID=0 pour controller).

## Lignes 22801-23000 (§5.2.30.1.1 Arbitration FID 01h ; §5.2.30.1.2 Power Management 02h ; §5.2.30.1.3 Temperature 04h ; §5.2.30.1.4 VWC 06h ; §5.2.30.1.5 NUMBER OF QUEUES 07h ; §5.2.30.1.6 AEC 0Bh début)
- Arbitration FID 01h CDW11 : HPW 31:24, MPW 23:16, LPW 15:8, AB 2:0 (2^n, 111 = illimité).
- Power Management FID 02h CDW11 : IIELL 31:16, WH 7:5?, PS 4:0.
- Temperature FID 04h CDW11 : TMPTHH 24:22, THSEL 21:20, TMPSEL 19:16, TMPTH 15:0 (Kelvin) ; défaut Over = WCTEMP.
- VWC FID 06h CDW11 : WCE bit 0.
- NUMBER OF QUEUES FID 07h (CRITIQUE pour l'agent) :
  CDW11 : NCQR bits 31:16 (I/O CQ demandées, 0-based, max 65534), NSQR bits 15:0 (I/O SQ demandées, 0-based).
  CQE DW0 : NCQA bits 31:16, NSQA bits 15:0 (0-based, allouées par le contrôleur, peuvent être inférieures à la demande).
  Doit être envoyé AVANT toute création de queue I/O (sinon Command Sequence Error). Le premier Set Features réussi après CLR fixe l'allocation jusqu'au prochain CLR.
- Pour l'agent : vérifier que main.rs envoie Set Features FID 07h avant Create IO CQ/SQ, et que NCQR/NSQR sont 0-based (valeur demandée - 1).

## Lignes 23001-23200 (§5.2.30.1.6 Asynchronous Event Configuration 0Bh : CDW11 bits (DLPCN 31, HDLPCN 30, ADLPCN 29, PMDRLPCN 28, ZDCN 27, RLCCN 25?, LHCN 24?, CCRCN 23?, ANSAN 19, RGRP0 18, RASSN 17, TTHRY 16, NNSSHDN 15, EGEALCN 14, LSIAN 13, PLEALCN 12, ANACN 11, TLN 10, FAN 9, NAN 8, SHCW 7:0) ; §5.2.30.1.7 APST 0Ch début)
- AEC 0Bh CDW11 : SHCW bits 7:0 (critical warnings SMART activés), NAN bit 8, FAN bit 9, TLN bit 10, ANACN bit 11, PLEALCN 12, LSIAN 13, EGEALCN 14, NNSSHDN 15, TTHRY 16, RASSN 17, RGRP0 18, ANSAN 19.
- Ces bits ne sont à activer que si OAES les supporte (sinon Invalid Field in Command).
- Pour l'agent : si AER est utilisé, SHCW=0xFF est un choix raisonnable à proposer ; non implémenté. Mention pour la suite.
- APST 0Ch : APSTE bit 0 (CDW11) et table de 32 entrées dans le data buffer.

## Lignes 23201-23400 (APST entrée : ITPT 31:8, ITPS 7:3? ; §5.2.30.1.8 Timestamp 0Eh ; §5.2.30.1.9 Keep Alive Timer 0Fh : KATO (ms) ; §5.2.30.1.10 HCTM 10h ; §5.2.30.1.11 NOPSC 11h ; §5.2.30.1.12 RRL 12h ; §5.2.30.1.13 PLM 13h début)
- APST entrée (32 o) : ITPT (ms) bits 31:8 ? (positions exactes dans fig.477 à confirmer) ; ITPS bits 7:3 ; 0 = désactivé.
- Timestamp (0Eh) : data 6 o TSTMP (ms depuis 01-01-1970 UTC) ; Get : TSTMPO 3:1, SYNC bit 0 (octet 6-7).
- Keep Alive (0Fh) : KATO = CDW11 31:0 en millisecondes ; défaut 0 en PCIe (pas de Keep Alive) ; 120000 ms pour RDMA/TCP.
- HCTM (10h) : TMT1 31:16 (K), TMT2 15:0 (K) ; 0 = désactivé.
- NOPSC (11h) : NOPPME bit 0.
- RRL (12h) : NVMSETID 15:0 dans CDW11, RRL 3:0 dans CDW12.
- Pour l'agent : aucun de ces FIDs n'est envoyé (hors 07h/19h).

## Lignes 23401-23600 (PLM 13h suite ; PLM Window 14h ; Host Behavior Support 16h : ACRE octet 0, ETDAS 1, LBAFEE 2, HDISNS 3, CDFE 5:4 ; Sanitize Config 17h NODRM bit 0 ; EG Event Config 18h ; I/O CS Profile 19h ; Spinup 1Ah)
- PLM 13h : NVMSETID CDW11 15:0 ; LPE CDW12 bit 0 ; structure seuil 512 o.
- PLM Window 14h : NVMSETID CDW11 ; WSEL CDW12 2:0 (2 DTWIN, 3 NDWIN).
- Host Behavior 16h (512 o) : ACRE octet 0 (bit 0), ETDAS octet 1, LBAFEE octet 2, HDISNS octet 3, CDFE octets 5:4 ; non sauvable ; défaut 0.
- Sanitize Config 17h : NODRM bit 0 (CDW11). Scope NVM subsystem.
- EG Event Config 18h : EGCW 23:16, ENDGID 15:0.
- I/O CS Profile 19h (CRITIQUE pour l'agent si IOCSS) : CDW11 IOCSCI 8:0 (index de la combinaison) ; index 0 réservé ; code I/O Command Set Combination Rejected si NS non supporté ; sans effet si CC.CSS != 110b.
- Spinup 1Ah : SCE bit 0 ; uniquement si EG rotationnel.
- Pour l'agent : FID 19h doit être envoyé si CC.CSS=110b et IOCSS=1 (étape 8a §3.5.1). À vérifier.

## Lignes 23601-23800 (Spinup suite ; §5.2.30.1.20 Power Loss Signaling Config 1Bh : PLSM 1:0 ; §5.2.30.1.21 FDP 1Dh ; §5.2.30.1.22 FDP Events 1Eh ; §5.2.30.1.23 Namespace Admin Label 1Fh ; §5.2.30.1.24 Controller Data Queue 21h)
- PLS 1Bh : PLSM bits 1:0 (00 off, 01 EPF, 10 FQ, 11 réservé).
- FDP 1Dh : ENDGID CDW11 15:0 ; CDW12 : FDPCIDX 15:8, FDPE bit 0 ; saveable.
- FDP Events 1Eh : NOET CDW11 23:16, PHNDL CDW11 15:0 ; CDW12 FDPEE bit 0.
- Namespace Admin Label 1Fh : 256 o NUL-terminated UTF-8.
- CDQ 21h : ETPT CDW11 bit 31, CDQID 15:0 ; HP CDW12 ; TPT CDW13.
- Pour l'agent : aucun de ces FIDs.

## Lignes 23801-24000 (CDQ fin ; §5.2.30.1.25 CDP 22h : PERID, CHPS, PERFS, PPSC, PMDSS ; authentification PCAS/PKAS ; Manufacturing Default Personality 00h ; Security Personality 01h début)
- CDP 22h : CDW13 : CHPS bit 9, PERFS bit 8, PERID bits 7:0 ; FFh = All Personalities ; personnalité 00h = défaut fabricant ; 01h sécurité ; 02h lockdown persistence ; 03h revert.
- Personnalités non supportées -> Invalid Field in Command.
- Hors périmètre agent ; pertinent seulement pour un contrôleur "sécurisé".
- Pour l'agent : aucun Set Features CDP.

## Lignes 24001-24200 (Security Personality 01h ; Lockdown Persistence 02h ; Revert 03h ; All Personalities FFh ; §5.2.30.1.26 Power Limit 23h ; §5.2.30.1.27 Power Threshold 24h ; §5.2.30.1.28 Power Measurement 25h début)
- Hors périmètre de l'agent (sécurité/personnalités/puissance).
- Power Limit 23h : PLS 17:16, PLV 15:0 (CDW13). Power Threshold 24h : EPT bit 31, PMTS 30:24?, PTS 19:18?, PTV 15:0. Power Measurement 25h : ACT 3:0 (0 stop, 1 start), PMTS 7:4?, SMT 31:16.
- Pour l'agent : rien.

## Lignes 24201-24400 (Power Measurement CDW12/CQE ; §5.2.30.1.29 Voltage Threshold 26h ; §5.2.30.1.30 Voltage Measurement 27h ; §5.2.30.1.31 EMCA 78h ; §5.2.30.1.32 HMAA 79h ; §5.2.30.1.33 Host Metadata 7Dh-7Fh début)
- Voltage Threshold 26h : CDW11 VSENS 31:30?, EVT bit 28?, OVT 27:14, UVT 13:0 (confirmé dans structure).
- Voltage Measurement 27h : CDW11 ACT 3:0 (0 stop, 1 start, 2 clear OVV/UVV) ; data 4096 o pour start : LVOLTA (VMLT bit 2, VSSEL 1:0, SVMT 31:16?), LVOLT (LOVT 29:16, LUVT 13:0).
- EMCA 78h et HMAA 79h : URI UTF-8 NUL-terminated dans 512 o (bytes 511:4).
- Host Metadata 7Dh-7Fh : CDW11 GDHM bit 0 (Get) ; EA 14:13 (Set).
- Pour l'agent : aucun.

## Lignes 24401-24600 (Host Metadata suite : EA, NMED, element types ; §5.2.30.1.34 Software Progress Marker 80h ; §5.2.30.1.35 HOST IDENTIFIER 81h ; §5.2.30.1.36 Reservation Notification Mask 82h ; §5.2.30.1.37 Reservation Persistence 83h début)
- Host Metadata data structure : NMED bytes 1:0 ; descripteurs ELEN, ER, ET, EVAL.
- Software Progress Marker 80h : PBSLC bits 7:0 (CDW11).
- Host Identifier 81h (CRITIQUE pour réservations / NVMe-oF) : CDW11 EXHID bit 0 (1 = 128 bits, 0 = 64 bits) ; data 16 o HOSTID (octets 15:0) ; valeur 0 = non associé ; non sauvegardable ; défaut 0 en PCIe.
- Reservation Notification Mask 82h : CDW11 RESPRE bit 3, RESREL bit 2, REGPRE bit 1.
- Reservation Persistence 83h : PTPL (par namespace), pas sauvegardable.
- Pour l'agent : Host ID et réservations non utilisés.

## Lignes 24601-24800 (Reservation Persistence 83h ; §5.2.30.1.38 NS Write Protection 84h : WPS 2:0 ; §5.2.30.1.39 BP Write Protection 85h : BP1WPS 5:3, BP0WPS 2:0 ; §5.2.30.2 PCIe : 08h Interrupt Coalescing ; 09h Interrupt Vector Config ; 0Dh Host Memory Buffer début)
- Reservation Persistence 83h : PTPL bit 0 (CDW11).
- NS WP 84h : WPS 2:0 (000 no WP, 001 WP, 010 WP until power cycle, 011 permanent, sinon réservé).
- BP WP 85h : BP1WPS 5:3, BP0WPS 2:0 (001 unlocked, 010 locked, 011 locked until power cycle, 100 RPMB).
- 08h Interrupt Coalescing (PCIe) : CDW11 TIME 15:8 (100 µs), THR 7:0 (0-based). Applicable aux I/O queues seulement.
- 09h Interrupt Vector Config : CDW11 CD bit 16, IV 15:0 ; la CQ doit exister au préalable.
- 0Dh HMB : CDW11 CTZ bit 3?, HMNARE bit 2, MR bit 1, EHM bit 0 ; CDW12 HSIZE (unités CC.MPS) ; CDW13 HMDLLA (16 o aligné).
- Pour l'agent : HMB, coalescing, vecteurs d'interruption non utilisés (polling).

## Lignes 24801-25000 (HMB suite : HMDLEC, HMDL descriptors, CQE DW0 HMNAR/HMNARE/EHM, attributs ; Fabrics ; §5.2.30.3.1 AEC ; §5.2.30.x Set Features status ; §5.2.31 Track Receive (3Eh))
- HMB : descripteur 16 o : BSIZE 127:96, BADD 63:0 ; HMDLEC CDW15 ; hors périmètre.
- Set Features status : 0Dh Feature Not Saveable ; 0Eh Feature Not Changeable ; 0Fh Not Namespace Specific ; 14h Overlapping ; 15h I/O CS Combination Rejected ; 1Fh Invalid Controller ID ; 37h Invalid CDQ ; 3Eh Invalid Power Limit.
- Track Receive (3Eh) : CDW10 SEL (0 Tracked Memory Changes) ; CDW11 CNTLID ; hors périmètre.
- Pour l'agent : aucun Track.

## Lignes 25001-25200 (Track Send fin ; §5.3 PCIe admin ; §5.3.1 CREATE I/O COMPLETION QUEUE (05h) : PRP1, CDW10, CDW11, statuts)
- Create I/O CQ (opcode 05h) :
  DPTR/PRP1 (CDW11.PC=1 -> adresse contiguë page-alignée ; PC=0 -> pointeur de liste PRP) ; offset 0 obligatoire.
  CDW10 : QSIZE bits 31:16 (0-based ; 0 ou > MQES -> Invalid Queue Size), QID bits 15:0 (0 interdit, > nombre de queues alloué, ou déjà utilisé -> Invalid Queue Identifier).
  CDW11 : IV bits 31:16 (transport), IEN bit 1 (interruptions), PC bit 0. Si CAP.CQR=1 et PC=0 -> Invalid Field in Command.
  Codes : 01h Invalid Queue Identifier ; 02h Invalid Queue Size ; 08h Invalid Interrupt Vector.
- Pour l'agent (vérification) : QID=1 (première I/O queue), QSIZE = taille-1 (0-based), PC=1 si mémoire contiguë, IEN=0 (polling), IV=0.
- CQ doit avoir P=0 initial (§4.2.4), CQ créée AVANT la SQ associée.

## Lignes 25201-25400 (§5.3.2 CREATE I/O SQ ; §5.3.3 DELETE I/O CQ ; §5.3.4 DELETE I/O SQ ; §5.3.5 Doorbell Buffer Config)
- Create I/O SQ (opcode 01h) : PRP1 (PC=1 contigu, PC=0 liste PRP, offset 0) ; CDW10 QSIZE 31:16 (0-based ; 0 ou > MQES -> Invalid Queue Size 02h), QID 15:0 (0 ou > Number of Queues ou déjà utilisé -> Invalid Queue Identifier 01h) ; CDW11 CQID 31:16 (0 ou hors plage -> Invalid QID 01h ; CQ non créée -> Completion Queue Invalid 00h), QPRIO 02:01, PC bit 0 (PC=0 et CAP.CQR=1 -> Invalid Field) ; CDW12 NVMSETID 15:0 (non nul hors NVM Set List -> Invalid Field si SQ Associations supportée).
- Invalid Queue Size 02h aussi si CC.IOSQES non initialisé.
- Delete I/O CQ (opcode 04h) : CDW10 QID 15:0 ; 0 interdit ; SQ associée encore présente -> Invalid Queue Deletion (0Ch).
- Delete I/O SQ (opcode 00h) : CDW10 QID 15:0 ; 0 interdit ; statut 01h Invalid Queue Identifier.
- Doorbell Buffer Config (opcode 7Ch, émulation seulement) : PRP1 shadow, PRP2 EventIdx ; non supporté sur contrôleur physique typique. Pour l'agent : ne pas utiliser.
- Condition : (4 << CAP.DSTRD) * (max(NSQA,NCQA)+1) <= 2^(12+CC.MPS).

## Lignes 25401-25600 (Doorbell Buffer PRP1/PRP2 ; §5.3.6 Virtualization Management ; §5.4 Fabrics admin ; §5.4.1 Cross-Controller Reset)
- Doorbell Buffer Config : PRP1 shadow (page-aligned), PRP2 EventIdx ; erreur Invalid Field si adresse invalide. Émulation uniquement.
- Virtualization Management (opcode C1h ou selon Figure) : CDW10 CNTLID 31:16, RT 10:08 (0=VQ, 1=VI), ACT 03:00 (1h Primary Flexible Allocation, 7h Secondary Offline, 8h Secondary Assign, 9h Secondary Online). CDW11 NR 15:00. Status 1Fh Invalid Controller ID, 20h Invalid Secondary Controller State, 21h Invalid Number of Controller Resources, 22h Invalid Resource Identifier. CQE DW0 NRM 15:0. Hors périmètre de l'agent (modifie allocation de ressources).
- Cross-Controller Reset (fabrics) : CDW10 CIU 23:16, ICID 15:0 ; CDW12/13 CIRN. CQE DW0 IRS bit 2, V bit 1, CLRI bit 0. Status 3Fh in progress, 40h Log Page Full, 41h Limit Exceeded. Non applicable (réinitialise un autre contrôleur) : NE PAS ENVOYER.
- Discovery Information Management : TAS 0h register, 1h deregister, 2h update. Fabrics seulement : NE PAS ENVOYER.
- Fabrics/Discovery = hors matériel PCIe local ; l'agent ne doit pas les utiliser.

## Lignes 25601-25800 (§5.4.2 Discovery Information Management : header, ETYPE, ENTFMT, EKTYPE, EID, ENAME/EVER, entrées basic/extended 1024/TEL)
- Fabrics/Discovery uniquement (NVMe-oF). Non applicable à l'agent PCIe local. Aucun appel à implémenter.
- Rappel : Discovery Info Mgmt = opcode admin Fabrics ; TAS 0h/1h/2h ; NE PAS ENVOYER.

## Lignes 25801-26000 (Discovery : NQN/TRADDR, TEL/NUMEXAT, Extended Attribute TYPE 1h Host ID 16 o, 2h/3h admin label ASCII/UTF-8 4-256 o ; tableau d'applicabilité)
- Fabrics uniquement. Rien à implémenter dans l'agent local.

## Lignes 26001-26200 (§5.4.2 statuts 2Fh/32h ; §5.4.3 FZL 30h-34h, ZDK CQE DW0 ; §5.4.4 FZR, §5.4.5 FZS (DW10 ZDK, DW11 ZDO dword-aligned, DW12 LF/ZDKC/NUMD) ; §5.4.6 Manage Exported Port (SEL 01h Create, 02h Delete ; struct 4096 o))
- Fabrics/Zoning/Export : hors périmètre local. Aucun envoi.

## Lignes 26201-26400 (§5.4.6 Delete Exported Port ; §5.4.7 Send Discovery Log Page (70h/71h/72h) ; FIN §5 ; DÉBUT §6 FABRICS COMMAND SET (opcode 7Fh, Connect, Property Get/Set, Auth, Disconnect))
- §6 Fabrics : commandes 7Fh distinguées par Fabrics Command Type (FCTYPE) ; traitées indépendamment de CC.EN.
- Property Get (FCTYPE 04h) / Property Set (00h) : registres via capsule (équivalent NVMe-oF de MMIO) — hors PCIe local.
- Connect (01h), Auth Send (05h), Auth Receive (06h), Disconnect (08h) : NVMe-oF uniquement. Ne pas envoyer.
- Décision agent : §6 non applicable (transport PCIe local). À noter pour la comparaison avec main.rs : aucune commande 7Fh.

## Lignes 26401-26600 (§6.1 Authentication Receive FCTYPE 06h ; §6.2 Authentication Send FCTYPE 05h ; §6.3 Connect FCTYPE 01h : QID, SQSIZE 0-based, CATTR, KATO, NVMSETID ; règles FFFEh/FFFFh, Connect Invalid Parameters)
- Fabrics/Auth/Connect : hors périmètre PCIe local. Non envoyé par l'agent.

## Lignes 26601-26800 (§6.3 Connect data HOSTID/CNTLID/SUBNQN/HOSTNQN ; Connect response AUTHREQ ; §6.4 Disconnect FCTYPE 08h ; §6.5 Property Get FCTYPE 04h (ATTRIB PRS 4/8 o, OFST) ; §6.6 Property Set FCTYPE 00h (PUS))
- Fabrics : Property Get/Set = accès aux registres du contrôleur via capsule. Équivalent logique des MMIO (CAP/VS/CC/CSTS). Agent PCIe : utilise MMIO direct, pas Property Get.
- Aucune commande Fabrics à envoyer.

## Lignes 26801-27000 (§6.6 fin Property Set ; FIN §6 ; DÉBUT §7 I/O COMMANDS : règle CSTS.RDY + queues créées ; Figure 645 opcodes I/O)
- Opcodes I/O communs (NVM I/O, toutes CS) : 00h Flush (7.2) ; 0Dh Reservation Register (7.6) ; 0Eh Reservation Report (7.8) ; 11h Reservation Acquire (7.5) ; 12h I/O Management Receive (7.3) ; 15h Reservation Release (7.7) ; 18h Cancel (7.1) ; 1Dh I/O Management Send (7.4) ; 7Fh Fabrics (§6).
- Règle : commandes I/O seulement si CSTS.RDY=1 ET queues I/O créées.
- Cancel (18h) : CDW10 Cancel Action + CID ; CDW11 ; abort des commandes sur la même SQ. Non utilisé par l'agent.
- Avertissement : NSID FFFFFFFFh non supporté sauf footnote 4 (Flush peut le supporter).
- Décision agent : envoyer seulement Flush (00h) / Read (02h) / Write (01h) / Identify ; pas Reservation (écriture de réservation) ni I/O Mgmt Send (1Dh) sans accord.

## Lignes 27001-27200 (§7.1 Cancel : CDW10 CID 15:0 (FFFFh si multi), SQID 31:16 ; CDW11 ACODE 01:00 (00h single, 01h multiple) ; status 84h Invalid Command ID ; CQE DW0 CEDA 31:16 / CMDA 15:0 ; §7.2 Flush : FB=11b/10b/00b selon VWC ; VWCNP ; §7.3 I/O Management Receive (MO 00h no-op, 01h RUH Status, FFh vendor) ; §7.4 I/O Mgmt Send (MO 01h RUH Update))
- Flush (00h) : utilisable pour l'agent (écrit cache -> média) MAIS c'est une écriture logique : demande accord avant envoi.
- Cancel (18h) et I/O Mgmt (12h/1Dh) : non nécessaires à l'agent.
- Note : Flush avec NSID=FFFFFFFFh et VWC FB=10b -> Invalid Namespace or Format (0Bh).

## Lignes 27201-27400 (§7.4 fin I/O Mgmt Send ; §7.5 Reservation Acquire 11h : DW10 RTYPE 15:08, IEKEY bit 3, DISNSRS bit 4, RACQA 2:0 (0 acquire, 1 preempt, 2 preempt+abort) ; data CRKEY/PRKEY ; §7.6 Reservation Register 0Dh : CPTPL 31:30, RREGA 2:0 (0 register,1 unregister,2 replace), IEKEY bit 3 ; §7.7 Reservation Release 15h : RRELA 0 release / 1 clear ; §7.8 Reservation Report 0Eh : NUMD DW10, EDS DW11 bit 0 ; Status: GEN, RTYPE, REGSTRNT, PTPLS)
- Réservations NVMe (namespace) : agent NE DOIT PAS écrire de réservation (modifie l'état de partage du namespace). Lecture (0Eh) = sans effet, possible si accord.
- Reservation Report (0Eh) est en lecture seule : acceptable pour la sonde si demandé.

## Lignes 27401-27600 (§7.8 Reservation Status Extended (EDS=1) ; registrant CNTLID FFFFh/FFFDh ; FIN §7 ; DÉBUT §8 EXTENDED CAPABILITIES ; §8.1.1 Asymmetric Namespace Access (ANA) : CMIC.ANARS bit, ANACAP RANAOS/RANANOS/RANAIS/RANAPLS/RANACS, ANAGRPID, états Optimized / Non-Optimized / Inaccessible (status Asymmetric Access Inaccessible) / Persistent Loss / Change)
- ANA : lire CMIC bit 3 (ANARS) et ANACAP (Identify Controller offset 0x…, voir spec Figure 338) ; à noter pour l'agent : Identify NS ANAGRPID.
- Pour l'agent : seulement lecture ; ANA non bloquant pour init PCIe.

## Lignes 27601-27800 (§8.1.1 ANA suite : Persistent Loss, Change, notices, effets sur commandes Get/Set Features/Log/Identify ; §8.1.2 hôte ANA ; §8.1.3 BOOT PARTITIONS : lecture via propriétés BPINFO/BPRSEL/BPMBL, procédure 1-8, écriture via Firmware Image Download + Commit 110b/111b, Write Protection Set Features / RPMB)
- Boot Partition (pertinent UEFI/firmware) : lecture possible sans queues, CC.EN indifférent. Procédure : CAP.BPS, BPINFO.ABPID/BPSZ/BRS, BPMBL (buffer contigu), BPRSEL (BPID, BPRSZ, BPROF), attendre BRS=10b (ok) ou 11b (erreur).
- Écriture boot partition = FIRMWARE : INTERDITE sans accord explicite (Firmware Download/Commit 110b/111b).
- Pour l'agent : lecture BPINFO/BPRSEL possible en mode lecture seule si l'utilisateur accepte ; demander avant d'écrire BPRSEL (c'est une écriture de propriété).

## Lignes 27801-28000 (§8.1.3.3 Boot Partition write protection : Set Features (états Write Unlocked / Locked / Locked Until Power Cycle), RPMB BP write protection ; §8.1.3.3.3 interactions ; §8.1.4 CAPACITY MANAGEMENT : Endurance Groups, NVM Sets, Fixed/Variable, UNVMCAP, Media Units, Capacity Management command)
- Protection écriture boot partition : défaut Write Locked (si Set Features supporté). Passage Write Unlocked = acte d'écriture de config : NE PAS FAIRE sans accord.
- Capacity Management (§8.1.4) : création/suppression d'Endurance Groups et NVM Sets = modifie la capacité et les namespaces. INTERDIT à l'agent sans accord.

## Lignes 28001-28200 (§8.1.4 suite : Fixed/Variable Capacity, Restore Default (RDCCS, DCCS) ; §8.1.5 COMMAND AND FEATURE LOCKDOWN (CFLS/CCFLS bits OACS, Lockdown command 5.2.16, log 5.2.13.1.20, status 'Command Prohibited by Command and Feature Lockdown') ; §8.1.6 CONFIGURABLE DEVICE PERSONALITY (CDP : Set Features CDP, CHPS, freeze, MRSTT, PSCUDE, IPS, Security Send/Receive CDP Auth, message types 00h-06h))
- Lockdown : vendeur seul décide quelles commandes sont verrouillables ; agent : ne pas lancer Lockdown (modifie accès aux commandes).
- CDP : changement de personnalité = modifie comportement du SSD ; INTERDIT à l'agent sans accord (Set Features 0x? CDP, Security Send). Lecture (Get Features CDP) acceptable mais jamais de Set.
- Sanitize/Firmware Commit/Download sont aborted (Access Denied) si personnalité en attente : information à garder.

## Lignes 28201-28400 (§8.1.6.1.1-2 CDP Authentication ; CDP Request/Response Data Frame (REQDFV=0, DFHL=4, RHCPS=6, CDPRMT, REQDS ; RC 00h ok, 01h fail, 02h auth fail, 03h DHK not programmed, 04h method not supported) ; §8.1.6.2 Programmable Key / Physical Credential (PPI) / Derived HMAC Key Programming (CDPRMT 3h, HPK) / Random Nonce (CDPRMT 5h?))
- Security Send/Receive CDP (SPSP 0002h) : jamais envoyés par l'agent.
- Physical Credential Unfreeze : utilise la Physical Presence Indicator (PPI) — hors agent.

## Lignes 28401-28600 (§8.1.6.2.2 Authenticated Unfreeze (PPI ou HMAC-SHA-384, nonce purgé) ; §8.1.6.3.1 Derived HMAC Key Programming : DHK = HMAC-SHA-384 MSB 32 o ; CDP Subsystem Unique String = SUBNQN[223:4] + FGUID + SN ; flow Figure 697 ; résultats RC)
- Tout ce bloc = sécurité CDP (authentification de dévérouillage de personnalité). Agent : aucune Security Send/Receive. Ne pas dériver de clé.
- Note : Identify Controller SN est utilisée dans le secret CDP -> confirmer la confidentialité de SN (cf. README : ne pas afficher).

## Lignes 28601-28800 (§8.1.6.3.2 Random Nonce ; §8.1.7 CONTROLLER DATA QUEUE : CDQP phase, Head/Tail, Create/Delete CDQ ; §8.1.8 DEVICE SELF-TEST : segments, Short (<=2 min, Current Percentage Complete log 5.2.13.1.7), Extended (temps Identify), abort par Format NVM (Figure 701), Sanitize, Self-test code Fh ; Figure 700 exemple segments)
- Device Self-test (opcode admin 14h) : diagnostic, test media (refresh). Pas dans la liste interdite stricte, mais opération sur le média -> DEMANDER ACCORD avant envoi.
- Log 5.2.13.1.7 = Self-test Result (LID 06h) : lecture seule autorisée.
- Format NVM (80h) avorte le self-test : confirmer que l'agent ne lance jamais Format.

## Lignes 28801-29000 (§8.1.9 DIRECTIVES : Directive Receive/Send ; DIRS bit OACS ; types 00h Identify, 01h Streams, 02h Data Placement, 0Fh Vendor ; DSPEC ; Identify Directive : Return Parameters (Receive op 01h, lecture seule) bits Directives Supported/Enabled ; Enable Directive (Send op 01h, écrit l'état) ; Streams : Allocate/Release Resources, Get Status ; statut 7Fh Stream Resource Allocation Failed)
- Directive Receive (opcode admin 3Ah) type 00h op 01h = lecture seule : OK pour l'agent si accord (ne modifie rien).
- Directive Send (opcode 39h) Enable Directive (op 01h) = écrit état : INTERDIT sans accord. Streams Allocate/Release = écrit : INTERDIT.
- Important : ne pas envoyer de Write avec DTYPE != 00h.

## Lignes 29001-29200 (§8.1.9.3 STREAMS : IDs 0001h-FFFFh, SSID/SRNZID/NSSC, figures 710-713 ; Directive Receive op 01h Return Parameters (MSL, NSSA, NSSO, NSSC, SWS, SGS, NSA, NSO) ; op 02h Get Status ; Directive Send op 01h Release Identifier, 02h Release Resources, op 03h Allocate Resources (Receive ; NSR CDW12) ; §8.1.9.4 Data Placement 02h : aucune opération ; §8.1.10 DISPERSED NAMESPACES (début, multi-subsystems fabrics))
- Streams & Data Placement : pas utilisés par l'agent. Allocate/Release/Enable = modifient l'état -> interdit sans accord.
- Dispersed namespaces : fabrics seulement. Hors périmètre.

## Lignes 29201-29400 (§8.1.10 suite : migration en ligne, réplication, Dispersed Namespace Management, NSID/NGUID/UUID, DISNS bit NMIC, LID 17h Dispersed Namespace Participating Subsystems, Set Features FID 16h HDISNS, Figure 720 commandes interdites si HDISNS=0 ; ANA ; Reservation DISNSRS ; §8.1.11 EXPORTING NVM RESOURCES (début))
- Dispersed : multi-sous-système, fabrics. Hors périmètre. Pour l'agent : lire DISNS dans NMIC (Identify NS) = information utile à noter.
- Set Features FID 16h (Host Behavior) : écriture d'état -> INTERDIT sans accord.
- §8.1.11 Exported NVM Resources : gestion d'exports (fabric) -> hors périmètre.

## Lignes 29401-29600 (§8.1.11 EXPORTING NVM RESOURCES suite : Exported Subsystem create/delete (Manage Exported NVM Subsystem Send/Receive opcodes 5.2.18/5.2.19, Clear Exported NVM Resource Configuration 5.2.4), Associate Controllers (SEL 05h), CNS 1Dh/21h, migration Migration Send/Receive (5.2.20/5.2.21), Exported Template UUID BC33D0BB_764F42EE_84A27F92_3C21F01Fh)
- Exported NVM = virtualisation/export multi-hôte. Hors périmètre local. Aucune de ces commandes ne doit être envoyée.
- Identify CNS 1Dh (liste Underlying NS) et 21h (liste Underlying Controllers) : lecture seule, utiles à noter pour l'inventaire (non bloquant).

## Lignes 29601-29800 (§8.1.11.2.3 migration Exported : Migration Receive 1h / Migration Send 0h Suspend, 1h Resume, 3h Set state ; §8.1.11.3 message-based : CNS 1Eh ports, Manage Exported Port, Access Mode, Allowed Host List, Revoke ; §8.1.12 FLEXIBLE DATA PLACEMENT (FDP) : Set Features FDP (FID 1Dh), FDP Events log, FDP Statistics log, Namespace Mgmt Placement Handle List, DSPEC = Placement Identifier, RG/PH/RUH, Invalid Placement Identifier event)
- Migration/Exports : hors périmètre (fabric/virtualisation).
- FDP : Set Features FDP (activer/désactiver) modifie l'Endurance Group -> INTERDIT sans accord. Il faut supprimer les namespaces avant changement -> pas pour l'agent.
- Write avec DSPEC FDP : pas envoyé par l'agent (DTYPE=0 uniquement).

## Lignes 29801-30000 (§8.1.12 FDP suite : Reclaim Unit Handle isolation (Initially/Persistently Isolated), Enabling FDP (étapes 1-9, Set Features FDP 1Dh avec FDPE + index, Directive Send Enable Data Placement) ; §8.1.13 HOST-INITIATED REFRESH (Device Self-test, NSID ignoré, DSTO HIRS, abort si Format/Sanitize/CLR) ; §8.1.14 HOST MANAGED LIVE MIGRATION (MMC/MMH/Migratable Controllers, HMLMS OACS, Migration Send 0h/1h/2h, Track Send 5.2.32 / Track Receive 5.2.31, User Data Migration Queue, TUDCS/THMCS TRATTR))
- Host-Initiated Refresh : lit/réécrit média (refresh) -> INTERDIT sans accord.
- Live Migration / Track : hors périmètre agent.
- Enabling FDP / Directive Send Enable : écriture -> INTERDIT sans accord.

## Lignes 30001-30200 (§8.1.14 suite : Live Migration étapes 1-8 ; §8.1.15 KEY PER I/O (KPIOS, KPIOSNS, CETYPE/CEV, KEYTAG) ; §8.1.16 MANAGEMENT ADDRESSES (URI, Management Address List LID, Lockdown EMCA/HMAA) ; §8.1.17 NAMESPACE MANAGEMENT (Namespace Mgmt 0Dh, Attach 15h, OACS NMS bit, Create/Delete/Restore, AEN Attached/Allocated Namespace Attribute Changed, CNS 02h/10h) ; §8.1.18 NAMESPACE WRITE PROTECTION (NWPC, états, persistance))
- Namespace Management/Attachment : INTERDIT sans accord (crée/supprime/attache). Lecture CNS 02h/10h/11h = OK.
- Key Per I/O : CETYPE=1h (KPIOTAG) dans Read/Write : agent n'envoie que CETYPE=0.
- Namespace Write Protection (Set Features/Namespace Mgmt) : écriture d'état -> INTERDIT sans accord.

## Lignes 30201-30400 (§8.1.18 suite : Namespace Write Protection (états No/Write Protect/Until Power Cycle/Permanent, NWPC Identify, Figure 737 commandes autorisées sur NSID protégé : Read, Compare, Verify, Flush, Get/Set Features (sauf), Identify, Reservation...) ; §8.1.19 POWER MANAGEMENT (NPSS, Power State Descriptors Figure 340 (MP, ENLAT, EXLAT, RRT/RWT/RRL/RWL, IDLP, ACTP, NOPS), Set/Get Features FID 02h, Autonomous PST FID 0Ch, Workload hints Figure 740, RTD3E/RTD3R, HCTM Thermal Mgmt Temp 1/2 (FID 10h)))
- Lecture Identify (NPSS + Power State Descriptors) : utile pour l'agent (inventaire) ; à lire sans effet.
- Set Features Power Mgmt (02h), APST (0Ch), HCTM (10h) : modifient le comportement -> INTERDIT sans accord.
- Avertissement spec : Power State transition via Set Features ; Non-Operational PS : pas d'I/O. Agent : ne pas changer de PS.
- RTD3E/RTD3R : Identify CC bytes (déjà noté : 87:84 / 91:88).

## Lignes 30401-30600 (§8.1.19.4-5 HCTM : TMT1/TMT2 (Set Features FID 10h), hystérésis vendeur, Identify HCTM attributes ; §8.1.19.5 PLS / Power Limit (CTRATT.PLS bit, Set/Get Features Power Limit (FID 0x?), PSD supprimés/renumérotés, APST, MUP/MUPS) ; §8.1.19.6.1-3 mécanismes ; §8.1.19.6.4 exemples Figures 742-743)
- Power Limit : Set Features modifie les PS -> INTERDIT sans accord. Lecture (Get Features) OK si accord.
- Identify : après Power Limit, NPSS et PSD peuvent changer : l'agent doit relire Identify au lieu de supposer les valeurs initiales.

## Lignes 30601-30800 (Power Limit suite cas 2-3 ; Idle I/O Exit Latency Limit (IIELLSS Identify, FID 02h) ; §8.1.20 POWER MEASUREMENT AND REPORTING (IPMSR Identify, types 0h = NVM subsystem total power, 1h-Bh réservé, Ch-Fh vendeur ; intervalles 1 s))
- Mesure de puissance : lecture seulement (Get Log/Identify). Utile pour sonde ; pas d'écriture.
- Figures 742-744 : exemples cohérents, confirment qu'il faut relire Identify après Power Limit.

## Lignes 30801-31000 (§8.1.20 suite : Power Measurement (PMS bit Identify, Set Features Power Measurement Action 0h stop / 1h start, Stop Measurement Time, MSMT, log Power Measurement LID, NPHDS/PHDO, histogramme PHBLT/PHBS/PHBC, formules) ; §8.1.21 PREDICTABLE LATENCY MODE (NVM Sets, DTWIN/NDWIN, Predictable Latency Per NVM Set log, Event Aggregate log, Mode Config/Window features))
- Power measurement : Set Features Power Measurement (action start/stop) = écriture -> INTERDIT sans accord. Lecture log = OK.
- PLM : Set Features Window/Config = écriture -> INTERDIT sans accord.

## Lignes 31001-31200 (§8.1.21 suite DTWIN/NDWIN, périodes, événements PLM ; §8.1.22 REACHABILITY (CRCAP RRSUP, RGRPID, RA characteristics 01h/02h/03h, logs LID Reachability Groups/Associations) ; §8.1.23 READ RECOVERY LEVEL (RRL 0-15 ; 4 = défaut M, 15 = Fast Fail M ; Set Features RRL config FID 12h) ; §8.1.24 RPMB (début : déconseillé pour CNSA 2.0 ; clé d'authentification programmée en environnement sûr))
- RRL : Set Features FID 12h = écriture -> INTERDIT sans accord.
- RPMB : clé d'authentification = secret -> l'agent ne programme ni ne lit de clé RPMB.

## Lignes 31201-31400 (§8.1.24 RPMB suite : RPMB Device Configuration Block (BPPED, BPLS, BPP0L/BPP1L, WPC: PWPC, WPUPCC), Request types 0001h-0007h (key programming, write counter, auth data write/read, result read, DCB write/read), Response 0100h-0700h, Operation Result OPSTAT 00h-08h (07h Authentication Key not yet programmed) ; §8.1.24 RPMB Contents (clé d'authentification write-once non lisible))
- RPMB : secret (clé write-once), DCB, contrôle de protection. Agent : aucune trame RPMB (Security Send/Receive). Ne jamais lire/écrire DCB.

## Lignes 31401-31600 (§8.1.24 : contenus RPMB (Authentication Key write-once, Write Counter 4 o, Data), RPMB Data Frame 256 o (STBY, MAC/Key, RBT, NCE, WCNTR, ADDR, SCNT, RSLT, RRM, DATA), HMAC-SHA-256 ; §8.1.24.2.1 Authentication Key Programming (Security Send RRM 0001h, puis 0005h, puis Security Receive))
- Programmation de la clé RPMB = écriture irréversible (write-once) : INTERDIT absolument sans accord écrit explicite. Jamais dans l'agent sans demande.

## Lignes 31601-31800 (§8.1.24.2.2 Read Write Counter (req 0002h / rsp 0200h) ; §8.1.24.2.3 Authenticated Data Write (req 0003h / rsp 0300h / vérif 0005h ; résultats 0085h counter expired, 0004h address, 0002h auth, 0003h counter, 0005h write, 0001h général))
- Écriture authentifiée RPMB = écriture de données persistantes : INTERDIT sans accord.
- Lecture write counter = protocole RPMB complet (Security Send/Receive) : hors agent.

## Lignes 31801-32000 (§8.1.24.2.3 fin Authenticated Data Write : Security Send 0003h puis 0005h, Receive 0300h ; §8.1.24.2.4 Authenticated Data Read (req 0004h, rsp 0400h, résultat 0006h read failure, 0004h address))
- Lecture RPMB authentifiée : hors agent (protocole Security Send/Receive). Pas de Security Send.

## Lignes 32001-32200 (§8.1.24.2.4 fin Authenticated Data Read ; §8.1.24.3 Authenticated Device Configuration Block Write (req 0006h, rsp 0600h ; résultats 0005h counter expired, 0002h auth, 0008h Invalid DCB si désactivation BPP, 0001h général) ; DCB write counter indépendant)
- DCB write : modifie protection des boot partitions -> INTERDIT (sécurité firmware).

## Lignes 32201-32400 (§8.1.24.3 fin DCB Write (rsp 0600h) ; §8.1.24.4 Authenticated DCB Read (req 0007h, rsp 0700h, Sector Count 1, MAC sur Response Type/Nonce/Address/Data/Result ; 0002h auth, 0001h général))
- DCB read : lecture authentifiée, hors agent (RPMB).

## Lignes 32401-32600 (§8.1.24.4 fin DCB Read ; FIN §8.1.24 RPMB ; §8.1.25 RESERVATIONS : RESERVS bit ONCS, RESCAP, PTPL, types (Write Exclusive, Exclusive Access, WE-Registrants Only, EA-Registrants Only, WE-All Registrants, EA-All Registrants), commandes Reservation, RHII/Host Identifier Feature, Host Identifier 0h = Host Identifier Not Initialized ; Invalid Command Opcode si non supporté ; Figure 768 comportement Read/Write)
- Réservations = verrous d'accès multi-hôte. Agent : aucune réservation (acquire/register/release) ; lecture Reservation Report seulement si accord.
- Règle utile pour sonde : si ONCS.RESERVS=0 => commandes réservation -> Invalid Command Opcode (0x01). Le contrôleur doit l'annoncer.

## Lignes 32601-32800 (Figure 768 suite : lignes Registrant / NonRegistrant ; Reservation Holder définitions ; réservations/registrations persistent CLR/NSR sauf perte d'alimentation (PTPLS) ; Figure 769 matrice A (Allowed) / C (Conflict) par commande : Read, Write, Copy, Capacity Mgmt, Flush, Format NVM, Namespace Attachment/Mgmt, Sanitize, Security Send, Reservation Acquire/Release)
- Pour l'agent : aucune commande d'écriture ne doit être émise sur un namespace réservé. Vérifier Reservation Report (0Eh) avant tout accès si ONCS.RESERVS=1 et que la sonde doit lire (lecture seule).

## Lignes 32801-33000 (Figure 769 fin ; Reservation Notifications (registration preempted / reservation released / reservation preempted, Reservation Notification log, mask feature) ; Registering / Unregistering / Acquiring / Releasing / Preempting (RACQA 001b Preempt, 010b Preempt+Abort ; PRKEY 0h = erreur) / Clearing (RRELA 001b) / Reporting ; Exported Subsystem interactions ; §8.1.26 ROTATIONAL MEDIA (NSFEAT bit RM, LID Rotational Media Info, Spinup Control FID 1Ah) ; §8.1.27 SANITIZE OPERATIONS début (Figure 770 portée NVM subsystem vs namespace))
- Réservations : toute écriture de RESERVATION (Register/Acquire/Release) = modifie l'accès -> INTERDIT sans accord.
- Sanitize (section 8.1.27) : efface les données utilisateur, irréversible -> INTERDIT sans accord explicite (Sanitize 84h, Sanitize Namespace 8Ch).
- Rotational Media : utile pour détecter disque mécanique (NSFEAT bit) : lecture Identify.

## Lignes 33001-33200 (§8.1.27 SANITIZE suite : Figure 770 ; types Block Erase / Crypto Erase / Overwrite (OIPBP, Overwrite Pass Count, Overwrite Pattern CDW10/11) ; SANICAP ; NODMMAS, NDI, VERS, NVERS ; sanitize en arrière-plan, AEN Sanitize Operation Completed (0x08?), SANACT invalide -> Invalid Field ; abort si firmware pending ; Access Denied si personnalité en attente ; PREQ/AUSE/EMVS/NDAS ; Media Verification state, Post-Verification Deallocation state ; Figure 771 Overwrite)
- SANITIZE : efface TOUT le contenu utilisateur (irréversible). INTERDIT sans accord explicite écrit, même si la spec le permet. Ne jamais envoyer 84h / 8Ch.
- Cette section confirme la règle de l'agent (Sanitize interdit).

## Lignes 33201-33400 (§8.1.27 suite : démarrage SANACT 010b Block Erase / 011b Overwrite / 100b Crypto Erase (NVM subsystem) ; Namespace Sanitize SANACT 100b ; Sanitize Status log (GDE, NDE, SPROG, SANS, FAILS, MVCNCLD) ; AEN 01h/02h/03h (LID 81h) ; NODRM ; §8.1.27.3 échecs ; §8.1.27.4 MACHINE D'ÉTAT (Idle, Restricted/Unrestricted Processing, Media Verification, Post-Verification Deallocation, Restricted/Unrestricted Failure))
- Machine à états sanitize : 80+ transitions ; l'agent n'en déclenche aucune (interdit). Lecture du Sanitize Status log (LID 81h) : autorisée en lecture seule si accord.

## Lignes 33401-33600 (§8.1.27.4.3 Restricted Failure ; §8.1.27.4.4 Unrestricted Processing ; §8.1.27.4.5 Unrestricted Failure (FAILS=3h, Exit Failure Mode 001b) ; §8.1.27.4.6 Media Verification State (Exit 101b, transitions vers Post-Verification Deallocation, lecture autorisée))
- Machine sanitize : aucune transition à déclencher par l'agent. Aucune commande Sanitize / Exit Failure Mode / Exit Media Verification (101b) à envoyer.

## Lignes 33601-33800 (§8.1.27.4.6 fin ; §8.1.27.4.7 Post-Verification Deallocation (suppression de toutes les données utilisateur) ; restrictions sanitize (suspension gestion puissance APST, abort device self-test, libération streams, Flush autorisé, CDP aborted, PMR désactivé) ; Exported Subsystems sanitize ; §8.1.28 SQ ASSOCIATIONS (CTRATT, NVM Set ↔ SQ à la création) ; §8.1.29 VENDOR SPECIFIC FORMAT (NDT/NDM) ; §8.1.30 TELEMETRY (début : LID Telemetry Host-Initiated 07h / Controller-Initiated 08h, LPA Data Area 4))
- Post-Verification Deallocation = efface les données utilisateur : INTERDIT.
- Telemetry : Get Log Page (LID 07h/08h) = lecture seule, acceptable si accord ; Telemetry Host-Initiated (Set Features) = écriture, interdit sans accord.

## Lignes 33801-34000 (§8.1.30 TELEMETRY suite : 3 ou 4 Data Areas, blocs 512 o, procédure de lecture (Retain Async Event, Generation Number, Controller-Initiated Data Available), Create Telemetry Host-Initiated Data = Get Log Page LSP bit ; §8.1.31 UUIDs VENDOR SPECIFIC (UUID list, NVMe Invalid UUID FFFFFFFF_FFFFFFFF_7FFFFFFF_FFFFFFFFh, UIDX 6:0) ; §8.1.32 VOLTAGE MEASUREMENT (VSEN1-4, IVMSR, Voltage Threshold feature, Voltage Measurement log, VMS bit))
- Telemetry : "Create Telemetry Host-Initiated Data" = déclenche une collecte côté contrôleur -> ÉCRIT (état) : INTERDIT sans accord. La lecture 512 o est OK en lecture seule si accord.
- UUID : UIDX dans CDW14 (bits 6:0). Ne jamais mettre UIDX non nul sans UUID connue.
- Voltage : Set Features Voltage Threshold = écriture -> interdit sans accord. Lecture Voltage Measurement log = OK si accord.

## Lignes 34001-34200 (§8.1.32 suite Voltage Measurement (Start/Stop, OVT/UVT, VLTHW SMART bit) ; §8.2 MEMORY-BASED TRANSPORT EXTENDED CAPABILITIES (PCIe) : §8.2.1 CMB (CAP.CMBS, CMBMSC.CRE/CMSE, CMBLOC/CMBSZ, CQMMS/CQPDS/CDPMLS/CDPCILS/CDMMMS, Invalid Use of CMB 1Bh?) ; §8.2.2 CXL ; §8.2.3 DOORBELL STRIDE (2^(2+DSTRD)) ; §8.2.4 HOST MEMORY BUFFER (HMB : descripteurs, HMMINDS/HMMAXD, Memory Return bit, reset) ; §8.2.5 PMR (CAP.PMRS, PMRCAP.BIR, PMRCTL.EN, PMRSTS.NRDY/ERR/HSTS, PMRMSCL/U, write barrier) ; §8.2.6 POWER LOSS SIGNALING (PLN/PLA, FQ / EPF, PLS states, Vault Time, Recovery Time))
- CMB/HMB/PMR : enable = écriture de propriétés et réservation de mémoire hôte : INTERDIT sans accord (risque de DMA mal dirigé).
- Attention : HMB donne de la mémoire hôte au SSD -> interdit à l'agent (il ne doit pas allouer de HMB).
- Doorbell stride : pour l'agent, DSTRD lu via CAP (offsets déjà notés : doorbells à 1000h + (2y)*(4<<DSTRD)).

## Lignes 34201-34400 (§8.2.6 PLS suite : Figure 787 états (PLS Not Ready, PLS Ready, FQ Processing/Complete, EPF Processing Port Disabled/Enabled, EPF Complete Port Disabled/Enabled) ; transitions §8.2.6.1.1-7 ; PLA values ; PLN Asserted/Deasserted)
- Power Loss Signaling : mécanisme d'hôte pour préparer une coupure. Agent : ne pas asserter PLN ni configurer PLS Config (écriture = interdit sans accord).
- Note pour sonde : PLS Not Ready après CLR ; SHST 00b = PLS Ready.

## Lignes 34401-34600 (§8.1.? PLS EPF suite ; FQ Processing : PLA Asserted-FQ, stop fetch, abort Set Features PLS Config (0x? status Commands Aborted due to PLN) ; EPF discards commands ; EPFRT/EPFRTS ; §8.2.7 VIRTUALIZATION ENHANCEMENTS (Primary/Secondary controllers, VQ/VI Resources, Online/Offline, SR-IOV PF/VF, Identify CNS 14h/15h, Virtualization Management 5.3.6) ; §8.2.8 RESTORE MANUFACTURING DEFAULT (RMDCA, DSCS bit, Personality, Sanitize optionnel) ; §8.3.1 DNS-SD discovery (fabrics))
- Virtualization : hors périmètre local (secondary controllers) -> ne jamais envoyer Virtualization Management.
- Restore Manufacturing Default : modifie tous les réglages -> INTERDIT sans accord (peut rendre données inaccessibles).
- §8.3 Fabrics : hors périmètre.

## Lignes 34601-34800 (§8.3.1 DNS-SD / mDNS discovery (_nvme-disc, _cdc, PTR/SRV/TXT/A/AAAA) ; §8.3.2 CENTRALIZED DISCOVERY (CDC/DDC, Discovery Information Management push/pull, Kickstart Discovery ICReq/ICResp/KDReq/KDResp, Connect, Get Log Page LID 70h PLEO/ALLSUBE, Fabric Zoning début §8.3.2.3))
- Entièrement fabrics / réseau. Hors périmètre agent. Aucune action.

## Lignes 34801-35000 (§8.3.2.3 FABRIC ZONING : ZoneDBConfig/ZoneDBActive, ZoneGroup (ZGORIG, ZGNAME, NUMZ), Zone (ZNAME, NUMZM, NUMZP), Zone Members TLV types 1h-6h, 11h, 12h, EFh ; rôles 1h host / 2h NVM subsystem ; ADRFAM 1h IPv4 / 2h IPv6 ; IP protocol 6h TCP / 11h UDP)
- Zoning Fabric = CDC uniquement. Hors périmètre agent.

## Lignes 35001-35200 (§8.3.2.3.4 suite : Zone member types 5h (NQN+PortID), 6h (NQN+Protocol+PortID+ADRFAM), 11h (IP+Protocol), 12h (IP+Protocol+Port), EFh ZoneAlias ; §8.3.2.3.5 ZONE PROPERTIES (1h Fabric Enforced Zone) ; §8.3.2.3.6-7 ZoneAlias ; §8.3.2.3.8 ZONING OPERATIONS (GAZ, push/pull DDC, FZL/FZR/FZS))
- Zoning : hors agent. Aucune action.

## Lignes 35201-35400 (§8.3.2.3.8.2 Push model DDC : GAZ (FZL OTYP 1h, FZR OTYP 2h), AAZ (FZL OTYP 3h, FZS OTYP 4h, verrou 30 s), RAZ (FZL OTYP 5h) ; §8.3.2.3.8.3 Pull model DDC : GAZ (OTYP 7h), AAZ (OTYP 9h, status Ah), RAZ ; statuts GAZOS 0h-5h)
- Zoning Fabric = protocole CDC/DDC : hors agent.

## Lignes 35401-35600 (§8.3.2.3.8.3 pull RAZ / statuts Pull Model (0h-5h) / Log Page Request ; AEN F0h Discovery Log Page Change ; §8.3.3 CROSS-CONTROLLER RESET RECOVERY (CIU, CIRN, CQT, étapes 1-5, IRS/V/CLRI/ACID) ; §8.3.4 FABRICS SECURE CHANNEL & IN-BAND AUTHENTICATION (AUTHREQ, Authentication Send/Receive, TLS, reauth, timeouts))
- Cross-Controller Reset : déclenche un CLR sur un AUTRE contrôleur -> INTERDIT (hors périmètre, peut couper un autre contrôleur).
- Authentication : fabrics, hors agent.

## Lignes 35601-35800 (§8.3.4.2.1 Discovery in-band auth ; §8.3.4.2.2 protocoles d'authentification NVMe in-band : SECP E9h, SPSP 01h/01h, Authentication Send/Receive mapping ; Secure Channel Concatenation (PSK) ; début §8.3.4.4 messages communs AUTH_Negotiate)
- Fabrics authentication : hors agent. Aucune Authentication Send/Receive émise.

## Lignes 35801-36000 (§8.3.4.4.1 AUTH_Negotiate (SC_C NOSC/NEWTLSPSK/REPLACETLSPSK, NAPD, protocole 01h KX-HMAC-CHAP) ; §8.3.4.4.2 AUTH_Failure1/2 (RCODE 01h, RCODEEX 01h-07h) ; §8.3.4.4.3 mapping ; §8.3.4.5 KX-HMAC-CHAP (ex-DH-HMAC-CHAP) : séquence Negotiate/Challenge/Reply/Success1/Success2, notations HMAC/H/KX)
- Protocole fabrics d'authentification. Hors agent (aucune Authentication Send).

## Lignes 36001-36200 (§8.3.4.5.2 descripteur KX-HMAC-CHAP (AuthID 01h, HALEN, KXLEN) ; hashs 01h SHA-256 (32 o), 02h SHA-384 (48), 03h SHA-512 (64) ; groupes KX 00h NULL, 01h ffdhe2048, 02h ffdhe3072, 03h ffdhe4096, 04h ffdhe6144, 05h ffdhe8192, 10h secp256r1, 11h secp384r1, 20h MLKEM768, 21h MLKEM1024, 30h/31h hybrides ; FFDHE §8.3.4.5.2.1 ; ECDHE §8.3.4.5.2.2 (point non compressé 04h))
- Cryptographie fabrics. Hors agent. Aucune dérivation de clé à implémenter.

## Lignes 36201-36400 (§8.3.4.5.2.2 ECDHE suite ; §8.3.4.5.2.3 ML-KEM (MLKEM768/1024, FIPS 203) ; §8.3.4.5.2.4 hybrides (SecP256r1MLKEM768, SecP384r1MLKEM1024) ; §8.3.4.5.3 KX-HMAC-CHAP_Challenge (HashID, KXgID, SEQNUM, CVAL, KXV) ; §8.3.4.5.4 KX-HMAC-CHAP_Reply (CVALID, SEQNUM, RVAL, CVAL, KXV))
- Cryptographie fabrics : hors agent.

## Lignes 36401-36600 (§8.3.4.5.4 fin Reply (R1 = HMAC(Kh, ...)) ; §8.3.4.5.5 Success1 (R2) ; §8.3.4.5.6 Success2 ; §8.3.4.5.7 exigences (clés uniques par NQN, éphémères, CSPRNG, effacement mémoire) ; §8.3.4.5.8 format de secret DHHC-1:xx:<base64>: avec CRC-32 ; §8.3.4.5.9 PSK TLS (KS = H(Z), PSK = HMAC(KS, C1||C2)) ; §8.3.4.5.10 mapping ; §8.3.4.5.11 configuration KX-HMAC-CHAP (Figure 878-881, AUTHREQ ASCR/ATR))
- Authentification fabrics : hors agent. Ne jamais traiter de secret DHHC-1 ni de clé.

## Lignes 36601-36800 (§8.3.4.5.11 suite : configurations KX-HMAC-CHAP avec TLS (Figures 882-885) ; §8.3.4.6 KX-HMAC-CHAP AUTHENTICATION VERIFICATION ENTITY (AVE) : Access-Request/Result, NQNe/Ke/PSKea, NQNAVE ; §8.3.4.6.2 connexions AVE (TLS 1.3 PSK))
- Fabrics : hors agent.

## Lignes 36801-37000 (§8.3.4.6.2 PSKea (HKDF) ; §8.3.4.6.3 AVE Access Protocol (Access-Request PDU AEh, Access-Result AFh, AuthRes 01h/02h, RCODE) ; §8.3.4.6.4 AVE discovery ; FIN §8.3 ; DÉBUT §9 ERROR REPORTING AND RECOVERY : §9.1 erreurs queues (delete/recreate SQ/CQ ; CLR si erreur admin ; Retry bit) ; §9.2 erreurs média (Unrecovered Read 81h, données à ignorer) ; §9.3 erreurs mémoire PCIe (Data Transfer Error) ; §9.4 Internal Error ; §9.5 CONTROLLER FATAL STATUS (CSTS.CFS, Controller Reset puis NSSR si nécessaire, NSSR déconseillé si OS FFE) ; §9.6 COMMUNICATION LOSS (Time-Based Recovery, Cross-Controller Reset, retry des commandes : Unrestricted / Delayed / State-Dependent))
- §9.5 PERTINENT agent : si CSTS.CFS=1 -> reset contrôleur (CC.EN=0 puis réinit), NSSR seulement si accord. Ajouté au plan de vérification du code (main.rs : gestion CFS).
- §9.6 : pour l'agent (mono-contrôleur, PCIe local), seule la voie Time-Based Recovery s'applique ; attendre CQT ou 10 s avant réinitialisation.

## Lignes 37001-37200 (§9.6.3 Command Retry : Unrestricted / Delayed / State-Dependent ; ANNEX A Sanitize (hidden storage, No-Deallocate, audit) ; ANNEX B Host Considerations : B.1 construction SQE (CDW0 OPC/FUSE/CID, NSID, MPTR, PRP1/2, CDW10-15), B.2 création SQ (opc 01h, QSIZE=191 pour 192 entrées, PC=0 avec PRP list, CQ créée avant), B.3 fused (Compare 05h FUSE=01b / Write 01h FUSE=10b), B.4 AER recommandations, B.5 Shadow Doorbell (algorithme X<=Y modulo), B.6 exemples d'erreurs de retry (write after write, non-idempotent, retry échoue))
- Annex B.2 : confirme que la SQ doit référencer une CQ existante et que la PRP list reste en mémoire jusqu'à suppression.
- Annex A : sanitize non vérifiable à 100 % ; décision agent inchangée (pas de sanitize).

## Lignes 37201-37300 (FIN DE LECTURE NVMe 2.4, lignes 1-37300)
- Annex B.6.3 : Reservation Register unregister retenté -> Reservation Conflict (non idempotent). Pas d'impact sur l'agent (aucune réservation).
- B.6.4 : coordination multi-hôte par écriture de données déconseillée ; utiliser réservations ou Compare+Write fused.
- B.7 Restore Manufacturing Default : Capacity Mgmt, Namespace Mgmt, CDP Set Features, NSSR, Sanitize -> INTERDIT à l'agent.
- Annex C (Power Management) : résumé des mécanismes, lecture seule utile (Identify PSD, Features Get).
=== FIN DE LECTURE NVME 2.4 (37300 lignes) ===
