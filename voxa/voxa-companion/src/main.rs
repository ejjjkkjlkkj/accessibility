//! Voxa Companion : lecteur de SECOURS pour Windows, WinRE et Windows Setup, en Rust sans dépendance externe.
//! Il ne remplace pas Narrator : il le complète là où il manque, ne répond plus ou n'est pas lancé.
//!   voxa-companion --once [--parler]            affiche (et dit) l'élément qui a le focus, puis quitte
//!   voxa-companion --veille [--secondes N] [--parler]   annonce chaque changement de focus (et de fenêtre) ; N=0 : sans fin
//!   voxa-companion --narrator                   lance Narrator.exe s'il ne tourne pas, puis quitte
//! Lecture seule : il ne modifie ni le registre, ni les fichiers, ni les réglages ; il ne fait que LIRE l'arbre UI Automation.
//! Les champs de mot de passe ne sont JAMAIS lus à voix haute (annonce « champ de mot de passe »).
//! Indices COM vérifiés dans UIAutomationClient.h (SDK 10.0.26100) : IUIAutomation::GetFocusedElement = 8 ; IUIAutomationElement : CurrentControlType 21,
//! CurrentLocalizedControlType 22, CurrentName 23, CurrentIsPassword 35. Statut : mesuré sur Windows 26300 (voir tests/companion.ps1) ; NOT_VALIDATED dans WinRE/Setup.
use std::ffi::c_void;
use std::io::Write;

type Hr = i32;
const CLSID_UIAUTOMATION: [u8; 16] = [0xA4, 0xDB, 0x48, 0xFF, 0xEF, 0x60, 0x01, 0x42, 0xAA, 0x87, 0x54, 0x10, 0x3E, 0xEF, 0x59, 0x4E]; // {ff48dba4-60ef-4201-aa87-54103eef594e}
const IID_IUIAUTOMATION: [u8; 16] = [0x7D, 0xE5, 0xCB, 0x30, 0xD0, 0xD9, 0x2A, 0x45, 0xAB, 0x13, 0x7A, 0xC5, 0xAC, 0x48, 0x25, 0xEE];   // {30cbe57d-d9d0-452a-ab13-7ac5ac4825ee}
const CLSID_SPVOICE: [u8; 16] = [0x77, 0x93, 0x74, 0x96, 0x91, 0x33, 0xD2, 0x11, 0x9E, 0xE3, 0x00, 0xC0, 0x4F, 0x79, 0x73, 0x96];
const IID_ISPVOICE: [u8; 16] = [0x74, 0xDF, 0x44, 0x6C, 0xB9, 0x72, 0x92, 0x49, 0xA1, 0xEC, 0xEF, 0x99, 0x6E, 0x04, 0x22, 0xD4];

#[link(name = "ole32")] extern "system" {
    fn CoInitializeEx(reserved: *const c_void, flags: u32) -> Hr;
    fn CoCreateInstance(clsid: *const [u8; 16], outer: *const c_void, ctx: u32, iid: *const [u8; 16], out: *mut *mut c_void) -> Hr;
}
#[link(name = "oleaut32")] extern "system" { fn SysFreeString(s: *mut u16); }
#[link(name = "user32")] extern "system" {
    fn GetForegroundWindow() -> isize;
    fn GetWindowTextW(h: isize, buf: *mut u16, max: i32) -> i32;
    fn SetForegroundWindow(h: isize) -> i32;
    fn SendInput(n: u32, inputs: *const KbInput, size: i32) -> u32;
}
#[link(name = "kernel32")] extern "system" {
    fn Sleep(ms: u32);
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
    fn Process32FirstW(h: isize, e: *mut ProcessEntry) -> i32;
    fn Process32NextW(h: isize, e: *mut ProcessEntry) -> i32;
    fn CloseHandle(h: isize) -> i32;
}
#[repr(C)] struct KbInput { kind: u32, _p: u32, vk: u16, scan: u16, flags: u32, time: u32, extra: usize, _pad: [u8; 8] }
#[repr(C)] struct ProcessEntry { size: u32, usage: u32, pid: u32, heap: usize, module: u32, threads: u32, parent: u32, prio: i32, flags: u32, exe: [u16; 260] }

unsafe fn vtbl(p: *mut c_void, i: usize) -> usize { let vt = *(p as *const *const usize); *vt.add(i) }
unsafe fn release(p: *mut c_void) { if !p.is_null() { let f: extern "system" fn(*mut c_void) -> u32 = std::mem::transmute(vtbl(p, 2)); f(p); } }
unsafe fn bstr_prop(el: *mut c_void, idx: usize) -> String {
    let f: extern "system" fn(*mut c_void, *mut *mut u16) -> Hr = std::mem::transmute(vtbl(el, idx));
    let mut b: *mut u16 = std::ptr::null_mut(); if f(el, &mut b) < 0 || b.is_null() { return String::new(); }
    let mut n = 0; while *b.add(n) != 0 && n < 1024 { n += 1; } let s = String::from_utf16_lossy(std::slice::from_raw_parts(b, n)); SysFreeString(b); s
}

struct Uia { p: *mut c_void }
/// Ce que le lecteur sait dire d'un élément : nom (éventuellement RECONSTITUÉ), d'où il vient, type, valeur, état à bascule, disponibilité.
struct Focus { name: String, source: &'static str, kind: String, password: bool, value: Option<String>, toggle: Option<i32>, enabled: bool }

/// Rend un identifiant d'automatisation lisible (« SearchBox_Edit » -> « search box edit ») ; vide si c'est un GUID, un nombre ou un identifiant opaque.
fn humanise_id(id: &str) -> String {
    let id = id.trim(); if id.is_empty() || id.len() > 48 || id.contains('{') { return String::new(); }
    let hexa = id.len() >= 8 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-'); if hexa || id.chars().all(|c| c.is_ascii_digit()) { return String::new(); }
    let mut out = String::new(); let mut prev_lower = false;
    for c in id.chars() { if c == '_' || c == '.' || c == '-' { if !out.ends_with(' ') { out.push(' '); } prev_lower = false; continue; } if c.is_uppercase() && prev_lower { out.push(' '); } out.extend(c.to_lowercase()); prev_lower = c.is_lowercase() || c.is_ascii_digit(); }
    out.trim().to_string()
}

/// Dictionnaire français des mots techniques courants des identifiants d'automatisation (« PART_CloseButton » -> « fermer »).
/// On ne traduit que si TOUS les mots significatifs sont connus : sinon on garde l'identifiant lisible tel quel (jamais de demi-traduction trompeuse).
const MOTS_BRUIT: [&str; 14] = ["part", "button", "btn", "control", "icon", "root", "container", "item", "template", "text", "label", "presenter", "content", "host"];
const DICO: &[(&str, &str)] = &[("close", "fermer"), ("minimize", "réduire"), ("maximize", "agrandir"), ("restore", "restaurer"), ("back", "retour"), ("forward", "suivant"), ("next", "suivant"), ("previous", "précédent"), ("prev", "précédent"),
    ("search", "rechercher"), ("find", "rechercher"), ("menu", "menu"), ("more", "plus d'options"), ("options", "options"), ("settings", "paramètres"), ("setting", "paramètre"), ("ok", "OK"), ("cancel", "annuler"), ("apply", "appliquer"),
    ("accept", "accepter"), ("decline", "refuser"), ("yes", "oui"), ("no", "non"), ("help", "aide"), ("info", "informations"), ("add", "ajouter"), ("new", "nouveau"), ("edit", "modifier"), ("save", "enregistrer"), ("open", "ouvrir"),
    ("refresh", "actualiser"), ("reload", "actualiser"), ("play", "lecture"), ("pause", "pause"), ("stop", "arrêt"), ("mute", "muet"), ("volume", "volume"), ("expand", "développer"), ("collapse", "réduire"), ("up", "haut"), ("down", "bas"),
    ("left", "gauche"), ("right", "droite"), ("home", "accueil"), ("share", "partager"), ("copy", "copier"), ("paste", "coller"), ("cut", "couper"), ("undo", "annuler la dernière action"), ("redo", "rétablir"), ("print", "imprimer"),
    ("download", "télécharger"), ("upload", "téléverser"), ("delete", "supprimer"), ("remove", "retirer"), ("clear", "effacer"), ("filter", "filtrer"), ("sort", "trier"), ("pin", "épingler"), ("unpin", "détacher"), ("favorite", "favori"),
    ("user", "utilisateur"), ("profile", "profil"), ("account", "compte"), ("notification", "notification"), ("notifications", "notifications"), ("camera", "caméra"), ("microphone", "microphone"), ("wifi", "wifi"), ("bluetooth", "bluetooth"),
    ("battery", "batterie"), ("brightness", "luminosité"), ("toggle", "bascule"), ("switch", "commutateur"), ("scroll", "défilement")];
fn traduire_id(lisible: &str) -> Option<String> {
    let mots: Vec<&str> = lisible.split_whitespace().filter(|m| !MOTS_BRUIT.contains(m)).collect(); if mots.is_empty() { return None; }
    let mut out: Vec<&str> = Vec::new(); for m in &mots { match DICO.iter().find(|(en, _)| en == m) { Some((_, fr)) => out.push(fr), None => return None } }
    Some(out.join(" "))
}
/// Choix du nom, par ordre de fiabilité : nom propre, aide, étiquette liée, identifiant lisible, texte voisin précédent, nom du parent.
fn choisir_nom(nom: &str, aide: &str, etiquette: &str, id: &str, voisin: &str, parent: &str) -> (String, &'static str) {
    for (v, s) in [(nom, "nom"), (aide, "aide"), (etiquette, "etiquette")] { if !v.trim().is_empty() { return (v.trim().to_string(), s); } }
    let h = humanise_id(id); if !h.is_empty() { return (traduire_id(&h).unwrap_or(h), "identifiant"); }
    if !voisin.trim().is_empty() { return (voisin.trim().to_string(), "voisin"); }
    if !parent.trim().is_empty() { return (parent.trim().to_string(), "parent"); }
    (String::new(), "aucun")
}

unsafe fn el_ptr(obj: *mut c_void, idx: usize) -> *mut c_void { let f: extern "system" fn(*mut c_void, *mut *mut c_void) -> Hr = std::mem::transmute(vtbl(obj, idx)); let mut r = std::ptr::null_mut(); if f(obj, &mut r) < 0 { std::ptr::null_mut() } else { r } }
unsafe fn el_bool(obj: *mut c_void, idx: usize) -> bool { let f: extern "system" fn(*mut c_void, *mut i32) -> Hr = std::mem::transmute(vtbl(obj, idx)); let mut v = 0i32; f(obj, &mut v) >= 0 && v != 0 }
unsafe fn el_int(obj: *mut c_void, idx: usize) -> Option<i32> { let f: extern "system" fn(*mut c_void, *mut i32) -> Hr = std::mem::transmute(vtbl(obj, idx)); let mut v = 0i32; if f(obj, &mut v) >= 0 { Some(v) } else { None } }
/// Vrai si le contrôle n'a AUCUNE surface (rectangle vide) : IUIAutomationElement::CurrentBoundingRectangle (indice 43). Un contrôle sans surface ET non focalisable est un fantôme de l'arbre (ex. champ de texte caché de Chromium) : aucun lecteur d'écran ne le présente, il ne compte pas comme défaut d'accessibilité.
unsafe fn rect_vide(obj: *mut c_void) -> bool { #[repr(C)] struct R { l: i32, t: i32, r: i32, b: i32 } let f: extern "system" fn(*mut c_void, *mut R) -> Hr = std::mem::transmute(vtbl(obj, 43)); let mut r = R { l: 0, t: 0, r: 0, b: 0 }; f(obj, &mut r) >= 0 && (r.r - r.l <= 0 || r.b - r.t <= 0) }
unsafe fn motif(el: *mut c_void, id: i32) -> *mut c_void { let f: extern "system" fn(*mut c_void, i32, *mut *mut c_void) -> Hr = std::mem::transmute(vtbl(el, 16)); let mut r = std::ptr::null_mut(); if f(el, id, &mut r) < 0 { std::ptr::null_mut() } else { r } }
unsafe fn voisin_de(walker: *mut c_void, el: *mut c_void, idx: usize) -> *mut c_void { if walker.is_null() { return std::ptr::null_mut(); } let f: extern "system" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> Hr = std::mem::transmute(vtbl(walker, idx)); let mut r = std::ptr::null_mut(); if f(walker, el, &mut r) < 0 { std::ptr::null_mut() } else { r } }

impl Uia {
    fn new() -> Option<Uia> { unsafe { CoInitializeEx(std::ptr::null(), 0); let mut p = std::ptr::null_mut(); if CoCreateInstance(&CLSID_UIAUTOMATION, std::ptr::null(), 1, &IID_IUIAUTOMATION, &mut p) < 0 || p.is_null() { None } else { Some(Uia { p }) } } }
    fn focus(&self) -> Option<Focus> { unsafe { let el = el_ptr(self.p, 8); if el.is_null() { return None; } let f = self.decrire(el); release(el); Some(f) } }
    /// Décrit un élément : nom reconstitué si besoin, valeur (ValuePattern 10002), état à bascule (TogglePattern 10015), jamais le contenu d'un mot de passe.
    unsafe fn decrire(&self, el: *mut c_void) -> Focus {
        let password = el_bool(el, 35); let enabled = el_bool(el, 28);
        let (nom, kind, aide, id) = (bstr_prop(el, 23), bstr_prop(el, 22), bstr_prop(el, 31), bstr_prop(el, 29));
        let (mut etiquette, mut voisin, mut parent) = (String::new(), String::new(), String::new());
        if nom.trim().is_empty() && aide.trim().is_empty() && !password {
            let lb = el_ptr(el, 44); if !lb.is_null() { etiquette = bstr_prop(lb, 23); release(lb); }
            if etiquette.trim().is_empty() && humanise_id(&id).is_empty() {
                let walker = el_ptr(self.p, 14);
                let genre = el_int(el, 21).unwrap_or(0); let ligne = [50007, 50029, 50024].contains(&genre);
                if ligne { let c1 = voisin_de(walker, el, 4); let mut c = c1; let mut n = 0; while !c.is_null() && n < 6 && voisin.trim().is_empty() { voisin = bstr_prop(c, 23); if voisin.trim().is_empty() { let g = voisin_de(walker, c, 4); if !g.is_null() { voisin = bstr_prop(g, 23); release(g); } } let s = voisin_de(walker, c, 6); release(c); c = s; n += 1; } release(c); }
                else { let pv = voisin_de(walker, el, 7); if !pv.is_null() { if el_int(pv, 21) == Some(50020) { voisin = bstr_prop(pv, 23); } release(pv); } }
                if voisin.trim().is_empty() { let pa = voisin_de(walker, el, 3); if !pa.is_null() { parent = bstr_prop(pa, 23); release(pa); } }
                release(walker);
            }
        }
        let (name, source) = choisir_nom(&nom, &aide, &etiquette, &id, &voisin, &parent);
        let mut value = None; let mut toggle = None;
        if !password {
            let vp = motif(el, 10002); if !vp.is_null() { let s = bstr_prop(vp, 4); release(vp); let s: String = s.chars().take(200).collect(); value = Some(s); }
            let tp = motif(el, 10015); if !tp.is_null() { toggle = el_int(tp, 4); release(tp); }
        }
        Focus { name, source, kind, password, value, toggle, enabled }
    }
}

struct Voix { p: *mut c_void }
impl Voix {
    fn new() -> Option<Voix> { unsafe { CoInitializeEx(std::ptr::null(), 0); let mut p = std::ptr::null_mut(); if CoCreateInstance(&CLSID_SPVOICE, std::ptr::null(), 1, &IID_ISPVOICE, &mut p) < 0 || p.is_null() { None } else { Some(Voix { p }) } } }
    fn dire(&self, t: &str) { let w: Vec<u16> = t.encode_utf16().chain(Some(0)).collect(); unsafe { let f: extern "system" fn(*mut c_void, *const u16, u32, *mut u32) -> Hr = std::mem::transmute(vtbl(self.p, 20)); f(self.p, w.as_ptr(), 1 | 2 | 16, std::ptr::null_mut()); } } // asynchrone, coupe la parole en cours, texte brut
    fn attendre(&self, ms: u32) { unsafe { let f: extern "system" fn(*mut c_void, u32) -> Hr = std::mem::transmute(vtbl(self.p, 32)); f(self.p, ms); } }
}

fn titre_fenetre_active() -> String { unsafe { let h = GetForegroundWindow(); if h == 0 { return String::new(); } let mut b = [0u16; 256]; let n = GetWindowTextW(h, b.as_mut_ptr(), 256); String::from_utf16_lossy(&b[..n.max(0) as usize]) } }

/// Phrase annoncée : « nom, type[, état][, valeur : …][, indisponible] » ; un champ de mot de passe n'est JAMAIS lu (ni nom, ni valeur).
fn phrase(f: &Focus) -> String {
    if f.password { return "Champ de mot de passe".into(); }
    let nom = f.name.trim(); let kind = f.kind.trim();
    let mut s = match (nom.is_empty(), kind.is_empty()) { (false, false) => format!("{nom}, {kind}"), (false, true) => nom.to_string(), (true, false) => format!("{kind} sans nom"), (true, true) => "élément sans nom".into() };
    match f.toggle { Some(0) => s.push_str(", non coché"), Some(1) => s.push_str(", coché"), Some(2) => s.push_str(", indéterminé"), _ => {} }
    if let Some(v) = &f.value { let v = v.trim(); if !v.is_empty() { s.push_str(&format!(", valeur : {v}")); } }
    if !f.enabled { s.push_str(", indisponible"); }
    s
}

fn nom_processus(pid: u32) -> String { unsafe {
    let h = CreateToolhelp32Snapshot(2, 0); if h == -1 { return String::new(); }
    let mut e: ProcessEntry = std::mem::zeroed(); e.size = std::mem::size_of::<ProcessEntry>() as u32; let mut r = String::new(); let mut ok = Process32FirstW(h, &mut e);
    while ok != 0 { if e.pid == pid { let n = e.exe.iter().position(|&c| c == 0).unwrap_or(260); r = String::from_utf16_lossy(&e.exe[..n]); break; } ok = Process32NextW(h, &mut e); }
    CloseHandle(h); r } }

fn narrator_tourne() -> bool { unsafe {
    let h = CreateToolhelp32Snapshot(2, 0); if h == -1 { return false; }
    let mut e: ProcessEntry = std::mem::zeroed(); e.size = std::mem::size_of::<ProcessEntry>() as u32; let mut trouve = false; let mut ok = Process32FirstW(h, &mut e);
    while ok != 0 { let n = e.exe.iter().position(|&c| c == 0).unwrap_or(260); if String::from_utf16_lossy(&e.exe[..n]).eq_ignore_ascii_case("narrator.exe") { trouve = true; break; } ok = Process32NextW(h, &mut e); }
    CloseHandle(h); trouve } }

fn say(s: &str) { let mut o = std::io::stdout().lock(); let _ = writeln!(o, "{s}"); let _ = o.flush(); }

/// Correspondance de titre : « =Titre » exige l'égalité exacte (insensible à la casse) ; sinon simple inclusion. Évite de mesurer par erreur une autre fenêtre dont le titre contient le mot.
fn titre_correspond(nom: &str, filtre: &str) -> bool { match filtre.strip_prefix('=') { Some(exact) => nom.trim().eq_ignore_ascii_case(exact.trim()) || nom.trim().to_lowercase() == exact.trim().to_lowercase(), None => nom.to_lowercase().contains(&filtre.to_lowercase()) } }

/// Types de contrôles interactifs (UIA_*ControlTypeId) que l'utilisateur doit pouvoir identifier et atteindre.
const INTERACTIFS: [i32; 14] = [50000, 50002, 50003, 50004, 50005, 50007, 50011, 50013, 50015, 50016, 50019, 50024, 50031, 50029];

/// Mesure, pour les fenêtres d'un processus, combien de contrôles interactifs n'ont pas de nom propre et combien le lecteur de secours sait nommer.
fn audit_processus(uia: &Uia, exe: &str, titre: Option<&str>) { unsafe {
    let walker = el_ptr(uia.p, 14); let racine = el_ptr(uia.p, 5); if walker.is_null() || racine.is_null() { say("AUDIT=indisponible"); return; }
    let (mut fen, mut elements, mut inter, mut sans_nom, mut combles, mut restants) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
    let mut top = voisin_de(walker, racine, 4);
    while !top.is_null() {
        let pid = el_int(top, 20).unwrap_or(0) as u32;
        let suivant = voisin_de(walker, top, 6);
        let titre_ok = titre.map(|t| titre_correspond(&bstr_prop(top, 23), t)).unwrap_or(true);
        if titre_ok && (exe == "*" || nom_processus(pid).eq_ignore_ascii_case(exe)) {
            fen += 1; let mut pile: Vec<*mut c_void> = vec![top]; let mut visites = 0;
            while let Some(e) = pile.pop() {
                visites += 1; elements += 1;
                if visites < 800 {
                    let mut c = voisin_de(walker, e, 4); let mut liste = Vec::new();
                    while !c.is_null() && liste.len() < 300 { liste.push(c); c = voisin_de(walker, c, 6); }
                    for x in liste.into_iter().rev() { pile.push(x); }
                }
                let kind = el_int(e, 21).unwrap_or(0);
                if INTERACTIFS.contains(&kind) && el_bool(e, 28) && (el_bool(e, 27) || !rect_vide(e)) {
                    inter += 1; let nom = bstr_prop(e, 23);
                    if nom.trim().is_empty() { sans_nom += 1; let f = uia.decrire(e); if f.source != "aucun" { combles += 1 } else { restants += 1; if std::env::var("VOXA_DETAIL").is_ok() { say(&format!("RESTE type={} classe={} id={}", f.kind, bstr_prop(e, 30), bstr_prop(e, 29))); } } }
                }
                if e != top { release(e); }
            }
        }
        release(top);
        top = suivant;
    }
    say(&format!("AUDIT processus={exe} fenetres={fen} elements={elements} interactifs={inter} sans_nom_propre={sans_nom} combles_par_le_lecteur={combles} toujours_sans_nom={restants}"));
} }


/// Appuie sur Tab (touche seule, jamais Entrée ni Espace) : le clavier est le moyen de navigation de tout lecteur d'écran.
fn envoyer_tab() { unsafe { let bas = KbInput { kind: 1, _p: 0, vk: 0x09, scan: 0, flags: 0, time: 0, extra: 0, _pad: [0; 8] }; let haut = KbInput { kind: 1, _p: 0, vk: 0x09, scan: 0, flags: 2, time: 0, extra: 0, _pad: [0; 8] }; SendInput(1, &bas, 40); SendInput(1, &haut, 40); } }

/// Vérifie la NAVIGATION AU CLAVIER d'une fenêtre (repérée par une partie de son titre) : on met la fenêtre au premier plan, on appuie sur Tab, et à chaque arrêt on lit ce que le lecteur dirait.
/// Mesure : arrêts, arrêts distincts, longueur du cycle (retour au premier arrêt), arrêts sans aucun nom, sorties de la fenêtre, couverture (arrêts distincts / contrôles interactifs focalisables vus).
fn parcours_tab(uia: &Uia, titre: &str, max: u32) { unsafe {
    let walker = el_ptr(uia.p, 14); let racine = el_ptr(uia.p, 5); if walker.is_null() || racine.is_null() { say("PARCOURS=indisponible"); return; }
    let mut cible: *mut c_void = std::ptr::null_mut();
    for _essai in 0..10 {
        let mut top = voisin_de(walker, racine, 4);
        while !top.is_null() { let suivant = voisin_de(walker, top, 6); if cible.is_null() && titre_correspond(&bstr_prop(top, 23), titre) { cible = top; } else { release(top); } top = suivant; }
        if !cible.is_null() { break; } Sleep(500);
    }
    if cible.is_null() { say(&format!("PARCOURS fenetre-introuvable titre~{titre}")); return; }
    let pid = el_int(cible, 20).unwrap_or(0);
    let hwnd: isize = { let f: extern "system" fn(*mut c_void, *mut isize) -> Hr = std::mem::transmute(vtbl(cible, 36)); let mut h = 0isize; f(cible, &mut h); h };
    // contrôles interactifs focalisables attendus
    let mut attendus = 0u32; let mut pids: Vec<i32> = vec![pid]; { let mut pile = vec![cible]; let mut v = 0; while let Some(e) = pile.pop() { v += 1; if let Some(p) = el_int(e, 20) { if !pids.contains(&p) { pids.push(p); } } if v < 800 { let mut c = voisin_de(walker, e, 4); let mut l = Vec::new(); while !c.is_null() && l.len() < 300 { l.push(c); c = voisin_de(walker, c, 6); } for x in l.into_iter().rev() { pile.push(x); } }
        let ty = el_int(e, 21).unwrap_or(0); let itinerant = [50007, 50024, 50019, 50011, 50029, 50013].contains(&ty); if INTERACTIFS.contains(&ty) && !itinerant && el_bool(e, 28) && el_bool(e, 27) && !el_bool(e, 38) { attendus += 1; } if e != cible { release(e); } } }
    if hwnd != 0 { SetForegroundWindow(hwnd); } Sleep(600);
    let (mut arrets, mut sans_nom, mut hors, mut cycle) = (0u32, 0u32, 0u32, 0u32); let mut vus: Vec<String> = Vec::new(); let mut distincts: Vec<String> = Vec::new();
    for i in 0..max {
        envoyer_tab(); Sleep(180);
        let el = el_ptr(uia.p, 8); if el.is_null() { continue; }
        let epid = el_int(el, 20).unwrap_or(-1); if !pids.contains(&epid) { hors += 1; release(el); if hors > 3 { break; } continue; }
        let f = uia.decrire(el); release(el); arrets += 1; if f.source == "aucun" { sans_nom += 1; }
        let cle = format!("{}|{}|{}", f.kind, f.name, f.source); if !distincts.contains(&cle) { distincts.push(cle.clone()); }
        if cycle == 0 && !vus.is_empty() && cle == vus[0] && i > 0 { cycle = arrets - 1; } vus.push(cle);
        if cycle > 0 && arrets >= cycle * 2 + 1 { break; }
    }
    let couv = if attendus > 0 { (distincts.len() as f64 / attendus as f64 * 100.0).min(100.0) } else { 0.0 };
    say(&format!("PARCOURS titre~{titre} attendus={attendus} arrets={arrets} distincts={} cycle={cycle} sans_nom={sans_nom} hors_fenetre={hors} couverture={couv:.0}%", distincts.len()));
} }
fn main() {
    let a: Vec<String> = std::env::args().collect(); let has = |n: &str| a.iter().any(|x| x == n);
    let val = |n: &str| a.iter().position(|x| x == n).and_then(|i| a.get(i + 1)).and_then(|s| s.parse::<u64>().ok());
    if has("--narrator") {
        if narrator_tourne() { say("NARRATOR=deja-lance"); return; }
        match std::process::Command::new("Narrator.exe").spawn() { Ok(_) => say("NARRATOR=lance"), Err(e) => say(&format!("NARRATOR=echec ({e})")) }
        return;
    }
    let Some(uia) = Uia::new() else { say("UIA=indisponible"); std::process::exit(2) };
    if let Some(i) = a.iter().position(|x| x == "--parcours-tab") { if let Some(t) = a.get(i + 1) { parcours_tab(&uia, t, val("--max").unwrap_or(60) as u32); return; } }
    if let Some(i) = a.iter().position(|x| x == "--audit-processus") { if let Some(exe) = a.get(i + 1) { let t = a.iter().position(|x| x == "--titre").and_then(|j| a.get(j + 1)).map(|s| s.as_str()); audit_processus(&uia, exe, t); return; } }
    let voix = if has("--parler") { Voix::new() } else { None };
    if has("--once") {
        match uia.focus() {
            Some(f) => { let p = phrase(&f); say(&format!("FOCUS={p}")); say(&format!("NOM={}", if f.password { String::new() } else { f.name.clone() })); say(&format!("NOMSOURCE={}", f.source)); say(&format!("TYPE={}", f.kind)); say(&format!("MOTDEPASSE={}", f.password)); if let Some(v) = &voix { v.dire(&p); v.attendre(10_000); } }
            None => { say("FOCUS=aucun"); std::process::exit(1) }
        }
        return;
    }
    if has("--veille") {
        let fin = val("--secondes").unwrap_or(0); let mut ecoule = 0u64; let (mut dernier, mut derniere_fenetre) = (String::new(), String::new());
        say(&format!("VEILLE=demarree voix={}", voix.is_some()));
        loop {
            let t = titre_fenetre_active();
            if !t.is_empty() && t != derniere_fenetre { derniere_fenetre = t.clone(); let m = format!("Fenêtre : {t}"); say(&m); if let Some(v) = &voix { v.dire(&m); } }
            if let Some(f) = uia.focus() { let p = phrase(&f); if p != dernier { say(&format!("FOCUS={p}")); if let Some(v) = &voix { v.dire(&p); } dernier = p; } }
            unsafe { Sleep(150); } ecoule += 150; if fin > 0 && ecoule >= fin * 1000 { break; }
        }
        if let Some(v) = &voix { v.attendre(5_000); } say("VEILLE=terminee"); return;
    }
    say("usage : voxa-companion --once [--parler] | --veille [--secondes N] [--parler] | --narrator | --audit-processus <exe>");
}

#[cfg(test)]
mod tests {
    use super::*;
    fn f(n: &str, k: &str, pw: bool) -> Focus { Focus { name: n.into(), source: "nom", kind: k.into(), password: pw, value: None, toggle: None, enabled: true } }
    #[test] fn phrases_are_name_then_type_and_never_leak_passwords() {
        assert_eq!(phrase(&f("Enregistrer", "bouton", false)), "Enregistrer, bouton");
        assert_eq!(phrase(&f("", "bouton", false)), "bouton sans nom"); assert_eq!(phrase(&f("Zone", "", false)), "Zone"); assert_eq!(phrase(&f("", "", false)), "élément sans nom");
        let mut p = f("Mot de passe : hunter2", "modifier", true); p.value = Some("hunter2".into()); assert_eq!(phrase(&p), "Champ de mot de passe", "ni nom ni valeur d'un mot de passe");
    }
    #[test] fn phrase_reads_state_value_and_availability() {
        let mut c = f("Mode sombre", "case à cocher", false); c.toggle = Some(1); assert_eq!(phrase(&c), "Mode sombre, case à cocher, coché");
        c.toggle = Some(0); c.enabled = false; assert_eq!(phrase(&c), "Mode sombre, case à cocher, non coché, indisponible");
        let mut e = f("Nom", "modifier", false); e.value = Some("  Camille ".into()); assert_eq!(phrase(&e), "Nom, modifier, valeur : Camille"); e.value = Some("".into()); assert_eq!(phrase(&e), "Nom, modifier", "valeur vide non dite");
    }
    #[test] fn unlabeled_controls_get_a_name_in_order_of_reliability() {
        assert_eq!(choisir_nom("OK", "aide", "etiq", "Id", "v", "p"), ("OK".into(), "nom"));
        assert_eq!(choisir_nom("", "Aide du bouton", "etiq", "Id", "v", "p"), ("Aide du bouton".into(), "aide"));
        assert_eq!(choisir_nom("", "", "Adresse", "x", "v", "p"), ("Adresse".into(), "etiquette"));
        assert_eq!(choisir_nom("", "", "", "SearchBox_Edit", "v", "p"), ("search box edit".into(), "identifiant"));
        assert_eq!(choisir_nom("", "", "", "1234", "Nom d'utilisateur", "Groupe"), ("Nom d'utilisateur".into(), "voisin"));
        assert_eq!(choisir_nom("", "", "", "", "", "Options avancées"), ("Options avancées".into(), "parent"));
        assert_eq!(choisir_nom("", "", "", "", "", ""), (String::new(), "aucun"));
    }
    #[test] fn technical_ids_are_translated_only_when_fully_known() {
        assert_eq!(traduire_id("part close button").as_deref(), Some("fermer")); assert_eq!(traduire_id("back").as_deref(), Some("retour")); assert_eq!(traduire_id("search box"), None, "« box » inconnu : on garde l'identifiant lisible");
        assert_eq!(traduire_id("more options").as_deref(), Some("plus d'options options")); assert_eq!(traduire_id("button"), None);
        assert_eq!(choisir_nom("", "", "", "PART_CloseButton", "", ""), ("fermer".into(), "identifiant")); assert_eq!(choisir_nom("", "", "", "SearchBox_Edit", "", "").0, "search box edit");
    }
    #[test] fn exact_title_match_avoids_false_positives() {
        assert!(titre_correspond("Son", "=Son")); assert!(titre_correspond("son", "=SON")); assert!(!titre_correspond("Réponse - Navigateur", "=Son")); assert!(titre_correspond("Réponse - Navigateur", "ponse")); assert!(!titre_correspond("Calculatrice", "Param"));
    }
    #[test] fn opaque_identifiers_are_not_spoken() {
        assert_eq!(humanise_id("{1A2B3C4D-0000-1111-2222-333344445555}"), ""); assert_eq!(humanise_id("3f2a9c1d77"), ""); assert_eq!(humanise_id("12345"), ""); assert_eq!(humanise_id(""), "");
        assert_eq!(humanise_id("SettingsPageTitle"), "settings page title"); assert_eq!(humanise_id("close.button"), "close button");
    }
}
