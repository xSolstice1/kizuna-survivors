//! The campaign: stages (worlds), difficulties, timelines, world mechanics and relics.
//! A stage is plain data; the director, bosses and hazards in `world` read it, so a new
//! stage is a new `StageDef` plus its art.

use crate::enemies::EnemyKind::{self, *};
use crate::meta::Bonus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageId {
    /// Prologue: the original 15-minute night.
    Shotengai,
    ShrineValley,
    EmberLeaf,
    PaleCitadel,
    Rainward,
    Shatterline,
}

pub const ALL_STAGES: [StageId; 6] = [StageId::Shotengai, StageId::ShrineValley, StageId::EmberLeaf, StageId::PaleCitadel, StageId::Rainward, StageId::Shatterline];

impl StageId {
    pub fn index(self) -> usize {
        ALL_STAGES.iter().position(|s| *s == self).unwrap()
    }
    /// Save-file key (stable; never rename).
    pub fn key(self) -> &'static str {
        match self {
            StageId::Shotengai => "shotengai",
            StageId::ShrineValley => "shrine_valley",
            StageId::EmberLeaf => "ember_leaf",
            StageId::PaleCitadel => "pale_citadel",
            StageId::Rainward => "rainward",
            StageId::Shatterline => "shatterline",
        }
    }
    pub fn from_key(k: &str) -> Option<StageId> {
        ALL_STAGES.iter().copied().find(|s| s.key() == k)
    }
    pub fn def(self) -> &'static StageDef {
        &STAGES[self.index()]
    }
    /// The stage whose Normal clear unlocks this one (None = open from the start).
    pub fn requires(self) -> Option<StageId> {
        match self {
            StageId::Shotengai | StageId::ShrineValley => None,
            StageId::EmberLeaf => Some(StageId::ShrineValley),
            StageId::PaleCitadel => Some(StageId::EmberLeaf),
            StageId::Rainward => Some(StageId::PaleCitadel),
            StageId::Shatterline => Some(StageId::Rainward),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    Normal,
    Hard,
    Nightmare,
}

pub const ALL_DIFFICULTIES: [Difficulty; 3] = [Difficulty::Normal, Difficulty::Hard, Difficulty::Nightmare];

impl Difficulty {
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
            Difficulty::Nightmare => "Nightmare",
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Difficulty::Normal => "normal",
            Difficulty::Hard => "hard",
            Difficulty::Nightmare => "nightmare",
        }
    }
    pub fn from_key(k: &str) -> Option<Difficulty> {
        ALL_DIFFICULTIES.iter().copied().find(|d| d.key() == k)
    }
    /// Enemy count multiplier (moderate / high / very high).
    pub fn density(self) -> f32 {
        [1.0, 1.3, 1.6][self.index()]
    }
    pub fn damage(self) -> f32 {
        [1.0, 1.5, 2.2][self.index()]
    }
    pub fn hp(self) -> f32 {
        [1.0, 1.8, 3.0][self.index()]
    }
    /// Multiplier on the time between elites (occasional / frequent / very frequent).
    pub fn elite_gap(self) -> f32 {
        [1.0, 0.65, 0.45][self.index()]
    }
    /// Boss phases added on top of each boss's base count.
    pub fn extra_phases(self) -> u8 {
        [0, 1, 2][self.index()]
    }
    /// Sen for the first clear at this difficulty.
    pub fn first_clear_sen(self) -> u32 {
        [300, 800, 2000][self.index()]
    }
    /// Sen multiplier for coins found on the ground.
    pub fn greed(self) -> f32 {
        [1.0, 1.4, 2.0][self.index()]
    }
}

/// A world's signature hazard, ramping up during the crisis window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mechanic {
    None,
    /// Hidden Shrine Valley: bullet curtains sweep across the field.
    Curtains,
    /// Ember Leaf Frontier: explosive tags land around you (they hurt Otherworlders too).
    Tags,
    /// Pale Citadel: spirit-pressure zones drift after you and weigh your steps.
    Pressure,
    /// Rainward 13: the downpour hides the ward; lightning strikes where you stand.
    Downpour,
    /// Shatterline Crossing: hostile domains open around you. Get out before they close.
    Domains,
}

impl Mechanic {
    pub fn name(self) -> &'static str {
        match self {
            Mechanic::None => "",
            Mechanic::Curtains => "Danmaku Curtains",
            Mechanic::Tags => "Explosive Tags",
            Mechanic::Pressure => "Spirit Pressure",
            Mechanic::Downpour => "Downpour",
            Mechanic::Domains => "Hostile Domains",
        }
    }
    pub fn describe(self) -> &'static str {
        match self {
            Mechanic::None => "",
            Mechanic::Curtains => "Walls of bullets sweep across the valley. Find the gap.",
            Mechanic::Tags => "Paper bombs land around you. They blow up Otherworlders too.",
            Mechanic::Pressure => "Zones of crushing spirit pressure drift after you and slow your steps.",
            Mechanic::Downpour => "The rain cuts your sight. Lightning strikes where you stand still.",
            Mechanic::Domains => "Enemy domains open around you and burn anyone inside. Get out.",
        }
    }
}

/// Permanent reward for a Nightmare clear. Every relic owned is always active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relic {
    MoonveilBell,
    EmberHeadband,
    PaleMaskShard,
    WardKey,
    ShatteredSeal,
}

pub const ALL_RELICS: [Relic; 5] = [Relic::MoonveilBell, Relic::EmberHeadband, Relic::PaleMaskShard, Relic::WardKey, Relic::ShatteredSeal];

impl Relic {
    pub fn name(self) -> &'static str {
        match self {
            Relic::MoonveilBell => "Moonveil Bell",
            Relic::EmberHeadband => "Ember Leaf Headband",
            Relic::PaleMaskShard => "Pale Mask Shard",
            Relic::WardKey => "Ward 13 Key",
            Relic::ShatteredSeal => "Shattered Seal",
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Relic::MoonveilBell => "moonveil_bell",
            Relic::EmberHeadband => "ember_headband",
            Relic::PaleMaskShard => "pale_mask_shard",
            Relic::WardKey => "ward_key",
            Relic::ShatteredSeal => "shattered_seal",
        }
    }
    pub fn from_key(k: &str) -> Option<Relic> {
        ALL_RELICS.iter().copied().find(|r| r.key() == k)
    }
    /// Art name of its icon.
    pub fn icon(self) -> &'static str {
        match self {
            Relic::MoonveilBell => "relic_w1",
            Relic::EmberHeadband => "relic_w2",
            Relic::PaleMaskShard => "relic_w3",
            Relic::WardKey => "relic_w4",
            Relic::ShatteredSeal => "relic_w5",
        }
    }
    pub fn describe(self) -> &'static str {
        match self {
            Relic::MoonveilBell => "+10% area and +5% cooldown reduction.",
            Relic::EmberHeadband => "+10% move speed and +1 armor.",
            Relic::PaleMaskShard => "+12% damage.",
            Relic::WardKey => "+20% max health and +0.3 HP per second.",
            Relic::ShatteredSeal => "Get back up once more per run, and +10% experience.",
        }
    }
    pub fn bonus(self) -> Bonus {
        let mut b = Bonus::default();
        match self {
            Relic::MoonveilBell => {
                b.area = 0.10;
                b.cooldown = 0.05;
            }
            Relic::EmberHeadband => {
                b.move_speed = 0.10;
                b.armor = 1.0;
            }
            Relic::PaleMaskShard => b.might = 0.12,
            Relic::WardKey => {
                b.max_hp = 0.20;
                b.recovery = 0.3;
            }
            Relic::ShatteredSeal => {
                b.revivals = 1;
                b.growth = 0.10;
            }
        }
        b
    }
}

pub const ROUNDS: u8 = 20;
pub const SUB_BOSS_ROUND: u8 = 16;
pub const MID_BOSS_ROUND: u8 = 18;

/// The set piece that opens every fourth round.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Headline {
    Uncommon,
    Elite,
    Leader,
    SubBoss,
    Boss,
}

/// One stage: who spawns, when the bosses come, and the world's twist.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageDef {
    pub id: StageId,
    pub name: &'static str,
    /// "World 1", "Prologue".
    pub chapter: &'static str,
    pub blurb: &'static str,
    /// Weakest swarm enemy and its tougher mix-in.
    pub fodder: EnemyKind,
    pub fodder_b: EnemyKind,
    pub ranged: EnemyKind,
    pub blinker: EnemyKind,
    pub elite: EnemyKind,
    pub mini: EnemyKind,
    /// Optional second boss, spawned at round 18.
    pub boss: Option<EnemyKind>,
    pub final_boss: EnemyKind,
    pub final_at: f32,
    /// Dawn: the run ends (lost) if the final boss still lives.
    pub limit: f32,
    /// A burst of elites at round 8.
    pub elite_wave: bool,
    /// Density spikes and the mechanic intensifies in these rounds (inclusive).
    pub crisis: Option<(u8, u8)>,
    pub shrine_rounds: &'static [u8],
    pub ring_rounds: &'static [u8],
    pub mechanic: Mechanic,
    /// Multiplier on the share of ranged spawns.
    pub ranged_mult: f32,
    /// Multiplier on every enemy's HP (later worlds are tougher).
    pub tier: f32,
    /// Extra HP growth over a long run: hp x (1 + late_hp * (t / final_at)^2).
    pub late_hp: f32,
    /// Nightmare-clear reward.
    pub relic: Option<Relic>,
}

impl StageDef {
    /// Pacing time: the 15-minute curves stretched so their peak lands on the final boss.
    pub fn pace(&self, t: f32) -> f32 {
        t * 900.0 / self.final_at
    }
    pub fn in_crisis(&self, t: f32) -> bool {
        let r = self.round_at(t);
        self.crisis.is_some_and(|(a, b)| r >= a && r <= b)
    }
    pub fn round_len(&self) -> f32 {
        self.final_at / (ROUNDS - 1) as f32
    }
    /// Round 1..=20; the last starts exactly at the final boss.
    pub fn round_at(&self, t: f32) -> u8 {
        if t >= self.final_at {
            return ROUNDS;
        }
        let mut r = ((t.max(0.0) / self.round_len()) as u8 + 1).min(ROUNDS - 1);
        while r < ROUNDS - 1 && t >= self.round_start(r + 1) {
            r += 1;
        }
        while r > 1 && t < self.round_start(r) {
            r -= 1;
        }
        r
    }
    pub fn round_start(&self, r: u8) -> f32 {
        if r >= ROUNDS {
            return self.final_at;
        }
        (r.max(1) - 1) as f32 * self.round_len()
    }
    pub fn headline(r: u8) -> Option<Headline> {
        match r {
            4 => Some(Headline::Uncommon),
            8 => Some(Headline::Elite),
            12 => Some(Headline::Leader),
            16 => Some(Headline::SubBoss),
            20 => Some(Headline::Boss),
            _ => None,
        }
    }
    pub fn long(&self) -> bool {
        self.final_at > 900.0
    }
}

const WORLD_SHRINES: [u8; 5] = [3, 6, 9, 13, 16];
const WORLD_RINGS: [u8; 5] = [3, 7, 11, 15, 19];

const fn world(
    id: StageId,
    name: &'static str,
    chapter: &'static str,
    blurb: &'static str,
    roster: [EnemyKind; 8],
    mechanic: Mechanic,
    ranged_mult: f32,
    tier: f32,
    relic: Relic,
) -> StageDef {
    StageDef {
        id,
        name,
        chapter,
        blurb,
        fodder: roster[0],
        fodder_b: roster[1],
        ranged: roster[2],
        blinker: roster[3],
        elite: roster[4],
        mini: roster[5],
        boss: Some(roster[6]),
        final_boss: roster[7],
        final_at: 1500.0,
        limit: 1800.0,
        elite_wave: true,
        crisis: Some((15, 17)),
        shrine_rounds: &WORLD_SHRINES,
        ring_rounds: &WORLD_RINGS,
        mechanic,
        ranged_mult,
        tier,
        late_hp: 0.6,
        relic: Some(relic),
    }
}

pub static STAGES: [StageDef; 6] = [
    StageDef {
        id: StageId::Shotengai,
        name: "Neon Shotengai",
        chapter: "Prologue",
        blurb: "One night in the shopping street. Defeat the Night Lord before dawn.",
        fodder: Wisp,
        fodder_b: Kasa,
        ranged: Chochin,
        blinker: Nopperabo,
        elite: Oni,
        mini: GreatOni,
        boss: None,
        final_boss: NightLord,
        final_at: 900.0,
        limit: 1020.0,
        elite_wave: false,
        crisis: None,
        shrine_rounds: &[5, 9, 13, 17],
        ring_rounds: &[7, 13, 18],
        mechanic: Mechanic::None,
        ranged_mult: 1.0,
        tier: 1.0,
        late_hp: 0.0,
        relic: None,
    },
    world(
        StageId::ShrineValley,
        "Hidden Shrine Valley",
        "World 1",
        "A valley hidden from the world by a veiled moon. Its spirits fight in patterns of light.",
        [Kodama, PaperCrane, LanternFairy, FoxMaskPhantom, Komainu, BellMaiden, ToriiSerpent, Mitsuhime],
        Mechanic::Curtains,
        2.2,
        1.0,
        Relic::MoonveilBell,
    ),
    world(
        StageId::EmberLeaf,
        "Ember Leaf Frontier",
        "World 2",
        "A burning border forest where a ninja war never ended. Watch the ground.",
        [StrawPuppet, LeafShinobi, KunaiThrower, SmokeNinja, IronRonin, Kagerou, EmberToad, BurningShogun],
        Mechanic::Tags,
        1.4,
        1.15,
        Relic::EmberHeadband,
    ),
    world(
        StageId::PaleCitadel,
        "Pale Citadel of Souls",
        "World 3",
        "White walls without end, where lost souls wait at the gates. The air itself presses down.",
        [LostSoul, MaskedHound, PaleArcher, ShadeDancer, BoneSentinel, GateCaptain, HollowMonarch, PaleSeraph],
        Mechanic::Pressure,
        1.3,
        1.3,
        Relic::PaleMaskShard,
    ),
    world(
        StageId::Rainward,
        "Rainward 13",
        "World 4",
        "A drowned city ward under a red moon. Something hunts in the rain; so does something else.",
        [DrownedRat, RainGhoul, GutterSpitter, UmbrellaStalker, WardButcher, NightInvestigator, GutterMother, CrimsonRainLord],
        Mechanic::Downpour,
        1.2,
        1.45,
        Relic::WardKey,
    ),
    world(
        StageId::Shatterline,
        "Shatterline Crossing",
        "World 5",
        "The crossing where the city broke. Curses walk the zebra lines, and domains open without warning.",
        [CursedSpawn, WailingCommuter, CurseEye, GlitchShade, GradeOneBrute, PatchworkSorcerer, CrossingCalamity, ShatterSovereign],
        Mechanic::Domains,
        1.4,
        1.6,
        Relic::ShatteredSeal,
    ),
];

/// The stage an enemy belongs to (for the Collection).
pub fn home_stage(k: EnemyKind) -> StageId {
    for s in &STAGES {
        let roster = [s.fodder, s.fodder_b, s.ranged, s.blinker, s.elite, s.mini, s.final_boss];
        if roster.contains(&k) || s.boss == Some(k) {
            return s.id;
        }
    }
    StageId::Shotengai
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::{Rank, ALL_ENEMIES};

    #[test]
    fn every_enemy_has_a_home_and_rosters_match_ranks() {
        for k in ALL_ENEMIES {
            let s = home_stage(k).def();
            let listed = [s.fodder, s.fodder_b, s.ranged, s.blinker, s.elite, s.mini, s.final_boss].contains(&k) || s.boss == Some(k);
            assert!(listed, "{k:?} is in no roster");
        }
        for s in &STAGES {
            assert_eq!(s.elite.rank(), Rank::Elite, "{:?}", s.id);
            assert_eq!(s.mini.rank(), Rank::Mini, "{:?}", s.id);
            assert_eq!(s.final_boss.rank(), Rank::Final, "{:?}", s.id);
            if let Some(b) = s.boss {
                assert_eq!(b.rank(), Rank::Boss);
            }
            assert!(s.final_at < s.limit);
            assert_eq!(s.id.def().id, s.id);
        }
    }

    #[test]
    fn rounds_split_the_night() {
        let s = StageId::Shotengai.def();
        assert_eq!(s.round_at(0.0), 1);
        assert_eq!(s.round_at(s.final_at - 0.01), 19);
        assert_eq!(s.round_at(s.final_at), 20);
        assert_eq!(s.round_at(s.limit), 20);
        assert!((s.round_start(16) - 15.0 * s.final_at / 19.0).abs() < 1e-3);
        assert_eq!(s.round_start(20), s.final_at);
    }

    #[test]
    fn headlines_every_four_rounds() {
        use Headline::*;
        let got: Vec<_> = (1..=20).filter_map(|r| StageDef::headline(r).map(|h| (r, h))).collect();
        assert_eq!(got, vec![(4, Uncommon), (8, Elite), (12, Leader), (16, SubBoss), (20, Boss)]);
    }

    #[test]
    fn round_starts_round_trip() {
        for s in &STAGES {
            for r in 1..=ROUNDS {
                assert_eq!(s.round_at(s.round_start(r)), r, "{:?} round {r}", s.id);
            }
        }
    }
}
