# Binaire UEFI de l'agent (version voix)

- Fichier : `hwa2_agent_voix.efi` (construit depuis `hwa-agent/src/` à partir de la version voix du 2026-11-13).
- SHA-256 : `030c593539ea82dea64d7ee13cce4ec7a0dad3e06d81c5aacd58881cb6dcab21`
- Le nom d'utilisateur présent dans les chemins de compilation a été remplacé par `usr` (même longueur, fichier toujours valide).
- Ce binaire quitte les services UEFI et accède aux pilotes matériels (phase R0) : ne l'exécuter que sur une machine de test, depuis une clé dédiée, en connaissance de cause.
- La CI ne le construit ni ne le lance.
