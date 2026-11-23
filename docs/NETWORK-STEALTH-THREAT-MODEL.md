# Réseau discret, surface d'attaque et sécurité dès la conception

**Statut global : SPECIFICATION — NOT_IMPLEMENTED / NOT_VALIDATED tant que chaque contrôle n'a pas de code, de tests reproductibles et de preuves.**  
But : réduire l'exposition réseau et les surprises de sécurité, sans promettre une « invisibilité » absolue.

## 1. Ce que signifie « réseau invisible »

Un ordinateur connecté ne peut pas être garanti invisible : le fournisseur d'accès, le point d'accès, les équipements intermédiaires et les pairs autorisés peuvent observer certaines métadonnées. Un système peut toutefois être **non exposé aux connexions entrantes non nécessaires**, ne pas annoncer de services, minimiser les réponses aux sondes et limiter les informations divulguées.

Les contrôles de discrétion doivent rester interopérables, documentés et réversibles. Ils ne doivent pas contourner les contrôles de sécurité d'un tiers, ni masquer une activité malveillante.

## 2. Politique réseau par défaut

- Aucun service réseau activé sans manifeste, propriétaire, justification et durée de vie.
- Refus par défaut des connexions entrantes non sollicitées ; exceptions minimales et liées à une interface précise.
- Aucun service d'écoute sur toutes les interfaces sans nécessité approuvée.
- Désactiver la découverte et la publicité de services inutiles (par exemple annonces de partage/imprimante ou découverte locale) ; conserver uniquement les fonctions demandées.
- Désactiver les protocoles, interfaces et services non utilisés, après inventaire et vérification des dépendances.
- Ne pas répondre aux requêtes de découverte non nécessaires lorsque la pile réseau et le protocole le permettent ; ne pas falsifier les réponses et ne pas altérer le trafic d'autres machines.
- Examiner IPv4 et IPv6, TCP et UDP, multicast/broadcast, DNS, DHCP, ICMP/ICMPv6, interfaces virtuelles, VPN, Wi-Fi, Bluetooth et tunnels.
- Traiter séparément les connexions sortantes : destination, résolution DNS, certificat, proxy/VPN, télémétrie, mises à jour et accès cloud.
- Ne pas bloquer aveuglément ICMPv6, DHCP, DNS, NTP, VPN, mises à jour ou mécanismes de récupération : cela peut casser IPv6, l'accessibilité distante ou la sécurité.
- Ne jamais confondre « aucun port ouvert » avec « aucun risque » : les connexions sortantes, les périphériques locaux, les pilotes et la chaîne de démarrage restent des surfaces d'attaque.

## 3. Modèle de menace à définir avant le code

Documenter, pour chaque version et profil :

- actifs à protéger : identité, fichiers, clés, microphone, caméra, audio, clavier, braille, données d'accessibilité et modèles locaux ;
- adversaires : machine distante, pair du réseau local, application compromise, compte non privilégié, administrateur compromis, chaîne de dépendances compromise et accès physique ;
- frontières de confiance : firmware, démarrage, noyau, pilotes, services, applications, hyperviseur, routeur, DNS, fournisseur d'accès et périphériques ;
- capacités supposées de l'adversaire, impacts acceptables, hypothèses, risques résiduels et méthode de validation ;
- exigences de confidentialité, intégrité, disponibilité, traçabilité et provenance.

Aucune architecture ne peut garantir zéro vulnérabilité ni empêcher toute attaque. Les objectifs doivent être des critères mesurables, avec risque résiduel explicite.

## 4. Familles d'attaques à couvrir

Les tests doivent être réalisés uniquement dans des environnements possédés ou explicitement autorisés, avec fixtures ou laboratoires isolés.

1. **Reconnaissance/exposition** : ports inattendus, services de découverte, bannières, métadonnées DNS, noms d'hôte, interfaces oubliées et surfaces IPv6.
2. **Authentification/autorisation** : identifiants faibles ou par défaut, sessions mal invalidées, droits excessifs, accès inter-utilisateurs et clés de service.
3. **Réseau en transit** : usurpation, empoisonnement, interception, downgrade, certificats invalides, rejeu et mauvaise validation des messages.
4. **Services et protocoles** : entrées malformées, parsing, limites de taille, timeouts, erreurs, concurrence et épuisement de ressources.
5. **Disponibilité** : saturation de ressources, boucles de reconnexion, tempêtes de logs, file d'attente non bornée et récupération après panne. Tests de charge limités au laboratoire.
6. **Mouvement latéral/confinement** : service compromis tentant d'atteindre des ressources non autorisées ; séparation des processus, utilisateurs, namespaces/sandbox et règles egress.
7. **DNS et confidentialité** : fuite hors VPN/proxy attendu, résolution inattendue, recherche de domaines, télémétrie non déclarée et fuite de données dans les logs.
8. **Wi-Fi/Bluetooth et interfaces locales** : reconnexion automatique non sûre, réseaux mémorisés, appairage, découverte et bascule réseau.
9. **Système bas niveau** : pilotes, DMA/IOMMU, périphériques USB, firmware, démarrage sécurisé, mises à jour et récupération. Les tests matériels restent distincts des simulations.
10. **Chaîne d'approvisionnement** : dépendances, compilateurs, actions CI, artefacts, signatures, provenance, reproductibilité et mises à jour compromises.
11. **Interface d'administration/accessibilité** : changement de pare-feu au clavier et au lecteur d'écran, annonces compréhensibles, confirmation des changements risqués, journal accessible et restauration possible.
12. **Confidentialité latérale** : journaux, temporaires, caches, horodatages et données de diagnostic. Les canaux auxiliaires matériels nécessitent une analyse de risque spécifique.

## 5. Défenses à concevoir en couches

- **Démarrage/confiance** : vérifier la provenance des composants et la chaîne de démarrage ; politique de récupération signée et testée.
- **Identités** : identités distinctes par service, privilèges minimaux, secrets protégés, rotation et révocation.
- **Confinement** : isoler les services, réduire les capacités système, contrôler accès fichiers/périphériques et communications interservices.
- **Réseau** : pare-feu entrant par défaut-deny, filtrage sortant par profil, segmentation, contrôle IPv4/IPv6 et règles d'écoute explicites.
- **Transport** : protocoles sûrs, validation stricte des certificats, chiffrement authentifié et absence de downgrade silencieux.
- **Disponibilité** : limites de mémoire, CPU, connexions, tailles de messages, délais, backoff et mécanismes de récupération.
- **Observabilité** : journaux minimaux, protégés contre l'altération, sans secrets, horodatés et consultables avec lecteur d'écran.
- **Mises à jour** : signature, provenance, validation, déploiement progressif et retour arrière.
- **Vie privée** : aucune télémétrie cachée ; expliquer chaque communication, destination, finalité et option de désactivation.
- **Mode dégradé** : si DNS, réseau, voix, modèle IA ou mise à jour échoue, conserver les fonctions locales essentielles et annoncer clairement l'état.

## 6. Contrat de manifeste réseau

Chaque service réseau doit déclarer : identifiant/version, propriétaire, finalité, protocole/port, interfaces, sens, destinations, identité d'exécution, authentification, chiffrement, dépendances, télémétrie, activation, durée de vie, limites de ressources, journalisation, tests et méthode de désactivation.

Les services sans manifeste, les destinations inattendues, les écarts entre attendu et observé, ou les contrôles non exécutés donnent FAIL/BLOCKED/NOT_RUN — jamais PASS.

## 7. Tests et portes de validation

- **G0 — menace** : scénarios, actifs, frontières et risques résiduels documentés.
- **G1 — inventaire** : composants, interfaces, ports, services, dépendances et flux sortants connus.
- **G2 — politique** : refus par défaut, exceptions minimales, absence de service non déclaré.
- **G3 — tests adversariaux** : cas négatifs, fuzzing de parsers en laboratoire, tests d'autorisation, tests de fuite et tests de disponibilité bornés.
- **G4 — confinement** : vérifier qu'un composant compromis ne peut pas accéder aux ressources interdites.
- **G5 — intégrité de la chaîne** : artefacts signés, dépendances et provenance enregistrées ; tests de mise à jour et de retour arrière.
- **G6 — accessibilité** : toute alerte, décision et restauration est accessible clavier, lecteur d'écran et braille lorsque pris en charge.
- **G7 — validation** : CI, machine virtuelle, matériel réel et revue humaine consignés séparément.

Pour chaque test, enregistrer l'environnement, la version, la commande, le résultat, le code de sortie, les journaux expurgés de secrets et l'empreinte des artefacts. Un test simulé ne valide pas le matériel réel. Une analyse statique ne prouve pas l'absence d'attaque.

## 8. Réaction à une anomalie

1. Détecter et classer l'écart sans supprimer aveuglément un service système.
2. Limiter l'accès au composant concerné si cela ne détruit pas les fonctions essentielles.
3. Préserver les preuves utiles sans collecter de données personnelles inutiles.
4. Alerter dans une forme accessible et indiquer l'impact, l'incertitude et les options.
5. Appliquer une correction signée, réversible et testée ; ne jamais désactiver silencieusement la voix, le clavier ou la récupération.
6. Ajouter un test de régression et rejouer les contrôles.
7. Documenter cause racine, périmètre, décision, risque résiduel et preuves.

## 9. Critères de non-surprise

- Pas de port/service/destination réseau sans déclaration.
- Pas de communication cloud ou de télémétrie cachée.
- Pas de changement automatique de pare-feu sans mode audit, aperçu et retour arrière.
- Pas de mise à jour sans provenance et mécanisme de récupération.
- Pas de PASS quand un contrôle est ignoré, impossible ou seulement simulé.
- Pas de dépendance obligatoire à un réseau ou à une IA cloud pour l'accessibilité locale essentielle.
- Pas de fermeture de service critique sans analyse de dépendances.
- Chaque fonctionnalité expose ses besoins, privilèges, données collectées, flux réseau et mode de désactivation.
- Chaque risque connu est classé, affecté à un responsable, testé et réévalué à chaque release.

## Références primaires

- NIST SP 800-207, Zero Trust Architecture : https://csrc.nist.gov/pubs/sp/800/207/final
- NIST SP 800-218, Secure Software Development Framework (SSDF).
- NIST CSF 2.0, gouvernance et gestion du risque cyber.
- IETF RFC 8386, considérations de confidentialité des protocoles reposant sur broadcast/multicast : https://www.rfc-editor.org/info/rfc8386/
- Documentation officielle des pare-feu, piles réseau, protocoles et mécanismes de démarrage de chaque OS ciblé.

Les références ne sont pas une certification. Cette feuille de route ne déclare pas les défenses implémentées : chaque capacité doit être reliée à un composant, à des tests et à des preuves.
