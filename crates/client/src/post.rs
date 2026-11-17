//! The lens: what happens to the world's picture on its way to the screen.
//!
//! The world is drawn into a target (`main.rs`), then `present` lays it on the screen through
//! one full-screen shader, with:
//! - **bloom**: bright pixels, blurred at a quarter and an eighth of the resolution, added back,
//!   so lanterns, spells and neon glow through the air around them;
//! - **tone mapping**: a filmic shoulder, so brights roll off into white instead of clipping;
//! - **colour grading** per world (`look`): lift (shadows' tint), gain (highlights'),
//!   saturation and contrast;
//! - **tilt-shift**: the top and bottom of the view gently out of focus (a diorama's depth);
//! - **heat haze**: the picture shimmers over fire (`heat` marks where, each frame);
//! - lens touches: a vignette, a little chromatic aberration towards the corners (more on a
//!   heavy hit) and fine film grain.
//!
//! How much of it runs is the graphics quality (`quality`): Low grades and vignettes only,
//! Medium adds bloom, High everything. Auto (the default) starts High and steps down when
//! frames run slow, so phones stay smooth.
//!
//! Every pass draws a unit quad through a vertex shader that ignores the camera, so each
//! target's rows keep the world target's order (row 0 = the top of the picture) and nothing
//! ever comes out upside down.
use std::cell::RefCell;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use sim::campaign::StageId;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Quality {
    Low,
    Medium,
    High,
}

/// The graphics setting as saved (`Profile::graphics`): 0 auto, 1 low, 2 medium, 3 high.
pub fn setting_label(v: u8) -> &'static str {
    match v {
        1 => "Low",
        2 => "Medium",
        3 => "High",
        _ => "Auto",
    }
}

/// A world's colour grade.
#[derive(Clone, Copy)]
struct Look {
    /// Added to the darks (their tint), and the brights' multiplier.
    lift: Vec3,
    gain: Vec3,
    saturation: f32,
    contrast: f32,
    bloom: f32,
    vignette: f32,
    /// How bright a pixel must be to bloom (bright worlds, snow, need more).
    threshold: f32,
}

fn look(stage: StageId) -> Look {
    let l = |lift: [f32; 3], gain: [f32; 3], saturation, contrast, bloom, vignette| Look { lift: Vec3::from(lift), gain: Vec3::from(gain), saturation, contrast, bloom, vignette, threshold: 0.70 };
    match stage {
        // Moonlit forest: cool blue-green shadows, warm lantern highlights.
        StageId::ShrineValley => l([0.010, 0.022, 0.040], [1.04, 1.01, 0.95], 1.08, 1.06, 0.55, 0.42),
        // Neon street: magenta-violet shadows, punchy colour.
        StageId::Shotengai => l([0.030, 0.006, 0.045], [1.03, 0.98, 1.03], 1.14, 1.07, 0.70, 0.45),
        // Burning wood: ember-warm throughout.
        StageId::EmberLeaf => l([0.035, 0.012, 0.004], [1.06, 0.98, 0.88], 1.10, 1.07, 0.60, 0.45),
        // Snow citadel: cold, clean, a little desaturated.
        StageId::PaleCitadel => Look { threshold: 0.9, ..l([0.004, 0.014, 0.040], [0.97, 0.99, 1.03], 0.94, 1.05, 0.35, 0.36) },
        // Rain city: teal shadows, sodium-orange lamps.
        StageId::Rainward => l([0.000, 0.022, 0.035], [1.04, 1.00, 0.94], 0.92, 1.08, 0.65, 0.48),
        // Shattered sky: violet.
        StageId::Shatterline => l([0.026, 0.004, 0.050], [1.02, 0.97, 1.05], 1.06, 1.06, 0.60, 0.44),
    }
}

const PASS_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec4 color;
varying highp vec2 uv;
uniform float FlipOut;
void main() {
    // A unit quad straight to clip space (the camera is ignored): row 0 of the source stays
    // row 0 of a target; on screen (FlipOut -1) it's the top.
    gl_Position = vec4(position.x * 2.0 - 1.0, (position.y * 2.0 - 1.0) * FlipOut, 0.0, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const WORLD_VERTEX: &str = r#"#version 100
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
const ADD_FRAGMENT: &str = r#"#version 100
precision lowp float;
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = texture2D(Texture, uv) * color;
}
"#;

const HEADER: &str = "#version 100\nprecision mediump float;\nvarying lowp vec4 color;\nvarying highp vec2 uv;\nuniform sampler2D Texture;\nfloat sm(float t) { t = clamp(t, 0.0, 1.0); return t * t * (3.0 - 2.0 * t); }\n";

/// Downsample with a soft threshold: only what's bright goes on to bloom.
const BRIGHT: &str = r#"
uniform vec2 Texel;
uniform float Threshold;
void main() {
    vec3 s = texture2D(Texture, uv + Texel * vec2(-1.0, -1.0)).rgb + texture2D(Texture, uv + Texel * vec2(1.0, -1.0)).rgb
        + texture2D(Texture, uv + Texel * vec2(-1.0, 1.0)).rgb + texture2D(Texture, uv + Texel * vec2(1.0, 1.0)).rgb;
    s *= 0.25;
    float l = max(s.r, max(s.g, s.b));
    gl_FragColor = vec4(s * sm((l - Threshold) / 0.35), 1.0) * color;
}
"#;

/// A plain 4-tap downsample.
const DOWN: &str = r#"
uniform vec2 Texel;
void main() {
    vec3 s = texture2D(Texture, uv + Texel * vec2(-1.0, -1.0)).rgb + texture2D(Texture, uv + Texel * vec2(1.0, -1.0)).rgb
        + texture2D(Texture, uv + Texel * vec2(-1.0, 1.0)).rgb + texture2D(Texture, uv + Texel * vec2(1.0, 1.0)).rgb;
    gl_FragColor = vec4(s * 0.25, 1.0) * color;
}
"#;

/// One direction of a 9-tap gaussian (5 reads, using the filtering between texels).
const BLUR: &str = r#"
uniform vec2 Step;
void main() {
    vec3 s = texture2D(Texture, uv).rgb * 0.2270270;
    s += (texture2D(Texture, uv + Step * 1.3846154).rgb + texture2D(Texture, uv - Step * 1.3846154).rgb) * 0.3162162;
    s += (texture2D(Texture, uv + Step * 3.2307692).rgb + texture2D(Texture, uv - Step * 3.2307692).rgb) * 0.0702703;
    gl_FragColor = vec4(s, 1.0) * color;
}
"#;

const COMPOSITE: &str = r#"
uniform sampler2D Bloom1;
uniform sampler2D Bloom2;
uniform sampler2D Soft;
uniform sampler2D Heat;
uniform vec3 Lift;
uniform vec3 Gain;
// x saturation, y contrast, z bloom, w vignette
uniform vec4 Look;
// x time, y grain, z tilt-shift, w aberration
uniform vec4 Fx;
uniform vec2 Res;
// A filmic shoulder: the picture is already display-ready, so below the knee nothing changes;
// above it, brights (bloom's overflow) roll smoothly into white instead of clipping.
vec3 shoulder(vec3 x) {
    vec3 k = vec3(0.72);
    vec3 o = max(x - k, 0.0);
    return min(x, k) + (1.0 - k) * (1.0 - exp(-o / (1.0 - k)));
}
void main() {
    vec2 p = uv;
    // Heat haze: the picture wavers where it's hot.
    float h = texture2D(Heat, uv).r;
    p += vec2(sin(uv.y * 160.0 + Fx.x * 7.0), cos(uv.x * 120.0 + Fx.x * 5.3)) * h * 0.0035;
    // Chromatic aberration, growing towards the corners.
    vec2 d = p - 0.5;
    vec2 ca = d * dot(d, d) * Fx.w;
    vec3 c = vec3(texture2D(Texture, p + ca).r, texture2D(Texture, p).g, texture2D(Texture, p - ca).b);
    // Tilt-shift: the top and bottom soften.
    float t = sm((abs(uv.y - 0.5) - 0.2) / 0.3) * Fx.z;
    c = mix(c, texture2D(Soft, p).rgb, t);
    // Bloom, tight and wide.
    // Bounded (a screen full of spells glows, it doesn't white out), and weaker where the
    // picture is already bright.
    vec3 b = texture2D(Bloom1, p).rgb * 0.45 + texture2D(Bloom2, p).rgb * 0.55;
    b = b / (1.0 + 1.5 * b);
    c += b * Look.z * (1.0 - 0.6 * dot(c, vec3(0.333)));
    c = shoulder(c);
    // Grade: tint the darks, scale the brights, then saturation and contrast.
    c = c * Gain + Lift * (1.0 - c);
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    c = mix(vec3(l), c, Look.x);
    c = (c - 0.5) * Look.y + 0.5;
    // Vignette.
    vec2 v = (uv - 0.5) * vec2(2.0, 1.7);
    c *= 1.0 - Look.w * sm((length(v) - 0.6) / 0.85);
    // Film grain, finer than a pixel's worth of colour.
    float n = fract(sin(dot(floor(uv * Res) + fract(Fx.x * 7.13) * 91.0, vec2(12.9898, 78.233))) * 43758.5453) - 0.5;
    c += n * Fx.y;
    gl_FragColor = vec4(clamp(c, 0.0, 1.0), 1.0);
}
"#;

struct Targets {
    size: (u32, u32),
    /// Quarter: bloom (a, b), the soft picture for tilt-shift (e, f), heat; eighth: bloom (c, d).
    a: RenderTarget,
    b: RenderTarget,
    c: RenderTarget,
    d: RenderTarget,
    e: RenderTarget,
    f: RenderTarget,
    heat: RenderTarget,
}

struct Post {
    targets: Option<Targets>,
    bright: Option<Material>,
    down: Option<Material>,
    blur: Option<Material>,
    composite: Option<Material>,
    /// Additive, through the world's camera (the heat map).
    add: Option<Material>,
    black: Option<Texture2D>,
    failed: bool,
    /// The setting (0 auto) and what's running.
    setting: u8,
    auto: Quality,
    /// Auto: smoothed frame time, and how long it's been too slow.
    frame_ms: f32,
    slow_for: f32,
    /// Heat sources this frame: world position, radius, strength.
    heat: Vec<(Vec2, f32, f32)>,
    /// A recent heavy hit (0..1, fades): the lens shudders.
    punch: f32,
}

thread_local! {
    static POST: RefCell<Post> = RefCell::new(Post {
        targets: None, bright: None, down: None, blur: None, composite: None, add: None, black: None, failed: false,
        setting: 0, auto: Quality::High, frame_ms: 16.0, slow_for: 0.0, heat: Vec::new(), punch: 0.0,
    });
}

/// The quality running now.
pub fn quality() -> Quality {
    POST.with(|p| {
        let p = p.borrow();
        match p.setting {
            1 => Quality::Low,
            2 => Quality::Medium,
            3 => Quality::High,
            _ => p.auto,
        }
    })
}

/// Applies the saved setting (`Profile::graphics`).
pub fn set_setting(v: u8) {
    POST.with(|p| p.borrow_mut().setting = v);
}

/// Called once a frame in a run (`dt` the frame's seconds). On Auto, frames running slower
/// than ~45 fps for three seconds step the quality down (never back up in a session: no
/// flip-flopping). `judge` false (menus, the dev harness) leaves it alone.
pub fn frame(dt: f32, judge: bool) {
    POST.with(|p| {
        let mut p = p.borrow_mut();
        p.punch = (p.punch - dt * 2.5).max(0.0);
        if !judge || p.setting != 0 || dt <= 0.0 || dt > 0.25 {
            return;
        }
        p.frame_ms += (dt * 1000.0 - p.frame_ms) * (dt * 2.0).min(1.0);
        p.slow_for = if p.frame_ms > 22.0 { p.slow_for + dt } else { 0.0 };
        if p.slow_for > 3.0 && p.auto > Quality::Low {
            p.auto = if p.auto == Quality::High { Quality::Medium } else { Quality::Low };
            p.slow_for = 0.0;
            p.frame_ms = 16.0;
            info!("graphics: auto quality down to {:?}", p.auto);
        }
    });
}

/// Marks heat at `p` (world units) this frame: the air above it wavers. `k` 0..1.
pub fn heat(p: Vec2, radius: f32, k: f32) {
    POST.with(|post| {
        let mut post = post.borrow_mut();
        if post.heat.len() < 64 {
            post.heat.push((p, radius, k));
        }
    });
}

/// A heavy hit landed (the lens shudders for a moment).
pub fn punch(k: f32) {
    POST.with(|p| {
        let mut p = p.borrow_mut();
        p.punch = p.punch.max(k.min(1.0));
    });
}

fn material(fragment: &str, uniforms: Vec<UniformDesc>, textures: Vec<String>) -> Option<Material> {
    let src = format!("{HEADER}{fragment}");
    load_material(
        ShaderSource::Glsl { vertex: PASS_VERTEX, fragment: &src },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::Zero)),
                alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::Zero)),
                ..Default::default()
            },
            uniforms: [vec![UniformDesc::new("FlipOut", UniformType::Float1)], uniforms].concat(),
            textures,
        },
    )
    .map_err(|e| {
        warn!("post material: {e:?}");
    })
    .ok()
}

fn target(w: u32, h: u32) -> RenderTarget {
    let rt = render_target(w.max(1), h.max(1));
    rt.texture.set_filter(FilterMode::Linear);
    rt
}

/// Draws `src` through `mat` into `dst` (whole), row order kept.
fn pass(dst: &RenderTarget, src: &Texture2D, mat: &Material) {
    let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, 1.0, 1.0));
    cam.render_target = Some(dst.clone());
    set_camera(&cam);
    mat.set_uniform("FlipOut", 1.0f32);
    gl_use_material(mat);
    draw_texture_ex(src, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(1.0, 1.0)), ..Default::default() });
    gl_use_default_material();
}

/// Lays the world's picture (`world`, drawn with `cam`) on the screen through the lens.
/// `glow` is a soft round sprite (for the heat map). Leaves the default camera set.
pub fn present(world: &Texture2D, cam: &Camera2D, stage: StageId, time: f64, glow: &Texture2D) {
    let q = quality();
    POST.with(|p| {
        let mut p = p.borrow_mut();
        let heat = std::mem::take(&mut p.heat);
        let punch = p.punch;
        if p.failed {
            set_default_camera();
            draw_texture_ex(world, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(screen_width(), screen_height())), ..Default::default() });
            return;
        }
        if p.composite.is_none() {
            let f2 = |n: &str| UniformDesc::new(n, UniformType::Float2);
            p.bright = material(BRIGHT, vec![f2("Texel"), UniformDesc::new("Threshold", UniformType::Float1)], vec![]);
            p.down = material(DOWN, vec![f2("Texel")], vec![]);
            p.blur = material(BLUR, vec![f2("Step")], vec![]);
            p.composite = material(
                COMPOSITE,
                vec![
                    UniformDesc::new("Lift", UniformType::Float3),
                    UniformDesc::new("Gain", UniformType::Float3),
                    UniformDesc::new("Look", UniformType::Float4),
                    UniformDesc::new("Fx", UniformType::Float4),
                    f2("Res"),
                ],
                ["Bloom1", "Bloom2", "Soft", "Heat"].iter().map(|s| s.to_string()).collect(),
            );
            p.add = load_material(
                ShaderSource::Glsl { vertex: WORLD_VERTEX, fragment: ADD_FRAGMENT },
                MaterialParams {
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::One)),
                        alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .map_err(|e| {
                warn!("post add material: {e:?}");
            })
            .ok();
            p.black = Some(Texture2D::from_rgba8(1, 1, &[0, 0, 0, 255]));
            if p.add.is_none() || p.bright.is_none() || p.down.is_none() || p.blur.is_none() || p.composite.is_none() {
                p.failed = true;
                drop(p);
                return present(world, cam, stage, time, glow);
            }
        }
        let (w, h) = (world.width() as u32, world.height() as u32);
        let (qw, qh) = ((w / 4).max(1), (h / 4).max(1));
        if q > Quality::Low && p.targets.as_ref().is_none_or(|t| t.size != (w, h)) {
            let (ew, eh) = ((w / 8).max(1), (h / 8).max(1));
            p.targets = Some(Targets { size: (w, h), a: target(qw, qh), b: target(qw, qh), c: target(ew, eh), d: target(ew, eh), e: target(qw, qh), f: target(qw, qh), heat: target(qw, qh) });
        }
        let lk = look(stage);
        let black = p.black.clone().unwrap();
        let (bright, down, blur, comp, add) = (p.bright.clone().unwrap(), p.down.clone().unwrap(), p.blur.clone().unwrap(), p.composite.clone().unwrap(), p.add.clone().unwrap());
        let texel = |t: &Texture2D| vec2(1.0 / t.width(), 1.0 / t.height());
        let blur2 = |a: &RenderTarget, b: &RenderTarget| {
            blur.set_uniform("Step", vec2(texel(&a.texture).x, 0.0));
            pass(b, &a.texture, &blur);
            blur.set_uniform("Step", vec2(0.0, texel(&b.texture).y));
            pass(a, &b.texture, &blur);
        };
        let (mut b1, mut b2, mut soft, mut heat_tex) = (black.clone(), black.clone(), black.clone(), black.clone());
        let mut tilt = 0.0;
        if let (true, Some(t)) = (q > Quality::Low, p.targets.as_ref()) {
            // Bloom: bright pass to a quarter, blurred; on High a wider eighth too.
            bright.set_uniform("Texel", texel(world));
            bright.set_uniform("Threshold", lk.threshold);
            pass(&t.a, world, &bright);
            blur2(&t.a, &t.b);
            b1 = t.a.texture.clone();
            b2 = b1.clone();
            if q == Quality::High {
                down.set_uniform("Texel", texel(&t.a.texture));
                pass(&t.c, &t.a.texture, &down);
                blur2(&t.c, &t.d);
                b2 = t.c.texture.clone();
                // The soft picture for tilt-shift.
                down.set_uniform("Texel", texel(world));
                pass(&t.e, world, &down);
                blur2(&t.e, &t.f);
                soft = t.e.texture.clone();
                tilt = 0.75;
                // Heat: soft blobs where it's hot, drawn with the world's camera.
                if !heat.is_empty() {
                    let hc = Camera2D { target: cam.target, zoom: cam.zoom, offset: cam.offset, rotation: cam.rotation, render_target: Some(t.heat.clone()), viewport: None };
                    set_camera(&hc);
                    clear_background(BLACK);
                    gl_use_material(&add);
                    for (c, r, k) in &heat {
                        draw_texture_ex(glow, c.x - r, c.y - r * 1.4, Color::new(*k, 0.0, 0.0, 1.0), DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() });
                    }
                    gl_use_default_material();
                    heat_tex = t.heat.texture.clone();
                }
            }
        }
        set_default_camera();
        comp.set_uniform("FlipOut", -1.0f32);
        comp.set_uniform("Lift", lk.lift);
        comp.set_uniform("Gain", lk.gain);
        comp.set_uniform("Look", vec4(lk.saturation, lk.contrast, if q > Quality::Low { lk.bloom } else { 0.0 }, lk.vignette));
        let ab = if q == Quality::High { 0.012 + 0.05 * punch } else { 0.0 };
        comp.set_uniform("Fx", vec4((time % 1000.0) as f32, if q > Quality::Low { 0.022 } else { 0.0 }, tilt, ab));
        comp.set_uniform("Res", vec2(screen_width(), screen_height()));
        comp.set_texture("Bloom1", b1);
        comp.set_texture("Bloom2", b2);
        comp.set_texture("Soft", soft);
        comp.set_texture("Heat", heat_tex);
        gl_use_material(&comp);
        draw_texture_ex(world, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(1.0, 1.0)), ..Default::default() });
        gl_use_default_material();
    });
}
