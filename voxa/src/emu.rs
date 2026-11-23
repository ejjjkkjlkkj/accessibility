// Emulateur x86-64 maison, limite au sous-ensemble d'instructions que le compilateur Voxa emet,
// avec un faux firmware UEFI (SystemTable, ConOut->OutputString) qui controle l'ABI Microsoft x64.
// Toute instruction hors sous-ensemble provoque une erreur explicite : jamais de comportement devine.
#![allow(dead_code)]
#[path = "hda.rs"]
pub mod hda;
use hda::{Hda, Codec, Mem};

const MEM: usize = 48 << 20;
const STACK_TOP: u64 = 0x7FFF00;
const IMG_BASE: u64 = 0x400000;
const SYSTAB: u64 = 0x300000;
const CONOUT: u64 = 0x301000;
const TRAP_OUTPUT: u64 = 0xFFF0_0000;
const TRAP_READKEY: u64 = 0xFFF2_0000;
const CONIN: u64 = 0x302000;
const RTSVC: u64 = 0x303000;
const TRAP_RESET: u64 = 0xFFF3_0000;
const BOOTSVC: u64 = 0x304000;
const GOP: u64 = 0x305000;
const SMBIOS_ENTRY: u64 = 0x30F000;
const CONFTAB: u64 = 0x30F100;
const SMBIOS_TABLES: u64 = 0x310000;
const GOPMODE: u64 = 0x305100;
const GOPINFO: u64 = 0x305200;
const TRAP_LOCATE: u64 = 0xFFF4_0000;
const TRAP_STALL: u64 = 0xFFF5_0000;
const TRAP_MEMMAP: u64 = 0xFFF6_0000;
const TRAP_LOCATEBUF: u64 = 0xFFF7_0000;
const TRAP_HIIEXPORT: u64 = 0xFFF8_0000;
const TRAP_ALLOCPOOL: u64 = 0xFFF9_0000;
const TRAP_EXITBS: u64 = 0xFFFA_0000;
const TRAP_HANDLEPROT: u64 = 0xFFFB_0000;
const LOADED_IMAGE: u64 = 0x321000;
const LI_GUID: [u8; 16] = [0xA1, 0x31, 0x1B, 0x5B, 0x62, 0x95, 0xD2, 0x11, 0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B];
const RSDP_ADDR: u64 = 0x320000;
const XSDT_ADDR: u64 = 0x320100;
const MCFG_ADDR: u64 = 0x320200;
const FADT_ADDR: u64 = 0x320300;
const DSDT_ADDR: u64 = 0x320400;
const ECAM_BASE: u64 = 0xF000_0000;
const ECAM_SPAN: u64 = 0x0100_0000;
const HIIDB: u64 = 0x306000;
const POOL_BASE: u64 = 0x200_0000;
const HII_GUID: [u8; 16] = [0x72, 0xC1, 0x9F, 0xEF, 0xB2, 0xA1, 0x93, 0x46, 0xB3, 0x27, 0x6D, 0x32, 0xFC, 0x41, 0x60, 0x42];
const HII_DATA: &[u8] = &[]; // écrans HII réels de la machine non publiés (données personnelles de matériel)
const GOP_GUID: [u8; 16] = [0xDE, 0xA9, 0x42, 0x90, 0xDC, 0x23, 0x38, 0x4A, 0x96, 0xFB, 0x7A, 0xDE, 0xD0, 0x80, 0x51, 0x6A];
const RET_MAGIC: u64 = 0xFFF1_0000;
const MAX_STEPS: u64 = 12_000_000_000;
/// Limite d'instructions : 12 milliards par defaut ; la variable d'environnement VOXA_MAX_STEPS la reduit pour echouer vite (la sortie serie reste alors affichee).
pub fn limite_pas() -> u64 { static L: std::sync::OnceLock<u64> = std::sync::OnceLock::new(); *L.get_or_init(|| std::env::var("VOXA_MAX_STEPS").ok().and_then(|s| s.parse().ok()).unwrap_or(MAX_STEPS)) }

#[derive(Default, Debug)]
pub struct Outcome {
    pub exited_bs: bool,
    pub rax: u64,
    pub console: String,
    pub serial: Vec<u8>,
    pub steps: u64,
    pub violations: Vec<String>,
    pub halted: bool,
    pub ports_read: Vec<u16>,
    pub main_ret: i64,
    pub reset: Option<i64>,
    pub fb: Vec<u8>,
    pub tones: Vec<(u64, u32)>,
    pub time_us: u64,
    pub scancodes_left: usize,
    pub screen: (u32, u32),
    pub pcm: Vec<u8>,
    pub plays: Vec<hda::PlayReport>,
    pub hda_errors: Vec<String>,
    pub codec_log: Vec<String>,
    pub miroir: String,
    pub annonces: String,
}

struct Cpu { r: [u64; 16], rip: u64, zf: bool, sf: bool, of: bool, cf: bool, pf: bool, df: bool, generic: bool, mem: Vec<u8>, out: Outcome, keys: std::collections::VecDeque<(u16, u16)>, plat: Platform, fb_on: bool }
const RAX: usize = 0; const RCX: usize = 1; const RDX: usize = 2; const RBX: usize = 3; const RSP: usize = 4; const RBP: usize = 5; const R8: usize = 8; const R9: usize = 9;

impl Cpu {
    fn new() -> Cpu { Cpu { r: [0; 16], rip: 0, zf: false, sf: false, of: false, cf: false, pf: false, df: false, generic: false, mem: vec![0; MEM], out: Outcome::default(), keys: Default::default(), plat: Platform::new(), fb_on: false } }
    fn chk(&self, a: u64, n: usize) -> Result<usize, String> {
        if (a as usize).checked_add(n).map(|e| e <= MEM).unwrap_or(false) { Ok(a as usize) } else { Err(format!("accès mémoire hors limites à 0x{a:X}")) }
    }
    fn rd(&self, a: u64, n: usize) -> Result<u64, String> {
        if a >= ECAM_BASE && a < ECAM_BASE + ECAM_SPAN { let (bus, dev, func, reg) = (((a - ECAM_BASE) >> 20) as u32, ((a >> 15) & 0x1F) as u32, ((a >> 12) & 7) as u32, (a & 0xFFF) as u32); let w = self.plat.cfg_read(bus, dev, func, reg & 0xFC) as u64 >> (8 * (reg & 3)); return Ok(match n { 1 => w & 0xFF, 2 => w & 0xFFFF, _ => w & 0xFFFF_FFFF }); }
        // Contrôleur I2C AMD (0xFEDC5000) et GPIO AMD (0xFED81500) : lecture seule, valeurs modélisées d après la documentation du composant (jamais mesurées).
        if a >= 0xFEDC_5000 && a < 0xFEDC_6000 { return Ok(match a - 0xFEDC_5000 { 0xFC => 0x4457_0140, 0x70 => 6, 0x6C => 0, _ => 0 }); }
        if a >= 0xFED8_1500 && a < 0xFED8_1900 { return Ok(0x0040_0000); }
        if let Some((i, off, en)) = self.plat.mmio_window(a) { return Ok(if en { self.plat.mmio_read(i, off, n) as u64 } else { 0xFFFF_FFFF }); }
        let i = self.chk(a, n)?; let mut v = 0u64; for k in 0..n { v |= (self.mem[i + k] as u64) << (8 * k); } Ok(v) }
    fn wr(&mut self, a: u64, n: usize, v: u64) -> Result<(), String> {
        if let Some((i, off, en)) = self.plat.mmio_window(a) {
            if en { let mut p = std::mem::take(&mut self.plat); { let mut m = VecMem(&mut self.mem); p.mmio_write(i, off, n, v as u32, &mut m); } self.plat = p; }
            return Ok(());
        }
        let i = self.chk(a, n)?; for k in 0..n { self.mem[i + k] = (v >> (8 * k)) as u8; } Ok(()) }
    fn push(&mut self, v: u64) -> Result<(), String> { self.r[RSP] = self.r[RSP].wrapping_sub(8); let s = self.r[RSP]; self.wr(s, 8, v) }
    fn pop(&mut self) -> Result<u64, String> { let s = self.r[RSP]; let v = self.rd(s, 8)?; self.r[RSP] = s.wrapping_add(8); Ok(v) }
    fn cmp(&mut self, a: u64, b: u64) { let res = a.wrapping_sub(b); self.zf = res == 0; self.sf = (res as i64) < 0; self.of = (((a ^ b) & (a ^ res)) >> 63) != 0; }

    fn trap_output(&mut self) -> Result<(), String> {
        if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel du firmware (rsp=0x{:X}, attendu ≡ 8 mod 16 à l'entrée)", self.r[RSP])); }
        if self.r[RCX] != CONOUT { self.out.violations.push(format!("ABI : OutputString appelé avec This=0x{:X} au lieu de ConOut=0x{CONOUT:X}", self.r[RCX])); }
        let mut p = self.r[RDX]; let mut units = Vec::new();
        loop { let u = self.rd(p, 2)? as u16; if u == 0 { break; } units.push(u); p += 2; if units.len() > 100_000 { return Err("chaîne UTF-16 sans fin".into()); } }
        self.out.console.push_str(&String::from_utf16_lossy(&units));
        self.r[RAX] = 0; // EFI_SUCCESS
        let ra = self.pop()?; self.rip = ra; Ok(())
    }

    /// EFI_SIMPLE_TEXT_INPUT_PROTOCOL.ReadKeyStroke(This, EFI_INPUT_KEY *Key) : ABI controlee comme OutputString.
    fn trap_readkey(&mut self) -> Result<(), String> {
        if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel de ReadKeyStroke (rsp=0x{:X})", self.r[RSP])); }
        if self.r[RCX] != CONIN { self.out.violations.push(format!("ABI : ReadKeyStroke appelé avec This=0x{:X} au lieu de ConIn=0x{CONIN:X}", self.r[RCX])); }
        match self.keys.pop_front() {
            Some((scan, uni)) => { let k = self.r[RDX]; self.wr(k, 2, scan as u64)?; self.wr(k + 2, 2, uni as u64)?; self.r[RAX] = 0; }
            None => self.r[RAX] = 0x8000_0000_0000_0006, // EFI_NOT_READY
        }
        let ra = self.pop()?; self.rip = ra; Ok(())
    }

    fn step(&mut self) -> Result<bool, String> {
        if self.rip == RET_MAGIC { return Ok(false); }
        if self.out.exited_bs && self.rip >= 0xFFF0_0000 && self.rip != RET_MAGIC && self.rip != TRAP_EXITBS { self.out.violations.push(format!("appel d'un service UEFI (trap {:#x}) APRES ExitBootServices", self.rip)); self.out.halted = true; return Ok(false); }
        if self.rip == TRAP_HANDLEPROT {
            // EFI_BOOT_SERVICES.HandleProtocol(Handle, Protocol, Interface) : seul EFI_LOADED_IMAGE_PROTOCOL est modelise (base 0x400000, taille 1 Mo : MODELE)
            if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel de HandleProtocol (rsp=0x{:X})", self.r[RSP])); }
            let mut g = [0u8; 16]; for k in 0..16 { g[k] = self.rd(self.r[RDX] + k as u64, 1)? as u8; }
            if g == LI_GUID { let o = self.r[R8]; self.wr(o, 8, LOADED_IMAGE)?; self.r[RAX] = 0; } else { self.r[RAX] = 0x8000_0000_0000_0003; }
            let ra = self.pop()?; self.rip = ra; return Ok(true);
        }
        if self.rip == TRAP_EXITBS {
            // EFI_BOOT_SERVICES.ExitBootServices(ImageHandle, MapKey)
            if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel de ExitBootServices (rsp=0x{:X})", self.r[RSP])); }
            if self.r[RDX] != 0x1234 { self.r[RAX] = 0x8000_0000_0000_0002; } else { self.out.exited_bs = true; self.r[RAX] = 0; }
            let ra = self.pop()?; self.rip = ra; return Ok(true);
        }
        if self.rip == TRAP_OUTPUT { self.trap_output()?; return Ok(true); }
        if self.rip == TRAP_READKEY { self.trap_readkey()?; return Ok(true); }
        if self.rip == TRAP_RESET {
            // EFI_RUNTIME_SERVICES.ResetSystem(ResetType, ResetStatus, DataSize, ResetData) : ne revient jamais
            if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel de ResetSystem (rsp=0x{:X})", self.r[RSP])); }
            if self.r[RDX] != 0 || self.r[R8] != 0 || self.r[R9] != 0 { self.out.violations.push("ABI : ResetStatus/DataSize/ResetData attendus nuls".into()); }
            self.out.reset = Some(self.r[RCX] as i64); self.out.halted = true; return Ok(false);
        }
        if self.rip == TRAP_LOCATE {
            // EFI_BOOT_SERVICES.LocateProtocol(Protocol, Registration, Interface)
            if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel de LocateProtocol (rsp=0x{:X})", self.r[RSP])); }
            let mut g = [0u8; 16]; for k in 0..16 { g[k] = self.rd(self.r[RCX] + k as u64, 1)? as u8; }
            if g == GOP_GUID { let o = self.r[R8]; self.wr(o, 8, GOP)?; self.r[RAX] = 0; } else if g == HII_GUID { let o = self.r[R8]; self.wr(o, 8, HIIDB)?; self.r[RAX] = 0; } else { self.r[RAX] = 0x8000_0000_0000_000E; }
            let ra = self.pop()?; self.rip = ra; return Ok(true);
        }
        if self.rip == TRAP_MEMMAP {
            // EFI_BOOT_SERVICES.GetMemoryMap(MemoryMapSize*, MemoryMap*, MapKey*, DescriptorSize*, DescriptorVersion*) : carte SYNTHÉTIQUE (pas celle d'un vrai firmware)
            if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel de GetMemoryMap (rsp=0x{:X})", self.r[RSP])); }
            const MAP: [(u32, u64, u64); 7] = [(7, 0x1000, 159), (7, 0x10_0000, 7168), (4, 0x1D0_0000, 1024), (5, 0x210_0000, 256), (9, 0x220_0000, 64), (7, 0x4000_0000, 262_144), (11, 0xF000_0000, 65_536)];
            let need = (MAP.len() * 48) as u64; let psz = self.r[RCX]; let have = self.rd(psz, 8)?; let key = self.r[R8]; let dsz = self.r[R9];
            let dver = self.rd(self.r[RSP] + 0x28, 8)?;
            self.wr(psz, 8, need)?;
            if have < need { self.r[RAX] = 0x8000_0000_0000_0005; } else {
                let buf = self.r[RDX];
                for (i, (ty, start, pages)) in MAP.iter().enumerate() { let o = buf + (i * 48) as u64; self.wr(o, 4, *ty as u64)?; self.wr(o + 4, 4, 0)?; self.wr(o + 8, 8, *start)?; self.wr(o + 16, 8, 0)?; self.wr(o + 24, 8, *pages)?; self.wr(o + 32, 8, 0xF)?; self.wr(o + 40, 8, 0)?; }
                self.wr(key, 8, 0x1234)?; self.wr(dsz, 8, 48)?; self.wr(dver, 4, 1)?; self.r[RAX] = 0;
            }
            let ra = self.pop()?; self.rip = ra; return Ok(true);
        }
        if self.rip == TRAP_LOCATEBUF {
            // EFI_BOOT_SERVICES.LocateHandleBuffer : le faux firmware n'a AUCUN volume (aucun Windows) -> EFI_NOT_FOUND ; le chemin de succès n'est pas simulé
            if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel de LocateHandleBuffer (rsp=0x{:X})", self.r[RSP])); }
            self.r[RAX] = 0x8000_0000_0000_000E;
            let ra = self.pop()?; self.rip = ra; return Ok(true);
        }
        if self.rip == TRAP_ALLOCPOOL {
            // EFI_BOOT_SERVICES.AllocatePool(PoolType, Size, Buffer*) : pointeur croissant, zone de 16 Mo
            if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel de AllocatePool (rsp=0x{:X})", self.r[RSP])); }
            let size = (self.r[RDX] + 15) & !15; let cur = self.rd(POOL_BASE - 8, 8)?; let cur = if cur == 0 { POOL_BASE } else { cur };
            if cur + size > POOL_BASE + (8 << 20) { self.r[RAX] = 0x8000_0000_0000_0009; } else { self.wr(self.r[R8], 8, cur)?; self.wr(POOL_BASE - 8, 8, cur + size)?; self.r[RAX] = 0; }
            let ra = self.pop()?; self.rip = ra; return Ok(true);
        }
        if self.rip == TRAP_HIIEXPORT {
            // EFI_HII_DATABASE_PROTOCOL.ExportPackageLists(This, Handle, BufferSize*, Buffer) : les ecrans reels du BIOS 308 (donnees extraites hors ligne)
            if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel de ExportPackageLists (rsp=0x{:X})", self.r[RSP])); }
            let psz = self.r[R8]; let have = self.rd(psz, 8)?; let buf = self.r[R9]; let need = HII_DATA.len() as u64;
            self.wr(psz, 8, need)?;
            if have < need || buf == 0 { self.r[RAX] = 0x8000_0000_0000_0005; } else { for (k, x) in HII_DATA.iter().enumerate() { self.wr(buf + k as u64, 1, *x as u64)?; } self.r[RAX] = 0; }
            let ra = self.pop()?; self.rip = ra; return Ok(true);
        }
        if self.rip == TRAP_STALL {
            if self.r[RSP] % 16 != 8 { self.out.violations.push(format!("ABI : pile mal alignée à l'appel de Stall (rsp=0x{:X})", self.r[RSP])); }
            self.plat.time_us += self.r[RCX]; self.r[RAX] = 0;
            let ra = self.pop()?; self.rip = ra; return Ok(true);
        }
        if self.generic { return self.step_generic(); }
        let mut b = [0u8; 16];
        for k in 0..16 { b[k] = self.rd(self.rip + k as u64, 1).unwrap_or(0) as u8; }
        let i32at = |o: usize| i32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) as i64;
        let mut len = 0usize;
        let rip = self.rip;
        macro_rules! bad { () => { return Err(format!("instruction non modélisée à 0x{rip:X} : {:02X} {:02X} {:02X} {:02X} {:02X}", b[0], b[1], b[2], b[3], b[4])) } }
        match b[0] {
            0x55 => { let v = self.r[RBP]; self.push(v)?; len = 1 }
            0x5D => { self.r[RBP] = self.pop()?; len = 1 }
            0x50 => { let v = self.r[RAX]; self.push(v)?; len = 1 }
            0x53 => { let v = self.r[RBX]; self.push(v)?; len = 1 }
            0x5B => { self.r[RBX] = self.pop()?; len = 1 }
            0x51 => { let v = self.r[RCX]; self.push(v)?; len = 1 }
            0x52 => { let v = self.r[RDX]; self.push(v)?; len = 1 }
            0x49 if b[1] == 0x89 && b[2] == 0xC0 => { self.r[R8] = self.r[RAX]; len = 3 }
            0x31 if b[1] == 0xC9 => { self.r[RCX] = 0; len = 2 }
            0x89 if b[1] == 0xC0 => { self.r[RAX] &= 0xFFFF_FFFF; len = 2 }
            0x4A if b[1] == 0x8B && b[2] == 0x04 && b[3] == 0xC4 => { let a = self.r[RSP].wrapping_add(self.r[R8].wrapping_mul(8)); self.r[RAX] = self.rd(a, 8)?; len = 4 }
            0x58 => { self.r[RAX] = self.pop()?; len = 1 }
            0x59 => { self.r[RCX] = self.pop()?; len = 1 }
            0x5A => { self.r[RDX] = self.pop()?; len = 1 }
            0xC3 => { self.rip = self.pop()?; return Ok(true) }
            0xF4 => { self.out.halted = true; return Ok(false) }
            0xE8 => { let t = rip.wrapping_add(5).wrapping_add(i32at(1) as u64); self.push(rip + 5)?; self.rip = t; return Ok(true) }
            0xE9 => { self.rip = rip.wrapping_add(5).wrapping_add(i32at(1) as u64); return Ok(true) }
            0xEB => { self.rip = rip.wrapping_add(2).wrapping_add(b[1] as i8 as i64 as u64); return Ok(true) }
            0xEE => { let (port, v) = (self.r[RDX] as u16, self.r[RAX] as u8); self.plat.out(port, v); len = 1 }
            0xEF => { let (port, v) = (self.r[RDX] as u16, self.r[RAX] as u32); self.plat.out32(port, v); len = 1 }
            0xED => { let port = self.r[RDX] as u16; self.r[RAX] = self.plat.in32(port) as u64; len = 1 }
            0x66 if b[1] == 0x89 && b[2] == 0x01 => { let a = self.r[RCX]; let v = self.r[RAX] & 0xFFFF; self.wr(a, 2, v)?; len = 3 }
            0xEC => { let port = self.r[RDX] as u16; self.out.ports_read.push(port); let v = self.plat.inb(port); self.r[RAX] = (self.r[RAX] & !0xFF) | v as u64; len = 1 }
            0xB0 => { self.r[RAX] = (self.r[RAX] & !0xFF) | b[1] as u64; len = 2 }
            0x8B if b[1] == 0x00 => { let a = self.r[RAX]; self.r[RAX] = self.rd(a, 4)?; len = 2 }
            0x8B if b[1] == 0x42 => { let a = self.r[RDX].wrapping_add(b[2] as i8 as i64 as u64); self.r[RAX] = self.rd(a, 4)?; len = 3 }
            0xB8 => { self.r[RAX] = i32at(1) as u32 as u64; len = 5 }
            0x88 if b[1] == 0x01 => { let a = self.r[RCX]; let v = self.r[RAX] & 0xFF; self.wr(a, 1, v)?; len = 2 }
            0x89 if b[1] == 0x01 => { let a = self.r[RCX]; let v = self.r[RAX] & 0xFFFF_FFFF; self.wr(a, 4, v)?; len = 2 }
            0x8B if b[1] == 0x05 => { let a = rip.wrapping_add(6).wrapping_add(i32at(2) as u64); self.r[RAX] = self.rd(a, 4)?; len = 6 }
            0xC1 if b[1] == 0xC8 => { let v = self.r[RAX] as u32; self.r[RAX] = v.rotate_right(b[2] as u32 & 31) as u64; len = 3 }
            0x31 if b[1] == 0xC0 => { self.r[RAX] = 0; len = 2 }
            0x31 if b[1] == 0xD2 => { self.r[RDX] = 0; len = 2 }
            0x45 if b[1] == 0x31 && b[2] == 0xC0 => { self.r[R8] = 0; len = 3 }
            0x45 if b[1] == 0x31 && b[2] == 0xC9 => { self.r[R9] = 0; len = 3 }
            0x66 if b[1] == 0xBA => { self.r[RDX] = (self.r[RDX] & !0xFFFF) | u16::from_le_bytes([b[2], b[3]]) as u64; len = 4 }
            0xFF if b[1] == 0xD0 => { let t = self.r[RAX]; self.push(rip + 2)?; self.rip = t; return Ok(true) }
            0x41 if b[1] == 0x58 => { self.r[R8] = self.pop()?; len = 2 }
            0x41 if b[1] == 0x59 => { self.r[R9] = self.pop()?; len = 2 }
            0x0F => match b[1] {
                0xA2 => { let (l, s) = (self.r[RAX] as u32, self.r[RCX] as u32); let c = unsafe { core::arch::x86_64::__cpuid_count(l, s) }; self.r[RAX] = c.eax as u64; self.r[RBX] = c.ebx as u64; self.r[RCX] = c.ecx as u64; self.r[RDX] = c.edx as u64; len = 2 }
                0x84 => { let tgt = rip.wrapping_add(6).wrapping_add(i32at(2) as u64); self.rip = if self.zf { tgt } else { rip + 6 }; return Ok(true) }
                0x85 => { let tgt = rip.wrapping_add(6).wrapping_add(i32at(2) as u64); self.rip = if !self.zf { tgt } else { rip + 6 }; return Ok(true) }
                0xAF if b[2] == 0xC1 => { self.r[RAX] = (self.r[RAX] as i64).wrapping_mul(self.r[RCX] as i64) as u64; len = 3 }
                0xB6 if b[2] == 0xC0 => { self.r[RAX] &= 0xFF; len = 3 }
                0x94 | 0x95 | 0x9C | 0x9D | 0x9E | 0x9F if b[2] == 0xC0 => {
                    let (zf, sf, of) = (self.zf, self.sf, self.of);
                    let v = match b[1] { 0x94 => zf, 0x95 => !zf, 0x9C => sf != of, 0x9D => sf == of, 0x9E => zf || sf != of, _ => !zf && sf == of };
                    self.r[RAX] = (self.r[RAX] & !0xFF) | v as u64; len = 3
                }
                _ => bad!(),
            },
            0x48 | 0x4C => {
                let rex_r = b[0] == 0x4C; // 4C : registre source r8..r15
                match (b[1], b[2]) {
                    (0x89, 0xE5) => { self.r[RBP] = self.r[RSP]; len = 3 }
                    (0x89, 0xEC) => { self.r[RSP] = self.r[RBP]; len = 3 }
                    (0x89, 0xE0) => { self.r[RAX] = self.r[RSP]; len = 3 }
                    (0x89, 0xC1) => { self.r[RCX] = self.r[RAX]; len = 3 }
                    (0x89, 0xC2) => { self.r[RDX] = self.r[RAX]; len = 3 }
                    (0x89, 0xD0) => { self.r[RAX] = self.r[RDX]; len = 3 }
                    (0x89, m @ (0x85 | 0x8D | 0x95)) => {
                        let src = if rex_r { if m == 0x85 { R8 } else { R9 } } else { match m { 0x85 => RAX, 0x8D => RCX, _ => RDX } };
                        let a = self.r[RBP].wrapping_add(i32at(3) as u64); let v = self.r[src]; self.wr(a, 8, v)?; len = 7
                    }
                    (0x8B, 0x85) => { let a = self.r[RBP].wrapping_add(i32at(3) as u64); self.r[RAX] = self.rd(a, 8)?; len = 7 }
                    (0x89, m @ (0x15 | 0x0D)) => { let a = rip.wrapping_add(7).wrapping_add(i32at(3) as u64); let v = if m == 0x15 { self.r[RDX] } else { self.r[RCX] }; self.wr(a, 8, v)?; len = 7 }
                    (0x8B, 0x0D) => { let a = rip.wrapping_add(7).wrapping_add(i32at(3) as u64); self.r[RCX] = self.rd(a, 8)?; len = 7 }
                    (0x8D, 0x15) => { self.r[RDX] = rip.wrapping_add(7).wrapping_add(i32at(3) as u64); len = 7 }
                    (0x8B, 0x49) => { let a = self.r[RCX].wrapping_add(b[3] as i8 as i64 as u64); self.r[RCX] = self.rd(a, 8)?; len = 4 }
                    (0x8B, 0x41) => { let a = self.r[RCX].wrapping_add(b[3] as i8 as i64 as u64); self.r[RAX] = self.rd(a, 8)?; len = 4 }
                    (0x89, 0x44) if b[3] == 0x24 => { let a = self.r[RSP].wrapping_add(b[4] as i8 as i64 as u64); let v = self.r[RAX]; self.wr(a, 8, v)?; len = 5 }
                    (0x8B, 0x64) if b[3] == 0x24 => { let a = self.r[RSP].wrapping_add(b[4] as i8 as i64 as u64); self.r[RSP] = self.rd(a, 8)?; len = 5 }
                    (0x8B, 0x05) => { let a = rip.wrapping_add(7).wrapping_add(i32at(3) as u64); self.r[RAX] = self.rd(a, 8)?; len = 7 }
                    (0x8B, 0x40) => { let a = self.r[RAX].wrapping_add(b[3] as i8 as i64 as u64); self.r[RAX] = self.rd(a, 8)?; len = 4 }
                    (0x0F, 0xB7) if b[3] == 0x00 => { let a = self.r[RAX]; self.r[RAX] = self.rd(a, 2)?; len = 4 }
                    (0x0F, 0xB6) if b[3] == 0x00 => { let a = self.r[RAX]; self.r[RAX] = self.rd(a, 1)?; len = 4 }
                    (0x8B, 0x80) => { let a = self.r[RAX].wrapping_add(i32at(3) as u64); self.r[RAX] = self.rd(a, 8)?; len = 7 }
                    (0x8D, 0x0D) => { self.r[RCX] = rip.wrapping_add(7).wrapping_add(i32at(3) as u64); len = 7 }
                    (0x8D, 0x05) => { let v = rip.wrapping_add(7).wrapping_add(i32at(3) as u64); if rex_r { self.r[R8] = v; } else { self.r[RAX] = v; } len = 7 }
                    (0x8B, 0x48) => { let a = self.r[RAX].wrapping_add(b[3] as i8 as i64 as u64); self.r[RCX] = self.rd(a, 8)?; len = 4 }
                    (0x8B, 0x51) => { let a = self.r[RCX].wrapping_add(b[3] as i8 as i64 as u64); self.r[RDX] = self.rd(a, 8)?; len = 4 }
                    (0x8B, 0x00) => { let a = self.r[RAX]; self.r[RAX] = self.rd(a, 8)?; len = 3 }
                    (0x89, 0x01) => { let a = self.r[RCX]; let v = self.r[RAX]; self.wr(a, 8, v)?; len = 3 }
                    (0x81, 0xEC) => { self.r[RSP] = self.r[RSP].wrapping_sub(i32at(3) as u64); len = 7 }
                    (0x83, 0xEC) => { self.r[RSP] = self.r[RSP].wrapping_sub(b[3] as i8 as i64 as u64); len = 4 }
                    (0x83, 0xC4) => { self.r[RSP] = self.r[RSP].wrapping_add(b[3] as i8 as i64 as u64); len = 4 }
                    (0x83, 0xE4) => { self.r[RSP] &= b[3] as i8 as i64 as u64; len = 4 }
                    (0xC7, 0xC0) => { self.r[RAX] = i32at(3) as u64; len = 7 }
                    (0x89, 0x05) => { let a = rip.wrapping_add(7).wrapping_add(i32at(3) as u64); let v = self.r[RAX]; self.wr(a, 8, v)?; len = 7 }
                    (0xC7, 0xC4) => { self.r[RSP] = i32at(3) as u64; len = 7 }
                    (0xB8, _) => { let mut v = [0u8; 8]; v.copy_from_slice(&b[2..10]); self.r[RAX] = u64::from_le_bytes(v); len = 10 }
                    (0x01, 0xC8) => { self.r[RAX] = self.r[RAX].wrapping_add(self.r[RCX]); len = 3 }
                    (0x29, 0xC8) => { self.r[RAX] = self.r[RAX].wrapping_sub(self.r[RCX]); len = 3 }
                    (0x0F, 0xAF) if b[3] == 0xC1 => { self.r[RAX] = (self.r[RAX] as i64).wrapping_mul(self.r[RCX] as i64) as u64; len = 4 }
                    (0x99, _) => { self.r[RDX] = if (self.r[RAX] as i64) < 0 { u64::MAX } else { 0 }; len = 2 }
                    (0xF7, 0xF9) => {
                        let d = self.r[RCX] as i64 as i128; if d == 0 { return Err(format!("#DE division par zéro à 0x{rip:X}")); }
                        let n = (((self.r[RDX] as i64 as i128) << 64) | self.r[RAX] as i128) as i128;
                        let (q, r) = (n / d, n % d);
                        if q > i64::MAX as i128 || q < i64::MIN as i128 { return Err(format!("#DE débordement à 0x{rip:X}")); }
                        self.r[RAX] = q as i64 as u64; self.r[RDX] = r as i64 as u64; len = 3
                    }
                    (0xF7, 0xD8) => { self.r[RAX] = (self.r[RAX] as i64).wrapping_neg() as u64; len = 3 }
                    (0x39, 0xC8) => { let (a, c) = (self.r[RAX], self.r[RCX]); self.cmp(a, c); len = 3 }
                    (0x85, 0xC9) => { let a = self.r[RCX]; self.zf = a == 0; self.sf = (a as i64) < 0; self.of = false; len = 3 }
                    (0x85, 0xC0) => { let a = self.r[RAX]; self.zf = a == 0; self.sf = (a as i64) < 0; self.of = false; len = 3 }
                    _ => bad!(),
                }
            }
            _ => bad!(),
        }
        self.rip = rip + len as u64;
        Ok(true)
    }

    fn run(&mut self) -> Result<(), String> {
        let r = self.run_inner();
        self.out.serial = self.plat.serial.clone(); self.out.tones = self.plat.tones.clone(); self.out.time_us = self.plat.time_us;
        self.out.scancodes_left = self.plat.scancodes.len(); self.out.screen = (self.plat.width, self.plat.height);
        for h in &self.plat.hdas { self.out.pcm.extend_from_slice(&h.pcm); self.out.plays.extend(h.plays.iter().cloned()); self.out.hda_errors.extend(h.errors.iter().cloned()); if let Some(c) = &h.codec { self.out.codec_log.extend(c.log.iter().cloned()); } }
        // miroir du texte affiché (applications EFI) : longueur à scratch+799000, points de code 16 bits à scratch+800000 ; scratch = image + 0x2000
        // journal des annonces (texte phonétique, une par ligne) : longueur à scratch+829000, octets à scratch+830000
        { let s = (IMG_BASE + 0x2000) as usize; if s + 840000 < MEM { let nl = u64::from_le_bytes(self.mem[s + 829000..s + 829008].try_into().unwrap()) as usize; if nl <= 4000 { self.out.annonces = String::from_utf8_lossy(&self.mem[s + 830000..s + 830000 + nl]).into_owned(); } } }
        { let s = (IMG_BASE + 0x2000) as usize; if s + 820000 < MEM { let nl = u64::from_le_bytes(self.mem[s + 799000..s + 799008].try_into().unwrap()) as usize; if nl < 7001 { self.out.miroir = (0..nl).map(|k| { let c = u16::from_le_bytes([self.mem[s + 800000 + 2 * k], self.mem[s + 800001 + 2 * k]]); char::from_u32(c as u32).unwrap_or('?') }).collect(); } } }
        let n = self.plat.width as usize * self.plat.height as usize * 4;
        if self.fb_on && FB_BASE as usize + n <= MEM { self.out.fb = self.mem[FB_BASE as usize..FB_BASE as usize + n].to_vec(); }
        r
    }
    fn run_inner(&mut self) -> Result<(), String> {
        while self.step()? {
            self.out.steps += 1;
            if self.out.steps > limite_pas() { self.out.violations.push("limite d'instructions atteinte : boucle sans fin probable".into()); self.out.halted = true; return Ok(()); }
        }
        self.out.rax = self.r[RAX];
        Ok(())
    }
}

fn load(c: &mut Cpu, base: u64, bytes: &[u8]) -> Result<(), String> {
    let i = c.chk(base, bytes.len())?; c.mem[i..i + bytes.len()].copy_from_slice(bytes); Ok(())
}

include!("emu_x64.rs");

/// Execute du code relogeable (mode hote) : appelle `entry` sans argument, retourne rax.
pub fn run_raw(code: &[u8], entry: usize) -> Result<Outcome, String> {
    let mut c = Cpu::new(); load(&mut c, IMG_BASE, code)?;
    c.r[RSP] = STACK_TOP - 8; let s = c.r[RSP]; c.wr(s, 8, RET_MAGIC)?;
    c.rip = IMG_BASE + entry as u64; c.run()?; Ok(c.out)
}
/// Image nue : demarre au premier octet, sa propre pile (le stub fait mov rsp,0x90000).
pub fn run_bare(code: &[u8]) -> Result<Outcome, String> { run_bare_full(code, &[]) }
pub fn run_bare_full(code: &[u8], scancodes: &[u8]) -> Result<Outcome, String> {
    let mut c = Cpu::new(); c.plat.scancodes = scancodes.iter().copied().collect(); load(&mut c, IMG_BASE, code)?; c.rip = IMG_BASE; c.r[RSP] = 0x1000; c.run()?; Ok(c.out)
}
/// Application UEFI : charge le PE, construit un faux SystemTable/ConOut, appelle l'entree en ABI MS x64.
pub fn run_efi(img: &[u8]) -> Result<Outcome, String> { run_efi_keys(img, false, &[]) }
pub fn run_efi_opts(img: &[u8], sans_console: bool) -> Result<Outcome, String> { run_efi_keys(img, sans_console, &[]) }

/// Touches UEFI : "down,up,left,right,enter,esc,home,end,pgup,pgdn,char:x" -> (ScanCode, UnicodeChar).
pub fn parse_touches(s: &str) -> Vec<(u16, u16)> {
    s.split(|c: char| c == ',' || c.is_whitespace()).filter(|t| !t.is_empty()).filter_map(|t| {
        Some(match t {
            "up" => (0x01, 0), "down" => (0x02, 0), "right" => (0x03, 0), "left" => (0x04, 0),
            "home" => (0x05, 0), "end" => (0x06, 0), "pgup" => (0x09, 0), "pgdn" => (0x0A, 0),
            "esc" => (0x17, 0), "enter" => (0, 13), "tab" => (0, 9), "space" => (0, 32),
            _ => { let c = t.strip_prefix("char:")?.chars().next()?; (0, c as u32 as u16) }
        })
    }).collect()
}

/// `sans_console` : SystemTable->ConOut = NULL (machine sans console texte). `keys` : touches que ConIn fournira dans l'ordre.
pub fn run_efi_keys(img: &[u8], sans_console: bool, keys: &[(u16, u16)]) -> Result<Outcome, String> { run_efi_full(img, sans_console, keys, &[]) }
/// `scancodes` : octets que le controleur clavier 8042 (ports 0x60/0x64) fournira dans l'ordre.
pub fn run_efi_full(img: &[u8], sans_console: bool, keys: &[(u16, u16)], scancodes: &[u8]) -> Result<Outcome, String> {
    let u16at = |o: usize| u16::from_le_bytes([img[o], img[o + 1]]) as usize;
    let u32at = |o: usize| u32::from_le_bytes([img[o], img[o + 1], img[o + 2], img[o + 3]]) as usize;
    if img.len() < 0x100 || &img[0..2] != b"MZ" { return Err("pas de signature MZ".into()); }
    let pe = u32at(0x3C);
    if img.len() < pe + 24 || &img[pe..pe + 4] != b"PE\0\0" { return Err("pas de signature PE".into()); }
    if u16at(pe + 4) != 0x8664 { return Err("machine différente de x86-64".into()); }
    let nsec = u16at(pe + 6); let optsz = u16at(pe + 20); let opt = pe + 24;
    if u16at(opt) != 0x20B { return Err("en-tête optionnel non PE32+".into()); }
    if u16at(opt + 68) != 10 { return Err(format!("sous-système {} au lieu de 10 (application EFI)", u16at(opt + 68))); }
    let entry = u32at(opt + 16) as u64; let size_img = u32at(opt + 56);
    let mut c = Cpu::new();
    if size_img > 0x300000 { return Err("image trop grande pour l'émulateur".into()); }
    let sec = opt + optsz;
    let mut entry_ok = false;
    for s in 0..nsec {
        let h = sec + 40 * s;
        let (vs, rva, rs, rp) = (u32at(h + 8), u32at(h + 12), u32at(h + 16), u32at(h + 20));
        if rp + rs > img.len() { return Err(format!("section {s} hors du fichier")); }
        let n = rs.min(vs.max(rs));
        load(&mut c, IMG_BASE + rva as u64, &img[rp..rp + n])?;
        if (entry as usize) >= rva && (entry as usize) < rva + vs.max(rs) { entry_ok = true; }
    }
    if !entry_ok { return Err("le point d'entrée n'est dans aucune section".into()); }
    // faux firmware : SystemTable->ConOut (+0x40), ConOut->OutputString (+0x08) = piege
    c.wr(SYSTAB + 0x40, 8, if sans_console { 0 } else { CONOUT })?; c.wr(CONOUT + 8, 8, TRAP_OUTPUT)?;
    c.wr(SYSTAB + 0x58, 8, RTSVC)?; c.wr(RTSVC + 0x68, 8, TRAP_RESET)?; // RuntimeServices (+0x58), ResetSystem (+0x68)
    c.wr(SYSTAB + 0x30, 8, CONIN)?; c.wr(CONIN + 8, 8, TRAP_READKEY)?; // EFI_SYSTEM_TABLE.ConIn (+0x30), ReadKeyStroke (+0x08)
    // relogements de base (PE produit par rustc) : DIR64 uniquement ; le mode général x86-64 est activé quand l image en contient
    { let rel = u32at(opt + 112 + 5 * 8) as u64; let rsz = u32at(opt + 112 + 5 * 8 + 4) as u64;
      if rsz > 0 {
        let ib = u64::from_le_bytes(img[opt + 24..opt + 32].try_into().unwrap()); let delta = IMG_BASE.wrapping_sub(ib);
        let (mut p, end) = (rel, rel + rsz);
        while p + 8 <= end { let page = c.rd(IMG_BASE + p, 4)?; let bs = c.rd(IMG_BASE + p + 4, 4)?; if bs < 8 { break; }
          let mut e = p + 8; while e + 2 <= p + bs { let ent = c.rd(IMG_BASE + e, 2)?; let (ty, off) = (ent >> 12, ent & 0xFFF);
            if ty == 10 { let a = IMG_BASE + page + off; let v = c.rd(a, 8)?.wrapping_add(delta); c.wr(a, 8, v)?; }
            else if ty != 0 { return Err(format!("type de relogement {ty} non géré")); }
            e += 2; }
          p += bs; }
        c.generic = true;
      } }
    c.keys = keys.iter().copied().collect();
    c.plat.scancodes = scancodes.iter().copied().collect();
    // BootServices (+0x60) : LocateProtocol (+0x140), Stall (+0xF8) ; GOP : Mode (+0x18) -> Info (+0x08), FrameBufferBase (+0x18)
    // tables de configuration : une entrée SMBIOS3 (GUID F2FD1544-9794-4A2C-992E-E5BBCF20E394) vers une table SMBIOS vide (la table réelle de la machine n est pas publiée)
    { let smb: &[u8] = &[]; /* table SMBIOS réelle non publiée (données de la machine) */ for (k, x) in smb.iter().enumerate() { c.wr(SMBIOS_TABLES + k as u64, 1, *x as u64)?; }
      let mut e = [0u8; 0x18]; e[..5].copy_from_slice(b"_SM3_"); e[6] = 0x18; e[7] = 3; e[8] = 3; e[12] = 1; e[0x0C..0x10].copy_from_slice(&(smb.len() as u32).to_le_bytes()); e[0x10..0x18].copy_from_slice(&SMBIOS_TABLES.to_le_bytes());
      let s: u8 = e.iter().fold(0u8, |a, x| a.wrapping_add(*x)); e[5] = 0u8.wrapping_sub(s);
      for (k, x) in e.iter().enumerate() { c.wr(SMBIOS_ENTRY + k as u64, 1, *x as u64)?; }
      let guid: [u8; 16] = [0x44, 0x15, 0xFD, 0xF2, 0x94, 0x97, 0x2C, 0x4A, 0x99, 0x2E, 0xE5, 0xBB, 0xCF, 0x20, 0xE3, 0x94];
      for (k, x) in guid.iter().enumerate() { c.wr(CONFTAB + k as u64, 1, *x as u64)?; }
      c.wr(CONFTAB + 16, 8, SMBIOS_ENTRY)?; c.wr(SYSTAB + 0x68, 8, 1)?; c.wr(SYSTAB + 0x70, 8, CONFTAB)?; }
    // ACPI 2.0 : RSDP -> XSDT -> {MCFG, FADT}. Valeurs d'un modele, pas celles de la vraie machine (sauf l'ECAM en 0xF0000000, lu dans la vraie table MCFG).
    { let cks = |c: &mut Cpu, a: u64, len: usize, off: usize| -> Result<(), String> { c.wr(a + off as u64, 1, 0)?; let mut s: u8 = 0; for k in 0..len { s = s.wrapping_add(c.rd(a + k as u64, 1)? as u8); } c.wr(a + off as u64, 1, (0u8.wrapping_sub(s)) as u64) };
      let put = |c: &mut Cpu, a: u64, b: &[u8]| -> Result<(), String> { for (k, x) in b.iter().enumerate() { c.wr(a + k as u64, 1, *x as u64)?; } Ok(()) };
      let hdr = |c: &mut Cpu, a: u64, sig: &[u8; 4], len: u32, rev: u8| -> Result<(), String> { put(c, a, sig)?; c.wr(a + 4, 4, len as u64)?; c.wr(a + 8, 1, rev as u64)?; put(c, a + 10, b"VOXAEM")?; put(c, a + 16, b"VOXAOS00")?; Ok(()) };
      // MCFG : en-tete 36 + 8 reserves + 1 entree de 16 octets
      hdr(&mut c, MCFG_ADDR, b"MCFG", 60, 1)?; c.wr(MCFG_ADDR + 44, 8, ECAM_BASE)?; c.wr(MCFG_ADDR + 52, 2, 0)?; c.wr(MCFG_ADDR + 54, 1, 0)?; c.wr(MCFG_ADDR + 55, 1, 15)?; cks(&mut c, MCFG_ADDR, 60, 9)?;
      // FADT : PM1a_CNT_BLK (offset 64) = 0x1804
      { let aml = [0x08u8, 0x5F, 0x53, 0x35, 0x5F, 0x12, 0x08, 0x04, 0x0A, 0x05, 0x0A, 0x05, 0x00, 0x00]; hdr(&mut c, DSDT_ADDR, b"DSDT", 36 + aml.len() as u32, 2)?; put(&mut c, DSDT_ADDR + 36, &aml)?; cks(&mut c, DSDT_ADDR, 36 + aml.len(), 9)?; }
      hdr(&mut c, FADT_ADDR, b"FACP", 276, 5)?; c.wr(FADT_ADDR + 64, 4, 0x1804)?; c.wr(FADT_ADDR + 40, 4, DSDT_ADDR)?; cks(&mut c, FADT_ADDR, 276, 9)?;
      // XSDT : 2 entrees
      hdr(&mut c, XSDT_ADDR, b"XSDT", 36 + 16, 1)?; c.wr(XSDT_ADDR + 36, 8, FADT_ADDR)?; c.wr(XSDT_ADDR + 44, 8, MCFG_ADDR)?; cks(&mut c, XSDT_ADDR, 52, 9)?;
      // RSDP rev 2 (36 octets)
      put(&mut c, RSDP_ADDR, b"RSD PTR ")?; put(&mut c, RSDP_ADDR + 9, b"VOXAEM")?; c.wr(RSDP_ADDR + 15, 1, 2)?; c.wr(RSDP_ADDR + 20, 4, 36)?; c.wr(RSDP_ADDR + 24, 8, XSDT_ADDR)?; cks(&mut c, RSDP_ADDR, 20, 8)?; cks(&mut c, RSDP_ADDR, 36, 32)?;
      let g: [u8; 16] = [0x71, 0xE8, 0x68, 0x88, 0xF1, 0xE4, 0xD3, 0x11, 0xBC, 0x22, 0x00, 0x80, 0xC7, 0x3C, 0x88, 0x81];
      put(&mut c, CONFTAB + 24, &g)?; c.wr(CONFTAB + 40, 8, RSDP_ADDR)?; c.wr(SYSTAB + 0x68, 8, 2)?; }
    c.wr(SYSTAB + 0x60, 8, BOOTSVC)?; c.wr(BOOTSVC + 0x140, 8, TRAP_LOCATE)?; c.wr(BOOTSVC + 0xF8, 8, TRAP_STALL)?; c.wr(BOOTSVC + 0x38, 8, TRAP_MEMMAP)?; c.wr(BOOTSVC + 0x138, 8, TRAP_LOCATEBUF)?; c.wr(BOOTSVC + 0x40, 8, TRAP_ALLOCPOOL)?; c.wr(BOOTSVC + 0xE8, 8, TRAP_EXITBS)?; c.wr(BOOTSVC + 0x98, 8, TRAP_HANDLEPROT)?; c.wr(LOADED_IMAGE + 0x40, 8, IMG_BASE)?; c.wr(LOADED_IMAGE + 0x48, 8, 0x10_0000)?; c.wr(HIIDB + 0x20, 8, TRAP_HIIEXPORT)?;
    let (sw, sh) = (c.plat.width as u64, c.plat.height as u64);
    c.wr(GOP + 0x18, 8, GOPMODE)?; c.wr(GOPMODE, 4, 1)?; c.wr(GOPMODE + 8, 8, GOPINFO)?; c.wr(GOPMODE + 16, 8, 36)?;
    c.wr(GOPMODE + 0x18, 8, FB_BASE)?; c.wr(GOPMODE + 0x20, 8, sw * sh * 4)?;
    c.wr(GOPINFO + 4, 4, sw)?; c.wr(GOPINFO + 8, 4, sh)?; c.wr(GOPINFO + 12, 4, 1)?; c.wr(GOPINFO + 32, 4, sw)?; // format 1 = BGRX, pas = largeur
    c.fb_on = true;
    c.r[RCX] = 0x1234; c.r[RDX] = SYSTAB;
    c.r[RSP] = STACK_TOP - 8; let s = c.r[RSP]; c.wr(s, 8, RET_MAGIC)?;
    let (rsp0, rbp0) = (c.r[RSP], c.r[RBP]);
    c.rip = IMG_BASE + entry; c.run()?;
    // journal d annonces écrit par l application Rust : marque VXJRNL01, longueur u64, octets
    { let lo = IMG_BASE as usize; let hi = (IMG_BASE as usize + size_img).min(c.mem.len()); if let Some(pos) = c.mem[lo..hi].windows(8).position(|w| w == b"VXJRNL01") { let j = lo + pos; let nl = u64::from_le_bytes(c.mem[j + 8..j + 16].try_into().unwrap()) as usize; if nl <= 4096 { c.out.annonces = String::from_utf8_lossy(&c.mem[j + 16..j + 16 + nl]).into_owned(); } } }
    if c.out.reset.is_some() { return Ok(c.out); } // ResetSystem ne revient pas : pas de controle de retour
    if c.out.exited_bs { return Ok(c.out); } // noyau apres ExitBootServices : il ne revient jamais au firmware, pas de controle de retour
    c.out.main_ret = c.rd(IMG_BASE + 0x1000 + 24, 8)? as i64; // resultat de main stocke par le stub
    if c.r[RSP] != rsp0 + 8 { c.out.violations.push(format!("ABI : pile déséquilibrée au retour (rsp=0x{:X}, attendu 0x{:X})", c.r[RSP], rsp0 + 8)); }
    if c.r[RBP] != rbp0 { c.out.violations.push("ABI : rbp non préservé".into()); }
    if c.out.rax != 0 { c.out.violations.push(format!("EFI_STATUS non nul : 0x{:X}", c.out.rax)); }
    Ok(c.out)
}

// ---------------------------------------------------------------- modele de plate-forme partage (emulateur ET interpreteur)
pub const FB_BASE: u64 = 0x100_0000;
/// Resolution de l'ecran simule : variable d'environnement VOXA_ECRAN="LxH", sinon 640x480.
pub fn screen_mode() -> (u32, u32) {
    std::env::var("VOXA_ECRAN").ok().and_then(|s| { let (w, h) = s.split_once('x')?; Some((w.trim().parse().ok()?, h.trim().parse().ok()?)) }).unwrap_or((640, 480))
}
/// Ports d'entree/sortie de la machine simulee : debug 0x3F8, PIT canal 2 (0x42/0x43), haut-parleur systeme (0x61), clavier 8042 (0x60/0x64).
#[derive(Debug, Clone, Default)]
pub struct Platform {
    pub serial: Vec<u8>,
    pub time_us: u64,
    /// evenements du haut-parleur : (instant en microsecondes, frequence en Hz ; 0 = silence)
    pub tones: Vec<(u64, u32)>,
    pub scancodes: std::collections::VecDeque<u8>,
    pit_armed: bool, pit_lo: Option<u8>, pub pit_divisor: u16, port61: u8,
    pub width: u32, pub height: u32,
    pci_addr: u32, pub hdas: Vec<Hda>, pci_cmd: [u16; 2], pub poweroff: bool, pub kbc_reset: bool, cmos_reg: u8,
}
impl Platform {
    pub fn new() -> Platform {
        let (w, h) = screen_mode();
        Platform { width: w, height: h, hdas: vec![Hda::new(HDA_BAR0, if std::env::var("VOXA_HDA_NOCODEC").is_ok() { None } else { Some(Codec::new()) }), Hda::new(HDA_BAR1, None)], ..Default::default() }
    }
    pub fn out(&mut self, port: u16, v: u8) {
        match port {
            0x3F8 => self.serial.push(v),
            0x70 => self.cmos_reg = v & 0x7F,
            0x64 => if v == 0xFE { self.kbc_reset = true; },
            0x43 => { self.pit_armed = v == 0xB6; self.pit_lo = None; } // canal 2, octet bas puis haut, mode 3
            0x42 => if self.pit_armed { match self.pit_lo.take() { None => self.pit_lo = Some(v), Some(lo) => self.pit_divisor = ((v as u16) << 8) | lo as u16 } },
            0x61 => {
                let (was_on, now_on) = (self.port61 & 3 == 3, v & 3 == 3);
                self.port61 = v;
                if now_on && !was_on { let f = if self.pit_divisor > 0 { 1_193_182 / self.pit_divisor as u32 } else { 0 }; self.tones.push((self.time_us, f)); }
                if !now_on && was_on { self.tones.push((self.time_us, 0)); }
            }
            _ => {}
        }
    }
    pub fn inb(&mut self, port: u16) -> u8 {
        match port {
            0x64 => if self.scancodes.is_empty() { 0 } else { 1 }, // bit 0 : tampon de sortie plein
            0x60 => self.scancodes.pop_front().unwrap_or(0),
            0x71 => match self.cmos_reg { 0x04 => 0x10, 0x02 => 0x42, 0x0B => 0x02, _ => 0 }, // horloge materielle MODELE : 10 h 42, BCD, 24 h
            0x61 => { self.port61 ^= 0x10; self.time_us += 15; self.port61 } // bit 4 : bascule d'horloge de rafraichissement (~15 us), sert aux delais
            _ => 0,
        }
    }
    pub fn fb_info(&self, n: i64) -> i64 {
        match n { 0 => FB_BASE as i64, 1 => self.width as i64, 2 => self.height as i64, 3 => self.width as i64, 4 => 1, 5 => 1, _ => 0 }
    }
}

// ---------------------------------------------------------------- PCI (mecanisme 1 : ports 0xCF8/0xCFC) et fenetres memoire des controleurs audio
pub const HDA_BAR0: u64 = 0xFCEF_0000; // controleur audio du haut-parleur (1022:15E3, bus 3 dispositif 0 fonction 6 sur cette machine)
pub const HDA_BAR1: u64 = 0xFCEF_8000; // controleur audio HDMI (1002:1637, fonction 1), sans codec
pub const HDA_SPAN: u64 = 0x4000;
impl Platform {
    pub fn out32(&mut self, port: u16, v: u32) {
        match port {
            0x1804 => if v & (1 << 13) != 0 { self.poweroff = true; }, // PM1a_CNT : SLP_EN (modele)
            0xCF8 => self.pci_addr = v,
            0xCFC => {
                let (bus, dev, func, reg) = ((self.pci_addr >> 16) & 0xFF, (self.pci_addr >> 11) & 0x1F, (self.pci_addr >> 8) & 7, self.pci_addr & 0xFC);
                if self.pci_addr & 0x8000_0000 != 0 && reg == 0x04 { if let Some(i) = Self::pci_hda(bus, dev, func) { self.pci_cmd[i] = v as u16; } }
            }
            _ => {}
        }
    }
    fn pci_hda(bus: u32, dev: u32, func: u32) -> Option<usize> { match (bus, dev, func) { (3, 0, 6) => Some(0), (3, 0, 1) => Some(1), _ => None } }
    pub fn in32(&mut self, port: u16) -> u32 {
        match port {
            0xCF8 => self.pci_addr,
            0xCFC => {
                if self.pci_addr & 0x8000_0000 == 0 { return 0xFFFF_FFFF; }
                let (bus, dev, func, reg) = ((self.pci_addr >> 16) & 0xFF, (self.pci_addr >> 11) & 0x1F, (self.pci_addr >> 8) & 7, self.pci_addr & 0xFC);
                // (fabricant, produit, classe<<8|revision, type d'en-tete, BAR0)
                let d: Option<(u32, u32, u32, u32, u32)> = match (bus, dev, func) {
                    (0, 0, 0) => Some((0x1022, 0x1630, 0x0600_0000, 0x80, 0)),
                    (3, 0, 0) => Some((0x1002, 0x1638, 0x0300_00C5, 0x80, 0)),
                    (3, 0, 1) => Some((0x1002, 0x1637, 0x0403_0000, 0x00, HDA_BAR1 as u32)),
                    (3, 0, 5) => Some((0x1022, 0x15E2, 0x0480_0001, 0x00, 0)),
                    (3, 0, 6) => Some((0x1022, 0x15E3, 0x0403_0000, 0x00, HDA_BAR0 as u32)),
                    _ => None,
                };
                match d {
                    None => 0xFFFF_FFFF,
                    Some((ven, dev, class, hdr, bar0)) => match reg {
                        0x00 => ven | dev << 16,
                        0x04 => Self::pci_hda(bus, (self.pci_addr >> 11) & 0x1F, func).map(|i| self.pci_cmd[i] as u32).unwrap_or(0),
                        0x08 => class,
                        0x0C => hdr << 16,
                        0x10 => bar0,
                        _ => 0,
                    },
                }
            }
            _ => 0,
        }
    }
    /// Lecture de l'espace de configuration d'un peripherique (utilisee par ECAM). Memes valeurs que le mecanisme 1.
    pub fn cfg_read(&self, bus: u32, dev: u32, func: u32, reg: u32) -> u32 {
        let d: Option<(u32, u32, u32, u32, u32)> = match (bus, dev, func) {
            (0, 0, 0) => Some((0x1022, 0x1630, 0x0600_0000, 0x80, 0)),
            (3, 0, 0) => Some((0x1002, 0x1638, 0x0300_00C5, 0x80, 0)),
            (3, 0, 1) => Some((0x1002, 0x1637, 0x0403_0000, 0x00, HDA_BAR1 as u32)),
            (3, 0, 5) => Some((0x1022, 0x15E2, 0x0480_0001, 0x00, 0)),
            (3, 0, 6) => Some((0x1022, 0x15E3, 0x0403_0000, 0x00, HDA_BAR0 as u32)),
            _ => None };
        match d { None => 0xFFFF_FFFF, Some((ven, dv, class, hdr, bar0)) => match reg { 0x00 => ven | dv << 16, 0x04 => Self::pci_hda(bus, dev, func).map(|i| self.pci_cmd[i] as u32).unwrap_or(0), 0x08 => class, 0x0C => hdr << 16, 0x10 => bar0, _ => 0 } }
    }
    /// Si l'adresse tombe dans la fenetre memoire d'un controleur : (indice, decalage, acces memoire active ?)
    pub fn mmio_window(&self, a: u64) -> Option<(usize, usize, bool)> {
        for (i, h) in self.hdas.iter().enumerate() { if a >= h.base && a < h.base + HDA_SPAN { return Some((i, (a - h.base) as usize, self.pci_cmd[i] & 2 != 0)); } }
        None
    }
    pub fn mmio_read(&self, idx: usize, off: usize, size: usize) -> u32 { self.hdas[idx].read(off, size) }
    pub fn mmio_write(&mut self, idx: usize, off: usize, size: usize, v: u32, mem: &mut dyn Mem) {
        let bm = self.pci_cmd[idx] & 4 != 0;
        self.hdas[idx].write(off, size, v, mem, bm);
    }
}
/// Memoire de l'emulateur vue par le modele de DMA.
pub struct VecMem<'a>(pub &'a mut Vec<u8>);
impl Mem for VecMem<'_> {
    fn rd8(&self, a: u64) -> u8 { self.0.get(a as usize).copied().unwrap_or(0) }
    fn wr8(&mut self, a: u64, v: u8) { if let Some(x) = self.0.get_mut(a as usize) { *x = v; } }
}

/// CPUID de la machine qui exécute l'outil (feuille l, sous-feuille 0) ; r : 0 eax, 1 ebx, 2 ecx, 3 edx.
pub fn cpuid_host(l: u32, r: usize) -> u32 {
    let c = unsafe { core::arch::x86_64::__cpuid_count(l, 0) };
    match r { 0 => c.eax, 1 => c.ebx, 2 => c.ecx, _ => c.edx }
}
