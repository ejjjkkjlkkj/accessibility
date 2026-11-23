#!/bin/bash
# Chaîne vocale COMPLÈTE de l'application Rust, exécutée dans l'émulateur : texte -> phonétique (Rust) -> synthèse (Rust) -> pilote HD Audio (Rust) -> modèle du contrôleur.
# Preuve produite : (1) chaque lecture est jugée « audible » par le modèle du codec ; (2) le PCM reçu par le contrôleur est identique octet par octet à la synthèse de
# référence recalculée sur l'hôte. NE PROUVE PAS : audio réel, audibilité par une personne, comportement du vrai Realtek ALC256.
# Usage : tests/voix-rs.sh [dossier-binaires-émulateur]
cd "$(dirname "$0")/.." || exit 99
B=${1:-target/release}; fail=0
ok() { echo "[PASS] $1"; }; ko() { echo "[FAIL] $1"; fail=$((fail+1)); }
(cd voxa-live-rs && CARGO_TARGET_DIR="$TEMP/vlrs-target" cargo build --release --target x86_64-unknown-uefi >/dev/null 2>&1) || { echo "[FAIL] compilation voxa-live-rs"; exit 98; }
(cd voxa-speech && CARGO_TARGET_DIR="$TEMP/speech-target" cargo build --release --bin pcm_attendu >/dev/null 2>&1) || { echo "[FAIL] compilation de pcm_attendu"; exit 98; }
EFI="$TEMP/vlrs-target/x86_64-unknown-uefi/release/voxa-live-rs.efi"
timeout 580 "$B/emu.exe" efi "$EFI" --ecran 640x480 --touches "down,end,enter,enter" --wav out/rust-voix.wav > /tmp/rsvoix.log 2>&1
grep -q "ResetSystem demandé : Some(2)" /tmp/rsvoix.log && ok "Rust+voix : extinction demandée, programme terminé" || ko "Rust+voix : extinction"
grep -q "\[PASS\] exécution émulée sans violation d'ABI" /tmp/rsvoix.log && ok "Rust+voix : aucune violation d'ABI" || ko "Rust+voix : ABI"
grep -q "contrôleur HD Audio initialisé" /tmp/rsvoix.log && ok "Rust+voix : pilote HD Audio Rust initialisé (contrôleur, codec, chemin broche->convertisseur)" || ko "Rust+voix : pilote audio non initialisé"
aud=$(grep -c "AUDIBLE dans le modèle" /tmp/rsvoix.log); non=$(grep -c "NON audible" /tmp/rsvoix.log)
[ "$aud" -ge 10 ] && [ "$non" = "0" ] && ok "Rust+voix : $aud lectures jugées audibles par le modèle du codec, 0 non audible" || ko "Rust+voix : $aud audibles, $non non audibles"
echo "$(grep -c 'tampon consommé par le DMA' /tmp/rsvoix.log) tampons consommés ; échecs DMA : $(grep -c 'ÉCHEC de lecture DMA' /tmp/rsvoix.log)" >/dev/null
[ "$(grep -c 'ÉCHEC de lecture DMA' /tmp/rsvoix.log)" = "0" ] && ok "Rust+voix : aucun échec de lecture DMA" || ko "Rust+voix : échec de lecture DMA"
sed -n '/Annonces/,/fin des annonces/p' /tmp/rsvoix.log | sed '1d;$d' | grep -v '^\[audio\]' | tr -d '\r' > /tmp/annonces-rs.txt
res=$("$TEMP/speech-target/release/pcm_attendu.exe" /tmp/annonces-rs.txt out/rust-voix.wav 2>&1); echo "$res" | tail -1 | grep -q "^\[PASS\]" && ok "Rust+voix : $(echo "$res" | tail -1 | sed 's/^\[PASS\] //')" || { ko "Rust+voix : PCM différent de la référence"; echo "$res"; }
echo "Bilan voix Rust : $fail échec(s). Émulation (modèle logiciel du contrôleur) uniquement ; aucune écoute, aucun matériel."
exit $fail
