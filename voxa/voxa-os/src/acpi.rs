//! Lecture des tables ACPI (RSDP -> XSDT/RSDT -> MCFG, FADT). Memoire physique identique a la virtuelle (cartographie du firmware). Lecture seule.
use crate::cpu::{rd16, rd32, rd64, rd8};

pub struct Acpi { pub tables: [([u8; 4], u64); 48], pub n: usize, pub ecam: Option<Ecam>, pub pm1a_cnt: u32, pub dsdt: u64, pub revision: u8 }
#[derive(Clone, Copy)] pub struct Ecam { pub base: u64, pub bus_debut: u8, pub bus_fin: u8 }

unsafe fn sig(a: u64) -> [u8; 4] { [rd8(a), rd8(a + 1), rd8(a + 2), rd8(a + 3)] }
unsafe fn somme(a: u64, len: u64) -> bool { let mut s = 0u8; for k in 0..len { s = s.wrapping_add(rd8(a + k)); } s == 0 }

/// Analyse depuis l'adresse du RSDP. None si la signature ou la somme de controle est fausse.
pub fn lire(rsdp: u64) -> Option<Acpi> {
    unsafe {
        if rd64(rsdp) != u64::from_le_bytes(*b"RSD PTR ") || !somme(rsdp, 20) { return None; }
        let revision = rd8(rsdp + 15); let (racine, large) = if revision >= 2 && rd64(rsdp + 24) != 0 { (rd64(rsdp + 24), true) } else { (rd32(rsdp + 16) as u64, false) };
        if racine == 0 { return None; }
        let len = rd32(racine + 4) as u64; if len < 36 || len > 0x10_0000 || !somme(racine, len) { return None; }
        let mut a = Acpi { tables: [([0; 4], 0); 48], n: 0, ecam: None, pm1a_cnt: 0, dsdt: 0, revision };
        let pas = if large { 8 } else { 4 }; let nb = ((len - 36) / pas) as usize;
        for i in 0..nb.min(48) { let t = if large { rd64(racine + 36 + i as u64 * 8) } else { rd32(racine + 36 + i as u64 * 4) as u64 }; if t == 0 { continue; }
            let s = sig(t); a.tables[a.n] = (s, t); a.n += 1;
            if &s == b"MCFG" && rd32(t + 4) >= 60 { a.ecam = Some(Ecam { base: rd64(t + 44), bus_debut: rd8(t + 54), bus_fin: rd8(t + 55) }); }
            if &s == b"FACP" { a.pm1a_cnt = rd32(t + 64); let x = if rd32(t + 4) >= 148 { rd64(t + 140) } else { 0 }; a.dsdt = if x != 0 { x } else { rd32(t + 40) as u64 }; }
        }
        Some(a)
    }
}

/// Type de sommeil S5 (extinction) lu dans l'objet `_S5_` du DSDT : (SLP_TYPa). Recherche de l'octet-code `_S5_` suivi d'un paquet ; None si absent.
pub fn type_s5(dsdt: u64) -> Option<u16> {
    unsafe {
        if dsdt == 0 || rd32(dsdt) != u32::from_le_bytes(*b"DSDT") { return None; }
        let len = rd32(dsdt + 4) as u64; if len < 44 || len > 0x40_0000 { return None; }
        let mut i = 36u64;
        while i + 10 < len {
            if rd8(dsdt + i) == b'_' && rd8(dsdt + i + 1) == b'S' && rd8(dsdt + i + 2) == b'5' && rd8(dsdt + i + 3) == b'_' && rd8(dsdt + i + 4) == 0x12 {
                let pl = rd8(dsdt + i + 5); let extra = (pl >> 6) as u64; let mut p = i + 6 + extra; // apres PkgLength
                p += 1; // NumElements
                let e = rd8(dsdt + p); let v = match e { 0x0A => rd8(dsdt + p + 1) as u16, 0x00 => 0, 0x01 => 1, 0x0B => rd16(dsdt + p + 1), _ => return None };
                return Some(v & 7);
            }
            i += 1;
        }
        None
    }
}
