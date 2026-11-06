//! Touch controls for phones and tablets: a floating joystick on the left of the screen,
//! a Special button and a Pause button. Shown only once a touch has been seen.
//! Everything else (menus, cards, buttons, the minimap) works through macroquad's
//! touch-to-mouse emulation.

use macroquad::prelude::*;

use crate::kit::{self, screen_height, screen_width};

/// Joystick knob travel in virtual pixels.
const STICK_RANGE: f32 = 56.0;

#[derive(Default)]
pub struct TouchControls {
    /// A touch has been seen this session (show the on-screen controls).
    pub active: bool,
    /// (touch id, origin, current) in virtual UI pixels.
    stick: Option<(u64, Vec2, Vec2)>,
    /// Pressed this frame.
    pub special: bool,
    pub pause: bool,
}

fn special_rect() -> Rect {
    let (w, h) = (screen_width(), screen_height());
    Rect::new(w - 170.0, h - 170.0, 130.0, 130.0)
}

fn pause_rect() -> Rect {
    Rect::new(324.0, 26.0, 48.0, 48.0)
}

impl TouchControls {
    /// Reads this frame's touches. `playing` = a run is active (controls only work then).
    pub fn update(&mut self, playing: bool) {
        self.special = false;
        self.pause = false;
        for t in touches() {
            self.active = true;
            let pos = kit::to_ui(t.position);
            match t.phase {
                TouchPhase::Started => {
                    if !playing {
                        continue;
                    }
                    if special_rect().contains(pos) {
                        self.special = true;
                    } else if pause_rect().contains(pos) {
                        self.pause = true;
                    } else if self.stick.is_none() && pos.x < screen_width() * 0.6 {
                        self.stick = Some((t.id, pos, pos));
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if let Some((id, o, _)) = self.stick {
                        if id == t.id {
                            self.stick = Some((id, o, pos));
                        }
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if self.stick.is_some_and(|(id, _, _)| id == t.id) {
                        self.stick = None;
                    }
                }
            }
        }
        if !playing {
            self.stick = None;
        }
    }

    /// Movement from the joystick (length 0..=1).
    pub fn dir(&self) -> sim::Vec2 {
        match self.stick {
            Some((_, o, c)) => {
                let d = c - o;
                let len = d.length();
                if len < 6.0 {
                    return sim::Vec2::ZERO;
                }
                let k = (len / STICK_RANGE).min(1.0) / len;
                sim::Vec2::new(d.x * k, d.y * k)
            }
            None => sim::Vec2::ZERO,
        }
    }

    /// Draws the on-screen controls (virtual UI coordinates).
    pub fn draw(&self, special: f32, special_ready: bool, form_left: Option<f32>) {
        if !self.active {
            return;
        }
        let h = screen_height();
        // Joystick: where the thumb went down, or a faint hint in the corner.
        match self.stick {
            Some((_, o, c)) => {
                draw_circle(o.x, o.y, STICK_RANGE + 14.0, Color::new(0.05, 0.04, 0.1, 0.35));
                draw_circle_lines(o.x, o.y, STICK_RANGE + 14.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.25));
                let d = c - o;
                let k = if d.length() > STICK_RANGE { STICK_RANGE / d.length() } else { 1.0 };
                let knob = o + d * k;
                draw_circle(knob.x, knob.y, 30.0, Color::new(1.0, 1.0, 1.0, 0.35));
                draw_circle_lines(knob.x, knob.y, 30.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.6));
            }
            None => {
                let o = vec2(140.0, h - 140.0);
                draw_circle_lines(o.x, o.y, STICK_RANGE + 14.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.12));
                draw_circle(o.x, o.y, 26.0, Color::new(1.0, 1.0, 1.0, 0.08));
            }
        }
        // Special button with its meter as a ring.
        let r = special_rect();
        let c = r.center();
        let rad = r.w / 2.0;
        let pulse = if special_ready { 0.75 + 0.25 * (get_time() as f32 * 6.0).sin() } else { 1.0 };
        draw_circle(c.x, c.y, rad, Color::new(0.08, 0.06, 0.14, 0.7));
        let frac = form_left.unwrap_or(special).clamp(0.0, 1.0);
        let segs = 48;
        for k in 0..((segs as f32 * frac) as usize) {
            let a0 = -std::f32::consts::FRAC_PI_2 + k as f32 / segs as f32 * std::f32::consts::TAU;
            let a1 = a0 + std::f32::consts::TAU / segs as f32;
            let col = if form_left.is_some() { Color::new(1.0, 0.4, 0.3, 1.0) } else { Color::new(pulse, 0.85 * pulse, 0.45 * pulse, 1.0) };
            draw_line(c.x + a0.cos() * (rad - 5.0), c.y + a0.sin() * (rad - 5.0), c.x + a1.cos() * (rad - 5.0), c.y + a1.sin() * (rad - 5.0), 7.0, col);
        }
        draw_circle_lines(c.x, c.y, rad, 2.0, Color::new(1.0, 0.82, 0.42, if special_ready { pulse } else { 0.4 }));
        let label = if form_left.is_some() { "ACTIVE" } else if special_ready { "SPECIAL" } else { "charging" };
        let d = kit::measure(label, 18);
        kit::text(label, c.x - d.width / 2.0, c.y + 6.0, 18.0, if special_ready { kit::GOLD } else { kit::FAINT });
        // Pause button.
        let p = pause_rect();
        kit::rounded(p.x, p.y, p.w, p.h, 10.0, Color::new(0.08, 0.06, 0.14, 0.85));
        kit::rounded_outline(p.x, p.y, p.w, p.h, 10.0, 1.5, kit::EDGE);
        draw_rectangle(p.x + 15.0, p.y + 13.0, 6.0, 22.0, kit::INK);
        draw_rectangle(p.x + 27.0, p.y + 13.0, 6.0, 22.0, kit::INK);
    }
}
