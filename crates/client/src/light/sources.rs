//! What gives off light: the one place to say so. Adding a glowing enemy, projectile, pickup
//! or prop means an entry here; the draw code never changes (docs/lighting.md).
//!
//! Projectiles and pickups are matched exhaustively, so a new kind won't build until it says
//! whether (and how) it glows. Enemies list only the ones that glow.
use macroquad::prelude::*;
use sim::content::{EnemyKind, PickupKind};
use sim::world::ProjKind;
use sim::World;

use super::Lights;

/// A light carried by something: reach (world units), colour added at the centre (light-map
/// units, 0.5 = doubles the art), and how far above its ground point it shines from.
#[derive(Clone, Copy)]
pub struct Glow {
    pub radius: f32,
    pub color: Color,
    pub lift: f32,
}

const fn glow(radius: f32, r: f32, g: f32, b: f32, lift: f32) -> Glow {
    Glow { radius, color: Color { r, g, b, a: 1.0 }, lift }
}

/// The hero's own small light, so they're never lost in the dark (casts no shadow).
pub const HERO: Glow = glow(170.0, 0.16, 0.15, 0.13, 10.0);

/// A player attack's light (its trail through the dark), or none.
pub fn projectile(k: ProjKind) -> Option<Glow> {
    use ProjKind::*;
    Some(match k {
        Talisman => glow(34.0, 0.07, 0.06, 0.05, 0.0),
        Foxfire | Comet | Spark => glow(46.0, 0.04, 0.10, 0.16, 0.0),
        GoldenOrb | Seal | SealOrb => glow(48.0, 0.14, 0.11, 0.04, 0.0),
        Familiar => glow(40.0, 0.08, 0.07, 0.10, 0.0),
        CrescentWave => glow(60.0, 0.14, 0.05, 0.10, 0.0),
        Petal => glow(30.0, 0.10, 0.05, 0.08, 0.0),
        SpiritBolt => glow(40.0, 0.03, 0.10, 0.12, 0.0),
        Fang => glow(36.0, 0.04, 0.12, 0.04, 0.0),
        RedStar => glow(50.0, 0.16, 0.04, 0.04, 0.0),
        VoidOrb => glow(56.0, 0.10, 0.03, 0.16, 0.0),
        Shuriken => return None, // steel: catches light, gives none
    })
}

/// A pickup's light (they glow so they're found in the dark).
pub fn pickup(k: PickupKind) -> Glow {
    use PickupKind::*;
    match k {
        Feast => glow(50.0, 0.16, 0.12, 0.08, 4.0),
        SwiftGeta => glow(50.0, 0.10, 0.14, 0.12, 4.0),
        OniSake => glow(50.0, 0.18, 0.08, 0.06, 4.0),
        ShrineBell | SenPurse => glow(54.0, 0.18, 0.15, 0.06, 4.0),
        Hourglass => glow(50.0, 0.06, 0.12, 0.18, 4.0),
        ExorcismSeal => glow(56.0, 0.16, 0.14, 0.10, 4.0),
    }
}

/// An enemy that carries a light (lanterns, fire spirits); most don't.
pub fn enemy(k: EnemyKind) -> Option<Glow> {
    use EnemyKind::*;
    Some(match k {
        Chochin => glow(60.0, 0.30, 0.17, 0.05, 4.0),
        LanternFairy | Kagerou => glow(70.0, 0.30, 0.18, 0.06, 10.0),
        NightLord => glow(200.0, 0.45, 0.25, 0.08, 24.0),
        _ => return None,
    })
}

/// A stage prop's light, from the colour its art glows (stage.rs `world_glows`).
pub fn prop(c: Color, flicker: f32) -> Glow {
    let k = 0.3 * flicker;
    glow(160.0, c.r * k, c.g * k, c.b * k, 0.0)
}

/// Adds `g` shining from `p` (its ground point): a steady light that shadows fall away from.
pub fn emit(l: &mut Lights, p: Vec2, g: Glow) {
    l.add(p - vec2(0.0, g.lift), g.radius, g.color);
}

/// Adds a passing light (moving, or there for a moment): it lights things but steers no
/// shadows, so they don't swing about as it goes by.
pub fn emit_passing(l: &mut Lights, p: Vec2, g: Glow) {
    l.add_own(p - vec2(0.0, g.lift), g.radius, g.color);
}

/// Everything in the world that glows, near the view: the shrine, fire pools, soul lanterns,
/// glowing enemies, the player's attacks, enemy fire, chests, pickups and gems.
pub fn gather(world: &World, l: &mut Lights, view: Rect, time: f64) {
    let v = |p: sim::Vec2| vec2(p.x, p.y);
    let near = |p: sim::Vec2, pad: f32| p.x > view.x - pad && p.x < view.x + view.w + pad && p.y > view.y - pad && p.y < view.y + view.h + pad;
    let flick = |k: f32| 0.88 + 0.12 * ((time * 9.0) as f32 + k).sin();
    if let Some(s) = world.shrine {
        emit(l, v(s), glow(150.0, 0.38, 0.30, 0.14, 14.0));
    }
    for pl in &world.pools {
        let k = (pl.life / pl.max_life * 3.0).min(1.0) * flick(pl.pos.x);
        emit(l, v(pl.pos), glow(pl.radius * 2.6, 0.55 * k, 0.24 * k, 0.05 * k, 0.0));
    }
    for ln in &world.lanterns {
        emit(l, v(ln.pos), glow(ln.radius * 1.6, 0.10, 0.26, 0.32, 14.0));
    }
    let e = &world.enemies;
    for i in 0..e.len() {
        if let Some(g) = enemy(e.kind[i]).filter(|g| near(e.pos[i], g.radius)) {
            emit_passing(l, v(e.pos[i]), g);
        }
    }
    // Attacks light their path; with hundreds in flight only some of them do (it adds up).
    let pr = &world.projectiles;
    let step = 1 + pr.pos.len() / 150;
    for i in (0..pr.pos.len()).step_by(step) {
        if let Some(g) = projectile(pr.kind[i]).filter(|_| near(pr.pos[i], 60.0)) {
            emit_passing(l, v(pr.pos[i]), Glow { radius: g.radius + pr.radius[i] * 2.0, ..g });
        }
    }
    for sh in world.shots.iter().filter(|s| near(s.pos, 30.0)) {
        let c = crate::stage::bullet_color(sh.style);
        emit_passing(l, v(sh.pos), glow(30.0, 0.08 + c.r * 0.16, c.g * 0.12, c.b * 0.12, 0.0));
    }
    for (c, _) in world.chests.iter().filter(|c| near(c.0, 70.0)) {
        emit_passing(l, v(*c), glow(70.0, 0.30, 0.24, 0.10, 4.0));
    }
    for (p, k) in world.pickups.iter().filter(|p| near(p.0, 60.0)) {
        emit_passing(l, v(*p), pickup(*k));
    }
    for g in world.gems.pos.iter().step_by(3).filter(|g| near(**g, 24.0)) {
        emit_passing(l, v(*g), glow(24.0, 0.01, 0.04, 0.06, 0.0));
    }
}
