//! Cut-out rig heroes: a hero split into layered parts (tools/ai/rig_cut.py) and animated in
//! code, every frame. `tools/ai/rig_preview.py` is the reference for all of the motion here.
//!
//! - Primary motion (body bob, lean, squash, legs, foot lift) comes from keyed curves, blended
//!   smoothly between idle and walk.
//! - Secondary motion follows through damped springs, stepped at a fixed 240 Hz: the head lags
//!   the body, each arm joint drags and overshoots, the hair is a 5-link spring chain driven by
//!   the head's acceleration (and the player starting, stopping and turning), with the outer
//!   locks trailing the inner ones, and the skirt sways and flares on each bounce.
//! - Hair and skirt are drawn as bending meshes; the other parts are rigid quads.
//!
//! All rig geometry is in the source image's pixels (1024 canvas); `units_per_px` and `feet`
//! map that onto the world.
use std::f32::consts::TAU;

use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;

use crate::sprites;

const SIM: f32 = 240.0;
const HIST: usize = 32;
const HAIR_LINKS: usize = 5;

/// Draw order, back to front.
const PARTS: [&str; 9] = ["hair_back", "arm_far_upper", "arm_far_fore", "leg_far", "leg_near", "torso", "head", "arm_near_upper", "arm_near_fore"];
const HAIR: usize = 0;
const ARM_FAR_UPPER: usize = 1;
const ARM_FAR_FORE: usize = 2;
const LEG_FAR: usize = 3;
const LEG_NEAR: usize = 4;
const TORSO: usize = 5;
const HEAD: usize = 6;
const ARM_NEAR_UPPER: usize = 7;
const ARM_NEAR_FORE: usize = 8;

struct Part {
    tex: Texture2D,
    /// x, y, w, h in source pixels.
    rect: Rect,
    pivot: Vec2,
}

/// A hero's parts and geometry (art/hd/rig/<hero>/*.png + art/rig/<hero>/game.json).
pub struct RigDef {
    parts: Vec<Part>,
    feet: Vec2,
    units_per_px: f32,
}

impl RigDef {
    pub fn load(hero: &str, json: &str) -> Self {
        let v: serde_json::Value = serde_json::from_str(json).expect("bad rig json");
        let pair = |x: &serde_json::Value| vec2(x[0].as_f64().unwrap() as f32, x[1].as_f64().unwrap() as f32);
        let parts = PARTS
            .iter()
            .map(|name| {
                let p = &v["parts"][name];
                let r = &p["rect"];
                let f = |i: usize| r[i].as_f64().unwrap() as f32;
                Part { tex: sprites::hi_art(&format!("rig/{hero}/{name}")), rect: Rect::new(f(0), f(1), f(2), f(3)), pivot: pair(&p["pivot"]) }
            })
            .collect();
        RigDef { parts, feet: pair(&v["feet"]), units_per_px: v["units_per_px"].as_f64().unwrap() as f32 }
    }
}

// --- springs --------------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct Spring {
    w: f32,
    z: f32,
    x: f32,
    v: f32,
}

impl Spring {
    fn new(hz: f32, zeta: f32) -> Self {
        Spring { w: TAU * hz, z: zeta, x: 0.0, v: 0.0 }
    }

    fn step(&mut self, target: f32, dt: f32, force: f32) {
        let a = self.w * self.w * (target - self.x) - 2.0 * self.z * self.w * self.v + force;
        self.v += a * dt;
        self.x += self.v * dt;
    }
}

/// A hanging strand: each link chases its parent, so motion travels down it.
#[derive(Clone, Copy)]
struct Chain<const N: usize> {
    links: [Spring; N],
}

impl<const N: usize> Chain<N> {
    fn new(hz: f32, zeta: f32) -> Self {
        Chain { links: std::array::from_fn(|i| Spring::new(hz * 0.85f32.powi(i as i32), zeta)) }
    }

    fn step(&mut self, rest: f32, dt: f32, force: f32) {
        let mut prev = rest;
        for (i, l) in self.links.iter_mut().enumerate() {
            l.step(prev, dt, force * (1.0 + 0.35 * i as f32));
            prev = l.x;
        }
    }

    fn sum(&self) -> f32 {
        self.links.iter().map(|l| l.x).sum()
    }
}

/// Soft limit: linear near 0, never past +-lim.
fn soft(x: f32, lim: f32) -> f32 {
    lim * (x / lim).tanh()
}

// --- keyed motion ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct Pose {
    bob: f32,
    lean: f32,
    squash: f32,
    leg: [f32; 2],
    lift: [f32; 2],
    upper: [f32; 2],
    fore: [f32; 2],
    wind: f32,
}

impl Pose {
    fn idle(t: f32) -> Pose {
        let b = (TAU * t / 3.0).sin();
        Pose {
            bob: 4.0 * b,
            lean: 0.0,
            squash: 0.006 * b,
            leg: [0.0; 2],
            lift: [0.0; 2],
            upper: [3.0 + 2.0 * b, -3.0 - 2.0 * b],
            fore: [6.0 + 2.0 * b; 2],
            wind: 0.012 * (TAU * t / 1.5).sin() + 0.004,
        }
    }

    /// `p` is the walk phase in radians (one stride pair per TAU).
    fn walk(p: f32) -> Pose {
        let (s, c) = p.sin_cos();
        let contact = s * s; // 1 when the legs are fully apart (feet strike)
        Pose {
            bob: 16.0 * contact - 6.0, // low on contact, high while passing
            lean: -4.0,
            squash: 0.03 * contact.powi(3),
            leg: [26.0 * s, -26.0 * s],
            lift: [0.16 * c.max(0.0).powf(1.5), 0.16 * (-c).max(0.0).powf(1.5)],
            upper: [-22.0 * s, 20.0 * s],
            fore: [14.0 + 12.0 * (-s).max(0.0), 14.0 + 12.0 * s.max(0.0)],
            wind: -0.07,
        }
    }

    fn lerp(a: Pose, b: Pose, k: f32) -> Pose {
        let l = |x: f32, y: f32| x + (y - x) * k;
        let l2 = |x: [f32; 2], y: [f32; 2]| [l(x[0], y[0]), l(x[1], y[1])];
        Pose {
            bob: l(a.bob, b.bob),
            lean: l(a.lean, b.lean),
            squash: l(a.squash, b.squash),
            leg: l2(a.leg, b.leg),
            lift: l2(a.lift, b.lift),
            upper: l2(a.upper, b.upper),
            fore: l2(a.fore, b.fore),
            wind: l(a.wind, b.wind),
        }
    }
}

const WALK_PERIOD: f32 = 0.8;

// --- live state -----------------------------------------------------------------------------

/// One rigged hero's animation state (lives in `Fx`, advanced once per frame).
pub struct RigState {
    head: Spring,
    head_dy: Spring,
    upper: [Spring; 2],
    fore: [Spring; 2],
    hair: Chain<HAIR_LINKS>,
    hair_spread: Spring,
    skirt: Chain<2>,
    skirt_flare: Spring,
    flinch: Spring,
    jolt: Spring,
    hist: [[f32; HAIR_LINKS]; HIST],
    hist_i: usize,
    prev: Option<(Vec2, Vec2)>,
    player_prev: Option<Vec2>,
    player_vel: f32,
    acc: f32,
    t: f32,
    walk: f32,
    phase: f32,
    pose: Pose,
    hurt: f32,
}

impl Default for RigState {
    fn default() -> Self {
        RigState {
            head: Spring::new(4.5, 0.45),
            head_dy: Spring::new(5.0, 0.4),
            upper: [Spring::new(3.2, 0.55); 2],
            fore: [Spring::new(4.0, 0.5); 2],
            hair: Chain::new(2.2, 0.32),
            hair_spread: Spring::new(2.6, 0.3),
            skirt: Chain::new(4.0, 0.35),
            skirt_flare: Spring::new(4.5, 0.3),
            flinch: Spring::new(5.0, 0.4),
            jolt: Spring::new(6.0, 0.35),
            hist: [[0.0; HAIR_LINKS]; HIST],
            hist_i: 0,
            prev: None,
            player_prev: None,
            player_vel: 0.0,
            acc: 0.0,
            t: 0.0,
            walk: 0.0,
            phase: 0.0,
            pose: Pose::idle(0.0),
            hurt: 0.0,
        }
    }
}

impl RigState {
    /// The hero took a hit: flinch back, squash, and flash.
    pub fn hurt(&mut self) {
        self.flinch.v += 140.0;
        self.jolt.v += 0.5;
        self.head.v += 80.0;
        self.hurt = 0.15;
    }

    /// Advance by `dt` seconds. `player` is the hero's world position; `left` = facing left.
    pub fn update(&mut self, def: &RigDef, dt: f32, moving: bool, player: Vec2, left: bool) {
        let dt = dt.clamp(0.0, 0.1);
        self.hurt = (self.hurt - dt).max(0.0);
        // The player's own speed along the facing direction (source px/s), so starting,
        // stopping and turning swing the hair. Teleports are ignored.
        if let Some(prev) = self.player_prev {
            let d = player - prev;
            if dt > 0.0 && d.length() < 20.0 {
                let fwd = if left { -d.x } else { d.x };
                self.player_vel = (fwd / dt / def.units_per_px).clamp(-3000.0, 3000.0);
            }
        }
        self.player_prev = Some(player);

        self.acc += dt;
        let h = 1.0 / SIM;
        let mut steps = 0;
        while self.acc >= h && steps < 40 {
            self.acc -= h;
            steps += 1;
            self.t += h;
            // Ease between idle and walk; a fresh walk starts with the legs together.
            let target = if moving { 1.0 } else { 0.0 };
            if moving && self.walk < 0.02 {
                self.phase = 0.0;
            }
            self.walk += (target - self.walk) * (1.0 - (-h * 9.0).exp());
            self.phase = (self.phase + h * TAU / WALK_PERIOD) % TAU;
            let q = Pose::lerp(Pose::idle(self.t), Pose::walk(self.phase), self.walk);
            self.step(def, q, h);
            self.pose = q;
        }
    }

    fn step(&mut self, def: &RigDef, q: Pose, dt: f32) {
        let head_pivot = def.parts[HEAD].pivot;
        let ang = (q.lean + self.flinch.x).to_radians();
        let pos = vec2(def.feet.x + (head_pivot.y - def.feet.y) * -ang.sin(), q.bob);
        let vel_body = match self.prev {
            Some((p, _)) => (pos - p) / dt,
            None => Vec2::ZERO,
        };
        let vel = vel_body + vec2(self.player_vel * 0.25, 0.0);
        let acc = match self.prev {
            Some((_, v)) => (vel - v) / dt,
            None => Vec2::ZERO,
        };
        self.prev = Some((pos, vel));
        let (ax, ay) = (acc.x / 1000.0, acc.y / 1000.0); // body heights / s^2

        self.head.step((q.lean + self.flinch.x) * 0.7, dt, -ax * 250.0);
        self.head_dy.step(0.0, dt, -ay * 1000.0);
        for i in 0..2 {
            self.upper[i].step(q.upper[i], dt, -ax * 600.0);
            self.fore[i].step(q.fore[i], dt, -ax * 450.0);
        }
        self.hair.step(q.wind, dt, -ax * 2.5);
        self.hair_spread.step(0.0, dt, ay * 3.0);
        self.skirt.step(q.wind * 0.6 - (q.leg[0] - q.leg[1]).to_radians() * 0.05, dt, -ax * 1.5);
        self.skirt_flare.step(0.0, dt, ay * 2.5);
        self.flinch.step(0.0, dt, 0.0);
        self.jolt.step(0.0, dt, 0.0);
        self.hist_i = (self.hist_i + 1) % HIST;
        self.hist[self.hist_i] = std::array::from_fn(|i| self.hair.links[i].x);
    }

    /// The hair chain as it was `lag` steps ago.
    fn hair_at(&self, lag: usize) -> &[f32; HAIR_LINKS] {
        &self.hist[(self.hist_i + HIST - lag.min(HIST - 1)) % HIST]
    }
}

// --- drawing --------------------------------------------------------------------------------

/// cv2-style rotation about `pivot` (+ = counter-clockwise on screen), then scale and shift.
fn rot(pivot: Vec2, deg: f32, shift: Vec2, scale: Vec2) -> Affine2 {
    let (s, c) = deg.to_radians().sin_cos();
    let m = Mat2::from_cols(vec2(c * scale.x, -s * scale.x), vec2(s * scale.y, c * scale.y));
    Affine2::from_mat2_translation(m, pivot - m * pivot + shift)
}

fn turn(pivot: Vec2, deg: f32) -> Affine2 {
    rot(pivot, deg, Vec2::ZERO, Vec2::ONE)
}

/// Draws a part as a `nx` x `ny` grid, each vertex shifted sideways by `bend(u, v)` (source px,
/// u/v = 0..1 across the part) before `m` maps it to the world.
fn draw_part(part: &Part, m: Affine2, tint: Color, nx: usize, ny: usize, bend: impl Fn(f32, f32) -> f32) {
    let r = part.rect;
    let mut vertices = Vec::with_capacity((nx + 1) * (ny + 1));
    for j in 0..=ny {
        let v = j as f32 / ny as f32;
        for i in 0..=nx {
            let u = i as f32 / nx as f32;
            let p = m.transform_point2(vec2(r.x + r.w * u + bend(u, v), r.y + r.h * v));
            vertices.push(Vertex::new(p.x, p.y, 0.0, u, v, tint));
        }
    }
    let mut indices = Vec::with_capacity(nx * ny * 6);
    for j in 0..ny {
        for i in 0..nx {
            let a = (j * (nx + 1) + i) as u16;
            let b = a + nx as u16 + 1;
            indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    draw_mesh(&Mesh { vertices, indices, texture: Some(part.tex.clone()) });
}

/// Draws the hero with its feet at `feet` (world units), mirrored when `left`.
pub fn draw(def: &RigDef, st: &RigState, feet: Vec2, left: bool, tint: Color) {
    let k = def.units_per_px;
    let world = Affine2::from_translation(feet) * Affine2::from_scale(vec2(if left { -k } else { k }, k)) * Affine2::from_translation(-def.feet);
    let q = st.pose;
    let tint = if st.hurt > 0.0 { Color::new(tint.r, tint.g * 0.55, tint.b * 0.6, tint.a) } else { tint };
    let p = |i: usize| def.parts[i].pivot;
    let sq = q.squash + st.jolt.x;
    let body = world * rot(def.feet, q.lean + st.flinch.x, vec2(0.0, q.bob), vec2(1.0 + sq * 0.6, 1.0 - sq));
    let head = body * rot(p(HEAD), st.head.x, vec2(0.0, st.head_dy.x), Vec2::ONE);
    let upper_near = body * turn(p(ARM_NEAR_UPPER), st.upper[0].x);
    let upper_far = body * turn(p(ARM_FAR_UPPER), st.upper[1].x);
    let fore_near = upper_near * turn(p(ARM_NEAR_FORE), st.fore[0].x);
    let fore_far = upper_far * turn(p(ARM_FAR_FORE), st.fore[1].x);
    let leg = |i: usize, deg: f32, lift: f32| world * rot(p(i), deg, vec2(0.0, q.bob * 0.35), vec2(1.0, 1.0 - lift));
    let rigid = |_: f32, _: f32| 0.0;

    // Hair: the chain's shape down its length; outer locks sample the chain a little in the
    // past and ripple, so the strands separate instead of moving as one sheet.
    let hair = &def.parts[HAIR];
    let (hh, top) = (hair.rect.h, 0.3);
    let link_v = st.hair.links.iter().map(|l| l.v).sum::<f32>() / HAIR_LINKS as f32;
    let ripple_amp = hh * 0.006 * (0.4 + 6.0 * link_v.abs());
    let spread = soft(st.hair_spread.x, 0.5);
    let len = hh * (1.0 - top) * 0.9 / HAIR_LINKS as f32;
    let hair_bend = |u: f32, v: f32| {
        let side = u * 2.0 - 1.0;
        let depth = ((v - top) / (1.0 - top)).clamp(0.0, 1.0);
        let th = st.hair_at((side.abs() * 0.09 * SIM) as usize);
        let seg = depth * HAIR_LINKS as f32;
        let disp: f32 = th.iter().enumerate().map(|(i, a)| len * soft(*a, 0.22) * (seg - i as f32).clamp(0.0, 1.0)).sum();
        let ripple = (TAU * 1.7 * st.t - depth * 7.0 + side * 6.0).sin() * depth * depth * ripple_amp;
        (0.8 + 0.45 * side.abs()) * disp + spread * side * depth.powf(1.5) * hh * 0.08 + ripple
    };
    // Skirt: the bottom 40% of the torso sways with the legs and flares on each bounce.
    let th = def.parts[TORSO].rect.h * 0.4;
    let sway = soft(st.skirt.sum(), 0.15) * th * 0.6;
    let flare = soft(st.skirt_flare.x, 0.4) * th * 0.25;
    let skirt_bend = |u: f32, v: f32| {
        let d = ((v - 0.6) / 0.4).clamp(0.0, 1.0).powf(1.3);
        d * (sway + flare * (u - 0.5) * 2.0)
    };

    let parts = &def.parts;
    draw_part(&parts[HAIR], head, tint, 8, 16, hair_bend);
    draw_part(&parts[ARM_FAR_UPPER], upper_far, tint, 1, 1, rigid);
    draw_part(&parts[ARM_FAR_FORE], fore_far, tint, 1, 1, rigid);
    draw_part(&parts[LEG_FAR], leg(LEG_FAR, q.leg[1], q.lift[1]), tint, 1, 1, rigid);
    draw_part(&parts[LEG_NEAR], leg(LEG_NEAR, q.leg[0], q.lift[0]), tint, 1, 1, rigid);
    draw_part(&parts[TORSO], body, tint, 6, 14, skirt_bend);
    draw_part(&parts[HEAD], head, tint, 1, 1, rigid);
    draw_part(&parts[ARM_NEAR_UPPER], upper_near, tint, 1, 1, rigid);
    draw_part(&parts[ARM_NEAR_FORE], fore_near, tint, 1, 1, rigid);
}
