//! Extraction des ecrans de configuration d'un firmware UEFI : listes de paquets HII (formulaires IFR + chaines), en LECTURE SEULE.
//! Sert a savoir exactement ce que dit et propose l'ecran de configuration, pour le faire parler sans modifier le firmware.
use std::collections::HashMap;

fn u16le(d: &[u8], o: usize) -> usize { d.get(o..o + 2).map_or(0, |x| u16::from_le_bytes([x[0], x[1]]) as usize) }
fn u32le(d: &[u8], o: usize) -> usize { d.get(o..o + 4).map_or(0, |x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]) as usize) }

pub struct Chaines { pub langue: String, pub textes: HashMap<usize, String> }

fn lire_chaines(p: &[u8]) -> Option<Chaines> {
    let info = u32le(p, 8); let mut l = 46; let mut langue = String::new(); while let Some(&c) = p.get(l) { if c == 0 { break; } langue.push(c as char); l += 1; if langue.len() > 32 { return None; } }
    if info < 46 || info >= p.len() { return None; }
    let (mut o, mut id, mut t) = (info, 1usize, HashMap::new());
    let scsu = |o: usize| -> (String, usize) { let mut e = o; while e < p.len() && p[e] != 0 { e += 1; } (p[o..e.min(p.len())].iter().map(|&b| b as char).collect(), e + 1) };
    let ucs2 = |o: usize| -> (String, usize) { let mut e = o; let mut v = Vec::new(); while e + 1 < p.len() { let c = u16::from_le_bytes([p[e], p[e + 1]]); e += 2; if c == 0 { break; } v.push(c); } (String::from_utf16_lossy(&v), e) };
    while o < p.len() {
        let ty = p[o]; o += 1;
        match ty {
            0x00 => break,
            0x10 => { let (s, n) = scsu(o); t.insert(id, s); id += 1; o = n; }
            0x11 => { let (s, n) = scsu(o + 1); t.insert(id, s); id += 1; o = n; }
            0x12 | 0x13 => { if ty == 0x13 { o += 1; } let c = u16le(p, o); o += 2; for _ in 0..c { let (s, n) = scsu(o); t.insert(id, s); id += 1; o = n; } }
            0x14 => { let (s, n) = ucs2(o); t.insert(id, s); id += 1; o = n; }
            0x15 => { let (s, n) = ucs2(o + 1); t.insert(id, s); id += 1; o = n; }
            0x16 | 0x17 => { if ty == 0x17 { o += 1; } let c = u16le(p, o); o += 2; for _ in 0..c { let (s, n) = ucs2(o); t.insert(id, s); id += 1; o = n; } }
            0x20 => { id += 1; }
            0x21 => { id += u16le(p, o); o += 2; }
            0x22 => { id += *p.get(o)? as usize; o += 1; }
            0x30 => { let l = *p.get(o + 1)? as usize; o += l.max(2) - 1; }
            0x31 => { let l = u16le(p, o + 1); o += l.max(3) - 1; }
            0x32 => { let l = u32le(p, o + 1); o += l.max(5) - 1; }
            0x40 | 0x41 | 0x42 => { return Some(Chaines { langue, textes: t }); }
            _ => break,
        }
    }
    Some(Chaines { langue, textes: t })
}

#[derive(Default)]
pub struct Resultat { pub defauts: std::collections::HashMap<String, Vec<u8>>, pub varstores: Vec<String>, pub brut: Vec<u8>, pub listes: usize, pub formulaires: usize, pub questions: usize, pub lignes: Vec<String> }

fn nom(ch: &Option<&Chaines>, id: usize) -> String { if id == 0 { return String::new(); } ch.and_then(|c| c.textes.get(&id)).map(|s| s.split_whitespace().collect::<Vec<_>>().join(" ")).unwrap_or_else(|| format!("<chaine {id}>")) }

fn lire_formulaires(p: &[u8], ch: &Option<&Chaines>, res: &mut Resultat, ctx: &str) {
    let mut stores: std::collections::HashMap<usize, (String, usize)> = std::collections::HashMap::new(); let mut courant: Option<u64> = None;
    let mut o = 4; let mut prof = 0i32;
    while o + 2 <= p.len() {
        let op = p[o]; let l = (p[o + 1] & 0x7F) as usize; let scope = p[o + 1] & 0x80 != 0; if l < 2 || o + l > p.len() { break; }
        let c = &p[o..o + l]; let ind = "  ".repeat(prof.max(0) as usize);
        let etiquette = |t: &str, prompt: usize, aide: usize| format!("{ind}{t} : {}{}", nom(ch, prompt), { let a = nom(ch, aide); if a.is_empty() { String::new() } else { format!(" -- {a}") } });
        match op {
            0x24 => { if l >= 22 { let mut nm = String::new(); for &b in &c[22..] { if b == 0 { break; } nm.push(b as char); } res.varstores.push(format!("magasin id {} taille {} octets nom {}", u16le(c, 18), u16le(c, 20), nm)); stores.insert(u16le(c, 18), (nm, u16le(c, 20))); } }
            0x0E => { res.lignes.push(format!("{ind}ENSEMBLE [{ctx}] : {}", nom(ch, u16le(c, 18)))); }
            0x01 => { res.formulaires += 1; res.lignes.push(format!("{ind}PAGE {} : {}", u16le(c, 2), nom(ch, u16le(c, 4)))); }
            0x02 => res.lignes.push(etiquette("Titre", u16le(c, 2), u16le(c, 4))),
            0x03 => res.lignes.push(etiquette("Texte", u16le(c, 2), u16le(c, 4)) + &format!(" = {}", nom(ch, u16le(c, 6)))),
            0x05 | 0x06 | 0x07 | 0x08 | 0x0C | 0x1A | 0x1B | 0x1C | 0x23 => { res.questions += 1; courant = None; let t = match op { 0x05 => "Choix", 0x06 => "Case", 0x07 => "Nombre", 0x08 => "Mot de passe", 0x0C => "Action", 0x1A => "Date", 0x1B => "Heure", 0x1C => "Texte libre", _ => "Liste ordonnee" };
                let mut ligne = etiquette(t, u16le(c, 2), u16le(c, 4));
                if matches!(op, 0x05 | 0x06 | 0x07) && l >= 14 { let (vs, off) = (u16le(c, 8), u16le(c, 10)); let taille = if op == 0x06 { 1 } else { 1usize << (c[13] & 3) };
                    if let Some((nm, _)) = stores.get(&vs) { if let Some(d) = res.defauts.get(nm) { if off + taille <= d.len() { let mut v = 0u64; for k in 0..taille { v |= (d[off + k] as u64) << (8 * k); } courant = Some(v);
                        match op { 0x06 => ligne += if v != 0 { " [par defaut : active]" } else { " [par defaut : desactive]" }, 0x07 => ligne += &format!(" [par defaut : {v}]"), _ => {} } } } } }
                res.lignes.push(ligne); }
            0x0F => { res.questions += 1; res.lignes.push(etiquette("Lien", u16le(c, 2), u16le(c, 4)) + &format!(" -> page {}", u16le(c, 13))); }
            0x09 => { let ty = if l > 5 { c[5] } else { 0 }; let n = match ty { 0 => 1, 1 => 2, 2 => 4, 3 => 8, _ => 0 }; let mut v = 0u64; for k in 0..n { v |= (*c.get(6 + k).unwrap_or(&0) as u64) << (8 * k); } let def = n > 0 && courant == Some(v);
                res.lignes.push(format!("{ind}  option : {}{}", nom(ch, u16le(c, 2)), if def { " [valeur par defaut]" } else { "" })); }
            _ => {}
        }
        if scope { prof += 1; } if op == 0x29 { prof -= 1; }
        o += l;
    }
}

/// Parcourt un blob a la recherche de listes de paquets HII valides (en-tete 20 octets + paquets terminees par 0xDF et de longueur exacte).
pub fn extraire(blob: &[u8], origine: &str, res: &mut Resultat) {
    if blob.len() < 44 { return; }
    let mut pos = 0;
    while pos + 44 <= blob.len() {
        let pl = u32le(blob, pos + 16);
        if (28..=0x400000).contains(&pl) && pos + pl <= blob.len() && matches!(blob[pos + 23], 1..=10 | 0xDF) {
            // marche stricte sur les paquets
            let (mut o, mut ok, mut paquets): (usize, bool, Vec<(u8, usize, usize)>) = (pos + 20, false, Vec::new());
            while o + 4 <= pos + pl { let h = u32le(blob, o); let l = h & 0xFFFFFF; let ty = (h >> 24) as u8; if l < 4 || o + l > pos + pl || !matches!(ty, 1..=10 | 0xDF | 0xE0..=0xFF) { break; } paquets.push((ty, o, l)); o += l; if ty == 0xDF { ok = o == pos + pl; break; } }
            if ok && paquets.iter().any(|p| p.0 == 2) {
                res.listes += 1; res.brut.extend_from_slice(&blob[pos..pos + pl]); let mut tous: Vec<Chaines> = Vec::new();
                for &(ty, o, l) in &paquets { if ty == 4 { if let Some(c) = lire_chaines(&blob[o..o + l]) { tous.push(c); } } }
                let choix = tous.iter().find(|c| c.langue.starts_with("fr")).or_else(|| tous.iter().find(|c| c.langue.starts_with("en"))).or_else(|| tous.first());
                let ctx = format!("{origine}@{pos:#x}{}", choix.map_or(String::new(), |c| format!(" langue {} {} chaines", c.langue, c.textes.len())));
                for &(ty, o, l) in &paquets { if ty == 2 { lire_formulaires(&blob[o..o + l], &choix, res, &ctx); } }
                pos += pl; continue;
            }
        }
        pos += 1;
    }
}
