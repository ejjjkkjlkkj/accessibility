//! Desassembleur NDS32 (Andes AndeStar V3, instructions de 16 et 32 bits) en GROS-BOUTIEN (big-endian), ecrit pour le micrologiciel du controleur embarque ITE.
//! Constantes de codage : table publique de l'en-tete open source `include/opcode/nds32.h` (binutils/GDB), relue le 2026-10-12 (contrat d'interoperabilite, aucun code repris).
//! Les registres a 4 bits (formats 16 bits) sont affiches tels quels (r0..r15) sans application de la table de correspondance de l'ISA : ils sont marques `r4:n`.
use super::x86::Flow;

pub struct Nds { pub len: usize, pub text: String, pub flow: Flow, pub target: Option<i64>, pub known: bool }

fn sx(v: u32, bits: u32) -> i64 { let m = 1u32 << (bits - 1); ((v ^ m).wrapping_sub(m)) as i32 as i64 }
fn r(n: u32) -> String { match n { 30 => "lp".into(), 31 => "sp".into(), 28 => "fp".into(), 29 => "gp".into(), _ => format!("r{n}") } }
fn r4(n: u32) -> String { format!("r4:{n}") }
fn sgn(v: i64) -> String { if v < 0 { format!("-{:#x}", -v) } else { format!("{v:#x}") } }

const ALU1: [&str; 32] = ["add", "sub", "and", "xor", "or", "nor", "slt", "slts", "slli", "srli", "srai", "rotri", "sll", "srl", "sra", "rotr", "seb", "seh", "bitc", "zeh", "wsbh", "or_srli", "divsr", "divr", "sva", "svs", "cmovz", "cmovn", "add_srli", "sub_srli", "and_srli", "xor_srli"];
fn alu2(s: u32) -> Option<&'static str> { Some(match s { 0 => "max", 1 => "min", 2 => "ave", 3 => "abs", 4 => "clips", 5 => "clip", 6 => "clo", 7 => "clz", 8 => "bset", 9 => "bclr", 10 => "btgl", 11 => "btst", 12 => "bse", 13 => "bsp", 14 => "ffb", 15 => "ffmism", 16 => "add.sc", 17 => "sub.sc", 18 => "add.wc", 19 => "sub.wc", 23 => "ffzmism", 24 => "kadd", 25 => "kmul", 32 => "mfusr", 33 => "mtusr", 36 => "mul", 40 => "mults64", 41 => "mult64", 42 => "madds64", 43 => "madd64", 44 => "msubs64", 45 => "msub64", 46 => "divs", 47 => "div", 49 => "mult32", 51 => "madd32", 53 => "msub32", _ => return None }) }
fn mem_sub(s: u32) -> Option<&'static str> { Some(match s { 0 => "lb", 1 => "lh", 2 => "lw", 3 => "ld", 4 => "lb.bi", 5 => "lh.bi", 6 => "lw.bi", 7 => "ld.bi", 8 => "sb", 9 => "sh", 10 => "sw", 11 => "sd", 12 => "sb.bi", 13 => "sh.bi", 14 => "sw.bi", 15 => "sd.bi", 16 => "lbs", 17 => "lhs", 18 => "lws", 19 => "dpref", 20 => "lbs.bi", 21 => "lhs.bi", 22 => "lws.bi", 24 => "llw", 25 => "scw", 32 => "lbup", 34 => "lwup", 40 => "sbup", 42 => "swup", _ => return None }) }
fn misc(s: u32) -> Option<&'static str> { Some(match s { 0 => "standby", 1 => "cctl", 2 => "mfsr", 3 => "mtsr", 4 => "iret", 5 => "trap", 6 => "teqz", 7 => "tnez", 8 => "dsb", 9 => "isb", 10 => "break", 11 => "syscall", 12 => "msync", 13 => "isync", 14 => "tlbop", _ => return None }) }

/// Decode une instruction a `pc` (adresse de l'instruction). `code` = octets en gros-boutien. None si octets manquants.
pub fn decode(code: &[u8], pc: i64) -> Option<Nds> {
    let h = (*code.first()? as u32) << 8 | *code.get(1)? as u32;
    if h & 0x8000 != 0 { return Some(decode16(h, pc)); }
    let w = h << 16 | (*code.get(2)? as u32) << 8 | *code.get(3)? as u32; Some(decode32(w, pc))
}

fn decode32(w: u32, pc: i64) -> Nds {
    let op = (w >> 25) & 0x3F; let (rt, ra, rb) = ((w >> 20) & 31, (w >> 15) & 31, (w >> 10) & 31); let i15 = sx(w & 0x7FFF, 15); let i20 = sx(w & 0xFFFFF, 20);
    let mut flow = Flow::Normal; let mut target = None; let mut known = true;
    let mut br = |imm: i64, flow_: &mut Flow, t: &mut Option<i64>, cond: bool| -> String { let tg = pc + imm * 2; *t = Some(tg); *flow_ = if cond { Flow::Jcc } else { Flow::Jmp }; format!("{tg:#x}") };
    let text = match op {
        0..=3 | 16..=18 => { let n = ["lbi", "lhi", "lwi", "ldi"]; let ns = ["lbsi", "lhsi", "lwsi"]; let (nm, sh) = if op < 4 { (n[op as usize], op) } else { (ns[(op - 16) as usize], op - 16) }; format!("{nm} {}, [{}+{}]", r(rt), r(ra), sgn(i15 << sh)) }
        4..=7 | 20..=22 => { let n = ["lbi.bi", "lhi.bi", "lwi.bi", "ldi.bi"]; let ns = ["lbsi.bi", "lhsi.bi", "lwsi.bi"]; let (nm, sh) = if op < 8 { (n[(op - 4) as usize], op - 4) } else { (ns[(op - 20) as usize], op - 20) }; format!("{nm} {}, [{}], {}", r(rt), r(ra), sgn(i15 << sh)) }
        8..=11 => { let n = ["sbi", "shi", "swi", "sdi"]; format!("{} {}, [{}+{}]", n[(op - 8) as usize], r(rt), r(ra), sgn(i15 << (op - 8))) }
        12..=15 => { let n = ["sbi.bi", "shi.bi", "swi.bi", "sdi.bi"]; format!("{} {}, [{}], {}", n[(op - 12) as usize], r(rt), r(ra), sgn(i15 << (op - 12))) }
        19 => format!("dprefi {}, [{}+{}]", rt, r(ra), sgn(i15)), 23 => format!("lbi.gp {}, [gp+{}]", r(rt), sgn(sx(w & 0x7FFFF, 19))),
        24 => format!("lwc cp, [{}+{}]", r(ra), sgn(i15 << 2)), 25 => format!("swc cp, [{}+{}]", r(ra), sgn(i15 << 2)), 26 => format!("ldc cp, [{}+{}]", r(ra), sgn(i15 << 3)), 27 => format!("sdc cp, [{}+{}]", r(ra), sgn(i15 << 3)),
        28 => { let sub = w & 0xFF; let sv = (w >> 8) & 3; match mem_sub(sub) { Some(n) => format!("{n} {}, [{}+({}<<{sv})]", r(rt), r(ra), r(rb)), None => { known = false; format!("mem.{sub:#x} {}, {}, {}", r(rt), r(ra), r(rb)) } } }
        29 => { let en4 = (w >> 6) & 15; let mode = (w >> 2) & 7; let sub = w & 3; format!("{} {}, [{}], {}, {:#x} ; mode={}", if w & 0x20 != 0 { "smw" } else { "lmw" }, r(rt), r(ra), r(rb), en4, mode | (sub << 3)) }
        30 => { let sub = (w >> 18) & 3; format!("{} {}, [gp+{}]", ["lhi.gp", "lhsi.gp", "shi.gp", "hwgp3"][sub as usize], r(rt), sgn(sx(w & 0x3FFFF, 18) << 1)) }
        31 => { let sub = (w >> 19) & 3; format!("{} {}, [gp+{}]", ["sbi.gp", "addi.gp", "lbsi.gp", "sbgp3"][sub as usize], r(rt), sgn(sx(w & 0x7FFFF, 19))) }
        32 => { let sub = w & 31; let rd = (w >> 5) & 31; let n = ALU1[sub as usize]; match sub { 8..=11 => format!("{n} {}, {}, {}", r(rt), r(ra), rb), 16 | 17 | 19 | 20 => format!("{n} {}, {}", r(rt), r(ra)), _ => if rd != 0 || matches!(sub, 21 | 28..=31) { format!("{n} {}, {}, {}, {}", r(rt), r(ra), r(rb), rd) } else { format!("{n} {}, {}, {}", r(rt), r(ra), r(rb)) } } }
        33 => { let sub = w & 0x3FF; match alu2(sub & 0x3F) { Some(n) => if matches!(sub & 0x3F, 3 | 6 | 7) { format!("{n} {}, {}", r(rt), r(ra)) } else { format!("{n} {}, {}, {}", r(rt), r(ra), r(rb)) }, None => { known = false; format!("alu2.{:#x} {}, {}, {}", sub, r(rt), r(ra), r(rb)) } } }
        34 => format!("movi {}, {}", r(rt), sgn(i20)), 35 => format!("sethi {}, {:#x}", r(rt), w & 0xFFFFF),
        36 => { let imm = sx(w & 0xFFFFFF, 24); let call = w & 0x0100_0000 != 0; let tg = br(imm, &mut flow, &mut target, false); if call { flow = Flow::Call; } format!("{} {tg}", if call { "jal" } else { "j" }) }
        37 => { let sub = w & 31; match sub { 0 => { flow = Flow::Indirect; format!("jr {}", r(rb)) } 1 => { flow = Flow::Call; format!("jral {}, {}", r(rt), r(rb)) } 2 => { flow = Flow::Indirect; format!("jrnez {}", r(rb)) } 3 => { flow = Flow::Call; format!("jralnez {}, {}", r(rt), r(rb)) } _ => { known = false; format!("jreg.{sub} {}", r(rb)) } } }
        38 => { let imm = sx(w & 0x3FFF, 14); let tg = br(imm, &mut flow, &mut target, true); format!("{} {}, {}, {tg}", if w & 0x4000 != 0 { "bne" } else { "beq" }, r(rt), r(ra)) }
        39 => { let sub = (w >> 16) & 15; let imm = sx(w & 0xFFFF, 16); let tg = br(imm, &mut flow, &mut target, true); match sub { 0 => { flow = Flow::Call; format!("ifcall {tg}") } 2 => format!("beqz {}, {tg}", r(rt)), 3 => format!("bnez {}, {tg}", r(rt)), 4 => format!("bgez {}, {tg}", r(rt)), 5 => format!("bltz {}, {tg}", r(rt)), 6 => format!("bgtz {}, {tg}", r(rt)), 7 => format!("blez {}, {tg}", r(rt)), 12 => { flow = Flow::Call; format!("bgezal {}, {tg}", r(rt)) } 13 => { flow = Flow::Call; format!("bltzal {}, {tg}", r(rt)) } _ => { known = false; format!("br2.{sub} {}, {tg}", r(rt)) } } }
        40 => format!("addi {}, {}, {}", r(rt), r(ra), sgn(i15)), 41 => format!("subri {}, {}, {}", r(rt), r(ra), sgn(i15)), 42 => format!("andi {}, {}, {:#x}", r(rt), r(ra), w & 0x7FFF), 43 => format!("xori {}, {}, {:#x}", r(rt), r(ra), w & 0x7FFF), 44 => format!("ori {}, {}, {:#x}", r(rt), r(ra), w & 0x7FFF),
        45 => { let imm = sx(w & 0xFF, 8); let tg = br(imm, &mut flow, &mut target, true); format!("{} {}, {}, {tg}", if w & 0x80000 != 0 { "bnec" } else { "beqc" }, r(rt), sx((w >> 8) & 0x7FF, 11)) }
        46 => format!("slti {}, {}, {}", r(rt), r(ra), sgn(i15)), 47 => format!("sltsi {}, {}, {}", r(rt), r(ra), sgn(i15)),
        48 => { known = false; format!("aext {:#010x}", w) } 49 => { known = false; format!("cext {:#010x}", w) }
        50 => { let sub = w & 31; match misc(sub) { Some(n) => { if sub == 4 { flow = Flow::Ret; } format!("{n} {}, {}", r(rt), (w >> 5) & 0x7FFF) } None => { known = false; format!("misc.{sub} {:#010x}", w) } } }
        51 => format!("bitci {}, {}, {:#x}", r(rt), r(ra), w & 0x7FFF),
        _ => { known = false; format!(".word {w:#010x} ; op6={op}") }
    };
    Nds { len: 4, text, flow, target, known }
}

fn decode16(h: u32, pc: i64) -> Nds {
    let mut flow = Flow::Normal; let mut target = None; let mut known = true;
    let op10 = (h >> 5) & 0x3FF; let op8 = (h >> 7) & 0xFF; let op7 = (h >> 8) & 0x7F; let op6 = (h >> 9) & 0x3F; let op5 = (h >> 10) & 0x1F; let op4 = (h >> 11) & 0xF;
    let (r5a, r5t, r3t, r3a, r3b) = (h & 31, (h >> 5) & 31, (h >> 6) & 7, (h >> 3) & 7, h & 7); let rt4 = (h >> 5) & 15;
    let text = if op8 == 248 || op8 == 249 { let re = ["r6", "r8", "r10", "r14"][((h >> 5) & 3) as usize]; format!("{} {re}, {}", if op8 == 248 { "push25" } else { "pop25" }, (h & 31) << 3) }
    else if matches!(op10, 744 | 745 | 746 | 748 | 749 | 848) { match op10 { 744 => { flow = Flow::Indirect; format!("jr5 {}", r(r5a)) } 745 => { flow = Flow::Call; format!("jral5 {}", r(r5a)) } 746 => format!("ex9.it5 {}", r5a), 748 => { flow = Flow::Ret; format!("ret5 {}", r(r5a)) } 749 => format!("add5.pc {}", r(r5a)), _ => format!("break16 {}", r5a) } }
    else if matches!(op7, 85 | 104 | 105) { let imm = sx(h & 0xFF, 8); let tg = pc + imm * 2; target = Some(tg); flow = if op7 == 85 { Flow::Jmp } else { Flow::Jcc }; match op7 { 85 => format!("j8 {tg:#x}"), 104 => format!("beqzs8 {tg:#x}"), _ => format!("bnezs8 {tg:#x}") } }
    else if matches!(op6, 53 | 60) { if op6 == 53 { format!("ex9.it {}", h & 0x1FF) } else { let tg = pc + (h & 0x1FF) as i64 * 2; target = Some(tg); flow = Flow::Call; format!("ifcall9 {tg:#x}") } }
    else if op5 == 27 { format!("addi10.sp {}", sgn(sx(h & 0x3FF, 10))) }
    else if matches!(op4, 8 | 9 | 10 | 11) { let imm = sx(h & 0xFF, 8); let tg = pc + imm * 2; target = Some(tg); flow = Flow::Jcc; let rt = (h >> 8) & 7; match op4 { 8 => format!("beqz38 {}, {tg:#x}", r(rt)), 9 => format!("bnez38 {}, {tg:#x}", r(rt)), 10 => format!("beqs38 {}, r5, {tg:#x}", r(rt)), _ => format!("bnes38 {}, r5, {tg:#x}", r(rt)) } }
    else if op4 == 7 || op4 == 14 { let rt = (h >> 8) & 7; let ls = (h >> 7) & 1; let imm = (h & 0x7F) << 2; if op4 == 7 { format!("{} {}, [+{}] ; xwi37", if ls == 0 { "lwi37" } else { "swi37" }, r(rt), imm) } else { format!("{} {}, [sp+{}]", if ls == 0 { "lwi37.sp" } else { "swi37.sp" }, r(rt), imm) } }
    else if op5 == 0 || op5 == 1 { if op5 == 0 { format!("mov55 {}, {}", r(r5t), r(r5a)) } else { format!("movi55 {}, {}", r(r5t), sgn(sx(r5a, 5))) } }
    else { match op6 {
        4 => format!("add45 {}, {}", r4(rt4), r(r5a)), 5 => format!("sub45 {}, {}", r4(rt4), r(r5a)), 6 => format!("addi45 {}, {}", r4(rt4), r5a), 7 => format!("subi45 {}, {}", r4(rt4), r5a), 8 => format!("srai45 {}, {}", r4(rt4), r5a), 9 => format!("srli45 {}, {}", r4(rt4), r5a),
        25 => format!("lwi45.fe {}, {}", r4(rt4), r5a), 26 => format!("lwi450 {}, [{}]", r4(rt4), r(r5a)), 27 => format!("swi450 {}, [{}]", r4(rt4), r(r5a)),
        48 => format!("slts45 {}, {}", r4(rt4), r(r5a)), 49 => format!("slt45 {}, {}", r4(rt4), r(r5a)), 50 => format!("sltsi45 {}, {}", r4(rt4), r5a), 51 => format!("slti45 {}, {}", r4(rt4), r5a), 61 => format!("movpi45 {}, {}", r4(rt4), r5a + 16),
        10 => format!("slli333 {}, {}, {}", r(r3t), r(r3a), r3b), 12 => format!("add333 {}, {}, {}", r(r3t), r(r3a), r(r3b)), 13 => format!("sub333 {}, {}, {}", r(r3t), r(r3a), r(r3b)), 14 => format!("addi333 {}, {}, {}", r(r3t), r(r3a), r3b), 15 => format!("subi333 {}, {}, {}", r(r3t), r(r3a), r3b),
        16 => format!("lwi333 {}, [{}+{}]", r(r3t), r(r3a), r3b << 2), 17 => format!("lwi333.bi {}, [{}], {}", r(r3t), r(r3a), r3b << 2), 18 => format!("lhi333 {}, [{}+{}]", r(r3t), r(r3a), r3b << 1), 19 => format!("lbi333 {}, [{}+{}]", r(r3t), r(r3a), r3b),
        20 => format!("swi333 {}, [{}+{}]", r(r3t), r(r3a), r3b << 2), 21 => format!("swi333.bi {}, [{}], {}", r(r3t), r(r3a), r3b << 2), 22 => format!("shi333 {}, [{}+{}]", r(r3t), r(r3a), r3b << 1), 23 => format!("sbi333 {}, [{}+{}]", r(r3t), r(r3a), r3b),
        11 => { let n = ["zeb33", "zeh33", "seb33", "seh33", "xlsb33", "x11b33", "bmski33", "fexti33"]; format!("{} {}, {}", n[r3b as usize], r(r3t), r(r3a)) }
        63 => { let n = ["", "", "neg33", "not33", "mul33", "xor33", "and33", "or33"]; if r3b >= 2 { format!("{} {}, {}", n[r3b as usize], r(r3t), r(r3a)) } else { known = false; format!("misc33.{r3b} {}, {}", r(r3t), r(r3a)) } }
        24 => format!("addri36.sp {}, {}", r(r3t), (h & 0x3F) << 2),
        _ => { known = false; format!(".half {h:#06x} ; op6={op6}") } } };
    Nds { len: 2, text, flow, target, known }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(b: &[u8]) -> Nds { decode(b, 0x1000).unwrap() }
    #[test] fn ret5_push25_pop25_gros_boutien() { assert_eq!(d(&[0xDD, 0x9E]).text, "ret5 lp"); assert_eq!(d(&[0xDD, 0x9E]).flow, Flow::Ret); assert_eq!(d(&[0xFC, 0x00]).text, "push25 r6, 0"); assert_eq!(d(&[0xFC, 0x80]).text, "pop25 r6, 0"); assert_eq!(d(&[0xDD, 0x9E]).len, 2); }
    #[test] fn longueurs_16_et_32() { assert_eq!(d(&[0x84, 0x00]).len, 2); assert_eq!(d(&[0x46, 0x10, 0x0F, 0x02]).len, 4); assert!(decode(&[0x46, 0x10, 0x0F], 0).is_none()); assert!(decode(&[0x84], 0).is_none()); }
    #[test] fn branches_relatives() { let i = decode(&[0xD5, 0x03], 0x2000).unwrap(); assert_eq!(i.text, "j8 0x2006"); assert_eq!(i.target, Some(0x2006)); let j = decode(&[0x48, 0x00, 0x00, 0x02], 0x3000).unwrap(); assert_eq!(j.text, "j 0x3004"); let jal = decode(&[0x49, 0x00, 0x00, 0x04], 0x3000).unwrap(); assert_eq!(jal.flow, Flow::Call); assert_eq!(jal.target, Some(0x3008)); }
    #[test] fn arithmetique_et_memoire() { assert_eq!(d(&[0x50, 0x00, 0x00, 0x10]).text, "addi r0, r0, 0x10"); assert!(d(&[0x04, 0x1F, 0x00, 0x04]).text.starts_with("lwi ")); }
}
