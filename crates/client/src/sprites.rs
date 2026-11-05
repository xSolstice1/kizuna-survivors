//! Parses `.px` sprite sources (see art/src) into RGBA textures.
//! Format: optional `mirror` line, palette lines `<char> #rrggbb`, `---`, then pixel rows.
//! `.` is transparent. With `mirror`, each row is the left half and is mirrored.

use macroquad::prelude::*;

pub struct Sprite {
    pub w: u16,
    pub h: u16,
    pub rgba: Vec<u8>,
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
    Sprite { w: w as u16, h: h as u16, rgba }
}

impl Sprite {
    pub fn texture(&self) -> Texture2D {
        let t = Texture2D::from_rgba8(self.w, self.h, &self.rgba);
        t.set_filter(FilterMode::Nearest);
        t
    }

    /// Copy with the pixels in columns `xs` and rows `ys` shifted up by one
    /// (used to lift a leg for walk frames). The bottom row of the region clears.
    pub fn lift(&self, xs: std::ops::Range<usize>, ys: std::ops::Range<usize>) -> Sprite {
        let w = self.w as usize;
        let mut rgba = self.rgba.clone();
        for y in ys.clone() {
            for x in xs.clone() {
                let dst = (y * w + x) * 4;
                if y + 1 < ys.end {
                    let src = ((y + 1) * w + x) * 4;
                    let px: [u8; 4] = self.rgba[src..src + 4].try_into().unwrap();
                    rgba[dst..dst + 4].copy_from_slice(&px);
                } else {
                    rgba[dst..dst + 4].fill(0);
                }
            }
        }
        Sprite { w: self.w, h: self.h, rgba }
    }
}

pub fn texture(src: &str) -> Texture2D {
    parse(src).texture()
}
