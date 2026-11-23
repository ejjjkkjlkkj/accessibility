//! Lecture des ressources de boîtes de dialogue (RT_DIALOG = 5) d'un fichier PE, sans exécuter le fichier, et contrôle d'accessibilité statique de leurs contrôles.
//! Précision : ne couvre QUE les gabarits de dialogue classiques (DLGTEMPLATE / DLGTEMPLATEEX). Les interfaces XAML / WinUI / HTML ne sont pas des gabarits de dialogue et ne sont pas analysées ici.
//! Toute lecture hors limites ou structure incohérente renvoie None / ignore l'élément : jamais de panique sur un fichier hostile.

fn u16at(b: &[u8], o: usize) -> Option<u16> { b.get(o..o + 2).map(|x| u16::from_le_bytes([x[0], x[1]])) }
fn u32at(b: &[u8], o: usize) -> Option<u32> { b.get(o..o + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]])) }

pub struct Pe<'a> { d: &'a [u8], secs: Vec<(u32, u32, u32)>, // (rva, taille virtuelle, offset fichier)
    rsrc_rva: u32 }
impl<'a> Pe<'a> {
    pub fn open(d: &'a [u8]) -> Option<Pe<'a>> {
        if d.get(0..2)? != b"MZ" { return None; } let pe = u32at(d, 0x3C)? as usize; if d.get(pe..pe + 4)? != b"PE\0\0" { return None; }
        let nsec = u16at(d, pe + 6)? as usize; let opt_size = u16at(d, pe + 20)? as usize; let opt = pe + 24; let magic = u16at(d, opt)?;
        let dd = opt + if magic == 0x20B { 112 } else if magic == 0x10B { 96 } else { return None };
        let rsrc_rva = u32at(d, dd + 2 * 8)?; let mut secs = vec![]; let st = opt + opt_size;
        for i in 0..nsec.min(96) { let s = st + i * 40; secs.push((u32at(d, s + 12)?, u32at(d, s + 8)?.max(u32at(d, s + 16)?), u32at(d, s + 20)?)); }
        Some(Pe { d, secs, rsrc_rva })
    }
    fn off(&self, rva: u32) -> Option<usize> { for &(va, sz, raw) in &self.secs { if rva >= va && rva < va.saturating_add(sz) { return Some((raw + (rva - va)) as usize); } } None }
    /// Entrée d'un répertoire de ressources : (id ou nom, est_sous_répertoire, décalage depuis la racine ressources).
    fn entries(&self, base: usize, dir: u32) -> Vec<(u32, bool, u32)> {
        let o = base + dir as usize; let n = u16at(self.d, o + 12).unwrap_or(0) as usize + u16at(self.d, o + 14).unwrap_or(0) as usize; let mut v = vec![];
        for i in 0..n.min(2048) { let e = o + 16 + i * 8; let (Some(id), Some(of)) = (u32at(self.d, e), u32at(self.d, e + 4)) else { break }; v.push((id, of & 0x8000_0000 != 0, of & 0x7FFF_FFFF)); }
        v
    }
    /// Octets de chaque gabarit de dialogue (une langue par gabarit) : (identifiant, octets).
    pub fn dialogs(&self) -> Vec<(u32, &'a [u8])> {
        let mut out = vec![]; let Some(base) = self.off(self.rsrc_rva) else { return out }; if self.rsrc_rva == 0 { return out; }
        for (ty, ty_dir, ty_off) in self.entries(base, 0) { if ty != 5 || !ty_dir { continue; }
            for (id, id_dir, id_off) in self.entries(base, ty_off) { if !id_dir { continue; }
                for (_lang, lang_dir, lang_off) in self.entries(base, id_off) { if lang_dir { continue; }
                    let e = base + lang_off as usize; let (Some(rva), Some(sz)) = (u32at(self.d, e), u32at(self.d, e + 4)) else { continue };
                    if let Some(o) = self.off(rva) { if let Some(b) = self.d.get(o..o.saturating_add(sz as usize)) { out.push((id, b)); } } } } }
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item { pub class: String, pub text: String, pub style: u32, pub id: u32 }
#[derive(Debug, Clone, PartialEq)]
pub struct Dialog { pub title: String, pub style: u32, pub items: Vec<Item> }

/// Chaîne ou ordinal terminé par 0 / 0xFFFF+ordinal, à partir de `o` (aligné sur 2). Renvoie (texte, ordinal, nouvelle position).
fn sz_or_ord(b: &[u8], o: usize) -> Option<(String, Option<u16>, usize)> {
    let first = u16at(b, o)?; if first == 0 { return Some((String::new(), None, o + 2)); } if first == 0xFFFF { return Some((String::new(), Some(u16at(b, o + 2)?), o + 4)); }
    let mut v = vec![]; let mut p = o; loop { let c = u16at(b, p)?; p += 2; if c == 0 { break; } v.push(c); if v.len() > 2048 { return None; } }
    Some((String::from_utf16_lossy(&v), None, p))
}
fn class_name(text: String, ord: Option<u16>) -> String { match ord { Some(0x80) => "Button".into(), Some(0x81) => "Edit".into(), Some(0x82) => "Static".into(), Some(0x83) => "ListBox".into(), Some(0x84) => "ScrollBar".into(), Some(0x85) => "ComboBox".into(), Some(n) => format!("#{n}"), None => text } }

pub fn parse_dialog(b: &[u8]) -> Option<Dialog> {
    let ex = u16at(b, 0)? == 1 && u16at(b, 2)? == 0xFFFF;
    let (style, cdit, mut o) = if ex { (u32at(b, 12)?, u16at(b, 16)? as usize, 26) } else { (u32at(b, 0)?, u16at(b, 8)? as usize, 18) };
    let (_menu, _o1, p) = sz_or_ord(b, o)?; o = p; let (c, co, p) = sz_or_ord(b, o)?; let _ = class_name(c, co); o = p; let (title, _t, p) = sz_or_ord(b, o)?; o = p;
    if style & 0x40 != 0 { o += if ex { 6 } else { 2 }; let (_f, _fo, p) = sz_or_ord(b, o)?; o = p; }
    let mut items = vec![];
    for _ in 0..cdit.min(512) {
        o = (o + 3) & !3;
        let (st, id, base) = if ex { (u32at(b, o + 8)?, u32at(b, o + 20)?, o + 24) } else { (u32at(b, o)?, u16at(b, o + 16)? as u32, o + 18) };
        let (c, co, p) = sz_or_ord(b, base)?; let (t, to, p) = sz_or_ord(b, p)?; let extra = u16at(b, p)? as usize; o = p + 2 + extra;
        items.push(Item { class: class_name(c, co), text: if to.is_some() { "#ordinal".into() } else { t }, style: st, id });
    }
    Some(Dialog { title, style, items })
}

pub const WS_TABSTOP: u32 = 0x0001_0000; pub const WS_DISABLED: u32 = 0x0800_0000; pub const WS_VISIBLE: u32 = 0x1000_0000;
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Defaut { SansTabulation, BoutonSansTexte, ChampSansEtiquette }
impl Defaut { pub fn nom(&self) -> &'static str { match self { Defaut::SansTabulation => "sans_tabulation", Defaut::BoutonSansTexte => "bouton_sans_texte", Defaut::ChampSansEtiquette => "champ_sans_etiquette" } } }

/// Contrôle d'accessibilité statique d'un dialogue : (index du contrôle, défaut).
pub fn lint(d: &Dialog) -> Vec<(usize, Defaut)> {
    let mut out = vec![];
    for (i, it) in d.items.iter().enumerate() {
        if it.style & WS_DISABLED != 0 { continue; }
        let k = it.style & 0xF; // sous-type de style pour les boutons
        let interactif = match it.class.as_str() { "Edit" | "ComboBox" | "ListBox" => true, "Button" => k != 7 /* groupe */, _ => false };
        if !interactif { continue; }
        // Boutons radio (BS_RADIOBUTTON 4, BS_AUTORADIOBUTTON 9) : dans un groupe seul le premier est un arrêt de tabulation, les autres se parcourent aux flèches : pas un défaut.
        let radio = it.class == "Button" && (k == 4 || k == 9);
        if it.style & WS_TABSTOP == 0 && !radio { out.push((i, Defaut::SansTabulation)); }
        if it.class == "Button" && it.text.trim().is_empty() { out.push((i, Defaut::BoutonSansTexte)); }
        if matches!(it.class.as_str(), "Edit" | "ComboBox" | "ListBox") { let etiquette = i > 0 && d.items[i - 1].class == "Static" && !d.items[i - 1].text.trim().is_empty() && d.items[i - 1].text != "#ordinal"; if !etiquette { out.push((i, Defaut::ChampSansEtiquette)); } }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn w16(v: &mut Vec<u8>, x: u16) { v.extend_from_slice(&x.to_le_bytes()); } fn w32(v: &mut Vec<u8>, x: u32) { v.extend_from_slice(&x.to_le_bytes()); }
    fn wsz(v: &mut Vec<u8>, s: &str) { for u in s.encode_utf16() { w16(v, u); } w16(v, 0); } fn align4(v: &mut Vec<u8>) { while v.len() % 4 != 0 { v.push(0); } }
    /// Gabarit DLGTEMPLATE classique à 4 contrôles : étiquette « Nom », champ (Tab), bouton « OK » (Tab), bouton SANS texte (sans Tab).
    fn gabarit() -> Vec<u8> {
        let mut v = vec![]; w32(&mut v, 0x80C8_0000); w32(&mut v, 0); w16(&mut v, 4); for x in [0u16, 0, 100, 60] { w16(&mut v, x); } w16(&mut v, 0); w16(&mut v, 0); wsz(&mut v, "Essai");
        let items: [(u32, u16, &str, u16); 4] = [(WS_VISIBLE, 0x82, "Nom", 1), (WS_VISIBLE | WS_TABSTOP, 0x81, "", 2), (WS_VISIBLE | WS_TABSTOP, 0x80, "OK", 3), (WS_VISIBLE, 0x80, "", 4)];
        for (st, cl, tx, id) in items { align4(&mut v); w32(&mut v, st); w32(&mut v, 0); for x in [0u16, 0, 10, 10] { w16(&mut v, x); } w16(&mut v, id); w16(&mut v, 0xFFFF); w16(&mut v, cl); wsz(&mut v, tx); w16(&mut v, 0); }
        v
    }
    #[test] fn classic_dialog_is_parsed_and_defects_are_found() {
        let d = parse_dialog(&gabarit()).expect("gabarit valide"); assert_eq!(d.title, "Essai"); assert_eq!(d.items.len(), 4);
        assert_eq!(d.items[0].class, "Static"); assert_eq!(d.items[2].text, "OK");
        let l = lint(&d); assert_eq!(l, vec![(3, Defaut::SansTabulation), (3, Defaut::BoutonSansTexte)], "le champ précédé de « Nom » est étiqueté ; le bouton muet est signalé deux fois");
    }
    #[test] fn truncated_or_hostile_input_never_panics() { let g = gabarit(); for cut in 0..g.len() { let _ = parse_dialog(&g[..cut]); } let _ = parse_dialog(&[0xFF; 64]); assert!(Pe::open(b"MZ\0\0").is_none()); assert!(Pe::open(&[]).is_none()); }
    #[test] fn radio_buttons_without_tabstop_are_not_defects() {
        let d = Dialog { title: "t".into(), style: 0, items: vec![Item { class: "Button".into(), text: "Option A".into(), style: WS_VISIBLE | 9, id: 1 }, Item { class: "Button".into(), text: "Option B".into(), style: WS_VISIBLE | 9, id: 2 }, Item { class: "Button".into(), text: "Case".into(), style: WS_VISIBLE | 3, id: 3 }] };
        assert_eq!(lint(&d), vec![(2, Defaut::SansTabulation)], "seule la case à cocher sans Tab est signalée");
    }
    #[test] fn field_without_label_is_reported() { let mut d = parse_dialog(&gabarit()).unwrap(); d.items.remove(0); assert!(lint(&d).contains(&(0, Defaut::ChampSansEtiquette))); }
}
