//! Voxa OS, jalon M0 : demarrage maison. Etapes (chacune est DITE avant la suivante) :
//!  1. collecte des faits du firmware (cadre graphique, tables de configuration, carte memoire, image chargee) ;
//!  2. ExitBootServices : le firmware n'est plus appele jamais ensuite ;
//!  3. noyau : console serie + ecran, memoire libre (allocateur de pages), tables ACPI, bus PCI par ECAM ;
//!  4. arret propre du processeur (pas de redemarrage ni d'extinction automatiques : decision de l'utilisateur).
//! Ce programme n'a jamais demarre sur un firmware reel (UNPROVEN) ; il est teste dans l'emulateur maison.
#![no_std]
#![no_main]
mod acpi;
mod console;
mod cpu;
mod gui;
mod hda;
mod os;
mod pci;
mod uefi;

use core::ffi::c_void;
use core::panic::PanicInfo;
use voxa_kernel::hw::{class_name, PciDevice};
use voxa_kernel::{boot_memory, FrameAllocator, Voice};

static mut MMAP: [u8; 65536] = [0; 65536];
static mut SMBIOS: [u8; 65536] = [0; 65536];
#[cfg(feature = "essai-violation")] static mut SYS_ST: *const u8 = core::ptr::null();
static mut DEVS: [PciDevice; 128] = [PciDevice { bus: 0, dev: 0, func: 0, id: voxa_kernel::PciId { vendor: 0, device: 0, class: 0, subclass: 0 } }; 128];

struct Faits { rsdp: u64, smbios: u64, smbios_len: usize, hii: Option<(u64, usize)>, fb_ok: bool, image: Option<(u64, u64)>, descripteurs: usize, pas: usize }

#[no_mangle]
pub extern "efiapi" fn efi_main(image: *mut c_void, system_table: *const u8) -> usize {
    let sys = uefi::Systeme { st: system_table, image };
    #[cfg(feature = "essai-violation")] unsafe { SYS_ST = system_table; }
    console::serial_init();
    let fb = sys.gop(); console::init(fb);
    let mut v = console::Parole;
    v.say("Voxa OS, jalon M0. Demarrage.");
    let faits0 = Faits { rsdp: sys.table_config(&uefi::GUID_ACPI20).unwrap_or(0), smbios: sys.table_config(&uefi::GUID_SMBIOS3).unwrap_or(0), smbios_len: sys.copier_smbios3(unsafe { &mut *core::ptr::addr_of_mut!(SMBIOS) }), hii: sys.exporter_hii(), fb_ok: fb.is_some(), image: sys.image_chargee(), descripteurs: 0, pas: 0 };
    v.say(if faits0.fb_ok { "Ecran graphique du firmware obtenu." } else { "Pas d'ecran graphique : journal serie seulement." });
    // carte memoire puis sortie des services : si la cle est perimee (la carte a change), on relit la carte une fois
    let mut faits = faits0;
    let mut sortie = false;
    for _ in 0..2 {
        let buf = unsafe { &mut *core::ptr::addr_of_mut!(MMAP) };
        let Some((n, pas, key)) = sys.carte_memoire(buf) else { v.say("Panne : carte memoire du firmware illisible. Arret."); cpu::halt_forever() };
        faits.descripteurs = n; faits.pas = pas;
        if sys.sortir(key) { sortie = true; break; }
    }
    if !sortie { v.say("Panne : le firmware refuse la sortie des services de demarrage. Arret."); cpu::halt_forever() }
    noyau(faits)
}

fn nb(mut n: u64, buf: &mut [u8; 20]) -> &str { let mut i = buf.len(); if n == 0 { i -= 1; buf[i] = b'0'; } while n > 0 { i -= 1; buf[i] = b'0' + (n % 10) as u8; n /= 10; } core::str::from_utf8(&buf[i..]).unwrap_or("?") }

/// Apres ExitBootServices : plus aucun appel au firmware.
fn noyau(f: Faits) -> ! {
    #[cfg(feature = "essai-violation")] { let s = uefi::Systeme { st: unsafe { SYS_ST }, image: core::ptr::null_mut() }; let _ = s.gop(); }
    let mut v = console::Parole;
    v.say("Services de demarrage du firmware quittes. Le noyau Voxa est seul aux commandes.");
    // memoire
    let map = unsafe { &*core::ptr::addr_of!(MMAP) };
    let mut frames: FrameAllocator = match boot_memory(&mut v, map, f.descripteurs, f.pas) { Some(a) => a, None => cpu::halt_forever() };
    if let Some((base, taille)) = f.image { frames.reserve_range(base, taille); }
    let sp = cpu::rsp(); frames.reserve_range(sp.saturating_sub(256 * 1024) & !0xFFF, 320 * 1024); // pile courante et sa marge
    frames.reserve_range(map.as_ptr() as u64, map.len() as u64);
    match f.hii { Some((p, n)) => { frames.reserve_range(p, n as u64); log!("BIOS : {} octets de listes HII copies avant la sortie des services.", n); } None => v.say("BIOS : listes HII indisponibles (la lecture du BIOS ne sera pas possible).") }
    let mut pci_n = 0usize; let mut audio_n = 0usize; let mut pm1a = 0u32; let mut s5 = None; let mut tables = [[0u8; 4]; 24]; let mut ntables = 0usize;
    let mut b = [0u8; 20];
    log!("Memoire libre apres reservation de l'image, de la pile et de la carte : {} Mo.", nb(frames.free_bytes() >> 20, &mut b));
    // ACPI
    if f.rsdp == 0 { v.say("ACPI : pas de RSDP dans les tables du firmware."); } else {
        match acpi::lire(f.rsdp) {
            None => v.say("ACPI : RSDP illisible ou somme de controle fausse."),
            Some(a) => {
                log!("ACPI revision {} : {} tables.", a.revision, a.n); ntables = a.n.min(24); for i in 0..ntables { tables[i] = a.tables[i].0; }
                for i in 0..a.n { let (s, ad) = a.tables[i]; log!("  table {} a {:#x}", core::str::from_utf8(&s).unwrap_or("????"), ad); }
                pm1a = a.pm1a_cnt; s5 = acpi::type_s5(a.dsdt); if a.pm1a_cnt != 0 { log!("Registre de controle d'alimentation PM1a : {:#x} ; DSDT a {:#x}.", a.pm1a_cnt, a.dsdt); }
                match a.ecam { Some(e) => log!("ECAM PCI : base {:#x}, bus {} a {}.", e.base, e.bus_debut, e.bus_fin), None => v.say("ACPI : pas de table MCFG, repli sur les ports PCI.") }
                // PCI
                let devs = unsafe { &mut *core::ptr::addr_of_mut!(DEVS) };
                let (n, methode) = pci::enumerer(a.ecam, devs);
                log!("PCI par {} : {} fonctions.", methode, n);
                let (mut audio, mut usb, mut stockage, mut graphique) = (0usize, 0usize, 0usize, 0usize); pci_n = n;
                for d in devs[..n].iter() {
                    log!("  {:02x}:{:02x}.{} {:04x}:{:04x} {}", d.bus, d.dev, d.func, d.id.vendor, d.id.device, class_name(d.id.class, d.id.subclass));
                    match (d.id.class, d.id.subclass) { (4, 3) | (4, 1) => audio += 1, (0x0C, 3) => usb += 1, (1, _) => stockage += 1, (3, _) => graphique += 1, _ => {} }
                }
                log!("Resume : {} audio, {} USB, {} stockage, {} graphique.", audio, usb, stockage, graphique);
                audio_n = audio; if audio == 0 { v.say("Attention : aucun controleur audio trouve ; la voix ne sera pas possible."); }
            }
        }
    }
    if f.smbios != 0 { log!("SMBIOS 3 a {:#x}.", f.smbios); }
    log!("Extinction ACPI : PM1a {:#x}, type S5 {:?}.", pm1a, s5);
    log!("FIN M0 : demarrage termine, lecteur d'ecran actif (clavier 8042). Echap pour quitter un ecran, Fin puis Entree deux fois pour eteindre.");
    let smb: &'static [u8] = unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(SMBIOS) as *const u8, f.smbios_len) };
    let hii: &'static [u8] = match f.hii { Some((p, n)) => unsafe { core::slice::from_raw_parts(p as *const u8, n) }, None => &[] };
    let mut systeme = os::Os::nouveau(smb, hii, pm1a, s5, frames.free_bytes() >> 20, pci_n, audio_n, tables, ntables);
    systeme.init_voix();
    os::boucle(systeme)
}

#[panic_handler]
fn panique(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    let _ = write!(console::Sortie, "\nPanne du noyau : {info}\n");
    cpu::halt_forever()
}
