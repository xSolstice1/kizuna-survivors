//! A simple kiting AI used by tests, the benchmark and (later) balance runs.

use crate::{Input, Vec2, World};

impl World {
    /// Flee nearby enemies (closer ones push harder) while drifting toward the
    /// nearest gem; flee purely when something is about to touch the player.
    pub fn bot_input(&self) -> Input {
        let p = self.player.pos;
        let mut flee = Vec2::ZERO;
        let mut closest = f32::MAX;
        for &e in &self.enemies.pos {
            let d = p - e;
            let d2 = d.length_sq();
            closest = closest.min(d2);
            if d2 < 140.0 * 140.0 && d2 > 1e-3 {
                flee += d * (1.0 / d2);
            }
        }
        // Fireballs are dodged harder than bodies.
        for s in &self.shots {
            let d = p - s.pos;
            let d2 = d.length_sq();
            if d2 < 70.0 * 70.0 && d2 > 1e-3 {
                flee += d * (2.5 / d2);
                closest = closest.min(d2);
            }
        }
        // Step out of telegraphed blasts, domains and pressure zones.
        let mut danger = false;
        for h in &self.hazards {
            let d = p - h.pos;
            let reach = h.radius + 14.0;
            if d.length_sq() < reach * reach {
                let away = if d.length_sq() > 1e-3 { d.normalize_or_zero() } else { Vec2::new(1.0, 0.0) };
                flee += away * 0.2;
                danger = true;
            }
        }
        let flee = flee.normalize_or_zero();
        if danger {
            return Input { move_dir: flee, special: self.player.special >= 1.0 && closest < 60.0 * 60.0 };
        }
        let special = self.player.special >= 1.0 && closest < 60.0 * 60.0;
        if closest < 40.0 * 40.0 {
            return Input { move_dir: flee, special };
        }
        let gem = self
            .gems
            .pos
            .iter()
            .min_by(|a, b| (**a - p).length_sq().partial_cmp(&(**b - p).length_sq()).unwrap())
            .map_or(Vec2::ZERO, |g| (*g - p).normalize_or_zero());
        Input { move_dir: flee * 0.6 + gem, special }
    }
}
