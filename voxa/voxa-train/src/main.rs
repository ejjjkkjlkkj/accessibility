//! Entraînement et évaluation du modèle de phonétisation de Voxa (expérience v0).
//! Usage : voxa-train <lexique> <baseline.txt> <sortie-dossier> [graine]
//!   <baseline.txt> : lignes « mot<TAB>phonétique produite par phon.vx » (la référence à battre).
//! Séparation apprentissage / validation / test : par empreinte FNV-1a du mot (déterministe, indépendante de l'ordre du fichier).
//! Le jeu de test n'est utilisé qu'UNE fois, après le choix de l'époque sur la validation.
use std::collections::{BTreeMap, HashMap};
use std::fs;

const V: usize = 41; // alphabet : 26 lettres, 13 accentuées, frontière, inconnu
const W: usize = 7; // fenêtre de lettres : 3 avant, la lettre, 3 après
// neurones cachés : 48 par défaut, modifiable à la compilation par la variable d environnement VOXA_H (expérience de capacité)
const fn parse_h(s: Option<&str>) -> usize { match s { None => 48, Some(t) => { let b = t.as_bytes(); let mut i = 0; let mut v = 0usize; while i < b.len() { v = v * 10 + (b[i] - b'0') as usize; i += 1; } v } } }
const H: usize = parse_h(option_env!("VOXA_H"));

fn sym(c: char) -> usize {
    match c {
        'a'..='z' => c as usize - 'a' as usize,
        'é' => 26, 'è' => 27, 'ê' => 28, 'à' => 29, 'â' => 30, 'î' => 31, 'ô' => 32, 'û' => 33, 'ù' => 34, 'ç' => 35, 'ë' => 36, 'ï' => 37, 'œ' => 38,
        '#' => 39,
        _ => 40,
    }
}
fn fnv(s: &str) -> u32 { let mut h = 2166136261u32; for b in s.bytes() { h ^= b as u32; h = h.wrapping_mul(16777619); } h }
fn split_of(w: &str) -> u8 { let k = fnv(w) % 100; if k < 70 { 0 } else if k < 85 { 1 } else { 2 } } // 0 apprentissage, 1 validation, 2 test

struct Entry { word: String, letters: Vec<char>, labels: Vec<String>, gold: String }

fn load(path: &str) -> (Vec<Entry>, Vec<String>) {
    let txt = fs::read_to_string(path).expect("lexique illisible");
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    let (mut out, mut errs) = (Vec::new(), Vec::new());
    for (n, line) in txt.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let Some((word, rest)) = line.split_once(':') else { errs.push(format!("ligne {}: pas de « : »", n + 1)); continue };
        let word = word.trim().to_string();
        if word.contains('\'') { continue; } // les mots avec apostrophe ne sont pas des unités du modèle
        if seen.insert(word.clone(), ()).is_some() { continue; } // doublon : première occurrence conservée
        let (mut letters, mut labels, mut gold) = (Vec::new(), Vec::new(), String::new());
        let mut ok = true;
        for chunk in rest.split_whitespace() {
            let Some((g, p)) = chunk.split_once('=') else { ok = false; break };
            let ph = if p == "_" { String::new() } else { p.to_string() };
            for (i, c) in g.chars().enumerate() { letters.push(c); labels.push(if i == 0 { ph.clone() } else { String::new() }); }
            gold.push_str(&ph);
        }
        let joined: String = letters.iter().collect();
        if !ok || joined != word { errs.push(format!("ligne {}: les graphèmes « {} » ne redonnent pas « {} »", n + 1, joined, word)); continue; }
        out.push(Entry { word, letters, labels, gold });
    }
    let classes: Vec<String> = { let mut s: Vec<String> = out.iter().flat_map(|e| e.labels.iter().cloned()).collect(); s.sort(); s.dedup(); s };
    (out, classes)
}

fn window(letters: &[char], i: usize) -> [usize; W] {
    let mut w = [39usize; W];
    for k in 0..W { let j = i as isize + k as isize - 3; if j >= 0 && (j as usize) < letters.len() { w[k] = sym(letters[j as usize]); } }
    w
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 }
    fn unit(&mut self) -> f32 { (self.next() >> 40) as f32 / (1u64 << 24) as f32 }
    fn gauss(&mut self) -> f32 { let s: f32 = (0..6).map(|_| self.unit()).sum(); (s - 3.0) * 1.4 }
}

struct Net { c: usize, w1: Vec<f32>, b1: Vec<f32>, w2: Vec<f32>, b2: Vec<f32> }
impl Net {
    fn new(c: usize, rng: &mut Rng) -> Net {
        let s1 = (2.0 / (W as f32 * 2.0)).sqrt(); let s2 = (2.0 / H as f32).sqrt();
        Net { c, w1: (0..H * W * V).map(|_| rng.gauss() * s1 * 0.5).collect(), b1: vec![0.0; H], w2: (0..c * H).map(|_| rng.gauss() * s2 * 0.5).collect(), b2: vec![0.0; c] }
    }
    fn hidden(&self, x: &[usize; W]) -> [f32; H] {
        let mut h = [0f32; H];
        for j in 0..H { let mut s = self.b1[j]; for (k, &v) in x.iter().enumerate() { s += self.w1[j * W * V + k * V + v]; } h[j] = s.max(0.0); }
        h
    }
    fn logits(&self, h: &[f32; H]) -> Vec<f32> { (0..self.c).map(|o| { let mut s = self.b2[o]; for j in 0..H { s += self.w2[o * H + j] * h[j]; } s }).collect() }
    fn predict(&self, x: &[usize; W]) -> usize { let l = self.logits(&self.hidden(x)); l.iter().enumerate().fold((0, f32::MIN), |b, (i, v)| if *v > b.1 { (i, *v) } else { b }).0 }
}

fn lev(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut d: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() { let mut prev = d[0]; d[0] = i; for j in 1..=b.len() { let t = d[j]; d[j] = (d[j] + 1).min(d[j - 1] + 1).min(prev + (a[i - 1] != b[j - 1]) as usize); prev = t; } }
    d[b.len()]
}

#[derive(Default, Clone, Copy)]
struct Score { words: usize, exact: usize, edits: usize, chars: usize }
impl Score {
    fn add(&mut self, pred: &str, gold: &str) { self.words += 1; if pred == gold { self.exact += 1; } self.edits += lev(pred, gold); self.chars += gold.chars().count(); }
    fn show(&self) -> String { format!("{}/{} mots exacts ({:.1} %), erreur de caractères phonétiques {:.1} %", self.exact, self.words, 100.0 * self.exact as f64 / self.words.max(1) as f64, 100.0 * self.edits as f64 / self.chars.max(1) as f64) }
}

fn decode(net: &Net, classes: &[String], e: &Entry) -> String { (0..e.letters.len()).map(|i| classes[net.predict(&window(&e.letters, i))].as_str()).collect() }

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() >= 4 && a[1] == "eval" {
        // voxa-train eval <lexique> <mot<TAB>phonétique> : mesure une référence sur tout un lexique et liste les écarts
        let (entries, _) = load(&a[2]);
        let base: HashMap<String, String> = fs::read_to_string(&a[3]).unwrap_or_default().lines().filter_map(|l| l.split_once('\t').map(|(w, p)| (w.to_string(), p.trim().replace(' ', "")))).collect();
        let mut s = Score::default();
        for e in &entries { let p = base.get(&e.word).cloned().unwrap_or_default(); s.add(&p, &e.gold); if p != e.gold { println!("ÉCART {}\tor={}\tphon={}", e.word, e.gold, p); } }
        println!("{} mots : {}", entries.len(), s.show());
        return;
    }
    if a.len() < 4 { eprintln!("Usage : voxa-train <lexique> <baseline.txt> <sortie> [graine]"); std::process::exit(2); }
    let seed: u64 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(20261011);
    // fraction du jeu d apprentissage utilisée (courbe d apprentissage) : sous-ensemble déterministe par empreinte du mot ; validation et test inchangés
    let frac: f64 = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let in_frac = |w: &str| ((fnv(w) / 100) % 100) as f64 / 100.0 < frac;
    let (entries, classes) = load(&a[1]);
    let base: HashMap<String, String> = fs::read_to_string(&a[2]).unwrap_or_default().lines().filter_map(|l| l.split_once('\t').map(|(w, p)| (w.to_string(), p.trim().replace(' ', "")))).collect();
    fs::create_dir_all(&a[3]).ok();
    let cidx: HashMap<&str, usize> = classes.iter().enumerate().map(|(i, c)| (c.as_str(), i)).collect();
    let mut report = String::new();
    macro_rules! say { ($($t:tt)*) => {{ let s = format!($($t)*); println!("{s}"); report.push_str(&s); report.push('\n'); }} }
    let n = [0u8, 1, 2].map(|k| entries.iter().filter(|e| split_of(&e.word) == k).count());
    say!("Lexique : {} mots valides ; classes de sortie : {} ; graine : {seed}", entries.len(), classes.len());
    say!("Fraction du jeu d apprentissage utilisée : {:.0} %", frac * 100.0);
    say!("Séparation par empreinte du mot : apprentissage {}, validation {}, test {}", n[0], n[1], n[2]);
    let train: Vec<(&Entry, usize)> = entries.iter().filter(|e| split_of(&e.word) == 0 && in_frac(&e.word)).flat_map(|e| (0..e.letters.len()).map(move |i| (e, i))).collect();
    let mut rng = Rng(seed | 1);
    let mut net = Net::new(classes.len(), &mut rng);
    let mut order: Vec<usize> = (0..train.len()).collect();
    let (mut best_val, mut best_ep) = (-1.0f64, 0usize);
    let mut best_net: Option<(Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>)> = None;
    let epochs: usize = a.get(6).and_then(|s| s.parse().ok()).unwrap_or(400);
    let lr0: f32 = a.get(7).and_then(|s| s.parse().ok()).unwrap_or(0.03);
    let mom = 0.9f32;
    say!("Capacité : {H} neurones cachés ; époques {epochs} ; pas initial {lr0}");
    let mut vel = (vec![0f32; net.w1.len()], vec![0f32; H], vec![0f32; net.w2.len()], vec![0f32; net.b2.len()]);
    for ep in 0..epochs {
        let lr = lr0 * (1.0 - ep as f32 / epochs as f32) + 0.002;
        for i in (1..order.len()).rev() { let j = (rng.next() % (i as u64 + 1)) as usize; order.swap(i, j); }
        for &t in &order {
            let (e, i) = train[t];
            let x = window(&e.letters, i);
            let y = cidx[e.labels[i].as_str()];
            let h = net.hidden(&x);
            let mut p = net.logits(&h);
            let mx = p.iter().cloned().fold(f32::MIN, f32::max);
            let z: f32 = p.iter().map(|v| (v - mx).exp()).sum();
            for v in p.iter_mut() { *v = (*v - mx).exp() / z; }
            p[y] -= 1.0; // gradient des logits
            let mut gh = [0f32; H];
            for o in 0..net.c { let g = p[o]; if g == 0.0 { continue; } for j in 0..H { gh[j] += g * net.w2[o * H + j]; let k = o * H + j; vel.2[k] = mom * vel.2[k] - lr * (g * h[j] + 1e-5 * net.w2[k]); net.w2[k] += vel.2[k]; } vel.3[o] = mom * vel.3[o] - lr * g; net.b2[o] += vel.3[o]; }
            for j in 0..H { if h[j] <= 0.0 { continue; } let g = gh[j]; for (k, &v) in x.iter().enumerate() { let idx = j * W * V + k * V + v; vel.0[idx] = mom * vel.0[idx] - lr * g; net.w1[idx] += vel.0[idx]; } vel.1[j] = mom * vel.1[j] - lr * g; net.b1[j] += vel.1[j]; }
        }
        if ep % 10 == 9 {
            let mut s = Score::default();
            for e in entries.iter().filter(|e| split_of(&e.word) == 1) { s.add(&decode(&net, &classes, e), &e.gold); }
            let acc = 1.0 - s.edits as f64 / s.chars.max(1) as f64;
            if acc > best_val { best_val = acc; best_ep = ep + 1; best_net = Some((net.w1.clone(), net.b1.clone(), net.w2.clone(), net.b2.clone())); }
        }
    }
    let (w1, b1, w2, b2) = best_net.unwrap();
    net.w1 = w1; net.b1 = b1; net.w2 = w2; net.b2 = b2;
    say!("Époque retenue (meilleure validation) : {best_ep} / {epochs}");
    let mut res = [(Score::default(), Score::default()); 3]; // (modèle, référence phon.vx)
    let mut lines = String::new();
    for e in &entries {
        let k = split_of(&e.word) as usize;
        let pred = decode(&net, &classes, e);
        res[k].0.add(&pred, &e.gold);
        let bp = base.get(&e.word).cloned().unwrap_or_default();
        res[k].1.add(&bp, &e.gold);
        if k == 2 { lines.push_str(&format!("{}\tor:{}\tmodèle:{}\tphon.vx:{}\n", e.word, e.gold, pred, bp)); }
    }
    for (k, nom) in ["apprentissage", "validation", "TEST"].iter().enumerate() {
        say!("{nom:14} modèle : {}", res[k].0.show());
        say!("{:14} phon.vx : {}", "", res[k].1.show());
    }
    let mut all = Score::default();
    for e in &entries { all.add(&base.get(&e.word).cloned().unwrap_or_default(), &e.gold); }
    say!("phon.vx sur TOUS les mots du lexique : {}", all.show());
    let verdict = if res[2].0.exact > res[2].1.exact && res[2].0.edits < res[2].1.edits { "LE MODÈLE BAT la référence sur le test (échantillon minuscule : à confirmer)" } else { "LE MODÈLE NE BAT PAS la référence sur le test : la référence est conservée" };
    say!("Verdict : {verdict}");
    fs::write(format!("{}/rapport.txt", a[3]), &report).unwrap();
    fs::write(format!("{}/test-detail.txt", a[3]), lines).unwrap();
    let dump = |v: &Vec<f32>| v.iter().map(|x| format!("{x:.6}")).collect::<Vec<_>>().join(" ");
    fs::write(format!("{}/modele-f32.txt", a[3]), format!("classes: {}\nH={H} W={W} V={V} graine={seed} epoque={best_ep}\nw1: {}\nb1: {}\nw2: {}\nb2: {}\n", classes.join(","), dump(&net.w1), dump(&net.b1), dump(&net.w2), dump(&net.b2))).unwrap();
}
