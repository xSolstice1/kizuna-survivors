//! Gacha-style chest opening: shake → burst (rarity reveal) → reward cards pop in one by one.

use macroquad::prelude::*;
#[allow(unused_imports)]
use crate::kit::{draw_circle, draw_circle_lines};

use crate::kit;
use crate::kit::{screen_height, screen_width};
use sim::content::{awakened_name, chest_rarity, Upgrade};
use sim::world::ChestItem;

use crate::render::Assets;
use crate::synth::Sfx;

const SHAKE: f32 = 0.9;
const BURST: f32 = 0.45;
const CARD_EVERY: f32 = 0.3;
const CARD_POP: f32 = 0.25;

struct Confetti {
    pos: Vec2,
    vel: Vec2,
    color: Color,
    spin: f32,
}

pub struct ChestAnim {
    items: Vec<ChestItem>,
    t: f32,
    confetti: Vec<Confetti>,
    burst_done: bool,
    /// Sounds to play this frame (drained by the caller).
    pub cues: Vec<Sfx>,
}

fn rarity_color(n: usize, time: f32) -> Color {
    match n {
        0 | 1 => Color::from_rgba(200, 225, 255, 255),
        2 => Color::from_rgba(80, 165, 255, 255),
        3 | 4 => Color::from_rgba(185, 110, 255, 255),
        // Legendary: cycling rainbow.
        _ => {
            let h = (time * 0.6).fract();
            let c = |o: f32| 0.5 + 0.5 * ((h + o) * std::f32::consts::TAU).cos();
            Color::new(c(0.0), c(0.33), c(0.67), 1.0)
        }
    }
}

fn ease_out_back(x: f32) -> f32 {
    let (c1, c3) = (1.70158, 2.70158);
    1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
}

fn centered(text: &str, cx: f32, y: f32, size: f32, color: Color) {
    let d = kit::measure(text, size as u16);
    kit::text(text, cx - d.width / 2.0, y, size, color);
}

impl ChestAnim {
    pub fn new(items: Vec<ChestItem>) -> Self {
        ChestAnim { items, t: 0.0, confetti: Vec::new(), burst_done: false, cues: Vec::new() }
    }

    fn reveal_end(&self) -> f32 {
        SHAKE + BURST + self.items.len() as f32 * CARD_EVERY + CARD_POP
    }

    fn finished(&self) -> bool {
        self.t >= self.reveal_end()
    }

    fn spawn_confetti(&mut self, center: Vec2) {
        let n = match self.items.len() {
            1 => 40,
            2 => 90,
            3 | 4 => 160,
            _ => 320,
        };
        for k in 0..n {
            let a = rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = rand::gen_range(200.0, 620.0);
            let color = if self.items.len() >= 5 {
                rarity_color(5, k as f32 * 0.37)
            } else {
                [rarity_color(self.items.len(), 0.0), Color::from_rgba(255, 220, 120, 255), WHITE][k % 3]
            };
            self.confetti.push(Confetti { pos: center, vel: vec2(a.cos(), a.sin()) * speed, color, spin: rand::gen_range(0.0, 6.0) });
        }
    }

    /// Draws one frame. Returns true when the player dismisses the screen.
    pub fn update_and_draw(&mut self, assets: &Assets, dt: f32) -> bool {
        let clicked = is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if clicked {
            if self.finished() {
                return true;
            }
            self.t = self.reveal_end(); // skip to the end
        }
        let before = self.t;
        self.t += dt;
        let crossed = |at: f32| before < at && self.t >= at || (before == 0.0 && at == 0.0);
        if crossed(0.0) {
            self.cues.push(Sfx::ChestShake);
        }
        if crossed(SHAKE) {
            self.cues.push(if self.items.len() >= 5 { Sfx::ChestLegendary } else { Sfx::ChestBurst });
        }
        for k in 0..self.items.len() {
            if crossed(SHAKE + BURST + k as f32 * CARD_EVERY) {
                self.cues.push(Sfx::Card);
            }
        }

        let (w, h) = (screen_width(), screen_height());
        let center = vec2(w / 2.0, h * 0.40);
        let n = self.items.len();
        let rc = rarity_color(n, self.t);

        draw_rectangle(0.0, 0.0, w, h, Color::new(0.02, 0.01, 0.05, 0.9));

        // Glow behind the chest, stronger for rarer chests and as the shake builds.
        let build = (self.t / SHAKE).min(1.0);
        let glow_r = 90.0 + 50.0 * build + n as f32 * 10.0;
        for k in 0..6 {
            let mut c = rc;
            c.a = 0.05 * build * (1.0 + n as f32 * 0.3) * (6 - k) as f32 / 6.0;
            draw_circle(center.x, center.y, glow_r * (0.5 + k as f32 * 0.18), c);
        }

        // Light rays once open.
        if self.t >= SHAKE {
            let rays = 10 + n * 2;
            let len = h * 0.6;
            for k in 0..rays {
                let a = self.t * 0.6 + k as f32 / rays as f32 * std::f32::consts::TAU;
                let half = 0.07;
                let mut c = rc;
                c.a = 0.16;
                draw_triangle(
                    center,
                    center + vec2((a - half).cos(), (a - half).sin()) * len,
                    center + vec2((a + half).cos(), (a + half).sin()) * len,
                    c,
                );
            }
        }

        // The chest: shaking while closed, open after the burst.
        let scale = 8.0;
        let (cw, ch) = (14.0 * scale, if self.t < SHAKE { 12.0 } else { 16.0 } * scale);
        let jitter = if self.t < SHAKE {
            let amp = 2.0 + 10.0 * build * build;
            vec2((self.t * 55.0).sin() * amp, (self.t * 41.0).cos() * amp * 0.4)
        } else {
            Vec2::ZERO
        };
        let tex = if self.t < SHAKE { &assets.chest } else { &assets.chest_open };
        draw_texture_ex(
            tex,
            (center.x - cw / 2.0 + jitter.x).round(),
            (center.y - ch / 2.0 + jitter.y).round(),
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(cw, ch)), ..Default::default() },
        );

        if self.t >= SHAKE && !self.burst_done {
            self.burst_done = true;
            self.spawn_confetti(center);
        }
        // White flash at the moment of opening.
        if self.t >= SHAKE && self.t < SHAKE + BURST {
            let f = 1.0 - (self.t - SHAKE) / BURST;
            draw_rectangle(0.0, 0.0, w, h, Color::new(1.0, 1.0, 1.0, 0.85 * f * f));
        }

        for c in &mut self.confetti {
            c.vel.y += 520.0 * dt;
            c.vel *= 0.985;
            c.pos += c.vel * dt;
            c.spin += dt * 8.0;
            let s = 3.0 + 2.0 * c.spin.sin().abs();
            draw_rectangle(c.pos.x, c.pos.y, s, 6.0 - s * 0.5, c.color);
        }
        self.confetti.retain(|c| c.pos.y < h + 20.0);

        // Rarity label.
        if self.t >= SHAKE {
            let label = chest_rarity(n).to_uppercase();
            let size = if n >= 5 { 64.0 } else { 52.0 };
            let pop = ease_out_back(((self.t - SHAKE) / 0.3).min(1.0));
            centered(&label, center.x + 3.0, h * 0.14 + 3.0, size * pop, Color::new(0.0, 0.0, 0.0, 0.6));
            centered(&label, center.x, h * 0.14, size * pop, rc);
            centered(&format!("{} reward{}", n, if n == 1 { "" } else { "s" }), center.x, h * 0.14 + 30.0, 22.0, WHITE);
        } else {
            centered("...", center.x, h * 0.14, 52.0, Color::new(1.0, 1.0, 1.0, 0.4 + 0.6 * build));
        }

        // Reward cards.
        let (card_w, card_h, gap) = (150.0, 196.0, 18.0);
        let total = n as f32 * card_w + (n as f32 - 1.0) * gap;
        let x0 = (w - total) / 2.0;
        let y0 = h * 0.62;
        for (i, item) in self.items.iter().enumerate() {
            let start = SHAKE + BURST + i as f32 * CARD_EVERY;
            if self.t < start {
                continue;
            }
            let p = ease_out_back(((self.t - start) / CARD_POP).min(1.0));
            let (cx, cy) = (x0 + i as f32 * (card_w + gap) + card_w / 2.0, y0 + card_h / 2.0);
            let (sw, sh) = (card_w * p, card_h * p);
            let (x, y) = (cx - sw / 2.0, cy - sh / 2.0);
            let awaken = matches!(item, ChestItem::Awakening(_));
            let border = if awaken { Color::from_rgba(255, 210, 90, 255) } else { rc };
            draw_rectangle(x, y, sw, sh, Color::new(0.08, 0.06, 0.14, 0.96));
            draw_rectangle_lines(x, y, sw, sh, if awaken { 4.0 } else { 2.0 }, border);
            if p < 0.9 {
                continue; // text only once the pop has settled
            }
            let (upgrade, title, sub) = match *item {
                ChestItem::Awakening(k) => (Upgrade::Weapon(k), awakened_name(k).to_string(), "AWAKENED!".to_string()),
                ChestItem::LevelUp(u, lvl) => (u, u.name().to_string(), format!("Lv {lvl}")),
                ChestItem::Heal => (Upgrade::Heal, "Onigiri".to_string(), "+30 HP".to_string()),
            };
            let icon = assets.icon(upgrade);
            if awaken {
                draw_circle(cx, y + 62.0, 42.0, Color::new(1.0, 0.85, 0.35, 0.25 + 0.1 * (self.t * 5.0).sin()));
            }
            draw_texture_ex(icon, cx - 32.0, y + 30.0, WHITE, DrawTextureParams { dest_size: Some(vec2(64.0, 64.0)), ..Default::default() });
            // Wrap long names onto two lines.
            let size = if kit::measure(&title, 22).width > card_w - 16.0 { 18.0 } else { 22.0 };
            let words: Vec<&str> = title.split_whitespace().collect();
            if kit::measure(&title, size as u16).width > card_w - 16.0 && words.len() > 1 {
                let mid = words.len().div_ceil(2);
                centered(&words[..mid].join(" "), cx, y + 124.0, size, WHITE);
                centered(&words[mid..].join(" "), cx, y + 144.0, size, WHITE);
            } else {
                kit::text_fit_centered(&title, cx, y + 132.0, card_w - 16.0, size, WHITE);
            }
            kit::text_fit_centered(&sub, cx, y + 176.0, card_w - 14.0, 20.0, if awaken { border } else { Color::new(0.8, 0.85, 1.0, 1.0) });
        }

        if self.finished() {
            let a = 0.55 + 0.45 * (self.t * 4.0).sin();
            centered("Click or press Enter to continue", w / 2.0, h * 0.96, 22.0, Color::new(1.0, 1.0, 1.0, a));
        }
        false
    }
}
