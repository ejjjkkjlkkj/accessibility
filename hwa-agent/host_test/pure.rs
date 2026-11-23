#![allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kbd { pub cfg_value: u8, pub iface: u8, pub ep_addr: u8, pub ep_mps: u16, pub ep_interval: u8, pub dci: u8 }
pub fn find_keyboard(cfg: &[u8]) -> Option<Kbd> {
    let total = (cfg[2] as usize) | ((cfg[3] as usize) << 8);
    let total = total.min(cfg.len());
    let cfg_value = cfg[5];
    let mut p = 0usize;
    let mut in_kbd = false;
    let mut iface = 0u8;
    while p + 2 <= total {
        let len = cfg[p] as usize;
        let ty = cfg[p + 1];
        if len < 2 || p + len > total {
            break;
        }
        if ty == 4 && len >= 9 {
            iface = cfg[p + 2];
            in_kbd = cfg[p + 5] == 3 && cfg[p + 6] == 1 && cfg[p + 7] == 1;
        } else if ty == 5 && len >= 7 && in_kbd {
            let addr = cfg[p + 2];
            let attr = cfg[p + 3];
            if addr & 0x80 != 0 && attr & 3 == 3 {
                let mps = (cfg[p + 4] as u16) | ((cfg[p + 5] as u16) << 8);
                let dci = (addr & 0xF) * 2 + 1;
                return Some(Kbd { cfg_value, iface, ep_addr: addr, ep_mps: mps & 0x7FF, ep_interval: cfg[p + 6], dci });
            }
        }
        p += len;
    }
    None
}

/// Configure le peripherique, passe en protocole boot, puis ouvre le point de terminaison clavier.
pub unsafe fn setup_keyboard(bar: u64, port: u8, slot: u8, k: Kbd) -> Result<(), &'static str> {
    // SAFETY: MMIO xHCI deja initialise ; pages DMA statiques.
    unsafe {
        // SET_CONFIGURATION
        let sc: u64 = (9u64 << 8) | ((k.cfg_value as u64) << 16);
        control(bar, slot, sc, 0)?;
        // SET_PROTOCOL = 0 (boot), sur l'interface clavier
        let sp: u64 = 0x21 | (0x0Bu64 << 8) | ((k.iface as u64) << 32);
        control(bar, slot, sp, 0)?;

        // Configure Endpoint : contexte d'entree avec emplacement + point de terminaison
        let inp = pg(4);
        let outc = pg(5);
        zero_page(inp);
        let capl = core::ptr::read_volatile(bar as *const u8) as u64;
        let _ = capl;
        let hcc1 = r32(bar + 0x10);
        let ctx: u64 = if (hcc1 >> 2) & 1 == 1 { 64 } else { 32 };
        let portsc = r32(bar + core::ptr::read_volatile(bar as *const u8) as u64 + 0x400 + 16 * (port as u64 - 1));
        let speed = (portsc >> 10) & 0xF;
        // Controle : A0 (emplacement) + A(dci)
        w32(inp + 4, 0x1 | (1u32 << k.dci));
        // Emplacement : entrees = dci, vitesse, port racine
        w32(inp + ctx, ((k.dci as u32) << 27) | (speed << 20));
        w32(inp + ctx + 4, (port as u32) << 16);
        // Point de terminaison (au rang dci) : interruption IN (type 7), intervalle, MPS
        let ep = inp + (k.dci as u64 + 1) * ctx;
        let interval = interrupt_interval(speed, k.ep_interval);
        w32(ep, (interval as u32) << 16);
        w32(ep + 4, (7 << 3) | (3 << 1) | ((k.ep_mps as u32) << 16));
        let ring = pg(8);
        zero_page(ring);
        w64(ep + 8, ring | 1);
        w32(ep + 16, 8 | ((k.ep_mps as u32) << 16));
        let _ = outc;

        // Anneau de commandes (page 1) : commande Configure Endpoint (type 12) a l'index suivant
        let idx = crate::usb_enum::next_cmd_index();
        let _ = portsc;
        trb_c(pg(1), idx, inp, 0, (12 << 10) | ((slot as u32) << 24), 1);
        w32(bar + dboff(bar), 0);
        let ev = wait_event(&[33])?;
        if ev.code != 1 {
            return Err("Configure Endpoint refuse");
        }
        INT_IDX = 0;
        INT_CYC = 1;
        Ok(())
    }
}

/// Intervalle de l'anneau (valeur du champ Interval du contexte) selon la vitesse.
fn interrupt_interval(speed: u32, b_interval: u8) -> u8 {
    match speed {
        // Haute / super vitesse : bInterval est deja un exposant (2^(n-1) microtrames)
        3 | 4 => b_interval.saturating_sub(1).min(15),
        // Pleine / basse vitesse : bInterval en millisecondes -> 2^n microtrames
        _ => {
            let mut v = (b_interval as u32).max(1) * 8;
            let mut n = 0u8;
            while v > 1 {
                v >>= 1;
                n += 1;
            }
            n.min(15)
        }
    }
}

/// Met un TRB normal dans l'anneau d'interruption (tampon = 8 octets par index).
unsafe fn queue_report(slot_dci: u8, bar: u64) {
    // SAFETY: anneau et tampons statiques ; doorbell du peripherique.
    unsafe {
        let ring = pg(8);
        let idx = INT_IDX;
        let cyc = INT_CYC;
        trb_c(ring, idx, pg(9) + idx * 8, 8, (1 << 10) | (1 << 5), cyc);
        INT_IDX += 1;
        if INT_IDX == 15 {
            // Lien vers le debut de l'anneau, avec bascule du bit de cycle
            trb_c(ring, 15, ring, 0, (6 << 10) | (1 << 1), cyc);
            INT_IDX = 0;
            INT_CYC ^= 1;
        }
        let _ = slot_dci;
        let _ = bar;
    }
}

/// Premier rapport : met un TRB en attente. A appeler une fois apres setup_keyboard.
pub unsafe fn start(bar: u64, slot: u8, dci: u8) {
    // SAFETY: appels MMIO / DMA statiques.
    unsafe {
        queue_report(dci, bar);
        w32(bar + dboff(bar) + slot as u64 * 4, dci as u32);
    }
}

/// Attend le prochain rapport clavier (8 octets), remet un TRB en attente, retourne le rapport.
pub unsafe fn next_report(bar: u64, slot: u8, dci: u8) -> Result<[u8; 8], &'static str> {
    // SAFETY: evenements et tampons statiques.
    unsafe {
        let ev = wait_event(&[32])?;
        if ev.code != 1 && ev.code != 13 {
            return Err("rapport clavier en erreur");
        }
        // Index du TRB termine : (adresse TRB - base de l'anneau) / 16
        let base = pg(8);
        let k = ((ev.p0.wrapping_sub(base)) / 16).min(14);
        let mut rep = [0u8; 8];
        for (i, b) in rep.iter_mut().enumerate() {
            *b = core::ptr::read_volatile((pg(9) + k * 8 + i as u64) as *const u8);
        }
        queue_report(dci, bar);
        w32(bar + dboff(bar) + slot as u64 * 4, dci as u32);
        Ok(rep)
    }
}

/// Convertit un code d'usage HID (protocole boot) en caractere ASCII. None si non imprimable.
/// modifier : octet 0 du rapport (bit 1 et bit 5 = Shift).
pub fn key_char(modifier: u8, code: u8) -> Option<u8> {
    let shift = modifier & 0x22 != 0;
    match code {
        0x04..=0x1D => {
            let c = b'a' + (code - 0x04);
            Some(if shift { c.to_ascii_uppercase() } else { c })
        }
        0x1E..=0x26 => {
            let base = b"123456789"[(code - 0x1E) as usize];
            Some(if shift { b"!@#$%^&*("[(code - 0x1E) as usize] } else { base })
        }
        0x27 => Some(if shift { b')' } else { b'0' }),
        0x2C => Some(b' '),
        0x2D => Some(if shift { b'_' } else { b'-' }),
        0x2E => Some(if shift { b'+' } else { b'=' }),
        0x36 => Some(if shift { b'<' } else { b',' }),
        0x37 => Some(if shift { b'>' } else { b'.' }),
        0x38 => Some(if shift { b'?' } else { b'/' }),
        0x28 => Some(b'\n'), // Entree
        0x2A => Some(0x08),  // Retour arriere
        _ => None,
    }
}


