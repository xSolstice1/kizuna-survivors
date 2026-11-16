//! Light and shadow for the world: the moon (high in the upper left) casts every shadow the
//! same way, each steady lamp casts its own away from it, and a light map darkens the night while lanterns, fire, spells and pickups throw
//! warm pools of light.
//!
//! - `Shadows`: each sprite's own texture, tinted black and slanted away from the moon onto the
//!   ground, so a shadow has its sprite's shape. Batched per texture into one mesh.
//! - `Lights`: a quarter-resolution light map. It starts at the stage's night ambient and
//!   lights are added on top; it then multiplies the world drawn so far, doubled (0.5 = as
//!   drawn, lower darker, higher brighter), so lit pools glow instead of only being less dark.
//!   `light_at` gives the same light on the CPU, for what is drawn after it (the hero).
pub mod sources;

use std::cell::RefCell;

use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;
use sim::campaign::StageId;

/// Where the top of a sprite's shadow falls, per unit of sprite height, from its feet: down and
/// to the right, away from a moon in the upper left, flattened onto the ground.
pub const SKEW: Vec2 = Vec2::new(0.36, 0.30);
/// How dark a full shadow is, at the feet.
pub const SHADOW_ALPHA: f32 = 0.5;
/// How much of that is left at a shadow's far tip: shadows harden at contact and soften and
/// fade with distance, as real ones do.
const TIP: f32 = 0.3;
/// The moon's share: moonlight is soft and dim, so its shadow is lighter than a lamp's.
const MOON: f32 = 0.8;

/// Most shadows one caster throws: the moon's and those of the strongest lamps near it.
pub const MAX_CAST: usize = 4;

/// Cast shadows for one draw pass, flushed (one mesh per texture) before the sprites. Every
/// caster gets the same treatment (`Lights::casts_at`): the moon's shadow, plus one away from
/// each steady light near it.
pub struct Shadows<'a> {
    quads: Vec<(&'a Texture2D, [Vec2; 4], bool, f32)>,
    lamps: Vec<Lamp>,
}

impl<'a> Shadows<'a> {
    pub fn new() -> Self {
        Shadows { quads: Vec::new(), lamps: LIGHTS.with(|l| l.borrow().lamps.clone()) }
    }

    /// A `w` x `h` sprite standing at `feet` (the ground point under it), mirrored if `flip`.
    /// `alpha` scales the darkness (a sprite floating off the ground casts a fainter one).
    pub fn push(&mut self, tex: &'a Texture2D, feet: Vec2, w: f32, h: f32, flip: bool, alpha: f32) {
        let mut casts = [(SKEW, 0.0); MAX_CAST];
        let n = casts_into(&self.lamps, feet, w * 0.5, &mut casts);
        for &(skew, a) in &casts[..n] {
            self.push_skew(tex, feet, w, h, flip, alpha * a, skew);
        }
    }

    /// One shadow of the sprite, cast along `skew` (per unit of height, like `SKEW`).
    #[allow(clippy::too_many_arguments)]
    pub fn push_skew(&mut self, tex: &'a Texture2D, feet: Vec2, w: f32, h: f32, flip: bool, alpha: f32, skew: Vec2) {
        if alpha <= 0.01 {
            return;
        }
        let top = feet + skew * h;
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
                let b = vertices.len() as u16;
                for (p, (u, v)) in c.iter().zip([(u0, 0.0), (u1, 0.0), (u1, 1.0), (u0, 1.0)]) {
                    let col = Color::new(0.0, 0.0, 0.0, SHADOW_ALPHA * a * (TIP + (1.0 - TIP) * v));
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
        StageId::ShrineValley => Color::new(0.32, 0.34, 0.45, 1.0),
        StageId::Shotengai => Color::new(0.35, 0.33, 0.44, 1.0),
        StageId::EmberLeaf => Color::new(0.37, 0.32, 0.34, 1.0),
        StageId::PaleCitadel => Color::new(0.37, 0.38, 0.47, 1.0),
        StageId::Rainward => Color::new(0.31, 0.35, 0.44, 1.0),
        StageId::Shatterline => Color::new(0.33, 0.31, 0.45, 1.0),
    }
}

/// A light: where, how far it reaches (world units) and its colour added at the centre.
#[derive(Clone, Copy)]
pub struct Light {
    pub pos: Vec2,
    /// The ground point under it (where its shadows fall away from).
    pub ground: Vec2,
    pub radius: f32,
    pub color: Color,
    /// Whether it casts shadows (steady lights; not the hero's own, nor passing ones).
    pub casts: bool,
    /// Whether it blooms (a visible source; not a soft fill like the hero's own light).
    pub halo: bool,
}

/// A steady light as a shadow caster: its ground point, reach and strength.
#[derive(Clone, Copy)]
pub struct Lamp {
    ground: Vec2,
    radius: f32,
    strength: f32,
}

/// How strong a lamp's shadow is at full falloff (its light's r + g + b times this).
const LAMP_SHADOW: f32 = 3.2;
/// How high a lamp shines from, for its shadows' length (world units).
const LAMP_HEIGHT: f32 = 64.0;

/// The shadows cast by something at `feet` (`own` = its half-width: a lamp that close is its
/// own light and casts none on it) into `out`: the moon's first, then each lamp's, away from it
/// and longer the further off it stands. Returns how many. Strength fades smoothly to nothing
/// at a lamp's edge, so walking past one never makes a shadow pop. Under a strong lamp the
/// moon's shadow fades (the lamp outshines it).
fn casts_into(lamps: &[Lamp], feet: Vec2, own: f32, out: &mut [(Vec2, f32); MAX_CAST]) -> usize {
    let mut n = 1;
    let mut total = 0.0;
    for l in lamps {
        let d = feet - l.ground;
        let dist = d.length();
        if dist <= own.max(4.0) || dist >= l.radius {
            continue;
        }
        let a = (falloff(dist, l.radius) * l.strength - 0.04).clamp(0.0, 1.0);
        if a <= 0.0 {
            continue;
        }
        total += a;
        let len = (dist / LAMP_HEIGHT).clamp(0.25, 1.25);
        // Fade in as the caster leaves the lamp's base (directly under it there's no shadow).
        let a = a * ((dist - own.max(4.0)) / 10.0).min(1.0);
        let skew = d / dist * len;
        if n < MAX_CAST {
            out[n] = (skew, a);
            n += 1;
        } else if let Some(w) = out[1..].iter_mut().min_by(|x, y| x.1.total_cmp(&y.1)).filter(|w| w.1 < a) {
            *w = (skew, a);
        }
    }
    out[0] = (SKEW, MOON * (1.0 - 0.6 * total.min(1.0)));
    n
}

/// The light map and its materials (one per thread: the game draws on one).
pub struct Lights {
    rt: Option<(RenderTarget, u32, u32)>,
    silhouette: Option<RenderTarget>,
    /// Last frame's steady lights, as shadow casters (shadows are drawn before this frame's
    /// lights are all known; a frame late is never seen).
    lamps: Vec<Lamp>,
    /// The hero's light tint, eased over time (attacks flashing by don't make it flicker).
    hero_tint: Option<Color>,
    multiply: Option<Material>,
    pub list: Vec<Light>,
    ambient: Color,
}

thread_local! {
    pub static LIGHTS: RefCell<Lights> = RefCell::new(Lights { rt: None, silhouette: None, lamps: Vec::new(), hero_tint: None, multiply: None, list: Vec::new(), ambient: WHITE });
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
        self.lamps.clear();
        for l in self.list.iter().filter(|l| l.casts) {
            self.lamps.push(Lamp { ground: l.ground, radius: l.radius, strength: (l.color.r + l.color.g + l.color.b) * LAMP_SHADOW });
        }
        self.list.clear();
        self.ambient = ambient(stage);
    }

    /// A steady light shining from `lift` above `ground`: it casts shadows.
    pub fn add(&mut self, ground: Vec2, lift: f32, radius: f32, color: Color) {
        if self.list.len() < 600 {
            self.list.push(Light { pos: ground - vec2(0.0, lift), ground, radius, color, casts: true, halo: true });
        }
    }

    /// A soft fill light with no visible source: lights things, casts and blooms nothing (the
    /// hero's own light).
    pub fn add_fill(&mut self, pos: Vec2, radius: f32, color: Color) {
        if self.list.len() < 600 {
            self.list.push(Light { pos, ground: pos, radius, color, casts: false, halo: false });
        }
    }

    /// A light that steers no shadows: the hero's own glow, and anything that moves or comes
    /// and goes (attacks, pickups, gems), which would make shadows swing about.
    pub fn add_own(&mut self, pos: Vec2, radius: f32, color: Color) {
        if self.list.len() < 600 {
            self.list.push(Light { pos, ground: pos, radius, color, casts: false, halo: true });
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

    /// `light_at(p)` eased over time (`dt` seconds), for the tint of one character.
    pub fn eased_light(&mut self, p: Vec2, dt: f32) -> Color {
        let t = self.light_at(p);
        let k = 1.0 - (-dt * 6.0).exp();
        let c = match self.hero_tint {
            Some(c) => Color::new(c.r + (t.r - c.r) * k, c.g + (t.g - c.g) * k, c.b + (t.b - c.b) * k, 1.0),
            None => t,
        };
        self.hero_tint = Some(c);
        c
    }

    /// The shadows cast at `feet` (see `casts_into`): the moon's, then each nearby lamp's.
    pub fn casts_at(&self, feet: Vec2, own: f32) -> Vec<(Vec2, f32)> {
        let mut out = [(SKEW, 0.0); MAX_CAST];
        let n = casts_into(&self.lamps, feet, own, &mut out);
        out[..n].to_vec()
    }

    /// The rig's shadows: the posed rig drawn black into a small soft target once, then laid
    /// on the ground from its feet along each of `casts_at`, so they walk and sway with the hero.
    #[allow(clippy::too_many_arguments)]
    pub fn rig_shadow(&mut self, def: &crate::rig::RigDef, st: &crate::rig::RigState, feet: Vec2, left: bool, alpha: f32, cam: &Camera2D) {
        const BOX: f32 = 48.0;
        const ABOVE: f32 = 44.0; // the box reaches this far above the feet, BOX - ABOVE below
        if self.silhouette.is_none() {
            let rt = render_target(96, 96);
            rt.texture.set_filter(FilterMode::Linear);
            self.silhouette = Some(rt);
        }
        let rt = self.silhouette.clone().unwrap();
        let mut c = Camera2D::from_display_rect(Rect::new(feet.x - BOX / 2.0, feet.y - ABOVE, BOX, BOX));
        c.render_target = Some(rt.clone());
        set_camera(&c);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        crate::rig::draw(def, st, feet, left, BLACK);
        set_camera(cam);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for (skew, a) in self.casts_at(feet, 6.0) {
            let b = vertices.len() as u16;
            for &(u, v) in &[(0.0f32, 0.0f32), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                let height = ABOVE - v * BOX;
                // Fade towards the tip (the box's top, v = 0, lies furthest from the feet).
                let tip = (height / ABOVE).clamp(0.0, 1.0);
                let col = Color::new(0.0, 0.0, 0.0, SHADOW_ALPHA * alpha * a * (1.0 - (1.0 - TIP) * tip));
                let q = feet + vec2((u - 0.5) * BOX, 0.0) + skew * height;
                vertices.push(Vertex::new(q.x, q.y, 0.0, u, 1.0 - v, col));
            }
            indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
        }
        draw_mesh(&Mesh { vertices, indices, texture: Some(rt.texture.clone()) });
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
        // Exactly what the camera sees (its clip-space corners in the world), so the map
        // covers the screen edge to edge whatever the zoom, shake or target size.
        let inv = cam.matrix().inverse();
        let a = inv.project_point3(vec3(-1.0, -1.0, 0.0));
        let b = inv.project_point3(vec3(1.0, 1.0, 0.0));
        let (x0, x1, y0, y1) = (a.x.min(b.x), a.x.max(b.x), a.y.min(b.y), a.y.max(b.y));
        let flip = a.y > b.y;
        let _ = view;
        gl_use_material(mul);
        draw_texture_ex(&rt.texture, x0, y0, WHITE, DrawTextureParams { dest_size: Some(vec2(x1 - x0, y1 - y0)), flip_y: !flip, ..Default::default() });
        gl_use_default_material();
        // Bloom: bright lights bleed a soft halo over whatever stands in front of them, as
        // they would through a lens, layered tight and wide.
        additive(&|| {
            for l in lights.iter() {
                let lum = l.color.r + l.color.g + l.color.b;
                if !l.halo || lum < 0.2 || l.pos.x < x0 - l.radius || l.pos.x > x1 + l.radius || l.pos.y < y0 - l.radius || l.pos.y > y1 + l.radius {
                    continue;
                }
                let k = ((lum - 0.2) / 0.5).min(1.0);
                for (reach, gain) in [(0.12, 0.9), (0.32, 0.35)] {
                    let r = l.radius * reach;
                    let c = Color::new(l.color.r * gain * k, l.color.g * gain * k, l.color.b * gain * k, 1.0);
                    draw_texture_ex(glow, l.pos.x - r, l.pos.y - r, c, DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() });
                }
            }
        });
    }
}
