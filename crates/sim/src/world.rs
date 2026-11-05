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
const MAX_GEMS: usize = 400;
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
    pub stress: Option<Stress>,
    pub spawn_radius: f32,
    pub recycle_radius: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        WorldConfig { seed: 1, stress: None, spawn_radius: 420.0, recycle_radius: 900.0 }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    /// Desired movement, any length (normalised internally).
    pub move_dir: Vec2,
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
}

impl Projectiles {
    pub fn len(&self) -> usize {
        self.pos.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pos.is_empty()
    }
    fn push(&mut self, pos: Vec2, vel: Vec2, life: f32, damage: f32, pierce: u8) {
        self.pos.push(pos);
        self.vel.push(vel);
        self.life.push(life);
        self.damage.push(damage);
        self.pierce.push(pierce);
        self.last_hit.push(u32::MAX);
    }
    fn swap_remove(&mut self, i: usize) {
        self.pos.swap_remove(i);
        self.vel.swap_remove(i);
        self.life.swap_remove(i);
        self.damage.swap_remove(i);
        self.pierce.swap_remove(i);
        self.last_hit.swap_remove(i);
    }
}

#[derive(Default)]
pub struct Gems {
    pub pos: Vec<Vec2>,
    pub value: Vec<u32>,
    /// Excess XP once MAX_GEMS is reached collects into one big gem.
    pub overflow: Option<(Vec2, u32)>,
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
}

#[derive(Clone, Copy, Debug)]
pub struct OwnedWeapon {
    pub kind: WeaponKind,
    pub level: u8,
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
            },
            weapons: vec![OwnedWeapon { kind: WeaponKind::Ofuda, level: 1, timer: 0.0, phase: 0.0 }],
            passives: Vec::new(),
            enemies: Enemies::default(),
            projectiles: Projectiles::default(),
            gems: Gems::default(),
            time: 0.0,
            tick: 0,
            kills: 0,
            state: RunState::Playing,
            offer: None,
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
        self.state != RunState::Playing || self.offer.is_some()
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
        self.move_projectiles_and_hit();
        self.remove_dead_enemies();
        self.collect_gems();
        self.contact_damage();
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
        55.0 * (1.0 + 0.30 * self.passive_level(PassiveKind::Magnet) as f32)
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

    // ───────────── Player ─────────────

    fn move_player(&mut self, input: Input) {
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
            if (enemies.pos[j] - p).length_sq() <= r * r {
                worst = worst.max(enemies.damage[j]);
            }
        });
        if worst > 0.0 {
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
        e.damage.push(d.damage);
        e.xp.push(d.xp);
        e.knock.push(d.knockback);
        e.flash.push(0);
        e.orb_cd.push(0);
    }

    fn director(&mut self) {
        let target = match self.cfg.stress {
            Some(s) => s.enemies,
            None => min_alive(self.time),
        };
        // Cap per tick so a big deficit fills over a few frames instead of one spike.
        let deficit = target.saturating_sub(self.enemies.len()).min(30);
        for _ in 0..deficit {
            let kind = if self.rng.f32() < kasa_share(self.time) { EnemyKind::Kasa } else { EnemyKind::Wisp };
            let r = self.cfg.spawn_radius + self.rng.range(0.0, 80.0);
            let p = self.ring_point(r);
            self.spawn_enemy(kind, p);
        }

        if self.cfg.stress.is_none() {
            if self.time >= self.next_elite {
                self.next_elite += ELITE_EVERY;
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
            let seek = (player - p).normalize_or_zero() * speed;
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
            let s = weapon_stats(w.kind, w.level);
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
                        for k in 0..s.amount {
                            let a = base + (k as f32 - (n - 1.0) / 2.0) * 0.13;
                            self.projectiles.push(self.player.pos, Vec2::from_angle(a) * s.speed, 1.4, s.damage * might, s.pierce);
                        }
                    }
                    self.weapons[wi].timer = timer;
                }
                WeaponKind::MoonOrbs => {
                    let phase = w.phase + s.speed * DT;
                    self.weapons[wi].phase = phase;
                    let (orbit, r) = (self.orb_orbit(w.level), self.orb_radius(w.level));
                    let rehit = (s.cooldown * 60.0) as u8;
                    for k in 0..s.amount {
                        let a = phase + k as f32 / s.amount as f32 * std::f32::consts::TAU;
                        let c = self.player.pos + Vec2::from_angle(a) * orbit;
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
                        let r = self.field_radius(w.level);
                        self.query_circle(self.player.pos, r);
                        for idx in 0..self.scratch.len() {
                            let j = self.scratch[idx] as usize;
                            let dir = self.enemies.pos[j] - self.player.pos;
                            self.hurt_enemy(j, s.damage * might, dir * 0.3);
                        }
                    }
                    self.weapons[wi].timer = timer;
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
                    self.projectiles.push(self.player.pos, Vec2::from_angle(a) * speed, 2.0, 3.0, 2);
                }
            }
        }
    }

    fn move_projectiles_and_hit(&mut self) {
        let mut i = 0;
        while i < self.projectiles.len() {
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
        if self.gems.pos.len() < MAX_GEMS {
            self.gems.pos.push(pos);
            self.gems.value.push(value);
        } else {
            let o = self.gems.overflow.get_or_insert((pos, 0));
            o.1 += value;
        }
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
        if let Some((pos, value)) = self.gems.overflow {
            let d = p - pos;
            if d.length_sq() < grab2 {
                gained += value;
                self.gems.overflow = None;
            } else if d.length_sq() < magnet * magnet {
                self.gems.overflow = Some((pos + d.normalize_or_zero() * (280.0 * DT), value));
            }
        }
        if gained > 0 {
            self.gain_xp(gained);
        }
    }

    fn make_offer(&mut self) {
        let mut pool: Vec<Upgrade> = Vec::new();
        for w in &self.weapons {
            if w.level < MAX_LEVEL {
                pool.push(Upgrade::Weapon(w.kind));
            }
        }
        if self.weapons.len() < MAX_WEAPONS {
            pool.extend(ALL_WEAPONS.iter().filter(|k| self.weapon_level(**k) == 0).map(|k| Upgrade::Weapon(*k)));
        }
        for (k, l) in &self.passives {
            if *l < MAX_LEVEL {
                pool.push(Upgrade::Passive(*k));
            }
        }
        if self.passives.len() < MAX_PASSIVES {
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
        match u {
            Upgrade::Weapon(k) => match self.weapons.iter_mut().find(|w| w.kind == k) {
                Some(w) => w.level += 1,
                None => self.weapons.push(OwnedWeapon { kind: k, level: 1, timer: 0.0, phase: 0.0 }),
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
        self.pending_levels = self.pending_levels.saturating_sub(1);
        if self.pending_levels > 0 {
            self.make_offer();
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
            w.step(Input { move_dir: Vec2::new(a.cos(), a.sin()) });
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
        let mut w = play(5, 60 * 20);
        for _ in 0..60 * 120 {
            if w.offer.is_some() {
                break;
            }
            assert_eq!(w.state, RunState::Playing, "died before an offer appeared");
            let a = w.time;
            w.step(Input { move_dir: Vec2::new(a.cos(), a.sin()) });
        }
        assert!(w.offer.is_some(), "no offer within 2 min");
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
}

