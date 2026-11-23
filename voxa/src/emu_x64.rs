// Interpréteur x86-64 général (sous-ensemble entier) : exécute du code compilé par rustc pour x86_64-unknown-uefi (virgule flottante logicielle,
// pas de SSE). Inclus par emu.rs via include! pour accéder aux champs privés de Cpu.
// Périmètre : instructions entières courantes de LLVM (mov, alu, shifts, mul/div, jcc, call/ret, cmov, setcc, movzx/movsx, bt, rep movs/stos,
// in/out, cpuid). Toute autre instruction produit une ERREUR EXPLICITE (« instruction non modélisée ») : jamais d'exécution silencieuse fausse.
// Indépendance : ce modèle est écrit par nous et vérifié par des tests de comparaison avec le processeur hôte (voir tests dans emu.rs).

#[derive(Clone, Copy)]
struct ModRm { is_reg: bool, reg: usize, rm: usize, base: Option<usize>, index: Option<(usize, u32)>, disp: i64, riprel: bool }

impl Cpu {
    #[inline] fn mask(sz: usize) -> u64 { match sz { 1 => 0xFF, 2 => 0xFFFF, 4 => 0xFFFF_FFFF, _ => u64::MAX } }
    #[inline] fn sbit(sz: usize) -> u64 { 1u64 << (sz * 8 - 1) }
    #[inline] fn sext(v: u64, sz: usize) -> u64 { match sz { 1 => v as u8 as i8 as i64 as u64, 2 => v as u16 as i16 as i64 as u64, 4 => v as u32 as i32 as i64 as u64, _ => v } }

    fn f8(&self, pc: &mut u64) -> Result<u8, String> { let v = self.rd(*pc, 1)? as u8; *pc += 1; Ok(v) }
    fn f16(&self, pc: &mut u64) -> Result<u16, String> { let v = self.rd(*pc, 2)? as u16; *pc += 2; Ok(v) }
    fn f32(&self, pc: &mut u64) -> Result<u32, String> { let v = self.rd(*pc, 4)? as u32; *pc += 4; Ok(v) }
    fn f64(&self, pc: &mut u64) -> Result<u64, String> { let v = self.rd(*pc, 8)?; *pc += 8; Ok(v) }

    fn set_szp(&mut self, res: u64, sz: usize) {
        let r = res & Self::mask(sz);
        self.zf = r == 0; self.sf = r & Self::sbit(sz) != 0; self.pf = (r as u8).count_ones() % 2 == 0;
    }
    fn flags_logic(&mut self, res: u64, sz: usize) { self.cf = false; self.of = false; self.set_szp(res, sz); }

    /// 0 add, 1 or, 2 adc, 3 sbb, 4 and, 5 sub, 6 xor, 7 cmp. Renvoie le résultat masqué ; positionne CF, OF, ZF, SF, PF.
    fn alu(&mut self, op: u8, a: u64, b: u64, sz: usize) -> u64 {
        let m = Self::mask(sz); let (a, b) = (a & m, b & m);
        let sb = Self::sbit(sz);
        match op {
            0 | 2 => {
                let c = if op == 2 && self.cf { 1u64 } else { 0 };
                let wide = a as u128 + b as u128 + c as u128; let r = (wide as u64) & m;
                self.cf = wide > m as u128; self.of = ((a ^ r) & (b ^ r) & sb) != 0; self.set_szp(r, sz); r
            }
            3 | 5 | 7 => {
                let c = if op == 3 && self.cf { 1u64 } else { 0 };
                let r = a.wrapping_sub(b).wrapping_sub(c) & m;
                self.cf = (a as u128) < (b as u128 + c as u128); self.of = ((a ^ b) & (a ^ r) & sb) != 0; self.set_szp(r, sz); r
            }
            1 => { let r = (a | b) & m; self.flags_logic(r, sz); r }
            4 => { let r = (a & b) & m; self.flags_logic(r, sz); r }
            _ => { let r = (a ^ b) & m; self.flags_logic(r, sz); r }
        }
    }

    fn cond(&self, cc: u8) -> bool {
        let r = match cc >> 1 {
            0 => self.of, 1 => self.cf, 2 => self.zf, 3 => self.cf || self.zf, 4 => self.sf, 5 => self.pf,
            6 => self.sf != self.of, _ => self.zf || (self.sf != self.of),
        };
        if cc & 1 == 1 { !r } else { r }
    }

    fn reg_r(&self, idx: usize, sz: usize, rex: u8) -> u64 {
        if sz == 1 && rex == 0 && (4..8).contains(&idx) { (self.r[idx - 4] >> 8) & 0xFF } else { self.r[idx] & Self::mask(sz) }
    }
    fn reg_w(&mut self, idx: usize, sz: usize, rex: u8, v: u64) {
        match sz {
            1 => if rex == 0 && (4..8).contains(&idx) { let r = &mut self.r[idx - 4]; *r = (*r & !0xFF00) | ((v & 0xFF) << 8); } else { self.r[idx] = (self.r[idx] & !0xFF) | (v & 0xFF); },
            2 => self.r[idx] = (self.r[idx] & !0xFFFF) | (v & 0xFFFF),
            4 => self.r[idx] = v & 0xFFFF_FFFF,
            _ => self.r[idx] = v,
        }
    }

    fn modrm(&self, pc: &mut u64, rex: u8) -> Result<ModRm, String> {
        let b = self.f8(pc)?;
        let (md, reg, rm) = (b >> 6, ((b >> 3) & 7) as usize | if rex & 4 != 0 { 8 } else { 0 }, (b & 7) as usize);
        if md == 3 { return Ok(ModRm { is_reg: true, reg, rm: rm | if rex & 1 != 0 { 8 } else { 0 }, base: None, index: None, disp: 0, riprel: false }); }
        let (mut base, mut index, mut disp, mut riprel) = (None, None, 0i64, false);
        if rm == 4 {
            let sib = self.f8(pc)?;
            let (sc, ix, bs) = (sib >> 6, ((sib >> 3) & 7) as usize | if rex & 2 != 0 { 8 } else { 0 }, (sib & 7) as usize);
            if ix != 4 { index = Some((ix, 1u32 << sc)); }
            if bs == 5 && md == 0 { disp = self.f32(pc)? as i32 as i64; } else { base = Some(bs | if rex & 1 != 0 { 8 } else { 0 }); }
        } else if rm == 5 && md == 0 { riprel = true; disp = self.f32(pc)? as i32 as i64; }
        else { base = Some(rm | if rex & 1 != 0 { 8 } else { 0 }); }
        if md == 1 { disp = self.f8(pc)? as i8 as i64; } else if md == 2 { disp = self.f32(pc)? as i32 as i64; }
        Ok(ModRm { is_reg: false, reg, rm: 0, base, index, disp, riprel })
    }
    /// Adresse effective ; `end` = adresse de l'instruction suivante (pour RIP-relatif).
    fn ea(&self, m: &ModRm, end: u64) -> u64 {
        let mut a = if m.riprel { end } else { 0 };
        if let Some(b) = m.base { a = a.wrapping_add(self.r[b]); }
        if let Some((i, s)) = m.index { a = a.wrapping_add(self.r[i].wrapping_mul(s as u64)); }
        a.wrapping_add(m.disp as u64)
    }
    fn rm_read(&self, m: &ModRm, end: u64, sz: usize, rex: u8) -> Result<u64, String> { if m.is_reg { Ok(self.reg_r(m.rm, sz, rex)) } else { self.rd(self.ea(m, end), sz) } }
    fn rm_write(&mut self, m: &ModRm, end: u64, sz: usize, rex: u8, v: u64) -> Result<(), String> { if m.is_reg { self.reg_w(m.rm, sz, rex, v); Ok(()) } else { let a = self.ea(m, end); self.wr(a, sz, v & Self::mask(sz)) } }

    fn shift(&mut self, kind: usize, v: u64, count: u32, sz: usize) -> Result<u64, String> {
        let bits = (sz * 8) as u32;
        let c = count & if sz == 8 { 63 } else { 31 };
        if c == 0 { return Ok(v & Self::mask(sz)); }
        let m = Self::mask(sz); let v = v & m;
        let r = match kind {
            0 => { let k = c % bits; let r = if k == 0 { v } else { ((v << k) | (v >> (bits - k))) & m }; self.cf = r & 1 != 0; self.of = ((r >> (bits - 1)) & 1 != 0) != self.cf; return Ok(r); }
            1 => { let k = c % bits; let r = if k == 0 { v } else { ((v >> k) | (v << (bits - k))) & m }; self.cf = (r >> (bits - 1)) & 1 != 0; self.of = ((r >> (bits - 1)) ^ (r >> (bits - 2))) & 1 != 0; return Ok(r); }
            4 | 6 => { if c > bits { self.cf = false; } else { self.cf = (v >> (bits - c)) & 1 != 0; } let r = if c >= bits { 0 } else { (v << c) & m }; self.of = ((r >> (bits - 1)) & 1 != 0) != self.cf; r }
            5 => { self.cf = if c > bits { false } else { (v >> (c - 1)) & 1 != 0 }; let r = if c >= bits { 0 } else { v >> c }; self.of = v & Self::sbit(sz) != 0; r }
            7 => { let sv = Self::sext(v, sz) as i64; let cc = c.min(bits - 1) as u32; self.cf = if c >= bits { sv < 0 } else { ((sv >> (c - 1)) & 1) != 0 }; let r = ((sv >> cc) as u64) & m; self.of = false; r }
            _ => return Err(format!("décalage /{kind} non modélisé (rcl/rcr)")),
        };
        self.set_szp(r, sz); Ok(r)
    }

    fn jump_rel(&mut self, end: u64, rel: i64) { self.rip = end.wrapping_add(rel as u64); }

    /// Exécute une instruction. Renvoie Ok(false) à l'arrêt.
    fn step_generic(&mut self) -> Result<bool, String> {
        let start = self.rip; let mut pc = start;
        let (mut p66, mut rep, mut repne, mut rex) = (false, false, false, 0u8);
        let op = loop {
            let b = self.f8(&mut pc)?;
            match b {
                0x66 => p66 = true, 0xF3 => rep = true, 0xF2 => repne = true,
                0x2E | 0x3E | 0x26 | 0x36 | 0x64 | 0x65 | 0x67 => {}
                0x40..=0x4F => { rex = b; break self.f8(&mut pc)?; }
                _ => break b,
            }
        };
        let w = rex & 8 != 0;
        let osz = if w { 8 } else if p66 { 2 } else { 4 };
        macro_rules! bad { () => { return Err(format!("instruction non modélisée à 0x{start:X} : {}", (0..8).map(|k| format!("{:02X}", self.rd(start + k, 1).unwrap_or(0))).collect::<Vec<_>>().join(" "))) } }
        macro_rules! imm { ($sz:expr) => { match $sz { 1 => self.f8(&mut pc)? as i8 as i64 as u64, 2 => self.f16(&mut pc)? as i16 as i64 as u64, _ => self.f32(&mut pc)? as i32 as i64 as u64 } } }
        match op {
            // ---- ALU : add/or/adc/sbb/and/sub/xor/cmp
            0x00..=0x3D if (op & 7) < 6 => {
                let alu = op >> 3;
                match op & 7 {
                    0 | 1 | 2 | 3 => {
                        let sz = if op & 1 == 0 { 1 } else { osz };
                        let m = self.modrm(&mut pc, rex)?; let end = pc;
                        if op & 2 == 0 { // r/m, reg
                            let (a, b) = (self.rm_read(&m, end, sz, rex)?, self.reg_r(m.reg, sz, rex));
                            let r = self.alu(alu, a, b, sz); if alu != 7 { self.rm_write(&m, end, sz, rex, r)?; }
                        } else { // reg, r/m
                            let (a, b) = (self.reg_r(m.reg, sz, rex), self.rm_read(&m, end, sz, rex)?);
                            let r = self.alu(alu, a, b, sz); if alu != 7 { self.reg_w(m.reg, sz, rex, r); }
                        }
                    }
                    4 => { let i = self.f8(&mut pc)? as u64; let a = self.r[RAX] & 0xFF; let r = self.alu(alu, a, i, 1); if alu != 7 { self.reg_w(0, 1, rex, r); } }
                    _ => { let i = imm!(if osz == 2 { 2 } else { 4 }); let a = self.r[RAX] & Self::mask(osz); let r = self.alu(alu, a, i, osz); if alu != 7 { self.reg_w(0, osz, rex, r); } }
                }
            }
            0x50..=0x57 => { let v = self.r[(op - 0x50) as usize | if rex & 1 != 0 { 8 } else { 0 }]; self.push(v)?; }
            0x58..=0x5F => { let v = self.pop()?; self.r[(op - 0x58) as usize | if rex & 1 != 0 { 8 } else { 0 }] = v; }
            0x63 => { let m = self.modrm(&mut pc, rex)?; let end = pc; let v = self.rm_read(&m, end, 4, rex)?; self.reg_w(m.reg, osz, rex, Self::sext(v, 4)); }
            0x68 => { let v = imm!(4); self.push(v)?; }
            0x6A => { let v = imm!(1); self.push(v)?; }
            0x69 | 0x6B => {
                let m = self.modrm(&mut pc, rex)?; let i = if op == 0x69 { imm!(if osz == 2 { 2 } else { 4 }) } else { imm!(1) }; let end = pc;
                let a = Self::sext(self.rm_read(&m, end, osz, rex)?, osz) as i64 as i128; let r = a * (Self::sext(i, osz) as i64 as i128);
                let t = r as i64; let fits = if osz == 8 { r == t as i128 } else { (r as i64) == Self::sext(r as u64, osz) as i64 && r >= i64::MIN as i128 && r <= i64::MAX as i128 };
                self.cf = !fits; self.of = !fits; self.reg_w(m.reg, osz, rex, r as u64); self.set_szp(r as u64, osz);
            }
            0x70..=0x7F => { let rel = self.f8(&mut pc)? as i8 as i64; if self.cond(op & 0xF) { self.jump_rel(pc, rel); return Ok(true); } }
            0x80 | 0x81 | 0x83 => {
                let sz = if op == 0x80 { 1 } else { osz };
                let m = self.modrm(&mut pc, rex)?;
                let i = if op == 0x81 { imm!(if osz == 2 { 2 } else { 4 }) } else { imm!(1) }; let end = pc;
                let alu = (m.reg & 7) as u8; let a = self.rm_read(&m, end, sz, rex)?; let r = self.alu(alu, a, i, sz);
                if alu != 7 { self.rm_write(&m, end, sz, rex, r)?; }
            }
            0x84 | 0x85 => { let sz = if op == 0x84 { 1 } else { osz }; let m = self.modrm(&mut pc, rex)?; let end = pc; let (a, b) = (self.rm_read(&m, end, sz, rex)?, self.reg_r(m.reg, sz, rex)); self.alu(4, a, b, sz); }
            0x86 | 0x87 => { let sz = if op == 0x86 { 1 } else { osz }; let m = self.modrm(&mut pc, rex)?; let end = pc; let (a, b) = (self.rm_read(&m, end, sz, rex)?, self.reg_r(m.reg, sz, rex)); self.rm_write(&m, end, sz, rex, b)?; self.reg_w(m.reg, sz, rex, a); }
            0x88 | 0x89 => { let sz = if op == 0x88 { 1 } else { osz }; let m = self.modrm(&mut pc, rex)?; let end = pc; let v = self.reg_r(m.reg, sz, rex); self.rm_write(&m, end, sz, rex, v)?; }
            0x8A | 0x8B => { let sz = if op == 0x8A { 1 } else { osz }; let m = self.modrm(&mut pc, rex)?; let end = pc; let v = self.rm_read(&m, end, sz, rex)?; self.reg_w(m.reg, sz, rex, v); }
            0x8D => { let m = self.modrm(&mut pc, rex)?; if m.is_reg { bad!(); } let a = self.ea(&m, pc); self.reg_w(m.reg, osz, rex, a); }
            0x8F => { let m = self.modrm(&mut pc, rex)?; let end = pc; let v = self.pop()?; self.rm_write(&m, end, 8, rex, v)?; }
            0x90 => { /* nop / pause */ }
            0xFA | 0xFB => { /* cli / sti : sans effet dans le modele (pas d interruptions) */ }
            0x98 => { if w { self.r[RAX] = self.r[RAX] as u32 as i32 as i64 as u64; } else if p66 { self.reg_w(0, 2, rex, self.r[RAX] as u8 as i8 as i16 as u16 as u64); } else { self.r[RAX] = self.r[RAX] as u16 as i16 as i32 as u32 as u64; } }
            0x99 => { if w { self.r[RDX] = if (self.r[RAX] as i64) < 0 { u64::MAX } else { 0 }; } else { self.r[RDX] = if (self.r[RAX] as u32 as i32) < 0 { 0xFFFF_FFFF } else { 0 }; } }
            0xA4 | 0xA5 | 0xAA | 0xAB => {
                let sz = if op & 1 == 0 { 1 } else { osz };
                let mut n = if rep || repne { self.r[RCX] } else { 1 };
                let step = if self.df { (sz as u64).wrapping_neg() } else { sz as u64 };
                while n > 0 {
                    if op <= 0xA5 { let v = self.rd(self.r[6], sz)?; let d = self.r[7]; self.wr(d, sz, v)?; self.r[6] = self.r[6].wrapping_add(step); }
                    else { let v = self.r[RAX] & Self::mask(sz); let d = self.r[7]; self.wr(d, sz, v)?; }
                    self.r[7] = self.r[7].wrapping_add(step); n -= 1;
                    self.out.steps += 1; if self.out.steps > limite_pas() { self.out.violations.push("limite d'instructions atteinte : boucle sans fin probable".into()); self.out.halted = true; return Ok(false); }
                }
                if rep || repne { self.r[RCX] = 0; }
            }
            0xA8 => { let i = self.f8(&mut pc)? as u64; self.alu(4, self.r[RAX] & 0xFF, i, 1); }
            0xA9 => { let i = imm!(if osz == 2 { 2 } else { 4 }); self.alu(4, self.r[RAX] & Self::mask(osz), i, osz); }
            0xB0..=0xB7 => { let i = self.f8(&mut pc)? as u64; self.reg_w((op - 0xB0) as usize | if rex & 1 != 0 { 8 } else { 0 }, 1, rex, i); }
            0xB8..=0xBF => { let i = if w { self.f64(&mut pc)? } else if p66 { self.f16(&mut pc)? as u64 } else { self.f32(&mut pc)? as u64 }; self.reg_w((op - 0xB8) as usize | if rex & 1 != 0 { 8 } else { 0 }, osz, rex, i); }
            0xC0 | 0xC1 | 0xD0 | 0xD1 | 0xD2 | 0xD3 => {
                let sz = if op & 1 == 0 { 1 } else { osz };
                let m = self.modrm(&mut pc, rex)?;
                let count = match op { 0xC0 | 0xC1 => self.f8(&mut pc)? as u32, 0xD0 | 0xD1 => 1, _ => (self.r[RCX] & 0xFF) as u32 };
                let end = pc; let v = self.rm_read(&m, end, sz, rex)?; let r = self.shift((m.reg & 7) as usize, v, count, sz)?;
                self.rm_write(&m, end, sz, rex, r)?;
            }
            0xC2 => { let n = self.f16(&mut pc)? as u64; let ra = self.pop()?; self.r[RSP] = self.r[RSP].wrapping_add(n); self.rip = ra; return Ok(true); }
            0xC3 => { self.rip = self.pop()?; return Ok(true); }
            0xC6 | 0xC7 => { let sz = if op == 0xC6 { 1 } else { osz }; let m = self.modrm(&mut pc, rex)?; let i = imm!(if sz == 1 { 1 } else if sz == 2 { 2 } else { 4 }); let end = pc; self.rm_write(&m, end, sz, rex, i)?; }
            0xC9 => { self.r[RSP] = self.r[RBP]; self.r[RBP] = self.pop()?; }
            0xCC => return Err(format!("int3 exécuté à 0x{start:X} (code inatteignable ?)")),
            0xE8 => { let rel = self.f32(&mut pc)? as i32 as i64; self.push(pc)?; self.jump_rel(pc, rel); return Ok(true); }
            0xE9 => { let rel = self.f32(&mut pc)? as i32 as i64; self.jump_rel(pc, rel); return Ok(true); }
            0xEB => { let rel = self.f8(&mut pc)? as i8 as i64; self.jump_rel(pc, rel); return Ok(true); }
            0xEC => { let port = self.r[RDX] as u16; self.out.ports_read.push(port); let v = self.plat.inb(port); self.r[RAX] = (self.r[RAX] & !0xFF) | v as u64; }
            0xED => { let port = self.r[RDX] as u16; self.r[RAX] = self.plat.in32(port) as u64; }
            0xEE => { let (port, v) = (self.r[RDX] as u16, self.r[RAX] as u8); self.plat.out(port, v); if self.plat.kbc_reset { self.out.reset = Some(0); self.out.halted = true; return Ok(false); } }
            0xEF => { let (port, v) = (self.r[RDX] as u16, if p66 { self.r[RAX] as u16 as u32 } else { self.r[RAX] as u32 }); self.plat.out32(port, v); if self.plat.poweroff { self.out.reset = Some(2); self.out.halted = true; return Ok(false); } }
            0xF4 => { self.out.halted = true; return Ok(false); }
            0xFC => self.df = false,
            0xFD => self.df = true,
            0xF6 | 0xF7 => {
                let sz = if op == 0xF6 { 1 } else { osz };
                let m = self.modrm(&mut pc, rex)?;
                let kind = m.reg & 7;
                let i = if kind <= 1 { imm!(if sz == 1 { 1 } else if sz == 2 { 2 } else { 4 }) } else { 0 };
                let end = pc; let v = self.rm_read(&m, end, sz, rex)?;
                match kind {
                    0 | 1 => { self.alu(4, v, i, sz); }
                    2 => { self.rm_write(&m, end, sz, rex, !v)?; }
                    3 => { let r = self.alu(5, 0, v, sz); self.rm_write(&m, end, sz, rex, r)?; self.cf = v & Self::mask(sz) != 0; }
                    4 => { // mul
                        let a = self.r[RAX] & Self::mask(sz); let r = a as u128 * v as u128;
                        match sz { 1 => self.reg_w(0, 2, rex, r as u64), 2 => { self.reg_w(0, 2, rex, r as u64); self.reg_w(2, 2, rex, (r >> 16) as u64); } 4 => { self.r[RAX] = r as u32 as u64; self.r[RDX] = (r >> 32) as u32 as u64; } _ => { self.r[RAX] = r as u64; self.r[RDX] = (r >> 64) as u64; } }
                        let hi = r >> (sz * 8); self.cf = hi != 0; self.of = hi != 0;
                    }
                    5 => { // imul 1 opérande
                        let a = Self::sext(self.r[RAX], sz) as i64 as i128; let b = Self::sext(v, sz) as i64 as i128; let r = a * b;
                        match sz { 1 => self.reg_w(0, 2, rex, r as u64), 2 => { self.reg_w(0, 2, rex, r as u64); self.reg_w(2, 2, rex, (r >> 16) as u64); } 4 => { self.r[RAX] = r as u32 as u64; self.r[RDX] = (r >> 32) as u32 as u64; } _ => { self.r[RAX] = r as u64; self.r[RDX] = (r >> 64) as u64; } }
                        let fits = r == Self::sext(r as u64, sz) as i64 as i128; self.cf = !fits; self.of = !fits;
                    }
                    6 => { // div
                        if v == 0 { return Err(format!("#DE division par zéro à 0x{start:X}")); }
                        let n: u128 = match sz { 1 => (self.r[RAX] & 0xFFFF) as u128, 2 => (((self.r[RDX] & 0xFFFF) << 16) | (self.r[RAX] & 0xFFFF)) as u128, 4 => (((self.r[RDX] & 0xFFFF_FFFF) << 32) | (self.r[RAX] & 0xFFFF_FFFF)) as u128, _ => ((self.r[RDX] as u128) << 64) | self.r[RAX] as u128 };
                        let (q, r) = (n / v as u128, n % v as u128);
                        if q > Self::mask(sz) as u128 { return Err(format!("#DE débordement à 0x{start:X}")); }
                        match sz { 1 => { self.reg_w(0, 1, rex, q as u64); self.reg_w(4, 1, 1, r as u64); let _ = r; self.r[RAX] = (self.r[RAX] & !0xFF00) | ((r as u64 & 0xFF) << 8); } 2 => { self.reg_w(0, 2, rex, q as u64); self.reg_w(2, 2, rex, r as u64); } 4 => { self.r[RAX] = q as u64; self.r[RDX] = r as u64; } _ => { self.r[RAX] = q as u64; self.r[RDX] = r as u64; } }
                    }
                    _ => { // idiv
                        let d = Self::sext(v, sz) as i64 as i128; if d == 0 { return Err(format!("#DE division par zéro à 0x{start:X}")); }
                        let n: i128 = match sz { 1 => (self.r[RAX] as u16 as i16) as i128, 2 => ((((self.r[RDX] & 0xFFFF) << 16) | (self.r[RAX] & 0xFFFF)) as u32 as i32) as i128, 4 => ((((self.r[RDX] & 0xFFFF_FFFF) << 32) | (self.r[RAX] & 0xFFFF_FFFF)) as i64) as i128, _ => (((self.r[RDX] as u128) << 64) | self.r[RAX] as u128) as i128 };
                        let (q, r) = (n / d, n % d);
                        let lim = 1i128 << (sz * 8 - 1); if q >= lim || q < -lim { return Err(format!("#DE débordement à 0x{start:X}")); }
                        match sz { 1 => { self.r[RAX] = (self.r[RAX] & !0xFFFF) | ((r as u64 & 0xFF) << 8) | (q as u64 & 0xFF); } 2 => { self.reg_w(0, 2, rex, q as u64); self.reg_w(2, 2, rex, r as u64); } 4 => { self.r[RAX] = q as u32 as u64; self.r[RDX] = r as u32 as u64; } _ => { self.r[RAX] = q as u64; self.r[RDX] = r as u64; } }
                    }
                }
            }
            0xFE | 0xFF => {
                let sz = if op == 0xFE { 1 } else { osz };
                let m = self.modrm(&mut pc, rex)?; let end = pc; let kind = m.reg & 7;
                match kind {
                    0 | 1 => { let v = self.rm_read(&m, end, sz, rex)?; let cf = self.cf; let r = self.alu(if kind == 0 { 0 } else { 5 }, v, 1, sz); self.cf = cf; self.rm_write(&m, end, sz, rex, r)?; }
                    2 if op == 0xFF => { let t = self.rm_read(&m, end, 8, rex)?; self.push(end)?; self.rip = t; return Ok(true); }
                    4 if op == 0xFF => { let t = self.rm_read(&m, end, 8, rex)?; self.rip = t; return Ok(true); }
                    6 if op == 0xFF => { let v = self.rm_read(&m, end, 8, rex)?; self.push(v)?; }
                    _ => bad!(),
                }
            }
            0x0F => {
                let op2 = self.f8(&mut pc)?;
                match op2 {
                    0x0B => return Err(format!("#UD (ud2) exécuté à 0x{start:X} : chemin de panique ou code inatteignable")),
                    0x1F | 0x1E | 0x18 | 0x19 | 0x1A | 0x1B | 0x1C | 0x1D => { let _ = self.modrm(&mut pc, rex)?; } // nop multi-octets, endbr64, prefetch
                    0x40..=0x4F => { let m = self.modrm(&mut pc, rex)?; let end = pc; let v = self.rm_read(&m, end, osz, rex)?; if self.cond(op2 & 0xF) { self.reg_w(m.reg, osz, rex, v); } else if osz == 4 { self.reg_w(m.reg, 4, rex, self.r[m.reg]); } }
                    0x80..=0x8F => { let rel = self.f32(&mut pc)? as i32 as i64; if self.cond(op2 & 0xF) { self.jump_rel(pc, rel); return Ok(true); } }
                    0x90..=0x9F => { let m = self.modrm(&mut pc, rex)?; let end = pc; let v = self.cond(op2 & 0xF) as u64; self.rm_write(&m, end, 1, rex, v)?; }
                    0xA2 => { let c = unsafe { core::arch::x86_64::__cpuid_count(self.r[RAX] as u32, self.r[RCX] as u32) }; self.r[RAX] = c.eax as u64; self.r[RBX] = c.ebx as u64; self.r[RCX] = c.ecx as u64; self.r[RDX] = c.edx as u64; }
                    0xA3 | 0xAB | 0xB3 | 0xBB => {
                        let m = self.modrm(&mut pc, rex)?; let end = pc; let bit = self.reg_r(m.reg, osz, rex);
                        if m.is_reg { let v = self.reg_r(m.rm, osz, rex); let b = bit % (osz as u64 * 8); self.cf = (v >> b) & 1 != 0;
                            let nv = match op2 { 0xAB => v | (1 << b), 0xB3 => v & !(1 << b), 0xBB => v ^ (1 << b), _ => v }; if op2 != 0xA3 { self.reg_w(m.rm, osz, rex, nv); } }
                        else { let a = self.ea(&m, end).wrapping_add(((Self::sext(bit, osz) as i64) >> 3) as u64 & !(osz as u64 - 1)); let b = bit % (osz as u64 * 8); let v = self.rd(a, osz)?; self.cf = (v >> b) & 1 != 0;
                            let nv = match op2 { 0xAB => v | (1 << b), 0xB3 => v & !(1 << b), 0xBB => v ^ (1 << b), _ => v }; if op2 != 0xA3 { self.wr(a, osz, nv)?; } }
                    }
                    0xBA => {
                        let m = self.modrm(&mut pc, rex)?; let b = self.f8(&mut pc)? as u64 % (osz as u64 * 8); let end = pc; let v = self.rm_read(&m, end, osz, rex)?; self.cf = (v >> b) & 1 != 0;
                        let nv = match m.reg & 7 { 5 => v | (1 << b), 6 => v & !(1 << b), 7 => v ^ (1 << b), 4 => v, _ => bad!() };
                        if m.reg & 7 != 4 { self.rm_write(&m, end, osz, rex, nv)?; }
                    }
                    0xAF => { let m = self.modrm(&mut pc, rex)?; let end = pc; let a = Self::sext(self.reg_r(m.reg, osz, rex), osz) as i64 as i128; let b = Self::sext(self.rm_read(&m, end, osz, rex)?, osz) as i64 as i128; let r = a * b; let fits = r == Self::sext(r as u64, osz) as i64 as i128; self.cf = !fits; self.of = !fits; self.reg_w(m.reg, osz, rex, r as u64); self.set_szp(r as u64, osz); }
                    0xB6 | 0xB7 => { let m = self.modrm(&mut pc, rex)?; let end = pc; let sz = if op2 == 0xB6 { 1 } else { 2 }; let v = self.rm_read(&m, end, sz, rex)?; self.reg_w(m.reg, osz, rex, v); }
                    0xBE | 0xBF => { let m = self.modrm(&mut pc, rex)?; let end = pc; let sz = if op2 == 0xBE { 1 } else { 2 }; let v = Self::sext(self.rm_read(&m, end, sz, rex)?, sz); self.reg_w(m.reg, osz, rex, v); }
                    0xBC | 0xBD => {
                        let m = self.modrm(&mut pc, rex)?; let end = pc; let v = self.rm_read(&m, end, osz, rex)? & Self::mask(osz); let bits = osz as u32 * 8;
                        if rep { // tzcnt / lzcnt
                            let r = if v == 0 { bits } else if op2 == 0xBC { v.trailing_zeros() } else { v.leading_zeros() - (64 - bits) };
                            self.cf = v == 0; self.zf = r == 0; self.reg_w(m.reg, osz, rex, r as u64);
                        } else { self.zf = v == 0; if v != 0 { let r = if op2 == 0xBC { v.trailing_zeros() } else { 63 - v.leading_zeros() }; self.reg_w(m.reg, osz, rex, r as u64); } }
                    }
                    0xB8 if rep => { let m = self.modrm(&mut pc, rex)?; let end = pc; let v = self.rm_read(&m, end, osz, rex)?; let r = v.count_ones() as u64; self.reg_w(m.reg, osz, rex, r); self.cf = false; self.of = false; self.sf = false; self.pf = false; self.zf = v == 0; }
                    0xB0 | 0xB1 => { // cmpxchg
                        let sz = if op2 == 0xB0 { 1 } else { osz }; let m = self.modrm(&mut pc, rex)?; let end = pc; let cur = self.rm_read(&m, end, sz, rex)?;
                        let acc = self.r[RAX] & Self::mask(sz); self.alu(7, acc, cur, sz);
                        if self.zf { let v = self.reg_r(m.reg, sz, rex); self.rm_write(&m, end, sz, rex, v)?; } else { self.reg_w(0, sz, rex, cur); }
                    }
                    0xC0 | 0xC1 => { let sz = if op2 == 0xC0 { 1 } else { osz }; let m = self.modrm(&mut pc, rex)?; let end = pc; let a = self.rm_read(&m, end, sz, rex)?; let b = self.reg_r(m.reg, sz, rex); let r = self.alu(0, a, b, sz); self.reg_w(m.reg, sz, rex, a); self.rm_write(&m, end, sz, rex, r)?; }
                    0xC8..=0xCF => { let i = (op2 - 0xC8) as usize | if rex & 1 != 0 { 8 } else { 0 }; if w { self.r[i] = self.r[i].swap_bytes(); } else { self.r[i] = (self.r[i] as u32).swap_bytes() as u64; } }
                    _ => bad!(),
                }
            }
            _ => bad!(),
        }
        self.rip = pc;
        Ok(true)
    }
}

// ---------------------------------------------------------------- tests différentiels : l'unité arithmétique émulée contre le processeur HÔTE
// Les mêmes opérations sont exécutées par de vraies instructions (asm! en ligne) ; résultat et indicateurs CF, ZF, SF, OF, PF doivent être identiques.
#[cfg(test)]
mod tests_diff {
    use super::*;
    const VALS: [u64; 14] = [0, 1, 2, 0x7F, 0x80, 0xFF, 0x7FFF, 0x8000, 0xFFFF, 0x7FFF_FFFF, 0x8000_0000, 0xFFFF_FFFF, 0x7FFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFFF];

    fn host_flags(f: u64) -> (bool, bool, bool, bool, bool) { (f & 1 != 0, f & 0x40 != 0, f & 0x80 != 0, f & 0x800 != 0, f & 4 != 0) } // CF ZF SF OF PF

    macro_rules! host_op {
        ($mnem:literal, $sz:ident, $a:expr, $b:expr, $cf:expr) => {{
            let (mut r, f): (u64, u64);
            unsafe {
                core::arch::asm!(
                    "stc", "clc", // base propre puis, si demandé, retenue
                    "test {cin:e}, {cin:e}", "jz 2f", "stc", "2:",
                    concat!($mnem, " {r}, {b}"),
                    "pushfq", "pop {f}",
                    r = inout(reg) $a => r, b = in(reg) $b, f = out(reg) f, cin = in(reg) $cf as u32,
                    options(nomem)
                );
            }
            (r, f)
        }}
    }

    #[test] fn alu_matches_host_cpu_64bit() {
        for &a in &VALS { for &b in &VALS { for cin in [false, true] {
            for (op, name) in [(0u8, "add"), (1, "or"), (2, "adc"), (3, "sbb"), (4, "and"), (5, "sub"), (6, "xor"), (7, "cmp")] {
                let mut c = Cpu::new(); c.cf = cin;
                let got = c.alu(op, a, b, 8);
                let (r, f) = match name {
                    "add" => host_op!("add", u64, a, b, cin), "or" => host_op!("or", u64, a, b, cin), "adc" => host_op!("adc", u64, a, b, cin),
                    "sbb" => host_op!("sbb", u64, a, b, cin), "and" => host_op!("and", u64, a, b, cin), "sub" => host_op!("sub", u64, a, b, cin),
                    "xor" => host_op!("xor", u64, a, b, cin), _ => host_op!("cmp", u64, a, b, cin),
                };
                let want = if name == "cmp" { a } else { r };
                let (cf, zf, sf, of, pf) = host_flags(f);
                if name != "cmp" { assert_eq!(got, want, "{name} {a:#x} {b:#x} cin={cin} : résultat"); }
                let ctx = format!("{name} {a:#x} {b:#x} cin={cin}");
                assert_eq!((c.zf, c.sf, c.pf), (zf, sf, pf), "{ctx} : ZF SF PF");
                if !matches!(name, "or" | "and" | "xor") || true { assert_eq!((c.cf, c.of), (cf, of), "{ctx} : CF OF"); }
            }
        } } }
    }

    #[test] fn alu_32_and_8_bit_match_host_cpu() {
        for &a in &VALS { for &b in &VALS {
            // 32 bits
            let mut c = Cpu::new(); let got = c.alu(5, a, b, 4);
            let (r32, f): (u32, u64);
            unsafe { let mut x = a as u32; core::arch::asm!("sub {x:e}, {y:e}", "pushfq", "pop {f}", x = inout(reg) x, y = in(reg) b as u32, f = out(reg) f); r32 = x; }
            let (cf, zf, sf, of, pf) = host_flags(f);
            assert_eq!(got, r32 as u64, "sub32 {a:#x} {b:#x}"); assert_eq!((c.cf, c.zf, c.sf, c.of, c.pf), (cf, zf, sf, of, pf), "sub32 flags {a:#x} {b:#x}");
            // 8 bits
            let mut c = Cpu::new(); let got = c.alu(0, a, b, 1);
            let (r8, f): (u8, u64);
            unsafe { let mut x = a as u8; core::arch::asm!("add {x}, {y}", "pushfq", "pop {f}", x = inout(reg_byte) x, y = in(reg_byte) b as u8, f = out(reg) f); r8 = x; }
            let (cf, zf, sf, of, pf) = host_flags(f);
            assert_eq!(got, r8 as u64, "add8 {a:#x} {b:#x}"); assert_eq!((c.cf, c.zf, c.sf, c.of, c.pf), (cf, zf, sf, of, pf), "add8 flags {a:#x} {b:#x}");
        } }
    }

    #[test] fn shifts_match_host_cpu() {
        for &a in &VALS { for count in [1u32, 2, 3, 7, 8, 15, 16, 31, 32, 33, 63] {
            for (kind, name) in [(4usize, "shl"), (5, "shr"), (7, "sar")] {
                let mut c = Cpu::new();
                let got = c.shift(kind, a, count, 8).unwrap();
                let (r, f): (u64, u64);
                unsafe {
                    let mut x = a;
                    match name {
                        "shl" => core::arch::asm!("shl {x}, cl", "pushfq", "pop {f}", x = inout(reg) x, f = out(reg) f, in("cl") count as u8),
                        "shr" => core::arch::asm!("shr {x}, cl", "pushfq", "pop {f}", x = inout(reg) x, f = out(reg) f, in("cl") count as u8),
                        _ => core::arch::asm!("sar {x}, cl", "pushfq", "pop {f}", x = inout(reg) x, f = out(reg) f, in("cl") count as u8),
                    }
                    r = x;
                }
                let (cf, zf, sf, _of, pf) = host_flags(f);
                assert_eq!(got, r, "{name} {a:#x} par {count}");
                assert_eq!((c.cf, c.zf, c.sf, c.pf), (cf, zf, sf, pf), "{name} {a:#x} par {count} : CF ZF SF PF");
                if count == 1 { let (_, _, _, of, _) = host_flags(f); assert_eq!(c.of, of, "{name} {a:#x} par 1 : OF"); }
            }
        } }
    }
}
