// Voix du lecteur d'ecran : texte francais -> phonemes -> son, par synthese a formants.
// Code entierement a nous : aucune bibliotheque, aucune donnee de voix d'un tiers, aucun appel
// au firmware, pas d'allocation, pas de virgule flottante (entiers seulement).
// Principe (acoustique publique) : une source (impulsions periodiques pour la voix, bruit pour
// les consonnes sourdes) passe dans des resonateurs a deux poles regles sur les formants.
//   y[n] = A x[n] + B y[n-1] + C y[n-2],  C = -r^2,  B = 2 r cos(2 pi F / fe),  A = 1 - B - C
// Sortie : echantillons 16 bits signes, mono, 16 000 Hz.
//
// Phonemes (un octet chacun) :
//   voyelles  i e(e ferme) E(e ouvert) a O(o ouvert) o u(ou) y(u) 2(eu) @(e muet) A(an) C(on) I(in)
//   consonnes p t k b d g f s S(ch) v z Z(j) m n l R j(y) w H(ui)
//   pauses    ' ' (courte) '_' (virgule) '.' (fin de phrase)

pub const FE: usize = 16000;

// ---------------------------------------------------------------------------------------------
// 1. Texte -> phonemes
// ---------------------------------------------------------------------------------------------

struct Sortie<'a> {
    b: &'a mut [u8],
    n: usize,
}
impl Sortie<'_> {
    fn p(&mut self, c: u8) {
        if self.n < self.b.len() {
            self.b[self.n] = c;
            self.n += 1;
        }
    }
    fn s(&mut self, t: &[u8]) {
        for &c in t {
            self.p(c);
        }
    }
}

// Lettres internes : a-z, chiffres, et 1 = e accent aigu, 2 = e accent grave ou circonflexe,
// 3 = c cedille, 4 = o circonflexe, 5 = oe lie.
fn voy(c: u8) -> bool {
    matches!(c, b'a' | b'e' | b'i' | b'o' | b'u' | b'y' | b'1' | b'2' | b'4' | b'5')
}
fn e_i(c: u8) -> bool {
    matches!(c, b'e' | b'i' | b'y' | b'1' | b'2')
}
fn muette_finale(c: u8) -> bool {
    matches!(c, b's' | b't' | b'd' | b'x' | b'z' | b'p')
}

fn nom_lettre(c: u8) -> &'static [u8] {
    match c {
        b'a' => b"a",
        b'b' => b"be",
        b'c' => b"se",
        b'd' => b"de",
        b'e' => b"2",
        b'f' => b"Ef",
        b'g' => b"Ze",
        b'h' => b"aS",
        b'i' => b"i",
        b'j' => b"Zi",
        b'k' => b"ka",
        b'l' => b"El",
        b'm' => b"Em",
        b'n' => b"En",
        b'o' => b"o",
        b'p' => b"pe",
        b'q' => b"ky",
        b'r' => b"ER",
        b's' => b"Es",
        b't' => b"te",
        b'u' => b"y",
        b'v' => b"ve",
        b'w' => b"dubl@ve",
        b'x' => b"iks",
        b'y' => b"igREk",
        b'z' => b"zEd",
        _ => b"",
    }
}
fn nom_chiffre(c: u8) -> &'static [u8] {
    match c {
        b'0' => b"zeRo",
        b'1' => b"I",
        b'2' => b"d2",
        b'3' => b"tRwa",
        b'4' => b"katR",
        b'5' => b"sIk",
        b'6' => b"sis",
        b'7' => b"sEt",
        b'8' => b"Hit",
        b'9' => b"n2f",
        _ => b"",
    }
}

/// Mots dont la prononciation ne suit pas les regles (liste courte, a completer a l'usage).
fn exception(m: &[u8]) -> Option<&'static [u8]> {
    Some(match m {
        b"et" => b"e",
        b"est" => b"E",
        b"test" => b"tEst",
        b"ouest" => b"wEst",
        b"un" => b"I",
        b"six" => b"sis",
        b"dix" => b"dis",
        b"sept" => b"sEt",
        b"huit" => b"Hit",
        b"plus" => b"plys",
        b"bus" => b"bys",
        b"ok" => b"oke",
        b"clef" => b"kle",
        b"ecran" => b"ekRA",
        b"femme" => b"fam",
        b"monsieur" => b"m@sj2",
        b"bios" => b"bjOs",
        _ => return None,
    })
}

/// Sigles toujours epeles.
fn sigle(m: &[u8]) -> bool {
    matches!(
        m,
        b"usb" | b"hda" | b"pci" | b"nvme" | b"acpi" | b"cpu" | b"xhci" | b"msr" | b"uefi" | b"efi" | b"gop" | b"hid" | b"os" | b"id"
    )
}

fn epeler(m: &[u8], chiffres_internes: bool, o: &mut Sortie) {
    for &c in m {
        if c.is_ascii_digit() && chiffres_internes {
            o.s(nom_chiffre(c));
        } else {
            // Les codes internes 1..5 ne se rencontrent ici que comme lettres accentuees.
            let l = match c {
                b'1' | b'2' => b'e',
                b'3' => b'c',
                b'4' => b'o',
                b'5' => b'e',
                x => x,
            };
            o.s(nom_lettre(l));
        }
        o.p(b' ');
    }
}

/// Un mot (lettres internes uniquement) -> phonemes, par regles d'orthographe francaise.
fn mot(w: &[u8], o: &mut Sortie) {
    let n = w.len();
    let at = |k: usize| -> u8 { if k < n { w[k] } else { 0 } };
    // Vraie si, a partir de k, il ne reste que des consonnes finales muettes (ou rien).
    let reste_muet = |k: usize| -> bool { (k..n).all(|j| muette_finale(w[j])) };
    // n ou m a l'indice k nasalise la voyelle precedente : pas suivi d'une voyelle ni d'un n/m.
    let nasal = |k: usize| -> bool {
        let x = at(k + 1);
        matches!(at(k), b'n' | b'm') && !voy(x) && x != b'n' && x != b'm'
    };
    let mut vue = false; // une voyelle a deja ete prononcee dans ce mot
    let mut i = 0usize;
    while i < n {
        let c = w[i];
        let x = at(i + 1);
        let x2 = at(i + 2);
        let prec = if i > 0 { w[i - 1] } else { 0 };
        let fin = i + 1 == n;
        let avant = o.n;
        match c {
            b'a' => {
                if x == b'i' && nasal(i + 2) {
                    o.p(b'I');
                    i += 3;
                } else if x == b'i' && x2 == b'l' && (at(i + 3) == 0 || at(i + 3) == b'l') {
                    o.s(b"aj");
                    i += if at(i + 3) == b'l' { 4 } else { 3 };
                } else if x == b'i' || x == b'y' && !voy(x2) {
                    o.p(b'E');
                    i += 2;
                } else if x == b'u' {
                    o.p(b'o');
                    i += 2;
                } else if nasal(i + 1) {
                    o.p(b'A');
                    i += 2;
                } else {
                    o.p(b'a');
                    i += 1;
                }
            }
            b'e' => {
                if x == b'a' && x2 == b'u' {
                    o.p(b'o');
                    i += 3;
                } else if x == b'i' && nasal(i + 2) {
                    o.p(b'I');
                    i += 3;
                } else if x == b'i' && x2 == b'l' && (at(i + 3) == 0 || at(i + 3) == b'l') {
                    o.s(b"Ej");
                    i += if at(i + 3) == b'l' { 4 } else { 3 };
                } else if x == b'i' {
                    o.p(b'E');
                    i += 2;
                } else if x == b'u' {
                    o.p(b'2');
                    i += 2;
                } else if x == b'n' && x2 == b't' && i + 3 == n && n > 4 {
                    o.p(b'A');
                    i = n;
                } else if nasal(i + 1) {
                    o.p(if prec == b'i' || prec == b'y' { b'I' } else { b'A' });
                    i += 2;
                } else if fin {
                    if !vue {
                        o.p(b'2');
                    }
                    i += 1;
                } else if x == b's' && i + 2 == n {
                    if !vue {
                        o.p(b'e');
                    }
                    i = n;
                } else if (x == b'r' && n > 3 || x == b'z' || x == b'd') && i + 2 == n {
                    o.p(b'e');
                    i = n;
                } else if x == b't' && i + 2 == n {
                    o.p(b'E');
                    i = n;
                } else if !voy(x) && (x2 == 0 || !voy(x2)) {
                    // e devant deux consonnes ou devant une consonne finale : e ouvert,
                    // sauf devant consonne + r ou l (secret, reflet) : e muet.
                    if matches!(x2, b'r' | b'l') && !matches!(x, b'r' | b'l') && at(i + 3) != 0 {
                        o.p(b'@');
                    } else {
                        o.p(b'E');
                    }
                    i += 1;
                } else {
                    o.p(b'@');
                    i += 1;
                }
            }
            b'1' => {
                o.p(b'e');
                i += 1;
            }
            b'2' => {
                o.p(b'E');
                i += 1;
            }
            b'5' => {
                o.p(b'2');
                i += if x == b'u' { 2 } else { 1 };
            }
            b'i' => {
                if x == b'l' && x2 == b'l' {
                    o.s(if voy(prec) { b"j" } else { b"ij" });
                    i += 3;
                } else if x == b'l' && i + 2 == n && voy(prec) {
                    o.p(b'j');
                    i += 2;
                } else if nasal(i + 1) {
                    o.p(b'I');
                    i += 2;
                } else if i > 0 && matches!(x, b'a' | b'e' | b'o' | b'u' | b'1' | b'2') && !(x == b'e' && i + 2 == n) {
                    o.p(b'j');
                    i += 1;
                } else {
                    o.p(b'i');
                    i += 1;
                }
            }
            b'o' => {
                if x == b'i' && nasal(i + 2) {
                    o.s(b"wI");
                    i += 3;
                } else if x == b'i' {
                    o.s(b"wa");
                    i += 2;
                } else if x == b'y' {
                    o.s(b"wa");
                    i += 1;
                } else if x == b'e' && x2 == b'u' {
                    o.p(b'2');
                    i += 3;
                } else if x == b'u' {
                    if voy(x2) && !(x2 == b'e' && i + 3 == n) {
                        o.p(b'w');
                    } else {
                        o.p(b'u');
                    }
                    i += 2;
                } else if nasal(i + 1) {
                    o.p(b'C');
                    i += 2;
                } else if reste_muet(i + 1) || x == b's' && voy(x2) {
                    o.p(b'o');
                    i += 1;
                } else {
                    o.p(b'O');
                    i += 1;
                }
            }
            b'4' => {
                o.p(b'o');
                i += 1;
            }
            b'u' => {
                if nasal(i + 1) {
                    o.p(b'I');
                    i += 2;
                } else if x == b'i' {
                    o.s(b"Hi");
                    i += 2;
                } else {
                    o.p(b'y');
                    i += 1;
                }
            }
            b'y' => {
                if voy(x) {
                    o.p(b'j');
                    i += 1;
                } else if nasal(i + 1) {
                    o.p(b'I');
                    i += 2;
                } else {
                    o.p(b'i');
                    i += 1;
                }
            }
            b'c' => {
                if x == b'h' {
                    o.p(if matches!(x2, b'r' | b'n' | b'l') { b'k' } else { b'S' });
                    i += 2;
                } else if e_i(x) {
                    o.p(b's');
                    i += 1;
                } else if x == b'c' && e_i(x2) {
                    o.s(b"ks");
                    i += 2;
                } else if x == b'c' || x == b'k' || x == b'q' {
                    o.p(b'k');
                    i += 2;
                } else {
                    o.p(b'k');
                    i += 1;
                }
            }
            b'3' => {
                o.p(b's');
                i += 1;
            }
            b'g' => {
                if x == b'n' {
                    o.s(b"nj");
                    i += 2;
                } else if x == b'u' && e_i(x2) {
                    o.p(b'g');
                    i += 2;
                } else if e_i(x) {
                    o.p(b'Z');
                    i += 1;
                } else if fin || reste_muet(i + 1) && i > 0 {
                    i += 1;
                } else {
                    o.p(b'g');
                    i += if x == b'g' { 2 } else { 1 };
                }
            }
            b'h' => i += 1,
            b'j' => {
                o.p(b'Z');
                i += 1;
            }
            b'p' => {
                if x == b'h' {
                    o.p(b'f');
                    i += 2;
                } else if reste_muet(i) && i > 0 {
                    i = n;
                } else {
                    o.p(b'p');
                    i += if x == b'p' { 2 } else { 1 };
                }
            }
            b'q' => {
                o.p(b'k');
                i += if x == b'u' { 2 } else { 1 };
            }
            b's' => {
                if reste_muet(i) && i > 0 {
                    i = n;
                } else if x == b's' {
                    o.p(b's');
                    i += 2;
                } else if x == b'c' && e_i(x2) {
                    o.p(b's');
                    i += 2;
                } else if voy(prec) && voy(x) {
                    o.p(b'z');
                    i += 1;
                } else {
                    o.p(b's');
                    i += 1;
                }
            }
            b't' => {
                if reste_muet(i) && i > 0 {
                    i = n;
                } else if x == b'i' && x2 == b'o' && at(i + 3) == b'n' && prec != b's' {
                    o.p(b's');
                    i += 1;
                } else if x == b'h' || x == b't' {
                    o.p(b't');
                    i += 2;
                } else {
                    o.p(b't');
                    i += 1;
                }
            }
            b'd' | b'z' => {
                if reste_muet(i) && i > 0 {
                    i = n;
                } else {
                    o.p(c);
                    i += if x == c { 2 } else { 1 };
                }
            }
            b'x' => {
                if reste_muet(i) && i > 0 {
                    i = n;
                } else if i == 1 && w[0] == b'e' && voy(x) {
                    o.s(b"gz");
                    i += 1;
                } else {
                    o.s(b"ks");
                    i += 1;
                }
            }
            b'r' => {
                o.p(b'R');
                i += if x == b'r' { 2 } else { 1 };
            }
            b'b' | b'f' | b'k' | b'l' | b'm' | b'n' | b'v' => {
                o.p(c);
                i += if x == c { 2 } else { 1 };
            }
            b'w' => {
                o.p(b'w');
                i += 1;
            }
            _ => i += 1,
        }
        if o.b[avant..o.n].iter().any(|&p| est_voyelle(p)) {
            vue = true;
        }
    }
}

fn est_voyelle(p: u8) -> bool {
    matches!(p, b'i' | b'e' | b'E' | b'a' | b'O' | b'o' | b'u' | b'y' | b'2' | b'@' | b'A' | b'C' | b'I')
}

/// Un groupe de lettres et de chiffres sans espace -> phonemes.
fn jeton(m: &[u8], o: &mut Sortie) {
    if m.is_empty() {
        return;
    }
    if let Some(p) = exception(m) {
        o.s(p);
    } else if sigle(m) || !m.iter().any(|&c| voy(c)) || m.len() == 1 && !matches!(m[0], b'a' | b'y' | b'o') {
        epeler(m, false, o);
    } else {
        mot(m, o);
    }
    o.p(b' ');
}

/// Vrai si le mot qui suit (apres des espaces, depuis j) commence par une voyelle : liaison possible.
fn commence_voyelle(t: &[u8], mut j: usize) -> bool {
    while j < t.len() && t[j] == b' ' {
        j += 1;
    }
    if j >= t.len() {
        return false;
    }
    let c = t[j];
    if matches!(c.to_ascii_lowercase(), b'a' | b'e' | b'i' | b'o' | b'u' | b'y') {
        return true;
    }
    if c == 0xC3 && j + 1 < t.len() {
        return matches!(t[j + 1] | 0x20, 0xA0 | 0xA2 | 0xA4 | 0xA8 | 0xA9 | 0xAA | 0xAB | 0xAE | 0xAF | 0xB4 | 0xB6 | 0xB9 | 0xBB | 0xBC);
    }
    false
}

/// Texte (UTF-8 ou ASCII, majuscules ou minuscules) -> phonemes. Retourne le nombre de phonemes.
pub fn phonemes(texte: &[u8], out: &mut [u8]) -> usize {
    let mut o = Sortie { b: out, n: 0 };
    let mut m = [0u8; 40]; // jeton courant, en lettres internes
    let mut k = 0usize;
    let mut nombre = false; // le jeton contient un vrai chiffre : il sera epele
    let mut i = 0usize;
    let n = texte.len();
    macro_rules! vider {
        () => {
            if k > 0 {
                if nombre {
                    // Jeton avec chiffres (nombre, adresse hexadecimale) : chaque signe est dit.
                    for &c in &m[..k] {
                        if c >= 0x80 {
                            o.s(nom_chiffre(c - 0x80));
                        } else {
                            epeler(&[c], false, &mut o);
                        }
                        o.p(b' ');
                    }
                } else {
                    jeton(&m[..k], &mut o);
                    // Liaison : s, x, z finals suivis d'une voyelle prononcent un z (les enfants -> lezAfA).
                    if k > 1 && matches!(m[k - 1], b's' | b'x') && !sigle(&m[..k]) && exception(&m[..k]).is_none() && commence_voyelle(texte, i) {
                        o.n -= 1; // retire l'espace ecrit par jeton, puis z + espace
                        o.p(b'z');
                        o.p(b' ');
                    }
                }
            }
        };
    }
    while i < n {
        let c = texte[i];
        let mut l = 0u8; // lettre interne, 0 = pas une lettre
        let mut pas = 1;
        if c.is_ascii_alphabetic() {
            l = c.to_ascii_lowercase();
        } else if c.is_ascii_digit() {
            l = c + 0x80; // chiffre reel, distinct des codes internes 1..5
            nombre = true;
        } else if c == 0xC3 && i + 1 < n {
            // Lettres accentuees latines en UTF-8 (C3 xx), majuscules et minuscules.
            pas = 2;
            l = match texte[i + 1] | 0x20 {
                0xA0 | 0xA2 | 0xA4 => b'a',
                0xA7 => b'3',
                0xA9 => b'1',
                0xA8 | 0xAA | 0xAB => b'2',
                0xAE | 0xAF => b'i',
                0xB4 | 0xB6 => b'4',
                0xB9 | 0xBB | 0xBC => b'u',
                _ => 0,
            };
        } else if c == 0xC5 && i + 1 < n && (texte[i + 1] == 0x93 || texte[i + 1] == 0x92) {
            pas = 2;
            l = b'5';
        } else if c >= 0x80 {
            // Autre caractere UTF-8 : saute toute la sequence.
            pas = if c >= 0xF0 { 4 } else if c >= 0xE0 { 3 } else { 2 };
        }
        if l != 0 {
            if k < m.len() {
                m[k] = l;
                k += 1;
            }
        } else if c == b'\'' && k > 0 && k <= 2 && !nombre {
            // Elision (l', d', j', qu', n', s', c', m', t') : la consonne se lie au mot suivant.
            let p = match m[0] {
                b'j' => b'Z',
                b'q' | b'c' if k == 2 => b'k',
                b'c' => b's',
                x => x,
            };
            if !voy(p) {
                o.p(p);
            }
            k = 0;
        } else {
            vider!();
            k = 0;
            nombre = false;
            match c {
                b'.' | b'!' | b'?' | b'\n' => o.p(b'.'),
                b',' | b';' | b':' | b'(' | b')' | b'|' | b'=' => o.p(b'_'),
                _ => {}
            }
        }
        i += pas;
    }
    vider!();
    o.n
}

// ---------------------------------------------------------------------------------------------
// 2. Phonemes -> son
// ---------------------------------------------------------------------------------------------

/// Reglage d'un phoneme : formants (Hz), duree (ms), force de la voix et du bruit (0..16),
/// frequence centrale du bruit (Hz), duree de la fermeture avant une occlusive (ms).
struct Ph {
    f: [i32; 3],
    ms: i32,
    voix: i32,
    bruit: i32,
    fb: i32,
    ferme: i32,
}

fn reglage(p: u8) -> Ph {
    let v = |f1, f2, f3, ms, voix| Ph { f: [f1, f2, f3], ms, voix, bruit: 0, fb: 0, ferme: 0 };
    let b = |fb, ms, bruit| Ph { f: [0, 0, 0], ms, voix: 0, bruit, fb, ferme: 0 };
    match p {
        b'i' => v(280, 2250, 3000, 95, 16),
        b'e' => v(370, 2100, 2700, 100, 16),
        b'E' => v(530, 1850, 2500, 110, 16),
        b'a' => v(750, 1350, 2500, 115, 16),
        b'O' => v(520, 1000, 2400, 110, 16),
        b'o' => v(380, 800, 2400, 110, 16),
        b'u' => v(300, 750, 2200, 100, 16),
        b'y' => v(280, 1800, 2200, 100, 16),
        b'2' => v(380, 1450, 2300, 110, 16),
        b'@' => v(450, 1400, 2400, 60, 14),
        b'A' => v(650, 1000, 2500, 135, 16),
        b'C' => v(450, 800, 2500, 135, 16),
        b'I' => v(600, 1500, 2500, 135, 16),
        b'j' => v(280, 2200, 3000, 50, 11),
        b'w' => v(300, 700, 2200, 50, 11),
        b'H' => v(280, 1750, 2200, 50, 11),
        b'l' => v(350, 1500, 2600, 60, 11),
        b'm' => v(250, 1100, 2300, 70, 8),
        b'n' => v(250, 1500, 2500, 70, 8),
        b'R' => Ph { f: [550, 1250, 2400], ms: 60, voix: 8, bruit: 1, fb: 2500, ferme: 0 }, // souffle du r : moins fort, hors du 2e formant
        b'f' => b(4500, 90, 9),
        b's' => b(5500, 100, 7),
        b'S' => b(2800, 100, 12),
        b'v' => Ph { f: [300, 1300, 2300], ms: 70, voix: 7, bruit: 3, fb: 4500, ferme: 0 },
        b'z' => Ph { f: [300, 1600, 2500], ms: 70, voix: 7, bruit: 6, fb: 5500, ferme: 0 },
        b'Z' => Ph { f: [300, 1700, 2400], ms: 70, voix: 7, bruit: 6, fb: 2800, ferme: 0 },
        b'p' => Ph { f: [0, 0, 0], ms: 10, voix: 0, bruit: 8, fb: 1000, ferme: 60 },
        b't' => Ph { f: [0, 0, 0], ms: 15, voix: 0, bruit: 12, fb: 4500, ferme: 50 },
        b'k' => Ph { f: [0, 0, 0], ms: 20, voix: 0, bruit: 12, fb: 2200, ferme: 55 },
        b'b' => Ph { f: [200, 1000, 2300], ms: 8, voix: 4, bruit: 5, fb: 1000, ferme: 50 },
        b'd' => Ph { f: [200, 1600, 2500], ms: 8, voix: 4, bruit: 7, fb: 4500, ferme: 45 },
        b'g' => Ph { f: [200, 1900, 2300], ms: 10, voix: 4, bruit: 7, fb: 2200, ferme: 45 },
        b' ' => v(0, 0, 0, 20, 0),
        b'_' => v(0, 0, 0, 150, 0),
        b'.' => v(0, 0, 0, 320, 0),
        _ => v(0, 0, 0, 0, 0),
    }
}

/// cos(2 pi f / FE) en virgule fixe Q14, par developpement limite (ordre 8). f de 0 a 6000 Hz.
fn cos_q14(f: i32) -> i64 {
    let x = (f as i64 * 421657) >> 14; // angle en radians, Q16 : f * 2 pi * 65536 / 16000
    let x2 = (x * x) >> 16;
    let x4 = (x2 * x2) >> 16;
    let x6 = (x4 * x2) >> 16;
    let x8 = (x4 * x4) >> 16;
    let c = 65536 - x2 / 2 + x4 / 24 - x6 / 720 + x8 / 40320;
    c >> 2
}

#[derive(Clone, Copy)]
struct Reso {
    a: i64,
    b: i64,
    c: i64,
    y1: i64,
    y2: i64,
}
impl Reso {
    const fn nouveau() -> Reso {
        Reso { a: 16384, b: 0, c: 0, y1: 0, y2: 0 }
    }
    /// r = rayon des poles en Q14 (largeur de bande), f = frequence en Hz.
    fn regler(&mut self, f: i32, r: i64) {
        self.c = -((r * r) >> 14);
        self.b = (2 * r * cos_q14(f)) >> 14;
        self.a = 16384 - self.b - self.c;
    }
    fn pas(&mut self, x: i64) -> i64 {
        let y = (self.a * x + self.b * self.y1 + self.c * self.y2) >> 14;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

// Rayons des poles (Q14) : exp(-pi * largeur / FE) pour 120, 140 et 170 Hz de largeur.
// Des resonances un peu plus larges que 80/100/150 Hz : moins metalliques, moins robotiques.
const R: [i64; 3] = [16002, 15960, 15860];
const R_BRUIT: i64 = 13730;

struct Synthe {
    r: [Reso; 3],
    rb: Reso,
    f: [i32; 3],    // formants courants (glissent vers la cible)
    voix: i32,      // force courante de la voix, Q8
    bruit: i32,     // force courante du bruit, Q8
    phase: i32,     // position dans la periode de la voix, en echantillons Q8
    glotte: i64,    // etat du filtre passe-bas de la source
    graine: u32,    // generateur de bruit
    f0: i32,        // hauteur de la voix, Hz Q8
    dc_x: i64,
    dc_y: i64,
}

impl Synthe {
    fn nouveau() -> Synthe {
        Synthe {
            r: [Reso::nouveau(); 3],
            rb: Reso::nouveau(),
            f: [450, 1400, 2400],
            voix: 0,
            bruit: 0,
            phase: 0,
            glotte: 0,
            graine: 0x1234_5678,
            f0: 128 << 8,
            dc_x: 0,
            dc_y: 0,
        }
    }

    /// Produit `ms` millisecondes vers la cible donnee. Retourne le nombre d'echantillons ecrits.
    fn tenir(&mut self, cible: &Ph, voix: i32, bruit: i32, ms: i32, out: &mut [i16], mut n: usize) -> usize {
        let trames = (ms / 5).max(if ms > 0 { 1 } else { 0 });
        for _ in 0..trames {
            // Une trame = 5 ms = 80 echantillons : les reglages glissent vers la cible.
            if cible.f[0] != 0 {
                for k in 0..3 {
                    self.f[k] += (cible.f[k] - self.f[k]) / 3;
                }
            }
            for k in 0..3 {
                self.r[k].regler(self.f[k], R[k]);
            }
            if cible.fb != 0 {
                self.rb.regler(cible.fb, R_BRUIT);
            }
            self.voix += ((voix << 8) - self.voix) / 2;
            self.bruit += ((bruit << 8) - self.bruit) / 2;
            // La hauteur descend lentement dans la phrase (de 128 Hz vers 96 Hz).
            if self.f0 > 96 << 8 {
                self.f0 -= 10;
            }
            // Hauteur naturelle : variation aleatoire de +-0,8 % a chaque trame (jitter), au lieu
            // d'une periode parfaitement reguliere, qui sonne mecanique.
            let periode = ((FE as i32) << 16) / self.f0; // en echantillons, Q8
            let periode = (periode as i64 * (1000 + ((self.graine >> 24) as i64 - 128) / 16) / 1000) as i32;
            for _ in 0..80 {
                if n >= out.len() {
                    return n;
                }
                // Source de voix : une impulsion par periode, adoucie par un passe-bas.
                self.phase += 256;
                let mut imp = 0i64;
                if self.phase >= periode {
                    self.phase -= periode;
                    imp = 60000;
                }
                self.glotte = (self.glotte * 15400 >> 14) + imp;
                // Bruit blanc : generateur congruentiel, valeurs de -2048 a 2047.
                self.graine = self.graine.wrapping_mul(1664525).wrapping_add(1013904223);
                let bl = ((self.graine >> 20) as i64) - 2048;
                // Voix (et souffle du R) dans les trois formants en cascade.
                let mut s = (self.glotte * self.voix as i64) >> 12;
                // Souffle de la glotte : un peu de bruit melange a chaque voyelle (aspiration),
                // au lieu d'une source purement periodique, qui sonne « buzz » et robotique.
                s += (bl * self.voix as i64) >> 16;
                if cible.voix != 0 && cible.bruit != 0 && cible.fb < 2000 {
                    s += (bl * self.bruit as i64) >> 11; // souffle des voyelles bruitees : niveau reduit (correction du grincement)
                }
                for k in 0..3 {
                    s = self.r[k].pas(s);
                }
                // Bruit des consonnes dans son propre resonateur, en parallele.
                let b = self.rb.pas((bl * self.bruit as i64) >> 10); // consonnes : niveau reduit (x0,25)
                let mut y = s + b;
                // Suppression de la composante continue.
                let d = y - self.dc_x + (self.dc_y * 16302 >> 14);
                self.dc_x = y;
                self.dc_y = d;
                y = d >> 3;
                out[n] = y.clamp(-32000, 32000) as i16;
                n += 1;
            }
        }
        n
    }
}

/// Phonemes -> echantillons 16 bits mono a 16 kHz. Retourne le nombre d'echantillons ecrits.
pub fn synthese(ph: &[u8], out: &mut [i16]) -> usize {
    let mut s = Synthe::nouveau();
    // Debut de mot : les formants partent de la cible du premier phoneme (pas de glissement
    // depuis des valeurs par defaut, qui produisait un beuglement au debut).
    if let Some(&p0) = ph.iter().find(|&&p| reglage(p).f[0] != 0) {
        s.f = reglage(p0).f;
    }
    let silence = reglage(b' ');
    let mut n = 0usize;
    for &p in ph {
        let mut g = reglage(p);
        // Parole un peu plus lente (x1,2) : voyelles et consonnes mieux separees, plus claires.
        g.ms = g.ms * 6 / 5;
        if g.ms == 0 {
            continue;
        }
        if p == b'.' {
            s.f0 = 128 << 8; // nouvelle phrase : la hauteur repart du haut
        }
        if g.ferme > 0 {
            // Occlusive : fermeture (silence, ou murmure pour b d g), puis explosion breve.
            let murmure = Ph { f: g.f, ms: g.ferme, voix: g.voix, bruit: 0, fb: 0, ferme: 0 };
            if g.voix != 0 {
                n = s.tenir(&murmure, g.voix, 0, g.ferme, out, n);
            } else {
                n = s.tenir(&silence, 0, 0, g.ferme, out, n);
            }
            n = s.tenir(&g, 0, g.bruit, g.ms, out, n);
        } else {
            n = s.tenir(&g, g.voix, g.bruit, g.ms, out, n);
        }
    }
    // Extinction douce : 30 ms vers le silence.
    s.tenir(&silence, 0, 0, 30, out, n)
}

/// Texte -> son. Retourne le nombre d'echantillons (16 kHz mono) ecrits dans `out`.
pub fn dire(texte: &[u8], out: &mut [i16]) -> usize {
    let mut ph = [0u8; 600];
    let np = phonemes(texte, &mut ph);
    let n = synthese(&ph[..np], out);
    lisser(&mut out[..n]);
    normaliser(&mut out[..n]);
    n
}

/// Filtre de sortie : passe-bas [1 2 1]/4 (retire les aigus proches de 8 kHz, cause du grincement),
/// puis fondu de 5 ms au debut et a la fin (pas de claquement).
fn lisser(x: &mut [i16]) {
    let n = x.len();
    if n < 3 {
        return;
    }
    let mut p1 = x[0] as i32;
    let mut p2 = x[0] as i32;
    for v in x.iter_mut() {
        let c = *v as i32;
        *v = ((p2 + 2 * p1 + c) / 4) as i16;
        p2 = p1;
        p1 = c;
    }
    // Debut : fondu de 5 ms. Fin : fondu de 15 ms, pour que les resonances qui traînent
    // s'eteignent sans beuglement ni claquement.
    let fd = 80usize.min(n / 2);
    for i in 0..fd {
        x[i] = ((x[i] as i32 * i as i32) / fd as i32) as i16;
    }
    let ff = 240usize.min(n / 2);
    for i in 0..ff {
        let j = n - 1 - i;
        x[j] = ((x[j] as i32 * i as i32) / ff as i32) as i16;
    }
}

/// Met le pic a 24 000 (environ 73 % de la pleine echelle) : voix plus forte et nette, sans ecretage.
/// Le gain est limite a 8 : un signal presque nul (silence) n'est pas amplifie.
fn normaliser(x: &mut [i16]) {
    let pic = x.iter().map(|&v| (v as i32).abs()).max().unwrap_or(0);
    if pic == 0 {
        return;
    }
    let cible = 24000i32.min(pic * 8);
    for v in x.iter_mut() {
        *v = ((*v as i32) * cible / pic) as i16;
    }
}
