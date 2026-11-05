//! Game content as plain data. Will move to RON files once the shapes settle.

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
    }
}

pub const MAX_ENEMY_RADIUS: f32 = 26.0;
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
}

pub const ALL_WEAPONS: [WeaponKind; 8] = [
    WeaponKind::Ofuda,
    WeaponKind::MoonOrbs,
    WeaponKind::SakuraField,
    WeaponKind::ThunderChain,
    WeaponKind::Taiko,
    WeaponKind::StarFall,
    WeaponKind::Crescent,
    WeaponKind::FlamePool,
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
}

pub struct HeroDef {
    pub id: HeroId,
    pub name: &'static str,
    /// Starting weapon; always available to this hero.
    pub signature: WeaponKind,
    /// Weapons with any of these tags (plus `Shared`) can be offered.
    pub pool_tags: &'static [Tag],
    pub weapon_slots: usize,
    pub passive_slots: usize,
}

pub fn hero_def(id: HeroId) -> HeroDef {
    match id {
        HeroId::Ren => HeroDef {
            id,
            name: "Ren Amane",
            signature: WeaponKind::Ofuda,
            pool_tags: &[Tag::Talisman, Tag::Celestial, Tag::Nature, Tag::Festival],
            weapon_slots: 4,
            passive_slots: 6,
        },
    }
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
    }
    s
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
