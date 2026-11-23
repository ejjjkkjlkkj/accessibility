// Affichage a l'ecran pendant toute l'analyse (GOP, texte 5x7 en blanc sur noir).
// Chaque etape est affichee avant de commencer : si le PC se bloque, la derniere
// ligne visible indique ou. Aucune ecriture materielle ici.
use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams};
use uefi::proto::console::gop::GraphicsOutput;

pub struct Screen {
    fb: usize,
    len: usize,
    stride: usize,
    width: usize,
    height: usize,
    scale: usize,
    col: usize,
    row: usize,
    cols_max: usize,
    rows_max: usize,
    line_h: usize,
}

/// Ouvre le framebuffer GOP (avant ExitBootServices). None si aucun ecran GOP.
pub fn open() -> Option<Screen> {
    let hgop = boot::get_handle_for_protocol::<GraphicsOutput>().ok()?;
    // SAFETY: GetProtocol, lecture seule du protocole ; seul le framebuffer est utilise ensuite.
    let mut g = unsafe {
        boot::open_protocol::<GraphicsOutput>(
            OpenProtocolParams { handle: hgop, agent: boot::image_handle(), controller: None },
            OpenProtocolAttributes::GetProtocol,
        )
    }
    .ok()?;
    let info = g.current_mode_info();
    let (w, h) = info.resolution();
    let stride = info.stride();
    let mut fb = g.frame_buffer();
    let ptr = fb.as_mut_ptr() as usize;
    let len = fb.size() / 4;
    if ptr == 0 || len == 0 || stride == 0 || w == 0 || h == 0 {
        return None;
    }
    // Pas d'effacement a l'ouverture : chaque caractere efface son propre fond, donc pas de flash noir.
    Some(Screen::new(ptr, len, stride, w, h))
}

impl Screen {
    fn new(fb: usize, len: usize, stride: usize, width: usize, height: usize) -> Screen {
        let scale = if width >= 1280 { 3 } else { 2 };
        let cell_w = 6 * scale;
        let line_h = 8 * scale + 2 * scale;
        let margin = 8;
        Screen {
            fb,
            len,
            stride,
            width,
            height,
            scale,
            col: 0,
            row: 0,
            cols_max: (width - 2 * margin) / cell_w,
            rows_max: (height - 2 * margin) / line_h,
            line_h,
        }
    }

    /// Pour l'affichage des exceptions : (framebuffer, longueur, stride, largeur, hauteur).
    pub fn info(&self) -> (usize, usize, usize, usize, usize) {
        (self.fb, self.len, self.stride, self.width, self.height)
    }


    fn newline(&mut self) {
        self.col = 0;
        self.row += 1;
        if self.row >= self.rows_max {
            self.scroll();
        }
    }

    /// Remonte l'affichage d'une ligne (sans effacer tout l'ecran : pas de flash noir).
    fn scroll(&mut self) {
        let total = (self.height * self.stride).min(self.len);
        let shift = (self.line_h * self.stride).min(total);
        // SAFETY: framebuffer GOP, bornes limitees a la taille verifiee a l'ouverture.
        let fb = unsafe { core::slice::from_raw_parts_mut(self.fb as *mut u32, self.len) };
        fb.copy_within(shift..total, 0);
        for px in fb[total - shift..total].iter_mut() {
            *px = 0;
        }
        self.row = self.rows_max - 1;
        self.col = 0;
    }

    /// Ecrit du texte ; '\n' passe a la ligne suivante, les lignes trop longues sont coupees.
    pub fn put(&mut self, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.newline();
                continue;
            }
            if self.col >= self.cols_max {
                self.newline();
            }
            let ch = if c.is_ascii() { c as u8 } else { b'?' };
            self.draw_char(ch);
            self.col += 1;
        }
    }

    fn draw_char(&mut self, ch: u8) {
        let s = self.scale;
        let x0 = 8 + self.col * 6 * s;
        let y0 = 8 + self.row * self.line_h;
        // SAFETY: framebuffer GOP ; chaque pixel est verifie contre la taille avant ecriture.
        let fb = unsafe { core::slice::from_raw_parts_mut(self.fb as *mut u32, self.len) };
        let bits = font(ch);
        for (r, rowbits) in bits.iter().enumerate() {
            for c in 0..5usize {
                let on = rowbits & (0x10 >> c) != 0;
                for dy in 0..s {
                    for dx in 0..s {
                        let x = x0 + c * s + dx;
                        let y = y0 + r * s + dy;
                        if x < self.width && y < self.height {
                            let idx = y * self.stride + x;
                            if idx < fb.len() {
                                fb[idx] = if on { 0x00FF_FFFF } else { 0 };
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Police 5x7 : chaque octet = une ligne, bits 4..0 = colonnes de gauche a droite.
/// Minuscules converties en majuscules. Caracteres inconnus = vide.
pub fn font(c: u8) -> [u8; 7] {
    let c = c.to_ascii_uppercase();
    match c {
        b' ' => [0; 7],
        b'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        b'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        b'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        b'D' => [0x1C, 0x12, 0x11, 0x11, 0x11, 0x12, 0x1C],
        b'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        b'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        b'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F],
        b'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        b'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        b'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
        b'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        b'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        b'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        b'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        b'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        b'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        b'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        b'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        b'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        b'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        b'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        b'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        b'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
        b'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        b'Y' => [0x11, 0x11, 0x11, 0x0A, 0x04, 0x04, 0x04],
        b'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        b'0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        b'1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        b'2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        b'3' => [0x1F, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0E],
        b'4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        b'5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        b'6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        b'7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        b'8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        b'9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        b'.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        b',' => [0x00, 0x00, 0x00, 0x00, 0x0C, 0x04, 0x08],
        b':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
        b'-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        b'_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F],
        b'/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        b'\\' => [0x10, 0x10, 0x08, 0x04, 0x02, 0x01, 0x01],
        b'(' => [0x02, 0x04, 0x08, 0x08, 0x08, 0x04, 0x02],
        b')' => [0x08, 0x04, 0x02, 0x02, 0x02, 0x04, 0x08],
        b'[' => [0x0E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x0E],
        b']' => [0x0E, 0x02, 0x02, 0x02, 0x02, 0x02, 0x0E],
        b'=' => [0x00, 0x00, 0x1F, 0x00, 0x1F, 0x00, 0x00],
        b'+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        b'#' => [0x0A, 0x0A, 0x1F, 0x0A, 0x1F, 0x0A, 0x0A],
        b'%' => [0x18, 0x19, 0x02, 0x04, 0x08, 0x13, 0x03],
        b'?' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04],
        b'!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        b'\'' => [0x0C, 0x0C, 0x08, 0x00, 0x00, 0x00, 0x00],
        b'<' => [0x02, 0x04, 0x08, 0x10, 0x08, 0x04, 0x02],
        b'>' => [0x08, 0x04, 0x02, 0x01, 0x02, 0x04, 0x08],
        _ => [0; 7],
    }
}
