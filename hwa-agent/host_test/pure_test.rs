// Test hote des fonctions pures de usb_hid.rs (copie fidele de la logique, sans acces materiel).
// Compiler : rustc --edition 2021 --test pure_test.rs -o pure_test.exe && pure_test.exe
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kbd {
    pub cfg_value: u8,
    pub iface: u8,
    pub ep_addr: u8,
    pub ep_mps: u16,
    pub ep_interval: u8,
    pub dci: u8,
}

pub fn find_keyboard(cfg: &[u8]) -> Option<Kbd> {
    let total = (cfg[2] as usize) | ((cfg[3] as usize) << 8);
    let total = total.min(cfg.len());
    let cfg_value = cfg[5];
    let mut p = 0usize;
    let mut in_kbd = false;
    let mut iface = 0u8;
    while p + 2 <= total {
        let len = cfg[p] as usize;
        let ty = cfg[p + 1];
        if len < 2 || p + len > total {
            break;
        }
        if ty == 4 && len >= 9 {
            iface = cfg[p + 2];
            in_kbd = cfg[p + 5] == 3 && cfg[p + 6] == 1 && cfg[p + 7] == 1;
        } else if ty == 5 && len >= 7 && in_kbd {
            let addr = cfg[p + 2];
            let attr = cfg[p + 3];
            if addr & 0x80 != 0 && attr & 3 == 3 {
                let mps = (cfg[p + 4] as u16) | ((cfg[p + 5] as u16) << 8);
                let dci = (addr & 0xF) * 2 + 1;
                return Some(Kbd { cfg_value, iface, ep_addr: addr, ep_mps: mps & 0x7FF, ep_interval: cfg[p + 6], dci });
            }
        }
        p += len;
    }
    None
}

fn interrupt_interval(speed: u32, b_interval: u8) -> u8 {
    match speed {
        3 | 4 => b_interval.saturating_sub(1).min(15),
        _ => {
            let mut v = (b_interval as u32).max(1) * 8;
            let mut n = 0u8;
            while v > 1 {
                v >>= 1;
                n += 1;
            }
            n.min(15)
        }
    }
}

pub fn key_char(modifier: u8, code: u8) -> Option<u8> {
    let shift = modifier & 0x22 != 0;
    match code {
        0x04..=0x1D => {
            let c = b'a' + (code - 0x04);
            Some(if shift { c.to_ascii_uppercase() } else { c })
        }
        0x1E..=0x26 => {
            let base = b"123456789"[(code - 0x1E) as usize];
            Some(if shift { b"!@#$%^&*("[(code - 0x1E) as usize] } else { base })
        }
        0x27 => Some(if shift { b')' } else { b'0' }),
        0x2C => Some(b' '),
        0x2D => Some(if shift { b'_' } else { b'-' }),
        0x2E => Some(if shift { b'+' } else { b'=' }),
        0x36 => Some(if shift { b'<' } else { b',' }),
        0x37 => Some(if shift { b'>' } else { b'.' }),
        0x38 => Some(if shift { b'?' } else { b'/' }),
        0x28 => Some(b'\n'),
        0x2A => Some(0x08),
        _ => None,
    }
}

// Descripteur de configuration typique : config (9) + interface HID clavier (9) + HID (9) + endpoint IN 0x81 (7)
fn sample_cfg() -> Vec<u8> {
    let mut v = vec![
        9, 2, 0, 0, 1, 1, 0, 0xA0, 50, // configuration, wTotalLength rempli plus bas
        9, 4, 0, 0, 1, 3, 1, 1, 0, // interface 0 : HID, boot, clavier
        9, 0x21, 0x11, 0x01, 0, 1, 0x22, 63, 0, // HID
        7, 5, 0x81, 3, 8, 0, 10, // endpoint 1 IN, interruption, 8 octets, bInterval 10
    ];
    let n = v.len() as u16;
    v[2] = (n & 0xFF) as u8;
    v[3] = (n >> 8) as u8;
    v
}

#[test]
fn trouve_le_clavier() {
    let k = find_keyboard(&sample_cfg()).expect("clavier");
    assert_eq!(k.cfg_value, 1);
    assert_eq!(k.iface, 0);
    assert_eq!(k.ep_addr, 0x81);
    assert_eq!(k.ep_mps, 8);
    assert_eq!(k.ep_interval, 10);
    assert_eq!(k.dci, 3); // endpoint 1 IN -> DCI = 1*2+1 = 3
}

#[test]
fn ignore_une_souris() {
    let mut v = sample_cfg();
    v[13] = 2; // sous-classe 2 au lieu de 1 : pas de boot
    // recalcul : l'interface est a l'offset 9 ; sous-classe a 9+6=15
    v[15] = 0;
    assert!(find_keyboard(&v).is_none());
}

#[test]
fn intervalle_pleine_vitesse() {
    // bInterval 10 ms -> 80 microtrames -> 2^6 = 64 -> exposant 6
    assert_eq!(interrupt_interval(1, 10), 6);
    assert_eq!(interrupt_interval(2, 1), 3); // 8 microtrames -> 2^3
}

#[test]
fn intervalle_haute_vitesse() {
    assert_eq!(interrupt_interval(3, 4), 3); // bInterval 4 -> exposant 3
}

#[test]
fn touches_de_base() {
    assert_eq!(key_char(0, 0x04), Some(b'a'));
    assert_eq!(key_char(0x02, 0x04), Some(b'A')); // Shift gauche
    assert_eq!(key_char(0x20, 0x04), Some(b'A')); // Shift droit
    assert_eq!(key_char(0, 0x1E), Some(b'1'));
    assert_eq!(key_char(0x02, 0x1E), Some(b'!'));
    assert_eq!(key_char(0, 0x27), Some(b'0'));
    assert_eq!(key_char(0, 0x2C), Some(b' '));
    assert_eq!(key_char(0, 0x28), Some(b'\n'));
    assert_eq!(key_char(0, 0x2A), Some(0x08));
    assert_eq!(key_char(0, 0x3B), None); // F2 : non imprimable
}
