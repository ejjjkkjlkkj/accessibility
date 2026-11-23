# Verifie la NAVIGATION AU CLAVIER (Tab seulement) de fenetres Windows integrees : arrets, cycle, arrets sans nom, couverture des controles focalisables.
# Ouvre Calculatrice et Parametres (le Bloc-notes est exclu : Tab y insere une tabulation dans le texte, ce n'est pas une navigation), mesure, puis referme UNIQUEMENT ce qu'il a ouvert. ASCII.
param([string]$Exe)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$avant = @(Get-Process CalculatorApp, SystemSettings -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
Start-Process calc.exe; Start-Process 'ms-settings:'; Start-Sleep -Seconds 9
try { foreach ($t in 'Calcul', 'Param') { & $Exe --parcours-tab $t --max 80 } }
finally {
  foreach ($w in [Windows.Automation.AutomationElement]::RootElement.FindAll('Children', [Windows.Automation.Condition]::TrueCondition)) {
    try { $pn = (Get-Process -Id $w.Current.ProcessId -ErrorAction Stop).ProcessName } catch { continue }
    if ($pn -eq 'ApplicationFrameHost') { try { $w.GetCurrentPattern([Windows.Automation.WindowPattern]::Pattern).Close() } catch {} }
  }
  Start-Sleep 2; Get-Process CalculatorApp, SystemSettings -ErrorAction SilentlyContinue | Where-Object { $avant -notcontains $_.Id } | Stop-Process -ErrorAction SilentlyContinue
  Start-Sleep 1; "applications restantes ouvertes par ce test : " + ((Get-Process CalculatorApp, SystemSettings -ErrorAction SilentlyContinue | Where-Object { $avant -notcontains $_.Id } | Select-Object -ExpandProperty ProcessName) -join ',')
}
