//! Terrain pictures for the campaign worlds, drawn by a fragment shader.
//!
//! The sim keeps high and low ground as 16-unit cells (collision); this draws the smooth field
//! behind them (`sim::terrain::height`), so edges are organic. For each 256-unit chunk the CPU
//! samples the field on a coarse grid (one texel every 4 units, ~1 ms) into a small "mask"
//! texture: level, trail share and a slow patch value. The GPU does the per-pixel work, reading
//! the mask with hardware interpolation: the level's ground texture (soft edges), patches and
//! brightness variation, winding trails with stairs where they climb, a rock face below every
//! south-facing edge with a shadow at its foot, and a dark rim on the other edges.
//! `tools/ai/terrain_bake.py` is the same recipe in numpy, for trying changes quickly.
//!
//! Material uniforms apply per pipeline when the batch flushes, so nothing here is set per
//! chunk: the vertex shader passes the world position, and each chunk's mask sits in its
//! texture coordinates.

use std::collections::HashMap;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, TextureWrap};
use macroquad::prelude::*;
use sim::campaign::StageId;
use sim::terrain::{height, road_distance, HIGH_ABOVE, LOW_BELOW};

use crate::sprites;

/// World units per chunk.
pub const CHUNK: f32 = 256.0;
/// Mask: one texel every STEP units, with MARGIN units around the chunk (edge effects look
/// up to ~26 units away).
const STEP: f32 = 4.0;
const MARGIN: f32 = 32.0;
const G: usize = ((CHUNK + 2.0 * MARGIN) / STEP) as usize;
const KEEP: usize = 48;
/// New masks per frame (each ~1 ms).
const PER_FRAME: usize = 3;

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec4 color;
varying highp vec2 uv;
varying highp vec2 world;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
    world = position.xy;
}
"#;

const FRAGMENT: &str = r#"#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
varying lowp vec4 color;
varying vec2 uv;
varying vec2 world;
uniform sampler2D Texture;
uniform sampler2D LowTex;
uniform sampler2D GroundA;
uniform sampler2D GroundB;
uniform sampler2D HighTex;
uniform sampler2D TrailTex;
uniform sampler2D StairsTex;
uniform sampler2D CliffTex;
uniform vec2 MaskScale;

float sm(float t) { t = clamp(t, 0.0, 1.0); return t * t * (3.0 - 2.0 * t); }
// The level (0 low, 1 ground, 2 high) `d` world units from here.
float L(vec2 d) { return texture2D(Texture, uv + d * MaskScale).r * 2.0; }

void main() {
    vec2 w = world;
    vec4 m = texture2D(Texture, uv);
    float lv = m.r * 2.0;
    float r = m.g;
    float pr = m.b;
    float lo = clamp(1.0 - lv, 0.0, 1.0);
    float hi = clamp(lv - 1.0, 0.0, 1.0);
    float mid = clamp(1.0 - lo - hi, 0.0, 1.0);
    vec3 g = mix(texture2D(GroundA, w / 64.0).rgb, texture2D(GroundB, w / 64.0).rgb, sm((pr - 0.5) * 5.0 + 0.5));
    vec3 c = texture2D(LowTex, w / 64.0).rgb * lo + g * mid + texture2D(HighTex, w / 64.0).rgb * hi;
    c *= 0.92 + 0.16 * pr;
    c = mix(c, texture2D(TrailTex, w / 48.0).rgb, r);
    // A rock face below a south-facing edge: depth from the edge picks the texture row.
    float edge = -1.0;
    for (int k = 1; k <= 6; k++) {
        float d = float(k) * 3.0;
        if (edge < 0.0 && L(vec2(0.0, -d)) - lv > 0.5) { edge = d; }
    }
    bool face = edge > 0.0 && r < 0.5;
    if (face) {
        c = mix(c, texture2D(CliffTex, vec2(w.x / 64.0, min(edge / 18.0, 1.0) * 0.98)).rgb, 1.0 - r);
    } else if (L(vec2(0.0, -26.0)) - lv > 0.5) {
        c *= 1.0 - 0.38 * (1.0 - r);
    }
    // Stairs where a trail climbs.
    float climb = abs(L(vec2(0.0, -10.0)) - L(vec2(0.0, 10.0))) + abs(L(vec2(-10.0, 0.0)) - L(vec2(10.0, 0.0)));
    if (r > 0.3 && climb > 0.5) { c = mix(c, texture2D(StairsTex, w / 32.0).rgb, 0.9 * r); }
    // Dark rims where the ground drops north, west or east; a light lip atop south faces.
    if (r < 0.5) {
        if (lv - L(vec2(0.0, -5.0)) > 0.5 || lv - L(vec2(-5.0, 0.0)) > 0.5 || lv - L(vec2(5.0, 0.0)) > 0.5) { c *= 0.55; }
        if (lv - L(vec2(0.0, 3.0)) > 0.5) { c *= 1.25; }
    }
    gl_FragColor = vec4(c * color.rgb, 1.0);
}
"#;

const SAMPLERS: [&str; 7] = ["LowTex", "GroundA", "GroundB", "HighTex", "TrailTex", "StairsTex", "CliffTex"];

fn sm(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A ground texture with its contrast softened around the mean (busy tiles read calmer),
/// tiling (repeat wrap) and mipmapped where the GPU allows.
fn ground_texture(path: &str, contrast: f32) -> Texture2D {
    let im = sprites::hd_image(path);
    let mut px = im.bytes.clone();
    if contrast < 1.0 {
        let n = (px.len() / 4) as f32;
        let mut mean = [0.0f32; 3];
        for c in px.chunks(4) {
            for i in 0..3 {
                mean[i] += c[i] as f32 / n;
            }
        }
        for c in px.chunks_mut(4) {
            for i in 0..3 {
                c[i] = (mean[i] + (c[i] as f32 - mean[i]) * contrast).clamp(0.0, 255.0) as u8;
            }
        }
    }
    let t = Texture2D::from_rgba8(im.width, im.height, &px);
    t.set_filter(FilterMode::Linear);
    let gl = unsafe { get_internal_gl() };
    let id = t.raw_miniquad_id();
    gl.quad_context.texture_set_wrap(id, TextureWrap::Repeat, TextureWrap::Repeat);
    if !gl.quad_context.info().gl_version_string.contains("WebGL 1") {
        gl.quad_context.texture_generate_mipmaps(id);
        gl.quad_context.texture_set_min_filter(id, macroquad::miniquad::FilterMode::Linear, macroquad::miniquad::MipmapFilterMode::Linear);
    }
    t
}

pub struct Baker {
    stage: StageId,
    material: Option<Material>,
    /// The material's textures. `Material::set_texture` keeps only the GPU id, so the handles
    /// must live here or the textures are freed (and sample as black).
    _textures: Vec<Texture2D>,
    fill: Color,
    chunks: HashMap<(i32, i32), Texture2D>,
}

impl Baker {
    pub fn new(stage: StageId) -> Self {
        let w = stage.index();
        let material = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
            MaterialParams {
                uniforms: vec![UniformDesc::new("MaskScale", UniformType::Float2)],
                textures: SAMPLERS.iter().map(|s| s.to_string()).collect(),
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
                    ..Default::default()
                },
            },
        );
        let mut textures = Vec::new();
        let material = match material {
            Ok(m) => {
                for (name, file, contrast) in [("LowTex", "low", 0.6), ("GroundA", "ground_a", 0.6), ("GroundB", "ground_b", 0.6), ("HighTex", "high", 0.6), ("TrailTex", "trail", 1.0), ("StairsTex", "stairs", 1.0), ("CliffTex", "cliff", 1.0)] {
                    let t = ground_texture(&format!("w{w}/{file}"), contrast);
                    m.set_texture(name, t.clone());
                    textures.push(t);
                }
                m.set_uniform("MaskScale", vec2(1.0 / (G as f32 * STEP), 1.0 / (G as f32 * STEP)));
                Some(m)
            }
            Err(e) => {
                macroquad::logging::warn!("terrain shader: {e:?}; drawing flat ground");
                None
            }
        };
        let im = sprites::hd_image(&format!("w{w}/ground_a"));
        let n = (im.bytes.len() / 4) as f32;
        let mut m = [0.0f32; 3];
        for c in im.bytes.chunks(4) {
            for i in 0..3 {
                m[i] += c[i] as f32 / n / 255.0;
            }
        }
        Baker { stage, material, _textures: textures, fill: Color::new(m[0], m[1], m[2], 1.0), chunks: HashMap::new() }
    }

    pub fn stage(&self) -> StageId {
        self.stage
    }

    /// A chunk's mask: level / 2 (R), trail share (G), slow patch value (B), sampled at the
    /// centre of each texel.
    fn mask(&self, key: (i32, i32)) -> Texture2D {
        let o = vec2(key.0 as f32 * CHUNK - MARGIN, key.1 as f32 * CHUNK - MARGIN);
        let lv = |h: f32| sm((h - LOW_BELOW) / 0.012 + 0.5) + sm((h - HIGH_ABOVE) / 0.012 + 0.5);
        let mut px = vec![255u8; G * G * 4];
        for j in 0..G {
            for i in 0..G {
                let p = o + vec2((i as f32 + 0.5) * STEP, (j as f32 + 0.5) * STEP);
                let q = sim::Vec2::new(p.x, p.y);
                let k = (j * G + i) * 4;
                px[k] = (lv(height(self.stage, q)) / 2.0 * 255.0).round() as u8;
                px[k + 1] = (sm((1.9 - road_distance(self.stage, q)) / 0.5) * 255.0).round() as u8;
                px[k + 2] = (sim::terrain::noise(p.x / 90.0, p.y / 90.0, 7) * 255.0).round() as u8;
            }
        }
        let t = Texture2D::from_rgba8(G as u16, G as u16, &px);
        t.set_filter(FilterMode::Linear);
        t
    }

    /// Makes masks for the chunks around the view (a few per frame, nearest first) and drops
    /// far ones.
    pub fn update(&mut self, centre: Vec2, view: Rect) {
        let (kx0, ky0) = ((view.x / CHUNK).floor() as i32 - 1, (view.y / CHUNK).floor() as i32 - 1);
        let (kx1, ky1) = (((view.x + view.w) / CHUNK).floor() as i32 + 1, ((view.y + view.h) / CHUNK).floor() as i32 + 1);
        let key_dist = |k: &(i32, i32)| (vec2((k.0 as f32 + 0.5) * CHUNK, (k.1 as f32 + 0.5) * CHUNK) - centre).length_squared();
        let mut missing: Vec<(i32, i32)> = (ky0..=ky1).flat_map(|y| (kx0..=kx1).map(move |x| (x, y))).filter(|k| !self.chunks.contains_key(k)).collect();
        missing.sort_by(|a, b| key_dist(a).partial_cmp(&key_dist(b)).unwrap());
        for k in missing.into_iter().take(PER_FRAME) {
            let t = self.mask(k);
            self.chunks.insert(k, t);
        }
        if self.chunks.len() > KEEP {
            let mut keys: Vec<_> = self.chunks.keys().copied().collect();
            keys.sort_by(|a, b| key_dist(a).partial_cmp(&key_dist(b)).unwrap());
            for k in keys.into_iter().skip(KEEP) {
                self.chunks.remove(&k);
            }
        }
    }

    /// Draws the chunks covering `view` (ones without a mask yet as the ground colour).
    pub fn draw(&self, view: Rect) {
        let (kx0, ky0) = ((view.x / CHUNK).floor() as i32, (view.y / CHUNK).floor() as i32);
        let (kx1, ky1) = (((view.x + view.w) / CHUNK).floor() as i32, ((view.y + view.h) / CHUNK).floor() as i32);
        let Some(mat) = &self.material else {
            draw_rectangle(view.x - CHUNK, view.y - CHUNK, view.w + 2.0 * CHUNK, view.h + 2.0 * CHUNK, self.fill);
            return;
        };
        // The chunk itself is texels MARGIN/STEP .. +CHUNK/STEP of its mask.
        let src = Rect::new(MARGIN / STEP, MARGIN / STEP, CHUNK / STEP, CHUNK / STEP);
        gl_use_material(mat);
        for ky in ky0..=ky1 {
            for kx in kx0..=kx1 {
                if let Some(t) = self.chunks.get(&(kx, ky)) {
                    draw_texture_ex(t, kx as f32 * CHUNK, ky as f32 * CHUNK, WHITE, DrawTextureParams { dest_size: Some(vec2(CHUNK, CHUNK)), source: Some(src), ..Default::default() });
                }
            }
        }
        gl_use_default_material();
        for ky in ky0..=ky1 {
            for kx in kx0..=kx1 {
                if !self.chunks.contains_key(&(kx, ky)) {
                    draw_rectangle(kx as f32 * CHUNK, ky as f32 * CHUNK, CHUNK, CHUNK, self.fill);
                }
            }
        }
    }
}
