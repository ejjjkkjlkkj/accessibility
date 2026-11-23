# Cartographie d'accessibilite en LECTURE SEULE : fenetres actives (UI Automation) + boites de dialogue classiques de Windows actif, de WinRE et de l'ISO Windows.
# Les images sont montees en lecture seule (DISM /ReadOnly) puis demontees (/Discard) ; l'ISO est demonte a la fin. Aucun nom/valeur de controle n'est ecrit. ASCII.
param([string]$Exe, [string]$Iso = "C:\Users\adm\Downloads\Windows11_Client_x64_fr-fr_26300_9457.iso", [string]$Out = "$env:TEMP\WINPE-VOXA\audit")
$dism = "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\DISM\dism.exe"
New-Item -ItemType Directory -Force $Out | Out-Null; $m = "$Out\m4"; New-Item -ItemType Directory -Force $m | Out-Null
"== FENETRES ACTIVES"; & $Exe fenetres --sortie "$Out\cartographie-fenetres.jsonl"
"== DIALOGUES : Windows actif (System32)"; & $Exe dialogues "C:\Windows\System32" --csv "$Out\dialogues-windows-actif.csv"
function Carto($etiquette, $wim, $idx, $csv) {
  & $dism /Mount-Image /ImageFile:$wim /Index:$idx /MountDir:$m /ReadOnly 2>&1 | Out-Null
  if (Test-Path "$m\Windows\System32") { try { "== DIALOGUES : $etiquette"; & $Exe dialogues "$m\Windows" --csv "$Out\$csv"; "== DIALOGUES : $etiquette (dossier sources)"; & $Exe dialogues "$m\sources" --csv "$Out\$csv.sources.csv" 2>$null } finally { & $dism /Unmount-Image /MountDir:$m /Discard 2>&1 | Out-Null } } else { "== $etiquette : MONTAGE ECHEC" }
}
Carto 'WinRE de la machine' '\\?\GLOBALROOT\device\harddisk0\partition5\Recovery\WindowsRE\Winre.wim' 1 'dialogues-winre.csv'
$img = Mount-DiskImage -ImagePath $Iso -Access ReadOnly -PassThru
try { $dl = ($img | Get-Volume).DriveLetter; Carto 'ISO : boot.wim idx2 (Windows Setup)' "${dl}:\sources\boot.wim" 2 'dialogues-iso-setup.csv'; "== DIALOGUES : ISO, dossier sources du media"; & $Exe dialogues "${dl}:\sources" --csv "$Out\dialogues-iso-sources.csv" }
finally { Dismount-DiskImage -ImagePath $Iso | Out-Null }
"ISO monte ? " + (Get-DiskImage -ImagePath $Iso).Attached
