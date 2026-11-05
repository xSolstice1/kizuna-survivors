//! Meta-progression: Sen (coins), Shrine Shop power-ups, hero unlocks and the save profile.
//! The profile is plain `key=value` text so the client can keep it in localStorage.

use crate::content::{HeroId, ALL_HEROES};
use crate::world::{RunState, World};

/// Additive stat bonuses from a hero's innate passive plus Shrine Shop ranks. Neutral by default.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bonus {
    /// Fractions (0.1 = +10%).
    pub might: f32,
    pub max_hp: f32,
    pub cooldown: f32,
    pub area: f32,
    pub move_speed: f32,
    pub magnet: f32,
    pub growth: f32,
    pub greed: f32,
    /// Flat damage reduction per hit.
    pub armor: f32,
    /// HP per second.
    pub recovery: f32,
    /// Extra lives.
    pub revivals: u8,
}

impl Bonus {
    pub fn plus(self, o: Bonus) -> Bonus {
        Bonus {
            might: self.might + o.might,
            max_hp: self.max_hp + o.max_hp,
            cooldown: self.cooldown + o.cooldown,
            area: self.area + o.area,
            move_speed: self.move_speed + o.move_speed,
            magnet: self.magnet + o.magnet,
            growth: self.growth + o.growth,
            greed: self.greed + o.greed,
            armor: self.armor + o.armor,
            recovery: self.recovery + o.recovery,
            revivals: self.revivals + o.revivals,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerUp {
    Might,
    Armor,
    MaxHp,
    Recovery,
    Cooldown,
    Area,
    MoveSpeed,
    Magnet,
    Growth,
    Greed,
    Revival,
}

pub const ALL_POWERUPS: [PowerUp; 11] = [
    PowerUp::Might,
    PowerUp::Armor,
    PowerUp::MaxHp,
    PowerUp::Recovery,
    PowerUp::Cooldown,
    PowerUp::Area,
    PowerUp::MoveSpeed,
    PowerUp::Magnet,
    PowerUp::Growth,
    PowerUp::Greed,
    PowerUp::Revival,
];

impl PowerUp {
    pub fn index(self) -> usize {
        ALL_POWERUPS.iter().position(|p| *p == self).unwrap()
    }
    pub fn name(self) -> &'static str {
        match self {
            PowerUp::Might => "Might",
            PowerUp::Armor => "Armor",
            PowerUp::MaxHp => "Max Health",
            PowerUp::Recovery => "Recovery",
            PowerUp::Cooldown => "Cooldown",
            PowerUp::Area => "Area",
            PowerUp::MoveSpeed => "Move Speed",
            PowerUp::Magnet => "Magnet",
            PowerUp::Growth => "Growth",
            PowerUp::Greed => "Greed",
            PowerUp::Revival => "Revival",
        }
    }
    /// Save-file key (stable; never rename).
    fn key(self) -> &'static str {
        match self {
            PowerUp::Might => "might",
            PowerUp::Armor => "armor",
            PowerUp::MaxHp => "max_hp",
            PowerUp::Recovery => "recovery",
            PowerUp::Cooldown => "cooldown",
            PowerUp::Area => "area",
            PowerUp::MoveSpeed => "move_speed",
            PowerUp::Magnet => "magnet",
            PowerUp::Growth => "growth",
            PowerUp::Greed => "greed",
            PowerUp::Revival => "revival",
        }
    }
    pub fn max_rank(self) -> u8 {
        match self {
            PowerUp::Might | PowerUp::Recovery | PowerUp::Growth | PowerUp::Greed => 5,
            PowerUp::Armor | PowerUp::MaxHp => 3,
            PowerUp::Cooldown | PowerUp::Area | PowerUp::MoveSpeed | PowerUp::Magnet => 2,
            PowerUp::Revival => 1,
        }
    }
    fn base_cost(self) -> u32 {
        match self {
            PowerUp::Might | PowerUp::Cooldown => 200,
            PowerUp::Armor | PowerUp::Area => 150,
            PowerUp::MaxHp | PowerUp::Recovery | PowerUp::MoveSpeed | PowerUp::Magnet => 100,
            PowerUp::Growth | PowerUp::Greed => 120,
            PowerUp::Revival => 1000,
        }
    }
    /// What one rank gives.
    fn per_rank(self) -> Bonus {
        let mut b = Bonus::default();
        match self {
            PowerUp::Might => b.might = 0.05,
            PowerUp::Armor => b.armor = 1.0,
            PowerUp::MaxHp => b.max_hp = 0.10,
            PowerUp::Recovery => b.recovery = 0.1,
            PowerUp::Cooldown => b.cooldown = 0.025,
            PowerUp::Area => b.area = 0.05,
            PowerUp::MoveSpeed => b.move_speed = 0.05,
            PowerUp::Magnet => b.magnet = 0.25,
            PowerUp::Growth => b.growth = 0.03,
            PowerUp::Greed => b.greed = 0.10,
            PowerUp::Revival => b.revivals = 1,
        }
        b
    }
    pub fn describe(self) -> &'static str {
        match self {
            PowerUp::Might => "+5% damage per rank.",
            PowerUp::Armor => "-1 damage taken per hit, per rank.",
            PowerUp::MaxHp => "+10% max health per rank.",
            PowerUp::Recovery => "Regenerate 0.1 HP per second, per rank.",
            PowerUp::Cooldown => "-2.5% weapon cooldowns per rank.",
            PowerUp::Area => "+5% weapon area per rank.",
            PowerUp::MoveSpeed => "+5% move speed per rank.",
            PowerUp::Magnet => "+25% pickup radius per rank.",
            PowerUp::Growth => "+3% experience per rank.",
            PowerUp::Greed => "+10% Sen per rank.",
            PowerUp::Revival => "Get back up once per run with half health.",
        }
    }
}

/// How a hero is unlocked. Ren is always available.
pub fn unlock_text(h: HeroId) -> &'static str {
    match h {
        HeroId::Ren => "Available from the start.",
        HeroId::Kaede => "Survive 10:00 in a single night.",
        HeroId::Mika => "Defeat the Great Oni.",
        HeroId::Haru => "Defeat 10,000 Otherworlders in total.",
        HeroId::Nanami => "Reach level 50 in a single night.",
        HeroId::Yuzu => "Defeat the Night Lord.",
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub sen: u32,
    /// Sen spent on ranks (returned by a refund).
    pub spent: u32,
    pub ranks: [u8; ALL_POWERUPS.len()],
    pub unlocked: Vec<HeroId>,
    pub selected: HeroId,
    pub runs: u32,
    pub total_kills: u64,
    pub best_time: f32,
    pub best_level: u32,
    pub great_onis: u32,
    pub night_lords: u32,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            sen: 0,
            spent: 0,
            ranks: [0; ALL_POWERUPS.len()],
            unlocked: vec![HeroId::Ren],
            selected: HeroId::Ren,
            runs: 0,
            total_kills: 0,
            best_time: 0.0,
            best_level: 0,
            great_onis: 0,
            night_lords: 0,
        }
    }
}

/// What a finished run gave the profile.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunReport {
    pub sen: u32,
    pub unlocked: Vec<HeroId>,
}

impl Profile {
    pub fn rank(&self, p: PowerUp) -> u8 {
        self.ranks[p.index()]
    }

    /// Price of the next rank. Every rank bought anywhere makes all power-ups 10% dearer.
    pub fn cost(&self, p: PowerUp) -> u32 {
        let owned: u32 = self.ranks.iter().map(|r| *r as u32).sum();
        let base = p.base_cost() * (self.rank(p) as u32 + 1);
        base + base * owned / 10
    }

    pub fn can_buy(&self, p: PowerUp) -> bool {
        self.rank(p) < p.max_rank() && self.sen >= self.cost(p)
    }

    pub fn buy(&mut self, p: PowerUp) -> bool {
        if !self.can_buy(p) {
            return false;
        }
        let c = self.cost(p);
        self.sen -= c;
        self.spent += c;
        self.ranks[p.index()] += 1;
        true
    }

    /// Resets all ranks and returns every Sen spent on them.
    pub fn refund(&mut self) {
        self.sen += self.spent;
        self.spent = 0;
        self.ranks = [0; ALL_POWERUPS.len()];
    }

    pub fn bonus(&self) -> Bonus {
        let mut b = Bonus::default();
        for p in ALL_POWERUPS {
            for _ in 0..self.rank(p) {
                b = b.plus(p.per_rank());
            }
        }
        b
    }

    pub fn is_unlocked(&self, h: HeroId) -> bool {
        self.unlocked.contains(&h)
    }

    /// Folds a finished run into the profile: banks Sen, updates records, unlocks heroes.
    pub fn record_run(&mut self, w: &World) -> RunReport {
        debug_assert!(w.state != RunState::Playing);
        self.runs += 1;
        self.sen += w.sen;
        self.total_kills += w.kills as u64;
        self.best_time = self.best_time.max(w.time);
        self.best_level = self.best_level.max(w.player.level);
        self.great_onis += w.great_oni_defeated as u32;
        let night_lord = w.state == RunState::Won;
        self.night_lords += night_lord as u32;
        let mut report = RunReport { sen: w.sen, unlocked: Vec::new() };
        for h in ALL_HEROES {
            let earned = match h {
                HeroId::Ren => true,
                HeroId::Kaede => w.time >= 600.0,
                HeroId::Mika => w.great_oni_defeated,
                HeroId::Haru => self.total_kills >= 10_000,
                HeroId::Nanami => w.player.level >= 50,
                HeroId::Yuzu => night_lord,
            };
            if earned && !self.is_unlocked(h) {
                self.unlocked.push(h);
                report.unlocked.push(h);
            }
        }
        report
    }

    pub fn serialize(&self) -> String {
        let mut s = String::from("v=1\n");
        s += &format!("sen={}\nspent={}\nruns={}\nkills={}\nbest_time={}\nbest_level={}\ngreat_onis={}\nnight_lords={}\n", self.sen, self.spent, self.runs, self.total_kills, self.best_time, self.best_level, self.great_onis, self.night_lords);
        s += &format!("selected={}\n", self.selected.key());
        let unlocked: Vec<&str> = self.unlocked.iter().map(|h| h.key()).collect();
        s += &format!("unlocked={}\n", unlocked.join(","));
        for p in ALL_POWERUPS {
            s += &format!("rank.{}={}\n", p.key(), self.rank(p));
        }
        s
    }

    /// Lenient: unknown keys and bad values are ignored, so old saves keep loading.
    pub fn parse(text: &str) -> Profile {
        let mut p = Profile::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let (k, v) = (k.trim(), v.trim());
            match k {
                "sen" => p.sen = v.parse().unwrap_or(0),
                "spent" => p.spent = v.parse().unwrap_or(0),
                "runs" => p.runs = v.parse().unwrap_or(0),
                "kills" => p.total_kills = v.parse().unwrap_or(0),
                "best_time" => p.best_time = v.parse().unwrap_or(0.0),
                "best_level" => p.best_level = v.parse().unwrap_or(0),
                "great_onis" => p.great_onis = v.parse().unwrap_or(0),
                "night_lords" => p.night_lords = v.parse().unwrap_or(0),
                "selected" => {
                    if let Some(h) = HeroId::from_key(v) {
                        p.selected = h;
                    }
                }
                "unlocked" => {
                    for h in v.split(',').filter_map(HeroId::from_key) {
                        if !p.unlocked.contains(&h) {
                            p.unlocked.push(h);
                        }
                    }
                }
                _ => {
                    if let Some(key) = k.strip_prefix("rank.") {
                        if let Some(pu) = ALL_POWERUPS.iter().find(|pu| pu.key() == key) {
                            p.ranks[pu.index()] = v.parse::<u8>().unwrap_or(0).min(pu.max_rank());
                        }
                    }
                }
            }
        }
        if !p.is_unlocked(p.selected) {
            p.selected = HeroId::Ren;
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorldConfig;

    #[test]
    fn profile_round_trips() {
        let mut p = Profile { sen: 1234, spent: 300, runs: 3, total_kills: 99_999, best_time: 612.5, best_level: 41, great_onis: 2, night_lords: 1, ..Default::default() };
        p.ranks[PowerUp::Greed.index()] = 4;
        p.unlocked.push(HeroId::Kaede);
        p.selected = HeroId::Kaede;
        assert_eq!(Profile::parse(&p.serialize()), p);
        assert_eq!(Profile::parse("garbage\nsen=abc\nrank.might=99"), Profile { ranks: { let mut r = [0; 11]; r[0] = 5; r }, ..Default::default() });
    }

    #[test]
    fn buying_and_refunding() {
        let mut p = Profile { sen: 10_000, ..Default::default() };
        let c1 = p.cost(PowerUp::Might);
        assert!(p.buy(PowerUp::Might));
        assert!(p.cost(PowerUp::Might) > c1, "next rank costs more");
        assert!(p.buy(PowerUp::Revival));
        assert!(!p.buy(PowerUp::Revival), "max rank");
        assert!((p.bonus().might - 0.05).abs() < 1e-6 && p.bonus().revivals == 1);
        p.refund();
        assert_eq!(p.sen, 10_000);
        assert_eq!(p.bonus(), Bonus::default());
    }

    #[test]
    fn runs_unlock_heroes() {
        let mut p = Profile::default();
        let mut w = World::new(WorldConfig::default());
        w.time = 601.0;
        w.sen = 40;
        w.state = RunState::Dead;
        let r = p.record_run(&w);
        assert_eq!(r.unlocked, vec![HeroId::Kaede]);
        assert_eq!(p.sen, 40);
        assert!(p.record_run(&w).unlocked.is_empty(), "only unlocks once");
    }
}
