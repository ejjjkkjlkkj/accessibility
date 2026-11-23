//! Décodeur LZMA (format « alone » : 5 octets de propriétés, 8 octets de taille, flux), écrit pour Voxa d'après la spécification publique LZMA.
//! Sans dépendance. Tout flux invalide renvoie Err (jamais de panique sur entrée hostile : bornes vérifiées).

struct Rc<'a> { d: &'a [u8], p: usize, range: u32, code: u32 }
const TOP: u32 = 1 << 24;
impl<'a> Rc<'a> {
    fn new(d: &'a [u8]) -> Result<Self, &'static str> {
        if d.len() < 5 || d[0] != 0 { return Err("flux range-coder invalide"); }
        Ok(Rc { d, p: 5, range: 0xFFFF_FFFF, code: u32::from_be_bytes([d[1], d[2], d[3], d[4]]) })
    }
    fn norm(&mut self) -> Result<(), &'static str> { if self.range < TOP { let b = *self.d.get(self.p).ok_or("flux tronqué")?; self.p += 1; self.range <<= 8; self.code = (self.code << 8) | b as u32; } Ok(()) }
    fn bit(&mut self, prob: &mut u16) -> Result<u32, &'static str> {
        self.norm()?;
        let bound = (self.range >> 11) * (*prob as u32);
        if self.code < bound { self.range = bound; *prob += (2048 - *prob) >> 5; Ok(0) } else { self.range -= bound; self.code -= bound; *prob -= *prob >> 5; Ok(1) }
    }
    fn direct(&mut self, n: u32) -> Result<u32, &'static str> {
        let mut r: u32 = 0; for _ in 0..n { self.norm()?; self.range >>= 1; self.code = self.code.wrapping_sub(self.range); let t = 0u32.wrapping_sub(self.code >> 31); self.code = self.code.wrapping_add(self.range & t); r = (r << 1).wrapping_add(t.wrapping_add(1)); } Ok(r)
    }
}
fn tree(rc: &mut Rc, probs: &mut [u16], bits: u32) -> Result<u32, &'static str> { let mut m = 1usize; for _ in 0..bits { m = (m << 1) + rc.bit(&mut probs[m])? as usize; } Ok(m as u32 - (1 << bits)) }
fn rtree(rc: &mut Rc, probs: &mut [u16], bits: u32) -> Result<u32, &'static str> { let mut m = 1usize; let mut s = 0; for i in 0..bits { let b = rc.bit(&mut probs[m])?; m = (m << 1) + b as usize; s |= b << i; } Ok(s) }

struct Len { choice: u16, choice2: u16, low: [[u16; 8]; 16], mid: [[u16; 8]; 16], high: [u16; 256] }
impl Len {
    fn new() -> Self { Len { choice: 1024, choice2: 1024, low: [[1024; 8]; 16], mid: [[1024; 8]; 16], high: [1024; 256] } }
    fn decode(&mut self, rc: &mut Rc, ps: usize) -> Result<u32, &'static str> {
        if rc.bit(&mut self.choice)? == 0 { return tree(rc, &mut self.low[ps], 3); }
        if rc.bit(&mut self.choice2)? == 0 { return Ok(8 + tree(rc, &mut self.mid[ps], 3)?); }
        Ok(16 + tree(rc, &mut self.high, 8)?)
    }
}

/// `data` = 5 octets de propriétés (lc/lp/pb + taille de dictionnaire) puis flux. `out_len` = taille attendue de la sortie.
pub fn decode(props: &[u8], stream: &[u8], out_len: usize) -> Result<Vec<u8>, &'static str> {
    if props.len() < 5 { return Err("propriétés manquantes"); }
    let mut d = props[0] as u32; if d >= 9 * 5 * 5 { return Err("propriétés invalides"); }
    let lc = d % 9; d /= 9; let lp = d % 5; let pb = d / 5;
    if out_len > 1 << 30 { return Err("taille de sortie déraisonnable"); }
    let mut rc = Rc::new(stream)?;
    let mut out: Vec<u8> = Vec::with_capacity(out_len);
    let mut lit = vec![1024u16; 0x300 << (lc + lp)];
    let (mut is_match, mut is_rep, mut rep_g0, mut rep_g1, mut rep_g2) = ([[1024u16; 16]; 12], [1024u16; 12], [1024u16; 12], [1024u16; 12], [1024u16; 12]);
    let mut is_rep0_long = [[1024u16; 16]; 12];
    let mut pos_slot = [[1024u16; 64]; 4]; let mut pos_dec = [1024u16; 115]; let mut align = [1024u16; 16];
    let (mut len, mut rep_len) = (Len::new(), Len::new());
    let (mut state, mut r0, mut r1, mut r2, mut r3) = (0usize, 0u32, 0u32, 0u32, 0u32);
    let pbm = (1usize << pb) - 1; let lpm = (1usize << lp) - 1;
    while out.len() < out_len {
        let pos = out.len(); let ps = pos & pbm;
        if rc.bit(&mut is_match[state][ps])? == 0 {
            let prev = if pos > 0 { out[pos - 1] as usize } else { 0 };
            let base = 0x300 * (((pos & lpm) << lc) + (prev >> (8 - lc)));
            let probs = &mut lit[base..base + 0x300];
            let mut sym = 1usize;
            if state >= 7 {
                let mut mb = out[pos - r0 as usize - 1] as usize;
                while sym < 0x100 { let mbit = (mb >> 7) & 1; mb <<= 1; let b = rc.bit(&mut probs[((1 + mbit) << 8) + sym])? as usize; sym = (sym << 1) | b; if mbit != b { break; } }
            }
            while sym < 0x100 { sym = (sym << 1) | rc.bit(&mut probs[sym])? as usize; }
            out.push(sym as u8); state = if state < 4 { 0 } else if state < 10 { state - 3 } else { state - 6 };
            continue;
        }
        let l;
        if rc.bit(&mut is_rep[state])? == 0 {
            r3 = r2; r2 = r1; r1 = r0;
            l = len.decode(&mut rc, ps)?; state = if state < 7 { 7 } else { 10 };
            let ls = (l.min(3)) as usize; let slot = tree(&mut rc, &mut pos_slot[ls], 6)?;
            if slot < 4 { r0 = slot } else {
                let nb = (slot >> 1) - 1; let mut v = (2 | (slot & 1)) << nb;
                if slot < 14 { v += rtree_at(&mut rc, &mut pos_dec, (v - slot) as usize, nb)?; } else { v += rc.direct(nb - 4)? << 4; v += rtree(&mut rc, &mut align, 4)?; }
                r0 = v;
            }
            if r0 == 0xFFFF_FFFF { break; }
        } else {
            if pos == 0 { return Err("répétition sans donnée"); }
            if rc.bit(&mut rep_g0[state])? == 0 {
                if rc.bit(&mut is_rep0_long[state][ps])? == 0 { state = if state < 7 { 9 } else { 11 }; let b = out[pos - r0 as usize - 1]; out.push(b); continue; }
            } else {
                let dist;
                if rc.bit(&mut rep_g1[state])? == 0 { dist = r1 } else { if rc.bit(&mut rep_g2[state])? == 0 { dist = r2 } else { dist = r3; r3 = r2 } r2 = r1 }
                r1 = r0; r0 = dist;
            }
            l = rep_len.decode(&mut rc, ps)?; state = if state < 7 { 8 } else { 11 };
        }
        let l = l as usize + 2; let dist = r0 as usize + 1;
        if dist > out.len() { return Err("distance hors fenêtre"); }
        for _ in 0..l { if out.len() >= out_len { break; } let b = out[out.len() - dist]; out.push(b); }
    }
    if out.len() != out_len { return Err("sortie plus courte qu'annoncée"); }
    Ok(out)
}
fn rtree_at(rc: &mut Rc, probs: &mut [u16], off: usize, bits: u32) -> Result<u32, &'static str> {
    // probabilités des distances courtes : tableau partagé, indexé à partir de `off` (convention LZMA : base + m, avec base = v - slot - 1)
    let mut m = 1usize; let mut s = 0;
    for i in 0..bits { let idx = off + m; let b = rc.bit(probs.get_mut(idx).ok_or("indice hors table")?)?; m = (m << 1) + b as usize; s |= b << i; }
    Ok(s)
}
