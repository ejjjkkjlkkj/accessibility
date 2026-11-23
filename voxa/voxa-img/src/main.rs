//! voxa-img : inspection en LECTURE SEULE d'une image (ISO 9660 + El Torito, MBR, GPT, FAT d'amorçage). N'écrit rien, ne monte rien.
//! Usage : voxa-img <image>    Code de sortie : 0 = démarrable UEFI vérifié (structure seulement), 1 = problème, 2 = usage.
//! Ce que cela prouve : la STRUCTURE de l'image. Cela ne prouve pas qu'un firmware réel la démarre (NOT_VALIDATED).
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn rd(f: &mut File, off: u64, n: usize) -> Option<Vec<u8>> { let mut b = vec![0u8; n]; f.seek(SeekFrom::Start(off)).ok()?; f.read_exact(&mut b).ok()?; Some(b) }
fn u16le(b: &[u8], o: usize) -> u16 { u16::from_le_bytes([b[o], b[o + 1]]) }
fn u32le(b: &[u8], o: usize) -> u32 { u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) }
fn crc32(d: &[u8]) -> u32 { let mut c = 0xFFFF_FFFFu32; for &b in d { c ^= b as u32; for _ in 0..8 { c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 }; } } !c }
fn text(b: &[u8]) -> String { String::from_utf8_lossy(b).trim_end_matches(|c| c == ' ' || c == '\0').to_string() }

struct Report { problems: Vec<String>, uefi_boot: bool }

/// Lit un volume FAT (BPB) à `base` et cherche EFI/BOOT/BOOTX64.EFI. Ne gère que FAT12/16/32 en lecture, noms 8.3 et noms longs ignorés.
fn inspect_fat(f: &mut File, base: u64, r: &mut Report) {
    let Some(b) = rd(f, base, 512) else { r.problems.push("FAT : secteur 0 illisible".into()); return };
    if b[510] != 0x55 || b[511] != 0xAA { r.problems.push("FAT : signature 55AA absente".into()); return; }
    let bps = u16le(&b, 11) as u64; let spc = b[13] as u64; let rsv = u16le(&b, 14) as u64; let nfat = b[16] as u64;
    let rootent = u16le(&b, 17) as u64; let tot16 = u16le(&b, 19) as u64; let spf16 = u16le(&b, 22) as u64; let tot32 = u32le(&b, 32) as u64; let spf32 = u32le(&b, 36) as u64;
    if bps == 0 || spc == 0 || nfat == 0 { r.problems.push("FAT : BPB invalide".into()); return; }
    let spf = if spf16 != 0 { spf16 } else { spf32 }; let tot = if tot16 != 0 { tot16 } else { tot32 };
    let rootsec = (rootent * 32 + bps - 1) / bps; let data0 = rsv + nfat * spf + rootsec;
    let clusters = (tot.saturating_sub(data0)) / spc; let kind = if clusters < 4085 { "FAT12" } else if clusters < 65525 { "FAT16" } else { "FAT32" };
    println!("  FAT : {kind}, {bps} octets/secteur, {spc} secteurs/cluster, {tot} secteurs, {clusters} clusters");
    let fat = rd(f, base + rsv * bps, (spf * bps) as usize).unwrap_or_default();
    let next = |c: u32| -> u32 { match kind { "FAT12" => { let o = (c as usize * 3) / 2; if o + 1 >= fat.len() { return 0xFFF; } let v = u16le(&fat, o) as u32; if c & 1 == 1 { v >> 4 } else { v & 0xFFF } } "FAT16" => { let o = c as usize * 2; if o + 1 >= fat.len() { 0xFFFF } else { u16le(&fat, o) as u32 } } _ => { let o = c as usize * 4; if o + 3 >= fat.len() { 0x0FFF_FFFF } else { u32le(&fat, o) & 0x0FFF_FFFF } } } };
    let eoc = |c: u32| match kind { "FAT12" => c >= 0xFF8, "FAT16" => c >= 0xFFF8, _ => c >= 0x0FFF_FFF8 };
    let csize = (spc * bps) as usize;
    let read_chain = |f: &mut File, start: u32, limit: usize| -> Vec<u8> { let mut out = vec![]; let mut c = start; let mut guard = 0; while c >= 2 && !eoc(c) && out.len() < limit && guard < 1 << 20 { if let Some(d) = rd(f, base + (data0 + (c as u64 - 2) * spc) * bps, csize) { out.extend(d) } else { break } c = next(c); guard += 1; } out };
    let mut dir: Vec<u8> = if kind == "FAT32" { read_chain(f, u32le(&b, 44), 1 << 20) } else { rd(f, base + (rsv + nfat * spf) * bps, (rootsec * bps) as usize).unwrap_or_default() };
    let mut size = 0usize; let mut cluster = 0u32; let mut path_ok = true;
    for comp in ["EFI", "BOOT", "BOOTX64.EFI"] {
        let want = if let Some((n, e)) = comp.split_once('.') { format!("{:<8}{:<3}", n, e) } else { format!("{:<11}", comp) };
        let mut found = None;
        for e in dir.chunks_exact(32) { if e[0] == 0 { break; } if e[0] == 0xE5 || e[11] == 0x0F { continue; } if &e[..11] == want.as_bytes() { found = Some((u16le(e, 26) as u32 | (u16le(e, 20) as u32) << 16, u32le(e, 28) as usize, e[11] & 0x10 != 0)); break; } }
        let Some((c, s, isdir)) = found else { r.problems.push(format!("FAT : {comp} absent")); path_ok = false; break };
        cluster = c; size = s; if isdir { dir = read_chain(f, c, 1 << 20); }
    }
    if path_ok {
        let data = read_chain(f, cluster, size); let ok_pe = data.len() >= size && size > 0x40 && &data[..2] == b"MZ";
        let machine = if ok_pe { let pe = u32le(&data, 0x3C) as usize; if data.len() > pe + 6 && &data[pe..pe + 4] == b"PE\0\0" { u16le(&data, pe + 4) } else { 0 } } else { 0 };
        println!("  EFI/BOOT/BOOTX64.EFI : {size} octets, en-tête MZ={ok_pe}, machine PE=0x{machine:04X}");
        if ok_pe && machine == 0x8664 { r.uefi_boot = true } else { r.problems.push("BOOTX64.EFI n'est pas un exécutable PE x86-64".into()) }
    }
}

pub fn run(a: Vec<String>) {
    if extras(&a) { return; }
    let Some(path) = a.get(1).cloned() else { eprintln!("usage : voxa-img <image>"); std::process::exit(2) };
    let mut f = File::open(&path).unwrap_or_else(|e| { eprintln!("ouverture impossible : {e}"); std::process::exit(2) });
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    println!("Image : {path} ({len} octets)");
    let mut r = Report { problems: vec![], uefi_boot: false };
    // MBR
    if let Some(m) = rd(&mut f, 0, 512) {
        if m[510] == 0x55 && m[511] == 0xAA { println!("MBR : signature 55AA présente"); for i in 0..4 { let e = &m[446 + i * 16..462 + i * 16]; if e[4] != 0 { println!("  partition {} : type 0x{:02X}, début LBA {}, {} secteurs, active={}", i + 1, e[4], u32le(e, 8), u32le(e, 12), e[0] == 0x80); } } } else { println!("MBR : absent (normal pour une ISO pure)"); }
    }
    // GPT
    if let Some(g) = rd(&mut f, 512, 92) { if &g[..8] == b"EFI PART" {
        println!("GPT : en-tête présent, {} entrées", u32le(&g, 80));
        // validation complète : CRC de l'en-tête, CRC du tableau, en-tête de secours, puis lecture de chaque partition système EFI (FAT)
        let mut h = g.clone(); let crc_lu = u32le(&h, 16); h[16..20].copy_from_slice(&[0; 4]);
        if crc32(&h) != crc_lu { r.problems.push("GPT : CRC32 de l'en-tête incorrect".into()) }
        let (n, taille, lba) = (u32le(&g, 80) as usize, u32le(&g, 84) as usize, u64::from_le_bytes(g[72..80].try_into().unwrap()));
        let secours = u64::from_le_bytes(g[32..40].try_into().unwrap());
        match rd(&mut f, lba * 512, n * taille) {
            Some(t) => {
                if crc32(&t) != u32le(&g, 88) { r.problems.push("GPT : CRC32 du tableau d'entrées incorrect".into()) }
                for i in 0..n { let e = &t[i * taille..(i + 1) * taille]; if e[..16] == [0u8; 16] { continue; }
                    let (deb, fin) = (u64::from_le_bytes(e[32..40].try_into().unwrap()), u64::from_le_bytes(e[40..48].try_into().unwrap()));
                    let esp = e[..16] == [0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11, 0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B];
                    println!("  GPT partition {} : {} secteurs {}..{}{}", i + 1, fin - deb + 1, deb, fin, if esp { " (système EFI)" } else { "" });
                    if esp { inspect_fat(&mut f, deb * 512, &mut r) } }
            }
            None => r.problems.push("GPT : tableau d'entrées illisible".into()),
        }
        match rd(&mut f, secours * 512, 92) { Some(b) if &b[..8] == b"EFI PART" => { let mut bh = b.clone(); let c = u32le(&bh, 16); bh[16..20].copy_from_slice(&[0; 4]); if crc32(&bh) != c { r.problems.push("GPT de secours : CRC32 incorrect".into()) } else { println!("GPT de secours : valide"); } } _ => r.problems.push("GPT de secours absente".into()) }
    } else { println!("GPT : absente"); } }
    // ISO 9660
    let mut iso = false; let mut pvd_blocks = 0u64;
    if let Some(p) = rd(&mut f, 16 * 2048, 2048) { if &p[1..6] == b"CD001" && p[0] == 1 { iso = true; pvd_blocks = u32le(&p, 80) as u64; println!("ISO 9660 : volume « {} », {} blocs de 2048 ({} octets)", text(&p[40..72]), pvd_blocks, pvd_blocks * 2048); if pvd_blocks * 2048 > len { r.problems.push("ISO : taille déclarée supérieure au fichier (image tronquée)".into()); } } }
    if !iso { println!("ISO 9660 : descripteur absent"); }
    // El Torito
    let mut sector = 17u64; let mut catalog = None;
    while iso && sector < 64 { let Some(d) = rd(&mut f, sector * 2048, 2048) else { break }; if &d[1..6] != b"CD001" { break; } if d[0] == 0 && text(&d[7..39]).starts_with("EL TORITO SPECIFICATION") { catalog = Some(u32le(&d, 71) as u64); } if d[0] == 255 { break; } sector += 1; }
    if let Some(c) = catalog {
        println!("El Torito : catalogue au bloc {c}");
        let cat = rd(&mut f, c * 2048, 2048).unwrap_or_default();
        if cat.len() == 2048 && cat[0] == 1 && cat[30] == 0x55 && cat[31] == 0xAA {
            let sum: u32 = cat[..32].chunks(2).map(|x| u16le(x, 0) as u32).sum(); if sum & 0xFFFF != 0 { r.problems.push("El Torito : somme de contrôle du catalogue incorrecte".into()); }
            let mut plat = cat[1]; let mut i = 32;
            while i + 32 <= cat.len() {
                let e = &cat[i..i + 32];
                match e[0] { 0x88 | 0x00 => { if e[0] == 0x88 { let rba = u32le(e, 8) as u64; let cnt = u16le(e, 6); let media = e[1] & 0xF;
                    println!("  entrée amorçable : plateforme 0x{:02X} ({}), média {}, {} secteurs de 512, bloc {}", plat, match plat { 0 => "x86 BIOS", 0xEF => "UEFI", _ => "autre" }, media, cnt, rba);
                    if plat == 0xEF { if media == 0 { inspect_fat(&mut f, rba * 2048, &mut r) } else { r.problems.push("entrée UEFI : média non « sans émulation »".into()) } } } else { break } }
                    0x90 | 0x91 => { plat = e[1]; } _ => break }
                i += 32;
            }
        } else { r.problems.push("El Torito : entrée de validation invalide".into()); }
    } else if iso { println!("El Torito : aucun catalogue (non démarrable en CD)"); }
    if !r.uefi_boot && r.problems.is_empty() { r.problems.push("aucune entrée UEFI démarrable trouvée".into()); }
    for p in &r.problems { println!("PROBLÈME : {p}"); }
    println!("Résultat : {}", if r.problems.is_empty() && r.uefi_boot { "PASS (structure UEFI valide ; non testé sur firmware réel)" } else { "FAIL" });
    std::process::exit(if r.problems.is_empty() && r.uefi_boot { 0 } else { 1 });
}

fn main() { run(std::env::args().collect()) }

// ---------------------------------------------------------------- listages (lecture seule)
/// Liste récursivement un arbre ISO 9660 (noms ISO, sans extension Joliet/Rock Ridge). Borné en profondeur et en nombre d'entrées.
fn liste_iso(f: &mut File, extent: u32, size: u32, indent: usize, budget: &mut usize) {
    if indent > 12 || *budget == 0 { return; }
    let Some(d) = rd(f, extent as u64 * 2048, size as usize) else { return };
    let mut o = 0;
    while o < d.len() {
        let l = d[o] as usize; if l == 0 { o = (o / 2048 + 1) * 2048; continue; } if o + l > d.len() || l < 34 { break; }
        let ext = u32le(&d, o + 2); let sz = u32le(&d, o + 10); let dir = d[o + 25] & 2 != 0; let nl = d[o + 32] as usize;
        let nm = if nl == 1 && d[o + 33] <= 1 { None } else { Some(String::from_utf8_lossy(&d[o + 33..o + 33 + nl.min(l - 33)]).into_owned()) };
        if let Some(n) = nm { if *budget == 0 { return; } *budget -= 1; println!("{}{}{}  ({} octets, bloc {})", "  ".repeat(indent + 1), n, if dir { "/" } else { "" }, sz, ext); if dir { liste_iso(f, ext, sz, indent + 1, budget); } }
        o += l;
    }
}

/// Liste récursivement un volume FAT (noms courts 8.3 + noms longs assemblés) à partir de `base`.
fn liste_fat(f: &mut File, base: u64) {
    let Some(b) = rd(f, base, 512) else { println!("FAT illisible"); return };
    let bps = u16le(&b, 11) as u64; let spc = b[13] as u64; let rsv = u16le(&b, 14) as u64; let nfat = b[16] as u64; let rootent = u16le(&b, 17) as u64;
    let spf = if u16le(&b, 22) != 0 { u16le(&b, 22) as u64 } else { u32le(&b, 36) as u64 }; let tot = if u16le(&b, 19) != 0 { u16le(&b, 19) as u64 } else { u32le(&b, 32) as u64 };
    if bps == 0 || spc == 0 || nfat == 0 || b[510] != 0x55 || b[511] != 0xAA { println!("Pas de volume FAT valide à cet endroit"); return; }
    let rootsec = (rootent * 32 + bps - 1) / bps; let data0 = rsv + nfat * spf + rootsec; let clusters = tot.saturating_sub(data0) / spc;
    let kind = if clusters < 4085 { 12 } else if clusters < 65525 { 16 } else { 32 };
    println!("Volume FAT{kind} : {tot} secteurs de {bps} octets, étiquette « {} »", text(&b[if kind == 32 { 71 } else { 43 }..if kind == 32 { 82 } else { 54 }]));
    let fat = rd(f, base + rsv * bps, (spf * bps) as usize).unwrap_or_default();
    let next = |c: u32| -> u32 { match kind { 12 => { let o = (c as usize * 3) / 2; if o + 1 >= fat.len() { return 0xFFF; } let v = u16le(&fat, o) as u32; if c & 1 == 1 { v >> 4 } else { v & 0xFFF } } 16 => { let o = c as usize * 2; if o + 1 >= fat.len() { 0xFFFF } else { u16le(&fat, o) as u32 } } _ => { let o = c as usize * 4; if o + 3 >= fat.len() { 0x0FFF_FFFF } else { u32le(&fat, o) & 0x0FFF_FFFF } } } };
    let eoc = |c: u32| match kind { 12 => c >= 0xFF8, 16 => c >= 0xFFF8, _ => c >= 0x0FFF_FFF8 };
    let csize = (spc * bps) as usize;
    let chain = |f: &mut File, start: u32| -> Vec<u8> { let mut out = vec![]; let mut c = start; let mut g = 0; while c >= 2 && !eoc(c) && g < 1 << 16 { if let Some(d) = rd(f, base + (data0 + (c as u64 - 2) * spc) * bps, csize) { out.extend(d) } else { break } c = next(c); g += 1; } out };
    let root = if kind == 32 { chain(f, u32le(&b, 44)) } else { rd(f, base + (rsv + nfat * spf) * bps, (rootsec * bps) as usize).unwrap_or_default() };
    let mut budget = 400usize; let mut stack = vec![(root, 0usize)];
    while let Some((dir, ind)) = stack.pop() {
        let mut lfn: Vec<u16> = vec![]; let mut subs = vec![];
        for e in dir.chunks_exact(32) {
            if e[0] == 0 { break; } if e[0] == 0xE5 { lfn.clear(); continue; }
            if e[11] == 0x0F { let mut part = vec![]; for r in [1usize, 3, 5, 7, 9, 14, 16, 18, 20, 22, 24, 28, 30] { part.push(u16le(e, r)); } let mut cur: Vec<u16> = part.into_iter().take_while(|&c| c != 0 && c != 0xFFFF).collect(); cur.extend(lfn.drain(..)); lfn = cur; continue; }
            if e[11] & 8 != 0 { lfn.clear(); continue; }
            let short = format!("{}{}{}", text(&e[..8]), if text(&e[8..11]).is_empty() { "" } else { "." }, text(&e[8..11]));
            let name = if lfn.is_empty() { short.clone() } else { String::from_utf16_lossy(&lfn) }; lfn.clear();
            if name == "." || name == ".." || short == "." || short == ".." { continue; }
            let dirf = e[11] & 0x10 != 0; let cl = u16le(e, 26) as u32 | (u16le(e, 20) as u32) << 16; let sz = u32le(e, 28);
            if budget == 0 { println!("(liste tronquée à 400 entrées)"); return; } budget -= 1;
            println!("{}{}{}  ({} octets)", "  ".repeat(ind + 1), name, if dirf { "/" } else { "" }, sz);
            if dirf && cl >= 2 { subs.push((chain(f, cl), ind + 1)); }
        }
        for s in subs.into_iter().rev() { stack.push(s); }
    }
}

fn extras(a: &[String]) -> bool {
    let Some(path) = a.get(1) else { return false };
    let mut f = match File::open(path) { Ok(f) => f, Err(_) => return false };
    match a.get(2).map(String::as_str) {
        Some("--liste") => { let Some(p) = rd(&mut f, 16 * 2048, 2048) else { return true }; if &p[1..6] != b"CD001" { println!("pas d'ISO 9660"); return true; } println!("Arborescence ISO 9660 :"); let mut budget = 500; liste_iso(&mut f, u32le(&p, 158), u32le(&p, 166), 0, &mut budget); true }
        Some("--fat") => { let off: u64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(0); liste_fat(&mut f, off); true }
        _ => false,
    }
}
