// Agent de dissection UEFI (sans OS) - LECTURE SEULE.
// Ecrit uniquement sur le volume d'ou il est lance (\EFI\HWA2\).
// Ne modifie ni disque, ni NVRAM, ni firmware, ni menu BIOS.
#![no_main]
#![no_std]

extern crate alloc;

mod idt;
mod screen;
mod usb_enum;
mod usb_hid;
mod usb_hub;
mod nvme;
mod hda_play;
mod usb_r0;
mod cpu_sonde;
mod lectures;
mod voix;

mod arena;
use screen::Screen;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use uefi::boot::{self, MemoryType, OpenProtocolAttributes, OpenProtocolParams};
use uefi::fs::FileSystem;
use uefi::mem::memory_map::MemoryMap;
use uefi::prelude::*;
use uefi::proto::hii::database::HiiDatabase;
use uefi::proto::media::block::BlockIO;
use uefi::proto::pci::root_bridge::PciRootBridgeIo;
use sha2::{Digest, Sha256};
use uefi::{system, runtime, CString16};

#[global_allocator]
static ALLOCATOR: arena::Arena = arena::Arena;

const OUT: &str = "\\EFI\\HWA2\\";
const SECTOR_MAX_READ: usize = 1 << 20; // 1 MiB de hachage par partition

// Reglages de securite : ces lectures peuvent bloquer le PC (ecran noir).
// Tout est desactive par defaut. Mettre true seulement si on accepte le risque.
const FULL_DISK_HASH: bool = false; // hachage de TOUT le disque : desactive (le PC s'arrete vers 65 Go, voir RAPPORT_UNIQUE.TXT)
// Mode complet : toutes les analyses sont actives. L'ecran n'est jamais efface pendant
// l'analyse (le texte reste visible meme si le PC se bloque). Le hachage complet du disque
// reste desactive : sur ce PC il s'est arrete vers 65 Go (voir RAPPORT_UNIQUE.TXT).
// Desactive : la lecture de 16 Mo a 0xFF000000 bloque le PC (le rapport s'arrete a l'ETAPE 10).
const FLASH_WINDOW: bool = false; // fenetre SPI 0xFF000000 (16 Mo)
// Desactive : les lectures MSR bloquaient le PC (phase 18 sans lecture).
const MSR_READS: bool = false; // MSR de configuration
const BAR_READS: bool = true; // registres BAR0 de chaque peripherique PCI
// Un seul fichier de sortie : RAPPORT_UNIQUE.TXT. Les dumps binaires separes ne sont
// ecrits que si ce drapeau est mis a true.
const DUMP_BINARIES: bool = true;

// Symbole C requis par le crate uefi (lien sans libc).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wcslen(s: *const u16) -> usize {
    let mut n = 0usize;
    // SAFETY: chaine UCS-2 terminee par 0, fournie par le firmware.
    unsafe {
        while *s.add(n) != 0 {
            n += 1;
        }
    }
    n
}

fn p(s: &str) -> CString16 {
    CString16::try_from(s).unwrap_or_else(|_| CString16::try_from("X").unwrap())
}

struct Rep {
    txt: String,
    fs: Option<FileSystem>,
    scr: Option<Screen>,
}

impl Rep {
    fn line(&mut self, s: &str) {
        let _ = writeln!(self.txt, "{}", s);
        if let Some(sc) = self.scr.as_mut() {
            sc.put(s);
            sc.put("\n");
        }
        // Avant chaque etape risquee : ecriture immediate sur la cle. Si le PC se fige ou
        // redemarre, le rapport indique la derniere etape commencee.
        if s.starts_with("ETAPE") || s.contains("peut bloquer") {
            self.save();
        }
    }
    /// Ecrit le rapport sur la cle. Sans effet apres la sortie des services UEFI (fs = None).
    fn save(&mut self) {
        if let Some(fs) = self.fs.as_mut() {
            let c = p(&format!("{}RAPPORT_UNIQUE.TXT", OUT));
            let _ = fs.write(uefi::fs::Path::new(&c), self.txt.as_bytes());
        }
    }
    fn file(&mut self, name: &str, data: &[u8]) {
        if !DUMP_BINARIES {
            return;
        }
        if let Some(fs) = self.fs.as_mut() {
            let c = p(&format!("{}{}", OUT, name));
            let _ = fs.write(uefi::fs::Path::new(&c), data);
        }
    }
}

fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn u64le(b: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(b[o..o + 8].try_into().unwrap())
}
fn u32le(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

fn guid_str(b: &[u8]) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        u32le(b, 0),
        u16::from_le_bytes([b[4], b[5]]),
        u16::from_le_bytes([b[6], b[7]]),
        b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

fn ctype_name(t: u32) -> &'static str {
    match t {
        0 => "Reserved", 1 => "LoaderCode", 2 => "LoaderData", 3 => "BootServicesCode",
        4 => "BootServicesData", 5 => "RuntimeServicesCode", 6 => "RuntimeServicesData",
        7 => "Conventional", 8 => "Unusable", 9 => "ACPIReclaim", 10 => "ACPINVS",
        11 => "MMIO", 12 => "MMIOPortSpace", 13 => "PalCode", 14 => "Persistent", _ => "?",
    }
}

fn fs_kind(s: &[u8]) -> &'static str {
    if s.len() < 512 { return "?"; }
    if &s[3..11] == b"NTFS    " { return "NTFS"; }
    if &s[3..11] == b"EXFAT   " { return "exFAT"; }
    if &s[0x52..0x5A] == b"FAT32   " { return "FAT32"; }
    if &s[0x36..0x3E] == b"FAT16   " || &s[0x36..0x3B] == b"FAT12" { return "FAT12/16"; }
    if s[510] == 0x55 && s[511] == 0xAA { return "boot-sig (MBR)"; }
    "inconnu"
}

fn open_bio(h: uefi::Handle) -> Option<uefi::boot::ScopedProtocol<BlockIO>> {
    // SAFETY: GetProtocol n'exclut pas le pilote, lecture seule.
    unsafe {
        boot::open_protocol::<BlockIO>(
            OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
            OpenProtocolAttributes::GetProtocol,
        )
        .ok()
    }
}

// Octet de la structure SMBIOS au decalage `off` (0 si hors de la zone formatee).
fn oct_smb(t: &[u8], p: usize, len: usize, off: usize) -> u8 {
    if off < len && p + off < t.len() { t[p + off] } else { 0 }
}

fn u16_smb(t: &[u8], p: usize, len: usize, off: usize) -> u16 {
    u16::from_le_bytes([oct_smb(t, p, len, off), oct_smb(t, p, len, off + 1)])
}

// Decode les structures SMBIOS utiles : BIOS (0), systeme (1), carte mere (2), processeur (4),
// tableau memoire (16), module memoire (17). Numeros de serie et UUID volontairement NON lus
// (regle du README de la cle : confidentialite).
fn decrire_smbios(t: &[u8], r: &mut Rep) {
    let mut p = 0usize;
    let mut n = 0;
    while p + 4 <= t.len() && n < 4096 {
        let typ = t[p];
        let len = t[p + 1] as usize;
        if len < 4 || p + len > t.len() { break; }
        // Chaines : apres la zone formatee, terminees par NUL ; liste finie par un NUL vide.
        let mut q = p + len;
        let mut chaines: Vec<String> = Vec::new();
        while q < t.len() {
            if t[q] == 0 {
                q += 1;
                break;
            }
            let debut = q;
            while q < t.len() && t[q] != 0 { q += 1; }
            chaines.push(String::from_utf8_lossy(&t[debut..q]).into_owned());
            q += 1;
        }
        let chaine = |idx: u8| -> String {
            if idx == 0 { String::from("-") } else { chaines.get(idx as usize - 1).cloned().unwrap_or_else(|| String::from("?")) }
        };
        match typ {
            0 => {
                let v = chaine(oct_smb(t, p, len, 0x05));
                let d = chaine(oct_smb(t, p, len, 0x08));
                let f = chaine(oct_smb(t, p, len, 0x04));
                r.line(&format!("  SMBIOS BIOS : fabricant {} | version {} | date {}", f, v, d));
            }
            1 => {
                let f = chaine(oct_smb(t, p, len, 0x04));
                let pr = chaine(oct_smb(t, p, len, 0x05));
                let v = chaine(oct_smb(t, p, len, 0x06));
                r.line(&format!("  SMBIOS systeme : fabricant {} | produit {} | version {}", f, pr, v));
            }
            2 => {
                let f = chaine(oct_smb(t, p, len, 0x04));
                let pr = chaine(oct_smb(t, p, len, 0x05));
                let v = chaine(oct_smb(t, p, len, 0x06));
                r.line(&format!("  SMBIOS carte mere : fabricant {} | produit {} | version {}", f, pr, v));
            }
            4 => {
                let sock = chaine(oct_smb(t, p, len, 0x04));
                let f = chaine(oct_smb(t, p, len, 0x07));
                // SMBIOS type 4 : Max Speed au decalage 0x14, Current Speed au decalage 0x16.
                let vmax = u16_smb(t, p, len, 0x14);
                let vact = u16_smb(t, p, len, 0x16);
                r.line(&format!("  SMBIOS processeur : socket {} | fabricant {} | vitesse max {} MHz | actuelle {} MHz", sock, f, vmax, vact));
            }
            16 => {
                let cap = u32::from_le_bytes([oct_smb(t, p, len, 7), oct_smb(t, p, len, 8), oct_smb(t, p, len, 9), oct_smb(t, p, len, 10)]);
                let nb = oct_smb(t, p, len, 0x0D);
                if cap == 0x8000_0000 {
                    let ext = u64::from_le_bytes([oct_smb(t, p, len, 0x0F), oct_smb(t, p, len, 0x10), oct_smb(t, p, len, 0x11), oct_smb(t, p, len, 0x12),
                        oct_smb(t, p, len, 0x13), oct_smb(t, p, len, 0x14), oct_smb(t, p, len, 0x15), oct_smb(t, p, len, 0x16)]);
                    r.line(&format!("  SMBIOS tableau memoire : capacite max {} Mo ; {} emplacements", ext / 1024, nb));
                } else {
                    r.line(&format!("  SMBIOS tableau memoire : capacite max {} Mo ; {} emplacements", cap / 1024, nb));
                }
            }
            17 => {
                let taille = u16_smb(t, p, len, 0x0C);
                let emplacement = chaine(oct_smb(t, p, len, 0x10));
                let vitesse = u16_smb(t, p, len, 0x15);
                let f = chaine(oct_smb(t, p, len, 0x17));
                let piece = chaine(oct_smb(t, p, len, 0x1A));
                let mo: String = if taille == 0 { String::from("vide") }
                    else if taille == 0xFFFF { String::from("inconnu") }
                    else if taille == 0x7FFF {
                        format!("{} Mo", u32::from_le_bytes([oct_smb(t, p, len, 0x1C), oct_smb(t, p, len, 0x1D), oct_smb(t, p, len, 0x1E), oct_smb(t, p, len, 0x1F)]))
                    }
                    else if taille & 0x8000 != 0 { format!("{} Ko", taille & 0x7FFF) }
                    else { format!("{} Mo", taille) };
                r.line(&format!("  SMBIOS module memoire : emplacement {} | taille {} | vitesse {} MT/s | fabricant {} | reference {}",
                    emplacement, mo, vitesse, f, piece));
            }
            127 => break,
            _ => {}
        }
        p = q;
        n += 1;
    }
}

fn le32(s: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([s[off], s[off + 1], s[off + 2], s[off + 3]])
}

fn nom_cap_pci(id: u8) -> &'static str {
    match id {
        0x01 => "Power Management",
        0x05 => "MSI",
        0x07 => "PCI-X",
        0x09 => "Vendor specific",
        0x10 => "PCI Express",
        0x11 => "MSI-X",
        0x12 => "SATA",
        0x13 => "Advanced Features",
        _ => "inconnue",
    }
}

fn nom_cap_ext(id: u16) -> &'static str {
    match id {
        0x01 => "AER (erreurs avancees)",
        0x02 => "Virtual Channel",
        0x03 => "Device Serial Number",
        0x04 => "Power Budgeting",
        0x0D => "ACS",
        0x0E => "ARI",
        0x10 => "SR-IOV",
        0x15 => "LTR",
        0x19 => "Secondary PCIe",
        0x1E => "L1 PM Substates",
        _ => "inconnue",
    }
}

// Decrit une fonction PCI a partir de sa configuration (lecture seule, aucune ecriture) :
// BAR (type, adresse), capacites standard et etendues.
fn decrire_pci(cfg: &[u8], ext: &[u8], bus: u8, dev: u8, fun: u8, r: &mut Rep) {
    let typ = cfg[14] & 0x7F;
    let nb = match typ { 0 => 6usize, 1 => 2usize, _ => 0usize };
    r.line(&format!("  {:02x}:{:02x}.{} type d'en-tete {} ; {} BAR", bus, dev, fun, typ, nb));
    let mut i = 0usize;
    while i < nb {
        let off = 0x10 + 4 * i;
        let v = le32(cfg, off);
        if v == 0 { i += 1; continue; }
        if v & 1 == 1 {
            r.line(&format!("    BAR{} E/S port {:#x}", i, v & !0x3));
        } else {
            let mode = (v >> 1) & 0x3;
            let prefetch = (v >> 3) & 1;
            let mut base = (v & !0xF) as u64;
            let mut taille_bar = 32;
            if mode == 2 && i + 1 < nb {
                base |= (le32(cfg, off + 4) as u64) << 32;
                taille_bar = 64;
                i += 1;
            }
            r.line(&format!("    BAR{} memoire {}b {:#x} prefetch {}", i, taille_bar, base, prefetch));
        }
        i += 1;
    }
    // Capacites standard (liste chainee depuis 0x34, si bit 4 du statut)
    if cfg[6] & 0x10 != 0 {
        let mut p = (cfg[0x34] & 0xFC) as usize;
        let mut n = 0;
        while p >= 0x40 && p < 0x100 && n < 48 {
            let id = cfg[p];
            r.line(&format!("    capacite {:#04x} @ {:#x} : {}", id, p, nom_cap_pci(id)));
            p = (cfg[p + 1] & 0xFC) as usize;
            n += 1;
        }
    }
    // Capacites etendues (PCIe, depuis 0x100)
    let mut p = 0x100usize;
    let mut n = 0;
    while p >= 0x100 && p + 4 <= ext.len() && n < 64 {
        let h = le32(ext, p);
        if h == 0 || h == 0xFFFF_FFFF { break; }
        let id = (h & 0xFFFF) as u16;
        r.line(&format!("    capacite etendue {:#06x} @ {:#x} : {}", id, p, nom_cap_ext(id)));
        if id == 0x0001 {
            r.line(&lectures::decrire_aer(ext, p));
        }
        let next = ((h >> 20) & 0xFFF) as usize;
        if next == 0 { break; }
        p = next;
        n += 1;
    }
}

/// Descripteur de configuration brut (GET_DESCRIPTOR, type 2, index 0) lu sur UsbIo, avant la sortie
/// des services UEFI. Lit d'abord l'en-tete de 9 octets pour connaitre wTotalLength.
fn config_brute(u: &mut uefi::proto::usb::io::UsbIo) -> Option<alloc::vec::Vec<u8>> {
    use uefi::proto::usb::io::ControlTransfer;
    let mut tete = [0u8; 9];
    u.control_transfer(0x80, 6, 0x0200, 0, ControlTransfer::DataIn(&mut tete), 1000).ok()?;
    let total = u16::from_le_bytes([tete[2], tete[3]]) as usize;
    if !(9..=4096).contains(&total) {
        return None;
    }
    let mut buf = alloc::vec![0u8; total];
    u.control_transfer(0x80, 6, 0x0200, 0, ControlTransfer::DataIn(&mut buf), 1000).ok()?;
    Some(buf)
}

fn read_lbas(bio: &mut BlockIO, media_id: u32, bs: usize, lba: u64, count: usize) -> Result<Vec<u8>, Status> {
    let mut buf = alloc::vec![0u8; bs * count];
    bio.read_blocks(media_id, lba, &mut buf).map_err(|e| e.status())?;
    Ok(buf)
}

// Noms des MSR (AMD APM vol.2 rev 3.43, Table A-1 / A-8 ; 0x1A0 et 0x1B selon Intel SDM / APM).
// C001_0111 = SMBASE, C001_0112 = SMM_ADDR, C001_0113 = SMM_MASK (APM Table A-8).
// C001_001F : absent de l'APM rev 3.43 (lecture peut etre ABSENT) : conserve pour le releve.
fn msr_name(m: u32) -> &'static str {
    match m {
        0x1B => "(APIC_BASE)",
        0x1A0 => "(IA32_MISC_ENABLE, Intel)",
        0x200..=0x207 => "(MTRRphysBase/Mask)",
        0x250 => "(MTRRfix64K_00000)",
        0x258 => "(MTRRfix16K_80000)",
        0x259 => "(MTRRfix16K_A0000)",
        0x26F => "(MTRRfix4K_F8000)",
        0x277 => "(PAT)",
        0x2FF => "(MTRRdefType)",
        0xC000_0080 => "(EFER)",
        0xC001_0010 => "(SYSCFG)",
        0xC001_0015 => "(HWCR)",
        0xC001_0111 => "(SMBASE)",
        0xC001_0112 => "(SMM_ADDR)",
        0xC001_0113 => "(SMM_MASK)",
        0xC001_0114 => "(VM_CR)",
        0xC001_001F => "(NB_CFG ?, non documente APM 3.43)",
        _ => "",
    }
}

// Lecture d'un MSR (CPL0 sous UEFI), protegee : si le MSR n'existe pas sur ce processeur,
// la #GP est interceptee par l'IDT et la fonction retourne None au lieu d'arreter le programme.
unsafe fn rdmsr(msr: u32) -> Option<u64> {
    let lo: u32;
    let hi: u32;
    idt::msr_guard_begin();
    // SAFETY: rdmsr en CPL0 ; une faute (MSR absent) est recuperee par l'IDT (idt_exc_rust).
    unsafe {
        core::arch::asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi, options(nomem, nostack, preserves_flags));
    }
    if idt::msr_guard_end() {
        return None;
    }
    Some(((hi as u64) << 32) | lo as u64)
}

// Lecture d'un registre de controle (CPL0).
fn read_cr(n: u8) -> u64 {
    let v: u64;
    // SAFETY: lecture seule des registres de controle en CPL0.
    unsafe {
        match n {
            0 => core::arch::asm!("mov {}, cr0", out(reg) v, options(nomem, nostack, preserves_flags)),
            3 => core::arch::asm!("mov {}, cr3", out(reg) v, options(nomem, nostack, preserves_flags)),
            4 => core::arch::asm!("mov {}, cr4", out(reg) v, options(nomem, nostack, preserves_flags)),
            _ => v = 0,
        }
    }
    v
}

// Ecrit 8 chiffres hexadecimaux dans `dst` (au moins 8 octets).
fn put_hex(dst: &mut [u8], v: u32) {
    let hex = b"0123456789ABCDEF";
    for i in 0..8 {
        dst[i] = hex[((v >> (28 - 4 * i)) & 0xF) as usize];
    }
}

// Efface la ligne `row` du framebuffer puis y ecrit `text` (police 5x7, echelle 4).
fn draw_row(fbp: usize, fbsz: usize, stride: usize, row: usize, text: &[u8]) {
    let scale = 4usize;
    // SAFETY: framebuffer GOP fourni par le firmware avant la sortie d'UEFI.
    unsafe {
        let fb = core::slice::from_raw_parts_mut(fbp as *mut u32, fbsz / 4);
        let y0 = 40 + row * 8 * scale;
        for y in y0..y0 + 8 * scale {
            for x in 40..(40 + 40 * 6 * scale).min(stride) {
                let idx = y * stride + x;
                if idx < fb.len() {
                    fb[idx] = 0;
                }
            }
        }
        for (ci, &ch) in text.iter().enumerate() {
            let x0 = 40 + ci * 6 * scale;
            for (r, bits) in screen::font(ch).iter().enumerate() {
                for col in 0..5usize {
                    if bits & (0x10 >> col) != 0 {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                let x = x0 + col * scale + dx;
                                let idx = (y0 + r * scale + dy) * stride + x;
                                if x < stride && idx < fb.len() {
                                    fb[idx] = 0x00FF_FFFF;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Listing recursif d'un systeme de fichiers (lecture seule).
fn list_dir(fs: &mut FileSystem, path: &str, depth: usize, out: &mut String, count: &mut usize) {
    if depth > 8 || *count > 20000 {
        return;
    }
    let c = p(path);
    let Ok(it) = fs.read_dir(uefi::fs::Path::new(&c)) else { return; };
    let entries: Vec<_> = it.filter_map(|e| e.ok()).collect();
    for e in entries {
        let n = format!("{}", e.file_name());
        if n == "." || n == ".." {
            continue;
        }
        let full = if path == "\\" { format!("\\{}", n) } else { format!("{}\\{}", path, n) };
        *count += 1;
        if e.is_directory() {
            out.push_str(&format!("DIR            {}\n", full));
            list_dir(fs, &full, depth + 1, out, count);
        } else {
            out.push_str(&format!("FILE {:>12} {}\n", e.file_size(), full));
        }
    }
}

#[entry]
fn main() -> Status {
    let fs = boot::get_image_file_system(boot::image_handle())
        .ok()
        .map(FileSystem::new);
    // Desactive le watchdog du firmware (5 min par defaut, sinon reset pendant le hachage des disques)
    let _ = boot::set_watchdog_timer(0, 0, None);
    // L'ecran est ouvert tout de suite : chaque etape s'affiche avant de commencer.
    let mut r = Rep { txt: String::new(), fs, scr: screen::open() };
    r.line("HWA2 ANALYSE EN COURS (ne pas eteindre)");
    // Table des interruptions : une exception affiche son numero sur l'ecran au lieu d'un ecran noir.
    if let Some(s) = r.scr.as_ref() {
        idt::set_screen(s.info());
    }
    // SAFETY: installation de l'IDT en CPL0, avant les acces materiels.
    unsafe { idt::install() };
    r.line("IDT installee : exceptions affichees a l'ecran");
    if let Some(fs) = r.fs.as_mut() {
        let c = p(OUT);
        let _ = fs.create_dir_all(uefi::fs::Path::new(&c));
    }

    r.line("===== AGENT DE DISSECTION UEFI (sans OS, lecture seule) =====");
    if let Ok(t) = runtime::get_time() {
        r.line(&format!("Horloge RTC : {:04}-{:02}-{:02} {:02}:{:02}:{:02}", t.year(), t.month(), t.day(), t.hour(), t.minute(), t.second()));
    }
    let mut vendor_buf = String::new();
    let _ = write!(vendor_buf, "{}", system::firmware_vendor());
    let rev = system::firmware_revision();
    r.line(&format!("Firmware : {} revision {:#010x} | UEFI {:?}", vendor_buf, rev, system::uefi_revision()));
    r.save();

    // ---------- Phase 1 : tables de configuration et ACPI ----------
    r.line("\n[phase 1] Tables de configuration");
    let mut rsdp_addr: usize = 0;
    let mut tables: Vec<(String, usize)> = Vec::new();
    system::with_config_table(|entries| {
        for e in entries {
            // SAFETY: uefi::Guid est repr(C) de 16 octets.
            let gb: [u8; 16] = unsafe { core::mem::transmute(e.guid) };
            let g = guid_str(&gb);
            tables.push((g.clone(), e.address as usize));
            if g == "8868e871-e4f1-11d3-bc22-0080c73c8881" {
                rsdp_addr = e.address as usize;
            }
            if rsdp_addr == 0 && g == "eb9d2d30-2d88-11d3-9a16-0090273fc14d" {
                rsdp_addr = e.address as usize;
            }
        }
    });
    for (g, a) in &tables {
        r.line(&format!("  {} @ {:#x}", g, a));
    }
    r.line(&format!("  total : {}", tables.len()));

    // ---------- Phase 2 : dump ACPI brut (RSDP -> XSDT -> chaque table) ----------
    r.line("\n[phase 2] Dump ACPI brut");
    if rsdp_addr != 0 {
        // SAFETY: adresse fournie par la table de configuration, identite en pre-boot.
        unsafe {
            let rsdp = core::slice::from_raw_parts(rsdp_addr as *const u8, 36);
            r.line(&format!("  RSDP revision {} | oem {}", rsdp[15], core::str::from_utf8(&rsdp[9..15]).unwrap_or("?")));
            r.file("ACPI_RSDP.bin", rsdp);
            // XSDT : present seulement pour une RSDP ACPI 2.0 ou plus (revision >= 2). Sinon 0.
            let xsdt_addr = if rsdp[15] >= 2 { u64le(rsdp, 24) as usize } else { 0 };
            if xsdt_addr != 0 {
                let hdr = core::slice::from_raw_parts(xsdt_addr as *const u8, 8);
                let len = u32le(hdr, 4) as usize;
                let x = core::slice::from_raw_parts(xsdt_addr as *const u8, len);
                r.file("ACPI_XSDT.bin", x);
                let n = len.saturating_sub(36) / 8;
                r.line(&format!("  XSDT {} octets, {} entrees", len, n));
                for i in 0..n {
                    let ta = u64le(x, 36 + i * 8) as usize;
                    if ta == 0 { continue; }
                    let th = core::slice::from_raw_parts(ta as *const u8, 36);
                    let tl = u32le(th, 4) as usize;
                    let sig = core::str::from_utf8(&th[0..4]).unwrap_or("????");
                    let t = core::slice::from_raw_parts(ta as *const u8, tl);
                    let oem = core::str::from_utf8(&th[10..16]).unwrap_or("?").trim();
                    r.line(&format!("  {:<4} {:>7} o rev {} OEM {} @ {:#x} fnv={:016x}", sig, tl, th[8], oem, ta, fnv1a(t)));
                    r.file(&format!("ACPI_{}_{:02}.aml", sig.trim(), i), t);
                    // Decodage des tables utiles a l'OS (lecture seule des octets deja lus)
                    for l in lectures::decrire_acpi(sig, t) {
                        r.line(&l);
                    }
                    if sig == "FACP" {
                        let fa = lectures::fadt_adresse_facs(t) as usize;
                        if fa != 0 {
                            // SAFETY: adresse de la FACS donnee par la FADT ; memoire ACPI NVS, lecture seule.
                            let fh = core::slice::from_raw_parts(fa as *const u8, 8);
                            let fl = (u32le(fh, 4) as usize).clamp(32, 64);
                            let facs = core::slice::from_raw_parts(fa as *const u8, fl);
                            for l in lectures::decrire_facs(facs) {
                                r.line(&l);
                            }
                        }
                    }
                }
            }
        }
    } else {
        r.line("  RSDP introuvable");
    }
    r.save();

    // ---------- Phase 3 : carte memoire ----------
    r.line("\n[phase 3] Carte memoire UEFI");
    let mut totals: [u64; 16] = [0; 16];
    if let Ok(mm) = boot::memory_map(MemoryType::LOADER_DATA) {
        let mut count = 0usize;
        for d in mm.entries() {
            let t = (d.ty.0 as usize).min(15);
            totals[t] += d.page_count * 4096;
            count += 1;
        }
        r.line(&format!("  entrees : {}", count));
        // Zones que l'OS ne doit jamais allouer ni ecraser (firmware runtime, ACPI NVS, reservees).
        // Le script de reveil S3 execute du code loge dans ces zones (UEFI PI V-8.7).
        r.line("  zones interdites a l'OS (adresse debut .. fin, taille) :");
        let mut nz = 0usize;
        for d in mm.entries() {
            if let Some(nom) = lectures::zone_interdite(d.ty.0) {
                if nz < 160 {
                    let taille = d.page_count * 4096;
                    r.line(&format!(
                        "    {:#014x} .. {:#014x} {:>9} Ko  {}",
                        d.phys_start, d.phys_start + taille - 1, taille / 1024, nom
                    ));
                }
                nz += 1;
            }
        }
        r.line(&format!("  zones interdites : {}{}", nz, if nz > 160 { " (liste coupee a 160)" } else { "" }));
        for (i, v) in totals.iter().enumerate() {
            if *v > 0 {
                r.line(&format!("  {:<22} {:>10} Mo", ctype_name(i as u32), v / (1024 * 1024)));
            }
        }
    } else {
        r.line("  carte memoire illisible");
    }
    r.save();

    // ---------- Phase 4 : HII, tous les menus (export complet) ----------
    r.line("\n[phase 4] HII : export complet des menus");
    if let Ok(hs) = boot::find_handles::<HiiDatabase>() {
        r.line(&format!("  bases HII : {}", hs.len()));
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<HiiDatabase>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(db) = p {
                match db.export_all_raw() {
                    Ok(buf) => {
                        r.line(&format!("  export : {} octets", buf.len()));
                        r.file("HII_packages_export.bin", &buf);
                    }
                    Err(e) => r.line(&format!("  export echec : {:?}", e.status())),
                }
            }
        }
    } else {
        r.line("  aucune base HII trouvee");
    }
    r.save();

    // ---------- Phase 5 : tous les disques, bas niveau ----------
    r.line("\n[phase 5] Disques (bas niveau, lecture seule)");
    let bio_handles = boot::find_handles::<BlockIO>().unwrap_or_default();
    r.line(&format!("  peripheriques BlockIO : {}", bio_handles.len()));
    let mut disk_idx = 0usize;
    let mut hash_differe: Vec<(usize, uefi::Handle)> = Vec::new();
    for h in bio_handles {
        let Some(mut bio) = open_bio(h) else { r.line("  ouverture refusee"); continue; };
        let m = bio.media();
        let bs = m.block_size() as usize;
        let last = m.last_block();
        let mid = m.media_id();
        let present = m.is_media_present();
        let logical = m.is_logical_partition();
        let removable = m.is_removable_media();
        let size_gb = ((last + 1) as u128 * bs as u128) / 1_000_000_000;
        r.line(&format!(
            "  BlockIO #{} : {} | bloc {} | dernier LBA {} | ~{} Go | amovible {} | presente {} | partition-logique {}",
            disk_idx, if logical { "partition" } else { "DISQUE" }, bs, last, size_gb, removable, present, logical
        ));
        if !present || bs == 0 {
            disk_idx += 1;
            continue;
        }
        if !logical {
            // LBA 0 a 33 (MBR + GPT)
            let n = 34usize.min((last + 1) as usize);
            match read_lbas(&mut bio, mid, bs, 0, n) {
                Ok(buf) => {
                    r.file(&format!("disque{}_lba0-33.bin", disk_idx), &buf);
                    r.line(&format!("    LBA0-{} : fnv={:016x} | systeme : {}", n - 1, fnv1a(&buf), fs_kind(&buf[..bs.min(buf.len())])));
                    // GPT
                    if buf.len() >= 2 * bs && &buf[bs..bs + 8] == b"EFI PART" {
                        let hdr = &buf[bs..bs + 92];
                        let ent_lba = u64le(hdr, 72);
                        let ent_num = u32le(hdr, 80) as usize;
                        let ent_sz = u32le(hdr, 84) as usize;
                        r.line(&format!("    GPT : entrees {} x {} o a LBA {} | disk-GUID {}", ent_num, ent_sz, ent_lba, guid_str(&hdr[56..72])));
                        // Garde-fous : taille d'entree GPT valide (spec UEFI 5.3.3) avant toute lecture.
                        let gpt_ok = (1..=256).contains(&ent_num) && (128..=1024).contains(&ent_sz);
                        let ent_blocks = (ent_num * ent_sz + bs - 1) / bs;
                        if !gpt_ok {
                            r.line("    GPT : entrees invalides, ignorees");
                        } else if let Ok(ents) = read_lbas(&mut bio, mid, bs, ent_lba, ent_blocks) {
                            for i in 0..ent_num {
                                let e = &ents[i * ent_sz..i * ent_sz + ent_sz];
                                if guid_str(&e[0..16]) == "00000000-0000-0000-0000-000000000000" { continue; }
                                let first = u64le(e, 32);
                                let lastl = u64le(e, 40);
                                let mut name = String::new();
                                for c in e[56..128].chunks(2) {
                                    let v = u16::from_le_bytes([c[0], c[1]]);
                                    if v == 0 { break; }
                                    name.push(char::from_u32(v as u32).unwrap_or('?'));
                                }
                                // Lecture du debut de la partition pour identifier le systeme
                                let (kind, hash) = match read_lbas(&mut bio, mid, bs, first, 8) {
                                    Ok(p) => {
                                        let k = fs_kind(&p);
                                        let mut big = Vec::new();
                                        let chunks = (SECTOR_MAX_READ / bs).min((lastl - first + 1) as usize);
                                        if let Ok(b) = read_lbas(&mut bio, mid, bs, first, chunks) { big = b; }
                                        (k, fnv1a(&big))
                                    }
                                    Err(_) => ("lecture echouee", 0),
                                };
                                r.line(&format!(
                                    "    part {:>2} type {} | LBA {}..{} ({} Mo) | nom '{}' | systeme {} | fnv1M={:016x}",
                                    i, guid_str(&e[0..16]), first, lastl, (lastl - first + 1) as u64 * bs as u64 / (1024 * 1024), name, kind, hash
                                ));
                            }
                        }
                    } else {
                        r.line("    pas de GPT (MBR ou disque nu)");
                    }
                }
                Err(s) => r.line(&format!("    lecture LBA0 echouee : {:?}", s)),
            }
            // Empreinte SHA-256 du disque ENTIER : differee apres la phase 18 (un blocage ici
            // ne doit pas empecher les autres sondes). Desactivee par defaut (FULL_DISK_HASH).
            if FULL_DISK_HASH {
                hash_differe.push((disk_idx, h));
            }
            // Empreinte d'ECHANTILLON : 64 Mo au debut et 64 Mo a la fin (rapide, ne lit pas tout le disque)
            if !FULL_DISK_HASH {
                let win = ((64usize << 20) / bs).max(1) as u64;
                let total = last + 1;
                let chunk_blocks = ((1usize << 20) / bs).max(1) as u64;
                let mut hasher = Sha256::new();
                let mut ok = true;
                let mut ranges: [(u64, u64); 2] = [(0, win.min(total)), (total.saturating_sub(win).max(win.min(total)), total)];
                if ranges[1].0 >= ranges[1].1 { ranges[1] = (0, 0); }
                'outer: for (a, b) in ranges {
                    let mut lba = a;
                    while lba < b {
                        let n = chunk_blocks.min(b - lba) as usize;
                        let mut buf = alloc::vec![0u8; n * bs];
                        if bio.read_blocks(mid, lba, &mut buf).is_err() {
                            r.line(&format!("    ERREUR lecture echantillon au LBA {}", lba));
                            ok = false;
                            break 'outer;
                        }
                        hasher.update(&buf);
                        lba += n as u64;
                    }
                }
                if ok {
                    let d = hasher.finalize();
                    let mut hex = String::new();
                    for b in d.iter() {
                        let _ = write!(hex, "{:02x}", b);
                    }
                    r.line(&format!("    SHA-256 echantillon (debut+fin 64 Mo) : {}", hex));
                }
            }
            // GPT de secours (fin du disque)
            if last >= 33 {
                if let Ok(tail) = read_lbas(&mut bio, mid, bs, last - 33, 34) {
                    r.file(&format!("disque{}_fin_lba-33.bin", disk_idx), &tail);
                    r.line(&format!("    fin du disque : fnv={:016x}", fnv1a(&tail)));
                }
            }
        } else {
            match read_lbas(&mut bio, mid, bs, 0, 8) {
                Ok(p) => r.line(&format!("    partition : systeme {}", fs_kind(&p))),
                Err(s) => r.line(&format!("    lecture echouee : {:?}", s)),
            }
        }
        disk_idx += 1;
        r.save();
    }

    // ---------- Phase 6 : peripheriques par protocole ----------
    r.line("\n[phase 6] Comptage des protocoles principaux");
    if let Ok(v) = boot::find_handles::<uefi::proto::media::fs::SimpleFileSystem>() {
        r.line(&format!("  SimpleFileSystem : {}", v.len()));
    }
    if let Ok(v) = boot::find_handles::<uefi::proto::console::gop::GraphicsOutput>() {
        r.line(&format!("  GraphicsOutput : {}", v.len()));
    }
    if let Ok(v) = boot::find_handles::<uefi::proto::pci::root_bridge::PciRootBridgeIo>() {
        r.line(&format!("  PciRootBridgeIo : {}", v.len()));
    }

    // ---------- Phase 7 : configuration PCI de toutes les fonctions ----------
    r.line("\n[phase 7] PCI : configuration 256 octets de chaque fonction");
    let mut pci_dump: Vec<u8> = Vec::new();
    let mut pci_ext: Vec<u8> = Vec::new();
    if let Ok(hs) = boot::find_handles::<PciRootBridgeIo>() {
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<PciRootBridgeIo>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(mut rb) = p {
                match rb.enumerate() {
                    Ok(tree) => {
                        let addrs: Vec<_> = tree.iter().copied().collect();
                        r.line(&format!("  root bridge : {} fonctions", addrs.len()));
                        for a in addrs {
                            let mut cfg = [0u8; 256];
                            if rb.pci().read(a.with_register(0), &mut cfg).is_ok() {
                                let vid = u16::from_le_bytes([cfg[0], cfg[1]]);
                                let did = u16::from_le_bytes([cfg[2], cfg[3]]);
                                r.line(&format!(
                                    "  {:02x}:{:02x}.{} {:04x}:{:04x} classe {:02x}/{:02x}/{:02x} rev {:02x}",
                                    a.bus, a.dev, a.fun, vid, did, cfg[11], cfg[10], cfg[9], cfg[8]
                                ));
                                pci_dump.extend_from_slice(&[a.bus, a.dev, a.fun, 0]);
                                pci_dump.extend_from_slice(&cfg);
                                // Configuration etendue (4096 octets, PCIe)
                                let mut ext = [0u8; 4096];
                                if rb.pci().read(a.with_extended_register(0), &mut ext).is_ok() {
                                    pci_ext.extend_from_slice(&[a.bus, a.dev, a.fun, 0]);
                                    pci_ext.extend_from_slice(&ext);
                                }
                                if let Some(note) = lectures::note_peripherique(vid, did, cfg[11], cfg[10], cfg[9]) {
                                    r.line(&note);
                                }
                                decrire_pci(&cfg, &ext, a.bus, a.dev, a.fun, &mut r);
                            }
                        }
                    }
                    Err(e) => r.line(&format!("  enumeration echec : {:?}", e.status())),
                }
            }
        }
    }
    r.file("PCI_cfg_all.bin", &pci_dump);
    r.file("PCI_cfg_etendue_4k.bin", &pci_ext);
    r.save();

    // ---------- Phase 8 : inventaire de TOUS les protocoles par handle ----------
    r.line("\n[phase 8] Inventaire des protocoles (tous les handles)");
    let mut inv = String::new();
    let mut nproto = 0usize;
    let mut reconnus: Vec<&'static str> = Vec::new();
    if let Ok(hs) = boot::locate_handle_buffer(boot::SearchType::AllHandles) {
        r.line(&format!("  handles : {}", hs.len()));
        for h in hs.iter() {
            let _ = writeln!(inv, "handle {:p}", h.as_ptr());
            if let Ok(ps) = boot::protocols_per_handle(*h) {
                for g in ps.iter() {
                    // SAFETY: uefi::Guid est repr(C) de 16 octets.
                    let gb: [u8; 16] = unsafe { core::mem::transmute(**g) };
                    let gs = guid_str(&gb);
                    let _ = writeln!(inv, "   {}", gs);
                    if let Some(nom) = lectures::nom_guid(&gs) {
                        if !reconnus.contains(&nom) {
                            reconnus.push(nom);
                        }
                    }
                    nproto += 1;
                }
            }
        }
    }
    r.line(&format!("  couples handle/protocole : {}", nproto));
    // Protocoles du firmware reconnus d'apres UEFI PI (SMM, S3, SPI, I2C, SMBus, PCI...) : presence seulement.
    r.line(&format!("  protocoles firmware reconnus (UEFI PI) : {}", reconnus.len()));
    for nom in &reconnus {
        r.line(&format!("    present : {}", nom));
    }
    r.file("PROTOCOLES_tous_handles.txt", inv.as_bytes());
    r.save();

    // ---------- Phase 9 : NVRAM complete (toutes les variables, donnees brutes) ----------
    r.line("\n[phase 9] NVRAM : toutes les variables (donnees brutes)");
    let mut nv: Vec<u8> = Vec::new();
    let mut nvcount = 0usize;
    let mut nverr = 0usize;
    let mut sb_state: Option<u8> = None;
    let mut setup_mode: Option<u8> = None;
    let mut boot_current: Option<u16> = None;
    let mut boot_order: Vec<u16> = Vec::new();
    let mut boot_entries: Vec<String> = Vec::new();
    for k in uefi::runtime::variable_keys() {
        let Ok(k) = k else { nverr += 1; continue; };
        let mut buf = alloc::vec![0u8; 1 << 16];
        match uefi::runtime::get_variable(&k.name, &k.vendor, &mut buf) {
            Ok((data, attr)) => {
                let nom = format!("{}", k.name);
                match nom.as_str() {
                    "SecureBoot" => sb_state = data.first().copied(),
                    "SetupMode" => setup_mode = data.first().copied(),
                    "BootCurrent" if data.len() >= 2 => boot_current = Some(u16::from_le_bytes([data[0], data[1]])),
                    "BootOrder" => boot_order = data.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect(),
                    _ if nom.starts_with("Boot") && nom.len() == 8 && nom[4..].chars().all(|c| c.is_ascii_hexdigit()) => {
                        // Description UTF-16 de l'entree de demarrage (apres 6 octets d'attributs)
                        let desc: String = data.get(6..).unwrap_or(&[])
                            .chunks_exact(2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .take_while(|&v| v != 0)
                            .map(|v| char::from_u32(v as u32).unwrap_or('?'))
                            .collect();
                        boot_entries.push(format!("{} = {}", nom, desc));
                    }
                    _ => {}
                }
                // SAFETY: uefi::Guid est repr(C) de 16 octets.
                let vg: [u8; 16] = unsafe { core::mem::transmute(k.vendor.0) };
                let nm: Vec<u16> = k.name.iter().map(|c| u16::from(*c)).collect();
                nv.extend_from_slice(&vg);
                nv.extend_from_slice(&attr.bits().to_le_bytes());
                nv.extend_from_slice(&(nm.len() as u32).to_le_bytes());
                for c in &nm { nv.extend_from_slice(&c.to_le_bytes()); }
                nv.extend_from_slice(&(data.len() as u32).to_le_bytes());
                nv.extend_from_slice(data);
                nvcount += 1;
            }
            Err(_) => nverr += 1,
        }
    }
    r.line(&format!("  variables lues : {} | erreurs : {}", nvcount, nverr));
    r.line(&format!("  Secure Boot : {} | SetupMode : {}",
        match sb_state { Some(1) => "ACTIF", Some(0) => "INACTIF", _ => "non indique" },
        match setup_mode { Some(1) => "oui", Some(0) => "non", _ => "non indique" }));
    r.line(&format!("  demarrage courant : {}", boot_current.map(|v| format!("Boot{:04X}", v)).unwrap_or_else(|| "non indique".into())));
    r.line(&format!("  ordre de demarrage : {}", boot_order.iter().map(|v| format!("{:04X}", v)).collect::<Vec<_>>().join(" ")));
    for e in &boot_entries {
        r.line(&format!("    {}", e));
    }
    r.file("NVRAM_toutes_variables.bin", &nv);
    r.save();

    // ---------- Phase 10 : fenetre firmware physique 0xFF000000-0xFFFFFFFF ----------
    if FLASH_WINDOW {
    r.line("ETAPE 10 debut : lecture fenetre firmware (peut bloquer)");
    r.save();
    r.line("\n[phase 10] Fenetre firmware physique 0xFF000000-0xFFFFFFFF (16 Mo, lecture)");
    // SAFETY: fenetre de la puce SPI mappee par le chipset, lecture seule en pre-boot.
    let fw = unsafe { core::slice::from_raw_parts(0xFF00_0000usize as *const u8, 16 << 20) };
    r.line(&format!("  FNV-64 de la fenetre : {:016x}", fnv1a(fw)));
    r.file("FLASH_fenetre_16M.bin", fw);
    // Comparaison automatique avec l'image officielle placee sur la cle :
    // \EFI\HWA2\REF\M1603QAAS.308 = en-tete 2048 o + image 16 Mo (la fenetre = image[0x800..])
    let refpath = p(&format!("{}REF\\M1603QAAS.308", OUT));
    let refdata = r.fs.as_mut().and_then(|fs| fs.read(uefi::fs::Path::new(&refpath)).ok());
    match refdata {
        Some(rf) if rf.len() >= 0x800 + fw.len() => {
            let img = &rf[0x800..0x800 + fw.len()];
            let mut bad_blocks = 0usize;
            let mut report = String::new();
            for (i, (blk_a, blk_b)) in fw.chunks(4096).zip(img.chunks(4096)).enumerate() {
                if blk_a != blk_b {
                    bad_blocks += 1;
                    let nd = blk_a.iter().zip(blk_b).filter(|(x, y)| x != y).count();
                    let _ = writeln!(report, "{:#010x} {} octets differents sur 4096", i * 4096, nd);
                }
            }
            r.line(&format!("  REFERENCE OFFICIELLE : {} blocs de 4 Ko differents sur {}", bad_blocks, fw.len() / 4096));
            r.file("FLASH_vs_officiel_blocs.txt", report.as_bytes());
        }
        _ => r.line("  REFERENCE OFFICIELLE absente (copier M1603QAAS.308 dans \\EFI\\HWA2\\REF\\)"),
    }
    r.save();
    } else {
        r.line("  fenetre firmware : ignoree (FLASH_WINDOW = false)");
    }

    // ---------- Phase 11 : SMBIOS brut ----------
    r.line("\n[phase 11] SMBIOS brut");
    let sm3 = tables.iter().find(|(g, _)| g == "f2fd1544-9794-4a2c-992e-e5bbcf20e394").map(|t| t.1);
    let sm2 = tables.iter().find(|(g, _)| g == "eb9d2d31-2d88-11d3-9a16-0090273fc14d").map(|t| t.1);
    // SAFETY: adresses fournies par la table de configuration, lecture seule.
    unsafe {
        if let Some(ep) = sm3 {
            let e = core::slice::from_raw_parts(ep as *const u8, 0x18);
            let tl = u32le(e, 0x0C) as usize;
            let ta = u64le(e, 0x10) as usize;
            r.line(&format!("  SMBIOS 3 : table {} octets @ {:#x}", tl, ta));
            r.file("SMBIOS3_entree.bin", e);
            r.file("SMBIOS3_table.bin", core::slice::from_raw_parts(ta as *const u8, tl));
            decrire_smbios(core::slice::from_raw_parts(ta as *const u8, tl), &mut r);
        } else if let Some(ep) = sm2 {
            let e = core::slice::from_raw_parts(ep as *const u8, 0x1F);
            let tl = u16::from_le_bytes([e[0x16], e[0x17]]) as usize;
            let ta = u32le(e, 0x18) as usize;
            r.line(&format!("  SMBIOS 2 : table {} octets @ {:#x}", tl, ta));
            r.file("SMBIOS2_entree.bin", e);
            r.file("SMBIOS2_table.bin", core::slice::from_raw_parts(ta as *const u8, tl));
            decrire_smbios(core::slice::from_raw_parts(ta as *const u8, tl), &mut r);
        } else {
            r.line("  SMBIOS introuvable");
        }
    }
    r.save();

    // ---------- Phase 12 : TPM 2.0 (TCG2) : journal des mesures de demarrage ----------
    r.line("\n[phase 12] TPM 2.0 : journal des mesures de demarrage (lecture seule)");
    let mut tlog = String::new();
    if let Ok(hs) = boot::find_handles::<uefi::proto::tcg::v2::Tcg>() {
        r.line(&format!("  protocoles TCG2 : {}", hs.len()));
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<uefi::proto::tcg::v2::Tcg>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(mut t) = p {
                match t.get_active_pcr_banks() {
                    Ok(b) => r.line(&format!("  banques PCR actives : {:?}", b)),
                    Err(e) => r.line(&format!("  banques PCR : echec {:?}", e.status())),
                }
                match t.get_event_log_v2() {
                    Ok(log) => {
                        let mut n = 0usize;
                        for ev in log.iter() {
                            let _ = writeln!(tlog, "{:?}", ev);
                            n += 1;
                        }
                        let _ = writeln!(tlog, "--- {:?}", log);
                        r.line(&format!("  evenements du journal : {}", n));
                    }
                    Err(e) => r.line(&format!("  journal v2 : echec {:?}", e.status())),
                }
            }
        }
    } else {
        r.line("  aucun TPM 2.0 (protocole TCG2) detecte");
    }
    r.file("TPM_journal_mesures.txt", tlog.as_bytes());
    r.save();

    // ---------- Phase 13 : NVMe (Identify controleur + journal SMART), lecture seule ----------
    r.line("ETAPE 13 debut : NVMe (peut bloquer)");
    r.save();
    r.line("\n[phase 13] NVMe : identification et SMART (commandes admin en lecture)");
    if let Ok(hs) = boot::find_handles::<uefi::proto::nvme::pass_thru::NvmePassThru>() {
        r.line(&format!("  controleurs NVMe : {}", hs.len()));
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<uefi::proto::nvme::pass_thru::NvmePassThru>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            let Ok(pt) = p else { continue; };
            let align = pt.io_align();
            // Identify Controller : opcode 0x06, CNS=1 (cdw10=1), 4096 octets
            let mut ctrl = pt.controller();
            let req = uefi::proto::nvme::NvmeRequestBuilder::new(align, 0x06, uefi_raw::protocol::nvme::NvmExpressQueueType::ADMIN)
                .with_cdw10(1)
                .with_transfer_buffer(4096);
            match req {
                Ok(b) => match ctrl.execute_command(b.build()) {
                    Ok(resp) => {
                        if let Some(d) = resp.transfer_buffer() {
                            let sn = core::str::from_utf8(&d[4..24]).unwrap_or("?").trim();
                            let mn = core::str::from_utf8(&d[24..64]).unwrap_or("?").trim();
                            let fw = core::str::from_utf8(&d[64..72]).unwrap_or("?").trim();
                            // Regle du LISEZ-MOI : les numeros de serie ne sont pas ecrits dans le rapport.
                            r.line(&format!("  Identify : modele '{}' | serie non ecrite ({} caracteres) | firmware '{}'", mn, sn.len(), fw));
                            r.file("NVMe_identify_controleur.bin", d);
                        }
                    }
                    Err(e) => r.line(&format!("  Identify echec : {:?}", e.status())),
                },
                Err(_) => r.line("  Identify : buffer non alloue"),
            }
            // Identify Namespace (CNS=0) pour chaque namespace du controleur
            for ns in pt.iter_namespaces() {
                let nsid = ns.namespace_id();
                let mut nsm = pt.iter_namespaces().find(|x| x.namespace_id() == nsid);
                let Some(ref mut nsref) = nsm else { continue; };
                let req3 = uefi::proto::nvme::NvmeRequestBuilder::new(align, 0x06, uefi_raw::protocol::nvme::NvmExpressQueueType::ADMIN)
                    .with_cdw10(0)
                    .with_transfer_buffer(4096);
                match req3 {
                    Ok(b) => match nsref.execute_command(b.build()) {
                        Ok(resp) => {
                            if let Some(d) = resp.transfer_buffer() {
                                let nsze = u64::from_le_bytes(d[0..8].try_into().unwrap_or([0; 8]));
                                let ncap = u64::from_le_bytes(d[8..16].try_into().unwrap_or([0; 8]));
                                r.line(&format!("  namespace {:?} : taille {} blocs | capacite {} blocs", nsid, nsze, ncap));
                                r.file(&format!("NVMe_identify_ns_{:?}.bin", nsid), d);
                            }
                        }
                        Err(e) => r.line(&format!("  namespace {:?} : echec {:?}", nsid, e.status())),
                    },
                    Err(_) => {}
                }
            }
            // Get Log Page SMART/Health (LID 0x02, 512 octets), NSID broadcast
            let mut bc = pt.broadcast();
            let req2 = uefi::proto::nvme::NvmeRequestBuilder::new(align, 0x02, uefi_raw::protocol::nvme::NvmExpressQueueType::ADMIN)
                .with_cdw10((127u32 << 16) | 0x02)
                .with_transfer_buffer(512);
            match req2 {
                Ok(b) => match bc.execute_command(b.build()) {
                    Ok(resp) => {
                        if let Some(d) = resp.transfer_buffer() {
                            let q = |o: usize| u64::from_le_bytes(d[o..o + 8].try_into().unwrap_or([0; 8]));
                            r.line(&format!("  SMART : temperature {} C | heures de marche {} | cycles d'alimentation {} | arrets brutaux {}",
                                u16::from_le_bytes([d[1], d[2]]).saturating_sub(273), q(128), q(112), q(144)));
                            r.line(&format!("  SMART : erreurs media {} | entrees journal erreurs {} | donnees lues (unites 512000 o) {} | ecrites {}",
                                q(160), q(176), q(32), q(48)));
                            r.line(&format!("  SMART : avertissement critique {:#04x} | spare {} %", d[0], d[3]));
                            r.file("NVMe_SMART.bin", d);
                        }
                    }
                    Err(e) => r.line(&format!("  SMART echec : {:?}", e.status())),
                },
                Err(_) => r.line("  SMART : buffer non alloue"),
            }
        }
    } else {
        r.line("  aucun controleur NVMe (protocole NvmePassThru) detecte");
    }
    r.save();

    // ---------- Phase 14 : USB, descripteurs de chaque peripherique ----------
    r.line("ETAPE 14 debut : USB (peut bloquer)");
    r.save();
    r.line("\n[phase 14] USB : descripteurs de chaque peripherique (lecture seule)");
    if let Ok(hs) = boot::find_handles::<uefi::proto::usb::io::UsbIo>() {
        r.line(&format!("  interfaces USB : {}", hs.len()));
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<uefi::proto::usb::io::UsbIo>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(mut u) = p {
                match u.device_descriptor() {
                    Ok(d) => r.line(&format!("  dispositif : {:?}", d)),
                    Err(e) => r.line(&format!("  descripteur dispositif : echec {:?}", e.status())),
                }
                match u.interface_descriptor() {
                    Ok(d) => r.line(&format!("    interface : {:?}", d)),
                    Err(e) => r.line(&format!("    interface : echec {:?}", e.status())),
                }
                // Descripteur de configuration complet, octet par octet (bas niveau).
                match config_brute(&mut u) {
                    Some(c) => {
                        r.line(&format!("    config brute : {} octets", c.len()));
                        for morceau in c.chunks(16) {
                            let hex: alloc::vec::Vec<alloc::string::String> =
                                morceau.iter().map(|b| format!("{:02x}", b)).collect();
                            r.line(&format!("      {}", hex.join(" ")));
                        }
                    }
                    None => r.line("    config brute : lecture refusee"),
                }
                // Chaines (fabricant, produit, serie) : indices 1 a 3, langue 0 si dispo
                let lang = u.supported_languages().ok().and_then(|l| l.first().copied()).unwrap_or(0x0409);
                for idx in 1u8..=3 {
                    match u.string_descriptor(lang, idx) {
                        Ok(s) => r.line(&format!("    chaine {} : {}", idx, &*s)),
                        Err(_) => {}
                    }
                }
            }
        }
    } else {
        r.line("  aucun peripherique USB (protocole UsbIo) detecte");
    }
    r.save();

    // ---------- Phase 15b : registres BAR0 de chaque peripherique PCI (lecture seule, 256 o) ----------
    // Donnees brutes pour ecrire les pilotes (NVMe, xHCI, HDA, Wi-Fi...). Aucune ecriture.
    r.line("ETAPE 15b debut : registres BAR0 (peut bloquer)");
    r.save();
    r.line("\n[phase 15b] Registres BAR0 de chaque peripherique PCI (256 octets, lecture)");
    let mut bar_dump: Vec<u8> = Vec::new();
    if BAR_READS { if let Ok(hs) = boot::find_handles::<PciRootBridgeIo>() {
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<PciRootBridgeIo>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(mut rb) = p {
                if let Ok(tree) = rb.enumerate() {
                    let addrs: Vec<_> = tree.iter().copied().collect();
                    for a in addrs {
                        let mut cfg = [0u8; 64];
                        if rb.pci().read(a.with_register(0), &mut cfg).is_err() { continue; }
                        let bar0 = u32::from_le_bytes([cfg[16], cfg[17], cfg[18], cfg[19]]);
                        if bar0 & 1 == 1 || bar0 == 0 || bar0 == 0xFFFF_FFFF { continue; }
                        let mut base = (bar0 & !0xF) as u64;
                        if (bar0 >> 1) & 3 == 2 {
                            let hi = u32::from_le_bytes([cfg[20], cfg[21], cfg[22], cfg[23]]);
                            base |= (hi as u64) << 32;
                        }
                        let mut regs = [0u8; 256];
                        match rb.memory().read::<u8>(base, &mut regs) {
                            Ok(()) => {
                                r.line(&format!(
                                    "  {:02x}:{:02x}.{} BAR0={:#x} : {:02x?}",
                                    a.bus, a.dev, a.fun, base, &regs[0..16]
                                ));
                                bar_dump.extend_from_slice(&[a.bus, a.dev, a.fun, 0]);
                                bar_dump.extend_from_slice(&base.to_le_bytes());
                                bar_dump.extend_from_slice(&regs);
                            }
                            Err(e) => r.line(&format!("  {:02x}:{:02x}.{} BAR0={:#x} : lecture refusee {:?}", a.bus, a.dev, a.fun, base, e.status())),
                        }
                    }
                }
            }
        }
    }
    }
    r.file("PCI_BAR0_registres.bin", &bar_dump);
    r.save();

    // ---------- Phase 15a : EDID de l'ecran (lecture seule) ----------
    r.line("\n[phase 15a] EDID de l'ecran");
    if let Ok(hs) = boot::find_handles::<uefi::proto::console::gop::EdidDiscovered>() {
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<uefi::proto::console::gop::EdidDiscovered>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(e) = p {
                if let Some(d) = e.edid() {
                    let manu = u16::from_be_bytes([d[8], d[9]]);
                    let c = |v: u16| [((v >> 10) & 31) as u8 + 64, ((v >> 5) & 31) as u8 + 64, (v & 31) as u8 + 64];
                    let m = c(manu);
                    r.line(&format!(
                        "  EDID {} octets | fabricant {}{}{} | produit {:04x} | version {}.{} | semaine {} annee {}",
                        d.len(), m[0] as char, m[1] as char, m[2] as char,
                        u16::from_le_bytes([d[10], d[11]]), d[18], d[19], d[16], 1990u32 + d[17] as u32
                    ));
                    r.file("EDID_ecran.bin", d);
                }
            }
        }
    } else {
        r.line("  aucun EDID detecte");
    }
    r.save();

    // ---------- Phase 15 : affichage GOP, modes disponibles ----------
    r.line("\n[phase 15] Affichage : modes GOP disponibles");
    if let Ok(hs) = boot::find_handles::<uefi::proto::console::gop::GraphicsOutput>() {
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<uefi::proto::console::gop::GraphicsOutput>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(g) = p {
                let mut n = 0usize;
                for m in g.modes() {
                    r.line(&format!("  mode {} : {:?}", n, m.info()));
                    n += 1;
                }
                r.line(&format!("  modes : {}", n));
            }
        }
    }
    r.save();

    // ---------- Phase 16 : contenu des systemes de fichiers de chaque partition ----------
    r.line("\n[phase 16] Fichiers de chaque partition (lecture seule)");
    let mut listing = String::new();
    if let Ok(hs) = boot::find_handles::<uefi::proto::media::fs::SimpleFileSystem>() {
        r.line(&format!("  systemes de fichiers : {}", hs.len()));
        for (i, h) in hs.iter().enumerate() {
            // SAFETY: GetProtocol, lecture seule.
            let p2 = unsafe {
                boot::open_protocol::<uefi::proto::media::fs::SimpleFileSystem>(
                    OpenProtocolParams { handle: *h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(sfs) = p2 {
                let mut f = FileSystem::new(sfs);
                let mut cnt = 0usize;
                let _ = writeln!(listing, "== systeme de fichiers {}", i);
                list_dir(&mut f, "\\", 0, &mut listing, &mut cnt);
                r.line(&format!("  systeme {} : {} entrees", i, cnt));
            }
        }
    }
    r.file("FS_listing_partitions.txt", listing.as_bytes());
    r.save();

    // ---------- Phase 17 : CPUID complet et registres de controle (CPL0, lecture seule) ----------
    r.line("ETAPE 17 debut : CPUID et registres de controle");
    r.save();
    r.line("\n[phase 17] CPUID complet et registres de controle");
    let mut cpuid_dump: Vec<u8> = Vec::new();
    let max_std = core::arch::x86_64::__cpuid(0).eax;
    let max_ext = core::arch::x86_64::__cpuid(0x8000_0000).eax;
    r.line(&format!("  feuilles standard max {:#x}, etendues max {:#x}", max_std, max_ext));
    // Famille / modele AMD (CPUID 1 EAX, AMD APM vol.3 Appendix E, Fn0000_0001_EAX) :
    // famille = base[11:8] + ext[27:20] si base = F ; modele = (ext[19:16] << 4) | base[7:4] si base = F or 6.
    {
        let eax = core::arch::x86_64::__cpuid(1).eax;
        let base_fam = (eax >> 8) & 0xF;
        let ext_fam = (eax >> 20) & 0xFF;
        let base_mod = (eax >> 4) & 0xF;
        let ext_mod = (eax >> 16) & 0xF;
        let fam = if base_fam == 0xF { base_fam + ext_fam } else { base_fam };
        let modl = if base_fam == 0xF || base_fam == 6 { (ext_mod << 4) | base_mod } else { base_mod };
        let stepping = eax & 0xF;
        r.line(&format!(
            "  famille {:#x} modele {:#x} stepping {:#x} (CPUID 1 EAX = {:#010x})",
            fam, modl, stepping, eax
        ));
        // Reference documentaire : la PPR presente dans <DOSSIER_SPECS_PDF> est "AMD Family 19h Model 51h A1".
        if fam == 0x19 && modl == 0x51 {
            r.line("  PPR de reference : Family 19h Model 51h A1 (AMD 56569) : CORRESPOND");
        } else {
            r.line("  PPR de reference : Family 19h Model 51h A1 (AMD 56569) : NE CORRESPOND PAS, verifier la PPR");
        }
    }
    // Sonde CPU complète (CPUID seul, aucun MSR) : bits de fonctionnalités, caches, topologie, XCR0.
    for l in cpu_sonde::lignes() {
        r.line(&l);
    }
    let subleaf_leaves = [0x4u32, 0x7, 0xB, 0xD, 0xF, 0x10, 0x12, 0x14, 0x17, 0x18];
    for leaf in 0..=max_std.min(0x1F) {
        let subs = if subleaf_leaves.contains(&leaf) { 8u32 } else { 1 };
        for sub in 0..subs {
            let c = core::arch::x86_64::__cpuid_count(leaf, sub);
            let mut rec = [0u8; 20];
            rec[0..4].copy_from_slice(&leaf.to_le_bytes());
            rec[4..8].copy_from_slice(&sub.to_le_bytes());
            rec[8..12].copy_from_slice(&c.eax.to_le_bytes());
            rec[12..16].copy_from_slice(&c.ebx.to_le_bytes());
            rec[16..20].copy_from_slice(&c.ecx.to_le_bytes());
            cpuid_dump.extend_from_slice(&rec);
            cpuid_dump.extend_from_slice(&c.edx.to_le_bytes());
        }
    }
    for leaf in 0x8000_0000..=max_ext.min(0x8000_0030) {
        let c = core::arch::x86_64::__cpuid(leaf);
        cpuid_dump.extend_from_slice(&leaf.to_le_bytes());
        cpuid_dump.extend_from_slice(&c.eax.to_le_bytes());
        cpuid_dump.extend_from_slice(&c.ebx.to_le_bytes());
        cpuid_dump.extend_from_slice(&c.ecx.to_le_bytes());
        cpuid_dump.extend_from_slice(&c.edx.to_le_bytes());
    }
    r.line(&format!("  octets CPUID : {}", cpuid_dump.len()));
    r.file("CPUID_complet.bin", &cpuid_dump);
    for n in [0u8, 3, 4] {
        r.line(&format!("  CR{} = {:#018x}", n, read_cr(n)));
    }
    // Décodage des bits (APM vol. 2 §3.1.1 et §3.1.3)
    for l in cpu_sonde::decrire_cr0(read_cr(0)) { r.line(&l); }
    for l in cpu_sonde::decrire_cr4(read_cr(4)) { r.line(&l); }
    r.save();

    // MSR : liste connue, journal ecrit AVANT chaque lecture. Un MSR absent sur ce processeur
    // (#GP) est intercepte par l'IDT : il est note "ABSENT" et l'analyse continue.
    let msrs: [u32; 23] = [
        0x1B, 0x200, 0x201, 0x202, 0x203, 0x204, 0x205, 0x206, 0x207,
        0x277, 0x2FF, 0xC000_0080, 0xC001_0010, 0xC001_0015, 0xC001_0111,
        0xC001_0112, 0xC001_0113, 0xC001_0114, 0xC001_001F, 0x250, 0x258, 0x259, 0x26F,
        // 0x1A0 (IA32_MISC_ENABLE, MSR Intel) n'est PLUS lu : sur ce PC AMD sa lecture a bloque la
        // machine deux fois (rapports du 2026-11-11 23:24 et du 2026-11-12 09:07, arretes sur cette
        // ligne) alors que la protection #GP etait deja en place. Tant qu'il etait lu, le hachage du
        // disque et toute la phase R0 (USB, clavier, NVMe, audio) n'etaient jamais atteints.
    ];
    let mut msr_dump: Vec<u8> = Vec::new();
    for m in msrs.into_iter().filter(|_| MSR_READS) {
        let nom = msr_name(m);
        r.line(&format!("  lecture MSR {:#010x} {} ...", m, nom));
        r.save();
        // SAFETY: lecture MSR protegee (voir rdmsr).
        match unsafe { rdmsr(m) } {
            Some(v) => {
                r.line(&format!("  MSR {:#010x} {} = {:#018x}", m, nom, v));
                msr_dump.extend_from_slice(&m.to_le_bytes());
                msr_dump.extend_from_slice(&v.to_le_bytes());
            }
            None => r.line(&format!("  MSR {:#010x} {} = ABSENT (#GP, non defini sur ce processeur)", m, nom)),
        }
    }
    r.file("MSR_configuration.bin", &msr_dump);
    r.save();
    r.save();

    // Hachages complets differes (FULL_DISK_HASH) : apres toutes les sondes, avant la sortie des services UEFI.
    // Hachage du disque a l'etape 18 : supprime a la demande (aucun affichage a cette etape).
    hash_differe.clear();

    // ---------- Etape R0 (seulement si R0_EBS = true) : sortie des services UEFI + pilotes materiel ----------
    {
    // Tout ce qui allouerait de la memoire est fait AVANT ExitBootServices.
    let mut fbp: usize = 0;
    let mut fbsz: usize = 0;
    let mut stride: usize = 0;
    if let Ok(hgop) = boot::get_handle_for_protocol::<uefi::proto::console::gop::GraphicsOutput>() {
        // SAFETY: GetProtocol, ecriture ulterieure seulement dans le framebuffer.
        let go = unsafe {
            boot::open_protocol::<uefi::proto::console::gop::GraphicsOutput>(
                OpenProtocolParams { handle: hgop, agent: boot::image_handle(), controller: None },
                OpenProtocolAttributes::GetProtocol,
            )
        };
        if let Ok(mut g) = go {
            stride = g.current_mode_info().stride();
            fbp = g.frame_buffer().as_mut_ptr() as usize;
            fbsz = g.frame_buffer().size();
        }
    }
    r.line("\n[phase R0] sortie des services UEFI et pilotes materiel (xHCI, HDA, NVMe, clavier)");
    r.save();
    let resume: [alloc::string::String; 7] = [
        "HWA2 R0 OK".into(),
        format!("CR0 {:016X}", read_cr(0)),
        format!("CR3 {:016X}", read_cr(3)),
        format!("CR4 {:016X}", read_cr(4)),
        format!("MSR {} CPUID {}", msrs.len(), cpuid_dump.len()),
        "FIN RAPPORT SUR CLE".into(),
        "R0 ACTIF : SORTIE AUTO, PC REDEMARRE".into(),
    ];
    let mut lines = [[b' '; 40]; 7];
    let mut llen = [0usize; 7];
    for (i, s) in resume.iter().enumerate() {
        let b = s.as_bytes();
        let n = b.len().min(40);
        lines[i][..n].copy_from_slice(&b[..n]);
        llen[i] = n;
    }
    // Adresse BAR0 du controleur xHCI (classe 0C/03/30), trouvee avant la sortie des services
    let mut xhci_bar: u64 = 0;
    let mut xhci_list = [0u64; 4];
    let mut xhci_n = 0usize;
    if let Ok(hs) = boot::find_handles::<PciRootBridgeIo>() {
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<PciRootBridgeIo>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(mut rb) = p {
                if let Ok(tree) = rb.enumerate() {
                    for a in tree.iter().copied() {
                        let mut cfg = [0u8; 64];
                        if rb.pci().read(a.with_register(0), &mut cfg).is_err() { continue; }
                        if cfg[11] == 0x0C && cfg[10] == 0x03 && cfg[9] == 0x30 {
                            let bar0 = u32::from_le_bytes([cfg[16], cfg[17], cfg[18], cfg[19]]);
                            let mut base = (bar0 & !0xF) as u64;
                            if (bar0 >> 1) & 3 == 2 {
                                let hi = u32::from_le_bytes([cfg[20], cfg[21], cfg[22], cfg[23]]);
                                base |= (hi as u64) << 32;
                            }
                            if base != 0 && xhci_n < 4 {
                                xhci_list[xhci_n] = base;
                                xhci_n += 1;
                            }
                            if xhci_bar == 0 && base != 0 {
                                xhci_bar = base;
                            }
                        }
                    }
                }
            }
        }
    }
    // Controleur NVMe (classe 01/08/02) : BAR0 (64 bits si besoin), avant la sortie des services
    let mut nvme_bar: u64 = 0;
    if let Ok(hs) = boot::find_handles::<PciRootBridgeIo>() {
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<PciRootBridgeIo>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(mut rb) = p {
                if let Ok(tree) = rb.enumerate() {
                    for a in tree.iter().copied() {
                        let mut cfg = [0u8; 64];
                        if rb.pci().read(a.with_register(0), &mut cfg).is_err() { continue; }
                        if cfg[11] == 0x01 && cfg[10] == 0x08 && cfg[9] == 0x02 && nvme_bar == 0 {
                            let bar0 = u32::from_le_bytes([cfg[16], cfg[17], cfg[18], cfg[19]]);
                            let mut base = (bar0 & !0xF) as u64;
                            if (bar0 >> 1) & 3 == 2 {
                                let hi = u32::from_le_bytes([cfg[20], cfg[21], cfg[22], cfg[23]]);
                                base |= (hi as u64) << 32;
                            }
                            nvme_bar = base;
                        }
                    }
                }
            }
        }
    }
    // Choix du controleur xHCI : le premier qui a un peripherique connecte (sinon le premier trouve)
    for k in 0..xhci_n {
        // SAFETY: lecture seule des PORTSC d'un BAR0 trouve par le scan PCI.
        if unsafe { usb_r0::ports_connected(xhci_list[k]) } > 0 {
            xhci_bar = xhci_list[k];
            break;
        }
    }
    // Controleur HDA 1022:15E3 : BAR0 et position PCI (avant la sortie des services)
    let mut hda_bar: u64 = 0;
    let mut hda_bdf: (u8, u8, u8) = (0, 0, 0);
    if let Ok(hs) = boot::find_handles::<PciRootBridgeIo>() {
        for h in hs {
            // SAFETY: GetProtocol, lecture seule.
            let p = unsafe {
                boot::open_protocol::<PciRootBridgeIo>(
                    OpenProtocolParams { handle: h, agent: boot::image_handle(), controller: None },
                    OpenProtocolAttributes::GetProtocol,
                )
            };
            if let Ok(mut rb) = p {
                if let Ok(tree) = rb.enumerate() {
                    for a in tree.iter().copied() {
                        let mut cfg = [0u8; 64];
                        if rb.pci().read(a.with_register(0), &mut cfg).is_err() { continue; }
                        let vid = u16::from_le_bytes([cfg[0], cfg[1]]);
                        let did = u16::from_le_bytes([cfg[2], cfg[3]]);
                        if vid == 0x1022 && did == 0x15E3 && hda_bar == 0 {
                            let bar0 = u32::from_le_bytes([cfg[16], cfg[17], cfg[18], cfg[19]]);
                            let mut base = (bar0 & !0xF) as u64;
                            if (bar0 >> 1) & 3 == 2 {
                                let hi = u32::from_le_bytes([cfg[20], cfg[21], cfg[22], cfg[23]]);
                                base |= (hi as u64) << 32;
                            }
                            if base != 0 {
                                hda_bar = base;
                                hda_bdf = (a.bus, a.dev, a.fun);
                            }
                        }
                    }
                }
            }
        }
    }
    r.line(&format!("  HDA 1022:15E3 BAR0 : {:#x} bdf {:02x}:{:02x}.{}", hda_bar, hda_bdf.0, hda_bdf.1, hda_bdf.2));
    r.line(&format!("  xHCI BAR0 detecte : {:#x}", xhci_bar));
    r.save();

    // Arbre complet du codec HDA lu AVANT la sortie des services : il est ecrit sur la cle, car apres
    // la sortie le rapport ne peut plus etre sauvegarde. Lecture seule (la sonde ne fait que sortir
    // le controleur du reset si besoin, comme la sonde R0).
    if hda_bar != 0 {
        // SAFETY: BAR0 HDA et position PCI trouves par le scan ci-dessus ; verbes de lecture.
        let hp = unsafe { usb_r0::hda_probe(hda_bar, hda_bdf.0, hda_bdf.1, hda_bdf.2) };
        r.line(&format!(
            "  HDA PRE-SORTIE : CMD {:#010x} GCTL {:#010x} STATESTS {:#06x} CODEC {} VENDEUR {:#010x}",
            hp.cmd_after, hp.gctl, hp.statests, if hp.codec_found { "oui" } else { "non" }, hp.vendor
        ));
        // SAFETY: verbes de lecture uniquement.
        let arbre = unsafe { usb_r0::hda_arbre_complet(hda_bar, 0) };
        for ligne in &arbre {
            r.line(&format!("  HDA ARBRE : {}", ligne));
        }
        r.save();
    }

    // Etat du controleur xHCI avant la sortie : qui le possede (USB Legacy), ports, registres.
    // Lecture seule ; ecrit sur la cle car apres la sortie le rapport ne peut plus etre sauvegarde.
    if xhci_bar != 0 {
        // SAFETY: BAR0 xHCI trouve par le scan PCI ; lectures MMIO seules.
        for ligne in unsafe { usb_r0::xhci_etat(xhci_bar) } {
            r.line(&format!("  XHCI PRE-SORTIE : {}", ligne));
        }
        // Image brute des registres xHCI (4 Kio) : la carte entiere, octet par octet, pour l'analyse.
        let mut image = alloc::vec![0u8; 4096];
        for (k, octet) in image.iter_mut().enumerate() {
            // SAFETY: lecture MMIO d'un octet dans les 4 Kio du BAR0 du xHCI, sans effet de bord connu.
            *octet = unsafe { core::ptr::read_volatile((xhci_bar + k as u64) as *const u8) };
        }
        r.file("xHCI_BAR0_4k.bin", &image);
        r.save();
    }

    // Sortie des services UEFI. Toutes les decouvertes par protocoles sont faites ; le rapport est
    // ecrit sur la cle juste avant. Apres ce point : acces materiels directs, sortie a l'ecran et
    // voix seulement (le systeme de fichiers UEFI n'est plus disponible).
    // Les protocoles encore ouverts sont oublies (pas fermes) : leur fermeture appellerait les
    // services de demarrage, qui n'existent plus.
    if let Some(fs) = r.fs.take() {
        core::mem::forget(fs);
    }
    // SAFETY: aucun protocole ouvert n'est utilise apres ce point ; le retour de cet appel est oublie.
    let carte = unsafe { boot::exit_boot_services(None) };
    core::mem::forget(carte);

    // Mise en route du xHCI en R0 (lecture seule, aucune ecriture disque). Resultat affiche a l'ecran.
    let mut usb_line = [b' '; 40];
    let usb_len;
    // Clavier HID : (slot, DCI de l'endpoint) si configure ; message d'etat affiche sur une ligne
    let mut hid_info: Option<(u8, u8)> = None;
    let mut hid_msg = usb_r0::Line::new();
    if xhci_bar == 0 {
        let t = b"USB ABSENT";
        usb_line[..t.len()].copy_from_slice(t);
        usb_len = t.len();
    } else {
        // SAFETY: BAR0 trouve par le scan PCI ; acces MMIO en CPL0.
        match unsafe { usb_r0::xhci_bringup(xhci_bar) } {
            Ok(b) => {
                r.line(&format!("  XHCI OK : {} port(s) avec peripherique", b.nconnected));
                if b.legacy_forced {
                    r.line("  XHCI : firmware n'a pas rendu le controleur ; bit BIOS force a 0 (handoff force)");
                }
                let mut ln = usb_r0::Line::new();
                ln.s(b"USB OK P").h(b.nconnected as u32, 1);
                if b.nconnected > 0 {
                    let nmax = (b.nconnected as usize).min(8);
                    for ci in 0..nmax {
                        let port = b.connected[ci].port;
                        // Zone DMA propre a ce peripherique (contexte, anneau EP0, interruption)
                        // SAFETY: changement de zone entre deux peripheriques, aucun transfert en cours.
                        // SAFETY: nouvelle zone DMA pour ce peripherique.
                        if unsafe { usb_r0::zone_alloc() }.is_err() {
                            ln.s(b" P").h(port as u32, 1).s(b" ZONE PLEINE");
                            break;
                        }
                        // SAFETY: reset de port et Enable Slot (spec xHCI 4.3.1, 4.6.3).
                        let s = match unsafe { usb_r0::enable_slot(xhci_bar, port) } {
                            Ok(s) => s,
                            Err(e) => {
                                ln.s(b" P").h(port as u32, 1).s(b" ").s(e.as_bytes());
                                continue;
                            }
                        };
                        // SAFETY: Address Device et GET_DESCRIPTOR (spec 4.6.5).
                        // SAFETY: remise a zero de la classe avant identification.
                        unsafe { usb_enum::LAST_CLASS = 0 };
                        let id = unsafe { usb_enum::identify(xhci_bar, port, s) };
                        ln.s(b" P").h(port as u32, 1).s(b" S").h(s as u32, 2).s(b" ").s(&id.b[..id.p]);
                        // Hub : enumeration des peripheriques en aval (chapitre 11 USB 2.0)
                        // SAFETY: zone du hub active ; transferts de controle et Configure Endpoint.
                        if unsafe { usb_enum::LAST_CLASS } == 9 {
                            let hub_route = unsafe { usb_enum::root_route(xhci_bar, port) };
                            match unsafe { usb_hub::enumerate(xhci_bar, port, s, hub_route) } {
                                Ok((hs, dci)) => {
                                    hid_info = Some((hs, dci));
                                    hid_msg = usb_r0::Line::new();
                                    hid_msg.s(b"CLAVIER HID OK VIA HUB P").h(port as u32, 1);
                                    break;
                                }
                                Err(e) => {
                                    hid_msg.s(b"HUB ").s(e.as_bytes()).s(b" ");
                                    continue;
                                }
                            }
                        }
                        // SAFETY: configuration HID (spec 4.6.6, 4.10).
                        let cfg = match unsafe { usb_hid::get_config(xhci_bar, s) } {
                            Ok(c) => c,
                            Err(e) => {
                                hid_msg.s(b"CONFIG ").s(e.as_bytes());
                                continue;
                            }
                        };
                        let k = match usb_hid::find_keyboard(&cfg) {
                            Some(k) => k,
                            None => {
                                hid_msg.s(b"PAS HID P").h(port as u32, 1).s(b" ");
                                continue;
                            }
                        };
                        // SAFETY: Configure Endpoint, protocole boot, premier rapport.
                        match unsafe { usb_hid::setup_keyboard(xhci_bar, port, s, k) } {
                            Ok(()) => {
                                unsafe { usb_hid::start(xhci_bar, s, k.dci) };
                                hid_info = Some((s, k.dci));
                                hid_msg = usb_r0::Line::new();
                                hid_msg.s(b"CLAVIER HID OK P").h(port as u32, 1);
                                break;
                            }
                            Err(e) => {
                                hid_msg.s(b"HID ").s(e.as_bytes()).s(b" ");
                            }
                        }
                    }
                }
                usb_line[..ln.p].copy_from_slice(&ln.b[..ln.p]);
                usb_len = ln.p;
            }
            Err(e) => {
                r.line(&format!("  XHCI ERREUR : {}", e));
                let t = b"USB ERREUR";
                usb_line[..t.len()].copy_from_slice(t);
                usb_len = t.len();
            }
        }
    }
    lines[6] = usb_line;
    llen[6] = usb_len;

    // Sondes materielles faites AVANT la sortie des services : leurs resultats vont dans le rapport.
    let (nvme_line, nvme_id) = if nvme_bar != 0 {
        // SAFETY: BAR0 NVMe trouve par le scan PCI ; initialisation et lecture (spec NVMe 3.5, 7.6).
        unsafe { nvme::read_sector(nvme_bar, 1) }
    } else {
        let mut l = usb_r0::Line::new();
        l.s(b"NVME ABSENT");
        (l, usb_r0::Line::new())
    };
    r.line(&format!("  NVME ID : {}", core::str::from_utf8(&nvme_id.b[..nvme_id.p]).unwrap_or("?")));
    // Table de partitions GPT du NVMe interne (lecture seule), une ligne par partition
    let mut gpt_lines: [usb_r0::Line; 40] = core::array::from_fn(|_| usb_r0::Line::new());
    let gpt_n = if nvme_bar != 0 {
        // SAFETY: BAR0 NVMe trouve par le scan PCI ; lectures seules apres initialisation des files.
        unsafe { nvme::read_gpt(nvme_bar, &mut gpt_lines) }
    } else {
        0
    };
    for i in 0..gpt_n {
        r.line(&format!("  NVME GPT : {}", core::str::from_utf8(&gpt_lines[i].b[..gpt_lines[i].p]).unwrap_or("?")));
    }
    let audio_line = if hda_bar != 0 {
        // SAFETY: BAR0 HDA trouve avant la sortie des services ; verbes et flux HDA (spec 3.3, 7.3).
        unsafe { hda_play::play_test(hda_bar, 0) }
    } else {
        let mut l = usb_r0::Line::new();
        l.s(b"AUDIO ABSENT");
        l
    };
    // Arbre du codec HDA (lecture de verbes seulement) : une ligne par noeud dans le rapport
    if hda_bar != 0 {
        // SAFETY: BAR0 HDA trouve ; verbes de lecture uniquement.
        let tree = unsafe { usb_r0::hda_tree(hda_bar, 0) };
        for i in 0..tree.n {
            r.line(&format!("  HDA : {}", core::str::from_utf8(&tree.lines[i].b[..tree.lines[i].p]).unwrap_or("?")));
        }
    }
    r.line(&format!("  USB : {}", core::str::from_utf8(&usb_line[..usb_len]).unwrap_or("?")));
    r.line(&format!("  CLAVIER : {}", core::str::from_utf8(&hid_msg.b[..hid_msg.p]).unwrap_or("?")));
    r.line(&format!("  NVME : {}", core::str::from_utf8(&nvme_line.b[..nvme_line.p]).unwrap_or("?")));
    r.line(&format!("  AUDIO HDA : {}", core::str::from_utf8(&audio_line.b[..audio_line.p]).unwrap_or("?")));
    r.save();
    // Sonde HDA (une seule fois) : resultat ecrit dans le rapport, reutilise pour l'affichage.
    let hda_probe_res = if hda_bar != 0 {
        // SAFETY: BAR0 et position trouves par le scan PCI avant la sortie des services ; CPL0.
        let hp = unsafe { usb_r0::hda_probe(hda_bar, hda_bdf.0, hda_bdf.1, hda_bdf.2) };
        r.line(&format!(
            "  HDA SONDE : CMD {:#010x} GCTL {:#010x} STATESTS {:#06x} CODEC {} VENDEUR {:#010x}",
            hp.cmd_after, hp.gctl, hp.statests, if hp.codec_found { "oui" } else { "non" }, hp.vendor
        ));
        r.save();
        Some(hp)
    } else {
        r.line("  HDA SONDE : controleur HDA absent");
        r.save();
        None
    };

    if fbp != 0 && stride != 0 {
        // SAFETY: framebuffer GOP fourni par le firmware, acces direct en CPL0.
        unsafe {
            let fb = core::slice::from_raw_parts_mut(fbp as *mut u32, fbsz / 4);
            let _ = fb;
        }
        // Pas d'effacement de tout l'ecran : chaque ligne efface seulement sa bande (draw_row).
        for (li, line) in lines.iter().enumerate() {
            draw_row(fbp, fbsz, stride, li, &line[..llen[li]]);
        }
    }
    // Sonde HDA en R0 : active memoire + maitre de bus, lit le fournisseur du codec (lecture seule du codec)
    if let (Some(hp), true) = (hda_probe_res, fbp != 0) {
        let mut l10 = [b' '; 40];
        l10[..8].copy_from_slice(b"HDA CMD ");
        put_hex(&mut l10[8..16], hp.cmd_after);
        draw_row(fbp, fbsz, stride, 10, &l10[..16]);
        let mut l11 = [b' '; 40];
        if hp.codec_found && hp.vendor != 0 {
            l11[..6].copy_from_slice(b"CODEC ");
            put_hex(&mut l11[6..14], hp.vendor);
            draw_row(fbp, fbsz, stride, 11, &l11[..14]);
        } else {
            l11[..12].copy_from_slice(b"CODEC ABSENT");
            draw_row(fbp, fbsz, stride, 11, &l11[..12]);
        }
        // Arbre du codec : groupes de fonctions, widgets, broches et peripheriques branches
        // SAFETY: BAR0 HDA trouve avant la sortie des services ; lecture de verbes uniquement.
        let tree = unsafe { usb_r0::hda_tree(hda_bar, 0) };
        for i in 0..tree.n {
            draw_row(fbp, fbsz, stride, 12 + i, &tree.lines[i].b[..tree.lines[i].p]);
        }
    }
    // NVMe et audio : resultats calcules avant la sortie des services (et ecrits dans le rapport).
    if fbp != 0 {
        draw_row(fbp, fbsz, stride, 23, &nvme_line.b[..nvme_line.p]);
        draw_row(fbp, fbsz, stride, 24, &audio_line.b[..audio_line.p]);
    }
    // Lecteur d'ecran : le rapport se lit ligne par ligne au clavier, chaque ligne est affichee
    // (ligne 22) et dite par la sortie audio (voix a formants de voix.rs, pilote HDA a nous).
    //   fleche bas / haut : ligne suivante / precedente    debut / fin : premiere / derniere ligne
    //   F1 : aide    entree : relit la saisie    lettres : echo de la touche    Echap : quitter
    // Fin : touche Echap, ou 3000 delais consecutifs sans touche. Tout reste dans RAPPORT_UNIQUE.TXT.
    let lignes: Vec<String> = r.txt.lines().filter(|l| !l.trim().is_empty()).map(String::from).collect();
    let dis = |t: &str| {
        if fbp != 0 {
            let mut a = [b' '; 40];
            let mut k = 0;
            for &c in t.as_bytes() {
                if c < 0x80 && k < a.len() {
                    a[k] = c;
                    k += 1;
                }
            }
            draw_row(fbp, fbsz, stride, 22, &a[..k]);
        }
        if hda_bar != 0 {
            // SAFETY: BAR0 HDA trouve par le scan PCI, controleur initialise par hda_probe.
            unsafe { hda_play::parler(hda_bar, 0, t.as_bytes()) };
        }
    };
    const AIDE: &str = "Flèche bas : ligne suivante. Flèche haut : ligne précédente. Échap : quitter.";
    let mut typed = String::new();
    let mut why = "clavier HID absent";
    if hid_info.is_none() {
        dis("Analyse terminée. Clavier absent.");
    }
    if let Some((s, dci)) = hid_info {
        if fbp != 0 {
            draw_row(fbp, fbsz, stride, 21, &hid_msg.b[..hid_msg.p]);
        }
        dis("Analyse terminée. Lecteur d'écran prêt. F 1 : aide.");
        why = "delai sans touche";
        let mut prev = [0u8; 8];
        let mut idle = 0u32;
        let mut cur = lignes.len(); // position dans le rapport (len = apres la derniere ligne)
        let mut debut_saisie = 0usize;
        loop {
            // SAFETY: lecture du prochain rapport clavier (evenement du controleur).
            match unsafe { usb_hid::next_report(xhci_bar, s, dci) } {
                Ok(rep) => {
                    idle = 0;
                    let m = rep[0];
                    if rep[2..8].contains(&0x29) {
                        why = "touche Echap";
                        dis("Fin.");
                        break;
                    }
                    for &code in &rep[2..8] {
                        if code == 0 || prev[2..8].contains(&code) {
                            continue; // touche deja enfoncee dans le rapport precedent
                        }
                        // Navigation dans le rapport (codes d'usage HID du clavier).
                        let avant = cur;
                        match code {
                            0x51 => cur = (cur + 1).min(lignes.len().saturating_sub(1)),
                            0x52 => cur = cur.saturating_sub(1),
                            0x4A => cur = 0,
                            0x4D => cur = lignes.len().saturating_sub(1),
                            0x3A => dis(AIDE),
                            _ => {}
                        }
                        if matches!(code, 0x51 | 0x52 | 0x4A | 0x4D) {
                            match lignes.get(cur) {
                                Some(l) if cur != avant || code == 0x4A || code == 0x4D => dis(l),
                                Some(_) => dis(if code == 0x51 { "Fin du rapport." } else { "Début du rapport." }),
                                None => dis("Rapport vide."),
                            }
                            continue;
                        }
                        if let Some(c) = usb_hid::key_char(m, code) {
                            match c {
                                0x08 => {
                                    if typed.len() > debut_saisie {
                                        typed.pop();
                                    }
                                    dis("efface");
                                }
                                b'\n' => {
                                    let saisie = String::from(&typed[debut_saisie..]);
                                    dis(if saisie.is_empty() { "vide" } else { &saisie });
                                    typed.push('|');
                                    debut_saisie = typed.len();
                                }
                                b' ' => {
                                    typed.push(' ');
                                    dis("espace");
                                }
                                _ => {
                                    typed.push(c as char);
                                    let mut un = [0u8; 4];
                                    dis((c as char).encode_utf8(&mut un));
                                }
                            }
                        }
                    }
                    prev = rep;
                    if fbp != 0 {
                        let b = typed.as_bytes();
                        let tail = if b.len() > 36 { &b[b.len() - 36..] } else { b };
                        draw_row(fbp, fbsz, stride, 20, tail);
                    }
                }
                Err(_) => {
                    idle += 1;
                    if idle > 3000 {
                        break;
                    }
                }
            }
        }
    }
    r.line(&format!("  TOUCHES : {}", typed));
    r.line(&format!("  FIN CLAVIER : {}", why));
    }

    r.line("\n===== FIN : rapport unique ecrit dans \\EFI\\HWA2\\RAPPORT_UNIQUE.TXT =====");
    r.save();
    // Environ 3 s pour lire l'ecran, puis redemarrage a froid : le firmware reprend la main.
    usb_r0::attente_tsc(10_000_000_000);
    uefi::runtime::reset(uefi::runtime::ResetType::COLD, Status::SUCCESS, None)
}
