# Lecture ligne à ligne — NVM Express over PCIe Transport 1.4 (2116 lignes)

Source : <DOSSIER_SPECS>\NVMe_PCIe_Transport_1.4.txt
Statut : lu en entier (lignes 1 à 2116). [AGENT] = utile au pilote de l'agent.

## Préliminaires
- Révision 1.4 (31 juillet 2026) : intègre 1.3, TP4195, TP4213, ECN131, ECN137.
- Priorité en cas de conflit : non-NVMe > Base > Transport > I/O Command Set > Management > Boot.
- Références : NVMe Base 2.4, PCIe Base 6.2, M.2 5.0, ACPI 6.5.

## 2 Transport
- Le transport PCIe repose sur : MMIO (données + registres), espace de configuration PCI, interruptions message (MSI/MSI-X).

## 3.1 Setup et initialisation
- Structures PCI : PMCAP, MSICAP, MSIXCAP, PXCAP, AERCAP (figure 3).
- MSI-X recommandé ; MSI possible en plus. Une seule interruption (pin, MSI simple ou MSI-X simple) tant que le nombre de files I/O n'est pas connu. [AGENT]
- Registres contrôleur : mappés via BAR0/BAR1 (MLBAR/MUBAR), à partir de l'offset 1000h.
- Accès : mémoire non cacheable, ordre conservé, largeur native ou accès 32 bits alignés. Pas d'accès verrouillés. Un accès qui touche deux registres n'est pas supporté. Bits réservés = lecture seule à 0.
- Doorbells [AGENT] :
  - SQyTDBL (SQ y Tail) à 1000h + (2y × (4 << CAP.DSTRD)) ; écriture 16 bits (SQT). Lecture non significative. Gérer le rollover.
  - CQyHDBL (CQ y Head) à 1000h + ((2y+1) × (4 << CAP.DSTRD)) ; écriture 16 bits (CQH). Gérer le rollover.
  - y = identifiant de file. Admin = y 0 (SQ0TDBL, CQ0HDBL). SQ1/CQ1 et suivantes optionnelles.
- Contrôleur administratif : CC.PI permet un pilote de gestion dédié.

## 3.2 Modèle de files
- Paires SQ/CQ en mémoire hôte. Plusieurs SQ peuvent partager une CQ.
- Create I/O Completion Queue, dword 11 : IEN = interruptions activées ; IV (Interrupt Vector) bits 31:16.
  - IV = 0 pour pin ou MSI simple. MSI-X : jusqu'à 2048 vecteurs. IV supérieur au nombre de messages supportés -> statut Invalid Interrupt Vector. [AGENT] : vérifier IV ≤ MSIXCAP.MXC.TS + 1.

## 3.3 Réinitialisations
- Reset de niveau contrôleur : Conventional Reset PCI et Function Level Reset (FLR). L'espace de configuration PCI est remis à zéro selon PCIe.
- NVM Subsystem Reset : tous les liens PCIe passent en LTSSM Detect.

## 3.4 Traitement des commandes (8 étapes)
1. L'hôte écrit la commande dans la prochaine case SQ libre.
2. L'hôte écrit SQyTDBL.
3. Le contrôleur lit la commande (arbitrage).
4. Exécution ; les commandes peuvent se terminer dans le désordre.
5. Le contrôleur écrit l'entrée CQ (SQ Head, Phase Tag inversé à chaque tour).
6. Interruption optionnelle (coalescing possible).
7. L'hôte consomme les entrées CQ tant que la phase correspond.
8. L'hôte écrit CQyHDBL (il peut regrouper plusieurs entrées).
- Réutilisation : une fois la CQE postée, les listes PRP de la commande sont réutilisables (sauf celles des SQ/CQ I/O).

## 3.5 Interruptions
- Quatre modes : pin, MSI simple, MSI multiple, MSI-X (recommandé).
- Agrégation (feature Interrupt Coalescing) : seuil et temps recommandés ; la CQ Admin n'est pas retardée.
- Pin / MSI simple / MSI multiple : registre IS interne (invisible) ; IS[v] = 1 si une CQE non acquittée existe sur un vecteur v, que la CQ a les interruptions activées et que INTM n'est pas masqué. Acquittement = écriture de CQyHDBL pour toutes les CQE du vecteur.
- INTM (INTMS/INTMC) masque avant la logique MSI : un vecteur masqué ne lève pas le bit Pending MSI.
- MSI-X : envoi si Function Mask = 0 et masque du vecteur = 0 ; sinon bit pendant dans la PBA. Recommandé : masquer le vecteur pendant le traitement des CQE.
- MSI multiple : MMC = log2 arrondi au-dessus du nombre de vecteurs demandés (3 -> 010b = 4). Jusqu'à 32 vecteurs.

## 3.6 Gestion de puissance
- États NVMe + spécificités PCIe. RTD3 = D3cold. D3hot ne retire pas l'alimentation principale.
- Un hôte ne doit jamais choisir un état qui dépasse la limite CSPLV × CSPLS (PXDCAP). Sinon comportement indéfini.
- Si DPA (Dynamic Power Allocation) est actif : puissance max = min(valeur DPA, état NVMe).
- Hôte sans gestion dynamique : choisir l'état le plus bas-numéroté respectant la limite.
- Restrictions de sécurité (ex. Write Protect Until Power Cycle) levées à chaque power cycle ; leur maintien dépend du firmware système, hors périmètre NVMe.

## 3.7 Erreurs
- AER recommandé.

## 3.8 Espace de configuration PCI (Type 0) — offsets utiles [AGENT]
- 00h ID : VID bits 15:00, DID bits 31:16.
- 04h CMD : bit 10 ID, bit 2 BME, bit 1 MSE, bit 0 IOSE. 06h STS : bit 4 CL, bit 3 IS.
- 08h RID. 09h CC : BCC 23:16, SCC 15:08, PI 07:00 (PI sert à distinguer le contrôleur admin).
- 0Ch CLS. 0Dh MLT. 0Eh HTYPE : bit 7 MFD, bits 6:0 HL.
- 0Fh BIST optionnel (lecture 0 si absent).
- 10h MLBAR (BAR0) : bits 31:14 BA ; bit 3 Prefetchable = 0 ; bits 2:1 TP (64 bits recommandé) ; bit 0 = 0 (mémoire).
- 14h MUBAR (BAR1) : 32 bits hauts. Derrière un bridge PCI, BAR 32 bits seulement.
- 18h BAR2 optionnel (index/data I/O ou vendeur). 1Ch à 27h BAR3 à BAR5 vendeur.
- 28h CCPTR = 0. 2Ch SS (SSID, SSVID). 30h EROM optionnel. 34h CAP (pointeur de capabilities). 3Ch INTR (IPIN RO, ILINE RW). 3Eh MGNT = 0. 3Fh MLAT = 0.
- PMCAP (gestion d'énergie PCI) : PID 01h ; PC : D1S/D2S non recommandés ; VS >= 1.2 ; PMCS : PS bits 1:0 (00 D0, 01 D1, 10 D2, 11 D3hot) ; NSFRST bit 3.
- MSICAP : MID 05h ; MC : MSIE bit 0, MMC bits 3:1, MME bits 6:4, C64 bit 7 (obligatoire pour NVMe), PVM bit 8 ; MA, MUA, MD ; MMASK et MPEND optionnels.
- MSIXCAP : MXID 11h ; MXC : TS bits 10:0 (taille = valeur + 1), FM bit 14, MXE bit 15 ; MTAB (TBIR bits 2:0, offset bits 31:3) ; MPBA (PBIR, offset). Table et PBA alignées sur 4 KiB ; conseil : les placer dans BAR0 après les doorbells.
- PXCAP (ID 10h) : PXCAP (DPT = endpoint, VER = 2h) ; PXDCAP : FLRC bit 28 = 1 obligatoire, CSPLV bits 25:18, CSPLS bits 27:26, MPS bits 2:0, ETFS bit 5, RER bit 15 = 1 obligatoire ; PXDC : IFLR bit 15, MRRS 14:12, ENS 11, APPME 10, PFE 9, ETE 8, MPS 7:5, ERO 4, URRE 3, FERE 2, NFERE 1, CERE 0 ; PXDS : TP bit 5, APD 4, URD 3, FED 2, NFED 1, CED 0 ; PXLCAP : PN 31:24, ASPMS 11:10, L1EL 17:15, L0SEL 14:12, MLW 9:4, SLS 3:0 ; PXLC : ASPMC 1:0, RCB bit 3 ; PXLS : CLS 3:0, NLW 9:4 ; PXDCAP2 : OBFFS, LTRS, CTDS, CTRS ; PXDC2 : CTD bit 4, CTV 3:0, LTRME bit 10.
- AER (AERCAP) : AERID ; AERUCES (statut non corrigé, sticky) ; AERUCEM (masques) ; AERUCESEV (1 = fatal) ; AERCES ; AERCEM ; AERCC (FEP bits 4:0, ECE/ECC/EGE/EGC, MHRE/MHRC, TPLP) ; AERHL (header log, 16 octets) ; AERTLP (TLP prefix log, 16 octets, optionnel).

## Power Loss Signaling (optionnel)
- Variables PLN et PLA. PLN# asserté -> PLN = Asserted pour tous les contrôleurs du domaine.
- PLA# suit l'état le plus contraignant (FQ, EPF-Enabled, EPF-Disabled). Avec EPF-Disabled, les liens PCIe du domaine deviennent inactifs.

## 3.9 Commandes Admin spécifiques
- Get Log Page LID 19h : Physical Interface Receiver Eye Opening Measurement. Portée = port PCIe ; non supporté pour les contrôleurs secondaires. CSI = N ; restauré au défaut = Y.
  - Action (CDW10 bits 11:10) : 00 lire ; 01 démarrer la mesure et lire ; 10 abandonner et effacer (EOMIP = 0) ; 11 réservé.
  - MQUAL (bits 9:8) : 00 good, 01 better, 10 best.
  - Identifiant spécifique (CDW11) : TC = identifiant du contrôleur ciblé, bits 15:00.
  - Taille : HSIZE si EOMIP 0 ou 1 ; HSIZE + DS × ND si EOMIP = 2.
  - En-tête EOM (HSIZE = 64) : EOMIP (0 rien, 1 en cours, 2 terminé), EDGN (incrémenté à chaque démarrage), LREV = 3, ODP (EDFP, PEFP), LNS, EPL (1 NRZ, 3 PAM4), LSPFV, MLS, LSIC, DS (taille d'un descripteur), ND, MAXTB, MAXLR, ETGOOD / ETBETTER / ETBEST (temps estimés en secondes).
  - Descripteur de voie (DS octets) : MSTAT (bit 0 = mesure réussie), LN, EYE, TOP / BTM / LFT / RGT, NROWS, NCOLS, EDLEN, Printable Eye ('0' dans l'œil, '1' hors de l'œil), Eye Data (vendeur), padding à 4 octets.
  - Figure 77 : exemple 32 lignes × 22 colonnes.
  - La mesure tourne en arrière-plan ; l'hôte doit couper l'activité du port pendant ce temps.

## TDISP (informatif)
- DEVICE_INTERFACE_REPORT (TNI = 4E564D65h, "NVMe" ; VER = 1h) : IOCQESV, IOSQESV, MPSV, CSSV, STV, SHSTV, SHNV, ENV, ACPS, CSVI, VER utilisé, AQV, ASV, ACV, CMBMSC, PMRMSC, PMRCTL, QH (SHA-384 de la NVMe Controller State). Lecture autorisée en CONFIG_LOCKED ou RUN ; toute modification (hors QH) fait passer en ERROR.
- Concerne la confidentialité et l'attestation ; hors périmètre de l'agent actuel.

## Annexe A (informatif) — flux hôte [AGENT]
- Soumission : construire la SQE, écrire SQxTDBL.
- Traitement CQ : lire la CQE ; SQID (DW2) et CID (DW3) ; SF (DW3) = statut ; écrire CQyHDBL (efface l'interruption) ; en cas d'erreur, récupération.
- Interruptions INTMS/INTMC : dans l'ISR, masquer (INTMS) ; en DPC, traiter les CQE et écrire CQyHDBL ; démasquer (INTMC).

## À recouper avec le code de l'agent (non vérifié ici)
- Lecture de CAP.DSTRD pour le calcul des offsets de doorbell.
- Phase tag de la CQ, wrap des SQ/CQ, écriture de CQyHDBL après consommation.
- MSI-X : IV ≤ taille de la table ; masque pendant le traitement.
- Pas de passage en D3cold sans accord explicite.
