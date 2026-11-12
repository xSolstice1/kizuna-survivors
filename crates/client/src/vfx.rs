//! Visual effects kit: generated effect sprites (art/hd/vfx/*.png, from one sprite sheet, see
//! docs/art-pipeline.md) and an additive-blend material, so glows, sparks and trails add light
//! instead of painting flat shapes over the scene.
//!
//! Use: `vfx.additive(|| { vfx.draw("glow", c, 24.0, 0.0, tint); ... })` for light, or
//! `vfx.draw` outside it for solid things (petals, leaves, talismans).

use macroquad::miniquad::{BlendFactor, BlendState, Equation};
use macroquad::prelude::*;

use crate::sprites;

pub const NAMES: [&str; 16] = ["glow", "sparkle", "petal", "smoke", "slash", "ring", "circle", "lightning", "flame", "streak", "moon", "burst", "ember", "splash", "ofuda", "leaf"];

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
