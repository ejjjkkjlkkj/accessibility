// Clavier USB (HID, protocole boot) : configuration, point de terminaison d'interruption,
// lecture des rapports. Reference : spec xHCI 1.2c (4.6.6 Configure Endpoint, 4.10 transferts),
// USB 2.0 chapitre 9, USB HID 1.11 (appendice B, protocole boot).
// Pages DMA : 4 contexte d'entree, 5 contexte de sortie, 6 anneau EP0, 7 donnees,
//             8 anneau d'interruption, 9 tampons de rapports (8 octets par TRB).
use crate::usb_r0::{dpg, wait_event, wait_event_filt, XHCI_MEM};

static mut INT_IDX: u64 = 0; // index courant dans l'anneau d'interruption (0..14)
static mut INT_CYC: u32 = 1; // bit de cycle courant du producteur

unsafe fn pg(i: u64) -> u64 {
    // SAFETY: page du tableau DMA statique.
    core::ptr::addr_of_mut!(XHCI_MEM) as u64 + i * 4096
}
unsafe fn w32(a: u64, v: u32) {
    // SAFETY: memoire DMA statique, alignee.
    unsafe { core::ptr::write_volatile(a as *mut u32, v) }
}
unsafe fn w64(a: u64, v: u64) {
    // SAFETY: memoire DMA statique, alignee sur 8.
    unsafe { core::ptr::write_volatile(a as *mut u64, v) }
}
unsafe fn r32(a: u64) -> u32 {
    // SAFETY: registre MMIO xHCI.
    unsafe { core::ptr::read_volatile(a as *const u32) }
}
unsafe fn zero_page(a: u64) {
    for i in 0..512u64 {
        // SAFETY: page DMA de 4096 octets.
        unsafe { w64(a + i * 8, 0) };
    }
}
unsafe fn dboff(bar: u64) -> u64 {
    // SAFETY: registre MMIO xHCI.
    unsafe { (r32(bar + 0x14) & !0x3) as u64 }
}

/// TRB avec un bit de cycle explicite.
unsafe fn trb_c(ring: u64, idx: u64, param: u64, status: u32, control: u32, cyc: u32) {
    // SAFETY: anneau DMA statique, idx < 256.
    unsafe {
        w64(ring + idx * 16, param);
        w32(ring + idx * 16 + 8, status);
        w32(ring + idx * 16 + 12, control | cyc);
    }
}

/// Transfert de controle sur EP0 : SETUP, puis DATA IN de `data_len` octets (si > 0), puis STATUS.
/// Les octets lus restent dans la page 7.
pub unsafe fn control(bar: u64, slot: u8, setup: u64, data_len: u32) -> Result<(), &'static str> {
    // SAFETY: MMIO xHCI deja initialise ; anneau EP0 statique.
    unsafe {
        let ep0 = dpg(2);
        if data_len == 0 {
            // Pas de donnees (TRT = 0), statut en IN
            let i = crate::usb_enum::ep0_take(2)?;
            trb_c(ep0, i, setup, 8, (2 << 10) | (1 << 6), 1);
            trb_c(ep0, i + 1, 0, 0, (4 << 10) | (1 << 5) | (1 << 16), 1);
        } else {
            // DATA IN (TRT = 3), statut en OUT
            let i = crate::usb_enum::ep0_take(3)?;
            trb_c(ep0, i, setup, 8, (2 << 10) | (1 << 6) | (3 << 16), 1);
            trb_c(ep0, i + 1, dpg(3), data_len, (3 << 10) | (1 << 16), 1);
            trb_c(ep0, i + 2, 0, 0, (4 << 10) | (1 << 5), 1);
        }
        w32(bar + dboff(bar) + slot as u64 * 4, 1);
        // Evenement de transfert du bon slot et du endpoint EP0 (DCI 1).
        let ev = wait_event_filt(&[32], Some(slot), Some(1))?;
        if ev.code != 1 && ev.code != 13 {
            return Err("transfert de controle en erreur");
        }
        Ok(())
    }
}

/// Lit une valeur de chaine de configuration : d'abord 9 octets, puis la taille totale.
pub unsafe fn get_config(bar: u64, slot: u8) -> Result<[u8; 256], &'static str> {
    // SAFETY: appels de controle EP0 ; page 7 en lecture.
    unsafe {
        let s9: u64 = 0x80 | (6u64 << 8) | (0x0200u64 << 16) | (9u64 << 48);
        control(bar, slot, s9, 9)?;
        let total = (core::ptr::read_volatile((dpg(3) + 2) as *const u8) as u32)
            | ((core::ptr::read_volatile((dpg(3) + 3) as *const u8) as u32) << 8);
        let total = total.min(255).max(9);
        let sfull: u64 = 0x80 | (6u64 << 8) | (0x0200u64 << 16) | ((total as u64) << 48);
        control(bar, slot, sfull, total)?;
        let mut out = [0u8; 256];
        for k in 0..total as usize {
            out[k] = core::ptr::read_volatile((dpg(3) + k as u64) as *const u8);
        }
        Ok(out)
    }
}

/// Informations du clavier trouve dans la configuration.
#[derive(Clone, Copy)]
pub struct Kbd {
    pub cfg_value: u8,
    pub iface: u8,
    pub ep_mps: u16,
    pub ep_interval: u8,
    pub dci: u8,
}

/// Cherche l'interface clavier boot (classe 3, sous-classe 1, protocole 1) et son point
/// de terminaison d'interruption IN dans le descripteur de configuration.
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
                return Some(Kbd { cfg_value, iface, ep_mps: mps & 0x7FF, ep_interval: cfg[p + 6], dci });
            }
        }
        p += len;
    }
    None
}

/// Configure le peripherique, passe en protocole boot, puis ouvre le point de terminaison clavier.
pub unsafe fn setup_keyboard(bar: u64, port: u8, slot: u8, k: Kbd) -> Result<(), &'static str> {
    // SAFETY: routage racine (vitesse lue dans PORTSC).
    unsafe { setup_keyboard_route(bar, port, slot, k, crate::usb_enum::root_route(bar, port)) }
}

/// Configure le clavier pour un peripherique routé (racine ou derriere un hub).
pub unsafe fn setup_keyboard_route(bar: u64, port: u8, slot: u8, k: Kbd, r: crate::usb_enum::Route) -> Result<(), &'static str> {
    // SAFETY: MMIO xHCI deja initialise ; pages DMA statiques.
    unsafe {
        // SET_CONFIGURATION
        let sc: u64 = (9u64 << 8) | ((k.cfg_value as u64) << 16);
        control(bar, slot, sc, 0)?;
        // SET_PROTOCOL = 0 (boot), sur l'interface clavier
        let sp: u64 = 0x21 | (0x0Bu64 << 8) | ((k.iface as u64) << 32);
        control(bar, slot, sp, 0)?;

        // Configure Endpoint : contexte d'entree avec emplacement + point de terminaison
        let inp = dpg(0);
        let outc = dpg(1);
        zero_page(inp);
        let capl = core::ptr::read_volatile(bar as *const u8) as u64;
        let _ = capl;
        let hcc1 = r32(bar + 0x10);
        let ctx: u64 = if (hcc1 >> 2) & 1 == 1 { 64 } else { 32 };
        let speed = r.speed;
        // Controle : A0 (emplacement) + A(dci)
        w32(inp + 4, 0x1 | (1u32 << k.dci));
        // Emplacement : entrees = dci, vitesse, port racine
        w32(inp + ctx, (r.route & 0xF_FFFF) | ((k.dci as u32) << 27) | (speed << 20));
        w32(inp + ctx + 8, (r.parent_slot as u32) | ((r.parent_port as u32) << 8));
        w32(inp + ctx + 4, (port as u32) << 16);
        // Point de terminaison (au rang dci) : interruption IN (type 7), intervalle, MPS
        let ep = inp + (k.dci as u64 + 1) * ctx;
        let interval = interrupt_interval(speed, k.ep_interval);
        w32(ep, (interval as u32) << 16);
        w32(ep + 4, (7 << 3) | (3 << 1) | ((k.ep_mps as u32) << 16));
        let ring = dpg(4);
        zero_page(ring);
        w64(ep + 8, ring | 1);
        w32(ep + 16, 8 | ((k.ep_mps as u32) << 16));
        let _ = outc;

        // Anneau de commandes (page 1) : commande Configure Endpoint (type 12) a l'index suivant
        let idx = crate::usb_enum::next_cmd_index();

        trb_c(pg(1), idx, inp, 0, (12 << 10) | ((slot as u32) << 24), crate::usb_enum::cmd_cycle());
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
        let ring = dpg(4);
        let idx = INT_IDX;
        let cyc = INT_CYC;
        trb_c(ring, idx, dpg(5) + idx * 8, 8, (1 << 10) | (1 << 5), cyc);
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
        // Rapport de l'endpoint clavier (DCI) du slot du clavier uniquement.
        let ev = wait_event_filt(&[32], Some(slot), Some(dci))?;
        if ev.code != 1 && ev.code != 13 {
            return Err("rapport clavier en erreur");
        }
        // Index du TRB termine : (adresse TRB - base de l'anneau) / 16
        let base = dpg(4);
        let k = ((ev.p0.wrapping_sub(base)) / 16).min(14);
        let mut rep = [0u8; 8];
        for (i, b) in rep.iter_mut().enumerate() {
            *b = core::ptr::read_volatile((dpg(5) + k * 8 + i as u64) as *const u8);
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
