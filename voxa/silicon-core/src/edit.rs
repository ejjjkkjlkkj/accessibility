//! Édition de texte accessible : tampon UTF-8 borné, curseur, déplacement par caractère / mot / début / fin, suppression, recherche, et descriptions parlées.
//! Conception inspirée de ce que font les lecteurs d'écran étudiés (TalkBack : granularités caractère/mot/ligne ; Fuchsia : épellation des caractères avec
//! « majuscule » et noms des symboles ; Emacspeak : l'application dit elle-même ce qui change). Le code est écrit par nous.
//! Convention d'annonce : saisie → on dit le caractère tapé ; déplacement → on dit le caractère sur lequel le curseur arrive (ou « fin du texte ») ;
//! suppression → on dit le caractère supprimé. Tout est borné : jamais de panique, jamais de coupe au milieu d'un caractère.

use crate::Out;

pub const EDIT_MAX: usize = 256;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditError { Full }

pub struct Edit { buf: [u8; EDIT_MAX], len: usize, cur: usize }

fn is_word(c: char) -> bool { c.is_alphanumeric() || c == '_' || c == '\'' || c == '’' }

impl Edit {
    pub const fn new() -> Edit { Edit { buf: [0; EDIT_MAX], len: 0, cur: 0 } }
    pub fn as_str(&self) -> &str { core::str::from_utf8(&self.buf[..self.len]).unwrap_or("") }
    /// Position du curseur en octets (toujours sur une limite de caractère).
    pub fn cursor_byte(&self) -> usize { self.cur }
    /// Position du curseur en caractères (nombre de caractères à gauche du curseur).
    pub fn cursor_chars(&self) -> usize { self.as_str()[..self.cur].chars().count() }
    pub fn len_chars(&self) -> usize { self.as_str().chars().count() }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn clear(&mut self) { self.len = 0; self.cur = 0; }

    pub fn insert(&mut self, c: char) -> Result<(), EditError> {
        let mut tmp = [0u8; 4]; let b = c.encode_utf8(&mut tmp).as_bytes();
        if self.len + b.len() > EDIT_MAX { return Err(EditError::Full); }
        self.buf.copy_within(self.cur..self.len, self.cur + b.len());
        self.buf[self.cur..self.cur + b.len()].copy_from_slice(b);
        self.len += b.len(); self.cur += b.len();
        Ok(())
    }
    pub fn char_at_cursor(&self) -> Option<char> { self.as_str()[self.cur..].chars().next() }
    fn char_before(&self) -> Option<char> { self.as_str()[..self.cur].chars().next_back() }

    /// Supprime le caractère à gauche du curseur et le renvoie (None au début).
    pub fn backspace(&mut self) -> Option<char> {
        let c = self.char_before()?; let n = c.len_utf8();
        self.buf.copy_within(self.cur..self.len, self.cur - n); self.len -= n; self.cur -= n; Some(c)
    }
    /// Supprime le caractère sous le curseur et le renvoie (None à la fin).
    pub fn delete(&mut self) -> Option<char> {
        let c = self.char_at_cursor()?; let n = c.len_utf8();
        self.buf.copy_within(self.cur + n..self.len, self.cur); self.len -= n; Some(c)
    }
    /// Déplace d'un caractère vers la gauche. Renvoie false au début du texte (le curseur ne bouge pas).
    pub fn left(&mut self) -> bool { match self.char_before() { Some(c) => { self.cur -= c.len_utf8(); true } None => false } }
    pub fn right(&mut self) -> bool { match self.char_at_cursor() { Some(c) => { self.cur += c.len_utf8(); true } None => false } }
    pub fn home(&mut self) { self.cur = 0; }
    pub fn end(&mut self) { self.cur = self.len; }
    /// Début du mot suivant (ou fin du texte). Renvoie false si déjà à la fin.
    pub fn word_right(&mut self) -> bool {
        if self.cur >= self.len { return false; }
        while let Some(c) = self.char_at_cursor() { if is_word(c) { self.cur += c.len_utf8(); } else { break; } }
        while let Some(c) = self.char_at_cursor() { if !is_word(c) { self.cur += c.len_utf8(); } else { break; } }
        true
    }
    /// Début du mot courant, ou du mot précédent si le curseur est déjà au début d'un mot. Renvoie false au début du texte.
    pub fn word_left(&mut self) -> bool {
        if self.cur == 0 { return false; }
        while let Some(c) = self.char_before() { if !is_word(c) { self.cur -= c.len_utf8(); } else { break; } }
        while let Some(c) = self.char_before() { if is_word(c) { self.cur -= c.len_utf8(); } else { break; } }
        true
    }
    /// Mot sous le curseur (ou qui commence à lui) ; "" si le curseur n'est pas sur un mot.
    pub fn word_at_cursor(&self) -> &str {
        let s = self.as_str(); let mut a = self.cur;
        while let Some(c) = s[..a].chars().next_back() { if is_word(c) { a -= c.len_utf8(); } else { break; } }
        let mut b = self.cur;
        while let Some(c) = s[b..].chars().next() { if is_word(c) { b += c.len_utf8(); } else { break; } }
        &s[a..b]
    }
    /// Recherche `needle` (sans tenir compte de la casse) à partir de l'octet `from` ; renvoie l'octet de début.
    pub fn find(&self, needle: &str, from: usize) -> Option<usize> {
        let s = self.as_str(); if needle.is_empty() || from > s.len() || !s.is_char_boundary(from) { return None; }
        let mut i = from;
        while i <= s.len() {
            if s.is_char_boundary(i) && starts_with_ci(&s[i..], needle) { return Some(i); }
            i += 1;
        }
        None
    }
    /// Recherche vers l'avant depuis le curseur (au-delà de la position courante), avec retour au début ; place le curseur au début de l'occurrence.
    /// Renvoie Some(true) si trouvée sans retour, Some(false) si trouvée après retour au début, None si absente.
    pub fn find_next(&mut self, needle: &str) -> Option<bool> {
        let step = self.char_at_cursor().map_or(0, |c| c.len_utf8());
        if let Some(i) = self.find(needle, (self.cur + step).min(self.len)) { self.cur = i; return Some(true); }
        if let Some(i) = self.find(needle, 0) { if i != self.cur || self.cur + step > self.len { self.cur = i; return Some(false); } }
        None
    }
}

fn starts_with_ci(hay: &str, needle: &str) -> bool {
    let mut h = hay.chars();
    for n in needle.chars() {
        match h.next() { Some(c) => { if c.to_lowercase().ne(n.to_lowercase()) { return false; } } None => return false }
    }
    true
}

/// Description parlée d'un caractère isolé (épellation) : « majuscule a », noms français des symboles, « espace ».
pub fn describe_char(c: char, out: &mut Out) {
    out.clear();
    let name = match c {
        ' ' => "espace", '\t' => "tabulation", ',' => "virgule", '.' => "point", ';' => "point-virgule", ':' => "deux-points", '!' => "point d'exclamation", '?' => "point d'interrogation",
        '\'' | '’' => "apostrophe", '"' => "guillemet", '-' => "tiret", '_' => "tiret bas", '/' => "barre oblique", '\\' => "barre oblique inverse", '(' => "parenthèse ouvrante",
        ')' => "parenthèse fermante", '[' => "crochet ouvrant", ']' => "crochet fermant", '{' => "accolade ouvrante", '}' => "accolade fermante", '@' => "arobase", '#' => "dièse",
        '%' => "pour cent", '&' => "et commercial", '*' => "astérisque", '+' => "plus", '=' => "égale", '<' => "inférieur à", '>' => "supérieur à", '€' => "euro", '$' => "dollar",
        '\n' => "retour à la ligne", _ => "",
    };
    if !name.is_empty() { out.push(name); return; }
    if c.is_uppercase() { out.push("majuscule "); }
    let mut tmp = [0u8; 4];
    for l in c.to_lowercase() { out.push(l.encode_utf8(&mut tmp)); }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    fn typed(s: &str) -> Edit { let mut e = Edit::new(); for c in s.chars() { e.insert(c).unwrap(); } e }
    fn dc(c: char) -> std::string::String { let mut o = Out::new(); describe_char(c, &mut o); std::string::String::from(o.as_str()) }

    #[test] fn insertion_au_curseur_avec_accents() {
        let mut e = typed("tête"); assert_eq!(e.as_str(), "tête"); assert_eq!(e.cursor_chars(), 4);
        e.left(); e.left(); e.insert('X').unwrap(); assert_eq!(e.as_str(), "têXte");
    }
    #[test] fn suppression_renvoie_le_caractere_supprime() {
        let mut e = typed("aé"); assert_eq!(e.backspace(), Some('é')); assert_eq!(e.as_str(), "a");
        e.home(); assert_eq!(e.delete(), Some('a')); assert!(e.is_empty()); assert_eq!(e.backspace(), None); assert_eq!(e.delete(), None);
    }
    #[test] fn deplacement_signale_les_butees() {
        let mut e = typed("ab"); assert!(!e.right(), "déjà à la fin : signalé"); assert!(e.left()); assert!(e.left()); assert!(!e.left(), "début du texte : signalé");
        assert_eq!(e.char_at_cursor(), Some('a'));
    }
    #[test] fn mots_a_gauche_et_a_droite() {
        let mut e = typed("un deux, trois"); e.home();
        assert!(e.word_right()); assert_eq!(e.char_at_cursor(), Some('d'));
        assert!(e.word_right()); assert_eq!(e.char_at_cursor(), Some('t'));
        assert!(e.word_right()); assert_eq!(e.cursor_byte(), e.as_str().len()); assert!(!e.word_right());
        assert!(e.word_left()); assert_eq!(e.word_at_cursor(), "trois");
        assert!(e.word_left()); assert_eq!(e.word_at_cursor(), "deux");
    }
    #[test] fn mot_sous_le_curseur() { let mut e = typed("bonjour le monde"); e.home(); assert_eq!(e.word_at_cursor(), "bonjour"); e.right(); e.right(); assert_eq!(e.word_at_cursor(), "bonjour"); }
    #[test] fn recherche_sans_casse_avec_retour_au_debut() {
        let mut e = typed("Le Chat et le chien"); e.home();
        assert_eq!(e.find_next("chat"), Some(true)); assert_eq!(e.cursor_chars(), 3);
        assert_eq!(e.find_next("le"), Some(true)); assert_eq!(e.cursor_chars(), 11, "l'occurrence suivante, pas celle du curseur");
        assert_eq!(e.find_next("le"), Some(false), "retour au début signalé"); assert_eq!(e.cursor_chars(), 0);
        assert_eq!(e.find_next("zèbre"), None); assert_eq!(e.find_next(""), None);
    }
    #[test] fn recherche_accents() { let mut e = typed("Éléphant"); e.home(); assert_eq!(e.find_next("éph"), Some(true)); }
    #[test] fn tampon_plein_est_une_erreur() { let mut e = Edit::new(); for _ in 0..EDIT_MAX { e.insert('a').unwrap(); } assert_eq!(e.insert('a'), Err(EditError::Full)); assert_eq!(e.insert('é'), Err(EditError::Full)); }
    #[test] fn epellation_des_caracteres() {
        assert_eq!(dc('a'), "a"); assert_eq!(dc('A'), "majuscule a"); assert_eq!(dc('É'), "majuscule é"); assert_eq!(dc(' '), "espace"); assert_eq!(dc('.'), "point"); assert_eq!(dc('@'), "arobase"); assert_eq!(dc('7'), "7");
    }
    #[test] fn jamais_de_panique_sur_suites_aleatoires() {
        let mut e = Edit::new(); let mut x = 0xC0FFEEu32;
        for _ in 0..20_000 {
            x ^= x << 13; x ^= x >> 17; x ^= x << 5;
            match x % 11 { 0 => { let _ = e.insert(['a', 'é', '€', ' ', 'Z', '😀'][(x / 11) as usize % 6]); } 1 => { e.backspace(); } 2 => { e.delete(); } 3 => { e.left(); } 4 => { e.right(); } 5 => e.home(), 6 => e.end(), 7 => { e.word_left(); } 8 => { e.word_right(); } 9 => { let _ = e.find_next("é"); } _ => { let _ = e.word_at_cursor(); } }
            assert!(e.as_str().is_char_boundary(e.cursor_byte()));
        }
    }
}
