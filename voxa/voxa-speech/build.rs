// Génère les tables du synthétiseur à la compilation (sur l'hôte, avec std) : cosinus et exponentielle en virgule fixe, et les paramètres des phonèmes
// lus dans ../bib/voix-params-final.txt (source unique des paramètres, la même qui produit bib/voix-donnees.vx). Le code embarqué n'utilise aucun flottant.
use std::{env, fs, path::PathBuf};
fn main() {
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let params = dir.join("../bib/voix-params-final.txt");
    println!("cargo:rerun-if-changed={}", params.display());
    let fs_hz = 22050.0f64;
    let mut out = String::new();
    out.push_str("pub static COS: [i64; 552] = [");
    for k in 0..=551 { out.push_str(&format!("{},", ((2.0 * std::f64::consts::PI * (k * 10) as f64 / fs_hz).cos() * 4096.0).round() as i64)); }
    out.push_str("];\npub static EXP: [i64; 101] = [");
    for k in 0..=100 { out.push_str(&format!("{},", ((-std::f64::consts::PI * (k * 10) as f64 / fs_hz).exp() * 4096.0).round() as i64)); }
    out.push_str("];\n");
    // valeurs par défaut identiques à voixlib::Params::defaut()
    let mut g = std::collections::BTreeMap::new();
    for (k, v) in [("dur_pct", 130), ("f0", 120), ("rad", 3), ("tilt", 3), ("bw_pct", 100), ("nz_pct", 100), ("bu_pct", 100), ("gain", 2), ("acc_pct", 170), ("decl", 30), ("f0_acc", 20), ("ferm_p", 55), ("ferm_b", 50), ("asp_ms", 22), ("burst_ms", 10), ("plat", 20), ("pause_ms", 45)] { g.insert(k.to_string(), v as i64); }
    let txt = fs::read_to_string(&params).expect("voix-params-final.txt illisible");
    let mut phones = Vec::new();
    for l in txt.lines() {
        if l.starts_with('#') || l.trim().is_empty() { continue; }
        if let Some(r) = l.strip_prefix("global ") { for kv in r.split_whitespace() { let (k, v) = kv.split_once('=').unwrap(); g.insert(k.to_string(), v.parse().unwrap()); } continue; }
        let m: Vec<&str> = l.split_whitespace().collect();
        phones.push(format!("Phone {{ sym: b'{}', ty: b'{}', dur: {}, f: [{}, {}, {}], amp: {}, bf: {}, bw: {} }}", m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8]));
    }
    out.push_str(&format!("pub static PHONES: [Phone; {}] = [{}];\n", phones.len(), phones.join(",")));
    out.push_str(&format!("pub static GLOBALS: Globals = Globals {{ {} }};\n", g.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(", ")));
    fs::write(PathBuf::from(env::var("OUT_DIR").unwrap()).join("tables.rs"), out).unwrap();
}
