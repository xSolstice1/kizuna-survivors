//! Game content as plain data. Will move to RON files once the shapes settle.

pub const MAX_LEVEL: u8 = 8;
pub const MAX_WEAPONS: usize = 6;
pub const MAX_PASSIVES: usize = 6;
pub const RUN_SECONDS: f32 = 15.0 * 60.0;

// ───────────────────────── Enemies ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Wisp,
    Kasa,
    Oni,
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
    }
}

pub const MAX_ENEMY_RADIUS: f32 = 14.0;

// ───────────────────────── Weapons ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponKind {
    /// Volley of paper talismans at the nearest enemy.
    Ofuda,
    /// Moon orbs circling the player.
    MoonOrbs,
    /// Sakura aura damaging everything around the player.
    SakuraField,
}

pub const ALL_WEAPONS: [WeaponKind; 3] = [WeaponKind::Ofuda, WeaponKind::MoonOrbs, WeaponKind::SakuraField];

#[derive(Clone, Copy, Debug)]
pub struct WeaponStats {
    pub damage: f32,
    pub cooldown: f32,
    pub amount: u32,
    pub area: f32,
    pub speed: f32,
    pub pierce: u8,
}

/// Base stats at `level` (1..=8) before passives.
pub fn weapon_stats(kind: WeaponKind, level: u8) -> WeaponStats {
    let l = level.clamp(1, MAX_LEVEL) as u32;
    let lf = l as f32;
    match kind {
        WeaponKind::Ofuda => WeaponStats {
            damage: 9.0 + 2.5 * (lf - 1.0),
            cooldown: 1.0 - 0.05 * (lf - 1.0),
            amount: 2 + (l >= 3) as u32 + (l >= 5) as u32 + (l >= 7) as u32 + (l >= 8) as u32,
            area: 1.0,
            speed: 340.0,
            pierce: 1 + (l >= 4) as u8 + (l >= 6) as u8 + (l >= 8) as u8,
        },
        WeaponKind::MoonOrbs => WeaponStats {
            damage: 8.0 + 2.0 * (lf - 1.0),
            // Re-hit interval on the same enemy.
            cooldown: 0.6,
            amount: 1 + (l >= 2) as u32 + (l >= 4) as u32 + (l >= 6) as u32 + (l >= 8) as u32,
            area: 1.0 + 0.08 * (lf - 1.0),
            speed: 3.2 + 0.15 * (lf - 1.0), // radians / second
            pierce: 0,
        },
        WeaponKind::SakuraField => WeaponStats {
            damage: 4.0 + 1.5 * (lf - 1.0),
            cooldown: 0.5, // tick interval
            amount: 1,
            area: 1.0 + 0.12 * (lf - 1.0),
            speed: 0.0,
            pierce: 0,
        },
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
}

pub const ALL_PASSIVES: [PassiveKind; 6] = [
    PassiveKind::Might,
    PassiveKind::Haste,
    PassiveKind::Expanse,
    PassiveKind::Swiftness,
    PassiveKind::Magnet,
    PassiveKind::Vitality,
];

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
            Upgrade::Passive(PassiveKind::Might) => "Might",
            Upgrade::Passive(PassiveKind::Haste) => "Haste",
            Upgrade::Passive(PassiveKind::Expanse) => "Expanse",
            Upgrade::Passive(PassiveKind::Swiftness) => "Swiftness",
            Upgrade::Passive(PassiveKind::Magnet) => "Spirit Magnet",
            Upgrade::Passive(PassiveKind::Vitality) => "Vitality",
            Upgrade::Heal => "Onigiri",
        }
    }

    /// Description of what taking this upgrade at `next_level` does.
    pub fn describe(self, next_level: u8) -> String {
        match self {
            Upgrade::Weapon(k) if next_level == 1 => match k {
                WeaponKind::Ofuda => "Throws paper talismans at the nearest Kegare.".into(),
                WeaponKind::MoonOrbs => "Moon orbs circle you, striking what they touch.".into(),
                WeaponKind::SakuraField => "A ring of sakura petals hurts nearby Kegare.".into(),
            },
            Upgrade::Weapon(k) => {
                let a = weapon_stats(k, next_level - 1);
                let b = weapon_stats(k, next_level);
                let mut parts = vec![format!("Damage {:.0} → {:.0}", a.damage, b.damage)];
                if b.amount != a.amount {
                    parts.push(format!("+{} {}", b.amount - a.amount, if k == WeaponKind::MoonOrbs { "orb" } else { "talisman" }));
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
                PassiveKind::Might => "+10% damage.".into(),
                PassiveKind::Haste => "−8% weapon cooldown.".into(),
                PassiveKind::Expanse => "+10% area of effect.".into(),
                PassiveKind::Swiftness => "+8% move speed.".into(),
                PassiveKind::Magnet => "+30% pickup radius.".into(),
                PassiveKind::Vitality => "+20 max HP and heal 20.".into(),
            },
            Upgrade::Heal => "Restore 30 HP.".into(),
        }
    }
}

// ───────────────────────── Progression & pacing ─────────────────────────

pub fn xp_needed(level: u32) -> u32 {
    let l = level as f32;
    (5.0 + 3.5 * l + 0.12 * l * l).round() as u32
}

/// Minimum enemies alive at time `t` (seconds). Linear between keyframes.
pub fn min_alive(t: f32) -> usize {
    const KEYS: [(f32, f32); 9] = [
        (0.0, 25.0),
        (60.0, 70.0),
        (180.0, 150.0),
        (300.0, 220.0),
        (480.0, 380.0),
        (660.0, 600.0),
        (780.0, 850.0),
        (870.0, 1300.0),
        (900.0, 1500.0),
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
    1.0 + t / 60.0 * 0.09
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
        assert_eq!(min_alive(900.0), 1500);
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
