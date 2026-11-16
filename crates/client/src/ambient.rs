//! Life in the air: each world's drifting motes (fireflies, embers, snow, neon dust, void
//! sparks), and the screen vignette's strength.
//!
//! Stateless: every mote is a pure function of its cell, its index and the clock, so the same
//! motes can be asked for twice a frame (their lights, before the light map; their sprites,
//! after it) and never need storing. Motes live in a grid of cells around the camera; each one
//! wanders within its cell and wraps, so the field is endless and even.
use macroquad::prelude::*;
use sim::campaign::StageId;

use crate::vfx::Vfx;

const CELL: f32 = 180.0;

/// How a world's motes move and look.
#[derive(Clone, Copy)]
struct Kind {
    /// Motes per cell.
    per_cell: u32,
    /// Drift (world units/s) and how far each wanders side to side.
    drift: Vec2,
    wander: f32,
    size: f32,
    color: Color,
    /// Light each throws (colour; zero = none) and its reach.
    light: Color,
    reach: f32,
    /// Blink rate (fireflies), 0 = steady.
    blink: f32,
    sprite: &'static str,
}

fn kind(stage: StageId) -> Kind {
    let c = |r, g, b| Color::new(r, g, b, 1.0);
    match stage {
        // Fireflies over the shrine valley: slow, wandering, blinking, each a tiny lamp.
        StageId::ShrineValley => Kind { per_cell: 3, drift: vec2(4.0, -2.0), wander: 26.0, size: 7.0, color: c(0.85, 1.0, 0.45), light: c(0.10, 0.13, 0.03), reach: 46.0, blink: 0.7, sprite: "glow" },
        // Embers rising from the burning wood.
        StageId::EmberLeaf => Kind { per_cell: 4, drift: vec2(6.0, -22.0), wander: 14.0, size: 5.0, color: c(1.0, 0.55, 0.2), light: c(0.10, 0.04, 0.01), reach: 30.0, blink: 0.0, sprite: "ember" },
        // Snow drifting down over the citadel.
        StageId::PaleCitadel => Kind { per_cell: 7, drift: vec2(-8.0, 24.0), wander: 12.0, size: 4.0, color: c(0.92, 0.96, 1.0), light: c(0.0, 0.0, 0.0), reach: 0.0, blink: 0.0, sprite: "glow" },
        // Neon dust hanging in the shopping street's air.
        StageId::Shotengai => Kind { per_cell: 3, drift: vec2(3.0, -4.0), wander: 18.0, size: 5.0, color: c(1.0, 0.45, 0.85), light: c(0.05, 0.02, 0.05), reach: 30.0, blink: 0.35, sprite: "glow" },
        // Wisps of mist light in the rain.
        StageId::Rainward => Kind { per_cell: 2, drift: vec2(-5.0, 2.0), wander: 30.0, size: 9.0, color: c(0.55, 0.8, 1.0), light: c(0.02, 0.05, 0.08), reach: 50.0, blink: 0.25, sprite: "glow" },
        // Void sparks drifting up through the shattered sky.
        StageId::Shatterline => Kind { per_cell: 4, drift: vec2(0.0, -12.0), wander: 20.0, size: 5.0, color: c(0.75, 0.45, 1.0), light: c(0.06, 0.02, 0.10), reach: 34.0, blink: 0.5, sprite: "glow" },
    }
}

fn hash(a: i32, b: i32, c: u32) -> u32 {
    sim::stage::hash(a, b, c)
}

fn unit(h: u32) -> f32 {
    (h & 0xFFFF) as f32 / 65535.0
}

/// Every visible mote: (position, size, brightness 0..1).
fn motes(stage: StageId, view: Rect, time: f64, mut f: impl FnMut(Vec2, f32, f32)) {
    let k = kind(stage);
    let t = time as f32;
    let (cx0, cy0) = (((view.x - CELL) / CELL).floor() as i32, ((view.y - CELL) / CELL).floor() as i32);
    let (cx1, cy1) = (((view.x + view.w + CELL) / CELL).floor() as i32, ((view.y + view.h + CELL) / CELL).floor() as i32);
    for cy in cy0..=cy1 {
        for cx in cx0..=cx1 {
            for i in 0..k.per_cell {
                let h = hash(cx, cy, 0xA11 + i);
                let (u, v, ph) = (unit(h), unit(h >> 8 ^ 0x5bd1), unit(h >> 16 ^ 0x2c3a) * std::f32::consts::TAU);
                // Drift and wander, wrapped inside the cell so the field never thins out.
                let base = vec2(u, v) * CELL + k.drift * t;
                let w = vec2((t * 0.37 + ph).sin(), (t * 0.29 + ph * 1.7).cos()) * k.wander;
                let local = vec2(base.x.rem_euclid(CELL), base.y.rem_euclid(CELL)) + w;
                let p = vec2(cx as f32 * CELL, cy as f32 * CELL) + local;
                if p.x < view.x - 20.0 || p.x > view.x + view.w + 20.0 || p.y < view.y - 20.0 || p.y > view.y + view.h + 20.0 {
                    continue;
                }
                let glow = if k.blink > 0.0 { (0.5 + 0.5 * (t * k.blink * 3.0 + ph * 3.0).sin()).powi(3) } else { 0.8 + 0.2 * (t * 2.0 + ph).sin() };
                f(p, k.size * (0.7 + 0.6 * unit(h >> 4)), glow);
            }
        }
    }
}

/// The motes' lights (call between `Lights::begin` and `apply`).
pub fn lights(stage: StageId, view: Rect, time: f64, l: &mut crate::light::Lights) {
    let k = kind(stage);
    if k.reach <= 0.0 {
        return;
    }
    motes(stage, view, time, |p, _, g| {
        if g > 0.15 {
            l.add_own(p, k.reach, Color::new(k.light.r * g, k.light.g * g, k.light.b * g, 1.0));
        }
    });
}

/// The motes themselves (after the light map: they glow at full colour).
pub fn draw(stage: StageId, view: Rect, time: f64, vfx: &Vfx) {
    let k = kind(stage);
    vfx.additive(|| {
        motes(stage, view, time, |p, size, g| {
            let c = Color::new(k.color.r, k.color.g, k.color.b, 0.15 + 0.85 * g);
            vfx.draw(k.sprite, p, size * (1.6 + g), 0.0, c);
            vfx.draw("glow", p, size * 0.6, 0.0, Color::new(1.0, 1.0, 1.0, 0.6 * g));
        });
    });
}

/// Each world's ground mist: colour (alpha = density) and drift (world units/s).
fn mist(stage: StageId) -> (Color, Vec2) {
    match stage {
        StageId::ShrineValley => (Color::new(0.72, 0.80, 0.92, 0.16), vec2(5.0, 1.5)),
        StageId::EmberLeaf => (Color::new(0.55, 0.45, 0.42, 0.14), vec2(3.0, -4.0)),
        StageId::PaleCitadel => (Color::new(0.85, 0.90, 1.0, 0.18), vec2(-6.0, 2.0)),
        StageId::Shotengai => (Color::new(0.55, 0.42, 0.70, 0.08), vec2(2.0, 1.0)),
        StageId::Rainward => (Color::new(0.62, 0.74, 0.88, 0.22), vec2(-7.0, 3.0)),
        StageId::Shatterline => (Color::new(0.55, 0.42, 0.85, 0.12), vec2(0.0, -3.0)),
    }
}

/// World units one mist tile covers.
const MIST_TILE: f32 = 640.0;

thread_local! {
    static MIST: std::cell::OnceCell<Texture2D> = const { std::cell::OnceCell::new() };
}

/// A tileable cloudy noise (fractal value noise on a wrapping grid), white with the cloud in
/// its alpha, built once.
fn mist_texture() -> Texture2D {
    MIST.with(|m| {
        m.get_or_init(|| {
            const N: usize = 128;
            let lattice = |o: u32, g: i32, x: i32, y: i32| unit(hash(x.rem_euclid(g), y.rem_euclid(g), o));
            let noise = |o: u32, g: i32, u: f32, v: f32| {
                let (x, y) = (u * g as f32, v * g as f32);
                let (xi, yi) = (x.floor() as i32, y.floor() as i32);
                let (fx, fy) = (x - xi as f32, y - yi as f32);
                let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
                let a = lattice(o, g, xi, yi) + (lattice(o, g, xi + 1, yi) - lattice(o, g, xi, yi)) * sx;
                let b = lattice(o, g, xi, yi + 1) + (lattice(o, g, xi + 1, yi + 1) - lattice(o, g, xi, yi + 1)) * sx;
                a + (b - a) * sy
            };
            let mut px = Vec::with_capacity(N * N * 4);
            for y in 0..N {
                for x in 0..N {
                    let (u, v) = (x as f32 / N as f32, y as f32 / N as f32);
                    let f = noise(1, 3, u, v) * 0.5 + noise(2, 6, u, v) * 0.3 + noise(3, 12, u, v) * 0.2;
                    // Banks of mist with clear gaps between.
                    let k = ((f - 0.38) / 0.4).clamp(0.0, 1.0);
                    px.extend_from_slice(&[255, 255, 255, (k * k * (3.0 - 2.0 * k) * 255.0) as u8]);
                }
            }
            let t = Texture2D::from_rgba8(N as u16, N as u16, &px);
            t.set_filter(FilterMode::Linear);
            t
        })
        .clone()
    })
}

/// Low mist drifting over the ground in two layers at different speeds (parallax), drawn
/// before the light map so lamps light it and it glows around them.
pub fn draw_mist(stage: StageId, view: Rect, time: f64) {
    let (c, drift) = mist(stage);
    let tex = mist_texture();
    let t = time as f32;
    for (layer, (scale, speed, share)) in [(1.0, 1.0, 0.6), (0.62, 1.7, 0.4)].into_iter().enumerate() {
        let size = MIST_TILE * scale;
        let off = drift * speed * t + vec2(layer as f32 * 211.0, layer as f32 * 97.0);
        let (x0, y0) = (((view.x - off.x) / size).floor(), ((view.y - off.y) / size).floor());
        let (x1, y1) = (((view.x + view.w - off.x) / size).ceil(), ((view.y + view.h - off.y) / size).ceil());
        // Breathe: density swells and thins slowly.
        let a = c.a * share * (0.85 + 0.15 * (t * 0.13 + layer as f32 * 2.0).sin());
        let mut y = y0;
        while y < y1 {
            let mut x = x0;
            while x < x1 {
                draw_texture_ex(&tex, x * size + off.x, y * size + off.y, Color::new(c.r, c.g, c.b, a), DrawTextureParams { dest_size: Some(vec2(size, size)), flip_x: layer == 1, ..Default::default() });
                x += 1.0;
            }
            y += 1.0;
        }
    }
}
