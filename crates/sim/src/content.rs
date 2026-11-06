//! Game content as plain data. Will move to RON files once the shapes settle.

use crate::meta::Bonus;

pub const MAX_LEVEL: u8 = 8;
pub const RUN_SECONDS: f32 = 15.0 * 60.0;

// ───────────────────────── Enemies ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Wisp,
    Kasa,
    Oni,
    /// Mid-boss at 8:00: telegraphed charges, drops a 3-item chest.
    GreatOni,
    /// Paper-lantern yokai: keeps its distance and spits fireballs.
    Chochin,
    /// Faceless ghost: fades out and reappears beside the player.
    Nopperabo,
    /// Night Lord at 15:00 (Chochin Daimyo). Defeating it wins the night.
    NightLord,
}

pub const ALL_ENEMIES: [EnemyKind; 7] = [
    EnemyKind::Wisp,
    EnemyKind::Kasa,
    EnemyKind::Chochin,
    EnemyKind::Nopperabo,
    EnemyKind::Oni,
    EnemyKind::GreatOni,
    EnemyKind::NightLord,
];

impl EnemyKind {
    pub fn index(self) -> usize {
        ALL_ENEMIES.iter().position(|k| *k == self).unwrap()
    }
    pub fn name(self) -> &'static str {
        match self {
            EnemyKind::Wisp => "Hitodama Wisp",
            EnemyKind::Kasa => "Kasa-obake",
            EnemyKind::Chochin => "Chochin-obake",
            EnemyKind::Nopperabo => "Nopperabo",
            EnemyKind::Oni => "Oni",
            EnemyKind::GreatOni => "Great Oni",
            EnemyKind::NightLord => "Night Lord: Chochin Daimyo",
        }
    }
    /// Collection entry text.
    pub fn lore(self) -> &'static str {
        match self {
            EnemyKind::Wisp => "A stray soul-flame from a story nobody finished. Alone it flickers; a crowd of them smothers.",
            EnemyKind::Kasa => "An umbrella left behind too long. Hops on one leg and stares with its one eye.",
            EnemyKind::Chochin => "A paper lantern with a grudge. Keeps its distance and spits fireballs.",
            EnemyKind::Nopperabo => "A faceless passer-by. Fades out and reappears right beside you.",
            EnemyKind::Oni => "A horned brute from the old tales. Hits hard and guards a treasure chest.",
            EnemyKind::GreatOni => "Arrives at 8:00. Plants its feet, then charges in a straight line.",
            EnemyKind::NightLord => "Lantern warlord of the Shotengai, risen at 15:00. Defeat it before dawn.",
        }
    }
    pub fn is_boss(self) -> bool {
        matches!(self, EnemyKind::GreatOni | EnemyKind::NightLord)
    }
}

pub struct EnemyDef {
    pub hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub radius: f32,
    pub xp: u32,
    /// 0 = no knockback, 1 = full.
    pub knockback: f32,
}

pub fn enemy_def(kind: EnemyKind) -> EnemyDef {
    match kind {
        EnemyKind::Wisp => EnemyDef { hp: 8.0, speed: 46.0, damage: 5.0, radius: 7.0, xp: 1, knockback: 1.0 },
        EnemyKind::Kasa => EnemyDef { hp: 22.0, speed: 56.0, damage: 9.0, radius: 7.0, xp: 2, knockback: 0.7 },
        EnemyKind::Oni => EnemyDef { hp: 420.0, speed: 38.0, damage: 20.0, radius: 14.0, xp: 30, knockback: 0.0 },
        EnemyKind::GreatOni => EnemyDef { hp: 7000.0, speed: 34.0, damage: 30.0, radius: 26.0, xp: 120, knockback: 0.0 },
        EnemyKind::Chochin => EnemyDef { hp: 30.0, speed: 34.0, damage: 8.0, radius: 8.0, xp: 3, knockback: 0.6 },
        EnemyKind::Nopperabo => EnemyDef { hp: 45.0, speed: 44.0, damage: 9.0, radius: 7.0, xp: 4, knockback: 0.5 },
        // HP is absolute (not time-scaled): see `night_lord_hp`.
        EnemyKind::NightLord => EnemyDef { hp: 1.0, speed: 28.0, damage: 35.0, radius: 30.0, xp: 0, knockback: 0.0 },
    }
}

pub const MAX_ENEMY_RADIUS: f32 = 30.0;

/// The Night Lord arrives at RUN_SECONDS; if still alive at this time, dawn takes the player.
pub const DAWN_LIMIT: f32 = 17.0 * 60.0;

pub fn night_lord_hp() -> f32 {
    50_000.0
}

/// Seconds a Nopperabo takes to fade back in after a blink (harmless meanwhile).
pub const NOPPERABO_FADE_IN: f32 = 0.6;

/// Enemy fireballs.
pub const SHOT_RADIUS: f32 = 5.0;
pub const SHOT_DAMAGE: f32 = 4.0;

/// Share of new spawns that are Chochin (from 4:00) and Nopperabo (from 9:00, after the mid-boss).
pub fn chochin_share(t: f32) -> f32 {
    if t < 240.0 { 0.0 } else { (0.01 + (t - 240.0) / 660.0 * 0.02).min(0.03) }
}

pub fn nopperabo_share(t: f32) -> f32 {
    if t < 540.0 { 0.0 } else { (0.02 + (t - 540.0) / 360.0 * 0.02).min(0.04) }
}
pub const MIDBOSS_AT: f32 = 480.0;

/// Minimum rewards in the chest dropped on death (0 = no chest).
pub fn chest_tier(kind: EnemyKind) -> u8 {
    match kind {
        EnemyKind::Oni => 1,
        EnemyKind::GreatOni => 3,
        _ => 0,
    }
}

// ───────────────────────── Ren's special ─────────────────────────

/// Kills needed to fill the special meter.
pub const SPECIAL_KILLS: f32 = 120.0;
pub const BARRIER_RADIUS: f32 = 170.0;
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
/// Shadow Step: teleport next to an enemy, the afterimage left behind explodes.
pub const HANNYA_STEP_EVERY: f32 = 1.8;
pub const HANNYA_AFTERIMAGE_RADIUS: f32 = 64.0;
pub const HANNYA_AFTERIMAGE_DAMAGE: f32 = 45.0;
/// Hannya's Wrath: the finale, charged over the last second.
pub const HANNYA_FINALE_CHARGE: f32 = 1.0;
pub const HANNYA_FINALE_RADIUS: f32 = 340.0;
pub const HANNYA_FINALE_DAMAGE: f32 = 220.0;
pub const HANNYA_DAMAGE_TAKEN: f32 = 0.7;
pub const HANNYA_MOVE_SPEED: f32 = 1.15;

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
}

pub const ALL_WEAPONS: [WeaponKind; 16] = [
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
            damage: 9.0 + 2.5 * k,
            cooldown: 1.0 - 0.05 * k,
            amount: 2 + at(l, 3) + at(l, 5) + at(l, 7) + at(l, 8),
            speed: 340.0,
            pierce: 1 + at(l, 4) as u8 + at(l, 6) as u8 + at(l, 8) as u8,
            ..base
        },
        WeaponKind::MoonOrbs => WeaponStats {
            damage: 8.0 + 2.0 * k,
            cooldown: 0.6, // re-hit interval on the same enemy
            amount: 1 + at(l, 2) + at(l, 4) + at(l, 6) + at(l, 8),
            area: 1.0 + 0.08 * k,
            speed: 3.2 + 0.15 * k, // radians / second
            ..base
        },
        WeaponKind::SakuraField => WeaponStats { damage: 4.0 + 1.5 * k, cooldown: 0.5, area: 1.0 + 0.12 * k, ..base },
        WeaponKind::ThunderChain => WeaponStats {
            damage: 12.0 + 3.0 * k,
            cooldown: 1.6 - 0.06 * k,
            amount: 3 + l / 2, // jumps
            area: 1.0 + 0.05 * k, // jump range
            ..base
        },
        WeaponKind::Taiko => WeaponStats {
            damage: 8.0 + 2.4 * k,
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
            damage: 5.0 + 1.5 * k, // per 0.3s tick
            cooldown: 3.2 - 0.12 * k,
            amount: 1 + at(l, 3) + at(l, 6),
            area: 1.0 + 0.08 * k,
            duration: 2.5 + 0.15 * k,
            ..base
        },
        WeaponKind::FoxfireOrb => WeaponStats {
            damage: 17.0 + 4.5 * k, // burst damage
            cooldown: 1.6 - 0.06 * k,
            amount: 1 + at(l, 4) + at(l, 7),
            area: 1.0 + 0.10 * k, // burst radius
            speed: 260.0,
            duration: 0.55, // charge time
            ..base
        },
        WeaponKind::Familiars => WeaponStats {
            damage: 10.0 + 2.5 * k,
            cooldown: 2.2 - 0.08 * k,
            amount: 2 + at(l, 3) + at(l, 5) + at(l, 7),
            speed: 170.0,
            pierce: 3 + at(l, 4) as u8 + at(l, 6) as u8 + at(l, 8) as u8,
            duration: 2.6, // lifetime
            ..base
        },
        WeaponKind::GaleShuriken => WeaponStats {
            damage: 12.0 + 3.0 * k,
            cooldown: 2.0 - 0.07 * k,
            amount: 1 + at(l, 3) + at(l, 6),
            area: 1.0 + 0.08 * k,
            speed: 300.0,
            duration: 1.4, // out-and-back time
            ..base
        },
        WeaponKind::FoxMirage => WeaponStats {
            damage: 9.0 + 2.5 * k, // per strike; the smoke pop deals 2x
            cooldown: 6.5 - 0.3 * k,
            amount: 1 + at(l, 3) + at(l, 5) + at(l, 7),
            area: 1.0 + 0.08 * k, // pop radius
            speed: 165.0,
            pierce: 3 + at(l, 4) as u8 + at(l, 8) as u8, // hits a clone can take
            duration: 4.0 + 0.3 * k,
        },
        WeaponKind::GraveCleaver => WeaponStats {
            damage: 16.0 + 4.0 * k,
            cooldown: 1.3 - 0.04 * k,
            amount: 1 + at(l, 4) + at(l, 7), // swings (alternating sides)
            area: 1.0 + 0.08 * k,
            speed: 320.0, // crescent wave speed
            ..base
        },
        WeaponKind::SoulTether => WeaponStats {
            damage: 4.0 + 1.2 * k, // per 0.25s tick
            cooldown: 3.0 - 0.1 * k,
            amount: 2 + at(l, 3) + at(l, 5) + at(l, 7), // chains
            area: 1.0 + 0.06 * k, // reach
            duration: 2.0 + 0.1 * k,
            ..base
        },
        WeaponKind::LanternVigil => WeaponStats {
            damage: 10.0 + 3.0 * k, // per pulse
            cooldown: 4.0 - 0.15 * k,
            amount: 1 + at(l, 3) + at(l, 6),
            area: 1.0 + 0.10 * k,
            duration: 4.0 + 0.3 * k,
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
}

pub const ALL_HEROES: [HeroId; 8] = [HeroId::Ren, HeroId::Riku, HeroId::Shun, HeroId::Kaede, HeroId::Mika, HeroId::Haru, HeroId::Nanami, HeroId::Yuzu];

/// What Space does when the meter is full.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialKind {
    /// Purifying Barrier: a ring that damages, knocks back and erases fireballs.
    Barrier,
    /// Ninefold Foxfire: a timed transformation (tails, aura, golden orb volleys).
    NineTails,
    /// Hannya Unbound: a timed berserk form (beams, crescent waves, shadow steps, finale blast).
    Hannya,
}

impl SpecialKind {
    /// Length of a transformation special (0 for instant ones).
    pub fn duration(self) -> f32 {
        match self {
            SpecialKind::Barrier => 0.0,
            SpecialKind::NineTails => FOX_DURATION,
            SpecialKind::Hannya => HANNYA_DURATION,
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
        }
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
    pub weapon_slots: usize,
    pub passive_slots: usize,
    /// False while the hero's own sprite and special move are still placeholders.
    pub finished: bool,
    pub special: SpecialKind,
    pub special_name: &'static str,
}

pub fn hero_def(id: HeroId) -> HeroDef {
    use Tag::*;
    let b = Bonus::default();
    let (name, archetype, innate_text, innate, signature, pool_tags, finished): (_, _, _, _, _, &'static [Tag], _) = match id {
        HeroId::Ren => ("Ren Amane", "Shrine maiden", "+10% pickup radius", Bonus { magnet: 0.10, ..b }, WeaponKind::Ofuda, &[Talisman, Celestial, Nature, Festival], true),
        HeroId::Kaede => ("Kaede Kurogane", "Ninja", "+10% move speed", Bonus { move_speed: 0.10, ..b }, WeaponKind::ThunderChain, &[Thunder, Blade, Talisman, Fire], false),
        HeroId::Mika => ("Mika Hoshizora", "Magical girl", "+8% damage", Bonus { might: 0.08, ..b }, WeaponKind::StarFall, &[Celestial, Nature, Festival, Thunder], false),
        HeroId::Haru => ("Haru Tachibana", "Swordsman", "+1 armor", Bonus { armor: 1.0, ..b }, WeaponKind::Crescent, &[Blade, Nature, Festival, Thunder], false),
        HeroId::Nanami => ("Nanami Kiri", "Alchemist", "+10% area", Bonus { area: 0.10, ..b }, WeaponKind::FlamePool, &[Fire, Festival, Nature, Celestial], false),
        HeroId::Yuzu => ("Yuzu Momose", "Idol", "+10% experience", Bonus { growth: 0.10, ..b }, WeaponKind::Taiko, &[Festival, Celestial, Nature, Talisman], false),
        HeroId::Riku => ("Riku Kazamine", "Fox-spirit shinobi", "+5% move speed, +5% damage", Bonus { move_speed: 0.05, might: 0.05, ..b }, WeaponKind::FoxfireOrb, &[Fox, Thunder, Blade, Fire], true),
        HeroId::Shun => ("Shun Yamikiri", "Spirit ferryman", "+10% area", Bonus { area: 0.10, ..b }, WeaponKind::GraveCleaver, &[Spirit, Blade, Thunder, Fire], true),
    };
    let (special, special_name) = match id {
        HeroId::Ren => (SpecialKind::Barrier, "Purify"),
        HeroId::Riku => (SpecialKind::NineTails, "Ninefold Foxfire"),
        HeroId::Shun => (SpecialKind::Hannya, "Hannya Unbound"),
        _ => (SpecialKind::Barrier, "Barrier"),
    };
    HeroDef { id, name, archetype, innate_text, innate, signature, pool_tags, weapon_slots: 4, passive_slots: 6, finished, special, special_name }
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
    }
}

/// Collection text for an Awakening.
pub fn awakened_text(kind: WeaponKind) -> &'static str {
    match kind {
        WeaponKind::Ofuda => "Seal Storm: an endless stream of homing seals that explode on contact, on top of twice the talismans.",
        WeaponKind::MoonOrbs => "Lunar Eclipse: a second ring of moons, and moonbeams sweep the whole screen every few seconds.",
        WeaponKind::SakuraField => "Hanafubuki: a ceaseless spiral storm of cutting petals pours out of a wider field that heals you.",
        WeaponKind::ThunderChain => "Heavenfall: lightning keeps crashing down from the sky on nearby enemies, plus twice the chains.",
        WeaponKind::Taiko => "Festival Quake: colossal shockwaves roll out across the screen between the beats.",
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

// ───────────────────────── Upgrades ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Upgrade {
    Weapon(WeaponKind),
    Passive(PassiveKind),
    /// Fallback once everything is maxed.
    Heal,
}

impl Upgrade {
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
                PassiveKind::Vitality => "+20 max HP and heal 20.",
                PassiveKind::Abundance => "+1 to every weapon's count.",
                PassiveKind::Persistence => "+12% effect duration and range.",
                PassiveKind::Recovery => "Regenerate 0.25 HP per second.",
                PassiveKind::Guard => "Take 1 less damage per hit.",
            }
            .into(),
            Upgrade::Heal => "Restore 30 HP.".into(),
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

pub fn chest_rarity(count: usize) -> &'static str {
    match count {
        0 | 1 => "Common",
        2 => "Rare",
        3 | 4 => "Epic",
        _ => "Legendary",
    }
}

// ───────────────────────── Shrine Events ─────────────────────────

/// Times (seconds) a roadside shrine appears.
pub const SHRINE_TIMES: [f32; 4] = [180.0, 360.0, 540.0, 720.0];

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

pub fn xp_needed(level: u32) -> u32 {
    let l = level as f32;
    (5.0 + 3.5 * l + 0.35 * l * l).round() as u32
}

/// Minimum enemies alive at time `t` (seconds). Linear between keyframes.
pub fn min_alive(t: f32) -> usize {
    const KEYS: [(f32, f32); 9] = [
        (0.0, 25.0),
        (60.0, 70.0),
        (180.0, 150.0),
        (300.0, 220.0),
        (480.0, 380.0),
        (660.0, 700.0),
        (780.0, 1100.0),
        (870.0, 1700.0),
        (900.0, 2000.0),
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

/// Max new enemies per tick when below `min_alive`: 4 at the start, 24 by 15:00.
pub fn refill_per_tick(t: f32) -> usize {
    4 + (t / 45.0) as usize
}

/// Share of new spawns that are Kasa (umbrella) instead of Wisp.
pub fn kasa_share(t: f32) -> f32 {
    if t < 90.0 {
        0.0
    } else {
        (0.15 + (t - 90.0) / 900.0 * 0.45).min(0.6)
    }
}

/// Enemy HP multiplier over the run.
pub fn hp_scale(t: f32) -> f32 {
    let m = t / 60.0;
    1.0 + 0.15 * m + 0.03 * m * m
}

/// Enemy contact-damage multiplier over the run: 1x at the start, 2.5x by 15:00.
pub fn damage_scale(t: f32) -> f32 {
    1.0 + t / 600.0
}

/// Seconds between Oni elites: every 2 minutes, every minute from 10:00.
pub fn elite_interval(t: f32) -> f32 {
    if t >= 600.0 {
        60.0
    } else {
        120.0
    }
}

pub const ELITE_EVERY: f32 = 120.0;
pub const SWARM_RING_TIMES: [f32; 3] = [300.0, 600.0, 840.0];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pacing_is_monotonic() {
        let mut prev = 0;
        for s in (0..=900).step_by(10) {
            let n = min_alive(s as f32);
            assert!(n >= prev, "min_alive dropped at {s}s");
            prev = n;
        }
        assert_eq!(min_alive(0.0), 25);
        assert_eq!(min_alive(900.0), 2000);
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
