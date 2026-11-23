//! Décodeur AML maison (DSDT/SSDT) en deux passes, SANS exécuter de code :
//!  1. espace de noms : Scope/Device/Processor/…, Name, Method (arité), External, OperationRegion, Field, blocs If/Else de définition ;
//!  2. les corps des méthodes _CRS/_PRS sont analysés par un analyseur de termes COMPLET (tous les opcodes AML 6.x, avec leur arité), puis
//!     « évalués » statiquement : Return(Buffer | nom | ConcatenateResTemplate(a,b) | Local), If/Else (toutes les branches, marquées conditionnelles).
//! Précision : ce qui n'est pas résolu (appel de méthode inconnue, valeur calculée) est COMPTÉ et listé dans `notes`, jamais deviné.
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Res { Io { base: u16, len: u8 }, IoRange { min: u16, max: u16, len: u8 }, Mem32 { base: u32, len: u32 }, Irq(Vec<u8>), ExtIrq(Vec<u32>), I2c { addr: u16, speed: u32, source: String }, Spi, Uart, Gpio { kind: &'static str, pin: u16, source: String }, Other(u8) }

#[derive(Debug, Default, Clone)]
pub struct Dev { pub path: String, pub hid: Option<String>, pub cid: Option<String>, pub uid: Option<String>, pub adr: Option<u64>, pub conditional: bool,
    /// (nom, ressources, conditionnel). Les ressources issues d'une méthode sont marquées conditionnelles si plusieurs retours sont possibles.
    pub res: Vec<(String, Vec<Res>, bool)> }

#[derive(Default)]
pub struct Namespace { pub devices: BTreeMap<String, Dev>, pub unknown: usize, pub notes: Vec<String>, pub skipped: Vec<(usize, u8)>, pub methods_seen: usize,
    pub regions: BTreeMap<String, u8>, pub fields: Vec<(String, String, usize, usize)>,
    names: BTreeMap<String, Val>, aliases: BTreeMap<String, String>, margs: BTreeMap<String, u8>, crs: Vec<(String, String, usize, usize, usize)>, tables: Vec<Vec<u8>> }

#[derive(Clone, Debug)]
enum Val { Int(u64), Str(String), Buf(Vec<u8>), Other }

// ------------------------------------------------------------ primitives
fn pkg(d: &[u8], i: usize) -> Option<(usize, usize)> {
    let lead = *d.get(i)?; let extra = (lead >> 6) as usize;
    if extra == 0 { return Some(((lead & 0x3F) as usize, 1)); }
    let mut v = (lead & 0x0F) as usize; for k in 0..extra { v |= (*d.get(i + 1 + k)? as usize) << (4 + 8 * k); }
    Some((v, 1 + extra))
}
fn is_lead(c: u8) -> bool { c == b'_' || c.is_ascii_uppercase() }
fn seg(d: &[u8], i: usize) -> Option<String> { let s = d.get(i..i + 4)?; if !is_lead(s[0]) || !s.iter().all(|&c| c == b'_' || c.is_ascii_uppercase() || c.is_ascii_digit()) { return None; } Some(String::from_utf8_lossy(s).into_owned()) }
fn is_name_start(c: u8) -> bool { is_lead(c) || matches!(c, b'\\' | b'^' | 0x2E | 0x2F) }
fn name(d: &[u8], mut i: usize) -> Option<(String, Vec<String>, usize)> {
    let s0 = i; let mut prefix = String::new();
    if *d.get(i)? == b'\\' { prefix.push('\\'); i += 1; } else { while *d.get(i)? == b'^' { prefix.push('^'); i += 1; } }
    let segs = match *d.get(i)? { 0 => { i += 1; vec![] } 0x2E => { i += 1; let a = seg(d, i)?; let b = seg(d, i + 4)?; i += 8; vec![a, b] }
        0x2F => { let n = *d.get(i + 1)? as usize; i += 2; let mut v = vec![]; for _ in 0..n { v.push(seg(d, i)?); i += 4; } v }
        _ => { let a = seg(d, i)?; i += 4; vec![a] } };
    Some((prefix, segs, i - s0))
}
fn join(scope: &[String], prefix: &str, segs: &[String]) -> Vec<String> {
    let mut p: Vec<String> = if prefix.starts_with('\\') { vec![] } else { scope.to_vec() };
    for c in prefix.chars() { if c == '^' { p.pop(); } }
    p.extend(segs.iter().cloned()); p
}
fn path_str(p: &[String]) -> String { let mut s = String::from("\\"); s.push_str(&p.join(".")); s }
fn int(d: &[u8], i: usize) -> Option<(u64, usize)> {
    Some(match *d.get(i)? { 0x00 => (0, 1), 0x01 => (1, 1), 0xFF => (u64::MAX, 1), 0x0A => (*d.get(i + 1)? as u64, 2),
        0x0B => (u16::from_le_bytes(d.get(i + 1..i + 3)?.try_into().ok()?) as u64, 3), 0x0C => (u32::from_le_bytes(d.get(i + 1..i + 5)?.try_into().ok()?) as u64, 5),
        0x0E => (u64::from_le_bytes(d.get(i + 1..i + 9)?.try_into().ok()?), 9), _ => return None })
}
/// _HID/_CID entier = identifiant EISA compressé (ex. PNP0303).
pub fn eisa(v: u64) -> String {
    let b = (v as u32).to_le_bytes(); let c = |x: u8| (x + 0x40) as char;
    format!("{}{}{}{:X}{:X}{:X}{:X}", c((b[0] >> 2) & 0x1F), c(((b[0] & 3) << 3) | (b[1] >> 5)), c(b[1] & 0x1F), b[2] >> 4, b[2] & 15, b[3] >> 4, b[3] & 15)
}

// ------------------------------------------------------------ ressources
pub fn resources(b: &[u8]) -> Vec<Res> {
    let mut v = vec![]; let mut i = 0;
    while i < b.len() {
        let t = b[i];
        if t & 0x80 == 0 {
            let kind = (t >> 3) & 0xF; let len = (t & 7) as usize; let body = match b.get(i + 1..i + 1 + len) { Some(x) => x, None => break };
            match kind {
                0xF => break,
                0x4 if len >= 2 => { let m = u16::from_le_bytes([body[0], body[1]]); v.push(Res::Irq((0..16).filter(|k| m >> k & 1 == 1).collect())); }
                0x8 if len == 7 => v.push(Res::IoRange { min: u16::from_le_bytes([body[1], body[2]]), max: u16::from_le_bytes([body[3], body[4]]), len: body[6] }),
                0x9 if len == 3 => v.push(Res::Io { base: u16::from_le_bytes([body[0], body[1]]), len: body[2] }),
                k => v.push(Res::Other(k)),
            }
            i += 1 + len;
        } else {
            let kind = t & 0x7F; let len = match b.get(i + 1..i + 3) { Some(x) => u16::from_le_bytes([x[0], x[1]]) as usize, None => break };
            let body = match b.get(i + 3..i + 3 + len) { Some(x) => x, None => break };
            match kind {
                0x06 if len == 9 => v.push(Res::Mem32 { base: u32::from_le_bytes(body[1..5].try_into().unwrap()), len: u32::from_le_bytes(body[5..9].try_into().unwrap()) }),
                0x09 if len >= 6 => { let n = body[1] as usize; v.push(Res::ExtIrq((0..n).filter_map(|k| body.get(2 + 4 * k..6 + 4 * k).map(|x| u32::from_le_bytes(x.try_into().unwrap()))).collect())); }
                0x0E if len >= 12 => match body[2] {
                    1 => { let speed = u32::from_le_bytes(body[9..13].try_into().unwrap_or([0; 4])); let addr = u16::from_le_bytes([*body.get(13).unwrap_or(&0), *body.get(14).unwrap_or(&0)]);
                        let src = String::from_utf8_lossy(body.get(15..).unwrap_or(&[]).split(|&c| c == 0).next().unwrap_or(&[])).into_owned(); v.push(Res::I2c { addr, speed, source: src }) }
                    2 => v.push(Res::Spi), 3 => v.push(Res::Uart), _ => v.push(Res::Other(0x0E)) },
                0x0C if len >= 18 => {
                    let pt = u16::from_le_bytes([body[11], body[12]]) as usize; let sn = u16::from_le_bytes([body[14], body[15]]) as usize;
                    let pin = b.get(i + pt..i + pt + 2).map(|x| u16::from_le_bytes([x[0], x[1]])).unwrap_or(0xFFFF);
                    let src = b.get(i + sn..).map(|x| String::from_utf8_lossy(x.split(|&c| c == 0).next().unwrap_or(&[])).into_owned()).unwrap_or_default();
                    v.push(Res::Gpio { kind: if body[1] == 0 { "interruption" } else { "E/S" }, pin, source: src }) }
                k => v.push(Res::Other(k | 0x80)),
            }
            i += 3 + len;
        }
    }
    v
}
pub fn describe(r: &Res) -> String {
    match r {
        Res::Io { base, len } => format!("E/S 0x{base:X} (+{len})"), Res::IoRange { min, max, len } => format!("E/S 0x{min:X}..0x{max:X} (+{len})"), Res::Mem32 { base, len } => format!("MÉM 0x{base:X} (0x{len:X} octets)"),
        Res::Irq(l) => format!("IRQ {l:?}"), Res::ExtIrq(l) => format!("IRQ étendue {l:?}"), Res::I2c { addr, speed, source } => format!("I2C adresse 0x{addr:X} à {speed} Hz sur {source}"),
        Res::Spi => "SPI".into(), Res::Uart => "UART".into(), Res::Gpio { kind, pin, source } => format!("GPIO {kind} broche {pin} sur {source}"), Res::Other(k) => format!("(descripteur 0x{k:X} non décodé)"),
    }
}

// ------------------------------------------------------------ analyseur de termes (arbre minimal)
#[derive(Clone, Debug)]
enum T { Int(u64), Str(String), Buf(Vec<u8>), Nm(String), Local(u8), Arg(u8), Call(String, Vec<T>), Op(u8, Vec<T>), Block(u8, Vec<T>, Vec<T>), Stmts(Vec<T>), Def(String, Box<T>), Other }

struct P<'a> { d: &'a [u8], margs: BTreeMap<String, u8>, unresolved: Vec<String> }

impl<'a> P<'a> {
    /// Un terme à la position i ; renvoie (terme, position suivante). None = octets invalides (le parcours s'arrête, il ne devine pas).
    fn term(&mut self, i: usize, end: usize, depth: usize) -> Option<(T, usize)> {
        if i >= end || depth > 64 { return None; }
        let d = self.d; let op = d[i];
        let simple = |s: &mut Self, n: usize, o: u8, from: usize, depth: usize| -> Option<(T, usize)> { let mut j = from; let mut a = vec![]; for _ in 0..n { let (t, nj) = s.term(j, end, depth + 1)?; a.push(t); j = nj; } Some((T::Op(o, a), j)) };
        match op {
            0x00 | 0x01 | 0xFF | 0x0A | 0x0B | 0x0C | 0x0E => { let (v, l) = int(d, i)?; Some((T::Int(v), i + l)) }
            0x0D => { let e = d.get(i + 1..end)?.iter().position(|&b| b == 0)?; Some((T::Str(String::from_utf8_lossy(&d[i + 1..i + 1 + e]).into_owned()), i + 2 + e)) }
            0x11 => { let (pl, hl) = pkg(d, i + 1)?; let pe = i + 1 + pl; if pe > end { return None; } let (t, j) = self.term(i + 1 + hl, pe, depth + 1)?; let size = if let T::Int(n) = t { n as usize } else { 0 };
                let data = d.get(j..pe)?; Some((T::Buf(data[..data.len().min(size)].to_vec()), pe)) }
            0x12 | 0x13 => { let (pl, hl) = pkg(d, i + 1)?; let pe = i + 1 + pl; if pe > end { return None; } let mut j = i + 1 + hl; if op == 0x12 { j += 1; } else { let (_, nj) = self.term(j, pe, depth + 1)?; j = nj; }
                let mut el = vec![]; while j < pe { let (t, nj) = self.term(j, pe, depth + 1)?; el.push(t); j = nj; } Some((T::Stmts(el), pe)) }
            0x60..=0x67 => Some((T::Local(op - 0x60), i + 1)), 0x68..=0x6E => Some((T::Arg(op - 0x68), i + 1)),
            0x08 => { let (pre, segs, nl) = name(d, i + 1)?; let (t, j) = self.term(i + 1 + nl, end, depth + 1)?; Some((T::Def(format!("{}{}", pre, segs.join(".")), Box::new(t)), j)) }
            0x14 => { let (pl, _) = pkg(d, i + 1)?; Some((T::Other, i + 1 + pl)) } // Method imbriquée : saut par longueur
            0xA0 | 0xA2 => { let (pl, hl) = pkg(d, i + 1)?; let pe = i + 1 + pl; if pe > end { return None; } let (c, mut j) = self.term(i + 1 + hl, pe, depth + 1)?; let mut body = vec![];
                while j < pe { let (t, nj) = self.term(j, pe, depth + 1)?; body.push(t); j = nj; }
                // Else éventuel juste après
                let mut els = vec![]; let mut next = pe; if op == 0xA0 && next < end && d[next] == 0xA1 { let (pl2, hl2) = pkg(d, next + 1)?; let pe2 = next + 1 + pl2; if pe2 > end { return None; } let mut k = next + 1 + hl2; while k < pe2 { let (t, nk) = self.term(k, pe2, depth + 1)?; els.push(t); k = nk; } next = pe2; }
                Some((T::Block(op, { let mut v = vec![c]; v.extend(body); v }, els), next)) }
            0xA1 => { let (pl, _) = pkg(d, i + 1)?; Some((T::Other, i + 1 + pl)) }
            0xA3 | 0xA5 | 0x9F | 0xCC | 0xA6 => Some((T::Other, i + 1)),
            0xA4 => { let (t, j) = self.term(i + 1, end, depth + 1)?; Some((T::Op(0xA4, vec![t]), j)) }
            0x71 | 0x75 | 0x76 | 0x83 | 0x87 | 0x8E => simple(self, 1, op, i + 1, depth),
            0x80 | 0x81 | 0x82 | 0x86 | 0x90 | 0x91 | 0x93 | 0x94 | 0x95 | 0x96 | 0x97 | 0x98 | 0x99 | 0x9D => simple(self, 2, op, i + 1, depth),
            0x72 | 0x73 | 0x74 | 0x77 | 0x79 | 0x7A | 0x7B | 0x7C | 0x7D | 0x7E | 0x7F | 0x84 | 0x85 | 0x88 | 0x9C => simple(self, 3, op, i + 1, depth),
            0x78 | 0x9E => simple(self, 4, op, i + 1, depth),
            0x92 => { if matches!(d.get(i + 1), Some(0x93..=0x95)) { simple(self, 2, 0x92, i + 2, depth) } else { simple(self, 1, 0x92, i + 1, depth) } }
            0x70 => simple(self, 2, 0x70, i + 1, depth),
            // DefMatch : SearchPkg(terme) MatchOpcode(octet) Opérande(terme) MatchOpcode(octet) Opérande(terme) StartIndex(terme) = 4 termes + 2 octets (ACPICA : ARGP_MATCH_OP)
            0x89 => { let (a, j) = self.term(i + 1, end, depth + 1)?; let (b, j) = self.term(j + 1, end, depth + 1)?; let (c, j) = self.term(j + 1, end, depth + 1)?; let (e, j) = self.term(j, end, depth + 1)?; Some((T::Op(0x89, vec![a, b, c, e]), j)) }
            0x8A | 0x8B | 0x8C | 0x8D | 0x8F => { let (a, j) = self.term(i + 1, end, depth + 1)?; let (b, j) = self.term(j, end, depth + 1)?; let (_, _, nl) = name(d, j)?; Some((T::Op(op, vec![a, b]), j + nl)) }
            0x5B => {
                let ext = *d.get(i + 1)?;
                match ext {
                    0x01 => { let (_, _, nl) = name(d, i + 2)?; Some((T::Other, i + 2 + nl + 1)) } 0x02 => { let (_, _, nl) = name(d, i + 2)?; Some((T::Other, i + 2 + nl)) }
                    0x1F => simple(self, 6, 0x1F, i + 2, depth), // LoadTable : 6 termes (ARGP_LOAD_TABLE_OP)
                    0x12 => simple(self, 2, 0x12, i + 2, depth), 0x13 => { let (a, j) = self.term(i + 2, end, depth + 1)?; let (b, j) = self.term(j, end, depth + 1)?; let (c, j) = self.term(j, end, depth + 1)?; let (_, _, nl) = name(d, j)?; Some((T::Op(0x13, vec![a, b, c]), j + nl)) }
                    0x20 => { let (_, _, nl) = name(d, i + 2)?; let (t, j) = self.term(i + 2 + nl, end, depth + 1)?; Some((T::Op(0x20, vec![t]), j)) }
                    0x21 | 0x22 | 0x24 | 0x26 | 0x27 | 0x2A => simple(self, 1, ext, i + 2, depth), 0x25 | 0x28 | 0x29 => simple(self, 2, ext, i + 2, depth),
                    0x23 => { let (t, j) = self.term(i + 2, end, depth + 1)?; Some((T::Op(0x23, vec![t]), j + 2)) }
                    0x30 | 0x31 | 0x33 => Some((T::Other, i + 2)),
                    0x32 => { let (t, j) = self.term(i + 7, end, depth + 1)?; Some((T::Op(0x32, vec![t]), j)) }
                    0x80 => { let (_, _, nl) = name(d, i + 2)?; let (a, j) = self.term(i + 2 + nl + 1, end, depth + 1)?; let (b, j) = self.term(j, end, depth + 1)?; Some((T::Op(0x80, vec![a, b]), j)) }
                    0x81 | 0x82 | 0x83 | 0x84 | 0x85 | 0x86 | 0x87 => { let (pl, _) = pkg(d, i + 2)?; Some((T::Other, i + 2 + pl)) }
                    0x88 => { let (_, _, nl) = name(d, i + 2)?; let mut j = i + 2 + nl; for _ in 0..3 { j = self.term(j, end, depth + 1)?.1; } Some((T::Other, j)) }
                    _ => None,
                }
            }
            c if is_name_start(c) => {
                let (pre, segs, nl) = name(d, i)?; let full = format!("{}{}", pre, segs.join(".")); let last = segs.last().cloned().unwrap_or_default();
                let n = match self.margs.get(&last) { Some(&n) => n as usize, None => { if !last.is_empty() { self.unresolved.push(full.clone()); } 0 } };
                let mut j = i + nl; let mut args = vec![]; for _ in 0..n { let (t, nj) = self.term(j, end, depth + 1)?; args.push(t); j = nj; }
                Some(if n > 0 || self.margs.contains_key(&last) { (T::Call(full, args), j) } else { (T::Nm(full), j) })
            }
            _ => None,
        }
    }
}

// ------------------------------------------------------------ évaluation statique de _CRS/_PRS
type Locals = BTreeMap<u8, Vec<Vec<u8>>>;
fn strip_end(b: &[u8]) -> Vec<u8> { let mut v = b.to_vec(); if v.len() >= 2 && v[v.len() - 2] == 0x79 { v.truncate(v.len() - 2); } else if v.len() >= 1 && v[v.len() - 1] == 0x79 { v.pop(); } v }

struct Ev<'a> { ns: &'a Namespace, scope: Vec<String>, notes: Vec<String>, returns: Vec<Vec<u8>>, conditional: bool, named: BTreeMap<String, Vec<u8>> }
impl<'a> Ev<'a> {
    fn lookup(&self, nm: &str) -> Option<&'a Val> {
        let last = nm.trim_start_matches(|c| c == '\\' || c == '^'); let segs: Vec<String> = last.split('.').map(String::from).collect();
        if nm.starts_with('\\') { return self.ns.names.get(&path_str(&segs)); }
        let mut sc = self.scope.clone(); loop { let mut p = sc.clone(); p.extend(segs.iter().cloned()); let k = path_str(&p); if let Some(v) = self.ns.names.get(&k).or_else(|| self.ns.aliases.get(&k).and_then(|s| self.ns.names.get(s))) { return Some(v); } if sc.pop().is_none() { return None; } }
    }
    fn eval(&mut self, t: &T, locals: &Locals) -> Vec<Vec<u8>> {
        match t {
            T::Buf(b) => vec![b.clone()],
            T::Nm(n) if self.named.contains_key(n.as_str()) => vec![self.named[n.as_str()].clone()],
            T::Nm(n) => match self.lookup(n) { Some(Val::Buf(b)) => vec![b.clone()], Some(Val::Int(v)) => { self.notes.push(format!("{n} = 0x{v:X} (entier) : fenêtre calculée à l'exécution par le firmware, non connaissable statiquement")); vec![] } _ => { self.notes.push(format!("nom {n} introuvable ou non tampon")); vec![] } },
            T::Local(k) => locals.get(k).cloned().unwrap_or_default(),
            T::Op(0x84, a) if a.len() >= 2 => { let (x, y) = (self.eval(&a[0], locals), self.eval(&a[1], locals)); let mut out = vec![]; for p in &x { for q in &y { let mut v = strip_end(p); v.extend_from_slice(q); out.push(v); } } out }
            T::Op(0x70, a) if a.len() == 2 => self.eval(&a[0], locals), // Store(x, cible) vaut x
            T::Call(n, _) => { self.notes.push(format!("appel de méthode {n} non évalué")); vec![] }
            _ => { self.notes.push("expression non évaluée".into()); vec![] }
        }
    }
    fn run(&mut self, stmts: &[T], locals: &mut Locals) {
        for s in stmts {
            match s {
                T::Def(n, v) => { if let T::Buf(b) = v.as_ref() { self.named.insert(n.clone(), b.clone()); } }
                T::Op(0xA4, a) => { let r = self.eval(&a[0], locals); self.returns.extend(r); }
                T::Op(0x70, a) if a.len() == 2 => { if let T::Local(k) = &a[1] { let v = self.eval(&a[0], locals); if !v.is_empty() { locals.insert(*k, v); } } }
                T::Block(0xA0, body, els) => { self.conditional = true; let mut l1 = locals.clone(); self.run(&body[1..], &mut l1); let mut l2 = locals.clone(); self.run(els, &mut l2); }
                T::Block(0xA2, body, _) => { self.conditional = true; let mut l1 = locals.clone(); self.run(&body[1..], &mut l1); }
                _ => {}
            }
        }
    }
}

// ------------------------------------------------------------ passe 1 : espace de noms
impl Namespace {
    fn walk(&mut self, tix: usize, mut i: usize, end: usize, scope: &[String], cur: Option<&str>, cond: bool) {
        while i < end {
            let d = std::mem::take(&mut self.tables[tix]); // emprunt temporaire de la table
            let r = self.step(&d, tix, i, end, scope, cur, cond);
            self.tables[tix] = d;
            match r { Some(n) => i = n, None => return }
        }
    }
    fn fail(&mut self, i: usize, op: u8, why: &str) -> Option<usize> { self.unknown += 1; self.notes.push(format!("arrêt @{i}:0x{op:02X} {why}")); None }
    fn step(&mut self, d: &[u8], tix: usize, i: usize, end: usize, scope: &[String], cur: Option<&str>, cond: bool) -> Option<usize> {
        let op = d[i];
        match op {
            0x10 | 0x5B => {
                let (ext, hdr) = if op == 0x5B { (d.get(i + 1).copied().unwrap_or(0), 2) } else { (0x10, 1) };
                match ext {
                    0x10 | 0x82 | 0x83 | 0x84 | 0x85 => {
                        let Some((pl, hl)) = pkg(d, i + hdr) else { return self.fail(i, op, "longueur illisible") };
                        let pend = i + hdr + pl; let body = i + hdr + hl;
                        if pend > d.len() || pend > end { return self.fail(i, op, "paquet hors limites"); }
                        let Some((pre, segs, nl)) = name(d, body) else { return self.fail(i, op, "nom illisible") };
                        let np = join(scope, &pre, &segs); let ps = path_str(&np); let mut inner = body + nl;
                        if ext == 0x83 { inner += 6; } if ext == 0x84 { inner += 3; }
                        if ext == 0x82 { let e = self.devices.entry(ps.clone()).or_insert_with(|| Dev { path: ps.clone(), ..Default::default() }); e.conditional |= cond; }
                        let c = if ext == 0x82 { Some(ps.clone()) } else { cur.map(String::from) };
                        // récursion : on repose la table le temps de l'appel
                        self.tables[tix] = d.to_vec(); self.walk(tix, inner.min(pend), pend, &np, c.as_deref(), cond); self.tables[tix] = Vec::new();
                        Some(pend)
                    }
                    0x80 => { // OperationRegion : on retient l'espace d'adresses (3 = contrôleur embarqué) ; décalage/longueur = termes quelconques
                        let Some((_, segs, nl)) = name(d, i + 2) else { return self.fail(i, op, "région") };
                        if let (Some(l), Some(&sp)) = (segs.last(), d.get(i + 2 + nl)) { self.regions.insert(l.clone(), sp); }
                        let mut p = P { d, margs: self.margs.clone(), unresolved: vec![] };
                        let r = p.term(i + 2 + nl + 1, end, 0).and_then(|(_, j)| p.term(j, end, 0)); match r { Some((_, j)) => Some(j), None => self.fail(i, op, "région à expression illisible") }
                    }
                    0x81 => { // Field : champs nommés avec leur décalage en bits dans la région (utile surtout pour le contrôleur embarqué)
                        let Some((pl, hl)) = pkg(d, i + 2) else { return self.fail(i, op, "champ") }; let pe = i + 2 + pl; if pe > end { return self.fail(i, op, "champ hors limites"); }
                        let Some((_, segs, nl)) = name(d, i + 2 + hl) else { return self.fail(i, op, "champ : région") };
                        let region = segs.last().cloned().unwrap_or_default(); let mut j = i + 2 + hl + nl + 1; let mut bit = 0usize;
                        while j < pe { match d[j] {
                            0x00 => { let Some((n, h)) = pkg(d, j + 1) else { break }; bit += n; j += 1 + h; }
                            0x01 => { j += 3; } 0x02 => { j += 1; } 0x03 => { j += 4; }
                            _ => { let Some(nm) = seg(d, j) else { break }; let Some((n, h)) = pkg(d, j + 4) else { break }; self.fields.push((region.clone(), nm, bit, n)); bit += n; j += 4 + h; } } }
                        Some(pe)
                    }
                    0x86 | 0x87 => { let Some((pl, _)) = pkg(d, i + 2) else { return self.fail(i, op, "champ") }; Some(i + 2 + pl) }
                    0x01 => { let Some((_, _, nl)) = name(d, i + 2) else { return self.fail(i, op, "mutex") }; Some(i + 2 + nl + 1) }
                    0x02 => { let Some((_, _, nl)) = name(d, i + 2) else { return self.fail(i, op, "event") }; Some(i + 2 + nl) }
                    0x88 => { let Some((_, _, nl)) = name(d, i + 2) else { return self.fail(i, op, "dataregion") }; let mut p = P { d, margs: self.margs.clone(), unresolved: vec![] }; let mut j = i + 2 + nl; for _ in 0..3 { match p.term(j, end, 0) { Some((_, nj)) => j = nj, None => return self.fail(i, op, "dataregion") } } Some(j) }
                    _ => self.generic(d, i, end, op),
                }
            }
            0x08 => {
                let Some((pre, segs, nl)) = name(d, i + 1) else { return self.fail(i, op, "Name illisible") };
                let mut p = P { d, margs: self.margs.clone(), unresolved: vec![] };
                let Some((t, j)) = p.term(i + 1 + nl, end, 0) else { return self.fail(i, op, "valeur de Name illisible") };
                let val = match &t { T::Int(v) => Val::Int(*v), T::Str(s) => Val::Str(s.clone()), T::Buf(b) => Val::Buf(b.clone()), _ => Val::Other };
                let full = join(scope, &pre, &segs); self.names.insert(path_str(&full), val.clone());
                if let (Some(c), true) = (cur, pre.is_empty() && segs.len() == 1) {
                    if let Some(dev) = self.devices.get_mut(c) { let n = segs[0].as_str();
                        let txt = |v: &Val| match v { Val::Int(x) => Some(if *x > 0xFFFF { eisa(*x) } else { format!("{x}") }), Val::Str(s) => Some(s.clone()), _ => None };
                        match (n, &val) { ("_HID", v) => dev.hid = txt(v), ("_CID", v) => dev.cid = txt(v), ("_UID", v) => dev.uid = txt(v), ("_ADR", Val::Int(x)) => dev.adr = Some(*x),
                            (_, Val::Buf(b)) => { let r = resources(b); if r.iter().any(|x| !matches!(x, Res::Other(_))) { dev.res.push((n.to_string(), r, false)); } } _ => {} } }
                }
                Some(j)
            }
            0x14 => {
                let Some((pl, hl)) = pkg(d, i + 1) else { return self.fail(i, op, "Method illisible") };
                let Some((pre, segs, nl)) = name(d, i + 1 + hl) else { return self.fail(i, op, "nom de Method") };
                let flags = *d.get(i + 1 + hl + nl)?; self.methods_seen += 1;
                let last = segs.last().cloned().unwrap_or_default(); self.margs.insert(last.clone(), flags & 7);
                if let (Some(c), true) = (cur, pre.is_empty() && segs.len() == 1) { if last == "_CRS" || last == "_PRS" { self.crs.push((c.to_string(), last, tix, i + 1 + hl + nl + 1, i + 1 + pl)); } }
                Some(i + 1 + pl)
            }
            0x06 => { let Some((p1, s1, a)) = name(d, i + 1) else { return self.fail(i, op, "Alias") }; let Some((p2, s2, b)) = name(d, i + 1 + a) else { return self.fail(i, op, "Alias") };
                self.aliases.insert(path_str(&join(scope, &p2, &s2)), path_str(&join(scope, &p1, &s1))); Some(i + 1 + a + b) }
            0x15 => { let Some((_, segs, a)) = name(d, i + 1) else { return self.fail(i, op, "External") }; let argc = *d.get(i + 1 + a + 1)?; if let Some(l) = segs.last() { self.margs.entry(l.clone()).or_insert(argc & 7); } Some(i + 1 + a + 2) }
            0xA0 | 0xA1 => { // bloc conditionnel de définitions : on y recherche aussi des Device/Name (marqués conditionnels)
                let Some((pl, hl)) = pkg(d, i + 1) else { return self.fail(i, op, "If") };
                let pe = i + 1 + pl; if pe > end { return self.fail(i, op, "If hors limites"); }
                self.skipped.push((i, op));
                let mut start = i + 1 + hl;
                if op == 0xA0 { let mut p = P { d, margs: self.margs.clone(), unresolved: vec![] }; match p.term(start, pe, 0) { Some((_, j)) => start = j, None => return Some(pe) } }
                self.tables[tix] = d.to_vec(); self.walk(tix, start, pe, scope, cur, true); self.tables[tix] = Vec::new();
                Some(pe)
            }
            0x00 => Some(i + 1),
            _ => self.generic(d, i, end, op),
        }
    }
    /// Terme quelconque à la racine ou dans une portée (Store, Notify, appel…) : analysé puis ignoré ; s'il est illisible, la portée s'arrête.
    fn generic(&mut self, d: &[u8], i: usize, end: usize, op: u8) -> Option<usize> {
        let mut p = P { d, margs: self.margs.clone(), unresolved: vec![] };
        match p.term(i, end, 0) { Some((_, j)) => { self.skipped.push((i, op)); Some(j) } None => self.fail(i, op, "opcode non géré") }
    }

    /// Passe 1 : une table complète. À appeler pour TOUTES les tables avant `resolve`.
    pub fn add_table(&mut self, t: &[u8]) { if t.len() <= 36 { return; } self.tables.push(t.to_vec()); let tix = self.tables.len() - 1; self.walk(tix, 36, t.len(), &[], None, false); }

    /// Passe 2 : analyse et évalue statiquement les corps de _CRS/_PRS.
    pub fn resolve(&mut self) {
        let jobs = std::mem::take(&mut self.crs);
        for (dev, mname, tix, start, end) in jobs {
            let d = self.tables[tix].clone(); let margs = self.margs.clone();
            let mut p = P { d: &d, margs: margs.clone(), unresolved: vec![] }; let mut stmts = vec![]; let mut j = start; let mut ok = true;
            while j < end { match p.term(j, end, 0) { Some((t, nj)) => { stmts.push(t); j = nj; } None => { ok = false; break } } }
            let scope: Vec<String> = dev.trim_start_matches('\\').split('.').map(String::from).collect();
            let (returns, conditional, notes) = { let mut ev = Ev { ns: self, scope, notes: vec![], returns: vec![], conditional: false, named: BTreeMap::new() }; let mut locals = Locals::new(); ev.run(&stmts, &mut locals); (ev.returns, ev.conditional, ev.notes) };
            if !ok { self.notes.push(format!("{dev}.{mname} : corps de méthode partiellement lu")); }
            for n in notes { self.notes.push(format!("{dev}.{mname} : {n}")); }
            let multi = returns.len() > 1 || conditional;
            let mut seen: Vec<Vec<u8>> = vec![]; for b in returns { if seen.contains(&b) { continue; } seen.push(b.clone()); let r = resources(&b);
                if let Some(dv) = self.devices.get_mut(&dev) { dv.res.push((format!("{mname}()"), r, multi)); } }
        }
    }
}

/// Raccourci pour un test ou un usage simple : une table.
pub fn parse_table(t: &[u8], ns: &mut Namespace) { ns.add_table(t); ns.resolve(); }

#[cfg(test)]
mod tests {
    use super::*;
    fn pk(body: &[u8]) -> Vec<u8> { let l = body.len() + 1; let mut v = if l < 64 { vec![l as u8] } else { let t = body.len() + 2; assert!(t < 4096); vec![0x40 | (t & 0x0F) as u8, (t >> 4) as u8] }; v.extend_from_slice(body); v }
    fn table(body: Vec<u8>) -> Vec<u8> { let mut t = vec![0u8; 36]; t.extend(body); t }
    fn dev(nm: &[u8; 4], inner: Vec<u8>) -> Vec<u8> { let mut b = nm.to_vec(); b.extend(inner); let mut v = vec![0x5B, 0x82]; v.extend(pk(&b)); v }
    fn name_buf(nm: &[u8; 4], buf: &[u8]) -> Vec<u8> { let mut v = vec![0x08]; v.extend_from_slice(nm); v.push(0x11); let mut b = vec![0x0A, buf.len() as u8]; b.extend_from_slice(buf); v.extend(pk(&b)); v }
    const KBD: [u8; 13] = [0x47, 1, 0x60, 0, 0x60, 0, 1, 1, 0x22, 0x02, 0x00, 0x79, 0];
    #[test] fn eisa_ids_decode() { assert_eq!(eisa(u32::from_le_bytes([0x41, 0xD0, 0x03, 0x03]) as u64), "PNP0303"); assert_eq!(eisa(u32::from_le_bytes([0x41, 0xD0, 0x0C, 0x50]) as u64), "PNP0C50"); }
    #[test] fn resource_descriptors_decode() {
        let b = [0x47, 1, 0x60, 0, 0x60, 0, 1, 1, 0x47, 1, 0x64, 0, 0x64, 0, 1, 1, 0x22, 0x02, 0x00, 0x86, 9, 0, 1, 0x00, 0x50, 0xDC, 0xFE, 0x00, 0x10, 0, 0, 0x79, 0];
        let r = resources(&b); assert_eq!(r[0], Res::IoRange { min: 0x60, max: 0x60, len: 1 }); assert_eq!(r[2], Res::Irq(vec![1])); assert_eq!(r[3], Res::Mem32 { base: 0xFEDC_5000, len: 0x1000 }); assert_eq!(r.len(), 4);
    }
    #[test] fn static_name_buffer_is_decoded() {
        let mut inner = vec![0x08, b'_', b'H', b'I', b'D', 0x0C, 0x41, 0xD0, 0x03, 0x03]; inner.extend(name_buf(b"RBUF", &KBD));
        let mut ns = Namespace::default(); parse_table(&table(dev(b"PS2K", inner)), &mut ns);
        let d = &ns.devices["\\PS2K"]; assert_eq!(d.hid.as_deref(), Some("PNP0303")); assert_eq!(d.res[0].1[1], Res::Irq(vec![1])); assert_eq!(ns.unknown, 0);
    }
    #[test] fn crs_method_returning_a_named_buffer_is_resolved() {
        let mut inner = name_buf(b"SBUF", &KBD);
        let mut m = b"_CRS".to_vec(); m.push(0x00); m.push(0xA4); m.extend_from_slice(b"SBUF"); let mut meth = vec![0x14]; meth.extend(pk(&m)); inner.extend(meth);
        let mut ns = Namespace::default(); parse_table(&table(dev(b"DEV0", inner)), &mut ns);
        let d = &ns.devices["\\DEV0"]; assert!(d.res.iter().any(|(n, r, c)| n == "_CRS()" && r[0] == Res::IoRange { min: 0x60, max: 0x60, len: 1 } && !*c), "{:?}", d.res);
    }
    #[test] fn concatenate_res_template_and_if_else_branches_are_both_reported() {
        let a = [0x47u8, 1, 0x60, 0, 0x60, 0, 1, 1, 0x79, 0]; let b = [0x22u8, 0x02, 0x00, 0x79, 0];
        let mut inner = name_buf(b"BUFA", &a); inner.extend(name_buf(b"BUFB", &b));
        // Method _CRS { If (Arg-free constant 1) { Return (ConcatenateResTemplate(BUFA, BUFB)) } Else { Return (BUFB) } }
        let mut ret1 = vec![0xA4, 0x84]; ret1.extend_from_slice(b"BUFA"); ret1.extend_from_slice(b"BUFB"); ret1.push(0x00);
        let mut ifb = vec![0x01]; ifb.extend(ret1); let mut ifop = vec![0xA0]; ifop.extend(pk(&ifb));
        let mut ret2 = vec![0xA4]; ret2.extend_from_slice(b"BUFB"); let mut elsop = vec![0xA1]; elsop.extend(pk(&ret2));
        let mut m = b"_CRS".to_vec(); m.push(0x00); m.extend(ifop); m.extend(elsop); let mut meth = vec![0x14]; meth.extend(pk(&m)); inner.extend(meth);
        let mut ns = Namespace::default(); parse_table(&table(dev(b"DEV1", inner)), &mut ns);
        let crs: Vec<_> = ns.devices["\\DEV1"].res.iter().filter(|x| x.0 == "_CRS()").collect(); assert_eq!(crs.len(), 2, "deux retours possibles"); assert!(crs.iter().all(|x| x.2), "marqués conditionnels");
        assert!(crs.iter().any(|x| x.1.len() == 2 && x.1[0] == Res::IoRange { min: 0x60, max: 0x60, len: 1 } && x.1[1] == Res::Irq(vec![1])), "la concaténation retire l'étiquette de fin de la première");
    }
    #[test] fn expression_operation_region_and_root_if_do_not_stop_the_walk() {
        // OperationRegion(REG0, SystemMemory, Add(0x10, 0x20), 0x1000) puis un Device : doit être trouvé
        let mut o = vec![0x5B, 0x80]; o.extend_from_slice(b"REG0"); o.push(0); o.extend([0x72, 0x0A, 0x10, 0x0A, 0x20, 0x00]); o.extend([0x0B, 0x00, 0x10]);
        let mut body = o; body.extend(dev(b"AFTR", vec![0x08, b'_', b'U', b'I', b'D', 0x0A, 0x07]));
        let mut ns = Namespace::default(); parse_table(&table(body), &mut ns); assert_eq!(ns.unknown, 0, "{:?}", ns.notes); assert_eq!(ns.devices["\\AFTR"].uid.as_deref(), Some("7"));
    }
    /// Arité vérifiée contre ACPICA (ARGP_*) : un terme doit consommer EXACTEMENT ses octets, sinon le suivant serait mal lu.
    #[test] fn term_arity_consumes_exactly_its_bytes() {
        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("Match (4 termes + 2 octets)", vec![0x89, 0x60, 0x01, 0x0A, 0x05, 0x01, 0x0A, 0x06, 0x00]),
            ("LoadTable (6 termes)", vec![0x5B, 0x1F, 0x0D, b'A', 0, 0x0D, b'B', 0, 0x0D, b'C', 0, 0x00, 0x00, 0x00]),
            ("Divide (4)", vec![0x78, 0x0A, 9, 0x0A, 2, 0x00, 0x00]), ("Mid (4)", vec![0x9E, 0x60, 0x00, 0x01, 0x00]), ("ToString (3)", vec![0x9C, 0x60, 0x00, 0x00]),
            ("LNotEqual (92 93, 2 termes)", vec![0x92, 0x93, 0x60, 0x61]), ("LNot (1)", vec![0x92, 0x60]), ("Index (3)", vec![0x88, 0x60, 0x00, 0x00]),
            ("CondRefOf (2)", vec![0x5B, 0x12, b'F', b'O', b'O', b'_', 0x00]), ("Acquire (1 + mot)", vec![0x5B, 0x23, 0x60, 0xFF, 0xFF]),
            ("CreateDWordField (2 + nom)", vec![0x8A, 0x60, 0x0A, 0x04, b'F', b'L', b'D', b'_']), ("Fatal (octet, dword, terme)", vec![0x5B, 0x32, 1, 0, 0, 0, 0, 0x00]),
        ];
        for (nom, bytes) in cases { let mut p = P { d: &bytes, margs: BTreeMap::new(), unresolved: vec![] }; let r = p.term(0, bytes.len(), 0); assert_eq!(r.map(|x| x.1), Some(bytes.len()), "{nom}"); }
    }
    #[test] fn unknown_bytes_are_counted_never_guessed() { let mut ns = Namespace::default(); parse_table(&table(vec![0xA7, 0x00]), &mut ns); assert_eq!(ns.unknown, 1); }
    #[test] fn truncated_input_never_panics() {
        let mut inner = name_buf(b"SBUF", &KBD); let mut m = b"_CRS".to_vec(); m.push(0x00); m.push(0xA4); m.extend_from_slice(b"SBUF"); let mut meth = vec![0x14]; meth.extend(pk(&m)); inner.extend(meth);
        let t = table(dev(b"DEV0", inner)); for cut in 0..t.len() { let mut ns = Namespace::default(); parse_table(&t[..cut], &mut ns); }
    }
}
