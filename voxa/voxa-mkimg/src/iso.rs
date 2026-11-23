//! Fabrication d'une VRAIE image ISO 9660 demarrable en UEFI (El Torito, plateforme 0xEF), hybride : le debut du fichier porte aussi un MBR de protection et une GPT
//! dont la partition systeme EFI pointe sur l'image de demarrage FAT16 contenue dans l'ISO (ecriture directe sur cle possible, ou extraction par un outil d'ISO).
//! Disposition (secteurs de 2048 octets) : 0-15 zone systeme (MBR + GPT) ; 16 descripteur primaire ; 17 enregistrement de demarrage El Torito ; 18 terminateur ;
//! 19-20 tables de chemins ; 21-24 repertoires ; 25 catalogue de demarrage ; ensuite fichiers et image FAT16 ; en fin de fichier, la copie de secours de la GPT.
use super::{crc32, guid_esp, nom83, p16, p32, p64, Dossier};

const S: usize = 2048;

/// Volume FAT16 de `total` secteurs de 512 octets contenant l'arbre `arbre` (1 secteur par cluster, 512 entrees a la racine).
pub fn fat16(arbre: &Dossier, total: usize, etiquette: &[u8; 11]) -> Result<Vec<u8>, String> {
    let (rsvd, nfat, racine_ent) = (1usize, 2usize, 512usize); let racine_sect = racine_ent * 32 / 512;
    let mut fatsz = 1usize; loop { let data = total - rsvd - nfat * fatsz - racine_sect; let besoin = ((data + 2) * 2 + 511) / 512; if besoin <= fatsz { break; } fatsz = besoin; }
    let nclus = total - rsvd - nfat * fatsz - racine_sect;
    if !(4085..65525).contains(&nclus) { return Err(format!("{nclus} clusters : hors de la plage FAT16 (4085 a 65524)")); }
    let mut fat = vec![0u16; nclus + 2]; fat[0] = 0xFFF8; fat[1] = 0xFFFF;
    let mut zone = vec![0u8; nclus * 512]; let mut racine = vec![0u8; racine_sect * 512];
    fn alloue(fat: &mut [u16], n: usize) -> Result<usize, String> {
        let (mut premier, mut prec, mut vus) = (0usize, 0usize, 0usize);
        for c in 2..fat.len() { if fat[c] == 0 { fat[c] = 0xFFFF; if premier == 0 { premier = c; } else { fat[prec] = c as u16; } prec = c; vus += 1; if vus == n.max(1) { return Ok(premier); } } }
        Err("image FAT16 trop petite".into())
    }
    fn ecrit(fat: &[u16], zone: &mut [u8], premier: usize, d: &[u8]) { let (mut c, mut i) = (premier, 0usize); loop { let o = (c - 2) * 512; let m = (d.len() - i).min(512); zone[o..o + m].copy_from_slice(&d[i..i + m]); i += m; let s = fat[c] as usize; if s >= 0xFFF8 || i >= d.len() { break; } c = s; } }
    fn entree(nom: [u8; 11], attr: u8, clus: usize, taille: u32) -> [u8; 32] {
        let mut e = [0u8; 32]; e[..11].copy_from_slice(&nom); e[11] = attr; let date: u16 = ((2026 - 1980) << 9) | (10 << 5) | 12;
        p16(&mut e, 16, date); p16(&mut e, 18, date); p16(&mut e, 24, date); p16(&mut e, 26, clus as u16); p32(&mut e, 28, taille); e
    }
    fn dossier(d: &Dossier, parent: usize, fat: &mut Vec<u16>, zone: &mut Vec<u8>) -> Result<usize, String> {
        let nent = d.sous.len() + d.fichiers.len() + 2; let clus = alloue(fat, (nent * 32 + 511) / 512)?; let mut tab = Vec::new();
        tab.extend_from_slice(&entree(*b".          ", 0x10, clus, 0)); tab.extend_from_slice(&entree(*b"..         ", 0x10, parent, 0));
        for (n, s) in &d.sous { let c = dossier(s, clus, fat, zone)?; tab.extend_from_slice(&entree(nom83(n)?, 0x10, c, 0)); }
        for (n, f) in &d.fichiers { let c = if f.is_empty() { 0 } else { let c = alloue(fat, (f.len() + 511) / 512)?; ecrit(fat, zone, c, f); c }; tab.extend_from_slice(&entree(nom83(n)?, 0x20, c, f.len() as u32)); }
        ecrit(fat, zone, clus, &tab); Ok(clus)
    }
    let mut rt = Vec::new(); rt.extend_from_slice(&entree(*etiquette, 0x08, 0, 0));
    for (n, s) in &arbre.sous { let c = dossier(s, 0, &mut fat, &mut zone)?; rt.extend_from_slice(&entree(nom83(n)?, 0x10, c, 0)); }
    for (n, f) in &arbre.fichiers { let c = if f.is_empty() { 0 } else { let c = alloue(&mut fat, (f.len() + 511) / 512)?; ecrit(&fat, &mut zone, c, f); c }; rt.extend_from_slice(&entree(nom83(n)?, 0x20, c, f.len() as u32)); }
    if rt.len() > racine.len() { return Err("racine FAT16 pleine".into()); } racine[..rt.len()].copy_from_slice(&rt);
    let mut bs = vec![0u8; 512]; bs[0..3].copy_from_slice(&[0xEB, 0x3C, 0x90]); bs[3..11].copy_from_slice(b"VOXA 1.0");
    p16(&mut bs, 11, 512); bs[13] = 1; p16(&mut bs, 14, rsvd as u16); bs[16] = nfat as u8; p16(&mut bs, 17, racine_ent as u16);
    if total < 65536 { p16(&mut bs, 19, total as u16); } else { p32(&mut bs, 32, total as u32); }
    bs[21] = 0xF8; p16(&mut bs, 22, fatsz as u16); p16(&mut bs, 24, 32); p16(&mut bs, 26, 2); bs[36] = 0x80; bs[38] = 0x29; p32(&mut bs, 39, 0x564F_5846); bs[43..54].copy_from_slice(etiquette); bs[54..62].copy_from_slice(b"FAT16   "); bs[510] = 0x55; bs[511] = 0xAA;
    let mut vol = vec![0u8; total * 512]; vol[..512].copy_from_slice(&bs);
    let mut fb = vec![0u8; fatsz * 512]; for (i, v) in fat.iter().enumerate() { fb[i * 2..i * 2 + 2].copy_from_slice(&v.to_le_bytes()); }
    for k in 0..nfat { let o = (rsvd + k * fatsz) * 512; vol[o..o + fb.len()].copy_from_slice(&fb); }
    let o = (rsvd + nfat * fatsz) * 512; vol[o..o + racine.len()].copy_from_slice(&racine);
    let o = o + racine.len(); vol[o..o + zone.len()].copy_from_slice(&zone); Ok(vol)
}

fn both32(b: &mut [u8], o: usize, v: u32) { b[o..o + 4].copy_from_slice(&v.to_le_bytes()); b[o + 4..o + 8].copy_from_slice(&v.to_be_bytes()); }
fn both16(b: &mut [u8], o: usize, v: u16) { b[o..o + 2].copy_from_slice(&v.to_le_bytes()); b[o + 2..o + 4].copy_from_slice(&v.to_be_bytes()); }
const DATE7: [u8; 7] = [126, 10, 12, 0, 0, 0, 0];
fn enreg(nom: &[u8], lba: u32, longueur: u32, dossier: bool) -> Vec<u8> {
    let l = 33 + nom.len() + if nom.len() % 2 == 0 { 1 } else { 0 }; let mut r = vec![0u8; l];
    r[0] = l as u8; both32(&mut r, 2, lba); both32(&mut r, 10, longueur); r[18..25].copy_from_slice(&DATE7); r[25] = if dossier { 2 } else { 0 }; both16(&mut r, 28, 1); r[32] = nom.len() as u8; r[33..33 + nom.len()].copy_from_slice(nom); r
}
fn repertoire(moi: u32, parent: u32, taille_moi: u32, taille_parent: u32, enfants: &mut Vec<(Vec<u8>, u32, u32, bool)>) -> Result<Vec<u8>, String> {
    enfants.sort_by(|a, b| a.0.cmp(&b.0)); let mut d = Vec::new();
    d.extend_from_slice(&enreg(&[0], moi, taille_moi, true)); d.extend_from_slice(&enreg(&[1], parent, taille_parent, true));
    for (n, lba, len, est_dossier) in enfants.iter() { d.extend_from_slice(&enreg(n, *lba, *len, *est_dossier)); }
    if d.len() > S { return Err("repertoire ISO trop grand pour un secteur".into()); } d.resize(S, 0); Ok(d)
}
fn texte_pad(b: &mut [u8], o: usize, n: usize, s: &str) { for i in 0..n { b[o + i] = b' '; } let m = s.len().min(n); b[o..o + m].copy_from_slice(&s.as_bytes()[..m]); }
fn date17(b: &mut [u8], o: usize) { b[o..o + 16].copy_from_slice(b"2026101217000000"); b[o + 16] = 0; }
fn date17_vide(b: &mut [u8], o: usize) { b[o..o + 16].copy_from_slice(b"0000000000000000"); b[o + 16] = 0; }

/// `efi` : BOOTX64.EFI ; `lisezmoi` : texte a la racine ; `etiquette` : nom du volume (majuscules, chiffres, _).
pub fn iso(efi: &[u8], lisezmoi: &[u8], etiquette: &str) -> Result<Vec<u8>, String> {
    // image de demarrage FAT16 (8 Mio) : EFI/BOOT/BOOTX64.EFI
    let mut boot = Dossier::default(); boot.fichiers.insert("BOOTX64.EFI".into(), efi.to_vec());
    let mut efi_dir = Dossier::default(); efi_dir.sous.insert("BOOT".into(), boot); let mut racine_fat = Dossier::default(); racine_fat.sous.insert("EFI".into(), efi_dir);
    let image = fat16(&racine_fat, 8 * 2048, b"VOXA EFI   ")?; let img_secteurs = (image.len() + S - 1) / S;
    // attribution des secteurs
    let (l_pvd, l_br, l_term, l_ptl, l_ptm) = (16u32, 17u32, 18u32, 19u32, 20u32);
    let (d_racine, d_efi, d_efiboot, d_boot, catalogue) = (21u32, 22u32, 23u32, 24u32, 25u32);
    let sect = |n: usize| ((n + S - 1) / S) as u32;
    let l_efi = 26u32; let l_lisez = l_efi + sect(efi.len()); let l_img = l_lisez + sect(lisezmoi.len()).max(1); let fin = l_img + img_secteurs as u32;
    let iso_octets = fin as usize * S; let total = iso_octets + 9 * S; let total512 = (total / 512) as u64;
    let mut out = vec![0u8; total];
    // repertoires
    let mut e_boot_efi = vec![(b"BOOTX64.EFI;1".to_vec(), l_efi, efi.len() as u32, false)];
    let r_boot_efi = repertoire(d_efiboot, d_efi, S as u32, S as u32, &mut e_boot_efi)?;
    let mut e_efi = vec![(b"BOOT".to_vec(), d_efiboot, S as u32, true)];
    let r_efi = repertoire(d_efi, d_racine, S as u32, S as u32, &mut e_efi)?;
    let mut e_boot = vec![(b"BOOT.CAT;1".to_vec(), catalogue, S as u32, false), (b"EFIBOOT.IMG;1".to_vec(), l_img, image.len() as u32, false)];
    let r_boot = repertoire(d_boot, d_racine, S as u32, S as u32, &mut e_boot)?;
    let mut e_racine = vec![(b"BOOT".to_vec(), d_boot, S as u32, true), (b"EFI".to_vec(), d_efi, S as u32, true), (b"LISEZMOI.TXT;1".to_vec(), l_lisez, lisezmoi.len() as u32, false)];
    let r_racine = repertoire(d_racine, d_racine, S as u32, S as u32, &mut e_racine)?;
    for (l, r) in [(d_racine, &r_racine), (d_efi, &r_efi), (d_efiboot, &r_boot_efi), (d_boot, &r_boot)] { out[l as usize * S..l as usize * S + S].copy_from_slice(r); }
    // tables de chemins : racine(1), BOOT(2, parent 1), EFI(3, parent 1), EFI/BOOT(4, parent 3)
    let chemins: [(&[u8], u32, u16); 4] = [(&[0], d_racine, 1), (b"BOOT", d_boot, 1), (b"EFI", d_efi, 1), (b"BOOT", d_efiboot, 3)];
    let (mut ptl, mut ptm) = (Vec::new(), Vec::new());
    for (nom, lba, parent) in chemins { for (t, big) in [(&mut ptl, false), (&mut ptm, true)] {
        t.push(nom.len() as u8); t.push(0); if big { t.extend_from_slice(&lba.to_be_bytes()); t.extend_from_slice(&parent.to_be_bytes()); } else { t.extend_from_slice(&lba.to_le_bytes()); t.extend_from_slice(&parent.to_le_bytes()); }
        t.extend_from_slice(nom); if nom.len() % 2 == 1 { t.push(0); } } }
    let pt_taille = ptl.len() as u32; out[l_ptl as usize * S..l_ptl as usize * S + ptl.len()].copy_from_slice(&ptl); out[l_ptm as usize * S..l_ptm as usize * S + ptm.len()].copy_from_slice(&ptm);
    // descripteur primaire
    { let p = &mut out[l_pvd as usize * S..(l_pvd as usize + 1) * S]; p[0] = 1; p[1..6].copy_from_slice(b"CD001"); p[6] = 1; texte_pad(p, 8, 32, ""); texte_pad(p, 40, 32, etiquette);
      both32(p, 80, fin); both16(p, 120, 1); both16(p, 124, 1); both16(p, 128, S as u16); both32(p, 132, pt_taille);
      p[140..144].copy_from_slice(&l_ptl.to_le_bytes()); p[148..152].copy_from_slice(&l_ptm.to_be_bytes());
      let rr = enreg(&[0], d_racine, S as u32, true); p[156..156 + rr.len()].copy_from_slice(&rr);
      texte_pad(p, 190, 128, ""); texte_pad(p, 318, 128, "VOXA"); texte_pad(p, 446, 128, "voxa-mkimg"); texte_pad(p, 574, 128, "VOXA OS LIVE"); texte_pad(p, 702, 37, ""); texte_pad(p, 739, 37, ""); texte_pad(p, 776, 37, "");
      date17(p, 813); date17(p, 830); date17_vide(p, 847); date17(p, 864); p[881] = 1; }
    // enregistrement de demarrage El Torito
    { let b = &mut out[l_br as usize * S..(l_br as usize + 1) * S]; b[0] = 0; b[1..6].copy_from_slice(b"CD001"); b[6] = 1; b[7..7 + 23].copy_from_slice(b"EL TORITO SPECIFICATION"); b[0x47..0x4B].copy_from_slice(&catalogue.to_le_bytes()); }
    // terminateur
    { let t = &mut out[l_term as usize * S..(l_term as usize + 1) * S]; t[0] = 255; t[1..6].copy_from_slice(b"CD001"); t[6] = 1; }
    // catalogue : entree de validation (plateforme 0xEF = EFI) puis entree par defaut amorcable, sans emulation, pointant sur l'image FAT16
    { let c = &mut out[catalogue as usize * S..(catalogue as usize + 1) * S]; c[0] = 1; c[1] = 0xEF; c[0x1E] = 0x55; c[0x1F] = 0xAA;
      let somme: u16 = (0..16).map(|i| u16::from_le_bytes([c[i * 2], c[i * 2 + 1]])).fold(0u16, |a, b| a.wrapping_add(b)); p16(c, 0x1C, 0u16.wrapping_sub(somme));
      c[32] = 0x88; c[33] = 0; p16(c, 38, (image.len() / 512).min(65535) as u16); p32(c, 40, l_img); }
    // fichiers
    out[l_efi as usize * S..l_efi as usize * S + efi.len()].copy_from_slice(efi);
    out[l_lisez as usize * S..l_lisez as usize * S + lisezmoi.len()].copy_from_slice(lisezmoi);
    out[l_img as usize * S..l_img as usize * S + image.len()].copy_from_slice(&image);
    // hybride : MBR de protection + GPT (partition systeme EFI = image FAT16) + copie de secours en fin de fichier
    out[446 + 2] = 0x02; out[446 + 4] = 0xEE; out[446 + 5] = 0xFF; out[446 + 6] = 0xFF; out[446 + 7] = 0xFF; p32(&mut out, 446 + 8, 1); p32(&mut out, 446 + 12, (total512 - 1).min(0xFFFF_FFFF) as u32); out[510] = 0x55; out[511] = 0xAA;
    let deb = l_img as u64 * 4; let fin_p = deb + (image.len() / 512) as u64 - 1;
    let graine = crc32(&image[..4096.min(image.len())]); let (mut dg, mut pg) = ([0u8; 16], [0u8; 16]);
    for i in 0..16 { dg[i] = (graine.rotate_left(i as u32 * 3) ^ 0x1234_5678 ^ (i as u32 * 0x1F)) as u8; pg[i] = (graine.rotate_left(i as u32 * 5 + 1) ^ 0x8765_4321 ^ (i as u32 * 0x2B)) as u8; }
    dg[7] = (dg[7] & 0x0F) | 0x40; dg[8] = (dg[8] & 0x3F) | 0x80; pg[7] = (pg[7] & 0x0F) | 0x40; pg[8] = (pg[8] & 0x3F) | 0x80;
    let mut ent = vec![0u8; 128 * 128]; ent[0..16].copy_from_slice(&guid_esp()); ent[16..32].copy_from_slice(&pg); p64(&mut ent, 32, deb); p64(&mut ent, 40, fin_p);
    for (i, c) in "VOXA EFI".encode_utf16().enumerate() { p16(&mut ent, 56 + i * 2, c); }
    let crc_ent = crc32(&ent);
    let entete = |courant: u64, secours: u64, lba_ent: u64| -> Vec<u8> {
        let mut h = vec![0u8; 512]; h[0..8].copy_from_slice(b"EFI PART"); p32(&mut h, 8, 0x0001_0000); p32(&mut h, 12, 92); p64(&mut h, 24, courant); p64(&mut h, 32, secours);
        p64(&mut h, 40, 34); p64(&mut h, 48, total512 - 34); h[56..72].copy_from_slice(&dg); p64(&mut h, 72, lba_ent); p32(&mut h, 80, 128); p32(&mut h, 84, 128); p32(&mut h, 88, crc_ent);
        let c = crc32(&h[..92]); p32(&mut h, 16, c); h };
    out[512..1024].copy_from_slice(&entete(1, total512 - 1, 2)); out[1024..1024 + ent.len()].copy_from_slice(&ent);
    let o = (total512 as usize - 33) * 512; out[o..o + ent.len()].copy_from_slice(&ent);
    let o = (total512 as usize - 1) * 512; out[o..o + 512].copy_from_slice(&entete(total512 - 1, 1, total512 - 33));
    Ok(out)
}
