# Spécification Architecture Réseau, Pare-feu, IDS/IPS, VPN & Tor — ZERO OS (v1.0)

**Statut : Spécification Système Native Bare-Metal — 0 Dépendance Python.**

Le présent document définit la spécification d'architecture réseau et de sécurité intégrée au noyau bare-metal ZERO OS pour les ordinateurs x86_64, avec ciblage direct du PC **ASUS VivoBook M1603QA** (AMD Ryzen 7 5800H, MediaTek MT7921 `VEN_14C3&DEV_7961`).

---

## 1. Couche Matérielle & Pilotes Réseau Bare-Metal

1. **Bus PCI/PCIe Express** :
   - Balayage direct de la configuration PCI (IO Ports 0xCF8 / 0xCFC ou MMIO ECAM) pour la détection de la carte Wi-Fi MediaTek MT7921 (`VEN_14C3&DEV_7961`).
   - Allocation de descripteurs DMA ring-buffers sécurisés pour la transmission et réception de paquets.
2. **Gestion des Interruptions Réseau** :
   - Gestionnaire d'interruptions MSI/MSI-X dédié avec traitement temps-réel non bloquant pour le moteur d'accessibilité.

---

## 2. Pile Protocolaire Réseau & Cryptographie

1. **Protocoles Réseau de Base** :
   - Ethernet II, ARP, IPv4, IPv6, ICMP/ICMPv6, UDP, TCP avec gestionnaires de fenêtres glissantes et timeouts.
2. **Cryptographie Sécurisée** :
   - Primitives cryptographiques éprouvées (Curve25519, ChaCha20-Poly1305, AES-GCM, SHA-256) sans réimplémentation improvisée.
   - TLS 1.3 natif pour les communications sécurisées HTTPS.

---

## 3. Pare-feu, IDS/IPS & Politiques de Confidentialité

1. **Pare-Feu Natif (Inbound/Outbound)** :
   - Filtrage de paquets par règles d'interface, adresse IP source/destination, ports et protocole (TCP/UDP/ICMP).
   - Mode "VPN Obligatoire" (Kill-Switch) bloquant tout trafic direct non chiffré.
2. **IDS/IPS (Système de Détection & Prévention des Intrusions)** :
   - Analyse d'anomalies de protocole et détection de balayages de ports.
   - Action IPS automatique (blocage d'adresse IP suspecte, limitation de débit).
3. **Canal d'Accessibilité Matériel Intégré (HAC)** :
   - Notification sonore et vocale instantanée lors de la détection d'intrusions ou du blocage de paquets par le pare-feu.

---

## 4. Routage VPN & Tor Natif

1. **VPN WireGuard / TUN-TAP** :
   - Gestion native des tunnels WireGuard avec échange de clés Curve25519 et chiffrement ChaCha20-Poly1305.
2. **Client Tor & Services Onion** :
   - Routage en circuits anonymisés à 3 nœuds pour les applications autorisées et accès natif aux domaines `.onion`.
3. **Protection contre les Fuites** :
   - Prévention des fuites DNS, IPv6 et de routes secondaires par isolation stricte du canal réseau.
