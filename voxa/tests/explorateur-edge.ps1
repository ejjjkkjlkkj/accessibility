# EXPLORATEUR EDGE : ouvre chaque page INTERNE de Microsoft Edge (consultation) dans une NOUVELLE fenetre, mesure l'accessibilite (lecture + Tab seul), puis ferme UNIQUEMENT cette fenetre.
# SECURITE : aucune page web externe, aucune page d'historique/mots de passe/profil/reinitialisation, aucun clic ; Tab seul ; aucun titre ni nom de controle n'est ecrit. ASCII.
param([string]$Exe, [string]$Sortie = "$env:TEMP\WINPE-VOXA\audit\explorateur-edge.csv", [int]$MaxTab = 40)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$AE = [Windows.Automation.AutomationElement]
$edge = (Get-Command msedge.exe -ErrorAction SilentlyContinue).Source; if (-not $edge) { $edge = "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe" }
$pages = 'edge://settings/privacy','edge://settings/system','edge://settings/appearance','edge://settings/sidebar','edge://settings/startHomeNTP','edge://settings/languages','edge://settings/downloads','edge://settings/accessibility','edge://settings/help','edge://settings/printing','edge://settings/defaultBrowser','edge://settings/content','edge://settings/content/cookies','edge://settings/content/location','edge://settings/content/camera','edge://settings/content/microphone','edge://settings/content/notifications','edge://flags','edge://version','edge://accessibility','edge://gpu','edge://about'
function Fen { $h = @{}; foreach ($w in $AE::RootElement.FindAll('Children', [Windows.Automation.Condition]::TrueCondition)) { try { $id = ($w.GetRuntimeId() -join '.'); $h[$id] = $w } catch {} }; $h }
$g = { param($s, $k) if ($s -match "$k=(\d+)") { [int]$Matches[1] } else { -1 } }
$lignes = @()
foreach ($p in $pages) {
  $avant = Fen; Start-Process $edge -ArgumentList '--new-window', 'about:blank'
  $nw = $null; for ($i = 0; $i -lt 20 -and -not $nw; $i++) { Start-Sleep -Milliseconds 700; $ap = Fen; foreach ($k in $ap.Keys) { if (-not $avant.ContainsKey($k)) { try { if ((Get-Process -Id $ap[$k].Current.ProcessId).ProcessName -eq 'msedge' -and $ap[$k].Current.Name) { $nw = $ap[$k]; break } } catch {} } } }
  if (-not $nw) { $lignes += [pscustomobject]@{ page = $p; ouvert = $false }; continue }
  Start-Sleep -Seconds 2
  # Edge ignore les URL edge:// passees en ligne de commande (il ouvre Nouvel onglet) : on les saisit dans la barre d'adresse de CETTE fenetre (Ctrl+L, adresse, Entree).
  $titre0 = $nw.Current.Name; try { $nw.SetFocus() } catch {}; Start-Sleep -Milliseconds 500
  $sh = New-Object -ComObject WScript.Shell; $sh.SendKeys('^l'); Start-Sleep -Milliseconds 400; $sh.SendKeys($p); $sh.SendKeys('{ENTER}')
  $navigue = $false; for ($i = 0; $i -lt 15 -and -not $navigue; $i++) { Start-Sleep -Milliseconds 700; $tn = $nw.Current.Name; if ($tn -and $tn -ne $titre0 -and $tn -notmatch 'Nouvel onglet|New tab|about:blank') { $navigue = $true } }
  Start-Sleep -Seconds 4; $titre = $nw.Current.Name
  if (-not $navigue) { $lignes += [pscustomobject]@{ page = $p; ouvert = $true; navigue = $false }; try { $nw.GetCurrentPattern([Windows.Automation.WindowPattern]::Pattern).Close() } catch {}; continue }
  $t = (& $Exe --parcours-tab ("=" + $titre) --max $MaxTab) -join ' '; $a = (& $Exe --audit-processus "msedge.exe" --titre ("=" + $titre) | Where-Object { $_ -like 'AUDIT*' }) -join ' '
  $lignes += [pscustomobject]@{ page = $p; ouvert = $true; navigue = $true; attendus = (& $g $t 'attendus'); distincts = (& $g $t 'distincts'); arrets_sans_nom = (& $g $t 'sans_nom'); interactifs = (& $g $a 'interactifs'); muets_natifs = (& $g $a 'sans_nom_propre'); combles = (& $g $a 'combles_par_le_lecteur'); toujours_muets = (& $g $a 'toujours_sans_nom') }
  try { $nw.GetCurrentPattern([Windows.Automation.WindowPattern]::Pattern).Close() } catch {}; Start-Sleep -Seconds 2
}
$lignes | Export-Csv $Sortie -NoTypeInformation -Encoding UTF8
$ok = $lignes | Where-Object { $_.ouvert -and $_.navigue }
"PAGES_NON_ATTEINTES=$(@($lignes | Where-Object { $_.ouvert -and -not $_.navigue }).Count)"
"PAGES_EDGE_DEMANDEES=$($pages.Count) OUVERTES_ET_MESUREES=$($ok.Count)"
"CONTROLES=$(($ok | Measure-Object interactifs -Sum).Sum) MUETS_NATIFS=$(($ok | Measure-Object muets_natifs -Sum).Sum) COMBLES=$(($ok | Measure-Object combles -Sum).Sum) TOUJOURS_MUETS=$(($ok | Measure-Object toujours_muets -Sum).Sum) ARRETS_SANS_NOM=$(($ok | Measure-Object arrets_sans_nom -Sum).Sum)"
$cov = $ok | Where-Object { $_.attendus -gt 0 }; "Tab complet : $(($cov | Where-Object { $_.distincts -ge $_.attendus }).Count) / $($cov.Count)"
