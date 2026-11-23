//! Cartographie octet par octet d'une image firmware : a quel "couche" appartient chaque octet de la memoire flash ?
//! Couches : structure AMD (EFS), repertoires et entrees PSP/BIOS (amorce, SMU, APCB, microcode...), volumes UEFI (en-tetes, fichiers par type, espace libre),
//! zones vides (0xFF / 0x00), reste non classe. LECTURE SEULE. Le taux de couverture est un fait mesure, pas une estimation.
use std::collections::BTreeMap;
use std::fmt::Write as _;

fn le16(d: &[u8], o: usize) -> usize { d.get(o..o + 2).map_or(0, |x| u16::from_le_bytes([x[0], x[1]]) as usize) }
fn le32(d: &[u8], o: usize) -> usize { d.get(o..o + 4).map_or(0, |x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]) as usize) }
fn le64(d: &[u8], o: usize) -> u64 { d.get(o..o + 8).map_or(0, |x| u64::from_le_bytes(x.try_into().unwrap())) }
fn le24(d: &[u8], o: usize) -> usize { d.get(o..o + 3).map_or(0, |b| b[0] as usize | (b[1] as usize) << 8 | (b[2] as usize) << 16) }

struct Carte { owner: Vec<u16>, noms: Vec<String>, index: BTreeMap<String, u16> }
impl Carte {
    fn new(n: usize) -> Carte { Carte { owner: vec![0; n], noms: vec!["non classe".into()], index: BTreeMap::new() } }
    fn id(&mut self, nom: &str) -> u16 { if let Some(&i) = self.index.get(nom) { return i; } let i = self.noms.len() as u16; self.noms.push(nom.into()); self.index.insert(nom.into(), i); i }
    /// Premier arrive, premier servi : un octet deja classe garde sa couche (les couches les plus precises sont peintes d'abord).
    fn peint(&mut self, a: usize, b: usize, nom: &str) { let b = b.min(self.owner.len()); if a >= b { return; } let id = self.id(nom); for x in &mut self.owner[a..b] { if *x == 0 { *x = id; } } }
}
fn type_ffs(t: u8) -> &'static str { match t { 1 => "RAW", 2 => "FREEFORM", 3 => "SECURITY_CORE", 4 => "PEI_CORE", 5 => "DXE_CORE", 6 => "PEIM", 7 => "DRIVER", 8 => "COMBINED_PEIM_DRIVER", 9 => "APPLICATION", 0xA => "SMM", 0xB => "FIRMWARE_VOLUME_IMAGE", 0xC => "COMBINED_SMM_DXE", 0xD => "SMM_CORE", 0xE => "MM_STANDALONE", 0xF => "MM_CORE_STANDALONE", 0xF0 => "PAD", _ => "AUTRE" } }
fn nom_psp(t: u8, bios: bool) -> &'static str {
    if bios { match t { 0x60 => "APCB", 0x61 => "APOB", 0x62 => "image BIOS (reinitialisation)", 0x63 => "APOB NV", 0x64 => "PMU code", 0x65 => "PMU donnees", 0x66 => "microcode CPU", 0x68 => "APCB de secours", 0x70 => "pointeur repertoire niveau 2", _ => "type inconnu" } }
    else { match t { 0x00 => "cle publique AMD", 0x01 => "amorce du PSP", 0x02 => "OS de confiance du PSP", 0x03 => "donnees NV du PSP", 0x08 => "firmware SMU", 0x12 => "firmware SMU (2)", 0x40 => "pointeur repertoire niveau 2", _ => "type inconnu" } }
}
/// Convertit une adresse d'entree AMD en decalage dans l'image (decalage direct, ou adresse physique mappee sous 4 Go).
pub fn decalage(loc: u64, n: usize, base: usize) -> Option<usize> { let l = loc & 0x3FFF_FFFF_FFFF_FFFF; let flash = n - base; if (l as usize) < flash && l < 0x8000_0000 { Some(base + l as usize) } else if (0x8000_0000..=0xFFFF_FFFF).contains(&l) { let back = (0x1_0000_0000u64 - l) as usize; if back <= flash { Some(base + flash - back) } else { None } } else { None } }

/// Les fichiers de mise a jour AMI sont une capsule : un en-tete (GUID 4A3CA68B-7723-48FB-803D-578CC1FEC44D, signature comprise) suivi de la memoire flash complete (puissance de deux). Retourne la taille de l'en-tete.
pub fn base_flash(img: &[u8]) -> usize { let n = img.len(); let mut p = 1usize; while p * 2 <= n { p *= 2; } let extra = n - p; if extra > 0 && extra <= 0x10000 && img.len() > 16 && img[..16] == [0x8b, 0xa6, 0x3c, 0x4a, 0x23, 0x77, 0xfb, 0x48, 0x80, 0x3d, 0x57, 0x8c, 0xc1, 0xfe, 0xc4, 0x4d] { extra } else { 0 } }

pub fn cartographier(img: &[u8], sortie: &str) -> String {
    let n = img.len(); let mut c = Carte::new(n); let mut rap = String::new();
    let base = base_flash(img);
    if base > 0 { c.peint(0, base, "en-tete de capsule de mise a jour AMI (GUID, tailles, signature, remplissage)"); let _ = writeln!(rap, "Capsule AMI : en-tete de {base:#x} octets puis memoire flash de {} octets ; les adresses AMD sont relatives au debut de la flash (decalage {base:#x}).", n - base); }
    let mut differe: Vec<(usize, usize, String)> = Vec::new();
    // 1. repertoires AMD et leurs entrees
    for cookie in [&b"$PSP"[..], b"$BHD", b"$PL2", b"$BL2"] {
        let mut i = 0;
        while i + 16 <= n {
            if &img[i..i + 4] == cookie { let cnt = le32(img, i + 8);
                if (1..=96).contains(&cnt) && i + 16 + cnt * (if cookie == b"$BHD" || cookie == b"$BL2" { 24 } else { 16 }) <= n {
                    let bios = cookie == b"$BHD" || cookie == b"$BL2"; let pas = if bios { 24 } else { 16 }; let cs = String::from_utf8_lossy(cookie).to_string();
                    for k in 0..cnt { let e = i + 16 + k * pas; let t = img[e]; let size = le32(img, e + 4); let loc = le64(img, e + 8);
                        if size != 0 && size != 0xFFFF_FFFF { if let Some(o) = decalage(loc, n, base) { c.peint(o, o + size, &format!("AMD {cs} : {} (type {:#04x})", super::amdtypes::nom(t as u32, bios), t)); } } }
                    c.peint(i, i + 16 + cnt * pas, &format!("AMD repertoire {cs} (en-tete et table d'entrees)"));
                } }
            i += 1;
        }
    }
    // 2b. structure de demarrage AMD (EFS) : signature 55 AA 55 AA a un multiple de 0x400 depuis le debut de la flash
    { let mut i = base; while i + 0x40 <= n { if le32(img, i) == 0x55AA_55AA {
        c.peint(i, i + 0x60, "AMD EFS (pointeurs de demarrage)");
        let noms = ["signature", "IMC", "GEC", "XHCI", "PSP (ancien)", "PSP combo", "BIOS 0", "BIOS 1", "BIOS 2", "reserve", "reserve", "reserve", "Promontory", "LP Promontory"];
        let _ = writeln!(rap, "EFS @ {:#x} (flash {:#x}) :", i, i - base);
        for (k, nm) in noms.iter().enumerate() { let v = le32(img, i + 4 * k); if v != 0 && v != 0xFFFF_FFFF { let _ = writeln!(rap, "   {nm} = {v:#x}{}", if k == 0 { String::new() } else { format!(" (fichier {:#x})", v + base) }); } }
      } i += 0x400; } }
    // 2c. repertoires combo (2PSP / 2BHD) : cookie, somme, nombre d'entrees, puis (selecteur, identifiant, adresse) par entree
    for cookie in [&b"2PSP"[..], b"2BHD"] { let mut i = 0; while i + 16 <= n { if &img[i..i + 4] == cookie { let cnt = le32(img, i + 8); if (1..=8).contains(&cnt) && i + 32 + cnt * 16 <= n {
        c.peint(i, i + 32 + cnt * 16, &format!("AMD repertoire combo {}", String::from_utf8_lossy(cookie)));
        for k in 0..cnt { let e = i + 32 + k * 16; let _ = writeln!(rap, "combo {} entree {} : selecteur {:#x} identifiant {:#x} adresse {:#x}", String::from_utf8_lossy(cookie), k, le32(img, e), le32(img, e + 4), le64(img, e + 8)); }
    } } i += 1; } }
    // 2d. micrologiciel du controleur embarque (EC) stocke dans la flash : repere par sa chaine de version, etendu jusqu'aux plages de 0xFF
    { let mut i = 0; while i + 8 <= n { if &img[i..i + 4] == b"ITE8" { let mut a = i; while a > 1024 && img[a - 1024..a].iter().any(|&b| b != 0xFF) { a -= 1; } while a < i && img[a] == 0xFF { a += 1; }
            let mut b = i; while b + 1024 < n && img[b..b + 1024].iter().any(|&x| x != 0xFF) { b += 1; } while b > i && img[b - 1] == 0xFF { b -= 1; }
            c.peint(a, b, "micrologiciel du controleur embarque (ITE)");
            let mut v = Vec::new(); let mut p = a; while p + 12 < b { if &img[p..p + 2] == b"F0" && img[p + 2..p + 8].iter().all(|x| x.is_ascii_digit()) { let mut e = p; while e < b && img[e] >= 32 && img[e] < 127 { e += 1; } let t = String::from_utf8_lossy(&img[p..e]).to_string(); if !v.contains(&t) { v.push(t); } p = e; } else { p += 1; } }
            let nom_ec = { let mut e = i; while e < n && img[e] >= 32 && img[e] < 127 { e += 1; } String::from_utf8_lossy(&img[i..e]).to_string() };
            let _ = writeln!(rap, "EC : {} octets de {:#x} a {:#x}, identifiant '{nom_ec}', versions : {}", b - a, a, b, v.join(" ; ")); break; } i += 1; } }
    // 3. volumes UEFI de premier niveau : en-tete, fichiers par type, espace libre
    let mut i = 0x28;
    while i + 4 <= n {
        if &img[i..i + 4] == b"_FVH" { let s = i - 0x28; let fl = le64(img, s + 0x20) as usize; let hl = le16(img, s + 0x30);
            if (0x48..=0x400).contains(&hl) && fl >= hl && s + fl <= n {
                c.peint(s, s + hl, "volume UEFI : en-tete"); let eo = le16(img, s + 0x34); let mut o = (hl + 7) & !7; if eo != 0 { let es = le32(img, s + eo + 16); o = (eo + es + 7) & !7; }
                if eo != 0 { c.peint(s + hl, s + o, "volume UEFI : en-tete etendu"); }
                while o + 24 <= fl { let name = &img[s + o..s + o + 16]; if name.iter().all(|&b| b == 0xFF) { break; }
                    let ty = img[s + o + 18]; let attr = img[s + o + 19]; let mut size = le24(img, s + o + 20); let mut hdr = 24; if attr & 1 != 0 { size = le64(img, s + o + 24) as usize; hdr = 32; }
                    if size < hdr || o + size > fl { break; }
                    let sig = if ty == 0xF0 { "remplissage".to_string() } else { type_ffs(ty).to_string() };
                    c.peint(s + o, s + o + hdr, "volume UEFI : en-tetes de fichiers"); c.peint(s + o + hdr, s + o + size, &format!("volume UEFI : fichiers {sig}"));
                    o = (o + size + 7) & !7; }
                c.peint(s + o.min(fl), s + fl, "volume UEFI : espace libre ou fin de volume"); i = s + fl + 0x28; continue; } }
        i += 1;
    }
    for (a, b, nm) in differe { c.peint(a, b, &nm); }
    // 4. zones vides (0xFF, 0x00) d'au moins 256 octets non classees
    for (val, nom) in [(0xFFu8, "espace vide (0xFF)"), (0x00u8, "zeros")] { let mut a = 0; while a < n { if c.owner[a] == 0 && img[a] == val { let mut b = a; while b < n && c.owner[b] == 0 && img[b] == val { b += 1; } if b - a >= 256 { c.peint(a, b, nom); } a = b; } else { a += 1; } } }
    // 4b. ce qui reste : on le decrit au mieux, sans pretendre le decoder (en-tetes signes, APCB, remplissage, donnees a forte entropie, structure non decodee)
    { let mut a = 0; while a < n { if c.owner[a] == 0 { let mut b = a; while b < n && c.owner[b] == 0 { b += 1; }
            let s = &img[a..b]; let mut h = [0usize; 256]; for &x in s { h[x as usize] += 1; } let l = s.len() as f64; let ent: f64 = h.iter().filter(|&&q| q > 0).map(|&q| { let p = q as f64 / l; -p * p.log2() }).sum();
            let uniforme = h.iter().filter(|&&q| q > 0).count() == 1; let m4 = if s.len() >= 4 { &s[..4] } else { &s[..] };
            let nom = if uniforme { "remplissage d'alignement (valeur unique)" } else if m4 == b"2PSP" || m4 == b"2BHD" || m4 == b"$PS1" { "AMD en-tete signe (PSP/BIOS)" } else if m4 == b"APCB" { "AMD APCB (en-tete de bloc de configuration)" }
                else if ent >= 7.0 { "non decode : forte entropie (signature, cle, contenu compresse ou chiffre)" } else if ent < 1.0 { "non decode : structure creuse (tables, pointeurs, marges)" } else { "non decode : structure ou donnees AMD adjacentes aux entrees" };
            c.peint(a, b, nom); a = b; } else { a += 1; } } }
    // detail : chaque plage non decodee avec ses voisines (pour la decoder structurellement)
    { let mut d = String::new(); let mut a = 0;
      while a < n { let lab = c.noms[c.owner[a] as usize].clone(); let mut b = a; while b < n && c.owner[b] == c.owner[a] { b += 1; }
        if lab.starts_with("non decode") || lab.starts_with("remplissage") || lab.starts_with("AMD en-tete") || lab.starts_with("AMD APCB (en-tete") {
            let prev = if a > 0 { c.noms[c.owner[a - 1] as usize].as_str() } else { "(debut)" }; let next = if b < n { c.noms[c.owner[b] as usize].as_str() } else { "(fin)" };
            let hex: String = img[a..(a + 16).min(b)].iter().map(|x| format!("{x:02x}")).collect(); let asc: String = img[a..(a + 16).min(b)].iter().map(|&x| if (32..127).contains(&x) { x as char } else { '.' }).collect();
            let _ = writeln!(d, "{:#09x}-{:#09x} {:>7} | {} | {} | {} | avant: {} | apres: {}", a, b, b - a, lab, hex, asc, prev, next); }
        a = b; }
      std::fs::write(format!("{sortie}.detail"), d).ok(); }
    // 5. rapport
    let mut tot: BTreeMap<u16, usize> = BTreeMap::new(); for &o in &c.owner { *tot.entry(o).or_default() += 1; }
    let _ = writeln!(rap, "CARTE DE L'IMAGE : {} octets", n);
    let mut v: Vec<(usize, &String)> = tot.iter().map(|(k, &b)| (b, &c.noms[*k as usize])).collect(); v.sort_by(|a, b| b.0.cmp(&a.0));
    for (b, nom) in &v { let _ = writeln!(rap, "{:>10} octets  {:>6.2} %  {}", b, 100.0 * *b as f64 / n as f64, nom); }
    let non = *tot.get(&0).unwrap_or(&0); let flou: usize = tot.iter().filter(|(k, _)| { let s = c.noms[**k as usize].as_str(); s.contains("inconnu") || s.starts_with("non decode") || s == "non classe" }).map(|(_, b)| *b).sum();
    let _ = writeln!(rap, "COUVERTURE : {:.2} % des octets rattaches a une couche ({} sans couche) ; IDENTIFIES PRECISEMENT : {:.2} % ; non decodes ou de type inconnu : {} octets ({:.2} %)", 100.0 * (n - non) as f64 / n as f64, non, 100.0 * (n - flou) as f64 / n as f64, flou, 100.0 * flou as f64 / n as f64);
    // plages non classees : on les decrit (taille, entropie, motifs) pour qu'aucune ne reste muette
    let _ = writeln!(rap, "PLAGES NON CLASSEES :"); let mut a = 0; let mut nb = 0;
    while a < n { if c.owner[a] == 0 { let mut b = a; while b < n && c.owner[b] == 0 { b += 1; }
            let mut h = [0usize; 256]; for &x in &img[a..b] { h[x as usize] += 1; } let l = (b - a) as f64; let ent: f64 = h.iter().filter(|&&q| q > 0).map(|&q| { let p = q as f64 / l; -p * p.log2() }).sum();
            let tete: String = img[a..(a + 8).min(b)].iter().map(|x| format!("{x:02x}")).collect(); nb += 1;
            if nb <= 60 { let _ = writeln!(rap, "   {:#09x}-{:#09x} ({:>8} octets) entropie {:.2} bits/octet debut {}", a, b, b - a, ent, tete); } a = b; } else { a += 1; } }
    if nb > 60 { let _ = writeln!(rap, "   ... {} plages de plus", nb - 60); }
    std::fs::write(sortie, &rap).ok(); rap
}
