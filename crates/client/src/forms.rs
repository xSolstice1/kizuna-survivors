//! The transformations, drawn as set pieces: geometry and shaders instead of flat sprites.
//!
//! - **Nine-Tails** (Riku): nine tails of living flame that are the attack itself: each
//!   follows its segment from the sim (`World::fox_tails`), resting long and fanned behind
//!   him and whipping out in turn to strike, a long flowing S-curve with whip lag, painted as
//!   the game's flames are (soft layers from an amber edge to a near-white heart, gentle
//!   tongues, wisps at the tip, no outline).
//! - **Hannya** (Shun), **Mukade** (Toya), Ren's silk (**Divine**) and Kuon's sky
//!   (**Firmament**): below.
//!
//! Ribbons (`ribbon`) and the fur material are general: any tail, tendril or body chain can
//! use them.
use std::cell::RefCell;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec4 color;
varying highp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

/// Fur along a ribbon: uv.x across (0..1), uv.y along (0 root .. 1 tip). Vertex colour: the
/// tail's tint (rgb) and its opacity (a).
const FUR: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying highp vec2 uv;
uniform float Time;
uniform vec3 Base;
uniform vec3 Mid;
uniform vec3 Tip;
float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0)), f.x), f.y);
}
void main() {
    float across = abs(uv.x * 2.0 - 1.0);
    float v = uv.y;
    // Fur: coarse locks and fine strands, both flowing slowly towards the tip.
    float s = vnoise(vec2(uv.x * 12.0, v * 6.0 - Time * 0.5)) * 0.55 + vnoise(vec2(uv.x * 31.0, v * 14.0 - Time * 0.9)) * 0.45;
    float tuft = vnoise(vec2(v * 16.0 - Time * 0.7, uv.x * 2.0));
    // The outline breaks into tufts, more towards the tip, and the tip into foxfire.
    float edge = across + (tuft - 0.5) * (0.6 + 0.5 * v) + (s - 0.5) * 0.25;
    float a = 1.0 - smoothstep(0.7, 0.95, edge);
    a *= 1.0 - smoothstep(0.88, 1.0, v + (tuft - 0.5) * 0.3);
    vec3 c = v < 0.5 ? mix(Base, Mid, v / 0.5) : mix(Mid, Tip, smoothstep(0.55, 0.92, v));
    float strands = 0.5 + 0.5 * sin((uv.x * 26.0 + s * 3.0) * 3.14159);
    c *= 0.7 + 0.3 * (1.0 - across * across);
    c *= 0.84 + 0.26 * strands * (0.4 + 0.6 * s);
    // A darker rim gives each tail a clean edge against the others and the ground.
    c *= mix(1.0, 0.5, smoothstep(0.45, 0.9, edge));
    // Sheen along the crest, stronger towards the tip.
    c += vec3(1.0, 0.9, 0.7) * pow(1.0 - across, 4.0) * 0.28 * v;
    c *= color.rgb;
    gl_FragColor = vec4(c, a * color.a);
}
"#;

/// Fire along a ribbon (a tail of living flame): uv.x across, uv.y root (0) to tip (1). A
/// white-hot core, an orange body, red licking edges, bright streaks flowing to the tip.
const FIRE: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying highp vec2 uv;
uniform float Time;
uniform vec3 Base;
uniform vec3 Mid;
uniform vec3 Tip;
float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0)), f.x), f.y);
}
void main() {
    float across = abs(uv.x * 2.0 - 1.0);
    float v = uv.y;
    // Painted fire, as the game's flames are drawn: soft layered colour from an amber edge to
    // a near-white heart, gentle tongues drifting to the tip (low frequency only: it flows,
    // it doesn't jitter), the tip dissolving into wisps. No outline.
    float w = vnoise(vec2(uv.x * 2.5, v * 3.0 - Time * 1.4)) * 0.65 + vnoise(vec2(uv.x * 5.0 + 3.0, v * 6.0 - Time * 2.2)) * 0.35;
    float edge = across + (w - 0.5) * (0.35 + 0.5 * v);
    float body = 1.0 - smoothstep(0.3, 0.95, edge);
    body *= 1.0 - smoothstep(0.72, 1.0, v + (w - 0.5) * 0.35);
    float depth = 1.0 - edge;
    vec3 c = mix(Base, Mid, smoothstep(0.05, 0.45, depth));
    c = mix(c, Tip, smoothstep(0.62, 0.95, depth) * (1.0 - 0.5 * v));
    gl_FragColor = vec4(c * color.rgb, body * color.a);
}
"#;

struct Fox {
    /// Each tail's chain, root first (world units).
    tails: Vec<Vec<Vec2>>,
    last: f64,
}

struct State {
    /// Each ribbon shader, painted and added as light (keyed by its source).
    mats: std::collections::HashMap<(usize, bool), Option<Material>>,
    fox: Option<Fox>,
    /// Ren's two silk sashes.
    silk: Option<(Vec<Vec<Vec2>>, f64)>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State { mats: Default::default(), fox: None, silk: None });
}

const POINTS: usize = 22;

/// The fur material: painted, or (`light`) added as light.
fn fur(light: bool) -> Option<Material> {
    shader(FUR, light)
}

/// The fire material: painted, or (`light`) added as light.
fn fire(light: bool) -> Option<Material> {
    shader(FIRE, light)
}

/// A shader with the ribbon uniforms (Time, Base, Mid, Tip), painted or added as light; made
/// once each.
fn shader(src: &'static str, light: bool) -> Option<Material> {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let key = src.as_ptr() as usize;
        if !s.mats.contains_key(&(key, false)) {
            let u = |n: &str, t| UniformDesc::new(n, t);
            let make = |sf: BlendFactor, df: BlendFactor| {
                load_material(
                    ShaderSource::Glsl { vertex: VERTEX, fragment: src },
                    MaterialParams {
                        pipeline_params: PipelineParams {
                            color_blend: Some(BlendState::new(Equation::Add, sf, df)),
                            alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One)),
                            ..Default::default()
                        },
                        uniforms: vec![u("Time", UniformType::Float1), u("Base", UniformType::Float3), u("Mid", UniformType::Float3), u("Tip", UniformType::Float3)],
                        ..Default::default()
                    },
                )
                .map_err(|e| {
                    warn!("form material: {e:?}");
                })
                .ok()
            };
            let paint = make(BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha));
            let add = make(BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::One);
            s.mats.insert((key, false), paint);
            s.mats.insert((key, true), add);
        }
        s.mats.get(&(key, light)).cloned().flatten()
    })
}

/// Sets the ribbon uniforms.
fn colours(m: &Material, t: f32, base: Vec3, mid: Vec3, tip: Vec3) {
    m.set_uniform("Time", t);
    m.set_uniform("Base", base);
    m.set_uniform("Mid", mid);
    m.set_uniform("Tip", tip);
}

/// A tapered quad from `a` to `b` (widths `w0`, `w1`), coloured `c0` to `c1`: a leg segment.
fn taper(a: Vec2, b: Vec2, w0: f32, w1: f32, c0: Color, c1: Color, vs: &mut Vec<Vertex>, is: &mut Vec<u16>) {
    let d = (b - a).normalize_or(vec2(0.0, -1.0));
    let n = vec2(-d.y, d.x);
    let k = vs.len() as u16;
    for (q, c) in [(a - n * w0 * 0.5, c0), (a + n * w0 * 0.5, c0), (b + n * w1 * 0.5, c1), (b - n * w1 * 0.5, c1)] {
        vs.push(Vertex::new(q.x, q.y, 0.0, 0.5, 0.5, c));
    }
    is.extend_from_slice(&[k, k + 1, k + 2, k, k + 2, k + 3]);
}

/// A square around `c` (half-size `r`), uv 0..1: for shaders that draw a disc.
fn quad(c: Vec2, r: f32, col: Color) -> Mesh {
    let v = |x: f32, y: f32, u: f32, w: f32| Vertex::new(c.x + x, c.y + y, 0.0, u, w, col);
    Mesh { vertices: vec![v(-r, -r, 0.0, 0.0), v(r, -r, 1.0, 0.0), v(r, r, 1.0, 1.0), v(-r, r, 0.0, 1.0)], indices: vec![0, 1, 2, 0, 2, 3], texture: None }
}

/// A ribbon along `pts` (root first): `width(t)` across at each point (t 0..1 along), uv.x
/// across and uv.y along, every vertex coloured `c`.
pub fn ribbon(pts: &[Vec2], width: impl Fn(f32) -> f32, c: Color) -> Mesh {
    let n = pts.len();
    let mut vertices = Vec::with_capacity(n * 2);
    let mut indices = Vec::with_capacity((n - 1) * 6);
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let a = pts[i.saturating_sub(1)];
        let b = pts[(i + 1).min(n - 1)];
        let d = (b - a).normalize_or(vec2(0.0, -1.0));
        let side = vec2(-d.y, d.x) * width(t) * 0.5;
        let (l, r) = (pts[i] - side, pts[i] + side);
        vertices.push(Vertex::new(l.x, l.y, 0.0, 0.0, t, c));
        vertices.push(Vertex::new(r.x, r.y, 0.0, 1.0, t, c));
        if i + 1 < n {
            let k = (i * 2) as u16;
            indices.extend_from_slice(&[k, k + 1, k + 3, k, k + 3, k + 2]);
        }
    }
    Mesh { vertices, indices, texture: None }
}

/// Nine-Tails: Riku's nine tails of living flame. Each follows its segment from the sim
/// (`World::fox_tails`, what it hits with): resting long and fanned behind him, whipping out
/// to strike and back, so the tails you see are the attack. A tail is a long S-curve (thin at
/// the root, swelling thick, tapering to a point) along its segment; the chain eases towards
/// it, loose at the tip, so a lash whips. Drawn as fire (a painted flame and its glow).
/// `facing` -1 left / 1 right; `tails` the sim's (root, tip) per tail.
pub fn draw_fox(_hips: Vec2, facing: f32, tails: &[(Vec2, Vec2)], time: f64, vfx: &crate::vfx::Vfx) {
    let t = time as f32;
    let _ = facing;
    let root = tails.first().map(|s| s.0).unwrap_or_default();
    // Foxfire rising round him, a warm glow on the ground, heat in the air.
    vfx.additive(|| {
        vfx.draw_wh("glow", root + vec2(0.0, 18.0), vec2(70.0, 34.0), 0.0, Color::new(1.0, 0.45, 0.1, 0.25));
        for k in 0..8 {
            let ph = (t * 0.9 + k as f32 * 0.137).fract();
            let ang = k as f32 * 0.785 + t * 0.4;
            let p = root + vec2(ang.cos() * 14.0, 16.0 + ang.sin() * 5.0 - ph * 30.0);
            vfx.draw_wh("flame", p, vec2(7.0, 11.0) * (1.0 - ph * 0.6), 0.0, Color::new(1.0, 0.6, 0.25, 0.5 * (1.0 - ph)));
        }
    });
    crate::post::heat(root + vec2(0.0, -12.0), 40.0, 0.7);
    let (Some(paint), Some(light)) = (fire(false), fire(true)) else { return };
    // Each tail's target curve along its segment: an S, bending less as it stretches out.
    let target = |k: usize, a: Vec2, b: Vec2| -> [Vec2; POINTS] {
        let d = b - a;
        let len = d.length().max(1.0);
        let side = vec2(-d.y, d.x) / len;
        let stretch = ((len - 55.0) / 75.0).clamp(0.0, 1.0);
        let s = if k % 2 == 0 { 1.0 } else { -1.0 };
        let mut out = [a; POINTS];
        for (j, o) in out.iter_mut().enumerate() {
            let u = j as f32 / (POINTS - 1) as f32;
            // A wave travelling down the tail: it flows.
            let wave = (t * 2.4 + k as f32 * 1.3 - u * 5.0).sin() * 0.12 * u;
            let bend = s * ((u * std::f32::consts::PI).sin() * 0.3 - u * u * u * 0.22 + wave) * len * (1.0 - 0.7 * stretch);
            *o = a + d * u + side * bend;
        }
        out
    };
    let chains = STATE.with(|st| {
        let mut st = st.borrow_mut();
        let fresh = st.fox.as_ref().is_none_or(|f| time - f.last > 0.5 || f.tails.len() != tails.len());
        if fresh {
            st.fox = Some(Fox { tails: tails.iter().enumerate().map(|(k, (a, b))| target(k, *a, *b).to_vec()).collect(), last: time });
        }
        let fox = st.fox.as_mut().unwrap();
        let dt = ((time - fox.last) as f32).clamp(0.0, 0.05);
        fox.last = time;
        for (k, chain) in fox.tails.iter_mut().enumerate() {
            let (a, b) = tails[k];
            let goal = target(k, a, b);
            chain[0] = a;
            for j in 1..POINTS {
                // Stiff at the root, loose at the tip: the tip trails, so a lash whips.
                let stiff = 22.0 - 16.0 * j as f32 / POINTS as f32;
                let cj = chain[j];
                chain[j] = cj + (goal[j] - cj) * (1.0 - (-dt * stiff).exp());
            }
        }
        fox.tails.clone()
    });
    // Thin at the root, swelling thick, tapering to a point.
    let width = |u: f32| 17.0 * (0.25 + 0.95 * (u.powf(0.75) * std::f32::consts::PI).sin().max(0.0)) * (1.0 - 0.4 * u);
    let set = |m: &Material| colours(m, t, vec3(0.92, 0.24, 0.04), vec3(1.0, 0.5, 0.1), vec3(1.0, 0.9, 0.62));
    // The glow first (wide and soft, added), then the flames.
    gl_use_material(&light);
    set(&light);
    for chain in &chains {
        draw_mesh(&ribbon(chain, |u| width(u) * 1.8, Color::new(1.0, 0.55, 0.3, 0.12)));
    }
    gl_use_default_material();
    gl_use_material(&paint);
    set(&paint);
    for chain in &chains {
        draw_mesh(&ribbon(chain, width, Color::new(1.0, 1.0, 1.0, 1.0)));
    }
    gl_use_default_material();
    // The tips burn brightest.
    vfx.additive(|| {
        for (k, chain) in chains.iter().enumerate() {
            let tip = chain[POINTS - 1];
            let flick = 0.8 + 0.2 * (t * 11.0 + k as f32 * 2.3).sin();
            vfx.draw("glow", tip, 18.0 * flick, 0.0, Color::new(1.0, 0.75, 0.4, 0.35));
        }
    });
}

/// Hannya (Shun): a scorched ground glowing crimson at the aura's reach `r`, the hannya mask
/// looming above him (painted, breathing, lit from below), and a crown of crimson flame rising
/// round him (painted fire, as the foxfire), dark smoke curling off it. `c` is his centre.
pub fn draw_hannya(c: Vec2, r: f32, time: f64, vfx: &crate::vfx::Vfx) {
    use std::f32::consts::PI;
    let t = time as f32;
    let breathe = 0.5 + 0.5 * (t * 2.2).sin();
    let g = c + vec2(0.0, 8.0);
    // Scorched ground, a crimson glow in it, the reach.
    vfx.draw_wh("glow", g, vec2(r * 2.0, r * 1.1), 0.0, Color::new(0.06, 0.0, 0.02, 0.5));
    vfx.additive(|| {
        vfx.draw_wh("glow", g, vec2(r * 1.2, r * 0.62), 0.0, Color::new(0.75, 0.04, 0.08, 0.12 + 0.06 * breathe));
        vfx.draw_wh("ring", g, vec2(r * 2.1, r * 1.15), 0.0, Color::new(1.0, 0.2, 0.25, 0.2));
    });
    // Smoke curling up behind (painted, dark).
    for k in 0..7 {
        let kf = k as f32;
        let ph = (t * 0.32 + kf * 0.143).fract();
        let q = c + vec2((kf * 2.1).sin() * 18.0 + (t + kf).sin() * 4.0, -18.0 - ph * 62.0);
        vfx.draw("smoke", q, 22.0 + ph * 34.0, kf + t * 0.3, Color::new(0.14, 0.03, 0.05, 0.34 * (ph * PI).sin()));
    }
    // The demon: the mask looming above him, breathing, lit by the fire below.
    let m = c + vec2(0.0, -52.0 - 3.0 * breathe);
    let size = 56.0 + 4.0 * breathe;
    vfx.additive(|| vfx.draw("glow", m + vec2(0.0, 8.0), size * 1.8, 0.0, Color::new(0.8, 0.02, 0.1, 0.3)));
    vfx.draw("hannya", m, size, 0.0, Color::new(0.85, 0.55, 0.55, 0.42));
    vfx.additive(|| {
        vfx.draw("hannya", m, size, 0.0, Color::new(1.0, 0.25, 0.2, 0.14 + 0.08 * breathe));
        for sx in [-1.0f32, 1.0] {
            vfx.draw("glow", m + vec2(sx * size * 0.17, -size * 0.04), 5.0 + 1.5 * breathe, 0.0, Color::new(1.0, 0.75, 0.45, 0.6));
        }
    });
    crate::post::heat(c + vec2(0.0, -10.0), 36.0, 0.6);
    // The crown of crimson flame: the game's painted hellfire, a bed of it round his feet
    // flickering (each flame stretching and leaning on its own), tongues breaking off and
    // rising, swaying, shrinking as they burn out. Painted, with their glow added under.
    let bed = |k: usize| {
        let kf = k as f32;
        let x = (kf / 4.0 - 0.5) * 2.0;
        let flick = (t * 7.0 + kf * 2.3).sin() * 0.5 + (t * 11.3 + kf * 1.1).sin() * 0.5;
        let h = (40.0 + 22.0 * (1.0 - x * x)) * (1.0 + 0.1 * flick);
        (c + vec2(x * 24.0, 16.0 - h * 0.5), vec2(h * (0.9 - 0.06 * flick), h), x * 0.3 + flick * 0.05)
    };
    let tongue = |k: usize| {
        let kf = k as f32;
        let ph = (t * 1.1 + kf * 0.618).fract();
        let x = ((kf * 0.381).fract() - 0.5) * 2.0;
        let sway = (t * 3.0 + kf * 1.9).sin() * 4.0 * ph;
        let size = 24.0 * (1.0 - ph * 0.6) * (0.8 + 0.4 * (kf * 0.73).fract());
        let fade = (ph / 0.15).min(1.0) * (1.0 - ph);
        (c + vec2(x * 22.0 * (1.0 + ph * 0.3) + sway, -14.0 - ph * 60.0), size, sway * 0.04, fade)
    };
    vfx.additive(|| {
        for k in 0..5 {
            let (q, wh, _) = bed(k);
            vfx.draw_wh("glow", q, wh * 1.4, 0.0, Color::new(0.85, 0.05, 0.1, 0.12));
        }
    });
    for k in 0..5 {
        let (q, wh, rot) = bed(k);
        vfx.draw_wh("hellfire", q, wh, rot, Color::new(1.0, 0.85, 0.85, 0.9));
    }
    for k in 0..14 {
        let (q, size, rot, fade) = tongue(k);
        vfx.draw_wh("flame", q, vec2(size * 0.7, size), rot, Color::new(1.0, 0.25, 0.3, 0.85 * fade));
    }
    vfx.additive(|| {
        for k in 0..14 {
            let (q, size, rot, fade) = tongue(k);
            vfx.draw_wh("flame", q, vec2(size * 0.45, size * 0.7), rot, Color::new(1.0, 0.45, 0.4, 0.5 * fade));
        }
        let (q, wh, _) = bed(2);
        vfx.draw_wh("hellfire", q + vec2(0.0, 4.0), wh * 0.7, 0.0, Color::new(1.0, 0.4, 0.35, 0.35));
    });
}

/// Lacquered chitin along a ribbon: overlapping plates (curved back at the flanks) near-black
/// `Base`, their seams glowing with the curse `Mid`, a gloss `Tip` off the spine from the
/// upper left, a crimson rim on the flanks.
const CHITIN: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying highp vec2 uv;
uniform float Time;
uniform vec3 Base;
uniform vec3 Mid;
uniform vec3 Tip;
void main() {
    float s = uv.x * 2.0 - 1.0;
    float across = abs(s);
    float seg = fract(uv.y * 15.0 + across * across * 0.5);
    float plate = sqrt(max(0.0, 1.0 - across * across));
    vec3 c = Base * (0.4 + 0.6 * plate) * (0.55 + 0.6 * (1.0 - seg));
    float seam = 1.0 - smoothstep(0.0, 0.12, seg);
    c += Mid * seam * (0.5 + 0.3 * sin(Time * 5.0 - uv.y * 14.0)) * (0.4 + 0.6 * plate);
    c += Tip * exp(-(s + 0.32) * (s + 0.32) * 12.0) * 0.6 * (1.0 - seg * 0.7);
    c += Mid * 0.45 * smoothstep(0.7, 0.98, across);
    float a = 1.0 - smoothstep(0.9, 1.0, across);
    gl_FragColor = vec4(c * color.rgb, a * color.a);
}
"#;

thread_local! {
    static MUKADE: RefCell<Option<(Vec<Vec2>, f64)>> = const { RefCell::new(None) };
}

/// Mukade (Toya): a great centipede growing from his back, rearing over his head with its
/// head hanging in front of him, hunting (swaying, striking forward now and then, the body
/// following through as he moves). Lacquered plates, amber legs scuttling in a wave, a shadow
/// on the ground; crimson tendrils writhing at his back. `root` is where it grows from,
/// `back` -1/1 the side behind him.
pub fn draw_mukade(root: Vec2, back: f32, time: f64, vfx: &crate::vfx::Vfx, head: &Texture2D) {
    use std::f32::consts::PI;
    let t = time as f32;
    const N: usize = 18;
    let ph = (t * 0.6).fract();
    let lunge = if ph < 0.22 { (ph / 0.22 * PI).sin().powf(0.7) } else { 0.0 };
    let rest: Vec<Vec2> = (0..N)
        .map(|s| {
            let u = s as f32 / (N - 1) as f32;
            let th = PI * (0.06 + 0.8 * u);
            let reach = 10.0 + 26.0 * u;
            let sway = (t * 1.7 - u * 3.0).sin() * 5.0 * u;
            let bob = (t * 3.0 - u * 2.0).sin() * 2.5 * u;
            let strike = vec2(-back * 26.0, 12.0) * lunge * u * u;
            root + vec2(back * th.cos() * reach + sway, -th.sin() * 50.0 * (0.4 + 0.6 * u.sqrt()) + bob) + strike
        })
        .collect();
    let body = MUKADE.with(|m| {
        let mut m = m.borrow_mut();
        if m.as_ref().is_none_or(|(_, last)| time - last > 0.5) {
            *m = Some((rest.clone(), time));
        }
        let (pts, last) = m.as_mut().unwrap();
        let dt = ((time - *last) as f32).clamp(0.0, 0.05);
        *last = time;
        pts[0] = rest[0];
        for j in 1..N {
            let stiff = 26.0 - 16.0 * j as f32 / N as f32;
            let pj = pts[j];
            pts[j] = pj + (rest[j] - pj) * (1.0 - (-dt * stiff).exp());
        }
        pts.clone()
    });
    let width = |u: f32| 17.0 * (0.45 + 0.55 * (u * 5.0).min(1.0)) * (1.0 - u * 0.25);
    // Its shadow on the ground under it.
    for (s, q) in body.iter().enumerate() {
        let u = s as f32 / (N - 1) as f32;
        let gnd = vec2(q.x + 5.0 + (root.y - q.y) * 0.25, root.y + 10.0);
        vfx.draw_wh("glow", gnd, vec2(width(u) * 1.4, width(u) * 0.7), 0.0, Color::new(0.03, 0.0, 0.02, 0.16));
    }
    // Crimson tendrils writhing behind him (light, so they glow).
    if let Some(light) = fur(true) {
        gl_use_material(&light);
        colours(&light, t, vec3(0.5, 0.0, 0.05), vec3(0.9, 0.1, 0.15), vec3(1.0, 0.45, 0.4));
        for k in 0..5 {
            let kf = k as f32;
            let a0 = -std::f32::consts::FRAC_PI_2 + back * (0.4 + 0.25 * kf) + (t * 2.5 + kf * 1.7).sin() * 0.3;
            let pts: Vec<Vec2> = (0..9)
                .map(|i| {
                    let u = i as f32 / 8.0;
                    let a = a0 + (t * 4.0 + kf * 2.0 - u * 5.0).sin() * 0.5 * u;
                    root + vec2(a.cos(), a.sin()) * (u * 30.0) + vec2(0.0, 4.0)
                })
                .collect();
            draw_mesh(&ribbon(&pts, |u| 6.0 * (1.0 - u * 0.8), Color::new(1.0, 1.0, 1.0, 0.5)));
        }
        gl_use_default_material();
    }
    // Legs: a pair per plate, thigh dark, shin amber to a pale tip, scuttling in a wave.
    let (mut vs, mut is) = (Vec::new(), Vec::new());
    for s in 2..N - 1 {
        let (a, b) = (body[s - 1], body[s + 1]);
        let d = (b - a).normalize_or(vec2(0.0, -1.0));
        let n = vec2(-d.y, d.x);
        let u = s as f32 / (N - 1) as f32;
        let w = width(u);
        let size = 12.0 * (1.0 - u * 0.3);
        let step = (t * 15.0 - s as f32 * 1.1).sin();
        for sgn in [-1.0f32, 1.0] {
            let st = step * sgn;
            let hip = body[s] + n * sgn * w * 0.35;
            let knee = hip + n * sgn * size * 0.7 + d * (st * 2.5 + 1.0);
            let foot = knee + n * sgn * size * 0.25 - d * (3.5 - st * 2.5) + vec2(0.0, 2.5);
            taper(hip, knee, 3.4, 2.6, Color::new(0.16, 0.02, 0.04, 1.0), Color::new(0.5, 0.1, 0.06, 1.0), &mut vs, &mut is);
            taper(knee, foot, 2.6, 0.7, Color::new(0.92, 0.42, 0.12, 1.0), Color::new(1.0, 0.82, 0.45, 1.0), &mut vs, &mut is);
        }
    }
    draw_mesh(&Mesh { vertices: vs, indices: is, texture: None });
    if let Some(mat) = shader(CHITIN, false) {
        gl_use_material(&mat);
        colours(&mat, t, vec3(0.2, 0.03, 0.05), vec3(0.9, 0.08, 0.14), vec3(1.0, 0.72, 0.65));
        draw_mesh(&ribbon(&body, width, WHITE));
        gl_use_default_material();
    }
    // The head at the tip, facing along the body, eyes burning.
    let (a, b) = (body[N - 3], body[N - 1]);
    let d = (b - a).normalize_or(vec2(0.0, 1.0));
    let size = 26.0;
    let hc = b + d * 6.0;
    draw_texture_ex(head, hc.x - size / 2.0, hc.y - size / 2.0, WHITE, DrawTextureParams { dest_size: Some(vec2(size, size)), rotation: d.y.atan2(d.x) - std::f32::consts::FRAC_PI_2, ..Default::default() });
    vfx.additive(|| {
        vfx.draw("glow", hc, 26.0, 0.0, Color::new(1.0, 0.1, 0.18, 0.1 + 0.05 * (t * 6.0).sin()));
        let n = vec2(-d.y, d.x);
        for sx in [-1.0f32, 1.0] {
            vfx.draw("glow", hc + n * sx * 4.0 + d * 2.0, 4.0, 0.0, Color::new(1.0, 0.6, 0.3, 0.8));
        }
    });
}

/// Silk: a celestial maiden's sash. Soft folds travelling along it, a sheen on each fold,
/// translucent towards its edges, `Base` near her fading to `Mid` at the ends.
const SILK: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying highp vec2 uv;
uniform float Time;
uniform vec3 Base;
uniform vec3 Mid;
uniform vec3 Tip;
void main() {
    float across = abs(uv.x * 2.0 - 1.0);
    float v = uv.y;
    float fold = 0.5 + 0.5 * sin(v * 16.0 - Time * 3.0 + uv.x * 2.5);
    float fold2 = 0.5 + 0.5 * sin(v * 6.0 + Time * 1.6);
    vec3 c = mix(Base, Mid, smoothstep(0.1, 1.0, v));
    c *= 0.62 + 0.45 * fold * (0.5 + 0.5 * fold2);
    c += Tip * pow(fold, 10.0) * 0.45 * (1.0 - across);
    // A gold trim along both edges.
    c = mix(c, vec3(1.0, 0.8, 0.38), smoothstep(0.62, 0.78, across) * 0.85);
    float a = (1.0 - smoothstep(0.82, 1.0, across)) * 0.92;
    a *= smoothstep(0.0, 0.08, v) * (1.0 - smoothstep(0.88, 1.0, v));
    gl_FragColor = vec4(c * color.rgb, a * color.a);
}
"#;

/// Divine (Ren): two silk sashes floating round her, looping out and trailing as she moves.
/// `feet` her feet.
pub fn draw_silk(feet: Vec2, time: f64, open: f32) {
    use std::f32::consts::PI;
    let t = time as f32;
    const P: usize = 30;
    let target = |k: usize| -> Vec<Vec2> {
        let sg = if k == 0 { 1.0 } else { -1.0 };
        (0..P)
            .map(|j| {
                let u = j as f32 / (P - 1) as f32;
                let th = sg * (0.4 + u * PI * 1.3) + (t * 0.8 + k as f32 * PI).sin() * 0.25;
                let r = (6.0 + 26.0 * u) * open;
                let float = (t * 1.8 + u * 6.0 + k as f32 * 2.0).sin() * 4.0 * u;
                feet + vec2(th.sin() * r, -21.0 + u * 10.0 - th.cos() * r * 0.32 + float)
            })
            .collect()
    };
    let sashes = STATE.with(|st| {
        let mut st = st.borrow_mut();
        if st.silk.as_ref().is_none_or(|(_, last)| time - last > 0.5) {
            st.silk = Some(((0..2).map(target).collect(), time));
        }
        let (ch, last) = st.silk.as_mut().unwrap();
        let dt = ((time - *last) as f32).clamp(0.0, 0.05);
        *last = time;
        for (k, c) in ch.iter_mut().enumerate() {
            let g = target(k);
            c[0] = g[0];
            for j in 1..P {
                let stiff = 14.0 - 10.0 * j as f32 / P as f32;
                let cj = c[j];
                c[j] = cj + (g[j] - cj) * (1.0 - (-dt * stiff).exp());
            }
        }
        ch.clone()
    });
    let width = |u: f32| 6.0 + 5.0 * u;
    let (Some(paint), Some(light)) = (shader(SILK, false), shader(SILK, true)) else { return };
    let set = |m: &Material| colours(m, t, vec3(0.98, 0.78, 0.84), vec3(0.88, 0.42, 0.66), vec3(1.0, 0.95, 0.8));
    gl_use_material(&light);
    set(&light);
    for c in &sashes {
        draw_mesh(&ribbon(c, |u| width(u) * 2.0, Color::new(1.0, 0.8, 0.7, 0.1)));
    }
    gl_use_default_material();
    gl_use_material(&paint);
    set(&paint);
    for c in &sashes {
        draw_mesh(&ribbon(c, width, WHITE));
    }
    gl_use_default_material();
}

/// The night sky of a domain: slow nebula clouds (domain-warped noise) turning, a field of
/// twinkling stars, a bright horizon at its edge. Drawn on a square, cut to the disc.
const SKY: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying highp vec2 uv;
uniform float Time;
uniform vec3 Base;
uniform vec3 Mid;
uniform vec3 Tip;
float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0)), f.x), f.y);
}
float fbm(vec2 p) {
    return vnoise(p) * 0.5 + vnoise(p * 2.1 + 5.2) * 0.3 + vnoise(p * 4.3 + 1.7) * 0.2;
}
void main() {
    vec2 p = uv * 2.0 - 1.0;
    float d = length(p);
    float ang = Time * 0.04;
    vec2 q = mat2(cos(ang), -sin(ang), sin(ang), cos(ang)) * p * 2.6;
    vec2 w = vec2(fbm(q + Time * 0.05), fbm(q * 1.3 - Time * 0.04 + 3.0));
    float n = clamp((fbm(q + w * 1.6) - 0.5) * 2.0 + 0.5, 0.0, 1.0);
    vec3 c = mix(Base, Mid, smoothstep(0.2, 0.65, n)) * 1.5;
    c = mix(c, Tip * 0.7, smoothstep(0.6, 0.95, n) * w.x * 0.8);
    vec2 g = p * 55.0;
    vec2 f = fract(g) - 0.5;
    float h = hash(floor(g));
    float star = step(0.9, h) * smoothstep(0.22, 0.0, length(f)) * (0.55 + 0.45 * sin(Time * 3.0 + h * 50.0));
    c += vec3(0.88, 0.94, 1.0) * star * (h - 0.88) * 9.0;
    float rim = smoothstep(0.84, 0.99, d);
    c += vec3(0.55, 0.75, 1.0) * rim * rim * 0.8;
    float a = (1.0 - smoothstep(0.98, 1.0, d)) * (0.84 + 0.16 * rim);
    gl_FragColor = vec4(c * color.rgb, a * color.a);
}
"#;

/// The Firmament's sky over the domain at `c`, radius `r`, `open` 0..1.
pub fn draw_sky(c: Vec2, r: f32, time: f64, open: f32) {
    let Some(m) = shader(SKY, false) else { return };
    gl_use_material(&m);
    colours(&m, time as f32, vec3(0.02, 0.03, 0.1), vec3(0.36, 0.18, 0.66), vec3(0.3, 0.7, 1.0));
    draw_mesh(&quad(c, r, Color::new(1.0, 1.0, 1.0, open)));
    gl_use_default_material();
}
