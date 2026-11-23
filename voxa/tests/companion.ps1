# Verification de voxa-companion contre la reference de Microsoft : le meme element (celui qui a le focus) est lu par le .NET UIAutomationClient et par voxa-companion
# (COM brut, indices verifies dans UIAutomationClient.h). On ouvre le Bloc-notes, on lui donne le focus, puis on le referme. ASCII uniquement.
param([string]$Exe)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
Add-Type -Namespace W -Name U -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(System.IntPtr h);'
$p = Start-Process notepad.exe -PassThru; Start-Sleep -Seconds 3
try {
  # Sous Windows 11 le Bloc-notes est une application empaquetee : le processus lance peut ne pas etre celui de la fenetre ; on cherche par nom de processus.
  $w = $null
  foreach ($c in [Windows.Automation.AutomationElement]::RootElement.FindAll('Children', [Windows.Automation.Condition]::TrueCondition)) {
    try { if ((Get-Process -Id $c.Current.ProcessId -ErrorAction Stop).ProcessName -eq 'Notepad') { $w = $c; break } } catch {}
  }
  if ($w -eq $null) { "[FAIL] fenetre du Bloc-notes introuvable"; return }
  [void][W.U]::SetForegroundWindow([IntPtr]$w.Current.NativeWindowHandle)
  $ed = $w.FindFirst('Descendants', (New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::Document)))
  if ($ed -eq $null) { $ed = $w.FindFirst('Descendants', (New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::Edit))) }
  if ($ed -ne $null) { try { $ed.SetFocus() } catch {} }; Start-Sleep -Seconds 1
  $ref = [Windows.Automation.AutomationElement]::FocusedElement
  $refNom = $ref.Current.Name; $refType = $ref.Current.LocalizedControlType
  # la sortie du programme est en UTF-8 : on la lit depuis un fichier pour ne pas subir le code de page de la console
  $f = Join-Path $env:TEMP ("companion-" + [guid]::NewGuid().ToString('N') + ".txt"); cmd.exe /c "`"$Exe`" --once > `"$f`" 2>&1" | Out-Null
  $sortie = [IO.File]::ReadAllText($f, [Text.Encoding]::UTF8) -split "`r?`n"; Remove-Item -LiteralPath $f -ErrorAction SilentlyContinue; $txt = ($sortie -join "`n")
  $nom = ($sortie | Where-Object { $_ -like 'NOM=*' } | Select-Object -First 1) -replace '^NOM=', ''
  $type = ($sortie | Where-Object { $_ -like 'TYPE=*' } | Select-Object -First 1) -replace '^TYPE=', ''
  if ($nom -eq $refNom -and $type -eq $refType) { "[PASS] voxa-companion lit le meme element focalise que .NET UIAutomation (type '$type', nom de {0} caracteres)" -f $nom.Length } else { "[FAIL] difference : reference type='$refType' nom='$refNom' ; companion type='$type' nom='$nom'" }
  if ($txt -match 'MOTDEPASSE=False') { "[PASS] voxa-companion indique correctement un champ qui n'est pas un mot de passe" } else { "[FAIL] indicateur de mot de passe : $txt" }
} finally {
  try { $wp = $w.GetCurrentPattern([Windows.Automation.WindowPattern]::Pattern); $wp.Close() } catch {}
  Start-Sleep -Milliseconds 500; Get-Process Notepad -ErrorAction SilentlyContinue | Stop-Process -ErrorAction SilentlyContinue
}
