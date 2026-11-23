# Depose un binaire EFI sur la cle HWA SANS remplacer ce qui existe deja.
#
# Regles :
#  - n'ecrit que dans EFI\HWA3\ de la cle (jamais EFI\BOOT, jamais EFI\HWA, jamais EFI\HWA2) ;
#  - ne modifie ni la NVRAM, ni le BIOS, ni le disque interne, ni l'ordre de demarrage ;
#  - n'ecrase jamais un fichier existant (sauf -Ecraser explicite) ;
#  - sans -Confirmer : affiche seulement le plan (aucune ecriture).
#
# Demarrage ensuite : menu de demarrage ponctuel du BIOS (Echap au logo ASUS), choisir la cle
# puis EFI\HWA3\<nom>. Le demarrage par defaut ne change pas.
#
# Exemple :
#   powershell -ExecutionPolicy Bypass -File scripts\deployer_cle.ps1 -Binaire target\x86_64-unknown-uefi\release\hwa2_agent.efi -Nom hwa3_essai.efi
#   (ajouter -Confirmer pour ecrire)

param(
    [Parameter(Mandatory = $true)] [string] $Binaire,
    [Parameter(Mandatory = $true)] [string] $Nom,
    [string] $Lettre = "D",
    [switch] $Confirmer,
    [switch] $Ecraser
)

$ErrorActionPreference = "Stop"

if ($Nom -notmatch '^[A-Za-z0-9_\-]+\.efi$') { throw "Nom invalide : $Nom (attendu : lettres, chiffres, _ -, suffixe .efi)" }

$vol = Get-Volume -DriveLetter $Lettre -ErrorAction Stop
if ($vol.FileSystemLabel -ne "HWA") { throw "Le lecteur $Lettre n'est pas la cle HWA (etiquette : '$($vol.FileSystemLabel)')." }
if ($vol.DriveType -ne "Removable") { throw "Le lecteur $Lettre n'est pas amovible : refus." }

$src = Resolve-Path $Binaire -ErrorAction Stop
$taille = (Get-Item $src).Length
if ($taille -lt 4096 -or $taille -gt 4MB) { throw "Taille suspecte pour un binaire EFI : $taille octets." }
$magic = [IO.File]::ReadAllBytes($src)[0..1]
if ([Text.Encoding]::ASCII.GetString($magic) -ne "MZ") { throw "Le fichier n'est pas un executable PE (MZ absent)." }

$dossier = "$($Lettre):\EFI\HWA3"
$cible = Join-Path $dossier $Nom
$hash = (Get-FileHash $src -Algorithm SHA256).Hash

Write-Output "Source      : $src ($taille octets, SHA-256 $hash)"
Write-Output "Destination : $cible"
Write-Output "Interdit    : EFI\BOOT, EFI\HWA, EFI\HWA2 (aucune ecriture), NVRAM, BIOS, disque interne"

if (Test-Path $cible) {
    if (-not $Ecraser) { throw "Le fichier existe deja : $cible (utiliser -Ecraser pour le remplacer)." }
    Write-Output "Remplacement explicite demande (-Ecraser)."
}

if (-not $Confirmer) {
    Write-Output "Mode essai : rien n'a ete ecrit. Ajouter -Confirmer pour deposer le fichier."
    return
}

New-Item -ItemType Directory -Force -Path $dossier | Out-Null
Copy-Item -Path $src -Destination $cible -Force:$Ecraser -ErrorAction Stop
$verif = (Get-FileHash $cible -Algorithm SHA256).Hash
if ($verif -ne $hash) { throw "Verification echouee : empreinte differente apres copie." }
Write-Output "Depose et verifie : $cible (SHA-256 $verif)"
Write-Output "Pour le lancer : menu de demarrage ponctuel du BIOS, puis EFI\HWA3\$Nom."
