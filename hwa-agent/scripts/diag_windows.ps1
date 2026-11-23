# Diagnostic Windows en LECTURE SEULE (mode utilisateur, sans pilote, sans ecriture).
# Il releve les memes informations que l'agent UEFI : disques, partitions, NVMe, PCI, USB,
# firmware, processeur. Il n'ecrit rien sur les disques ni sur la cle, et n'installe rien.
#
# Usage (PowerShell, en administrateur pour les informations de stockage) :
#   powershell -ExecutionPolicy Bypass -File scripts\diag_windows.ps1 [-Sortie chemin\rapport.txt]

param(
    [string] $Sortie = (Join-Path $PSScriptRoot "..\rapport_windows.txt")
)

$lignes = New-Object System.Collections.Generic.List[string]
function Ajouter([string] $t) { $lignes.Add($t) | Out-Null }

Ajouter "===== DIAGNOSTIC WINDOWS (lecture seule) $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') ====="

Ajouter "`n[firmware]"
$bios = Get-CimInstance Win32_BIOS
Ajouter "  Fabricant : $($bios.Manufacturer)  Version : $($bios.SMBIOSBIOSVersion)  Date : $($bios.ReleaseDate)"
$carte = Get-CimInstance Win32_BaseBoard
Ajouter "  Carte mere : $($carte.Manufacturer) $($carte.Product) (serie masquee)"
$secure = try { Confirm-SecureBootUEFI } catch { "indisponible" }
Ajouter "  Secure Boot : $secure"

Ajouter "`n[processeur]"
foreach ($cpu in Get-CimInstance Win32_Processor) {
    Ajouter "  $($cpu.Name) | coeurs $($cpu.NumberOfCores) | logiques $($cpu.NumberOfLogicalProcessors) | famille $($cpu.Family) | revision $($cpu.Revision)"
}

Ajouter "`n[disques physiques]"
foreach ($d in Get-Disk) {
    Ajouter "  Disque $($d.Number) : $($d.FriendlyName) | bus $($d.BusType) | $([math]::Round($d.Size/1GB,1)) Go | $($d.PartitionStyle) | $($d.OperationalStatus)"
}

Ajouter "`n[partitions]"
foreach ($p in Get-Partition) {
    $lettre = if ($p.DriveLetter) { $p.DriveLetter } else { "-" }
    Ajouter "  Disque $($p.DiskNumber) part $($p.PartitionNumber) | lettre $lettre | $([math]::Round($p.Size/1MB,0)) Mo | type $($p.Type) | debut $($p.Offset)"
}

Ajouter "`n[NVMe / sante (si disponible)]"
foreach ($pd in Get-PhysicalDisk) {
    if ($pd.BusType -eq "NVMe") {
        $fiab = $pd | Get-StorageReliabilityCounter -ErrorAction SilentlyContinue
        if ($fiab) {
            Ajouter "  $($pd.FriendlyName) : temperature $($fiab.Temperature) C | usure $($fiab.Wear) % | heures $($fiab.PowerOnHours) | erreurs lecture $($fiab.ReadErrorsTotal)"
        } else {
            Ajouter "  $($pd.FriendlyName) : compteurs de fiabilite indisponibles (lancer en administrateur)"
        }
    }
}

Ajouter "`n[PCI (peripheriques presents)]"
foreach ($dev in Get-PnpDevice -PresentOnly -Class "System","Display","Net","USB","MEDIA","HDC" -ErrorAction SilentlyContinue) {
    Ajouter "  $($dev.Class) | $($dev.FriendlyName) | $($dev.InstanceId)"
}

Ajouter "`n[USB]"
foreach ($u in Get-PnpDevice -PresentOnly -Class "USB" -ErrorAction SilentlyContinue | Where-Object { $_.InstanceId -like "USB*" }) {
    Ajouter "  $($u.FriendlyName) | $($u.InstanceId)"
}

Ajouter "`n===== FIN : aucune ecriture effectuee ====="

$chemin = [System.IO.Path]::GetFullPath($Sortie)
[System.IO.File]::WriteAllLines($chemin, $lignes, [System.Text.Encoding]::UTF8)
Write-Output "Rapport ecrit : $chemin ($($lignes.Count) lignes)"
