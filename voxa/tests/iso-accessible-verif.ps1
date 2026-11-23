# Verifie la copie personnelle de l'ISO Windows face a l'original : (1) seuls des AJOUTS (autounattend.xml, \voxa, \$WinPEDriver$) ; (2) tous les fichiers d'origine sont identiques
# (taille + SHA-256 pour les fichiers de demarrage et les images .wim/.esd) ; (3) l'amorcage BIOS+UEFI est present. Les deux ISO sont montes en lecture seule puis demontes. ASCII.
param([string]$Original = "C:\Users\adm\Downloads\Windows11_Client_x64_fr-fr_26300_9457.iso", [string]$Copie = "$env:TEMP\WINPE-VOXA\iso-accessible\Windows11_26300_fr-fr_VOXA-ACCESSIBLE-USAGE-PERSONNEL.iso")
$ErrorActionPreference = 'Stop'
function Ok($c, $m) { if ($c) { "[PASS] $m" } else { "[FAIL] $m" } }
$a = Mount-DiskImage -ImagePath $Original -Access ReadOnly -PassThru; $b = Mount-DiskImage -ImagePath $Copie -Access ReadOnly -PassThru
try {
  $da = ($a | Get-Volume).DriveLetter; $db = ($b | Get-Volume).DriveLetter
  $fa = Get-ChildItem "${da}:\" -Recurse -File -Force | ForEach-Object { [pscustomobject]@{ R = $_.FullName.Substring(3).ToLower(); L = $_.Length; F = $_.FullName } }
  $fb = Get-ChildItem "${db}:\" -Recurse -File -Force | ForEach-Object { [pscustomobject]@{ R = $_.FullName.Substring(3).ToLower(); L = $_.Length; F = $_.FullName } }
  $ha = @{}; $fa | ForEach-Object { $ha[$_.R] = $_ }; $hb = @{}; $fb | ForEach-Object { $hb[$_.R] = $_ }
  $manquants = $fa | Where-Object { -not $hb.ContainsKey($_.R) }; $ajoutes = $fb | Where-Object { -not $ha.ContainsKey($_.R) }
  $tailles = $fa | Where-Object { $hb.ContainsKey($_.R) -and $hb[$_.R].L -ne $_.L }
  Ok ($manquants.Count -eq 0) "aucun fichier d'origine manquant ($($fa.Count) fichiers dans l'original)"
  Ok ($tailles.Count -eq 0) "aucune taille modifiee parmi les fichiers d'origine"
  $horsAjouts = $ajoutes | Where-Object { $_.R -notlike 'voxa\*' -and $_.R -notlike '$winpedriver$\*' -and $_.R -ne 'autounattend.xml' }
  Ok ($horsAjouts.Count -eq 0) "seuls des ajouts attendus : $($ajoutes.Count) fichiers (autounattend.xml, voxa, `$WinPEDriver`$)"
  $cles = $fa | Where-Object { $_.R -match '\.(wim|esd)$' -or $_.R -in @('bootmgr', 'bootmgr.efi', 'boot\bcd', 'efi\boot\bootx64.efi', 'efi\microsoft\boot\bcd', 'boot\etfsboot.com', 'efi\microsoft\boot\efisys.bin', 'setup.exe') }
  $diff = @(); foreach ($f in $cles) { if ((Get-FileHash $f.F -Algorithm SHA256).Hash -ne (Get-FileHash $hb[$f.R].F -Algorithm SHA256).Hash) { $diff += $f.R } }
  Ok ($diff.Count -eq 0) "SHA-256 identiques pour les $($cles.Count) fichiers cles (images .wim/.esd, gestionnaires de demarrage, setup.exe)"
  Ok (Test-Path "${db}:\autounattend.xml") "autounattend.xml present a la racine"
  Ok (Test-Path "${db}:\voxa\voxa-companion.exe") "voxa-companion.exe present"
  $pd = Get-ChildItem "${db}:\`$WinPEDriver`$" -Recurse -Filter *.inf -ErrorAction SilentlyContinue; Ok ($pd.Count -ge 5) "pilotes de la machine dans `$WinPEDriver`$ : $($pd.Count) fichiers .inf"
} finally { Dismount-DiskImage -ImagePath $Original | Out-Null; Dismount-DiskImage -ImagePath $Copie | Out-Null }
"ISO montes ? " + ((Get-DiskImage -ImagePath $Original).Attached -or (Get-DiskImage -ImagePath $Copie).Attached)
