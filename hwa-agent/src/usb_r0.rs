// Pilote xHCI en R0 (apres ExitBootServices). Etape 1 : mise en route du controleur
// et lecture des ports. AUCUNE ecriture disque. Non teste sur materiel.
#![allow(dead_code)]

#[repr(C, align(4096))]
pub struct Pg(pub [u8; 4096]);

// Pages DMA statiques : 0 DCBAA, 1 anneau de commandes, 2 anneau d'evenements, 3 ERST
pub static mut XHCI_MEM: [Pg; 64] = [const { Pg([0; 4096]) }; 64];

/// Premiere page de la zone du peripherique courant (6 pages par peripherique, a partir de la page 16).
/// Decalages : 0 contexte d'entree, 1 contexte de sortie, 2 anneau EP0, 3 donnees,
/// 4 anneau d'interruption, 5 tampons de rapports.
pub static mut DEV_BASE: u64 = 16;

/// Adresse de la page `off` de la zone du peripherique courant.
pub unsafe fn dpg(off: u64) -> u64 {
    // SAFETY: adresse dans le tableau DMA statique (DEV_BASE + off < 64 par construction).
    unsafe { core::ptr::addr_of_mut!(XHCI_MEM) as u64 + (DEV_BASE + off) * 4096 }
}

/// Change de zone pour le peripherique suivant (base = numero de page, 16 + 6 * index).
pub unsafe fn set_device_base(base: u64) {
    // SAFETY: appele entre deux peripheriques, sans transfert en cours.
    unsafe {
        DEV_BASE = base;
        crate::usb_enum::reset_ep0();
    }
}

unsafe fn rd32(a: u64) -> u32 {
    // SAFETY: adresse MMIO du controleur xHCI, lue en volatile.
    unsafe { core::ptr::read_volatile(a as *const u32) }
}
unsafe fn wr32(a: u64, v: u32) {
    // SAFETY: adresse MMIO du controleur xHCI.
    unsafe { core::ptr::write_volatile(a as *mut u32, v) }
}
unsafe fn wr64(a: u64, v: u64) {
    // SAFETY: adresse MMIO du controleur xHCI, deux mots 32 bits.
    unsafe {
        wr32(a, v as u32);
        wr32(a + 4, (v >> 32) as u32);
    }
}

pub struct PortInfo {
    pub port: u8,
    pub portsc: u32,
}

/// Nombre maximal de ports racine lus (MaxPorts peut aller jusqu'a 255 : xHCI 5.3.1 ; 32 suffisent ici).
pub const MAX_PORTS_LUS: usize = 32;

pub struct Bringup {
    pub max_slots: u8,
    pub max_ports: u8,
    pub connected: [PortInfo; MAX_PORTS_LUS],
    pub nconnected: usize,
    /// true si le firmware n'a pas rendu le controleur et que le bit BIOS a ete force a 0.
    pub legacy_forced: bool,
}

const WAIT: u32 = 2_000_000;

/// Met en route le controleur xHCI situe a l'adresse BAR0 `bar`.
/// Retourne Err(message) si une etape echoue (aucune ecriture hors du controleur et des pages statiques).
pub unsafe fn xhci_bringup(bar: u64) -> Result<Bringup, &'static str> {
    // SAFETY: bar fourni par le scan PCI avant ExitBootServices.
    unsafe {
        let capl = core::ptr::read_volatile(bar as *const u8) as u64;
        let op = bar + capl;
        let hcs1 = rd32(bar + 4);
        let hcs2 = rd32(bar + 8);
        let max_slots = (hcs1 & 0xFF) as u8;
        let max_ports = ((hcs1 >> 24) & 0xFF) as u8;
        // xHCI 1.2c, 5.3.3 HCSPARAMS2 : Max Scratchpad Buffers Hi = bits 25:21, Lo = bits 31:27.
        // Total = (Hi << 5) | Lo.
        let max_scratch = ((((hcs2 >> 21) & 0x1F) << 5) | ((hcs2 >> 27) & 0x1F)) as u32;
        // Les tampons scratchpad occupent les pages 11 a 11+n-1 ; les zones des peripheriques
        // commencent a la page 16. Au-dela de 5 tampons ils se chevaucheraient (xHCI 4.20 : le
        // logiciel ne doit jamais ecrire dans un tampon scratchpad). Ce PC en demande 2.
        if max_scratch > 5 {
            return Err("scratchpad : trop de pages");
        }
        // PAGESIZE (xHCI 5.4.3, op+0x08) : bit 0 = pages de 4 Kio, seule taille geree ici.
        if rd32(bar + capl + 0x08) & 1 == 0 {
            return Err("xHCI : pages de 4 Kio non gerees");
        }
        let rtsoff = rd32(bar + 0x18) & !0x1F;
        let rt = bar + rtsoff as u64;

        // 0. Prise de possession par le systeme (USB Legacy Support, spec xHCI 4.22.1) :
        // le firmware peut garder le controleur ; on demande l'ownership puis on coupe les SMI.
        let hcc1 = rd32(bar + 0x10);
        let mut xoff = ((hcc1 >> 16) & 0xFFFF) as u64 * 4;
        let mut guard = 0;
        let mut legacy_forced = false;
        while xoff != 0 && guard < 64 {
            guard += 1;
            let cap = rd32(bar + xoff);
            if cap & 0xFF == 1 {
                // xHCI 7.1.1 : le systeme pose TOUJOURS son semaphore (bit 24), puis attend que
                // celui du BIOS (bit 16) retombe. La propriete est acquise quand OS = 1 et BIOS = 0.
                wr32(bar + xoff, cap | (1 << 24)); // OS owned
                let mut m = 0;
                while rd32(bar + xoff) & (1 << 16) != 0 {
                    m += 1;
                    if m > WAIT {
                        // Le firmware ne rend pas la main (SMI desactivees a ce stade). Comme Linux
                        // (xhci_handoff), on force le bit BIOS a 0 ; la propriete OS est deja posee.
                        // Signale dans le rapport par Bringup.legacy_forced.
                        wr32(bar + xoff, rd32(bar + xoff) & !(1 << 16));
                        legacy_forced = true;
                        break;
                    }
                }
                wr32(bar + xoff + 4, 0); // USBLEGCTLSTS : pas de SMI
            }
            let next = (cap >> 8) & 0xFF;
            if next == 0 {
                break;
            }
            xoff += next as u64 * 4;
        }

        // 1. Arret : RS = 0, attendre HCH = 1.
        // xHCI 5.4.2 (USBSTS bit 11, CNR) : aucune ecriture d'un registre operationnel tant que CNR = 1.
        let mut n = 0;
        while rd32(op + 4) & (1 << 11) != 0 {
            n += 1;
            if n > WAIT {
                return Err("xHCI : controleur pas pret (CNR)");
            }
        }
        wr32(op, rd32(op) & !1);
        n = 0;
        while rd32(op + 4) & 1 == 0 {
            n += 1;
            if n > WAIT {
                return Err("arret xHCI : HCH jamais a 1");
            }
        }
        // 2. Reinitialisation : HCRST, attendre HCRST = 0 et CNR = 0 (USBSTS bit 11)
        wr32(op, rd32(op) | 2);
        n = 0;
        while rd32(op) & 2 != 0 || rd32(op + 4) & (1 << 11) != 0 {
            n += 1;
            if n > WAIT {
                return Err("reinitialisation xHCI : HCRST/CNR bloque");
            }
        }
        // 3. Nombre d'emplacements actifs
        wr32(op + 0x38, max_slots as u32);

        // 4. DCBAA (page 0). Scratchpad (spec 4.20) : tableau de pointeurs vers des pages
        // (page 10), pages 11 et suivantes. DCBAA[0] pointe vers ce tableau.
        let base = core::ptr::addr_of_mut!(XHCI_MEM) as u64;
        let dcbaa = base;
        let cmd_ring = base + 4096;
        let evt_ring = base + 2 * 4096;
        let erst = base + 3 * 4096;
        if max_scratch > 0 {
            let arr = base + 10 * 4096;
            for i in 0..max_scratch as u64 {
                wr64(arr + i * 8, base + (11 + i) * 4096);
            }
            wr64(dcbaa, arr);
        }
        wr64(op + 0x30, dcbaa);

        // 5. Anneau de commandes (CRCR) avec RCS = 1
        wr64(op + 0x18, cmd_ring | 1);

        // 6. Anneau d'evenements : ERST (une entree : base, taille 256 TRB)
        let erst_ptr = erst as *mut u64;
        core::ptr::write_volatile(erst_ptr, evt_ring);
        core::ptr::write_volatile((erst + 8) as *mut u32, 256);
        let ir0 = rt + 0x20;
        // SAFETY: ecriture unique de l'adresse du registre ERDP, utilisee par wait_event.
        ERDP_REG = ir0 + 0x18;
        wr32(ir0 + 8, 1); // ERSTSZ
        wr64(ir0 + 0x18, evt_ring); // ERDP
        wr64(ir0 + 0x10, erst); // ERSTBA
        wr32(ir0, rd32(ir0) | 2); // IMAN.IE

        // 7. Demarrage : RS = 1, INTE = 1
        wr32(op, rd32(op) | 1 | 4);
        n = 0;
        while rd32(op + 4) & 1 != 0 {
            n += 1;
            if n > WAIT {
                return Err("demarrage xHCI : HCH reste a 1");
            }
        }

        // 8. Lecture des ports : PORTSC = op + 0x400 + 16 * (port - 1)
        let mut b = Bringup {
            max_slots,
            max_ports,
            connected: [const { PortInfo { port: 0, portsc: 0 } }; MAX_PORTS_LUS],
            nconnected: 0,
            legacy_forced,
        };
        for p in 0..max_ports.min(MAX_PORTS_LUS as u8) {
            let portsc = rd32(op + 0x400 + 16 * p as u64);
            if portsc & 1 != 0 && b.nconnected < MAX_PORTS_LUS {
                b.connected[b.nconnected] = PortInfo { port: p + 1, portsc };
                b.nconnected += 1;
            }
        }
        Ok(b)
    }
}

// ---------- HDA : sonde du codec (R0). Seule ecriture : commande PCI (memoire + maitre de bus) ----------

pub unsafe fn outl(port: u16, v: u32) {
    // SAFETY: ecriture d'un port d'E/S en CPL0.
    unsafe { core::arch::asm!("out dx, eax", in("dx") port, in("eax") v, options(nomem, nostack, preserves_flags)) };
}

pub unsafe fn inl(port: u16) -> u32 {
    let v: u32;
    // SAFETY: lecture d'un port d'E/S en CPL0.
    unsafe { core::arch::asm!("in eax, dx", out("eax") v, in("dx") port, options(nomem, nostack, preserves_flags)) };
    v
}

fn pci_addr(bus: u8, dev: u8, fun: u8, reg: u8) -> u32 {
    0x8000_0000u32 | ((bus as u32) << 16) | ((dev as u32) << 11) | ((fun as u32) << 8) | (reg as u32 & 0xFC)
}

pub unsafe fn pci_read32(bus: u8, dev: u8, fun: u8, reg: u8) -> u32 {
    // SAFETY: acces configuration PCI type 1 par les ports 0xCF8/0xCFC.
    unsafe {
        outl(0xCF8, pci_addr(bus, dev, fun, reg));
        inl(0xCFC)
    }
}

pub unsafe fn pci_write32(bus: u8, dev: u8, fun: u8, reg: u8, v: u32) {
    // SAFETY: ecriture de configuration PCI, utilisee uniquement pour la commande (registre 0x04).
    unsafe {
        outl(0xCF8, pci_addr(bus, dev, fun, reg));
        outl(0xCFC, v);
    }
}

#[derive(Clone, Copy)]
pub struct HdaProbe {
    pub cmd_after: u32,
    pub gctl: u32,
    pub statests: u16,
    pub vendor: u32,
    pub codec_found: bool,
}

/// Active memoire + maitre de bus du controleur HDA, sort du reset si besoin,
/// puis lit le fournisseur du codec 0 (verbe GET_PARAMETER, noeud 0, parametre 0).
pub unsafe fn hda_probe(bar: u64, bus: u8, dev: u8, fun: u8) -> HdaProbe {
    let mut res = HdaProbe { cmd_after: 0, gctl: 0, statests: 0, vendor: 0, codec_found: false };
    // SAFETY: BAR0 du controleur HDA ; acces MMIO en CPL0.
    unsafe {
        // 1. Commande PCI : bit 1 (memoire) et bit 2 (maitre de bus). On n'ecrit que les 16 bas (pas W1C du statut).
        let cmd = pci_read32(bus, dev, fun, 0x04);
        let cmd_new = (cmd | 0x6) & 0xFFFF;
        pci_write32(bus, dev, fun, 0x04, cmd_new);
        res.cmd_after = pci_read32(bus, dev, fun, 0x04) & 0xFFFF;

        // 2. GCTL bit 0 (CRST) : sortie du reset si necessaire
        let mut gctl = rd32(bar + 0x08);
        if gctl & 1 == 0 {
            wr32(bar + 0x08, gctl | 1);
            let mut n = 0;
            while rd32(bar + 0x08) & 1 == 0 {
                n += 1;
                if n > WAIT {
                    break;
                }
            }
            gctl = rd32(bar + 0x08);
            // HDA 1.0a, 4.3 : attendre au moins 521 us (25 trames) apres CRST lu a 1 avant de
            // considerer les codecs enumeres dans STATESTS.
            attente_tsc(8_000_000);
        }
        res.gctl = gctl;
        if gctl & 1 == 0 {
            return res; // controleur reste en reset : aucun registre n'est valide (HDA 3.3.7)
        }

        // 3. STATESTS (0x0E) : codecs presents
        res.statests = core::ptr::read_volatile((bar + 0x0E) as *const u16);
        let mask = res.statests & 0x7FFF;
        if mask == 0 {
            return res;
        }
        let cad = mask.trailing_zeros() as u32;

        // 4. Interface immediate : attendre ICB=0, envoyer le verbe GET_PARAMETER (cad, noeud 0, 0xF00, param 0)
        let mut n = 0;
        while core::ptr::read_volatile((bar + 0x68) as *const u16) & 1 != 0 {
            n += 1;
            if n > WAIT {
                return res;
            }
        }
        let verb = (cad << 28) | (0xF00u32 << 8) | 0x00;
        // HDA 3.4.3 : IRV (bit 1, RW1C) doit etre efface avant chaque nouvelle commande.
        core::ptr::write_volatile((bar + 0x68) as *mut u16, 0x0002);
        wr32(bar + 0x60, verb);
        core::ptr::write_volatile((bar + 0x68) as *mut u16, 0x0001);
        n = 0;
        while core::ptr::read_volatile((bar + 0x68) as *const u16) & 0x0002 == 0 {
            n += 1;
            if n > WAIT {
                return res;
            }
        }
        res.vendor = rd32(bar + 0x64);
        res.codec_found = true;
    }
    res
}

// ---------- HDA : arbre du codec (commandes de lecture uniquement, sans allocation) ----------

/// Noeud racine du codec : nombre de groupes de fonctions (verbe GET_PARAMETER 04h).
/// Quatre essais espaces d'environ 1 ms : un codec peut ne pas repondre tout de suite apres le reset.
/// Retourne None si aucune reponse non nulle.
pub unsafe fn codec_root(bar: u64, cad: u32) -> Option<u32> {
    // SAFETY: verbes de lecture sur le controleur HDA, CPL0.
    unsafe {
        for _ in 0..4 {
            if let Some(v) = hda_cmd(bar, cad, 0, (0xF00 << 8) | 0x04) {
                if v != 0 {
                    return Some(v);
                }
            }
            attente_tsc(8_000_000);
        }
        None
    }
}

/// Une commande verbe HDA (interface immediate). Retourne la reponse ou None en cas de delai.
pub unsafe fn hda_cmd(bar: u64, cad: u32, node: u32, verb: u32) -> Option<u32> {
    // SAFETY: registres MMIO du controleur HDA, CPL0.
    unsafe {
        let mut n = 0;
        while core::ptr::read_volatile((bar + 0x68) as *const u16) & 1 != 0 {
            n += 1;
            if n > WAIT {
                return None;
            }
        }
        // HDA 3.4.3 : IRV (bit 1, RW1C) doit etre efface avant chaque nouvelle commande, sinon
        // l'attente ci-dessous sort tout de suite et on relit la reponse de la commande precedente.
        core::ptr::write_volatile((bar + 0x68) as *mut u16, 0x0002);
        wr32(bar + 0x60, (cad << 28) | (node << 20) | verb);
        core::ptr::write_volatile((bar + 0x68) as *mut u16, 0x0001);
        n = 0;
        while core::ptr::read_volatile((bar + 0x68) as *const u16) & 0x0002 == 0 {
            n += 1;
            if n > WAIT {
                return None;
            }
        }
        Some(rd32(bar + 0x64))
    }
}

/// Attente active d'au moins `ticks` unites du compteur TSC (sans service UEFI).
/// 8 000 000 ticks = au moins 1 ms tant que le TSC bat a moins de 8 GHz.
pub fn attente_tsc(ticks: u64) {
    // SAFETY: lecture du compteur de temps du processeur, sans effet.
    let t0 = unsafe { core::arch::x86_64::_rdtsc() };
    loop {
        let t = unsafe { core::arch::x86_64::_rdtsc() };
        if t.wrapping_sub(t0) >= ticks {
            break;
        }
        core::hint::spin_loop();
    }
}

/// Arbre complet du codec, lecture seule, sans limite de nombre de lignes : groupes de fonctions,
/// chaque noeud avec ses capacites, sa liste de connexions et, pour une broche, sa configuration.
/// A utiliser AVANT ExitBootServices (la sortie a l'ecran ne suffit pas pour garder le resultat).
pub unsafe fn hda_arbre_complet(bar: u64, cad: u32) -> alloc::vec::Vec<alloc::string::String> {
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;
    let mut out: Vec<String> = Vec::new();
    // SAFETY: verbes de lecture uniquement sur le controleur HDA deja initialise.
    unsafe {
        let root = match codec_root(bar, cad) {
            Some(v) => v,
            None => {
                out.push(String::from("CODEC : pas de reponse"));
                return out;
            }
        };
        let fg_start = (root >> 16) & 0xFF;
        let fg_count = root & 0xFF;
        out.push(format!("CODEC {:#x} : groupes de fonctions {:02x}+{:02x}", cad, fg_start, fg_count));
        for fg in fg_start..fg_start + fg_count {
            let fgt = hda_cmd(bar, cad, fg, (0xF00 << 8) | 0x05).unwrap_or(0) & 0xFF;
            let fgn = hda_cmd(bar, cad, fg, (0xF00 << 8) | 0x04).unwrap_or(0);
            let ws = (fgn >> 16) & 0xFF;
            let wc = fgn & 0xFF;
            out.push(format!("FG {:02x} type {:02x} widgets {:02x}+{:02x}", fg, fgt, ws, wc));
            for w in ws..ws + wc {
                let cap = hda_cmd(bar, cad, w, (0xF00 << 8) | 0x09).unwrap_or(0);
                let wt = ((cap >> 20) & 0xF) as usize;
                let nom = WIDGET_NAMES[wt];
                let len_raw = hda_cmd(bar, cad, w, (0xF00 << 8) | 0x0E).unwrap_or(0);
                let n = (len_raw & 0x7F) as u32;
                let mut ligne = format!(
                    "  noeud {:02x} {} cap {:08x} connexions {}",
                    w,
                    core::str::from_utf8(nom).unwrap_or("?"),
                    cap,
                    n
                );
                if wt == 4 {
                    let cfg = hda_cmd(bar, cad, w, 0xF1C << 8).unwrap_or(0);
                    let dev = ((cfg >> 20) & 0xF) as usize;
                    ligne.push_str(&format!(
                        " | broche {} conn {} config {:08x}",
                        core::str::from_utf8(DEV_NAMES[dev]).unwrap_or("?"),
                        (cfg >> 30) & 0x3,
                        cfg
                    ));
                }
                out.push(ligne);
                // Liste de connexions : le verbe F02h renvoie 4 entrees a la fois (une par octet).
                let mut k = 0u32;
                let mut liste = String::from("    entrees :");
                while k < n {
                    let quad = hda_cmd(bar, cad, w, (0xF02 << 8) | (k & !3)).unwrap_or(0);
                    for b in 0..4 {
                        let idx = k + b;
                        if idx >= n {
                            break;
                        }
                        let e = (quad >> (8 * b)) & 0xFF;
                        liste.push_str(&format!(" {:02x}", e));
                    }
                    k += 4;
                }
                if n > 0 {
                    out.push(liste);
                }
            }
        }
    }
    out
}

/// Etat complet du controleur xHCI, lecture seule, avant ExitBootServices : registres de capacite,
/// liste des capacites etendues (dont USB Legacy Support : qui possede le controleur), registres
/// operationnels, runtime et PORTSC de chaque port. Aucune ecriture.
pub unsafe fn xhci_etat(bar: u64) -> alloc::vec::Vec<alloc::string::String> {
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;
    let mut o: Vec<String> = Vec::new();
    // SAFETY: lectures MMIO du controleur xHCI (BAR0 trouve par le scan PCI).
    unsafe {
        let capl = core::ptr::read_volatile(bar as *const u8) as u64;
        let hciv = core::ptr::read_volatile((bar + 2) as *const u16);
        let op = bar + capl;
        let hcs1 = rd32(bar + 4);
        let hcs2 = rd32(bar + 8);
        let hcs3 = rd32(bar + 0xC);
        let hcc1 = rd32(bar + 0x10);
        let dboff = rd32(bar + 0x14);
        let rtsoff = rd32(bar + 0x18);
        let hcc2 = rd32(bar + 0x1C);
        o.push(format!("xHCI BAR0 {:#x} : CAPLENGTH {:#x} HCIVERSION {:#06x}", bar, capl, hciv));
        o.push(format!("  HCSPARAMS1 {:08x} HCSPARAMS2 {:08x} HCSPARAMS3 {:08x}", hcs1, hcs2, hcs3));
        o.push(format!("  HCCPARAMS1 {:08x} DBOFF {:08x} RTSOFF {:08x} HCCPARAMS2 {:08x}", hcc1, dboff, rtsoff, hcc2));
        o.push(format!("  ports {} slots {} interruptions {}", (hcs1 >> 24) & 0xFF, hcs1 & 0xFF, (hcs1 >> 8) & 0x7FF));
        // Capacites etendues (xHCI 7.1) : chaine a partir de HCCPARAMS1.xECP (bits 31:16, en DWORD).
        let mut xoff = ((hcc1 >> 16) & 0xFFFF) as u64 * 4;
        let mut garde = 0;
        while xoff != 0 && garde < 32 {
            garde += 1;
            let cap = rd32(bar + xoff);
            let id = cap & 0xFF;
            let suivant = (cap >> 8) & 0xFF;
            if id == 1 {
                let ctl = rd32(bar + xoff + 4);
                o.push(format!(
                    "  USB Legacy Support @{:#x} : BIOS owned {} OS owned {} | USBLEGCTLSTS {:08x}",
                    bar + xoff,
                    (cap >> 16) & 1,
                    (cap >> 24) & 1,
                    ctl
                ));
            } else if id == 2 {
                let ver = rd32(bar + xoff);
                o.push(format!("  Supported Protocol @{:#x} : {:08x}", bar + xoff, ver));
            } else {
                o.push(format!("  capacite {:#x} @{:#x} : {:08x}", id, bar + xoff, cap));
            }
            if suivant == 0 {
                break;
            }
            xoff += suivant as u64 * 4;
        }
        o.push(format!(
            "  USBCMD {:08x} USBSTS {:08x} PAGESIZE {:08x} DNCTRL {:08x} CONFIG {:08x}",
            rd32(op),
            rd32(op + 4),
            rd32(op + 8),
            rd32(op + 0x14),
            rd32(op + 0x38)
        ));
        o.push(format!(
            "  DCBAAP {:08x}{:08x} CRCR {:08x}{:08x}",
            rd32(op + 0x34),
            rd32(op + 0x30),
            rd32(op + 0x1C),
            rd32(op + 0x18)
        ));
        let maxp = ((hcs1 >> 24) & 0xFF) as u64;
        for p in 0..maxp.min(MAX_PORTS_LUS as u64) {
            let portsc = rd32(op + 0x400 + 16 * p);
            o.push(format!(
                "  port {:02} PORTSC {:08x} CCS {} PED {} vitesse {} PLS {}",
                p + 1,
                portsc,
                portsc & 1,
                (portsc >> 1) & 1,
                (portsc >> 10) & 0xF,
                (portsc >> 5) & 0xF
            ));
        }
        // Registres de capacite bruts (32 premiers octets, pour verification manuelle).
        for ligne in 0..2u64 {
            let mut hex = String::new();
            for k in 0..16u64 {
                hex.push_str(&format!("{:02x} ", core::ptr::read_volatile((bar + ligne * 16 + k) as *const u8)));
            }
            o.push(format!("  octets {:02x} : {}", ligne * 16, hex.trim_end()));
        }
    }
    o
}

/// Ligne de texte de taille fixe (pas d'allocation : utilisable apres ExitBootServices).
#[derive(Clone, Copy)]
pub struct Line {
    pub b: [u8; 40],
    pub p: usize,
}

impl Line {
    pub fn new() -> Line {
        Line { b: [b' '; 40], p: 0 }
    }
    pub fn s(&mut self, t: &[u8]) -> &mut Line {
        for &c in t {
            if self.p < 40 {
                self.b[self.p] = c;
                self.p += 1;
            }
        }
        self
    }
    pub fn h(&mut self, v: u32, digits: usize) -> &mut Line {
        let hex = b"0123456789ABCDEF";
        for i in (0..digits).rev() {
            let c = hex[((v >> (4 * i)) & 0xF) as usize];
            if self.p < 40 {
                self.b[self.p] = c;
                self.p += 1;
            }
        }
        self
    }
}

pub struct HdaTree {
    pub lines: [Line; 12],
    pub n: usize,
}

const DEV_NAMES: [&[u8]; 16] = [
    b"LINE OUT", b"SPEAKER", b"HP OUT", b"CD", b"SPDIF OUT", b"DIG OUT", b"MODEM LINE", b"MODEM HS",
    b"LINE IN", b"AUX", b"MIC IN", b"TELEPHONY", b"SPDIF IN", b"DIG IN", b"OTHER", b"OTHER",
];
const WIDGET_NAMES: [&[u8]; 16] = [
    b"AUDIO OUT", b"AUDIO IN", b"MIXER", b"SELECTOR", b"PIN", b"POWER", b"VOLUME", b"BEEP",
    b"?8", b"?9", b"?A", b"?B", b"?C", b"?D", b"?E", b"VENDOR",
];

/// Reinitialise le port (PR=1, attend PRC) puis envoie une commande Enable Slot (TRB type 9).
/// Retourne le numero d'emplacement (Slot ID). Spec xHCI 1.2c, sections 4.3.1 et 4.6.3.
/// Ecritures : PORTSC (reset du port), anneau de commandes (page 1), doorbell 0.
pub unsafe fn enable_slot(bar: u64, port: u8) -> Result<u8, &'static str> {
    // SAFETY: MMIO xHCI deja initialise par xhci_bringup ; pages DMA statiques.
    unsafe {
        let capl = core::ptr::read_volatile(bar as *const u8) as u64;
        let op = bar + capl;
        let portsc = op + 0x400 + 16 * (port as u64 - 1);
        let v = rd32(portsc);
        if v & 1 == 0 {
            return Err("port sans peripherique");
        }
        // Ecriture : on met a 0 les bits RW1C (17..23) et PED (bit 1) pour ne rien effacer par erreur.
        let keep = !(0x2u32 | 0x00FE_0000);
        wr32(portsc, (v & keep) | (1 << 4)); // PR = 1
        let mut n = 0;
        while rd32(portsc) & (1 << 21) == 0 {
            n += 1;
            if n > WAIT {
                return Err("reset port : PRC absent");
            }
        }
        wr32(portsc, (rd32(portsc) & keep) | (1 << 21)); // efface PRC
        if rd32(portsc) & 0x2 == 0 {
            return Err("port non active apres reset");
        }
        enable_slot_cmd(bar)
    }
}

/// Commande Enable Slot seule (sans reset de port), a l'index suivant de l'anneau de commandes.
/// Utilisable pour un peripherique derriere un hub (le port racine ne doit pas etre reinitialise).
pub unsafe fn enable_slot_cmd(bar: u64) -> Result<u8, &'static str> {
    // SAFETY: anneau de commandes (page 1) et doorbell du controleur.
    unsafe {
        let idx = crate::usb_enum::next_cmd_index();
        let cmd = core::ptr::addr_of_mut!(XHCI_MEM) as u64 + 4096;
        let trb = (cmd + idx * 16) as *mut u32;
        core::ptr::write_volatile(trb, 0);
        core::ptr::write_volatile(trb.add(1), 0);
        core::ptr::write_volatile(trb.add(2), 0);
        core::ptr::write_volatile(trb.add(3), (9 << 10) | crate::usb_enum::cmd_cycle());
        let dboff = (rd32(bar + 0x14) & !0x3) as u64;
        wr32(bar + dboff, 0);
        let ev = wait_event(&[33])?;
        if ev.code == 1 {
            Ok(ev.slot as u8)
        } else {
            Err("Enable Slot refuse")
        }
    }
}

/// Zone DMA suivante (numero de page de base). Chaque peripherique en prend une.
pub static mut ZONE_NEXT: u64 = 16;

/// Reserve une zone DMA et la rend courante. Erreur si la zone statique est pleine.
pub unsafe fn zone_alloc() -> Result<u64, &'static str> {
    // SAFETY: compteur statique, appele hors transfert.
    unsafe {
        if ZONE_NEXT + 6 > 64 {
            return Err("plus de zone DMA");
        }
        let base = ZONE_NEXT;
        ZONE_NEXT += 6;
        set_device_base(base);
        Ok(base)
    }
}

/// Evenement recu dans l'anneau d'evenements.
#[derive(Clone, Copy)]
pub struct Ev {
    pub ty: u32,
    pub code: u32,
    pub slot: u32,
    pub ep: u32,
    pub p0: u64,
}

/// Curseur de lecture de l'anneau d'evenements (page 2). Avance a chaque evenement consomme.
pub static mut EVT_NEXT: u64 = 0;
/// Cycle bit attendu (CCS) de l'anneau d'evenements : il s'inverse a chaque tour complet.
pub static mut EVT_CCS: u32 = 1;
/// Nombre de ports avec un peripherique connecte (CCS = 1), lu sans modifier le controleur.
pub unsafe fn ports_connected(bar: u64) -> u8 {
    // SAFETY: lecture seule des registres PORTSC.
    unsafe {
        let capl = core::ptr::read_volatile(bar as *const u8) as u64;
        let op = bar + capl;
        let maxp = ((rd32(bar + 4) >> 24) & 0xFF) as u64;
        let mut n = 0u8;
        for p in 0..maxp.min(32) {
            if rd32(op + 0x400 + 16 * p) & 1 != 0 {
                n += 1;
            }
        }
        n
    }
}

/// Adresse du registre ERDP du premier interrupteur (0 = pas encore initialise).
pub static mut ERDP_REG: u64 = 0;

/// Attend un evenement dont le type est dans `want` (les autres types sont consommes et ignores).
pub unsafe fn wait_event(want: &[u32]) -> Result<Ev, &'static str> {
    // SAFETY: appel de wait_event_filt sans filtre de slot ni d'endpoint.
    unsafe { wait_event_filt(want, None, None) }
}

/// Attend un evenement de `want`, en ne retenant que ceux du slot `slot` et de l'endpoint `dci`
/// (xHCI 6.4.2.1 : un evenement de transfert porte Slot ID et Endpoint ID).
/// Les evenements qui ne correspondent pas sont consommes et ignores.
pub unsafe fn wait_event_filt(want: &[u32], slot: Option<u8>, dci: Option<u8>) -> Result<Ev, &'static str> {
    // SAFETY: anneau d'evenements statique, lecture seule des TRB ecrits par le controleur.
    unsafe {
        let evt = core::ptr::addr_of_mut!(XHCI_MEM) as u64 + 2 * 4096;
        let mut n = 0usize;
        loop {
            let idx = EVT_NEXT;
            let e = (evt + idx * 16) as *const u32;
            let d3 = core::ptr::read_volatile(e.add(3));
            // Le TRB est valide quand son cycle bit = CCS attendu (CCS s'inverse a chaque tour).
            if (d3 & 1) == EVT_CCS {
                // Avance le curseur, avec retour au debut de l'anneau (256 TRB) et inversion du CCS.
                let next = idx + 1;
                if next >= 256 {
                    EVT_NEXT = 0;
                    EVT_CCS ^= 1;
                } else {
                    EVT_NEXT = next;
                }
                // ERDP : adresse du prochain TRB a lire (bit 3 = EHB). Sans cela le controleur
                // considere l'anneau plein et cesse d'en produire.
                if ERDP_REG != 0 {
                    wr64(ERDP_REG, (evt + (EVT_NEXT * 16)) | (1 << 3));
                }
                let ty = (d3 >> 10) & 0x3F;
                let ev_slot = d3 >> 24;
                let ev_ep = (d3 >> 16) & 0x1F;
                let bon_slot = slot.map_or(true, |s| ev_slot == s as u32);
                let bon_ep = dci.map_or(true, |d| ev_ep == d as u32);
                if want.contains(&ty) && bon_slot && bon_ep {
                    let d2 = core::ptr::read_volatile(e.add(2));
                    let p0 = (core::ptr::read_volatile(e.add(1)) as u64) << 32 | core::ptr::read_volatile(e) as u64;
                    return Ok(Ev { ty, code: d2 >> 24, slot: ev_slot, ep: ev_ep, p0 });
                }
                continue;
            }
            n += 1;
            if n > WAIT as usize {
                return Err("delai depasse : evenement absent");
            }
        }
    }
}

/// Lit l'arbre du codec `cad` : groupes de fonctions et widgets (lecture seule).
/// Pour chaque broche (PIN) : peripherique branche et connectivite.
pub unsafe fn hda_tree(bar: u64, cad: u32) -> HdaTree {
    let mut t = HdaTree { lines: [Line::new(); 12], n: 0 };
    // SAFETY: lecture de verbes HDA uniquement (GET_PARAMETER, GET_CONFIG_DEFAULT).
    unsafe {
        let push = |t: &mut HdaTree, l: &Line| {
            if t.n < 12 {
                t.lines[t.n] = *l;
                t.n += 1;
            }
        };
        let root = match codec_root(bar, cad) {
            Some(v) => v,
            None => {
                let mut l = Line::new();
                l.s(b"CODEC : pas de reponse (4 essais)");
                push(&mut t, &l);
                return t;
            }
        };
        // HDA 1.0a, 7.3.4.3 (Subordinate Node Count, figure 83) : bits 23:16 = noeud de depart,
        // bits 7:0 = nombre de noeuds (meme ordre que dans hda_play.rs).
        let fg_start = (root >> 16) & 0xFF;
        let fg_count = root & 0xFF;
        let mut l = Line::new();
        l.s(b"CODEC NOEUDS FG ").h(fg_start, 2).s(b" +").h(fg_count, 2);
        push(&mut t, &l);
        for fg in fg_start..fg_start + fg_count {
            let fgt = hda_cmd(bar, cad, fg, (0xF00 << 8) | 0x05).unwrap_or(0) & 0xFF;
            let fgs = hda_cmd(bar, cad, fg, (0xF00 << 8) | 0x04).unwrap_or(0);
            let ws = (fgs >> 16) & 0xFF;
            let wc = fgs & 0xFF;
            let mut l = Line::new();
            l.s(b"FG ").h(fg, 2).s(b" TYPE ").h(fgt, 2).s(b" WIDGETS ").h(ws, 2).s(b"+").h(wc, 2);
            push(&mut t, &l);
            if fgt != 1 {
                continue; // seulement les groupes audio
            }
            for w in ws..ws + wc {
                let cap = hda_cmd(bar, cad, w, (0xF00 << 8) | 0x09).unwrap_or(0);
                let wt = ((cap >> 20) & 0xF) as usize;
                if wt == 4 {
                    // broche : configuration par defaut (verbe F1C)
                    let cfg = hda_cmd(bar, cad, w, 0xF1C << 8).unwrap_or(0);
                    let dev = ((cfg >> 20) & 0xF) as usize;
                    let conn = (cfg >> 30) & 0x3;
                    let mut l = Line::new();
                    l.s(b"PIN ").h(w, 2).s(b" ").s(DEV_NAMES[dev]).s(b" CONN ").h(conn, 1);
                    push(&mut t, &l);
                } else {
                    let mut l = Line::new();
                    l.s(b"WID ").h(w, 2).s(b" ").s(WIDGET_NAMES[wt]);
                    push(&mut t, &l);
                }
            }
        }
    }
    t
}
