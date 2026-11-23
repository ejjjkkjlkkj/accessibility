//! Adaptateur Piper pour Voxa.
//!
//! Piper (licence MIT) est appelé comme programme séparé : son binaire n'est pas embarqué dans ce dépôt.
//! Le moteur de phonétisation espeak-ng (GPL-3), livré avec Piper, reste donc hors de ce code MIT.
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

/// Synthétise `texte` avec le binaire `piper` et le modèle `modele`, et écrit un fichier WAV dans `sortie`.
pub fn synthetiser(piper: &Path, modele: &Path, texte: &str, sortie: &Path) -> io::Result<()> {
    let mut enfant = Command::new(piper)
        .arg("--model").arg(modele)
        .arg("--output_file").arg(sortie)
        .stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null())
        .spawn()?;
    enfant.stdin.take().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "entrée standard indisponible"))?
        .write_all(texte.as_bytes())?;
    let etat = enfant.wait()?;
    if !etat.success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("piper a échoué : {etat}")));
    }
    if std::fs::metadata(sortie)?.len() < 44 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "WAV vide"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binaire_absent_donne_une_erreur() {
        let r = synthetiser(Path::new("/chemin/inexistant/piper"), Path::new("m.onnx"), "bonjour", Path::new("/tmp/x.wav"));
        assert!(r.is_err());
    }
}
