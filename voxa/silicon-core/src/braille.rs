//! Braille : transcription du texte français en cellules 8 points (intégral, sans abréviations) et constantes de la page d'usage HID « Braille Display ».
//! Cellule = masque de bits : point 1 = bit 0 … point 8 = bit 7 ; le caractère Unicode braille correspondant est U+2800 + masque.
//! STATUT : table établie à partir des points usuels du braille français ; **à faire valider par un transcripteur braille**. Aucun afficheur braille n'est
//! disponible : la transmission au matériel (pilote HID) n'est pas écrite ; seuls la transcription et le défilement sont testés.
//! Les valeurs de la page d'usage HID 0x41 sont celles du standard public USB-IF (HUTRR78), relevées dans les en-têtes ouverts de Darwin (IOHIDUsageTables.h).

pub const HID_PAGE_BRAILLE_DISPLAY: u16 = 0x41;
pub mod usage {
    pub const BRAILLE_DISPLAY: u16 = 0x01; pub const BRAILLE_ROW: u16 = 0x02; pub const CELL_8_DOTS: u16 = 0x03; pub const CELL_6_DOTS: u16 = 0x04;
    pub const NUMBER_OF_CELLS: u16 = 0x05; pub const SCREEN_READER_CONTROL: u16 = 0x06; pub const SCREEN_READER_IDENTIFIER: u16 = 0x07;
    pub const ROUTER_KEY: u16 = 0x100; pub const ROW_ROUTER_KEY: u16 = 0x101; pub const BRAILLE_BUTTONS: u16 = 0x200;
    pub const KEYBOARD_DOT1: u16 = 0x201; pub const KEYBOARD_DOT8: u16 = 0x208; pub const KEYBOARD_SPACE: u16 = 0x209;
    pub const PAN_LEFT: u16 = 0x21A; pub const PAN_RIGHT: u16 = 0x21B;
}

const NUMBER_SIGN: u8 = 0x3C; // points 3456
const CAPITAL_SIGN: u8 = 0x20; // point 6

/// Lettres a à z (w à la française : points 2456). Index 0 = a.
const LETTERS: [u8; 26] = [
    0x01, 0x03, 0x09, 0x19, 0x11, 0x0B, 0x1B, 0x13, 0x0A, 0x1A, // a b c d e f g h i j
    0x05, 0x07, 0x0D, 0x1D, 0x15, 0x0F, 0x1F, 0x17, 0x0E, 0x1E, // k l m n o p q r s t
    0x25, 0x27, 0x3A, 0x2D, 0x3D, 0x35,                          // u v w x y z
];

fn accent(c: char) -> Option<u8> {
    Some(match c { 'é' => 0x3F, 'à' => 0x37, 'è' => 0x2E, 'ù' => 0x3E, 'â' => 0x21, 'ê' => 0x23, 'î' => 0x29, 'ô' => 0x39, 'û' => 0x31, 'ç' => 0x2F, 'ë' => 0x2B, 'ï' => 0x3B, 'œ' => 0x2A, 'ü' => 0x33, _ => return None })
}
fn punct(c: char) -> Option<u8> {
    Some(match c { ',' => 0x02, ';' => 0x06, ':' => 0x12, '.' => 0x32, '?' => 0x22, '!' => 0x16, '\'' | '’' => 0x04, '-' => 0x24, ' ' => 0x00, _ => return None })
}
fn digit(c: char) -> Option<u8> { match c { '1'..='9' => Some(LETTERS[c as usize - '1' as usize]), '0' => Some(LETTERS[9]), _ => None } }

/// Résultat d'une transcription : nombre de cellules écrites, caractères non pris en charge (rendus par une cellule vide), et débordement du tampon.
pub struct Transcription { pub cells: usize, pub unsupported: usize, pub truncated: bool }

/// Transcrit `text` en cellules dans `out`. Majuscule : point 6 ; suite de 2 majuscules ou plus : point 6 doublé ; chiffres : signe numérique (3456) une fois par nombre.
pub fn to_cells(text: &str, out: &mut [u8]) -> Transcription {
    let mut r = Transcription { cells: 0, unsupported: 0, truncated: false };
    let mut push = |v: u8, r: &mut Transcription| { if r.cells < out.len() { out[r.cells] = v; r.cells += 1; } else { r.truncated = true; } };
    let chars: [char; 0] = [];
    let _ = chars;
    let mut in_number = false;
    let mut it = text.chars().peekable();
    let mut prev_upper = false;
    while let Some(c) = it.next() {
        if let Some(d) = digit(c) { if !in_number { push(NUMBER_SIGN, &mut r); in_number = true; } push(d, &mut r); prev_upper = false; continue; }
        in_number = false;
        let lower: char = c.to_lowercase().next().unwrap_or(c);
        let upper = c != lower;
        if upper {
            if !prev_upper { push(CAPITAL_SIGN, &mut r); if it.peek().map_or(false, |n| n.is_uppercase()) { push(CAPITAL_SIGN, &mut r); } }
        }
        prev_upper = upper;
        if lower.is_ascii_lowercase() { push(LETTERS[lower as usize - 'a' as usize], &mut r); }
        else if let Some(a) = accent(lower) { push(a, &mut r); }
        else if let Some(p) = punct(lower) { push(p, &mut r); prev_upper = false; }
        else { push(0x00, &mut r); r.unsupported += 1; prev_upper = false; }
    }
    r
}

/// Fenêtre d'affichage de `width` cellules à partir de `pos` (défilement vers l'avant/arrière par les touches de panoramique).
pub fn window(cells: &[u8], pos: usize, width: usize) -> &[u8] { let s = pos.min(cells.len()); let e = (s + width).min(cells.len()); &cells[s..e] }
/// Caractère Unicode braille d'une cellule (pour affichage de contrôle et tests).
pub fn to_unicode(cell: u8) -> char { char::from_u32(0x2800 + cell as u32).unwrap_or('\u{2800}') }

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    fn s(t: &str) -> std::string::String { let mut b = [0u8; 128]; let r = to_cells(t, &mut b); b[..r.cells].iter().map(|c| to_unicode(*c)).collect() }
    #[test] fn lettres_de_base() { assert_eq!(s("abc"), "⠁⠃⠉"); assert_eq!(s("jouer"), "⠚⠕⠥⠑⠗"); assert_eq!(s("w"), "⠺"); assert_eq!(s("xyz"), "⠭⠽⠵"); }
    #[test] fn accents_francais() { assert_eq!(s("é"), "⠿"); assert_eq!(s("àèùçêôûîëï"), "⠷⠮⠾⠯⠣⠹⠱⠩⠫⠻"); assert_eq!(s("œ"), "⠪"); }
    #[test] fn majuscules() { assert_eq!(s("Bonjour"), "⠠⠃⠕⠝⠚⠕⠥⠗"); assert_eq!(s("ABC"), "⠠⠠⠁⠃⠉", "suite de majuscules : point 6 doublé une fois"); }
    #[test] fn nombres() { assert_eq!(s("12"), "⠼⠁⠃"); assert_eq!(s("a1"), "⠁⠼⠁"); assert_eq!(s("3200"), "⠼⠉⠃⠚⠚"); assert_eq!(s("1 2"), "⠼⠁⠀⠼⠃", "le signe numérique est répété après une espace"); }
    #[test] fn ponctuation() { assert_eq!(s("a, b."), "⠁⠂⠀⠃⠲"); assert_eq!(s("l'été"), "⠇⠄⠿⠞⠿"); }
    #[test] fn non_pris_en_charge_est_signale() {
        let mut b = [0u8; 16]; let r = to_cells("a(b)", &mut b); assert_eq!(r.unsupported, 2, "les parenthèses ne sont pas encore validées : signalées, jamais inventées"); assert_eq!(r.cells, 4);
    }
    #[test] fn tampon_trop_petit_est_signale() { let mut b = [0u8; 3]; let r = to_cells("abcdef", &mut b); assert!(r.truncated); assert_eq!(r.cells, 3); }
    #[test] fn defilement_par_fenetres() { let c = [1u8, 2, 3, 4, 5]; assert_eq!(window(&c, 0, 2), &[1, 2]); assert_eq!(window(&c, 3, 4), &[4, 5]); assert!(window(&c, 9, 4).is_empty()); }
    #[test] fn constantes_hid_standard() { assert_eq!(HID_PAGE_BRAILLE_DISPLAY, 0x41); assert_eq!(usage::PAN_RIGHT, 0x21B); assert_eq!(usage::KEYBOARD_DOT8 - usage::KEYBOARD_DOT1, 7); }
}
