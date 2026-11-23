#!/bin/bash
# Fabrique les images de TEST a ecrire sur une cle USB (c'est l'utilisateur qui les ecrit : ce script n'ecrit sur AUCUN peripherique, seulement des fichiers dans livraison-usb/).
#   voxa-os-test.img   : Voxa OS (systeme maison, clavier 8042 apres la sortie des services UEFI, voix HD Audio directe)
#   voxa-live-test.img : Voxa Live (repli : lit le clavier par le firmware UEFI, voix HD Audio) si le clavier de l'OS ne repond pas sur la vraie machine
# Verifications : voxa-img (structure GPT/FAT/PE) ; copie VHD montee en lecture seule par Windows (tests/verif-image-windows.ps1) ; sommes SHA-256.
# Usage : tests/fabrique-usb.sh
cd "$(dirname "$0")/.." || exit 99
fail=0; ok() { echo "[PASS] $1"; }; ko() { echo "[FAIL] $1"; fail=$((fail+1)); }
OUT=livraison-usb; mkdir -p "$OUT"
(cd voxa-os && CARGO_TARGET_DIR="$TEMP/os-target" cargo build --release --target x86_64-unknown-uefi >/dev/null 2>&1) && ok "compilation voxa-os (UEFI)" || { ko "compilation voxa-os"; exit 98; }
(cd voxa-live-rs && CARGO_TARGET_DIR="$TEMP/vlrs-target" cargo build --release --target x86_64-unknown-uefi >/dev/null 2>&1) && ok "compilation voxa-live-rs (UEFI)" || { ko "compilation voxa-live-rs"; exit 98; }
(cd voxa-mkimg && CARGO_TARGET_DIR="$TEMP/mkimg-target" cargo build --release >/dev/null 2>&1) && ok "compilation voxa-mkimg" || { ko "compilation voxa-mkimg"; exit 98; }
(cd voxa-img && CARGO_TARGET_DIR="$TEMP/img-target" cargo build --release >/dev/null 2>&1) && ok "compilation voxa-img" || { ko "compilation voxa-img"; exit 98; }
MK="$TEMP/mkimg-target/release/voxa-mkimg.exe"; IMG="$TEMP/img-target/release/voxa-img.exe"
OS="$TEMP/os-target/x86_64-unknown-uefi/release/voxa-os.efi"; LIVE="$TEMP/vlrs-target/x86_64-unknown-uefi/release/voxa-live-rs.efi"
cp LISEZMOI-USB.txt "$TEMP/LISEZMOI-USB.txt" 2>/dev/null || printf 'VOXA - image de test. Voir LISEZMOI-TEST-USB.md.\r\n' > "$TEMP/LISEZMOI-USB.txt"
for nom in os live; do
  if [ "$nom" = os ]; then E="$OS"; else E="$LIVE"; fi
  "$MK" --sortie "$OUT/voxa-$nom-test.img" --taille-mio 64 EFI/BOOT/BOOTX64.EFI="$E" LISEZMOI.TXT="$TEMP/LISEZMOI-USB.txt" >/dev/null && ok "image voxa-$nom-test.img fabriquee" || ko "fabrication voxa-$nom-test.img"
  "$IMG" "$OUT/voxa-$nom-test.img" | tail -1 | grep -q "^Résultat : PASS" && ok "voxa-img : voxa-$nom-test.img structure valide (MBR protecteur, GPT + copie de secours + CRC32, FAT32, BOOTX64.EFI = PE x86-64)" || ko "voxa-img : voxa-$nom-test.img"
  "$MK" --sortie "$TEMP/verif-$nom.vhd" --taille-mio 64 --vhd EFI/BOOT/BOOTX64.EFI="$E" LISEZMOI.TXT="$TEMP/LISEZMOI-USB.txt" >/dev/null
  r=$(powershell.exe -NoProfile -ExecutionPolicy Bypass -File tests/verif-image-windows.ps1 -Vhd "$(cygpath -w "$TEMP/verif-$nom.vhd")" -Efi "$(cygpath -w "$E")" 2>&1 | tr -d '\r'); echo "$r" | tail -1 | grep -q "^\[PASS\]" && ok "Windows (pilote FAT/GPT natif, lecture seule) : voxa-$nom : fichier identique a la source" || { ko "verification Windows voxa-$nom"; echo "$r" | tail -4; }
done
( cd "$OUT" && sha256sum voxa-os-test.img voxa-live-test.img > SHA256SUMS.txt ) && ok "sommes SHA-256 ecrites dans $OUT/SHA256SUMS.txt"
echo "Bilan fabrication USB : $fail echec(s). Structure verifiee ; AUCUN demarrage reel n'a ete teste."
exit $fail
