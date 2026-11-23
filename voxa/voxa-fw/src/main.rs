//! voxa-fw : analyse en LECTURE SEULE d'une image firmware UEFI. N'écrit rien dans l'image, n'exécute rien, ne touche à aucun matériel.
//! Usage : voxa-fw <image> [--chercher mot,mot] [--extraire dossier]
mod lzma;
mod tiano;
mod hii;
mod sha256;
mod exploit;
mod carte;
mod amdtypes;
use std::collections::BTreeMap;

fn le16(d: &[u8], o: usize) -> Option<usize> { Some(u16::from_le_bytes(d.get(o..o + 2)?.try_into().ok()?) as usize) }
fn le32(d: &[u8], o: usize) -> Option<usize> { Some(u32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?) as usize) }
fn le64(d: &[u8], o: usize) -> Option<u64> { Some(u64::from_le_bytes(d.get(o..o + 8)?.try_into().ok()?)) }
fn le24(d: &[u8], o: usize) -> Option<usize> { let b = d.get(o..o + 3)?; Some(b[0] as usize | (b[1] as usize) << 8 | (b[2] as usize) << 16) }
fn guid(d: &[u8]) -> String { format!("{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}", u32::from_le_bytes(d[0..4].try_into().unwrap()), u16::from_le_bytes(d[4..6].try_into().unwrap()), u16::from_le_bytes(d[6..8].try_into().unwrap()), d[8], d[9], d[10], d[11], d[12], d[13], d[14], d[15]) }
fn utf16(d: &[u8]) -> String { let v: Vec<u16> = d.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&c| c != 0).collect(); String::from_utf16_lossy(&v) }

const LZMA_GUID: &str = "EE4E5898-3914-4259-9D6E-DC7BD79403CF";
const LZMA_X86_GUID: &str = "D42AE6BD-1352-4BFB-909A-CA72A6EAE889";
const TIANO_GUID: &str = "A31280AD-481E-41B6-95E8-127F4C984779";

#[derive(Default)]
struct Stats { volumes: usize, files: usize, sections: usize, compressed_ok: usize, compressed_fail: BTreeMap<String, usize>, pe: usize, names: Vec<(String, String, usize)>, tailles: Vec<usize>, depex: Vec<(String, String, String)>, versions: Vec<(String, String)>, types_sections: BTreeMap<u8, usize>, depth_max: usize, blobs: Vec<(String, Vec<u8>)> }

fn type_name(t: u8) -> &'static str { match t { 1 => "RAW", 2 => "FREEFORM", 3 => "SECURITY_CORE", 4 => "PEI_CORE", 5 => "DXE_CORE", 6 => "PEIM", 7 => "DRIVER", 8 => "COMBINED_PEIM_DRIVER", 9 => "APPLICATION", 0xA => "SMM", 0xB => "VOLUME_IMAGE", 0xC => "COMBINED_SMM_DXE", 0xD => "SMM_CORE", 0xF0 => "FFS_PAD", _ => "?" } }

fn walk_sections(d: &[u8], file: &str, depth: usize, st: &mut Stats, ui: &mut String) {
    st.depth_max = st.depth_max.max(depth);
    let mut o = 0;
    while o + 4 <= d.len() {
        let Some(mut size) = le24(d, o) else { break };
        let ty = d[o + 3]; let mut hdr = 4;
        if size == 0xFFFFFF { size = le32(d, o + 4).unwrap_or(0); hdr = 8; }
        if size < hdr || o + size > d.len() { break; }
        let body = &d[o + hdr..o + size]; st.sections += 1; *st.types_sections.entry(ty).or_default() += 1;
        match ty {
            0x15 => *ui = utf16(body),
            0x10 | 0x12 => { st.pe += 1; st.blobs.push((format!("{}", if ui.is_empty() { file } else { ui.as_str() }), body.to_vec())); }
            0x17 => scan_volume(body, st, depth + 1),
            0x19 => { st.blobs.push((format!("{}.raw", if ui.is_empty() { file } else { ui.as_str() }), body.to_vec())); }
            0x14 => { if body.len() >= 2 { st.versions.push((if ui.is_empty() { file.to_string() } else { ui.clone() }, utf16(&body[2..]))); } }
            0x18 => { if body.len() > 16 { st.blobs.push((format!("{}.sous-type-{}.raw", if ui.is_empty() { file } else { ui.as_str() }, guid(&body[..16])), body[16..].to_vec())); } }
            0x13 | 0x1B | 0x1C => { let kind = match ty { 0x13 => "DXE", 0x1B => "PEI", _ => "SMM" }; st.depex.push((if ui.is_empty() { file.to_string() } else { ui.clone() }, kind.to_string(), decode_depex(body))); }
            0x01 => if body.len() >= 5 {
                let (ulen, ctype) = (le32(body, 0).unwrap_or(0), body[4]);
                if ctype == 0 { walk_sections(&body[5..], file, depth + 1, st, &mut *ui); st.compressed_ok += 1; }
                else if ctype == 1 { match tiano::decode_auto(&body[5..]) { Ok(out) if out.len() == ulen => { st.compressed_ok += 1; walk_sections(&out, file, depth + 1, st, &mut *ui) } Ok(out) => *st.compressed_fail.entry(format!("Tiano/EFI : taille {} au lieu de {}", out.len(), ulen)).or_default() += 1, Err(e) => *st.compressed_fail.entry(format!("Tiano/EFI : {}", e)).or_default() += 1 } }
                else { *st.compressed_fail.entry(format!("compression EFI type {} inconnu, {} octets", ctype, ulen)).or_default() += 1; }
            },
            0x02 => if body.len() >= 20 {
                let g = guid(&body[..16]); let off = le16(body, 16).unwrap_or(0);
                let data = body.get(off.saturating_sub(hdr)..).unwrap_or(&[]);
                if g == LZMA_GUID || g == LZMA_X86_GUID {
                    if data.len() >= 13 { let ul = le64(data, 5).unwrap_or(0) as usize;
                        match lzma::decode(&data[..5], &data[13..], ul) { Ok(out) => { st.compressed_ok += 1; walk_sections(&out, file, depth + 1, st, &mut *ui) } Err(e) => *st.compressed_fail.entry(format!("LZMA : {}", e)).or_default() += 1 } }
                } else if g == TIANO_GUID { match tiano::decode_auto(data) { Ok(out) => { st.compressed_ok += 1; walk_sections(&out, file, depth + 1, st, &mut *ui) } Err(e) => *st.compressed_fail.entry(format!("Tiano (GUID) : {}", e)).or_default() += 1 } }
                else { walk_sections(data, file, depth + 1, st, &mut *ui); }
            },
            _ => {}
        }
        o += (size + 3) & !3;
    }
}

fn scan_volume(d: &[u8], st: &mut Stats, depth: usize) {
    if d.len() < 0x38 || &d[0x28..0x2C] != b"_FVH" { return; }
    st.volumes += 1;
    let flen = (le64(d, 0x20).unwrap_or(0) as usize).min(d.len()); let hl = le16(d, 0x30).unwrap_or(0x48);
    let eo = le16(d, 0x34).unwrap_or(0); let mut o = (hl + 7) & !7;
    if eo != 0 { if let Some(es) = le32(d, eo + 16) { o = (eo + es + 7) & !7; } }
    while o + 24 <= flen {
        let name = &d[o..o + 16]; if name.iter().all(|&b| b == 0xFF) { break; }
        let ty = d[o + 18]; let attr = d[o + 19]; let mut size = le24(d, o + 20).unwrap_or(0); let mut hdr = 24;
        if attr & 1 != 0 { size = le64(d, o + 24).unwrap_or(0) as usize; hdr = 32; }
        if size < hdr || o + size > flen { break; }
        let g = guid(name); st.files += 1;
        if ty != 0xF0 && ty != 1 || ty == 1 {
            let mut ui = String::new(); walk_sections(&d[o + hdr..o + size], &g, depth, st, &mut ui);
            st.names.push((g, if ui.is_empty() { "-".into() } else { ui }, ty as usize)); st.tailles.push(size);
        }
        o = (o + size + 7) & !7;
    }
}

pub fn run(a: Vec<String>) {

    let Some(path) = a.get(1) else { eprintln!("usage : voxa-fw <image> [--chercher mot,mot] [--extraire dossier]"); std::process::exit(2) };
    let img = std::fs::read(path).unwrap_or_else(|e| { eprintln!("lecture impossible : {e}"); std::process::exit(1) });
    let mut st = Stats::default();
    let mut i = 0x28; while i + 4 <= img.len() { if &img[i..i + 4] == b"_FVH" { let s = i - 0x28; let fl = le64(&img, s + 0x20).unwrap_or(0) as usize; let hl = le16(&img, s + 0x30).unwrap_or(0); if (0x48..=0x400).contains(&hl) && fl >= hl && s + fl <= img.len() { scan_volume(&img[s..s + fl], &mut st, 0); i = s + fl + 0x28; continue; } } i += 1; }
    println!("Image : {} ({} octets)", path, img.len());
    println!("Volumes: {}  Fichiers FFS: {}  Sections: {}  Compressions décodées: {}  Exécutables PE/TE: {}  Profondeur max: {}", st.volumes, st.files, st.sections, st.compressed_ok, st.pe, st.depth_max);
    for (k, v) in &st.compressed_fail { println!("ÉCHEC/NON GÉRÉ ×{v} : {k}"); }
    let mut mots: Vec<String> = vec![]; let mut ext: Option<String> = None; let mut setup: Option<String> = None; let mut expl: Option<String> = None; let mut carte_sortie: Option<String> = None;
    let mut j = 2; while j < a.len() { match a[j].as_str() { "--chercher" => { if let Some(v) = a.get(j + 1) { mots = v.split(',').map(|s| s.to_lowercase()).collect(); } j += 1 } "--extraire" => { ext = a.get(j + 1).cloned(); j += 1 } "--setup" => { setup = a.get(j + 1).cloned(); j += 1 } "--exploiter" => { expl = a.get(j + 1).cloned(); j += 1 } "--carte" => { carte_sortie = a.get(j + 1).cloned(); j += 1 } _ => {} } j += 1; }
    if !mots.is_empty() { println!("--- modules dont le nom contient : {:?}", mots); for (g, n, t) in &st.names { let l = n.to_lowercase(); if mots.iter().any(|m| l.contains(m)) { println!("{g}  {}  {}", type_name(*t as u8), n); } } }
    else { println!("--- {} noms de modules (UI) ; utilisez --chercher pour filtrer", st.names.iter().filter(|x| x.1 != "-").count()); }
    if let Some(f) = setup { let mut r = hii::Resultat::default(); for (n, d) in exploit::nvar_liste(&img) { r.defauts.entry(n).or_insert(d); } for (n, b) in st.blobs.iter() { hii::extraire(b, n, &mut r); } hii::extraire(&img, "image", &mut r);
        println!("ECRANS DE CONFIGURATION : {} listes HII, {} pages, {} questions, {} lignes -> {f}", r.listes, r.formulaires, r.questions, r.lignes.len()); std::fs::write(&f, r.lignes.join("
")).ok(); std::fs::write(format!("{f}.hii"), &r.brut).ok(); }
    if let Some(f) = carte_sortie { let r = carte::cartographier(&img, &f); for l in r.lines().filter(|l| l.starts_with("COUVERTURE") || l.starts_with("CARTE")) { println!("{l}"); } }
    if let Some(dir) = expl {
        std::fs::create_dir_all(&dir).ok();
        let noms: Vec<exploit::Entree> = st.names.iter().zip(st.tailles.iter()).map(|((g, n, t), s)| exploit::Entree { guid: g.clone(), nom: n.clone(), ty: *t as u8, taille: *s }).collect();
        { let mut d = String::new(); for (n, k, e) in &st.depex { let _ = std::fmt::Write::write_fmt(&mut d, format_args!("{n} [{k}] : {e}\n")); } std::fs::write(format!("{dir}/dependances.txt"), d).ok();
          let mut t = String::new(); for (k, v) in &st.types_sections { let _ = std::fmt::Write::write_fmt(&mut t, format_args!("section type {:#04x} : {}\n", k, v)); } std::fs::write(format!("{dir}/sections-par-type.txt"), t).ok(); }
        { let mut v = String::new(); for (n, s) in &st.versions { if !s.is_empty() { let _ = std::fmt::Write::write_fmt(&mut v, format_args!("{n} : {s}\n")); } } std::fs::write(format!("{dir}/versions-des-modules.txt"), v).ok(); }
        let ni = exploit::images(&dir, &st.blobs); println!("IMAGES : {ni} images embarquees (extraites dans {dir}/images)");
        let nz = exploit::nvram(&dir, &img); println!("NVRAM : {nz} zone(s) de variables dans l'image");
        let na = exploit::entrees_amd(&dir, &img); println!("ENTREES AMD : {na} entrees (type, nom, taille, emplacement, en-tete signe, entropie, empreinte)");
        let (pm, pd) = exploit::protocoles(&dir, &st.blobs, &st.depex); println!("PROTOCOLES : {pm} modules utilisent {pd} identifiants publics connus");
        let m = exploit::modules(&dir, &noms); let e = exploit::executables(&dir, &st.blobs); let r = exploit::repertoires(&dir, &img);
        let mut src: Vec<(&str, &[u8])> = vec![("image", &img[..])]; for (n, b) in st.blobs.iter() { src.push((n.as_str(), b.as_slice())); }
        let t = exploit::acpi(&dir, &src); let (c1, c2) = exploit::chaines(&dir, &st.blobs);
        let mut rr = hii::Resultat::default(); for (n, d) in exploit::nvar_liste(&img) { rr.defauts.entry(n).or_insert(d); }
        for (n, b) in st.blobs.iter() { hii::extraire(b, n, &mut rr); } hii::extraire(&img, "image", &mut rr);
        std::fs::write(format!("{dir}/magasins-de-variables.txt"), rr.varstores.join("\n")).ok(); std::fs::write(format!("{dir}/ecrans-de-configuration.txt"), rr.lignes.join("\n")).ok(); std::fs::write(format!("{dir}/ecrans.hii"), &rr.brut).ok();
        println!("EXPLOITATION : {m} modules, {e} executables (empreintes), {r} repertoires AMD, {t} tables ACPI, {c1} chaines ({c2} utiles), {} magasins de variables, {} pages, {} questions -> {dir}", rr.varstores.len(), rr.formulaires, rr.questions);
    }
    if let Some(dir) = ext { std::fs::create_dir_all(&dir).ok(); for (k, (n, b)) in st.blobs.iter().enumerate() { let safe: String = n.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect(); let _ = std::fs::write(format!("{dir}/{k:04}_{safe}.efi"), b); } println!("{} exécutables extraits dans {dir}", st.blobs.len()); }
}

fn main() { run(std::env::args().collect()) }

/// Expression de dependances (depex) en texte : GUID requis (PUSH), operateurs, BEFORE/AFTER.
fn decode_depex(b: &[u8]) -> String {
    let mut o = 0; let mut s = Vec::new();
    while o < b.len() {
        match b[o] { 0x00 | 0x01 | 0x02 if o + 17 <= b.len() => { let nom = ["BEFORE", "AFTER", "PUSH"][b[o] as usize]; s.push(format!("{nom} {}", guid(&b[o + 1..o + 17]))); o += 17; }
            0x03 => { s.push("AND".into()); o += 1 } 0x04 => { s.push("OR".into()); o += 1 } 0x05 => { s.push("NOT".into()); o += 1 } 0x06 => { s.push("TRUE".into()); o += 1 } 0x07 => { s.push("FALSE".into()); o += 1 }
            0x08 => { s.push("END".into()); break; } 0x09 => { s.push("SOR".into()); o += 1 } _ => { s.push(format!("?{:02x}", b[o])); o += 1 } }
    }
    s.join(" ; ")
}
