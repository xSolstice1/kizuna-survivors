//! A simple kiting AI used by tests, the benchmark and (later) balance runs.

use crate::content::Upgrade;
use crate::{Input, Vec2, World};

impl World {
    /// Takes the first card that is not a pointless replacement. A replacement card is taken only to overwrite
    /// the lowest-level skill of a lower tier; with none, the bot cancels and takes the first plain card.
    pub fn bot_pick(&mut self) {
        if self.replacing.is_some() {
            if self.bot_replace() {
                return;
            }
            self.replace_with(None);
        }
        let offer = self.offer.clone().unwrap_or_default();
        let worthwhile = |w: &World, u: Upgrade| !w.is_replacement(u) || w.replace_target_below(u).is_some();
        let pick = offer.iter().position(|u| worthwhile(self, *u)).unwrap_or(0);
        self.choose(pick);
        if self.replacing.is_some() && !self.bot_replace() {
            self.replace_with(None);
            let plain = self.offer.as_ref().and_then(|o| o.iter().position(|u| !self.is_replacement(*u))).unwrap_or(0);
            self.choose(plain);
        }
    }

    fn bot_replace(&mut self) -> bool {
        let Some(new) = self.replacing else { return false };
        let Some(j) = self.replace_target_below(new) else { return false };
        self.replace_with(Some(j));
        true
    }

    /// Index into the `replace_targets` that `new` would overwrite: lowest level among strictly lower tiers.
    fn replace_target_below(&self, new: Upgrade) -> Option<usize> {
        let targets = self.replace_targets_for(Some(new));
        targets.iter().enumerate().filter(|(_, (u, _))| u.tier() < new.tier()).min_by_key(|(_, (u, level))| (*level, u.tier())).map(|(j, _)| j)
    }

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
