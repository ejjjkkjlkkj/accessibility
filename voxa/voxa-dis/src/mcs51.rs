//! Desassembleur 8051 complet (255 opcodes definis + A5 reserve). Sert au micrologiciel du controleur embarque (ITE, noyau 8051).
use super::x86::Flow;

pub struct I51 { pub len: usize, pub text: String, pub flow: Flow, pub target: Option<u32> }

fn sfr(a: u8) -> String { let n = match a { 0x80 => "P0", 0x81 => "SP", 0x82 => "DPL", 0x83 => "DPH", 0x87 => "PCON", 0x88 => "TCON", 0x89 => "TMOD", 0x8A => "TL0", 0x8B => "TL1", 0x8C => "TH0", 0x8D => "TH1", 0x90 => "P1", 0x98 => "SCON", 0x99 => "SBUF", 0xA0 => "P2", 0xA8 => "IE", 0xB0 => "P3", 0xB8 => "IP", 0xD0 => "PSW", 0xE0 => "ACC", 0xF0 => "B", _ => "" };
    if !n.is_empty() { n.into() } else { format!("{a:#04x}") } }
fn bit(b: u8) -> String { if b < 0x80 { format!("{:#04x}.{}", 0x20 + b / 8, b % 8) } else { let base = b & 0xF8; let s = sfr(base); format!("{s}.{}", b & 7) } }
fn rn(op: u8) -> String { format!("R{}", op & 7) }

/// Decode a l'adresse `pc` ; retourne None si les octets manquent.
pub fn decode(code: &[u8], pc: u32) -> Option<I51> {
    let op = *code.first()?; let b1 = code.get(1).copied(); let b2 = code.get(2).copied();
    let need = |n: usize| code.len() >= n;
    let rel = |at: usize, len: usize| -> Option<u32> { let r = *code.get(at)? as i8 as i64; Some((pc as i64 + len as i64 + r) as u32 & 0xFFFF) };
    let ri = |op: u8| format!("@R{}", op & 1);
    let mut flow = Flow::Normal; let mut target = None; let len;
    let text = match op {
        0x00 => { len = 1; "NOP".into() }
        _ if op & 0x1F == 0x01 || op & 0x1F == 0x11 => { if !need(2) { return None; } len = 2; let a = ((pc + 2) & 0xF800) | ((op as u32 >> 5) << 8) | b1? as u32; target = Some(a); let call = op & 0x10 != 0; flow = if call { Flow::Call } else { Flow::Jmp }; format!("{} {a:#06x}", if call { "ACALL" } else { "AJMP" }) }
        0x02 | 0x12 => { if !need(3) { return None; } len = 3; let a = (b1? as u32) << 8 | b2? as u32; target = Some(a); flow = if op == 0x12 { Flow::Call } else { Flow::Jmp }; format!("{} {a:#06x}", if op == 0x12 { "LCALL" } else { "LJMP" }) }
        0x22 => { len = 1; flow = Flow::Ret; "RET".into() } 0x32 => { len = 1; flow = Flow::Ret; "RETI".into() }
        0x03 => { len = 1; "RR A".into() } 0x13 => { len = 1; "RRC A".into() } 0x23 => { len = 1; "RL A".into() } 0x33 => { len = 1; "RLC A".into() }
        0x10 | 0x20 | 0x30 => { if !need(3) { return None; } len = 3; let t = rel(2, 3)?; target = Some(t); flow = Flow::Jcc; format!("{} {}, {t:#06x}", ["JBC", "JB", "JNB"][(op >> 4) as usize - 1], bit(b1?)) }
        0x40 | 0x50 | 0x60 | 0x70 => { if !need(2) { return None; } len = 2; let t = rel(1, 2)?; target = Some(t); flow = Flow::Jcc; format!("{} {t:#06x}", ["JC", "JNC", "JZ", "JNZ"][(op >> 4) as usize - 4]) }
        0x80 => { if !need(2) { return None; } len = 2; let t = rel(1, 2)?; target = Some(t); flow = Flow::Jmp; format!("SJMP {t:#06x}") }
        0x73 => { len = 1; flow = Flow::Indirect; "JMP @A+DPTR".into() }
        0x04 | 0x14 => { len = 1; format!("{} A", if op == 4 { "INC" } else { "DEC" }) }
        0x05 | 0x15 => { if !need(2) { return None; } len = 2; format!("{} {}", if op == 5 { "INC" } else { "DEC" }, sfr(b1?)) }
        0x06 | 0x07 | 0x16 | 0x17 => { len = 1; format!("{} {}", if op < 0x10 { "INC" } else { "DEC" }, ri(op)) }
        0x08..=0x0F | 0x18..=0x1F => { len = 1; format!("{} {}", if op < 0x10 { "INC" } else { "DEC" }, rn(op)) }
        0x24 | 0x34 | 0x44 | 0x54 | 0x64 | 0x94 => { if !need(2) { return None; } len = 2; format!("{} A, #{:#04x}", ["ADD", "ADDC", "ORL", "ANL", "XRL", "", "", "SUBB"][((op >> 4) as usize) - 2], b1?) }
        0x25 | 0x35 | 0x45 | 0x55 | 0x65 | 0x95 => { if !need(2) { return None; } len = 2; format!("{} A, {}", ["ADD", "ADDC", "ORL", "ANL", "XRL", "", "", "SUBB"][((op >> 4) as usize) - 2], sfr(b1?)) }
        0x26 | 0x27 | 0x36 | 0x37 | 0x46 | 0x47 | 0x56 | 0x57 | 0x66 | 0x67 | 0x96 | 0x97 => { len = 1; format!("{} A, {}", ["ADD", "ADDC", "ORL", "ANL", "XRL", "", "", "SUBB"][((op >> 4) as usize) - 2], ri(op)) }
        0x28..=0x2F | 0x38..=0x3F | 0x48..=0x4F | 0x58..=0x5F | 0x68..=0x6F | 0x98..=0x9F => { len = 1; format!("{} A, {}", ["ADD", "ADDC", "ORL", "ANL", "XRL", "", "", "SUBB"][((op >> 4) as usize) - 2], rn(op)) }
        0x42 | 0x52 | 0x62 => { if !need(2) { return None; } len = 2; format!("{} {}, A", ["ORL", "ANL", "XRL"][((op >> 4) as usize) - 4], sfr(b1?)) }
        0x43 | 0x53 | 0x63 => { if !need(3) { return None; } len = 3; format!("{} {}, #{:#04x}", ["ORL", "ANL", "XRL"][((op >> 4) as usize) - 4], sfr(b1?), b2?) }
        0x72 | 0xA0 | 0x82 | 0xB0 => { if !need(2) { return None; } len = 2; let (n, inv) = match op { 0x72 => ("ORL", ""), 0xA0 => ("ORL", "/"), 0x82 => ("ANL", ""), _ => ("ANL", "/") }; format!("{n} C, {inv}{}", bit(b1?)) }
        0x74 => { if !need(2) { return None; } len = 2; format!("MOV A, #{:#04x}", b1?) } 0x75 => { if !need(3) { return None; } len = 3; format!("MOV {}, #{:#04x}", sfr(b1?), b2?) }
        0x76 | 0x77 => { if !need(2) { return None; } len = 2; format!("MOV {}, #{:#04x}", ri(op), b1?) } 0x78..=0x7F => { if !need(2) { return None; } len = 2; format!("MOV {}, #{:#04x}", rn(op), b1?) }
        0x83 => { len = 1; "MOVC A, @A+PC".into() } 0x93 => { len = 1; "MOVC A, @A+DPTR".into() } 0x84 => { len = 1; "DIV AB".into() } 0xA4 => { len = 1; "MUL AB".into() }
        0x85 => { if !need(3) { return None; } len = 3; format!("MOV {}, {}", sfr(b2?), sfr(b1?)) } 0x86 | 0x87 => { if !need(2) { return None; } len = 2; format!("MOV {}, {}", sfr(b1?), ri(op)) }
        0x88..=0x8F => { if !need(2) { return None; } len = 2; format!("MOV {}, {}", sfr(b1?), rn(op)) }
        0x90 => { if !need(3) { return None; } len = 3; format!("MOV DPTR, #{:#06x}", (b1? as u32) << 8 | b2? as u32) }
        0x92 => { if !need(2) { return None; } len = 2; format!("MOV {}, C", bit(b1?)) } 0xA2 => { if !need(2) { return None; } len = 2; format!("MOV C, {}", bit(b1?)) } 0xA3 => { len = 1; "INC DPTR".into() }
        0xA5 => { len = 1; "RESERVED(A5)".into() }
        0xA6 | 0xA7 => { if !need(2) { return None; } len = 2; format!("MOV {}, {}", ri(op), sfr(b1?)) } 0xA8..=0xAF => { if !need(2) { return None; } len = 2; format!("MOV {}, {}", rn(op), sfr(b1?)) }
        0xB2 => { if !need(2) { return None; } len = 2; format!("CPL {}", bit(b1?)) } 0xB3 => { len = 1; "CPL C".into() }
        0xB4 | 0xB5 => { if !need(3) { return None; } len = 3; let t = rel(2, 3)?; target = Some(t); flow = Flow::Jcc; if op == 0xB4 { format!("CJNE A, #{:#04x}, {t:#06x}", b1?) } else { format!("CJNE A, {}, {t:#06x}", sfr(b1?)) } }
        0xB6 | 0xB7 => { if !need(3) { return None; } len = 3; let t = rel(2, 3)?; target = Some(t); flow = Flow::Jcc; format!("CJNE {}, #{:#04x}, {t:#06x}", ri(op), b1?) }
        0xB8..=0xBF => { if !need(3) { return None; } len = 3; let t = rel(2, 3)?; target = Some(t); flow = Flow::Jcc; format!("CJNE {}, #{:#04x}, {t:#06x}", rn(op), b1?) }
        0xC0 => { if !need(2) { return None; } len = 2; format!("PUSH {}", sfr(b1?)) } 0xD0 => { if !need(2) { return None; } len = 2; format!("POP {}", sfr(b1?)) }
        0xC2 => { if !need(2) { return None; } len = 2; format!("CLR {}", bit(b1?)) } 0xC3 => { len = 1; "CLR C".into() } 0xD2 => { if !need(2) { return None; } len = 2; format!("SETB {}", bit(b1?)) } 0xD3 => { len = 1; "SETB C".into() }
        0xC4 => { len = 1; "SWAP A".into() } 0xD4 => { len = 1; "DA A".into() }
        0xC5 => { if !need(2) { return None; } len = 2; format!("XCH A, {}", sfr(b1?)) } 0xC6 | 0xC7 => { len = 1; format!("XCH A, {}", ri(op)) } 0xC8..=0xCF => { len = 1; format!("XCH A, {}", rn(op)) }
        0xD5 => { if !need(3) { return None; } len = 3; let t = rel(2, 3)?; target = Some(t); flow = Flow::Jcc; format!("DJNZ {}, {t:#06x}", sfr(b1?)) } 0xD6 | 0xD7 => { len = 1; format!("XCHD A, {}", ri(op)) }
        0xD8..=0xDF => { if !need(2) { return None; } len = 2; let t = rel(1, 2)?; target = Some(t); flow = Flow::Jcc; format!("DJNZ {}, {t:#06x}", rn(op)) }
        0xE0 => { len = 1; "MOVX A, @DPTR".into() } 0xE2 | 0xE3 => { len = 1; format!("MOVX A, {}", ri(op)) } 0xE4 => { len = 1; "CLR A".into() }
        0xE5 => { if !need(2) { return None; } len = 2; format!("MOV A, {}", sfr(b1?)) } 0xE6 | 0xE7 => { len = 1; format!("MOV A, {}", ri(op)) } 0xE8..=0xEF => { len = 1; format!("MOV A, {}", rn(op)) }
        0xF0 => { len = 1; "MOVX @DPTR, A".into() } 0xF2 | 0xF3 => { len = 1; format!("MOVX {}, A", ri(op)) } 0xF4 => { len = 1; "CPL A".into() }
        0xF5 => { if !need(2) { return None; } len = 2; format!("MOV {}, A", sfr(b1?)) } 0xF6 | 0xF7 => { len = 1; format!("MOV {}, A", ri(op)) } 0xF8..=0xFF => { len = 1; format!("MOV {}, A", rn(op)) }
        _ => return None,
    };
    Some(I51 { len, text, flow, target })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn jeu_complet_sans_trou() { let mut inconnus = 0; for op in 0..=255u8 { let code = [op, 0x12, 0x34, 0x56]; if decode(&code, 0x100).is_none() { inconnus += 1; } } assert_eq!(inconnus, 0, "tous les opcodes 8051 (A5 inclus comme reserve) doivent etre decodes"); }
    #[test] fn exemples() { let t = |h: &[u8]| decode(h, 0x100).unwrap(); assert_eq!(t(&[0x02, 0x12, 0x34]).text, "LJMP 0x1234"); assert_eq!(t(&[0x75, 0x90, 0xff]).text, "MOV P1, #0xff"); assert_eq!(t(&[0x12, 0x00, 0x10]).text, "LCALL 0x0010"); assert_eq!(t(&[0x80, 0xfe]).target, Some(0x100)); assert_eq!(t(&[0x22]).text, "RET"); assert_eq!(t(&[0xd2, 0x90]).text, "SETB P1.0"); assert_eq!(t(&[0x85, 0x30, 0x40]).text, "MOV 0x40, 0x30"); }
    #[test] fn longueurs_connues() { for (op, l) in [(0x00u8, 1usize), (0x02, 3), (0x74, 2), (0x75, 3), (0x85, 3), (0x90, 3), (0xB4, 3), (0xD5, 3), (0xE5, 2), (0xF0, 1), (0x11, 2), (0x01, 2)] { let code = [op, 0, 0, 0]; assert_eq!(decode(&code, 0).unwrap().len, l, "{op:#x}"); } }
}
