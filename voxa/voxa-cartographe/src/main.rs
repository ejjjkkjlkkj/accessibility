#![cfg(windows)]
//! Voxa Cartographe : outil maison, LECTURE SEULE, qui cartographie l'accessibilité d'une interface Windows.
//!   voxa-cartographe fenetres [--sortie fichier.jsonl]      toutes les fenêtres de premier niveau de la session ACTIVE (UI Automation) : contrôles, noms, atteignabilité clavier ;
//!                                                           pour chaque contrôle muet, dit si le lecteur de secours Voxa sait le nommer (voisin, parent, identifiant…)
//!   voxa-cartographe dialogues <fichier-ou-dossier> [--csv fichier.csv]   boîtes de dialogue classiques (RT_DIALOG) des binaires PE (exe/dll/mui…) : un dossier est parcouru récursivement,
//!                                                           ce qui permet d'analyser une image WinRE/Setup MONTÉE EN LECTURE SEULE ; défauts : sans Tab, bouton sans texte, champ sans étiquette
//! Confidentialité : les NOMS et VALEURS des contrôles ne sont jamais écrits (ni à l'écran ni dans le fichier) : seulement types, compteurs et identifiants techniques.
//! Limites : « rendre accessible » est fait AU MOMENT DE LA LECTURE par voxa-companion (nom reconstitué) ; cet outil ne modifie aucune application, aucun fichier Windows ni aucune image.
mod pe;
use std::ffi::c_void;
use std::io::Write;

type Hr = i32;
const CLSID_UIAUTOMATION: [u8; 16] = [0xA4, 0xDB, 0x48, 0xFF, 0xEF, 0x60, 0x01, 0x42, 0xAA, 0x87, 0x54, 0x10, 0x3E, 0xEF, 0x59, 0x4E];
const IID_IUIAUTOMATION: [u8; 16] = [0x7D, 0xE5, 0xCB, 0x30, 0xD0, 0xD9, 0x2A, 0x45, 0xAB, 0x13, 0x7A, 0xC5, 0xAC, 0x48, 0x25, 0xEE];
#[link(name = "ole32")] extern "system" {
    fn CoInitializeEx(reserved: *const c_void, flags: u32) -> Hr;
    fn CoCreateInstance(clsid: *const [u8; 16], outer: *const c_void, ctx: u32, iid: *const [u8; 16], out: *mut *mut c_void) -> Hr;
}
#[link(name = "oleaut32")] extern "system" { fn SysFreeString(s: *mut u16); }
#[link(name = "kernel32")] extern "system" {
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
    fn Process32FirstW(h: isize, e: *mut ProcessEntry) -> i32;
    fn Process32NextW(h: isize, e: *mut ProcessEntry) -> i32;
    fn CloseHandle(h: isize) -> i32;
}
#[repr(C)] struct ProcessEntry { size: u32, usage: u32, pid: u32, heap: usize, module: u32, threads: u32, parent: u32, prio: i32, flags: u32, exe: [u16; 260] }

// Indices COM vérifiés dans UIAutomationClient.h (SDK 10.0.26100) :
// IUIAutomation : GetRootElement 5, GetFocusedElement 8, get_ControlViewWalker 14 ; TreeWalker : GetParent 3, GetFirstChild 4, GetNextSibling 6, GetPreviousSibling 7 ;
// IUIAutomationElement : GetCurrentPattern 16, ProcessId 20, ControlType 21, LocalizedControlType 22, Name 23, IsKeyboardFocusable 27, IsEnabled 28, AutomationId 29, HelpText 31, IsPassword 35, LabeledBy 44.
unsafe fn vtbl(p: *mut c_void, i: usize) -> usize { let vt = *(p as *const *const usize); *vt.add(i) }
unsafe fn release(p: *mut c_void) { if !p.is_null() { let f: extern "system" fn(*mut c_void) -> u32 = std::mem::transmute(vtbl(p, 2)); f(p); } }
unsafe fn bstr_prop(el: *mut c_void, idx: usize) -> String {
    let f: extern "system" fn(*mut c_void, *mut *mut u16) -> Hr = std::mem::transmute(vtbl(el, idx));
    let mut b: *mut u16 = std::ptr::null_mut(); if f(el, &mut b) < 0 || b.is_null() { return String::new(); }
    let mut n = 0; while *b.add(n) != 0 && n < 1024 { n += 1; } let s = String::from_utf16_lossy(std::slice::from_raw_parts(b, n)); SysFreeString(b); s
}
unsafe fn el_ptr(obj: *mut c_void, idx: usize) -> *mut c_void { let f: extern "system" fn(*mut c_void, *mut *mut c_void) -> Hr = std::mem::transmute(vtbl(obj, idx)); let mut r = std::ptr::null_mut(); if f(obj, &mut r) < 0 { std::ptr::null_mut() } else { r } }
unsafe fn el_bool(obj: *mut c_void, idx: usize) -> bool { let f: extern "system" fn(*mut c_void, *mut i32) -> Hr = std::mem::transmute(vtbl(obj, idx)); let mut v = 0i32; f(obj, &mut v) >= 0 && v != 0 }
unsafe fn el_int(obj: *mut c_void, idx: usize) -> Option<i32> { let f: extern "system" fn(*mut c_void, *mut i32) -> Hr = std::mem::transmute(vtbl(obj, idx)); let mut v = 0i32; if f(obj, &mut v) >= 0 { Some(v) } else { None } }
unsafe fn voisin_de(walker: *mut c_void, el: *mut c_void, idx: usize) -> *mut c_void { if walker.is_null() { return std::ptr::null_mut(); } let f: extern "system" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> Hr = std::mem::transmute(vtbl(walker, idx)); let mut r = std::ptr::null_mut(); if f(walker, el, &mut r) < 0 { std::ptr::null_mut() } else { r } }

fn nom_processus(pid: u32) -> String { unsafe {
    let h = CreateToolhelp32Snapshot(2, 0); if h == -1 { return String::new(); }
    let mut e: ProcessEntry = std::mem::zeroed(); e.size = std::mem::size_of::<ProcessEntry>() as u32; let mut r = String::new(); let mut ok = Process32FirstW(h, &mut e);
    while ok != 0 { if e.pid == pid { let n = e.exe.iter().position(|&c| c == 0).unwrap_or(260); r = String::from_utf16_lossy(&e.exe[..n]); break; } ok = Process32NextW(h, &mut e); }
    CloseHandle(h); r } }

/// Identifiant d'automatisation lisible, ou vide s'il est opaque (GUID, nombre, trop long).
fn humanise_id(id: &str) -> String {
    let id = id.trim(); if id.is_empty() || id.len() > 48 || id.contains('{') { return String::new(); }
    if (id.len() >= 8 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-')) || id.chars().all(|c| c.is_ascii_digit()) { return String::new(); }
    id.to_string()
}
const INTERACTIFS: [i32; 14] = [50000, 50002, 50003, 50004, 50005, 50007, 50011, 50013, 50015, 50016, 50019, 50024, 50031, 50029];

#[derive(Default)] struct Stats { elements: u32, interactifs: u32, muets_natifs: u32, par_aide: u32, par_etiquette: u32, par_identifiant: u32, par_voisin: u32, par_parent: u32, toujours_muets: u32, non_focalisables: u32, mots_de_passe: u32 }

unsafe fn crawl(uia: *mut c_void, walker: *mut c_void, top: *mut c_void, max: u32) -> Stats {
    let mut s = Stats::default(); let mut pile: Vec<*mut c_void> = vec![top]; let mut visites = 0u32;
    while let Some(e) = pile.pop() {
        visites += 1; s.elements += 1;
        if visites < max { let mut c = voisin_de(walker, e, 4); let mut l = Vec::new(); while !c.is_null() && l.len() < 300 { l.push(c); c = voisin_de(walker, c, 6); } for x in l.into_iter().rev() { pile.push(x); } }
        if INTERACTIFS.contains(&el_int(e, 21).unwrap_or(0)) && el_bool(e, 28) {
            s.interactifs += 1; if el_bool(e, 35) { s.mots_de_passe += 1; }
            if !el_bool(e, 27) { s.non_focalisables += 1; }
            if bstr_prop(e, 23).trim().is_empty() {
                s.muets_natifs += 1;
                if !bstr_prop(e, 31).trim().is_empty() { s.par_aide += 1; }
                else { let lb = el_ptr(e, 44); let lbn = if lb.is_null() { String::new() } else { let n = bstr_prop(lb, 23); release(lb); n };
                    if !lbn.trim().is_empty() { s.par_etiquette += 1; }
                    else if !humanise_id(&bstr_prop(e, 29)).is_empty() { s.par_identifiant += 1; }
                    else { let pv = voisin_de(walker, e, 7); let v = if !pv.is_null() { let n = if el_int(pv, 21) == Some(50020) { bstr_prop(pv, 23) } else { String::new() }; release(pv); n } else { String::new() };
                        if !v.trim().is_empty() { s.par_voisin += 1; } else { let pa = voisin_de(walker, e, 3); let p = if !pa.is_null() { let n = bstr_prop(pa, 23); release(pa); n } else { String::new() }; if !p.trim().is_empty() { s.par_parent += 1; } else { s.toujours_muets += 1; } } } }
            }
        }
        if e != top { release(e); }
    }
    let _ = uia; s
}

fn fenetres(sortie: Option<&str>) {
    unsafe {
        CoInitializeEx(std::ptr::null(), 0); let mut uia = std::ptr::null_mut();
        if CoCreateInstance(&CLSID_UIAUTOMATION, std::ptr::null(), 1, &IID_IUIAUTOMATION, &mut uia) < 0 || uia.is_null() { println!("UIA=indisponible"); return; }
        let walker = el_ptr(uia, 14); let racine = el_ptr(uia, 5); if walker.is_null() || racine.is_null() { println!("UIA=arbre-indisponible"); return; }
        let mut lignes = String::new(); let mut tot = Stats::default(); let mut nf = 0u32; let mut top = voisin_de(walker, racine, 4);
        while !top.is_null() {
            let suivant = voisin_de(walker, top, 6); let pid = el_int(top, 20).unwrap_or(0) as u32; let proc = nom_processus(pid);
            if !proc.is_empty() { let s = crawl(uia, walker, top, 800); nf += 1;
                lignes.push_str(&format!("{{\"processus\":\"{}\",\"elements\":{},\"interactifs\":{},\"muets_natifs\":{},\"combles\":{{\"aide\":{},\"etiquette\":{},\"identifiant\":{},\"voisin\":{},\"parent\":{}}},\"toujours_muets\":{},\"non_focalisables\":{},\"mots_de_passe\":{}}}\n", proc.replace('\\', "/"), s.elements, s.interactifs, s.muets_natifs, s.par_aide, s.par_etiquette, s.par_identifiant, s.par_voisin, s.par_parent, s.toujours_muets, s.non_focalisables, s.mots_de_passe));
                tot.elements += s.elements; tot.interactifs += s.interactifs; tot.muets_natifs += s.muets_natifs; tot.par_aide += s.par_aide; tot.par_etiquette += s.par_etiquette; tot.par_identifiant += s.par_identifiant; tot.par_voisin += s.par_voisin; tot.par_parent += s.par_parent; tot.toujours_muets += s.toujours_muets; tot.non_focalisables += s.non_focalisables; tot.mots_de_passe += s.mots_de_passe; }
            release(top); top = suivant;
        }
        if let Some(f) = sortie { let _ = std::fs::write(f, &lignes); }
        let combles = tot.par_aide + tot.par_etiquette + tot.par_identifiant + tot.par_voisin + tot.par_parent;
        println!("FENETRES={nf} ELEMENTS={} INTERACTIFS={} MUETS_NATIFS={} COMBLES_PAR_LE_LECTEUR={combles} (aide {} etiquette {} identifiant {} voisin {} parent {}) TOUJOURS_MUETS={} NON_FOCALISABLES={} MOTS_DE_PASSE={}", tot.elements, tot.interactifs, tot.muets_natifs, tot.par_aide, tot.par_etiquette, tot.par_identifiant, tot.par_voisin, tot.par_parent, tot.toujours_muets, tot.non_focalisables, tot.mots_de_passe);
    }
}

fn parcourir(p: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if p.is_file() { out.push(p.to_path_buf()); return; }
    if let Ok(rd) = std::fs::read_dir(p) { for e in rd.flatten() { let q = e.path(); if q.is_dir() { parcourir(&q, out) } else if q.extension().map(|x| { let x = x.to_string_lossy().to_lowercase(); matches!(x.as_str(), "exe" | "dll" | "mui" | "cpl" | "ocx" | "scr") }).unwrap_or(false) { out.push(q) } } }
}

fn dialogues(chemin: &str, csv: Option<&str>) {
    let mut fichiers = vec![]; parcourir(std::path::Path::new(chemin), &mut fichiers);
    let (mut nf, mut ndlg, mut nctl, mut xaml) = (0u32, 0u32, 0u32, 0u32); let mut par_defaut = [0u32; 3]; let mut table = String::from("fichier,dialogue,controles,defauts\n");
    for f in &fichiers {
        if let Some(n) = f.file_name().map(|n| n.to_string_lossy().to_lowercase()) { if n.ends_with(".xbf") || n.ends_with(".xaml") { xaml += 1; } }
        let Ok(data) = std::fs::read(f) else { continue }; let Some(pe) = pe::Pe::open(&data) else { continue }; nf += 1;
        for (id, bytes) in pe.dialogs() { let Some(d) = pe::parse_dialog(bytes) else { continue }; ndlg += 1; nctl += d.items.len() as u32;
            let l = pe::lint(&d); for (_, k) in &l { par_defaut[match k { pe::Defaut::SansTabulation => 0, pe::Defaut::BoutonSansTexte => 1, pe::Defaut::ChampSansEtiquette => 2 }] += 1; }
            if !l.is_empty() { table.push_str(&format!("{},{},{},{}\n", f.file_name().unwrap().to_string_lossy(), id, d.items.len(), l.iter().map(|(_, k)| k.nom()).collect::<Vec<_>>().join("|"))); } }
    }
    if let Some(c) = csv { let _ = std::fs::write(c, table); }
    println!("FICHIERS_PE={nf} DIALOGUES={ndlg} CONTROLES={nctl} DEFAUTS: sans_tabulation={} bouton_sans_texte={} champ_sans_etiquette={} (fichiers xaml/xbf vus dans l'arborescence : {xaml}, NON analysés)", par_defaut[0], par_defaut[1], par_defaut[2]);
    let _ = std::io::stdout().flush();
}

fn main() {
    let a: Vec<String> = std::env::args().collect(); let opt = |n: &str| a.iter().position(|x| x == n).and_then(|i| a.get(i + 1)).map(|s| s.as_str());
    match a.get(1).map(|s| s.as_str()) {
        Some("fenetres") => fenetres(opt("--sortie")),
        Some("dialogues") => match a.get(2) { Some(p) if !p.starts_with("--") => dialogues(p, opt("--csv")), _ => eprintln!("usage : dialogues <fichier-ou-dossier> [--csv fichier]") },
        _ => eprintln!("usage : voxa-cartographe fenetres [--sortie f.jsonl] | dialogues <chemin> [--csv f.csv]"),
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("outil Windows uniquement");
}
