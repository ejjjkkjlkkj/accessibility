# Verifie le COMBLAGE des noms manquants par voxa-companion sur un specimen REEL : une fenetre WPF avec un texte puis un bouton SANS nom accessible
# (WPF ne reconstitue pas de nom : c'est le cas typique des applications mal etiquetees). Reference : .NET UIAutomation confirme que le Name natif est vide.
# Attendu : le lecteur de secours nomme le bouton d'apres le texte qui le precede (source "voisin"). ASCII uniquement.
param([string]$Exe)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$code = @'
Add-Type -AssemblyName PresentationFramework
$w = New-Object Windows.Window; $w.Title = 'VoxaTestComblage'; $w.Width = 360; $w.Height = 160; $w.WindowStartupLocation = 'CenterScreen'
$sp = New-Object Windows.Controls.StackPanel
$t = New-Object Windows.Controls.TextBlock; $t.Text = 'Parametre important'; $t.Margin = 10
$b = New-Object Windows.Controls.Button; $b.Width = 120; $b.Height = 30; $b.HorizontalAlignment = 'Left'; $b.Margin = 10
[void]$sp.Children.Add($t); [void]$sp.Children.Add($b); $w.Content = $sp
$timer = New-Object Windows.Threading.DispatcherTimer; $timer.Interval = [TimeSpan]::FromSeconds(30); $timer.Add_Tick({ $w.Close() }); $timer.Start()
[void]$w.ShowDialog()
'@
$p = Start-Process powershell.exe -ArgumentList '-NoProfile', '-STA', '-Command', $code -PassThru
try {
  $w = $null
  for ($i = 0; $i -lt 20 -and $w -eq $null; $i++) { Start-Sleep -Milliseconds 500; $w = [Windows.Automation.AutomationElement]::RootElement.FindFirst('Children', (New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, 'VoxaTestComblage'))) }
  if ($w -eq $null) { "[FAIL] fenetre specimen introuvable"; return }
  Start-Sleep -Seconds 1
  $btns = $w.FindAll('Descendants', (New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::Button)))
  $b2 = $btns | Where-Object { [string]::IsNullOrWhiteSpace($_.Current.Name) -and $_.Current.IsKeyboardFocusable } | Select-Object -First 1
  if ($b2 -eq $null) { "[FAIL] bouton sans nom introuvable ; boutons vus : " + (($btns | ForEach-Object { '[' + $_.Current.Name + '|focus=' + $_.Current.IsKeyboardFocusable + ']' }) -join ' '); return }
  "[PASS] specimen reel : bouton SANS nom natif confirme par .NET UIAutomation"
  $b2.SetFocus(); Start-Sleep -Milliseconds 800
  $f = Join-Path $env:TEMP ("comblage-" + [guid]::NewGuid().ToString('N') + ".txt"); cmd.exe /c "`"$Exe`" --once > `"$f`" 2>&1" | Out-Null
  $txt = [IO.File]::ReadAllText($f, [Text.Encoding]::UTF8); Remove-Item -LiteralPath $f -ErrorAction SilentlyContinue
  if ($txt -match 'NOMSOURCE=voisin' -and $txt -match 'NOM=Parametre important') { "[PASS] voxa-companion nomme le bouton muet d'apres le texte voisin : 'Parametre important'" } else { "[FAIL] comblage : " + ($txt -replace "`r?`n", ' | ') }
} finally { Stop-Process -Id $p.Id -ErrorAction SilentlyContinue }
