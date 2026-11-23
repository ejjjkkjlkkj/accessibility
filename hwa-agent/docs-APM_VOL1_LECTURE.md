# APM vol. 1 (AMD64 Architecture Programmer's Manual, Volume 1, rév. 3.23, oct. 2020)
Lecture ligne par ligne, par tranches de 250 lignes. Notes par tranche.

## Lignes 1-250 : page de titre, copyright, table des matières
- Vol. 1 « Application Programming », publication 24592, rév. 3.23, oct. 2020.
- Chapitres : 1 Vue d'ensemble (1.1 à 1.2 modes) ; 2 Modèle mémoire (2.1 à 2.5) ; 3 Programmation générale (3.1 registres, 3.2 opérandes, 3.3 résumé, 3.4 64 bits, 3.5 préfixes, 3.6 détection, 3.7 transferts de contrôle, 3.8 E/S, 3.9 mémoire, 3.10 performance) ; 4 SSE ; 5 MMX et 3DNow! ; 6 x87.
## Lignes 251-500 : TOC suite, listes de figures et tableaux, historique des révisions
- Liste des tableaux utiles au projet : Tableau 3-1 modes, 3-2 registres par mode, 3-4 préfixes d'adresse, 3-5 registres GPR implicites, 3-9 et suivants (tailles d'opérande, tableaux rFLAGS des CMOVcc/SETcc/Jcc, préfixes hérités), 2-1 Interruptions et exceptions.
- Historique : rév. 3.23 (oct. 2020) ajoute le shadow stack ; la rév. 3.22 intègre AVX2, SSE4.1/4.2, F16C, BMI, TBM, XOP, YMM.
- Les lignes 1-500 ne contiennent aucun élément de comportement à implémenter : sommaire, figures, tableaux, historique.
## Lignes 501-750 : préface, conventions de notation, glossaire
- Découpage de la série : vol. 1 application (24592), vol. 2 système (24593), vol. 3 instructions générales et système (24594), vol. 4 médias 128/256 bits (26568), vol. 5 médias 64 bits et x87 (26569).
- Conventions : CR0[PE] = champ PE de CR0 ; plages de bits écrites de haut en bas ; nombres sans suffixe = décimal.
- Glossaire utile au projet : mode long = mode 64 bits + compatibilité ; mode legacy = réel, protégé, virtuel-8086 ; IST = table de piles en mode long ; IRB/IVT = modes anciens ; GIF = indicateur global d'interruption (SVM) ; LIP = CS.base + rIP ; MBZ = doit être zéro (#GP si écrit à 1) ; RAZ/IGN/SBZ = règles de lecture/écriture des champs réservés.
- Règle générale à retenir pour l'agent : un champ réservé ne doit pas être supposé à une valeur ; ne pas en dépendre.
## Lignes 751-1000 : fin du glossaire, ordre des octets, bibliographie, 1.1 introduction, 1.1.1 fonctionnalités
- Ordre des octets : little-endian (octet de poids faible à l'adresse la plus basse). Règle à respecter dans tout l'agent (déjà appliquée dans les lectures de tables).
- 1.1.1 : 16 GPR de 64 bits (8 de plus qu'en x86 legacy) ; 16 registres XMM/YMM (8 de plus) ; accès uniforme aux registres octets via REX ; préfixe REX pour les registres étendus ; mode long : adresses virtuelles jusqu'à 64 bits, RIP 64 bits, adressage relatif à RIP, espace plat.
- Figure 1-1 : jeu de registres d'application (GPR, MMX/FPR, XMM/YMM, RFLAGS, RIP, MXCSR, x87).
- Tableau 1-1 : modes de fonctionnement (suite lue ensuite).
## Lignes 1001-1250 : tableau 1-1 (modes), 1.1.2 registres (tableau 1-2), 1.1.3 à 1.1.5 jeux d'instructions
- Tableau 1-1 : long mode (64 bits, OS 64 bits, compatibilité, 16/32 bits) ; mode legacy (protégé 32 bits, réel 16 bits, virtuel-8086). Compatibilité : extensions de registres non disponibles. Adresses par défaut : 64 bits en mode 64 bits, 32 bits en compatibilité/legacy 32, 16 bits en 16.
- Tableau 1-2 : mode 64 bits : GPR 64 bits (RAX..R15), YMM0-15 (256 bits), XMM0-15 (128 bits), MMX0-7 (64 bits), FPR0-7 (80 bits), RIP 64 bits, RFLAGS 64 bits. Legacy : 8 GPR 32 bits, YMM0-7, XMM0-7, EIP, EFLAGS.
- Les XMM recouvrent la moitié basse des YMM ; MMX0-7 sont mappés sur FPR0-7 (x87).
- 1.1.3 : quatre sous-ensembles : général, SSE (YMM/XMM), multimédia (MMX/3DNow!, peu recommandé, voir vol. 5), x87.
- 1.1.5 : SSE = IEEE-754 simple et double précision, exceptions dédiées ; MMX = simple précision sans exceptions ; x87 = simple, double, étendue 80 bits, transcendantes.
- Pertinence agent : les rapports CR/RFLAGS et les tailles de registres du rapport doivent suivre ce tableau (RIP 64 bits, RFLAGS 64 bits).
## Lignes 1251-1500 : 1.2 modes (long, 64 bits, compatibilité, legacy), 2.1 organisation mémoire (2.1.1 à 2.1.4)
- 1.2.1 : le mode long n'inclut ni real mode ni virtual-8086, et pas de commutation de tâche matérielle.
- 1.2.2 : mode 64 bits = adressage plat ; CS, FS, GS reconnus ; DS/ES/SS ont une base nulle et ne sont pas contrôlés ; RIP 64 bits ; adressage relatif à RIP ; REX ajoute R8-R15, élargit les GPR à 64 bits et ajoute XMM/YMM8-15 ; REX donne accès aux octets bas de tous les GPR.
- 1.2.3 : mode compatibilité : segmentation 16/32 bits identique au protégé legacy ; accès aux 4 premiers Go.
- 1.2.4 : legacy = protégé (4 Go), virtuel-8086 (1 Mo), réel (1 Mo). Les processeurs démarrent en réel legacy.
- 2.1.1 : mode 64 bits flat (bases à 0, sauf FS/GS) ; compatibilité = segmentation multi-segments.
- 2.1.2 : FS et GS sont les seuls segments à base active en 64 bits (voir §2.2, "FS and GS as Base of Address Calculation").
- 2.1.3 / 2.1.4 : adresses virtuelles jusqu'à 64 bits, physiques jusqu'à 52 bits (spécifique à chaque implémentation) ; en compatibilité, l'adresse effective 16/32 bits + sélecteur donne une adresse linéaire étendue à 64 bits.
- Renvois utiles : "Segmented Virtual Memory" et "System-Management Instructions" dans vol. 2.
## Lignes 1501-1750 : 2.1.4.2 gestion mémoire legacy, 2.2 adressage, ordre des octets, adresses canoniques, préfixe 67h, RIP-relatif, pointeurs
- 2.1.4.2 : protégé = 16K segments jusqu'à 4 Go, pagination optionnelle ; virtuel-8086 = adresse linéaire 20 bits, paginable ; réel = segment<<4 + offset sur 20 bits (1 Mo).
- 2.2.1 : little-endian, octet de poids faible à l'adresse la plus basse. Exemple : 48 B8 8877665544332211 = MOV RAX, 1122334455667788.
- 2.2.2 : adresses canoniques : bits 63 jusqu'au bit de poids fort implémenté tous égaux ; sinon #GP ou #SS. Point de contrôle pour la carte mémoire.
- 2.2.3 : cinq modes d'adressage effectif (absolu, relatif à IP, indexé à registre de base, pile, chaînes RSI/RDI). En 64 bits : base et index 64 bits, déplacements étendus au signe.
- 2.2.3.1 : en compatibilité, les adresses 16/32 bits sont étendues à zéro sur 64 bits : seuls les 4 premiers Go sont accessibles.
- 2.2.3.2 : déplacements et immédiats sur 32 bits maximum (étendus au signe en 64 bits) ; exceptions MOV à 64 bits.
- 2.2.3.3 : seuls FS et GS ont une base non nulle en 64 bits ; DS/ES/SS ont une base nulle.
- 2.2.4 / tableau 2-1 : préfixe 67h ; défaut 64 bits en 64 bits, surcharge en 32 bits seulement (pas de 16 bits en 64 bits).
- 2.2.5 : adressage RIP-relatif en 64 bits pour toute instruction à ModRM ; déplacement signé 32 bits (±2 Go). Le préfixe 67h ne le désactive pas.
## Lignes 1751-2000 : 2.3 pointeurs, 2.4 pile, 2.5 pointeur d'instruction, 3 programmation générale (3.1 registres)
- 2.3 : en 64 bits, un seul segment de données plat : pas de pointeurs lointains ; en compatibilité et legacy, pointeurs lointains (sélecteur + offset).
- 2.4 : pile : rBP/rSP ; SS pointe sur la base ; en legacy 16 ou 32 bits selon le bit B du descripteur de pile ; en 64 bits, RSP toujours 64 bits ; pile shadow (protection contre ROP) activable par le système (vol. 2 « Shadow Stacks »).
- 2.5 : rIP non lisible directement ; poussé par CALL. IP / EIP / RIP selon le mode.
- 3.1 (figure 3-1) : en 64 bits, DS, ES, SS ignorés ; registres étendus ; RFLAGS 64 bits, dont les 32 bits hauts sont RAZ (ignorés en écriture, lus à zéro).
- 3.1.1 : registres legacy : 8 octets (AH..DH, AL..DL : codages 4-7 = octets hauts), 16 bits, 32 bits. Codage 4-7 en octet = AH/CH/DH/BH sans REX.
- 3.1.2 : 64 bits : octets bas de tous les GPR (SIL, DIL, BPL, SPL, R8B..R15B) ; AH..DH inaccessibles avec un préfixe REX.
- Règle de taille (à respecter dans l'agent et les décodages) : octet/mot écrit ne modifie pas les bits hauts ; mot 32 bits écrit met à zéro les 32 bits hauts.
## Lignes 2001-2250 : 3.1.2 registres en mode 64 bits, 3.1.2.1 à 3.1.2.4, 3.1.3 usages implicites
- 3.1.2.1 : taille d'opérande par défaut 32 bits ; 66h pour 16 bits ; REX.W pour 64 bits.
- 3.1.2.3 : écriture 32 bits d'un GPR = extension à zéro sur 64 bits ; 8 et 16 bits préservent les bits hauts non écrits. Le logiciel doit étendre au signe les résultats 8/16/32 bits avant de s'en servir pour une adresse 64 bits.
- 3.1.2.4 : quitter le mode 64 bits vers compatibilité/legacy : les 32 bits hauts des GPR deviennent indéfinis ; ne jamais en dépendre.
- 3.1.3 / tableau 3-1 : usages implicites : RAX (multiplication, division, E/S, CMPXCHG, décimal), RBX (XLAT), RCX (compteur de boucles et de chaînes, décalages), RDX (E/S, multiplication 64 bits, port), RSI/RDI (adresses de chaînes), RSP/RBP (pile et cadre) ; CPUID utilise EAX/ECX et écrit EAX, EBX, ECX, EDX.
- Pertinence agent : les sondes CPUID et XGETBV (et leur sous-feuille dans ECX) suivent ce tableau.
## Lignes 2251-2500 : 3.1.3 usages implicites (suite), 3.1.4 registre rFLAGS, 3.2.1 à 3.2.2 types de données
- rFLAGS (figure 3-5) : CF bit 0, PF bit 2, AF bit 4, ZF bit 6, SF bit 7, DF bit 10, OF bit 11 ; bits 31:16 réservés au système (vol. 2) ; bits 63:32 RAZ, écriture ignorée.
- Réinitialisation : rFLAGS = 0x00000002 (bit 1 réservé à 1). Utile pour interpréter les valeurs RFLAGS des rapports.
- OF indéfini après DIV et après un décalage de plus d'un bit. Les opérations logiques remettent CF et OF à zéro. INC/DEC n'affectent pas CF.
- 3.1.3.12 : PUSHA/POPA utilisent tous les GPR ; ENTER/LEAVE utilisent RBP ; base RSP ou RBP = segment SS par défaut ; autre base = DS.
- 3.1.3.13 : CPUID lit EAX (et ECX en sous-fonction), écrit EAX, EBX, ECX, EDX.
- Types de données : entiers signés/non signés 8/16/32/64/128 bits ; BCD ; flottants 16, 32, 64 bits (vol. 1 §4.3.3 pour SSE).
## Lignes 2501-2750 : 3.2.2 types (entiers, BCD, chaînes), 3.2.3 tailles d'opérande, 3.2.4 adressage des opérandes
- Plages (tableau 3-2) : octet -128..127 / 0..255 ; mot ; double ; quadruple ; octo-mot 128 bits (RDX:RAX pour MUL/IMUL/DIV/IDIV/CQO en 64 bits).
- BCD : chiffre 0-9 sur 4 bits ; packed (2 par octet), unpacked (1 par octet). Ajustements AAA/AAS/AAD/AAM/DAA/DAS.
- Chaînes : longueur maximale 2^32-1 (legacy/compat) ou 2^64-1 (64 bits).
- Tableau 3-3 : préfixe 66h = 16 bits ; REX.W = 64 bits (REX prime sur 66h). En 64 bits : taille par défaut 32 bits, sauf branchements proches, pile (PUSH/POP), LOOP/Jcc : 64 bits par défaut.
- 3.2.3.3 : immédiats 8/16/32 bits ; en 64 bits, un MOV reg,imm64 (opcodes B8h-BFh avec REX.W) donne un immédiat 64 bits réel ; sinon l'immédiat est étendu au signe à 64 bits.
- 3.2.4 : opérandes en registre (REX pour étendre), mémoire (§2.2), ports E/S (§3.8).
## Lignes 2751-3000 : 3.2.4.4 immédiats, 3.2.5 alignement, 3.3 résumé (transferts, CMOVcc, pile, conversions, BSWAP)
- 3.2.5 : alignement d'un accès = adresse multiple de sa taille ; pas d'obligation, mais coût en performance. Règles de performance listées.
- 3.3.2 : MOV, MOVBE (octets inversés), MOVSX/MOVZX, MOVD, MOVNTI (non temporel). Source et destination ne peuvent pas être deux mémoires.
- Tableau 3-4 : conditions CMOVcc (CF, ZF, SF, OF, PF) : toutes les paires cc/alias ; CMOVL/CMOVGE/CMOVLE/CMOVG utilisent SF<>OF.
- Pile : PUSH décrémente de 2/4/8 et écrit à SS:rSP ; POP inverse. PUSHA/PUSHAD sauvegardent eAX..eDI dans cet ordre (eSP capturé au début) ; POPA/POPAD l'inverse.
- ENTER (size, depth 0-31) crée le cadre ; LEAVE équivaut à mov rsp,rbp ; pop rbp.
- 3.3.3 : CBW/CWDE/CDQE (extension vers rAX), CWD/CDQ/CQO (extension vers rDX:rAX) ; ces instructions n'affectent pas les drapeaux.
- MOVMSKPS/PD : masque de signe vers GPR (extension à zéro).
- XLAT : déconseillé, équivalent à MOV AL,[rBX+AL].
- AAA/AAD/AAM/AAS : corrections BCD non empaquetées ; DAA/DAS : empaquetées.
- BSWAP : inverse l'ordre des octets d'un 32 ou 64 bits (figure 3-8).
## Lignes 3001-3250 : chargement de segments, LEA, arithmétique, rotations/décalages, bits, comparaisons, SETcc, BOUND, logique, chaînes (début)
- 3.3.4 : LDS/LES/LFS/LGS/LSS chargent sélecteur + offset ; MOV segReg et POP segReg chargent aussi le descripteur caché. En 64 bits : POP DS/ES/SS invalides.
- 3.3.5 : LEA calcule une adresse effective sans lire la mémoire (utile pour l'arithmétique).
- 3.3.6 : ADD/ADC/SUB/SBB/NEG ; MUL/IMUL (formes 1, 2, 3 opérandes) ; DIV/IDIV (quotient en rAX, reste en rDX) ; INC/DEC n'affectent pas CF.
- 3.3.7 : rotations ROL/ROR/RCL/RCR et décalages SHL/SHR/SAR/SAL/SHLD/SHRD ; compte masqué à 5 bits (32 bits) ou 6 bits (64 bits). OF indéfini pour les rotations multi-bits. SAR n'est pas identique à IDIV pour les négatifs.
- 3.3.8 : BMI/TBM (BLSI, BLSR, BLSMSK, BLCS, BLCI, TZMSK, T1MSKC, BEXTR, LZCNT/TZCNT/POPCNT). Les formes TBM ne sont pas présentes sur tous les processeurs (voir CPUID).
- BSF/BSR : ZF = 1 si un bit trouvé ; sinon ZF = 0 et destination indéfinie.
- 3.3.9 : CMP, TEST, BT/BTC/BTR/BTS (CF = bit testé) ; SETcc selon tableau 3-5 ; BOUND lève une exception de limite.
- 3.3.10 : AND/OR/XOR remettent CF et OF à zéro ; NOT ne modifie pas les drapeaux ; ANDN = ET(NOT src1, src2).
- 3.3.11 : chaînes CMPS, SCAS, MOVS, LODS, STOS... : rSI/rDI avancent selon DF ; REP utilise rCX.
## Lignes 3251-3500 : fin des chaînes, transferts de contrôle (JMP, Jcc, LOOPcc, CALL, RET), interruptions, drapeaux, E/S, sémaphores, CPUID, cache, NOP, appels système, 3.4 début
- Table 3-6 : Jcc (JO..JG). JCXZ/JECXZ/JRCXZ testent rCX. LOOPE/LOOPNE/LOOP décrémentent rCX sans toucher aux drapeaux.
- CALL : même formes que JMP sans la forme courte. CALL/JMP lointains invalides en 64 bits. RET : option imm16 pour dépiler des paramètres.
- INT n, INTO (#OF si OF=1), IRET/IRETD/IRETQ : l'IRET en 64 bits dépile SS:rSP.
- 3.3.13 : POPF/PUSHF 16 bits ; POPFD/PUSHFD 32 ; POPFQ/PUSHFQ 64 ; seuls les bits de la figure 3-5 changent. CLI/STI : IF (système). LAHF/SAHF : octet bas (SF, ZF, AF, PF, CF).
- 3.3.14 : IN/OUT : ports 0-255 par immédiat, 0-65535 par DX ; INS/OUTS avec REP.
- 3.3.15 : CMPXCHG, CMPXCHG8B, CMPXCHG16B (ZF), XADD, XCHG (verrouillage implicite en mémoire). LOCK rend atomiques CMPXCHG/XADD en mémoire.
- 3.3.16 : CPUID ; présence testée par RFLAGS.ID (si non inscriptible, CPUID = #UD).
- 3.3.17 : LFENCE/SFENCE/MFENCE, PREFETCH*, CLFLUSH, CLWB, WBNOINVD (vol. 2 pour les privilégiés).
- 3.3.19 : SYSCALL/SYSRET valides en 64 bits ; SYSENTER/SYSEXIT invalides en 64 bits.
- 3.3.20 : CRC32 et POPCNT (accélérateurs).
## Lignes 3501-3750 : 3.4 règles 64 bits, 3.5 préfixes (tableau 3-7), 3.6 détection de fonctionnalités
- 3.4.4 : par défaut 32 bits ; 64 bits par REX.W ; branchements et instructions à RSP : 64 bits par défaut ; CPUID toujours 32 bits.
- 3.4.6 : invalides en 64 bits : AAA, AAD, AAM, AAS, BOUND, CALL/JMP lointains absolus, DAA, DAS, INTO, POP DS/ES/SS, POPA/POPAD, PUSH CS/DS/ES/SS, PUSHA/PUSHAD. Invalides en long mode : SYSENTER/SYSEXIT. Réassignés : ARPL (devient MOVSXD), INC/DEC (un octet, deviennent des préfixes REX), LDS/LES.
- 3.5.1 (tableau 3-7) : préfixes legacy : 66h (taille opérande), 67h (taille adresse), segments 2Eh/3Eh/26h/64h/65h/36h, LOCK F0h, REP F3h, REPE/REPZ F3h, REPNE/REPNZ F2h. Un seul préfixe par groupe ; les préfixes répétés ont un effet indéfini.
- 3.5.1.3 : LOCK ne vaut que pour ADC, ADD, AND, BTC, BTR, BTS, CMPXCHG, CMPXCHG8B, DEC, INC, NEG, NOT, OR, SBB, SUB, XADD, XCHG, XOR (formes mémoire) ; autre instruction = #UD.
- 3.5.1.2 : les préfixes de segment sont ignorés pour les accès pile (SS toujours) et pour la destination de chaîne (ES toujours). CS ne peut pas être surchargé pour la récupération d'instructions.
- 3.5.2 : REX = 40h à 4Fh, uniquement en mode 64 bits ; doit précéder immédiatement l'opcode ou l'échappement.
- 3.5.3 : VEX = C4h/C5h ; XOP = **8Eh** selon ce texte. Attention : le projet (vex.js, xop.js) utilise 8Fh. À vérifier dans APM vol. 3 (la table d'encodage fait foi).
- 3.6 : CPUID : présence vérifiée par RFLAGS.ID. Logiciel invité : vérifier le bit CPUID avant chaque fonctionnalité, une fois par initialisation. Appendice D (vol. 3) : bits de fonctionnalités ; appendice E : toutes les informations CPUID.
## Lignes 3751-4000 : 3.7 transferts de contrôle (privilèges, pile de procédure, sauts, appels, retours, shadow stack)
- 3.7.2 : niveaux de privilège 0 (système) à 3 (applications) ; le matériel contrôle les transferts entre niveaux.
- 3.7.3.1 : alignement de pile : mot en 16 bits, double-mot en 32, quadruple-mot (8 octets) en 64 bits.
- 3.7.3.2 / tableau 3-8 : pile 64 bits en mode 64 bits ; PUSH/POP/CALL/RET/ENTER/LEAVE/PUSHF/POPF défaut 64 bits ; pas de push/pop 32 bits en 64 bits.
- 3.7.4 : sauts courts/proches/lointains ; Jcc uniquement proches ; un saut ne change pas de niveau de privilège.
- 3.7.5 : CALL proche (pousse rIP), lointain même niveau (CS:rIP), lointain vers niveau supérieur (passage de pile via TSS et porte d'appel ; pas de copie de paramètres en long mode), changement de tâche (non supporté en long mode).
- Shadow stack (si activée) : CALL pousse aussi la LIP sur la pile shadow ; RET compare et lève #CP en cas de différence ; un CALL vers un niveau plus privilégié change de SSP (MSR PLn_SSP).
- 3.7.6 : RET proche/lointain ; RET imm16 libère des octets de paramètres sans les lire.
## Lignes 4001-4250 : retours de privilège, SYSCALL/SYSRET, branchements 64 bits, interruptions et exceptions (tableau 3-10)
- Retour lointain vers moins privilégié : dépile CS:rIP puis SS:rSP ; shadow stack selon le CPL cible (CPL 3 : PL3_SSP, pas de contrôle de l'adresse).
- SYSCALL/SYSRET : valides en protégé et en long mode ; SYSRET seulement depuis CPL 0. SYSENTER/SYSEXIT : mode legacy uniquement.
- 3.7.9 : tous les branchements proches ont 64 bits par défaut ; le déplacement reste sur 32 bits ; les branchements lointains directs sont invalides en 64 bits. CALL/RET/JMP/Jcc/LOOP/JRCXZ listés au tableau 3-9.
- 3.7.10 : IDT : 4 octets en réel, 8 octets en protégé, 16 octets en long mode (portes d'interruption, de tâche, de trappe ; pas de porte d'appel).
- Vecteurs (tableau 3-10) : 0 #DE, 1 #DB, 2 NMI, 3 #BP, 4 #OF, 5 #BR, 6 #UD, 7 #NM, 8 #DF, 10 #TS, 11 #NP, 12 #SS, 13 #GP, 14 #PF, 16 #MF, 17 #AC, 18 #MC, 19 #XF, 21 #CP ; 9, 15, 20, 22-31 réservés. Ces vecteurs sont ceux que l'IDT peut recevoir.
- Types : fautes (retour avant l'instruction), trappes (après), abandons (imprécis).
- Pour l'agent : toute sonde qui fait une lecture risquée doit tenir compte des fautes #GP, #UD, #PF ; une lecture de MSR absent lève #GP, un MSR AMD non documenté peut donc être absent.
## Lignes 4251-4500 : interruptions (même/plus haut privilège, IRET), E/S (IOPL, bitmap TSS), 3.9 optimisation mémoire (ordre, barrières, atomicité, caches, MOESI)
- Interruption vers même niveau en legacy : pousse RFLAGS, CS:rIP, code d'erreur éventuel. Vers plus haut niveau ou en long mode : pile via TSS, SS:rSP sauvé.
- IRET : dépile RIP, CS, RFLAGS ; SS:rSP si changement de niveau ou en 64 bits. Avant IRET, retirer le code d'erreur.
- E/S : IN/OUT et leurs variantes chaînes sont sensibles à IOPL : #GP si CPL > IOPL (sauf bitmap TSS). Seul CPL 0 change IOPL (POPF, IRET).
- Accès MMIO : mémoire, ordonnancement faible. Barrières : LFENCE, SFENCE, MFENCE ; sérialisants : CPUID, IRET, MFENCE ; OUT avant IN ne se réordonnent pas.
- Atomicité : accès naturellement atomiques jusqu'à 8 octets sans franchir une frontière 8 octets ; CMPXCHG16B pour 16 octets.
- Caches : lignes de 32 ou 64 octets ; protocole MOESI ; écritures dans le cache d'instructions invalident la ligne. Pertinent pour toute écriture de code (agent : aucune).
## Lignes 4501-4750 : fin du chapitre 3 (cache, préchargement PREFETCH*, stores non temporels, CLFLUSH, performance) ; début du chapitre 4 (SSE, registres XMM/YMM)
- 3.9.6 : PREFETCHT0/T1/T2/NTA (SSE) ; PREFETCH et PREFETCHW (3DNow!). Un prefetch d'adresse invalide ne lève pas d'exception.
- Stores non temporels : MOVNTI (GPR), MOVNTDQ/PD/PS, MASKMOVDQU (XMM), MOVNTQ, MASKMOVQ (MMX). CLFLUSH écrit et invalide la ligne.
- 3.10 : conseils de performance (opérandes larges, formes courtes, alignement, CMOVcc pour éviter les branches, préchargement, registres, dépendances store-load).
- Chapitre 4 : SSE = SSE1, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, SSE4A (legacy) + AES, AVX, AVX2, CLMUL, FMA4, FMA, XOP (étendu). Legacy : 2 opérandes ; AVX/AVX2 : 3 ; FMA4/XOP : 4.
- Registres : XMM0-15 (128 bits) recouvrent les 128 bits bas de YMM0-15 (256 bits). MXCSR séparé de x87, avec ses propres exceptions.
- Compatibilité : binaires SSE legacy s'exécutent sans modification en legacy et en compatibilité.
## Lignes 4751-5000 : registres SSE (4.2.1), MXCSR (4.2.2), opérandes (4.3), alignement (4.3.2), types entiers (4.3.3.1)
- XMM/YMM : reset à zéro ; #INIT ne les modifie pas. Instructions SSE legacy : bits 255:128 de YMM inchangés ; formes étendues (VEX) : bits hauts mis à zéro.
- MXCSR (figure 4-2) : IE 0, DE 1, ZE 2, OE 3, UE 4, PE 5 (drapeaux collants) ; DAZ 6 ; masques PM 12, UM 11, OM 10, ZM 9, DM 8, IM 7 ; RC bits 14:13 (00 nearest, 01 bas, 10 haut, 11 zéro) ; FZ 15 ; MM 17 (misaligned). Valeur de réinitialisation : 0x1F80.
- Alignement : SSE legacy : accès vecteur non aligné = #GP sauf MXCSR.MM (si CPUID 8000_0001 ECX[MisAlignSse]). FXSAVE/FXRSTOR, MOVAPS/PD, MOVDQA, MOVNTx : toujours alignés.
- Entiers 4.3.3.1 : signés en complément à deux ; octet 8 bits ; mot 16 ; double 32 ; quadruple 64 ; octo 128 bits.
- Note pour le rapport : la valeur MXCSR de réinitialisation (0x1F80) est une constante utile pour interpréter un MXCSR relu.
## Lignes 5001-5250 : saturation, types flottants simple/double précision, nombres normaux/dénormaux/zéro/infini/NaN, encodages
- Saturation (4.3.3.1) : PACKx, PADDSx/PADDUSx, PSUBSx/PSUBUSx saturent ; les autres ne saturent pas.
- Flottants (4.3.3.3, figure 4-4) : simple = signe 1, exposant 8 bits (biais 127), fraction 23 ; double = signe 1, exposant 11 bits (biais 1023), fraction 52 ; bit entier implicite.
- Plages : simple normal 2^-126 à 2^127·(2-2^-23) ; double 2^-1022 à 2^1023·(2-2^-52).
- Dénormaux : exposant 0, fraction non nulle (DE à la lecture). Infini : exposant max, fraction 0. NaN : exposant max, fraction non nulle ; QNaN = bit de fraction de poids fort à 1, SNaN = à 0. Le processeur ne produit jamais un SNaN.
- Tableau 4-5 : règles de résultat NaN (QNaN gagne, opérande 1 prioritaire si deux NaN de même type) ; indéfini = NaN spécial.
- Compatibilité x87 : x87 calcule en double-étendu par défaut, donc ses bits de poids faible peuvent différer des résultats SSE.
## Lignes 5251-5500 : encodages (suite), valeurs indéfinies (tableau 4-7), arrondi (tableau 4-8), flottant demi-précision (16 bits)
- Indéfini : flottant simple FFC00000h, double FFF8000000000000h ; entiers 16/32/64 bits 8000h / 80000000h / 8000000000000000h. Renvoyé quand IE est masquée.
- Arrondi RC (MXCSR 14:13) : 00 au plus proche (défaut, ties-to-even) ; 01 vers -∞ ; 10 vers +∞ ; 11 vers zéro. Pas d'arrondi pour les résultats NaN ; l'arrondi lève PE.
- Demi-précision : 1 bit de signe, exposant 5 bits (biais 15), fraction 10 bits ; convertie par VCVTPH2PS/VCVTPS2PH (section 4.7.2.5).
## Lignes 5501-5750 : fin des encodages demi-précision (tableau 4-9), types vectoriels et scalaires (4.3.3.4), figures des types 128 bits
- Demi-précision : infini (exposant 11111, fraction 0) ; NaN : exposant 11111 et fraction non nulle (SNaN si bit de poids fort 0).
- 4.3.3.4 : types vectoriels simple/double précision, entiers signés/non signés (quadruple, double, mot, octet), scalaires flottants et entiers. Le matériel ne vérifie pas le type : c'est au logiciel.
- Numérotation des éléments : de droite à gauche, à partir de 0. Paires paires (0,2,4) / impaires (1,3,5).
## Lignes 5751-6000 : fin des types 128 bits (figure 4-6), types 256 bits (figures 4-7 et 4-8)
- Types 128 bits : scalaires et vecteurs (flottants, entiers signés et non signés) ; figure 4-6.
- Types 256 bits : vecteurs flottants (8 simple précision ou 4 double précision) et entiers (octo-mot, quadruple, double, mot, octet) ; figures 4-7 et 4-8 (disposition de bits, pas de règle nouvelle).
## Lignes 6001-6250 : fin des types 256 bits (figure 4-8), 4.3.4 tailles d'opérande SSE, 4.4 opérations vectorielles
- 4.3.4 : les préfixes 66h/F2h/F3h des instructions SSE modifient l'opcode ; la règle générale de tailles de vol. 1 §3.2.3 ne s'applique pas. Détail dans vol. 4.
- 4.4.1 : opérations vectorielles : même opération sur tous les éléments en parallèle, sans propagation de retenue entre éléments. Largeur du résultat = largeur des opérandes.
- Forme legacy : le résultat écrase l'opérande 1 ; formes étendues (VEX/XOP) : destination distincte, sans écraser les sources.
## Lignes 6251-6500 : 4.4.2 opérations flottantes vectorielles ; 4.5 syntaxe SSE legacy et étendue ; conventions de noms ; 4.5.3 déplacements ; 4.5.4 conversions
- 4.5.1 : syntaxe legacy MNEMONIC xmm1, xmm2/mem128 (destination = source 1). Syntaxe étendue : destination distincte (VEX/XOP), 3 à 5 opérandes.
- 4.5.2 : préfixe V = instruction étendue ; P = « packed » ; suffixes A (aligné), U (non aligné), S (signé/saturation), US (saturation non signée), HL/LH, PD/PS/SD/SS.
- 4.5.3 : déplacements XMM/GPR/MMX ; un déplacement vers la mémoire n'étend jamais à zéro ; les transferts XMM<->GPR copient jusqu'à 8 octets.
- MASKMOVDQU : écrit les octets selon un masque, en écriture combinée, à DS:rDI (aligné avec les fins de copie en bloc).
- 4.5.4 : conversions entre entiers, flottants, GPR, MMX et mémoire ; UNPCK/PACK/SHUFFLE réordonnent les éléments.
## Lignes 6501-6750 : pack, décalages, shuffle, PINSR, opérations matricielles (PMADDWD, PSADBW, moyenne), saturation, suppression de branches (4.5.6)
- Pack (PACKSSDW) réduit la précision ; PSHUFD mélange par un octet de contrôle ; PINSRx insère un élément depuis GPR/mémoire (legacy : autres éléments inchangés).
- PMADDWD : produits 16 bits, sommes par paires en 32 bits. PSADBW : somme des différences absolues (compression vidéo).
- Moyenne d'octets/mots non signés (utile en décodage MPEG).
- 4.5.6 : suppression de branches : comparaison vectorielle -> masque, puis ET / ANDN / OU pour sélectionner ; MOVMSK* copie les bits de signe vers un GPR (pour un branchement dépendant des données).
## Lignes 6751-7000 : 4.6 résumé des instructions entières SSE, 4.6.1 transferts (MOVD/MOVQ/MOVDQA/MOVDQU/LDDQU, MOVDQ2Q, MOVQ2DQ), 4.6.1.2 déplacements non temporels
- Forme AVX (VEX) : efface les bits 255:128 de YMM ; forme legacy : les conserve.
- MOVD 32/64 bits depuis GPR/mémoire : extension à zéro sur 128 bits ; MOVQ = MOVD 64 bits ; MOVDQA exige un alignement ; MOVDQU non.
- MOVNTDQA (lecture non temporelle) : exige un alignement, sinon #GP ou #AC selon MXCSR.MM ; VMOVNTDQA : toujours #GP si non aligné.
- MASKMOVDQU : écrit les octets sélectionnés à l'adresse DS:rDI ; l'adresse peut ne pas être alignée.
- Remarque : un MOVx pour mettre un registre à zéro est moins efficace que PXOR (identique à la note de vol. 1 §3.3.2).
## Lignes 7001-7250 : PMOVMSKB, conditionnels vectoriels XOP (VPCMOV, VPPERM), conversions (4.6.2), réordonnancement (4.6.3)
- PMOVMSKB : bit de signe de chaque octet -> GPR, extension à zéro (32 bits, 64 bits en AVX2 pour 256 bits).
- 4.6.2 : CVTDQ2PS/PD, CVTPI2PS/PD (MMX -> XMM), CVTSI2SS/SD (GPR -> XMM), PMOVSX/PMOVZX (extension d'entiers). Arrondi si inexact.
- 4.6.3.1 : PACKSSDW, PACKUSDW, PACKSSWB, PACKUSWB : réduction de précision avec saturation ; la forme 256 bits agit sur chaque moitié séparément.
- 4.6.3.2 : PBLENDVB (masque = bits de signe), PBLENDW ; mise à zéro d'un bit de masque = élément de la première source.
- 4.6.3.3 : PUNPCKH/PUNPCKL (entrelacement des octets, mots, doubles, quadruples).
- 4.6.3.4 : EXTRQ/INSERTQ (SSE4A), PEXTRx/PINSRx : insertion/extraction selon un octet immédiat.
## Lignes 7251-7500 : PSHUF*, 4.6.4 arithmétique (valeur absolue, addition, soustraction, multiplication)
- PSHUFB : octets de contrôle (bit de poids fort = mise à zéro) ; PSHUFD : un octet immédiat pour 4 doubles ; PSHUFHW/PSHUFLW : mots haut/bas.
- 4.6.4.1 : PABSB/W/D : valeur absolue signée (résultat non signé).
- 4.6.4.2 : PADDx : débordement ignoré (on garde le bas) ; PADDSx/PADDUSx : saturation.
- 4.6.4.3 : PSUBx ; PSUBSx / PSUBUSx : saturation ; même logique que l'addition.
- 4.6.4.4 : PMULHW (haut 16 bits du produit 32 bits), PMULHRSW (arrondi et échelle), PMULLW (bas), PMULHUW (non signé), PMULUDQ (64 bits sur les doubles bas), PMULLD, PMULDQ.
## Lignes 7501-7750 : PMULUDQ/PMULLD/PMULDQ, PMADDWD, 4.6.5 instructions améliorées (multiplication-accumulation XOP : VPMACS*, VPMADCS*)
- PMULDQ : produit 64 bits signé de la double basse de chaque quadruple. PMULLD : bas 32 bits du produit signé 32 bits.
- PMADDWD : somme des produits adjacents : dst[j] = src1[2j]*src2[2j] + src1[2j+1]*src2[2j+1] (signés 16 bits).
- 4.6.5.1 : instructions XOP de multiplication-accumulation (VPMACSSWW, VPMACSWW, VPMACSSWD, VPMACSWD, VPMACSSDD, VPMACSDD, VPMACSSDQL/H, VPMACSDQL/H) : produits signés puis addition à l'accumulateur ; saturation selon l'instruction.
## Lignes 7751-8000 : 4.6.5.2 à 4.6.7.1 (horizontal, moyenne, SAD, décalages, rotations, comparaisons)
- 4.6.5.2 : PHADD/PHSUB (XOP) : sommes d'éléments adjacents. 4.6.5.3 : PAVGB/PAVGW : moyenne arrondie (a + b + 1) >> 1. 4.6.5.4 : PSADBW (deux sommes 16 bits). 4.6.5.5 : MPSADBW (SAD 4 octets, 8 résultats).
- 4.6.6.1 à 4.6.6.3 : PSLL/PSRL (logique : zéro entrant) ; PSRA (arithmétique : bit de signe entrant) ; PSLLDQ/PSRLDQ : décalage par octets. Décalage par comptes > largeur : résultat zéro (logique) ou signe (arithmétique).
- 4.6.6.4/5 (XOP) : VPSHL/VPSHA (compte par élément, négatif = sens inverse) ; VPROT (rotation).
- 4.6.7.1 : PCMPEQx / PCMPGTx produisent un masque de 1 ou 0 par élément (signé pour GT). Les opérandes identiques donnent tout à 1.
- Exemple de multiplexeur sans branchement (figure 4-21) : PCMPGTW + PAND + PANDN + POR.
## Lignes 8001-8250 : min/max, comparaisons XOP (VPCOM), chaînes SSE4.2 (PCMPxSTRx), logique (PAND/PANDN/PTEST/POR/PXOR), sauvegarde (FXSAVE, XSAVE, STMXCSR), 4.7 début (transferts flottants)
- Table 4-10 : prédicats VPCOM (LT, LE, GT, GE, EQ, NE, FALSE, TRUE) dans les 3 bits de poids faible de l'immédiat.
- PCMPESTRI/M, PCMPISTRI/M : comparaisons de chaînes explicites (EAX/EDX) ou terminées par NUL ; résultat dans ECX/RCX (index) ou XMM0 (masque) ; pas de contrôle d'alignement.
- PTEST : ZF si (src & mask) == 0 ; CF si (src & ~mask) == 0. Utile pour brancher sur des signes ou des champs à zéro.
- FXSAVE/FXRSTOR : état complet 512 octets (x87, MMX, SSE) ; XSAVE/XRSTOR : extensible (YMM etc.). XSAVEOPT, XSAVEC, XSAVES, XRSTORS : variantes (vol. 4).
- STMXCSR / LDMXCSR : MXCSR sur 32 bits.
- 4.7.1 : MOVAPS/PD exigent l'alignement (#GP sinon, ou #AC selon MXCSR.MM) ; MOVUPS/PD non.
## Lignes 8251-8500 : 4.7.1 transferts flottants (MOVLPS/HPS/SS/SD, duplication MOVDDUP/MOVSLDUP/MOVSHDUP), déplacements non temporels, MOVMSKPS/PD ; 4.7.2 conversions (4.7.2.1 à 4.7.2.5) ; 4.7.3 début
- MOVSS/MOVSD depuis mémoire : les bits hauts du registre sont mis à zéro ; depuis registre : ils sont inchangés (legacy).
- Conversions : CVTPS2PD/CVTPD2PS, CVTSS2SD/CVTSD2SS (arrondi si inexact) ; CVTPS2DQ (arrondi) vs CVTTPS2DQ (troncature) ; mêmes règles pour PD ; CVTxPS2PI / CVTxPD2PI vers MMX (initialiser l'état MMX avant, conflit avec x87) ; CVTSS2SI/CVTSD2SI/T... vers GPR 32 ou 64 bits.
- F16C : VCVTPH2PS et VCVTPS2PH (VEX C4h, 128 et 256 bits) : conversion demi-précision.
- 4.7.3.1 : EXTRACTPS, INSERTPS (ZMASK pour remettre à +0.0) ; 4.7.3.2 : BLENDPS/PD, BLENDVPS/PD (masque = bits de signe ; XMM0 implicite en forme legacy).
- 4.7.3.3 : UNPCKHPS/PD, UNPCKLPS/PD ; 4.7.3.4 : SHUFPS/PD.
## Lignes 8501-8750 : fin de SHUFPS/PD, 4.7.3.5 extraction de fraction (XOP), 4.7.4 arithmétique flottante (addition, horizontale, soustraction, multiplication, division, racine)
- VFRCZ* (XOP) : fraction des flottants. VSHUFPS/PD : formes 256 bits = même opération sur l'octet haut.
- ADDPS/PD/ADDSS/ADDSD ; HADDPS/PD ; SUBPS/PD/SUBSS/SUBSD ; HSUBPS/PD ; ADDSUBPS/PD (alternance) ; MULPS/PD/SS/SD ; DIVPS/PD/SS/SD ; SQRTPS/PD/SS/SD.
- Formes scalaires legacy : les bits hauts du destinataire sont inchangés ; formes VEX copient les bits hauts de la première source.
- PHMINPOSUW : minimum non signé et son index.
- Note : diviser par une constante = multiplier par l'inverse, si la précision le permet (puissances de 2 exactes).
## Lignes 8751-9000 : racine, inverse de racine et inverse estimés (erreur relative <= 1.5*2^-12), produit scalaire, arrondi (ROUND*), FMA (4.7.5 début)
- RSQRT*, RCP* : estimations avec erreur relative max 1.5 x 2^-12 : pas de précision complète.
- DPPS/DPPD : produit scalaire sélectif ; immédiat choisit les produits et les éléments destinataires.
- ROUNDPS/PD/SS/SD : mode de l'immédiat (au plus proche, vers -inf, +inf, troncature) ou mode courant de MXCSR.
- 4.7.5 : FMA4 (4 opérandes, destination distincte) et FMA (3 opérandes, écrasement de la première source) : a*b+c avec un seul arrondi. Variantes : FMADD, FMSUB, FNMADD, FNMSUB, FMADDSUB, FMSUBADD.
- Forme FMA3 : VFMADD132/213/231 ; la forme détermine quelle source reste en place (table 4-48).
## Lignes 9001-9250 : FMA suite (alternance, soustraction, négatives), 4.7.6 comparaisons (CMPPS/PD/SS/SD, MIN/MAX, COMIS/UCOMIS), 4.7.7 logique flottante, 4.8 préfixes SSE, 4.9 détection
- COMISS/COMISD (ordonné) : IE si un opérande est NaN (QNaN ou SNaN) ; UCOMISS/UCOMISD (non ordonné) : IE seulement si SNaN. Drapeaux : ZF, PF, CF ; OF, AF, SF = 0.
- CMPPS/PD : prédicat dans l'immédiat ; masque 32 bits (simple) ou 64 bits (double) par élément.
- MIN/MAX : comparaison d'un élément, le résultat est le min ou max (utile au serrage de valeurs).
- 4.8.1 : préfixes utilisables avec SSE : 67h (mémoire seulement), 66h, F2h, F3h (opcodes, pas de répétition), REX (registres étendus, FXSAVE). F0h (LOCK) => #UD.
- 4.9 : une fonction SSE n'est utilisable que si le bit CPUID du sous-ensemble est présent, que le système active le contexte et que le sous-ensemble est validé par l'OS (sinon #UD).
## Lignes 9251-9500 : exceptions SIMD (4.10) : #GP, #UD, #NM, #MF, #XF ; exceptions flottantes IE, DE, ZE, OE, UE, PE (4.10.2) ; priorité (4.10.3)
- 4.10.1 : #UD si CPUID absent, si CR4.OSXMMEXCPT = 0 avec une exception flottante non masquée, ou si le sous-ensemble n'est pas activé ; #NM si CR0.TS=1 ; #NM sur FXSAVE si CR0.EM=1.
- 4.10.1.2 : SSE ne lève pas #DE, #OF, #BR, #TS, #NP, #BP, NMI.
- 4.10.2.1 : une exception flottante non masquée lève #XF (vecteur 19) si CR4.OSXMMEXCPT=1, sinon #UD (vecteur 6).
- Tableau 4-12 (bits MXCSR) : IE 0, DE 1, ZE 2, OE 3, UE 4, PE 5 ; tableau 4-14 : priorités IE (SNaN) > QNaN > IE > ZE > DE (pré-calcul) > OE, UE > PE (post-calcul).
- Relation avec x87 : mécanismes d'exception distincts ; prévoir des routines séparées si les deux sont utilisés.
## Lignes 9501-9750 : 4.10.4 masques MXCSR (IM 7, DM 8, ZM 9, OM 10, UM 11, PM 12 ; reset = masqués), réponses masquées (tableau 4-16)
- Réponses masquées : opérations invalides -> QNaN (ou indéfini) ; MIN/MAX -> deuxième source ; CMP non ordonnée -> masque tout à 1 ; COMIS non ordonné -> ZF,PF,CF = 1, OF/SF/AF = 0.
- Débordement : la réponse dépend du mode d'arrondi (+inf, -inf, zéro) et du signe ; UE avec FZ=1 : zéro de même signe.
- Tous les masques sont à 1 après le reset (0x1F80 dans MXCSR).
## Lignes 9751-10000 : 4.10.4.2 à 4.10.4.3 (exceptions non masquées ; NaN de diagnostic), 4.11 sauvegarde et transmission d'état, 4.12 performance (lectures non temporelles, exemples producteur/consommateur)
- Exception non masquée : aucun résultat écrit, sources intactes ; vecteur #XF ou #UD selon CR4.OSXMMEXCPT.
- Sauvegarde : FXSAVE (SSE + x87), XSAVE/XRSTOR (contexte étendu incluant YMM), STMXCSR (MXCSR seul). Passage de paramètres : décrémenter rSP de 16 (XMM) ou 32 (YMM).
- Mélange SSE / MMX / x87 : EMMS avant tout passage à x87.
- MOVNTDQA sur mémoire WC : aligné sur 16 octets ; ne pas utiliser sur des registres d'E/S ayant des effets de bord (lecture spéculative).
- Exemples : producteur/consommateur avec MFENCE pour l'ordre des accès WC.
## Lignes 10001-10250 : fin de 4.12 (alignement, cacheabilité, préchargement), 4.12.8 à 4.12.11 ; début du chapitre 5 (MMX/3DNow!)
- Chapitre 5 : MMX et 3DNow! (vecteurs 64 bits) ; registres MMX = état x87 (pas de nouveaux registres ni de bits de statut) ; sauvegarde par FXSAVE/FNSAVE.
- 3DNow! : AMD ne recommande plus son usage (préférer SSE). Les instructions MMX saturent sans lever d'exception flottante.
- Conseils : éviter les branchements (multiplexeur par comparaison et masque, comme en SSE) ; transposition avant accès colonnes.
- Rappel : cette section est une version 64 bits des mêmes idées que le chapitre 4 ; les détails sont dans vol. 5 du manuel complet.
## Lignes 10251-10500 : fin du branch-removal (5.3.5), 5.3.6 3DNow! (flottants simple précision par paires), 5.4 registres MMX, 5.5 opérandes et types
- 5.3.6 : 3DNow! : deux flottants simple précision par registre 64 bits ; instructions flottantes et entières mélangeables sur les seuls registres MMX.
- 5.4.1 : MMX0-7 = bits bas des FPR0-7 x87 ; la pile x87 (ST) n'est pas utilisée par MMX ; MMX change TOP, les bits d'étiquette x87 et les bits hauts des FPR.
- 5.5.1 : types : flottants simple précision par paire, entiers signés/non signés (quadruple, double, mot, octet), scalaires signés/non signés.
- Pertinence : l'agent n'utilise pas MMX ; la règle "MMX change l'état x87" est importante si un programme mélange les deux (cf. FNINIT/EMMS).
## Lignes 10501-10750 : 5.5 opérandes (registres, mémoire, immédiats, alignement #GP pour 128 bits), 5.5.5 entiers MMX (saturation, arrondi PMULHRW), 5.5.6 3DNow! (format, plage, arrondi, pas d'infini/NaN/dénormal, pas d'exception), 5.6 résumé entier (syntaxe, EMMS/FEMMS, transferts, MOVNTQ/MASKMOVQ)
- 3DNow! : flottant simple précision normalisé uniquement ; arrondi au plus proche sauf conversions vers entier (vers zéro) ; pas d'infini, NaN, dénormal ; pas d'exception flottante : le logiciel doit garantir des opérandes normaux et des résultats dans la plage.
- EMMS / FEMMS : vident l'état MMX (efface la pile x87 à la fin) ; EMMS recommandé pour la compatibilité.
- MOVD : 32 bits vers MMX avec extension à zéro ; MOVQ 64 bits ; MOVDQ2Q / MOVQ2DQ : passage XMM <-> MMX.
- Alignement #GP : CVTPD2PI, CVTTPD2PI, FXSAVE, FXRSTOR si opérande 128 bits non aligné.
- Remarque : les instructions MMX n'utilisent pas rFLAGS (aucun bit modifié).
## Lignes 10751-11000 : MASKMOVQ/MOVNTQ, PMOVMSKB, 5.6.4 conversions MMX, 5.6.5 réordonnancement (pack, unpack, extract/insert, shuffle, swap), 5.6.6 arithmétique (addition, soustraction, multiplication, PMADDWD)
- PMOVMSKB 64 bits : octets -> GPR. PI2FW/PI2FD (3DNow!) : entiers vers flottants ; PI2FD tronque.
- PACK/PUNPCK/PSHUFW/PSWAPD : réordonnancement ; PEXTRW/PINSRW : mot selon immédiat.
- PMULHRW : arrondi (+8000h sur le mot bas) ; PMULUDQ : produit 32x32 non signé sur 64 bits.
- PMADDWD : seul cas de débordement : quatre valeurs 8000h donnent 8000_0000h.
- Addition/soustraction : débordement ignoré (bits bas) ; variantes saturées S et US.
## Lignes 11001-11250 : PAVG, PSADBW, décalages MMX, comparaisons, min/max, logique (PAND/PANDN/POR/PXOR), sauvegarde (FSAVE/FNSAVE/FRSTOR, FXSAVE), 5.7 flottants 3DNow! (conversion, arithmétique, réciproque, racine inverse, comparaison)
- 5.6.10 : FSAVE (FWAIT + FNSAVE, réinitialise x87), FNSAVE (sans FWAIT), FRSTOR (94 ou 108 octets selon la taille) ; FXSAVE/FXRSTOR : 512 octets, sans réinitialisation de x87.
- 5.7.2 : PF2IW/PF2ID (tronque, NaN/infini/dénormal non supportés) ; CVTPS2PI (arrondi selon MXCSR) et CVTTPS2PI (tronque).
- 5.7.3 : PFADD, PFSUB, PFSUBR, PFMUL, PFACC, PFNACC, PFPNACC ; PFRCP/PFRCPIT1/PFRCPIT2 et PFRSQRT/PFRSQIT1 : approximation puis affinage Newton-Raphson ; 15 bits de précision par étape.
- 5.7.4 : PFCMPEQ, PFCMPGT, PFCMPGE (masque), PFMAX, PFMIN.
- 5.8 : les instructions MMX et 3DNow! ne lisent ni n'écrivent rFLAGS ni les drapeaux d'exception x87 ; elles n'utilisent pas le masque x87.
- 5.9.1 : préfixes utilisables (67h, 66h, segments) identiques à la section 4.8.
## Lignes 11501-11750 : 6.2 registres x87 (FPR, pile ST, TOP), mot d'état FSW (bits B, C3..C0, TOP, ES, SF, PE..IE), mot de contrôle FCW (début)
- Pile x87 : TOP (bits 13:11 de FSW) = index physique de ST(0) ; un push décrémente TOP.
- FSW : IE 0, DE 1, ZE 2, OE 3, UE 4, PE 5, SF 6 (pile), ES 7 (exception non masquée), C0 8, C1 9 (C1=1 = dépassement de pile), C2 10, TOP 13:11, C3 14, B 15.
- FCW : masques IM 0 ... PM 5 ; PC bits 9:8 ; RC bits 11:10 ; Y bit 12 (compatibilité 80287). Init FINIT/FNINIT = 037Fh (arrondi au plus proche, précision étendue, tout masqué).
- Pertinence : FNINIT est la valeur de référence pour interpréter FCW dans un rapport.
## Lignes 11751-12000 : FCW (masques, PC tableau 6-1, RC tableau 6-2, Y), FTW (tags 00 valide, 01 zéro, 10 spécial, 11 vide ; FINIT écrit tout vide), pointeurs et opcode x87 (6.2.5), environnement x87 (tableau 6-4), émulation CR0.EM, types de données (début)
- PC : 00 simple (réservé), 01 réservé, 10 double, 11 étendue (défaut) ; n'affecte que F(I)ADD/SUB/MUL/DIV et FSQRT.
- Pointeur d'instruction : 64 bits en mode 64 bits (sans CS) ; en protégé/compatibilité CS:eIP ; en réel/8086 adresse linéaire. Opcode 11 bits : opcode[10:8] = bits bas du premier octet, opcode[7:0] = second octet. FINIT/FNINIT remettent tout à zéro.
- FNSAVE/FRSTOR/FLDENV : environnement complet ou sans les registres de données. FSTCW/FLDCW : FCW seul.
- CR0.EM = 1 : x87 lève #NM (sauf WAIT/FWAIT) : émulation logicielle.
## Lignes 12001-12250 : types x87 (flottants simple/double/étendu 80 bits, entiers 16/32/64, BCD 80 bits), plages (tableau 6-5), nombres (normaux, dénormaux, pseudo-dénormaux, zéro, infini, NaN), 6.3.4 encodages
- Étendu 80 bits : signe 1, exposant 15 bits (biais 16383), significande 64 bits avec bit entier explicite. Simple : exposant 8 (biais 127), fraction 23. Double : exposant 11 (biais 1023), fraction 52.
- FBLD/FBSTP : 18 chiffres BCD sur 72 bits, signe bit 79, bits 78:72 ignorés/zéro.
- Non supportés : unnormal, pseudo-infini, pseudo-NaN (IE si utilisés). Pseudo-dénormal accepté en entrée (traité comme exposant 1, DE).
## Lignes 12251-12500 : table 6-7 (NaN résultants), encodages supportés (tableau 6-8), non supportés (tableau 6-9, pseudo-NaN, pseudo-infini, non normaux), indéfinis x87 (6.3.4.3)
- Résultat NaN : si deux QNaN, le plus grand significande ; si deux SNaN, le plus grand significande converti en QNaN ; sinon le QNaN.
- Précision étendue : bit entier explicite en bit 63 ; fraction en 62..0. Biais : 127 / 1023 / 16383.
- Non supportés : pseudo-NaN, pseudo-infini, non normaux (IE si utilisés en entrée). Le processeur ne les produit jamais.
- Indéfini : NaN QNaN négatif (significande 1.100...0) ; entiers 80..00h ; BCD : indéfini sans autre sens.
## Lignes 12501-12750 : indéfinis (tableau 6-10), précision PC (6.3.5), arrondi (6.3.6), 6.4 résumé x87 (syntaxe, transferts et conversions, FCMOVcc, FXCH, FXTRACT, constantes, arithmétique, FPREM, FSQRT, début transcendantales)
- Précision : simple 24 bits, double 53, étendue 64 ; réduction : bits inutiles remis à zéro.
- FISTTP : troncature, ignore RC. FBSTP : arrondi selon RC.
- FCMOVcc : conditions dans rFLAGS (tableau 6-13) ; FPREM (troncature) vs FPREM1 (arrondi au plus proche, conforme IEEE 754).
- Constantes : FLDZ, FLD1, FLDPI, FLDL2E, FLDL2T, FLDLG2, FLDLN2.
- FABS/FCHS : bit de signe seul ; FRNDINT : arrondi selon RC.
## Lignes 12751-13000 : transcendantales (FSIN, FCOS, FSINCOS, FPTAN, FPATAN, F2XM1, FSCALE, FYL2X, FYL2XP1), comparaisons (FCOM, FCOMI, FUCOM, FICOM, FTST), FXAM, pile (FDECSTP, FINCSTP, FFREE), FNOP, début 6.4.9 contrôle
- Arguments trigonométriques : |x| < 2^63 seulement ; hors plage : C2 = 1, argument renvoyé inchangé ; réduction avec FPREM/FPREM1.
- FCOMI : ZF, PF, CF = 0/0/0 (>), 0/0/1 (<), 1/0/0 (=), 1/1/1 (non ordonné) ; FUCOMI IE seulement pour SNaN ; FCOMI meilleur que FSTSW AX + branchement.
- FXAM : C3 C2 C0 + C1 (signe) = classification (normal, zéro, infini, NaN, dénormal, vide, non supporté).
- FFREE : marque un registre vide ; FDECSTP/FINCSTP changent TOP sans toucher aux tags.

## Lignes 13001-13250 (x87 : FINIT/FNINIT, FWAIT, FCLEX, FLDCW/FSTCW, FSTSW, FLDENV/FSTENV, FSAVE/FRSTOR, FXSAVE/FXRSTOR, rFLAGS, préfixes, exceptions #MF)
- FINIT = FWAIT + FNINIT. FNINIT/FINIT écrivent CW=037Fh (arrondi au plus proche, toutes exceptions masquées, précision étendue double), tag word = vide (tous registres vides), SW et les 4 bits de condition à 0, pointeurs et état d'opcode à 0.
- FWAIT/WAIT : teste et traite les exceptions x87 non masquées en attente.
- FCLEX/FNCLEX : effacent les flags d'exception de SW, le stack-fault et le busy ; les 4 bits de condition sont indéfinis.
- FLDCW : une CW qui démasque une exception en attente la signale à la prochaine instruction x87 non-contrôle ou MMX/64-bit.
- FSTSW AX : forme ancienne ; FCOMI + FCMOVcc est préféré pour les branchements.
- FLDENV/FSTENV : bloc de 14 octets (16 bits) ou 28 octets (32 bits) ; préfixe 66h sélectionne le format, REX.W force le format 32 bits.
- FSAVE/FRSTOR : 94 ou 108 octets ; FSAVE effectue un FINIT après sauvegarde.
- FXSAVE/FXRSTOR : 512 octets ; ne réinitialise pas l'état x87 et ne signale pas les exceptions en attente. REX.W choisit le format 64 bits.
- Table 6-16 : FCMOVcc/FCOMI/FCOMIP/FUCOMI/FUCOMIP lisent ou modifient ZF, PF, CF (OF/SF/AF à 0 pour les comparaisons COMI).
- Préfixes : 66h, 67h, segments (CS/SS/DS/ES ignorés en 64 bits), REX.W utilisé par FXSAVE/FLDENV ; REP/REPNE ignorés ; LOCK (F0h) = #UD.
- Détection : CPUID 1 et 8000_0001 bit 0 (FPU), bit 15 (CMOV), bit 24 (FXSR), bit 29 (LM).
- Exceptions générées : #DB, #BP, #UD, #NM, #DF, #SS, #GP, #PF, #MF (vecteur 16), #AC, #MC. Non générées : #DE, NMI, #OF, #BR, #CSO, #TS, #NP, #XF.
- Exceptions #MF = OU logique de 6 exceptions x87 : IE (bit 0), DE (bit 1), ZE (bit 2), OE (bit 3), UE (bit 4), PE (bit 5), plus SF (stack fault, toujours avec IE).
- IE causes : opérandes SNaN ou formats non supportés (pseudo-NaN, pseudo-infini, non normal) ; FYL2X (négatif, zéro, infini), FYL2XP1 (< -1), FCOS/FPTAN/FSIN/FSINCOS (infini).
- Relevance pour l'agent : aucun code agent ne touche l'x87 pour l'instant ; ne pas exécuter FINIT/FNINIT sans nécessité (écrit l'état). Le FPU est nécessaire au firmware UEFI, donc l'agent doit éviter de changer CW/SW sans restauration.

## Lignes 13251-13500 (causes #MF, priorités, masques, réponses masquées)
- IE : FCOM/FCOMP/FCOMPP/FCOMI/FCOMIP sur QNaN ; FPREM/FPREM1 (dividende infini ou diviseur nul) ; FIST/FISTP/FISTTP (dépassement de taille) ; FBSTP (dépassement BCD) ; débordement/sous-passage de pile (SF, C1=1 dépassement, C1=0 sous-passage).
- DE : opérande dénormalisé dans une arithmétique, ou chargement simple/double précision dénormalisé.
- ZE : diviseur nul (FDIV*/FIDIV*) sur dividende fini non nul ; FYL2X ou FXTRACT sur zéro.
- OE/UE : arrondi hors plage normale ; PE : résultat inexact.
- SF : débordement/sous-passage de pile ; toujours accompagné de IE ; pas de bit de masque.
- Priorité (Table 6-19) : 1-4 IE+SF/unsupported/SNaN pré-calcul ; 5 QNaN ; 6 ZE ; 7 DE ; 8 OE/UE post ; 9 PE post.
- Piège : une exception pré-opération non masquée est signalée au prochain bord d'instruction non-contrôle ; du code non-x87 peut écraser les opérandes entre-temps. Insérer un WAIT après les opérations à risque.
- Masques CW : IM bit0, DM bit1, ZM bit2, OM bit3, UM bit4, PM bit5. FINIT/FNINIT = tous masqués ; reset = tous démasqués.
- Réponses masquées : SNaN -> IE + QNaN ; unsupported/inf-inf/0*inf/0/0/racine négative -> IE + indéfini ; FCOM avec NaN -> IE, C3-C0 selon résultat ; FCOMI avec NaN -> IE, EFLAGS "non ordonné" ; FIST overflow -> IE + valeur entière indéfinie ; FXCH sur registre vide -> IE, échange avec indéfini ; FDIV/0 -> ZE + infini signé (XOR des signes) ; FYL2X ST0=0 -> ZE + infini ; FXTRACT 0 -> ZE ; DE masqué -> résultat avec opérande dénormalisé.
- Pour l'agent : aucune instruction x87 n'est encore utilisée ; le code de sonde doit rester CPUID/MSR et ne pas toucher à CW/SW/TW.

## Lignes 13501-13789 (arrondi OE/UE, PE, réponses non masquées, FERR#/IGNNE#, NaN de diagnostic, sauvegarde d'état, performance, début d'index)
- OE masqué : arrondi nearest -> infini signé ; toward+/- et toward 0 -> infini ou plus grand fini selon le signe.
- UE masqué : résultat dénormalisé et inexact -> UE + résultat dénormalisé ; dénormalisé non inexact -> pas de UE.
- PE : positionne PE, C1 = arrondi vers le haut ; masqué OE/UE -> PE + réponse.
- Réponses non masquées (Table 6-22) : IE/DE/ZE -> flags + ES, appel #MF, destination et TOP inchangés. OE/UE sur registre : division/multiplication par 2^24576 (sic, la valeur du PDF), arrondi, écriture, push/pop, ES, #MF. OE/UE sur mémoire : destination et TOP inchangés.
- Chaque réponse non masquée assert FERR# en fin d'instruction.
- CR0.NE = 1 (recommandé) : #MF immédiat au prochain bord non-contrôle. CR0.NE = 0 : FERR#/IGNNE# externes gèrent (FERR# bloque si IGNNE#=0).
- NaN de diagnostic : SNaN préchargé pour déclencher IE ; mécanisme purement logiciel.
- Sauvegarde : FSAVE/FNSAVE/FRSTOR (94/108 octets) ; FSAVE gère les exceptions en attente, FNSAVE non ; après sauvegarde, tags = vides. FXSAVE/FXRSTOR : ne modifient pas les tags, ne gèrent pas les exceptions ; FINIT nécessaire pour invalider.
- Performance : préférer SSE/XMM ; FCOMI+FCMOVcc ; FSINCOS au lieu de FSIN+FCOS ; FXCH pour chaînes de dépendances.
- Index : rappel de #AC, #BP, #BR, #DB, #DE, #DF, #GP, #MC, #MF, #NM, #NP, #OF, #PF, #SS, #SX, #TS, #UD, #XF ; pages de référence pour FPU (288-297 registres, 289 FSW, 292 CW, 294 FTW, 298 CR0.EM).
- Pour l'agent : toute gestion de FERR/NE/IGNNE ou de la CW doit être évitée ; la sonde ne doit pas émettre FINIT/FSAVE/FXSAVE sans sauvegarde explicite.

## Lignes 13790-14073 (index uniquement : pas de contenu normatif nouveau)
- Index consulté pour retrouver les pages des sujets déjà lus : #MF (pages 94, 278, 291, 329), FCW/FSW/FTW (292, 289, 294), environnement x87 (297, 325), FSAVE/FXSAVE (269-270, 325), MXCSR (115-117), rFLAGS (34-36), préfixes (76-79, 326), REX (26), CPUID (70, 80, 276, 327), SSE/AVX/FMA/XOP (xx-xxvii), priorités d'exceptions (332), réponses masquées/non masquées (333, 337).
- Remarque : les notes de 13501-13789 mentionnent 2^24576 pour OE/UE ; la valeur est restituée depuis le texte extrait (exposant superscript perdu), à vérifier dans le PDF si nécessaire.

## FIN DE LECTURE APM VOL.1 (lignes 1-14073)
- Volume 1 AMD64 APM rev 3.23 lu intégralement, ligne par ligne par blocs de 250, notes dans ce fichier.
- Application à l'agent : aucune logique x87/SSE/MMX n'est exécutée par l'agent ; seules CPUID, CR0/CR4 (lecture), MSR de liste fixe et PCI/NVMe/xHCI/HDA/USB sont utilisés. Les bits CR0.NE/TS/EM et les réglages x87 ne doivent pas être modifiés.
- Prochaine étape demandée : NVMe 2.4, puis UEFI 2.11, puis HDA (vérification du code contre chaque doc, log dans SPECS_VERIFIES.md).
