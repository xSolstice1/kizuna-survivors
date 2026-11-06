//! Platform bits: fullscreen. In the browser this calls a tiny JS plugin (web/index.html);
//! natively it toggles the window.

#[cfg(target_arch = "wasm32")]
mod backend {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn kz_fullscreen_toggle();
        fn kz_is_fullscreen() -> i32;
        fn kz_fullscreen_supported() -> i32;
    }
    pub fn toggle() {
        unsafe { kz_fullscreen_toggle() }
    }
    pub fn is_fullscreen() -> bool {
        unsafe { kz_is_fullscreen() != 0 }
    }
    pub fn supported() -> bool {
        unsafe { kz_fullscreen_supported() != 0 }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FULL: AtomicBool = AtomicBool::new(false);
    pub fn toggle() {
        let next = !FULL.load(Ordering::Relaxed);
        FULL.store(next, Ordering::Relaxed);
        macroquad::window::set_fullscreen(next);
    }
    pub fn is_fullscreen() -> bool {
        FULL.load(Ordering::Relaxed)
    }
    pub fn supported() -> bool {
        true
    }
}

/// Toggles fullscreen. Call from a click/tap handler (browsers require a user gesture).
pub fn toggle_fullscreen() {
    backend::toggle()
}

pub fn is_fullscreen() -> bool {
    backend::is_fullscreen()
}

/// False on browsers without a fullscreen API (iPhone Safari: use Add to Home Screen).
pub fn fullscreen_supported() -> bool {
    backend::supported()
}

/// Label for a fullscreen button.
pub fn fullscreen_label() -> &'static str {
    if !fullscreen_supported() {
        "Fullscreen: Add to Home Screen"
    } else if is_fullscreen() {
        "Exit fullscreen"
    } else {
        "Fullscreen"
    }
}
