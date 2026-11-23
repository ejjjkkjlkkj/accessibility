# Verifie le kit : XML bien forme, namespace unattend, passe windowsPE, composants attendus, absence d'elements destructifs, exe present. ASCII.
param([string]$Dossier)
$x = New-Object System.Xml.XmlDocument; $x.Load((Join-Path $Dossier 'autounattend.xml'))
$ns = New-Object System.Xml.XmlNamespaceManager($x.NameTable); $ns.AddNamespace('u', 'urn:schemas-microsoft-com:unattend')
function Ok($c, $m) { if ($c) { "[PASS] $m" } else { "[FAIL] $m" } }
Ok ($x.DocumentElement.NamespaceURI -eq 'urn:schemas-microsoft-com:unattend') 'autounattend.xml : bien forme, espace de noms unattend'
Ok ($x.SelectNodes('//u:settings[@pass="windowsPE"]', $ns).Count -eq 1) 'une seule passe windowsPE'
Ok ($x.SelectNodes('//u:component[@name="Microsoft-Windows-Setup"]/u:RunSynchronous/u:RunSynchronousCommand', $ns).Count -eq 2) 'deux commandes RunSynchronous (Narrator, lecteur de secours)'
Ok (($x.SelectNodes('//u:RunSynchronousCommand/u:Order', $ns) | ForEach-Object { $_.InnerText }) -join ',' -eq '1,2') 'ordre des commandes 1,2'
$txt = Get-Content (Join-Path $Dossier 'autounattend.xml') -Raw
Ok (($txt -notmatch '<DiskConfiguration') -and ($txt -notmatch '<ProductKey') -and ($txt -notmatch '<AcceptEula') -and ($txt -notmatch '<UserAccounts') -and ($txt -notmatch 'Format|Wipe|CreatePartition')) 'aucun partitionnement, aucune cle, aucune licence acceptee, aucun compte'
Ok ((Get-Item (Join-Path $Dossier 'voxa\voxa-companion.exe')).Length -gt 100000) 'voxa-companion.exe present'
