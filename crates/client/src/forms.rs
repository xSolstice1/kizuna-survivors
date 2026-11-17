//! The transformations, drawn as set pieces: geometry and shaders instead of flat sprites.
//!
//! - **Nine-Tails** (Riku): nine fluffy fox tails fanned behind him, each a chain of points
//!   following a curled rest shape with stiffness falling towards the tip (so they swish, lag
//!   and trail when he runs), drawn as ribbons through a fur shader: strand-broken edges, a
//!   base-to-tip gradient from ember orange to a white-hot tip, a soft centre highlight. The
//!   tails' sweeping attack (`World::fox_tails`, what the sim hits with) shows as translucent
//!   foxfire lashes, so the hit is readable without looking like the body.
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

struct Fox {
    /// Each tail's chain, root first (world units).
    tails: Vec<Vec<Vec2>>,
    last: f64,
}

struct State {
    /// The fur, painted (tails) and added as light (spirit tails).
    fur: Option<Material>,
    glow: Option<Material>,
    failed: bool,
    fox: Option<Fox>,
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State { fur: None, glow: None, failed: false, fox: None }) };
}

const TAILS: usize = 9;
const POINTS: usize = 14;
/// A tail's length and greatest width (world units).
const LENGTH: f32 = 50.0;
const WIDTH: f32 = 13.0;

/// The fur material: painted, or (`light`) added as light.
fn fur(light: bool) -> Option<Material> {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.fur.is_none() && !s.failed {
            let u = |n: &str, t| UniformDesc::new(n, t);
            let make = |src: BlendFactor, dst: BlendFactor| {
                load_material(
                    ShaderSource::Glsl { vertex: VERTEX, fragment: FUR },
                    MaterialParams {
                        pipeline_params: PipelineParams {
                            color_blend: Some(BlendState::new(Equation::Add, src, dst)),
                            alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One)),
                            ..Default::default()
                        },
                        uniforms: vec![u("Time", UniformType::Float1), u("Base", UniformType::Float3), u("Mid", UniformType::Float3), u("Tip", UniformType::Float3)],
                        ..Default::default()
                    },
                )
                .map_err(|e| {
                    warn!("fur material: {e:?}");
                })
                .ok()
            };
            s.fur = make(BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha));
            s.glow = make(BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::One);
            s.failed = s.fur.is_none() || s.glow.is_none();
        }
        if light { s.glow.clone() } else { s.fur.clone() }
    })
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

/// Where tail `k`'s points rest, rooted at `root`: fanned behind (upwards on screen, a
/// little away from where Riku faces), curling outwards towards the tip, breathing.
fn rest(root: Vec2, k: usize, facing: f32, t: f32) -> [Vec2; POINTS] {
    let kf = k as f32 - (TAILS - 1) as f32 / 2.0; // -4 .. 4
    let spread = kf / 4.0; // -1 .. 1
    // Straight up is -pi/2; the fan leans away from the way he faces.
    let base = -std::f32::consts::FRAC_PI_2 + spread * 1.4 - facing * 0.25;
    let mut out = [root; POINTS];
    let mut p = root;
    // Each tail its own: a little longer or shorter, a little off the even fan.
    let h = |n: u32| ((k as u32 * 2654435761u32 ^ n).wrapping_mul(2246822519) >> 16) as f32 / 65535.0;
    let base = base + (h(1) - 0.5) * 0.16;
    let seg = LENGTH / (POINTS - 1) as f32 * (0.82 + 0.22 * (1.0 - spread.abs()) + 0.16 * h(2));
    for (j, o) in out.iter_mut().enumerate().skip(1) {
        let u = j as f32 / (POINTS - 1) as f32;
        // An S: out from the root, then curling back in at the tip, swaying as a wave runs
        // down it.
        let sway = (t * (1.8 + 0.6 * h(3)) + k as f32 * 1.7 - u * 3.0).sin() * (0.2 + 0.12 * h(4)) * u;
        let side = if spread.abs() < 0.05 { if k % 2 == 0 { 1.0 } else { -1.0 } } else { spread.signum() };
        let curl = side * ((0.35 + spread.abs() * 0.4) * u - (0.9 + spread.abs() * 0.6) * u * u * u);
        let a = base + curl + sway;
        p += vec2(a.cos(), a.sin()) * seg;
        *o = p;
    }
    out
}

/// Nine-Tails: tails behind, lashes where they strike, foxfire rising. `hips` is the tails'
/// root (world units), `facing` -1 left / 1 right, `strikes` the sim's tail segments
/// (root, tip). Draw before the hero (the tails are behind them).
pub fn draw_fox(hips: Vec2, facing: f32, strikes: &[(Vec2, Vec2)], time: f64, vfx: &crate::vfx::Vfx) {
    let t = time as f32;
    // The lashes are drawn with the tails below (ghostly tails where the sim's strike), and
    // the tips flare where they land.
    vfx.additive(|| {
        for (k, &(_, b)) in strikes.iter().enumerate() {
            let flick = 0.8 + 0.2 * (t * 14.0 + k as f32).sin();
            vfx.draw("glow", b, 20.0 * flick, 0.0, Color::new(1.0, 0.6, 0.2, 0.18));
        }
        // Foxfire rising round him, and a warm glow on the ground.
        vfx.draw_wh("glow", hips + vec2(0.0, 12.0), vec2(70.0, 34.0), 0.0, Color::new(1.0, 0.55, 0.15, 0.22));
        for k in 0..8 {
            let ph = (t * 0.9 + k as f32 * 0.137).fract();
            let ang = k as f32 * 0.785 + t * 0.4;
            let p = hips + vec2(ang.cos() * 15.0, 10.0 + ang.sin() * 5.0 - ph * 30.0);
            vfx.draw_wh("flame", p, vec2(7.0, 11.0) * (1.0 - ph * 0.6), 0.0, Color::new(1.0, 0.68, 0.3, 0.5 * (1.0 - ph)));
        }
    });
    crate::post::heat(hips + vec2(0.0, -10.0), 30.0, 0.5);

    // The tails, physically: each point eases towards its rest, stiff at the root and
    // loose at the tip, then the chain is held to its segment lengths.
    let (Some(mat), Some(glow)) = (fur(false), fur(true)) else { return };
    let tails = STATE.with(|s| {
        let mut s = s.borrow_mut();
        let fresh = s.fox.as_ref().is_none_or(|f| time - f.last > 0.5);
        if fresh {
            s.fox = Some(Fox { tails: (0..TAILS).map(|k| rest(hips, k, facing, t).to_vec()).collect(), last: time });
        }
        let fox = s.fox.as_mut().unwrap();
        let dt = ((time - fox.last) as f32).clamp(0.0, 0.05);
        fox.last = time;
        for (k, chain) in fox.tails.iter_mut().enumerate() {
            let goal = rest(hips, k, facing, t);
            chain[0] = hips;
            let seg = LENGTH / (POINTS - 1) as f32;
            for j in 1..POINTS {
                let stiff = 26.0 - 20.0 * j as f32 / POINTS as f32;
                let cj = chain[j];
                chain[j] = cj + (goal[j] - cj) * (1.0 - (-dt * stiff).exp());
                let d = chain[j] - chain[j - 1];
                let l = d.length().max(1e-3);
                chain[j] = chain[j - 1] + d / l * l.min(seg * 1.15).max(seg * 0.85);
            }
        }
        fox.tails.clone()
    });
    // Outer tails first (they're furthest back), the middle ones over them; the far ones
    // a shade darker.
    let mut order: Vec<usize> = (0..TAILS).collect();
    order.sort_by(|a, b| {
        let d = |k: usize| (k as f32 - (TAILS - 1) as f32 / 2.0).abs();
        d(*b).total_cmp(&d(*a))
    });
    gl_use_material(&mat);
    mat.set_uniform("Time", t);
    mat.set_uniform("Base", vec3(0.78, 0.26, 0.05));
    mat.set_uniform("Mid", vec3(1.0, 0.6, 0.16));
    mat.set_uniform("Tip", vec3(1.0, 0.95, 0.8));
    for k in order {
        let far = (k as f32 - (TAILS - 1) as f32 / 2.0).abs() / 4.0;
        let shade = 1.0 - 0.22 * far;
        // Thin at the root, fullest past the middle, tapering to the tip.
        let width = |u: f32| WIDTH * (0.25 + 0.95 * (u.powf(0.8) * std::f32::consts::PI).sin().max(0.0).powf(0.8)) * (1.0 - 0.15 * u);
        draw_mesh(&ribbon(&tails[k], width, Color::new(shade, shade, shade, 0.96)));
    }
    gl_use_default_material();
    // The strikes: spirit tails of light lashing out along the sim's tail segments, curving a
    // little against their sweep, so they read as the attack, not the body.
    gl_use_material(&glow);
    glow.set_uniform("Time", t);
    glow.set_uniform("Base", vec3(0.9, 0.35, 0.05));
    glow.set_uniform("Mid", vec3(1.0, 0.62, 0.18));
    glow.set_uniform("Tip", vec3(1.0, 0.9, 0.6));
    for (k, &(a, b)) in strikes.iter().enumerate() {
        let d = b - a;
        let len = d.length();
        if len < 4.0 {
            continue;
        }
        let side = vec2(-d.y, d.x) / len;
        let pts: Vec<Vec2> = (0..10)
            .map(|i| {
                let u = i as f32 / 9.0;
                a + d * (0.15 + 0.85 * u) + side * (u * (1.0 - u) * len * 0.18 * if k % 2 == 0 { 1.0 } else { -1.0 })
            })
            .collect();
        let width = |u: f32| 9.0 * (0.3 + (u * std::f32::consts::PI).sin().powf(0.7));
        draw_mesh(&ribbon(&pts, width, Color::new(1.0, 1.0, 1.0, 0.45)));
    }
    gl_use_default_material();
    // Foxfire: the tails glow, most at their tips.
    vfx.additive(|| {
        for (k, chain) in tails.iter().enumerate() {
            for (j, q) in chain.iter().enumerate().skip(POINTS / 2) {
                let u = j as f32 / (POINTS - 1) as f32;
                vfx.draw("glow", *q, 10.0 + 10.0 * u, 0.0, Color::new(1.0, 0.6, 0.2, 0.07 * u));
            }
            let tip = chain[POINTS - 1];
            let flick = 0.8 + 0.2 * (t * 11.0 + k as f32 * 2.3).sin();
            vfx.draw("glow", tip, 16.0 * flick, 0.0, Color::new(1.0, 0.85, 0.55, 0.3));
        }
    });
}

/// Hannya (Shun): a scorched ground with turning crimson cracks and a thin ring at the aura's
/// reach `r`, a huge hannya mask looming behind him, and tongues of crimson flame and dark
/// smoke rising round him (ribbons through the fur shader, so they flow and taper). `c` is
/// his centre (world units).
pub fn draw_hannya(c: Vec2, r: f32, time: f64, vfx: &crate::vfx::Vfx) {
    let t = time as f32;
    let pulse = 0.5 + 0.5 * (t * 5.0).sin();
    // Scorched ground, darker at the heart.
    vfx.draw_wh("glow", c + vec2(0.0, 8.0), vec2(r * 2.3, r * 1.4), 0.0, Color::new(0.08, 0.0, 0.02, 0.55));
    vfx.additive(|| {
        for k in 0..6 {
            let a = k as f32 * 1.047 + t * 0.15;
            vfx.draw_wh("crack", c + vec2(a.cos(), a.sin() * 0.55) * r * 0.45 + vec2(0.0, 8.0), vec2(r * 0.7, r * 0.35), a, Color::new(1.0, 0.15, 0.2, 0.22 + 0.1 * pulse));
        }
        vfx.draw_wh("ring", c + vec2(0.0, 8.0), vec2(r * 2.2, r * 1.3), 0.0, Color::new(1.0, 0.2, 0.25, 0.35));
        // The demon behind him: a red glow and the mask, breathing.
        let m = c + vec2(0.0, -34.0 - 2.0 * pulse);
        vfx.draw("glow", m, 80.0, 0.0, Color::new(0.6, 0.0, 0.08, 0.3 + 0.1 * pulse));
        vfx.draw("hannya", m, 62.0 + 4.0 * pulse, 0.0, Color::new(1.0, 0.45, 0.45, 0.32 + 0.08 * pulse));
    });
    crate::post::heat(c, 34.0, 0.6);
    let (Some(paint), Some(light)) = (fur(false), fur(true)) else { return };
    // Flame tongues rising round him, each a short chain waving as it climbs.
    let tongue = |k: usize, lift: f32| -> Vec<Vec2> {
        let kf = k as f32;
        let ph = (t * 0.9 + kf * 0.173).fract();
        let ang = kf * 2.39 + 0.4;
        let base = c + vec2(ang.cos() * (8.0 + (k % 3) as f32 * 7.0), 10.0 + ang.sin() * 5.0);
        (0..9)
            .map(|i| {
                let u = i as f32 / 8.0;
                let wave = (t * 6.0 + kf * 1.9 - u * 4.0).sin() * 5.0 * u;
                base + vec2(wave, -u * (24.0 + 10.0 * (1.0 - ph)) - lift)
            })
            .collect()
    };
    // Dark smoke behind the flames.
    gl_use_material(&paint);
    paint.set_uniform("Time", t);
    paint.set_uniform("Base", vec3(0.12, 0.0, 0.02));
    paint.set_uniform("Mid", vec3(0.2, 0.02, 0.05));
    paint.set_uniform("Tip", vec3(0.3, 0.08, 0.1));
    for k in 0..6 {
        draw_mesh(&ribbon(&tongue(k + 11, 6.0), |u| 10.0 * (1.0 - u * 0.6), Color::new(1.0, 1.0, 1.0, 0.45)));
    }
    gl_use_default_material();
    gl_use_material(&light);
    light.set_uniform("Time", t);
    light.set_uniform("Base", vec3(0.7, 0.02, 0.06));
    light.set_uniform("Mid", vec3(1.0, 0.18, 0.15));
    light.set_uniform("Tip", vec3(1.0, 0.6, 0.45));
    for k in 0..9 {
        draw_mesh(&ribbon(&tongue(k, 0.0), |u| 7.0 * (1.0 - u * 0.7) * (0.6 + 0.4 * (u * std::f32::consts::PI).sin()), Color::new(1.0, 1.0, 1.0, 0.7)));
    }
    gl_use_default_material();
}

/// Chitin along a ribbon: plated segments (`Segments` of them) with dark seams, a glossy
/// highlight down the spine, darker flanks.
const CHITIN: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying highp vec2 uv;
uniform float Segments;
uniform vec3 Shell;
uniform vec3 Gloss;
void main() {
    float across = abs(uv.x * 2.0 - 1.0);
    float seg = fract(uv.y * Segments);
    // Each plate rounded across and along; a dark seam between plates.
    float plate = 1.0 - across * across;
    float seam = smoothstep(0.0, 0.12, seg) * (1.0 - smoothstep(0.86, 1.0, seg));
    vec3 c = Shell * (0.45 + 0.55 * plate) * (0.35 + 0.65 * seam);
    // The plate's front edge catches light; a glossy line down the spine.
    c += Gloss * (1.0 - smoothstep(0.0, 0.25, seg)) * 0.35 * plate;
    c += Gloss * pow(1.0 - across, 6.0) * 0.6 * seam;
    float a = 1.0 - smoothstep(0.86, 1.0, across);
    gl_FragColor = vec4(c * color.rgb, a * color.a);
}
"#;

thread_local! {
    static CHITIN_MAT: std::cell::OnceCell<Option<Material>> = const { std::cell::OnceCell::new() };
    static MUKADE: RefCell<Option<(Vec<Vec2>, f64)>> = const { RefCell::new(None) };
}

fn chitin() -> Option<Material> {
    CHITIN_MAT.with(|m| {
        m.get_or_init(|| {
            let u = |n: &str, t| UniformDesc::new(n, t);
            load_material(
                ShaderSource::Glsl { vertex: VERTEX, fragment: CHITIN },
                MaterialParams {
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
                        alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
                        ..Default::default()
                    },
                    uniforms: vec![u("Segments", UniformType::Float1), u("Shell", UniformType::Float3), u("Gloss", UniformType::Float3)],
                    ..Default::default()
                },
            )
            .map_err(|e| {
                warn!("chitin material: {e:?}");
            })
            .ok()
        })
        .clone()
    })
}

/// Mukade (Toya): the great centipede tail arcing over his back, an armoured body with
/// scuttling legs and its head at the tip, following through as he moves; crimson tendrils
/// writhing at his back. `root` is where it grows from, `back` -1/1 the side behind him.
pub fn draw_mukade(root: Vec2, back: f32, time: f64, vfx: &crate::vfx::Vfx, head: &Texture2D) {
    use std::f32::consts::PI;
    let t = time as f32;
    const N: usize = 16;
    // Rest: an arch over his back, a wave running along it.
    let rest: Vec<Vec2> = (0..N)
        .map(|s| {
            let u = s as f32 / (N - 1) as f32;
            let ang = PI * (0.12 + 0.88 * u);
            let wave = (t * 4.0 - u * 6.0).sin() * 4.0;
            vec2(root.x + back * (ang.cos() * -36.0 - 8.0) + back * wave * 0.3, root.y - ang.sin() * 50.0 + u * 12.0 + wave * 0.6)
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
            let stiff = 30.0 - 22.0 * j as f32 / N as f32;
            let pj = pts[j];
            pts[j] = pj + (rest[j] - pj) * (1.0 - (-dt * stiff).exp());
        }
        pts.clone()
    });
    // Crimson tendrils writhing behind him (light, so they glow).
    if let Some(light) = fur(true) {
        gl_use_material(&light);
        light.set_uniform("Time", t);
        light.set_uniform("Base", vec3(0.5, 0.0, 0.05));
        light.set_uniform("Mid", vec3(0.9, 0.1, 0.15));
        light.set_uniform("Tip", vec3(1.0, 0.45, 0.4));
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
            draw_mesh(&ribbon(&pts, |u| 6.0 * (1.0 - u * 0.8), Color::new(1.0, 1.0, 1.0, 0.6)));
        }
        gl_use_default_material();
    }
    // Legs: a pair per plate, scuttling in a wave down the body.
    for (s, w) in body.windows(2).enumerate().skip(1) {
        let (a, b) = (w[0], w[1]);
        let d = (b - a).normalize_or(vec2(0.0, -1.0));
        let side = vec2(-d.y, d.x);
        let u = s as f32 / (N - 1) as f32;
        let size = 13.0 * (1.0 - u * 0.4);
        let step = (t * 14.0 - s as f32 * 0.9).sin();
        for sgn in [-1.0f32, 1.0] {
            let knee = a + side * sgn * size * 0.75 + d * step * 2.0 * sgn;
            let foot = knee + side * sgn * size * 0.3 + d * (3.0 + step * 2.5);
            crate::vfx::shape::line(a.x, a.y, knee.x, knee.y, 1.9, Color::new(0.2, 0.03, 0.04, 1.0));
            crate::vfx::shape::line(knee.x, knee.y, foot.x, foot.y, 1.5, Color::new(0.85, 0.3, 0.16, 1.0));
        }
    }
    if let Some(mat) = chitin() {
        gl_use_material(&mat);
        mat.set_uniform("Segments", (N - 1) as f32);
        mat.set_uniform("Shell", vec3(0.42, 0.06, 0.08));
        mat.set_uniform("Gloss", vec3(1.0, 0.5, 0.42));
        // Narrow where it grows from his back, full along the body, a little slimmer at the
        // neck.
        draw_mesh(&ribbon(&body, |u| 18.0 * (0.5 + 0.5 * (u * 4.0).min(1.0)) * (1.0 - u * 0.2), WHITE));
        gl_use_default_material();
    }
    // The head at the tip, facing along the body, with glowing eyes.
    let (a, b) = (body[N - 2], body[N - 1]);
    let d = b - a;
    let size = 24.0;
    draw_texture_ex(head, b.x - size / 2.0, b.y - size / 2.0, WHITE, DrawTextureParams { dest_size: Some(vec2(size, size)), rotation: d.y.atan2(d.x) + std::f32::consts::FRAC_PI_2, ..Default::default() });
    vfx.additive(|| vfx.draw("glow", b, 26.0, 0.0, Color::new(1.0, 0.15, 0.2, 0.3 + 0.1 * (t * 6.0).sin())));
}
