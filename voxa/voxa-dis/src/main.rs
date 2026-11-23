//! voxa-dis : desassembleur maison, LECTURE SEULE. Modes :
//!   voxa-dis pe <fichier.efi> [--listing sortie.asm]      un module UEFI (PE32+, PE32 ou TE)
//!   voxa-dis lot <dossier> <sortie>                       tous les modules d'un dossier (listing par module + resume.csv)
//!   voxa-dis ec <fichier> [--debut hex] [--fin hex]       micrologiciel 8051 du controleur embarque
//!   voxa-dis x86 <hex>                                    desassemble quelques octets (aide au test)
mod mcs51;
mod nds32;
mod x86;
use std::fmt::Write as _;
use x86::Flow;

fn le16(d: &[u8], o: usize) -> usize { d.get(o..o + 2).map_or(0, |x| u16::from_le_bytes([x[0], x[1]]) as usize) }
fn le32(d: &[u8], o: usize) -> usize { d.get(o..o + 4).map_or(0, |x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]) as usize) }
fn le64(d: &[u8], o: usize) -> u64 { d.get(o..o + 8).map_or(0, |x| u64::from_le_bytes(x.try_into().unwrap())) }

struct Section { nom: String, va: u64, off: usize, taille: usize, exec: bool }
struct Image { machine: usize, base: u64, entree: u64, sections: Vec<Section>, te: bool }

fn lire_image(d: &[u8]) -> Option<Image> {
    if d.starts_with(b"VZ") { // TE : en-tete de 40 octets
        let nsec = *d.get(4)? as usize; let strip = le16(d, 6); let mut sections = Vec::new();
        for k in 0..nsec { let s = 40 + k * 40; let va = le32(d, s + 12) as u64; let raw = le32(d, s + 20); let rs = le32(d, s + 16); let fl = le32(d, s + 36);
            let off = raw.checked_sub(strip)? + 40; if off + rs > d.len() { return None; } sections.push(Section { nom: String::from_utf8_lossy(&d[s..s + 8]).trim_end_matches('\0').to_string(), va, off, taille: rs, exec: fl & 0x2000_0000 != 0 || fl & 0x20 != 0 }); }
        return Some(Image { machine: le16(d, 2), base: le64(d, 16), entree: le32(d, 8) as u64, sections, te: true });
    }
    if !d.starts_with(b"MZ") { return None; } let pe = le32(d, 0x3C); if d.get(pe..pe + 4)? != b"PE\0\0" { return None; }
    let machine = le16(d, pe + 4); let nsec = le16(d, pe + 6); let osz = le16(d, pe + 0x14); let opt = pe + 0x18; let magic = le16(d, opt);
    let base = if magic == 0x20b { le64(d, opt + 24) } else { le32(d, opt + 28) as u64 }; let entree = le32(d, opt + 16) as u64; let mut sections = Vec::new();
    for k in 0..nsec { let s = opt + osz + k * 40; if s + 40 > d.len() { return None; } let vs = le32(d, s + 8); let va = le32(d, s + 12) as u64; let rs = le32(d, s + 16); let raw = le32(d, s + 20); let fl = le32(d, s + 36);
        let taille = rs.min(vs.max(1)).min(d.len().saturating_sub(raw)); sections.push(Section { nom: String::from_utf8_lossy(&d[s..s + 8]).trim_end_matches('\0').to_string(), va, off: raw, taille, exec: fl & 0x2000_0000 != 0 || fl & 0x20 != 0 }); }
    Some(Image { machine, base, entree, sections, te: false })
}

#[derive(Default, Clone)]
struct Stats { insn: usize, exacts: usize, generiques: usize, octets_code: usize, octets_db: usize, appels: usize, sauts: usize, chaines: usize }

/// Chaine lisible (ASCII ou UTF-16) a l'adresse virtuelle `va` si elle existe dans l'image.
fn chaine_a(img: &Image, d: &[u8], va: u64) -> Option<String> {
    let rva = va.checked_sub(img.base)?; let s = img.sections.iter().find(|s| rva >= s.va && rva < s.va + s.taille as u64)?; let o = s.off + (rva - s.va) as usize; let b = d.get(o..(o + 160).min(d.len()))?;
    let a: String = b.iter().take_while(|&&c| (32..127).contains(&c)).map(|&c| c as char).collect(); if a.len() >= 5 && b.get(a.len()) == Some(&0) { return Some(a); }
    let mut u = String::new(); for p in b.chunks_exact(2) { let c = u16::from_le_bytes([p[0], p[1]]); if (32..127).contains(&c) { u.push(c as u8 as char); } else { break; } } if u.len() >= 4 && b.get(u.len() * 2..u.len() * 2 + 2) == Some(&[0, 0][..]) { return Some(u); }
    None
}

fn desassembler_image(nom: &str, d: &[u8], listing: Option<&mut String>) -> Result<Stats, String> {
    let img = lire_image(d).ok_or("format non reconnu")?; let m64 = img.machine == 0x8664; if !(m64 || img.machine == 0x14c) { return Err(format!("machine {:#x} non geree", img.machine)); }
    let mut st = Stats::default(); let mut out = String::new(); let _ = writeln!(out, "; {nom} : machine {:#x}{}, base {:#x}, entree {:#x}", img.machine, if img.te { " (TE)" } else { "" }, img.base, img.base + img.entree);
    for s in img.sections.iter().filter(|s| s.exec) {
        let code = &d[s.off..s.off + s.taille]; let _ = writeln!(out, "; section {} va {:#x} taille {}", s.nom, img.base + s.va, s.taille); let mut p = 0;
        while p < code.len() {
            let addr = img.base + s.va + p as u64;
            match x86::decode(&code[p..], addr, m64) {
                Some(i) => { let hex: String = code[p..p + i.len].iter().map(|b| format!("{b:02x}")).collect(); let mut ligne = format!("{addr:#012x}: {hex:<24} {}", i.text);
                    if let Some(r) = i.rip_ref { if let Some(c) = chaine_a(&img, d, r) { st.chaines += 1; let _ = write!(ligne, "   ; {r:#x} -> \"{c}\""); } else { let _ = write!(ligne, "   ; {r:#x}"); } }
                    out.push_str(&ligne); out.push('\n'); st.insn += 1; if i.exact { st.exacts += 1; } else { st.generiques += 1; } st.octets_code += i.len; match i.flow { Flow::Call => st.appels += 1, Flow::Jmp | Flow::Jcc | Flow::Indirect => st.sauts += 1, _ => {} } p += i.len; }
                None => { let _ = writeln!(out, "{addr:#012x}: {:02x}                       db {:#04x}", code[p], code[p]); st.octets_db += 1; p += 1; }
            }
        }
    }
    if let Some(l) = listing { l.push_str(&out); } Ok(st)
}

fn pourcent(a: usize, b: usize) -> f64 { if b == 0 { 100.0 } else { 100.0 * a as f64 / b as f64 } }

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(|s| s.as_str()) {
        Some("pe") => { let Some(f) = a.get(2) else { eprintln!("usage : pe <fichier>"); return }; let d = std::fs::read(f).unwrap_or_default(); let mut l = String::new();
            match desassembler_image(f, &d, Some(&mut l)) { Ok(s) => { println!("INSTRUCTIONS={} NOMMEES={:.2} % GENERIQUES={} OCTETS_CODE={} OCTETS_DB={} COUVERTURE={:.2} % APPELS={} SAUTS={} CHAINES={}", s.insn, pourcent(s.exacts, s.insn), s.generiques, s.octets_code, s.octets_db, pourcent(s.octets_code, s.octets_code + s.octets_db), s.appels, s.sauts, s.chaines);
                if let Some(i) = a.iter().position(|x| x == "--listing") { if let Some(p) = a.get(i + 1) { std::fs::write(p, l).ok(); } } } Err(e) => println!("ECHEC : {e}") } }
        Some("lot") => { let (Some(dir), Some(sortie)) = (a.get(2), a.get(3)) else { eprintln!("usage : lot <dossier> <sortie>"); return }; std::fs::create_dir_all(sortie).ok();
            let mut csv = String::from("module;machine;instructions;nommees_pct;generiques;octets_code;octets_db;couverture_pct;appels;sauts;chaines\n"); let mut tot = Stats::default(); let (mut ok, mut ko, mut ignores) = (0, 0, 0);
            let mut fichiers: Vec<_> = std::fs::read_dir(dir).map(|r| r.flatten().map(|e| e.path()).collect()).unwrap_or_default(); fichiers.sort();
            for p in fichiers { let d = std::fs::read(&p).unwrap_or_default(); if !(d.starts_with(b"MZ") || d.starts_with(b"VZ")) { ignores += 1; continue; } let nom = p.file_name().unwrap().to_string_lossy().to_string(); let mut l = String::new();
                match desassembler_image(&nom, &d, Some(&mut l)) { Ok(s) => { ok += 1; std::fs::write(format!("{sortie}/{nom}.asm"), &l).ok(); let _ = writeln!(csv, "{nom};{};{};{:.2};{};{};{};{:.2};{};{};{}", if d.starts_with(b"VZ") { "TE" } else { "PE" }, s.insn, pourcent(s.exacts, s.insn), s.generiques, s.octets_code, s.octets_db, pourcent(s.octets_code, s.octets_code + s.octets_db), s.appels, s.sauts, s.chaines);
                    tot.insn += s.insn; tot.exacts += s.exacts; tot.generiques += s.generiques; tot.octets_code += s.octets_code; tot.octets_db += s.octets_db; tot.appels += s.appels; tot.sauts += s.sauts; tot.chaines += s.chaines; } Err(e) => { ko += 1; let _ = writeln!(csv, "{nom};ECHEC {e};;;;;;;;;"); } } }
            std::fs::write(format!("{sortie}/resume.csv"), csv).ok();
            println!("MODULES={ok} ECHECS={ko} NON_EXECUTABLES_IGNORES={ignores} INSTRUCTIONS={} NOMMEES={:.2} % GENERIQUES={} OCTETS_CODE={} OCTETS_DB={} COUVERTURE={:.2} % APPELS={} SAUTS={} CHAINES_REFERENCEES={}", tot.insn, pourcent(tot.exacts, tot.insn), tot.generiques, tot.octets_code, tot.octets_db, pourcent(tot.octets_code, tot.octets_code + tot.octets_db), tot.appels, tot.sauts, tot.chaines); }
        Some("ec") => { let Some(f) = a.get(2) else { eprintln!("usage : ec <fichier> [--isa nds32|8051] [--debut hex] [--fin hex] [--listing f]"); return }; let d = std::fs::read(f).unwrap_or_default();
            let val = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).and_then(|s| usize::from_str_radix(s.trim_start_matches("0x"), 16).ok());
            let isa = a.iter().position(|x| x == "--isa").and_then(|i| a.get(i + 1)).map(|s| s.as_str()).unwrap_or("nds32");
            let (debut, fin) = (val("--debut").unwrap_or(0), val("--fin").unwrap_or(d.len()).min(d.len())); let code = &d[debut.min(fin)..fin]; let mut p = 0; let mut out = String::new();
            let (mut n, mut db, mut octets, mut appels, mut sauts, mut retours, mut inconnus, mut i16, mut i32_, mut prologues) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
            let mut starts = std::collections::HashSet::new(); let mut cibles: Vec<i64> = Vec::new();
            while p < code.len() {
                let pc = (debut + p) as i64;
                let r: Option<(usize, String, Flow, Option<i64>, bool)> = if isa == "8051" { mcs51::decode(&code[p..], pc as u32).map(|i| (i.len, i.text, i.flow, i.target.map(|t| t as i64), true)) } else { nds32::decode(&code[p..], pc).map(|i| (i.len, i.text, i.flow, i.target, i.known)) };
                match r { Some((len, text, flow, tg, known)) => { let hex: String = code[p..p + len].iter().map(|b| format!("{b:02x}")).collect(); let _ = writeln!(out, "{pc:#08x}: {hex:<10} {text}"); n += 1; octets += len; starts.insert(pc); if !known { inconnus += 1; } if len == 2 { i16 += 1; } else if len == 4 { i32_ += 1; }
                        if text.starts_with("push25") { prologues += 1; } match flow { Flow::Call => appels += 1, Flow::Jmp | Flow::Jcc | Flow::Indirect => sauts += 1, Flow::Ret => retours += 1, _ => {} } if let Some(t) = tg { if flow != Flow::Indirect { cibles.push(t); } } p += len; }
                    None => { let _ = writeln!(out, "{pc:#08x}: {:02x}       db {:#04x}", code[p], code[p]); db += 1; p += 1; } } }
            let (deb, fi) = (debut as i64, fin as i64); let dedans = cibles.iter().filter(|&&t| t >= deb && t < fi).count(); let sur_debut = cibles.iter().filter(|&&t| starts.contains(&t)).count();
            println!("ISA={isa} OCTETS={} INSTRUCTIONS={n} (16 bits={i16} 32 bits={i32_}) OCTETS_DECODES={octets} OCTETS_DB={db} NON_RECONNUES={inconnus} APPELS={appels} SAUTS={sauts} RETOURS={retours} PROLOGUES_PUSH25={prologues}", code.len());
            println!("COHERENCE : cibles_de_branchement={} dans_la_zone={} ({:.1} %) tombant_sur_un_debut_d_instruction={} ({:.1} %)", cibles.len(), dedans, pourcent(dedans, cibles.len()), sur_debut, pourcent(sur_debut, cibles.len()));
            if let Some(i) = a.iter().position(|x| x == "--listing") { if let Some(p) = a.get(i + 1) { std::fs::write(p, out).ok(); } } }
        Some("verifier") => { let (Some(f), Some(t)) = (a.get(2), a.get(3)) else { eprintln!("usage : verifier <fichier.efi> <sortie-dumpbin.txt> [--details]"); return }; verifier(f, t, a.iter().any(|x| x == "--details")); }
        Some("x86") => { let hex: String = a[2..].join(""); let b: Vec<u8> = (0..hex.len() / 2).filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()).collect(); let mut p = 0;
            while p < b.len() { match x86::decode(&b[p..], 0x1000 + p as u64, true) { Some(i) => { println!("{:#06x}: {}", 0x1000 + p, i.text); p += i.len; } None => { println!("{:#06x}: db {:#04x}", 0x1000 + p, b[p]); p += 1; } } } }
        _ => println!("usage : voxa-dis pe <f> | lot <dossier> <sortie> | ec <f> [--isa nds32|8051] [--debut h --fin h] | verifier <f.efi> <dumpbin.txt> | x86 <hex>"),
    }
}

// ---- verification contre une verite terrain externe : la sortie de `dumpbin /disasm` de Microsoft ----
fn nombres_et_texte(s: &str) -> (String, Vec<u64>) {
    // sépare les nombres (hexa 0x.., hexa suffixe h, decimaux) du reste ; le reste est normalise (minuscules, sans espaces)
    let c: Vec<char> = s.to_lowercase().chars().collect(); let mut t = String::new(); let mut v = Vec::new(); let mut i = 0;
    while i < c.len() {
        let neg = c[i] == '-' && i + 1 < c.len() && c[i + 1].is_ascii_digit();
        let j0 = if neg { i + 1 } else { i };
        if j0 < c.len() && (c[j0].is_ascii_digit() || (c[j0].is_ascii_hexdigit() && { let mut k = j0; while k < c.len() && c[k].is_ascii_hexdigit() { k += 1; } k < c.len() && c[k] == 'h' && k - j0 >= 2 })) && (j0 == 0 || !(c[j0 - 1].is_ascii_alphabetic() || c[j0 - 1] == '_')) {
            let mut j = j0; while j < c.len() && c[j].is_ascii_hexdigit() { j += 1; } let tok: String = c[j0..j].iter().collect();
            let (val, fin) = if tok.starts_with("0x") || (j < c.len() && c[j] == 'x' && tok == "0") { let mut k = j + 1; while k < c.len() && c[k].is_ascii_hexdigit() { k += 1; } (u64::from_str_radix(&c[j + 1..k].iter().collect::<String>(), 16).unwrap_or(0), k) }
                else if j < c.len() && c[j] == 'h' { (u64::from_str_radix(&tok, 16).unwrap_or(0), j + 1) } else if tok.len() >= 8 { (u64::from_str_radix(&tok, 16).unwrap_or(0), j) } else if tok.chars().all(|x| x.is_ascii_digit()) { (tok.parse().unwrap_or(0), j) } else { t.push(c[i]); i += 1; continue };
            let val = if neg { 0u64.wrapping_sub(val) } else { val }; v.push(val); t.push('#'); i = fin; continue; }
        if c[i] != ' ' && c[i] != '\t' { t.push(c[i]); } i += 1; }
    (t.replace("+#", "#"), v)
}
fn nombres_egaux(a: u64, b: u64) -> bool { a == b || (a & 0xFFFF_FFFF) == (b & 0xFFFF_FFFF) || (a & 0xFFFF) == (b & 0xFFFF) || (a & 0xFF) == (b & 0xFF) }
fn alias_mnem(m: &str) -> String { match m { "int3" => "int".into(), "fwait" => "wait".into(), "pushf" => "pushfq".into(), "popf" => "popfq".into(), "je" => "je".into(), "jz" => "je".into(), "jnz" => "jne".into(), "jb" | "jc" => "jb".into(), "jnb" | "jae" | "jnc" => "jae".into(), "jna" => "jbe".into(), "ja" | "jnbe" => "ja".into(), "jnge" => "jl".into(), "jge" | "jnl" => "jge".into(), "jng" => "jle".into(), "jg" | "jnle" => "jg".into(), "jpe" => "jp".into(), "jpo" => "jnp".into(), o => o.into() } }

fn verifier(fichier: &str, dumpbin: &str, details: bool) {
    let d = std::fs::read(fichier).unwrap_or_default(); let Some(img) = lire_image(&d) else { println!("format non reconnu"); return }; let m64 = img.machine == 0x8664;
    let txt = std::fs::read(dumpbin).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
    let (mut n, mut longueur, mut mnem, mut texte, mut absent) = (0usize, 0usize, 0usize, 0usize, 0usize); let mut ecarts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let lignes: Vec<&str> = txt.lines().collect();
    for (li, l) in lignes.iter().enumerate() {
        let l = l.trim_start(); if l.len() < 20 || l.as_bytes()[16] != b':' || !l[..16].chars().all(|c| c.is_ascii_hexdigit()) { continue; }
        let Ok(addr) = u64::from_str_radix(&l[..16], 16) else { continue }; let reste = l[17..].trim_start(); let mut octets = 0usize; let mut pos = 0; let b = reste.as_bytes();
        while pos + 2 <= b.len() && (b[pos] as char).is_ascii_hexdigit() && (b[pos + 1] as char).is_ascii_hexdigit() && (pos + 2 == b.len() || b[pos + 2] == b' ') { octets += 1; pos += 3; }
        let asm = reste.get(pos.min(reste.len())..).unwrap_or("").trim(); if asm.is_empty() { continue; }
        { let mut k = li + 1; while k < lignes.len() { let s = lignes[k].trim(); if s.is_empty() || !s.split_whitespace().all(|w| w.len() == 2 && w.chars().all(|c| c.is_ascii_hexdigit())) { break; } octets += s.split_whitespace().count(); k += 1; } } n += 1;
        let Some(sec) = img.sections.iter().find(|s| addr >= img.base + s.va && addr < img.base + s.va + s.taille as u64) else { absent += 1; continue };
        let off = sec.off + (addr - img.base - sec.va) as usize; let code = &d[off..sec.off + sec.taille];
        let Some(i) = x86::decode(code, addr, m64) else { absent += 1; *ecarts.entry(format!("{asm}  =>  (invalide)")).or_default() += 1; continue };
        if i.len == octets { longueur += 1; }
        let mut nous = i.text.clone(); if let Some(r) = i.rip_ref { if let (Some(a), Some(z)) = (nous.find("[rip"), nous.find(']')) { if z > a { nous = format!("{}[{:#x}]{}", &nous[..a], r, &nous[z + 1..]); } } }
        let (mut a_m, a_o) = match asm.split_once(char::is_whitespace) { Some((m, o)) => (m.to_string(), o.to_string()), None => (asm.to_string(), String::new()) };
        let mut prefixe = String::new(); while ["lock", "rep", "repe", "repne", "bnd", "notrack"].contains(&a_m.as_str()) { prefixe.push_str(&a_m); prefixe.push(' '); let r = a_o.clone(); let (m2, o2) = match r.trim().split_once(char::is_whitespace) { Some((m, o)) => (m.to_string(), o.to_string()), None => (r.trim().to_string(), String::new()) }; a_m = m2; let _ = o2; break; }
        let nous = normaliser_nous(&nous); let (n_m, n_o) = match nous.split_once(char::is_whitespace) { Some((m, o)) => (m.to_string(), o.to_string()), None => (nous.clone(), String::new()) };
        let asm_norm = normaliser_ref(asm);
        let ref_complet = asm_norm.to_lowercase(); let ref_m = alias_mnem(&ref_complet.split_whitespace().next().unwrap_or("")); let n_m_alias = alias_mnem(&n_m);
        let (t1, v1) = nombres_et_texte(&format!("{}", ref_complet.splitn(2, char::is_whitespace).nth(1).unwrap_or("").trim())); let (t2, v2) = nombres_et_texte(&n_o);
        let m_ok = ref_m == n_m_alias || ref_complet.split_whitespace().skip(1).next().map(|x| alias_mnem(x) == n_m_alias).unwrap_or(false);
        if m_ok { mnem += 1; }
        let (t1, t2) = (t1.replace("ptr", ""), t2.replace("ptr", ""));
        let ops_ok = t1 == t2 && v1.len() == v2.len() && v1.iter().zip(v2.iter()).all(|(a, b)| nombres_egaux(*a, *b));
        if m_ok && ops_ok { texte += 1; } else { *ecarts.entry(format!("{}  =>  {}", asm.split_whitespace().collect::<Vec<_>>().join(" "), nous)).or_default() += 1; }
        let _ = (&a_o, &n_m, &prefixe);
    }
    println!("REFERENCE={n} LONGUEUR_IDENTIQUE={longueur} ({:.3} %) MNEMONIQUE_IDENTIQUE={mnem} ({:.3} %) TEXTE_EQUIVALENT={texte} ({:.3} %) HORS_IMAGE_OU_INVALIDE={absent}", pourcent(longueur, n), pourcent(mnem, n), pourcent(texte, n));
    if details { let mut v: Vec<_> = ecarts.into_iter().collect(); v.sort_by(|a, b| b.1.cmp(&a.1)); for (k, c) in v.into_iter().take(40) { println!("  {c:>5} x {k}"); } }
}

fn normaliser_ref(a: &str) -> String {
    let b = a.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase().replace(" mmword ptr", " qword ptr").replace(",mmword ptr", ",qword ptr");
    if b == "int 3" { return "int3".into(); }
    for (pre, nom) in [("rep stos ", "stos"), ("rep movs ", "movs"), ("rep lods ", "lods"), ("repne scas ", "scas"), ("repe cmps ", "cmps"), ("repe scas ", "scas"), ("repne cmps ", "cmps"), ("stos ", "stos"), ("outs ", "outs"), ("ins ", "ins"), ("movs ", "movs"), ("lods ", "lods"), ("scas ", "scas"), ("cmps ", "cmps")] {
        if b.starts_with(pre) { let suf = if b.contains("byte ptr") { "b" } else if b.contains("qword ptr") { "q" } else if b.contains("dword ptr") { "d" } else { "w" }; let p = pre.split_whitespace().take(pre.split_whitespace().count() - 1).collect::<Vec<_>>().join(" "); return if p.is_empty() { format!("{nom}{suf}") } else { format!("{p} {nom}{suf}") }; } }
    let (m, o) = b.split_once(' ').unwrap_or((b.as_str(), ""));
    if ["mul", "div", "idiv"].contains(&m) { let v: Vec<&str> = o.split(',').collect(); if v.len() == 2 && ["rax", "eax", "ax", "al"].contains(&v[0].trim()) { return format!("{m} {}", v[1]); } }
    if m == "xchg" { let v: Vec<&str> = o.split(',').collect(); if v.len() == 2 { let (x, y) = (v[0].trim(), v[1].trim()); let mut p = [x, y]; p.sort(); return format!("xchg {},{}", p[0], p[1]); } }
    if ["clflush", "fxsave", "fxrstor", "prefetcht0", "prefetcht1", "prefetcht2", "prefetchnta"].contains(&m) { return format!("{m} {}", o.replace("byte ptr ", "")); }
    b
}
fn normaliser_nous(n: &str) -> String {
    let (m, o) = n.split_once(' ').unwrap_or((n, ""));
    if m == "xchg" { let v: Vec<&str> = o.split(',').collect(); if v.len() == 2 { let mut p = [v[0].trim(), v[1].trim()]; p.sort(); return format!("xchg {},{}", p[0], p[1]); } }
    if ["clflush", "fxsave", "fxrstor", "prefetcht0", "prefetcht1", "prefetcht2", "prefetchnta"].contains(&m) { return format!("{m} {}", o.replace("byte ptr ", "")); }
    if let Some(r) = n.strip_prefix("rep ").or_else(|| n.strip_prefix("repne ")) { let _ = r; }
    n.to_string()
}
