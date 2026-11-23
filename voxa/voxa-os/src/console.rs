//! Console du noyau : journal serie (COM1) + texte sur le cadre graphique du firmware (police maison 5x7 agrandie). C'est la "voix" provisoire (couche 0) :
//! tant que le pilote audio n'existe pas, `say` ecrit ; le jalon M2 la remplacera par la parole sans changer les appelants.
use crate::cpu::{inb, outb};
use core::fmt::Write;
use voxa_kernel::Voice;

const POLICE: &str = include_str!("../../bib/police-5x7.txt");
static mut GLYPHES: [[u8; 7]; 95] = [[0; 7]; 95];
/// Rangees (7 x 5 bits) du glyphe ASCII `code` (32 a 126) ; un carre pour tout autre code.
pub fn glyphe_bits(code: u32) -> [u8; 7] { if (32..127).contains(&code) { unsafe { GLYPHES[(code - 32) as usize] } } else { [0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F] } }

pub fn serial_init() {
    outb(0x3F9, 0); outb(0x3FB, 0x80); outb(0x3F8, 1); outb(0x3F9, 0); outb(0x3FB, 0x03); outb(0x3FA, 0xC7); outb(0x3FC, 0x0B);
}
pub fn serial_byte(b: u8) { for _ in 0..10_000 { if inb(0x3FD) & 0x20 != 0 { break; } } outb(0x3F8, b); }

/// Cadre graphique du firmware (reste valide apres ExitBootServices).
#[derive(Clone, Copy)]
pub struct Fb { pub base: u64, pub w: u32, pub h: u32, pub stride: u32, pub bgr: bool }

/// Journal du noyau (anneau de texte ASCII) : consultable a l'ecran par le bureau graphique (touche Tab), meme quand le journal ne s'ecrit plus directement sur l'ecran.
static mut JOURNAL: [u8; 6000] = [0; 6000];
static mut JLEN: usize = 0;
/// Vrai tant que le journal s'ecrit directement sur l'ecran (avant le bureau graphique).
pub static mut ECRAN_ACTIF: bool = true;
pub fn journal() -> &'static [u8] { unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(JOURNAL) as *const u8, JLEN) } }
fn journal_ajoute(b: u8) { unsafe { if JLEN == JOURNAL.len() { JOURNAL.copy_within(1000.., 0); JLEN -= 1000; } JOURNAL[JLEN] = b; JLEN += 1; } }
pub fn fb() -> Option<Fb> { unsafe { CONSOLE.fb } }

pub struct Console { fb: Option<Fb>, col: u32, row: u32, scale: u32 }
pub static mut CONSOLE: Console = Console { fb: None, col: 0, row: 0, scale: 2 };

fn charger_police() {
    let mut i = 0;
    for l in POLICE.lines() { if l.starts_with('#') || l.trim().is_empty() || i >= 95 { continue; }
        let mut r = 0; for t in l.split_whitespace() { if r < 7 { unsafe { GLYPHES[i][r] = u8::from_str_radix(t, 16).unwrap_or(0); } r += 1; } } i += 1; }
}
pub fn init(fb: Option<Fb>) { charger_police(); unsafe { CONSOLE.fb = fb; CONSOLE.col = 0; CONSOLE.row = 0; if let Some(f) = fb { CONSOLE.scale = if f.w >= 1600 { 3 } else { 2 }; fond(&f); } } }

pub fn pixel(f: &Fb, x: u32, y: u32, rgb: u32) {
    if x >= f.w || y >= f.h { return; }
    let v = if f.bgr { rgb } else { (rgb & 0xFF) << 16 | (rgb & 0xFF00) | (rgb >> 16) & 0xFF };
    unsafe { core::ptr::write_volatile((f.base + (y as u64 * f.stride as u64 + x as u64) * 4) as *mut u32, v); }
}
fn fond(f: &Fb) { for y in 0..f.h { for x in 0..f.w { pixel(f, x, y, 0x0010_1030); } } }
fn glyphe(f: &Fb, c: char, cx: u32, cy: u32, s: u32, fg: u32) {
    let code = c as u32; let g = if (32..127).contains(&code) { unsafe { GLYPHES[(code - 32) as usize] } } else { [0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F] };
    for (r, bits) in g.iter().enumerate() { for b in 0..5 { let on = bits & (0x10 >> b) != 0; let rgb = if on { fg } else { 0x0010_1030 }; for dy in 0..s { for dx in 0..s { pixel(f, cx + (b * s + dx), cy + (r as u32 * s + dy), rgb); } } } }
}
/// Ecrit un caractere sur l'ecran (retour a la ligne, defilement par effacement de page).
fn ecran_char(c: char) {
    unsafe {
        if (c as u32) < 128 { journal_ajoute(c as u8); }
        if !ECRAN_ACTIF { return; }
        let Some(f) = CONSOLE.fb else { return }; let s = CONSOLE.scale; let (cw, ch) = (6 * s, 9 * s); let cols = f.w / cw; let rows = f.h / ch;
        if c == '\n' { CONSOLE.col = 0; CONSOLE.row += 1; } else { glyphe(&f, c, 8 + CONSOLE.col * cw, 8 + CONSOLE.row * ch, s, 0x00F0F0F0); CONSOLE.col += 1; if CONSOLE.col >= cols - 2 { CONSOLE.col = 0; CONSOLE.row += 1; } }
        if CONSOLE.row >= rows - 2 { fond(&f); CONSOLE.row = 0; CONSOLE.col = 0; }
    }
}
fn ascii_pour_ecran(c: char) -> char { match c { 'à' | 'â' | 'ä' => 'a', 'é' | 'è' | 'ê' | 'ë' => 'e', 'î' | 'ï' => 'i', 'ô' | 'ö' => 'o', 'ù' | 'û' | 'ü' => 'u', 'ç' => 'c', 'À' => 'A', 'É' | 'È' | 'Ê' => 'E', 'Ç' => 'C', '’' => '\'', '«' | '»' => '"', c => c } }

pub struct Sortie;
impl Write for Sortie {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() { let a = ascii_pour_ecran(c); if a == '\n' { serial_byte(b'\r'); serial_byte(b'\n'); } else if (a as u32) < 128 { serial_byte(a as u8); } else { serial_byte(b'?'); } ecran_char(a); }
        Ok(())
    }
}
#[macro_export] macro_rules! log { ($($arg:tt)*) => {{ use core::fmt::Write as _; let _ = writeln!($crate::console::Sortie, $($arg)*); }} }

/// Voix provisoire : ecrit ce qui sera dit.
pub struct Parole;
impl Voice for Parole { fn say(&mut self, text: &str) { let _ = Sortie.write_str(text); let _ = Sortie.write_str("\n"); } }
