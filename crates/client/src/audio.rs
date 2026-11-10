//! Sound engine: plays synthesised SFX (with a per-sound limiter) and looping music.
//! Backed by macroquad audio in the browser (and natively with `--features native-audio`);
//! otherwise every call is a silent no-op.

use sim::content::{Rank, WeaponKind};
use sim::Event;

use crate::synth::{self, Sfx, Track, ALL_SFX};

#[cfg(any(target_arch = "wasm32", feature = "native-audio"))]
mod backend {
    use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};

    pub struct Handle(Sound);

    pub async fn load(wav: &[u8]) -> Option<Handle> {
        load_sound_from_bytes(wav).await.ok().map(Handle)
    }
    pub fn play(h: &Handle, volume: f32, looped: bool) {
        play_sound(&h.0, PlaySoundParams { looped, volume });
    }
    pub fn stop(h: &Handle) {
        stop_sound(&h.0);
    }
    pub fn volume(h: &Handle, v: f32) {
        set_sound_volume(&h.0, v);
    }
}

#[cfg(not(any(target_arch = "wasm32", feature = "native-audio")))]
mod backend {
    pub struct Handle;

    pub async fn load(_wav: &[u8]) -> Option<Handle> {
        Some(Handle)
    }
    pub fn play(_: &Handle, _: f32, _: bool) {}
    pub fn stop(_: &Handle) {}
    pub fn volume(_: &Handle, _: f32) {}
}

/// Per-sound mixing: base volume and the minimum gap between two plays (the limiter).
fn tuning(s: Sfx) -> (f32, f64) {
    match s {
        Sfx::Hit => (0.25, 0.05),
        Sfx::Pop => (0.35, 0.035),
        Sfx::Gem => (0.35, 0.04),
        Sfx::Coin => (0.5, 0.05),
        Sfx::Paper => (0.25, 0.12),
        Sfx::Puff => (0.3, 0.1),
        Sfx::Zap => (0.35, 0.07),
        Sfx::Whoosh => (0.35, 0.06),
        Sfx::Thud => (0.4, 0.06),
        Sfx::Drum => (0.45, 0.12),
        Sfx::Fwoosh => (0.3, 0.15),
        Sfx::Charge => (0.3, 0.15),
        Sfx::FoxBurst => (0.4, 0.06),
        Sfx::Yip => (0.3, 0.1),
        Sfx::Spin => (0.3, 0.2),
        Sfx::Volley => (0.45, 0.2),
        Sfx::Poof => (0.4, 0.08),
        Sfx::Cleave => (0.45, 0.08),
        Sfx::Chain => (0.35, 0.15),
        Sfx::IaiSfx => (0.45, 0.1),
        Sfx::Pulse => (0.25, 0.12),
        Sfx::LashSfx => (0.3, 0.06),
        Sfx::Chomp => (0.4, 0.2),
        Sfx::Crunch => (0.4, 0.07),
        Sfx::Rise => (0.5, 0.2),
        Sfx::Thunder => (0.55, 0.12),
        Sfx::Meteor => (0.8, 0.2),
        Sfx::BeamSfx => (0.4, 0.12),
        Sfx::DarkBurst => (0.45, 0.08),
        Sfx::Warble => (0.35, 0.12),
        Sfx::Hurt => (0.6, 0.12),
        Sfx::Click => (0.5, 0.03),
        _ => (0.7, 0.05),
    }
}

/// Most SFX that may start in one frame (beyond this, quieter ones are dropped).
const MAX_PER_FRAME: usize = 8;

pub struct Audio {
    sfx: Vec<(Sfx, backend::Handle)>,
    music: Vec<(Track, backend::Handle)>,
    current: Option<Track>,
    next_ok: Vec<f64>,
    this_frame: usize,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

const MUSIC_BASE: f32 = 0.55;

impl Audio {
    /// Synthesises and loads everything (a fraction of a second).
    pub async fn load(music_volume: f32, sfx_volume: f32) -> Audio {
        let mut sfx = Vec::new();
        for s in ALL_SFX {
            if let Some(h) = backend::load(&synth::sfx(s).wav()).await {
                sfx.push((s, h));
            }
        }
        let mut music = Vec::new();
        for t in [Track::Menu, Track::Battle, Track::Boss] {
            if let Some(h) = backend::load(&synth::music(t).wav()).await {
                music.push((t, h));
            }
        }
        Audio { sfx, music, current: None, next_ok: vec![0.0; ALL_SFX.len()], this_frame: 0, music_volume, sfx_volume }
    }

    /// Synthesises and loads a track if it isn't loaded yet (world themes load on demand).
    pub async fn prepare(&mut self, t: Track) {
        if self.music.iter().any(|(x, _)| *x == t) {
            return;
        }
        if let Some(h) = backend::load(&synth::music(t).wav()).await {
            self.music.push((t, h));
        }
    }

    /// Call once per frame before any `play`.
    pub fn begin_frame(&mut self) {
        self.this_frame = 0;
    }

    pub fn play(&mut self, s: Sfx) {
        let now = macroquad::time::get_time();
        let i = ALL_SFX.iter().position(|x| *x == s).unwrap();
        let (vol, gap) = tuning(s);
        // The per-frame cap only drops the quiet, spammy sounds; big moments always play.
        if now < self.next_ok[i] || (self.this_frame >= MAX_PER_FRAME && vol < 0.5) || self.sfx_volume <= 0.0 {
            return;
        }
        self.next_ok[i] = now + gap;
        self.this_frame += 1;
        if let Some((_, h)) = self.sfx.iter().find(|(x, _)| *x == s) {
            backend::play(h, vol * self.sfx_volume, false);
        }
    }

    /// Switches the looping music (None = silence). No-op if already playing that track.
    pub fn set_music(&mut self, t: Option<Track>) {
        if self.current == t {
            return;
        }
        if let Some(cur) = self.current.and_then(|c| self.music.iter().find(|(x, _)| *x == c)) {
            backend::stop(&cur.1);
        }
        self.current = t;
        if let Some((_, h)) = t.and_then(|t| self.music.iter().find(|(x, _)| *x == t)) {
            backend::play(h, MUSIC_BASE * self.music_volume, true);
        }
    }

    pub fn apply_music_volume(&self) {
        if let Some((_, h)) = self.current.and_then(|c| self.music.iter().find(|(x, _)| *x == c)) {
            backend::volume(h, MUSIC_BASE * self.music_volume);
        }
    }

    /// Plays the sounds for one sim step's events.
    pub fn absorb(&mut self, events: &[Event]) {
        let mut gonged = false;
        for ev in events {
            let s = match *ev {
                Event::Hit { .. } => Sfx::Hit,
                Event::Kill { kind, .. } => match kind.rank() {
                    Rank::Mini | Rank::Boss | Rank::Final => Sfx::Boom,
                    Rank::Elite => Sfx::BigPop,
                    Rank::Fodder => Sfx::Pop,
                },
                Event::BossPhase { .. } => Sfx::GongDeep,
                Event::Hazard { kind, .. } => match kind {
                    0 => Sfx::Meteor,
                    1 => Sfx::Thunder,
                    _ => Sfx::GongDeep,
                },
                Event::Curtain => Sfx::Chime,
                Event::GearDrop { rarity, .. } => if rarity >= sim::gear::Rarity::Legendary { Sfx::Awaken } else if rarity >= sim::gear::Rarity::Epic { Sfx::Chime } else { Sfx::Coin },
                Event::PlayerHurt => Sfx::Hurt,
                Event::LevelUp => Sfx::LevelUp,
                Event::SpecialUsed { .. } => Sfx::Special,
                Event::BossSpawned(_) | Event::LeaderSpawned { .. } | Event::EliteWave | Event::RoundStarted { headline: Some(_), .. } => Sfx::Gong,
                Event::BossTelegraph => Sfx::Rumble,
                Event::Awakened(_) => Sfx::Awaken,
                Event::Bolt { .. } => Sfx::Zap,
                Event::Slash { .. } => Sfx::Whoosh,
                Event::Impact { .. } => Sfx::Thud,
                Event::ShrineAppeared => Sfx::Bell,
                Event::ShrineChosen(_) => Sfx::Chime,
                Event::NightLordSpawned(_) => Sfx::GongDeep,
                Event::Revived => Sfx::Revive,
                Event::SenPicked { .. } => Sfx::Coin,
                Event::Blink { .. } => Sfx::Warble,
                Event::GemsPicked => Sfx::Gem,
                Event::EnemyShot => Sfx::Puff,
                Event::Fired(WeaponKind::Ofuda) => Sfx::Paper,
                Event::Fired(WeaponKind::Taiko) => Sfx::Drum,
                Event::Fired(WeaponKind::FlamePool) => Sfx::Fwoosh,
                Event::Fired(WeaponKind::FoxfireOrb) => Sfx::Charge,
                Event::Fired(WeaponKind::Familiars) => Sfx::Yip,
                Event::Fired(WeaponKind::GaleShuriken) => Sfx::Spin,
                Event::Fired(WeaponKind::GraveCleaver) => Sfx::Cleave,
                Event::Fired(WeaponKind::GravityWell) => Sfx::Warble,
                Event::Fired(WeaponKind::SpiritSeal) => Sfx::Chime,
                Event::Fired(WeaponKind::PurificationWave) => Sfx::Bell,
                Event::Fired(WeaponKind::RepulsionStar) => Sfx::Zap,
                Event::Fired(WeaponKind::VoidLance) => Sfx::BeamSfx,
                Event::Fired(WeaponKind::SoulTether) => Sfx::Chain,
                Event::Fired(WeaponKind::IaiFlash) => Sfx::IaiSfx,
                Event::Fired(_) | Event::RoundStarted { .. } => continue,
                Event::Burst { style, .. } => match style {
                    sim::world::style::METEOR => Sfx::Meteor,
                    sim::world::style::SKYBOLT => Sfx::Thunder,
                    sim::world::style::QUAKE => Sfx::Drum,
                    sim::world::style::SEAL => Sfx::Pop,
                    sim::world::style::SPINE => Sfx::Crunch,
                    sim::world::style::FRENZY => Sfx::Boom,
                    sim::world::style::PURIFY => Sfx::Boom,
                    sim::world::style::YINYANG => Sfx::Thud,
                    _ => Sfx::FoxBurst,
                },
                Event::Signature(k) => match k {
                    WeaponKind::MoonOrbs => Sfx::Chime,
                    WeaponKind::GaleShuriken => Sfx::Spin,
                    WeaponKind::GraveCleaver => Sfx::Cleave,
                    WeaponKind::IaiFlash => Sfx::IaiSfx,
                    WeaponKind::StarFall => Sfx::Fwoosh,
                    WeaponKind::FoxfireOrb => Sfx::Charge,
                    WeaponKind::Taiko => Sfx::Rumble,
                    _ => continue,
                },
                Event::FoxAwakened { .. } => Sfx::FoxAwaken,
                Event::FoxVolley => Sfx::Volley,
                Event::FoxFaded => Sfx::FoxFade,
                Event::CloneSummoned { .. } | Event::ClonePopped { .. } => Sfx::Poof,
                Event::CloneStrike { .. } => Sfx::Whoosh,
                Event::HannyaAwakened { .. } => Sfx::Roar,
                Event::Beam { .. } => Sfx::BeamSfx,
                Event::ShadowStep { .. } => Sfx::Step,
                Event::AfterimageBlast { .. } => Sfx::DarkBurst,
                Event::FinaleCharge { .. } => Sfx::Charge,
                Event::FinaleBlast { .. } => Sfx::Boom,
                Event::Iai { .. } => continue, // the Fired cue covers the cut
                Event::LanternPulse { .. } => Sfx::Pulse,
                Event::Lash { .. } => Sfx::LashSfx,
                Event::WellCollapse { .. } => Sfx::DarkBurst,
                Event::Warded => Sfx::Chime,
                Event::BarrierBlock => Sfx::Chime,
                Event::DivineAwakened { .. } => Sfx::GongDeep,
                Event::DivineCharge { .. } => Sfx::Charge,
                Event::DomainOpened { .. } => Sfx::GongDeep,
                Event::VoidCharge { .. } => Sfx::Charge,
                Event::VoidMaximum { .. } => Sfx::Meteor,
                Event::PickupSpawned { .. } => Sfx::Bell,
                Event::PickupTaken { kind, .. } => match kind {
                    sim::content::PickupKind::ExorcismSeal => Sfx::Special,
                    sim::content::PickupKind::SenPurse => Sfx::Coin,
                    sim::content::PickupKind::Hourglass => Sfx::Warble,
                    _ => Sfx::LevelUp,
                },
                Event::PickupStolen { .. } => Sfx::Card,
                Event::LeaderFell { .. } => Sfx::ChestBurst,
                Event::HazardWarn { .. } => Sfx::Click,
                Event::Devour { .. } => Sfx::Chomp,
                Event::MukadeAwakened { .. } => Sfx::Roar,
            };
            if matches!(s, Sfx::Gong) {
                if gonged {
                    continue;
                }
                gonged = true;
            }
            self.play(s);
        }
    }
}
