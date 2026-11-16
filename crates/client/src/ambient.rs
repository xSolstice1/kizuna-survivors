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
