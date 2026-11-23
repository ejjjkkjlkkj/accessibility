//! Usage : voxa-piper --piper <binaire> --modele <voix.onnx> --sortie <fichier.wav> "texte"
use std::path::PathBuf;

fn main() {
    let mut piper = PathBuf::from("piper");
    let mut modele = None;
    let mut sortie = PathBuf::from("voxa.wav");
    let mut texte = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--piper" => piper = PathBuf::from(args.next().unwrap_or_default()),
            "--modele" => modele = args.next().map(PathBuf::from),
            "--sortie" => sortie = PathBuf::from(args.next().unwrap_or_default()),
            other => texte = Some(other.to_string()),
        }
    }
    let Some(modele) = modele else { eprintln!("--modele requis"); std::process::exit(2) };
    let Some(texte) = texte else { eprintln!("texte requis"); std::process::exit(2) };
    match voxa_piper::synthetiser(&piper, &modele, &texte, &sortie) {
        Ok(()) => println!("WAV écrit : {}", sortie.display()),
        Err(e) => { eprintln!("erreur : {e}"); std::process::exit(1) }
    }
}
