# Fabrique une COPIE PERSONNELLE de l'ISO Windows avec des ajouts d'accessibilite PRIS EN CHARGE par Microsoft, sans toucher aux fichiers de Windows :
#   - autounattend.xml a la racine (passe windowsPE : francais, clavier AZERTY, lance Narrator puis le lecteur de secours) ;
#   - \voxa\ (voxa-companion.exe + notice) ;
#   - \$WinPEDriver$\ : pilotes de CETTE machine (audio Realtek, AMD ACP, Wi-Fi MediaTek, I2C/GPIO AMD) exportes depuis le magasin de pilotes ; Setup les charge seul.
# L'ISO d'origine n'est ni modifiee ni ecrasee ; boot.wim, install.wim et tous les fichiers de Windows restent identiques (verifie par somme SHA-256 apres reconstruction).
# Cette copie est reservee a l'usage personnel de la machine : elle ne doit pas etre redistribuee. ASCII.
param([string]$Iso = "C:\Users\adm\Downloads\Windows11_Client_x64_fr-fr_26300_9457.iso", [string]$Kit, [string]$Out = "$env:TEMP\WINPE-VOXA\iso-accessible", [string]$Companion)
$ErrorActionPreference = 'Stop'
$oscdimg = "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\Oscdimg\oscdimg.exe"
$media = "$Out\media"; $sortie = "$Out\Windows11_26300_fr-fr_VOXA-ACCESSIBLE-USAGE-PERSONNEL.iso"
if (Test-Path $media) { Remove-Item -LiteralPath $media -Recurse -Force }; New-Item -ItemType Directory -Force $media | Out-Null
"== 1. Copie des fichiers de l'ISO (lecture seule)"
$img = Mount-DiskImage -ImagePath $Iso -Access ReadOnly -PassThru
try { $dl = ($img | Get-Volume).DriveLetter; $label = ($img | Get-Volume).FileSystemLabel; robocopy "${dl}:\" $media /E /COPY:DAT /R:1 /W:1 /NFL /NDL /NJH /NJS /NP | Out-Null } finally { Dismount-DiskImage -ImagePath $Iso | Out-Null }
attrib -r "$media\*" /s /d | Out-Null
"etiquette du volume : $label"
"== 2. Ajouts"
Copy-Item (Join-Path $Kit 'autounattend.xml') "$media\autounattend.xml"
New-Item -ItemType Directory -Force "$media\voxa" | Out-Null
Copy-Item $Companion "$media\voxa\voxa-companion.exe"; Copy-Item (Join-Path $Kit 'LISEZMOI-KIT.txt') "$media\voxa\LISEZMOI-KIT.txt"
$pd = Join-Path $media '$WinPEDriver$'; New-Item -ItemType Directory -Force $pd | Out-Null
foreach ($d in @(@('oem8.inf', 'audio-realtek'), @('oem37.inf', 'audio-amd-acp'), @('oem26.inf', 'wifi-mediatek'), @('oem34.inf', 'amd-i2c'), @('oem33.inf', 'amd-gpio'))) {
  $dest = Join-Path $pd $d[1]; New-Item -ItemType Directory -Force $dest | Out-Null; pnputil /export-driver $d[0] $dest 2>&1 | Out-Null
  "pilote $($d[1]) : " + (@(Get-ChildItem $dest -Recurse -File).Count) + " fichiers"
}
"== 3. Reconstruction de l'ISO (oscdimg, amorcage BIOS + UEFI comme l'original)"
if (Test-Path $sortie) { Remove-Item -LiteralPath $sortie -Force }
$boot = "2#p0,e,b$media\boot\etfsboot.com#pEF,e,b$media\efi\microsoft\boot\efisys.bin"
$ErrorActionPreference = "Continue"; & $oscdimg -m -o -u2 -udfver102 "-l$label" "-bootdata:$boot" $media $sortie 2>&1 | Select-Object -Last 4; if ($LASTEXITCODE -ne 0) { "OSCDIMG ECHEC code $LASTEXITCODE" }
"ISO : " + $sortie + " (" + (Get-Item $sortie).Length + " octets)"
