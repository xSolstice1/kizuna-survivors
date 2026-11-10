//! Text fields for in-game forms (sign-in, sign-up). In the browser the game draws each
//! field's box and web/input.js places a real <input> on top of it (mobile keyboards,
//! password managers, paste and IME all work). Natively a simple keyboard fallback is used.
//!
//! Each frame: `begin()`, then `field(...)` for every visible field, then `end()`.

use macroquad::prelude::*;

use crate::kit;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Text,
    Email,
    Password,
    NewPassword,
}

/// What the player did with the keyboard this frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    None,
    /// Enter in field `id`.
    Enter(u32),
    /// Tab out of field `id`.
    Tab(u32),
}

#[cfg(target_arch = "wasm32")]
mod backend {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn kz_input_begin();
        fn kz_input_field(id: u32, kind: u32, x: f32, y: f32, w: f32, h: f32, size: f32, ph: *const u8, ph_len: u32);
        fn kz_input_end();
        fn kz_input_focus(id: u32);
        fn kz_input_focused() -> i32;
        fn kz_input_get_len(id: u32) -> u32;
        fn kz_input_get(ptr: *mut u8, len: u32);
        fn kz_input_set(id: u32, ptr: *const u8, len: u32);
        fn kz_input_submitted() -> i32;
    }

    #[no_mangle]
    pub extern "C" fn kz_input_crate_version() -> u32 {
        1
    }

    pub fn begin() {
        unsafe { kz_input_begin() }
    }
    pub fn place(id: u32, kind: u32, r: [f32; 4], size: f32, ph: &str) {
        unsafe { kz_input_field(id, kind, r[0], r[1], r[2], r[3], size, ph.as_ptr(), ph.len() as u32) }
    }
    pub fn end() {
        unsafe { kz_input_end() }
    }
    pub fn focus(id: u32) {
        unsafe { kz_input_focus(id) }
    }
    pub fn focused() -> Option<u32> {
        let f = unsafe { kz_input_focused() };
        (f >= 0).then_some(f as u32)
    }
    pub fn value(id: u32) -> String {
        let n = unsafe { kz_input_get_len(id) };
        let mut buf = vec![0u8; n as usize];
        unsafe { kz_input_get(buf.as_mut_ptr(), n) };
        String::from_utf8(buf).unwrap_or_default()
    }
    #[allow(dead_code)]
    pub fn set(id: u32, s: &str) {
        unsafe { kz_input_set(id, s.as_ptr(), s.len() as u32) }
    }
    pub fn submitted() -> i32 {
        unsafe { kz_input_submitted() }
    }
    /// The browser draws the text itself.
    pub const DRAWS_TEXT: bool = false;
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use std::cell::RefCell;
    use std::collections::HashMap;

    thread_local! {
        static VALUES: RefCell<HashMap<u32, String>> = RefCell::new(HashMap::new());
        static FOCUS: RefCell<Option<u32>> = const { RefCell::new(None) };
        static KEY: RefCell<i32> = const { RefCell::new(-1) };
    }

    pub fn begin() {
        use macroquad::prelude::*;
        let focus = FOCUS.with(|f| *f.borrow());
        if let Some(id) = focus {
            while let Some(c) = get_char_pressed() {
                if !c.is_control() {
                    VALUES.with(|v| v.borrow_mut().entry(id).or_default().push(c));
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                VALUES.with(|v| {
                    v.borrow_mut().entry(id).or_default().pop();
                });
            }
            if is_key_pressed(KeyCode::Enter) {
                KEY.with(|k| *k.borrow_mut() = id as i32);
            }
            if is_key_pressed(KeyCode::Tab) {
                KEY.with(|k| *k.borrow_mut() = -2 - id as i32);
            }
        }
    }
    pub fn place(id: u32, _kind: u32, r: [f32; 4], _size: f32, _ph: &str) {
        use macroquad::prelude::*;
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= r[0] && mx <= r[0] + r[2] && my >= r[1] && my <= r[1] + r[3] {
                FOCUS.with(|f| *f.borrow_mut() = Some(id));
            }
        }
    }
    pub fn end() {}
    pub fn focus(id: u32) {
        FOCUS.with(|f| *f.borrow_mut() = Some(id));
    }
    pub fn focused() -> Option<u32> {
        FOCUS.with(|f| *f.borrow())
    }
    pub fn value(id: u32) -> String {
        VALUES.with(|v| v.borrow().get(&id).cloned().unwrap_or_default())
    }
    #[allow(dead_code)]
    pub fn set(id: u32, s: &str) {
        VALUES.with(|v| {
            v.borrow_mut().insert(id, s.to_string());
        });
    }
    pub fn submitted() -> i32 {
        KEY.with(|k| std::mem::replace(&mut *k.borrow_mut(), -1))
    }
    pub const DRAWS_TEXT: bool = true;
}

pub fn begin() {
    backend::begin();
}

pub fn end() {
    backend::end();
}

/// Draws a field (label above, box, focus ring) in virtual UI coordinates and places the
/// input over it.
pub fn field(id: u32, kind: Kind, r: Rect, label: &str, placeholder: &str) {
    let focused = backend::focused() == Some(id);
    kit::text(label, r.x + 2.0, r.y - 8.0, 13.0, kit::DIM);
    kit::rounded(r.x, r.y, r.w, r.h, 9.0, Color::new(0.04, 0.03, 0.08, 0.95));
    kit::rounded_outline(r.x, r.y, r.w, r.h, 9.0, if focused { 2.0 } else { 1.2 }, if focused { kit::GOLD } else { Color::new(1.0, 1.0, 1.0, 0.16) });
    let s = kit::ui_scale();
    let k = match kind {
        Kind::Text => 0,
        Kind::Email => 1,
        Kind::Password => 2,
        Kind::NewPassword => 3,
    };
    backend::place(id, k, [r.x * s, r.y * s, r.w * s, r.h * s], 16.0 * s, placeholder);
    if backend::DRAWS_TEXT {
        let v = value(id);
        let shown = if matches!(kind, Kind::Password | Kind::NewPassword) { "•".repeat(v.chars().count()) } else { v };
        let (t, c) = if shown.is_empty() && !focused { (placeholder.to_string(), kit::FAINT) } else { (shown, kit::INK) };
        kit::text_fit(&t, r.x + 12.0, r.y + r.h / 2.0 + 6.0, r.w - 24.0, 16.0, c);
    }
}

pub fn value(id: u32) -> String {
    backend::value(id)
}

#[allow(dead_code)]
pub fn set(id: u32, s: &str) {
    backend::set(id, s);
}

pub fn focus(id: u32) {
    backend::focus(id);
}

pub fn key() -> Key {
    match backend::submitted() {
        -1 => Key::None,
        n if n >= 0 => Key::Enter(n as u32),
        n => Key::Tab((-2 - n) as u32),
    }
}
