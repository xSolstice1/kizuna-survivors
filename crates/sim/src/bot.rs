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
        let flee = flee.normalize_or_zero();
        if closest < 40.0 * 40.0 {
            return Input { move_dir: flee };
        }
        let gem = self
            .gems
            .pos
            .iter()
            .chain(self.gems.overflow.as_ref().map(|(g, _)| g))
            .min_by(|a, b| (**a - p).length_sq().partial_cmp(&(**b - p).length_sq()).unwrap())
            .map_or(Vec2::ZERO, |g| (*g - p).normalize_or_zero());
        Input { move_dir: flee * 0.6 + gem }
    }
}
