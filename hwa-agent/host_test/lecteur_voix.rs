// Lecteur d'ecran (option 2) : lit messages.txt (une phrase par ligne), genere chaque phrase
// avec src/voix.rs (notre synthese) en WAV 16 kHz mono, puis la joue par l'API audio de Windows.
// Compilation : rustc --edition 2024 host_test/lecteur_voix.rs -o target/lecteur_voix.exe
#[path = "../src/voix.rs"]
mod voix;

use std::os::windows::ffi::OsStrExt;

#[link(name = "winmm")]
unsafe extern "system" {
    fn PlaySoundW(pszsound: *const u16, hmod: *mut core::ffi::c_void, fdwsound: u32) -> i32;
}
const SND_FILENAME: u32 = 0x0002_0000;
const SND_SYNC: u32 = 0x0000_0000;

fn wav(nom: &str, pcm: &[i16]) {
    let data = pcm.len() * 2;
    let mut f: Vec<u8> = Vec::with_capacity(44 + data);
    f.extend(b"RIFF");
    f.extend(((36 + data) as u32).to_le_bytes());
    f.extend(b"WAVEfmt ");
    f.extend(16u32.to_le_bytes());
    f.extend(1u16.to_le_bytes());
    f.extend(1u16.to_le_bytes());
    f.extend(16000u32.to_le_bytes());
    f.extend(32000u32.to_le_bytes());
    f.extend(2u16.to_le_bytes());
    f.extend(16u16.to_le_bytes());
    f.extend(b"data");
    f.extend((data as u32).to_le_bytes());
    for s in pcm {
        f.extend(s.to_le_bytes());
    }
    std::fs::write(nom, f).unwrap();
}

fn jouer(nom: &str) {
    let chemin: Vec<u16> = std::ffi::OsStr::new(nom).encode_wide().chain(std::iter::once(0)).collect();
    // SAFETY: appel a l'API audio de Windows avec une chaine UTF-16 terminee par zero.
    unsafe {
        PlaySoundW(chemin.as_ptr(), std::ptr::null_mut(), SND_FILENAME | SND_SYNC);
    }
}

fn main() {
    let texte = std::fs::read_to_string("messages.txt").unwrap_or_default();
    for (i, ligne) in texte.lines().enumerate() {
        let ligne = ligne.trim();
        if ligne.is_empty() {
            continue;
        }
        let mut pcm = vec![0i16; 16000 * 12];
        let n = voix::dire(ligne.as_bytes(), &mut pcm);
        let nom = format!("msg_{:02}.wav", i + 1);
        wav(&nom, &pcm[..n]);
        println!("{nom} : {ligne}");
        jouer(&nom);
    }
}
