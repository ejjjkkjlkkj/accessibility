//! Regard : decrit une image comme le ferait une personne qui regarde l'ecran : ou sont les images, de quelles couleurs, quelle part de tons chair, quelle luminosite, quelles formes dominantes.
//! Aucun modele d'IA : mesures directes sur les pixels. Il NE PEUT PAS dire ce que represente une image (personne, objet) : cela exige un modele entraine (voir VOXA-VISION.md).
use super::*;

pub struct Rgb { pub w: usize, pub h: usize, pub px: Vec<[u8; 3]> }

pub unsafe fn capturer_rgb(x: i32, y: i32, w: usize, h: usize) -> Rgb {
    let sc = GetDC(0); let dc = CreateCompatibleDC(sc); let bmp = CreateCompatibleBitmap(sc, w as i32, h as i32); let old = SelectObject(dc, bmp);
    BitBlt(dc, 0, 0, w as i32, h as i32, sc, x, y, 0x00CC0020 | 0x40000000); SelectObject(dc, old);
    let mut bmi = Bmi { size: 40, w: w as i32, h: -(h as i32), planes: 1, bpc: 32, comp: 0, img: 0, xp: 0, yp: 0, used: 0, imp: 0 };
    let mut buf = vec![0u8; w * h * 4]; GetDIBits(dc, bmp, 0, h as u32, buf.as_mut_ptr() as *mut c_void, &mut bmi, 0);
    DeleteObject(bmp); DeleteDC(dc); ReleaseDC(0, sc);
    Rgb { w, h, px: buf.chunks(4).map(|p| [p[2], p[1], p[0]]).collect() }
}

/// Lecture d'un BMP non compresse 24 ou 32 bits (pour tester sur des images connues).
pub fn lire_bmp(o: &[u8]) -> Option<Rgb> {
    if o.len() < 54 || &o[0..2] != b"BM" { return None; }
    let off = u32::from_le_bytes(o[10..14].try_into().ok()?) as usize; let w = i32::from_le_bytes(o[18..22].try_into().ok()?); let hh = i32::from_le_bytes(o[22..26].try_into().ok()?);
    let bpp = u16::from_le_bytes(o[28..30].try_into().ok()?); if (bpp != 24 && bpp != 32) || w <= 0 || hh == 0 { return None; }
    let (w, h, bas) = (w as usize, hh.unsigned_abs() as usize, hh > 0); let oct = bpp as usize / 8; let ligne = (w * oct + 3) & !3; let mut px = vec![[0u8; 3]; w * h];
    for y in 0..h { let src = if bas { h - 1 - y } else { y }; for x in 0..w { let i = off + src * ligne + x * oct; if i + 2 >= o.len() { return None; } px[y * w + x] = [o[i + 2], o[i + 1], o[i]]; } }
    Some(Rgb { w, h, px })
}

fn hsv(p: [u8; 3]) -> (f32, f32, f32) {
    let (r, g, b) = (p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0); let mx = r.max(g).max(b); let mn = r.min(g).min(b); let d = mx - mn;
    let h = if d == 0.0 { 0.0 } else if mx == r { 60.0 * (((g - b) / d) % 6.0) } else if mx == g { 60.0 * ((b - r) / d + 2.0) } else { 60.0 * ((r - g) / d + 4.0) };
    ((h + 360.0) % 360.0, if mx == 0.0 { 0.0 } else { d / mx }, mx)
}
fn nom_couleur(p: [u8; 3]) -> &'static str {
    let (h, s, v) = hsv(p);
    if v < 0.15 { return "noir"; } if s < 0.12 { return if v > 0.85 { "blanc" } else if v > 0.55 { "gris clair" } else { "gris foncé" }; }
    if (10.0..45.0).contains(&h) && s < 0.65 && v > 0.35 { return if v > 0.75 && s < 0.35 { "beige" } else if v < 0.5 { "marron" } else { "tons chair/beige" }; }
    match h { x if !(15.0..345.0).contains(&x) => "rouge", x if x < 45.0 => "orange", x if x < 70.0 => "jaune", x if x < 165.0 => "vert", x if x < 200.0 => "cyan", x if x < 260.0 => "bleu", x if x < 290.0 => "violet", _ => "rose/magenta" }
}
/// Teinte "peau" : regle classique en HSV (toutes carnations confondues), c'est un indice de couleur, pas une identification.
fn est_chair(p: [u8; 3]) -> bool { let (h, s, v) = hsv(p); (h <= 50.0 || h >= 350.0) && s > 0.15 && s < 0.75 && v > 0.30 }

const C: usize = 8; // cellule de 8x8 pixels
#[derive(Clone, Copy, PartialEq)] enum Nature { Plat, Texte, Image }

pub fn decrire(im: &Rgb) -> Vec<String> {
    let (cw, ch) = (im.w / C, im.h / C); if cw == 0 || ch == 0 { return vec!["Image trop petite.".into()]; }
    let mut nat = vec![Nature::Plat; cw * ch];
    for cy in 0..ch { for cx in 0..cw {
        let mut couleurs: std::collections::HashSet<u16> = std::collections::HashSet::new(); let (mut sat, mut grad) = (0.0f32, 0.0f32);
        for y in 0..C { for x in 0..C { let p = im.px[(cy * C + y) * im.w + cx * C + x]; couleurs.insert(((p[0] >> 4) as u16) << 8 | ((p[1] >> 4) as u16) << 4 | (p[2] >> 4) as u16); sat += hsv(p).1;
            if x > 0 { let q = im.px[(cy * C + y) * im.w + cx * C + x - 1]; grad += ((p[0] as i32 - q[0] as i32).abs() + (p[1] as i32 - q[1] as i32).abs() + (p[2] as i32 - q[2] as i32).abs()) as f32; } } }
        let sat = sat / (C * C) as f32; let grad = grad / (C * (C - 1)) as f32; let nb = couleurs.len();
        nat[cy * cw + cx] = if nb <= 3 && grad < 6.0 { Nature::Plat } else if sat < 0.15 && nb <= 14 && grad > 25.0 { Nature::Texte } else if nb >= 8 && (sat > 0.12 || grad > 4.0) && !(sat < 0.15 && grad > 25.0) { Nature::Image } else if grad > 25.0 { Nature::Texte } else { Nature::Plat };
    } }
    // composantes connexes de cellules "image" (4-voisinage) ; on elimine les petites (anti-aliasing de l'interface)
    let mut vu = vec![false; cw * ch]; let mut regions: Vec<(usize, usize, usize, usize, usize)> = Vec::new();
    for s in 0..cw * ch { if vu[s] || nat[s] != Nature::Image { continue; } let (mut x0, mut y0, mut x1, mut y1, mut n) = (cw, ch, 0, 0, 0); let mut pile = vec![s]; vu[s] = true;
        while let Some(i) = pile.pop() { let (x, y) = (i % cw, i / cw); x0 = x0.min(x); x1 = x1.max(x); y0 = y0.min(y); y1 = y1.max(y); n += 1;
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] { let (nx, ny) = (x as i32 + dx, y as i32 + dy); if nx < 0 || ny < 0 || nx >= cw as i32 || ny >= ch as i32 { continue; } let j = ny as usize * cw + nx as usize; if !vu[j] && nat[j] == Nature::Image { vu[j] = true; pile.push(j); } } }
        if n >= 20 && (x1 - x0 + 1) >= 5 && (y1 - y0 + 1) >= 5 { regions.push((x0, y0, x1, y1, n)); } }
    let mut out = Vec::new(); let textes = nat.iter().filter(|&&n| n == Nature::Texte).count(); let total = cw * ch;
    out.push(format!("Zone de {} sur {} pixels : {} région(s) d'image, {} % de texte, {} % de fond uni.", im.w, im.h, regions.len(), 100 * textes / total, 100 * nat.iter().filter(|&&n| n == Nature::Plat).count() / total));
    regions.sort_by(|a, b| b.4.cmp(&a.4));
    for (i, &(x0, y0, x1, y1, _)) in regions.iter().enumerate().take(8) {
        let (px0, py0, px1, py1) = (x0 * C, y0 * C, (x1 + 1) * C, (y1 + 1) * C); let (rw, rh) = (px1 - px0, py1 - py0);
        let mut compte: std::collections::HashMap<&str, usize> = std::collections::HashMap::new(); let (mut chair, mut n, mut lum) = (0usize, 0usize, 0.0f32);
        let (mut cdx, mut cdy, mut cs) = (0.0f64, 0.0f64, 0.0f64); // centre de masse des pixels "chair"
        for y in (py0..py1.min(im.h)).step_by(2) { for x in (px0..px1.min(im.w)).step_by(2) { let p = im.px[y * im.w + x]; *compte.entry(nom_couleur(p)).or_insert(0) += 1; n += 1; lum += hsv(p).2; if est_chair(p) { chair += 1; cdx += x as f64; cdy += y as f64; cs += 1.0; } } }
        let mut v: Vec<_> = compte.into_iter().collect(); v.sort_by(|a, b| b.1.cmp(&a.1));
        let dom: Vec<String> = v.iter().take(3).map(|(k, c)| format!("{} {} %", k, 100 * c / n.max(1))).collect();
        let cxr = (px0 + px1) as f32 / 2.0 / im.w as f32; let cyr = (py0 + py1) as f32 / 2.0 / im.h as f32;
        let pos = format!("{}{}", if cyr < 0.33 { "en haut" } else if cyr > 0.66 { "en bas" } else { "au milieu" }, if cxr < 0.33 { " à gauche" } else if cxr > 0.66 { " à droite" } else { "" });
        let lumi = lum / n.max(1) as f32; let forme = if rw as f32 / rh as f32 > 1.8 { "large et basse" } else if (rh as f32 / rw as f32) > 1.8 { "haute et étroite" } else { "de proportions proches du carré" };
        let mut s = format!("Image {} : {} {} sur {} pixels ({}), couleurs dominantes : {} ; {}.", i + 1, pos, rw, rh, forme, dom.join(", "), if lumi > 0.7 { "image lumineuse" } else if lumi < 0.35 { "image sombre" } else { "luminosité moyenne" });
        let pc = 100 * chair / n.max(1); if pc >= 8 { let mx = (cdx / cs) as f32 / im.w as f32; let my = (cdy / cs) as f32 / im.h as f32; s += &format!(" Teintes de peau sur {} % de la surface, concentrées {}{}. (Indice de couleur seulement : je ne peux pas dire ce que l'image représente.)", pc, if my < 0.4 { "vers le haut" } else if my > 0.6 { "vers le bas" } else { "vers le milieu" }, if mx < 0.4 { " à gauche" } else if mx > 0.6 { " à droite" } else { "" }); }
        out.push(s);
    }
    out
}
