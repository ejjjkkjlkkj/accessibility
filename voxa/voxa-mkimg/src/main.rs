//! voxa-mkimg : fabrique une image disque UEFI demarrable, de zero (aucune bibliotheque, aucun outil Microsoft).
//! Contenu : MBR de protection, GPT primaire et de secours (CRC32), une partition « systeme EFI » alignee a 1 Mio, formatee en FAT32 (2 FAT, FSInfo, secteur de secours).
//! Usage : voxa-mkimg --sortie img [--taille-mio 64] [--vhd] CHEMIN=FICHIER [CHEMIN=FICHIER ...]
//!   CHEMIN : chemin dans la partition, composants en 8.3 majuscules (ex. EFI/BOOT/BOOTX64.EFI). --vhd ajoute un pied de page VHD fixe (pour monter l'image en lecture seule sous Windows et la verifier).
//! Ce que cela prouve : la STRUCTURE de l'image (verifiee ensuite par voxa-img et par le pilote FAT de Windows). Cela ne prouve PAS qu'un firmware reel la demarre.
use std::collections::BTreeMap;
use std::fs;
mod iso;

const SECTEUR: usize = 512;
const DEBUT_PART: u64 = 2048;

fn crc32(d: &[u8]) -> u32 { let mut c = 0xFFFF_FFFFu32; for &b in d { c ^= b as u32; for _ in 0..8 { c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 }; } } !c }
fn p16(b: &mut [u8], o: usize, v: u16) { b[o..o + 2].copy_from_slice(&v.to_le_bytes()); }
fn p32(b: &mut [u8], o: usize, v: u32) { b[o..o + 4].copy_from_slice(&v.to_le_bytes()); }
fn p64(b: &mut [u8], o: usize, v: u64) { b[o..o + 8].copy_from_slice(&v.to_le_bytes()); }

/// Arbre de fichiers a ecrire.
#[derive(Default)]
struct Dossier { sous: BTreeMap<String, Dossier>, fichiers: BTreeMap<String, Vec<u8>> }

fn nom83(s: &str) -> Result<[u8; 11], String> {
    let (n, e) = match s.rsplit_once('.') { Some((n, e)) => (n, e), None => (s, "") };
    let ok = |t: &str, max: usize| !t.is_empty() || max == 3 && t.is_empty();
    if n.is_empty() || n.len() > 8 || e.len() > 3 || !ok(n, 8) { return Err(format!("nom non 8.3 : {s}")); }
    let valide = |c: char| c.is_ascii_uppercase() || c.is_ascii_digit() || "_-$~!#%&@^".contains(c);
    if !n.chars().all(valide) || !e.chars().all(valide) { return Err(format!("caracteres non autorises (majuscules, chiffres, _-) dans : {s}")); }
    let mut r = [b' '; 11]; r[..n.len()].copy_from_slice(n.as_bytes()); r[8..8 + e.len()].copy_from_slice(e.as_bytes()); Ok(r)
}

struct Fat { spc: usize, rsvd: usize, nfat: usize, fatsz: usize, data: usize, nclus: usize, fat: Vec<u32>, zone: Vec<u8> /* zone de donnees */ }
impl Fat {
    fn alloue(&mut self, n: usize) -> Result<usize, String> {
        let besoin = n.max(1); let mut premier = 0; let mut prec = 0;
        let mut trouves = 0;
        for c in 2..self.nclus + 2 { if self.fat[c] == 0 { self.fat[c] = 0x0FFF_FFFF; if premier == 0 { premier = c; } else { self.fat[prec] = c as u32; } prec = c; trouves += 1; if trouves == besoin { return Ok(premier); } } }
        Err("partition trop petite".into())
    }
    fn ecrit_chaine(&mut self, premier: usize, d: &[u8]) {
        let t = self.spc * SECTEUR; let mut c = premier; let mut i = 0;
        loop { let o = (c - 2) * t; let m = (d.len() - i).min(t); self.zone[o..o + m].copy_from_slice(&d[i..i + m]); i += m; let s = self.fat[c] as usize; if s >= 0x0FFF_FFF8 || i >= d.len() { break; } c = s; }
    }
    fn entree(nom: [u8; 11], attr: u8, clus: usize, taille: u32) -> [u8; 32] {
        let mut e = [0u8; 32]; e[..11].copy_from_slice(&nom); e[11] = attr;
        let date: u16 = ((2026 - 1980) << 9) | (10 << 5) | 12;
        p16(&mut e, 14, 0); p16(&mut e, 16, date); p16(&mut e, 18, date); p16(&mut e, 20, (clus >> 16) as u16); p16(&mut e, 22, 0); p16(&mut e, 24, date); p16(&mut e, 26, clus as u16); p32(&mut e, 28, taille); e
    }
    /// Ecrit un dossier (et recursivement ses contenus) ; renvoie son premier cluster. `parent` = cluster du parent (0 pour la racine).
    fn dossier(&mut self, d: &Dossier, parent: usize, est_racine: bool) -> Result<usize, String> {
        let nent = d.sous.len() + d.fichiers.len() + if est_racine { 1 } else { 2 };
        let octets = nent * 32 + 32; let clus = self.alloue((octets + self.spc * SECTEUR - 1) / (self.spc * SECTEUR))?;
        let mut tab: Vec<u8> = Vec::new();
        if est_racine { tab.extend_from_slice(&Self::entree(*b"VOXA-ESP   ", 0x08, 0, 0)); }
        else { tab.extend_from_slice(&Self::entree(*b".          ", 0x10, clus, 0)); tab.extend_from_slice(&Self::entree(*b"..         ", 0x10, parent, 0)); }
        for (n, s) in &d.sous { let c = self.dossier(s, clus, false)?; tab.extend_from_slice(&Self::entree(nom83(n)?, 0x10, c, 0)); }
        for (n, f) in &d.fichiers {
            let c = if f.is_empty() { 0 } else { let c = self.alloue((f.len() + self.spc * SECTEUR - 1) / (self.spc * SECTEUR))?; self.ecrit_chaine(c, f); c };
            tab.extend_from_slice(&Self::entree(nom83(n)?, 0x20, c, f.len() as u32));
        }
        self.ecrit_chaine(clus, &tab); Ok(clus)
    }
}

pub(crate) fn guid_esp() -> [u8; 16] { [0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11, 0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B] }

fn construire(arbre: &Dossier, taille_mio: u64) -> Result<Vec<u8>, String> {
    let total: u64 = taille_mio * 2048; // secteurs
    if total < 2048 * 40 { return Err("taille minimale 40 Mio (FAT32 exige au moins 65 525 clusters)".into()); }
    let part = ((total - 34 - DEBUT_PART) / 2048) * 2048; // secteurs, multiple de 1 Mio
    let spc = 1usize; let rsvd = 32usize; let nfat = 2usize;
    // taille d'une FAT : iteration
    let mut fatsz = 1usize; loop { let data = part as usize - rsvd - nfat * fatsz; let nclus = data / spc; let besoin = ((nclus + 2) * 4 + SECTEUR - 1) / SECTEUR; if besoin <= fatsz { break; } fatsz = besoin; }
    let data = part as usize - rsvd - nfat * fatsz; let nclus = data / spc;
    if nclus < 65_525 { return Err(format!("{nclus} clusters seulement : FAT32 en exige 65 525")); }
    let mut f = Fat { spc, rsvd, nfat, fatsz, data, nclus, fat: vec![0; nclus + 2], zone: vec![0; nclus * spc * SECTEUR] };
    f.fat[0] = 0x0FFF_FFF8; f.fat[1] = 0x0FFF_FFFF;
    let racine = f.dossier(arbre, 0, true)?; if racine != 2 { return Err("la racine doit etre le cluster 2".into()); }
    let libres = f.fat.iter().skip(2).filter(|&&x| x == 0).count();
    // secteur de demarrage FAT32
    let mut bs = vec![0u8; SECTEUR];
    bs[0..3].copy_from_slice(&[0xEB, 0x58, 0x90]); bs[3..11].copy_from_slice(b"VOXA 1.0");
    p16(&mut bs, 11, SECTEUR as u16); bs[13] = spc as u8; p16(&mut bs, 14, rsvd as u16); bs[16] = nfat as u8; p16(&mut bs, 17, 0); p16(&mut bs, 19, 0); bs[21] = 0xF8; p16(&mut bs, 22, 0);
    p16(&mut bs, 24, 63); p16(&mut bs, 26, 255); p32(&mut bs, 28, DEBUT_PART as u32); p32(&mut bs, 32, part as u32);
    p32(&mut bs, 36, fatsz as u32); p16(&mut bs, 40, 0); p16(&mut bs, 42, 0); p32(&mut bs, 44, 2); p16(&mut bs, 48, 1); p16(&mut bs, 50, 6);
    bs[64] = 0x80; bs[66] = 0x29; p32(&mut bs, 67, 0x564F_5841); bs[71..82].copy_from_slice(b"VOXA-ESP   "); bs[82..90].copy_from_slice(b"FAT32   "); bs[510] = 0x55; bs[511] = 0xAA;
    let mut fsinfo = vec![0u8; SECTEUR]; p32(&mut fsinfo, 0, 0x4161_5252); p32(&mut fsinfo, 484, 0x6141_7272); p32(&mut fsinfo, 488, libres as u32); p32(&mut fsinfo, 492, 0xFFFF_FFFF); p32(&mut fsinfo, 508, 0xAA55_0000);
    // volume
    let mut vol = vec![0u8; part as usize * SECTEUR];
    vol[..SECTEUR].copy_from_slice(&bs); vol[SECTEUR..2 * SECTEUR].copy_from_slice(&fsinfo);
    vol[6 * SECTEUR..7 * SECTEUR].copy_from_slice(&bs); vol[7 * SECTEUR..8 * SECTEUR].copy_from_slice(&fsinfo);
    let mut fatb = vec![0u8; fatsz * SECTEUR]; for (i, v) in f.fat.iter().enumerate() { fatb[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes()); }
    for k in 0..nfat { let o = (rsvd + k * fatsz) * SECTEUR; vol[o..o + fatb.len()].copy_from_slice(&fatb); }
    let od = (f.rsvd + f.nfat * f.fatsz) * SECTEUR; let _ = f.data; vol[od..od + f.zone.len()].copy_from_slice(&f.zone);
    // disque
    let mut img = vec![0u8; total as usize * SECTEUR];
    // MBR de protection
    img[446 + 1] = 0x00; img[446 + 2] = 0x02; img[446 + 3] = 0x00; img[446 + 4] = 0xEE; img[446 + 5] = 0xFF; img[446 + 6] = 0xFF; img[446 + 7] = 0xFF;
    p32(&mut img, 446 + 8, 1); p32(&mut img, 446 + 12, (total - 1).min(0xFFFF_FFFF) as u32); img[510] = 0x55; img[511] = 0xAA;
    // identifiants derives du contenu (deterministes : meme entree, meme image)
    let graine = crc32(&vol[..65536.min(vol.len())]); let mut disque_guid = [0u8; 16]; let mut part_guid = [0u8; 16];
    for i in 0..16 { disque_guid[i] = (graine.rotate_left(i as u32 * 3) ^ 0xA5A5_5A5A ^ (i as u32 * 0x1F)) as u8; part_guid[i] = (graine.rotate_left(i as u32 * 5 + 1) ^ 0x3C3C_C3C3 ^ (i as u32 * 0x2B)) as u8; }
    disque_guid[7] = (disque_guid[7] & 0x0F) | 0x40; disque_guid[8] = (disque_guid[8] & 0x3F) | 0x80; part_guid[7] = (part_guid[7] & 0x0F) | 0x40; part_guid[8] = (part_guid[8] & 0x3F) | 0x80;
    let mut ent = vec![0u8; 128 * 128];
    ent[0..16].copy_from_slice(&guid_esp()); ent[16..32].copy_from_slice(&part_guid); p64(&mut ent, 32, DEBUT_PART); p64(&mut ent, 40, DEBUT_PART + part - 1); p64(&mut ent, 48, 0);
    for (i, c) in "VOXA EFI".encode_utf16().enumerate() { p16(&mut ent, 56 + i * 2, c); }
    let crc_ent = crc32(&ent);
    let entete = |courant: u64, secours: u64, lba_ent: u64| -> Vec<u8> {
        let mut h = vec![0u8; SECTEUR]; h[0..8].copy_from_slice(b"EFI PART"); p32(&mut h, 8, 0x0001_0000); p32(&mut h, 12, 92); p64(&mut h, 24, courant); p64(&mut h, 32, secours);
        p64(&mut h, 40, 34); p64(&mut h, 48, total - 34); h[56..72].copy_from_slice(&disque_guid); p64(&mut h, 72, lba_ent); p32(&mut h, 80, 128); p32(&mut h, 84, 128); p32(&mut h, 88, crc_ent);
        let c = crc32(&h[..92]); p32(&mut h, 16, c); h
    };
    img[SECTEUR..2 * SECTEUR].copy_from_slice(&entete(1, total - 1, 2));
    img[2 * SECTEUR..2 * SECTEUR + ent.len()].copy_from_slice(&ent);
    let o = (total as usize - 33) * SECTEUR; img[o..o + ent.len()].copy_from_slice(&ent);
    let o = (total as usize - 1) * SECTEUR; img[o..o + SECTEUR].copy_from_slice(&entete(total - 1, 1, total - 33));
    let o = DEBUT_PART as usize * SECTEUR; img[o..o + vol.len()].copy_from_slice(&vol);
    Ok(img)
}

/// Pied de page VHD fixe (specification Microsoft VHD 1.0) : permet de monter l'image en lecture seule sous Windows pour la verifier avec le pilote FAT de Windows.
fn pied_vhd(taille: u64, id: u32) -> Vec<u8> {
    let mut p = vec![0u8; 512]; p[0..8].copy_from_slice(b"conectix"); p[8..12].copy_from_slice(&2u32.to_be_bytes()); p[12..16].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    p[16..24].copy_from_slice(&u64::MAX.to_be_bytes()); p[24..28].copy_from_slice(&0u32.to_be_bytes()); p[28..32].copy_from_slice(b"voxa"); p[32..36].copy_from_slice(&0x0001_0000u32.to_be_bytes()); p[36..40].copy_from_slice(b"Wi2k");
    p[40..48].copy_from_slice(&taille.to_be_bytes()); p[48..56].copy_from_slice(&taille.to_be_bytes());
    // geometrie CHS (annexe de la specification)
    let mut secteurs = (taille / 512).min(65535 * 16 * 255); let (c, h, s);
    if secteurs >= 65535 * 16 * 63 { secteurs = 65535 * 16 * 63; s = 255; h = 16; c = secteurs / s / h; }
    else { let mut ss = 17; let mut hh = ((secteurs / ss) + 1023) / 1024; if hh < 4 { hh = 4; }
        if secteurs >= hh * ss * 1024 || hh > 16 { ss = 31; hh = 16; } if secteurs >= hh * ss * 1024 { ss = 63; hh = 16; }
        s = ss; h = hh; c = secteurs / s / h; }
    p[56..58].copy_from_slice(&(c as u16).to_be_bytes()); p[58] = h as u8; p[59] = s as u8; p[60..64].copy_from_slice(&2u32.to_be_bytes());
    for i in 0..16 { p[68 + i] = (id.rotate_left(i as u32) ^ 0x5Au32 ^ (i as u32 * 0x11)) as u8; }
    let somme: u32 = p.iter().map(|&b| b as u32).fold(0u32, |a, b| a.wrapping_add(b)); p[64..68].copy_from_slice(&(!somme).to_be_bytes()); p
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (mut sortie, mut taille, mut vhd, mut iso) = (None, 64u64, false, false); let mut fichiers: Vec<(String, String)> = Vec::new(); let mut i = 0;
    while i < a.len() {
        match a[i].as_str() {
            "--sortie" => { i += 1; sortie = a.get(i).cloned(); }
            "--taille-mio" => { i += 1; taille = a.get(i).and_then(|s| s.parse().ok()).unwrap_or(64); }
            "--vhd" => vhd = true,
            "--iso" => iso = true,
            x if x.contains('=') => { let (c, f) = x.split_once('=').unwrap(); fichiers.push((c.to_string(), f.to_string())); }
            x => { eprintln!("argument inconnu : {x}"); std::process::exit(2); }
        }
        i += 1;
    }
    let Some(sortie) = sortie else { eprintln!("Usage : voxa-mkimg --sortie img [--taille-mio 64] [--vhd] CHEMIN=FICHIER ..."); std::process::exit(2); };
    if fichiers.is_empty() { eprintln!("aucun fichier a ecrire"); std::process::exit(2); }
    let mut racine = Dossier::default();
    for (chemin, src) in &fichiers {
        let donnees = match fs::read(src) { Ok(d) => d, Err(e) => { eprintln!("lecture de {src} impossible : {e}"); std::process::exit(1); } };
        let comp: Vec<&str> = chemin.split('/').filter(|c| !c.is_empty()).collect();
        let Some((nom, dossiers)) = comp.split_last() else { eprintln!("chemin vide"); std::process::exit(2); };
        let mut d = &mut racine; for c in dossiers { d = d.sous.entry(c.to_string()).or_default(); }
        d.fichiers.insert(nom.to_string(), donnees);
    }
    if iso {
        let efi = match racine.sous.get("EFI").and_then(|e| e.sous.get("BOOT")).and_then(|b| b.fichiers.get("BOOTX64.EFI")) { Some(e) => e.clone(), None => { eprintln!("--iso exige EFI/BOOT/BOOTX64.EFI"); std::process::exit(2); } };
        let lisez = racine.fichiers.get("LISEZMOI.TXT").cloned().unwrap_or_else(|| b"VOXA OS - systeme live (UEFI)
".to_vec());
        match iso::iso(&efi, &lisez, "VOXA_OS_LIVE") { Ok(img) => { if let Err(e) = fs::write(&sortie, &img) { eprintln!("ecriture de {sortie} impossible : {e}"); std::process::exit(1); } println!("ISO ecrite : {sortie} ({} octets, ISO 9660 + El Torito UEFI + hybride GPT)", img.len()); return; } Err(e) => { eprintln!("echec : {e}"); std::process::exit(1); } }
    }
    match construire(&racine, taille) {
        Ok(mut img) => {
            if vhd { let t = img.len() as u64; img.extend_from_slice(&pied_vhd(t, crc32(&img[..4096]))); }
            if let Err(e) = fs::write(&sortie, &img) { eprintln!("ecriture de {sortie} impossible : {e}"); std::process::exit(1); }
            println!("image ecrite : {sortie} ({} octets, {} fichier(s), GPT + FAT32{})", img.len(), fichiers.len(), if vhd { ", pied VHD fixe" } else { "" });
        }
        Err(e) => { eprintln!("echec : {e}"); std::process::exit(1); }
    }
}
