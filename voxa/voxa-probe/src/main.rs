#![cfg(windows)]
//! voxa-probe : sonde maison, LECTURE SEULE, sans aucune dépendance (appels directs à kernel32/advapi32 + instruction CPUID).
//! N'écrit que dans le dossier de sortie demandé. Ne charge aucun pilote, ne modifie ni registre ni firmware.
//! Usage : voxa-probe <dossier-de-sortie>
//! Limites : depuis l'espace utilisateur on ne peut PAS lire l'espace de configuration PCI brut, les MSR ni la mémoire physique ; ces lectures
//! se feront dans Voxa lui-même (démarrage depuis une clé).
mod aml;
mod sha256;
use std::fmt::Write as _;
use std::fs;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{__cpuid_count, CpuidResult};

#[link(name = "kernel32")] extern "system" {
    fn EnumSystemFirmwareTables(sig: u32, buf: *mut u8, size: u32) -> u32;
    fn GetSystemFirmwareTable(sig: u32, id: u32, buf: *mut u8, size: u32) -> u32;
}
#[link(name = "advapi32")] extern "system" {
    fn RegOpenKeyExW(h: isize, sub: *const u16, opt: u32, sam: u32, out: *mut isize) -> i32;
    fn RegEnumKeyExW(h: isize, i: u32, name: *mut u16, nlen: *mut u32, res: *mut u32, class: *mut u16, clen: *mut u32, ft: *mut u64) -> i32;
    fn RegQueryValueExW(h: isize, name: *const u16, res: *mut u32, ty: *mut u32, data: *mut u8, size: *mut u32) -> i32;
    fn RegEnumValueW(h: isize, i: u32, name: *mut u16, nlen: *mut u32, res: *mut u32, ty: *mut u32, data: *mut u8, dlen: *mut u32) -> i32;
    fn RegCloseKey(h: isize) -> i32;
}
const HKLM: isize = 0x8000_0002u32 as i32 as isize; const KEY_READ64: u32 = 0x20019 | 0x100;

fn w(s: &str) -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() }
fn open(path: &str) -> Option<isize> { let mut h = 0isize; (unsafe { RegOpenKeyExW(HKLM, w(path).as_ptr(), 0, KEY_READ64, &mut h) } == 0).then_some(h) }
fn subkeys(h: isize) -> Vec<String> {
    let mut v = vec![]; for i in 0.. { let mut buf = [0u16; 256]; let mut n = 256u32; if unsafe { RegEnumKeyExW(h, i, buf.as_mut_ptr(), &mut n, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()) } != 0 { break; } v.push(String::from_utf16_lossy(&buf[..n as usize])); } v
}
fn value(h: isize, name: &str) -> Option<Vec<u8>> {
    let mut n = 0u32; if unsafe { RegQueryValueExW(h, w(name).as_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), &mut n) } != 0 { return None; }
    let mut d = vec![0u8; n as usize]; (unsafe { RegQueryValueExW(h, w(name).as_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), d.as_mut_ptr(), &mut n) } == 0).then(|| { d.truncate(n as usize); d })
}
fn u32le(b: &[u8], o: usize) -> u32 { b.get(o..o + 4).map(|x| u32::from_le_bytes(x.try_into().unwrap())).unwrap_or(0) }
fn u64le(b: &[u8], o: usize) -> u64 { b.get(o..o + 8).map(|x| u64::from_le_bytes(x.try_into().unwrap())).unwrap_or(0) }

/// CM_RESOURCE_LIST : décode ports, interruptions et mémoire avec leurs longueurs exactes.
fn resource_list(b: &[u8]) -> Vec<String> {
    let mut out = vec![]; let count = u32le(b, 0) as usize; let mut o = 4;
    for _ in 0..count.min(8) {
        let pc = u32le(b, o + 12) as usize; o += 16; // InterfaceType, BusNumber, Version/Revision, Count
        for _ in 0..pc.min(64) {
            let ty = *b.get(o).unwrap_or(&0xFF); let fl = u16::from_le_bytes([*b.get(o + 2).unwrap_or(&0), *b.get(o + 3).unwrap_or(&0)]);
            match ty {
                1 => out.push(format!("E/S    début 0x{:X}, longueur 0x{:X}", u64le(b, o + 4), u32le(b, o + 12))),
                2 => out.push(format!("IRQ    niveau {}, vecteur {}", u32le(b, o + 4), u32le(b, o + 8))),
                3 => out.push(format!("MÉMOIRE début 0x{:X}, longueur 0x{:X}{}", u64le(b, o + 4), u32le(b, o + 12), if fl & 0x20 != 0 { " (préchargeable)" } else { "" })),
                5 => out.push("(ressource spéciale)".into()),
                _ => {}
            }
            o += 20;
        }
    }
    out
}

#[cfg(target_arch = "x86_64")]
fn cpuid(leaf: u32, sub: u32) -> CpuidResult { unsafe { __cpuid_count(leaf, sub) } }

fn txt_cpu(out: &mut String) {
    #[cfg(target_arch = "x86_64")] {
        let r = cpuid(0, 0); let mut v = [0u8; 12]; v[..4].copy_from_slice(&r.ebx.to_le_bytes()); v[4..8].copy_from_slice(&r.edx.to_le_bytes()); v[8..].copy_from_slice(&r.ecx.to_le_bytes());
        let max = r.eax; let _ = writeln!(out, "Fabricant : {}  (feuille standard max 0x{:X})", String::from_utf8_lossy(&v), max);
        let e = cpuid(0x8000_0000, 0).eax; let _ = writeln!(out, "Feuille étendue max : 0x{:X}", e);
        if e >= 0x8000_0004 { let mut b = vec![]; for l in 0x8000_0002..=0x8000_0004u32 { let c = cpuid(l, 0); for x in [c.eax, c.ebx, c.ecx, c.edx] { b.extend(x.to_le_bytes()); } } let _ = writeln!(out, "Nom : {}", String::from_utf8_lossy(&b).trim_matches(char::from(0)).trim()); }
        let f = cpuid(1, 0); let base = (f.eax >> 8) & 0xF; let fam = if base == 0xF { base + ((f.eax >> 20) & 0xFF) } else { base }; let model = if base == 0xF || base == 6 { ((f.eax >> 4) & 0xF) | (((f.eax >> 16) & 0xF) << 4) } else { (f.eax >> 4) & 0xF };
        let _ = writeln!(out, "Famille 0x{:X}, modèle 0x{:X}, pas {}", fam, model, f.eax & 0xF);
        let _ = writeln!(out, "Fonctions (feuille 1) : SSE2={} SSE4.2={} AVX={} AES={} hyperviseur-présent={}", f.edx >> 26 & 1, f.ecx >> 20 & 1, f.ecx >> 28 & 1, f.ecx >> 25 & 1, f.ecx >> 31 & 1);
        if e >= 0x8000_0001 { let x = cpuid(0x8000_0001, 0); let _ = writeln!(out, "SVM (AMD-V) pris en charge : {}   NX={}  1Gio-pages={}", x.ecx >> 2 & 1, x.edx >> 20 & 1, x.edx >> 26 & 1); }
        if e >= 0x8000_000A { let s = cpuid(0x8000_000A, 0); let _ = writeln!(out, "SVM : révision {}, ASID max {}, NPT={} LBR-virt={} NRIP-save={} Flush-par-ASID={} AVIC={}", s.eax & 0xFF, s.ebx, s.edx & 1, s.edx >> 1 & 1, s.edx >> 3 & 1, s.edx >> 6 & 1, s.edx >> 13 & 1); }
        let mut leaves = String::new();
        for l in (0..=max.min(0x20)).chain(0x8000_0000..=e.min(0x8000_0026)) { let sub_max = if matches!(l, 4 | 7 | 0xB | 0xD | 0x8000_001D) { 6 } else { 1 }; for s in 0..sub_max { let c = cpuid(l, s); let _ = writeln!(leaves, "CPUID {:08X}.{} : eax={:08X} ebx={:08X} ecx={:08X} edx={:08X}", l, s, c.eax, c.ebx, c.ecx, c.edx); } }
        let _ = write!(out, "\n--- feuilles brutes ---\n{leaves}");
    }
}

fn acpi(dir: &str, out: &mut String) {
    const SIG: u32 = 0x4143_5049; // 'ACPI'
    let n = unsafe { EnumSystemFirmwareTables(SIG, std::ptr::null_mut(), 0) } as usize; let mut ids = vec![0u8; n];
    if n == 0 || unsafe { EnumSystemFirmwareTables(SIG, ids.as_mut_ptr(), n as u32) } == 0 { let _ = writeln!(out, "ACPI : énumération impossible"); return; }
    let _ = fs::create_dir_all(format!("{dir}/acpi")); let mut ids = ids; ids.extend_from_slice(b"DSDT"); let mut seen = std::collections::HashSet::new();
    for id in ids.chunks_exact(4) {
        let idv = u32::from_le_bytes(id.try_into().unwrap()); let name = String::from_utf8_lossy(id).into_owned();
        if name == "MSDM" { let _ = writeln!(out, "  MSDM : présente, NON lue (contient la clé de licence Windows : donnée secrète)"); continue; }
        if !seen.insert(idv) { let _ = writeln!(out, "  {name} : doublon (l'API n'en renvoie qu'un par signature)"); continue; }
        let sz = unsafe { GetSystemFirmwareTable(SIG, idv, std::ptr::null_mut(), 0) } as usize; if sz == 0 { continue; }
        let mut t = vec![0u8; sz]; let got = unsafe { GetSystemFirmwareTable(SIG, idv, t.as_mut_ptr(), sz as u32) } as usize; t.truncate(got);
        let _ = fs::write(format!("{dir}/acpi/{}.bin", name.trim()), &t);
        let oem = String::from_utf8_lossy(t.get(10..16).unwrap_or(&[])).into_owned(); let table_oem = String::from_utf8_lossy(t.get(16..24).unwrap_or(&[])).into_owned();
        let _ = writeln!(out, "  {name} : {} octets, révision {}, OEM « {} » « {} »", t.len(), t.get(8).copied().unwrap_or(0), oem.trim(), table_oem.trim());
        if name == "MCFG" { let mut o = 44; while o + 16 <= t.len() { let _ = writeln!(out, "    MCFG : base ECAM 0x{:X}, segment {}, bus {}..{} (espace de configuration PCI mappé en mémoire : lisible par Voxa au démarrage)", u64le(&t, o), t[o + 8] as u16 | (t[o + 9] as u16) << 8, t[o + 10], t[o + 11]); o += 16; } }
        if name == "FACP" { let _ = writeln!(out, "    FADT : profil PM {}, SCI IRQ {}, drapeaux 0x{:X}", t.get(45).copied().unwrap_or(0), u16::from_le_bytes([t.get(46).copied().unwrap_or(0), t.get(47).copied().unwrap_or(0)]), u32le(&t, 112)); }
        if name == "HPET" { let _ = writeln!(out, "    HPET : base 0x{:X}", u64le(&t, 44)); }
    }
}

fn pci_resources(out: &mut String) {
    let Some(pci) = open("SYSTEM\\CurrentControlSet\\Enum\\PCI") else { let _ = writeln!(out, "registre PCI inaccessible"); return };
    for dev in subkeys(pci) {
        let Some(dk) = open(&format!("SYSTEM\\CurrentControlSet\\Enum\\PCI\\{dev}")) else { continue };
        for inst in subkeys(dk) {
            let base = format!("SYSTEM\\CurrentControlSet\\Enum\\PCI\\{dev}\\{inst}");
            let Some(ik) = open(&base) else { continue };
            let desc = value(ik, "DeviceDesc").map(|d| { let u: Vec<u16> = d.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&c| c != 0).collect(); String::from_utf16_lossy(&u) }).unwrap_or_default();
            let loc = value(ik, "LocationInformation").map(|d| { let u: Vec<u16> = d.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&c| c != 0).collect(); String::from_utf16_lossy(&u) }).unwrap_or_default();
            let res = open(&format!("{base}\\LogConf")).and_then(|lk| { let v = value(lk, "BootConfig"); unsafe { RegCloseKey(lk) }; v }).map(|b| resource_list(&b)).unwrap_or_default();
            let _ = writeln!(out, "{dev}  {}  {}", desc.rsplit(';').next().unwrap_or(""), loc);
            for r in res { let _ = writeln!(out, "    {r}"); }
            unsafe { RegCloseKey(ik) };
        }
        unsafe { RegCloseKey(dk) };
    }
    unsafe { RegCloseKey(pci) };
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--aml") {
        // Decode des tables ACPI (fichiers .aml) extraites d'une image firmware : DSDT d'abord, puis SSDT. Lecture seule.
        let mut ns = aml::Namespace::default(); let mut fich: Vec<String> = std::env::args().skip(2).collect(); fich.sort_by_key(|f| !f.contains("DSDT"));
        for f in &fich { if let Ok(t) = fs::read(f) { ns.add_table(&t); } } ns.resolve();
        println!("TABLES={} PERIPHERIQUES={} METHODES={} ARRETS={} CHAMPS={} REGIONS={:?}", fich.len(), ns.devices.len(), ns.methods_seen, ns.unknown, ns.fields.len(), ns.regions);
        for dev in ns.devices.values() { if dev.hid.is_none() && dev.adr.is_none() && dev.res.is_empty() { continue; } println!("{}  HID={} UID={} ADR={}", dev.path, dev.hid.as_deref().unwrap_or("-"), dev.uid.as_deref().unwrap_or("-"), dev.adr.map(|a| format!("0x{a:X}")).unwrap_or("-".into())); for (n, r, c) in &dev.res { for x in r { println!("      {n}{}: {}", if *c { " [selon la branche]" } else { "" }, aml::describe(x)); } } }
        return;
    }
    let Some(dir) = std::env::args().nth(1) else { eprintln!("usage : voxa-probe <dossier-de-sortie>"); std::process::exit(2) };
    let _ = fs::create_dir_all(&dir);
    let mut cpu = String::new(); txt_cpu(&mut cpu); let _ = fs::write(format!("{dir}/cpuid.txt"), &cpu);
    let mut a = String::new(); let _ = writeln!(a, "Tables ACPI :"); acpi(&dir, &mut a); ssdt_registre(&dir, &mut a); let _ = fs::write(format!("{dir}/acpi.txt"), &a);
    let mut p = String::new(); pci_resources(&mut p); let _ = fs::write(format!("{dir}/pci-ressources.txt"), &p);
    let (ok, ko, det) = verifier_et_decoder(&dir); let _ = fs::write(format!("{dir}/acpi-peripheriques.txt"), &det); println!("Intégrité ACPI : {ok} PASS, {ko} FAIL");
    manifeste(&dir);
    println!("{}", cpu.lines().take(8).collect::<Vec<_>>().join("\n")); println!("{a}"); println!("PCI : {} lignes dans pci-ressources.txt", p.lines().count());
}

/// Vérifie chaque table ACPI écrite (somme de contrôle, longueur d'en-tête = taille du fichier) et décode l'espace de noms AML de DSDT et SSDT.
fn verifier_et_decoder(dir: &str) -> (usize, usize, String) {
    let (mut ok, mut ko) = (0, 0); let mut rap = String::from("Contrôles d'intégrité des tables ACPI :\n"); let mut ns = aml::Namespace::default();
    let mut files: Vec<_> = fs::read_dir(format!("{dir}/acpi")).map(|r| r.flatten().map(|e| e.path()).collect()).unwrap_or_default(); files.sort();
    for p in files {
        let Ok(t) = fs::read(&p) else { continue }; let n = p.file_name().unwrap().to_string_lossy().into_owned();
        let sum = t.iter().fold(0u8, |a, &b| a.wrapping_add(b)); let hl = u32le(&t, 4) as usize; let good = sum == 0 && hl == t.len();
        if good { ok += 1 } else { ko += 1 }
        let _ = writeln!(rap, "  {n:<10} somme={sum} longueur-en-tête={hl} taille-fichier={} {}", t.len(), if good { "PASS" } else { "FAIL" });
        if n.starts_with("DSDT") || n.starts_with("SSDT") { ns.add_table(&t); }
    }
    ns.resolve();
    let mut allf = String::new(); for (reg, nm, bit, n) in &ns.fields { let _ = writeln!(allf, "{reg}	{}	{nm}	octet 0x{:X}	bit {}	{n} bits", ns.regions.get(reg).map(|s| s.to_string()).unwrap_or("?".into()), bit / 8, bit % 8); } let _ = fs::write(format!("{dir}/acpi-champs.txt"), allf);
    let mut ecs = String::new();
    for (reg, nm, bit, n) in &ns.fields { if ns.regions.get(reg) == Some(&3) { let _ = writeln!(ecs, "  EC {reg}.{nm} : octet 0x{:02X}{} ({} bit{})", bit / 8, if bit % 8 != 0 { format!(" bit {}", bit % 8) } else { String::new() }, n, if *n > 1 { "s" } else { "" }); } }
    let mut d = String::new();
    let _ = writeln!(d, "Périphériques ACPI décodés : {} ({} méthodes vues ; arrêts d'analyse : {} ; blocs/termes lus sans effet sur les ressources : {})", ns.devices.len(), ns.methods_seen, ns.unknown, ns.skipped.len());
    let _ = writeln!(d, "Régions d'opération (nom=espace d'adresses) : {:?}", ns.regions);
    let _ = writeln!(d, "Registres du contrôleur embarqué ({} champs au total, dont ceux des régions d'espace 3 ci-dessous) :\n{ecs}", ns.fields.len());
    for n in ns.notes.iter().take(60) { let _ = writeln!(d, "  note : {n}"); }
    if ns.notes.len() > 60 { let _ = writeln!(d, "  … {} notes de plus", ns.notes.len() - 60); }
    for dev in ns.devices.values() {
        if dev.hid.is_none() && dev.adr.is_none() && dev.res.is_empty() { continue; }
        let _ = writeln!(d, "{}  HID={} CID={} UID={} ADR={}{}", dev.path, dev.hid.as_deref().unwrap_or("-"), dev.cid.as_deref().unwrap_or("-"), dev.uid.as_deref().unwrap_or("-"), dev.adr.map(|a| format!("0x{a:X}")).unwrap_or("-".into()), if dev.conditional { "  [défini sous condition]" } else { "" });
        for (n, r, c) in &dev.res { for x in r { let _ = writeln!(d, "      {n}{}: {}", if *c { " [selon la branche]" } else { "" }, aml::describe(x)); } }
    }
    (ok, ko, format!("{rap}\n{d}"))
}

/// Manifeste SHA-256 de tout ce que la sonde a écrit : permet de prouver qu'un relevé n'a pas changé.
fn manifeste(dir: &str) {
    let mut m = String::new(); let mut stack = vec![std::path::PathBuf::from(dir)]; let mut all = vec![];
    while let Some(p) = stack.pop() { if let Ok(rd) = fs::read_dir(&p) { for e in rd.flatten() { let q = e.path(); if q.is_dir() { stack.push(q) } else if q.file_name().map(|n| n != "manifeste.sha256").unwrap_or(true) { all.push(q) } } } }
    all.sort(); for q in all { if let Ok(b) = fs::read(&q) { let _ = writeln!(m, "{}  {}", sha256::sha256_hex(&b), q.strip_prefix(dir).unwrap_or(&q).display()); } }
    let _ = fs::write(format!("{dir}/manifeste.sha256"), m);
}

/// Récupère TOUTES les tables SSDT depuis le registre (HARDWARE\ACPI\SSDT\<OEM>\<table>\<révision>) : l'API GetSystemFirmwareTable n'en renvoie qu'une par signature.
fn ssdt_registre(dir: &str, out: &mut String) {
    let _ = fs::create_dir_all(format!("{dir}/acpi")); let mut n = 0;
    let Some(root) = open(r"HARDWARE\ACPI\SSDT") else { let _ = writeln!(out, "Registre SSDT inaccessible"); return };
    for oem in subkeys(root) { let p1 = format!(r"HARDWARE\ACPI\SSDT\{oem}"); let Some(k1) = open(&p1) else { continue };
        for tid in subkeys(k1) { let p2 = format!(r"{p1}\{tid}"); let Some(k2) = open(&p2) else { continue };
            for rev in subkeys(k2) { let p3 = format!(r"{p2}\{rev}"); let Some(k3) = open(&p3) else { continue };
                for i in 0.. { let mut nm = [0u16; 128]; let mut nl = 128u32; let mut ty = 0u32; let mut dl = 0u32;
                    if unsafe { RegEnumValueW(k3, i, nm.as_mut_ptr(), &mut nl, std::ptr::null_mut(), &mut ty, std::ptr::null_mut(), &mut dl) } != 0 { break; }
                    let name = String::from_utf16_lossy(&nm[..nl as usize]); let mut data = vec![0u8; dl as usize]; let mut nl2 = 128u32; let mut dl2 = dl;
                    if unsafe { RegEnumValueW(k3, i, nm.as_mut_ptr(), &mut nl2, std::ptr::null_mut(), &mut ty, data.as_mut_ptr(), &mut dl2) } != 0 { continue; }
                    data.truncate(dl2 as usize); if data.len() < 36 || &data[..4] != b"SSDT" { continue; }
                    let f = format!("{dir}/acpi/SSDT_{}_{}_{}_{}.bin", oem.trim(), tid.trim(), rev, name); let _ = fs::write(&f, &data); n += 1; }
                unsafe { RegCloseKey(k3) }; }
            unsafe { RegCloseKey(k2) }; }
        unsafe { RegCloseKey(k1) }; }
    unsafe { RegCloseKey(root) }; let _ = writeln!(out, "SSDT lues depuis le registre : {n}");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("outil Windows uniquement");
}
