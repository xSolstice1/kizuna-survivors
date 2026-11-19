//! The movement key: every hero's own way across the field (`MoveKind`, `move_def`).
//!
//! One key, aimed (`Input::dash`, `Input::aim`; no aim = the way they face). A move has
//! charges that come back one at a time over its cooldown, and the hero can't be hit while
//! it runs (and a moment after). A move is never only a dash: each does something on the way
//! or where it lands, and more in the hero's transformation.
//!
//! - Travelling moves (`MoveDef::time` > 0) carry the hero along the path tick by tick,
//!   sliding along walls like walking does. Teleports find the farthest free point on the line.
//! - What's left behind (Ren's ofuda line, Yuzu's notes, Haru's delayed cut) is a `Trail`.
use super::*;

/// A move in progress: the hero travels `from` -> `to` over `dur` seconds.
#[derive(Clone, Copy, Debug)]
pub struct DashRun {
    pub kind: MoveKind,
    pub from: Vec2,
    pub to: Vec2,
    pub t: f32,
    pub dur: f32,
}

/// The movement key's state.
#[derive(Clone, Copy, Debug)]
pub struct Dash {
    pub charges: u8,
    pub max_charges: u8,
    /// Seconds until the next charge comes back.
    pub recharge: f32,
    pub active: Option<DashRun>,
    /// A move that arrived this tick, its landing still to play (`dash_actions`).
    landed: Option<DashRun>,
    /// Kuon's star marker.
    pub star: Option<Vec2>,
    /// The enemy Daichi's chain caught (`Enemies.id`), slammed on arrival.
    pub hook: Option<u32>,
}

/// What a move leaves on the ground.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrailKind {
    /// A line of ofuda: cuts enemies crossing it, stops fireballs.
    Ofuda,
    /// Music notes: speed and armour while Yuzu stands on them.
    Notes,
    /// A cut waiting to land along the line (`Trail::delay`).
    Cut,
}

#[derive(Clone, Copy, Debug)]
pub struct Trail {
    pub kind: TrailKind,
    pub a: Vec2,
    pub b: Vec2,
    pub width: f32,
    pub life: f32,
    pub max_life: f32,
    pub delay: f32,
    damage: f32,
    tick: f32,
}

/// Seconds the hero can't be hit after a move ends.
const AFTER: f32 = 0.15;
/// Encore notes: armour while on them.
pub const ENCORE_ARMOR: f32 = 3.0;

impl Dash {
    pub fn new(kind: MoveKind, extra: u8) -> Self {
        let n = move_def(kind).charges + extra;
        Dash { charges: n, max_charges: n, recharge: 0.0, active: None, landed: None, star: None, hook: None }
    }
}

impl World {
    pub fn move_kind(&self) -> MoveKind {
        hero_def(self.cfg.hero).movement
    }

    /// Seconds a charge takes to come back (the Firmament: almost none inside the domain).
    pub fn move_cooldown(&self) -> f32 {
        let base = move_def(self.move_kind()).cooldown * (1.0 - self.bonus.move_cd.min(0.5));
        if self.form_kind() == Some(SpecialKind::Firmament) {
            base * 0.25
        } else {
            base
        }
    }

    /// 0..1: how far the next charge has come back (1 = all charges ready).
    pub fn move_ready(&self) -> f32 {
        if self.dash.charges >= self.dash.max_charges {
            1.0
        } else {
            1.0 - self.dash.recharge / self.move_cooldown().max(0.01)
        }
    }

    /// The farthest point from `from` towards `to` the hero can stand on (walls and props stop
    /// it).
    pub(crate) fn free_to(&self, from: Vec2, to: Vec2) -> Vec2 {
        let d = to - from;
        let len = d.length();
        let steps = (len / 4.0).ceil().max(1.0) as usize;
        let mut last = from;
        for k in 1..=steps {
            let p = from + d * (k as f32 / steps as f32);
            if self.terrain.solid_at(p) || (stage::resolve_with(&self.terrain, p, PLAYER_RADIUS) - p).length_sq() > 0.5 {
                break;
            }
            last = p;
        }
        last
    }

    /// Recharge and carry the hero along a move in progress (before the enemies' grid is
    /// built). Returns true while a move is carrying the hero (walking is skipped).
    pub(crate) fn update_dash(&mut self) -> bool {
        let cd = self.move_cooldown();
        if self.dash.charges < self.dash.max_charges {
            self.dash.recharge -= DT;
            if self.dash.recharge <= 0.0 {
                self.dash.charges += 1;
                self.dash.recharge = if self.dash.charges < self.dash.max_charges { cd } else { 0.0 };
            }
        }
        self.encore = (self.encore - DT).max(0.0);
        let Some(mut run) = self.dash.active else { return false };
        run.t += DT;
        let k = (run.t / run.dur).min(1.0);
        // Eased: fast out, settling in.
        let e = 1.0 - (1.0 - k) * (1.0 - k);
        let want = run.from + (run.to - run.from) * e;
        self.player.pos = stage::resolve_with(&self.terrain, want, PLAYER_RADIUS);
        self.player.iframes = self.player.iframes.max(AFTER);
        if k >= 1.0 {
            self.dash.active = None;
            self.dash.landed = Some(run);
        } else {
            self.dash.active = Some(run);
        }
        true
    }

    /// What moves do to the field: trails, a landing, a new move on the key (needs this tick's
    /// grid).
    pub(crate) fn dash_actions(&mut self, input: Input) {
        self.update_trails();
        if let Some(run) = self.dash.landed.take() {
            self.land(run);
        }
        if input.dash && self.dash.active.is_none() && self.dash.charges > 0 && self.state == RunState::Playing {
            let aim = if input.aim.length_sq() > 1e-4 { input.aim.normalize_or_zero() } else { self.player.facing.normalize_or(Vec2::new(1.0, 0.0)) };
            self.start_move(aim);
        }
    }

    fn start_move(&mut self, aim: Vec2) {
        let kind = self.move_kind();
        let m = move_def(kind);
        let p = self.player.pos;
        self.dash.charges -= 1;
        if self.dash.recharge <= 0.0 {
            self.dash.recharge = self.move_cooldown();
        }
        self.player.facing = aim;
        let mut to = p + aim * m.range;
        let dmg = m.damage * self.might();
        let elem = Some(self.hero_element);
        match kind {
            MoveKind::VeilCrossing => {
                // Behind the strongest enemy near the aim (else straight ahead).
                let mut best: Option<(usize, f32)> = None;
                self.query_circle(p, m.range);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    let d = self.enemies.pos[j] - p;
                    if d.normalize_or_zero().dot_with(aim) < 0.2 {
                        continue;
                    }
                    let score = self.enemies.kind[j].rank().threat() * 100.0 + self.enemies.hp[j].min(5000.0) / 50.0 - d.length() * 0.1;
                    if best.is_none_or(|(_, s)| score > s) {
                        best = Some((j, score));
                    }
                }
                if let Some((j, _)) = best {
                    let e = self.enemies.pos[j];
                    to = e + (e - p).normalize_or(aim) * (self.enemies.radius[j] + 12.0);
                    self.enemies.marked[j] = true;
                    let dir = e - p;
                    self.hurt_enemy(j, dmg, dir, elem);
                }
                if self.form_kind() == Some(SpecialKind::Hannya) {
                    self.afterimages.push((p, 0.35));
                }
            }
            MoveKind::StarShift => {
                if let Some(s) = self.dash.star.take() {
                    if (s - p).length() < 600.0 {
                        to = s;
                    }
                }
                self.dash.star = Some(p);
                // The old spot slows what's there.
                self.query_circle(p, 60.0);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    self.enemies.slow[j] = self.enemies.slow[j].max(2.5);
                    let dir = self.enemies.pos[j] - p;
                    self.hurt_enemy(j, dmg, dir, elem);
                }
            }
            MoveKind::SeveredChain => {
                // The chain flies along the aim: it catches the first big enemy on the line, or
                // bites into the first wall or prop; with nothing to catch it's a short lunge.
                self.dash.hook = None;
                let mut caught: Option<(usize, f32)> = None;
                self.query_circle(p + aim * (m.range * 0.5), m.range * 0.5 + 30.0);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    if self.enemies.kind[j].rank() == crate::enemies::Rank::Fodder {
                        continue;
                    }
                    let d = self.enemies.pos[j] - p;
                    let t = d.dot_with(aim);
                    let off = (d - aim * t).length();
                    if t > 0.0 && t < m.range && off < self.enemies.radius[j] + 12.0 && caught.is_none_or(|(_, b)| t < b) {
                        caught = Some((j, t));
                    }
                }
                let wall = self.free_to(p, p + aim * m.range);
                to = match caught {
                    Some((j, t)) if t < (wall - p).length() + 1.0 => {
                        self.dash.hook = Some(self.enemies.id[j]);
                        self.free_to(p, self.enemies.pos[j] - aim * (self.enemies.radius[j] + 10.0))
                    }
                    _ if (wall - p).length() < m.range - 6.0 => wall,
                    _ => self.free_to(p, p + aim * 70.0),
                };
            }
            _ => {}
        }
        let to = if m.time > 0.0 { to } else { self.free_to(p, to) };
        self.events.push(Event::Dash { kind, from: p, to });
        match kind {
            MoveKind::FlaskJump => {
                let r = 34.0 * self.area_mult();
                self.nova(p, r, dmg, style::METEOR, elem);
                self.pools.push(Pool { pos: p, radius: r, life: 3.0, max_life: 3.0, tick: 0.0, damage: dmg * 0.15, elem });
            }
            MoveKind::FoxBlink => {
                let area = self.area_mult();
                self.clones.push(MirageClone { pos: p, facing: aim, moving: false, life: 1.4, hits: 3, iframes: 0.3, strike_cd: 9.0, striking: 0.0, damage: dmg, pop_radius: 46.0 * area, speed: 0.0, slot: 0.0, elem });
                self.events.push(Event::CloneSummoned { pos: p });
            }
            _ => {}
        }
        // The chain reels him in fast, however far it caught.
        let dur = if kind == MoveKind::SeveredChain { ((to - p).length() / 620.0).max(0.08) } else { m.time };
        let run = DashRun { kind, from: p, to, t: 0.0, dur };
        if dur > 0.0 {
            self.dash.active = Some(run);
            self.player.iframes = self.player.iframes.max(dur + AFTER);
        } else {
            self.player.pos = to;
            self.player.iframes = self.player.iframes.max(AFTER + 0.1);
            self.land(run);
        }
    }

    /// The move arrived: its effect along the way and where it landed.
    fn land(&mut self, run: DashRun) {
        let m = move_def(run.kind);
        let (from, to) = (run.from, self.player.pos);
        let dmg = m.damage * self.might();
        let elem = Some(self.hero_element);
        let form = self.form_kind();
        match run.kind {
            MoveKind::PurifyingStep => {
                let divine = form == Some(SpecialKind::Divine);
                let life = if divine { 4.0 } else { 2.4 };
                self.trails.push(Trail { kind: TrailKind::Ofuda, a: from, b: to, width: if divine { 14.0 } else { 9.0 }, life, max_life: life, delay: 0.0, damage: dmg * if divine { 2.0 } else { 1.0 }, tick: 0.0 });
            }
            MoveKind::LightningFlash => {
                self.hurt_along(from, to, 14.0, dmg, elem);
                self.lash_targets(to, 90.0, 3, dmg * 0.6, elem);
            }
            MoveKind::StarlightLeap => {
                self.nova(to, 46.0 * self.area_mult(), dmg, style::SEAL, elem);
                // Pull in the drops round her.
                for g in self.gems.pos.iter_mut() {
                    let d = to - *g;
                    if d.length_sq() < 110.0 * 110.0 {
                        *g += d * 0.85;
                    }
                }
            }
            MoveKind::IaidoRush => {
                self.trails.push(Trail { kind: TrailKind::Cut, a: from, b: to, width: 16.0, life: 0.5, max_life: 0.5, delay: 0.22, damage: dmg, tick: 0.0 });
            }
            MoveKind::EncoreSlide => {
                self.trails.push(Trail { kind: TrailKind::Notes, a: from, b: to, width: 18.0, life: 4.0, max_life: 4.0, delay: 0.0, damage: 0.0, tick: 0.0 });
            }
            MoveKind::FoxBlink => {
                if form == Some(SpecialKind::NineTails) {
                    let d = dmg * 1.5 * self.fox_power();
                    self.hurt_along(from, to, 22.0, d, elem);
                    self.events.push(Event::Lash { from, to });
                }
            }
            MoveKind::TendrilPull => {
                let mukade = form == Some(SpecialKind::Mukade);
                let (w, d) = if mukade { (26.0, dmg * 3.0) } else { (14.0, dmg) };
                self.dead.clear();
                self.query_circle((from + to) * 0.5, (to - from).length() * 0.5 + w + 20.0);
                let seg = to - from;
                let len2 = seg.length_sq().max(1.0);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    let e = self.enemies.pos[j];
                    let t = ((e - from).dot_with(seg) / len2).clamp(0.0, 1.0);
                    if (from + seg * t - e).length() <= w + self.enemies.radius[j] {
                        self.enemies.slow[j] = self.enemies.slow[j].max(2.0);
                    }
                }
                self.hurt_along(from, to, w, d, elem);
                self.events.push(Event::Lash { from, to });
            }
            MoveKind::SeveredChain => {
                // A ground smash where he lands; whatever the chain caught takes the brunt and he
                // rebounds off it.
                let r = 46.0 * self.area_mult();
                self.query_circle(to, r);
                for idx in 0..self.scratch.len() {
                    let j = self.scratch[idx] as usize;
                    let dir = self.enemies.pos[j] - to;
                    self.hurt_enemy(j, dmg, dir, elem);
                    self.shove(j, dir, 260.0);
                }
                self.events.push(Event::Burst { pos: to, radius: r, style: style::QUAKE });
                if let Some(id) = self.dash.hook.take() {
                    if let Some(j) = self.enemies.id.iter().position(|&x| x == id) {
                        let dir = self.enemies.pos[j] - from;
                        self.hurt_enemy(j, dmg * 2.5, dir, elem);
                        let back = self.free_to(to, to - dir.normalize_or_zero() * 22.0);
                        self.player.pos = back;
                    }
                }
                if form == Some(SpecialKind::Daidara) {
                    self.daidara_slam(to, 0.12, 0.8);
                }
            }
            MoveKind::FlaskJump | MoveKind::VeilCrossing | MoveKind::StarShift => {}
        }
        self.events.push(Event::DashLand { kind: run.kind, pos: to });
    }

    fn update_trails(&mut self) {
        let mut i = 0;
        while i < self.trails.len() {
            let mut tr = self.trails[i];
            tr.life -= DT;
            if tr.life <= 0.0 {
                self.trails.swap_remove(i);
                continue;
            }
            let elem = Some(self.hero_element);
            match tr.kind {
                TrailKind::Ofuda => {
                    tr.tick -= DT;
                    if tr.tick <= 0.0 {
                        tr.tick = 0.3;
                        self.hurt_along(tr.a, tr.b, tr.width, tr.damage, elem);
                    }
                    let (a, b, w) = (tr.a, tr.b, tr.width);
                    self.shots.retain(|s| seg_dist(s.pos, a, b) > w + 4.0);
                }
                TrailKind::Notes => {
                    if seg_dist(self.player.pos, tr.a, tr.b) < tr.width {
                        self.encore = 0.2;
                        self.speed_buff = self.speed_buff.max(0.2);
                    }
                }
                TrailKind::Cut => {
                    if tr.delay > 0.0 {
                        tr.delay -= DT;
                        if tr.delay <= 0.0 {
                            self.hurt_along(tr.a, tr.b, tr.width, tr.damage, elem);
                            self.events.push(Event::Iai { from: tr.a, to: tr.b });
                        }
                    }
                }
            }
            self.trails[i] = tr;
            i += 1;
        }
    }
}

/// Distance from `p` to the segment `a`-`b`.
pub(crate) fn seg_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let d = b - a;
    let t = ((p - a).dot_with(d) / d.length_sq().max(1e-6)).clamp(0.0, 1.0);
    (a + d * t - p).length()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::ALL_STAGES;

    fn quiet(hero: HeroId) -> World {
        let mut w = World::new(WorldConfig { hero, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
        w.weapons.clear();
        w
    }

    fn press(w: &mut World, aim: Vec2) {
        w.step(Input { dash: true, aim, ..Default::default() });
    }

    fn idle(w: &mut World, secs: f32) {
        for _ in 0..(secs / DT) as usize {
            w.close_chest();
            w.choose_shrine(None);
            w.step(Input::default());
        }
    }

    #[test]
    fn every_hero_moves_spends_a_charge_and_gets_it_back() {
        for h in ALL_HEROES {
            let mut w = quiet(h);
            let start = w.player.pos;
            press(&mut w, Vec2::new(1.0, 0.0));
            idle(&mut w, 0.6);
            let m = move_def(w.move_kind());
            let went = (w.player.pos - start).length();
            // (Daichi's chain, with nothing to catch, is a short lunge.)
            assert!(went > m.range * 0.35, "{h:?} moved {went}");
            assert_eq!(w.dash.charges, w.dash.max_charges - 1, "{h:?} spent a charge");
            idle(&mut w, m.cooldown + 0.1);
            assert_eq!(w.dash.charges, w.dash.max_charges, "{h:?} got it back");
        }
    }

    #[test]
    fn no_charge_no_move() {
        let mut w = quiet(HeroId::Haru);
        press(&mut w, Vec2::new(1.0, 0.0));
        idle(&mut w, 0.4);
        let p = w.player.pos;
        press(&mut w, Vec2::new(1.0, 0.0));
        idle(&mut w, 0.4);
        assert!((w.player.pos - p).length() < 1.0, "a second move without a charge");
    }

    #[test]
    fn untouchable_while_moving() {
        let mut w = quiet(HeroId::Toya);
        press(&mut w, Vec2::new(1.0, 0.0));
        assert!(w.dash.active.is_some());
        let hp = w.player.hp;
        w.hurt_player(30.0, None);
        assert!(w.player.iframes > 0.0);
        // Contact damage checks iframes; a move keeps them up for its whole length.
        idle(&mut w, 0.1);
        assert!(w.player.iframes > 0.0 && w.player.hp >= hp - 30.0);
    }

    #[test]
    fn teleports_never_end_inside_walls() {
        for stage in ALL_STAGES {
            let mut w = World::new(WorldConfig { hero: HeroId::Kaede, stage, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
            w.weapons.clear();
            if !w.terrain.on {
                continue;
            }
            for k in 0..16 {
                let aim = Vec2::from_angle(k as f32 * std::f32::consts::TAU / 16.0);
                w.dash.charges = 1;
                press(&mut w, aim);
                assert!(!w.terrain.solid_at(w.player.pos), "{stage:?}: blinked into a wall at {:?}", w.player.pos);
            }
        }
    }

    #[test]
    fn ofuda_line_stops_fireballs() {
        let mut w = quiet(HeroId::Ren);
        let p = w.player.pos;
        press(&mut w, Vec2::new(1.0, 0.0));
        idle(&mut w, 0.4);
        assert_eq!(w.trails.len(), 1);
        let mid = (p + w.player.pos) * 0.5;
        w.shots.push(Shot { pos: mid, vel: Vec2::ZERO, life: 3.0, style: 0, elem: None });
        idle(&mut w, 0.05);
        assert!(w.shots.is_empty(), "the line erased the fireball");
    }

    #[test]
    fn encore_guards_on_the_notes() {
        let mut w = quiet(HeroId::Yuzu);
        press(&mut w, Vec2::new(1.0, 0.0));
        idle(&mut w, 0.4);
        assert!(w.encore > 0.0 && w.armor() >= ENCORE_ARMOR);
        idle(&mut w, 4.5);
        assert!(w.encore <= 0.0, "the notes faded");
    }

    #[test]
    fn star_shift_swaps_with_the_marker() {
        let mut w = quiet(HeroId::Kuon);
        let home = w.player.pos;
        press(&mut w, Vec2::new(1.0, 0.0));
        assert_eq!(w.dash.star, Some(home));
        let away = w.player.pos;
        let cd = w.move_cooldown();
        idle(&mut w, cd + 0.1);
        press(&mut w, Vec2::new(0.0, 1.0));
        assert!((w.player.pos - home).length() < 1.0, "back at the marker");
        assert_eq!(w.dash.star, Some(away));
    }

    #[test]
    fn veil_crossing_lands_behind_and_marks() {
        let mut w = quiet(HeroId::Shun);
        let p = w.player.pos;
        w.spawn_enemy(EnemyKind::Oni, p + Vec2::new(80.0, 0.0));
        press(&mut w, Vec2::new(1.0, 0.0));
        assert!(w.player.pos.x > p.x + 80.0, "behind it: {:?}", w.player.pos);
        assert!(w.enemies.hp[0] < w.enemies.max_hp[0]);
    }

    #[test]
    fn moves_are_deterministic() {
        let run = || {
            let mut w = World::new(WorldConfig { seed: 7, hero: HeroId::Riku, ..Default::default() });
            for t in 0..1800 {
                if w.offer.is_some() {
                    w.bot_pick();
                }
                w.close_chest();
                w.choose_shrine(None);
                let mut i = w.bot_input_dashing();
                if t % 200 == 0 {
                    i.dash = true;
                }
                w.step(i);
            }
            w.state_hash()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn the_chain_catches_a_big_enemy_and_slams_it() {
        let mut w = quiet(HeroId::Daichi);
        let p = w.player.pos;
        w.spawn_enemy(EnemyKind::Oni, p + Vec2::new(140.0, 0.0));
        let hp = w.enemies.hp[0];
        press(&mut w, Vec2::new(1.0, 0.0));
        assert!(w.dash.hook.is_some(), "caught the oni");
        idle(&mut w, 0.4);
        assert!(w.player.pos.x > p.x + 90.0 && w.player.pos.x < w.enemies.pos[0].x, "pulled in, short of it: {:?}", w.player.pos);
        assert!(w.enemies.hp[0] < hp - 60.0, "slammed: {} -> {}", hp, w.enemies.hp[0]);
        assert_eq!(w.dash.charges, 1, "two charges");
    }

    #[test]
    fn the_chain_bites_walls_and_never_ends_inside_one() {
        for stage in ALL_STAGES {
            let mut w = World::new(WorldConfig { hero: HeroId::Daichi, stage, stress: Some(Stress { enemies: 0, projectiles: 0 }), ..Default::default() });
            w.weapons.clear();
            for k in 0..12 {
                let aim = Vec2::from_angle(k as f32 * std::f32::consts::TAU / 12.0);
                w.dash.charges = 2;
                press(&mut w, aim);
                idle(&mut w, 0.4);
                assert!(!w.terrain.solid_at(w.player.pos), "{stage:?}: inside a wall at {:?}", w.player.pos);
            }
        }
    }

    #[test]
    fn daidara_fists_land_on_the_crowd() {
        let mut w = quiet(HeroId::Daichi);
        let p = w.player.pos;
        for k in 0..6 {
            w.spawn_enemy(EnemyKind::Oni, p + Vec2::new(120.0 + k as f32 * 6.0, 10.0));
        }
        let before: f32 = w.enemies.hp.iter().sum();
        w.player.special = 1.0;
        w.step(Input { special: true, ..Default::default() });
        assert_eq!(w.form_kind(), Some(SpecialKind::Daidara));
        idle(&mut w, 1.5);
        assert!(w.blasts.iter().any(|b| b.style == style::DAIDARA) || w.enemies.hp.iter().sum::<f32>() < before * 0.6, "the fists came down");
        idle(&mut w, DAIDARA_DURATION);
        assert_eq!(w.form_kind(), None);
    }

    #[test]
    fn unbroken_will_below_half_health() {
        let mut w = quiet(HeroId::Daichi);
        let (m, s) = (w.might(), w.move_speed());
        w.player.hp = w.player.max_hp * 0.4;
        assert!(w.unbroken());
        assert!((w.might() / m - (1.0 + UNBROKEN_MIGHT)).abs() < 1e-3);
        assert!(w.move_speed() > s);
        let mut r = quiet(HeroId::Toya);
        r.player.hp = r.player.max_hp * 0.4;
        assert!(!r.unbroken(), "Daichi's alone");
    }
}
