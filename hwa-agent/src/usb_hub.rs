// Hubs USB en cascade : alimentation des ports, reinitialisation, puis recherche du clavier
// derriere le hub, recursivement (jusqu'a 5 niveaux, limite de la chaine de routage xHCI).
// Reference : USB 2.0 chapitre 11 (GET_DESCRIPTOR hub 0x29 via 0xA0 ; SET_FEATURE / CLEAR_FEATURE
// 0x23 ; GET_STATUS port 0xA3 ; wPortStatus : bit 0 connexion, bit 1 activation, bit 4 reset,
// bit 9 basse vitesse, bit 10 haute vitesse ; features PORT_RESET 4, PORT_POWER 8,
// C_PORT_CONNECTION 16, C_PORT_RESET 20), spec xHCI 1.2c : Configure Endpoint (4.6.6),
// contexte d'emplacement (6.2.2 : route 19:0, hub bit 26, nombre de ports 31:24, TT 7:0 / 15:8),
// chaine de routage (8.9 : 4 bits par niveau, niveau 1 en bits 3:0).
use crate::usb_enum::{self, Route};
use crate::usb_hid;
use crate::usb_r0::{self, dpg, wait_event, XHCI_MEM};

const SET_FEATURE: u8 = 3;
const CLEAR_FEATURE: u8 = 1;
const PORT_RESET: u16 = 4;
const PORT_POWER: u16 = 8;
const C_PORT_CONNECTION: u16 = 16;
const C_PORT_RESET: u16 = 20;
const MAX_DEPTH: u32 = 5;

fn req(bm: u8, br: u8, val: u16, idx: u16, len: u16) -> u64 {
    (bm as u64) | ((br as u64) << 8) | ((val as u64) << 16) | ((idx as u64) << 32) | ((len as u64) << 48)
}

/// Attente approximative (boucle d'attente, sans horloge).
fn busy_ms(ms: u32) {
    for _ in 0..(ms as u64 * 1_000_000) {
        core::hint::spin_loop();
    }
}

unsafe fn byte(a: u64) -> u8 {
    // SAFETY: zone DMA statique, lecture d'un octet.
    unsafe { core::ptr::read_volatile(a as *const u8) }
}

/// Nombre de niveaux de la chaine de routage (nibbles non nuls).
fn depth_of(route: u32) -> u32 {
    let mut d = 0;
    let mut r = route;
    while r & 0xF != 0 && d < 8 {
        r >>= 4;
        d += 1;
    }
    d
}

/// Etat (wPortStatus) d'un port du hub, lu dans la zone de donnees (2 octets utiles).
unsafe fn port_status(bar: u64, hub_slot: u8, p: u8) -> Result<u16, &'static str> {
    // SAFETY: transfert de controle EP0 du hub.
    unsafe {
        usb_hid::control(bar, hub_slot, req(0xA3, 0, 0, p as u16, 4), 4)?;
        Ok(byte(dpg(3)) as u16 | ((byte(dpg(3) + 1) as u16) << 8))
    }
}

/// Configure le contexte d'emplacement du hub (route, bit Hub, nombre de ports, TT) par Configure Endpoint.
unsafe fn configure_hub_slot(bar: u64, root_port: u8, hub_slot: u8, r: Route, nports: u8) -> Result<(), &'static str> {
    // SAFETY: anneau de commandes et contexte d'entree de la zone courante.
    unsafe {
        let hcc1 = core::ptr::read_volatile((bar + 0x10) as *const u32);
        let ctx: u64 = if (hcc1 >> 2) & 1 == 1 { 64 } else { 32 };
        let inp = dpg(0);
        for i in 0..512u64 {
            core::ptr::write_volatile((inp + i * 8) as *mut u64, 0);
        }
        core::ptr::write_volatile((inp + 4) as *mut u32, 0x1);
        core::ptr::write_volatile((inp + ctx) as *mut u32, (r.route & 0xF_FFFF) | (1 << 26) | (1 << 27) | (r.speed << 20));
        core::ptr::write_volatile((inp + ctx + 4) as *mut u32, ((nports as u32) << 24) | ((root_port as u32) << 16));
        core::ptr::write_volatile((inp + ctx + 8) as *mut u32, (r.parent_slot as u32) | ((r.parent_port as u32) << 8));

        let idx = usb_enum::next_cmd_index();
        let ring = core::ptr::addr_of_mut!(XHCI_MEM) as u64 + 4096;
        let trb = ring + idx * 16;
        core::ptr::write_volatile(trb as *mut u64, inp);
        core::ptr::write_volatile((trb + 8) as *mut u32, 0);
        core::ptr::write_volatile((trb + 12) as *mut u32, (12 << 10) | ((hub_slot as u32) << 24) | usb_enum::cmd_cycle());
        let dboff = (core::ptr::read_volatile((bar + 0x14) as *const u32) & !0x3) as u64;
        core::ptr::write_volatile((bar + dboff) as *mut u32, 0);
        let ev = wait_event(&[33])?;
        if ev.code == 1 {
            Ok(())
        } else {
            Err("hub : Configure Endpoint refuse")
        }
    }
}

/// Enumere le hub `hub_slot` (zone DMA du hub deja active). Retourne (slot, DCI) du clavier trouve.
/// Recursif pour les hubs en cascade ; profondeur limitee a MAX_DEPTH.
pub unsafe fn enumerate(bar: u64, root_port: u8, hub_slot: u8, hub_route: Route) -> Result<(u8, u8), &'static str> {
    // SAFETY: transferts de controle EP0 du hub, zones DMA par peripherique.
    unsafe {
        if depth_of(hub_route.route) >= MAX_DEPTH {
            return Err("hub : trop de niveaux");
        }
        // Zone DMA du hub, lue une fois a l'entree de la fonction.
        let hub_zone = usb_r0::DEV_BASE;
        // 1. Descripteur du hub (0x29) : nombre de ports en octet 2
        usb_hid::control(bar, hub_slot, req(0xA0, 6, 0x2900, 0, 9), 9)?;
        let nports = byte(dpg(3) + 2).min(8);
        if nports == 0 {
            return Err("hub sans port");
        }
        // 2. Le slot devient un hub
        configure_hub_slot(bar, root_port, hub_slot, hub_route, nports)?;
        // 3. Alimentation de chaque port
        for p in 1..=nports {
            usb_hid::control(bar, hub_slot, req(0x23, SET_FEATURE, PORT_POWER, p as u16, 0), 0)?;
        }
        busy_ms(100);
        // 4. Ports avec un peripherique
        let mut present = [0u8; 8];
        let mut np = 0usize;
        for p in 1..=nports {
            if port_status(bar, hub_slot, p)? & 1 != 0 {
                present[np] = p;
                np += 1;
            }
        }
        if np == 0 {
            return Err("hub : aucun peripherique");
        }
        // 5. Reinitialisation de chaque port, puis identification, hub imbrique ou clavier
        let mut last_err: &'static str = "hub : pas de clavier";
        let child_level = depth_of(hub_route.route) + 1;
        // La zone DMA courante peut avoir ete changee par un enfant precedent (zone_alloc) :
        // on revient a la zone du hub avant chaque transfert de controle de ce hub.
        let hub_base = hub_zone;
        for i in 0..np {
            usb_r0::set_device_base(hub_base);
            let p = present[i];
            usb_hid::control(bar, hub_slot, req(0x23, SET_FEATURE, PORT_RESET, p as u16, 0), 0)?;
            busy_ms(30);
            let st = port_status(bar, hub_slot, p)?;
            let _ = usb_hid::control(bar, hub_slot, req(0x23, CLEAR_FEATURE, C_PORT_RESET, p as u16, 0), 0);
            let _ = usb_hid::control(bar, hub_slot, req(0x23, CLEAR_FEATURE, C_PORT_CONNECTION, p as u16, 0), 0);
            if st & 2 == 0 {
                last_err = "hub : port non active";
                continue;
            }
            let speed = if st & 0x400 != 0 {
                3
            } else if st & 0x200 != 0 {
                2
            } else {
                1
            };
            // Chaine de routage : le port p au niveau suivant
            let route = hub_route.route | ((p as u32) << (4 * (child_level - 1)));
            // TT : peripherique bas/pleine vitesse. Son hub parent haute vitesse est le hub courant
            // si celui-ci est haute vitesse, sinon le TT du hub courant est reutilise.
            let (parent_slot, parent_port) = if speed == 3 {
                (0, 0)
            } else if hub_route.speed == 3 {
                (hub_slot, p)
            } else {
                (hub_route.parent_slot, hub_route.parent_port)
            };
            let child = Route { route, speed, parent_slot, parent_port };

            if usb_r0::zone_alloc().is_err() {
                last_err = "hub : plus de zone DMA";
                break;
            }
            let slot = match usb_r0::enable_slot_cmd(bar) {
                Ok(s) => s,
                Err(e) => {
                    last_err = e;
                    continue;
                }
            };
            if let Err(e) = usb_enum::address_device_route(bar, root_port, slot, child) {
                last_err = e;
                continue;
            }
            // Classe du peripherique : hub (9) -> recursion
            let desc = match usb_enum::get_device_descriptor(bar, slot) {
                Ok(d) => d,
                Err(e) => {
                    last_err = e;
                    continue;
                }
            };
            if desc[4] == 9 {
                match enumerate(bar, root_port, slot, child) {
                    Ok(found) => return Ok(found),
                    Err(e) => {
                        last_err = e;
                        continue;
                    }
                }
            }
            let cfg = match usb_hid::get_config(bar, slot) {
                Ok(c) => c,
                Err(e) => {
                    last_err = e;
                    continue;
                }
            };
            let Some(k) = usb_hid::find_keyboard(&cfg) else {
                last_err = "hub : port sans clavier";
                continue;
            };
            match usb_hid::setup_keyboard_route(bar, root_port, slot, k, child) {
                Ok(()) => {
                    usb_hid::start(bar, slot, k.dci);
                    return Ok((slot, k.dci));
                }
                Err(e) => last_err = e,
            }
        }
        Err(last_err)
    }
}
