//! Desassembleur x86-64 / x86-32 maison (syntaxe Intel). Ecrit pour les modules UEFI : jeu d'instructions entier complet, SSE/SSE2/SSE3/SSSE3/SSE4, AVX (VEX) et AVX-512 (EVEX) en LONGUEUR exacte avec noms pour les cas courants.
//! `exact` = le mnemonic est connu ; sinon l'instruction est decodee en longueur seulement (nom generique).
use std::fmt::Write as _;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Flow { Normal, Call, Jmp, Jcc, Ret, Stop, Indirect }
pub struct Insn { pub len: usize, pub text: String, pub flow: Flow, pub target: Option<u64>, pub rip_ref: Option<u64>, pub exact: bool }

const R64: [&str; 16] = ["rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"];
const R32: [&str; 16] = ["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi", "r8d", "r9d", "r10d", "r11d", "r12d", "r13d", "r14d", "r15d"];
const R16: [&str; 16] = ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di", "r8w", "r9w", "r10w", "r11w", "r12w", "r13w", "r14w", "r15w"];
const R8: [&str; 16] = ["al", "cl", "dl", "bl", "spl", "bpl", "sil", "dil", "r8b", "r9b", "r10b", "r11b", "r12b", "r13b", "r14b", "r15b"];
const R8L: [&str; 8] = ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh"];
const SEG: [&str; 8] = ["es", "cs", "ss", "ds", "fs", "gs", "?", "?"];
const CC: [&str; 16] = ["o", "no", "b", "ae", "e", "ne", "be", "a", "s", "ns", "p", "np", "l", "ge", "le", "g"];
const ALU: [&str; 8] = ["add", "or", "adc", "sbb", "and", "sub", "xor", "cmp"];
const SHIFT: [&str; 8] = ["rol", "ror", "rcl", "rcr", "shl", "shr", "shl", "sar"];

struct D<'a> { b: &'a [u8], p: usize, addr: u64, m64: bool, rex: u8, opsz: u32, adsz64: bool, seg: &'static str, rep: u8, p66: bool, lock: bool, rip: Option<u64>, ok: bool, exact: bool, vex_l: u8, vex_v: u8, vex: bool }
type M = (u8, u8, u8, Option<String>); // mod, reg (avec REX.R), rm (avec REX.B), memoire

impl<'a> D<'a> {
    fn u8(&mut self) -> u8 { match self.b.get(self.p) { Some(&x) => { self.p += 1; x } None => { self.ok = false; 0 } } }
    fn i8(&mut self) -> i64 { self.u8() as i8 as i64 }
    fn u16(&mut self) -> u64 { let a = self.u8() as u64; let b = self.u8() as u64; a | b << 8 }
    fn i32(&mut self) -> i64 { let mut v = 0u32; for k in 0..4 { v |= (self.u8() as u32) << (8 * k); } v as i32 as i64 }
    fn u64(&mut self) -> u64 { let mut v = 0u64; for k in 0..8 { v |= (self.u8() as u64) << (8 * k); } v }
    fn rexw(&self) -> bool { self.rex & 8 != 0 }
    fn reg(&self, n: u8, size: u32) -> String { let n = n as usize & 15; match size { 64 => R64[n].into(), 32 => R32[n].into(), 16 => R16[n].into(), _ => if self.rex != 0 || n >= 8 { R8[n].into() } else { R8L[n & 7].into() } } }
    fn xmm(&self, n: u8) -> String { format!("{}{}", if self.vex_l == 1 { "ymm" } else if self.vex_l == 2 { "zmm" } else { "xmm" }, n) }
    fn imm(&mut self, size: u32) -> String { match size { 8 => { let v = self.u8(); format!("{v:#x}") } 16 => { let v = self.u16(); format!("{v:#x}") } 64 => { let v = self.u64(); format!("{v:#x}") } _ => { let v = self.i32(); if v < 0 { format!("-{:#x}", -v) } else { format!("{v:#x}") } } } }
    fn ptr(size: u32) -> &'static str { match size { 8 => "byte ptr ", 16 => "word ptr ", 32 => "dword ptr ", 64 => "qword ptr ", 128 => "xmmword ptr ", 256 => "ymmword ptr ", 80 => "tbyte ptr ", _ => "" } }
    fn modrm(&mut self) -> M {
        let m = self.u8(); let md = m >> 6; let reg = ((m >> 3) & 7) | if self.rex & 4 != 0 { 8 } else { 0 }; let rm0 = m & 7; let rmx = rm0 | if self.rex & 1 != 0 { 8 } else { 0 };
        if md == 3 { return (md, reg, rmx, None); }
        let areg: &[&str; 16] = if self.adsz64 { &R64 } else { &R32 };
        let mut s = String::new(); if !self.seg.is_empty() { let _ = write!(s, "{}:", self.seg); } s.push('[');
        let (mut base, mut index, mut scale): (Option<&str>, Option<&str>, u8) = (None, None, 1); let mut disp: i64 = 0; let mut has_disp = false; let mut ripr = false;
        if rm0 == 4 { let sib = self.u8(); scale = 1 << (sib >> 6); let idx = ((sib >> 3) & 7) | if self.rex & 2 != 0 { 8 } else { 0 }; let bs = (sib & 7) | if self.rex & 1 != 0 { 8 } else { 0 };
            if idx != 4 { index = Some(areg[idx as usize]); }
            if (sib & 7) == 5 && md == 0 { disp = self.i32(); has_disp = true; } else { base = Some(areg[bs as usize]); } }
        else if rm0 == 5 && md == 0 { if self.m64 { ripr = true; disp = self.i32(); has_disp = true; } else { disp = self.i32(); has_disp = true; } }
        else { base = Some(areg[rmx as usize]); }
        if md == 1 { disp = self.i8(); has_disp = true; } else if md == 2 { disp = self.i32(); has_disp = true; }
        if ripr { s.push_str("rip"); }
        if let Some(b) = base { s.push_str(b); }
        if let Some(i) = index { if base.is_some() { s.push('+'); } s.push_str(i); if scale > 1 { let _ = write!(s, "*{scale}"); } }
        if has_disp && (disp != 0 || (base.is_none() && index.is_none() && !ripr) || ripr) { if disp < 0 { let _ = write!(s, "-{:#x}", -disp); } else { if base.is_some() || index.is_some() || ripr { s.push('+'); } let _ = write!(s, "{disp:#x}"); } }
        s.push(']'); if ripr { self.rip = Some(disp as u64); }
        (md, reg, rmx, Some(s))
    }
    fn rm_s(&self, m: &M, size: u32) -> String { match &m.3 { Some(s) => format!("{}{}", Self::ptr(size), s), None => self.reg(m.2, size) } }
    fn rm_x(&self, m: &M, size: u32) -> String { match &m.3 { Some(s) => format!("{}{}", Self::ptr(size), s), None => self.xmm(m.2) } }
}

fn sse_name(op: u8, pfx: u8) -> Option<&'static str> {
    // pfx : 0 aucun, 1 = 66, 2 = F3, 3 = F2
    let n = match (op, pfx) {
        (0x10, 0) => "movups", (0x10, 1) => "movupd", (0x10, 2) => "movss", (0x10, 3) => "movsd", (0x11, 0) => "movups", (0x11, 1) => "movupd", (0x11, 2) => "movss", (0x11, 3) => "movsd",
        (0x12, 0) => "movlps", (0x12, 1) => "movlpd", (0x12, 2) => "movsldup", (0x12, 3) => "movddup", (0x13, 0) => "movlps", (0x13, 1) => "movlpd", (0x14, 0) => "unpcklps", (0x14, 1) => "unpcklpd", (0x15, 0) => "unpckhps", (0x15, 1) => "unpckhpd",
        (0x16, 0) => "movhps", (0x16, 1) => "movhpd", (0x16, 2) => "movshdup", (0x17, 0) => "movhps", (0x17, 1) => "movhpd", (0x28, 0) => "movaps", (0x28, 1) => "movapd", (0x29, 0) => "movaps", (0x29, 1) => "movapd",
        (0x2A, 0) => "cvtpi2ps", (0x2A, 1) => "cvtpi2pd", (0x2A, 2) => "cvtsi2ss", (0x2A, 3) => "cvtsi2sd", (0x2B, 0) => "movntps", (0x2B, 1) => "movntpd", (0x2C, 2) => "cvttss2si", (0x2C, 3) => "cvttsd2si", (0x2D, 2) => "cvtss2si", (0x2D, 3) => "cvtsd2si",
        (0x2E, 0) => "ucomiss", (0x2E, 1) => "ucomisd", (0x2F, 0) => "comiss", (0x2F, 1) => "comisd", (0x50, 0) => "movmskps", (0x50, 1) => "movmskpd",
        (0x51, 0) => "sqrtps", (0x51, 1) => "sqrtpd", (0x51, 2) => "sqrtss", (0x51, 3) => "sqrtsd", (0x52, 0) => "rsqrtps", (0x52, 2) => "rsqrtss", (0x53, 0) => "rcpps", (0x53, 2) => "rcpss",
        (0x54, 0) => "andps", (0x54, 1) => "andpd", (0x55, 0) => "andnps", (0x55, 1) => "andnpd", (0x56, 0) => "orps", (0x56, 1) => "orpd", (0x57, 0) => "xorps", (0x57, 1) => "xorpd",
        (0x58, 0) => "addps", (0x58, 1) => "addpd", (0x58, 2) => "addss", (0x58, 3) => "addsd", (0x59, 0) => "mulps", (0x59, 1) => "mulpd", (0x59, 2) => "mulss", (0x59, 3) => "mulsd",
        (0x5A, 0) => "cvtps2pd", (0x5A, 1) => "cvtpd2ps", (0x5A, 2) => "cvtss2sd", (0x5A, 3) => "cvtsd2ss", (0x5B, 0) => "cvtdq2ps", (0x5B, 1) => "cvtps2dq", (0x5B, 2) => "cvttps2dq",
        (0x5C, 0) => "subps", (0x5C, 1) => "subpd", (0x5C, 2) => "subss", (0x5C, 3) => "subsd", (0x5D, 0) => "minps", (0x5D, 1) => "minpd", (0x5D, 2) => "minss", (0x5D, 3) => "minsd",
        (0x5E, 0) => "divps", (0x5E, 1) => "divpd", (0x5E, 2) => "divss", (0x5E, 3) => "divsd", (0x5F, 0) => "maxps", (0x5F, 1) => "maxpd", (0x5F, 2) => "maxss", (0x5F, 3) => "maxsd",
        (0x60, _) => "punpcklbw", (0x61, _) => "punpcklwd", (0x62, _) => "punpckldq", (0x63, _) => "packsswb", (0x64, _) => "pcmpgtb", (0x65, _) => "pcmpgtw", (0x66, _) => "pcmpgtd", (0x67, _) => "packuswb",
        (0x68, _) => "punpckhbw", (0x69, _) => "punpckhwd", (0x6A, _) => "punpckhdq", (0x6B, _) => "packssdw", (0x6C, 1) => "punpcklqdq", (0x6D, 1) => "punpckhqdq",
        (0x6F, 1) => "movdqa", (0x6F, 2) => "movdqu", (0x6F, _) => "movq", (0x7F, 1) => "movdqa", (0x7F, 2) => "movdqu", (0x7F, _) => "movq",
        (0x70, 1) => "pshufd", (0x70, 2) => "pshufhw", (0x70, 3) => "pshuflw", (0x70, 0) => "pshufw", (0x74, _) => "pcmpeqb", (0x75, _) => "pcmpeqw", (0x76, _) => "pcmpeqd", (0x77, _) => "emms",
        (0x7C, 1) => "haddpd", (0x7C, 3) => "haddps", (0x7D, 1) => "hsubpd", (0x7D, 3) => "hsubps", (0xC2, 0) => "cmpps", (0xC2, 1) => "cmppd", (0xC2, 2) => "cmpss", (0xC2, 3) => "cmpsd", (0xC6, 0) => "shufps", (0xC6, 1) => "shufpd",
        (0xD0, 1) => "addsubpd", (0xD0, 3) => "addsubps", (0xD1, _) => "psrlw", (0xD2, _) => "psrld", (0xD3, _) => "psrlq", (0xD4, _) => "paddq", (0xD5, _) => "pmullw", (0xD6, 1) => "movq", (0xD7, _) => "pmovmskb",
        (0xD8, _) => "psubusb", (0xD9, _) => "psubusw", (0xDA, _) => "pminub", (0xDB, _) => "pand", (0xDC, _) => "paddusb", (0xDD, _) => "paddusw", (0xDE, _) => "pmaxub", (0xDF, _) => "pandn",
        (0xE0, _) => "pavgb", (0xE1, _) => "psraw", (0xE2, _) => "psrad", (0xE3, _) => "pavgw", (0xE4, _) => "pmulhuw", (0xE5, _) => "pmulhw", (0xE6, 1) => "cvttpd2dq", (0xE6, 2) => "cvtdq2pd", (0xE6, 3) => "cvtpd2dq", (0xE7, 1) => "movntdq", (0xE7, _) => "movntq",
        (0xE8, _) => "psubsb", (0xE9, _) => "psubsw", (0xEA, _) => "pminsw", (0xEB, _) => "por", (0xEC, _) => "paddsb", (0xED, _) => "paddsw", (0xEE, _) => "pmaxsw", (0xEF, _) => "pxor",
        (0xF0, 3) => "lddqu", (0xF1, _) => "psllw", (0xF2, _) => "pslld", (0xF3, _) => "psllq", (0xF4, _) => "pmuludq", (0xF5, _) => "pmaddwd", (0xF6, _) => "psadbw", (0xF7, 1) => "maskmovdqu", (0xF7, _) => "maskmovq",
        (0xF8, _) => "psubb", (0xF9, _) => "psubw", (0xFA, _) => "psubd", (0xFB, _) => "psubq", (0xFC, _) => "paddb", (0xFD, _) => "paddw", (0xFE, _) => "paddd", _ => return None };
    Some(n)
}

/// Decode une instruction a `addr`. `m64` = mode 64 bits ; sinon mode protege 32 bits. Retourne None si les octets ne forment pas une instruction valide.
pub fn decode(code: &[u8], addr: u64, m64: bool) -> Option<Insn> {
    let mut d = D { b: code, p: 0, addr, m64, rex: 0, opsz: 32, adsz64: m64, seg: "", rep: 0, p66: false, lock: false, rip: None, ok: true, exact: true, vex_l: 0, vex_v: 0, vex: false };
    let mut pfx = 0; let mut flow = Flow::Normal; let mut target: Option<u64> = None;
    loop { let c = *code.get(d.p)?; match c { 0x66 => { d.p66 = true; } 0x67 => { d.adsz64 = !m64; } 0xF2 | 0xF3 => { d.rep = c; } 0xF0 => { d.lock = true; }
            0x26 => d.seg = "es", 0x2E => d.seg = "cs", 0x36 => d.seg = "ss", 0x3E => d.seg = "ds", 0x64 => d.seg = "fs", 0x65 => d.seg = "gs", _ => break } d.p += 1; if d.p > 14 { return None; } }
    if m64 { if let Some(&c) = code.get(d.p) { if c & 0xF0 == 0x40 { d.rex = c; d.p += 1; } } }
    d.opsz = if d.rexw() { 64 } else if d.p66 { 16 } else { 32 };
    let _ = &mut pfx; pfx = if d.p66 { 1 } else if d.rep == 0xF3 { 2 } else if d.rep == 0xF2 { 3 } else { 0 };
    let op = d.u8(); let osz = d.opsz; let mut t = String::new();
    macro_rules! fin { ($s:expr) => {{ t = $s; }} }
    match op {
        0x00..=0x3F if op & 7 < 6 && !matches!(op, 0x0F) => { let name = ALU[(op >> 3) as usize & 7]; match op & 7 {
            0 => { let m = d.modrm(); fin!(format!("{name} {}, {}", d.rm_s(&m, 8), d.reg(m.1, 8))) } 1 => { let m = d.modrm(); fin!(format!("{name} {}, {}", d.rm_s(&m, osz), d.reg(m.1, osz))) }
            2 => { let m = d.modrm(); fin!(format!("{name} {}, {}", d.reg(m.1, 8), d.rm_s(&m, 8))) } 3 => { let m = d.modrm(); fin!(format!("{name} {}, {}", d.reg(m.1, osz), d.rm_s(&m, osz))) }
            4 => { let i = d.imm(8); fin!(format!("{name} al, {i}")) } _ => { let i = d.imm(if osz == 16 { 16 } else { 32 }); fin!(format!("{name} {}, {i}", d.reg(0, osz))) } } }
        0x0F => return decode_0f(d, pfx),
        0x06 | 0x07 | 0x0E | 0x16 | 0x17 | 0x1E | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F if m64 => return None,
        0x06 => fin!("push es".into()), 0x07 => fin!("pop es".into()), 0x0E => fin!("push cs".into()), 0x16 => fin!("push ss".into()), 0x17 => fin!("pop ss".into()), 0x1E => fin!("push ds".into()), 0x1F => fin!("pop ds".into()),
        0x27 => fin!("daa".into()), 0x2F => fin!("das".into()), 0x37 => fin!("aaa".into()), 0x3F => fin!("aas".into()),
        0x40..=0x4F if !m64 => { fin!(format!("{} {}", if op < 0x48 { "inc" } else { "dec" }, d.reg(op & 7, osz))) }
        0x50..=0x57 => fin!(format!("push {}", d.reg((op & 7) | if d.rex & 1 != 0 { 8 } else { 0 }, if m64 { if d.p66 { 16 } else { 64 } } else { osz }))),
        0x58..=0x5F => fin!(format!("pop {}", d.reg((op & 7) | if d.rex & 1 != 0 { 8 } else { 0 }, if m64 { if d.p66 { 16 } else { 64 } } else { osz }))),
        0x60 | 0x61 | 0x62 if !m64 => { if op == 0x62 { let m = d.modrm(); fin!(format!("bound {}, {}", d.reg(m.1, osz), d.rm_s(&m, osz))) } else { fin!(if op == 0x60 { "pushad".into() } else { "popad".into() }) } }
        0x62 if m64 => return decode_evex(d),
        0x63 => { let m = d.modrm(); if m64 { fin!(format!("movsxd {}, {}", d.reg(m.1, osz), d.rm_s(&m, 32))) } else { fin!(format!("arpl {}, {}", d.rm_s(&m, 16), d.reg(m.1, 16))) } }
        0x68 => { let i = d.imm(if osz == 16 { 16 } else { 32 }); fin!(format!("push {i}")) } 0x6A => { let i = d.imm(8); fin!(format!("push {i}")) }
        0x69 => { let m = d.modrm(); let i = d.imm(if osz == 16 { 16 } else { 32 }); fin!(format!("imul {}, {}, {i}", d.reg(m.1, osz), d.rm_s(&m, osz))) }
        0x6B => { let m = d.modrm(); let i = d.imm(8); fin!(format!("imul {}, {}, {i}", d.reg(m.1, osz), d.rm_s(&m, osz))) }
        0x6C => fin!("insb".into()), 0x6D => fin!("insd".into()), 0x6E => fin!("outsb".into()), 0x6F => fin!("outsd".into()),
        0x70..=0x7F => { let r = d.i8(); let tg = (addr as i64 + d.p as i64 + r) as u64; target = Some(tg); flow = Flow::Jcc; fin!(format!("j{} {tg:#x}", CC[(op & 15) as usize])) }
        0x80 | 0x81 | 0x82 | 0x83 => { let m = d.modrm(); let sz = if op & 1 == 0 { 8 } else { osz }; let i = d.imm(if op == 0x81 { if osz == 16 { 16 } else { 32 } } else { 8 }); fin!(format!("{} {}, {i}", ALU[(m.1 & 7) as usize], d.rm_s(&m, sz))) }
        0x84 => { let m = d.modrm(); fin!(format!("test {}, {}", d.rm_s(&m, 8), d.reg(m.1, 8))) } 0x85 => { let m = d.modrm(); fin!(format!("test {}, {}", d.rm_s(&m, osz), d.reg(m.1, osz))) }
        0x86 => { let m = d.modrm(); fin!(format!("xchg {}, {}", d.rm_s(&m, 8), d.reg(m.1, 8))) } 0x87 => { let m = d.modrm(); fin!(format!("xchg {}, {}", d.rm_s(&m, osz), d.reg(m.1, osz))) }
        0x88 => { let m = d.modrm(); fin!(format!("mov {}, {}", d.rm_s(&m, 8), d.reg(m.1, 8))) } 0x89 => { let m = d.modrm(); fin!(format!("mov {}, {}", d.rm_s(&m, osz), d.reg(m.1, osz))) }
        0x8A => { let m = d.modrm(); fin!(format!("mov {}, {}", d.reg(m.1, 8), d.rm_s(&m, 8))) } 0x8B => { let m = d.modrm(); fin!(format!("mov {}, {}", d.reg(m.1, osz), d.rm_s(&m, osz))) }
        0x8C => { let m = d.modrm(); fin!(format!("mov {}, {}", d.rm_s(&m, if m.3.is_some() { 16 } else { osz }), SEG[(m.1 & 7) as usize])) } 0x8E => { let m = d.modrm(); fin!(format!("mov {}, {}", SEG[(m.1 & 7) as usize], d.rm_s(&m, 16))) }
        0x8D => { let m = d.modrm(); if m.3.is_none() { return None; } fin!(format!("lea {}, {}", d.reg(m.1, osz), m.3.clone().unwrap())) }
        0x8F => { let m = d.modrm(); fin!(format!("pop {}", d.rm_s(&m, if m64 { 64 } else { osz }))) }
        0x90 => { if d.rep == 0xF3 { fin!("pause".into()) } else if d.rex & 1 != 0 { fin!(format!("xchg r8, {}", d.reg(0, osz))) } else { fin!("nop".into()) } }
        0x91..=0x97 => fin!(format!("xchg {}, {}", d.reg((op & 7) | if d.rex & 1 != 0 { 8 } else { 0 }, osz), d.reg(0, osz))),
        0x98 => fin!(match osz { 64 => "cdqe", 32 => "cwde", _ => "cbw" }.into()), 0x99 => fin!(match osz { 64 => "cqo", 32 => "cdq", _ => "cwd" }.into()),
        0x9A if !m64 => { let o = if osz == 16 { d.u16() } else { d.i32() as u64 }; let s = d.u16(); fin!(format!("call far {s:#x}:{o:#x}")) }
        0x9B => fin!("fwait".into()), 0x9C => fin!(if m64 { "pushfq".into() } else { "pushfd".into() }), 0x9D => fin!(if m64 { "popfq".into() } else { "popfd".into() }), 0x9E => fin!("sahf".into()), 0x9F => fin!("lahf".into()),
        0xA0..=0xA3 => { let a = if m64 && d.adsz64 { d.u64() } else { d.i32() as u64 }; let (r, sz) = if op & 1 == 0 { (d.reg(0, 8), 8) } else { (d.reg(0, osz), osz) }; fin!(if op < 0xA2 { format!("mov {r}, {}[{a:#x}]", D::ptr(sz)) } else { format!("mov {}[{a:#x}], {r}", D::ptr(sz)) }) }
        0xA4 => fin!(rep_s(d.rep, "movsb")), 0xA5 => fin!(rep_s(d.rep, match osz { 64 => "movsq", 32 => "movsd", _ => "movsw" })), 0xA6 => fin!(repe_s(d.rep, "cmpsb")), 0xA7 => fin!(repe_s(d.rep, match osz { 64 => "cmpsq", 32 => "cmpsd", _ => "cmpsw" })),
        0xA8 => { let i = d.imm(8); fin!(format!("test al, {i}")) } 0xA9 => { let i = d.imm(if osz == 16 { 16 } else { 32 }); fin!(format!("test {}, {i}", d.reg(0, osz))) }
        0xAA => fin!(rep_s(d.rep, "stosb")), 0xAB => fin!(rep_s(d.rep, match osz { 64 => "stosq", 32 => "stosd", _ => "stosw" })), 0xAC => fin!(rep_s(d.rep, "lodsb")), 0xAD => fin!(rep_s(d.rep, match osz { 64 => "lodsq", 32 => "lodsd", _ => "lodsw" })),
        0xAE => fin!(repe_s(d.rep, "scasb")), 0xAF => fin!(repe_s(d.rep, match osz { 64 => "scasq", 32 => "scasd", _ => "scasw" })),
        0xB0..=0xB7 => { let i = d.imm(8); fin!(format!("mov {}, {i}", d.reg((op & 7) | if d.rex & 1 != 0 { 8 } else { 0 }, 8))) }
        0xB8..=0xBF => { let i = d.imm(match osz { 64 => 64, 16 => 16, _ => 32 }); fin!(format!("mov {}, {i}", d.reg((op & 7) | if d.rex & 1 != 0 { 8 } else { 0 }, osz))) }
        0xC0 | 0xC1 | 0xD0 | 0xD1 | 0xD2 | 0xD3 => { let m = d.modrm(); let sz = if op & 1 == 0 { 8 } else { osz }; let c = match op { 0xC0 | 0xC1 => d.imm(8), 0xD0 | 0xD1 => "1".into(), _ => "cl".into() }; fin!(format!("{} {}, {c}", SHIFT[(m.1 & 7) as usize], d.rm_s(&m, sz))) }
        0xC2 => { let i = d.u16(); flow = Flow::Ret; fin!(format!("ret {i:#x}")) } 0xC3 => { flow = Flow::Ret; fin!(rep_s(d.rep, "ret")) }
        0xC4 | 0xC5 if !m64 && code.get(d.p).map_or(false, |&b| b & 0xC0 != 0xC0) => { let m = d.modrm(); fin!(format!("{} {}, {}", if op == 0xC4 { "les" } else { "lds" }, d.reg(m.1, osz), d.rm_s(&m, osz))) }
        0xC4 | 0xC5 => return decode_vex(d, op),
        0xC6 | 0xC7 => { let m = d.modrm(); if m.1 & 7 != 0 { if op == 0xC6 && m.1 & 7 == 7 && m.3.is_none() && code.get(d.p).is_some() { let i = d.imm(8); fin!(format!("xabort {i}")) } else { return None; } } else { let sz = if op == 0xC6 { 8 } else { osz }; let i = d.imm(if op == 0xC6 { 8 } else if osz == 16 { 16 } else { 32 }); fin!(format!("mov {}, {i}", d.rm_s(&m, sz))) } }
        0xC8 => { let a = d.u16(); let b = d.u8(); fin!(format!("enter {a:#x}, {b:#x}")) } 0xC9 => fin!("leave".into()), 0xCA => { let i = d.u16(); flow = Flow::Ret; fin!(format!("retf {i:#x}")) } 0xCB => { flow = Flow::Ret; fin!(if osz == 64 { "retfq".into() } else { "retf".into() }) }
        0xCC => { flow = Flow::Stop; fin!("int3".into()) } 0xCD => { let i = d.imm(8); fin!(format!("int {i}")) } 0xCE if !m64 => fin!("into".into()), 0xCF => { flow = Flow::Ret; fin!(if osz == 64 { "iretq".into() } else { "iretd".into() }) }
        0xD4 | 0xD5 if !m64 => { let i = d.imm(8); fin!(format!("{} {i}", if op == 0xD4 { "aam" } else { "aad" })) } 0xD6 if !m64 => fin!("salc".into()), 0xD7 => fin!("xlat".into()),
        0xD8..=0xDF => { let m = d.modrm(); match x87(op, m.0, m.1, m.2, m.3.as_deref()) { Some(s) => fin!(s), None => return None } }
        0xE0..=0xE3 => { let r = d.i8(); let tg = (addr as i64 + d.p as i64 + r) as u64; target = Some(tg); flow = Flow::Jcc; fin!(format!("{} {tg:#x}", ["loopne", "loope", "loop", if m64 { "jrcxz" } else { "jecxz" }][(op & 3) as usize])) }
        0xE4 => { let i = d.imm(8); fin!(format!("in al, {i}")) } 0xE5 => { let i = d.imm(8); fin!(format!("in {}, {i}", if osz == 16 { "ax" } else { "eax" })) } 0xE6 => { let i = d.imm(8); fin!(format!("out {i}, al")) } 0xE7 => { let i = d.imm(8); fin!(format!("out {i}, {}", if osz == 16 { "ax" } else { "eax" })) }
        0xE8 | 0xE9 => { let r = d.i32(); let tg = (addr as i64 + d.p as i64 + r) as u64; target = Some(tg); flow = if op == 0xE8 { Flow::Call } else { Flow::Jmp }; fin!(format!("{} {tg:#x}", if op == 0xE8 { "call" } else { "jmp" })) }
        0xEA if !m64 => { let o = d.i32() as u64; let s = d.u16(); flow = Flow::Jmp; fin!(format!("jmp far {s:#x}:{o:#x}")) }
        0xEB => { let r = d.i8(); let tg = (addr as i64 + d.p as i64 + r) as u64; target = Some(tg); flow = Flow::Jmp; fin!(format!("jmp {tg:#x}")) }
        0xEC => fin!("in al, dx".into()), 0xED => fin!(format!("in {}, dx", if osz == 16 { "ax" } else { "eax" })), 0xEE => fin!("out dx, al".into()), 0xEF => fin!(format!("out dx, {}", if osz == 16 { "ax" } else { "eax" })),
        0xF1 => fin!("int1".into()), 0xF4 => { flow = Flow::Stop; fin!("hlt".into()) } 0xF5 => fin!("cmc".into()),
        0xF6 | 0xF7 => { let m = d.modrm(); let sz = if op == 0xF6 { 8 } else { osz }; let r = m.1 & 7; if r < 2 { let i = d.imm(if op == 0xF6 { 8 } else if osz == 16 { 16 } else { 32 }); fin!(format!("test {}, {i}", d.rm_s(&m, sz))) } else { fin!(format!("{} {}", ["", "", "not", "neg", "mul", "imul", "div", "idiv"][r as usize], d.rm_s(&m, sz))) } }
        0xF8 => fin!("clc".into()), 0xF9 => fin!("stc".into()), 0xFA => fin!("cli".into()), 0xFB => fin!("sti".into()), 0xFC => fin!("cld".into()), 0xFD => fin!("std".into()),
        0xFE => { let m = d.modrm(); match m.1 & 7 { 0 => fin!(format!("inc {}", d.rm_s(&m, 8))), 1 => fin!(format!("dec {}", d.rm_s(&m, 8))), _ => return None } }
        0xFF => { let m = d.modrm(); match m.1 & 7 { 0 => fin!(format!("inc {}", d.rm_s(&m, osz))), 1 => fin!(format!("dec {}", d.rm_s(&m, osz))), 2 => { flow = Flow::Call; fin!(format!("call {}", d.rm_s(&m, if m64 { 64 } else { osz }))) }
            3 => fin!(format!("call far {}", d.rm_s(&m, osz))), 4 => { flow = Flow::Indirect; fin!(format!("jmp {}", d.rm_s(&m, if m64 { 64 } else { osz }))) } 5 => { flow = Flow::Indirect; fin!(format!("jmp far {}", d.rm_s(&m, osz))) } 6 => fin!(format!("push {}", d.rm_s(&m, if m64 { 64 } else { osz }))), _ => return None } }
        _ => return None,
    }
    if !d.ok { return None; }
    if d.lock { t = format!("lock {t}"); }
    let rip_ref = d.rip.map(|disp| (addr as i64 + d.p as i64 + disp as i64) as u64);
    Some(Insn { len: d.p, text: t, flow, target, rip_ref, exact: d.exact })
}
fn repe_s(rep: u8, s: &str) -> String { match rep { 0xF3 => format!("repe {s}"), 0xF2 => format!("repne {s}"), _ => s.into() } }
fn rep_s(rep: u8, s: &str) -> String { match rep { 0xF3 => format!("rep {s}"), 0xF2 => format!("repne {s}"), _ => s.into() } }

fn decode_0f(mut d: D, pfx: u8) -> Option<Insn> {
    let op = d.u8(); let osz = d.opsz; let addr = d.addr; let mut flow = Flow::Normal; let mut target = None; let mut t: String;
    macro_rules! xmm_rm { ($name:expr) => {{ let m = d.modrm(); format!("{} {}, {}", $name, d.xmm(m.1), d.rm_x(&m, 128)) }} }
    match op {
        0x00 => { let m = d.modrm(); t = format!("{} {}", ["sldt", "str", "lldt", "ltr", "verr", "verw", "?", "?"][(m.1 & 7) as usize], d.rm_s(&m, 16)); }
        0x01 => { let m = d.modrm(); if m.3.is_none() { let m2 = 0xC0 | ((m.1 & 7) << 3) | (m.2 & 7); match sys0f01(m2) { Some(n) => t = n.into(), None => return None } } else { t = format!("{} {}", ["sgdt", "sidt", "lgdt", "lidt", "smsw", "?", "lmsw", "invlpg"][(m.1 & 7) as usize], d.rm_s(&m, if (m.1 & 7) < 4 { 80 } else { 64 })); } }
        0x02 | 0x03 => { let m = d.modrm(); t = format!("{} {}, {}", if op == 2 { "lar" } else { "lsl" }, d.reg(m.1, osz), d.rm_s(&m, 16)); }
        0x05 => t = "syscall".into(), 0x06 => t = "clts".into(), 0x07 => t = "sysret".into(), 0x08 => t = "invd".into(), 0x09 => t = "wbinvd".into(), 0x0B => { flow = Flow::Stop; t = "ud2".into(); }
        0x0D => { let m = d.modrm(); t = format!("prefetchw {}", d.rm_s(&m, 0)); }
        0x10..=0x17 | 0x28 | 0x29 | 0x2A | 0x2B | 0x2C | 0x2D | 0x2E | 0x2F | 0x50..=0x6F | 0x74..=0x76 | 0x7C | 0x7D | 0x7E | 0x7F | 0xC2 | 0xC6 | 0xD0..=0xFE if op != 0x70 && !(op == 0x7E && pfx != 2 && pfx != 0 && false) => {
            let m = d.modrm(); let imm = if matches!(op, 0xC2 | 0xC6) { Some(d.imm(8)) } else { None };
            let (nm, ex) = match (op, pfx) { (0x7E, 2) => ("movq", true), (0x7E, _) => (if d.rexw() { "movq" } else { "movd" }, true), (0x6E, _) => (if d.rexw() { "movq" } else { "movd" }, true), (0x12, 0) if m.3.is_none() => ("movhlps", true), (0x16, 0) if m.3.is_none() => ("movlhps", true), _ => match sse_name(op, pfx) { Some(n) => (n, true), None => ("sse", false) } };
            if !ex { d.exact = false; }
            let nm_s = if ex { nm.to_string() } else { format!("sse.0f{op:02x}") };
            // operandes : instructions a registre general / memoire selon le cas
            let store = matches!(op, 0x11 | 0x13 | 0x17 | 0x29 | 0x2B | 0x7F | 0xD6 | 0xE7);
            if op == 0x7E { t = format!("{nm_s} {}, {}", d.rm_s(&m, if d.rexw() { 64 } else { 32 }), d.xmm(m.1)); }
            else if op == 0x6E { t = format!("{nm_s} {}, {}", d.xmm(m.1), d.rm_s(&m, if d.rexw() { 64 } else { 32 })); }
            else if matches!(op, 0x2A) { t = format!("{nm_s} {}, {}", d.xmm(m.1), d.rm_s(&m, if d.rexw() { 64 } else { 32 })); }
            else if matches!(op, 0x2C | 0x2D) { t = format!("{nm_s} {}, {}", d.reg(m.1, if d.rexw() { 64 } else { 32 }), d.rm_x(&m, 128)); }
            else if matches!(op, 0x50 | 0xD7) { t = format!("{nm_s} {}, {}", d.reg(m.1, 32), d.xmm(m.2)); }
            else if store { let tm = if pfx == 3 && op == 0x11 { 64 } else if pfx == 2 && op == 0x11 { 32 } else { 128 }; t = format!("{nm_s} {}, {}", d.rm_x(&m, tm), d.xmm(m.1)); }
            else { let tm = if pfx == 3 && matches!(op, 0x10 | 0x51 | 0x58 | 0x59 | 0x5C | 0x5D | 0x5E | 0x5F | 0xC2) { 64 } else if pfx == 2 && matches!(op, 0x10 | 0x51 | 0x52 | 0x53 | 0x58 | 0x59 | 0x5C | 0x5D | 0x5E | 0x5F | 0xC2) { 32 } else if matches!(op, 0x2E | 0x2F) { if pfx == 1 { 64 } else { 32 } } else { 128 }; t = format!("{nm_s} {}, {}", d.xmm(m.1), d.rm_x(&m, tm)); }
            if let Some(i) = imm { t = format!("{t}, {i}"); }
        }
        0x70 => { let m = d.modrm(); let i = d.imm(8); t = format!("{} {}, {}, {i}", sse_name(0x70, pfx).unwrap_or("pshuf"), d.xmm(m.1), d.rm_x(&m, 128)); }
        0x71..=0x73 => { let m = d.modrm(); let i = d.imm(8); let nm = match (op, m.1 & 7) { (0x71, 2) => "psrlw", (0x71, 4) => "psraw", (0x71, 6) => "psllw", (0x72, 2) => "psrld", (0x72, 4) => "psrad", (0x72, 6) => "pslld", (0x73, 2) => "psrlq", (0x73, 3) => "psrldq", (0x73, 6) => "psllq", (0x73, 7) => "pslldq", _ => return None }; t = format!("{nm} {}, {i}", d.xmm(m.2)); }
        0x77 => t = "emms".into(), 0x30 => t = "wrmsr".into(), 0x31 => t = "rdtsc".into(), 0x32 => t = "rdmsr".into(), 0x33 => t = "rdpmc".into(), 0x34 => t = "sysenter".into(), 0x35 => t = "sysexit".into(),
        0x18 => { let m = d.modrm(); t = format!("{} {}", ["prefetchnta", "prefetcht0", "prefetcht1", "prefetcht2", "nop", "nop", "nop", "nop"][(m.1 & 7) as usize], d.rm_s(&m, 8)); }
        0x19..=0x1F => { let m = d.modrm(); t = format!("nop {}", d.rm_s(&m, osz)); if op == 0x1E && d.rep == 0xF3 && m.3.is_none() && m.1 & 7 == 1 { t = format!("rdssp{} {}", if d.rexw() { "q" } else { "d" }, d.reg(m.2, osz)); } else if op == 0x1E && d.rep == 0xF3 && m.3.is_none() { t = match m.1 & 7 { 7 if m.2 & 7 == 2 => "endbr64".into(), 7 if m.2 & 7 == 3 => "endbr32".into(), _ => t } ; } }
        0x20..=0x23 => { let m = d.modrm(); t = if op & 2 == 0 { format!("mov {}, {}{}", d.reg(m.2, 64), if op & 1 == 0 { "cr" } else { "dr" }, m.1) } else { format!("mov {}{}, {}", if op & 1 == 0 { "cr" } else { "dr" }, m.1, d.reg(m.2, 64)) }; }
        0x38 => { let o2 = d.u8(); let m = d.modrm(); d.exact = true; let nm = match o2 { 0x00 => "pshufb", 0x01 => "phaddw", 0x02 => "phaddd", 0x04 => "pmaddubsw", 0x08 => "psignb", 0x0B => "pmulhrsw", 0x10 => "pblendvb", 0x17 => "ptest", 0x1C => "pabsb", 0x1D => "pabsw", 0x1E => "pabsd", 0x20 => "pmovsxbw", 0x28 => "pmuldq", 0x29 => "pcmpeqq", 0x2B => "packusdw", 0x30 => "pmovzxbw", 0x37 => "pcmpgtq", 0x38 => "pminsb", 0x39 => "pminsd", 0x3A => "pminuw", 0x3B => "pminud", 0x3C => "pmaxsb", 0x3D => "pmaxsd", 0x3E => "pmaxuw", 0x3F => "pmaxud", 0x40 => "pmulld", 0xDB => "aesimc", 0xDC => "aesenc", 0xDD => "aesenclast", 0xDE => "aesdec", 0xDF => "aesdeclast",
            0xF0 | 0xF1 if pfx == 3 => "crc32", 0xF0 => "movbe", 0xF1 => "movbe", 0xC8 => "sha1nexte", 0xC9 => "sha1msg1", 0xCA => "sha1msg2", 0xCB => "sha256rnds2", 0xCC => "sha256msg1", 0xCD => "sha256msg2", 0xF6 => "adcx", _ => { d.exact = false; "ssse3.0f38" } };
            if o2 == 0xF0 || o2 == 0xF1 { t = if pfx == 3 { format!("crc32 {}, {}", d.reg(m.1, if d.rexw() { 64 } else { 32 }), d.rm_s(&m, if o2 == 0xF0 { 8 } else { osz })) } else if o2 == 0xF0 { format!("movbe {}, {}", d.reg(m.1, osz), d.rm_s(&m, osz)) } else { format!("movbe {}, {}", d.rm_s(&m, osz), d.reg(m.1, osz)) }; }
            else { t = format!("{nm} {}, {}", d.xmm(m.1), d.rm_x(&m, 128)); } }
        0x3A => { let o2 = d.u8(); let m = d.modrm(); let i = d.imm(8); d.exact = true; let nm = match o2 { 0x08 => "roundps", 0x09 => "roundpd", 0x0A => "roundss", 0x0B => "roundsd", 0x0C => "blendps", 0x0D => "blendpd", 0x0E => "pblendw", 0x0F => "palignr", 0x14 => "pextrb", 0x15 => "pextrw", 0x16 => "pextrd", 0x17 => "extractps", 0x20 => "pinsrb", 0x21 => "insertps", 0x22 => "pinsrd", 0x40 => "dpps", 0x41 => "dppd", 0x42 => "mpsadbw", 0x44 => "pclmulqdq", 0x60 => "pcmpestrm", 0x61 => "pcmpestri", 0x62 => "pcmpistrm", 0x63 => "pcmpistri", 0xDF => "aeskeygenassist", 0xCC => "sha1rnds4", _ => { d.exact = false; "sse4.0f3a" } }; t = format!("{nm} {}, {}, {i}", d.xmm(m.1), d.rm_x(&m, 128)); }
        0x40..=0x4F => { let m = d.modrm(); t = format!("cmov{} {}, {}", CC[(op & 15) as usize], d.reg(m.1, osz), d.rm_s(&m, osz)); }
        0x80..=0x8F => { let r = d.i32(); let tg = (addr as i64 + d.p as i64 + r) as u64; target = Some(tg); flow = Flow::Jcc; t = format!("j{} {tg:#x}", CC[(op & 15) as usize]); }
        0x90..=0x9F => { let m = d.modrm(); t = format!("set{} {}", CC[(op & 15) as usize], d.rm_s(&m, 8)); }
        0xA0 => t = "push fs".into(), 0xA1 => t = "pop fs".into(), 0xA8 => t = "push gs".into(), 0xA9 => t = "pop gs".into(), 0xA2 => t = "cpuid".into(), 0xAA => t = "rsm".into(),
        0xA3 | 0xAB | 0xB3 | 0xBB => { let m = d.modrm(); t = format!("{} {}, {}", match op { 0xA3 => "bt", 0xAB => "bts", 0xB3 => "btr", _ => "btc" }, d.rm_s(&m, osz), d.reg(m.1, osz)); }
        0xA4 | 0xAC => { let m = d.modrm(); let i = d.imm(8); t = format!("{} {}, {}, {i}", if op == 0xA4 { "shld" } else { "shrd" }, d.rm_s(&m, osz), d.reg(m.1, osz)); }
        0xA5 | 0xAD => { let m = d.modrm(); t = format!("{} {}, {}, cl", if op == 0xA5 { "shld" } else { "shrd" }, d.rm_s(&m, osz), d.reg(m.1, osz)); }
        0xAE => { let m = d.modrm(); t = if m.3.is_none() { match m.1 & 7 { 5 if d.rep == 0xF3 => format!("incssp{} {}", if d.rexw() { "q" } else { "d" }, d.reg(m.2, osz)), 5 => "lfence".into(), 6 => "mfence".into(), 7 => "sfence".into(), 0 | 1 | 2 | 3 if d.rep == 0xF3 => format!("{} {}", ["rdfsbase", "rdgsbase", "wrfsbase", "wrgsbase"][(m.1 & 7) as usize], d.reg(m.2, osz)), _ => return None } } else { format!("{} {}", ["fxsave", "fxrstor", "ldmxcsr", "stmxcsr", "xsave", "xrstor", "xsaveopt", "clflush"][(m.1 & 7) as usize], d.rm_s(&m, if matches!(m.1 & 7, 2 | 3) { 32 } else { 8 })) }; }
        0xAF => { let m = d.modrm(); t = format!("imul {}, {}", d.reg(m.1, osz), d.rm_s(&m, osz)); }
        0xB0 | 0xB1 => { let m = d.modrm(); let sz = if op == 0xB0 { 8 } else { osz }; t = format!("cmpxchg {}, {}", d.rm_s(&m, sz), d.reg(m.1, sz)); }
        0xB6 | 0xB7 | 0xBE | 0xBF => { let m = d.modrm(); let src = if op & 1 == 0 { 8 } else { 16 }; t = format!("{} {}, {}", if op < 0xB8 { "movzx" } else { "movsx" }, d.reg(m.1, osz), d.rm_s(&m, src)); }
        0xB8 if d.rep == 0xF3 => { let m = d.modrm(); t = format!("popcnt {}, {}", d.reg(m.1, osz), d.rm_s(&m, osz)); }
        0xBA => { let m = d.modrm(); let i = d.imm(8); let nm = match m.1 & 7 { 4 => "bt", 5 => "bts", 6 => "btr", 7 => "btc", _ => return None }; t = format!("{nm} {}, {i}", d.rm_s(&m, osz)); }
        0xBC => { let m = d.modrm(); t = format!("{} {}, {}", if d.rep == 0xF3 { "tzcnt" } else { "bsf" }, d.reg(m.1, osz), d.rm_s(&m, osz)); }
        0xBD => { let m = d.modrm(); t = format!("{} {}, {}", if d.rep == 0xF3 { "lzcnt" } else { "bsr" }, d.reg(m.1, osz), d.rm_s(&m, osz)); }
        0xC0 | 0xC1 => { let m = d.modrm(); let sz = if op == 0xC0 { 8 } else { osz }; t = format!("xadd {}, {}", d.rm_s(&m, sz), d.reg(m.1, sz)); }
        0xC3 => { let m = d.modrm(); t = format!("movnti {}, {}", d.rm_s(&m, osz), d.reg(m.1, osz)); }
        0xC4 => { let m = d.modrm(); let i = d.imm(8); t = format!("pinsrw {}, {}, {i}", d.xmm(m.1), d.rm_s(&m, 32)); }
        0xC5 => { let m = d.modrm(); let i = d.imm(8); t = format!("pextrw {}, {}, {i}", d.reg(m.1, 32), d.xmm(m.2)); }
        0xC7 => { let m = d.modrm(); t = match (m.1 & 7, m.3.is_some()) { (1, true) => format!("{} {}", if d.rexw() { "cmpxchg16b" } else { "cmpxchg8b" }, d.rm_s(&m, 128)), (6, false) => format!("rdrand {}", d.reg(m.2, osz)), (7, false) => format!("rdseed {}", d.reg(m.2, osz)), _ => { d.exact = false; format!("grp9.{} {}", m.1 & 7, d.rm_s(&m, 64)) } }; }
        0xC8..=0xCF => { t = format!("bswap {}", d.reg((op & 7) | if d.rex & 1 != 0 { 8 } else { 0 }, osz)); }
        0xB9 => { let m = d.modrm(); t = format!("ud1 {}, {}", d.reg(m.1, osz), d.rm_s(&m, osz)); flow = Flow::Stop; }
        0xFF => { let m = d.modrm(); t = format!("ud0 {}, {}", d.reg(m.1, osz), d.rm_s(&m, osz)); flow = Flow::Stop; }
        _ => return None,
    }
    if !d.ok { return None; }
    let rip_ref = d.rip.map(|disp| (addr as i64 + d.p as i64 + disp as i64) as u64);
    Some(Insn { len: d.p, text: if d.lock { format!("lock {t}") } else { t }, flow, target, rip_ref, exact: d.exact })
}

fn decode_vex(mut d: D, first: u8) -> Option<Insn> {
    let addr = d.addr; d.vex = true; let (map, pp, l); let rexr; let mut rexw = false; let (mut rexx, mut rexb) = (false, false);
    if first == 0xC5 { let b = d.u8(); rexr = b & 0x80 == 0; d.vex_v = (!(b >> 3)) & 15; l = (b >> 2) & 1; pp = b & 3; map = 1; }
    else { let b1 = d.u8(); let b2 = d.u8(); rexr = b1 & 0x80 == 0; rexx = b1 & 0x40 == 0; rexb = b1 & 0x20 == 0; map = b1 & 0x1F; rexw = b2 & 0x80 != 0; d.vex_v = (!(b2 >> 3)) & 15; l = (b2 >> 2) & 1; pp = b2 & 3; }
    d.rex = 0x40 | if rexw { 8 } else { 0 } | if rexr { 4 } else { 0 } | if rexx { 2 } else { 0 } | if rexb { 1 } else { 0 }; d.vex_l = l;
    let op = d.u8(); let m = d.modrm(); let imm = (map == 3) || (map == 1 && matches!(op, 0x70..=0x73 | 0xC2 | 0xC4..=0xC6)); let i = if imm { Some(d.imm(8)) } else { None };
    let pfx = match pp { 0 => 0, 1 => 1, 2 => 2, _ => 3 };
    let nm = if map == 1 { sse_name(op, pfx).map(|n| format!("v{n}")) } else { None };
    d.exact = nm.is_some() || (map == 2 && matches!(op, 0x18 | 0x19 | 0x58 | 0x78 | 0x79 | 0x45 | 0x47 | 0xF7 | 0xF2 | 0xF3 | 0xF5 | 0xF6));
    let name = nm.unwrap_or_else(|| match (map, op) { (2, 0x18) => "vbroadcastss".into(), (2, 0x58) => "vpbroadcastd".into(), (2, 0xF7) => match pp { 0 => "bextr".into(), 1 => "shlx".into(), 2 => "sarx".into(), _ => "shrx".into() }, (2, 0xF2) => "andn".into(), (2, 0xF5) => match pp { 0 => "bzhi".into(), 2 => "pext".into(), _ => "pdep".into() }, (2, 0xF6) => "mulx".into(), _ => format!("vex.m{map}.{op:02x}") });
    let t = if map == 2 && matches!(op, 0xF7 | 0xF2 | 0xF5 | 0xF6) { format!("{name} {}, {}, {}", d.reg(m.1, if rexw { 64 } else { 32 }), d.rm_s(&m, if rexw { 64 } else { 32 }), d.reg(d.vex_v, if rexw { 64 } else { 32 })) }
        else { let mut s = format!("{name} {}", d.xmm(m.1)); if d.vex_v != 0 || true { let _ = write!(s, ", {}", d.xmm(d.vex_v)); } let _ = write!(s, ", {}", d.rm_x(&m, if l == 1 { 256 } else { 128 })); if let Some(i) = i { let _ = write!(s, ", {i}"); } s };
    if !d.ok { return None; }
    let rip_ref = d.rip.map(|disp| (addr as i64 + d.p as i64 + disp as i64) as u64);
    Some(Insn { len: d.p, text: t, flow: Flow::Normal, target: None, rip_ref, exact: d.exact })
}

fn decode_evex(mut d: D) -> Option<Insn> {
    let addr = d.addr; let p0 = d.u8(); let p1 = d.u8(); let p2 = d.u8(); let map = p0 & 7; let ll = (p2 >> 5) & 3; d.vex_l = if ll == 2 { 2 } else { ll.min(1) };
    d.rex = 0x40 | if p1 & 0x80 != 0 { 8 } else { 0 } | if p0 & 0x80 == 0 { 4 } else { 0 } | if p0 & 0x40 == 0 { 2 } else { 0 } | if p0 & 0x20 == 0 { 1 } else { 0 };
    let op = d.u8(); let before = d.p; let m = d.modrm();
    // disp8*N compresse : la longueur est inchangee ; seule la valeur affichee serait mise a l'echelle
    let _ = before; let imm = map == 3 || (map == 1 && matches!(op, 0x70..=0x73 | 0xC2 | 0xC4..=0xC6)); let i = if imm { Some(d.imm(8)) } else { None };
    d.exact = false; let mut t = format!("evex.m{map}.{op:02x} {}, {}", d.xmm(m.1), d.rm_x(&m, 512)); if let Some(i) = i { let _ = write!(t, ", {i}"); }
    if !d.ok { return None; }
    let rip_ref = d.rip.map(|disp| (addr as i64 + d.p as i64 + disp as i64) as u64);
    Some(Insn { len: d.p, text: t, flow: Flow::Normal, target: None, rip_ref, exact: false })
}

/// x87 complet : formes memoire et registre.
fn x87(op: u8, md: u8, reg: u8, rm: u8, mem: Option<&str>) -> Option<String> {
    let r = reg & 7; let i = rm & 7;
    if let Some(m) = mem {
        let (n, sz) = match (op, r) {
            (0xD8, _) => (["fadd", "fmul", "fcom", "fcomp", "fsub", "fsubr", "fdiv", "fdivr"][r as usize], "dword ptr "),
            (0xD9, 0) => ("fld", "dword ptr "), (0xD9, 2) => ("fst", "dword ptr "), (0xD9, 3) => ("fstp", "dword ptr "), (0xD9, 4) => ("fldenv", ""), (0xD9, 5) => ("fldcw", "word ptr "), (0xD9, 6) => ("fnstenv", ""), (0xD9, 7) => ("fnstcw", "word ptr "),
            (0xDA, _) => (["fiadd", "fimul", "ficom", "ficomp", "fisub", "fisubr", "fidiv", "fidivr"][r as usize], "dword ptr "),
            (0xDB, 0) => ("fild", "dword ptr "), (0xDB, 1) => ("fisttp", "dword ptr "), (0xDB, 2) => ("fist", "dword ptr "), (0xDB, 3) => ("fistp", "dword ptr "), (0xDB, 5) => ("fld", "tbyte ptr "), (0xDB, 7) => ("fstp", "tbyte ptr "),
            (0xDC, _) => (["fadd", "fmul", "fcom", "fcomp", "fsub", "fsubr", "fdiv", "fdivr"][r as usize], "qword ptr "),
            (0xDD, 0) => ("fld", "qword ptr "), (0xDD, 1) => ("fisttp", "qword ptr "), (0xDD, 2) => ("fst", "qword ptr "), (0xDD, 3) => ("fstp", "qword ptr "), (0xDD, 4) => ("frstor", ""), (0xDD, 6) => ("fnsave", ""), (0xDD, 7) => ("fnstsw", "word ptr "),
            (0xDE, _) => (["fiadd", "fimul", "ficom", "ficomp", "fisub", "fisubr", "fidiv", "fidivr"][r as usize], "word ptr "),
            (0xDF, 0) => ("fild", "word ptr "), (0xDF, 1) => ("fisttp", "word ptr "), (0xDF, 2) => ("fist", "word ptr "), (0xDF, 3) => ("fistp", "word ptr "), (0xDF, 4) => ("fbld", "tbyte ptr "), (0xDF, 5) => ("fild", "qword ptr "), (0xDF, 6) => ("fbstp", "tbyte ptr "), (0xDF, 7) => ("fistp", "qword ptr "),
            _ => return None };
        return Some(format!("{n} {sz}{m}")); }
    let _ = md;
    Some(match (op, r) {
        (0xD8, _) => format!("{} st, st({i})", ["fadd", "fmul", "fcom", "fcomp", "fsub", "fsubr", "fdiv", "fdivr"][r as usize]),
        (0xD9, 0) => format!("fld st({i})"), (0xD9, 1) => format!("fxch st({i})"), (0xD9, 2) if i == 0 => "fnop".into(),
        (0xD9, 4) => ["fchs", "fabs", "", "", "ftst", "fxam", "", ""][i as usize].to_string(), (0xD9, 5) => ["fld1", "fldl2t", "fldl2e", "fldpi", "fldlg2", "fldln2", "fldz", ""][i as usize].to_string(),
        (0xD9, 6) => ["f2xm1", "fyl2x", "fptan", "fpatan", "fxtract", "fprem1", "fdecstp", "fincstp"][i as usize].to_string(), (0xD9, 7) => ["fprem", "fyl2xp1", "fsqrt", "fsincos", "frndint", "fscale", "fsin", "fcos"][i as usize].to_string(),
        (0xDA, 0..=3) => format!("{} st, st({i})", ["fcmovb", "fcmove", "fcmovbe", "fcmovu"][r as usize]), (0xDA, 5) if i == 1 => "fucompp".into(),
        (0xDB, 0..=3) => format!("{} st, st({i})", ["fcmovnb", "fcmovne", "fcmovnbe", "fcmovnu"][r as usize]), (0xDB, 4) => match i { 2 => "fnclex".into(), 3 => "fninit".into(), _ => return None },
        (0xDB, 5) => format!("fucomi st, st({i})"), (0xDB, 6) => format!("fcomi st, st({i})"),
        (0xDC, _) => format!("{} st({i}), st", ["fadd", "fmul", "fcom", "fcomp", "fsubr", "fsub", "fdivr", "fdiv"][r as usize]),
        (0xDD, 0) => format!("ffree st({i})"), (0xDD, 2) => format!("fst st({i})"), (0xDD, 3) => format!("fstp st({i})"), (0xDD, 4) => format!("fucom st({i})"), (0xDD, 5) => format!("fucomp st({i})"),
        (0xDE, 0) => format!("faddp st({i}), st"), (0xDE, 1) => format!("fmulp st({i}), st"), (0xDE, 3) if i == 1 => "fcompp".into(), (0xDE, 4) => format!("fsubrp st({i}), st"), (0xDE, 5) => format!("fsubp st({i}), st"), (0xDE, 6) => format!("fdivrp st({i}), st"), (0xDE, 7) => format!("fdivp st({i}), st"),
        (0xDF, 4) if i == 0 => "fnstsw ax".into(), (0xDF, 5) => format!("fucomip st, st({i})"), (0xDF, 6) => format!("fcomip st, st({i})"),
        _ => return None })
}
fn sys0f01(m2: u8) -> Option<&'static str> { Some(match m2 { 0xC1 => "vmcall", 0xC2 => "vmlaunch", 0xC3 => "vmresume", 0xC4 => "vmxoff", 0xC8 => "monitor", 0xC9 => "mwait", 0xCA => "clac", 0xCB => "stac", 0xD0 => "xgetbv", 0xD1 => "xsetbv", 0xD4 => "vmfunc", 0xD5 => "xend", 0xD6 => "xtest", 0xEE => "rdpkru", 0xEF => "wrpkru", 0xF8 => "swapgs", 0xF9 => "rdtscp", 0xFA => "monitorx", 0xFB => "mwaitx", _ => return None }) }

#[cfg(test)]
mod tests {
    use super::*;
    fn d(hex: &str) -> (usize, String) { let b: Vec<u8> = (0..hex.len() / 2).map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap()).collect(); let i = decode(&b, 0x1000, true).expect(hex); (i.len, i.text) }
    #[test] fn instructions_courantes() {
        for (h, t) in [("4883ec28", "sub rsp, 0x28"), ("488b0510200000", "mov rax, qword ptr [rip+0x2010]"), ("e800000000", "call 0x1005"), ("0fb6c0", "movzx eax, al"), ("c3", "ret"), ("cc", "int3"), ("0f05", "syscall"), ("f30f1efa", "endbr64"), ("0f1f440000", "nop dword ptr [rax+rax]"),
            ("4157", "push r15"), ("4c8d0d00100000", "lea r9, [rip+0x1000]"), ("c5f9efc0", "vpxor xmm0, xmm0, xmm0"), ("ff2500000000", "jmp qword ptr [rip+0x0]"), ("48b88877665544332211", "mov rax, 0x1122334455667788"), ("7405", "je 0x1007"),
            ("31c0", "xor eax, eax"), ("85c0", "test eax, eax"), ("4889e5", "mov rbp, rsp"), ("488d4c2430", "lea rcx, [rsp+0x30]"), ("0fafc1", "imul eax, ecx"), ("f3a4", "rep movsb"), ("660f6ec0", "movd xmm0, eax"), ("f20f1000", "movsd xmm0, qword ptr [rax]"), ("d9ee", "fldz"), ("0f0b", "ud2"), ("48c7c0ffffffff", "mov rax, -0x1")] {
            let (len, txt) = d(h); assert_eq!(txt, t, "{h}"); assert_eq!(len * 2, h.len(), "longueur {h}"); } }
    #[test] fn longueurs_sse_avx_evex_exactes() { for (h, n) in [("0f28c1", 3), ("660fefc0", 4), ("c5fe6f0c10", 5), ("c4e27d1800", 5), ("62f17c4828c1", 6), ("0f3a0fc108", 5), ("0f3800c1", 4)] { let b: Vec<u8> = (0..h.len() / 2).map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap()).collect(); assert_eq!(decode(&b, 0, true).expect(h).len, n, "{h}"); } }
    #[test] fn invalides_en_64_bits() { for h in ["06", "07", "27", "60", "d6"] { let b = [u8::from_str_radix(h, 16).unwrap(), 0, 0, 0]; assert!(decode(&b, 0, true).is_none(), "{h}"); } }
    #[test] fn octets_tronques_ne_plantent_pas() { for n in 0..8 { let _ = decode(&[0x48, 0x8b, 0x05, 0x10, 0x20, 0x00, 0x00][..n.min(7)], 0, true); let _ = decode(&[0x0f, 0x3a][..n.min(2)], 0, true); } }
    #[test] fn mode_32_bits() { let b = [0x40u8]; let i = decode(&b, 0, false).unwrap(); assert_eq!(i.text, "inc eax"); let b = [0x55u8]; assert_eq!(decode(&b, 0, false).unwrap().text, "push ebp"); }
}
