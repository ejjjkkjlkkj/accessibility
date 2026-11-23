// Sonde CPU intégrée à l'agent (no_std). CPUID seul : aucun MSR, aucune écriture, aucun accès
// matériel. XGETBV(0) n'est lu que si CPUID 1 ECX[27] (OSXSAVE) est à 1.
// Bits de fonctionnalités : AMD APM vol. 3 rév. 3.39 (annexe D table D-1, annexe E), comme cpuid31.js.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::arch::x86_64::{__cpuid, __cpuid_count, CpuidResult};

#[derive(Clone, Copy)]
enum Reg { Eax, Ebx, Ecx, Edx }

struct Flag {
    nom: &'static str,
    feuille: u32,
    reg: Reg,
    bit: u32,
}

const FLAGS: &[Flag] = &[
    Flag { nom: "FPU", feuille: 1, reg: Reg::Edx, bit: 0 },
    Flag { nom: "TSC", feuille: 1, reg: Reg::Edx, bit: 4 },
    Flag { nom: "MSR (RDMSR/WRMSR)", feuille: 1, reg: Reg::Edx, bit: 5 },
    Flag { nom: "CMPXCHG8B", feuille: 1, reg: Reg::Edx, bit: 8 },
    Flag { nom: "CMOV", feuille: 1, reg: Reg::Edx, bit: 15 },
    Flag { nom: "CLFLUSH", feuille: 1, reg: Reg::Edx, bit: 19 },
    Flag { nom: "MMX", feuille: 1, reg: Reg::Edx, bit: 23 },
    Flag { nom: "FXSR", feuille: 1, reg: Reg::Edx, bit: 24 },
    Flag { nom: "SSE", feuille: 1, reg: Reg::Edx, bit: 25 },
    Flag { nom: "SSE2", feuille: 1, reg: Reg::Edx, bit: 26 },
    Flag { nom: "SSE3", feuille: 1, reg: Reg::Ecx, bit: 0 },
    Flag { nom: "PCLMULQDQ", feuille: 1, reg: Reg::Ecx, bit: 1 },
    Flag { nom: "SSSE3", feuille: 1, reg: Reg::Ecx, bit: 9 },
    Flag { nom: "FMA", feuille: 1, reg: Reg::Ecx, bit: 12 },
    Flag { nom: "CMPXCHG16B", feuille: 1, reg: Reg::Ecx, bit: 13 },
    Flag { nom: "SSE4.1", feuille: 1, reg: Reg::Ecx, bit: 19 },
    Flag { nom: "SSE4.2", feuille: 1, reg: Reg::Ecx, bit: 20 },
    Flag { nom: "MOVBE", feuille: 1, reg: Reg::Ecx, bit: 22 },
    Flag { nom: "POPCNT", feuille: 1, reg: Reg::Ecx, bit: 23 },
    Flag { nom: "AES", feuille: 1, reg: Reg::Ecx, bit: 25 },
    Flag { nom: "XSAVE", feuille: 1, reg: Reg::Ecx, bit: 26 },
    Flag { nom: "OSXSAVE", feuille: 1, reg: Reg::Ecx, bit: 27 },
    Flag { nom: "AVX", feuille: 1, reg: Reg::Ecx, bit: 28 },
    Flag { nom: "F16C", feuille: 1, reg: Reg::Ecx, bit: 29 },
    Flag { nom: "RDRAND", feuille: 1, reg: Reg::Ecx, bit: 30 },
    Flag { nom: "HYPERVISOR", feuille: 1, reg: Reg::Ecx, bit: 31 },
    Flag { nom: "FSGSBASE (RD/WR FS/GS BASE)", feuille: 7, reg: Reg::Ebx, bit: 0 },
    Flag { nom: "TSC_ADJUST", feuille: 7, reg: Reg::Ebx, bit: 1 },
    Flag { nom: "BMI1", feuille: 7, reg: Reg::Ebx, bit: 3 },
    Flag { nom: "AVX2", feuille: 7, reg: Reg::Ebx, bit: 5 },
    Flag { nom: "SMEP", feuille: 7, reg: Reg::Ebx, bit: 7 },
    Flag { nom: "BMI2", feuille: 7, reg: Reg::Ebx, bit: 8 },
    Flag { nom: "ERMS", feuille: 7, reg: Reg::Ebx, bit: 9 },
    Flag { nom: "INVPCID", feuille: 7, reg: Reg::Ebx, bit: 10 },
    Flag { nom: "RDSEED", feuille: 7, reg: Reg::Ebx, bit: 18 },
    Flag { nom: "ADX (ADCX/ADOX)", feuille: 7, reg: Reg::Ebx, bit: 19 },
    Flag { nom: "SMAP", feuille: 7, reg: Reg::Ebx, bit: 20 },
    Flag { nom: "CLFLUSHOPT", feuille: 7, reg: Reg::Ebx, bit: 23 },
    Flag { nom: "CLWB", feuille: 7, reg: Reg::Ebx, bit: 24 },
    Flag { nom: "SHA", feuille: 7, reg: Reg::Ebx, bit: 29 },
    Flag { nom: "UMIP", feuille: 7, reg: Reg::Ecx, bit: 2 },
    Flag { nom: "PKU", feuille: 7, reg: Reg::Ecx, bit: 3 },
    Flag { nom: "GFNI", feuille: 7, reg: Reg::Ecx, bit: 8 },
    Flag { nom: "VAES", feuille: 7, reg: Reg::Ecx, bit: 9 },
    Flag { nom: "RDPID", feuille: 7, reg: Reg::Ecx, bit: 22 },
    Flag { nom: "MOVDIRI", feuille: 7, reg: Reg::Ecx, bit: 27 },
    Flag { nom: "MOVDIR64B", feuille: 7, reg: Reg::Ecx, bit: 28 },
    Flag { nom: "CET_SS", feuille: 7, reg: Reg::Ecx, bit: 7 },
    Flag { nom: "LAHF/SAHF (64 bits)", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 0 },
    Flag { nom: "SVM", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 2 },
    Flag { nom: "LZCNT (ABM)", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 5 },
    Flag { nom: "SSE4A", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 6 },
    Flag { nom: "3DNowPrefetch", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 8 },
    Flag { nom: "XOP", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 11 },
    Flag { nom: "SKINIT", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 12 },
    Flag { nom: "LWP", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 15 },
    Flag { nom: "FMA4", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 16 },
    Flag { nom: "TBM", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 21 },
    Flag { nom: "TOPOEXT", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 22 },
    Flag { nom: "MONITORX/MWAITX", feuille: 0x8000_0001, reg: Reg::Ecx, bit: 29 },
    Flag { nom: "SYSCALL/SYSRET", feuille: 0x8000_0001, reg: Reg::Edx, bit: 11 },
    Flag { nom: "NX", feuille: 0x8000_0001, reg: Reg::Edx, bit: 20 },
    Flag { nom: "PDPE1GB", feuille: 0x8000_0001, reg: Reg::Edx, bit: 26 },
    Flag { nom: "RDTSCP", feuille: 0x8000_0001, reg: Reg::Edx, bit: 27 },
    Flag { nom: "LM (mode long 64 bits)", feuille: 0x8000_0001, reg: Reg::Edx, bit: 29 },
    Flag { nom: "3DNow!", feuille: 0x8000_0001, reg: Reg::Edx, bit: 31 },
    Flag { nom: "CLZERO", feuille: 0x8000_0008, reg: Reg::Ebx, bit: 0 },
    Flag { nom: "RDPRU", feuille: 0x8000_0008, reg: Reg::Ebx, bit: 4 },
    Flag { nom: "MCOMMIT", feuille: 0x8000_0008, reg: Reg::Ebx, bit: 8 },
    Flag { nom: "WBNOINVD", feuille: 0x8000_0008, reg: Reg::Ebx, bit: 9 },
    Flag { nom: "SEV", feuille: 0x8000_001F, reg: Reg::Eax, bit: 1 },
    Flag { nom: "SEV-ES", feuille: 0x8000_001F, reg: Reg::Eax, bit: 3 },
    Flag { nom: "SEV-SNP", feuille: 0x8000_001F, reg: Reg::Eax, bit: 4 },
];

// Décodage des bits de CR0 et CR4 (AMD APM vol. 2, rév. 3.43, §3.1.1 et §3.1.3).
// Les valeurs brutes sont aussi gardées dans le rapport.
pub fn decrire_cr0(v: u64) -> Vec<String> {
    let noms = [
        (0, "PE"), (1, "MP"), (2, "EM"), (3, "TS"), (4, "ET"), (5, "NE"), (16, "WP"),
        (18, "AM"), (29, "NW"), (30, "CD"), (31, "PG"),
    ];
    let mut l = Vec::new();
    for (b, nom) in noms {
        l.push(format!("  CR0.{} (bit {}) = {}", nom, b, (v >> b) & 1));
    }
    l
}

pub fn decrire_cr4(v: u64) -> Vec<String> {
    let noms = [
        (0, "VME"), (1, "PVI"), (2, "TSD"), (3, "DE"), (4, "PSE"), (5, "PAE"), (6, "MCE"),
        (7, "PGE"), (8, "PCE"), (9, "OSFXSR"), (10, "OSXMMEXCPT"), (11, "UMIP"), (12, "LA57"),
        (16, "FSGSBASE"), (17, "PCIDE"), (18, "OSXSAVE"), (20, "SMEP"), (21, "SMAP"),
        (22, "PKE"), (23, "CET"),
    ];
    let mut l = Vec::new();
    for (b, nom) in noms {
        l.push(format!("  CR4.{} (bit {}) = {}", nom, b, (v >> b) & 1));
    }
    l
}

fn bit_de(r: &CpuidResult, reg: Reg) -> u32 {
    match reg {
        Reg::Eax => r.eax,
        Reg::Ebx => r.ebx,
        Reg::Ecx => r.ecx,
        Reg::Edx => r.edx,
    }
}

fn nom_reg(reg: Reg) -> &'static str {
    match reg { Reg::Eax => "EAX", Reg::Ebx => "EBX", Reg::Ecx => "ECX", Reg::Edx => "EDX" }
}

fn chaine_cpuid(feuilles: &[CpuidResult]) -> String {
    let mut o: Vec<u8> = Vec::new();
    for c in feuilles {
        for v in [c.eax, c.ebx, c.ecx, c.edx] {
            o.extend_from_slice(&v.to_le_bytes());
        }
    }
    String::from_utf8_lossy(&o).trim_matches(|c: char| c == '\0' || c == ' ').into()
}

// Renvoie les lignes du rapport de la sonde. Ne lit que CPUID et, si OSXSAVE, XGETBV(0).
pub fn lignes() -> Vec<String> {
    let mut l: Vec<String> = Vec::new();
    let max_std = __cpuid(0).eax;
    let max_ext = __cpuid(0x8000_0000).eax;
    let c0 = __cpuid(0);
    // CPUID 0 renvoie le constructeur en EBX, EDX, ECX (dans cet ordre) : on remet les champs dans cet ordre.
    let constructeur = chaine_cpuid(&[CpuidResult { eax: 0, ebx: c0.ebx, ecx: c0.edx, edx: c0.ecx }]);
    l.push("--- SONDE CPU (CPUID seul, aucun MSR) ---".into());
    l.push(format!("Constructeur : {}", constructeur));
    l.push(format!("Feuille standard max {:#x} ; étendue max {:#x}", max_std, max_ext));

    let f1 = __cpuid(1);
    let base_fam = (f1.eax >> 8) & 0xF;
    let ext_fam = (f1.eax >> 20) & 0xFF;
    let base_mod = (f1.eax >> 4) & 0xF;
    let ext_mod = (f1.eax >> 16) & 0xF;
    let fam = if base_fam == 0xF { base_fam + ext_fam } else { base_fam };
    let modl = if base_fam == 0xF || base_fam == 6 { (ext_mod << 4) | base_mod } else { base_mod };
    l.push(format!("Famille {:#x} modèle {:#x} stepping {:#x} (CPUID 1 EAX {:#010x})", fam, modl, f1.eax & 0xF, f1.eax));

    if max_ext >= 0x8000_0004 {
        let nom = chaine_cpuid(&[__cpuid(0x8000_0002), __cpuid(0x8000_0003), __cpuid(0x8000_0004)]);
        l.push(format!("Nom : {}", nom));
    }

    l.push("--- Fonctionnalités (feuille.registre[bit]) ---".into());
    for f in FLAGS {
        let max = if f.feuille >= 0x8000_0000 { max_ext } else { max_std };
        let etat = if f.feuille > max {
            "feuille absente"
        } else {
            let r = __cpuid_count(f.feuille, 0);
            if (bit_de(&r, f.reg) >> f.bit) & 1 == 1 { "OUI" } else { "non" }
        };
        l.push(format!("{} {:#x}.{}[{}] : {}", f.nom, f.feuille, nom_reg(f.reg), f.bit, etat));
    }

    l.push("--- États activés par l'OS (XCR0) ---".into());
    if (f1.ecx >> 27) & 1 == 1 {
        let x = unsafe { core::arch::x86_64::_xgetbv(0) };
        l.push(format!("XCR0 {:#x} : x87 {} SSE {} AVX {} opmask/ZMM {}",
            x, x & 1, (x >> 1) & 1, (x >> 2) & 1, (x >> 5) & 1));
    } else {
        l.push("OSXSAVE absent : XCR0 non lu".into());
    }

    l.push("--- Caches (CPUID 4) ---".into());
    if max_std >= 4 {
        for i in 0..16u32 {
            let r = __cpuid_count(4, i);
            let typ = r.eax & 0x1F;
            if typ == 0 { break; }
            let niveau = (r.eax >> 5) & 0x7;
            let lignes = (r.ebx & 0xFFF) + 1;
            let parts = ((r.ebx >> 12) & 0x3FF) + 1;
            let assoc = ((r.ebx >> 22) & 0x3FF) + 1;
            let ens = r.ecx + 1;
            let taille = assoc as u64 * parts as u64 * lignes as u64 * ens as u64;
            let type_nom = match typ { 1 => "données", 2 => "instructions", 3 => "unifié", _ => "?" };
            l.push(format!("L{} {} : {} Kio, {} voies, ligne {} o", niveau, type_nom, taille / 1024, assoc, lignes));
        }
    } else {
        l.push("feuille 4 absente".into());
    }

    l.push("--- Topologie (CPUID B) ---".into());
    if max_std >= 0xB {
        let r0 = __cpuid_count(0xB, 0);
        let r1 = __cpuid_count(0xB, 1);
        l.push(format!("processeurs logiques par paquet : {}", r1.ebx & 0xFFFF));
        l.push(format!("bits APIC au niveau thread : {}", r0.eax & 0x1F));
    } else {
        l.push("feuille B absente".into());
    }

    if max_ext >= 0x8000_0008 {
        let r = __cpuid(0x8000_0008);
        l.push(format!("bits adresses physiques {} ; virtuelles {}", r.eax & 0xFF, (r.eax >> 8) & 0xFF));
    }

    // Fonctions étendues décrites dans APM vol. 3 rév. 3.39, annexe E (E.4.9, E.4.11, E.4.15,
    // E.4.16, E.4.17). Lecture seule, CPUID uniquement.
    if max_ext >= 0x8000_000A {
        let r = __cpuid(0x8000_000A);
        l.push(format!("SVM : révision {} ; ASID disponibles {}", r.eax & 0xFF, r.ebx));
    }
    if max_ext >= 0x8000_0019 {
        let r = __cpuid(0x8000_0019);
        l.push(format!("TLB 1 Go : L1 données {} entrées, L1 instructions {} entrées ; L2 données {} entrées, L2 instructions {} entrées",
            (r.eax >> 16) & 0xFFF, r.eax & 0xFFF, (r.ebx >> 16) & 0xFFF, r.ebx & 0xFFF));
    }
    if max_ext >= 0x8000_001D {
        l.push("--- Topologie des caches (CPUID 8000_001D, APM E.4.15) ---".into());
        for i in 0..8u32 {
            let r = __cpuid_count(0x8000_001D, i);
            let typ = r.eax & 0x1F;
            if typ == 0 { break; }
            let niveau = (r.eax >> 5) & 0x7;
            let partage = ((r.eax >> 14) & 0xFFF) + 1;
            let voies = ((r.ebx >> 22) & 0x3FF) + 1;
            let lignes_n = (r.ebx & 0xFFF) + 1;
            let parts = ((r.ebx >> 12) & 0x3FF) + 1;
            let ens = r.ecx + 1;
            let taille = voies as u64 * parts as u64 * lignes_n as u64 * ens as u64;
            let type_nom = match typ { 1 => "données", 2 => "instructions", 3 => "unifié", _ => "?" };
            l.push(format!("  cache {} : niveau {} {} : {} Kio, {} voies, partagé par {} processeurs logiques",
                i, niveau, type_nom, taille / 1024, voies, partage));
        }
    }
    if max_ext >= 0x8000_001E {
        let r = __cpuid(0x8000_001E);
        l.push(format!("Topologie : APIC étendu {:#x} ; threads par unité de calcul {} ; unité de calcul {} ; nœud {} sur {} nœud(s) par paquet",
            r.eax, ((r.ebx >> 8) & 0xFF) + 1, r.ebx & 0xFF, r.ecx & 0xFF, ((r.ecx >> 8) & 0x7) + 1));
    }
    if max_ext >= 0x8000_001F {
        let r = __cpuid(0x8000_001F);
        let noms = [
            (0, "SME"), (1, "SEV"), (2, "PageFlushMsr"), (3, "SEV-ES"), (4, "SEV-SNP"),
            (5, "VMPL"), (6, "RMPQUERY"), (7, "VmplSSS"), (8, "SecureTsc"), (9, "TscAuxVirt"),
            (10, "HwEnfCacheCoh"), (11, "64BitHost"), (12, "RestrictedInj"), (13, "AlternateInj"), (14, "DebugVirt"),
        ];
        let mut actifs: Vec<&str> = Vec::new();
        for (b, nom) in noms {
            if (r.eax >> b) & 1 == 1 { actifs.push(nom); }
        }
        l.push(format!("SEV (8000_001F EAX) : {}", if actifs.is_empty() { String::from("aucun") } else { actifs.join(", ") }));
        l.push(format!("SEV : VMPL {} ; bit C {} ; réduction adresse physique {} ; invités chiffrés simultanés {} ; ASID min {}",
            (r.ebx >> 12) & 0xF, r.ebx & 0x3F, (r.ebx >> 6) & 0x3F, r.ecx, r.edx));
    }
    l
}
