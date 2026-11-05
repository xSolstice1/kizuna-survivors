//! Draws the world: ground, pickups, enemies, player, weapons, effects.

use macroquad::prelude::*;
use sim::content::{EnemyKind, WeaponKind};
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
        }
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
