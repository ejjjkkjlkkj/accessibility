// Decodeurs issus des lectures de specifications (ACPI 6.5, UEFI 2.11, UEFI PI 1.10, PCIe).
// Tout ce module est en LECTURE SEULE : il ne fait que decoder des octets deja lus par l'agent.
// Aucun acces materiel, aucune ecriture. Les fonctions sont pures (testables sur l'hote).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

fn o8(t: &[u8], o: usize) -> u8 {
    t.get(o).copied().unwrap_or(0)
}
fn o16(t: &[u8], o: usize) -> u16 {
    if o + 2 <= t.len() { u16::from_le_bytes([t[o], t[o + 1]]) } else { 0 }
}
fn o32(t: &[u8], o: usize) -> u32 {
    if o + 4 <= t.len() { u32::from_le_bytes([t[o], t[o + 1], t[o + 2], t[o + 3]]) } else { 0 }
}
fn o64(t: &[u8], o: usize) -> u64 {
    if o + 8 <= t.len() {
        u64::from_le_bytes([t[o], t[o + 1], t[o + 2], t[o + 3], t[o + 4], t[o + 5], t[o + 6], t[o + 7]])
    } else {
        0
    }
}

/// Adresse de la FACS donnee par la FADT (X_FIRMWARE_CTRL si present et non nul, sinon FIRMWARE_CTRL).
pub fn fadt_adresse_facs(t: &[u8]) -> u64 {
    let x = o64(t, 132);
    if x != 0 { x } else { o32(t, 36) as u64 }
}

/// Decode une table ACPI connue. Rend des lignes de rapport (vide si la table n'est pas decodee ici).
pub fn decrire_acpi(sig: &str, t: &[u8]) -> Vec<String> {
    match sig {
        "FACP" => decrire_fadt(t),
        "APIC" => decrire_madt(t),
        "MCFG" => decrire_mcfg(t),
        "HPET" => decrire_hpet(t),
        _ => Vec::new(),
    }
}

// FADT (ACPI 6.5, table 5.9). Sert a l'OS pour : activer l'ACPI, la veille, l'arret, le reset.
fn decrire_fadt(t: &[u8]) -> Vec<String> {
    let mut v = Vec::new();
    if t.len() < 116 {
        v.push(format!("    FADT trop courte ({} octets) : non decodee", t.len()));
        return v;
    }
    let flags = o32(t, 112);
    let iapc = o16(t, 109);
    v.push(format!(
        "    FADT : SCI_INT {} | SMI_CMD port {:#x} | ACPI_ENABLE {:#04x} | ACPI_DISABLE {:#04x}",
        o16(t, 46), o32(t, 48), o8(t, 52), o8(t, 53)
    ));
    v.push(format!(
        "    FADT : PM1a_EVT {:#x} (long {}) | PM1a_CNT {:#x} (long {}) | PM_TMR {:#x} | GPE0 {:#x} (long {})",
        o32(t, 56), o8(t, 88), o32(t, 64), o8(t, 89), o32(t, 76), o32(t, 80), o8(t, 92)
    ));
    v.push(format!(
        "    FADT : profil {} | drapeaux {:#010x} | ACPI materiel reduit {} | S0 basse conso {} | minuterie PM 32 bits {}",
        o8(t, 45),
        flags,
        if flags & (1 << 20) != 0 { "oui" } else { "non" },
        if flags & (1 << 21) != 0 { "oui" } else { "non" },
        if flags & (1 << 8) != 0 { "oui" } else { "non" }
    ));
    v.push(format!(
        "    FADT : IAPC_BOOT_ARCH {:#06x} | controleur clavier 8042 {} | VGA absent {} | MSI interdit {}",
        iapc,
        if iapc & 2 != 0 { "present" } else { "non annonce" },
        if iapc & 4 != 0 { "oui" } else { "non" },
        if iapc & 8 != 0 { "oui" } else { "non" }
    ));
    if t.len() >= 129 && flags & (1 << 10) != 0 {
        v.push(format!(
            "    FADT : registre de reset : espace {} adresse {:#x} valeur {:#04x} (ecriture = redemarrage, non faite)",
            o8(t, 116), o64(t, 120), o8(t, 128)
        ));
    } else {
        v.push(String::from("    FADT : registre de reset non annonce"));
    }
    v.push(format!(
        "    FADT : DSDT {:#x} | FACS {:#x}",
        if o64(t, 140) != 0 { o64(t, 140) } else { o32(t, 40) as u64 },
        fadt_adresse_facs(t)
    ));
    v.push(String::from(
        "    [regle] ecrire sur le port SMI_CMD ou dans PM1a_CNT lance du code firmware ou une mise en veille : jamais sans accord",
    ));
    v
}

/// FACS (ACPI 6.5, table 5.13) : vecteurs de reveil que l'OS doit remplir avant la veille S3.
pub fn decrire_facs(t: &[u8]) -> Vec<String> {
    let mut v = Vec::new();
    if t.len() < 32 || &t[0..4] != b"FACS" {
        v.push(String::from("    FACS : signature absente, non decodee"));
        return v;
    }
    v.push(format!(
        "    FACS : longueur {} | signature materielle {:#010x} | vecteur de reveil 32 bits {:#x} | vecteur 64 bits {:#x} | drapeaux {:#x} | version {}",
        o32(t, 4), o32(t, 8), o32(t, 12), o64(t, 24), o32(t, 20), o8(t, 32)
    ));
    v.push(String::from(
        "    [regle] au reveil S3 le firmware ne restaure que chipset et processeur ; l'OS remet tout le reste en etat",
    ));
    v
}

// MADT (ACPI 6.5, 5.2.12) : processeurs et controleurs d'interruptions.
fn decrire_madt(t: &[u8]) -> Vec<String> {
    let mut v = Vec::new();
    if t.len() < 44 {
        return v;
    }
    v.push(format!(
        "    MADT : APIC local a {:#x} | drapeaux {:#x} (bit 0 = PIC 8259 present, a masquer)",
        o32(t, 36), o32(t, 40)
    ));
    let mut p = 44usize;
    let (mut ncpu, mut nactif) = (0u32, 0u32);
    let mut n = 0;
    while p + 2 <= t.len() && n < 1024 {
        let ty = t[p];
        let len = t[p + 1] as usize;
        if len < 2 || p + len > t.len() {
            break;
        }
        match ty {
            0 => {
                ncpu += 1;
                if o32(t, p + 4) & 1 != 0 { nactif += 1; }
            }
            9 => {
                ncpu += 1;
                if o32(t, p + 8) & 1 != 0 { nactif += 1; }
            }
            1 => v.push(format!(
                "    MADT : IOAPIC id {} a {:#x}, base GSI {}",
                o8(t, p + 2), o32(t, p + 4), o32(t, p + 8)
            )),
            2 => v.push(format!(
                "    MADT : redirection IRQ ISA {} -> GSI {} (drapeaux {:#x})",
                o8(t, p + 3), o32(t, p + 4), o16(t, p + 8)
            )),
            4 => v.push(format!(
                "    MADT : NMI APIC local : processeur {:#x}, LINT{} (drapeaux {:#x})",
                o8(t, p + 2), o8(t, p + 5), o16(t, p + 3)
            )),
            5 => v.push(format!("    MADT : adresse APIC local 64 bits {:#x}", o64(t, p + 4))),
            _ => {}
        }
        p += len;
        n += 1;
    }
    v.push(format!("    MADT : {} processeurs logiques decrits, {} actifs", ncpu, nactif));
    v.push(String::from(
        "    [regle] ne jamais reveiller un coeur absent de la MADT ou non actif",
    ));
    v
}

// MCFG : fenetre memoire de la configuration PCI Express (ECAM).
fn decrire_mcfg(t: &[u8]) -> Vec<String> {
    let mut v = Vec::new();
    let mut p = 44usize;
    while p + 16 <= t.len() {
        v.push(format!(
            "    MCFG : ECAM base {:#x} | segment {} | bus {}..{}",
            o64(t, p), o16(t, p + 8), o8(t, p + 10), o8(t, p + 11)
        ));
        p += 16;
    }
    if v.is_empty() {
        v.push(String::from("    MCFG : aucune entree"));
    }
    v
}

fn decrire_hpet(t: &[u8]) -> Vec<String> {
    let mut v = Vec::new();
    if t.len() >= 56 {
        v.push(format!(
            "    HPET : identifiant {:#010x} | adresse {:#x} | numero {} | tick minimal {}",
            o32(t, 36), o64(t, 44), o8(t, 52), o16(t, 53)
        ));
    }
    v
}

/// Type de memoire UEFI que l'OS ne doit jamais allouer ni ecraser.
/// Rend le nom de la zone, ou None si la zone redevient libre apres ExitBootServices.
pub fn zone_interdite(ty: u32) -> Option<&'static str> {
    match ty {
        0 => Some("RESERVEE"),
        5 => Some("RUNTIME CODE (firmware)"),
        6 => Some("RUNTIME DONNEES (firmware)"),
        8 => Some("INUTILISABLE (erreurs)"),
        9 => Some("ACPI A RELIRE (tables)"),
        10 => Some("ACPI NVS (code de reveil S3)"),
        11 => Some("MMIO"),
        12 => Some("MMIO PORTS"),
        13 => Some("PAL CODE"),
        14 => Some("PERSISTANTE"),
        15 => Some("NON ACCEPTEE"),
        t if t > 15 => Some("TYPE OEM / INCONNU"),
        _ => None,
    }
}

/// Nom d'un protocole firmware reconnu (GUID releves dans UEFI PI 1.10, parties III a V).
/// `g` est au format de guid_str : minuscules, 8-4-4-4-12.
pub fn nom_guid(g: &str) -> Option<&'static str> {
    Some(match g {
        "86212936-0e76-41c8-a03a-2af2fc1c39e2" => "Codes d'etat : RSC Handler",
        "13a3f0f6-264a-3ef0-f2e0-dec512342f34" => "PCD (reglages plateforme)",
        "fd0f4478-0efd-461d-ba2d-e58c45fd5f5e" => "PCD : informations",
        "f4ccbfb7-f6e0-47fd-9dd4-10a8f150c191" => "SMM : MM Base",
        "c2702b74-800c-4131-8746-8fb5b89ce4ac" => "SMM : MM Access (ouverture/verrou MMRAM)",
        "843dc720-ab1e-42cb-9357-8a0078f3561b" => "SMM : MM Control (declenche une SMI)",
        "26eeb3de-b689-492e-80f0-be8bd7da4ba7" => "SMM : MM Configuration",
        "60ff8964-e906-41d0-afed-f241e974e08e" => "SMM : DXE MM Ready To Lock (MMRAM verrouillee)",
        "c68ed8e2-9dc6-4cbd-9d94-db65acc5c332" => "SMM : MM Communication",
        "378daedc-f06b-4446-8314-40ab933c87a3" => "SMM : MM Communication2",
        "f7234a14-0df2-46c0-ad28-90e6b883a72f" => "SMM : MM Communication3",
        "e49d33ed-513d-4634-b698-6f55aa751c1b" => "SMBus : controleur hote",
        "03583ff6-cb36-4940-947e-b9b39f4afaf7" => "SMBIOS (protocole)",
        "a1e37052-80d9-4e65-a317-3e9a55c43ec9" => "IDE : initialisation du controleur",
        "d432a67f-14dc-484b-b3bb-3f0291849327" => "Disk Info (Identify du disque)",
        "e857caf6-c046-45dc-be3f-ee0765fba887" => "S3 : Save State (script de reveil)",
        "eb97088e-cfdf-49c6-be4b-d906a5b20e86" => "ACPI SDT",
        "cf8034be-6768-4d8b-b739-7cce683a9fbe" => "PCI : allocation des ressources du host bridge",
        "30cfe3e7-3de1-4586-be20-deaba1b3b793" => "PCI : enumeration terminee",
        "07d75280-27d4-4d69-90d0-5643e238b341" => "PCI : plateforme",
        "b5b35764-460c-4a06-99fc-77a17c1b5ceb" => "PCI : override",
        "eb23f55a-7863-4ac2-8d3d-956535de0375" => "PCI : peripheriques incompatibles",
        "aa0e8bc1-dabc-46b0-a844-37b8169b2bea" => "PCI : branchement a chaud (init)",
        "19cb87ab-2cb9-4665-8360-ddcf6054f79d" => "PCI : branchement a chaud (requete)",
        "215fdd18-bd50-4feb-890b-58ca0b4739e9" => "Super I/O",
        "b91978df-9fc1-427d-bb05-4c828455ca27" => "Super I/O : controle",
        "bcdaf080-1bde-4e22-ae6a-43541e128ec4" => "ISA/LPC : controleur hote",
        "fad7933a-6c21-4234-a434-0a8a0d2b0781" => "ISA/LPC : service binding",
        "ad61f191-ae5f-4c0e-b9fa-e869d288c64f" => "CPU I/O 2",
        "70101eaf-0085-440c-b356-8ee36fef24f0" => "Legacy Region 2 (zone 0xC0000-0xFFFFF)",
        "cd72881f-45b5-4feb-98c8-313da8117462" => "I2C : maitre",
        "a5aab9e3-c727-48cd-8bbf-427233854948" => "I2C : hote",
        "b60a3e6b-18c4-46e5-a29a-c9a10665a28e" => "I2C : E/S peripherique",
        "55b71fb5-17c6-410e-b5bd-5fa2e3d4466b" => "I2C : configuration du bus",
        "da8cd7c4-1c00-49e2-803e-5214e701894c" => "I2C : enumeration",
        "85a6d3e6-b65b-4afc-b38f-c6d54af6ddc8" => "SPI : configuration de la carte",
        "b57ec3fe-f833-4ba6-8578-2a7d6a87444b" => "SPI : flash NOR",
        "f01bed57-04bc-4f3f-9660-d6f2ea228259" => "SPI : flash heritee (protection BIOS)",
        "c74e5db2-fa96-4ae2-b399-15977fe3002d" => "SPI : controleur hote",
        "39136fc7-1a11-49de-bf35-0e78ddb524fc" => "SPI : controleur herite (verrou)",
        "eeff0ef1-6a4c-4b4b-9f84-59045c634bc3" => "SPI : carte des blocs de la flash",
        _ => return None,
    })
}

/// Capacite etendue AER (identifiant 0x0001) a l'offset `p` de la configuration etendue.
/// Lecture seule des registres d'etat deja copies dans `ext`.
pub fn decrire_aer(ext: &[u8], p: usize) -> String {
    let unc = o32(ext, p + 0x04);
    let cor = o32(ext, p + 0x10);
    format!(
        "      AER : erreurs non corrigibles {:#010x} | corrigibles {:#010x}{}",
        unc,
        cor,
        if unc != 0 || cor != 0 { " <- ERREURS DE LIEN ENREGISTREES" } else { " (aucune)" }
    )
}

/// Lignes d'information pour un peripherique PCI reconnu par les lectures (None sinon).
pub fn note_peripherique(vid: u16, did: u16, classe: u8, sous: u8, prog: u8) -> Option<String> {
    if vid == 0x14C3 {
        return Some(format!(
            "    >> Wi-Fi MediaTek {:04x}:{:04x} (famille MT7921, nom interne MT7961) : fonction PCIe ; ne marche qu'apres envoi du correctif WIFI_MT7961_patch_mcu_1_2_hdr.bin puis de WIFI_RAM_CODE_MT7961_1.bin ; Bluetooth a part, en USB ; aucun pilote dans cet agent",
            vid, did
        ));
    }
    match (classe, sous, prog) {
        (0x01, 0x08, 0x02) => Some(String::from("    >> stockage NVMe (pilote present dans l'agent, lecture seule)")),
        (0x0C, 0x03, 0x30) => Some(String::from("    >> controleur USB xHCI (pilote present dans l'agent)")),
        (0x04, 0x03, _) => Some(String::from("    >> audio HDA (pilote present dans l'agent)")),
        (0x02, 0x80, _) | (0x0D, _, _) => Some(String::from("    >> reseau sans fil : aucun pilote dans cet agent")),
        (0x0C, 0x05, _) => Some(String::from("    >> controleur SMBus : lecture SPD possible, aucune ecriture")),
        (0x06, 0x01, _) => Some(String::from("    >> pont ISA/LPC : controleur embarque et clavier interne derriere")),
        _ => None,
    }
}
