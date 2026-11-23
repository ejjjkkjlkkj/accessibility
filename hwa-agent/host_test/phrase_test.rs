// Ecrit phrase_lecteur.wav : la phrase du lecteur d'ecran, pour ecoute (16 kHz mono).
// Compilation : rustc --edition 2024 host_test/phrase_test.rs -o target/phrase.exe
#[path = "../src/voix.rs"]
mod voix;

fn wav(nom: &str, pcm: &[i16]) {
    let data = pcm.len() * 2;
    let mut f: Vec<u8> = Vec::with_capacity(44 + data);
    f.extend(b"RIFF");
    f.extend(((36 + data) as u32).to_le_bytes());
    f.extend(b"WAVEfmt ");
    f.extend(16u32.to_le_bytes());
    f.extend(1u16.to_le_bytes()); // PCM
    f.extend(1u16.to_le_bytes()); // mono
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

fn main() {
    let mut pcm = vec![0i16; 16000 * 12];
    let n = voix::dire("Bonjour. Le lecteur d'écran est prêt.".as_bytes(), &mut pcm);
    wav("phrase_lecteur.wav", &pcm[..n]);
    println!("phrase_lecteur.wav : {} echantillons", n);
}
