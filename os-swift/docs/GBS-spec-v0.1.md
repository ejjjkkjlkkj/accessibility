# Graine Boot Specification (GBS) — version 0.1 (brouillon)

Statut : brouillon à valider. Ce document définit une couche de démarrage au-dessus d'UEFI. Il ne remplace pas le firmware.

## 1. Périmètre

- GBS est une couche de démarrage écrite en Swift. Il ne remplace pas le firmware et n'utilise pas UEFI dans sa v0.1.
- GBS définit : la chaîne de confiance, le format de configuration, le journal de démarrage et le retour automatique.
- Hors périmètre pour la v0.1 : pilotes matériels, réseau, interface graphique.

## 2. Composants

| Composant | Rôle | Équivalent connu |
|---|---|---|
| Chargeur GBS | Charge la configuration, vérifie et lance le noyau | GRUB, Windows Boot Manager |
| Noyau graine | Micro-noyau : mémoire, processus, communication entre processus | noyau Linux, noyau NT |
| Environnement de secours | Système minimal de dépannage | WinPE |
| Environnement de récupération | Réparation du système installé | WinRE |

## 3. Chaîne de confiance

1. Une clé racine GBS signe un manifeste de démarrage.
2. Le manifeste liste l'empreinte (SHA-256) et la taille de chaque fichier de démarrage.
3. Le chargeur refuse tout fichier dont l'empreinte ne correspond pas au manifeste.
4. Le manifeste lui-même est vérifié par signature avant toute lecture des fichiers.

## 4. Configuration déclarative

- Un seul fichier texte `boot.gbs` à la racine de la partition de démarrage.
- Format : paires clé-valeur, une par ligne, commentaires commençant par `#`.
- Exemple :

```
# boot.gbs v0.1
kernel    = \graine\noyau.bin
secours   = \secours\env.bin
recuperation = \recuperation\env.bin
retour    = secours
```

- Aucune variable en mémoire non volatile : toute la configuration est dans le fichier.

## 5. Journal de démarrage

- Chaque étape écrit une entrée : étape, nom du fichier, empreinte, résultat (ok, échec).
- Le journal est écrit dans `\journal\boot.log` et conservé entre deux démarrages.

## 6. Retour automatique

- Si le noyau ne répond pas dans un délai donné (valeur à définir), le chargeur lance `retour` (ici `secours`).
- Le journal enregistre l'échec et la raison.

## 7. Décisions (v0.1, en vigueur)

- **Délai avant retour automatique** : 10 secondes par défaut, réglable par la clé `delai`.
- **Empreintes** : SHA-256 pour chaque fichier, dans le manifeste `\manifeste.gbs`.
- **Signature du manifeste** : Ed25519. Implémentée dans une version ultérieure ; la v0.1 vérifie les empreintes seulement.
- **Langage** : tout est en Swift, chargeur compris. Pas de C, pas de code UEFI. Le chargeur est une application Swift qui lit `boot.gbs`, vérifie le manifeste, lance le noyau et gère le retour automatique.
- **Lancement** : la v0.1 lance les fichiers par une interface `KernelLauncher`, testable sur Windows. Le lancement réel sur matériel viendra avec le noyau graine.

## 8. Format du manifeste (v0.1)

- Une ligne par fichier : `<sha256-hex> <taille-octets> <chemin-relatif>`.
- Exemple :

```
3f2a... 4096 graine\noyau.bin
9c1b... 2048 secours\env.bin
```

- Les lignes vides et les lignes commençant par `#` sont ignorées.
