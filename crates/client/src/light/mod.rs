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
pub const MAX_CAST: usize = 3;

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

    /// Draws the batch into the shadow layer (`Lights::shadow_layer`), so where shadows
    /// overlap they merge into one instead of stacking darker. `cam` is the camera drawn with.
    pub fn flush(&mut self, cam: &Camera2D) {
        if self.quads.is_empty() {
            return;
        }
        self.quads.sort_by_key(|q| q.0 as *const Texture2D as usize);
        let quads = std::mem::take(&mut self.quads);
        let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for p in quads.iter().flat_map(|q| q.1) {
            lo = lo.min(p);
            hi = hi.max(p);
        }
        let bounds = Rect::new(lo.x, lo.y, hi.x - lo.x, hi.y - lo.y);
        LIGHTS.with(|l| {
            l.borrow_mut().shadow_layer(cam, bounds, &|| {
                let mut i = 0;
                while i < quads.len() {
                    let tex = quads[i].0;
                    let mut vertices = Vec::new();
                    let mut indices = Vec::new();
                    while i < quads.len() && std::ptr::eq(quads[i].0, tex) && vertices.len() < 60000 {
                        let (_, c, flip, a) = quads[i];
                        let (u0, u1) = if flip { (1.0, 0.0) } else { (0.0, 1.0) };
                        let b = vertices.len() as u16;
                        for (p, (u, v)) in c.iter().zip([(u0, 0.0), (u1, 0.0), (u1, 1.0), (u0, 1.0)]) {
                            vertices.push(Vertex::new(p.x, p.y, 0.0, u, v, strength(a * (TIP + (1.0 - TIP) * v))));
                        }
                        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
                        i += 1;
                    }
                    draw_mesh(&Mesh { vertices, indices, texture: Some(tex.clone()) });
                }
            })
        });
    }
}

/// A shadow strength (0..1) as a vertex colour for the shadow layer (it's added up in red).
fn strength(s: f32) -> Color {
    Color::new(s.clamp(0.0, 1.0), 0.0, 0.0, 1.0)
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

/// The light map's brightest per world (light-map units: 0.7 = 1.4x the art). Bright
/// worlds (snow) take less before they wash out.
fn cap(stage: StageId) -> f32 {
    match stage {
        StageId::PaleCitadel => 0.58,
        _ => 0.68,
    }
}

/// What `cam` sees, as a world rect, and whether a render target drawn over it must be
/// flipped (`flip_y`): a target's texture comes out upside down when drawn by a screen camera,
/// the right way up by a render target's camera.
fn view_of(cam: &Camera2D) -> (Rect, bool) {
    let inv = cam.matrix().inverse();
    let a = inv.project_point3(vec3(-1.0, -1.0, 0.0));
    let b = inv.project_point3(vec3(1.0, 1.0, 0.0));
    (Rect::new(a.x.min(b.x), a.y.min(b.y), (a.x - b.x).abs(), (a.y - b.y).abs()), a.y > b.y)
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
    color: Color,
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
        let len = (dist / LAMP_HEIGHT).clamp(0.3, 1.0);
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
    /// The shadow layer: a half-resolution target where a batch's shadows add up, and the
    /// materials that add them and lay the (capped, softened) result on the world.
    shadow_rt: Option<(RenderTarget, u32, u32)>,
    shadow_add: Option<Material>,
    shadow_lay: Option<Material>,
    /// The light map's brightest (per world: snow can't take as much as a dark street).
    cap: f32,
    /// Wet ground (Rainward): lamps reflect as long streaks below them.
    wet: bool,
    pub list: Vec<Light>,
    ambient: Color,
}

thread_local! {
    pub static LIGHTS: RefCell<Lights> = RefCell::new(Lights { rt: None, silhouette: None, lamps: Vec::new(), hero_tint: None, multiply: None, shadow_rt: None, shadow_add: None, shadow_lay: None, cap: 0.7, wet: false, list: Vec::new(), ambient: WHITE });
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
uniform lowp float Cap;
void main() {
    // At most 2 x Cap brighter: pools glow, crowds of lights never wash the scene out. The
    // approach to the cap is rolled off, so a pool's bright centre fades into it, not clips.
    vec3 c = texture2D(Texture, uv).rgb;
    vec3 knee = vec3(Cap - 0.12);
    vec3 over = max(c - knee, 0.0);
    c = min(c, knee) + 0.12 * over / (over + 0.12);
    gl_FragColor = vec4(c, 1.0);
}
"#;

/// Adds each shadow's strength (vertex red) where its sprite is solid.
const SHADOW_ADD: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = vec4(texture2D(Texture, uv).a * color.r, 0.0, 0.0, 1.0);
}
"#;

/// Lays the shadow layer on the world: overlaps merge (capped at one full shadow), and a
/// small blur gives every edge a soft penumbra.
const SHADOW_LAY: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
uniform vec2 Texel;
uniform float Alpha;
void main() {
    // Four bilinear taps half a texel off each diagonal: a 4x4 texel blur for four reads.
    vec2 o = Texel * 0.75;
    float s = 0.25 * (texture2D(Texture, uv + o).r + texture2D(Texture, uv - o).r
        + texture2D(Texture, uv + vec2(o.x, -o.y)).r + texture2D(Texture, uv + vec2(-o.x, o.y)).r);
    s = 1.0 - exp(-2.2 * s);
    gl_FragColor = vec4(0.0, 0.0, 0.0, min(s, 0.92) * Alpha);
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
        self.cap = cap(stage);
        self.wet = stage == StageId::Rainward;
        self.lamps.clear();
        for l in self.list.iter().filter(|l| l.casts) {
            self.lamps.push(Lamp { ground: l.ground, radius: l.radius, strength: (l.color.r + l.color.g + l.color.b) * LAMP_SHADOW, color: l.color });
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

    /// How a character at `feet` is lit (crate::litsprite): by the moon, and when a lamp is
    /// near, warmly from the lamp's side, the rim taking its colour.
    pub fn rim_at(&self, feet: Vec2) -> crate::litsprite::Light {
        // The hero is the one thing the eye follows: the moon models them a little more firmly
        // than the crowd.
        let m0 = crate::litsprite::MOON;
        let moon = crate::litsprite::Light { rim: Color { a: 0.36, ..m0.rim }, shade: Color { a: 0.48, ..m0.shade }, ..m0 };
        let best = self
            .lamps
            .iter()
            .map(|l| {
                let d = l.ground - feet;
                (l, d, (falloff(d.length(), l.radius) * l.strength).min(1.0))
            })
            .filter(|x| x.2 > 0.02 && x.1.length() > 4.0)
            .max_by(|a, b| a.2.total_cmp(&b.2));
        let Some((l, d, k)) = best else { return moon };
        // Towards the lamp, which shines from above it: a little upwards too.
        let toward = (d.normalize() + vec2(0.0, -0.35)).normalize();
        // The lamp's hue, softened towards white: light grazing an edge, not paint.
        let m = l.color.r.max(l.color.g).max(l.color.b).max(1e-3);
        let soft = |c: f32| 0.45 + 0.55 * c / m;
        let lamp = Color::new(soft(l.color.r), soft(l.color.g), soft(l.color.b), 1.0);
        let mix = |a: f32, b: f32| a + (b - a) * k;
        crate::litsprite::Light {
            dir: (moon.dir * (1.0 - k) + toward * k).normalize_or(moon.dir),
            rim: Color::new(mix(moon.rim.r, lamp.r), mix(moon.rim.g, lamp.g), mix(moon.rim.b, lamp.b), mix(moon.rim.a, 0.55)),
            shade: Color { a: moon.shade.a * 1.1, ..moon.shade },
        }
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
                let q = feet + vec2((u - 0.5) * BOX, 0.0) + skew * height;
                vertices.push(Vertex::new(q.x, q.y, 0.0, u, 1.0 - v, strength(alpha * a * (1.0 - (1.0 - TIP) * tip))));
            }
            indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
        }
        let tex = rt.texture.clone();
        let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for v in &vertices {
            lo = lo.min(v.position.truncate());
            hi = hi.max(v.position.truncate());
        }
        self.shadow_layer(cam, Rect::new(lo.x, lo.y, hi.x - lo.x, hi.y - lo.y), &|| draw_mesh(&Mesh { vertices: vertices.clone(), indices: indices.clone(), texture: Some(tex.clone()) }));
    }

    /// Draws one batch of shadows (`draw`, in world units, with vertex red = strength) into
    /// the shadow layer, then lays it on what `cam` draws to: overlapping shadows merge into
    /// one, and edges soften. One call per batch (props, the crowd, the hero).
    ///
    /// `bounds` (world units) holds everything `draw` draws: only that much of the layer is
    /// laid on the world (a prop's or the hero's shadows cover a sliver of the screen).
    pub fn shadow_layer(&mut self, cam: &Camera2D, bounds: Rect, draw: &dyn Fn()) {
        // Half the world target's resolution (CSS pixels, like it): soft, and cheap.
        let (w, h) = ((screen_width() / 2.0).max(16.0) as u32, (screen_height() / 2.0).max(16.0) as u32);
        if self.shadow_rt.as_ref().is_none_or(|(_, rw, rh)| *rw != w || *rh != h) {
            let rt = render_target(w, h);
            rt.texture.set_filter(FilterMode::Linear);
            self.shadow_rt = Some((rt, w, h));
        }
        if self.shadow_add.is_none() {
            let blend = |src, dst| PipelineParams {
                color_blend: Some(BlendState::new(Equation::Add, src, dst)),
                alpha_blend: Some(BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One)),
                ..Default::default()
            };
            self.shadow_add = load_material(
                ShaderSource::Glsl { vertex: MUL_VERTEX, fragment: SHADOW_ADD },
                MaterialParams { pipeline_params: blend(BlendFactor::One, BlendFactor::One), ..Default::default() },
            )
            .map_err(|e| { warn!("shadow add material: {e:?}"); })
            .ok();
            self.shadow_lay = load_material(
                ShaderSource::Glsl { vertex: MUL_VERTEX, fragment: SHADOW_LAY },
                MaterialParams {
                    pipeline_params: blend(BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha)),
                    uniforms: vec![UniformDesc::new("Texel", UniformType::Float2), UniformDesc::new("Alpha", UniformType::Float1)],
                    ..Default::default()
                },
            )
            .map_err(|e| { warn!("shadow lay material: {e:?}"); })
            .ok();
        }
        let (Some((rt, _, _)), Some(add), Some(lay)) = (self.shadow_rt.as_ref(), self.shadow_add.as_ref(), self.shadow_lay.as_ref()) else { return };
        let rt = rt.clone();
        let sc = Camera2D { target: cam.target, zoom: cam.zoom, offset: cam.offset, rotation: cam.rotation, render_target: Some(rt.clone()), viewport: None };
        set_camera(&sc);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        gl_use_material(add);
        draw();
        gl_use_default_material();
        set_camera(cam);
        let (r, flip) = view_of(cam);
        // The part of the view the shadows cover (a little more, for the blur), in texels;
        // rows count from the texture's top, which is the view's bottom when it's flipped.
        let pad = r.w / w as f32 * 3.0;
        let Some(b) = Rect::new(bounds.x - pad, bounds.y - pad, bounds.w + pad * 2.0, bounds.h + pad * 2.0).intersect(r) else { return };
        let (sx, sy) = (w as f32 / r.w, h as f32 / r.h);
        let row = if flip { r.y + r.h - (b.y + b.h) } else { b.y - r.y };
        let src = Rect::new((b.x - r.x) * sx, row * sy, b.w * sx, b.h * sy);
        lay.set_uniform("Texel", vec2(1.0 / w as f32, 1.0 / h as f32));
        lay.set_uniform("Alpha", SHADOW_ALPHA);
        gl_use_material(lay);
        draw_texture_ex(&rt.texture, b.x, b.y, WHITE, DrawTextureParams { dest_size: Some(vec2(b.w, b.h)), source: Some(src), flip_y: flip, ..Default::default() });
        gl_use_default_material();
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
                    uniforms: vec![UniformDesc::new("Cap", UniformType::Float1)],
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
        let wet = self.wet;
        additive(&|| {
            for l in lights.iter() {
                let d = l.radius * 2.0;
                draw_texture_ex(glow, l.pos.x - l.radius, l.pos.y - l.radius, l.color, DrawTextureParams { dest_size: Some(vec2(d, d)), ..Default::default() });
                // Wet ground mirrors a raised light: a long bright streak running down from
                // under it, as lamps shine on a rainy street.
                let lift = l.ground.y - l.pos.y;
                if wet && l.casts && lift > 4.0 {
                    let (w, h) = (l.radius * 0.32, l.radius * 1.1);
                    let c = Color::new(l.color.r * 0.9, l.color.g * 0.9, l.color.b * 0.9, 1.0);
                    draw_texture_ex(glow, l.ground.x - w / 2.0, l.ground.y + lift * 0.6 - h * 0.35, c, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
                }
            }
        });
        set_camera(cam);
        // Exactly what the camera sees, so the map covers the screen edge to edge whatever
        // the zoom, shake or target size.
        let (r, flip) = view_of(cam);
        let (x0, x1, y0, y1) = (r.x, r.x + r.w, r.y, r.y + r.h);
        let _ = view;
        mul.set_uniform("Cap", self.cap);
        gl_use_material(mul);
        draw_texture_ex(&rt.texture, x0, y0, WHITE, DrawTextureParams { dest_size: Some(vec2(x1 - x0, y1 - y0)), flip_y: flip, ..Default::default() });
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
