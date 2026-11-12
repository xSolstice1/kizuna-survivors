//! Stage art: tiled ground, scattered props with glows, weather and a vignette.
//! The prologue (Neon Shotengai) has its own tiles; each campaign world has four ground
//! tiles and four props (A tall light, B medium, C wide, D small) in `art/src/wN/`.
//! The prop layout lives in `sim::stage` (it is also used for collision).

use macroquad::prelude::*;

use crate::render::world_scale;
use crate::sprites;

use sim::campaign::StageId;
use sim::stage::{hash, PropKind, CHUNK, PATH_EVERY, TILE};

/// Per world: ground_a, ground_b, decal, path, prop_a, prop_b, prop_c, prop_d.
type WorldSet = [Texture2D; 8];

pub struct StageArt {
    tex: Vec<(&'static str, Texture2D)>,
    worlds: Vec<WorldSet>,
    /// Baked terrain chunks for the current campaign world (made on first use).
    baker: std::cell::RefCell<Option<crate::terrain_bake::Baker>>,
    vignette: Texture2D,
    /// Soft white radial falloff, tinted per light.
    glow: Texture2D,
    /// Black with a clear centre: the downpour's limited sight.
    dark: Texture2D,
}

fn rgb(hex: u32, a: f32) -> Color {
    Color::new(((hex >> 16) & 255) as f32 / 255.0, ((hex >> 8) & 255) as f32 / 255.0, (hex & 255) as f32 / 255.0, a)
}

/// Enemy bullet colours by shot style (0 = prologue fireball, otherwise the stage index).
pub fn bullet_color(style: u8) -> Color {
    match style {
        1 => rgb(0xff6ab8, 1.0),
        2 => rgb(0x9af4ff, 1.0),
        3 => rgb(0xff6ad5, 1.0),
        4 => rgb(0xc8f03a, 1.0),
        5 => rgb(0xff5ad0, 1.0),
        _ => Color::new(1.0, 0.45, 0.2, 1.0),
    }
}

/// The stage's accent (boss auras, map cards, minimap tint).
pub fn accent(stage: StageId) -> Color {
    match stage {
        StageId::Shotengai => rgb(0xff9a4a, 1.0),
        StageId::ShrineValley => rgb(0xff8ac8, 1.0),
        StageId::EmberLeaf => rgb(0xff8a2a, 1.0),
        StageId::PaleCitadel => rgb(0x8fd0ff, 1.0),
        StageId::Rainward => rgb(0xff4a58, 1.0),
        StageId::Shatterline => rgb(0xb050ff, 1.0),
    }
}

/// Dominant ground colour (minimap background, card backdrop).
pub fn ground_color(stage: StageId) -> Color {
    match stage {
        StageId::Shotengai => rgb(0x2b2840, 1.0),
        StageId::ShrineValley => rgb(0x1a2830, 1.0),
        StageId::EmberLeaf => rgb(0x2a1e1c, 1.0),
        StageId::PaleCitadel => rgb(0x2a3041, 1.0),
        StageId::Rainward => rgb(0x1d2129, 1.0),
        StageId::Shatterline => rgb(0x1e1a26, 1.0),
    }
}

/// Glow of each world prop (A, B, C, D) and of the decal tile.
fn world_glows(stage: StageId) -> ([Option<Color>; 4], Option<Color>) {
    match stage {
        StageId::ShrineValley => ([Some(rgb(0xffcf7a, 0.14)), Some(rgb(0xffd27a, 0.06)), None, None], Some(rgb(0xff8ac8, 0.05))),
        StageId::EmberLeaf => ([Some(rgb(0xff8a2a, 0.15)), None, Some(rgb(0xff6a1a, 0.06)), Some(rgb(0xff6a1a, 0.08))], Some(rgb(0xff8a2a, 0.10))),
        StageId::PaleCitadel => ([Some(rgb(0x8fd0ff, 0.15)), Some(rgb(0x4f9be0, 0.06)), None, None], Some(rgb(0x4f9be0, 0.10))),
        StageId::Rainward => ([Some(rgb(0xffb347, 0.16)), None, None, None], Some(rgb(0xc8323c, 0.08))),
        StageId::Shatterline => ([Some(rgb(0xff4a3a, 0.14)), None, None, Some(rgb(0xb050ff, 0.10))], Some(rgb(0xb050ff, 0.05))),
        StageId::Shotengai => ([None; 4], None),
    }
}

fn radial(n: u16, rgb: [u8; 3], alpha: impl Fn(f32) -> f32) -> Texture2D {
    let mut px = vec![0u8; n as usize * n as usize * 4];
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = ((x as f32 + 0.5) / n as f32 - 0.5, (y as f32 + 0.5) / n as f32 - 0.5);
            let d = (dx * dx + dy * dy).sqrt() * 2.0;
            let i = (y as usize * n as usize + x as usize) * 4;
            px[i..i + 3].copy_from_slice(&rgb);
            px[i + 3] = (alpha(d).clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    Texture2D::from_rgba8(n, n, &px)
}

impl StageArt {
    pub fn load() -> Self {
        let tex = vec![
        ("crates", sprites::art("stage/crates")),
        ("lantern", sprites::art("stage/lantern")),
        ("manhole", sprites::art_tile("stage/manhole")),
        ("neon_sign", sprites::art("stage/neon_sign")),
        ("paving_a", sprites::art_tile("stage/paving_a")),
        ("paving_b", sprites::art_tile("stage/paving_b")),
        ("puddle", sprites::art_tile("stage/puddle")),
        ("stall", sprites::art("stage/stall")),
        ("tactile", sprites::art_tile("stage/tactile")),
        ("vending", sprites::art("stage/vending")),
        ];
        let worlds = vec![
            [
                sprites::art_tile("w1/ground_a"),
                sprites::art_tile("w1/ground_b"),
                sprites::art_tile("w1/decal"),
                sprites::art_tile("w1/path"),
                sprites::art("w1/prop_a"),
                sprites::art("w1/prop_b"),
                sprites::art("w1/prop_c"),
                sprites::art("w1/prop_d"),
            ],
            [
                sprites::art_tile("w2/ground_a"),
                sprites::art_tile("w2/ground_b"),
                sprites::art_tile("w2/decal"),
                sprites::art_tile("w2/path"),
                sprites::art("w2/prop_a"),
                sprites::art("w2/prop_b"),
                sprites::art("w2/prop_c"),
                sprites::art("w2/prop_d"),
            ],
            [
                sprites::art_tile("w3/ground_a"),
                sprites::art_tile("w3/ground_b"),
                sprites::art_tile("w3/decal"),
                sprites::art_tile("w3/path"),
                sprites::art("w3/prop_a"),
                sprites::art("w3/prop_b"),
                sprites::art("w3/prop_c"),
                sprites::art("w3/prop_d"),
            ],
            [
                sprites::art_tile("w4/ground_a"),
                sprites::art_tile("w4/ground_b"),
                sprites::art_tile("w4/decal"),
                sprites::art_tile("w4/path"),
                sprites::art("w4/prop_a"),
                sprites::art("w4/prop_b"),
                sprites::art("w4/prop_c"),
                sprites::art("w4/prop_d"),
            ],
            [
                sprites::art_tile("w5/ground_a"),
                sprites::art_tile("w5/ground_b"),
                sprites::art_tile("w5/decal"),
                sprites::art_tile("w5/path"),
                sprites::art("w5/prop_a"),
                sprites::art("w5/prop_b"),
                sprites::art("w5/prop_c"),
                sprites::art("w5/prop_d"),
            ]
        ];
        let vignette = radial(128, [8, 4, 18], |d| {
            let v = (d - 0.45).clamp(0.0, 1.0);
            v * v * 0.78
        });
        let glow = radial(64, [255, 255, 255], |d| (1.0 - d).max(0.0).powi(2));
        let dark = radial(128, [2, 3, 8], |d| ((d - 0.22) / 0.4).clamp(0.0, 1.0) * 0.86);
        StageArt { tex, worlds, baker: std::cell::RefCell::new(None), vignette, glow, dark }
    }

    fn get(&self, name: &str) -> &Texture2D {
        &self.tex.iter().find(|(n, _)| *n == name).expect("stage sprite").1
    }

    /// A world's tile (0..4) or prop (4..8) sprite.
    pub fn world_tex(&self, stage: StageId, slot: usize) -> Option<&Texture2D> {
        let i = stage.index();
        (i > 0).then(|| &self.worlds[i - 1][slot])
    }

    /// Ground tile at a tile coordinate.
    fn tile(&self, stage: StageId, tx: i32, ty: i32) -> (&Texture2D, bool) {
        let path = tx.rem_euclid(PATH_EVERY) == 0;
        let roll = if stage == StageId::Shotengai { hash(tx, ty, 1) % 200 } else { (hash(tx, ty, 41) >> 9) % 200 };
        match self.world_tex(stage, 0) {
            None => {
                let name = if path {
                    "tactile"
                } else {
                    match roll {
                        0 => "manhole",
                        1..=2 => "puddle",
                        3..=100 => "paving_a",
                        _ => "paving_b",
                    }
                };
                (self.get(name), false)
            }
            Some(_) => {
                let w = &self.worlds[stage.index() - 1];
                if path {
                    (&w[3], false)
                } else {
                    match roll {
                        0..=2 => (&w[2], true),
                        3..=110 => (&w[0], false),
                        _ => (&w[1], false),
                    }
                }
            }
        }
    }

    /// Draws a small patch of a stage's ground (map-select cards).
    pub fn draw_patch(&self, stage: StageId, x: f32, y: f32, w: f32, h: f32, scale: f32) {
        let s = TILE * scale;
        let (cols, rows) = ((w / s).ceil() as i32, (h / s).ceil() as i32);
        for ty in 0..rows {
            for tx in 0..cols {
                let (t, _) = self.tile(stage, tx + 3, ty + 5);
                let (dw, dh) = (s.min(w - tx as f32 * s), s.min(h - ty as f32 * s));
                let src = Rect::new(0.5, 0.5, TILE * dw / s * crate::sprites::UP - 1.0, TILE * dh / s * crate::sprites::UP - 1.0);
                draw_texture_ex(t, x + tx as f32 * s, y + ty as f32 * s, WHITE, DrawTextureParams { dest_size: Some(vec2(dw, dh)), source: Some(src), ..Default::default() });
            }
        }
    }

    /// Ground tiles, props and their glows (everything under the characters).
    pub fn draw_ground(&self, stage: StageId, terrain: &sim::terrain::Terrain, cam: &Camera2D, time: f64) {
        let (sw, sh) = (screen_width() / world_scale(), screen_height() / world_scale());
        let (x0, y0) = (cam.target.x - sw / 2.0, cam.target.y - sh / 2.0);
        if terrain.on {
            let mut baker = self.baker.borrow_mut();
            if baker.as_ref().is_none_or(|b| b.stage() != stage) {
                *baker = Some(crate::terrain_bake::Baker::new(stage));
            }
            let b = baker.as_mut().unwrap();
            let view = Rect::new(x0, y0, sw, sh);
            b.update(cam.target, view);
            b.draw(view);
        }
        let (tx0, ty0) = ((x0 / TILE).floor() as i32, (y0 / TILE).floor() as i32);
        let (prop_glows, decal_glow) = world_glows(stage);
        let mut decals = Vec::new();
        for ty in (ty0..=ty0 + (sh / TILE) as i32 + 1).filter(|_| !terrain.on) {
            for tx in tx0..=tx0 + (sw / TILE) as i32 + 1 {
                let (t, decal) = self.tile(stage, tx, ty);
                // Inset half a texel: tiles live in the shared atlas, and linear filtering must not
                // sample a neighbour (that shows as seams).
                draw_texture_ex(t, tx as f32 * TILE, ty as f32 * TILE, WHITE, DrawTextureParams { dest_size: Some(vec2(TILE, TILE)), source: Some(Rect::new(0.5, 0.5, t.width() - 1.0, t.height() - 1.0)), ..Default::default() });
                if decal {
                    decals.push(vec2(tx as f32 * TILE + TILE / 2.0, ty as f32 * TILE + TILE / 2.0));
                }
            }
        }
        if let Some(g) = decal_glow {
            for d in decals {
                let pulse = 0.8 + 0.2 * ((time * 2.0) as f32 + d.x * 0.07).sin();
                let r = 40.0;
                draw_texture_ex(&self.glow, d.x - r, d.y - r, Color::new(g.r, g.g, g.b, (g.a * 3.0 * pulse).min(1.0)), DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() });
            }
        }

        // Props: up to two per chunk, never on the path (layout from the sim).
        let margin = 40.0;
        let (cx0, cy0) = (((x0 - margin) / CHUNK).floor() as i32, ((y0 - margin) / CHUNK).floor() as i32);
        let (cx1, cy1) = (((x0 + sw + margin) / CHUNK).floor() as i32, ((y0 + sh + margin) / CHUNK).floor() as i32);
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let (props, n) = terrain.chunk_props(cx, cy);
                for prop in &props[..n] {
                    let (px, py) = (prop.pos.x, prop.pos.y);
                    let (t, glow) = match prop.kind {
                        PropKind::Lantern => (self.get("lantern"), Some(Color::new(1.0, 0.55, 0.3, 0.13))),
                        PropKind::Vending => (self.get("vending"), Some(Color::new(0.4, 0.75, 1.0, 0.12))),
                        PropKind::Stall => (self.get("stall"), Some(Color::new(1.0, 0.8, 0.4, 0.08))),
                        PropKind::NeonSign => (self.get("neon_sign"), Some(Color::new(1.0, 0.35, 0.7, 0.12))),
                        PropKind::Crates => (self.get("crates"), None),
                        k => {
                            let slot = match k {
                                PropKind::A => 0,
                                PropKind::B => 1,
                                PropKind::C => 2,
                                _ => 3,
                            };
                            match self.world_tex(stage, 4 + slot) {
                                Some(t) => (t, prop_glows[slot]),
                                None => continue,
                            }
                        }
                    };
                    if let Some(mut g) = glow {
                        let flicker = 0.85 + 0.15 * ((time * 3.0) as f32 + px * 0.13).sin();
                        g.a = (g.a * 3.2 * flicker).min(1.0);
                        let r = 64.0;
                        let lift = if prop.kind == PropKind::A { crate::sprites::nat(t).y - 8.0 } else { 6.0 };
                        draw_texture_ex(&self.glow, px - r, py - r - lift, g, DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() });
                    }
                    // Contact shadow under the solid footprint.
                    let (min, max) = prop.footprint();
                    crate::vfx::shape::ellipse((min.x + max.x) / 2.0, max.y - 0.5, (max.x - min.x) / 2.0 + 2.0, 3.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.4));
                    let n = crate::sprites::nat(t);
                    draw_texture_ex(t, (px - n.x / 2.0).round(), (py - n.y).round() + 6.0, WHITE, DrawTextureParams { dest_size: Some(n), ..Default::default() });
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

    /// Rainward 13's downpour: sight shrinks to a pool around the player, rain streaks
    /// across the screen, and lightning flashes it white. `centre` is in screen pixels.
    pub fn draw_downpour(&self, centre: Vec2, time: f64, flash: f32) {
        let (w, h) = (screen_width(), screen_height());
        let r = w.max(h) * 0.8;
        draw_texture_ex(&self.dark, centre.x - r, centre.y - r, WHITE, DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() });
        // Fill beyond the texture so the corners stay dark.
        let c = Color::new(2.0 / 255.0, 3.0 / 255.0, 8.0 / 255.0, 0.86);
        draw_rectangle(0.0, 0.0, w, (centre.y - r).max(0.0), c);
        draw_rectangle(0.0, centre.y + r, w, (h - centre.y - r).max(0.0), c);
        let t = time as f32;
        let streak = Color::new(0.65, 0.75, 0.95, 0.22);
        let len = h * 0.035;
        for k in 0..140 {
            let seed = hash(k, 7, 3);
            let sx = (seed % 1000) as f32 / 1000.0;
            let speed = 0.9 + (seed >> 10) as f32 % 100.0 / 200.0;
            let y = ((t * speed + (seed >> 20) as f32 / 4096.0) % 1.0) * (h + len) - len;
            let x = (sx * (w + 80.0) - 40.0 + y * 0.18) % (w + 80.0);
            crate::vfx::shape::line(x, y, x - len * 0.18, y + len, 1.5, streak);
        }
        if flash > 0.0 {
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.85, 0.9, 1.0, flash.min(1.0) * 0.55));
        }
    }
}
