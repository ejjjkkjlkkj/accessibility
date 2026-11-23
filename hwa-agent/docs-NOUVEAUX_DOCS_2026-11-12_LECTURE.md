# Nouveaux documents ajoutés le 2026-11-12 (dossier Documents) — lecture complète

Relevé fait le 2026-11-12 à 15:22 : deux fichiers nouveaux seulement (créés à 13:20 et 13:22). Aucun autre fichier créé depuis 13:00 dans Documents, Téléchargements ou Bureau.

## 1. MT7921LEN Product Brief (août 2023) — 88 lignes de texte, lu en entier

Source : <DOSSIER_DOCUMENTS>\MT7921LEN-Product_Brief_202308.pdf ; texte : <DOSSIER_SPECS>\MT7921LEN_Product_Brief_202308.txt

- Puce combinée Wi-Fi 802.11a/b/g/n/ac/ax 2T2R (Wi-Fi 6, 2,4 GHz en 20/40 MHz, 5 GHz en 20/40/80 MHz) + Bluetooth 5.2.
- Deux sous-systèmes, chacun avec son microcontrôleur RISC 32 bits, SRAM/ROM embarquées.
- Interface hôte : périphérique PCIe conforme à PCIe v2.1. UART avec contrôle de flux matériel, GPIO multiplexés.
- Alimentation unique 3,3 V, quartz 40 MHz ; boîtier DRQFN 11,5 x 8,7 mm, 109 broches ; -10 à 70 °C.
- WLAN : MU-MIMO RX, DBDC (MT7921), STBC, LDPC, beamforming ; 802.11 d/e/h/i/j/k/mc/r/v/w ; sécurité annoncée : WPA/WPA2 personnel, WPS 2.0, WAPI ; WMM.
- Bluetooth : 2.1+EDR, 4.2 LE, 5.2, canaux isochrones LE, jusqu'à 7 liens BT + 16 liens BLE.
- eFuse pour les données propres à la puce et la calibration RF ; coexistence Wi-Fi/Bluetooth.
- Les sections « brochage », « boîtier » et « marquage » sont des figures : rien dans le texte.

- [AGENT] Ce que ce document apporte au pilote Wi-Fi de l'OS : (1) le périphérique est une fonction PCIe ordinaire (identifiant fabricant 14C3 vu sous Windows) : énumération, BAR mémoire, MSI comme pour les autres. (2) La puce contient son propre processeur : elle ne fonctionne qu'après chargement d'un microprogramme (firmware) par l'hôte ; le MAC 802.11 bas niveau tourne dans la puce, l'hôte lui parle par messages. (3) La calibration RF est dans l'eFuse : ne jamais écrire l'eFuse.
- [AGENT] Ce qui MANQUE (document insuffisant pour écrire le pilote) : aucune carte des registres, aucun format des messages hôte-puce, aucune procédure de chargement du firmware, aucun format des anneaux d'émission/réception. Ces informations ne sont pas publiées par MediaTek dans un PDF public ; la seule description publique connue est le code source du pilote Linux « mt76 / mt7921 » (licence libre) et les fichiers firmware distribués avec linux-firmware. Les utiliser comme référence d'interface est une décision de l'utilisateur (règle « entièrement à nous » : contrat d'interopérabilité seulement).
- [AGENT] Écart avec ce qui a été observé sur la machine : le pilote Windows annonce WPA3 ; la fiche ne cite que WPA/WPA2 : WPA3 (SAE) est fait côté hôte, pas par la puce.

## 2. PCI-SIG « PCI Express Technology for Automotive Functional Safety (FuSa) » — 526 lignes de texte, lu en entier

Source : <DOSSIER_DOCUMENTS>\PCI-SIG PCIe for Automotive Functional Safety_v5_Final.pdf ; texte : <DOSSIER_SPECS>\PCISIG_PCIe_Automotive_Functional_Safety_v5.txt

- Nature : diaporama de webinaire (27 diapositives, intervenants Synopsys et Intel). Ce n'est PAS une spécification et il ne contient aucun registre hormis un en-tête.
- Contenu : usages du PCIe dans l'automobile (calcul, dorsale de données, stockage NVMe, télématique Wi-Fi/Bluetooth/5G) ; rappel des normes IEC 61508 et ISO 26262 (défauts systématiques et défauts matériels aléatoires, permanents ou transitoires, niveaux ASIL) ; exemple « alerte de franchissement de ligne ».
- Mécanismes PCIe utiles à la sûreté : LCRC 32 bits par lien avec retransmission ; ECRC 32 bits de bout en bout dans le champ TLP Digest (le LCRC est recalculé par les commutateurs, d'où l'intérêt de l'ECRC) ; Advanced Error Reporting (AER) : capacité étendue d'identifiant 0x0001, version 1 ou 2, en-tête { ID bits 15:0, version bits 19:16, décalage de la capacité suivante bits 31:20 }, erreurs corrigibles et non corrigibles, gravité, masques, source ; erreurs signalées par le champ d'état de complétion ou par messages d'erreur vers le root complex.
- Mécanismes ajoutés pour l'ASIL B : parité sur les chemins de données, ECC sur les mémoires, protection de l'espace de registres, autotest (LBIST), gestionnaire de sûreté ; réaction prévue : reset sur défaut permanent, correction ou rejeu sur défaut transitoire.

- [AGENT] Utile pour l'OS : (1) le format de l'en-tête des capacités étendues PCIe (ID 16 bits, version 4 bits, suivant 12 bits) et l'identifiant AER = 0x0001 : permet de parcourir l'espace étendu à partir de l'offset 0x100 et de lire les erreurs du lien du NVMe ou de la carte Wi-Fi (lecture seule). (2) Tout le reste est du contexte automobile sans effet sur le projet.
- [AGENT] Ce document ne remplace pas la spécification PCI Express de base, toujours absente du dossier (ligne « PCI » de DOCS_UTILES.md).

## 3. Recherche de documents MediaTek (2026-11-12, après la remarque « il y a un document MediaTek »)

### Sur le PC
- Aucun autre document MediaTek que la fiche produit (recherche par nom : mediatek, mt79, mt76, 7921, 7922, wifi, wlan ; profil utilisateur et tous les dossiers de C:\ hors Windows et Program Files). Le lecteur D: n'a pas été fouillé.
- En revanche le pilote Windows installé contient les fichiers de la puce, dans C:\Windows\System32\DriverStore\FileRepository :
  - mtkwl6ex.inf_amd64_66f69464451b48c3 : mtkwl6ex.sys (pilote Wi-Fi, 1 430 Ko), mtkwl6ex.inf (150 Ko), mtkihvx.dll, WIFI_MT7961_patch_mcu_1_2_hdr.bin (106 Ko, correctif du microcontrôleur), WIFI_RAM_CODE_MT7961_1.bin (662 Ko, microprogramme Wi-Fi).
  - mtkbtfilter.inf_amd64_121188dc27ea91c1 : mtkbtfilterx.sys, BT_RAM_CODE_MT7961_1_2_hdr.bin (518 Ko, microprogramme Bluetooth).
- [AGENT] Constat : le nom interne de la puce est MT7961 (c'est le nom porté par les fichiers de microprogramme du MT7921). Les deux fichiers Wi-Fi (patch + RAM code) sont ceux que l'hôte doit envoyer à la puce au démarrage ; ils sont la propriété de MediaTek (à charger tels quels, non modifiables, non redistribuables sans vérifier la licence). Fichiers seulement repérés, pas ouverts.

### En ligne (documents officiels MediaTek)
- Fiche produit MT7921LEN (déjà lue) : https://i.mediatek.com/hubfs/MediaTek%20Assets/Pdfs/Genio%20Documents/MT7921LEN-Product_Brief_202308.pdf
- Page officielle « MT7921 Module » de la documentation Genio IoT Yocto : https://genio.mediatek.com/doc/iot-yocto/latest/sw/yocto/app-dev/Wi-Fi/Wi-Fi-Module-MT7921.html (lue par résumé automatique de la page, pas mot à mot). Contenu : variantes MT7921L (Wi-Fi en PCIe, Bluetooth en USB, M.2 clé E) et MT7921S (SDIO) ; pilote Linux nommé MT7921E pour le PCIe (options CONFIG_MT7921E, CONFIG_BT_HCIBTUSB_MTK) ; exemples d'arbre matériel pour les cartes Genio ; alimentation 3,3 V. Aucun registre, aucun nom de fichier firmware, aucun lien vers une datasheet.
- Aucune datasheet de niveau registre du MT7921 trouvée en accès public (deux recherches). Une datasheet d'une autre puce (MT7905) circule sur un site tiers avec un filigrane au nom d'une société : diffusion non publique, à ne pas utiliser.
- [AGENT] Conclusion inchangée : MediaTek confirme lui-même que le support de cette puce passe par le pilote Linux mt76/mt7921 (code ouvert). C'est la seule description publique de l'interface hôte (registres WFDMA, anneaux, messages MCU, chargement du patch puis du RAM code). Pour notre carte : Wi-Fi sur PCIe, Bluetooth sur USB (donc via le pilote xHCI).
