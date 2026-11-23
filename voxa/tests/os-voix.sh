#!/bin/bash
# Voix de Voxa OS APRES ExitBootServices : texte -> phonetique -> synthese -> pilote HD Audio (E/S + DMA, sans service UEFI) -> MODELE du controleur et du codec (emulateur).
# Preuve produite : (1) le pilote s'initialise apres la sortie des services UEFI ; (2) chaque lecture est jugee audible par le modele du codec, aucun echec DMA ;
# (3) le PCM recu par le controleur est identique octet par octet a la synthese de reference recalculee sur l'hote a partir du texte exact dit par l'OS (lignes « = »).
# NE PROUVE PAS : audio reel, audibilite par une personne, comportement du vrai Realtek ALC256 ni du controleur AMD, ni la compatibilite de l'ordre d'initialisation sur ce PC.
# Usage : tests/os-voix.sh [dossier-binaires-emulateur]
cd "$(dirname "$0")/.." || exit 99
B=${1:-target2/release}; fail=0
ok() { echo "[PASS] $1"; }; ko() { echo "[FAIL] $1"; fail=$((fail+1)); }
(cd voxa-os && CARGO_TARGET_DIR="$TEMP/os-target" cargo build --release --target x86_64-unknown-uefi >/dev/null 2>&1) || { echo "[FAIL] compilation de voxa-os"; exit 98; }
(cd voxa-speech && CARGO_TARGET_DIR="$TEMP/speech-target" cargo build --release --bin pcm_attendu >/dev/null 2>&1) || { echo "[FAIL] compilation de pcm_attendu"; exit 98; }
EFI="$TEMP/os-target/x86_64-unknown-uefi/release/voxa-os.efi"; WAV="$TEMP/os-voix-test.wav"; LOG="$TEMP/os-voix-test.log"
# trois fleches bas, Entree (Tester le son), Fin, Entree, Entree (extinction)
VOXA_MAX_STEPS=3000000000 timeout 580 "$B/emu.exe" efi "$EFI" --scancodes "E0,50,E0,50,E0,50,1C,E0,4F,1C,1C" --wav "$WAV" > "$LOG" 2>&1
grep -aq "ResetSystem demandé : Some(2)" "$LOG" && ok "OS+voix : extinction demandee, programme termine" || ko "OS+voix : extinction"
grep -aq "\[PASS\] exécution émulée sans violation d'ABI" "$LOG" && ! grep -aq "APRES ExitBootServices" "$LOG" && ok "OS+voix : aucun appel UEFI apres ExitBootServices, aucune violation d'ABI" || ko "OS+voix : violation"
grep -aq "audio\] controleur HD Audio initialise apres la sortie des services UEFI" "$LOG" && ok "OS+voix : pilote HD Audio initialise APRES la sortie des services UEFI (codec, broche, convertisseur trouves)" || ko "OS+voix : pilote non initialise"
aud=$(grep -ac "AUDIBLE dans le modèle" "$LOG"); non=$(grep -ac "NON audible" "$LOG"); dma=$(grep -ac "ECHEC de lecture DMA\|ÉCHEC de lecture DMA" "$LOG")
[ "$aud" -ge 10 ] && [ "$non" = "0" ] && [ "$dma" = "0" ] && ok "OS+voix : $aud lectures jugees audibles par le modele du codec, 0 non audible, 0 echec DMA" || ko "OS+voix : $aud audibles, $non non audibles, $dma echecs DMA"
grep -a "Port série" "$LOG" | head -1 | perl -pe 's/\x5cr\x5cn/\n/g' | grep -a "^= " | sed 's/^= //' | grep -v '^$' > "$TEMP/os-annonces-exact.txt"
res=$("$TEMP/speech-target/release/pcm_attendu.exe" "$TEMP/os-annonces-exact.txt" "$WAV" 2>&1); echo "$res" | tail -1 | grep -q "^\[PASS\]" && ok "OS+voix : $(echo "$res" | tail -1 | sed 's/^\[PASS\] //')" || { ko "OS+voix : PCM different de la synthese de reference"; echo "$res" | tail -3; }
grep -aq "Ceci est un test du son" "$LOG" && ok "OS+voix : « Tester le son » dit une phrase par la voix (le bip de secours n'est utilise qu'en cas d'echec)" || ko "OS+voix : test du son"
echo "Bilan voix OS : $fail echec(s). Emulation (modele logiciel du controleur) uniquement ; aucune ecoute, aucun materiel."
exit $fail
