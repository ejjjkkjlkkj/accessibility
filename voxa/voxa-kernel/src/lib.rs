//! Fondations du noyau Voxa (no_std, sans allocation) : lecture de la carte mémoire UEFI et allocateur de pages à bitmap.
//! Statut : testé sur l'hôte avec des cartes mémoire synthétiques ; PAS encore exécuté sur un firmware réel (NOT_VALIDATED).
#![cfg_attr(not(test), no_std)]

pub mod efi;
pub mod hw;
pub mod usb;
pub const PAGE: u64 = 4096;
/// Capacité de l'allocateur : 4 Gio de mémoire physique suivie (1 048 576 pages, bitmap de 128 Kio).
pub const MAX_PAGES: usize = 1 << 20;
const WORDS: usize = MAX_PAGES / 64;

/// Types de mémoire UEFI (EFI_MEMORY_TYPE) dont la mémoire est libre après ExitBootServices.
pub fn reusable(mem_type: u32) -> bool { matches!(mem_type, 1 | 2 | 3 | 4 | 7) } // LoaderCode, LoaderData, BootCode, BootData, Conventional

/// Un descripteur UEFI (les champs utiles). `pages` = nombre de pages de 4 Kio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region { pub kind: u32, pub start: u64, pub pages: u64 }

/// Lit un descripteur dans le tampon renvoyé par GetMemoryMap ; `stride` = DescriptorSize (≥ 40, peut dépasser la taille de la structure).
pub fn parse_descriptor(buf: &[u8], idx: usize, stride: usize) -> Option<Region> {
    let o = idx.checked_mul(stride)?;
    let d = buf.get(o..o.checked_add(40)?)?;
    let r32 = |i: usize| u32::from_le_bytes([d[i], d[i + 1], d[i + 2], d[i + 3]]);
    let r64 = |i: usize| u64::from_le_bytes([d[i], d[i + 1], d[i + 2], d[i + 3], d[i + 4], d[i + 5], d[i + 6], d[i + 7]]);
    Some(Region { kind: r32(0), start: r64(8), pages: r64(24) })
}

/// Allocateur de pages physiques : bit à 1 = page occupée. Tout est occupé au départ ; seules les régions libérables sont ouvertes.
pub struct FrameAllocator { bits: [u64; WORDS], free: usize, hint: usize, ignored_pages: u64 }

impl FrameAllocator {
    pub const fn new() -> Self { FrameAllocator { bits: [!0u64; WORDS], free: 0, hint: 0, ignored_pages: 0 } }

    /// Construit depuis la carte mémoire. La page 0 reste toujours réservée (pointeur nul). Les pages au-delà de 4 Gio sont comptées dans `ignored_pages`.
    pub fn from_map(buf: &[u8], count: usize, stride: usize) -> Self { let mut a = Self::new(); a.fill_from_map(buf, count, stride); a }

    /// Remplit en place (sans copie de 128 Kio sur la pile) : à utiliser pour un allocateur statique.
    pub fn fill_from_map(&mut self, buf: &[u8], count: usize, stride: usize) {
        self.bits = [!0u64; WORDS]; self.free = 0; self.hint = 0; self.ignored_pages = 0;
        for i in 0..count { if let Some(r) = parse_descriptor(buf, i, stride) { if reusable(r.kind) { self.release_range(r.start, r.pages); } } }
        self.reserve(0);
    }

    fn release_range(&mut self, start: u64, pages: u64) {
        let first = start / PAGE;
        for p in first..first.saturating_add(pages) {
            if (p as usize) < MAX_PAGES { self.clear_bit(p as usize); } else { self.ignored_pages += 1; }
        }
    }
    fn clear_bit(&mut self, p: usize) { let (w, b) = (p / 64, p % 64); if self.bits[w] >> b & 1 == 1 { self.bits[w] &= !(1 << b); self.free += 1; } }
    fn set_bit(&mut self, p: usize) { let (w, b) = (p / 64, p % 64); if self.bits[w] >> b & 1 == 0 { self.bits[w] |= 1 << b; self.free -= 1; } }

    /// Marque une page comme occupée (ex. page 0, image du noyau).
    pub fn reserve(&mut self, page: usize) { if page < MAX_PAGES { self.set_bit(page); } }
    pub fn reserve_range(&mut self, addr: u64, bytes: u64) { let a = addr / PAGE; let b = (addr + bytes + PAGE - 1) / PAGE; for p in a..b { self.reserve(p as usize); } }

    pub fn free_pages(&self) -> usize { self.free }
    pub fn free_bytes(&self) -> u64 { self.free as u64 * PAGE }
    pub fn ignored_pages(&self) -> u64 { self.ignored_pages }

    /// Alloue une page ; renvoie son adresse physique.
    pub fn alloc(&mut self) -> Option<u64> { self.alloc_contiguous(1) }

    /// Alloue `n` pages contiguës (premier ajustement à partir du dernier succès, puis depuis le début).
    pub fn alloc_contiguous(&mut self, n: usize) -> Option<u64> {
        if n == 0 || n > self.free { return None; }
        for pass in 0..2 {
            let (from, to) = if pass == 0 { (self.hint, MAX_PAGES) } else { (0, self.hint.saturating_add(n).min(MAX_PAGES)) };
            let mut run = 0usize; let mut p = from;
            while p < to {
                if self.bits[p / 64] >> (p % 64) & 1 == 0 { run += 1; if run == n { let s = p + 1 - n; for q in s..=p { self.set_bit(q); } self.hint = p + 1; return Some(s as u64 * PAGE); } }
                else { run = 0; if self.bits[p / 64] == !0u64 { p = (p / 64 + 1) * 64; continue; } }
                p += 1;
            }
        }
        None
    }

    /// Libère une page. Renvoie false (sans rien changer) pour une adresse non alignée, hors plage ou déjà libre : une double libération ne corrompt pas le compteur.
    pub fn free(&mut self, addr: u64) -> bool {
        if addr % PAGE != 0 { return false; }
        let p = (addr / PAGE) as usize;
        if p == 0 || p >= MAX_PAGES || self.bits[p / 64] >> (p % 64) & 1 == 0 { return false; }
        self.clear_bit(p); if p < self.hint { self.hint = p; } true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn desc(kind: u32, start: u64, pages: u64) -> [u8; 48] { let mut d = [0u8; 48]; d[0..4].copy_from_slice(&kind.to_le_bytes()); d[8..16].copy_from_slice(&start.to_le_bytes()); d[24..32].copy_from_slice(&pages.to_le_bytes()); d }
    fn map(regions: &[(u32, u64, u64)]) -> std::vec::Vec<u8> { let mut v = std::vec::Vec::new(); for r in regions { v.extend_from_slice(&desc(r.0, r.1, r.2)); } v }
    #[test] fn descriptor_uses_the_firmware_stride() {
        let mut v = std::vec::Vec::new(); for r in [(7u32, 0x1000u64, 4u64), (0, 0x9000, 2)] { v.extend_from_slice(&desc(r.0, r.1, r.2)); v.extend_from_slice(&[0xAA; 8]); } // stride 56
        assert_eq!(parse_descriptor(&v, 1, 56), Some(Region { kind: 0, start: 0x9000, pages: 2 }));
        assert_eq!(parse_descriptor(&v, 2, 56), None, "hors tampon");
    }
    #[test] fn only_reusable_types_are_opened_and_page_zero_is_reserved() {
        let m = map(&[(7, 0, 16), (11, 0x10_000, 8), (5, 0x20_000, 8), (2, 0x30_000, 4)]); // conventionnelle, MMIO, runtime code, loader data
        let a = FrameAllocator::from_map(&m, 4, 48);
        assert_eq!(a.free_pages(), 15 + 4, "page 0 exclue, MMIO et services d'exécution exclus");
    }
    #[test] fn alloc_returns_distinct_aligned_pages_until_exhausted() {
        let m = map(&[(7, 0x10_000, 3)]); let mut a = FrameAllocator::from_map(&m, 1, 48);
        let x = a.alloc().unwrap(); let y = a.alloc().unwrap(); let z = a.alloc().unwrap();
        assert!(x % PAGE == 0 && x != y && y != z && x != z); assert_eq!(a.alloc(), None); assert_eq!(a.free_pages(), 0);
    }
    #[test] fn free_reuses_and_rejects_double_free_and_bad_addresses() {
        let m = map(&[(7, 0x10_000, 4)]); let mut a = FrameAllocator::from_map(&m, 1, 48);
        let x = a.alloc().unwrap(); assert_eq!(a.free_pages(), 3);
        assert!(a.free(x)); assert!(!a.free(x), "double libération refusée"); assert_eq!(a.free_pages(), 4);
        assert!(!a.free(x + 1) && !a.free(0) && !a.free(1 << 40));
        assert_eq!(a.alloc(), Some(x), "la page libérée est réutilisée");
    }
    #[test] fn contiguous_allocation_needs_a_real_run() {
        let m = map(&[(7, 0x10_000, 2), (7, 0x20_000, 4)]); let mut a = FrameAllocator::from_map(&m, 2, 48);
        assert_eq!(a.alloc_contiguous(4), Some(0x20_000)); assert_eq!(a.alloc_contiguous(3), None, "2 pages restantes seulement");
        assert_eq!(a.alloc_contiguous(2), Some(0x10_000)); assert_eq!(a.alloc_contiguous(0), None);
    }
    #[test] fn reserved_ranges_are_never_handed_out_and_memory_above_4gib_is_counted() {
        let m = map(&[(7, 0x10_000, 8), (7, 0x2_0000_0000, 5)]); let mut a = FrameAllocator::from_map(&m, 2, 48);
        a.reserve_range(0x10_000, 3 * PAGE + 1); // 4 pages (arrondi supérieur)
        assert_eq!(a.free_pages(), 4); assert_eq!(a.ignored_pages(), 5, "5 pages au-delà de 4 Gio signalées, pas perdues en silence");
        let mut got = std::vec::Vec::new(); while let Some(p) = a.alloc() { got.push(p); }
        assert!(got.iter().all(|&p| p >= 0x10_000 + 4 * PAGE && p < 0x10_000 + 8 * PAGE));
    }
}

/// COUCHE 0 : la voix est le premier service du noyau, avant l'allocateur. Tout ce que le noyau fait ou rate passe d'abord par `Voice`.
/// Aucune allocation, aucune dépendance : une implémentation peut être le pilote audio, les bips du haut-parleur ou l'écran (secours).
pub trait Voice { fn say(&mut self, text: &str); }

/// Écrit un entier décimal dans un tampon fixe et le renvoie comme texte.
fn num(buf: &mut [u8; 20], mut n: u64) -> &str {
    let mut i = buf.len(); if n == 0 { i -= 1; buf[i] = b'0'; }
    while n > 0 { i -= 1; buf[i] = b'0' + (n % 10) as u8; n /= 10; }
    core::str::from_utf8(&buf[i..]).unwrap_or("?")
}

/// Étapes de démarrage, annoncées dans l'ordre. Une panne est dite AVANT toute tentative de poursuite.
pub fn boot_memory<V: Voice>(v: &mut V, map: &[u8], count: usize, stride: usize) -> Option<FrameAllocator> {
    v.say("Noyau Voxa. Voix active avant la mémoire.");
    if count == 0 || stride < 40 || map.len() < count.saturating_mul(stride) { v.say("Panne au démarrage : carte mémoire illisible. Arrêt."); return None; }
    let a = FrameAllocator::from_map(map, count, stride);
    if a.free_pages() < 256 { v.say("Panne au démarrage : moins de 1 Mo de mémoire utilisable. Arrêt."); return None; }
    let mut b = [0u8; 20]; let mut m = [0u8; 160]; let mut n = 0;
    for part in ["Mémoire utilisable : ", num(&mut b, a.free_bytes() >> 20), " Mo."] { let p = part.as_bytes(); m[n..n + p.len()].copy_from_slice(p); n += p.len(); }
    v.say(core::str::from_utf8(&m[..n]).unwrap_or("Mémoire utilisable."));
    if a.ignored_pages() > 0 { v.say("Attention : de la mémoire au-delà de 4 Go n'est pas utilisée."); }
    Some(a)
}

/// Appelé par le gestionnaire de panique du noyau : dit la cause, pas seulement un écran figé.
pub fn announce_panic<V: Voice>(v: &mut V, reason: &str) { v.say("Panne du noyau."); v.say(reason); }

#[cfg(test)]
mod voice_tests {
    use super::*;
    struct Fake(std::vec::Vec<std::string::String>);
    impl Voice for Fake { fn say(&mut self, t: &str) { self.0.push(t.into()); } }
    fn d(kind: u32, start: u64, pages: u64) -> std::vec::Vec<u8> { let mut x = [0u8; 48]; x[0..4].copy_from_slice(&kind.to_le_bytes()); x[8..16].copy_from_slice(&start.to_le_bytes()); x[24..32].copy_from_slice(&pages.to_le_bytes()); x.to_vec() }
    #[test] fn voice_speaks_before_memory_and_reports_the_amount() {
        let mut v = Fake(vec![]); let m = d(7, 0x10_0000, 2560); // 10 Mo
        assert!(boot_memory(&mut v, &m, 1, 48).is_some());
        assert_eq!(v.0[0], "Noyau Voxa. Voix active avant la mémoire."); assert_eq!(v.0[1], "Mémoire utilisable : 10 Mo.");
    }
    #[test] fn failures_are_spoken_not_silent() {
        let mut v = Fake(vec![]); assert!(boot_memory(&mut v, &[], 0, 48).is_none()); assert!(v.0.last().unwrap().starts_with("Panne au démarrage : carte mémoire illisible"));
        let mut v = Fake(vec![]); assert!(boot_memory(&mut v, &d(7, 0x10_000, 10), 1, 48).is_none()); assert!(v.0.last().unwrap().contains("moins de 1 Mo"));
        let mut v = Fake(vec![]); assert!(boot_memory(&mut v, &d(7, 0x10_000, 10), 5, 48).is_none(), "tampon plus court que annoncé");
        let mut v = Fake(vec![]); announce_panic(&mut v, "double libération"); assert_eq!(v.0, ["Panne du noyau.", "double libération"]);
    }
    #[test] fn memory_beyond_4gib_is_announced() {
        let mut m = d(7, 0x10_0000, 2560); m.extend(d(7, 0x2_0000_0000, 3)); let mut v = Fake(vec![]);
        assert!(boot_memory(&mut v, &m, 2, 48).is_some()); assert!(v.0.iter().any(|t| t.starts_with("Attention : de la mémoire au-delà de 4 Go")));
    }
}

// ===== Modèle de pilote Voxa : tout pilote s'écrit contre ce trait, et chaque liaison ou échec est DIT par la voix (couche 0). =====

/// Identité d'un périphérique PCI (lue dans l'espace de configuration) ; `class`/`subclass` = octets 0x0B/0x0A.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciId { pub vendor: u16, pub device: u16, pub class: u8, pub subclass: u8 }

impl PciId {
    /// Décode les 16 premiers octets de l'espace de configuration PCI. Fournisseur 0xFFFF = emplacement vide.
    pub fn from_config(c: &[u8]) -> Option<PciId> {
        if c.len() < 16 { return None; }
        let vendor = u16::from_le_bytes([c[0], c[1]]); if vendor == 0xFFFF { return None; }
        Some(PciId { vendor, device: u16::from_le_bytes([c[2], c[3]]), subclass: c[10], class: c[11] })
    }
}

/// Importance pour un utilisateur aveugle : un pilote vital manquant est annoncé comme une panne, pas passé sous silence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Need { Vital, Important, Optional }

pub trait Driver {
    fn name(&self) -> &'static str;
    fn need(&self) -> Need;
    fn matches(&self, id: &PciId) -> bool;
    /// Initialise le périphérique. L'erreur est un texte français dit à l'utilisateur.
    fn init(&mut self, id: &PciId) -> Result<(), &'static str>;
}

/// Résultat de la liaison : combien de pilotes sont actifs, combien ont échoué.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct BindReport { pub bound: usize, pub failed: usize, pub vital_missing: usize }

/// Lie chaque périphérique au premier pilote qui le reconnaît et DIT le résultat. `needs_vital` = noms des pilotes vitaux attendus
/// (ex. audio) : s'ils n'ont lié aucun périphérique, c'est annoncé comme une panne.
pub fn bind_all<V: Voice>(v: &mut V, devices: &[PciId], drivers: &mut [&mut dyn Driver]) -> BindReport {
    let mut r = BindReport::default(); let mut bound_names = [false; 32];
    for id in devices {
        for (i, d) in drivers.iter_mut().enumerate() {
            if !d.matches(id) { continue; }
            match d.init(id) {
                Ok(()) => { r.bound += 1; if i < 32 { bound_names[i] = true; } v.say(d.name()); v.say("actif."); }
                Err(e) => { r.failed += 1; v.say("Échec du pilote "); v.say(d.name()); v.say(e); }
            }
            break;
        }
    }
    for (i, d) in drivers.iter().enumerate() {
        if d.need() == Need::Vital && !(i < 32 && bound_names[i]) { r.vital_missing += 1; v.say("Pilote vital absent : "); v.say(d.name()); }
    }
    r
}

#[cfg(test)]
mod driver_tests {
    use super::*;
    struct Fake(std::vec::Vec<std::string::String>);
    impl Voice for Fake { fn say(&mut self, t: &str) { self.0.push(t.into()); } }
    struct Hda { ok: bool, inits: usize }
    impl Driver for Hda {
        fn name(&self) -> &'static str { "Audio haute définition" } fn need(&self) -> Need { Need::Vital }
        fn matches(&self, id: &PciId) -> bool { id.class == 4 && id.subclass == 3 }
        fn init(&mut self, _: &PciId) -> Result<(), &'static str> { self.inits += 1; if self.ok { Ok(()) } else { Err("codec muet") } }
    }
    fn cfg(v: u16, d: u16, class: u8, sub: u8) -> [u8; 16] { let mut c = [0u8; 16]; c[0..2].copy_from_slice(&v.to_le_bytes()); c[2..4].copy_from_slice(&d.to_le_bytes()); c[10] = sub; c[11] = class; c }
    #[test] fn config_space_is_decoded_and_empty_slots_ignored() {
        assert_eq!(PciId::from_config(&cfg(0x1022, 0x1457, 4, 3)), Some(PciId { vendor: 0x1022, device: 0x1457, class: 4, subclass: 3 }));
        assert_eq!(PciId::from_config(&cfg(0xFFFF, 0xFFFF, 0xFF, 0xFF)), None); assert_eq!(PciId::from_config(&[0; 8]), None);
    }
    #[test] fn bound_failed_and_missing_vital_drivers_are_all_spoken() {
        let audio = PciId::from_config(&cfg(0x1022, 0x1457, 4, 3)).unwrap(); let gpu = PciId::from_config(&cfg(0x1002, 0x1638, 3, 0)).unwrap();
        let mut h = Hda { ok: true, inits: 0 }; let mut v = Fake(vec![]);
        let r = bind_all(&mut v, &[gpu, audio], &mut [&mut h]);
        assert_eq!(r, BindReport { bound: 1, failed: 0, vital_missing: 0 }); assert_eq!(v.0, ["Audio haute définition", "actif."]); assert_eq!(h.inits, 1, "le GPU n'est pas confié au pilote audio");
        let mut h = Hda { ok: false, inits: 0 }; let mut v = Fake(vec![]);
        let r = bind_all(&mut v, &[audio], &mut [&mut h]); assert_eq!(r, BindReport { bound: 0, failed: 1, vital_missing: 1 });
        assert!(v.0.contains(&"codec muet".to_string()) && v.0.contains(&"Pilote vital absent : ".to_string()));
        let mut h = Hda { ok: true, inits: 0 }; let mut v = Fake(vec![]);
        let r = bind_all(&mut v, &[gpu], &mut [&mut h]); assert_eq!(r.vital_missing, 1, "aucun périphérique audio trouvé : panne annoncée, pas silence");
    }
}
