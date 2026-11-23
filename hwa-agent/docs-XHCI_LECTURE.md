# Lecture xHCI 1.2c ligne par ligne (USB_xHCI_1.2c.txt, 27951 lignes)
Objectif : vérifier usb_r0.rs / usb_enum.rs / usb_hid.rs. Lecture seule, aucune écriture matérielle.

## Lignes 1-450 (page de garde, sommaire, listes de figures/tableaux, historique des révisions 0.96 -> 1.2c, références)
- Révision 1.2c, octobre 2025, document 868295. Sections utiles pour l'agent : 4.2 (init contrôleur), 4.3 (dispositif USB), 4.6 (commandes), 4.9 (anneaux), 4.19 (ports), 4.21-4.22 (PCI, handoff BIOS->OS), 5.3-5.4 (registres), 6.x (structures), 6.6 (scratchpad), 7.1 (USB Legacy), 7.2 (Supported Protocol).
- Rien à vérifier dans cette partie (pas de spécification de comportement).

## Lignes 451-900 (références, index, glossaire 1.6 : Chip Hardware Reset, Core/Aux Power, DCI, Doorbell Array (256 registres, DB0 = contrôleur), Device Slot, Endpoint Address (bits 4-1 numéro, bit 0 direction ; contrôle = 1), Enqueue/Dequeue Pointer, ESIT, Event Data TRB, HIRD, Isoch TD, Link TRB, Microframe 125 us, MSI/MSI-X, Normal TRB, Scratchpad, Slot ID ; conventions : "shall" obligatoire, "should" recommandé, nombres hex suffixe h)
- Conventions : 'shall' = obligatoire ; 'must' déprécié. Ne sert qu'à comprendre le vocabulaire.
- Aucun écart constaté à ce stade.

## Lignes 901-1350 (§2 Introduction : motivations, objectifs, fonctionnalités ; §3 Architecture : §3.1 trois espaces (Config PCI, MMIO, Host Memory), BAR -> Capability/Operational/Runtime/Doorbell ; §3.2 structures : DCBAA (entrée 0 = scratchpad), Device Context (32 contextes : Slot + 31 Endpoint), Slot Context (champs réservés xHC, modifiables seulement si Disabled), Endpoint Context (TR Dequeue Pointer, Streams, CErr), Input Context (Input Control Context + Slot + 31 Endpoints), Rings (Command 1, Event 1 par Interrupter, Transfer 1 par endpoint/stream), TRB (64 bits pointeur + 32 status + 32 control, Length max 64K), Link TRB ; chaînage CH : tous sauf le dernier ; règle : ne jamais basculer le cycle bit tant que les TRB ne sont pas complets)
- Point à vérifier dans usb_enum.rs / usb_r0.rs : l'anneau de commande doit être terminé par un Link TRB (Toggle Cycle) et la DCBAA entrée 0 pointe vers le scratchpad.
- Rappel de la spec : 'Slot Context reserved fields' ne doivent pas être modifiés par le logiciel hors état Disabled.

## Lignes 1351-1800 (§3.2.8-3.2.11 : scatter/gather (TRB Length 0..64K, pas de contrainte de page), Control TD (Setup/Data/Status, Setup Immediate, Status direction), Bulk/Interrupt Normal TRB, Isoch TD (ring underrun/overrun), §3.3 Command Interface (liste des commandes, Table 3-1), §3.4-3.6, §4.1 Command Operation (DB0, Command Completion Event), §4.2 HOST CONTROLLER INITIALIZATION)
- **Point normatif critique (ligne 1767)** : après Chip Hardware Reset, attendre CNR=0 dans USBSTS AVANT toute écriture dans les registres Operational ou Runtime.
- **Ordre d'init (lignes 1771-1791)**, à compléter avant de mettre R/S=1 : CONFIG.MaxSlotsEn ; DCBAAP ; CRCR (pointeur du premier TRB de l'anneau de commande) ; MSI-X (table, PBA, Message Control) ; interrupteurs (Event Ring Segment(s), ERSTSZ, ERSTBA, ERDP) ; puis USBCMD.R/S=1.
- À comparer avec usb_r0.rs : CNR (ligne 110 signalée par l'agent), ordre CONFIG/DCBAAP/CRCR/ERST, MaxSlotsEn.

## Lignes 1801-2250 (§4.2 suite : Event Ring (ERSTBA écrit = active l'anneau), INTE, IE, R/S=1 ; §4.3 INITIALISATION D'UN DISPOSITIF : étapes 1-12 (CCS/CSC, PORTSC, Enable Slot, Input Context A0/A1, Slot Context (Root Hub Port Number, Route String, Context Entries=1), EP0 (EP Type Control, MaxPacketSize selon vitesse, TR Dequeue DCS=1, CErr=3), DCBAA[slot]=Output Context, Address Device BSR=0 ; GET_DESCRIPTOR 8 octets pour FS (Setup IDT, Data DIR=1, Status IOC=1, Evaluate Context A1 pour MaxPacketSize réelle) ; Configure Endpoint + SET_CONFIGURATION ; SET_INTERFACE ; §4.3.7 Split LS/FS (MTT, Parent Hub Slot ID, Parent Port Number, Hub flag) ; §4.3.9 eUSB2V2 ; §4.4 Device Detach (CCS=0, CSC=1, Disable Slot) ; §4.5 Device Slot : DCI = (EP*2)+Direction (OUT=0, IN=1), Control : DCI = EP*2+1 ; Slot Context init ; §4.5.3 états Disabled/Enabled/Default/Addressed/Configured)
- **Points de contrôle pour usb_r0.rs / usb_enum.rs** : (a) EP0 CErr=3 ; (b) DCI EP0 = 1 (contrôle) ; (c) Context Entries=1 à Address Device ; (d) Input Control Context A0=A1=1 ; (e) BSR=0 pour adresse normale ; (f) Evaluate Context A1 après lecture de bMaxPacketSize0 pour FS ; (g) attendre l'événement de complétion entre deux commandes sur le même slot ; (h) DCBAA[slot] pointe vers le Device Context de sortie, initialisé à zéro.

## Lignes 2251-2700 (§4.5.3 états de slot détaillés : Disabled (seule commande = Enable Slot), Enabled (Address Device, Disable Slot), Default (Address BSR=0, Evaluate, Reset/Stop Endpoint, Set TR Dequeue, Disable), Addressed (Configure Endpoint, Reset Device...), Configured ; §4.5.4 mapping requêtes USB (SET_ADDRESS bloqué sur EP0 -> TRB Error ; SET_CONFIGURATION avec Configure Endpoint ; SET_INTERFACE) ; §4.6 COMMAND INTERFACE : CRCR, un seul Command Ring, doorbell 0 (DB Target = commande) ; §4.6.1 Command Ring (CRR, CS, CA, Command Ring Stopped, Command Aborted) ; §4.6.2 No Op ; §4.6.3 Enable Slot (Slot Type de Supported Protocol, No Slots Available) ; §4.6.4 Disable Slot (DCBAA[slot] = 0 après complétion) ; §4.6.5 Address Device (BSR, SET_ADDRESS, Input Control A0/A1=1, Context State Error, Parameter Error))
- **Règle critique (ligne 2489)** : si CRR ne retombe pas dans les 5 s après CA, l'hôte doit assert HCRST.
- **Règle (ligne 2521)** : écrire DCBAA[slot] avant toute commande pour le slot ou sonnette sur EP0.
- **Règle (ligne 2590)** : après Disable Slot, remettre DCBAA[slot] à 0.
- Ces points sont à vérifier dans usb_enum.rs (commande Enable Slot / Address Device / Disable Slot).

## Lignes 2701-3150 (§4.6.5 suite Address Device : USB Device Address pool épuisé -> Resource Error ; BSR=0 : SET_ADDRESS (wValue=adresse choisie par xHC, wLength=0, Route String) ; §4.6.6 CONFIGURE ENDPOINT : DC=1 désactive DCI 2-31 et Context Entries=1 ; règles A0 obligatoire, A1/D0/D1=0 ; ordre D avant A ; checks ressources / bande passante (Resource Error, Bandwidth Error, Parameter Error si Add invalide, Context State Error si slot ≠ Addressed/Configured, Slot Not Enabled) ; succès : Output EP State = Running, Slot State = Configured (ou Addressed si tout désactivé), Context Entries = dernier DCI valide ; §4.6.6.1 ELD ; §4.6.7 EVALUATE CONTEXT (D = 0, A0/A1 ; Parameter Error ; Max Exit Latency Too Large ; Context State Error) ; §4.6.8 RESET ENDPOINT (TSP ; Halted -> Stopped ; Context State Error si non Halted) ; §4.6.8.1 Soft Retry)
- **Point de vérification** : Configure Endpoint avec Context Entries = dernier DCI valide ; DC=1 pour déconfigurer.
- **Règle** : Reset Endpoint seulement si EP State=Halted.
- À vérifier dans usb_enum.rs / usb_hid.rs : séquence Configure Endpoint puis SET_CONFIGURATION (et inverse pour Deconfigure).

## Lignes 3151-3600 (§4.6.8.1 Soft Retry (interdit sur Isoch et derrière TT) ; §4.6.9 STOP ENDPOINT (SP flag, Table 4-2, Transfer Event Stopped/FSE, Context State Error si non Running) ; §4.6.10 SET TR DEQUEUE POINTER (seulement si Stopped ou Error ; DCS ; Stream ID ; TRB Error si MaxPStreams>0) ; §4.6.11 RESET DEVICE (Slot -> Default, Context Entries=1, adresse=0, EP non-0 Disabled) ; Command Completion Event)
- Rappel : Stop/Reset Endpoint/Set TR Dequeue/Reset Device modifient l'état USB -> à ne pas envoyer sans accord utilisateur pour la partie sonde.

## Lignes 3601-4050 (§4.6.11 suite Reset Device ; §4.6.12 Force Event (VF) ; §4.6.13 Negotiate Bandwidth (BNC) ; §4.6.14 Set LTV (LTC) ; §4.6.15 Get Port Bandwidth (Dev Speed, Hub Slot ID, Port Bandwidth Context) ; §4.6.16 Force Header ; §4.6.17 Get Extended Property (GSC, Table 4-3 : bit0 Audio, bit1 Camera, bit 11 Time Stamp...) ; §4.6.18 Set Extended Property ; §4.7 DOORBELLS (DB0 = commande, DB1-255 = slots, DB Target/Stream ID ; ne pas écrire doorbell d'un slot Disabled ; Endpoint Not Enabled Error ; ignorer doorbells Halted/Error) ; §4.8 ENDPOINT ADDRESSING : DCI pour EP0 = IN (odd, DCI 1) ; pour contrôle bidirectionnel EP>0 : utiliser DCI impair, ignorer OUT pair ; §4.8.2 Endpoint Context init : EP0 CErr=3, MaxPacketSize selon bMaxPacketSize0)
- **Vérifier dans le code** : (a) EP0 DCI = 1 (et non 0 ni 2) ; (b) doorbell du slot écrite avec DB Target = DCI de l'EP (EP0 = 1) ; (c) ne jamais sonner un doorbell de slot désactivé.

## Lignes 4051-4500 (§4.8.2.3-5 Endpoint Context init : Bulk (CErr=3), Isoch (CErr=0, Mult, Max Burst), Interrupt (CErr=3, Max ESIT Payload) ; §4.8.3 ENDPOINT STATE (Disabled, Running, Halted sur Stall/USB Transaction Error/Babble, Stopped, Error sur TRB Error ; Reset Endpoint pour Halted->Stopped ; Stop sur Halted = Context State Error ; Stall EP0 -> Halted ; Doorbell sur Stopped -> Running) ; §4.9 TRB RING (TRB = 16 octets ; ring ne traverse pas 64K ; ring vide : Dequeue = Enqueue ; plein : Enqueue+1 = Dequeue ; PCS/CCS, Cycle bit ; Link TRB Toggle Cycle ; §4.9.2.2 Pointer advancement ; ring initialisé à zéro, PCS=1))
- **Règle critique (ligne 4226)** : TRB = 16 octets ; **ligne 4232** : un ring ne doit pas franchir une limite de 64 Ko.
- **Règle (ligne 4437)** : PCS et CCS initialisés à 1 ; bit cycle à 0 dans toutes les TRB au départ.
- **Vérifier** dans usb_r0.rs / usb_enum.rs : taille TRB 16 octets, pas de franchissement 64 Ko, Link TRB en fin de segment avec Toggle Cycle, PCS=1 initial, ring initialisé à zéro.
- Problème déjà signalé par l'agent : anneau de commandes sans Link TRB (usb_r0.rs l.400-406) -> point à confirmer par lecture du code.

## Lignes 4501-4950 (§4.9.2.3-4 agrandir/réduire un ring (Link TRB propriétaire seulement) ; §4.9.3 COMMAND RING (CRCR écrit ring arrêté -> Dequeue = Command Ring Pointer ; pas de TD multi-TRB ; Command Ring non vide ; CRR peut rester à 1) ; §4.9.4 EVENT RING (xHC = producteur, PCS initial 1, CCS logiciel initial 1, ERDP écrit par logiciel après traitement ; ERST : Base Address/Size, ERSTSZ, ERSTBA ; **Size de chaque segment >= 16** ; ordre d'écriture : segments à zéro, ERST, ERSTSZ, ERDP, puis ERSTBA ; Figure 4-12 machine d'état ; Event Ring Full Error ; écrire ERDP sans répéter la même valeur)
- **Règle critique** : ERSTBA écrit en dernier (il démarre la machine d'état de l'anneau d'événements).
- **Règle** : taille de segment d'événements >= 16 TRB.
- **Vérifier dans usb_r0.rs** : ordre d'écriture ERSTSZ / ERDP / ERSTBA, taille des segments, zeroing avant écriture, écriture ERDP après traitement (pas de doublon).

## Lignes 4951-5400 (§4.9.4 suite : Event Ring Full (xHC garde une place libre ; Event Ring Full Error ; secondaires) ; §4.9.4.1-2 agrandir/réduire ERST (ERSTSZ) ; §4.9.4.3 Primary/Secondary Event Rings (Interrupter 0 = primaire) ; §4.10 HOST CONTROLLER TRB HANDLING : §4.10.1 Transfer TRBs (IOC, ISP, Short Packet, Event Data, PAE) ; §4.10.1.1 Short Transfers ; §4.10.2 ERRORS : Stall -> Halted + Reset Endpoint ; TRB Error ; USB Transaction Error (CErr) ; Babble -> Halted (sans décrémenter le compteur) ; Data Buffer Error ; HSE (USBSTS) -> R/S=0 ; §4.10.2.7 Bus Error Counter (CErr))
- **Règle (ligne 5338-5341)** : contrôle logiciel des événements : pointeur TRB valide, slot activé, endpoint activé, champs cohérents.
- **Règle (ligne 5337)** : HSE -> le contrôleur met R/S=0 ; à traiter comme erreur fatale dans la sonde.
- **Vérifier** : traitement des Transfer Events, arrêt sur Halted, HSE/HCE lus dans USBSTS.

## Lignes 5401-5850 (§4.10.2.7-8 CErr : réinitialisé à CErr à chaque succès, Isoch n'a pas de retry ; Isoch Transaction Error -> Completion Babble ; §4.10.3 Ring Overrun/Underrun (IN=Overrun, OUT=Underrun ; TRB Pointer=Dequeue) ; Missed Service Error ; Split Transaction Error ; Short Packet ; §4.10.4 IOC ; §4.11 TRBs (template, Parameter/Status/Control, CH/ENT, **toutes les TRB initialisées à 0 à la création du ring**, atomicité 16 octets) ; §4.11.2 TRB de transfert (Normal, Setup/Data/Status : Setup IDT=1 Length=8 ; Table 4-7 DIR/TRT ; TRB Error pour SET_ADDRESS sur EP0 ; Stall = Halted ; §4.11.2.3 Isoch TRB : TBC/TLBPC/ FrameID / SIA / Microframe alignment))
- **Règle critique (ligne 5557 et 5779)** : les TRB de commande/transfert doivent être à 0 à la création du ring ; **un SET_ADDRESS envoyé sur EP0 par logiciel est refusé avec TRB Error** (l'xHC le bloque).
- **Règle (ligne 5566-5568)** : si les 4 dwords ne sont pas écrits atomiquement, écrire Parameter et Status AVANT le Control (qui contient le bit cycle).
- **Vérifier** dans usb_r0.rs : l'ordre d'écriture des TRB (Control en dernier) ; Setup TRB Length=8, IDT=1 ; DIR correct pour Data/Status ; pas de SET_ADDRESS envoyé par logiciel sur EP0.

## Lignes 5851-6300 (§4.9.1 Td / TD Size ; §4.11.2.4-5 TD Size, Frame ID (End/Start Frame ID, CFC), ESIT rules, Resynchronisation ; §4.11.3 Event TRB (IMPLEMENTATION NOTE : Dword 3 (cycle) écrite en dernier) ; §4.11.4 Command TRB (champs à zéro, C = PCS) ; §4.11.5 AUTRES TRB : **§4.11.5.1 LINK TRB** : règles de construction : segments de transfert alignés 16 o, **segments de commande alignés 64 o**, multiples de 16 o, **Link TRB = dernière TRB de chaque segment**, pointeur vers le segment suivant (ou le début du ring), le Link TRB du dernier segment pointe vers le début du ring, **Toggle Cycle dans au moins un Link TRB** ; §4.11.5.2 EVENT DATA TRB (EDTLA 24 bits) ; §4.11.6 vendor TRB ; §4.11.7 TD usage rules)
- **Règle critique (ligne 6107-6111)** : anneau de commande : Link TRB obligatoire en dernière position, pointant vers le début, Toggle Cycle positionné.
- **Règle (ligne 6107)** : segment de commande aligné 64 o.
- **Vérifier usb_enum.rs / usb_r0.rs** : l'anneau de commande doit se terminer par un Link TRB (Toggle Cycle=1) ; alignement 64 o ; sinon écriture hors anneau (déjà signalé par l'agent).

## Lignes 6301-6750 (§4.11.7 TD Usage : Link consécutives interdites ; TD Fragments (MBP = (MBS+1)*MPS, IOC placement, première TRB écrite en dernier) ; Figure 4-16 à 4-18 exemples ; §4.12 STREAMS (SS Bulk uniquement, IDs 1..65533, 0/65534/65535 réservés, MaxPStreams, Primary/Secondary Stream Array, Stream Context) ; §4.12.1 xHC Stream Protocol (xSPSM : Disabled, Prime Pipe, Idle, Start Stream, Move Data ; IPPV/LFCV/DBPV/DSICV ; HID))
- Règle : **la TRB de tête d'un TD est écrite en dernier** (Control avec bit cycle) pour que le contrôleur ne voie pas un TD partiel.
- Règle : **une seule Link TRB par TD**, pas de Link consécutives, pas de Link en première ou dernière position d'un TD multi-TRB.
- Pour l'agent : Streams non utilisés (MaxPStreams=0). Vérifier qu'il n'y a pas de Stream ID envoyé.

## Lignes 6751-7200 (§4.12.2 Stream ID management (PSID/SSID, Stream Context Type, bounds checking) ; §4.12.3 ENT ; §4.13 DEVICE NOTIFICATIONS (DNCTRL) ; §4.13.1 LTM (BELT, Set LTV) ; §4.13.2 FUNCTION_WAKE ; §4.14 MANAGING TRANSFER RINGS : §4.14.1 scheduling (SO, MSOPC, SOPC, TDPC, TRPC, Average TRB Length) ; §4.14.2 PERIODIC SCHEDULING : ESIT = 2^Interval × 125 µs ; Max ESIT Payload ; **Reserved BW = MaxESITPayload / (2^Interval × 0.000125)** ; **90 % max périodique** (FS/SS) ; MFINDEX bits 13:3 Frame, 2:0 µframe ; §4.14.2.1 Isoch ; §4.14.2.1.1 HS high-bandwidth (Max Burst Size max 3 pour HS)... )
- **Règle** : limite de 90 % de la bande passante périodique (FS/SS) = réservation ; utile pour l'agent si un Configure Endpoint échoue (Bandwidth Error).
- Non applicable à la sonde directe, mais utile pour comprendre les erreurs Bandwidth Error / Resource Error.

## Lignes 7201-7650 (§4.14.2.1.3-4 Isoch SS, IST (HCSPARAMS2, 1 microframe de marge) ; §4.14.3 INTERRUPT scheduling (IST + ESIT ; un SO par ESIT ; Table 4-8 USB2 / Table 4-9 USB3) ; §4.14.4 ASYNC scheduling (round-robin, Max Service Transfer Size = MPS*(MBS+1), MSOPC) ; §4.14.4.1 SS burst ; §4.15 SUSPEND-RESUME (PLS=3 U3 ; U3C/U3E CONFIG ; §4.15.1.1 SP flag ; §4.15.2 port resume (PLS=15 Resume, LFPS, 20 ms pour USB2))
- Règles pour l'agent : la sonde ne doit pas laisser de port en U3 sans reprise ; la reprise USB2 exige 20 ms (TDRSMDN) avant d'écrire PLS=0.
- Pas d'impact direct sur le code actuel sauf si usb_hid.rs gère les ports en suspension.

## Lignes 7651-8100 (§4.15.2.2-3 host/wakeup resume ; §4.16 BANDWIDTH (BI, Secondary Bandwidth Domain TT) ; §4.16.1 Negotiate ; §4.16.2 Bandwidth Domains ; §4.17 INTERRUPTERS (MaxIntrs, IMAN, IMOD, ERDP ; MSI/MSI-X ; INTx pour Interrupter 0 seulement ; **IMODI défaut = 4000 (1 ms)** ; formule Interrupts/s = 1/(250 ns × IMODI) ; §4.17.1-2 Interrupter Mapping ; §4.17.3-5 Interrupt Pin support / Target / Blocking ; Figures 4-22 à 4-24 Interrupt throttle)
- **Règle** : IMODI au reset = 4000 (1 ms) ; une valeur 0 désactive la modération.
- **Règle** : INTx# : seul l'Interrupter 0 ; MSI-X : Table size = MaxIntrs.
- **Règle (IMAN)** : IP effacé par écriture de IMAN (INTx) ; par la complétion PCI (MSI/MSI-X).
- **Vérifier usb_r0.rs** : IMOD (IMODI), IMAN.IE / IMAN.IP acquittement, interruptions désactivées ou non.

## Lignes 8101-8550 (§4.17.3-5 Interrupt pin (INTx seulement Interrupter 0 ; PCI Interrupt Pin register = 0 si pas d'INTx) ; Interrupter Target (Primary fixe / TRB / Slot Context) ; **Interrupt Blocking** (BEI, IPE) ; §4.18 NO SNOOP / RELAXED ORDERING (Table 4-11) ; §4.19 ROOT HUB : §4.19.1.1 USB2 PORTS (Powered-off -> Disconnected -> Disabled -> Reset -> Enabled (U0/U2/U3/Resume) ; PR=1 pour reset ; PED ; PRC=1 à la fin du reset ; PP=0 ou OCA -> Powered-off) ; §4.19.1.2 USB3 PORTS (Disconnected -> Polling -> Enabled ; RxDetect ; Loopback ; Compliance ; Error ; HCRST -> Disconnected ; Figure 4-27))
- **Règle** : pendant l'état Reset USB2, PLS non significatif ; reset USB2 : PR=1 puis attendre PRC=1.
- **Règle** : une déconnexion USB2 n'est pas détectable pendant le Reset (le contrôleur conduit le bus).
- **Règle (HCRST)** : après HCRST, tous les ports repartent en Disconnected.
- À vérifier dans usb_r0.rs / usb_hub.rs : séquence PR -> attente PRC -> lecture de PORTSC (vitesse), gestion Powered-off si PP=0.

## Lignes 8551-9000 (§4.19.1.2 USB3 ports : Disconnected (HCRST garde Disconnected ; Connect Detect -> Polling) ; Polling : Training / CfgExcg / DbC ; Config Error -> Error ; Reset (PR/WPR) ; Error ; Compliance ; Loopback ; Enabled : U0 / U1' / U2' / U3' / Recovery ; §4.19.1.2.13 U3' ; §4.19.2 PORT STATUS CHANGE : bits CSC, PEC, WRC, OCC, PRC, PLC, CEC ; effacés par écriture 1 ; PSCEG -> Port Status Change Event ; PME# ; ALIGNEMENT : pas de cohérence entre lecture PORTSC et événement)
- **Règle** : pour acquitter un changement, écrire 1 dans le bit changement correspondant de PORTSC (ne pas effacer PP/PR par erreur : ces bits sont écrits à 1 pour ne pas réinitialiser).
- **Règle (ligne 8956)** : ne pas supposer que l'événement est déjà dans l'anneau quand PORTSC montre un changement.
- Pour la sonde : relire PORTSC après le Port Status Change Event avant de conclure.

## Lignes 9001-9450 (§4.19.2 PORT STATUS CHANGE GENERATION (PSCEG, HCH, PME#) ; §4.19.3 connect reporting ; §4.19.4 PORT POWER (PP=1 automatique après reset ; USB3 : Disconnected -> Polling ; PP=0 -> Powered-off) ; §4.19.4.1 U1/U2 timeouts (PORTPMSC) ; §4.19.5 PORT RESET (PR=1 -> PED=0 ; fin : PLS=U0, PED=1, PRC=1, Port Speed valide seulement après PR 1->0) ; §4.19.5.1 Warm Reset (WPR, WRC) ; §4.19.6 **PORT TEST MODES** (USB2 seulement, séquence : désactiver slots, PP=0, R/S=0 et attendre HCH, Port Test Control, HCRST pour sortir) ; §4.19.7 PORT ROUTING (Supported Protocol : Compatible Port Offset/Count) ; §4.19.8 CAS (D3) ; §4.19.9 PORT SPEED (SSP, Sublink Speed Device Notification) ; §4.20 SCRATCHPAD (HCSPARAMS2, DCBAA[0], SPR, PAGESIZE aligné, alloué AVANT R/S=1) ; §4.21 PCI (BME : R/S=0 et HCH=1 avant de couper BME ; sinon HCE) ; §4.22.1 OS HANDOFF (USBLEGSUP : BIOS Owned bit16, OS Owned bit24 ; machine d'état ; OS demande : OS Owned=1 puis attend BIOS Owned=0 au plus 1 s))
- **Point critique pour l'agent** : c'est une application pré-OS. Si le firmware possède le contrôleur (BIOS Owned=1), la procédure correcte est : mettre OS Owned=1, attendre BIOS Owned=0 (max 1 s), **et ne pas supposer que OS Owned n'est posé que si BIOS Owned est déjà 1**. À comparer avec usb_r0.rs l.89-91 (constat de l'agent à vérifier).
- **Scratchpad** : allouer et initialiser à zéro les pages avant R/S=1 ; DCBAA[0] = pointeur vers le tableau de scratchpad.
- **Test modes** : USB2 uniquement ; ne pas utiliser pour la sonde.

## Lignes 9451-9900 (§4.22.2-3 renvois ; §4.23 POWER MANAGEMENT : §4.23.1 POWER WELLS (Aux : reset seulement à power-up ou HCRST ; Core : Chip Hardware Reset/HCRST/D3hot->D0 ; LHCRST ne touche pas Aux ; USBLEGSUP exception) ; §4.23.2 **SAVE/RESTORE STATE** (étapes 1-10 de sauvegarde : Stop Endpoint, Command Ring arrêté, R/S=0, lire USBCMD/DNCTRL/DCBAAP/CONFIG/ERSTSZ/ERSTBA/ERDP/IMAN/IMOD, CSS=1 attendre SSS=0, scratchpad si SPR=1, DCBAA/contextes ; restauration étapes 1-10 : écrire registres AVANT CRS=1, réinitialiser Command Ring, CRCR, R/S=1, réinitialiser PORTSC, relancer doorbells) ; §4.23.2.1 ; §4.23.3 PCI PM ; §4.23.4 USB PM ; §4.23.5 LPM : Table 4-12 (L0/U0, L1/U2, L2/U3) ; USB2 LPM : PLS=2 avec LWS -> transaction LPM ; L1S (Success 1, Not Yet 2, Not Supported 3, Timeout/Error 4) ; HLE, HLC, BLC ; HIRD vs BESL ; DBESL/DBESLD)
- **Règle critique (étape 5-7 et restore 4-5)** : écrire les registres de restauration **avant** CRS=1 ; après restauration réécrire CRCR ; R/S=1 en dernier.
- Pour l'agent : pas de Save/Restore à faire (pas de D3) sauf demande explicite.
- LPM : ne jamais écrire PLS=2 (U2) pour la sonde.

## Lignes 9901-10350 (§4.23.5.1 HIRD/BESL : Table 4-13 (BESL/HIRD encodage, 0..15) ; LPM sans remote wake : éviter HIRDM=1 ; §4.23.5.2 MAX EXIT LATENCY (Slot Context, TL1ExitLatency = BESL + 10 µs ; 0 = pas de gestion) ; §4.23.5.2.1 No Ping Response Error ; §4.23.5.2.2 Max Exit Latency Too Large (CME/CMC) ; §4.24 HOST CONTROLLER MANAGEMENT : §4.24.1 **HCE** (cesse toute activité, réinitialiser HCRST puis réinitialiser ; si HCE réapparaît : erreur matérielle) ; §4.24.2 Port to Connector Mapping ; §4.25 USB VTIO (DMA-ID alternatif, VTIOCAP/VTIOCA1, pages distinctes par DMA-ID))
- **HCE** : détecter et traiter comme erreur fatale dans la sonde ; signaler si HCE persiste après HCRST.
- VTIO : optionnel, ne pas activer.

## Lignes 10351-10650 (§4.25.1-2 VTIO : DMA-ID setup (Table 4-14) ; FIN §4.x ; **CHAPITRE 5 REGISTER INTERFACE** : §5.1 conventions (écriture 64 bits en Qword ou 2 Dword bas puis haut si AC64=1 ; little-endian) ; §5.1.1 attributs (RO, RW, RW1C, RW1S, RWS, RW1CS, Rsvd, RsvdO, RsvdP, RsvdZ) ; §5.2 PCI CONFIG (Type 0 header ; BAR0/BAR1 pour MMIO 64 bits) ; **Class Code = 0C0330h (PI 30h = xHCI, sub 03h USB, base 0Ch serial bus)** ; **SBRN 60h = 30h (USB 3.0)** ; **FLADJ 61h, défaut 20h, modifiable uniquement si HCH=1**, RWS dans Aux Power)
- Vérification à faire : usb_r0.rs doit lire BAR0 (MMIO) et non BAR1 seul ; pour xHCI la classe 0C0330 sert à identifier le contrôleur (vérifier usb_enum / cpu : décrire_pci).
- FLADJ ne doit pas être modifié pendant R/S=1 (la sonde ne doit pas le toucher).

## Lignes 10651-10950 (§5.2.4 FLADJ : bits 5:0 timing, NFC bit 6, SOF = 59488 + valeur, défaut 20h = 60000 ; §5.2.5-6 DBESL (62h bits 3:0) / DBESLD (bits 7:4) ; §5.2.7 PCI PM capability : PMCSR, NO_SOFT_RESET)
- Vérification : FLADJ = 60000 par défaut (20h). Si le code modifie FLADJ, c'est hors spec sauf HCH=1.

## Lignes 11101-11400 (§5.2.8.5 accès MSI-X : entrée K à base + K×16 ; PBA Qword = base + (K div 64)×8 ; §5.2.9 PCIe Capability ; §5.2.10 SR-IOV ; **§5.3 CAPABILITY REGISTERS** (tous RO, offsets par rapport à Base ; table 5-9 : CAPLENGTH 00h (1 o), HCIVERSION 02h (2 o), HCSPARAMS1 04h, HCSPARAMS2 08h, HCSPARAMS3 0Ch, HCCPARAMS1 10h, DBOFF 14h, RTSOFF 18h, HCCPARAMS2 1Ch ; §5.3.1 CAPLENGTH = décalage de l'espace Operational ; §5.3.2 HCIVERSION BCD : 0100h = 1.0.0, 0110h = 1.1.0 ; §5.3.3 HCSPARAMS1 : MaxSlots bits 7:0, MaxIntrs 18:8?, MaxPorts 31:24)
- Vérification : usb_r0.rs lit hcs1 bits 7:0 = MaxSlots (correct) et bits 31:24 = MaxPorts (correct). Vérifier MaxIntrs (bits 18:8 à confirmer dans la table 5-10 suivante).

## Lignes 11401-11850 (§5.3.3 HCSPARAMS1 : MaxSlots 7:0, **MaxIntrs 18:8**, MaxPorts 31:24 ; §5.3.4 HCSPARAMS2 : IST 3:0, ERST Max 7:4, **Max Scratchpad Hi 25:21, SPR bit 26, Max Scratchpad Lo 31:27** ; §5.3.5 HCSPARAMS3 : U1 Device Exit Latency 7:0, U2 Device Exit Latency 31:16 ; §5.3.6 HCCPARAMS1 : **AC64 bit 0, BNC bit 1, CSZ bit 2 (1 = contextes de 64 octets, 0 = 32 octets), PPC bit 3, PIND bit 4, LHRC bit 5, LTC bit 6, NSS 7, PAE 8, SPC 9, SEC 10, CFC 11, MaxPSASize 15:12, xECP 31:16 (offset en dwords depuis Base)** ; §5.3.7 DBOFF : offset tableau de doorbells, bits 31:2)
- **CONTRÔLE IMPORTANT** : la taille des contextes dépend de CSZ (bit 2 HCCPARAMS1). Si le code suppose 32 o alors que CSZ=1, toutes les adresses de contextes sont fausses. À vérifier dans usb_r0.rs / usb_enum.rs.
- Vérifier aussi : MaxIntrs (18:8), SPR (bit 26), Max Scratchpad (hi 25:21 / lo 31:27) — déjà confirmé dans usb_r0.rs.

## Lignes 11851-12300 (§5.3.7-10 DBOFF (bits 31:2, dwords ; PAGESIZE-aligné si virtualisation) ; RTSOFF (bits 31:5, 32 o) ; **HCCPARAMS2** : U3C 0, CMC 1, FSC 2, CTC 3, LEC 4, CIC 5, ETC 6, ETC_TSC 7, GSC 8, VTC 9, DIC 11, E2V2C 12 ; VTIOSOFF ; §5.4 OPERATIONAL REGISTERS (base = CAPLENGTH) : table 5-18 : USBCMD 00h, USBSTS 04h, PAGESIZE 08h, DNCTRL 14h, CRCR 18h, DCBAAP 30h, CONFIG 38h, ports à 400h + 10h×(port-1) : PORTSC +0, PORTPMSC +4, PORTLI +8, PORTHLPMC +Ch ; §5.4.1 USBCMD bits : R/S 0, HCRST 1, INTE 2, HSE 3, LHCRST 7, CSS 8, CRS 9, EWE 10, EU3S 11, CME 13, ETE 14, TSC_EN 15, VTIOE 16)
- **Point de vérification** : les registres de port commencent à 400h (table 5-18) ; contrôler dans usb_r0.rs / usb_hid.rs l'adresse de PORTSC (base + CAPLENGTH + 400h + 16×(n-1) -> attention : 16 o par port ? à confirmer dans la table 5-19 : 10h par port).
- USBCMD : R/S bit 0, HCRST bit 1 — le code doit attendre HCRST=0 et CNR=0.

## Lignes 12301-12750 (§5.4.1 USBCMD détaillé : R/S bit0 (arrêt < 16 ms), **HCRST bit1 (ne pas écrire les registres Operational/Runtime tant que HCRST=1)**, INTE bit2, HSEE bit3, LHCRST bit7, CSS bit8, CRS bit9, EWE bit10, EU3S bit11, CME bit13, ETE bit14, TSC_EN bit15, VTIOE bit16 ; §5.4.1.1 halt procedure ; **§5.4.2 USBSTS** : HCH bit0 (RO, défaut 1), HSE bit2 (RW1C), EINT bit3 (RW1C), PCD bit4 (RW1C), SSS bit8, RSS bit9, SRE bit10 (RW1C), **CNR bit11 (défaut 1 ; 0 = prêt)**, **HCE bit12 (RO)** ; §5.4.3 PAGESIZE)
- **Point critique confirmé** : CNR (bit 11 USBSTS) = 1 au départ ; ne rien écrire (sauf USBSTS) tant que CNR=1. Le constat de l'agent (CNR non attendu, usb_r0.rs l.110) est confirmé par la spec (ligne ~12710).
- Acquittement : écrire 1 dans les bits RW1C (HSE, EINT, PCD, SRE) ; ne pas écrire 1 dans les autres bits.
- Si HCE=1 : réinitialiser via HCRST.

## Lignes 12751-13150 (§5.4.3 PAGESIZE (bit n = page 2^(n+12)) ; §5.4.4 DNCTRL (N0..N15, défaut 0 ; N1 FUNCTION_WAKE) ; §5.4.5 **CRCR** : RCS bit0, CS bit1, CA bit2, CRR bit3 (RO), Command Ring Pointer bits 63:6 (**anneau 64 o aligné, bits 5:0 = 0**) ; §5.4.6 **DCBAAP** : 64 o aligné ; §5.4.7 **CONFIG** : MaxSlotsEn 7:0, U3E bit8, CIE bit9, SOC bit10 (MaxSlotsEn ne doit pas être modifié si R/S=1) ; §5.4.8 **PORTSC** : adresse = Operational + 400h + 10h×(n−1) ; CCS bit0 ; PED bit1 ; attendre 20 ms après PP=1 (si PPC=1) ; PORTSC inaccessible si HCH=1 (ligne 13121))
- **Vérifications** : (a) CRCR pointeur aligné 64 o ; (b) DCBAAP aligné 64 o ; (c) PORTSC à 400h+10h×(n−1) ; (d) écrire CONFIG.MaxSlotsEn avant R/S=1 ; (e) 20 ms après PP=1 si PPC=1.

## Lignes 13151-13600 (§5.4.8 PORTSC suite : CCS bit0 ROS ; PED bit1 RW1CS (écrire 1 pour désactiver, jamais pour activer) ; TM bit2 ; OCA bit3 RO ; **PR bit4 RW1S** (1 = reset, pas de "PR=1 et PED=1" en même temps) ; **PLS bits 8:5 RWS** (0 U0, 2 U2 USB2, 3 U3, 5 RxDetect (si Disabled), 10 Compliance, 15 Resume) ; **PP bit9 RWS (défaut 1)** ; **PS bits 13:10 ROS (vitesse)** ; PIC 15:14 ; **LWS bit16** (doit être 1 pour écrire PLS) ; **CSC bit17 RW1CS** ; PEC bit18 ; WRC bit19 ; OCC bit20 ; PRC bit21 ; PLC bit22 ...)
- **Règle d'écriture PORTSC** : écrire PR/PLS/PP avec précaution ; les bits RW1C (CSC, PEC, WRC, OCC, PRC, PLC, CEC) doivent être acquittés par écriture de 1, sinon les autres 1 lus seraient effacés par erreur (écrire 0 aux bits change non concernés).
- **Vérifier usb_hid.rs / usb_hub.rs** : la lecture de PORTSC doit masquer/préserver ces bits correctement lors d'une écriture.

## Lignes 13601-14050 (§5.4.8 PLC conditions (Table) ; CEC bit23 ; CAS bit24 ; WCE 25, WDE 26, WOE 27 (RWS) ; DR bit30 ; WPR bit31 (RW1S, lu à 0) ; Table 5-28 mapping USB2/USB3 ; §5.4.9 **PORTPMSC** = Operational + 404h + 10h×(n−1) : USB3 : U1 Timeout 7:0, U2 Timeout 15:8, FLA bit16 ; USB2 : L1S 2:0 (1 Success, 2 NYET, 3 STALL, 4 Timeout/Error), RWE bit3, BESL 7:4, L1 Device Slot 15:8, HLE bit16, ECM 26?, **Port Test Control 31:28** ; §5.4.10 **PORTLI** = 408h + 10h×(n−1))
- Vérification : pour lire la vitesse, utiliser PORTSC bits 13:10 ; pour LPM ne rien écrire (HLE, BESL) à la sonde.
- Test Control (31:28) : ne jamais écrire de valeur non nulle (mode test), hors sujet.

## Lignes 14051-14350 (§5.4.10 à §5.5.2)
- PORTLI USB3 (Table 5-31) : 15:0 Link Error Count RW (reset HCRST, PR 1->0, ou écriture 0) ; 19:16 RLC (zero-based, valide si CCS=1) ; 23:20 TLC ; 31:24 RsvdP.
- PORTLI USB2 (Table 5-32) : seulement si E2V2C=1, sinon RsvdP. 3:0 RDR, 7:4 TDR, 31:8 RsvdP.
- PORTHLPMC (USB2) à 40Ch + 10h(n-1), RWS, Aux power well : 1:0 HIRDM ; 9:2 L1 Timeout (00h=128 µs, pas 128 µs par unité) ; 13:10 BESLD ; 31:14 RsvdP. Valide seulement si HLC=1.
- PORTEXSC (USB3, LSECC=1) : 15:0 Link Soft Error Count RW ; 31:16 RsvdP. Note : le texte donne aussi 40Ch pour PORTEXSC (même adresse que PORTHLPMC, ce qui est incohérent) — à vérifier avant utilisation.
- MFINDEX (Runtime Base + 0000h) : 13:0 Microframe Index RO, +1 chaque microframe (125 µs), incrémente seulement si R/S=1.
- Interrupter Register Set n : Runtime Base + 020h + 32*n (n 0..1023). Offsets : 00h IMAN (IP bit0 RW1C, IE bit1 RW), 04h IMOD (intervalle 15:0, compteur 31:16), 08h ERSTSZ, 10h ERSTBA (64 bits), 18h ERDP (64 bits ; DESI 2:0, EHB bit 3).
- Règle : tous les registres de l'interrupteur primaire doivent être initialisés AVANT R/S=1 ; secondaires peuvent l'être après, mais avant tout événement qui les vise.

## Lignes 14501-14950 (§5.5.2 IMAN/IMOD/ERST, §5.6 Doorbells, §5.7 VTIO début)
- IMAN (Runtime+020h+32n) : bit0 IP RW1C, bit1 IE RW. Écrire 1 sur IP efface (si pas MSI/MSI-X).
- IMOD (Runtime+024h+32n), défaut 4000 (~1 ms). 15:0 IMODI en pas de 250 ns ; 0 = pas de throttling. 31:16 IMODC compteur. Formule : interrupts/s = 1/(250ns × IMODI). Max 8000/s si IMODI=500 => 125 µs.
- ERSTSZ (Runtime+028h+32n) : 15:0. Primaire ne peut pas être désactivé (0 = undefined). Secondaire : 0 désactive. Max = ERST Max (HCSPARAMS2).
  Rappel : allouer le tableau ERST arrondi au 64 octets.
- ERSTBA (Runtime+030h+32n, 64 bits) : bits 63:6 ; 5:0 RsvdP. L'écriture remet la machine d'état EREP à Start. Primaire : ne modifier que si HCH=1 (halted). Ordre : ERSTSZ avant ERSTBA.
- ERDP (Runtime+038h+32n, 64 bits) : 2:0 DESI (indice segment, alias si ERSTSZ>8), bit3 EHB RW1C (mis à 1 quand IP=1, effacé par écriture du pointeur), 63:4 pointeur. Ne pas écrire deux fois la même valeur sauf transition FULL->EMPTY.
- Doorbell array : base = Base + DBOFF ; 1 registre 32 bits par slot ; MaxSlots dans HCSPARAMS1.
  - DB 0 = Command ring (DB Target=0 ; le DB Stream ID doit être 0).
  - DB n (1..255) : DB Target 7:0 : 1 = EP0 enqueue update, 2/3 = EP1 OUT/IN, … 30/31 = EP15 OUT/IN ; 248..255 vendor.
  - DB Stream ID 31:16 (ignoré si MaxPStreams=0 et ≠0).
  - IMPORTANT : ne jamais sonner la doorbell d'un endpoint avant Configure Endpoint + Command Completion Event succès.
- VTIO (§5.7) : registres VTIOCAP (RO, PDMAID 15:0 / ADMAID 31:16 au format BDF), VTIOCA1 (RW, core power). Non utilisé par notre agent.

## Lignes 14951-15400 (§5.7 VTIO fin ; §6 Data Structures début ; §6.1 DCBAA ; §6.2.1 Device Context)
- VTIO : VTIOCA1 bits CRDIDA(1), DCBAADIDA(2), SPBADIDA(3), SPBDIDA(4), ICDIDA(6), MSIDIDA(7), PBDIDA(8), DCDIDA(9), EPDIDA(10). VTIODA 1..8 (bit n = slot n). VTIOIA 1..32 (bit 0 = primaire). VTIOEA @ 100h+4*(slot-1), bits DCI 1..31. Hors périmètre agent (sans virtualisation). Ne rien écrire.
- Règle globale §6 : aucune structure ≤64 KB ne traverse une frontière 64 KB ; aucune structure ≤ PAGESIZE ne traverse une frontière PAGESIZE. Lire PAGESIZE (MMIO Operational+08h) avant d'allouer.
- DCBAA (§6.1) : MaxSlotsEn+1 entrées, 64 octets alignés, contiguës dans une page, max 256 entrées = 2 KB. Écrite dans DCBAAP avant R/S=1. Entrées slot non alloué = 0. Ne pas modifier une entrée pendant que le slot est enabled.
  - Entrée 0 : si Max Scratchpad Buffers > 0 → pointeur vers le Scratchpad Buffer Array (bits 5:0 RsvdZ) ; sinon entrée 0 réservée = 0.
- Device Context (§6.2.1) : 32 entrées (Slot + 31 EP). CSZ=0 : 32 octets par contexte ; CSZ=1 : 64 octets. Offsets : EP0 à 020h (CSZ=0) / 040h (CSZ=1), EP DCI n à (n+1)×ctxsize.
  - Output Device Context : propriété passée au xHC à la première sonnerie de la doorbell commande pour Address Device ; doit être mis à zéro avant. Rendu au logiciel quand le slot passe Disabled.
  - Software ne doit pas écrire le Device Context pendant que le xHC le possède.
- Tailles/alignements (Table 6-1, texte partiellement mélangé dans l'extraction) : Input Control Context 64 octets, Slot 32 ou 64, EP 32/64, Stream Context 16, Stream Array linéaire 1M, Transfer/Command/Event ring segments 64 K max ; ERST 512 K max ; Scratchpad Buffer Array ≤248 octets (donc 8×N arrondi), Scratchpad Buffers = PAGESIZE alignés.
- Vérification agent : usb_r0.rs doit écrire DCBAA[0] = scratchpad array AVANT R/S=1, et le mettre à 0 si Max Scratchpad=0. Doit contrôler que DCBAA et buffers ne chevauchent pas la zone DMA.

## Lignes 15401-15850 (§6.2.2 Slot Context ; §6.2.2.1-4 usage ; §6.2.3 Endpoint Context début)
- Slot Context (32 o si CSZ=0, 64 o si CSZ=1 ; bytes 32-63 RsvdO) :
  - DW0 : 19:0 Route String ; 23:20 Speed ; 25 MTT ; 26 Hub ; 31:27 Context Entries (1..31, 0 réservé).
  - DW1 : 15:0 Max Exit Latency (µs) ; 23:16 Root Hub Port Number (1..MaxPorts) ; 31:24 Number of Ports (si Hub=1, sinon 0).
  - DW2 : 7:0 Parent Hub Slot ID ; 15:8 Parent Port Number ; 17:16 TTT (si HS hub) ; 31:22 Interrupter Target (0..MaxIntrs-1).
  - DW3 : 7:0 USB Device Address (entrée à 0) ; 31:27 Slot State (0 Disabled/Enabled, 1 Default, 2 Addressed, 3 Configured).
- Address Device : Slot Context input doit avoir Context Entries = 1, Root Hub Port 1..MaxPorts ; Output Slot doit être Disabled avant la première commande (zéro-initialisé).
- Configure Endpoint : Context Entries = index du dernier EP valide ; Slot doit être Addressed ou Configured sinon Context State Error.
- Evaluate Context : seuls Interrupter Target et Max Exit Latency sont évalués (si flag A0).
- Reset Device : Context Entries remis à 1 et Slot State à Default.
- Endpoint Context (32 o/64 o) :
  - DW0 : 2:0 EP State (0 Disabled, 1 Running, 2 Halted, 3 Stopped, 4 Error) ; 7:3 RsvdZ ; 9:8 Mult (0..2 ; LEC=1 → RsvdZ et calculé) ; 14:10 MaxPStreams ; 15 LSA ; 23:16 Interval ; 31:24 Max ESIT Payload Hi.
  - DW1 : 2:1 CErr ; 5:3 EP Type ; 7 HID ; 15:8 Max Burst Size ; 31:16 Max Packet Size.
  - DW2 : bit0 DCS ; 63:4 TR Dequeue Pointer.
  - DW4 : 15:0 Average TRB Length ; 31:16 Max ESIT Payload Lo.
- Vérification agent : CErr doit être 3 pour les EP de contrôle/bulk/interrupt usuels (à vérifier dans usb_enum.rs) ; EP0 DCI=1 ; Context Entries doit suivre le plus haut DCI configuré.

## Lignes 15851-16300 (§6.2.3 EP Context suite ; §6.2.3.1-8 ; §6.2.4 Stream Context début)
- MaxPStreams 14:10 : 0 = TR Dequeue pointe un Transfer Ring ; >0 = pointe un Stream Context Array. LSA bit15 (0 si MaxPStreams=0).
- Interval 23:16 : période = 125 µs × 2^Interval. Pour FS/LS interrupt et FS isoch, traduire bInterval (Table 6-12) : FS/LS Interrupt bInterval 1-255 → Interval 3-10 (arrondi bas à la puissance de 2 de bInterval×8) ; FS Isoch 1-16 → 3-18 ; SS/HS interrupt/isoch 1-16 → Interval 0-15. Bulk/Control SS : Interval non utilisé.
- Max ESIT Payload Hi 31:24 (si LEC=1), Lo 31:16 du DW4. LEC=0 → max 48 KB.
- CErr : 2:1 du DW1. Doc : mettre 3 pour usage normal ; 1 ou 2 réduit la fiabilité ; 0 = pas de compte d'erreurs (test/debug uniquement). CErr=0 pour Isoch.
- EP Type 5:3 : 0 Not Valid, 1 Isoch Out, 2 Bulk Out, 3 Int Out, 4 Control (bidir), 5 Isoch In, 6 Bulk In, 7 Int In.
- HID bit 7 (Host Initiate Disable, streams uniquement).
- Max Burst Size 15:8 (zero-based) : 0 pour FS/LS et HS control/bulk ; HS interrupt = bits 12:11 de wMaxPacketSize ; HS isoch via companion ; SS = bMaxBurst du Companion Descriptor.
- Max Packet Size 31:16 : valeur linéaire en octets (PAS l'encodage wMaxPacketSize). Pour HS/SS isoch avec eUSB2 companion : wMaxPacketSize ; sinon bits 10:0 de wMaxPacketSize.
- DCS bit0 du DW2 : doit être 0 si MaxPStreams>0 ; TR Dequeue Pointer 63:4 : aligné 16 octets.
- Average TRB Length : 15:0 du DW4, >0 ; **8 pour les endpoints de contrôle** (à vérifier dans usb_enum.rs).
- Address Device : seul DCI=1 est évalué ; EP Type = Control ; DCS = 1 ; MaxPStreams = 0 ; la Max Packet Size initiale doit être le défaut selon vitesse (8 octets LS/FS, 64 HS, 512 SS). Puis Evaluate Context pour corriger MPS après lecture du Device Descriptor.
- Configure Endpoint : DCI 1 non référencé ; EP doit être Disabled dans l'Input.
- Evaluate Context : seul le Max Packet Size du DCI1 est évalué (flag A1) ; EP State doit être Running:Idle ou Stopped.
- Reset Device : EP0 conserve Max Packet Size, EP Type, CErr, TR Dequeue, Average TRB ; EP State → Running ; le reste à 0.
- Stream Context (16 octets) : DW0 bit0 DCS ; 3:1 SCT (0 Secondary TR, 1 Primary TR, 2..7 Primary SSA de 8..256 entrées, puissance de 2) ; TR Dequeue Pointer 63:4. Stream ID 0 réservé.

## Lignes 16301-16750 (§6.2.4.1 suite Stream Context ; §6.2.5 Input Context ; §6.2.5.1 Input Control ; §6.2.6 Port Bandwidth ; §6.2.7 ; §6.3-6.4 TRB début ; Normal TRB)
- Stream Context : DW1 23:0 Stopped EDTLA (si SEC=1) ; 16 octets toujours (CSZ ne s'applique pas). Valide si TR Dequeue=0 (non assigné) ou pointe un TR/SSA valide.
- Input Context : tableau de 33 entrées. Offset 0 = Input Control Context ; slot à ICI 1 ; EP à ICI = DCI+1. Taille d'entrée = 32 o (CSZ=0) ou 64 o (CSZ=1). Contiguë dans une page.
- Input Control Context : DW0 = Drop flags D2..D31 (bits 31:2, bits 1:0 RsvdZ) ; DW1 = Add flags A0..A31 ; DW7 (offset 1Ch) = 7:0 Configuration Value, 15:8 Interface Number, 23:16 Alternate Setting (renseignés si CIE=1 et SET_CONFIGURATION/SET_INTERFACE).
  - Indices : Control EP → ICI = (ep+1)×2 ; Isoch/Int/Bulk → ICI = ep×2 + 1 + dir (OUT=0, IN=1).
  - Le champ Input Control ne doit pas être modifié entre l'envoi de la commande et l'événement de complétion.
- Port Bandwidth Context : 8 octets arrondis, 1 octet par port (0 à 80% HS, 0 à 90% SS/FS, en hexa décimal).
- Extended Property Context : aligné 64 octets, contigu dans une page.
- TRB : aligné 16 octets. Un tampon de données ne doit pas traverser 64 KB ; sinon chaîner.
- Normal TRB (§6.4.1.1) : DW0 = Data Buffer Pointer (64 bits, ou données immédiates si IDT). DW2 : 16:0 TRB Transfer Length (0..64K), 21:17 TD Size, 31:22 Interrupter Target. DW3 : bit0 C (cycle), 1 ENT, 2 ISP, 3 NS, 4 CH (chain), 5 IOC (interrupt on completion), 6 IDT, 15:10 TRB Type (dans l'extraction, le type est bien dans bits 15:10, à confirmer dans §6.4 Table).

## Lignes 16751-17350 (Normal TRB suite ; §6.4.1.2 Control TRBs : Setup, Data, Status ; §6.4.1.3 Isoch TRB début)
- Normal TRB DW3 : bit5 IOC, bit6 IDT (si IDT: longueur 0..8 ; IDT interdit si MPS<8 ou endpoint IN), bit9 BEI, 15:10 TRB Type (Normal = 1).
- **Setup Stage TRB** (DW0 = bmRequestType 7:0, bRequest 15:8, wValue 31:16 ; DW1 = wIndex 15:0, wLength 31:16 ; DW2 = TRB Transfer Length **toujours 8**, Interrupter Target 31:22 ; DW3 : bit0 C, bit5 IOC, bit6 **IDT=1 obligatoire**, 15:10 Type = Setup (2), **17:16 TRT** (0 No Data, 2 OUT Data, 3 IN Data ; 1 réservé)).
- **Data Stage TRB** : DW0-1 pointeur buffer ; DW2 : 16:0 Transfer Length (1..64K), 21:17 TD Size, 31:22 Interrupter Target ; DW3 : bit0 C, bit1 ENT, bit2 ISP, bit3 NS, bit4 CH, bit5 IOC, bit6 IDT, 15:10 Type = Data (3), **bit16 DIR** (0 OUT/Write, 1 IN/Read).
- **Status Stage TRB** : DW2 : 21:0 RsvdZ, 31:22 Interrupter Target ; DW3 : bit0 C, bit1 ENT, bit4 CH, bit5 IOC, 15:10 Type = Status (4), **bit16 DIR** (0 OUT host→dev, 1 IN dev→host). IOC normalement uniquement sur le Status Stage.
- Isoch TRB (début) : DW2 : 16:0 Transfer Length (0..64K), 21:17 TD Size ou TBC (si ETE=1), 31:22 Interrupter Target. DW3 : bit6 IDT, bit9 BEI, bits 19:16 TBC/TRBSts, Frame ID 30:20, SIA bit31 ; 15:10 Type = Isoch (8).
- Vérification agent : usb_enum.rs ne doit pas utiliser TRB Transfer Length 8 sur autre chose que Setup ; TRT doit correspondre (No Data = 0 si wLength=0) ; IDT=1 obligatoire dans le Setup TRB ; DIR de Data/Status doit être inversé par rapport au sens.

## Lignes 17351-17950 (Isoch TRB suite ; No Op TRB ; §6.4.2 Event TRBs : Transfer, Command Completion, Port Status Change, Bandwidth Request)
- **CORRECTION** : la note précédente indiquait Isoch TRB type = 8 ; c'est faux. Le type d'un Isoch TRB est **5** (la valeur 8 est No-Op). Les types de Transfer TRB rappelés ici : Normal 1, Setup 2, Data 3, Status 4, Isoch 5, Link 6, Event Data 7, No-Op 8 (valeurs de Table 6-91 à confirmer en lisant cette table).
- Isoch TRB suite : DW3 bit4 CH, bit5 IOC, bit6 IDT, bits 8:7 TBC/TRBSts, bit9 BEI, 19:16 TLBPC, 30:20 Frame ID, bit31 SIA.
- No-Op TRB : DW2 bits 21:0 RsvdZ, 31:22 Interrupter Target ; DW3 bit0 C, bit1 ENT, bit4 CH, bit5 IOC, 15:10 Type = No-Op (8), 31:16 RsvdZ.
- **Transfer Event TRB** (type 32) : DW0-1 = TRB Pointer (aligné 16 o, sauf ED=1 → donnée d'événement 64 bits) ; DW2 : 23:0 TRB Transfer Length (résidu), 31:24 Completion Code ; DW3 : bit0 C, bit2 ED, 15:10 Type, **20:16 Endpoint ID (DCI)**, **31:24 Slot ID**.
- **Command Completion Event TRB** (type 33) : DW0-1 = Command TRB Pointer (63:4) ; DW2 : 23:0 Command Completion Parameter, 31:24 Completion Code ; DW3 : bit0 C, 15:10 Type, 23:16 VF ID (0 si VF désactivé), 31:24 Slot ID (0 pour No-Op/Set Latency/Get Port Bandwidth/Force Event ; nouveau slot pour Enable Slot).
- **Port Status Change Event TRB** (type 34) : DW0 bits 31:24 Port ID (numéro de port root hub), autres RsvdZ ; DW2 bits 31:24 Completion Code = Success ; DW3 bit0 C, 15:10 Type.
- **Bandwidth Request Event** (type 35) : DW2 31:24 Completion Code = Success ; DW3 31:24 Slot ID.
- Vérification agent : usb_r0.rs/usb_enum.rs doivent décoder Transfer Event par Slot ID (bits 31:24 DW3) et EP ID (20:16), Command Completion par Completion Code (DW2 31:24) et TRB Pointer (63:4), Port Status Change par Port ID (DW0 31:24) ; le cycle bit C (DW3 bit0) doit être comparé au CCS consumer.

## Lignes 17951-18550 (§6.4.2.5 Doorbell Event ; §6.4.2.6 Host Controller Event ; §6.4.2.7 Device Notification ; §6.4.2.8 MFINDEX Wrap ; §6.4.3 Command TRBs : No-Op, Enable Slot, Disable Slot, Address Device)
- Doorbell Event (DW0 4:0 DB Reason, DW2 31:24 Completion Code = Success, DW3 bit0 C, 15:10 Type, 23:16 VF ID, 31:24 Slot ID) : VF uniquement.
- Host Controller Event (DW2 31:24 Completion Code, DW3 C + Type ; sert à signaler erreurs xHC, ex. Event Ring Full). Destiné à l'Event Ring 0 ou secondaire.
- Device Notification Event (DW0 7:4 Notification Type, 63:8 Device Notification Data ; DW2 31:24 Completion Code = Success ; DW3 31:24 Slot ID). Uniquement si DNCTRL activé.
- MFINDEX Wrap Event : DW2 31:24 Completion Code = Success ; DW3 C + Type. Reçu sur Event Ring 0.
- **Command TRBs** (sur Command Ring ; DW3 bit0 = C, 15:10 = Type) :
  - No-Op Command : aucun champ utile (la note indique Slot Type à 20:16 dans la figure ; à vérifier).
  - **Enable Slot** : DW3 20:16 Slot Type (Table 7-9), 15:10 Type.
  - **Disable Slot** : DW3 31:24 Slot ID, 15:10 Type.
  - **Address Device** : DW0-1 = Input Context Pointer 63:4 (aligné 16 o, RsvdZ 3:0) ; DW3 bit9 **BSR** (0 = émet SET_ADDRESS ; 1 = pas de SET_ADDRESS), 31:24 Slot ID, 15:10 Type.
- Note générale : les données tampon des command TRBs ne traversent pas PAGESIZE.
- Vérification agent : usb_enum.rs doit mettre BSR=0 pour un nouveau périphérique (sinon pas de SET_ADDRESS), et DW3 bits 31:24 = Slot ID, Input Context Pointer aligné 16 o.

## Lignes 18551-19150 (§6.4.3.5 Configure Endpoint ; §6.4.3.6 Evaluate Context ; §6.4.3.7 Reset Endpoint ; §6.4.3.8 Stop Endpoint ; §6.4.3.9 Set TR Dequeue Pointer ; §6.4.3.10 Reset Device ; §6.4.3.11 Force Event ; §6.4.3.12 Negotiate BW ; §6.4.3.13 Set LTV début)
- **Configure Endpoint** (type 12) : DW0-1 Input Context Pointer 63:4 (aligné 16) ; DW3 bit9 **DC** (Deconfigure : si 1, Input Context Pointer ignoré), 31:24 Slot ID.
- **Evaluate Context** (type 13) : même format que Address Device sans BSR (DW3 bit9 non utilisé), 31:24 Slot ID.
- **Reset Endpoint** (type 14) : DW3 bit9 **TSP** (1 = état de transfert préservé ; 0 = Data Toggle/Sequence Number remis à 0), 20:16 Endpoint ID (DCI), 31:24 Slot ID.
- **Stop Endpoint** (type 15) : DW3 bit23 **SP** (Suspend : arrêt ≥10 ms avant suspension), 20:16 Endpoint ID (1..Context Entries), 31:24 Slot ID. Rappel : attendre ≥10 ms après PLS=3 (U3/suspend) avant un port resume.
- **Set TR Dequeue Pointer** (type 16) : DW0 bit0 DCS, 3:1 SCT, 63:4 New TR Dequeue Pointer ; DW2 31:16 Stream ID ; DW3 20:16 Endpoint ID, 31:24 Slot ID. Ne doit être émis que si l'endpoint/stream est en Error ou Stopped.
- **Reset Device** (type 17) : DW3 31:24 Slot ID seulement. Slot → Default, adresse 0, EP≠0 désactivés.
- **Force Event** (type 18, VF, optionnel) : hors périmètre agent.
- **Negotiate Bandwidth** (type 19, optionnel) : même format que Disable Slot.
- **Set LTV** (type 20, optionnel) : BELT bits 27:16 ; hors périmètre agent.
- Vérification agent : aucune de ces commandes ne doit être émise en phase boot sauf Enable Slot, Address Device, Evaluate Context, Configure Endpoint, Reset Endpoint si besoin (pas de Stop Endpoint/Set TR Dequeue nécessaire tant qu'il n'y a pas d'erreur). Vérifier que usb_enum.rs envoie Configure avec DW3 DC=0.

## Lignes 19151-19750 (§6.4.3.13 suite LTV ; §6.4.3.14 Get Port Bandwidth ; §6.4.3.15 Force Header ; §6.4.3.16-17 Get/Set Extended Property ; §6.4.4.1 Link TRB ; §6.4.4.2 Event Data TRB début)
- Set LTV : BELT 27:16 ; type à 15:10 (optionnel).
- Get Port Bandwidth (optionnel) : DW0-1 Port Bandwidth Context Pointer 63:4 (aligné 16) ; DW3 19:16 Dev Speed (Table 5-27), 31:24 Hub Slot ID (0 = root hub).
- Force Header (optionnel) : DW0 4:0 Packet Type, 95:5 Header Info ; DW2 31:24 Root Hub Port Number. Hors périmètre.
- Get/Set Extended Property (optionnel) : DW0-1 pointeur contexte ; DW2 15:0 Extended Capability Identifier, 23:16 Capability Parameter ; DW3 18:16 Command SubType, 23:19 EP ID, 31:24 Slot ID. Hors périmètre.
- **Link TRB** (type 6) : DW0-1 = Ring Segment Pointer 63:4 (aligné 16) ; DW2 31:22 Interrupter Target (ignoré sur Command Ring) ; DW3 bit0 **C** (cycle), bit1 **TC (Toggle Cycle)** : si 1, le xHC inverse son CCS après ce lien ; bit4 CH ; bit5 IOC ; 15:10 Type = 6.
  - Règle : un Command Ring doit terminer par un Link TRB pointant sur le début avec TC=1 (sinon la ronde n'est pas fermée). **Confirmation d'un point d'erreur possible dans usb_r0.rs (command ring sans Link TRB ni toggle).**
- **Event Data TRB** (type 7) : DW0-1 Event Data (64 bits, copié dans TRB Pointer de l'événement) ; DW2 31:22 Interrupter Target ; DW3 : bit0 C, bit1 ENT, bit4? BEI (figure ambiguë), bit5 IOC, bit6? CH, 15:10 Type.
- Vérification agent : usb_r0.rs / usb_enum.rs doivent placer un Link TRB avec TC=1 en fin du command ring et des transfer rings, et le cycle bit doit être inversé au passage.

## Lignes 19751-20350 (§6.4.4.2 suite Event Data TRB ; §6.4.5 Completion Codes Table 6-90 ; §6.4.6 Table 6-91 types)
- Event Data TRB DW3 (correction de la note précédente) : bit0 C (utilisé ; le texte « ignored in a Link TRB » est une coquille de copie), bit1 ENT, bit4 CH, bit5 IOC, bit9 BEI, 15:10 Type = 7.
- **Table 6-91 (types TRB, à retenir)** : 1 Normal, 2 Setup Stage, 3 Data Stage, 4 Status Stage, 5 Isoch, 6 Link, 7 Event Data, 8 No-Op (Transfer Ring), 9 Enable Slot, 10 Disable Slot, 11 Address Device, 12 Configure Endpoint, 13 Evaluate Context, 14 Reset Endpoint, 15 Stop Endpoint, 16 Set TR Dequeue Pointer, 17 Reset Device, 18 Force Event (opt.), 19 Negotiate Bandwidth (opt.), 20 Set LTV (opt.), 21 Get Port Bandwidth (opt.), 22 Force Header, 23 No-Op Command, 24 Get Extended Property (opt.), 25 Set Extended Property (opt.), 32 Transfer Event, 33 Command Completion Event, 34 Port Status Change Event, 35 Bandwidth Request Event, 36 Doorbell Event, 37 Host Controller Event, 38 Device Notification Event, 39 MFINDEX Wrap Event. (Valeurs 26-31 réservées.)
  - Un même ID peut signifier un type différent selon le type d'anneau (Command / Transfer / Event). Ex. ID 8 = No-Op sur Transfer Ring, ID 23 = No-Op sur Command Ring.
- **Table 6-90 (Completion Codes)** : 0 Invalid, 1 Success, 2 Data Buffer Error, 3 Babble Detected, 4 USB Transaction Error, 5 TRB Error, 6 Stall Error, 7 Resource Error, 8 Bandwidth Error, 9 No Slots Available, 10 Invalid Stream Type, 11 Slot Not Enabled, 12 Endpoint Not Enabled, 13 Short Packet, 14 Ring Underrun, 15 Ring Overrun, 16 VF Event Ring Full, 17 Parameter Error, 18 Bandwidth Overrun, 19 Context State Error, 20 No Ping Response, 21 Event Ring Full, 22 Incompatible Device, 23 Missed Service, 24 Command Ring Stopped, 25 Command Aborted, 26 Stopped, 27 Stopped - Length Invalid, 28 Stopped - Short Packet, 29 Max Exit Latency Too Large, 31 Isoch Buffer Overrun, 32 Event Lost, 33 Undefined (fatal), 34 Invalid Stream ID, 35 Secondary Bandwidth Error, 36 Split Transaction Error, 37-191 réservés, 192-223 vendor error, 224-255 vendor info (Success).
- Règle : un Undefined Error est fatal pour le logiciel. Un Incompatible Device Error impose Disable Slot.
- Vérification agent : pour tout code ≠ 1 (Success) ou 13 (Short Packet), le code doit arrêter la boucle d'énumération et logger le code (pas de succès supposé). À vérifier dans usb_enum.rs.

## Lignes 20351-20950 (§6.4.6 suite Table 6-91 : Event/Command/Transfer rings ; Table 6-92/93 ; §6.5 ERST ; §6.6 Scratchpad ; §7 xHCI Extended Capabilities ; §7.1 USB Legacy Support)
- **Command Ring** : seuls les types marqués Allowed sur la colonne Command sont exécutés ; tout autre type → Command Completion Event Completion Code = **TRB Error**, Command TRB Pointer = adresse du TRB fautif, Slot ID = 0.
- **Transfer Ring** : seuls les types Allowed (Transfer) sont exécutés ; tout autre type → Transfer Event Completion Code = TRB Error, TRB Pointer = adresse du TRB fautif, Slot/EP ID du ring ; ring passe en Error.
- Table 6-92 (types par type d'endpoint) : Isoch → Isoch, Normal, Event Data, No-Op, Link ; Interrupt/Bulk → Normal, Event Data, No-Op, Link ; Control → Setup, Data, Status, Normal, Event Data, No-Op, Link (Normal pour Control est aussi optionnel) ; Vendor Defined optionnel.
- Table 6-93 (types par TD) : TD Isoch = Isoch, Normal, Event Data, No-Op ; TD Interrupt/Bulk = Normal, Event Data, No-Op ; TD Control = Setup, Data, Status, Normal, Event Data, No-Op ; Vendor defined.
- **ERST Entry** (16 octets ; §6.5) : DW0-1 Ring Segment Base Address 63:6 (aligné 64 octets, bits 5:0 RsvdZ) ; DW2 15:0 **Ring Segment Size** = nombre de TRB : 16 à 4096 (16 minimum). Allouer le segment arrondi à 64 octets.
- **Scratchpad Buffer Array** (§6.6) : entrée 64 bits ; bits 11:0 RsvdZ ; bits 63:PSZ = adresse ; **PSZ = 12 + décalage de PAGESIZE** (4 KB → PSZ=12, aucun bit réservé). Toutes les entrées identiques ; pointeur placé dans DCBAA[0].
- §7 Extended Capabilities : xECP (HCCPARAMS1 bits 31:16 en DWORD). Chaque en-tête : bits 7:0 Capability ID, 15:8 Next Pointer (offset en DWORDs à partir de cet en-tête ; 0 = fin), 31:16 spécifique. IDs : 1 USB Legacy Support, 2 Supported Protocol, 3 Extended Power Management, 4 I/O Virtualization, 5 Message Interrupt, 6 Local Memory, 10 USB Debug Capability, 17 Extended Message Interrupt, 19 USB3 Tunnelling (à confirmer dans Table 7-2), 192-255 vendor.
- §7.1 **USBLEGSUP** (xECP+00h, Aux Power) : 7:0 Capability ID (1), 15:8 Next ptr, **bit16 HC BIOS Owned Semaphore** (RW, BIOS met à 1 pour posséder ; BIOS met à 0 en réponse à une demande OS), **bit24 HC OS Owned Semaphore** (RW, l'OS met à 1 pour demander ; possession obtenue quand bit24=1 ET bit16=0).
- §7.1.2 **USBLEGCTLSTS** (xECP+04h, défaut 0) : bit0 USB SMI Enable ; bit4 SMI on HSE Enable ; bit13 SMI on OS Ownership Enable ; bit14 SMI on PCI Command Enable ; bit15 SMI on BAR Enable ; bit16 SMI on Event Interrupt (RO, shadow EINT) ; bit20 SMI on HSE (RO) ; bit29 SMI on OS Ownership Change (RW1C) ; bit30 SMI on PCI Command (RW1C) ; bit31 SMI on BAR (RW1C). Bits 21:16 = shadow USBSTS[5:0].
- **Vérification agent** : la séquence de prise de possession doit être : lire USBLEGSUP ; si BIOS Owned (bit16)=1, écrire bit24=1 et attendre que bit16 passe à 0 (avec délai ≤ quelques secondes) ; NE PAS supposer que bit24 est déjà positionné. Le point signalé dans XHCI_HDA_VERIFICATION.md (usb_r0.rs l.89-91 : OS Owned mis à 1 seulement si BIOS Owned déjà à 1) est à corriger selon cette règle.

## Lignes 20951-21550 (§7.1.2 fin ; §7.2 Supported Protocol Capability ; §7.2.1 PSI ; §7.2.2 Protocoles ; §7.2.2.1.1 mapping par défaut ; §7.2.2.1.3.1 USB3 Protocol Defined)
- **Supported Protocol Capability** (ID = 2) : DW0 : 7:0 ID=2, 15:8 Next (DWORDs), 23:16 Minor (BCD), 31:24 Major (BCD) ; DW1 : Name String = "USB " = **0x20425355** ; DW2 : 7:0 **Compatible Port Offset** (≥1 ; la première capability vaut 1), 15:8 **Compatible Port Count**, 27:16 Protocol Defined, 31:28 **PSIC** (nombre de PSI Dwords) ; DW3 : 4:0 **Protocol Slot Type** (ne pas supposer un mapping fixe ; 0 = USB Device).
- **PSI Dword** (à partir de l'offset 10h, PSIC dwords) : 3:0 PSIV (0 réservé) ; 5:4 PSIE (0 bps, 1 Kb/s, 2 Mb/s, 3 Gb/s) ; 7:6 PLT (0 symétrique, 2 Asym Rx, 3 Asym Tx) ; bit8 PFD (1 full-duplex) ; 15:14 LP (0 SuperSpeed, 1 SuperSpeedPlus) ; 31:16 PSIM (mantisse).
  - Débit = PSIM × 10^(3×PSIE) bps. Ex. 480 Mb/s = PSIM 480, PSIE 2.
- **PORTSC Speed** (bits 13:10) pointe vers un PSIV d'une de ces capabilities. Le mapping par défaut (si PSIC=0) : **1 = Full-speed 12 Mb/s**, **2 = Low-speed 1.5 Mb/s**, **3 = High-speed 480 Mb/s**, **4 = SuperSpeed Gen1 x1 5 Gb/s**, 5 = SuperSpeedPlus Gen2 x1 10 Gb/s, 6 = SSP Gen1 x2 5 Gb/s, 7 = SSP Gen2 x2 10 Gb/s (mapping 3.2) — la table des valeurs par défaut est dans Table 7-13.
- Mapping implicite (PSIC=0) autorisé seulement pour USB 2.0 (FS, LS, HS), USB 3.0 (SS Gen1 x1), USB 3.1 (SS + SSP Gen2 x1), USB 3.2 (4 modes par défaut) ; tout autre (SSIC, etc.) doit avoir PSIC≠0.
- Règles : un Compatible Port Offset de 1 doit exister ; les plages de ports USB2 et USB3 ne doivent pas se chevaucher ; les ports doivent être déclarés par une capability (sinon comportement indéfini).
- **USB3 Protocol Defined (DW2 bits 27:16)** : 23:16 RsvdP ; **24 LSECC** (Link Soft Error Count Capability, RO) ; **27:25 MHD** (Max Hub Depth, 0 = contraintes standard). Tous les ports USB3 supportent LPM.
- **Vérification agent** : usb_r0.rs doit parcourir les Supported Protocol Capabilities (et non supposer USB2 = ports 1..N). Le champ Speed de PORTSC doit être interprété via le PSI correspondant, pas via une table fixe, si PSIC≠0.

## Lignes 21551-22050 (§7.2.2.1.3.2 USB2 Protocol Defined ; §7.2.2.1.4 ; §7.3-7.5 Power Mgmt / MSI-X / MSI ; §7.6 Debug Capability début, 7.6.1-7.6.6.3)
- **USB2 Protocol Defined (DW2)** : bit17 HSO (High-speed Only ; 0 = LS/FS/HS) ; bit18 IHI (Integrated Hub Implemented ; si 1 le mapping root hub→port externe n'est pas standard, ACPI requis) ; bit19 **HLC** (Hardware LPM Capability, obligatoire xHCI 1.1/1.2) ; bit20 **BLC** (BESL LPM Capability, obligatoire 1.1/1.2 ; si 1 BESL, sinon HIRD) ; 27:25 MHD (Hub Depth).
- §7.2.2.1.4 : Protocol Slot Type = 0 pour USB2 et USB3.
- §7.3 Extended Power Management (requis hors PCI ; Next ptr en mots 32 bits) ; §7.4 Extended MSI-X ; §7.5 MSI — non utilisé par l'agent (il lit les capacités seulement).
- **§7.6 Debug Capability (DbC)** : système « cible de débogage » séparé de l'interface xHCI ; **R/S n'a aucun effet dessus**. Activé uniquement par DCCTRL.DCE. Le DbC occupe un port root hub (le premier détectant un hôte SS). Note : « ne pas accéder aux registres DbC pendant CNR=1 ».
  - **Règle pour l'agent** : ne jamais écrire DCCTRL.DCE=1 ; si on lit le DbC, pas d'écriture (hors périmètre). Sur un port DbC, PORTSC doit paraître « no device » tant qu'il est assigné au DbC.
- Séquence d'initialisation DbC (§7.6.4.1) : allouer ERST + segments, DCERSTSZ, DCERSTBA, DCERDP, DCCP (Debug Context), puis DCE=1. Hors périmètre agent.
- §7.6.4.2 Event Generation DbC : bits CSC/PRC/PLC/CEC de DCPORTSC, Port ID toujours 0 dans les événements DbC.
- §7.6.3 : Endpoint Context DbC : Max Packet Size = 1024 ; EP Type 2 (Bulk OUT) et 6 (Bulk IN) ; Average TRB Length logiciel.
- Vérification agent : usb_r0.rs ne doit pas écrire dans les registres DbC ni activer DCE. Si le code lit DCCTRL, le faire seulement quand CNR=0.

## Lignes 22051-22650 (§7.6.4-7.6.8 DbC : port state machine, enumeration/run mode, structure des registres)
- Hors périmètre agent (DbC non utilisé). Points notés pour cohérence : DbC-Off après reset ; DCE=1 → DbC-Disconnected ; SET_CONFIGURATION(0) ou PED=0 → sortie de DbC-Configured ; DbC Run (DCR) passe à 1 après SET_CONFIGURATION.
- Structure DbC (base = Capability DbC) : 00h DCID (cap ID, DCERST Max bits 20:16), 04h DCDB (DB Target 15:8 : 0 = EP1 OUT, 1 = EP1 IN), 08h DCERSTSZ, 10h DCERSTBA (63:4), 18h DCERDP (2:0 DESI, 3 RsvdP, 63:4), 20h DCCTRL (DCE, DCR, HOT, HIT, LSE, DRC), 24h DCST, 28h DCPORTSC, 30h DCCP (63:4), 38h DCDDI1, 3Ch DCDDI2 (Vendor ID / Product ID / Device Revision / DbC Protocol).
- Règle : DbC ne doit pas être activé (DCE) par l'agent. Pas de TRB IDT=1 sur DbC.
- Règle générale PORTSC : sur un port assigné au DbC, PORTSC doit paraître vide (pas de périphérique) ; les autres ports restent normaux.

## Lignes 22651-23250 (§7.6.8.4 DCCTRL ; §7.6.8.5 DCST ; §7.6.8.6 DCPORTSC ; §7.6.8.7 DCCP ; §7.6.8.8-9 DCDDI1/2 ; §7.6.9 Data Structures début ; §7.6.9.1 DbCIC début)
- DCCTRL (20h) : bit0 DCR (RO), bit1 LSE, bit2 HOT (RW1S), bit3 HIT (RW1S), bit4 DRC (RW1C), 23:16 Debug Max Burst Size (RO), 30:24 Device Address (RO), **bit31 DCE** (RW, active le DbC et libère le port root hub).
- DCST (24h) : bit0 ER (Event Ring Not Empty), bit1 SBR (RO), 31:24 Debug Port Number.
- DCPORTSC (28h) : bit0 CCS, bit1 PED (RW), bit4 PR (RO), 8:5 PLS, 13:10 Port Speed (Debug supporte seulement SS/PSI ; LS/FS/HS non), bit17 CSC, bit21 PRC, bit22 PLC, bit23 CEC (RW1C). Champs « réaffectés » côté device (pas comme PORTSC normal).
- DCCP (30h) : 63:4 pointeur sur Debug Capability Context (64 octets par structure : DbC Info puis OUT EP à 40h, IN EP à 80h).
- DCDDI1 (38h) : 7:0 DbC Protocol (0 = Debug Target vendor, 1 = GNU Remote Debug, 2-255 réservé), 31:16 Vendor ID. DCDDI2 (3Ch) : 15:0 Product ID, 31:16 Device Revision.
- Note : le DbC ne déclenche pas de Port Status Change pour les ports normaux ; si Event Ring DbC plein, les bits changement restent à 1 jusqu'à effacement.
- Hors périmètre agent ; ne jamais écrire DCE / DCCP / DCDDI sans besoin validé.

## Lignes 23251-23850 (§7.6.9.1 DbCIC suite ; §7.6.9.2 DbC EP Context ; §7.6.10 Descripteurs DbC : Device, Configuration, Interface, Endpoint OUT, SS Companion début)
- DbCIC (64 o) : 00h String 0 Descriptor Address (63:1), 08h Manufacturer, 10h Product, 18h Serial Number (adresses 64 bits, bit0 RsvdZ), 20h longueurs : 7:0 String0, 15:8 Manufacturer, 23:16 Product, 31:24 Serial.
- DbC EP Context : MaxPStreams, LSA, HID = 0 ; Interval, Mult, Max ESIT = 0 ; taille fixe 64 o.
- Descripteurs DbC : Device (12h, bcdUSB 0320h, bMaxPacketSize0 = 09h, idVendor/idProduct/bcdDevice = DCDDI, iManufacturer 01h, iProduct 02h, iSerial 03h, 1 config). Configuration (wTotalLength 002Ch, bmAttributes C0h, 1 interface, bConfigurationValue 01h). Interface (class **DCh** ou 02h, subclass 02h, protocol = DCDDI1.DbC Protocol, 2 endpoints). Endpoint OUT : bEndpointAddress 01h, Bulk, wMaxPacketSize 0400h (1 KB). Companion SS : bMaxBurst à suivre.
- Hors périmètre agent.

## Lignes 23851-24450 (§7.6.10.5 SS Companion OUT suite ; §7.6.10.6-7 EP IN + Companion ; §7.6.10.8 BOS ; §7.6.10.9 Strings ; §7.7 xHCI-IOV début ; 7.7.1-7.7.3)
- DbC : EP IN bEndpointAddress 81h ; companion bMaxBurst via DCCTRL ; bmAttributes 0 (Mult=0, MaxStreams=0) ; wBytesPerInterval 0.
- **BOS** : 0Fh, wTotalLength 0Fh, 1 device capability = SuperSpeed USB (type 03h) ; bmAttributes 00h (pas LTM) ; wSpeedsSupported 0008h (5 Gb/s) ; bFunctionalitySupported 03h ; U1/U2 exit latency = défini par vendor.
- Strings DbC : une seule LANGID dans String 0.
- **xHCI-IOV** (§7.7, optionnel normatif) : virtualisation ; hors périmètre agent. Registres : capability header (ID en 7:0, next 15:8), VF Interrupter Range (xECP + 4×VF_ID, bits IRROFF 9:0, IRRCNT 19:10, VFR 20, VFH 21), VF Device Slot Assignment (xECP + 04h + 4×TotalVFs + 4×SlotID). Limite TotalVFs ≤ 63. Interrupteur 0 jamais assigné à un VF. Page size : un seul PAGESIZE supporté.
- Vérification agent : aucune écriture dans ces registres ; ils ne sont pas pertinents pour un boot mono-OS.

## Lignes 24451-25050 (§7.7.3 suite VF Device Slot Assignment ; §7.8 Local Memory ; §7.9 Audio Sideband ; §7.10 Time Stamp Correlation ; §7.11 USB3 Tunneling Support ; §7.12 début Camera Sideband)
- VF Device Slot Assignment : bits 5:0 DSAVFID, bit6 DSASE (Slot Emulated) ; écrire vide l'adresse USB du slot.
- **Local Memory** (§7.8, optionnel) : LME bit16 ; taille en KB (×256 DWORDs). Hors périmètre.
- **Audio Sideband / Camera Sideband** (§7.9, §7.12) : Get/Set Extended Property avec ECI bit0 ; sous-types 001 (Get Supported Resources : TRB type 24, Resource Count 7:0 1-based), 010 (Get Endpoint Properties : bit20 TT, 19:16 Port Speed PSIV, 15:8 Resource Number, 7:0 Root Hub Port). Set Resource Assignment (001). Hors périmètre agent.
- **Time Stamp Correlation** (§7.10) : ECI bit12 ; sous-type 001 Get Time Stamp ; Extended Property Context : System Time Hi/Lo, USB Bus Interval Counter 29:16, Delta 12:0. Hors périmètre.
- **USB3 Tunneling Support** (§7.11) : bit16 Tunneled Mode Bit Supported. Si 1, le bit Tunneled Mode dans PORTSC est valide.
- Vérification agent : PORTSC bit « Tunneled Mode » (si présent) ne doit pas être écrit ; à lire seulement si USB3 Tunneling Capability présente.

## Lignes 25051-25650 (§7.12 Camera Sideband suite : Get Supported Resources, Endpoint Properties, Resource Properties ; §7.12.2 Set ; §8 Virtualization début ; §8.1 ; §8.1.1 MMIO)
- Camera Sideband : ECI bit1 ; sous-types 001 Get Supported Resources, 010 Get Endpoint Properties, 011 Get Resource Properties (RS bit4 status, RD 3:2 direction, RT 1:0 type). Hors périmètre.
- Virtualisation (§8) : hors périmètre. Adresses MMIO utiles : PF0 : Capability à PBAR0, Operational à PBAR0+CAPLENGTH, Runtime à PBAR0+RTSOFF, Doorbell à PBAR0+DBOFF. Pour l'agent (mono-PF) : mêmes formules sans VF.
- Règle : seul PF0 présente SR-IOV et xHCI-IOV ; un VF est vu comme xHC non virtualisé.

## Lignes 25651-26250 (§8.1.1 MMIO VF ; §8.1.1.2-3 Device Slots / Interrupters ; §8.1.2 Enumeration virtualisée ; §8.2 SR-IOV ; §8.3 Doorbell ; §8.4-8.5 ; Annexe A PCI PM ; Annexe B HS high-bandwidth isoch début)
- Virtualisation : hors périmètre. Formules utiles : doorbell n = PBAR0 + DBOFF + 4n (sans VF) ; interrupter n = PBAR0 + RTSOFF + 32n. Interrupter 0 réservé PF0 ; un seul Event Ring primaire.
- **Annexe A (PCI PM)** : D0 requis ; D1/D2 optionnels ; D3hot/D3cold requis. Le driver doit suspendre les ports actifs avant de quitter D0. USB Legacy Support et PORTSC sont dans Aux Power (conservés D3hot→D0), **reset seulement cold reset / HCRST**. Pour l'agent : pas de transition de puissance PCI pendant la session ; si besoin, ports en suspend d'abord.
- Annexe B (HS high-bandwidth isoch, tables B-1 OUT) : Burst = Max Burst Size+1 ; PID DATA0/1/2 et MDATA selon micro-trame. Hors périmètre agent (pas d'isochrone).
- Vérification agent : aucune écriture PCI PM n'est faite (ok). Pas de isochrone (ok).

## Lignes 26251-26850 (Annexe B suite : Table B-2 IN HS high-bandwidth ; Annexe C Stream usage ; Annexe D Port to Connector Mapping début, exemple ACPI _UPC/_PLD)
- Annexe B : hors périmètre (isoch HS haute bande passante).
- Annexe C (usage des Streams, disque de masse UAS) : hors périmètre agent.
- Annexe D (mapping ports → connecteurs via ACPI _UPC / _PLD) : c'est la seule source de la correspondance « port root hub → connecteur physique » ; elle n'est pas dans les registres. Pour l'agent : si on veut afficher les connecteurs, il faut lire les tables ACPI (_UPC, _PLD) ; ce n'est pas un registre xHCI. Exemple : _UPC type 0x03 = USB3 Type A ; _PLD visible=bit 0x00800000 (bit 23 = User visible), Panel/Group Token etc.
- Règle : ne pas déduire la visibilité d'un port sans _UPC/_PLD.

## Lignes 26851-27450 (Annexe D fin : exemple ACPI _UPC/_PLD ; Annexe E conventions de machines d'état ; Annexe F SS bus access début, F.1 Bulk OUT, F.2 Interrupt)
- Annexe D.2 : exemple ACPI complet (USB0/RHUB/HCP1..4, IP1..4, EP1..2). Règle _UPC : 0xFF = connectable, type 0x03 = USB3 Type A, 0x00 = non visible. _PLD : bit visible (0x00800000 = user visible), couleur 0072C6h = bleu USB3.
- Annexe E : légende des machines d'état ; uniquement pour le comportement attendu.
- Annexe F (bande passante SS, hors périmètre agent) : overhead 48 o par DP Bulk OUT (32 o DP + LGOOD_n 8 o + L_CRD 8 o) ; max ~905 DP 1-octet par microframe ; interruption ≤90% de la microframe ; ≤3 Max Packet Size DP par microframe pour un endpoint interrupt.

## Lignes 27451-27951 (F.3 Isoch SS ; Annexe G 0.96 exceptions ; Annexe H Release 1.1 ; Annexe I Release 1.2 ; I.7 eUSB2V2) — FIN DE LECTURE xHCI 1.2c (27951 lignes)
- F.3 : Isoch OUT overhead 32 o ; ≤48 Max Packet Size DPs par microframe ; TD > Max ESIT Payload → Bandwidth Overrun Error (hors périmètre).
- G : différences 0.96 : Link TRB IOC skip ; FSE (HCCPARAMS1 bit8 Forced Stopped Event) ; SBD (bit9) ; L1C (PORTSC/PORTPMSC valides seulement si bit16 du Supported Protocol USB2 = 1) → **ne pas écrire PORTPMSC/PLS L1 si L1C=0**.
- H.1 : Release 1.1 requis : HLC, BLC, CFC (Contiguous Frame ID), SEC, U3C, SPC, FSC, CTC, CIC ; CIE=1 obligatoire pour xHCI 1.1 ; nouveau 1.1 : Ring Underrun/Overrun → TRB Pointer = adresse du TRB invalide.
- I : Release 1.2 : Get/Set Extended Properties (optionnels), USB 3.2 (Table 7-11/13), Audio Sideband, Time Stamp Correlation, PORTEXSC USB3/USB2, LPM. Get Port Bandwidth passe optionnel (TRB Error si absent). I.4 : PORTLI RW, Slot Context MTT/TTT (Speed 3/1/2), EP Mult pour SS Isoch ; I.7 eUSB2V2 et HCCPARAMS2[12:11].
- Règle agent : tester HCCPARAMS1 bit8/9, Supported Protocol Revision (3.2 → PSI nécessaires) avant d'interpréter Speed.

## FIN DE LECTURE xHCI 1.2c (27951 lignes) — 2026-11-12
Points à vérifier dans le code (voir synthèse) : CNR, CSZ, Link TRB sur command/transfer rings (TC=1), ERSTBA/ERSTSZ/ERDP ordre + EHB, DCBAA[0] scratchpad avant R/S=1, PSZ=12 pour scratchpad, HCCPARAMS1 xECP / USBLEGSUP (OS Owned), PORTSC Speed via PSI, Input Control Context (DW7 CfgValue), ICI formula, Setup TRB (IDT=1, length 8, TRT, DIR), Command completion par Slot ID et CC, CErr=3, Average TRB Length=8 EP0, Max Packet Size EP0 selon vitesse, Doorbell 0 = command, DB EP = DCI.

# VERIFICATION DU CODE CONTRE xHCI 1.2c (2026-11-12)

Fichiers relus : src/usb_r0.rs (l.60-200, 366-500), src/usb_enum.rs (l.30-160), src/usb_hid.rs (l.150-250). **Non relus** : usb_hub.rs (Configure Endpoint hub), interrupt_interval(), control(), set_device_base(), PAGESIZE.

## Conforme (vérifié ligne à ligne)
- HCSPARAMS2 scratchpad : (Hi<<5)|Lo, Hi bits 25:21, Lo bits 31:27 (usb_r0.rs l.74).
- HCSPARAMS1 MaxSlots 7:0, MaxPorts 31:24 (l.70-71). DBOFF (cap+14h), RTSOFF (cap+18h) (l.78).
- xECP = (HCCPARAMS1 >>16)*4 (l.84). USBLEGSUP bit16 BIOS / bit24 OS.
- Séquence : RS=0 puis attente HCH (USBSTS bit0) ; HCRST (USBCMD bit1) puis attente HCRST=0 et CNR=0 (USBSTS bit11) (l.110-126) : conforme §4.2.
- CONFIG (op+38h) = MaxSlots (l.128). DCBAAP (op+30h) avant R/S=1 (l.144). DCBAA[0] = scratchpad array si Max Scratchpad>0 (l.137-143). Entrées scratchpad = pages 11.. (l.139-141).
- CRCR (op+18h) = ring | RCS=1 (l.147).
- ERST : entrée 0 = base évt, DW2 size=256 (16..4096 OK) (l.151-152). ERSTSZ=1, ERDP, ERSTBA, IMAN.IE (l.156-159) : ordre conforme (ERSTBA écrit après ERDP, HCH=1 donc autorisé).
- RS=1 + INTE (USBCMD bit2) (l.162), attente HCH=0.
- PORTSC = op+400h+10h×(n-1) (l.179 et root_route). Masque RW1C conservé dans enable_slot : PED bit1 + bits 17-23 (l.378) ; PR bit4 ; PRC bit21 efface (l.387).
- Slot Context : route 19:0, speed 23:20, Context Entries 31:27=1 (Address Device) (l.133), root port 23:16 (l.134), TT parent slot 7:0 / port 15:8 (l.135).
- Input Control : A0+A1 = 0x3 (l.130). Contexte EP0 à offset 2×ctx (DCI 1) (l.137).
- EP0 : type 4 bits 5:3, CErr=3 bits 2:1, MPS 31:16 (l.138) ; TR dequeue | DCS=1 (l.139) ; Average TRB 8 (l.140). Conforme §6.2.3 / §4.6.5.
- Address Device TRB : type 11, slot 31:24, BSR=0 (l.148). Doorbell 0 = 0 (l.151).
- Enable Slot TRB : type 9 (usb_r0 l.406).
- Boucle d'événements : cycle bit d3 bit0 vs CCS, type d3 15:10, code d2 31:24, slot d3 31:24, ERDP avec EHB (bit3) à chaque consommation (l.476-497). Conforme §4.9.4.
- usb_hid.rs : Configure Endpoint — A0 | A(DCI), Context Entries = DCI (bits 31:27), EP ctx à (DCI+1)×ctx (ICI correct), EP type 7 (Interrupt IN), CErr 3, DCS=1, Average TRB 8, Max ESIT = MPS (cas simple). DC bit9 = 0.
- usb_hid.rs : anneau d'interruption : Normal TRB (type 1, IOC bit5) ; Link TRB index 15 (type 6, TC bit1) avec bascule du cycle : conforme §6.4.4.1.

## Écarts (à corriger ; par gravité)
1. **[Moyen] Prise de possession USB Legacy** (usb_r0.rs l.89-98) : le code ne positionne USBLEGSUP bit24 (OS Owned) que si le bit16 (BIOS Owned) est déjà à 1. Spec §7.1.1 : l'OS met bit24=1 pour demander l'ownership, et l'obtient quand bit24=1 ET bit16=0. Correction : toujours écrire bit24=1, puis attendre bit16=0 (avec délai max).
2. **[Moyen, latent] Anneau de commandes sans Link TRB** (usb_r0.rs l.134, usb_enum.rs l.66-74, usb_hid.rs) : CMD_NEXT n'est jamais ramené à 0 ; au-delà de 255 commandes, l'écriture déborde sur la page 2 (anneau d'événements). Correction : placer un Link TRB (type 6, TC=1) à l'index 255 ou boucler avec cycle inversé.
3. **[Faible] Pagesize non lu** : scratchpad et alignements supposent 4 KiB (PSZ=12). Correction : lire PAGESIZE (op+08h) et calculer PSZ = 12 + décalage ; refuser si la page n'est pas 4 KiB.
4. **[Faible] Vitesse → MPS EP0** (usb_r0.rs l.113-117) : mapping fixe 3=HS(64), 4=SS(512), sinon 8. Spec §7.2.2.1.1 : mapping par défaut seulement si PSIC=0 ; pour SSP (valeurs 5-7) la MPS EP0 doit être 512 et non 8. Correction : utiliser la capability Supported Protocol correspondante (PSIV/PSIC).
5. **[Faible] CIE/CIC non renseignés** (usb_hid.rs Configure Endpoint) : xHCI 1.1+ demande CIE=1 et les champs Configuration Value / Interface / Alt. Setting (Input Control DW7). Pour un clavier boot, la valeur configurée doit être bConfigurationValue ; à ajouter si CIC=1.
6. **[Info] EHB écrit à 1** dans ERDP (usb_r0.rs l.491) : RW1C, donc correct (efface EHB). Pas de correction nécessaire.
7. **[Info] IMOD non écrit** : valeur par défaut 4000 (~1 ms) conservée. Conforme (§5.5.2.2).

## Non vérifié / hors relecture
- usb_hub.rs (Configure Endpoint hub, DW7), interrupt_interval() (table 6-12 : FS/LS interrupt bInterval 1..255 → Interval 3..10), control() (Setup TRB IDT=1, length 8, TRT, DIR des Data/Status), set_device_base().
- HDA : inversion des champs de nœuds subordonnés (usb_r0.rs l.531-540) : **inversion confirmée** par la relecture précédente, non corrigée (décision utilisateur en attente).
