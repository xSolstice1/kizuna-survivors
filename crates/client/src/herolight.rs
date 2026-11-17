//! The hero, lit as one figure.
//!
//! A rigged hero is a dozen parts (hair, arms, torso, legs, gear). Lighting each part on its
//! own puts rims and shadows along every seam inside the body, which reads as wrong. So the
//! posed hero is first drawn into an off-screen image at screen resolution (premultiplied
//! alpha, so overlapping parts merge cleanly), and that one image is lit:
//!
//! - a soft **rim** on the outer silhouette facing the light (just inside the art's outline,
//!   so it reads as light, not a coloured border), in the moon's or the nearest lamp's colour;
//! - a **core shadow** along the far side;
//! - **volume** top to bottom: lit from above, darker towards the feet, with a little
//!   occlusion where they meet the ground;
//! - the world's **colour grade** (`post::grade`), so the hero sits in the scene instead of
//!   being pasted over it, while staying sharp (drawn after the lens, at full resolution).
use std::cell::RefCell;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

/// The image's box around the feet (world units): wide enough for weapons and the Arsenal,
/// tall enough for hair and hats.
const W: f32 = 64.0;
const ABOVE: f32 = 56.0;
const BELOW: f32 = 8.0;

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

/// Drawing the parts into the image: premultiplied, alpha accumulating (parts over parts).
const PLAIN: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying highp vec2 uv;
uniform sampler2D Texture;
void main() {
    vec4 t = texture2D(Texture, uv) * color;
    gl_FragColor = vec4(t.rgb * t.a, t.a);
}
"#;

const LIGHT: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying highp vec2 uv;
uniform sampler2D Texture;
// One world unit in uv; towards the light in uv (the image's v runs up).
uniform vec2 Step;
uniform vec2 Light;
uniform vec4 Rim;
uniform vec4 Shade;
// The feet's v; the grade (as post.rs): lift, gain, x saturation, y contrast.
uniform float Feet;
uniform vec3 Lift;
uniform vec3 Gain;
uniform vec2 Look;
float A(vec2 o) { return texture2D(Texture, uv + o).a; }
float sm(float t) { t = clamp(t, 0.0, 1.0); return t * t * (3.0 - 2.0 * t); }
void main() {
    vec4 t = texture2D(Texture, uv);
    if (t.a < 0.004) { gl_FragColor = vec4(0.0); return; }
    vec3 c = t.rgb / t.a;
    vec2 l = Light * Step;
    // Rim: within about a unit of the silhouette's edge facing the light, but not on the
    // outermost sliver (the art's own outline).
    float out1 = 1.0 - 0.5 * (A(l * 0.9) + A(l * 1.6));
    float rim = clamp(out1, 0.0, 1.0) * sm(A(l * 0.35) * 1.4 - 0.2);
    // Core shadow: the far side.
    float dark = clamp(1.0 - 0.5 * (A(-l * 1.4) + A(-l * 2.8)), 0.0, 1.0);
    // Volume: lit from above, darker towards the feet, occlusion right at them.
    float h = clamp((uv.y - Feet) / (1.0 - Feet) * 2.0, 0.0, 1.0);
    c *= 0.84 + 0.22 * h;
    c *= 1.0 - 0.22 * (1.0 - sm(h / 0.12));
    c = mix(c, c * Shade.rgb, dark * Shade.a);
    c += Rim.rgb * rim * Rim.a * 1.5 * (0.5 + 0.5 * dot(c, vec3(0.333)));
    // The world's grade.
    c = c * Gain + Lift * (1.0 - c);
    float g = dot(c, vec3(0.299, 0.587, 0.114));
    c = mix(vec3(g), c, Look.x);
    c = (c - 0.5) * Look.y + 0.5;
    c = clamp(c, 0.0, 1.0);
    gl_FragColor = vec4(c * t.a, t.a) * color.a;
}
"#;

struct State {
    rt: Option<(RenderTarget, u32, u32)>,
    plain: Option<Material>,
    light: Option<Material>,
    failed: bool,
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State { rt: None, plain: None, light: None, failed: false }) };
}

fn premultiplied(src: BlendFactor) -> PipelineParams {
    PipelineParams {
        color_blend: Some(BlendState::new(Equation::Add, src, BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
        alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
        ..Default::default()
    }
}

/// Draws the posed hero at `feet` lit by `light`, graded for `stage`. `cam` is the camera
/// the hero is drawn with (it sets the image's resolution: one texel per screen pixel).
#[allow(clippy::too_many_arguments)]
pub fn draw(def: &crate::rig::RigDef, st: &crate::rig::RigState, feet: Vec2, left: bool, tint: Color, light: &crate::litsprite::Light, cam: &Camera2D, stage: sim::campaign::StageId) {
    let lit = crate::post::quality() >= crate::post::Quality::Medium;
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if !lit || s.failed {
            set_camera(cam);
            crate::rig::draw(def, st, feet, left, tint);
            return;
        }
        if s.plain.is_none() {
            s.plain = load_material(ShaderSource::Glsl { vertex: VERTEX, fragment: PLAIN }, MaterialParams { pipeline_params: premultiplied(BlendFactor::One), ..Default::default() }).ok();
            let u = |n: &str, t| UniformDesc::new(n, t);
            s.light = load_material(
                ShaderSource::Glsl { vertex: VERTEX, fragment: LIGHT },
                MaterialParams {
                    pipeline_params: premultiplied(BlendFactor::One),
                    uniforms: vec![
                        u("Step", UniformType::Float2),
                        u("Light", UniformType::Float2),
                        u("Rim", UniformType::Float4),
                        u("Shade", UniformType::Float4),
                        u("Feet", UniformType::Float1),
                        u("Lift", UniformType::Float3),
                        u("Gain", UniformType::Float3),
                        u("Look", UniformType::Float2),
                    ],
                    ..Default::default()
                },
            )
            .map_err(|e| {
                warn!("hero light material: {e:?}");
            })
            .ok();
            if s.plain.is_none() || s.light.is_none() {
                s.failed = true;
                set_camera(cam);
                crate::rig::draw(def, st, feet, left, tint);
                return;
            }
        }
        // One texel per device pixel at this camera's zoom (capped).
        let ppu = (cam.zoom.x.abs() * screen_width() * screen_dpi_scale() / 2.0).clamp(1.0, 8.0);
        let (w, h) = ((W * ppu).ceil() as u32, ((ABOVE + BELOW) * ppu).ceil() as u32);
        if s.rt.as_ref().is_none_or(|(_, rw, rh)| *rw != w || *rh != h) {
            let rt = render_target(w, h);
            rt.texture.set_filter(FilterMode::Linear);
            s.rt = Some((rt, w, h));
        }
        let rt = s.rt.as_ref().unwrap().0.clone();
        let (plain, mat) = (s.plain.clone().unwrap(), s.light.clone().unwrap());
        drop(s);
        let bx = Rect::new(feet.x - W / 2.0, feet.y - ABOVE, W, ABOVE + BELOW);
        let mut rc = Camera2D::from_display_rect(bx);
        rc.render_target = Some(rt.clone());
        set_camera(&rc);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        gl_use_material(&plain);
        crate::rig::draw(def, st, feet, left, tint);
        gl_use_default_material();

        set_camera(cam);
        let (lift, gain, saturation, contrast) = crate::post::grade(stage);
        let d = light.dir.normalize_or(vec2(-0.8, -0.6));
        mat.set_uniform("Step", vec2(1.0 / W, 1.0 / (ABOVE + BELOW)));
        // The image's v runs up (its top is v = 1), so the light's y flips.
        mat.set_uniform("Light", vec2(d.x, -d.y));
        mat.set_uniform("Rim", vec4(light.rim.r, light.rim.g, light.rim.b, light.rim.a));
        mat.set_uniform("Shade", vec4(light.shade.r, light.shade.g, light.shade.b, light.shade.a));
        mat.set_uniform("Feet", BELOW / (ABOVE + BELOW));
        mat.set_uniform("Lift", lift);
        mat.set_uniform("Gain", gain);
        mat.set_uniform("Look", vec2(saturation, contrast));
        gl_use_material(&mat);
        draw_texture_ex(&rt.texture, bx.x, bx.y, WHITE, DrawTextureParams { dest_size: Some(vec2(bx.w, bx.h)), flip_y: true, ..Default::default() });
        gl_use_default_material();
    });
}
