//! The run simulation: player, weapons, spawn director, enemies, pickups.

use crate::content::*;
use crate::grid::Grid;
use crate::meta::Bonus;
use crate::stage;
use crate::rng::Rng;
use crate::Vec2;

pub const DT: f32 = 1.0 / 60.0;

const PLAYER_RADIUS: f32 = 8.0;
const PROJECTILE_RADIUS: f32 = 4.0;
const SEPARATION_NEIGHBOURS: u32 = 10;
const MAX_ENEMIES: usize = 4000;
/// When this many gems are on the field, nearby gems merge into higher-value ones...
const GEM_MERGE_AT: usize = 400;
/// ...until at most this many remain.
const GEM_MERGE_TO: usize = 250;
const ORB_RADIUS: f32 = 8.0;
const ORB_ORBIT: f32 = 46.0;
const FIELD_RADIUS: f32 = 44.0;
const IFRAMES: f32 = 0.5;

/// Debug load test: fixed enemy count + extra radial projectiles, no damage, no level-ups.
#[derive(Clone, Copy, Debug)]
pub struct Stress {
    pub enemies: usize,
    pub projectiles: usize,
}

#[derive(Clone, Debug)]
pub struct WorldConfig {
    pub seed: u64,
    pub hero: HeroId,
    pub stress: Option<Stress>,
    pub spawn_radius: f32,
    pub recycle_radius: f32,
    /// Permanent bonuses from the Shrine Shop.
    pub meta: Bonus,
}

impl Default for WorldConfig {
    fn default() -> Self {
        WorldConfig { seed: 1, hero: HeroId::Ren, stress: None, spawn_radius: 420.0, recycle_radius: 900.0, meta: Bonus::default() }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    /// Desired movement, any length (normalised internally).
    pub move_dir: Vec2,
    /// Trigger the character's special move (ignored until the meter is full).
    pub special: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    Playing,
    Dead,
    Won,
}

/// Things that happened this tick, for rendering/audio. Cleared every step.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Hit { pos: Vec2, amount: f32 },
    Kill { pos: Vec2, kind: EnemyKind },
    PlayerHurt,
    LevelUp,
    SpecialUsed { pos: Vec2 },
    BossSpawned,
    /// The Great Oni is winding up a charge in `dir`.
    BossTelegraph,
    Awakened(WeaponKind),
    /// One segment of a Thunder Chain bolt.
    Bolt { from: Vec2, to: Vec2 },
    /// A Crescent Slash arc (`full` = Awakened full circle).
    Slash { pos: Vec2, dir: Vec2, radius: f32, full: bool },
    /// A Star Fall strike landing.
    Impact { pos: Vec2, radius: f32 },
    ShrineAppeared,
    ShrineChosen(ShrineEvent),
    NightLordSpawned,
    /// The player went down but a Revival brought them back.
    Revived,
    SenPicked { amount: u32 },
    /// A Nopperabo reappeared beside the player.
    Blink { pos: Vec2 },
    /// XP gems were collected this tick.
    GemsPicked,
    /// An enemy fired a fireball.
    EnemyShot,
    /// A weapon fired a volley / beat / pool (once per trigger, not per projectile).
    Fired(WeaponKind),
}

/// Enemy fireball.
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
}

/// Expanding Taiko shockwave ring.
#[derive(Clone, Copy, Debug)]
pub struct Wave {
    pub center: Vec2,
    pub radius: f32,
    pub max_radius: f32,
    pub speed: f32,
    pub damage: f32,
}

/// Star Fall strike waiting to land.
#[derive(Clone, Copy, Debug)]
pub struct Strike {
    pub pos: Vec2,
    pub delay: f32,
    pub radius: f32,
    pub damage: f32,
}

/// Burning ground from Flame Pool.
#[derive(Clone, Copy, Debug)]
pub struct Pool {
    pub pos: Vec2,
    pub radius: f32,
    pub life: f32,
    pub max_life: f32,
    tick: f32,
    pub damage: f32,
}

/// One reward from an opened chest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChestItem {
    LevelUp(Upgrade, u8),
    Awakening(WeaponKind),
    Heal,
}

#[derive(Default)]
pub struct Enemies {
    pub id: Vec<u32>,
    pub kind: Vec<EnemyKind>,
    pub pos: Vec<Vec2>,
    pub vel: Vec<Vec2>,
    /// Knockback velocity, decays every tick.
    pub push: Vec<Vec2>,
    pub hp: Vec<f32>,
    pub speed: Vec<f32>,
    pub radius: Vec<f32>,
    pub damage: Vec<f32>,
    pub xp: Vec<u32>,
    pub knock: Vec<f32>,
    /// Ticks of hit-flash remaining (render only).
    pub flash: Vec<u8>,
    /// Ticks until moon orbs may hit this enemy again.
    pub orb_cd: Vec<u8>,
    /// Boss AI: state (0 chase, 1 telegraph, 2 charge), timer, charge direction.
    pub ai_state: Vec<u8>,
    pub ai_timer: Vec<f32>,
    pub ai_dir: Vec<Vec2>,
    /// Ticks until a Taiko wave may hit this enemy again.
    pub wave_cd: Vec<u8>,
}

impl Enemies {
    pub fn len(&self) -> usize {
        self.pos.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pos.is_empty()
    }
    fn swap_remove(&mut self, i: usize) {
        self.id.swap_remove(i);
        self.kind.swap_remove(i);
        self.pos.swap_remove(i);
        self.vel.swap_remove(i);
        self.push.swap_remove(i);
        self.hp.swap_remove(i);
        self.speed.swap_remove(i);
        self.radius.swap_remove(i);
        self.damage.swap_remove(i);
        self.xp.swap_remove(i);
        self.knock.swap_remove(i);
        self.flash.swap_remove(i);
        self.orb_cd.swap_remove(i);
        self.ai_state.swap_remove(i);
        self.ai_timer.swap_remove(i);
        self.ai_dir.swap_remove(i);
        self.wave_cd.swap_remove(i);
    }
}

#[derive(Default)]
pub struct Projectiles {
    pub pos: Vec<Vec2>,
    pub vel: Vec<Vec2>,
    pub life: Vec<f32>,
    pub damage: Vec<f32>,
    pub pierce: Vec<u8>,
    /// Id of the last enemy hit, so one projectile can't multi-hit while overlapping.
    pub last_hit: Vec<u32>,
    /// Max turn rate toward the nearest enemy, radians/second (0 = straight).
    pub homing: Vec<f32>,
}

impl Projectiles {
    pub fn len(&self) -> usize {
        self.pos.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pos.is_empty()
    }
    fn push(&mut self, pos: Vec2, vel: Vec2, life: f32, damage: f32, pierce: u8, homing: f32) {
        self.pos.push(pos);
        self.vel.push(vel);
        self.life.push(life);
        self.damage.push(damage);
        self.pierce.push(pierce);
        self.last_hit.push(u32::MAX);
        self.homing.push(homing);
    }
    fn swap_remove(&mut self, i: usize) {
        self.pos.swap_remove(i);
        self.vel.swap_remove(i);
        self.life.swap_remove(i);
        self.damage.swap_remove(i);
        self.pierce.swap_remove(i);
        self.last_hit.swap_remove(i);
        self.homing.swap_remove(i);
    }
}

#[derive(Default)]
pub struct Gems {
    pub pos: Vec<Vec2>,
    pub value: Vec<u32>,
}

impl Gems {
    /// Merges gems that share a grid cell into one gem holding their total value,
    /// placed at their value-weighted centre. The cell doubles until at most `target`
    /// gems remain, so XP always stays close to where it dropped. Deterministic.
    fn merge(&mut self, target: usize) {
        let mut cell = 48.0f32;
        while self.pos.len() > target && cell < 4096.0 {
            let key = |p: Vec2| ((p.x / cell).floor() as i64, (p.y / cell).floor() as i64);
            let mut order: Vec<usize> = (0..self.pos.len()).collect();
            order.sort_by_key(|&i| key(self.pos[i]));
            let (mut pos, mut value) = (Vec::new(), Vec::new());
            let mut k = 0;
            while k < order.len() {
                let cell_key = key(self.pos[order[k]]);
                let (mut sum, mut total) = (Vec2::ZERO, 0u32);
                while k < order.len() && key(self.pos[order[k]]) == cell_key {
                    let i = order[k];
                    sum += self.pos[i] * self.value[i] as f32;
                    total += self.value[i];
                    k += 1;
                }
                pos.push(sum * (1.0 / total as f32));
                value.push(total);
            }
            self.pos = pos;
            self.value = value;
            cell *= 2.0;
        }
    }
}

pub struct Player {
    pub pos: Vec2,
    pub facing: Vec2,
    pub moving: bool,
    pub hp: f32,
    pub max_hp: f32,
    pub iframes: f32,
    pub xp: u32,
    pub level: u32,
    /// Special-move charge, 0..=1.
    pub special: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct OwnedWeapon {
    pub kind: WeaponKind,
    pub level: u8,
    pub awakened: bool,
    timer: f32,
    /// Orbit angle (MoonOrbs) / last pulse age (SakuraField), exposed for rendering.
    pub phase: f32,
}

pub struct World {
    pub cfg: WorldConfig,
    pub player: Player,
    pub weapons: Vec<OwnedWeapon>,
    pub passives: Vec<(PassiveKind, u8)>,
    pub enemies: Enemies,
    pub projectiles: Projectiles,
    pub gems: Gems,
    pub time: f32,
    pub tick: u64,
    pub kills: u64,
    pub state: RunState,
    /// Level-up choices waiting for the player. The sim pauses while this is Some.
    pub offer: Option<Vec<Upgrade>>,
    /// Chests on the ground: position and tier (number of rewards).
    pub chests: Vec<(Vec2, u8)>,
    /// Rewards from the chest just opened. The sim pauses until `close_chest`.
    pub chest_result: Option<Vec<ChestItem>>,
    pub waves: Vec<Wave>,
    pub strikes: Vec<Strike>,
    pub pools: Vec<Pool>,
    /// Run modifiers from Shrine Events.
    pub mods: Modifiers,
    /// Enemy fireballs in flight.
    pub shots: Vec<Shot>,
    /// Hero innate + Shrine Shop bonuses.
    pub bonus: Bonus,
    revivals_left: u8,
    /// Sen collected this run.
    pub sen: u32,
    /// Sen coins on the ground: (position, value).
    pub coins: Vec<(Vec2, u32)>,
    pub great_oni_defeated: bool,
    /// Kills per enemy kind, indexed like `ALL_ENEMIES`.
    pub kills_by_kind: [u32; ALL_ENEMIES.len()],
    /// Shrine Events taken this run.
    pub shrines_taken: Vec<ShrineEvent>,
    /// True if the run ended because dawn arrived with the Night Lord alive.
    pub dawn_out: bool,
    night_lord_spawned: bool,
    pending_spawns: Vec<(EnemyKind, Vec2)>,
    /// A shrine waiting on the ground.
    pub shrine: Option<Vec2>,
    /// Two Blessing+Price pairs to choose from. The sim pauses until `choose_shrine`.
    pub shrine_offer: Option<Vec<ShrineEvent>>,
    shrines_done: usize,
    xp_carry: f32,
    boss_spawned: bool,
    pub events: Vec<Event>,
    pending_levels: u32,
    next_id: u32,
    next_elite: f32,
    rings_done: usize,
    rng: Rng,
    grid: Grid,
    scratch: Vec<u32>,
    dead: Vec<usize>,
}

impl World {
    pub fn new(cfg: WorldConfig) -> Self {
        let rng = Rng::new(cfg.seed);
        let hero = hero_def(cfg.hero);
        let signature = hero.signature;
        let bonus = hero.innate.plus(cfg.meta);
        let max_hp = 100.0 * (1.0 + bonus.max_hp);
        let cell = 28.0;
        let dim = ((cfg.recycle_radius * 2.2) / cell).ceil() as usize;
        World {
            cfg,
            player: Player {
                pos: Vec2::ZERO,
                facing: Vec2::new(1.0, 0.0),
                moving: false,
                hp: max_hp,
                max_hp,
                iframes: 0.0,
                xp: 0,
                level: 1,
                special: 0.0,
            },
            weapons: vec![OwnedWeapon { kind: signature, level: 1, awakened: false, timer: 0.0, phase: 0.0 }],
            passives: Vec::new(),
            enemies: Enemies::default(),
            projectiles: Projectiles::default(),
            gems: Gems::default(),
            time: 0.0,
            tick: 0,
            kills: 0,
            state: RunState::Playing,
            offer: None,
            chests: Vec::new(),
            chest_result: None,
            waves: Vec::new(),
            strikes: Vec::new(),
            pools: Vec::new(),
            mods: Modifiers::default(),
            shots: Vec::new(),
            bonus,
            revivals_left: bonus.revivals,
            sen: 0,
            coins: Vec::new(),
            great_oni_defeated: false,
            kills_by_kind: [0; ALL_ENEMIES.len()],
            shrines_taken: Vec::new(),
            dawn_out: false,
            night_lord_spawned: false,
            pending_spawns: Vec::new(),
            shrine: None,
            shrine_offer: None,
            shrines_done: 0,
            xp_carry: 0.0,
            boss_spawned: false,
            events: Vec::new(),
            pending_levels: 0,
            next_id: 0,
            next_elite: ELITE_EVERY,
            rings_done: 0,
            rng,
            grid: Grid::new(cell, dim),
            scratch: Vec::new(),
            dead: Vec::new(),
        }
    }

    pub fn paused(&self) -> bool {
        self.state != RunState::Playing || self.offer.is_some() || self.chest_result.is_some() || self.shrine_offer.is_some()
    }

    pub fn step(&mut self, input: Input) {
        self.events.clear();
        if self.paused() {
            return;
        }
        self.tick += 1;
        self.time += DT;
        self.move_player(input);
        self.director();
        self.grid.rebuild(self.player.pos, &self.enemies.pos);
        self.move_enemies();
        self.update_weapons();
        self.update_effects();
        self.move_projectiles_and_hit();
        if input.special {
            self.use_special();
        }
        self.contact_damage();
        self.update_shots();
        // Everything that reads the grid must run before this: removal reorders
        // the enemy arrays and invalidates the grid's indices.
        self.remove_dead_enemies();
        self.collect_gems();
        self.collect_chests();
        self.collect_shrine();
        if self.state == RunState::Playing && self.time >= DAWN_LIMIT {
            self.dawn_out = true;
            self.state = RunState::Dead;
        }
        if self.pending_levels > 0 && self.offer.is_none() && self.cfg.stress.is_none() {
            self.make_offer();
        }
    }

    // ───────────── Stats from passives ─────────────

    pub fn passive_level(&self, kind: PassiveKind) -> u8 {
        self.passives.iter().find(|(k, _)| *k == kind).map_or(0, |(_, l)| *l)
    }
    pub fn weapon_level(&self, kind: WeaponKind) -> u8 {
        self.weapons.iter().find(|w| w.kind == kind).map_or(0, |w| w.level)
    }
    fn might(&self) -> f32 {
        (1.0 + 0.10 * self.passive_level(PassiveKind::Might) as f32 + self.bonus.might) * self.mods.damage
    }
    fn haste(&self) -> f32 {
        1.0 - 0.08 * self.passive_level(PassiveKind::Haste) as f32 - self.bonus.cooldown
    }
    pub fn area_mult(&self) -> f32 {
        1.0 + 0.10 * self.passive_level(PassiveKind::Expanse) as f32 + self.bonus.area
    }
    fn move_speed(&self) -> f32 {
        110.0 * (1.0 + 0.08 * self.passive_level(PassiveKind::Swiftness) as f32 + self.bonus.move_speed) * self.mods.move_speed
    }
    pub fn magnet(&self) -> f32 {
        55.0 * (1.0 + self.bonus.magnet) * (1.0 + 0.30 * self.passive_level(PassiveKind::Magnet) as f32) * self.mods.magnet
    }
    pub fn xp_to_next(&self) -> u32 {
        xp_needed(self.player.level)
    }

    /// World radius of the moon-orb orbit and Sakura Field, for rendering.
    pub fn orb_orbit(&self, level: u8) -> f32 {
        ORB_ORBIT * weapon_stats(WeaponKind::MoonOrbs, level).area * self.area_mult()
    }
    pub fn orb_radius(&self, level: u8) -> f32 {
        ORB_RADIUS * weapon_stats(WeaponKind::MoonOrbs, level).area * self.area_mult()
    }
    pub fn field_radius(&self, level: u8) -> f32 {
        FIELD_RADIUS * weapon_stats(WeaponKind::SakuraField, level).area * self.area_mult()
    }

    /// Stats for an owned weapon, Awakened form included (before passives).
    pub fn stats_of(&self, w: &OwnedWeapon) -> WeaponStats {
        let mut s = if w.awakened { awakened_stats(w.kind) } else { weapon_stats(w.kind, w.level) };
        if w.kind != WeaponKind::SakuraField {
            s.amount += self.passive_level(PassiveKind::Abundance) as u32;
        }
        s.duration *= self.persistence();
        s
    }

    pub fn field_radius_of(&self, w: &OwnedWeapon) -> f32 {
        FIELD_RADIUS * self.stats_of(w).area * self.area_mult()
    }

    /// World positions of each moon orb. Awakened adds a counter-rotating outer ring.
    pub fn orb_positions(&self, w: &OwnedWeapon) -> Vec<Vec2> {
        let s = self.stats_of(w);
        let orbit = ORB_ORBIT * s.area * self.area_mult();
        let n = s.amount;
        let mut out = Vec::with_capacity(n as usize * 2);
        for k in 0..n {
            let a = w.phase + k as f32 / n as f32 * std::f32::consts::TAU;
            out.push(self.player.pos + Vec2::from_angle(a) * orbit);
            if w.awakened {
                let b = -w.phase * 0.8 + (k as f32 + 0.5) / n as f32 * std::f32::consts::TAU;
                out.push(self.player.pos + Vec2::from_angle(b) * (orbit * 1.65));
            }
        }
        out
    }

    /// The living boss, if any (Night Lord first): (index, hp fraction, kind).
    pub fn boss(&self) -> Option<(usize, f32, EnemyKind)> {
        let k = &self.enemies.kind;
        let i = k.iter().position(|k| *k == EnemyKind::NightLord).or_else(|| k.iter().position(|k| *k == EnemyKind::GreatOni))?;
        let max = match k[i] {
            EnemyKind::NightLord => night_lord_hp(),
            _ => enemy_def(EnemyKind::GreatOni).hp * hp_scale(MIDBOSS_AT),
        };
        Some((i, (self.enemies.hp[i] / max).clamp(0.0, 1.0), k[i]))
    }

    // ───────────── Player ─────────────

    fn armor(&self) -> f32 {
        self.passive_level(PassiveKind::Guard) as f32 + self.mods.armor + self.bonus.armor
    }

    /// Applies a hit to the player (after armor) and handles death / Revival.
    fn hurt_player(&mut self, raw: f32) {
        let dmg = (raw - self.armor()).max(1.0);
        self.player.hp = (self.player.hp - dmg).max(0.0);
        self.player.iframes = IFRAMES;
        self.events.push(Event::PlayerHurt);
        if self.player.hp <= 0.0 {
            if self.revivals_left > 0 {
                self.revivals_left -= 1;
                self.player.hp = self.player.max_hp * 0.5;
                self.player.iframes = BARRIER_IFRAMES;
                self.player.special = 1.0;
                self.events.push(Event::Revived);
            } else {
                self.state = RunState::Dead;
            }
        }
    }

    fn persistence(&self) -> f32 {
        1.0 + 0.12 * self.passive_level(PassiveKind::Persistence) as f32
    }

    fn move_player(&mut self, input: Input) {
        let regen = 0.25 * self.passive_level(PassiveKind::Recovery) as f32 + self.bonus.recovery;
        self.heal(regen * DT);
        let d = input.move_dir.normalize_or_zero();
        self.player.moving = d != Vec2::ZERO;
        if self.player.moving {
            self.player.facing = d;
        }
        self.player.pos += d * (self.move_speed() * DT);
        self.player.pos = stage::resolve(self.player.pos, PLAYER_RADIUS);
    }

    fn contact_damage(&mut self) {
        if self.cfg.stress.is_some() {
            return;
        }
        if self.player.iframes > 0.0 {
            self.player.iframes -= DT;
            return;
        }
        let p = self.player.pos;
        let mut worst = 0.0f32;
        let enemies = &self.enemies;
        self.grid.for_each_near(p, PLAYER_RADIUS + MAX_ENEMY_RADIUS, |j| {
            let j = j as usize;
            let r = PLAYER_RADIUS + enemies.radius[j];
            let faded = enemies.kind[j] == EnemyKind::Nopperabo && enemies.ai_state[j] != 0;
            if enemies.hp[j] > 0.0 && !faded && (enemies.pos[j] - p).length_sq() <= r * r {
                worst = worst.max(enemies.damage[j]);
            }
        });
        if worst > 0.0 {
            self.hurt_player(worst);
        }
    }

    // ───────────── Spawning ─────────────

    fn ring_point(&mut self, radius: f32) -> Vec2 {
        let a = self.rng.range(0.0, std::f32::consts::TAU);
        self.player.pos + Vec2::from_angle(a) * radius
    }

    fn spawn_enemy(&mut self, kind: EnemyKind, pos: Vec2) {
        if self.enemies.len() >= MAX_ENEMIES {
            return;
        }
        let d = enemy_def(kind);
        let e = &mut self.enemies;
        e.id.push(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        e.kind.push(kind);
        e.pos.push(pos);
        e.vel.push(Vec2::ZERO);
        e.push.push(Vec2::ZERO);
        e.hp.push(if kind == EnemyKind::NightLord { night_lord_hp() } else { d.hp * hp_scale(self.time) * self.mods.enemy_hp });
        e.speed.push(d.speed * self.rng.range(0.9, 1.1) * self.mods.enemy_speed);
        e.radius.push(d.radius);
        e.damage.push(d.damage * damage_scale(self.time));
        e.xp.push(d.xp);
        e.knock.push(d.knockback);
        e.flash.push(0);
        e.orb_cd.push(0);
        e.ai_state.push(0);
        e.ai_timer.push(match kind {
            EnemyKind::Chochin => 1.5 + self.rng.range(0.0, 1.5),
            EnemyKind::Nopperabo => 2.5 + self.rng.range(0.0, 2.0),
            _ => 3.0,
        });
        e.ai_dir.push(Vec2::ZERO);
        e.wave_cd.push(0);
    }

    fn director(&mut self) {
        let target = match self.cfg.stress {
            Some(s) => s.enemies,
            None if self.night_lord_spawned => 0,
            None => (min_alive(self.time) as f32 * self.mods.curse) as usize,
        };
        // Refill is rate-limited (ramping over the run) so strong area weapons can't turn
        // the director into an endless XP fountain. Stress mode fills fast.
        let per_tick = if self.cfg.stress.is_some() { 30 } else { refill_per_tick(self.time) };
        let deficit = target.saturating_sub(self.enemies.len()).min(per_tick);
        for _ in 0..deficit {
            let r = self.rng.f32();
            let (c, n) = (chochin_share(self.time), nopperabo_share(self.time));
            let kind = if r < c {
                EnemyKind::Chochin
            } else if r < c + n {
                EnemyKind::Nopperabo
            } else if self.rng.f32() < kasa_share(self.time) {
                EnemyKind::Kasa
            } else {
                EnemyKind::Wisp
            };
            let r = self.cfg.spawn_radius + self.rng.range(0.0, 80.0);
            let p = self.ring_point(r);
            self.spawn_enemy(kind, p);
        }

        if self.cfg.stress.is_none() {
            if !self.night_lord_spawned && self.time >= RUN_SECONDS {
                self.night_lord_spawned = true;
                let p = self.ring_point(self.cfg.spawn_radius * 0.8);
                self.spawn_enemy(EnemyKind::NightLord, p);
                self.events.push(Event::NightLordSpawned);
            }
            for (k, p) in std::mem::take(&mut self.pending_spawns) {
                self.spawn_enemy(k, p);
            }
            if !self.boss_spawned && self.time >= MIDBOSS_AT {
                self.boss_spawned = true;
                let p = self.ring_point(self.cfg.spawn_radius * 0.8);
                self.spawn_enemy(EnemyKind::GreatOni, p);
                self.events.push(Event::BossSpawned);
            }
            if self.shrines_done < SHRINE_TIMES.len() && self.time >= SHRINE_TIMES[self.shrines_done] {
                self.shrines_done += 1;
                let a = self.rng.range(0.0, std::f32::consts::TAU);
                self.shrine = Some(stage::resolve(self.player.pos + Vec2::from_angle(a) * 240.0, 18.0));
                self.events.push(Event::ShrineAppeared);
            }
            if self.time >= self.next_elite {
                self.next_elite += elite_interval(self.time);
                let p = self.ring_point(self.cfg.spawn_radius);
                self.spawn_enemy(EnemyKind::Oni, p);
            }
            if self.rings_done < SWARM_RING_TIMES.len() && self.time >= SWARM_RING_TIMES[self.rings_done] {
                self.rings_done += 1;
                let n = 120;
                for k in 0..n {
                    let a = k as f32 / n as f32 * std::f32::consts::TAU;
                    let p = self.player.pos + Vec2::from_angle(a) * 270.0;
                    self.spawn_enemy(EnemyKind::Wisp, p);
                }
            }
        }

        // Enemies left far behind are moved back onto the spawn ring.
        let r2 = self.cfg.recycle_radius * self.cfg.recycle_radius;
        for i in 0..self.enemies.len() {
            if (self.enemies.pos[i] - self.player.pos).length_sq() > r2 {
                let r = self.cfg.spawn_radius + 40.0;
                self.enemies.pos[i] = self.ring_point(r);
            }
        }
    }

    // ───────────── Enemies ─────────────

    fn move_enemies(&mut self) {
        let player = self.player.pos;
        let time = self.time;
        let e = &mut self.enemies;
        // Pass 1: desired velocity = seek player + separation from grid neighbours.
        for i in 0..e.pos.len() {
            let p = e.pos[i];
            let ri = e.radius[i];
            let mut speed = e.speed[i];
            if e.kind[i] == EnemyKind::Kasa {
                // Hopping gait.
                speed *= 0.45 + 0.75 * ((time * 7.0 + e.id[i] as f32 * 0.7).sin().abs());
            }
            let to_player = player - p;
            match e.kind[i] {
                EnemyKind::Chochin => {
                    // Hold a firing range, strafing; spit a fireball every ~2.5s.
                    let d = to_player.length();
                    let dir = to_player.normalize_or_zero();
                    let side = Vec2::new(-dir.y, dir.x) * if e.id[i] % 2 == 0 { 1.0 } else { -1.0 };
                    let mv = if d > 220.0 { dir } else if d < 170.0 { dir * -1.0 } else { side * 0.6 };
                    e.vel[i] = mv * speed;
                    e.ai_timer[i] -= DT;
                    if e.ai_timer[i] <= 0.0 && d < 300.0 {
                        e.ai_timer[i] = 4.5;
                        self.shots.push(Shot { pos: p, vel: dir * 115.0, life: 4.0 });
                        self.events.push(Event::EnemyShot);
                    }
                    continue;
                }
                EnemyKind::Nopperabo => {
                    // Walk -> fade out (0.5s) -> reappear 140px from the player and fade in (0.6s).
                    // While faded (states 1 and 2) it stands still and deals no contact damage.
                    e.ai_timer[i] -= DT;
                    match e.ai_state[i] {
                        0 => {
                            e.vel[i] = to_player.normalize_or_zero() * speed;
                            if e.ai_timer[i] <= 0.0 {
                                e.ai_state[i] = 1;
                                e.ai_timer[i] = 0.5;
                            }
                        }
                        1 => {
                            e.vel[i] = Vec2::ZERO;
                            if e.ai_timer[i] <= 0.0 {
                                let a = self.rng.range(0.0, std::f32::consts::TAU);
                                e.pos[i] = player + Vec2::from_angle(a) * 140.0;
                                e.ai_state[i] = 2;
                                e.ai_timer[i] = NOPPERABO_FADE_IN;
                                self.events.push(Event::Blink { pos: e.pos[i] });
                            }
                        }
                        _ => {
                            e.vel[i] = Vec2::ZERO;
                            if e.ai_timer[i] <= 0.0 {
                                e.ai_state[i] = 0;
                                e.ai_timer[i] = 6.0 + self.rng.range(0.0, 3.0);
                            }
                        }
                    }
                    continue;
                }
                EnemyKind::NightLord => {
                    // Slow chase; a constant spiral; every 3s cycle burst -> aimed fan -> summon.
                    e.vel[i] = to_player.normalize_or_zero() * speed;
                    e.ai_dir[i].x += DT * 2.2;
                    e.ai_dir[i].y -= DT;
                    if e.ai_dir[i].y <= 0.0 {
                        e.ai_dir[i].y = 0.14;
                        let a = e.ai_dir[i].x;
                        for k in 0..2 {
                            let b = a + k as f32 * std::f32::consts::PI;
                            self.shots.push(Shot { pos: p, vel: Vec2::from_angle(b) * 95.0, life: 6.0 });
                        }
                    }
                    e.ai_timer[i] -= DT;
                    if e.ai_timer[i] <= 0.0 {
                        e.ai_timer[i] = 3.0;
                        match e.ai_state[i] {
                            0 => {
                                for k in 0..24 {
                                    let b = k as f32 / 24.0 * std::f32::consts::TAU;
                                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(b) * 110.0, life: 6.0 });
                                }
                            }
                            1 => {
                                let base = to_player.y.atan2(to_player.x);
                                for k in 0..9 {
                                    let b = base + (k as f32 - 4.0) * 0.12;
                                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(b) * 150.0, life: 5.0 });
                                }
                            }
                            _ => {
                                for k in 0..10 {
                                    let b = k as f32 / 10.0 * std::f32::consts::TAU;
                                    self.pending_spawns.push((EnemyKind::Chochin, player + Vec2::from_angle(b) * 210.0));
                                }
                            }
                        }
                        e.ai_state[i] = (e.ai_state[i] + 1) % 3;
                    }
                    continue;
                }
                _ => {}
            }
            if e.kind[i] == EnemyKind::GreatOni {
                // Chase → telegraph (stand still, 0.8s) → charge (0.7s) → chase.
                e.ai_timer[i] -= DT;
                e.vel[i] = match e.ai_state[i] {
                    0 => {
                        if e.ai_timer[i] <= 0.0 && to_player.length_sq() < 380.0 * 380.0 {
                            e.ai_state[i] = 1;
                            e.ai_timer[i] = 0.8;
                            e.ai_dir[i] = to_player.normalize_or_zero();
                            self.events.push(Event::BossTelegraph);
                        }
                        to_player.normalize_or_zero() * speed
                    }
                    1 => {
                        if e.ai_timer[i] <= 0.0 {
                            e.ai_state[i] = 2;
                            e.ai_timer[i] = 0.7;
                        }
                        Vec2::ZERO
                    }
                    _ => {
                        if e.ai_timer[i] <= 0.0 {
                            e.ai_state[i] = 0;
                            e.ai_timer[i] = 5.0;
                        }
                        e.ai_dir[i] * 290.0
                    }
                };
                continue;
            }
            let seek = to_player.normalize_or_zero() * speed;
            let mut sep = Vec2::ZERO;
            let mut seen = 0u32;
            let pos = &e.pos;
            let radius = &e.radius;
            self.grid.for_each_near(p, ri + MAX_ENEMY_RADIUS, |j| {
                let j = j as usize;
                if seen >= SEPARATION_NEIGHBOURS || j == i {
                    return;
                }
                let d = p - pos[j];
                let sep_r = ri + radius[j];
                let d2 = d.length_sq();
                if d2 < sep_r * sep_r && d2 > 1e-4 {
                    seen += 1;
                    let dist = d2.sqrt();
                    sep += d * ((sep_r - dist) / (dist * sep_r));
                }
            });
            e.vel[i] = seek + sep * 70.0;
        }
        // Pass 2: integrate, then slide around props (bosses smash straight through).
        for i in 0..e.pos.len() {
            let v = e.vel[i] + e.push[i];
            e.pos[i] += v * DT;
            if !matches!(e.kind[i], EnemyKind::GreatOni | EnemyKind::NightLord) {
                let before = e.pos[i];
                let after = stage::resolve_local(before, e.radius[i]);
                let n = after - before;
                if n.length_sq() > 1e-8 {
                    // Blocked: step along the obstacle face, on the side closer to the player.
                    let n = n.normalize_or_zero();
                    let t = Vec2::new(-n.y, n.x);
                    let to_p = player - after;
                    let dot = t.x * to_p.x + t.y * to_p.y;
                    let side = if dot.abs() > 1.0 { dot.signum() } else if e.id[i] % 2 == 0 { 1.0 } else { -1.0 };
                    e.pos[i] = after + t * (side * e.speed[i] * 0.8 * DT);
                } else {
                    e.pos[i] = after;
                }
            }
            e.push[i] = e.push[i] * 0.86;
            e.flash[i] = e.flash[i].saturating_sub(1);
            e.orb_cd[i] = e.orb_cd[i].saturating_sub(1);
            e.wave_cd[i] = e.wave_cd[i].saturating_sub(1);
        }
    }

    fn hurt_enemy(&mut self, j: usize, amount: f32, dir: Vec2) {
        let e = &mut self.enemies;
        e.hp[j] -= amount;
        e.flash[j] = 5;
        e.push[j] += dir.normalize_or_zero() * (150.0 * e.knock[j]);
        self.events.push(Event::Hit { pos: e.pos[j], amount });
    }

    /// Indices of enemies whose body overlaps a circle, into `self.scratch`.
    fn query_circle(&mut self, c: Vec2, r: f32) {
        self.scratch.clear();
        let e = &self.enemies;
        let out = &mut self.scratch;
        self.grid.for_each_near(c, r + MAX_ENEMY_RADIUS, |j| {
            let rr = r + e.radius[j as usize];
            if e.hp[j as usize] > 0.0 && (e.pos[j as usize] - c).length_sq() <= rr * rr {
                out.push(j);
            }
        });
    }

    fn nearest_enemy(&self, radius: f32) -> Option<Vec2> {
        let p = self.player.pos;
        let mut best: Option<(f32, Vec2)> = None;
        self.grid.for_each_near(p, radius, |j| {
            let e = self.enemies.pos[j as usize];
            let d2 = (e - p).length_sq();
            if d2 <= radius * radius && best.is_none_or(|(b, _)| d2 < b) {
                best = Some((d2, e));
            }
        });
        best.map(|(_, e)| e)
    }

    fn remove_dead_enemies(&mut self) {
        self.dead.clear();
        self.dead.extend((0..self.enemies.len()).filter(|&i| self.enemies.hp[i] <= 0.0));
        // Descending order keeps swap_remove indices valid.
        for k in (0..self.dead.len()).rev() {
            let i = self.dead[k];
            let (pos, xp, kind) = (self.enemies.pos[i], self.enemies.xp[i], self.enemies.kind[i]);
            self.drop_gem(pos, xp);
            let tier = chest_tier(kind);
            if tier > 0 {
                self.chests.push((pos, tier));
            }
            self.player.special = (self.player.special + self.mods.special_rate / SPECIAL_KILLS).min(1.0);
            self.events.push(Event::Kill { pos, kind });
            self.kills_by_kind[kind.index()] += 1;
            if self.cfg.stress.is_none() {
                let coin = match kind {
                    EnemyKind::NightLord => 150,
                    EnemyKind::GreatOni => 50,
                    EnemyKind::Oni => 5,
                    _ if self.rng.f32() < 0.004 => 1,
                    _ => 0,
                };
                if coin > 0 {
                    self.coins.push((pos, coin));
                }
            }
            if kind == EnemyKind::GreatOni {
                self.great_oni_defeated = true;
            }
            if kind == EnemyKind::NightLord && self.state == RunState::Playing {
                self.state = RunState::Won;
            }
            self.enemies.swap_remove(i);
            self.kills += 1;
        }
    }

    // ───────────── Weapons ─────────────

    fn update_weapons(&mut self) {
        let (might, haste) = (self.might(), self.haste());
        for wi in 0..self.weapons.len() {
            let w = self.weapons[wi];
            let s = self.stats_of(&w);
            match w.kind {
                WeaponKind::Ofuda => {
                    let mut timer = w.timer - DT;
                    if timer <= 0.0 {
                        timer = s.cooldown * haste;
                        let dir = match self.nearest_enemy(360.0) {
                            Some(e) => (e - self.player.pos).normalize_or_zero(),
                            None => self.player.facing,
                        };
                        let base = dir.y.atan2(dir.x);
                        let n = s.amount as f32;
                        let homing = if w.awakened { 9.0 } else { 4.5 };
                        let spread = if w.awakened { 0.22 } else { 0.13 };
                        self.events.push(Event::Fired(WeaponKind::Ofuda));
                        for k in 0..s.amount {
                            let a = base + (k as f32 - (n - 1.0) / 2.0) * spread;
                            let life = 1.4 * self.persistence();
                            self.projectiles.push(self.player.pos, Vec2::from_angle(a) * s.speed, life, s.damage * might, s.pierce, homing);
                        }
                    }
                    self.weapons[wi].timer = timer;
                }
                WeaponKind::MoonOrbs => {
                    let phase = w.phase + s.speed * DT;
                    self.weapons[wi].phase = phase;
                    let r = self.orb_radius(w.level);
                    let rehit = (s.cooldown * 60.0) as u8;
                    let current = self.weapons[wi];
                    for c in self.orb_positions(&current) {
                        self.query_circle(c, r);
                        for idx in 0..self.scratch.len() {
                            let j = self.scratch[idx] as usize;
                            if self.enemies.orb_cd[j] == 0 {
                                self.enemies.orb_cd[j] = rehit;
                                let dir = self.enemies.pos[j] - self.player.pos;
                                self.hurt_enemy(j, s.damage * might, dir);
                            }
                        }
                    }
                }
                WeaponKind::SakuraField => {
                    let mut timer = w.timer - DT;
                    self.weapons[wi].phase = w.phase + DT;
                    if timer <= 0.0 {
                        timer = s.cooldown * haste;
                        self.weapons[wi].phase = 0.0;
                        let r = self.field_radius_of(&w);
                        self.query_circle(self.player.pos, r);
                        for idx in 0..self.scratch.len() {
                            let j = self.scratch[idx] as usize;
                            let dir = self.enemies.pos[j] - self.player.pos;
                            self.hurt_enemy(j, s.damage * might, dir * 0.3);
                        }
                        if w.awakened {
                            self.heal(1.0);
                        }
                    }
                    self.weapons[wi].timer = timer;
                }
                WeaponKind::ThunderChain
                | WeaponKind::Taiko
                | WeaponKind::StarFall
                | WeaponKind::Crescent
                | WeaponKind::FlamePool => {
                    let timer = w.timer - DT;
                    if timer <= 0.0 {
                        self.weapons[wi].timer = s.cooldown * haste;
                        self.fire_new_weapon(&w, &s, might);
                    } else {
                        self.weapons[wi].timer = timer;
                    }
                }
            }
        }

        // Stress load: radial spray to hold extra projectiles alive.
        if let Some(st) = self.cfg.stress {
            let alive = self.projectiles.len();
            if alive < st.projectiles {
                for _ in 0..(st.projectiles - alive).min(120) {
                    let a = self.rng.range(0.0, std::f32::consts::TAU);
                    let speed = self.rng.range(150.0, 320.0);
                    self.projectiles.push(self.player.pos, Vec2::from_angle(a) * speed, 2.0, 3.0, 2, 0.0);
                }
            }
        }
    }

    /// Nearest living enemy to `from` within `radius`, skipping ids in `exclude`.
    fn nearest_index(&self, from: Vec2, radius: f32, exclude: &[u32]) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        let e = &self.enemies;
        self.grid.for_each_near(from, radius, |j| {
            let j = j as usize;
            if e.hp[j] <= 0.0 || exclude.contains(&e.id[j]) {
                return;
            }
            let d2 = (e.pos[j] - from).length_sq();
            if d2 <= radius * radius && best.is_none_or(|(b, _)| d2 < b) {
                best = Some((d2, j));
            }
        });
        best.map(|(_, j)| j)
    }

    fn aim_dir(&self) -> Vec2 {
        match self.nearest_index(self.player.pos, 320.0, &[]) {
            Some(j) => (self.enemies.pos[j] - self.player.pos).normalize_or_zero(),
            None => self.player.facing,
        }
    }

    /// Up to `n` random living enemy positions within `radius` of the player.
    fn random_targets(&mut self, radius: f32, n: u32) -> Vec<Vec2> {
        self.query_circle(self.player.pos, radius);
        let mut out = Vec::with_capacity(n as usize);
        for _ in 0..n {
            if self.scratch.is_empty() {
                let a = self.rng.range(0.0, std::f32::consts::TAU);
                let r = self.rng.range(40.0, radius * 0.7);
                out.push(self.player.pos + Vec2::from_angle(a) * r);
            } else {
                let k = (self.rng.next_u32() as usize) % self.scratch.len();
                out.push(self.enemies.pos[self.scratch.swap_remove(k) as usize]);
            }
        }
        out
    }

    fn fire_new_weapon(&mut self, w: &OwnedWeapon, s: &WeaponStats, might: f32) {
        let area = s.area * self.area_mult();
        let dmg = s.damage * might;
        let p = self.player.pos;
        match w.kind {
            WeaponKind::ThunderChain => {
                let range = 110.0 * area;
                let mut hit: Vec<u32> = Vec::with_capacity(s.amount as usize);
                let mut from = p;
                let mut target = self.nearest_index(p, 300.0, &[]);
                for _ in 0..s.amount {
                    let Some(j) = target else { break };
                    let to = self.enemies.pos[j];
                    self.events.push(Event::Bolt { from, to });
                    hit.push(self.enemies.id[j]);
                    self.hurt_enemy(j, dmg, to - from);
                    from = to;
                    target = self.nearest_index(from, range, &hit);
                }
            }
            WeaponKind::Taiko => {
                self.events.push(Event::Fired(WeaponKind::Taiko));
                for n in 0..s.amount {
                    // Later waves start "behind" the first so they arrive as separate beats.
                    let start = -40.0 * n as f32;
                    self.waves.push(Wave { center: p, radius: start, max_radius: 135.0 * area, speed: s.speed, damage: dmg });
                }
            }
            WeaponKind::StarFall => {
                let r = 24.0 * area;
                for pos in self.random_targets(260.0, s.amount) {
                    let delay = 0.45 + self.rng.range(0.0, 0.15);
                    self.strikes.push(Strike { pos, delay, radius: r, damage: dmg });
                }
            }
            WeaponKind::Crescent => {
                let radius = 58.0 * area;
                let dir = self.aim_dir();
                let cone = 60f32.to_radians().cos();
                for n in 0..s.amount {
                    // 1st forward, 2nd behind, 3rd+ to the sides.
                    let turn = [0.0, std::f32::consts::PI, std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2][n.min(3) as usize];
                    let d = Vec2::from_angle(dir.y.atan2(dir.x) + turn);
                    self.query_circle(p, radius);
                    for idx in 0..self.scratch.len() {
                        let j = self.scratch[idx] as usize;
                        let to = (self.enemies.pos[j] - p).normalize_or_zero();
                        if w.awakened || to.x * d.x + to.y * d.y >= cone {
                            self.hurt_enemy(j, dmg, to);
                        }
                    }
                    self.events.push(Event::Slash { pos: p, dir: d, radius, full: w.awakened });
                    if w.awakened {
                        break; // the full circle already covers every direction
                    }
                }
            }
            WeaponKind::FlamePool => {
                self.events.push(Event::Fired(WeaponKind::FlamePool));
                let r = 26.0 * area;
                for pos in self.random_targets(220.0, s.amount) {
                    self.pools.push(Pool { pos, radius: r, life: s.duration, max_life: s.duration, tick: 0.0, damage: dmg });
                }
            }
            _ => {}
        }
    }

    /// Advances shockwaves, pending strikes and burning pools.
    fn update_effects(&mut self) {
        let mut i = 0;
        while i < self.waves.len() {
            let mut wv = self.waves[i];
            wv.radius += wv.speed * DT;
            if wv.radius > wv.max_radius {
                self.waves.swap_remove(i);
                continue;
            }
            self.waves[i] = wv;
            if wv.radius > 0.0 {
                self.query_circle(wv.center, wv.radius + 6.0);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    let off = self.enemies.pos[j] - wv.center;
                    let band = wv.radius - 12.0 - self.enemies.radius[j];
                    if self.enemies.wave_cd[j] == 0 && off.length_sq() >= band.max(0.0) * band.max(0.0) {
                        self.enemies.wave_cd[j] = 20;
                        self.hurt_enemy(j, wv.damage, off * 2.0);
                    }
                }
            }
            i += 1;
        }

        let mut i = 0;
        while i < self.strikes.len() {
            self.strikes[i].delay -= DT;
            if self.strikes[i].delay > 0.0 {
                i += 1;
                continue;
            }
            let st = self.strikes.swap_remove(i);
            self.query_circle(st.pos, st.radius);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let dir = self.enemies.pos[j] - st.pos;
                self.hurt_enemy(j, st.damage, dir);
            }
            self.events.push(Event::Impact { pos: st.pos, radius: st.radius });
        }

        let mut i = 0;
        while i < self.pools.len() {
            let mut pl = self.pools[i];
            pl.life -= DT;
            pl.tick -= DT;
            if pl.life <= 0.0 {
                self.pools.swap_remove(i);
                continue;
            }
            if pl.tick <= 0.0 {
                pl.tick = 0.3;
                self.query_circle(pl.pos, pl.radius);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    self.hurt_enemy(j, pl.damage, Vec2::ZERO);
                }
            }
            self.pools[i] = pl;
            i += 1;
        }
    }

    fn move_projectiles_and_hit(&mut self) {
        let mut i = 0;
        while i < self.projectiles.len() {
            let turn = self.projectiles.homing[i];
            if turn > 0.0 {
                let here = self.projectiles.pos[i];
                let skip = self.projectiles.last_hit[i];
                let mut best: Option<(f32, Vec2)> = None;
                let e = &self.enemies;
                self.grid.for_each_near(here, 130.0, |j| {
                    let j = j as usize;
                    if e.id[j] == skip || e.hp[j] <= 0.0 {
                        return;
                    }
                    let d2 = (e.pos[j] - here).length_sq();
                    if d2 < 130.0 * 130.0 && best.is_none_or(|(b, _)| d2 < b) {
                        best = Some((d2, e.pos[j]));
                    }
                });
                if let Some((_, target)) = best {
                    let v = self.projectiles.vel[i];
                    let (cur, want) = (v.y.atan2(v.x), (target - here).y.atan2((target - here).x));
                    let mut diff = want - cur;
                    while diff > std::f32::consts::PI {
                        diff -= std::f32::consts::TAU;
                    }
                    while diff < -std::f32::consts::PI {
                        diff += std::f32::consts::TAU;
                    }
                    let a = cur + diff.clamp(-turn * DT, turn * DT);
                    self.projectiles.vel[i] = Vec2::from_angle(a) * v.length();
                }
            }
            let p = self.projectiles.pos[i] + self.projectiles.vel[i] * DT;
            self.projectiles.pos[i] = p;
            self.projectiles.life[i] -= DT;
            self.query_circle(p, PROJECTILE_RADIUS);
            let mut pierce = self.projectiles.pierce[i];
            for idx in 0..self.scratch.len() {
                if pierce == 0 {
                    break;
                }
                let j = self.scratch[idx] as usize;
                if self.enemies.id[j] == self.projectiles.last_hit[i] || self.enemies.hp[j] <= 0.0 {
                    continue;
                }
                self.projectiles.last_hit[i] = self.enemies.id[j];
                let (dmg, vel) = (self.projectiles.damage[i], self.projectiles.vel[i]);
                self.hurt_enemy(j, dmg, vel);
                pierce -= 1;
            }
            self.projectiles.pierce[i] = pierce;
            if pierce == 0 || self.projectiles.life[i] <= 0.0 {
                self.projectiles.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    // ───────────── Pickups & levelling ─────────────

    fn drop_gem(&mut self, pos: Vec2, value: u32) {
        if self.gems.pos.len() >= GEM_MERGE_AT {
            self.gems.merge(GEM_MERGE_TO);
        }
        self.gems.pos.push(pos);
        self.gems.value.push(value);
    }

    fn heal(&mut self, amount: f32) {
        self.player.hp = (self.player.hp + amount * self.mods.healing).min(self.player.max_hp);
    }

    fn gain_xp(&mut self, amount: u32) {
        self.xp_carry += amount as f32 * self.mods.xp * (1.0 + self.bonus.growth);
        let whole = self.xp_carry.floor();
        self.xp_carry -= whole;
        self.player.xp += whole as u32;
        while self.player.xp >= self.xp_to_next() {
            self.player.xp -= self.xp_to_next();
            self.player.level += 1;
            self.pending_levels += 1;
            self.events.push(Event::LevelUp);
        }
    }

    fn collect_gems(&mut self) {
        let p = self.player.pos;
        let magnet = self.magnet();
        let grab2 = (PLAYER_RADIUS * 2.0) * (PLAYER_RADIUS * 2.0);
        let mut gained = 0;
        let mut i = 0;
        while i < self.gems.pos.len() {
            let d = p - self.gems.pos[i];
            let d2 = d.length_sq();
            if d2 < grab2 {
                gained += self.gems.value[i];
                self.gems.pos.swap_remove(i);
                self.gems.value.swap_remove(i);
                continue;
            }
            if d2 < magnet * magnet {
                self.gems.pos[i] += d.normalize_or_zero() * (280.0 * DT);
            }
            i += 1;
        }
        if gained > 0 {
            self.gain_xp(gained);
            self.events.push(Event::GemsPicked);
        }
        let mut sen = 0;
        self.coins.retain_mut(|(c, v)| {
            let d = p - *c;
            let d2 = d.length_sq();
            if d2 < grab2 {
                sen += *v;
                return false;
            }
            if d2 < magnet * magnet {
                *c += d.normalize_or_zero() * (280.0 * DT);
            }
            true
        });
        if sen > 0 {
            self.gain_sen(sen);
        }
    }

    fn gain_sen(&mut self, amount: u32) {
        let amount = (amount as f32 * (1.0 + self.bonus.greed)).round() as u32;
        self.sen += amount;
        self.events.push(Event::SenPicked { amount });
    }

    fn update_shots(&mut self) {
        let p = self.player.pos;
        let hit_r = PLAYER_RADIUS + SHOT_RADIUS;
        let mut hit = false;
        self.shots.retain_mut(|s| {
            s.pos += s.vel * DT;
            s.life -= DT;
            if (s.pos - p).length_sq() <= hit_r * hit_r {
                hit = true;
                return false;
            }
            s.life > 0.0
        });
        if hit && self.cfg.stress.is_none() && self.player.iframes <= 0.0 {
            self.hurt_player(SHOT_DAMAGE * damage_scale(self.time));
        }
    }

    fn collect_chests(&mut self) {
        let p = self.player.pos;
        let grab = PLAYER_RADIUS + 10.0;
        if let Some(i) = self.chests.iter().position(|(c, _)| (*c - p).length_sq() < grab * grab) {
            let (_, tier) = self.chests.swap_remove(i);
            self.open_chest(tier);
        }
    }

    /// Weapons that could Awaken right now.
    pub fn awakenable(&self) -> Vec<WeaponKind> {
        if self.time < AWAKEN_FROM {
            return Vec::new();
        }
        self.weapons
            .iter()
            .filter(|w| {
                let (p, need) = complement(w.kind);
                !w.awakened && w.level >= MAX_LEVEL && self.passive_level(p) >= need
            })
            .map(|w| w.kind)
            .collect()
    }

    /// Rolls the reward count (85% 1 / 10% 2 / 4% 3 / 1% 5, at least `min_rewards`), then fills it:
    /// an Awakening first if one is eligible, then levels for owned items.
    pub fn open_chest(&mut self, min_rewards: u8) {
        let tier = chest_roll(self.rng.f32()).max(min_rewards);
        let sen = 10 * tier as u32 + self.rng.range(0.0, 10.0) as u32;
        self.gain_sen(sen);
        let mut items = Vec::new();
        if let Some(&k) = self.awakenable().first() {
            if let Some(w) = self.weapons.iter_mut().find(|w| w.kind == k) {
                w.awakened = true;
            }
            self.events.push(Event::Awakened(k));
            items.push(ChestItem::Awakening(k));
        }
        while items.len() < tier as usize {
            let mut pool: Vec<Upgrade> = Vec::new();
            pool.extend(self.weapons.iter().filter(|w| w.level < MAX_LEVEL).map(|w| Upgrade::Weapon(w.kind)));
            pool.extend(self.passives.iter().filter(|(k, l)| *l < passive_max(*k)).map(|(k, _)| Upgrade::Passive(*k)));
            if pool.is_empty() {
                self.heal(30.0);
                items.push(ChestItem::Heal);
                continue;
            }
            let u = pool[(self.rng.next_u32() as usize) % pool.len()];
            self.apply_upgrade(u);
            items.push(ChestItem::LevelUp(u, self.next_level(u) - 1));
        }
        self.chest_result = Some(items);
    }

    pub fn close_chest(&mut self) {
        self.chest_result = None;
    }

    fn use_special(&mut self) {
        if self.player.special < 1.0 {
            return;
        }
        self.player.special = 0.0;
        self.player.iframes = BARRIER_IFRAMES;
        let p = self.player.pos;
        self.query_circle(p, BARRIER_RADIUS);
        let dmg = BARRIER_DAMAGE * self.might();
        for idx in 0..self.scratch.len() {
            let j = self.scratch[idx] as usize;
            let dir = self.enemies.pos[j] - p;
            self.hurt_enemy(j, dmg, dir);
            self.enemies.push[j] += dir.normalize_or_zero() * (320.0 * self.enemies.knock[j]);
        }
        let r2 = BARRIER_RADIUS * BARRIER_RADIUS;
        self.shots.retain(|s| (s.pos - p).length_sq() > r2);
        self.events.push(Event::SpecialUsed { pos: p });
    }

    fn collect_shrine(&mut self) {
        let Some(s) = self.shrine else { return };
        let r = PLAYER_RADIUS + 14.0;
        if (s - self.player.pos).length_sq() > r * r {
            return;
        }
        self.shrine = None;
        let mut pool = ALL_SHRINE_EVENTS.to_vec();
        let mut offer = Vec::new();
        for _ in 0..2 {
            let i = (self.rng.next_u32() as usize) % pool.len();
            offer.push(pool.swap_remove(i));
        }
        self.shrine_offer = Some(offer);
    }

    /// Accept shrine choice `i`, or walk away with `None`.
    pub fn choose_shrine(&mut self, choice: Option<usize>) {
        let Some(offer) = self.shrine_offer.take() else { return };
        let Some(&ev) = choice.and_then(|i| offer.get(i)) else { return };
        self.shrines_taken.push(ev);
        let m = &mut self.mods;
        match ev {
            ShrineEvent::Fury => {
                m.damage *= 1.3;
                m.curse *= 1.25;
            }
            ShrineEvent::GlassBlade => {
                m.damage *= 1.5;
                self.player.max_hp *= 0.7;
            }
            ShrineEvent::Feast => {
                self.player.hp = self.player.max_hp;
                match (0..self.passives.len()).min_by_key(|&i| self.passives[i].1) {
                    Some(i) if self.passives[i].1 > 1 => self.passives[i].1 -= 1,
                    Some(i) => {
                        self.passives.remove(i);
                    }
                    None => self.player.max_hp -= 10.0,
                }
            }
            ShrineEvent::Fortune => {
                self.player.max_hp -= 15.0;
                self.open_chest(1);
            }
            ShrineEvent::Patience => {
                m.enemy_hp *= 1.15;
                let open: Vec<usize> = (0..self.weapons.len()).filter(|&i| self.weapons[i].level < MAX_LEVEL).collect();
                if !open.is_empty() {
                    let i = open[(self.rng.next_u32() as usize) % open.len()];
                    self.weapons[i].level = (self.weapons[i].level + 2).min(MAX_LEVEL);
                }
            }
            ShrineEvent::Harvest => {
                m.xp *= 1.4;
                m.enemy_speed *= 1.1;
            }
            ShrineEvent::IronSkin => {
                m.armor += 3.0;
                m.move_speed *= 0.85;
            }
            ShrineEvent::SecondWind => {
                self.player.special = 1.0;
                m.special_rate *= 1.3;
                self.player.hp = (self.player.hp - 25.0).max(1.0);
            }
            ShrineEvent::Tailwind => {
                m.move_speed *= 1.2;
                m.magnet *= 0.75;
            }
            ShrineEvent::Offering => {
                m.healing *= 0.5;
                self.player.level += 3;
                self.pending_levels += 3;
                if self.offer.is_none() {
                    self.make_offer();
                }
            }
        }
        self.player.max_hp = self.player.max_hp.max(10.0);
        self.player.hp = self.player.hp.min(self.player.max_hp);
        self.events.push(Event::ShrineChosen(ev));
    }

    fn make_offer(&mut self) {
        let mut pool: Vec<Upgrade> = Vec::new();
        for w in &self.weapons {
            if w.level < MAX_LEVEL {
                pool.push(Upgrade::Weapon(w.kind));
            }
        }
        let hero = hero_def(self.cfg.hero);
        if self.weapons.len() < hero.weapon_slots {
            pool.extend(hero.weapon_pool().into_iter().filter(|k| self.weapon_level(*k) == 0).map(Upgrade::Weapon));
        }
        for (k, l) in &self.passives {
            if *l < passive_max(*k) {
                pool.push(Upgrade::Passive(*k));
            }
        }
        if self.passives.len() < hero.passive_slots {
            pool.extend(ALL_PASSIVES.iter().filter(|k| self.passive_level(**k) == 0).map(|k| Upgrade::Passive(*k)));
        }
        let mut offer = Vec::new();
        while offer.len() < 3 && !pool.is_empty() {
            let i = (self.rng.next_u32() as usize) % pool.len();
            offer.push(pool.swap_remove(i));
        }
        if offer.is_empty() {
            offer.push(Upgrade::Heal);
        }
        self.offer = Some(offer);
    }

    /// Level the upgrade would reach if picked (for UI).
    pub fn next_level(&self, u: Upgrade) -> u8 {
        match u {
            Upgrade::Weapon(k) => self.weapon_level(k) + 1,
            Upgrade::Passive(k) => self.passive_level(k) + 1,
            Upgrade::Heal => 1,
        }
    }

    /// Apply choice `i` of the current offer.
    pub fn choose(&mut self, i: usize) {
        let Some(offer) = self.offer.take() else { return };
        let Some(&u) = offer.get(i) else {
            self.offer = Some(offer);
            return;
        };
        self.apply_upgrade(u);
        self.pending_levels = self.pending_levels.saturating_sub(1);
        if self.pending_levels > 0 {
            self.make_offer();
        }
    }

    fn apply_upgrade(&mut self, u: Upgrade) {
        match u {
            Upgrade::Weapon(k) => match self.weapons.iter_mut().find(|w| w.kind == k) {
                Some(w) => w.level += 1,
                None => self.weapons.push(OwnedWeapon { kind: k, level: 1, awakened: false, timer: 0.0, phase: 0.0 }),
            },
            Upgrade::Passive(k) => {
                match self.passives.iter_mut().find(|(pk, _)| *pk == k) {
                    Some((_, l)) => *l += 1,
                    None => self.passives.push((k, 1)),
                }
                if k == PassiveKind::Vitality {
                    self.player.max_hp += 20.0;
                    self.heal(20.0);
                }
            }
            Upgrade::Heal => self.heal(30.0),
        }
    }

    /// Dev shortcut: fill the hero's weapon slots at max level with their complements, ready to Awaken.
    pub fn debug_max_loadout(&mut self) {
        let hero = hero_def(self.cfg.hero);
        for k in hero.weapon_pool().into_iter().take(hero.weapon_slots) {
            match self.weapons.iter_mut().find(|w| w.kind == k) {
                Some(w) => w.level = MAX_LEVEL,
                None => self.weapons.push(OwnedWeapon { kind: k, level: MAX_LEVEL, awakened: false, timer: 0.0, phase: 0.0 }),
            }
            let (c, need) = complement(k);
            match self.passives.iter_mut().find(|(pk, _)| *pk == c) {
                Some((_, l)) => *l = (*l).max(need),
                None => self.passives.push((c, need)),
            }
        }
    }

    /// Dev shortcut: every library weapon at level 5, ignoring the hero's pool and slots.
    pub fn debug_all_weapons(&mut self) {
        for k in ALL_WEAPONS {
            if self.weapon_level(k) == 0 {
                self.weapons.push(OwnedWeapon { kind: k, level: 5, awakened: false, timer: 0.0, phase: 0.0 });
            }
        }
    }

    /// Cheap fingerprint of the world state, for determinism tests.
    pub fn state_hash(&self) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        let mut mix = |v: u32| {
            h ^= v as u64;
            h = h.wrapping_mul(0x100000001b3);
        };
        mix(self.player.pos.x.to_bits());
        mix(self.player.pos.y.to_bits());
        mix(self.player.hp.to_bits());
        for p in &self.enemies.pos {
            mix(p.x.to_bits());
            mix(p.y.to_bits());
        }
        mix(self.projectiles.len() as u32);
        mix(self.kills as u32);
        mix(self.player.level);
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays a run with a circling player that always takes the first offer.
    fn play(seed: u64, ticks: u32) -> World {
        let mut w = World::new(WorldConfig { seed, ..Default::default() });
        for t in 0..ticks {
            if w.offer.is_some() {
                w.choose(0);
            }
            w.close_chest();
            w.choose_shrine(None);
            let a = t as f32 * 0.01;
            w.step(Input { move_dir: Vec2::new(a.cos(), a.sin()), ..Default::default() });
        }
        w
    }

    #[test]
    fn deterministic_for_same_seed() {
        assert_eq!(play(42, 3000).state_hash(), play(42, 3000).state_hash());
        assert_ne!(play(42, 3000).state_hash(), play(43, 3000).state_hash());
    }

    #[test]
    fn kiting_bot_levels_up_and_gains_upgrades() {
        let mut w = World::new(WorldConfig { seed: 1, ..Default::default() });
        for _ in 0..60 * 90 {
            if w.offer.is_some() {
                w.choose(0);
            }
            w.close_chest();
            w.choose_shrine(None);
            let input = w.bot_input();
            w.step(input);
        }
        assert_eq!(w.state, RunState::Playing, "bot died at {:.0}s", w.time);
        assert!(w.kills > 20, "kills {}", w.kills);
        let left: u32 = w.gems.value.iter().sum();
        assert!(w.player.level >= 5, "level {} kills {} xp left on field {}", w.player.level, w.kills, left);
        assert!(w.weapons.len() + w.passives.len() >= 3);
    }

    #[test]
    fn standing_still_eventually_dies() {
        let mut w = World::new(WorldConfig::default());
        for _ in 0..60 * 600 {
            if w.offer.is_some() {
                w.choose(0);
            }
            w.close_chest();
            w.choose_shrine(None);
            w.step(Input::default());
            if w.state == RunState::Dead {
                return;
            }
        }
        panic!("player survived 10 min standing still: hp {}", w.player.hp);
    }

    #[test]
    fn sim_pauses_during_offer() {
        let mut w = World::new(WorldConfig { seed: 5, ..Default::default() });
        for _ in 0..60 * 120 {
            if w.offer.is_some() {
                break;
            }
            let input = w.bot_input();
            w.step(input);
        }
        assert!(w.offer.is_some(), "no offer within 2 min (state {:?})", w.state);
        let t = w.time;
        w.step(Input::default());
        assert_eq!(w.time, t);
        w.choose(0);
    }

    #[test]
    fn stress_mode_holds_targets() {
        let mut w = World::new(WorldConfig { stress: Some(Stress { enemies: 800, projectiles: 500 }), ..Default::default() });
        for _ in 0..600 {
            w.step(Input::default());
        }
        assert!(w.enemies.len() >= 700 && w.offer.is_none() && w.state == RunState::Playing);
    }

    #[test]
    fn barrier_needs_full_meter_and_hits_nearby() {
        let mut w = play(9, 60 * 20);
        w.player.special = 0.5;
        w.use_special();
        assert!(w.player.iframes <= 1.0, "barrier must not fire on a half meter");
        w.player.special = 1.0;
        w.use_special();
        assert!(w.player.iframes >= BARRIER_IFRAMES - 1e-3 && w.player.special == 0.0);
        assert!(w.events.iter().any(|e| matches!(e, Event::Hit { .. })), "nothing was in range");
    }

    #[test]
    fn chest_awakens_eligible_weapon_only_after_eight_minutes() {
        let mut w = World::new(WorldConfig::default());
        w.weapons[0].level = MAX_LEVEL;
        w.passives.push((complement(WeaponKind::Ofuda).0, complement(WeaponKind::Ofuda).1));
        w.time = AWAKEN_FROM - 1.0;
        w.open_chest(1);
        assert!(!w.weapons[0].awakened, "too early to awaken");
        w.close_chest();
        w.time = AWAKEN_FROM;
        w.open_chest(3);
        assert!(w.weapons[0].awakened);
        let res = w.chest_result.clone().unwrap();
        assert_eq!(res[0], ChestItem::Awakening(WeaponKind::Ofuda));
        assert!(res.len() >= 3, "boss-size chest gives at least 3, got {}", res.len());
        assert!(w.paused());
        w.close_chest();
        assert!(!w.paused());
        let s = w.stats_of(&w.weapons[0]);
        assert_eq!(s.amount, weapon_stats(WeaponKind::Ofuda, MAX_LEVEL).amount * 2);
    }

    #[test]
    fn great_oni_spawns_at_eight_minutes_and_drops_a_big_chest() {
        let mut w = World::new(WorldConfig::default());
        w.time = MIDBOSS_AT - 0.5;
        w.player.iframes = 1e9;
        for _ in 0..60 {
            w.step(Input::default());
        }
        let (i, frac, _) = w.boss().expect("boss should be alive");
        assert!(frac > 0.9);
        w.enemies.hp[i] = -1.0;
        w.step(Input::default());
        assert!(w.boss().is_none());
        assert!(w.chests.iter().any(|(_, t)| *t == 3));
    }

    /// A ring of stationary dummies around the player, then fire one weapon for a few seconds.
    fn damage_dealt_by(kind: WeaponKind) -> f32 {
        let mut w = World::new(WorldConfig { seed: 3, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        // Library check: test every weapon even if no current hero can roll it.
        w.weapons = vec![OwnedWeapon { kind, level: 4, awakened: false, timer: 0.0, phase: 0.0 }];
        for k in 0..40 {
            let a = k as f32 / 40.0 * std::f32::consts::TAU;
            let r = 30.0 + (k % 4) as f32 * 25.0;
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(a) * r);
        }
        let n = w.enemies.len();
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        let mut dealt = 0.0;
        for _ in 0..60 * 4 {
            w.step(Input::default());
            dealt += w.events.iter().filter_map(|e| if let Event::Hit { amount, .. } = e { Some(*amount) } else { None }).sum::<f32>();
        }
        assert_eq!(w.enemies.len(), n, "dummies shouldn't die in 4s");
        dealt
    }

    #[test]
    fn every_weapon_deals_damage() {
        for k in ALL_WEAPONS {
            let d = damage_dealt_by(k);
            assert!(d > 50.0, "{k:?} dealt only {d}");
        }
    }

    #[test]
    fn chests_respect_minimum_and_roll_bigger_sometimes() {
        let mut w = World::new(WorldConfig { seed: 11, ..Default::default() });
        let mut sizes = [0u32; 6];
        for _ in 0..2000 {
            w.open_chest(1);
            sizes[w.chest_result.take().unwrap().len()] += 1;
        }
        assert!(sizes[1] > 1500 && sizes[2] > 100 && sizes[3] > 30 && sizes[5] > 0, "{sizes:?}");
        for _ in 0..200 {
            w.open_chest(3);
            assert!(w.chest_result.take().unwrap().len() >= 3);
        }
    }

    #[test]
    fn capped_passives_stop_being_offered() {
        let mut w = World::new(WorldConfig::default());
        w.passives.push((PassiveKind::Abundance, passive_max(PassiveKind::Abundance)));
        for _ in 0..200 {
            w.make_offer();
            assert!(!w.offer.take().unwrap().contains(&Upgrade::Passive(PassiveKind::Abundance)));
        }
    }

    #[test]
    fn offers_respect_hero_pool_and_weapon_slots() {
        let ren = hero_def(HeroId::Ren);
        let pool = ren.weapon_pool();
        assert!(pool.contains(&WeaponKind::Ofuda) && pool.contains(&WeaponKind::Taiko));
        assert!(!pool.contains(&WeaponKind::ThunderChain) && !pool.contains(&WeaponKind::Crescent));

        let mut w = World::new(WorldConfig::default());
        for _ in 0..300 {
            w.make_offer();
            for u in w.offer.take().unwrap() {
                if let Upgrade::Weapon(k) = u {
                    assert!(pool.contains(&k), "offered {k:?} outside Ren's pool");
                }
            }
        }
        // With every slot filled, no new weapon may be offered.
        w.weapons = pool.iter().take(ren.weapon_slots).map(|&k| OwnedWeapon { kind: k, level: 2, awakened: false, timer: 0.0, phase: 0.0 }).collect();
        for _ in 0..300 {
            w.make_offer();
            for u in w.offer.take().unwrap() {
                if let Upgrade::Weapon(k) = u {
                    assert!(w.weapon_level(k) > 0, "offered a 5th weapon {k:?}");
                }
            }
        }
    }

    #[test]
    fn awakening_needs_the_complement_level() {
        let mut w = World::new(WorldConfig::default());
        w.time = AWAKEN_FROM;
        w.weapons[0].level = MAX_LEVEL;
        assert!(w.awakenable().is_empty(), "no complement owned yet");
        let (p, need) = complement(WeaponKind::Ofuda);
        w.passives.push((p, need));
        assert_eq!(w.awakenable(), vec![WeaponKind::Ofuda]);
    }

    #[test]
    fn far_kills_still_drop_gems_and_merging_keeps_all_xp() {
        let mut w = World::new(WorldConfig::default());
        // Fill the field far from the player, past the merge threshold.
        for k in 0..(GEM_MERGE_AT as u32 + 300) {
            let p = Vec2::new(600.0 + (k % 40) as f32 * 9.0, -300.0 + (k / 40) as f32 * 9.0);
            w.drop_gem(p, 1);
        }
        let total: u32 = w.gems.value.iter().sum();
        assert_eq!(total, GEM_MERGE_AT as u32 + 300, "merging must not lose XP");
        assert!(w.gems.pos.len() <= GEM_MERGE_AT);
        assert!(w.gems.value.iter().any(|v| *v > 1), "some gems should have merged up");
        // A kill far away, with the field crowded, still leaves a gem right there.
        let far = Vec2::new(-1500.0, 900.0);
        w.drop_gem(far, 3);
        assert!(w.gems.pos.iter().zip(&w.gems.value).any(|(p, v)| (*p - far).length() < 1.0 && *v == 3));
        // Merged gems stay near where their XP dropped.
        for p in &w.gems.pos {
            assert!(p.x > 400.0 || (*p - far).length() < 1.0, "gem drifted to {p:?}");
        }
    }

    #[test]
    fn shrine_appears_offers_two_choices_and_applies_blessing_and_price() {
        let mut w = World::new(WorldConfig::default());
        w.time = SHRINE_TIMES[0] - 0.01;
        w.player.iframes = 1e9;
        w.step(Input::default());
        let s = w.shrine.expect("shrine should appear at 3:00");
        w.player.pos = s;
        w.step(Input::default());
        let offer = w.shrine_offer.clone().expect("touching the shrine opens the offer");
        assert_eq!(offer.len(), 2);
        assert_ne!(offer[0], offer[1]);
        assert!(w.paused());
        w.shrine_offer = Some(vec![ShrineEvent::Fury, ShrineEvent::Tailwind]);
        w.choose_shrine(Some(0));
        assert!((w.mods.damage - 1.3).abs() < 1e-6 && (w.mods.curse - 1.25).abs() < 1e-6);
        assert!(!w.paused());
    }

    #[test]
    fn walking_away_from_a_shrine_changes_nothing() {
        let mut w = World::new(WorldConfig::default());
        w.shrine_offer = Some(vec![ShrineEvent::GlassBlade, ShrineEvent::Offering]);
        let hp = w.player.max_hp;
        w.choose_shrine(None);
        assert!(w.shrine_offer.is_none());
        assert_eq!(w.player.max_hp, hp);
        assert_eq!(w.mods.damage, 1.0);
    }

    #[test]
    fn every_shrine_event_applies_without_breaking_the_run() {
        for ev in ALL_SHRINE_EVENTS {
            let mut w = play(4, 60 * 30);
            w.shrine_offer = Some(vec![ev]);
            w.choose_shrine(Some(0));
            assert!(w.player.hp > 0.0 && w.player.hp <= w.player.max_hp, "{ev:?} broke HP");
            w.close_chest();
            while w.offer.is_some() {
                w.choose(0);
            }
            for _ in 0..120 {
                w.step(Input::default());
            }
        }
    }

    #[test]
    fn chochin_fires_and_fireballs_hurt() {
        let mut w = World::new(WorldConfig { seed: 8, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.cfg.stress = None;
        w.weapons.clear();
        w.time = 1.0;
        w.spawn_enemy(EnemyKind::Chochin, Vec2::new(160.0, 0.0));
        let hp = w.player.hp;
        let mut fired = false;
        for _ in 0..60 * 8 {
            w.step(Input::default());
            fired |= !w.shots.is_empty();
            w.next_elite = f32::MAX; // keep the test arena to the one lantern
        }
        assert!(fired, "chochin never fired");
        assert!(w.player.hp < hp, "fireballs should hurt");
    }

    #[test]
    fn barrier_erases_fireballs() {
        let mut w = World::new(WorldConfig::default());
        w.shots.push(Shot { pos: Vec2::new(50.0, 0.0), vel: Vec2::ZERO, life: 5.0 });
        w.shots.push(Shot { pos: Vec2::new(900.0, 0.0), vel: Vec2::ZERO, life: 5.0 });
        w.player.special = 1.0;
        w.use_special();
        assert_eq!(w.shots.len(), 1, "only the far fireball survives");
    }

    #[test]
    fn nopperabo_reappears_beside_the_player() {
        let mut w = World::new(WorldConfig::default());
        w.player.iframes = 1e9;
        w.spawn_enemy(EnemyKind::Nopperabo, Vec2::new(400.0, 0.0));
        let mut blinked = false;
        for _ in 0..60 * 6 {
            w.step(Input::default());
            if w.events.iter().any(|e| matches!(e, Event::Blink { .. })) {
                blinked = true;
                break;
            }
        }
        assert!(blinked);
        let i = w.enemies.kind.iter().position(|k| *k == EnemyKind::Nopperabo).unwrap();
        assert!((w.enemies.pos[i] - w.player.pos).length() < 160.0);
    }

    #[test]
    fn night_lord_ends_the_night() {
        let mut w = World::new(WorldConfig::default());
        w.time = RUN_SECONDS - 0.01;
        w.player.iframes = 1e9;
        w.step(Input::default());
        let (i, _, kind) = w.boss().expect("Night Lord at 15:00");
        assert_eq!(kind, EnemyKind::NightLord);
        assert_eq!(w.state, RunState::Playing, "surviving to 15:00 no longer wins by itself");
        w.enemies.hp[i] = -1.0;
        w.step(Input::default());
        assert_eq!(w.state, RunState::Won);

        let mut w = World::new(WorldConfig::default());
        w.time = DAWN_LIMIT - 0.01;
        w.night_lord_spawned = true;
        w.player.iframes = 1e9;
        w.step(Input::default());
        assert_eq!(w.state, RunState::Dead);
        assert!(w.dawn_out);
    }

    #[test]
    fn player_cannot_walk_through_props() {
        // Find a stall and walk straight at it from the left.
        let (prop, _) = (-30..30)
            .flat_map(|cy| (-30..30).map(move |cx| (cx, cy)))
            .find_map(|(cx, cy)| {
                let (ps, n) = crate::stage::chunk_props(cx, cy);
                ps[..n].iter().find(|p| p.kind == crate::stage::PropKind::Stall).map(|p| (*p, ()))
            })
            .expect("a stall");
        let (min, max) = prop.footprint();
        let mut w = World::new(WorldConfig { stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.player.pos = Vec2::new(min.x - 20.0, (min.y + max.y) / 2.0);
        for _ in 0..120 {
            w.step(Input { move_dir: Vec2::new(1.0, 0.0), ..Default::default() });
        }
        assert!(w.player.pos.x <= min.x - PLAYER_RADIUS + 0.5, "walked into the stall: {:?}", w.player.pos);
    }

    #[test]
    fn revival_saves_one_death() {
        let mut w = World::new(WorldConfig { meta: Bonus { revivals: 1, ..Default::default() }, ..Default::default() });
        w.hurt_player(1e6);
        assert_eq!(w.state, RunState::Playing);
        assert!(w.player.hp > 0.0);
        w.player.iframes = 0.0;
        w.hurt_player(1e6);
        assert_eq!(w.state, RunState::Dead);
    }

    #[test]
    fn coins_are_collected_with_greed() {
        let mut w = World::new(WorldConfig { meta: Bonus { greed: 0.5, ..Default::default() }, ..Default::default() });
        w.coins.push((w.player.pos, 10));
        w.collect_gems();
        assert_eq!(w.sen, 15);
        assert!(w.coins.is_empty());
    }

    #[test]
    fn heroes_start_with_their_signature_and_innate() {
        for h in ALL_HEROES {
            let d = hero_def(h);
            let w = World::new(WorldConfig { hero: h, ..Default::default() });
            assert_eq!(w.weapons[0].kind, d.signature);
            assert!(d.weapon_pool().len() >= 4, "{:?} pool too small", h);
        }
        let w = World::new(WorldConfig { hero: HeroId::Kaede, ..Default::default() });
        assert!(w.move_speed() > World::new(WorldConfig::default()).move_speed());
    }

    /// Full 15-minute bot run; prints how far it got. `cargo test -p sim balance -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn balance_full_run() {
        let seeds: u64 = std::env::var("BALANCE_SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
        for seed in 1..=seeds {
            let started = std::time::Instant::now();
            let mut w = World::new(WorldConfig { seed, ..Default::default() });
            while w.state == RunState::Playing {
                if w.offer.is_some() {
                    w.choose(0);
                }
                if w.chest_result.is_some() {
                    w.close_chest();
                }
                if w.shrine_offer.is_some() {
                    w.choose_shrine(Some(0));
                }
                let input = w.bot_input();
                w.step(input);
            }
            let awakened: Vec<_> = w.weapons.iter().filter(|x| x.awakened).map(|x| awakened_name(x.kind)).collect();
            let nl = w.boss().filter(|b| b.2 == EnemyKind::NightLord).map(|b| format!("{:.0}%", b.1 * 100.0)).unwrap_or("-".into());
            println!(
                "seed {seed} [NightLord hp {nl}]: {:?} at {:>5.0}s  level {:>2}  kills {:>6}  ({:.1}s real)  weapons {:?}  awakened {:?}",
                w.state,
                w.time,
                w.player.level,
                w.kills,
                started.elapsed().as_secs_f32(),
                w.weapons.iter().map(|x| (x.kind, x.level)).collect::<Vec<_>>(),
                awakened
            );
        }
    }
}
