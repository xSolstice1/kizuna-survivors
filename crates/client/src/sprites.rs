//! Parses `.px` sprite sources (see art/src) into RGBA textures.
//! Format: optional `mirror` line, palette lines `<char> #rrggbb`, `---`, then pixel rows.
//! `.` is transparent. With `mirror`, each row is the left half and is mirrored.

use macroquad::prelude::*;

pub struct Sprite {
    pub w: u16,
    pub h: u16,
    pub rgba: Vec<u8>,
    /// Already at `UP`x resolution (a high-resolution PNG): no upscaling.
    pub hd: bool,
}

mod index {
    include!(concat!(env!("OUT_DIR"), "/art_index.rs"));
}

/// The pixel source of an art path (e.g. "w1/boss").
pub fn px(path: &str) -> &'static str {
    index::px(path)
}

fn decode_hd(path: &str) -> Option<Sprite> {
    let bytes = index::png(path)?;
    let img = Image::from_file_with_format(bytes, Some(ImageFormat::Png)).ok()?;
    // Must be exactly UP x the pixel art, so layouts and hitboxes stay the same.
    let base = parse(px(path));
    if img.width as u32 != base.w as u32 * UP as u32 || img.height as u32 != base.h as u32 * UP as u32 {
        macroquad::logging::warn!("hd art {path}: {}x{} is not {}x the pixel art; using pixels", img.width, img.height, UP);
        return None;
    }
    Some(Sprite { w: img.width, h: img.height, rgba: img.bytes, hd: true })
}

/// A sprite by art path: the HD redraw when there is one, else the pixel source.
pub fn sprite(path: &str) -> Sprite {
    decode_hd(path).unwrap_or_else(|| parse(px(path)))
}

/// A high-resolution PNG of any size (art/hd/<path>.png, no pixel source): its art size is
/// its texel size / `UP`. Used by the generated (AI) sprites.
pub fn free_art(path: &str) -> Texture2D {
    let bytes = index::png(path).unwrap_or_else(|| panic!("missing art/hd/{path}.png"));
    let img = Image::from_file_with_format(bytes, Some(ImageFormat::Png)).expect("bad png");
    Sprite { w: img.width, h: img.height, rgba: img.bytes, hd: true }.texture()
}

/// The decoded RGBA image of art/hd/<path>.png (for CPU-side work like the terrain bake).
pub fn hd_image(path: &str) -> Image {
    let bytes = index::png(path).unwrap_or_else(|| panic!("missing art/hd/{path}.png"));
    Image::from_file_with_format(bytes, Some(ImageFormat::Png)).expect("bad png")
}

/// A hi-res generated sprite (art/hd/<path>.png), with mipmaps so it stays clean when drawn
/// smaller than its texture.
pub fn hi_art(path: &str) -> Texture2D {
    let t = free_art(path);
    mipmaps(&t);
    t
}

/// Trilinear filtering. Skipped on WebGL 1, which can't mipmap non-power-of-two textures.
fn mipmaps(t: &Texture2D) {
    use macroquad::miniquad::{FilterMode as F, MipmapFilterMode as M};
    let gl = unsafe { get_internal_gl() };
    if gl.quad_context.info().gl_version_string.contains("WebGL 1") {
        return;
    }
    let id = t.raw_miniquad_id();
    gl.quad_context.texture_generate_mipmaps(id);
    gl.quad_context.texture_set_min_filter(id, F::Linear, M::Linear);
}

/// A texture by art path.
pub fn art(path: &str) -> Texture2D {
    sprite(path).texture()
}

/// A texture and its outline silhouette, decoded once.
pub fn art_and_outline(path: &str) -> (Texture2D, Texture2D) {
    let s = sprite(path);
    (s.texture(), silhouette(&s))
}

/// The sprite's shape in flat white: tinted and offset behind it, it draws an outline. Faint
/// halo pixels in the art are dropped so the outline hugs the figure.
pub fn silhouette(s: &Sprite) -> Texture2D {
    const HALO: u8 = 48;
    let mut rgba = s.rgba.clone();
    rgba.chunks_exact_mut(4).for_each(|p| {
        p[..3].fill(255);
        if p[3] < HALO {
            p[3] = 0;
        }
    });
    Sprite { w: s.w, h: s.h, rgba, hd: s.hd }.texture()
}

/// A seamless ground tile by art path.
pub fn art_tile(path: &str) -> Texture2D {
    sprite(path).tile_texture()
}

/// Replaces palette entries (`<char> #rrggbb` lines) in a `.px` source.
pub fn recolor(src: &str, swaps: &[(char, &str)]) -> String {
    let mut out = String::with_capacity(src.len());
    let mut in_palette = true;
    for line in src.lines() {
        if line.trim_end() == "---" {
            in_palette = false;
        }
        let swap = if in_palette { swaps.iter().find(|(c, _)| line.starts_with(*c) && line[c.len_utf8()..].starts_with(' ')) } else { None };
        match swap {
            Some((c, hex)) => out.push_str(&format!("{c} {hex}")),
            None => out.push_str(line),
        }
        out.push('\n');
    }
    out
}

pub fn parse(src: &str) -> Sprite {
    let mut mirror = false;
    let mut palette: Vec<(char, [u8; 3])> = Vec::new();
    let mut lines = src.lines().map(str::trim_end).filter(|l| !l.starts_with('#') && !l.is_empty());
    for line in lines.by_ref() {
        if line == "---" {
            break;
        }
        if line == "mirror" {
            mirror = true;
            continue;
        }
        let (c, hex) = line.split_once(' ').expect("palette line: <char> #rrggbb");
        let hex = hex.trim().trim_start_matches('#');
        let v = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex colour");
        palette.push((c.chars().next().unwrap(), [v(0), v(2), v(4)]));
    }
    let rows: Vec<String> = lines
        .map(|l| if mirror { format!("{l}{}", l.chars().rev().collect::<String>()) } else { l.to_string() })
        .collect();
    let w = rows[0].chars().count();
    let h = rows.len();
    let mut rgba = vec![0u8; w * h * 4];
    for (y, row) in rows.iter().enumerate() {
        assert_eq!(row.chars().count(), w, "row {y} has wrong width");
        for (x, ch) in row.chars().enumerate() {
            if ch == '.' {
                continue;
            }
            let rgb = palette.iter().find(|(c, _)| *c == ch).unwrap_or_else(|| panic!("unknown palette char {ch:?}")).1;
            let o = (y * w + x) * 4;
            rgba[o..o + 3].copy_from_slice(&rgb);
            rgba[o + 3] = 255;
        }
    }
    Sprite { w: w as u16, h: h as u16, rgba, hd: false }
}

/// Every sprite texture is stored at this multiple of its art size, smoothed (Scale2x twice)
/// and linearly filtered, so pixel art reads as clean, rounded shapes instead of blocks.
/// Draw with `nat(t)` (the art size) as the destination size.
pub const UP: f32 = 4.0;

/// A texture's size in art pixels.
pub fn nat(t: &Texture2D) -> Vec2 {
    vec2(t.width(), t.height()) / UP
}

/// Scale2x (EPX): doubles a pixel image, rounding diagonal edges. `wrap` treats the
/// image as tiling (for ground tiles).
fn scale2x(w: usize, h: usize, src: &[u8], wrap: bool) -> Vec<u8> {
    let px = |x: isize, y: isize| -> [u8; 4] {
        let (x, y) = if wrap {
            (x.rem_euclid(w as isize) as usize, y.rem_euclid(h as isize) as usize)
        } else {
            (x.clamp(0, w as isize - 1) as usize, y.clamp(0, h as isize - 1) as usize)
        };
        let o = (y * w + x) * 4;
        [src[o], src[o + 1], src[o + 2], src[o + 3]]
    };
    let (w2, _h2) = (w * 2, h * 2);
    let mut out = vec![0u8; w * h * 16];
    for y in 0..h as isize {
        for x in 0..w as isize {
            let (e, b, d, f, hh) = (px(x, y), px(x, y - 1), px(x - 1, y), px(x + 1, y), px(x, y + 1));
            let e0 = if d == b && b != f && d != hh { d } else { e };
            let e1 = if b == f && b != d && f != hh { f } else { e };
            let e2 = if d == hh && d != b && hh != f { d } else { e };
            let e3 = if hh == f && d != hh && b != f { f } else { e };
            for (k, c) in [e0, e1, e2, e3].iter().enumerate() {
                let (ox, oy) = (x as usize * 2 + k % 2, y as usize * 2 + k / 2);
                let o = (oy * w2 + ox) * 4;
                out[o..o + 4].copy_from_slice(c);
            }
        }
    }
    out
}

/// Gives transparent pixels the colour of an opaque neighbour, so linear filtering
/// doesn't pull a dark fringe in from (0,0,0,0).
fn bleed(w: usize, h: usize, px: &mut [u8]) {
    let src = px.to_vec();
    for y in 0..h {
        for x in 0..w {
            let o = (y * w + x) * 4;
            if src[o + 3] != 0 {
                continue;
            }
            let (mut r, mut g, mut b, mut n) = (0u32, 0u32, 0u32, 0u32);
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let q = (ny as usize * w + nx as usize) * 4;
                if src[q + 3] != 0 {
                    r += src[q] as u32;
                    g += src[q + 1] as u32;
                    b += src[q + 2] as u32;
                    n += 1;
                }
            }
            if n > 0 {
                px[o] = (r / n) as u8;
                px[o + 1] = (g / n) as u8;
                px[o + 2] = (b / n) as u8;
            }
        }
    }
}

/// A light 3x3 (1-2-1) blur at 4x resolution: anti-aliases the remaining stair steps
/// without visibly blurring the art. Colours are premultiplied so edges don't darken.
fn soften(w: usize, h: usize, src: &[u8], wrap: bool) -> Vec<u8> {
    let at = |x: isize, y: isize| -> usize {
        let (x, y) = if wrap {
            (x.rem_euclid(w as isize) as usize, y.rem_euclid(h as isize) as usize)
        } else {
            (x.clamp(0, w as isize - 1) as usize, y.clamp(0, h as isize - 1) as usize)
        };
        (y * w + x) * 4
    };
    let mut out = vec![0u8; src.len()];
    const K: [f32; 3] = [1.0, 2.0, 1.0];
    for y in 0..h as isize {
        for x in 0..w as isize {
            let (mut r, mut g, mut b, mut a, mut wsum) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
            for dy in -1..=1isize {
                for dx in -1..=1isize {
                    let k = K[(dx + 1) as usize] * K[(dy + 1) as usize];
                    let o = at(x + dx, y + dy);
                    let al = src[o + 3] as f32 / 255.0;
                    r += src[o] as f32 * al * k;
                    g += src[o + 1] as f32 * al * k;
                    b += src[o + 2] as f32 * al * k;
                    a += al * k;
                    wsum += k;
                }
            }
            let o = at(x, y);
            let alpha = a / wsum;
            if a > 0.0 {
                out[o] = (r / a).min(255.0) as u8;
                out[o + 1] = (g / a).min(255.0) as u8;
                out[o + 2] = (b / a).min(255.0) as u8;
            }
            out[o + 3] = (alpha * 255.0).round() as u8;
        }
    }
    out
}

impl Sprite {
    fn upscaled(&self, wrap: bool) -> (u16, u16, Vec<u8>) {
        if self.hd {
            return (self.w, self.h, self.rgba.clone());
        }
        let (w, h) = (self.w as usize, self.h as usize);
        let a = scale2x(w, h, &self.rgba, wrap);
        let mut b = scale2x(w * 2, h * 2, &a, wrap);
        bleed(w * 4, h * 4, &mut b);
        let b = soften(w * 4, h * 4, &b, wrap);
        ((w * 4) as u16, (h * 4) as u16, b)
    }

    pub fn texture(&self) -> Texture2D {
        let (w, h, px) = self.upscaled(false);
        let t = Texture2D::from_rgba8(w, h, &px);
        t.set_filter(FilterMode::Linear);
        t
    }

    /// A seamless ground tile.
    pub fn tile_texture(&self) -> Texture2D {
        let (w, h, px) = self.upscaled(true);
        let t = Texture2D::from_rgba8(w, h, &px);
        t.set_filter(FilterMode::Linear);
        t
    }

    /// Copy with the pixels in columns `xs` and rows `ys` shifted up by one
    /// (used to lift a leg for walk frames). The bottom row of the region clears.
    pub fn lift(&self, xs: std::ops::Range<usize>, ys: std::ops::Range<usize>) -> Sprite {
        // Ranges are in art pixels; HD sprites shift by one art pixel (UP texels).
        let k = if self.hd { UP as usize } else { 1 };
        let (xs, ys) = (xs.start * k..xs.end * k, ys.start * k..ys.end * k);
        let w = self.w as usize;
        let mut rgba = self.rgba.clone();
        for y in ys.clone() {
            for x in xs.clone() {
                let dst = (y * w + x) * 4;
                if y + k < ys.end {
                    let src = ((y + k) * w + x) * 4;
                    let px: [u8; 4] = self.rgba[src..src + 4].try_into().unwrap();
                    rgba[dst..dst + 4].copy_from_slice(&px);
                } else {
                    rgba[dst..dst + 4].fill(0);
                }
            }
        }
        Sprite { w: self.w, h: self.h, rgba, hd: self.hd }
    }
}


