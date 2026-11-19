//! The movement key's look: each hero's move drawn as its own (sim::world::movement).
//!
//! - On the ground, under the hero: what moves leave (Ren's ofuda line, Yuzu's notes, the
//!   line Haru's cut will land on) and Kuon's star marker.
//! - In the air: the move's streak from where it began to where it went (lightning for
//!   Kaede, foxfire for Riku, shadow for Shun...), a puff where a blink left and arrived,
//!   dust and a ring where a dash or leap lands. A leap lifts the hero along an arc (`lift`).
use macroquad::prelude::*;
use sim::content::{move_def, MoveKind};
use sim::world::{Event, TrailKind, World};

use crate::render::Assets;

const FLIGHT: f32 = 0.32;
const LAND: f32 = 0.45;

#[derive(Default)]
pub struct MoveFx {
    /// (kind, from, to, age)
    flights: Vec<(MoveKind, Vec2, Vec2, f32)>,
    /// (kind, where, age)
    lands: Vec<(MoveKind, Vec2, f32)>,
}

fn v(p: sim::Vec2) -> Vec2 {
    vec2(p.x, p.y)
}

/// Each hero's colour for their move.
pub fn tint(kind: MoveKind) -> Color {
    use MoveKind::*;
    match kind {
        PurifyingStep => Color::new(1.0, 0.85, 0.5, 1.0),
        LightningFlash => Color::new(0.6, 0.8, 1.0, 1.0),
        StarlightLeap => Color::new(1.0, 0.72, 0.9, 1.0),
        IaidoRush => Color::new(0.85, 0.92, 1.0, 1.0),
        FlaskJump => Color::new(1.0, 0.6, 0.3, 1.0),
        EncoreSlide => Color::new(1.0, 0.6, 0.85, 1.0),
        FoxBlink => Color::new(0.45, 0.8, 1.0, 1.0),
        VeilCrossing => Color::new(1.0, 0.25, 0.3, 1.0),
        TendrilPull => Color::new(0.9, 0.12, 0.2, 1.0),
        StarShift => Color::new(0.75, 0.85, 1.0, 1.0),
    }
}

impl MoveFx {
    pub fn absorb(&mut self, ev: &Event) {
        match *ev {
            Event::Dash { kind, from, to } => self.flights.push((kind, v(from), v(to), 0.0)),
            Event::DashLand { kind, pos } => self.lands.push((kind, v(pos), 0.0)),
            _ => {}
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.flights.iter_mut().for_each(|f| f.3 += dt);
        self.flights.retain(|f| f.3 < FLIGHT);
        self.lands.iter_mut().for_each(|l| l.2 += dt);
        self.lands.retain(|l| l.2 < LAND);
    }

    pub fn clear(&mut self) {
        self.flights.clear();
        self.lands.clear();
    }
}

/// How high a leaping hero is off the ground (world units).
pub fn lift(world: &World) -> f32 {
    match world.dash.active {
        Some(r) if move_def(r.kind).leap => {
            let k = (r.t / r.dur).clamp(0.0, 1.0);
            (k * std::f32::consts::PI).sin() * 16.0
        }
        _ => 0.0,
    }
}

/// What moves left on the ground, and Kuon's marker (under the hero).
pub fn draw_ground(world: &World, a: &Assets, t: f32) {
    let vx = &a.vfx;
    for tr in &world.trails {
        let (p0, p1) = (v(tr.a), v(tr.b));
        let d = p1 - p0;
        let len = d.length().max(1.0);
        let ang = d.y.atan2(d.x);
        let fade = (tr.life / 0.4).min(1.0) * ((tr.max_life - tr.life) / 0.12).min(1.0);
        match tr.kind {
            TrailKind::Ofuda => {
                vx.additive(|| vx.draw_wh("beam", (p0 + p1) * 0.5, vec2(len + 10.0, tr.width * 1.6), ang, Color::new(1.0, 0.82, 0.45, 0.28 * fade)));
                let n = (len / 11.0) as usize + 1;
                for k in 0..=n {
                    let q = p0 + d * (k as f32 / n as f32);
                    let bob = (t * 4.0 + k as f32 * 1.7).sin();
                    vx.draw("ofuda", q + vec2(0.0, -4.0 + bob), 9.0, ang + 1.57 + bob * 0.15, Color::new(1.0, 1.0, 1.0, fade));
                }
            }
            TrailKind::Notes => {
                vx.additive(|| {
                    vx.draw_wh("beam", (p0 + p1) * 0.5, vec2(len + 12.0, tr.width * 1.4), ang, Color::new(1.0, 0.55, 0.85, 0.22 * fade));
                    let n = (len / 14.0) as usize + 1;
                    for k in 0..=n {
                        let q = p0 + d * (k as f32 / n as f32);
                        let hop = ((t * 3.0 + k as f32 * 0.9).sin() * 0.5 + 0.5) * 5.0;
                        vx.draw(if k % 2 == 0 { "star" } else { "sparkle" }, q + vec2(0.0, -6.0 - hop), 8.0, t + k as f32, Color::new(1.0, 0.7, 0.9, 0.85 * fade));
                    }
                });
            }
            TrailKind::Cut => {
                if tr.delay > 0.0 {
                    let k = 1.0 - tr.delay / 0.22;
                    vx.additive(|| vx.draw_wh("streak", (p0 + p1) * 0.5 + vec2(0.0, -8.0), vec2(len, 1.5 + 2.5 * k), ang, Color::new(0.85, 0.92, 1.0, 0.35 + 0.5 * k)));
                }
            }
        }
    }
    if let Some(s) = world.dash.star {
        let s = v(s);
        let pulse = 0.5 + 0.5 * (t * 3.0).sin();
        vx.additive(|| {
            vx.draw_wh("circle", s + vec2(0.0, 2.0), vec2(30.0, 15.0), t * 0.6, Color::new(0.6, 0.75, 1.0, 0.35));
            vx.draw("glow", s + vec2(0.0, -10.0), 26.0, 0.0, Color::new(0.5, 0.65, 1.0, 0.3 + 0.15 * pulse));
            vx.draw("star", s + vec2(0.0, -12.0 - 2.0 * pulse), 13.0, t * 0.8, Color::new(0.85, 0.92, 1.0, 0.9));
        });
    }
}

/// The moves in flight and their landings (over the ground, round the hero).
pub fn draw_air(world: &World, fx: &MoveFx, a: &Assets, t: f32) {
    use MoveKind::*;
    let vx = &a.vfx;
    // While travelling: a smear of motion trailing the hero.
    if let Some(r) = world.dash.active {
        let c = tint(r.kind);
        let p = v(world.player.pos) + vec2(0.0, -12.0 - lift(world));
        let back = (v(r.from) - v(r.to)).normalize_or_zero();
        vx.additive(|| {
            for k in 0..4 {
                let q = p + back * (6.0 + k as f32 * 7.0);
                vx.draw("glow", q, 16.0 - k as f32 * 3.0, 0.0, Color::new(c.r, c.g, c.b, 0.35 - k as f32 * 0.07));
            }
            vx.draw_wh("streak", p + back * 16.0, vec2(34.0, 10.0), back.y.atan2(back.x), Color::new(c.r, c.g, c.b, 0.6));
        });
    }
    for &(kind, from, to, age) in &fx.flights {
        let k = age / FLIGHT;
        let fade = 1.0 - k;
        let c = tint(kind);
        let (f, tt) = (from + vec2(0.0, -12.0), to + vec2(0.0, -12.0));
        let d = tt - f;
        let len = d.length().max(1.0);
        let ang = d.y.atan2(d.x);
        let mid = (f + tt) * 0.5;
        vx.additive(|| match kind {
            LightningFlash => {
                vx.draw_wh("lightning", mid, vec2(len + 10.0, 26.0 * fade), ang, Color::new(0.75, 0.88, 1.0, fade));
                vx.draw_wh("beam", mid, vec2(len, 14.0 * fade), ang, Color::new(0.5, 0.7, 1.0, 0.6 * fade));
                vx.draw("glow", f, 30.0 * fade, 0.0, Color::new(0.6, 0.8, 1.0, 0.6 * fade));
                vx.draw("spark_ball", tt, 22.0 * (0.5 + k), t * 9.0, Color::new(0.8, 0.9, 1.0, fade));
            }
            FoxBlink | VeilCrossing | StarShift => {
                let (puff, col) = match kind {
                    FoxBlink => ("foxspirit", c),
                    VeilCrossing => ("hellfire", Color::new(1.0, 0.3, 0.35, 1.0)),
                    _ => ("sparkle", c),
                };
                vx.draw_wh("streak", mid, vec2(len, 5.0 * fade), ang, Color::new(col.r, col.g, col.b, 0.5 * fade));
                vx.draw(puff, f, 26.0 * (0.7 + 0.6 * k), t, Color::new(col.r, col.g, col.b, 0.8 * fade));
                vx.draw(puff, tt, 22.0 * (1.0 + 0.5 * (1.0 - k)), -t, Color::new(col.r, col.g, col.b, 0.7 * fade));
                vx.draw("glow", tt, 34.0 * fade, 0.0, Color::new(col.r, col.g, col.b, 0.4 * fade));
            }
            IaidoRush => {
                vx.draw_wh("streak", mid, vec2(len + 20.0, 4.0 + 6.0 * fade), ang, Color::new(1.0, 1.0, 1.0, 0.8 * fade));
                vx.draw_wh("beam", mid, vec2(len, 10.0 * fade), ang, Color::new(0.6, 0.75, 1.0, 0.4 * fade));
            }
            FlaskJump => {
                vx.draw("flame", f + vec2(0.0, 6.0 - 20.0 * k), 22.0 * fade + 8.0, 0.0, Color::new(1.0, 0.55, 0.25, 0.8 * fade));
            }
            _ => {
                vx.draw_wh("beam", mid, vec2(len, 12.0 * fade), ang, Color::new(c.r, c.g, c.b, 0.35 * fade));
            }
        });
        if matches!(kind, FlaskJump | VeilCrossing | FoxBlink) {
            vx.draw("smoke", f + vec2(0.0, 4.0 - 10.0 * k), 18.0 + 22.0 * k, k, Color::new(0.75, 0.72, 0.7, 0.4 * fade));
        }
    }
    for &(kind, p, age) in &fx.lands {
        let k = age / LAND;
        let fade = 1.0 - k;
        let c = tint(kind);
        let g = p + vec2(0.0, 4.0);
        // Dust kicked up, a ring on the ground.
        for s in 0..3 {
            let a0 = s as f32 * 2.1 + p.x * 0.1;
            let q = g + vec2(a0.cos() * (6.0 + 14.0 * k), -3.0 - 6.0 * k + a0.sin() * 2.0);
            vx.draw("smoke", q, 10.0 + 14.0 * k, a0, Color::new(0.72, 0.66, 0.58, 0.35 * fade));
        }
        vx.additive(|| vx.draw_wh("ring", g, vec2(1.0, 0.5) * (14.0 + 40.0 * k), 0.0, Color::new(c.r, c.g, c.b, 0.6 * fade)));
    }
}
