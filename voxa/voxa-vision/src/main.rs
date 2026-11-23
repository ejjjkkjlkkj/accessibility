//! Voxa Vision : lecture de l'ecran par les pixels, sans UI Automation, sans MSAA, sans cadre d'accessibilite.
//! Seuls des appels systeme de bas niveau sont utilises : copie d'ecran (GDI BitBlt) et dessin de caracteres (GDI TextOut) pour FABRIQUER notre propre atlas de glyphes.
//! Utilisation : voxa-vision --autotest | --ecran x y l h | --premier-plan [--compter]
#![allow(non_snake_case, clippy::too_many_arguments)]
use std::ffi::c_void;
mod regard;
type H = isize;
#[repr(C)] struct Rect { l: i32, t: i32, r: i32, b: i32 }
#[repr(C)] struct Bmi { size: u32, w: i32, h: i32, planes: u16, bpc: u16, comp: u32, img: u32, xp: i32, yp: i32, used: u32, imp: u32 }
#[link(name = "gdi32")] extern "system" {
    fn CreateCompatibleDC(h: H) -> H; fn CreateCompatibleBitmap(h: H, w: i32, hh: i32) -> H; fn SelectObject(h: H, o: H) -> H;
    fn BitBlt(d: H, x: i32, y: i32, w: i32, hh: i32, s: H, sx: i32, sy: i32, rop: u32) -> i32; fn GetDIBits(h: H, b: H, s: u32, n: u32, bits: *mut c_void, bmi: *mut Bmi, u: u32) -> i32;
    fn CreateFontW(h: i32, w: i32, e: i32, o: i32, wt: i32, i: u32, u: u32, s: u32, cs: u32, op: u32, cp: u32, q: u32, pf: u32, face: *const u16) -> H;
    fn TextOutW(h: H, x: i32, y: i32, s: *const u16, n: i32) -> i32; fn SetBkColor(h: H, c: u32) -> u32; fn SetTextColor(h: H, c: u32) -> u32; fn SetBkMode(h: H, m: i32) -> i32;
    fn GetCharWidth32W(h: H, a: u32, b: u32, out: *mut i32) -> i32; fn DeleteObject(o: H) -> i32; fn DeleteDC(h: H) -> i32; fn PatBlt(h: H, x: i32, y: i32, w: i32, hh: i32, rop: u32) -> i32;
}
#[link(name = "user32")] extern "system" { fn GetDC(w: H) -> H; fn ReleaseDC(w: H, d: H) -> i32; fn GetForegroundWindow() -> H; fn GetWindowRect(w: H, r: *mut Rect) -> i32; }

struct Img { w: usize, h: usize, lum: Vec<u8> }
unsafe fn lire_bitmap(dc: H, bmp: H, w: usize, h: usize) -> Img {
    let mut bmi = Bmi { size: 40, w: w as i32, h: -(h as i32), planes: 1, bpc: 32, comp: 0, img: 0, xp: 0, yp: 0, used: 0, imp: 0 };
    let mut px = vec![0u8; w * h * 4]; GetDIBits(dc, bmp, 0, h as u32, px.as_mut_ptr() as *mut c_void, &mut bmi, 0);
    let lum = px.chunks(4).map(|p| ((p[2] as u32 * 30 + p[1] as u32 * 59 + p[0] as u32 * 11) / 100) as u8).collect(); Img { w, h, lum }
}
unsafe fn capturer(x: i32, y: i32, w: usize, h: usize) -> Img {
    let sc = GetDC(0); let dc = CreateCompatibleDC(sc); let bmp = CreateCompatibleBitmap(sc, w as i32, h as i32); let old = SelectObject(dc, bmp);
    BitBlt(dc, 0, 0, w as i32, h as i32, sc, x, y, 0x00CC0020 | 0x40000000); SelectObject(dc, old);
    let img = lire_bitmap(dc, bmp, w, h); DeleteObject(bmp); DeleteDC(dc); ReleaseDC(0, sc); img
}
fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }
unsafe fn police(taille: i32, qualite: u32) -> H { CreateFontW(-taille, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, qualite, 0, wide("Segoe UI").as_ptr()) }

/// Rend un texte (noir sur blanc, ou clair sur sombre) : sert a l'atlas (un caractere) et aux auto-tests (une phrase).
unsafe fn rendre(txt: &str, taille: i32, qualite: u32, inverse: bool, w: usize, h: usize, ox: i32, oy: i32) -> Img {
    let sc = GetDC(0); let dc = CreateCompatibleDC(sc); let bmp = CreateCompatibleBitmap(sc, w as i32, h as i32); let old = SelectObject(dc, bmp);
    let (fond, encre) = if inverse { (0x00202020u32, 0x00F0F0F0u32) } else { (0x00FFFFFF, 0) };
    SetBkColor(dc, fond); SetTextColor(dc, encre); SetBkMode(dc, 2);
    let pinceau_ok = PatBlt(dc, 0, 0, w as i32, h as i32, if inverse { 0x00000042 } else { 0x00FF0062 }); let _ = pinceau_ok;
    if inverse { // fond gris sombre : on remplit en dessinant un grand espace avec la couleur de fond
        let f0 = police(h as i32, 5); let o0 = SelectObject(dc, f0); let big: Vec<u16> = std::iter::repeat(0x20u16).take(400).collect(); TextOutW(dc, 0, 0, big.as_ptr(), big.len() as i32); SelectObject(dc, o0); DeleteObject(f0); }
    let f = police(taille, qualite); let of = SelectObject(dc, f); let u: Vec<u16> = txt.encode_utf16().collect(); TextOutW(dc, ox, oy, u.as_ptr(), u.len() as i32);
    SelectObject(dc, of); SelectObject(dc, old); let img = lire_bitmap(dc, bmp, w, h); DeleteObject(f); DeleteObject(bmp); DeleteDC(dc); ReleaseDC(0, sc); img
}

struct Glyphe { c: char, adv: i32, w: usize, h: usize, ink: Vec<u8> }
struct Atlas { taille: i32, glyphes: Vec<Glyphe> }
const JEU: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,;:!?()[]{}-_+=/\\%&*'\"@#<>|àâäçéèêëîïôöùûüÿœÀÂÇÉÈÊÎÔÙÛŒ’«»…–—°";
const PAD: i32 = 2;
fn flou(v: &[u8], w: usize) -> Vec<u8> { let mut o = v.to_vec(); for row in v.chunks(w).enumerate() { let (r, ch) = row; for x in 0..w { let a = if x > 0 { ch[x-1] as u32 } else { 0 }; let c = if x + 1 < w { ch[x+1] as u32 } else { 0 }; o[r*w + x] = ((a + 2 * ch[x] as u32 + c) / 4) as u8; } } o }
fn fabriquer_atlas(taille: i32, qual: u32) -> Atlas { unsafe {
    let sc = GetDC(0); let dc = CreateCompatibleDC(sc); let f = police(taille, qual); let of = SelectObject(dc, f);
    let mut glyphes = Vec::new(); let hh = (taille as f32 * 1.45) as usize + 2;
    for c in " ".chars().chain(JEU.chars()) { let mut adv = 0i32; GetCharWidth32W(dc, c as u32, c as u32, &mut adv); if adv <= 0 { continue; }
        let w = (adv + 2 * PAD) as usize; let img = rendre(&c.to_string(), taille, qual, false, w, hh, PAD, 0);
        let brut: Vec<u8> = img.lum.iter().map(|v| 255 - v).collect(); glyphes.push(Glyphe { c, adv, w, h: hh, ink: flou(&brut, w) }); }
    SelectObject(dc, of); DeleteObject(f); DeleteDC(dc); ReleaseDC(0, sc); Atlas { taille, glyphes } } }

fn score(ink: &[u8], iw: usize, ih: usize, g: &Glyphe, x: i32, y: i32) -> f32 {
    if g.c == ' ' { let mut s = 0u32; for yy in 0..ih { for xx in 0..g.adv { let px = x + xx; if px >= 0 && (px as usize) < iw { s += ink[yy * iw + px as usize] as u32; } } } return if (s as f32) / (255.0 * g.adv as f32 * ih as f32) < 0.012 { 0.75 } else { 0.0 }; }
    let (mut mn, mut mx) = (0u32, 0u32);
    for gy in 0..g.h { let yy = y + gy as i32; for gx in 0..g.w { let xx = x - PAD + gx as i32; let a = g.ink[gy * g.w + gx] as u32;
        let b = if xx >= 0 && yy >= 0 && (xx as usize) < iw && (yy as usize) < ih { ink[yy as usize * iw + xx as usize] as u32 } else { 0 };
        mn += a.min(b); mx += a.max(b); } }
    if mx == 0 { 0.0 } else { mn as f32 / mx as f32 }
}
fn mediane(v: &mut [u8]) -> u8 { v.sort_unstable(); v[v.len() / 2] }

/// Lecture d'une image : retourne des lignes (x, y, texte, confiance moyenne).
fn lire(img: &Img, atlas: &[Atlas]) -> Vec<(usize, usize, String, f32)> {
    let (w, h) = (img.w, img.h); let mut res = Vec::new();
    // 1. bandes de lignes : lignes de pixels contenant des contours francs
    let mut occ = vec![0u32; h];
    for y in 1..h { for x in 1..w { let a = img.lum[y * w + x] as i32; if (a - img.lum[y * w + x - 1] as i32).abs() > 60 || (a - img.lum[(y - 1) * w + x] as i32).abs() > 60 { occ[y] += 1; } } }
    let mut y = 0;
    while y < h {
        if occ[y] < 2 { y += 1; continue; }
        let y0 = y; while y < h && occ[y] >= 2 { y += 1; } let y1 = y; let bh = y1 - y0;
        if !(7..=40).contains(&bh) { continue; }
        // 2. encre de la bande : ecart a la couleur de fond (mediane)
        let mut tous: Vec<u8> = (y0..y1).flat_map(|yy| img.lum[yy * w..(yy + 1) * w].iter().copied()).collect(); let bg = mediane(&mut tous) as i32;
        let maxc = (y0..y1).flat_map(|yy| img.lum[yy * w..(yy + 1) * w].iter()).map(|&v| (v as i32 - bg).abs()).max().unwrap_or(1).max(1);
        let ym = y0.saturating_sub(4); let y_max = (y1 + 6).min(h); let bandh = y_max - ym;
        let ink0: Vec<u8> = (ym..y_max).flat_map(|yy| img.lum[yy * w..(yy + 1) * w].iter().map(move |&v| (((v as i32 - bg).abs() * 255) / maxc).min(255) as u8)).collect(); let ink = flou(&ink0, w);
        // 3. mots : colonnes occupees separees par des vides
        let colonne: Vec<bool> = (0..w).map(|x| (y0 - ym..y1 - ym).any(|yy| ink[yy * w + x] > 90)).collect();
        let vide = ((bh as f32) * 0.17).max(2.0) as usize;
        let mut mots: Vec<(usize, usize)> = Vec::new(); let mut x = 0;
        let _ = vide; if let (Some(p), Some(d)) = (colonne.iter().position(|&c| c), colonne.iter().rposition(|&c| c)) { mots.push((p, d + 1)); } let _ = &mut x;
        let mut ligne = String::new(); let (mut somme, mut n, mut xdebut) = (0.0f32, 0u32, usize::MAX); let mut xfin_prec = 0usize;
        for (x0, x1) in mots {
            // 4. reconnaissance du mot : (a) amorces par atlas -> taille/qualite et decalage vertical ; (b) programmation dynamique sur la segmentation du mot
            let mut cands: Vec<(f32, usize, i32)> = Vec::new();
            for (ai, at) in atlas.iter().enumerate().filter(|(_, a)| (a.taille as f32 * 0.9) as i32 <= bh as i32 + 2 && (a.taille as f32 * 1.4) as i32 >= bh as i32) {
                let mut best = (0.0f32, 0i32);
                for g in at.glyphes.iter().filter(|g| g.c != ' ') { for sx in (x0 as i32 - 1)..=(x0 as i32 + 2) { for dy in 0..=(bandh as i32 - at.taille).clamp(1, 14) { let s = score(&ink, w, bandh, g, sx, dy); if s > best.0 { best = (s, dy); } } } }
                if best.0 > 0.4 { cands.push((best.0, ai, best.1)); } }
            cands.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap()); cands.truncate(12);
            let mut meilleur: (f32, String) = (0.0, String::new());
            for (_, ai, dy0) in cands {
                let at = &atlas[ai]; let (xa, xb) = (x0 as i32 - 2, x1 as i32 + 3); let nxs = (xb - xa) as usize + 1;
                let sc: Vec<Vec<f32>> = (0..nxs).map(|i| at.glyphes.iter().map(|g| (-1..=1).map(|d| score(&ink, w, bandh, g, xa + i as i32, dy0 + d)).fold(0.0f32, f32::max)).collect()).collect();
                // best[i] = (valeur, precedent, glyphe) ; i = position d'origine du prochain glyphe
                let mut best: Vec<Option<(f32, usize, usize)>> = vec![None; nxs + 12];
                for i in 0..5.min(nxs) { best[i] = Some((0.0, usize::MAX, usize::MAX)); }
                for i in 0..nxs { let Some((v, _, _)) = best[i] else { continue };
                    for (gi, g) in at.glyphes.iter().enumerate() { let s = sc[i][gi]; if s < 0.3 { continue; }
                        for j in -1i32..=1 { let ni = i as i32 + g.adv + if g.c == ' ' { 0 } else { j }; if ni <= i as i32 || ni as usize >= best.len() { continue; } let nv = v + (s - 0.45) - 0.03 * j.abs() as f32;
                            if best[ni as usize].map_or(true, |b| nv > b.0) { best[ni as usize] = Some((nv, i, gi)); } } }
                    if i + 6 < nxs { let nv = v - 0.6; if best[i + 6].map_or(true, |b| nv > b.0) { best[i + 6] = Some((nv, i, usize::MAX - 1)); } } }
                let fin = (nxs.saturating_sub(7))..(nxs + 1).min(best.len()); let mut fbest: Option<(f32, usize)> = None;
                for i in fin { if let Some((v, _, _)) = best[i] { if fbest.map_or(true, |b| v > b.0) { fbest = Some((v, i)); } } }
                let Some((vtot, mut i)) = fbest else { continue };
                let mut chars: Vec<char> = Vec::new(); let (mut tot, mut cnt) = (0.0f32, 0u32);
                while let Some((_, p, gi)) = best[i] { if p == usize::MAX { break; } if gi == usize::MAX - 1 { chars.push('?'); } else { chars.push(at.glyphes[gi].c); tot += sc[p][gi]; cnt += 1; } i = p; }
                chars.reverse(); let moy = if cnt > 0 { tot / cnt as f32 } else { 0.0 }; let _ = moy; if vtot > meilleur.0 { meilleur = (vtot, chars.into_iter().collect()); } }
            if meilleur.1.is_empty() { continue; }
            if !ligne.is_empty() && x0 > xfin_prec { ligne.push(' '); } if xdebut == usize::MAX { xdebut = x0; } ligne.push_str(&meilleur.1); xfin_prec = x1; somme += meilleur.0; n += 1;
        }
        if n > 0 { res.push((xdebut, y0, ligne, somme / n as f32)); }
    }
    res
}
fn distance(a: &str, b: &str) -> usize { let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect()); let mut d: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() { let mut prev = d[0]; d[0] = i; for j in 1..=b.len() { let t = d[j]; d[j] = (d[j] + 1).min(d[j - 1] + 1).min(prev + (a[i - 1] != b[j - 1]) as usize); prev = t; } } d[b.len()] }

fn main() { unsafe {
    let a: Vec<String> = std::env::args().collect();
    if a.iter().any(|x| x == "--decrire") {
        let im = if let Some(i) = a.iter().position(|x| x == "--fichier") { std::fs::read(&a[i + 1]).ok().and_then(|o| regard::lire_bmp(&o)) } else if a.iter().any(|x| x == "--premier-plan") { let fw = GetForegroundWindow(); let mut r = Rect { l: 0, t: 0, r: 0, b: 0 }; GetWindowRect(fw, &mut r); Some(regard::capturer_rgb(r.l, r.t, (r.r - r.l) as usize, (r.b - r.t) as usize)) } else if let Some(i) = a.iter().position(|x| x == "--ecran") { let g = |k: usize| a.get(i + k).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0); Some(regard::capturer_rgb(g(1), g(2), g(3) as usize, g(4) as usize)) } else { None };
        match im { Some(im) => for l in regard::decrire(&im) { println!("{l}"); }, None => println!("usage : --decrire (--fichier x.bmp | --ecran x y l h | --premier-plan)") } return; }
    let atlas: Vec<Atlas> = (11..=26).flat_map(|t| [fabriquer_atlas(t, 5), fabriquer_atlas(t, 4)]).collect();
    if a.iter().any(|x| x == "--autotest") {
        let phrases = ["Paramètres système", "Bluetooth et appareils", "Mise à jour de Windows 11", "Écran, luminosité : 80 %", "Activer le mode « Hors connexion »", "Se connecter avec un code PIN (recommandé)"];
        let (mut total, mut erreurs) = (0usize, 0usize); let mut cas = 0;
        for &(taille, qual, inv) in &[(16, 5u32, false), (16, 4, false), (16, 5, true), (14, 5, false), (20, 5, false), (13, 5, true)] {
            let (mut t, mut e) = (0usize, 0usize);
            for p in phrases { let img = rendre(p, taille, qual, inv, 700, 40, 10, 8); let l = lire(&img, &atlas); let lu = l.first().map(|x| x.2.clone()).unwrap_or_default();
                let d = distance(p, &lu); t += p.chars().count(); e += d; if d > 0 && std::env::var("VOXA_DEBUG").is_ok() { println!("  attendu: {p}\n  lu     : {lu}"); } }
            cas += 1; println!("cas {cas}: taille {taille} qualite {qual} {} : {} erreurs sur {} caracteres ({:.1} %)", if inv { "clair-sur-sombre" } else { "sombre-sur-clair" }, e, t, 100.0 * (t - e.min(t)) as f64 / t as f64); total += t; erreurs += e; }
        println!("AUTOTEST precision caracteres = {:.1} %", 100.0 * (total - erreurs.min(total)) as f64 / total as f64); return; }
    let (x, y, w, h) = if a.iter().any(|x| x == "--premier-plan") { let fw = GetForegroundWindow(); let mut r = Rect { l: 0, t: 0, r: 0, b: 0 }; GetWindowRect(fw, &mut r); (r.l, r.t, (r.r - r.l) as usize, (r.b - r.t) as usize) }
        else if let Some(i) = a.iter().position(|x| x == "--ecran") { let g = |k: usize| a.get(i + k).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0); (g(1), g(2), g(3) as usize, g(4) as usize) } else { println!("usage : --autotest | --ecran x y l h | --premier-plan [--compter]"); return };
    let img = capturer(x, y, w, h); let lignes = lire(&img, &atlas);
    if a.iter().any(|x| x == "--compter") { println!("LIGNES={} MOTS={} CONFIANCE_MOY={:.2}", lignes.len(), lignes.iter().map(|l| l.2.split(' ').count()).sum::<usize>(), lignes.iter().map(|l| l.3).sum::<f32>() / lignes.len().max(1) as f32); }
    else { for (lx, ly, t, c) in lignes { println!("{},{}: {} ({:.2})", lx, ly, t, c); } }
} }
