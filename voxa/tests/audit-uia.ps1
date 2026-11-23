# Audit MESURE (lecture seule) de l'accessibilite des fenetres Windows ouvertes via UI Automation, l'interface que les lecteurs d'ecran (Narrator, NVDA, JAWS) lisent.
# Pour chaque fenetre de premier niveau : on parcourt l'arbre (ControlView, borne), et on compte parmi les controles INTERACTIFS visibles et actives :
#   - sans nom accessible (ni Name ni HelpText ni LabeledBy) : un lecteur d'ecran dit seulement "bouton" ;
#   - non atteignables au clavier (IsKeyboardFocusable faux) : probleme de navigation.
# Limites : ne prouve pas que la lecture soit CORRECTE ou que l'ordre de navigation soit logique ; mesure seulement deux conditions necessaires. ASCII uniquement.
param([int]$MaxParFenetre = 600, [int]$SecParFenetre = 25, [string]$Sortie = "$env:TEMP\WINPE-VOXA\audit\uia.csv", [string[]]$Ignorer = @('Claude', 'ChatGPT', 'msedge', 'Program Manager'))
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$AE = [Windows.Automation.AutomationElement]; $CT = [Windows.Automation.ControlType]
$interactifs = @($CT::Button, $CT::CheckBox, $CT::ComboBox, $CT::Edit, $CT::Hyperlink, $CT::ListItem, $CT::MenuItem, $CT::RadioButton, $CT::Slider, $CT::Spinner, $CT::TabItem, $CT::TreeItem, $CT::SplitButton, $CT::DataItem) | ForEach-Object { $_.Id }
$walker = [Windows.Automation.TreeWalker]::ControlViewWalker
$wins = $AE::RootElement.FindAll([Windows.Automation.TreeScope]::Children, [Windows.Automation.Condition]::TrueCondition)
$rows = @()
foreach ($w in $wins) {
  try { $nom = $w.Current.Name; $proc = (Get-Process -Id $w.Current.ProcessId -ErrorAction SilentlyContinue).ProcessName } catch { continue }
  if ([string]::IsNullOrWhiteSpace($nom) -or ($Ignorer | Where-Object { $nom -like "*$_*" -or $proc -like "*$_*" })) { continue }
  $n = 0; $inter = 0; $sansNom = 0; $nonFoc = 0; $stop = [DateTime]::Now.AddSeconds($SecParFenetre)
  $pile = New-Object System.Collections.Stack; $pile.Push($w)
  while ($pile.Count -gt 0 -and $n -lt $MaxParFenetre -and [DateTime]::Now -lt $stop) {
    $e = $pile.Pop(); $n++
    try {
      $c = $e.Current
      if ($interactifs -contains $c.ControlType.Id -and $c.IsEnabled -and -not $c.IsOffscreen) {
        $inter++
        $nommee = (-not [string]::IsNullOrWhiteSpace($c.Name)) -or (-not [string]::IsNullOrWhiteSpace($c.HelpText)) -or ($c.LabeledBy -ne $null)
        if (-not $nommee) { $sansNom++ }
        if (-not $c.IsKeyboardFocusable) { $nonFoc++ }
      }
      $ch = $walker.GetFirstChild($e); $tmp = @()
      while ($ch -ne $null -and $tmp.Count -lt 400) { $tmp += $ch; $ch = $walker.GetNextSibling($ch) }
      [array]::Reverse($tmp); foreach ($x in $tmp) { $pile.Push($x) }
    } catch { }
  }
  $rows += [pscustomobject]@{ Fenetre = "(titre masque)"; Processus = $proc; Elements = $n; Interactifs = $inter; SansNom = $sansNom; NonFocalisables = $nonFoc; Tronque = ($pile.Count -gt 0) }
}
$rows | Export-Csv $Sortie -NoTypeInformation -Encoding UTF8
$rows | Format-Table -AutoSize | Out-String -Width 220
$I = ($rows | Measure-Object Interactifs -Sum).Sum; $S = ($rows | Measure-Object SansNom -Sum).Sum; $F = ($rows | Measure-Object NonFocalisables -Sum).Sum
if ($I -gt 0) { "TOTAL : {0} fenetres, {1} controles interactifs mesures ; avec nom accessible : {2:P1} ; atteignables au clavier : {3:P1}" -f $rows.Count, $I, (1 - $S / $I), (1 - $F / $I) } else { "Aucun controle interactif mesure" }
