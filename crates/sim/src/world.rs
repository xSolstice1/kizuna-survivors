//! The run simulation: player, weapons, spawn director, enemies, pickups.

use crate::campaign::{Difficulty, Mechanic, StageDef, StageId};
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
/// Move speed inside spirit pressure.
const PRESSURE_SLOW: f32 = 0.6;

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
    pub stage: StageId,
    pub difficulty: Difficulty,
}

impl Default for WorldConfig {
    fn default() -> Self {
        WorldConfig { seed: 1, hero: HeroId::Ren, stress: None, spawn_radius: 420.0, recycle_radius: 900.0, meta: Bonus::default(), stage: StageId::Shotengai, difficulty: Difficulty::Normal }
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
    /// A mini-boss or boss arrived.
    BossSpawned(EnemyKind),
    /// The elite wave began.
    EliteWave,
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
    /// The stage's final boss arrived.
    NightLordSpawned(EnemyKind),
    /// A boss crossed into a new phase.
    BossPhase { pos: Vec2 },
    /// A hazard went off: 0 blast/tag, 1 lightning, 2 domain opened.
    Hazard { pos: Vec2, radius: f32, kind: u8 },
    /// A danmaku curtain started sweeping in.
    Curtain,
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
    /// A bursting projectile or a delayed blast went off (see `style`).
    Burst { pos: Vec2, radius: f32, style: u8 },
    /// An Awakened weapon's signature effect fired.
    Signature(WeaponKind),
    /// Ninefold Foxfire began (with its opening blast) / ended.
    FoxAwakened { pos: Vec2 },
    FoxFaded,
    /// Ninefold Foxfire fired a ring of golden orbs.
    FoxVolley,
    /// A Fox Mirage clone appeared / struck / burst into smoke.
    CloneSummoned { pos: Vec2 },
    CloneStrike { pos: Vec2 },
    ClonePopped { pos: Vec2, radius: f32 },
    /// Hannya Unbound began (onset shockwave).
    HannyaAwakened { pos: Vec2 },
    /// A beam from `from` to `to` (Rending Beam = crimson, Lunar Eclipse = moon).
    Beam { from: Vec2, to: Vec2, style: u8 },
    /// Shadow Step: the player blinked from `from` to `to` (an afterimage stays at `from`).
    ShadowStep { from: Vec2, to: Vec2 },
    AfterimageBlast { pos: Vec2, radius: f32 },
    /// Hannya's Wrath is charging / went off.
    FinaleCharge { pos: Vec2 },
    FinaleBlast { pos: Vec2, radius: f32 },
    /// An Iai Flash cut along a line.
    Iai { from: Vec2, to: Vec2 },
    /// A crimson tendril lashed from `from` to `to`.
    Lash { from: Vec2, to: Vec2 },
    /// Devour's draining pulse.
    Devour { pos: Vec2, radius: f32 },
    /// Hundred-Leg Rampage began (with the Thousand-Leg Burst).
    MukadeAwakened { pos: Vec2 },
    /// A Gravity Well collapsed.
    WellCollapse { pos: Vec2, radius: f32 },
    /// Kuon's ward negated a hit.
    Warded,
    /// Ren's barrier blocked a fireball.
    BarrierBlock,
    /// Divine Purification began / its final circle is charging.
    DivineAwakened { pos: Vec2 },
    DivineCharge { pos: Vec2 },
    /// Domain: Endless Firmament opened / is charging its finale / launched Void Lance: Maximum.
    DomainOpened { pos: Vec2 },
    VoidCharge { pos: Vec2 },
    VoidMaximum { pos: Vec2 },
    /// A floor pickup appeared / was taken.
    PickupSpawned { pos: Vec2, kind: PickupKind },
    PickupTaken { pos: Vec2, kind: PickupKind },
    /// A soul-lantern pulsed.
    LanternPulse { pos: Vec2, radius: f32 },
}

/// A Soul Tether chain on one enemy (found by id each tick).
#[derive(Clone, Copy, Debug)]
pub struct Tether {
    pub enemy: u32,
    pub life: f32,
    tick: f32,
    damage: f32,
    reach: f32,
    heals: bool,
}

/// A Lantern Vigil soul-lantern on the ground.
#[derive(Clone, Copy, Debug)]
pub struct Lantern {
    pub pos: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub radius: f32,
    tick: f32,
    damage: f32,
    slows: bool,
}

/// A lingering Moonsever scar that keeps cutting.
#[derive(Clone, Copy, Debug)]
pub struct Scar {
    pub from: Vec2,
    pub to: Vec2,
    pub life: f32,
    tick: f32,
    damage: f32,
}

/// Something the minimap should show. Stages and modes add markers here; the UI draws
/// whatever it gets, so new content needs no minimap changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkerKind {
    /// Ordinary enemy (drawn as a faint dot).
    Enemy,
    Elite,
    /// A mid-run boss (the Great Oni).
    MiniBoss,
    /// The stage boss (the Night Lord).
    Boss,
    Chest { tier: u8 },
    Shrine,
    Pickup(PickupKind),
    /// Future objectives (escort targets, exits, ...): drawn as a pulsing star.
    Objective,
}

#[derive(Clone, Copy, Debug)]
pub struct MapMarker {
    pub pos: Vec2,
    pub kind: MarkerKind,
}

/// A Fox Mirage clone: fights near the player, draws enemies, pops in damaging smoke.
#[derive(Clone, Copy, Debug)]
pub struct MirageClone {
    pub pos: Vec2,
    pub facing: Vec2,
    pub moving: bool,
    pub life: f32,
    /// Hits it can still take.
    pub hits: u8,
    iframes: f32,
    strike_cd: f32,
    /// Seconds left of the current strike pose (for drawing).
    pub striking: f32,
    damage: f32,
    pop_radius: f32,
    speed: f32,
    /// Formation slot around the player while idle.
    slot: f32,
}

/// Enemy fireball.
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    /// Look: 0 fireball, otherwise the stage index (themed bullets).
    pub style: u8,
}

/// A world hazard (see `Mechanic`) or a boss's telegraphed strike.
#[derive(Clone, Copy, Debug)]
pub struct Hazard {
    pub pos: Vec2,
    pub radius: f32,
    /// Seconds of warning before it is live.
    pub warn: f32,
    /// Seconds it stays live after the warning (0 = one-shot).
    pub life: f32,
    pub kind: HazardKind,
    tick: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HazardKind {
    /// Explodes once; hurts the player and Otherworlders inside.
    Blast { damage: f32 },
    /// Strikes once from the sky.
    Lightning { damage: f32 },
    /// Drifts after the player; slows them inside.
    Pressure,
    /// Burns the player every second while they are inside.
    Domain { damage: f32 },
}

/// Expanding Taiko shockwave ring.
#[derive(Clone, Copy, Debug)]
pub struct Wave {
    pub center: Vec2,
    pub radius: f32,
    pub max_radius: f32,
    pub speed: f32,
    pub damage: f32,
    /// Purification Wave: blasts enemies back harder and erases fireballs it passes.
    pub purify: bool,
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
    /// Ticks until an infinitely piercing projectile (shuriken, golden orb) may hit again.
    pub pierce_cd: Vec<u8>,
    /// Boss AI: state (0 chase, 1 telegraph, 2 charge), timer, charge direction.
    pub ai_state: Vec<u8>,
    pub ai_timer: Vec<f32>,
    pub ai_dir: Vec<Vec2>,
    /// Ticks until a Taiko wave may hit this enemy again.
    pub wave_cd: Vec<u8>,
    pub max_hp: Vec<f32>,
    /// Boss phase reached (0-based).
    pub phase: Vec<u8>,
    /// Seconds left of a committed move (dash, charge); the move's direction.
    pub ai_busy: Vec<f32>,
    pub ai_aux: Vec<Vec2>,
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
        self.pierce_cd.swap_remove(i);
        self.ai_state.swap_remove(i);
        self.ai_timer.swap_remove(i);
        self.ai_dir.swap_remove(i);
        self.wave_cd.swap_remove(i);
        self.max_hp.swap_remove(i);
        self.phase.swap_remove(i);
        self.ai_busy.swap_remove(i);
        self.ai_aux.swap_remove(i);
    }
}

/// What a projectile is (behaviour and look).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjKind {
    Talisman,
    /// Charges in the hand, flies, bursts on contact (Foxfire Orb).
    Foxfire,
    /// Awakened Foxfire Orb: bursts into three sparks too.
    Comet,
    Spark,
    /// Ninefold Foxfire volley.
    GoldenOrb,
    Familiar,
    /// Flies out and returns to the player.
    Shuriken,
    /// A crescent of spirit energy (Grave Cleaver, Hannya Unbound).
    CrescentWave,
    /// Hundredfold Seal: a homing seal that explodes on contact.
    Seal,
    /// Hanafubuki petal.
    Petal,
    /// Lantern Volley spirit bolt.
    SpiritBolt,
    /// Venom Fang: homes in and bursts.
    Fang,
    /// Repulsion Star: pierces and blasts enemies back.
    RedStar,
    /// Void Lance: a slow orb that erases its line.
    VoidOrb,
    /// Spirit Seal orb: gathers around Ren, then homes in and bursts.
    SealOrb,
}

/// A Gravity Well singularity: pulls enemies in, then collapses.
#[derive(Clone, Copy, Debug)]
pub struct Well {
    pub pos: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub radius: f32,
    damage: f32,
    tick: f32,
}

/// Burst / blast looks (shared by the sim events and the renderer).
pub mod style {
    pub const FOXFIRE: u8 = 0;
    pub const SEAL: u8 = 5;
    pub const METEOR: u8 = 6;
    pub const FOX: u8 = 7;
    pub const SKYBOLT: u8 = 9;
    pub const COMET: u8 = 10;
    pub const QUAKE: u8 = 11;
    /// Giant yin-yang crash and Final Purification.
    pub const YINYANG: u8 = 19;
    pub const PURIFY: u8 = 20;
    pub const SPINE: u8 = 12;
    pub const VENOM: u8 = 13;
    /// Thousand-Leg Burst.
    pub const FRENZY: u8 = 14;
    /// Beam styles.
    pub const CRIMSON: u8 = 0;
    pub const MOON: u8 = 1;
}

/// A delayed area blast: meteors, comets, sky lightning. Drawn as a telegraph until it lands.
#[derive(Clone, Copy, Debug)]
pub struct Blast {
    pub pos: Vec2,
    pub delay: f32,
    pub max_delay: f32,
    pub radius: f32,
    damage: f32,
    pub style: u8,
    /// Leaves burning ground.
    burns: bool,
}

#[derive(Default)]
pub struct Projectiles {
    pub kind: Vec<ProjKind>,
    pub radius: Vec<f32>,
    /// Seconds left charging in the hand (> 0 = not launched yet).
    pub charge: Vec<f32>,
    /// Burst radius on removal (0 = none).
    pub burst: Vec<f32>,
    pub max_life: Vec<f32>,
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
        self.push_kind(ProjKind::Talisman, PROJECTILE_RADIUS, pos, vel, life, damage, pierce, homing);
    }
    #[allow(clippy::too_many_arguments)]
    fn push_kind(&mut self, kind: ProjKind, radius: f32, pos: Vec2, vel: Vec2, life: f32, damage: f32, pierce: u8, homing: f32) {
        self.kind.push(kind);
        self.radius.push(radius);
        self.charge.push(0.0);
        self.burst.push(0.0);
        self.max_life.push(life);
        self.pos.push(pos);
        self.vel.push(vel);
        self.life.push(life);
        self.damage.push(damage);
        self.pierce.push(pierce);
        self.last_hit.push(u32::MAX);
        self.homing.push(homing);
    }
    fn swap_remove(&mut self, i: usize) {
        self.kind.swap_remove(i);
        self.radius.swap_remove(i);
        self.charge.swap_remove(i);
        self.burst.swap_remove(i);
        self.max_life.swap_remove(i);
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
    /// Seconds of Ninefold Foxfire left (0 = not transformed).
    pub form: f32,
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
    fox_tick: f32,
    fox_volley: f32,
    fox_volleys: u32,
    hannya_beam: f32,
    hannya_wave: f32,
    hannya_step: f32,
    beam_rot: f32,
    finale_charging: bool,
    /// Shadow Step afterimages waiting to explode: (position, seconds left).
    pub afterimages: Vec<(Vec2, f32)>,
    revivals_left: u8,
    /// Sen collected this run.
    pub sen: u32,
    /// Sen coins on the ground: (position, value).
    pub coins: Vec<(Vec2, u32)>,
    pub clones: Vec<MirageClone>,
    pub tethers: Vec<Tether>,
    pub wells: Vec<Well>,
    /// Ren's barrier: seconds until it blocks again.
    pub barrier_cd: f32,
    divine_ofuda: f32,
    divine_orb: f32,
    divine_ring: f32,
    divine_charging: bool,
    /// Kuon's ward: seconds until it can block again (0 = ready).
    pub ward_cd: f32,
    /// Centre of Endless Firmament while it is open.
    pub domain_center: Vec2,
    domain_charging: bool,
    /// Floor pickups waiting on the ground.
    pub pickups: Vec<(Vec2, PickupKind)>,
    next_pickup: f32,
    /// Seconds left on pickup buffs.
    pub speed_buff: f32,
    pub damage_buff: f32,
    pub freeze: f32,
    pub vacuum: f32,
    pub blasts: Vec<Blast>,
    /// Per-weapon signature timers, indexed like `ALL_WEAPONS`.
    sig_timers: [f32; ALL_WEAPONS.len()],
    pub lanterns: Vec<Lantern>,
    pub scars: Vec<Scar>,
    pub great_oni_defeated: bool,
    /// Kills per enemy kind, indexed like `ALL_ENEMIES`.
    pub kills_by_kind: [u32; ALL_ENEMIES.len()],
    /// Shrine Events taken this run.
    pub shrines_taken: Vec<ShrineEvent>,
    /// Run counters for the Stats screen.
    pub chests_opened: u32,
    pub specials_used: u32,
    /// True if the run ended because dawn arrived with the Night Lord alive.
    pub dawn_out: bool,
    night_lord_spawned: bool,
    /// Bosses defeated this run: bit 0 mini-boss, bit 1 boss, bit 2 final boss.
    pub bosses_down: u8,
    second_boss_spawned: bool,
    elite_wave_done: bool,
    /// World hazards and boss strikes on the field.
    pub hazards: Vec<Hazard>,
    next_hazard: f32,
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
                form: 0.0,
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
            fox_tick: 0.0,
            fox_volley: 0.0,
            fox_volleys: 0,
            hannya_beam: 0.0,
            hannya_wave: 0.0,
            hannya_step: 0.0,
            beam_rot: 0.0,
            finale_charging: false,
            afterimages: Vec::new(),
            revivals_left: bonus.revivals,
            sen: 0,
            coins: Vec::new(),
            clones: Vec::new(),
            tethers: Vec::new(),
            wells: Vec::new(),
            barrier_cd: 0.0,
            divine_ofuda: 0.0,
            divine_orb: 0.0,
            divine_ring: 0.0,
            divine_charging: false,
            ward_cd: 0.0,
            domain_center: Vec2::ZERO,
            domain_charging: false,
            pickups: Vec::new(),
            next_pickup: 30.0,
            speed_buff: 0.0,
            damage_buff: 0.0,
            freeze: 0.0,
            vacuum: 0.0,
            blasts: Vec::new(),
            sig_timers: [0.0; ALL_WEAPONS.len()],
            lanterns: Vec::new(),
            scars: Vec::new(),
            great_oni_defeated: false,
            kills_by_kind: [0; ALL_ENEMIES.len()],
            shrines_taken: Vec::new(),
            chests_opened: 0,
            specials_used: 0,
            dawn_out: false,
            night_lord_spawned: false,
            bosses_down: 0,
            second_boss_spawned: false,
            elite_wave_done: false,
            hazards: Vec::new(),
            next_hazard: 45.0,
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
        self.update_mechanic();
        self.grid.rebuild(self.player.pos, &self.enemies.pos);
        // Needs the fresh grid (queries index into the enemy arrays).
        self.update_bosses();
        self.move_enemies();
        self.update_weapons();
        self.update_signatures();
        self.update_fox();
        self.update_hannya();
        self.update_mukade();
        self.update_domain();
        self.update_divine();
        self.update_clones();
        self.update_spirit_arts();
        self.update_effects();
        self.move_projectiles_and_hit();
        if input.special {
            self.use_special();
        }
        self.contact_damage();
        self.update_shots();
        self.update_hazards();
        self.update_pickups();
        // Everything that reads the grid must run before this: removal reorders
        // the enemy arrays and invalidates the grid's indices.
        self.remove_dead_enemies();
        self.collect_gems();
        self.collect_chests();
        self.collect_shrine();
        if self.state == RunState::Playing && self.time >= self.stage().limit {
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
        let sake = if self.damage_buff > 0.0 { DAMAGE_BUFF } else { 1.0 } * if self.form_kind() == Some(SpecialKind::Firmament) { DOMAIN_MIGHT } else { 1.0 };
        (1.0 + 0.10 * self.passive_level(PassiveKind::Might) as f32 + self.bonus.might) * self.mods.damage * sake
    }
    fn haste(&self) -> f32 {
        let frenzy = match self.form_kind() {
            Some(SpecialKind::Mukade) => MUKADE_HASTE,
            Some(SpecialKind::Firmament) => DOMAIN_HASTE,
            _ => 1.0,
        };
        (1.0 - 0.08 * self.passive_level(PassiveKind::Haste) as f32 - self.bonus.cooldown) * frenzy
    }
    pub fn area_mult(&self) -> f32 {
        1.0 + 0.10 * self.passive_level(PassiveKind::Expanse) as f32 + self.bonus.area
    }
    fn move_speed(&self) -> f32 {
        let fox = match self.form_kind() {
            Some(SpecialKind::NineTails) => FOX_MOVE_SPEED,
            Some(SpecialKind::Hannya) => HANNYA_MOVE_SPEED,
            Some(SpecialKind::Mukade) => MUKADE_MOVE_SPEED,
            _ => 1.0,
        } * if self.speed_buff > 0.0 { SPEED_BUFF } else { 1.0 };
        110.0 * (1.0 + 0.08 * self.passive_level(PassiveKind::Swiftness) as f32 + self.bonus.move_speed) * self.mods.move_speed * fox
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
    /// Yin-yang orbs: they orbit Ren on a radius that swings wide and back.
    pub fn yinyang_positions(&self, w: &OwnedWeapon) -> Vec<Vec2> {
        let s = self.stats_of(w);
        let n = s.amount.max(1);
        let swing = 0.5 + 0.5 * (w.phase * 0.35).sin();
        let r = (34.0 + 52.0 * swing) * s.area * self.area_mult();
        (0..n).map(|k| self.player.pos + Vec2::new(0.0, -6.0) + Vec2::from_angle(w.phase + k as f32 / n as f32 * std::f32::consts::TAU) * r).collect()
    }

    pub fn yinyang_radius(&self, w: &OwnedWeapon) -> f32 {
        9.0 * self.stats_of(w).area * self.area_mult()
    }

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

    /// The living boss of the highest rank, if any: (index, hp fraction, kind).
    pub fn boss(&self) -> Option<(usize, f32, EnemyKind)> {
        let k = &self.enemies.kind;
        let i = (0..k.len()).filter(|&i| k[i].is_boss()).max_by_key(|&i| k[i].rank())?;
        Some((i, (self.enemies.hp[i] / self.enemies.max_hp[i]).clamp(0.0, 1.0), k[i]))
    }

    // ───────────── Player ─────────────

    fn armor(&self) -> f32 {
        self.passive_level(PassiveKind::Guard) as f32 + self.mods.armor + self.bonus.armor
    }

    /// Applies a hit to the player (after armor) and handles death / Revival.
    fn hurt_player(&mut self, raw: f32) {
        // Divine Purification: nothing touches the shrine maiden.
        if self.form_kind() == Some(SpecialKind::Divine) {
            return;
        }
        // Kuon's ward: a hit is simply not allowed to land.
        if self.cfg.hero == HeroId::Kuon && self.ward_cd <= 0.0 {
            self.ward_cd = WARD_RECHARGE;
            self.player.iframes = 0.6;
            self.events.push(Event::Warded);
            return;
        }
        let raw = match self.form_kind() {
            Some(SpecialKind::NineTails) => raw * FOX_DAMAGE_TAKEN,
            Some(SpecialKind::Hannya) => raw * HANNYA_DAMAGE_TAKEN,
            Some(SpecialKind::Mukade) => raw * MUKADE_DAMAGE_TAKEN,
            _ => raw,
        };
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
        let pressure = if self.under_pressure() { PRESSURE_SLOW } else { 1.0 };
        self.player.pos += d * (self.move_speed() * pressure * DT);
        self.player.pos = stage::resolve(self.cfg.stage, self.player.pos, PLAYER_RADIUS);
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
        let frozen = self.freeze > 0.0 || self.form_kind() == Some(SpecialKind::Firmament);
        let mut worst = 0.0f32;
        let enemies = &self.enemies;
        self.grid.for_each_near(p, PLAYER_RADIUS + MAX_ENEMY_RADIUS, |j| {
            let j = j as usize;
            let r = PLAYER_RADIUS + enemies.radius[j];
            let faded = (enemy_def(enemies.kind[j]).ai == Ai::Blink && enemies.ai_state[j] != 0) || (frozen && !enemies.kind[j].is_boss());
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

    pub fn stage(&self) -> &'static StageDef {
        self.cfg.stage.def()
    }

    /// Fodder/elite HP multiplier now: the stage's stretched curve, its tier and difficulty.
    fn enemy_hp_scale(&self) -> f32 {
        let st = self.stage();
        let late = 1.0 + st.late_hp * (self.time / st.final_at).min(1.2).powi(2);
        hp_scale(st.pace(self.time)) * late * st.tier * self.cfg.difficulty.hp()
    }

    /// Enemy damage multiplier now (contact, bullets, hazards).
    pub fn enemy_damage_scale(&self) -> f32 {
        damage_scale(self.stage().pace(self.time)) * self.cfg.difficulty.damage()
    }

    fn spawn_enemy(&mut self, kind: EnemyKind, pos: Vec2) {
        if self.enemies.len() >= MAX_ENEMIES {
            return;
        }
        let d = enemy_def(kind);
        let hp = if kind.is_boss() {
            d.hp * self.stage().tier * self.cfg.difficulty.hp() * self.mods.enemy_hp
        } else {
            d.hp * self.enemy_hp_scale() * self.mods.enemy_hp
        };
        let damage = d.damage * self.enemy_damage_scale();
        let timer = match d.ai {
            Ai::Ranged { every, .. } => every * 0.4 + self.rng.range(0.0, every * 0.5),
            Ai::Blink => 2.5 + self.rng.range(0.0, 2.0),
            Ai::Dash => 1.5 + self.rng.range(0.0, 2.0),
            Ai::Boss => 2.0,
            _ => 3.0,
        };
        let speed = d.speed * self.rng.range(0.9, 1.1) * self.mods.enemy_speed;
        let e = &mut self.enemies;
        e.id.push(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        e.kind.push(kind);
        e.pos.push(pos);
        e.vel.push(Vec2::ZERO);
        e.push.push(Vec2::ZERO);
        e.hp.push(hp);
        e.max_hp.push(hp);
        e.speed.push(speed);
        e.radius.push(d.radius);
        e.damage.push(damage);
        e.xp.push(d.xp);
        e.knock.push(d.knockback);
        e.flash.push(0);
        e.orb_cd.push(0);
        e.pierce_cd.push(0);
        e.ai_state.push(0);
        e.ai_timer.push(timer);
        e.ai_dir.push(Vec2::ZERO);
        e.wave_cd.push(0);
        e.phase.push(0);
        e.ai_busy.push(0.0);
        e.ai_aux.push(Vec2::ZERO);
    }

    fn spawn_boss(&mut self, kind: EnemyKind) {
        let p = self.ring_point(self.cfg.spawn_radius * 0.8);
        self.spawn_enemy(kind, p);
        self.events.push(if kind.rank() == Rank::Final { Event::NightLordSpawned(kind) } else { Event::BossSpawned(kind) });
    }

    fn director(&mut self) {
        let st = self.stage();
        let pace = st.pace(self.time);
        let density = self.cfg.difficulty.density() * if st.in_crisis(self.time) { 1.35 } else { 1.0 };
        let target = match self.cfg.stress {
            Some(s) => s.enemies,
            None if self.night_lord_spawned => 0,
            None => (min_alive(pace) as f32 * self.mods.curse * density) as usize,
        };
        // Refill is rate-limited (ramping over the run) so strong area weapons can't turn
        // the director into an endless XP fountain. Stress mode fills fast.
        let per_tick = if self.cfg.stress.is_some() { 30 } else { (refill_per_tick(pace) as f32 * density).ceil() as usize };
        let deficit = target.saturating_sub(self.enemies.len()).min(per_tick);
        for _ in 0..deficit {
            let r = self.rng.f32();
            let (c, n) = (chochin_share(pace) * st.ranged_mult, nopperabo_share(pace));
            let kind = if r < c {
                st.ranged
            } else if r < c + n {
                st.blinker
            } else if self.rng.f32() < kasa_share(pace) {
                st.fodder_b
            } else {
                st.fodder
            };
            let r = self.cfg.spawn_radius + self.rng.range(0.0, 80.0);
            let p = self.ring_point(r);
            self.spawn_enemy(kind, p);
        }

        if self.cfg.stress.is_none() {
            if !self.night_lord_spawned && self.time >= st.final_at {
                self.night_lord_spawned = true;
                self.spawn_boss(st.final_boss);
            }
            for (k, p) in std::mem::take(&mut self.pending_spawns) {
                self.spawn_enemy(k, p);
            }
            if !self.boss_spawned && self.time >= st.mini_at {
                self.boss_spawned = true;
                self.spawn_boss(st.mini);
            }
            if let Some((b, at)) = st.boss {
                if !self.second_boss_spawned && self.time >= at {
                    self.second_boss_spawned = true;
                    self.spawn_boss(b);
                }
            }
            if self.shrines_done < st.shrine_times.len() && self.time >= st.shrine_times[self.shrines_done] {
                self.shrines_done += 1;
                let a = self.rng.range(0.0, std::f32::consts::TAU);
                self.shrine = Some(stage::resolve(self.cfg.stage, self.player.pos + Vec2::from_angle(a) * 240.0, 18.0));
                self.events.push(Event::ShrineAppeared);
            }
            if self.time >= self.next_elite {
                self.next_elite += elite_interval(pace) * self.cfg.difficulty.elite_gap();
                let p = self.ring_point(self.cfg.spawn_radius);
                self.spawn_enemy(st.elite, p);
            }
            if let Some(at) = st.elite_wave_at {
                if !self.elite_wave_done && self.time >= at {
                    // The elite wave: a ring of elites closing in at once.
                    self.elite_wave_done = true;
                    let n = 4 + self.cfg.difficulty.index() * 2;
                    for k in 0..n {
                        let a = k as f32 / n as f32 * std::f32::consts::TAU;
                        let p = self.player.pos + Vec2::from_angle(a) * self.cfg.spawn_radius;
                        self.spawn_enemy(st.elite, p);
                    }
                    self.events.push(Event::EliteWave);
                }
            }
            if self.rings_done < st.ring_times.len() && self.time >= st.ring_times[self.rings_done] {
                self.rings_done += 1;
                let n = (120.0 * self.cfg.difficulty.density()) as usize;
                for k in 0..n {
                    let a = k as f32 / n as f32 * std::f32::consts::TAU;
                    let p = self.player.pos + Vec2::from_angle(a) * 270.0;
                    self.spawn_enemy(st.fodder, p);
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

    // ───────────── Bosses and hazards ─────────────

    fn shot_style(&self) -> u8 {
        if self.cfg.stage == StageId::Shotengai { 0 } else { self.cfg.stage.index() as u8 }
    }

    /// Generic bosses: phases by HP, a rotating spray, and a cycle of moves from their kit.
    fn update_bosses(&mut self) {
        let player = self.player.pos;
        let extra = self.cfg.difficulty.extra_phases();
        let style = self.shot_style();
        let dmg = self.enemy_damage_scale();
        let domain = (self.form_kind() == Some(SpecialKind::Firmament)).then_some(self.domain_center);
        let tau = std::f32::consts::TAU;
        for i in 0..self.enemies.len() {
            let kind = self.enemies.kind[i];
            if enemy_def(kind).ai != Ai::Boss || self.enemies.hp[i] <= 0.0 {
                continue;
            }
            let kit = boss_kit(kind);
            let phases = boss_phases(kind, extra);
            let e = &mut self.enemies;
            let p = e.pos[i];
            let to = player - p;
            let base = to.y.atan2(to.x);
            let slow = match domain {
                Some(c) if (p - c).length_sq() < DOMAIN_RADIUS * DOMAIN_RADIUS => 0.15,
                _ => 1.0,
            };
            // Phase changes: each equal slice of HP lost.
            let frac = e.hp[i] / e.max_hp[i];
            let want = (((1.0 - frac) * phases as f32) as u8).min(phases - 1);
            if want > e.phase[i] {
                e.phase[i] = want;
                let n = 24 + 6 * want as usize;
                for k in 0..n {
                    let a = k as f32 / n as f32 * tau;
                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(a) * 120.0, life: 6.0, style });
                }
                for k in 0..6 {
                    let a = k as f32 / 6.0 * tau;
                    self.pending_spawns.push((kit.minion, player + Vec2::from_angle(a) * 220.0));
                }
                e.ai_timer[i] = 1.2;
                self.events.push(Event::BossPhase { pos: p });
            }
            let ph = e.phase[i] as f32;
            let more = 1.0 + 0.25 * ph;
            // A committed charge: 0.8s planted (telegraph), then 0.7s straight run.
            if e.ai_busy[i] > 0.0 {
                e.ai_busy[i] -= DT;
                e.vel[i] = if e.ai_busy[i] > 0.7 { Vec2::ZERO } else { e.ai_aux[i] * (kit.charge_speed * slow) };
                continue;
            }
            e.vel[i] = to.normalize_or_zero() * (e.speed[i] * (1.0 + 0.08 * ph) * slow);
            if let Some(sp) = kit.spiral {
                if e.phase[i] >= sp.from {
                    e.ai_dir[i].x += DT * sp.turn * (1.0 + 0.15 * ph);
                    e.ai_dir[i].y -= DT;
                    if e.ai_dir[i].y <= 0.0 {
                        e.ai_dir[i].y = sp.every;
                        for k in 0..sp.arms {
                            let a = e.ai_dir[i].x + k as f32 / sp.arms as f32 * tau;
                            self.shots.push(Shot { pos: p, vel: Vec2::from_angle(a) * sp.speed, life: 7.0, style });
                        }
                    }
                }
            }
            e.ai_timer[i] -= DT;
            if e.ai_timer[i] > 0.0 {
                continue;
            }
            e.ai_timer[i] = (kit.cycle * (1.0 - 0.12 * ph)).max(kit.cycle * 0.55);
            let mv = kit.moves[e.ai_state[i] as usize % kit.moves.len()];
            e.ai_state[i] = ((e.ai_state[i] as usize + 1) % kit.moves.len()) as u8;
            match mv {
                Move::Ring { n, layers, speed } => {
                    let n = (n as f32 * more) as usize;
                    for l in 0..layers {
                        let off = l as f32 * 0.5 / n as f32 * tau;
                        let v = speed * (1.0 + 0.2 * l as f32) * (1.0 + 0.08 * ph);
                        for k in 0..n {
                            let a = off + k as f32 / n as f32 * tau;
                            self.shots.push(Shot { pos: p, vel: Vec2::from_angle(a) * v, life: 7.0, style });
                        }
                    }
                }
                Move::Fan { n, spread, speed } => {
                    let n = n as usize + 2 * e.phase[i] as usize;
                    for k in 0..n {
                        let a = base + (k as f32 / (n - 1).max(1) as f32 - 0.5) * spread;
                        self.shots.push(Shot { pos: p, vel: Vec2::from_angle(a) * (speed * (1.0 + 0.08 * ph)), life: 6.0, style });
                    }
                }
                Move::Charge => {
                    e.ai_busy[i] = 1.5;
                    e.ai_aux[i] = to.normalize_or_zero();
                    e.vel[i] = Vec2::ZERO;
                    self.events.push(Event::BossTelegraph);
                }
                Move::Blink => {
                    let a = self.rng.range(0.0, tau);
                    let q = player + Vec2::from_angle(a) * 190.0;
                    e.pos[i] = q;
                    let n = 12 + 4 * e.phase[i] as usize;
                    for k in 0..n {
                        let b = k as f32 / n as f32 * tau;
                        self.shots.push(Shot { pos: q, vel: Vec2::from_angle(b) * 90.0, life: 6.0, style });
                    }
                    self.events.push(Event::Blink { pos: q });
                }
                Move::Rain { n } => {
                    let n = (n as f32 * more) as usize;
                    for k in 0..n {
                        let off = if k == 0 { Vec2::ZERO } else { Vec2::from_angle(self.rng.range(0.0, tau)) * self.rng.range(40.0, 170.0) };
                        self.hazards.push(Hazard { pos: player + off, radius: 34.0, warn: 1.1 + k as f32 * 0.06, life: 0.0, kind: HazardKind::Blast { damage: 14.0 * dmg }, tick: 0.0 });
                    }
                }
                Move::Summon { n } => {
                    let n = (n as f32 * more) as usize;
                    for k in 0..n {
                        let a = k as f32 / n as f32 * tau;
                        self.pending_spawns.push((kit.minion, player + Vec2::from_angle(a) * 210.0));
                    }
                }
            }
        }
    }

    /// The stage's signature hazard (none in the prologue).
    fn update_mechanic(&mut self) {
        let st = self.stage();
        if st.mechanic == Mechanic::None || self.cfg.stress.is_some() {
            return;
        }
        self.next_hazard -= DT;
        if self.next_hazard > 0.0 {
            return;
        }
        let crisis = st.in_crisis(self.time);
        let diff = self.cfg.difficulty.index() as f32;
        let dmg = self.enemy_damage_scale();
        let p = self.player.pos;
        let tau = std::f32::consts::TAU;
        let gap = |calm: f32, hot: f32| (if crisis { hot } else { calm }) * (1.0 - 0.12 * diff);
        match st.mechanic {
            Mechanic::None => {}
            Mechanic::Curtains => {
                self.next_hazard = gap(60.0, 32.0);
                // A wall of bullets sweeps in from one side with two gaps to slip through.
                let d = Vec2::from_angle(self.rng.range(0.0, tau));
                let side = Vec2::new(-d.y, d.x);
                let start = p - d * 380.0;
                let gaps = [self.rng.range(-220.0, 0.0), self.rng.range(20.0, 220.0)];
                let style = self.shot_style();
                let mut x = -460.0;
                while x <= 460.0 {
                    if !gaps.iter().any(|g| (x - g).abs() < 42.0) {
                        self.shots.push(Shot { pos: start + side * x, vel: d * 75.0, life: 11.0, style });
                    }
                    x += 18.0;
                }
                self.events.push(Event::Curtain);
            }
            Mechanic::Tags => {
                self.next_hazard = gap(6.5, 3.5);
                let n = 2 + diff as usize + crisis as usize;
                for k in 0..n {
                    let off = Vec2::from_angle(self.rng.range(0.0, tau)) * if k == 0 { self.rng.range(0.0, 30.0) } else { self.rng.range(50.0, 170.0) };
                    self.hazards.push(Hazard { pos: p + off, radius: 44.0, warn: 1.3, life: 0.0, kind: HazardKind::Blast { damage: 12.0 * dmg }, tick: 0.0 });
                }
            }
            Mechanic::Pressure => {
                self.next_hazard = gap(15.0, 8.0);
                let pos = p + Vec2::from_angle(self.rng.range(0.0, tau)) * 280.0;
                self.hazards.push(Hazard { pos, radius: 85.0, warn: 1.0, life: 22.0, kind: HazardKind::Pressure, tick: 0.0 });
            }
            Mechanic::Downpour => {
                self.next_hazard = gap(6.0, 3.0);
                let lead = if self.player.moving { self.player.facing * 30.0 } else { Vec2::ZERO };
                self.hazards.push(Hazard { pos: p + lead, radius: 36.0, warn: 1.2, life: 0.0, kind: HazardKind::Lightning { damage: 16.0 * dmg }, tick: 0.0 });
            }
            Mechanic::Domains => {
                self.next_hazard = gap(100.0, 55.0);
                let pos = p + Vec2::from_angle(self.rng.range(0.0, tau)) * self.rng.range(0.0, 80.0);
                self.hazards.push(Hazard { pos, radius: 210.0, warn: 3.0, life: 14.0, kind: HazardKind::Domain { damage: 6.0 * dmg }, tick: 1.0 });
            }
        }
    }

    /// True while the player stands in live spirit pressure.
    fn under_pressure(&self) -> bool {
        let p = self.player.pos;
        self.hazards.iter().any(|h| h.kind == HazardKind::Pressure && h.warn <= 0.0 && (h.pos - p).length_sq() < h.radius * h.radius)
    }

    /// Counts down hazards, fires blasts and lightning, burns inside domains. Reads the grid.
    fn update_hazards(&mut self) {
        if self.hazards.is_empty() {
            return;
        }
        let p = self.player.pos;
        let mut hurt = 0.0f32;
        let mut fired: Vec<(Vec2, f32, u8)> = Vec::new();
        for h in &mut self.hazards {
            let inside = (h.pos - p).length_sq() <= (h.radius + PLAYER_RADIUS).powi(2);
            if h.warn > 0.0 {
                h.warn -= DT;
                if h.warn <= 0.0 {
                    match h.kind {
                        HazardKind::Blast { damage } | HazardKind::Lightning { damage } => {
                            if inside {
                                hurt = hurt.max(damage);
                            }
                            fired.push((h.pos, h.radius, matches!(h.kind, HazardKind::Lightning { .. }) as u8));
                            h.life = -1.0;
                        }
                        HazardKind::Domain { .. } => fired.push((h.pos, h.radius, 2)),
                        HazardKind::Pressure => {}
                    }
                }
                continue;
            }
            h.life -= DT;
            match h.kind {
                HazardKind::Pressure => {
                    let d = p - h.pos;
                    if d.length_sq() > 1.0 {
                        h.pos += d.normalize_or_zero() * (24.0 * DT);
                    }
                }
                HazardKind::Domain { damage } => {
                    h.tick -= DT;
                    if h.tick <= 0.0 {
                        h.tick = 1.0;
                        if inside {
                            hurt = hurt.max(damage);
                        }
                    }
                }
                _ => {}
            }
        }
        self.hazards.retain(|h| h.warn > 0.0 || h.life > 0.0);
        let blast = 60.0 * self.enemy_hp_scale();
        for (pos, radius, kind) in fired {
            self.events.push(Event::Hazard { pos, radius, kind });
            if kind < 2 {
                // Tags and lightning don't take sides.
                self.query_circle(pos, radius);
                for k in 0..self.scratch.len() {
                    let j = self.scratch[k] as usize;
                    if !self.enemies.kind[j].is_boss() {
                        let dir = self.enemies.pos[j] - pos;
                        self.hurt_enemy(j, blast, dir);
                    }
                }
            }
        }
        if hurt > 0.0 && self.player.iframes <= 0.0 && self.cfg.stress.is_none() {
            self.hurt_player(hurt);
        }
    }

    // ───────────── Enemies ─────────────

    fn move_enemies(&mut self) {
        let player = self.player.pos;
        let time = self.time;
        // Fox Mirage decoys: nearby enemies chase a clone instead of the player.
        let decoys: Vec<Vec2> = self.clones.iter().map(|c| c.pos).collect();
        let stillness = self.cfg.hero == HeroId::Kuon;
        let domain = (self.form_kind() == Some(SpecialKind::Firmament)).then_some(self.domain_center);
        let style = if self.cfg.stage == StageId::Shotengai { 0 } else { self.cfg.stage.index() as u8 };
        let e = &mut self.enemies;
        let frozen = self.freeze > 0.0;
        // Pass 1: desired velocity = seek player + separation from grid neighbours.
        for i in 0..e.pos.len() {
            if frozen && !e.kind[i].is_boss() {
                e.vel[i] = Vec2::ZERO;
                continue;
            }
            // Endless Firmament: everything inside is suspended (bosses only crawl).
            if let Some(c) = domain {
                if (e.pos[i] - c).length_sq() < DOMAIN_RADIUS * DOMAIN_RADIUS {
                    if e.kind[i].is_boss() {
                        e.vel[i] = (player - e.pos[i]).normalize_or_zero() * (e.speed[i] * 0.15);
                    } else {
                        e.vel[i] = Vec2::ZERO;
                    }
                    continue;
                }
            }
            let p = e.pos[i];
            let ri = e.radius[i];
            let mut speed = e.speed[i];
            // Stillness: space thickens around Kuon.
            if stillness && (e.pos[i] - player).length_sq() < STILLNESS_RADIUS * STILLNESS_RADIUS {
                speed *= STILLNESS_SLOW;
            }
            let ai = enemy_def(e.kind[i]).ai;
            if ai == Ai::Hop {
                // Hopping gait.
                speed *= 0.45 + 0.75 * ((time * 7.0 + e.id[i] as f32 * 0.7).sin().abs());
            }
            let mut to_player = player - p;
            if !decoys.is_empty() && !e.kind[i].is_boss() {
                let mut best = to_player.length_sq();
                for d in &decoys {
                    let v = *d - p;
                    let d2 = v.length_sq();
                    if d2 < DECOY_RADIUS * DECOY_RADIUS && d2 < best {
                        best = d2;
                        to_player = v;
                    }
                }
            }
            match ai {
                // Bosses are steered (and fire) in `update_bosses`.
                Ai::Boss => continue,
                Ai::Ranged { volley, every } => {
                    // Hold a firing range, strafing; fire every `every` seconds.
                    let d = to_player.length();
                    let dir = to_player.normalize_or_zero();
                    let side = Vec2::new(-dir.y, dir.x) * if e.id[i] % 2 == 0 { 1.0 } else { -1.0 };
                    let mv = if d > 220.0 { dir } else if d < 170.0 { dir * -1.0 } else { side * 0.6 };
                    e.vel[i] = mv * speed;
                    e.ai_timer[i] -= DT;
                    if e.ai_timer[i] <= 0.0 && d < 300.0 {
                        e.ai_timer[i] = every;
                        let base = dir.y.atan2(dir.x);
                        match volley {
                            Volley::Single { speed } => self.shots.push(Shot { pos: p, vel: dir * speed, life: 4.0, style }),
                            Volley::Fan3 { speed } => {
                                for k in -1..=1 {
                                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(base + k as f32 * 0.28) * speed, life: 4.0, style });
                                }
                            }
                            Volley::Ring6 { speed } => {
                                for k in 0..6 {
                                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(base + k as f32 / 6.0 * std::f32::consts::TAU) * speed, life: 4.0, style });
                                }
                            }
                        }
                        self.events.push(Event::EnemyShot);
                    }
                    continue;
                }
                Ai::Dash => {
                    // Chase; then crouch (0.35s) and lunge (0.5s) along a locked line.
                    if e.ai_busy[i] > 0.0 {
                        e.ai_busy[i] -= DT;
                        e.vel[i] = if e.ai_busy[i] > 0.5 { Vec2::ZERO } else { e.ai_aux[i] * (speed * 3.2) };
                        if e.ai_busy[i] <= 0.0 {
                            e.ai_timer[i] = 2.5 + self.rng.range(0.0, 2.0);
                        }
                        continue;
                    }
                    e.ai_timer[i] -= DT;
                    if e.ai_timer[i] <= 0.0 && to_player.length_sq() < 240.0 * 240.0 {
                        e.ai_busy[i] = 0.85;
                        e.ai_aux[i] = to_player.normalize_or_zero();
                        e.vel[i] = Vec2::ZERO;
                        continue;
                    }
                }
                Ai::Blink => {
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
                Ai::Daimyo => {
                    // Slow chase; a constant spiral; every 3s cycle burst -> aimed fan -> summon.
                    e.vel[i] = to_player.normalize_or_zero() * speed;
                    e.ai_dir[i].x += DT * 2.2;
                    e.ai_dir[i].y -= DT;
                    if e.ai_dir[i].y <= 0.0 {
                        e.ai_dir[i].y = 0.14;
                        let a = e.ai_dir[i].x;
                        for k in 0..2 {
                            let b = a + k as f32 * std::f32::consts::PI;
                            self.shots.push(Shot { pos: p, vel: Vec2::from_angle(b) * 95.0, life: 6.0, style: 0 });
                        }
                    }
                    e.ai_timer[i] -= DT;
                    if e.ai_timer[i] <= 0.0 {
                        e.ai_timer[i] = 3.0;
                        match e.ai_state[i] {
                            0 => {
                                for k in 0..24 {
                                    let b = k as f32 / 24.0 * std::f32::consts::TAU;
                                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(b) * 110.0, life: 6.0, style: 0 });
                                }
                            }
                            1 => {
                                let base = to_player.y.atan2(to_player.x);
                                for k in 0..9 {
                                    let b = base + (k as f32 - 4.0) * 0.12;
                                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(b) * 150.0, life: 5.0, style: 0 });
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
                Ai::Charger { speed: charge, rest } => {
                    // Chase -> telegraph (stand still, 0.8s) -> charge (0.7s) -> chase.
                    e.ai_timer[i] -= DT;
                    e.vel[i] = match e.ai_state[i] {
                        0 => {
                            if e.ai_timer[i] <= 0.0 && to_player.length_sq() < 380.0 * 380.0 {
                                e.ai_state[i] = 1;
                                e.ai_timer[i] = 0.8;
                                e.ai_dir[i] = to_player.normalize_or_zero();
                                if e.kind[i].is_boss() {
                                    self.events.push(Event::BossTelegraph);
                                }
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
                                e.ai_timer[i] = rest;
                            }
                            e.ai_dir[i] * charge
                        }
                    };
                    continue;
                }
                Ai::Chase | Ai::Hop => {}
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
        let stage_id = self.cfg.stage;
        for i in 0..e.pos.len() {
            let v = e.vel[i] + e.push[i];
            e.pos[i] += v * DT;
            if !e.kind[i].is_boss() {
                let before = e.pos[i];
                let after = stage::resolve_local(stage_id, before, e.radius[i]);
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
            e.pierce_cd[i] = e.pierce_cd[i].saturating_sub(1);
            e.wave_cd[i] = e.wave_cd[i].saturating_sub(1);
        }
    }

    fn burst_projectile(&mut self, i: usize) {
        let (c, r, dmg, kind) = (self.projectiles.pos[i], self.projectiles.burst[i], self.projectiles.damage[i], self.projectiles.kind[i]);
        self.query_circle(c, r);
        for idx in 0..self.scratch.len() {
            let j = self.scratch[idx] as usize;
            let dir = self.enemies.pos[j] - c;
            self.hurt_enemy(j, dmg, dir);
        }
        let st = match kind {
            ProjKind::Seal => style::SEAL,
            ProjKind::Familiar => style::FOX,
            ProjKind::Fang => style::VENOM,
            ProjKind::SealOrb => style::SEAL,
            _ => style::FOXFIRE,
        };
        self.events.push(Event::Burst { pos: c, radius: r, style: st });
        if kind == ProjKind::Comet {
            let a0 = self.rng.range(0.0, std::f32::consts::TAU);
            for k in 0..3 {
                let a = a0 + k as f32 * std::f32::consts::TAU / 3.0;
                self.projectiles.push_kind(ProjKind::Spark, 5.0, c, Vec2::from_angle(a) * 240.0, 0.6, dmg * 0.4, 3, 5.0);
            }
        }
    }

    /// Awakened weapons' signature effects, plus the delayed blasts they create.
    fn update_signatures(&mut self) {
        let p = self.player.pos;
        let might = self.might();
        for wi in 0..self.weapons.len() {
            let w = self.weapons[wi];
            let every = signature_every(w.kind);
            if !w.awakened || every <= 0.0 {
                continue;
            }
            let ti = ALL_WEAPONS.iter().position(|k| *k == w.kind).unwrap();
            self.sig_timers[ti] -= DT;
            if self.sig_timers[ti] > 0.0 {
                continue;
            }
            self.sig_timers[ti] = every * self.haste();
            let s = self.stats_of(&w);
            let dmg = s.damage * might;
            let area = s.area * self.area_mult();
            let loud = every >= 0.3; // continuous streams don't announce every shot
            match w.kind {
                WeaponKind::Ofuda => {
                    // Seal Storm: a stream of homing seals that explode.
                    let d = self.aim_dir();
                    let a = d.y.atan2(d.x) + self.rng.range(-0.5, 0.5);
                    self.projectiles.push_kind(ProjKind::Seal, 6.0, p, Vec2::from_angle(a) * 380.0, 1.6, dmg * 0.9, 1, 10.0);
                    let last = self.projectiles.len() - 1;
                    self.projectiles.burst[last] = 28.0 * area;
                }
                WeaponKind::MoonOrbs => {
                    // Lunar Eclipse: six moonbeams sweep out from the player.
                    let rot = self.time * 0.9;
                    for k in 0..6 {
                        let a = rot + k as f32 * std::f32::consts::TAU / 6.0;
                        let to = p + Vec2::from_angle(a) * 340.0;
                        self.hurt_along(p, to, 12.0, dmg * 2.5);
                        self.events.push(Event::Beam { from: p, to, style: style::MOON });
                    }
                }
                WeaponKind::SakuraField => {
                    // Hanafubuki: a spiral stream of cutting petals.
                    for arm in 0..2 {
                        let a = self.time * 7.0 + arm as f32 * std::f32::consts::PI;
                        let v = Vec2::from_angle(a) * 210.0 + Vec2::from_angle(a + 1.4) * 60.0;
                        self.projectiles.push_kind(ProjKind::Petal, 5.0, p, v, 1.3, dmg * 1.4, 3, 0.0);
                    }
                }
                WeaponKind::ThunderChain => {
                    // Heavenfall: sky lightning on a random nearby enemy.
                    if let Some(t) = self.random_targets(320.0, 1).first().copied() {
                        self.blasts.push(Blast { pos: t, delay: 0.12, max_delay: 0.12, radius: 46.0 * area, damage: dmg * 1.4, style: style::SKYBOLT, burns: false });
                    }
                }
                WeaponKind::Taiko => {
                    // Festival Quake: a colossal shockwave.
                    self.waves.push(Wave { center: p, radius: 0.0, max_radius: 320.0 * area, speed: 420.0, damage: dmg * 1.8, purify: false });
                    self.events.push(Event::Burst { pos: p, radius: 60.0, style: style::QUAKE });
                }
                WeaponKind::StarFall => {
                    // Meteor: a giant meteor on the thickest part of the horde, leaving a crater.
                    if let Some(t) = self.random_targets(300.0, 1).first().copied() {
                        self.blasts.push(Blast { pos: t, delay: 0.7, max_delay: 0.7, radius: 110.0 * area, damage: dmg * 3.0, style: style::METEOR, burns: true });
                    }
                }
                WeaponKind::FlamePool => {
                    // Foxfire Trail: burning ground under the player's feet.
                    self.pools.push(Pool { pos: p, radius: 30.0 * area, life: 2.4, max_life: 2.4, tick: 0.0, damage: dmg });
                }
                WeaponKind::FoxfireOrb => {
                    // Comet Rain: five blue comets fall on the horde.
                    for t in self.random_targets(320.0, 5) {
                        let d = self.rng.range(0.4, 0.9);
                        self.blasts.push(Blast { pos: t, delay: d, max_delay: d, radius: 50.0 * area, damage: dmg * 1.3, style: style::COMET, burns: false });
                    }
                }
                WeaponKind::GaleShuriken => {
                    // Cyclone: a ring of shuriken bursts out and returns.
                    for k in 0..10 {
                        let a = k as f32 / 10.0 * std::f32::consts::TAU;
                        self.projectiles.push_kind(ProjKind::Shuriken, 9.0 * area, p, Vec2::from_angle(a) * 300.0, s.duration * self.persistence(), dmg, 255, 0.0);
                    }
                }
                WeaponKind::FoxMirage => {
                    // Mirage Barrage: every clone hurls a foxfire orb.
                    let clones: Vec<Vec2> = self.clones.iter().map(|c| c.pos).collect();
                    for c in clones {
                        if let Some(j) = self.nearest_index(c, 220.0, &[]) {
                            let d = (self.enemies.pos[j] - c).normalize_or_zero();
                            self.projectiles.push_kind(ProjKind::Foxfire, 6.0, c, d * 280.0, 1.2, dmg * 1.5, 1, 0.0);
                            let last = self.projectiles.len() - 1;
                            self.projectiles.burst[last] = 30.0 * area;
                        }
                    }
                }
                WeaponKind::GraveCleaver => {
                    // Judgement: a colossal slow crescent.
                    let d = self.aim_dir();
                    self.projectiles.push_kind(ProjKind::CrescentWave, 42.0 * area, p + d * 20.0, d * 230.0, 1.6, dmg * 3.0, 255, 0.0);
                    self.events.push(Event::Slash { pos: p, dir: d, radius: 90.0, full: true });
                }
                WeaponKind::LanternVigil => {
                    // Lantern Volley: every lantern fires a spirit bolt.
                    let lanterns: Vec<Vec2> = self.lanterns.iter().map(|l| l.pos).collect();
                    for l in lanterns {
                        if let Some(j) = self.nearest_index(l, 200.0, &[]) {
                            let d = (self.enemies.pos[j] - l).normalize_or_zero();
                            self.projectiles.push_kind(ProjKind::SpiritBolt, 5.0, l + Vec2::new(0.0, -12.0), d * 320.0, 0.9, dmg * 0.9, 2, 6.0);
                        }
                    }
                }
                WeaponKind::YinYangOrbs => {
                    // Yin-Yang Crash: a giant orb crashes onto the horde.
                    if let Some(t) = self.random_targets(300.0, 1).first().copied() {
                        self.blasts.push(Blast { pos: t, delay: 0.6, max_delay: 0.6, radius: 90.0 * area, damage: dmg * 4.0, style: style::YINYANG, burns: false });
                    }
                }
                WeaponKind::SpiritSeal => {
                    // Seal Ring: a full ring of seal orbs.
                    for k in 0..14 {
                        let a = k as f32 / 14.0 * std::f32::consts::TAU;
                        self.projectiles.push_kind(ProjKind::SealOrb, 7.0, p, Vec2::from_angle(a) * 300.0, 1.8, dmg, 1, 5.0);
                        let last = self.projectiles.len() - 1;
                        self.projectiles.burst[last] = 26.0 * area;
                    }
                }
                WeaponKind::GravityWell => {
                    // Event Horizon: a vast singularity.
                    if let Some(t) = self.random_targets(260.0, 1).first().copied() {
                        self.wells.push(Well { pos: t, life: 1.6, max_life: 1.6, radius: 170.0 * area, damage: dmg * 2.0, tick: 0.0 });
                    }
                }
                WeaponKind::RepulsionStar => {
                    // Red Nova: a ring of crimson stars.
                    for k in 0..10 {
                        let a = k as f32 / 10.0 * std::f32::consts::TAU;
                        self.projectiles.push_kind(ProjKind::RedStar, 10.0 * area, p, Vec2::from_angle(a) * 340.0, 1.3, dmg, 255, 0.0);
                    }
                }
                WeaponKind::VoidLance => {
                    // Void Cascade: three lances fan out.
                    let d = self.aim_dir();
                    for k in -1..=1 {
                        let a = d.y.atan2(d.x) + k as f32 * 0.55;
                        self.projectiles.push_kind(ProjKind::VoidOrb, 26.0 * area, p, Vec2::from_angle(a) * 170.0, 3.0, dmg * 1.3, 255, 0.0);
                    }
                }
                WeaponKind::CrimsonTendrils => {
                    // Hundred Lashes: twelve tendrils tear through everything around.
                    self.lash_targets(p, 160.0 * area, 12, dmg * 1.2);
                }
                WeaponKind::MukadeSpines => {
                    // Marching Spine: a line of spines crawls out along the aim.
                    let d = self.aim_dir();
                    for k in 0..9 {
                        let pos = p + d * (40.0 + 34.0 * k as f32);
                        let delay = 0.12 + 0.07 * k as f32;
                        self.blasts.push(Blast { pos, delay, max_delay: delay, radius: 34.0 * area, damage: dmg * 1.5, style: style::SPINE, burns: false });
                    }
                }
                WeaponKind::Devour => {
                    // Feeding Wave: a great draining shockwave that heals.
                    self.query_circle(p, 200.0 * area);
                    let n = self.scratch.len();
                    for idx in 0..n {
                        let j = self.scratch[idx] as usize;
                        let dir = p - self.enemies.pos[j];
                        self.hurt_enemy(j, dmg * 2.0, dir * 0.5);
                    }
                    self.heal((n as f32 * 0.5).min(15.0));
                    self.events.push(Event::Devour { pos: p, radius: 200.0 * area });
                }
                WeaponKind::VenomFang => {
                    // Fang Ring: a ring of homing fangs.
                    for k in 0..12 {
                        let a = k as f32 / 12.0 * std::f32::consts::TAU;
                        self.projectiles.push_kind(ProjKind::Fang, 6.0, p, Vec2::from_angle(a) * 280.0, 1.6, dmg, 1, 7.0);
                        let last = self.projectiles.len() - 1;
                        self.projectiles.burst[last] = 26.0 * area;
                    }
                }
                WeaponKind::IaiFlash => {
                    // Thousand Cuts: a radial storm of twelve cuts.
                    for k in 0..12 {
                        let a = k as f32 / 12.0 * std::f32::consts::TAU + self.time;
                        let to = p + Vec2::from_angle(a) * 300.0;
                        self.hurt_along(p, to, 12.0, dmg * 0.8);
                        self.events.push(Event::Iai { from: p, to });
                    }
                }
                _ => continue,
            }
            if loud {
                self.events.push(Event::Signature(w.kind));
            }
        }
        // Delayed blasts land.
        let mut i = 0;
        while i < self.blasts.len() {
            self.blasts[i].delay -= DT;
            if self.blasts[i].delay > 0.0 {
                i += 1;
                continue;
            }
            let b = self.blasts.swap_remove(i);
            self.query_circle(b.pos, b.radius);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let dir = self.enemies.pos[j] - b.pos;
                self.hurt_enemy(j, b.damage, dir);
            }
            if b.burns {
                self.pools.push(Pool { pos: b.pos, radius: b.radius * 0.6, life: 3.0, max_life: 3.0, tick: 0.0, damage: b.damage * 0.08 });
            }
            self.events.push(Event::Burst { pos: b.pos, radius: b.radius, style: b.style });
        }
    }

    /// Lashes up to `n` distinct nearby enemies with tendrils that pierce along the line.
    fn lash_targets(&mut self, from: Vec2, reach: f32, n: u32, dmg: f32) {
        let mut taken: Vec<u32> = Vec::new();
        for _ in 0..n {
            let Some(j) = self.nearest_index(from, reach, &taken) else { break };
            taken.push(self.enemies.id[j]);
            let to = self.enemies.pos[j];
            // Reach a little past the target so the lash pierces whatever is behind it.
            let tip = from + (to - from).normalize_or_zero() * ((to - from).length() + 20.0);
            self.hurt_along(from, tip, 8.0, dmg);
            self.events.push(Event::Lash { from, to: tip });
        }
    }

    fn start_divine(&mut self) {
        let p = self.player.pos;
        self.player.form = DIVINE_DURATION;
        self.player.iframes = BARRIER_IFRAMES;
        self.divine_ofuda = 0.0;
        self.divine_orb = 0.3;
        self.divine_ring = 0.5;
        self.divine_charging = false;
        self.shots.clear();
        self.events.push(Event::DivineAwakened { pos: p });
    }

    fn update_divine(&mut self) {
        self.barrier_cd = (self.barrier_cd - DT).max(0.0);
        if self.form_kind() != Some(SpecialKind::Divine) {
            return;
        }
        self.player.form -= DT;
        let p = self.player.pos;
        let power = self.fox_power();
        // Ofuda Storm: a stream of homing talismans in every direction.
        self.divine_ofuda -= DT;
        while self.divine_ofuda <= 0.0 {
            self.divine_ofuda += DIVINE_OFUDA_EVERY;
            for _ in 0..3 {
                let a = self.rng.range(0.0, std::f32::consts::TAU);
                self.projectiles.push_kind(ProjKind::Seal, 6.0, p, Vec2::from_angle(a) * 360.0, 1.6, DIVINE_OFUDA_DAMAGE * power, 1, 9.0);
                let last = self.projectiles.len() - 1;
                self.projectiles.burst[last] = 22.0;
            }
        }
        // Yin-Yang Cataclysm: giant orbs crash onto enemy groups.
        self.divine_orb -= DT;
        if self.divine_orb <= 0.0 {
            self.divine_orb = DIVINE_ORB_EVERY;
            for t in self.random_targets(340.0, 3) {
                let d = self.rng.range(0.35, 0.7);
                self.blasts.push(Blast { pos: t, delay: d, max_delay: d, radius: 80.0, damage: DIVINE_ORB_DAMAGE * power, style: style::YINYANG, burns: false });
            }
        }
        // Spirit-seal barrage: rings of orbs in every direction.
        self.divine_ring -= DT;
        if self.divine_ring <= 0.0 {
            self.divine_ring = DIVINE_RING_EVERY;
            let rot = self.time;
            for k in 0..18 {
                let a = rot + k as f32 / 18.0 * std::f32::consts::TAU;
                self.projectiles.push_kind(ProjKind::SealOrb, 8.0, p, Vec2::from_angle(a) * 260.0, 1.8, DIVINE_RING_DAMAGE * power, 1, 2.0);
                let last = self.projectiles.len() - 1;
                self.projectiles.burst[last] = 26.0;
            }
        }
        if !self.divine_charging && self.player.form <= DIVINE_FINALE_CHARGE {
            self.divine_charging = true;
            self.events.push(Event::DivineCharge { pos: p });
        }
        if self.player.form <= 0.0 {
            self.player.form = 0.0;
            // Final Purification.
            self.query_circle(p, DIVINE_FINALE_RADIUS);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let dir = self.enemies.pos[j] - p;
                self.hurt_enemy(j, DIVINE_FINALE_DAMAGE * power, dir);
                self.enemies.push[j] += dir.normalize_or_zero() * (450.0 * self.enemies.knock[j]);
            }
            self.shots.clear();
            self.events.push(Event::Burst { pos: p, radius: DIVINE_FINALE_RADIUS, style: style::PURIFY });
        }
    }

    fn start_domain(&mut self) {
        let p = self.player.pos;
        self.player.form = DOMAIN_DURATION;
        self.player.iframes = BARRIER_IFRAMES;
        self.domain_center = p;
        self.domain_charging = false;
        self.fox_tick = 0.0;
        self.shots.clear();
        self.events.push(Event::DomainOpened { pos: p });
    }

    fn update_domain(&mut self) {
        self.ward_cd = (self.ward_cd - DT).max(0.0);
        if self.form_kind() != Some(SpecialKind::Firmament) {
            return;
        }
        self.player.form -= DT;
        let c = self.domain_center;
        let power = self.fox_power();
        // Waves of spatial collapse over everything inside.
        self.fox_tick -= DT;
        if self.fox_tick <= 0.0 {
            self.fox_tick = DOMAIN_WAVE_EVERY;
            self.query_circle(c, DOMAIN_RADIUS);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let dir = self.enemies.pos[j] - c;
                self.hurt_enemy(j, DOMAIN_WAVE_DAMAGE * power, dir * 0.0);
            }
        }
        if !self.domain_charging && self.player.form <= DOMAIN_FINALE_CHARGE {
            self.domain_charging = true;
            self.events.push(Event::VoidCharge { pos: self.player.pos });
        }
        if self.player.form <= 0.0 {
            self.player.form = 0.0;
            // Void Lance: Maximum, erasing everything along its line.
            let p = self.player.pos;
            let d = self.aim_dir();
            self.projectiles.push_kind(ProjKind::VoidOrb, DOMAIN_FINALE_RADIUS, p, d * 260.0, 4.0, DOMAIN_FINALE_DAMAGE * power, 255, 0.0);
            self.events.push(Event::VoidMaximum { pos: p });
        }
    }

    fn start_mukade(&mut self) {
        let p = self.player.pos;
        self.player.form = MUKADE_DURATION;
        self.player.iframes = BARRIER_IFRAMES;
        self.fox_tick = 0.0;
        self.hannya_beam = 0.4; // eruption timer
        let dmg = MUKADE_BURST_DAMAGE * self.fox_power();
        self.query_circle(p, MUKADE_BURST_RADIUS);
        for idx in 0..self.scratch.len() {
            let j = self.scratch[idx] as usize;
            let dir = self.enemies.pos[j] - p;
            self.hurt_enemy(j, dmg, dir);
            self.enemies.push[j] += dir.normalize_or_zero() * (400.0 * self.enemies.knock[j]);
        }
        let r2 = MUKADE_BURST_RADIUS * MUKADE_BURST_RADIUS;
        self.shots.retain(|s| (s.pos - p).length_sq() > r2);
        self.events.push(Event::MukadeAwakened { pos: p });
        self.events.push(Event::Burst { pos: p, radius: MUKADE_BURST_RADIUS, style: style::FRENZY });
    }

    fn update_mukade(&mut self) {
        if self.form_kind() != Some(SpecialKind::Mukade) {
            return;
        }
        self.player.form -= DT;
        let power = self.fox_power();
        let p = self.player.pos;
        self.heal(MUKADE_REGEN * DT);
        // Tendril barrage.
        self.fox_tick -= DT;
        if self.fox_tick <= 0.0 {
            self.fox_tick = MUKADE_LASH_EVERY;
            let picks = self.random_targets(MUKADE_LASH_REACH, MUKADE_LASHES);
            for t in picks {
                let tip = p + (t - p).normalize_or_zero() * ((t - p).length() + 20.0);
                self.hurt_along(p, tip, 9.0, MUKADE_LASH_DAMAGE * power);
                self.events.push(Event::Lash { from: p, to: tip });
            }
        }
        // Centipede eruptions: a ring of segmented spines bursting from the ground.
        self.hannya_beam -= DT;
        if self.hannya_beam <= 0.0 {
            self.hannya_beam = MUKADE_ERUPT_EVERY;
            let rot = self.rng.range(0.0, std::f32::consts::TAU);
            for k in 0..8 {
                let a = rot + k as f32 / 8.0 * std::f32::consts::TAU;
                let r = 90.0 + 70.0 * (k % 2) as f32;
                let pos = p + Vec2::from_angle(a) * r;
                let d = 0.15 + 0.04 * k as f32;
                self.blasts.push(Blast { pos, delay: d, max_delay: d, radius: 48.0, damage: MUKADE_ERUPT_DAMAGE * power, style: style::SPINE, burns: false });
            }
        }
        if self.player.form <= 0.0 {
            self.player.form = 0.0;
            self.events.push(Event::FoxFaded);
        }
    }

    /// Spawns floor pickups over time, ticks buffs, and applies a pickup when touched.
    fn update_pickups(&mut self) {
        for t in [&mut self.speed_buff, &mut self.damage_buff, &mut self.freeze, &mut self.vacuum] {
            *t = (*t - DT).max(0.0);
        }
        if self.cfg.stress.is_some() {
            return;
        }
        if self.time >= self.next_pickup {
            self.next_pickup = self.time + self.rng.range(PICKUP_EVERY.0, PICKUP_EVERY.1);
            if self.pickups.len() < MAX_PICKUPS {
                let a = self.rng.range(0.0, std::f32::consts::TAU);
                let d = self.rng.range(220.0, 380.0);
                let pos = stage::resolve(self.cfg.stage, self.player.pos + Vec2::from_angle(a) * d, 12.0);
                let k = PickupKind::roll(self.rng.f32());
                self.pickups.push((pos, k));
                self.events.push(Event::PickupSpawned { pos, kind: k });
            }
        }
        let p = self.player.pos;
        let grab = PLAYER_RADIUS + 12.0;
        let Some(i) = self.pickups.iter().position(|(c, _)| (*c - p).length_sq() < grab * grab) else { return };
        let (pos, kind) = self.pickups.swap_remove(i);
        match kind {
            PickupKind::Feast => {
                let amount = self.player.max_hp * 0.35;
                self.player.hp = (self.player.hp + amount).min(self.player.max_hp);
            }
            PickupKind::SwiftGeta => self.speed_buff = BUFF_SECONDS,
            PickupKind::OniSake => self.damage_buff = BUFF_SECONDS,
            PickupKind::ShrineBell => self.vacuum = VACUUM_SECONDS,
            PickupKind::Hourglass => self.freeze = FREEZE_SECONDS,
            PickupKind::ExorcismSeal => {
                self.query_circle(p, EXORCISM_RADIUS);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    if !self.enemies.kind[j].is_boss() {
                        let dir = self.enemies.pos[j] - p;
                        let hp = self.enemies.hp[j];
                        self.hurt_enemy(j, hp + 1.0, dir);
                    }
                }
                self.shots.clear();
            }
            PickupKind::SenPurse => self.gain_sen(40),
        }
        self.events.push(Event::PickupTaken { pos, kind });
    }

    fn update_spirit_arts(&mut self) {
        let p = self.player.pos;
        // Gravity Wells: drag enemies to the centre, then collapse.
        let mut i = 0;
        while i < self.wells.len() {
            let mut wl = self.wells[i];
            wl.life -= DT;
            wl.tick -= DT;
            self.query_circle(wl.pos, wl.radius);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let to = wl.pos - self.enemies.pos[j];
                if !self.enemies.kind[j].is_boss() {
                    self.enemies.push[j] += to * (4.0 * DT * 60.0 / 10.0);
                }
            }
            if wl.tick <= 0.0 {
                wl.tick = 0.25;
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    self.hurt_enemy(j, wl.damage * 0.25, Vec2::ZERO);
                }
            }
            if wl.life <= 0.0 {
                self.query_circle(wl.pos, wl.radius * 0.8);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    self.hurt_enemy(j, wl.damage, Vec2::ZERO);
                }
                self.events.push(Event::WellCollapse { pos: wl.pos, radius: wl.radius });
                self.wells.swap_remove(i);
                continue;
            }
            self.wells[i] = wl;
            i += 1;
        }
        // Soul Tether: drain and drag.
        let mut i = 0;
        while i < self.tethers.len() {
            let mut t = self.tethers[i];
            t.life -= DT;
            t.tick -= DT;
            let j = self.enemies.id.iter().position(|id| *id == t.enemy);
            let alive = j.filter(|&j| self.enemies.hp[j] > 0.0 && t.life > 0.0 && (self.enemies.pos[j] - p).length() < t.reach);
            let Some(j) = alive else {
                // Endless River: the chain leaps from a dead enemy to the next one and heals.
                if t.heals && t.life > 0.0 {
                    let from = j.map_or(p, |j| self.enemies.pos[j]);
                    let taken: Vec<u32> = self.tethers.iter().map(|t| t.enemy).collect();
                    if let Some(n) = self.nearest_index(from, 140.0, &taken) {
                        self.heal(1.0);
                        self.tethers[i].enemy = self.enemies.id[n];
                        i += 1;
                        continue;
                    }
                }
                self.tethers.swap_remove(i);
                continue;
            };
            let to_player = p - self.enemies.pos[j];
            self.enemies.push[j] += to_player.normalize_or_zero() * 9.0 * self.enemies.knock[j].max(0.2); // ~60 px/s drag once push decay settles
            if t.tick <= 0.0 {
                t.tick = 0.25;
                self.hurt_enemy(j, t.damage, to_player);
                if t.heals && self.enemies.hp[j] <= 0.0 {
                    self.heal(1.0);
                }
            }
            self.tethers[i] = t;
            i += 1;
        }
        // Lantern Vigil: pulse damage around each lantern.
        let mut i = 0;
        while i < self.lanterns.len() {
            let mut l = self.lanterns[i];
            l.life -= DT;
            l.tick -= DT;
            if l.life <= 0.0 {
                self.lanterns.swap_remove(i);
                continue;
            }
            if l.tick <= 0.0 {
                l.tick = 0.6;
                self.query_circle(l.pos, l.radius);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    let dir = self.enemies.pos[j] - l.pos;
                    self.hurt_enemy(j, l.damage, dir);
                    if l.slows {
                        self.enemies.push[j] = self.enemies.push[j] * 0.5 - self.enemies.vel[j] * 0.3;
                    }
                }
                self.events.push(Event::LanternPulse { pos: l.pos, radius: l.radius });
            }
            self.lanterns[i] = l;
            i += 1;
        }
        // Moonsever scars keep cutting.
        let mut i = 0;
        while i < self.scars.len() {
            let mut s = self.scars[i];
            s.life -= DT;
            s.tick -= DT;
            if s.life <= 0.0 {
                self.scars.swap_remove(i);
                continue;
            }
            if s.tick <= 0.0 {
                s.tick = 0.3;
                self.hurt_along(s.from, s.to, 10.0, s.damage);
            }
            self.scars[i] = s;
            i += 1;
        }
    }

    fn update_clones(&mut self) {
        let mut i = 0;
        while i < self.clones.len() {
            let mut c = self.clones[i];
            c.life -= DT;
            c.iframes = (c.iframes - DT).max(0.0);
            c.strike_cd -= DT;
            c.striking = (c.striking - DT).max(0.0);
            // Take hits from enemies touching it.
            if c.iframes <= 0.0 {
                let touched = self.nearest_index(c.pos, 8.0 + MAX_ENEMY_RADIUS, &[]).is_some_and(|j| {
                    (self.enemies.pos[j] - c.pos).length() <= 8.0 + self.enemies.radius[j]
                });
                if touched {
                    c.hits = c.hits.saturating_sub(1);
                    c.iframes = 0.4;
                }
            }
            if c.life <= 0.0 || c.hits == 0 {
                // Burst into smoke that hurts everything around it.
                self.query_circle(c.pos, c.pop_radius);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    let dir = self.enemies.pos[j] - c.pos;
                    self.hurt_enemy(j, c.damage * 2.0, dir);
                }
                self.events.push(Event::ClonePopped { pos: c.pos, radius: c.pop_radius });
                self.clones.swap_remove(i);
                continue;
            }
            // Chase the nearest enemy near the player; otherwise hold a slot beside the player.
            let target = self.nearest_index(c.pos, 170.0, &[]).map(|j| self.enemies.pos[j]);
            let leash = (c.pos - self.player.pos).length() > 220.0;
            let goal = match target {
                Some(t) if !leash => t,
                _ => self.player.pos + Vec2::from_angle(c.slot + self.time * 0.5) * 34.0,
            };
            let to = goal - c.pos;
            let dist = to.length();
            c.moving = dist > 12.0;
            if c.moving {
                c.facing = to.normalize_or_zero();
                c.pos += c.facing * (c.speed.min(dist / DT) * DT);
            }
            c.pos = stage::resolve_local(self.cfg.stage, c.pos, 6.0);
            if target.is_some() && !leash && dist < 22.0 && c.strike_cd <= 0.0 {
                c.strike_cd = 0.45;
                c.striking = 0.15;
                let hit = c.pos + c.facing * 10.0;
                self.query_circle(hit, 20.0);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    self.hurt_enemy(j, c.damage, c.facing);
                }
                self.events.push(Event::CloneStrike { pos: hit });
            }
            self.clones[i] = c;
            i += 1;
        }
    }

    /// Ninefold Foxfire's power: scales with Might and level so it stays an endgame threat.
    fn fox_power(&self) -> f32 {
        self.might() * (1.0 + self.player.level as f32 / 40.0)
    }

    /// The nine tails as (root, tip) segments. Shared with the renderer so hits match visuals.
    pub fn fox_tails(&self) -> [(Vec2, Vec2); FOX_TAILS] {
        let t = self.time;
        let root = self.player.pos + Vec2::new(0.0, -6.0);
        let mut out = [(root, root); FOX_TAILS];
        for (k, seg) in out.iter_mut().enumerate() {
            let kf = k as f32;
            let a = t * 0.7 + kf * std::f32::consts::TAU / FOX_TAILS as f32 + 0.35 * (t * 2.3 + kf * 1.7).sin();
            let lash = 0.5 + 0.5 * (t * 5.0 + kf * 2.1).sin();
            let reach = FOX_TAIL_REACH.0 + (FOX_TAIL_REACH.1 - FOX_TAIL_REACH.0) * lash;
            *seg = (root, root + Vec2::from_angle(a) * reach);
        }
        out
    }

    fn start_fox(&mut self) {
        let p = self.player.pos;
        self.player.form = FOX_DURATION;
        self.player.iframes = BARRIER_IFRAMES;
        self.fox_tick = 0.0;
        self.fox_volley = 0.4;
        let dmg = FOX_BLAST_DAMAGE * self.fox_power();
        self.query_circle(p, FOX_BLAST_RADIUS);
        for idx in 0..self.scratch.len() {
            let j = self.scratch[idx] as usize;
            let dir = self.enemies.pos[j] - p;
            self.hurt_enemy(j, dmg, dir);
            self.enemies.push[j] += dir.normalize_or_zero() * (420.0 * self.enemies.knock[j]);
        }
        let r2 = FOX_BLAST_RADIUS * FOX_BLAST_RADIUS;
        self.shots.retain(|s| (s.pos - p).length_sq() > r2);
        self.events.push(Event::FoxAwakened { pos: p });
    }

    /// Everything the minimap should show, nearest-first not guaranteed.
    pub fn map_markers(&self) -> Vec<MapMarker> {
        let e = &self.enemies;
        let mut out = Vec::with_capacity(e.len() + 16);
        for i in 0..e.len() {
            let kind = match e.kind[i].rank() {
                Rank::Final | Rank::Boss => MarkerKind::Boss,
                Rank::Mini => MarkerKind::MiniBoss,
                Rank::Elite => MarkerKind::Elite,
                Rank::Fodder => MarkerKind::Enemy,
            };
            out.push(MapMarker { pos: e.pos[i], kind });
        }
        out.extend(self.chests.iter().map(|(p, t)| MapMarker { pos: *p, kind: MarkerKind::Chest { tier: *t } }));
        out.extend(self.shrine.map(|p| MapMarker { pos: p, kind: MarkerKind::Shrine }));
        out.extend(self.pickups.iter().map(|(p, k)| MapMarker { pos: *p, kind: MarkerKind::Pickup(*k) }));
        out
    }

    /// Solid obstacles overlapping the area (min, max), as (min, max) boxes, for the current
    /// stage. The minimap only asks this, so new stages show up without UI changes.
    pub fn map_obstacles(&self, min: Vec2, max: Vec2) -> Vec<(Vec2, Vec2)> {
        let (cx0, cy0) = stage::chunk_of(min);
        let (cx1, cy1) = stage::chunk_of(max);
        let mut out = Vec::new();
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let (props, n) = stage::chunk_props(self.cfg.stage, cx, cy);
                out.extend(props[..n].iter().map(|p| p.footprint()));
            }
        }
        out
    }

    /// Playable area for bounded stages (min, max); None = endless (stage 1 streets).
    pub fn map_bounds(&self) -> Option<(Vec2, Vec2)> {
        None
    }

    /// The active transformation, if any.
    pub fn form_kind(&self) -> Option<SpecialKind> {
        (self.player.form > 0.0).then(|| hero_def(self.cfg.hero).special)
    }

    fn start_hannya(&mut self) {
        let p = self.player.pos;
        self.player.form = HANNYA_DURATION;
        self.player.iframes = BARRIER_IFRAMES;
        self.fox_tick = 0.0;
        self.hannya_beam = 0.3;
        self.hannya_wave = 0.1;
        self.hannya_step = 0.9;
        self.finale_charging = false;
        let dmg = HANNYA_ONSET_DAMAGE * self.fox_power();
        self.query_circle(p, HANNYA_ONSET_RADIUS);
        for idx in 0..self.scratch.len() {
            let j = self.scratch[idx] as usize;
            let dir = self.enemies.pos[j] - p;
            self.hurt_enemy(j, dmg, dir);
            self.enemies.push[j] += dir.normalize_or_zero() * (380.0 * self.enemies.knock[j]);
        }
        let r2 = HANNYA_ONSET_RADIUS * HANNYA_ONSET_RADIUS;
        self.shots.retain(|s| (s.pos - p).length_sq() > r2);
        self.events.push(Event::HannyaAwakened { pos: p });
    }

    /// Damages every enemy within `width` of the segment once.
    fn hurt_along(&mut self, from: Vec2, to: Vec2, width: f32, dmg: f32) {
        let d = to - from;
        let len = d.length().max(1.0);
        let steps = (len / 40.0).ceil() as usize;
        self.dead.clear();
        for k in 0..=steps {
            let c = from + d * (k as f32 / steps as f32);
            self.query_circle(c, 30.0 + width);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let e = self.enemies.pos[j];
                let t = ((e - from).dot_with(d) / (len * len)).clamp(0.0, 1.0);
                if (from + d * t - e).length() <= width + self.enemies.radius[j] && !self.dead.contains(&j) {
                    self.dead.push(j);
                    self.hurt_enemy(j, dmg, d);
                }
            }
        }
        self.dead.clear();
    }

    fn update_hannya(&mut self) {
        // Afterimages explode even if the form just ended.
        let mut k = 0;
        while k < self.afterimages.len() {
            self.afterimages[k].1 -= DT;
            if self.afterimages[k].1 <= 0.0 {
                let (c, _) = self.afterimages.swap_remove(k);
                let dmg = HANNYA_AFTERIMAGE_DAMAGE * self.fox_power();
                self.query_circle(c, HANNYA_AFTERIMAGE_RADIUS);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    let dir = self.enemies.pos[j] - c;
                    self.hurt_enemy(j, dmg, dir);
                }
                self.events.push(Event::AfterimageBlast { pos: c, radius: HANNYA_AFTERIMAGE_RADIUS });
            } else {
                k += 1;
            }
        }
        if self.form_kind() != Some(SpecialKind::Hannya) {
            return;
        }
        self.player.form -= DT;
        let power = self.fox_power();
        let p = self.player.pos;
        // Dread Aura.
        self.fox_tick -= DT;
        if self.fox_tick <= 0.0 {
            self.fox_tick = FOX_TICK;
            self.query_circle(p, HANNYA_AURA_RADIUS);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let dir = self.enemies.pos[j] - p;
                self.hurt_enemy(j, HANNYA_AURA_DAMAGE * power, dir);
            }
        }
        // Rending Beams in every direction.
        self.hannya_beam -= DT;
        if self.hannya_beam <= 0.0 {
            self.hannya_beam = HANNYA_BEAM_EVERY;
            self.beam_rot += 0.37;
            for b in 0..HANNYA_BEAMS {
                let a = self.beam_rot + b as f32 / HANNYA_BEAMS as f32 * std::f32::consts::TAU;
                let to = p + Vec2::from_angle(a) * HANNYA_BEAM_LENGTH;
                self.hurt_along(p, to, 9.0, HANNYA_BEAM_DAMAGE * power);
                self.events.push(Event::Beam { from: p, to, style: style::CRIMSON });
            }
        }
        // Crescent waves at the nearest enemy.
        self.hannya_wave -= DT;
        if self.hannya_wave <= 0.0 {
            self.hannya_wave = HANNYA_WAVE_EVERY;
            let d = self.aim_dir();
            self.projectiles.push_kind(ProjKind::CrescentWave, 18.0, p, d * 360.0, 0.8, HANNYA_WAVE_DAMAGE * power, 255, 0.0);
            self.events.push(Event::Slash { pos: p, dir: d, radius: 46.0, full: false });
        }
        // Shadow Step: a crimson afterimage dashes into the horde and explodes there.
        // (The player stays put: teleporting the player itself was disorienting.)
        self.hannya_step -= DT;
        if self.hannya_step <= 0.0 {
            self.hannya_step = HANNYA_STEP_EVERY;
            for t in self.random_targets(260.0, 2) {
                self.afterimages.push((t, 0.35));
                self.events.push(Event::ShadowStep { from: p, to: t });
            }
        }
        // Hannya's Wrath: charge, then a giant blast as the form ends.
        if !self.finale_charging && self.player.form <= HANNYA_FINALE_CHARGE {
            self.finale_charging = true;
            self.events.push(Event::FinaleCharge { pos: self.player.pos });
        }
        if self.player.form <= 0.0 {
            self.player.form = 0.0;
            let c = self.player.pos;
            self.query_circle(c, HANNYA_FINALE_RADIUS);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let dir = self.enemies.pos[j] - c;
                self.hurt_enemy(j, HANNYA_FINALE_DAMAGE * power, dir);
                self.enemies.push[j] += dir.normalize_or_zero() * (450.0 * self.enemies.knock[j]);
            }
            self.shots.clear();
            self.events.push(Event::FinaleBlast { pos: c, radius: HANNYA_FINALE_RADIUS });
        }
    }

    fn update_fox(&mut self) {
        if self.form_kind() != Some(SpecialKind::NineTails) {
            return;
        }
        self.player.form -= DT;
        let power = self.fox_power();
        let p = self.player.pos;
        // Tails + aura tick.
        self.fox_tick -= DT;
        if self.fox_tick <= 0.0 {
            self.fox_tick = FOX_TICK;
            self.dead.clear(); // reused as "already hit this tick"
            for (root, tip) in self.fox_tails() {
                let mid = (root + tip) * 0.5;
                let half = (tip - root).length() * 0.5;
                self.query_circle(mid, half + 10.0);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    let e = self.enemies.pos[j];
                    // Distance from the enemy to the tail segment.
                    let d = tip - root;
                    let t = (((e - root).dot_with(d)) / d.length_sq().max(1e-6)).clamp(0.0, 1.0);
                    let close = (root + d * t - e).length() <= 9.0 + self.enemies.radius[j];
                    if close && !self.dead.contains(&j) {
                        self.dead.push(j);
                        self.hurt_enemy(j, FOX_TAIL_DAMAGE * power, e - p);
                    }
                }
            }
            self.query_circle(p, FOX_AURA_RADIUS);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let dir = self.enemies.pos[j] - p;
                self.hurt_enemy(j, FOX_AURA_DAMAGE * power, dir);
            }
            self.dead.clear();
        }
        // Golden orb volleys in every direction.
        self.fox_volley -= DT;
        if self.fox_volley <= 0.0 {
            self.fox_volley = FOX_ORB_EVERY;
            self.fox_volleys += 1;
            let rot = self.fox_volleys as f32 * 0.31;
            for k in 0..FOX_ORBS {
                let a = rot + k as f32 / FOX_ORBS as f32 * std::f32::consts::TAU;
                self.projectiles.push_kind(ProjKind::GoldenOrb, 13.0, p, Vec2::from_angle(a) * 250.0, 1.8, FOX_ORB_DAMAGE * power, 255, 0.0);
            }
            self.events.push(Event::FoxVolley);
        }
        if self.player.form <= 0.0 {
            self.player.form = 0.0;
            self.events.push(Event::FoxFaded);
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
            if self.player.form <= 0.0 {
                self.player.special = (self.player.special + self.mods.special_rate / SPECIAL_KILLS).min(1.0);
            }
            self.events.push(Event::Kill { pos, kind });
            self.kills_by_kind[kind.index()] += 1;
            if self.cfg.stress.is_none() {
                let coin = match coin_drop(kind) {
                    0 if self.rng.f32() < 0.004 => 1,
                    c => c,
                };
                let coin = (coin as f32 * self.cfg.difficulty.greed()).round() as u32;
                if coin > 0 {
                    self.coins.push((pos, coin));
                }
            }
            if kind.rank() == Rank::Elite && self.cfg.stress.is_none() && self.rng.f32() < ONI_PICKUP_CHANCE && self.pickups.len() < MAX_PICKUPS {
                let k = PickupKind::roll(self.rng.f32());
                self.pickups.push((pos, k));
                self.events.push(Event::PickupSpawned { pos, kind: k });
            }
            match kind.rank() {
                Rank::Mini => {
                    self.bosses_down |= 1;
                    self.great_oni_defeated |= kind == EnemyKind::GreatOni;
                }
                Rank::Boss => self.bosses_down |= 2,
                Rank::Final => {
                    self.bosses_down |= 4;
                    if self.state == RunState::Playing {
                        self.state = RunState::Won;
                    }
                }
                _ => {}
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
                WeaponKind::YinYangOrbs => {
                    let phase = w.phase + s.speed * DT;
                    self.weapons[wi].phase = phase;
                    let rehit = (s.cooldown * 60.0) as u8;
                    let r = 9.0 * s.area * self.area_mult();
                    let current = self.weapons[wi];
                    for c in self.yinyang_positions(&current) {
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
                | WeaponKind::FlamePool
                | WeaponKind::FoxfireOrb
                | WeaponKind::Familiars
                | WeaponKind::GaleShuriken
                | WeaponKind::FoxMirage
                | WeaponKind::GraveCleaver
                | WeaponKind::SoulTether
                | WeaponKind::LanternVigil
                | WeaponKind::IaiFlash
                | WeaponKind::CrimsonTendrils
                | WeaponKind::MukadeSpines
                | WeaponKind::Devour
                | WeaponKind::VenomFang
                | WeaponKind::GravityWell
                | WeaponKind::RepulsionStar
                | WeaponKind::VoidLance
                | WeaponKind::SpiritSeal
                | WeaponKind::PurificationWave => {
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
                    self.waves.push(Wave { center: p, radius: start, max_radius: 135.0 * area, speed: s.speed, damage: dmg, purify: false });
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
                        // Moonlit Waves: eight crescents fly out of the full-circle cut.
                        for k in 0..8 {
                            let wd = Vec2::from_angle(k as f32 * std::f32::consts::FRAC_PI_4 + self.time);
                            self.projectiles.push_kind(ProjKind::CrescentWave, 14.0 * area, p + wd * 24.0, wd * 330.0, 0.8, dmg * 0.7, 255, 0.0);
                        }
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
            WeaponKind::FoxfireOrb => {
                self.events.push(Event::Fired(WeaponKind::FoxfireOrb));
                let kind = if w.awakened { ProjKind::Comet } else { ProjKind::Foxfire };
                let (vel, life) = (self.player.facing * s.speed, 2.0 * self.persistence());
                for n in 0..s.amount {
                    let pr = &mut self.projectiles;
                    pr.push_kind(kind, 7.0, p, vel, life, dmg, 1, 0.0);
                    let last = pr.len() - 1;
                    // Extra orbs gather one after another.
                    pr.charge[last] = s.duration * (1.0 + 0.4 * n as f32);
                    pr.burst[last] = 34.0 * area;
                }
            }
            WeaponKind::Familiars => {
                self.events.push(Event::Fired(WeaponKind::Familiars));
                let life = s.duration * self.persistence();
                for n in 0..s.amount {
                    let a = self.rng.range(0.0, std::f32::consts::TAU) + n as f32;
                    self.projectiles.push_kind(ProjKind::Familiar, 6.0, p, Vec2::from_angle(a) * s.speed, life, dmg, s.pierce, 6.0);
                    if w.awakened {
                        // Kitsune Blaze: each spirit explodes in foxfire when it strikes.
                        let last = self.projectiles.len() - 1;
                        self.projectiles.burst[last] = 30.0 * area;
                        self.projectiles.damage[last] = dmg * 1.6;
                    }
                }
            }
            WeaponKind::GraveCleaver => {
                self.events.push(Event::Fired(WeaponKind::GraveCleaver));
                let radius = 66.0 * area;
                let dir = self.aim_dir();
                let cone = 72f32.to_radians().cos();
                for n in 0..s.amount {
                    // Alternate swings: forward, behind, forward again.
                    let turn = if n % 2 == 1 { std::f32::consts::PI } else { 0.0 };
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
                    // The cut looses a crescent wave (both ways once Awakened).
                    let waves: &[f32] = if w.awakened { &[0.0, std::f32::consts::PI] } else { &[0.0] };
                    for t in waves {
                        let wd = Vec2::from_angle(d.y.atan2(d.x) + t);
                        self.projectiles.push_kind(ProjKind::CrescentWave, 13.0 * area, p + wd * 20.0, wd * s.speed, 0.7, dmg * 0.6, 255, 0.0);
                    }
                    if w.awakened {
                        break;
                    }
                }
            }
            WeaponKind::SoulTether => {
                self.events.push(Event::Fired(WeaponKind::SoulTether));
                let reach = 170.0 * area;
                let mut taken: Vec<u32> = self.tethers.iter().map(|t| t.enemy).collect();
                for _ in 0..s.amount {
                    let Some(j) = self.nearest_index(p, reach, &taken) else { break };
                    let id = self.enemies.id[j];
                    taken.push(id);
                    self.tethers.push(Tether { enemy: id, life: s.duration * self.persistence(), tick: 0.0, damage: dmg, reach: reach * 1.3, heals: w.awakened });
                }
            }
            WeaponKind::LanternVigil => {
                self.events.push(Event::Fired(WeaponKind::LanternVigil));
                let life = s.duration * self.persistence();
                for _ in 0..s.amount {
                    let a = self.rng.range(0.0, std::f32::consts::TAU);
                    let d = self.rng.range(30.0, 110.0);
                    let pos = stage::resolve(self.cfg.stage, p + Vec2::from_angle(a) * d, 8.0);
                    self.lanterns.push(Lantern { pos, life, max_life: life, radius: 46.0 * area, tick: 0.2, damage: dmg, slows: w.awakened });
                }
            }
            WeaponKind::IaiFlash => {
                self.events.push(Event::Fired(WeaponKind::IaiFlash));
                let len = 210.0 * area;
                let base = self.aim_dir();
                let n = s.amount as f32;
                for k in 0..s.amount {
                    let a = base.y.atan2(base.x) + (k as f32 - (n - 1.0) / 2.0) * 0.45;
                    let to = p + Vec2::from_angle(a) * len;
                    self.hurt_along(p, to, 12.0, dmg);
                    self.events.push(Event::Iai { from: p, to });
                    if w.awakened {
                        self.scars.push(Scar { from: p, to, life: 1.5, tick: 0.3, damage: dmg * 0.25 });
                    }
                }
            }
            WeaponKind::CrimsonTendrils => {
                self.events.push(Event::Fired(WeaponKind::CrimsonTendrils));
                let reach = 120.0 * area;
                self.lash_targets(p, reach, s.amount, dmg);
            }
            WeaponKind::MukadeSpines => {
                self.events.push(Event::Fired(WeaponKind::MukadeSpines));
                for (k, t) in self.random_targets(240.0, s.amount).into_iter().enumerate() {
                    let d = 0.3 + 0.06 * k as f32;
                    self.blasts.push(Blast { pos: t, delay: d, max_delay: d, radius: 30.0 * area, damage: dmg, style: style::SPINE, burns: false });
                }
            }
            WeaponKind::Devour => {
                let r = 70.0 * area;
                self.query_circle(p, r);
                let n = self.scratch.len();
                for idx in 0..n {
                    let j = self.scratch[idx] as usize;
                    let dir = p - self.enemies.pos[j];
                    self.hurt_enemy(j, dmg, dir * 0.3);
                }
                let per = if w.awakened { 1.0 } else { 0.5 };
                self.heal((n as f32 * per).min(8.0));
                self.events.push(Event::Devour { pos: p, radius: r });
            }
            WeaponKind::VenomFang => {
                self.events.push(Event::Fired(WeaponKind::VenomFang));
                let d = self.aim_dir();
                let n = s.amount as f32;
                for k in 0..s.amount {
                    let a = d.y.atan2(d.x) + (k as f32 - (n - 1.0) / 2.0) * 0.35;
                    self.projectiles.push_kind(ProjKind::Fang, 6.0, p, Vec2::from_angle(a) * s.speed, 1.6, dmg, 1, 7.0);
                    let last = self.projectiles.len() - 1;
                    self.projectiles.burst[last] = 26.0 * area;
                }
            }
            WeaponKind::SpiritSeal => {
                self.events.push(Event::Fired(WeaponKind::SpiritSeal));
                let n = s.amount;
                for k in 0..n {
                    let slot = k as f32 / n as f32 * std::f32::consts::TAU;
                    self.projectiles.push_kind(ProjKind::SealOrb, 7.0, p, Vec2::from_angle(slot) * s.speed, 2.0, dmg, 1, 8.0);
                    let last = self.projectiles.len() - 1;
                    self.projectiles.charge[last] = s.duration + 0.05 * k as f32;
                    self.projectiles.burst[last] = 26.0 * area;
                }
            }
            WeaponKind::PurificationWave => {
                self.events.push(Event::Fired(WeaponKind::PurificationWave));
                self.waves.push(Wave { center: p, radius: 0.0, max_radius: 150.0 * area, speed: s.speed, damage: dmg, purify: true });
                if w.awakened {
                    self.waves.push(Wave { center: p, radius: -70.0, max_radius: 230.0 * area, speed: s.speed * 1.1, damage: dmg * 1.4, purify: true });
                }
            }
            WeaponKind::GravityWell => {
                self.events.push(Event::Fired(WeaponKind::GravityWell));
                for t in self.random_targets(260.0, s.amount) {
                    self.wells.push(Well { pos: t, life: s.duration, max_life: s.duration, radius: 70.0 * area, damage: dmg, tick: 0.0 });
                }
            }
            WeaponKind::RepulsionStar => {
                self.events.push(Event::Fired(WeaponKind::RepulsionStar));
                let d = self.aim_dir();
                let n = s.amount as f32;
                for k in 0..s.amount {
                    let a = d.y.atan2(d.x) + (k as f32 - (n - 1.0) / 2.0) * 0.3;
                    self.projectiles.push_kind(ProjKind::RedStar, 10.0 * area, p, Vec2::from_angle(a) * s.speed, 1.4, dmg, 255, 0.0);
                }
            }
            WeaponKind::VoidLance => {
                self.events.push(Event::Fired(WeaponKind::VoidLance));
                let d = self.aim_dir();
                let n = s.amount as f32;
                for k in 0..s.amount {
                    let a = d.y.atan2(d.x) + (k as f32 - (n - 1.0) / 2.0) * 0.6;
                    self.projectiles.push_kind(ProjKind::VoidOrb, 22.0 * area, p, Vec2::from_angle(a) * s.speed, 3.0, dmg, 255, 0.0);
                }
            }
            WeaponKind::FoxMirage => {
                self.events.push(Event::Fired(WeaponKind::FoxMirage));
                let n = s.amount;
                for k in 0..n {
                    let slot = k as f32 / n as f32 * std::f32::consts::TAU;
                    let pos = stage::resolve(self.cfg.stage, p + Vec2::from_angle(slot) * 18.0, 6.0);
                    self.clones.push(MirageClone {
                        pos,
                        facing: self.player.facing,
                        moving: false,
                        life: s.duration * self.persistence(),
                        hits: s.pierce,
                        iframes: 0.5,
                        strike_cd: 0.3,
                        striking: 0.0,
                        damage: dmg,
                        pop_radius: 40.0 * area,
                        speed: s.speed,
                        slot,
                    });
                    self.events.push(Event::CloneSummoned { pos });
                }
            }
            WeaponKind::GaleShuriken => {
                self.events.push(Event::Fired(WeaponKind::GaleShuriken));
                let dir = match self.nearest_enemy(320.0) {
                    Some(e) => (e - p).normalize_or_zero(),
                    None => self.player.facing,
                };
                let base = dir.y.atan2(dir.x);
                let n = s.amount as f32;
                for k in 0..s.amount {
                    let a = base + (k as f32 - (n - 1.0) / 2.0) * 0.5;
                    self.projectiles.push_kind(ProjKind::Shuriken, 10.0 * area, p, Vec2::from_angle(a) * s.speed, s.duration * self.persistence(), dmg, 255, 0.0);
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
                        if wv.purify {
                            self.enemies.push[j] += off.normalize_or_zero() * (380.0 * self.enemies.knock[j].max(0.3));
                        }
                    }
                }
                if wv.purify {
                    let (c, r) = (wv.center, wv.radius);
                    self.shots.retain(|s| ((s.pos - c).length() - r).abs() > 14.0);
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
            // Charging in the hand: follow the player, then launch at the nearest enemy.
            if self.projectiles.charge[i] > 0.0 {
                self.projectiles.charge[i] -= DT;
                if self.projectiles.kind[i] == ProjKind::SealOrb {
                    // Gather in a ring around the player (the slot angle rides in the velocity).
                    let v = self.projectiles.vel[i];
                    let a = v.y.atan2(v.x) + self.time * 5.0;
                    self.projectiles.pos[i] = self.player.pos + Vec2::new(0.0, -8.0) + Vec2::from_angle(a) * 30.0;
                } else {
                    let hand = Vec2::new(if self.player.facing.x < 0.0 { -9.0 } else { 9.0 }, -6.0);
                    self.projectiles.pos[i] = self.player.pos + hand;
                }
                if self.projectiles.charge[i] <= 0.0 {
                    let speed = self.projectiles.vel[i].length().max(1.0);
                    let dir = match self.nearest_enemy(380.0) {
                        Some(e) => (e - self.projectiles.pos[i]).normalize_or_zero(),
                        None => self.player.facing,
                    };
                    self.projectiles.vel[i] = dir * speed;
                }
                i += 1;
                continue;
            }
            if self.projectiles.kind[i] == ProjKind::Shuriken {
                let pr = &mut self.projectiles;
                if pr.life[i] < pr.max_life[i] * 0.5 {
                    // Coming back: home on the player; caught when close.
                    let to = self.player.pos - pr.pos[i];
                    let speed = pr.vel[i].length();
                    pr.vel[i] = to.normalize_or_zero() * speed;
                    if to.length_sq() < 14.0 * 14.0 {
                        pr.swap_remove(i);
                        continue;
                    }
                    pr.last_hit[i] = u32::MAX; // may hit the same enemy again on the way back
                } else {
                    pr.vel[i] = pr.vel[i] * 0.985; // slows toward the turn
                }
            }
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
            self.query_circle(p, self.projectiles.radius[i]);
            let mut pierce = self.projectiles.pierce[i];
            let bursts = self.projectiles.burst[i] > 0.0;
            for idx in 0..self.scratch.len() {
                if pierce == 0 {
                    break;
                }
                let j = self.scratch[idx] as usize;
                if self.enemies.id[j] == self.projectiles.last_hit[i] || self.enemies.hp[j] <= 0.0 {
                    continue;
                }
                if pierce == 255 {
                    if self.enemies.pierce_cd[j] > 0 {
                        continue;
                    }
                    self.enemies.pierce_cd[j] = 20; // 1/3 s
                }
                self.projectiles.last_hit[i] = self.enemies.id[j];
                if bursts {
                    pierce = 0; // the burst below does the damage
                    break;
                }
                let (dmg, vel) = (self.projectiles.damage[i], self.projectiles.vel[i]);
                self.hurt_enemy(j, dmg, vel);
                if self.projectiles.kind[i] == ProjKind::RedStar {
                    self.enemies.push[j] += vel.normalize_or_zero() * (420.0 * self.enemies.knock[j].max(0.3));
                }
                pierce = pierce.saturating_sub(if pierce == 255 { 0 } else { 1 });
            }
            self.projectiles.pierce[i] = pierce;
            if pierce == 0 || self.projectiles.life[i] <= 0.0 {
                if bursts {
                    self.burst_projectile(i);
                }
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
            if self.vacuum > 0.0 {
                self.gems.pos[i] += d.normalize_or_zero() * ((560.0 * DT).min(d2.sqrt()));
            } else if d2 < magnet * magnet {
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
            if self.cfg.hero == HeroId::Ren {
                // Barrier: blocks a fireball outright when charged; otherwise softens it.
                if self.barrier_cd <= 0.0 {
                    self.barrier_cd = barrier_recharge(self.player.level);
                    self.events.push(Event::BarrierBlock);
                    return;
                }
                self.hurt_player(SHOT_DAMAGE * self.enemy_damage_scale() * BARRIER_SHOT_DAMAGE);
                return;
            }
            self.hurt_player(SHOT_DAMAGE * self.enemy_damage_scale());
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
        self.chests_opened += 1;
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
        if self.player.special < 1.0 || self.player.form > 0.0 {
            return;
        }
        self.player.special = 0.0;
        self.specials_used += 1;
        match hero_def(self.cfg.hero).special {
            SpecialKind::NineTails => return self.start_fox(),
            SpecialKind::Hannya => return self.start_hannya(),
            SpecialKind::Mukade => return self.start_mukade(),
            SpecialKind::Firmament => return self.start_domain(),
            SpecialKind::Divine => return self.start_divine(),
            SpecialKind::Barrier => {}
        }
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

    /// Dev shortcut: Awaken every owned weapon.
    pub fn debug_awaken_all(&mut self) {
        for w in &mut self.weapons {
            w.level = MAX_LEVEL;
            w.awakened = true;
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
        w.cfg.hero = HeroId::Kaede; // a hero whose special is the plain Barrier
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
        let mut w = World::new(WorldConfig { seed: 8, hero: HeroId::Kaede, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
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
        let mut w = World::new(WorldConfig { hero: HeroId::Kaede, ..Default::default() });
        w.shots.push(Shot { pos: Vec2::new(50.0, 0.0), vel: Vec2::ZERO, life: 5.0, style: 0 });
        w.shots.push(Shot { pos: Vec2::new(900.0, 0.0), vel: Vec2::ZERO, life: 5.0, style: 0 });
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
                let (ps, n) = crate::stage::chunk_props(StageId::Shotengai, cx, cy);
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

    #[test]
    fn foxfire_orb_charges_in_hand_then_launches_and_bursts() {
        let mut w = World::new(WorldConfig { hero: HeroId::Riku, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.spawn_enemy(EnemyKind::Oni, Vec2::new(150.0, 0.0));
        w.enemies.speed[0] = 0.0;
        let mut charged = false;
        let mut burst = false;
        for _ in 0..60 * 4 {
            w.step(Input::default());
            if (0..w.projectiles.len()).any(|i| w.projectiles.charge[i] > 0.0 && (w.projectiles.pos[i] - w.player.pos).length() < 15.0) {
                charged = true;
            }
            burst |= w.events.iter().any(|e| matches!(e, Event::Burst { .. }));
        }
        assert!(charged, "orb never charged in the hand");
        assert!(burst, "orb never burst");
    }

    #[test]
    fn ninefold_foxfire_transforms_attacks_and_fades() {
        let mut w = World::new(WorldConfig { hero: HeroId::Riku, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.weapons.clear();
        for k in 0..30 {
            let a = k as f32 / 30.0 * std::f32::consts::TAU;
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(a) * (60.0 + (k % 3) as f32 * 40.0));
        }
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        w.player.special = 1.0;
        w.step(Input { special: true, ..Default::default() });
        assert!(w.events.iter().any(|e| matches!(e, Event::FoxAwakened { .. })));
        assert!(w.player.form > 0.0);
        let speed = w.move_speed();
        let (mut volleys, mut faded) = (0, false);
        for _ in 0..60 * 9 {
            w.step(Input::default());
            volleys += w.events.iter().filter(|e| matches!(e, Event::FoxVolley)).count();
            faded |= w.events.iter().any(|e| matches!(e, Event::FoxFaded));
        }
        assert!(faded && w.player.form == 0.0, "should fade after its duration");
        assert!(volleys >= 6, "golden orb volleys: {volleys}");
        assert!(w.kills >= 20, "the tails and orbs should flatten a ring of Oni, killed {}", w.kills);
        assert!(w.move_speed() < speed, "speed boost ends with the form");
    }

    #[test]
    fn fox_mirage_clones_decoy_strike_and_pop() {
        let mut w = World::new(WorldConfig { hero: HeroId::Riku, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.weapons = vec![OwnedWeapon { kind: WeaponKind::FoxMirage, level: 1, awakened: false, timer: 0.0, phase: 0.0 }];
        w.step(Input::default());
        assert_eq!(w.clones.len(), 1, "one clone at level 1");
        // Decoy: an enemy to the right of the player, a clone just above the enemy.
        // The enemy should head up toward the clone rather than left toward the player.
        w.spawn_enemy(EnemyKind::Kasa, Vec2::new(200.0, 0.0));
        w.enemies.hp[0] = 1e6;
        w.clones[0].pos = Vec2::new(200.0, 70.0);
        w.step(Input::default());
        let e = w.enemies.pos[0];
        assert!(e.y > 0.3 && e.x > 199.0, "enemy should chase the clone, at {e:?}");
        let (mut strikes, mut popped) = (0, false);
        for _ in 0..60 * 8 {
            w.step(Input::default());
            strikes += w.events.iter().filter(|e| matches!(e, Event::CloneStrike { .. })).count();
            popped |= w.events.iter().any(|e| matches!(e, Event::ClonePopped { .. }));
        }
        assert!(strikes > 3, "clone should keep striking, got {strikes}");
        assert!(popped, "clone should pop when its time or hits run out");
    }

    #[test]
    fn hannya_unbound_beams_steps_and_ends_in_a_finale() {
        let mut w = World::new(WorldConfig { hero: HeroId::Shun, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        assert_eq!(w.weapons[0].kind, WeaponKind::GraveCleaver);
        w.weapons.clear();
        for k in 0..36 {
            let a = k as f32 / 36.0 * std::f32::consts::TAU;
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(a) * (80.0 + (k % 4) as f32 * 50.0));
        }
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        w.player.special = 1.0;
        w.step(Input { special: true, ..Default::default() });
        assert!(w.events.iter().any(|e| matches!(e, Event::HannyaAwakened { .. })));
        assert_eq!(w.form_kind(), Some(SpecialKind::Hannya));
        let (mut beams, mut steps, mut blasts, mut finale) = (0, 0, 0, false);
        for _ in 0..60 * 9 {
            w.step(Input::default());
            for e in &w.events {
                match e {
                    Event::Beam { .. } => beams += 1,
                    Event::ShadowStep { .. } => steps += 1,
                    Event::AfterimageBlast { .. } => blasts += 1,
                    Event::FinaleBlast { .. } => finale = true,
                    _ => {}
                }
            }
        }
        assert!(beams >= 30, "beams: {beams}");
        assert!(steps >= 2 && blasts >= 2, "shadow steps {steps}, afterimage blasts {blasts}");
        assert!(finale, "Hannya's Wrath should go off at the end");
        assert!(w.player.pos.length() < 5.0, "the form must not teleport the player, ended at {:?}", w.player.pos);
        assert_eq!(w.form_kind(), None);
        assert!(w.kills >= 15, "should tear through the ring, killed {}", w.kills);
    }

    #[test]
    fn hundred_leg_rampage_lashes_erupts_and_regenerates() {
        let mut w = World::new(WorldConfig { hero: HeroId::Toya, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        assert_eq!(w.weapons[0].kind, WeaponKind::CrimsonTendrils);
        w.weapons.clear();
        for k in 0..36 {
            let a = k as f32 / 36.0 * std::f32::consts::TAU;
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(a) * (70.0 + (k % 4) as f32 * 40.0));
        }
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        w.player.hp = 40.0;
        w.player.special = 1.0;
        let speed = w.move_speed();
        w.step(Input { special: true, ..Default::default() });
        assert!(w.events.iter().any(|e| matches!(e, Event::MukadeAwakened { .. })));
        assert!(w.move_speed() > speed, "predator speed");
        let (mut lashes, mut spines) = (0, 0);
        for _ in 0..60 * 9 {
            w.step(Input::default());
            for e in &w.events {
                match e {
                    Event::Lash { .. } => lashes += 1,
                    Event::Burst { style: s, .. } if *s == style::SPINE => spines += 1,
                    _ => {}
                }
            }
        }
        assert!(lashes >= 60 && spines >= 30, "lashes {lashes}, spines {spines}");
        assert!(w.player.hp > 60.0, "should regenerate, hp {}", w.player.hp);
        assert_eq!(w.form_kind(), None);
        assert!(w.kills >= 15, "killed {}", w.kills);
    }

    #[test]
    fn pickups_spawn_and_apply() {
        let mut w = World::new(WorldConfig::default());
        w.player.iframes = 1e9;
        for _ in 0..60 * 31 {
            w.step(Input::default());
            if w.offer.is_some() {
                w.choose(0);
            }
        }
        assert!(!w.pickups.is_empty(), "a pickup should appear by 30s");
        // Each kind does its thing.
        for k in ALL_PICKUPS {
            let mut w = World::new(WorldConfig::default());
            w.player.hp = 20.0;
            for n in 0..20 {
                w.spawn_enemy(EnemyKind::Wisp, Vec2::from_angle(n as f32) * 150.0);
            }
            w.step(Input::default());
            w.pickups.push((w.player.pos, k));
            w.step(Input::default());
            assert!(w.events.iter().any(|e| matches!(e, Event::PickupTaken { .. })), "{k:?} not taken");
            match k {
                PickupKind::Feast => assert!(w.player.hp > 50.0),
                PickupKind::SwiftGeta => assert!(w.speed_buff > 0.0),
                PickupKind::OniSake => assert!(w.damage_buff > 0.0),
                PickupKind::ShrineBell => assert!(w.vacuum > 0.0),
                PickupKind::Hourglass => assert!(w.freeze > 0.0),
                PickupKind::SenPurse => assert!(w.sen >= 40),
                PickupKind::ExorcismSeal => {
                    w.step(Input::default());
                    assert!(w.kills >= 20, "seal should banish the ring, killed {}", w.kills);
                }
            }
        }
    }

    #[test]
    fn map_markers_cover_everything_on_the_field() {
        let mut w = World::new(WorldConfig::default());
        w.spawn_enemy(EnemyKind::Wisp, Vec2::new(50.0, 0.0));
        w.spawn_enemy(EnemyKind::Oni, Vec2::new(80.0, 0.0));
        w.spawn_enemy(EnemyKind::GreatOni, Vec2::new(120.0, 0.0));
        w.chests.push((Vec2::new(10.0, 10.0), 3));
        w.shrine = Some(Vec2::new(-40.0, 0.0));
        w.pickups.push((Vec2::new(0.0, 60.0), PickupKind::OniSake));
        let m = w.map_markers();
        let has = |k: MarkerKind| m.iter().any(|x| x.kind == k);
        assert!(has(MarkerKind::Enemy) && has(MarkerKind::Elite) && has(MarkerKind::MiniBoss));
        assert!(has(MarkerKind::Chest { tier: 3 }) && has(MarkerKind::Shrine) && has(MarkerKind::Pickup(PickupKind::OniSake)));
        let obstacles = w.map_obstacles(Vec2::new(-2000.0, -2000.0), Vec2::new(2000.0, 2000.0));
        assert!(!obstacles.is_empty(), "the stage's props should be reported to the map");
    }

    #[test]
    fn kuon_ward_stillness_and_endless_firmament() {
        let mut w = World::new(WorldConfig { hero: HeroId::Kuon, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        assert_eq!(w.weapons[0].kind, WeaponKind::GravityWell);
        w.cfg.stress = None;
        // Ward: the first hit is negated, the next one lands.
        let hp = w.player.hp;
        w.hurt_player(30.0);
        assert_eq!(w.player.hp, hp);
        w.player.iframes = 0.0;
        w.hurt_player(30.0);
        assert!(w.player.hp < hp);
        // Stillness: an enemy right next to Kuon crawls.
        w.cfg.stress = Some(Stress { enemies: 0, projectiles: 0 });
        w.weapons.clear();
        w.spawn_enemy(EnemyKind::Kasa, Vec2::new(40.0, 0.0));
        w.enemies.hp[0] = 1e9;
        let before = w.enemies.pos[0];
        w.step(Input::default());
        assert!((w.enemies.pos[0] - before).length() < 0.6, "stillness should slow it, moved {}", (w.enemies.pos[0] - before).length());
        // Domain: enemies inside freeze, take damage, and the finale lance fires.
        for k in 0..20 {
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(k as f32) * 200.0);
        }
        w.player.special = 1.0;
        w.step(Input { special: true, ..Default::default() });
        assert!(w.events.iter().any(|e| matches!(e, Event::DomainOpened { .. })));
        let i = w.enemies.len() - 1;
        let frozen_at = w.enemies.pos[i];
        let mut finale = false;
        for _ in 0..60 * 9 {
            w.step(Input::default());
            finale |= w.events.iter().any(|e| matches!(e, Event::VoidMaximum { .. }));
        }
        assert!(finale, "Void Lance: Maximum should launch at the end");
        assert!(w.kills >= 10, "domain waves should kill the suspended ring, killed {}", w.kills);
        let _ = frozen_at;
    }

    #[test]
    fn ren_barrier_blocks_fireballs_and_divine_purification_is_invulnerable() {
        let mut w = World::new(WorldConfig::default());
        assert_eq!(hero_def(HeroId::Ren).special, SpecialKind::Divine);
        let hp = w.player.hp;
        w.shots.push(Shot { pos: w.player.pos, vel: Vec2::ZERO, life: 5.0, style: 0 });
        w.step(Input::default());
        assert_eq!(w.player.hp, hp, "the barrier blocks the first fireball");
        assert!(w.barrier_cd > 0.0);
        // Divine Purification: invulnerable, storms talismans, ends in the final blast.
        w.weapons.clear();
        for k in 0..30 {
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(k as f32 * 0.21) * (90.0 + (k % 3) as f32 * 60.0));
        }
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        w.player.special = 1.0;
        w.step(Input { special: true, ..Default::default() });
        assert!(w.events.iter().any(|e| matches!(e, Event::DivineAwakened { .. })));
        let hp = w.player.hp;
        w.hurt_player(999.0);
        assert_eq!(w.player.hp, hp, "invulnerable during the form");
        let mut finale = false;
        for _ in 0..60 * 9 {
            w.step(Input::default());
            finale |= w.events.iter().any(|e| matches!(e, Event::Burst { style: s, .. } if *s == style::PURIFY));
        }
        assert!(finale, "Final Purification should go off");
        assert!(w.kills >= 20, "killed {}", w.kills);
    }

    fn damage_over(kind: WeaponKind, awakened: bool, hero: HeroId) -> f32 {
        let mut w = World::new(WorldConfig { seed: 5, hero, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.time = AWAKEN_FROM + 1.0;
        w.weapons = vec![OwnedWeapon { kind, level: MAX_LEVEL, awakened, timer: 0.0, phase: 0.0 }];
        for k in 0..60 {
            let a = k as f32 / 60.0 * std::f32::consts::TAU;
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(a) * (40.0 + (k % 5) as f32 * 45.0));
        }
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        w.enemies.hp.iter_mut().for_each(|h| *h = 1e9);
        let mut dealt = 0.0;
        for _ in 0..60 * 6 {
            w.step(Input::default());
            dealt += w.events.iter().filter_map(|e| if let Event::Hit { amount, .. } = e { Some(*amount) } else { None }).sum::<f32>();
        }
        dealt
    }

    #[test]
    fn awakenings_are_a_huge_power_spike() {
        for k in ALL_WEAPONS {
            let hero = ALL_HEROES.iter().copied().find(|h| hero_def(*h).weapon_pool().contains(&k)).unwrap_or(HeroId::Ren);
            let (base, aw) = (damage_over(k, false, hero), damage_over(k, true, hero));
            println!("{k:?}: max level {base:.0}, awakened {aw:.0} ({:.1}x)", aw / base.max(1.0));
            assert!(aw > base * 2.0, "{k:?}: awakened {aw:.0} vs max level {base:.0}");
        }
    }

    /// Full 15-minute bot run; prints how far it got. `cargo test -p sim balance -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn balance_full_run() {
        let seeds: u64 = std::env::var("BALANCE_SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
        for seed in 1..=seeds {
            let started = std::time::Instant::now();
            let hero = std::env::var("BALANCE_HERO").ok().and_then(|h| HeroId::from_key(&h)).unwrap_or(HeroId::Ren);
            let stage = std::env::var("BALANCE_STAGE").ok().and_then(|h| StageId::from_key(&h)).unwrap_or(StageId::Shotengai);
            let difficulty = std::env::var("BALANCE_DIFF").ok().and_then(|h| Difficulty::from_key(&h)).unwrap_or(Difficulty::Normal);
            let mut w = World::new(WorldConfig { seed, hero, stage, difficulty, ..Default::default() });
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
            let nl = w.boss().map(|b| format!("{} {:.0}%", b.2.short_name(), b.1 * 100.0)).unwrap_or("-".into());
            println!(
                "seed {seed} [boss {nl}] bosses_down {:03b}: {:?} at {:>5.0}s  level {:>2}  kills {:>6}  ({:.1}s real)  weapons {:?}  awakened {:?}",
                w.bosses_down,
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
