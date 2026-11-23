# Deploiement sur la cle HWA sans remplacer le premier agent

## Ce qui reste intact
- `EFI\BOOT\BOOTX64.EFI` : premier agent, jamais modifie par le script.
- `EFI\HWA\`, `EFI\HWA2\` : rapports et donnees deja produits, jamais modifies.
- BIOS (puce SPI), NVRAM (ordre de demarrage), disque interne : jamais touches.

## Ce que fait le script
`scripts\deployer_cle.ps1` depose un binaire EFI dans `EFI\HWA3\` de la cle :
- refuse si le lecteur n'est pas la cle HWA amovible ;
- refuse un fichier qui n'est pas un PE (signature MZ) ou de taille suspecte ;
- n'ecrase jamais un fichier existant (sauf `-Ecraser`) ;
- verifie le SHA-256 apres copie ;
- sans `-Confirmer`, n'ecrit rien (plan seulement).

## Demarrer la nouvelle version
Menu de demarrage ponctuel du BIOS (Echap au logo ASUS), choisir la cle, puis `EFI\HWA3\<nom>.efi`.
Le demarrage par defaut ne change pas. Rien n'est enregistre dans la NVRAM.

## Ce qui n'existe pas encore : la couche « firmware pour l'OS »
Une couche qui tourne apres la sortie des services UEFI, avec son propre ordonnancement, ses pilotes
(xHCI, HDA, NVMe, clavier) et son interface, est un projet en plusieurs etapes :
1. separer les pilotes en modules sans allocation ni service UEFI (fait en grande partie) ;
2. boucle d'evenements unique (clavier, voix, lecture de rapport) ;
3. stockage de ses propres journaux, car la cle n'est plus accessible apres la sortie ;
4. gestion des erreurs sans console UEFI.
Les points 1 et 2 sont dans `src/` ; les points 3 et 4 restent a faire.
