//! Equipment: slots, rarities, rolled bonuses, uniques, world sets, the Arsenal slot,
//! drops, the Shrine Draw and crafting. Pure data and maths; the profile owns the items
//! and `World` reads a hero's loadout through `Loadout`.
//!
//! Every item has a main stat fixed by its slot, then one rolled bonus per rarity step
//! (Common 1 ... Mythic 6). Each bonus rolls within a range and remembers its quality
//! (0..1, shown as %). Values scale with item level, so endless Ascension keeps
//! producing better gear. Legendary and Mythic items also carry a unique effect.

use crate::campaign::StageId;
use crate::content::{Element, WeaponKind, ALL_ELEMENTS};
use crate::meta::Bonus;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Hat,
    Top,
    Bottom,
    Gloves,
    Shoes,
    Necklace,
    Ring,
    Earring,
    Charm,
    Omamori,
    /// A bonus weapon that does not take a weapon slot in the run.
    Arsenal,
}

pub const ALL_SLOTS: [Slot; 11] = [Slot::Hat, Slot::Top, Slot::Bottom, Slot::Gloves, Slot::Shoes, Slot::Necklace, Slot::Ring, Slot::Earring, Slot::Charm, Slot::Omamori, Slot::Arsenal];

/// The 12 loadout positions (two rings).
pub const LOADOUT: [Slot; 12] = [Slot::Hat, Slot::Top, Slot::Bottom, Slot::Gloves, Slot::Shoes, Slot::Necklace, Slot::Ring, Slot::Ring, Slot::Earring, Slot::Charm, Slot::Omamori, Slot::Arsenal];

impl Slot {
    pub fn name(self) -> &'static str {
        match self {
            Slot::Hat => "Hat",
            Slot::Top => "Top",
            Slot::Bottom => "Bottom",
            Slot::Gloves => "Gloves",
            Slot::Shoes => "Shoes",
            Slot::Necklace => "Necklace",
            Slot::Ring => "Ring",
            Slot::Earring => "Earring",
            Slot::Charm => "Charm",
            Slot::Omamori => "Omamori",
            Slot::Arsenal => "Arsenal",
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Slot::Hat => "hat",
            Slot::Top => "top",
            Slot::Bottom => "bottom",
            Slot::Gloves => "gloves",
            Slot::Shoes => "shoes",
            Slot::Necklace => "necklace",
            Slot::Ring => "ring",
            Slot::Earring => "earring",
            Slot::Charm => "charm",
            Slot::Omamori => "omamori",
            Slot::Arsenal => "arsenal",
        }
    }
    pub fn from_key(k: &str) -> Option<Slot> {
        ALL_SLOTS.iter().copied().find(|s| s.key() == k)
    }
    /// The stat every item in this slot carries (None for the Arsenal).
    pub fn main_stat(self) -> Option<Stat> {
        Some(match self {
            Slot::Hat => Stat::MaxHp,
            Slot::Top => Stat::Armor,
            Slot::Bottom => Stat::Recovery,
            Slot::Gloves => Stat::Might,
            Slot::Shoes => Stat::MoveSpeed,
            Slot::Necklace => Stat::Cooldown,
            Slot::Ring => Stat::CritChance,
            Slot::Earring => Stat::CritDamage,
            Slot::Charm => Stat::Area,
            Slot::Omamori => Stat::SpecialCharge,
            Slot::Arsenal => return None,
        })
    }
    fn bases(self) -> &'static [&'static str] {
        match self {
            Slot::Hat => &["Straw Kasa", "Silk Eboshi", "Fox-Ear Hood", "Moon Circlet", "Tengu Cap"],
            Slot::Top => &["Haori", "Travel Kimono", "Lacquer Cuirass", "Festival Happi", "Shrine Vestment"],
            Slot::Bottom => &["Hakama", "Wrapped Leggings", "Pleated Skirt", "Kobakama"],
            Slot::Gloves => &["Tekko", "Archer's Glove", "Ink-Stained Gloves", "Iron Bracers"],
            Slot::Shoes => &["Geta", "Waraji", "Tabi Boots", "Cloud Sandals"],
            Slot::Necklace => &["Prayer Beads", "Magatama Cord", "Bell Choker", "Lantern Pendant"],
            Slot::Ring => &["Jade Ring", "Moonstone Band", "Cinder Ring", "Spirit Loop"],
            Slot::Earring => &["Tassel Earring", "Coral Drop", "Star Stud", "Feather Earring"],
            Slot::Charm => &["Daruma Charm", "Folded Crane", "Lucky Cat", "Paper Lantern Charm"],
            Slot::Omamori => &["Health Omamori", "Victory Omamori", "Travel Omamori", "Love Omamori"],
            Slot::Arsenal => &["Arsenal"],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
    /// Hand-made artefacts: a fixed signature effect, 6 high-quality rolls, one worn at a time.
    Exotic,
    /// A world's set piece: Exotic-quality rolls on a set slot, with a unique.
    Set,
}

pub const ALL_RARITIES: [Rarity; 8] = [Rarity::Common, Rarity::Uncommon, Rarity::Rare, Rarity::Epic, Rarity::Legendary, Rarity::Mythic, Rarity::Exotic, Rarity::Set];

impl Rarity {
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        ["Common", "Uncommon", "Rare", "Epic", "Legendary", "Mythic", "Exotic", "Set"][self.index()]
    }
    /// Rolled bonuses: Common 1 ... Mythic 6 (Exotic and Set 6).
    pub fn affixes(self) -> usize {
        (self.index() + 1).min(6)
    }
    fn mult(self) -> f32 {
        [1.0, 1.06, 1.12, 1.2, 1.3, 1.45, 1.6, 1.75][self.index()]
    }
    /// Spirit Dust from salvaging.
    pub fn dust(self) -> u32 {
        [1, 3, 8, 20, 50, 140, 400, 800][self.index()]
    }
    /// Level of the library weapon a low-rarity Arsenal grants.
    pub fn arsenal_level(self) -> u8 {
        [1, 2, 4, 6, 8, 8, 8, 8][self.index()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    Might,
    MaxHp,
    Armor,
    Recovery,
    Cooldown,
    Area,
    MoveSpeed,
    Magnet,
    Growth,
    Greed,
    CritChance,
    CritDamage,
    BossDamage,
    SpecialCharge,
    Luck,
    GearFind,
}

pub const ALL_STATS: [Stat; 16] = [
    Stat::Might,
    Stat::MaxHp,
    Stat::Armor,
    Stat::Recovery,
    Stat::Cooldown,
    Stat::Area,
    Stat::MoveSpeed,
    Stat::Magnet,
    Stat::Growth,
    Stat::Greed,
    Stat::CritChance,
    Stat::CritDamage,
    Stat::BossDamage,
    Stat::SpecialCharge,
    Stat::Luck,
    Stat::GearFind,
];

impl Stat {
    pub fn index(self) -> usize {
        ALL_STATS.iter().position(|s| *s == self).unwrap()
    }
    pub fn name(self) -> &'static str {
        match self {
            Stat::Might => "Damage",
            Stat::MaxHp => "Max Health",
            Stat::Armor => "Armor",
            Stat::Recovery => "Recovery",
            Stat::Cooldown => "Cooldown Reduction",
            Stat::Area => "Area",
            Stat::MoveSpeed => "Move Speed",
            Stat::Magnet => "Magnet",
            Stat::Growth => "Experience",
            Stat::Greed => "Sen Gain",
            Stat::CritChance => "Crit Chance",
            Stat::CritDamage => "Crit Damage",
            Stat::BossDamage => "Boss Damage",
            Stat::SpecialCharge => "Special Charge",
            Stat::Luck => "Luck",
            Stat::GearFind => "Gear Find",
        }
    }
    /// Roll range at item level 1, before rarity.
    fn range(self) -> (f32, f32) {
        match self {
            Stat::Might => (0.02, 0.05),
            Stat::MaxHp => (0.03, 0.07),
            Stat::Armor => (0.3, 1.0),
            Stat::Recovery => (0.05, 0.15),
            Stat::Cooldown => (0.01, 0.03),
            Stat::Area => (0.02, 0.05),
            Stat::MoveSpeed => (0.02, 0.04),
            Stat::Magnet => (0.05, 0.15),
            Stat::Growth => (0.02, 0.05),
            Stat::Greed => (0.04, 0.10),
            Stat::CritChance => (0.01, 0.03),
            Stat::CritDamage => (0.05, 0.15),
            Stat::BossDamage => (0.03, 0.08),
            Stat::SpecialCharge => (0.03, 0.08),
            Stat::Luck => (0.03, 0.08),
            Stat::GearFind => (0.04, 0.10),
        }
    }
    /// Flat stats are shown as numbers, the rest as percentages.
    pub fn flat(self) -> bool {
        matches!(self, Stat::Armor | Stat::Recovery)
    }
    pub fn format(self, v: f32) -> String {
        match self {
            Stat::Armor => format!("+{v:.1} {}", self.name()),
            Stat::Recovery => format!("+{v:.2} HP/s"),
            _ => format!("+{:.1}% {}", v * 100.0, self.name()),
        }
    }
    fn apply(self, b: &mut Bonus, v: f32) {
        match self {
            Stat::Might => b.might += v,
            Stat::MaxHp => b.max_hp += v,
            Stat::Armor => b.armor += v,
            Stat::Recovery => b.recovery += v,
            Stat::Cooldown => b.cooldown += v,
            Stat::Area => b.area += v,
            Stat::MoveSpeed => b.move_speed += v,
            Stat::Magnet => b.magnet += v,
            Stat::Growth => b.growth += v,
            Stat::Greed => b.greed += v,
            Stat::CritChance => b.crit_chance += v,
            Stat::CritDamage => b.crit_damage += v,
            Stat::BossDamage => b.boss_damage += v,
            Stat::SpecialCharge => b.special_charge += v,
            Stat::Luck => b.luck += v,
            Stat::GearFind => b.gear_find += v,
        }
    }
    fn key(self) -> String {
        format!("{self:?}").to_lowercase()
    }
}

/// Value scale for an item level (endless: no cap).
pub fn ilvl_scale(ilvl: u16) -> f32 {
    1.0 + (ilvl.max(1) - 1) as f32 * 0.03
}

/// Legendary and Mythic effects. Mythic versions are twice as strong.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unique {
    /// Below 40% health: more damage.
    Berserker,
    /// Dropping below 30% health grants 3s of invulnerability (long cooldown).
    SecondWind,
    /// Lightning strikes the nearest enemies every few seconds.
    Thunderclap,
    /// A nova ring bursts from you every few seconds.
    Starburst,
    /// Kills heal you.
    Vampiric,
    /// Chests give extra rewards.
    TreasureSense,
    /// Transformations last longer.
    Overflow,
    /// More damage, less health.
    GlassLotus,
    /// More Sen and gear find.
    Fortune,
    /// Blocks a hit every few seconds.
    Aegis,
    /// Get back up once more per run.
    Phoenix,
    /// Crits explode.
    Eclipse,
}

pub const ALL_UNIQUES: [Unique; 12] = [
    Unique::Berserker,
    Unique::SecondWind,
    Unique::Thunderclap,
    Unique::Starburst,
    Unique::Vampiric,
    Unique::TreasureSense,
    Unique::Overflow,
    Unique::GlassLotus,
    Unique::Fortune,
    Unique::Aegis,
    Unique::Phoenix,
    Unique::Eclipse,
];

impl Unique {
    pub fn name(self) -> &'static str {
        match self {
            Unique::Berserker => "Oni's Fury",
            Unique::SecondWind => "Second Wind",
            Unique::Thunderclap => "Raijin's Clap",
            Unique::Starburst => "Starburst",
            Unique::Vampiric => "Hungry Spirit",
            Unique::TreasureSense => "Treasure Sense",
            Unique::Overflow => "Overflowing Soul",
            Unique::GlassLotus => "Glass Lotus",
            Unique::Fortune => "Maneki Fortune",
            Unique::Aegis => "Paper Aegis",
            Unique::Phoenix => "Phoenix Feather",
            Unique::Eclipse => "Eclipse",
        }
    }
    pub fn key(self) -> String {
        format!("{self:?}").to_lowercase()
    }
    /// Description at strength `s` (1 = Legendary, 2 = Mythic).
    pub fn describe(self, s: f32) -> String {
        match self {
            Unique::Berserker => format!("Below 40% health, deal +{:.0}% damage.", 30.0 * s),
            Unique::SecondWind => format!("Falling below 30% health grants 3s of invulnerability (every {:.0}s).", 60.0 / s),
            Unique::Thunderclap => format!("Every 4s, lightning strikes the {} nearest Otherworlders.", (3.0 * s) as u32),
            Unique::Starburst => format!("Every {:.1}s, a starburst nova erupts around you.", 6.0 / s),
            Unique::Vampiric => format!("Every 25 kills heal {:.0}% of max health.", 1.0 * s),
            Unique::TreasureSense => format!("Chests give {} extra reward{}.", s as u32, if s > 1.5 { "s" } else { "" }),
            Unique::Overflow => format!("Transformations last {:.0}s longer.", 2.0 * s),
            Unique::GlassLotus => format!("+{:.0}% damage, but -20% max health.", 35.0 * s),
            Unique::Fortune => format!("+{:.0}% Sen and +{:.0}% gear find.", 50.0 * s, 30.0 * s),
            Unique::Aegis => format!("Blocks a hit every {:.0}s.", 14.0 / s),
            Unique::Phoenix => format!("Get back up {} more time{} per run.", s as u32, if s > 1.5 { "s" } else { "" }),
            Unique::Eclipse => format!("Critical hits explode for {:.0}% of the damage around the target.", 40.0 * s),
        }
    }
}

/// Attacks and summons only an Arsenal item can give (Legendary and up).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArsenalArt {
    /// Thunder drums orbit you and strike the nearest Otherworlders.
    RaijinDrums,
    /// A procession of fox spirits sweeps across the field.
    FoxProcession,
    /// A tengu fan unleashes a giant wind crescent.
    TenguFan,
    /// A sky dragon coils around you, crushing what it touches.
    DragonCoil,
    /// Moons fall on the thickest crowds.
    Moonfall,
    /// A giant skeletal hand slams the densest spot.
    SkeletonHand,
    // Exotic-only.
    /// Two great yin-yang orbs bounce around you, erasing bullets.
    YinYangBreaker,
    /// Shadow clones dash through everything in a star.
    ShadowStar,
    /// A sweeping cut that mows the field and its bullets.
    GrassCutter,
}

pub const LEGENDARY_ARTS: [ArsenalArt; 6] = [ArsenalArt::RaijinDrums, ArsenalArt::FoxProcession, ArsenalArt::TenguFan, ArsenalArt::DragonCoil, ArsenalArt::Moonfall, ArsenalArt::SkeletonHand];
pub const ALL_ARTS: [ArsenalArt; 9] = [
    ArsenalArt::RaijinDrums,
    ArsenalArt::FoxProcession,
    ArsenalArt::TenguFan,
    ArsenalArt::DragonCoil,
    ArsenalArt::Moonfall,
    ArsenalArt::SkeletonHand,
    ArsenalArt::YinYangBreaker,
    ArsenalArt::ShadowStar,
    ArsenalArt::GrassCutter,
];

impl ArsenalArt {
    pub fn name(self) -> &'static str {
        match self {
            ArsenalArt::RaijinDrums => "Raijin's Drum Circle",
            ArsenalArt::FoxProcession => "Night Fox Procession",
            ArsenalArt::TenguFan => "Tengu Gale Fan",
            ArsenalArt::DragonCoil => "Sky Dragon Coil",
            ArsenalArt::Moonfall => "Moonfall",
            ArsenalArt::SkeletonHand => "Gashadokuro's Hand",
            ArsenalArt::YinYangBreaker => "Unbroken Yin-Yang",
            ArsenalArt::ShadowStar => "Thousand Shadow Star",
            ArsenalArt::GrassCutter => "Grass-Cutter Sweep",
        }
    }
    pub fn element(self) -> Element {
        match self {
            ArsenalArt::RaijinDrums => Element::Electric,
            ArsenalArt::FoxProcession => Element::Fire,
            ArsenalArt::TenguFan | ArsenalArt::GrassCutter => Element::Wind,
            ArsenalArt::DragonCoil => Element::Water,
            ArsenalArt::Moonfall => Element::Ice,
            ArsenalArt::SkeletonHand | ArsenalArt::ShadowStar => Element::Darkness,
            ArsenalArt::YinYangBreaker => Element::Light,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            ArsenalArt::RaijinDrums => "raijin_drums",
            ArsenalArt::FoxProcession => "fox_procession",
            ArsenalArt::TenguFan => "tengu_fan",
            ArsenalArt::DragonCoil => "dragon_coil",
            ArsenalArt::Moonfall => "moonfall",
            ArsenalArt::SkeletonHand => "skeleton_hand",
            ArsenalArt::YinYangBreaker => "unbroken_yinyang",
            ArsenalArt::ShadowStar => "clone_scroll",
            ArsenalArt::GrassCutter => "kusanagi",
        }
    }
    /// Description; `mythic` arts are stronger.
    pub fn describe(self, mythic: bool) -> String {
        let m = |a: &str, b: &str| if mythic { b.to_string() } else { a.to_string() };
        match self {
            ArsenalArt::RaijinDrums => format!("{} thunder drums orbit you; each strikes the nearest Otherworlder every 1.2s.", m("3", "5")),
            ArsenalArt::FoxProcession => format!("Every 6s a procession of {} fox spirits sweeps across the field.", m("6", "10")),
            ArsenalArt::TenguFan => format!("Every {}s a giant wind crescent blasts everything in front of you away.", m("3", "2")),
            ArsenalArt::DragonCoil => format!("A sky dragon of {} coils circles you, crushing what it touches.", m("10", "16")),
            ArsenalArt::Moonfall => format!("Every 7s, {} moons fall on the thickest crowds.", m("4", "7")),
            ArsenalArt::SkeletonHand => format!("Every {}s a giant skeletal hand slams the densest spot.", m("6", "4")),
            ArsenalArt::YinYangBreaker => "Two great yin-yang orbs bounce around you, crushing Otherworlders and erasing every bullet they touch.".into(),
            ArsenalArt::ShadowStar => "Every 4s, eight shadow clones dash out in a star, cutting through everything in their path.".into(),
            ArsenalArt::GrassCutter => "Every 4s, a sweeping cut mows down everything around you and cuts their bullets out of the air.".into(),
        }
    }
}

/// Exotic artefacts, one or two per world plus the Three Sacred Treasures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exotic {
    LanternHeart,
    JeweledBranch,
    UnbrokenYinYang,
    NineSeal,
    CloneScroll,
    HollowMask,
    WhiteHaori,
    HungerOrgan,
    ThousandCounter,
    SovereignFinger,
    ClosedHandVow,
    Kusanagi,
    YataMirror,
    Yasakani,
}

pub const ALL_EXOTICS: [Exotic; 14] = [
    Exotic::LanternHeart,
    Exotic::JeweledBranch,
    Exotic::UnbrokenYinYang,
    Exotic::NineSeal,
    Exotic::CloneScroll,
    Exotic::HollowMask,
    Exotic::WhiteHaori,
    Exotic::HungerOrgan,
    Exotic::ThousandCounter,
    Exotic::SovereignFinger,
    Exotic::ClosedHandVow,
    Exotic::Kusanagi,
    Exotic::YataMirror,
    Exotic::Yasakani,
];

/// Fingers count from the whole bag, up to this many.
pub const MAX_FINGERS: u32 = 10;
/// Chance a world's final boss drops one of that world's Exotics, before the difficulty multiplier.
pub const EXOTIC_CHANCE: f32 = 0.0025;
/// Chance a Mini, Boss or Final kill drops one of that world's set pieces, before the difficulty multiplier.
pub const SET_CHANCE: f32 = 0.13;

/// Exotic drop multiplier by difficulty (Normal, Hard, Nightmare); Ascension adds 10% per tier.
pub fn exotic_mult(difficulty: usize, ascension: u32) -> f32 {
    [1.0, 2.0, 4.0][difficulty.min(2)] * (1.0 + 0.1 * ascension as f32)
}

impl Exotic {
    pub fn index(self) -> usize {
        ALL_EXOTICS.iter().position(|e| *e == self).unwrap()
    }
    pub fn name(self) -> &'static str {
        match self {
            Exotic::LanternHeart => "Lantern Daimyo's Heart",
            Exotic::JeweledBranch => "Branch of the Jeweled Moon",
            Exotic::UnbrokenYinYang => "Unbroken Yin-Yang",
            Exotic::NineSeal => "Nine-Seal Scroll",
            Exotic::CloneScroll => "Forbidden Thousand-Clone Scroll",
            Exotic::HollowMask => "Shard of the Hollow Mask",
            Exotic::WhiteHaori => "Captain's White Haori",
            Exotic::HungerOrgan => "Predator's Hunger Organ",
            Exotic::ThousandCounter => "The Thousand Minus Seven",
            Exotic::SovereignFinger => "Sovereign's Severed Finger",
            Exotic::ClosedHandVow => "Vow of the Closed Hand",
            Exotic::Kusanagi => "Kusanagi, the Grass-Cutter",
            Exotic::YataMirror => "Yata no Kagami",
            Exotic::Yasakani => "Yasakani no Magatama",
        }
    }
    pub fn element(self) -> Element {
        match self {
            Exotic::LanternHeart | Exotic::NineSeal => Element::Fire,
            Exotic::JeweledBranch | Exotic::UnbrokenYinYang | Exotic::YataMirror => Element::Light,
            Exotic::CloneScroll | Exotic::HollowMask | Exotic::HungerOrgan | Exotic::SovereignFinger => Element::Darkness,
            Exotic::WhiteHaori => Element::Ice,
            Exotic::ThousandCounter => Element::Water,
            Exotic::ClosedHandVow => Element::Electric,
            Exotic::Kusanagi => Element::Wind,
            Exotic::Yasakani => Element::Earth,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Exotic::LanternHeart => "lantern_heart",
            Exotic::JeweledBranch => "jeweled_branch",
            Exotic::UnbrokenYinYang => "unbroken_yinyang",
            Exotic::NineSeal => "nine_seal",
            Exotic::CloneScroll => "clone_scroll",
            Exotic::HollowMask => "hollow_mask",
            Exotic::WhiteHaori => "white_haori",
            Exotic::HungerOrgan => "hunger_organ",
            Exotic::ThousandCounter => "thousand_counter",
            Exotic::SovereignFinger => "sovereign_finger",
            Exotic::ClosedHandVow => "closed_hand_vow",
            Exotic::Kusanagi => "kusanagi",
            Exotic::YataMirror => "yata_mirror",
            Exotic::Yasakani => "yasakani_jewel",
        }
    }
    pub fn from_key(k: &str) -> Option<Exotic> {
        ALL_EXOTICS.iter().copied().find(|e| e.key() == k)
    }
    pub fn slot(self) -> Slot {
        match self {
            Exotic::LanternHeart => Slot::Charm,
            Exotic::JeweledBranch => Slot::Necklace,
            Exotic::UnbrokenYinYang | Exotic::CloneScroll | Exotic::Kusanagi => Slot::Arsenal,
            Exotic::NineSeal | Exotic::SovereignFinger => Slot::Omamori,
            Exotic::HollowMask => Slot::Earring,
            Exotic::WhiteHaori => Slot::Top,
            Exotic::HungerOrgan => Slot::Bottom,
            Exotic::ThousandCounter | Exotic::ClosedHandVow => Slot::Ring,
            Exotic::YataMirror => Slot::Hat,
            Exotic::Yasakani => Slot::Charm,
        }
    }
    /// The world whose final boss drops it (the only way to get it).
    pub fn home(self) -> StageId {
        match self {
            Exotic::LanternHeart => StageId::Shotengai,
            Exotic::JeweledBranch | Exotic::UnbrokenYinYang | Exotic::YataMirror => StageId::ShrineValley,
            Exotic::NineSeal | Exotic::CloneScroll | Exotic::Kusanagi => StageId::EmberLeaf,
            Exotic::HollowMask | Exotic::WhiteHaori | Exotic::Yasakani => StageId::PaleCitadel,
            Exotic::HungerOrgan | Exotic::ThousandCounter => StageId::Rainward,
            Exotic::SovereignFinger | Exotic::ClosedHandVow => StageId::Shatterline,
        }
    }
    /// One of the Three Sacred Treasures (all three worn: Imperial Regalia).
    pub fn treasure(self) -> bool {
        matches!(self, Exotic::Kusanagi | Exotic::YataMirror | Exotic::Yasakani)
    }
    pub fn art(self) -> Option<ArsenalArt> {
        match self {
            Exotic::UnbrokenYinYang => Some(ArsenalArt::YinYangBreaker),
            Exotic::CloneScroll => Some(ArsenalArt::ShadowStar),
            Exotic::Kusanagi => Some(ArsenalArt::GrassCutter),
            _ => None,
        }
    }
    pub fn describe(self) -> String {
        match self {
            Exotic::LanternHeart => "Every 10s a ring of lantern fire bursts from you. Every 50 kills relight it 1s sooner.".into(),
            Exotic::JeweledBranch => "Enemy bullets grazing past you charge your special. Your special also casts Last Word: every bullet on screen is erased and a starburst devastates the area.".into(),
            Exotic::UnbrokenYinYang => ArsenalArt::YinYangBreaker.describe(true),
            Exotic::NineSeal => "Each special breaks one of nine seals: +10% damage per broken seal. With all nine broken, fox fire burns around you, but you take 30% more damage.".into(),
            Exotic::CloneScroll => ArsenalArt::ShadowStar.describe(true),
            Exotic::HollowMask => "Using your special puts on the mask for 6s: +80% damage. When it breaks, your health drops to half (if above).".into(),
            Exotic::WhiteHaori => "While a boss is on the field, take 40% less damage. +30% boss damage.".into(),
            Exotic::HungerOrgan => "Kills heal 0.4% of max health. If you go 3s without a kill, hunger drains your health.".into(),
            Exotic::ThousandCounter => "Every 7th hit is a critical hit. Every 1,000 kills unleash a crimson blast across the screen.".into(),
            Exotic::SovereignFinger => format!("Counts every Finger you own (up to {MAX_FINGERS}): +6% damage and +4% enemy health each. With 5 or more, your special also opens your own domain."),
            Exotic::ClosedHandVow => "A binding vow: you can never heal, but deal +60% damage with +15% crit chance.".into(),
            Exotic::Kusanagi => ArsenalArt::GrassCutter.describe(true),
            Exotic::YataMirror => "25% of the bullets that would hit you are reflected at the nearest Otherworlder.".into(),
            Exotic::Yasakani => "+1 revival and +20% special charge.".into(),
        }
    }
}

/// Bonus for wearing all Three Sacred Treasures.
pub const REGALIA_TEXT: &str = "Imperial Regalia (all three Treasures): +1 revival, and every 30s every bullet on screen is cleansed.";

/// An Exotic item with its fixed design and six high-quality rolls.
pub fn roll_exotic(rng: &mut Rng, e: Exotic, ilvl: u16) -> Item {
    let mut it = roll_item(rng, Some(e.slot()), Rarity::Exotic, ilvl);
    it.unique = None;
    it.arsenal = None;
    it.art = e.art();
    it.exotic = Some(e);
    it.element = e.element();
    for a in &mut it.affixes {
        a.quality = 0.7 + 0.3 * a.quality;
    }
    it
}

/// Bonuses for wearing pieces of a world's set.
pub fn set_name(s: StageId) -> &'static str {
    match s {
        StageId::Shotengai => "Lantern Street",
        StageId::ShrineValley => "Moonveil",
        StageId::EmberLeaf => "Ember Leaf",
        StageId::PaleCitadel => "Pale Gate",
        StageId::Rainward => "Rainward",
        StageId::Shatterline => "Shatterline",
    }
}

/// The six slots a world's set covers; Rings come as a pair.
pub fn set_pieces(s: StageId) -> [Slot; 6] {
    use Slot::*;
    match s {
        StageId::Shotengai => [Hat, Top, Bottom, Gloves, Shoes, Charm],
        StageId::ShrineValley => [Hat, Top, Necklace, Earring, Ring, Ring],
        StageId::EmberLeaf => [Top, Bottom, Gloves, Shoes, Ring, Ring],
        StageId::PaleCitadel => [Hat, Top, Bottom, Necklace, Earring, Omamori],
        StageId::Rainward => [Gloves, Shoes, Necklace, Omamori, Ring, Ring],
        StageId::Shatterline => [Hat, Gloves, Earring, Charm, Ring, Ring],
    }
}

/// (2-piece, 4-piece, 5-piece, 6-piece) set text.
pub fn set_text(s: StageId) -> [&'static str; 4] {
    let (two, four) = match s {
        StageId::Shotengai => ("+10% Sen gain.", "+10% damage."),
        StageId::ShrineValley => ("+15% area.", "Enemy bullets have a 30% chance to pass through you harmlessly."),
        StageId::EmberLeaf => ("+10% move speed.", "Every 6s an explosive tag lands on nearby Otherworlders. Tags can't hurt you."),
        StageId::PaleCitadel => ("+10% damage.", "Immune to spirit pressure. Kills sometimes release a soul that heals you."),
        StageId::Rainward => ("+20% max health.", "Lightning strikes an Otherworlder near you every 3s. Lightning can't hurt you."),
        StageId::Shatterline => ("+8% cooldown reduction.", "Immune to hostile domains, and +25% damage."),
    };
    let six = match s {
        StageId::Shotengai => "Burning Otherworlders pass their flames on when they fall.",
        StageId::ShrineValley => "Staggered Otherworlders take 30% more damage.",
        StageId::EmberLeaf => "Wind hits trigger their effect twice as often.",
        StageId::PaleCitadel => "Slowed Otherworlders shatter when they fall, hurting those nearby.",
        StageId::Rainward => "Every Water hit soaks. Soaked Otherworlders take 30% more Electric damage.",
        StageId::Shatterline => "Electric chains jump to one more Otherworlder.",
    };
    let five = match s.element() {
        Element::Fire => "+25% Fire damage and +25% Fire resistance.",
        Element::Water => "+25% Water damage and +25% Water resistance.",
        Element::Electric => "+25% Electric damage and +25% Electric resistance.",
        Element::Earth => "+25% Earth damage and +25% Earth resistance.",
        Element::Wind => "+25% Wind damage and +25% Wind resistance.",
        Element::Ice => "+25% Ice damage and +25% Ice resistance.",
        Element::Light => "+25% Light damage and +25% Light resistance.",
        Element::Darkness => "+25% Darkness damage and +25% Darkness resistance.",
    };
    [two, four, five, six]
}

fn set_two(s: StageId, b: &mut Bonus) {
    match s {
        StageId::Shotengai => b.greed += 0.10,
        StageId::ShrineValley => b.area += 0.15,
        StageId::EmberLeaf => b.move_speed += 0.10,
        StageId::PaleCitadel => b.might += 0.10,
        StageId::Rainward => b.max_hp += 0.20,
        StageId::Shatterline => b.cooldown += 0.08,
    }
}

/// Weapons an Arsenal item can grant (summons and autonomous attacks).
pub const ARSENAL_WEAPONS: [WeaponKind; 10] = [
    WeaponKind::Familiars,
    WeaponKind::FoxMirage,
    WeaponKind::LanternVigil,
    WeaponKind::GravityWell,
    WeaponKind::ThunderChain,
    WeaponKind::StarFall,
    WeaponKind::FlamePool,
    WeaponKind::MoonOrbs,
    WeaponKind::Taiko,
    WeaponKind::YinYangOrbs,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affix {
    pub stat: Stat,
    /// Quality within the roll range, 0..=1.
    pub quality: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub id: u32,
    pub slot: Slot,
    pub rarity: Rarity,
    pub ilvl: u16,
    /// Upgrade level, 0..=10.
    pub plus: u8,
    pub set: Option<StageId>,
    /// Which base name (index into the slot's list).
    pub base: u8,
    pub affixes: Vec<Affix>,
    pub unique: Option<Unique>,
    /// Low-rarity Arsenal: a library weapon.
    pub arsenal: Option<WeaponKind>,
    /// Legendary+ Arsenal: an exclusive attack or summon.
    pub art: Option<ArsenalArt>,
    pub exotic: Option<Exotic>,
    pub locked: bool,
    pub element: Element,
}

pub const MAX_PLUS: u8 = 10;
/// Element damage and resistance per item, before the item's scale.
pub const ELEM_ITEM: f32 = 0.02;
pub const RESIST_CAP: f32 = 0.60;

/// The one element rule, for fresh rolls (`r` from the rng) and old saves (`r` from the item id).
fn pick_element(it: &Item, r: u32) -> Element {
    if let Some(e) = it.exotic {
        return e.element();
    }
    if let Some(k) = it.arsenal {
        return k.element();
    }
    if let Some(a) = it.art {
        return a.element();
    }
    if let (Rarity::Set, Some(s)) = (it.rarity, it.set) {
        return s.element();
    }
    if it.rarity >= Rarity::Legendary && r % 100 < 10 {
        return if r & 0x100 == 0 { Element::Light } else { Element::Darkness };
    }
    ALL_ELEMENTS[(r % 6) as usize]
}

impl Item {
    fn scale(&self) -> f32 {
        ilvl_scale(self.ilvl) * self.rarity.mult() * (1.0 + 0.06 * self.plus as f32)
    }
    pub fn elem_value(&self) -> f32 {
        ELEM_ITEM * self.scale()
    }
    pub fn main_value(&self) -> Option<(Stat, f32)> {
        let st = self.slot.main_stat()?;
        let (lo, hi) = st.range();
        Some((st, (lo + hi) * 1.25 * self.scale()))
    }
    pub fn affix_value(&self, a: &Affix) -> f32 {
        let (lo, hi) = a.stat.range();
        (lo + (hi - lo) * a.quality) * self.scale()
    }
    /// Average roll quality of the bonuses (0..1).
    pub fn quality(&self) -> f32 {
        if self.affixes.is_empty() {
            return 1.0;
        }
        self.affixes.iter().map(|a| a.quality).sum::<f32>() / self.affixes.len() as f32
    }
    pub fn unique_strength(&self) -> f32 {
        if self.rarity >= Rarity::Mythic {
            2.0
        } else {
            1.0
        }
    }
    pub fn name(&self) -> String {
        if let Some(e) = self.exotic {
            return if self.plus > 0 { format!("{} +{}", e.name(), self.plus) } else { e.name().to_string() };
        }
        if let Some(a) = self.art {
            return if self.plus > 0 { format!("{} +{}", a.name(), self.plus) } else { a.name().to_string() };
        }
        let base = match self.arsenal {
            Some(k) => crate::content::Upgrade::Weapon(k).name().to_string(),
            None => self.slot.bases()[self.base as usize % self.slot.bases().len()].to_string(),
        };
        let mut n = match (self.unique, self.set) {
            (_, Some(s)) if self.rarity == Rarity::Set => format!("{} {}", set_name(s), base),
            (Some(u), _) => format!("{} {}", u.name(), base),
            (None, Some(s)) => format!("{} {}", set_name(s), base),
            _ => base,
        };
        if self.plus > 0 {
            n = format!("{n} +{}", self.plus);
        }
        n
    }
    /// Gear score (sort key / at-a-glance power).
    pub fn score(&self) -> u32 {
        let r = (self.rarity.index() + 1) as f32;
        (self.ilvl as f32 * 4.0 + r * r * 12.0 + self.quality() * 40.0 + self.plus as f32 * 15.0 + self.unique.is_some() as u32 as f32 * 60.0) as u32
    }
    /// Spirit Dust to upgrade to the next plus level.
    pub fn upgrade_cost(&self) -> u32 {
        (self.rarity.dust() * 2 + self.ilvl as u32) * (self.plus as u32 + 1)
    }
    /// Spirit Dust to reroll one bonus.
    pub fn reroll_cost(&self) -> u32 {
        self.rarity.dust() * 3 + self.ilvl as u32 / 2
    }
    pub fn salvage(&self) -> u32 {
        self.rarity.dust() * (1 + self.ilvl as u32 / 10) + self.plus as u32 * 3
    }

    /// Re-rolls bonus `i`: a new stat (not already on the item) and a new quality.
    pub fn reroll(&mut self, i: usize, rng: &mut Rng) {
        if i >= self.affixes.len() {
            return;
        }
        let taken: Vec<Stat> = self.affixes.iter().enumerate().filter(|(k, _)| *k != i).map(|(_, a)| a.stat).chain(self.slot.main_stat()).collect();
        let pool: Vec<Stat> = ALL_STATS.iter().copied().filter(|s| !taken.contains(s)).collect();
        let quality = if self.rarity >= Rarity::Exotic { 0.7 + 0.3 * rng.f32() } else { rng.f32() };
        self.affixes[i] = Affix { stat: pool[(rng.next_u32() as usize) % pool.len()], quality };
    }

    pub fn serialize(&self) -> String {
        let affixes: Vec<String> = self.affixes.iter().map(|a| format!("{}:{:.3}", a.stat.key(), a.quality)).collect();
        format!(
            "{};{};{};{};{};{};{};{};{};{};{};{};{};{}",
            self.id,
            self.slot.key(),
            self.rarity.index(),
            self.ilvl,
            self.plus,
            self.set.map_or("-", |s| s.key()),
            self.base,
            affixes.join(","),
            self.unique.map_or("-".to_string(), |u| u.key()),
            self.arsenal.map_or("-".to_string(), |w| format!("{w:?}").to_lowercase()),
            self.locked as u8,
            self.art.map_or("-", |a| a.key()),
            self.exotic.map_or("-", |e| e.key()),
            self.element.key(),
        )
    }

    pub fn parse(s: &str) -> Option<Item> {
        let f: Vec<&str> = s.split(';').collect();
        if f.len() < 11 {
            return None;
        }
        let affixes = f[7]
            .split(',')
            .filter(|x| !x.is_empty())
            .filter_map(|x| {
                let (k, q) = x.split_once(':')?;
                let stat = ALL_STATS.iter().copied().find(|s| s.key() == k)?;
                Some(Affix { stat, quality: q.parse::<f32>().ok()?.clamp(0.0, 1.0) })
            })
            .collect();
        let mut item = Item {
            id: f[0].parse().ok()?,
            slot: Slot::from_key(f[1])?,
            rarity: *ALL_RARITIES.get(f[2].parse::<usize>().ok()?)?,
            ilvl: f[3].parse().ok()?,
            plus: f[4].parse::<u8>().ok()?.min(MAX_PLUS),
            set: StageId::from_key(f[5]),
            base: f[6].parse().ok()?,
            affixes,
            unique: ALL_UNIQUES.iter().copied().find(|u| u.key() == f[8]),
            arsenal: crate::content::ALL_WEAPONS.iter().copied().find(|w| format!("{w:?}").to_lowercase() == f[9]),
            locked: f[10] == "1",
            art: f.get(11).and_then(|k| ALL_ARTS.iter().copied().find(|a| a.key() == *k)),
            exotic: f.get(12).and_then(|k| Exotic::from_key(k)),
            element: Element::Fire,
        };
        item.element = f.get(13).and_then(|k| Element::from_key(k)).unwrap_or_else(|| pick_element(&item, crate::stage::hash(item.id as i32, 7, 0xE1E)));
        Some(item)
    }
}

/// Rolls a rarity: harder content (`tier`) and gear find shift weight to the top.
pub fn roll_rarity(rng: &mut Rng, tier: f32, find: f32, min: Rarity) -> Rarity {
    let base = [55.0f32, 27.0, 12.0, 4.5, 1.3, 0.2];
    let mut w = [0.0f32; 6];
    let min = min.min(Rarity::Mythic);
    for (r, b) in base.iter().enumerate() {
        let shift = (1.0 + 0.25 * tier).powi(r as i32);
        let find = if r >= 2 { 1.0 + find } else { 1.0 };
        w[r] = if r < min.index() { 0.0 } else { b * shift * find };
    }
    let total: f32 = w.iter().sum();
    let mut x = rng.f32() * total;
    for (r, wt) in w.iter().enumerate() {
        if x < *wt {
            return ALL_RARITIES[r];
        }
        x -= wt;
    }
    Rarity::Mythic
}

/// A fresh item. `slot` None = any slot.
pub fn roll_item(rng: &mut Rng, slot: Option<Slot>, rarity: Rarity, ilvl: u16) -> Item {
    let slot = slot.unwrap_or(ALL_SLOTS[(rng.next_u32() as usize) % ALL_SLOTS.len()]);
    let mut item = Item {
        id: 0,
        slot,
        rarity,
        ilvl: ilvl.max(1),
        plus: 0,
        set: None,
        base: (rng.next_u32() % 8) as u8,
        affixes: Vec::new(),
        unique: None,
        arsenal: None,
        art: None,
        exotic: None,
        locked: false,
        element: Element::Fire,
    };
    if slot == Slot::Arsenal {
        // Legendary and Mythic Arsenals carry attacks found nowhere else.
        if rarity >= Rarity::Legendary {
            item.art = Some(LEGENDARY_ARTS[(rng.next_u32() as usize) % LEGENDARY_ARTS.len()]);
        } else {
            item.arsenal = Some(ARSENAL_WEAPONS[(rng.next_u32() as usize) % ARSENAL_WEAPONS.len()]);
        }
    }
    let mut pool: Vec<Stat> = ALL_STATS.iter().copied().filter(|s| Some(*s) != slot.main_stat()).collect();
    for _ in 0..rarity.affixes() {
        let i = (rng.next_u32() as usize) % pool.len();
        let stat = pool.swap_remove(i);
        item.affixes.push(Affix { stat, quality: rng.f32() });
    }
    if rarity >= Rarity::Legendary && slot != Slot::Arsenal {
        item.unique = Some(ALL_UNIQUES[(rng.next_u32() as usize) % ALL_UNIQUES.len()]);
    }
    item.element = pick_element(&item, rng.next_u32());
    item
}

/// A random piece of a world's set, with Exotic-level rolls.
pub fn roll_set_piece(rng: &mut Rng, s: StageId, ilvl: u16) -> Item {
    let slot = set_pieces(s)[(rng.next_u32() % 6) as usize];
    let mut it = roll_item(rng, Some(slot), Rarity::Set, ilvl);
    it.set = Some(s);
    it.element = pick_element(&it, 0);
    for a in &mut it.affixes {
        a.quality = 0.7 + 0.3 * a.quality;
    }
    it
}

/// Item level of drops for a stage, difficulty and Ascension tier.
pub fn drop_ilvl(stage: StageId, difficulty: usize, ascension: u32) -> u16 {
    (1 + stage.index() * 6 + difficulty * 12 + ascension as usize * 4) as u16
}

/// How far up the rarity table content sits (feeds `roll_rarity`).
pub fn drop_tier(stage: StageId, difficulty: usize, ascension: u32) -> f32 {
    difficulty as f32 * 2.0 + stage.index() as f32 * 0.5 + ascension as f32 * 0.6
}

/// Shrine Draw prices and pity.
pub const DRAW_COST: u32 = 250;
pub const DRAW10_COST: u32 = 2250;
/// Draws without a Legendary or better before one is guaranteed.
pub const DRAW_PITY: u32 = 50;
/// Shrine Draw odds (percent), in rarity order.
pub const DRAW_ODDS: [f32; 6] = [40.0, 30.0, 18.0, 9.0, 2.7, 0.3];

pub fn draw_rarity(rng: &mut Rng, pity: u32) -> Rarity {
    if pity + 1 >= DRAW_PITY {
        return if rng.f32() < 0.1 { Rarity::Mythic } else { Rarity::Legendary };
    }
    let mut x = rng.f32() * 100.0;
    for (r, p) in DRAW_ODDS.iter().enumerate() {
        if x < *p {
            return ALL_RARITIES[r];
        }
        x -= p;
    }
    Rarity::Mythic
}

/// (legacy tagged pieces, Set pieces) of world `s` worn. A Set ring counts only with its pair.
pub fn set_count(items: &[&Item], s: StageId) -> (u32, u32) {
    let mine = |it: &&&Item| it.set == Some(s);
    let legacy = items.iter().filter(mine).filter(|it| it.rarity < Rarity::Set).count() as u32;
    let set_items = || items.iter().filter(mine).filter(|it| it.rarity == Rarity::Set);
    let rings = set_items().filter(|it| it.slot == Slot::Ring).count();
    let others = set_items().filter(|it| it.slot != Slot::Ring).count() as u32;
    (legacy, others + if rings >= 2 { 2 } else { 0 })
}

/// Everything a run needs from the hero's equipped gear.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Loadout {
    pub bonus: Bonus,
    /// (unique, strength) of every Legendary / Mythic worn.
    pub uniques: Vec<(Unique, f32)>,
    /// Worlds whose 4-piece set bonus is active.
    pub sets4: Vec<StageId>,
    /// Worlds whose 6-piece set bonus is active.
    pub sets6: Vec<StageId>,
    /// Worlds whose 5-piece bonus is active (new Set pieces only).
    pub sets5: Vec<StageId>,
    /// The Arsenal weapon: (kind, level, awakened, damage power).
    pub arsenal: Option<(WeaponKind, u8, bool, f32)>,
    /// An exclusive Arsenal attack: (art, damage power, mythic-strength).
    pub art: Option<(ArsenalArt, f32, bool)>,
    pub exotics: Vec<Exotic>,
    /// Sovereign's Fingers owned (counts only with one worn).
    pub fingers: u32,
    /// Element damage and resistance from gear, indexed by `Element as usize`.
    pub elem_damage: [f32; 8],
    pub elem_resist: [f32; 8],
}

impl Loadout {
    pub fn from_items(items: &[&Item]) -> Loadout {
        let mut l = Loadout::default();
        for it in items {
            if let Some((st, v)) = it.main_value() {
                st.apply(&mut l.bonus, v);
            }
            for a in &it.affixes {
                a.stat.apply(&mut l.bonus, it.affix_value(a));
            }
            l.elem_damage[it.element as usize] += it.elem_value();
            l.elem_resist[it.element as usize] += it.elem_value();
            if let Some(u) = it.unique {
                l.uniques.push((u, it.unique_strength()));
            }
            let power = ilvl_scale(it.ilvl) * (1.0 + 0.06 * it.plus as f32);
            if let Some(k) = it.arsenal {
                l.arsenal = Some((k, it.rarity.arsenal_level(), it.rarity >= Rarity::Mythic, power));
            }
            if let Some(a) = it.art {
                l.art = Some((a, power * if it.rarity == Rarity::Exotic { 1.4 } else { 1.0 }, it.rarity >= Rarity::Mythic));
            }
            if let Some(e) = it.exotic {
                l.exotics.push(e);
                match e {
                    Exotic::WhiteHaori => l.bonus.boss_damage += 0.30,
                    Exotic::ClosedHandVow => {
                        l.bonus.might += 0.60;
                        l.bonus.crit_chance += 0.15;
                    }
                    Exotic::Yasakani => {
                        l.bonus.revivals += 1;
                        l.bonus.special_charge += 0.20;
                    }
                    _ => {}
                }
            }
        }
        if l.regalia() {
            l.bonus.revivals += 1;
        }
        for s in crate::campaign::ALL_STAGES {
            let (legacy, new) = set_count(items, s);
            let n = legacy + new;
            if n >= 2 {
                set_two(s, &mut l.bonus);
            }
            if new >= 5 {
                l.elem_damage[s.element() as usize] += 0.25;
                l.elem_resist[s.element() as usize] += 0.25;
                l.sets5.push(s);
            }
            if new >= 6 {
                l.sets6.push(s);
            }
            if n >= 4 {
                l.sets4.push(s);
                if s == StageId::Shotengai {
                    l.bonus.might += 0.10;
                }
                if s == StageId::Shatterline {
                    l.bonus.might += 0.25;
                }
            }
        }
        l
    }

    pub fn resist(&self, e: Element) -> f32 {
        self.elem_resist[e as usize].min(RESIST_CAP)
    }

    pub fn has(&self, e: Exotic) -> bool {
        self.exotics.contains(&e)
    }
    /// All Three Sacred Treasures worn.
    pub fn regalia(&self) -> bool {
        self.has(Exotic::Kusanagi) && self.has(Exotic::YataMirror) && self.has(Exotic::Yasakani)
    }

    pub fn unique(&self, u: Unique) -> f32 {
        self.uniques.iter().filter(|(x, _)| *x == u).map(|(_, s)| *s).sum()
    }
    pub fn has_set(&self, s: StageId) -> bool {
        self.sets4.contains(&s)
    }
    pub fn has_set6(&self, s: StageId) -> bool {
        self.sets6.contains(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_collection_takes_about_forty_runs() {
        const PLAYERS: u32 = 2000;
        let chance = SET_CHANCE * exotic_mult(0, 0);
        let mut rng = Rng::new(40);
        let mut total_runs = 0u32;
        for _ in 0..PLAYERS {
            let mut owned = 0u8;
            while owned != 0b11_1111 {
                total_runs += 1;
                // Measured: riku, shrine_valley, Normal, seeds 21-30 gave 3 eligible kills in 8 runs of 10, 2 in the other 2.
                let kills = if total_runs % 10 < 2 { 2 } else { 3 };
                for _ in 0..kills {
                    if rng.f32() < chance {
                        owned |= 1 << (rng.next_u32() % 6);
                    }
                }
            }
        }
        let mean = total_runs as f32 / PLAYERS as f32;
        assert!((35.0..=46.0).contains(&mean), "mean runs to a full set: {mean}");
    }

    #[test]
    fn rarity_sets_the_number_of_bonuses_and_items_round_trip() {
        let mut rng = Rng::new(7);
        for r in &ALL_RARITIES[..6] {
            let r = *r;
            for _ in 0..50 {
                let mut it = roll_item(&mut rng, None, r, 30);
                assert_eq!(it.affixes.len(), r.affixes());
                assert_eq!(it.unique.is_some(), r >= Rarity::Legendary && it.slot != Slot::Arsenal);
                assert_eq!(it.arsenal.is_some(), it.slot == Slot::Arsenal && r < Rarity::Legendary);
                assert_eq!(it.art.is_some(), it.slot == Slot::Arsenal && r >= Rarity::Legendary);
                let mut stats: Vec<Stat> = it.affixes.iter().map(|a| a.stat).collect();
                stats.extend(it.slot.main_stat());
                let n = stats.len();
                stats.sort_by_key(|s| s.index());
                stats.dedup();
                assert_eq!(stats.len(), n, "no duplicate stats on one item");
                it.id = 42;
                it.plus = 3;
                let back = Item::parse(&it.serialize()).unwrap();
                assert_eq!(back.slot, it.slot);
                assert_eq!(back.affixes.len(), it.affixes.len());
                assert!((back.quality() - it.quality()).abs() < 0.01);
                assert_eq!(back.unique, it.unique);
                assert_eq!(back.arsenal, it.arsenal);
                assert_eq!(back.art, it.art);
            }
        }
        for e in ALL_EXOTICS {
            let it = roll_exotic(&mut rng, e, 50);
            assert_eq!(it.slot, e.slot());
            assert!(it.affixes.len() == 6 && it.affixes.iter().all(|a| a.quality >= 0.7));
            let back = Item::parse(&it.serialize()).unwrap();
            assert_eq!(back.exotic, Some(e));
            assert_eq!(back.art, e.art());
        }
    }

    #[test]
    fn higher_item_levels_and_upgrades_are_stronger() {
        let mut rng = Rng::new(3);
        let mut a = roll_item(&mut rng, Some(Slot::Gloves), Rarity::Epic, 1);
        let lo = a.main_value().unwrap().1;
        a.ilvl = 80;
        let hi = a.main_value().unwrap().1;
        a.plus = 10;
        assert!(hi > lo * 3.0 && a.main_value().unwrap().1 > hi);
    }

    #[test]
    fn harder_content_rolls_better_and_pity_guarantees() {
        let mut rng = Rng::new(9);
        let avg = |rng: &mut Rng, t: f32| (0..4000).map(|_| roll_rarity(rng, t, 0.0, Rarity::Common).index()).sum::<usize>() as f32 / 4000.0;
        let (easy, hard) = (avg(&mut rng, 0.0), avg(&mut rng, 6.0));
        assert!(hard > easy + 0.5, "{easy} vs {hard}");
        assert!(draw_rarity(&mut rng, DRAW_PITY - 1) >= Rarity::Legendary);
        assert!((DRAW_ODDS.iter().sum::<f32>() - 100.0).abs() < 0.01);
        assert!((0..20_000).all(|_| roll_rarity(&mut rng, 30.0, 5.0, Rarity::Common) != Rarity::Exotic), "Exotics never roll as plain drops");
    }

    #[test]
    fn sets_and_uniques_reach_the_loadout() {
        let mut rng = Rng::new(1);
        let mut items = Vec::new();
        for s in [Slot::Hat, Slot::Top, Slot::Bottom, Slot::Shoes] {
            let mut it = roll_item(&mut rng, Some(s), Rarity::Legendary, 20);
            it.set = Some(StageId::Shatterline);
            items.push(it);
        }
        let refs: Vec<&Item> = items.iter().collect();
        let l = Loadout::from_items(&refs);
        assert!(l.has_set(StageId::Shatterline));
        assert_eq!(l.uniques.len(), 4);
        assert!(l.bonus.max_hp > 0.0 && l.bonus.armor > 0.0);
    }

    fn elemental(rng: &mut Rng, slot: Option<Slot>, rarity: Rarity, e: Element) -> Item {
        let mut it = roll_item(rng, slot, rarity, 100);
        it.element = e;
        it
    }

    #[test]
    fn item_elements_follow_the_rules() {
        let mut rng = Rng::new(11);
        for r in &ALL_RARITIES[..6] {
            let (mut rare, mut plain) = (0, 0);
            for _ in 0..2000 {
                let it = roll_item(&mut rng, None, *r, 30);
                if let Some(k) = it.arsenal {
                    assert_eq!(it.element, k.element());
                } else if let Some(a) = it.art {
                    assert_eq!(it.element, a.element());
                } else if *r < Rarity::Legendary {
                    assert!(!it.element.is_rare(), "{r:?} rolled {:?}", it.element);
                } else {
                    plain += 1;
                    rare += it.element.is_rare() as u32;
                }
                assert_eq!(Item::parse(&it.serialize()).unwrap().element, it.element);
            }
            if *r >= Rarity::Legendary {
                let share = rare as f32 / plain as f32;
                assert!((0.05..=0.15).contains(&share), "{r:?} rare share {share}");
            }
        }
        for e in ALL_EXOTICS {
            let it = roll_exotic(&mut rng, e, 50);
            assert_eq!(it.element, e.element());
            assert_eq!(Item::parse(&it.serialize()).unwrap().element, e.element());
        }
    }

    #[test]
    fn old_items_get_a_stable_element() {
        let mut rng = Rng::new(5);
        for _ in 0..200 {
            let mut it = roll_item(&mut rng, None, Rarity::Rare, 20);
            it.id = rng.next_u32() % 5000;
            let line = it.serialize();
            let old = line.rsplitn(2, ';').nth(1).unwrap();
            let a = Item::parse(old).unwrap();
            let b = Item::parse(old).unwrap();
            assert_eq!(a.element, b.element);
            assert_eq!(a.element, pick_element(&a, crate::stage::hash(a.id as i32, 7, 0xE1E)));
            let saved = a.serialize();
            assert_eq!(Item::parse(&saved).unwrap().serialize(), saved);
            let pre_art: String = line.split(';').take(11).collect::<Vec<_>>().join(";");
            assert!(Item::parse(&pre_art).is_some());
        }
    }

    #[test]
    fn loadout_sums_item_elements() {
        let mut rng = Rng::new(2);
        let mut items: Vec<Item> = (0..3).map(|_| elemental(&mut rng, Some(Slot::Hat), Rarity::Rare, Element::Fire)).collect();
        items.push(elemental(&mut rng, Some(Slot::Top), Rarity::Rare, Element::Ice));
        let refs: Vec<&Item> = items.iter().collect();
        let l = Loadout::from_items(&refs);
        let fire: f32 = items[..3].iter().map(|i| i.elem_value()).sum();
        assert!((l.elem_damage[Element::Fire as usize] - fire).abs() < 1e-4);
        assert!(l.resist(Element::Ice) > 0.0);
        assert_eq!(l.resist(Element::Water), 0.0);

        let many: Vec<Item> = (0..12).map(|_| elemental(&mut rng, Some(Slot::Hat), Rarity::Mythic, Element::Fire)).collect();
        let refs: Vec<&Item> = many.iter().collect();
        let l = Loadout::from_items(&refs);
        assert!(l.elem_resist[Element::Fire as usize] > RESIST_CAP);
        assert_eq!(l.resist(Element::Fire), RESIST_CAP);
    }

    fn set_piece(rng: &mut Rng, s: StageId, slot: Slot) -> Item {
        let mut it = roll_item(rng, Some(slot), Rarity::Set, 20);
        it.set = Some(s);
        it.element = s.element();
        it
    }

    fn legacy_piece(rng: &mut Rng, s: StageId, slot: Slot) -> Item {
        let mut it = roll_item(rng, Some(slot), Rarity::Legendary, 20);
        it.set = Some(s);
        it
    }

    fn items_value(items: &[Item], e: Element) -> f32 {
        items.iter().filter(|i| i.element == e).map(|i| i.elem_value()).sum()
    }

    #[test]
    fn set_pieces_match_the_spec() {
        let mut rng = Rng::new(3);
        for s in crate::campaign::ALL_STAGES {
            let pieces = set_pieces(s);
            assert_eq!(pieces.len(), 6);
            assert!(!pieces.contains(&Slot::Arsenal));
            let rings = pieces.iter().filter(|p| **p == Slot::Ring).count();
            assert!(rings == 0 || rings == 2);
            for _ in 0..600 {
                let mut it = roll_set_piece(&mut rng, s, 30);
                assert!(pieces.contains(&it.slot));
                assert_eq!(it.rarity, Rarity::Set);
                assert_eq!(it.set, Some(s));
                assert_eq!(it.element, s.element());
                assert!(it.unique.is_some());
                assert_eq!(it.affixes.len(), 6);
                assert!(it.affixes.iter().all(|a| a.quality >= 0.7));
                it.id = 9;
                let back = Item::parse(&it.serialize()).unwrap();
                assert_eq!(back.rarity, Rarity::Set);
                assert_eq!((back.slot, back.set, back.element, back.unique), (it.slot, it.set, it.element, it.unique));
            }
        }
        let mut it = roll_item(&mut rng, None, Rarity::Epic, 10);
        assert_eq!(it.set, None);
        it = roll_item(&mut rng, None, Rarity::Mythic, 10);
        assert_eq!(it.set, None);
        assert!((0..20_000).all(|_| roll_rarity(&mut rng, 30.0, 5.0, Rarity::Common) != Rarity::Set));
        assert!((0..20_000).all(|_| draw_rarity(&mut rng, 0) != Rarity::Set));
    }

    #[test]
    fn set_rings_count_only_as_a_pair() {
        let mut rng = Rng::new(4);
        let s = StageId::ShrineValley;
        let mut items: Vec<Item> = [Slot::Hat, Slot::Top, Slot::Necklace, Slot::Earring, Slot::Ring].iter().map(|sl| set_piece(&mut rng, s, *sl)).collect();
        let l = Loadout::from_items(&items.iter().collect::<Vec<_>>());
        assert!(l.has_set(s));
        assert!(!l.has_set6(s));
        assert!((l.elem_damage[Element::Earth as usize] - items_value(&items, Element::Earth)).abs() < 1e-4, "no 5-piece bonus");
        items.push(set_piece(&mut rng, s, Slot::Ring));
        let l = Loadout::from_items(&items.iter().collect::<Vec<_>>());
        assert!(l.has_set6(s));
        assert!((l.elem_damage[Element::Earth as usize] - items_value(&items, Element::Earth) - 0.25).abs() < 1e-4);
        let lone = [legacy_piece(&mut rng, s, Slot::Hat), legacy_piece(&mut rng, s, Slot::Ring)];
        let l = Loadout::from_items(&[&lone[0], &lone[1]]);
        let bare = Loadout::from_items(&[&roll_item(&mut rng, Some(Slot::Hat), Rarity::Legendary, 20), &roll_item(&mut rng, Some(Slot::Ring), Rarity::Legendary, 20)]);
        assert!(l.bonus.area - bare.bonus.area > 0.14, "legacy ring still counts toward the 2-piece");
    }

    #[test]
    fn legacy_pieces_stop_at_four() {
        let mut rng = Rng::new(5);
        let s = StageId::Shotengai;
        let mut items = vec![legacy_piece(&mut rng, s, Slot::Hat)];
        items.extend([Slot::Top, Slot::Bottom, Slot::Gloves, Slot::Shoes, Slot::Charm].iter().map(|sl| set_piece(&mut rng, s, *sl)));
        let l = Loadout::from_items(&items.iter().collect::<Vec<_>>());
        assert!(l.has_set(s));
        assert!(!l.has_set6(s));
        assert!(l.sets5.contains(&s), "5 Set pieces fill sets5");
        let fire = s.element();
        let base = items_value(&items, fire);
        assert!((l.elem_damage[fire as usize] - base - 0.25).abs() < 1e-4, "5 pieces add exactly +0.25 damage");
        assert!((l.elem_resist[fire as usize] - base - 0.25).abs() < 1e-4, "5 pieces add exactly +0.25 resist");
        let mut mixed: Vec<Item> = [Slot::Hat, Slot::Top, Slot::Bottom, Slot::Gloves].iter().map(|sl| legacy_piece(&mut rng, s, *sl)).collect();
        mixed.extend([Slot::Shoes, Slot::Charm].iter().map(|sl| set_piece(&mut rng, s, *sl)));
        let l = Loadout::from_items(&mixed.iter().collect::<Vec<_>>());
        assert!(l.has_set(s));
        assert!((l.elem_damage[fire as usize] - items_value(&mixed, fire)).abs() < 1e-4, "2 Set pieces give no 5-piece");
        let (legacy, new) = set_count(&mixed.iter().collect::<Vec<_>>(), s);
        assert_eq!((legacy, new), (4, 2));
    }

    #[test]
    fn reroll_keeps_exotic_quality_floor() {
        let mut rng = Rng::new(11);
        for rarity in [Rarity::Exotic, Rarity::Set] {
            let mut it = roll_item(&mut rng, Some(Slot::Hat), rarity, 30);
            for _ in 0..200 {
                it.reroll(0, &mut rng);
                assert!(it.affixes[0].quality >= 0.7, "{rarity:?} reroll fell to {}", it.affixes[0].quality);
            }
        }
    }

    #[test]
    fn set_prices_above_exotic() {
        assert!(Rarity::Set.dust() > Rarity::Exotic.dust());
        let mut rng = Rng::new(6);
        let ex = roll_item(&mut rng, Some(Slot::Hat), Rarity::Exotic, 30);
        let st = roll_item(&mut rng, Some(Slot::Hat), Rarity::Set, 30);
        assert!(st.upgrade_cost() > ex.upgrade_cost());
        assert!(st.reroll_cost() > ex.reroll_cost());
        assert!(st.salvage() > ex.salvage());
    }
}
