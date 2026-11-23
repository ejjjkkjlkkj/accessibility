//! voxa-hii : lecture des ecrans de configuration du firmware (HII), sans allocation, en LECTURE SEULE.
//! Entree : le tampon produit par EFI_HII_DATABASE_PROTOCOL.ExportPackageLists (listes de paquets bout a bout) ou n'importe quelle suite de listes valides.
//! Sortie : une suite de "lignes" lisibles (ensemble, page, titre, texte, choix, case, nombre, lien, option...) avec indentation, texte et aide transcrits en ASCII.
//! Aucune ecriture dans le firmware, aucun acces a NVRAM : on LIT ce que le firmware decrit, c'est tout.
#![cfg_attr(not(test), no_std)]

pub const LMAX: usize = 120;

/// Petit texte ASCII a taille fixe (les accents sont ramenes a la lettre de base, le reste devient '?').
#[derive(Clone, Copy)]
pub struct Txt { pub b: [u8; LMAX], pub n: usize }
impl Txt {
    pub const fn vide() -> Self { Txt { b: [0; LMAX], n: 0 } }
    pub fn as_bytes(&self) -> &[u8] { &self.b[..self.n] }
    pub fn est_vide(&self) -> bool { self.n == 0 }
    fn pousse(&mut self, c: u8) { if self.n < LMAX { if c == b' ' && (self.n == 0 || self.b[self.n - 1] == b' ') { return; } self.b[self.n] = c; self.n += 1; } }
    fn pousse_ucs2(&mut self, c: u16) {
        let a = match c { 0..=8 | 11..=12 | 14..=31 => return, 9 | 10 | 13 => b' ', 32..=126 => c as u8,
            0xE0..=0xE5 => b'a', 0xE7 => b'c', 0xE8..=0xEB => b'e', 0xEC..=0xEF => b'i', 0xF2..=0xF6 => b'o', 0xF9..=0xFC => b'u',
            0xC0..=0xC5 => b'A', 0xC7 => b'C', 0xC8..=0xCB => b'E', 0xCC..=0xCF => b'I', 0xD2..=0xD6 => b'O', 0xD9..=0xDC => b'U',
            0x2018 | 0x2019 => b'\'', 0x2013 | 0x2014 => b'-', _ => b'?' };
        self.pousse(a);
    }
}

fn u16le(d: &[u8], o: usize) -> usize { match d.get(o..o + 2) { Some(x) => u16::from_le_bytes([x[0], x[1]]) as usize, None => 0 } }
fn u32le(d: &[u8], o: usize) -> usize { match d.get(o..o + 4) { Some(x) => u32::from_le_bytes([x[0], x[1], x[2], x[3]]) as usize, None => 0 } }

/// Un paquet : type, debut, longueur (dans le tampon complet).
#[derive(Clone, Copy)] struct Paquet { ty: u8, o: usize, l: usize }

/// Liste de paquets valide a partir de `pos` : retourne la longueur totale si l'en-tete et la chaine de paquets (terminee par 0xDF) sont coherents.
fn liste_valide(d: &[u8], pos: usize) -> Option<usize> {
    let pl = u32le(d, pos + 16); if pl < 28 || pl > 0x400000 || pos + pl > d.len() { return None; }
    let mut o = pos + 20;
    while o + 4 <= pos + pl { let h = u32le(d, o); let l = h & 0xFF_FFFF; let ty = (h >> 24) as u8; if l < 4 || o + l > pos + pl { return None; }
        if ty == 0xDF { return if o + l == pos + pl { Some(pl) } else { None }; } o += l; }
    None
}
fn paquet_suivant(d: &[u8], liste: usize, fin: usize, o: usize) -> Option<Paquet> {
    let _ = liste; if o + 4 > fin { return None; } let h = u32le(d, o); let l = h & 0xFF_FFFF; let ty = (h >> 24) as u8; if l < 4 || o + l > fin || ty == 0xDF { return None; } Some(Paquet { ty, o, l })
}

/// Cherche la chaine `id` dans un paquet de chaines (SIBT) et l'ecrit dans `out`. Retourne vrai si trouvee.
fn chaine_dans(p: &[u8], id: usize, out: &mut Txt) -> bool {
    let info = u32le(p, 8); if info < 46 || info >= p.len() { return false; }
    let (mut o, mut cur) = (info, 1usize);
    macro_rules! scsu { ($out:expr) => {{ let mut e = o; while e < p.len() && p[e] != 0 { if cur == id { $out.pousse_ucs2(p[e] as u16); } e += 1; } o = e + 1; }} }
    macro_rules! ucs2 { ($out:expr) => {{ let mut e = o; while e + 1 < p.len() { let c = u16::from_le_bytes([p[e], p[e + 1]]); e += 2; if c == 0 { break; } if cur == id { $out.pousse_ucs2(c); } } o = e; }} }
    while o < p.len() {
        let ty = p[o]; o += 1;
        match ty {
            0x00 => return false,
            0x10 => { scsu!(out); if cur == id { return true; } cur += 1; }
            0x11 => { o += 1; scsu!(out); if cur == id { return true; } cur += 1; }
            0x12 | 0x13 => { if ty == 0x13 { o += 1; } let c = u16le(p, o); o += 2; for _ in 0..c { scsu!(out); if cur == id { return true; } cur += 1; } }
            0x14 => { ucs2!(out); if cur == id { return true; } cur += 1; }
            0x15 => { o += 1; ucs2!(out); if cur == id { return true; } cur += 1; }
            0x16 | 0x17 => { if ty == 0x17 { o += 1; } let c = u16le(p, o); o += 2; for _ in 0..c { ucs2!(out); if cur == id { return true; } cur += 1; } }
            0x20 => { if cur == id { return false; } cur += 1; }
            0x21 => { cur += u16le(p, o); o += 2; } 0x22 => { cur += *p.get(o).unwrap_or(&0) as usize; o += 1; }
            0x30 => { let l = *p.get(o + 1).unwrap_or(&2) as usize; o += l.max(2) - 1; }
            0x31 => { let l = u16le(p, o + 1); o += l.max(3) - 1; }
            0x32 => { let l = u32le(p, o + 1); o += l.max(5) - 1; }
            _ => return false,
        }
        if cur > id + 1 && id != 0 { return false; }
    }
    false
}

/// Genre de ligne affichable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Genre { Ensemble, Page, Titre, Texte, Choix, Case, Nombre, MotDePasse, Action, Date, Heure, TexteLibre, ListeOrdonnee, Lien, Option }
impl Genre {
    /// Mot francais annonce par la voix avant le texte.
    pub fn mot(self) -> &'static str { match self { Genre::Ensemble => "Ensemble", Genre::Page => "Page", Genre::Titre => "Titre", Genre::Texte => "Information", Genre::Choix => "Choix", Genre::Case => "Case a cocher", Genre::Nombre => "Nombre", Genre::MotDePasse => "Mot de passe", Genre::Action => "Action", Genre::Date => "Date", Genre::Heure => "Heure", Genre::TexteLibre => "Zone de texte", Genre::ListeOrdonnee => "Liste ordonnee", Genre::Lien => "Lien", Genre::Option => "Option" } }
}
pub struct Ligne { pub genre: Genre, pub profondeur: u8, pub id_page: u16, pub cible: u16, pub texte: Txt, pub aide: Txt, pub valeur: Txt }
impl Ligne { pub const fn vide() -> Self { Ligne { genre: Genre::Titre, profondeur: 0, id_page: 0, cible: 0, texte: Txt::vide(), aide: Txt::vide(), valeur: Txt::vide() } } }

fn genre_de(op: u8) -> Option<Genre> { Some(match op { 0x0E => Genre::Ensemble, 0x01 => Genre::Page, 0x02 => Genre::Titre, 0x03 => Genre::Texte, 0x05 => Genre::Choix, 0x06 => Genre::Case, 0x07 => Genre::Nombre, 0x08 => Genre::MotDePasse, 0x0C => Genre::Action, 0x1A => Genre::Date, 0x1B => Genre::Heure, 0x1C => Genre::TexteLibre, 0x23 => Genre::ListeOrdonnee, 0x0F => Genre::Lien, 0x09 => Genre::Option, _ => return None }) }

/// Parcourt toutes les lignes. `appel(index, ligne)` retourne `false` pour arreter. Retourne le nombre de lignes vues.
pub fn parcourir(d: &[u8], cible: Option<usize>) -> (usize, Option<Ligne>) { parcourir2(d, cible, None) }
/// Comme `parcourir`, avec en plus la recherche de la ligne Page d identifiant donne.
pub fn parcourir2(d: &[u8], cible: Option<usize>, page: Option<u16>) -> (usize, Option<Ligne>) { marche(d, cible, page, None) }
/// Une seule traversee : `appel(index, ligne)` est appele pour CHAQUE ligne, dans l ordre ; retourner `false` arrete. (`ligne(n)` repart du debut a chaque appel : parcourir toutes les lignes avec elle coute le carre du nombre de lignes.)
pub fn visiter(d: &[u8], appel: &mut dyn FnMut(usize, &Ligne) -> bool) { marche(d, None, None, Some(appel)); }
fn marche(d: &[u8], cible: Option<usize>, page: Option<u16>, mut vis: Option<&mut dyn FnMut(usize, &Ligne) -> bool>) -> (usize, Option<Ligne>) {
    let (mut compte, mut pos) = (0usize, 0usize);
    while pos + 24 <= d.len() {
        let Some(pl) = liste_valide(d, pos) else { pos += 1; continue };
        let fin = pos + pl;
        // paquet de chaines prefere : fr, sinon en, sinon premier
        let mut chaines: Option<Paquet> = None; let mut o = pos + 20; let mut tout: [Option<Paquet>; 6] = [None; 6]; let mut nt = 0usize;
        while let Some(p) = paquet_suivant(d, pos, fin, o) { if p.ty == 4 && nt < 6 { tout[nt] = Some(p); nt += 1; } o += p.l; }
        let langue = |p: &Paquet| -> (u8, u8) { let b = &d[p.o..p.o + p.l]; (b.get(46).copied().unwrap_or(0), b.get(47).copied().unwrap_or(0)) };
        for p in tout.iter().flatten() { if langue(p) == (b'f', b'r') { chaines = Some(*p); } }
        if chaines.is_none() { for p in tout.iter().flatten() { if langue(p) == (b'e', b'n') { chaines = Some(*p); break; } } }
        if chaines.is_none() { chaines = tout[0]; }
        let mut o = pos + 20;
        while let Some(p) = paquet_suivant(d, pos, fin, o) {
            o += p.l; if p.ty != 2 { continue; }
            let f = &d[p.o..p.o + p.l]; let (mut q, mut prof) = (4usize, 0i32);
            while q + 2 <= f.len() {
                let op = f[q]; let l = (f[q + 1] & 0x7F) as usize; let scope = f[q + 1] & 0x80 != 0; if l < 2 || q + l > f.len() { break; } let c = &f[q..q + l];
                if let Some(g) = genre_de(op) {
                    if vis.is_some() || Some(compte) == cible || (g == Genre::Page && page.is_some() && page == Some(u16le(c, 2) as u16)) {
                        let mut ln = Ligne::vide(); ln.genre = g; ln.profondeur = prof.max(0) as u8;
                        let (prompt, aide, sec) = match g { Genre::Ensemble => (u16le(c, 18), u16le(c, 20), 0), Genre::Page => (u16le(c, 4), 0, 0), Genre::Option => (u16le(c, 2), 0, 0), Genre::Texte => (u16le(c, 2), u16le(c, 4), u16le(c, 6)), _ => (u16le(c, 2), u16le(c, 4), 0) };
                        if g == Genre::Page { ln.id_page = u16le(c, 2) as u16; } if g == Genre::Lien { ln.cible = u16le(c, 13) as u16; }
                        if let Some(cp) = chaines { let pk = &d[cp.o..cp.o + cp.l]; if prompt != 0 { chaine_dans(pk, prompt, &mut ln.texte); } if aide != 0 { chaine_dans(pk, aide, &mut ln.aide); } if sec != 0 { chaine_dans(pk, sec, &mut ln.valeur); } }
                        if let Some(v) = vis.as_mut() { if !v(compte, &ln) { return (compte + 1, None); } } else { return (compte, Some(ln)); }
                    }
                    compte += 1;
                }
                if scope { prof += 1; } if op == 0x29 { prof -= 1; }
                q += l;
            }
        }
        pos = fin;
    }
    (compte, None)
}
pub fn nb_lignes(d: &[u8]) -> usize { parcourir(d, None).0 }
/// Indice de la ligne où commence la page d'identifiant donné.
pub fn indice_page(d: &[u8], page: u16) -> Option<usize> { let (i, l) = parcourir2(d, None, Some(page)); l.map(|_| i) }
pub fn ligne(d: &[u8], n: usize) -> Option<Ligne> { parcourir(d, Some(n)).1 }

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<u8> { std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/fixtures/hii-bios-308.bin")).expect("fixture HII") }
    #[test] fn compte_identique_a_voxa_fw() { assert_eq!(nb_lignes(&fixture()), 1698); }
    #[test] fn premiere_page_et_menu_principal() {
        let d = fixture(); let mut menus = std::vec::Vec::new();
        for i in 0..nb_lignes(&d) { let l = ligne(&d, i).unwrap(); if l.genre == Genre::Lien { menus.push(std::string::String::from_utf8_lossy(l.texte.as_bytes()).into_owned()); } }
        for m in ["Main", "Advanced", "Chipset", "Security", "Boot", "Save & Exit"] { assert!(menus.iter().any(|x| x == m), "menu {m} absent"); }
    }
    #[test] fn pages_et_questions() {
        let d = fixture(); let (mut pages, mut qs) = (0, 0);
        for i in 0..nb_lignes(&d) { match ligne(&d, i).unwrap().genre { Genre::Page => pages += 1, Genre::Choix | Genre::Case | Genre::Nombre | Genre::MotDePasse | Genre::Action | Genre::Date | Genre::Heure | Genre::TexteLibre | Genre::ListeOrdonnee | Genre::Lien => qs += 1, _ => {} } }
        assert_eq!(pages, 72); assert_eq!(qs, 565);
    }
    #[test] fn tampon_hostile_ne_plante_pas() {
        assert_eq!(nb_lignes(&[]), 0); assert_eq!(nb_lignes(&[0u8; 100]), 0);
        let mut d = fixture(); for i in (0..d.len()).step_by(37) { d[i] ^= 0xA5; } let _ = nb_lignes(&d); // doit simplement terminer
        let d = fixture(); let _ = nb_lignes(&d[..d.len() / 2]);
    }
    #[test] fn page_retrouvee_par_identifiant() { let d = fixture(); let i = indice_page(&d, 10001).expect("page Main"); let l = ligne(&d, i).unwrap(); assert_eq!(l.genre, Genre::Page); assert_eq!(l.texte.as_bytes(), b"Main"); assert!(indice_page(&d, 65000).is_none()); }
    #[test] fn visiter_donne_les_memes_lignes_que_ligne_et_sait_s_arreter() {
        let d = fixture(); let mut vues = std::vec::Vec::new(); visiter(&d, &mut |i, l| { vues.push((i, l.genre, l.texte.as_bytes().to_vec())); true });
        assert_eq!(vues.len(), 1698); for i in (0..1698).step_by(97) { let l = ligne(&d, i).unwrap(); assert_eq!(vues[i].0, i); assert_eq!(vues[i].1, l.genre); assert_eq!(vues[i].2, l.texte.as_bytes()); }
        let mut n = 0; visiter(&d, &mut |_, _| { n += 1; n < 5 }); assert_eq!(n, 5);
    }
    #[test] fn texte_ascii_sans_accent() { let mut t = Txt::vide(); for c in "Écran  é\u{2019}".encode_utf16() { t.pousse_ucs2(c); } assert_eq!(t.as_bytes(), b"Ecran e'"); }
}
