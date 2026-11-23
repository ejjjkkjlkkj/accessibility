//! Voxa Agent : un assistant qui EFFECTUE des taches, sur la machine, avec NOTRE modele d'intentions.
//! Chaine : saisie -> normalisation -> caracteristiques (trigrammes de lettres et mots hachees) -> perceptron int8 (voxa-neural) -> intention -> outil.
//! Regles de securite (non negociables) : (1) hors ligne, deterministe, sans allocation ; (2) un outil a risque (extinction, redemarrage) n'est jamais execute sans une
//! confirmation explicite au tour suivant ; (3) si le modele hesite (marge faible) ou s'il est absent, on retombe sur des regles simples, et si elles ne reconnaissent rien l'agent dit
//! qu'il n'a pas compris au lieu de deviner ; (4) l'agent n'agit que par les outils fournis par l'hote : il n'a aucun autre acces.
#![cfg_attr(not(test), no_std)]
use core::fmt::Write;
use voxa_neural::{argmax, relu, Dense, Q16};

pub const D: usize = 768; // 512 trigrammes + 256 mots
pub const H: usize = 64;
pub const C: usize = 15;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Intention { Inconnu = 0, Aide = 1, Materiel = 2, Memoire = 3, Pci = 4, Audio = 5, BiosLire = 6, BiosChercher = 7, Eteindre = 8, Redemarrer = 9, Repeter = 10, Heure = 11, TestSon = 12, Quitter = 13, Acpi = 14 }
impl Intention {
    pub fn depuis(n: usize) -> Intention { match n { 1 => Intention::Aide, 2 => Intention::Materiel, 3 => Intention::Memoire, 4 => Intention::Pci, 5 => Intention::Audio, 6 => Intention::BiosLire, 7 => Intention::BiosChercher, 8 => Intention::Eteindre, 9 => Intention::Redemarrer, 10 => Intention::Repeter, 11 => Intention::Heure, 12 => Intention::TestSon, 13 => Intention::Quitter, 14 => Intention::Acpi, _ => Intention::Inconnu } }
    /// Outil a risque : exige une confirmation explicite.
    pub fn risque(self) -> bool { matches!(self, Intention::Eteindre | Intention::Redemarrer) }
}

// ---------------------------------------------------------------- texte
/// Tampon de texte a capacite fixe (reponses de l'agent).
#[derive(Clone, Copy)]
pub struct Texte { b: [u8; 256], n: usize }
impl Texte { pub const fn vide() -> Texte { Texte { b: [0; 256], n: 0 } } pub fn as_str(&self) -> &str { core::str::from_utf8(&self.b[..self.n]).unwrap_or("") } pub fn clear(&mut self) { self.n = 0; } pub fn est_vide(&self) -> bool { self.n == 0 } }
impl Write for Texte { fn write_str(&mut self, s: &str) -> core::fmt::Result { for c in s.chars() { let l = c.len_utf8(); if self.n + l > self.b.len() { return Ok(()); } c.encode_utf8(&mut self.b[self.n..]); self.n += l; } Ok(()) } }

/// Texte normalise : minuscules ASCII sans accents, ponctuation et apostrophes remplacees par un espace, espaces simples.
pub struct Norm { pub b: [u8; 200], pub n: usize }
impl Norm { pub fn as_str(&self) -> &str { core::str::from_utf8(&self.b[..self.n]).unwrap_or("") } pub fn mots(&self) -> impl Iterator<Item = &str> { self.as_str().split(' ').filter(|m| !m.is_empty()) } }
pub fn normaliser(t: &str) -> Norm {
    let mut o = Norm { b: [0; 200], n: 0 }; let mut espace = true;
    for c in t.chars().flat_map(|c| c.to_lowercase()) {
        let a: &str = match c { 'à' | 'â' | 'ä' => "a", 'é' | 'è' | 'ê' | 'ë' => "e", 'î' | 'ï' => "i", 'ô' | 'ö' => "o", 'ù' | 'û' | 'ü' => "u", 'ç' => "c", 'œ' => "oe", 'a'..='z' | '0'..='9' => "", _ => " " };
        let mut buf = [0u8; 4];
        let s: &str = if a.is_empty() { c.encode_utf8(&mut buf) } else { a };
        for u in s.bytes() { if u == b' ' { if espace { continue; } espace = true; } else { espace = false; } if o.n < o.b.len() { o.b[o.n] = u; o.n += 1; } }
    }
    while o.n > 0 && o.b[o.n - 1] == b' ' { o.n -= 1; } o
}
fn fnv(b: &[u8]) -> u32 { let mut h = 2166136261u32; for &x in b { h ^= x as u32; h = h.wrapping_mul(16777619); } h }
/// Caracteristiques int8 (compte 0..3 x 40) : trigrammes de lettres (avec bornes de mot) dans les 512 premiers indices, mots entiers dans les 256 suivants.
pub fn caracteristiques(t: &str, x: &mut [i8; D]) {
    *x = [0; D]; let n = normaliser(t);
    for mot in n.mots() {
        let mut cadre = [b' '; 40]; let m = mot.as_bytes(); let l = m.len().min(36); cadre[1..1 + l].copy_from_slice(&m[..l]);
        for i in 0..l { let h = (fnv(&cadre[i..i + 3]) % 512) as usize; x[h] = (x[h] as i32 + 40).min(120) as i8; }
        let h = 512 + (fnv(m) % 256) as usize; x[h] = (x[h] as i32 + 40).min(120) as i8;
    }
}

// ---------------------------------------------------------------- modele
/// Modele embarque : `VAGT` + marge (i8) + echelles Q16 (u32 x2) + poids int8 et biais int32 des deux couches. Produit par `voxa-agent-train`.
pub struct Modele { w1: &'static [i8], b1: [i32; H], w2: &'static [i8], b2: [i32; C], s1: Q16, s2: Q16, pub seuil: i32 }
fn octets_i8(b: &'static [u8]) -> &'static [i8] { unsafe { core::slice::from_raw_parts(b.as_ptr() as *const i8, b.len()) } }
impl Modele {
    pub fn depuis_octets(b: &'static [u8]) -> Option<Modele> {
        if b.len() < 20 || &b[..4] != b"VAGT" { return None; }
        let (d, h, c) = (u16::from_le_bytes([b[4], b[5]]) as usize, u16::from_le_bytes([b[6], b[7]]) as usize, u16::from_le_bytes([b[8], b[9]]) as usize);
        if (d, h, c) != (D, H, C) { return None; }
        let seuil = b[10] as i8 as i32; let s1 = Q16(u32::from_le_bytes(b[12..16].try_into().ok()?)); let s2 = Q16(u32::from_le_bytes(b[16..20].try_into().ok()?));
        let mut p = 20; if b.len() != 20 + H * D + H * 4 + C * H + C * 4 { return None; }
        let w1 = octets_i8(&b[p..p + H * D]); p += H * D; let mut b1 = [0i32; H]; for i in 0..H { b1[i] = i32::from_le_bytes(b[p + 4 * i..p + 4 * i + 4].try_into().ok()?); } p += H * 4;
        let w2 = octets_i8(&b[p..p + C * H]); p += C * H; let mut b2 = [0i32; C]; for i in 0..C { b2[i] = i32::from_le_bytes(b[p + 4 * i..p + 4 * i + 4].try_into().ok()?); }
        Some(Modele { w1, b1, w2, b2, s1, s2, seuil })
    }
    /// (classe, marge entre les deux meilleurs scores). La marge sert a refuser de deviner.
    pub fn predire_x(&self, x: &[i8; D]) -> Option<(usize, i32)> {
        let mut h = [0i8; H]; Dense { inn: D, out: H, weights: self.w1, bias: &self.b1, scale: self.s1 }.forward(x, &mut h).ok()?; relu(&mut h);
        let mut o = [0i8; C]; Dense { inn: H, out: C, weights: self.w2, bias: &self.b2, scale: self.s2 }.forward(&h, &mut o).ok()?;
        let b = argmax(&o)?; let mut second = i32::MIN; for (i, v) in o.iter().enumerate() { if i != b && (*v as i32) > second { second = *v as i32; } }
        Some((b, o[b] as i32 - second))
    }
    pub fn predire(&self, texte: &str) -> (Intention, i32) {
        let mut x = [0i8; D]; caracteristiques(texte, &mut x);
        match self.predire_x(&x) { Some((c, m)) if m >= self.seuil => (Intention::depuis(c), m), Some((_, m)) => (Intention::Inconnu, m), None => (Intention::Inconnu, 0) }
    }
}
/// Le modele fourni avec cette version (None s'il est vide ou invalide : on utilise alors les regles).
pub fn modele() -> Option<Modele> { Modele::depuis_octets(include_bytes!("../data/modele.bin")) }

// ---------------------------------------------------------------- regles de repli (reference a battre)
pub fn regles(texte: &str) -> Intention {
    let n = normaliser(texte); let s = n.as_str(); let a = |m: &str| s.contains(m); let mot = |m: &str| n.mots().any(|w| w == m);
    if a("acpi") { return Intention::Acpi; }
    if a("eteind") || a("eteins") || a("eteign") || a("shutdown") || a("hors tension") || (a("arret") && (a("machine") || a("ordinateur") || mot("pc"))) || (mot("arrete") && (a("machine") || a("ordinateur") || mot("pc") || a("tout"))) { return Intention::Eteindre; }
    if a("redemar") || a("reboot") || a("relanc") || a("reinitial") { return Intention::Redemarrer; }
    if a("bios") || a("firmware") || a("uefi") || a("secure boot") || a("fast boot") || a("demarrage rapide") || a("demarrage securise") {
        return if a("valeur") || a("cherche") || a("trouve") || a("recherche") || a("option") || a("regl") && !a("lis") || a("defaut") || a("actif") || a("active") || a("vaut") || a("c est quoi") { Intention::BiosChercher } else { Intention::BiosLire };
    }
    if (a("test") || a("essai") || a("bip") || a("joue") || a("verifie") || a("bruit")) && (a("son") || a("audio") || a("haut parleur") || a("bip") || a("bruit")) { return Intention::TestSon; }
    if a("memoire") || mot("ram") || a("mega") || a("giga") { return Intention::Memoire; }
    if a("audio") || mot("son") || a("haut parleur") || a("carte son") { return Intention::Audio; }
    if a("pci") || a("periph") || a("composant") || a("inventaire") || mot("bus") { return Intention::Pci; }
    if a("materiel") || a("processeur") || a("machine") || a("caracteristique") || a("carte mere") || a("modele") || a("ordinateur") { return Intention::Materiel; }
    if a("heure") { return Intention::Heure; }
    if a("repet") || a("redis") || mot("encore") || a("pardon") || a("pas compris") { return Intention::Repeter; }
    if a("quitt") || a("retour") || a("sortir") || a("au revoir") || mot("fin") || a("assistant") || a("merci") { return Intention::Quitter; }
    if a("aide") || mot("help") || a("commande") || a("sais faire") || a("possibilit") || a("sers") { return Intention::Aide; }
    Intention::Inconnu
}

/// Decision finale a partir du modele (m) et des regles (r). REGLE DE SECURITE : une action a risque n'est retenue que si le modele ET les regles sont d'accord ; sinon on retombe sur l'intention sans risque ou on dit qu'on n'a pas compris.
pub fn choisir(m: Intention, r: Intention) -> (Intention, Source) {
    if m.risque() { return if r == m { (m, Source::Modele) } else if r != Intention::Inconnu && !r.risque() { (r, Source::Regles) } else { (Intention::Inconnu, Source::Aucune) }; }
    if m != Intention::Inconnu { return (m, Source::Modele); }
    if r.risque() { return (Intention::Inconnu, Source::Aucune); } // les regles seules ne declenchent pas une action a risque : le modele doit confirmer
    if r != Intention::Inconnu { (r, Source::Regles) } else { (Intention::Inconnu, Source::Aucune) }
}

// ---------------------------------------------------------------- agent
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action { Rien, Eteindre, Redemarrer, Quitter, OuvrirBios, Bip }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source { Modele, Regles, Aucune }
pub struct Reponse { pub texte: Texte, pub action: Action, pub intention: Intention, pub source: Source }

/// Les seuls outils de l'agent : l'hote les fournit. Tous sont en LECTURE SEULE ; les actions a effet passent par `Action`, apres confirmation.
pub trait Hote {
    fn materiel(&mut self, t: &mut Texte);
    fn memoire(&mut self, t: &mut Texte);
    fn pci(&mut self, t: &mut Texte);
    fn audio(&mut self, t: &mut Texte);
    fn acpi(&mut self, t: &mut Texte);
    fn heure(&mut self, t: &mut Texte) -> bool;
    /// Cherche les mots dans les ecrans du BIOS ; ecrit au plus quelques lignes ; retourne le nombre de lignes trouvees.
    fn bios_chercher(&mut self, mots: &[&str], t: &mut Texte) -> usize;
}

const MOTS_VIDES: [&str; 40] = ["bios", "dans", "quelle", "quel", "est", "valeur", "cherche", "trouve", "option", "reglage", "regarde", "firmware", "que", "vaut", "des", "les", "pour", "par", "defaut", "parametre", "parametres", "activ", "active", "actif", "moi", "me", "le", "la", "du", "de", "un", "une", "uefi", "si", "il", "c", "quoi", "comme", "dit", "sur"];

pub struct Agent { attente: Option<Intention>, derniere: Texte, modele: Option<Modele> }
impl Agent {
    pub fn nouveau() -> Agent { Agent { attente: None, derniere: Texte::vide(), modele: modele() } }
    /// Une confirmation en attente existe-t-elle ?
    pub fn attend_confirmation(&self) -> bool { self.attente.is_some() }
    pub fn tour<Ho: Hote>(&mut self, saisie: &str, h: &mut Ho) -> Reponse {
        let mut t = Texte::vide(); let n = normaliser(saisie);
        // confirmation d'une action a risque : seul un oui explicite l'execute ; tout le reste l'annule
        if let Some(i) = self.attente.take() {
            let oui = n.mots().any(|w| w == "oui" || w == "ok" || w == "confirme" || w == "confirmer" || w == "yes") || n.as_str().contains("d accord") || n.as_str().contains("vas y");
            if oui { let _ = t.write_str(if i == Intention::Eteindre { "Extinction en cours." } else { "Redemarrage en cours." }); return Reponse { texte: t, action: if i == Intention::Eteindre { Action::Eteindre } else { Action::Redemarrer }, intention: i, source: Source::Aucune }; }
            let _ = t.write_str("Annule. Je n'ai rien fait."); self.derniere = t; return Reponse { texte: t, action: Action::Rien, intention: Intention::Inconnu, source: Source::Aucune };
        }
        if n.n == 0 { let _ = t.write_str("Je vous ecoute."); return Reponse { texte: t, action: Action::Rien, intention: Intention::Inconnu, source: Source::Aucune }; }
        let pm = match &self.modele { Some(m) => m.predire(saisie).0, None => Intention::Inconnu };
        let (intention, source) = if self.modele.is_none() { let r = regles(saisie); (r, if r == Intention::Inconnu { Source::Aucune } else { Source::Regles }) } else { choisir(pm, regles(saisie)) };
        let mut action = Action::Rien; let mut garder = true;
        match intention {
            Intention::Inconnu => { let _ = t.write_str("Je n'ai pas compris. Dites aide pour connaitre ce que je sais faire."); garder = false; }
            Intention::Aide => { let _ = t.write_str("Je peux donner le materiel, la memoire, les peripheriques, l'audio, l'heure et les tables ACPI ; lire ou chercher dans le BIOS ; faire un bip ; eteindre ou redemarrer apres confirmation. Dites par exemple : combien de memoire."); }
            Intention::Materiel => h.materiel(&mut t), Intention::Memoire => h.memoire(&mut t), Intention::Pci => h.pci(&mut t), Intention::Audio => h.audio(&mut t), Intention::Acpi => h.acpi(&mut t),
            Intention::Heure => { if !h.heure(&mut t) { let _ = t.write_str("Je ne peux pas lire l'heure sur cette machine."); } }
            Intention::BiosLire => { let _ = t.write_str("J'ouvre la configuration du BIOS en lecture seule."); action = Action::OuvrirBios; }
            Intention::BiosChercher => {
                let mut mots: [&str; 4] = [""; 4]; let mut k = 0; for w in n.mots() { if w.len() >= 3 && !MOTS_VIDES.contains(&w) && k < 4 { mots[k] = w; k += 1; } }
                if k == 0 { let _ = t.write_str("Que dois-je chercher dans le BIOS ? Par exemple : fast boot, secure boot, usb."); garder = false; }
                else if h.bios_chercher(&mots[..k], &mut t) == 0 { let _ = t.write_str("Je n'ai rien trouve dans le BIOS pour ces mots."); }
            }
            Intention::Eteindre | Intention::Redemarrer => { let _ = t.write_str(if intention == Intention::Eteindre { "Je vais eteindre l'ordinateur. Dites oui pour confirmer, autre chose pour annuler." } else { "Je vais redemarrer l'ordinateur. Dites oui pour confirmer, autre chose pour annuler." }); self.attente = Some(intention); garder = false; }
            Intention::Repeter => { if self.derniere.est_vide() { let _ = t.write_str("Je n'ai rien a repeter."); } else { t = self.derniere; } garder = false; }
            Intention::TestSon => { let _ = t.write_str("Voici un bip."); action = Action::Bip; }
            Intention::Quitter => { let _ = t.write_str("Retour au menu."); action = Action::Quitter; garder = false; }
        }
        if garder { self.derniere = t; }
        Reponse { texte: t, action, intention, source }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Faux { appels: u32 }
    impl Hote for Faux {
        fn materiel(&mut self, t: &mut Texte) { self.appels += 1; let _ = t.write_str("Machine : modele de test."); }
        fn memoire(&mut self, t: &mut Texte) { self.appels += 1; let _ = t.write_str("Memoire utilisable : 1056 Mo."); }
        fn pci(&mut self, t: &mut Texte) { self.appels += 1; let _ = t.write_str("5 peripheriques PCI."); }
        fn audio(&mut self, t: &mut Texte) { self.appels += 1; let _ = t.write_str("2 controleurs audio."); }
        fn acpi(&mut self, t: &mut Texte) { self.appels += 1; let _ = t.write_str("2 tables ACPI."); }
        fn heure(&mut self, t: &mut Texte) -> bool { self.appels += 1; let _ = t.write_str("Il est 10 heures."); true }
        fn bios_chercher(&mut self, mots: &[&str], t: &mut Texte) -> usize { self.appels += 1; if mots.contains(&"fast") || mots.contains(&"usb") { let _ = t.write_str("Choix : Fast Boot."); 1 } else { 0 } }
    }
    #[test] fn normalisation() { assert_eq!(normaliser("  Éteins   l'ordinateur, S'IL TE PLAÎT !  ").as_str(), "eteins l ordinateur s il te plait"); assert_eq!(normaliser("cœur").as_str(), "coeur"); }
    #[test] fn caracteristiques_deterministes_et_bornees() { let (mut a, mut b) = ([0i8; D], [0i8; D]); caracteristiques("Quelle mémoire ?", &mut a); caracteristiques("quelle memoire", &mut b); assert_eq!(a, b); assert!(a.iter().any(|&v| v > 0)); assert!(a.iter().all(|&v| (0..=120).contains(&v))); }
    #[test] fn regles_de_reference() { for (p, i) in [("eteins l'ordinateur", Intention::Eteindre), ("redemarre la machine", Intention::Redemarrer), ("combien de memoire", Intention::Memoire), ("ouvre le bios", Intention::BiosLire), ("quelle est la valeur de fast boot dans le bios", Intention::BiosChercher), ("quelle heure est-il", Intention::Heure), ("liste les tables acpi", Intention::Acpi), ("fais un bip", Intention::TestSon), ("blague", Intention::Inconnu)] { assert_eq!(regles(p), i, "{p}"); } }
    #[test] fn extinction_exige_confirmation() {
        let mut a = Agent { attente: None, derniere: Texte::vide(), modele: None }; let mut h = Faux { appels: 0 };
        let r = a.tour("eteins l'ordinateur", &mut h); assert_eq!(r.action, Action::Rien); assert!(r.texte.as_str().contains("Dites oui")); assert!(a.attend_confirmation());
        let r = a.tour("oui", &mut h); assert_eq!(r.action, Action::Eteindre); assert!(!a.attend_confirmation());
    }
    #[test] fn tout_sauf_oui_annule() { for non in ["non", "peut-etre", "je ne sais pas", "eteins", ""] { let mut a = Agent { attente: None, derniere: Texte::vide(), modele: None }; let mut h = Faux { appels: 0 }; a.tour("redemarre la machine", &mut h); let r = a.tour(non, &mut h); assert_eq!(r.action, Action::Rien, "{non}"); assert!(!a.attend_confirmation(), "{non}"); } }
    #[test] fn inconnu_ne_fait_rien() { let mut a = Agent { attente: None, derniere: Texte::vide(), modele: None }; let mut h = Faux { appels: 0 }; let r = a.tour("raconte-moi une blague", &mut h); assert_eq!(r.action, Action::Rien); assert_eq!(h.appels, 0); assert!(r.texte.as_str().contains("pas compris")); }
    #[test] fn outils_lecture_seule_et_repetition() { let mut a = Agent { attente: None, derniere: Texte::vide(), modele: None }; let mut h = Faux { appels: 0 }; let r = a.tour("combien de memoire", &mut h); assert!(r.texte.as_str().contains("1056")); let r2 = a.tour("repete", &mut h); assert!(r2.texte.as_str().contains("1056")); assert_eq!(h.appels, 1); }
    #[test] fn recherche_bios_extrait_les_mots_utiles() { let mut a = Agent { attente: None, derniere: Texte::vide(), modele: None }; let mut h = Faux { appels: 0 }; let r = a.tour("quelle est la valeur de fast boot dans le bios", &mut h); assert!(r.texte.as_str().contains("Fast Boot")); let r = a.tour("cherche zzz dans le bios", &mut h); assert!(r.texte.as_str().contains("rien trouve")); }
    #[test] fn action_a_risque_exige_l_accord_modele_et_regles() { use Intention::*; assert_eq!(choisir(Eteindre, Eteindre).0, Eteindre); assert_eq!(choisir(Eteindre, Inconnu).0, Inconnu); assert_eq!(choisir(Eteindre, Memoire).0, Memoire); assert_eq!(choisir(Inconnu, Eteindre).0, Inconnu); assert_eq!(choisir(Redemarrer, Eteindre).0, Inconnu); assert_eq!(choisir(Memoire, Eteindre).0, Memoire); assert_eq!(choisir(Inconnu, Aide).0, Aide); }
    #[test] fn modele_embarque_si_present() { if let Some(m) = modele() { let (i, marge) = m.predire("eteins l'ordinateur"); assert_eq!(i, Intention::Eteindre); assert!(marge >= m.seuil); } }
}
