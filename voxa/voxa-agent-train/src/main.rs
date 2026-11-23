//! Entrainement du modele d'intentions de Voxa Agent. Usage : voxa-agent-train <dossier-sortie> [graine]
//! Protocole (VOXA-MODEL-PROGRAMME.md) : apprentissage sur TR (augmente), choix de l'epoque et du seuil sur VA, mesure finale UNIQUE sur TE avec d'autres tournures et d'autres mots parasites ;
//! reference a battre = les regles de repli ; le modele n'est ecrit dans voxa-agent/data que s'il bat la reference.
mod donnees;
use donnees::*;
use voxa_agent::{caracteristiques, choisir, regles, Intention, Modele, C, D, H};
use voxa_neural::Q16;

struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } fn unit(&mut self) -> f32 { (self.next() >> 40) as f32 / (1u64 << 24) as f32 } fn gauss(&mut self) -> f32 { let s: f32 = (0..6).map(|_| self.unit()).sum(); (s - 3.0) * 1.4 } fn below(&mut self, n: usize) -> usize { (self.next() % n as u64) as usize } }

/// Variante realiste d'une phrase : mot parasite avant ou apres, faute de frappe, majuscules.
fn varier(p: &str, rng: &mut Rng, mots: &[&str]) -> String {
    let mut s = p.to_string();
    match rng.below(4) { 0 => {} 1 => s = format!("{} {}", mots[rng.below(mots.len())], s), 2 => s = format!("{} {}", s, mots[rng.below(mots.len())]), _ => s = format!("{} {} {}", mots[rng.below(mots.len())], s, mots[rng.below(mots.len())]) }
    if rng.below(3) == 0 { let mut c: Vec<char> = s.chars().collect(); if c.len() > 4 { let i = 1 + rng.below(c.len() - 2); if rng.below(2) == 0 { c.remove(i); } else { c.swap(i, i - 1); } } s = c.into_iter().collect(); }
    if rng.below(8) == 0 { s = s.to_uppercase(); }
    s
}

type Ex = (Vec<(usize, f32)>, usize, String);
fn creuse(texte: &str) -> Vec<(usize, f32)> { let mut x = [0i8; D]; caracteristiques(texte, &mut x); x.iter().enumerate().filter(|(_, v)| **v != 0).map(|(i, v)| (i, *v as f32 / 40.0)).collect() }
fn construire(ens: &[&[&str]; 15], n_var: usize, mots: &[&str], rng: &mut Rng) -> Vec<Ex> {
    let mut v = Vec::new();
    for (c, ps) in ens.iter().enumerate() { for p in ps.iter() { v.push((creuse(p), c, p.to_string())); for _ in 0..n_var { let t = varier(p, rng, mots); v.push((creuse(&t), c, t)); } } }
    v
}

struct Reseau { w1: Vec<f32>, b1: Vec<f32>, w2: Vec<f32>, b2: Vec<f32> } // w1 : D x H (primitive d'abord), w2 : C x H
impl Reseau {
    fn nouveau(rng: &mut Rng) -> Reseau { Reseau { w1: (0..D * H).map(|_| rng.gauss() * 0.05).collect(), b1: vec![0.0; H], w2: (0..C * H).map(|_| rng.gauss() * 0.1).collect(), b2: vec![0.0; C] } }
    fn cachee(&self, x: &[(usize, f32)]) -> [f32; H] { let mut z = [0f32; H]; z.copy_from_slice(&self.b1); for &(i, v) in x { let r = &self.w1[i * H..(i + 1) * H]; for j in 0..H { z[j] += r[j] * v; } } z }
    fn logits(&self, h: &[f32; H]) -> [f32; C] { let mut o = [0f32; C]; for c in 0..C { let mut s = self.b2[c]; for j in 0..H { s += self.w2[c * H + j] * h[j].max(0.0); } o[c] = s; } o }
    fn predire(&self, x: &[(usize, f32)]) -> usize { let o = self.logits(&self.cachee(x)); o.iter().enumerate().fold((0, f32::MIN), |b, (i, v)| if *v > b.1 { (i, *v) } else { b }).0 }
    fn pas(&mut self, x: &[(usize, f32)], y: usize, lr: f32) {
        let z = self.cachee(x); let o = self.logits(&z); let m = o.iter().cloned().fold(f32::MIN, f32::max); let mut p = [0f32; C]; let mut s = 0.0; for c in 0..C { p[c] = (o[c] - m).exp(); s += p[c]; } for c in 0..C { p[c] /= s; }
        let mut dh = [0f32; H];
        for c in 0..C { let g = p[c] - if c == y { 1.0 } else { 0.0 }; for j in 0..H { dh[j] += g * self.w2[c * H + j]; self.w2[c * H + j] -= lr * (g * z[j].max(0.0) + 1e-5 * self.w2[c * H + j]); } self.b2[c] -= lr * g; }
        for j in 0..H { if z[j] <= 0.0 { dh[j] = 0.0; } }
        for j in 0..H { self.b1[j] -= lr * dh[j]; }
        for &(i, v) in x { let r = &mut self.w1[i * H..(i + 1) * H]; for j in 0..H { r[j] -= lr * (dh[j] * v + 1e-5 * r[j]); } }
    }
}

/// Quantification int8 (formules dans VOXA-MODEL-PROGRAMME.md) : entree = compte x 40 ; echelles Q16 des deux couches ; la marge est comptee dans l'unite des logits int8.
fn quantifier(r: &Reseau, entrainement: &[Ex]) -> Vec<u8> {
    // echelle du cache : percentile eleve des pre-activations positives
    let mut zs: Vec<f32> = Vec::new(); for (x, _, _) in entrainement.iter().step_by(3) { for v in r.cachee(x) { if v > 0.0 { zs.push(v); } } } zs.sort_by(|a, b| a.partial_cmp(b).unwrap()); let zmax = zs[(zs.len() as f32 * 0.999) as usize].max(1e-3); let sh = zmax / 100.0;
    let sw1 = r.w1.iter().fold(0f32, |m, v| m.max(v.abs())) / 127.0; let sw2 = r.w2.iter().fold(0f32, |m, v| m.max(v.abs())) / 127.0;
    let mut lmax = 0f32; for (x, _, _) in entrainement.iter().step_by(3) { for v in r.logits(&r.cachee(x)) { lmax = lmax.max(v.abs()); } } let so = lmax.max(1e-3) / 100.0;
    let s1 = Q16::from_f32(sw1 / (40.0 * sh)).expect("echelle 1"); let s2 = Q16::from_f32(sw2 * sh / so).expect("echelle 2");
    let mut b = Vec::new(); b.extend_from_slice(b"VAGT"); for v in [D as u16, H as u16, C as u16] { b.extend_from_slice(&v.to_le_bytes()); } b.push(0); b.push(0); b.extend_from_slice(&s1.0.to_le_bytes()); b.extend_from_slice(&s2.0.to_le_bytes());
    // poids de la couche 1 rangés par SORTIE : w1q[j][i]
    for j in 0..H { for i in 0..D { b.push(((r.w1[i * H + j] / sw1).round().clamp(-127.0, 127.0) as i8) as u8); } }
    for j in 0..H { b.extend_from_slice(&((r.b1[j] * 40.0 / sw1).round() as i32).to_le_bytes()); }
    for c in 0..C { for j in 0..H { b.push(((r.w2[c * H + j] / sw2).round().clamp(-127.0, 127.0) as i8) as u8); } }
    for c in 0..C { b.extend_from_slice(&((r.b2[c] / (sw2 * sh)).round() as i32).to_le_bytes()); }
    b
}

fn pred_int8(m: &Modele, texte: &str, seuil: i32) -> (usize, i32) { let mut x = [0i8; D]; caracteristiques(texte, &mut x); match m.predire_x(&x) { Some((c, marge)) => (if marge >= seuil { c } else { 0 }, marge), None => (0, 0) } }

fn main() {
    let a: Vec<String> = std::env::args().collect(); let sortie = a.get(1).cloned().unwrap_or_else(|| ".".into()); let graine: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(20261012);
    let mut rng = Rng(graine); let trr: [&[&str]; 15] = TR; let var: [&[&str]; 15] = VA; let ter: [&[&str]; 15] = TE;
    let train = construire(&trr, 40, &PREFIXES_TR, &mut rng); let val = construire(&var, 6, &PREFIXES_TR, &mut rng);
    println!("Donnees : apprentissage {} exemples (de {} phrases ecrites), validation {} exemples (de {} phrases).", train.len(), trr.iter().map(|x| x.len()).sum::<usize>(), val.len(), var.iter().map(|x| x.len()).sum::<usize>());
    let mut net = Reseau::nouveau(&mut rng); let mut meilleur = (0.0f32, 0usize, Reseau { w1: vec![], b1: vec![], w2: vec![], b2: vec![] });
    let mut ordre: Vec<usize> = (0..train.len()).collect();
    for ep in 0..40 {
        for i in (1..ordre.len()).rev() { let j = rng.below(i + 1); ordre.swap(i, j); }
        let lr = 0.05 * (1.0 - ep as f32 / 50.0);
        for &i in &ordre { let (x, y, _) = &train[i]; net.pas(x, *y, lr); }
        let acc = val.iter().filter(|(x, y, _)| net.predire(x) == *y).count() as f32 / val.len() as f32;
        if acc > meilleur.0 { meilleur = (acc, ep, Reseau { w1: net.w1.clone(), b1: net.b1.clone(), w2: net.w2.clone(), b2: net.b2.clone() }); }
        if ep % 8 == 0 || ep == 39 { println!("  epoque {ep:>2} : validation flottante {:.1} %", 100.0 * acc); }
    }
    println!("Meilleure epoque : {} (validation flottante {:.1} %).", meilleur.1, 100.0 * meilleur.0);
    let net = meilleur.2;
    // quantification et choix du seuil sur la validation
    let mut octets = quantifier(&net, &train); let modele_tmp: &'static [u8] = Box::leak(octets.clone().into_boxed_slice()); let m = Modele::depuis_octets(modele_tmp).expect("modele invalide");
    let mut meilleur_seuil = (0.0f32, 0i32); for s in 0..=100 { let acc = val.iter().filter(|(_, y, t)| pred_int8(&m, t, s).0 == *y).count() as f32 / val.len() as f32; if acc > meilleur_seuil.0 + 1e-6 { meilleur_seuil = (acc, s); } }
    println!("Quantification int8 : validation {:.1} % avec le seuil de marge {} (choisi sur la validation).", 100.0 * meilleur_seuil.0, meilleur_seuil.1); octets[10] = meilleur_seuil.1 as i8 as u8;
    let modele_final: &'static [u8] = Box::leak(octets.clone().into_boxed_slice()); let m = Modele::depuis_octets(modele_final).expect("modele final"); let seuil = meilleur_seuil.1;
    // MESURE FINALE UNIQUE sur le test : originaux + variantes avec d'AUTRES mots parasites
    let mut rt = Rng(graine ^ 0xABCDEF); let test = construire(&ter, 3, &PREFIXES_TE, &mut rt);
    let mut conf_modele = 0; let mut conf_regles = 0; let mut conf_hybride = 0; let (mut risque_modele, mut risque_regles, mut risque_hybride) = (0, 0, 0); let mut erreurs: Vec<String> = Vec::new();
    for (_, y, t) in &test {
        let pm = pred_int8(&m, t, seuil).0; let pr = regles(t) as usize; let ph = choisir(Intention::depuis(pm), Intention::depuis(pr)).0 as usize;
        if pm == *y { conf_modele += 1; } if pr == *y { conf_regles += 1; } if ph == *y { conf_hybride += 1; }
        let risq = |p: usize| (p == Intention::Eteindre as usize || p == Intention::Redemarrer as usize) && p != *y;
        if risq(pm) { risque_modele += 1; } if risq(pr) { risque_regles += 1; } if risq(ph) { risque_hybride += 1; }
        if ph != *y && erreurs.len() < 12 { erreurs.push(format!("   « {t} » : attendu {} predit {} (modele {} / regles {})", y, ph, pm, pr)); }
    }
    let n = test.len() as f32;
    println!("\nMESURE FINALE sur {} phrases de test (jamais vues, autres tournures, autres mots parasites, fautes de frappe) :", test.len());
    println!("  regles de reference : {:.1} % ; faux declenchements d'actions a risque : {}", 100.0 * conf_regles as f32 / n, risque_regles);
    println!("  modele int8 seul    : {:.1} % ; faux declenchements d'actions a risque : {}", 100.0 * conf_modele as f32 / n, risque_modele);
    println!("  agent (modele puis regles) : {:.1} % ; faux declenchements d'actions a risque : {}", 100.0 * conf_hybride as f32 / n, risque_hybride);
    for e in &erreurs { println!("{e}"); }
    let retenu = conf_hybride > conf_regles && risque_hybride <= risque_regles;
    println!("\nDECISION : {}", if retenu { "le modele bat la reference sur le test : RETENU (voxa-agent/data/modele.bin)" } else { "le modele ne bat pas la reference : NON retenu, les regles sont conservees (modele-candidat.bin ecrit a titre d'archive)" });
    let nom = if retenu { "modele.bin" } else { "modele-candidat.bin" }; std::fs::create_dir_all(&sortie).ok(); std::fs::write(format!("{sortie}/{nom}"), &octets).expect("ecriture");
    println!("Fichier : {sortie}/{nom} ({} octets). Graine {graine}.", octets.len());
}
