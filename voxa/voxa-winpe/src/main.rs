#![cfg(windows)]
//! Voxa pour WinPE / Windows : le même cœur (silicon-core) et la même voix (voxa-speech) que Voxa Live, dans un programme natif Windows x64
//! sans aucune dépendance externe (appels directs à kernel32 et winmm). Pensé pour être lancé par `startnet.cmd` dans un WinPE construit avec l'ADK.
//!
//! Usage : voxa-winpe.exe                      session interactive (console), voix par WinMM si un périphérique audio existe
//!         voxa-winpe.exe --touches "down,enter"  mode d'essai sans console : touches scriptées, annonces écrites sur la sortie standard
//!         voxa-winpe.exe --audio-probe          dit combien de périphériques de sortie audio WinMM existent et quitte
//! Variables : VOXA_WINPE=1 autorise réellement `wpeutil reboot|shutdown` (sinon l'extinction/redémarrage sont seulement simulés et écrits).
//! Statut : NOT_VALIDATED dans un vrai WinPE (l'audio en WinPE n'est pas un scénario pris en charge par Microsoft).
use silicon_core::menu::{Key, MenuApp, Platform, Power};
use silicon_core::sysinfo::{smbios_lines, Lines};
use silicon_core::Out;
use std::io::Write;
use voxa_speech::phon::{phon_texte, utf8_to_u16, PhonOut};
use voxa_speech::synth::{parler, Work, FS};

// ---------------------------------------------------------------- interfaces Windows (déclarées à la main)
#[repr(C, packed)] struct WaveFormatEx { tag: u16, channels: u16, rate: u32, avg: u32, align: u16, bits: u16, cb: u16 }
#[repr(C)] struct WaveHdr { data: *mut u8, len: u32, rec: u32, user: usize, flags: u32, loops: u32, next: *mut WaveHdr, reserved: usize }
#[repr(C)] struct InputRecord { kind: u16, _pad: u16, down: i32, repeat: u16, vk: u16, scan: u16, ch: u16, ctrl: u32 }

#[link(name = "winmm")] extern "system" {
    fn waveOutGetNumDevs() -> u32;
    fn waveOutOpen(h: *mut isize, id: u32, fmt: *const WaveFormatEx, cb: usize, inst: usize, flags: u32) -> u32;
    fn waveOutPrepareHeader(h: isize, hdr: *mut WaveHdr, size: u32) -> u32;
    fn waveOutWrite(h: isize, hdr: *mut WaveHdr, size: u32) -> u32;
    fn waveOutUnprepareHeader(h: isize, hdr: *mut WaveHdr, size: u32) -> u32;
    fn waveOutClose(h: isize) -> u32;
}
#[link(name = "kernel32")] extern "system" {
    fn Beep(freq: u32, ms: u32) -> i32;
    fn Sleep(ms: u32);
    fn GetStdHandle(n: u32) -> isize;
    fn GetConsoleMode(h: isize, mode: *mut u32) -> i32;
    fn ReadConsoleInputW(h: isize, buf: *mut InputRecord, n: u32, read: *mut u32) -> i32;
    fn GetSystemFirmwareTable(sig: u32, id: u32, buf: *mut u8, size: u32) -> u32;
}
const WAVE_MAPPER: u32 = 0xFFFF_FFFF; const WHDR_DONE: u32 = 1;

const MAX_SAMPLES: usize = 90_000;

// ---------------------------------------------------------------- voix du système (SAPI 5) : voix française claire (ex. Hortense), sans dépendance externe
use std::ffi::c_void;
#[link(name = "ole32")] extern "system" {
    fn CoInitializeEx(reserved: *const c_void, flags: u32) -> i32;
    fn CoCreateInstance(clsid: *const [u8; 16], outer: *const c_void, ctx: u32, iid: *const [u8; 16], out: *mut *mut c_void) -> i32;
}
const CLSID_SPVOICE: [u8; 16] = [0x77, 0x93, 0x74, 0x96, 0x91, 0x33, 0xD2, 0x11, 0x9E, 0xE3, 0x00, 0xC0, 0x4F, 0x79, 0x73, 0x96]; // {96749377-3391-11D2-9EE3-00C04F797396}
const IID_ISPVOICE: [u8; 16] = [0x74, 0xDF, 0x44, 0x6C, 0xB9, 0x72, 0x92, 0x49, 0xA1, 0xEC, 0xEF, 0x99, 0x6E, 0x04, 0x22, 0xD4];   // {6C44DF74-72B9-4992-A1EC-EF996E0422D4}
/// ISpVoice par sa table de méthodes COM : Speak = 20, SetRate = 28, WaitUntilDone = 32 (13 méthodes héritées + rang dans ISpVoice).
struct Sapi { p: *mut c_void }
impl Sapi {
    fn new() -> Option<Sapi> {
        unsafe {
            CoInitializeEx(std::ptr::null(), 0); // COINIT_MULTITHREADED
            let mut p: *mut c_void = std::ptr::null_mut();
            if CoCreateInstance(&CLSID_SPVOICE, std::ptr::null(), 1, &IID_ISPVOICE, &mut p) != 0 || p.is_null() { return None; }
            Some(Sapi { p })
        }
    }
    unsafe fn method(&self, i: usize) -> usize { let vt = *(self.p as *const *const usize); *vt.add(i) }
    /// Parle de façon asynchrone ; `purge` coupe la parole en cours (SPF_ASYNC | SPF_PURGEBEFORESPEAK | SPF_IS_NOT_XML).
    fn speak(&self, text: &str, purge: bool) -> bool {
        let w: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        unsafe { let f: extern "system" fn(*mut c_void, *const u16, u32, *mut u32) -> i32 = std::mem::transmute(self.method(20)); f(self.p, w.as_ptr(), 1 | 16 | if purge { 2 } else { 0 }, std::ptr::null_mut()) >= 0 }
    }
    fn wait(&self, ms: u32) { unsafe { let f: extern "system" fn(*mut c_void, u32) -> i32 = std::mem::transmute(self.method(32)); f(self.p, ms); } }
    fn set_rate(&self, r: i32) { unsafe { let f: extern "system" fn(*mut c_void, i32) -> i32 = std::mem::transmute(self.method(28)); f(self.p, r); } }
}

struct WinPlatform { work: Box<Work>, mono: Vec<i16>, audio_devices: u32, play: bool, scripted: bool, simulate_power: bool, sapi: Option<Sapi>, formant_only: bool, audible: bool }

impl WinPlatform {
    fn new(scripted: bool) -> WinPlatform {
        let n = unsafe { waveOutGetNumDevs() };
        WinPlatform { work: Box::new(Work::new()), mono: vec![0i16; MAX_SAMPLES], audio_devices: n, play: n > 0, scripted, simulate_power: std::env::var("VOXA_WINPE").ok().as_deref() != Some("1"), sapi: None, formant_only: std::env::var("VOXA_ENGINE").ok().as_deref() == Some("formant"), audible: !scripted || std::env::var("VOXA_AUDIO").ok().as_deref() == Some("1") }
    }
    /// Joue un tampon mono 16 bits ; renvoie false si le périphérique refuse ou ne répond pas (secours par bips côté cœur).
    fn jouer(&self, samples: &[i16]) -> bool {
        let fmt = WaveFormatEx { tag: 1, channels: 1, rate: FS as u32, avg: FS as u32 * 2, align: 2, bits: 16, cb: 0 };
        let mut h: isize = 0;
        unsafe {
            if waveOutOpen(&mut h, WAVE_MAPPER, &fmt, 0, 0, 0) != 0 { return false; }
            let mut hdr = WaveHdr { data: samples.as_ptr() as *mut u8, len: (samples.len() * 2) as u32, rec: 0, user: 0, flags: 0, loops: 0, next: std::ptr::null_mut(), reserved: 0 };
            let sz = std::mem::size_of::<WaveHdr>() as u32;
            let mut ok = waveOutPrepareHeader(h, &mut hdr, sz) == 0 && waveOutWrite(h, &mut hdr, sz) == 0;
            if ok { let mut waited = 0; while std::ptr::read_volatile(&hdr.flags) & WHDR_DONE == 0 { Sleep(5); waited += 5; if waited > 30_000 { ok = false; break; } } }
            waveOutUnprepareHeader(h, &mut hdr, sz); waveOutClose(h);
            ok
        }
    }
    fn parler_audio(&mut self, text: &str) -> bool {
        if !self.play { return false; }
        let mut src = [0u16; 256]; let n = utf8_to_u16(text, &mut src);
        let (mut d, mut ok) = (0usize, true); let mut o = PhonOut::new();
        while d < n {
            let nd = phon_texte(&src[..n], d, &mut o); d = if nd > d { nd } else { d + 1 };
            if o.len() == 0 { continue; }
            let r = parler(&mut self.work, o.as_str(), &mut self.mono);
            if r.samples == 0 { continue; }
            let samples = self.mono[..r.samples].to_vec();
            if !self.jouer(&samples) { ok = false; }
            if r.truncated { ok = false; }
        }
        ok
    }
}

fn say_line(s: &str) { let mut out = std::io::stdout().lock(); let _ = writeln!(out, "{s}"); let _ = out.flush(); }

fn cpu_brand() -> String {
    let mut b = [0u8; 48];
    for (i, leaf) in (0x8000_0002u32..=0x8000_0004).enumerate() { let r = std::arch::x86_64::__cpuid(leaf); for (j, w) in [r.eax, r.ebx, r.ecx, r.edx].iter().enumerate() { b[i * 16 + j * 4..i * 16 + j * 4 + 4].copy_from_slice(&w.to_le_bytes()); } }
    String::from_utf8_lossy(&b).trim_matches(|c: char| c == '\0' || c == ' ').to_string()
}

impl Platform for WinPlatform {
    fn speak(&mut self, text: &str, interrupt: bool) -> bool {
        if self.scripted { say_line(&format!("ANNONCE: {text}")); } else { say_line(text); }
        // Mode d'essai silencieux : le texte écrit est la sortie ; on ne déclenche PAS le secours par bips (aucun son parasite pendant les tests).
        if !self.audible { return true; }
        // Voix du système d'abord (voix française claire) ; synthèse maison en secours ; sinon le cœur passera aux bips.
        if let Some(s) = self.sapi.as_ref() { if s.speak(text, interrupt) { return true; } }
        self.parler_audio(text)
    }
    fn power(&mut self, p: Power) {
        let verbe = match p { Power::Restart => "reboot", Power::Shutdown => "shutdown" };
        if self.simulate_power { say_line(&format!("POWER: {verbe} (simulé : VOXA_WINPE non défini)")); return; }
        let _ = std::process::Command::new("wpeutil").arg(verbe).status();
    }
    fn test_sound(&mut self) -> bool {
        if !self.audible { return true; }
        // Le test du son est une phrase dite avec la voix choisie (pas un bip) ; bip seulement si aucune voix ne marche.
        if let Some(s) = self.sapi.as_ref() { if s.speak("Ceci est un test du son. Si vous m'entendez, la voix fonctionne.", true) { return true; } }
        if self.parler_audio("Ceci est un test du son.") { return true; }
        unsafe { Beep(880, 300) != 0 }
    }
    fn hardware_summary(&self, out: &mut Out) { out.push("Processeur "); out.push(&cpu_brand()); out.push("."); }
    fn system_lines(&self, lines: &mut Lines) {
        let cpu = cpu_brand();
        let mut buf = vec![0u8; 1 << 16];
        let n = unsafe { GetSystemFirmwareTable(0x5253_4D42, 0, buf.as_mut_ptr(), buf.len() as u32) } as usize; // 'RSMB'
        if n > 8 && n <= buf.len() {
            let len = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
            if 8 + len <= n { smbios_lines(&buf[8..8 + len], &cpu, lines); return; }
        }
        let mut o = Out::new(); o.push("Processeur : "); o.push(&cpu); lines.add(o.as_str()); lines.add("Tables SMBIOS indisponibles.");
        let mut o = Out::new(); o.push("Périphériques audio WinMM : "); o.push(&self.audio_devices.to_string()); lines.add(o.as_str());
    }
    fn beep(&mut self, freq: u32, ms: u32) -> bool {
        if !self.audible { return true; } // aucun bip pendant les essais scriptés
        unsafe { if freq == 0 { Sleep(ms); true } else { Beep(freq, ms) != 0 } }
    }
}

// ---------------------------------------------------------------- entrées
fn parse_touches(s: &str) -> Vec<Key> {
    s.split(|c: char| c == ',' || c.is_whitespace()).filter(|t| !t.is_empty()).filter_map(|t| Some(match t {
        "up" => Key::Up, "down" => Key::Down, "left" => Key::Left, "right" => Key::Right, "home" => Key::Home, "end" => Key::End, "pgup" => Key::PageUp, "pgdn" => Key::PageDown,
        "esc" => Key::Escape, "enter" => Key::Enter, "tab" => Key::Tab, "bksp" => Key::Backspace, "del" => Key::Delete, "space" => Key::Char(' '),
        _ => Key::Char(t.strip_prefix("char:")?.chars().next()?),
    })).collect()
}

fn map_vk(vk: u16, ch: u16) -> Option<Key> {
    match vk { 0x26 => Some(Key::Up), 0x28 => Some(Key::Down), 0x25 => Some(Key::Left), 0x27 => Some(Key::Right), 0x24 => Some(Key::Home), 0x23 => Some(Key::End), 0x21 => Some(Key::PageUp), 0x22 => Some(Key::PageDown),
        0x0D => Some(Key::Enter), 0x1B => Some(Key::Escape), 0x08 => Some(Key::Backspace), 0x2E => Some(Key::Delete), 0x09 => Some(Key::Tab),
        _ => if ch >= 32 { char::from_u32(ch as u32).map(Key::Char) } else { None } }
}

fn lire_touche_console() -> Option<Key> {
    unsafe {
        let h = GetStdHandle(0xFFFF_FFF6); // STD_INPUT_HANDLE
        let mut mode = 0u32; if GetConsoleMode(h, &mut mode) == 0 { return None; }
        let mut rec = InputRecord { kind: 0, _pad: 0, down: 0, repeat: 0, vk: 0, scan: 0, ch: 0, ctrl: 0 }; let mut n = 0u32;
        if ReadConsoleInputW(h, &mut rec, 1, &mut n) == 0 || n == 0 { return None; }
        if rec.kind == 1 && rec.down != 0 { map_vk(rec.vk, rec.ch) } else { None }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg_after = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    // --audio-probe : SILENCIEUX. Dit s'il existe un périphérique audio WinMM, si la voix du système (SAPI) s'initialise et combien d'échantillons la synthèse maison produit.
    if args.iter().any(|a| a == "--audio-probe") {
        println!("AUDIO_DEVICES={}", unsafe { waveOutGetNumDevs() });
        println!("SAPI_OK={}", Sapi::new().is_some());
        let mut p = WinPlatform::new(true);
        let mut o = PhonOut::new(); let mut src = [0u16; 8]; let k = utf8_to_u16("bonjour", &mut src); phon_texte(&src[..k], 0, &mut o);
        let r = parler(&mut p.work, o.as_str(), &mut p.mono); println!("SYNTH_SAMPLES={}", r.samples);
        return;
    }
    // --dire "texte" [--formant] : dit le texte à voix haute (voix du système par défaut, synthèse maison avec --formant) puis quitte.
    if let Some(texte) = arg_after("--dire") {
        let formant = args.iter().any(|a| a == "--formant");
        let mut p = WinPlatform::new(false);
        let sapi = if formant { None } else { Sapi::new() };
        let ok = match sapi.as_ref() { Some(s) => { let r = s.speak(&texte, true); s.wait(30_000); r } None => p.parler_audio(&texte) };
        println!("MOTEUR={} DIT={ok}", if sapi.is_some() { "SAPI" } else { "synthese-maison" });
        return;
    }
    let touches = arg_after("--touches").map(|s| parse_touches(&s));
    let mut platform = WinPlatform::new(touches.is_some());
    if platform.audible && !platform.formant_only { platform.sapi = Sapi::new(); }
    if touches.is_some() { say_line(&format!("AUDIO_DEVICES={}", platform.audio_devices)); say_line(&format!("MOTEUR={}", if !platform.audible { "silencieux" } else if platform.sapi.is_some() { "SAPI" } else { "synthese-maison" })); }
    let mut app = MenuApp::new();
    app.flush(&mut platform);
    if let Some(keys) = touches { for k in keys { app.handle(k); app.flush(&mut platform); } if let Some(s) = platform.sapi.as_ref() { s.wait(30_000); } return; }
    loop {
        match lire_touche_console() { Some(k) => { app.handle(k); app.flush(&mut platform); } None => unsafe { Sleep(20) } }
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("outil Windows uniquement");
}
