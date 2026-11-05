//! Tiny offline synthesiser: every sound effect and music loop is generated in code at startup
//! (sfxr-style), then encoded as WAV for the audio backend. No audio files ship with the game.

pub const SR: f32 = 22_050.0;

#[derive(Clone, Copy)]
pub enum Osc {
    Sine,
    Tri,
    Saw,
    /// Pulse with the given duty cycle.
    Square(f32),
    Noise,
}

/// One synthesised note: oscillator, pitch slide, envelope and a one-pole low-pass.
#[derive(Clone, Copy)]
pub struct Note {
    pub osc: Osc,
    pub freq: f32,
    /// Pitch at the end of the note (exponential slide); equal to `freq` for none.
    pub freq_end: f32,
    pub dur: f32,
    pub vol: f32,
    pub attack: f32,
    /// Decay shape: envelope = (1 - t)^curve after the attack. 1 = linear, higher = plucky.
    pub curve: f32,
    /// Low-pass cutoff in Hz (0 = off).
    pub lowpass: f32,
    /// Vibrato depth (fraction of pitch) and rate (Hz).
    pub vibrato: (f32, f32),
}

impl Note {
    pub fn new(osc: Osc, freq: f32, dur: f32, vol: f32) -> Self {
        Note { osc, freq, freq_end: freq, dur, vol, attack: 0.002, curve: 2.0, lowpass: 0.0, vibrato: (0.0, 0.0) }
    }
    pub fn slide(mut self, to: f32) -> Self {
        self.freq_end = to;
        self
    }
    pub fn env(mut self, attack: f32, curve: f32) -> Self {
        self.attack = attack;
        self.curve = curve;
        self
    }
    pub fn lp(mut self, hz: f32) -> Self {
        self.lowpass = hz;
        self
    }
    pub fn vib(mut self, depth: f32, rate: f32) -> Self {
        self.vibrato = (depth, rate);
        self
    }
}

/// A mono sample buffer. With `wrap`, writes past the end fold back to the start (seamless loops).
pub struct Buf {
    pub s: Vec<f32>,
    wrap: bool,
    seed: u32,
}

impl Buf {
    pub fn new(seconds: f32) -> Self {
        Buf { s: vec![0.0; (seconds * SR) as usize], wrap: false, seed: 0x1234_5678 }
    }
    pub fn looped(seconds: f32) -> Self {
        Buf { wrap: true, ..Buf::new(seconds) }
    }

    fn noise(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Mixes `n` in, starting `at` seconds.
    pub fn add(&mut self, at: f32, n: Note) {
        let len = self.s.len();
        let start = (at * SR) as usize;
        let count = (n.dur * SR) as usize;
        let mut phase = 0.0f32;
        let mut lp = 0.0f32;
        let a = if n.lowpass > 0.0 { 1.0 - (-std::f32::consts::TAU * n.lowpass / SR).exp() } else { 1.0 };
        let ratio = n.freq_end / n.freq;
        for i in 0..count {
            let t = i as f32 / count as f32;
            let mut f = n.freq * ratio.powf(t);
            if n.vibrato.0 > 0.0 {
                f *= 1.0 + n.vibrato.0 * (std::f32::consts::TAU * n.vibrato.1 * i as f32 / SR).sin();
            }
            phase = (phase + f / SR).fract();
            let x = match n.osc {
                Osc::Sine => (phase * std::f32::consts::TAU).sin(),
                Osc::Tri => 1.0 - 4.0 * (phase - 0.5).abs(),
                Osc::Saw => 2.0 * phase - 1.0,
                Osc::Square(d) => {
                    if phase < d {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Osc::Noise => self.noise(),
            };
            lp += a * (x - lp);
            let secs = i as f32 / SR;
            let env = if secs < n.attack { secs / n.attack } else { (1.0 - t).max(0.0).powf(n.curve) };
            let j = start + i;
            let j = if self.wrap { j % len } else if j < len { j } else { break };
            self.s[j] += lp * env * n.vol;
        }
    }

    /// Soft-clips and returns 16-bit mono WAV bytes.
    pub fn wav(&self) -> Vec<u8> {
        let n = self.s.len() as u32;
        let mut out = Vec::with_capacity(44 + n as usize * 2);
        let mut put = |b: &[u8]| out.extend_from_slice(b);
        put(b"RIFF");
        put(&(36 + n * 2).to_le_bytes());
        put(b"WAVEfmt ");
        put(&16u32.to_le_bytes());
        put(&1u16.to_le_bytes()); // PCM
        put(&1u16.to_le_bytes()); // mono
        put(&(SR as u32).to_le_bytes());
        put(&(SR as u32 * 2).to_le_bytes());
        put(&2u16.to_le_bytes());
        put(&16u16.to_le_bytes());
        put(b"data");
        put(&(n * 2).to_le_bytes());
        for &x in &self.s {
            let y = x.tanh();
            out.extend_from_slice(&((y * 32_000.0) as i16).to_le_bytes());
        }
        out
    }
}

pub fn midi(n: f32) -> f32 {
    440.0 * 2f32.powf((n - 69.0) / 12.0)
}

// ───────────────────────── Sound effects ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx {
    Hit,
    Pop,
    BigPop,
    Boom,
    Hurt,
    Gem,
    Coin,
    LevelUp,
    Special,
    Gong,
    GongDeep,
    Rumble,
    Awaken,
    Zap,
    Whoosh,
    Thud,
    Bell,
    Chime,
    Revive,
    Warble,
    Puff,
    Paper,
    Drum,
    Fwoosh,
    Click,
    Buy,
    ChestShake,
    ChestBurst,
    ChestLegendary,
    Card,
    Victory,
    Defeat,
}

pub const ALL_SFX: [Sfx; 32] = [
    Sfx::Hit,
    Sfx::Pop,
    Sfx::BigPop,
    Sfx::Boom,
    Sfx::Hurt,
    Sfx::Gem,
    Sfx::Coin,
    Sfx::LevelUp,
    Sfx::Special,
    Sfx::Gong,
    Sfx::GongDeep,
    Sfx::Rumble,
    Sfx::Awaken,
    Sfx::Zap,
    Sfx::Whoosh,
    Sfx::Thud,
    Sfx::Bell,
    Sfx::Chime,
    Sfx::Revive,
    Sfx::Warble,
    Sfx::Puff,
    Sfx::Paper,
    Sfx::Drum,
    Sfx::Fwoosh,
    Sfx::Click,
    Sfx::Buy,
    Sfx::ChestShake,
    Sfx::ChestBurst,
    Sfx::ChestLegendary,
    Sfx::Card,
    Sfx::Victory,
    Sfx::Defeat,
];

/// Bell-like tone: inharmonic sine partials.
fn bell(b: &mut Buf, at: f32, f: f32, dur: f32, vol: f32) {
    for (mult, v) in [(1.0, 1.0), (2.76, 0.45), (5.40, 0.25), (8.93, 0.12)] {
        b.add(at, Note::new(Osc::Sine, f * mult, dur / mult.sqrt(), vol * v).env(0.002, 3.0));
    }
}

fn taiko(b: &mut Buf, at: f32, vol: f32) {
    b.add(at, Note::new(Osc::Sine, 120.0, 0.35, vol).slide(48.0).env(0.002, 2.5));
    b.add(at, Note::new(Osc::Noise, 1.0, 0.05, vol * 0.4).lp(1800.0).env(0.001, 3.0));
}

pub fn sfx(s: Sfx) -> Buf {
    use Osc::*;
    let mut b;
    match s {
        Sfx::Hit => {
            b = Buf::new(0.04);
            b.add(0.0, Note::new(Noise, 1.0, 0.04, 0.5).lp(5000.0).env(0.001, 4.0));
            b.add(0.0, Note::new(Square(0.5), 900.0, 0.03, 0.15).slide(500.0));
        }
        Sfx::Pop => {
            b = Buf::new(0.08);
            b.add(0.0, Note::new(Square(0.3), 620.0, 0.07, 0.3).slide(180.0).lp(3000.0));
            b.add(0.0, Note::new(Noise, 1.0, 0.04, 0.2).lp(2500.0).env(0.001, 3.0));
        }
        Sfx::BigPop => {
            b = Buf::new(0.35);
            b.add(0.0, Note::new(Square(0.4), 300.0, 0.3, 0.45).slide(60.0).lp(1800.0));
            b.add(0.0, Note::new(Noise, 1.0, 0.25, 0.4).lp(1200.0).env(0.001, 2.0));
        }
        Sfx::Boom => {
            b = Buf::new(1.4);
            b.add(0.0, Note::new(Sine, 90.0, 1.4, 0.6).slide(30.0).env(0.002, 2.0));
            b.add(0.0, Note::new(Noise, 1.0, 1.0, 0.6).lp(700.0).env(0.002, 2.0));
            bell(&mut b, 0.05, 520.0, 1.2, 0.2);
        }
        Sfx::Hurt => {
            b = Buf::new(0.22);
            b.add(0.0, Note::new(Square(0.5), 220.0, 0.2, 0.4).slide(90.0).lp(1500.0));
            b.add(0.0, Note::new(Noise, 1.0, 0.12, 0.35).lp(2000.0));
        }
        Sfx::Gem => {
            b = Buf::new(0.08);
            b.add(0.0, Note::new(Sine, 1320.0, 0.08, 0.35).slide(1980.0).env(0.002, 1.5));
            b.add(0.0, Note::new(Tri, 2640.0, 0.05, 0.1));
        }
        Sfx::Coin => {
            b = Buf::new(0.3);
            b.add(0.0, Note::new(Square(0.25), midi(88.0), 0.06, 0.25).env(0.001, 0.5).lp(6000.0));
            b.add(0.06, Note::new(Square(0.25), midi(93.0), 0.24, 0.25).env(0.001, 2.0).lp(6000.0));
        }
        Sfx::LevelUp => {
            b = Buf::new(0.7);
            for (i, n) in [72.0, 76.0, 79.0, 84.0, 88.0].iter().enumerate() {
                b.add(i as f32 * 0.07, Note::new(Square(0.25), midi(*n), 0.3, 0.22).lp(5000.0));
                b.add(i as f32 * 0.07, Note::new(Tri, midi(*n + 12.0), 0.35, 0.15));
            }
        }
        Sfx::Special => {
            b = Buf::new(1.0);
            b.add(0.0, Note::new(Noise, 1.0, 0.8, 0.5).lp(4000.0).env(0.05, 2.0));
            b.add(0.0, Note::new(Sine, 300.0, 0.9, 0.4).slide(1200.0).env(0.02, 1.5).vib(0.02, 9.0));
            bell(&mut b, 0.08, 880.0, 0.9, 0.3);
        }
        Sfx::Gong => {
            b = Buf::new(2.6);
            bell(&mut b, 0.0, 130.0, 2.6, 0.42);
            b.add(0.0, Note::new(Sine, 65.0, 2.2, 0.35).env(0.01, 2.0).vib(0.01, 3.0));
        }
        Sfx::GongDeep => {
            b = Buf::new(3.6);
            bell(&mut b, 0.0, 87.0, 3.6, 0.45);
            b.add(0.0, Note::new(Saw, 43.5, 3.2, 0.35).lp(300.0).env(0.3, 1.5));
            taiko(&mut b, 0.0, 0.6);
            taiko(&mut b, 0.45, 0.5);
        }
        Sfx::Rumble => {
            b = Buf::new(0.5);
            b.add(0.0, Note::new(Noise, 1.0, 0.5, 1.4).lp(250.0).env(0.05, 1.5));
        }
        Sfx::Awaken => {
            b = Buf::new(1.8);
            for n in [60.0, 64.0, 67.0, 72.0, 76.0] {
                b.add(0.0, Note::new(Saw, midi(n), 1.6, 0.12).lp(2500.0).env(0.25, 1.5).vib(0.004, 5.0));
            }
            bell(&mut b, 0.3, midi(84.0), 1.4, 0.35);
        }
        Sfx::Zap => {
            b = Buf::new(0.12);
            b.add(0.0, Note::new(Noise, 1.0, 0.12, 0.45).lp(7000.0).env(0.001, 2.0));
            b.add(0.0, Note::new(Square(0.5), 1600.0, 0.1, 0.15).slide(400.0));
        }
        Sfx::Whoosh => {
            b = Buf::new(0.18);
            b.add(0.0, Note::new(Noise, 1.0, 0.18, 0.4).lp(2500.0).env(0.04, 1.5));
        }
        Sfx::Thud => {
            b = Buf::new(0.3);
            b.add(0.0, Note::new(Sine, 160.0, 0.28, 0.6).slide(50.0));
            b.add(0.0, Note::new(Noise, 1.0, 0.1, 0.3).lp(1500.0));
            b.add(0.0, Note::new(Sine, 1760.0, 0.15, 0.08));
        }
        Sfx::Bell => {
            b = Buf::new(1.6);
            bell(&mut b, 0.0, midi(79.0), 1.6, 0.45);
            bell(&mut b, 0.25, midi(86.0), 1.3, 0.3);
        }
        Sfx::Chime => {
            b = Buf::new(0.9);
            for (i, n) in [79.0, 83.0, 86.0].iter().enumerate() {
                bell(&mut b, i as f32 * 0.08, midi(*n), 0.8, 0.3);
            }
        }
        Sfx::Revive => {
            b = Buf::new(1.2);
            for (i, n) in [67.0, 71.0, 74.0, 79.0, 83.0, 86.0].iter().enumerate() {
                b.add(i as f32 * 0.09, Note::new(Tri, midi(*n), 0.5, 0.25));
            }
            b.add(0.0, Note::new(Sine, 200.0, 1.1, 0.3).slide(800.0).env(0.1, 1.5));
        }
        Sfx::Warble => {
            b = Buf::new(0.25);
            b.add(0.0, Note::new(Sine, 500.0, 0.25, 0.3).slide(250.0).vib(0.15, 22.0).env(0.01, 1.5));
        }
        Sfx::Puff => {
            b = Buf::new(0.15);
            b.add(0.0, Note::new(Noise, 1.0, 0.14, 0.35).lp(900.0).env(0.005, 2.0));
            b.add(0.0, Note::new(Sine, 260.0, 0.1, 0.2).slide(140.0));
        }
        Sfx::Paper => {
            b = Buf::new(0.07);
            b.add(0.0, Note::new(Noise, 1.0, 0.07, 0.25).lp(6000.0).env(0.01, 2.0));
        }
        Sfx::Drum => {
            b = Buf::new(0.4);
            taiko(&mut b, 0.0, 0.8);
        }
        Sfx::Fwoosh => {
            b = Buf::new(0.4);
            b.add(0.0, Note::new(Noise, 1.0, 0.4, 0.45).lp(1200.0).env(0.06, 1.5));
            b.add(0.0, Note::new(Saw, 90.0, 0.35, 0.15).lp(400.0).env(0.05, 1.5));
        }
        Sfx::Click => {
            b = Buf::new(0.05);
            b.add(0.0, Note::new(Square(0.5), 1400.0, 0.03, 0.2).slide(1000.0).lp(5000.0));
        }
        Sfx::Buy => {
            b = Buf::new(0.6);
            for (i, n) in [84.0, 88.0, 91.0].iter().enumerate() {
                b.add(i as f32 * 0.06, Note::new(Square(0.25), midi(*n), 0.25, 0.2).lp(6000.0));
            }
            bell(&mut b, 0.18, midi(96.0), 0.4, 0.2);
        }
        Sfx::ChestShake => {
            b = Buf::new(0.9);
            for k in 0..14 {
                let t = k as f32 * 0.062;
                b.add(t, Note::new(Noise, 1.0, 0.04, 0.15 + 0.03 * k as f32).lp(1500.0 + 300.0 * k as f32));
            }
            b.add(0.0, Note::new(Sine, 200.0, 0.9, 0.2).slide(700.0).env(0.3, 0.5));
        }
        Sfx::ChestBurst => {
            b = Buf::new(1.0);
            b.add(0.0, Note::new(Noise, 1.0, 0.3, 0.5).lp(5000.0).env(0.001, 2.0));
            for (i, n) in [72.0, 79.0, 84.0].iter().enumerate() {
                bell(&mut b, i as f32 * 0.05, midi(*n), 0.9, 0.3);
            }
        }
        Sfx::ChestLegendary => {
            b = Buf::new(2.0);
            b.add(0.0, Note::new(Noise, 1.0, 0.4, 0.5).lp(6000.0).env(0.001, 2.0));
            for (i, n) in [72.0, 76.0, 79.0, 84.0, 88.0, 91.0, 96.0].iter().enumerate() {
                bell(&mut b, i as f32 * 0.07, midi(*n), 1.4, 0.28);
            }
            for n in [60.0, 67.0, 72.0] {
                b.add(0.3, Note::new(Saw, midi(n), 1.6, 0.12).lp(3000.0).env(0.2, 1.5));
            }
        }
        Sfx::Card => {
            b = Buf::new(0.3);
            b.add(0.0, Note::new(Tri, midi(84.0), 0.25, 0.3).slide(midi(91.0)).env(0.002, 2.0));
        }
        Sfx::Victory => {
            b = Buf::new(3.0);
            let tune = [(0.0, 72.0), (0.18, 74.0), (0.36, 76.0), (0.54, 79.0), (0.9, 81.0), (1.08, 79.0), (1.26, 84.0)];
            for (t, n) in tune {
                b.add(t, Note::new(Square(0.25), midi(n), 0.5, 0.2).lp(4000.0));
                b.add(t, Note::new(Tri, midi(n - 12.0), 0.5, 0.2));
            }
            for n in [60.0, 64.0, 67.0, 72.0] {
                b.add(1.26, Note::new(Saw, midi(n), 1.7, 0.1).lp(2500.0).env(0.05, 1.5));
            }
            taiko(&mut b, 0.0, 0.6);
            taiko(&mut b, 1.26, 0.8);
        }
        Sfx::Defeat => {
            b = Buf::new(2.4);
            for (i, n) in [69.0, 68.0, 65.0, 62.0].iter().enumerate() {
                b.add(i as f32 * 0.35, Note::new(Tri, midi(*n), 0.7, 0.3).vib(0.01, 5.0));
            }
            bell(&mut b, 1.05, midi(50.0), 1.3, 0.4);
        }
    }
    b
}

// ───────────────────────── Music ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Track {
    Menu,
    Battle,
    Boss,
}

struct SongSpec {
    bpm: f32,
    /// Scale as semitone offsets within an octave.
    scale: &'static [i32],
    root: f32,
    bars: usize,
    seed: u32,
    /// Chord roots (scale degrees), one per two bars, cycled.
    chords: &'static [i32],
    drums: u8,
    pad: bool,
}

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn range(&mut self, n: u32) -> i32 {
        (self.next() % n) as i32
    }
}

fn degree(spec: &SongSpec, d: i32) -> f32 {
    let n = spec.scale.len() as i32;
    let oct = d.div_euclid(n);
    spec.root + (oct * 12 + spec.scale[d.rem_euclid(n) as usize]) as f32
}

/// A two-bar motif: (eighth-note slot, scale degree relative to the chord, length in eighths).
fn motif(rng: &mut Rng, density: u32) -> Vec<(usize, i32, usize)> {
    let mut out = Vec::new();
    let mut d = rng.range(5);
    let mut slot = 0;
    while slot < 16 {
        if (rng.next() % 10) < density || slot == 0 || slot == 8 {
            let len = [1, 1, 2, 2, 3][rng.range(5) as usize];
            d = (d + rng.range(5) - 2).clamp(-2, 7);
            out.push((slot, d, len));
            slot += len;
        } else {
            slot += 1;
        }
    }
    out
}

fn song(spec: SongSpec) -> Buf {
    let beat = 60.0 / spec.bpm;
    let eighth = beat / 2.0;
    let bar = beat * 4.0;
    let mut b = Buf::looped(bar * spec.bars as f32);
    let mut rng = Rng(spec.seed);
    let a = motif(&mut rng, 6);
    let a2 = {
        let mut m = a.clone();
        if let Some(last) = m.last_mut() {
            last.1 -= 2; // answer phrase: same rhythm, lands lower
        }
        m
    };
    let bm = motif(&mut rng, 5);
    // Phrase plan per two bars.
    let plan = [&a, &a2, &bm, &a, &a, &a2, &bm, &a2];
    for (p, phrase) in (0..spec.bars / 2).map(|p| (p, plan[p % plan.len()])) {
        let t0 = p as f32 * bar * 2.0;
        let chord = spec.chords[p % spec.chords.len()];
        // Lead: koto-like pluck.
        for &(slot, d, len) in phrase.iter() {
            let f = midi(degree(&spec, chord + d) + 12.0);
            let dur = (len as f32 * eighth + 0.25).min(1.2);
            b.add(t0 + slot as f32 * eighth, Note::new(Osc::Tri, f, dur, 0.16).env(0.003, 3.0).lp(3500.0));
            b.add(t0 + slot as f32 * eighth, Note::new(Osc::Sine, f * 2.0, dur * 0.5, 0.05).env(0.002, 4.0));
        }
        // Bass.
        let root = midi(degree(&spec, chord) - 12.0);
        for k in 0..16 {
            let on = match spec.drums {
                0 => k % 4 == 0,
                _ => k % 2 == 0 || k % 8 == 7,
            };
            if on {
                let f = if spec.drums > 0 && k % 4 == 2 { root * 2.0 } else { root };
                b.add(t0 + k as f32 * eighth, Note::new(Osc::Square(0.25), f, eighth * 1.6, 0.14).lp(700.0).env(0.004, 1.5));
            }
        }
        // Pad: soft sustained triad.
        if spec.pad {
            for (i, d) in [0, 2, 4].iter().enumerate() {
                let f = midi(degree(&spec, chord + d));
                b.add(t0, Note::new(Osc::Saw, f * (1.0 + 0.003 * i as f32), bar * 2.0, 0.045).lp(900.0).env(0.6, 0.6));
            }
        }
        // Drums.
        if spec.drums > 0 {
            for k in 0..16 {
                let t = t0 + k as f32 * eighth;
                if k % 4 == 0 || (spec.drums > 1 && k % 2 == 0) {
                    taiko(&mut b, t, if k % 8 == 0 { 0.55 } else { 0.4 });
                }
                if k == 15 && p % 2 == 1 {
                    taiko(&mut b, t, 0.35);
                    taiko(&mut b, t + eighth / 2.0, 0.3);
                }
                if k % 2 == 1 {
                    b.add(t, Note::new(Osc::Noise, 1.0, 0.04, 0.09).lp(9000.0).env(0.001, 3.0));
                }
            }
        }
    }
    // Even out loudness between tracks (target RMS ~0.1); tanh in `wav` catches peaks.
    let rms = (b.s.iter().map(|x| x * x).sum::<f32>() / b.s.len() as f32).sqrt().max(1e-6);
    let gain = (0.1 / rms).min(3.0);
    for x in &mut b.s {
        *x *= gain;
    }
    b
}

pub fn music(t: Track) -> Buf {
    match t {
        // Yo scale (bright, folk), slow, pad, no drums.
        Track::Menu => song(SongSpec { bpm: 84.0, scale: &[0, 2, 5, 7, 9], root: 62.0, bars: 16, seed: 0xBEEF_0001, chords: &[0, 3, 1, 4], drums: 0, pad: true }),
        // Minyo scale, festival drums.
        Track::Battle => song(SongSpec { bpm: 132.0, scale: &[0, 3, 5, 7, 10], root: 57.0, bars: 16, seed: 0xC0DE_0007, chords: &[0, 0, 3, 4], drums: 1, pad: false }),
        // In scale (dark), driving drums.
        Track::Boss => song(SongSpec { bpm: 148.0, scale: &[0, 1, 5, 7, 8], root: 50.0, bars: 16, seed: 0xD00D_0042, chords: &[0, 1, 0, 3], drums: 2, pad: true }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sfx_renders_finite_audio() {
        for s in ALL_SFX {
            let b = sfx(s);
            assert!(!b.s.is_empty(), "{s:?} empty");
            assert!(b.s.iter().all(|x| x.is_finite()), "{s:?} has NaN");
            assert!(b.s.iter().any(|x| x.abs() > 0.01), "{s:?} silent");
        }
    }

    #[test]
    fn music_loops_and_encodes() {
        for t in [Track::Menu, Track::Battle, Track::Boss] {
            let b = music(t);
            let secs = b.s.len() as f32 / SR;
            assert!((20.0..60.0).contains(&secs), "{t:?} is {secs}s");
            let wav = b.wav();
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(wav.len(), 44 + b.s.len() * 2);
        }
    }
}

#[cfg(test)]
mod levels {
    use super::*;

    /// `cargo test -p client levels -- --ignored --nocapture` prints peak / RMS and writes WAVs to /tmp/kz_audio.
    #[test]
    #[ignore]
    fn print_levels() {
        let _ = std::fs::create_dir_all("/tmp/kz_audio");
        let stat = |name: &str, b: &Buf| {
            let peak = b.s.iter().fold(0.0f32, |m, x| m.max(x.abs()));
            let rms = (b.s.iter().map(|x| x * x).sum::<f32>() / b.s.len() as f32).sqrt();
            println!("{name:<16} {:>6.2}s peak {peak:>5.2} rms {rms:>5.3}", b.s.len() as f32 / SR);
            std::fs::write(format!("/tmp/kz_audio/{name}.wav"), b.wav()).unwrap();
        };
        for s in ALL_SFX {
            stat(&format!("{s:?}"), &sfx(s));
        }
        for t in [Track::Menu, Track::Battle, Track::Boss] {
            stat(&format!("music_{t:?}"), &music(t));
        }
    }
}
