# Verification INDEPENDANTE d'une image : Windows (son propre pilote GPT/FAT32) monte la copie VHD en LECTURE SEULE, on compare le SHA-256 de BOOTX64.EFI a la source, puis on demonte.
# N'ecrit rien sur l'image, ne touche a aucun disque reel. Derniere ligne : [PASS] ou [FAIL].
param([string]$Vhd, [string]$Efi)
$res = '[FAIL] verification non terminee'
try {
  $img = Mount-DiskImage -ImagePath $Vhd -Access ReadOnly -PassThru -ErrorAction Stop
  $disk = $img | Get-Disk
  if ($disk.PartitionStyle -ne 'GPT' -or -not $disk.IsReadOnly) { throw "disque non GPT ou non lecture seule" }
  $p = Get-Partition -DiskNumber $disk.Number | Where-Object { $_.Type -eq 'System' } | Select-Object -First 1
  if (-not $p) { throw "aucune partition systeme EFI vue par Windows" }
  $lettre = (68..90 | ForEach-Object { [char]$_ } | Where-Object { -not (Test-Path "$($_):\") } | Select-Object -First 1)
  Add-PartitionAccessPath -DiskNumber $disk.Number -PartitionNumber $p.PartitionNumber -AccessPath "$($lettre):" -ErrorAction Stop
  # le volume peut ne pas etre pret juste apres l'attribution de la lettre : jusqu'a 10 essais espaces d'une seconde (une vraie erreur de lecture persiste et fait echouer le test)
  $h1 = $null; for ($t = 0; $t -lt 10 -and -not $h1; $t++) { Start-Sleep -Seconds 1; $h1 = (Get-FileHash "$($lettre):\EFI\BOOT\BOOTX64.EFI" -Algorithm SHA256 -ErrorAction SilentlyContinue).Hash }
  if (-not $h1) { throw "BOOTX64.EFI illisible apres 10 essais" }
  $h2 = (Get-FileHash $Efi -Algorithm SHA256).Hash
  Remove-PartitionAccessPath -DiskNumber $disk.Number -PartitionNumber $p.PartitionNumber -AccessPath "$($lettre):" -ErrorAction SilentlyContinue
  if ($h1 -eq $h2) { $res = "[PASS] SHA256 identique ($h1)" } else { $res = "[FAIL] SHA256 different : image $h1, source $h2" }
} catch { $res = "[FAIL] $($_.Exception.Message)" }
finally { Dismount-DiskImage -ImagePath $Vhd -ErrorAction SilentlyContinue | Out-Null }
$res
