// Controleur NVMe : initialisation (files admin et E/S) et lecture d'un secteur.
// Reference : NVMe Base Spec 2.4 (CAP, CC, CSTS, AQA, ASQ, ACQ ; SQE 64 octets, CQE 16 octets ;
// phase tag DW3 bit 16, etat DW3 bits 31:17 ; doorbells 1000h + (2y) * (4 << DSTRD) pour SQ tail,
// 1000h + (2y+1) * (4 << DSTRD) pour CQ head ; Identify (06h) ; Create IO CQ (05h) ;
// Create IO SQ (01h) ; Read (02h) ; section 3.5 : initialisation).
//
// Ordre des pages statiques : 0 SQ admin, 1 CQ admin, 2 SQ E/S, 3 CQ E/S, 4 identify, 5 lecture.
// Adresses physiques = adresses virtuelles (identite en pre-boot et apres ExitBootServices).
use crate::usb_r0::Line;

const QSIZE: u32 = 16;

#[repr(C, align(4096))]
struct Mem([u8; 6 * 4096]);
static mut MEM: Mem = Mem([0; 6 * 4096]);

#[derive(Clone, Copy)]
struct Q {
    sq: u64,
    cq: u64,
    tail: u32,
    head: u32,
    phase: u32,
    db_sq: u64,
    db_cq: u64,
}

static mut QA: Q = Q { sq: 0, cq: 0, tail: 0, head: 0, phase: 1, db_sq: 0, db_cq: 0 };
static mut QI: Q = Q { sq: 0, cq: 0, tail: 0, head: 0, phase: 1, db_sq: 0, db_cq: 0 };
static mut CID: u32 = 1;

unsafe fn page(i: u64) -> u64 {
    // SAFETY: adresse dans le tableau statique.
    core::ptr::addr_of_mut!(MEM) as u64 + i * 4096
}
unsafe fn r32(a: u64) -> u32 {
    // SAFETY: registre MMIO NVMe.
    unsafe { core::ptr::read_volatile(a as *const u32) }
}
unsafe fn w32(a: u64, v: u32) {
    // SAFETY: registre MMIO NVMe.
    unsafe { core::ptr::write_volatile(a as *mut u32, v) }
}
unsafe fn w64(a: u64, v: u64) {
    // SAFETY: registre MMIO 64 bits en deux ecritures 32 bits (bas puis haut).
    unsafe {
        w32(a, v as u32);
        w32(a + 4, (v >> 32) as u32);
    }
}
unsafe fn zero(a: u64, n: usize) {
    for i in 0..n {
        // SAFETY: memoire statique.
        unsafe { core::ptr::write_volatile((a + i as u64) as *mut u8, 0) };
    }
}

/// Attend que `f` soit vrai, au plus `limit` tours.
unsafe fn wait_until(limit: u64, mut f: impl FnMut() -> bool) -> bool {
    for _ in 0..limit {
        if f() {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

/// Soumet une commande (64 octets) a la file `q`. Retourne l'identifiant de commande.
unsafe fn submit(q: &mut Q, dw: &[u32; 16]) -> u32 {
    // SAFETY: file statique ; index < QSIZE.
    unsafe {
        let cid = CID;
        CID += 1;
        let base = q.sq + (q.tail as u64) * 64;
        for (i, v) in dw.iter().enumerate() {
            w32(base + (i as u64) * 4, *v);
        }
        // DW0 : opcode (bits 7:0) et identifiant de commande (bits 31:16)
        w32(base, (dw[0] & 0xFFFF) | (cid << 16));
        q.tail = (q.tail + 1) % QSIZE;
        w32(q.db_sq, q.tail);
        cid
    }
}

/// Attend la fin de commande sur la file `q`. Retourne le champ d'etat (0 = succes).
unsafe fn complete(q: &mut Q) -> Result<u32, &'static str> {
    // SAFETY: file statique ; index < QSIZE.
    unsafe {
        let ok = wait_until(200_000_000, || {
            let e = q.cq + (q.head as u64) * 16;
            (r32(e + 12) >> 16) & 1 == q.phase
        });
        if !ok {
            return Err("NVMe : pas de reponse");
        }
        let e = q.cq + (q.head as u64) * 16;
        let dw3 = r32(e + 12);
        q.head = (q.head + 1) % QSIZE;
        if q.head == 0 {
            q.phase ^= 1;
        }
        w32(q.db_cq, q.head);
        Ok(dw3 >> 17)
    }
}

/// Initialise les files admin et E/S, puis lit le secteur `lba` du namespace 1.
/// Retourne une ligne resumant le resultat (taille de secteur, debut du secteur).
pub unsafe fn read_sector(bar: u64, lba: u64) -> (Line, Line) {
    let mut id = Line::new();
    let mut l = Line::new();
    // SAFETY: MMIO NVMe et memoire statique ; seules les ecritures de configuration sont faites.
    unsafe {
        let cap = (r32(bar) as u64) | ((r32(bar + 4) as u64) << 32);
        let mqes = (cap & 0xFFFF) as u32;
        let dstrd = ((cap >> 32) & 0xF) as u32;
        if mqes < QSIZE - 1 {
            l.s(b"NVME QUEUE TROP PETITE");
            return (l, id);
        }
        let stride = 4u64 << dstrd;
        // 1. Arret du controleur si actif, attente de CSTS.RDY = 0
        if r32(bar + 0x14) & 1 != 0 {
            w32(bar + 0x14, r32(bar + 0x14) & !1);
        }
        if !wait_until(200_000_000, || r32(bar + 0x1C) & 1 == 0) {
            l.s(b"NVME ARRET IMPOSSIBLE");
            return (l, id);
        }
        // 2. Files : profondeur, adresses admin (ASQ, ACQ), puis E/S
        zero(page(0), 6 * 4096);
        QA = Q { sq: page(0), cq: page(1), tail: 0, head: 0, phase: 1, db_sq: bar + 0x1000, db_cq: bar + 0x1000 + stride };
        QI = Q { sq: page(2), cq: page(3), tail: 0, head: 0, phase: 1, db_sq: bar + 0x1000 + 2 * stride, db_cq: bar + 0x1000 + 3 * stride };
        w32(bar + 0x24, ((QSIZE - 1) << 16) | (QSIZE - 1)); // AQA : ACQS et ASQS
        w64(bar + 0x28, QA.sq); // ASQ
        w64(bar + 0x30, QA.cq); // ACQ
        // 3. CC : IOSQES 6 (64 o), IOCQES 4 (16 o), MPS 0 (4 Ko), EN = 1
        w32(bar + 0x14, (4 << 20) | (6 << 16) | 1);
        if !wait_until(200_000_000, || r32(bar + 0x1C) & 1 == 1) {
            l.s(b"NVME DEMARRAGE IMPOSSIBLE");
            return (l, id);
        }
        // 3b. Identify Controller (06h, CNS 1) : modele (MN, octets 24..63), serie (SN, 4..23), firmware (FR, 64..71)
        let mut dw = [0u32; 16];
        dw[0] = 0x06;
        dw[6] = page(4) as u32;
        dw[7] = (page(4) >> 32) as u32;
        dw[10] = 1;
        let _ = submit(&mut *(&raw mut QA), &dw);
        match complete(&mut *(&raw mut QA)) {
            Ok(0) => {
            // Modele : MN fait 40 octets (24..63), on les lit en entier.
            let mut m = [b' '; 40];
            for (k, slot) in m.iter_mut().enumerate() {
                let c = core::ptr::read_volatile((page(4) + 24 + k as u64) as *const u8);
                *slot = if (0x20..0x7F).contains(&c) { c } else { b' ' };
            }
            id.s(b"NVME ");
            id.s(&m);
            let mut f = [b' '; 8];
            for (k, slot) in f.iter_mut().enumerate() {
                let c = core::ptr::read_volatile((page(4) + 64 + k as u64) as *const u8);
                *slot = if (0x20..0x7F).contains(&c) { c } else { b' ' };
            }
            id.s(b" FW ");
            id.s(&f);
            }
            Ok(s) => {
                id.s(b"ID CTRL ERR ").h(s, 4);
            }
            Err(e) => {
                id.s(e.as_bytes());
            }
        }
        // 4. Identify Namespace (06h, CNS 0) pour le namespace 1
        let mut dw = [0u32; 16];
        dw[0] = 0x06;
        dw[1] = 1;
        dw[6] = page(4) as u32;
        dw[7] = (page(4) >> 32) as u32;
        let _ = submit(&mut *(&raw mut QA), &dw);
        match complete(&mut *(&raw mut QA)) {
            Ok(0) => {}
            Ok(s) => {
                l.s(b"IDENTIFY ERR ").h(s, 4);
                return (l, id);
            }
            Err(e) => {
                l.s(e.as_bytes());
                return (l, id);
            }
        }
        let ns = page(4);
        let flbas = core::ptr::read_volatile((ns + 26) as *const u8) & 0xF;
        let lbads = core::ptr::read_volatile((ns + 128 + 4 * flbas as u64 + 2) as *const u8);
        // LBADS > 15 n'est pas un format valide : on le rejette avant tout decalage.
        let sector = if lbads < 16 { 1usize << lbads } else { usize::MAX };
        if sector > 4096 {
            l.s(b"SECTEUR TROP GRAND");
            return (l, id);
        }
        // 5. File E/S : Create IO CQ (05h) puis Create IO SQ (01h), identifiant 1
        let mut dw = [0u32; 16];
        dw[0] = 0x05;
        dw[6] = page(3) as u32;
        dw[7] = (page(3) >> 32) as u32;
        dw[10] = ((QSIZE - 1) << 16) | 1;
        dw[11] = 1; // physiquement contigue
        let _ = submit(&mut *(&raw mut QA), &dw);
        if let Err(e) = complete(&mut *(&raw mut QA)) {
            l.s(e.as_bytes());
            return (l, id);
        }
        let mut dw = [0u32; 16];
        dw[0] = 0x01;
        dw[6] = page(2) as u32;
        dw[7] = (page(2) >> 32) as u32;
        dw[10] = ((QSIZE - 1) << 16) | 1;
        dw[11] = (1 << 16) | 1; // CQ 1, physiquement contigue
        let _ = submit(&mut *(&raw mut QA), &dw);
        if let Err(e) = complete(&mut *(&raw mut QA)) {
            l.s(e.as_bytes());
            return (l, id);
        }
        // 6. Lecture du secteur (02h, NSID 1, un bloc)
        let mut dw = [0u32; 16];
        dw[0] = 0x02;
        dw[1] = 1;
        dw[6] = page(5) as u32;
        dw[7] = (page(5) >> 32) as u32;
        dw[10] = lba as u32;
        dw[11] = (lba >> 32) as u32;
        dw[12] = 0; // NLB = 0 : un bloc
        submit(&mut *(&raw mut QI), &dw);
        match complete(&mut *(&raw mut QI)) {
            Ok(0) => {}
            Ok(s) => {
                l.s(b"LECTURE ERR ").h(s, 4);
                return (l, id);
            }
            Err(e) => {
                l.s(e.as_bytes());
                return (l, id);
            }
        }
        // 7. Affichage : taille de secteur et signature (GPT "EFI PART" en LBA 1)
        l.s(b"NVME LBA ").h(lba as u32, 1).s(b" S").h(sector as u32, 4).s(b" ");
        for i in 0..8u64 {
            let c = core::ptr::read_volatile((page(5) + i) as *const u8);
            let c = if (0x20..0x7F).contains(&c) { c } else { b'.' };
            l.s(&[c]);
        }
    }
    (l, id)
}

/// Initialise les files E/S du controleur (memes etapes que `read_sector`, sans Identify
/// controleur). Retourne la taille de secteur du namespace 1, ou un message d'erreur.
unsafe fn init_io(bar: u64) -> Result<usize, &'static str> {
    // SAFETY: MMIO NVMe et memoire statique ; uniquement des ecritures de configuration.
    unsafe {
        let cap = (r32(bar) as u64) | ((r32(bar + 4) as u64) << 32);
        let mqes = (cap & 0xFFFF) as u32;
        let dstrd = ((cap >> 32) & 0xF) as u32;
        if mqes < QSIZE - 1 {
            return Err("NVME QUEUE TROP PETITE");
        }
        let stride = 4u64 << dstrd;
        if r32(bar + 0x14) & 1 != 0 {
            w32(bar + 0x14, r32(bar + 0x14) & !1);
        }
        if !wait_until(200_000_000, || r32(bar + 0x1C) & 1 == 0) {
            return Err("NVME ARRET IMPOSSIBLE");
        }
        zero(page(0), 6 * 4096);
        QA = Q { sq: page(0), cq: page(1), tail: 0, head: 0, phase: 1, db_sq: bar + 0x1000, db_cq: bar + 0x1000 + stride };
        QI = Q { sq: page(2), cq: page(3), tail: 0, head: 0, phase: 1, db_sq: bar + 0x1000 + 2 * stride, db_cq: bar + 0x1000 + 3 * stride };
        w32(bar + 0x24, ((QSIZE - 1) << 16) | (QSIZE - 1));
        w64(bar + 0x28, QA.sq);
        w64(bar + 0x30, QA.cq);
        w32(bar + 0x14, (4 << 20) | (6 << 16) | 1);
        if !wait_until(200_000_000, || r32(bar + 0x1C) & 1 == 1) {
            return Err("NVME DEMARRAGE IMPOSSIBLE");
        }
        // Identify Namespace (06h, CNS 0) pour le namespace 1 : taille de secteur
        let mut dw = [0u32; 16];
        dw[0] = 0x06;
        dw[1] = 1;
        dw[6] = page(4) as u32;
        dw[7] = (page(4) >> 32) as u32;
        let _ = submit(&mut *(&raw mut QA), &dw);
        match complete(&mut *(&raw mut QA)) {
            Ok(0) => {}
            Ok(_) => return Err("IDENTIFY NS ERREUR"),
            Err(e) => return Err(e),
        }
        let ns = page(4);
        let flbas = core::ptr::read_volatile((ns + 26) as *const u8) & 0xF;
        let lbads = core::ptr::read_volatile((ns + 128 + 4 * flbas as u64 + 2) as *const u8);
        if lbads >= 16 {
            return Err("LBADS INVALIDE");
        }
        let sector = 1usize << lbads;
        if sector > 4096 {
            return Err("SECTEUR TROP GRAND");
        }
        // File E/S : Create IO CQ (05h) puis Create IO SQ (01h), identifiant 1
        let mut dw = [0u32; 16];
        dw[0] = 0x05;
        dw[6] = page(3) as u32;
        dw[7] = (page(3) >> 32) as u32;
        dw[10] = ((QSIZE - 1) << 16) | 1;
        dw[11] = 1;
        let _ = submit(&mut *(&raw mut QA), &dw);
        if let Err(e) = complete(&mut *(&raw mut QA)) {
            return Err(e);
        }
        let mut dw = [0u32; 16];
        dw[0] = 0x01;
        dw[6] = page(2) as u32;
        dw[7] = (page(2) >> 32) as u32;
        dw[10] = ((QSIZE - 1) << 16) | 1;
        dw[11] = (1 << 16) | 1;
        let _ = submit(&mut *(&raw mut QA), &dw);
        if let Err(e) = complete(&mut *(&raw mut QA)) {
            return Err(e);
        }
        Ok(sector)
    }
}

/// Lit un bloc (LBA `lba`, namespace 1) dans la page 5. Lecture seule.
unsafe fn read_block(lba: u64) -> Result<(), &'static str> {
    // SAFETY: file E/S initialisee par `init_io` ; tampon statique de 4 Ko.
    unsafe {
        let mut dw = [0u32; 16];
        dw[0] = 0x02;
        dw[1] = 1;
        dw[6] = page(5) as u32;
        dw[7] = (page(5) >> 32) as u32;
        dw[10] = lba as u32;
        dw[11] = (lba >> 32) as u32;
        dw[12] = 0;
        let _ = submit(&mut *(&raw mut QI), &dw);
        match complete(&mut *(&raw mut QI)) {
            Ok(0) => Ok(()),
            Ok(_) => Err("LECTURE ERREUR"),
            Err(e) => Err(e),
        }
    }
}

/// Lit la table de partitions GPT du namespace 1 (lecture seule) : en-tete en LBA 1,
/// puis les entrees. Ecrit une ligne d'en-tete puis une ligne par partition dans `out`.
/// Retourne le nombre de lignes ecrites.
pub unsafe fn read_gpt(bar: u64, out: &mut [Line]) -> usize {
    if out.is_empty() {
        return 0;
    }
    // SAFETY: MMIO NVMe et memoire statique ; lectures seulement apres l'initialisation des files.
    unsafe {
        let sector = match init_io(bar) {
            Ok(s) => s,
            Err(e) => {
                out[0].s(e.as_bytes());
                return 1;
            }
        };
        if let Err(e) = read_block(1) {
            out[0].s(b"GPT LBA1 : ").s(e.as_bytes());
            return 1;
        }
        let hdr = page(5);
        let sig = core::slice::from_raw_parts(hdr as *const u8, 8);
        if sig != b"EFI PART" {
            out[0].s(b"GPT ABSENTE (pas de EFI PART en LBA 1)");
            return 1;
        }
        let entries_lba = u64le_at(hdr, 72);
        let count = u32le_at(hdr, 80) as u64;
        let esize = u32le_at(hdr, 84) as u64;
        if esize != 128 || count == 0 || count > 256 || sector < 512 {
            out[0].s(b"GPT ENTREES NON PRISES EN CHARGE");
            return 1;
        }
        let per = sector as u64 / esize;
        let total_sectors = count.div_ceil(per);
        let mut n = 0usize;
        let mut starts = [0u64; 32];
        let mut nst = 0usize;
        let mut h = Line::new();
        h.s(b"GPT ").h(count as u32, 3).s(b" ENTREES, SECTEUR ").h(sector as u32, 4);
        out[0] = h;
        n += 1;
        for s in 0..total_sectors {
            if let Err(e) = read_block(entries_lba + s) {
                if n < out.len() {
                    let mut l = Line::new();
                    l.s(b"GPT ERREUR ").s(e.as_bytes());
                    out[n] = l;
                    n += 1;
                }
                return n;
            }
            for k in 0..per {
                let idx = s * per + k;
                if idx >= count {
                    break;
                }
                let e = page(5) + k * esize;
                let tipe = core::slice::from_raw_parts(e as *const u8, 16);
                if tipe.iter().all(|&c| c == 0) {
                    continue;
                }
                let first = u64le_at(e, 32);
                if nst < 32 {
                    starts[nst] = first;
                    nst += 1;
                }
                let last = u64le_at(e, 40);
                let mut l = Line::new();
                l.s(b"P").h(idx as u32, 2).s(b" ").h(first as u32, 8).s(b"-").h(last as u32, 8).s(b" ");
                for c in 0..16u64 {
                    let ch = core::ptr::read_volatile((e + 56 + 2 * c) as *const u8);
                    let ch = if (0x20..0x7F).contains(&ch) { ch } else { b'.' };
                    l.s(&[ch]);
                }
                if n < out.len() {
                    out[n] = l;
                    n += 1;
                }
            }
        }
        // Lecture seule du systeme de fichiers NTFS de chaque partition qui en porte la signature
        for &st in &starts[..nst] {
            n = ntfs_one(st, out, n);
        }
        n
    }
}

fn le16(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
fn le32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}
fn le64(b: &[u8], o: usize) -> u64 {
    let mut v = 0u64;
    for i in 0..8 {
        v |= (b[o + i] as u64) << (8 * i);
    }
    v
}
fn msg(t: &[u8], e: &str) -> Line {
    let mut l = Line::new();
    l.s(t);
    l.s(e.as_bytes());
    l
}
fn push(out: &mut [Line], n: usize, l: Line) -> usize {
    if n < out.len() {
        out[n] = l;
        n + 1
    } else {
        n
    }
}

/// Lit un secteur de 512 o (namespace 1). Lecture seule.
unsafe fn read_sector512(lba: u64, dst: &mut [u8; 512]) -> Result<(), &'static str> {
    // SAFETY: tampon statique (page 5) ; copie de 512 o.
    unsafe {
        read_block(lba)?;
        core::ptr::copy_nonoverlapping(page(5) as *const u8, dst.as_mut_ptr(), 512);
    }
    Ok(())
}

/// Lit l'enregistrement MFT `r` (1 Ko) a partir de l'LBA `base`, applique les fixups.
unsafe fn read_record(base: u64, r: u64, rec: &mut [u8; 1024]) -> Result<(), &'static str> {
    let lba = base + r * 2;
    let mut s0 = [0u8; 512];
    let mut s1 = [0u8; 512];
    // SAFETY: lectures seules via read_sector512.
    unsafe {
        read_sector512(lba, &mut s0)?;
        read_sector512(lba + 1, &mut s1)?;
    }
    rec[..512].copy_from_slice(&s0);
    rec[512..].copy_from_slice(&s1);
    if &rec[0..4] != b"FILE" {
        return Err("signature FILE absente");
    }
    let uo = le16(rec, 4) as usize;
    let uc = le16(rec, 6) as usize;
    if uc != 3 || uo + 6 > 1024 {
        return Err("fixup inattendu");
    }
    for i in 1..3 {
        let end = i * 512 - 2;
        if rec[end] != rec[uo] || rec[end + 1] != rec[uo + 1] {
            return Err("fixup incoherent");
        }
        rec[end] = rec[uo + 2 * i];
        rec[end + 1] = rec[uo + 2 * i + 1];
    }
    Ok(())
}

/// Premier run de l'attribut $DATA non residant (0x80) : (LCN absolu, nombre de clusters).
fn first_run(rec: &[u8; 1024]) -> Option<(u64, u64)> {
    let mut o = le16(rec, 0x14) as usize;
    while o + 0x22 <= 1024 {
        let t = le32(rec, o);
        if t == 0xFFFF_FFFF {
            break;
        }
        let len = le32(rec, o + 4) as usize;
        if len == 0 || o + len > 1024 {
            break;
        }
        if t == 0x80 && rec[o + 8] == 1 {
            let p = o + le16(rec, o + 0x20) as usize;
            let hdr = rec[p];
            let lb = (hdr & 0xF) as usize;
            let ob = (hdr >> 4) as usize;
            if lb == 0 || lb > 8 || ob == 0 || ob > 7 || p + 1 + lb + ob > 1024 {
                return None;
            }
            let mut cnt = 0u64;
            for i in 0..lb {
                cnt |= (rec[p + 1 + i] as u64) << (8 * i);
            }
            let mut lcn: i64 = 0;
            for i in 0..ob {
                lcn |= (rec[p + 1 + lb + i] as i64) << (8 * i);
            }
            if rec[p + lb + ob] & 0x80 != 0 {
                lcn -= 1i64 << (8 * ob);
            }
            if lcn < 0 {
                return None;
            }
            return Some((lcn as u64, cnt));
        }
        o += len;
    }
    None
}

/// Premier attribut $FILE_NAME residant qui n'est pas un nom DOS (espace 2) :
/// (repertoire parent, offset du nom, longueur du nom en caracteres).
fn fname(rec: &[u8; 1024]) -> Option<(u64, usize, usize)> {
    let mut o = le16(rec, 0x14) as usize;
    while o + 0x22 <= 1024 {
        let t = le32(rec, o);
        if t == 0xFFFF_FFFF {
            break;
        }
        let len = le32(rec, o + 4) as usize;
        if len == 0 || o + len > 1024 {
            break;
        }
        if t == 0x30 && rec[o + 8] == 0 {
            let clen = le32(rec, o + 16) as usize;
            let c = o + le16(rec, o + 20) as usize;
            if clen >= 66 && c + 66 <= 1024 && rec[c + 65] != 2 {
                let nlen = rec[c + 64] as usize;
                if c + 66 + nlen * 2 <= 1024 {
                    let parent = le64(rec, c) & 0xFFFF_FFFF_FFFF;
                    return Some((parent, c + 66, nlen));
                }
            }
        }
        o += len;
    }
    None
}

/// Lecture seule d'une partition NTFS : secteur de demarrage, premier run de la MFT, puis
/// les premiers enregistrements (256 max) dont le parent est la racine (enregistrement 5).
unsafe fn ntfs_one(start: u64, out: &mut [Line], mut n: usize) -> usize {
    let mut boot = [0u8; 512];
    // SAFETY: lecture seule d'un secteur.
    if unsafe { read_sector512(start, &mut boot) }.is_err() {
        return n;
    }
    if &boot[3..11] != b"NTFS    " {
        return n;
    }
    let spc = boot[0x0D] as u64;
    let mft_lcn = le64(&boot, 0x30);
    let rv = boot[0x40] as i8;
    if spc == 0 {
        return push(out, n, msg(b"NTFS : cluster invalide", ""));
    }
    let rec_size: u64 = if rv > 0 {
        rv as u64 * spc * 512
    } else {
        1u64 << ((-(rv as i32)) as u32)
    };
    if rec_size != 1024 {
        return push(out, n, msg(b"NTFS : enregistrement MFT != 1 Ko", ""));
    }
    let mft_base = start + mft_lcn * spc;
    let mut rec = [0u8; 1024];
    // SAFETY: lecture seule.
    if let Err(e) = unsafe { read_record(mft_base, 0, &mut rec) } {
        return push(out, n, msg(b"NTFS $MFT : ", e));
    }
    let (run_lcn, run_clusters) = match first_run(&rec) {
        Some(v) => v,
        None => return push(out, n, msg(b"NTFS : run $MFT introuvable", "")),
    };
    let data_base = start + run_lcn * spc;
    let mut h = Line::new();
    h.s(b"NTFS MFT LCN ").h(mft_lcn as u32, 8).s(b" CLUS ").h(run_clusters as u32, 4);
    n = push(out, n, h);
    let max_rec = (run_clusters * spc * 512 / 1024).min(256);
    for r in 0..max_rec {
        // SAFETY: lecture seule.
        if unsafe { read_record(data_base, r, &mut rec) }.is_err() {
            continue;
        }
        if le16(&rec, 0x16) & 1 == 0 {
            continue; // enregistrement libre
        }
        let dir = le16(&rec, 0x16) & 2 != 0;
        if let Some((parent, noff, nlen)) = fname(&rec) {
            if parent != 5 {
                continue;
            }
            let mut l = Line::new();
            l.s(b"NTFS R").h(r as u32, 3);
            l.s(if dir { b" D " } else { b" F " });
            for c in 0..nlen.min(28) {
                let ch = rec[noff + 2 * c];
                let ch = if (0x20..0x7F).contains(&ch) { ch } else { b'.' };
                l.s(&[ch]);
            }
            n = push(out, n, l);
        }
    }
    n
}

/// Lecture little-endian a partir d'une adresse physique (memoire statique).
unsafe fn u64le_at(base: u64, off: u64) -> u64 {
    // SAFETY: adresse dans la page 5 (4 Ko), off + 8 <= 4096.
    unsafe {
        let mut v = 0u64;
        for i in 0..8u64 {
            v |= (core::ptr::read_volatile((base + off + i) as *const u8) as u64) << (8 * i);
        }
        v
    }
}

unsafe fn u32le_at(base: u64, off: u64) -> u32 {
    // SAFETY: adresse dans la page 5 (4 Ko), off + 4 <= 4096.
    unsafe {
        let mut v = 0u32;
        for i in 0..4u64 {
            v |= (core::ptr::read_volatile((base + off + i) as *const u8) as u32) << (8 * i);
        }
        v
    }
}
