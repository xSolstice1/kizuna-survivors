//! Every Otherworlder: names, lore, stats, behaviour archetypes and boss move sets.
//! Stages (see `campaign`) pick their roster from here; the sim drives each enemy by its
//! `Ai`, so a new enemy is a few lines of data plus a sprite.

use crate::content::Element;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    // ── Prologue: Neon Shotengai ──
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
    RokurokubiClerk,
    TsurubeDrop,
    CaptainKitsunebi,
    // ── World 1: Hidden Shrine Valley ──
    Kodama,
    PaperCrane,
    LanternFairy,
    FoxMaskPhantom,
    Komainu,
    BellMaiden,
    ToriiSerpent,
    Mitsuhime,
    ShardMoth,
    BoneEaterCrow,
    AdmiralTengu,
    // ── World 2: Ember Leaf Frontier ──
    StrawPuppet,
    LeafShinobi,
    KunaiThrower,
    SmokeNinja,
    IronRonin,
    Kagerou,
    EmberToad,
    BurningShogun,
    RoguePuppeteer,
    ClayBird,
    MistCaptain,
    // ── World 3: Pale Citadel of Souls ──
    LostSoul,
    MaskedHound,
    PaleArcher,
    ShadeDancer,
    BoneSentinel,
    GateCaptain,
    HollowMonarch,
    PaleSeraph,
    MaskedRetainer,
    HollowHusk,
    NumberedQuartermaster,
    // ── World 4: Rainward 13 ──
    DrownedRat,
    RainGhoul,
    GutterSpitter,
    UmbrellaStalker,
    WardButcher,
    NightInvestigator,
    GutterMother,
    CrimsonRainLord,
    BladetailRunner,
    MaskCollector,
    DrownedCaptain,
    // ── World 5: Shatterline Crossing ──
    CursedSpawn,
    WailingCommuter,
    CurseEye,
    GlitchShade,
    GradeOneBrute,
    PatchworkSorcerer,
    CrossingCalamity,
    ShatterSovereign,
    SwarmMother,
    ToolThief,
    CurseCaptain,
}

use EnemyKind::*;

pub const ALL_ENEMIES: [EnemyKind; 65] = [
    Wisp, Kasa, Chochin, Nopperabo, Oni, GreatOni, NightLord, RokurokubiClerk, TsurubeDrop, CaptainKitsunebi,
    Kodama, PaperCrane, LanternFairy, FoxMaskPhantom, Komainu, BellMaiden, ToriiSerpent, Mitsuhime, ShardMoth, BoneEaterCrow, AdmiralTengu,
    StrawPuppet, LeafShinobi, KunaiThrower, SmokeNinja, IronRonin, Kagerou, EmberToad, BurningShogun, RoguePuppeteer, ClayBird, MistCaptain,
    LostSoul, MaskedHound, PaleArcher, ShadeDancer, BoneSentinel, GateCaptain, HollowMonarch, PaleSeraph, MaskedRetainer, HollowHusk, NumberedQuartermaster,
    DrownedRat, RainGhoul, GutterSpitter, UmbrellaStalker, WardButcher, NightInvestigator, GutterMother, CrimsonRainLord, BladetailRunner, MaskCollector, DrownedCaptain,
    CursedSpawn, WailingCommuter, CurseEye, GlitchShade, GradeOneBrute, PatchworkSorcerer, CrossingCalamity, ShatterSovereign, SwarmMother, ToolThief, CurseCaptain,
];

/// Where an enemy sits in a stage's roster.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    Fodder,
    Elite,
    Mini,
    Boss,
    Final,
}

impl Rank {
    /// How much one enemy of this rank adds to a crowd, in fodder-equivalents.
    /// Any mini-boss or boss in range is a crowd on its own.
    pub fn threat(self) -> f32 {
        match self {
            Rank::Fodder => 1.0,
            Rank::Elite => 6.0,
            Rank::Mini | Rank::Boss | Rank::Final => 20.0,
        }
    }
}

/// How rare an enemy is, for outlines, drops and the boss bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnemyRarity {
    Common,
    Uncommon,
    Elite,
    Leader,
    SubBoss,
    Boss,
}

impl EnemyRarity {
    pub fn name(self) -> &'static str {
        match self {
            EnemyRarity::Common => "Common",
            EnemyRarity::Uncommon => "Uncommon",
            EnemyRarity::Elite => "Elite",
            EnemyRarity::Leader => "Leader",
            EnemyRarity::SubBoss => "Sub-boss",
            EnemyRarity::Boss => "Boss",
        }
    }
}

/// What an ordinary ranged enemy fires.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Volley {
    Single { speed: f32 },
    Fan3 { speed: f32 },
    Ring6 { speed: f32 },
}

/// Behaviour archetype.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ai {
    Chase,
    /// Hopping gait (umbrella yokai).
    Hop,
    /// Chases, then crouches (0.35s) and lunges at 3.2x speed.
    Dash,
    /// Holds a firing range and shoots every `every` seconds.
    Ranged { volley: Volley, every: f32 },
    /// Fades out and reappears beside the player.
    Blink,
    /// Chase -> telegraph -> straight charge (Great Oni and charging elites).
    Charger { speed: f32, rest: f32 },
    /// The Night Lord's hand-written pattern.
    Daimyo,
    /// Generic boss driven by its `BossKit`.
    Boss,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyDef {
    /// For fodder and elites: base HP (scaled with time). For bosses: absolute HP.
    pub hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub radius: f32,
    pub xp: u32,
    /// 0 = no knockback, 1 = full.
    pub knockback: f32,
    pub ai: Ai,
}

const fn def(hp: f32, speed: f32, damage: f32, radius: f32, xp: u32, knockback: f32, ai: Ai) -> EnemyDef {
    EnemyDef { hp, speed, damage, radius, xp, knockback, ai }
}

pub fn enemy_def(kind: EnemyKind) -> EnemyDef {
    let fan = |speed| Ai::Ranged { volley: Volley::Fan3 { speed }, every: 4.0 };
    match kind {
        Wisp => def(8.0, 46.0, 5.0, 7.0, 1, 1.0, Ai::Chase),
        Kasa => def(22.0, 56.0, 9.0, 7.0, 2, 0.7, Ai::Hop),
        Oni => def(420.0, 38.0, 20.0, 14.0, 30, 0.0, Ai::Chase),
        // 7000 base HP scaled to 8:00, as it always was.
        GreatOni => def(18_760.0, 34.0, 30.0, 26.0, 120, 0.0, Ai::Charger { speed: 290.0, rest: 5.0 }),
        Chochin => def(30.0, 34.0, 8.0, 8.0, 3, 0.6, Ai::Ranged { volley: Volley::Single { speed: 115.0 }, every: 4.5 }),
        Nopperabo => def(45.0, 44.0, 9.0, 7.0, 4, 0.5, Ai::Blink),
        NightLord => def(50_000.0, 28.0, 35.0, 30.0, 0, 0.0, Ai::Daimyo),

        RokurokubiClerk => def(66.0, 56.0, 11.0, 9.0, 4, 0.5, Ai::Dash),
        TsurubeDrop => def(66.0, 54.0, 11.0, 9.0, 4, 0.5, Ai::Blink),
        CaptainKitsunebi => def(760.0, 34.0, 24.0, 16.0, 60, 0.0, Ai::Boss),

        Kodama => def(8.0, 48.0, 5.0, 7.0, 1, 1.0, Ai::Chase),
        PaperCrane => def(18.0, 52.0, 8.0, 7.0, 2, 0.8, Ai::Dash),
        LanternFairy => def(28.0, 36.0, 8.0, 8.0, 3, 0.6, fan(110.0)),
        FoxMaskPhantom => def(42.0, 44.0, 9.0, 7.0, 4, 0.5, Ai::Blink),
        Komainu => def(460.0, 36.0, 20.0, 14.0, 30, 0.0, Ai::Chase),
        BellMaiden => def(26_000.0, 30.0, 28.0, 24.0, 150, 0.0, Ai::Boss),
        ToriiSerpent => def(55_000.0, 34.0, 32.0, 30.0, 220, 0.0, Ai::Boss),
        Mitsuhime => def(120_000.0, 26.0, 36.0, 34.0, 0, 0.0, Ai::Boss),

        ShardMoth => def(40.0, 57.0, 11.0, 9.0, 4, 0.5, Ai::Chase),
        BoneEaterCrow => def(54.0, 52.0, 11.0, 9.0, 4, 0.5, Ai::Dash),
        AdmiralTengu => def(830.0, 34.0, 24.0, 16.0, 60, 0.0, Ai::Boss),

        StrawPuppet => def(10.0, 44.0, 6.0, 7.0, 1, 1.0, Ai::Chase),
        LeafShinobi => def(20.0, 58.0, 9.0, 7.0, 2, 0.8, Ai::Dash),
        KunaiThrower => def(28.0, 40.0, 8.0, 8.0, 3, 0.6, fan(150.0)),
        SmokeNinja => def(40.0, 48.0, 9.0, 7.0, 4, 0.5, Ai::Blink),
        IronRonin => def(480.0, 36.0, 22.0, 14.0, 30, 0.0, Ai::Charger { speed: 240.0, rest: 6.0 }),
        Kagerou => def(26_000.0, 40.0, 30.0, 22.0, 150, 0.0, Ai::Boss),
        EmberToad => def(58_000.0, 26.0, 34.0, 32.0, 220, 0.0, Ai::Boss),
        BurningShogun => def(125_000.0, 30.0, 38.0, 32.0, 0, 0.0, Ai::Boss),

        RoguePuppeteer => def(42.0, 54.0, 11.0, 9.0, 4, 0.5, Ai::Ranged { volley: Volley::Single { speed: 150.0 }, every: 4.5 }),
        ClayBird => def(42.0, 58.0, 8.0, 9.0, 4, 0.5, Ai::Charger { speed: 240.0, rest: 6.0 }),
        MistCaptain => def(480.0, 34.0, 12.0, 16.0, 60, 0.0, Ai::Boss),

        LostSoul => def(8.0, 46.0, 5.0, 7.0, 1, 1.0, Ai::Chase),
        MaskedHound => def(24.0, 54.0, 9.0, 7.0, 2, 0.7, Ai::Dash),
        PaleArcher => def(26.0, 34.0, 8.0, 8.0, 3, 0.6, Ai::Ranged { volley: Volley::Single { speed: 190.0 }, every: 3.5 }),
        ShadeDancer => def(44.0, 46.0, 9.0, 7.0, 4, 0.5, Ai::Blink),
        BoneSentinel => def(520.0, 34.0, 22.0, 15.0, 30, 0.0, Ai::Chase),
        GateCaptain => def(28_000.0, 32.0, 30.0, 24.0, 150, 0.0, Ai::Boss),
        HollowMonarch => def(60_000.0, 30.0, 34.0, 32.0, 220, 0.0, Ai::Boss),
        PaleSeraph => def(130_000.0, 26.0, 38.0, 34.0, 0, 0.0, Ai::Boss),

        MaskedRetainer => def(72.0, 54.0, 11.0, 9.0, 4, 0.5, Ai::Dash),
        HollowHusk => def(90.0, 49.0, 11.0, 11.0, 4, 0.5, Ai::Chase),
        NumberedQuartermaster => def(940.0, 34.0, 24.0, 16.0, 60, 0.0, Ai::Boss),

        DrownedRat => def(6.0, 60.0, 5.0, 6.0, 1, 1.0, Ai::Chase),
        RainGhoul => def(26.0, 54.0, 10.0, 7.0, 2, 0.7, Ai::Chase),
        GutterSpitter => def(32.0, 32.0, 8.0, 8.0, 3, 0.5, Ai::Ranged { volley: Volley::Single { speed: 90.0 }, every: 3.0 }),
        UmbrellaStalker => def(46.0, 44.0, 10.0, 7.0, 4, 0.5, Ai::Blink),
        WardButcher => def(540.0, 36.0, 24.0, 15.0, 30, 0.0, Ai::Charger { speed: 250.0, rest: 6.0 }),
        NightInvestigator => def(28_000.0, 38.0, 30.0, 22.0, 150, 0.0, Ai::Boss),
        GutterMother => def(62_000.0, 22.0, 36.0, 34.0, 220, 0.0, Ai::Boss),
        CrimsonRainLord => def(135_000.0, 30.0, 40.0, 34.0, 0, 0.0, Ai::Boss),

        BladetailRunner => def(78.0, 54.0, 11.0, 9.0, 4, 0.5, Ai::Dash),
        MaskCollector => def(78.0, 52.0, 11.0, 9.0, 4, 0.5, Ai::Blink),
        DrownedCaptain => def(970.0, 34.0, 24.0, 16.0, 60, 0.0, Ai::Boss),

        CursedSpawn => def(9.0, 46.0, 6.0, 7.0, 1, 1.0, Ai::Chase),
        WailingCommuter => def(28.0, 50.0, 10.0, 7.0, 2, 0.7, Ai::Hop),
        CurseEye => def(30.0, 32.0, 8.0, 8.0, 3, 0.5, Ai::Ranged { volley: Volley::Ring6 { speed: 100.0 }, every: 4.5 }),
        GlitchShade => def(46.0, 48.0, 10.0, 7.0, 4, 0.5, Ai::Blink),
        GradeOneBrute => def(560.0, 38.0, 24.0, 15.0, 30, 0.0, Ai::Charger { speed: 260.0, rest: 5.5 }),
        PatchworkSorcerer => def(30_000.0, 32.0, 32.0, 22.0, 150, 0.0, Ai::Boss),
        CrossingCalamity => def(65_000.0, 30.0, 36.0, 34.0, 220, 0.0, Ai::Boss),
        ShatterSovereign => def(140_000.0, 30.0, 40.0, 34.0, 0, 0.0, Ai::Boss),
        SwarmMother => def(84.0, 50.0, 11.0, 9.0, 4, 0.5, Ai::Hop),
        ToolThief => def(84.0, 48.0, 11.0, 9.0, 4, 0.5, Ai::Blink),
        CurseCaptain => def(1010.0, 34.0, 24.0, 16.0, 60, 0.0, Ai::Boss),
    }
}

impl EnemyKind {
    pub fn index(self) -> usize {
        ALL_ENEMIES.iter().position(|k| *k == self).unwrap()
    }

    pub fn rank(self) -> Rank {
        match self {
            Oni | Komainu | IronRonin | BoneSentinel | WardButcher | GradeOneBrute => Rank::Elite,
            CaptainKitsunebi | AdmiralTengu | MistCaptain | NumberedQuartermaster | DrownedCaptain | CurseCaptain => Rank::Elite,
            GreatOni | BellMaiden | Kagerou | GateCaptain | NightInvestigator | PatchworkSorcerer => Rank::Mini,
            ToriiSerpent | EmberToad | HollowMonarch | GutterMother | CrossingCalamity => Rank::Boss,
            NightLord | Mitsuhime | BurningShogun | PaleSeraph | CrimsonRainLord | ShatterSovereign => Rank::Final,
            _ => Rank::Fodder,
        }
    }

    pub fn element(self) -> Element {
        match self {
            Wisp | Chochin | Oni | NightLord | LanternFairy | EmberToad | BurningShogun | WardButcher | TsurubeDrop | CaptainKitsunebi => Element::Fire,
            Kasa | DrownedRat | RainGhoul | GutterSpitter | GutterMother | CrimsonRainLord | PatchworkSorcerer | MistCaptain | BladetailRunner | DrownedCaptain => Element::Water,
            NightInvestigator | CursedSpawn | WailingCommuter | CurseEye | GlitchShade | CrossingCalamity | MaskCollector | ToolThief | CurseCaptain => Element::Electric,
            GreatOni | Kodama | FoxMaskPhantom | Komainu | BellMaiden | ToriiSerpent | IronRonin | BoneSentinel | GradeOneBrute => Element::Earth,
            RokurokubiClerk | BoneEaterCrow | AdmiralTengu | SwarmMother => Element::Earth,
            Nopperabo | PaperCrane | StrawPuppet | LeafShinobi | KunaiThrower | SmokeNinja | Kagerou | UmbrellaStalker | ShardMoth | RoguePuppeteer | ClayBird => Element::Wind,
            LostSoul | MaskedHound | PaleArcher | ShadeDancer | GateCaptain | MaskedRetainer | HollowHusk | NumberedQuartermaster => Element::Ice,
            Mitsuhime | PaleSeraph => Element::Light,
            HollowMonarch | ShatterSovereign => Element::Darkness,
        }
    }

    pub fn is_leader(self) -> bool {
        self.rarity() == EnemyRarity::Leader
    }

    pub fn rarity(self) -> EnemyRarity {
        match self {
            RokurokubiClerk | TsurubeDrop | ShardMoth | BoneEaterCrow | RoguePuppeteer | ClayBird | MaskedRetainer | HollowHusk | BladetailRunner
            | MaskCollector | SwarmMother | ToolThief => EnemyRarity::Uncommon,
            CaptainKitsunebi | AdmiralTengu | MistCaptain | NumberedQuartermaster | DrownedCaptain | CurseCaptain => EnemyRarity::Leader,
            k => match k.rank() {
                Rank::Fodder => EnemyRarity::Common,
                Rank::Elite => EnemyRarity::Elite,
                Rank::Mini => EnemyRarity::SubBoss,
                Rank::Boss | Rank::Final => EnemyRarity::Boss,
            },
        }
    }

    /// Mini-bosses, bosses and final bosses (ignore freezes, decoys and props).
    pub fn is_boss(self) -> bool {
        self.rank() >= Rank::Mini
    }

    pub fn name(self) -> &'static str {
        match self {
            Wisp => "Hitodama Wisp",
            Kasa => "Kasa-obake",
            Chochin => "Chochin-obake",
            Nopperabo => "Nopperabo",
            Oni => "Oni",
            GreatOni => "Great Oni",
            NightLord => "Night Lord: Chochin Daimyo",
            RokurokubiClerk => "Rokurokubi Clerk",
            TsurubeDrop => "Tsurube Drop",
            CaptainKitsunebi => "Captain Kitsunebi",
            Kodama => "Kodama Sprite",
            PaperCrane => "Paper Crane Shikigami",
            LanternFairy => "Lantern Fairy",
            FoxMaskPhantom => "Fox-Mask Phantom",
            Komainu => "Komainu Guardian",
            BellMaiden => "Kagura Bell Maiden",
            ToriiSerpent => "Thousand-Torii Serpent",
            Mitsuhime => "Mitsuhime of the Veiled Moon",
            ShardMoth => "Shard Moth",
            BoneEaterCrow => "Bone-Eater Crow",
            AdmiralTengu => "Admiral Tengu",
            StrawPuppet => "Straw Puppet Soldier",
            LeafShinobi => "Leaf Shinobi",
            KunaiThrower => "Kunai Thrower",
            SmokeNinja => "Smoke Ninja",
            IronRonin => "Iron Mask Ronin",
            Kagerou => "Kagerou, Twin-Blade Captain",
            EmberToad => "Great Ember Toad",
            BurningShogun => "Shogun of the Burning Leaf",
            RoguePuppeteer => "Rogue Puppeteer",
            ClayBird => "Exploding Clay Bird",
            MistCaptain => "Mist Swordsman Captain",
            LostSoul => "Lost Soul",
            MaskedHound => "Masked Hound",
            PaleArcher => "Pale Archer",
            ShadeDancer => "Shade Dancer",
            BoneSentinel => "Bone Sentinel",
            GateCaptain => "Captain of the Ninth Gate",
            HollowMonarch => "The Hollow Monarch",
            PaleSeraph => "Seraph of the Pale Throne",
            MaskedRetainer => "Masked Retainer",
            HollowHusk => "Hollow Husk",
            NumberedQuartermaster => "Numbered Quartermaster",
            DrownedRat => "Drowned Rat",
            RainGhoul => "Rain Ghoul",
            GutterSpitter => "Gutter Spitter",
            UmbrellaStalker => "Umbrella Stalker",
            WardButcher => "Ward Butcher",
            NightInvestigator => "The Night Investigator",
            GutterMother => "Mother of Gutters",
            CrimsonRainLord => "The Crimson Rain Lord",
            BladetailRunner => "Bladetail Runner",
            MaskCollector => "Mask Collector",
            DrownedCaptain => "Drowned Pirate Captain",
            CursedSpawn => "Cursed Spawn",
            WailingCommuter => "Wailing Commuter",
            CurseEye => "Curse Eye",
            GlitchShade => "Glitch Shade",
            GradeOneBrute => "Grade-One Brute",
            PatchworkSorcerer => "The Patchwork Sorcerer",
            CrossingCalamity => "Calamity of the Crossing",
            ShatterSovereign => "The Shatterline Sovereign",
            SwarmMother => "Grade-Three Swarm Mother",
            ToolThief => "Cursed Tool Thief",
            CurseCaptain => "Curse User Captain",
        }
    }

    /// Short name for the timer ("Mitsuhime in 4:12").
    pub fn short_name(self) -> &'static str {
        match self {
            NightLord => "Night Lord",
            Mitsuhime => "Mitsuhime",
            BurningShogun => "The Shogun",
            PaleSeraph => "The Seraph",
            CrimsonRainLord => "Rain Lord",
            ShatterSovereign => "The Sovereign",
            k => k.name(),
        }
    }

    /// Collection entry text.
    pub fn lore(self) -> &'static str {
        match self {
            Wisp => "A stray soul-flame from a story nobody finished. Alone it flickers; a crowd of them smothers.",
            Kasa => "An umbrella left behind too long. Hops on one leg and stares with its one eye.",
            Chochin => "A paper lantern with a grudge. Keeps its distance and spits fireballs.",
            Nopperabo => "A faceless passer-by. Fades out and reappears right beside you.",
            Oni => "A horned brute from the old tales. Hits hard and guards a treasure chest.",
            GreatOni => "Arrives at 8:00. Plants its feet, then charges in a straight line.",
            NightLord => "Lantern warlord of the Shotengai, risen at 15:00. Defeat it before dawn.",
            RokurokubiClerk => "A shopkeeper whose neck stretches to watch the aisles. Lunges the moment you look away from the till.",
            TsurubeDrop => "A well-bucket that falls from the awnings. It drops out of nowhere, then drops again.",
            CaptainKitsunebi => "Leads a crew of foxfire down the arcade under a flag of flame. Where the banner goes, they follow.",
            Kodama => "A sapling's echo. Harmless alone, but the valley is full of them tonight.",
            PaperCrane => "A folded familiar. It hangs still, then darts at you edge-first.",
            LanternFairy => "Carries a light it never stops throwing. Three sparks at a time.",
            FoxMaskPhantom => "A festival mask with nobody behind it. Appears wherever you look away.",
            Komainu => "A stone guardian that forgot who it was guarding. Holds a chest in its jaws.",
            BellMaiden => "Dances the old kagura for a god who left. Every step rings out a ring of light.",
            ToriiSerpent => "A thousand gates linked into one long body. It charges along its own approach.",
            Mitsuhime => "The veiled moon princess who keeps the valley hidden. Her sky is made of bullets.",
            ShardMoth => "A moth made of cut moonlight. Quick, brittle, and everywhere at once.",
            BoneEaterCrow => "Picks the valley clean and holds a grudge about it. Dives from a crouch.",
            AdmiralTengu => "A mountain admiral who commands from a ship of cloud. Calls down the storm and his crew with it.",
            StrawPuppet => "A training dummy given orders. It still marches after the war ended.",
            LeafShinobi => "Crouches in the brush, then closes the gap in a blink of leaves.",
            KunaiThrower => "Throws in fans of three. Keep moving sideways.",
            SmokeNinja => "Half smoke already. Vanishes and strikes from the other side.",
            IronRonin => "A masterless blade in an iron demon mask. Lines up, then charges.",
            Kagerou => "Captain of the twin blades. Heat-haze steps, then a storm of steel.",
            EmberToad => "A war toad with a shrine on its back. It rains burning tags on the field.",
            BurningShogun => "The warlord who set the frontier alight to win it. His fan commands fire.",
            RoguePuppeteer => "Cut its own strings and kept the dolls. Throws knives one at a time.",
            ClayBird => "A glazed bird packed with powder. It charges, and it does not stop for you.",
            MistCaptain => "Captain of a mist-bound blade crew. Steps through the fog, then cuts through you.",
            LostSoul => "A soul that missed its gate. It follows anything warm.",
            MaskedHound => "Wears a mask to hide what it ate. Lunges from a crouch.",
            PaleArcher => "A guard who still keeps watch. Its arrows are slow to fall and quick to fly.",
            ShadeDancer => "A shadow on a ribbon. It steps out of yours.",
            BoneSentinel => "A halberdier of the citadel, tireless, armoured in its own bones.",
            GateCaptain => "Keeper of the Ninth Gate. Its shield is the gate itself.",
            HollowMonarch => "A beast-king with nothing inside. It tries to fill the hole with you.",
            PaleSeraph => "Throne-keeper of the citadel. Six wings, a halo of blades, no mercy.",
            MaskedRetainer => "A retainer who never took off the mask. Crouches, then lunges.",
            HollowHusk => "A husk of armour with nothing left to lose. Slow, heavy, and hard to put down.",
            NumberedQuartermaster => "Keeps the citadel's ranks in numbered order. His banner draws a ring, and the crew falls in.",
            DrownedRat => "Came up with the flood. Fast, small, never alone.",
            RainGhoul => "Hunched, soaked, hungry. Long arms that reach before it does.",
            GutterSpitter => "A bloated sewer thing that spits slow acid.",
            UmbrellaStalker => "Only its eyes under the umbrella. It is always one streetlight closer.",
            WardButcher => "The ward's worst rumour. Raincoat, meat hook, a straight-line charge.",
            NightInvestigator => "Hunts what hunts the ward. Its briefcase unfolds into something sharp.",
            GutterMother => "Rises from the manhole when the drains overflow. Everything here is her brood.",
            CrimsonRainLord => "When the rain turns red, he is already here.",
            BladetailRunner => "Sprints the flooded alleys on a blade for a tail. Lunges low and fast.",
            MaskCollector => "Collects faces. Slips out of the rain beside you to size up yours.",
            DrownedCaptain => "Sailed the flood and never reached shore. His ship's crew rises out of the water at his word.",
            CursedSpawn => "Curse energy that clotted into eyes. Many, many eyes.",
            WailingCommuter => "Missed the last train forever. Lurches forward in hops, wailing.",
            CurseEye => "A floating eye in purple fire. Blinks out a ring of curses.",
            GlitchShade => "A shadow that does not render properly. Appears beside you mid-frame.",
            GradeOneBrute => "A many-armed curse stitched from a crowd. Charges like a train.",
            PatchworkSorcerer => "Sews curses together for fun. Pulls the strings of the crowd.",
            CrossingCalamity => "The crossing itself, risen: wrecked cars and signals in one body.",
            ShatterSovereign => "Monarch of the shattered city. When he opens his domain, there is no outside.",
            SwarmMother => "The curse that mothers the crossing's swarms. She hops in, and the spawn follow.",
            ToolThief => "Steals what you hold dear and wears it. Blinks in to take more.",
            CurseCaptain => "Runs a crew of curses like a ship's company. Every order lands as a blast of ruin.",
        }
    }
}

/// Minimum rewards in the chest dropped on death (0 = no chest).
pub fn chest_tier(kind: EnemyKind) -> u8 {
    match kind.rank() {
        Rank::Elite => 2,
        Rank::Mini | Rank::Boss => 3,
        _ => 0,
    }
}

/// Sen dropped on death (bosses only; fodder rolls a rare coin separately).
pub fn coin_drop(kind: EnemyKind) -> u32 {
    match kind.rank() {
        Rank::Final => 150,
        Rank::Boss => 90,
        Rank::Mini => 50,
        Rank::Elite => 5,
        Rank::Fodder => 0,
    }
}

// ───────────────────────── Boss move sets ─────────────────────────

/// One step of a boss's attack cycle. Counts and speeds grow with each phase.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    /// `n` bullets in a circle, `layers` rings at stepped speeds (a flower).
    Ring { n: u8, layers: u8, speed: f32 },
    /// `n` bullets aimed at the player across `spread` radians.
    Fan { n: u8, spread: f32, speed: f32 },
    /// Telegraphed straight charge.
    Charge,
    /// Vanish and reappear near the player, with a ring.
    Blink,
    /// Telegraphed blasts landing around the player.
    Rain { n: u8 },
    /// Calls `n` of the kit's minions around the player.
    Summon { n: u8 },
}

/// A constant rotating spray, on from phase `from` (0 = always).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spiral {
    pub arms: u8,
    /// Radians per second.
    pub turn: f32,
    pub every: f32,
    pub speed: f32,
    pub from: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BossKit {
    /// Seconds between moves (shrinks per phase).
    pub cycle: f32,
    pub moves: &'static [Move],
    pub spiral: Option<Spiral>,
    pub minion: EnemyKind,
    pub charge_speed: f32,
}

pub fn boss_kit(kind: EnemyKind) -> BossKit {
    use Move::*;
    let sp = |arms, turn, every, speed, from| Some(Spiral { arms, turn, every, speed, from });
    let kit = |cycle, moves: &'static [Move], spiral, minion| BossKit { cycle, moves, spiral, minion, charge_speed: 300.0 };
    match kind {
        BellMaiden => kit(2.6, &[Ring { n: 16, layers: 2, speed: 90.0 }, Fan { n: 7, spread: 0.7, speed: 130.0 }, Summon { n: 6 }], sp(3, 1.6, 0.2, 85.0, 1), FoxMaskPhantom),
        ToriiSerpent => kit(2.8, &[Charge, Ring { n: 20, layers: 1, speed: 100.0 }, Charge, Fan { n: 9, spread: 0.9, speed: 140.0 }], sp(2, -1.2, 0.18, 90.0, 1), PaperCrane),
        Mitsuhime => kit(2.4, &[Ring { n: 24, layers: 3, speed: 80.0 }, Fan { n: 11, spread: 1.0, speed: 150.0 }, Rain { n: 6 }, Blink, Summon { n: 10 }], sp(4, 1.4, 0.15, 85.0, 0), LanternFairy),
        Kagerou => kit(2.4, &[Blink, Fan { n: 5, spread: 0.4, speed: 175.0 }, Charge, Summon { n: 5 }], None, LeafShinobi),
        EmberToad => kit(3.0, &[Rain { n: 8 }, Ring { n: 18, layers: 1, speed: 95.0 }, Summon { n: 8 }, Charge], sp(2, 0.9, 0.25, 80.0, 2), StrawPuppet),
        BurningShogun => kit(2.5, &[Fan { n: 9, spread: 0.8, speed: 160.0 }, Rain { n: 10 }, Charge, Ring { n: 22, layers: 2, speed: 100.0 }, Summon { n: 10 }], sp(2, 1.8, 0.16, 95.0, 1), KunaiThrower),
        GateCaptain => kit(2.7, &[Charge, Fan { n: 7, spread: 0.5, speed: 150.0 }, Ring { n: 16, layers: 1, speed: 100.0 }], sp(4, 0.8, 0.3, 80.0, 1), MaskedHound),
        HollowMonarch => kit(2.6, &[Ring { n: 20, layers: 2, speed: 90.0 }, Summon { n: 8 }, Blink, Fan { n: 9, spread: 0.9, speed: 140.0 }], sp(3, -1.5, 0.18, 90.0, 1), ShadeDancer),
        PaleSeraph => kit(2.3, &[Ring { n: 28, layers: 3, speed: 85.0 }, Fan { n: 13, spread: 1.2, speed: 150.0 }, Blink, Rain { n: 8 }, Summon { n: 10 }], sp(6, 1.1, 0.16, 85.0, 0), PaleArcher),
        NightInvestigator => kit(2.4, &[Fan { n: 5, spread: 0.3, speed: 190.0 }, Charge, Fan { n: 9, spread: 0.8, speed: 150.0 }, Blink], None, RainGhoul),
        GutterMother => kit(3.0, &[Summon { n: 12 }, Ring { n: 18, layers: 2, speed: 80.0 }, Rain { n: 8 }], sp(3, 0.7, 0.3, 70.0, 1), DrownedRat),
        CrimsonRainLord => kit(2.3, &[Rain { n: 12 }, Fan { n: 11, spread: 1.0, speed: 160.0 }, Blink, Ring { n: 24, layers: 2, speed: 100.0 }, Summon { n: 10 }], sp(3, 1.6, 0.15, 95.0, 0), UmbrellaStalker),
        PatchworkSorcerer => kit(2.5, &[Summon { n: 8 }, Fan { n: 7, spread: 0.6, speed: 150.0 }, Blink, Ring { n: 16, layers: 1, speed: 100.0 }], sp(2, 1.2, 0.22, 85.0, 1), WailingCommuter),
        CrossingCalamity => kit(2.8, &[Charge, Ring { n: 22, layers: 2, speed: 95.0 }, Rain { n: 10 }, Charge, Summon { n: 8 }], sp(4, -1.0, 0.2, 90.0, 1), CursedSpawn),
        CaptainKitsunebi => kit(3.0, &[Fan { n: 5, spread: 0.6, speed: 150.0 }], None, Wisp),
        AdmiralTengu => kit(3.0, &[Blink, Rain { n: 4 }], None, Kodama),
        MistCaptain => kit(3.0, &[Blink, Charge], None, StrawPuppet),
        NumberedQuartermaster => kit(3.0, &[Ring { n: 14, layers: 1, speed: 95.0 }], None, LostSoul),
        DrownedCaptain => kit(3.0, &[Summon { n: 4 }], None, DrownedRat),
        CurseCaptain => kit(3.0, &[Rain { n: 5 }], None, CursedSpawn),
        _ => kit(2.2, &[Ring { n: 30, layers: 3, speed: 90.0 }, Fan { n: 13, spread: 1.1, speed: 160.0 }, Rain { n: 12 }, Blink, Charge, Summon { n: 12 }], sp(4, 1.7, 0.14, 95.0, 0), GlitchShade),
    }
}

/// Phases a boss fights through (each below an equal slice of its HP).
pub fn boss_phases(kind: EnemyKind, extra: u8) -> u8 {
    if !kind.is_boss() {
        return 1;
    }
    let base = match kind.rank() {
        Rank::Mini => 1,
        Rank::Boss => 2,
        Rank::Final => 3,
        _ => 1,
    };
    base + extra
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rarity_follows_rank_and_roster() {
        let (mut uncommon, mut leader) = (0, 0);
        for k in ALL_ENEMIES {
            match k.rarity() {
                EnemyRarity::Uncommon => {
                    uncommon += 1;
                    assert_eq!(k.rank(), Rank::Fodder, "{k:?}");
                }
                EnemyRarity::Leader => {
                    leader += 1;
                    assert_eq!(k.rank(), Rank::Elite, "{k:?}");
                    assert_eq!(enemy_def(k).ai, Ai::Boss, "{k:?}");
                }
                EnemyRarity::Common => assert_eq!(k.rank(), Rank::Fodder, "{k:?}"),
                EnemyRarity::Elite => assert_eq!(k.rank(), Rank::Elite, "{k:?}"),
                EnemyRarity::SubBoss => assert_eq!(k.rank(), Rank::Mini, "{k:?}"),
                EnemyRarity::Boss => assert!(k.rank() >= Rank::Boss, "{k:?}"),
            }
        }
        assert_eq!((uncommon, leader), (12, 6));
        for (i, k) in ALL_ENEMIES.iter().enumerate() {
            assert_eq!(k.index(), i);
            assert!(!ALL_ENEMIES[..i].contains(k), "{k:?} listed twice");
        }
    }

    #[test]
    fn leaders_are_not_bosses() {
        for k in ALL_ENEMIES.into_iter().filter(|k| k.is_leader()) {
            assert!(!k.is_boss(), "{k:?}");
            assert_eq!(boss_phases(k, 2), 1, "{k:?}");
            assert_eq!(chest_tier(k), 2, "{k:?}");
            assert_eq!(coin_drop(k), 5, "{k:?}");
            assert_eq!(boss_kit(k).cycle, 3.0, "{k:?}");
        }
    }
}
