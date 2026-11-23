// Brique partagee UI Automation (COM a la main, sans dependance). LECTURE SEULE. Utilisee par `uia` et `lecteur`.
#![allow(dead_code)]
#[repr(C)] pub struct Guid(pub u32, pub u16, pub u16, pub [u8; 8]);
#[repr(C)] pub struct Variant { pub vt: u16, pub r: [u16; 3], pub data: [u64; 2] }
#[link(name = "ole32")] extern "system" {
    pub fn CoInitializeEx(r: *mut u8, f: u32) -> i32;
    pub fn CoCreateInstance(c: *const Guid, o: *mut u8, ctx: u32, i: *const Guid, out: *mut *mut *mut usize) -> i32;
}
#[link(name = "oleaut32")] extern "system" { pub fn VariantClear(v: *mut Variant) -> i32; }
#[link(name = "user32")] extern "system" {
    pub fn GetForegroundWindow() -> isize;
    pub fn GetWindowTextW(h: isize, buf: *mut u16, n: i32) -> i32;
    pub fn GetWindowThreadProcessId(h: isize, pid: *mut u32) -> u32;
}
pub type Obj = *mut *mut usize;
pub fn slot<T: Copy>(o: Obj, i: usize) -> T { unsafe { std::mem::transmute_copy(&*(*o).add(i)) } }
pub fn release(o: Obj) { if !o.is_null() { let f: extern "system" fn(Obj) -> u32 = slot(o, 2); f(o); } }

pub fn automation() -> Result<Obj, String> {
    let clsid = Guid(0xff48dba4, 0x60ef, 0x4201, [0xaa, 0x87, 0x54, 0x10, 0x3e, 0xef, 0x59, 0x4e]);
    let iid = Guid(0x30cbe57d, 0xd9d0, 0x452a, [0xab, 0x13, 0x7a, 0xc5, 0xac, 0x48, 0x25, 0xee]);
    unsafe {
        CoInitializeEx(std::ptr::null_mut(), 0);
        let mut a: Obj = std::ptr::null_mut();
        let hr = CoCreateInstance(&clsid, std::ptr::null_mut(), 23, &iid, &mut a);
        if hr != 0 || a.is_null() { Err(format!("UI Automation indisponible (0x{:08X})", hr as u32)) } else { Ok(a) }
    }
}
// IUIAutomation : GetRootElement=5, ElementFromHandle=6, GetFocusedElement=8, get_ControlViewWalker=14, CreateTrueCondition=21
pub fn root(a: Obj) -> Obj { let f: extern "system" fn(Obj, *mut Obj) -> i32 = slot(a, 5); let mut e = std::ptr::null_mut(); f(a, &mut e); e }
pub fn from_handle(a: Obj, h: isize) -> Obj { let f: extern "system" fn(Obj, isize, *mut Obj) -> i32 = slot(a, 6); let mut e = std::ptr::null_mut(); f(a, h, &mut e); e }
pub fn focused(a: Obj) -> Obj { let f: extern "system" fn(Obj, *mut Obj) -> i32 = slot(a, 8); let mut e = std::ptr::null_mut(); f(a, &mut e); e }
pub fn walker(a: Obj) -> Obj { let f: extern "system" fn(Obj, *mut Obj) -> i32 = slot(a, 14); let mut e = std::ptr::null_mut(); f(a, &mut e); e }
// IUIAutomationTreeWalker : GetParentElement=3, GetFirstChildElement=4, GetNextSiblingElement=6, GetPreviousSiblingElement=7
fn walk(w: Obj, e: Obj, i: usize) -> Obj { let f: extern "system" fn(Obj, Obj, *mut Obj) -> i32 = slot(w, i); let mut r = std::ptr::null_mut(); f(w, e, &mut r); r }
pub fn parent(w: Obj, e: Obj) -> Obj { walk(w, e, 3) }
pub fn first_child(w: Obj, e: Obj) -> Obj { walk(w, e, 4) }
pub fn next_sibling(w: Obj, e: Obj) -> Obj { walk(w, e, 6) }
pub fn prev_sibling(w: Obj, e: Obj) -> Obj { walk(w, e, 7) }

// IUIAutomationElement : GetCurrentPropertyValue=10
fn prop(e: Obj, id: i32) -> Option<Variant> {
    let f: extern "system" fn(Obj, i32, *mut Variant) -> i32 = slot(e, 10);
    let mut v = Variant { vt: 0, r: [0; 3], data: [0; 2] };
    if f(e, id, &mut v) == 0 { Some(v) } else { None }
}
pub fn text(e: Obj, id: i32) -> String {
    match prop(e, id) {
        Some(mut v) => {
            let s = if v.vt == 8 && v.data[0] != 0 { let p = v.data[0] as *const u16; let mut n = 0; unsafe { while *p.add(n) != 0 { n += 1; } String::from_utf16_lossy(std::slice::from_raw_parts(p, n)) } } else { String::new() };
            unsafe { VariantClear(&mut v); } s
        }
        None => String::new(),
    }
}
pub fn int(e: Obj, id: i32) -> i32 { match prop(e, id) { Some(mut v) => { let r = if v.vt == 3 { v.data[0] as i32 } else { -1 }; unsafe { VariantClear(&mut v); } r } None => -1 } }
/// Some(true/false) si la propriete est un booleen, None si absente ou d'un autre type.
pub fn boolean(e: Obj, id: i32) -> Option<bool> { match prop(e, id) { Some(mut v) => { let r = if v.vt == 11 { Some((v.data[0] & 0xFFFF) != 0) } else { None }; unsafe { VariantClear(&mut v); } r } None => None } }
pub fn name(e: Obj) -> String { text(e, 30005) }
pub fn ctype(e: Obj) -> i32 { int(e, 30003) }
pub fn pid(e: Obj) -> i32 { int(e, 30002) }
pub fn fg_title_pid() -> (String, u32) {
    unsafe {
        let h = GetForegroundWindow(); let mut buf = [0u16; 512]; let n = GetWindowTextW(h, buf.as_mut_ptr(), 512);
        let mut pid = 0u32; GetWindowThreadProcessId(h, &mut pid);
        (String::from_utf16_lossy(&buf[..n.max(0) as usize]), pid)
    }
}
pub fn role(t: i32) -> &'static str {
    match t {
        50000 => "bouton", 50001 => "calendrier", 50002 => "case à cocher", 50003 => "liste déroulante", 50004 => "zone de saisie", 50005 => "lien", 50006 => "image",
        50007 => "élément de liste", 50008 => "liste", 50009 => "menu", 50010 => "barre de menus", 50011 => "élément de menu", 50012 => "barre de progression", 50013 => "bouton radio",
        50014 => "barre de défilement", 50015 => "curseur", 50016 => "bouton à incrément", 50017 => "barre d'état", 50018 => "onglets", 50019 => "onglet", 50020 => "texte",
        50021 => "barre d'outils", 50022 => "info-bulle", 50023 => "arbre", 50024 => "élément d'arbre", 50026 => "groupe", 50028 => "élément de données", 50029 => "tableau de données",
        50030 => "document", 50032 => "fenêtre", 50033 => "panneau", 50034 => "en-tête", 50035 => "élément d'en-tête", 50036 => "tableau", 50037 => "barre de titre", 50038 => "séparateur",
        _ => "élément",
    }
}

/// Description lisible d'un element : { nom, role, etats, valeur, position }.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Desc { pub name: String, pub role: String, pub states: Vec<String>, pub value: Option<String>, pub pos: Option<String> }
impl Desc {
    pub fn parole(&self) -> String {
        let mut p: Vec<String> = vec![];
        if !self.name.is_empty() { p.push(self.name.clone()); }
        p.push(self.role.clone());
        p.extend(self.states.iter().cloned());
        if let Some(v) = &self.value { p.push(format!("valeur {v}")); }
        if let Some(x) = &self.pos { p.push(x.clone()); }
        p.join(", ")
    }
}

fn position_among_siblings(w: Obj, e: Obj, ct: i32) -> Option<(usize, usize)> {
    if w.is_null() { return None; }
    let (mut before, mut after) = (0usize, 0usize);
    let mut cur = prev_sibling(w, e);
    while !cur.is_null() && before < 2000 { let nxt = prev_sibling(w, cur); if ctype(cur) == ct { before += 1; } release(cur); cur = nxt; }
    let mut cur = next_sibling(w, e);
    while !cur.is_null() && after < 2000 { let nxt = next_sibling(w, cur); if ctype(cur) == ct { after += 1; } release(cur); cur = nxt; }
    Some((before + 1, before + after + 1))
}

pub fn describe(e: Obj, w: Obj) -> Desc {
    let ct = ctype(e);
    let mut d = Desc { name: name(e), role: role(ct).to_string(), ..Default::default() };
    let password = boolean(e, 30019) == Some(true);
    if boolean(e, 30010) == Some(false) { d.states.push("indisponible".into()); }
    let (has_toggle, has_sel, has_exp) = (boolean(e, 30041) == Some(true), boolean(e, 30036) == Some(true), boolean(e, 30028) == Some(true));
    match if has_toggle { int(e, 30086) } else { -1 } { // etat de bascule (TogglePattern)
        0 => d.states.push(if ct == 50002 { "non cochée".into() } else { "désactivé".into() }),
        1 => d.states.push(if ct == 50002 { "cochée".into() } else { "activé".into() }),
        2 => d.states.push("partiellement cochée".into()),
        _ => {}
    }
    match if has_sel { boolean(e, 30079) } else { None } { Some(true) => d.states.push("sélectionné".into()), Some(false) if ct == 50013 => d.states.push("non sélectionné".into()), _ => {} }
    match if has_exp { int(e, 30070) } else { -1 } { 0 => d.states.push("réduit".into()), 1 => d.states.push("développé".into()), _ => {} }
    if password { d.states.push("mot de passe".into()); d.value = Some("protégée, non lue".into()); }
    else {
        let v = text(e, 30045);
        // pas de valeur pour un document ou une fenêtre (adresse web, texte entier) ; valeur longue tronquée
        if !v.is_empty() && ct != 50030 && ct != 50032 {
            let court: String = v.chars().take(120).collect();
            d.value = Some(if v.chars().count() > 120 { format!("{court}…") } else { court });
        }
        else if ct == 50015 || ct == 50012 { /* curseur, progression : valeur numerique */ }
    }
    if boolean(e, 30046) == Some(true) && ct == 50004 { d.states.push("lecture seule".into()); }
    let (p, n) = (int(e, 30152), int(e, 30153));
    if p > 0 && n > 0 { d.pos = Some(format!("{p} sur {n}")); }
    else if matches!(ct, 50007 | 50011 | 50013 | 50019 | 50024) {
        if let Some((p, n)) = position_among_siblings(w, e, ct) { if n > 1 { d.pos = Some(format!("{p} sur {n}")); } }
    }
    d
}

// ---------------------------------------------------------------- lecture de texte (TextPattern), fenetres, dialogues, souris
#[link(name = "oleaut32")] extern "system" { fn SysFreeString(b: *mut u16); }
#[link(name = "user32")] extern "system" { fn GetCursorPos(p: *mut [i32; 2]) -> i32; }
fn bstr_vers_texte(p: *mut u16) -> String {
    if p.is_null() { return String::new(); }
    let mut n = 0; unsafe { while *p.add(n) != 0 { n += 1; } let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, n)); SysFreeString(p); s }
}
/// Motif (pattern) d'un element : TextPattern = 10014, ValuePattern = 10002. Renvoie un objet COM a liberer, ou nul.
pub fn pattern(e: Obj, id: i32) -> Obj { let f: extern "system" fn(Obj, i32, *mut Obj) -> i32 = slot(e, 16); let mut p = std::ptr::null_mut(); f(e, id, &mut p); p }
/// Texte de l'unite (0 caractere, 2 mot, 3 ligne, 4 paragraphe) qui entoure le curseur ou la selection ; None si l'element n'a pas de TextPattern.
pub fn texte_unite(e: Obj, unite: i32) -> Option<String> {
    let tp = pattern(e, 10014); if tp.is_null() { return None; }
    let get_sel: extern "system" fn(Obj, *mut Obj) -> i32 = slot(tp, 5); let mut arr = std::ptr::null_mut();
    get_sel(tp, &mut arr);
    let mut resultat = None;
    if !arr.is_null() {
        let len: extern "system" fn(Obj, *mut i32) -> i32 = slot(arr, 3); let get: extern "system" fn(Obj, i32, *mut Obj) -> i32 = slot(arr, 4);
        let mut n = 0; len(arr, &mut n);
        if n > 0 {
            let mut r = std::ptr::null_mut(); get(arr, 0, &mut r);
            if !r.is_null() {
                let expand: extern "system" fn(Obj, i32) -> i32 = slot(r, 6); expand(r, unite);
                let gt: extern "system" fn(Obj, i32, *mut *mut u16) -> i32 = slot(r, 12); let mut b = std::ptr::null_mut(); gt(r, 2000, &mut b);
                resultat = Some(bstr_vers_texte(b).trim_matches(|c: char| c == '\r' || c == '\n' || c == ' ').to_string()); release(r);
            }
        }
        release(arr);
    }
    release(tp); resultat
}
/// Texte du document entier (2000 caracteres au plus).
pub fn texte_tout(e: Obj) -> Option<String> {
    let tp = pattern(e, 10014); if tp.is_null() { return None; }
    let doc: extern "system" fn(Obj, *mut Obj) -> i32 = slot(tp, 7); let mut r = std::ptr::null_mut(); doc(tp, &mut r);
    let mut t = None;
    if !r.is_null() { let gt: extern "system" fn(Obj, i32, *mut *mut u16) -> i32 = slot(r, 12); let mut b = std::ptr::null_mut(); gt(r, 2000, &mut b); t = Some(bstr_vers_texte(b)); release(r); }
    release(tp); t
}
pub fn classe(e: Obj) -> String { text(e, 30012) }
/// Fenetre de premier niveau qui contient l'element (nouvelle reference), ou nul.
pub fn fenetre_de(w: Obj, e: Obj) -> Obj {
    let mut cur = parent(w, e); let mut derniere: Obj = std::ptr::null_mut(); let mut n = 0;
    if ctype(e) == 50032 { let p = parent(w, e); release(p); return from_self(e); }
    while !cur.is_null() && n < 30 { if ctype(cur) == 50032 { if !derniere.is_null() { release(derniere); } derniere = cur; let nx = parent(w, cur); cur = nx; } else { let nx = parent(w, cur); release(cur); cur = nx; } n += 1; }
    if !cur.is_null() { release(cur); }
    derniere
}
fn from_self(e: Obj) -> Obj { let f: extern "system" fn(Obj) -> u32 = slot(e, 1); f(e); e } // AddRef
/// Description d'une boite de dialogue standard (classe #32770) : titre, textes, boutons. None si ce n'est pas un dialogue.
pub fn description_dialogue(w: Obj, fen: Obj) -> Option<String> {
    if fen.is_null() || classe(fen) != "#32770" { return None; }
    let (mut textes, mut boutons) = (vec![], vec![]);
    let mut c = first_child(w, fen);
    while !c.is_null() {
        let t = ctype(c); let nom = name(c);
        if !nom.is_empty() { if t == 50020 { textes.push(nom); } else if t == 50000 { boutons.push(nom); } }
        let nx = next_sibling(w, c); release(c); c = nx;
    }
    Some(format!("Boîte de dialogue : {}. {}{}", name(fen), if textes.is_empty() { String::new() } else { format!("{}. ", textes.join(". ")) }, if boutons.is_empty() { String::new() } else { format!("Boutons : {}", boutons.join(", ")) }).trim().to_string())
}
/// Element sous le pointeur de la souris (nouvelle reference) et position.
pub fn sous_la_souris(a: Obj) -> (Obj, i32, i32) {
    let mut p = [0i32; 2]; unsafe { GetCursorPos(&mut p); }
    let f: extern "system" fn(Obj, i64, *mut Obj) -> i32 = slot(a, 7); let mut e = std::ptr::null_mut();
    f(a, (p[0] as u32 as i64) | ((p[1] as i64) << 32), &mut e);
    (e, p[0], p[1])
}
