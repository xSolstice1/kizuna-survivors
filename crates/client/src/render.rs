//! Draws the world: ground, pickups, enemies, player, weapons, effects.

use macroquad::prelude::*;
use sim::content::{EnemyKind, HeroId, PassiveKind, SpecialKind, Upgrade, WeaponKind, ALL_HEROES};
use sim::world::{style, ProjKind};
use sim::{Event, World};

use crate::sprites;
use crate::voice;

/// World pixels → screen pixels (integer for crisp pixel art).
pub const SCALE: f32 = 2.0;

pub struct Assets {
    /// Per hero (in `ALL_HEROES` order): idle, walk A, walk B.
    heroes: Vec<[Texture2D; 3]>,
    /// Riku in Ninefold Foxfire form (idle, walk A, walk B).
    riku_fox: [Texture2D; 3],
    /// Shun in Hannya Unbound form, and his blade in both forms.
    shun_hannya: [Texture2D; 3],
    cleaver: Texture2D,
    cleaver_hannya: Texture2D,
    familiar: Texture2D,
    shuriken: Texture2D,
    pub coin: Texture2D,
    pub wisp: Texture2D,
    pub kasa: Texture2D,
    pub oni: Texture2D,
    pub chochin: Texture2D,
    pub nopperabo: Texture2D,
    pub night_lord: Texture2D,
    pub talisman: Texture2D,
    pub orb: Texture2D,
    pub gem: Texture2D,
    pub chest: Texture2D,
    pub chest_open: Texture2D,
    pub shrine: Texture2D,
    icons: Vec<(&'static str, Texture2D)>,
}

impl Assets {
    pub fn load() -> Self {
        let base = include_str!("../../../art/src/heroine.px");
        // Legs occupy rows 27..32; left leg columns 5..10, right leg 14..19 (Riku matches this frame).
        let frames = |hero: sprites::Sprite| {
            let walk_a = hero.lift(5..10, 27..32);
            let walk_b = hero.lift(14..19, 27..32);
            [hero.texture(), walk_a.texture(), walk_b.texture()]
        };
        let heroes = ALL_HEROES
            .iter()
            .map(|h| match h {
                HeroId::Riku => frames(sprites::parse(include_str!("../../../art/src/riku.px"))),
                HeroId::Shun => frames(sprites::parse(include_str!("../../../art/src/shun.px"))),
                _ => frames(sprites::parse(&sprites::recolor(base, hero_palette(*h)))),
            })
            .collect();
        Assets {
            heroes,
            riku_fox: frames(sprites::parse(include_str!("../../../art/src/riku_fox.px"))),
            shun_hannya: frames(sprites::parse(include_str!("../../../art/src/shun_hannya.px"))),
            cleaver: sprites::texture(include_str!("../../../art/src/cleaver.px")),
            cleaver_hannya: sprites::texture(include_str!("../../../art/src/cleaver_hannya.px")),
            familiar: sprites::texture(include_str!("../../../art/src/familiar.px")),
            shuriken: sprites::texture(include_str!("../../../art/src/shuriken.px")),
            coin: sprites::texture(include_str!("../../../art/src/coin.px")),
            wisp: sprites::texture(include_str!("../../../art/src/wisp.px")),
            kasa: sprites::texture(include_str!("../../../art/src/kasa.px")),
            oni: sprites::texture(include_str!("../../../art/src/oni.px")),
            chochin: sprites::texture(include_str!("../../../art/src/chochin.px")),
            nopperabo: sprites::texture(include_str!("../../../art/src/nopperabo.px")),
            night_lord: sprites::texture(include_str!("../../../art/src/night_lord.px")),
            talisman: sprites::texture(include_str!("../../../art/src/talisman.px")),
            orb: sprites::texture(include_str!("../../../art/src/orb.px")),
            gem: sprites::texture(include_str!("../../../art/src/gem.px")),
            chest: sprites::texture(include_str!("../../../art/src/chest.px")),
            chest_open: sprites::texture(include_str!("../../../art/src/chest_open.px")),
            shrine: sprites::texture(include_str!("../../../art/src/shrine.px")),
            icons: vec![
            ("abundance", sprites::texture(include_str!("../../../art/src/icons/abundance.px"))),
            ("crescent", sprites::texture(include_str!("../../../art/src/icons/crescent.px"))),
            ("expanse", sprites::texture(include_str!("../../../art/src/icons/expanse.px"))),
            ("flame_pool", sprites::texture(include_str!("../../../art/src/icons/flame_pool.px"))),
            ("foxfire_orb", sprites::texture(include_str!("../../../art/src/icons/foxfire_orb.px"))),
            ("familiars", sprites::texture(include_str!("../../../art/src/icons/familiars.px"))),
            ("gale_shuriken", sprites::texture(include_str!("../../../art/src/icons/gale_shuriken.px"))),
            ("fox_mirage", sprites::texture(include_str!("../../../art/src/icons/fox_mirage.px"))),
            ("grave_cleaver", sprites::texture(include_str!("../../../art/src/icons/grave_cleaver.px"))),
            ("soul_tether", sprites::texture(include_str!("../../../art/src/icons/soul_tether.px"))),
            ("lantern_vigil", sprites::texture(include_str!("../../../art/src/icons/lantern_vigil.px"))),
            ("iai_flash", sprites::texture(include_str!("../../../art/src/icons/iai_flash.px"))),
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

    pub fn hero(&self, h: HeroId) -> &[Texture2D; 3] {
        &self.heroes[ALL_HEROES.iter().position(|x| *x == h).unwrap()]
    }

    pub fn icon_named(&self, key: &str) -> &Texture2D {
        &self.icons.iter().find(|(n, _)| *n == key).expect("icon").1
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
                WeaponKind::FoxfireOrb => "foxfire_orb",
                WeaponKind::Familiars => "familiars",
                WeaponKind::GaleShuriken => "gale_shuriken",
                WeaponKind::FoxMirage => "fox_mirage",
                WeaponKind::GraveCleaver => "grave_cleaver",
                WeaponKind::SoulTether => "soul_tether",
                WeaponKind::LanternVigil => "lantern_vigil",
                WeaponKind::IaiFlash => "iai_flash",
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

/// Placeholder looks: Ren's sprite with each hero's own hair / outfit / eye / ribbon colours.
/// Palette chars: H/h/L hair, C/c outfit, I eyes, R ribbon.
fn hero_palette(h: HeroId) -> &'static [(char, &'static str)] {
    match h {
        HeroId::Ren | HeroId::Riku | HeroId::Shun => &[],
        HeroId::Kaede => &[('H', "#33334a"), ('h', "#1c1c2a"), ('L', "#5a5a7a"), ('C', "#2f5a3a"), ('c', "#1d3a25"), ('I', "#e84a4a"), ('R', "#c9a040")],
        HeroId::Mika => &[('H', "#ffd45a"), ('h', "#d49a2a"), ('L', "#fff0a0"), ('C', "#e88ac8"), ('c', "#b0609a"), ('I', "#4ac0ff"), ('R', "#ff5aa0")],
        HeroId::Haru => &[('H', "#c8cde0"), ('h', "#8890aa"), ('L', "#ffffff"), ('C', "#3a3a8a"), ('c', "#24245a"), ('I', "#6a7cff"), ('R', "#e83a3a")],
        HeroId::Nanami => &[('H', "#3ac0a8"), ('h', "#1f8070"), ('L', "#9af0e0"), ('C', "#8a5a3a"), ('c', "#5a3a24"), ('I', "#a06aff"), ('R', "#ffb040")],
        HeroId::Yuzu => &[('H', "#ff9a3a"), ('h', "#c86420"), ('L', "#ffd0a0"), ('C', "#b05ad8"), ('c', "#7a3aa0"), ('I', "#ff6aa0"), ('R', "#4ad0ff")],
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
    /// Bursts: (pos, radius, age, style). Style: 0 foxfire blue, 1 gold blast, 2 crimson, 3 finale.
    bursts: Vec<(Vec2, f32, f32, u8)>,
    /// Beams: (from, to, age, style).
    beams: Vec<(Vec2, Vec2, f32, u8)>,
    /// Signature flourishes under the player: age.
    sig_rings: Vec<f32>,
    /// Iai Flash cuts: (from, to, age).
    iai: Vec<(Vec2, Vec2, f32)>,
    /// Lantern pulse rings: (pos, radius, age).
    pulses: Vec<(Vec2, f32, f32)>,
    /// Shadow Step streaks: (from, to, age).
    steps: Vec<(Vec2, Vec2, f32)>,
    /// Sword swing: seconds left and direction.
    pub swing: (f32, Vec2),

    /// Smoke puffs from Fox Mirage clones appearing / popping: (pos, radius, age).
    smokes: Vec<(Vec2, f32, f32)>,
    pub shake: f32,
    pub show_numbers: bool,
    /// Short line spoken by the hero, shown above her head.
    pub bark: Option<(String, f32)>,
    /// Big centre-screen announcement.
    pub banner: Option<(String, f32)>,
    leveled_once: bool,
    pub low_hp_warned: bool,
    hero: Option<HeroId>,
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
        self.bursts.clear();
        self.smokes.clear();
        self.beams.clear();
        self.sig_rings.clear();
        self.steps.clear();
        self.iai.clear();
        self.pulses.clear();
        self.shake = 0.0;
        self.banner = None;
        self.leveled_once = false;
        self.low_hp_warned = false;
        self.say(self.lines().start);
    }

    pub fn set_hero(&mut self, h: HeroId) {
        self.hero = Some(h);
    }

    pub fn lines(&self) -> voice::Lines {
        voice::lines(self.hero.unwrap_or(HeroId::Ren))
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
                        EnemyKind::Chochin => (8, Color::from_rgba(255, 150, 70, 255)),
                        EnemyKind::Nopperabo => (8, Color::from_rgba(200, 180, 230, 255)),
                        EnemyKind::NightLord => (200, Color::from_rgba(255, 220, 120, 255)),
                    };
                    if kind == EnemyKind::NightLord {
                        self.shake = self.shake.max(0.6);
                        self.say(self.lines().night_lord_down);
                    }
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
                    self.say(self.lines().first_level);
                }
                Event::SpecialUsed { pos } => {
                    self.rings.push((vec2(pos.x, pos.y), 0.0));
                    self.shake = self.shake.max(0.25);
                    self.say(self.lines().special);
                }
                Event::BossSpawned => {
                    self.announce("A GREAT ONI APPROACHES".into());
                    self.say(self.lines().boss);
                }
                Event::NightLordSpawned => {
                    self.announce("THE NIGHT LORD RISES".into());
                    self.say(self.lines().night_lord);
                    self.shake = self.shake.max(0.4);
                }
                Event::Blink { pos } => {
                    for k in 0..10 {
                        if self.particles.len() >= MAX_PARTICLES {
                            break;
                        }
                        let a = k as f32 / 10.0 * std::f32::consts::TAU;
                        self.particles.push(Particle {
                            pos: vec2(pos.x, pos.y - 8.0),
                            vel: vec2(a.cos(), a.sin()) * 40.0,
                            age: 0.0,
                            life: 0.35,
                            color: Color::from_rgba(190, 170, 230, 255),
                        });
                    }
                }
                Event::BossTelegraph => self.shake = self.shake.max(0.08),
                Event::ShrineAppeared => {
                    self.announce("A ROADSIDE SHRINE APPEARS".into());
                    self.say(self.lines().shrine);
                }
                Event::ShrineChosen(ev) => self.say(&format!("{}.", ev.name())),
                Event::Revived => {
                    self.announce("REVIVAL".into());
                    self.say(self.lines().revived);
                    self.shake = self.shake.max(0.3);
                }
                Event::Bolt { from, to } => self.bolts.push((vec2(from.x, from.y), vec2(to.x, to.y), 0.0)),
                Event::Slash { pos, dir, radius, full } => {
                    self.slashes.push((vec2(pos.x, pos.y), vec2(dir.x, dir.y), radius, full, 0.0));
                    if self.swing.0 > 0.0 {
                        self.swing.1 = vec2(dir.x, dir.y);
                    }
                }
                Event::Impact { pos, radius } => {
                    self.impacts.push((vec2(pos.x, pos.y), radius, 0.0));
                    self.shake = self.shake.max(0.04);
                }
                Event::Burst { pos, radius, style } => {
                    self.bursts.push((vec2(pos.x, pos.y), radius, 0.0, style));
                    match style {
                        style::METEOR => self.shake = self.shake.max(0.3),
                        style::QUAKE => self.shake = self.shake.max(0.2),
                        style::SKYBOLT => self.shake = self.shake.max(0.06),
                        _ => {}
                    }
                }
                Event::Signature(_) => self.sig_rings.push(0.0),
                Event::HannyaAwakened { pos } => {
                    self.bursts.push((vec2(pos.x, pos.y), sim::content::HANNYA_ONSET_RADIUS, 0.0, 2));
                    self.announce("HANNYA UNBOUND".into());
                    self.say(self.lines().special);
                    self.shake = self.shake.max(0.5);
                }
                Event::Beam { from, to, style } => self.beams.push((vec2(from.x, from.y), vec2(to.x, to.y), 0.0, style)),
                Event::Iai { from, to } => {
                    self.iai.push((vec2(from.x, from.y), vec2(to.x, to.y), 0.0));
                    self.shake = self.shake.max(0.05);
                }
                Event::LanternPulse { pos, radius } => self.pulses.push((vec2(pos.x, pos.y), radius, 0.0)),
                Event::ShadowStep { from, to } => self.steps.push((vec2(from.x, from.y), vec2(to.x, to.y), 0.0)),
                Event::AfterimageBlast { pos, radius } => {
                    self.bursts.push((vec2(pos.x, pos.y), radius, 0.0, 2));
                    self.shake = self.shake.max(0.08);
                }
                Event::FinaleCharge { .. } => self.say("This ends now!"),
                Event::FinaleBlast { pos, radius } => {
                    self.bursts.push((vec2(pos.x, pos.y), radius, 0.0, 3));
                    self.announce("HANNYA'S WRATH".into());
                    self.shake = self.shake.max(0.8);
                }
                Event::Fired(WeaponKind::GraveCleaver) => self.swing = (0.2, Vec2::ZERO),
                Event::FoxAwakened { pos } => {
                    self.bursts.push((vec2(pos.x, pos.y), sim::content::FOX_BLAST_RADIUS, 0.0, 1));
                    self.announce("NINEFOLD FOXFIRE".into());
                    self.say(self.lines().special);
                    self.shake = self.shake.max(0.5);
                }
                Event::FoxFaded => self.say("Phew... the foxfire's cooling."),
                Event::CloneSummoned { pos } => self.smokes.push((vec2(pos.x, pos.y), 16.0, 0.0)),
                Event::ClonePopped { pos, radius } => self.smokes.push((vec2(pos.x, pos.y), radius, 0.0)),
                Event::Awakened(k) => {
                    self.announce(format!("AWAKENING: {}", sim::content::awakened_name(k)));
                    self.say(self.lines().awaken);
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
        self.bursts.iter_mut().for_each(|b| b.2 += dt);
        self.bursts.retain(|b| b.2 < burst_life(b.3));
        self.beams.iter_mut().for_each(|b| b.2 += dt);
        self.beams.retain(|b| b.2 < if b.3 == style::MOON { 0.45 } else { 0.3 });
        self.sig_rings.iter_mut().for_each(|r| *r += dt);
        self.sig_rings.retain(|r| *r < 0.5);
        self.iai.iter_mut().for_each(|b| b.2 += dt);
        self.iai.retain(|b| b.2 < 0.25);
        self.pulses.iter_mut().for_each(|b| b.2 += dt);
        self.pulses.retain(|b| b.2 < 0.4);
        self.steps.iter_mut().for_each(|b| b.2 += dt);
        self.steps.retain(|b| b.2 < 0.3);
        self.swing.0 = (self.swing.0 - dt).max(0.0);

        self.smokes.iter_mut().for_each(|s| s.2 += dt);
        self.smokes.retain(|s| s.2 < 0.5);
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

pub fn draw_world(world: &World, a: &Assets, stage: &crate::stage::StageArt, fx: &Fx, cam: &Camera2D, time: f64) {
    set_camera(cam);
    stage.draw_ground(cam, time);
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
    // Gems: tier by value (merged gems are worth more and look it).
    for (g, &v) in world.gems.pos.iter().zip(&world.gems.value) {
        let (size, tint) = match v {
            0..=4 => (7.0, WHITE),
            5..=24 => (9.0, Color::new(0.45, 1.0, 0.45, 1.0)),
            25..=99 => (11.0, Color::new(1.0, 0.4, 0.45, 1.0)),
            _ => (14.0, Color::new(1.0, 0.85, 0.3, 1.0)),
        };
        if v >= 25 {
            let pulse = 0.2 + 0.1 * (time * 4.0 + g.x as f64).sin() as f32;
            draw_circle(g.x, g.y, size * 0.9, Color::new(tint.r, tint.g, tint.b, pulse));
        }
        draw_texture_ex(
            &a.gem,
            (g.x - size / 2.0).round(),
            (g.y - size / 2.0).round(),
            tint,
            DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() },
        );
    }
    if let Some(s) = world.shrine {
        let pulse = 0.5 + 0.5 * (time * 3.0).sin() as f32;
        draw_rectangle(s.x - 5.0, s.y - 120.0, 10.0, 120.0, Color::new(1.0, 0.9, 0.6, 0.10 + 0.06 * pulse));
        draw_circle(s.x, s.y, 22.0 + 3.0 * pulse, Color::new(1.0, 0.85, 0.5, 0.16 + 0.08 * pulse));
        draw_texture_ex(&a.shrine, (s.x - 16.0).round(), (s.y - 30.0).round(), WHITE, DrawTextureParams { dest_size: Some(vec2(32.0, 36.0)), ..Default::default() });
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
    for l in &world.lanterns {
        let fade = (l.life / 0.4).min(1.0) * ((l.max_life - l.life) / 0.3).min(1.0);
        let bob = ((time * 3.0 + l.pos.x as f64).sin() * 2.0) as f32;
        draw_circle(l.pos.x, l.pos.y, l.radius, Color::new(0.5, 0.75, 1.0, 0.06 * fade));
        draw_circle_lines(l.pos.x, l.pos.y, l.radius, 1.0, Color::new(0.7, 0.85, 1.0, 0.35 * fade));
        let (x, y) = (l.pos.x, l.pos.y - 12.0 + bob);
        draw_circle(x, y, 10.0, Color::new(0.55, 0.8, 1.0, 0.18 * fade));
        draw_rectangle(x - 2.0, y - 8.0, 4.0, 2.0, Color::new(0.16, 0.14, 0.2, fade));
        draw_ellipse(x, y, 5.0, 6.0, 0.0, Color::new(0.95, 0.92, 0.86, fade));
        draw_circle(x, y, 2.5, Color::new(0.55, 0.82, 1.0, fade));
        draw_rectangle(x - 2.0, y + 6.0, 4.0, 2.0, Color::new(0.16, 0.14, 0.2, fade));
    }
    for (c, v) in &world.coins {
        let size = if *v >= 50 { 16.0 } else if *v >= 5 { 12.0 } else { 10.0 };
        let bob = ((time * 4.0 + c.x as f64).sin() * 1.0) as f32;
        draw_circle(c.x, c.y, size * 0.7, Color::new(1.0, 0.85, 0.3, 0.15));
        draw_texture_ex(&a.coin, (c.x - size / 2.0).round(), (c.y - size / 2.0 + bob).round(), WHITE, DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() });
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
        if e.kind[i] == EnemyKind::NightLord {
            // Big, bobbing lantern with a warm glow; drawn at 2x (96px).
            let bob = ((time * 2.0).sin() * 3.0) as f32;
            let glow = 0.18 + 0.06 * (time * 5.0).sin() as f32;
            draw_circle(pos.x, pos.y - 24.0, 60.0, Color::new(1.0, 0.55, 0.2, glow));
            draw_ellipse(pos.x, pos.y + 8.0, 34.0, 9.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.4));
            draw_texture_ex(
                &a.night_lord,
                (pos.x - 48.0).round(),
                (pos.y - 84.0 + bob).round(),
                tint,
                DrawTextureParams { dest_size: Some(vec2(96.0, 96.0)), ..Default::default() },
            );
            continue;
        }
        let (tex, half, bottom) = match e.kind[i] {
            EnemyKind::Wisp => (&a.wisp, 8.0, 8.0),
            EnemyKind::Kasa => (&a.kasa, 8.0, 12.0),
            EnemyKind::Chochin => (&a.chochin, 8.0, 14.0),
            EnemyKind::Nopperabo => (&a.nopperabo, 8.0, 20.0),
            _ => (&a.oni, 16.0, 26.0),
        };
        let mut tint = tint;
        if e.kind[i] == EnemyKind::Chochin {
            draw_circle(pos.x, pos.y - 4.0, 11.0, Color::new(1.0, 0.6, 0.25, 0.14));
        } else if e.kind[i] == EnemyKind::Nopperabo && e.ai_state[i] != 0 {
            // Fading out before a blink (state 1), fading back in after it (state 2).
            let t = if e.ai_state[i] == 1 { e.ai_timer[i] / 0.5 } else { 1.0 - e.ai_timer[i] / sim::content::NOPPERABO_FADE_IN };
            tint.a = t.clamp(0.1, 1.0) * 0.85;
        }
        draw_texture_ex(&tex, (pos.x - half).round(), (pos.y - bottom).round(), tint, DrawTextureParams { flip_x: flip, ..Default::default() });
    }

    // Player: walk cycle while moving, blink during i-frames.
    let pl = &world.player;
    let frame = if pl.moving { 1 + ((time * 8.0) as usize % 2) } else { 0 };
    let form = world.form_kind();
    let fox = form == Some(SpecialKind::NineTails);
    let hannya = form == Some(SpecialKind::Hannya);
    let blink = form.is_none() && pl.iframes > 0.0 && ((time * 20.0) as i32) % 2 == 0;
    let tint = if blink { Color::new(1.0, 1.0, 1.0, 0.35) } else { WHITE };
    let t = time as f32;
    if fox {
        draw_fox_aura_and_tails(world, t);
    }
    if hannya {
        draw_hannya_aura(world, t);
    }
    // Shadow Step afterimages (explode shortly) and streaks.
    for (c, left) in &world.afterimages {
        let k = left / 0.35;
        draw_circle(c.x, c.y - 10.0, 18.0 * (1.5 - k), Color::new(0.6, 0.05, 0.1, 0.25 * (1.0 - k) + 0.1));
        draw_texture_ex(&a.shun_hannya[0], (c.x - 12.0).round(), (c.y - 26.0).round(), Color::new(0.9, 0.15, 0.2, 0.55 * k + 0.2), DrawTextureParams::default());
    }
    for (from, to, age) in &fx.steps {
        let k = 1.0 - age / 0.3;
        draw_line(from.x, from.y - 10.0, to.x, to.y - 10.0, 6.0 * k, Color::new(0.5, 0.0, 0.08, 0.6 * k));
        draw_line(from.x, from.y - 10.0, to.x, to.y - 10.0, 2.0 * k, Color::new(1.0, 0.4, 0.4, 0.8 * k));
    }
    // Fox Mirage clones: translucent, blue-shifted copies of Riku (gold while he's transformed).
    for c in &world.clones {
        let f = if c.moving { 1 + ((time * 9.0 + c.pos.x as f64 * 0.1) as usize % 2) } else { 0 };
        let ctex = if fox { &a.riku_fox[f] } else { &a.hero(world.cfg.hero)[f] };
        let fade = (c.life / 0.4).min(1.0);
        let flicker = 0.7 + 0.15 * (t * 13.0 + c.pos.y).sin();
        let tint = Color::new(0.75, 0.85, 1.0, flicker * fade);
        let (cx, cy) = ((c.pos.x - 12.0).round(), (c.pos.y - 26.0).round());
        draw_ellipse(c.pos.x, c.pos.y + 5.0, 8.0, 2.5, 0.0, Color::new(0.0, 0.0, 0.0, 0.2 * fade));
        // Afterimage trailing behind when moving.
        if c.moving {
            let back = c.facing * -5.0;
            draw_texture_ex(ctex, cx + back.x, cy + back.y, Color::new(0.5, 0.65, 1.0, 0.25 * fade), DrawTextureParams { flip_x: c.facing.x < 0.0, ..Default::default() });
        }
        draw_texture_ex(ctex, cx, cy, tint, DrawTextureParams { flip_x: c.facing.x < 0.0, ..Default::default() });
        if c.striking > 0.0 {
            let base = c.facing.y.atan2(c.facing.x);
            let k = c.striking / 0.15;
            for s in 0..6 {
                let a0 = base - 0.9 + s as f32 * 0.3;
                let a1 = a0 + 0.3;
                let r = 16.0;
                draw_line(c.pos.x + a0.cos() * r, c.pos.y - 8.0 + a0.sin() * r, c.pos.x + a1.cos() * r, c.pos.y - 8.0 + a1.sin() * r, 3.0, Color::new(0.85, 0.95, 1.0, 0.9 * k));
            }
        }
    }
    let awakened = world.weapons.iter().filter(|w| w.awakened).count();
    if awakened > 0 {
        // A golden rune circle that grows richer with every Awakening.
        let r = 16.0 + 3.0 * awakened as f32;
        draw_circle_lines(p.x, p.y + 3.0, r, 1.5, Color::new(1.0, 0.85, 0.4, 0.55));
        for k in 0..(6 + 2 * awakened) {
            let a = t * 1.2 + k as f32 * std::f32::consts::TAU / (6 + 2 * awakened) as f32;
            draw_rectangle(p.x + a.cos() * r - 1.0, p.y + 3.0 + a.sin() * r * 0.45 - 1.0, 2.0, 2.0, Color::new(1.0, 0.9, 0.5, 0.9));
        }
    }
    for age in &fx.sig_rings {
        let k = age / 0.5;
        draw_circle_lines(p.x, p.y + 3.0, 20.0 + 60.0 * k, 3.0 * (1.0 - k) + 1.0, Color::new(1.0, 0.9, 0.5, 0.8 * (1.0 - k)));
    }
    draw_ellipse(p.x, p.y + 5.0, 9.0, 3.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.35));
    let tex = if fox {
        &a.riku_fox[frame]
    } else if hannya {
        &a.shun_hannya[frame]
    } else {
        &a.hero(world.cfg.hero)[frame]
    };
    let (x, y) = ((p.x - 12.0).round(), (p.y - 26.0).round());
    let params = DrawTextureParams { flip_x: pl.facing.x < 0.0, ..Default::default() };
    if fox || hannya {
        // Flaring outline: the sprite tinted (gold foxfire / crimson-black dread), offset around.
        let flare = 0.45 + 0.25 * (t * 9.0).sin();
        let glow = if fox { Color::new(1.0, 0.75, 0.2, flare) } else { Color::new(0.75, 0.0, 0.1, flare + 0.2) };
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
            draw_texture_ex(tex, x + dx, y + dy, glow, params.clone());
        }
    }
    let shun = world.cfg.hero == HeroId::Shun;
    let flip = pl.facing.x < 0.0;
    // Shun's blade sits behind him at rest; mid-swing it's drawn in front.
    let swinging = fx.swing.0 > 0.0;
    if shun && !swinging {
        draw_blade(a, p, flip, hannya, None);
    }
    draw_texture_ex(tex, x, y, tint, params);
    if shun && swinging {
        draw_blade(a, p, flip, hannya, Some(fx.swing));
    }
    match world.cfg.hero {
        HeroId::Riku => draw_spirit_motes(p, t, fox),
        HeroId::Shun => draw_reaper_wisps(p, t, hannya),
        _ => {}
    }
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
    let t = time as f32;
    for i in 0..pr.len() {
        let (q, v) = (pr.pos[i], pr.vel[i]);
        match pr.kind[i] {
            ProjKind::Talisman => draw_texture_ex(
                &a.talisman,
                q.x - 3.0,
                q.y - 5.0,
                ink,
                DrawTextureParams { rotation: v.y.atan2(v.x) + std::f32::consts::FRAC_PI_2, ..Default::default() },
            ),
            ProjKind::Foxfire | ProjKind::Comet => {
                let comet = pr.kind[i] == ProjKind::Comet;
                // While charging the orb swells and sparks spiral into it.
                let grow = if pr.charge[i] > 0.0 { 1.0 - (pr.charge[i] / 0.55).min(1.0) } else { 1.0 };
                let r = 3.0 + 5.0 * grow;
                if pr.charge[i] > 0.0 {
                    for k in 0..5 {
                        let a = t * 9.0 + k as f32 * 1.256;
                        let d = 14.0 * (1.0 - ((t * 2.5 + k as f32 * 0.2) % 1.0));
                        draw_rectangle(q.x + a.cos() * d - 1.0, q.y + a.sin() * d - 1.0, 2.0, 2.0, Color::new(0.75, 0.95, 1.0, 0.9));
                    }
                } else {
                    let back = v.normalize_or_zero() * -1.0;
                    for k in 1..4 {
                        let c = q + back * (k as f32 * 4.0);
                        draw_circle(c.x, c.y, r * (1.0 - k as f32 * 0.22), Color::new(0.25, 0.6, 1.0, 0.35 / k as f32));
                    }
                }
                let (outer, mid) = if comet {
                    (Color::new(0.55, 0.45, 1.0, 0.35), Color::new(0.5, 0.75, 1.0, 1.0))
                } else {
                    (Color::new(0.2, 0.55, 1.0, 0.3), Color::new(0.25, 0.65, 1.0, 1.0))
                };
                draw_circle(q.x, q.y, r * 1.8, outer);
                draw_circle(q.x, q.y, r, mid);
                draw_circle(q.x, q.y, r * 0.55, Color::new(0.85, 0.97, 1.0, 1.0));
                // Swirl: two bright arcs rotating around the core.
                for k in 0..2 {
                    let a = t * 14.0 + k as f32 * std::f32::consts::PI;
                    draw_circle(q.x + a.cos() * r * 0.7, q.y + a.sin() * r * 0.7, r * 0.22, WHITE);
                }
            }
            ProjKind::Spark => {
                draw_circle(q.x, q.y, 5.0, Color::new(0.4, 0.7, 1.0, 0.35));
                draw_circle(q.x, q.y, 2.5, Color::new(0.85, 0.97, 1.0, 1.0));
            }
            ProjKind::GoldenOrb => {
                let r = 13.0;
                draw_circle(q.x, q.y, r * 1.7, Color::new(1.0, 0.55, 0.1, 0.22));
                draw_circle(q.x, q.y, r, Color::new(1.0, 0.65, 0.15, 0.95));
                draw_circle(q.x, q.y, r * 0.65, Color::new(1.0, 0.88, 0.4, 1.0));
                draw_circle(q.x, q.y, r * 0.3, WHITE);
                for k in 0..4 {
                    let a = t * 16.0 + k as f32 * std::f32::consts::FRAC_PI_2;
                    draw_line(q.x + a.cos() * r * 0.3, q.y + a.sin() * r * 0.3, q.x + (a + 0.9).cos() * r * 0.95, q.y + (a + 0.9).sin() * r * 0.95, 2.0, Color::new(1.0, 1.0, 0.85, 0.9));
                }
            }
            ProjKind::Familiar => {
                let back = v.normalize_or_zero() * -1.0;
                for k in 1..4 {
                    let c = q + back * (k as f32 * 3.5);
                    draw_rectangle(c.x - 1.0, c.y - 1.0, 2.0, 2.0, Color::new(1.0, 0.75, 0.3, 0.8 - k as f32 * 0.2));
                }
                draw_texture_ex(&a.familiar, (q.x - 6.0).round(), (q.y - 5.0).round(), WHITE, DrawTextureParams { flip_x: v.x < 0.0, ..Default::default() });
            }
            ProjKind::CrescentWave => {
                let hannya = world.form_kind() == Some(SpecialKind::Hannya);
                let r = pr.radius[i] * 1.6;
                let base = v.y.atan2(v.x);
                let life = pr.life[i] / pr.max_life[i];
                let (outer, inner) = if hannya {
                    (Color::new(0.55, 0.0, 0.08, 0.8 * life + 0.2), Color::new(1.0, 0.35, 0.35, life))
                } else {
                    (Color::new(0.45, 0.35, 0.9, 0.7 * life + 0.2), Color::new(0.9, 0.9, 1.0, life))
                };
                for s in 0..10 {
                    let a0 = base - 1.2 + s as f32 * 0.24;
                    let a1 = a0 + 0.24;
                    let thick = 6.0 * (1.0 - ((s as f32 - 4.5).abs() / 5.0));
                    draw_line(q.x + a0.cos() * r, q.y + a0.sin() * r, q.x + a1.cos() * r, q.y + a1.sin() * r, thick + 2.0, outer);
                    draw_line(q.x + a0.cos() * r, q.y + a0.sin() * r, q.x + a1.cos() * r, q.y + a1.sin() * r, thick * 0.5, inner);
                }
            }
            ProjKind::Seal => {
                let back = v.normalize_or_zero() * -1.0;
                for k in 1..5 {
                    let c = q + back * (k as f32 * 4.0);
                    draw_circle(c.x, c.y, 4.0 - k as f32 * 0.7, Color::new(1.0, 0.8, 0.3, 0.5 - k as f32 * 0.1));
                }
                draw_circle(q.x, q.y, 8.0, Color::new(1.0, 0.85, 0.35, 0.25));
                draw_texture_ex(&a.talisman, q.x - 3.0, q.y - 5.0, Color::new(1.0, 0.85, 0.35, 1.0), DrawTextureParams { rotation: v.y.atan2(v.x) + std::f32::consts::FRAC_PI_2, ..Default::default() });
            }
            ProjKind::Petal => {
                let a = t * 10.0 + q.x * 0.1;
                draw_circle(q.x, q.y, 5.0, Color::new(1.0, 0.6, 0.8, 0.25));
                draw_ellipse(q.x, q.y, 3.5, 2.0, a.to_degrees(), Color::new(1.0, 0.72, 0.85, 1.0));
                draw_ellipse(q.x, q.y, 1.5, 1.0, a.to_degrees(), Color::new(1.0, 0.95, 0.98, 1.0));
            }
            ProjKind::SpiritBolt => {
                let back = v.normalize_or_zero() * -1.0;
                for k in 1..4 {
                    let c = q + back * (k as f32 * 4.0);
                    draw_circle(c.x, c.y, 3.5 - k as f32 * 0.8, Color::new(0.6, 0.8, 1.0, 0.5 - k as f32 * 0.12));
                }
                draw_circle(q.x, q.y, 6.0, Color::new(0.55, 0.8, 1.0, 0.3));
                draw_circle(q.x, q.y, 3.0, Color::new(0.9, 0.97, 1.0, 1.0));
            }
            ProjKind::Shuriken => {
                let size = pr.radius[i] * 2.2;
                draw_circle(q.x, q.y, size * 0.55, Color::new(0.8, 0.9, 1.0, 0.12));
                draw_texture_ex(
                    &a.shuriken,
                    q.x - size / 2.0,
                    q.y - size / 2.0,
                    WHITE,
                    DrawTextureParams { dest_size: Some(vec2(size, size)), rotation: t * 22.0, ..Default::default() },
                );
            }
        }
    }
    for (c, r, age) in &fx.smokes {
        let k = age / 0.5;
        for i in 0..7 {
            let a = i as f32 * 0.9 + c.x * 0.05;
            let d = r * 0.5 * k.sqrt();
            let pr = r * (0.35 + 0.35 * k) * (0.8 + 0.2 * ((i * 37 % 10) as f32 / 10.0));
            draw_circle(c.x + a.cos() * d, c.y - 6.0 + a.sin() * d * 0.7 - k * 6.0, pr, Color::new(0.85, 0.88, 0.95, 0.45 * (1.0 - k)));
        }
    }
    for t in &world.tethers {
        if let Some(j) = world.enemies.id.iter().position(|id| *id == t.enemy) {
            let e = world.enemies.pos[j];
            let from = vec2(p.x, p.y - 8.0);
            let to = vec2(e.x, e.y - 4.0);
            let d = to - from;
            let links = (d.length() / 7.0).max(2.0) as usize;
            let sag = vec2(-d.y, d.x).normalize_or_zero() * (6.0 * (time as f32 * 4.0 + e.x * 0.05).sin());
            for k in 0..=links {
                let u = k as f32 / links as f32;
                let c = from + d * u + sag * (4.0 * u * (1.0 - u));
                let col = if k % 2 == 0 { Color::new(0.7, 0.55, 1.0, 0.9) } else { Color::new(0.4, 0.28, 0.8, 0.9) };
                draw_circle_lines(c.x, c.y, 2.2, 1.2, col);
            }
            draw_circle(to.x, to.y, 6.0, Color::new(0.6, 0.45, 1.0, 0.25));
        }
    }
    for s in &world.scars {
        let k = (s.life / 1.5).min(1.0);
        draw_line(s.from.x, s.from.y - 6.0, s.to.x, s.to.y - 6.0, 4.0, Color::new(0.55, 0.4, 1.0, 0.35 * k));
        draw_line(s.from.x, s.from.y - 6.0, s.to.x, s.to.y - 6.0, 1.5, Color::new(1.0, 1.0, 1.0, 0.6 * k));
    }
    for (from, to, age) in &fx.iai {
        let k = 1.0 - age / 0.25;
        draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 10.0 * k, Color::new(0.55, 0.45, 1.0, 0.5 * k));
        draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 3.0, Color::new(1.0, 1.0, 1.0, k));
    }
    for (c, r, age) in &fx.pulses {
        let k = age / 0.4;
        draw_circle_lines(c.x, c.y, r * (0.4 + 0.6 * k), 2.5, Color::new(0.6, 0.85, 1.0, 0.8 * (1.0 - k)));
    }
    for (from, to, age, st) in &fx.beams {
        if *st == style::MOON {
            let k = 1.0 - age / 0.45;
            draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 26.0 * k, Color::new(0.45, 0.6, 1.0, 0.25 * k));
            draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 13.0 * k, Color::new(0.75, 0.85, 1.0, 0.6 * k));
            draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 4.0 * k, Color::new(1.0, 1.0, 0.95, k));
            continue;
        }
        let k = 1.0 - age / 0.3;
        draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 16.0 * k, Color::new(0.35, 0.0, 0.05, 0.55 * k));
        draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 9.0 * k, Color::new(0.9, 0.05, 0.15, 0.85 * k));
        draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 3.0 * k, Color::new(1.0, 0.75, 0.75, k));
    }
    draw_blasts(world, time as f32);
    for (c, r, age, style) in &fx.bursts {
        let k = age / burst_life(*style);
        let rr = r * (0.3 + 0.7 * k.sqrt());
        let fade = 1.0 - k;
        let (fill, edge, width) = match *style {
            1 => (Color::new(1.0, 0.7, 0.2, 0.35 * fade), Color::new(1.0, 0.9, 0.5, 0.9 * fade), 5.0),
            style::SEAL => (Color::new(1.0, 0.85, 0.3, 0.3 * fade), Color::new(1.0, 0.95, 0.6, fade), 2.0),
            style::METEOR => (Color::new(1.0, 0.45, 0.1, 0.5 * fade), Color::new(1.0, 0.85, 0.4, fade), 7.0),
            style::FOX => (Color::new(1.0, 0.55, 0.15, 0.35 * fade), Color::new(1.0, 0.85, 0.4, fade), 2.5),
            style::SKYBOLT => (Color::new(0.7, 0.85, 1.0, 0.4 * fade), Color::new(1.0, 1.0, 1.0, fade), 3.0),
            style::COMET => (Color::new(0.3, 0.6, 1.0, 0.45 * fade), Color::new(0.85, 0.95, 1.0, fade), 4.0),
            style::QUAKE => (Color::new(1.0, 0.6, 0.2, 0.25 * fade), Color::new(1.0, 0.85, 0.4, fade), 6.0),
            2 => (Color::new(0.6, 0.0, 0.1, 0.4 * fade), Color::new(1.0, 0.3, 0.3, 0.9 * fade), 3.0),
            3 => (Color::new(0.25, 0.0, 0.05, 0.55 * fade), Color::new(1.0, 0.15, 0.2, fade), 8.0),
            _ => (Color::new(0.3, 0.65, 1.0, 0.35 * fade), Color::new(0.8, 0.95, 1.0, 0.9 * fade), 2.0),
        };
        draw_circle(c.x, c.y, rr, fill);
        match *style {
            style::SEAL => {
                // A spinning seal glyph: square inside the ring.
                let a = k * 3.0;
                let q = rr * 0.6;
                for i in 0..4 {
                    let (a0, a1) = (a + i as f32 * std::f32::consts::FRAC_PI_2, a + (i + 1) as f32 * std::f32::consts::FRAC_PI_2);
                    draw_line(c.x + a0.cos() * q, c.y + a0.sin() * q, c.x + a1.cos() * q, c.y + a1.sin() * q, 2.0, Color::new(1.0, 0.3, 0.25, fade));
                }
            }
            style::METEOR => {
                draw_circle(c.x, c.y, rr * 0.55, Color::new(1.0, 0.95, 0.7, 0.7 * fade));
                for i in 0..12 {
                    let a = i as f32 * 0.52 + c.x;
                    let d = rr * (0.4 + 0.8 * k);
                    draw_rectangle(c.x + a.cos() * d, c.y + a.sin() * d * 0.7, 3.0, 3.0, Color::new(0.4, 0.2, 0.1, fade));
                }
            }
            style::SKYBOLT => {
                // The bolt column from the top of the sky.
                let mut prev = vec2(c.x, c.y - 300.0);
                for s in 1..=8 {
                    let y = c.y - 300.0 + 300.0 * s as f32 / 8.0;
                    let x = if s < 8 { c.x + ((s as f32 * 5.3 + c.y).sin()) * 10.0 } else { c.x };
                    draw_line(prev.x, prev.y, x, y, 6.0 * fade, Color::new(0.6, 0.8, 1.0, 0.7 * fade));
                    draw_line(prev.x, prev.y, x, y, 2.5 * fade, Color::new(1.0, 1.0, 1.0, fade));
                    prev = vec2(x, y);
                }
            }
            style::QUAKE => {
                draw_circle_lines(c.x, c.y, rr * 0.6, 3.0, Color::new(1.0, 0.4, 0.3, 0.8 * fade));
            }
            _ => {}
        }
        if *style == 3 {
            draw_circle(c.x, c.y, rr * 0.6, Color::new(0.05, 0.0, 0.02, 0.5 * fade));
            draw_circle_lines(c.x, c.y, rr * 0.8, 4.0, Color::new(0.9, 0.05, 0.1, 0.7 * fade));
        }
        draw_circle_lines(c.x, c.y, rr, width, edge);
    }

    // Enemy fireballs: drawn above everything so they always read as danger.
    for s in &world.shots {
        let r = sim::content::SHOT_RADIUS;
        let flick = 0.8 + 0.2 * (time as f32 * 20.0 + s.pos.x).sin();
        draw_circle(s.pos.x, s.pos.y, r * 2.0, Color::new(1.0, 0.35, 0.15, 0.22 * flick));
        draw_circle(s.pos.x, s.pos.y, r, Color::new(1.0, 0.45, 0.2, 1.0));
        draw_circle(s.pos.x, s.pos.y, r * 0.5, Color::new(1.0, 0.95, 0.7, 1.0));
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
    if let Some(s) = world.shrine {
        let sp = cam.world_to_screen(vec2(s.x, s.y));
        let (w, h) = (screen_width(), screen_height());
        // Only when the shrine sprite is fully out of view (it extends ~60px above its anchor).
        if sp.x < -30.0 || sp.y < -10.0 || sp.x > w + 30.0 || sp.y > h + 70.0 {
            let c = vec2(w / 2.0, h / 2.0);
            let d = (sp - c).normalize_or_zero();
            // Push the arrow to the screen edge along the direction to the shrine.
            let t = ((w / 2.0 - 40.0) / d.x.abs().max(1e-3)).min((h / 2.0 - 40.0) / d.y.abs().max(1e-3));
            let tip = c + d * t;
            let side = vec2(-d.y, d.x);
            let pulse = 0.7 + 0.3 * (time * 5.0).sin() as f32;
            draw_triangle(tip + d * 14.0, tip - d * 8.0 + side * 10.0, tip - d * 8.0 - side * 10.0, Color::new(1.0, 0.85, 0.4, pulse));
            draw_texture_ex(&a.shrine, tip.x - d.x * 34.0 - 12.0, tip.y - d.y * 34.0 - 14.0, Color::new(1.0, 1.0, 1.0, 0.85), DrawTextureParams { dest_size: Some(vec2(24.0, 27.0)), ..Default::default() });
        }
    }
    if let Some((line, left)) = &fx.bark {
        let s = cam.world_to_screen(vec2(p.x, p.y - 34.0));
        let d = measure_text(line, None, 20, 1.0);
        let alpha = left.min(0.4) / 0.4;
        draw_rectangle(s.x - d.width / 2.0 - 8.0, s.y - 20.0, d.width + 16.0, 28.0, Color::new(0.05, 0.03, 0.1, 0.8 * alpha));
        draw_text(line, s.x - d.width / 2.0, s.y, 20.0, Color::new(1.0, 0.9, 0.95, alpha));
    }
}

/// Ninefold Foxfire: golden aura and nine lashing foxfire tails behind the player.
fn draw_fox_aura_and_tails(world: &World, t: f32) {
    let p = world.player.pos;
    let pulse = 0.5 + 0.5 * (t * 6.0).sin();
    let ar = sim::content::FOX_AURA_RADIUS;
    draw_circle(p.x, p.y - 6.0, ar * 1.25, Color::new(1.0, 0.55, 0.1, 0.07 + 0.04 * pulse));
    draw_circle(p.x, p.y - 6.0, ar, Color::new(1.0, 0.7, 0.2, 0.10 + 0.05 * pulse));
    draw_circle_lines(p.x, p.y - 6.0, ar, 1.5, Color::new(1.0, 0.85, 0.4, 0.5));
    for (k, (root, tip)) in world.fox_tails().iter().enumerate() {
        let d = *tip - *root;
        let n = vec2(-d.y, d.x).normalize_or_zero();
        // A curved, tapering flame: bend the midpoint sideways, wave along the length.
        let bend = (t * 6.0 + k as f32 * 1.3).sin() * 14.0;
        let steps = 22;
        for i in 0..=steps {
            let u = i as f32 / steps as f32;
            let base = vec2(root.x, root.y) + vec2(d.x, d.y) * u;
            let off = n * (bend * 4.0 * u * (1.0 - u) + (t * 11.0 - u * 6.0 + k as f32).sin() * 2.0 * u);
            let c = base + off;
            let r = 7.5 * (1.0 - u * 0.7);
            draw_circle(c.x, c.y, r * 1.5, Color::new(1.0, 0.45, 0.05, 0.12));
            draw_circle(c.x, c.y, r, Color::new(1.0, 0.62, 0.12, 0.55));
            draw_circle(c.x, c.y, r * 0.5, Color::new(1.0, 0.92, 0.55, 0.75));
        }
    }
}

/// Golden spirit motes around Riku: a few in base form, a storm while transformed.
fn draw_spirit_motes(p: sim::Vec2, t: f32, fox: bool) {
    let n = if fox { 28 } else { 6 };
    for k in 0..n {
        let kf = k as f32;
        let phase = (t * (0.5 + (k % 5) as f32 * 0.08) + kf * 0.37).fract();
        let ang = kf * 2.399;
        let rad = if fox { 14.0 + (k % 7) as f32 * 7.0 } else { 9.0 + (k % 3) as f32 * 3.0 };
        let x = p.x + ang.cos() * rad;
        let y = p.y - 8.0 + ang.sin() * rad * 0.5 - phase * if fox { 46.0 } else { 18.0 };
        let a = (1.0 - phase) * if fox { 0.95 } else { 0.55 };
        let s = if fox && k % 3 == 0 { 2.0 } else { 1.0 };
        draw_rectangle(x, y, s, s, Color::new(1.0, 0.85, 0.35, a));
    }
}

fn burst_life(st: u8) -> f32 {
    match st {
        style::METEOR => 0.8,
        style::SKYBOLT => 0.3,
        style::COMET => 0.5,
        style::QUAKE => 0.5,
        style::SEAL => 0.3,
        1 => 0.7,
        2 => 0.4,
        3 => 1.0,
        _ => 0.35,
    }
}

/// Shun's oversized nodachi, pivoting at his right hand. `swing`: (time left, direction).
fn draw_blade(a: &Assets, p: sim::Vec2, flip: bool, hannya: bool, swing: Option<(f32, Vec2)>) {
    let tex = if hannya { &a.cleaver_hannya } else { &a.cleaver };
    let side = if flip { -1.0 } else { 1.0 };
    let hand = vec2(p.x + 8.0 * side, p.y - 6.0);
    let rot = match swing {
        // Sweep the blade through ~150 degrees around the swing direction.
        Some((left, dir)) => {
            let k = 1.0 - left / 0.2;
            let d = if dir == Vec2::ZERO { vec2(side, 0.0) } else { dir };
            d.y.atan2(d.x) + std::f32::consts::FRAC_PI_2 - 1.3 + 2.6 * k
        }
        // Resting over the shoulder.
        None => 0.55 * side,
    };
    draw_texture_ex(
        tex,
        hand.x - 4.5,
        hand.y - 35.0,
        WHITE,
        DrawTextureParams { rotation: rot, pivot: Some(hand), flip_x: flip, ..Default::default() },
    );
}

/// Hannya Unbound: a pulsing crimson-black aura with dark flames licking upward.
fn draw_hannya_aura(world: &World, t: f32) {
    let p = world.player.pos;
    let pulse = 0.5 + 0.5 * (t * 7.0).sin();
    let r = sim::content::HANNYA_AURA_RADIUS;
    draw_circle(p.x, p.y - 6.0, r * 1.3, Color::new(0.3, 0.0, 0.05, 0.16 + 0.06 * pulse));
    draw_circle(p.x, p.y - 6.0, r, Color::new(0.55, 0.0, 0.08, 0.14 + 0.06 * pulse));
    draw_circle_lines(p.x, p.y - 6.0, r * (0.95 + 0.05 * pulse), 2.0, Color::new(1.0, 0.2, 0.25, 0.6));
    for k in 0..18 {
        let kf = k as f32;
        let ph = (t * 1.6 + kf * 0.29).fract();
        let a = kf * 2.39;
        let rr = 10.0 + (k % 5) as f32 * 9.0;
        let x = p.x + a.cos() * rr;
        let y = p.y - 4.0 + a.sin() * rr * 0.5 - ph * 40.0;
        let c = if k % 3 == 0 { Color::new(0.05, 0.0, 0.02, 0.8 * (1.0 - ph)) } else { Color::new(0.95, 0.1, 0.15, 0.85 * (1.0 - ph)) };
        draw_rectangle(x, y, 2.0, 3.0, c);
    }
    // Hannya's Wrath charging: a ring of dread collapsing inward.
    if world.player.form <= sim::content::HANNYA_FINALE_CHARGE {
        let k = 1.0 - world.player.form / sim::content::HANNYA_FINALE_CHARGE;
        let rr = sim::content::HANNYA_FINALE_RADIUS * (1.0 - k * 0.8);
        draw_circle_lines(p.x, p.y - 6.0, rr, 4.0, Color::new(1.0, 0.1, 0.15, 0.4 + 0.5 * k));
        draw_circle(p.x, p.y - 30.0, 6.0 + 16.0 * k, Color::new(0.1, 0.0, 0.03, 0.9));
        draw_circle_lines(p.x, p.y - 30.0, 6.0 + 16.0 * k, 2.0, Color::new(1.0, 0.2, 0.25, 1.0));
    }
}

/// Spirit wisps around Shun: pale violet in base form, crimson and black when unbound.
fn draw_reaper_wisps(p: sim::Vec2, t: f32, hannya: bool) {
    let n = if hannya { 10 } else { 7 };
    for k in 0..n {
        let kf = k as f32;
        let a = t * (0.9 + 0.1 * (k % 3) as f32) + kf * std::f32::consts::TAU / n as f32;
        let r = 13.0 + 3.0 * (t * 2.0 + kf).sin();
        let x = p.x + a.cos() * r;
        let y = p.y - 10.0 + a.sin() * r * 0.6;
        let c = if hannya {
            if k % 2 == 0 { Color::new(0.9, 0.1, 0.15, 0.8) } else { Color::new(0.05, 0.0, 0.03, 0.8) }
        } else if k % 3 == 0 {
            Color::new(0.2, 0.15, 0.3, 0.6)
        } else {
            Color::new(0.75, 0.7, 1.0, 0.55)
        };
        draw_rectangle(x, y, 2.0, 2.0, c);
    }
}

/// Telegraphs for delayed blasts: falling meteors and comets, sky-bolt warnings.
fn draw_blasts(world: &World, t: f32) {
    for b in &world.blasts {
        let k = 1.0 - b.delay / b.max_delay; // 0 -> 1 as it lands
        let c = b.pos;
        match b.style {
            style::METEOR | style::COMET => {
                let meteor = b.style == style::METEOR;
                let (ring, body, core) = if meteor {
                    (Color::new(1.0, 0.3, 0.1, 0.35 + 0.4 * k), Color::new(1.0, 0.45, 0.1, 1.0), Color::new(1.0, 0.9, 0.5, 1.0))
                } else {
                    (Color::new(0.3, 0.6, 1.0, 0.3 + 0.4 * k), Color::new(0.3, 0.65, 1.0, 1.0), Color::new(0.9, 0.97, 1.0, 1.0))
                };
                draw_circle(c.x, c.y, b.radius * k, Color::new(ring.r, ring.g, ring.b, 0.12));
                draw_circle_lines(c.x, c.y, b.radius * (1.0 - 0.6 * k) + 4.0, 2.0, ring);
                // The body falls in from the upper right with a fiery trail.
                let size = if meteor { 18.0 } else { 9.0 };
                let off = vec2(1.0, -1.6) * (1.0 - k) * 320.0;
                let head = vec2(c.x, c.y) + off;
                for i in 1..10 {
                    let tp = head + vec2(1.0, -1.6).normalize() * (i as f32 * size * 0.7);
                    let a = 0.7 * (1.0 - i as f32 / 10.0);
                    draw_circle(tp.x, tp.y, size * (1.0 - i as f32 / 12.0), Color::new(body.r, body.g * 0.8, body.b, a * 0.6));
                }
                draw_circle(head.x, head.y, size * 1.4, Color::new(body.r, body.g, body.b, 0.3));
                draw_circle(head.x, head.y, size, body);
                draw_circle(head.x, head.y, size * 0.5, core);
                let _ = t;
            }
            style::SKYBOLT => {
                draw_circle_lines(c.x, c.y, b.radius * (1.0 - 0.5 * k), 1.5, Color::new(0.8, 0.9, 1.0, 0.8));
            }
            _ => {}
        }
    }
}
