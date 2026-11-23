// Test sur l'hote des decodeurs de src/lectures.rs (fonctions pures, aucun materiel).
// Compilation : rustc --edition 2024 --test host_test/lectures_test.rs -o target/lectures_test.exe
extern crate alloc;

#[path = "../src/lectures.rs"]
mod lectures;

fn put32(t: &mut [u8], o: usize, v: u32) {
    t[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
fn put64(t: &mut [u8], o: usize, v: u64) {
    t[o..o + 8].copy_from_slice(&v.to_le_bytes());
}

#[test]
fn fadt_champs() {
    let mut t = vec![0u8; 276];
    t[0..4].copy_from_slice(b"FACP");
    put32(&mut t, 36, 0x7A00_0000); // FIRMWARE_CTRL
    t[46] = 9; // SCI_INT
    put32(&mut t, 48, 0xB2); // SMI_CMD
    t[52] = 0xA0;
    t[53] = 0xA1;
    put32(&mut t, 56, 0x800); // PM1a_EVT
    put32(&mut t, 64, 0x804); // PM1a_CNT
    put32(&mut t, 76, 0x808); // PM_TMR
    t[109] = 0x02; // 8042 present
    put32(&mut t, 112, (1 << 10) | (1 << 8)); // RESET_REG_SUP + TMR 32 bits
    t[116] = 1; // espace E/S
    put64(&mut t, 120, 0xCF9);
    t[128] = 0x06;
    let l = lectures::decrire_acpi("FACP", &t).join("\n");
    assert!(l.contains("SCI_INT 9"), "{l}");
    assert!(l.contains("SMI_CMD port 0xb2"), "{l}");
    assert!(l.contains("PM1a_CNT 0x804"), "{l}");
    assert!(l.contains("8042 present"), "{l}");
    assert!(l.contains("adresse 0xcf9 valeur 0x06"), "{l}");
    assert_eq!(lectures::fadt_adresse_facs(&t), 0x7A00_0000);
    put64(&mut t, 132, 0x1_7A00_0000); // X_FIRMWARE_CTRL prioritaire
    assert_eq!(lectures::fadt_adresse_facs(&t), 0x1_7A00_0000);
}

#[test]
fn fadt_tronquee_ne_panique_pas() {
    for n in [0usize, 10, 44, 115, 116, 130] {
        let t = vec![0u8; n];
        let _ = lectures::decrire_acpi("FACP", &t);
        let _ = lectures::decrire_acpi("APIC", &t);
        let _ = lectures::decrire_acpi("MCFG", &t);
        let _ = lectures::decrire_acpi("HPET", &t);
        let _ = lectures::decrire_facs(&t);
        let _ = lectures::decrire_aer(&t, 0x100);
    }
}

#[test]
fn madt_compte_les_coeurs() {
    let mut t = vec![0u8; 44];
    put32(&mut t, 36, 0xFEE0_0000);
    put32(&mut t, 40, 1);
    // deux APIC locaux actifs, un inactif
    for (id, fl) in [(0u8, 1u32), (1, 1), (2, 0)] {
        let mut e = vec![0u8, 8, id, id, 0, 0, 0, 0];
        e[4..8].copy_from_slice(&fl.to_le_bytes());
        t.extend_from_slice(&e);
    }
    // IOAPIC id 4 a 0xFEC00000, GSI 0
    let mut io = vec![1u8, 12, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    io[4..8].copy_from_slice(&0xFEC0_0000u32.to_le_bytes());
    t.extend_from_slice(&io);
    // redirection IRQ 0 -> GSI 2
    let mut iso = vec![2u8, 10, 0, 0, 0, 0, 0, 0, 0, 0];
    iso[4..8].copy_from_slice(&2u32.to_le_bytes());
    t.extend_from_slice(&iso);
    // entree tronquee (longueur depasse la table) : doit arreter proprement
    t.extend_from_slice(&[0u8, 8, 9]);
    let l = lectures::decrire_acpi("APIC", &t).join("\n");
    assert!(l.contains("APIC local a 0xfee00000"), "{l}");
    assert!(l.contains("IOAPIC id 4 a 0xfec00000, base GSI 0"), "{l}");
    assert!(l.contains("IRQ ISA 0 -> GSI 2"), "{l}");
    assert!(l.contains("3 processeurs logiques decrits, 2 actifs"), "{l}");
}

#[test]
fn mcfg_et_hpet() {
    let mut t = vec![0u8; 60];
    put64(&mut t, 44, 0xF000_0000);
    t[54] = 0;
    t[55] = 0x7F;
    let l = lectures::decrire_acpi("MCFG", &t).join("\n");
    assert!(l.contains("ECAM base 0xf0000000 | segment 0 | bus 0..127"), "{l}");
    let mut h = vec![0u8; 56];
    put64(&mut h, 44, 0xFED0_0000);
    let l = lectures::decrire_acpi("HPET", &h).join("\n");
    assert!(l.contains("adresse 0xfed00000"), "{l}");
}

#[test]
fn facs_vecteurs() {
    let mut t = vec![0u8; 64];
    t[0..4].copy_from_slice(b"FACS");
    put32(&mut t, 4, 64);
    put32(&mut t, 12, 0x1000);
    let l = lectures::decrire_facs(&t).join("\n");
    assert!(l.contains("vecteur de reveil 32 bits 0x1000"), "{l}");
    assert!(lectures::decrire_facs(b"XXXX0000000000000000000000000000000000")[0].contains("absente"));
}

#[test]
fn zones_memoire() {
    assert!(lectures::zone_interdite(7).is_none()); // conventionnelle
    assert!(lectures::zone_interdite(1).is_none()); // loader code
    assert!(lectures::zone_interdite(3).is_none()); // boot services code
    assert!(lectures::zone_interdite(4).is_none());
    assert_eq!(lectures::zone_interdite(10), Some("ACPI NVS (code de reveil S3)"));
    assert!(lectures::zone_interdite(0).is_some());
    assert!(lectures::zone_interdite(5).is_some());
    assert!(lectures::zone_interdite(6).is_some());
    assert!(lectures::zone_interdite(11).is_some());
    assert!(lectures::zone_interdite(0x7000_0001).is_some());
}

#[test]
fn guids_et_peripheriques() {
    assert_eq!(lectures::nom_guid("e857caf6-c046-45dc-be3f-ee0765fba887"), Some("S3 : Save State (script de reveil)"));
    assert!(lectures::nom_guid("00000000-0000-0000-0000-000000000000").is_none());
    let n = lectures::note_peripherique(0x14C3, 0x7961, 0x02, 0x80, 0).unwrap();
    assert!(n.contains("MediaTek 14c3:7961") && n.contains("WIFI_RAM_CODE_MT7961_1.bin"), "{n}");
    assert!(lectures::note_peripherique(0x1022, 0x15E3, 0x04, 0x03, 0).unwrap().contains("HDA"));
    assert!(lectures::note_peripherique(0x1022, 0x1630, 0x06, 0x00, 0).is_none());
}

#[test]
fn aer() {
    let mut e = vec![0u8; 4096];
    assert!(lectures::decrire_aer(&e, 0x150).contains("(aucune)"));
    put32(&mut e, 0x150 + 0x10, 0x1);
    assert!(lectures::decrire_aer(&e, 0x150).contains("ERREURS DE LIEN"));
}
