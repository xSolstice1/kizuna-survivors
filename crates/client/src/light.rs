//! Light and shadow for the world: one moon (high in the upper left) casts every shadow the
//! same way, and a light map darkens the night while lanterns, fire, spells and pickups throw
//! warm pools of light.
//!
//! - `Shadows`: each sprite's own texture, tinted black and slanted away from the moon onto the
//!   ground, so a shadow has its sprite's shape. Batched per texture into one mesh.
//! - `Lights`: a quarter-resolution light map. It starts at the stage's night ambient and
//!   lights are added on top; it then multiplies the world drawn so far, doubled (0.5 = as
//!   drawn, lower darker, higher brighter), so lit pools glow instead of only being less dark.
//!   `light_at` gives the same light on the CPU, for what is drawn after it (the hero).
use std::cell::RefCell;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;
use sim::campaign::StageId;

/// Where the top of a sprite's shadow falls, per unit of sprite height, from its feet: down and
/// to the right, away from a moon in the upper left, flattened onto the ground.
pub const SKEW: Vec2 = Vec2::new(0.62, 0.30);
/// How dark a full shadow is.
pub const SHADOW_ALPHA: f32 = 0.30;

/// Cast shadows for one draw pass, flushed (one mesh per texture) before the sprites.
pub struct Shadows<'a> {
    quads: Vec<(&'a Texture2D, [Vec2; 4], bool, f32)>,
}

impl<'a> Shadows<'a> {
    pub fn new() -> Self {
        Shadows { quads: Vec::new() }
    }

    /// A `w` x `h` sprite standing at `feet` (the ground point under it), mirrored if `flip`.
    /// `alpha` scales the darkness (a sprite floating off the ground casts a fainter one).
    pub fn push(&mut self, tex: &'a Texture2D, feet: Vec2, w: f32, h: f32, flip: bool, alpha: f32) {
        if alpha <= 0.01 {
            return;
        }
        let top = feet + SKEW * h;
        let half = vec2(w / 2.0, 0.0);
        // Texture top (v = 0) lies away from the light, its feet (v = 1) at the sprite's feet.
        self.quads.push((tex, [top - half, top + half, feet + half, feet - half], flip, alpha));
    }

    pub fn flush(&mut self) {
        self.quads.sort_by_key(|q| q.0 as *const Texture2D as usize);
        let mut i = 0;
        while i < self.quads.len() {
            let tex = self.quads[i].0;
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            while i < self.quads.len() && std::ptr::eq(self.quads[i].0, tex) && vertices.len() < 60000 {
                let (_, c, flip, a) = self.quads[i];
                let (u0, u1) = if flip { (1.0, 0.0) } else { (0.0, 1.0) };
                let col = Color::new(0.0, 0.0, 0.0, SHADOW_ALPHA * a);
                let b = vertices.len() as u16;
                for (p, (u, v)) in c.iter().zip([(u0, 0.0), (u1, 0.0), (u1, 1.0), (u0, 1.0)]) {
                    vertices.push(Vertex::new(p.x, p.y, 0.0, u, v, col));
                }
                indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
                i += 1;
            }
            draw_mesh(&Mesh { vertices, indices, texture: Some(tex.clone()) });
        }
        self.quads.clear();
    }
}

/// The night's base light per world (light-map units: 0.5 = the art as painted).
pub fn ambient(stage: StageId) -> Color {
    match stage {
        StageId::ShrineValley => Color::new(0.36, 0.38, 0.47, 1.0),
        StageId::Shotengai => Color::new(0.38, 0.36, 0.45, 1.0),
        StageId::EmberLeaf => Color::new(0.40, 0.35, 0.36, 1.0),
        StageId::PaleCitadel => Color::new(0.40, 0.41, 0.48, 1.0),
        StageId::Rainward => Color::new(0.35, 0.38, 0.45, 1.0),
        StageId::Shatterline => Color::new(0.37, 0.35, 0.47, 1.0),
    }
}

/// A light: where, how far it reaches (world units) and its colour added at the centre.
#[derive(Clone, Copy)]
pub struct Light {
    pub pos: Vec2,
    pub radius: f32,
    pub color: Color,
}

/// The light map and its materials (one per thread: the game draws on one).
pub struct Lights {
    rt: Option<(RenderTarget, u32, u32)>,
    multiply: Option<Material>,
    pub list: Vec<Light>,
    ambient: Color,
}

thread_local! {
    pub static LIGHTS: RefCell<Lights> = RefCell::new(Lights { rt: None, multiply: None, list: Vec::new(), ambient: WHITE });
}

const MUL_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec4 color;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;
const MUL_FRAGMENT: &str = r#"#version 100
precision lowp float;
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    // At most 1.4x brighter: pools glow, crowds of lights never wash the scene out.
    gl_FragColor = vec4(min(texture2D(Texture, uv).rgb, vec3(0.7)), 1.0);
}
"#;

/// Light falloff at `d` from a light of reach `r`: soft, like the glow sprite drawn for it.
fn falloff(d: f32, r: f32) -> f32 {
    let k = (1.0 - (d / r).powi(2)).max(0.0);
    k * k
}

impl Lights {
    /// Starts a frame's lights over the stage's ambient.
    pub fn begin(&mut self, stage: StageId) {
        self.list.clear();
        self.ambient = ambient(stage);
    }

    pub fn add(&mut self, pos: Vec2, radius: f32, color: Color) {
        if self.list.len() < 600 {
            self.list.push(Light { pos, radius, color });
        }
    }

    /// The light at `p` as a tint for a sprite drawn after the map (its doubled light, at
    /// most the art's own colour).
    pub fn light_at(&self, p: Vec2) -> Color {
        let mut c = self.ambient;
        for l in &self.list {
            let k = falloff((l.pos - p).length(), l.radius);
            c.r += l.color.r * k;
            c.g += l.color.g * k;
            c.b += l.color.b * k;
        }
        Color::new((c.r.min(0.7) * 2.0).min(1.0), (c.g.min(0.7) * 2.0).min(1.0), (c.b.min(0.7) * 2.0).min(1.0), 1.0)
    }

    /// Renders the light map over `view` (world units) with `cam`, then multiplies it onto
    /// `target` (the world drawn so far). `glow` is a soft round sprite, drawn additively.
    pub fn apply(&mut self, cam: &Camera2D, view: Rect, glow: &Texture2D, additive: &dyn Fn(&dyn Fn())) {
        let (w, h) = (((screen_width() * screen_dpi_scale()) / 4.0).max(16.0) as u32, ((screen_height() * screen_dpi_scale()) / 4.0).max(16.0) as u32);
        if self.rt.as_ref().is_none_or(|(_, rw, rh)| *rw != w || *rh != h) {
            let rt = render_target(w, h);
            rt.texture.set_filter(FilterMode::Linear);
            self.rt = Some((rt, w, h));
        }
        if self.multiply.is_none() {
            self.multiply = load_material(
                ShaderSource::Glsl { vertex: MUL_VERTEX, fragment: MUL_FRAGMENT },
                MaterialParams {
                    pipeline_params: PipelineParams {
                        // dst * src + src * dst: the world times twice the light.
                        color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::DestinationColor), BlendFactor::Value(BlendValue::SourceColor))),
                        alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .map_err(|e| {
                warn!("light multiply material: {e:?}");
            })
            .ok();
        }
        let (Some((rt, _, _)), Some(mul)) = (self.rt.as_ref(), self.multiply.as_ref()) else { return };
        if cfg!(not(target_arch = "wasm32")) && std::env::var("KZ_NOLIGHT").is_ok() {
            return; // dev: compare with the lighting off
        }
        let rt = rt.clone();
        let lc = Camera2D { target: cam.target, zoom: cam.zoom, offset: cam.offset, rotation: cam.rotation, render_target: Some(rt.clone()), viewport: None };
        set_camera(&lc);
        clear_background(self.ambient);
        let lights = &self.list;
        additive(&|| {
            for l in lights.iter() {
                let d = l.radius * 2.0;
                draw_texture_ex(glow, l.pos.x - l.radius, l.pos.y - l.radius, l.color, DrawTextureParams { dest_size: Some(vec2(d, d)), ..Default::default() });
            }
        });
        set_camera(cam);
        gl_use_material(mul);
        draw_texture_ex(&rt.texture, view.x, view.y, WHITE, DrawTextureParams { dest_size: Some(vec2(view.w, view.h)), ..Default::default() });
        gl_use_default_material();
    }
}
