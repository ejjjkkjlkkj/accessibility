//! Phrases ecrites par nous pour le modele d'intentions. Trois ensembles DISJOINTS : apprentissage (TR), validation (VA : choix du seuil), test (TE : mesure finale, touche une seule fois).
//! Provenance : redigees par l'assistant IA du projet, non relues par des francophones (limite connue). Index = numero d'Intention (0 = inconnu).
pub const TR: [&[&str]; 15] = [
    &["il fait beau aujourd'hui", "raconte-moi une blague", "quel temps fait-il", "qui es-tu", "bonjour", "je voudrais manger une pizza", "combien font deux plus deux", "chante une chanson", "où est la gare", "j'aime la musique", "mon chat dort", "demain je pars en voyage", "ouvre la porte", "achète du pain", "la lune est ronde", "ma mémoire me joue des tours", "la mémoire de l'éléphant est légendaire", "le bus est en retard", "l'ordinateur de mon voisin est vert", "il est tard", "bon appétit", "quel film regarder ce soir", "je cherche mes clés", "traduis ce mot en anglais", "mon frère travaille à paris", "la soupe est chaude", "allume la lumière", "quel est ton nom", "raconte une histoire de dragon", "comment faire un gâteau au chocolat", "combien coûte un billet de train", "les enfants jouent dans le jardin", "je suis fatigué", "quelle est la meilleure saison", "mets de la musique", "où habites-tu", "le match commence à vingt heures", "dessine un cercle", "calcule dix fois douze", "pourquoi le ciel est bleu", "apporte-moi un verre d'eau", "j'ai perdu mon téléphone", "le train part demain", "va voir ce qui se passe", "ça sent bon dans la cuisine", "donne-moi un conseil", "bonne nuit", "à bientôt", "lis-moi un conte", "la porte est fermée"],
    &["aide", "que peux-tu faire", "quelles sont tes commandes", "comment ça marche", "dis-moi ce que tu sais faire", "liste des commandes", "help", "à quoi tu sers", "j'ai besoin d'aide", "montre les possibilités"],
    &["informations sur le matériel", "quel est mon processeur", "quelle machine est-ce", "décris l'ordinateur", "donne-moi les caractéristiques de la machine", "quel modèle d'ordinateur", "dis-moi le processeur", "fiche technique", "identité de la machine", "quelle carte mère"],
    &["combien de mémoire", "mémoire libre", "quelle quantité de ram", "combien de mégaoctets disponibles", "état de la mémoire", "la ram", "taille de la mémoire vive", "mémoire disponible", "combien de gigaoctets de mémoire", "est-ce qu'il reste de la mémoire"],
    &["liste des périphériques", "quels périphériques sont branchés", "bus pci", "montre les composants", "quels sont les périphériques pci", "énumère le matériel connecté", "composants de la carte mère", "inventaire des périphériques", "liste le bus", "combien de périphériques"],
    &["y a-t-il du son", "contrôleur audio", "carte son", "combien de contrôleurs audio", "le son fonctionne-t-il", "périphérique audio", "carte audio présente", "est-ce que j'ai une sortie audio", "infos sur l'audio", "haut-parleurs"],
    &["lis la configuration du bios", "ouvre le bios", "montre les réglages du bios", "lire le bios", "afficher les paramètres du firmware", "configuration du bios", "parcourir le bios", "ouvre les écrans du bios", "lecture du bios", "écrans de configuration"],
    &["quelle est la valeur de fast boot dans le bios", "cherche secure boot dans le bios", "le démarrage rapide est-il activé", "est-ce que l'usb est activé dans le bios", "trouve l'option boot du bios", "que dit le bios sur le secure boot", "recherche dans le bios l'audio", "valeur par défaut de fast boot", "option usb support du bios", "le secure boot est il actif"],
    &["éteins l'ordinateur", "arrête la machine", "éteindre", "coupe l'alimentation", "shutdown", "mets l'ordinateur hors tension", "j'aimerais éteindre", "arrêt de l'ordinateur", "éteignez la machine", "fais un arrêt complet"],
    &["redémarre l'ordinateur", "relance la machine", "redémarrer", "reboot", "fais un redémarrage", "recommence le démarrage", "redémarrage de l'ordinateur", "relancer le système", "redémarrez la machine", "reboote"],
    &["répète", "peux-tu répéter", "redis-le", "encore une fois", "répète la dernière phrase", "je n'ai pas compris", "dis-le encore", "répétition", "pardon", "recommence ta phrase"],
    &["quelle heure est-il", "donne-moi l'heure", "l'heure", "il est quelle heure", "dis l'heure", "heure actuelle", "quelle est l'heure", "l'heure s'il te plaît", "peux-tu me dire l'heure", "date et heure"],
    &["test du son", "fais un bip", "teste le haut-parleur", "joue un son", "essai sonore", "émets un bip", "fais du bruit", "test audio", "bip de test", "vérifie le son"],
    &["quitte l'assistant", "retour au menu", "sortir", "fin", "quitter", "retourne au menu principal", "ferme l'assistant", "c'est tout merci", "au revoir", "arrête l'assistant"],
    &["liste les tables acpi", "tables acpi", "quelles tables acpi", "montre les tables du firmware", "combien de tables acpi", "acpi", "affiche acpi", "tables d'alimentation", "détails acpi", "tables de description du système"],
];
pub const VA: [&[&str]; 15] = [
    &["le ciel est bleu", "écris une histoire", "j'ai faim", "dors bien"],
    &["que sais-tu faire", "affiche l'aide", "donne-moi un coup de main"],
    &["parle-moi de la machine", "quel processeur est installé", "donne les infos du pc"],
    &["quelle taille de mémoire", "montre la ram libre", "mémoire utilisable"],
    &["affiche les périphériques", "détaille les composants", "quels appareils sont présents"],
    &["quel contrôleur audio", "montre la carte son", "est-ce qu'il y a de l'audio"],
    &["affiche le bios", "ouvre les réglages du firmware", "je veux lire le bios"],
    &["que vaut fast boot", "le bios dit quoi pour l'usb", "cherche tpm dans le firmware"],
    &["arrête l'ordinateur maintenant", "éteins tout", "coupe la machine"],
    &["relance l'ordinateur", "redémarre maintenant", "fais un reboot"],
    &["redis ta phrase", "répète encore", "comment ?"],
    &["donne l'heure", "il est quelle heure déjà", "heure s'il te plaît"],
    &["fais un test de son", "joue un bip", "essaie le haut-parleur"],
    &["retour", "quitte", "termine"],
    &["liste acpi", "donne les tables acpi", "affiche les tables acpi"],
];
/// Jeu de test v2 : ecrit APRES la lecture des erreurs du v1 (brule, archive dans donnees.rs.v1-test-brule), avec d'autres tournures. Mesure finale unique.
pub const TE: [&[&str]; 15] = [
    &["prépare-moi un café", "il y a du monde dans la rue", "peins un tableau", "quel âge as-tu", "les oiseaux chantent le matin", "pose ton sac ici", "je m'ennuie un peu", "dis bonjour à ta mère"],
    &["tu peux faire quoi exactement", "montre-moi les fonctions disponibles", "guide-moi", "comment on s'en sert", "que proposes-tu", "donne la liste de tes talents"],
    &["parle-moi du modèle de ce portable", "quelle marque est cet ordinateur", "donne-moi le nom du processeur", "décris cette machine", "détails de la carte mère", "quel est ce matériel"],
    &["combien reste-t-il de ram", "mémoire vive installée", "dis-moi la mémoire libre", "quelle mémoire est utilisable", "capacité de la ram", "est-ce qu'il y a beaucoup de mémoire"],
    &["quels sont les appareils connectés", "dresse la liste des composants", "montre le bus de la carte mère", "énumère les périphériques du système", "quels périphériques existent", "recense le matériel"],
    &["y a-t-il une carte son", "le contrôleur audio est-il présent", "informations sur le son", "est-ce que l'audio marche", "liste le matériel audio", "dis-moi pour la sortie son"],
    &["peux-tu lire la configuration du bios", "montre-moi le menu du bios", "ouvre l'interface du firmware", "affiche les réglages uefi", "je voudrais consulter le bios", "fais défiler les options du bios"],
    &["dans le bios, fast boot est-il activé", "que dit le firmware sur secure boot", "quelle est la valeur de usb support", "regarde l'option csm du bios", "trouve-moi tpm dans le bios", "le démarrage rapide est-il actif dans le firmware"],
    &["peux-tu arrêter l'ordinateur", "éteins le pc maintenant", "coupe tout", "il faut éteindre la machine", "mets l'ordinateur à l'arrêt", "extinction de la machine"],
    &["peux-tu relancer l'ordinateur", "redémarre le pc maintenant", "il faut redémarrer la machine", "fais redémarrer le système", "relance tout", "reboote le pc"],
    &["peux-tu redire cela", "redis la dernière réponse", "je n'ai rien entendu", "répète s'il te plaît", "encore une fois ta réponse", "recommence"],
    &["peux-tu me donner l'heure", "quelle heure est-il maintenant", "dis-moi l'heure exacte", "l'heure actuelle s'il te plaît", "il est quelle heure là", "annonce l'heure"],
    &["fais-moi entendre un bip", "teste le son du haut-parleur", "joue un bruit de test", "émets un son", "bip pour vérifier", "essaie un son"],
    &["j'ai terminé merci", "reviens au menu", "quitte maintenant", "tu peux fermer l'assistant", "c'est fini", "sors de l'assistant"],
    &["liste moi les tables acpi", "quelles sont les tables acpi du firmware", "affiche toutes les tables acpi", "combien y a-t-il de tables acpi", "montre acpi", "donne le détail des tables acpi"],
];
pub const PREFIXES_TR: [&str; 8] = ["s'il te plaît", "dis-moi", "peux-tu", "j'aimerais", "je voudrais", "merci", "please", "vite"];
pub const PREFIXES_TE: [&str; 7] = ["stp", "euh", "voyons", "alors", "hé", "au fait", "ok"];
