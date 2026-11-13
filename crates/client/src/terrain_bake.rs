//! Terrain pictures for the campaign worlds, drawn by a fragment shader.
//!
//! The sim keeps high and low ground as 16-unit cells (collision); this draws the smooth field
//! behind them (`sim::terrain::height`), so edges are organic. For each 256-unit chunk the CPU
//! samples the field on a coarse grid (one texel every 4 units) into a small "mask" texture:
//! the height around each of the two level thresholds (R: low/ground, B: ground/high, scaled so
//! 8 bits are precise near the contour), the distance to a road (G) and a slow patch value (A).
//! Hardware interpolation turns those into smooth fields, so the GPU gets clean, anti-aliased
//! contours at any zoom: signed distance = field / gradient. Per pixel it draws the level's
//! ground (low ground darker and cooler, high ground brighter), a rock face below every edge
//! that has higher ground to the north (its depth found by marching up the field, so the face
//! texture is continuous), a shadow at its foot, a lip of light on top, dark rims and soft
//! shadows on the other edges, and roads with frayed edges that become ramps where they cross a
//! face. Ground textures are hex-tiled (three randomly offset reads blended per pixel) so their
//! repeat never shows as a grid. Textures come from art/hd/<world>/terrain (1024 px per 128
//! units, see `tools/ai/terrain_hd.py`).
//!
//! Material uniforms apply per pipeline when the batch flushes, so nothing here is set per
//! chunk: the vertex shader passes the world position, and each chunk's mask sits in its
//! texture coordinates.

use std::collections::HashMap;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation, TextureWrap};
use macroquad::prelude::*;
use sim::campaign::StageId;
use sim::terrain::{height, road_distances, HIGH_ABOVE, LOW_BELOW};

use crate::sprites;

/// World units per chunk.
pub const CHUNK: f32 = 256.0;
/// Mask: one texel every STEP units, with MARGIN units around the chunk (faces look up to
/// FACE + 16 units north).
const STEP: f32 = 4.0;
const MARGIN: f32 = 56.0;
/// Height around a threshold is stored as (h - threshold) * K + 0.5.
const K: f32 = 8.0;
const G: usize = ((CHUNK + 2.0 * MARGIN) / STEP) as usize;
const KEEP: usize = 48;
/// New masks per frame (each ~1 ms).
const PER_FRAME: usize = 2;

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
uniform sampler2D GroundTex;
uniform sampler2D HighTex;
uniform sampler2D TrailTex;
uniform sampler2D CliffTex;
uniform sampler2D PatchTex;
uniform vec2 MaskScale;
uniform float Exposure;
// Per texture (low, ground, high, trail): 1 hex-tiled, 0 plain repeat (regular grids of
// tiles or slabs, whose joints must line up).
uniform vec4 Hex;

// A south rock face is FACE units tall (2 sim cells); its foot shadow SHADOW more.
const float FACE = 32.0;
const float SHADOW = 16.0;

float sm(float t) { t = clamp(t, 0.0, 1.0); return t * t * (3.0 - 2.0 * t); }
vec4 M(vec2 d) { return texture2D(Texture, uv + d * MaskScale); }

// Hex tiling: each ground texture is read three times, at random offsets fixed to the
// vertices of a triangle grid, and blended by (sharpened) barycentric weights, so the repeat
// of the tile never lines up into a visible grid.
vec2 hash2(vec2 p) { return fract(sin(vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)))) * 43758.5453); }
vec2 o1; vec2 o2; vec2 o3; vec3 hw;
void hexGrid(vec2 st) {
    st *= 3.4641016;
    vec2 sk = vec2(st.x - 0.57735027 * st.y, 1.15470054 * st.y);
    vec2 base = floor(sk);
    vec2 f = fract(sk);
    float z = 1.0 - f.x - f.y;
    vec2 v1; vec2 v2; vec2 v3;
    if (z > 0.0) { hw = vec3(z, f.y, f.x); v1 = base; v2 = base + vec2(0.0, 1.0); v3 = base + vec2(1.0, 0.0); }
    else { hw = vec3(-z, 1.0 - f.y, 1.0 - f.x); v1 = base + vec2(1.0); v2 = base + vec2(1.0, 0.0); v3 = base + vec2(0.0, 1.0); }
    o1 = hash2(v1); o2 = hash2(v2); o3 = hash2(v3);
    hw = hw * hw * hw * hw;
    hw /= hw.x + hw.y + hw.z;
}
vec3 hexTex(sampler2D t, vec2 p, float on) {
    if (on < 0.5) { return texture2D(t, p).rgb; }
    return texture2D(t, p + o1).rgb * hw.x + texture2D(t, p + o2).rgb * hw.y + texture2D(t, p + o3).rgb * hw.z;
}

// How far north (units) the field `ch` (0: low edge, 1: high edge) turns positive, from a pixel
// where it is f0 < 0; -1 if not within FACE + SHADOW. Linear between 4-unit samples (the mask's
// own resolution), so the depth is continuous.
float upEdge(float f0, float ch) {
    float prev = f0;
    for (int k = 1; k <= 12; k++) {
        vec4 s = M(vec2(0.0, -4.0 * float(k)));
        float f = mix(s.r, s.b, ch) - 0.5;
        if (f >= 0.0) { return 4.0 * (float(k - 1) + prev / (prev - f)); }
        prev = f;
    }
    return -1.0;
}

void main() {
    vec2 w = world;
    vec4 m = M(vec2(0.0));
    vec4 ex = M(vec2(4.0, 0.0)) - M(vec2(-4.0, 0.0));
    vec4 ey = M(vec2(0.0, 4.0)) - M(vec2(0.0, -4.0));
    // Signed distances (units) to the two contours, positive on the higher side.
    vec2 gl = vec2(ex.r, ey.r) / 8.0;
    vec2 gh = vec2(ex.b, ey.b) / 8.0;
    float sl = (m.r - 0.5) / max(length(gl), 1e-5);
    float sh = (m.b - 0.5) / max(length(gh), 1e-5);
    float lo = clamp(0.5 - sl / 1.2, 0.0, 1.0);
    float hi = clamp(0.5 + sh / 1.2, 0.0, 1.0);
    float mid = clamp(1.0 - lo - hi, 0.0, 1.0);
    float pr = texture2D(PatchTex, w / 1024.0).r;
    // Mid-scale wobble (about 12 units) for road edges.
    float wob = texture2D(PatchTex, w / 96.0 + vec2(0.37, 0.71)).r;

    vec2 tw = w / 128.0;
    hexGrid(w / 160.0);
    vec3 trail = hexTex(TrailTex, tw, Hex.w);
    // Roads: G is the distance to a north-south road, A to an east-west one (cells / 4). The
    // edge wanders with noise and with the road texture's own light and dark, so it frays
    // into the ground instead of running as a ruled line.
    float rd = min(m.g, m.a) * 4.0;
    float tl = dot(trail, vec3(0.3, 0.59, 0.11)) / max(dot(texture2D(TrailTex, vec2(0.5)).rgb, vec3(0.3, 0.59, 0.11)), 0.05);
    float r = 1.0 - sm((rd - 1.45 + (wob - 0.5) * 0.9 + (tl - 1.0) * 0.35) / 0.55);

    // Ground: each level its own texture and light (hollows dim and cool, plateaus bright).
    vec3 c = vec3(0.0);
    if (lo > 0.0) { c += hexTex(LowTex, tw, Hex.x) * lo * vec3(0.82, 0.88, 0.98); }
    if (mid > 0.0) { c += hexTex(GroundTex, tw, Hex.y) * mid; }
    if (hi > 0.0) { c += hexTex(HighTex, tw, Hex.z) * hi * vec3(1.06, 1.05, 1.02); }
    c *= 0.9 + 0.2 * pr;
    // Plateaus catch light towards their edges; hollows darken near the walls around them.
    c *= 1.0 + 0.12 * hi * (1.0 - sm(sh / 24.0));
    float occl = max((1.0 - sm(-sl / 24.0)) * step(sl, 0.0), (1.0 - sm(-sh / 24.0)) * step(sh, 0.0));
    c *= 1.0 - 0.35 * occl * (1.0 - 0.6 * r);
    // A soft dark fringe where the road meets the ground.
    c *= 1.0 - 0.18 * sm(1.0 - abs(r - 0.5) * 2.0);
    c = mix(c, trail * (0.92 + 0.16 * pr), r);

    // Rims: a dark band on the high side of edges that don't face south, a lip of light on
    // the top of south faces.
    for (int i = 0; i < 2; i++) {
        float s = i == 0 ? sl : sh;
        vec2 gr = i == 0 ? gl : gh;
        float south = sm((-normalize(gr + 1e-6).y - 0.35) / 0.4);
        float band = (1.0 - sm(s / 4.0)) * step(0.0, s) * (1.0 - r);
        c *= 1.0 - band * (0.4 * (1.0 - south));
        c *= 1.0 + band * 0.22 * south * (1.0 - sm(s / 2.0));
    }

    // Rock faces below edges with higher ground to the north: the higher edge first, the
    // nearer (lower) edge drawn over it. Where a road crosses a face it becomes a ramp: the
    // road surface carried down the slope, shaded darker towards the foot, with the rock
    // face rising beside it.
    float onRoad = sm((r - 0.3) / 0.45);
    float faced = 0.0;
    for (int i = 0; i < 2; i++) {
        float ch = i == 0 ? 1.0 : 0.0;
        float f0 = mix(m.r, m.b, ch) - 0.5;
        if (f0 >= 0.0) { continue; }
        float d = upEdge(f0, ch);
        if (d < 0.0) { continue; }
        if (d < FACE) {
            float v = d / FACE;
            vec3 rock = texture2D(CliffTex, vec2(w.x / 64.0, v * 0.98 + 0.01)).rgb * (1.05 - 0.3 * v);
            rock *= 1.0 - 0.35 * sm((v - 0.8) / 0.2);
            vec3 ramp = trail * (1.0 - 0.28 * v);
            c = mix(rock, ramp, onRoad);
            faced = 1.0;
        } else if (faced < 0.5) {
            c *= 1.0 - 0.45 * (1.0 - sm((d - FACE) / SHADOW)) * (1.0 - r);
        }
    }
    gl_FragColor = vec4(c * Exposure * color.rgb, 1.0);
}
"#;

const SAMPLERS: [&str; 6] = ["LowTex", "GroundTex", "HighTex", "TrailTex", "CliffTex", "PatchTex"];

/// A ground texture with its contrast softened around the mean (busy tiles read calmer),
/// tiling (repeat wrap) and mipmapped where the GPU allows.
pub fn ground_texture(path: &str, contrast: f32) -> Texture2D {
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

/// Slow light/dark variation across the ground (which of the two ground textures shows):
/// smooth value noise on a wrapping 8x8 lattice, 1024 world units per repeat.
fn patch_texture() -> Texture2D {
    const N: usize = 128;
    const L: i32 = 8;
    let v = |x: i32, y: i32| (sim::stage::hash(x.rem_euclid(L), y.rem_euclid(L), 7) & 0xFFFF) as f32 / 65535.0;
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let mut px = vec![255u8; N * N * 4];
    for j in 0..N {
        for i in 0..N {
            let (x, y) = (i as f32 / N as f32 * L as f32, j as f32 / N as f32 * L as f32);
            let (xi, yi) = (x.floor() as i32, y.floor() as i32);
            let (fx, fy) = (s(x - xi as f32), s(y - yi as f32));
            let top = v(xi, yi) + (v(xi + 1, yi) - v(xi, yi)) * fx;
            let bot = v(xi, yi + 1) + (v(xi + 1, yi + 1) - v(xi, yi + 1)) * fx;
            let k = (j * N + i) * 4;
            px[k..k + 3].fill(((top + (bot - top) * fy) * 255.0) as u8);
        }
    }
    let t = Texture2D::from_rgba8(N as u16, N as u16, &px);
    t.set_filter(FilterMode::Linear);
    let gl = unsafe { get_internal_gl() };
    gl.quad_context.texture_set_wrap(t.raw_miniquad_id(), TextureWrap::Repeat, TextureWrap::Repeat);
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
                uniforms: vec![UniformDesc::new("MaskScale", UniformType::Float2), UniformDesc::new("Exposure", UniformType::Float1), UniformDesc::new("Hex", UniformType::Float4)],
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
                for (name, file, contrast) in [("LowTex", "low", 1.0), ("GroundTex", "ground_a", 1.0), ("HighTex", "high", 1.0), ("TrailTex", "trail", 1.0), ("CliffTex", "cliff", 1.0)] {
                    let t = ground_texture(&format!("w{w}/terrain/{file}"), contrast);
                    m.set_texture(name, t.clone());
                    textures.push(t);
                }
                let p = patch_texture();
                m.set_texture("PatchTex", p.clone());
                textures.push(p);
                // The marble terrace (w3) and the sidewalk slabs (w4) are regular grids.
                let grid_high = matches!(stage, StageId::PaleCitadel | StageId::Rainward);
                m.set_uniform("Hex", vec4(1.0, 1.0, if grid_high { 0.0 } else { 1.0 }, 1.0));
                m.set_uniform("MaskScale", vec2(1.0 / (G as f32 * STEP), 1.0 / (G as f32 * STEP)));
                Some(m)
            }
            Err(e) => {
                macroquad::logging::warn!("terrain shader: {e:?}; drawing flat ground");
                None
            }
        };
        let im = sprites::hd_image(&format!("w{w}/terrain/ground_a"));
        let n = (im.bytes.len() / 4) as f32;
        let mut m = [0.0f32; 3];
        for c in im.bytes.chunks(4) {
            for i in 0..3 {
                m[i] += c[i] as f32 / n / 255.0;
            }
        }
        // Night worlds with very dark ground are lifted so their levels still read.
        let exposure = (0.17 / (0.3 * m[0] + 0.59 * m[1] + 0.11 * m[2]).max(0.01)).clamp(1.0, 1.6);
        if let Some(mat) = &material {
            mat.set_uniform("Exposure", exposure);
        }
        let fill = Color::new(m[0] * exposure, m[1] * exposure, m[2] * exposure, 1.0);
        Baker { stage, material, _textures: textures, fill, chunks: HashMap::new() }
    }

    pub fn stage(&self) -> StageId {
        self.stage
    }

    /// A chunk's mask, sampled at the centre of each texel: the height around the low (R) and
    /// high (B) thresholds, distance to a north-south (G) and an east-west road (A), cells / 4.
    fn mask(&self, key: (i32, i32)) -> Texture2D {
        let o = vec2(key.0 as f32 * CHUNK - MARGIN, key.1 as f32 * CHUNK - MARGIN);
        let enc = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let mut px = vec![255u8; G * G * 4];
        for j in 0..G {
            for i in 0..G {
                let p = o + vec2((i as f32 + 0.5) * STEP, (j as f32 + 0.5) * STEP);
                let q = sim::Vec2::new(p.x, p.y);
                let k = (j * G + i) * 4;
                let h = height(self.stage, q);
                px[k] = enc((h - LOW_BELOW) * K + 0.5);
                let (dv, dh) = road_distances(self.stage, q);
                px[k + 1] = enc(dv / 4.0);
                px[k + 2] = enc((h - HIGH_ABOVE) * K + 0.5);
                px[k + 3] = enc(dh / 4.0);
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
