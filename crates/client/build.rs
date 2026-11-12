//! Indexes the art: every `art/src/**/*.px` (pixel source) and every `art/hd/**/*.png`
//! (high-resolution redraw, 4x the pixel size). `sprites::art("w1/boss")` then embeds the
//! HD PNG when one exists and falls back to the pixel source otherwise. Opaque textures (the
//! terrain) may be `.jpg` instead, much smaller; they are indexed the same way, by path
//! without the extension.
use std::fmt::Write;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, ext, out);
        } else if p.extension().is_some_and(|x| x == ext) {
            out.push(p);
        }
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../art").canonicalize().unwrap();
    let (src, hd) = (root.join("src"), root.join("hd"));
    println!("cargo:rerun-if-changed={}", src.display());
    println!("cargo:rerun-if-changed={}", hd.display());
    let key = |base: &Path, p: &Path| p.strip_prefix(base).unwrap().with_extension("").to_string_lossy().replace('\\', "/");
    let (mut pxs, mut pngs) = (Vec::new(), Vec::new());
    walk(&src, "px", &mut pxs);
    walk(&hd, "png", &mut pngs);
    walk(&hd, "jpg", &mut pngs);
    pxs.sort();
    pngs.sort();
    let mut s = String::new();
    s += "pub fn px(path: &str) -> &'static str {\n    match path {\n";
    for p in &pxs {
        println!("cargo:rerun-if-changed={}", p.display());
        writeln!(s, "        {:?} => include_str!({:?}),", key(&src, p), p.display().to_string()).unwrap();
    }
    s += "        _ => panic!(\"unknown art {path}\"),\n    }\n}\n\n";
    s += "pub fn png(path: &str) -> Option<&'static [u8]> {\n    match path {\n";
    for p in &pngs {
        println!("cargo:rerun-if-changed={}", p.display());
        writeln!(s, "        {:?} => Some(include_bytes!({:?})),", key(&hd, p), p.display().to_string()).unwrap();
    }
    s += "        _ => None,\n    }\n}\n";
    std::fs::write(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("art_index.rs"), s).unwrap();
}
