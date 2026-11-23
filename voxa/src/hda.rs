// Modele logiciel d'un controleur HD Audio et de son codec (comportement decrit par la specification Intel High Definition Audio 1.0).
// Sert a verifier la LOGIQUE du pilote Voxa : il capture les echantillons envoyes par DMA et dit si le chemin analogique serait audible
// (alimentation, amplificateurs, selection, EAPD...). Ce n'est pas du materiel : une erreur de ma lecture de la specification
// resterait invisible ici. Le test physique tranchera.
#![allow(dead_code)]

pub trait Mem { fn rd8(&self, a: u64) -> u8; fn wr8(&mut self, a: u64, v: u8); }
fn rd32(m: &dyn Mem, a: u64) -> u32 { u32::from_le_bytes([m.rd8(a), m.rd8(a + 1), m.rd8(a + 2), m.rd8(a + 3)]) }
fn wr32(m: &mut dyn Mem, a: u64, v: u32) { for (i, b) in v.to_le_bytes().iter().enumerate() { m.wr8(a + i as u64, *b); } }
fn rd64(m: &dyn Mem, a: u64) -> u64 { rd32(m, a) as u64 | (rd32(m, a + 4) as u64) << 32 }

// Le premier flux de sortie se trouve a 0x80 + (nombre de flux d'entree) * 0x20 : lu dans GCAP, configurable pour les essais (VOXA_HDA_ISS).

#[derive(Clone, Debug, Default)]
pub struct Node { pub power: u8, pub amp_out: [u8; 2], pub amp_in: [[u8; 2]; 4], pub conn_sel: u8, pub stream: u8, pub channel: u8, pub format: u16, pub pin_ctl: u8, pub eapd: u8 }

/// Description statique d'un widget : type (0 convertisseur, 2 melangeur, 3 selecteur, 4 broche, 7 generateur de bip ; 0xFF = absent), capacites, connexions...
#[derive(Clone, Debug)]
pub struct Def { pub typ: u8, pub caps: u32, pub conns: Vec<u8>, pub defcfg: u32, pub pincaps: u32, pub ampout: u32, pub ampin: u32 }
impl Def { fn vide() -> Def { Def { typ: 0xFF, caps: 0, conns: vec![], defcfg: 0, pincaps: 0, ampout: 0, ampin: 0 } } }

#[derive(Clone, Debug, Default)]
pub struct Codec { pub defs: Vec<Def>, pub n: Vec<Node>, pub afg_amp: u32, pub premier: u32, pub gpio: [u8; 3], pub log: Vec<String> }

fn coupe(a: &[u8; 2]) -> bool { a[0] & 0x80 != 0 || a[1] & 0x80 != 0 }
const AMP_SORTIE: u32 = 0x8002_0A4A; // coupable, 0 dB a 0x4A
const AMP_ENTREE: u32 = 0x8000_0A4A;

impl Codec {
    /// Topologie choisie par VOXA_HDA_CODEC : "simple" (defaut) ou "alc256" (plus proche d'un Realtek ALC256 : selecteur, melangeurs, impasses, amplificateurs herites).
    pub fn new() -> Codec { Codec::topologie(&std::env::var("VOXA_HDA_CODEC").unwrap_or_default()) }
    pub fn topologie(nom: &str) -> Codec {
        let mut d: Vec<Def> = vec![];
        let def = |typ: u8, caps: u32, conns: &[u8], defcfg: u32, pincaps: u32, ampout: u32, ampin: u32| Def { typ, caps, conns: conns.to_vec(), defcfg, pincaps, ampout, ampin };
        let (premier, afg_amp);
        if nom == "alc256" {
            for _ in 0..0x25 { d.push(Def::vide()); }
            d[2] = def(0, 0x0415, &[], 0, 0, 0, 0); d[3] = def(0, 0x0415, &[], 0, 0, 0, 0);
            d[0x0C] = def(2, 0x0503, &[2, 3], 0, 0, 0, AMP_ENTREE); d[0x0D] = def(2, 0x0503, &[3, 2], 0, 0, 0, AMP_ENTREE);
            d[0x0F] = def(3, 0x0105, &[0x0C, 0x0D], 0, 0, 0, 0);
            d[0x1D] = def(7, 0x0400, &[], 0, 0, 0, 0);
            d[0x14] = def(4, 0x0505, &[0x1D, 0x0F], 0x9017_0110, 0x0001_0010, AMP_SORTIE, 0);
            d[0x21] = def(4, 0x0505, &[0x0D, 0x0C, 0x0F], 0x0321_1020, 0x0001_0018, AMP_SORTIE, 0);
            d[0x19] = def(4, 0x0501, &[], 0x03A1_1030, 0x0000_0024, 0, 0);
            premier = 2; afg_amp = AMP_SORTIE;
        } else {
            for _ in 0..7 { d.push(Def::vide()); }
            d[2] = def(0, 0x0415, &[], 0, 0, AMP_SORTIE, 0); d[3] = def(0, 0x0415, &[], 0, 0, AMP_SORTIE, 0);
            d[4] = def(2, 0x0503, &[2, 3], 0, 0, 0, AMP_ENTREE);
            d[5] = def(4, 0x0505, &[4], 0x9017_0110, 0x0001_0010, AMP_SORTIE, 0); d[6] = def(4, 0x0505, &[4], 0x0321_1020, 0x0001_0010, AMP_SORTIE, 0);
            premier = 2; afg_amp = 0;
        }
        let mut n = vec![Node::default(); d.len()];
        for x in n.iter_mut() { x.power = 3; x.amp_out = [0x80, 0x80]; x.amp_in = [[0x80, 0x80]; 4]; } // D3, amplificateurs coupes
        Codec { defs: d, n, afg_amp, premier, gpio: [0; 3], log: vec![] }
    }
    fn param(&self, nid: usize, id: u32) -> u32 {
        if nid == 0 { return match id { 0x00 => 0x10EC_0256, 0x02 => 0x0010_0002, 0x04 => 0x0001_0001, _ => 0 }; }
        if nid == 1 { return match id { 0x04 => (self.premier << 16) | (self.defs.len() as u32 - self.premier), 0x05 => 1, 0x0F => 0x0F, 0x0A => 0x0002_00FF, 0x0B => 1, 0x12 => self.afg_amp, 0x0D => self.afg_amp, _ => 0 }; }
        let d = &self.defs[nid];
        if d.typ == 0xFF { return 0; }
        match id { 0x09 => d.caps | ((d.typ as u32) << 20), 0x0A => 0x0002_00FF, 0x0B => 1, 0x0C => d.pincaps, 0x0E => d.conns.len() as u32, 0x0F => 0x0F, 0x12 => d.ampout, 0x0D => d.ampin, _ => 0 }
    }
    /// Execute un verbe de 32 bits ; None = pas de reponse (codec inconnu).
    pub fn verb(&mut self, v: u32) -> Option<u32> {
        let (cad, nid) = ((v >> 28) as usize, ((v >> 20) & 0xFF) as usize);
        if cad != 0 || nid >= self.n.len() { return None; }
        let (short, pay8, pay16, id4) = ((v >> 8) & 0xFFF, (v & 0xFF) as u8, (v & 0xFFFF) as u16, (v >> 16) & 0xF);
        let resp0 = Some(0u32);
        if id4 == 0x2 { self.n[nid].format = pay16; self.log.push(format!("nid {nid:#X} format {pay16:#06X}")); return resp0; }
        if id4 == 0xA { return Some(self.n[nid].format as u32); }
        if id4 == 0x3 {
            let (out, inp, left, right, idx, val) = (pay16 & 0x8000 != 0, pay16 & 0x4000 != 0, pay16 & 0x2000 != 0, pay16 & 0x1000 != 0, ((pay16 >> 8) & 0xF) as usize, (pay16 & 0xFF) as u8);
            let node = &mut self.n[nid];
            if out { if left { node.amp_out[0] = val; } if right { node.amp_out[1] = val; } }
            if inp && idx < 4 { if left { node.amp_in[idx][0] = val; } if right { node.amp_in[idx][1] = val; } }
            self.log.push(format!("nid {nid:#X} amplificateur {} indice {idx} {}{} valeur {val:#04X}", if out { "sortie" } else { "entrée" }, if left { "G" } else { "" }, if right { "D" } else { "" }));
            return resp0;
        }
        if id4 == 0xB {
            let (out, left, idx) = (pay16 & 0x8000 != 0, pay16 & 0x2000 != 0, (pay16 & 0xF) as usize); let ch = if left { 0 } else { 1 }; let node = &self.n[nid];
            return Some((if out { node.amp_out[ch] } else if idx < 4 { node.amp_in[idx][ch] } else { 0 }) as u32);
        }
        if short == 0xF00 { return Some(self.param(nid, pay8 as u32)); }
        let defcfg = self.defs.get(nid).map(|d| d.defcfg).unwrap_or(0);
        let conns: Vec<u8> = self.defs.get(nid).map(|d| d.conns.clone()).unwrap_or_default();
        let node = &mut self.n[nid];
        Some(match short {
            0xF02 => { let mut r = 0u32; for k in 0..4 { r |= (*conns.get(pay8 as usize + k).unwrap_or(&0) as u32) << (8 * k); } r }
            0xF01 => node.conn_sel as u32, 0x701 => { node.conn_sel = pay8; self.log.push(format!("nid {nid:#X} sélection de connexion {pay8}")); 0 }
            0xF05 => ((node.power as u32) << 4) | node.power as u32, 0x705 => { node.power = pay8 & 0xF; self.log.push(format!("nid {nid:#X} alimentation D{}", pay8 & 0xF)); 0 }
            0xF07 => node.pin_ctl as u32, 0x707 => { node.pin_ctl = pay8; self.log.push(format!("nid {nid:#X} commande de broche {pay8:#04X}")); 0 }
            0xF0C => node.eapd as u32, 0x70C => { node.eapd = pay8; self.log.push(format!("nid {nid:#X} EAPD {pay8:#04X}")); 0 }
            0xF06 => ((node.stream as u32) << 4) | node.channel as u32, 0x706 => { node.stream = pay8 >> 4; node.channel = pay8 & 0xF; self.log.push(format!("nid {nid:#X} flux {} canal {}", pay8 >> 4, pay8 & 0xF)); 0 }
            0xF09 => 0,                                  // aucune prise branchee
            0xF1C => defcfg,
            0xF15 => self.gpio[2] as u32, 0x715 => { self.gpio[2] = pay8; 0 } 0xF16 => self.gpio[1] as u32, 0x716 => { self.gpio[1] = pay8; 0 } 0xF17 => self.gpio[0] as u32, 0x717 => { self.gpio[0] = pay8; 0 }
            _ => 0,
        })
    }
    fn trace(&self, nid: usize, tag: u8, fmt: u16, prof: u8, why: &mut Vec<String>) -> bool {
        if prof > 8 || nid >= self.defs.len() || self.defs[nid].typ == 0xFF { why.push(format!("chemin interrompu au nœud {nid:#X}")); return false; }
        let (d, nd) = (&self.defs[nid], &self.n[nid]);
        match d.typ {
            0 => {
                let mut ok = true;
                if tag == 0 || nd.stream != tag { why.push(format!("convertisseur {nid:#X} n'écoute pas le flux {tag}")); ok = false; }
                if nd.format != fmt { why.push(format!("convertisseur {nid:#X} : format {:#06X} différent du flux {fmt:#06X}", nd.format)); ok = false; }
                if nd.power != 0 { why.push(format!("convertisseur {nid:#X} en D{}", nd.power)); ok = false; }
                if d.caps & 4 != 0 && coupe(&nd.amp_out) { why.push(format!("amplificateur du convertisseur {nid:#X} coupé")); ok = false; }
                ok
            }
            2 => {
                let mut ok_p = true;
                if nd.power != 0 { why.push(format!("mélangeur {nid:#X} en D{}", nd.power)); ok_p = false; }
                let mut meilleur: Vec<String> = vec![]; let mut any = false;
                for (i, c) in d.conns.iter().enumerate() {
                    let mut w = vec![]; let mut ok = self.trace(*c as usize, tag, fmt, prof + 1, &mut w);
                    if d.caps & 2 != 0 && i < 4 && coupe(&nd.amp_in[i]) { w.push(format!("entrée {i} du mélangeur {nid:#X} coupée")); ok = false; }
                    if ok { any = true; break; }
                    if meilleur.is_empty() || w.len() < meilleur.len() { meilleur = w; }
                }
                if !any { why.extend(meilleur); }
                ok_p && any
            }
            3 => {
                let mut ok_p = true;
                if nd.power != 0 { why.push(format!("sélecteur {nid:#X} en D{}", nd.power)); ok_p = false; }
                let r = match d.conns.get(nd.conn_sel as usize) { Some(c) => self.trace(*c as usize, tag, fmt, prof + 1, why), None => { why.push(format!("sélecteur {nid:#X} sans entrée valide")); false } };
                ok_p && r
            }
            t => { why.push(format!("nœud {nid:#X} (type {t}) ne mène à aucun convertisseur")); false }
        }
    }
    /// Le son d'un flux (etiquette `tag`, format `fmt`) sortirait-il par une broche de sortie ? Renvoie les conditions non remplies de la meilleure broche.
    pub fn audible(&self, tag: u8, fmt: u16) -> (bool, Vec<String>) {
        let mut commun = vec![];
        if self.n[1].power != 0 { commun.push(format!("groupe audio en D{} au lieu de D0", self.n[1].power)); }
        let mut meilleur: Option<Vec<String>> = None;
        for pin in 2..self.defs.len() {
            let (d, nd) = (&self.defs[pin], &self.n[pin]);
            if d.typ != 4 { continue; }
            let (conn, dev) = ((d.defcfg >> 30) & 3, (d.defcfg >> 20) & 0xF);
            if conn == 1 || dev > 2 { continue; }
            let mut why = commun.clone();
            if nd.power != 0 { why.push(format!("broche {pin:#X} en D{}", nd.power)); }
            if nd.pin_ctl & 0x40 == 0 { why.push(format!("broche {pin:#X} : sortie non activée")); }
            if d.pincaps & 0x10000 != 0 && nd.eapd & 2 == 0 { why.push("amplificateur externe (EAPD) non activé".into()); }
            if d.caps & 4 != 0 && coupe(&nd.amp_out) { why.push(format!("amplificateur de la broche {pin:#X} coupé")); }
            match d.conns.get(nd.conn_sel as usize) { Some(c) => { self.trace(*c as usize, tag, fmt, 0, &mut why); } None => why.push(format!("broche {pin:#X} sans connexion sélectionnée")) }
            if why.is_empty() { return (true, vec![]); }
            if meilleur.as_ref().map(|m| why.len() < m.len()).unwrap_or(true) { meilleur = Some(why); }
        }
        let r = meilleur.unwrap_or_else(|| vec!["aucune broche de sortie utilisable".to_string()]);
        (false, r)
    }
}

#[derive(Clone, Debug, Default)]
pub struct PlayReport { pub audible: bool, pub reasons: Vec<String>, pub bytes: usize, pub fmt: u16, pub tag: u8 }

#[derive(Clone, Debug, Default)]
pub struct Hda { pub base: u64, pub codec: Option<Codec>, r: Vec<u8>, corb_rp: u16, rirb_wp: u16, pub pcm: Vec<u8>, pub plays: Vec<PlayReport>, pub errors: Vec<String>, run_prev: bool, sd0: usize, corb_n: u16, rirb_n: u16 }

impl Hda {
    pub fn new(base: u64, codec: Option<Codec>) -> Hda { let mut h = Hda { base, codec, r: vec![0; 0x400], ..Default::default() }; h.hw_reset(); h }
    fn hw_reset(&mut self) {
        for b in self.r.iter_mut() { *b = 0; }
        let iss: usize = std::env::var("VOXA_HDA_ISS").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
        let petit = std::env::var("VOXA_HDA_CORB").map(|v| v == "16").unwrap_or(false);
        self.sd0 = 0x80 + iss * 0x20;
        self.set16(0x00, (4 << 12) | ((iss as u16) << 8) | 1); self.r[0x02] = 0; self.r[0x03] = 1; // GCAP : 64 bits, ISS flux d'entree, 4 de sortie ; VMIN 0, VMAJ 1
        self.set16(0x04, 0x3C); self.set16(0x06, 0x1D);
        self.corb_rp = 0; self.rirb_wp = 0; self.run_prev = false;
        let (cap, code, n) = if petit { (0x20u8, 1u8, 16u16) } else { (0x70u8, 2u8, 256u16) }; // capacites (bits 7:4) et taille par defaut
        self.r[0x4E] = cap | code; self.r[0x5E] = cap | code; self.corb_n = n; self.rirb_n = n;
    }
    fn get16(&self, o: usize) -> u16 { u16::from_le_bytes([self.r[o], self.r[o + 1]]) }
    fn set16(&mut self, o: usize, v: u16) { self.r[o..o + 2].copy_from_slice(&v.to_le_bytes()); }
    fn get32(&self, o: usize) -> u32 { u32::from_le_bytes([self.r[o], self.r[o + 1], self.r[o + 2], self.r[o + 3]]) }
    pub fn read(&self, off: usize, size: usize) -> u32 {
        if off + size > self.r.len() { return 0xFFFF_FFFF; }
        let mut v = 0u32; for k in 0..size { v |= (self.r[off + k] as u32) << (8 * k); } v
    }
    pub fn write(&mut self, off: usize, size: usize, val: u32, mem: &mut dyn Mem, bus_master: bool) {
        if off + size > self.r.len() { return; }
        let bytes = val.to_le_bytes();
        match off {
            0x08 => { // GCTL
                let was = self.r[0x08] & 1;
                let now = bytes[0] & 1;
                if now == 0 { self.hw_reset(); } else if was == 0 { self.r[0x08] = 1; if self.codec.is_some() { self.set16(0x0E, 1); } }
                return;
            }
            0x0E => { let c = self.get16(0x0E) & !(val as u16); self.set16(0x0E, c); return; } // STATESTS : ecriture de 1 pour effacer
            0x4D | 0x5D => { self.r[off] &= !bytes[0]; return; } // CORBSTS / RIRBSTS
            0x4A => { // CORBRP : remise a zero par le bit 15
                if val & 0x8000 != 0 { self.set16(0x4A, 0x8000); } else if self.get16(0x4A) & 0x8000 != 0 { self.set16(0x4A, 0); self.corb_rp = 0; }
                return;
            }
            0x4E | 0x5E => {
                let code = bytes[0] & 3; let cap = self.r[off] & 0xF0;
                let (bit, n) = match code { 0 => (0x10u8, 2u16), 1 => (0x20, 16), _ => (0x40, 256) };
                if cap & bit == 0 { self.errors.push(format!("taille de file {n} non prise en charge (capacités {cap:#04X})")); return; }
                self.r[off] = cap | code; if off == 0x4E { self.corb_n = n; } else { self.rirb_n = n; }
                return;
            }
            0x58 => { if val & 0x8000 != 0 { self.rirb_wp = 0; self.set16(0x58, 0); } return; }
            0x48 => {
                let wp = (val & 0xFF) as u16;
                if wp >= self.corb_n { self.errors.push(format!("CORBWP {wp} hors de la file de {} entrées", self.corb_n)); return; }
                self.set16(0x48, wp); self.process_corb(mem, bus_master); return;
            }
            _ => {}
        }
        if off == self.sd0 + 3 && size == 1 { self.r[off] &= !bytes[0]; return; } // STS du flux : ecriture de 1 pour effacer
        // un demarrage (RUN) sur un descripteur de flux autre que le premier flux de sortie est une erreur du pilote
        if off >= 0x80 && off < 0x80 + 0x20 * 8 && (off - 0x80) % 0x20 == 0 && (off < self.sd0 || off >= self.sd0 + 0x20) && bytes[0] & 2 != 0 {
            self.errors.push(format!("RUN sur le descripteur de flux {off:#X} alors que le premier flux de sortie est à {:#X}", self.sd0));
        }
        for k in 0..size { self.r[off + k] = bytes[k]; }
        // controle du premier flux de sortie : 3 octets a partir de self.sd0 (bit 0 SRST, bit 1 RUN)
        if off >= self.sd0 && off < self.sd0 + 3 {
            let ctl = self.get32(self.sd0) & 0x00FF_FFFF;
            let (srst, run) = (ctl & 1 != 0, ctl & 2 != 0);
            if run && !self.run_prev && !srst { self.run_stream(mem, bus_master, (ctl >> 20) as u8 & 0xF); }
            if !run { self.run_prev = false; } else { self.run_prev = true; }
        }
    }
    fn process_corb(&mut self, mem: &mut dyn Mem, bus_master: bool) {
        if self.corb_rp == self.get16(0x48) & 0xFF { return; } // rien a traiter
        if self.r[0x4C] & 2 == 0 || self.r[0x5C] & 2 == 0 { self.errors.push("verbes ignorés : DMA CORB/RIRB non démarré".into()); return; }
        if !bus_master { self.errors.push("verbes ignorés : maître du bus PCI non activé".into()); return; }
        let corb = self.get32(0x40) as u64 | (self.get32(0x44) as u64) << 32; let rirb = self.get32(0x50) as u64 | (self.get32(0x54) as u64) << 32;
        let wp = self.get16(0x48) & 0xFF;
        while self.corb_rp != wp {
            self.corb_rp = (self.corb_rp + 1) % self.corb_n;
            let v = rd32(mem, corb + 4 * self.corb_rp as u64);
            let resp = match self.codec.as_mut() { Some(c) => c.verb(v), None => None };
            if let Some(r) = resp {
                self.rirb_wp = (self.rirb_wp + 1) % self.rirb_n;
                let a = rirb + 8 * self.rirb_wp as u64; wr32(mem, a, r); wr32(mem, a + 4, 0);
                self.r[0x5D] |= 1;
            }
        }
        let rp = self.corb_rp; self.set16(0x4A, rp); let rw = self.rirb_wp; self.set16(0x58, rw);
    }
    fn run_stream(&mut self, mem: &mut dyn Mem, bus_master: bool, tag: u8) {
        if !bus_master { self.errors.push("flux ignoré : maître du bus PCI non activé".into()); return; }
        let cbl = self.get32(self.sd0 + 8) as usize; let lvi = (self.get16(self.sd0 + 0xC) & 0xFF) as u64; let fmt = self.get16(self.sd0 + 0x12);
        let bdl = self.get32(self.sd0 + 0x18) as u64 | (self.get32(self.sd0 + 0x1C) as u64) << 32;
        if cbl == 0 { self.errors.push("flux ignoré : longueur de tampon cyclique nulle".into()); return; }
        let mut total = 0usize; let mut ioc = false; let start = self.pcm.len();
        for i in 0..=lvi {
            let e = bdl + 16 * i; let (addr, len, fl) = (rd64(mem, e), rd32(mem, e + 8) as u64, rd32(mem, e + 12));
            if fl & 1 != 0 { ioc = true; }
            for k in 0..len { self.pcm.push(mem.rd8(addr + k)); }
            total += len as usize;
        }
        if total != cbl { self.errors.push(format!("liste de tampons : {total} octets au lieu de {cbl}")); }
        self.r[self.sd0 + 4..self.sd0 + 8].copy_from_slice(&(cbl as u32).to_le_bytes()); // LPIB = fin du tampon
        if ioc { self.r[self.sd0 + 3] |= 4; }
        let (ok, why) = match &self.codec { Some(c) => c.audible(tag, fmt), None => (false, vec!["aucun codec".to_string()]) };
        self.plays.push(PlayReport { audible: ok, reasons: why, bytes: self.pcm.len() - start, fmt, tag });
    }
}

// ---------------------------------------------------------------- auto-test du modele : chaque etape retiree doit rendre le chemin NON audible
fn v12(nid: u32, verbe: u32, pl: u32) -> u32 { nid << 20 | verbe << 8 | pl }
fn v16(nid: u32, v4: u32, pl: u32) -> u32 { nid << 20 | v4 << 16 | pl }
pub fn selftest() -> Vec<(String, bool)> {
    let etapes: Vec<(&str, u32)> = vec![
        ("alimentation du groupe audio", v12(1, 0x705, 0)),
        ("alimentation du convertisseur", v12(2, 0x705, 0)),
        ("amplificateur du convertisseur", v16(2, 3, 0xB000 + 0x4A)),
        ("format du convertisseur", v16(2, 2, 0x4111)),
        ("flux du convertisseur", v12(2, 0x706, 0x10)),
        ("alimentation du mélangeur", v12(4, 0x705, 0)),
        ("entrée du mélangeur", v16(4, 3, 0x7000 + 0x4A)),
        ("alimentation de la broche", v12(5, 0x705, 0)),
        ("sortie de la broche", v12(5, 0x707, 0x40)),
        ("EAPD de la broche", v12(5, 0x70C, 2)),
        ("amplificateur de la broche", v16(5, 3, 0xB000 + 0x4A)),
    ];
    let jouer = |omis: Option<usize>| { let mut c = Codec::topologie("simple"); for (i, (_, v)) in etapes.iter().enumerate() { if Some(i) != omis { c.verb(*v); } } c.audible(1, 0x4111) };
    let mut r = vec![];
    let (ok, why) = jouer(None);
    r.push((format!("séquence complète : audible (raisons : {why:?})"), ok));
    for (i, (nom, _)) in etapes.iter().enumerate() {
        let (ok, why) = jouer(Some(i));
        r.push((format!("sans « {nom} » : non audible, motif signalé {why:?}"), !ok && !why.is_empty()));
    }
    let mut c = Codec::topologie("simple"); for (_, v) in &etapes { c.verb(*v); }
    r.push(("mauvaise étiquette de flux : non audible".to_string(), !c.audible(2, 0x4111).0));
    r.push(("mauvais format de flux : non audible".to_string(), !c.audible(1, 0x0111).0));
    r.push(("codec inexistant (adresse 3) : aucune réponse".to_string(), Codec::topologie("simple").verb(0x3000_0000 | v12(0, 0xF00, 0)).is_none()));
    r
}
