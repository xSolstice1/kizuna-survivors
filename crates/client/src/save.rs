//! Persists the meta `Profile`: localStorage on the web (via a tiny JS plugin in index.html),
//! a text file next to the binary on desktop.

use sim::meta::Profile;

#[cfg(target_arch = "wasm32")]
mod backend {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn kz_save(ptr: *const u8, len: u32);
        fn kz_load_len() -> u32;
        fn kz_load(ptr: *mut u8, len: u32);
    }

    /// Lets the JS side check that the plugin matches (see miniquad's plugin version check).
    #[no_mangle]
    pub extern "C" fn kz_storage_crate_version() -> u32 {
        1
    }

    pub fn write(s: &str) {
        unsafe { kz_save(s.as_ptr(), s.len() as u32) }
    }

    pub fn read() -> String {
        unsafe {
            let n = kz_load_len();
            let mut buf = vec![0u8; n as usize];
            kz_load(buf.as_mut_ptr(), n);
            String::from_utf8(buf).unwrap_or_default()
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    const PATH: &str = "kizuna_save.txt";

    pub fn write(s: &str) {
        let _ = std::fs::write(PATH, s);
    }

    pub fn read() -> String {
        std::fs::read_to_string(PATH).unwrap_or_default()
    }
}

pub fn load() -> Profile {
    Profile::parse(&backend::read())
}

pub fn store(p: &Profile) {
    backend::write(&p.serialize());
    crate::cloud::push(p);
}

/// Saves locally only (used when loading a cloud save, which needs no upload).
pub fn store_local(p: &Profile) {
    backend::write(&p.serialize());
}

/// The raw saved text (dev hooks read flags from it).
pub fn raw() -> String {
    backend::read()
}
