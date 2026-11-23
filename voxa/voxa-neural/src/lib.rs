//! VOXA Neural : moteur d'inférence entier.
//!
//! Ce que c'est : des opérations numériques déterministes (int8 × int8 → int32, requantification en virgule fixe, activations,
//! argmax) avec vérification explicite des dimensions et des débordements. `#![no_std]`, aucune allocation, aucune dépendance :
//! l'appelant fournit tous les tampons, donc l'empreinte mémoire est connue à l'avance.
//!
//! Ce que ce n'est PAS : un modèle. Aucun poids entraîné n'existe dans ce dépôt. Créer un modèle exige des données, un entraînement
//! et une évaluation qui n'ont pas été faits. Les tests ci-dessous prouvent la conformité numérique du moteur, pas l'utilité d'un modèle.
//! Statut : testé sur l'hôte de développement ; jamais exécuté sur la machine cible.
#![cfg_attr(not(test), no_std)]

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    /// Dimensions incompatibles (poids, entrée, sortie, biais).
    Shape,
    /// Facteur d'échelle invalide (nul ou trop grand pour la virgule fixe).
    Scale,
    /// L'accumulateur int32 aurait débordé : la couche est trop large pour ce format.
    Overflow,
}

/// Facteur d'échelle en virgule fixe Q16 : valeur réelle = entier × (m / 65536). m doit être > 0.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Q16(pub u32);

impl Q16 {
    pub const ONE: Q16 = Q16(65536);
    pub fn from_f32(x: f32) -> Result<Q16, Error> {
        if !(x > 0.0) || x >= 32768.0 { return Err(Error::Scale); }
        let m = (x * 65536.0 + 0.5) as u32;
        if m == 0 { Err(Error::Scale) } else { Ok(Q16(m)) }
    }
}

/// Multiplie un entier 32 bits par un facteur Q16 avec arrondi au plus proche (moitiés vers +∞), puis sature en i8.
pub fn requantize(acc: i32, scale: Q16) -> i8 {
    let p = acc as i64 * scale.0 as i64;
    let r = (p + (1 << 15)) >> 16; // décalage arithmétique : arrondi vers +∞ sur les moitiés
    r.clamp(-128, 127) as i8
}

/// Couche dense int8 : sortie[j] = saturate( requantize( Σ_i poids[j][i] × entrée[i] + biais[j] ) ).
/// `poids` est rangé par ligne de sortie (out × inn).
pub struct Dense<'a> { pub inn: usize, pub out: usize, pub weights: &'a [i8], pub bias: &'a [i32], pub scale: Q16 }

impl<'a> Dense<'a> {
    pub fn check(&self) -> Result<(), Error> {
        if self.inn == 0 || self.out == 0 { return Err(Error::Shape); }
        if self.weights.len() != self.inn * self.out || self.bias.len() != self.out { return Err(Error::Shape); }
        // pire cas : |poids| ≤ 128, |entrée| ≤ 128 → 16384 par terme ; le biais ajoute au plus i32::MAX/2 si on le borne
        if (self.inn as u64) * 16384 > (i32::MAX as u64) / 2 { return Err(Error::Overflow); }
        if self.scale.0 == 0 { return Err(Error::Scale); }
        Ok(())
    }
    /// Calcule la couche. Vérifie tout avant de calculer ; ne modifie `output` qu'en cas de succès.
    pub fn forward(&self, input: &[i8], output: &mut [i8]) -> Result<(), Error> {
        self.check()?;
        if input.len() != self.inn || output.len() != self.out { return Err(Error::Shape); }
        for j in 0..self.out {
            let row = &self.weights[j * self.inn..(j + 1) * self.inn];
            let mut acc: i32 = 0;
            for i in 0..self.inn { acc += row[i] as i32 * input[i] as i32; }
            output[j] = requantize(acc.saturating_add(self.bias[j]), self.scale);
        }
        Ok(())
    }
}

pub fn relu(x: &mut [i8]) { for v in x.iter_mut() { if *v < 0 { *v = 0; } } }

/// Indice du maximum ; en cas d'égalité, le plus petit indice (déterministe). None si la liste est vide.
pub fn argmax(x: &[i8]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (i, v) in x.iter().enumerate() { if best.map_or(true, |b| *v > x[b]) { best = Some(i); } }
    best
}

/// Quantifie un tableau de flottants en int8 avec une échelle donnée (valeur = entier × scale). Renvoie le nombre de valeurs saturées.
pub fn quantize_f32(src: &[f32], scale: f32, dst: &mut [i8]) -> Result<usize, Error> {
    if src.len() != dst.len() { return Err(Error::Shape); }
    if !(scale > 0.0) { return Err(Error::Scale); }
    let mut sat = 0;
    for (d, s) in dst.iter_mut().zip(src) {
        let q = s / scale;
        // arrondi au plus proche sans std ; les valeurs non finies (NaN, infinis) sont comptées comme saturées
        let c: i32 = if !(q > -1.0e9 && q < 1.0e9) { sat += 1; if q > 0.0 { 127 } else if q < 0.0 { -128 } else { 0 } }
            else { let r = if q >= 0.0 { (q + 0.5) as i32 } else { (q - 0.5) as i32 }; if r > 127 { sat += 1; 127 } else if r < -128 { sat += 1; -128 } else { r } };
        *d = c as i8;
    }
    Ok(sat)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Générateur pseudo-aléatoire déterministe (xorshift) : les tests sont reproductibles.
    struct Rng(u64);
    impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } fn i8(&mut self) -> i8 { (self.next() >> 24) as i8 } }

    #[test] fn requantize_rounds_and_saturates() {
        assert_eq!(requantize(100, Q16::ONE), 100);
        assert_eq!(requantize(1000, Q16::ONE), 127, "saturation haute");
        assert_eq!(requantize(-1000, Q16::ONE), -128, "saturation basse");
        assert_eq!(requantize(3, Q16(32768)), 2, "1,5 arrondi à 2");
        assert_eq!(requantize(-3, Q16(32768)), -1, "-1,5 arrondi vers +∞ à -1 (règle documentée)");
        assert_eq!(requantize(i32::MAX, Q16(u32::MAX)), 127, "pas de débordement i64");
        assert_eq!(requantize(i32::MIN, Q16(u32::MAX)), -128);
    }
    #[test] fn dense_matches_exact_integer_reference() {
        let mut r = Rng(0x1234_5678_9ABC_DEF1);
        let (inn, out) = (37, 11);
        let w: std::vec::Vec<i8> = (0..inn * out).map(|_| r.i8()).collect();
        let b: std::vec::Vec<i32> = (0..out).map(|_| (r.next() % 2000) as i32 - 1000).collect();
        let x: std::vec::Vec<i8> = (0..inn).map(|_| r.i8()).collect();
        let sc = Q16::from_f32(0.003).unwrap();
        let d = Dense { inn, out, weights: &w, bias: &b, scale: sc };
        let mut y = std::vec![0i8; out];
        d.forward(&x, &mut y).unwrap();
        for j in 0..out {
            let mut acc: i64 = b[j] as i64;
            for i in 0..inn { acc += w[j * inn + i] as i64 * x[i] as i64; }
            let want = (((acc * sc.0 as i64) + (1 << 15)) >> 16).clamp(-128, 127) as i8;
            assert_eq!(y[j], want, "sortie {j}");
        }
    }
    #[test] fn dense_close_to_float_reference() {
        // conformité numérique : l'erreur int8 reste à ±1 quantum de la valeur flottante lorsque rien ne sature
        let mut r = Rng(42);
        let (inn, out) = (64, 8);
        let w: std::vec::Vec<i8> = (0..inn * out).map(|_| r.i8() / 8).collect();
        let x: std::vec::Vec<i8> = (0..inn).map(|_| r.i8() / 4).collect();
        let b = std::vec![0i32; out];
        let s = 0.01f32;
        let d = Dense { inn, out, weights: &w, bias: &b, scale: Q16::from_f32(s).unwrap() };
        let mut y = std::vec![0i8; out];
        d.forward(&x, &mut y).unwrap();
        for j in 0..out {
            let f: f64 = (0..inn).map(|i| w[j * inn + i] as f64 * x[i] as f64).sum::<f64>() * s as f64;
            if f.abs() < 126.0 { assert!((y[j] as f64 - f).abs() <= 1.0, "sortie {j} : int8 {} contre flottant {f}", y[j]); }
        }
    }
    #[test] fn shape_errors_are_detected_before_computing() {
        let w = [1i8; 6]; let b = [0i32; 2];
        let d = Dense { inn: 3, out: 2, weights: &w, bias: &b, scale: Q16::ONE };
        let mut y = [99i8; 2];
        assert_eq!(d.forward(&[1, 2], &mut y), Err(Error::Shape));
        assert_eq!(y, [99, 99], "sortie intacte en cas d'erreur");
        let mut y3 = [0i8; 3];
        assert_eq!(d.forward(&[1, 2, 3], &mut y3), Err(Error::Shape));
        let bad = Dense { inn: 3, out: 2, weights: &w[..5], bias: &b, scale: Q16::ONE };
        assert_eq!(bad.check(), Err(Error::Shape));
        let zero = Dense { inn: 0, out: 2, weights: &[], bias: &b, scale: Q16::ONE };
        assert_eq!(zero.check(), Err(Error::Shape));
    }
    #[test] fn too_wide_layer_is_rejected_not_wrapped() {
        let inn = 70_000;
        let w = std::vec![127i8; inn];
        let b = [0i32; 1];
        let d = Dense { inn, out: 1, weights: &w, bias: &b, scale: Q16::ONE };
        assert_eq!(d.check(), Err(Error::Overflow));
    }
    #[test] fn invalid_scale_is_rejected() {
        assert_eq!(Q16::from_f32(0.0), Err(Error::Scale));
        assert_eq!(Q16::from_f32(-1.0), Err(Error::Scale));
        assert_eq!(Q16::from_f32(f32::NAN), Err(Error::Scale));
        assert_eq!(Q16::from_f32(1e-9), Err(Error::Scale));
        assert_eq!(Q16::from_f32(40000.0), Err(Error::Scale));
    }
    #[test] fn relu_and_argmax_are_deterministic() {
        let mut v = [-5i8, 3, -1, 7, 7, 0];
        relu(&mut v);
        assert_eq!(v, [0, 3, 0, 7, 7, 0]);
        assert_eq!(argmax(&v), Some(3), "égalité : plus petit indice");
        assert_eq!(argmax(&[]), None);
    }
    #[test] fn quantize_counts_saturation() {
        let mut q = [0i8; 4];
        let sat = quantize_f32(&[0.5, 100.0, -100.0, 0.0], 0.5, &mut q).unwrap();
        assert_eq!(q, [1, 127, -128, 0]);
        assert_eq!(sat, 2);
        assert_eq!(quantize_f32(&[1.0], 0.0, &mut [0i8; 1]), Err(Error::Scale));
        assert_eq!(quantize_f32(&[1.0, 2.0], 1.0, &mut [0i8; 1]), Err(Error::Shape));
    }
    #[test] fn two_layer_network_runs_end_to_end_and_is_reproducible() {
        // réseau jouet 4 → 3 → 2 à poids fixés à la main : vérifie l'enchaînement, PAS une capacité d'apprentissage
        let w1: [i8; 12] = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1];
        let w2: [i8; 6] = [1, 0, 0, 0, 1, 1];
        let l1 = Dense { inn: 4, out: 3, weights: &w1, bias: &[0; 3], scale: Q16::ONE };
        let l2 = Dense { inn: 3, out: 2, weights: &w2, bias: &[0; 2], scale: Q16::ONE };
        let run = |x: &[i8]| { let mut h = [0i8; 3]; let mut o = [0i8; 2]; l1.forward(x, &mut h).unwrap(); relu(&mut h); l2.forward(&h, &mut o).unwrap(); argmax(&o).unwrap() };
        assert_eq!(run(&[10, 0, 0, 0]), 0);
        assert_eq!(run(&[0, 5, 5, 5]), 1);
        assert_eq!(run(&[10, 0, 0, 0]), run(&[10, 0, 0, 0]));
    }
}
