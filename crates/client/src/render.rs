//! Draws the world: ground, pickups, enemies, player, weapons, effects.

use macroquad::prelude::*;
use sim::content::{EnemyKind, PassiveKind, Upgrade, WeaponKind};
use sim::{Event, World};

use crate::sprites;

/// World pixels → screen pixels (integer for crisp pixel art).
pub const SCALE: f32 = 2.0;

pub struct Assets {
    pub hero: [Texture2D; 3], // idle, walk A, walk B
    pub wisp: Texture2D,
    pub kasa: Texture2D,
    pub oni: Texture2D,
    pub talisman: Texture2D,
    pub orb: Texture2D,
    pub gem: Texture2D,
    pub chest: Texture2D,
    pub chest_open: Texture2D,
    icons: Vec<(&'static str, Texture2D)>,
}

impl Assets {
    pub fn load() -> Self {
        let hero = sprites::parse(include_str!("../../../art/src/heroine.px"));
        // Legs occupy rows 27..32; left leg columns 5..10, right leg 14..19.
        let walk_a = hero.lift(5..10, 27..32);
        let walk_b = hero.lift(14..19, 27..32);
        Assets {
            hero: [hero.texture(), walk_a.texture(), walk_b.texture()],
            wisp: sprites::texture(include_str!("../../../art/src/wisp.px")),
            kasa: sprites::texture(include_str!("../../../art/src/kasa.px")),
            oni: sprites::texture(include_str!("../../../art/src/oni.px")),
            talisman: sprites::texture(include_str!("../../../art/src/talisman.px")),
            orb: sprites::texture(include_str!("../../../art/src/orb.px")),
            gem: sprites::texture(include_str!("../../../art/src/gem.px")),
            chest: sprites::texture(include_str!("../../../art/src/chest.px")),
            chest_open: sprites::texture(include_str!("../../../art/src/chest_open.px")),
            icons: vec![
            ("abundance", sprites::texture(include_str!("../../../art/src/icons/abundance.px"))),
            ("crescent", sprites::texture(include_str!("../../../art/src/icons/crescent.px"))),
            ("expanse", sprites::texture(include_str!("../../../art/src/icons/expanse.px"))),
            ("flame_pool", sprites::texture(include_str!("../../../art/src/icons/flame_pool.px"))),
            ("guard", sprites::texture(include_str!("../../../art/src/icons/guard.px"))),
            ("haste", sprites::texture(include_str!("../../../art/src/icons/haste.px"))),
            ("magnetism", sprites::texture(include_str!("../../../art/src/icons/magnetism.px"))),
            ("might", sprites::texture(include_str!("../../../art/src/icons/might.px"))),
            ("moon_orbs", sprites::texture(include_str!("../../../art/src/icons/moon_orbs.px"))),
            ("ofuda", sprites::texture(include_str!("../../../art/src/icons/ofuda.px"))),
            ("onigiri", sprites::texture(include_str!("../../../art/src/icons/onigiri.px"))),
            ("persistence", sprites::texture(include_str!("../../../art/src/icons/persistence.px"))),
            ("recovery", sprites::texture(include_str!("../../../art/src/icons/recovery.px"))),
            ("sakura_field", sprites::texture(include_str!("../../../art/src/icons/sakura_field.px"))),
            ("star_fall", sprites::texture(include_str!("../../../art/src/icons/star_fall.px"))),
            ("swiftness", sprites::texture(include_str!("../../../art/src/icons/swiftness.px"))),
            ("taiko", sprites::texture(include_str!("../../../art/src/icons/taiko.px"))),
            ("thunder_chain", sprites::texture(include_str!("../../../art/src/icons/thunder_chain.px"))),
            ("vitality", sprites::texture(include_str!("../../../art/src/icons/vitality.px"))),
            ],
        }
    }

    pub fn icon(&self, u: Upgrade) -> &Texture2D {
        let key = match u {
            Upgrade::Weapon(k) => match k {
                WeaponKind::Ofuda => "ofuda",
                WeaponKind::MoonOrbs => "moon_orbs",
                WeaponKind::SakuraField => "sakura_field",
                WeaponKind::ThunderChain => "thunder_chain",
                WeaponKind::Taiko => "taiko",
                WeaponKind::StarFall => "star_fall",
                WeaponKind::Crescent => "crescent",
                WeaponKind::FlamePool => "flame_pool",
            },
            Upgrade::Passive(p) => match p {
                PassiveKind::Might => "might",
                PassiveKind::Haste => "haste",
                PassiveKind::Expanse => "expanse",
                PassiveKind::Swiftness => "swiftness",
                PassiveKind::Magnet => "magnetism",
                PassiveKind::Vitality => "vitality",
                PassiveKind::Abundance => "abundance",
                PassiveKind::Persistence => "persistence",
                PassiveKind::Recovery => "recovery",
                PassiveKind::Guard => "guard",
            },
            Upgrade::Heal => "onigiri",
        };
        &self.icons.iter().find(|(n, _)| *n == key).expect("icon").1
    }
}

struct DamageNumber {
    pos: Vec2,
    amount: f32,
    age: f32,
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    age: f32,
    life: f32,
    color: Color,
}

/// Purely visual state derived from sim events.
#[derive(Default)]
pub struct Fx {
    numbers: Vec<DamageNumber>,
    particles: Vec<Particle>,
    /// Expanding barrier rings: (centre, age).
    rings: Vec<(Vec2, f32)>,
    /// Thunder Chain segments: (from, to, age).
    bolts: Vec<(Vec2, Vec2, f32)>,
    /// Crescent Slash arcs: (pos, dir, radius, full, age).
    slashes: Vec<(Vec2, Vec2, f32, bool, f32)>,
    /// Star Fall impacts: (pos, radius, age).
    impacts: Vec<(Vec2, f32, f32)>,
    pub shake: f32,
    pub show_numbers: bool,
    /// Short line spoken by the hero, shown above her head.
    pub bark: Option<(String, f32)>,
    /// Big centre-screen announcement.
    pub banner: Option<(String, f32)>,
    leveled_once: bool,
    pub low_hp_warned: bool,
}

const MAX_NUMBERS: usize = 120;
const MAX_PARTICLES: usize = 800;

impl Fx {
    pub fn new() -> Self {
        Fx { show_numbers: true, ..Default::default() }
    }

    pub fn clear(&mut self) {
        self.numbers.clear();
        self.particles.clear();
        self.rings.clear();
        self.bolts.clear();
        self.slashes.clear();
        self.impacts.clear();
        self.shake = 0.0;
        self.banner = None;
        self.leveled_once = false;
        self.low_hp_warned = false;
        self.say("Night shift. Again.");
    }

    pub fn say(&mut self, line: &str) {
        self.bark = Some((line.to_string(), 2.2));
    }

    pub fn announce(&mut self, text: String) {
        self.banner = Some((text, 2.8));
    }

    pub fn absorb(&mut self, events: &[Event]) {
        for ev in events {
            match *ev {
                Event::Hit { pos, amount } if self.show_numbers => {
                    if self.numbers.len() >= MAX_NUMBERS {
                        self.numbers.remove(0);
                    }
                    self.numbers.push(DamageNumber { pos: vec2(pos.x, pos.y - 8.0), amount, age: 0.0 });
                }
                Event::Kill { pos, kind } => {
                    let (n, color) = match kind {
                        EnemyKind::Wisp => (5, Color::from_rgba(155, 111, 214, 255)),
                        EnemyKind::Kasa => (6, Color::from_rgba(194, 59, 59, 255)),
                        EnemyKind::Oni => (24, Color::from_rgba(240, 122, 90, 255)),
                        EnemyKind::GreatOni => (80, Color::from_rgba(255, 200, 90, 255)),
                    };
                    for k in 0..n {
                        if self.particles.len() >= MAX_PARTICLES {
                            break;
                        }
                        let a = k as f32 / n as f32 * std::f32::consts::TAU + rand::gen_range(-0.3, 0.3);
                        let speed = rand::gen_range(30.0, 90.0);
                        self.particles.push(Particle {
                            pos: vec2(pos.x, pos.y),
                            vel: vec2(a.cos(), a.sin()) * speed,
                            age: 0.0,
                            life: rand::gen_range(0.25, 0.45),
                            color,
                        });
                    }
                }
                Event::PlayerHurt => self.shake = self.shake.max(0.18),
                Event::LevelUp if !self.leveled_once => {
                    self.leveled_once = true;
                    self.say("Noted. Moving on.");
                }
                Event::SpecialUsed { pos } => {
                    self.rings.push((vec2(pos.x, pos.y), 0.0));
                    self.shake = self.shake.max(0.25);
                    self.say("Purify!");
                }
                Event::BossSpawned => {
                    self.announce("A GREAT ONI APPROACHES".into());
                    self.say("You're not in my ledger. Let's fix that.");
                }
                Event::BossTelegraph => self.shake = self.shake.max(0.08),
                Event::Bolt { from, to } => self.bolts.push((vec2(from.x, from.y), vec2(to.x, to.y), 0.0)),
                Event::Slash { pos, dir, radius, full } => self.slashes.push((vec2(pos.x, pos.y), vec2(dir.x, dir.y), radius, full, 0.0)),
                Event::Impact { pos, radius } => {
                    self.impacts.push((vec2(pos.x, pos.y), radius, 0.0));
                    self.shake = self.shake.max(0.04);
                }
                Event::Awakened(k) => {
                    self.announce(format!("AWAKENING: {}", sim::content::awakened_name(k)));
                    self.say("Chapter thirteen starts here.");
                }
                _ => {}
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        for n in &mut self.numbers {
            n.age += dt;
            n.pos.y -= 28.0 * dt;
        }
        self.numbers.retain(|n| n.age < 0.6);
        for p in &mut self.particles {
            p.age += dt;
            p.pos += p.vel * dt;
            p.vel *= 0.9;
        }
        self.particles.retain(|p| p.age < p.life);
        for r in &mut self.rings {
            r.1 += dt;
        }
        self.rings.retain(|r| r.1 < 0.45);
        self.bolts.iter_mut().for_each(|b| b.2 += dt);
        self.bolts.retain(|b| b.2 < 0.16);
        self.slashes.iter_mut().for_each(|s| s.4 += dt);
        self.slashes.retain(|s| s.4 < 0.2);
        self.impacts.iter_mut().for_each(|i| i.2 += dt);
        self.impacts.retain(|i| i.2 < 0.3);
        self.shake = (self.shake - dt).max(0.0);
        for t in [&mut self.bark, &mut self.banner] {
            if let Some((_, left)) = t {
                *left -= dt;
                if *left <= 0.0 {
                    *t = None;
                }
            }
        }
    }
}

pub fn camera_for(world: &World, fx: &Fx) -> Camera2D {
    let (sw, sh) = (screen_width() / SCALE, screen_height() / SCALE);
    let p = world.player.pos;
    let jitter = if fx.shake > 0.0 { vec2(rand::gen_range(-2.0, 2.0), rand::gen_range(-2.0, 2.0)) } else { Vec2::ZERO };
    // Positive y zoom: macroquad already flips y when drawing to the screen, so this gives
    // y-down world coordinates (from_display_rect would flip twice and render upside down).
    Camera2D { target: vec2(p.x.round(), p.y.round()) + jitter, zoom: vec2(2.0 / sw, 2.0 / sh), ..Default::default() }
}

fn draw_ground(cam: &Camera2D) {
    let (sw, sh) = (screen_width() / SCALE, screen_height() / SCALE);
    let tile = 64.0;
    let (x0, y0) = (cam.target.x - sw / 2.0, cam.target.y - sh / 2.0);
    let (tx0, ty0) = ((x0 / tile).floor() as i32, (y0 / tile).floor() as i32);
    for ty in ty0..ty0 + (sh / tile) as i32 + 2 {
        for tx in tx0..tx0 + (sw / tile) as i32 + 2 {
            if (tx + ty).rem_euclid(2) == 0 {
                draw_rectangle(tx as f32 * tile, ty as f32 * tile, tile, tile, Color::from_rgba(34, 30, 50, 255));
            }
        }
    }
}

pub fn draw_world(world: &World, a: &Assets, fx: &Fx, cam: &Camera2D, time: f64) {
    set_camera(cam);
    draw_ground(cam);
    let p = world.player.pos;

    // Sakura Field sits on the ground under everything.
    for w in &world.weapons {
        if w.kind == WeaponKind::SakuraField {
            let r = world.field_radius_of(w);
            let pulse = (1.0 - (w.phase / 0.25).min(1.0)) * 0.12;
            let base = if w.awakened { Color::new(1.0, 0.75, 0.55, 0.14 + pulse) } else { Color::new(1.0, 0.55, 0.75, 0.10 + pulse) };
            draw_circle(p.x, p.y, r, base);
            draw_circle_lines(p.x, p.y, r, 1.0, Color::new(1.0, 0.7, 0.85, 0.55));
            for k in 0..10 {
                let ang = time as f32 * 0.8 + k as f32 * 0.628;
                let rr = r * (0.55 + 0.4 * ((k * 37 % 10) as f32 / 10.0));
                draw_rectangle(p.x + ang.cos() * rr, p.y + ang.sin() * rr, 2.0, 2.0, Color::from_rgba(255, 190, 215, 220));
            }
        }
    }

    for pl in &world.pools {
        let life = pl.life / pl.max_life;
        let flicker = 0.85 + 0.15 * ((time * 13.0) as f32 + pl.pos.x).sin();
        let fade = (life * 3.0).min(1.0) * flicker;
        draw_circle(pl.pos.x, pl.pos.y, pl.radius, Color::new(1.0, 0.45, 0.1, 0.45 * fade));
        draw_circle(pl.pos.x, pl.pos.y, pl.radius * 0.6, Color::new(1.0, 0.8, 0.3, 0.45 * fade));
        draw_circle_lines(pl.pos.x, pl.pos.y, pl.radius, 1.5, Color::new(1.0, 0.6, 0.2, 0.8 * fade));
        for k in 0..5 {
            let a = k as f32 * 1.256 + pl.pos.y * 0.1;
            let rise = ((time as f32 * 2.0 + k as f32 * 0.3) % 1.0) * 10.0;
            draw_rectangle(pl.pos.x + a.cos() * pl.radius * 0.6, pl.pos.y + a.sin() * pl.radius * 0.5 - rise, 2.0, 2.0, Color::new(1.0, 0.85, 0.3, 0.9));
        }
    }
    for st in &world.strikes {
        // Landing marker shrinks as the star approaches.
        let t = (st.delay / 0.6).clamp(0.0, 1.0);
        draw_circle_lines(st.pos.x, st.pos.y, st.radius * (0.4 + 0.6 * t), 1.5, Color::new(1.0, 0.95, 0.5, 0.7));
    }
    for g in &world.gems.pos {
        draw_texture(&a.gem, (g.x - 3.5).round(), (g.y - 3.5).round(), WHITE);
    }
    if let Some((g, _)) = world.gems.overflow {
        let bob = (time * 4.0).sin() as f32;
        draw_texture_ex(
            &a.gem,
            (g.x - 7.0).round(),
            (g.y - 7.0 + bob).round(),
            Color::new(1.0, 0.35, 0.45, 1.0),
            DrawTextureParams { dest_size: Some(vec2(14.0, 14.0)), ..Default::default() },
        );
    }

    for (c, tier) in &world.chests {
        let bob = ((time * 3.0).sin() * 1.5) as f32;
        let size = if *tier >= 3 { 2.0 } else { 1.0 };
        draw_circle(c.x, c.y + 2.0, 10.0 * size, Color::new(1.0, 0.85, 0.4, 0.12 + 0.06 * (time * 5.0).sin() as f32));
        draw_texture_ex(
            &a.chest,
            (c.x - 7.0 * size).round(),
            (c.y - 9.0 * size + bob).round(),
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(14.0 * size, 12.0 * size)), ..Default::default() },
        );
    }

    let e = &world.enemies;
    let flash = Color::new(1.0, 0.45, 0.55, 1.0);
    for i in 0..e.len() {
        let pos = e.pos[i];
        let tint = if e.flash[i] > 0 { flash } else { WHITE };
        let flip = e.kind[i] != EnemyKind::Wisp && pos.x > p.x;
        if e.kind[i] == EnemyKind::GreatOni {
            // Double-size oni; flashes and shows its line while winding up a charge.
            let winding = e.ai_state[i] == 1;
            if winding {
                let d = e.ai_dir[i];
                draw_line(pos.x, pos.y, pos.x + d.x * 230.0, pos.y + d.y * 230.0, 10.0, Color::new(1.0, 0.2, 0.2, 0.22));
            }
            let tint = if winding && ((time * 16.0) as i32) % 2 == 0 {
                Color::new(1.0, 1.0, 1.0, 1.0)
            } else if e.flash[i] > 0 {
                flash
            } else {
                Color::new(0.85, 0.7, 1.0, 1.0)
            };
            draw_ellipse(pos.x, pos.y + 6.0, 24.0, 7.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.35));
            draw_texture_ex(
                &a.oni,
                (pos.x - 32.0).round(),
                (pos.y - 52.0).round(),
                tint,
                DrawTextureParams { flip_x: flip, dest_size: Some(vec2(64.0, 64.0)), ..Default::default() },
            );
            continue;
        }
        let (tex, half, bottom) = match e.kind[i] {
            EnemyKind::Wisp => (&a.wisp, 8.0, 8.0),
            EnemyKind::Kasa => (&a.kasa, 8.0, 12.0),
            _ => (&a.oni, 16.0, 26.0),
        };
        draw_texture_ex(&tex, (pos.x - half).round(), (pos.y - bottom).round(), tint, DrawTextureParams { flip_x: flip, ..Default::default() });
    }

    // Player: walk cycle while moving, blink during i-frames.
    let pl = &world.player;
    let frame = if pl.moving { 1 + ((time * 8.0) as usize % 2) } else { 0 };
    let blink = pl.iframes > 0.0 && ((time * 20.0) as i32) % 2 == 0;
    let tint = if blink { Color::new(1.0, 1.0, 1.0, 0.35) } else { WHITE };
    draw_ellipse(p.x, p.y + 5.0, 9.0, 3.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.35));
    draw_texture_ex(
        &a.hero[frame],
        (p.x - 12.0).round(),
        (p.y - 26.0).round(),
        tint,
        DrawTextureParams { flip_x: pl.facing.x < 0.0, ..Default::default() },
    );
    // Small HP bar under the player.
    let frac = pl.hp / pl.max_hp;
    draw_rectangle(p.x - 10.0, p.y + 9.0, 20.0, 2.0, Color::from_rgba(40, 20, 30, 255));
    draw_rectangle(p.x - 10.0, p.y + 9.0, 20.0 * frac, 2.0, Color::from_rgba(255, 80, 110, 255));

    for w in &world.weapons {
        if w.kind == WeaponKind::MoonOrbs {
            let r = world.orb_radius(w.level);
            for c in world.orb_positions(w) {
                draw_circle(c.x, c.y, r * 1.3, Color::new(1.0, 0.95, 0.7, 0.18));
                draw_texture_ex(&a.orb, c.x - r, c.y - r, WHITE, DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() });
            }
        }
    }

    for wv in &world.waves {
        if wv.radius > 0.0 {
            let f = 1.0 - wv.radius / wv.max_radius;
            draw_circle_lines(wv.center.x, wv.center.y, wv.radius, 4.0, Color::new(1.0, 0.7, 0.3, 0.75 * f + 0.1));
            draw_circle_lines(wv.center.x, wv.center.y, wv.radius - 5.0, 1.5, Color::new(1.0, 0.95, 0.8, 0.5 * f));
        }
    }
    for st in &world.strikes {
        let drop = st.delay * 320.0;
        let (sx, sy) = (st.pos.x + drop * 0.35, st.pos.y - drop);
        draw_line(sx + 10.0, sy - 22.0, sx, sy, 3.0, Color::new(0.55, 0.85, 1.0, 0.6));
        draw_poly(sx, sy, 5, 5.0, time as f32 * 400.0, Color::new(1.0, 0.92, 0.4, 1.0));
    }
    for (c, r, age) in &fx.impacts {
        let t = age / 0.3;
        draw_circle(c.x, c.y, r * (0.6 + 0.6 * t), Color::new(1.0, 0.95, 0.6, 0.45 * (1.0 - t)));
    }
    for (from, to, age) in &fx.bolts {
        let a = 1.0 - age / 0.16;
        // Jagged line: offset midpoints perpendicular to the segment.
        let d = *to - *from;
        let n = vec2(-d.y, d.x).normalize_or_zero();
        let mut prev = *from;
        for k in 1..=4 {
            let t = k as f32 / 4.0;
            let jag = if k < 4 { ((k as f32 * 7.3 + from.x) .sin()) * 7.0 } else { 0.0 };
            let p2 = *from + d * t + n * jag;
            draw_line(prev.x, prev.y, p2.x, p2.y, 3.0, Color::new(0.75, 0.85, 1.0, a));
            draw_line(prev.x, prev.y, p2.x, p2.y, 1.2, Color::new(1.0, 1.0, 1.0, a));
            prev = p2;
        }
    }
    for (c, dir, r, full, age) in &fx.slashes {
        let a = 1.0 - age / 0.2;
        let base = dir.y.atan2(dir.x);
        let (span, segs) = if *full { (std::f32::consts::PI, 24) } else { (60f32.to_radians(), 10) };
        for k in 0..segs {
            let t0 = base - span + 2.0 * span * k as f32 / segs as f32;
            let t1 = base - span + 2.0 * span * (k + 1) as f32 / segs as f32;
            let rr = r * (0.75 + 0.25 * (age / 0.2));
            draw_line(c.x + t0.cos() * rr, c.y + t0.sin() * rr, c.x + t1.cos() * rr, c.y + t1.sin() * rr, 5.0, Color::new(0.85, 0.95, 1.0, 0.8 * a));
        }
    }

    let gold = world.weapons.iter().any(|w| w.kind == WeaponKind::Ofuda && w.awakened);
    let ink = if gold { Color::new(1.0, 0.85, 0.35, 1.0) } else { WHITE };
    let pr = &world.projectiles;
    for i in 0..pr.len() {
        let (q, v) = (pr.pos[i], pr.vel[i]);
        draw_texture_ex(
            &a.talisman,
            q.x - 3.0,
            q.y - 5.0,
            ink,
            DrawTextureParams { rotation: v.y.atan2(v.x) + std::f32::consts::FRAC_PI_2, ..Default::default() },
        );
    }

    for (c, age) in &fx.rings {
        let t = age / 0.45;
        let r = sim::content::BARRIER_RADIUS * (0.2 + 0.8 * t.sqrt());
        draw_circle(c.x, c.y, r, Color::new(1.0, 0.95, 0.85, 0.18 * (1.0 - t)));
        draw_circle_lines(c.x, c.y, r, 3.0, Color::new(1.0, 0.9, 0.6, 0.9 * (1.0 - t)));
    }

    for pt in &fx.particles {
        let alpha = 1.0 - pt.age / pt.life;
        let mut c = pt.color;
        c.a = alpha;
        draw_rectangle(pt.pos.x - 1.0, pt.pos.y - 1.0, 2.0, 2.0, c);
    }

    // Damage numbers are drawn in screen space so the text stays crisp.
    set_default_camera();
    for n in &fx.numbers {
        let s = cam.world_to_screen(n.pos);
        let alpha = 1.0 - (n.age / 0.6).powi(2);
        let txt = format!("{:.0}", n.amount);
        draw_text(&txt, s.x - 5.0 + 1.0, s.y + 1.0, 18.0, Color::new(0.0, 0.0, 0.0, alpha));
        draw_text(&txt, s.x - 5.0, s.y, 18.0, Color::new(1.0, 1.0, 1.0, alpha));
    }
    if let Some((line, left)) = &fx.bark {
        let s = cam.world_to_screen(vec2(p.x, p.y - 34.0));
        let d = measure_text(line, None, 20, 1.0);
        let alpha = left.min(0.4) / 0.4;
        draw_rectangle(s.x - d.width / 2.0 - 8.0, s.y - 20.0, d.width + 16.0, 28.0, Color::new(0.05, 0.03, 0.1, 0.8 * alpha));
        draw_text(line, s.x - d.width / 2.0, s.y, 20.0, Color::new(1.0, 0.9, 0.95, alpha));
    }
}
