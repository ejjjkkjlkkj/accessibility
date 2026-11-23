# Audit STATIQUE (lecture seule) de l'accessibilite lecteur d'ecran des environnements WinPE / WinRE / Setup : presence des composants necessaires
# a la lecture ET a la navigation par lecteur d'ecran (Narrator, pile audio, UI Automation, moteur vocal, donnees de voix francaise).
# Chaque image est montee en LECTURE SEULE dans un dossier de travail puis demontee avec /Discard. ASCII uniquement.
param([string]$Iso = "C:\Users\adm\Downloads\Windows11_Client_x64_fr-fr_26300_9457.iso", [string]$Out = "$env:TEMP\WINPE-VOXA\audit")
$ErrorActionPreference = 'Stop'
$dism = "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Deployment Tools\amd64\DISM\dism.exe"
New-Item -ItemType Directory -Force $Out | Out-Null
$mnt = "$Out\m"; New-Item -ItemType Directory -Force $mnt | Out-Null
$items = [ordered]@{
  'Narrator.exe' = 'Windows\System32\Narrator.exe'; 'Narrator fr-FR (MUI)' = 'Windows\System32\fr-FR\Narrator.exe.mui'
  'UIAutomationCore.dll' = 'Windows\System32\UIAutomationCore.dll'; 'audiosrv.dll' = 'Windows\System32\audiosrv.dll'
  'AudioEndpointBuilder.dll' = 'Windows\System32\AudioEndpointBuilder.dll'; 'audiodg.exe' = 'Windows\System32\audiodg.exe'
  'mmdevapi.dll' = 'Windows\System32\mmdevapi.dll'; 'hdaudbus.sys' = 'Windows\System32\drivers\hdaudbus.sys'
  'HdAudio.sys' = 'Windows\System32\drivers\HdAudio.sys'; 'sapi.dll' = 'Windows\System32\Speech\Common\sapi.dll'
  'MSTTSEngine.dll' = 'Windows\System32\Speech\Engines\TTS\MSTTSEngine.dll'; 'Speech_OneCore (voix)' = 'Windows\System32\Speech_OneCore'
  'beep.sys' = 'Windows\System32\drivers\beep.sys' }
function Inspect($label, $wim, $index) {
  $r = [ordered]@{ Image = $label }
  $mo = & $dism /Mount-Image /ImageFile:"$wim" /Index:$index /MountDir:$mnt /ReadOnly 2>&1
  if (-not (Test-Path "$mnt\Windows\System32")) { $r["Resultat"] = "MONTAGE ECHEC : " + (($mo | Select-Object -Last 3) -join " "); return [pscustomobject]$r }
  try {
    foreach ($k in $items.Keys) { $r[$k] = if (Test-Path "$mnt\$($items[$k])") { 'oui' } else { 'NON' } }
    # Donnees de la voix francaise : Windows\Speech\Engines\TTS\fr-FR (APM = voix, MSTTSLocFrFR.dat = lexique)
    $vf = "$mnt\Windows\Speech\Engines\TTS\fr-FR"
    $r['Voix francaise (APM)'] = if ((Test-Path $vf) -and @(Get-ChildItem -Path $vf -Filter *.APM -File -ErrorAction SilentlyContinue).Count -gt 0) { 'oui' } else { 'NON' }
    $r['Lexique fr (MSTTSLocFrFR.dat)'] = if (Test-Path "$vf\MSTTSLocFrFR.dat") { 'oui' } else { 'NON' }
    $r['sapisvr.exe'] = if (Test-Path "$mnt\Windows\Speech\Common\sapisvr.exe") { 'oui' } else { 'NON' }
  } finally { & $dism /Unmount-Image /MountDir:$mnt /Discard 2>&1 | Out-Null }
  [pscustomobject]$r
}
$res = @()
$res += Inspect 'WinPE ADK 26100 (winpe.wim)' "${env:ProgramFiles(x86)}\Windows Kits\10\Assessment and Deployment Kit\Windows Preinstallation Environment\amd64\en-us\winpe.wim" 1
$re = '\\?\GLOBALROOT\device\harddisk0\partition5\Recovery\WindowsRE\Winre.wim'   # lu directement par DISM, en lecture seule
$res += Inspect 'WinRE de la machine (26100.9444)' $re 1
$img = Mount-DiskImage -ImagePath $Iso -Access ReadOnly -PassThru
try {
  $dl = ($img | Get-Volume).DriveLetter; $bw = "${dl}:\sources\boot.wim"
  $res += Inspect 'ISO 26300 : boot.wim idx1 (WinPE)' $bw 1
  $res += Inspect 'ISO 26300 : boot.wim idx2 (Setup)' $bw 2
} finally { Dismount-DiskImage -ImagePath $Iso | Out-Null }
$res | Export-Csv "$Out\matrice.csv" -NoTypeInformation -Encoding UTF8
$res | Format-List | Out-String -Width 200
