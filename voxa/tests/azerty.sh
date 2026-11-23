#!/bin/bash
# Convertit un texte en codes du jeu 1 (clavier AZERTY francais, sans Maj) pour --scancodes. Caracteres : lettres, espace, apostrophe (touche 4), virgule, point-virgule (shift non gere).
declare -A K=([a]=10 [z]=11 [e]=12 [r]=13 [t]=14 [y]=15 [u]=16 [i]=17 [o]=18 [p]=19 [q]=1E [s]=1F [d]=20 [f]=21 [g]=22 [h]=23 [j]=24 [k]=25 [l]=26 [m]=27 [w]=2C [x]=2D [c]=2E [v]=2F [b]=30 [n]=31 [" "]=39 ["'"]=05 [","]=32 [";"]=33 [":"]=34)
texte="$1"; out=""
for ((i=0; i<${#texte}; i++)); do c="${texte:$i:1}"; code="${K[$c]}"; [ -z "$code" ] && { echo "caractere non gere : $c" >&2; exit 1; }; out="$out,$code"; done
echo "${out#,}"
