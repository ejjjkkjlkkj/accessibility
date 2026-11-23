// Test hote de src/voix.rs (code pur). Ecrit aussi des fichiers WAV a ecouter sous Windows.
// Compilation : rustc --edition 2024 --test host_test/voix_test.rs -o target/voix_test.exe
#[path = "../src/voix.rs"]
mod voix;

fn ph(t: &str) -> String {
    let mut b = [0u8; 600];
    let n = voix::phonemes(t.as_bytes(), &mut b);
    String::from_utf8_lossy(&b[..n]).trim().to_string()
}

fn wav(nom: &str, pcm: &[i16]) {
    let mut f = Vec::new();
    let n = (pcm.len() * 2) as u32;
    f.extend_from_slice(b"RIFF");
    f.extend_from_slice(&(36 + n).to_le_bytes());
    f.extend_from_slice(b"WAVEfmt ");
    f.extend_from_slice(&16u32.to_le_bytes());
    f.extend_from_slice(&1u16.to_le_bytes());
    f.extend_from_slice(&1u16.to_le_bytes());
    f.extend_from_slice(&16000u32.to_le_bytes());
    f.extend_from_slice(&32000u32.to_le_bytes());
    f.extend_from_slice(&2u16.to_le_bytes());
    f.extend_from_slice(&16u16.to_le_bytes());
    f.extend_from_slice(b"data");
    f.extend_from_slice(&n.to_le_bytes());
    for s in pcm {
        f.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::create_dir_all("voix_wav").unwrap();
    std::fs::write(format!("voix_wav/{nom}.wav"), f).unwrap();
}

/// Energie d'une frequence dans un signal (algorithme de Goertzel).
fn energie(pcm: &[i16], f: f64) -> f64 {
    let w = 2.0 * std::f64::consts::PI * f / 16000.0;
    let c = 2.0 * w.cos();
    let (mut s1, mut s2) = (0.0, 0.0);
    for &x in pcm {
        let s0 = x as f64 + c * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - c * s1 * s2) / pcm.len() as f64
}
/// Energie d'une bande : somme sur les harmoniques proches.
fn bande(pcm: &[i16], f0: f64, f1: f64) -> f64 {
    let mut e = 0.0;
    let mut f = f0;
    while f <= f1 {
        e += energie(pcm, f);
        f += 20.0;
    }
    e
}

#[test]
fn prononciation() {
    for (t, p) in [
        ("bonjour", "bCZuR"),
        ("lecteur d'écran", "lEkt2R dekRA"),
        ("clavier", "klavje"),
        ("analyse terminée", "analiz tERmine"),
        ("le disque est lu", "l2 disk E ly"),
        ("mémoire", "memwaR"),
        ("processeur", "pROsEs2R"),
        ("attention", "atAsjC"),
        ("un chien", "I SjI"),
        ("la fille travaille", "la fij tRavaj"),
        ("les touches", "le tuS"),
        ("aucune erreur", "okyn ER2R"),
        ("fin du rapport", "fI dy RapOR"),
        ("oui", "wi"),
        ("haut-parleur", "o paRl2R"),
        ("quatre", "katR"),
        ("langue", "lAg"),
    ] {
        assert_eq!(ph(t), p, "texte : {t}");
    }
}

#[test]
fn sigles_et_nombres() {
    assert_eq!(ph("USB"), "y Es be");
    assert_eq!(ph("42"), "katR d2", "42");
    assert!(ph("0x7A").starts_with("zeRo iks"), "{}", ph("0x7A"));
    assert_eq!(ph("a, b."), "a _be  .");
}

#[test]
fn entrees_hostiles() {
    let mut b = [0u8; 600];
    let mut pcm = vec![0i16; 16000];
    for t in [&b""[..], b"'", b"''''", b"\xC3", b"\xC3\xA9", b"\xFF\xFE\xF0", b"e", b"x", b"qu'", b"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"] {
        let n = voix::phonemes(t, &mut b);
        let _ = voix::synthese(&b[..n], &mut pcm);
    }
    // Tampon de sortie trop petit : jamais de depassement.
    let mut petit = [0u8; 3];
    assert!(voix::phonemes(b"bonjour tout le monde", &mut petit) <= 3);
    let mut court = [0i16; 100];
    assert!(voix::dire(b"bonjour", &mut court) <= 100);
}

#[test]
fn son_des_voyelles() {
    // Chaque voyelle tenue doit avoir son energie autour de ses formants (reglage du synthetiseur).
    let mut pcm = vec![0i16; 16000];
    let n = voix::synthese(b"aaaa", &mut pcm);
    let a = &pcm[2000..n - 1000];
    let (bas, f1) = (bande(a, 200.0, 400.0), bande(a, 650.0, 850.0));
    assert!(f1 > 2.0 * bas, "a : F1 {f1} bas {bas}");
    let n = voix::synthese(b"iiii", &mut pcm);
    let i = &pcm[2000..n - 1000];
    let (f1, milieu, f2) = (bande(i, 200.0, 400.0), bande(i, 900.0, 1500.0), bande(i, 2100.0, 2400.0));
    assert!(f1 > 2.0 * milieu, "i : F1 {f1} milieu {milieu}");
    assert!(f2 > milieu / 4.0, "i : F2 {f2} milieu {milieu}");
    // s : bruit aigu, sans voix.
    let n = voix::synthese(b"ssss", &mut pcm);
    let s = &pcm[1000..n - 800];
    assert!(bande(s, 4500.0, 6500.0) > 10.0 * bande(s, 200.0, 1200.0));
}

#[test]
fn niveau_et_fichiers() {
    let phrases = [
        ("01_bonjour", "Bonjour. Le lecteur d'écran est prêt."),
        ("02_analyse", "Analyse terminée. Aucune erreur."),
        ("03_touches", "Flèche bas : ligne suivante. Flèche haut : ligne précédente. Échap : quitter."),
        ("04_rapport", "Processeur AMD Ryzen 7. Mémoire : 16 giga. Disque NVMe présent. Clavier USB trouvé."),
        ("05_alphabet", "a b c d e f g h i j k l m n o p q r s t u v w x y z"),
        ("06_chiffres", "0 1 2 3 4 5 6 7 8 9"),
    ];
    for (nom, t) in phrases {
        let mut pcm = vec![0i16; 16000 * 20];
        let n = voix::dire(t.as_bytes(), &mut pcm);
        let p = &pcm[..n];
        let pic = p.iter().map(|x| x.unsigned_abs()).max().unwrap();
        let sature = p.iter().filter(|x| x.unsigned_abs() >= 32000).count();
        let rms = (p.iter().map(|&x| (x as f64) * (x as f64)).sum::<f64>() / n as f64).sqrt();
        println!("{nom} : {:.2} s, pic {pic}, rms {rms:.0}, satures {sature}", n as f64 / 16000.0);
        assert!(n > 8000 && pic > 6000, "{nom} trop faible");
        assert!(sature * 1000 < n, "{nom} sature : {sature} sur {n}");
        wav(nom, p);
    }
}

#[test]
fn niveaux_par_phoneme() {
    for p in b"ieEaOouy2@ACIjwHlmnRfsSvzZptkbdg" {
        let mut pcm = vec![0i16; 16000];
        let n = voix::synthese(&[b'a', *p, *p, b'a'], &mut pcm);
        let seg = &pcm[2200..n.min(2200 + 2400)];
        let pic = seg.iter().map(|x| x.unsigned_abs()).max().unwrap();
        println!("  {} pic {pic}", *p as char);
    }
}
