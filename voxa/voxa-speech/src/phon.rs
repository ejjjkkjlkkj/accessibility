//! Texte français -> notation phonétique de la voix de Voxa. Port en Rust `no_std` de `bib/phon.vx` (mêmes règles, mêmes exceptions).
//! La conformité au code Voxa est vérifiée par comparaison avec des sorties de référence figées (tests/ref-phonvx-*.txt, produites par phon.vx exécuté
//! dans l'émulateur). Mots : règles du français ; nombres : en toutes lettres ; sigles et codes : épelés. Pas de liaisons ; noms étrangers « à la française ».

const OUT_MAX: usize = 220;
const WORD_MAX: usize = 24;

/// Tampon de sortie phonétique de capacité fixe (256 octets).
pub struct PhonOut { buf: [u8; 256], len: usize }
impl PhonOut {
    pub const fn new() -> PhonOut { PhonOut { buf: [0; 256], len: 0 } }
    pub fn clear(&mut self) { self.len = 0; }
    pub fn len(&self) -> usize { self.len }
    pub fn as_bytes(&self) -> &[u8] { &self.buf[..self.len] }
    pub fn as_str(&self) -> &str { core::str::from_utf8(&self.buf[..self.len]).unwrap_or("") }
    fn ep(&mut self, s: &str) { for b in s.bytes() { if self.len < OUT_MAX { self.buf[self.len] = b; self.len += 1; } } }
    fn sp(&mut self) { if self.len > 0 && self.len < OUT_MAX { self.buf[self.len] = b' '; self.len += 1; } }
    fn emit(&mut self, s: &str) { self.ep(s); self.sp(); }
}

struct Word { w: [u8; WORD_MAX], n: usize, digits: usize, letters: usize, caps: usize, forced: bool }

impl Word {
    fn lt(&self, i: isize) -> u8 { if i < 0 || i as usize >= self.n { 0 } else { self.w[i as usize] } }
}

fn est_v(c: u8) -> bool { matches!(c, 97 | 101 | 105 | 111 | 117 | 121) || (1..=9).contains(&c) || matches!(c, 11 | 12 | 13) }
fn doux(x: u8) -> bool { matches!(x, 101 | 105 | 121 | 1 | 2 | 3 | 6 | 11 | 12) }
fn muta(c: u8) -> bool { matches!(c, 112 | 98 | 116 | 100 | 99 | 103 | 102 | 118) }
fn liquide(c: u8) -> bool { c == 114 || c == 108 }

impl Word {
    fn apres_nasale(&self, j: isize) -> bool { let b = self.lt(j); if b == 0 { return true; } !(est_v(b) || b == 110 || b == 109) }
    fn nas(&self, i: isize) -> bool { let a = self.lt(i + 1); (a == 110 || a == 109) && self.apres_nasale(i + 2) }

    // ---- règles contextuelles : renvoient l'indice suivant (> 0) si elles ont émis des phonèmes, 0 sinon
    fn regle_i(&self, o: &mut PhonOut, i: isize) -> isize {
        let n = self.n;
        let (p, a, b) = (self.lt(i - 1), self.lt(i + 1), self.lt(i + 2));
        if p == 0 || est_v(p) { return 0; }
        if a == 101 {
            if b == 110 { if self.apres_nasale(i + 3) { o.ep("jI"); return i + 3; } o.ep("j"); return i + 1; }
            if b == 114 && self.lt(i + 3) == 0 { o.ep("je"); return i + 3; }
            if b == 100 && self.lt(i + 3) == 0 { o.ep("je"); return i + 3; }
            if b == 117 { o.ep("j2"); return i + 3; }
            if b == 0 && n > 2 { o.ep("i"); return i + 2; }
            o.ep("j"); return i + 1;
        }
        if a == 108 { if b == 108 { o.ep("ij"); return i + 3; } return 0; }
        if est_v(a) { o.ep("j"); return i + 1; }
        0
    }
    fn regle_a(&self, o: &mut PhonOut, i: isize) -> isize {
        let (a, b) = (self.lt(i + 1), self.lt(i + 2));
        if a == 105 && b == 108 {
            if self.lt(i + 3) == 108 { o.ep("aj"); return i + 4; }
            if self.lt(i + 3) == 0 { o.ep("aj"); return i + 3; }
        }
        if a == 6 { o.ep("E"); return i + 2; }
        0
    }
    fn regle_e(&self, o: &mut PhonOut, i: isize) -> isize {
        let (a, b) = (self.lt(i + 1), self.lt(i + 2));
        if a == 105 {
            if b == 108 {
                if self.lt(i + 3) == 108 { o.ep("Ej"); return i + 4; }
                if self.lt(i + 3) == 0 { o.ep("Ej"); return i + 3; }
            }
            if b == 110 && self.apres_nasale(i + 3) { o.ep("I"); return i + 3; }
            return 0;
        }
        if a == 117 { if b == 120 && self.lt(i + 3) == 0 { o.ep("2"); return i + 3; } return 0; }
        if a == 109 { if b == 112 && self.lt(i + 3) == 115 && self.lt(i + 4) == 0 { o.ep("Q"); return i + 4; } return 0; }
        if a == 120 { o.ep("E"); return i + 1; }
        if a == 114 && b == 105 && self.lt(i + 3) == 101 && self.lt(i + 4) == 0 { return i + 1; }
        if a != 0 && !est_v(a) && b != 0 && !est_v(b) && muta(a) && liquide(b) { o.ep("@"); return i + 1; }
        0
    }
    fn regle_o(&self, o: &mut PhonOut, i: isize) -> isize {
        let (a, b) = (self.lt(i + 1), self.lt(i + 2));
        if a == 117 { if b == 105 { o.ep("wi"); return i + 3; } return 0; }
        if a == 121 && est_v(b) { o.ep("waj"); return i + 2; }
        if a == 115 {
            if b == 0 { o.ep("o"); return i + 1; }
            if b == 101 && self.lt(i + 3) == 0 { o.ep("o"); return i + 1; }
        }
        if b == 0 && (a == 116 || a == 100 || a == 112) { o.ep("o"); return i + 1; }
        0
    }
    fn regle_u(&self, o: &mut PhonOut, i: isize) -> isize {
        let p = self.lt(i - 1);
        if p == 0 || est_v(p) { return 0; }
        if self.lt(i + 1) == 105 {
            if self.lt(i + 2) == 110 || self.lt(i + 2) == 109 { return 0; }
            o.ep("H"); return i + 1;
        }
        0
    }

    /// Prononciation d'un mot normalisé.
    fn g2p(&self, o: &mut PhonOut) {
        let n = self.n as isize;
        let mut i: isize = 0;
        while i < n {
            let c = self.lt(i);
            let (a, b, p) = (self.lt(i + 1), self.lt(i + 2), self.lt(i - 1));
            let mut s = i + 1;
            match c {
                97 => {
                    let r = self.regle_a(o, i);
                    if r > 0 { s = r; }
                    else if a == 117 { o.ep("o"); s = i + 2; }
                    else if a == 105 {
                        if (b == 110 || b == 109) && self.apres_nasale(i + 3) { o.ep("I"); s = i + 3; } else { o.ep("E"); s = i + 2; }
                    }
                    else if self.nas(i) { o.ep("Q"); s = i + 2; }
                    else { o.ep("a"); }
                }
                1 => o.ep("e"), 2 | 3 | 11 => o.ep("E"), 4 | 5 => o.ep("a"), 6 | 12 => o.ep("i"), 7 => o.ep("o"), 8 | 9 => o.ep("y"), 10 => o.ep("s"),
                13 => { o.ep("9"); if a == 117 { s = i + 2; } }
                101 => {
                    let r = self.regle_e(o, i);
                    if r > 0 { s = r; }
                    else if a == 97 { if b == 117 { o.ep("o"); s = i + 3; } else { o.ep("@"); } }
                    else if a == 117 {
                        if b == 0 { o.ep("2"); }
                        else if b == 115 { if self.lt(i + 3) == 0 { o.ep("2"); } else { o.ep("9"); } }
                        else if b == 122 { o.ep("2"); }
                        else { o.ep("9"); }
                        s = i + 2;
                    }
                    else if a == 105 { o.ep("E"); s = i + 2; }
                    else if self.nas(i) { o.ep("Q"); s = i + 2; }
                    else if a == 114 || a == 122 { if b == 0 { o.ep("e"); s = i + 2; } else if est_v(b) { o.ep("@"); } else { o.ep("E"); } }
                    else if a == 115 {
                        if b == 0 { if n > 3 { s = i + 2; } else { o.ep("E"); s = i + 2; } }
                        else if est_v(b) { o.ep("@"); } else { o.ep("E"); }
                    }
                    else if a == 116 {
                        if b == 0 { if n == 2 { o.ep("e"); } else { o.ep("E"); } s = i + 2; }
                        else if est_v(b) { o.ep("@"); } else { o.ep("E"); }
                    }
                    else if a == 0 { if n == 2 || n == 1 { o.ep("@"); } }
                    else if !est_v(a) { if est_v(b) { o.ep("@"); } else { o.ep("E"); } }
                    else { o.ep("@"); }
                }
                105 => {
                    let r = self.regle_i(o, i);
                    if r > 0 { s = r; } else if self.nas(i) { o.ep("I"); s = i + 2; } else { o.ep("i"); }
                }
                111 => {
                    let r = self.regle_o(o, i);
                    if r > 0 { s = r; }
                    else if a == 101 { if b == 117 { o.ep("9"); s = i + 3; } else { o.ep("O"); } }
                    else if a == 117 { o.ep("u"); s = i + 2; }
                    else if a == 105 {
                        if b == 110 && self.apres_nasale(i + 3) { o.ep("wI"); s = i + 3; } else { o.ep("wa"); s = i + 2; }
                    }
                    else if self.nas(i) { o.ep("U"); s = i + 2; }
                    else if a == 0 { o.ep("o"); }
                    else { o.ep("O"); }
                }
                117 => {
                    let r = self.regle_u(o, i);
                    if r > 0 { s = r; } else if self.nas(i) { o.ep("I"); s = i + 2; } else { o.ep("y"); }
                }
                121 => { if est_v(p) && est_v(a) { o.ep("j"); } else { o.ep("i"); } }
                _ if a == c && !est_v(c) => { if c == 99 && doux(b) { o.ep("k"); } s = i + 1; }
                98 => o.ep("b"),
                99 => {
                    if a == 104 { o.ep("S"); s = i + 2; }
                    else if doux(a) { o.ep("s"); }
                    else if a == 0 { if p == 110 { s = i + 1; } else { o.ep("k"); } }
                    else { o.ep("k"); }
                }
                100 => { if a == 0 { if n < 3 { o.ep("d"); } } else { o.ep("d"); } }
                102 => o.ep("f"),
                103 => {
                    if a == 110 { o.ep("nj"); s = i + 2; }
                    else if a == 117 { if doux(b) { o.ep("g"); s = i + 2; } else { o.ep("g"); } }
                    else if doux(a) { o.ep("Z"); }
                    else if a == 0 { if n > 2 { if p == 110 { s = i + 1; } else { o.ep("g"); } } else { o.ep("g"); } }
                    else if a == 116 { if b == 0 { if p == 110 { s = i + 1; } else { o.ep("g"); } } else { o.ep("g"); } }
                    else { o.ep("g"); }
                }
                104 => { s = i + 1; }
                106 => o.ep("Z"), 107 => o.ep("k"), 108 => o.ep("l"),
                109 => { if a == 110 && est_v(p) { s = i + 1; } else { o.ep("m"); } }
                110 => o.ep("n"),
                112 => {
                    if a == 104 { o.ep("f"); s = i + 2; }
                    else if a == 0 { if n > 3 { s = i + 1; } else { o.ep("p"); } }
                    else if a == 115 && b == 0 && p == 109 { s = i + 1; }
                    else { o.ep("p"); }
                }
                113 => { if a == 117 { o.ep("k"); s = i + 2; } else { o.ep("k"); } }
                114 => o.ep("R"),
                115 => {
                    if a == 99 { if doux(b) { o.ep("s"); s = i + 2; } else { o.ep("s"); } }
                    else if a == 0 { if n < 3 { o.ep("s"); } }
                    else if est_v(p) && est_v(a) { o.ep("z"); }
                    else { o.ep("s"); }
                }
                116 => {
                    if a == 105 { if b == 111 && p != 115 { o.ep("sj"); s = i + 2; } else { o.ep("t"); } }
                    else if a == 104 { o.ep("t"); s = i + 2; }
                    else if a == 0 { if n < 3 { o.ep("t"); } }
                    else { o.ep("t"); }
                }
                118 => o.ep("v"), 119 => o.ep("w"),
                120 => { if a == 0 { if n < 3 { o.ep("ks"); } } else { o.ep("ks"); } }
                122 => { if a == 0 { if p != 101 { o.ep("z"); } } else { o.ep("z"); } }
                _ => {}
            }
            i = s;
        }
        o.sp();
    }
}

fn nom_lettre(c: u8) -> &'static str {
    match c { 97 => "a", 98 => "be", 99 => "se", 100 => "de", 101 => "2", 102 => "Ef", 103 => "Ze", 104 => "aS", 105 => "i", 106 => "Zi", 107 => "ka", 108 => "El",
        109 => "Em", 110 => "En", 111 => "o", 112 => "pe", 113 => "ky", 114 => "ER", 115 => "Es", 116 => "te", 117 => "y", 118 => "ve", 119 => "dubl@ve",
        120 => "iks", 121 => "igREk", 122 => "zEd", _ => "" }
}
fn nom_chiffre(d: u64) -> &'static str { match d { 0 => "zeRo", 1 => "I", 2 => "d2", 3 => "tRwa", 4 => "katR", 5 => "sIk", 6 => "sis", 7 => "sEt", 8 => "Hit", _ => "n9f" } }
fn nom_dix(k: u64) -> &'static str { match k { 0 => "dis", 1 => "UZ", 2 => "duz", 3 => "tREz", 4 => "katORz", 5 => "kIz", 6 => "sEz", 7 => "disEt", 8 => "dizHit", _ => "dizn9f" } }
fn nom_dizaine(t: u64) -> &'static str { match t { 2 => "vIt", 3 => "tRQt", 4 => "kaRQt", 5 => "sIkQt", 6 => "swasQt", _ => "katRvI" } }

fn dire_99(o: &mut PhonOut, n: u64) {
    if n == 0 { return; }
    if n < 10 { o.emit(nom_chiffre(n)); return; }
    if n < 20 { o.emit(nom_dix(n - 10)); return; }
    let (t, u) = (n / 10, n % 10);
    if t == 7 { o.emit(nom_dizaine(6)); if u == 1 { o.emit("e"); } o.emit(nom_dix(u)); return; }
    if t == 9 { o.emit(nom_dizaine(8)); o.emit(nom_dix(u)); return; }
    o.emit(nom_dizaine(t));
    if u == 1 && t < 8 { o.emit("e"); }
    if u > 0 { o.emit(nom_chiffre(u)); }
}
fn dire_999(o: &mut PhonOut, n: u64) {
    let h = n / 100;
    if h == 1 { o.emit("sQ"); }
    if h > 1 { o.emit(nom_chiffre(h)); o.emit("sQ"); }
    dire_99(o, n % 100);
}
fn dire_nombre(o: &mut PhonOut, n: u64) {
    if n == 0 { o.emit("zeRo"); return; }
    let m = n / 1_000_000;
    if m > 0 { dire_999(o, m); o.emit("miljU"); }
    let k = (n / 1000) % 1000;
    if k == 1 { o.emit("mil"); }
    if k > 1 { dire_999(o, k); o.emit("mil"); }
    dire_999(o, n % 1000);
}

impl Word {
    fn epele_p(&self) -> bool {
        let n = self.n;
        if self.forced || self.digits > 0 { return true; }
        if self.caps == n && n < 6 { return true; }
        if self.caps == n && n > 1 && !est_v(self.lt(1)) && !est_v(self.lt(0)) { return true; }
        n == 1 && self.caps == 1
    }
    fn egal(&self, s: &str) -> bool { self.n == s.len() && s.bytes().enumerate().all(|(i, b)| self.w[i] == b) }
    fn exception(&self, o: &mut PhonOut) -> bool {
        const T: [(&str, &str); 27] = [("mhz", "mEgaERts"), ("ghz", "gigaERts"), ("hz", "ERts"), ("mo", "mEgaoktE"), ("go", "gigaoktE"), ("ko", "kilooktE"), ("to", "teRaoktE"),
            ("mt", "mEgatRQsfER"), ("bus", "bys"), ("mars", "maRs"), ("femme", "fam"), ("dix", "dis"), ("six", "sis"), ("seconde", "s@gUd"), ("internet", "ItERnEt"),
            ("hier", "jER"), ("hiver", "ivER"), ("photo", "foto"), ("jeudi", "Z2di"), ("ville", "vil"), ("mille", "mil"), ("village", "vilaZ"), ("tranquille", "tRQkil"),
            ("sept", "sEt"), ("huit", "Hit"), ("ryzen", "RizEn"), ("wifi", "wifi")];
        for (k, v) in T.iter() { if self.egal(k) { o.emit(v); return true; } }
        false
    }
    fn dire_chiffres(&self, o: &mut PhonOut, i: usize) -> usize {
        let mut j = i;
        while j < self.n && (48..=57).contains(&self.w[j]) { j += 1; }
        let lu = j - i <= 4 && !(self.w[i] == 48 && j - i > 1);
        if lu { let mut v = 0u64; for k in i..j { v = v * 10 + (self.w[k] - 48) as u64; } dire_nombre(o, v); }
        else { for k in i..j { o.emit(nom_chiffre((self.w[k] - 48) as u64)); } }
        j
    }
    fn g2p_un(o: &mut PhonOut, c: u8) { match c { 1 => o.emit("e"), 10 => o.emit("se"), _ => o.emit("2") } }
    fn dire_jeton(&self, o: &mut PhonOut) {
        let n = self.n;
        if n == 0 { return; }
        if self.letters == 0 {
            if n < 7 { let mut v = 0u64; for k in 0..n { v = v * 10 + (self.w[k] - 48) as u64; } dire_nombre(o, v); return; }
            for k in 0..n { o.emit(nom_chiffre((self.w[k] - 48) as u64)); }
            return;
        }
        if self.exception(o) { return; }
        if self.epele_p() {
            let mut i = 0;
            while i < n {
                let c = self.w[i];
                if c >= 48 {
                    if c <= 57 { i = self.dire_chiffres(o, i); }
                    else { if c < 97 { Word::g2p_un(o, c); } else { o.emit(nom_lettre(c)); } i += 1; }
                } else { Word::g2p_un(o, c); i += 1; }
            }
            return;
        }
        self.g2p(o);
    }
}

/// Normalise un point de code : 0 séparateur, -1 apostrophe, 1..=13 lettres accentuées, 97..=122 lettres, 48..=57 chiffres.
fn norm(cp: u32) -> i32 {
    match cp {
        97..=122 => cp as i32, 65..=90 => cp as i32 + 32, 48..=57 => cp as i32, 39 | 8217 => -1,
        233 | 201 => 1, 232 | 200 => 2, 234 | 202 => 3, 224 => 4, 226 => 5, 238 => 6, 244 => 7, 251 => 8, 249 => 9, 231 => 10, 235 => 11, 239 => 12, 339 => 13,
        _ => 0,
    }
}
fn est_maj(cp: u32) -> bool { (65..=90).contains(&cp) || (192..=221).contains(&cp) }

/// Convertit src[deb..] en phonétique dans `o` ; s'arrête après . : ; ou retour à la ligne, ou quand la sortie dépasse 24 caractères.
/// Renvoie l'indice où reprendre (== src.len() à la fin).
pub fn phon_texte(src: &[u16], deb: usize, o: &mut PhonOut) -> usize {
    o.clear();
    let fin = src.len();
    let mut i = deb;
    while i < fin {
        let cp = src[i] as u32;
        let k = norm(cp);
        if k == 0 {
            i += 1;
            let mut cp = cp;
            if cp == 92 { o.emit("QtislaS"); }
            if cp == 95 { o.emit("tiRe dy bA"); }
            if cp == 47 {
                let mut seconde = false;
                if i < fin && norm(src[i] as u32) == 115 { if i + 1 >= fin { seconde = true; } else if norm(src[i + 1] as u32) == 0 { seconde = true; } }
                if seconde { o.emit("paR sEgUd"); i += 1; } else { o.emit("baR"); }
            }
            if cp == 64 { o.emit("aRobaz"); }
            if cp == 37 { o.emit("puRsQ"); }
            if cp == 46 && i < fin && i > deb + 1 && norm(src[i] as u32) > 0 && i >= 2 && norm(src[i - 2] as u32) > 0 { o.emit("pwI"); cp = 0; }
            if (cp == 46 || cp == 58 || cp == 59 || cp == 10) && o.len() > 0 { return i; }
        } else {
            if o.len() > 24 { return i; }
            let mut w = Word { w: [0; WORD_MAX], n: 0, digits: 0, letters: 0, caps: 0, forced: false };
            loop {
                if i >= fin { break; }
                let cq = src[i] as u32; let kq = norm(cq);
                if kq == 0 { break; }
                if kq > 0 {
                    if w.n >= WORD_MAX { break; }
                    w.w[w.n] = kq as u8; w.n += 1;
                    if (48..=57).contains(&kq) { w.digits += 1; } else { w.letters += 1; }
                    if est_maj(cq) { w.caps += 1; }
                    i += 1;
                } else { i += 1; }
            }
            if w.n > 8 && (w.letters == 0 || w.epele_p()) { i -= w.n - 8; w.n = 8; w.forced = true; }
            w.dire_jeton(o);
        }
    }
    i
}

const fn nom_symbole(cp: u32) -> &'static str {
    match cp { 44 => "virgyl", 46 => "pwI", 58 => "d2pwI", 59 => "pwIvirgyl", 45 => "tRe", 47 => "baR", 40 => "paRQtEzuvERt", 41 => "paRQtEzfERme", 33 => "pwIdEksklamasjU",
        63 => "pwIdIteRogasjU", 61 => "egal", 43 => "plys", _ => "" }
}
fn nom_accent(k: i32) -> &'static str {
    match k { 1 => "e aksQtegy", 2 => "2 aksQgRav", 3 => "2 aksQsiRkUflEks", 4 => "a aksQgRav", 5 => "a aksQsiRkUflEks", 6 => "i aksQsiRkUflEks",
        7 => "o aksQsiRkUflEks", 8 => "y aksQsiRkUflEks", 9 => "y aksQgRav", 10 => "se sedij", 11 => "2 tRema", 12 => "i tRema", _ => "o e" }
}
/// Épelle src[deb..] lettre par lettre (morceaux de 30 caractères phonétiques) ; renvoie l'indice où reprendre.
pub fn phon_epeler(src: &[u16], deb: usize, o: &mut PhonOut) -> usize {
    o.clear();
    let mut i = deb;
    while i < src.len() {
        if o.len() > 30 { return i; }
        let cp = src[i] as u32; let k = norm(cp);
        if k >= 97 { o.emit(nom_lettre(k as u8)); }
        else if k >= 48 { o.emit(nom_chiffre((k - 48) as u64)); }
        else if k > 0 { o.emit(nom_accent(k)); }
        else if k == -1 { o.emit("apOstRof"); }
        else { let s = nom_symbole(cp); if !s.is_empty() { o.emit(s); } }
        i += 1;
    }
    i
}

/// Convertit un texte UTF-8 en points de code 16 bits (les caractères hors du plan de base deviennent « ? ») ; renvoie le nombre écrit.
pub fn utf8_to_u16(s: &str, dst: &mut [u16]) -> usize {
    let mut n = 0;
    for c in s.chars() { if n >= dst.len() { break; } let cp = c as u32; dst[n] = if cp < 0x10000 { cp as u16 } else { 63 }; n += 1; }
    n
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::string::String;

    /// Phonétique d'un mot ou texte entier (morceaux concaténés par « | » comme dans les sorties de référence).
    fn phon(s: &str) -> String {
        let mut buf = [0u16; 512]; let n = utf8_to_u16(s, &mut buf); let src = &buf[..n];
        let mut o = PhonOut::new(); let mut d = 0; let mut r = String::new();
        while d < src.len() { d = phon_texte(src, d, &mut o); r.push_str(o.as_str()); r.push('|'); }
        r
    }
    fn mot(s: &str) -> String { let mut buf = [0u16; 64]; let n = utf8_to_u16(s, &mut buf); let mut o = PhonOut::new(); phon_texte(&buf[..n], 0, &mut o); String::from(o.as_str().trim_end()) }

    fn verifie_reference(nom: &str, contenu: &str) -> (usize, std::vec::Vec<String>) {
        let mut n = 0; let mut ecarts = std::vec::Vec::new();
        for l in contenu.lines() {
            let l = l.trim_end_matches('\r'); if l.is_empty() { continue; }
            let (m, att) = l.split_once('\t').unwrap_or((l, ""));
            let got = mot(m); n += 1;
            if got != att.trim_end() { ecarts.push(std::format!("{nom}: {m}: voxa={att} rust={got}")); }
        }
        (n, ecarts)
    }

    #[test] fn identique_a_phon_vx_sur_les_lexiques() {
        let mut total = 0; let mut tous = std::vec::Vec::new();
        for (nom, c) in [("v0", include_str!("../../tests/ref-phonvx-v0.txt")), ("holdout", include_str!("../../tests/ref-phonvx-holdout.txt")), ("holdout2", include_str!("../../tests/ref-phonvx-holdout2.txt"))] {
            let (n, e) = verifie_reference(nom, c); total += n; tous.extend(e);
        }
        assert!(total > 300, "références trop petites : {total}");
        assert!(tous.is_empty(), "{} écarts avec phon.vx sur {total} mots : {:?}", tous.len(), &tous[..tous.len().min(10)]);
    }
    #[test] fn phrases_connues() {
        assert_eq!(phon("Informations sur le matériel"), "IfORmasjU syR l@ mateRjEl |");
        assert_eq!(phon("Barrette 1 : 8192 Mo, 3200 MT/s"), "baREt I |Hit mil sQ katRvI duz mEgaoktE |tRwa mil d2 sQ mEgatRQsfER paR sEgUd |");
        assert_eq!(phon("C:\\Windows\\System32"), "se |QtislaS wIdOw QtislaS Es igREk Es te 2 Em tRQt d2 |");
    }
    #[test] fn nombres() {
        assert_eq!(mot("71"), "swasQt e UZ"); assert_eq!(mot("80"), "katRvI"); assert_eq!(mot("1999"), "mil n9f sQ katRvI dizn9f");
        assert_eq!(mot("0"), "zeRo");
    }
    #[test] fn epellation() {
        let mut buf = [0u16; 16]; let n = utf8_to_u16("Ab5é", &mut buf); let mut o = PhonOut::new(); phon_epeler(&buf[..n], 0, &mut o);
        assert_eq!(o.as_str().trim_end(), "a be sIk e aksQtegy");
    }
    #[test] fn jamais_de_depassement_de_tampon() {
        let long = "anticonstitutionnellement ".repeat(30); let mut buf = std::vec![0u16; 1024]; let n = utf8_to_u16(&long, &mut buf);
        let mut o = PhonOut::new(); let mut d = 0; let mut guard = 0;
        while d < n { let nd = phon_texte(&buf[..n], d, &mut o); assert!(o.len() <= 256); assert!(nd > d || nd == n, "pas de progrès"); d = nd; guard += 1; assert!(guard < 5000); }
    }
}
