//! Sound engine: plays synthesised SFX (with a per-sound limiter) and looping music.
//! Backed by macroquad audio in the browser (and natively with `--features native-audio`);
//! otherwise every call is a silent no-op.

use sim::content::{EnemyKind, WeaponKind};
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
        for ev in events {
            let s = match *ev {
                Event::Hit { .. } => Sfx::Hit,
                Event::Kill { kind, .. } => match kind {
                    EnemyKind::GreatOni | EnemyKind::NightLord => Sfx::Boom,
                    EnemyKind::Oni => Sfx::BigPop,
                    _ => Sfx::Pop,
                },
                Event::PlayerHurt => Sfx::Hurt,
                Event::LevelUp => Sfx::LevelUp,
                Event::SpecialUsed { .. } => Sfx::Special,
                Event::BossSpawned => Sfx::Gong,
                Event::BossTelegraph => Sfx::Rumble,
                Event::Awakened(_) => Sfx::Awaken,
                Event::Bolt { .. } => Sfx::Zap,
                Event::Slash { .. } => Sfx::Whoosh,
                Event::Impact { .. } => Sfx::Thud,
                Event::ShrineAppeared => Sfx::Bell,
                Event::ShrineChosen(_) => Sfx::Chime,
                Event::NightLordSpawned => Sfx::GongDeep,
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
                Event::Fired(WeaponKind::SoulTether) => Sfx::Chain,
                Event::Fired(WeaponKind::IaiFlash) => Sfx::IaiSfx,
                Event::Fired(_) => continue,
                Event::Burst { .. } => Sfx::FoxBurst,
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
            };
            self.play(s);
        }
    }
}
