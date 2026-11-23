// Vefi.map(|o| (o.main_ret, String::from_utf8_lossy(&o.serial).to_string(), o.violations, o.tones, o.fb == run.fb, o.pcm == run.pcm, o.plays, o.hda_errors))xa : langage minimal, de zero, pour le logiciel ET le materiel.
// - Interpreteur (reference de semantique) et compilateur x86-64 natif sans dependance.
// - Regle d'accessibilite imposee par le compilateur : toute fonction `pub` porte une description parlee.
// - Messages d'erreur en phrases simples, une par ligne, avec numero de ligne.
use std::collections::HashMap;
use std::{env, fs};
mod emu;

// ---------------------------------------------------------------- lexique
#[derive(Debug, Clone, PartialEq)]
enum Tok { Num(i64), Str(String), Id(String), P(&'static str), Eof }
#[derive(Debug, Clone)]
struct T { t: Tok, line: usize }

const P2: [&str; 5] = ["==", "!=", "<=", ">=", "->"];
const P1: [&str; 16] = ["+", "-", "*", "/", "%", "<", ">", "=", "(", ")", "{", "}", ",", ";", ":", "!"];

fn lex(src: &str) -> Result<Vec<T>, String> {
    let b: Vec<char> = src.chars().collect();
    let (mut i, mut line, mut out) = (0usize, 1usize, Vec::new());
    while i < b.len() {
        let c = b[i];
        if c == '\n' { line += 1; i += 1; continue; }
        if c.is_whitespace() { i += 1; continue; }
        if c == '/' && i + 1 < b.len() && b[i + 1] == '/' { while i < b.len() && b[i] != '\n' { i += 1; } continue; }
        if c.is_ascii_digit() {
            if c == '0' && i + 1 < b.len() && (b[i + 1] == 'x' || b[i + 1] == 'X') {
                i += 2; let st = i;
                while i < b.len() && b[i].is_ascii_hexdigit() { i += 1; }
                let s: String = b[st..i].iter().collect();
                let v = i64::from_str_radix(&s, 16).map_err(|_| format!("Ligne {line} : nombre hexadécimal invalide."))?;
                out.push(T { t: Tok::Num(v), line }); continue;
            }
            let st = i;
            while i < b.len() && b[i].is_ascii_digit() { i += 1; }
            let s: String = b[st..i].iter().collect();
            let v = s.parse::<i64>().map_err(|_| format!("Ligne {line} : nombre trop grand."))?;
            out.push(T { t: Tok::Num(v), line }); continue;
        }
        if c.is_alphabetic() || c == '_' {
            let st = i;
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') { i += 1; }
            out.push(T { t: Tok::Id(b[st..i].iter().collect()), line }); continue;
        }
        if c == '"' {
            i += 1; let mut s = String::new();
            loop {
                if i >= b.len() { return Err(format!("Ligne {line} : texte non fermé, il manque un guillemet.")); }
                match b[i] {
                    '"' => { i += 1; break; }
                    '\\' if i + 1 < b.len() => { s.push(match b[i + 1] { 'n' => '\n', 't' => '\t', o => o }); i += 2; }
                    '\n' => { line += 1; s.push('\n'); i += 1; }
                    o => { s.push(o); i += 1; }
                }
            }
            out.push(T { t: Tok::Str(s), line }); continue;
        }
        let two: String = b[i..(i + 2).min(b.len())].iter().collect();
        if let Some(p) = P2.iter().find(|p| **p == two) { out.push(T { t: Tok::P(p), line }); i += 2; continue; }
        if let Some(p) = P1.iter().find(|p| p.chars().next() == Some(c)) { out.push(T { t: Tok::P(p), line }); i += 1; continue; }
        return Err(format!("Ligne {line} : caractère inconnu « {c} »."));
    }
    out.push(T { t: Tok::Eof, line });
    Ok(out)
}

// ---------------------------------------------------------------- arbre
#[derive(Debug, Clone)]
enum Expr { Str(String), Num(i64), Var(String, usize), Bin(&'static str, Box<Expr>, Box<Expr>), Neg(Box<Expr>), Not(Box<Expr>), Call(String, Vec<Expr>, usize) }
#[derive(Debug, Clone)]
enum Stmt { Let(String, Expr), Set(String, Expr, usize), If(Expr, Vec<Stmt>, Vec<Stmt>), While(Expr, Vec<Stmt>), Ret(Option<Expr>), Say(String), Ex(Expr) }
#[derive(Debug, Clone)]
struct Func { name: String, params: Vec<String>, doc: Option<String>, public: bool, body: Vec<Stmt>, line: usize }

struct Parser { t: Vec<T>, p: usize }
impl Parser {
    fn cur(&self) -> &T { &self.t[self.p] }
    fn line(&self) -> usize { self.cur().line }
    fn is_p(&self, s: &str) -> bool { matches!(&self.cur().t, Tok::P(x) if *x == s) }
    fn is_id(&self, s: &str) -> bool { matches!(&self.cur().t, Tok::Id(x) if x == s) }
    fn eat_p(&mut self, s: &str) -> bool { if self.is_p(s) { self.p += 1; true } else { false } }
    fn want_p(&mut self, s: &str) -> Result<(), String> {
        if self.eat_p(s) { Ok(()) } else { Err(format!("Ligne {} : j'attendais « {} ».", self.line(), s)) }
    }
    fn ident(&mut self) -> Result<String, String> {
        if let Tok::Id(s) = self.cur().t.clone() { self.p += 1; Ok(s) } else { Err(format!("Ligne {} : j'attendais un nom.", self.line())) }
    }
    fn program(&mut self) -> Result<Vec<Func>, String> {
        let mut v = Vec::new();
        while self.cur().t != Tok::Eof { v.push(self.func()?); }
        Ok(v)
    }
    fn func(&mut self) -> Result<Func, String> {
        let line = self.line();
        let public = if self.is_id("pub") { self.p += 1; true } else { false };
        if !self.is_id("fn") { return Err(format!("Ligne {line} : j'attendais le mot « fn »; une fonction commence par « fn » ou « pub fn »." )); }
        self.p += 1;
        let name = self.ident()?;
        self.want_p("(")?;
        let mut params = Vec::new();
        while !self.is_p(")") {
            params.push(self.ident()?);
            if self.eat_p(":") { let ty = self.ident()?; if ty != "int" { return Err(format!("Ligne {}: seul le type « int » existe pour l'instant.", self.line())); } }
            if !self.eat_p(",") { break; }
        }
        self.want_p(")")?;
        let doc = if let Tok::Str(s) = self.cur().t.clone() { self.p += 1; Some(s) } else { None };
        if self.eat_p("->") { let ty = self.ident()?; if ty != "int" { return Err(format!("Ligne {}: seul le type « int » existe pour l'instant.", self.line())); } }
        let body = self.block()?;
        Ok(Func { name, params, doc, public, body, line })
    }
    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.want_p("{")?;
        let mut v = Vec::new();
        while !self.is_p("}") {
            if self.cur().t == Tok::Eof { return Err(format!("Ligne {} : bloc non fermé, il manque « }}».", self.line())); }
            v.push(self.stmt()?);
        }
        self.want_p("}")?;
        Ok(v)
    }
    fn stmt(&mut self) -> Result<Stmt, String> {
        let line = self.line();
        if self.is_id("let") {
            self.p += 1; let n = self.ident()?; self.want_p("=")?; let e = self.expr()?; self.want_p(";")?; return Ok(Stmt::Let(n, e));
        }
        if self.is_id("if") {
            self.p += 1; let c = self.expr()?; let a = self.block()?;
            let b = if self.is_id("else") { self.p += 1; if self.is_id("if") { vec![self.stmt()?] } else { self.block()? } } else { vec![] };
            return Ok(Stmt::If(c, a, b));
        }
        if self.is_id("while") { self.p += 1; let c = self.expr()?; let b = self.block()?; return Ok(Stmt::While(c, b)); }
        if self.is_id("return") {
            self.p += 1;
            let e = if self.is_p(";") { None } else { Some(self.expr()?) };
            self.want_p(";")?; return Ok(Stmt::Ret(e));
        }
        // annonces accessibles VX1 : "speak" (parole) et "alert" (alerte), voir VOXA-ACCES.md
        for (kw, kind) in [("speak", "parole"), ("alert", "alerte")] {
            if self.is_id(kw) {
                self.p += 1;
                if let Tok::Str(s) = self.cur().t.clone() { self.p += 1; self.want_p(";")?; return Ok(Stmt::Say(format!("VX1 {kind}: {s}\n"))); }
                return Err(format!("Ligne {line} : « {kw} » attend un texte entre guillemets."));
            }
        }
        if self.is_id("say") {
            self.p += 1;
            if let Tok::Str(s) = self.cur().t.clone() { self.p += 1; self.want_p(";")?; return Ok(Stmt::Say(s)); }
            return Err(format!("Ligne {line} : « say » attend un texte entre guillemets."));
        }
        // affectation : nom = expr ;
        if let (Tok::Id(n), Tok::P("=")) = (&self.cur().t, &self.t[self.p + 1].t) {
            let n = n.clone(); self.p += 2; let e = self.expr()?; self.want_p(";")?; return Ok(Stmt::Set(n, e, line));
        }
        let e = self.expr()?; self.want_p(";")?; Ok(Stmt::Ex(e))
    }
    fn expr(&mut self) -> Result<Expr, String> {
        let mut l = self.add()?;
        loop {
            let op = ["==", "!=", "<=", ">=", "<", ">"].into_iter().find(|o| self.is_p(o));
            match op { Some(o) => { self.p += 1; let r = self.add()?; l = Expr::Bin(o, Box::new(l), Box::new(r)); } None => return Ok(l) }
        }
    }
    fn add(&mut self) -> Result<Expr, String> {
        let mut l = self.mul()?;
        loop {
            let op = ["+", "-"].into_iter().find(|o| self.is_p(o));
            match op { Some(o) => { self.p += 1; let r = self.mul()?; l = Expr::Bin(o, Box::new(l), Box::new(r)); } None => return Ok(l) }
        }
    }
    fn mul(&mut self) -> Result<Expr, String> {
        let mut l = self.unary()?;
        loop {
            let op = ["*", "/", "%"].into_iter().find(|o| self.is_p(o));
            match op { Some(o) => { self.p += 1; let r = self.unary()?; l = Expr::Bin(o, Box::new(l), Box::new(r)); } None => return Ok(l) }
        }
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if self.eat_p("-") { return Ok(Expr::Neg(Box::new(self.unary()?))); }
        if self.eat_p("!") { return Ok(Expr::Not(Box::new(self.unary()?))); }
        self.primary()
    }
    fn primary(&mut self) -> Result<Expr, String> {
        let line = self.line();
        match self.cur().t.clone() {
            Tok::Num(n) => { self.p += 1; Ok(Expr::Num(n)) }
            Tok::Str(s) => { self.p += 1; Ok(Expr::Str(s)) }
            Tok::P("(") => { self.p += 1; let e = self.expr()?; self.want_p(")")?; Ok(e) }
            Tok::Id(n) => {
                self.p += 1;
                if self.eat_p("(") {
                    let mut a = Vec::new();
                    while !self.is_p(")") { a.push(self.expr()?); if !self.eat_p(",") { break; } }
                    self.want_p(")")?; Ok(Expr::Call(n, a, line))
                } else { Ok(Expr::Var(n, line)) }
            }
            _ => Err(format!("Ligne {line} : j'attendais une valeur, un nom ou une parenthèse.")),
        }
    }
}

// ---------------------------------------------------------------- regles (accessibilite + noms)
const BUILTINS: [(&str, usize); 19] = [("cpuid", 2), ("systab", 0), ("out32", 2), ("in32", 1), ("peek16", 1), ("poke16", 2), ("out", 2), ("inb", 1), ("peek", 1), ("poke", 2), ("readkey", 0), ("reset", 1), ("peek8", 1), ("poke8", 2), ("peek32", 1), ("poke32", 2), ("fbinfo", 1), ("scratch", 0), ("stall", 1)];

fn check(funcs: &[Func]) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for f in funcs {
        if seen.insert(&f.name, f.line).is_some() { errs.push(format!("Ligne {} : la fonction « {} » existe déjà.", f.line, f.name)); }
        if f.public && f.doc.as_deref().map(|d| d.trim().is_empty()).unwrap_or(true) {
            errs.push(format!("Ligne {} : la fonction publique « {} » n'a pas de description parlée. Ajoutez un texte entre guillemets après les parenthèses.", f.line, f.name));
        }
    }
    if !funcs.iter().any(|f| f.name == "main" && f.params.is_empty()) { errs.push("Il manque la fonction « main » sans paramètre.".into()); }
    if errs.is_empty() { Ok(()) } else { Err(errs) }
}

// ---------------------------------------------------------------- interpreteur (reference)
enum Flow { Next, Ret(i64) }
struct Interp<'a> { funcs: HashMap<&'a str, &'a Func>, mem: HashMap<i64, u8>, plat: emu::Platform, fb: Vec<u8>, strs: HashMap<String, i64>, nstr: i64, steps: u64, keys: std::collections::VecDeque<i64> }
const STR_BASE: i64 = 0x2000_0000;
const SCRATCH_ADDR: i64 = 0x3000_0000;
impl<'a> Interp<'a> {
    fn byte(&self, a: i64) -> u8 {
        let fb0 = emu::FB_BASE as i64;
        if a >= fb0 && ((a - fb0) as usize) < self.fb.len() { self.fb[(a - fb0) as usize] } else { *self.mem.get(&a).unwrap_or(&0) }
    }
    fn rd(&self, a: i64, n: usize) -> i64 {
        if let Some((i, off, en)) = self.plat.mmio_window(a as u64) { return if en { self.plat.mmio_read(i, off, n) as i64 } else { 0xFFFF_FFFF }; }
        let mut v = 0u64; for k in 0..n { v |= (self.byte(a.wrapping_add(k as i64)) as u64) << (8 * k); } v as i64 }
    fn wr(&mut self, a: i64, n: usize, v: i64) {
        if let Some((i, off, en)) = self.plat.mmio_window(a as u64) {
            if en { let mut p = std::mem::take(&mut self.plat); { let mut m = InterpMem { mem: &mut self.mem, fb: &mut self.fb }; p.mmio_write(i, off, n, v as u32, &mut m); } self.plat = p; }
            return;
        }
        let fb0 = emu::FB_BASE as i64;
        for k in 0..n {
            let b = ((v as u64) >> (8 * k)) as u8; let ad = a.wrapping_add(k as i64);
            if ad >= fb0 && ((ad - fb0) as usize) < self.fb.len() { self.fb[(ad - fb0) as usize] = b; } else { self.mem.insert(ad, b); }
        }
    }
    fn call(&mut self, name: &str, args: Vec<i64>) -> Result<i64, String> {
        if let Some(f) = self.funcs.get(name).copied() {
            if f.params.len() != args.len() { return Err(format!("« {} » attend {} valeur(s), reçu {}.", name, f.params.len(), args.len())); }
            let mut env: HashMap<String, i64> = f.params.iter().cloned().zip(args).collect();
            return Ok(match self.block(&f.body, &mut env)? { Flow::Ret(v) => v, Flow::Next => 0 });
        }
        match (name, args.as_slice()) {
            ("out", [p, v]) => { self.plat.out(*p as u16, *v as u8); Ok(0) }
            ("inb", [p]) => Ok(self.plat.inb(*p as u16) as i64),
            ("out32", [p, v]) => { self.plat.out32(*p as u16, *v as u32); Ok(0) }
            ("in32", [p]) => Ok(self.plat.in32(*p as u16) as i64),
            ("peek16", [a]) => Ok(self.rd(*a, 2)),
            ("poke16", [a, v]) => { self.wr(*a, 2, *v); Ok(0) }
            ("peek8", [a]) => Ok(self.rd(*a, 1)),
            ("peek32", [a]) => Ok(self.rd(*a, 4)),
            ("poke8", [a, v]) => { self.wr(*a, 1, *v); Ok(0) }
            ("poke32", [a, v]) => { self.wr(*a, 4, *v); Ok(0) }
            ("fbinfo", [n]) => Ok(self.plat.fb_info(*n)),
            ("scratch", []) => Ok(SCRATCH_ADDR),
            ("cpuid", [l, r]) => Ok(emu::cpuid_host(*l as u32, *r as usize) as i64),
            ("systab", []) => Ok(0),
            ("stall", [us]) => { self.plat.time_us += *us as u64; Ok(0) }
            ("readkey", []) => Ok(self.keys.pop_front().unwrap_or(-1)),
            ("reset", [k]) => Err(format!("RESET:{k}")),
            ("peek", [a]) => Ok(self.rd(*a, 8)),
            ("poke", [a, v]) => { self.wr(*a, 8, *v); Ok(0) }
            _ => Err(format!("Fonction inconnue « {name} ».")),
        }
    }
    fn block(&mut self, b: &[Stmt], env: &mut HashMap<String, i64>) -> Result<Flow, String> {
        for s in b {
            self.steps += 1;
            if self.steps > 200_000_000 { return Err("Trop d'étapes : boucle sans fin probable.".into()); }
            match s {
                Stmt::Let(n, e) => { let v = self.eval(e, env)?; env.insert(n.clone(), v); }
                Stmt::Set(n, e, l) => { let v = self.eval(e, env)?; if !env.contains_key(n) { return Err(format!("Ligne {l} : « {n} » n'est pas déclaré.")); } env.insert(n.clone(), v); }
                Stmt::If(c, a, b2) => { let br = if self.eval(c, env)? != 0 { a } else { b2 }; if let Flow::Ret(v) = self.block(br, env)? { return Ok(Flow::Ret(v)); } }
                Stmt::While(c, body) => { while self.eval(c, env)? != 0 { self.steps += 1; if self.steps > 200_000_000 { return Err("Trop d'étapes : boucle sans fin probable.".into()); } if let Flow::Ret(v) = self.block(body, env)? { return Ok(Flow::Ret(v)); } } }
                Stmt::Ret(e) => { let v = match e { Some(e) => self.eval(e, env)?, None => 0 }; return Ok(Flow::Ret(v)); }
                Stmt::Say(t) => { for b in t.bytes() { self.plat.serial.push(b); } }
                Stmt::Ex(e) => { self.eval(e, env)?; }
            }
        }
        Ok(Flow::Next)
    }
    fn eval(&mut self, e: &Expr, env: &mut HashMap<String, i64>) -> Result<i64, String> {
        Ok(match e {
            Expr::Num(n) => *n,
            Expr::Str(s) => { if let Some(a) = self.strs.get(s) { *a } else { let a = STR_BASE + self.nstr; let mut v = s.as_bytes().to_vec(); v.push(0); for (i, b) in v.iter().enumerate() { self.mem.insert(a + i as i64, *b); } self.nstr += v.len() as i64; self.strs.insert(s.clone(), a); a } }
            Expr::Var(n, l) => *env.get(n).ok_or_else(|| format!("Ligne {l} : « {n} » n'est pas déclaré."))?,
            Expr::Neg(x) => self.eval(x, env)?.wrapping_neg(),
            Expr::Not(x) => (self.eval(x, env)? == 0) as i64,
            Expr::Call(n, a, l) => { let mut v = Vec::new(); for x in a { v.push(self.eval(x, env)?); } self.call(n, v).map_err(|m| if m.starts_with("Ligne") || m.starts_with("RESET:") { m } else { format!("Ligne {l} : {m}") })? }
            Expr::Bin(op, a, b) => {
                let (x, y) = (self.eval(a, env)?, self.eval(b, env)?);
                match *op {
                    "+" => x.wrapping_add(y), "-" => x.wrapping_sub(y), "*" => x.wrapping_mul(y),
                    "/" => x.checked_div(y).ok_or("Division impossible (par zéro ou débordement).")?,
                    "%" => x.checked_rem(y).ok_or("Reste impossible (par zéro ou débordement).")?,
                    "==" => (x == y) as i64, "!=" => (x != y) as i64, "<" => (x < y) as i64,
                    "<=" => (x <= y) as i64, ">" => (x > y) as i64, ">=" => (x >= y) as i64,
                    _ => unreachable!(),
                }
            }
        })
    }
}

// ---------------------------------------------------------------- generateur x86-64
#[derive(Clone, Copy, PartialEq)]
enum Mode { Hosted, Bare, Efi }
// Disposition de l'image EFI : .data a la RVA 0x1000 (1 Mio, zero au chargement), .text a la RVA 0x101000.
const EFI_DATA: i64 = 0x1000;
const EFI_TEXT: i64 = 0x101000;
const SCRATCH_OFF: i64 = 0x1000;
struct Gen { code: Vec<u8>, offs: HashMap<String, usize>, fix: Vec<(usize, String, usize)>, sig: HashMap<String, usize>, loc: HashMap<String, i32>, nslots: i32, hw: bool, keyuse: bool, efi: bool, strs: Vec<Vec<u16>>, sfix: Vec<(usize, usize)>, blobs: Vec<Vec<u8>>, bfix: Vec<(usize, usize)> }
impl Gen {
    /// instruction rip-relative vers la zone .data : opcode + disp32 vers EFI_DATA+off
    fn data_ref(&mut self, op: &[u8], off: i64) {
        self.b(op); let p = self.code.len() as i64;
        let disp = (EFI_DATA + off) - (EFI_TEXT + p + 4);
        self.i32(disp as i32);
    }
    fn b(&mut self, x: &[u8]) { self.code.extend_from_slice(x); }
    fn i32(&mut self, v: i32) { self.code.extend_from_slice(&v.to_le_bytes()); }
    fn jump(&mut self, op: &[u8]) -> usize { self.b(op); let p = self.code.len(); self.i32(0); p }
    fn patch(&mut self, p: usize, target: usize) { let rel = (target as i64 - (p as i64 + 4)) as i32; self.code[p..p + 4].copy_from_slice(&rel.to_le_bytes()); }
    fn slot(&mut self, n: &str) -> i32 { self.nslots += 1; let d = -8 * self.nslots; self.loc.insert(n.to_string(), d); d }
    fn epilogue(&mut self) { self.b(&[0x48, 0x89, 0xEC, 0x5D, 0xC3]); }

    fn func(&mut self, f: &Func) -> Result<(), String> {
        if f.params.len() > 4 { return Err(format!("Ligne {} : « {} » a plus de 4 paramètres, limite actuelle.", f.line, f.name)); }
        self.offs.insert(f.name.clone(), self.code.len());
        self.loc.clear(); self.nslots = 0;
        self.b(&[0x55, 0x48, 0x89, 0xE5, 0x48, 0x81, 0xEC]);
        let fs = self.code.len(); self.i32(0);
        let st: [&[u8]; 4] = [&[0x48, 0x89, 0x8D], &[0x48, 0x89, 0x95], &[0x4C, 0x89, 0x85], &[0x4C, 0x89, 0x8D]];
        for (i, p) in f.params.iter().enumerate() { let d = self.slot(p); self.b(st[i]); self.i32(d); }
        self.block(&f.body)?;
        self.b(&[0x31, 0xC0]); self.epilogue();
        let size = ((self.nslots * 8 + 15) / 16 * 16).max(16);
        self.code[fs..fs + 4].copy_from_slice(&size.to_le_bytes());
        Ok(())
    }
    fn block(&mut self, b: &[Stmt]) -> Result<(), String> { for s in b { self.stmt(s)?; } Ok(()) }
    fn stmt(&mut self, s: &Stmt) -> Result<(), String> {
        match s {
            Stmt::Let(n, e) => { self.expr(e)?; let d = self.slot(n); self.b(&[0x48, 0x89, 0x85]); self.i32(d); }
            Stmt::Set(n, e, l) => { self.expr(e)?; let d = *self.loc.get(n).ok_or(format!("Ligne {l} : « {n} » n'est pas déclaré."))?; self.b(&[0x48, 0x89, 0x85]); self.i32(d); }
            Stmt::If(c, a, e) => {
                self.expr(c)?; self.b(&[0x48, 0x85, 0xC0]);
                let jz = self.jump(&[0x0F, 0x84]);
                self.block(a)?;
                if e.is_empty() { let t = self.code.len(); self.patch(jz, t); }
                else { let je = self.jump(&[0xE9]); let t = self.code.len(); self.patch(jz, t); self.block(e)?; let t2 = self.code.len(); self.patch(je, t2); }
            }
            Stmt::While(c, body) => {
                let top = self.code.len();
                self.expr(c)?; self.b(&[0x48, 0x85, 0xC0]);
                let jz = self.jump(&[0x0F, 0x84]);
                self.block(body)?;
                let j = self.jump(&[0xE9]); self.patch(j, top);
                let t = self.code.len(); self.patch(jz, t);
            }
            Stmt::Ret(e) => { match e { Some(e) => self.expr(e)?, None => self.b(&[0x31, 0xC0]) } self.epilogue(); }
            Stmt::Say(t) => {
                self.hw = true;
                for byte in t.bytes() { self.b(&[0x66, 0xBA, 0xF8, 0x03, 0xB0, byte, 0xEE]); }
                if self.efi {
                    let mut u: Vec<u16> = Vec::new();
                    for ch in t.chars() { if ch == '\n' { u.push(13); u.push(10); } else { let mut b = [0u16; 2]; u.extend_from_slice(ch.encode_utf16(&mut b)); } }
                    u.push(0);
                    let idx = self.strs.len(); self.strs.push(u);
                    // aligne la pile sur 16, reserve 32 octets d'ombre + 16 pour sauvegarder rsp
                    self.b(&[0x48, 0x89, 0xE0, 0x48, 0x83, 0xE4, 0xF0, 0x48, 0x83, 0xEC, 0x30, 0x48, 0x89, 0x44, 0x24, 0x20]);
                    self.data_ref(&[0x48, 0x8B, 0x0D], 0);          // rcx = SystemTable
                    self.b(&[0x48, 0x8B, 0x49, 0x40]);              // rcx = SystemTable->ConOut
                    self.b(&[0x48, 0x85, 0xC9]);                    // test rcx, rcx : pas de console => on saute l'appel
                    let jnull = self.jump(&[0x0F, 0x84]);
                    self.b(&[0x48, 0x8D, 0x15]); let p = self.code.len(); self.i32(0); self.sfix.push((p, idx)); // rdx = &texte UTF-16
                    self.b(&[0x48, 0x8B, 0x41, 0x08, 0xFF, 0xD0]);  // call ConOut->OutputString
                    let here = self.code.len(); self.patch(jnull, here);
                    self.b(&[0x48, 0x8B, 0x64, 0x24, 0x20]);        // rsp restaure
                }
            }
            Stmt::Ex(e) => self.expr(e)?,
        }
        Ok(())
    }
    fn expr(&mut self, e: &Expr) -> Result<(), String> {
        match e {
            Expr::Num(n) => { self.b(&[0x48, 0xB8]); self.code.extend_from_slice(&n.to_le_bytes()); }
            Expr::Str(s) => { let idx = self.blobs.len(); let mut v = s.as_bytes().to_vec(); v.push(0); self.blobs.push(v); self.b(&[0x48, 0x8D, 0x05]); let p = self.code.len(); self.i32(0); self.bfix.push((p, idx)); }
            Expr::Var(n, l) => { let d = *self.loc.get(n).ok_or(format!("Ligne {l} : « {n} » n'est pas déclaré."))?; self.b(&[0x48, 0x8B, 0x85]); self.i32(d); }
            Expr::Neg(x) => { self.expr(x)?; self.b(&[0x48, 0xF7, 0xD8]); }
            Expr::Not(x) => { self.expr(x)?; self.b(&[0x48, 0x85, 0xC0, 0x0F, 0x94, 0xC0, 0x0F, 0xB6, 0xC0]); }
            Expr::Bin(op, a, b) => {
                self.expr(a)?; self.b(&[0x50]); self.expr(b)?; self.b(&[0x48, 0x89, 0xC1, 0x58]); // rcx=droite, rax=gauche
                match *op {
                    "+" => self.b(&[0x48, 0x01, 0xC8]),
                    "-" => self.b(&[0x48, 0x29, 0xC8]),
                    "*" => self.b(&[0x48, 0x0F, 0xAF, 0xC1]),
                    "/" => self.b(&[0x48, 0x99, 0x48, 0xF7, 0xF9]),
                    "%" => self.b(&[0x48, 0x99, 0x48, 0xF7, 0xF9, 0x48, 0x89, 0xD0]),
                    _ => {
                        let cc = match *op { "==" => 0x94, "!=" => 0x95, "<" => 0x9C, "<=" => 0x9E, ">" => 0x9F, ">=" => 0x9D, _ => unreachable!() };
                        self.b(&[0x48, 0x39, 0xC8, 0x0F, cc, 0xC0, 0x0F, 0xB6, 0xC0]);
                    }
                }
            }
            Expr::Call(n, a, l) => {
                let want = BUILTINS.iter().find(|(b, _)| b == n).map(|(_, k)| *k).or_else(|| self.sig.get(n).copied())
                    .ok_or(format!("Ligne {l} : fonction inconnue « {n} »."))?;
                if a.len() != want { return Err(format!("Ligne {l} : « {n} » attend {want} valeur(s), reçu {}.", a.len())); }
                match n.as_str() {
                    "out" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x50]); self.expr(&a[1])?; self.b(&[0x5A, 0xEE, 0x31, 0xC0]); }
                    "inb" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x48, 0x89, 0xC2, 0x31, 0xC0, 0xEC]); }
                    "peek" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x48, 0x8B, 0x00]); }
                    "out32" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x50]); self.expr(&a[1])?; self.b(&[0x5A, 0xEF, 0x31, 0xC0]); }
                    "in32" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x48, 0x89, 0xC2, 0x31, 0xC0, 0xED]); }
                    "peek16" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x48, 0x0F, 0xB7, 0x00]); }
                    "poke16" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x50]); self.expr(&a[1])?; self.b(&[0x59, 0x66, 0x89, 0x01, 0x31, 0xC0]); }
                    "peek8" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x48, 0x0F, 0xB6, 0x00]); }
                    "peek32" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x8B, 0x00]); }
                    "poke8" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x50]); self.expr(&a[1])?; self.b(&[0x59, 0x88, 0x01, 0x31, 0xC0]); }
                    "poke32" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x50]); self.expr(&a[1])?; self.b(&[0x59, 0x89, 0x01, 0x31, 0xC0]); }
                    "scratch" => {
                        // bloc de travail de 16 Kio hors pile : zone .data en UEFI, adresse fixe 0x20000 en image nue
                        self.hw = true;
                        if self.efi { self.data_ref(&[0x48, 0x8D, 0x05], SCRATCH_OFF); } else { self.b(&[0x48, 0xB8]); self.code.extend_from_slice(&0x20000u64.to_le_bytes()); }
                    }
                    "fbinfo" => {
                        // 0 base, 1 largeur, 2 hauteur, 3 pas (pixels par ligne), 4 format (1 = BGRX), 5 present : lus dans la zone remplie au demarrage
                        self.hw = true; self.keyuse = true;
                        if self.efi {
                            let idx = Expr::Bin("*", Box::new(a[0].clone()), Box::new(Expr::Num(8)));
                            self.expr(&idx)?;
                            self.data_ref(&[0x48, 0x8D, 0x0D], 32);   // rcx = &table
                            self.b(&[0x48, 0x01, 0xC8, 0x48, 0x8B, 0x00]); // rax = table[n]
                        } else { self.expr(&a[0])?; self.b(&[0x31, 0xC0]); }
                    }
                    "cpuid" => {
                        // CPUID(feuille, sous-feuille 0) : renvoie le registre r (0 eax, 1 ebx, 2 ecx, 3 edx) ; rbx sauvegardé (callee-saved)
                        self.hw = true;
                        self.expr(&a[0])?; self.b(&[0x50]); self.expr(&a[1])?;
                        self.b(&[0x49, 0x89, 0xC0, 0x58, 0x53, 0x31, 0xC9, 0x0F, 0xA2, 0x52, 0x51, 0x53, 0x50, 0x4A, 0x8B, 0x04, 0xC4, 0x48, 0x83, 0xC4, 0x20, 0x5B, 0x89, 0xC0]);
                    }
                    "systab" => {
                        self.hw = true;
                        if self.efi { self.data_ref(&[0x48, 0x8B, 0x05], 0); } else { self.b(&[0x31, 0xC0]); }
                    }
                    "stall" => {
                        // BootServices->Stall(microsecondes)
                        self.hw = true; self.keyuse = true;
                        self.expr(&a[0])?;
                        if self.efi {
                            self.b(&[0x48, 0x89, 0xC1]);
                            self.b(&[0x48, 0x89, 0xE0, 0x48, 0x83, 0xE4, 0xF0, 0x48, 0x83, 0xEC, 0x30, 0x48, 0x89, 0x44, 0x24, 0x20]);
                            self.data_ref(&[0x48, 0x8B, 0x05], 0);   // rax = SystemTable
                            self.b(&[0x48, 0x8B, 0x40, 0x60]);       // rax = BootServices
                            self.b(&[0x48, 0x8B, 0x80, 0xF8, 0x00, 0x00, 0x00]); // rax = Stall
                            self.b(&[0xFF, 0xD0]);
                            self.b(&[0x48, 0x8B, 0x64, 0x24, 0x20]);
                        }
                        self.b(&[0x31, 0xC0]);
                    }
                    "readkey" => {
                        // SystemTable->ConIn->ReadKeyStroke(ConIn, &EFI_INPUT_KEY) ; renvoie (touche_scan << 16) | unicode, ou -1 si aucune touche
                        self.hw = true; self.keyuse = true;
                        if !self.efi { self.b(&[0x48, 0xC7, 0xC0, 0xFF, 0xFF, 0xFF, 0xFF]); } // hors UEFI : pas de clavier => -1
                        else {
                            self.b(&[0x48, 0x89, 0xE0, 0x48, 0x83, 0xE4, 0xF0, 0x48, 0x83, 0xEC, 0x30, 0x48, 0x89, 0x44, 0x24, 0x20]);
                            self.data_ref(&[0x48, 0x8B, 0x0D], 0);   // rcx = SystemTable
                            self.b(&[0x48, 0x8B, 0x49, 0x30]);       // rcx = SystemTable->ConIn
                            self.b(&[0x48, 0x85, 0xC9]);             // pas de clavier => aucune touche
                            let jnone1 = self.jump(&[0x0F, 0x84]);
                            self.data_ref(&[0x48, 0x8D, 0x15], 16);  // rdx = &tampon EFI_INPUT_KEY
                            self.b(&[0x48, 0x8B, 0x41, 0x08, 0xFF, 0xD0]); // call ConIn->ReadKeyStroke
                            self.b(&[0x48, 0x85, 0xC0]);             // statut 0 = touche lue
                            let jnone2 = self.jump(&[0x0F, 0x85]);
                            self.data_ref(&[0x8B, 0x05], 16);        // eax = ScanCode | Unicode << 16
                            self.b(&[0xC1, 0xC8, 0x10]);             // ror eax,16 => Scan << 16 | Unicode
                            let jend = self.jump(&[0xE9]);
                            let none = self.code.len(); self.patch(jnone1, none); self.patch(jnone2, none);
                            self.b(&[0x48, 0xC7, 0xC0, 0xFF, 0xFF, 0xFF, 0xFF]); // rax = -1
                            let end = self.code.len(); self.patch(jend, end);
                            self.b(&[0x48, 0x8B, 0x64, 0x24, 0x20]); // rsp restaure
                        }
                    }
                    "reset" => {
                        // SystemTable->RuntimeServices->ResetSystem(type, 0, 0, NULL) ; type : 0 redemarrage a froid, 1 a chaud, 2 extinction
                        self.hw = true; self.keyuse = true;
                        self.expr(&a[0])?;
                        if self.efi {
                            self.b(&[0x48, 0x89, 0xC1]);             // rcx = type
                            self.b(&[0x48, 0x89, 0xE0, 0x48, 0x83, 0xE4, 0xF0, 0x48, 0x83, 0xEC, 0x30, 0x48, 0x89, 0x44, 0x24, 0x20]);
                            self.data_ref(&[0x48, 0x8B, 0x05], 0);   // rax = SystemTable
                            self.b(&[0x48, 0x8B, 0x40, 0x58]);       // rax = RuntimeServices
                            self.b(&[0x48, 0x8B, 0x40, 0x68]);       // rax = ResetSystem
                            self.b(&[0x31, 0xD2, 0x45, 0x31, 0xC0, 0x45, 0x31, 0xC9]); // rdx = r8 = r9 = 0
                            self.b(&[0xFF, 0xD0]);                   // call ResetSystem (ne revient pas)
                            self.b(&[0x48, 0x8B, 0x64, 0x24, 0x20]); // rsp restaure si le firmware revient
                        }
                    }
                    "poke" => { self.hw = true; self.expr(&a[0])?; self.b(&[0x50]); self.expr(&a[1])?; self.b(&[0x59, 0x48, 0x89, 0x01, 0x31, 0xC0]); }
                    _ => {
                        if a.len() > 4 { return Err(format!("Ligne {l} : plus de 4 valeurs, limite actuelle.")); }
                        for x in a { self.expr(x)?; self.b(&[0x50]); }
                        let pops: [&[u8]; 4] = [&[0x59], &[0x5A], &[0x41, 0x58], &[0x41, 0x59]];
                        for i in (0..a.len()).rev() { self.b(pops[i]); }
                        self.b(&[0xE8]); let p = self.code.len(); self.i32(0); self.fix.push((p, n.clone(), *l));
                    }
                }
            }
        }
        Ok(())
    }
}

struct Built { code: Vec<u8>, main: usize, hw: bool, keys: bool }
/// Ecrit une application UEFI (PE32+, sous-systeme 10, x86-64) : .data RW (0x1000) puis .text RX (0x2000).
fn write_efi(code: &[u8]) -> Vec<u8> {
    let al = |x: usize, a: usize| (x + a - 1) / a * a;
    let raw_text = al(code.len().max(1), 0x200);
    let size_img = 0x101000 + al(code.len().max(1), 0x1000);
    let mut o: Vec<u8> = vec![0; 0x40];
    o[0] = b'M'; o[1] = b'Z'; o[0x3C..0x40].copy_from_slice(&0x40u32.to_le_bytes());
    o.extend_from_slice(b"PE\0\0");
    macro_rules! w { ($t:ty, $v:expr) => { o.extend_from_slice(&(($v) as $t).to_le_bytes()) } }
    w!(u16, 0x8664); w!(u16, 2); w!(u32, 0); w!(u32, 0); w!(u32, 0); w!(u16, 0xF0); w!(u16, 0x0022);
    w!(u16, 0x20B); w!(u8, 0); w!(u8, 0); w!(u32, raw_text); w!(u32, 0x200); w!(u32, 0); w!(u32, 0x101000); w!(u32, 0x101000);
    w!(u64, 0); w!(u32, 0x1000); w!(u32, 0x200);
    w!(u16, 0); w!(u16, 0); w!(u16, 0); w!(u16, 0); w!(u16, 0); w!(u16, 0);
    w!(u32, 0); w!(u32, size_img); w!(u32, 0x200); w!(u32, 0); w!(u16, 10); w!(u16, 0);
    w!(u64, 0x100000); w!(u64, 0x1000); w!(u64, 0x100000); w!(u64, 0x1000); w!(u32, 0); w!(u32, 16);
    o.extend_from_slice(&[0u8; 128]);
    o.extend_from_slice(b".data\0\0\0"); w!(u32, 0x100000); w!(u32, 0x1000); w!(u32, 0x200); w!(u32, 0x200); w!(u32, 0); w!(u32, 0); w!(u16, 0); w!(u16, 0); w!(u32, 0xC0000040u32);
    o.extend_from_slice(b".text\0\0\0"); w!(u32, code.len()); w!(u32, 0x101000); w!(u32, raw_text); w!(u32, 0x400); w!(u32, 0); w!(u32, 0); w!(u16, 0); w!(u16, 0); w!(u32, 0x60000020u32);
    o.resize(0x200, 0);
    o.resize(0x400, 0);
    o.extend_from_slice(code);
    o.resize(0x400 + raw_text, 0);
    o
}

fn compile(funcs: &[Func], mode: Mode) -> Result<Built, String> {
    let bare = mode == Mode::Bare;
    let mut g = Gen { code: vec![], offs: HashMap::new(), fix: vec![], sig: funcs.iter().map(|f| (f.name.clone(), f.params.len())).collect(), loc: HashMap::new(), nslots: 0, hw: false, keyuse: false, efi: mode == Mode::Efi, strs: vec![], sfix: vec![], blobs: vec![], bfix: vec![] };
    if mode == Mode::Efi {
        g.data_ref(&[0x48, 0x89, 0x15], 0); // [SystemTable] = rdx
        g.data_ref(&[0x48, 0x89, 0x0D], 8); // [ImageHandle] = rcx
        g.b(&[0x48, 0x83, 0xEC, 0x28]);     // alignement + espace d'ombre
        // --- recherche du GOP (UEFI 2.11 §12.9) : on memorise base, largeur, hauteur, pas, format dans la zone de donnees
        let gi = g.blobs.len(); g.blobs.push(vec![0xDE, 0xA9, 0x42, 0x90, 0xDC, 0x23, 0x38, 0x4A, 0x96, 0xFB, 0x7A, 0xDE, 0xD0, 0x80, 0x51, 0x6A]);
        g.data_ref(&[0x48, 0x8B, 0x05], 0);                  // rax = SystemTable
        g.b(&[0x48, 0x8B, 0x40, 0x60]);                      // rax = BootServices
        g.b(&[0x48, 0x85, 0xC0]); let j1 = g.jump(&[0x0F, 0x84]);
        g.b(&[0x48, 0x8B, 0x80, 0x40, 0x01, 0x00, 0x00]);    // rax = LocateProtocol
        g.b(&[0x48, 0x8D, 0x0D]); let p = g.code.len(); g.i32(0); g.bfix.push((p, gi)); // rcx = &GUID
        g.b(&[0x31, 0xD2]);                                  // rdx = NULL
        g.data_ref(&[0x4C, 0x8D, 0x05], 80);                 // r8 = &interface
        g.b(&[0xFF, 0xD0]);                                  // call LocateProtocol
        g.b(&[0x48, 0x85, 0xC0]); let j2 = g.jump(&[0x0F, 0x85]);
        g.data_ref(&[0x48, 0x8B, 0x05], 80);                 // rax = GOP
        g.b(&[0x48, 0x85, 0xC0]); let j3 = g.jump(&[0x0F, 0x84]);
        g.b(&[0x48, 0x8B, 0x48, 0x18]);                      // rcx = GOP->Mode
        g.b(&[0x48, 0x8B, 0x51, 0x08]);                      // rdx = Mode->Info
        g.b(&[0x48, 0x8B, 0x41, 0x18]);                      // rax = Mode->FrameBufferBase
        g.data_ref(&[0x48, 0x89, 0x05], 32);
        g.b(&[0x8B, 0x42, 0x04]); g.data_ref(&[0x48, 0x89, 0x05], 40); // largeur
        g.b(&[0x8B, 0x42, 0x08]); g.data_ref(&[0x48, 0x89, 0x05], 48); // hauteur
        g.b(&[0x8B, 0x42, 0x20]); g.data_ref(&[0x48, 0x89, 0x05], 56); // pixels par ligne
        g.b(&[0x8B, 0x42, 0x0C]); g.data_ref(&[0x48, 0x89, 0x05], 64); // format de pixel
        g.b(&[0xB8, 0x01, 0x00, 0x00, 0x00]); g.data_ref(&[0x48, 0x89, 0x05], 72); // present
        let fin = g.code.len(); g.patch(j1, fin); g.patch(j2, fin); g.patch(j3, fin);
        g.b(&[0xE8]); let p = g.code.len(); g.i32(0); g.fix.push((p, "main".into(), 0));
        g.data_ref(&[0x48, 0x89, 0x05], 24); // [resultat de main] = rax (lu par l'emulateur pour verification)
        g.b(&[0x48, 0x83, 0xC4, 0x28, 0x31, 0xC0, 0xC3]); // EFI_SUCCESS
    }
    if bare {
        g.b(&[0x48, 0xC7, 0xC4, 0x00, 0x00, 0x09, 0x00]); // mov rsp, 0x90000
        g.b(&[0xE8]); let p = g.code.len(); g.i32(0); g.fix.push((p, "main".into(), 0));
        g.b(&[0xF4, 0xEB, 0xFD]); // hlt ; jmp hlt
    }
    for f in funcs { g.func(f)?; }
    if g.efi {
        if g.code.len() % 2 == 1 { g.code.push(0); }
        for (p, idx) in g.sfix.clone() {
            let target = g.code.len();
            for c in g.strs[idx].clone() { g.code.extend_from_slice(&c.to_le_bytes()); }
            g.patch(p, target);
        }
    }
    for (p, idx) in g.bfix.clone() {
        let target = g.code.len();
        let bl = g.blobs[idx].clone(); g.code.extend_from_slice(&bl);
        g.patch(p, target);
    }
    for (p, n, l) in g.fix.clone() { let t = *g.offs.get(&n).ok_or(format!("Ligne {l} : fonction inconnue « {n} »."))?; g.patch(p, t); }
    Ok(Built { main: g.offs["main"], hw: g.hw, keys: g.keyuse, code: g.code })
}

// ---------------------------------------------------------------- execution native (Windows, memoire executable)
#[cfg(windows)]
mod native {
    extern "system" { fn VirtualAlloc(a: *mut u8, sz: usize, ty: u32, pr: u32) -> *mut u8; }
    pub fn run(code: &[u8], main: usize) -> i64 {
        unsafe {
            let m = VirtualAlloc(std::ptr::null_mut(), code.len().max(4096), 0x3000, 0x40);
            assert!(!m.is_null(), "VirtualAlloc a échoué");
            std::ptr::copy_nonoverlapping(code.as_ptr(), m, code.len());
            let f: extern "win64" fn() -> i64 = std::mem::transmute(m.add(main));
            f()
        }
    }
}
#[cfg(not(windows))]
mod native { pub fn run(_: &[u8], _: usize) -> i64 { panic!("exécution native supportée sous Windows uniquement") } }

// ---------------------------------------------------------------- outils
/// Inclusion de fichiers : une ligne « include "chemin" » est remplacee par le contenu du fichier (chemin relatif au fichier courant).
fn expand_includes(path: &str, depth: usize) -> Result<String, String> {
    if depth > 8 { return Err(format!("inclusions trop imbriquées à partir de {path}")); }
    let src = fs::read_to_string(path).map_err(|e| format!("Lecture impossible de {path} : {e}"))?;
    let dir = std::path::Path::new(path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let mut out = String::new();
    for line in src.lines() {
        if let Some(rest) = line.trim().strip_prefix("include ") {
            let name = rest.trim().trim_end_matches(';').trim().trim_matches('"');
            let p = dir.join(name);
            out.push_str(&expand_includes(&p.to_string_lossy(), depth + 1)?); out.push('\n');
        } else { out.push_str(line); out.push('\n'); }
    }
    Ok(out)
}
fn load(path: &str) -> Result<Vec<Func>, Vec<String>> {
    let src = expand_includes(path, 0).map_err(|e| vec![e])?;
    let toks = lex(&src).map_err(|e| vec![format!("{e} (numéro de ligne après inclusion des fichiers)")])?;
    let funcs = Parser { t: toks, p: 0 }.program().map_err(|e| vec![format!("{e} (numéro de ligne après inclusion des fichiers)")])?;
    check(&funcs)?;
    Ok(funcs)
}
fn scancodes_of(path: &str) -> Vec<u8> {
    fs::read_to_string(format!("{path}.scancodes")).map(|s| s.split(|c: char| c == ',' || c.is_whitespace()).filter_map(|t| u8::from_str_radix(t.trim_start_matches("0x"), 16).ok()).collect()).unwrap_or_default()
}
/// Touches de test : fichier « <programme>.touches », liste separee par des virgules (down,up,left,right,enter,esc,char:x ...).
fn touches_of(path: &str) -> Vec<(u16, u16)> {
    fs::read_to_string(format!("{path}.touches")).map(|s| emu::parse_touches(&s)).unwrap_or_default()
}
/// Resultat complet d'une execution de reference.
struct Run { r: i64, serial: Vec<u8>, fb: Vec<u8>, tones: Vec<(u64, u32)>, time_us: u64, scancodes_left: usize, screen: (u32, u32), pcm: Vec<u8>, plays: Vec<emu::hda::PlayReport> }
fn interp_full(funcs: &[Func], keys: &[(u16, u16)], scancodes: &[u8]) -> Result<Run, String> {
    let keys = keys.iter().map(|(s, u)| ((*s as i64) << 16) | *u as i64).collect();
    let mut plat = emu::Platform::new(); plat.scancodes = scancodes.iter().copied().collect();
    let fb = vec![0u8; plat.width as usize * plat.height as usize * 4];
    let mut it = Interp { funcs: funcs.iter().map(|f| (f.name.as_str(), f)).collect(), mem: HashMap::new(), plat, fb, strs: HashMap::new(), nstr: 0, steps: 0, keys };
    let r = match it.call("main", vec![]) { Ok(v) => v, Err(e) if e.starts_with("RESET:") => i64::MIN + e[6..].parse::<i64>().unwrap_or(0), Err(e) => return Err(e) };
    let screen = (it.plat.width, it.plat.height);
    let mut pcm = vec![]; let mut plays = vec![];
    for h in &it.plat.hdas { pcm.extend_from_slice(&h.pcm); plays.extend(h.plays.iter().cloned()); }
    Ok(Run { r, serial: it.plat.serial.clone(), fb: it.fb.clone(), tones: it.plat.tones.clone(), time_us: it.plat.time_us, scancodes_left: it.plat.scancodes.len(), screen, pcm, plays })
}
fn interp(funcs: &[Func], keys: &[(u16, u16)]) -> Result<(i64, Vec<u8>), String> { interp_full(funcs, keys, &[]).map(|r| (r.r, r.serial)) }
fn show(errs: Vec<String>) { for e in errs { println!("Erreur : {e}"); } }

fn main() {
    let a: Vec<String> = env::args().collect();
    let cmd = a.get(1).map(String::as_str).unwrap_or("aide");
    match cmd {
        "run" | "describe" | "build" | "exec" if a.len() >= 3 => {
            let funcs = match load(&a[2]) { Ok(f) => f, Err(e) => return show(e) };
            match cmd {
                "run" => match interp(&funcs, &touches_of(&a[2])) {
                    Ok((r, s)) => { if !s.is_empty() { println!("Sortie série : {}", String::from_utf8_lossy(&s)); } println!("Résultat de main : {r}"); }
                    Err(e) => println!("Erreur à l'exécution : {e}"),
                },
                "describe" => {
                    for f in funcs.iter().filter(|f| f.public) {
                        println!("Fonction {} : {} Elle prend {} valeur(s) : {}.", f.name, f.doc.clone().unwrap_or_default(), f.params.len(), if f.params.is_empty() { "aucune".into() } else { f.params.join(", ") });
                    }
                }
                "build" => {
                    let out = a.get(3).cloned().unwrap_or_else(|| "sortie.bin".into());
                    let bare = a.iter().any(|x| x == "--bare");
                    let efi = a.iter().any(|x| x == "--efi");
                    let mode = if efi { Mode::Efi } else if bare { Mode::Bare } else { Mode::Hosted };
                    match compile(&funcs, mode) {
                        Ok(b) if efi => { let img = write_efi(&b.code); fs::write(&out, &img).unwrap(); println!("Écrit {out} : application UEFI x86-64, {} octets ({} de code). À placer en \\EFI\\BOOT\\BOOTX64.EFI sur une partition FAT32.", img.len(), b.code.len()); println!("Non testée sur un firmware : seul un essai sur matériel réel le prouvera."); }
                        Ok(b) => { fs::write(&out, &b.code).unwrap(); println!("Écrit {out} : {} octets de code machine x86-64.", b.code.len()); if bare { println!("Image nue : démarre au premier octet, pile à 0x90000, port série 0x3F8."); } else { println!("main commence à l'octet {}.", b.main); } }
                        Err(e) => println!("Erreur : {e}"),
                    }
                }
                _ => match compile(&funcs, Mode::Hosted) {
                    Ok(b) if b.hw => println!("Ce programme touche au matériel (ports, mémoire, say) : exécution native refusée sous Windows. Utilisez « run »."),
                    Ok(b) => println!("Résultat natif de main : {}", native::run(&b.code, b.main)),
                    Err(e) => println!("Erreur : {e}"),
                },
            }
        }
        "verify" if a.len() >= 3 => {
            let (mut pass, mut fail, mut skip) = (0, 0, 0);
            for p in &a[2..] {
                let funcs = match load(p) { Ok(f) => f, Err(e) => { println!("[FAIL] {p} : {}", e.join(" ")); fail += 1; continue; } };
                let touches = touches_of(p); let scans = scancodes_of(p);
                let run = match interp_full(&funcs, &touches, &scans) { Ok(r) => r, Err(e) => { println!("[FAIL] {p} : interpréteur : {e}"); fail += 1; continue; } };
                let (r, iserial) = (run.r, run.serial.clone());
                match compile(&funcs, Mode::Hosted) {
                    Err(e) => { println!("[FAIL] {p} : compilation : {e}"); fail += 1; }
                    Ok(b) if b.hw => {
                        // Matériel : exécution dans l'émulateur maison (image nue + application UEFI). Ce n'est PAS du matériel réel.
                        let efi = compile(&funcs, Mode::Efi).and_then(|x| emu::run_efi_full(&write_efi(&x.code), false, &touches, &scans));
                        let bare = if b.keys { None } else { Some(compile(&funcs, Mode::Bare).and_then(|x| emu::run_bare_full(&x.code, &scans))) };
                        let reset_expect = if r >= i64::MIN && r < i64::MIN + 16 { Some(r - i64::MIN) } else { None };
                        let efi_ok = matches!(&efi, Ok(eo) if eo.serial == iserial && eo.violations.is_empty() && eo.pcm == run.pcm && eo.plays.iter().all(|p| p.audible) && eo.hda_errors.is_empty() && eo.tones == run.tones && eo.time_us == run.time_us && eo.scancodes_left == run.scancodes_left
                            && (eo.fb == run.fb || run.fb.iter().all(|x| *x == 0) && eo.fb.iter().all(|x| *x == 0)) && match reset_expect { Some(k) => eo.reset == Some(k), None => eo.reset.is_none() && eo.main_ret == r });
                        let bare_ok = match &bare { None => true, Some(Ok(bo)) => bo.rax as i64 == r && bo.serial == iserial && bo.violations.is_empty() && bo.tones == run.tones && bo.time_us == run.time_us && bo.scancodes_left == run.scancodes_left, Some(Err(_)) => false };
                        if efi_ok && bare_ok {
                            let image = if run.fb.iter().any(|x| *x != 0) { format!(", image {}×{} identique pixel pour pixel", run.screen.0, run.screen.1) } else { String::new() };
                            let sons = if run.tones.is_empty() { String::new() } else { format!(", {} événements de haut-parleur identiques", run.tones.len()) };
                            let sons = if run.plays.is_empty() { sons } else { format!("{sons}, audio HD : {} octets PCM identiques et chemin analogique audible dans le modèle", run.pcm.len()) };
                            println!("[PASS] {p} : interpréteur = émulateur UEFI{} = {}, sorties identiques{image}{sons}, ABI respectée (émulé, non testé sur matériel réel)", if b.keys { format!(" (clavier simulé : {} touches, {} codes 8042)", touches.len(), scans.len()) } else { " = émulateur nu".into() }, match reset_expect { Some(k) => format!("redémarrage/extinction demandé (type {k})"), None => r.to_string() }); pass += 1;
                        } else {
                            println!("[FAIL] {p} : écart émulateur (interpréteur {r}, série {:?}, sons {:?}). uefi = {:?} ; nu = {:?}", String::from_utf8_lossy(&iserial), run.tones, efi.map(|o| (o.main_ret, String::from_utf8_lossy(&o.serial).to_string(), o.violations, o.tones, o.fb == run.fb)), bare.map(|x| x.map(|o| (o.rax as i64, o.violations)))); fail += 1;
                        }
                        skip += 1; // le chemin matériel reste non prouvé sur silicium
                    }
                    Ok(b) => {
                        let n = native::run(&b.code, b.main);
                        let e = emu::run_raw(&b.code, b.main).map(|o| o.rax as i64);
                        match e {
                            Ok(em) if n == r && em == r => { println!("[PASS] {p} : interpréteur = processeur réel = émulateur maison = {r}"); pass += 1; }
                            other => { println!("[FAIL] {p} : interpréteur {r}, natif {n}, émulateur {:?}", other); fail += 1; }
                        }
                    }
                }
            }
            println!("Bilan : PASS={pass} FAIL={fail} dont chemin matériel réel non prouvé={skip}");
        }
        _ => println!("Voxa. Commandes : run fichier | exec fichier | describe fichier | build fichier sortie [--bare] | verify fichiers..."),
    }
}

/// Memoire de l'interpreteur vue par le modele de DMA du controleur audio.
struct InterpMem<'a> { mem: &'a mut HashMap<i64, u8>, fb: &'a mut Vec<u8> }
impl emu::hda::Mem for InterpMem<'_> {
    fn rd8(&self, a: u64) -> u8 { let a = a as i64; let fb0 = emu::FB_BASE as i64; if a >= fb0 && ((a - fb0) as usize) < self.fb.len() { self.fb[(a - fb0) as usize] } else { *self.mem.get(&a).unwrap_or(&0) } }
    fn wr8(&mut self, a: u64, v: u8) { let a = a as i64; let fb0 = emu::FB_BASE as i64; if a >= fb0 && ((a - fb0) as usize) < self.fb.len() { self.fb[(a - fb0) as usize] = v; } else { self.mem.insert(a, v); } }
}

