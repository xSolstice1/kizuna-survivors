//! What the fight leaves behind: enemies burning away when they fall, and scorch marks where
//! blasts land.
//!
//! - **Dissolve**: a fallen enemy's sprite burns away over `DISSOLVE` seconds along a noise
//!   pattern, its burning edge glowing in the kill's colour (shader below; the progress rides
//!   in the vertex alpha, so every dying enemy is one batch). Drawn before the light map, so
//!   it's lit like the living.
//! - **Scorches**: dark soft marks on the ground under blasts, glowing hot for a moment (a
//!   passing light) and fading over `SCORCH` seconds. Drawn on the ground, under props.
use std::cell::OnceCell;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use sim::content::{EnemyKind, Rank};

/// Seconds an enemy takes to burn away, and a scorch mark to fade.
pub const DISSOLVE: f32 = 0.42;
pub const SCORCH: f32 = 7.0;
/// Most at once (a crowd wiped by one blast doesn't need hundreds).
pub const MAX_DYING: usize = 120;
pub const MAX_SCORCHES: usize = 48;

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

const FRAGMENT: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying highp vec2 uv;
uniform sampler2D Texture;
float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0)), f.x), f.y);
}
void main() {
    vec4 t = texture2D(Texture, uv);
    // Progress 0..1 rides in the vertex alpha; the edge colour in its rgb.
    float k = color.a;
    float n = vnoise(uv * 7.0) * 0.6 + vnoise(uv * 19.0) * 0.4;
    // The top burns first, the feet last.
    n = n * 0.75 + uv.y * 0.25;
    float cut = k * 1.12;
    float alive = step(cut, n);
    float edge = 1.0 - clamp((n - cut) / 0.09, 0.0, 1.0);
    // Charring as it goes, a glowing rim at the burn line.
    vec3 body = t.rgb * (1.0 - 0.55 * k);
    vec3 c = mix(body, color.rgb * 1.8, edge * step(0.01, k));
    gl_FragColor = vec4(c, t.a * alive);
}
"#;

thread_local! {
    static MATERIAL: OnceCell<Option<Material>> = const { OnceCell::new() };
}

fn material() -> Option<Material> {
    MATERIAL.with(|m| {
        m.get_or_init(|| {
            load_material(
                ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
                MaterialParams {
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
                        alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
                .map_err(|e| {
                    warn!("dissolve material: {e:?}");
                })
                .ok()
        })
        .clone()
    })
}

/// How big an enemy's sprite draws (bosses at 2x), from its natural size.
fn size(kind: EnemyKind, nat: Vec2) -> Vec2 {
    nat * match kind.rank() {
        Rank::Mini | Rank::Boss => 2.0,
        Rank::Final => 2.2,
        _ => 1.0,
    }
}

/// Every enemy still burning away in `view`. `dying` holds (position, kind, edge colour, age).
pub fn draw_dying(dying: &[(Vec2, EnemyKind, Color, f32)], view: Rect, tex: &dyn Fn(EnemyKind) -> Texture2D) {
    if dying.is_empty() {
        return;
    }
    let Some(mat) = material() else { return };
    gl_use_material(&mat);
    for &(p, kind, c, age) in dying {
        if p.x < view.x - 60.0 || p.x > view.x + view.w + 60.0 || p.y < view.y - 60.0 || p.y > view.y + view.h + 60.0 {
            continue;
        }
        let t = tex(kind);
        let s = size(kind, crate::sprites::nat(&t));
        let feet = vec2(p.x, p.y + s.y * 0.25);
        let k = (age / DISSOLVE).clamp(0.0, 1.0);
        // Rising a touch as it burns.
        let y = feet.y - s.y - k * 3.0;
        let flip = (p.x * 7.3 + p.y * 3.1).sin() > 0.0;
        draw_texture_ex(&t, feet.x - s.x / 2.0, y, Color::new(c.r, c.g, c.b, k.max(0.004)), DrawTextureParams { dest_size: Some(s), flip_x: flip, ..Default::default() });
    }
    gl_use_default_material();
}

/// Scorch marks: (position, radius, age). `glow` is a soft round sprite.
pub fn draw_scorches(scorches: &[(Vec2, f32, f32)], view: Rect, glow: &Texture2D) {
    for &(p, r, age) in scorches {
        if p.x < view.x - r * 1.5 || p.x > view.x + view.w + r * 1.5 || p.y < view.y - r * 1.5 || p.y > view.y + view.h + r * 1.5 {
            continue;
        }
        let fade = 1.0 - (age / SCORCH).clamp(0.0, 1.0);
        let fade = fade * fade.sqrt();
        // Flattened onto the ground (seen at a slant), a dark wide mark with a darker heart.
        for (scale, a) in [(1.25, 0.5), (0.7, 0.45), (0.38, 0.4)] {
            let w = r * scale * 2.0;
            draw_texture_ex(glow, p.x - w / 2.0, p.y - w * 0.36, Color::new(0.04, 0.025, 0.02, a * fade), DrawTextureParams { dest_size: Some(vec2(w, w * 0.72)), ..Default::default() });
        }
    }
}

/// The scorches' heat: glowing embers for their first moments (passing lights).
pub fn scorch_lights(scorches: &[(Vec2, f32, f32)], l: &mut crate::light::Lights) {
    for &(p, r, age) in scorches {
        let hot = (1.0 - age / 1.6).max(0.0);
        if hot > 0.0 {
            let k = hot * hot;
            l.add_own(p, r * 1.8, Color::new(0.30 * k, 0.12 * k, 0.03 * k, 1.0));
        }
    }
}
