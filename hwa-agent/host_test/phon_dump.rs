// Affiche les phonemes produits par voix::phonemes pour une liste de phrases de reference.
// Compilation : rustc --edition 2024 host_test/phon_dump.rs -o target/phon_dump.exe
#[path = "../src/voix.rs"]
mod voix;

fn main() {
    let phrases = [
        "Bonjour",
        "Le lecteur d'écran est prêt.",
        "Titre : menu principal.",
        "Invite : tapez une commande.",
        "Résultat : opération terminée.",
        "Erreur : fichier introuvable.",
        "Les enfants mangent des pommes.",
        "Il est six heures.",
        "Un chat, deux chiens, trois oiseaux.",
        "Vous êtes très gentil.",
        "Je suis content aujourd'hui.",
        "Nous avons fini le travail.",
    ];
    for p in phrases {
        let mut out = [0u8; 600];
        let n = voix::phonemes(p.as_bytes(), &mut out);
        let s: String = out[..n].iter().map(|&c| c as char).collect();
        println!("{p}\n  -> {s}");
    }
}
