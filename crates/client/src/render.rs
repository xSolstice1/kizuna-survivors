//! Draws the world: ground, pickups, enemies, player, weapons, effects.

use macroquad::prelude::*;
#[allow(unused_imports)]
use crate::kit::{draw_circle, draw_circle_lines};

use crate::kit;
use sim::content::{Element, EnemyKind, HeroId, Rank, PassiveKind, PickupKind, SpecialKind, Upgrade, WeaponKind, ALL_HEROES, ALL_PICKUPS};
use sim::enemies::EnemyRarity;
use sim::world::{style, ProjKind};
use sim::{Event, World};

use crate::sprites;
use crate::voice;

/// World pixels → screen pixels (integer for crisp pixel art). Small screens (phones in
/// landscape) zoom out to 1x so the horde stays visible.
pub fn world_scale() -> f32 {
    // screen_height() is in CSS pixels (macroquad scales to the device's density itself).
    if screen_height() < 560.0 {
        1.0
    } else {
        2.0
    }
}

/// Source of an enemy's sprite (None for the original prologue kinds, which have named fields).
fn world_enemy_src(k: EnemyKind) -> Option<&'static str> {
    Some(match k {
        EnemyKind::Kodama => "w1/fodder_a",
        EnemyKind::PaperCrane => "w1/fodder_b",
        EnemyKind::LanternFairy => "w1/ranged",
        EnemyKind::FoxMaskPhantom => "w1/blinker",
        EnemyKind::Komainu => "w1/elite",
        EnemyKind::BellMaiden => "w1/mini",
        EnemyKind::ToriiSerpent => "w1/boss",
        EnemyKind::Mitsuhime => "w1/final",
        EnemyKind::StrawPuppet => "w2/fodder_a",
        EnemyKind::LeafShinobi => "w2/fodder_b",
        EnemyKind::KunaiThrower => "w2/ranged",
        EnemyKind::SmokeNinja => "w2/blinker",
        EnemyKind::IronRonin => "w2/elite",
        EnemyKind::Kagerou => "w2/mini",
        EnemyKind::EmberToad => "w2/boss",
        EnemyKind::BurningShogun => "w2/final",
        EnemyKind::LostSoul => "w3/fodder_a",
        EnemyKind::MaskedHound => "w3/fodder_b",
        EnemyKind::PaleArcher => "w3/ranged",
        EnemyKind::ShadeDancer => "w3/blinker",
        EnemyKind::BoneSentinel => "w3/elite",
        EnemyKind::GateCaptain => "w3/mini",
        EnemyKind::HollowMonarch => "w3/boss",
        EnemyKind::PaleSeraph => "w3/final",
        EnemyKind::DrownedRat => "w4/fodder_a",
        EnemyKind::RainGhoul => "w4/fodder_b",
        EnemyKind::GutterSpitter => "w4/ranged",
        EnemyKind::UmbrellaStalker => "w4/blinker",
        EnemyKind::WardButcher => "w4/elite",
        EnemyKind::NightInvestigator => "w4/mini",
        EnemyKind::GutterMother => "w4/boss",
        EnemyKind::CrimsonRainLord => "w4/final",
        EnemyKind::CursedSpawn => "w5/fodder_a",
        EnemyKind::WailingCommuter => "w5/fodder_b",
        EnemyKind::CurseEye => "w5/ranged",
        EnemyKind::GlitchShade => "w5/blinker",
        EnemyKind::GradeOneBrute => "w5/elite",
        EnemyKind::PatchworkSorcerer => "w5/mini",
        EnemyKind::CrossingCalamity => "w5/boss",
        EnemyKind::ShatterSovereign => "w5/final",
        EnemyKind::RokurokubiClerk => "rokurokubi_clerk",
        EnemyKind::TsurubeDrop => "tsurube_drop",
        EnemyKind::CaptainKitsunebi => "captain_kitsunebi",
        EnemyKind::ShardMoth => "w1/uncommon_a",
        EnemyKind::BoneEaterCrow => "w1/uncommon_b",
        EnemyKind::AdmiralTengu => "w1/leader",
        EnemyKind::RoguePuppeteer => "w2/uncommon_a",
        EnemyKind::ClayBird => "w2/uncommon_b",
        EnemyKind::MistCaptain => "w2/leader",
        EnemyKind::MaskedRetainer => "w3/uncommon_a",
        EnemyKind::HollowHusk => "w3/uncommon_b",
        EnemyKind::NumberedQuartermaster => "w3/leader",
        EnemyKind::BladetailRunner => "w4/uncommon_a",
        EnemyKind::MaskCollector => "w4/uncommon_b",
        EnemyKind::DrownedCaptain => "w4/leader",
        EnemyKind::SwarmMother => "w5/uncommon_a",
        EnemyKind::ToolThief => "w5/uncommon_b",
        EnemyKind::CurseCaptain => "w5/leader",
        _ => return None,
    })
}

/// A hero's generated animation set (tools/ai/anim.py): walk + idle loops and a walking
/// throw, all on one canvas so switching never jumps. Values come from art/hd/<hero>_anim.txt.
pub struct HeroAnim {
    pub walk: Vec<Texture2D>,
    pub idle: Vec<Texture2D>,
    pub wattack: Vec<Texture2D>,
    /// Standing throw.
    pub attack: Vec<Texture2D>,
    /// Frame (in each throw) where the projectile leaves the hand.
    pub attack_release: usize,
    pub wattack_release: usize,
    pub fps: f32,
    /// Walk frame closest to the idle pose (start walking from here).
    pub walk_start: usize,
    /// Walk frame to continue from after the walking throw.
    pub resume: usize,
    /// Frame size in world units (feet at the bottom edge).
    pub size: Vec2,
}

impl HeroAnim {
    fn load(hero: &str, meta: &str) -> Self {
        let val = |k: &str| -> Vec<f32> {
            meta.lines()
                .find_map(|l| l.strip_prefix(k).filter(|r| r.starts_with(' ')).map(|r| r.split_whitespace().filter_map(|x| x.parse().ok()).collect()))
                .unwrap_or_default()
        };
        let set = |name: &str| -> Vec<Texture2D> { (0..val(name)[0] as usize).map(|i| sprites::hi_art(&format!("{hero}_anim_{name}_{i}"))).collect() };
        let size = val("size");
        HeroAnim {
            walk: set("walk"),
            idle: set("idle"),
            wattack: set("wattack"),
            attack: set("attack"),
            attack_release: val("attack_release")[0] as usize,
            wattack_release: val("wattack_release")[0] as usize,
            fps: val("fps")[0],
            walk_start: val("walk_start")[0] as usize,
            resume: val("wattack_resume")[0] as usize,
            size: vec2(size[0], size[1]),
        }
    }

    /// Seconds a throw lasts, and from its start to the release.
    fn throw_times(&self, walking: bool) -> (f64, f64) {
        let (frames, release) = if walking { (self.wattack.len(), self.wattack_release) } else { (self.attack.len(), self.attack_release) };
        (frames as f64 / self.fps as f64, release as f64 / self.fps as f64)
    }
}

/// What the animated hero is doing.
#[derive(Clone, Copy, Default, PartialEq)]
pub enum AnimMode {
    #[default]
    Idle,
    Walk,
    /// Standing throw.
    Attack,
    /// Throw mid-stride.
    WalkAttack,
}

pub struct Assets {
    /// Per hero (in `ALL_HEROES` order): idle, walk A, walk B.
    heroes: Vec<[Texture2D; 3]>,
    /// Riku in Ninefold Foxfire form (idle, walk A, walk B).
    riku_fox: [Texture2D; 3],
    /// Shun in Hannya Unbound form, and his blade in both forms.
    shun_hannya: [Texture2D; 3],
    toya_mukade: [Texture2D; 3],
    kuon_domain: [Texture2D; 3],
    ren_divine: [Texture2D; 3],
    /// Ren's rigged animation (tools/ai/rig.py): walk cycle and breathing idle.
    pub ren_anim: HeroAnim,
    /// Head crop of the front view, for the HUD portrait circle.
    pub ren_face: Texture2D,
    cleaver: Texture2D,
    cleaver_hannya: Texture2D,
    cleaver_rest: Texture2D,
    cleaver_rest_hannya: Texture2D,
    familiar: Texture2D,
    /// Floor pickups, in `ALL_PICKUPS` order.
    pickups: Vec<Texture2D>,
    shuriken: Texture2D,
    pub coin: Texture2D,
    pub wisp: Texture2D,
    pub kasa: Texture2D,
    pub oni: Texture2D,
    pub chochin: Texture2D,
    pub nopperabo: Texture2D,
    pub night_lord: Texture2D,
    oni_outline: Texture2D,
    night_lord_outline: Texture2D,
    pub talisman: Texture2D,
    pub orb: Texture2D,
    pub gem: Texture2D,
    pub chest: Texture2D,
    pub chest_open: Texture2D,
    pub shrine: Texture2D,
    icons: Vec<(&'static str, Texture2D)>,
    /// Campaign enemies, indexed like `ALL_ENEMIES` (None for the prologue's own sprites).
    world_enemies: Vec<Option<Texture2D>>,
    /// White silhouettes of the Uncommon-and-above campaign enemies, same indexing.
    enemy_outlines: Vec<Option<Texture2D>>,
    gear_icons: Vec<(&'static str, Texture2D)>,
    /// Soft round shadow, batched under crowds.
    pub shadow: Texture2D,
}

impl Assets {
    pub fn load() -> Self {
        let base = sprites::px("heroine");
        // Legs occupy rows 27..32; left leg columns 5..10, right leg 14..19 (Riku matches this frame).
        let frames = |hero: sprites::Sprite| {
            let walk_a = hero.lift(5..10, 27..32);
            let walk_b = hero.lift(14..19, 27..32);
            [hero.texture(), walk_a.texture(), walk_b.texture()]
        };
        let (oni, oni_outline) = sprites::art_and_outline("oni");
        let (night_lord, night_lord_outline) = sprites::art_and_outline("night_lord");
        let (world_enemies, enemy_outlines): (Vec<_>, Vec<_>) = sim::content::ALL_ENEMIES
            .iter()
            .map(|k| match world_enemy_src(*k) {
                Some(src) => {
                    let sprite = sprites::sprite(src);
                    let outline = (k.rarity() > sim::enemies::EnemyRarity::Common).then(|| sprites::silhouette(&sprite));
                    (Some(sprite.texture()), outline)
                }
                None => (None, None),
            })
            .unzip();
        let heroes = ALL_HEROES
            .iter()
            .map(|h| match h {
                HeroId::Riku => frames(sprites::sprite("riku")),
                HeroId::Shun => frames(sprites::sprite("shun")),
                HeroId::Toya => frames(sprites::sprite("toya")),
                HeroId::Kuon => frames(sprites::sprite("kuon")),
                // Front view for portraits, the armory and menus (in a run she uses ren_anim).
                HeroId::Ren => {
                    let front = sprites::hi_art("ren_front");
                    [front.clone(), front.clone(), front]
                }
                _ => frames(sprites::parse(&sprites::recolor(base, hero_palette(*h)))),
            })
            .collect();
        Assets {
            heroes,
            riku_fox: frames(sprites::sprite("riku_fox")),
            shun_hannya: frames(sprites::sprite("shun_hannya")),
            toya_mukade: frames(sprites::sprite("toya_mukade")),
            kuon_domain: frames(sprites::sprite("kuon_domain")),
            ren_divine: frames(sprites::sprite("ren_divine")),
            ren_anim: HeroAnim::load("ren", include_str!("../../../art/hd/ren_anim.txt")),
            ren_face: sprites::hi_art("ren_face"),
            cleaver: sprites::art("cleaver"),
            cleaver_hannya: sprites::art("cleaver_hannya"),
            cleaver_rest: sprites::art("cleaver_rest"),
            cleaver_rest_hannya: sprites::art("cleaver_rest_hannya"),
            familiar: sprites::art("familiar"),
            pickups: vec![
                sprites::art("pickups/feast"),
                sprites::art("pickups/swift_geta"),
                sprites::art("pickups/oni_sake"),
                sprites::art("pickups/shrine_bell"),
                sprites::art("pickups/hourglass"),
                sprites::art("pickups/exorcism_seal"),
                sprites::art("pickups/sen_purse"),
            ],
            shuriken: sprites::art("shuriken"),
            coin: sprites::art("coin"),
            wisp: sprites::art("wisp"),
            kasa: sprites::art("kasa"),
            oni,
            chochin: sprites::art("chochin"),
            nopperabo: sprites::art("nopperabo"),
            night_lord,
            oni_outline,
            night_lord_outline,
            talisman: sprites::art("talisman"),
            orb: sprites::art("orb"),
            gem: sprites::art("gem"),
            chest: sprites::art("chest"),
            chest_open: sprites::art("chest_open"),
            shrine: sprites::art("shrine"),
            icons: vec![
            ("abundance", sprites::art("icons/abundance")),
            ("crescent", sprites::art("icons/crescent")),
            ("expanse", sprites::art("icons/expanse")),
            ("flame_pool", sprites::art("icons/flame_pool")),
            ("foxfire_orb", sprites::art("icons/foxfire_orb")),
            ("familiars", sprites::art("icons/familiars")),
            ("gale_shuriken", sprites::art("icons/gale_shuriken")),
            ("fox_mirage", sprites::art("icons/fox_mirage")),
            ("grave_cleaver", sprites::art("icons/grave_cleaver")),
            ("soul_tether", sprites::art("icons/soul_tether")),
            ("lantern_vigil", sprites::art("icons/lantern_vigil")),
            ("iai_flash", sprites::art("icons/iai_flash")),
            ("crimson_tendrils", sprites::art("icons/crimson_tendrils")),
            ("mukade_spines", sprites::art("icons/mukade_spines")),
            ("devour", sprites::art("icons/devour")),
            ("venom_fang", sprites::art("icons/venom_fang")),
            ("gravity_well", sprites::art("icons/gravity_well")),
            ("repulsion_star", sprites::art("icons/repulsion_star")),
            ("void_lance", sprites::art("icons/void_lance")),
            ("yinyang_orbs", sprites::art("icons/yinyang_orbs")),
            ("spirit_seal", sprites::art("icons/spirit_seal")),
            ("purification_wave", sprites::art("icons/purification_wave")),
            ("guard", sprites::art("icons/guard")),
            ("haste", sprites::art("icons/haste")),
            ("magnetism", sprites::art("icons/magnetism")),
            ("might", sprites::art("icons/might")),
            ("moon_orbs", sprites::art("icons/moon_orbs")),
            ("ofuda", sprites::art("icons/ofuda")),
            ("onigiri", sprites::art("icons/onigiri")),
            ("persistence", sprites::art("icons/persistence")),
            ("recovery", sprites::art("icons/recovery")),
            ("sakura_field", sprites::art("icons/sakura_field")),
            ("star_fall", sprites::art("icons/star_fall")),
            ("swiftness", sprites::art("icons/swiftness")),
            ("taiko", sprites::art("icons/taiko")),
            ("thunder_chain", sprites::art("icons/thunder_chain")),
            ("vitality", sprites::art("icons/vitality")),
            ("relic_w1", sprites::art("icons/relic_w1")),
            ("relic_w2", sprites::art("icons/relic_w2")),
            ("relic_w3", sprites::art("icons/relic_w3")),
            ("relic_w4", sprites::art("icons/relic_w4")),
            ("relic_w5", sprites::art("icons/relic_w5")),
            ],
            world_enemies,
            enemy_outlines,
            shadow: {
                let (w, h) = (32u16, 16u16);
                let mut px = vec![0u8; w as usize * h as usize * 4];
                for y in 0..h {
                    for x in 0..w {
                        let (dx, dy) = ((x as f32 + 0.5) / w as f32 * 2.0 - 1.0, (y as f32 + 0.5) / h as f32 * 2.0 - 1.0);
                        let d = (dx * dx + dy * dy).sqrt();
                        let i = (y as usize * w as usize + x as usize) * 4;
                        px[i + 3] = ((1.0 - d).clamp(0.0, 1.0).powf(0.6) * 255.0) as u8;
                    }
                }
                let t = Texture2D::from_rgba8(w, h, &px);
                t.set_filter(FilterMode::Linear);
                t
            },
            gear_icons: vec![
            ("lantern_heart", sprites::art("exotic/lantern_heart")),
            ("jeweled_branch", sprites::art("exotic/jeweled_branch")),
            ("unbroken_yinyang", sprites::art("exotic/unbroken_yinyang")),
            ("nine_seal", sprites::art("exotic/nine_seal")),
            ("clone_scroll", sprites::art("exotic/clone_scroll")),
            ("hollow_mask", sprites::art("exotic/hollow_mask")),
            ("white_haori", sprites::art("exotic/white_haori")),
            ("hunger_organ", sprites::art("exotic/hunger_organ")),
            ("thousand_counter", sprites::art("exotic/thousand_counter")),
            ("sovereign_finger", sprites::art("exotic/sovereign_finger")),
            ("closed_hand_vow", sprites::art("exotic/closed_hand_vow")),
            ("kusanagi", sprites::art("exotic/kusanagi")),
            ("yata_mirror", sprites::art("exotic/yata_mirror")),
            ("yasakani_jewel", sprites::art("exotic/yasakani_jewel")),
            ("raijin_drums", sprites::art("exotic/raijin_drums")),
            ("fox_procession", sprites::art("exotic/fox_procession")),
            ("tengu_fan", sprites::art("exotic/tengu_fan")),
            ("dragon_coil", sprites::art("exotic/dragon_coil")),
            ("moonfall", sprites::art("exotic/moonfall")),
            ("skeleton_hand", sprites::art("exotic/skeleton_hand")),
            ("hat", sprites::art("gear/hat")),
            ("top", sprites::art("gear/top")),
            ("bottom", sprites::art("gear/bottom")),
            ("gloves", sprites::art("gear/gloves")),
            ("shoes", sprites::art("gear/shoes")),
            ("necklace", sprites::art("gear/necklace")),
            ("ring", sprites::art("gear/ring")),
            ("earring", sprites::art("gear/earring")),
            ("charm", sprites::art("gear/charm")),
            ("omamori", sprites::art("gear/omamori")),
            ("arsenal", sprites::art("gear/arsenal")),
            ("dust", sprites::art("gear/dust")),
            ("draw", sprites::art("gear/draw")),
            ],
        }
    }

    pub fn gear_icon(&self, s: sim::gear::Slot) -> &Texture2D {
        self.gear_icon_named(s.key())
    }

    pub fn gear_icon_named(&self, key: &str) -> &Texture2D {
        &self.gear_icons.iter().find(|(n, _)| *n == key).expect("gear icon").1
    }

    /// Any enemy's sprite (prologue kinds use their named textures).
    pub fn enemy_tex(&self, k: EnemyKind) -> &Texture2D {
        if let Some(t) = &self.world_enemies[k.index()] {
            return t;
        }
        match k {
            EnemyKind::Wisp => &self.wisp,
            EnemyKind::Kasa => &self.kasa,
            EnemyKind::Chochin => &self.chochin,
            EnemyKind::Nopperabo => &self.nopperabo,
            EnemyKind::NightLord => &self.night_lord,
            _ => &self.oni,
        }
    }

    pub fn hero(&self, h: HeroId) -> &[Texture2D; 3] {
        &self.heroes[ALL_HEROES.iter().position(|x| *x == h).unwrap()]
    }

    /// The hero's current look (transformed form if active), idle frame.
    pub fn portrait(&self, h: HeroId, form: bool) -> &Texture2D {
        match (h, form) {
            (HeroId::Riku, true) => &self.riku_fox[0],
            (HeroId::Shun, true) => &self.shun_hannya[0],
            (HeroId::Toya, true) => &self.toya_mukade[0],
            (HeroId::Kuon, true) => &self.kuon_domain[0],
            (HeroId::Ren, true) => &self.ren_divine[0],
            _ => &self.hero(h)[0],
        }
    }

    pub fn pickup(&self, k: PickupKind) -> &Texture2D {
        &self.pickups[ALL_PICKUPS.iter().position(|x| *x == k).unwrap()]
    }

    pub fn icon_named(&self, key: &str) -> &Texture2D {
        &self.icons.iter().find(|(n, _)| *n == key).expect("icon").1
    }

    pub fn icon(&self, u: Upgrade) -> &Texture2D {
        let key = match u {
            Upgrade::Weapon(k) => match k {
                WeaponKind::Ofuda => "ofuda",
                WeaponKind::MoonOrbs => "moon_orbs",
                WeaponKind::SakuraField => "sakura_field",
                WeaponKind::ThunderChain => "thunder_chain",
                WeaponKind::Taiko => "taiko",
                WeaponKind::StarFall => "star_fall",
                WeaponKind::Crescent => "crescent",
                WeaponKind::FlamePool => "flame_pool",
                WeaponKind::FoxfireOrb => "foxfire_orb",
                WeaponKind::Familiars => "familiars",
                WeaponKind::GaleShuriken => "gale_shuriken",
                WeaponKind::FoxMirage => "fox_mirage",
                WeaponKind::GraveCleaver => "grave_cleaver",
                WeaponKind::SoulTether => "soul_tether",
                WeaponKind::LanternVigil => "lantern_vigil",
                WeaponKind::IaiFlash => "iai_flash",
                WeaponKind::CrimsonTendrils => "crimson_tendrils",
                WeaponKind::MukadeSpines => "mukade_spines",
                WeaponKind::Devour => "devour",
                WeaponKind::VenomFang => "venom_fang",
                WeaponKind::GravityWell => "gravity_well",
                WeaponKind::RepulsionStar => "repulsion_star",
                WeaponKind::VoidLance => "void_lance",
                WeaponKind::YinYangOrbs => "yinyang_orbs",
                WeaponKind::SpiritSeal => "spirit_seal",
                WeaponKind::PurificationWave => "purification_wave",
            },
            Upgrade::Passive(p) => match p {
                PassiveKind::Might => "might",
                PassiveKind::Haste => "haste",
                PassiveKind::Expanse => "expanse",
                PassiveKind::Swiftness => "swiftness",
                PassiveKind::Magnet => "magnetism",
                PassiveKind::Vitality => "vitality",
                PassiveKind::Abundance => "abundance",
                PassiveKind::Persistence => "persistence",
                PassiveKind::Recovery => "recovery",
                PassiveKind::Guard => "guard",
            },
            Upgrade::Heal => "onigiri",
            Upgrade::Sen => return &self.coin,
            Upgrade::Blessing => "might",
        };
        &self.icons.iter().find(|(n, _)| *n == key).expect("icon").1
    }
}

/// Placeholder looks: Ren's sprite with each hero's own hair / outfit / eye / ribbon colours.
/// Palette chars: H/h/L hair, C/c outfit, I eyes, R ribbon.
fn hero_palette(h: HeroId) -> &'static [(char, &'static str)] {
    match h {
        HeroId::Ren | HeroId::Riku | HeroId::Shun | HeroId::Toya | HeroId::Kuon => &[],
        HeroId::Kaede => &[('H', "#33334a"), ('h', "#1c1c2a"), ('L', "#5a5a7a"), ('C', "#2f5a3a"), ('c', "#1d3a25"), ('I', "#e84a4a"), ('R', "#c9a040")],
        HeroId::Mika => &[('H', "#ffd45a"), ('h', "#d49a2a"), ('L', "#fff0a0"), ('C', "#e88ac8"), ('c', "#b0609a"), ('I', "#4ac0ff"), ('R', "#ff5aa0")],
        HeroId::Haru => &[('H', "#c8cde0"), ('h', "#8890aa"), ('L', "#ffffff"), ('C', "#3a3a8a"), ('c', "#24245a"), ('I', "#6a7cff"), ('R', "#e83a3a")],
        HeroId::Nanami => &[('H', "#3ac0a8"), ('h', "#1f8070"), ('L', "#9af0e0"), ('C', "#8a5a3a"), ('c', "#5a3a24"), ('I', "#a06aff"), ('R', "#ffb040")],
        HeroId::Yuzu => &[('H', "#ff9a3a"), ('h', "#c86420"), ('L', "#ffd0a0"), ('C', "#b05ad8"), ('c', "#7a3aa0"), ('I', "#ff6aa0"), ('R', "#4ad0ff")],
    }
}

struct DamageNumber {
    /// From the player, so numbers ride along with the camera instead of drifting off.
    offset: Vec2,
    txt: String,
    size: f32,
    /// Measured once at push so drawing never formats or measures.
    width: f32,
    age: f32,
    crit: bool,
    elem: Option<Element>,
    mult: f32,
}

impl DamageNumber {
    fn new(offset: Vec2, amount: f32, crit: bool, elem: Option<Element>, mult: f32) -> Self {
        let base = if crit { 24.0 } else { 18.0 };
        let size = if elem.is_some() && mult > 1.0 { if crit { 28.0 } else { 22.0 } } else { base };
        let txt = format!("{amount:.0}");
        let width = kit::measure(&txt, size as u16).width;
        DamageNumber { offset, txt, size, width, age: 0.0, crit, elem, mult }
    }
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    age: f32,
    life: f32,
    color: Color,
}

/// Purely visual state derived from sim events.
#[derive(Default)]
pub struct Fx {
    /// The animated hero's state: mode, when it began (s), and the walk frame it started on.
    pub anim: (AnimMode, f64, usize),
    numbers: Vec<DamageNumber>,
    particles: Vec<Particle>,
    /// Expanding barrier rings: (centre, age).
    rings: Vec<(Vec2, f32)>,
    /// Thunder Chain segments: (from, to, age).
    bolts: Vec<(Vec2, Vec2, f32)>,
    /// Crescent Slash arcs: (pos, dir, radius, full, age).
    slashes: Vec<(Vec2, Vec2, f32, bool, f32)>,
    /// Star Fall impacts: (pos, radius, age).
    impacts: Vec<(Vec2, f32, f32)>,
    /// Bursts: (pos, radius, age, style). Style: 0 foxfire blue, 1 gold blast, 2 crimson, 3 finale.
    bursts: Vec<(Vec2, f32, f32, u8)>,
    free_bursts: Vec<(Vec2, f32, f32, u8)>,
    /// Beams: (from, to, age, style).
    beams: Vec<(Vec2, Vec2, f32, u8)>,
    /// Signature flourishes under the player: age.
    sig_rings: Vec<f32>,
    /// Iai Flash cuts: (from, to, age).
    iai: Vec<(Vec2, Vec2, f32)>,
    /// Tendril lashes: (from, to, age).
    lashes: Vec<(Vec2, Vec2, f32)>,
    /// Devour pulses: (pos, radius, age).
    devours: Vec<(Vec2, f32, f32)>,
    /// Recent player positions for Rampage afterimages: (pos, facing left, age).
    trail: Vec<(Vec2, bool, f32)>,
    trail_tick: f32,
    /// Player position last frame (for effects spawned by events without a position).
    pub last_player: Vec2,
    /// Lantern pulse rings: (pos, radius, age).
    pulses: Vec<(Vec2, f32, f32)>,
    /// Shadow Step streaks: (from, to, age).
    steps: Vec<(Vec2, Vec2, f32)>,
    /// Sword swing: seconds left and direction.
    pub swing: (f32, Vec2),

    /// Smoke puffs from Fox Mirage clones appearing / popping: (pos, radius, age).
    smokes: Vec<(Vec2, f32, f32)>,
    free_smokes: Vec<(Vec2, f32, f32)>,
    pub shake: f32,
    pub show_numbers: bool,
    /// Short line spoken by the hero, shown above her head.
    pub bark: Option<(String, f32)>,
    /// Big centre-screen announcement.
    pub banner: Option<(String, f32)>,
    leveled_once: bool,
    pub low_hp_warned: bool,
    hero: Option<HeroId>,
    /// The stage's accent colour (kill sparks, boss auras).
    pub accent: Color,
    /// Lightning flash, 0..1, decays.
    pub flash: f32,
    number_tick: u32,
    /// Equipment drops: (pos, rarity, age).
    /// (world pos, rarity, age, label, label width) — label and width are cached at push.
    gear_drops: Vec<(Vec2, sim::gear::Rarity, f32, String, f32)>,
}

const MAX_NUMBERS: usize = 40;
const MAX_PARTICLES: usize = 360;

impl Fx {
    pub fn new() -> Self {
        Fx { show_numbers: true, ..Default::default() }
    }

    pub fn clear(&mut self) {
        self.numbers.clear();
        self.particles.clear();
        self.rings.clear();
        self.bolts.clear();
        self.slashes.clear();
        self.impacts.clear();
        self.bursts.clear();
        self.free_bursts.clear();
        self.smokes.clear();
        self.free_smokes.clear();
        self.beams.clear();
        self.sig_rings.clear();
        self.steps.clear();
        self.iai.clear();
        self.lashes.clear();
        self.devours.clear();
        self.trail.clear();
        self.pulses.clear();
        self.gear_drops.clear();
        self.shake = 0.0;
        self.banner = None;
        self.leveled_once = false;
        self.low_hp_warned = false;
        self.say(self.lines().start);
    }

    pub fn set_hero(&mut self, h: HeroId) {
        self.hero = Some(h);
    }

    pub fn lines(&self) -> voice::Lines {
        voice::lines(self.hero.unwrap_or(HeroId::Ren))
    }

    /// A ring of `n` sparks flying out from `pos`.
    fn spray(&mut self, pos: Vec2, n: usize, color: Color) {
        for k in 0..n {
            if self.particles.len() >= MAX_PARTICLES {
                break;
            }
            let a = k as f32 / n as f32 * std::f32::consts::TAU + rand::gen_range(-0.3, 0.3);
            let speed = rand::gen_range(30.0, 90.0);
            self.particles.push(Particle { pos, vel: vec2(a.cos(), a.sin()) * speed, age: 0.0, life: rand::gen_range(0.25, 0.45), color });
        }
    }

    pub fn say(&mut self, line: &str) {
        self.bark = Some((line.to_string(), 2.2));
    }

    pub fn announce(&mut self, text: String) {
        self.banner = Some((text, 2.8));
    }

    pub fn absorb(&mut self, events: &[Event], player: sim::Vec2, reach: f32) {
        let mut round_banner = None;
        let boss_arrived = events.iter().any(|e| matches!(e, Event::BossSpawned(_) | Event::NightLordSpawned(_)));
        for ev in events {
            match *ev {
                // Numbers are sampled so crowds don't bury the screen: crits and big hits always
                // show, ordinary hits one in six.
                Event::Hit { pos, amount, crit, elem, mult } if self.show_numbers && (crit || amount >= 150.0 || { self.number_tick = self.number_tick.wrapping_add(1); self.number_tick % 6 == 0 }) => {
                    if self.numbers.len() >= MAX_NUMBERS {
                        self.numbers.remove(0);
                    }
                    self.numbers.push(DamageNumber::new(vec2(pos.x - player.x, pos.y - 8.0 - player.y), amount, crit, elem, mult));
                }
                Event::Kill { pos, kind } => {
                    let (n, color) = match kind {
                        EnemyKind::Wisp => (5, Color::from_rgba(155, 111, 214, 255)),
                        EnemyKind::Kasa => (6, Color::from_rgba(194, 59, 59, 255)),
                        EnemyKind::Oni => (24, Color::from_rgba(240, 122, 90, 255)),
                        EnemyKind::GreatOni => (80, Color::from_rgba(255, 200, 90, 255)),
                        EnemyKind::Chochin => (8, Color::from_rgba(255, 150, 70, 255)),
                        EnemyKind::Nopperabo => (8, Color::from_rgba(200, 180, 230, 255)),
                        EnemyKind::NightLord => (200, Color::from_rgba(255, 220, 120, 255)),
                        k => match k.rank() {
                            Rank::Fodder => (6, self.accent),
                            Rank::Elite => (24, self.accent),
                            Rank::Mini => (80, Color::from_rgba(255, 200, 90, 255)),
                            Rank::Boss => (140, Color::from_rgba(255, 210, 110, 255)),
                            Rank::Final => (200, Color::from_rgba(255, 220, 120, 255)),
                        },
                    };
                    if kind.rank() == Rank::Final {
                        self.shake = self.shake.max(0.6);
                        self.say(self.lines().night_lord_down);
                    }
                    self.spray(vec2(pos.x, pos.y), n, color);
                }
                Event::PlayerHurt => self.shake = self.shake.max(0.18),
                Event::LevelUp if !self.leveled_once => {
                    self.leveled_once = true;
                    self.say(self.lines().first_level);
                }
                Event::SpecialUsed { pos } => {
                    self.rings.push((vec2(pos.x, pos.y), 0.0));
                    self.shake = self.shake.max(0.25);
                    self.say(self.lines().special);
                }
                Event::RoundStarted { round, headline: Some(h) } => {
                    round_banner = Some(format!("ROUND {round} — {}", crate::ui::headline_name(h)));
                }
                Event::BossSpawned(kind) => {
                    let name = if kind == EnemyKind::GreatOni { "A GREAT ONI".to_string() } else { kind.name().to_uppercase() };
                    self.announce(format!("{name} APPROACHES"));
                    self.say(self.lines().boss);
                }
                Event::LeaderSpawned { kind } => {
                    if !boss_arrived {
                        self.announce(format!("{} rallies the crew", kind.name()));
                    }
                    self.shake = self.shake.max(0.2);
                }
                Event::EliteWave => {
                    self.announce("ELITE WAVE".into());
                    self.shake = self.shake.max(0.2);
                }
                Event::NightLordSpawned(kind) => {
                    self.announce(if kind == EnemyKind::NightLord { "THE NIGHT LORD RISES".into() } else { format!("{} DESCENDS", kind.name().to_uppercase()) });
                    self.say(self.lines().night_lord);
                    self.shake = self.shake.max(0.4);
                }
                Event::BossPhase { pos } => {
                    self.free_bursts.push((vec2(pos.x, pos.y), 90.0, 0.0, 1));
                    self.shake = self.shake.max(0.35);
                }
                Event::Hazard { pos, radius, kind, friendly } => {
                    let style = match kind {
                        0 => style::METEOR,
                        1 => style::SKYBOLT,
                        _ => 3,
                    };
                    let list = if friendly { &mut self.bursts } else { &mut self.free_bursts };
                    list.push((vec2(pos.x, pos.y), radius, 0.0, style));
                    if kind == 1 {
                        self.flash = 1.0;
                    }
                    self.shake = self.shake.max(if kind == 2 { 0.3 } else { 0.12 });
                }
                Event::Curtain => {}
                Event::GearDrop { pos, rarity } => {
                    let label = format!("{} gear", rarity.name());
                    let width = kit::measure(&label, 15).width;
                    self.gear_drops.push((vec2(pos.x, pos.y), rarity, 0.0, label, width));
                    if rarity >= sim::gear::Rarity::Legendary {
                        self.announce(format!("{} GEAR!", rarity.name().to_uppercase()));
                    }
                }
                Event::Blink { pos } => {
                    for k in 0..10 {
                        if self.particles.len() >= MAX_PARTICLES {
                            break;
                        }
                        let a = k as f32 / 10.0 * std::f32::consts::TAU;
                        self.particles.push(Particle {
                            pos: vec2(pos.x, pos.y - 8.0),
                            vel: vec2(a.cos(), a.sin()) * 40.0,
                            age: 0.0,
                            life: 0.35,
                            color: Color::from_rgba(190, 170, 230, 255),
                        });
                    }
                }
                Event::BossTelegraph => self.shake = self.shake.max(0.08),
                Event::ShrineAppeared => {
                    self.announce("A ROADSIDE SHRINE APPEARS".into());
                    self.say(self.lines().shrine);
                }
                Event::ShrineChosen(ev) => self.say(&format!("{}.", ev.name())),
                Event::Revived => {
                    self.announce("REVIVAL".into());
                    self.say(self.lines().revived);
                    self.shake = self.shake.max(0.3);
                }
                // A weapon fired: throw (unless mid-throw; a short gap keeps it from flailing).
                Event::Bolt { from, to } => self.bolts.push((vec2(from.x, from.y), vec2(to.x, to.y), 0.0)),
                Event::Slash { pos, dir, radius, full } => {
                    self.slashes.push((vec2(pos.x, pos.y), vec2(dir.x, dir.y), radius, full, 0.0));
                    if self.swing.0 > 0.0 {
                        self.swing.1 = vec2(dir.x, dir.y);
                    }
                }
                Event::Impact { pos, radius } => {
                    self.impacts.push((vec2(pos.x, pos.y), radius, 0.0));
                    self.shake = self.shake.max(0.04);
                }
                Event::Burst { pos, radius, style } => {
                    self.bursts.push((vec2(pos.x, pos.y), radius, 0.0, style));
                    match style {
                        style::METEOR => self.shake = self.shake.max(0.3),
                        style::QUAKE => self.shake = self.shake.max(0.2),
                        style::SKYBOLT => self.shake = self.shake.max(0.06),
                        _ => {}
                    }
                }
                Event::Signature(_) => self.sig_rings.push(0.0),
                Event::HannyaAwakened { pos } => {
                    self.bursts.push((vec2(pos.x, pos.y), sim::content::HANNYA_ONSET_RADIUS.min(reach), 0.0, 2));
                    self.announce("HANNYA UNBOUND".into());
                    self.say(self.lines().special);
                    self.shake = self.shake.max(0.5);
                }
                Event::Beam { from, to, style } => self.beams.push((vec2(from.x, from.y), vec2(to.x, to.y), 0.0, style)),
                Event::Iai { from, to } => {
                    self.iai.push((vec2(from.x, from.y), vec2(to.x, to.y), 0.0));
                    self.shake = self.shake.max(0.05);
                }
                Event::LanternPulse { pos, radius } => self.pulses.push((vec2(pos.x, pos.y), radius, 0.0)),
                Event::ShadowStep { from, to } => self.steps.push((vec2(from.x, from.y), vec2(to.x, to.y), 0.0)),
                Event::AfterimageBlast { pos, radius } => {
                    self.bursts.push((vec2(pos.x, pos.y), radius, 0.0, 2));
                    self.shake = self.shake.max(0.08);
                }
                Event::FinaleCharge { .. } => self.say("This ends now!"),
                Event::FinaleBlast { pos, radius } => {
                    self.bursts.push((vec2(pos.x, pos.y), radius, 0.0, 3));
                    self.announce("HANNYA'S WRATH".into());
                    self.shake = self.shake.max(0.8);
                }
                Event::Fired(WeaponKind::GraveCleaver) => self.swing = (0.2, Vec2::ZERO),
                Event::FoxAwakened { pos } => {
                    self.bursts.push((vec2(pos.x, pos.y), sim::content::FOX_BLAST_RADIUS.min(reach), 0.0, 1));
                    self.announce("NINEFOLD FOXFIRE".into());
                    self.say(self.lines().special);
                    self.shake = self.shake.max(0.5);
                }
                Event::FoxFaded => {
                    if self.hero == Some(HeroId::Riku) {
                        self.say("Phew... the foxfire's cooling.");
                    }
                }
                Event::PickupTaken { pos, kind } => {
                    self.say(&format!("{} ({})! {}", kind.name(), kind.tier().name(), kind.describe()));
                    if kind == PickupKind::ExorcismSeal {
                        self.bursts.push((vec2(pos.x, pos.y), sim::content::EXORCISM_RADIUS, 0.0, 1));
                        self.shake = self.shake.max(0.5);
                    }
                }
                Event::LeaderFell { pos } => {
                    self.free_bursts.push((vec2(pos.x, pos.y), sim::content::BANNER_RADIUS, 0.0, LEADER_FELL));
                    self.spray(vec2(pos.x, pos.y), 28, leader_color());
                    self.shake = self.shake.max(0.3);
                }
                Event::PickupStolen { pos, kind } => {
                    self.free_smokes.push((vec2(pos.x, pos.y), 14.0, 0.0));
                    self.say(&format!("Hey! That thief took the {}!", kind.name()));
                }
                Event::Lash { from, to } => self.lashes.push((vec2(from.x, from.y), vec2(to.x, to.y), 0.0)),
                Event::WellCollapse { pos, radius } => {
                    self.bursts.push((vec2(pos.x, pos.y), radius, 0.0, 16));
                    self.shake = self.shake.max(0.08);
                }
                Event::Warded => self.bursts.push((self.last_player, 26.0, 0.0, 17)),
                Event::BarrierBlock => self.bursts.push((self.last_player, 22.0, 0.0, 21)),
                Event::DivineAwakened { .. } => {
                    self.announce("DIVINE PURIFICATION".into());
                    self.say(self.lines().special);
                    self.shake = self.shake.max(0.4);
                }
                Event::DivineCharge { .. } => self.say("Be purified... all of you!"),
                Event::DomainOpened { .. } => {
                    self.announce("DOMAIN: ENDLESS FIRMAMENT".into());
                    self.say(self.lines().special);
                    self.shake = self.shake.max(0.4);
                }
                Event::VoidCharge { .. } => self.say("Now... erase."),
                Event::VoidMaximum { pos } => {
                    self.bursts.push((vec2(pos.x, pos.y), 140.0, 0.0, 18));
                    self.announce("VOID LANCE: MAXIMUM".into());
                    self.shake = self.shake.max(0.7);
                }
                Event::Devour { pos, radius } => self.devours.push((vec2(pos.x, pos.y), radius, 0.0)),
                Event::MukadeAwakened { .. } => {
                    self.announce("HUNDRED-LEG RAMPAGE".into());
                    self.say(self.lines().special);
                    self.shake = self.shake.max(0.55);
                }
                Event::CloneSummoned { pos } => self.smokes.push((vec2(pos.x, pos.y), 16.0, 0.0)),
                Event::ClonePopped { pos, radius } => self.smokes.push((vec2(pos.x, pos.y), radius, 0.0)),
                Event::Awakened(k) => {
                    self.announce(format!("AWAKENING: {}", sim::content::awakened_name(k)));
                    self.say(self.lines().awaken);
                }
                _ => {}
            }
        }
        if let Some(text) = round_banner {
            self.banner = Some((text, 2.5));
        }
    }

    /// Records Rampage afterimages while Toya moves in his form.
    pub fn track_trail(&mut self, world: &World, dt: f32) {
        self.last_player = vec2(world.player.pos.x, world.player.pos.y - 8.0);
        self.trail_tick -= dt;
        if world.form_kind() == Some(SpecialKind::Mukade) && world.player.moving && self.trail_tick <= 0.0 {
            self.trail_tick = 0.05;
            let p = world.player.pos;
            self.trail.push((vec2(p.x, p.y), world.player.facing.x < 0.0, 0.0));
        }
    }

    /// Advances the animated hero (once per frame, before drawing). Walk and idle loop. A throw
    /// (standing or mid-stride) starts early enough that its release frame lands exactly when
    /// the signature weapon fires (the sim reports its timer), then the walk resumes on the
    /// matching frame.
    pub fn animate(&mut self, anim: &HeroAnim, world: &World, now: f64) {
        let moving = world.player.moving;
        let (mode, since, _) = self.anim;
        let throwing = matches!(mode, AnimMode::Attack | AnimMode::WalkAttack);
        if throwing && now - since < anim.throw_times(mode == AnimMode::WalkAttack).0 {
            return;
        }
        let next = world.next_fire_in(sim::content::hero_def(world.cfg.hero).signature);
        let (_, lead) = anim.throw_times(moving);
        self.anim = match next {
            Some(t) if !world.paused() && (t as f64) <= lead => {
                // Start partway in if the shot is sooner than the wind-up.
                (if moving { AnimMode::WalkAttack } else { AnimMode::Attack }, now - (lead - t as f64), 0)
            }
            _ if throwing => {
                if moving {
                    (AnimMode::Walk, now, anim.resume)
                } else {
                    (AnimMode::Idle, now, 0)
                }
            }
            _ if mode == AnimMode::Idle && moving => (AnimMode::Walk, now, anim.walk_start),
            _ if mode == AnimMode::Walk && !moving => (AnimMode::Idle, now, 0),
            _ => self.anim,
        };
    }

    /// The animated hero's current frame.
    pub fn anim_frame<'a>(&self, anim: &'a HeroAnim, now: f64) -> &'a Texture2D {
        let (mode, since, start) = self.anim;
        let n = ((now - since).max(0.0) * anim.fps as f64) as usize;
        match mode {
            AnimMode::Idle => &anim.idle[n % anim.idle.len()],
            AnimMode::Walk => &anim.walk[(start + n) % anim.walk.len()],
            AnimMode::Attack => &anim.attack[n.min(anim.attack.len() - 1)],
            AnimMode::WalkAttack => &anim.wattack[n.min(anim.wattack.len() - 1)],
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.flash = (self.flash - dt * 4.0).max(0.0);
        for g in &mut self.gear_drops {
            g.2 += dt;
        }
        self.gear_drops.retain(|g| g.2 < 2.2);
        for n in &mut self.numbers {
            n.age += dt;
            n.offset.y -= 28.0 * dt;
        }
        self.numbers.retain(|n| n.age < 0.6);
        for p in &mut self.particles {
            p.age += dt;
            p.pos += p.vel * dt;
            p.vel *= 0.9;
        }
        self.particles.retain(|p| p.age < p.life);
        for r in &mut self.rings {
            r.1 += dt;
        }
        self.rings.retain(|r| r.1 < 0.45);
        self.bolts.iter_mut().for_each(|b| b.2 += dt);
        self.bolts.retain(|b| b.2 < 0.16);
        self.slashes.iter_mut().for_each(|s| s.4 += dt);
        self.slashes.retain(|s| s.4 < 0.2);
        self.impacts.iter_mut().for_each(|i| i.2 += dt);
        self.impacts.retain(|i| i.2 < 0.3);
        self.bursts.iter_mut().for_each(|b| b.2 += dt);
        self.bursts.retain(|b| b.2 < burst_life(b.3));
        self.free_bursts.iter_mut().for_each(|b| b.2 += dt);
        self.free_bursts.retain(|b| b.2 < burst_life(b.3));
        self.beams.iter_mut().for_each(|b| b.2 += dt);
        self.beams.retain(|b| b.2 < if b.3 == style::MOON { 0.45 } else { 0.3 });
        self.sig_rings.iter_mut().for_each(|r| *r += dt);
        self.sig_rings.retain(|r| *r < 0.5);
        self.iai.iter_mut().for_each(|b| b.2 += dt);
        self.iai.retain(|b| b.2 < 0.25);
        self.lashes.iter_mut().for_each(|b| b.2 += dt);
        self.lashes.retain(|b| b.2 < 0.22);
        self.devours.iter_mut().for_each(|b| b.2 += dt);
        self.devours.retain(|b| b.2 < 0.4);
        self.trail.iter_mut().for_each(|b| b.2 += dt);
        self.trail.retain(|b| b.2 < 0.35);
        self.pulses.iter_mut().for_each(|b| b.2 += dt);
        self.pulses.retain(|b| b.2 < 0.4);
        self.steps.iter_mut().for_each(|b| b.2 += dt);
        self.steps.retain(|b| b.2 < 0.3);
        self.swing.0 = (self.swing.0 - dt).max(0.0);

        self.smokes.iter_mut().for_each(|s| s.2 += dt);
        self.smokes.retain(|s| s.2 < 0.5);
        self.free_smokes.iter_mut().for_each(|s| s.2 += dt);
        self.free_smokes.retain(|s| s.2 < 0.5);
        self.shake = (self.shake - dt).max(0.0);
        for t in [&mut self.bark, &mut self.banner] {
            if let Some((_, left)) = t {
                *left -= dt;
                if *left <= 0.0 {
                    *t = None;
                }
            }
        }
    }
}

/// A campaign enemy: sized by rank, with its telegraphs (charge lines, dash crouch) and a
/// boss aura in the stage's accent colour.
/// Sprites held back so a layer draws all its shapes first, then its sprites grouped by
/// texture: every texture switch is a separate GPU draw call.
struct Sprites<'a>(Vec<(&'a Texture2D, f32, f32, Color, DrawTextureParams)>);

impl<'a> Sprites<'a> {
    fn new() -> Self {
        Sprites(Vec::new())
    }
    fn push(&mut self, t: &'a Texture2D, x: f32, y: f32, c: Color, p: DrawTextureParams) {
        self.0.push((t, x, y, c, p));
    }
    fn flush(&mut self) {
        self.0.sort_by_key(|s| s.0 as *const Texture2D as usize);
        for (t, x, y, c, p) in self.0.drain(..) {
            draw_texture_ex(t, x, y, c, p);
        }
    }
}

/// Darker and greyer: marks an enemy the player's attacks cannot reach.
fn dimmed(c: Color) -> Color {
    let grey = 0.3 * c.r + 0.59 * c.g + 0.11 * c.b;
    let k = 0.55;
    Color::new((grey + (c.r - grey) * 0.5) * k, (grey + (c.g - grey) * 0.5) * k, (grey + (c.b - grey) * 0.5) * k, c.a)
}

#[allow(clippy::too_many_arguments)]
fn draw_world_enemy<'a>(
    world: &World,
    i: usize,
    t: &'a Texture2D,
    outline: Option<&'a Texture2D>,
    tint: Color,
    flip: bool,
    accent: Color,
    time: f64,
    sprites: &mut Sprites<'a>,
    shadows: &mut Sprites<'a>,
    outlines: &mut Sprites<'a>,
    cues: &mut Vec<(f32, f32, EnemyRarity, f32)>,
    shadow: &'a Texture2D,
) {
    use sim::content::{enemy_def, Ai, Rank};
    let e = &world.enemies;
    let (pos, kind) = (e.pos[i], e.kind[i]);
    let rank = kind.rank();
    let ai = enemy_def(kind).ai;
    let scale = match rank {
        Rank::Mini | Rank::Boss => 2.0,
        Rank::Final => 2.2,
        _ => 1.0,
    };
    let (w, h) = { let n = crate::sprites::nat(t); (n.x * scale, n.y * scale) };
    let tf = time as f32;
    let bob = if rank >= Rank::Mini { (tf * 2.0 + i as f32).sin() * 2.5 } else { 0.0 };
    let mut tint = tint;
    let mut warn = false;
    match ai {
        Ai::Charger { .. } if e.ai_state[i] == 1 => {
            let d = e.ai_dir[i];
            draw_line(pos.x, pos.y, pos.x + d.x * 230.0, pos.y + d.y * 230.0, e.radius[i] * 0.8, Color::new(1.0, 0.2, 0.2, 0.22));
            warn = true;
        }
        Ai::Boss if e.ai_busy[i] > 0.7 => {
            let d = e.ai_aux[i];
            draw_line(pos.x, pos.y, pos.x + d.x * 320.0, pos.y + d.y * 320.0, e.radius[i] * 1.4, Color::new(1.0, 0.2, 0.2, 0.2));
            warn = true;
        }
        Ai::Dash if e.ai_busy[i] > 0.5 => {
            let d = e.ai_aux[i];
            draw_line(pos.x, pos.y, pos.x + d.x * 80.0, pos.y + d.y * 80.0, 3.0, Color::new(1.0, 0.3, 0.3, 0.35));
            warn = true;
        }
        Ai::Blink if e.ai_state[i] != 0 => tint.a = blink_alpha(e, i),
        _ => {}
    }
    if warn && ((tf * 16.0) as i32) % 2 == 0 && e.flash[i] == 0 {
        tint = Color::new(1.0, 0.75, 0.75, 1.0);
    }
    if rank >= Rank::Mini {
        let pulse = 0.16 + 0.06 * (tf * 4.0).sin();
        draw_circle(pos.x, pos.y - h * 0.3 + bob, w * 0.55, Color::new(accent.r, accent.g, accent.b, pulse * 0.6));
        let ring = crate::ui::element_color(kind.element());
        draw_circle_lines(pos.x, pos.y + 2.0, e.radius[i] + 4.0, 2.0, Color::new(ring.r, ring.g, ring.b, 0.6));
    } else if matches!(ai, Ai::Ranged { .. }) {
        draw_circle(pos.x, pos.y - h * 0.3, w * 0.55, Color::new(accent.r, accent.g, accent.b, 0.10));
    }
    let (sw, sh) = (w * 0.7, (w * 0.2).max(5.0));
    shadows.push(shadow, pos.x - sw / 2.0, pos.y + h * 0.25 - 1.0 - sh / 2.0, Color::new(0.0, 0.0, 0.0, 0.4), DrawTextureParams { dest_size: Some(vec2(sw, sh)), ..Default::default() });
    if !world.in_reach(pos, e.radius[i]) {
        tint = dimmed(tint);
    }
    let (x, y) = ((pos.x - w / 2.0).round(), (pos.y - h * 0.75 + bob).round());
    let rarity = kind.rarity();
    if let (Some(outline), Some(c)) = (outline, outline_tint(rarity, tint.a)) {
        for (dx, dy) in OUTLINE_OFFSETS {
            outlines.push(outline, x + dx, y + dy, c, DrawTextureParams { flip_x: flip, dest_size: Some(vec2(w, h)), ..Default::default() });
        }
        cues.push((pos.x, y, rarity, tint.a));
    }
    if rarity == EnemyRarity::Leader {
        let c = leader_color();
        let pulse = 0.22 + 0.08 * (tf * 3.0).sin();
        draw_circle(pos.x, pos.y, sim::content::BANNER_RADIUS, Color::new(c.r, c.g, c.b, 0.05));
        draw_circle_lines(pos.x, pos.y, sim::content::BANNER_RADIUS, 2.0, Color::new(c.r, c.g, c.b, pulse));
    }
    sprites.push(t, x, y, tint, DrawTextureParams { flip_x: flip, dest_size: Some(vec2(w, h)), ..Default::default() });
}

/// A Blink enemy fading out before a blink (state 1) or back in after it (state 2).
fn blink_alpha(e: &sim::world::Enemies, i: usize) -> f32 {
    let f = match e.ai_state[i] {
        0 => return 1.0,
        1 => e.ai_timer[i] / 0.5,
        _ => 1.0 - e.ai_timer[i] / sim::content::NOPPERABO_FADE_IN,
    };
    f.clamp(0.1, 1.0) * 0.85
}

const OUTLINE_OFFSETS: [(f32, f32); 8] = [(-1.5, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5), (-1.1, -1.1), (1.1, -1.1), (-1.1, 1.1), (1.1, 1.1)];

fn outline_tint(rarity: EnemyRarity, alpha: f32) -> Option<Color> {
    crate::ui::enemy_rarity_color(rarity).map(|c| Color::new(c.r, c.g, c.b, alpha))
}

/// Outline for an enemy drawn straight to the screen (not batched), under its sprite.
fn draw_outline_now(outline: &Texture2D, x: f32, y: f32, size: Vec2, flip: bool, rarity: EnemyRarity) {
    let Some(c) = outline_tint(rarity, 1.0) else { return };
    for (dx, dy) in OUTLINE_OFFSETS {
        draw_texture_ex(outline, x + dx, y + dy, c, DrawTextureParams { flip_x: flip, dest_size: Some(size), ..Default::default() });
    }
}

/// Shape cue above the sprite, so rarity reads without the colour: chevrons for Uncommon and
/// Elite, a pennant on a pole for a Leader.
fn draw_rarity_cue(x: f32, top: f32, rarity: EnemyRarity, alpha: f32) {
    let Some(c) = crate::ui::enemy_rarity_color(rarity) else { return };
    let c = Color::new(c.r, c.g, c.b, c.a * alpha);
    let dark = Color::new(0.0, 0.0, 0.0, 0.6 * alpha);
    let chevrons = match rarity {
        EnemyRarity::Uncommon => 1,
        EnemyRarity::Elite => 2,
        _ => 0,
    };
    for k in 0..chevrons {
        let y = top - 3.0 - k as f32 * 5.0;
        draw_triangle(vec2(x - 5.0, y + 1.0), vec2(x + 5.0, y + 1.0), vec2(x, y - 5.0), dark);
        draw_triangle(vec2(x - 4.0, y), vec2(x + 4.0, y), vec2(x, y - 4.0), c);
    }
    if rarity == EnemyRarity::Leader {
        draw_line(x, top + 2.0, x, top - 18.0, 2.0, dark);
        draw_line(x, top + 2.0, x, top - 17.0, 1.0, c);
        draw_triangle(vec2(x, top - 18.0), vec2(x + 11.0, top - 14.0), vec2(x, top - 10.0), c);
    }
}

/// World hazards: tag/blast and lightning telegraphs, spirit pressure, hostile domains.
fn draw_hazards(world: &World, t: f32, friendly: bool) {
    use sim::world::HazardKind;
    for h in world.hazards.iter().filter(|h| h.friendly == friendly) {
        let (x, y, r) = (h.pos.x, h.pos.y, h.radius);
        match h.kind {
            HazardKind::Blast { .. } if h.spare_enemies => {
                // A Clay Bird's shell: a clay ring that fills from empty, with a fuse spark on top.
                let k = (1.0 - h.warn / sim::content::CLAY_BLAST_WARN).clamp(0.0, 1.0);
                draw_circle(x, y, r * k, Color::new(0.8, 0.5, 0.28, 0.24));
                for seg in (0..24).step_by(2) {
                    let a0 = seg as f32 / 24.0 * std::f32::consts::TAU;
                    let a1 = a0 + std::f32::consts::TAU / 24.0;
                    draw_line(x + a0.cos() * r, y + a0.sin() * r, x + a1.cos() * r, y + a1.sin() * r, 2.0, Color::new(0.85, 0.55, 0.3, 0.6 + 0.35 * k));
                }
                draw_circle(x, y, 4.0, Color::new(0.55, 0.32, 0.18, 0.95));
                let spark = 0.6 + 0.4 * (t * 30.0).sin().abs();
                draw_circle(x + 1.0, y - 5.0, 1.5 + k, Color::new(1.0, 0.85, 0.3, spark));
            }
            HazardKind::Blast { .. } => {
                // A paper tag / falling strike: a ring that fills as it is about to go off.
                let k = (1.0 - h.warn / 1.3).clamp(0.0, 1.0);
                draw_circle(x, y, r * k, Color::new(1.0, 0.35, 0.15, 0.22));
                draw_circle_lines(x, y, r, 2.0, Color::new(1.0, 0.45, 0.2, 0.55 + 0.4 * (t * 14.0).sin().abs()));
                draw_rectangle(x - 3.0, y - 5.0, 6.0, 10.0, Color::new(0.95, 0.9, 0.75, 0.9));
                draw_rectangle(x - 1.0, y - 3.0, 2.0, 6.0, Color::new(0.85, 0.15, 0.15, 1.0));
            }
            HazardKind::Lightning { .. } => {
                let k = (1.0 - h.warn / 1.2).clamp(0.0, 1.0);
                draw_circle(x, y, r * k, Color::new(0.7, 0.85, 1.0, 0.18));
                draw_circle_lines(x, y, r, 2.0, Color::new(0.75, 0.9, 1.0, 0.5 + 0.5 * (t * 18.0).sin().abs()));
                draw_line(x - r * 0.4, y, x + r * 0.4, y, 1.5, Color::new(0.85, 0.95, 1.0, 0.6));
                draw_line(x, y - r * 0.4, x, y + r * 0.4, 1.5, Color::new(0.85, 0.95, 1.0, 0.6));
            }
            HazardKind::Pressure => {
                let live = h.warn <= 0.0;
                let a = if live { (h.life / 2.0).min(1.0) } else { 0.4 * (1.0 - h.warn) };
                draw_circle(x, y, r, Color::new(0.45, 0.6, 1.0, 0.14 * a));
                for k in 0..3 {
                    let rr = r * (((t * 0.6 + k as f32 / 3.0) % 1.0) * 0.8 + 0.2);
                    draw_circle_lines(x, y, rr, 1.5, Color::new(0.65, 0.8, 1.0, 0.35 * a * (1.0 - rr / r)));
                }
                draw_circle_lines(x, y, r, 2.0, Color::new(0.7, 0.85, 1.0, 0.45 * a));
            }
            HazardKind::Domain { .. } => {
                if h.warn > 0.0 {
                    // The domain is forming: a flickering boundary.
                    let blink = ((t * 8.0) as i32 % 2 == 0) as i32 as f32;
                    let segs = 48;
                    for k in (0..segs).step_by(2) {
                        let a0 = k as f32 / segs as f32 * std::f32::consts::TAU + t * 0.5;
                        let a1 = a0 + std::f32::consts::TAU / segs as f32;
                        draw_line(x + a0.cos() * r, y + a0.sin() * r, x + a1.cos() * r, y + a1.sin() * r, 3.0, Color::new(0.75, 0.3, 1.0, 0.4 + 0.5 * blink));
                    }
                } else {
                    let a = (h.life / 1.0).min(1.0);
                    draw_circle(x, y, r, Color::new(0.18, 0.02, 0.28, 0.32 * a));
                    draw_circle_lines(x, y, r, 4.0, Color::new(0.75, 0.3, 1.0, 0.8 * a));
                    for k in 0..12 {
                        let ang = k as f32 / 12.0 * std::f32::consts::TAU - t * 0.3;
                        let (cx, cy) = (x + ang.cos() * (r - 12.0), y + ang.sin() * (r - 12.0));
                        draw_poly(cx, cy, 3, 5.0, ang.to_degrees(), Color::new(0.9, 0.5, 1.0, 0.7 * a));
                    }
                }
            }
        }
    }
}

/// The Hollow Mask aura, a player-form flourish.
fn draw_mask(world: &World, t: f32) {
    let p = world.player.pos;
    if world.mask > 0.0 {
        let a = 0.25 + 0.1 * (t * 10.0).sin();
        draw_circle(p.x, p.y - 8.0, 30.0, Color::new(0.9, 0.1, 0.15, a));
        draw_circle_lines(p.x, p.y - 8.0, 34.0, 2.0, Color::new(1.0, 0.9, 0.9, 0.6));
    }
}

fn draw_art(world: &World, t: f32) {
    use sim::gear::ArsenalArt;
    let Some(a) = &world.art else { return };
    match a.art {
        ArsenalArt::RaijinDrums => {
            for d in world.art_points() {
                draw_circle(d.x, d.y, 16.0, Color::new(1.0, 0.85, 0.3, 0.18 + 0.08 * (t * 9.0).sin()));
                draw_circle(d.x, d.y, 9.0, Color::new(0.55, 0.22, 0.12, 1.0));
                draw_circle(d.x, d.y, 6.5, Color::new(0.95, 0.85, 0.6, 1.0));
                draw_poly_lines(d.x, d.y, 3, 4.0, t * 120.0, 1.5, Color::new(0.85, 0.2, 0.1, 1.0));
                draw_circle_lines(d.x, d.y, 9.0, 1.5, Color::new(1.0, 0.9, 0.4, 1.0));
            }
        }
        ArsenalArt::DragonCoil => {
            let pts = world.art_points();
            for (k, c) in pts.iter().enumerate().rev() {
                let r = if k == 0 { 13.0 } else { 10.0 - k as f32 * 0.25 };
                draw_circle(c.x, c.y, r + 3.0, Color::new(0.2, 0.9, 0.6, 0.15));
                draw_circle(c.x, c.y, r, if k % 2 == 0 { Color::new(0.15, 0.62, 0.42, 1.0) } else { Color::new(0.2, 0.75, 0.5, 1.0) });
                draw_circle(c.x, c.y - r * 0.3, r * 0.35, Color::new(1.0, 0.85, 0.35, 0.9));
            }
            if let Some(h) = pts.first() {
                draw_circle(h.x + 4.0, h.y - 3.0, 2.5, Color::new(1.0, 0.3, 0.2, 1.0));
            }
        }
        ArsenalArt::FoxProcession => {
            for (fp, fv, life) in &a.foxes {
                let alpha = (life / 0.3).min(1.0);
                let d = fv.normalize_or_zero();
                for k in 1..5 {
                    let q = *fp - d * (k as f32 * 7.0);
                    draw_circle(q.x, q.y, 7.0 - k as f32, Color::new(0.4, 0.75, 1.0, 0.35 * alpha));
                }
                draw_circle(fp.x, fp.y, 8.0, Color::new(0.95, 0.97, 1.0, alpha));
                draw_triangle(vec2(fp.x - 5.0, fp.y - 4.0), vec2(fp.x - 2.0, fp.y - 13.0), vec2(fp.x + 1.0, fp.y - 5.0), Color::new(0.95, 0.97, 1.0, alpha));
                draw_triangle(vec2(fp.x + 5.0, fp.y - 4.0), vec2(fp.x + 2.0, fp.y - 13.0), vec2(fp.x - 1.0, fp.y - 5.0), Color::new(0.95, 0.97, 1.0, alpha));
                draw_circle(fp.x + 3.0 * d.x, fp.y - 1.0, 1.5, Color::new(0.9, 0.2, 0.3, alpha));
            }
        }
        ArsenalArt::YinYangBreaker => {
            for (op, _) in &a.orbs {
                draw_circle(op.x, op.y, 32.0, Color::new(1.0, 0.95, 0.8, 0.14));
                draw_yinyang(vec2(op.x, op.y), 22.0, t * 4.0);
                draw_circle_lines(op.x, op.y, 23.0, 2.0, Color::new(1.0, 0.82, 0.42, 1.0));
            }
        }
        _ => {}
    }
}

/// World units the camera moves at full trauma (1.0).
const SHAKE_MAX: f32 = 6.0;

/// Trauma-squared amplitude with a smooth sum-of-sines wobble, so small shakes stay small.
fn shake_offset(trauma: f32, t: f32) -> Vec2 {
    let amp = SHAKE_MAX * trauma * trauma;
    let x = (t * 47.0).sin() + 0.5 * (t * 29.0 + 1.3).sin();
    let y = (t * 41.0 + 0.7).sin() + 0.5 * (t * 23.0).sin();
    vec2(x, y) * (amp / 1.5)
}

/// The world area the camera shows (no shake), for skipping what's off-screen.
fn view_rect(cam: &Camera2D) -> Rect {
    let (sw, sh) = (screen_width() / world_scale(), screen_height() / world_scale());
    Rect::new(cam.target.x - sw / 2.0, cam.target.y - sh / 2.0, sw, sh)
}

/// `(x, y)` lies within `pad` world units of the view.
fn near(view: &Rect, x: f32, y: f32, pad: f32) -> bool {
    x > view.x - pad && x < view.x + view.w + pad && y > view.y - pad && y < view.y + view.h + pad
}

pub fn camera_for(world: &World, fx: &Fx) -> Camera2D {
    let (sw, sh) = (screen_width() / world_scale(), screen_height() / world_scale());
    let p = world.player.pos;
    let jitter = shake_offset(fx.shake, get_time() as f32);
    // Positive y zoom: macroquad already flips y when drawing to the screen, so this gives
    // y-down world coordinates (from_display_rect would flip twice and render upside down).
    Camera2D { target: vec2(p.x.round(), p.y.round()) + jitter, zoom: vec2(2.0 / sw, 2.0 / sh), ..Default::default() }
}

fn draw_smokes(smokes: &[(Vec2, f32, f32)]) {
    for (c, r, age) in smokes {
        let k = age / 0.5;
        for i in 0..7 {
            let a = i as f32 * 0.9 + c.x * 0.05;
            let d = r * 0.5 * k.sqrt();
            let pr = r * (0.35 + 0.35 * k) * (0.8 + 0.2 * ((i * 37 % 10) as f32 / 10.0));
            draw_circle(c.x + a.cos() * d, c.y - 6.0 + a.sin() * d * 0.7 - k * 6.0, pr, Color::new(0.85, 0.88, 0.95, 0.45 * (1.0 - k)));
        }
    }
}

fn draw_bursts(bursts: &[(Vec2, f32, f32, u8)]) {
    for (c, r, age, style) in bursts {
        let k = age / burst_life(*style);
        let rr = r * (0.3 + 0.7 * k.sqrt());
        let fade = 1.0 - k;
        let (fill, edge, width) = match *style {
            1 => (Color::new(1.0, 0.7, 0.2, 0.35 * fade), Color::new(1.0, 0.9, 0.5, 0.9 * fade), 5.0),
            style::SEAL => (Color::new(1.0, 0.85, 0.3, 0.3 * fade), Color::new(1.0, 0.95, 0.6, fade), 2.0),
            style::METEOR => (Color::new(1.0, 0.45, 0.1, 0.5 * fade), Color::new(1.0, 0.85, 0.4, fade), 7.0),
            style::FOX => (Color::new(1.0, 0.55, 0.15, 0.35 * fade), Color::new(1.0, 0.85, 0.4, fade), 2.5),
            style::SKYBOLT => (Color::new(0.7, 0.85, 1.0, 0.4 * fade), Color::new(1.0, 1.0, 1.0, fade), 3.0),
            style::COMET => (Color::new(0.3, 0.6, 1.0, 0.45 * fade), Color::new(0.85, 0.95, 1.0, fade), 4.0),
            style::QUAKE => (Color::new(1.0, 0.6, 0.2, 0.25 * fade), Color::new(1.0, 0.85, 0.4, fade), 6.0),
            style::SPINE => (Color::new(0.4, 0.02, 0.06, 0.35 * fade), Color::new(0.9, 0.2, 0.25, fade), 2.0),
            16 => (Color::new(0.2, 0.35, 0.9, 0.35 * fade), Color::new(0.75, 0.88, 1.0, fade), 3.0),
            21 => (Color::new(1.0, 0.6, 0.75, 0.3 * fade), Color::new(1.0, 0.9, 0.95, fade), 2.0),
            style::YINYANG => (Color::new(1.0, 1.0, 1.0, 0.3 * fade), Color::new(1.0, 0.4, 0.45, fade), 4.0),
            style::PURIFY => (Color::new(1.0, 0.92, 0.65, 0.4 * fade), Color::new(1.0, 0.35, 0.4, fade), 9.0),
            17 => (Color::new(0.45, 0.8, 1.0, 0.3 * fade), Color::new(0.9, 0.97, 1.0, fade), 2.5),
            18 => (Color::new(0.55, 0.25, 1.0, 0.4 * fade), Color::new(0.9, 0.75, 1.0, fade), 7.0),
            LEADER_FELL => {
                let c = leader_color();
                (Color::new(c.r, c.g, c.b, 0.18 * fade), Color::new(c.r, c.g, c.b, fade), 6.0)
            }
            style::VENOM => (Color::new(0.3, 0.75, 0.2, 0.35 * fade), Color::new(0.6, 1.0, 0.4, fade), 2.0),
            style::FRENZY => (Color::new(0.5, 0.0, 0.08, 0.4 * fade), Color::new(1.0, 0.25, 0.3, fade), 6.0),
            2 => (Color::new(0.6, 0.0, 0.1, 0.4 * fade), Color::new(1.0, 0.3, 0.3, 0.9 * fade), 3.0),
            3 => (Color::new(0.25, 0.0, 0.05, 0.55 * fade), Color::new(1.0, 0.15, 0.2, fade), 8.0),
            _ => (Color::new(0.3, 0.65, 1.0, 0.35 * fade), Color::new(0.8, 0.95, 1.0, 0.9 * fade), 2.0),
        };
        draw_circle(c.x, c.y, rr, fill);
        match *style {
            style::SEAL => {
                // A spinning seal glyph: square inside the ring.
                let a = k * 3.0;
                let q = rr * 0.6;
                for i in 0..4 {
                    let (a0, a1) = (a + i as f32 * std::f32::consts::FRAC_PI_2, a + (i + 1) as f32 * std::f32::consts::FRAC_PI_2);
                    draw_line(c.x + a0.cos() * q, c.y + a0.sin() * q, c.x + a1.cos() * q, c.y + a1.sin() * q, 2.0, Color::new(1.0, 0.3, 0.25, fade));
                }
            }
            style::METEOR => {
                draw_circle(c.x, c.y, rr * 0.55, Color::new(1.0, 0.95, 0.7, 0.7 * fade));
                for i in 0..12 {
                    let a = i as f32 * 0.52 + c.x;
                    let d = rr * (0.4 + 0.8 * k);
                    draw_rectangle(c.x + a.cos() * d, c.y + a.sin() * d * 0.7, 3.0, 3.0, Color::new(0.4, 0.2, 0.1, fade));
                }
            }
            style::SKYBOLT => {
                // The bolt column from the top of the sky.
                let mut prev = vec2(c.x, c.y - 300.0);
                for s in 1..=8 {
                    let y = c.y - 300.0 + 300.0 * s as f32 / 8.0;
                    let x = if s < 8 { c.x + ((s as f32 * 5.3 + c.y).sin()) * 10.0 } else { c.x };
                    draw_line(prev.x, prev.y, x, y, 6.0 * fade, Color::new(0.6, 0.8, 1.0, 0.7 * fade));
                    draw_line(prev.x, prev.y, x, y, 2.5 * fade, Color::new(1.0, 1.0, 1.0, fade));
                    prev = vec2(x, y);
                }
            }
            style::QUAKE => {
                draw_circle_lines(c.x, c.y, rr * 0.6, 3.0, Color::new(1.0, 0.4, 0.3, 0.8 * fade));
            }
            style::SPINE => draw_spine_burst(vec2(c.x, c.y), *r, k),
            style::FRENZY => {
                // Thousand-Leg Burst: a crown of spikes radiating out.
                for i in 0..28 {
                    let a = i as f32 / 28.0 * std::f32::consts::TAU;
                    let r0 = rr * 0.35;
                    let r1 = rr * (0.85 + 0.15 * ((i * 7 % 5) as f32 / 5.0));
                    let base = vec2(c.x, c.y);
                    let l = base + vec2((a - 0.05).cos(), (a - 0.05).sin()) * r0;
                    let rgt = base + vec2((a + 0.05).cos(), (a + 0.05).sin()) * r0;
                    let tip = base + vec2(a.cos(), a.sin()) * r1;
                    draw_triangle(l, rgt, tip, Color::new(0.75, 0.08, 0.15, fade));
                }
            }
            _ => {}
        }
        if *style == style::PURIFY {
            draw_circle_lines(c.x, c.y, rr * 0.8, 5.0, Color::new(1.0, 0.95, 0.75, fade));
            draw_circle_lines(c.x, c.y, rr * 0.55, 3.0, Color::new(1.0, 0.5, 0.6, fade));
        }
        if *style == 3 {
            draw_circle(c.x, c.y, rr * 0.6, Color::new(0.05, 0.0, 0.02, 0.5 * fade));
            draw_circle_lines(c.x, c.y, rr * 0.8, 4.0, Color::new(0.9, 0.05, 0.1, 0.7 * fade));
        }
        draw_circle_lines(c.x, c.y, rr, width, edge);
    }
}

pub fn draw_world(world: &World, a: &Assets, stage: &crate::stage::StageArt, fx: &Fx, cam: &Camera2D, time: f64, target: Option<&RenderTarget>) {
    set_camera(cam);
    if target.is_some() {
        clear_background(Color::from_rgba(28, 24, 40, 255));
    }
    stage.draw_ground(world.cfg.stage, cam, time);
    let p = world.player.pos;
    let view = view_rect(cam);

    // The hero, when its art is too detailed for the world target (drawn at the end of this fn).
    let mut hires_player: Option<(Texture2D, f32, f32, Color, DrawTextureParams)> = None;

    // Sakura Field sits on the ground under everything.
    for w in &world.weapons {
        if w.kind == WeaponKind::SakuraField {
            let r = world.field_radius_of(w);
            let pulse = (1.0 - (w.phase / 0.25).min(1.0)) * 0.12;
            let base = if w.awakened { Color::new(1.0, 0.75, 0.55, 0.14 + pulse) } else { Color::new(1.0, 0.55, 0.75, 0.10 + pulse) };
            draw_circle(p.x, p.y, r, base);
            draw_circle_lines(p.x, p.y, r, 1.0, Color::new(1.0, 0.7, 0.85, 0.55));
            for k in 0..10 {
                let ang = time as f32 * 0.8 + k as f32 * 0.628;
                let rr = r * (0.55 + 0.4 * ((k * 37 % 10) as f32 / 10.0));
                draw_rectangle(p.x + ang.cos() * rr, p.y + ang.sin() * rr, 2.0, 2.0, Color::from_rgba(255, 190, 215, 220));
            }
        }
    }

    for pl in &world.pools {
        let life = pl.life / pl.max_life;
        let flicker = 0.85 + 0.15 * ((time * 13.0) as f32 + pl.pos.x).sin();
        let fade = (life * 3.0).min(1.0) * flicker;
        draw_circle(pl.pos.x, pl.pos.y, pl.radius, Color::new(1.0, 0.45, 0.1, 0.45 * fade));
        draw_circle(pl.pos.x, pl.pos.y, pl.radius * 0.6, Color::new(1.0, 0.8, 0.3, 0.45 * fade));
        draw_circle_lines(pl.pos.x, pl.pos.y, pl.radius, 1.5, Color::new(1.0, 0.6, 0.2, 0.8 * fade));
        for k in 0..5 {
            let a = k as f32 * 1.256 + pl.pos.y * 0.1;
            let rise = ((time as f32 * 2.0 + k as f32 * 0.3) % 1.0) * 10.0;
            draw_rectangle(pl.pos.x + a.cos() * pl.radius * 0.6, pl.pos.y + a.sin() * pl.radius * 0.5 - rise, 2.0, 2.0, Color::new(1.0, 0.85, 0.3, 0.9));
        }
    }
    for st in &world.strikes {
        // Landing marker shrinks as the star approaches.
        let t = (st.delay / 0.6).clamp(0.0, 1.0);
        draw_circle_lines(st.pos.x, st.pos.y, st.radius * (0.4 + 0.6 * t), 1.5, Color::new(1.0, 0.95, 0.5, 0.7));
    }
    if let Some(s) = world.shrine {
        let pulse = 0.5 + 0.5 * (time * 3.0).sin() as f32;
        draw_rectangle(s.x - 5.0, s.y - 120.0, 10.0, 120.0, Color::new(1.0, 0.9, 0.6, 0.10 + 0.06 * pulse));
        draw_circle(s.x, s.y, 22.0 + 3.0 * pulse, Color::new(1.0, 0.85, 0.5, 0.16 + 0.08 * pulse));
        draw_texture_ex(&a.shrine, (s.x - 16.0).round(), (s.y - 30.0).round(), WHITE, DrawTextureParams { dest_size: Some(vec2(32.0, 36.0)), ..Default::default() });
    }
    for wl in &world.wells {
        draw_well(vec2(wl.pos.x, wl.pos.y), wl.radius, wl.life / wl.max_life, time as f32);
    }
    // Endless Firmament: the battlefield darkens and becomes a starfield inside the domain.
    if world.form_kind() == Some(SpecialKind::Firmament) {
        draw_domain(world, time as f32);
    }
    for l in &world.lanterns {
        let fade = (l.life / 0.4).min(1.0) * ((l.max_life - l.life) / 0.3).min(1.0);
        let bob = ((time * 3.0 + l.pos.x as f64).sin() * 2.0) as f32;
        draw_circle(l.pos.x, l.pos.y, l.radius, Color::new(0.5, 0.75, 1.0, 0.06 * fade));
        draw_circle_lines(l.pos.x, l.pos.y, l.radius, 1.0, Color::new(0.7, 0.85, 1.0, 0.35 * fade));
        let (x, y) = (l.pos.x, l.pos.y - 12.0 + bob);
        draw_circle(x, y, 10.0, Color::new(0.55, 0.8, 1.0, 0.18 * fade));
        draw_rectangle(x - 2.0, y - 8.0, 4.0, 2.0, Color::new(0.16, 0.14, 0.2, fade));
        draw_ellipse(x, y, 5.0, 6.0, 0.0, Color::new(0.95, 0.92, 0.86, fade));
        draw_circle(x, y, 2.5, Color::new(0.55, 0.82, 1.0, fade));
        draw_rectangle(x - 2.0, y + 6.0, 4.0, 2.0, Color::new(0.16, 0.14, 0.2, fade));
    }

    let e = &world.enemies;
    let mut sprites = Sprites::new();
    let mut shadows = Sprites::new();
    let mut outlines = Sprites::new();
    let mut cues = Vec::new();
    let flash = Color::new(1.0, 0.45, 0.55, 1.0);
    for i in 0..e.len() {
        let pos = e.pos[i];
        // Bosses and elites always draw; chargers' dash warnings reach 230 units ahead.
        let pad = if e.kind[i].rank() >= Rank::Elite { f32::INFINITY } else if matches!(sim::content::enemy_def(e.kind[i]).ai, sim::content::Ai::Charger { .. }) { 240.0 } else { 48.0 };
        if !near(&view, pos.x, pos.y, pad) {
            continue;
        }
        let in_domain = world.form_kind() == Some(SpecialKind::Firmament) && (e.pos[i] - world.domain_center).length() < world.domain_radius();
        let frozen = (world.freeze > 0.0 || in_domain) && !e.kind[i].is_boss();
        let scattering = e.scatter[i] > sim::content::SCATTER_TIME - SCATTER_CUE && ((time * 12.0) as i32) % 2 == 0;
        let tint = if e.flash[i] > 0 {
            flash
        } else if frozen {
            Color::new(0.6, 0.85, 1.0, 1.0)
        } else if scattering {
            Color::new(0.85, 0.7, 1.0, 1.0)
        } else {
            WHITE
        };
        if e.banner[i] {
            let c = leader_color();
            draw_circle_lines(pos.x, pos.y + 2.0, e.radius[i] + 3.0, 1.0, Color::new(c.r, c.g, c.b, 0.45));
        }
        // Champions: a pulsing crimson-gold aura and a warm tint.
        let tint = if e.champion[i] {
            let pulse = 0.5 + 0.5 * (time as f32 * 6.0 + i as f32).sin();
            draw_circle(pos.x, pos.y - 4.0, e.radius[i] + 6.0, Color::new(1.0, 0.35, 0.2, 0.12 + 0.08 * pulse));
            draw_circle_lines(pos.x, pos.y - 4.0, e.radius[i] + 4.0, 1.5, Color::new(1.0, 0.75, 0.3, 0.55 + 0.3 * pulse));
            if e.flash[i] > 0 { tint } else { Color::new(1.0, 0.78, 0.6, 1.0) }
        } else {
            tint
        };
        let flip = e.kind[i] != EnemyKind::Wisp && pos.x > p.x;
        let out_of_reach = !world.in_reach(pos, e.radius[i]);
        if e.kind[i] == EnemyKind::GreatOni {
            // Double-size oni; flashes and shows its line while winding up a charge.
            let winding = e.ai_state[i] == 1;
            if winding {
                let d = e.ai_dir[i];
                draw_line(pos.x, pos.y, pos.x + d.x * 230.0, pos.y + d.y * 230.0, 10.0, Color::new(1.0, 0.2, 0.2, 0.22));
            }
            let tint = if winding && ((time * 16.0) as i32) % 2 == 0 {
                Color::new(1.0, 1.0, 1.0, 1.0)
            } else if e.flash[i] > 0 {
                flash
            } else {
                Color::new(0.85, 0.7, 1.0, 1.0)
            };
            let tint = if out_of_reach { dimmed(tint) } else { tint };
            draw_ellipse(pos.x, pos.y + 6.0, 24.0, 7.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.35));
            draw_outline_now(&a.oni_outline, (pos.x - 32.0).round(), (pos.y - 52.0).round(), vec2(64.0, 64.0), flip, e.kind[i].rarity());
            cues.push((pos.x, (pos.y - 52.0).round(), e.kind[i].rarity(), tint.a));
            draw_texture_ex(
                &a.oni,
                (pos.x - 32.0).round(),
                (pos.y - 52.0).round(),
                tint,
                DrawTextureParams { flip_x: flip, dest_size: Some(vec2(64.0, 64.0)), ..Default::default() },
            );
            continue;
        }
        if e.kind[i] == EnemyKind::NightLord {
            // Big, bobbing lantern with a warm glow; drawn at 2x (96px).
            let bob = ((time * 2.0).sin() * 3.0) as f32;
            let glow = 0.18 + 0.06 * (time * 5.0).sin() as f32;
            let tint = if out_of_reach { dimmed(tint) } else { tint };
            draw_circle(pos.x, pos.y - 24.0, 60.0, Color::new(1.0, 0.55, 0.2, glow));
            draw_ellipse(pos.x, pos.y + 8.0, 34.0, 9.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.4));
            draw_outline_now(&a.night_lord_outline, (pos.x - 48.0).round(), (pos.y - 84.0 + bob).round(), vec2(96.0, 96.0), false, e.kind[i].rarity());
            cues.push((pos.x, (pos.y - 84.0 + bob).round(), e.kind[i].rarity(), tint.a));
            draw_texture_ex(
                &a.night_lord,
                (pos.x - 48.0).round(),
                (pos.y - 84.0 + bob).round(),
                tint,
                DrawTextureParams { dest_size: Some(vec2(96.0, 96.0)), ..Default::default() },
            );
            continue;
        }
        let kind = e.kind[i];
        if let Some(t) = &a.world_enemies[kind.index()] {
            draw_world_enemy(world, i, t, a.enemy_outlines[kind.index()].as_ref(), tint, flip, fx.accent, time, &mut sprites, &mut shadows, &mut outlines, &mut cues, &a.shadow);
            continue;
        }
        let (tex, half, bottom) = match e.kind[i] {
            EnemyKind::Wisp => (&a.wisp, 8.0, 8.0),
            EnemyKind::Kasa => (&a.kasa, 8.0, 12.0),
            EnemyKind::Chochin => (&a.chochin, 8.0, 14.0),
            EnemyKind::Nopperabo => (&a.nopperabo, 8.0, 20.0),
            _ => (&a.oni, 16.0, 26.0),
        };
        let mut tint = tint;
        if e.kind[i] == EnemyKind::Chochin {
            draw_circle(pos.x, pos.y - 4.0, 11.0, Color::new(1.0, 0.6, 0.25, 0.14));
        } else if e.kind[i] == EnemyKind::Nopperabo && e.ai_state[i] != 0 {
            tint.a = blink_alpha(e, i);
        }
        if out_of_reach {
            tint = dimmed(tint);
        }
        let (x, y) = ((pos.x - half).round(), (pos.y - bottom).round());
        let rarity = e.kind[i].rarity();
        if let (EnemyKind::Oni, Some(c)) = (e.kind[i], outline_tint(rarity, tint.a)) {
            for (dx, dy) in OUTLINE_OFFSETS {
                outlines.push(&a.oni_outline, x + dx, y + dy, c, DrawTextureParams { dest_size: Some(crate::sprites::nat(tex)), flip_x: flip, ..Default::default() });
            }
            cues.push((pos.x, y, rarity, tint.a));
        }
        sprites.push(tex, x, y, tint, DrawTextureParams { dest_size: Some(crate::sprites::nat(tex)), flip_x: flip, ..Default::default() });
    }
    shadows.flush();
    outlines.flush();
    sprites.flush();
    for (x, top, rarity, alpha) in cues {
        draw_rarity_cue(x, top, rarity, alpha);
    }
    for i in 0..e.len() {
        if let Some(k) = e.carry[i] {
            let pos = e.pos[i];
            if near(&view, pos.x, pos.y, 48.0) {
                let bob = (time as f32 * 6.0 + i as f32).sin() * 2.0;
                let tint = Color::new(1.0, 1.0, 1.0, blink_alpha(e, i));
                draw_texture_ex(a.pickup(k), (pos.x - 6.0).round(), (pos.y - 40.0 + bob).round(), tint, DrawTextureParams { dest_size: Some(vec2(12.0, 12.0)), ..Default::default() });
            }
        }
    }

    // Player: walk cycle while moving, blink during i-frames.
    let pl = &world.player;
    let frame = if pl.moving { 1 + ((time * 8.0) as usize % 2) } else { 0 };
    let form = world.form_kind();
    let fox = form == Some(SpecialKind::NineTails);
    let hannya = form == Some(SpecialKind::Hannya);
    let blink = form.is_none() && pl.iframes > 0.0 && ((time * 20.0) as i32) % 2 == 0;
    let tint = if blink { Color::new(1.0, 1.0, 1.0, 0.35) } else { WHITE };
    let t = time as f32;
    if fox {
        draw_fox_aura_and_tails(world, t);
    }
    if hannya {
        draw_hannya_aura(world, t);
    }
    let mukade = form == Some(SpecialKind::Mukade);
    let domain = form == Some(SpecialKind::Firmament);
    let divine = form == Some(SpecialKind::Divine);
    if divine {
        draw_divine(world, t);
    }
    if world.cfg.hero == HeroId::Ren && !divine && world.barrier_cd <= 0.0 {
        // Barrier ready: a soft pink shimmer.
        draw_circle_lines(p.x, p.y - 8.0, 18.0 + 1.5 * (t * 4.0).sin(), 1.5, Color::new(1.0, 0.7, 0.85, 0.5));
    }
    if world.cfg.hero == HeroId::Kuon && !domain {
        draw_stillness(world, t);
    }
    if world.cfg.hero == HeroId::Toya {
        draw_tendrils_and_tail(world, t, mukade);
        // Rampage afterimages trailing behind.
        for (pos, left, age) in &fx.trail {
            let k = 1.0 - age / 0.35;
            draw_texture_ex(&a.toya_mukade[0], (pos.x - 12.0).round(), (pos.y - 26.0).round(), Color::new(0.9, 0.1, 0.15, 0.35 * k), DrawTextureParams { dest_size: Some(crate::sprites::nat(&a.toya_mukade[0])), flip_x: *left, ..Default::default() });
        }
    }
    // Shadow Step afterimages (explode shortly) and streaks.
    for (c, left) in &world.afterimages {
        let k = left / 0.35;
        draw_circle(c.x, c.y - 10.0, 18.0 * (1.5 - k), Color::new(0.6, 0.05, 0.1, 0.25 * (1.0 - k) + 0.1));
        draw_texture_ex(&a.shun_hannya[0], (c.x - 12.0).round(), (c.y - 26.0).round(), Color::new(0.9, 0.15, 0.2, 0.55 * k + 0.2), DrawTextureParams { dest_size: Some(crate::sprites::nat(&a.shun_hannya[0])), ..Default::default() });
    }
    for (from, to, age) in &fx.steps {
        let k = 1.0 - age / 0.3;
        draw_line(from.x, from.y - 10.0, to.x, to.y - 10.0, 6.0 * k, Color::new(0.5, 0.0, 0.08, 0.6 * k));
        draw_line(from.x, from.y - 10.0, to.x, to.y - 10.0, 2.0 * k, Color::new(1.0, 0.4, 0.4, 0.8 * k));
    }
    // Fox Mirage clones: translucent, blue-shifted copies of Riku (gold while he's transformed).
    for c in &world.clones {
        let f = if c.moving { 1 + ((time * 9.0 + c.pos.x as f64 * 0.1) as usize % 2) } else { 0 };
        let ctex = if fox { &a.riku_fox[f] } else { &a.hero(world.cfg.hero)[f] };
        let fade = (c.life / 0.4).min(1.0);
        let flicker = 0.7 + 0.15 * (t * 13.0 + c.pos.y).sin();
        let tint = Color::new(0.75, 0.85, 1.0, flicker * fade);
        let (cx, cy) = ((c.pos.x - 12.0).round(), (c.pos.y - 26.0).round());
        draw_ellipse(c.pos.x, c.pos.y + 5.0, 8.0, 2.5, 0.0, Color::new(0.0, 0.0, 0.0, 0.2 * fade));
        // Afterimage trailing behind when moving.
        if c.moving {
            let back = c.facing * -5.0;
            draw_texture_ex(ctex, cx + back.x, cy + back.y, Color::new(0.5, 0.65, 1.0, 0.25 * fade), DrawTextureParams { dest_size: Some(crate::sprites::nat(ctex)), flip_x: c.facing.x < 0.0, ..Default::default() });
        }
        draw_texture_ex(ctex, cx, cy, tint, DrawTextureParams { dest_size: Some(crate::sprites::nat(ctex)), flip_x: c.facing.x < 0.0, ..Default::default() });
        if c.striking > 0.0 {
            let base = c.facing.y.atan2(c.facing.x);
            let k = c.striking / 0.15;
            for s in 0..6 {
                let a0 = base - 0.9 + s as f32 * 0.3;
                let a1 = a0 + 0.3;
                let r = 16.0;
                draw_line(c.pos.x + a0.cos() * r, c.pos.y - 8.0 + a0.sin() * r, c.pos.x + a1.cos() * r, c.pos.y - 8.0 + a1.sin() * r, 3.0, Color::new(0.85, 0.95, 1.0, 0.9 * k));
            }
        }
    }
    let awakened = world.weapons.iter().filter(|w| w.awakened).count();
    if awakened > 0 {
        // A golden rune circle that grows richer with every Awakening.
        let r = 16.0 + 3.0 * awakened as f32;
        draw_circle_lines(p.x, p.y + 3.0, r, 1.5, Color::new(1.0, 0.85, 0.4, 0.55));
        for k in 0..(6 + 2 * awakened) {
            let a = t * 1.2 + k as f32 * std::f32::consts::TAU / (6 + 2 * awakened) as f32;
            draw_rectangle(p.x + a.cos() * r - 1.0, p.y + 3.0 + a.sin() * r * 0.45 - 1.0, 2.0, 2.0, Color::new(1.0, 0.9, 0.5, 0.9));
        }
    }
    for age in &fx.sig_rings {
        let k = age / 0.5;
        draw_circle_lines(p.x, p.y + 3.0, 20.0 + 60.0 * k, 3.0 * (1.0 - k) + 1.0, Color::new(1.0, 0.9, 0.5, 0.8 * (1.0 - k)));
    }
    draw_ellipse(p.x, p.y + 5.0, 9.0, 3.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.35));
    // Ren: generated animation set (walk / idle / throws), see Fx::animate.
    let animated = (world.cfg.hero == HeroId::Ren && !divine).then(|| fx.anim_frame(&a.ren_anim, time));
    let tex = if fox {
        &a.riku_fox[frame]
    } else if hannya {
        &a.shun_hannya[frame]
    } else if mukade {
        &a.toya_mukade[frame]
    } else if domain {
        &a.kuon_domain[frame]
    } else if divine {
        &a.ren_divine[frame]
    } else if let Some(tex) = animated {
        tex
    } else {
        &a.hero(world.cfg.hero)[frame]
    };
    // Centred on the player, feet 6 units below the position (24x32 sprites: -12, -26).
    let size = if animated.is_some() { a.ren_anim.size } else { crate::sprites::nat(tex) };
    let (x, y) = ((p.x - size.x / 2.0).round(), (p.y + 6.0 - size.y).round());
    let params = DrawTextureParams { dest_size: Some(size), flip_x: pl.facing.x < 0.0, ..Default::default() };
    if fox || hannya || mukade || domain || divine {
        // Flaring outline: the sprite tinted (gold foxfire / crimson-black dread / star-blue), offset around.
        let flare = 0.45 + 0.25 * (t * 9.0).sin();
        let glow = if fox || divine {
            Color::new(1.0, 0.75, 0.2, flare)
        } else if domain {
            Color::new(0.45, 0.8, 1.0, flare + 0.2)
        } else {
            Color::new(0.75, 0.0, 0.1, flare + 0.2)
        };
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
            draw_texture_ex(tex, x + dx, y + dy, glow, params.clone());
        }
    }
    let shun = world.cfg.hero == HeroId::Shun;
    let flip = pl.facing.x < 0.0;
    // Shun's blade sits behind him at rest; mid-swing it's drawn in front.
    let swinging = fx.swing.0 > 0.0;
    if shun && !swinging {
        draw_blade(a, p, flip, hannya, None);
    }
    if animated.is_some() && target.is_some() {
        // Hi-res art: drawn after the world target is on screen, at full device resolution.
        hires_player = Some((tex.clone(), x, y, tint, params));
    } else {
        draw_texture_ex(tex, x, y, tint, params);
    }
    if shun && swinging {
        draw_blade(a, p, flip, hannya, Some(fx.swing));
    }
    match world.cfg.hero {
        HeroId::Riku => draw_spirit_motes(p, t, fox),
        HeroId::Shun => draw_reaper_wisps(p, t, hannya),
        HeroId::Toya => draw_curse_motes(p, t, mukade),
        HeroId::Kuon => draw_star_motes(p, t, domain),
        _ => {}
    }
    // Small HP bar under the player.
    let frac = pl.hp / pl.max_hp;
    draw_rectangle(p.x - 10.0, p.y + 9.0, 20.0, 2.0, Color::from_rgba(40, 20, 30, 255));
    draw_rectangle(p.x - 10.0, p.y + 9.0, 20.0 * frac, 2.0, Color::from_rgba(255, 80, 110, 255));

    for w in &world.weapons {
        if w.kind == WeaponKind::YinYangOrbs {
            let r = world.yinyang_radius(w);
            for c in world.yinyang_positions(w) {
                draw_circle(c.x, c.y, r * 1.5, Color::new(1.0, 0.75, 0.85, 0.15));
                draw_yinyang(vec2(c.x, c.y), r, time as f32 * 4.0);
            }
        }
        if w.kind == WeaponKind::MoonOrbs {
            let r = world.orb_radius(w.level);
            for c in world.orb_positions(w) {
                draw_circle(c.x, c.y, r * 1.3, Color::new(1.0, 0.95, 0.7, 0.18));
                draw_texture_ex(&a.orb, c.x - r, c.y - r, WHITE, DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() });
            }
        }
    }

    for wv in &world.waves {
        if wv.radius > 0.0 && wv.purify {
            let f = 1.0 - wv.radius / wv.max_radius;
            draw_circle(wv.center.x, wv.center.y, wv.radius, Color::new(1.0, 0.85, 0.92, 0.08 * f));
            draw_circle_lines(wv.center.x, wv.center.y, wv.radius, 6.0, Color::new(1.0, 0.6, 0.75, 0.8 * f + 0.1));
            draw_circle_lines(wv.center.x, wv.center.y, wv.radius - 6.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.7 * f));
            continue;
        }
        if wv.radius > 0.0 {
            let f = 1.0 - wv.radius / wv.max_radius;
            draw_circle_lines(wv.center.x, wv.center.y, wv.radius, 4.0, Color::new(1.0, 0.7, 0.3, 0.75 * f + 0.1));
            draw_circle_lines(wv.center.x, wv.center.y, wv.radius - 5.0, 1.5, Color::new(1.0, 0.95, 0.8, 0.5 * f));
        }
    }
    for st in &world.strikes {
        let drop = st.delay * 320.0;
        let (sx, sy) = (st.pos.x + drop * 0.35, st.pos.y - drop);
        draw_line(sx + 10.0, sy - 22.0, sx, sy, 3.0, Color::new(0.55, 0.85, 1.0, 0.6));
        draw_poly(sx, sy, 5, 5.0, time as f32 * 400.0, Color::new(1.0, 0.92, 0.4, 1.0));
    }
    for (c, r, age) in &fx.impacts {
        let t = age / 0.3;
        draw_circle(c.x, c.y, r * (0.6 + 0.6 * t), Color::new(1.0, 0.95, 0.6, 0.45 * (1.0 - t)));
    }
    for (from, to, age) in &fx.bolts {
        let a = 1.0 - age / 0.16;
        // Jagged line: offset midpoints perpendicular to the segment.
        let d = *to - *from;
        let n = vec2(-d.y, d.x).normalize_or_zero();
        let mut prev = *from;
        for k in 1..=4 {
            let t = k as f32 / 4.0;
            let jag = if k < 4 { ((k as f32 * 7.3 + from.x) .sin()) * 7.0 } else { 0.0 };
            let p2 = *from + d * t + n * jag;
            draw_line(prev.x, prev.y, p2.x, p2.y, 3.0, Color::new(0.75, 0.85, 1.0, a));
            draw_line(prev.x, prev.y, p2.x, p2.y, 1.2, Color::new(1.0, 1.0, 1.0, a));
            prev = p2;
        }
    }
    for (c, dir, r, full, age) in &fx.slashes {
        let a = 1.0 - age / 0.2;
        let base = dir.y.atan2(dir.x);
        let (span, segs) = if *full { (std::f32::consts::PI, 24) } else { (60f32.to_radians(), 10) };
        for k in 0..segs {
            let t0 = base - span + 2.0 * span * k as f32 / segs as f32;
            let t1 = base - span + 2.0 * span * (k + 1) as f32 / segs as f32;
            let rr = r * (0.75 + 0.25 * (age / 0.2));
            draw_line(c.x + t0.cos() * rr, c.y + t0.sin() * rr, c.x + t1.cos() * rr, c.y + t1.sin() * rr, 5.0, Color::new(0.85, 0.95, 1.0, 0.8 * a));
        }
    }

    let gold = world.weapons.iter().any(|w| w.kind == WeaponKind::Ofuda && w.awakened);
    let ink = if gold { Color::new(1.0, 0.85, 0.35, 1.0) } else { WHITE };
    let pr = &world.projectiles;
    let t = time as f32;
    for i in 0..pr.len() {
        let (q, v) = (pr.pos[i], pr.vel[i]);
        if !near(&view, q.x, q.y, 64.0) {
            continue;
        }
        match pr.kind[i] {
            ProjKind::Talisman => sprites.push(
                &a.talisman,
                q.x - 3.0,
                q.y - 5.0,
                ink,
                DrawTextureParams { dest_size: Some(crate::sprites::nat(&a.talisman)), rotation: v.y.atan2(v.x) + std::f32::consts::FRAC_PI_2, ..Default::default() },
            ),
            ProjKind::Foxfire | ProjKind::Comet => {
                let comet = pr.kind[i] == ProjKind::Comet;
                // While charging the orb swells and sparks spiral into it.
                let grow = if pr.charge[i] > 0.0 { 1.0 - (pr.charge[i] / 0.55).min(1.0) } else { 1.0 };
                let r = 3.0 + 5.0 * grow;
                if pr.charge[i] > 0.0 {
                    for k in 0..5 {
                        let a = t * 9.0 + k as f32 * 1.256;
                        let d = 14.0 * (1.0 - ((t * 2.5 + k as f32 * 0.2) % 1.0));
                        draw_rectangle(q.x + a.cos() * d - 1.0, q.y + a.sin() * d - 1.0, 2.0, 2.0, Color::new(0.75, 0.95, 1.0, 0.9));
                    }
                } else {
                    let back = v.normalize_or_zero() * -1.0;
                    for k in 1..4 {
                        let c = q + back * (k as f32 * 4.0);
                        draw_circle(c.x, c.y, r * (1.0 - k as f32 * 0.22), Color::new(0.25, 0.6, 1.0, 0.35 / k as f32));
                    }
                }
                let (outer, mid) = if comet {
                    (Color::new(0.55, 0.45, 1.0, 0.35), Color::new(0.5, 0.75, 1.0, 1.0))
                } else {
                    (Color::new(0.2, 0.55, 1.0, 0.3), Color::new(0.25, 0.65, 1.0, 1.0))
                };
                draw_circle(q.x, q.y, r * 1.8, outer);
                draw_circle(q.x, q.y, r, mid);
                draw_circle(q.x, q.y, r * 0.55, Color::new(0.85, 0.97, 1.0, 1.0));
                // Swirl: two bright arcs rotating around the core.
                for k in 0..2 {
                    let a = t * 14.0 + k as f32 * std::f32::consts::PI;
                    draw_circle(q.x + a.cos() * r * 0.7, q.y + a.sin() * r * 0.7, r * 0.22, WHITE);
                }
            }
            ProjKind::Spark => {
                draw_circle(q.x, q.y, 5.0, Color::new(0.4, 0.7, 1.0, 0.35));
                draw_circle(q.x, q.y, 2.5, Color::new(0.85, 0.97, 1.0, 1.0));
            }
            ProjKind::GoldenOrb => {
                let r = 13.0;
                draw_circle(q.x, q.y, r * 1.7, Color::new(1.0, 0.55, 0.1, 0.22));
                draw_circle(q.x, q.y, r, Color::new(1.0, 0.65, 0.15, 0.95));
                draw_circle(q.x, q.y, r * 0.65, Color::new(1.0, 0.88, 0.4, 1.0));
                draw_circle(q.x, q.y, r * 0.3, WHITE);
                for k in 0..4 {
                    let a = t * 16.0 + k as f32 * std::f32::consts::FRAC_PI_2;
                    draw_line(q.x + a.cos() * r * 0.3, q.y + a.sin() * r * 0.3, q.x + (a + 0.9).cos() * r * 0.95, q.y + (a + 0.9).sin() * r * 0.95, 2.0, Color::new(1.0, 1.0, 0.85, 0.9));
                }
            }
            ProjKind::Familiar => {
                let back = v.normalize_or_zero() * -1.0;
                for k in 1..4 {
                    let c = q + back * (k as f32 * 3.5);
                    draw_rectangle(c.x - 1.0, c.y - 1.0, 2.0, 2.0, Color::new(1.0, 0.75, 0.3, 0.8 - k as f32 * 0.2));
                }
                sprites.push(&a.familiar, (q.x - 6.0).round(), (q.y - 5.0).round(), WHITE, DrawTextureParams { dest_size: Some(crate::sprites::nat(&a.familiar)), flip_x: v.x < 0.0, ..Default::default() });
            }
            ProjKind::CrescentWave => {
                let hannya = world.form_kind() == Some(SpecialKind::Hannya);
                let r = pr.radius[i] * 1.6;
                let base = v.y.atan2(v.x);
                let life = pr.life[i] / pr.max_life[i];
                let (outer, inner) = if hannya {
                    (Color::new(0.55, 0.0, 0.08, 0.8 * life + 0.2), Color::new(1.0, 0.35, 0.35, life))
                } else {
                    (Color::new(0.45, 0.35, 0.9, 0.7 * life + 0.2), Color::new(0.9, 0.9, 1.0, life))
                };
                for s in 0..10 {
                    let a0 = base - 1.2 + s as f32 * 0.24;
                    let a1 = a0 + 0.24;
                    let thick = 6.0 * (1.0 - ((s as f32 - 4.5).abs() / 5.0));
                    draw_line(q.x + a0.cos() * r, q.y + a0.sin() * r, q.x + a1.cos() * r, q.y + a1.sin() * r, thick + 2.0, outer);
                    draw_line(q.x + a0.cos() * r, q.y + a0.sin() * r, q.x + a1.cos() * r, q.y + a1.sin() * r, thick * 0.5, inner);
                }
            }
            ProjKind::Seal => {
                let back = v.normalize_or_zero() * -1.0;
                for k in 1..5 {
                    let c = q + back * (k as f32 * 4.0);
                    draw_circle(c.x, c.y, 4.0 - k as f32 * 0.7, Color::new(1.0, 0.8, 0.3, 0.5 - k as f32 * 0.1));
                }
                draw_circle(q.x, q.y, 8.0, Color::new(1.0, 0.85, 0.35, 0.25));
                sprites.push(&a.talisman, q.x - 3.0, q.y - 5.0, Color::new(1.0, 0.85, 0.35, 1.0), DrawTextureParams { dest_size: Some(crate::sprites::nat(&a.talisman)), rotation: v.y.atan2(v.x) + std::f32::consts::FRAC_PI_2, ..Default::default() });
            }
            ProjKind::Petal => {
                let a = t * 10.0 + q.x * 0.1;
                draw_circle(q.x, q.y, 5.0, Color::new(1.0, 0.6, 0.8, 0.25));
                draw_ellipse(q.x, q.y, 3.5, 2.0, a.to_degrees(), Color::new(1.0, 0.72, 0.85, 1.0));
                draw_ellipse(q.x, q.y, 1.5, 1.0, a.to_degrees(), Color::new(1.0, 0.95, 0.98, 1.0));
            }
            ProjKind::SpiritBolt => {
                let back = v.normalize_or_zero() * -1.0;
                for k in 1..4 {
                    let c = q + back * (k as f32 * 4.0);
                    draw_circle(c.x, c.y, 3.5 - k as f32 * 0.8, Color::new(0.6, 0.8, 1.0, 0.5 - k as f32 * 0.12));
                }
                draw_circle(q.x, q.y, 6.0, Color::new(0.55, 0.8, 1.0, 0.3));
                draw_circle(q.x, q.y, 3.0, Color::new(0.9, 0.97, 1.0, 1.0));
            }
            ProjKind::SealOrb => {
                let r = pr.radius[i];
                let hue = (q.x * 0.01 + q.y * 0.013).sin() * 0.5 + 0.5;
                let col = Color::new(1.0, 0.55 + 0.3 * hue, 0.75 - 0.2 * hue, 1.0);
                draw_circle(q.x, q.y, r * 1.9, Color::new(col.r, col.g, col.b, 0.22));
                draw_circle(q.x, q.y, r, col);
                draw_circle(q.x, q.y, r * 0.5, WHITE);
            }
            ProjKind::RedStar => {
                let r = pr.radius[i];
                let back = v.normalize_or_zero() * -1.0;
                for k in 1..5 {
                    let c = q + back * (k as f32 * 5.0);
                    draw_circle(c.x, c.y, r * (0.6 - k as f32 * 0.1), Color::new(1.0, 0.2, 0.25, 0.4 - k as f32 * 0.08));
                }
                draw_circle(q.x, q.y, r * 1.6, Color::new(1.0, 0.2, 0.25, 0.22));
                let rot = t * 10.0;
                for k in 0..4 {
                    let a = rot + k as f32 * std::f32::consts::FRAC_PI_2;
                    let tip = vec2(q.x, q.y) + vec2(a.cos(), a.sin()) * r * 1.3;
                    let l = vec2(q.x, q.y) + vec2((a + 0.6).cos(), (a + 0.6).sin()) * r * 0.35;
                    let rr = vec2(q.x, q.y) + vec2((a - 0.6).cos(), (a - 0.6).sin()) * r * 0.35;
                    draw_triangle(tip, l, rr, Color::new(1.0, 0.25, 0.3, 1.0));
                }
                draw_circle(q.x, q.y, r * 0.35, Color::new(1.0, 0.9, 0.9, 1.0));
            }
            ProjKind::VoidOrb => {
                let r = pr.radius[i];
                draw_circle(q.x, q.y, r * 1.5, Color::new(0.5, 0.2, 1.0, 0.18));
                draw_circle(q.x, q.y, r, Color::new(0.55, 0.25, 1.0, 0.9));
                draw_circle(q.x, q.y, r * 0.72, Color::new(0.05, 0.02, 0.12, 1.0));
                for k in 0..10 {
                    let a = t * 3.0 + k as f32 * 0.628;
                    let rr = r * (0.9 + 0.12 * (t * 9.0 + k as f32).sin());
                    draw_rectangle(q.x + a.cos() * rr - 1.0, q.y + a.sin() * rr - 1.0, 2.0, 2.0, Color::new(0.9, 0.8, 1.0, 1.0));
                }
                draw_circle_lines(q.x, q.y, r, 2.0, Color::new(0.85, 0.7, 1.0, 1.0));
            }
            ProjKind::Fang => {
                let back = v.normalize_or_zero() * -1.0;
                for k in 1..4 {
                    let c = q + back * (k as f32 * 3.5);
                    draw_circle(c.x, c.y, 2.5 - k as f32 * 0.5, Color::new(0.45, 0.9, 0.3, 0.6 - k as f32 * 0.15));
                }
                let a0 = v.y.atan2(v.x);
                let tip = q + sim::Vec2::from_angle(a0) * 6.0;
                let b1 = q + sim::Vec2::from_angle(a0 + 2.4) * 4.0;
                let b2 = q + sim::Vec2::from_angle(a0 - 2.4) * 4.0;
                draw_triangle(vec2(tip.x, tip.y), vec2(b1.x, b1.y), vec2(b2.x, b2.y), Color::new(0.95, 0.92, 0.85, 1.0));
                draw_circle(q.x, q.y, 1.5, Color::new(0.5, 0.95, 0.35, 1.0));
            }
            ProjKind::Shuriken => {
                let size = pr.radius[i] * 2.2;
                draw_circle(q.x, q.y, size * 0.55, Color::new(0.8, 0.9, 1.0, 0.12));
                draw_texture_ex(
                    &a.shuriken,
                    q.x - size / 2.0,
                    q.y - size / 2.0,
                    WHITE,
                    DrawTextureParams { dest_size: Some(vec2(size, size)), rotation: t * 22.0, ..Default::default() },
                );
            }
        }
    }
    sprites.flush();
    draw_smokes(&fx.smokes);
    for t in &world.tethers {
        if let Some(j) = world.enemies.id.iter().position(|id| *id == t.enemy) {
            let e = world.enemies.pos[j];
            let from = vec2(p.x, p.y - 8.0);
            let to = vec2(e.x, e.y - 4.0);
            let d = to - from;
            let links = (d.length() / 7.0).max(2.0) as usize;
            let sag = vec2(-d.y, d.x).normalize_or_zero() * (6.0 * (time as f32 * 4.0 + e.x * 0.05).sin());
            for k in 0..=links {
                let u = k as f32 / links as f32;
                let c = from + d * u + sag * (4.0 * u * (1.0 - u));
                let col = if k % 2 == 0 { Color::new(0.7, 0.55, 1.0, 0.9) } else { Color::new(0.4, 0.28, 0.8, 0.9) };
                draw_circle_lines(c.x, c.y, 2.2, 1.2, col);
            }
            draw_circle(to.x, to.y, 6.0, Color::new(0.6, 0.45, 1.0, 0.25));
        }
    }
    for s in &world.scars {
        let k = (s.life / 1.5).min(1.0);
        draw_line(s.from.x, s.from.y - 6.0, s.to.x, s.to.y - 6.0, 4.0, Color::new(0.55, 0.4, 1.0, 0.35 * k));
        draw_line(s.from.x, s.from.y - 6.0, s.to.x, s.to.y - 6.0, 1.5, Color::new(1.0, 1.0, 1.0, 0.6 * k));
    }
    for (from, to, age) in &fx.iai {
        let k = 1.0 - age / 0.25;
        draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 10.0 * k, Color::new(0.55, 0.45, 1.0, 0.5 * k));
        draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 3.0, Color::new(1.0, 1.0, 1.0, k));
    }
    for (from, to, age) in &fx.lashes {
        let k = 1.0 - age / 0.22;
        let d = *to - *from;
        let n = vec2(-d.y, d.x).normalize_or_zero();
        let mut prev = *from;
        for s in 1..=10 {
            let u = s as f32 / 10.0;
            let bend = (u * std::f32::consts::PI).sin() * 10.0 * (1.0 - age / 0.22);
            let pt = *from + d * u + n * bend;
            let wdt = (9.0 * (1.0 - u * 0.65)) * k;
            draw_line(prev.x, prev.y - 8.0, pt.x, pt.y - 8.0, wdt + 2.0, Color::new(0.08, 0.0, 0.02, 0.8 * k));
            draw_line(prev.x, prev.y - 8.0, pt.x, pt.y - 8.0, wdt, Color::new(0.85, 0.12, 0.2, k));
            prev = pt;
        }
        draw_circle(to.x, to.y - 8.0, 4.0 * k, Color::new(1.0, 0.4, 0.4, k));
    }
    for (c, r, age) in &fx.devours {
        // Collapses inward: the hunger pulls.
        let k = age / 0.4;
        let rr = r * (1.0 - 0.7 * k);
        draw_circle(c.x, c.y, rr, Color::new(0.35, 0.0, 0.06, 0.2 * (1.0 - k)));
        draw_circle_lines(c.x, c.y, rr, 3.0, Color::new(0.9, 0.15, 0.2, 0.8 * (1.0 - k)));
    }
    for (c, r, age) in &fx.pulses {
        let k = age / 0.4;
        draw_circle_lines(c.x, c.y, r * (0.4 + 0.6 * k), 2.5, Color::new(0.6, 0.85, 1.0, 0.8 * (1.0 - k)));
    }
    for (from, to, age, st) in &fx.beams {
        if *st == style::MOON {
            let k = 1.0 - age / 0.45;
            draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 26.0 * k, Color::new(0.45, 0.6, 1.0, 0.25 * k));
            draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 13.0 * k, Color::new(0.75, 0.85, 1.0, 0.6 * k));
            draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 4.0 * k, Color::new(1.0, 1.0, 0.95, k));
            continue;
        }
        let k = 1.0 - age / 0.3;
        draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 16.0 * k, Color::new(0.35, 0.0, 0.05, 0.55 * k));
        draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 9.0 * k, Color::new(0.9, 0.05, 0.15, 0.85 * k));
        draw_line(from.x, from.y - 6.0, to.x, to.y - 6.0, 3.0 * k, Color::new(1.0, 0.75, 0.75, k));
    }
    draw_blasts(world, time as f32);
    draw_bursts(&fx.bursts);
    draw_hazards(world, time as f32, true);
    draw_art(world, time as f32);
    draw_bursts(&fx.free_bursts);
    draw_smokes(&fx.free_smokes);

    // Enemy fireballs: drawn above everything so they always read as danger.
    draw_hazards(world, time as f32, false);
    draw_mask(world, time as f32);
    for s in &world.shots {
        let r = sim::content::SHOT_RADIUS;
        let flick = 0.8 + 0.2 * (time as f32 * 20.0 + s.pos.x).sin();
        let c = crate::stage::bullet_color(s.style);
        draw_circle(s.pos.x, s.pos.y, r * 2.0, Color::new(c.r, c.g * 0.8, c.b * 0.8, 0.24 * flick));
        draw_circle(s.pos.x, s.pos.y, r, c);
        draw_circle(s.pos.x, s.pos.y, r * 0.5, Color::new(1.0, 0.97, 0.9, 1.0));
    }

    for (c, age) in &fx.rings {
        let t = age / 0.45;
        let r = sim::content::BARRIER_RADIUS * (0.2 + 0.8 * t.sqrt());
        draw_circle(c.x, c.y, r, Color::new(1.0, 0.95, 0.85, 0.18 * (1.0 - t)));
        draw_circle_lines(c.x, c.y, r, 3.0, Color::new(1.0, 0.9, 0.6, 0.9 * (1.0 - t)));
    }

    for pt in fx.particles.iter().filter(|pt| near(&view, pt.pos.x, pt.pos.y, 4.0)) {
        let alpha = 1.0 - pt.age / pt.life;
        let mut c = pt.color;
        c.a = alpha;
        draw_rectangle(pt.pos.x - 1.0, pt.pos.y - 1.0, 2.0, 2.0, c);
    }
    draw_drops(world, a, fx, &view, time);

    // The world was drawn into a screen-resolution target (cheap on high-DPI screens):
    // put it on screen before the crisp overlays.
    if let Some(rt) = target {
        set_default_camera();
        draw_texture_ex(&rt.texture, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(screen_width(), screen_height())), ..Default::default() });
        if let Some((tex, x, y, tint, params)) = hires_player.take() {
            let direct = Camera2D { target: cam.target, zoom: cam.zoom, offset: cam.offset, rotation: cam.rotation, render_target: None, viewport: cam.viewport };
            set_camera(&direct);
            draw_texture_ex(&tex, x, y, tint, params);
            set_default_camera();
        }
    }
    // Damage numbers are drawn in screen space so the text stays crisp.
    // Screen-space overlays use the virtual UI space so they scale with the rest of the UI.
    kit::begin_ui();
    let player = vec2(world.player.pos.x, world.player.pos.y);
    for n in &fx.numbers {
        let s = kit::to_ui(cam.world_to_screen(player + n.offset));
        let alpha = 1.0 - (n.age / 0.6).powi(2);
        let c = if n.crit { Color::new(1.0, 0.8, 0.25, alpha) } else { Color::new(1.0, 1.0, 1.0, alpha) };
        let c = match n.elem {
            Some(e) if n.mult > 1.0 => {
                let ec = crate::ui::element_color(e);
                Color::new(ec.r, ec.g, ec.b, alpha)
            }
            _ if n.mult < 1.0 => Color::new(kit::FAINT.r, kit::FAINT.g, kit::FAINT.b, alpha),
            _ => c,
        };
        let x = s.x - n.width / 2.0;
        kit::text(&n.txt, x + 1.0, s.y + 1.0, n.size, Color::new(0.0, 0.0, 0.0, alpha));
        kit::text(&n.txt, x, s.y, n.size, c);
    }
    for (pos, r, age, label, width) in &fx.gear_drops {
        let sp = kit::to_ui(cam.world_to_screen(vec2(pos.x, pos.y - 30.0 - age * 14.0)));
        let a = (1.0 - age / 2.2).clamp(0.0, 1.0);
        let c = crate::armory::rarity_color(*r);
        kit::text(label, sp.x - width / 2.0 + 1.0, sp.y + 1.0, 15.0, Color::new(0.0, 0.0, 0.0, a));
        kit::text(label, sp.x - width / 2.0, sp.y, 15.0, Color::new(c.r, c.g, c.b, a));
    }
    if let Some(s) = world.shrine {
        let sp = kit::to_ui(cam.world_to_screen(vec2(s.x, s.y)));
        let (w, h) = (kit::screen_width(), kit::screen_height());
        // Only when the shrine sprite is fully out of view (it extends ~60px above its anchor).
        if sp.x < -30.0 || sp.y < -10.0 || sp.x > w + 30.0 || sp.y > h + 70.0 {
            let c = vec2(w / 2.0, h / 2.0);
            let d = (sp - c).normalize_or_zero();
            // Push the arrow to the screen edge along the direction to the shrine.
            let t = ((w / 2.0 - 40.0) / d.x.abs().max(1e-3)).min((h / 2.0 - 40.0) / d.y.abs().max(1e-3));
            let tip = c + d * t;
            let side = vec2(-d.y, d.x);
            let pulse = 0.7 + 0.3 * (time * 5.0).sin() as f32;
            draw_triangle(tip + d * 14.0, tip - d * 8.0 + side * 10.0, tip - d * 8.0 - side * 10.0, Color::new(1.0, 0.85, 0.4, pulse));
            draw_texture_ex(&a.shrine, tip.x - d.x * 34.0 - 12.0, tip.y - d.y * 34.0 - 14.0, Color::new(1.0, 1.0, 1.0, 0.85), DrawTextureParams { dest_size: Some(vec2(24.0, 27.0)), ..Default::default() });
        }
    }
    // Off-screen drops get a badge and an arrow at the screen edge; chests last, on top.
    let pulse = 0.7 + 0.3 * (time * 6.0).sin() as f32;
    let ui = |c: &sim::Vec2| kit::to_ui(cam.world_to_screen(vec2(c.x, c.y)));
    for (c, _) in world.coins.iter().filter(|(_, v)| *v >= 50) {
        edge_badge(ui(c), &a.coin, vec2(18.0, 18.0), Color::new(1.0, 0.85, 0.3, pulse), 2.0);
    }
    for (c, k) in &world.pickups {
        let col = pickup_color(*k);
        edge_badge(ui(c), a.pickup(*k), vec2(20.0, 20.0), Color::new(col.r, col.g, col.b, pulse), 2.0);
    }
    for (c, tier) in &world.chests {
        edge_badge(ui(c), &a.chest, vec2(20.0, 17.0), Color::new(1.0, 0.82, 0.35, pulse), if *tier >= 3 { 3.0 } else { 2.0 });
    }
    if let Some((line, left)) = &fx.bark {
        let s = kit::to_ui(cam.world_to_screen(vec2(p.x, p.y - 34.0)));
        let d = kit::measure(line, 20);
        let alpha = left.min(0.4) / 0.4;
        draw_rectangle(s.x - d.width / 2.0 - 8.0, s.y - 20.0, d.width + 16.0, 28.0, Color::new(0.05, 0.03, 0.1, 0.8 * alpha));
        kit::text(line, s.x - d.width / 2.0, s.y, 20.0, Color::new(1.0, 0.9, 0.95, alpha));
    }
}

/// Ninefold Foxfire: golden aura and nine lashing foxfire tails behind the player.
fn draw_fox_aura_and_tails(world: &World, t: f32) {
    let p = world.player.pos;
    let pulse = 0.5 + 0.5 * (t * 6.0).sin();
    let ar = sim::content::FOX_AURA_RADIUS;
    draw_circle(p.x, p.y - 6.0, ar * 1.25, Color::new(1.0, 0.55, 0.1, 0.07 + 0.04 * pulse));
    draw_circle(p.x, p.y - 6.0, ar, Color::new(1.0, 0.7, 0.2, 0.10 + 0.05 * pulse));
    draw_circle_lines(p.x, p.y - 6.0, ar, 1.5, Color::new(1.0, 0.85, 0.4, 0.5));
    for (k, (root, tip)) in world.fox_tails().iter().enumerate() {
        let d = *tip - *root;
        let n = vec2(-d.y, d.x).normalize_or_zero();
        // A curved, tapering flame: bend the midpoint sideways, wave along the length.
        let bend = (t * 6.0 + k as f32 * 1.3).sin() * 14.0;
        let steps = 22;
        for i in 0..=steps {
            let u = i as f32 / steps as f32;
            let base = vec2(root.x, root.y) + vec2(d.x, d.y) * u;
            let off = n * (bend * 4.0 * u * (1.0 - u) + (t * 11.0 - u * 6.0 + k as f32).sin() * 2.0 * u);
            let c = base + off;
            let r = 7.5 * (1.0 - u * 0.7);
            draw_circle(c.x, c.y, r * 1.5, Color::new(1.0, 0.45, 0.05, 0.12));
            draw_circle(c.x, c.y, r, Color::new(1.0, 0.62, 0.12, 0.55));
            draw_circle(c.x, c.y, r * 0.5, Color::new(1.0, 0.92, 0.55, 0.75));
        }
    }
}

/// Golden spirit motes around Riku: a few in base form, a storm while transformed.
fn draw_spirit_motes(p: sim::Vec2, t: f32, fox: bool) {
    let n = if fox { 28 } else { 6 };
    for k in 0..n {
        let kf = k as f32;
        let phase = (t * (0.5 + (k % 5) as f32 * 0.08) + kf * 0.37).fract();
        let ang = kf * 2.399;
        let rad = if fox { 14.0 + (k % 7) as f32 * 7.0 } else { 9.0 + (k % 3) as f32 * 3.0 };
        let x = p.x + ang.cos() * rad;
        let y = p.y - 8.0 + ang.sin() * rad * 0.5 - phase * if fox { 46.0 } else { 18.0 };
        let a = (1.0 - phase) * if fox { 0.95 } else { 0.55 };
        let s = if fox && k % 3 == 0 { 2.0 } else { 1.0 };
        draw_rectangle(x, y, s, s, Color::new(1.0, 0.85, 0.35, a));
    }
}

/// Seconds a scattering crew flickers after its Leader falls.
const SCATTER_CUE: f32 = 0.5;

/// Client-only burst style: a Leader's banner collapsing.
const LEADER_FELL: u8 = 22;

fn leader_color() -> Color {
    crate::ui::enemy_rarity_color(EnemyRarity::Leader).unwrap_or(WHITE)
}

fn burst_life(st: u8) -> f32 {
    match st {
        LEADER_FELL => 0.7,
        style::SPINE => 0.45,
        16 => 0.45,
        21 => 0.3,
        style::YINYANG => 0.5,
        style::PURIFY => 1.0,
        17 => 0.3,
        18 => 0.8,
        style::FRENZY => 0.7,
        style::VENOM => 0.3,
        style::METEOR => 0.8,
        style::SKYBOLT => 0.3,
        style::COMET => 0.5,
        style::QUAKE => 0.5,
        style::SEAL => 0.3,
        1 => 0.7,
        2 => 0.4,
        3 => 1.0,
        _ => 0.35,
    }
}

/// Shun's oversized nodachi in his right hand. At rest it's a pre-drawn 45-degree sprite
/// (pixel-clean, no rotation) snapped to whole pixels; mid-swing the straight blade rotates.
fn draw_blade(a: &Assets, p: sim::Vec2, flip: bool, hannya: bool, swing: Option<(f32, Vec2)>) {
    // Same origin the body sprite uses, so the grip lines up with his hand pixel.
    let (sx, sy) = ((p.x - 12.0).round(), (p.y - 26.0).round());
    let hand = vec2(if flip { sx + 4.0 } else { sx + 19.0 }, sy + 21.0);
    match swing {
        None => {
            let tex = if hannya { &a.cleaver_rest_hannya } else { &a.cleaver_rest };
            // Grip anchor inside the 30x30 sprite is (5, 23); mirror it when facing left.
            let x = if flip { hand.x - 24.0 } else { hand.x - 5.0 };
            draw_texture_ex(tex, x, hand.y - 23.0, WHITE, DrawTextureParams { dest_size: Some(crate::sprites::nat(tex)), flip_x: flip, ..Default::default() });
        }
        Some((left, dir)) => {
            let tex = if hannya { &a.cleaver_hannya } else { &a.cleaver };
            let side = if flip { -1.0 } else { 1.0 };
            let k = 1.0 - left / 0.2;
            let d = if dir == Vec2::ZERO { vec2(side, 0.0) } else { dir };
            // Sweep the blade through ~150 degrees around the swing direction.
            let rot = d.y.atan2(d.x) + std::f32::consts::FRAC_PI_2 - 1.3 + 2.6 * k;
            draw_texture_ex(
                tex,
                hand.x - 4.5,
                hand.y - 35.0,
                WHITE,
                DrawTextureParams { dest_size: Some(crate::sprites::nat(tex)), rotation: rot, pivot: Some(hand), flip_x: flip, ..Default::default() },
            );
        }
    }
}

/// Hannya Unbound: a pulsing crimson-black aura with dark flames licking upward.
fn draw_hannya_aura(world: &World, t: f32) {
    let p = world.player.pos;
    let pulse = 0.5 + 0.5 * (t * 7.0).sin();
    let r = sim::content::HANNYA_AURA_RADIUS;
    draw_circle(p.x, p.y - 6.0, r * 1.3, Color::new(0.3, 0.0, 0.05, 0.16 + 0.06 * pulse));
    draw_circle(p.x, p.y - 6.0, r, Color::new(0.55, 0.0, 0.08, 0.14 + 0.06 * pulse));
    draw_circle_lines(p.x, p.y - 6.0, r * (0.95 + 0.05 * pulse), 2.0, Color::new(1.0, 0.2, 0.25, 0.6));
    for k in 0..18 {
        let kf = k as f32;
        let ph = (t * 1.6 + kf * 0.29).fract();
        let a = kf * 2.39;
        let rr = 10.0 + (k % 5) as f32 * 9.0;
        let x = p.x + a.cos() * rr;
        let y = p.y - 4.0 + a.sin() * rr * 0.5 - ph * 40.0;
        let c = if k % 3 == 0 { Color::new(0.05, 0.0, 0.02, 0.8 * (1.0 - ph)) } else { Color::new(0.95, 0.1, 0.15, 0.85 * (1.0 - ph)) };
        draw_rectangle(x, y, 2.0, 3.0, c);
    }
    // Hannya's Wrath charging: a ring of dread collapsing inward.
    if world.player.form <= sim::content::HANNYA_FINALE_CHARGE {
        let k = 1.0 - world.player.form / sim::content::HANNYA_FINALE_CHARGE;
        let rr = world.reach_cap(sim::content::HANNYA_FINALE_RADIUS) * (1.0 - k * 0.8);
        draw_circle_lines(p.x, p.y - 6.0, rr, 4.0, Color::new(1.0, 0.1, 0.15, 0.4 + 0.5 * k));
        draw_circle(p.x, p.y - 30.0, 6.0 + 16.0 * k, Color::new(0.1, 0.0, 0.03, 0.9));
        draw_circle_lines(p.x, p.y - 30.0, 6.0 + 16.0 * k, 2.0, Color::new(1.0, 0.2, 0.25, 1.0));
    }
}

/// Spirit wisps around Shun: pale violet in base form, crimson and black when unbound.
fn draw_reaper_wisps(p: sim::Vec2, t: f32, hannya: bool) {
    let n = if hannya { 10 } else { 7 };
    for k in 0..n {
        let kf = k as f32;
        let a = t * (0.9 + 0.1 * (k % 3) as f32) + kf * std::f32::consts::TAU / n as f32;
        let r = 13.0 + 3.0 * (t * 2.0 + kf).sin();
        let x = p.x + a.cos() * r;
        let y = p.y - 10.0 + a.sin() * r * 0.6;
        let c = if hannya {
            if k % 2 == 0 { Color::new(0.9, 0.1, 0.15, 0.8) } else { Color::new(0.05, 0.0, 0.03, 0.8) }
        } else if k % 3 == 0 {
            Color::new(0.2, 0.15, 0.3, 0.6)
        } else {
            Color::new(0.75, 0.7, 1.0, 0.55)
        };
        draw_rectangle(x, y, 2.0, 2.0, c);
    }
}

/// Telegraphs for delayed blasts: falling meteors and comets, sky-bolt warnings.
fn draw_blasts(world: &World, t: f32) {
    for b in &world.blasts {
        let k = 1.0 - b.delay / b.max_delay; // 0 -> 1 as it lands
        let c = b.pos;
        match b.style {
            style::METEOR | style::COMET => {
                let meteor = b.style == style::METEOR;
                let (ring, body, core) = if meteor {
                    (Color::new(1.0, 0.3, 0.1, 0.35 + 0.4 * k), Color::new(1.0, 0.45, 0.1, 1.0), Color::new(1.0, 0.9, 0.5, 1.0))
                } else {
                    (Color::new(0.3, 0.6, 1.0, 0.3 + 0.4 * k), Color::new(0.3, 0.65, 1.0, 1.0), Color::new(0.9, 0.97, 1.0, 1.0))
                };
                draw_circle(c.x, c.y, b.radius * k, Color::new(ring.r, ring.g, ring.b, 0.12));
                draw_circle_lines(c.x, c.y, b.radius * (1.0 - 0.6 * k) + 4.0, 2.0, ring);
                // The body falls in from the upper right with a fiery trail.
                let size = if meteor { 18.0 } else { 9.0 };
                let off = vec2(1.0, -1.6) * (1.0 - k) * 320.0;
                let head = vec2(c.x, c.y) + off;
                for i in 1..10 {
                    let tp = head + vec2(1.0, -1.6).normalize() * (i as f32 * size * 0.7);
                    let a = 0.7 * (1.0 - i as f32 / 10.0);
                    draw_circle(tp.x, tp.y, size * (1.0 - i as f32 / 12.0), Color::new(body.r, body.g * 0.8, body.b, a * 0.6));
                }
                draw_circle(head.x, head.y, size * 1.4, Color::new(body.r, body.g, body.b, 0.3));
                draw_circle(head.x, head.y, size, body);
                draw_circle(head.x, head.y, size * 0.5, core);
                let _ = t;
            }
            style::SKYBOLT => {
                draw_circle_lines(c.x, c.y, b.radius * (1.0 - 0.5 * k), 1.5, Color::new(0.8, 0.9, 1.0, 0.8));
            }
            style::SEAL => {
                let gold = Color::new(1.0, 0.85, 0.3, 0.5 + 0.5 * k);
                draw_circle_lines(c.x, c.y, b.radius * (1.0 - 0.6 * k), 1.5, Color::new(1.0, 0.4, 0.3, 0.3 + 0.5 * k));
                let (spin, q) = (t * 3.0, 9.0);
                for i in 0..4 {
                    let (a0, a1) = (spin + i as f32 * std::f32::consts::FRAC_PI_2, spin + (i + 1) as f32 * std::f32::consts::FRAC_PI_2);
                    draw_line(c.x + a0.cos() * q, c.y + a0.sin() * q, c.x + a1.cos() * q, c.y + a1.sin() * q, 2.0, gold);
                }
                draw_circle(c.x, c.y, 2.0, gold);
            }
            style::YINYANG => {
                draw_circle_lines(c.x, c.y, b.radius * (1.0 - 0.5 * k) + 4.0, 2.0, Color::new(1.0, 0.5, 0.6, 0.4 + 0.5 * k));
                // The orb drops from above, spinning.
                let size = b.radius * 0.45;
                let y = c.y - (1.0 - k) * 220.0;
                draw_yinyang(vec2(c.x, y), size, t * 8.0);
            }
            _ => {}
        }
    }
}

/// Spines erupting from the ground: a cracked ring and segmented spikes thrusting up.
fn draw_spine_burst(c: Vec2, r: f32, k: f32) {
    let rise = (k / 0.35).min(1.0);
    let fade = 1.0 - k;
    draw_ellipse(c.x, c.y + 2.0, r * 0.8, r * 0.3, 0.0, Color::new(0.1, 0.0, 0.02, 0.5 * fade));
    for i in 0..5 {
        let a = i as f32 * 1.256 + c.x * 0.01;
        let off = vec2(a.cos() * r * 0.35, a.sin() * r * 0.15);
        let h = r * (0.7 + 0.3 * ((i * 3 % 4) as f32 / 4.0)) * rise;
        let base = c + off;
        // Segments stacked up the spike.
        for s in 0..4 {
            let y0 = base.y - h * s as f32 / 4.0;
            let w = (4.0 - s as f32) * 1.4;
            let col = if s % 2 == 0 { Color::new(0.75, 0.1, 0.18, fade) } else { Color::new(0.3, 0.02, 0.06, fade) };
            draw_rectangle(base.x - w / 2.0, y0 - h / 4.0, w, h / 4.0 + 1.0, col);
        }
        draw_triangle(vec2(base.x - 1.5, base.y - h), vec2(base.x + 1.5, base.y - h), vec2(base.x, base.y - h - 6.0), Color::new(0.92, 0.86, 0.78, fade));
    }
}

/// Toya's four crimson tendrils (base form) or the great centipede tail (Rampage).
fn draw_tendrils_and_tail(world: &World, t: f32, mukade: bool) {
    let p = world.player.pos;
    let root = vec2(p.x, p.y - 8.0);
    let back = if world.player.facing.x < 0.0 { 1.0 } else { -1.0 };
    let (n, len, width) = if mukade { (6, 34.0, 4.0) } else { (4, 20.0, 2.6) };
    for k in 0..n {
        let kf = k as f32;
        let base_a = -std::f32::consts::FRAC_PI_2 + back * (0.5 + 0.35 * kf / n as f32) + if k % 2 == 0 { 0.25 } else { -0.25 };
        let mut prev = root;
        for s in 1..=8 {
            let u = s as f32 / 8.0;
            let sway = (t * 3.0 + kf * 1.7 + u * 3.0).sin() * 0.5 * u;
            let a = base_a + sway + back * u * 0.8;
            let pt = prev + vec2(a.cos(), a.sin()) * (len / 8.0);
            let wd = width * (1.0 - u * 0.6);
            draw_line(prev.x, prev.y, pt.x, pt.y, wd + 1.5, Color::new(0.06, 0.0, 0.02, 0.9));
            draw_line(prev.x, prev.y, pt.x, pt.y, wd, Color::new(0.78, 0.1, 0.18, 1.0));
            prev = pt;
        }
        draw_circle(prev.x, prev.y, width * 0.5, Color::new(1.0, 0.45, 0.45, 1.0));
    }
    if mukade {
        // The great centipede tail: a segmented arc over his back with legs along it.
        let segs = 14;
        let mut pts = Vec::with_capacity(segs);
        for s in 0..segs {
            let u = s as f32 / (segs - 1) as f32;
            let ang = std::f32::consts::PI * (0.15 + 0.85 * u);
            let wave = (t * 4.0 - u * 6.0).sin() * 4.0;
            let x = root.x + back * (ang.cos() * -34.0 - 6.0) + back * wave * 0.3;
            let y = root.y - ang.sin() * 46.0 + u * 10.0 + wave * 0.5;
            pts.push(vec2(x, y));
        }
        for (s, pt) in pts.iter().enumerate().rev() {
            let u = s as f32 / (segs - 1) as f32;
            let r = 7.0 * (1.0 - u * 0.55);
            // Legs.
            let leg = (t * 12.0 + s as f32).sin() * 3.0;
            draw_line(pt.x - r, pt.y, pt.x - r - 5.0, pt.y + 4.0 + leg, 1.5, Color::new(0.85, 0.2, 0.25, 1.0));
            draw_line(pt.x + r, pt.y, pt.x + r + 5.0, pt.y + 4.0 - leg, 1.5, Color::new(0.85, 0.2, 0.25, 1.0));
            draw_circle(pt.x, pt.y, r + 1.5, Color::new(0.06, 0.0, 0.02, 1.0));
            draw_circle(pt.x, pt.y, r, if s % 2 == 0 { Color::new(0.55, 0.05, 0.12, 1.0) } else { Color::new(0.25, 0.02, 0.05, 1.0) });
            // Blood-red energy pulsing down the segments.
            let pulse = ((t * 6.0 - u * 8.0).sin() * 0.5 + 0.5).powi(3);
            draw_circle(pt.x, pt.y - r * 0.3, r * 0.45, Color::new(1.0, 0.25, 0.3, 0.3 + 0.6 * pulse));
        }
        // Pincers at the tip.
        if let Some(tip) = pts.last() {
            draw_triangle(*tip, *tip + vec2(-6.0, -6.0), *tip + vec2(-1.0, -9.0), Color::new(0.92, 0.86, 0.78, 1.0));
            draw_triangle(*tip, *tip + vec2(6.0, -6.0), *tip + vec2(1.0, -9.0), Color::new(0.92, 0.86, 0.78, 1.0));
        }
        draw_circle(p.x, p.y - 6.0, 60.0, Color::new(0.5, 0.0, 0.08, 0.08 + 0.04 * (t * 6.0).sin()));
    }
}

/// Dark crimson motes and black fragments around Toya.
fn draw_curse_motes(p: sim::Vec2, t: f32, mukade: bool) {
    let n = if mukade { 22 } else { 7 };
    for k in 0..n {
        let kf = k as f32;
        let ph = (t * (0.6 + (k % 4) as f32 * 0.1) + kf * 0.31).fract();
        let a = kf * 2.39;
        let r = if mukade { 12.0 + (k % 6) as f32 * 7.0 } else { 10.0 + (k % 3) as f32 * 3.0 };
        let x = p.x + a.cos() * r;
        let y = p.y - 8.0 + a.sin() * r * 0.5 - ph * if mukade { 34.0 } else { 14.0 };
        let c = if k % 3 == 0 { Color::new(0.05, 0.0, 0.02, 0.9 * (1.0 - ph)) } else { Color::new(0.9, 0.12, 0.2, 0.8 * (1.0 - ph)) };
        draw_rectangle(x, y, 2.0, 2.0, c);
    }
}

/// Everything the player can pick up, drawn above enemies and effects so it stays readable.
fn draw_drops(world: &World, a: &Assets, fx: &Fx, view: &Rect, time: f64) {
    let mut sprites = Sprites::new();
    // Gems tier by value (merged gems are worth more and look it).
    for (g, &v) in world.gems.pos.iter().zip(&world.gems.value) {
        if !near(view, g.x, g.y, 16.0) {
            continue;
        }
        let (size, tint) = match v {
            0..=4 => (7.0, WHITE),
            5..=24 => (9.0, Color::new(0.45, 1.0, 0.45, 1.0)),
            25..=99 => (11.0, Color::new(1.0, 0.4, 0.45, 1.0)),
            _ => (14.0, Color::new(1.0, 0.85, 0.3, 1.0)),
        };
        if v >= 25 {
            let pulse = 0.2 + 0.1 * (time * 4.0 + g.x as f64).sin() as f32;
            draw_circle(g.x, g.y, size * 0.9, Color::new(tint.r, tint.g, tint.b, pulse));
            draw_circle_lines(g.x, g.y, size * 0.9, 1.5, tint);
        }
        sprites.push(&a.gem, (g.x - size / 2.0).round(), (g.y - size / 2.0).round(), tint, DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() });
    }
    sprites.flush();
    for (c, v) in &world.coins {
        if !near(view, c.x, c.y, 16.0) {
            continue;
        }
        let size = if *v >= 50 { 16.0 } else if *v >= 5 { 12.0 } else { 10.0 };
        let bob = ((time * 4.0 + c.x as f64).sin() * 1.0) as f32;
        backing(vec2(c.x, c.y + bob), size * 0.75, Color::new(1.0, 0.85, 0.3, 1.0), 0.8);
        sprites.push(&a.coin, (c.x - size / 2.0).round(), (c.y - size / 2.0 + bob).round(), WHITE, DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() });
    }
    sprites.flush();
    for (c, k) in &world.pickups {
        draw_pickup(a, vec2(c.x, c.y), *k, time as f32);
    }
    for (c, tier) in &world.chests {
        let bob = ((time * 3.0).sin() * 1.5) as f32;
        let size = if *tier >= 3 { 2.0 } else { 1.0 };
        // Beacon like the floor pickups: gold for chests, a taller cycling pillar for boss chests.
        let t = time as f32;
        let col = if *tier >= 3 {
            let hcyc = (t * 0.5).fract();
            let ch = |o: f32| 0.6 + 0.4 * ((hcyc + o) * std::f32::consts::TAU).cos();
            Color::new(ch(0.0), ch(0.33), ch(0.67), 1.0)
        } else {
            Color::new(1.0, 0.82, 0.35, 1.0)
        };
        let pulse = 0.5 + 0.5 * (t * 4.0 + c.x * 0.1).sin();
        let ph = if *tier >= 3 { 220.0 } else { 150.0 };
        draw_rectangle(c.x - 7.0 * size, c.y - ph, 14.0 * size, ph, Color::new(col.r, col.g, col.b, 0.10 + 0.06 * pulse));
        draw_rectangle(c.x - 2.0 * size, c.y - ph, 4.0 * size, ph, Color::new(col.r, col.g, col.b, 0.2 + 0.1 * pulse));
        draw_ellipse(c.x, c.y + 3.0, (17.0 + 4.0 * pulse) * size, (6.0 + 1.5 * pulse) * size, 0.0, Color::new(col.r, col.g, col.b, 0.22));
        draw_ellipse_lines(c.x, c.y + 3.0, (17.0 + 4.0 * pulse) * size, (6.0 + 1.5 * pulse) * size, 0.0, 1.5, Color::new(col.r, col.g, col.b, 0.85));
        for i in 0..5 {
            let rise = (t * 0.8 + i as f32 * 0.2).fract();
            let x = c.x + ((i as f32 * 2.3 + t).sin()) * 10.0 * size;
            draw_rectangle(x, c.y - 6.0 - rise * 46.0 * size, 2.0, 2.0, Color::new(1.0, 1.0, 1.0, 1.0 - rise));
        }
        backing(vec2(c.x, c.y - 3.0 * size + bob), 11.0 * size, col, 0.6 + 0.4 * pulse);
        draw_texture_ex(
            &a.chest,
            (c.x - 7.0 * size).round(),
            (c.y - 9.0 * size + bob).round(),
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(14.0 * size, 12.0 * size)), ..Default::default() },
        );
    }
    for (pos, r, age, ..) in &fx.gear_drops {
        let c = crate::armory::rarity_color(*r);
        let a = (1.0 - age / 2.2).clamp(0.0, 1.0);
        let h = 70.0 + 30.0 * r.index() as f32;
        draw_rectangle(pos.x - 3.0, pos.y - h * a.sqrt(), 6.0, h * a.sqrt(), Color::new(c.r, c.g, c.b, 0.5 * a));
        draw_circle(pos.x, pos.y, 10.0 + 6.0 * age, Color::new(c.r, c.g, c.b, 0.3 * a));
    }
}
/// A dark disc with a coloured rim behind a drop, so it reads over bright effects.
fn backing(c: Vec2, r: f32, col: Color, alpha: f32) {
    draw_circle(c.x, c.y, r, Color::new(0.04, 0.03, 0.08, 0.7));
    draw_circle_lines(c.x, c.y, r, 1.5, Color::new(col.r, col.g, col.b, alpha));
}

/// When `sp` (UI space) is off screen: a badge with `tex` and an arrow at the edge pointing to it.
fn edge_badge(sp: Vec2, tex: &Texture2D, size: Vec2, col: Color, rim: f32) {
    let (w, h) = (kit::screen_width(), kit::screen_height());
    if sp.x >= 0.0 && sp.y >= 0.0 && sp.x <= w && sp.y <= h {
        return;
    }
    let ctr = vec2(w / 2.0, h / 2.0);
    let d = (sp - ctr).normalize_or_zero();
    let t = ((w / 2.0 - 34.0) / d.x.abs().max(1e-3)).min((h / 2.0 - 34.0) / d.y.abs().max(1e-3));
    let tip = ctr + d * t;
    let side = vec2(-d.y, d.x);
    let bc = tip - d * 24.0;
    draw_circle(bc.x, bc.y, 15.0, Color::new(0.05, 0.04, 0.1, 0.85));
    draw_circle_lines(bc.x, bc.y, 15.0, rim, col);
    draw_texture_ex(tex, bc.x - size.x / 2.0, bc.y - size.y / 2.0, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
    draw_triangle(tip + d * 10.0, tip - d * 4.0 + side * 7.0, tip - d * 4.0 - side * 7.0, col);
}

pub fn pickup_color(k: PickupKind) -> Color {
    match k {
        PickupKind::Feast => Color::new(0.45, 1.0, 0.55, 1.0),
        PickupKind::SwiftGeta => Color::new(0.45, 0.95, 1.0, 1.0),
        PickupKind::OniSake => Color::new(1.0, 0.35, 0.3, 1.0),
        PickupKind::ShrineBell => Color::new(1.0, 0.85, 0.3, 1.0),
        PickupKind::Hourglass => Color::new(0.6, 0.85, 1.0, 1.0),
        PickupKind::ExorcismSeal => Color::new(0.75, 0.55, 1.0, 1.0),
        PickupKind::SenPurse => Color::new(1.0, 0.75, 0.35, 1.0),
    }
}

/// A pickup on the ground: a coloured light pillar, a pulsing ring and the bobbing icon.
fn draw_pickup(a: &Assets, c: Vec2, k: PickupKind, t: f32) {
    let col = pickup_color(k);
    let pulse = 0.5 + 0.5 * (t * 4.0 + c.x * 0.1).sin();
    draw_rectangle(c.x - 6.0, c.y - 140.0, 12.0, 140.0, Color::new(col.r, col.g, col.b, 0.10 + 0.06 * pulse));
    draw_rectangle(c.x - 2.0, c.y - 140.0, 4.0, 140.0, Color::new(col.r, col.g, col.b, 0.18 + 0.1 * pulse));
    draw_ellipse(c.x, c.y + 4.0, 16.0 + 4.0 * pulse, 6.0 + 1.5 * pulse, 0.0, Color::new(col.r, col.g, col.b, 0.25));
    draw_ellipse_lines(c.x, c.y + 4.0, 16.0 + 4.0 * pulse, 6.0 + 1.5 * pulse, 0.0, 1.5, Color::new(col.r, col.g, col.b, 0.8));
    let tier = crate::ui::tier_color(k.tier());
    draw_ellipse_lines(c.x, c.y + 4.0, 21.0 + 4.0 * pulse, 8.0 + 1.5 * pulse, 0.0, 1.0, Color::new(tier.r, tier.g, tier.b, 0.9));
    let bob = (t * 3.0 + c.y * 0.1).sin() * 2.5;
    backing(vec2(c.x, c.y - 10.0 + bob), 13.0, col, 0.6 + 0.4 * pulse);
    draw_texture_ex(a.pickup(k), (c.x - 10.0).round(), (c.y - 20.0 + bob).round(), WHITE, DrawTextureParams { dest_size: Some(vec2(20.0, 20.0)), ..Default::default() });
    for i in 0..4 {
        let ph = (t * 0.8 + i as f32 * 0.25).fract();
        let x = c.x + ((i as f32 * 2.3 + t).sin()) * 9.0;
        draw_rectangle(x, c.y - 6.0 - ph * 40.0, 2.0, 2.0, Color::new(1.0, 1.0, 1.0, 1.0 - ph));
    }
}

/// A Gravity Well: a dark vortex with spiralling arms that tightens as it nears collapse.
fn draw_well(c: Vec2, r: f32, life: f32, t: f32) {
    let rr = r * (0.45 + 0.55 * life);
    draw_circle(c.x, c.y, rr, Color::new(0.05, 0.08, 0.25, 0.45));
    draw_circle(c.x, c.y, rr * 0.45, Color::new(0.0, 0.0, 0.05, 0.85));
    for arm in 0..4 {
        let mut prev = c;
        for k in 1..14 {
            let u = k as f32 / 13.0;
            let a = t * 5.0 + arm as f32 * std::f32::consts::FRAC_PI_2 + u * 3.5;
            let pt = c + vec2(a.cos(), a.sin()) * rr * u;
            draw_line(prev.x, prev.y, pt.x, pt.y, 2.0, Color::new(0.45, 0.65, 1.0, 0.75 * (1.0 - u * 0.5)));
            prev = pt;
        }
    }
    draw_circle_lines(c.x, c.y, rr, 1.5, Color::new(0.75, 0.88, 1.0, 0.7));
}

/// Stillness: a faint rippling distortion around Kuon, brighter while the ward is ready.
fn draw_stillness(world: &World, t: f32) {
    let p = world.player.pos;
    let r = sim::content::STILLNESS_RADIUS;
    let ready = world.ward_cd <= 0.0;
    let segs = 40;
    for k in 0..segs {
        let a0 = k as f32 / segs as f32 * std::f32::consts::TAU;
        let a1 = (k + 1) as f32 / segs as f32 * std::f32::consts::TAU;
        let w0 = r + (a0 * 5.0 + t * 3.0).sin() * 2.5;
        let w1 = r + (a1 * 5.0 + t * 3.0).sin() * 2.5;
        draw_line(p.x + a0.cos() * w0, p.y - 6.0 + a0.sin() * w0, p.x + a1.cos() * w1, p.y - 6.0 + a1.sin() * w1, 1.5, Color::new(0.55, 0.8, 1.0, 0.22));
    }
    draw_circle(p.x, p.y - 6.0, r, Color::new(0.4, 0.7, 1.0, 0.04));
    if ready {
        // The ward: a slow hexagon of light.
        let rot = t * 0.8;
        for k in 0..6 {
            let a0 = rot + k as f32 * std::f32::consts::FRAC_PI_3;
            let a1 = a0 + std::f32::consts::FRAC_PI_3;
            draw_line(p.x + a0.cos() * 20.0, p.y - 8.0 + a0.sin() * 20.0, p.x + a1.cos() * 20.0, p.y - 8.0 + a1.sin() * 20.0, 1.5, Color::new(0.8, 0.92, 1.0, 0.55));
        }
    }
}

/// Endless Firmament: darkens the world, a starfield and geometric rings fill the domain,
/// and the finale's purple sphere gathers above Kuon.
fn draw_domain(world: &World, t: f32) {
    let c = world.domain_center;
    let p = world.player.pos;
    let left = world.player.form;
    let open = ((sim::content::DOMAIN_DURATION - left) / 0.6).min(1.0);
    let r = world.domain_radius() * open;
    // Darken everything (a very large rect around the player).
    draw_rectangle(p.x - 2000.0, p.y - 2000.0, 4000.0, 4000.0, Color::new(0.0, 0.0, 0.04, 0.45 * open));
    draw_circle(c.x, c.y, r, Color::new(0.02, 0.03, 0.12, 0.65));
    // Stars, deterministic per index.
    for k in 0..140 {
        let a = k as f32 * 2.399;
        let d = r * ((k as f32 * 0.618).fract()).sqrt();
        let x = c.x + a.cos() * d;
        let y = c.y + a.sin() * d;
        let tw = 0.5 + 0.5 * (t * 3.0 + k as f32).sin();
        let s = if k % 9 == 0 { 2.0 } else { 1.0 };
        draw_rectangle(x, y, s, s, Color::new(0.85, 0.92, 1.0, 0.4 + 0.6 * tw));
    }
    // Concentric rings and a rotating hexagram.
    for k in 1..=4 {
        draw_circle_lines(c.x, c.y, r * k as f32 / 4.0, 1.0, Color::new(0.5, 0.75, 1.0, 0.25));
    }
    for tri in 0..2 {
        let rot = t * 0.3 * if tri == 0 { 1.0 } else { -1.0 } + tri as f32 * std::f32::consts::PI / 3.0;
        for k in 0..3 {
            let a0 = rot + k as f32 * std::f32::consts::TAU / 3.0;
            let a1 = a0 + std::f32::consts::TAU / 3.0;
            draw_line(c.x + a0.cos() * r * 0.9, c.y + a0.sin() * r * 0.9, c.x + a1.cos() * r * 0.9, c.y + a1.sin() * r * 0.9, 1.5, Color::new(0.65, 0.85, 1.0, 0.35));
        }
    }
    draw_circle_lines(c.x, c.y, r, 3.0, Color::new(0.75, 0.9, 1.0, 0.8));
    // Finale: the purple sphere gathering above Kuon.
    if left <= sim::content::DOMAIN_FINALE_CHARGE {
        let k = 1.0 - left / sim::content::DOMAIN_FINALE_CHARGE;
        let s = 8.0 + 34.0 * k;
        draw_circle(p.x, p.y - 40.0, s * 1.6, Color::new(0.5, 0.2, 1.0, 0.25));
        draw_circle(p.x, p.y - 40.0, s, Color::new(0.55, 0.25, 1.0, 0.95));
        draw_circle(p.x, p.y - 40.0, s * 0.7, Color::new(0.05, 0.02, 0.12, 1.0));
        draw_circle_lines(p.x, p.y - 40.0, s, 2.0, Color::new(0.9, 0.8, 1.0, 1.0));
    }
}

/// Star sparkles around Kuon; a constellation storm inside his domain.
fn draw_star_motes(p: sim::Vec2, t: f32, domain: bool) {
    let n = if domain { 24 } else { 8 };
    for k in 0..n {
        let kf = k as f32;
        let a = t * (0.5 + 0.1 * (k % 4) as f32) + kf * std::f32::consts::TAU / n as f32;
        let r = if domain { 18.0 + (k % 5) as f32 * 6.0 } else { 14.0 };
        let x = p.x + a.cos() * r;
        let y = p.y - 10.0 + a.sin() * r * 0.55;
        let tw = 0.5 + 0.5 * (t * 5.0 + kf).sin();
        draw_rectangle(x, y, 2.0, 2.0, Color::new(0.8, 0.92, 1.0, 0.4 + 0.5 * tw));
    }
}

/// A yin-yang orb: white and dark halves, each with a dot of the other.
pub fn draw_yinyang(c: Vec2, r: f32, rot: f32) {
    let dark = Color::new(0.1, 0.08, 0.14, 1.0);
    draw_circle(c.x, c.y, r + 1.5, Color::new(0.85, 0.15, 0.2, 1.0));
    draw_circle(c.x, c.y, r, WHITE);
    // Dark half (a fan of triangles), then the two lobes.
    let segs = 16;
    for k in 0..segs {
        let a0 = rot + std::f32::consts::FRAC_PI_2 + k as f32 / segs as f32 * std::f32::consts::PI;
        let a1 = rot + std::f32::consts::FRAC_PI_2 + (k + 1) as f32 / segs as f32 * std::f32::consts::PI;
        draw_triangle(c, c + vec2(a0.cos(), a0.sin()) * r, c + vec2(a1.cos(), a1.sin()) * r, dark);
    }
    let up = vec2((rot - std::f32::consts::FRAC_PI_2).cos(), (rot - std::f32::consts::FRAC_PI_2).sin()) * (r * 0.5);
    draw_circle(c.x + up.x, c.y + up.y, r * 0.5, dark);
    draw_circle(c.x - up.x, c.y - up.y, r * 0.5, WHITE);
    draw_circle(c.x + up.x, c.y + up.y, r * 0.16, WHITE);
    draw_circle(c.x - up.x, c.y - up.y, r * 0.16, dark);
}

/// Divine Purification: a turning magic circle, giant yin-yang orbs and floating ofuda.
fn draw_divine(world: &World, t: f32) {
    let p = world.player.pos;
    let left = world.player.form;
    let open = ((sim::content::DIVINE_DURATION - left) / 0.5).min(1.0);
    let r = 90.0 * open;
    let c = vec2(p.x, p.y + 2.0);
    draw_circle(c.x, c.y, r, Color::new(1.0, 0.85, 0.6, 0.08));
    for k in 0..3 {
        draw_circle_lines(c.x, c.y, r * (0.45 + 0.27 * k as f32), 1.5, Color::new(1.0, 0.8, 0.5, 0.6));
    }
    // Rotating rune ticks and a pentagram.
    for k in 0..24 {
        let a = t * 0.6 + k as f32 / 24.0 * std::f32::consts::TAU;
        let (r0, r1) = (r * 0.9, r * 0.98);
        draw_line(c.x + a.cos() * r0, c.y + a.sin() * r0 * 0.5, c.x + a.cos() * r1, c.y + a.sin() * r1 * 0.5, 2.0, Color::new(1.0, 0.4, 0.45, 0.7));
    }
    for k in 0..5 {
        let a0 = -t * 0.4 + k as f32 * std::f32::consts::TAU / 5.0;
        let a1 = a0 + 2.0 * std::f32::consts::TAU / 5.0;
        draw_line(c.x + a0.cos() * r * 0.7, c.y + a0.sin() * r * 0.35, c.x + a1.cos() * r * 0.7, c.y + a1.sin() * r * 0.35, 1.5, Color::new(1.0, 0.9, 0.6, 0.7));
    }
    // Giant orbiting yin-yang orbs.
    for k in 0..4 {
        let a = t * 1.4 + k as f32 * std::f32::consts::FRAC_PI_2;
        let o = vec2(p.x + a.cos() * 52.0, p.y - 10.0 + a.sin() * 30.0);
        draw_yinyang(o, 11.0, t * 5.0 + k as f32);
    }
    // Floating ofuda.
    for k in 0..18 {
        let a = -t * 0.9 + k as f32 * std::f32::consts::TAU / 18.0;
        let rr = 30.0 + (k % 3) as f32 * 9.0;
        let x = p.x + a.cos() * rr;
        let y = p.y - 12.0 + a.sin() * rr * 0.6 + (t * 3.0 + k as f32).sin() * 3.0;
        draw_rectangle(x - 1.5, y - 3.0, 3.0, 6.0, Color::new(1.0, 0.97, 0.9, 0.9));
        draw_rectangle(x - 1.5, y - 1.0, 3.0, 1.0, Color::new(0.9, 0.2, 0.25, 0.9));
    }
    // Final Purification charging: the circle swells and brightens.
    if left <= sim::content::DIVINE_FINALE_CHARGE {
        let k = 1.0 - left / sim::content::DIVINE_FINALE_CHARGE;
        let fr = world.reach_cap(sim::content::DIVINE_FINALE_RADIUS) * (0.2 + 0.8 * k);
        draw_circle(c.x, c.y, fr, Color::new(1.0, 0.85, 0.6, 0.08 + 0.12 * k));
        draw_circle_lines(c.x, c.y, fr, 3.0, Color::new(1.0, 0.45, 0.5, 0.5 + 0.5 * k));
    }
    let _ = world;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boss_banner_survives_a_leader_in_the_same_batch() {
        let mut fx = Fx::new();
        fx.absorb(&[Event::BossSpawned(EnemyKind::GreatOni), Event::LeaderSpawned { kind: EnemyKind::Oni }], sim::Vec2::ZERO, 192.0);
        assert!(fx.banner.as_ref().is_some_and(|(t, _)| t.contains("APPROACHES")), "{:?}", fx.banner);
    }

    #[test]
    fn every_enemy_has_its_own_sprite() {
        use EnemyKind::*;
        // GreatOni is the oni texture drawn at double size with a tint, so it has no art path either.
        let own_texture = [Wisp, Kasa, Chochin, Nopperabo, NightLord, Oni, GreatOni];
        for k in sim::content::ALL_ENEMIES {
            if !own_texture.contains(&k) {
                assert!(world_enemy_src(k).is_some(), "{k:?} falls back to the oni sprite");
            }
        }
    }
}
