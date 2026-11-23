//! Voxa Speech : chaîne vocale native, `no_std`, sans allocation.
//! `phon` : texte français -> notation phonétique (port de phon.vx). `synth` : phonétique -> échantillons audio (port du synthétiseur à formants).
#![cfg_attr(not(test), no_std)]
pub mod phon;
pub mod synth;
