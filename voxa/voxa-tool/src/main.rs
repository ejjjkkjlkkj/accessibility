//! voxa-tool : outil universel maison, LECTURE SEULE. Ne modifie ni disque, ni firmware, ni pilote ; n'installe rien ; n'exécute rien d'extérieur.
//!   voxa-tool fw <image> [...]            analyse d'une image firmware UEFI (voir voxa-fw)
//!   voxa-tool img <image>                 inspection d'une image ISO/disque (voir voxa-img)
//!   voxa-tool pilotes <peripheriques.csv> [dossier-inf] [plan.md]
//!                                         croise le matériel présent avec (1) les pilotes que Windows utilise, (2) les INF locaux, (3) NOS pilotes ;
//!                                         écrit le plan de pilotes Voxa à créer, classé par importance pour un utilisateur aveugle
#[path = "../../voxa-fw/src/main.rs"] #[allow(dead_code)] mod fw;
#[path = "../../voxa-img/src/main.rs"] #[allow(dead_code)] mod img;
use std::collections::{BTreeMap, BTreeSet};

/// État de NOS pilotes (voxa-live-rs / voxa-kernel). Seule source de vérité : cette table ; à tenir à jour avec les tests.
struct Ours { name: &'static str, status: &'static str, patterns: &'static [&'static str] }
const OURS: &[Ours] = &[
    Ours { name: "HDA (audio haute définition)", status: "PARTIAL : pilote Rust écrit et exécuté seulement dans le modèle émulé ; jamais entendu sur le vrai matériel", patterns: &["HDAUDIO\\FUNC_01", "ACP\\DEVTYPE_0001", "PCI\\VEN_1022&DEV_1457", "PCI\\VEN_1022&DEV_15E3"] },
    Ours { name: "Clavier 8042 / PS2", status: "PARTIAL : lecture par UEFI (ConIn) ; accès direct aux ports 0x60/0x64 testé en émulation seulement", patterns: &["ACPI\\ATK3001", "ACPI\\PNP0303", "*PNP0303"] },
    Ours { name: "SMBIOS (identité de la machine)", status: "PASS sur l'image de cette machine (valeurs identiques à Windows) ; lecture via le firmware", patterns: &[] },
    Ours { name: "Haut-parleur PC (bips de secours)", status: "PARTIAL : ports 0x42/0x43/0x61, émulé", patterns: &["ACPI\\PNP0800"] },
];

#[derive(Default, Clone)]
struct Dev { class: String, name: String, state: String, ids: Vec<String>, driver: String }

fn csv_line(l: &str) -> Vec<String> {
    let (mut out, mut cur, mut q) = (vec![], String::new(), false);
    let mut it = l.chars().peekable();
    while let Some(c) = it.next() { match c { '"' if q && it.peek() == Some(&'"') => { cur.push('"'); it.next(); } '"' => q = !q, ',' if !q => out.push(std::mem::take(&mut cur)), _ => cur.push(c) } }
    out.push(cur); out
}

/// Lit un INF (UTF-16 ou ANSI) et renvoie ses identifiants matériels (PCI\, ACPI\, USB\, HID\, HDAUDIO\, ACP\, *XXXX) en majuscules.
fn inf_ids(path: &std::path::Path) -> Vec<String> {
    let Ok(b) = std::fs::read(path) else { return vec![] };
    let t = if b.len() > 2 && b[0] == 0xFF && b[1] == 0xFE { let v: Vec<u16> = b[2..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect(); String::from_utf16_lossy(&v) } else { String::from_utf8_lossy(&b).into_owned() };
    let mut ids = vec![];
    for line in t.lines() { let l = line.split(';').next().unwrap_or(""); if let Some((_, rhs)) = l.split_once('=') { for part in rhs.split(',').skip(1) { let p = part.trim().to_uppercase(); for pre in ["PCI\\", "ACPI\\", "USB\\", "HID\\", "HDAUDIO\\", "ACP\\", "*"] { if p.starts_with(pre) && p.len() > pre.len() + 2 { ids.push(p.clone()); break; } } } } }
    ids
}

fn importance(class: &str, name: &str) -> (u8, &'static str) {
    let c = class.to_lowercase(); let n = name.to_lowercase();
    if c.contains("media") || n.contains("audio") || n.contains("hda") { (1, "voix : sans audio, pas de lecteur d'écran") }
    else if c.contains("keyboard") || c.contains("clavier") || n.contains("clavier") || n.contains("keyboard") { (1, "entrée : le clavier est l'outil principal") }
    else if c.contains("hid") || n.contains("pavé") || n.contains("touchpad") || n.contains("hid") { (2, "entrée : pointeur et braille passent par HID") }
    else if c.contains("usb") || n.contains("usb") || n.contains("xhci") { (2, "entrée : claviers, afficheurs braille, clés de démarrage") }
    else if c.contains("disk") || c.contains("hdc") || c.contains("scsi") || n.contains("nvme") || n.contains("stockage") { (2, "fichiers : sans stockage, pas de système utilisable") }
    else if c.contains("net") || c.contains("réseau") || n.contains("wi-fi") || n.contains("ethernet") || n.contains("wireless") { (3, "réseau : courrier, web, mises à jour") }
    else if c.contains("display") || c.contains("écran") || n.contains("graphi") || n.contains("radeon") || n.contains("amd") && n.contains("gpu") { (3, "affichage : utile aux malvoyants et au dépannage ; non vital pour un aveugle") }
    else if c.contains("battery") || c.contains("batterie") || n.contains("battery") || n.contains("acpi") || n.contains("alimentation") { (3, "énergie : annoncer la batterie, éteindre proprement") }
    else { (4, "autre : plus tard") }
}

fn pilotes(args: &[String]) -> i32 {
    let Some(csv) = args.get(0) else { eprintln!("usage : voxa-tool pilotes <peripheriques.csv> [dossier-inf] [plan.md]"); return 2 };
    let inf_dir = args.get(1).map(String::as_str).unwrap_or("C:\\Windows\\INF"); let plan = args.get(2);
    let Ok(txt) = std::fs::read_to_string(csv) else { eprintln!("lecture impossible : {csv}"); return 2 };
    let mut devs = vec![]; for (i, l) in txt.lines().enumerate() { if i == 0 || l.trim().is_empty() { continue; } let f = csv_line(l); if f.len() < 8 { continue; }
        devs.push(Dev { class: f[0].clone(), name: f[1].clone(), state: f[2].clone(), ids: f[6].split('|').map(|s| s.trim().to_uppercase()).filter(|s| !s.is_empty()).collect(), driver: f[7].clone() }); }
    let mut inf: BTreeMap<String, String> = BTreeMap::new(); let mut n_inf = 0;
    if let Ok(rd) = std::fs::read_dir(inf_dir) { for e in rd.flatten() { let p = e.path(); if p.extension().map(|x| x.eq_ignore_ascii_case("inf")).unwrap_or(false) { n_inf += 1; let nm = p.file_name().unwrap().to_string_lossy().into_owned(); for id in inf_ids(&p) { inf.entry(id).or_insert(nm.clone()); } } } }
    println!("{} périphériques lus, {} fichiers INF analysés dans {inf_dir} ({} identifiants matériels indexés)", devs.len(), n_inf, inf.len());
    let (mut with_inf, mut with_ours, mut rows): (usize, usize, Vec<(u8, String)>) = (0, 0, vec![]);
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for d in &devs {
        if d.name.is_empty() { continue; }
        let inf_hit = d.ids.iter().find_map(|i| inf.get(i).map(|f| f.clone())); if inf_hit.is_some() { with_inf += 1; }
        let ours = OURS.iter().find(|o| d.ids.iter().any(|i| o.patterns.iter().any(|p| i.starts_with(&p.to_uppercase()))));
        if ours.is_some() { with_ours += 1; }
        let (prio, why) = importance(&d.class, &d.name);
        let key = format!("{}|{}", d.name, ours.map(|o| o.name).unwrap_or("-")); if !seen.insert(key) { continue; }
        let st = match ours { Some(o) => o.status.to_string(), None => "AUCUN pilote Voxa".to_string() };
        rows.push((prio, format!("| {} | {} | {} | {} | {} | {} |", prio, d.name.replace('|', "/"), if d.driver.is_empty() { "-" } else { d.driver.as_str() }, inf_hit.as_deref().unwrap_or("aucun INF local"), st, why)));
    }
    rows.sort_by_key(|r| r.0);
    println!("Avec un INF local : {with_inf} ; avec un pilote Voxa (au moins partiel) : {with_ours} ; lignes distinctes : {}", rows.len());
    for p in 1..=4u8 { println!("Priorité {p} : {} lignes", rows.iter().filter(|r| r.0 == p).count()); }
    if let Some(pl) = plan {
        let mut o = String::from("# Plan de pilotes Voxa (généré par `voxa-tool pilotes`, lecture seule)\nSource : audit de la machine (Windows) croisé avec les INF locaux et la table de NOS pilotes (`voxa-tool/src/main.rs`, `OURS`).\nStatuts : un pilote Voxa n'est jamais PASS sur matériel tant qu'il n'a pas été essayé sur la machine.\nPriorité 1 = indispensable à un utilisateur aveugle ; 2 = très important ; 3 = important ; 4 = plus tard.\n\n| Prio | Périphérique | Pilote Windows | INF Windows local | Pilote Voxa | Pourquoi |\n|---|---|---|---|---|---|\n");
        for r in &rows { o.push_str(&r.1); o.push('\n'); }
        o.push_str("\n**Méthode pour créer un pilote Voxa** : (1) lire les ressources réelles (ACPI/DSDT, PCI) ; (2) écrire le pilote contre le trait `Driver` de `voxa-kernel` ; (3) le tester dans l'émulateur avec un modèle de registres ; (4) l'annoncer par `Voice` ; (5) seulement alors, l'essayer sur la machine depuis une clé. Les pilotes de l'éditeur (INF/SYS) ne sont jamais copiés ni chargés : ils servent de référence d'identifiants uniquement.\n");
        match std::fs::write(pl, o) { Ok(_) => println!("Plan écrit : {pl}"), Err(e) => { eprintln!("écriture impossible : {e}"); return 1 } }
    }
    0
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("fw") => fw::run(a[1..].to_vec()),
        Some("img") => img::run(a[1..].to_vec()),
        Some("pilotes") => std::process::exit(pilotes(&a[2..])),
        _ => { eprintln!("voxa-tool : fw <image> | img <image> | pilotes <csv> [dossier-inf] [plan.md]"); std::process::exit(2) }
    }
}
