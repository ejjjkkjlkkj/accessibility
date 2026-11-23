//! Informations système structurées : lecture des tables SMBIOS (fournies par le firmware) et mise en forme en lignes lisibles une à une.
//! Les données du firmware ne sont pas dignes de confiance : tous les accès sont vérifiés (aucun indice hors tableau ne provoque de panique).
//! Les numéros de série ne sont jamais lus ni restitués. Statut : testé sur l'hôte avec une copie de la table réelle de cette machine (data/smbios.bin),
//! comparée aux valeurs que Windows donne ; jamais exécuté sur le firmware en conditions réelles autrement que par l'émulateur.

use crate::{Out, TextN};

pub const MAX_LINES: usize = 24;
pub const LINE_MAX: usize = 96;
pub type LineText = TextN<LINE_MAX>;

/// Lignes de texte d'une page d'informations (capacité fixe).
pub struct Lines { items: [LineText; MAX_LINES], n: usize }
impl Lines {
    pub fn new() -> Lines { Lines { items: [LineText::empty(); MAX_LINES], n: 0 } }
    /// Ajoute une ligne (tronquée à LINE_MAX octets sur une limite de caractère). Renvoie false si la page est pleine.
    pub fn add(&mut self, s: &str) -> bool { if self.n >= MAX_LINES { return false; } self.items[self.n] = LineText::from(s); self.n += 1; true }
    pub fn len(&self) -> usize { self.n }
    pub fn get(&self, i: usize) -> Option<&str> { if i < self.n { Some(self.items[i].as_str()) } else { None } }
}

fn b(t: &[u8], i: usize) -> u8 { t.get(i).copied().unwrap_or(0) }
fn w(t: &[u8], i: usize) -> u32 { b(t, i) as u32 | (b(t, i + 1) as u32) << 8 }
fn d(t: &[u8], i: usize) -> u32 { w(t, i) | w(t, i + 2) << 16 }

/// Structure SMBIOS : (type, début, longueur de la partie formatée).
struct Iter<'a> { t: &'a [u8], p: usize, guard: usize }
impl<'a> Iterator for Iter<'a> {
    type Item = (u8, usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.p + 4 > self.t.len() || self.guard > 512 { return None; }
        self.guard += 1;
        let (ty, l) = (self.t[self.p], self.t[self.p + 1] as usize);
        if ty == 127 || l < 4 { return None; }
        let start = self.p;
        let mut q = start + l;
        while q + 1 < self.t.len() && !(self.t[q] == 0 && self.t[q + 1] == 0) { q += 1; }
        self.p = q + 2;
        Some((ty, start, l))
    }
}
fn structures(t: &[u8]) -> Iter<'_> { Iter { t, p: 0, guard: 0 } }

/// Chaîne numéro `idx` (à partir de 1) d'une structure, nettoyée des espaces ; "" si absente ou non ASCII.
fn string<'a>(t: &'a [u8], start: usize, l: usize, idx: u8) -> &'a str {
    if idx == 0 { return ""; }
    let mut q = start + l; let mut k = 1;
    while k < idx { while q < t.len() && t[q] != 0 { q += 1; } q += 1; k += 1; if q >= t.len() { return ""; } }
    let mut e = q; while e < t.len() && t[e] != 0 { e += 1; }
    match core::str::from_utf8(&t[q..e]) { Ok(s) if s.is_ascii() => s.trim(), _ => "" }
}

fn push_num(o: &mut Out, mut n: u64) {
    let mut dg = [0u8; 20]; let mut i = 20;
    if n == 0 { i -= 1; dg[i] = b'0'; }
    while n > 0 { i -= 1; dg[i] = b'0' + (n % 10) as u8; n /= 10; }
    o.push(core::str::from_utf8(&dg[i..]).unwrap_or("0"));
}

/// Construit les lignes d'informations à partir des tables SMBIOS et de la marque du processeur (CPUID, texte déjà décodé).
pub fn smbios_lines(table: &[u8], cpu_brand: &str, out: &mut Lines) {
    let mut o = Out::new();
    // processeur
    o.clear(); o.push("Processeur : "); o.push(if cpu_brand.is_empty() { "inconnu" } else { cpu_brand });
    if let Some((_, s, _)) = structures(table).find(|x| x.0 == 4) {
        let (cores, threads, mhz) = (b(table, s + 0x23), b(table, s + 0x25), w(table, s + 0x16));
        if cores > 0 { o.push(", "); push_num(&mut o, cores as u64); o.push(" cœurs"); }
        if threads > 0 { o.push(", "); push_num(&mut o, threads as u64); o.push(" fils"); }
        if mhz > 0 { o.push(", "); push_num(&mut o, mhz as u64); o.push(" MHz"); }
    }
    out.add(o.as_str());
    // machine, carte mère, firmware
    if let Some((_, s, l)) = structures(table).find(|x| x.0 == 1) {
        o.clear(); o.push("Machine : "); o.push(string(table, s, l, b(table, s + 4))); o.push(" "); o.push(string(table, s, l, b(table, s + 5))); out.add(o.as_str());
    }
    if let Some((_, s, l)) = structures(table).find(|x| x.0 == 2) { o.clear(); o.push("Carte mère : "); o.push(string(table, s, l, b(table, s + 5))); out.add(o.as_str()); }
    if let Some((_, s, l)) = structures(table).find(|x| x.0 == 0) { o.clear(); o.push("Firmware : "); o.push(string(table, s, l, b(table, s + 4))); o.push(" "); o.push(string(table, s, l, b(table, s + 5))); out.add(o.as_str()); }
    // mémoire : une ligne par barrette installée, puis le total
    let (mut total, mut k) = (0u64, 0u64);
    for (_, s, l) in structures(table).filter(|x| x.0 == 17) {
        let mut mo = w(table, s + 0x0C) as u64;
        if mo == 0x7FFF { mo = d(table, s + 0x1C) as u64; } else if mo & 0x8000 != 0 { mo = (mo & 0x7FFF) / 1024; } // bit 15 : unités de 1 Kio
        k += 1;
        if mo == 0 || mo == 0xFFFF { continue; }
        total += mo;
        o.clear(); o.push("Barrette "); push_num(&mut o, k); o.push(" : "); push_num(&mut o, mo); o.push(" Mo, "); push_num(&mut o, w(table, s + 0x15) as u64); o.push(" MT/s, ");
        o.push(string(table, s, l, b(table, s + 0x17))); o.push(", "); o.push(string(table, s, l, b(table, s + 0x1A)));
        out.add(o.as_str());
    }
    o.clear(); o.push("Mémoire installée : "); push_num(&mut o, total); o.push(" Mo"); out.add(o.as_str());
}
