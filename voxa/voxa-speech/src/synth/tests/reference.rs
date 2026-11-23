// Bibliotheque du synthetiseur de parole francais a formants de Voxa (arithmetique ENTIERE : meme algorithme que bib/voix.vx).
// Les parametres des sons (formants, durees, niveaux) sont des donnees reglables ; ils sont ajustes automatiquement par `ajuster`
// en mesurant la parole francaise (les voix installees servent d'instrument de mesure, aucun echantillon n'est conserve).
#![allow(dead_code)]
use std::fs;

pub const FS: i64 = 22050;
pub fn table_cos() -> Vec<i64> { (0..=551).map(|k| ((2.0 * std::f64::consts::PI * (k * 10) as f64 / FS as f64).cos() * 4096.0).round() as i64).collect() }
pub fn table_exp() -> Vec<i64> { (0..=100).map(|k| ((-std::f64::consts::PI * (k * 10) as f64 / FS as f64).exp() * 4096.0).round() as i64).collect() }

#[derive(Clone, Copy)]
struct Res { a: i64, b: i64, c: i64, y1: i64, y2: i64 }
impl Res {
    fn new() -> Res { Res { a: 4096, b: 0, c: 0, y1: 0, y2: 0 } }
    fn regler(&mut self, cos: &[i64], exp: &[i64], f: i64, bw: i64) {
        let r = exp[(bw / 10).clamp(0, 100) as usize]; let c = cos[(f / 10).clamp(0, 551) as usize];
        self.b = 2 * r * c / 4096; self.c = -(r * r / 4096); self.a = 4096 - self.b - self.c;
    }
    fn pas(&mut self, x: i64) -> i64 { let y = ((self.a * x + self.b * self.y1 + self.c * self.y2) / 4096).clamp(-200000, 200000); self.y2 = self.y1; self.y1 = y; y }
}

#[derive(Clone, Debug)]
pub struct Phone { pub sym: char, pub ty: char, pub dur: i64, pub f: [i64; 3], pub amp: i64, pub bf: i64, pub bw: i64 }
// Types : V voyelle, G glissante, L liquide, N nasale, F fricative sourde, Z fricative voisee, P occlusive sourde, B occlusive voisee
fn ph(sym: char, ty: char, dur: i64, f1: i64, f2: i64, f3: i64, amp: i64, bf: i64, bw: i64) -> Phone { Phone { sym, ty, dur, f: [f1, f2, f3], amp, bf, bw } }

#[derive(Clone, Debug)]
pub struct Params { pub phones: Vec<Phone>, pub dur_pct: i64, pub f0: i64, pub rad: i64, pub tilt: i64, pub bw_pct: i64, pub nz_pct: i64, pub bu_pct: i64, pub gain: i64, pub acc_pct: i64, pub decl: i64, pub f0_acc: i64, pub ferm_p: i64, pub ferm_b: i64, pub asp_ms: i64, pub burst_ms: i64, pub plat: i64, pub pause_ms: i64 }
impl Params {
    pub fn defaut() -> Params {
        Params { phones: vec![
            ph('a', 'V', 120, 700, 1250, 2600, 100, 0, 0), ph('e', 'V', 110, 380, 2100, 2900, 90, 0, 0), ph('E', 'V', 120, 560, 1800, 2600, 95, 0, 0),
            ph('i', 'V', 100, 290, 2250, 3000, 80, 0, 0), ph('o', 'V', 120, 400, 850, 2500, 95, 0, 0), ph('O', 'V', 120, 580, 1000, 2600, 100, 0, 0),
            ph('u', 'V', 110, 320, 800, 2300, 85, 0, 0), ph('y', 'V', 100, 290, 1750, 2150, 80, 0, 0), ph('2', 'V', 115, 400, 1500, 2300, 90, 0, 0),
            ph('9', 'V', 120, 560, 1450, 2300, 95, 0, 0), ph('@', 'V', 70, 450, 1400, 2400, 70, 0, 0),
            ph('Q', 'V', 140, 650, 1050, 2500, 85, 0, 0), ph('I', 'V', 140, 500, 1750, 2500, 85, 0, 0), ph('U', 'V', 140, 450, 850, 2500, 85, 0, 0),
            ph('j', 'G', 60, 290, 2250, 3000, 80, 0, 0), ph('w', 'G', 60, 320, 800, 2300, 80, 0, 0), ph('H', 'G', 60, 290, 1750, 2150, 80, 0, 0),
            ph('l', 'L', 70, 380, 1500, 2700, 75, 0, 0), ph('R', 'L', 80, 450, 1300, 2400, 70, 2000, 1500),
            ph('m', 'N', 85, 250, 1200, 2200, 60, 0, 0), ph('n', 'N', 80, 250, 1700, 2600, 60, 0, 0),
            ph('f', 'F', 100, 0, 0, 0, 22, 4500, 3500), ph('s', 'F', 110, 0, 0, 0, 60, 5500, 1800), ph('S', 'F', 110, 0, 0, 0, 55, 2800, 1400),
            ph('v', 'Z', 80, 300, 1200, 2400, 30, 4500, 3500), ph('z', 'Z', 90, 300, 1500, 2600, 40, 5500, 1800), ph('Z', 'Z', 90, 300, 1400, 2400, 35, 2800, 1400),
            ph('p', 'P', 80, 0, 0, 0, 40, 800, 1500), ph('t', 'P', 80, 0, 0, 0, 45, 4000, 2000), ph('k', 'P', 85, 0, 0, 0, 45, 1800, 1200),
            ph('b', 'B', 70, 250, 1100, 2300, 35, 800, 1500), ph('d', 'B', 70, 250, 1600, 2600, 35, 4000, 2000), ph('g', 'B', 75, 250, 1400, 2300, 35, 1800, 1200),
        ], dur_pct: 130, f0: 120, rad: 3, tilt: 3, bw_pct: 100, nz_pct: 100, bu_pct: 100, gain: 2, acc_pct: 170, decl: 30, f0_acc: 20, ferm_p: 55, ferm_b: 50, asp_ms: 22, burst_ms: 10, plat: 20, pause_ms: 45 }
    }
    pub fn phone(&self, c: char) -> Option<&Phone> { self.phones.iter().find(|p| p.sym == c) }
    pub fn sauver(&self, chemin: &str) {
        let mut s = format!("# parametres du synthetiseur Voxa (ajustes automatiquement)\nglobal dur_pct={} f0={} rad={} tilt={} bw_pct={} nz_pct={} bu_pct={} gain={}\n", self.dur_pct, self.f0, self.rad, self.tilt, self.bw_pct, self.nz_pct, self.bu_pct, self.gain);
        for p in &self.phones { s += &format!("{} {} {} {} {} {} {} {} {}\n", p.sym, p.ty, p.dur, p.f[0], p.f[1], p.f[2], p.amp, p.bf, p.bw); }
        fs::write(chemin, s).unwrap();
    }
    pub fn charger(chemin: &str) -> Option<Params> {
        let t = fs::read_to_string(chemin).ok()?; let mut p = Params::defaut(); p.phones.clear();
        for l in t.lines() {
            if l.starts_with('#') || l.trim().is_empty() { continue; }
            if let Some(r) = l.strip_prefix("global ") {
                for kv in r.split_whitespace() { let (k, v) = kv.split_once('=')?; let v: i64 = v.parse().ok()?;
                    match k { "dur_pct" => p.dur_pct = v, "f0" => p.f0 = v, "rad" => p.rad = v, "tilt" => p.tilt = v, "bw_pct" => p.bw_pct = v, "nz_pct" => p.nz_pct = v, "bu_pct" => p.bu_pct = v, "gain" => p.gain = v, "acc_pct" => p.acc_pct = v, "decl" => p.decl = v, "f0_acc" => p.f0_acc = v, "ferm_p" => p.ferm_p = v, "ferm_b" => p.ferm_b = v, "asp_ms" => p.asp_ms = v, "burst_ms" => p.burst_ms = v, "plat" => p.plat = v, "pause_ms" => p.pause_ms = v, _ => {} } }
                continue;
            }
            let m: Vec<&str> = l.split_whitespace().collect(); if m.len() < 9 { return None; }
            let n = |i: usize| m[i].parse::<i64>().ok();
            p.phones.push(Phone { sym: m[0].chars().next()?, ty: m[1].chars().next()?, dur: n(2)?, f: [n(3)?, n(4)?, n(5)?], amp: n(6)?, bf: n(7)?, bw: n(8)? });
        }
        Some(p)
    }
}

#[derive(Clone, Copy, Debug)]
struct Kf { t: i64, p: [i64; 10], formants: bool }
// p : 0 f0, 1 f1, 2 f2, 3 f3, 4 amp, 5 voix, 6 bruit_fc, 7 bruit_bw, 8 bruit_gain, 9 souffle
const VOY: &str = "aeEioOuy29@QIU";

struct Synth<'a> { k: &'a Params, cos: Vec<i64>, exp: Vec<i64>, rng: i64, g1: i64, g2: i64, gp: i64, np: i64, phase: i64, r: [Res; 5], rn: Res, out: Vec<i16> }
impl<'a> Synth<'a> {
    fn bruit(&mut self) -> i64 { self.rng = (self.rng * 1103515245 + 12345) % 2147483648; (self.rng / 65536) % 65536 - 32768 }
    fn rendre(&mut self, kfs: &[Kf]) {
        let fin = kfs.last().map(|k| k.t).unwrap_or(0);
        let mut i = 0usize; let mut p = [0i64; 10];
        for t in 0..fin {
            while i + 2 < kfs.len() && t >= kfs[i + 1].t { i += 1; }
            if t % 32 == 0 {
                let (a, b) = (kfs[i], kfs[(i + 1).min(kfs.len() - 1)]); let den = (b.t - a.t).max(1); let u = (t - a.t).clamp(0, den);
                for k in 0..10 { p[k] = a.p[k] + (b.p[k] - a.p[k]) * u / den; }
                let bw = self.k.bw_pct;
                self.r[0].regler(&self.cos, &self.exp, p[1].max(150), 70 * bw / 100); self.r[1].regler(&self.cos, &self.exp, p[2].max(400), 90 * bw / 100); self.r[2].regler(&self.cos, &self.exp, p[3].max(800), 130 * bw / 100);
                self.r[3].regler(&self.cos, &self.exp, 3500, 200); self.r[4].regler(&self.cos, &self.exp, 4500, 250);
                if p[8] > 0 { self.rn.regler(&self.cos, &self.exp, p[6].max(200), p[7].max(100)); }
            }
            self.phase += 1;
            let periode = FS / p[0].max(60);
            let imp = if self.phase >= periode { self.phase -= periode; 40000 } else { 0 };
            let tl = self.k.tilt.max(1);
            self.g1 += (imp - self.g1) / tl; self.g2 += (self.g1 - self.g2) / tl;
            let nz = self.bruit();
            let src = if self.k.rad > 0 { let d = (self.g2 - self.gp) * self.k.rad; self.gp = self.g2; d } else { self.g2 };
            let mut x = src * p[5] / 100 + nz * p[9] / 400;
            for r in 0..5 { x = self.r[r].pas(x); }
            let mut y = x * p[4] / 100;
            if p[8] > 0 { let d = nz - self.np; let b = self.rn.pas(d / 2 * p[8] / 100); y += b * p[4].max(30) / 100 / 2; }
            self.np = nz;
            self.out.push((y / self.k.gain.max(1)).clamp(-30000, 30000) as i16);
        }
    }
}
fn ms(m: i64) -> i64 { m * FS / 1000 }

/// Transcription phonetique -> echantillons. Espace = frontiere de mot.
pub fn parler(k: &Params, t: &str) -> Vec<i16> {
    let mut s = Synth { k, cos: table_cos(), exp: table_exp(), rng: 12345, g1: 0, g2: 0, gp: 0, np: 0, phase: 0, r: [Res::new(); 5], rn: Res::new(), out: vec![] };
    let mots: Vec<&str> = t.split_whitespace().collect();
    let total_syll: i64 = t.chars().filter(|c| VOY.contains(*c)).count() as i64;
    let mut vu = 0i64; let mut kfs: Vec<Kf> = vec![]; let mut tc = 0i64;
    let silence = |tc: i64, kfs: &mut Vec<Kf>, d: i64, f0: i64| { kfs.push(Kf { t: tc, p: [f0, 0, 0, 0, 0, 0, 0, 0, 0, 0], formants: false }); kfs.push(Kf { t: tc + d, p: [f0, 0, 0, 0, 0, 0, 0, 0, 0, 0], formants: false }); };
    silence(tc, &mut kfs, ms(30), k.f0); tc += ms(30);
    for (im, mot) in mots.iter().enumerate() {
        let chars: Vec<char> = mot.chars().collect();
        let derniere_v = chars.iter().rposition(|c| VOY.contains(*c));
        for (i, &c) in chars.iter().enumerate() {
            let q = match k.phone(c) { Some(p) => p.clone(), None => continue };
            let (sym, ty, f1, f2, f3, amp, bf, bw) = (q.sym, q.ty, q.f[0], q.f[1], q.f[2], q.amp, q.bf, q.bw);
            let accent = Some(i) == derniere_v;
            let dur = ms((if accent { q.dur * k.acc_pct / 100 } else { q.dur }) * k.dur_pct / 100);
            let f0 = k.f0 - k.decl * vu / total_syll.max(1) + if accent { k.f0_acc } else { 0 };
            if VOY.contains(sym) { vu += 1; }
            match ty {
                'V' | 'G' | 'L' | 'N' => {
                    let br = if bf > 0 { (bf, bw, 12) } else { (0, 0, 0) };
                    let p = [f0, f1, f2, f3, amp, 100, br.0, br.1, br.2, 0];
                    kfs.push(Kf { t: tc + dur * k.plat / 100, p, formants: true }); kfs.push(Kf { t: tc + dur * (100 - k.plat) / 100, p, formants: true });
                }
                'F' | 'Z' => {
                    let v = if ty == 'Z' { 70 } else { 0 }; let a = if ty == 'Z' { 40 } else { 0 };
                    let p = [f0, f1, f2, f3, a, v, bf, bw, amp * 2 * k.nz_pct / 100, 0]; let fo = ty == 'Z';
                    kfs.push(Kf { t: tc, p, formants: fo }); kfs.push(Kf { t: tc + dur, p, formants: fo });
                }
                _ => {
                    let voisee = ty == 'B'; let ferm = ms(if voisee { k.ferm_b } else { k.ferm_p });
                    let bar = [f0, 250, 1200, 2400, if voisee { 12 } else { 0 }, if voisee { 100 } else { 0 }, 0, 0, 0, 0];
                    kfs.push(Kf { t: tc, p: bar, formants: false }); kfs.push(Kf { t: tc + ferm, p: bar, formants: false });
                    let ex = [f0, 0, 0, 0, 30, 0, bf, bw, amp * 2 * k.bu_pct / 100, 0];
                    kfs.push(Kf { t: tc + ferm + 1, p: ex, formants: false }); kfs.push(Kf { t: tc + ferm + ms(k.burst_ms), p: ex, formants: false });
                    let mut tt = tc + ferm + ms(k.burst_ms);
                    if !voisee { let ap = [f0, 0, 0, 0, 60, 0, 0, 0, 0, 120]; kfs.push(Kf { t: tt + 1, p: ap, formants: false }); kfs.push(Kf { t: tt + ms(k.asp_ms), p: ap, formants: false }); tt += ms(k.asp_ms); }
                    tc = tt - dur;
                }
            }
            tc += dur;
        }
        if im + 1 < mots.len() { silence(tc, &mut kfs, ms(k.pause_ms), k.f0 - 10); tc += ms(k.pause_ms); }
    }
    silence(tc, &mut kfs, ms(100), k.f0 - 20);
    let n = kfs.len();
    for i in 0..n {
        if kfs[i].formants { continue; }
        let prev = (0..i).rev().find(|&j| kfs[j].formants); let next = (i + 1..n).find(|&j| kfs[j].formants);
        for c in 1..4 {
            kfs[i].p[c] = match (prev, next) {
                (Some(a), Some(b)) => { let den = (kfs[b].t - kfs[a].t).max(1); kfs[a].p[c] + (kfs[b].p[c] - kfs[a].p[c]) * (kfs[i].t - kfs[a].t) / den }
                (Some(a), None) => kfs[a].p[c], (None, Some(b)) => kfs[b].p[c], _ => [0, 500, 1500, 2500][c],
            };
        }
    }
    for i in 1..kfs.len() { if kfs[i].t < kfs[i - 1].t { kfs[i].t = kfs[i - 1].t; } }
    s.rendre(&kfs);
    s.out
}

pub fn wav_mono(pcm: &[i16]) -> Vec<u8> {
    let n = (pcm.len() * 2) as u32; let mut w = Vec::new();
    w.extend_from_slice(b"RIFF"); w.extend_from_slice(&(36 + n).to_le_bytes()); w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes()); w.extend_from_slice(&1u16.to_le_bytes()); w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&(FS as u32).to_le_bytes()); w.extend_from_slice(&((FS * 2) as u32).to_le_bytes()); w.extend_from_slice(&2u16.to_le_bytes()); w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data"); w.extend_from_slice(&n.to_le_bytes()); for s in pcm { w.extend_from_slice(&s.to_le_bytes()); }
    w
}
