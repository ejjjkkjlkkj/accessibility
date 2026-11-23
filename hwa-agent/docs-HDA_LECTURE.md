# Lecture ligne à ligne — Intel High Definition Audio Specification Rev 1.0a (10 233 lignes)

Source : <DOSSIER_SPECS>\high-definition-audio-specification.txt
Statut : en cours (lignes 1 à 3200 lues). [AGENT] = utile au pilote de l'agent.

## Ch. 1-2 Introduction et architecture
- HD Audio ≠ AC'97 (pas de compatibilité, pas de mélange sur le même lien).
- Contrôleur : maître bus, DMA par stream. Lien : BCLK 24 MHz, SYNC 48 kHz, SDO (sortie, double pompé), SDI (entrée par codec, simple pompé), RST#.
- 15 streams entrée + 15 sortie simultanés ; échantillons 8/16/20/24/32 bits ; jusqu'à 16 canaux par stream ; 6 kHz à 192 kHz.
- Débit : 48 Mbit/s par SDO, 24 Mbit/s par SDI ; striping SDO 2 ou 4.
- Multi-SDI pour codecs à grande bande passante.
- Codec : fonction groups (audio, modem, vendeur). Découverte complète par verbes.

## Ch. 3 Registres du contrôleur [AGENT]
- Règles d'accès : toujours alignés sur leur taille (byte/word/dword). RsvdP = read-modify-write ; RsvdZ = écrire 0 ; RO dans un registre RW = écrire la valeur lue.
- Types : RO, RW, RW1C, ROS, RWS, RW1CS, HWINIT, RsvdP, RsvdZ, RSM (bits dans le "resume well").
- 64 bits : GCAP bit 0 = 64OK. Registres d'adresses 64 bits : partie haute à 0 par défaut.
- Table des registres (offsets) :
  - 00h GCAP (2 o) : bits 15:12 OSS, 11:8 ISS, 7:3 BSS, 2:1 NSDO (00=1 SDO, 01=2, 10=4), 0 64OK.
  - 02h VMIN = 00h ; 03h VMAJ = 01h.
  - 04h OUTPAY (2 o) : capacité sortie en mots 16 bits par trame 48 kHz (défaut 24 MHz -> 60 mots utiles).
  - 06h INPAY (2 o) : capacité entrée (29 mots utiles par trame).
  - 08h GCTL (4 o) : bit 8 UNSOL (accepter réponses non sollicitées) ; bit 1 FCNTRL (flush) ; bit 0 CRST (0 = reset, 1 = sortie de reset ; lire 1 avant d'accéder aux registres).
  - 0Ch WAKEEN (2 o) : bits 14:0 SDIWEN (masque wake par SDI).
  - 0Eh STATESTS (2 o) : bits 14:0 SDIWAKE (RW1CS), indique les SDI ayant changé d'état (codec présent).
  - 10h GSTS (2 o) : bit 1 FSTS (flush terminé, RW1C).
  - 18h OUTSTRMPAY, 1Ah INSTRMPAY : limite de mots par trame par stream (0 = pas de limite).
  - 20h INTCTL (4 o) : bit 31 GIE, bit 30 CIE, bits 29:0 SIE (un bit par stream, ordre : entrées, sorties, bidirectionnels).
  - 24h INTSTS (4 o) : bit 31 GIS (OR de tout), bit 30 CIS, bits 29:0 SIS.
  - 30h WALCLK (4 o) : compteur 32 bits à la fréquence BCLK (rollover ~179 s). Alias 2030h.
  - 38h SSYNC (4 o) : bits 29:0, bloquent les streams pour démarrage/arrêt synchrones.
  - 40h CORBLBASE / 44h CORBUBASE (base 1 KB ; bits 6:0 forcés à 0 = alignement 128 o).
  - 48h CORBWP (1 o) ; 4Ah CORBRP (bit 15 CORBRPRST, bits 7:0 RP) ; 4Ch CORBCTL (bit 1 CORBRUN, bit 0 CMEIE) ; 4Dh CORBSTS (bit 0 CMEI, RW1C) ; 4Eh CORBSIZE (bits 7:4 capacité 2/16/256 entrées ; bits 1:0 taille choisie 00=2, 01=16, 10=256).
  - 50h/54h RIRBLBASE/UBASE (base 2 KB, alignement 128 o) ; 58h RIRBWP (bit 15 RIRBWPRST, bits 7:0 WP) ; 5Ah RINTCNT (nombre de réponses avant IRQ, 0 = 256) ; 5Ch RIRBCTL (bit 2 RIRBOIC, bit 1 RIRBDMAEN, bit 0 RINTCTL) ; 5Dh RIRBSTS (bit 2 RIRBOIS RW1C, bit 0 RINTFL RW1C) ; 5Eh RIRBSIZE.
  - 70h DPLBASE (bits 31:7 base, bit 0 DMA position enable) ; 74h DPUBASE.
  - Stream descriptors commencent à 80h, taille 20h chacun : entrées (x = 80h + ISS*20h), sorties (y), bidirectionnels (z).
    - +00 CTL (3 o) : bits 23:20 STRM (numéro de stream, 0 réservé) ; bit 19 DIR (bidir) ; bit 18 TP ; bits 17:16 STRIPE ; bit 4 DEIE ; bit 3 FEIE ; bit 2 IOCE ; bit 1 RUN ; bit 0 SRST.
    - +03 STS (1 o) : bit 5 FIFORDY (RO), bit 4 DESE (RW1C), bit 3 FIFOE (RW1C), bit 2 BCIS (RW1C).
    - +04 LPIB (4 o) : position dans le buffer cyclique (octets).
    - +08 CBL (4 o) : taille du buffer cyclique (octets).
    - +0C LVI (2 o) : index du dernier BDL entry valide (≥ 1).
    - +10 FIFOS (2 o) : taille FIFO (octets, impl.dép).
    - +12 FMT (2 o) : bit 14 BASE (0 = 48 kHz, 1 = 44.1 kHz), bits 13:11 MULT, bits 10:8 DIV, bits 6:4 BITS (000 8b, 001 16b, 010 20b, 011 24b, 100 32b), bits 3:0 CHAN (nb canaux - 1).
    - +18 BDPL (bits 31:7), +1C BDPU : adresse BDL (128 o).
  - Bits de contrôle à conserver : éviter de modifier CBL, LVI, BDL quand RUN = 1.
- Immediate Command (PIO, optionnel) : 60h ICW (verbe), 64h IRR (réponse), 68h ICS (bit 0 ICB, bit 1 IRV RW1C, bit 2 ICVER, bit 3 IRRUNSOL, bits 7:4 IRRADD). Ne pas utiliser en même temps que CORB/RIRB.
- Arbre d'interruptions (fig. 4) : bit d'état -> bit d'activation -> GIE -> ligne PCI ou MSI.
- Structures : DMA position buffer (un DWORD par stream, 128 o aligné) ; BDL (≥ 2 entrées, max 256, 128 o) ; BDLE = adresse 64 bits (alignée 128 o), longueur 32 bits (multiple de mot), bit IOC ; CORB (entrées 32 bits) ; RIRB (entrées 64 bits : réponse 32 bits + Resp_Ex : bits 3:0 SDI n°, bit 4 unsolicited).
- Format verbe sortant : bits 31:28 adresse codec, 27:20 Node ID, 19:0 payload.
- Réponse solicitée : 32 bits opaques. Réponse non sollicitée : bits 31:26 Tag, 25:21 Sub Tag, 20:0 vendeur.

## Ch. 4 Programmation [AGENT]
- 4.2 Init : configurer PCI, puis CRST=1 et attendre que CRST relise 1 avant d'accéder aux registres (ni plus de 521 µs après lecture de CRST pour la découverte des codecs = 25 trames).
- 4.3 Découverte : STATESTS indique les SDI présents (ex. 05h = codecs 0 et 2).
- 4.4.1 CORB : écriture à WP+1 ; WP mis à jour par logiciel ; hardware avance RP ; CORBRUN doit être posé ; reset RP via CORBRPRST (lire 1 puis remettre à 0 et vérifier).
  - Insérer des verbes NULL (00000000h) pour laisser le codec émettre des non-sollicitées.
- 4.4.2 RIRB : hardware avance WP ; logiciel garde RP ; interruption après RINTCNT réponses ou quand plus de réponses sur les SDI.
- 4.5.1 Données : échantillons alignés, containers 8/16/32 bits (LSB à 0 pour justifier) ; bloc = container × canaux ; paquet = plusieurs blocs par trame (48 kHz de base) ; buffer multiple de 128 o.
- 4.5.3 Démarrage : allouer buffer + BDL, configurer descripteur (FMT, BDL, CBL, LVI), configurer converters du codec (stream id + format), puis RUN=1.
- 4.5.4 Arrêt : RUN=0 ; transition en < 40 µs (fin de trame).
- 4.5.6 Régime : ISR en accès byte pour le statut ; ne pas écrire le registre CTL dans l'ISR.
- 4.5.7 Synchronisation : SSYNC pour démarrer deux streams au même moment ; attendre FIFORDY ; effacer SSYNC.
- 4.5.8 Power : après D3 -> D0, restaurer tous les registres codec (seuls les RSM sont préservés).
- 4.5.9 Wake : S0/D0 -> réponse non sollicitée ; D3 (CRST=1) -> change d'état sur SDI ; S3 -> PME.

## Ch. 5 Protocole de lien (5.2 à 5.2.3)
- Bits par trame : 500 cellules SDI, 1000 cellules SDO (double pompé).
- SDI : chaque codec a son propre SDI ; contrôleur doit avoir des pull-down faibles sur SDI (anti parasites wake).
- Basique : 1 SDO commun ; 1 SDI par codec ; jusqu'à 15 codecs.
- Multi-SDO / multi-SDI : SDO0 et au moins un SDI obligatoires pour tout codec.
- Pas d'intégration Dock avec hot-plug connue ici ; voir Ch 5.5.

## Lignes 3201-4800 — résumé (5.2.3 timing à 6.4 mesures)
- Timing : SDO double pompé (bit n+1 posé sur front 2, échantillonné front 3) ; SDI simple pompé (front montant) ; trame = 20,833 µs ; SYNC haut 4 cycles BCLK (8 bits SDO) marque le début.
- 5.3 Trame : [Command/Response] puis paquets stream (tag + blocs) puis Null (zéros). Pas de null entre paquets.
- Commande sortante 40 bits (verbe 32 + 8 réservés) ; réponse entrante 36 bits (32 + 4).
- Tag de stream sortant : préambule 4 bits (3 hauts + 1 bas) puis ID 4 bits, transmis sur SYNC pendant les 8 derniers bits de la trame précédente.
- Tag entrant : 4 bits ID + 6 bits longueur en octets (10 bits, en ligne sur SDI). Longueur 0 = fin de données de la trame. Padding de 0 à l'octet (max 4 bits) en fin de paquet.
- Striping SDO : bit i sur SDO (i mod n) ; la commande reste sur SDO0 (copiée sur les autres). Règle de dimensionnement : nb canaux × bits / nb SDO ≥ 8 (48 kHz).
- 5.4 Taux non 48 kHz : base 48 kHz = BCLK/500 ; base 44,1 kHz = 147/160 × 48 kHz ; cadence "12-11" pour 44,1 kHz (12-11-11-12-11-11-12-11-11-12-11-11-11 répété ; 147 blocs / 160 trames).
- 5.4.3 Source synchrone (S/PDIF, modem) : allocation nominale, extra blocs seulement si la place le permet.
- 5.5 Reset : entrée en reset = fin de trame sans SYNC, RST# asserté 4 cycles plus tard, BCLK arrêté 4 fronts après. Sortie : BCLK stable 100 µs (2400 cycles) avant RST# high ; SYNC à partir de ≥ 4 cycles après.
- 5.5.2 Function Group Reset (verbe FUNCTION_RESET) : reset d'un function group sans toucher au lien ; deux resets = reset complet.
- 5.5.3 Initialisation codec : trames Connect (codec lève SDI pendant le dernier cycle du Frame Sync, dans les 25 premières trames), Turnaround (SDI retourné), Address (contrôleur maintient SDIn haut n cycles -> adresse n ; codec compte 0..14 ; 15 réservé). Codec sans adresse = ignore le lien (pas de nouvelle tentative avant nouveau reset).
- 5.6 Power : codec en veille peut signaler wake (SDI haut en asynchrone si BCLK arrêté, ou non-sollicité si BCLK actif).
- 6.2 3,3 V : Vcc 3,135–3,465 ; Vih ≥ 0,65 Vcc ; Vil ≤ 0,35 Vcc ; Voh ≥ 0,9 Vcc ; Vol ≤ 0,1 Vcc ; Cin 7,5 pF ; slew 1–3 V/ns ; AC rating 6,3 V, 60 Ω.
- 6.3 1,5 V : Vcc 1,418–1,583 ; Vih 0,6 Vcc ; Vil 0,4 Vcc ; slew 0,5–1,5 V/ns ; Rs : 65, 101, 108, 133 Ω selon point de mesure.
- 6.4 : mesures au point 0,5 Vcc ; charge de test identique 1,5 V / 3,3 V.
- [AGENT] : signaux physiques hors du pilote (électrique) ; mais le contrôleur doit lever RST# / arrêter BCLK selon les séquences ci-dessus si on implémente un reset de lien.

## Lignes 4801-6400 — résumé (6.4 à 7.3.3.9)
- Table 67 (conditions de mesure) : 1,5 V → Vh 0,7 Vcc, Vl 0,3 Vcc, Vtest 0,5 Vcc ; 3,3 V → Vh 0,75 Vcc, Vl 0,25 Vcc, Vtest 0,5 Vcc. Overdrive d'entrée 0,1 Vcc.
- 6.5 Table 68 (contrôleur) : BCLK 23,9976–24,0024 MHz ; Tcyc 41,363–41,971 ns (typ 41,67) ; Thigh/Tlow min 18,75 ns, max 22,91 ns ; jitter max 300 ps (typ 150) ; Tval min 7 ns (SDO valide avant le front) ; Tsu SDI min 15 ns ; Th SDI min 0 ns.
- Table 69 (codec) : Tcyc 41,163–42,171 ns ; Tco (SDI valide après front BCLK) 17,5 à 24,16 ns ; Tsu SDO min 11 ns ; Th SDO min 5 ns (valeurs extraites du PDF, à revérifier sur la page 117 si besoin).
- Notes : duty cycle contrôleur 45/55 %, codec 42/58 % ; fréquence moyenne sur 1 ms ; tolérance 100 ppm ; slew 1–3 V/ns (3,3 V), 0,5–1,5 V/ns (1,5 V).
- Fig. 42/43 : SDO et SYNC/RST# ont les mêmes définitions que SDO (double pompage) ; SDI défini seulement au front montant.
- 6.6 : fournisseur codec doit fournir IBIS, V/I, slew, valeurs absolues max.
- 6.8 : même alimentation et masse commune pour tous les composants ; 3,3 V ou 1,5 V non mélangés.
- 6.9 : skew total BCLK–SDO ±2 ns (Table 70) ; flight time max BCLK contrôleur→codec 7 ns, SDI codec→contrôleur 7 ns (Table 71) ; aller-retour max 14 ns.
- 6.10 : impédance cible 55–60 Ω ±15 % ; retard aller-retour BCLK/SDI ≤ 14 ns ; dégagement de découplage à < 0,5 pouce si changement de plan de référence.
- 6.11 : topologies (figures 44–46) : étoile desktop/mobile, branches avec résistances 33 Ω/39 Ω/0 Ω selon configuration.
- 6.12 : hot-attach : solution asynchrone (tri-state, buffer ajoute un délai) ou synchrone (flip-flops, deux au minimum, reset DOCK 1 ms via R2·C1·ln).
- 7.1 : architecture : nœud racine NID 0 → function groups → widgets ; adresse codec CAd (4 bits) + NID (7 ou 15 bits) ; CAd 15 = broadcast ; verbe : bits 31:28 CAd, bit 27 indirect, bits 26:20 NID, bits 19:0 verbe/données.
- 7.1.3 règles d'interconnexion : widgets exclusifs à un function group ; stéréo→stéréo L→L, R→R ; mono→stéréo : source sur les deux canaux ; stéréo→mono : seul L (sauf mixer mono à 1 entrée qui mélange L et R en deux entrées).
- 7.2.2 AFG : widgets convertisseurs, pin, mixer, sélecteur ; VSM-FG : verbes obligatoires F00, F05/705, F08/708, F20, Function Reset 7FFh.
- 7.3.1 réponse obligatoire à la trame suivante : champ réponse 36 bits (bit 35 Valid, bit 34 UnSol, bits 31:0 données). Verbe invalide = bits 27:0 tous à 0 vers NID 0. Verbe Pin Sense Execute : réponse immédiate 0, valeur à relire plus tard.
- 7.3.2 multi-SDI : un root node peut avoir plusieurs SDI ; réponses toujours sur le SDI primaire.
- 7.3.3 verbes (format 4 bits ou 12 bits) :
  - F00 Get Parameter ; F01/701 Connection Select ; F02 Get Connection List Entry (4 entrées short-form ou 2 long-form par réponse).
  - F03/703 Processing State (00 off, 01 on, 02 benign).
  - D/5 Coefficient Index ; 4h Processing Coefficient.
  - Bh/3h Amp Gain/Mute : get bit 15 = output(1)/input(0), bit 13 = left(1)/right(0), index 3:0 ; set bit 15 out, 14 in, 13 left, 12 right, 11:8 index, 7 mute, 6:0 gain. Reset : gain = offset, mute = 1 en général.
  - Ah/2h Converter Format (même format que le descripteur de stream).
  - F0Dh/70Dh/70Eh/73Eh/73Fh : S/PDIF Converter Control 1–4 (bits SIC).
  - Tables 80 : préambules S/PDIF B/M/W encodés sur les 4 bits de poids faible de la trame.
- [AGENT] : le pilote doit toujours lire une réponse par commande ; CORB/RIRB n'est pas seulement un tampon, la réponse doit suivre la trame suivante (timeout = erreur). Vérifier dans usb_r0.rs / code HDA que les verbes Get Amp, Connection Select et Format sont traités aux bons payloads.

## Lignes 6401-8000 — résumé (SIC S/PDIF, power states, verbes 7.3.3.10 à 7.3.3.33)
- F0Dh/70Dh etc. : bits SIC (KAE, ICT, CC[6:0], L, PRO, /AUDIO, /CP, PRE, VCFG, V, DigEn). KAE : horloge et silence continus en D0–D3 sur sortie numérique ; non applicable aux entrées. VCFG/V règlent le bit Validity (bit 28 de sous-trame). DigEn = 0 bloque le flux.
- F05h/705h Power State : PS-Set bits 3:0, PS-Act bits 7:4, PS-Error bit 8, PS-ClkStopOk bit 9 (Function Group seulement), PS-SettingsReset bit 10.
- États : D0 pleine puissance ; D1 retour D0 en 1 ms ; D2 retour en 2 ms ; D3 (D3hot) ; D3cold optionnel (sort seulement par POR, de-assertion RST#, ou double FG reset ; retour D0 sous 200 ms).
- Règle : le bus ne doit pas être arrêté si ClkStopOk=0 ; sinon reset complet. EPSS : retour D3→D0 < 10 ms ; PS-Error signale un refus.
- Table 84 : persistance des réglages (format convertisseur, stream/channel, select index, amp gain/mute, pin control, etc.) à travers Dx et reset.
- F06h/706h Converter Stream/Channel : stream bits 7:4 (0 réservé), channel bits 3:0 (canal gauche ; droite = canal+1).
- F04h/704h Input Converter SDI Select (multi-SDI).
- F07h/707h Pin Widget Control : H-Phn Enable bit 7, Out Enable bit 6, In Enable bit 5, VRefEn bits 4:3 (Table 88 : 000 Hi-Z, 001 50 %, 010 GND, 100 80 %, 101 100 %), EPT bits 1:0 (HDMI/DP).
- F08h/708h Unsolicited Response : Enable bit 7, Tag bits 5:0 (renvoyé en bits 31:26 de la réponse non sollicitée).
  - UR intrinsèque : sub-tag 0 ; bit 0 PD (Presence Detect), bit 1 ELDV ; une seule UR en attente à la fois.
  - UR CP : sub-tag non nul via Set CP ; READY bit 0→1 déclenche UR.
- F09h/709h Pin Sense (Execute) : bit 31 PD, bit 30 ELD valid (digital), bits 30:0 impédance (analogique) ; 7FFFFFFFh = pas de mesure valide.
- F0Ch/70Ch EAPD/BTL : bit 2 L-R Swap, bit 1 EAPD, bit 0 BTL.
- F10h/710h GPI Data ; F11h/711h GPI Wake ; F12h/712h GPI Unsolicited ; F13h/713h GPI Sticky ; F14h/714h GPO Data.
- F15h/715h GPIO Data ; F16h/716h GPIO Enable ; F17h/717h GPIO Direction ; F18h/718h GPIO Wake ; F19h/719h GPIO Unsolicited ; F1Ah/71Ah GPIO Sticky (1 = sticky, edge).
- F0Ah/70Ah Beep Generation : diviseur 8 bits ; fréquence = 48 kHz / (4 × diviseur) ; 00h = arrêt ; 47 Hz à 12 kHz.
- F0Fh/70Fh Volume Knob : bit 7 Direct, bits 6:0 Volume.
- F20h/720h–723h Implementation ID (32 bits, lecture/écriture ; 0xFFFFFFFF possible jusqu'à 7 ms après RST#).
- F1Ch/71Ch–71Fh Configuration Default (32 bits du Pin Widget) : bits 31:30 Port Connectivity (00 jack, 01 pas de connexion, 10 périphérique interne, 11 jack+interne) ; 29:24 Location (bits 5:4 gross : 00 external, 01 internal, 10 separate chassis, 11 other ; bits 3:0 géométrique) ; 23:20 Default Device (0h Line Out, 1h Speaker, 2h HP Out, ..., Ah Line In, Bh AUX, Ch Mic In, ...) ; 19:16 Connection Type (1h 1/8", 2h 1/4", 4h RCA, 5h optical, ...) ; 15:12 Color ; 11:8 Misc (bit 0 = pas de détection de présence) ; 7:4 Default Association (0000 réservé, 1111 = priorité la plus basse) ; 3:0 Sequence.
- F24h/724h Stripe Control : Stripe Capability bits 2:0 (bit0 = SDO0 toujours, bit1 = SDO0–1, bit2 = SDO0–3) ; Stripe Control bits 1:0 ; doit matcher le champ Stripe Control du descripteur de stream.
- 7FFh Function Reset : remet le function group et ses widgets aux valeurs POR, sans toucher la logique de lien (qui n'est reset que par RST#).
- [AGENT] : pour un pilote minimal il faut au moins F00 (paramètres, dont Subordinate Node Count 04h, Function Group Type 05h, Widget Capabilities 09h, Pin Capabilities 0Ch, Config Default), F01/701 (connection select), F02, F07/707 (pin enable), F0C/70C (EAPD), F1C/71C (config default), Bh/3h (amp), Ah/2h (format), 706h (stream/channel) et 7FFh (reset). Vérifier que le code source ne confond pas les masques de payload (16 bits pour 3h/Bh, 8 bits pour 70x).

## Lignes 8001-10233 — résumé (fin 7.3.3 à 7.6, fin du document)
- 7FFh Function Reset : réponse obligatoire puis reset ; le codec doit répondre au verbe suivant la trame qui suit la réponse. Avec EPSS, un reset simple ne réinitialise pas les réglages persistants ; un « double Function Reset » (deux 7FFh) fait l'init complète (sauf Implementation ID et Configuration Default). Un reset de lien efface la mémoire d'un reset simple.
- 7.3.3.34 ELD (F2Fh) : payload = offset d'octet ; réponse bit 31 = ELD valid, bits 7:0 = octet. Structure : en-tête 4 octets (ELD_Ver, Baseline_ELD_Len en DW), bloc baseline (MNL, CEA_EDID_Ver, SAD_Count, Aud_Synch_Delay, Conn_Type, S_AI, HDCP, Port_ID, Manufacturer, Product Code, Monitor Name, CEA_SADs), bloc vendeur. Taille via DIP-Size bit 3 = 1.
- 7.3.3.35 Converter Channel Count (F2Dh/72Dh) : nombre de canaux 0-based, combiné au Channel de F06h.
- 7.3.3.36 DIP-Size (F2Eh) ; 7.3.3.37 DIP-Index (F30h/730h, PI bits 7:5, byte index 4:0, auto-incrément, wrap à 32) ; 7.3.3.38 DIP-Data (F31h/731h) ; 7.3.3.39 DIP-XmitCtrl (F32h/732h : 00 désactivé, 10 une fois, 11 best effort). HDMI doit avoir au moins 4 buffers DIP.
- 7.3.3.40 CP Control (F33h/733h) : bit 9 CES, bit 8 READY, bits 7:3 sub-tag UR, bits 1:0 CP state (01 off, 10 on). Séquence : vérifier READY avant d'envoyer, UR quand READY passe 0→1.
- 7.3.3.41 ASP channel mapping (F34h/734h) ; mapping par défaut : canal 0→slot 0, 1→1, 2→3, 3→2, 4..7→4..7 ; valeur F ou > count = non alloué.
- 7.3.4 paramètres (ID) : 00 Vendor ID (bits 31:16 vendor, 15:0 device) ; 02 Revision ID ; 04 Subordinate Node Count (bits 23:16 Starting Node, bits 7:0 Total) ; 05 FG Type (01 Audio FG, 02 Modem FG, bit 8 UnSol Capable) ; 08 AFG Capabilities (bits 15:12 beep gen? bits 11:8 Output Delay, 7:4 Input Delay) ; 09 Audio Widget Capabilities (bits 23:20 Type, 19:16 Delay, 15:13 & bit 0 = nombre de canaux-1, bit 12 CP Caps, bit 11 L-R Swap, bit 10 Power Cntrl, bit 9 Digital, bit 8 ConnList, bit 7 UnSol, bit 6 Proc, bit 5 Stripe, bit 4 Format Override, bit 3 Amp Param Override, bit 2 Out Amp, bit 1 In Amp, bit 0 Stereo) ; 0Ah PCM size/rates (bits B32..B8 et R1..R12, R7 = 48 kHz obligatoire) ; 0Bh Stream Formats (bit 0 PCM, bit 1 Float32, bit 2 AC3) ; 0Ch Pin Capabilities (bit 0 Impedance Sense, bit 1 Trigger Req, bit 2 Presence Detect, bit 3 Headphone Drive, bit 4 Output Capable, bit 5 Input Capable, bit 6 Balanced I/O, bit 7 HDMI, bits 15:8 VRef, bit 16 EAPD, bit 24 DP, bit 27 HBR) ; 0Dh/12h Amp Capabilities (bit 31 Mute, bits 22:16 StepSize, 14:8 NumSteps, 6:0 Offset) ; 0Eh Connection List Length (bit 7 long form, 6:0 count) ; 0Fh Supported Power States (bit 0 D0, bit 1 D1, bit 2 D2, bit 3 D3, bit 4 D3cold, bit 30 CLKSTOP, bit 31 EPSS) ; 10h Processing Caps ; 11h GP I/O Count ; 13h Volume Knob Caps ; 20h HDMI LPCM CAD (obsolète).
- 7.3.5 : verbes vendeurs F70h–FFFh sur les types de widget audio ; pas de verbe vendeur pour une fonction déjà couverte.
- 7.3.6 Table 140/141 : verbes requis par type de nœud : Root Node (F00, 7FFh) ; AFG (F00, F05, F20/720-723, 7FFh, etc.) ; Output Converter (F00, F02 si liste, A/2 format, F06/706, 7FFh, F24/724 stripe si stripe bit) ; Pin Widget (F00, F02, F07/707, F09/709, F1C/71C config default, F0C/70C, F08 si UnSol) ; Power Widget (F05/705, F00).
- 7.4.1 pinout standard 48 broches : PORT-A doit être sortie ; PORT-B/C re-taskable (VrefOut) ; SDO, BITCLK, SDI, SYNC, RESET#, PCBEEP, SPDIF. Straps : pins 33 et 40, pull-up 1 kΩ, pull-down faible pendant RST#.
- 7.4.2 détection de jack : résistances 39,2 kΩ, 20 kΩ, 10 kΩ, 5,1 kΩ (±1 %) sur les pins Sense ; débounce 50 ms ; UR dans les 10 ms après stabilité ; si BCLK arrêté, la demande de wake doit partir dans les 10 ms.
- 7.5 : Controller Reset ne doit pas rétablir l'état primitif (Implementation ID et Configuration Default conservés).
- 7.6 Mode test : entrée par séquence RST# et SDO/SDI sur fronts BCLK (≥ 1 BCLK, ≥ 100 µs) ; codes : 00 normal, 10 ATE, 11 vendeur. Non requis.
- [AGENT] : lecture complète du verbe obligatoire de réponse + reset de function group + Config Default + Pin/Amp/Format : ces verbes forment la base minimale à implémenter et tester. À cross-checker avec usb_r0.rs / usb_hid.rs seulement si un pilote HDA est écrit ; aucun pilote HDA n'est présent dans le code lu.

## Statut final
- Lecture HDA terminée (lignes 1 à 10233). Notes complètes dans ce fichier.
- Points restant à vérifier sur la page : valeurs Table 69 (codec) extraites avec ordre de colonnes approximatif (Tsu, Th) ; à revérifier dans le PDF avant tout calcul de timing.
