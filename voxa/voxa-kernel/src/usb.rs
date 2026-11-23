//! USB : début du pilote xHCI (registres de capacités, réinitialisation, ports) et décodeur de rapports HID clavier (protocole « boot »).
//! Testé contre un contrôleur SIMULÉ. NON fait : anneaux de commandes/événements, énumération des périphériques, transferts. NOT_VALIDATED sur matériel réel.
use crate::hw::{key_from_set1, Key};
use crate::Voice;

/// Accès aux registres mappés en mémoire (MMIO) d'un contrôleur, relatifs à sa base.
pub trait Mmio { fn r8(&mut self, off: usize) -> u8; fn r16(&mut self, off: usize) -> u16; fn r32(&mut self, off: usize) -> u32; fn w32(&mut self, off: usize, v: u32); }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XhciInfo { pub version: u16, pub max_slots: u8, pub max_ports: u8, pub op_base: usize }

const USBCMD: usize = 0x00; const USBSTS: usize = 0x04; const PORTSC0: usize = 0x400;
const CMD_RUN: u32 = 1; const CMD_HCRST: u32 = 2; const STS_HCH: u32 = 1; const STS_CNR: u32 = 1 << 11;

/// Lit les registres de capacités. Refuse un contrôleur dont la longueur de capacités ou le nombre de ports est absurde.
pub fn xhci_probe<M: Mmio>(m: &mut M) -> Result<XhciInfo, &'static str> {
    let caplen = m.r8(0) as usize; let version = m.r16(2); let p1 = m.r32(4);
    if caplen < 0x10 || version == 0 || version == 0xFFFF { return Err("contrôleur USB absent ou illisible"); }
    let (max_slots, max_ports) = ((p1 & 0xFF) as u8, (p1 >> 24) as u8);
    if max_ports == 0 || max_slots == 0 { return Err("contrôleur USB sans port"); }
    Ok(XhciInfo { version, max_slots, max_ports, op_base: caplen })
}

/// Arrête puis réinitialise le contrôleur (HCRST) et attend qu'il soit prêt. `spins` borne chaque attente : jamais de boucle infinie.
pub fn xhci_reset<M: Mmio>(m: &mut M, info: &XhciInfo, spins: u32) -> Result<(), &'static str> {
    let o = info.op_base;
    let cmd = m.r32(o + USBCMD); m.w32(o + USBCMD, cmd & !CMD_RUN);
    if !wait(spins, || m.r32(o + USBSTS) & STS_HCH != 0) { return Err("le contrôleur USB ne s'arrête pas"); }
    m.w32(o + USBCMD, CMD_HCRST);
    if !wait(spins, || m.r32(o + USBCMD) & CMD_HCRST == 0) { return Err("la réinitialisation du contrôleur USB ne se termine pas"); }
    if !wait(spins, || m.r32(o + USBSTS) & STS_CNR == 0) { return Err("le contrôleur USB ne devient pas prêt"); }
    Ok(())
}
fn wait(spins: u32, mut f: impl FnMut() -> bool) -> bool { for _ in 0..spins { if f() { return true; } } false }

/// Vitesse d'après le champ PORTSC bits 10-13 (ID de vitesse du protocole par défaut).
pub fn speed_name(id: u32) -> &'static str { match id { 1 => "pleine vitesse", 2 => "basse vitesse", 3 => "haute vitesse", 4 => "super vitesse", 5 => "super vitesse plus", _ => "vitesse inconnue" } }

/// Compte les ports avec un périphérique branché (bit CCS) et les liste (borné).
pub fn xhci_connected<M: Mmio>(m: &mut M, info: &XhciInfo, speeds: &mut [u32]) -> usize {
    let mut n = 0;
    for p in 0..info.max_ports as usize { let s = m.r32(info.op_base + PORTSC0 + 0x10 * p); if s & 1 != 0 { if n < speeds.len() { speeds[n] = (s >> 10) & 0xF; } n += 1; } }
    n
}

fn num(buf: &mut [u8], n: &mut usize, mut v: u32) { let mut t = [0u8; 10]; let mut i = t.len(); if v == 0 { i -= 1; t[i] = b'0'; } while v > 0 { i -= 1; t[i] = b'0' + (v % 10) as u8; v /= 10; } for &b in &t[i..] { buf[*n] = b; *n += 1; } }

/// Sonde, réinitialise et DIT : nombre de ports, périphériques branchés et vitesses ; toute erreur est dite.
pub fn announce_xhci<V: Voice, M: Mmio>(v: &mut V, m: &mut M, spins: u32) -> bool {
    let info = match xhci_probe(m) { Ok(i) => i, Err(e) => { v.say(e); return false } };
    if let Err(e) = xhci_reset(m, &info, spins) { v.say(e); return false; }
    let mut sp = [0u32; 16]; let c = xhci_connected(m, &info, &mut sp);
    let mut b = [0u8; 120]; let mut n = 0; let put = |b: &mut [u8], n: &mut usize, s: &str| { let x = s.as_bytes(); b[*n..*n + x.len()].copy_from_slice(x); *n += x.len(); };
    put(&mut b, &mut n, "Contrôleur USB prêt, "); num(&mut b, &mut n, info.max_ports as u32); put(&mut b, &mut n, " ports, "); num(&mut b, &mut n, c as u32); put(&mut b, &mut n, " périphériques branchés.");
    v.say(core::str::from_utf8(&b[..n]).unwrap_or("Contrôleur USB prêt."));
    for s in sp.iter().take(c.min(16)) { v.say(speed_name(*s)); }
    true
}

// ---------------------------------------------------------------- HID clavier (protocole boot)
/// Correspondance usage HID (page 0x07) -> position physique du jeu de codes 1, pour réutiliser la disposition AZERTY.
fn usage_to_set1(u: u8) -> Option<u8> {
    const L: [u8; 26] = [0x1E, 0x30, 0x2E, 0x20, 0x12, 0x21, 0x22, 0x23, 0x17, 0x24, 0x25, 0x26, 0x32, 0x31, 0x18, 0x19, 0x10, 0x13, 0x1F, 0x14, 0x16, 0x2F, 0x11, 0x2D, 0x15, 0x2C];
    match u { 0x04..=0x1D => Some(L[(u - 4) as usize]), 0x1E..=0x27 => Some(0x02 + (u - 0x1E)), 0x28 => Some(0x1C), 0x29 => Some(0x01), 0x2A => Some(0x0E), 0x2B => Some(0x0F), 0x2C => Some(0x39),
        0x2D => Some(0x0C), 0x2E => Some(0x0D), 0x2F => Some(0x1A), 0x30 => Some(0x1B), 0x31 | 0x32 => Some(0x2B), 0x33 => Some(0x27), 0x34 => Some(0x28), 0x35 => Some(0x29), 0x36 => Some(0x33), 0x37 => Some(0x34), 0x38 => Some(0x35), 0x64 => Some(0x56), _ => None }
}

/// Garde le rapport précédent pour ne produire que les NOUVEAUX appuis (une touche maintenue ne se répète pas ici ; la répétition est gérée plus haut).
#[derive(Default)]
pub struct HidKbd { prev: [u8; 6] }

impl HidKbd {
    /// `report` = 8 octets : modificateurs, réservé, 6 codes d'usage. Un rapport invalide (trop court, ou 0x01 = trop de touches) est ignoré.
    pub fn feed(&mut self, report: &[u8], out: &mut [Key]) -> usize {
        if report.len() < 8 { return 0; }
        let keys = [report[2], report[3], report[4], report[5], report[6], report[7]];
        if keys.iter().any(|&k| k == 0x01) { return 0; } // erreur de débordement : ne pas inventer de touches
        let shift = report[0] & 0x22 != 0; let mut n = 0;
        for &k in &keys {
            if k == 0 || self.prev.contains(&k) { continue; }
            let key = match k { 0x4F => Some(Key::Right), 0x50 => Some(Key::Left), 0x51 => Some(Key::Down), 0x52 => Some(Key::Up), 0x4A => Some(Key::Home), 0x4D => Some(Key::End), 0x4B => Some(Key::PageUp), 0x4E => Some(Key::PageDown), 0x4C => Some(Key::Delete),
                _ => usage_to_set1(k).and_then(|c| key_from_set1(c, shift)) };
            if let Some(key) = key { if n < out.len() { out[n] = key; n += 1; } }
        }
        self.prev = keys; n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// xHCI simulé : le reset ne se termine qu'après `delay` lectures ; `stuck` fige le contrôleur.
    struct Sim { regs: std::collections::HashMap<usize, u32>, caplen: u8, version: u16, p1: u32, delay: u32, reads: u32, stuck: bool, resetting: bool }
    impl Sim { fn new() -> Self { let mut regs = std::collections::HashMap::new(); regs.insert(0x20 + USBSTS, STS_HCH);
        regs.insert(0x20 + PORTSC0, 1 | (4 << 10)); regs.insert(0x20 + PORTSC0 + 0x10, 0); regs.insert(0x20 + PORTSC0 + 0x20, 1 | (2 << 10));
        Sim { regs, caplen: 0x20, version: 0x0110, p1: 0x0400_0040 | 0x20, delay: 3, reads: 0, stuck: false, resetting: false } } }
    impl Mmio for Sim {
        fn r8(&mut self, off: usize) -> u8 { if off == 0 { self.caplen } else { 0 } }
        fn r16(&mut self, off: usize) -> u16 { if off == 2 { self.version } else { 0 } }
        fn r32(&mut self, off: usize) -> u32 {
            if off == 4 { return self.p1; }
            if self.resetting { self.reads += 1; if self.reads > self.delay && !self.stuck { self.resetting = false; self.regs.insert(0x20 + USBCMD, 0); self.regs.insert(0x20 + USBSTS, STS_HCH); } else if off == 0x20 + USBSTS { return STS_HCH | STS_CNR; } }
            *self.regs.get(&off).unwrap_or(&0)
        }
        fn w32(&mut self, off: usize, v: u32) { if off == 0x20 + USBCMD && v & CMD_HCRST != 0 { self.resetting = true; self.reads = 0; } self.regs.insert(off, v); }
    }
    struct Say(std::vec::Vec<std::string::String>); impl Voice for Say { fn say(&mut self, t: &str) { self.0.push(t.into()); } }

    #[test] fn probe_reads_capabilities_and_rejects_absent_controllers() {
        let mut s = Sim::new(); let i = xhci_probe(&mut s).unwrap(); assert_eq!((i.version, i.max_slots, i.max_ports, i.op_base), (0x0110, 0x60 & 0xFF, 4, 0x20));
        let mut s = Sim::new(); s.version = 0xFFFF; assert!(xhci_probe(&mut s).is_err()); let mut s = Sim::new(); s.p1 = 0; assert!(xhci_probe(&mut s).is_err());
    }
    #[test] fn reset_completes_or_fails_with_a_spoken_reason_never_hangs() {
        let mut s = Sim::new(); let i = xhci_probe(&mut s).unwrap(); assert_eq!(xhci_reset(&mut s, &i, 100), Ok(()));
        let mut s = Sim::new(); s.stuck = true; let i = xhci_probe(&mut s).unwrap(); assert_eq!(xhci_reset(&mut s, &i, 50), Err("la réinitialisation du contrôleur USB ne se termine pas"));
    }
    #[test] fn announcement_lists_ports_connected_devices_and_speeds() {
        let mut s = Sim::new(); let mut v = Say(vec![]); assert!(announce_xhci(&mut v, &mut s, 100));
        assert_eq!(v.0, ["Contrôleur USB prêt, 4 ports, 2 périphériques branchés.", "super vitesse", "basse vitesse"]);
        let mut s = Sim::new(); s.stuck = true; let mut v = Say(vec![]); assert!(!announce_xhci(&mut v, &mut s, 20)); assert!(v.0[0].contains("réinitialisation"));
        let mut s = Sim::new(); s.version = 0; let mut v = Say(vec![]); assert!(!announce_xhci(&mut v, &mut s, 20)); assert_eq!(v.0, ["contrôleur USB absent ou illisible"]);
    }
    #[test] fn hid_keyboard_reports_give_new_presses_in_azerty() {
        let mut k = HidKbd::default(); let mut o = [Key::Tab; 6];
        assert_eq!(k.feed(&[0, 0, 0x14, 0, 0, 0, 0, 0], &mut o), 1); assert_eq!(o[0], Key::Char('a'), "usage 0x14 (Q en QWERTY) = A en AZERTY");
        assert_eq!(k.feed(&[0, 0, 0x14, 0, 0, 0, 0, 0], &mut o), 0, "touche maintenue : pas de nouvel appui");
        assert_eq!(k.feed(&[0x02, 0, 0x14, 0x04, 0, 0, 0, 0], &mut o), 1); assert_eq!(o[0], Key::Char('Q'), "Maj + usage 0x04 = Q");
        assert_eq!(k.feed(&[0, 0, 0, 0, 0, 0, 0, 0], &mut o), 0, "relâchement");
        assert_eq!(k.feed(&[0, 0, 0x52, 0x28, 0, 0, 0, 0], &mut o), 2); assert_eq!((o[0], o[1]), (Key::Up, Key::Enter));
        assert_eq!(k.feed(&[0, 0, 1, 1, 1, 1, 1, 1], &mut o), 0, "débordement ignoré"); assert_eq!(k.feed(&[0, 0, 4], &mut o), 0, "rapport trop court ignoré");
    }
}
