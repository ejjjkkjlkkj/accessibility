// Adressage du peripherique USB (Address Device) et lecture du descripteur de peripherique.
// Reference : spec xHCI 1.2c, section 4.6.5 (Address Device) et 4.11.2.x (controle EP0) ;
// USB 2.0, chapitre 9 (SETUP, GET_DESCRIPTOR). Pages DMA : voir usb_r0::XHCI_MEM.
//   page 0 : DCBAA   page 1 : anneau de commandes   page 2 : anneau d'evenements
//   page 4 : contexte d'entree   page 5 : contexte de sortie   page 6 : anneau EP0   page 7 : donnees
use crate::usb_r0::{dpg, wait_event, wait_event_filt, Line, XHCI_MEM};

// Prochain index libre de l'anneau de commandes (index 0 = Enable Slot).
static mut CMD_NEXT: u64 = 0;
// Bit de cycle (PCS) de l'anneau de commandes : il bascule a chaque tour complet (Link TRB).
static mut CMD_PCS: u32 = 1;
// Prochain index libre de l'anneau EP0.
static mut EP0_NEXT: u64 = 0;

unsafe fn pg(i: u64) -> u64 {
    // SAFETY: adresse d'une page du tableau DMA statique.
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
    // SAFETY: registre MMIO xHCI, lecture volatile.
    unsafe { core::ptr::read_volatile(a as *const u32) }
}
unsafe fn zero_page(a: u64) {
    for i in 0..512u64 {
        // SAFETY: page DMA de 4096 octets.
        unsafe { w64(a + i * 8, 0) };
    }
}

/// Ecrit un TRB (16 octets) avec le bit de cycle a 1.
unsafe fn write_trb(ring: u64, idx: u64, param: u64, status: u32, control: u32) {
    // SAFETY: anneau DMA statique, idx < 256.
    unsafe {
        w64(ring + idx * 16, param);
        w32(ring + idx * 16 + 8, status);
        w32(ring + idx * 16 + 12, control | 1);
    }
}

/// Compteur EP0 de chaque zone DMA (indexe par le numero de page de base de la zone).
/// Chaque zone a son propre anneau EP0 : son compteur doit survivre aux changements de zone.
static mut EP0_TAB: [u64; 64] = [0; 64];

/// Charge le compteur EP0 de la zone courante (appele apres chaque changement de zone).
pub unsafe fn reset_ep0() {
    // SAFETY: lecture des compteurs statiques ; DEV_BASE < 64 par construction.
    unsafe {
        let base = crate::usb_r0::DEV_BASE as usize;
        EP0_NEXT = EP0_TAB[base];
    }
}

/// Reserve `n` TRB consecutifs sur l'anneau EP0 de la zone courante.
pub unsafe fn ep0_take(n: u64) -> Result<u64, &'static str> {
    // SAFETY: compteurs statiques, pas de reference.
    unsafe {
        if EP0_NEXT + n > 256 {
            return Err("anneau EP0 plein");
        }
        let i = EP0_NEXT;
        EP0_NEXT += n;
        let base = crate::usb_r0::DEV_BASE as usize;
        EP0_TAB[base] = EP0_NEXT;
        Ok(i)
    }
}

/// Reserve l'index suivant de l'anneau de commandes (page 1).
pub unsafe fn next_cmd_index() -> u64 {
    // SAFETY: compteurs statiques ; le lien TRB est ecrit en page 1 (anneau de commandes).
    unsafe {
        // xHCI 4.9.2.3 : l'anneau se referme par un Link TRB (index 255) avec Toggle Cycle = 1.
        // Le producteur bascule alors son bit PCS (CMD_PCS) et repart du debut de l'anneau.
        if CMD_NEXT == 255 {
            let link = pg(1) + 255 * 16;
            w64(link, pg(1));
            w32(link + 8, 0);
            w32(link + 12, (6 << 10) | (1 << 1) | CMD_PCS);
            CMD_PCS ^= 1;
            CMD_NEXT = 0;
        }
        let i = CMD_NEXT;
        CMD_NEXT += 1;
        i
    }
}

/// Bit de cycle (PCS) a ecrire dans chaque TRB de commande, pour la position courante de l'anneau.
pub unsafe fn cmd_cycle() -> u32 {
    // SAFETY: lecture d'un compteur statique.
    unsafe { CMD_PCS }
}

/// Routage d'un peripherique : chaine de routage du hub, vitesse, et hub parent (TT) si besoin.
#[derive(Clone, Copy)]
pub struct Route {
    pub route: u32,
    pub speed: u32,
    pub parent_slot: u8,
    pub parent_port: u8,
}

/// Classe du dernier peripherique identifie (9 = hub). Mise a jour par identify().
pub static mut LAST_CLASS: u8 = 0;

/// Routage d'un peripherique branche directement sur un port racine (vitesse lue dans PORTSC).
pub unsafe fn root_route(bar: u64, port: u8) -> Route {
    // SAFETY: lecture seule de PORTSC.
    unsafe {
        let capl = core::ptr::read_volatile(bar as *const u8) as u64;
        let portsc = r32(bar + capl + 0x400 + 16 * (port as u64 - 1));
        Route { route: 0, speed: (portsc >> 10) & 0xF, parent_slot: 0, parent_port: 0 }
    }
}

/// Adresse un peripherique racine (compatibilite).
pub unsafe fn address_device(bar: u64, port: u8, slot: u8) -> Result<(), &'static str> {
    // SAFETY: appel avec routage racine.
    unsafe { address_device_route(bar, port, slot, root_route(bar, port)) }
}

/// Adresse d'un peripherique sur `slot` (Address Device, BSR = 0).
pub unsafe fn address_device_route(bar: u64, port: u8, slot: u8, r: Route) -> Result<(), &'static str> {
    // SAFETY: MMIO xHCI deja initialise ; pages DMA statiques.
    unsafe {
        let capl = core::ptr::read_volatile(bar as *const u8) as u64;
        let op = bar + capl;
        let _ = (op, port);
        let speed = r.speed;
        // Taille maximale du paquet EP0 selon la vitesse (USB 2.0 9.6.6)
        let mps: u32 = match speed {
            3 => 64,  // haute vitesse
            4 => 512, // super vitesse
            _ => 8,   // pleine ou basse vitesse
        };
        let hcc1 = r32(bar + 0x10);
        let ctx: u64 = if (hcc1 >> 2) & 1 == 1 { 64 } else { 32 };

        let inp = dpg(0);
        let outc = dpg(1);
        let ep0 = dpg(2);
        zero_page(inp);
        zero_page(outc);
        zero_page(ep0);
        zero_page(dpg(3));

        // Contexte d'entree : controle (A0 = contexte emplacement, A1 = EP0)
        w32(inp + 4, 0x3);
        // Contexte emplacement : Route String, Speed, Context Entries = 1 ; DWORD1 : port racine
        // DWORD2 : TT (hub parent et port) pour un peripherique bas/pleine vitesse derriere un hub haute vitesse
        w32(inp + ctx, (r.route & 0xF_FFFF) | (speed << 20) | (1 << 27));
        w32(inp + ctx + 4, (port as u32) << 16);
        w32(inp + ctx + 8, (r.parent_slot as u32) | ((r.parent_port as u32) << 8));
        // Contexte EP0 (DCI 1) : type controle (4), CErr = 3, taille max paquet, dequeue + DCS
        let e = inp + 2 * ctx;
        w32(e + 4, (4 << 3) | (3 << 1) | (mps << 16));
        w64(e + 8, ep0 | 1);
        w32(e + 16, 8); // taille moyenne TRB

        // DCBAA[slot] = contexte de sortie
        w64(pg(0) + slot as u64 * 8, outc);

        // Commande Address Device sur l'anneau de commandes (page 1)
        let idx = next_cmd_index();
        // TRB Address Device avec le bit de cycle courant de l'anneau de commandes.
        w64(pg(1) + idx * 16, inp);
        w32(pg(1) + idx * 16 + 8, 0);
        w32(pg(1) + idx * 16 + 12, (11 << 10) | ((slot as u32) << 24) | cmd_cycle());
        // Doorbell 0 (commandes)
        let dboff = (r32(bar + 0x14) & !0x3) as u64;
        w32(bar + dboff, 0);

        let ev = wait_event(&[33])?;
        if ev.code == 1 {
            Ok(())
        } else {
            Err("Address Device refuse")
        }
    }
}

/// Lit le descripteur de peripherique (18 octets) par controle EP0 : SETUP, DATA IN, STATUS.
pub unsafe fn get_device_descriptor(bar: u64, slot: u8) -> Result<[u8; 18], &'static str> {
    // SAFETY: MMIO xHCI ; anneau EP0 et page de donnees statiques.
    unsafe {
        let i = ep0_take(3)?;
        let ep0 = dpg(2);
        // Paquet SETUP : bmRequestType 0x80 (IN, standard, peripherique), GET_DESCRIPTOR (6),
        // wValue 0x0100 (descripteur de peripherique), wIndex 0, wLength 18
        let setup: u64 = 0x80 | (6u64 << 8) | (0x0100u64 << 16) | (18u64 << 48);
        write_trb(ep0, i, setup, 8, (2 << 10) | (1 << 6) | (3 << 16)); // Setup Stage, IDT, TRT = IN
        write_trb(ep0, i + 1, dpg(3), 18, (3 << 10) | (1 << 16)); // Data Stage, DIR = IN
        write_trb(ep0, i + 2, 0, 0, (4 << 10) | (1 << 5)); // Status Stage, IOC
        // Doorbell du peripherique, cible = EP0 (DCI 1)
        let dboff = (r32(bar + 0x14) & !0x3) as u64;
        w32(bar + dboff + slot as u64 * 4, 1);

        // Evenement de transfert du slot et de l'endpoint EP0 (DCI 1) uniquement.
        let ev = wait_event_filt(&[32], Some(slot), Some(1))?;
        if ev.code != 1 && ev.code != 13 {
            return Err("GET_DESCRIPTOR en erreur");
        }
        let mut out = [0u8; 18];
        for (k, b) in out.iter_mut().enumerate() {
            // SAFETY: page de donnees statique, 18 octets lus.
            *b = core::ptr::read_volatile((dpg(3) + k as u64) as *const u8);
        }
        Ok(out)
    }
}

/// Adresse le peripherique, lit son descripteur et retourne une ligne d'affichage.
pub unsafe fn identify(bar: u64, port: u8, slot: u8) -> Line {
    let mut l = Line::new();
    // SAFETY: appels MMIO xHCI deja initialises.
    unsafe {
        match address_device(bar, port, slot) {
            Err(e) => {
                l.s(b"ADRESSE ").s(e.as_bytes());
            }
            Ok(()) => match get_device_descriptor(bar, slot) {
                Ok(d) => {
                    LAST_CLASS = d[4];
                    let vid = u16::from_le_bytes([d[8], d[9]]) as u32;
                    let pid = u16::from_le_bytes([d[10], d[11]]) as u32;
                    l.s(b"DEV ").h(vid, 4).s(b":").h(pid, 4).s(b" CL ").h(d[4] as u32, 2);
                }
                Err(e) => {
                    l.s(b"DESC ").s(e.as_bytes());
                }
            },
        }
    }
    l
}
