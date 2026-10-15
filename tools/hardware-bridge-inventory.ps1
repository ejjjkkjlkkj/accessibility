# Read-only Windows physical-device inventory for the accessibility hardware bridge.
# Compatible with Windows PowerShell 5.1 and PowerShell 7. No admin required.
# Does not write firmware, access PCI config space/MMIO/ports/DMA, or reboot.
[CmdletBinding()]
param(
    [string]$OutputPath = (Join-Path $env:TEMP 'accessibility-hardware-inventory.json'),
    [switch]$SelfTest
)
$ErrorActionPreference = 'Stop'
function Test-DeviceRecord {
    param([object]$Record)
    return (
        $null -ne $Record -and
        -not [string]::IsNullOrWhiteSpace([string]$Record.Name) -and
        -not [string]::IsNullOrWhiteSpace([string]$Record.Origin) -and
        $Record.Origin -eq 'WindowsPnPInventory' -and
        $Record.PnpDeviceId -is [string]
    )
}
if ($SelfTest) {
    $good = [pscustomobject]@{ Name='Test device'; Origin='WindowsPnPInventory'; PnpDeviceId='PCI\VEN_1234&DEV_5678' }
    $bad = [pscustomobject]@{ Name=''; Origin='Unknown'; PnpDeviceId=$null }
    if ((Test-DeviceRecord $good) -and -not (Test-DeviceRecord $bad)) {
        'SELFTEST PASS: inventory record validation'
    } else {
        throw 'SELFTEST FAIL: inventory record validation'
    }
    return
}
$started = [DateTime]::UtcNow
$records = @()
try {
    $devices = Get-CimInstance -ClassName Win32_PnPEntity -ErrorAction Stop
    foreach ($device in $devices) {
        $pnpId = [string]$device.PNPDeviceID
        $record = [pscustomobject]@{
            Id = if ($pnpId) { $pnpId } else { 'UNKNOWN-' + [guid]::NewGuid().ToString('N') }
            Name = [string]$device.Name
            PnpDeviceId = $pnpId
            PnpClass = [string]$device.PNPClass
            Manufacturer = [string]$device.Manufacturer
            Status = [string]$device.Status
            ConfigManagerErrorCode = if ($null -ne $device.ConfigManagerErrorCode) { [int]$device.ConfigManagerErrorCode } else { $null }
            IsPci = ($pnpId -match '^(PCI|ACPI)\\')
            Origin = 'WindowsPnPInventory'
            Evidence = 'Reported by Windows Plug and Play; not independently verified against hardware.'
        }
        if (Test-DeviceRecord $record) { $records += $record }
    }
} catch {
    throw ('Inventory collection failed: ' + $_.Exception.Message)
}
$report = [pscustomobject]@{
    SchemaVersion = 1
    CollectedAtUtc = $started.ToString('o')
    ComputerName = $env:COMPUTERNAME
    Collector = 'accessibility-hardware-bridge/windows-read-only'
    ReadOnly = $true
    Origin = 'WindowsPnPInventory'
    EvidenceLevel = 'OS_REPORTED_NOT_PHYSICALLY_VERIFIED'
    Safety = [pscustomobject]@{
        FirmwareModified = $false
        DirectPciConfigAccess = $false
        MmioAccess = $false
        PortIo = $false
        Dma = $false
        RebootRequested = $false
    }
    DeviceCount = $records.Count
    Devices = @($records | Sort-Object PnpClass, Name)
}
$parent = Split-Path -Parent $OutputPath
if ($parent -and -not (Test-Path -LiteralPath $parent)) {
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
}
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
Write-Output ('PASS: collected ' + $report.DeviceCount + ' Windows PnP records (OS-reported, not physical proof).')
Write-Output ('JSON: ' + (Resolve-Path -LiteralPath $OutputPath).Path)
