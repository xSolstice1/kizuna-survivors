//! The run simulation: player, weapons, spawn director, enemies, pickups.

use crate::content::*;
use crate::grid::Grid;
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
}

impl Default for WorldConfig {
    fn default() -> Self {
        WorldConfig { seed: 1, hero: HeroId::Ren, stress: None, spawn_radius: 420.0, recycle_radius: 900.0 }
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
        let signature = hero_def(cfg.hero).signature;
        let cell = 28.0;
        let dim = ((cfg.recycle_radius * 2.2) / cell).ceil() as usize;
        World {
            cfg,
            player: Player {
                pos: Vec2::ZERO,
                facing: Vec2::new(1.0, 0.0),
                moving: false,
                hp: 100.0,
                max_hp: 100.0,
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
        self.state != RunState::Playing || self.offer.is_some() || self.chest_result.is_some()
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
        // Everything that reads the grid must run before this: removal reorders
        // the enemy arrays and invalidates the grid's indices.
        self.remove_dead_enemies();
        self.collect_gems();
        self.collect_chests();
        if self.state == RunState::Playing && self.time >= RUN_SECONDS {
            self.state = RunState::Won;
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
        1.0 + 0.10 * self.passive_level(PassiveKind::Might) as f32
    }
    fn haste(&self) -> f32 {
        1.0 - 0.08 * self.passive_level(PassiveKind::Haste) as f32
    }
    pub fn area_mult(&self) -> f32 {
        1.0 + 0.10 * self.passive_level(PassiveKind::Expanse) as f32
    }
    fn move_speed(&self) -> f32 {
        110.0 * (1.0 + 0.08 * self.passive_level(PassiveKind::Swiftness) as f32)
    }
    pub fn magnet(&self) -> f32 {
        // Ren's innate "Shrine Keeper's Eye": +10% pickup radius.
        55.0 * 1.10 * (1.0 + 0.30 * self.passive_level(PassiveKind::Magnet) as f32)
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

    /// The living mid-boss, if any: (index, hp fraction).
    pub fn boss(&self) -> Option<(usize, f32)> {
        let i = self.enemies.kind.iter().position(|k| *k == EnemyKind::GreatOni)?;
        let max = enemy_def(EnemyKind::GreatOni).hp * hp_scale(MIDBOSS_AT);
        Some((i, (self.enemies.hp[i] / max).clamp(0.0, 1.0)))
    }

    // ───────────── Player ─────────────

    fn persistence(&self) -> f32 {
        1.0 + 0.12 * self.passive_level(PassiveKind::Persistence) as f32
    }

    fn move_player(&mut self, input: Input) {
        let regen = 0.25 * self.passive_level(PassiveKind::Recovery) as f32;
        self.player.hp = (self.player.hp + regen * DT).min(self.player.max_hp);
        let d = input.move_dir.normalize_or_zero();
        self.player.moving = d != Vec2::ZERO;
        if self.player.moving {
            self.player.facing = d;
        }
        self.player.pos += d * (self.move_speed() * DT);
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
            if enemies.hp[j] > 0.0 && (enemies.pos[j] - p).length_sq() <= r * r {
                worst = worst.max(enemies.damage[j]);
            }
        });
        if worst > 0.0 {
            let worst = (worst - self.passive_level(PassiveKind::Guard) as f32).max(1.0);
            self.player.hp = (self.player.hp - worst).max(0.0);
            self.player.iframes = IFRAMES;
            self.events.push(Event::PlayerHurt);
            if self.player.hp <= 0.0 {
                self.state = RunState::Dead;
            }
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
        e.hp.push(d.hp * hp_scale(self.time));
        e.speed.push(d.speed * self.rng.range(0.9, 1.1));
        e.radius.push(d.radius);
        e.damage.push(d.damage * damage_scale(self.time));
        e.xp.push(d.xp);
        e.knock.push(d.knockback);
        e.flash.push(0);
        e.orb_cd.push(0);
        e.ai_state.push(0);
        e.ai_timer.push(3.0);
        e.ai_dir.push(Vec2::ZERO);
        e.wave_cd.push(0);
    }

    fn director(&mut self) {
        let target = match self.cfg.stress {
            Some(s) => s.enemies,
            None => min_alive(self.time),
        };
        // Refill is rate-limited (ramping over the run) so strong area weapons can't turn
        // the director into an endless XP fountain. Stress mode fills fast.
        let per_tick = if self.cfg.stress.is_some() { 30 } else { refill_per_tick(self.time) };
        let deficit = target.saturating_sub(self.enemies.len()).min(per_tick);
        for _ in 0..deficit {
            let kind = if self.rng.f32() < kasa_share(self.time) { EnemyKind::Kasa } else { EnemyKind::Wisp };
            let r = self.cfg.spawn_radius + self.rng.range(0.0, 80.0);
            let p = self.ring_point(r);
            self.spawn_enemy(kind, p);
        }

        if self.cfg.stress.is_none() {
            if !self.boss_spawned && self.time >= MIDBOSS_AT {
                self.boss_spawned = true;
                let p = self.ring_point(self.cfg.spawn_radius * 0.8);
                self.spawn_enemy(EnemyKind::GreatOni, p);
                self.events.push(Event::BossSpawned);
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
        // Pass 2: integrate.
        for i in 0..e.pos.len() {
            let v = e.vel[i] + e.push[i];
            e.pos[i] += v * DT;
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
            self.player.special = (self.player.special + 1.0 / SPECIAL_KILLS).min(1.0);
            self.events.push(Event::Kill { pos, kind });
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
                            self.player.hp = (self.player.hp + 1.0).min(self.player.max_hp);
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

    fn gain_xp(&mut self, amount: u32) {
        self.player.xp += amount;
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
                self.player.hp = (self.player.hp + 30.0).min(self.player.max_hp);
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
        self.events.push(Event::SpecialUsed { pos: p });
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
                    self.player.hp = (self.player.hp + 20.0).min(self.player.max_hp);
                }
            }
            Upgrade::Heal => self.player.hp = (self.player.hp + 30.0).min(self.player.max_hp),
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
        let (i, frac) = w.boss().expect("boss should be alive");
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

    /// Full 15-minute bot run; prints how far it got. `cargo test -p sim balance -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn balance_full_run() {
        for seed in 1..=10 {
            let started = std::time::Instant::now();
            let mut w = World::new(WorldConfig { seed, ..Default::default() });
            while w.state == RunState::Playing {
                if w.offer.is_some() {
                    w.choose(0);
                }
                if w.chest_result.is_some() {
                    w.close_chest();
                }
                let input = w.bot_input();
                w.step(input);
            }
            let awakened: Vec<_> = w.weapons.iter().filter(|x| x.awakened).map(|x| awakened_name(x.kind)).collect();
            println!(
                "seed {seed}: {:?} at {:>5.0}s  level {:>2}  kills {:>6}  ({:.1}s real)  weapons {:?}  awakened {:?}",
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
