//! Stage 1, Neon Shotengai: tiled night street, scattered props, glows and a vignette.
//! Layout is a pure function of world coordinates, so it is infinite and identical every run.

use macroquad::prelude::*;

use crate::render::SCALE;
use crate::sprites;

const TILE: f32 = 32.0;
const CHUNK: f32 = 192.0;

pub struct StageArt {
    tex: Vec<(&'static str, Texture2D)>,
    vignette: Texture2D,
    /// Soft white radial falloff, tinted per light.
    glow: Texture2D,
}

fn hash(x: i32, y: i32, salt: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}

impl StageArt {
    pub fn load() -> Self {
        let tex = vec![
        ("crates", sprites::texture(include_str!("../../../art/src/stage/crates.px"))),
        ("lantern", sprites::texture(include_str!("../../../art/src/stage/lantern.px"))),
        ("manhole", sprites::texture(include_str!("../../../art/src/stage/manhole.px"))),
        ("neon_sign", sprites::texture(include_str!("../../../art/src/stage/neon_sign.px"))),
        ("paving_a", sprites::texture(include_str!("../../../art/src/stage/paving_a.px"))),
        ("paving_b", sprites::texture(include_str!("../../../art/src/stage/paving_b.px"))),
        ("puddle", sprites::texture(include_str!("../../../art/src/stage/puddle.px"))),
        ("stall", sprites::texture(include_str!("../../../art/src/stage/stall.px"))),
        ("tactile", sprites::texture(include_str!("../../../art/src/stage/tactile.px"))),
        ("vending", sprites::texture(include_str!("../../../art/src/stage/vending.px"))),
        ];
        // Radial vignette: clear centre, dark edges.
        let n = 128u16;
        let mut rgba = vec![0u8; n as usize * n as usize * 4];
        for y in 0..n {
            for x in 0..n {
                let (dx, dy) = ((x as f32 + 0.5) / n as f32 - 0.5, (y as f32 + 0.5) / n as f32 - 0.5);
                let d = ((dx * dx + dy * dy).sqrt() * 2.0 - 0.45).clamp(0.0, 1.0);
                let i = (y as usize * n as usize + x as usize) * 4;
                rgba[i..i + 3].copy_from_slice(&[8, 4, 18]);
                rgba[i + 3] = (d * d * 200.0) as u8;
            }
        }
        let vignette = Texture2D::from_rgba8(n, n, &rgba);
        let g = 64u16;
        let mut glow = vec![0u8; g as usize * g as usize * 4];
        for y in 0..g {
            for x in 0..g {
                let (dx, dy) = ((x as f32 + 0.5) / g as f32 - 0.5, (y as f32 + 0.5) / g as f32 - 0.5);
                let f = (1.0 - (dx * dx + dy * dy).sqrt() * 2.0).clamp(0.0, 1.0);
                let i = (y as usize * g as usize + x as usize) * 4;
                glow[i..i + 3].copy_from_slice(&[255, 255, 255]);
                glow[i + 3] = (f * f * 255.0) as u8;
            }
        }
        StageArt { tex, vignette, glow: Texture2D::from_rgba8(g, g, &glow) }
    }

    fn get(&self, name: &str) -> &Texture2D {
        &self.tex.iter().find(|(n, _)| *n == name).expect("stage sprite").1
    }

    /// Ground tiles, props and their glows (everything under the characters).
    pub fn draw_ground(&self, cam: &Camera2D, time: f64) {
        let (sw, sh) = (screen_width() / SCALE, screen_height() / SCALE);
        let (x0, y0) = (cam.target.x - sw / 2.0, cam.target.y - sh / 2.0);
        let (tx0, ty0) = ((x0 / TILE).floor() as i32, (y0 / TILE).floor() as i32);
        for ty in ty0..=ty0 + (sh / TILE) as i32 + 1 {
            for tx in tx0..=tx0 + (sw / TILE) as i32 + 1 {
                let name = if tx.rem_euclid(14) == 0 {
                    "tactile"
                } else {
                    match hash(tx, ty, 1) % 200 {
                        0 => "manhole",
                        1..=2 => "puddle",
                        3..=100 => "paving_a",
                        _ => "paving_b",
                    }
                };
                draw_texture(self.get(name), tx as f32 * TILE, ty as f32 * TILE, WHITE);
            }
        }

        // Props: up to two per chunk, never on the tactile path.
        let margin = 40.0;
        let (cx0, cy0) = (((x0 - margin) / CHUNK).floor() as i32, ((y0 - margin) / CHUNK).floor() as i32);
        let (cx1, cy1) = (((x0 + sw + margin) / CHUNK).floor() as i32, ((y0 + sh + margin) / CHUNK).floor() as i32);
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let h = hash(cx, cy, 2);
                let mut placed: Vec<Vec2> = Vec::new();
                for k in 0..(h % 3) {
                    let hk = hash(cx, cy, 10 + k);
                    let px = cx as f32 * CHUNK + (hk % 160) as f32 + 16.0;
                    let py = cy as f32 * CHUNK + ((hk >> 8) % 160) as f32 + 16.0;
                    let path_x = ((px / (TILE * 14.0)).round() * TILE * 14.0) + TILE / 2.0;
                    if (px - path_x).abs() < 40.0 || placed.iter().any(|q| (*q - vec2(px, py)).length() < 56.0) {
                        continue;
                    }
                    placed.push(vec2(px, py));
                    let (name, glow) = match (hk >> 16) % 10 {
                        0..=2 => ("lantern", Some(Color::new(1.0, 0.55, 0.3, 0.13))),
                        3 => ("vending", Some(Color::new(0.4, 0.75, 1.0, 0.12))),
                        4..=5 => ("stall", Some(Color::new(1.0, 0.8, 0.4, 0.08))),
                        6..=7 => ("neon_sign", Some(Color::new(1.0, 0.35, 0.7, 0.12))),
                        _ => ("crates", None),
                    };
                    if let Some(mut g) = glow {
                        let flicker = 0.85 + 0.15 * ((time * 3.0) as f32 + px * 0.13).sin();
                        g.a = (g.a * 3.2 * flicker).min(1.0);
                        let r = 64.0;
                        draw_texture_ex(&self.glow, px - r, py - r - 6.0, g, DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() });
                    }
                    let t = self.get(name);
                    draw_texture(t, (px - t.width() / 2.0).round(), (py - t.height()).round() + 6.0, WHITE);
                }
            }
        }
    }

    /// Screen-space vignette, drawn after the world and before the UI.
    pub fn draw_vignette(&self) {
        draw_texture_ex(
            &self.vignette,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(screen_width(), screen_height())), ..Default::default() },
        );
    }
}
