# INVENTAIRE TOTAL (lecture seule) : recense TOUTES les applications installees (Windows + tierces), deduit leur technologie d'interface,
# rassemble la documentation locale disponible (aide, lisez-moi, liens editeur), et audite l'accessibilite des fenetres DEJA OUVERTES (UI Automation, aucune touche, aucun clic).
# Ne lance aucune application, n'ecrit AUCUN titre de fenetre ni nom/valeur de controle (seulement nom de processus et compteurs). ASCII.
param([string]$Exe, [string]$Sortie = "$env:TEMP\WINPE-VOXA\audit\inventaire-total")
$ErrorActionPreference = 'SilentlyContinue'; New-Item -ItemType Directory -Force $Sortie | Out-Null
# --- 1. Applications installees (registre + raccourcis du menu Demarrer)
$cles = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*', 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*', 'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*'
$apps = @{}
foreach ($k in $cles) { foreach ($e in Get-ItemProperty $k) { if ($e.DisplayName -and -not $e.SystemComponent) { $apps[$e.DisplayName] = [pscustomobject]@{ nom = $e.DisplayName; editeur = $e.Publisher; version = $e.DisplayVersion; dossier = $e.InstallLocation; aide = $e.HelpLink; info = $e.URLInfoAbout } } } }
$sh = New-Object -ComObject WScript.Shell; $menu = @("$env:ProgramData\Microsoft\Windows\Start Menu\Programs", "$env:APPDATA\Microsoft\Windows\Start Menu\Programs")
foreach ($l in Get-ChildItem $menu -Recurse -Filter *.lnk) { $t = $sh.CreateShortcut($l.FullName).TargetPath; if ($t -like '*.exe' -and (Test-Path $t)) { $n = $l.BaseName; if (-not $apps.ContainsKey($n)) { $apps[$n] = [pscustomobject]@{ nom = $n; editeur = ''; version = ''; dossier = (Split-Path $t); aide = ''; info = '' } }; $apps[$n] | Add-Member -NotePropertyName exe -NotePropertyValue $t -Force } }
function Techno($dir) {
  if (-not $dir -or -not (Test-Path $dir)) { return 'inconnue' }
  $f = @(Get-ChildItem $dir -Recurse -Depth 2 -File -Include *.dll, *.exe, *.jar, *.pak, *.asar -ErrorAction SilentlyContinue | Select-Object -First 4000 | ForEach-Object { $_.Name.ToLower() })
  $r = @()
  if ($f -match '^qt[56]?core') { $r += 'Qt' }; if ($f -contains 'chrome_elf.dll' -or $f -contains 'resources.pak' -or $f -contains 'app.asar') { $r += 'Chromium/Electron' }
  if ($f -contains 'jvm.dll' -or ($f -match '\.jar$')) { $r += 'Java' }; if ($f -contains 'presentationframework.dll') { $r += 'WPF' }
  if ($f -match '^microsoft\.ui\.xaml') { $r += 'WinUI' }; if ($f -match 'libgtk|gtk-3') { $r += 'GTK' }; if ($f -match 'wxmsw') { $r += 'wxWidgets' }
  if ($f -match 'system\.windows\.forms\.dll') { $r += 'WinForms' }; if ($f -match 'vcl\d|rtl\d{2,3}\.bpl') { $r += 'Delphi-VCL' }; if ($f -match 'libcef|cef\.dll') { $r += 'CEF' }
  if ($r.Count -eq 0) { 'Win32/natif' } else { $r -join '+' } }
$risque = @{ 'Qt' = 'moyen : accessibilite UIA/MSAA presente si le plugin accessible est charge'; 'Chromium/Electron' = 'faible a moyen : arbre expose si le lecteur est detecte (option --force-renderer-accessibility)'; 'Java' = 'eleve : necessite Java Access Bridge'; 'WPF' = 'faible : UIA natif, trous si controles sans AutomationProperties.Name'; 'WinUI' = 'faible : UIA natif'; 'WinForms' = 'faible a moyen : MSAA/UIA, trous sur controles dessines'; 'Delphi-VCL' = 'moyen a eleve : controles dessines'; 'GTK' = 'eleve : pas d UIA natif sous Windows'; 'wxWidgets' = 'faible : controles natifs'; 'CEF' = 'moyen : comme Chromium' ; 'Win32/natif' = 'faible : controles standards' }
$docs = foreach ($a in $apps.Values) {
  $d = $a.dossier; if (-not $d -and $a.exe) { $d = Split-Path $a.exe }
  $t = Techno $d
  $loc = @(); if ($d -and (Test-Path $d)) { $loc = @(Get-ChildItem $d -Recurse -Depth 2 -File -Include *.chm, *.hlp, readme*, lisezmoi*, *.pdf, manual* -ErrorAction SilentlyContinue | Select-Object -First 8 | ForEach-Object { $_.FullName.Substring($d.Length).TrimStart('\') }) }
  $rk = ($t -split '\+' | ForEach-Object { $risque[$_] }) -join ' ; '
  [pscustomobject]@{ nom = $a.nom; editeur = $a.editeur; version = $a.version; technologie = $t; risque_accessibilite = $rk; docs_locales = ($loc -join ' | '); aide_en_ligne = $a.aide; info_editeur = $a.info }
}
$docs | Sort-Object nom | Export-Csv "$Sortie\applications-docs.csv" -NoTypeInformation -Encoding UTF8
# --- 2. Audit des fenetres deja ouvertes (lecture seule)
$procs = Get-Process | Where-Object { $_.MainWindowHandle -ne 0 -and $_.ProcessName -ne 'voxa-companion' } | Select-Object -ExpandProperty ProcessName -Unique
$g = { param($s, $k) if ($s -match "$k=(\d+)") { [int]$Matches[1] } else { -1 } }
$aud = foreach ($p in $procs) { $a = (& $Exe --audit-processus "$p.exe" | Where-Object { $_ -like 'AUDIT*' }) -join ' '
  [pscustomobject]@{ processus = $p; interactifs = (& $g $a 'interactifs'); muets_natifs = (& $g $a 'sans_nom_propre'); combles = (& $g $a 'combles_par_le_lecteur'); toujours_muets = (& $g $a 'toujours_sans_nom') } }
$aud | Export-Csv "$Sortie\fenetres-ouvertes.csv" -NoTypeInformation -Encoding UTF8
# --- 3. Documentation de reference (liens officiels, a consulter hors ligne si besoin)
@'
UI Automation (Windows) : https://learn.microsoft.com/windows/win32/winauto/entry-uiauto-win32
MSAA : https://learn.microsoft.com/windows/win32/winauto/microsoft-active-accessibility
Narrator : https://support.microsoft.com/windows/complete-guide-to-narrator-e4397a0d-ef4f-b386-d8ae-c172f109bdb1
Qt : https://doc.qt.io/qt-6/accessible.html
Electron : https://www.electronjs.org/docs/latest/tutorial/accessibility
Java Access Bridge : https://docs.oracle.com/javase/accessibility/
WPF : https://learn.microsoft.com/dotnet/desktop/wpf/advanced/accessibility
WinUI : https://learn.microsoft.com/windows/apps/design/accessibility/accessibility-overview
Normes : https://www.w3.org/TR/WCAG22/
'@ | Set-Content "$Sortie\references-officielles.txt" -Encoding ASCII
$v = $aud | Where-Object { $_.interactifs -ge 0 }
"APPLICATIONS_INVENTORIEES=$($docs.Count) AVEC_DOCS_LOCALES=$(($docs | Where-Object { $_.docs_locales }).Count) ; TECHNOLOGIES : " + (($docs | Group-Object technologie | Sort-Object Count -Descending | Select-Object -First 8 | ForEach-Object { "$($_.Name)=$($_.Count)" }) -join ', ')
"FENETRES_OUVERTES_AUDITEES=$($v.Count) CONTROLES=$(($v | Measure-Object interactifs -Sum).Sum) MUETS_NATIFS=$(($v | Measure-Object muets_natifs -Sum).Sum) TOUJOURS_MUETS=$(($v | Measure-Object toujours_muets -Sum).Sum)"
"A RISQUE (muets>0) : " + (($v | Where-Object { $_.toujours_muets -gt 0 } | ForEach-Object { "$($_.processus)=$($_.toujours_muets)" }) -join ', ')
"Sortie : $Sortie"
