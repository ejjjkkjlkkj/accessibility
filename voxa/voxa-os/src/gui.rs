//! Bureau graphique de Voxa OS. Principe d'accessibilite : l'ecran est dessine a partir du MEME etat que la voix (l'arbre semantique et les modes de `silicon-core`),
//! jamais d'un etat parallele ; chaque ecran porte une LEGENDE (ce que la voix vient de dire, en grand) pour les personnes malvoyantes ou les accompagnants.
//! Taille du texte (5 niveaux) et contraste (5 themes) viennent des reglages du menu. Rien ne repose sur la seule couleur : le focus est marque par un cadre epais ET le signe « > ».
//! Police : 5x7 maison, agrandie par un facteur entier, avec accents composes (aigu, grave, circonflexe, trema, cedille).
use crate::console::{self, Fb};
use crate::os::Os;
use silicon_core::menu::{MenuApp, Mode, Platform, Power, ITEMS};
use silicon_core::Out;

/// Etat propre a l'affichage (la legende, le journal, l'assistant). Les reglages de taille et de contraste sont recopies du menu a chaque dessin.
pub struct Etat {
    legende: [u8; 240], legende_n: usize,
    pub journal: bool, pub jdec: usize,
    pub assistant: bool, pub a_entree: [u8; 120], pub a_n: usize, pub a_rep: [u8; 240], pub a_rep_n: usize,
    taille: u8, contraste: u8,
}
impl Etat {
    pub const fn nouveau() -> Etat { Etat { legende: [0; 240], legende_n: 0, journal: false, jdec: 0, assistant: false, a_entree: [0; 120], a_n: 0, a_rep: [0; 240], a_rep_n: 0, taille: 1, contraste: 0 } }
    /// Memorise ce que la voix dit (texte exact, avec accents) pour l'afficher en legende.
    pub fn dire(&mut self, t: &str) { let mut n = t.len().min(self.legende.len()); while !t.is_char_boundary(n) { n -= 1; } self.legende[..n].copy_from_slice(&t.as_bytes()[..n]); self.legende_n = n; }
    pub fn legende(&self) -> &str { core::str::from_utf8(&self.legende[..self.legende_n]).unwrap_or("") }
    pub fn reponse(&mut self, t: &str) { let mut n = t.len().min(self.a_rep.len()); while !t.is_char_boundary(n) { n -= 1; } self.a_rep[..n].copy_from_slice(&t.as_bytes()[..n]); self.a_rep_n = n; }
}

struct Theme { fond: u32, texte: u32, barre: u32, barre_txt: u32, focus: u32, focus_txt: u32, alerte: u32, alerte_txt: u32 }
fn theme(c: u8) -> Theme {
    match c % 5 {
        0 => Theme { fond: 0x0010_1830, texte: 0x00F2_F2F2, barre: 0x001E_3A8A, barre_txt: 0x00FF_FFFF, focus: 0x00FF_D400, focus_txt: 0, alerte: 0x00FF_6A00, alerte_txt: 0 },
        1 => Theme { fond: 0, texte: 0x00FF_FFFF, barre: 0x00FF_FFFF, barre_txt: 0, focus: 0x00FF_FF00, focus_txt: 0, alerte: 0x00FF_4040, alerte_txt: 0 },
        2 => Theme { fond: 0x00FF_FFFF, texte: 0, barre: 0, barre_txt: 0x00FF_FFFF, focus: 0, focus_txt: 0x00FF_FFFF, alerte: 0x00C0_0000, alerte_txt: 0x00FF_FFFF },
        3 => Theme { fond: 0, texte: 0x00FF_FF00, barre: 0x00FF_FF00, barre_txt: 0, focus: 0x00FF_FF00, focus_txt: 0, alerte: 0x00FF_8000, alerte_txt: 0 },
        _ => Theme { fond: 0, texte: 0x0000_FF40, barre: 0x0000_FF40, barre_txt: 0, focus: 0x0000_FF40, focus_txt: 0, alerte: 0x00FF_FF00, alerte_txt: 0 },
    }
}

fn rect(f: &Fb, x: u32, y: u32, w: u32, h: u32, rgb: u32) {
    let (x1, y1) = ((x + w).min(f.w), (y + h).min(f.h)); if x >= x1 || y >= y1 { return; }
    let v = if f.bgr { rgb } else { (rgb & 0xFF) << 16 | (rgb & 0xFF00) | (rgb >> 16) & 0xFF };
    for yy in y..y1 { let p = (f.base + (yy as u64 * f.stride as u64 + x as u64) * 4) as *mut u32; unsafe { core::slice::from_raw_parts_mut(p, (x1 - x) as usize).fill(v); } }
}

/// (lettre de base, accent) : 1 aigu, 2 grave, 3 circonflexe, 4 trema, 5 cedille.
fn decompose(c: char) -> (char, u8) {
    match c {
        'é' => ('e', 1), 'è' => ('e', 2), 'ê' => ('e', 3), 'ë' => ('e', 4), 'à' => ('a', 2), 'â' => ('a', 3), 'ä' => ('a', 4), 'î' => ('i', 3), 'ï' => ('i', 4),
        'ô' => ('o', 3), 'ö' => ('o', 4), 'ù' => ('u', 2), 'û' => ('u', 3), 'ü' => ('u', 4), 'ç' => ('c', 5), 'É' => ('E', 1), 'È' => ('E', 2), 'Ê' => ('E', 3),
        'À' => ('A', 2), 'Â' => ('A', 3), 'Î' => ('I', 3), 'Ô' => ('O', 3), 'Ù' => ('U', 2), 'Ç' => ('C', 5),
        '’' | '‘' => ('\'', 0), '«' | '»' | '“' | '”' => ('"', 0), '…' | '·' => ('.', 0), '–' | '—' => ('-', 0), 'œ' => ('o', 0), '\u{a0}' => (' ', 0),
        c => (c, 0),
    }
}
/// 9 rangees : les 7 de la police 5x7, sauf g, j, p, q, y dessines avec un vrai jambage (la police 5x7 les confond sinon avec 9, J, P, Q, Y : defaut de lisibilite pour les malvoyants).
fn rangees(b: char) -> [u8; 9] {
    match b {
        'g' => [0, 0, 0x0F, 0x11, 0x11, 0x11, 0x0F, 0x01, 0x0E],
        'j' => [0, 0x02, 0, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
        'p' => [0, 0, 0x1E, 0x11, 0x11, 0x11, 0x1E, 0x10, 0x10],
        'q' => [0, 0, 0x0F, 0x11, 0x11, 0x11, 0x0F, 0x01, 0x01],
        'y' => [0, 0, 0x11, 0x11, 0x11, 0x0F, 0x01, 0x11, 0x0E],
        _ => { let g = console::glyphe_bits(b as u32); [g[0], g[1], g[2], g[3], g[4], g[5], g[6], 0, 0] }
    }
}
fn glyphe(f: &Fb, c: char, x: u32, y: u32, s: u32, fg: u32) {
    let (b, acc) = decompose(c); let g = rangees(b);
    for (r, bits) in g.iter().enumerate() { for col in 0..5u32 { if bits & (0x10 >> col) != 0 { rect(f, x + col * s, y + r as u32 * s, s, s, fg); } } }
    let pts: &[(u32, i32)] = match acc { 1 => &[(3, -2), (2, -1)], 2 => &[(1, -2), (2, -1)], 3 => &[(2, -2), (1, -1), (3, -1)], 4 => &[(1, -1), (3, -1)], 5 => &[(2, 7), (1, 8)], _ => &[] };
    for &(col, row) in pts { let yy = (y as i32 + row * s as i32).max(0) as u32; rect(f, x + col * s, yy, s, s, fg); }
}
/// Ecrit `t` a partir de (x, y), sans depasser `max_w` pixels (un « ~ » marque la coupure). Renvoie le nombre de caracteres dessines.
fn texte(f: &Fb, x: u32, y: u32, s: u32, fg: u32, t: &str, max_w: u32) -> usize {
    let cols = (max_w / (6 * s)) as usize; let total = t.chars().count(); let mut n = 0;
    for (i, c) in t.chars().enumerate() { if i >= cols { break; } let c = if total > cols && i + 1 == cols { '~' } else { c }; glyphe(f, c, x + i as u32 * 6 * s, y, s, fg); n += 1; }
    n
}
/// Coupe `t` en lignes de `cols` colonnes au plus, sur un espace si possible. Remplit `out` d'intervalles d'octets ; renvoie le nombre de lignes.
fn couper(t: &str, cols: usize, out: &mut [(usize, usize)]) -> usize {
    let cols = cols.max(4); let mut n = 0; let mut deb = 0; let b = t.as_bytes();
    while deb < b.len() && n < out.len() {
        let mut fin = deb; let mut nc = 0; let mut dernier_espace = None;
        while fin < b.len() && nc < cols { if b[fin] == b' ' { dernier_espace = Some(fin); } let l = utf8_len(b[fin]); fin += l; nc += 1; }
        let mut coupe = fin; if fin < b.len() { if let Some(e) = dernier_espace { if e > deb { coupe = e; } } }
        out[n] = (deb, coupe); n += 1; deb = coupe; while deb < b.len() && b[deb] == b' ' { deb += 1; }
    }
    n
}
fn utf8_len(b: u8) -> usize { if b < 0x80 { 1 } else if b >> 5 == 0b110 { 2 } else if b >> 4 == 0b1110 { 3 } else if b >> 3 == 0b11110 { 4 } else { 1 } }

/// Liste a defilement : la ligne qui a le focus est toujours visible, entouree d'un cadre epais et precedee de « > ».
fn liste(f: &Fb, th: &Theme, s: u32, x: u32, y: u32, w: u32, h: u32, n: usize, focus: usize, txt: &mut dyn FnMut(usize, &mut Out)) {
    let rh = 15 * s; let vis = ((h / rh) as usize).max(1);
    let first = if n <= vis { 0 } else { focus.saturating_sub(vis / 2).min(n - vis) };
    for k in 0..vis.min(n.saturating_sub(first)) {
        let i = first + k; let ry = y + k as u32 * rh; let mut o = Out::new(); txt(i, &mut o);
        if i == focus {
            rect(f, x, ry, w, rh - s, th.texte); rect(f, x + s, ry + s, w - 2 * s, rh - 3 * s, th.focus);
            glyphe(f, '>', x + 2 * s, ry + 3 * s, s, th.focus_txt); texte(f, x + 10 * s, ry + 3 * s, s, th.focus_txt, o.as_str(), w - 12 * s);
        } else { texte(f, x + 10 * s, ry + 3 * s, s, th.texte, o.as_str(), w - 12 * s); }
    }
    if first > 0 { glyphe(f, '^', x + w - 8 * s, y, s, th.texte); }
    if first + vis < n { glyphe(f, 'v', x + w - 8 * s, y + (vis as u32 - 1) * rh + 3 * s, s, th.texte); }
}

/// Un cadre avec texte replie sur plusieurs lignes ; `curseur` = indice de caractere ou dessiner la barre d'insertion.
fn boite(f: &Fb, th: &Theme, s: u32, x: u32, y: u32, w: u32, lignes: u32, titre: &str, t: &str, curseur: Option<usize>) {
    let h = (lignes * 9 + 14) * s; rect(f, x, y, w, h, th.texte); rect(f, x + s, y + s, w - 2 * s, h - 2 * s, th.fond);
    texte(f, x + 3 * s, y + 3 * s, s, th.texte, titre, w - 6 * s);
    let cols = ((w - 6 * s) / (6 * s)) as usize; let top = y + 12 * s;
    for (i, c) in t.chars().enumerate() { let (l, k) = (i / cols, i % cols); if l as u32 >= lignes { break; } glyphe(f, c, x + 3 * s + k as u32 * 6 * s, top + l as u32 * 9 * s, s, th.texte); }
    if let Some(ci) = curseur { let (l, k) = (ci / cols, ci % cols); if (l as u32) < lignes { rect(f, x + 3 * s + k as u32 * 6 * s - s / 2, top + l as u32 * 9 * s - s, s.max(2), 9 * s, th.focus); } }
}

fn trois(o: &mut Out, mut n: usize) { let mut t = [0u8; 12]; let mut i = t.len(); if n == 0 { i -= 1; t[i] = b'0'; } while n > 0 { i -= 1; t[i] = b'0' + (n % 10) as u8; n /= 10; } o.push(core::str::from_utf8(&t[i..]).unwrap_or("0")); }

struct Mise { s: u32, hh: u32, haut: u32, bas: u32, cap_top: u32 }
fn mise(f: &Fb, taille: u8) -> Mise {
    let s_fit = (f.h / 224).max(1); let s = ((s_fit * (3 + taille as u32)) / 4).max(1);
    let hh = 12 * s; let cap_h = 40 * s; let cap_top = f.h.saturating_sub(cap_h);
    Mise { s, hh, haut: hh + 2 * s, bas: cap_top.saturating_sub(2 * s), cap_top }
}
fn entete(f: &Fb, th: &Theme, m: &Mise, gauche: &str, droite: &str) {
    rect(f, 0, 0, f.w, m.hh, th.barre); texte(f, 2 * m.s, 3 * m.s, m.s, th.barre_txt, gauche, f.w / 2);
    let n = droite.chars().count() as u32 * 6 * m.s; if n + 4 * m.s < f.w { texte(f, f.w - n - 2 * m.s, 3 * m.s, m.s, th.barre_txt, droite, n + 1); }
}
fn legende(f: &Fb, th: &Theme, m: &Mise, t: &str) {
    rect(f, 0, m.cap_top, f.w, f.h - m.cap_top, th.barre); texte(f, 2 * m.s, m.cap_top + 2 * m.s, m.s, th.barre_txt, "Voix", f.w / 4);
    let cols = ((f.w - 4 * m.s) / (6 * m.s)) as usize; let mut lg = [(0usize, 0usize); 4]; let n = couper(t, cols, &mut lg);
    for (k, &(a, b)) in lg.iter().enumerate().take(n) { texte(f, 2 * m.s, m.cap_top + (12 + k as u32 * 7 + k as u32 * 2) * m.s, m.s, th.barre_txt, &t[a..b], f.w - 4 * m.s); }
}

/// Dessine l'ecran courant du menu (menu, informations, BIOS, saisie, recherche, journal).
pub fn dessiner(app: &MenuApp, os: &mut Os) {
    let Some(f) = console::fb() else { return };
    unsafe { console::ECRAN_ACTIF = false; }
    os.gui.taille = app.text_size; os.gui.contraste = app.contrast;
    let th = theme(app.contrast); let m = mise(&f, app.text_size); let s = m.s;
    rect(&f, 0, 0, f.w, f.h, th.fond);
    let largeur = f.w - 4 * s; let hauteur = m.bas.saturating_sub(m.haut);
    if os.gui.journal { journal(&f, &th, &m, os); legende(&f, &th, &m, os.gui.legende()); return; }
    match app.mode() {
        Mode::Menu => {
            let mut droite = Out::new(); droite.push("élément "); trois(&mut droite, app.focus_index() + 1); droite.push(" sur 11");
            entete(&f, &th, &m, "Voxa OS  -  Menu principal", droite.as_str());
            let (ts, ct) = (app.text_size as usize, app.contrast as usize);
            liste(&f, &th, s, 2 * s, m.haut, largeur, hauteur, ITEMS.len(), app.focus_index(), &mut |i, o| {
                trois(o, i + 1); o.push(". "); o.push(ITEMS[i]);
                if i == 1 { o.push("  ["); trois(o, ts + 1); o.push(" sur 5]"); } else if i == 2 { o.push("  ["); trois(o, ct + 1); o.push(" sur 5]"); }
            });
            if let Some(p) = app.confirming() {
                let y = m.cap_top - 14 * s; rect(&f, 0, y, f.w, 12 * s, th.alerte);
                texte(&f, 2 * s, y + 3 * s, s, th.alerte_txt, if p == Power::Restart { "Confirmer le redémarrage : Entrée une seconde fois. Échap pour annuler." } else { "Confirmer l'extinction : Entrée une seconde fois. Échap pour annuler." }, f.w - 4 * s);
            }
        }
        Mode::Info => {
            let (lignes, cur) = app.info_view().unwrap();
            let mut droite = Out::new(); trois(&mut droite, cur + 1); droite.push(" sur "); trois(&mut droite, lignes.len());
            entete(&f, &th, &m, "Informations sur le matériel", droite.as_str());
            liste(&f, &th, s, 2 * s, m.haut, largeur, hauteur, lignes.len(), cur, &mut |i, o| { o.push(lignes.get(i).unwrap_or("")); });
        }
        Mode::Bios => {
            let (cur, n) = app.bios_view().unwrap();
            let mut droite = Out::new(); trois(&mut droite, cur + 1); droite.push(" sur "); trois(&mut droite, n);
            entete(&f, &th, &m, "BIOS  -  lecture seule", droite.as_str());
            liste(&f, &th, s, 2 * s, m.haut, largeur, hauteur, n, cur, &mut |i, o| { let mut t = Out::new(); if os.bios_line(i, &mut t).is_some() { o.push(t.as_str()); } else { o.push("(ligne illisible)"); } });
        }
        Mode::Edit | Mode::Search => {
            let (t, c, q) = app.edit_view().unwrap();
            entete(&f, &th, &m, "Zone de saisie", if q.is_some() { "recherche" } else { "saisie" });
            let cols = ((largeur - 6 * s) / (6 * s)) as u32; let lignes = ((hauteur / s).saturating_sub(40) / 9).clamp(3, 8).min(((t.chars().count() as u32 + 1) / cols.max(1)) + 3);
            boite(&f, &th, s, 2 * s, m.haut, largeur, lignes, "Texte", t, if q.is_none() { Some(c) } else { None });
            if let Some(q) = q { boite(&f, &th, s, 2 * s, m.haut + (lignes * 9 + 18) * s, largeur, 1, "Texte recherché", q, Some(q.chars().count())); }
        }
    }
    legende(&f, &th, &m, os.gui.legende());
}

fn journal(f: &Fb, th: &Theme, m: &Mise, os: &Os) {
    entete(f, th, m, "Journal du système", "Tab ou Échap : retour");
    let j = console::journal(); let s = m.s; let lh = 9 * s; let vis = ((m.bas.saturating_sub(m.haut)) / lh) as usize;
    // lignes du journal, de la fin vers le debut ; `jdec` lignes de decalage vers le haut
    let mut fin = j.len(); let mut passees = 0usize; let mut y = m.bas.saturating_sub(lh); let mut dessinees = 0;
    while fin > 0 && dessinees < vis {
        let mut deb = fin; while deb > 0 && j[deb - 1] != b'\n' { deb -= 1; }
        if fin > deb || deb == 0 { let ligne = core::str::from_utf8(&j[deb..fin]).unwrap_or(""); if !ligne.is_empty() { if passees >= os.gui.jdec { texte(f, 2 * s, y, s, th.texte, ligne, f.w - 4 * s); if y < lh { break; } y -= lh; dessinees += 1; } else { passees += 1; } } }
        fin = deb.saturating_sub(1); if deb == 0 { break; }
    }
}

/// Dessine l'assistant (saisie + derniere reponse).
pub fn dessiner_assistant(os: &mut Os) {
    let Some(f) = console::fb() else { return };
    unsafe { console::ECRAN_ACTIF = false; }
    let th = theme(os.gui.contraste); let m = mise(&f, os.gui.taille); let s = m.s;
    rect(&f, 0, 0, f.w, f.h, th.fond);
    entete(&f, &th, &m, "Voxa OS  -  Assistant Voxa", "Échap : retour au menu");
    let entree = core::str::from_utf8(&os.gui.a_entree[..os.gui.a_n]).unwrap_or("").to_owned_copy();
    let rep = core::str::from_utf8(&os.gui.a_rep[..os.gui.a_rep_n]).unwrap_or("").to_owned_copy();
    let largeur = f.w - 4 * s; let cols = ((largeur - 6 * s) / (6 * s)) as usize;
    let l_in = (((entree.as_str().chars().count() + 1) / cols.max(1)) as u32 + 1).min(3);
    boite(&f, &th, s, 2 * s, m.haut, largeur, l_in, "Vous (Entrée : envoyer)", entree.as_str(), Some(entree.as_str().chars().count()));
    let y2 = m.haut + (l_in * 9 + 14 + 4) * s;
    let mut lg = [(0usize, 0usize); 8]; let n = couper(rep.as_str(), cols, &mut lg);
    let lignes_rep = (n as u32).clamp(1, 8);
    let h = (lignes_rep * 9 + 14) * s; rect(&f, 2 * s, y2, largeur, h, th.texte); rect(&f, 3 * s, y2 + s, largeur - 2 * s, h - 2 * s, th.fond);
    texte(&f, 5 * s, y2 + 3 * s, s, th.texte, "Voxa", largeur);
    for (k, &(a, b)) in lg.iter().enumerate().take(n) { texte(&f, 5 * s, y2 + (12 + k as u32 * 9) * s, s, th.texte, &rep.as_str()[a..b], largeur - 6 * s); }
    texte(&f, 2 * s, y2 + h + 4 * s, s, th.texte, "Exemples : combien de mémoire, quelle heure est-il, liste les tables ACPI, ouvre le bios, éteins l'ordinateur.", largeur);
    legende(&f, &th, &m, os.gui.legende());
}

/// Petite copie bornee d'un texte (pas d'allocation) pour pouvoir emprunter `os` ailleurs pendant le dessin.
trait CopieBornee { fn to_owned_copy(&self) -> Copie; }
struct Copie { b: [u8; 256], n: usize }
impl Copie { fn as_str(&self) -> &str { core::str::from_utf8(&self.b[..self.n]).unwrap_or("") } }
impl CopieBornee for str { fn to_owned_copy(&self) -> Copie { let mut n = self.len().min(256); while !self.is_char_boundary(n) { n -= 1; } let mut b = [0u8; 256]; b[..n].copy_from_slice(&self.as_bytes()[..n]); Copie { b, n } } }
