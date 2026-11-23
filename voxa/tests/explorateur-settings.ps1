# EXPLORATEUR AUTOMATIQUE : utilise Windows a la place de l'utilisateur pour parcourir les pages de Parametres et mesurer leur accessibilite (lecture + navigation clavier).
# SECURITE : n'ouvre QUE des pages de consultation de la liste officielle ms-settings: de Microsoft (aucune URI se terminant par -action, aucun assistant, aucune inscription) ;
# n'envoie QUE la touche Tab ; ne clique rien, ne valide rien ; referme la fenetre Parametres a la fin. Aucun nom ni valeur de controle n'est ecrit. ASCII.
param([string]$Exe, [string]$Sortie = "$env:TEMP\WINPE-VOXA\audit\explorateur-settings.csv", [int]$MaxTab = 40)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$pages = @(
 'workplace','emailandaccounts','otherusers','signinoptions','sync','yourinfo',
 'appsfeatures','appsforwebsites','defaultapps','optionalfeatures','maps','startupapps','videoplayback',
 'controlcenter','autoplay','bluetooth','connecteddevices','camera','mousetouchpad','pen','printers','devices-touch','devices-touchpad','typing','usb',
 'easeofaccess-audio','easeofaccess-closedcaptioning','easeofaccess-colorfilter','easeofaccess-display','easeofaccess-eyecontrol','easeofaccess-hearingaids','easeofaccess-highcontrast','easeofaccess-keyboard','easeofaccess-magnifier','easeofaccess-mouse','easeofaccess-mousepointer','easeofaccess-narrator','easeofaccess-speechrecognition','easeofaccess-cursor','easeofaccess-visualeffects',
 'gaming-gamebar','gaming-gamemode',
 'network-status','network-advancedsettings','network-airplanemode','network-ethernet','network-wifisettings','network-mobilehotspot','network-proxy','network-vpn','network-wifi',
 'personalization','personalization-background','personalization-start','personalization-colors','personalization-textinput','personalization-touchkeyboard','themes','fonts','lockscreen','taskbar',
 'privacy','privacy-microphone','privacy-webcam','privacy-location','privacy-speech','privacy-speechtyping','privacy-notifications','privacy-activityhistory','privacy-feedback','privacy-appdiagnostics',
 'search','search-permissions','sound','sound-devices','apps-volume',
 'about','display','batterysaver','clipboard','deviceencryption','multitasking','nightlight','project','crossdevice','notifications','remotedesktop','powersleep','storagesense','storagepolicies','disksandvolumes','savelocations',
 'dateandtime','regionformatting','keyboard','regionlanguage','speech',
 'activation','delivery-optimization','findmydevice','developers','recovery','troubleshoot','windowsupdate','windowsupdate-options','windowsupdate-history','windowsupdate-optionalupdates')
# Complement : autres pages de CONSULTATION de la liste officielle Microsoft (learn.microsoft.com, "Launch Windows Settings"), hors pages d'assistant/inscription/action.
$suppl = Join-Path $PSScriptRoot '..\data-ms-settings-supplement.txt'
if (Test-Path $suppl) { $pages = @($pages + ((Get-Content $suppl -Raw) -split '\s+' | Where-Object { $_ })) | Select-Object -Unique }
$titreS = '=Param' + [char]0xE8 + 'tres'
$deja = @(Get-Process SystemSettings -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
$lignes = @(); $n = 0
foreach ($p in $pages) {
  $n++; Start-Process "ms-settings:$p"; Start-Sleep -Milliseconds 3500
  $t = (& $Exe --parcours-tab $titreS --max $MaxTab) -join ' '
  $a = (& $Exe --audit-processus '*' --titre $titreS) -join ' '
  $g = { param($s, $k) if ($s -match "$k=(\d+)") { [int]$Matches[1] } else { -1 } }
  $lignes += [pscustomobject]@{ page = $p; trouvee = ($t -notmatch 'introuvable'); attendus = (& $g $t 'attendus'); arrets = (& $g $t 'arrets'); distincts = (& $g $t 'distincts'); cycle = (& $g $t 'cycle'); arrets_sans_nom = (& $g $t 'sans_nom'); hors_fenetre = (& $g $t 'hors_fenetre')
    interactifs = (& $g $a 'interactifs'); muets_natifs = (& $g $a 'sans_nom_propre'); combles = (& $g $a 'combles_par_le_lecteur'); toujours_muets = (& $g $a 'toujours_sans_nom') }
}
$lignes | Export-Csv $Sortie -NoTypeInformation -Encoding UTF8
foreach ($w in [Windows.Automation.AutomationElement]::RootElement.FindAll('Children', [Windows.Automation.Condition]::TrueCondition)) {
  try { $pn = (Get-Process -Id $w.Current.ProcessId -ErrorAction Stop).ProcessName } catch { continue }
  if ($pn -eq 'ApplicationFrameHost' -and $w.Current.Name -like '*Param*') { try { $w.GetCurrentPattern([Windows.Automation.WindowPattern]::Pattern).Close() } catch {} }
}
Start-Sleep 2; Get-Process SystemSettings -ErrorAction SilentlyContinue | Where-Object { $deja -notcontains $_.Id } | Stop-Process -ErrorAction SilentlyContinue
$ok = $lignes | Where-Object { $_.trouvee -and $_.interactifs -ge 0 }
"PAGES_DEMANDEES=$($pages.Count) PAGES_MESUREES=$($ok.Count) PAGES_NON_TROUVEES=$(($lignes | Where-Object { -not $_.trouvee }).Count)"
"CONTROLES_INTERACTIFS=$(($ok | Measure-Object interactifs -Sum).Sum) MUETS_NATIFS=$(($ok | Measure-Object muets_natifs -Sum).Sum) COMBLES=$(($ok | Measure-Object combles -Sum).Sum) TOUJOURS_MUETS=$(($ok | Measure-Object toujours_muets -Sum).Sum)"
$cov = $ok | Where-Object { $_.attendus -gt 0 }; "PAGES_AVEC_CONTROLES_FOCALISABLES=$($cov.Count) ; couverture Tab complete (distincts >= attendus) : $(($cov | Where-Object { $_.distincts -ge $_.attendus }).Count) ; couverture partielle : $(($cov | Where-Object { $_.distincts -lt $_.attendus }).Count) ; arrets sans nom : $(($cov | Measure-Object arrets_sans_nom -Sum).Sum)"
