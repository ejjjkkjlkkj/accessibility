//! Decompresseur maison du format UEFI "standard" (EFI 1.1) et "Tiano" : LZ77 + codes de Huffman canoniques, en blocs.
//! Ecrit a partir de la description du format (contrat d'interoperabilite), sans reprendre de code tiers.
//! Flux : 8 octets d'en-tete (taille compressee u32, taille d'origine u32) puis bits lus du plus fort au plus faible.
//! Les deux variantes ne different que par la largeur du champ de la table des positions : 4 bits (EFI) ou 5 bits (Tiano).

struct Bits<'a> { d: &'a [u8], pos: usize }
impl Bits<'_> {
    fn peek(&self, n: usize) -> u32 { let mut v = 0u32; for i in 0..n { let p = self.pos + i; let b = self.d.get(p / 8).map_or(0, |x| (x >> (7 - p % 8)) & 1); v = v << 1 | b as u32; } v }
    fn get(&mut self, n: usize) -> u32 { let v = self.peek(n); self.pos += n; v }
    fn fini(&self) -> bool { self.pos > self.d.len() * 8 + 32 }
}

/// Table de Huffman canonique (codes plus courts = valeurs numeriques plus petites, puis ordre des symboles) ou symbole unique de longueur de code nulle.
enum Table { Unique(u16), Code { nb: [u16; 17], symboles: Vec<u16> } }
fn construire(longueurs: &[u8]) -> Result<Table, String> {
    let mut nb = [0u16; 17]; for &l in longueurs { if l > 16 { return Err("longueur de code > 16".into()); } nb[l as usize] += 1; } nb[0] = 0;
    let mut symboles: Vec<u16> = Vec::new(); for l in 1..=16u8 { for (s, &sl) in longueurs.iter().enumerate() { if sl == l { symboles.push(s as u16); } } }
    Ok(Table::Code { nb, symboles })
}
fn decoder(b: &mut Bits, t: &Table) -> Result<u16, String> {
    match t { Table::Unique(s) => Ok(*s),
        Table::Code { nb, symboles } => { let (mut code, mut premier, mut index) = (0i32, 0i32, 0i32);
            for l in 1..=16usize { code |= b.get(1) as i32; let n = nb[l] as i32; if code - premier < n { return symboles.get((index + code - premier) as usize).copied().ok_or_else(|| "symbole hors table".to_string()); }
                index += n; premier = (premier + n) << 1; code <<= 1; }
            Err("code de Huffman invalide".into()) } }
}
/// Lecture des longueurs de la table des "extras" (nn symboles, champ de nbit bits) ou de la table des positions.
fn lire_pt(b: &mut Bits, nn: usize, nbit: usize, special: Option<usize>) -> Result<Table, String> {
    let nombre = b.get(nbit) as usize;
    if nombre == 0 { return Ok(Table::Unique(b.get(nbit) as u16)); }
    let mut len = vec![0u8; nn]; let mut i = 0;
    while i < nombre.min(nn) {
        let mut c = b.peek(3); if c == 7 { let mut uns = 0usize; while uns < 13 && b.peek(4 + uns) & 1 == 1 { uns += 1; } c = 7 + uns as u32; }
        b.get(if c < 7 { 3 } else { c as usize - 3 }); len[i] = c as u8; i += 1;
        if Some(i) == special { let z = b.get(2) as usize; for _ in 0..z { if i < nn { len[i] = 0; i += 1; } } }
    }
    construire(&len)
}
const NC: usize = 510; const NT: usize = 19; const MAXNP: usize = 31;
fn lire_c(b: &mut Bits, pt: &Table) -> Result<Table, String> {
    let nombre = b.get(9) as usize; if nombre == 0 { return Ok(Table::Unique(b.get(9) as u16)); }
    let mut len = vec![0u8; NC]; let mut i = 0;
    while i < nombre.min(NC) {
        let c = decoder(b, pt)? as usize;
        if c <= 2 { let n = match c { 0 => 1, 1 => b.get(4) as usize + 3, _ => b.get(9) as usize + 20 }; for _ in 0..n { if i < NC { len[i] = 0; i += 1; } } }
        else { len[i] = (c - 2) as u8; i += 1; }
        if b.fini() { return Err("flux epuise".into()); }
    }
    construire(&len)
}

/// `src` = en-tete de 8 octets + flux ; `pbit` = 4 (EFI) ou 5 (Tiano). Retourne les octets d'origine.
pub fn decode(src: &[u8], pbit: usize) -> Result<Vec<u8>, String> {
    if src.len() < 8 { return Err("en-tete trop court".into()); }
    let orig = u32::from_le_bytes(src[4..8].try_into().unwrap()) as usize; if orig > 1 << 28 { return Err("taille d'origine absurde".into()); }
    let mut b = Bits { d: &src[8..], pos: 0 }; let mut out: Vec<u8> = Vec::with_capacity(orig);
    let (mut restant, mut tc, mut tp): (usize, Option<Table>, Option<Table>) = (0, None, None);
    while out.len() < orig {
        if b.fini() { return Err(format!("flux epuise a {} / {} octets", out.len(), orig)); }
        if restant == 0 {
            restant = b.get(16) as usize; if restant == 0 { return Err("bloc vide".into()); }
            let pt = lire_pt(&mut b, NT, 5, Some(3))?; tc = Some(lire_c(&mut b, &pt)?); tp = Some(lire_pt(&mut b, MAXNP, pbit, None)?);
        }
        restant -= 1; let c = decoder(&mut b, tc.as_ref().unwrap())? as usize;
        if c < 256 { out.push(c as u8); continue; }
        let longueur = c - 253; let v = decoder(&mut b, tp.as_ref().unwrap())? as usize;
        let pos = if v <= 1 { v } else { (1usize << (v - 1)) + b.get(v - 1) as usize };
        if pos + 1 > out.len() { return Err(format!("reference arriere invalide ({} > {})", pos + 1, out.len())); }
        let debut = out.len() - pos - 1; for k in 0..longueur { if out.len() >= orig { break; } let x = out[debut + k]; out.push(x); }
    }
    Ok(out)
}
/// Essaie les deux variantes et garde celle qui reconstitue exactement la taille annoncee.
pub fn decode_auto(src: &[u8]) -> Result<Vec<u8>, String> {
    match decode(src, 4) { Ok(v) => Ok(v), Err(e1) => decode(src, 5).map_err(|e2| format!("EFI: {e1} ; Tiano: {e2}")) }
}
