//! Synthétiseur de parole française à formants, arithmétique ENTIÈRE, `no_std`, sans allocation. Port de `src/voixlib.rs` (référence, hôte) et de `bib/voix.vx`.
//! La conformité est vérifiée échantillon par échantillon contre la référence (tests ci-dessous). Les paramètres viennent de ../bib/voix-params-final.txt (build.rs).
//! Entrée : transcription phonétique (voir VOXA-VOIX.md). Sortie : échantillons 16 bits mono à 22 050 Hz.

pub const FS: i64 = 22050;
const MAX_KF: usize = 512;

pub struct Phone { pub sym: u8, pub ty: u8, pub dur: i64, pub f: [i64; 3], pub amp: i64, pub bf: i64, pub bw: i64 }
pub struct Globals { pub dur_pct: i64, pub f0: i64, pub rad: i64, pub tilt: i64, pub bw_pct: i64, pub nz_pct: i64, pub bu_pct: i64, pub gain: i64, pub acc_pct: i64, pub decl: i64, pub f0_acc: i64, pub ferm_p: i64, pub ferm_b: i64, pub asp_ms: i64, pub burst_ms: i64, pub plat: i64, pub pause_ms: i64 }
include!(concat!(env!("OUT_DIR"), "/tables.rs"));

#[derive(Clone, Copy)]
struct Res { a: i64, b: i64, c: i64, y1: i64, y2: i64 }
impl Res {
    const fn new() -> Res { Res { a: 4096, b: 0, c: 0, y1: 0, y2: 0 } }
    fn regler(&mut self, f: i64, bw: i64) {
        let r = EXP[(bw / 10).clamp(0, 100) as usize]; let c = COS[(f / 10).clamp(0, 551) as usize];
        self.b = 2 * r * c / 4096; self.c = -(r * r / 4096); self.a = 4096 - self.b - self.c;
    }
    fn pas(&mut self, x: i64) -> i64 { let y = ((self.a * x + self.b * self.y1 + self.c * self.y2) / 4096).clamp(-200000, 200000); self.y2 = self.y1; self.y1 = y; y }
}

#[derive(Clone, Copy)]
struct Kf { t: i64, p: [i64; 10], formants: bool }
const KF0: Kf = Kf { t: 0, p: [0; 10], formants: false };

/// Mémoire de travail de la synthèse (points-clés). À conserver entre les appels pour éviter de grosses piles.
pub struct Work { kfs: [Kf; MAX_KF], n: usize, overflow: bool }
impl Work { pub const fn new() -> Work { Work { kfs: [KF0; MAX_KF], n: 0, overflow: false } } }

fn est_voy(c: u8) -> bool { matches!(c, b'a' | b'e' | b'E' | b'i' | b'o' | b'O' | b'u' | b'y' | b'2' | b'9' | b'@' | b'Q' | b'I' | b'U') }
fn phone(c: u8) -> Option<&'static Phone> { PHONES.iter().find(|p| p.sym == c) }
fn ms(m: i64) -> i64 { m * FS / 1000 }

impl Work {
    fn push(&mut self, t: i64, p: [i64; 10], formants: bool) { if self.n < MAX_KF { self.kfs[self.n] = Kf { t, p, formants }; self.n += 1; } else { self.overflow = true; } }
    fn silence(&mut self, tc: i64, d: i64, f0: i64) { let p = [f0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; self.push(tc, p, false); self.push(tc + d, p, false); }
}

pub struct Result { pub samples: usize, /// vrai si le texte était trop long (points-clés ou tampon) : la sortie est tronquée
    pub truncated: bool }

/// Synthétise `phon` (mots séparés par des espaces) dans `out`. Renvoie le nombre d'échantillons écrits.
pub fn parler(w: &mut Work, phon: &str, out: &mut [i16]) -> Result {
    let g = &GLOBALS;
    w.n = 0; w.overflow = false;
    let total_syll = phon.bytes().filter(|c| est_voy(*c)).count() as i64;
    let (mut vu, mut tc) = (0i64, 0i64);
    w.silence(tc, ms(30), g.f0); tc += ms(30);
    let nmots = phon.split_whitespace().count();
    for (im, mot) in phon.split_whitespace().enumerate() {
        let chars = mot.as_bytes();
        let derniere_v = chars.iter().rposition(|c| est_voy(*c));
        for (i, &c) in chars.iter().enumerate() {
            let q = match phone(c) { Some(p) => p, None => continue };
            let (sym, ty, f1, f2, f3, amp, bf, bw) = (q.sym, q.ty, q.f[0], q.f[1], q.f[2], q.amp, q.bf, q.bw);
            let accent = Some(i) == derniere_v;
            let dur = ms((if accent { q.dur * g.acc_pct / 100 } else { q.dur }) * g.dur_pct / 100);
            let f0 = g.f0 - g.decl * vu / total_syll.max(1) + if accent { g.f0_acc } else { 0 };
            if est_voy(sym) { vu += 1; }
            match ty {
                b'V' | b'G' | b'L' | b'N' => {
                    let br = if bf > 0 { (bf, bw, 12) } else { (0, 0, 0) };
                    let p = [f0, f1, f2, f3, amp, 100, br.0, br.1, br.2, 0];
                    w.push(tc + dur * g.plat / 100, p, true); w.push(tc + dur * (100 - g.plat) / 100, p, true);
                }
                b'F' | b'Z' => {
                    let v = if ty == b'Z' { 70 } else { 0 }; let a = if ty == b'Z' { 40 } else { 0 };
                    let p = [f0, f1, f2, f3, a, v, bf, bw, amp * 2 * g.nz_pct / 100, 0]; let fo = ty == b'Z';
                    w.push(tc, p, fo); w.push(tc + dur, p, fo);
                }
                _ => {
                    let voisee = ty == b'B'; let ferm = ms(if voisee { g.ferm_b } else { g.ferm_p });
                    let bar = [f0, 250, 1200, 2400, if voisee { 12 } else { 0 }, if voisee { 100 } else { 0 }, 0, 0, 0, 0];
                    w.push(tc, bar, false); w.push(tc + ferm, bar, false);
                    let ex = [f0, 0, 0, 0, 30, 0, bf, bw, amp * 2 * g.bu_pct / 100, 0];
                    w.push(tc + ferm + 1, ex, false); w.push(tc + ferm + ms(g.burst_ms), ex, false);
                    let mut tt = tc + ferm + ms(g.burst_ms);
                    if !voisee { let ap = [f0, 0, 0, 0, 60, 0, 0, 0, 0, 120]; w.push(tt + 1, ap, false); w.push(tt + ms(g.asp_ms), ap, false); tt += ms(g.asp_ms); }
                    tc = tt - dur;
                }
            }
            tc += dur;
        }
        if im + 1 < nmots { w.silence(tc, ms(g.pause_ms), g.f0 - 10); tc += ms(g.pause_ms); }
    }
    w.silence(tc, ms(100), g.f0 - 20);
    let n = w.n;
    for i in 0..n {
        if w.kfs[i].formants { continue; }
        let prev = (0..i).rev().find(|&j| w.kfs[j].formants); let next = (i + 1..n).find(|&j| w.kfs[j].formants);
        for c in 1..4 {
            w.kfs[i].p[c] = match (prev, next) {
                (Some(a), Some(b)) => { let den = (w.kfs[b].t - w.kfs[a].t).max(1); w.kfs[a].p[c] + (w.kfs[b].p[c] - w.kfs[a].p[c]) * (w.kfs[i].t - w.kfs[a].t) / den }
                (Some(a), None) => w.kfs[a].p[c], (None, Some(b)) => w.kfs[b].p[c], _ => [0, 500, 1500, 2500][c],
            };
        }
    }
    for i in 1..n { if w.kfs[i].t < w.kfs[i - 1].t { w.kfs[i].t = w.kfs[i - 1].t; } }
    let written = rendre(w, out);
    Result { samples: written, truncated: w.overflow || written < w.kfs[n - 1].t.max(0) as usize }
}

fn rendre(w: &Work, out: &mut [i16]) -> usize {
    let kfs = &w.kfs[..w.n]; let g = &GLOBALS;
    let fin = kfs.last().map(|k| k.t).unwrap_or(0);
    let mut r = [Res::new(); 5]; let mut rn = Res::new();
    let (mut rng, mut g1, mut g2, mut gp, mut np, mut phase) = (12345i64, 0i64, 0i64, 0i64, 0i64, 0i64);
    let mut i = 0usize; let mut p = [0i64; 10]; let mut n = 0usize;
    for t in 0..fin {
        if n >= out.len() { break; }
        while i + 2 < kfs.len() && t >= kfs[i + 1].t { i += 1; }
        if t % 32 == 0 {
            let (a, b) = (kfs[i], kfs[(i + 1).min(kfs.len() - 1)]); let den = (b.t - a.t).max(1); let u = (t - a.t).clamp(0, den);
            for k in 0..10 { p[k] = a.p[k] + (b.p[k] - a.p[k]) * u / den; }
            let bw = g.bw_pct;
            r[0].regler(p[1].max(150), 70 * bw / 100); r[1].regler(p[2].max(400), 90 * bw / 100); r[2].regler(p[3].max(800), 130 * bw / 100);
            r[3].regler(3500, 200); r[4].regler(4500, 250);
            if p[8] > 0 { rn.regler(p[6].max(200), p[7].max(100)); }
        }
        phase += 1;
        let periode = FS / p[0].max(60);
        let imp = if phase >= periode { phase -= periode; 40000 } else { 0 };
        let tl = g.tilt.max(1);
        g1 += (imp - g1) / tl; g2 += (g1 - g2) / tl;
        rng = (rng * 1103515245 + 12345) % 2147483648; let nz = (rng / 65536) % 65536 - 32768;
        let src = if g.rad > 0 { let d = (g2 - gp) * g.rad; gp = g2; d } else { g2 };
        let mut x = src * p[5] / 100 + nz * p[9] / 400;
        for k in 0..5 { x = r[k].pas(x); }
        let mut y = x * p[4] / 100;
        if p[8] > 0 { let d = nz - np; let b = rn.pas(d / 2 * p[8] / 100); y += b * p[4].max(30) / 100 / 2; }
        np = nz;
        out[n] = (y / g.gain.max(1)).clamp(-30000, 30000) as i16; n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    #[allow(dead_code)]
    #[path = "../../../../src/voixlib.rs"]
    mod reference;

    fn rust(t: &str) -> std::vec::Vec<i16> {
        let mut w = std::boxed::Box::new(Work::new()); let mut out = std::vec![0i16; 200_000];
        let r = parler(&mut w, t, &mut out); assert!(!r.truncated, "tronqué : {t}"); out.truncate(r.samples); out
    }
    #[test] fn echantillons_identiques_a_la_reference() {
        let k = reference::Params::charger("../bib/voix-params-final.txt").expect("paramètres illisibles");
        let phrases = ["m@ny pRIsipal", "OpsjU I syR sEt liR lEd", "vOksa lajv QviRonmQ aksEsibl", "kUfiRme l@ R@demaRaZ QtRe yn s@gUd fwa eSap puR anyle",
            "pROsEs9R a Em de RizEn sEt sIk mil Hit sQ aS", "tEste l@ sU", "R@demaRe lORdinat9R", "bUZuR vOksa", "etIdR lORdinat9R", "fiSje dosje"];
        for t in phrases {
            let (a, b) = (rust(t), reference::parler(&k, t));
            assert_eq!(a.len(), b.len(), "longueur différente pour « {t} »");
            let ecarts = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
            assert_eq!(ecarts, 0, "{ecarts} échantillons différents sur {} pour « {t} »", a.len());
        }
    }
    #[test] fn texte_trop_long_est_signale_pas_ignore() {
        let mut w = std::boxed::Box::new(Work::new()); let mut out = std::vec![0i16; 1000];
        let r = parler(&mut w, "m@ny pRIsipal OpsjU I syR sEt liR lEd", &mut out);
        assert!(r.truncated, "un tampon trop petit doit être signalé"); assert_eq!(r.samples, 1000);
    }
    #[test] fn silence_pour_texte_vide() {
        let mut w = std::boxed::Box::new(Work::new()); let mut out = std::vec![0i16; 10_000];
        let r = parler(&mut w, "", &mut out); assert!(out[..r.samples].iter().all(|s| *s == 0));
    }
}
