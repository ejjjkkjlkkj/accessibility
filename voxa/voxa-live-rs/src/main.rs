//! Voxa Live (Rust) : application UEFI `no_std` qui embarque le cœur `silicon-core`.
//!
//! Chemin des données : touche (ConIn UEFI) -> `Key` -> `MenuApp::handle` (cœur) -> annonces -> `Platform::speak` -> {journal d'annonces,
//! affichage texte ConOut, voix (non encore portée en Rust : `speak` renvoie `false`)} ; si la voix est indisponible le cœur demande des bips
//! (`Platform::beep`) : autant de bips aigus que le rang de l'élément.
//!
//! Ce que ce programme ne fait PAS (encore) : aucune synthèse vocale ni pilote audio en Rust ; aucun accès au cadre graphique (texte ConOut
//! seulement). Il n'a jamais démarré sur un firmware réel.
#![no_std]
#![no_main]

use core::ffi::c_void;
use core::panic::PanicInfo;
use silicon_core::menu::{Key, MenuApp, Platform, Power};
use silicon_core::sysinfo::{smbios_lines, Lines};
use silicon_core::Out;
use voxa_speech::phon::{phon_texte, utf8_to_u16, PhonOut};
use voxa_speech::synth::{parler, Work};

mod hda;

// ---------------------------------------------------------------- interface UEFI minimale (offsets de la spécification UEFI 2.x)
const EFI_NOT_READY: usize = 0x8000_0000_0000_0006;
const ST_CON_IN: usize = 0x30;
const ST_CON_OUT: usize = 0x40;
const ST_RUNTIME: usize = 0x58;
const ST_BOOT: usize = 0x60;
const BS_STALL: usize = 0xF8;
const RS_RESET: usize = 0x68;
const IN_READ_KEY: usize = 0x08;
const OUT_STRING: usize = 0x08;

#[repr(C)]
struct InputKey { scan: u16, ch: u16 }

#[inline(always)]
unsafe fn field(base: *const u8, off: usize) -> *const u8 { *(base.add(off) as *const *const u8) }
#[inline(always)]
unsafe fn fnptr(base: *const u8, off: usize) -> usize { *(base.add(off) as *const usize) }

static mut SYSTEM_TABLE: *const u8 = core::ptr::null();

fn stall_us(us: usize) {
    unsafe {
        let bs = field(SYSTEM_TABLE, ST_BOOT);
        let f: extern "efiapi" fn(usize) -> usize = core::mem::transmute(fnptr(bs, BS_STALL));
        f(us);
    }
}

// ---------------------------------------------------------------- noyau Voxa : carte mémoire UEFI et bus PCI (crate voxa-kernel)
const BS_GET_MEMORY_MAP: usize = 0x38;
static mut MMAP: [u8; 32768] = [0; 32768];
static mut FRAMES: voxa_kernel::FrameAllocator = voxa_kernel::FrameAllocator::new();

/// Mémoire réutilisable (en Mo) d'après la carte mémoire du firmware, et pages ignorées au-delà de 4 Gio. None si le firmware refuse ou si la carte est illisible.
fn memoire_utilisable() -> Option<(u64, u64)> {
    unsafe {
        let bs = field(SYSTEM_TABLE, ST_BOOT);
        let f: extern "efiapi" fn(*mut usize, *mut u8, *mut usize, *mut usize, *mut u32) -> usize = core::mem::transmute(fnptr(bs, BS_GET_MEMORY_MAP));
        let (mut size, mut key, mut dsz, mut ver) = (32768usize, 0usize, 0usize, 0u32);
        let st = f(&mut size, core::ptr::addr_of_mut!(MMAP) as *mut u8, &mut key, &mut dsz, &mut ver);
        if st != 0 || dsz < 40 || size > 32768 { return None; }
        let buf = core::slice::from_raw_parts(core::ptr::addr_of!(MMAP) as *const u8, size);
        let fr = &mut *core::ptr::addr_of_mut!(FRAMES);
        fr.fill_from_map(buf, size / dsz, dsz);
        Some((fr.free_bytes() >> 20, fr.ignored_pages()))
    }
}

/// Accès réel aux ports d'E/S x86 pour les pilotes de voxa-kernel.
struct RealPorts;
impl voxa_kernel::hw::Ports for RealPorts {
    fn inb(&mut self, port: u16) -> u8 { let v: u8; unsafe { core::arch::asm!("in al, dx", out("al") v, in("dx") port, options(nomem, nostack, preserves_flags)); } v }
    fn outb(&mut self, port: u16, v: u8) { unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") v, options(nomem, nostack, preserves_flags)); } }
    fn inl(&mut self, port: u16) -> u32 { let v: u32; unsafe { core::arch::asm!("in eax, dx", out("eax") v, in("dx") port, options(nomem, nostack, preserves_flags)); } v }
    fn outl(&mut self, port: u16, v: u32) { unsafe { core::arch::asm!("out dx, eax", in("dx") port, in("eax") v, options(nomem, nostack, preserves_flags)); } }
}

/// Ajoute à la page d'informations : mémoire utilisable, nombre de périphériques PCI et leurs classes (en français).
fn noyau_lignes(lines: &mut Lines) {
    match memoire_utilisable() {
        Some((mo, ign)) => { let mut o = Out::new(); o.push("Mémoire utilisable : "); num(&mut o, mo); o.push(" Mo."); lines.add(o.as_str());
            if ign > 0 { lines.add("Attention : de la mémoire au-delà de 4 Go n'est pas utilisée."); } }
        None => { lines.add("Carte mémoire du firmware indisponible."); }
    }
    let mut devs = [voxa_kernel::hw::PciDevice { bus: 0, dev: 0, func: 0, id: voxa_kernel::PciId { vendor: 0, device: 0, class: 0, subclass: 0 } }; 96];
    let n = voxa_kernel::hw::pci_enumerate(&mut RealPorts, 0xFF, &mut devs);
    let mut o = Out::new(); o.push("Bus PCI : "); num(&mut o, n as u64); o.push(" périphériques."); lines.add(o.as_str());
    voxa_kernel::hw::for_each_class(&devs[..n], |nom, c| { let mut o = Out::new(); num(&mut o, c as u64); o.push(" "); o.push(nom); lines.add(o.as_str()); });
}

/// Écrit un entier décimal dans un `Out`.
fn num(o: &mut Out, mut n: u64) { let mut t = [0u8; 20]; let mut i = t.len(); if n == 0 { i -= 1; t[i] = b'0'; } while n > 0 { i -= 1; t[i] = b'0' + (n % 10) as u8; n /= 10; } o.push(core::str::from_utf8(&t[i..]).unwrap_or("?")); }

// ---------------------------------------------------------------- démarrer Windows installé (lecture seule, aucune écriture)
const BS_HANDLE_PROTOCOL: usize = 0x98;
const BS_LOAD_IMAGE: usize = 0xC8;
const BS_START_IMAGE: usize = 0xD0;
const BS_LOCATE_HANDLE_BUFFER: usize = 0x138;
const BS_LOCATE_PROTOCOL: usize = 0x140;
const BS_ALLOCATE_POOL: usize = 0x40;
/// EFI_HII_DATABASE_PROTOCOL_GUID (EF9FC172-A1B2-4693-B327-6D32FC416042).
const GUID_HII_DATABASE: [u8; 16] = [0x72, 0xC1, 0x9F, 0xEF, 0xB2, 0xA1, 0x93, 0x46, 0xB3, 0x27, 0x6D, 0x32, 0xFC, 0x41, 0x60, 0x42];
static mut IMAGE_HANDLE: *const c_void = core::ptr::null();
static mut DPBUF: [u8; 1024] = [0; 1024];

/// Cherche `\EFI\Microsoft\Boot\bootmgfw.efi` sur les volumes lisibles par le firmware, le charge et le démarre. Ne revient pas si Windows démarre.
/// Ne lit que l'existence du fichier (ouverture en lecture seule) ; n'écrit rien. NOT_VALIDATED sur firmware réel.
fn demarrer_windows(say: &mut dyn FnMut(&str)) -> Result<(), &'static str> {
    use voxa_kernel::efi::{device_path_len, file_device_path, path_utf16, GUID_DEVICE_PATH, GUID_SIMPLE_FILE_SYSTEM, WINDOWS_BOOT_PATH, WINDOWS_SETUP_MARKER};
    unsafe {
        let bs = field(SYSTEM_TABLE, ST_BOOT);
        let locate: extern "efiapi" fn(u32, *const u8, *const c_void, *mut usize, *mut *const *const c_void) -> usize = core::mem::transmute(fnptr(bs, BS_LOCATE_HANDLE_BUFFER));
        let handle_protocol: extern "efiapi" fn(*const c_void, *const u8, *mut *const u8) -> usize = core::mem::transmute(fnptr(bs, BS_HANDLE_PROTOCOL));
        let (mut count, mut handles) = (0usize, core::ptr::null::<*const c_void>());
        if locate(2, GUID_SIMPLE_FILE_SYSTEM.as_ptr(), core::ptr::null(), &mut count, &mut handles) != 0 || count == 0 || handles.is_null() { return Err("aucun volume lisible"); }
        let (mut boot16, mut setup16) = ([0u16; 64], [0u16; 64]);
        if path_utf16(WINDOWS_BOOT_PATH, &mut boot16).is_none() || path_utf16(WINDOWS_SETUP_MARKER, &mut setup16).is_none() { return Err("chemin interne trop long"); }
        // Ouvre un fichier en LECTURE SEULE sur un volume pour tester son existence, puis le referme.
        let exists = |sfs: *const u8, path: &[u16]| -> bool {
            let open_volume: extern "efiapi" fn(*const u8, *mut *const u8) -> usize = core::mem::transmute(fnptr(sfs, 0x08));
            let mut root: *const u8 = core::ptr::null();
            if open_volume(sfs, &mut root) != 0 || root.is_null() { return false; }
            let open: extern "efiapi" fn(*const u8, *mut *const u8, *const u16, u64, u64) -> usize = core::mem::transmute(fnptr(root, 0x08));
            let close_root: extern "efiapi" fn(*const u8) -> usize = core::mem::transmute(fnptr(root, 0x10));
            let mut file: *const u8 = core::ptr::null();
            let found = open(root, &mut file, path.as_ptr(), 1, 0) == 0 && !file.is_null(); // 1 = EFI_FILE_MODE_READ
            if found { let close_file: extern "efiapi" fn(*const u8) -> usize = core::mem::transmute(fnptr(file, 0x10)); close_file(file); }
            close_root(root); found
        };
        // Un volume avec \sources\boot.wim est une clé d'installation de Windows ; sans, c'est un Windows installé. On préfère le Windows installé.
        let (mut installed, mut installer): (Option<*const c_void>, Option<*const c_void>) = (None, None);
        for i in 0..count.min(32) {
            let h = *handles.add(i);
            let mut sfs: *const u8 = core::ptr::null();
            if handle_protocol(h, GUID_SIMPLE_FILE_SYSTEM.as_ptr(), &mut sfs) != 0 || sfs.is_null() { continue; }
            if !exists(sfs, &boot16) { continue; }
            if exists(sfs, &setup16) { if installer.is_none() { installer = Some(h); } } else if installed.is_none() { installed = Some(h); }
        }
        let (h, setup) = match (installed, installer) {
            (Some(h), _) => { say("Windows installé trouvé. Démarrage. Quand Windows sera lancé, Narrator se démarre avec Windows, Contrôle et Entrée."); (h, false) }
            (None, Some(h)) => { say("Clé d'installation de Windows trouvée. Démarrage de l'installation. Pour activer le lecteur d'écran, appuyez sur Windows, Contrôle et Entrée dès que l'écran d'installation apparaît."); (h, true) }
            (None, None) => return Err("gestionnaire de démarrage de Windows absent"),
        };
        let _ = setup;
        let mut dp: *const u8 = core::ptr::null();
        if handle_protocol(h, GUID_DEVICE_PATH.as_ptr(), &mut dp) != 0 || dp.is_null() { return Err("chemin du volume illisible"); }
        let mut o = 0usize;
        loop { if o + 4 > 512 { return Err("chemin du volume trop long"); } let (t, s, l) = (*dp.add(o), *dp.add(o + 1), u16::from_le_bytes([*dp.add(o + 2), *dp.add(o + 3)]) as usize); if l < 4 { return Err("chemin du volume invalide"); } o += l; if t == 0x7F && s == 0xFF { break; } }
        let vol = core::slice::from_raw_parts(dp, o);
        if device_path_len(vol) != Some(o) { return Err("chemin du volume invalide"); }
        let buf = &mut *core::ptr::addr_of_mut!(DPBUF);
        let Some(_n) = file_device_path(vol, WINDOWS_BOOT_PATH, buf) else { return Err("chemin de démarrage trop long") };
        let load: extern "efiapi" fn(u8, *const c_void, *const u8, *const c_void, usize, *mut *const c_void) -> usize = core::mem::transmute(fnptr(bs, BS_LOAD_IMAGE));
        let start: extern "efiapi" fn(*const c_void, *mut usize, *mut *const u16) -> usize = core::mem::transmute(fnptr(bs, BS_START_IMAGE));
        let mut img: *const c_void = core::ptr::null();
        if load(0, IMAGE_HANDLE, buf.as_ptr(), core::ptr::null(), 0, &mut img) != 0 || img.is_null() { return Err("le firmware refuse de charger le gestionnaire de démarrage"); }
        let (mut sz, mut data) = (0usize, core::ptr::null::<u16>());
        start(img, &mut sz, &mut data);
        Err("le gestionnaire de démarrage de Windows s'est terminé sans démarrer le système")
    }
}

/// Touche en attente sur ConIn, ou None.
fn read_key() -> Option<(u16, u16)> {
    unsafe {
        let con_in = field(SYSTEM_TABLE, ST_CON_IN);
        let f: extern "efiapi" fn(*const u8, *mut InputKey) -> usize = core::mem::transmute(fnptr(con_in, IN_READ_KEY));
        let mut k = InputKey { scan: 0, ch: 0 };
        let st = f(con_in, &mut k);
        if st == EFI_NOT_READY { None } else if st == 0 { Some((k.scan, k.ch)) } else { None }
    }
}

/// Écrit du texte UTF-8 sur la console texte (conversion en UTF-16, lignes de 64 caractères au plus).
fn console_print(s: &str) {
    unsafe {
        let con_out = field(SYSTEM_TABLE, ST_CON_OUT);
        if con_out.is_null() { return; }
        let f: extern "efiapi" fn(*const u8, *const u16) -> usize = core::mem::transmute(fnptr(con_out, OUT_STRING));
        let mut buf = [0u16; 64];
        let mut n = 0;
        for c in s.chars() {
            let cp = c as u32;
            let u = if cp < 0x10000 { cp as u16 } else { b'?' as u16 };
            buf[n] = u; n += 1;
            if n == 62 { buf[n] = 0; f(con_out, buf.as_ptr()); n = 0; }
        }
        buf[n] = 0x0D; buf[n + 1] = 0x0A; buf[n + 2] = 0;
        f(con_out, buf.as_ptr());
    }
}

fn reset(kind: u32) -> ! {
    unsafe {
        let rs = field(SYSTEM_TABLE, ST_RUNTIME);
        let f: extern "efiapi" fn(u32, usize, usize, *const c_void) = core::mem::transmute(fnptr(rs, RS_RESET));
        f(kind, 0, 0, core::ptr::null());
    }
    loop { core::hint::spin_loop(); }
}

// ---------------------------------------------------------------- journal d'annonces (lisible par l'émulateur et les outils de test)
#[repr(C)]
struct Journal { magic: [u8; 8], len: u64, buf: [u8; 4096] }
#[no_mangle]
#[used]
static mut JOURNAL: Journal = Journal { magic: *b"VXJRNL01", len: 0, buf: [0; 4096] };

fn journal_num(prefix: &str, mut n: usize) {
    let mut digits = [0u8; 20]; let mut i = 20;
    if n == 0 { i -= 1; digits[i] = b'0'; }
    while n > 0 { i -= 1; digits[i] = b'0' + (n % 10) as u8; n /= 10; }
    unsafe {
        let j = &mut *core::ptr::addr_of_mut!(JOURNAL);
        for b in prefix.bytes().chain(digits[i..].iter().copied()).chain(core::iter::once(10u8)) { if (j.len as usize) < j.buf.len() { j.buf[j.len as usize] = b; j.len += 1; } }
    }
}

fn journal(s: &str) {
    unsafe {
        let j = &mut *core::ptr::addr_of_mut!(JOURNAL);
        for b in s.bytes().chain(core::iter::once(b'\n')) {
            if (j.len as usize) < j.buf.len() { j.buf[j.len as usize] = b; j.len += 1; }
        }
    }
}

// ---------------------------------------------------------------- entrées/sorties directes (ports) : haut-parleur système
#[inline(always)]
unsafe fn outb(port: u16, v: u8) { core::arch::asm!("out dx, al", in("dx") port, in("al") v, options(nomem, nostack, preserves_flags)); }
#[inline(always)]
unsafe fn inb(port: u16) -> u8 { let v: u8; core::arch::asm!("in al, dx", out("al") v, in("dx") port, options(nomem, nostack, preserves_flags)); v }

fn speaker_tone(freq: u32, ms: u32) {
    unsafe {
        if freq >= 20 {
            let d = (1_193_182 / freq).min(65_535) as u16;
            outb(0x43, 0xB6);
            outb(0x42, (d & 0xFF) as u8);
            outb(0x42, (d >> 8) as u8);
            let p = inb(0x61);
            outb(0x61, (p & !3) | 3);
        }
        stall_us(ms as usize * 1000);
        let q = inb(0x61);
        outb(0x61, q & !3);
    }
}

/// Table SMBIOS 3 du firmware (table de configuration UEFI, GUID F2FD1544-9794-4A2C-992E-E5BBCF20E394). None si absente ou incohérente. Les données du firmware ne sont pas fiables : tailles bornées.
fn smbios3_table() -> Option<&'static [u8]> {
    const GUID: [u8; 16] = [0x44, 0x15, 0xFD, 0xF2, 0x94, 0x97, 0x2C, 0x4A, 0x99, 0x2E, 0xE5, 0xBB, 0xCF, 0x20, 0xE3, 0x94];
    unsafe {
        let n = *(SYSTEM_TABLE.add(0x68) as *const usize);
        let tab = *(SYSTEM_TABLE.add(0x70) as *const *const u8);
        if tab.is_null() || n == 0 || n > 512 { return None; }
        for i in 0..n {
            let e = tab.add(i * 24);
            if core::slice::from_raw_parts(e, 16) == GUID {
                let entry = *(e.add(16) as *const *const u8);
                if entry.is_null() || core::slice::from_raw_parts(entry, 5) != b"_SM3_" { return None; }
                let size = *(entry.add(0x0C) as *const u32) as usize;
                let addr = *(entry.add(0x10) as *const u64) as *const u8;
                if addr.is_null() || size == 0 || size > 65_536 { return None; }
                return Some(core::slice::from_raw_parts(addr, size));
            }
        }
    }
    None
}

// ---------------------------------------------------------------- tampons audio (statiques, alignés pour le DMA)
const MAX_SAMPLES: usize = 90_000;
#[repr(C, align(128))]
struct Stereo([i16; MAX_SAMPLES * 2]);
static mut STEREO: Stereo = Stereo([0; MAX_SAMPLES * 2]);
static mut MONO: [i16; MAX_SAMPLES] = [0; MAX_SAMPLES];
static mut WORK: Work = Work::new();

// ---------------------------------------------------------------- plateforme UEFI
struct UefiPlatform { audio: Option<hda::Hda>, audio_error: u8, bios: *const u8, bios_len: usize, bios_n: usize }

impl UefiPlatform {
    /// Synthétise `text` (français) et le joue par le contrôleur HD Audio. true = tous les morceaux ont été soumis et consommés par le DMA ;
    /// cela ne prouve pas que le son a été entendu.
    fn parler_audio(&mut self, text: &str) -> bool {
        let Some(h) = self.audio.as_mut() else { return false };
        let mut src = [0u16; 256];
        let n = utf8_to_u16(text, &mut src);
        let (mut d, mut ok) = (0usize, true);
        let mut o = PhonOut::new();
        while d < n {
            let nd = phon_texte(&src[..n], d, &mut o);
            d = if nd > d { nd } else { d + 1 };
            if o.len() == 0 { continue; }
            let (mono, stereo, work) = unsafe { (&mut *core::ptr::addr_of_mut!(MONO), &mut *core::ptr::addr_of_mut!(STEREO), &mut *core::ptr::addr_of_mut!(WORK)) };
            let r = parler(work, o.as_str(), mono);
            journal_num("[audio] échantillons synthétisés : ", r.samples);
            if r.samples == 0 { continue; }
            let frames = (r.samples + 31) / 32 * 32; // multiple de 128 octets
            for k in 0..frames { let s = if k < r.samples { mono[k] } else { 0 }; stereo.0[2 * k] = s; stereo.0[2 * k + 1] = s; }
            let lu = h.jouer(stereo.0.as_ptr() as u64, (frames * 4) as u32);
            journal(if lu { "[audio] tampon consommé par le DMA" } else { "[audio] ÉCHEC de lecture DMA" });
            if !lu { ok = false; }
            if r.truncated { ok = false; }
        }
        ok
    }
}

impl UefiPlatform {
    /// Demande au firmware lui-meme l'export de ses listes de paquets HII (ecrans de configuration). LECTURE SEULE : aucune variable n'est ecrite.
    fn charger_bios(&mut self) -> bool {
        if !self.bios.is_null() { return true; }
        unsafe {
            let bs = field(SYSTEM_TABLE, ST_BOOT);
            let locate: extern "efiapi" fn(*const u8, *const c_void, *mut *const u8) -> usize = core::mem::transmute(fnptr(bs, BS_LOCATE_PROTOCOL));
            let mut db: *const u8 = core::ptr::null();
            if locate(GUID_HII_DATABASE.as_ptr(), core::ptr::null(), &mut db) != 0 || db.is_null() { return false; }
            let export: extern "efiapi" fn(*const u8, *const c_void, *mut usize, *mut u8) -> usize = core::mem::transmute(fnptr(db, 0x20));
            let mut size = 0usize;
            export(db, core::ptr::null(), &mut size, core::ptr::null_mut());
            if size == 0 || size > (32 << 20) { return false; }
            let alloc: extern "efiapi" fn(u32, usize, *mut *mut u8) -> usize = core::mem::transmute(fnptr(bs, BS_ALLOCATE_POOL));
            let mut buf: *mut u8 = core::ptr::null_mut();
            if alloc(2, size, &mut buf) != 0 || buf.is_null() { return false; }
            if export(db, core::ptr::null(), &mut size, buf) != 0 { return false; }
            self.bios = buf; self.bios_len = size; self.bios_n = voxa_hii::nb_lignes(core::slice::from_raw_parts(buf, size)); true
        }
    }
    fn bios_slice(&self) -> &[u8] { if self.bios.is_null() { &[] } else { unsafe { core::slice::from_raw_parts(self.bios, self.bios_len) } } }
}

impl Platform for UefiPlatform {
    fn speak(&mut self, text: &str, _interrupt: bool) -> bool {
        journal(text);
        console_print(text);
        // Si l'audio est indisponible ou en échec, on le dit au cœur, qui passera au secours sonore (bips).
        self.parler_audio(text)
    }
    fn power(&mut self, p: Power) { reset(match p { Power::Restart => 0, Power::Shutdown => 2 }); }
    fn test_sound(&mut self) -> bool {
        // Le test du son est une PHRASE dite par la voix (chaîne audio complète), pas un bip ; le haut-parleur PC ne sert que si la voix échoue.
        if self.parler_audio("Ceci est un test du son.") { return true; }
        speaker_tone(880, 300); true
    }
    fn hardware_summary(&self, out: &mut Out) {
        // marque du processeur par CPUID (feuilles 0x80000002 à 0x80000004)
        let mut brand = [0u8; 48];
        for (i, leaf) in (0x8000_0002u32..=0x8000_0004).enumerate() {
            let r = unsafe { core::arch::x86_64::__cpuid(leaf) };
            for (j, w) in [r.eax, r.ebx, r.ecx, r.edx].iter().enumerate() { brand[i * 16 + j * 4..i * 16 + j * 4 + 4].copy_from_slice(&w.to_le_bytes()); }
        }
        out.push("Processeur ");
        let s = core::str::from_utf8(&brand).unwrap_or("inconnu").trim_matches(|c: char| c == '\0' || c == ' ');
        out.push(s);
        out.push(".");
    }
    fn system_lines(&self, lines: &mut Lines) {
        let mut brand = [0u8; 48];
        for (i, leaf) in (0x8000_0002u32..=0x8000_0004).enumerate() {
            let r = core::arch::x86_64::__cpuid(leaf);
            for (j, w) in [r.eax, r.ebx, r.ecx, r.edx].iter().enumerate() { brand[i * 16 + j * 4..i * 16 + j * 4 + 4].copy_from_slice(&w.to_le_bytes()); }
        }
        let cpu = core::str::from_utf8(&brand).unwrap_or("").trim_matches(|c: char| c == ' ' || c == ' ');
        match smbios3_table() {
            Some(t) => smbios_lines(t, cpu, lines),
            None => { let mut o = Out::new(); o.push("Processeur : "); o.push(cpu); lines.add(o.as_str()); lines.add("Tables SMBIOS indisponibles : mémoire et carte mère non lisibles."); }
        }
        noyau_lignes(lines);
    }
    fn beep(&mut self, freq: u32, ms: u32) -> bool { speaker_tone(freq, ms); true }
    fn bios_count(&mut self) -> usize { if self.charger_bios() { self.bios_n } else { 0 } }
    fn bios_line(&mut self, n: usize, out: &mut Out) -> Option<u16> {
        if !self.charger_bios() { return None; }
        let l = voxa_hii::ligne(self.bios_slice(), n)?;
        out.push(l.genre.mot()); out.push(" : ");
        if l.texte.est_vide() { out.push("sans titre"); } else { out.push(core::str::from_utf8(l.texte.as_bytes()).unwrap_or("?")); }
        if !l.valeur.est_vide() { out.push(" = "); out.push(core::str::from_utf8(l.valeur.as_bytes()).unwrap_or("?")); }
        if !l.aide.est_vide() { out.push(". "); out.push(core::str::from_utf8(l.aide.as_bytes()).unwrap_or("?")); }
        Some(l.cible)
    }
    fn bios_page_line(&mut self, page: u16) -> Option<usize> { if !self.charger_bios() { return None; } voxa_hii::indice_page(self.bios_slice(), page) }
    fn boot_windows(&mut self) -> Result<(), &'static str> { demarrer_windows(&mut |t| { self.speak(t, true); }) }
}

// ---------------------------------------------------------------- boucle principale
fn map_key(scan: u16, ch: u16) -> Option<Key> {
    match (scan, ch) {
        (1, _) => Some(Key::Up),
        (2, _) => Some(Key::Down),
        (5, _) => Some(Key::Home),
        (6, _) => Some(Key::End),
        (3, _) => Some(Key::Right),
        (4, _) => Some(Key::Left),
        (8, _) => Some(Key::Delete),
        (9, _) => Some(Key::PageUp),
        (10, _) => Some(Key::PageDown),
        (0x17, _) => Some(Key::Escape),
        (_, 13) => Some(Key::Enter),
        (_, 8) => Some(Key::Backspace),
        (_, 9) => Some(Key::Tab),
        (0, c) if c != 0 => char::from_u32(c as u32).map(Key::Char),
        _ => None,
    }
}

#[no_mangle]
pub extern "efiapi" fn efi_main(image: *mut c_void, system_table: *const u8) -> usize {
    unsafe { SYSTEM_TABLE = system_table; IMAGE_HANDLE = image as *const c_void; }
    let mut platform = UefiPlatform { audio: None, audio_error: 0, bios: core::ptr::null(), bios_len: 0, bios_n: 0 };
    match hda::Hda::init(stall_us) {
        Ok(h) => { platform.audio = Some(h); journal("[audio] contrôleur HD Audio initialisé"); }
        Err(code) => { platform.audio_error = code; journal("[audio] indisponible"); console_print("Audio : indisponible (code d erreur du pilote)"); }
    }
    let mut app = MenuApp::new();
    app.flush(&mut platform);
    loop {
        match read_key() {
            Some((scan, ch)) => {
                if let Some(k) = map_key(scan, ch) { app.handle(k); app.flush(&mut platform); }
            }
            None => stall_us(10_000),
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    journal("PANIC");
    loop { core::hint::spin_loop(); }
}
