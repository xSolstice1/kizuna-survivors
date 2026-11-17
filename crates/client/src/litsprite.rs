//! Light on the characters: sprites drawn with volume instead of flat.
//!
//! A sprite has no depth, but its outline says where its edges are. The shader looks a step
//! towards the light: if that's outside the sprite, this pixel is an edge facing the light and
//! catches a **rim** of it (inside the outline, like a cel-shaded 3D model). A step away from
//! the light outside the sprite means an edge turned away: a soft **core shadow**. Together
//! the shape reads as round and lit from one side.
//!
//! Steps are in world units (`Step` = 1 / the sprite's size in units), so every sprite, big
//! or small, gets the same rim. A mirrored sprite (facing left) mirrors the light in texture
//! space; that rides in each vertex's z, so both facings share one draw call.
//!
//! Enemies are lit by the moon (cool, upper left); the hero also by the strongest lamp near
//! them (warm, from its side). See `Lights::rim_at`.
use std::cell::OnceCell;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec4 color;
varying highp vec2 uv;
varying lowp float mirror;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position.xy, 0.0, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
    mirror = position.z;
}
"#;

const FRAGMENT: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying highp vec2 uv;
varying lowp float mirror;
uniform sampler2D Texture;
uniform vec2 Step;
// Towards the light (texture space, y down, unit length).
uniform vec2 Light;
// rgb: the rim's colour, a: its strength; rgb: the core shadow's tint, a: its strength.
uniform vec4 Rim;
uniform vec4 Shade;
void main() {
    vec4 t = texture2D(Texture, uv);
    vec2 l = vec2(mix(Light.x, -Light.x, mirror), Light.y) * Step;
    // Solid towards the light? (two steps: a soft rim, about a unit wide)
    float lit = 1.0 - 0.5 * (texture2D(Texture, uv + l * 0.9).a + texture2D(Texture, uv + l * 1.8).a);
    // Solid away from it? (wider: the core shadow falls across the far side)
    float dark = 1.0 - 0.5 * (texture2D(Texture, uv - l * 1.6).a + texture2D(Texture, uv - l * 3.4).a);
    vec3 c = t.rgb * color.rgb;
    c = mix(c, c * Shade.rgb, clamp(dark, 0.0, 1.0) * Shade.a);
    c += Rim.rgb * clamp(lit, 0.0, 1.0) * Rim.a * (0.55 + 0.45 * dot(t.rgb, vec3(0.33)));
    gl_FragColor = vec4(c, t.a * color.a);
}
"#;

thread_local! {
    static MATERIAL: OnceCell<Option<Material>> = const { OnceCell::new() };
}

/// The lit-sprite material (None if it failed: draw flat), or None below Medium quality.
pub fn material() -> Option<Material> {
    if crate::post::quality() < crate::post::Quality::Medium {
        return None;
    }
    MATERIAL.with(|m| {
        m.get_or_init(|| {
            let u = |n: &str, t| UniformDesc::new(n, t);
            load_material(
                ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
                MaterialParams {
                    pipeline_params: PipelineParams {
                        color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
                        alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
                        ..Default::default()
                    },
                    uniforms: vec![u("Step", UniformType::Float2), u("Light", UniformType::Float2), u("Rim", UniformType::Float4), u("Shade", UniformType::Float4)],
                    ..Default::default()
                },
            )
            .map_err(|e| {
                warn!("lit sprite material: {e:?}");
            })
            .ok()
        })
        .clone()
    })
}

/// How a batch is lit: towards the light (world/screen space, unit), rim colour and
/// strength, core shadow tint and strength.
#[derive(Clone, Copy)]
pub struct Light {
    pub dir: Vec2,
    pub rim: Color,
    pub shade: Color,
}

/// The moon's light on everything (cool, from the upper left).
pub const MOON: Light = Light { dir: Vec2::new(-0.8, -0.6), rim: Color { r: 0.62, g: 0.74, b: 1.0, a: 0.32 }, shade: Color { r: 0.62, g: 0.6, b: 0.78, a: 0.42 } };

/// Sets the light for the next draws (call with the material in use).
pub fn set_light(mat: &Material, l: &Light) {
    mat.set_uniform("Light", l.dir.normalize_or(vec2(-0.8, -0.6)));
    mat.set_uniform("Rim", vec4(l.rim.r, l.rim.g, l.rim.b, l.rim.a));
    mat.set_uniform("Shade", vec4(l.shade.r, l.shade.g, l.shade.b, l.shade.a));
}

/// One sprite's quad as `draw_texture_ex` would lay it out (dest size, flip, rotation about
/// the pivot, a source rect), with the mirror flag in z.
pub fn quad(t: &Texture2D, x: f32, y: f32, c: Color, p: &DrawTextureParams, out: &mut Vec<Vertex>, idx: &mut Vec<u16>) {
    let (tw, th) = (t.width(), t.height());
    let src = p.source.unwrap_or(Rect::new(0.0, 0.0, tw, th));
    let size = p.dest_size.unwrap_or(vec2(src.w, src.h));
    let (mut u0, mut u1) = (src.x / tw, (src.x + src.w) / tw);
    let (mut v0, mut v1) = (src.y / th, (src.y + src.h) / th);
    if p.flip_x {
        std::mem::swap(&mut u0, &mut u1);
    }
    if p.flip_y {
        std::mem::swap(&mut v0, &mut v1);
    }
    let pivot = p.pivot.unwrap_or(vec2(x + size.x / 2.0, y + size.y / 2.0));
    let (s, co) = p.rotation.sin_cos();
    let r = |q: Vec2| {
        let d = q - pivot;
        pivot + vec2(d.x * co - d.y * s, d.x * s + d.y * co)
    };
    let z = if p.flip_x { 1.0 } else { 0.0 };
    let b = out.len() as u16;
    for (q, (u, v)) in [(vec2(x, y), (u0, v0)), (vec2(x + size.x, y), (u1, v0)), (vec2(x + size.x, y + size.y), (u1, v1)), (vec2(x, y + size.y), (u0, v1))] {
        let q = r(q);
        out.push(Vertex::new(q.x, q.y, z, u, v, c));
    }
    idx.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
}

/// Draws sprites (grouped by texture) lit by `light`. `step` gives a texture's world size
/// (its rim is measured in units). Falls back to flat drawing without the material.
pub fn draw_batch(items: &mut [(&Texture2D, f32, f32, Color, DrawTextureParams)], light: &Light) {
    let Some(mat) = material() else {
        for (t, x, y, c, p) in items.iter() {
            draw_texture_ex(t, *x, *y, *c, p.clone());
        }
        return;
    };
    items.sort_by_key(|s| s.0 as *const Texture2D as usize);
    gl_use_material(&mat);
    set_light(&mat, light);
    let mut i = 0;
    while i < items.len() {
        let tex = items[i].0;
        let (mut vs, mut is) = (Vec::new(), Vec::new());
        // A unit in uv: the sprite's drawn size (all of a texture's sprites draw about alike).
        let size = items[i].4.dest_size.unwrap_or(vec2(tex.width(), tex.height()));
        while i < items.len() && std::ptr::eq(items[i].0, tex) && vs.len() < 60000 {
            let (t, x, y, c, p) = &items[i];
            quad(t, *x, *y, *c, p, &mut vs, &mut is);
            i += 1;
        }
        mat.set_uniform("Step", vec2(1.0 / size.x.max(1.0), 1.0 / size.y.max(1.0)));
        draw_mesh(&Mesh { vertices: vs, indices: is, texture: Some(tex.clone()) });
    }
    gl_use_default_material();
}
