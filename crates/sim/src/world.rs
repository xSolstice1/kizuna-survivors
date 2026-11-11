//! The run simulation: player, weapons, spawn director, enemies, pickups.

use crate::campaign::{Difficulty, Headline, Mechanic, StageDef, StageId, LEADER_ROUNDS, MID_BOSS_ROUND, SUB_BOSS_ROUND};
use crate::content::*;
use crate::gear::{self, ArsenalArt, Exotic, Item, Loadout, Rarity, Unique};
use crate::grid::Grid;
use crate::meta::Bonus;
use crate::stage;
use crate::rng::Rng;
use crate::Vec2;

pub const DT: f32 = 1.0 / 60.0;

const PLAYER_RADIUS: f32 = 8.0;
/// Reach for floor pickups and chests, so the player needn't stand exactly on them.
const ITEM_GRAB: f32 = PLAYER_RADIUS + 20.0;
const PROJECTILE_RADIUS: f32 = 4.0;
const SEPARATION_NEIGHBOURS: u32 = 10;
const MAX_ENEMIES: usize = 4000;
/// When this many gems are on the field, nearby gems merge into higher-value ones...
const GEM_MERGE_AT: usize = 220;
/// ...until at most this many remain.
const GEM_MERGE_TO: usize = 140;
const ORB_RADIUS: f32 = 8.0;
const ORB_ORBIT: f32 = 46.0;
const FIELD_RADIUS: f32 = 44.0;
const IFRAMES: f32 = 0.5;
/// Move speed inside spirit pressure.
const PRESSURE_SLOW: f32 = 0.6;
/// Seconds a Leader shrugs off Earth after a stagger ends, so Earth builds cannot stun-lock it.
const LEADER_STAGGER_IMMUNE: f32 = 1.5;

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
    /// Nightmare+N (0 = plain difficulty).
    pub ascension: u32,
    /// The hero's equipped gear.
    pub gear: Loadout,
    /// Fire the special by itself when crowded or low on health (Shrine Shop unlock).
    pub auto_special: bool,
}

impl Default for WorldConfig {
    fn default() -> Self {
        WorldConfig { seed: 1, hero: HeroId::Ren, stress: None, spawn_radius: 420.0, recycle_radius: 900.0, meta: Bonus::default(), stage: StageId::Shotengai, difficulty: Difficulty::Normal, ascension: 0, gear: Loadout::default(), auto_special: false }
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
    Hit { pos: Vec2, amount: f32, crit: bool, elem: Option<Element>, mult: f32 },
    /// Equipment dropped (auto-collected).
    GearDrop { pos: Vec2, rarity: Rarity },
    Kill { pos: Vec2, kind: EnemyKind },
    PlayerHurt,
    LevelUp,
    SpecialUsed { pos: Vec2 },
    /// A mini-boss or boss arrived.
    BossSpawned(EnemyKind),
    /// A Leader arrived with its crew.
    LeaderSpawned { kind: EnemyKind },
    /// The elite wave began.
    EliteWave,
    /// A new round began.
    RoundStarted { round: u8, headline: Option<Headline> },
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
    Hazard { pos: Vec2, radius: f32, kind: u8, friendly: bool },
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
    /// A Tool Thief snatched a floor pickup.
    PickupStolen { pos: Vec2, kind: PickupKind },
    /// A Leader died and its crew scattered.
    LeaderFell { pos: Vec2 },
    /// A Clay Bird's blast began its warning.
    HazardWarn { pos: Vec2, radius: f32 },
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
    pub elem: Option<Element>,
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
    pub elem: Option<Element>,
}

/// A lingering Moonsever scar that keeps cutting.
#[derive(Clone, Copy, Debug)]
pub struct Scar {
    pub from: Vec2,
    pub to: Vec2,
    pub life: f32,
    tick: f32,
    damage: f32,
    pub elem: Option<Element>,
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
    /// A Sen coin on the ground.
    Coin,
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
    pub elem: Option<Element>,
}

/// Enemy fireball.
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    /// Look: 0 fireball, otherwise the stage index (themed bullets).
    pub style: u8,
    /// The shooter's element, which gear resistance matches against.
    pub elem: Option<Element>,
}

/// An exclusive Arsenal attack in play (summons drawn by the client).
#[derive(Clone, Debug)]
pub struct ArtState {
    pub art: ArsenalArt,
    pub power: f32,
    pub mythic: bool,
    timer: f32,
    /// Rotation for drums and the dragon.
    pub phase: f32,
    /// Bouncing yin-yang orbs: (pos, vel).
    pub orbs: Vec<(Vec2, Vec2)>,
    /// Marching foxes: (pos, vel, life).
    pub foxes: Vec<(Vec2, Vec2, f32)>,
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
    /// Set bonuses' own strikes: never hurt the player.
    pub friendly: bool,
    /// Blasts that hurt only the player.
    pub spare_enemies: bool,
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
    pub elem: Option<Element>,
}

/// Star Fall strike waiting to land.
#[derive(Clone, Copy, Debug)]
pub struct Strike {
    pub pos: Vec2,
    pub delay: f32,
    pub radius: f32,
    pub damage: f32,
    pub elem: Option<Element>,
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
    pub elem: Option<Element>,
}

/// One reward from an opened chest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChestItem {
    LevelUp(Upgrade, u8),
    Awakening(WeaponKind),
    /// One of the maxed-out fallbacks (Onigiri, Sen Pouch, Blessing) and the amount it gave.
    Extra(Upgrade, f32),
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
    /// Element statuses: seconds left of each, plus the burn's damage per second.
    pub burn: Vec<f32>,
    pub burn_dps: Vec<f32>,
    pub soak: Vec<f32>,
    pub stagger: Vec<f32>,
    /// Seconds left before a Leader can be staggered again.
    pub stagger_immune: Vec<f32>,
    pub slow: Vec<f32>,
    /// Light: the next hit on this enemy crits.
    pub marked: Vec<bool>,
    /// Inside a live Leader's banner this tick.
    pub banner: Vec<bool>,
    /// Seconds left fleeing `scatter_from`, the point a Leader fell.
    pub scatter: Vec<f32>,
    pub scatter_from: Vec<Vec2>,
    /// A Tool Thief's stolen pickup, and seconds left fleeing the player with it.
    pub carry: Vec<Option<PickupKind>>,
    pub flee: Vec<f32>,
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
        self.burn.swap_remove(i);
        self.burn_dps.swap_remove(i);
        self.soak.swap_remove(i);
        self.stagger.swap_remove(i);
        self.stagger_immune.swap_remove(i);
        self.slow.swap_remove(i);
        self.marked.swap_remove(i);
        self.banner.swap_remove(i);
        self.scatter.swap_remove(i);
        self.scatter_from.swap_remove(i);
        self.carry.swap_remove(i);
        self.flee.swap_remove(i);
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
    pub elem: Option<Element>,
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
    pub elem: Option<Element>,
    /// `Enemies.id` of the enemy this blast rides until it lands.
    pub follow: Option<u32>,
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
    pub elem: Vec<Option<Element>>,
}

impl Projectiles {
    pub fn len(&self) -> usize {
        self.pos.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pos.is_empty()
    }
    fn push(&mut self, pos: Vec2, vel: Vec2, life: f32, damage: f32, pierce: u8, homing: f32) {
        self.push_kind(ProjKind::Talisman, PROJECTILE_RADIUS, pos, vel, life, damage, pierce, homing, None);
    }
    #[allow(clippy::too_many_arguments)]
    fn push_kind(&mut self, kind: ProjKind, radius: f32, pos: Vec2, vel: Vec2, life: f32, damage: f32, pierce: u8, homing: f32, elem: Option<Element>) {
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
        self.elem.push(elem);
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
        self.elem.swap_remove(i);
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
    /// Granted by the Arsenal slot: takes no weapon slot and never levels in the run.
    pub extra: bool,
    /// Damage multiplier (Arsenal item level).
    pub power: f32,
}

pub struct World {
    pub cfg: WorldConfig,
    /// Half the visible world size; the client sets it each frame.
    pub view_half: Vec2,
    reach_scale: f32,
    hero_element: Element,
    /// Worlds whose 6-piece signature is active, by `StageId::index`.
    set6: [bool; 6],
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
    /// Highest round whose start has been handled.
    pub(crate) round_seen: u8,
    /// Bosses defeated this run: bit 0 mini-boss, bit 1 boss, bit 2 final boss.
    pub bosses_down: u8,
    /// World hazards and boss strikes on the field.
    pub hazards: Vec<Hazard>,
    /// Equipment found this run (banked when it ends).
    pub loot: Vec<Item>,
    /// Gear effect timers: thunderclap, starburst, set tag, set lightning, aegis, second wind.
    gear_timers: [f32; 6],
    gear_kills: u32,
    /// Crits waiting to explode (Eclipse): (position, damage).
    eclipses: Vec<(Vec2, f32)>,
    pub art: Option<ArtState>,
    /// Exotic blasts waiting for a safe point: (centre, radius, base damage).
    pending_exo_blasts: Vec<(Vec2, f32, f32)>,
    /// Exotic state: lantern / fox-fire / regalia timers, hollow mask seconds left,
    /// seconds since the last kill, hits counted (Thousand Minus Seven).
    exo_timers: [f32; 3],
    pub mask: f32,
    since_kill: f32,
    hit_count: u32,
    next_hazard: f32,
    pending_spawns: Vec<(EnemyKind, Vec2)>,
    /// A shrine waiting on the ground.
    pub shrine: Option<Vec2>,
    /// Two Blessing+Price pairs to choose from. The sim pauses until `choose_shrine`.
    pub shrine_offer: Option<Vec<ShrineEvent>>,
    xp_carry: f32,
    pub events: Vec<Event>,
    pending_levels: u32,
    /// Replacement card taken, waiting for the skill it will overwrite.
    pub replacing: Option<Upgrade>,
    /// Balance counters; never read by the simulation.
    pub replacements: u32,
    /// Skills overwritten by a replacement; the profile still credits them.
    pub retired_weapons: Vec<OwnedWeapon>,
    pub retired_passives: Vec<(PassiveKind, u8)>,
    pub unique_offered: bool,
    pub slots_full_round: Option<u8>,
    next_id: u32,
    next_elite: f32,
    rng: Rng,
    grid: Grid,
    scratch: Vec<u32>,
    dead: Vec<usize>,
}

impl World {
    pub fn new(cfg: WorldConfig) -> Self {
        let first_elite = cfg.stage.def().round_start(8);
        let rng = Rng::new(cfg.seed);
        let hero = hero_def(cfg.hero);
        let signature = hero.signature;
        let art = cfg.gear.art.map(|(art, power, mythic)| ArtState { art, power, mythic, timer: 1.5, phase: 0.0, orbs: Vec::new(), foxes: Vec::new() });
        let mut bonus = hero.innate.plus(cfg.meta).plus(cfg.gear.bonus);
        let g = &cfg.gear;
        bonus.revivals += g.unique(Unique::Phoenix) as u8;
        bonus.greed += 0.5 * g.unique(Unique::Fortune);
        bonus.gear_find += 0.3 * g.unique(Unique::Fortune);
        // Endless gear can stack stats without bound; keep the run playable.
        bonus.cooldown = bonus.cooldown.min(0.6);
        bonus.move_speed = bonus.move_speed.min(1.0);
        bonus.crit_chance = bonus.crit_chance.min(1.0);
        let glass = if g.unique(Unique::GlassLotus) > 0.0 { 0.8 } else { 1.0 };
        let max_hp = 100.0 * (1.0 + bonus.max_hp) * glass;
        let mut weapons = vec![OwnedWeapon { kind: signature, level: 1, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }];
        if let Some((kind, level, awakened, power)) = g.arsenal {
            match weapons.iter_mut().find(|w| w.kind == kind) {
                // Same as the signature: the Arsenal raises it instead.
                Some(w) => {
                    w.level = w.level.max(level);
                    w.awakened |= awakened;
                }
                None => weapons.push(OwnedWeapon { kind, level, awakened, timer: 0.0, phase: 0.0, extra: true, power }),
            }
        }
        let set6 = crate::campaign::ALL_STAGES.map(|s| cfg.gear.has_set6(s));
        let cell = 28.0;
        let dim = ((cfg.recycle_radius * 2.2) / cell).ceil() as usize;
        World {
            cfg,
            view_half: DEFAULT_VIEW_HALF,
            reach_scale: hero.reach,
            hero_element: hero.element,
            set6,
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
            weapons,
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
            round_seen: 0,
            bosses_down: 0,
            hazards: Vec::new(),
            loot: Vec::new(),
            gear_timers: [4.0, 6.0, 6.0, 3.0, 0.0, 0.0],
            gear_kills: 0,
            eclipses: Vec::new(),
            art,
            exo_timers: [10.0, 2.0, 30.0],
            mask: 0.0,
            pending_exo_blasts: Vec::new(),
            since_kill: 0.0,
            hit_count: 0,
            next_hazard: 45.0,
            pending_spawns: Vec::new(),
            shrine: None,
            shrine_offer: None,
            xp_carry: 0.0,
            events: Vec::new(),
            pending_levels: 0,
            replacing: None,
            replacements: 0,
            retired_weapons: Vec::new(),
            retired_passives: Vec::new(),
            unique_offered: false,
            slots_full_round: None,
            next_id: 0,
            next_elite: first_elite,
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
        self.update_banners();
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
        if input.special || self.wants_auto_special() {
            let before = self.player.form;
            let used = self.specials_used;
            self.use_special();
            if before <= 0.0 && self.player.form > 0.0 {
                self.player.form += 2.0 * self.cfg.gear.unique(Unique::Overflow);
            }
            if self.specials_used > used {
                self.on_special_exotics();
            }
        }
        self.contact_damage();
        self.update_shots();
        self.update_hazards();
        self.update_gear();
        self.update_pickups();
        self.tick_statuses();
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
        let g = &self.cfg.gear;
        let fury = if self.player.hp < self.player.max_hp * 0.4 { 1.0 + 0.3 * g.unique(Unique::Berserker) } else { 1.0 };
        let lotus = 1.0 + 0.35 * g.unique(Unique::GlassLotus);
        let reach = hero_def(self.cfg.hero).reach_might;
        (1.0 + 0.10 * self.passive_level(PassiveKind::Might) as f32 + self.bonus.might) * self.mods.damage * sake * fury * lotus * reach * self.exotic_might()
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
        s.damage *= w.power;
        s
    }

    pub fn field_radius_of(&self, w: &OwnedWeapon) -> f32 {
        self.reach_cap(FIELD_RADIUS * self.stats_of(w).area * self.area_mult())
    }

    /// Yin-yang orbs: each throw sends every orb straight out to the edge of Ren's reach and back.
    pub fn yinyang_positions(&self, w: &OwnedWeapon) -> Vec<Vec2> {
        let n = self.stats_of(w).amount.max(1);
        let throw = w.phase / std::f32::consts::TAU;
        let heading = throw.floor() * 2.4;
        let out_and_back = (std::f32::consts::PI * throw.fract()).sin();
        let radius = self.yinyang_radius(w);
        (0..n)
            .map(|k| {
                let dir = Vec2::from_angle(heading + k as f32 / n as f32 * std::f32::consts::TAU);
                let edge = (self.clip_to_reach(self.player.pos, self.player.pos + dir * 1000.0) - self.player.pos).length() - radius;
                self.player.pos + dir * (edge.max(0.0) * out_and_back)
            })
            .collect()
    }

    pub fn yinyang_radius(&self, w: &OwnedWeapon) -> f32 {
        9.0 * self.stats_of(w).area * self.area_mult()
    }

    /// World positions of each moon orb. Awakened adds a counter-rotating outer ring.
    pub fn orb_positions(&self, w: &OwnedWeapon) -> Vec<Vec2> {
        let s = self.stats_of(w);
        let orbit = ORB_ORBIT * s.area * self.area_mult();
        let n = s.amount;
        let mut out = Vec::with_capacity(n as usize * 2);
        for k in 0..n {
            let a = w.phase + k as f32 / n as f32 * std::f32::consts::TAU;
            out.push(self.player.pos + Vec2::from_angle(a) * self.reach_cap(orbit));
            if w.awakened {
                let b = -w.phase * 0.8 + (k as f32 + 0.5) / n as f32 * std::f32::consts::TAU;
                out.push(self.player.pos + Vec2::from_angle(b) * self.reach_cap(orbit * 1.65));
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
    fn hurt_player(&mut self, raw: f32, elem: Option<Element>) {
        // Divine Purification: nothing touches the shrine maiden.
        if self.form_kind() == Some(SpecialKind::Divine) {
            return;
        }
        let aegis = self.cfg.gear.unique(Unique::Aegis);
        if aegis > 0.0 && self.gear_timers[4] <= 0.0 {
            self.gear_timers[4] = 14.0 / aegis;
            self.player.iframes = 0.5;
            self.events.push(Event::Warded);
            return;
        }
        // Kuon's ward: a hit is simply not allowed to land.
        if self.cfg.hero == HeroId::Kuon && self.ward_cd <= 0.0 {
            self.ward_cd = WARD_RECHARGE;
            self.player.iframes = 0.6;
            self.events.push(Event::Warded);
            return;
        }
        let mut raw = raw * (1.0 - elem.map_or(0.0, |e| self.cfg.gear.resist(e)));
        if self.cfg.gear.has(Exotic::WhiteHaori) && self.boss().is_some() {
            raw *= 0.6;
        }
        if self.cfg.gear.has(Exotic::NineSeal) && self.specials_used >= 9 {
            raw *= 1.3;
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
        let wind = self.cfg.gear.unique(Unique::SecondWind);
        if wind > 0.0 && self.player.hp > 0.0 && self.player.hp < self.player.max_hp * 0.3 && self.gear_timers[5] <= 0.0 {
            self.gear_timers[5] = 60.0 / wind;
            self.player.iframes = 3.0;
            self.events.push(Event::Warded);
        }
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
        1.0 + PERSISTENCE_PER_LEVEL * self.passive_level(PassiveKind::Persistence) as f32
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
        let mut worst_elem = None;
        let enemies = &self.enemies;
        self.grid.for_each_near(p, PLAYER_RADIUS + MAX_ENEMY_RADIUS, |j| {
            let j = j as usize;
            let r = PLAYER_RADIUS + enemies.radius[j];
            let faded = (enemy_def(enemies.kind[j]).ai == Ai::Blink && enemies.ai_state[j] != 0) || (frozen && !enemies.kind[j].is_boss());
            if enemies.hp[j] > 0.0 && !faded && (enemies.pos[j] - p).length_sq() <= r * r {
                let damage = enemies.damage[j] * if enemies.banner[j] { BANNER_DAMAGE } else { 1.0 };
                if damage > worst {
                    worst = damage;
                    worst_elem = Some(enemies.kind[j].element());
                }
            }
        });
        if worst > 0.0 {
            self.hurt_player(worst, worst_elem);
        }
    }

    // ───────────── Spawning ─────────────

    fn ring_point(&mut self, radius: f32) -> Vec2 {
        let a = self.rng.range(0.0, std::f32::consts::TAU);
        self.player.pos + Vec2::from_angle(a) * radius
    }

    pub fn round(&self) -> u8 {
        self.stage().round_at(self.time)
    }

    /// Difficulty pace held at the current round's start, so it steps once per round.
    pub fn round_pace(&self) -> f32 {
        let st = self.stage();
        st.pace(st.round_start(self.round()))
    }

    /// Compounds once per headline round reached, so difficulty never dips after one.
    fn headline_bump(&self) -> f32 {
        let reached = (1..=self.round()).filter(|&r| StageDef::headline(r).is_some()).count();
        HEADLINE_BUMP.powi(reached as i32)
    }

    pub fn stage(&self) -> &'static StageDef {
        self.cfg.stage.def()
    }

    /// Fodder/elite HP multiplier now: the stage's stretched curve, its tier and difficulty.
    fn enemy_hp_scale(&self) -> f32 {
        let st = self.stage();
        let late = 1.0 + st.late_hp * (self.time / st.final_at).min(1.2).powi(2);
        hp_scale(self.round_pace()) * self.headline_bump() * late * st.tier * self.cfg.difficulty.hp() * self.ascension_hp()
    }

    fn ascension_hp(&self) -> f32 {
        let fingers = if self.cfg.gear.has(Exotic::SovereignFinger) { self.cfg.gear.fingers as f32 } else { 0.0 };
        (1.0 + 0.4 * self.cfg.ascension as f32) * (1.0 + 0.04 * fingers)
    }

    /// Damage multiplier from Exotics (Nine Seals, the Hollow Mask, Fingers).
    fn exotic_might(&self) -> f32 {
        let g = &self.cfg.gear;
        if g.exotics.is_empty() {
            return 1.0;
        }
        let mut m = 1.0;
        if g.has(Exotic::NineSeal) {
            m *= 1.0 + 0.1 * self.specials_used.min(9) as f32;
        }
        if self.mask > 0.0 {
            m *= 1.8;
        }
        if g.has(Exotic::SovereignFinger) {
            m *= 1.0 + 0.06 * g.fingers as f32;
        }
        m
    }

    /// Enemy damage multiplier now (contact, bullets, hazards).
    pub fn enemy_damage_scale(&self) -> f32 {
        damage_scale(self.round_pace()) * self.headline_bump() * self.cfg.difficulty.damage() * (1.0 + 0.12 * self.cfg.ascension as f32)
    }

    fn spawn_enemy(&mut self, kind: EnemyKind, pos: Vec2) {
        if self.enemies.len() >= MAX_ENEMIES {
            return;
        }
        let d = enemy_def(kind);
        let hp = if kind.is_boss() {
            d.hp * BOSS_HP * self.stage().tier * self.cfg.difficulty.hp() * self.ascension_hp() * self.mods.enemy_hp
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
        e.burn.push(0.0);
        e.burn_dps.push(0.0);
        e.soak.push(0.0);
        e.stagger.push(0.0);
        e.stagger_immune.push(0.0);
        e.slow.push(0.0);
        e.marked.push(false);
        e.banner.push(false);
        e.scatter.push(0.0);
        e.scatter_from.push(Vec2::ZERO);
        e.carry.push(None);
        e.flee.push(0.0);
    }

    fn spawn_boss(&mut self, kind: EnemyKind) {
        let p = self.ring_point(self.cfg.spawn_radius * 0.8);
        self.spawn_enemy(kind, p);
        self.events.push(if kind.rank() == Rank::Final { Event::NightLordSpawned(kind) } else { Event::BossSpawned(kind) });
    }

    fn leader_alive(&self) -> bool {
        self.leaders().next().is_some()
    }

    fn spawn_leader(&mut self) {
        if !self.leader_alive() {
            let p = self.ring_point(self.cfg.spawn_radius);
            self.spawn_leader_at(p);
        }
    }

    fn spawn_leader_at(&mut self, p: Vec2) {
        let st = self.stage();
        self.spawn_enemy(st.leader, p);
        let crew = (10.0 * self.cfg.difficulty.density()) as usize;
        for k in 0..crew {
            let a = k as f32 / crew as f32 * std::f32::consts::TAU;
            self.spawn_enemy(st.fodder, p + Vec2::from_angle(a) * 50.0);
        }
        self.events.push(Event::LeaderSpawned { kind: st.leader });
    }

    fn start_round(&mut self, round: u8, basic: EnemyKind, rung: usize) {
        let st = self.stage();
        let density = self.cfg.difficulty.density();
        self.events.push(Event::RoundStarted { round, headline: StageDef::headline(round) });
        match round {
            4 => {
                let n = (24.0 * density) as usize;
                self.spawn_ring(&st.uncommon, n, 270.0);
                self.chests.push((self.player.pos + Vec2::new(0.0, -60.0), chest_tier(st.elite)));
            }
            8 if st.elite_wave => {
                let n = 4 + self.cfg.difficulty.index() * 2;
                self.spawn_ring(&[st.elite], n, self.cfg.spawn_radius);
                self.events.push(Event::EliteWave);
            }
            SUB_BOSS_ROUND => self.spawn_boss(st.mini),
            MID_BOSS_ROUND => {
                if let Some(b) = st.boss {
                    self.spawn_boss(b);
                }
            }
            _ => {}
        }
        if LEADER_ROUNDS.contains(&round) {
            self.spawn_leader();
        }
        if st.shrine_rounds.contains(&round) {
            let a = self.rng.range(0.0, std::f32::consts::TAU);
            self.shrine = Some(stage::resolve(self.cfg.stage, self.player.pos + Vec2::from_angle(a) * 240.0, 18.0));
            self.events.push(Event::ShrineAppeared);
        }
        if st.ring_rounds.contains(&round) {
            let n = (120.0 * density * LADDER_COUNT[rung]) as usize;
            self.spawn_ring(&[basic], n, 270.0);
        }
    }

    fn spawn_ring(&mut self, kinds: &[EnemyKind], n: usize, radius: f32) {
        for k in 0..n {
            let a = k as f32 / n as f32 * std::f32::consts::TAU;
            let p = self.player.pos + Vec2::from_angle(a) * radius;
            self.spawn_enemy(kinds[k % kinds.len()], p);
        }
    }

    fn director(&mut self) {
        let st = self.stage();
        let pace = self.round_pace();
        let rung = ladder_rung(pace);
        let basic = if rung > 0 { st.fodder_b } else { st.fodder };
        let density = self.cfg.difficulty.density() * (1.0 + 0.04 * self.cfg.ascension as f32).min(1.6) * if st.in_crisis(self.time) { 1.35 } else { 1.0 } * LADDER_COUNT[rung];
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
            let lift = LADDER_LIFT[rung];
            let (c, n) = (chochin_share(pace) * st.ranged_mult * lift, nopperabo_share(pace) * lift);
            let kind = if self.round() >= 5 && self.rng.f32() < UNCOMMON_SHARE {
                st.uncommon[(self.rng.f32() * 2.0) as usize % 2]
            } else if r < c {
                st.ranged
            } else if r < c + n {
                st.blinker
            } else if rung > 0 || self.rng.f32() < kasa_share(pace) {
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
            let current = self.round();
            if current > self.round_seen + 1 {
                if current > SUB_BOSS_ROUND && self.round_seen < SUB_BOSS_ROUND {
                    self.spawn_boss(st.mini);
                }
                if let (true, Some(b)) = (current > MID_BOSS_ROUND && self.round_seen < MID_BOSS_ROUND, st.boss) {
                    self.spawn_boss(b);
                }
                if LEADER_ROUNDS.iter().any(|&r| r > self.round_seen && r < current) {
                    self.spawn_leader();
                }
                self.round_seen = current - 1;
            }
            while self.round_seen < current {
                self.round_seen += 1;
                self.start_round(self.round_seen, basic, rung);
            }
            if self.time >= self.next_elite {
                self.next_elite += elite_interval(pace) * self.cfg.difficulty.elite_gap();
                let p = self.ring_point(self.cfg.spawn_radius);
                self.spawn_enemy(st.elite, p);
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

    pub fn leaders(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.enemies.len()).filter(|&i| self.enemies.kind[i].is_leader() && self.enemies.hp[i] > 0.0)
    }

    fn update_banners(&mut self) {
        let leaders: Vec<Vec2> = self.leaders().map(|i| self.enemies.pos[i]).collect();
        let e = &mut self.enemies;
        for i in 0..e.len() {
            e.scatter[i] = (e.scatter[i] - DT).max(0.0);
            e.banner[i] = !leaders.is_empty()
                && !e.kind[i].is_leader()
                && !e.kind[i].is_boss()
                && leaders.iter().any(|&l| (e.pos[i] - l).length_sq() < BANNER_RADIUS * BANNER_RADIUS);
        }
    }

    /// Generic bosses: phases by HP, a rotating spray, and a cycle of moves from their kit.
    fn update_bosses(&mut self) {
        let player = self.player.pos;
        let extra = self.cfg.difficulty.extra_phases();
        let style = self.shot_style();
        let dmg = self.enemy_damage_scale();
        let domain = (self.form_kind() == Some(SpecialKind::Firmament)).then_some(self.domain_center);
        let dr = self.domain_radius();
        let tau = std::f32::consts::TAU;
        for i in 0..self.enemies.len() {
            let kind = self.enemies.kind[i];
            if enemy_def(kind).ai != Ai::Boss || self.enemies.hp[i] <= 0.0 {
                continue;
            }
            if !kind.is_boss() {
                let held = self.freeze > 0.0 || self.enemies.stagger[i] > 0.0 || domain.is_some_and(|c| (self.enemies.pos[i] - c).length_sq() < dr * dr);
                if held {
                    self.enemies.vel[i] = Vec2::ZERO;
                    continue;
                }
            }
            let kit = boss_kit(kind);
            let boss = Some(kind.element());
            let phases = boss_phases(kind, extra);
            let e = &mut self.enemies;
            let p = e.pos[i];
            let to = player - p;
            let base = to.y.atan2(to.x);
            let slow = match domain {
                Some(c) if (p - c).length_sq() < dr * dr => 0.15,
                _ => 1.0,
            };
            let slow = if !kind.is_boss() && self.cfg.hero == HeroId::Kuon && to.length_sq() < STILLNESS_RADIUS * STILLNESS_RADIUS {
                slow * STILLNESS_SLOW
            } else {
                slow
            };
            // Phase changes: each equal slice of HP lost.
            let frac = e.hp[i] / e.max_hp[i];
            let want = (((1.0 - frac) * phases as f32) as u8).min(phases - 1);
            if want > e.phase[i] {
                e.phase[i] = want;
                let n = 24 + 6 * want as usize;
                for k in 0..n {
                    let a = k as f32 / n as f32 * tau;
                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(a) * 120.0, life: 6.0, style, elem: boss });
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
                            self.shots.push(Shot { pos: p, vel: Vec2::from_angle(a) * sp.speed, life: 7.0, style, elem: boss });
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
                            self.shots.push(Shot { pos: p, vel: Vec2::from_angle(a) * v, life: 7.0, style, elem: boss });
                        }
                    }
                }
                Move::Fan { n, spread, speed } => {
                    let n = n as usize + 2 * e.phase[i] as usize;
                    for k in 0..n {
                        let a = base + (k as f32 / (n - 1).max(1) as f32 - 0.5) * spread;
                        self.shots.push(Shot { pos: p, vel: Vec2::from_angle(a) * (speed * (1.0 + 0.08 * ph)), life: 6.0, style, elem: boss });
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
                        self.shots.push(Shot { pos: q, vel: Vec2::from_angle(b) * 90.0, life: 6.0, style, elem: boss });
                    }
                    self.events.push(Event::Blink { pos: q });
                }
                Move::Rain { n } => {
                    let n = (n as f32 * more) as usize;
                    for k in 0..n {
                        let off = if k == 0 { Vec2::ZERO } else { Vec2::from_angle(self.rng.range(0.0, tau)) * self.rng.range(40.0, 170.0) };
                        self.hazards.push(Hazard { pos: player + off, radius: 34.0, warn: 1.1 + k as f32 * 0.06, life: 0.0, kind: HazardKind::Blast { damage: 14.0 * dmg }, tick: 0.0, friendly: false, spare_enemies: false });
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
                        self.shots.push(Shot { pos: start + side * x, vel: d * 75.0, life: 11.0, style, elem: None });
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
                    self.hazards.push(Hazard { pos: p + off, radius: 44.0, warn: 1.3, life: 0.0, kind: HazardKind::Blast { damage: 12.0 * dmg }, tick: 0.0, friendly: false, spare_enemies: false });
                }
            }
            Mechanic::Pressure => {
                self.next_hazard = gap(15.0, 8.0);
                let pos = p + Vec2::from_angle(self.rng.range(0.0, tau)) * 280.0;
                self.hazards.push(Hazard { pos, radius: 85.0, warn: 1.0, life: 22.0, kind: HazardKind::Pressure, tick: 0.0, friendly: false, spare_enemies: false });
            }
            Mechanic::Downpour => {
                self.next_hazard = gap(6.0, 3.0);
                let lead = if self.player.moving { self.player.facing * 30.0 } else { Vec2::ZERO };
                self.hazards.push(Hazard { pos: p + lead, radius: 36.0, warn: 1.2, life: 0.0, kind: HazardKind::Lightning { damage: 16.0 * dmg }, tick: 0.0, friendly: false, spare_enemies: false });
            }
            Mechanic::Domains => {
                self.next_hazard = gap(100.0, 55.0);
                let pos = p + Vec2::from_angle(self.rng.range(0.0, tau)) * self.rng.range(0.0, 80.0);
                self.hazards.push(Hazard { pos, radius: 210.0, warn: 3.0, life: 14.0, kind: HazardKind::Domain { damage: 6.0 * dmg }, tick: 1.0, friendly: false, spare_enemies: false });
            }
        }
    }

    /// True while the player stands in live spirit pressure.
    fn under_pressure(&self) -> bool {
        if self.cfg.gear.has_set(StageId::PaleCitadel) {
            return false;
        }
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
        let mut fired: Vec<(Vec2, f32, u8, bool, bool)> = Vec::new();
        let mut own_domains: Vec<(Vec2, f32, f32)> = Vec::new();
        let g = &self.cfg.gear;
        let (tag_safe, bolt_safe, domain_safe) = (g.has_set(StageId::EmberLeaf), g.has_set(StageId::Rainward), g.has_set(StageId::Shatterline));
        for h in &mut self.hazards {
            let inside = (h.pos - p).length_sq() <= (h.radius + PLAYER_RADIUS).powi(2);
            if h.warn > 0.0 {
                h.warn -= DT;
                if h.warn <= 0.0 {
                    match h.kind {
                        HazardKind::Blast { damage } | HazardKind::Lightning { damage } => {
                            let bolt = matches!(h.kind, HazardKind::Lightning { .. });
                            let safe = h.friendly || if bolt { bolt_safe } else { tag_safe };
                            if inside && !safe {
                                hurt = hurt.max(damage);
                            }
                            fired.push((h.pos, h.radius, bolt as u8, h.friendly, h.spare_enemies));
                            h.life = -1.0;
                        }
                        HazardKind::Domain { .. } => fired.push((h.pos, h.radius, 2, false, false)),
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
                        h.tick = if h.friendly { 0.5 } else { 1.0 };
                        if h.friendly {
                            own_domains.push((h.pos, h.radius, damage));
                        } else if inside && !domain_safe {
                            hurt = hurt.max(damage);
                        }
                    }
                }
                _ => {}
            }
        }
        self.hazards.retain(|h| h.warn > 0.0 || h.life > 0.0);
        let blast = 60.0 * self.enemy_hp_scale();
        for (pos, radius, dmg) in own_domains {
            self.query_circle(pos, radius);
            for k in 0..self.scratch.len() {
                let j = self.scratch[k] as usize;
                let dir = self.enemies.pos[j] - pos;
                self.hurt_enemy(j, dmg, dir, None);
            }
        }
        for (pos, radius, kind, friendly, spare) in fired {
            self.events.push(Event::Hazard { pos, radius, kind, friendly });
            let blast = if friendly { blast * 2.0 } else { blast };
            let hit_elem = friendly.then_some(if kind == 0 { Element::Wind } else { Element::Electric });
            if kind < 2 && !spare {
                // Tags and lightning don't take sides.
                self.query_circle(pos, radius);
                for k in 0..self.scratch.len() {
                    let j = self.scratch[k] as usize;
                    if !self.enemies.kind[j].is_boss() {
                        let dir = self.enemies.pos[j] - pos;
                        self.hurt_enemy(j, blast, dir, hit_elem);
                    }
                }
            }
        }
        if hurt > 0.0 && self.player.iframes <= 0.0 && self.cfg.stress.is_none() {
            self.hurt_player(hurt, None);
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
        let dr = self.domain_radius();
        let style = if self.cfg.stage == StageId::Shotengai { 0 } else { self.cfg.stage.index() as u8 };
        let e = &mut self.enemies;
        let frozen = self.freeze > 0.0;
        // Pass 1: desired velocity = seek player + separation from grid neighbours.
        for i in 0..e.pos.len() {
            if (frozen || e.stagger[i] > 0.0) && !e.kind[i].is_boss() {
                e.vel[i] = Vec2::ZERO;
                continue;
            }
            // Endless Firmament: everything inside is suspended (bosses only crawl).
            if let Some(c) = domain {
                if (e.pos[i] - c).length_sq() < dr * dr {
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
            if e.scatter[i] > 0.0 {
                e.vel[i] = (p - e.scatter_from[i]).normalize_or_zero() * speed;
                continue;
            }
            if e.banner[i] {
                speed *= BANNER_SPEED;
            }
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
                    let shooter = Some(e.kind[i].element());
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
                            Volley::Single { speed } => self.shots.push(Shot { pos: p, vel: dir * speed, life: 4.0, style, elem: shooter }),
                            Volley::Fan3 { speed } => {
                                for k in -1..=1 {
                                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(base + k as f32 * 0.28) * speed, life: 4.0, style, elem: shooter });
                                }
                            }
                            Volley::Ring6 { speed } => {
                                for k in 0..6 {
                                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(base + k as f32 / 6.0 * std::f32::consts::TAU) * speed, life: 4.0, style, elem: shooter });
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
                    if e.flee[i] > 0.0 {
                        e.flee[i] = (e.flee[i] - DT).max(0.0);
                        e.vel[i] = (p - player).normalize_or_zero() * speed;
                        continue;
                    }
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
                    let shooter = Some(e.kind[i].element());
                    // Slow chase; a constant spiral; every 3s cycle burst -> aimed fan -> summon.
                    e.vel[i] = to_player.normalize_or_zero() * speed;
                    e.ai_dir[i].x += DT * 2.2;
                    e.ai_dir[i].y -= DT;
                    if e.ai_dir[i].y <= 0.0 {
                        e.ai_dir[i].y = 0.14;
                        let a = e.ai_dir[i].x;
                        for k in 0..2 {
                            let b = a + k as f32 * std::f32::consts::PI;
                            self.shots.push(Shot { pos: p, vel: Vec2::from_angle(b) * 95.0, life: 6.0, style: 0, elem: shooter });
                        }
                    }
                    e.ai_timer[i] -= DT;
                    if e.ai_timer[i] <= 0.0 {
                        e.ai_timer[i] = 3.0;
                        match e.ai_state[i] {
                            0 => {
                                for k in 0..24 {
                                    let b = k as f32 / 24.0 * std::f32::consts::TAU;
                                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(b) * 110.0, life: 6.0, style: 0, elem: shooter });
                                }
                            }
                            1 => {
                                let base = to_player.y.atan2(to_player.x);
                                for k in 0..9 {
                                    let b = base + (k as f32 - 4.0) * 0.12;
                                    self.shots.push(Shot { pos: p, vel: Vec2::from_angle(b) * 150.0, life: 5.0, style: 0, elem: shooter });
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
            let slow = if e.slow[i] > 0.0 { SLOW_FACTOR } else { 1.0 };
            e.pos[i] += (e.vel[i] * slow + e.push[i]) * DT;
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
        let (c, r, dmg, kind, elem) = (self.projectiles.pos[i], self.projectiles.burst[i], self.projectiles.damage[i], self.projectiles.kind[i], self.projectiles.elem[i]);
        self.query_circle(c, r);
        for idx in 0..self.scratch.len() {
            let j = self.scratch[idx] as usize;
            let dir = self.enemies.pos[j] - c;
            self.hurt_enemy(j, dmg, dir, elem);
        }
        let st = match kind {
            ProjKind::Seal => style::SEAL,
            ProjKind::Familiar => style::FOX,
            ProjKind::Fang => style::VENOM,
            ProjKind::SealOrb => style::SEAL,
            _ => style::FOXFIRE,
        };
        if self.in_view(c) {
            self.events.push(Event::Burst { pos: c, radius: r, style: st });
        }
        if kind == ProjKind::Comet {
            let a0 = self.rng.range(0.0, std::f32::consts::TAU);
            for k in 0..3 {
                let a = a0 + k as f32 * std::f32::consts::TAU / 3.0;
                self.projectiles.push_kind(ProjKind::Spark, 5.0, c, Vec2::from_angle(a) * 240.0, 0.6, dmg * 0.4, 3, 5.0, elem);
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
            let elem = Some(w.kind.element());
            let area = s.area * self.area_mult();
            let loud = every >= 0.3; // continuous streams don't announce every shot
            match w.kind {
                WeaponKind::Ofuda => {
                    // Seal Storm: a stream of homing seals that explode.
                    let d = self.aim_dir();
                    let a = d.y.atan2(d.x) + self.rng.range(-0.5, 0.5);
                    self.projectiles.push_kind(ProjKind::Seal, 6.0, p, Vec2::from_angle(a) * 380.0, 1.6, dmg * 0.9, 1, 10.0, elem);
                    let last = self.projectiles.len() - 1;
                    self.projectiles.burst[last] = 28.0 * area;
                }
                WeaponKind::MoonOrbs => {
                    // Lunar Eclipse: six moonbeams sweep out from the player.
                    let rot = self.time * 0.9;
                    for k in 0..6 {
                        let a = rot + k as f32 * std::f32::consts::TAU / 6.0;
                        let to = p + Vec2::from_angle(a) * 340.0;
                        self.hurt_along(p, to, 12.0, dmg * 2.5, elem);
                        self.events.push(Event::Beam { from: p, to: self.clip_to_reach(p, to), style: style::MOON });
                    }
                }
                WeaponKind::SakuraField => {
                    // Hanafubuki: a spiral stream of cutting petals.
                    for arm in 0..2 {
                        let a = self.time * 7.0 + arm as f32 * std::f32::consts::PI;
                        let v = Vec2::from_angle(a) * 210.0 + Vec2::from_angle(a + 1.4) * 60.0;
                        self.projectiles.push_kind(ProjKind::Petal, 5.0, p, v, 1.3, dmg * 1.4, 3, 0.0, elem);
                    }
                }
                WeaponKind::ThunderChain => {
                    // Heavenfall: sky lightning on a random nearby enemy.
                    if let Some(t) = self.random_targets(320.0, 1).first().copied() {
                        self.blasts.push(Blast { pos: t, delay: 0.12, max_delay: 0.12, radius: 46.0 * area, damage: dmg * 1.4, style: style::SKYBOLT, burns: false, elem, follow: None });
                    }
                }
                WeaponKind::Taiko => {
                    // Festival Quake: a colossal shockwave.
                    self.waves.push(Wave { center: p, radius: 0.0, max_radius: self.reach_cap(320.0 * area), speed: 420.0, damage: dmg * 1.8, purify: false, elem });
                    self.events.push(Event::Burst { pos: p, radius: 60.0, style: style::QUAKE });
                }
                WeaponKind::StarFall => {
                    // Meteor: a giant meteor on the thickest part of the horde, leaving a crater.
                    if let Some(t) = self.random_targets(300.0, 1).first().copied() {
                        self.blasts.push(Blast { pos: t, delay: 0.7, max_delay: 0.7, radius: 110.0 * area, damage: dmg * 3.0, style: style::METEOR, burns: true, elem, follow: None });
                    }
                }
                WeaponKind::FlamePool => {
                    // Foxfire Trail: burning ground under the player's feet.
                    self.pools.push(Pool { pos: p, radius: 30.0 * area, life: 2.4, max_life: 2.4, tick: 0.0, damage: dmg, elem });
                }
                WeaponKind::FoxfireOrb => {
                    // Comet Rain: five blue comets fall on the horde.
                    for t in self.random_targets(320.0, 5) {
                        let d = self.rng.range(0.4, 0.9);
                        self.blasts.push(Blast { pos: t, delay: d, max_delay: d, radius: 50.0 * area, damage: dmg * 1.3, style: style::COMET, burns: false, elem, follow: None });
                    }
                }
                WeaponKind::GaleShuriken => {
                    // Cyclone: a ring of shuriken bursts out and returns.
                    for k in 0..10 {
                        let a = k as f32 / 10.0 * std::f32::consts::TAU;
                        self.projectiles.push_kind(ProjKind::Shuriken, 9.0 * area, p, Vec2::from_angle(a) * 300.0, s.duration, dmg, 255, 0.0, elem);
                    }
                }
                WeaponKind::FoxMirage => {
                    // Mirage Barrage: every clone hurls a foxfire orb.
                    let clones: Vec<Vec2> = self.clones.iter().map(|c| c.pos).collect();
                    for c in clones {
                        if let Some(j) = self.nearest_index(c, 220.0, &[]) {
                            let d = (self.enemies.pos[j] - c).normalize_or_zero();
                            self.projectiles.push_kind(ProjKind::Foxfire, 6.0, c, d * 280.0, 1.2, dmg * 1.5, 1, 0.0, elem);
                            let last = self.projectiles.len() - 1;
                            self.projectiles.burst[last] = 30.0 * area;
                        }
                    }
                }
                WeaponKind::GraveCleaver => {
                    // Judgement: a colossal slow crescent.
                    let d = self.aim_dir();
                    self.projectiles.push_kind(ProjKind::CrescentWave, 42.0 * area, p + d * 20.0, d * 230.0, 1.6, dmg * 3.0, 255, 0.0, elem);
                    self.events.push(Event::Slash { pos: p, dir: d, radius: 90.0, full: true });
                }
                WeaponKind::LanternVigil => {
                    // Lantern Volley: every lantern fires a spirit bolt.
                    let lanterns: Vec<Vec2> = self.lanterns.iter().map(|l| l.pos).collect();
                    for l in lanterns {
                        if let Some(j) = self.nearest_index(l, 200.0, &[]) {
                            let d = (self.enemies.pos[j] - l).normalize_or_zero();
                            self.projectiles.push_kind(ProjKind::SpiritBolt, 5.0, l + Vec2::new(0.0, -12.0), d * 320.0, 0.9, dmg * 0.9, 2, 6.0, elem);
                        }
                    }
                }
                WeaponKind::YinYangOrbs => {
                    // Yin-Yang Crash: a giant orb crashes onto the horde.
                    if let Some(t) = self.random_targets(300.0, 1).first().copied() {
                        self.blasts.push(Blast { pos: t, delay: 0.6, max_delay: 0.6, radius: 90.0 * area, damage: dmg * 4.0, style: style::YINYANG, burns: false, elem, follow: None });
                    }
                }
                WeaponKind::SpiritSeal => {
                    // Seal Ring: a full ring of seal orbs.
                    for k in 0..14 {
                        let a = k as f32 / 14.0 * std::f32::consts::TAU;
                        self.projectiles.push_kind(ProjKind::SealOrb, 7.0, p, Vec2::from_angle(a) * 300.0, 1.8, dmg, 1, 5.0, elem);
                        let last = self.projectiles.len() - 1;
                        self.projectiles.burst[last] = 26.0 * area;
                    }
                }
                WeaponKind::GravityWell => {
                    // Event Horizon: a vast singularity.
                    if let Some(t) = self.random_targets(260.0, 1).first().copied() {
                        self.wells.push(Well { pos: t, life: 1.6, max_life: 1.6, radius: 170.0 * area, damage: dmg * 2.0, tick: 0.0, elem });
                    }
                }
                WeaponKind::RepulsionStar => {
                    // Red Nova: a ring of crimson stars.
                    for k in 0..10 {
                        let a = k as f32 / 10.0 * std::f32::consts::TAU;
                        self.projectiles.push_kind(ProjKind::RedStar, 10.0 * area, p, Vec2::from_angle(a) * 340.0, 1.3, dmg, 255, 0.0, elem);
                    }
                }
                WeaponKind::VoidLance => {
                    // Void Cascade: three lances fan out.
                    let d = self.aim_dir();
                    for k in -1..=1 {
                        let a = d.y.atan2(d.x) + k as f32 * 0.55;
                        self.projectiles.push_kind(ProjKind::VoidOrb, 26.0 * area, p, Vec2::from_angle(a) * 170.0, 3.0, dmg * 1.3, 255, 0.0, elem);
                    }
                }
                WeaponKind::CrimsonTendrils => {
                    // Hundred Lashes: twelve tendrils tear through everything around.
                    self.lash_targets(p, 160.0 * area, 12, dmg * 1.2, elem);
                }
                WeaponKind::MukadeSpines => {
                    // Marching Spine: a line of spines crawls out along the aim.
                    let d = self.aim_dir();
                    for k in 0..9 {
                        let pos = p + d * (40.0 + 34.0 * k as f32);
                        let delay = 0.12 + 0.07 * k as f32;
                        self.blasts.push(Blast { pos, delay, max_delay: delay, radius: 34.0 * area, damage: dmg * 1.5, style: style::SPINE, burns: false, elem, follow: None });
                    }
                }
                WeaponKind::Devour => {
                    // Feeding Wave: a great draining shockwave that heals.
                    let r = self.reach_cap(200.0 * area);
                    self.query_circle(p, r);
                    let n = self.scratch.len();
                    for idx in 0..n {
                        let j = self.scratch[idx] as usize;
                        let dir = p - self.enemies.pos[j];
                        self.hurt_enemy(j, dmg * 2.0, dir * 0.5, elem);
                    }
                    self.heal((n as f32 * 0.5).min(15.0));
                    self.events.push(Event::Devour { pos: p, radius: r });
                }
                WeaponKind::VenomFang => {
                    // Fang Ring: a ring of homing fangs.
                    for k in 0..12 {
                        let a = k as f32 / 12.0 * std::f32::consts::TAU;
                        self.projectiles.push_kind(ProjKind::Fang, 6.0, p, Vec2::from_angle(a) * 280.0, 1.6, dmg, 1, 7.0, elem);
                        let last = self.projectiles.len() - 1;
                        self.projectiles.burst[last] = 26.0 * area;
                    }
                }
                WeaponKind::IaiFlash => {
                    // Thousand Cuts: a radial storm of twelve cuts.
                    for k in 0..12 {
                        let a = k as f32 / 12.0 * std::f32::consts::TAU + self.time;
                        let to = p + Vec2::from_angle(a) * 300.0;
                        self.hurt_along(p, to, 12.0, dmg * 0.8, elem);
                        self.events.push(Event::Iai { from: p, to: self.clip_to_reach(p, to) });
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
            if let Some(id) = self.blasts[i].follow {
                if let Some(j) = self.enemies.id.iter().position(|&x| x == id) {
                    if self.in_reach(self.enemies.pos[j], self.enemies.radius[j]) {
                        self.blasts[i].pos = self.enemies.pos[j];
                    } else {
                        self.blasts[i].follow = None;
                    }
                }
            }
            if self.blasts[i].delay > 0.0 {
                i += 1;
                continue;
            }
            let b = self.blasts.swap_remove(i);
            self.query_circle(b.pos, b.radius);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let dir = self.enemies.pos[j] - b.pos;
                self.hurt_enemy(j, b.damage, dir, b.elem);
            }
            if b.burns {
                self.pools.push(Pool { pos: b.pos, radius: b.radius * 0.6, life: 3.0, max_life: 3.0, tick: 0.0, damage: b.damage * 0.08, elem: b.elem });
            }
            if self.in_view(b.pos) {
                self.events.push(Event::Burst { pos: b.pos, radius: b.radius, style: b.style });
            }
        }
    }

    /// Lashes up to `n` distinct nearby enemies with tendrils that pierce along the line.
    fn lash_targets(&mut self, from: Vec2, reach: f32, n: u32, dmg: f32, elem: Option<Element>) {
        let mut taken: Vec<u32> = Vec::new();
        for _ in 0..n {
            let Some(j) = self.nearest_index(from, reach, &taken) else { break };
            taken.push(self.enemies.id[j]);
            let to = self.enemies.pos[j];
            // Reach a little past the target so the lash pierces whatever is behind it.
            let tip = from + (to - from).normalize_or_zero() * ((to - from).length() + 20.0);
            self.hurt_along(from, tip, 8.0, dmg, elem);
            self.events.push(Event::Lash { from, to: self.clip_to_reach(from, tip) });
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
        let elem = Some(self.hero_element);
        // Ofuda Storm: a stream of homing talismans in every direction.
        self.divine_ofuda -= DT;
        while self.divine_ofuda <= 0.0 {
            self.divine_ofuda += DIVINE_OFUDA_EVERY;
            for _ in 0..3 {
                let a = self.rng.range(0.0, std::f32::consts::TAU);
                self.projectiles.push_kind(ProjKind::Seal, 6.0, p, Vec2::from_angle(a) * 360.0, 1.6, DIVINE_OFUDA_DAMAGE * power, 1, 9.0, elem);
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
                self.blasts.push(Blast { pos: t, delay: d, max_delay: d, radius: 80.0, damage: DIVINE_ORB_DAMAGE * power, style: style::YINYANG, burns: false, elem, follow: None });
            }
        }
        // Spirit-seal barrage: rings of orbs in every direction.
        self.divine_ring -= DT;
        if self.divine_ring <= 0.0 {
            self.divine_ring = DIVINE_RING_EVERY;
            let rot = self.time;
            for k in 0..18 {
                let a = rot + k as f32 / 18.0 * std::f32::consts::TAU;
                self.projectiles.push_kind(ProjKind::SealOrb, 8.0, p, Vec2::from_angle(a) * 260.0, 1.8, DIVINE_RING_DAMAGE * power, 1, 2.0, elem);
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
                self.hurt_enemy(j, DIVINE_FINALE_DAMAGE * power, dir, elem);
                self.shove(j, dir, 450.0);
            }
            self.shots.clear();
            self.events.push(Event::Burst { pos: p, radius: self.reach_cap(DIVINE_FINALE_RADIUS), style: style::PURIFY });
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
        let elem = Some(self.hero_element);
        // Waves of spatial collapse over everything inside.
        self.fox_tick -= DT;
        if self.fox_tick <= 0.0 {
            self.fox_tick = DOMAIN_WAVE_EVERY;
            self.query_circle(c, self.domain_radius());
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let dir = self.enemies.pos[j] - c;
                self.hurt_enemy(j, DOMAIN_WAVE_DAMAGE * power, dir * 0.0, elem);
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
            self.projectiles.push_kind(ProjKind::VoidOrb, DOMAIN_FINALE_RADIUS, p, d * 260.0, 4.0, DOMAIN_FINALE_DAMAGE * power, 255, 0.0, elem);
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
        let elem = Some(self.hero_element);
        let radius = self.reach_cap(MUKADE_BURST_RADIUS);
        self.query_circle(p, radius);
        for idx in 0..self.scratch.len() {
            let j = self.scratch[idx] as usize;
            let dir = self.enemies.pos[j] - p;
            self.hurt_enemy(j, dmg, dir, elem);
            self.shove(j, dir, 400.0);
        }
        let r2 = radius * radius;
        self.shots.retain(|s| (s.pos - p).length_sq() > r2);
        self.events.push(Event::MukadeAwakened { pos: p });
        self.events.push(Event::Burst { pos: p, radius, style: style::FRENZY });
    }

    fn update_mukade(&mut self) {
        if self.form_kind() != Some(SpecialKind::Mukade) {
            return;
        }
        self.player.form -= DT;
        let power = self.fox_power();
        let elem = Some(self.hero_element);
        let p = self.player.pos;
        self.heal(MUKADE_REGEN * DT);
        // Tendril barrage.
        self.fox_tick -= DT;
        if self.fox_tick <= 0.0 {
            self.fox_tick = MUKADE_LASH_EVERY;
            let picks = self.random_targets(MUKADE_LASH_REACH, MUKADE_LASHES);
            for t in picks {
                let tip = p + (t - p).normalize_or_zero() * ((t - p).length() + 20.0);
                self.hurt_along(p, tip, 9.0, MUKADE_LASH_DAMAGE * power, elem);
                self.events.push(Event::Lash { from: p, to: self.clip_to_reach(p, tip) });
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
                self.blasts.push(Blast { pos, delay: d, max_delay: d, radius: 48.0, damage: MUKADE_ERUPT_DAMAGE * power, style: style::SPINE, burns: false, elem, follow: None });
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
        self.thieves_steal();
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
        let grab = ITEM_GRAB;
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
                    if !self.enemies.kind[j].is_boss() && self.in_view(self.enemies.pos[j]) {
                        let dir = self.enemies.pos[j] - p;
                        let hp = self.enemies.hp[j];
                        self.apply_hit(j, hp + 1.0, dir, None);
                    }
                }
                self.shots.clear();
            }
            PickupKind::SenPurse => self.gain_sen(40),
        }
        self.events.push(Event::PickupTaken { pos, kind });
    }

    fn thieves_steal(&mut self) {
        let domain = (self.form_kind() == Some(SpecialKind::Firmament)).then_some((self.domain_center, self.domain_radius()));
        let frozen = self.freeze > 0.0;
        for i in 0..self.enemies.len() {
            let e = &mut self.enemies;
            if e.kind[i] != EnemyKind::ToolThief || e.carry[i].is_some() || e.hp[i] <= 0.0 {
                continue;
            }
            let held = frozen || e.stagger[i] > 0.0 || domain.is_some_and(|(c, r)| (e.pos[i] - c).length_sq() < r * r);
            if e.ai_state[i] != 0 || held {
                continue;
            }
            let (pos, reach) = (e.pos[i], e.radius[i] + THIEF_REACH);
            let nearest = self.pickups.iter().enumerate().map(|(k, (c, _))| (k, (*c - pos).length_sq())).filter(|&(_, d)| d <= reach * reach).min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((k, _)) = nearest {
                let (at, kind) = self.pickups.swap_remove(k);
                e.carry[i] = Some(kind);
                e.flee[i] = THIEF_FLEE;
                self.events.push(Event::PickupStolen { pos: at, kind });
            }
        }
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
                if !self.enemies.kind[j].is_boss() && self.in_reach(self.enemies.pos[j], self.enemies.radius[j]) {
                    self.enemies.push[j] += to * (4.0 * DT * 60.0 / 10.0);
                }
            }
            if wl.tick <= 0.0 {
                wl.tick = 0.25;
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    self.hurt_enemy(j, wl.damage * 0.25, Vec2::ZERO, wl.elem);
                }
            }
            if wl.life <= 0.0 {
                self.query_circle(wl.pos, wl.radius * 0.8);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    self.hurt_enemy(j, wl.damage, Vec2::ZERO, wl.elem);
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
            let alive = j.filter(|&j| self.enemies.hp[j] > 0.0 && t.life > 0.0 && (self.enemies.pos[j] - p).length() < t.reach && self.in_reach(self.enemies.pos[j], self.enemies.radius[j]));
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
            if self.in_reach(self.enemies.pos[j], self.enemies.radius[j]) {
                self.enemies.push[j] += to_player.normalize_or_zero() * 9.0 * self.enemies.knock[j].max(0.2); // ~60 px/s drag once push decay settles
            }
            if t.tick <= 0.0 {
                t.tick = 0.25;
                self.hurt_enemy(j, t.damage, to_player, t.elem);
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
                    self.hurt_enemy(j, l.damage, dir, l.elem);
                    if l.slows && self.in_reach(self.enemies.pos[j], self.enemies.radius[j]) {
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
                self.hurt_along(s.from, s.to, 10.0, s.damage, s.elem);
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
                    self.hurt_enemy(j, c.damage * 2.0, dir, c.elem);
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
                    self.hurt_enemy(j, c.damage, c.facing, c.elem);
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
        let elem = Some(self.hero_element);
        let radius = self.reach_cap(FOX_BLAST_RADIUS);
        self.query_circle(p, radius);
        for idx in 0..self.scratch.len() {
            let j = self.scratch[idx] as usize;
            let dir = self.enemies.pos[j] - p;
            self.hurt_enemy(j, dmg, dir, elem);
            self.shove(j, dir, 420.0);
        }
        let r2 = radius * radius;
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
        out.extend(self.coins.iter().map(|(p, _)| MapMarker { pos: *p, kind: MarkerKind::Coin }));
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
        let elem = Some(self.hero_element);
        let radius = self.reach_cap(HANNYA_ONSET_RADIUS);
        self.query_circle(p, radius);
        for idx in 0..self.scratch.len() {
            let j = self.scratch[idx] as usize;
            let dir = self.enemies.pos[j] - p;
            self.hurt_enemy(j, dmg, dir, elem);
            self.shove(j, dir, 380.0);
        }
        let r2 = radius * radius;
        self.shots.retain(|s| (s.pos - p).length_sq() > r2);
        self.events.push(Event::HannyaAwakened { pos: p });
    }

    /// Damages every enemy within `width` of the segment once.
    fn hurt_along(&mut self, from: Vec2, to: Vec2, width: f32, dmg: f32, elem: Option<Element>) {
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
                    self.hurt_enemy(j, dmg, d, elem);
                }
            }
        }
        self.dead.clear();
    }

    fn update_hannya(&mut self) {
        let elem = Some(self.hero_element);
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
                    self.hurt_enemy(j, dmg, dir, elem);
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
                self.hurt_enemy(j, HANNYA_AURA_DAMAGE * power, dir, elem);
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
                self.hurt_along(p, to, 9.0, HANNYA_BEAM_DAMAGE * power, elem);
                self.events.push(Event::Beam { from: p, to: self.clip_to_reach(p, to), style: style::CRIMSON });
            }
        }
        // Crescent waves at the nearest enemy.
        self.hannya_wave -= DT;
        if self.hannya_wave <= 0.0 {
            self.hannya_wave = HANNYA_WAVE_EVERY;
            let d = self.aim_dir();
            self.projectiles.push_kind(ProjKind::CrescentWave, 18.0, p, d * 360.0, 0.8, HANNYA_WAVE_DAMAGE * power, 255, 0.0, elem);
            self.events.push(Event::Slash { pos: p, dir: d, radius: 46.0, full: false });
        }
        // Shadow Step: a crimson afterimage dashes into the horde and explodes there.
        // (The player stays put: teleporting the player itself was disorienting.)
        self.hannya_step -= DT;
        if self.hannya_step <= 0.0 {
            self.hannya_step = HANNYA_STEP_EVERY;
            for t in self.random_targets(260.0, 2) {
                self.afterimages.push((t, 0.35));
                self.events.push(Event::ShadowStep { from: p, to: self.clip_to_reach(p, t) });
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
                self.hurt_enemy(j, HANNYA_FINALE_DAMAGE * power, dir, elem);
                self.shove(j, dir, 450.0);
            }
            self.shots.clear();
            self.events.push(Event::FinaleBlast { pos: c, radius: self.reach_cap(HANNYA_FINALE_RADIUS) });
        }
    }

    fn update_fox(&mut self) {
        if self.form_kind() != Some(SpecialKind::NineTails) {
            return;
        }
        self.player.form -= DT;
        let power = self.fox_power();
        let elem = Some(self.hero_element);
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
                        self.hurt_enemy(j, FOX_TAIL_DAMAGE * power, e - p, elem);
                    }
                }
            }
            self.query_circle(p, FOX_AURA_RADIUS);
            for idx in 0..self.scratch.len() {
                let j = self.scratch[idx] as usize;
                let dir = self.enemies.pos[j] - p;
                self.hurt_enemy(j, FOX_AURA_DAMAGE * power, dir, elem);
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
                self.projectiles.push_kind(ProjKind::GoldenOrb, 13.0, p, Vec2::from_angle(a) * 250.0, 1.8, FOX_ORB_DAMAGE * power, 255, 0.0, elem);
            }
            self.events.push(Event::FoxVolley);
        }
        if self.player.form <= 0.0 {
            self.player.form = 0.0;
            self.events.push(Event::FoxFaded);
        }
    }

    pub fn reach_half(&self) -> Vec2 {
        Vec2::new(
            (REACH_HALF.x * self.reach_scale).min(self.view_half.x - REACH_EDGE).max(0.0),
            (REACH_HALF.y * self.reach_scale).min(self.view_half.y - REACH_EDGE).max(0.0),
        )
    }

    pub fn domain_radius(&self) -> f32 {
        self.reach_cap(DOMAIN_RADIUS)
    }

    /// Knockback that, like damage, stops at the reach box.
    fn shove(&mut self, j: usize, dir: Vec2, force: f32) {
        if self.in_reach(self.enemies.pos[j], self.enemies.radius[j]) {
            self.enemies.push[j] += dir.normalize_or_zero() * (force * self.enemies.knock[j]);
        }
    }

    pub fn in_reach(&self, pos: Vec2, radius: f32) -> bool {
        self.within_box(self.reach_half(), pos, radius)
    }

    /// `pos` is on screen; the centre must be visible, not just an edge of the body.
    pub fn in_view(&self, pos: Vec2) -> bool {
        self.within_box(self.view_half, pos, 0.0)
    }

    /// `r` limited to the reach box's long half-extent, so player-centred circles stay in reach.
    pub fn reach_cap(&self, r: f32) -> f32 {
        r.min(self.reach_half().x).max(1.0)
    }

    /// Where `from -> to` leaves the reach box (`from` inside it), or `to` if it never does.
    fn clip_to_reach(&self, from: Vec2, to: Vec2) -> Vec2 {
        let (c, h, d) = (self.player.pos, self.reach_half(), to - from);
        let mut t: f32 = 1.0;
        for (o, dv, hv, cv) in [(from.x, d.x, h.x, c.x), (from.y, d.y, h.y, c.y)] {
            if dv != 0.0 {
                t = t.min(((cv + hv * dv.signum() - o) / dv).max(0.0));
            }
        }
        from + d * t
    }

    fn within_box(&self, half: Vec2, pos: Vec2, radius: f32) -> bool {
        let d = pos - self.player.pos;
        d.x.abs() <= half.x + radius && d.y.abs() <= half.y + radius
    }

    fn hurt_enemy(&mut self, j: usize, amount: f32, dir: Vec2, elem: Option<Element>) {
        // Reach rule (spec "Attack reach").
        if !self.in_reach(self.enemies.pos[j], self.enemies.radius[j]) {
            return;
        }
        self.apply_hit(j, amount, dir, elem);
        if let Some(elem) = elem {
            let chance = if elem == Element::Wind && self.has_set6(StageId::EmberLeaf) { 2.0 * PROC_CHANCE } else { PROC_CHANCE };
            if self.rng.f32() < chance {
                self.on_hit_effect(j, elem, amount, dir);
            }
        }
    }

    fn has_set6(&self, s: StageId) -> bool {
        self.set6[s.index()]
    }

    /// `amount` is the raw hit, before element, affinity and crit multipliers.
    fn on_hit_effect(&mut self, j: usize, elem: Element, amount: f32, dir: Vec2) {
        let e = &mut self.enemies;
        match elem {
            Element::Fire => {
                e.burn[j] = BURN_SECONDS;
                e.burn_dps[j] = amount * BURN_SHARE / BURN_SECONDS;
            }
            Element::Water => e.soak[j] = SOAK_SECONDS,
            Element::Electric => {
                let mut from = j;
                let jumps = if self.has_set6(StageId::Shatterline) { 2 } else { 1 };
                let mut visited = [j, j];
                for n in 0..jumps {
                    let Some(k) = self.chain_target(from, &visited) else { break };
                    self.events.push(Event::Bolt { from: self.enemies.pos[from], to: self.enemies.pos[k] });
                    self.apply_hit(k, amount * CHAIN_SHARE, dir, Some(Element::Electric));
                    visited[n] = k;
                    from = k;
                }
            }
            Element::Earth => {
                if !e.kind[j].is_boss() && e.stagger_immune[j] <= 0.0 {
                    e.stagger[j] = STAGGER_SECONDS;
                    if e.kind[j].is_leader() {
                        e.stagger_immune[j] = STAGGER_SECONDS + LEADER_STAGGER_IMMUNE;
                    }
                }
            }
            Element::Wind => e.push[j] += dir.normalize_or_zero() * (KNOCK_EXTRA * e.knock[j]),
            Element::Ice => e.slow[j] = SLOW_SECONDS,
            Element::Light => e.marked[j] = true,
            Element::Darkness => self.heal(DRAIN_HEAL),
        }
    }

    fn chain_target(&self, j: usize, skip: &[usize; 2]) -> Option<usize> {
        let e = &self.enemies;
        let from = e.pos[j];
        let mut best: Option<(f32, usize)> = None;
        self.grid.for_each_near(from, CHAIN_RANGE, |k| {
            let k = k as usize;
            let d2 = (e.pos[k] - from).length_sq();
            if k != j
                && !skip.contains(&k)
                && e.hp[k] > 0.0
                && d2 <= CHAIN_RANGE * CHAIN_RANGE
                && self.in_reach(e.pos[k], e.radius[k])
                && best.is_none_or(|(b, _)| d2 < b) {
                best = Some((d2, k));
            }
        });
        best.map(|(_, k)| k)
    }

    fn tick_statuses(&mut self) {
        for j in 0..self.enemies.len() {
            let e = &mut self.enemies;
            for t in [&mut e.soak[j], &mut e.stagger[j], &mut e.stagger_immune[j], &mut e.slow[j]] {
                *t = (*t - DT).max(0.0);
            }
            if e.burn[j] <= 0.0 || e.hp[j] <= 0.0 {
                continue;
            }
            let before = e.burn[j];
            e.burn[j] = (before - DT).max(0.0);
            let ticked = (e.burn[j] / BURN_TICK).ceil() < (before / BURN_TICK).ceil();
            let (pos, radius, tick) = (e.pos[j], e.radius[j], e.burn_dps[j] * BURN_TICK);
            if ticked && self.in_reach(pos, radius) {
                self.apply_hit(j, tick, Vec2::ZERO, Some(Element::Fire));
            }
        }
    }

    fn apply_hit(&mut self, j: usize, amount: f32, dir: Vec2, elem: Option<Element>) {
        let mult = elem.map_or(1.0, |e| element_mult(e, self.enemies.kind[j].element()));
        let mut amount = amount * mult;
        if elem == Some(self.hero_element) {
            amount *= AFFINITY_DAMAGE;
        }
        if let Some(e) = elem {
            amount *= 1.0 + self.cfg.gear.elem_damage[e as usize];
        }
        if elem == Some(Element::Electric) && self.enemies.soak[j] > 0.0 {
            amount *= SOAK_ELECTRIC;
        }
        if elem == Some(Element::Water) && self.has_set6(StageId::Rainward) {
            self.enemies.soak[j] = SOAK_SECONDS;
        }
        if self.enemies.stagger[j] > 0.0 && self.has_set6(StageId::ShrineValley) {
            amount *= SET6_STAGGER;
        }
        if self.enemies.kind[j].rank() >= Rank::Elite {
            amount *= 1.0 + self.bonus.boss_damage;
        }
        let marked = std::mem::take(&mut self.enemies.marked[j]);
        let mut crit = marked || (self.bonus.crit_chance > 0.0 && self.rng.f32() < self.bonus.crit_chance);
        if self.cfg.gear.has(Exotic::ThousandCounter) {
            self.hit_count += 1;
            crit |= self.hit_count % 7 == 0;
        }
        if crit {
            amount *= 1.5 + self.bonus.crit_damage;
            let ecl = self.cfg.gear.unique(Unique::Eclipse);
            if ecl > 0.0 && self.eclipses.len() < 64 {
                self.eclipses.push((self.enemies.pos[j], amount * 0.4 * ecl));
            }
        }
        let e = &mut self.enemies;
        e.hp[j] -= amount;
        e.flash[j] = 5;
        e.push[j] += dir.normalize_or_zero() * (150.0 * e.knock[j]);
        self.events.push(Event::Hit { pos: e.pos[j], amount, crit, elem, mult });
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
            if d2 <= radius * radius
                && self.in_reach(e, self.enemies.radius[j as usize])
                && best.is_none_or(|(b, _)| d2 < b) {
                best = Some((d2, e));
            }
        });
        best.map(|(_, e)| e)
    }

    /// Rolls a piece of equipment for this stage, difficulty and Ascension.
    fn drop_gear(&mut self, pos: Vec2, min: Rarity) {
        let (st, d, a) = (self.cfg.stage, self.cfg.difficulty.index(), self.cfg.ascension);
        let r = gear::roll_rarity(&mut self.rng, gear::drop_tier(st, d, a), self.bonus.gear_find, min);
        let item = gear::roll_item(&mut self.rng, None, r, gear::drop_ilvl(st, d, a));
        self.events.push(Event::GearDrop { pos, rarity: r });
        self.loot.push(item);
    }

    /// Damage scale for gear-driven attacks: item power, might, and a slow climb over the run.
    fn gear_power(&self, item_power: f32) -> f32 {
        item_power * self.might() * (1.0 + self.time / 450.0)
    }

    fn nova(&mut self, c: Vec2, r: f32, dmg: f32, style: u8, elem: Option<Element>) {
        let r = self.reach_cap(r);
        self.query_circle(c, r);
        for k in 0..self.scratch.len() {
            let j = self.scratch[k] as usize;
            let dir = self.enemies.pos[j] - c;
            self.hurt_enemy(j, dmg, dir, elem);
        }
        self.events.push(Event::Burst { pos: c, radius: r, style });
    }

    /// Screen-clearing blast: hits everything on screen, not just inside reach.
    fn view_nova(&mut self, c: Vec2, r: f32, dmg: f32, style: u8, elem: Option<Element>) {
        self.query_circle(c, r);
        for k in 0..self.scratch.len() {
            let j = self.scratch[k] as usize;
            if self.in_view(self.enemies.pos[j]) {
                let dir = self.enemies.pos[j] - c;
                self.apply_hit(j, dmg, dir, elem);
            }
        }
        self.events.push(Event::Burst { pos: c, radius: r, style });
    }

    /// Positions of drum / dragon summons (for the client too).
    pub fn art_points(&self) -> Vec<Vec2> {
        let Some(a) = &self.art else { return Vec::new() };
        let p = self.player.pos;
        match a.art {
            ArsenalArt::RaijinDrums => {
                let n = if a.mythic { 5 } else { 3 };
                (0..n).map(|k| p + Vec2::from_angle(a.phase + k as f32 / n as f32 * std::f32::consts::TAU) * 74.0).collect()
            }
            ArsenalArt::DragonCoil => {
                let n = if a.mythic { 16 } else { 10 };
                (0..n).map(|k| p + Vec2::from_angle(a.phase * 1.3 - k as f32 * 0.24) * (112.0 + 10.0 * (a.phase * 2.0 + k as f32 * 0.5).sin())).collect()
            }
            _ => Vec::new(),
        }
    }

    /// Exclusive Arsenal attacks.
    fn update_art(&mut self) {
        let Some(mut a) = self.art.take() else { return };
        let p = self.player.pos;
        let pw = self.gear_power(a.power);
        let tau = std::f32::consts::TAU;
        a.timer -= DT;
        a.phase += DT * 1.6;
        let fire = a.timer <= 0.0;
        match a.art {
            ArsenalArt::RaijinDrums => {
                if fire {
                    a.timer = 1.2;
                    self.art = Some(a.clone());
                    for d in self.art_points() {
                        self.query_circle(d, 260.0);
                        let best = self.scratch.iter().map(|j| *j as usize).filter(|j| self.in_reach(self.enemies.pos[*j], self.enemies.radius[*j])).min_by(|x, y| (self.enemies.pos[*x] - d).length_sq().partial_cmp(&(self.enemies.pos[*y] - d).length_sq()).unwrap());
                        if let Some(j) = best {
                            let to = self.enemies.pos[j];
                            self.events.push(Event::Bolt { from: d, to });
                            self.hurt_enemy(j, 30.0 * pw, to - d, Some(a.art.element()));
                        }
                    }
                }
            }
            ArsenalArt::FoxProcession => {
                if fire {
                    a.timer = 6.0;
                    let n = if a.mythic { 10 } else { 6 };
                    let d = Vec2::from_angle(self.rng.range(0.0, tau));
                    let side = Vec2::new(-d.y, d.x);
                    for k in 0..n {
                        let off = (k as f32 - (n as f32 - 1.0) / 2.0) * 30.0;
                        a.foxes.push((p - d * (380.0 + (k % 2) as f32 * 40.0) + side * off, d * 430.0, 1.9));
                    }
                }
                for f in &mut a.foxes {
                    f.0 += f.1 * DT;
                    f.2 -= DT;
                }
                a.foxes.retain(|f| f.2 > 0.0);
                let foxes = a.foxes.clone();
                for (fp, fv, _) in foxes {
                    self.query_circle(fp, 18.0);
                    for k in 0..self.scratch.len() {
                        let j = self.scratch[k] as usize;
                        self.hurt_enemy(j, 90.0 * pw * DT, fv, Some(a.art.element()));
                    }
                }
            }
            ArsenalArt::TenguFan => {
                if fire {
                    a.timer = if a.mythic { 2.0 } else { 3.0 };
                    let dir = self.player.facing.normalize_or_zero();
                    let r = self.reach_cap(175.0 * self.area_mult());
                    self.query_circle(p, r);
                    for k in 0..self.scratch.len() {
                        let j = self.scratch[k] as usize;
                        let to = self.enemies.pos[j] - p;
                        let n = to.normalize_or_zero();
                        if n.x * dir.x + n.y * dir.y > 0.2 {
                            self.hurt_enemy(j, 70.0 * pw, to * 4.0, Some(a.art.element()));
                        }
                    }
                    self.events.push(Event::Slash { pos: p, dir, radius: r, full: false });
                }
            }
            ArsenalArt::DragonCoil => {
                self.art = Some(a.clone());
                for c in self.art_points() {
                    self.query_circle(c, 15.0);
                    for k in 0..self.scratch.len() {
                        let j = self.scratch[k] as usize;
                        let dir = self.enemies.pos[j] - p;
                        self.hurt_enemy(j, 75.0 * pw * DT, dir, Some(a.art.element()));
                    }
                }
            }
            ArsenalArt::Moonfall => {
                if fire {
                    a.timer = 7.0;
                    let n = if a.mythic { 7 } else { 4 };
                    for _ in 0..n {
                        let q = self.random_targets(self.reach_half().length(), 1)[0];
                        let delay = 0.7 + self.rng.range(0.0, 0.5);
                        self.blasts.push(Blast { pos: q, delay, max_delay: delay, radius: 70.0 * self.area_mult(), damage: 110.0 * pw, style: style::METEOR, burns: false, elem: Some(a.art.element()), follow: None });
                    }
                }
            }
            ArsenalArt::SkeletonHand => {
                if fire {
                    a.timer = if a.mythic { 4.0 } else { 6.0 };
                    // The densest of a few sampled spots.
                    let mut best = (p, 0usize);
                    for q in self.random_targets(self.reach_half().length(), 12) {
                        self.query_circle(q, 90.0);
                        if self.scratch.len() > best.1 {
                            best = (q, self.scratch.len());
                        }
                    }
                    self.blasts.push(Blast { pos: best.0, delay: 0.6, max_delay: 0.6, radius: 115.0 * self.area_mult(), damage: 220.0 * pw, style: style::QUAKE, burns: false, elem: Some(a.art.element()), follow: None });
                }
            }
            ArsenalArt::YinYangBreaker => {
                if a.orbs.is_empty() {
                    for k in 0..2 {
                        a.orbs.push((p + Vec2::new(if k == 0 { -60.0 } else { 60.0 }, 0.0), Vec2::from_angle(0.7 + k as f32 * 2.4) * 260.0));
                    }
                }
                let box_r = 230.0;
                for o in &mut a.orbs {
                    o.0 += o.1 * DT;
                    let rel = o.0 - p;
                    if rel.x.abs() > box_r {
                        o.1.x = -o.1.x.abs() * rel.x.signum();
                    }
                    if rel.y.abs() > box_r * 0.7 {
                        o.1.y = -o.1.y.abs() * rel.y.signum();
                    }
                }
                let orbs = a.orbs.clone();
                for (op, ov) in orbs {
                    self.query_circle(op, 26.0);
                    for k in 0..self.scratch.len() {
                        let j = self.scratch[k] as usize;
                        self.hurt_enemy(j, 170.0 * pw * DT, ov, Some(a.art.element()));
                    }
                    self.shots.retain(|s| (s.pos - op).length_sq() > 32.0 * 32.0);
                }
            }
            ArsenalArt::ShadowStar => {
                if fire {
                    a.timer = 4.0;
                    for k in 0..8 {
                        let to = p + Vec2::from_angle(k as f32 / 8.0 * tau + a.phase) * 280.0;
                        self.hurt_along(p, to, 14.0, 90.0 * pw, Some(a.art.element()));
                        self.events.push(Event::ShadowStep { from: p, to: self.clip_to_reach(p, to) });
                    }
                }
            }
            ArsenalArt::GrassCutter => {
                if fire {
                    a.timer = 4.0;
                    let r = self.reach_cap(230.0 * self.area_mult());
                    self.nova(p, r, 120.0 * pw, 1, Some(a.art.element()));
                    self.shots.retain(|s| (s.pos - p).length_sq() > r * r);
                    self.events.push(Event::Slash { pos: p, dir: self.player.facing, radius: r, full: true });
                }
            }
        }
        self.art = Some(a);
    }

    /// Exotic effects that tick.
    fn update_exotics(&mut self) {
        for (c, r, base) in std::mem::take(&mut self.pending_exo_blasts) {
            let dmg = self.gear_power(1.0) * base;
            self.view_nova(c, r, dmg, 2, Some(Exotic::ThousandCounter.element()));
        }
        let g = &self.cfg.gear;
        if g.exotics.is_empty() {
            return;
        }
        let (lantern, seals, hunger, regalia) = (g.has(Exotic::LanternHeart), g.has(Exotic::NineSeal) && self.specials_used >= 9, g.has(Exotic::HungerOrgan), g.regalia());
        let p = self.player.pos;
        for t in self.exo_timers.iter_mut() {
            *t -= DT;
        }
        if lantern && self.exo_timers[0] <= 0.0 {
            self.exo_timers[0] = 10.0;
            let d = 90.0 * self.gear_power(1.0);
            self.nova(p, 170.0 * self.area_mult(), d, 2, Some(Exotic::LanternHeart.element()));
        }
        if seals && self.exo_timers[1] <= 0.0 {
            self.exo_timers[1] = 2.0;
            let d = 20.0 * self.gear_power(1.0);
            self.nova(p, 90.0 * self.area_mult(), d, style::FOXFIRE, Some(Exotic::NineSeal.element()));
        }
        if regalia && self.exo_timers[2] <= 0.0 {
            self.exo_timers[2] = 30.0;
            self.shots.clear();
            self.events.push(Event::Burst { pos: p, radius: self.reach_cap(300.0), style: style::PURIFY });
        }
        if self.mask > 0.0 {
            self.mask -= DT;
            if self.mask <= 0.0 {
                self.player.hp = self.player.hp.min(self.player.max_hp * 0.5);
            }
        }
        if hunger {
            self.since_kill += DT;
            if self.since_kill > 3.0 {
                self.player.hp = (self.player.hp - 2.0 * DT).max(1.0);
            }
        }
    }

    /// Exotics that answer the special move.
    fn on_special_exotics(&mut self) {
        let g = &self.cfg.gear;
        if g.exotics.is_empty() {
            return;
        }
        let p = self.player.pos;
        let (branch, mask, fingers) = (g.has(Exotic::JeweledBranch), g.has(Exotic::HollowMask), g.has(Exotic::SovereignFinger) && g.fingers >= 5);
        if branch {
            // Last Word.
            self.shots.clear();
            let d = 150.0 * self.gear_power(1.0);
            self.nova(p, 320.0, d, style::PURIFY, Some(Exotic::JeweledBranch.element()));
        }
        if mask {
            self.mask = 6.0;
        }
        if fingers {
            let d = 40.0 * self.gear_power(1.0);
            self.hazards.push(Hazard { pos: p, radius: self.reach_cap(220.0), warn: 0.0, life: 4.0, kind: HazardKind::Domain { damage: d }, tick: 0.0, friendly: true, spare_enemies: false });
            self.events.push(Event::DomainOpened { pos: p });
        }
    }

    /// The Yata Mirror turns a bullet back on the nearest Otherworlder.
    fn reflect_shot(&mut self) {
        let p = self.player.pos;
        let e = &self.enemies;
        let best = (0..e.len()).filter(|&i| (e.pos[i] - p).length_sq() < 300.0 * 300.0 && self.in_reach(e.pos[i], e.radius[i])).min_by(|&a, &b| (e.pos[a] - p).length_sq().partial_cmp(&(e.pos[b] - p).length_sq()).unwrap());
        if let Some(j) = best {
            let to = self.enemies.pos[j];
            let d = 50.0 * self.gear_power(1.0);
            self.events.push(Event::Bolt { from: p, to });
            self.hurt_enemy(j, d, to - p, Some(Exotic::YataMirror.element()));
        }
    }

    /// Unique effects and 4-piece set strikes. Reads the grid.
    fn update_gear(&mut self) {
        for (pos, dmg) in std::mem::take(&mut self.eclipses) {
            self.query_circle(pos, 40.0);
            for k in 0..self.scratch.len() {
                let j = self.scratch[k] as usize;
                let dir = self.enemies.pos[j] - pos;
                self.hurt_enemy(j, dmg, dir, None);
            }
            self.events.push(Event::Burst { pos, radius: 40.0, style: 1 });
        }
        if self.cfg.stress.is_some() {
            return;
        }
        self.update_art();
        self.update_exotics();
        if self.cfg.gear.uniques.is_empty() && self.cfg.gear.sets4.is_empty() {
            return;
        }
        for t in self.gear_timers.iter_mut() {
            *t -= DT;
        }
        let p = self.player.pos;
        let power = self.might() * (1.0 + self.time / 600.0);
        let tc = self.cfg.gear.unique(Unique::Thunderclap);
        if tc > 0.0 && self.gear_timers[0] <= 0.0 {
            self.gear_timers[0] = 4.0;
            self.query_circle(p, 320.0);
            let mut near: Vec<usize> = self.scratch.iter().map(|j| *j as usize).filter(|&j| self.in_reach(self.enemies.pos[j], self.enemies.radius[j])).collect();
            near.sort_by(|a, b| (self.enemies.pos[*a] - p).length_sq().partial_cmp(&(self.enemies.pos[*b] - p).length_sq()).unwrap());
            for &j in near.iter().take((3.0 * tc) as usize) {
                let to = self.enemies.pos[j];
                self.events.push(Event::Bolt { from: p, to });
                self.hurt_enemy(j, 45.0 * power, to - p, Some(Element::Electric));
            }
        }
        let sb = self.cfg.gear.unique(Unique::Starburst);
        if sb > 0.0 && self.gear_timers[1] <= 0.0 {
            self.gear_timers[1] = 6.0 / sb;
            let r = self.reach_cap(120.0 * self.area_mult());
            self.query_circle(p, r);
            for k in 0..self.scratch.len() {
                let j = self.scratch[k] as usize;
                let dir = self.enemies.pos[j] - p;
                self.hurt_enemy(j, 35.0 * power, dir, Some(Element::Light));
            }
            self.events.push(Event::Burst { pos: p, radius: r, style: 1 });
        }
        let pick = |w: &World, r: f32| -> Option<Vec2> {
            let e = &w.enemies;
            (0..e.len()).filter(|&i| (e.pos[i] - p).length_sq() < r * r && w.in_reach(e.pos[i], e.radius[i])).min_by(|&a, &b| (e.pos[a] - p).length_sq().partial_cmp(&(e.pos[b] - p).length_sq()).unwrap()).map(|i| e.pos[i])
        };
        if self.cfg.gear.has_set(StageId::EmberLeaf) && self.gear_timers[2] <= 0.0 {
            self.gear_timers[2] = 6.0;
            if let Some(q) = pick(self, 260.0) {
                self.hazards.push(Hazard { pos: q, radius: 56.0, warn: 0.6, life: 0.0, kind: HazardKind::Blast { damage: 0.0 }, tick: 0.0, friendly: true, spare_enemies: false });
            }
        }
        if self.cfg.gear.has_set(StageId::Rainward) && self.gear_timers[3] <= 0.0 {
            self.gear_timers[3] = 3.0;
            if let Some(q) = pick(self, 250.0) {
                self.hazards.push(Hazard { pos: q, radius: 40.0, warn: 0.35, life: 0.0, kind: HazardKind::Lightning { damage: 0.0 }, tick: 0.0, friendly: true, spare_enemies: false });
            }
        }
    }

    /// Lantern Street and Pale Gate signatures: dying enemies act on the living before removal, and a death caused here does not trigger them again.
    fn pass_on_deaths(&mut self) {
        let lantern = self.has_set6(StageId::Shotengai);
        let pale = self.has_set6(StageId::PaleCitadel);
        if !lantern && !pale {
            return;
        }
        let shatter_cap = 200.0 * self.gear_power(1.0);
        let dying: Vec<usize> = (0..self.enemies.len()).filter(|&i| self.enemies.hp[i] <= 0.0).collect();
        for i in dying {
            let e = &self.enemies;
            let (pos, burn, burn_dps, slowed, max_hp) = (e.pos[i], e.burn[i], e.burn_dps[i], e.slow[i] > 0.0, e.max_hp[i]);
            if lantern && burn > 0.0 {
                if let Some(k) = self.chain_target(i, &[i, i]) {
                    self.enemies.burn[k] = burn;
                    self.enemies.burn_dps[k] = burn_dps;
                }
            }
            if pale && slowed {
                self.query_circle(pos, SET6_SHATTER_R);
                let amount = (SET6_SHATTER * max_hp).min(shatter_cap);
                for n in 0..self.scratch.len() {
                    let k = self.scratch[n] as usize;
                    let dir = self.enemies.pos[k] - pos;
                    self.hurt_enemy(k, amount, dir, Some(Element::Ice));
                }
            }
        }
    }

    /// A fallen Leader ends the banner at once; the crew near it runs.
    fn scatter_crews(&mut self) {
        let e = &mut self.enemies;
        let mut fell = false;
        for &d in &self.dead {
            if !e.kind[d].is_leader() {
                continue;
            }
            fell = true;
            let from = e.pos[d];
            self.events.push(Event::LeaderFell { pos: from });
            for i in 0..e.len() {
                if e.hp[i] > 0.0 && !e.kind[i].is_boss() && (e.pos[i] - from).length_sq() < BANNER_RADIUS * BANNER_RADIUS {
                    e.scatter[i] = SCATTER_TIME;
                    e.scatter_from[i] = from;
                }
            }
        }
        if fell {
            e.banner.fill(false);
        }
    }

    fn death_trick(&mut self, pos: Vec2, kind: EnemyKind) {
        match kind {
            EnemyKind::ClayBird => {
                let damage = CLAY_BLAST_DAMAGE * self.enemy_damage_scale();
                self.hazards.push(Hazard { pos, radius: CLAY_BLAST_RADIUS, warn: CLAY_BLAST_WARN, life: 0.0, kind: HazardKind::Blast { damage }, tick: 0.0, friendly: false, spare_enemies: true });
                if self.in_view(pos) {
                    self.events.push(Event::HazardWarn { pos, radius: CLAY_BLAST_RADIUS });
                }
            }
            EnemyKind::SwarmMother => {
                for k in 0..SWARM_SPLIT {
                    let angle = std::f32::consts::TAU * k as f32 / SWARM_SPLIT as f32;
                    self.pending_spawns.push((EnemyKind::CursedSpawn, pos + Vec2::from_angle(angle) * SWARM_SPLIT_RADIUS));
                }
            }
            _ => {}
        }
    }

    fn remove_dead_enemies(&mut self) {
        self.pass_on_deaths();
        self.dead.clear();
        self.dead.extend((0..self.enemies.len()).filter(|&i| self.enemies.hp[i] <= 0.0));
        self.scatter_crews();
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
                self.player.special = (self.player.special + self.mods.special_rate * (1.0 + self.bonus.special_charge) / SPECIAL_KILLS).min(1.0);
            }
            self.events.push(Event::Kill { pos, kind });
            if self.cfg.stress.is_none() {
                self.death_trick(pos, kind);
            }
            self.kills_by_kind[kind.index()] += 1;
            if let Some(k) = self.enemies.carry[i] {
                self.pickups.push((pos, k));
                self.events.push(Event::PickupSpawned { pos, kind: k });
            }
            if self.cfg.stress.is_none() {
                let find = 1.0 + self.bonus.gear_find;
                let (n, min) = match kind.rank() {
                    Rank::Fodder if kind.rarity() == EnemyRarity::Uncommon => ((self.rng.f32() < UNCOMMON_GEAR * find) as usize, Rarity::Common),
                    Rank::Fodder => ((self.rng.f32() < 0.0004 * find) as usize, Rarity::Common),
                    Rank::Elite => ((self.rng.f32() < 0.06 * find) as usize, Rarity::Common),
                    Rank::Mini => (1, Rarity::Rare),
                    Rank::Boss => (1 + (self.rng.f32() < 0.5) as usize, Rarity::Rare),
                    Rank::Final => (2, Rarity::Epic),
                };
                for _ in 0..n {
                    self.drop_gear(pos, min);
                }
                self.gear_kills += 1;
                self.since_kill = 0.0;
                let g = &self.cfg.gear;
                if g.has(Exotic::HungerOrgan) {
                    self.player.hp = (self.player.hp + self.player.max_hp * 0.004).min(self.player.max_hp);
                }
                if g.has(Exotic::LanternHeart) && self.gear_kills % 50 == 0 {
                    self.exo_timers[0] -= 1.0;
                }
                if g.has(Exotic::ThousandCounter) && self.gear_kills % 1000 == 0 {
                    self.pending_exo_blasts.push((self.player.pos, 450.0, 400.0));
                }
                // Exotics: only this world's final boss drops this world's Exotics, rarely.
                if kind.rank() == Rank::Final {
                    let chance = gear::EXOTIC_CHANCE * gear::exotic_mult(self.cfg.difficulty.index(), self.cfg.ascension) * find;
                    if self.rng.f32() < chance {
                        let pool: Vec<Exotic> = gear::ALL_EXOTICS.iter().copied().filter(|e| e.home() == self.cfg.stage).collect();
                        if !pool.is_empty() {
                            let e = pool[(self.rng.next_u32() as usize) % pool.len()];
                            let (st, d, a) = (self.cfg.stage, self.cfg.difficulty.index(), self.cfg.ascension);
                            let it = gear::roll_exotic(&mut self.rng, e, gear::drop_ilvl(st, d, a));
                            self.events.push(Event::GearDrop { pos, rarity: Rarity::Exotic });
                            self.loot.push(it);
                        }
                    }
                }
                if matches!(kind.rank(), Rank::Mini | Rank::Boss | Rank::Final) {
                    let chance = gear::SET_CHANCE * gear::exotic_mult(self.cfg.difficulty.index(), self.cfg.ascension) * find;
                    if self.rng.f32() < chance {
                        let (st, d, a) = (self.cfg.stage, self.cfg.difficulty.index(), self.cfg.ascension);
                        let it = gear::roll_set_piece(&mut self.rng, st, gear::drop_ilvl(st, d, a));
                        self.events.push(Event::GearDrop { pos, rarity: Rarity::Set });
                        self.loot.push(it);
                    }
                }
                let vamp = self.cfg.gear.unique(Unique::Vampiric);
                if vamp > 0.0 && self.gear_kills % 25 == 0 {
                    self.heal(self.player.max_hp * 0.01 * vamp);
                }
                if self.cfg.gear.has_set(StageId::PaleCitadel) && self.rng.f32() < 0.02 {
                    self.heal(2.0);
                }
            }
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
            let pickup_chance = if kind.is_leader() { LEADER_PICKUP_CHANCE } else { ONI_PICKUP_CHANCE };
            if kind.rank() == Rank::Elite && self.cfg.stress.is_none() && self.rng.f32() < pickup_chance * (1.0 + self.bonus.luck) && self.pickups.len() < MAX_PICKUPS {
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
            let elem = Some(w.kind.element());
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
                            self.projectiles.push_kind(ProjKind::Talisman, PROJECTILE_RADIUS, self.player.pos, Vec2::from_angle(a) * s.speed, life, s.damage * might, s.pierce, homing, elem);
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
                                self.hurt_enemy(j, s.damage * might, dir, elem);
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
                                self.hurt_enemy(j, s.damage * might, dir, elem);
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
                            self.hurt_enemy(j, s.damage * might, dir * 0.3, elem);
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
            if e.hp[j] <= 0.0 || exclude.contains(&e.id[j]) || !self.in_reach(e.pos[j], e.radius[j]) {
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
        let mut scratch = std::mem::take(&mut self.scratch);
        scratch.retain(|&j| self.in_reach(self.enemies.pos[j as usize], self.enemies.radius[j as usize]));
        self.scratch = scratch;
        let mut out = Vec::with_capacity(n as usize);
        for _ in 0..n {
            if self.scratch.is_empty() {
                let h = self.reach_half();
                out.push(self.player.pos + Vec2::new(self.rng.range(-h.x, h.x), self.rng.range(-h.y, h.y)));
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
        let elem = Some(w.kind.element());
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
                    self.hurt_enemy(j, dmg, to - from, elem);
                    from = to;
                    target = self.nearest_index(from, range, &hit);
                }
            }
            WeaponKind::Taiko => {
                self.events.push(Event::Fired(WeaponKind::Taiko));
                for n in 0..s.amount {
                    // Later waves start "behind" the first so they arrive as separate beats.
                    let start = -40.0 * n as f32;
                    self.waves.push(Wave { center: p, radius: start, max_radius: self.reach_cap(135.0 * area), speed: s.speed, damage: dmg, purify: false, elem });
                }
            }
            WeaponKind::StarFall => {
                let r = 24.0 * area;
                for pos in self.random_targets(260.0, s.amount) {
                    let delay = 0.45 + self.rng.range(0.0, 0.15);
                    self.strikes.push(Strike { pos, delay, radius: r, damage: dmg, elem });
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
                            self.hurt_enemy(j, dmg, to, elem);
                        }
                    }
                    self.events.push(Event::Slash { pos: p, dir: d, radius, full: w.awakened });
                    if w.awakened {
                        // Moonlit Waves: eight crescents fly out of the full-circle cut.
                        for k in 0..8 {
                            let wd = Vec2::from_angle(k as f32 * std::f32::consts::FRAC_PI_4 + self.time);
                            self.projectiles.push_kind(ProjKind::CrescentWave, 14.0 * area, p + wd * 24.0, wd * 330.0, 0.8, dmg * 0.7, 255, 0.0, elem);
                        }
                        break; // the full circle already covers every direction
                    }
                }
            }
            WeaponKind::FlamePool => {
                self.events.push(Event::Fired(WeaponKind::FlamePool));
                let r = 26.0 * area;
                for pos in self.random_targets(220.0, s.amount) {
                    self.pools.push(Pool { pos, radius: r, life: s.duration, max_life: s.duration, tick: 0.0, damage: dmg, elem });
                }
            }
            WeaponKind::FoxfireOrb => {
                self.events.push(Event::Fired(WeaponKind::FoxfireOrb));
                let kind = if w.awakened { ProjKind::Comet } else { ProjKind::Foxfire };
                let (vel, life) = (self.player.facing * s.speed, 2.0 * self.persistence());
                for n in 0..s.amount {
                    let pr = &mut self.projectiles;
                    pr.push_kind(kind, 7.0, p, vel, life, dmg, 1, 0.0, elem);
                    let last = pr.len() - 1;
                    // Extra orbs gather one after another.
                    pr.charge[last] = s.duration * (1.0 + 0.4 * n as f32);
                    pr.burst[last] = 34.0 * area;
                }
            }
            WeaponKind::Familiars => {
                self.events.push(Event::Fired(WeaponKind::Familiars));
                let life = s.duration;
                for n in 0..s.amount {
                    let a = self.rng.range(0.0, std::f32::consts::TAU) + n as f32;
                    self.projectiles.push_kind(ProjKind::Familiar, 6.0, p, Vec2::from_angle(a) * s.speed, life, dmg, s.pierce, 6.0, elem);
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
                            self.hurt_enemy(j, dmg, to, elem);
                        }
                    }
                    self.events.push(Event::Slash { pos: p, dir: d, radius, full: w.awakened });
                    // The cut looses a crescent wave (both ways once Awakened).
                    let waves: &[f32] = if w.awakened { &[0.0, std::f32::consts::PI] } else { &[0.0] };
                    for t in waves {
                        let wd = Vec2::from_angle(d.y.atan2(d.x) + t);
                        self.projectiles.push_kind(ProjKind::CrescentWave, 13.0 * area, p + wd * 20.0, wd * s.speed, 0.7, dmg * 0.6, 255, 0.0, elem);
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
                    self.tethers.push(Tether { enemy: id, life: s.duration, tick: 0.0, damage: dmg, reach: reach * 1.3, heals: w.awakened, elem });
                }
            }
            WeaponKind::LanternVigil => {
                self.events.push(Event::Fired(WeaponKind::LanternVigil));
                let life = s.duration;
                for _ in 0..s.amount {
                    let a = self.rng.range(0.0, std::f32::consts::TAU);
                    let d = self.rng.range(30.0, 110.0);
                    let pos = stage::resolve(self.cfg.stage, p + Vec2::from_angle(a) * d, 8.0);
                    self.lanterns.push(Lantern { pos, life, max_life: life, radius: 46.0 * area, tick: 0.2, damage: dmg, slows: w.awakened, elem });
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
                    self.hurt_along(p, to, 12.0, dmg, elem);
                    let tip = self.clip_to_reach(p, to);
                    self.events.push(Event::Iai { from: p, to: tip });
                    if w.awakened {
                        self.scars.push(Scar { from: p, to: tip, life: 1.5, tick: 0.3, damage: dmg * 0.25, elem });
                    }
                }
            }
            WeaponKind::CrimsonTendrils => {
                self.events.push(Event::Fired(WeaponKind::CrimsonTendrils));
                let reach = 120.0 * area;
                self.lash_targets(p, reach, s.amount, dmg, elem);
            }
            WeaponKind::MukadeSpines => {
                self.events.push(Event::Fired(WeaponKind::MukadeSpines));
                for (k, t) in self.random_targets(240.0, s.amount).into_iter().enumerate() {
                    let d = 0.3 + 0.06 * k as f32;
                    self.blasts.push(Blast { pos: t, delay: d, max_delay: d, radius: 30.0 * area, damage: dmg, style: style::SPINE, burns: false, elem, follow: None });
                }
            }
            WeaponKind::Devour => {
                let r = 70.0 * area;
                self.query_circle(p, r);
                let mut n = 0;
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    let dir = p - self.enemies.pos[j];
                    n += self.in_reach(self.enemies.pos[j], self.enemies.radius[j]) as u32;
                    self.hurt_enemy(j, dmg, dir * 0.3, elem);
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
                    self.projectiles.push_kind(ProjKind::Fang, 6.0, p, Vec2::from_angle(a) * s.speed, 1.6, dmg, 1, 7.0, elem);
                    let last = self.projectiles.len() - 1;
                    self.projectiles.burst[last] = 26.0 * area;
                }
            }
            WeaponKind::SpiritSeal => {
                self.events.push(Event::Fired(WeaponKind::SpiritSeal));
                let reach = self.reach_half();
                self.query_circle(p, reach.length());
                let mut scratch = std::mem::take(&mut self.scratch);
                scratch.retain(|&j| self.in_reach(self.enemies.pos[j as usize], self.enemies.radius[j as usize]));
                for _ in 0..s.amount {
                    if scratch.is_empty() {
                        break;
                    }
                    let j = scratch.swap_remove(self.rng.next_u32() as usize % scratch.len()) as usize;
                    let (pos, id) = (self.enemies.pos[j], self.enemies.id[j]);
                    self.blasts.push(Blast { pos, delay: SEAL_STAMP_DELAY, max_delay: SEAL_STAMP_DELAY, radius: SEAL_STAMP_RADIUS * area, damage: dmg, style: style::SEAL, burns: false, elem, follow: Some(id) });
                }
                self.scratch = scratch;
            }
            WeaponKind::PurificationWave => {
                self.events.push(Event::Fired(WeaponKind::PurificationWave));
                self.waves.push(Wave { center: p, radius: 0.0, max_radius: self.reach_cap(150.0 * area), speed: s.speed, damage: dmg, purify: true, elem });
                if w.awakened {
                    self.waves.push(Wave { center: p, radius: -70.0, max_radius: self.reach_cap(230.0 * area), speed: s.speed * 1.1, damage: dmg * 1.4, purify: true, elem });
                }
            }
            WeaponKind::GravityWell => {
                self.events.push(Event::Fired(WeaponKind::GravityWell));
                for t in self.random_targets(260.0, s.amount) {
                    self.wells.push(Well { pos: t, life: s.duration, max_life: s.duration, radius: 70.0 * area, damage: dmg, tick: 0.0, elem });
                }
            }
            WeaponKind::RepulsionStar => {
                self.events.push(Event::Fired(WeaponKind::RepulsionStar));
                let d = self.aim_dir();
                let n = s.amount as f32;
                for k in 0..s.amount {
                    let a = d.y.atan2(d.x) + (k as f32 - (n - 1.0) / 2.0) * 0.3;
                    self.projectiles.push_kind(ProjKind::RedStar, 10.0 * area, p, Vec2::from_angle(a) * s.speed, 1.4, dmg, 255, 0.0, elem);
                }
            }
            WeaponKind::VoidLance => {
                self.events.push(Event::Fired(WeaponKind::VoidLance));
                let d = self.aim_dir();
                let n = s.amount as f32;
                for k in 0..s.amount {
                    let a = d.y.atan2(d.x) + (k as f32 - (n - 1.0) / 2.0) * 0.6;
                    self.projectiles.push_kind(ProjKind::VoidOrb, 22.0 * area, p, Vec2::from_angle(a) * s.speed, 3.0, dmg, 255, 0.0, elem);
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
                        life: s.duration,
                        hits: s.pierce,
                        iframes: 0.5,
                        strike_cd: 0.3,
                        striking: 0.0,
                        damage: dmg,
                        pop_radius: 40.0 * area,
                        speed: s.speed,
                        slot,
                        elem,
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
                    self.projectiles.push_kind(ProjKind::Shuriken, 10.0 * area, p, Vec2::from_angle(a) * s.speed, s.duration, dmg, 255, 0.0, elem);
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
                        self.hurt_enemy(j, wv.damage, off * 2.0, wv.elem);
                        if wv.purify && self.in_reach(self.enemies.pos[j], self.enemies.radius[j]) {
                            self.enemies.push[j] += off.normalize_or_zero() * (380.0 * self.enemies.knock[j].max(0.3));
                        }
                    }
                }
                if wv.purify {
                    let (c, r) = (wv.center, wv.radius);
                    let mut shots = std::mem::take(&mut self.shots);
                    shots.retain(|s| ((s.pos - c).length() - r).abs() > 14.0 || !self.in_reach(s.pos, 0.0));
                    self.shots = shots;
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
                self.hurt_enemy(j, st.damage, dir, st.elem);
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
                    self.hurt_enemy(j, pl.damage, Vec2::ZERO, pl.elem);
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
                let hand = Vec2::new(if self.player.facing.x < 0.0 { -9.0 } else { 9.0 }, -6.0);
                self.projectiles.pos[i] = self.player.pos + hand;
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
                    if e.id[j] == skip || e.hp[j] <= 0.0 || !self.in_reach(e.pos[j], e.radius[j]) {
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
                if self.enemies.id[j] == self.projectiles.last_hit[i]
                    || self.enemies.hp[j] <= 0.0
                    || !self.in_reach(self.enemies.pos[j], self.enemies.radius[j])
                {
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
                let (dmg, vel, elem) = (self.projectiles.damage[i], self.projectiles.vel[i], self.projectiles.elem[i]);
                self.hurt_enemy(j, dmg, vel, elem);
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
        if self.cfg.gear.has(Exotic::ClosedHandVow) {
            return;
        }
        self.player.hp = (self.player.hp + amount * self.mods.healing).min(self.player.max_hp);
    }

    fn gain_xp(&mut self, amount: u32) {
        self.xp_carry += amount as f32 * XP_GAIN * self.mods.xp * (1.0 + self.bonus.growth);
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
        let mut hit_elem = None;
        self.shots.retain_mut(|s| {
            s.pos += s.vel * DT;
            s.life -= DT;
            if (s.pos - p).length_sq() <= hit_r * hit_r {
                hit = true;
                hit_elem = s.elem;
                return false;
            }
            s.life > 0.0
        });
        if self.cfg.gear.has(Exotic::JeweledBranch) && self.player.form <= 0.0 {
            let g2 = 28.0 * 28.0;
            let grazes = self.shots.iter().filter(|s| (s.pos - p).length_sq() < g2).count();
            self.player.special = (self.player.special + grazes as f32 * 0.0006).min(1.0);
        }
        if hit && self.cfg.gear.has_set(StageId::ShrineValley) && self.rng.f32() < 0.3 {
            hit = false;
        }
        if hit && self.cfg.gear.has(Exotic::YataMirror) && self.rng.f32() < 0.25 {
            hit = false;
            self.reflect_shot();
        }
        if hit && self.cfg.stress.is_none() && self.player.iframes <= 0.0 {
            if self.cfg.hero == HeroId::Ren {
                // Barrier: blocks a fireball outright when charged; otherwise softens it.
                if self.barrier_cd <= 0.0 {
                    self.barrier_cd = barrier_recharge(self.player.level);
                    self.events.push(Event::BarrierBlock);
                    return;
                }
                self.hurt_player(SHOT_DAMAGE * self.enemy_damage_scale() * BARRIER_SHOT_DAMAGE, hit_elem);
                return;
            }
            self.hurt_player(SHOT_DAMAGE * self.enemy_damage_scale(), hit_elem);
        }
    }

    fn collect_chests(&mut self) {
        let p = self.player.pos;
        let grab = ITEM_GRAB;
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
                !w.awakened && !w.extra && w.level >= MAX_LEVEL && self.passive_level(p) >= need
            })
            .map(|w| w.kind)
            .collect()
    }

    /// Rolls the reward count (85% 1 / 10% 2 / 4% 3 / 1% 5 at the start, about 50/33/13/3 from
    /// 10:00, at least `min_rewards`), then fills it: an Awakening first if one is eligible, then levels.
    pub fn open_chest(&mut self, min_rewards: u8) {
        self.chests_opened += 1;
        // Luck and time push the roll toward the rarer reward counts.
        let roll = 1.0 - (1.0 - self.rng.f32()) / (1.0 + self.bonus.luck + chest_time_luck(self.round_pace()));
        let extra = self.cfg.gear.unique(Unique::TreasureSense) as u8;
        let tier = chest_roll(roll).max(min_rewards) + extra;
        if self.cfg.stress.is_none() && self.rng.f32() < 0.05 * (1.0 + self.bonus.gear_find) {
            let p = self.player.pos;
            self.drop_gear(p, Rarity::Common);
        }
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
            // Chests level what you own first, then fill empty slots.
            let mut pool = self.upgrade_pool(false);
            if pool.is_empty() {
                pool = self.upgrade_pool(true);
            }
            if pool.is_empty() {
                let u = MAXED_FALLBACKS[(self.rng.next_u32() as usize) % MAXED_FALLBACKS.len()];
                let amount = self.fallback_amount(u);
                self.apply_upgrade(u);
                items.push(ChestItem::Extra(u, amount));
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

    fn special_ready(&self) -> bool {
        self.player.special >= 1.0 && self.player.form <= 0.0
    }

    /// Summed `Rank::threat` of live enemies within `radius` of the player. Uses this tick's
    /// grid, so call it before `remove_dead_enemies`.
    fn threat_near(&self, radius: f32) -> f32 {
        let (p, e, r2) = (self.player.pos, &self.enemies, radius * radius);
        let mut threat = 0.0;
        self.grid.for_each_near(p, radius, |i| {
            let i = i as usize;
            if e.hp[i] > 0.0 && (e.pos[i] - p).length_sq() < r2 {
                threat += e.kind[i].rank().threat();
            }
        });
        threat
    }

    fn wants_auto_special(&self) -> bool {
        if !self.cfg.auto_special || !self.special_ready() {
            return false;
        }
        self.player.hp < self.player.max_hp * AUTO_SPECIAL_HP && self.threat_near(AUTO_SPECIAL_RADIUS) >= AUTO_SPECIAL_THREAT
    }

    fn use_special(&mut self) {
        if !self.special_ready() {
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
        let elem = Some(self.hero_element);
        for idx in 0..self.scratch.len() {
            let j = self.scratch[idx] as usize;
            let dir = self.enemies.pos[j] - p;
            self.hurt_enemy(j, dmg, dir, elem);
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
                let open: Vec<usize> = (0..self.weapons.len()).filter(|&i| self.weapons[i].level < MAX_LEVEL && !self.weapons[i].extra).collect();
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

    /// Levels for owned weapons and passives, plus new ones for empty slots when `new_items`.
    fn upgrade_pool(&self, new_items: bool) -> Vec<Upgrade> {
        let mut pool: Vec<Upgrade> = Vec::new();
        pool.extend(self.weapons.iter().filter(|w| w.level < MAX_LEVEL && !w.extra).map(|w| Upgrade::Weapon(w.kind)));
        pool.extend(self.passives.iter().filter(|(k, l)| *l < passive_max(*k)).map(|(k, _)| Upgrade::Passive(*k)));
        if !new_items {
            return pool;
        }
        let hero = hero_def(self.cfg.hero);
        if self.weapons.iter().filter(|w| !w.extra).count() < hero.weapon_slots {
            pool.extend(hero.weapon_pool().into_iter().filter(|k| self.weapon_level(*k) == 0).map(Upgrade::Weapon));
        }
        if self.passives.len() < hero.passive_slots {
            pool.extend(ALL_PASSIVES.iter().filter(|k| self.passive_level(**k) == 0).map(|k| Upgrade::Passive(*k)));
        }
        pool
    }

    fn weighted_index(&mut self, weights: &[f32]) -> usize {
        let mut roll = self.rng.f32() * weights.iter().sum::<f32>();
        for (i, w) in weights.iter().enumerate() {
            roll -= w;
            if roll < 0.0 {
                return i;
            }
        }
        weights.len() - 1
    }

    fn offer_weight(&self, u: Upgrade) -> f32 {
        let owned = match u {
            Upgrade::Weapon(k) => self.weapon_level(k) > 0,
            Upgrade::Passive(k) => self.passive_level(k) > 0,
            _ => true,
        };
        if owned { 1.0 } else { u.tier().weight() }
    }

    fn weapon_slots_full(&self) -> bool {
        self.weapons.iter().filter(|w| !w.extra).count() >= hero_def(self.cfg.hero).weapon_slots
    }

    fn passive_slots_full(&self) -> bool {
        self.passives.len() >= hero_def(self.cfg.hero).passive_slots
    }

    /// An unowned skill that can only be taken by overwriting an owned one.
    pub fn is_replacement(&self, u: Upgrade) -> bool {
        match u {
            Upgrade::Weapon(k) => self.weapon_level(k) == 0 && self.weapon_slots_full(),
            Upgrade::Passive(k) => self.passive_level(k) == 0 && self.passive_slots_full(),
            _ => false,
        }
    }

    fn replacement_candidates(&self) -> Vec<Upgrade> {
        let mut out = Vec::new();
        if self.weapon_slots_full() {
            out.extend(hero_def(self.cfg.hero).weapon_pool().into_iter().filter(|k| self.weapon_level(*k) == 0).map(Upgrade::Weapon));
        }
        if self.passive_slots_full() {
            out.extend(ALL_PASSIVES.iter().filter(|k| self.passive_level(**k) == 0).map(|k| Upgrade::Passive(*k)));
        }
        out
    }

    /// A maxed build still gets one replacement card, favouring the passive that would awaken a weapon.
    fn maxed_offer(&mut self, candidates: &[Upgrade]) -> Vec<Upgrade> {
        let awakening: Vec<Upgrade> = self
            .weapons
            .iter()
            .filter(|w| !w.extra && !w.awakened && w.level >= MAX_LEVEL)
            .map(|w| Upgrade::Passive(complement(w.kind).0))
            .filter(|u| candidates.contains(u))
            .collect();
        let from = if awakening.is_empty() { candidates } else { &awakening };
        let weights: Vec<f32> = from.iter().map(|u| u.tier().weight()).collect();
        let replacement = from[self.weighted_index(&weights)];
        let mut fallbacks = MAXED_FALLBACKS.to_vec();
        let skip = (self.rng.next_u32() as usize) % fallbacks.len();
        fallbacks.remove(skip);
        let mut offer = fallbacks;
        let at = (self.rng.next_u32() as usize) % (offer.len() + 1);
        offer.insert(at, replacement);
        self.unique_offered |= offer.iter().any(|u| u.tier() == Tier::Unique);
        offer
    }

    fn make_offer(&mut self) {
        let mut pool = self.upgrade_pool(true);
        let candidates = self.replacement_candidates();
        if pool.is_empty() {
            let replace = !candidates.is_empty() && self.rng.f32() < MAXED_REPLACE_CHANCE;
            self.offer = Some(if replace { self.maxed_offer(&candidates) } else { MAXED_FALLBACKS.to_vec() });
            return;
        }
        let mut offer = Vec::new();
        let mut replacement = None;
        if !candidates.is_empty() && self.rng.f32() < REPLACE_CHANCE {
            let weights: Vec<f32> = candidates.iter().map(|u| u.tier().weight()).collect();
            replacement = Some(candidates[self.weighted_index(&weights)]);
        }
        let regular = if replacement.is_some() { 2 } else { 3 };
        while offer.len() < regular && !pool.is_empty() {
            let weights: Vec<f32> = pool.iter().map(|u| self.offer_weight(*u)).collect();
            let i = self.weighted_index(&weights);
            offer.push(pool.swap_remove(i));
        }
        if let Some(r) = replacement {
            let at = (self.rng.next_u32() as usize) % (offer.len() + 1);
            offer.insert(at, r);
        }
        self.unique_offered |= offer.iter().any(|u| u.tier() == Tier::Unique);
        self.offer = Some(offer);
    }

    fn note_slots(&mut self) {
        if self.slots_full_round.is_none() && self.weapon_slots_full() {
            self.slots_full_round = Some(self.round());
        }
    }

    /// Level the upgrade would reach if picked (for UI).
    pub fn next_level(&self, u: Upgrade) -> u8 {
        match u {
            Upgrade::Weapon(k) => self.weapon_level(k) + 1,
            Upgrade::Passive(k) => self.passive_level(k) + 1,
            Upgrade::Heal | Upgrade::Sen | Upgrade::Blessing => 1,
        }
    }

    /// Apply choice `i` of the current offer. A replacement card waits for `replace_with`.
    pub fn choose(&mut self, i: usize) {
        if self.replacing.is_some() {
            return;
        }
        let Some(&u) = self.offer.as_ref().and_then(|o| o.get(i)) else { return };
        if self.is_replacement(u) {
            self.replacing = Some(u);
            return;
        }
        self.offer = None;
        self.apply_upgrade(u);
        self.finish_pick();
    }

    fn finish_pick(&mut self) {
        self.pending_levels = self.pending_levels.saturating_sub(1);
        if self.pending_levels > 0 {
            self.make_offer();
        }
    }

    /// Owned skills the pending replacement card could overwrite, with their levels, in slot order.
    pub fn replace_targets(&self) -> Vec<(Upgrade, u8)> {
        self.replace_targets_for(self.replacing)
    }

    pub(crate) fn replace_targets_for(&self, new: Option<Upgrade>) -> Vec<(Upgrade, u8)> {
        match new {
            Some(Upgrade::Weapon(_)) => self.weapons.iter().filter(|w| !w.extra).map(|w| (Upgrade::Weapon(w.kind), w.level)).collect(),
            Some(Upgrade::Passive(_)) => self.passives.iter().map(|(k, l)| (Upgrade::Passive(*k), *l)).collect(),
            _ => Vec::new(),
        }
    }

    /// `None` cancels back to the same offer; `Some(j)` overwrites `replace_targets()[j]` at half its level.
    pub fn replace_with(&mut self, target: Option<usize>) {
        let Some(new) = self.replacing else { return };
        let Some(j) = target else {
            self.replacing = None;
            return;
        };
        let Some(&(old, old_level)) = self.replace_targets().get(j) else { return };
        self.replacing = None;
        let level = inherit_level(new, old_level);
        match (old, new) {
            (Upgrade::Weapon(_), Upgrade::Weapon(k)) => {
                let slot = self.weapons.iter().enumerate().filter(|(_, w)| !w.extra).nth(j).map(|(i, _)| i).expect("replace target index checked above");
                self.retired_weapons.push(self.weapons[slot]);
                self.weapons[slot] = OwnedWeapon { kind: k, level, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 };
            }
            (Upgrade::Passive(_), Upgrade::Passive(k)) => {
                self.retired_passives.push(self.passives[j]);
                if let Upgrade::Passive(PassiveKind::Vitality) = old {
                    // shortcut: flat subtraction ignores Glass Blade's ×0.7, track granted HP if players notice
                    self.player.max_hp = (self.player.max_hp - VITALITY_HP * old_level as f32).max(10.0);
                }
                self.passives[j] = (k, level);
                if k == PassiveKind::Vitality {
                    self.player.max_hp += VITALITY_HP * level as f32;
                    self.heal(VITALITY_HP * level as f32);
                }
                self.player.hp = self.player.hp.min(self.player.max_hp);
            }
            _ => {}
        }
        self.replacements += 1;
        self.note_slots();
        self.offer = None;
        self.finish_pick();
    }

    /// What a maxed-out fallback gives right now: HP healed, Sen, or damage fraction gained.
    /// The one place to tune how they scale over a run.
    pub fn fallback_amount(&self, u: Upgrade) -> f32 {
        match u {
            Upgrade::Heal => self.player.max_hp * ONIGIRI_HEAL,
            Upgrade::Sen => SEN_POUCH * self.cfg.difficulty.greed() * (1.0 + self.round_pace() / SEN_POUCH_RAMP),
            Upgrade::Blessing => BLESSING_DAMAGE - 1.0,
            _ => 0.0,
        }
    }

    /// Card text for `u`, with live amounts for the maxed-out fallbacks.
    pub fn describe_upgrade(&self, u: Upgrade, next_level: u8) -> String {
        if MAXED_FALLBACKS.contains(&u) { describe_fallback(u, self.fallback_amount(u)) } else { u.describe(next_level) }
    }

    fn apply_upgrade(&mut self, u: Upgrade) {
        match u {
            Upgrade::Weapon(k) => match self.weapons.iter_mut().find(|w| w.kind == k) {
                Some(w) => w.level += 1,
                None => self.weapons.push(OwnedWeapon { kind: k, level: 1, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }),
            },
            Upgrade::Passive(k) => {
                match self.passives.iter_mut().find(|(pk, _)| *pk == k) {
                    Some((_, l)) => *l += 1,
                    None => self.passives.push((k, 1)),
                }
                if k == PassiveKind::Vitality {
                    self.player.max_hp += VITALITY_HP;
                    self.heal(VITALITY_HP);
                }
            }
            Upgrade::Heal => self.heal(self.fallback_amount(u)),
            Upgrade::Sen => self.gain_sen(self.fallback_amount(u).round() as u32),
            Upgrade::Blessing => self.mods.damage *= 1.0 + self.fallback_amount(u),
        }
        self.note_slots();
    }

    /// Dev shortcut: fill the hero's weapon slots at max level with their complements, ready to Awaken.
    pub fn debug_max_loadout(&mut self) {
        let hero = hero_def(self.cfg.hero);
        for k in hero.weapon_pool().into_iter().take(hero.weapon_slots) {
            match self.weapons.iter_mut().find(|w| w.kind == k) {
                Some(w) => w.level = MAX_LEVEL,
                None => self.weapons.push(OwnedWeapon { kind: k, level: MAX_LEVEL, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }),
            }
            let (c, need) = complement(k);
            match self.passives.iter_mut().find(|(pk, _)| *pk == c) {
                Some((_, l)) => *l = (*l).max(need),
                None => self.passives.push((c, need)),
            }
        }
    }

    /// Dev shortcut: Awaken every owned weapon.
    /// Dev: defeats every non-boss enemy on the field (they drop as usual).
    pub fn debug_clear_enemies(&mut self) {
        for i in 0..self.enemies.len() {
            if !self.enemies.kind[i].is_boss() {
                self.enemies.hp[i] = 0.0;
            }
        }
    }

    /// Dev: the stage's Leader with its crew 120 units away, plus 3 of each Uncommon beside the player.
    pub fn debug_spawn_newcomers(&mut self) {
        if !self.leader_alive() {
            self.spawn_leader_at(self.player.pos + Vec2::new(120.0, 0.0));
        }
        let st = self.stage();
        for k in 0..6 {
            let p = self.player.pos + Vec2::from_angle(k as f32 / 6.0 * std::f32::consts::TAU) * 90.0;
            self.spawn_enemy(st.uncommon[k % 2], p);
        }
    }

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
                self.weapons.push(OwnedWeapon { kind: k, level: 5, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 });
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
        play_as(HeroId::Ren, seed, ticks)
    }

    fn play_as(hero: HeroId, seed: u64, ticks: u32) -> World {
        let mut w = World::new(WorldConfig { seed, hero, ..Default::default() });
        for t in 0..ticks {
            if w.offer.is_some() {
                w.bot_pick();
            }
            w.close_chest();
            w.choose_shrine(None);
            let a = t as f32 * 0.01;
            w.step(Input { move_dir: Vec2::new(a.cos(), a.sin()), ..Default::default() });
        }
        w
    }

    fn world_with_enemy_at(dx: f32) -> World {
        let mut w = World::new(WorldConfig::default());
        let at = w.player.pos + Vec2::new(dx, 0.0);
        w.spawn_enemy(EnemyKind::Wisp, at);
        w
    }

    fn hit_hp_lost(w: &mut World, elem: Option<Element>) -> f32 {
        let before = w.enemies.hp[0];
        w.hurt_enemy(0, 5.0, Vec2::new(1.0, 0.0), elem);
        before - w.enemies.hp[0]
    }

    #[test]
    fn out_of_reach_enemies_take_no_damage() {
        let mut w = world_with_enemy_at(REACH_HALF.x + 40.0);
        assert_eq!(hit_hp_lost(&mut w, None), 0.0);
        w.enemies.pos[0] = w.player.pos + Vec2::new(100.0, 0.0);
        assert!(hit_hp_lost(&mut w, None) > 0.0);
    }

    #[test]
    fn reach_ignores_extra_screen_width() {
        let mut w = world_with_enemy_at(REACH_HALF.x + 40.0);
        w.view_half = Vec2::new(900.0, 180.0);
        assert_eq!(hit_hp_lost(&mut w, None), 0.0);
    }

    #[test]
    fn element_multiplies_damage() {
        let lost = |kind: EnemyKind, hero: HeroId, elem: Option<Element>| {
            let mut w = World::new(WorldConfig { hero, ..Default::default() });
            w.spawn_enemy(kind, w.player.pos + Vec2::new(40.0, 0.0));
            let hp = hit_hp_lost(&mut w, elem);
            let mult = w.events.iter().find_map(|e| if let Event::Hit { mult, .. } = e { Some(*mult) } else { None });
            (hp, mult.unwrap())
        };
        assert_eq!(lost(EnemyKind::Wisp, HeroId::Ren, None), (5.0, 1.0));
        assert_eq!(lost(EnemyKind::Wisp, HeroId::Ren, Some(Element::Water)), (7.5, 1.5));
        assert_eq!(lost(EnemyKind::Wisp, HeroId::Ren, Some(Element::Ice)).0, 3.75);
        assert!((lost(EnemyKind::Nopperabo, HeroId::Riku, Some(Element::Fire)).0 - 5.5).abs() < 1e-4);
    }

    #[test]
    fn every_weapon_hit_carries_its_element() {
        for k in ALL_WEAPONS {
            for awakened in [false, true] {
                let mut w = World::new(WorldConfig { seed: 5, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
                w.time = AWAKEN_FROM + 1.0;
                w.weapons = vec![OwnedWeapon { kind: k, level: MAX_LEVEL, awakened, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }];
                for n in 0..12 {
                    let a = n as f32 / 12.0 * std::f32::consts::TAU;
                    w.spawn_enemy(EnemyKind::Oni, w.player.pos + Vec2::from_angle(a) * (60.0 + (n % 4) as f32 * 30.0));
                }
                w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
                w.enemies.hp.iter_mut().for_each(|h| *h = 1e9);
                let ti = ALL_WEAPONS.iter().position(|x| *x == k).unwrap();
                let (mut hits, mut signatures) = (0, 0);
                for _ in 0..60 * 10 {
                    if w.offer.is_some() {
                        w.bot_pick();
                    }
                    let timer = w.sig_timers[ti];
                    w.step(Input::default());
                    if w.sig_timers[ti] > timer {
                        signatures += 1;
                    }
                    for e in &w.events {
                        if let Event::Hit { elem, .. } = e {
                            assert_eq!(*elem, Some(k.element()), "{k:?} awakened={awakened}");
                            hits += 1;
                        }
                    }
                }
                assert!(hits > 0, "{k:?} awakened={awakened} never hit");
                if awakened && signature_every(k) > 0.0 {
                    assert!(signatures >= 2, "{k:?} signature fired {signatures} times");
                }
            }
        }
    }

    fn loadout_with_fire_damage(bonus: f32) -> gear::Loadout {
        let mut l = gear::Loadout::default();
        l.elem_damage[Element::Fire as usize] = bonus;
        l
    }

    #[test]
    fn gear_element_damage_adds_to_hits() {
        let lost = |gear: gear::Loadout| {
            let mut w = World::new(WorldConfig { seed: 5, gear, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
            w.spawn_enemy(EnemyKind::Kasa, w.player.pos + Vec2::new(60.0, 0.0));
            let before = w.enemies.hp[0];
            w.apply_hit(0, 10.0, Vec2::new(1.0, 0.0), Some(Element::Fire));
            before - w.enemies.hp[0]
        };
        let (base, boosted) = (lost(gear::Loadout::default()), lost(loadout_with_fire_damage(0.5)));
        assert!(base > 0.0);
        assert!((boosted - base * 1.5).abs() < 1e-3, "{boosted} vs {base}");
    }

    #[test]
    fn resistance_caps_and_matches_element() {
        let lost = |elem: Option<Element>| {
            let mut gear = gear::Loadout::default();
            gear.elem_resist[Element::Fire as usize] = 2.0;
            let mut w = World::new(WorldConfig { gear, ..Default::default() });
            let (hp, armor) = (w.player.hp, w.armor());
            w.hurt_player(100.0, elem);
            (hp - w.player.hp, armor)
        };
        let (fire, armor) = lost(Some(Element::Fire));
        assert!((fire - (100.0 * 0.4 - armor)).abs() < 1e-3, "{fire}");
        assert!((lost(Some(Element::Water)).0 - (100.0 - armor)).abs() < 1e-3);
        assert!((lost(None).0 - (100.0 - armor)).abs() < 1e-3);
    }

    #[test]
    fn shots_carry_their_shooter_element() {
        let mut w = World::new(WorldConfig { seed: 5, ..Default::default() });
        w.spawn_enemy(EnemyKind::Chochin, w.player.pos + Vec2::new(200.0, 0.0));
        for _ in 0..60 * 10 {
            w.move_enemies();
            if let Some(s) = w.shots.first() {
                assert_eq!(s.elem, Some(EnemyKind::Chochin.element()));
                return;
            }
        }
        panic!("no shot fired");
    }

    #[test]
    fn gear_hits_carry_an_element() {
        type Setup = Box<dyn Fn(&mut World)>;
        let mut cases: Vec<(String, Element, Setup)> = Vec::new();
        for a in gear::ALL_ARTS {
            cases.push((format!("{a:?}"), a.element(), Box::new(move |w: &mut World| w.art = Some(ArtState { art: a, power: 1.0, mythic: false, timer: 0.0, phase: 0.0, orbs: Vec::new(), foxes: Vec::new() }))));
        }
        cases.push(("Thunderclap".into(), Element::Electric, Box::new(|w| w.cfg.gear.uniques.push((Unique::Thunderclap, 1.0)))));
        cases.push(("Starburst".into(), Element::Light, Box::new(|w| w.cfg.gear.uniques.push((Unique::Starburst, 1.0)))));
        cases.push(("LanternHeart".into(), Exotic::LanternHeart.element(), Box::new(|w| w.cfg.gear.exotics.push(Exotic::LanternHeart))));
        for (name, want, setup) in cases {
            let mut w = World::new(WorldConfig { seed: 5, ..Default::default() });
            w.weapons.clear();
            w.player.hp = 1e9;
            for n in 0..12 {
                let a = n as f32 / 12.0 * std::f32::consts::TAU;
                w.spawn_enemy(EnemyKind::Oni, w.player.pos + Vec2::from_angle(a) * (60.0 + (n % 4) as f32 * 30.0));
            }
            w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
            w.enemies.hp.iter_mut().for_each(|h| *h = 1e9);
            setup(&mut w);
            let mut hits = 0;
            for _ in 0..60 * 12 {
                if w.offer.is_some() {
                    w.bot_pick();
                    w.weapons.clear();
                }
                w.step(Input::default());
                for e in &w.events {
                    if let Event::Hit { elem, .. } = e {
                        assert_eq!(*elem, Some(want), "{name}");
                        hits += 1;
                    }
                }
            }
            assert!(hits > 0, "{name} never hit");
        }
    }

    #[test]
    fn ultimate_hits_carry_hero_element() {
        for h in ALL_HEROES.into_iter().filter(|h| hero_def(*h).finished) {
            let mut w = World::new(WorldConfig { hero: h, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
            w.weapons.clear();
            for k in 0..30 {
                w.spawn_enemy(EnemyKind::Oni, w.player.pos + Vec2::from_angle(k as f32 * 0.21) * (45.0 + (k % 3) as f32 * 30.0));
            }
            w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
            w.enemies.hp.iter_mut().for_each(|hp| *hp = 1e9);
            w.player.special = 1.0;
            w.step(Input { special: true, ..Default::default() });
            let mut hits = 0;
            for _ in 0..60 * 9 {
                for e in &w.events {
                    if let Event::Hit { elem, .. } = e {
                        assert_eq!(*elem, Some(hero_def(h).element), "{h:?}");
                        hits += 1;
                    }
                }
                if w.offer.is_some() {
                    w.bot_pick();
                }
                w.step(Input::default());
            }
            assert!(hits > 0, "{h:?} special never hit");
        }
    }

    fn quiet_world() -> World {
        let mut w = World::new(WorldConfig { stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.weapons.clear();
        w.bonus.crit_chance = 0.0;
        w
    }

    fn add_enemy(w: &mut World, kind: EnemyKind, dx: f32) -> usize {
        w.spawn_enemy(kind, w.player.pos + Vec2::new(dx, 0.0));
        let j = w.enemies.len() - 1;
        w.enemies.hp[j] = 1000.0;
        w.enemies.speed[j] = 0.0;
        j
    }

    fn hit_count(w: &World) -> usize {
        w.events.iter().filter(|e| matches!(e, Event::Hit { .. })).count()
    }

    const RIGHT: Vec2 = Vec2::new(1.0, 0.0);

    #[test]
    fn burn_respects_reach() {
        let mut w = quiet_world();
        let near = add_enemy(&mut w, EnemyKind::Oni, 100.0);
        let far = add_enemy(&mut w, EnemyKind::Oni, 120.0);
        w.on_hit_effect(near, Element::Fire, 100.0, RIGHT);
        w.on_hit_effect(far, Element::Fire, 100.0, RIGHT);
        w.enemies.pos[far] = w.player.pos + Vec2::new(w.reach_half().x + 60.0, 0.0);
        let (near_hp, far_hp) = (w.enemies.hp[near], w.enemies.hp[far]);
        let mut hits = 0;
        for _ in 0..((BURN_SECONDS + 0.5) / DT) as usize {
            w.step(Input::default());
            hits += hit_count(&w);
        }
        assert!((near_hp - w.enemies.hp[near] - 30.0).abs() < 0.5, "burned {}", near_hp - w.enemies.hp[near]);
        assert!(hits <= 7, "{hits} burn ticks");
        assert_eq!(w.enemies.hp[far], far_hp);
    }

    #[test]
    fn soak_boosts_only_electric() {
        let mut w = quiet_world();
        let j = add_enemy(&mut w, EnemyKind::Kodama, 40.0);
        w.on_hit_effect(j, Element::Water, 5.0, RIGHT);
        let mut lost = |elem| {
            let before = w.enemies.hp[j];
            w.apply_hit(j, 5.0, RIGHT, Some(elem));
            before - w.enemies.hp[j]
        };
        assert!((lost(Element::Electric) - 5.0 * element_mult(Element::Electric, Element::Earth) * SOAK_ELECTRIC).abs() < 1e-3);
        assert!((lost(Element::Fire) - 5.0 * element_mult(Element::Fire, Element::Earth)).abs() < 1e-3);
    }

    #[test]
    fn chain_stays_in_reach_and_stops() {
        let mut w = quiet_world();
        let first = add_enemy(&mut w, EnemyKind::Oni, 170.0);
        let outside = add_enemy(&mut w, EnemyKind::Oni, 225.0);
        let second = add_enemy(&mut w, EnemyKind::Oni, 110.0);
        assert!(!w.in_reach(w.enemies.pos[outside], w.enemies.radius[outside]));
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        let before: Vec<f32> = w.enemies.hp.clone();
        w.on_hit_effect(first, Element::Electric, 100.0, RIGHT);
        assert_eq!(w.enemies.hp[first], before[first]);
        assert_eq!(w.enemies.hp[outside], before[outside]);
        let expected = 100.0 * CHAIN_SHARE * element_mult(Element::Electric, Element::Fire);
        assert!((before[second] - w.enemies.hp[second] - expected).abs() < 1e-3);
        assert_eq!(hit_count(&w), 1);
        assert!(w.events.iter().any(|e| matches!(e, Event::Bolt { from, to } if *from == w.enemies.pos[first] && *to == w.enemies.pos[second])));
    }

    fn six_world(stage: StageId) -> World {
        let gear = Loadout { sets6: vec![stage], ..Default::default() };
        let mut w = World::new(WorldConfig { gear, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.weapons.clear();
        w.bonus.crit_chance = 0.0;
        w
    }

    fn kill_step(w: &mut World, j: usize) {
        w.enemies.hp[j] = 0.0;
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        w.remove_dead_enemies();
    }

    fn drop_shares(kind: EnemyKind, kills: usize) -> (f32, f32, usize) {
        let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, ..Default::default() });
        w.weapons.clear();
        let (mut gear, mut pickups) = (0, 0);
        for _ in 0..kills {
            w.events.clear();
            w.pickups.clear();
            w.spawn_enemy(kind, w.player.pos + Vec2::new(300.0, 0.0));
            let j = w.enemies.len() - 1;
            kill_step(&mut w, j);
            gear += w.events.iter().filter(|e| matches!(e, Event::GearDrop { .. })).count();
            pickups += w.pickups.len();
        }
        (gear as f32 / kills as f32, pickups as f32 / kills as f32, w.chests.len())
    }

    #[test]
    fn uncommon_and_leader_drops() {
        let (gear, _, chests) = drop_shares(EnemyKind::ToolThief, 200_000);
        assert!((gear - UNCOMMON_GEAR).abs() <= UNCOMMON_GEAR * 0.4, "uncommon gear share {gear}");
        assert_eq!(chests, 0);
        let (gear, _, _) = drop_shares(EnemyKind::Wisp, 2000);
        assert!(gear < 0.003, "common gear share {gear}");
        let (_, pickups, chests) = drop_shares(EnemyKind::CaptainKitsunebi, 2000);
        assert!((0.4..=0.6).contains(&pickups), "leader pickup share {pickups}");
        assert_eq!(chests, 2000);
    }

    fn set_drop_world() -> World {
        let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, ..Default::default() });
        w.bonus.gear_find = 1000.0;
        w.weapons.clear();
        w
    }

    #[test]
    fn bosses_drop_this_worlds_set() {
        for kind in [EnemyKind::GreatOni, EnemyKind::ToriiSerpent, EnemyKind::NightLord] {
            let mut w = set_drop_world();
            let j = add_enemy(&mut w, kind, 100.0);
            kill_step(&mut w, j);
            assert!(w.loot.iter().any(|i| i.rarity == Rarity::Set && i.set == Some(StageId::ShrineValley)), "{kind:?} dropped no set piece");
            assert!(w.events.iter().any(|e| matches!(e, Event::GearDrop { rarity: Rarity::Set, .. })));
        }
        for kind in [EnemyKind::Wisp, EnemyKind::Oni, EnemyKind::Komainu] {
            let mut w = set_drop_world();
            for _ in 0..50 {
                let j = add_enemy(&mut w, kind, 100.0);
                kill_step(&mut w, j);
            }
            assert!(!w.loot.iter().any(|i| i.rarity == Rarity::Set), "{kind:?} dropped a set piece");
        }
    }

    #[test]
    fn lantern_six_passes_burn_on_death() {
        let mut w = six_world(StageId::Shotengai);
        let dying = add_enemy(&mut w, EnemyKind::Oni, 100.0);
        add_enemy(&mut w, EnemyKind::Oni, 150.0);
        let out_x = w.reach_half().x + 300.0;
        add_enemy(&mut w, EnemyKind::Oni, out_x);
        w.enemies.burn[dying] = 2.0;
        w.enemies.burn_dps[dying] = 7.0;
        kill_step(&mut w, dying);
        assert_eq!(w.enemies.len(), 2);
        let (n, f) = (w.enemies.pos.iter().position(|p| p.x == 150.0).unwrap(), w.enemies.pos.iter().position(|p| p.x > 400.0).unwrap());
        assert_eq!((w.enemies.burn[n], w.enemies.burn_dps[n]), (2.0, 7.0));
        assert_eq!(w.enemies.burn[f], 0.0);
    }

    #[test]
    fn moonveil_six_boosts_staggered() {
        let mut w = six_world(StageId::ShrineValley);
        let j = add_enemy(&mut w, EnemyKind::Oni, 100.0);
        let lost = |w: &mut World| {
            let before = w.enemies.hp[j];
            w.apply_hit(j, 10.0, RIGHT, None);
            before - w.enemies.hp[j]
        };
        let plain = lost(&mut w);
        w.enemies.stagger[j] = 0.4;
        assert!((lost(&mut w) - plain * SET6_STAGGER).abs() < 1e-3);
    }

    #[test]
    fn ember_six_doubles_wind_procs() {
        let mut w = six_world(StageId::EmberLeaf);
        let j = add_enemy(&mut w, EnemyKind::Wisp, 100.0);
        w.enemies.hp[j] = 1e9;
        let mut procs = 0;
        for _ in 0..10_000 {
            w.enemies.push[j] = Vec2::ZERO;
            w.hurt_enemy(j, 1.0, RIGHT, Some(Element::Wind));
            if w.enemies.push[j].length() > 1.0 + 150.0 * w.enemies.knock[j] {
                procs += 1;
            }
        }
        let rate = procs as f32 / 10_000.0;
        assert!((0.25..=0.35).contains(&rate), "proc rate {rate}");
    }

    #[test]
    fn pale_six_shatters_slowed() {
        let mut w = six_world(StageId::PaleCitadel);
        let dying = add_enemy(&mut w, EnemyKind::Oni, 100.0);
        let close = add_enemy(&mut w, EnemyKind::Oni, 130.0);
        let outside = add_enemy(&mut w, EnemyKind::Oni, 100.0 + SET6_SHATTER_R + 30.0);
        w.enemies.slow[dying] = 1.0;
        w.enemies.max_hp[dying] = 100.0;
        let (close_x, outside_x) = (w.enemies.pos[close].x, w.enemies.pos[outside].x);
        kill_step(&mut w, dying);
        let at = |w: &World, x: f32| w.enemies.pos.iter().position(|p| p.x == x).unwrap();
        let expected = 50.0 * element_mult(Element::Ice, EnemyKind::Oni.element());
        assert!((1000.0 - w.enemies.hp[at(&w, close_x)] - expected).abs() < 1e-3);
        assert_eq!(w.enemies.hp[at(&w, outside_x)], 1000.0);
    }

    #[test]
    fn rainward_six_always_soaks() {
        let mut w = six_world(StageId::Rainward);
        let j = add_enemy(&mut w, EnemyKind::Oni, 100.0);
        w.enemies.hp[j] = 1e9;
        for _ in 0..200 {
            w.enemies.soak[j] = 0.0;
            w.hurt_enemy(j, 1.0, RIGHT, Some(Element::Water));
            assert!(w.enemies.soak[j] > 0.0);
        }
        w.enemies.soak[j] = 0.0;
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        w.view_nova(w.player.pos, 400.0, 1.0, 0, Some(Element::Water));
        assert!(w.enemies.soak[j] > 0.0, "view_nova Water hit soaks");
    }

    #[test]
    fn shatterline_six_chains_twice() {
        let mut w = six_world(StageId::Shatterline);
        let first = add_enemy(&mut w, EnemyKind::Oni, 100.0);
        let second = add_enemy(&mut w, EnemyKind::Oni, 150.0);
        let third = add_enemy(&mut w, EnemyKind::Oni, 190.0);
        let out_x = w.reach_half().x + 300.0;
        let outside = add_enemy(&mut w, EnemyKind::Oni, out_x);
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        w.on_hit_effect(first, Element::Electric, 100.0, RIGHT);
        assert_eq!(w.enemies.hp[first], 1000.0);
        assert_eq!(w.enemies.hp[outside], 1000.0);
        assert!(w.enemies.hp[second] < 1000.0);
        assert!(w.enemies.hp[third] < 1000.0);
        assert_eq!(hit_count(&w), 2);
    }

    #[test]
    fn stagger_skips_bosses() {
        let mut w = quiet_world();
        let wisp = add_enemy(&mut w, EnemyKind::Wisp, 150.0);
        let oni = add_enemy(&mut w, EnemyKind::GreatOni, -150.0);
        w.enemies.speed[wisp] = 60.0;
        w.enemies.speed[oni] = 60.0;
        w.on_hit_effect(wisp, Element::Earth, 5.0, RIGHT);
        w.on_hit_effect(oni, Element::Earth, 5.0, RIGHT);
        let (wisp_at, oni_at) = (w.enemies.pos[wisp], w.enemies.pos[oni]);
        w.step(Input::default());
        assert!(w.enemies.pos[oni].x > oni_at.x);
        for _ in 0..22 {
            w.step(Input::default());
            assert_eq!(w.enemies.vel[wisp], Vec2::ZERO);
        }
        assert_eq!(w.enemies.pos[wisp], wisp_at);
        for _ in 0..6 {
            w.step(Input::default());
        }
        assert!(w.enemies.pos[wisp].x < wisp_at.x - 1.0);
    }

    #[test]
    fn slow_and_knockback() {
        let mut w = quiet_world();
        let j = add_enemy(&mut w, EnemyKind::Wisp, 150.0);
        w.enemies.speed[j] = 60.0;
        w.on_hit_effect(j, Element::Ice, 5.0, RIGHT);
        let step_dx = |w: &mut World| {
            let x = w.enemies.pos[j].x;
            w.step(Input::default());
            x - w.enemies.pos[j].x
        };
        let slowed = 60.0 * SLOW_FACTOR * DT;
        assert!((step_dx(&mut w) - slowed).abs() < 1e-3);
        for _ in 0..100 {
            step_dx(&mut w);
        }
        assert!((step_dx(&mut w) - slowed).abs() < 1e-3);
        for _ in 0..30 {
            step_dx(&mut w);
        }
        assert!((step_dx(&mut w) - 60.0 * DT).abs() < 1e-3);
        let k = add_enemy(&mut w, EnemyKind::Wisp, -80.0);
        w.on_hit_effect(k, Element::Wind, 5.0, RIGHT * 3.0);
        assert_eq!(w.enemies.push[k], RIGHT * (KNOCK_EXTRA * w.enemies.knock[k]));
        assert!(w.enemies.push[k].x > 0.0);
    }

    #[test]
    fn mark_makes_next_hit_crit() {
        let mut w = quiet_world();
        let j = add_enemy(&mut w, EnemyKind::Oni, 100.0);
        w.on_hit_effect(j, Element::Light, 5.0, RIGHT);
        let crit = |w: &mut World| {
            w.events.clear();
            w.hurt_enemy(j, 5.0, RIGHT, None);
            w.events.iter().any(|e| matches!(e, Event::Hit { crit: true, .. }))
        };
        assert!(crit(&mut w));
        assert!(!crit(&mut w));
    }

    #[test]
    fn drain_caps_at_max_hp() {
        let mut w = quiet_world();
        let j = add_enemy(&mut w, EnemyKind::Oni, 100.0);
        w.player.hp = w.player.max_hp - 1.0;
        w.on_hit_effect(j, Element::Darkness, 5.0, RIGHT);
        assert_eq!(w.player.hp, w.player.max_hp - 1.0 + DRAIN_HEAL);
        w.player.hp = w.player.max_hp - 0.2;
        w.on_hit_effect(j, Element::Darkness, 5.0, RIGHT);
        assert_eq!(w.player.hp, w.player.max_hp);
        w.cfg.gear.exotics.push(Exotic::ClosedHandVow);
        w.player.hp = w.player.max_hp - 1.0;
        w.on_hit_effect(j, Element::Darkness, 5.0, RIGHT);
        assert_eq!(w.player.hp, w.player.max_hp - 1.0);
    }

    #[test]
    fn statuses_follow_swap_remove() {
        let mut w = quiet_world();
        add_enemy(&mut w, EnemyKind::Oni, 100.0);
        add_enemy(&mut w, EnemyKind::Oni, 130.0);
        for elem in [Element::Fire, Element::Water, Element::Earth, Element::Ice, Element::Light] {
            w.on_hit_effect(0, elem, 100.0, RIGHT);
        }
        w.enemies.hp[0] = 0.0;
        w.step(Input::default());
        let e = &w.enemies;
        assert_eq!(e.len(), 1);
        assert_eq!((e.burn[0], e.burn_dps[0], e.soak[0], e.stagger[0], e.slow[0], e.marked[0]), (0.0, 0.0, 0.0, 0.0, 0.0, false));
    }

    #[test]
    fn banner_columns_follow_swap_remove() {
        let mut w = quiet_world();
        add_enemy(&mut w, EnemyKind::Oni, 100.0);
        let k = add_enemy(&mut w, EnemyKind::Oni, 130.0);
        w.enemies.banner[k] = true;
        w.enemies.scatter[k] = 1.5;
        w.enemies.scatter_from[k] = Vec2::new(7.0, 8.0);
        w.enemies.hp[0] = 0.0;
        w.remove_dead_enemies();
        let e = &w.enemies;
        assert_eq!(e.len(), 1);
        assert_eq!((e.banner[0], e.scatter[0], e.scatter_from[0]), (true, 1.5, Vec2::new(7.0, 8.0)));
    }

    fn leader_world() -> (World, usize) {
        let mut w = quiet_world();
        let leader = w.stage().leader;
        let l = add_enemy(&mut w, leader, 300.0);
        (w, l)
    }

    #[test]
    fn banner_buffs_only_nearby() {
        let (mut w, l) = leader_world();
        let near = add_enemy(&mut w, EnemyKind::Wisp, 400.0);
        let far = add_enemy(&mut w, EnemyKind::Wisp, 500.0);
        w.enemies.speed[near] = 50.0;
        w.enemies.speed[far] = 50.0;
        w.step(Input::default());
        let e = &w.enemies;
        assert!(e.banner[near] && !e.banner[far] && !e.banner[l]);
        let ratio = e.vel[near].length() / e.vel[far].length();
        assert!((ratio - BANNER_SPEED).abs() < 0.01, "ratio {ratio}");
    }

    #[test]
    fn banner_hits_harder() {
        let hit = |banner_near: bool| {
            let mut w = World::new(WorldConfig::default());
            w.weapons.clear();
            w.enemies = Enemies::default();
            let p = w.player.pos;
            w.spawn_enemy(EnemyKind::Wisp, p);
            w.enemies.damage[0] = 20.0;
            if banner_near {
                let leader = w.stage().leader;
                w.spawn_enemy(leader, p + Vec2::new(60.0, 0.0));
                w.enemies.damage[1] = 0.0;
            }
            w.update_banners();
            w.grid.rebuild(w.player.pos, &w.enemies.pos);
            let before = w.player.hp;
            w.contact_damage();
            (before - w.player.hp, w.armor())
        };
        let ((plain, armor), (buffed, _)) = (hit(false), hit(true));
        assert_eq!(plain, 20.0 - armor);
        assert_eq!(buffed, 20.0 * BANNER_DAMAGE - armor);
    }

    #[test]
    fn banner_ends_when_leader_dies() {
        let (mut w, l) = leader_world();
        let near = add_enemy(&mut w, EnemyKind::Wisp, 400.0);
        w.enemies.speed[near] = 50.0;
        w.step(Input::default());
        assert!(w.enemies.banner[near]);
        let death = w.enemies.pos[l];
        kill_step(&mut w, l);
        assert!(w.events.iter().any(|e| matches!(*e, Event::LeaderFell { pos } if pos == death)));
        assert!(w.enemies.banner.iter().all(|b| !b));
        assert_eq!(w.enemies.len(), 1);
        assert_eq!(w.enemies.scatter[0], SCATTER_TIME);
        assert_eq!(w.enemies.scatter_from[0], death);
        let before = (w.enemies.pos[0] - death).length();
        w.step(Input::default());
        assert!((w.enemies.pos[0] - death).length() > before);
        for _ in 0..((SCATTER_TIME + 0.1) / DT) as usize {
            w.step(Input::default());
        }
        let to_player = (w.player.pos - w.enemies.pos[0]).normalize_or_zero();
        w.step(Input::default());
        let v = w.enemies.vel[0].normalize_or_zero();
        assert!(v.x * to_player.x + v.y * to_player.y > 0.9);
    }

    #[test]
    fn earth_cannot_stun_lock_a_leader() {
        let mut w = quiet_world();
        let leader = w.stage().leader;
        let l = add_enemy(&mut w, leader, 150.0);
        let wisp = add_enemy(&mut w, EnemyKind::Wisp, -150.0);
        let mut free_ticks = 0;
        for _ in 0..(3.0 / DT) as usize {
            w.on_hit_effect(l, Element::Earth, 5.0, RIGHT);
            w.on_hit_effect(wisp, Element::Earth, 5.0, RIGHT);
            w.tick_statuses();
            assert!(w.enemies.stagger[wisp] > 0.0, "common enemies stay staggerable");
            if w.enemies.stagger[l] <= 0.0 {
                free_ticks += 1;
            }
        }
        let cycle = STAGGER_SECONDS + LEADER_STAGGER_IMMUNE;
        let expected = (3.0 / cycle) * LEADER_STAGGER_IMMUNE / DT;
        assert!(free_ticks as f32 > expected * 0.8, "Leader free for only {free_ticks} ticks");
    }

    #[test]
    fn frozen_leader_holds_still() {
        for hold in ["none", "freeze", "stagger", "firmament"] {
            let mut w = World::new(WorldConfig { hero: HeroId::Kuon, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
            w.weapons.clear();
            let leader = w.stage().leader;
            let l = add_enemy(&mut w, leader, 300.0);
            w.enemies.speed[l] = 34.0;
            w.enemies.ai_timer[l] = 0.0;
            match hold {
                "freeze" => w.freeze = 1.0,
                "stagger" => w.enemies.stagger[l] = 1.0,
                "firmament" => {
                    w.player.form = DOMAIN_DURATION;
                    w.domain_center = w.enemies.pos[l];
                }
                _ => {}
            }
            let at = w.enemies.pos[l];
            for _ in 0..30 {
                w.step(Input::default());
            }
            if hold == "none" {
                assert_ne!(w.enemies.pos[l], at, "a free Leader moves");
                assert!(!w.shots.is_empty() || !w.hazards.is_empty(), "a free Leader fires");
            } else {
                assert_eq!(w.enemies.pos[l], at, "{hold}");
                assert!(w.shots.is_empty() && w.hazards.is_empty(), "{hold}");
            }
        }
    }

    #[test]
    fn proc_rate_is_about_fifteen_percent() {
        let mut w = quiet_world();
        let j = add_enemy(&mut w, EnemyKind::Oni, 100.0);
        w.enemies.hp[j] = 1e9;
        let mut procs = 0;
        for _ in 0..4000 {
            w.hurt_enemy(j, 1.0, RIGHT, Some(Element::Earth));
            if w.enemies.stagger[j] > 0.0 {
                procs += 1;
                w.enemies.stagger[j] = 0.0;
            }
        }
        let rate = procs as f32 / 4000.0;
        assert!((0.12..=0.18).contains(&rate), "proc rate {rate}");
    }

    #[test]
    fn melee_heroes_reach_less() {
        let reach = |hero| World::new(WorldConfig { hero, ..Default::default() });
        let (ren, shun) = (reach(HeroId::Ren), reach(HeroId::Shun));
        assert_eq!(ren.reach_half(), REACH_HALF);
        assert_eq!(shun.reach_half(), REACH_HALF * 0.8);
        let at = Vec2::new(REACH_HALF.x * 0.9, 0.0);
        assert!(ren.in_reach(at, 0.0));
        assert!(!shun.in_reach(at, 0.0));
    }

    #[test]
    fn small_screens_shrink_reach() {
        let mut w = World::new(WorldConfig::default());
        w.view_half = Vec2::new(200.0, 120.0);
        assert_eq!(w.reach_half(), Vec2::new(176.0, 96.0));
    }

    #[test]
    fn edge_overlap_counts() {
        let w = World::new(WorldConfig::default());
        let r = 10.0;
        let edge = w.player.pos + Vec2::new(REACH_HALF.x + r - 1.0, 0.0);
        assert!(w.in_reach(edge, r));
        assert!(!w.in_reach(edge + Vec2::new(2.0, 0.0), r));
    }

    fn seal_world() -> World {
        let mut w = World::new(WorldConfig::default());
        let p = w.player.pos;
        w.spawn_enemy(EnemyKind::Wisp, p + Vec2::new(250.0, 0.0));
        w.spawn_enemy(EnemyKind::Wisp, p + Vec2::new(500.0, 0.0));
        w
    }

    #[test]
    fn exorcism_hits_the_visible_screen_only() {
        let mut w = seal_world();
        let (near, far) = (w.enemies.hp[0], w.enemies.hp[1]);
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        w.pickups.push((w.player.pos, PickupKind::ExorcismSeal));
        w.update_pickups();
        assert!(w.enemies.hp[0] < near, "outside reach but on screen should die");
        assert_eq!(w.enemies.hp[1], far, "off screen must be spared");
    }

    #[test]
    fn thousand_counter_nova_hits_the_visible_screen_only() {
        let mut w = seal_world();
        let (near, far) = (w.enemies.hp[0], w.enemies.hp[1]);
        w.pending_exo_blasts.push((w.player.pos, 450.0, 400.0));
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        w.update_exotics();
        assert!(w.enemies.hp[0] < near);
        assert_eq!(w.enemies.hp[1], far);
    }

    #[test]
    fn in_view_needs_centre_on_screen() {
        let mut w = quiet_world();
        let p = w.player.pos;
        let edge = w.view_half.x;
        w.spawn_enemy(EnemyKind::Oni, p + Vec2::new(edge + 5.0, 0.0));
        w.spawn_enemy(EnemyKind::Oni, p + Vec2::new(edge - 5.0, 0.0));
        assert!(w.enemies.radius[0] > 5.0, "the off-screen body must overlap the edge");
        let hp = (w.enemies.hp[0], w.enemies.hp[1]);
        w.grid.rebuild(p, &w.enemies.pos);
        w.view_nova(p, 450.0, 10.0, 0, None);
        assert_eq!(w.enemies.hp[0], hp.0, "centre off screen must be spared");
        assert!(w.enemies.hp[1] < hp.1, "centre on screen is hit");
    }

    #[test]
    fn moonfall_lands_in_reach() {
        for art in [ArsenalArt::Moonfall, ArsenalArt::SkeletonHand] {
            let mut w = quiet_world();
            let p = w.player.pos;
            for k in 0..24 {
                let a = k as f32 / 24.0 * std::f32::consts::TAU;
                let r = if k % 2 == 0 { 80.0 } else { 420.0 + 10.0 * k as f32 };
                w.spawn_enemy(EnemyKind::Wisp, p + Vec2::from_angle(a) * r);
            }
            w.art = Some(ArtState { art, power: 1.0, mythic: true, timer: 0.0, phase: 0.0, orbs: Vec::new(), foxes: Vec::new() });
            w.grid.rebuild(p, &w.enemies.pos);
            for _ in 0..40 {
                w.art.as_mut().unwrap().timer = 0.0;
                w.update_art();
            }
            assert!(w.blasts.len() >= 40, "{art:?} cast {} blasts", w.blasts.len());
            for b in &w.blasts {
                assert!(w.in_reach(b.pos, MAX_ENEMY_RADIUS), "{art:?} landed at {:?}", b.pos - p);
            }
        }
    }

    #[test]
    fn off_view_blasts_are_silent() {
        let mut w = quiet_world();
        let p = w.player.pos;
        let off = p + Vec2::new(w.view_half.x + 60.0, 0.0);
        let bursts = |w: &World| w.events.iter().filter(|e| matches!(e, Event::Burst { .. })).count();
        let blast = |pos| Blast { pos, delay: 0.0, max_delay: 0.0, radius: 40.0, damage: 10.0, style: style::METEOR, burns: false, elem: None, follow: None };
        w.blasts.push(blast(off));
        w.update_signatures();
        assert_eq!(bursts(&w), 0, "off-view delayed blast");
        w.projectiles.push_kind(ProjKind::Seal, 6.0, off, Vec2::ZERO, 1.0, 10.0, 1, 0.0, None);
        let last = w.projectiles.len() - 1;
        w.projectiles.burst[last] = 28.0;
        w.burst_projectile(last);
        assert_eq!(bursts(&w), 0, "off-view seal burst");
        w.blasts.push(blast(p));
        w.update_signatures();
        assert_eq!(bursts(&w), 1, "an on-screen blast still bursts");
    }

    fn assert_visuals_in_reach(hero: HeroId, awakened: bool) {
        use WeaponKind::*;
        let mut w = World::new(WorldConfig { seed: 9, hero, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.time = AWAKEN_FROM + 1.0;
        w.bonus.area = 2.0;
        w.weapons = [Taiko, MoonOrbs, SakuraField, YinYangOrbs, PurificationWave, IaiFlash, CrimsonTendrils]
            .map(|kind| OwnedWeapon { kind, level: MAX_LEVEL, awakened, timer: 0.0, phase: 0.0, extra: false, power: 1.0 })
            .to_vec();
        w.art = Some(ArtState { art: ArsenalArt::ShadowStar, power: 1.0, mythic: false, timer: 0.0, phase: 0.0, orbs: Vec::new(), foxes: Vec::new() });
        for k in 0..48 {
            let a = k as f32 / 48.0 * std::f32::consts::TAU;
            w.spawn_enemy(EnemyKind::Oni, w.player.pos + Vec2::from_angle(a) * (60.0 + (k % 6) as f32 * 50.0));
        }
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        w.enemies.hp.iter_mut().for_each(|h| *h = 1e9);
        w.player.special = 1.0;
        let cap = w.reach_half().x + 0.01;
        let label = format!("{hero:?} awakened={awakened}");
        let (mut waves, mut lines) = (0, 0);
        for t in 0..60 * 8 {
            if w.offer.is_some() {
                w.bot_pick();
            }
            w.step(Input { special: t == 0, ..Default::default() });
            let p = w.player.pos;
            for wv in &w.waves {
                waves += 1;
                assert!(wv.max_radius <= cap, "{label}: wave to {}", wv.max_radius);
            }
            for o in w.weapons.clone() {
                let orbs = match o.kind {
                    MoonOrbs => w.orb_positions(&o),
                    YinYangOrbs => w.yinyang_positions(&o),
                    _ => Vec::new(),
                };
                for c in orbs {
                    assert!((c - p).length() <= cap + 6.0, "{label}: {:?} orbit {}", o.kind, (c - p).length());
                }
                if o.kind == SakuraField {
                    assert!(w.field_radius_of(&o) <= cap, "{label}: field {}", w.field_radius_of(&o));
                }
            }
            for e in &w.events {
                if let Event::Beam { from, to, .. } | Event::Iai { from, to } | Event::ShadowStep { from, to } | Event::Lash { from, to } = e {
                    lines += 1;
                    assert!(w.in_reach(*from, 0.01) && w.in_reach(*to, 0.01), "{label}: {e:?} leaves reach");
                }
            }
            for s in &w.scars {
                assert!(w.in_reach(s.to, 0.01), "{label}: scar to {:?}", s.to - p);
            }
        }
        assert!(waves > 0 && lines > 0, "{label}: waves {waves}, lines {lines}");
    }

    #[test]
    fn visuals_stay_inside_reach() {
        assert_visuals_in_reach(HeroId::Ren, false);
        assert_visuals_in_reach(HeroId::Ren, true);
        assert_visuals_in_reach(HeroId::Shun, true);
    }

    #[test]
    fn gear_bolts_pick_only_enemies_in_reach() {
        let mut w = World::new(WorldConfig::default());
        let p = w.player.pos;
        w.cfg.gear.uniques.push((Unique::Thunderclap, 1.0));
        w.spawn_enemy(EnemyKind::Wisp, p + Vec2::new(0.0, 150.0));
        w.spawn_enemy(EnemyKind::Wisp, p + Vec2::new(180.0, 0.0));
        let (out, inside) = (w.enemies.hp[0], w.enemies.hp[1]);
        w.gear_timers[0] = 0.0;
        w.grid.rebuild(p, &w.enemies.pos);
        w.update_gear();
        let bolts: Vec<Vec2> = w.events.iter().filter_map(|e| if let Event::Bolt { to, .. } = e { Some(*to) } else { None }).collect();
        assert_eq!(bolts, vec![w.enemies.pos[1]], "only the in-reach enemy is bolted");
        assert_eq!(w.enemies.hp[0], out);
        assert!(w.enemies.hp[1] < inside);
    }

    fn reach_pair() -> World {
        let mut w = World::new(WorldConfig::default());
        let p = w.player.pos;
        w.spawn_enemy(EnemyKind::Wisp, p + Vec2::new(0.0, 150.0));
        w.spawn_enemy(EnemyKind::Wisp, p + Vec2::new(180.0, 0.0));
        w
    }

    fn assert_only_second_hurt(w: &World, hp: (f32, f32)) {
        assert_eq!(w.enemies.hp[0], hp.0, "out-of-reach enemy untouched");
        assert!(w.enemies.hp[1] < hp.1, "in-reach enemy hurt");
    }

    #[test]
    fn raijin_drums_strike_only_in_reach() {
        let mut w = reach_pair();
        w.art = Some(ArtState { art: ArsenalArt::RaijinDrums, power: 1.0, mythic: false, timer: 0.0, phase: 0.0, orbs: Vec::new(), foxes: Vec::new() });
        let hp = (w.enemies.hp[0], w.enemies.hp[1]);
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        w.update_art();
        assert_only_second_hurt(&w, hp);
    }

    #[test]
    fn yata_mirror_reflects_only_in_reach() {
        let mut w = reach_pair();
        w.cfg.gear.exotics.push(Exotic::YataMirror);
        let hp = (w.enemies.hp[0], w.enemies.hp[1]);
        w.reflect_shot();
        assert_only_second_hurt(&w, hp);
    }

    #[test]
    fn ember_leaf_set_blasts_only_in_reach() {
        let mut w = reach_pair();
        w.cfg.gear.sets4.push(StageId::EmberLeaf);
        w.gear_timers[2] = 0.0;
        w.update_gear();
        assert_eq!(w.hazards.len(), 1);
        assert_eq!(w.hazards[0].pos, w.enemies.pos[1]);
    }

    #[test]
    fn rainward_set_lightning_only_in_reach() {
        let mut w = reach_pair();
        w.cfg.gear.sets4.push(StageId::Rainward);
        w.gear_timers[3] = 0.0;
        w.update_gear();
        assert_eq!(w.hazards.len(), 1);
        assert_eq!(w.hazards[0].pos, w.enemies.pos[1]);
    }

    #[test]
    fn fallback_targets_spread_inside_reach() {
        let mut w = World::new(WorldConfig::default());
        let h = w.reach_half();
        let targets = w.random_targets(340.0, 200);
        let near_edge = targets.iter().filter(|t| {
            let d = **t - w.player.pos;
            assert!(d.x.abs() <= h.x && d.y.abs() <= h.y);
            d.x.abs() > h.x - 4.0 || d.y.abs() > h.y - 4.0
        }).count();
        assert!(near_edge < 20, "{near_edge} of 200 piled on the reach edge");
    }

    fn fly_projectiles(w: &mut World, frames: usize) {
        for _ in 0..frames {
            w.grid.rebuild(w.player.pos, &w.enemies.pos);
            w.move_projectiles_and_hit();
        }
    }

    #[test]
    fn projectiles_pass_out_of_reach_enemies() {
        for burst in [0.0, 20.0] {
            let mut w = World::new(WorldConfig::default());
            w.spawn_enemy(EnemyKind::Wisp, Vec2::new(REACH_HALF.x + 60.0, 0.0));
            w.spawn_enemy(EnemyKind::Wisp, Vec2::new(100.0, 0.0));
            w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
            let (far_hp, near_hp) = (w.enemies.hp[0], w.enemies.hp[1]);
            w.projectiles.push(Vec2::new(REACH_HALF.x + 110.0, 0.0), Vec2::new(-300.0, 0.0), 2.0, 5.0, 2, 0.0);
            w.projectiles.burst[0] = burst;
            fly_projectiles(&mut w, 25);
            assert_eq!(w.projectiles.len(), 1, "burst {burst}: the projectile stopped at an out-of-reach enemy");
            assert_eq!(w.projectiles.pierce[0], 2, "burst {burst}: pierce spent out of reach");
            fly_projectiles(&mut w, 25);
            assert_eq!(w.enemies.hp[0], far_hp);
            assert!(w.enemies.hp[1] < near_hp, "burst {burst}: the in-reach enemy behind was not hit");
        }
    }

    #[test]
    fn homing_ignores_out_of_reach() {
        let mut w = World::new(WorldConfig::default());
        w.spawn_enemy(EnemyKind::Wisp, Vec2::new(230.0, 0.0));
        w.spawn_enemy(EnemyKind::Wisp, Vec2::new(60.0, 0.0));
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        w.projectiles.push(Vec2::new(150.0, 0.0), Vec2::new(0.0, -200.0), 2.0, 5.0, 1, 10.0);
        fly_projectiles(&mut w, 1);
        assert!(w.projectiles.vel[0].x < 0.0, "steered toward the out-of-reach enemy: {:?}", w.projectiles.vel[0]);
    }

    #[test]
    fn random_target_fallback_stays_in_reach() {
        let mut w = World::new(WorldConfig::default());
        let h = w.reach_half();
        for t in w.random_targets(340.0, 200) {
            let d = t - w.player.pos;
            assert!(d.x.abs() <= h.x && d.y.abs() <= h.y, "fallback point {d:?} outside reach {h:?}");
        }
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
                w.bot_pick();
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
                w.bot_pick();
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
        let mut w = play_as(HeroId::Kaede, 9, 60 * 20);
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
    fn great_oni_spawns_at_round_16_and_drops_a_big_chest() {
        let mut w = World::new(WorldConfig::default());
        w.time = w.stage().round_start(16) - 0.5;
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
        w.weapons = vec![OwnedWeapon { kind, level: 4, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }];
        for k in 0..40 {
            let a = k as f32 / 40.0 * std::f32::consts::TAU;
            let r = 30.0 + (k % 4) as f32 * 25.0;
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(a) * r);
        }
        let n = w.enemies.len();
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        w.enemies.hp.iter_mut().for_each(|h| *h = 1e6);
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
        // With every slot filled, a new weapon may only come as a replacement card.
        w.weapons = pool.iter().take(ren.weapon_slots).map(|&k| OwnedWeapon { kind: k, level: 2, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }).collect();
        for _ in 0..300 {
            w.make_offer();
            for u in w.offer.take().unwrap() {
                if let Upgrade::Weapon(k) = u {
                    assert!(w.weapon_level(k) > 0 || w.is_replacement(u), "offered a 5th weapon {k:?}");
                }
            }
        }
    }

    fn weapon(kind: WeaponKind, level: u8) -> OwnedWeapon {
        OwnedWeapon { kind, level, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }
    }

    fn full_loadout(w: &mut World, level: u8) {
        let ren = hero_def(HeroId::Ren);
        w.weapons = ren.weapon_pool().into_iter().take(ren.weapon_slots).map(|k| weapon(k, level)).collect();
        w.passives = ALL_PASSIVES.iter().take(ren.passive_slots).map(|k| (*k, level)).collect();
    }

    fn with_replacement(w: &mut World, u: Upgrade) {
        w.offer = Some(vec![u, Upgrade::Heal]);
        w.pending_levels = 1;
        w.choose(0);
    }

    fn unowned_weapon(w: &World) -> WeaponKind {
        hero_def(w.cfg.hero).weapon_pool().into_iter().find(|k| w.weapon_level(*k) == 0).unwrap()
    }

    #[test]
    fn rare_skills_are_offered_less() {
        let mut w = World::new(WorldConfig::default());
        w.weapons.truncate(1);
        let mut counts: Vec<(WeaponKind, u32)> = Vec::new();
        for _ in 0..4000 {
            w.make_offer();
            for u in w.offer.take().unwrap() {
                if let Upgrade::Weapon(k) = u {
                    if w.weapon_level(k) == 0 {
                        match counts.iter_mut().find(|(c, _)| *c == k) {
                            Some((_, n)) => *n += 1,
                            None => counts.push((k, 1)),
                        }
                    }
                }
            }
        }
        let mean = |keep: fn(Tier) -> bool| {
            let v: Vec<u32> = counts.iter().filter(|(k, _)| keep(Upgrade::Weapon(*k).tier())).map(|(_, n)| *n).collect();
            assert!(!v.is_empty(), "no weapons in that tier band");
            v.iter().sum::<u32>() as f32 / v.len() as f32
        };
        let (common, high) = (mean(|t| t == Tier::Common), mean(|t| t >= Tier::Legendary));
        assert!(common > high * 1.8, "common {common} vs high {high}");
    }

    #[test]
    fn owned_levels_ignore_tier() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, 2);
        for p in &mut w.passives {
            p.1 = passive_max(p.0);
        }
        let mut counts = vec![0u32; w.weapons.len()];
        for _ in 0..3000 {
            w.make_offer();
            for u in w.offer.take().unwrap() {
                if let Some(i) = w.weapons.iter().position(|o| u == Upgrade::Weapon(o.kind)) {
                    counts[i] += 1;
                }
            }
        }
        let mean = counts.iter().sum::<u32>() as f32 / counts.len() as f32;
        assert!(counts.iter().all(|c| (*c as f32 - mean).abs() <= mean * 0.15), "{counts:?}");
    }

    #[test]
    fn at_most_one_replacement_card() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, 2);
        let mut with = 0;
        for _ in 0..2000 {
            w.make_offer();
            let n = w.offer.take().unwrap().iter().filter(|u| w.is_replacement(**u)).count();
            assert!(n <= 1);
            with += n;
        }
        assert!(with as f32 > 2000.0 * REPLACE_CHANCE * 0.7, "{with}");
    }

    #[test]
    fn maxed_build_is_offered_its_missing_complement() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, MAX_LEVEL);
        for p in &mut w.passives {
            p.1 = passive_max(p.0);
        }
        let missing = complement(w.weapons[0].kind).0;
        let slots = hero_def(w.cfg.hero).passive_slots;
        let mut owned: Vec<PassiveKind> = Vec::new();
        for k in w.weapons.iter().map(|x| complement(x.kind).0).filter(|k| *k != missing) {
            if !owned.contains(&k) {
                owned.push(k);
            }
        }
        let needed = owned.len();
        let filler: Vec<PassiveKind> = ALL_PASSIVES.iter().copied().filter(|k| *k != missing && !owned.contains(k)).take(slots - needed).collect();
        owned.extend(filler);
        w.passives = owned.iter().map(|k| (*k, passive_max(*k))).collect();
        let fallbacks_ok = |rest: &[Upgrade]| rest.len() == 2 && rest[0] != rest[1] && rest.iter().all(|u| MAXED_FALLBACKS.contains(u));
        let mut with = 0;
        for _ in 0..1000 {
            w.make_offer();
            let offer = w.offer.take().unwrap();
            assert_eq!(offer.len(), 3);
            let (rep, rest): (Vec<Upgrade>, Vec<Upgrade>) = offer.into_iter().partition(|u| w.is_replacement(*u));
            if rep.is_empty() {
                assert_eq!(rest, MAXED_FALLBACKS.to_vec());
                continue;
            }
            with += 1;
            assert_eq!(rep, vec![Upgrade::Passive(missing)]);
            assert!(fallbacks_ok(&rest), "{rest:?}");
        }
        let share = with as f32 / 1000.0;
        assert!((share - MAXED_REPLACE_CHANCE).abs() < 0.07, "{share}");

        w.passives[slots - 1] = (missing, passive_max(missing));
        for _ in 0..200 {
            w.make_offer();
            let offer = w.offer.take().unwrap();
            assert_eq!(offer.len(), 3);
            let (rep, rest): (Vec<Upgrade>, Vec<Upgrade>) = offer.into_iter().partition(|u| w.is_replacement(*u));
            assert!(rep.len() <= 1);
            assert!(rep.is_empty() || fallbacks_ok(&rest));
        }
    }

    #[test]
    fn fallback_pick_in_replace_offer_applies_without_replacing() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, MAX_LEVEL);
        let rep = *w.replacement_candidates().first().unwrap();
        w.offer = Some(vec![rep, Upgrade::Heal, Upgrade::Sen]);
        w.pending_levels = 1;
        w.choose(1);
        assert!(w.replacing.is_none());
        assert!(w.offer.is_none());
    }

    #[test]
    fn replacement_inherits_half() {
        for (old, expect) in [(7, 3), (1, 1)] {
            let mut w = World::new(WorldConfig::default());
            full_loadout(&mut w, 2);
            w.weapons[2].level = old;
            w.weapons[2].awakened = true;
            let (gone, new) = (w.weapons[2].kind, unowned_weapon(&w));
            with_replacement(&mut w, Upgrade::Weapon(new));
            assert!(w.replacing.is_some() && w.offer.is_some() && w.pending_levels == 1);
            w.replace_with(Some(2));
            assert_eq!((w.weapons[2].kind, w.weapons[2].level, w.weapons[2].awakened), (new, expect, false));
            assert_eq!(w.weapon_level(gone), 0);
            assert_eq!(w.pending_levels, 0);
        }
    }

    #[test]
    fn bot_only_replaces_with_a_higher_tier() {
        use WeaponKind::*;
        let mut w = World::new(WorldConfig::default());
        w.weapons = [FoxMirage, MukadeSpines, Ofuda, FlamePool].map(|k| weapon(k, 2)).to_vec();
        w.offer = Some(vec![Upgrade::Weapon(RepulsionStar), Upgrade::Heal]);
        w.pending_levels = 1;
        w.bot_pick();
        assert!(w.replacing.is_none() && w.offer.is_none());
        assert_eq!(w.weapon_level(RepulsionStar), 0);

        w.weapons[1] = weapon(RepulsionStar, 1);
        w.offer = Some(vec![Upgrade::Weapon(SpiritSeal), Upgrade::Heal]);
        w.pending_levels = 1;
        w.bot_pick();
        assert_eq!((w.weapons[1].kind, w.weapons[1].level), (SpiritSeal, 1));
    }

    #[test]
    fn cancel_keeps_the_offer() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, 2);
        let new = unowned_weapon(&w);
        with_replacement(&mut w, Upgrade::Weapon(new));
        let before = w.offer.clone();
        w.replace_with(None);
        assert!(w.replacing.is_none());
        assert_eq!(w.offer, before);
        assert_eq!(w.pending_levels, 1);
    }

    #[test]
    fn awakening_survives_complement_replacement() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, 2);
        w.weapons[0].awakened = true;
        let comp = complement(w.weapons[0].kind).0;
        let j = w.passives.iter().position(|(k, _)| *k == comp).unwrap_or_else(|| {
            w.passives[0].0 = comp;
            0
        });
        let new = *ALL_PASSIVES.iter().find(|k| w.passive_level(**k) == 0).unwrap();
        with_replacement(&mut w, Upgrade::Passive(new));
        w.replace_with(Some(j));
        assert_eq!(w.passive_level(comp), 0);
        assert!(w.weapons[0].awakened);
    }

    #[test]
    fn replacing_vitality_returns_its_hp() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, 2);
        let base = w.player.max_hp;
        w.passives = ALL_PASSIVES.iter().filter(|k| **k != PassiveKind::Vitality).take(hero_def(HeroId::Ren).passive_slots).map(|k| (*k, 2)).collect();
        w.passives[0] = (PassiveKind::Vitality, 4);
        w.player.max_hp += 80.0;
        let other = *ALL_PASSIVES.iter().find(|k| w.passive_level(**k) == 0 && **k != PassiveKind::Vitality).unwrap();
        with_replacement(&mut w, Upgrade::Passive(other));
        w.replace_with(Some(0));
        assert_eq!(w.player.max_hp, base);

        w.passives[0] = (other, 6);
        with_replacement(&mut w, Upgrade::Passive(PassiveKind::Vitality));
        w.replace_with(Some(0));
        assert_eq!(w.passive_level(PassiveKind::Vitality), 3);
        assert_eq!(w.player.max_hp, base + 60.0);
    }

    #[test]
    fn inherited_level_respects_the_new_skills_cap() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, 2);
        w.passives[0] = (ALL_PASSIVES.iter().copied().find(|k| *k != PassiveKind::Abundance && *k != PassiveKind::Vitality).unwrap(), 8);
        w.passives.retain(|(k, _)| *k != PassiveKind::Abundance);
        while w.passives.len() < hero_def(HeroId::Ren).passive_slots {
            let k = *ALL_PASSIVES.iter().find(|k| w.passive_level(**k) == 0 && **k != PassiveKind::Abundance).unwrap();
            w.passives.push((k, 2));
        }
        with_replacement(&mut w, Upgrade::Passive(PassiveKind::Abundance));
        w.replace_with(Some(0));
        assert_eq!(w.passive_level(PassiveKind::Abundance), 3);
    }

    #[test]
    fn replacing_vitality_floors_max_hp() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, 2);
        w.passives = ALL_PASSIVES.iter().filter(|k| **k != PassiveKind::Vitality).take(hero_def(HeroId::Ren).passive_slots).map(|k| (*k, 2)).collect();
        w.passives[0] = (PassiveKind::Vitality, 8);
        w.player.max_hp = 127.0;
        w.player.hp = 127.0;
        let other = *ALL_PASSIVES.iter().find(|k| w.passive_level(**k) == 0 && **k != PassiveKind::Vitality).unwrap();
        with_replacement(&mut w, Upgrade::Passive(other));
        w.replace_with(Some(0));
        assert_eq!(w.player.max_hp, 10.0);
        assert!(w.player.hp > 0.0 && w.player.hp <= w.player.max_hp);
    }

    #[test]
    fn extra_weapons_are_not_replaceable() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, 2);
        let mut extra = weapon(unowned_weapon(&w), 1);
        extra.extra = true;
        w.weapons.insert(1, extra);
        let new = w.weapons.iter().find(|o| o.extra).map(|_| *ALL_WEAPONS.iter().find(|k| w.weapon_level(**k) == 0 && hero_def(HeroId::Ren).weapon_pool().contains(k)).unwrap()).unwrap();
        w.replacing = Some(Upgrade::Weapon(new));
        let targets = w.replace_targets();
        assert_eq!(targets.len(), 4);
        assert!(targets.iter().all(|(u, _)| *u != Upgrade::Weapon(w.weapons[1].kind)));
        assert!(!w.is_replacement(Upgrade::Weapon(w.weapons[1].kind)));
    }

    #[test]
    fn replacement_consumes_one_level() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, 2);
        let new = unowned_weapon(&w);
        w.offer = Some(vec![Upgrade::Weapon(new)]);
        w.pending_levels = 3;
        w.choose(0);
        w.replace_with(Some(1));
        assert!(w.offer.is_some());
        assert_eq!(w.pending_levels, 2);
    }

    #[test]
    fn signature_can_be_replaced() {
        let mut w = World::new(WorldConfig::default());
        full_loadout(&mut w, 2);
        w.weapons[0].kind = hero_def(HeroId::Ren).signature;
        w.replacing = Some(Upgrade::Weapon(unowned_weapon(&w)));
        assert!(w.replace_targets().iter().any(|(u, _)| *u == Upgrade::Weapon(hero_def(HeroId::Ren).signature)));
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
        w.time = w.stage().round_start(w.stage().shrine_rounds[0]) - 0.01;
        w.player.iframes = 1e9;
        w.step(Input::default());
        let s = w.shrine.expect("shrine should appear at its round");
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
                w.bot_pick();
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
        w.shots.push(Shot { pos: Vec2::new(50.0, 0.0), vel: Vec2::ZERO, life: 5.0, style: 0, elem: None });
        w.shots.push(Shot { pos: Vec2::new(900.0, 0.0), vel: Vec2::ZERO, life: 5.0, style: 0, elem: None });
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

    fn step_to(w: &mut World, t: f32) {
        w.player.iframes = 1e9;
        while w.time < t {
            w.step(Input::default());
        }
    }

    fn count_of(w: &World, k: EnemyKind) -> usize {
        w.enemies.kind.iter().filter(|e| **e == k).count()
    }

    #[test]
    fn mini_spawns_at_round_16_once() {
        let mut w = World::new(WorldConfig::default());
        let st = w.stage();
        let at = st.round_start(16);
        w.time = at - 1.0;
        w.round_seen = 15;
        step_to(&mut w, at - DT);
        assert_eq!(count_of(&w, st.mini), 0);
        step_to(&mut w, at + 3.0);
        assert_eq!(count_of(&w, st.mini), 1);
        let spawns = |w: &World| w.events.iter().filter(|e| matches!(e, Event::BossSpawned(k) if *k == st.mini)).count();
        let mut seen = 1;
        for _ in 0..240 {
            w.events.clear();
            w.step(Input::default());
            seen += spawns(&w);
        }
        assert_eq!(seen, 1);
    }

    #[test]
    fn mid_boss_at_round_18_only_on_worlds() {
        for (id, expect) in [(StageId::ShrineValley, 1), (StageId::Shotengai, 0)] {
            let mut w = World::new(WorldConfig { stage: id, ..Default::default() });
            let st = w.stage();
            let at = st.round_start(18);
            w.time = at - 1.0;
            w.round_seen = 17;
            step_to(&mut w, at + 1.0);
            let bosses = w.enemies.kind.iter().filter(|k| k.rank() == Rank::Boss).count();
            assert_eq!(bosses, expect, "{id:?}");
            if let Some(b) = st.boss {
                assert_eq!(count_of(&w, b), 1);
            }
        }
    }

    #[test]
    fn elites_start_at_round_8() {
        let mut w = World::new(WorldConfig::default());
        let st = w.stage();
        let at = st.round_start(8);
        w.time = at - 2.0;
        w.round_seen = 7;
        step_to(&mut w, at - DT);
        assert_eq!(count_of(&w, st.elite), 0);
        step_to(&mut w, at + 0.1);
        assert_eq!(count_of(&w, st.elite), 1);
    }

    #[test]
    fn round_4_drops_a_chest() {
        let mut w = World::new(WorldConfig::default());
        let st = w.stage();
        let at = st.round_start(4);
        w.time = at - 1.0;
        w.round_seen = 3;
        step_to(&mut w, at - DT);
        let chests = w.chests.len();
        step_to(&mut w, at + 0.1);
        assert_eq!(w.chests.len(), chests + 1);
    }

    fn is_uncommon(k: EnemyKind) -> bool {
        k.rarity() == EnemyRarity::Uncommon
    }

    fn live_leaders(w: &World) -> usize {
        w.enemies.kind.iter().filter(|k| k.is_leader()).count()
    }

    #[test]
    fn round_four_spawns_uncommons() {
        let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, ..Default::default() });
        let st = w.stage();
        let at = st.round_start(4);
        w.time = at - 1.0;
        w.round_seen = 3;
        step_to(&mut w, at - DT);
        let (chests, cranes) = (w.chests.len(), count_of(&w, st.fodder_b));
        step_to(&mut w, at + 0.1);
        assert_eq!(w.chests.len(), chests + 1);
        assert!(count_of(&w, st.uncommon[0]) + count_of(&w, st.uncommon[1]) >= 20);
        assert!(count_of(&w, st.fodder_b) <= cranes + 2);
    }

    #[test]
    fn uncommons_join_the_mix() {
        let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, seed: 7, ..Default::default() });
        w.player.iframes = 1e9;
        let st = w.stage();
        let (mut early, mut late, mut late_total) = (0, 0, 0);
        while w.time < st.round_start(11) {
            w.enemies.hp.iter_mut().for_each(|h| *h = 0.0);
            w.grid.rebuild(w.player.pos, &w.enemies.pos);
            w.remove_dead_enemies();
            if w.paused() {
                w.time += 0.5;
                continue;
            }
            let first = w.next_id;
            w.step(Input::default());
            let round = w.round();
            let fresh = w.enemies.id.iter().zip(&w.enemies.kind).filter(|(id, _)| id.wrapping_sub(first) < 1 << 31);
            for (_, k) in fresh {
                if round <= 4 && is_uncommon(*k) && w.time < st.round_start(4) {
                    early += 1;
                }
                if round >= 5 && round != 8 && !k.is_boss() && k.rank() != Rank::Elite {
                    late_total += 1;
                    late += is_uncommon(*k) as usize;
                }
            }
        }
        assert_eq!(early, 0);
        let share = late as f32 / late_total as f32;
        assert!((0.015..0.06).contains(&share), "share {share} ({late}/{late_total})");
    }

    #[test]
    fn leaders_arrive_on_their_rounds() {
        let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, ..Default::default() });
        let st = w.stage();
        w.player.iframes = 1e9;
        w.next_elite = f32::MAX;
        let mut arrived = Vec::new();
        for r in 11..=20u8 {
            let at = st.round_start(r);
            w.time = at - 0.5;
            w.round_seen = r - 1;
            let elites = count_of(&w, st.elite);
            w.events.clear();
            while w.time < at + 0.2 {
                w.step(Input::default());
                if w.events.iter().any(|e| matches!(e, Event::LeaderSpawned { kind } if *kind == st.leader)) {
                    arrived.push(r);
                    w.events.clear();
                }
            }
            if r == 12 {
                assert_eq!(count_of(&w, st.elite), elites, "round 12 must not add stand-in elites");
            }
            if let Some(j) = w.enemies.kind.iter().position(|k| k.is_leader()) {
                kill_step(&mut w, j);
            }
        }
        assert_eq!(arrived, vec![12, 15, 18]);
    }

    #[test]
    fn leader_near_kuon_is_slowed_by_stillness() {
        let speed_near = |hero| {
            let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, hero, ..Default::default() });
            w.spawn_leader_at(w.player.pos + Vec2::new(40.0, 0.0));
            let l = w.enemies.kind.iter().position(|k| k.is_leader()).unwrap();
            w.update_bosses();
            w.enemies.vel[l].length()
        };
        assert!(speed_near(HeroId::Kuon) < speed_near(HeroId::Riku) * 0.5);
    }

    #[test]
    fn leader_brings_a_crew() {
        let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, ..Default::default() });
        let st = w.stage();
        w.spawn_leader();
        assert_eq!(live_leaders(&w), 1);
        assert_eq!(count_of(&w, st.fodder), (10.0 * w.cfg.difficulty.density()) as usize);
        let l = w.enemies.kind.iter().position(|k| k.is_leader()).unwrap();
        for i in 0..w.enemies.len() {
            if i != l {
                let d = (w.enemies.pos[i] - w.enemies.pos[l]).length();
                assert!((d - 50.0).abs() < 0.01, "crew at {d}");
            }
        }
        w.spawn_leader();
        assert_eq!(live_leaders(&w), 1);
    }

    #[test]
    fn round_skip_spawns_one_leader() {
        let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, ..Default::default() });
        let st = w.stage();
        w.player.iframes = 1e9;
        w.time = st.round_start(11) + 0.1;
        w.round_seen = 11;
        w.step(Input::default());
        w.time = st.round_start(19) + 0.1;
        w.events.clear();
        w.step(Input::default());
        assert_eq!(live_leaders(&w), 1);
        assert_eq!(w.events.iter().filter(|e| matches!(e, Event::LeaderSpawned { .. })).count(), 1);
    }

    #[test]
    fn round_events_fire_once_each() {
        let mut w = World::new(WorldConfig::default());
        let end = w.stage().final_at + 1.0;
        let mut started = Vec::new();
        while w.time < end {
            w.player.iframes = 1e9;
            w.enemies.hp.iter_mut().for_each(|h| *h = 1e9);
            if w.paused() {
                w.time += 0.5;
                continue;
            }
            w.events.clear();
            w.step(Input::default());
            for e in &w.events {
                if let Event::RoundStarted { round, headline } = e {
                    assert_eq!(*headline, StageDef::headline(*round));
                    started.push(*round);
                }
            }
        }
        assert_eq!(started, (1..=20).collect::<Vec<u8>>());
    }

    #[test]
    fn time_jump_fires_only_current_round() {
        let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, ..Default::default() });
        let st = w.stage();
        w.player.iframes = 1e9;
        w.time = st.round_start(16) + 0.1;
        w.events.clear();
        w.step(Input::default());
        let rounds: Vec<u8> = w.events.iter().filter_map(|e| if let Event::RoundStarted { round, .. } = e { Some(*round) } else { None }).collect();
        assert_eq!(rounds, vec![16]);
        assert_eq!(count_of(&w, st.mini), 1);
        assert!(w.chests.is_empty());
        assert!(!w.events.iter().any(|e| matches!(e, Event::EliteWave)));
    }

    #[test]
    fn late_time_jump_keeps_one_boss_each() {
        let jump = |w: &mut World, t: f32| {
            w.time = t;
            w.step(Input::default());
        };
        let totals = |w: &World| {
            let st = w.stage();
            (count_of(w, st.mini), count_of(w, st.boss.unwrap()), count_of(w, st.final_boss))
        };
        for (hops, want) in [
            (vec![16.1_f32, 19.1], (1, 1, 0)),
            (vec![16.1, 18.1, 20.0], (1, 1, 1)),
            (vec![19.1], (1, 1, 0)),
        ] {
            let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, ..Default::default() });
            w.player.iframes = 1e9;
            let st = w.stage();
            for r in hops {
                let t = if r == 20.0 { st.final_at + 0.1 } else { st.round_start(r as u8) + (r - r.floor()) };
                jump(&mut w, t);
            }
            assert_eq!(totals(&w), want);
        }
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
        w.hurt_player(1e6, None);
        assert_eq!(w.state, RunState::Playing);
        assert!(w.player.hp > 0.0);
        w.player.iframes = 0.0;
        w.hurt_player(1e6, None);
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
        w.weapons = vec![OwnedWeapon { kind: WeaponKind::FoxMirage, level: 1, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }];
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
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(a) * (40.0 + (k % 4) as f32 * 25.0));
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
                w.bot_pick();
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
    fn maxed_fallbacks_scale_with_the_run() {
        let mut w = World::new(WorldConfig::default());
        w.player.max_hp = 1200.0;
        assert_eq!(w.fallback_amount(Upgrade::Heal), 1200.0 * ONIGIRI_HEAL);
        assert!(w.describe_upgrade(Upgrade::Heal, 1).contains("360"), "{}", w.describe_upgrade(Upgrade::Heal, 1));
        let early = w.fallback_amount(Upgrade::Sen);
        w.time = 600.0;
        let late = w.fallback_amount(Upgrade::Sen);
        assert!(late > early * 2.0, "Sen pouch should grow over the run: {early} -> {late}");
        let before = w.sen;
        w.apply_upgrade(Upgrade::Sen);
        assert_eq!(w.sen - before, late.round() as u32);
    }

    #[test]
    fn threat_counts_live_enemies_in_range_by_rank() {
        let mut w = World::new(WorldConfig { stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        let p = w.player.pos;
        for n in 0..12 {
            w.spawn_enemy(EnemyKind::Wisp, p + Vec2::from_angle(n as f32) * 60.0);
            w.spawn_enemy(EnemyKind::Wisp, p + Vec2::from_angle(n as f32) * 600.0);
        }
        w.spawn_enemy(EnemyKind::Oni, p + Vec2::new(0.0, 80.0));
        w.enemies.hp[0] = 0.0;
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        assert_eq!(w.threat_near(AUTO_SPECIAL_RADIUS), 11.0 + Rank::Elite.threat());
    }

    #[test]
    fn difficulty_steps_per_round() {
        let mut w = World::new(WorldConfig::default());
        let mut hp_at = |t: f32| {
            w.time = t;
            w.enemy_hp_scale()
        };
        let st = StageId::Shotengai.def();
        let (early, late, next) = (hp_at(st.round_start(5) + 1.0), hp_at(st.round_start(6) - 1.0), hp_at(st.round_start(6) + 1.0));
        assert_eq!(early, late, "HP scale should hold flat within a round");
        assert!(next > late, "HP scale should step up at the next round");
    }

    #[test]
    fn headline_rounds_step_harder() {
        let mut w = World::new(WorldConfig::default());
        let mut scales_in = |r: u8| {
            w.time = w.stage().round_start(r) + 1.0;
            (w.enemy_hp_scale(), w.enemy_damage_scale())
        };
        let ((h6, d6), (h7, d7), (h8, d8)) = (scales_in(6), scales_in(7), scales_in(8));
        assert!(h8 / h7 > h7 / h6, "HP step into headline round 8 ({}) should beat the step before ({})", h8 / h7, h7 / h6);
        assert!(d8 / d7 > d7 / d6, "damage step into headline round 8 ({}) should beat the step before ({})", d8 / d7, d7 / d6);
    }

    #[test]
    fn difficulty_never_drops() {
        for stage in crate::campaign::ALL_STAGES {
            let mut w = World::new(WorldConfig { stage, ..Default::default() });
            let mut scales_in = |r: u8| {
                let st = w.stage();
                w.time = st.round_start(r) + st.round_len() / 2.0;
                (w.enemy_hp_scale(), w.enemy_damage_scale())
            };
            for r in 1..=19 {
                let ((h, d), (h2, d2)) = (scales_in(r), scales_in(r + 1));
                assert!(h2 >= h, "{stage:?}: HP scale drops from round {r} ({h}) to {} ({h2})", r + 1);
                assert!(d2 >= d, "{stage:?}: damage scale drops from round {r} ({d}) to {} ({d2})", r + 1);
            }
        }
    }

    #[test]
    fn late_rings_are_thinned_like_the_ladder() {
        let mut w = World::new(WorldConfig::default());
        let at = w.stage().round_start(*w.stage().ring_rounds.last().unwrap());
        assert!(ladder_rung(w.stage().pace(at)) > 0, "first ring should land on a later rung");
        w.step(Input::default());
        w.enemies = Enemies::default();
        w.time = at;
        w.round_seen = w.round() - 1;
        w.step(Input::default());
        let p = w.player.pos;
        let ring = w.enemies.pos.iter().filter(|e| ((**e - p).length() - 270.0).abs() < 20.0).count();
        let want = (120.0 * w.cfg.difficulty.density() * LADDER_COUNT[1]) as usize;
        assert!(ring <= want + 2, "ring spawned {ring}, expected about {want}");
    }

    #[test]
    fn items_are_grabbed_from_a_short_walk_away() {
        let mut w = World::new(WorldConfig::default());
        w.step(Input::default());
        w.pickups.push((w.player.pos + Vec2::new(25.0, 0.0), PickupKind::OniSake));
        w.chests.push((w.player.pos + Vec2::new(0.0, -25.0), 1));
        w.step(Input::default());
        assert!(w.pickups.is_empty(), "pickup 25px away should be grabbed");
        assert!(w.chests.is_empty(), "chest 25px away should be opened");
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
        w.coins.push((Vec2::new(-60.0, 0.0), 5));
        let m = w.map_markers();
        let has = |k: MarkerKind| m.iter().any(|x| x.kind == k);
        assert!(has(MarkerKind::Enemy) && has(MarkerKind::Elite) && has(MarkerKind::MiniBoss));
        assert!(has(MarkerKind::Chest { tier: 3 }) && has(MarkerKind::Shrine) && has(MarkerKind::Pickup(PickupKind::OniSake)));
        assert!(has(MarkerKind::Coin), "Sen coins should show on the map");
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
        w.hurt_player(30.0, None);
        assert_eq!(w.player.hp, hp);
        w.player.iframes = 0.0;
        w.hurt_player(30.0, None);
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
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(k as f32) * 90.0);
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
        w.shots.push(Shot { pos: w.player.pos, vel: Vec2::ZERO, life: 5.0, style: 0, elem: None });
        w.step(Input::default());
        assert_eq!(w.player.hp, hp, "the barrier blocks the first fireball");
        assert!(w.barrier_cd > 0.0);
        // Divine Purification: invulnerable, storms talismans, ends in the final blast.
        w.weapons.clear();
        for k in 0..30 {
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(k as f32 * 0.21) * (45.0 + (k % 3) as f32 * 30.0));
        }
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        w.player.special = 1.0;
        w.step(Input { special: true, ..Default::default() });
        assert!(w.events.iter().any(|e| matches!(e, Event::DivineAwakened { .. })));
        let hp = w.player.hp;
        w.hurt_player(999.0, None);
        assert_eq!(w.player.hp, hp, "invulnerable during the form");
        let mut finale = false;
        for _ in 0..60 * 9 {
            if w.offer.is_some() {
                w.bot_pick();
            }
            w.step(Input::default());
            finale |= w.events.iter().any(|e| matches!(e, Event::Burst { style: s, .. } if *s == style::PURIFY));
        }
        assert!(finale, "Final Purification should go off");
        assert!(w.kills >= 20, "killed {}", w.kills);
    }

    fn damage_over(kind: WeaponKind, awakened: bool, hero: HeroId) -> f32 {
        let mut w = World::new(WorldConfig { seed: 5, hero, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.time = AWAKEN_FROM + 1.0;
        w.weapons = vec![OwnedWeapon { kind, level: MAX_LEVEL, awakened, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }];
        for k in 0..60 {
            let a = k as f32 / 60.0 * std::f32::consts::TAU;
            w.spawn_enemy(EnemyKind::Oni, Vec2::from_angle(a) * (20.0 + (k % 5) as f32 * 22.0));
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

    #[test]
    fn crowds_walk_around_props_in_every_stage() {
        for stage in crate::campaign::ALL_STAGES {
            // A wide prop to walk around.
            let mut found = None;
            'outer: for cy in 0..40 {
                for cx in 0..40 {
                    let (ps, n) = crate::stage::chunk_props(stage, cx, cy);
                    for p in &ps[..n] {
                        if matches!(p.kind, crate::stage::PropKind::C | crate::stage::PropKind::Stall) {
                            found = Some(*p);
                            break 'outer;
                        }
                    }
                }
            }
            let prop = found.expect("a wide prop");
            let mut w = World::new(WorldConfig { stage, stress: None, ..Default::default() });
            w.weapons.clear();
            w.player.pos = prop.pos + Vec2::new(0.0, 70.0);
            w.player.hp = 1e9;
            w.player.max_hp = 1e9;
            for k in 0..30 {
                let kind = w.stage().fodder;
                w.spawn_enemy(kind, prop.pos + Vec2::new(-30.0 + k as f32 * 2.0, -60.0));
            }
            let (min, max) = prop.footprint();
            let mut worst = 0.0f32;
            for _ in 0..600 {
                w.time = 1.0;
                w.step(Input::default());
                for i in 0..w.enemies.len() {
                    let c = w.enemies.pos[i];
                    let q = Vec2::new(c.x.clamp(min.x, max.x), c.y.clamp(min.y, max.y));
                    let pen = w.enemies.radius[i] - (c - q).length();
                    worst = worst.max(pen);
                }
            }
            assert!(worst < 2.5, "{stage:?}: enemies sank {worst:.1}px into a prop");
        }
    }

    /// Full 15-minute bot run; prints how far it got. `cargo test -p sim balance -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn balance_full_run() {
        let seeds: u64 = std::env::var("BALANCE_SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
        let first: u64 = std::env::var("BALANCE_SEED0").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
        for seed in first..first + seeds {
            let started = std::time::Instant::now();
            let hero = std::env::var("BALANCE_HERO").ok().and_then(|h| HeroId::from_key(&h)).unwrap_or(HeroId::Ren);
            let stage = std::env::var("BALANCE_STAGE").ok().and_then(|h| StageId::from_key(&h)).unwrap_or(StageId::Shotengai);
            let difficulty = std::env::var("BALANCE_DIFF").ok().and_then(|h| Difficulty::from_key(&h)).unwrap_or(Difficulty::Normal);
            // BALANCE_ASC=n: Nightmare+n. BALANCE_GEAR=count:rarity (e.g. 4:epic): that many random
            // pieces at the stage's drop item level. BALANCE_AFK=1: the bot never moves.
            // BALANCE_TRACE=1: a line per minute (lowest HP, damage taken, enemies alive).
            let ascension: u32 = std::env::var("BALANCE_ASC").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
            let afk = std::env::var("BALANCE_AFK").is_ok_and(|v| v == "1");
            // BALANCE_AFK_AFTER=secs: play normally, then stop moving (a "semi-AFK" player).
            let afk_after: f32 = std::env::var("BALANCE_AFK_AFTER").ok().and_then(|s| s.parse().ok()).unwrap_or(f32::INFINITY);
            let trace = std::env::var("BALANCE_TRACE").is_ok_and(|v| v == "1");
            let gear = match std::env::var("BALANCE_GEAR").ok().and_then(|g| g.split_once(':').map(|(n, r)| (n.parse::<usize>().ok(), r.to_string()))) {
                Some((Some(n), r)) => {
                    let rarity = [Rarity::Common, Rarity::Uncommon, Rarity::Rare, Rarity::Epic, Rarity::Legendary, Rarity::Mythic]
                        .into_iter()
                        .find(|x| x.name().eq_ignore_ascii_case(&r))
                        .unwrap_or(Rarity::Rare);
                    let mut rng = crate::rng::Rng::new(seed ^ 0x9e37);
                    let ilvl = crate::gear::drop_ilvl(stage, difficulty.index(), ascension);
                    let slots = [crate::gear::Slot::Hat, crate::gear::Slot::Top, crate::gear::Slot::Bottom, crate::gear::Slot::Gloves, crate::gear::Slot::Shoes, crate::gear::Slot::Necklace, crate::gear::Slot::Ring, crate::gear::Slot::Earring, crate::gear::Slot::Charm, crate::gear::Slot::Omamori];
                    let items: Vec<_> = slots.iter().take(n).map(|s| crate::gear::roll_item(&mut rng, Some(*s), rarity, ilvl)).collect();
                    Loadout::from_items(&items.iter().collect::<Vec<_>>())
                }
                _ => Loadout::default(),
            };
            // BALANCE_META=max: every Shrine Shop upgrade at max rank and every world's relic.
            let meta = if std::env::var("BALANCE_META").is_ok_and(|v| v == "max") {
                let mut prof = crate::meta::Profile::default();
                for p in crate::meta::ALL_POWERUPS {
                    prof.ranks[p.index()] = p.max_rank();
                }
                prof.relics = crate::campaign::ALL_STAGES.iter().filter_map(|s| s.def().relic).collect();
                prof.bonus()
            } else {
                Bonus::default()
            };
            let mut w = World::new(WorldConfig { seed, hero, stage, difficulty, ascension, gear, meta, ..Default::default() });
            let (mut hp_low, mut taken, mut last_hp, mut next_trace) = (1.0f32, 0.0f32, 0.0f32, 60.0f32);
            let mut dealt = 0.0f64;
            let mut boss_at: Vec<(EnemyKind, f32)> = Vec::new();
            last_hp = last_hp.max(w.player.hp);
            let mut level_at_15 = None;
            let mut maxed_offers = 0u32;
            let mut first_maxed = None;
            let mut gear_drops = 0u32;
            while w.state == RunState::Playing {
                if level_at_15.is_none() && w.time >= RUN_SECONDS {
                    level_at_15 = Some(w.player.level);
                }
                if w.offer.is_some() {
                    if w.upgrade_pool(true).is_empty() {
                        maxed_offers += 1;
                        first_maxed.get_or_insert(w.time);
                    }
                    w.bot_pick();
                }
                if w.chest_result.is_some() {
                    w.close_chest();
                }
                if w.shrine_offer.is_some() {
                    w.choose_shrine(Some(0));
                }
                let input = if afk || w.time >= afk_after { Input::default() } else { w.bot_input() };
                w.step(input);
                gear_drops += w.events.iter().filter(|e| matches!(e, Event::GearDrop { .. })).count() as u32;
                dealt += w.events.iter().map(|e| if let Event::Hit { amount, .. } = e { *amount as f64 } else { 0.0 }).sum::<f64>();
                for e in &w.events {
                    match e {
                        Event::BossSpawned(k) | Event::NightLordSpawned(k) => boss_at.push((*k, w.time)),
                        Event::Kill { kind, .. } if kind.is_boss() => {
                            if let Some(i) = boss_at.iter().position(|(k, _)| k == kind) {
                                let (k, t0) = boss_at.remove(i);
                                if trace {
                                    println!("  boss {:?} spawned {:.0}s, killed in {:.1}s", k, t0, w.time - t0);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if w.player.hp < last_hp {
                    taken += last_hp - w.player.hp;
                }
                last_hp = w.player.hp;
                hp_low = hp_low.min(w.player.hp / w.player.max_hp);
                if trace && w.time >= next_trace {
                    // dps_ratio: damage dealt this minute / enemy HP scale = how many "base enemies" of
                    // HP the player chews through per second. Flat = enemies keep pace with the build.
                    let dps = dealt / 60.0;
                    println!("  {:>4.0}s  hp_low {:>3.0}%  taken {:>6.0}  alive {:>4}  kills {:>6}  level {:>3}  enemy_hp x{:>6.1}  dmg x{:.1}  dps {:>9.0}  dps/hp {:>7.0}", w.time, hp_low * 100.0, taken, w.enemies.len(), w.kills, w.player.level, w.enemy_hp_scale(), w.enemy_damage_scale(), dps, dps / w.enemy_hp_scale() as f64);
                    dealt = 0.0;
                    next_trace += 60.0;
                    hp_low = 1.0;
                }
            }
            let awakened: Vec<_> = w.weapons.iter().filter(|x| x.awakened).map(|x| awakened_name(x.kind)).collect();
            let nl = w.boss().map(|b| format!("{} {:.0}%", b.2.short_name(), b.1 * 100.0)).unwrap_or("-".into());
            let uncommon_kills: u32 = ALL_ENEMIES.iter().filter(|k| k.rarity() == EnemyRarity::Uncommon).map(|k| w.kills_by_kind[k.index()]).sum();
            let leader_kills: u32 = ALL_ENEMIES.iter().filter(|k| k.is_leader()).map(|k| w.kills_by_kind[k.index()]).sum();
            let eligible: u32 = ALL_ENEMIES.iter().filter(|k| matches!(k.rank(), Rank::Mini | Rank::Boss | Rank::Final)).map(|k| w.kills_by_kind[k.index()]).sum();
            println!(
                "seed {seed} [boss {nl}] uncommon_kills {uncommon_kills} leader_kills {leader_kills} gear_drops {gear_drops} set_eligible_kills {eligible} bosses_down {:03b}: {:?} at {:>5.0}s  level {:>2}  level@15:00 {:?}  kills {:>6}  slots_full {:?}  maxed {}  unique {}  repl {}  maxed_offers {}  first_maxed {:?}  ({:.1}s real)  weapons {:?}  awakened {:?}",
                w.bosses_down,
                w.state,
                w.time,
                w.player.level,
                level_at_15,
                w.kills,
                w.slots_full_round,
                w.weapons.iter().filter(|x| !x.extra && x.level >= MAX_LEVEL).count(),
                w.unique_offered as u8,
                w.replacements,
                maxed_offers,
                first_maxed.map(|t| t as u32),
                started.elapsed().as_secs_f32(),
                w.weapons.iter().map(|x| (x.kind, x.level)).collect::<Vec<_>>(),
                awakened
            );
        }
    }

    fn damage_on_one(kind: WeaponKind, hero: HeroId) -> f32 {
        let mut w = World::new(WorldConfig { seed: 5, hero, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.weapons = vec![OwnedWeapon { kind, level: MAX_LEVEL, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }];
        w.spawn_enemy(EnemyKind::Oni, Vec2::new(60.0, 0.0));
        w.enemies.speed.iter_mut().for_each(|s| *s = 0.0);
        w.enemies.hp.iter_mut().for_each(|h| *h = 1e9);
        let mut dealt = 0.0;
        for _ in 0..60 * 6 {
            w.step(Input::default());
            dealt += w.events.iter().filter_map(|e| if let Event::Hit { amount, .. } = e { Some(*amount) } else { None }).sum::<f32>();
        }
        dealt
    }

    /// Ranks every weapon (crowd and single-target damage) and passive (bot wins). `cargo test --release -p sim skill_strength -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn skill_strength() {
        let hero = std::env::var("SKILL_HERO").ok().map_or(HeroId::Ren, |k| HeroId::from_key(&k).expect("SKILL_HERO is a hero key"));
        let pool = hero_def(hero).weapon_pool();
        let mut weapons: Vec<(WeaponKind, f32, f32, f32)> = ALL_WEAPONS
            .iter()
            .filter(|k| pool.contains(k))
            .map(|&k| {
                let (crowd, single) = (damage_over(k, false, hero), damage_on_one(k, hero));
                (k, crowd, single, (crowd * single).sqrt())
            })
            .collect();
        weapons.sort_by(|a, b| b.3.total_cmp(&a.3));
        println!("WEAPONS {hero:?} (strength = sqrt(crowd * single))");
        for (k, crowd, single, s) in &weapons {
            println!("{k:?}\tcrowd {crowd:.0}\tsingle {single:.0}\tstrength {s:.0}");
        }
        let first: u64 = std::env::var("SKILL_SEED0").ok().and_then(|s| s.parse().ok()).unwrap_or(21);
        let seeds: u64 = std::env::var("SKILL_SEEDS").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
        let only = std::env::var("SKILL_ONLY").ok();
        let mut passives: Vec<(Option<PassiveKind>, u32, f32)> = Vec::new();
        let candidates = std::iter::once(None).chain(ALL_PASSIVES.iter().map(|p| Some(*p)));
        for p in candidates.filter(|p| p.is_none_or(|p| only.as_ref().is_none_or(|o| format!("{p:?}") == *o))) {
            let (mut wins, mut total_time) = (0, 0.0);
            for seed in first..first + seeds {
                let mut w = World::new(WorldConfig { seed, hero, stage: StageId::ShrineValley, difficulty: Difficulty::Normal, ..Default::default() });
                if let Some(p) = p {
                    for _ in 0..passive_max(p) {
                        w.apply_upgrade(Upgrade::Passive(p));
                    }
                }
                while w.state == RunState::Playing {
                    if w.offer.is_some() {
                        w.bot_pick();
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
                wins += (w.state == RunState::Won) as u32;
                total_time += w.time;
            }
            passives.push((p, wins, total_time / seeds as f32));
        }
        passives.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.total_cmp(&a.2)));
        println!("PASSIVES (wins of {seeds}, ties by mean time)");
        for (p, wins, t) in &passives {
            println!("{}\twins {wins}\tmean time {t:.0}", p.map_or("(none)".to_string(), |p| format!("{p:?}")));
        }
    }

    #[test]
    fn vitality_raises_max_hp_for_the_rest_of_the_run() {
        let mut w = play(3, 60 * 10);
        let before = w.player.max_hp;
        w.offer = Some(vec![Upgrade::Passive(PassiveKind::Vitality)]);
        w.pending_levels = 1;
        w.bot_pick();
        for _ in 0..60 * 5 {
            w.step(Input::default());
        }
        assert_eq!(w.player.max_hp, before + 20.0);
    }

    #[test]
    fn chests_fill_free_slots_before_falling_back_to_onigiri() {
        let mut w = World::new(WorldConfig::default());
        w.weapons[0].level = MAX_LEVEL;
        w.open_chest(3);
        let res = w.chest_result.take().unwrap();
        assert!(!res.iter().any(|i| matches!(i, ChestItem::Extra(..))), "free slots left, but the chest gave a fallback: {res:?}");
    }

    #[test]
    fn chests_get_richer_as_the_night_goes_on() {
        let single_share = |t: f32| {
            let mut w = World::new(WorldConfig::default());
            w.time = t;
            let n = 4000;
            let singles = (0..n)
                .filter(|_| {
                    w.open_chest(1);
                    w.chest_result.take().unwrap().len() == 1
                })
                .count();
            singles as f32 / n as f32
        };
        let (early, late) = (single_share(0.0), single_share(600.0));
        assert!((early - 0.85).abs() < 0.03, "early single-reward share {early}");
        assert!((late - 0.5).abs() < 0.04, "10:00 single-reward share {late}");
        assert_eq!(chest_tier(EnemyKind::Oni), 2, "elite chests hold at least two rewards");
    }

    #[test]
    fn basic_fodder_stops_spawning_after_five_minutes() {
        let mut w = World::new(WorldConfig::default());
        w.time = 400.0;
        w.round_seen = w.round();
        for _ in 0..120 {
            w.director();
        }
        let fodder = w.stage().fodder;
        assert!(w.enemies.len() > 50);
        assert!(!w.enemies.kind.contains(&fodder), "basic fodder spawned past 5:00");
    }

    #[test]
    fn auto_special_fires_only_when_unlocked_and_in_trouble() {
        let mut w = play_as(HeroId::Kaede, 9, 60 * 20);
        w.player.special = 1.0;
        w.player.hp = w.player.max_hp * 0.3;
        w.step(Input::default());
        assert_eq!(w.specials_used, 0, "fired without being unlocked");
        w.cfg.auto_special = true;
        let crowd = |w: &mut World| {
            w.enemies = Enemies::default();
            for n in 0..AUTO_SPECIAL_THREAT as usize {
                w.spawn_enemy(EnemyKind::Wisp, w.player.pos + Vec2::from_angle(n as f32) * 60.0);
            }
            w.enemies.hp.iter_mut().for_each(|h| *h = 1e9);
        };
        w.player.hp = w.player.max_hp;
        crowd(&mut w);
        w.step(Input::default());
        assert_eq!(w.specials_used, 0, "fired at full health just for a crowd");
        w.player.hp = w.player.max_hp * 0.3;
        w.enemies = Enemies::default();
        w.step(Input::default());
        assert_eq!(w.specials_used, 0, "fired at low health with nobody around");
        crowd(&mut w);
        w.step(Input::default());
        assert_eq!(w.specials_used, 1, "low health and a crowd should trigger it");
    }
    fn ren_with(kind: WeaponKind, seed: u64) -> World {
        let mut w = World::new(WorldConfig { seed, hero: HeroId::Ren, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.weapons = vec![OwnedWeapon { kind, level: 1, awakened: false, timer: 0.0, phase: 0.0, extra: false, power: 1.0 }];
        w.bonus.crit_chance = 0.0;
        w
    }

    fn place(w: &mut World, offset: Vec2) -> u32 {
        w.spawn_enemy(EnemyKind::Wisp, w.player.pos + offset);
        let j = w.enemies.len() - 1;
        w.enemies.hp[j] = 1000.0;
        w.enemies.speed[j] = 0.0;
        w.enemies.id[j]
    }

    fn index_of(w: &World, id: u32) -> usize {
        w.enemies.id.iter().position(|&x| x == id).expect("enemy alive")
    }

    fn fire_seal(w: &mut World, amount: u32) {
        let weapon = w.weapons[0];
        let mut s = w.stats_of(&weapon);
        s.amount = amount;
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        w.fire_new_weapon(&weapon, &s, 1.0);
    }

    fn throw_samples(w: &mut World) -> Vec<Vec<Vec2>> {
        let mut out = vec![w.yinyang_positions(&w.weapons[0])];
        while w.weapons[0].phase < std::f32::consts::TAU {
            w.step(Input::default());
            out.push(w.yinyang_positions(&w.weapons[0]));
        }
        out
    }

    #[test]
    fn yinyang_flies_out_and_back() {
        let mut w = ren_with(WeaponKind::YinYangOrbs, 3);
        let p = w.player.pos;
        let weapon = w.weapons[0];
        let n = w.stats_of(&weapon).amount as usize;
        let samples = throw_samples(&mut w);
        let r = w.yinyang_radius(&weapon);
        for k in 0..n {
            let dir = Vec2::from_angle(k as f32 / n as f32 * std::f32::consts::TAU);
            let edge = (w.clip_to_reach(p, p + dir * 1000.0) - p).length() - r;
            let peak = samples.iter().map(|s| (s[k] - p).length()).fold(0.0, f32::max);
            assert!(peak >= 0.9 * edge, "orb {k} peaked at {peak} of {edge}");
            assert!((samples[0][k] - p).length() <= 12.0 && (samples[samples.len() - 1][k] - p).length() <= 12.0, "orb {k} not home at the ends");
            assert!(samples.iter().all(|s| w.in_reach(s[k], 0.0)), "orb {k} left the reach box");
        }
    }

    #[test]
    fn yinyang_does_not_orbit() {
        let mut w = ren_with(WeaponKind::YinYangOrbs, 3);
        let p = w.player.pos;
        let angle = |c: Vec2| (c - p).y.atan2((c - p).x);
        let first: Vec<f32> = throw_samples(&mut w).iter().map(|s| s[0]).filter(|c| (*c - p).length() > 20.0).map(angle).collect();
        assert!(first.windows(2).all(|a| (a[0] - a[1]).abs() < 0.01), "orb 0 swung around during one throw");
        for _ in 0..30 {
            w.step(Input::default());
        }
        let second = w.yinyang_positions(&w.weapons[0])[0];
        let mut turned = (angle(second) - first[0]).abs() % std::f32::consts::TAU;
        turned = turned.min(std::f32::consts::TAU - turned);
        assert!(turned > 0.5, "next throw flew the same way");
    }

    #[test]
    fn spirit_seal_stamps_random_in_reach() {
        let mut w = ren_with(WeaponKind::SpiritSeal, 5);
        let reach = w.reach_half();
        let inside: Vec<u32> = (0..3).map(|k| place(&mut w, Vec2::new(40.0 + 30.0 * k as f32, 10.0))).collect();
        for k in 0..2 {
            place(&mut w, Vec2::new(reach.x + 60.0 + 20.0 * k as f32, 0.0));
        }
        fire_seal(&mut w, 5);
        let mut stamped: Vec<u32> = w.blasts.iter().filter_map(|b| b.follow).collect();
        stamped.sort();
        assert_eq!(stamped, inside, "exactly the in-reach enemies, once each");

        let mut seen = std::collections::HashSet::new();
        for seed in 0..50 {
            let mut w = ren_with(WeaponKind::SpiritSeal, seed);
            for k in 0..10 {
                place(&mut w, Vec2::new(30.0 + 12.0 * k as f32, 20.0 - 4.0 * k as f32));
            }
            fire_seal(&mut w, 1);
            seen.extend(w.blasts.iter().filter_map(|b| b.follow));
        }
        assert!(seen.len() >= 4, "only {} distinct enemies stamped over 50 seeds", seen.len());
    }

    fn seal_radius(w: &World) -> f32 {
        SEAL_STAMP_RADIUS * w.stats_of(&w.weapons[0]).area * w.area_mult()
    }

    fn run_ticks(w: &mut World, ticks: u32) {
        for _ in 0..ticks {
            w.step(Input::default());
        }
    }

    #[test]
    fn spirit_seal_bursts_after_delay() {
        let mut w = ren_with(WeaponKind::SpiritSeal, 5);
        w.weapons[0].timer = 1e9;
        let stamped = place(&mut w, Vec2::new(60.0, 0.0));
        let beside_start = place(&mut w, Vec2::new(60.0, -30.0));
        let radius = seal_radius(&w);
        fire_seal(&mut w, 1);
        assert_eq!(w.blasts[0].follow, Some(stamped));
        let moved = Vec2::new(60.0, 70.0);
        let beside_end = place(&mut w, moved + Vec2::new(radius * 0.5, 0.0));
        let hp = |w: &World, id| w.enemies.hp[index_of(w, id)];
        run_ticks(&mut w, 30);
        let j = index_of(&w, stamped);
        w.enemies.pos[j] = w.player.pos + moved;
        run_ticks(&mut w, 55);
        assert_eq!((hp(&w, stamped), hp(&w, beside_end), hp(&w, beside_start)), (1000.0, 1000.0, 1000.0), "seal hurt before 1.5 s");
        run_ticks(&mut w, 10);
        assert!(hp(&w, stamped) < 1000.0 && hp(&w, beside_end) < 1000.0, "burst missed the enemy's current position");
        assert_eq!(hp(&w, beside_start), 1000.0, "burst landed where the stamp was placed");
    }

    #[test]
    fn stamp_stops_following_out_of_reach() {
        let mut w = ren_with(WeaponKind::SpiritSeal, 5);
        w.weapons[0].timer = 1e9;
        let stamped = place(&mut w, Vec2::new(60.0, 0.0));
        let last_in_reach = Vec2::new(60.0, 40.0);
        let radius = seal_radius(&w);
        fire_seal(&mut w, 1);
        assert_eq!(w.blasts[0].follow, Some(stamped));
        let neighbour = place(&mut w, last_in_reach + Vec2::new(radius * 0.5, 0.0));
        let j = index_of(&w, stamped);
        w.enemies.pos[j] = w.player.pos + last_in_reach;
        run_ticks(&mut w, 30);
        let j = index_of(&w, stamped);
        w.enemies.pos[j] = w.player.pos + Vec2::new(w.reach_half().x + 80.0, 40.0);
        run_ticks(&mut w, 65);
        assert!(w.blasts.is_empty(), "seal never burst");
        assert!(w.enemies.hp[index_of(&w, neighbour)] < 1000.0, "burst followed the enemy out of reach");
    }

    #[test]
    fn stamp_on_killed_enemy_still_bursts() {
        let mut w = ren_with(WeaponKind::SpiritSeal, 5);
        w.weapons[0].timer = 1e9;
        let stamped = place(&mut w, Vec2::new(60.0, 0.0));
        let offset = seal_radius(&w) * 0.5;
        let neighbour = place(&mut w, Vec2::new(60.0 + offset, 0.0));
        fire_seal(&mut w, 1);
        run_ticks(&mut w, 30);
        let j = index_of(&w, stamped);
        kill_step(&mut w, j);
        assert!(w.enemies.id.iter().all(|&id| id != stamped));
        run_ticks(&mut w, 55);
        assert_eq!(w.enemies.hp[index_of(&w, neighbour)], 1000.0);
        run_ticks(&mut w, 10);
        assert!(w.enemies.hp[index_of(&w, neighbour)] < 1000.0, "burst never landed");
    }

    #[test]
    fn spirit_seal_fires_no_orbiting_orbs() {
        let mut w = ren_with(WeaponKind::SpiritSeal, 5);
        place(&mut w, Vec2::new(60.0, 0.0));
        fire_seal(&mut w, 5);
        assert!((0..w.projectiles.len()).all(|i| w.projectiles.kind[i] != ProjKind::SealOrb));
    }

    fn live_world() -> World {
        let mut w = World::new(WorldConfig::default());
        w.weapons.clear();
        w.enemies = Enemies::default();
        w
    }

    fn count_kind(w: &World, kind: EnemyKind) -> usize {
        w.enemies.kind.iter().filter(|&&k| k == kind).count()
    }

    #[test]
    fn clay_bird_death_leaves_blast() {
        let mut w = live_world();
        let j = add_enemy(&mut w, EnemyKind::ClayBird, 300.0);
        let pos = w.enemies.pos[j];
        kill_step(&mut w, j);
        assert_eq!(w.hazards.len(), 1);
        let h = &w.hazards[0];
        assert_eq!((h.pos, h.radius, h.warn, h.spare_enemies, h.friendly), (pos, 34.0, 1.0, true, false));
        assert_eq!(h.kind, HazardKind::Blast { damage: 14.0 * w.enemy_damage_scale() });
        assert!(w.events.iter().any(|e| matches!(*e, Event::HazardWarn { pos: p, radius: 34.0 } if p == pos)));
    }

    #[test]
    fn clay_bird_blast_spares_enemies() {
        let mut w = live_world();
        let a = add_enemy(&mut w, EnemyKind::ClayBird, 300.0);
        let b = add_enemy(&mut w, EnemyKind::ClayBird, 310.0);
        w.player.pos = w.enemies.pos[a];
        w.player.iframes = 0.0;
        w.hazards.push(Hazard { pos: w.player.pos, radius: 34.0, warn: DT, life: 0.0, kind: HazardKind::Blast { damage: 10.0 }, tick: 0.0, friendly: false, spare_enemies: true });
        let (hp, before) = (w.enemies.hp[b], w.player.hp);
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        w.update_hazards();
        assert!(w.player.hp < before);
        assert_eq!(w.enemies.hp[b], hp);
        assert_eq!(w.enemies.hp[a], hp);
        assert!(w.hazards.is_empty());
    }

    #[test]
    fn swarm_mother_splits_into_three() {
        let mut w = live_world();
        let j = add_enemy(&mut w, EnemyKind::SwarmMother, 300.0);
        let death = w.enemies.pos[j];
        w.enemies.hp[j] = 0.0;
        w.step(Input::default());
        w.step(Input::default());
        assert_eq!(count_kind(&w, EnemyKind::CursedSpawn), 3);
        let spawn: Vec<usize> = (0..w.enemies.len()).filter(|&i| w.enemies.kind[i] == EnemyKind::CursedSpawn).collect();
        for &i in &spawn {
            assert!(((w.enemies.pos[i] - death).length() - 16.0).abs() < 0.01 + w.enemies.speed[i] * DT * 2.0);
            assert_eq!(w.enemies.xp[i], enemy_def(EnemyKind::CursedSpawn).xp);
        }
    }

    #[test]
    fn swarm_split_survives_screen_clear() {
        let mut w = live_world();
        let j = add_enemy(&mut w, EnemyKind::SwarmMother, 100.0);
        w.enemies.hp[j] = 10.0;
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        w.pickups.push((w.player.pos, PickupKind::ExorcismSeal));
        w.step(Input::default());
        assert_eq!(count_kind(&w, EnemyKind::SwarmMother), 0);
        w.step(Input::default());
        assert_eq!(count_kind(&w, EnemyKind::CursedSpawn), 3);
    }

    #[test]
    fn swarm_split_respects_max_enemies() {
        let mut w = live_world();
        for _ in 0..MAX_ENEMIES - 1 {
            w.spawn_enemy(EnemyKind::Wisp, w.player.pos + Vec2::new(800.0, 0.0));
        }
        let j = add_enemy(&mut w, EnemyKind::SwarmMother, 300.0);
        w.enemies.hp[j] = 0.0;
        w.step(Input::default());
        w.step(Input::default());
        assert!(w.enemies.len() <= MAX_ENEMIES);
    }

    #[test]
    fn tricks_off_in_stress() {
        let mut w = quiet_world();
        let a = add_enemy(&mut w, EnemyKind::ClayBird, 300.0);
        let b = add_enemy(&mut w, EnemyKind::SwarmMother, 320.0);
        w.enemies.hp[a] = 0.0;
        w.enemies.hp[b] = 0.0;
        w.remove_dead_enemies();
        assert!(w.hazards.is_empty() && w.pending_spawns.is_empty());
    }

    fn thief_world(carry: Option<PickupKind>) -> (World, usize) {
        let mut w = live_world();
        let j = add_enemy(&mut w, EnemyKind::ToolThief, 300.0);
        w.enemies.carry[j] = carry;
        w.enemies.speed[j] = 40.0;
        (w, j)
    }

    fn steal_step(w: &mut World) {
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        w.update_pickups();
        w.move_enemies();
    }

    #[test]
    fn thief_steals_pickup_and_flees() {
        let (mut w, j) = thief_world(None);
        let at = w.enemies.pos[j] + Vec2::new(w.enemies.radius[j] + THIEF_REACH - 1.0, 0.0);
        w.pickups.push((at, PickupKind::OniSake));
        steal_step(&mut w);
        assert!(w.pickups.is_empty());
        assert_eq!(w.enemies.carry[j], Some(PickupKind::OniSake));
        assert!(w.events.iter().any(|e| matches!(*e, Event::PickupStolen { pos, kind: PickupKind::OniSake } if pos == at)));
        assert!((w.enemies.flee[j] - (THIEF_FLEE - DT)).abs() < 1e-4);
        let (v, away) = (w.enemies.vel[j], w.enemies.pos[j] - w.player.pos);
        assert!(v.x * away.x + v.y * away.y > 0.0, "runs from the player");
    }

    #[test]
    fn thief_steals_only_while_visible_and_free() {
        let holds: [fn(&mut World, usize); 4] = [
            |w, j| w.enemies.ai_state[j] = 1,
            |w, j| w.enemies.ai_state[j] = 2,
            |w, _| w.freeze = 1.0,
            |w, j| w.enemies.stagger[j] = 1.0,
        ];
        for (n, hold) in holds.iter().enumerate() {
            let (mut w, j) = thief_world(None);
            hold(&mut w, j);
            w.pickups.push((w.enemies.pos[j], PickupKind::OniSake));
            steal_step(&mut w);
            assert_eq!((w.pickups.len(), w.enemies.carry[j]), (1, None), "hold {n}");
        }
        let mut w = World::new(WorldConfig { hero: HeroId::Kuon, ..Default::default() });
        w.weapons.clear();
        w.enemies = Enemies::default();
        let j = add_enemy(&mut w, EnemyKind::ToolThief, 60.0);
        w.player.form = 1.0;
        w.domain_center = w.player.pos;
        w.pickups.push((w.enemies.pos[j], PickupKind::OniSake));
        steal_step(&mut w);
        assert_eq!((w.pickups.len(), w.enemies.carry[j]), (1, None), "inside Firmament");
    }

    #[test]
    fn thief_carries_one_at_a_time() {
        let (mut w, j) = thief_world(None);
        let at = w.enemies.pos[j];
        w.pickups.push((at, PickupKind::Feast));
        w.pickups.push((at, PickupKind::SenPurse));
        steal_step(&mut w);
        assert_eq!(w.pickups.len(), 1);
        assert!(w.enemies.carry[j].is_some());
    }

    #[test]
    fn carried_pickup_frees_slot() {
        let (mut w, j) = thief_world(None);
        let at = w.enemies.pos[j];
        w.pickups = vec![(at, PickupKind::Feast); MAX_PICKUPS];
        let next = w.next_pickup;
        steal_step(&mut w);
        assert_eq!(w.pickups.len(), MAX_PICKUPS - 1);
        assert_eq!(w.next_pickup, next);
        assert!(w.enemies.carry[j].is_some());
    }

    #[test]
    fn thief_resumes_blink_after_flee() {
        let (mut w, j) = thief_world(None);
        let at = w.enemies.pos[j];
        w.pickups.push((at, PickupKind::Hourglass));
        steal_step(&mut w);
        for _ in 0..(THIEF_FLEE / DT) as usize + 1 {
            w.move_enemies();
        }
        assert_eq!(w.enemies.flee[j], 0.0);
        assert_eq!(w.enemies.carry[j], Some(PickupKind::Hourglass));
        assert_eq!(w.enemies.ai_state[j], 0);
        for _ in 0..(5.0 / DT) as usize {
            w.move_enemies();
        }
        assert_ne!(w.enemies.ai_state[j], 0, "blink cycle runs again");
    }

    #[test]
    fn thief_drops_pickup_on_death() {
        let (mut w, j) = thief_world(Some(PickupKind::ShrineBell));
        let pos = w.enemies.pos[j];
        kill_step(&mut w, j);
        assert_eq!(w.pickups, vec![(pos, PickupKind::ShrineBell)]);
        assert!(w.events.iter().any(|e| matches!(e, Event::PickupSpawned { kind: PickupKind::ShrineBell, .. })));
    }

    #[test]
    fn thief_drop_ignores_pickup_cap() {
        let (mut w, j) = thief_world(Some(PickupKind::Feast));
        w.pickups = vec![(Vec2::new(5000.0, 0.0), PickupKind::Feast); MAX_PICKUPS];
        kill_step(&mut w, j);
        assert_eq!(w.pickups.len(), MAX_PICKUPS + 1);
    }

    #[test]
    fn recycled_thief_keeps_pickup() {
        let (mut w, j) = thief_world(Some(PickupKind::OniSake));
        w.enemies.pos[j] = w.player.pos + Vec2::new(w.cfg.recycle_radius + 50.0, 0.0);
        w.enemies.flee[j] = 3.0;
        w.step(Input::default());
        let d = (w.enemies.pos[j] - w.player.pos).length();
        assert!((d - (w.cfg.spawn_radius + 40.0)).abs() < 40.0, "on the spawn ring, got {d}");
        assert_eq!(w.enemies.carry[j], Some(PickupKind::OniSake));
        assert!(w.enemies.flee[j] < 3.0 && w.enemies.flee[j] > 2.0);
    }

    #[test]
    fn other_enemies_never_steal() {
        let mut w = live_world();
        let j = add_enemy(&mut w, EnemyKind::Nopperabo, 300.0);
        let at = w.enemies.pos[j];
        w.pickups.push((at, PickupKind::Feast));
        steal_step(&mut w);
        assert_eq!(w.pickups.len(), 1);
        assert_eq!(w.enemies.carry[j], None);
    }

    #[test]
    fn finale_pushes_stop_at_reach() {
        for hero in [HeroId::Ren, HeroId::Shun] {
            let mut w = World::new(WorldConfig { hero, ..Default::default() });
            w.weapons.clear();
            w.enemies = Enemies::default();
            w.player.special = 1.0;
            w.step(Input { special: true, ..Default::default() });
            w.enemies = Enemies::default();
            let near = add_enemy(&mut w, EnemyKind::Oni, 100.0);
            let far = add_enemy(&mut w, EnemyKind::Oni, 300.0);
            for j in [near, far] {
                w.enemies.hp[j] = 1e9;
                w.enemies.knock[j] = 1.0;
            }
            w.player.form = 0.001;
            w.step(Input::default());
            assert!(w.enemies.push[near].length() > 0.0, "{hero:?}: near enemy not pushed");
            assert_eq!(w.enemies.push[far], Vec2::ZERO, "{hero:?}: far enemy pushed");
        }
    }

    #[test]
    fn player_bursts_stay_in_reach() {
        for hero in [HeroId::Ren, HeroId::Riku, HeroId::Shun, HeroId::Toya, HeroId::Kuon] {
            let mut gear = Loadout::default();
            gear.exotics = vec![Exotic::Kusanagi, Exotic::YataMirror, Exotic::Yasakani, Exotic::JeweledBranch, Exotic::LanternHeart];
            let mut w = World::new(WorldConfig { hero, gear, ..Default::default() });
            w.debug_max_loadout();
            let mut checked = 0;
            for t in 0..60 * 120 {
                if w.offer.is_some() {
                    w.bot_pick();
                }
                w.close_chest();
                w.choose_shrine(None);
                w.player.special = 1.0;
                w.player.hp = w.player.max_hp;
                let before = w.player.pos;
                let a = t as f32 * 0.01;
                w.step(Input { move_dir: Vec2::new(a.cos(), a.sin()), special: true, ..Default::default() });
                let cap = w.reach_half().x + 0.01;
                for e in &w.events {
                    if let Event::Burst { pos, radius, .. } | Event::FinaleBlast { pos, radius } = *e {
                        if (pos - before).length() < 8.0 || (pos - w.player.pos).length() < 8.0 {
                            checked += 1;
                            assert!(radius <= cap, "{hero:?}: burst radius {radius} past reach {cap}");
                        }
                    }
                }
            }
            assert!(checked > 0, "{hero:?}: no player-centred bursts seen");
        }
    }

    #[test]
    fn onset_pushes_stop_at_reach() {
        for hero in [HeroId::Riku, HeroId::Shun, HeroId::Toya] {
            let mut w = World::new(WorldConfig { hero, ..Default::default() });
            w.weapons.clear();
            w.enemies = Enemies::default();
            let reach = w.reach_half().x;
            let uncapped = match hero {
                HeroId::Riku => FOX_BLAST_RADIUS,
                HeroId::Shun => HANNYA_ONSET_RADIUS,
                _ => MUKADE_BURST_RADIUS,
            };
            let near = add_enemy(&mut w, EnemyKind::Oni, 100.0);
            let far = add_enemy(&mut w, EnemyKind::Oni, 0.0);
            w.enemies.pos[far].x = w.player.pos.x + reach + w.enemies.radius[far] + (uncapped - reach) / 2.0;
            for j in [near, far] {
                w.enemies.hp[j] = 1e9;
                w.enemies.knock[j] = 1.0;
            }
            w.player.special = 1.0;
            w.step(Input { special: true, ..Default::default() });
            assert!(w.enemies.push[near].length() > 0.0, "{hero:?}: near enemy not pushed");
            assert_eq!(w.enemies.push[far], Vec2::ZERO, "{hero:?}: far enemy pushed");
        }
    }

    fn edge_pair(w: &World) -> (Vec2, Vec2) {
        let half = w.reach_half();
        let d = half.y + 40.0;
        assert!(d < half.x, "inside control must sit in the box");
        (Vec2::new(d, 0.0), Vec2::new(0.0, d))
    }

    fn put_at(w: &mut World, offset: Vec2) -> usize {
        let id = place(w, offset);
        index_of(w, id)
    }

    #[test]
    fn well_pull_stops_at_reach() {
        let mut w = ren_with(WeaponKind::GravityWell, 1);
        let (inside, outside) = edge_pair(&w);
        let (i, o) = (put_at(&mut w, inside), put_at(&mut w, outside));
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        let pos = w.player.pos + (inside + outside) * 0.5;
        w.wells.push(Well { pos, life: 1.0, max_life: 1.0, radius: 170.0, damage: 1.0, tick: 1.0, elem: None });
        w.update_spirit_arts();
        assert!(w.enemies.push[i].length() > 0.0);
        assert_eq!(w.enemies.push[o], Vec2::ZERO);
    }

    #[test]
    fn lantern_slow_stops_at_reach() {
        let mut w = ren_with(WeaponKind::LanternVigil, 1);
        let (inside, outside) = edge_pair(&w);
        let (i, o) = (put_at(&mut w, inside), put_at(&mut w, outside));
        w.enemies.vel[i] = Vec2::new(10.0, 0.0);
        w.enemies.vel[o] = Vec2::new(10.0, 0.0);
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        let pos = w.player.pos + (inside + outside) * 0.5;
        w.lanterns.push(Lantern { pos, life: 5.0, max_life: 5.0, radius: 170.0, tick: 0.0, damage: 1.0, slows: true, elem: None });
        w.update_spirit_arts();
        assert!(w.enemies.push[i].length() > 0.0);
        assert_eq!(w.enemies.push[o], Vec2::ZERO);
    }

    #[test]
    fn purify_push_and_clear_stop_at_reach() {
        let mut w = ren_with(WeaponKind::PurificationWave, 1);
        let (inside, outside) = edge_pair(&w);
        let (i, o) = (put_at(&mut w, inside), put_at(&mut w, outside));
        w.grid.rebuild(w.player.pos, &w.enemies.pos);
        let p = w.player.pos;
        let shot = |off: Vec2| Shot { pos: p + off, vel: Vec2::ZERO, life: 5.0, style: 0, elem: None };
        w.shots = vec![shot(inside * -1.0), shot(outside * -1.0)];
        w.waves.push(Wave { center: p, radius: inside.length(), max_radius: 500.0, speed: 0.0, damage: 1.0, purify: true, elem: None });
        w.update_effects();
        assert!(w.enemies.push[i].length() > 0.0);
        assert_eq!(w.enemies.push[o], Vec2::ZERO);
        assert_eq!(w.shots.len(), 1);
        assert_eq!(w.shots[0].pos, p + outside * -1.0);
    }

    #[test]
    fn tether_drag_stops_at_reach() {
        let mut w = ren_with(WeaponKind::SoulTether, 1);
        let (inside, outside) = edge_pair(&w);
        let (i, o) = (put_at(&mut w, inside), put_at(&mut w, outside));
        for j in [i, o] {
            let enemy = w.enemies.id[j];
            w.tethers.push(Tether { enemy, life: 5.0, tick: 1.0, damage: 1.0, reach: 1000.0, heals: false, elem: None });
        }
        w.update_spirit_arts();
        assert!(w.enemies.push[i].length() > 0.0);
        assert_eq!(w.enemies.push[o], Vec2::ZERO);
        assert_eq!(w.tethers.len(), 1);
        assert_eq!(w.tethers[0].enemy, w.enemies.id[i]);
    }

    #[test]
    fn devour_heal_counts_only_in_reach() {
        let mut w = ren_with(WeaponKind::Devour, 1);
        w.reach_scale = 0.3;
        for _ in 0..10 {
            put_at(&mut w, Vec2::new(0.0, 60.0));
        }
        put_at(&mut w, Vec2::new(5.0, 0.0));
        w.player.hp = 1.0;
        w.mods.healing = 1.0;
        fire_seal(&mut w, 1);
        assert!((w.player.hp - 1.5).abs() < 1e-3, "healed {}", w.player.hp - 1.0);
    }

    #[test]
    fn persistence_applied_once() {
        let near = Vec2::new(50.0, 0.0);
        let lives = |w: &World| -> Vec<f32> {
            let mut v: Vec<f32> = w.projectiles.life.clone();
            v.extend(w.lanterns.iter().map(|l| l.life));
            v.extend(w.tethers.iter().map(|t| t.life));
            v.extend(w.clones.iter().map(|c| c.life));
            v
        };
        for kind in [WeaponKind::LanternVigil, WeaponKind::Familiars, WeaponKind::SoulTether, WeaponKind::FoxMirage, WeaponKind::GaleShuriken] {
            let mut w = ren_with(kind, 1);
            w.passives.push((PassiveKind::Persistence, 5));
            put_at(&mut w, near);
            let want = weapon_stats(kind, 1).duration * w.persistence();
            fire_seal(&mut w, 1);
            let got = lives(&w);
            assert!(!got.is_empty() && got.iter().all(|l| (l - want).abs() < 1e-3), "{kind:?}: {got:?} vs {want}");
        }
        let mut w = ren_with(WeaponKind::GaleShuriken, 1);
        w.weapons[0].awakened = true;
        w.passives.push((PassiveKind::Persistence, 5));
        let want = awakened_stats(WeaponKind::GaleShuriken).duration * w.persistence();
        w.update_signatures();
        let got = lives(&w);
        assert!(got.len() == 10 && got.iter().all(|l| (l - want).abs() < 1e-3), "cyclone: {got:?} vs {want}");
    }
}
