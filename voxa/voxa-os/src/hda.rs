//! Pilote HD Audio (Intel High Definition Audio) en Rust : port de `bib/hda.vx`.
//! Découverte PCI (classe 0x0403), réinitialisation du contrôleur, files CORB/RIRB, recherche récursive d'un chemin broche -> convertisseur dans le codec,
//! lecture par DMA d'un tampon 16 bits stéréo à 22 050 Hz (format 0x4111).
//! STATUT : validé contre un MODÈLE logiciel du contrôleur et du codec (émulateur) ; jamais exécuté sur un contrôleur réel. Détecté != initialisé != audible.
//! Codes d'erreur (comme hda.vx) : 1 contrôleur sans adresse, 2 aucun codec, 3 files de commandes, 4 aucun chemin de sortie, 5 aucun contrôleur audio.

use core::ptr::{read_volatile, write_volatile};

pub const ERR_NO_BAR: u8 = 1; pub const ERR_NO_CODEC: u8 = 2; pub const ERR_QUEUES: u8 = 3; pub const ERR_NO_PATH: u8 = 4; pub const ERR_NO_CONTROLLER: u8 = 5;

#[repr(C, align(128))]
pub struct Dma { corb: [u32; 256], rirb: [u64; 256], bdl: [u32; 4] }
static mut DMA: Dma = Dma { corb: [0; 256], rirb: [0; 256], bdl: [0; 4] };

#[inline(always)] unsafe fn outl(port: u16, v: u32) { core::arch::asm!("out dx, eax", in("dx") port, in("eax") v, options(nomem, nostack, preserves_flags)); }
#[inline(always)] unsafe fn inl(port: u16) -> u32 { let v: u32; core::arch::asm!("in eax, dx", out("eax") v, in("dx") port, options(nomem, nostack, preserves_flags)); v }
fn pci_adr(b: u32, d: u32, f: u32) -> u32 { 0x8000_0000 + b * 65536 + d * 2048 + f * 256 }
fn pci_lire(adr: u32, reg: u32) -> u32 { unsafe { outl(0xCF8, adr + reg); inl(0xCFC) } }
fn pci_ecrire(adr: u32, reg: u32, v: u32) { unsafe { outl(0xCF8, adr + reg); outl(0xCFC, v); } }

pub struct Hda {
    base: u64, pub pci: u32, cad: u32, afg: u32, pub dac: u32, pub pin: u32, pub vendor: u32,
    wp: usize, rp: usize, corb_n: usize, rirb_n: usize, sd: u64, stall: fn(usize), pub error: u8,
}

impl Hda {
    fn r8(&self, o: u64) -> u32 { unsafe { read_volatile((self.base + o) as *const u8) as u32 } }
    fn r16(&self, o: u64) -> u32 { unsafe { read_volatile((self.base + o) as *const u16) as u32 } }
    fn r32(&self, o: u64) -> u32 { unsafe { read_volatile((self.base + o) as *const u32) } }
    fn w8(&self, o: u64, v: u32) { unsafe { write_volatile((self.base + o) as *mut u8, v as u8) } }
    fn w16(&self, o: u64, v: u32) { unsafe { write_volatile((self.base + o) as *mut u16, v as u16) } }
    fn w32(&self, o: u64, v: u32) { unsafe { write_volatile((self.base + o) as *mut u32, v) } }
    fn stall(&self, us: usize) { (self.stall)(us); }

    fn attendre_bit(&self, o: u64, taille: u32, masque: u32, voulu: u32) -> bool {
        for _ in 0..1000 {
            let v = match taille { 1 => self.r8(o), 2 => self.r16(o), _ => self.r32(o) };
            if (v / masque) % 2 == voulu { return true; }
            self.stall(100);
        }
        false
    }
    fn reset(&self) -> u32 {
        self.w32(8, 0);
        if !self.attendre_bit(8, 4, 1, 0) { return 0; }
        self.stall(200);
        self.w32(8, 1);
        if !self.attendre_bit(8, 4, 1, 1) { return 0; }
        self.stall(2000);
        self.r16(14) % 256
    }
    fn corb_init(&mut self) -> bool {
        self.w8(76, 0); self.w8(92, 0);
        self.sd = 128 + ((self.r16(0) / 256 % 16) as u64) * 32; // premier flux de sortie = 0x80 + ISS * 0x20
        let cs = self.r8(78); let mut code = 0; self.corb_n = 2;
        if (cs / 32) % 2 == 1 { code = 1; self.corb_n = 16; }
        if (cs / 64) % 2 == 1 { code = 2; self.corb_n = 256; }
        self.w8(78, code);
        let rs = self.r8(94); let mut rcode = 0; self.rirb_n = 2;
        if (rs / 32) % 2 == 1 { rcode = 1; self.rirb_n = 16; }
        if (rs / 64) % 2 == 1 { rcode = 2; self.rirb_n = 256; }
        self.w8(94, rcode);
        let (corb, rirb) = unsafe { (core::ptr::addr_of!(DMA.corb) as u64, core::ptr::addr_of!(DMA.rirb) as u64) };
        self.w32(64, corb as u32); self.w32(68, (corb >> 32) as u32); self.w32(80, rirb as u32); self.w32(84, (rirb >> 32) as u32);
        self.w16(74, 32768);
        if !self.attendre_bit(74, 2, 32768, 1) { return false; }
        self.w16(74, 0);
        if !self.attendre_bit(74, 2, 32768, 0) { return false; }
        self.w16(72, 0); self.w16(88, 32768); self.w8(76, 2); self.w8(92, 2);
        self.wp = 0; self.rp = 0;
        true
    }
    /// Envoie un verbe ; None si le codec ne répond pas.
    fn cmd(&mut self, v: u32) -> Option<u32> {
        let wp = (self.wp + 1) % self.corb_n;
        unsafe { write_volatile(core::ptr::addr_of_mut!(DMA.corb[wp]), v); }
        self.wp = wp; self.w16(72, wp as u32);
        for _ in 0..2000 {
            if self.r16(88) % 256 != self.rp as u32 {
                self.rp = (self.rp + 1) % self.rirb_n;
                return Some(unsafe { read_volatile(core::ptr::addr_of!(DMA.rirb[self.rp])) } as u32);
            }
            self.stall(10);
        }
        None
    }
    fn verbe12(&self, nid: u32, verbe: u32, pl: u32) -> u32 { (self.cad << 28) + nid * 1_048_576 + verbe * 256 + pl }
    fn verbe16(&self, nid: u32, v4: u32, pl: u32) -> u32 { (self.cad << 28) + nid * 1_048_576 + v4 * 65_536 + pl }
    fn param(&mut self, nid: u32, id: u32) -> u32 { let v = self.verbe12(nid, 0xF00, id); self.cmd(v).unwrap_or(0xFFFF_FFFF) }
    fn alimenter(&mut self, nid: u32) { let v = self.verbe12(nid, 0x705, 0); self.cmd(v); self.stall(2000); }
    fn gain_nominal(&mut self, nid: u32, caps: u32) -> u32 { let mut v = self.param(nid, caps); if v == 0 { let a = self.afg; v = self.param(a, caps); } v % 128 }
    fn debloquer_sortie(&mut self, nid: u32) { let g = self.gain_nominal(nid, 18); let v = self.verbe16(nid, 3, 0xB000 + g); self.cmd(v); }
    fn debloquer_entree(&mut self, nid: u32, i: u32) { let g = self.gain_nominal(nid, 13); let v = self.verbe16(nid, 3, 0x7000 + i * 256 + g); self.cmd(v); }
    fn configurer(&mut self, nid: u32, typ: u32, i: u32) {
        self.alimenter(nid);
        if typ == 4 {
            let v = self.verbe12(nid, 0x707, 0x40); self.cmd(v);
            let v = self.verbe12(nid, 0x70C, 2); self.cmd(v);
            if self.param(nid, 18) / 0x8000_0000 == 1 { self.debloquer_sortie(nid); }
        }
        if typ == 3 || typ == 4 { let v = self.verbe12(nid, 0x701, i); self.cmd(v); }
        if typ == 2 { self.debloquer_entree(nid, i); }
    }
    fn activer_dac(&mut self, nid: u32) {
        self.alimenter(nid); self.debloquer_sortie(nid);
        let v = self.verbe16(nid, 2, 0x4111); self.cmd(v);
        let v = self.verbe12(nid, 0x706, 16); self.cmd(v);
    }
    /// Cherche récursivement un convertisseur relié à `nid` ; configure le chemin au retour. Renvoie le convertisseur ou 0.
    fn chemin(&mut self, nid: u32, prof: u32) -> u32 {
        let typ = self.param(nid, 9) / 1_048_576 % 16;
        if typ == 0 { self.activer_dac(nid); return nid; }
        if prof > 5 || !(typ == 4 || typ == 2 || typ == 3) { return 0; }
        let nconn = self.param(nid, 14) % 128;
        for i in 0..nconn {
            let v = self.verbe12(nid, 0xF02, i - i % 4);
            let groupe = self.cmd(v).unwrap_or(0);
            let c = (groupe >> (8 * (i % 4))) % 256;
            let dac = self.chemin(c, prof + 1);
            if dac != 0 { self.configurer(nid, typ, i); return dac; }
        }
        0
    }
    fn decouvrir(&mut self) -> bool {
        let vend = self.param(0, 0);
        if vend == 0xFFFF_FFFF { return false; }
        self.vendor = vend;
        let sub = self.param(0, 4);
        let (mut g, fin) = ((sub / 65536) % 256, (sub / 65536) % 256 + sub % 256);
        let mut afg = 0;
        while g < fin { if self.param(g, 5) % 256 == 1 { afg = g; } g += 1; }
        if afg == 0 { return false; }
        self.afg = afg; self.alimenter(afg);
        let w = self.param(afg, 4);
        let (mut n, fin2) = ((w / 65536) % 256, (w / 65536) % 256 + w % 256);
        let (mut hp, mut pin) = (0, 0);
        while n < fin2 {
            if self.param(n, 9) / 1_048_576 % 16 == 4 {
                let v = self.verbe12(n, 0xF1C, 0); let cfg = self.cmd(v).unwrap_or(0);
                if cfg / 0x4000_0000 % 4 != 1 { // connectivité : broche reliée
                    let dev = cfg / 1_048_576 % 16;
                    if dev == 1 { pin = n; } // haut-parleur interne
                    if dev == 2 { hp = n; }  // casque
                }
            }
            n += 1;
        }
        if pin == 0 { pin = hp; }
        if pin == 0 { return false; }
        self.pin = pin;
        let dac = self.chemin(pin, 0);
        if dac == 0 { return false; }
        self.dac = dac; true
    }
    fn pm_d0(adr: u32) {
        let st = pci_lire(adr, 4) / 65536;
        if (st / 16) % 2 == 0 { return; }
        let mut cp = pci_lire(adr, 52) % 256; cp -= cp % 4;
        let mut n = 0;
        while cp >= 64 {
            if n > 8 { return; }
            let ent = pci_lire(adr, cp);
            if ent % 256 == 1 { let pm = pci_lire(adr, cp + 4); if pm % 4 != 0 { pci_ecrire(adr, cp + 4, pm - pm % 4); } return; }
            cp = (ent / 256) % 256; cp -= cp % 4; n += 1;
        }
    }
    fn essayer(adr: u32, stall: fn(usize)) -> Result<Hda, u8> {
        let bar = pci_lire(adr, 16);
        if bar == 0xFFFF_FFFF || bar % 2 == 1 { return Err(ERR_NO_BAR); }
        let mut base = (bar - bar % 16) as u64;
        if (bar / 2) % 4 == 2 { base += (pci_lire(adr, 20) as u64) << 32; }
        if base == 0 { return Err(ERR_NO_BAR); }
        Self::pm_d0(adr);
        let mut cmd = pci_lire(adr, 4) % 65536;
        if (cmd / 2) % 2 == 0 { cmd += 2; }
        if (cmd / 4) % 2 == 0 { cmd += 4; }
        pci_ecrire(adr, 4, cmd);
        let mut h = Hda { base, pci: adr, cad: 0, afg: 0, dac: 0, pin: 0, vendor: 0, wp: 0, rp: 0, corb_n: 2, rirb_n: 2, sd: 0, stall, error: 0 };
        let codecs = h.reset();
        if codecs == 0 { return Err(ERR_NO_CODEC); }
        for c in 0..8 {
            if (codecs >> c) & 1 == 1 {
                h.cad = c;
                if !h.corb_init() { return Err(ERR_QUEUES); }
                if h.decouvrir() { return Ok(h); }
            }
        }
        Err(ERR_NO_PATH)
    }
    /// Cherche un contrôleur audio utilisable : d'abord hors processeur graphique (haut-parleurs du portable), puis les sorties HDMI.
    pub fn init(stall: fn(usize)) -> Result<Hda, u8> {
        let mut dernier = ERR_NO_CONTROLLER;
        for passe in 0..2u32 {
            for b in 0..8 { for d in 0..32 { for f in 0..8 {
                let adr = pci_adr(b, d, f); let id = pci_lire(adr, 0);
                if id == 0xFFFF_FFFF || pci_lire(adr, 8) / 256 != 0x0004_0300 { continue; }
                let gpu = (id % 65536 == 0x1002) as u32;
                if gpu != passe { continue; }
                match Self::essayer(adr, stall) { Ok(h) => return Ok(h), Err(e) => dernier = e }
            } } }
        }
        Err(dernier)
    }

    /// Joue `octets` octets (multiple de 128) de PCM 16 bits stéréo 22 050 Hz situés à l'adresse physique `buf` (alignée 128), puis arrête le flux.
    /// Renvoie true si la position du DMA a atteint la fin (le modèle ou le matériel a consommé le tampon) ; cela ne prouve PAS que le son a été entendu.
    pub fn jouer(&mut self, buf: u64, octets: u32) -> bool {
        let sd = self.sd;
        self.w8(sd, 0); self.w8(sd, 1);
        if !self.attendre_bit(sd, 1, 1, 1) { return false; }
        self.w8(sd, 0);
        if !self.attendre_bit(sd, 1, 1, 0) { return false; }
        let bdl = unsafe { core::ptr::addr_of_mut!(DMA.bdl) } as *mut u32;
        unsafe { write_volatile(bdl, buf as u32); write_volatile(bdl.add(1), (buf >> 32) as u32); write_volatile(bdl.add(2), octets); write_volatile(bdl.add(3), 1); }
        let bdl_a = bdl as u64;
        self.w32(sd + 8, octets); self.w16(sd + 12, 0); self.w16(sd + 18, 0x4111);
        self.w32(sd + 24, bdl_a as u32); self.w32(sd + 28, (bdl_a >> 32) as u32);
        self.w8(sd + 2, 16); self.w8(sd, 2);
        let us = octets as usize / 4 * 1_000_000 / 22_050;
        self.stall(us);
        let mut fini = false;
        for _ in 0..200 { if self.r32(sd + 4) >= octets { fini = true; break; } self.stall(1000); }
        self.w8(sd, 0);
        fini
    }
}
