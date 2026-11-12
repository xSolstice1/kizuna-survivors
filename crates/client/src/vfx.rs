//! Visual effects kit: generated effect sprites (art/hd/vfx/*.png, from one sprite sheet, see
//! docs/art-pipeline.md) and an additive-blend material, so glows, sparks and trails add light
//! instead of painting flat shapes over the scene.
//!
//! Use: `vfx.additive(|| { vfx.draw("glow", c, 24.0, 0.0, tint); ... })` for light, or
//! `vfx.draw` outside it for solid things (petals, leaves, talismans).

use macroquad::miniquad::{BlendFactor, BlendState, Equation};
use macroquad::prelude::*;

use crate::sprites;

pub const NAMES: [&str; 32] = [
    "glow", "sparkle", "petal", "smoke", "slash", "ring", "circle", "lightning", "flame", "streak", "moon", "burst", "ember", "splash", "ofuda", "leaf",
    "bullet", "kunai", "vortex", "star", "seal", "spirit", "tendril", "foxtail", "beam", "bubble", "wind", "spike", "shield", "crack", "reticle", "feather",
];

const FRAGMENT: &str = r#"#version 100
precision lowp float;
varying vec2 uv;
varying vec4 color;
uniform sampler2D Texture;
void main() {
    vec4 t = texture2D(Texture, uv) * color;
    gl_FragColor = vec4(t.rgb * t.a, t.a);
}
"#;

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

#[derive(Clone)]
pub struct Vfx {
    tex: Vec<Texture2D>,
    add: Option<Material>,
}

impl Vfx {
    pub fn load() -> Self {
        let tex = NAMES.iter().map(|n| sprites::hi_art(&format!("vfx/{n}"))).collect();
        // Additive: the source (premultiplied in the shader) is added onto what's there.
        let add = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::One)),
                    alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One)),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .map_err(|e| {
            macroquad::logging::warn!("vfx additive material: {e:?}");
        })
        .ok();
        Vfx { tex, add }
    }

    pub fn tex(&self, name: &str) -> &Texture2D {
        &self.tex[NAMES.iter().position(|n| *n == name).unwrap_or_else(|| panic!("no vfx sprite {name}"))]
    }

    /// Draws `f` with additive blending (falls back to normal blending if the material failed).
    pub fn additive(&self, f: impl FnOnce()) {
        if let Some(m) = &self.add {
            gl_use_material(m);
            f();
            gl_use_default_material();
        } else {
            f();
        }
    }

    /// An effect sprite centred at `c`, `size` world units across, turned by `rot` radians.
    pub fn draw(&self, name: &str, c: Vec2, size: f32, rot: f32, tint: Color) {
        self.draw_wh(name, c, vec2(size, size), rot, tint);
    }

    /// Like `draw`, stretched to `wh` (e.g. a streak along a velocity).
    pub fn draw_wh(&self, name: &str, c: Vec2, wh: Vec2, rot: f32, tint: Color) {
        draw_texture_ex(self.tex(name), c.x - wh.x / 2.0, c.y - wh.y / 2.0, tint, DrawTextureParams { dest_size: Some(wh), rotation: rot, ..Default::default() });
    }
}

thread_local! {
    static KIT: std::cell::OnceCell<Vfx> = const { std::cell::OnceCell::new() };
}

/// Makes the kit available to `shape` (called once, when the assets load).
pub fn install(v: &Vfx) {
    KIT.with(|k| {
        let _ = k.set(v.clone());
    });
}

/// Drop-in replacements for macroquad's shape calls (same arguments), drawn with the effect
/// sprites instead of flat geometry, so every effect in the world is art rather than code
/// shapes. Solid circles become a bright-cored orb, faint ones a soft glow; outlines become the
/// shockwave ring; lines a beam; small squares embers; triangles a spike aimed at the tip;
/// polygons a star or sparkle; dark ellipses soft shadows.
pub mod shape {
    use super::KIT;
    use macroquad::prelude::*;

    fn spr(name: &str, c: Vec2, wh: Vec2, rot: f32, tint: Color) {
        KIT.with(|k| {
            if let Some(v) = k.get() {
                v.draw_wh(name, c, wh, rot, tint);
            }
        });
    }

    fn luma(c: Color) -> f32 {
        0.3 * c.r + 0.59 * c.g + 0.11 * c.b
    }

    pub fn circle(x: f32, y: f32, r: f32, c: Color) {
        if c.a >= 0.7 {
            spr("bullet", vec2(x, y), Vec2::splat(r * 2.3), 0.0, c);
        } else {
            spr("glow", vec2(x, y), Vec2::splat(r * 2.6), 0.0, Color::new(c.r, c.g, c.b, (c.a * 1.7).min(1.0)));
        }
    }

    pub fn circle_lines(x: f32, y: f32, r: f32, thickness: f32, c: Color) {
        let k = if thickness >= 4.0 { 1.0 } else { 0.8 };
        spr("ring", vec2(x, y), Vec2::splat(r * 2.25), 0.0, Color::new(c.r, c.g, c.b, (c.a * k * 1.3).min(1.0)));
    }

    pub fn line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, c: Color) {
        let d = vec2(x2 - x1, y2 - y1);
        let len = d.length();
        if len < 0.01 {
            return;
        }
        spr("beam", vec2(x1 + x2, y1 + y2) * 0.5, vec2(len + thickness * 2.0, thickness * 3.2), d.y.atan2(d.x), c);
    }

    pub fn rectangle(x: f32, y: f32, w: f32, h: f32, c: Color) {
        let mid = vec2(x + w / 2.0, y + h / 2.0);
        if w.max(h) <= 4.0 {
            spr("ember", mid, Vec2::splat(w.max(h) * 3.0), 0.0, c);
        } else if h > w {
            spr("beam", mid, vec2(h, w * 2.4), std::f32::consts::FRAC_PI_2, c);
        } else {
            spr("beam", mid, vec2(w, h * 2.4), 0.0, c);
        }
    }

    pub fn triangle(a: Vec2, b: Vec2, t: Vec2, c: Color) {
        // The tip is the vertex farthest from the other two; the spike stands on their middle.
        let pts = [a, b, t];
        let far = (0..3).max_by(|&i, &j| {
            let s = |k: usize| (pts[k] - pts[(k + 1) % 3]).length() + (pts[k] - pts[(k + 2) % 3]).length();
            s(i).partial_cmp(&s(j)).unwrap()
        }).unwrap();
        let tip = pts[far];
        let (p, q) = (pts[(far + 1) % 3], pts[(far + 2) % 3]);
        let base = (p + q) * 0.5;
        let axis = tip - base;
        let len = axis.length();
        if len < 0.01 {
            return;
        }
        let rot = axis.y.atan2(axis.x) + std::f32::consts::FRAC_PI_2;
        spr("spike", (tip + base) * 0.5, vec2((p - q).length() * 1.7, len * 1.25), rot, c);
    }

    /// A moon orb of radius `r`: halo, then the orb turning by `rot`.
    pub fn moon_orb(c: Vec2, r: f32, rot: f32) {
        spr("glow", c, Vec2::splat(r * 4.0), 0.0, Color::new(1.0, 0.9, 0.6, 0.45));
        spr("moon", c, Vec2::splat(r * 2.4), rot * 0.3, WHITE);
    }

    pub fn poly(x: f32, y: f32, sides: u8, r: f32, rotation: f32, c: Color) {
        spr(if sides == 5 { "star" } else { "sparkle" }, vec2(x, y), Vec2::splat(r * 2.8), rotation.to_radians(), c);
    }

    pub fn poly_lines(x: f32, y: f32, _sides: u8, r: f32, _rotation: f32, thickness: f32, c: Color) {
        circle_lines(x, y, r, thickness, c);
    }

    pub fn ellipse(x: f32, y: f32, w: f32, h: f32, rotation: f32, c: Color) {
        let rot = rotation.to_radians();
        if luma(c) < 0.15 {
            // A soft shadow.
            spr("glow", vec2(x, y), vec2(w * 2.6, h * 2.6), rot, Color::new(0.0, 0.0, 0.0, (c.a * 1.5).min(1.0)));
        } else if w.max(h) <= 5.0 {
            spr("petal", vec2(x, y), vec2(w * 3.0, h * 3.0), rot, c);
        } else {
            spr("glow", vec2(x, y), vec2(w * 2.6, h * 2.6), rot, c);
        }
    }
}
