//! Enumeration PCI : ECAM (config en memoire, table MCFG) de preference, ports 0xCF8/0xCFC en repli. Lecture seule.
use crate::acpi::Ecam;
use crate::cpu::{rd32, RealPorts};
use voxa_kernel::hw::{pci_enumerate, PciDevice};
use voxa_kernel::PciId;

fn ecam_addr(e: &Ecam, bus: u8, dev: u8, func: u8) -> u64 { e.base + (((bus - e.bus_debut) as u64) << 20 | (dev as u64) << 15 | (func as u64) << 12) }

/// Remplit `out` avec les fonctions trouvees. Retourne (nombre, methode).
pub fn enumerer(ecam: Option<Ecam>, out: &mut [PciDevice]) -> (usize, &'static str) {
    if let Some(e) = ecam {
        let mut n = 0;
        for bus in e.bus_debut..=e.bus_fin { for dev in 0..32u8 {
            let a0 = ecam_addr(&e, bus, dev, 0); if unsafe { rd32(a0) } & 0xFFFF == 0xFFFF { continue; }
            let multi = unsafe { rd32(a0 + 0x0C) } >> 16 & 0x80 != 0;
            for func in 0..(if multi { 8u8 } else { 1 }) {
                let a = ecam_addr(&e, bus, dev, func); let v = unsafe { rd32(a) }; if v & 0xFFFF == 0xFFFF { continue; }
                let c = unsafe { rd32(a + 8) }; let mut cfg = [0u8; 16]; cfg[..4].copy_from_slice(&v.to_le_bytes()); cfg[8..12].copy_from_slice(&c.to_le_bytes());
                if let Some(id) = PciId::from_config(&cfg) { if n < out.len() { out[n] = PciDevice { bus, dev, func, id }; n += 1; } }
            } } }
        return (n, "ECAM (memoire)");
    }
    (pci_enumerate(&mut RealPorts, 15, out), "ports 0xCF8/0xCFC")
}
