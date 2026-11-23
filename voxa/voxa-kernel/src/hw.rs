//! Pilotes matériels écrits pour Voxa, derrière le trait `Ports` (E/S x86) pour être testés sur l'hôte avec des contrôleurs simulés :
//! énumération PCI (ports 0xCF8/0xCFC), clavier 8042 (ports 0x60/0x64, jeu de codes 1, disposition AZERTY), haut-parleur PC (0x42/0x43/0x61).
//! Statut : PASS en test hôte contre des modèles ; AUCUN n'a été exécuté sur le matériel réel (NOT_VALIDATED).
use crate::{PciId, Voice};

/// Accès aux ports d'E/S. Sur le matériel : instructions in/out ; dans les tests : un contrôleur simulé.
pub trait Ports { fn inb(&mut self, port: u16) -> u8; fn outb(&mut self, port: u16, v: u8); fn inl(&mut self, port: u16) -> u32; fn outl(&mut self, port: u16, v: u32); }

// ---------------------------------------------------------------- PCI
pub fn pci_read32<P: Ports>(p: &mut P, bus: u8, dev: u8, func: u8, off: u8) -> u32 {
    p.outl(0xCF8, 0x8000_0000 | (bus as u32) << 16 | (dev as u32) << 11 | (func as u32) << 8 | (off & 0xFC) as u32);
    p.inl(0xCFC)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciDevice { pub bus: u8, pub dev: u8, pub func: u8, pub id: PciId }

/// Énumère les bus 0..=max_bus (fonctions 0..7 si le bit multifonction est posé). Remplit `out` ; renvoie le nombre trouvé (borné par `out.len()`).
pub fn pci_enumerate<P: Ports>(p: &mut P, max_bus: u8, out: &mut [PciDevice]) -> usize {
    let mut n = 0;
    for bus in 0..=max_bus { for dev in 0..32u8 {
        let id0 = pci_read32(p, bus, dev, 0, 0); if id0 & 0xFFFF == 0xFFFF { continue; }
        let multi = (pci_read32(p, bus, dev, 0, 0x0C) >> 16) & 0x80 != 0;
        for func in 0..(if multi { 8 } else { 1 }) {
            let a = pci_read32(p, bus, dev, func, 0); if a & 0xFFFF == 0xFFFF { continue; }
            let c = pci_read32(p, bus, dev, func, 8);
            if n < out.len() { out[n] = PciDevice { bus, dev, func, id: PciId { vendor: a as u16, device: (a >> 16) as u16, class: (c >> 24) as u8, subclass: (c >> 16) as u8 } }; n += 1; }
        }
    } }
    n
}

/// Nom français d'une classe PCI, pour que le système puisse DIRE chaque périphérique.
pub fn class_name(class: u8, sub: u8) -> &'static str {
    match (class, sub) {
        (0x01, 0x08) => "contrôleur de stockage NVMe", (0x01, 0x06) => "contrôleur de stockage SATA", (0x01, _) => "contrôleur de stockage",
        (0x02, 0x00) => "carte réseau Ethernet", (0x02, 0x80) => "carte réseau sans fil", (0x02, _) => "carte réseau",
        (0x03, _) => "contrôleur graphique", (0x04, 0x01) => "contrôleur audio", (0x04, 0x03) => "contrôleur audio haute définition", (0x04, _) => "contrôleur multimédia",
        (0x05, _) => "contrôleur de mémoire", (0x06, 0x00) => "pont hôte", (0x06, 0x01) => "pont ISA", (0x06, 0x04) => "pont PCI", (0x06, _) => "pont",
        (0x07, _) => "contrôleur de communication", (0x08, 0x06) => "contrôleur IOMMU", (0x08, _) => "périphérique système",
        (0x0C, 0x03) => "contrôleur USB", (0x0C, 0x05) => "contrôleur SMBus", (0x0C, _) => "contrôleur de bus série",
        (0x0D, _) => "contrôleur sans fil", (0x10, _) => "contrôleur de chiffrement", (0x11, _) => "contrôleur de signal", _ => "périphérique inconnu",
    }
}

fn push_num(buf: &mut [u8], n: &mut usize, mut v: u32) { let mut t = [0u8; 10]; let mut i = t.len(); if v == 0 { i -= 1; t[i] = b'0'; } while v > 0 { i -= 1; t[i] = b'0' + (v % 10) as u8; v /= 10; } for &b in &t[i..] { buf[*n] = b; *n += 1; } }

/// Dit « N périphériques PCI », puis chaque classe avec son nombre (ex. « 2 contrôleur USB »), pour que rien ne reste muet.
pub fn announce_pci<V: Voice>(v: &mut V, devs: &[PciDevice]) {
    let mut buf = [0u8; 64]; let mut n = 0; push_num(&mut buf, &mut n, devs.len() as u32);
    let s = " périphériques sur le bus PCI.".as_bytes(); buf[n..n + s.len()].copy_from_slice(s); n += s.len(); v.say(core::str::from_utf8(&buf[..n]).unwrap_or("Périphériques PCI."));
    let mut done = [false; 64];
    for (i, d) in devs.iter().enumerate().take(64) {
        if done[i] { continue; }
        let name = class_name(d.id.class, d.id.subclass); let mut count = 0u32;
        for (j, e) in devs.iter().enumerate().take(64) { if class_name(e.id.class, e.id.subclass) == name { count += 1; done[j] = true; } }
        let mut b = [0u8; 80]; let mut m = 0; push_num(&mut b, &mut m, count); b[m] = b' '; m += 1;
        let nb = name.as_bytes(); let k = nb.len().min(b.len() - m); b[m..m + k].copy_from_slice(&nb[..k]); m += k;
        v.say(core::str::from_utf8(&b[..m]).unwrap_or("périphérique"));
    }
}

// ---------------------------------------------------------------- Clavier 8042
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key { Char(char), Enter, Escape, Backspace, Tab, Delete, Up, Down, Left, Right, Home, End, PageUp, PageDown }

/// Décodeur du jeu de codes 1 (traduction par défaut du 8042), disposition AZERTY française. Gère E0 (touches étendues), relâchements et Maj.
#[derive(Default)]
pub struct Kbd { ext: bool, shift: bool }

const AZERTY: [(u8, char, char); 49] = [
    (0x02, '&', '1'), (0x03, 'é', '2'), (0x04, '"', '3'), (0x05, '\'', '4'), (0x06, '(', '5'), (0x07, '-', '6'), (0x08, 'è', '7'), (0x09, '_', '8'), (0x0A, 'ç', '9'), (0x0B, 'à', '0'), (0x0C, ')', '°'), (0x0D, '=', '+'),
    (0x10, 'a', 'A'), (0x11, 'z', 'Z'), (0x12, 'e', 'E'), (0x13, 'r', 'R'), (0x14, 't', 'T'), (0x15, 'y', 'Y'), (0x16, 'u', 'U'), (0x17, 'i', 'I'), (0x18, 'o', 'O'), (0x19, 'p', 'P'),
    (0x1E, 'q', 'Q'), (0x1F, 's', 'S'), (0x20, 'd', 'D'), (0x21, 'f', 'F'), (0x22, 'g', 'G'), (0x23, 'h', 'H'), (0x24, 'j', 'J'), (0x25, 'k', 'K'), (0x26, 'l', 'L'), (0x27, 'm', 'M'),
    (0x2C, 'w', 'W'), (0x2D, 'x', 'X'), (0x2E, 'c', 'C'), (0x2F, 'v', 'V'), (0x30, 'b', 'B'), (0x31, 'n', 'N'), (0x32, ',', '?'), (0x33, ';', '.'), (0x34, ':', '/'), (0x35, '!', '§'),
    (0x39, ' ', ' '), (0x1A, '^', '¨'), (0x1B, '$', '£'), (0x28, 'ù', '%'), (0x2B, '*', 'µ'), (0x29, '²', '²'), (0x56, '<', '>'),
];

impl Kbd {
    /// Traite un octet du contrôleur ; renvoie une touche si l'octet complète un appui (jamais pour un relâchement ou un préfixe).
    pub fn feed(&mut self, b: u8) -> Option<Key> {
        if b == 0xE0 { self.ext = true; return None; }
        let (code, release) = (b & 0x7F, b & 0x80 != 0); let ext = core::mem::take(&mut self.ext);
        if !ext && (code == 0x2A || code == 0x36) { self.shift = !release; return None; }
        if release { return None; }
        if ext { return match code { 0x48 => Some(Key::Up), 0x50 => Some(Key::Down), 0x4B => Some(Key::Left), 0x4D => Some(Key::Right), 0x47 => Some(Key::Home), 0x4F => Some(Key::End), 0x49 => Some(Key::PageUp), 0x51 => Some(Key::PageDown), 0x53 => Some(Key::Delete), 0x1C => Some(Key::Enter), _ => None }; }
        key_from_set1(code, self.shift)
    }
}

/// Touche (disposition AZERTY) pour une position physique du jeu de codes 1 ; partagé avec le clavier USB.
pub fn key_from_set1(code: u8, shift: bool) -> Option<Key> {
    match code { 0x01 => Some(Key::Escape), 0x0E => Some(Key::Backspace), 0x0F => Some(Key::Tab), 0x1C => Some(Key::Enter),
        _ => AZERTY.iter().find(|e| e.0 == code).map(|e| Key::Char(if shift { e.2 } else { e.1 })) }
}

/// Lit tout ce que le contrôleur a en attente (bit 0 du port 0x64 = tampon plein ; bit 5 = donnée de souris, ignorée ici). Borné à 16 octets par appel.
pub fn kbd_poll<P: Ports>(p: &mut P, k: &mut Kbd, out: &mut [Key]) -> usize {
    let mut n = 0;
    for _ in 0..16 {
        let st = p.inb(0x64); if st & 1 == 0 { break; }
        let b = p.inb(0x60); if st & 0x20 != 0 { continue; }
        if let Some(key) = k.feed(b) { if n < out.len() { out[n] = key; n += 1; } }
    }
    n
}

// ---------------------------------------------------------------- Haut-parleur PC
/// Joue une fréquence (Hz) : programme le canal 2 du PIT (0x42/0x43) puis ouvre la porte (0x61 bits 0-1). `hz == 0` coupe le son.
pub fn speaker<P: Ports>(p: &mut P, hz: u32) {
    if hz == 0 { let v = p.inb(0x61); p.outb(0x61, v & !3); return; }
    let div = (1_193_182 / hz.clamp(20, 20_000)) as u16;
    p.outb(0x43, 0xB6); p.outb(0x42, div as u8); p.outb(0x42, (div >> 8) as u8);
    let v = p.inb(0x61); p.outb(0x61, v | 3);
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Machine simulée : espace de configuration PCI, file d'octets du clavier, journal des écritures de ports.
    struct Sim { cfg: std::collections::HashMap<(u8, u8, u8, u8), u32>, addr: u32, kbd: std::collections::VecDeque<u8>, log: std::vec::Vec<(u16, u8)>, p61: u8 }
    impl Sim { fn new() -> Self { Sim { cfg: Default::default(), addr: 0, kbd: Default::default(), log: vec![], p61: 0 } }
        fn dev(&mut self, bus: u8, dev: u8, func: u8, vendor: u16, device: u16, class: u8, sub: u8, multi: bool) { self.cfg.insert((bus, dev, func, 0), (device as u32) << 16 | vendor as u32); self.cfg.insert((bus, dev, func, 8), (class as u32) << 24 | (sub as u32) << 16); if func == 0 { self.cfg.insert((bus, dev, 0, 0x0C), if multi { 0x0080_0000 } else { 0 }); } } }
    impl Ports for Sim {
        fn inb(&mut self, port: u16) -> u8 { match port { 0x64 => if self.kbd.is_empty() { 0 } else { 1 }, 0x60 => self.kbd.pop_front().unwrap_or(0), 0x61 => self.p61, _ => 0xFF } }
        fn outb(&mut self, port: u16, v: u8) { self.log.push((port, v)); if port == 0x61 { self.p61 = v; } }
        fn inl(&mut self, port: u16) -> u32 { if port != 0xCFC { return 0xFFFF_FFFF; } let a = self.addr; let key = ((a >> 16) as u8, ((a >> 11) & 31) as u8, ((a >> 8) & 7) as u8, (a & 0xFC) as u8); *self.cfg.get(&key).unwrap_or(&0xFFFF_FFFF) }
        fn outl(&mut self, port: u16, v: u32) { if port == 0xCF8 { self.addr = v; } }
    }
    struct Say(std::vec::Vec<std::string::String>); impl Voice for Say { fn say(&mut self, t: &str) { self.0.push(t.into()); } }

    #[test] fn pci_enumeration_finds_functions_only_when_multifunction() {
        let mut s = Sim::new();
        s.dev(0, 0, 0, 0x1022, 0x1630, 6, 0, false);
        s.dev(0, 8, 0, 0x1022, 0x1632, 6, 0, true); s.dev(0, 8, 1, 0x1002, 0x1638, 3, 0, false); s.dev(0, 8, 2, 0x1022, 0x15DF, 0x10, 0x80, false);
        s.dev(0, 9, 0, 0x1234, 0x5678, 4, 3, false); s.dev(0, 9, 1, 0xDEAD, 0xBEEF, 2, 0, false); // fonction 1 d'un appareil non multifonction : ignorée
        let mut out = [PciDevice { bus: 0, dev: 0, func: 0, id: PciId { vendor: 0, device: 0, class: 0, subclass: 0 } }; 16];
        let n = pci_enumerate(&mut s, 0, &mut out);
        assert_eq!(n, 5); assert_eq!(out[2].id, PciId { vendor: 0x1002, device: 0x1638, class: 3, subclass: 0 }); assert!(out[..n].iter().all(|d| d.id.vendor != 0xDEAD));
    }
    #[test] fn pci_enumeration_is_bounded_by_the_output_buffer() {
        let mut s = Sim::new(); for d in 0..10 { s.dev(0, d, 0, 0x1022, d as u16, 6, 0, false); }
        let mut out = [PciDevice { bus: 0, dev: 0, func: 0, id: PciId { vendor: 0, device: 0, class: 0, subclass: 0 } }; 4]; assert_eq!(pci_enumerate(&mut s, 0, &mut out), 4);
    }
    #[test] fn every_pci_device_class_is_spoken_with_counts() {
        let mk = |c, s| PciDevice { bus: 0, dev: 0, func: 0, id: PciId { vendor: 1, device: 1, class: c, subclass: s } };
        let mut v = Say(vec![]); announce_pci(&mut v, &[mk(0x0C, 3), mk(0x0C, 3), mk(4, 3), mk(3, 0), mk(0x99, 0)]);
        assert_eq!(v.0, ["5 périphériques sur le bus PCI.", "2 contrôleur USB", "1 contrôleur audio haute définition", "1 contrôleur graphique", "1 périphérique inconnu"]);
    }
    #[test] fn keyboard_decodes_azerty_shift_extended_and_ignores_releases() {
        let mut k = Kbd::default(); let mut f = |b| k.feed(b);
        assert_eq!(f(0x10), Some(Key::Char('a'))); assert_eq!(f(0x90), None, "relâchement ignoré"); assert_eq!(f(0x1E), Some(Key::Char('q')));
        assert_eq!(f(0x2A), None); assert_eq!(f(0x10), Some(Key::Char('A'))); assert_eq!(f(0x03), Some(Key::Char('2'))); assert_eq!(f(0xAA), None); assert_eq!(f(0x03), Some(Key::Char('é')));
        assert_eq!(f(0xE0), None); assert_eq!(f(0x48), Some(Key::Up)); assert_eq!(f(0xE0), None); assert_eq!(f(0xC8), None); assert_eq!(f(0xE0), None); assert_eq!(f(0x53), Some(Key::Delete));
        assert_eq!(f(0x1C), Some(Key::Enter)); assert_eq!(f(0x01), Some(Key::Escape)); assert_eq!(f(0x39), Some(Key::Char(' '))); assert_eq!(f(0x7F), None, "code inconnu ignoré");
    }
    #[test] fn keyboard_poll_reads_the_controller_and_skips_mouse_bytes() {
        struct M { q: std::collections::VecDeque<(u8, u8)> } // (état, donnée)
        impl Ports for M { fn inb(&mut self, p: u16) -> u8 { match p { 0x64 => self.q.front().map(|x| x.0).unwrap_or(0), 0x60 => self.q.pop_front().map(|x| x.1).unwrap_or(0), _ => 0 } } fn outb(&mut self, _: u16, _: u8) {} fn inl(&mut self, _: u16) -> u32 { 0 } fn outl(&mut self, _: u16, _: u32) {} }
        let mut m = M { q: [(1, 0x23), (0x21, 0x10), (1, 0x17)].into_iter().collect() }; let mut k = Kbd::default(); let mut out = [Key::Tab; 8];
        let n = kbd_poll(&mut m, &mut k, &mut out); assert_eq!(&out[..n], [Key::Char('h'), Key::Char('i')], "l'octet de souris (bit 5) est sauté");
    }
    #[test] fn speaker_programs_the_pit_and_gate_then_stops() {
        let mut s = Sim::new(); speaker(&mut s, 1000);
        assert_eq!(s.log[0], (0x43, 0xB6)); let div = 1_193_182u32 / 1000; assert_eq!((s.log[1], s.log[2]), ((0x42, div as u8), (0x42, (div >> 8) as u8))); assert_eq!(s.p61 & 3, 3);
        speaker(&mut s, 0); assert_eq!(s.p61 & 3, 0);
    }
}

/// Parcourt les classes présentes (nom français, nombre) sans allocation ; ordre de première apparition. Borné à 64 appareils.
pub fn for_each_class(devs: &[PciDevice], mut f: impl FnMut(&'static str, u32)) {
    let mut done = [false; 64];
    for (i, d) in devs.iter().enumerate().take(64) {
        if done[i] { continue; }
        let name = class_name(d.id.class, d.id.subclass); let mut count = 0u32;
        for (j, e) in devs.iter().enumerate().take(64) { if class_name(e.id.class, e.id.subclass) == name { count += 1; done[j] = true; } }
        f(name, count);
    }
}

#[cfg(test)]
mod class_tests {
    use super::*;
    #[test] fn classes_are_grouped_in_first_seen_order() {
        let mk = |c, s| PciDevice { bus: 0, dev: 0, func: 0, id: PciId { vendor: 1, device: 1, class: c, subclass: s } };
        let mut got = std::vec::Vec::new(); for_each_class(&[mk(0x0C, 3), mk(4, 3), mk(0x0C, 3)], |n, c| got.push((n, c)));
        assert_eq!(got, [("contrôleur USB", 2), ("contrôleur audio haute définition", 1)]);
    }
}
