# Réduction de la surface réseau : seuls les ports nécessaires restent ouverts

**Statut : SPECIFICATION — NOT_IMPLEMENTED tant qu'un moteur et des preuves d'exécution ne sont pas fournis.**  
Objectif : aucun port d'écoute ne doit être exposé sans besoin explicite, propriétaire, justification, périmètre et test.

## Règle de sécurité

- Refuser par défaut les connexions entrantes non sollicitées, en IPv4 comme en IPv6.
- N'autoriser un port que si un service déclaré en a besoin, avec protocole (TCP/UDP), adresse d'écoute, interface, propriétaire, durée de vie et justification.
- Préférer une écoute sur loopback (`127.0.0.1` / `::1`) pour un service local. Ne pas écouter sur toutes les interfaces (`0.0.0.0` / `::`) sans exigence documentée.
- Appliquer le moindre privilège aux flux sortants aussi : destinations, ports et protocoles autorisés explicitement quand le modèle de fonctionnement le permet.
- Fermer le socket et retirer la règle temporaire dès que le service s'arrête ou que le besoin expire.
- Contrôler les ports TCP **et** UDP, IPv4 **et** IPv6, ainsi que les mécanismes d'accès locaux (sockets Unix/named pipes) : fermer un port TCP ne suffit pas à supprimer une surface d'attaque.
- Les ports éphémères de connexions clientes ne sont pas des services d'écoute et ne doivent pas être bloqués aveuglément.

## Contrat obligatoire pour chaque service

Chaque service réseau doit déclarer un manifeste contenant au minimum :

- identifiant et version du service ;
- besoin métier et propriétaire responsable ;
- protocole et port ou plage strictement nécessaire ;
- adresse/interface d'écoute attendue ;
- sens du flux (entrant, sortant, les deux) ;
- destinations distantes autorisées, si applicables ;
- conditions d'activation, dépendances et durée de vie ;
- authentification, chiffrement, limites de débit et journalisation adaptées ;
- méthode de vérification et tests de régression ;
- procédure de désactivation/récupération.

L'absence de manifeste, une justification vide, un port trop large, une écoute plus exposée que prévu ou un test non exécuté doit produire **FAIL**, **BLOCKED** ou **NOT_RUN**, jamais PASS.

## Contrôle au démarrage et en continu

1. Construire l'inventaire attendu à partir des services activés et de leurs manifestes.
2. Relever les sockets d'écoute réellement présents (TCP/UDP, IPv4/IPv6), leur PID/propriétaire, leur adresse et le service associé.
3. Comparer attendu et observé : chaque écoute non déclarée est une anomalie ; chaque écoute déclarée mais absente est une anomalie de disponibilité.
4. Vérifier les règles du pare-feu hôte et les règles réseau amont. Une règle de pare-feu ne prouve pas à elle seule qu'aucun service n'écoute.
5. Refaire le contrôle après installation, mise à jour, activation d'un service, changement réseau et redémarrage de test.
6. Enregistrer un rapport horodaté avec inventaire, règles, écarts, commandes/outils, code retour et empreinte des artefacts.
7. En cas d'écart, alerter et bloquer la validation de release. Ne pas tuer automatiquement un processus système inconnu et ne pas supprimer une règle active sans vérifier les dépendances : appliquer une remédiation contrôlée, réversible et journalisée.

## Tests HAKER requis

- Un service sans manifeste est détecté.
- Un service arrêté ne laisse ni socket d'écoute ni règle temporaire orpheline.
- Un port déclaré sur loopback mais lié à toutes les interfaces échoue.
- Les contrôles couvrent TCP, UDP, IPv4 et IPv6.
- Un port ouvert non prévu est détecté ; un port nécessaire et correctement restreint n'est pas signalé à tort.
- Une règle permissive (`any/any` ou plage injustifiée) échoue.
- Les tests distinguent écoute locale, connexion cliente éphémère et port réellement exposé.
- Les rapports n'incluent pas de secrets et sont lisibles au clavier et par lecteur d'écran.
- Tout contrôle non exécuté ou impossible à observer est marqué **NOT_RUN/BLOCKED**, jamais PASS.

## Limites et déploiement

- Ne pas appliquer une liste universelle de « ports à fermer » : les besoins varient selon les rôles et les fonctions activées.
- Ne pas couper à l'aveugle DHCP, DNS, NTP, VPN, mises à jour, accessibilité distante ou services de récupération nécessaires au système.
- Commencer en **audit read-only** ; produire les écarts avant toute modification. Ensuite, générer des règles de pare-feu minimales par profil (poste local, serveur, récupération), avec simulation, validation et retour arrière.
- Les politiques doivent être adaptées au backend de chaque OS (Windows Filtering Platform/Windows Defender Firewall, nftables ou équivalent) ; la spécification commune ne remplace pas leurs tests natifs.
- Un résultat en CI ou en VM prouve uniquement ce qui a été observé dans cet environnement, pas l'état réseau d'un PC physique.

## Critères d'acceptation

- [ ] Inventaire des sockets d'écoute reproductible, sans privilèges inutiles.
- [ ] Manifeste obligatoire pour chaque service réseau.
- [ ] Pare-feu entrant en refus par défaut et exceptions minimales documentées.
- [ ] Politique de sortie définie par profil.
- [ ] Vérification IPv4/IPv6, TCP/UDP et interfaces réelles.
- [ ] Détection de dérive en continu et rapport accessible.
- [ ] Tests positifs, négatifs, régression et remédiation réversible.
- [ ] Validation séparée sur chaque OS ciblé et sur matériel réel lorsque nécessaire.

## Références

- NIST Cybersecurity Framework 2.0 : gestion, protection, détection, réponse et récupération.
- NIST SP 800-207 : Zero Trust et contrôle explicite des accès.
- NIST SP 800-218 (SSDF) : pratiques de développement sécurisé.
- Documentation officielle du pare-feu et de la pile réseau de chaque OS cible.

Ces références orientent la conception ; elles ne prouvent pas que les contrôles sont déjà implémentés.
