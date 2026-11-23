//! Aides UEFI pures (sans appel au firmware) pour lancer le gestionnaire de démarrage d'un autre système installé (ex. Windows) depuis Voxa Live.
//! Voxa ne copie, ne modifie et n'écrit rien sur le disque : il lit le chemin `\EFI\Microsoft\Boot\bootmgfw.efi` de la partition système EFI de la machine
//! et demande au firmware de le charger. Testé ici contre des chemins synthétiques ; l'appel réel au firmware reste NOT_VALIDATED.

/// Chemin standard du gestionnaire de démarrage de Windows sur la partition système EFI.
pub const WINDOWS_BOOT_PATH: &str = "\\EFI\\Microsoft\\Boot\\bootmgfw.efi";
/// Présent sur une clé d'installation de Windows et absent d'un Windows installé : distingue les deux.
pub const WINDOWS_SETUP_MARKER: &str = "\\sources\\boot.wim";
/// EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID {964E5B22-6459-11D2-8E39-00A0C969723B}
pub const GUID_SIMPLE_FILE_SYSTEM: [u8; 16] = [0x22, 0x5B, 0x4E, 0x96, 0x59, 0x64, 0xD2, 0x11, 0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B];
/// EFI_DEVICE_PATH_PROTOCOL_GUID {09576E91-6D3F-11D2-8E39-00A0C969723B}
pub const GUID_DEVICE_PATH: [u8; 16] = [0x91, 0x6E, 0x57, 0x09, 0x3F, 0x6D, 0xD2, 0x11, 0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B];

/// Longueur (fin comprise) d'un chemin de périphérique UEFI : parcourt les nœuds [type, sous-type, longueur u16] jusqu'au nœud de fin (0x7F, 0xFF).
/// None si un nœud est plus court que 4 octets ou dépasse le tampon.
pub fn device_path_len(dp: &[u8]) -> Option<usize> {
    let mut o = 0;
    loop {
        let h = dp.get(o..o + 4)?; let len = u16::from_le_bytes([h[2], h[3]]) as usize;
        if len < 4 || o + len > dp.len() { return None; }
        o += len;
        if h[0] == 0x7F && h[1] == 0xFF { return Some(o); }
    }
}

/// Construit « chemin du volume + nœud Média/Chemin de fichier + fin » dans `out`. `volume_dp` = chemin du périphérique du volume (fin comprise).
/// Renvoie la longueur écrite, ou None si le chemin du volume est invalide ou si `out` est trop petit.
pub fn file_device_path(volume_dp: &[u8], path: &str, out: &mut [u8]) -> Option<usize> {
    let full = device_path_len(volume_dp)?; let body = full - 4; // sans le nœud de fin d'origine
    let units = path.encode_utf16().count() + 1; let node = 4 + 2 * units;
    let total = body + node + 4;
    if node > 0xFFFF || out.len() < total { return None; }
    out[..body].copy_from_slice(&volume_dp[..body]);
    out[body] = 0x04; out[body + 1] = 0x04; out[body + 2..body + 4].copy_from_slice(&(node as u16).to_le_bytes());
    let mut o = body + 4; for u in path.encode_utf16() { out[o..o + 2].copy_from_slice(&u.to_le_bytes()); o += 2; }
    out[o] = 0; out[o + 1] = 0; o += 2;
    out[o..o + 4].copy_from_slice(&[0x7F, 0xFF, 0x04, 0x00]);
    Some(total)
}

/// Le chemin en UTF-16 terminé par 0, pour `EFI_FILE_PROTOCOL.Open`. None si `out` est trop petit.
pub fn path_utf16(path: &str, out: &mut [u16]) -> Option<usize> {
    let n = path.encode_utf16().count(); if out.len() < n + 1 { return None; }
    for (i, u) in path.encode_utf16().enumerate() { out[i] = u; } out[n] = 0; Some(n + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hd_node() -> std::vec::Vec<u8> { // un nœud « partition » (type 4, sous-type 1) de 42 octets
        let mut v = vec![0x04, 0x01, 42, 0]; v.extend(vec![0xAB; 38]); v
    }
    fn end() -> [u8; 4] { [0x7F, 0xFF, 4, 0] }
    #[test] fn device_path_length_walks_nodes_to_the_end_node() {
        let mut dp = hd_node(); dp.extend_from_slice(&end()); assert_eq!(device_path_len(&dp), Some(46));
        assert_eq!(device_path_len(&dp[..45]), None, "fin tronquée"); assert_eq!(device_path_len(&[0x04, 0x01, 2, 0]), None, "nœud trop court");
        assert_eq!(device_path_len(&[0x04, 0x01, 200, 0, 1, 2]), None, "nœud plus long que le tampon");
    }
    #[test] fn file_path_node_is_appended_before_a_new_end_node() {
        let mut dp = hd_node(); dp.extend_from_slice(&end()); let mut out = [0u8; 256];
        let n = file_device_path(&dp, "\\EFI\\X.efi", &mut out).unwrap();
        assert_eq!(&out[..42], &hd_node()[..], "les nœuds du volume sont conservés"); assert_eq!(&out[42..44], &[0x04, 0x04]);
        let node_len = u16::from_le_bytes([out[44], out[45]]) as usize; assert_eq!(node_len, 4 + 2 * (10 + 1));
        assert_eq!(u16::from_le_bytes([out[46], out[47]]), b'\\' as u16); assert_eq!(&out[42 + node_len - 2..42 + node_len], &[0, 0], "chaîne terminée par 0");
        assert_eq!(&out[n - 4..n], &end()); assert_eq!(device_path_len(&out[..n]), Some(n), "le résultat est lui-même un chemin valide");
    }
    #[test] fn bad_inputs_are_refused_without_writing_out_of_bounds() {
        let mut dp = hd_node(); dp.extend_from_slice(&end()); let mut small = [0u8; 50];
        assert_eq!(file_device_path(&dp, WINDOWS_BOOT_PATH, &mut small), None, "tampon trop petit"); assert_eq!(file_device_path(&[1, 2, 3], "x", &mut [0u8; 64]), None, "chemin de volume invalide");
    }
    #[test] fn windows_path_encodes_to_utf16_with_terminator() {
        let mut b = [0xFFFFu16; 64]; let n = path_utf16(WINDOWS_BOOT_PATH, &mut b).unwrap(); assert_eq!(n, WINDOWS_BOOT_PATH.len() + 1); assert_eq!(b[n - 1], 0); assert_eq!(b[0], b'\\' as u16);
        assert_eq!(path_utf16(WINDOWS_BOOT_PATH, &mut [0u16; 5]), None);
    }
}
