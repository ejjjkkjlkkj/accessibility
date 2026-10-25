# Spécification Canonique de l'Architecture du Jeu d'Instructions : ZERO-ISA 64-bit (v1.0)

**Statut : Spécification Canonique Tier-0 64-bit Native — Indépendante de tout système d'exploitation ou langage tiers.**

L'ISA ZERO (Instruction Set Architecture) est une architecture de processeur 64-bit pure et native, conçue pour s'exécuter directement sur le matériel x86_64 / ARM64 bare-metal ou en firmware UEFI sans aucune dépendance envers un runtime, un langage hébergé (0 language tier) ou un système d'exploitation hôte.

---

## 1. Vue d'Ensemble & Principes Fondamentaux

1. **Autonomie Tier-0 Pure 64-bit** : Le processeur fonctionne exclusivement en mots 64-bit (instructions et données) du démarrage à froid (Reset Vector à `0x0000_0000_0000_0000`) jusqu'à l'exécution du noyau d'OS et des modèles d'IA gigantesques.
2. **Format d'Instruction 64-bit et Extension 128-bit** :
   - Instructions Standard de 64-bit (mot unique) pour toutes les opérations système, arithmétique 64-bit, mémoire et contrôle.
   - Instructions Étendues de 128-bit (double mot) pour l'accélération matérielle d'IA (Tenseurs, MatMul, Attention, Quantification).
3. **Niveaux de Privilège Matériels (Rings 0 à 3)** :
   - **Ring 0 (Firmware / Hardware Core)** : Accès direct aux registres physiques 64-bit, interruptions bas niveau, boot UEFI.
   - **Ring 1 (Noyau OS / MMU)** : Gestion des tables de pages 4 niveaux 64-bit, mécanismes de sécurité, pilotes de périphériques.
   - **Ring 2 (Services Système & Runtime)** : Tâches système isolées, pipelines audio et accessibilité.
   - **Ring 3 (Utilisateur / Modèles IA)** : Applications utilisateur et exécution isolée du graphe de calcul IA.
4. **Canal d'Accessibilité Matériel Intégré (HAC)** : Registres 64-bit dédiés à l'émission directe de signaux braille, flux audio PCM et gestion d'événements de focus sans dépendance à un OS.

---

## 2. Registres du Processeur (64-bit Nactifs)

Le processeur ZERO-ISA dispose de 4 jeux de registres 64-bit natifs :

### 2.1 Registres Généraux (GPR) - 64-bit
- `r0` : Toujours égal à zéro (`0x0`).
- `r1` à `r27` : Registres d'usage général 64-bit pour le calcul et les adresses.
- `r28` (`gp`) : Pointeur global 64-bit (Global Pointer).
- `r29` (`tp`) : Pointeur de thread / contexte 64-bit (Thread Pointer).
- `r30` (`fp`) : Pointeur de frame 64-bit (Frame Pointer).
- `r31` (`sp`) : Pointeur de pile 64-bit (Stack Pointer).

### 2.2 Registres Tenseurs & Vectoriels (VECT/TENSOR) - 512-bit
- `t0` à `t31` : Registres vectoriels de 512 bits chacun (16x float32, 32x float16, 64x int8).

### 2.3 Registres d'Accessibilité Matérielle (HAC) - 64-bit
- `a0` (`HAC_ROLE`) : Identifiant du rôle accessible 64-bit.
- `a1` (`HAC_STATE`) : Masque d'état 64-bit.
- `a2` (`HAC_EVENT`) : Type d'événement matériel 64-bit.
- `a3` (`HAC_AUDIO_BUF`) : Adresse mémoire physique 64-bit du tampon audio PCM.
- `a4` (`HAC_AUDIO_LEN`) : Longueur 64-bit du tampon audio.
- `a5` à `a7` : Registres 64-bit braille et retours haptiques.

### 2.4 Registres de Contrôle et d'État Système (CSR) - 64-bit
- `csr0` (`SYS_STATUS`) : État du processeur 64-bit.
- `csr1` (`SYS_PBDT`) : Adresse physique 64-bit de la table de pages racine.
- `csr2` (`SYS_EPC`) : Pointeur d'instruction exception 64-bit.
- `csr3` (`SYS_CAUSE`) : Code de cause d'interruption 64-bit.
- `csr4` (`SYS_TVEC`) : Adresse du vecteur d'interruption 64-bit.
- `csr5` (`SYS_CYCLE`) : Compteur 64-bit de cycles d'horloge.

---

## 3. Formats d'Encodage 64-bit et 128-bit

### 3.1 Format Standard 64-bit (8 octets)

```text
Format Standard 64-bit :
 [63:58 Opcode (6b)] [57:53 Rd (5b)] [52:48 Rs1 (5b)] [47:43 Rs2 (5b)] [42:0 Imm43 / Subcode (43b)]
```

### 3.2 Format Étendu IA / Tenseur 128-bit (16 octets)

```text
Format Tenseur 128-bit :
 Mot 0: [63:58 Opcode (6b)] [57:53 Td (5b)] [52:48 Ts1 (5b)] [47:43 Ts2 (5b)] [42:38 Ts3 (5b)] [37:0 Reserved]
 Mot 1: [63:32 Precision & Control] [31:0 Immediate Extended 32b]
```

---

## 4. Jeu d'Instructions Exhaustif 64-bit

### 4.1 Arithmétique & Logique 64-bit
- `ADD Rd, Rs1, Rs2` : `Rd = Rs1 + Rs2` (64-bit)
- `SUB Rd, Rs1, Rs2` : `Rd = Rs1 - Rs2` (64-bit)
- `MUL Rd, Rs1, Rs2` : `Rd = Rs1 * Rs2` (64-bit)
- `ADDI Rd, Rs1, Imm` : `Rd = Rs1 + Imm` (64-bit)

### 4.2 Accès Mémoire 64-bit & Atomiques
- `LD Rd, Imm(Rs1)` : Charge 64 bits depuis `Mémoire[Rs1 + Imm]` dans `Rd`.
- `ST Rs1, Rs2, Imm` : Stocke 64 bits de `Rs2` à `Mémoire[Rs1 + Imm]`.

### 4.3 Contrôle de Flux & Sauts 64-bit
- `JMP Target` : Saut inconditionnel 64-bit.
- `BEQ Rs1, Rs2, Offset` : Branchement 64-bit si égalité.
- `HALT` : Arrêt du processeur.

### 4.4 Extension IA Gigantesque (128-bit)
- `TMATMUL Td, Ts1, Ts2` : Produit matriciel rapide de tenseurs 512-bit.
- `TVGELU Td, Ts1` : Activation GELU sur registre de tenseurs.
- `TVATTEN Td, Ts1, Ts2, Ts3` : Bloc d'attention multi-tête vectoriel.

### 4.5 Canal d'Accessibilité Matériel Intégré (HAC 64-bit)
- `HAC_EMIT_EVENT EventType, Role, State` : Émission d'événement matériel d'accessibilité.
- `HAC_SPEAK_AUDIO BufReg, LenReg` : Lecture audio PCM 64-bit directe.

---

## 5. Preuve de Conformité

L'émulateur et l'assembleur 64-bit de référence dans `ZERO/zero_isa_reference.py` exécutent ce jeu d'instructions 64-bit sans dépendance hôte et sont validés par `tests/test_zero_isa.py`.
