//! Game content as plain data. Will move to RON files once the shapes settle.

use crate::meta::Bonus;

pub const MAX_LEVEL: u8 = 8;
pub const RUN_SECONDS: f32 = 15.0 * 60.0;

// ───────────────────────── Enemies ─────────────────────────

pub use crate::enemies::*;

pub const MAX_ENEMY_RADIUS: f32 = 36.0;

use crate::Vec2;

/// Attacks only land inside this box around the player (shrunk on small screens).
pub const REACH_HALF: Vec2 = Vec2::new(192.0, 108.0);
pub const REACH_RANGED: f32 = 1.0;
pub const REACH_HYBRID: f32 = 0.9;
pub const REACH_MELEE: f32 = 0.8;
/// Reach stays at least this far inside the visible edge.
pub const REACH_EDGE: f32 = 24.0;
pub const DEFAULT_VIEW_HALF: Vec2 = Vec2::new(320.0, 180.0);
/// Hero damage multipliers, set so every hero's three best weapons land near one strength
/// (`skill_strength`): ranged ~3600, hybrid ~3900, melee ~4150 (melee stands in the crowd, so it
/// keeps a small edge). Before this Riku and Toya were ~3x Ren.
pub const MIGHT_MELEE: f32 = 2.04;
pub const MIGHT_TOYA: f32 = 1.75;
/// Effect duration and range gained per Persistence level.
pub const PERSISTENCE_PER_LEVEL: f32 = 0.12;
pub const MIGHT_RIKU: f32 = 1.73;
pub const MIGHT_REN: f32 = 2.07;
pub const MIGHT_KUON: f32 = 1.82;
/// shortcut: bot kites ~140 px so bosses sit outside reach; revisit with human playtests in phase 3.
pub const BOSS_HP: f32 = 2.0;
/// Final bosses take this share of `BOSS_HP` (they must fall before dawn).
pub const FINAL_BOSS_HP: f32 = 0.35;

/// The Night Lord arrives at RUN_SECONDS; if still alive at this time, dawn takes the player.
pub const DAWN_LIMIT: f32 = 17.0 * 60.0;

/// Seconds a Nopperabo takes to fade back in after a blink (harmless meanwhile).
pub const NOPPERABO_FADE_IN: f32 = 0.6;

/// Enemy fireballs.
pub const SHOT_RADIUS: f32 = 5.0;
pub const SHOT_DAMAGE: f32 = 4.0;

/// Share of refill spawns, from round 5, that are an Uncommon.
pub const UNCOMMON_SHARE: f32 = 0.015;

/// A Leader's banner: enemies this close move and hit harder.
pub const BANNER_RADIUS: f32 = 140.0;
pub const BANNER_SPEED: f32 = 1.1;
pub const BANNER_DAMAGE: f32 = 1.1;
/// A Clay Bird's death leaves a blast that hurts only the player.
pub const CLAY_BLAST_RADIUS: f32 = 34.0;
pub const CLAY_BLAST_WARN: f32 = 1.0;
pub const CLAY_BLAST_DAMAGE: f32 = 14.0;
pub const SEAL_STAMP_DELAY: f32 = 1.5;
pub const SEAL_STAMP_RADIUS: f32 = 36.0;
/// A Swarm Mother's death splits into Cursed Spawn around the spot.
pub const SWARM_SPLIT: usize = 3;
pub const SWARM_SPLIT_RADIUS: f32 = 16.0;
/// A Tool Thief takes a pickup within its radius plus this, then flees the player for a while.
pub const THIEF_REACH: f32 = 8.0;
pub const THIEF_FLEE: f32 = 4.0;
/// Seconds the crew runs from a fallen Leader.
pub const SCATTER_TIME: f32 = 2.0;

/// Share of new spawns that are Chochin (from 4:00) and Nopperabo (from 9:00).
pub fn chochin_share(t: f32) -> f32 {
    if t < 240.0 { 0.0 } else { (0.01 + (t - 240.0) / 660.0 * 0.02).min(0.03) }
}

pub fn nopperabo_share(t: f32) -> f32 {
    if t < 540.0 { 0.0 } else { (0.02 + (t - 540.0) / 360.0 * 0.02).min(0.04) }
}

// ───────────────────────── Ren's special ─────────────────────────

/// Kills needed to fill the special meter.
pub const SPECIAL_KILLS: f32 = 120.0;
pub const BARRIER_RADIUS: f32 = 170.0;
/// Auto special fires below this share of max HP, or with this many enemies within the radius.
pub const AUTO_SPECIAL_HP: f32 = 0.4;
/// A crowd: this much `Rank::threat` within `AUTO_SPECIAL_RADIUS`, e.g. 20 fodder,
/// 3 elites and 2 fodder, or any boss.
pub const AUTO_SPECIAL_THREAT: f32 = 20.0;
pub const AUTO_SPECIAL_RADIUS: f32 = 200.0;
pub const BARRIER_DAMAGE: f32 = 60.0;
pub const BARRIER_IFRAMES: f32 = 1.5;

// ───────────────────────── Riku's special: Ninefold Foxfire ─────────────────────────

pub const FOX_DURATION: f32 = 8.0;
/// Opening explosion.
pub const FOX_BLAST_RADIUS: f32 = 230.0;
pub const FOX_BLAST_DAMAGE: f32 = 110.0;
pub const FOX_TAILS: usize = 9;
pub const FOX_TAIL_REACH: (f32, f32) = (55.0, 125.0);
pub const FOX_TAIL_DAMAGE: f32 = 18.0;
/// Seconds between tail / aura damage ticks.
pub const FOX_TICK: f32 = 0.25;
pub const FOX_AURA_RADIUS: f32 = 64.0;
pub const FOX_AURA_DAMAGE: f32 = 9.0;
/// A ring of golden orbs every this many seconds.
pub const FOX_ORB_EVERY: f32 = 1.1;
pub const FOX_ORBS: u32 = 10;
pub const FOX_ORB_DAMAGE: f32 = 32.0;
/// Damage taken while transformed is multiplied by this.
pub const FOX_DAMAGE_TAKEN: f32 = 0.7;
pub const FOX_MOVE_SPEED: f32 = 1.2;

// ───────────────────────── Shun's special: Hannya Unbound ─────────────────────────

pub const HANNYA_DURATION: f32 = 8.0;
pub const HANNYA_ONSET_RADIUS: f32 = 160.0;
pub const HANNYA_ONSET_DAMAGE: f32 = 80.0;
pub const HANNYA_AURA_RADIUS: f32 = 72.0;
pub const HANNYA_AURA_DAMAGE: f32 = 10.0;
/// Rending Beams: this many crimson beams every `HANNYA_BEAM_EVERY` seconds.
pub const HANNYA_BEAMS: u32 = 6;
pub const HANNYA_BEAM_EVERY: f32 = 1.4;
pub const HANNYA_BEAM_LENGTH: f32 = 380.0;
pub const HANNYA_BEAM_DAMAGE: f32 = 40.0;
/// Crescent waves at the nearest enemy.
pub const HANNYA_WAVE_EVERY: f32 = 0.45;
pub const HANNYA_WAVE_DAMAGE: f32 = 26.0;
/// Shadow Step: two afterimages dash into the horde and explode.
pub const HANNYA_STEP_EVERY: f32 = 1.8;
pub const HANNYA_AFTERIMAGE_RADIUS: f32 = 64.0;
pub const HANNYA_AFTERIMAGE_DAMAGE: f32 = 45.0;
/// Hannya's Wrath: the finale, charged over the last second.
pub const HANNYA_FINALE_CHARGE: f32 = 1.0;
pub const HANNYA_FINALE_RADIUS: f32 = 340.0;
pub const HANNYA_FINALE_DAMAGE: f32 = 220.0;
pub const HANNYA_DAMAGE_TAKEN: f32 = 0.7;
pub const HANNYA_MOVE_SPEED: f32 = 1.15;

// ───────────────────────── Floor pickups ─────────────────────────

/// Rare pickups that appear on the ground (and from Oni elites).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    /// Heals 35% of max HP.
    Feast,
    /// +40% move speed for a while.
    SwiftGeta,
    /// +50% damage for a while.
    OniSake,
    /// Pulls every gem on the map to the player.
    ShrineBell,
    /// Freezes non-boss enemies.
    Hourglass,
    /// Wipes every non-boss enemy on screen.
    ExorcismSeal,
    /// A purse of Sen.
    SenPurse,
}

pub const ALL_PICKUPS: [PickupKind; 7] = [
    PickupKind::Feast,
    PickupKind::SwiftGeta,
    PickupKind::OniSake,
    PickupKind::ShrineBell,
    PickupKind::Hourglass,
    PickupKind::ExorcismSeal,
    PickupKind::SenPurse,
];

impl PickupKind {
    pub fn name(self) -> &'static str {
        match self {
            PickupKind::Feast => "Onigiri Feast",
            PickupKind::SwiftGeta => "Swift Geta",
            PickupKind::OniSake => "Oni Sake",
            PickupKind::ShrineBell => "Shrine Bell",
            PickupKind::Hourglass => "Frozen Hourglass",
            PickupKind::ExorcismSeal => "Exorcism Seal",
            PickupKind::SenPurse => "Sen Purse",
        }
    }
    pub fn describe(self) -> &'static str {
        match self {
            PickupKind::Feast => "Heal 35% HP",
            PickupKind::SwiftGeta => "+40% speed for 12s",
            PickupKind::OniSake => "+50% damage for 12s",
            PickupKind::ShrineBell => "Every gem flies to you",
            PickupKind::Hourglass => "Enemies frozen for 5s",
            PickupKind::ExorcismSeal => "Banishes every enemy on screen",
            PickupKind::SenPurse => "+40 Sen",
        }
    }
    pub fn tier(self) -> Tier {
        match self {
            PickupKind::Feast => Tier::Common,
            PickupKind::SwiftGeta | PickupKind::OniSake | PickupKind::ShrineBell | PickupKind::SenPurse => Tier::Uncommon,
            PickupKind::Hourglass => Tier::Rare,
            PickupKind::ExorcismSeal => Tier::Legendary,
        }
    }
    /// Spawn weight.
    fn weight(self) -> f32 {
        match self {
            PickupKind::Feast => 25.0,
            PickupKind::SwiftGeta | PickupKind::OniSake | PickupKind::ShrineBell => 15.0,
            PickupKind::SenPurse => 12.0,
            PickupKind::Hourglass => 10.0,
            PickupKind::ExorcismSeal => 8.0,
        }
    }
    /// Weighted pick from a uniform roll in 0..1.
    pub fn roll(r: f32) -> PickupKind {
        let total: f32 = ALL_PICKUPS.iter().map(|p| p.weight()).sum();
        let mut x = r * total;
        for p in ALL_PICKUPS {
            if x < p.weight() {
                return p;
            }
            x -= p.weight();
        }
        PickupKind::Feast
    }
}

pub const BUFF_SECONDS: f32 = 12.0;
pub const FREEZE_SECONDS: f32 = 5.0;
pub const VACUUM_SECONDS: f32 = 3.0;
pub const SPEED_BUFF: f32 = 1.4;
pub const DAMAGE_BUFF: f32 = 1.5;
/// Seconds between ground pickups (random within the range); the first comes at 30s.
pub const PICKUP_EVERY: (f32, f32) = (35.0, 55.0);
pub const MAX_PICKUPS: usize = 4;
/// Chance an Oni elite drops a pickup.
pub const ONI_PICKUP_CHANCE: f32 = 0.25;
/// Chance a Leader drops a pickup.
pub const LEADER_PICKUP_CHANCE: f32 = 0.5;
/// Chance an Uncommon drops a gear item.
pub const UNCOMMON_GEAR: f32 = 0.0015;
/// Radius the Exorcism Seal clears (about a screen).
pub const EXORCISM_RADIUS: f32 = 420.0;

// ───────────────────────── Toya's special: Hundred-Leg Rampage ─────────────────────────

pub const MUKADE_DURATION: f32 = 8.0;
/// Thousand-Leg Burst: the opening radial spike explosion.
pub const MUKADE_BURST_RADIUS: f32 = 220.0;
pub const MUKADE_BURST_DAMAGE: f32 = 110.0;
/// Tendril barrage: lashes this often, at this many enemies.
pub const MUKADE_LASH_EVERY: f32 = 0.18;
pub const MUKADE_LASHES: u32 = 3;
pub const MUKADE_LASH_REACH: f32 = 170.0;
pub const MUKADE_LASH_DAMAGE: f32 = 22.0;
/// Centipede eruptions: a ring of spines bursting from the ground.
pub const MUKADE_ERUPT_EVERY: f32 = 1.0;
pub const MUKADE_ERUPT_DAMAGE: f32 = 50.0;
pub const MUKADE_REGEN: f32 = 4.0;
pub const MUKADE_MOVE_SPEED: f32 = 1.3;
/// Weapon cooldowns are multiplied by this while the form lasts.
pub const MUKADE_HASTE: f32 = 0.7;
pub const MUKADE_DAMAGE_TAKEN: f32 = 0.75;

// ───────────────────────── Ren: Barrier and Divine Purification ─────────────────────────

/// Ren's Barrier blocks a fireball, then recharges (faster with levels).
pub fn barrier_recharge(level: u32) -> f32 {
    (4.0 - 0.04 * level as f32).max(1.5)
}
/// Fireballs that do land on Ren hurt this much less.
pub const BARRIER_SHOT_DAMAGE: f32 = 0.75;
pub const DIVINE_DURATION: f32 = 8.0;
/// Ofuda Storm: homing talismans every this many seconds.
pub const DIVINE_OFUDA_EVERY: f32 = 0.08;
pub const DIVINE_OFUDA_DAMAGE: f32 = 14.0;
/// Yin-Yang Cataclysm: giant orbs crash onto enemy groups.
pub const DIVINE_ORB_EVERY: f32 = 1.0;
pub const DIVINE_ORB_DAMAGE: f32 = 70.0;
/// Spirit-seal barrage rings.
pub const DIVINE_RING_EVERY: f32 = 1.4;
pub const DIVINE_RING_DAMAGE: f32 = 30.0;
/// Final Purification: a magic circle charges, then explodes.
pub const DIVINE_FINALE_CHARGE: f32 = 1.2;
pub const DIVINE_FINALE_RADIUS: f32 = 380.0;
pub const DIVINE_FINALE_DAMAGE: f32 = 300.0;

// ───────────────────────── Kuon: Stillness and Endless Firmament ─────────────────────────

/// Stillness: enemies this close to Kuon move at this fraction of their speed.
pub const STILLNESS_RADIUS: f32 = 64.0;
pub const STILLNESS_SLOW: f32 = 0.3;
/// The ward negates one hit, then recharges.
pub const WARD_RECHARGE: f32 = 6.0;
pub const DOMAIN_DURATION: f32 = 8.0;
pub const DOMAIN_RADIUS: f32 = 360.0;
/// Waves of spatial collapse inside the domain.
pub const DOMAIN_WAVE_EVERY: f32 = 0.5;
pub const DOMAIN_WAVE_DAMAGE: f32 = 20.0;
/// Inside the domain Kuon's attacks hit harder and come faster.
pub const DOMAIN_MIGHT: f32 = 1.8;
pub const DOMAIN_HASTE: f32 = 0.6;
/// Void Lance: Maximum, launched as the domain ends.
pub const DOMAIN_FINALE_CHARGE: f32 = 1.2;
pub const DOMAIN_FINALE_RADIUS: f32 = 70.0;
pub const DOMAIN_FINALE_DAMAGE: f32 = 400.0;

/// Enemies within this distance of a Fox Mirage clone chase it instead of the player.
pub const DECOY_RADIUS: f32 = 110.0;
/// Earliest time a chest may Awaken a weapon.
pub const AWAKEN_FROM: f32 = 480.0;

// ───────────────────────── Weapons ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponKind {
    /// Volley of paper talismans at the nearest enemy (Ren's signature).
    Ofuda,
    /// Moon orbs circling the player.
    MoonOrbs,
    /// Sakura aura damaging everything around the player.
    SakuraField,
    /// A bolt that leaps from enemy to enemy.
    ThunderChain,
    /// Drum beats that send an expanding shockwave ring.
    Taiko,
    /// Delayed strikes that crash down on enemies near the player.
    StarFall,
    /// A melee arc in front of the player.
    Crescent,
    /// Thrown flasks that leave burning ground.
    FlamePool,
    /// Foxfire gathers in the hand, then launches and bursts (Riku's signature, charge type).
    FoxfireOrb,
    /// Fox spirits that hunt the nearest enemies.
    Familiars,
    /// A big spinning shuriken that flies out and comes back.
    GaleShuriken,
    /// Illusory copies of Riku that fight, draw enemies and pop in damaging smoke.
    FoxMirage,
    /// A huge sweeping cut in front that also looses a crescent wave (Shun's signature).
    GraveCleaver,
    /// Spectral chains latch onto enemies, drain them and drag them in.
    SoulTether,
    /// Soul-lanterns placed on the ground that pulse damage around them.
    LanternVigil,
    /// A lightning quick-draw that cuts a long straight line.
    IaiFlash,
    /// Crimson tendrils lash through the nearest enemies (Toya's signature).
    CrimsonTendrils,
    /// Segmented spines erupt from the ground under enemies.
    MukadeSpines,
    /// A hungry pulse that drains enemies around you and heals you.
    Devour,
    /// Venomous fangs that home in and burst.
    VenomFang,
    /// A gravity singularity that pulls a cluster together, then crushes it (Kuon's signature).
    GravityWell,
    /// A crimson star that blasts enemies back along its path.
    RepulsionStar,
    /// A slow purple orb that erases everything in its line.
    VoidLance,
    /// Two yin-yang orbs that orbit Ren, swinging wide and back (Ren).
    YinYangOrbs,
    /// Spirit orbs gather around Ren, then home in and burst (Ren).
    SpiritSeal,
    /// A wide purifying ring that blasts enemies back and erases fireballs (Ren).
    PurificationWave,
}

pub const ALL_WEAPONS: [WeaponKind; 26] = [
    WeaponKind::Ofuda,
    WeaponKind::MoonOrbs,
    WeaponKind::SakuraField,
    WeaponKind::ThunderChain,
    WeaponKind::Taiko,
    WeaponKind::StarFall,
    WeaponKind::Crescent,
    WeaponKind::FlamePool,
    WeaponKind::FoxfireOrb,
    WeaponKind::Familiars,
    WeaponKind::GaleShuriken,
    WeaponKind::FoxMirage,
    WeaponKind::GraveCleaver,
    WeaponKind::SoulTether,
    WeaponKind::LanternVigil,
    WeaponKind::IaiFlash,
    WeaponKind::CrimsonTendrils,
    WeaponKind::MukadeSpines,
    WeaponKind::Devour,
    WeaponKind::VenomFang,
    WeaponKind::GravityWell,
    WeaponKind::RepulsionStar,
    WeaponKind::VoidLance,
    WeaponKind::YinYangOrbs,
    WeaponKind::SpiritSeal,
    WeaponKind::PurificationWave,
];

#[derive(Clone, Copy, Debug)]
pub struct WeaponStats {
    pub damage: f32,
    pub cooldown: f32,
    /// Projectiles, orbs, chain jumps, waves, strikes, arcs or flasks.
    pub amount: u32,
    pub area: f32,
    pub speed: f32,
    pub pierce: u8,
    /// Lifetime of lingering effects in seconds (Flame Pool).
    pub duration: f32,
}

const fn at(l: u32, n: u32) -> u32 {
    (l >= n) as u32
}

/// Base stats at `level` (1..=8) before passives.
pub fn weapon_stats(kind: WeaponKind, level: u8) -> WeaponStats {
    let l = level.clamp(1, MAX_LEVEL) as u32;
    let k = l as f32 - 1.0; // levels gained
    let base = WeaponStats { damage: 0.0, cooldown: 1.0, amount: 1, area: 1.0, speed: 0.0, pierce: 0, duration: 0.0 };
    match kind {
        WeaponKind::Ofuda => WeaponStats {
            damage: 4.93 + 1.36 * k,
            cooldown: 1.0 - 0.05 * k,
            amount: 2 + at(l, 3) + at(l, 5) + at(l, 7) + at(l, 8),
            speed: 340.0,
            pierce: 1 + at(l, 4) as u8 + at(l, 6) as u8 + at(l, 8) as u8,
            ..base
        },
        WeaponKind::MoonOrbs => WeaponStats {
            damage: 13.0 + 3.24 * k,
            cooldown: 0.6, // re-hit interval on the same enemy
            amount: 1 + at(l, 2) + at(l, 4) + at(l, 6) + at(l, 8),
            area: 1.0 + 0.08 * k,
            speed: 3.2 + 0.15 * k, // radians / second
            ..base
        },
        WeaponKind::SakuraField => WeaponStats { damage: 4.8 + 1.8 * k, cooldown: 0.5, area: 1.0 + 0.12 * k, ..base },
        WeaponKind::ThunderChain => WeaponStats {
            damage: 41.0 + 10.3 * k,
            cooldown: 1.6 - 0.06 * k,
            amount: 3 + l / 2, // jumps
            area: 1.0 + 0.05 * k, // jump range
            ..base
        },
        WeaponKind::Taiko => WeaponStats {
            damage: 15.6 + 4.56 * k,
            cooldown: 3.0 - 0.1 * k,
            amount: 1 + at(l, 5),
            area: 1.0 + 0.10 * k,
            speed: 260.0, // ring expansion px/s
            ..base
        },
        WeaponKind::StarFall => WeaponStats {
            damage: 14.0 + 3.5 * k,
            cooldown: 1.6 - 0.05 * k,
            amount: 2 + at(l, 3) + at(l, 5) + at(l, 7),
            area: 1.0 + 0.07 * k,
            ..base
        },
        WeaponKind::Crescent => WeaponStats {
            damage: 14.0 + 3.5 * k,
            cooldown: 1.1 - 0.04 * k,
            amount: 1 + at(l, 4) + at(l, 7),
            area: 1.0 + 0.08 * k,
            ..base
        },
        WeaponKind::FlamePool => WeaponStats {
            damage: 2.8 + 0.84 * k, // per 0.3s tick
            cooldown: 3.2 - 0.12 * k,
            amount: 1 + at(l, 3) + at(l, 6),
            area: 1.0 + 0.08 * k,
            duration: 2.5 + 0.15 * k,
            ..base
        },
        WeaponKind::FoxfireOrb => WeaponStats {
            damage: 10.9 + 2.87 * k, // burst damage
            cooldown: 1.6 - 0.06 * k,
            amount: 1 + at(l, 4) + at(l, 7),
            area: 1.0 + 0.10 * k, // burst radius
            speed: 260.0,
            duration: 0.55, // charge time
            ..base
        },
        WeaponKind::Familiars => WeaponStats {
            damage: 14.0 + 3.5 * k,
            cooldown: 2.2 - 0.08 * k,
            amount: 2 + at(l, 3) + at(l, 5) + at(l, 7),
            speed: 170.0,
            pierce: 3 + at(l, 4) as u8 + at(l, 6) as u8 + at(l, 8) as u8,
            duration: 2.6, // lifetime
            ..base
        },
        WeaponKind::GaleShuriken => WeaponStats {
            damage: 21.0 + 5.2 * k,
            cooldown: 2.0 - 0.07 * k,
            amount: 1 + at(l, 3) + at(l, 6),
            area: 1.0 + 0.08 * k,
            speed: 300.0,
            duration: 1.4, // out-and-back time
            ..base
        },
        WeaponKind::FoxMirage => WeaponStats {
            damage: 7.1 + 1.96 * k, // per strike; the smoke pop deals 2x
            cooldown: 6.5 - 0.3 * k,
            amount: 1 + at(l, 3) + at(l, 5) + at(l, 7),
            area: 1.0 + 0.08 * k, // pop radius
            speed: 165.0,
            pierce: 3 + at(l, 4) as u8 + at(l, 8) as u8, // hits a clone can take
            duration: 4.0 + 0.3 * k,
        },
        WeaponKind::GraveCleaver => WeaponStats {
            damage: 8.6 + 2.13 * k,
            cooldown: 1.3 - 0.04 * k,
            amount: 1 + at(l, 4) + at(l, 7), // swings (alternating sides)
            area: 1.0 + 0.08 * k,
            speed: 320.0, // crescent wave speed
            ..base
        },
        WeaponKind::SoulTether => WeaponStats {
            damage: 15.2 + 4.6 * k, // per 0.25s tick
            cooldown: 3.0 - 0.1 * k,
            amount: 2 + at(l, 3) + at(l, 5) + at(l, 7), // chains
            area: 1.0 + 0.06 * k, // reach
            duration: 2.0 + 0.1 * k,
            ..base
        },
        WeaponKind::LanternVigil => WeaponStats {
            damage: 6.6 + 1.95 * k, // per pulse
            cooldown: 4.0 - 0.15 * k,
            amount: 1 + at(l, 3) + at(l, 6),
            area: 1.0 + 0.10 * k,
            duration: 4.0 + 0.3 * k,
            ..base
        },
        WeaponKind::CrimsonTendrils => WeaponStats {
            damage: 11.70 + 3.15 * k,
            cooldown: 0.9 - 0.03 * k,
            amount: 4 + at(l, 4) + at(l, 7), // tendrils (targets)
            area: 1.0 + 0.07 * k, // reach
            ..base
        },
        WeaponKind::MukadeSpines => WeaponStats {
            damage: 11.54 + 3.21 * k,
            cooldown: 1.7 - 0.06 * k,
            amount: 3 + at(l, 3) + at(l, 5) + at(l, 7),
            area: 1.0 + 0.08 * k,
            ..base
        },
        WeaponKind::Devour => WeaponStats {
            damage: 11.0 + 3.4 * k,
            cooldown: 1.5 - 0.05 * k,
            amount: 1,
            area: 1.0 + 0.10 * k,
            ..base
        },
        WeaponKind::VenomFang => WeaponStats {
            damage: 4.00 + 1.19 * k,
            cooldown: 1.2 - 0.04 * k,
            amount: 2 + at(l, 3) + at(l, 6) + at(l, 8),
            area: 1.0 + 0.06 * k,
            speed: 260.0,
            ..base
        },
        WeaponKind::GravityWell => WeaponStats {
            damage: 4.6 + 1.22 * k, // collapse damage (the pull ticks for a quarter of this)
            cooldown: 2.2 - 0.08 * k,
            amount: 1 + at(l, 4) + at(l, 7),
            area: 1.0 + 0.09 * k,
            duration: 1.0,
            ..base
        },
        WeaponKind::RepulsionStar => WeaponStats {
            damage: 18.2 + 5.2 * k,
            cooldown: 1.3 - 0.05 * k,
            amount: 1 + at(l, 3) + at(l, 6),
            area: 1.0 + 0.06 * k,
            speed: 330.0,
            ..base
        },
        WeaponKind::VoidLance => WeaponStats {
            damage: 21.15 + 5.82 * k,
            cooldown: 3.4 - 0.12 * k,
            amount: 1 + at(l, 5),
            area: 1.0 + 0.08 * k,
            speed: 150.0,
            ..base
        },
        WeaponKind::YinYangOrbs => WeaponStats {
            damage: 19.8 + 5.04 * k,
            cooldown: 0.4, // re-hit interval
            amount: 2 + at(l, 4) + at(l, 7),
            area: 1.0 + 0.07 * k,
            speed: 3.0 + 0.12 * k, // throw cycle radians / second
            ..base
        },
        WeaponKind::SpiritSeal => WeaponStats {
            damage: 14.1 + 3.46 * k, // per seal burst
            cooldown: 2.6 - 0.09 * k,
            amount: 5 + at(l, 3) + at(l, 5) + at(l, 7),
            area: 1.0 + 0.07 * k,
            speed: 300.0,
            duration: SEAL_STAMP_DELAY, // stamp delay
            ..base
        },
        WeaponKind::PurificationWave => WeaponStats {
            damage: 16.8 + 4.2 * k,
            cooldown: 3.2 - 0.12 * k,
            amount: 1,
            area: 1.0 + 0.09 * k,
            speed: 300.0,
            ..base
        },
        WeaponKind::IaiFlash => WeaponStats {
            damage: 30.0 + 8.0 * k,
            cooldown: 2.2 - 0.08 * k,
            amount: 1 + at(l, 4) + at(l, 7), // extra cuts fan out
            area: 1.0 + 0.08 * k, // length
            ..base
        },
    }
}

// ───────────────────────── Weapon library ─────────────────────────

/// Theme tags. Heroes choose their weapon pool by tag; `Shared` weapons fit any hero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tag {
    Talisman,
    Celestial,
    Nature,
    Festival,
    Fire,
    Thunder,
    Blade,
    /// Riku's fox-spirit arts.
    Fox,
    /// Shun's grave-spirit arts.
    Spirit,
    /// Toya's Omukade curse.
    Curse,
    /// Kuon's star-reading arts.
    Star,
    /// Ren's shrine arts.
    Shrine,
    Shared,
}

/// Library entry for a weapon. Behaviour lives in code (one per `WeaponKind`);
/// everything a hero roster needs to know is here.
pub struct WeaponDef {
    pub tags: &'static [Tag],
    /// Passive (and the level of it) required, with the weapon at max level, to Awaken.
    pub complement: (PassiveKind, u8),
}

pub fn weapon_def(kind: WeaponKind) -> WeaponDef {
    use PassiveKind as P;
    use Tag::*;
    let (tags, complement): (&'static [Tag], (PassiveKind, u8)) = match kind {
        WeaponKind::Ofuda => (&[Talisman], (P::Haste, 1)),
        WeaponKind::MoonOrbs => (&[Celestial], (P::Expanse, 1)),
        WeaponKind::SakuraField => (&[Nature], (P::Vitality, 1)),
        WeaponKind::StarFall => (&[Celestial], (P::Abundance, 1)),
        WeaponKind::Taiko => (&[Festival], (P::Guard, 1)),
        WeaponKind::FlamePool => (&[Fire, Festival], (P::Persistence, 1)),
        // Built for future heroes (thunder / blade users); not in Ren's pool.
        WeaponKind::ThunderChain => (&[Thunder], (P::Might, 1)),
        WeaponKind::Crescent => (&[Blade], (P::Swiftness, 1)),
        WeaponKind::FoxfireOrb => (&[Fox], (P::Magnet, 1)),
        WeaponKind::Familiars => (&[Fox], (P::Recovery, 1)),
        WeaponKind::GaleShuriken => (&[Fox], (P::Haste, 1)),
        WeaponKind::FoxMirage => (&[Fox], (P::Persistence, 1)),
        WeaponKind::GraveCleaver => (&[Spirit], (P::Might, 1)),
        WeaponKind::SoulTether => (&[Spirit], (P::Vitality, 1)),
        WeaponKind::LanternVigil => (&[Spirit], (P::Expanse, 1)),
        WeaponKind::IaiFlash => (&[Spirit], (P::Swiftness, 1)),
        WeaponKind::CrimsonTendrils => (&[Curse], (P::Might, 1)),
        WeaponKind::MukadeSpines => (&[Curse], (P::Expanse, 1)),
        WeaponKind::Devour => (&[Curse], (P::Vitality, 1)),
        WeaponKind::VenomFang => (&[Curse], (P::Haste, 1)),
        WeaponKind::GravityWell => (&[Star], (P::Expanse, 1)),
        WeaponKind::RepulsionStar => (&[Star], (P::Might, 1)),
        WeaponKind::VoidLance => (&[Star], (P::Persistence, 1)),
        WeaponKind::YinYangOrbs => (&[Shrine], (P::Swiftness, 1)),
        WeaponKind::SpiritSeal => (&[Shrine], (P::Abundance, 1)),
        WeaponKind::PurificationWave => (&[Shrine], (P::Guard, 1)),
    };
    WeaponDef { tags, complement }
}

/// Complement passive for `kind`: (passive, required level).
pub fn complement(kind: WeaponKind) -> (PassiveKind, u8) {
    weapon_def(kind).complement
}

// ───────────────────────── Heroes ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeroId {
    Ren,
    Kaede,
    Mika,
    Haru,
    Nanami,
    Yuzu,
    Riku,
    Shun,
    Toya,
    Kuon,
}

pub const ALL_HEROES: [HeroId; 10] = [HeroId::Ren, HeroId::Riku, HeroId::Shun, HeroId::Toya, HeroId::Kuon, HeroId::Kaede, HeroId::Mika, HeroId::Haru, HeroId::Nanami, HeroId::Yuzu];

/// What Space does when the meter is full.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialKind {
    /// Purifying Barrier: a ring that damages, knocks back and erases fireballs.
    Barrier,
    /// Ninefold Foxfire: a timed transformation (tails, aura, golden orb volleys).
    NineTails,
    /// Hannya Unbound: a timed berserk form (beams, crescent waves, shadow steps, finale blast).
    Hannya,
    /// Hundred-Leg Rampage: a timed predator form (tendril barrage, eruptions, regeneration).
    Mukade,
    /// Domain: Endless Firmament: enemies in the domain freeze and take damage; ends in a giant lance.
    Firmament,
    /// Divine Purification: invulnerable shrine-maiden barrage ending in a great purifying blast.
    Divine,
}

impl SpecialKind {
    /// Length of a transformation special (0 for instant ones).
    pub fn duration(self) -> f32 {
        match self {
            SpecialKind::Barrier => 0.0,
            SpecialKind::NineTails => FOX_DURATION,
            SpecialKind::Hannya => HANNYA_DURATION,
            SpecialKind::Mukade => MUKADE_DURATION,
            SpecialKind::Firmament => DOMAIN_DURATION,
            SpecialKind::Divine => DIVINE_DURATION,
        }
    }
}

impl HeroId {
    /// Save-file key (stable; never rename).
    pub fn key(self) -> &'static str {
        match self {
            HeroId::Ren => "ren",
            HeroId::Kaede => "kaede",
            HeroId::Mika => "mika",
            HeroId::Haru => "haru",
            HeroId::Nanami => "nanami",
            HeroId::Yuzu => "yuzu",
            HeroId::Riku => "riku",
            HeroId::Shun => "shun",
            HeroId::Toya => "toya",
            HeroId::Kuon => "kuon",
        }
    }
    pub fn index(self) -> usize {
        ALL_HEROES.iter().position(|h| *h == self).unwrap()
    }
    pub fn from_key(k: &str) -> Option<HeroId> {
        ALL_HEROES.iter().copied().find(|h| h.key() == k)
    }
}

pub struct HeroDef {
    pub id: HeroId,
    pub name: &'static str,
    /// Story-world archetype, shown on character select.
    pub archetype: &'static str,
    /// Innate passive, as shown to the player.
    pub innate_text: &'static str,
    pub innate: Bonus,
    /// Starting weapon; always available to this hero.
    pub signature: WeaponKind,
    /// Weapons with any of these tags (plus `Shared`) can be offered.
    pub pool_tags: &'static [Tag],
    /// Scales `REACH_HALF`.
    pub reach: f32,
    /// Per-hero damage lever; melee heroes offset their shorter reach.
    pub reach_might: f32,
    pub weapon_slots: usize,
    pub passive_slots: usize,
    /// False while the hero's own sprite and special move are still placeholders.
    pub finished: bool,
    pub special: SpecialKind,
    pub special_name: &'static str,
    pub element: Element,
}

pub fn hero_def(id: HeroId) -> HeroDef {
    use Tag::*;
    let b = Bonus::default();
    let (name, archetype, innate_text, innate, signature, pool_tags, finished): (_, _, _, _, _, &'static [Tag], _) = match id {
        HeroId::Ren => ("Ren Amane", "Shrine maiden", "Barrier: blocks a fireball every few seconds; +10% pickup radius", Bonus { magnet: 0.10, ..b }, WeaponKind::Ofuda, &[Shrine, Talisman, Celestial, Nature, Festival], true),
        HeroId::Kaede => ("Kaede Kurogane", "Ninja", "+10% move speed", Bonus { move_speed: 0.10, ..b }, WeaponKind::ThunderChain, &[Thunder, Blade, Talisman, Fire], false),
        HeroId::Mika => ("Mika Hoshizora", "Magical girl", "+8% damage", Bonus { might: 0.08, ..b }, WeaponKind::StarFall, &[Celestial, Nature, Festival, Thunder], false),
        HeroId::Haru => ("Haru Tachibana", "Swordsman", "+1 armor", Bonus { armor: 1.0, ..b }, WeaponKind::Crescent, &[Blade, Nature, Festival, Thunder], false),
        HeroId::Nanami => ("Nanami Kiri", "Alchemist", "+10% area", Bonus { area: 0.10, ..b }, WeaponKind::FlamePool, &[Fire, Festival, Nature, Celestial], false),
        HeroId::Yuzu => ("Yuzu Momose", "Idol", "+10% experience", Bonus { growth: 0.10, ..b }, WeaponKind::Taiko, &[Festival, Celestial, Nature, Talisman], false),
        HeroId::Riku => ("Riku Kazamine", "Fox-spirit shinobi", "+5% move speed, +5% damage", Bonus { move_speed: 0.05, might: 0.05, ..b }, WeaponKind::FoxfireOrb, &[Fox, Thunder, Blade, Fire], true),
        HeroId::Kuon => ("Kuon Hoshimi", "Star-reading onmyoji", "Stillness: nearby enemies crawl; a ward blocks a hit every 6s", b, WeaponKind::GravityWell, &[Star, Celestial, Talisman, Thunder], true),
        HeroId::Toya => ("Toya Hyakuashi", "Cursed youth", "Regenerate 0.3 HP/s, +5% move speed", Bonus { recovery: 0.3, move_speed: 0.05, ..b }, WeaponKind::CrimsonTendrils, &[Curse, Blade, Fire, Thunder], true),
        HeroId::Shun => ("Shun Yamikiri", "Spirit ferryman", "+10% area", Bonus { area: 0.10, ..b }, WeaponKind::GraveCleaver, &[Spirit, Blade, Thunder, Fire], true),
    };
    let (special, special_name) = match id {
        HeroId::Ren => (SpecialKind::Divine, "Divine Purification"),
        HeroId::Riku => (SpecialKind::NineTails, "Ninefold Foxfire"),
        HeroId::Shun => (SpecialKind::Hannya, "Hannya Unbound"),
        HeroId::Toya => (SpecialKind::Mukade, "Hundred-Leg Rampage"),
        HeroId::Kuon => (SpecialKind::Firmament, "Domain: Endless Firmament"),
        _ => (SpecialKind::Barrier, "Barrier"),
    };
    let (reach, reach_might) = match id {
        HeroId::Ren => (REACH_RANGED, MIGHT_REN),
        HeroId::Kuon => (REACH_RANGED, MIGHT_KUON),
        HeroId::Mika | HeroId::Nanami => (REACH_RANGED, 1.0),
        HeroId::Riku => (REACH_HYBRID, MIGHT_RIKU),
        HeroId::Kaede | HeroId::Yuzu => (REACH_HYBRID, 1.0),
        HeroId::Toya => (REACH_MELEE, MIGHT_TOYA),
        HeroId::Shun | HeroId::Haru => (REACH_MELEE, MIGHT_MELEE),
    };
    let element = match id {
        HeroId::Ren => Element::Light,
        HeroId::Riku | HeroId::Nanami => Element::Fire,
        HeroId::Shun => Element::Darkness,
        HeroId::Toya => Element::Earth,
        HeroId::Kuon => Element::Ice,
        HeroId::Kaede | HeroId::Yuzu => Element::Electric,
        HeroId::Mika => Element::Water,
        HeroId::Haru => Element::Wind,
    };
    HeroDef { id, name, archetype, innate_text, innate, signature, pool_tags, reach, reach_might, weapon_slots: 4, passive_slots: 6, finished, special, special_name, element }
}

impl HeroDef {
    /// Weapons this hero can ever be offered.
    pub fn weapon_pool(&self) -> Vec<WeaponKind> {
        ALL_WEAPONS
            .iter()
            .copied()
            .filter(|k| *k == self.signature || weapon_def(*k).tags.iter().any(|t| *t == Tag::Shared || self.pool_tags.contains(t)))
            .collect()
    }
}

pub fn awakened_name(kind: WeaponKind) -> &'static str {
    match kind {
        WeaponKind::Ofuda => "Hundredfold Seal",
        WeaponKind::MoonOrbs => "Twin Moon Mandala",
        WeaponKind::SakuraField => "Undying Sakura",
        WeaponKind::ThunderChain => "Raiju Storm",
        WeaponKind::Taiko => "Grand Matsuri Drum",
        WeaponKind::StarFall => "Meteor Festival",
        WeaponKind::Crescent => "Full Moon Cut",
        WeaponKind::FlamePool => "Undying Foxfire",
        WeaponKind::FoxfireOrb => "Foxfire Comet",
        WeaponKind::Familiars => "Kitsune Procession",
        WeaponKind::GaleShuriken => "Tempest Wheel",
        WeaponKind::FoxMirage => "Thousand-Fox Mirage",
        WeaponKind::GraveCleaver => "Ferryman's Judgement",
        WeaponKind::SoulTether => "River of Souls",
        WeaponKind::LanternVigil => "Thousand Lantern Vigil",
        WeaponKind::IaiFlash => "Moonsever",
        WeaponKind::CrimsonTendrils => "Hundred Tendrils",
        WeaponKind::MukadeSpines => "Centipede Spine",
        WeaponKind::Devour => "Gluttony",
        WeaponKind::VenomFang => "Venom Rain",
        WeaponKind::GravityWell => "Event Horizon",
        WeaponKind::RepulsionStar => "Red Nova",
        WeaponKind::VoidLance => "Void Cascade",
        WeaponKind::YinYangOrbs => "Great Yin-Yang",
        WeaponKind::SpiritSeal => "Myriad Spirit Seal",
        WeaponKind::PurificationWave => "Heaven-Sweeping Wave",
    }
}

/// Collection text for an Awakening.
pub fn awakened_text(kind: WeaponKind) -> &'static str {
    match kind {
        WeaponKind::Ofuda => "Seal Storm: an endless stream of homing seals that explode on contact, on top of twice the talismans.",
        WeaponKind::MoonOrbs => "Lunar Eclipse: a second ring of moons, and moonbeams sweep your reach every few seconds.",
        WeaponKind::SakuraField => "Hanafubuki: a ceaseless spiral storm of cutting petals pours out of a wider field that heals you.",
        WeaponKind::ThunderChain => "Heavenfall: lightning keeps crashing down from the sky on nearby enemies, plus twice the chains.",
        WeaponKind::Taiko => "Festival Quake: colossal shockwaves roll out across your reach between the beats.",
        WeaponKind::StarFall => "Meteor: giant meteors crash down and leave burning craters, on top of twice the stars.",
        WeaponKind::Crescent => "Moonlit Waves: cuts a full circle and every swing fires eight crescent waves.",
        WeaponKind::FlamePool => "Foxfire Trail: you leave a trail of burning ground wherever you walk; pools burn longer and wider.",
        WeaponKind::FoxfireOrb => "Comet Rain: blue comets keep falling on the horde; orbs charge twice as fast and burst into sparks.",
        WeaponKind::Familiars => "Kitsune Blaze: twice the fox spirits, and every one explodes in foxfire when it strikes.",
        WeaponKind::GaleShuriken => "Cyclone: every few seconds a whole ring of shuriken bursts out and returns.",
        WeaponKind::FoxMirage => "Mirage Barrage: twice the clones, and every clone hurls foxfire at the horde.",
        WeaponKind::GraveCleaver => "Judgement: full-circle cuts, waves both ways, and a colossal crescent of judgement every few seconds.",
        WeaponKind::SoulTether => "Endless River: twice the chains; when a chained enemy dies the chain leaps to the next and heals you.",
        WeaponKind::LanternVigil => "Lantern Volley: more, wider lanterns that slow enemies and fire spirit bolts at them.",
        WeaponKind::IaiFlash => "Thousand Cuts: longer cuts that leave scars, and a radial storm of twelve cuts every few seconds.",
        WeaponKind::CrimsonTendrils => "Hundred Tendrils: more tendrils, and a storm of twelve lashes tears through everything around you.",
        WeaponKind::MukadeSpines => "Centipede Spine: a marching line of spines crawls out across the ground every few seconds.",
        WeaponKind::Devour => "Gluttony: a wider pulse that heals twice as much, and a great feeding wave every few seconds.",
        WeaponKind::VenomFang => "Venom Rain: more fangs, and a ring of venom fangs bursts out every few seconds.",
        WeaponKind::GravityWell => "Event Horizon: a vast singularity swallows the horde every few seconds.",
        WeaponKind::RepulsionStar => "Red Nova: a ring of crimson stars blasts outward every few seconds.",
        WeaponKind::VoidLance => "Void Cascade: three void lances fan out across the field every few seconds.",
        WeaponKind::YinYangOrbs => "Great Yin-Yang: the orbs grow huge, and a giant yin-yang crashes down on the horde every few seconds.",
        WeaponKind::SpiritSeal => "Myriad Spirit Seal: more seals at once, and a full ring of seal orbs bursts out every few seconds.",
        WeaponKind::PurificationWave => "Heaven-Sweeping Wave: a second, wider wave follows every pulse.",
    }
}

/// Stats after Awakening. Some Awakenings also change behaviour (see world.rs).
pub fn awakened_stats(kind: WeaponKind) -> WeaponStats {
    let mut s = weapon_stats(kind, MAX_LEVEL);
    match kind {
        WeaponKind::Ofuda => {
            s.amount *= 2;
            s.pierce += 3;
            s.damage *= 1.2;
        }
        WeaponKind::MoonOrbs => s.damage *= 1.25,
        WeaponKind::SakuraField => {
            s.area *= 1.3;
            s.damage *= 1.2;
        }
        WeaponKind::ThunderChain => {
            s.amount *= 2;
            s.damage *= 1.3;
            s.cooldown *= 0.8;
        }
        WeaponKind::Taiko => {
            s.amount += 1;
            s.area *= 1.3;
            s.damage *= 1.3;
        }
        WeaponKind::StarFall => {
            s.amount *= 2;
            s.damage *= 1.2;
        }
        WeaponKind::Crescent => s.damage *= 1.3, // and cuts a full circle
        WeaponKind::FlamePool => {
            s.duration *= 1.6;
            s.area *= 1.25;
            s.damage *= 1.25;
        }
        WeaponKind::FoxfireOrb => {
            s.duration *= 0.5;
            s.damage *= 1.4;
            s.area *= 1.3;
        }
        WeaponKind::Familiars => {
            s.amount *= 2;
            s.pierce += 3;
            s.damage *= 1.25;
        }
        WeaponKind::GaleShuriken => {
            s.amount += 2;
            s.area *= 1.3;
            s.damage *= 1.25;
        }
        WeaponKind::FoxMirage => {
            s.amount *= 2;
            s.damage *= 1.3;
            s.area *= 1.3;
        }
        WeaponKind::GraveCleaver => s.damage *= 1.3, // and cuts a full circle
        WeaponKind::SoulTether => {
            s.amount *= 2;
            s.damage *= 1.3;
        }
        WeaponKind::LanternVigil => {
            s.amount += 2;
            s.area *= 1.4;
        }
        WeaponKind::IaiFlash => {
            s.amount += 2;
            s.area *= 1.5;
            s.damage *= 1.4;
        }
        WeaponKind::CrimsonTendrils => s.amount += 2,
        WeaponKind::MukadeSpines => s.amount += 2,
        WeaponKind::Devour => s.area *= 1.3,
        WeaponKind::VenomFang => s.amount += 2,
        WeaponKind::GravityWell => s.area *= 1.3,
        WeaponKind::RepulsionStar => s.amount += 1,
        WeaponKind::VoidLance => s.area *= 1.3,
        WeaponKind::YinYangOrbs => s.area *= 1.6,
        WeaponKind::SpiritSeal => s.amount += 3,
        WeaponKind::PurificationWave => s.area *= 1.3,
    }
    // Every Awakening is a real power spike on top of its own changes (and its signature).
    s.damage *= AWAKEN_DAMAGE;
    s
}

/// Extra damage every Awakened weapon gets.
pub const AWAKEN_DAMAGE: f32 = 1.5;

/// Seconds between an Awakened weapon's signature effects.
pub fn signature_every(kind: WeaponKind) -> f32 {
    match kind {
        WeaponKind::Ofuda => 0.12,
        WeaponKind::MoonOrbs => 2.4,
        WeaponKind::SakuraField => 0.07,
        WeaponKind::ThunderChain => 0.35,
        WeaponKind::Taiko => 1.6,
        WeaponKind::StarFall => 1.1,
        WeaponKind::Crescent => 0.0, // part of every swing
        WeaponKind::FlamePool => 0.3,
        WeaponKind::FoxfireOrb => 1.8,
        WeaponKind::Familiars => 0.0, // the spirits themselves explode
        WeaponKind::GaleShuriken => 3.0,
        WeaponKind::FoxMirage => 1.2,
        WeaponKind::GraveCleaver => 2.8,
        WeaponKind::SoulTether => 0.0, // chains jump on kills
        WeaponKind::LanternVigil => 0.5,
        WeaponKind::IaiFlash => 4.0,
        WeaponKind::CrimsonTendrils => 0.8,
        WeaponKind::MukadeSpines => 2.0,
        WeaponKind::Devour => 2.5,
        WeaponKind::VenomFang => 2.2,
        WeaponKind::GravityWell => 4.0,
        WeaponKind::RepulsionStar => 2.6,
        WeaponKind::VoidLance => 3.5,
        WeaponKind::YinYangOrbs => 2.8,
        WeaponKind::SpiritSeal => 3.0,
        WeaponKind::PurificationWave => 0.0, // the double wave is part of every pulse
    }
}

/// Name of an Awakened weapon's signature effect (shown in the Collection and on pickup).
pub fn signature_name(kind: WeaponKind) -> &'static str {
    match kind {
        WeaponKind::Ofuda => "Seal Storm",
        WeaponKind::MoonOrbs => "Lunar Eclipse",
        WeaponKind::SakuraField => "Hanafubuki",
        WeaponKind::ThunderChain => "Heavenfall",
        WeaponKind::Taiko => "Festival Quake",
        WeaponKind::StarFall => "Meteor",
        WeaponKind::Crescent => "Moonlit Waves",
        WeaponKind::FlamePool => "Foxfire Trail",
        WeaponKind::FoxfireOrb => "Comet Rain",
        WeaponKind::Familiars => "Kitsune Blaze",
        WeaponKind::GaleShuriken => "Cyclone",
        WeaponKind::FoxMirage => "Mirage Barrage",
        WeaponKind::GraveCleaver => "Judgement",
        WeaponKind::SoulTether => "Endless River",
        WeaponKind::LanternVigil => "Lantern Volley",
        WeaponKind::IaiFlash => "Thousand Cuts",
        WeaponKind::CrimsonTendrils => "Hundred Lashes",
        WeaponKind::MukadeSpines => "Marching Spine",
        WeaponKind::Devour => "Feeding Wave",
        WeaponKind::VenomFang => "Fang Ring",
        WeaponKind::GravityWell => "Event Horizon",
        WeaponKind::RepulsionStar => "Red Nova",
        WeaponKind::VoidLance => "Void Cascade",
        WeaponKind::YinYangOrbs => "Yin-Yang Crash",
        WeaponKind::SpiritSeal => "Seal Ring",
        WeaponKind::PurificationWave => "Double Wave",
    }
}

// ───────────────────────── Passives ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassiveKind {
    Might,
    Haste,
    Expanse,
    Swiftness,
    Magnet,
    Vitality,
    Abundance,
    Persistence,
    Recovery,
    Guard,
}

pub const ALL_PASSIVES: [PassiveKind; 10] = [
    PassiveKind::Might,
    PassiveKind::Haste,
    PassiveKind::Expanse,
    PassiveKind::Swiftness,
    PassiveKind::Magnet,
    PassiveKind::Vitality,
    PassiveKind::Abundance,
    PassiveKind::Persistence,
    PassiveKind::Recovery,
    PassiveKind::Guard,
];

/// Some passives are strong per level, so they cap lower.
pub fn passive_max(kind: PassiveKind) -> u8 {
    match kind {
        PassiveKind::Abundance => 3,
        PassiveKind::Guard => 5,
        _ => MAX_LEVEL,
    }
}

// ───────────────────────── Elements ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    Fire,
    Water,
    Electric,
    Earth,
    Wind,
    Ice,
    Light,
    Darkness,
}

pub const ALL_ELEMENTS: [Element; 8] = [Element::Fire, Element::Water, Element::Electric, Element::Earth, Element::Wind, Element::Ice, Element::Light, Element::Darkness];

pub const ELEM_STRONG: f32 = 1.5;
pub const ELEM_WEAK: f32 = 0.75;
pub const ELEM_INTO_RARE: f32 = 0.9;
pub const ELEM_FROM_RARE: f32 = 1.15;
pub const ELEM_RARE_CLASH: f32 = 1.5;
/// Damage bonus for a hero's skills of their own element.
pub const AFFINITY_DAMAGE: f32 = 1.1;

pub const PROC_CHANCE: f32 = 0.15;
/// Fire: the burn deals `BURN_SHARE` of the hit, spread over `BURN_SECONDS`, one tick every `BURN_TICK`.
pub const BURN_SECONDS: f32 = 3.0;
pub const BURN_SHARE: f32 = 0.3;
pub const BURN_TICK: f32 = 0.5;
pub const SOAK_SECONDS: f32 = 4.0;
/// Electric damage multiplier on soaked enemies.
pub const SOAK_ELECTRIC: f32 = 1.3;
pub const CHAIN_RANGE: f32 = 90.0;
pub const CHAIN_SHARE: f32 = 0.5;
pub const STAGGER_SECONDS: f32 = 0.4;
/// 6-piece signatures: stagger damage multiplier, Pale Gate shatter share of max HP and radius.
pub const SET6_STAGGER: f32 = 1.3;
pub const SET6_SHATTER: f32 = 0.5;
pub const SET6_SHATTER_R: f32 = 60.0;
pub const KNOCK_EXTRA: f32 = 220.0;
pub const SLOW_SECONDS: f32 = 2.0;
pub const SLOW_FACTOR: f32 = 0.65;
/// Darkness: player HP restored per proc.
pub const DRAIN_HEAL: f32 = 0.5;

impl Element {
    pub fn name(self) -> &'static str {
        match self {
            Element::Fire => "Fire",
            Element::Water => "Water",
            Element::Electric => "Electric",
            Element::Earth => "Earth",
            Element::Wind => "Wind",
            Element::Ice => "Ice",
            Element::Light => "Light",
            Element::Darkness => "Darkness",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Element::Fire => "fire",
            Element::Water => "water",
            Element::Electric => "electric",
            Element::Earth => "earth",
            Element::Wind => "wind",
            Element::Ice => "ice",
            Element::Light => "light",
            Element::Darkness => "darkness",
        }
    }

    pub fn from_key(k: &str) -> Option<Element> {
        ALL_ELEMENTS.iter().copied().find(|e| e.key() == k)
    }

    pub fn is_rare(self) -> bool {
        matches!(self, Element::Light | Element::Darkness)
    }

    /// The element this one is strong against; Light and Darkness sit outside the cycle.
    pub fn beats(self) -> Option<Element> {
        match self {
            Element::Water => Some(Element::Fire),
            Element::Fire => Some(Element::Ice),
            Element::Ice => Some(Element::Wind),
            Element::Wind => Some(Element::Earth),
            Element::Earth => Some(Element::Electric),
            Element::Electric => Some(Element::Water),
            Element::Light | Element::Darkness => None,
        }
    }
}

pub fn element_mult(attacker: Element, target: Element) -> f32 {
    match (attacker.is_rare(), target.is_rare()) {
        (true, true) if attacker != target => ELEM_RARE_CLASH,
        (true, false) => ELEM_FROM_RARE,
        (false, true) => ELEM_INTO_RARE,
        (false, false) if attacker.beats() == Some(target) => ELEM_STRONG,
        (false, false) if target.beats() == Some(attacker) => ELEM_WEAK,
        _ => 1.0,
    }
}

impl WeaponKind {
    pub fn element(self) -> Element {
        use WeaponKind::*;
        match self {
            FlamePool | FoxfireOrb | Familiars | RepulsionStar => Element::Fire,
            PurificationWave | Devour | VenomFang => Element::Water,
            ThunderChain | IaiFlash | Taiko => Element::Electric,
            StarFall | CrimsonTendrils | MukadeSpines => Element::Earth,
            Ofuda | SakuraField | Crescent | GaleShuriken | FoxMirage => Element::Wind,
            MoonOrbs | SoulTether | YinYangOrbs => Element::Ice,
            SpiritSeal | LanternVigil => Element::Light,
            GraveCleaver | GravityWell | VoidLance => Element::Darkness,
        }
    }
}

// ───────────────────────── Upgrades ─────────────────────────

/// Skill rarity; gear has its own `Rarity`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Common,
    Uncommon,
    Rare,
    Legendary,
    Mythic,
    Unique,
}

pub const ALL_TIERS: [Tier; 6] = [Tier::Common, Tier::Uncommon, Tier::Rare, Tier::Legendary, Tier::Mythic, Tier::Unique];

impl Tier {
    pub fn name(self) -> &'static str {
        match self {
            Tier::Common => "Common",
            Tier::Uncommon => "Uncommon",
            Tier::Rare => "Rare",
            Tier::Legendary => "Legendary",
            Tier::Mythic => "Mythic",
            Tier::Unique => "Unique",
        }
    }
    /// Starting level-up offer weight for a skill not yet owned.
    pub fn weight(self) -> f32 {
        match self {
            Tier::Common => 1.0,
            Tier::Uncommon => 0.8,
            Tier::Rare => 0.6,
            Tier::Legendary => 0.4,
            Tier::Mythic => 0.25,
            Tier::Unique => 0.15,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Upgrade {
    Weapon(WeaponKind),
    Passive(PassiveKind),
    /// The three fallbacks once everything is maxed.
    Heal,
    Sen,
    Blessing,
}

/// Chance an offer carries a replacement card when one is possible.
pub const REPLACE_CHANCE: f32 = 0.15;
/// Same, once every owned skill is maxed and only fallback cards remain.
pub const MAXED_REPLACE_CHANCE: f32 = 0.25;

pub const MAXED_FALLBACKS: [Upgrade; 3] = [Upgrade::Heal, Upgrade::Sen, Upgrade::Blessing];

/// Card text for a maxed-out fallback that gives `amount` (see `World::fallback_amount`).
pub fn describe_fallback(u: Upgrade, amount: f32) -> String {
    match u {
        Upgrade::Heal => format!("Restore {amount:.0} HP ({:.0}% of max).", ONIGIRI_HEAL * 100.0),
        Upgrade::Sen => format!("+{amount:.0} Sen."),
        Upgrade::Blessing => format!("+{:.0}% damage for the rest of the run. Adds up.", amount * 100.0),
        _ => u.describe(1),
    }
}

/// Steady healing per second, as a share of max HP (all regeneration / on-kill / on-hit heals
/// combined), and how many seconds of it can bank up.
pub const HEAL_CAP: f32 = 0.03;
pub const HEAL_WINDOW: f32 = 2.0;

/// Share of max HP an Onigiri restores.
pub const ONIGIRI_HEAL: f32 = 0.3;
/// Sen Pouch at the start of a run; it gains this much again every `SEN_POUCH_RAMP` seconds of pace.
pub const SEN_POUCH: f32 = 25.0;
pub const SEN_POUCH_RAMP: f32 = 300.0;
/// Damage per Blessing taken: 1.03 = +3%, added up across Blessings (see `World::blessing_mult`).
pub const BLESSING_DAMAGE: f32 = 1.03;

impl Upgrade {
    pub fn tier(self) -> Tier {
        use WeaponKind::*;
        match self {
            Upgrade::Weapon(SpiritSeal | VenomFang) => Tier::Unique,
            Upgrade::Weapon(GravityWell | LanternVigil) => Tier::Mythic,
            Upgrade::Weapon(GraveCleaver | FoxfireOrb | VoidLance) => Tier::Legendary,
            Upgrade::Weapon(FoxMirage | MukadeSpines | Ofuda | FlamePool | Crescent) => Tier::Rare,
            Upgrade::Weapon(CrimsonTendrils | IaiFlash | YinYangOrbs | StarFall | Familiars | MoonOrbs | GaleShuriken) => Tier::Uncommon,
            Upgrade::Weapon(_) => Tier::Common,
            Upgrade::Passive(PassiveKind::Might) => Tier::Mythic,
            Upgrade::Passive(PassiveKind::Guard) => Tier::Legendary,
            Upgrade::Passive(PassiveKind::Recovery | PassiveKind::Vitality) => Tier::Rare,
            Upgrade::Passive(PassiveKind::Haste | PassiveKind::Abundance | PassiveKind::Swiftness) => Tier::Uncommon,
            Upgrade::Passive(_) | Upgrade::Heal | Upgrade::Sen | Upgrade::Blessing => Tier::Common,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Upgrade::Weapon(WeaponKind::Ofuda) => "Ofuda Volley",
            Upgrade::Weapon(WeaponKind::MoonOrbs) => "Moon Orbs",
            Upgrade::Weapon(WeaponKind::SakuraField) => "Sakura Field",
            Upgrade::Weapon(WeaponKind::ThunderChain) => "Thunder Chain",
            Upgrade::Weapon(WeaponKind::Taiko) => "Taiko Shockwave",
            Upgrade::Weapon(WeaponKind::StarFall) => "Star Fall",
            Upgrade::Weapon(WeaponKind::Crescent) => "Crescent Slash",
            Upgrade::Weapon(WeaponKind::FlamePool) => "Flame Pool",
            Upgrade::Weapon(WeaponKind::FoxfireOrb) => "Foxfire Orb",
            Upgrade::Weapon(WeaponKind::Familiars) => "Kitsune Familiars",
            Upgrade::Weapon(WeaponKind::GaleShuriken) => "Gale Shuriken",
            Upgrade::Weapon(WeaponKind::FoxMirage) => "Fox Mirage",
            Upgrade::Weapon(WeaponKind::GraveCleaver) => "Grave Cleaver",
            Upgrade::Weapon(WeaponKind::SoulTether) => "Soul Tether",
            Upgrade::Weapon(WeaponKind::LanternVigil) => "Lantern Vigil",
            Upgrade::Weapon(WeaponKind::IaiFlash) => "Iai Flash",
            Upgrade::Weapon(WeaponKind::CrimsonTendrils) => "Crimson Tendrils",
            Upgrade::Weapon(WeaponKind::MukadeSpines) => "Mukade Spines",
            Upgrade::Weapon(WeaponKind::Devour) => "Devour",
            Upgrade::Weapon(WeaponKind::VenomFang) => "Venom Fang",
            Upgrade::Weapon(WeaponKind::GravityWell) => "Gravity Well",
            Upgrade::Weapon(WeaponKind::RepulsionStar) => "Repulsion Star",
            Upgrade::Weapon(WeaponKind::VoidLance) => "Void Lance",
            Upgrade::Weapon(WeaponKind::YinYangOrbs) => "Yin-Yang Orbs",
            Upgrade::Weapon(WeaponKind::SpiritSeal) => "Spirit Seal",
            Upgrade::Weapon(WeaponKind::PurificationWave) => "Purification Wave",
            Upgrade::Passive(PassiveKind::Might) => "Might",
            Upgrade::Passive(PassiveKind::Haste) => "Haste",
            Upgrade::Passive(PassiveKind::Expanse) => "Expanse",
            Upgrade::Passive(PassiveKind::Swiftness) => "Swiftness",
            Upgrade::Passive(PassiveKind::Magnet) => "Magnetism",
            Upgrade::Passive(PassiveKind::Vitality) => "Vitality",
            Upgrade::Passive(PassiveKind::Abundance) => "Abundance",
            Upgrade::Passive(PassiveKind::Persistence) => "Persistence",
            Upgrade::Passive(PassiveKind::Recovery) => "Recovery",
            Upgrade::Passive(PassiveKind::Guard) => "Guard",
            Upgrade::Heal => "Onigiri",
            Upgrade::Sen => "Sen Pouch",
            Upgrade::Blessing => "Kizuna Blessing",
        }
    }

    pub fn max_level(self) -> u8 {
        match self {
            Upgrade::Passive(p) => passive_max(p),
            _ => MAX_LEVEL,
        }
    }

    /// Description of what taking this upgrade at `next_level` does.
    pub fn describe(self, next_level: u8) -> String {
        match self {
            Upgrade::Weapon(k) if next_level == 1 => match k {
                WeaponKind::Ofuda => "Throws homing paper talismans at the nearest enemy.",
                WeaponKind::MoonOrbs => "Moon orbs circle you, striking what they touch.",
                WeaponKind::SakuraField => "A ring of sakura petals hurts nearby enemies.",
                WeaponKind::ThunderChain => "A bolt leaps between nearby enemies.",
                WeaponKind::Taiko => "Drum beats send out a damaging shockwave.",
                WeaponKind::StarFall => "Stars crash down on enemies around you.",
                WeaponKind::Crescent => "A crescent slash cuts in front of you.",
                WeaponKind::FlamePool => "Throws flasks that leave burning ground.",
                WeaponKind::FoxfireOrb => "Gathers foxfire in your hand, then hurls it to burst on the nearest enemy.",
                WeaponKind::Familiars => "Fox spirits dart out and bite through nearby enemies.",
                WeaponKind::GaleShuriken => "A huge shuriken spins out and returns to you, cutting both ways.",
                WeaponKind::FoxMirage => "Conjures illusory copies of you. They fight, draw enemies away, and burst into smoke.",
                WeaponKind::GraveCleaver => "A huge blade sweeps in front of you and looses a crescent wave of spirit energy.",
                WeaponKind::SoulTether => "Spectral chains latch onto nearby enemies, draining them and dragging them closer.",
                WeaponKind::LanternVigil => "Sets soul-lanterns on the ground that pulse spirit fire at anything nearby.",
                WeaponKind::IaiFlash => "A lightning quick-draw that cuts a long straight line through the horde.",
                WeaponKind::CrimsonTendrils => "Crimson tendrils lash out and pierce straight through the nearest enemies.",
                WeaponKind::MukadeSpines => "Segmented spines erupt from the ground beneath nearby enemies.",
                WeaponKind::Devour => "A hungry pulse drains everything around you and heals you for each enemy caught.",
                WeaponKind::VenomFang => "Venomous fangs home in on enemies and burst on impact.",
                WeaponKind::GravityWell => "A gravity singularity drags a cluster of enemies together, then crushes them.",
                WeaponKind::RepulsionStar => "A crimson star tears through enemies and blasts them back.",
                WeaponKind::VoidLance => "A slow orb of void erases everything along its line.",
                WeaponKind::YinYangOrbs => "Two yin-yang orbs fly out to the edge of your reach and spin back through enemies.",
                WeaponKind::SpiritSeal => "Stamps a seal on random enemies; each bursts a moment later and hurts the crowd around it.",
                WeaponKind::PurificationWave => "A wide purifying ring blasts enemies back and erases fireballs.",
            }
            .into(),
            Upgrade::Weapon(k) => {
                let a = weapon_stats(k, next_level - 1);
                let b = weapon_stats(k, next_level);
                let mut parts = vec![format!("Damage {:.0} -> {:.0}", a.damage, b.damage)];
                if b.amount != a.amount {
                    let unit = match k {
                        WeaponKind::Ofuda => "talisman",
                        WeaponKind::MoonOrbs => "orb",
                        WeaponKind::ThunderChain => "jump",
                        WeaponKind::Taiko => "wave",
                        WeaponKind::StarFall => "star",
                        WeaponKind::Crescent => "slash",
                        WeaponKind::FlamePool => "flask",
                        WeaponKind::FoxfireOrb => "orb",
                        WeaponKind::Familiars => "spirit",
                        WeaponKind::GaleShuriken => "shuriken",
                        WeaponKind::FoxMirage => "clone",
                        WeaponKind::GraveCleaver => "swing",
                        WeaponKind::SoulTether => "chain",
                        WeaponKind::LanternVigil => "lantern",
                        WeaponKind::IaiFlash => "cut",
                        WeaponKind::CrimsonTendrils => "tendril",
                        WeaponKind::MukadeSpines => "spine",
                        WeaponKind::Devour => "",
                        WeaponKind::VenomFang => "fang",
                        WeaponKind::GravityWell => "well",
                        WeaponKind::RepulsionStar => "star",
                        WeaponKind::VoidLance => "lance",
                        WeaponKind::YinYangOrbs => "orb",
                        WeaponKind::SpiritSeal => "seal",
                        WeaponKind::PurificationWave => "",
                        WeaponKind::SakuraField => "",
                    };
                    parts.push(format!("+{} {unit}", b.amount - a.amount));
                }
                if b.pierce != a.pierce {
                    parts.push("+1 pierce".into());
                }
                if (b.area - a.area).abs() > 1e-3 {
                    parts.push(format!("+{:.0}% area", (b.area / a.area - 1.0) * 100.0));
                }
                if (b.cooldown - a.cooldown).abs() > 1e-3 {
                    parts.push("faster".into());
                }
                parts.join(", ")
            }
            Upgrade::Passive(p) => match p {
                PassiveKind::Might => "+10% damage.",
                PassiveKind::Haste => "-8% weapon cooldown.",
                PassiveKind::Expanse => "+10% area of effect.",
                PassiveKind::Swiftness => "+8% move speed.",
                PassiveKind::Magnet => "+30% pickup radius.",
                PassiveKind::Vitality => "+20 max HP. Heals 20 when taken.",
                PassiveKind::Abundance => "+1 to every weapon's count.",
                PassiveKind::Persistence => "+12% effect duration and range.",
                PassiveKind::Recovery => "Regenerate 0.25 HP per second.",
                PassiveKind::Guard => "Take 1 less damage per hit.",
            }
            .into(),
            Upgrade::Heal => format!("Restore {:.0}% of max HP.", ONIGIRI_HEAL * 100.0),
            Upgrade::Sen => "Sen that grows as the night goes on.".into(),
            Upgrade::Blessing => format!("+{:.0}% damage for the rest of the run. Adds up.", (BLESSING_DAMAGE - 1.0) * 100.0),
        }
    }
}

// ───────────────────────── Chests ─────────────────────────

/// Number of rewards in a chest: 85% 1, 10% 2, 4% 3, 1% 5. `r` is uniform in [0, 1).
pub fn chest_roll(r: f32) -> u8 {
    if r < 0.85 {
        1
    } else if r < 0.95 {
        2
    } else if r < 0.99 {
        3
    } else {
        5
    }
}

/// Extra chest luck that ramps in over the first 10 minutes of stage pace, in step with the spawn ladder.
pub fn chest_time_luck(t: f32) -> f32 {
    (t / 600.0).min(1.0) * 2.3
}

pub fn chest_rarity(count: usize) -> &'static str {
    match count {
        0 | 1 => "Common",
        2 => "Rare",
        3 | 4 => "Epic",
        _ => "Legendary",
    }
}

// ───────────────────────── Shrine Events ─────────────────────────

/// A shrine offers a blessing that always comes with a price.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShrineEvent {
    Fury,
    GlassBlade,
    Feast,
    Fortune,
    Patience,
    Harvest,
    IronSkin,
    SecondWind,
    Tailwind,
    Offering,
}

pub const ALL_SHRINE_EVENTS: [ShrineEvent; 10] = [
    ShrineEvent::Fury,
    ShrineEvent::GlassBlade,
    ShrineEvent::Feast,
    ShrineEvent::Fortune,
    ShrineEvent::Patience,
    ShrineEvent::Harvest,
    ShrineEvent::IronSkin,
    ShrineEvent::SecondWind,
    ShrineEvent::Tailwind,
    ShrineEvent::Offering,
];

impl ShrineEvent {
    pub fn name(self) -> &'static str {
        match self {
            ShrineEvent::Fury => "Fury",
            ShrineEvent::GlassBlade => "Glass Blade",
            ShrineEvent::Feast => "Feast",
            ShrineEvent::Fortune => "Fortune",
            ShrineEvent::Patience => "Patience",
            ShrineEvent::Harvest => "Harvest",
            ShrineEvent::IronSkin => "Iron Skin",
            ShrineEvent::SecondWind => "Second Wind",
            ShrineEvent::Tailwind => "Tailwind",
            ShrineEvent::Offering => "Offering",
        }
    }

    /// (blessing, price)
    pub fn text(self) -> (&'static str, &'static str) {
        match self {
            ShrineEvent::Fury => ("+30% damage", "25% more enemies"),
            ShrineEvent::GlassBlade => ("+50% damage", "-30% max HP"),
            ShrineEvent::Feast => ("Fully heal", "Lose a level from your lowest passive"),
            ShrineEvent::Fortune => ("Open a treasure chest now", "-15 max HP"),
            ShrineEvent::Patience => ("+2 levels to a random weapon", "Enemies have +15% HP"),
            ShrineEvent::Harvest => ("+40% XP gained", "Enemies move 10% faster"),
            ShrineEvent::IronSkin => ("Take 3 less damage per hit", "-15% move speed"),
            ShrineEvent::SecondWind => ("Special full now, charges 30% faster", "Lose 25 HP"),
            ShrineEvent::Tailwind => ("+20% move speed", "-25% pickup radius"),
            ShrineEvent::Offering => ("Gain 3 levels", "Healing halved"),
        }
    }
}

/// Run-wide multipliers and bonuses from Shrine Events. Neutral by default.
#[derive(Clone, Copy, Debug)]
pub struct Modifiers {
    pub damage: f32,
    /// Multiplies how many enemies the director keeps alive.
    pub curse: f32,
    pub enemy_hp: f32,
    pub enemy_speed: f32,
    pub xp: f32,
    pub move_speed: f32,
    pub magnet: f32,
    pub armor: f32,
    pub special_rate: f32,
    pub healing: f32,
}

impl Default for Modifiers {
    fn default() -> Self {
        Modifiers {
            damage: 1.0,
            curse: 1.0,
            enemy_hp: 1.0,
            enemy_speed: 1.0,
            xp: 1.0,
            move_speed: 1.0,
            magnet: 1.0,
            armor: 0.0,
            special_rate: 1.0,
            healing: 1.0,
        }
    }
}

// ───────────────────────── Progression & pacing ─────────────────────────

/// Attacks only land inside the reach box, so kills are rarer; each gem is worth more to keep levelling on pace.
pub const XP_GAIN: f32 = 2.0;

pub fn xp_needed(level: u32) -> u32 {
    let l = level as f32;
    (5.0 + 3.5 * l + 0.35 * l * l).round() as u32
}

/// Minimum enemies alive at time `t` (seconds). Linear between keyframes.
pub fn min_alive(t: f32) -> usize {
    const KEYS: [(f32, f32); 9] = [
        (0.0, 28.0),
        (60.0, 80.0),
        (180.0, 175.0),
        (300.0, 260.0),
        (480.0, 450.0),
        (660.0, 820.0),
        (780.0, 1250.0),
        (870.0, 1850.0),
        (900.0, 2200.0),
    ];
    let m = t.min(900.0);
    for w in KEYS.windows(2) {
        let ((t0, a), (t1, b)) = (w[0], w[1]);
        if m <= t1 {
            return (a + (b - a) * ((m - t0) / (t1 - t0))) as usize;
        }
    }
    KEYS[KEYS.len() - 1].1 as usize
}

/// Max new enemies per tick when below `min_alive`: 5 at the start, 30 by 15:00.
pub fn refill_per_tick(t: f32) -> usize {
    5 + (t / 36.0) as usize
}

/// Spawn ladder rung: 0 until 5:00, 1 until 10:00, then 2. From rung 1 the basic fodder
/// stops spawning and the crowd thins out, with stronger kinds taking its place.
pub fn ladder_rung(t: f32) -> usize {
    if t < 300.0 { 0 } else if t < 600.0 { 1 } else { 2 }
}
/// Ranged / blinker share multiplier per rung.
pub const LADDER_LIFT: [f32; 3] = [1.0, 2.0, 3.0];
/// Alive-count multiplier per rung; tougher kinds keep total enemy HP about the same.
pub const LADDER_COUNT: [f32; 3] = [1.0, 0.85, 0.85];

/// Share of new spawns that are Kasa (umbrella) instead of Wisp.
pub fn kasa_share(t: f32) -> f32 {
    if t < 90.0 {
        0.0
    } else {
        (0.15 + (t - 90.0) / 900.0 * 0.45).min(0.6)
    }
}

/// Extra enemy HP and damage step for each headline round reached.
pub const HEADLINE_BUMP: f32 = 1.01;

/// Enemy HP (x base) at 0:00, rising to `HP_START` by `HP_START_BY` pace-minutes: the first
/// minute still gives a few quick levels, then fodder takes several hits from a fresh weapon.
pub const HP_OPEN: f32 = 1.5;
pub const HP_START: f32 = 5.0;
pub const HP_START_BY: f32 = 3.0;
/// Extra enemy HP per minute after `HP_RAMP_FROM` minutes (compounding), so enemies keep up
/// with a build that has come together instead of melting from mid-run on.
pub const HP_RAMP: f32 = 0.17;
pub const HP_RAMP_FROM: f32 = 1.0;

/// Enemy HP multiplier over the run (`t` in pace seconds).
pub fn hp_scale(t: f32) -> f32 {
    let m = t / 60.0;
    let start = HP_OPEN + (HP_START - HP_OPEN) * (m / HP_START_BY).min(1.0);
    (start + 0.2 * m + 0.03 * m * m) * (1.0 + HP_RAMP).powf((m - HP_RAMP_FROM).max(0.0))
}

/// Enemies one area attack hits at full damage; each further enemy takes `AREA_FALLOFF` less,
/// down to `AREA_FLOOR`. Big crowds stop being free damage for screen-wide weapons, while
/// single-target and few-target hits (and boss fights) are untouched.
pub const AREA_FULL: u32 = 6;
pub const AREA_FALLOFF: f32 = 0.07;
pub const AREA_FLOOR: f32 = 0.3;

/// Damage share for the `n`-th enemy (0-based) one area attack hits.
pub fn area_falloff(n: u32) -> f32 {
    if n < AREA_FULL {
        1.0
    } else {
        (1.0 - AREA_FALLOFF * (n - AREA_FULL + 1) as f32).max(AREA_FLOOR)
    }
}

/// Enemy armour: a flat cut to every hit, phased in from `ARMOR_FROM` to `ARMOR_FULL` pace-
/// minutes up to `ARMOR_SHARE` x the enemy HP scale (so it keeps pace with the run). A hit always
/// deals at least `ARMOR_FLOOR` of itself: big hits barely notice, rapid small ticks lose up to half.
pub const ARMOR_SHARE: f32 = 0.15;
pub const ARMOR_FROM: f32 = 5.0;
pub const ARMOR_FULL: f32 = 10.0;
pub const ARMOR_FLOOR: f32 = 0.5;

/// Flat damage taken off each hit at pace time `t` (seconds).
pub fn enemy_armor(t: f32) -> f32 {
    let m = t / 60.0;
    ARMOR_SHARE * hp_scale(t) * ((m - ARMOR_FROM) / (ARMOR_FULL - ARMOR_FROM)).clamp(0.0, 1.0)
}

/// Enemy move-speed multiplier over the run (non-bosses): +3.5% per pace-minute, capped at
/// +50% by 15:00 so the fastest fodder (~58 base) still stays under the player's base 110.
pub fn speed_scale(t: f32) -> f32 {
    (1.0 + 0.035 * t / 60.0).min(1.5)
}

/// Enemy contact-damage multiplier over the run: 1x at the start, 4.75x by 15:00.
pub fn damage_scale(t: f32) -> f32 {
    1.0 + t / 240.0
}

/// Seconds between Oni elites: every 2 minutes, every minute from 10:00.
pub fn elite_interval(t: f32) -> f32 {
    if t >= 600.0 {
        60.0
    } else {
        120.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_cycle_matches_spec() {
        use Element::*;
        let cycle = [Water, Fire, Ice, Wind, Earth, Electric];
        for (i, a) in cycle.iter().enumerate() {
            let b = cycle[(i + 1) % 6];
            assert_eq!(a.beats(), Some(b));
            assert_eq!(element_mult(*a, b), 1.5);
            assert_eq!(element_mult(b, *a), 0.75);
            assert_eq!(element_mult(*a, Light), 0.9);
            assert_eq!(element_mult(Darkness, *a), 1.15);
        }
        assert_eq!(element_mult(Fire, Wind), 1.0);
        assert_eq!(element_mult(Light, Darkness), 1.5);
        assert_eq!(element_mult(Darkness, Light), 1.5);
        assert_eq!(element_mult(Light, Light), 1.0);
    }

    #[test]
    fn weapon_elements_follow_the_rules() {
        for e in ALL_ELEMENTS {
            let n = ALL_WEAPONS.iter().filter(|k| k.element() == e).count();
            if e.is_rare() { assert!(n <= 3, "{e:?} has {n}") } else { assert!(n >= 3, "{e:?} has {n}") }
        }
        for k in ALL_WEAPONS.iter().filter(|k| k.element().is_rare()) {
            assert!(Upgrade::Weapon(*k).tier() >= Tier::Legendary, "{k:?} is rare-element below Legendary");
        }
    }

    #[test]
    fn hero_levers_are_per_hero() {
        assert_eq!(hero_def(HeroId::Riku).reach_might, MIGHT_RIKU);
        assert_eq!(hero_def(HeroId::Toya).reach_might, MIGHT_TOYA);
        assert_eq!(hero_def(HeroId::Shun).reach_might, MIGHT_MELEE);
        assert_eq!(hero_def(HeroId::Haru).reach_might, MIGHT_MELEE);
        assert_eq!(hero_def(HeroId::Kaede).reach_might, 1.0);
        assert_eq!(hero_def(HeroId::Ren).reach_might, MIGHT_REN);
        assert_eq!(hero_def(HeroId::Kuon).reach_might, MIGHT_KUON);
    }

    #[test]
    fn tiers_follow_the_buckets() {
        use Tier::*;
        let count = |tiers: Vec<Tier>| ALL_TIERS.map(|t| tiers.iter().filter(|x| **x == t).count());
        let weapons = count(ALL_WEAPONS.iter().map(|k| Upgrade::Weapon(*k).tier()).collect());
        let passives = count(ALL_PASSIVES.iter().map(|k| Upgrade::Passive(*k).tier()).collect());
        assert_eq!(weapons, [7, 7, 5, 3, 2, 2]);
        assert_eq!(passives, [3, 3, 2, 1, 1, 0]);
        assert!(MAXED_FALLBACKS.iter().all(|u| u.tier() == Common));
    }

    #[test]
    fn signatures_are_never_unique() {
        for h in ALL_HEROES {
            assert_ne!(Upgrade::Weapon(hero_def(h).signature).tier(), Tier::Unique, "{h:?}");
        }
    }

    #[test]
    fn pickup_tiers_follow_spawn_weights() {
        for a in ALL_PICKUPS {
            for b in ALL_PICKUPS {
                if a.weight() > b.weight() {
                    assert!(a.tier() <= b.tier(), "{a:?} spawns more often than {b:?} but has a higher tier");
                }
            }
        }
    }

    #[test]
    fn tier_weights_fall_with_rarity() {
        let weights = ALL_TIERS.map(Tier::weight);
        assert_eq!(weights, [1.0, 0.8, 0.6, 0.4, 0.25, 0.15]);
        assert!(weights.windows(2).all(|w| w[0] > w[1]));
    }

    #[test]
    fn area_falloff_spares_small_groups() {
        assert!((0..AREA_FULL).all(|n| area_falloff(n) == 1.0));
        assert!(area_falloff(AREA_FULL) < 1.0);
        assert!((AREA_FULL..200).all(|n| area_falloff(n + 1) <= area_falloff(n)));
        assert_eq!(area_falloff(500), AREA_FLOOR);
    }

    #[test]
    fn pacing_is_monotonic() {
        let mut prev = 0;
        for s in (0..=900).step_by(10) {
            let n = min_alive(s as f32);
            assert!(n >= prev, "min_alive dropped at {s}s");
            prev = n;
        }
        assert_eq!(min_alive(0.0), 28);
        assert_eq!(min_alive(900.0), 2200);
    }

    #[test]
    fn chest_odds_match_spec() {
        let mut counts = [0u32; 6];
        let n = 100_000;
        for i in 0..n {
            counts[chest_roll((i as f32 + 0.5) / n as f32) as usize] += 1;
        }
        assert_eq!((counts[1], counts[2], counts[3], counts[5]), (85_000, 10_000, 4_000, 1_000));
    }

    #[test]
    fn weapon_levels_never_get_worse() {
        for k in ALL_WEAPONS {
            for l in 1..MAX_LEVEL {
                let (a, b) = (weapon_stats(k, l), weapon_stats(k, l + 1));
                assert!(b.damage >= a.damage && b.amount >= a.amount && b.cooldown <= a.cooldown);
            }
        }
    }
}

/// Max HP one Vitality level grants.
pub const VITALITY_HP: f32 = 20.0;

/// Level a replacing skill joins at: half the overwritten level, within the new skill's cap.
pub fn inherit_level(new: Upgrade, old: u8) -> u8 {
    (old / 2).max(1).min(new.max_level())
}
