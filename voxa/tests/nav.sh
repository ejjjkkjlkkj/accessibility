#!/bin/bash
# Tests de navigation au clavier et de secours sonore de Voxa Live. ÉMULATION UNIQUEMENT (faux firmware, clavier simulé, pas de son réel).
# L'audio est volontairement indisponible (VOXA_HDA_NOCODEC=1) : on vérifie ainsi (1) la logique de navigation via le journal des annonces,
# (2) le retour sonore de secours (bips aigus = rang de l'élément), (3) la confirmation avant redémarrage / extinction.
# Usage : tests/nav.sh [dossier-binaires]
cd "$(dirname "$0")/.." || exit 99
B=${1:-target/release}
fail=0
ok() { echo "[PASS] $1"; }
ko() { echo "[FAIL] $1"; fail=$((fail+1)); }
"$B/voxa.exe" build exemples/live3.vx out/nav_live3.efi --efi >/dev/null 2>&1 || { echo "[FAIL] compilation de live3"; exit 98; }
run() { VOXA_HDA_NOCODEC=1 "$B/emu.exe" efi out/nav_live3.efi --ecran 640x480 --touches "$1" 2>&1; }
annonces() { sed -n '/Annonces (journal/,/fin des annonces/p' | sed '1d;$d' | tr -d '\r' | sed 's/ *$//'; }
nb() { grep -cF -- "$1"; }
nbx() { grep -cx -- "$1"; }   # lignes entières

# A. Navigation directe : 3, Fin, Début, bas, bas, 7, puis extinction confirmée
out=$(run "char:3,end,home,down,down,char:7,enter,enter"); ann=$(echo "$out" | annonces)
want=$'OpsjU tRwa syR sEt SQZe l@ kUtRast\nOpsjU sEt syR sEt etIdR lORdinat9R\nOpsjU I syR sEt liR lEd\nOpsjU d2 syR sEt SQZe la taj dy tEkst\nOpsjU tRwa syR sEt SQZe l@ kUtRast\nOpsjU sEt syR sEt etIdR lORdinat9R'
if echo "$ann" | grep -qF -- "$want"; then ok "navigation directe (3, Fin, Début, bas, bas, 7) : six annonces dans le bon ordre"; else ko "navigation directe : annonces inattendues"; echo "$ann"; fi
# retour sonore de secours : 3 + 7 + 1 + 2 + 3 + 7 = 23 bips de 1200 Hz (voix indisponible)
bips=$(echo "$out" | grep "Haut-parleur" | grep -o ", 1200)" | wc -l)
if [ "$bips" = "23" ]; then ok "secours sonore : 23 bips aigus (rang de chaque élément) sans voix"; else ko "secours sonore : $bips bips au lieu de 23"; fi
echo "$out" | grep -q "ResetSystem demandé : Some(2)" && ok "extinction après double Entrée" || ko "extinction attendue"

# B. Confirmation : une seule Entrée ne suffit pas ; Échap annule ; il faut reconfirmer
out=$(run "end,enter,esc,enter,enter"); ann=$(echo "$out" | annonces)
c=$(echo "$ann" | nb "kUfiRme lEstIksjU"); a=$(echo "$ann" | nbx "anyle"); f=$(echo "$ann" | nb "EstIksjU Q kuR")
if [ "$c" = "2" ] && [ "$a" = "1" ] && [ "$f" = "1" ]; then ok "confirmation d'extinction : demandée 2 fois, annulée 1 fois par Échap, exécutée après la 2e confirmation"; else ko "confirmation d'extinction ($c demandes, $a annulations, $f exécutions)"; fi

# C. Redémarrage : confirmation puis exécution (la demande de redémarrage n'est qu'enregistrée par l'émulateur)
out=$(run "char:6,enter,enter"); ann=$(echo "$out" | annonces)
if echo "$ann" | grep -qF "kUfiRme l@ R@demaRaZ" && echo "$ann" | grep -qF "R@demaRaZ Q kuR" && echo "$out" | grep -q "ResetSystem demandé : Some(0)"; then ok "redémarrage : confirmation puis demande (émulation)"; else ko "redémarrage"; fi

# D. Se déplacer annule une confirmation en attente
out=$(run "end,enter,up,down,enter,enter"); ann=$(echo "$out" | annonces)
if [ "$(echo "$ann" | nbx "anyle")" = "1" ] && [ "$(echo "$ann" | nb "kUfiRme lEstIksjU")" = "2" ]; then ok "déplacement : annule la confirmation en attente (reconfirmation nécessaire)"; else ko "annulation par déplacement"; fi

# E. Échap sans confirmation en attente
out=$(run "esc,end,enter,enter"); ann=$(echo "$out" | annonces)
echo "$ann" | grep -qF "RjI a anyle" && ok "Échap sans action en attente : « rien à annuler »" || ko "Échap sans action"

# F. Revue d'une page d'informations : lecture continue puis Fin relit la dernière ligne
out=$(run "char:5,enter,end,enter,enter,enter,char:7,enter,enter"); ann=$(echo "$out" | annonces)
n=$(echo "$ann" | nbx "o m@ny")
if [ "$n" = "2" ]; then ok "revue de page : dernière ligne dite à l'ouverture puis relue avec Fin"; else ko "revue de page : $n occurrences de la dernière ligne (attendu 2)"; fi

echo "Bilan navigation : $fail échec(s). Émulation uniquement ; aucun résultat sur matériel physique."
exit $fail
