#!/bin/bash
# Mesure phon.vx sur un lexique aligné : tests/eval-phon.sh <lexique> [dossier-binaires]
# Écrit les écarts et le taux de mots exacts. Émulation uniquement (phon.vx exécuté dans l'émulateur maison).
cd "$(dirname "$0")/.." || exit 99
LEX=$1; B=${2:-target2/release}
grep -v '^#' "$LEX" | cut -d: -f1 | grep -v "'" | grep -v '^$' | awk '!s[$0]++' > /tmp/ev-mots.txt
{ echo 'include "../bib/ecran.vx"'; echo 'include "../bib/phon.vx"'; cat <<'EOT'
fn essai(t: int) {
    let src = scratch() + 600000;
    let out = scratch() + 650000;
    let n = utf8_vers_u16(t, src);
    let d = 0;
    while d < n {
        d = phon_texte(src, d, n, out);
        afficher(out);
    }
    afficher("\n");
}
fn main() -> int {
    let ok = ecran_init();
    echelle(1);
EOT
while IFS= read -r w; do echo "    essai(\"$w\");"; done < /tmp/ev-mots.txt; echo '    reset(2);'; echo '    return 0;'; echo '}'; } > exemples/phon_eval_tmp.vx
"$B/voxa.exe" build exemples/phon_eval_tmp.vx out/phon_eval_tmp.efi --efi >/dev/null 2>&1 || { echo "compilation impossible"; exit 98; }
"$B/emu.exe" efi out/phon_eval_tmp.efi --ecran 600x9000 2>&1 | sed -n '/Texte affiché/,/fin du texte/p' | sed '1d;$d' | sed 's/[ ]*$//' | grep -v '^$' | tr -d '\r' > /tmp/ev-phon.txt
if [ "$(wc -l < /tmp/ev-mots.txt)" != "$(wc -l < /tmp/ev-phon.txt)" ]; then echo "nombre de lignes différent : $(wc -l < /tmp/ev-mots.txt) mots, $(wc -l < /tmp/ev-phon.txt) sorties"; exit 97; fi
paste /tmp/ev-mots.txt /tmp/ev-phon.txt > /tmp/ev-base.txt
"${TRAIN:-$TEMP/train-target/release/voxa-train.exe}" eval "$LEX" /tmp/ev-base.txt
