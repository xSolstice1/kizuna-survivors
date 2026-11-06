//! Meta-progression: Sen (coins), Shrine Shop power-ups, hero unlocks and the save profile.
//! The profile is plain `key=value` text so the client can keep it in localStorage.

use crate::campaign::{Difficulty, Relic, StageId, ALL_RELICS, ALL_STAGES};
use crate::gear::{self, Item, Loadout, Rarity, Slot, LOADOUT};
use crate::rng::Rng;
use crate::content::{EnemyKind, HeroId, PassiveKind, ShrineEvent, WeaponKind, ALL_ENEMIES, ALL_HEROES, ALL_PASSIVES, ALL_SHRINE_EVENTS, ALL_WEAPONS};
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
    /// Equipment-only stats (fractions).
    pub crit_chance: f32,
    pub crit_damage: f32,
    /// Extra damage to elites and bosses.
    pub boss_damage: f32,
    pub special_charge: f32,
    /// Better chests and more floor pickups.
    pub luck: f32,
    /// More and rarer equipment drops.
    pub gear_find: f32,
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
            crit_chance: self.crit_chance + o.crit_chance,
            crit_damage: self.crit_damage + o.crit_damage,
            boss_damage: self.boss_damage + o.boss_damage,
            special_charge: self.special_charge + o.special_charge,
            luck: self.luck + o.luck,
            gear_find: self.gear_find + o.gear_find,
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
        HeroId::Riku => "As Ren, clear Neon Shotengai.",
        HeroId::Shun => "As Riku, clear Hidden Shrine Valley.",
        HeroId::Toya => "As Shun, clear Pale Citadel of Souls.",
        HeroId::Kaede => "As Ren, Awaken a weapon.",
        HeroId::Mika => "As Riku, defeat the Night Lord.",
        HeroId::Haru => "As Toya, open 10 chests in one night.",
        HeroId::Nanami => "As Shun, take 3 shrine blessings in one night.",
        HeroId::Yuzu => "As Toya, reach level 60.",
        HeroId::Kuon => "As Toya, clear Rainward 13.",
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
    /// Collection: kills per enemy kind (indexed like `ALL_ENEMIES`; > 0 = discovered).
    pub enemy_kills: [u64; ALL_ENEMIES.len()],
    pub weapons_found: Vec<WeaponKind>,
    pub awakenings_found: Vec<WeaponKind>,
    pub passives_found: Vec<PassiveKind>,
    pub shrines_found: Vec<ShrineEvent>,
    /// Lifetime statistics.
    pub wins: u32,
    pub deaths: u32,
    pub dawn_outs: u32,
    pub best_kills: u64,
    pub time_played: f32,
    pub sen_earned: u64,
    pub chests_opened: u32,
    pub shrines_taken: u32,
    pub awakenings: u32,
    pub specials_used: u32,
    /// Per hero (indexed like `ALL_HEROES`): nights, wins, longest night.
    pub hero_runs: [u32; ALL_HEROES.len()],
    pub hero_wins: [u32; ALL_HEROES.len()],
    pub hero_best: [f32; ALL_HEROES.len()],
    /// Campaign: the stage and difficulty picked on the map, clears and best times per
    /// stage x difficulty, bosses ever defeated per stage (bits: mini, boss, final), relics.
    pub stage: StageId,
    pub difficulty: Difficulty,
    pub cleared: [[bool; 3]; ALL_STAGES.len()],
    pub stage_best: [[f32; 3]; ALL_STAGES.len()],
    pub stage_bosses: [u8; ALL_STAGES.len()],
    pub relics: Vec<Relic>,
    /// Nightmare+N: highest Ascension open per stage (a Nightmare+N clear opens N+1) and the pick.
    pub asc_open: [u32; ALL_STAGES.len()],
    pub ascension: u32,
    /// Equipment: the shared bag, each hero's 12 slots (item ids, 0 = empty), Spirit Dust,
    /// Shrine Draw pity and the next item id.
    pub items: Vec<Item>,
    pub loadouts: [[u32; 12]; ALL_HEROES.len()],
    pub dust: u32,
    pub draw_pity: u32,
    pub next_item: u32,
    /// Settings (0..=1).
    pub music_volume: f32,
    pub sfx_volume: f32,
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
            enemy_kills: [0; ALL_ENEMIES.len()],
            weapons_found: Vec::new(),
            awakenings_found: Vec::new(),
            passives_found: Vec::new(),
            shrines_found: Vec::new(),
            wins: 0,
            deaths: 0,
            dawn_outs: 0,
            best_kills: 0,
            time_played: 0.0,
            sen_earned: 0,
            chests_opened: 0,
            shrines_taken: 0,
            awakenings: 0,
            specials_used: 0,
            hero_runs: [0; ALL_HEROES.len()],
            hero_wins: [0; ALL_HEROES.len()],
            hero_best: [0.0; ALL_HEROES.len()],
            stage: StageId::ShrineValley,
            difficulty: Difficulty::Normal,
            cleared: [[false; 3]; ALL_STAGES.len()],
            stage_best: [[0.0; 3]; ALL_STAGES.len()],
            stage_bosses: [0; ALL_STAGES.len()],
            relics: Vec::new(),
            asc_open: [0; ALL_STAGES.len()],
            ascension: 0,
            items: Vec::new(),
            loadouts: [[0; 12]; ALL_HEROES.len()],
            dust: 0,
            draw_pity: 0,
            next_item: 1,
            music_volume: 0.6,
            sfx_volume: 0.8,
        }
    }
}

/// What a finished run gave the profile.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunReport {
    pub sen: u32,
    pub unlocked: Vec<HeroId>,
    /// New Collection entries this run.
    pub discovered: u32,
    /// First clear of this stage at this difficulty (its Sen bonus is in `sen`).
    pub first_clear: Option<Difficulty>,
    pub relic: Option<Relic>,
    /// A stage this clear opened.
    pub stage_unlocked: Option<StageId>,
    /// Equipment banked from the run (ids into the bag).
    pub gear: Vec<u32>,
    /// A new Ascension tier this clear opened.
    pub ascension_opened: Option<u32>,
}

/// Bag size; drops beyond it are salvaged into Spirit Dust automatically.
pub const BAG_SIZE: usize = 300;

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
        for r in &self.relics {
            b = b.plus(r.bonus());
        }
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

    // ───────────── Equipment ─────────────

    pub fn item(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Adds an item to the bag (or salvages it when the bag is full). Returns its id (0 if salvaged).
    pub fn add_item(&mut self, mut it: Item) -> u32 {
        if self.items.len() >= BAG_SIZE {
            self.dust += it.salvage();
            return 0;
        }
        it.id = self.next_item;
        self.next_item += 1;
        let id = it.id;
        self.items.push(it);
        id
    }

    /// Who is wearing item `id`: (hero index, loadout position).
    pub fn worn_by(&self, id: u32) -> Option<(usize, usize)> {
        for (h, l) in self.loadouts.iter().enumerate() {
            if let Some(p) = l.iter().position(|x| *x == id) {
                return Some((h, p));
            }
        }
        None
    }

    /// Equips `id` on `hero` at loadout position `pos` (moving it off anyone else).
    pub fn equip(&mut self, hero: HeroId, pos: usize, id: u32) -> bool {
        let Some(it) = self.item(id) else { return false };
        if pos >= LOADOUT.len() || LOADOUT[pos] != it.slot {
            return false;
        }
        if let Some((h, p)) = self.worn_by(id) {
            self.loadouts[h][p] = 0;
        }
        self.loadouts[hero.index()][pos] = id;
        true
    }

    pub fn unequip(&mut self, hero: HeroId, pos: usize) {
        if pos < LOADOUT.len() {
            self.loadouts[hero.index()][pos] = 0;
        }
    }

    /// Where `slot` goes on `hero`: the first empty matching position, else the first match.
    pub fn slot_for(&self, hero: HeroId, slot: Slot) -> usize {
        let l = &self.loadouts[hero.index()];
        let matching: Vec<usize> = (0..LOADOUT.len()).filter(|&p| LOADOUT[p] == slot).collect();
        matching.iter().copied().find(|&p| l[p] == 0).unwrap_or(matching[0])
    }

    pub fn loadout(&self, hero: HeroId) -> Loadout {
        let items: Vec<&Item> = self.loadouts[hero.index()].iter().filter_map(|id| if *id == 0 { None } else { self.item(*id) }).collect();
        Loadout::from_items(&items)
    }

    /// Breaks an item into Spirit Dust (locked items are kept). Unequips it first.
    pub fn salvage(&mut self, id: u32) -> u32 {
        let Some(i) = self.items.iter().position(|x| x.id == id) else { return 0 };
        if self.items[i].locked {
            return 0;
        }
        if let Some((h, p)) = self.worn_by(id) {
            self.loadouts[h][p] = 0;
        }
        let d = self.items.swap_remove(i).salvage();
        self.dust += d;
        d
    }

    /// Salvages every unlocked, unworn item at or below `max` rarity.
    pub fn salvage_below(&mut self, max: Rarity) -> u32 {
        let ids: Vec<u32> = self.items.iter().filter(|i| i.rarity <= max && !i.locked && self.worn_by(i.id).is_none()).map(|i| i.id).collect();
        ids.into_iter().map(|id| self.salvage(id)).sum()
    }

    pub fn upgrade(&mut self, id: u32) -> bool {
        let Some(i) = self.items.iter().position(|x| x.id == id) else { return false };
        let cost = self.items[i].upgrade_cost();
        if self.items[i].plus >= gear::MAX_PLUS || self.dust < cost {
            return false;
        }
        self.dust -= cost;
        self.items[i].plus += 1;
        true
    }

    pub fn reroll(&mut self, id: u32, affix: usize, seed: u64) -> bool {
        let Some(i) = self.items.iter().position(|x| x.id == id) else { return false };
        let cost = self.items[i].reroll_cost();
        if self.dust < cost || affix >= self.items[i].affixes.len() {
            return false;
        }
        self.dust -= cost;
        let mut rng = Rng::new(seed);
        self.items[i].reroll(affix, &mut rng);
        true
    }

    /// Highest item level this profile has earned access to (Shrine Draw item level).
    pub fn draw_ilvl(&self) -> u16 {
        let mut best = gear::drop_ilvl(StageId::Shotengai, 0, 0);
        for (si, st) in ALL_STAGES.iter().enumerate() {
            for d in 0..3 {
                if self.cleared[si][d] {
                    let asc = if d == 2 { self.asc_open[si] } else { 0 };
                    best = best.max(gear::drop_ilvl(*st, d, asc));
                }
            }
        }
        best
    }

    /// Shrine Draw: `n` draws for Sen. Returns the new item ids (empty if unaffordable).
    pub fn draw(&mut self, n: u32, seed: u64) -> Vec<u32> {
        let cost = if n >= 10 { gear::DRAW10_COST } else { gear::DRAW_COST * n };
        if self.sen < cost {
            return Vec::new();
        }
        self.sen -= cost;
        let mut rng = Rng::new(seed);
        let ilvl = self.draw_ilvl();
        let mut out = Vec::new();
        for _ in 0..n {
            let r = gear::draw_rarity(&mut rng, self.draw_pity);
            self.draw_pity = if r >= Rarity::Legendary { 0 } else { self.draw_pity + 1 };
            let it = gear::roll_item(&mut rng, None, r, ilvl, None);
            out.push(self.add_item(it));
        }
        out
    }

    pub fn ascension_open(&self, s: StageId) -> u32 {
        if self.cleared[s.index()][2] {
            self.asc_open[s.index()].max(1)
        } else {
            0
        }
    }

    pub fn stage_open(&self, s: StageId) -> bool {
        s.requires().is_none_or(|r| self.cleared[r.index()][0])
    }

    /// Normal is open with the stage; Hard after a Normal clear; Nightmare after Hard.
    pub fn difficulty_open(&self, s: StageId, d: Difficulty) -> bool {
        self.stage_open(s) && (d == Difficulty::Normal || self.cleared[s.index()][d.index() - 1])
    }

    /// Folds a finished run into the profile: banks Sen, updates records, unlocks heroes.
    pub fn record_run(&mut self, w: &World) -> RunReport {
        debug_assert!(w.state != RunState::Playing);
        self.runs += 1;
        let (stage, diff) = (w.cfg.stage, w.cfg.difficulty);
        let won = w.state == RunState::Won;
        let mut report = RunReport { sen: w.sen, ..Default::default() };
        {
            let si = stage.index();
            self.stage_best[si][diff.index()] = self.stage_best[si][diff.index()].max(w.time);
            self.stage_bosses[si] |= w.bosses_down;
            if won && !self.cleared[si][diff.index()] {
                let opened_before: Vec<bool> = ALL_STAGES.iter().map(|s| self.stage_open(*s)).collect();
                self.cleared[si][diff.index()] = true;
                report.first_clear = Some(diff);
                report.sen += diff.first_clear_sen();
                if diff == Difficulty::Nightmare {
                    if let Some(r) = stage.def().relic {
                        if !self.relics.contains(&r) {
                            self.relics.push(r);
                            report.relic = Some(r);
                        }
                    }
                }
                report.stage_unlocked = ALL_STAGES.iter().enumerate().find(|(i, s)| !opened_before[*i] && self.stage_open(**s)).map(|(_, s)| *s);
            }
        }
        self.sen += report.sen;
        // Ascension: a Nightmare+N clear opens N+1.
        if won && diff == Difficulty::Nightmare {
            let si = stage.index();
            let next = w.cfg.ascension + 1;
            if next > self.asc_open[si] {
                self.asc_open[si] = next;
                report.ascension_opened = Some(next);
            }
        }
        for it in &w.loot {
            let id = self.add_item(it.clone());
            if id != 0 {
                report.gear.push(id);
            }
        }
        self.total_kills += w.kills as u64;
        self.best_time = self.best_time.max(w.time);
        self.best_level = self.best_level.max(w.player.level);
        self.great_onis += w.great_oni_defeated as u32;
        // Hero feats that name the Night Lord need the prologue's Night Lord.
        let night_lord = won && stage == StageId::Shotengai;
        self.night_lords += night_lord as u32;
        self.wins += won as u32;
        self.deaths += (!won && !w.dawn_out) as u32;
        self.dawn_outs += w.dawn_out as u32;
        self.best_kills = self.best_kills.max(w.kills);
        self.time_played += w.time;
        self.sen_earned += w.sen as u64;
        self.chests_opened += w.chests_opened;
        self.shrines_taken += w.shrines_taken.len() as u32;
        self.awakenings += w.weapons.iter().filter(|x| x.awakened).count() as u32;
        self.specials_used += w.specials_used;
        let hi = w.cfg.hero.index();
        self.hero_runs[hi] += 1;
        self.hero_wins[hi] += won as u32;
        self.hero_best[hi] = self.hero_best[hi].max(w.time);
        for (i, k) in w.kills_by_kind.iter().enumerate() {
            report.discovered += (self.enemy_kills[i] == 0 && *k > 0) as u32;
            self.enemy_kills[i] += *k as u64;
        }
        fn add<T: PartialEq + Copy>(set: &mut Vec<T>, v: T, n: &mut u32) {
            if !set.contains(&v) {
                set.push(v);
                *n += 1;
            }
        }
        for wpn in &w.weapons {
            add(&mut self.weapons_found, wpn.kind, &mut report.discovered);
            if wpn.awakened {
                add(&mut self.awakenings_found, wpn.kind, &mut report.discovered);
            }
        }
        for (k, _) in &w.passives {
            add(&mut self.passives_found, *k, &mut report.discovered);
        }
        for ev in &w.shrines_taken {
            add(&mut self.shrines_found, *ev, &mut report.discovered);
        }
        // Each hero is unlocked by a distinct feat performed as a specific hero, so the
        // roster opens up as a chain. (Future stages can add stage-specific feats here.)
        let as_hero = w.cfg.hero;
        let awakened = w.weapons.iter().any(|x| x.awakened);
        for h in ALL_HEROES {
            let earned = match h {
                HeroId::Ren => true,
                HeroId::Riku => as_hero == HeroId::Ren && won && stage == StageId::Shotengai,
                HeroId::Shun => as_hero == HeroId::Riku && won && stage == StageId::ShrineValley,
                HeroId::Toya => as_hero == HeroId::Shun && won && stage == StageId::PaleCitadel,
                HeroId::Kaede => as_hero == HeroId::Ren && awakened,
                HeroId::Mika => as_hero == HeroId::Riku && night_lord,
                HeroId::Haru => as_hero == HeroId::Toya && w.chests_opened >= 10,
                HeroId::Nanami => as_hero == HeroId::Shun && w.shrines_taken.len() >= 3,
                HeroId::Yuzu => as_hero == HeroId::Toya && w.player.level >= 60,
                HeroId::Kuon => as_hero == HeroId::Toya && won && stage == StageId::Rainward,
            };
            if earned && !self.is_unlocked(h) {
                self.unlocked.push(h);
                report.unlocked.push(h);
                report.discovered += 1;
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
        for (i, k) in ALL_ENEMIES.iter().enumerate() {
            s += &format!("enemy.{}={}\n", key_of(k), self.enemy_kills[i]);
        }
        s += &format!("weapons={}\n", join(&self.weapons_found));
        s += &format!("awakenings={}\n", join(&self.awakenings_found));
        s += &format!("passives={}\n", join(&self.passives_found));
        s += &format!("shrines={}\n", join(&self.shrines_found));
        s += &format!("music_volume={}\nsfx_volume={}\n", self.music_volume, self.sfx_volume);
        s += &format!(
            "wins={}\ndeaths={}\ndawn_outs={}\nbest_kills={}\ntime_played={}\nsen_earned={}\nchests={}\nshrines_taken={}\nawakened_total={}\nspecials={}\n",
            self.wins, self.deaths, self.dawn_outs, self.best_kills, self.time_played, self.sen_earned, self.chests_opened, self.shrines_taken, self.awakenings, self.specials_used
        );
        for (i, h) in ALL_HEROES.iter().enumerate() {
            s += &format!("hero.{}={},{},{}\n", h.key(), self.hero_runs[i], self.hero_wins[i], self.hero_best[i]);
        }
        s += &format!("map={}\ndifficulty={}\n", self.stage.key(), self.difficulty.key());
        for (i, st) in ALL_STAGES.iter().enumerate() {
            let c = self.cleared[i];
            let b = self.stage_best[i];
            s += &format!("stage.{}={},{},{},{},{},{},{}\n", st.key(), c[0] as u8, c[1] as u8, c[2] as u8, b[0], b[1], b[2], self.stage_bosses[i]);
        }
        let relics: Vec<&str> = self.relics.iter().map(|r| r.key()).collect();
        s += &format!("relics={}\n", relics.join(","));
        let asc: Vec<String> = self.asc_open.iter().map(|a| a.to_string()).collect();
        s += &format!("asc_open={}\nascension={}\ndust={}\ndraw_pity={}\nnext_item={}\n", asc.join(","), self.ascension, self.dust, self.draw_pity, self.next_item);
        for it in &self.items {
            s += &format!("item={}\n", it.serialize());
        }
        for (i, h) in ALL_HEROES.iter().enumerate() {
            if self.loadouts[i].iter().any(|x| *x != 0) {
                let ids: Vec<String> = self.loadouts[i].iter().map(|x| x.to_string()).collect();
                s += &format!("loadout.{}={}\n", h.key(), ids.join(","));
            }
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
                "weapons" => p.weapons_found = split(v, &ALL_WEAPONS),
                "awakenings" => p.awakenings_found = split(v, &ALL_WEAPONS),
                "passives" => p.passives_found = split(v, &ALL_PASSIVES),
                "shrines" => p.shrines_found = split(v, &ALL_SHRINE_EVENTS),
                "wins" => p.wins = v.parse().unwrap_or(0),
                "deaths" => p.deaths = v.parse().unwrap_or(0),
                "dawn_outs" => p.dawn_outs = v.parse().unwrap_or(0),
                "best_kills" => p.best_kills = v.parse().unwrap_or(0),
                "time_played" => p.time_played = v.parse().unwrap_or(0.0),
                "sen_earned" => p.sen_earned = v.parse().unwrap_or(0),
                "chests" => p.chests_opened = v.parse().unwrap_or(0),
                "shrines_taken" => p.shrines_taken = v.parse().unwrap_or(0),
                "awakened_total" => p.awakenings = v.parse().unwrap_or(0),
                "specials" => p.specials_used = v.parse().unwrap_or(0),
                "asc_open" => {
                    for (i, x) in v.split(',').enumerate().take(ALL_STAGES.len()) {
                        p.asc_open[i] = x.parse().unwrap_or(0);
                    }
                }
                "ascension" => p.ascension = v.parse().unwrap_or(0),
                "dust" => p.dust = v.parse().unwrap_or(0),
                "draw_pity" => p.draw_pity = v.parse().unwrap_or(0),
                "next_item" => p.next_item = v.parse().unwrap_or(1).max(1),
                "item" => {
                    if let Some(it) = Item::parse(v) {
                        p.next_item = p.next_item.max(it.id + 1);
                        p.items.push(it);
                    }
                }
                "map" => {
                    if let Some(st) = StageId::from_key(v) {
                        p.stage = st;
                    }
                }
                "difficulty" => {
                    if let Some(d) = Difficulty::from_key(v) {
                        p.difficulty = d;
                    }
                }
                "relics" => {
                    for r in v.split(',').filter_map(Relic::from_key) {
                        if !p.relics.contains(&r) {
                            p.relics.push(r);
                        }
                    }
                }
                "music_volume" => p.music_volume = v.parse::<f32>().unwrap_or(0.6).clamp(0.0, 1.0),
                "sfx_volume" => p.sfx_volume = v.parse::<f32>().unwrap_or(0.8).clamp(0.0, 1.0),
                _ => {
                    if let Some(key) = k.strip_prefix("hero.") {
                        if let Some(h) = HeroId::from_key(key) {
                            let f: Vec<&str> = v.split(',').collect();
                            let i = h.index();
                            p.hero_runs[i] = f.first().and_then(|x| x.parse().ok()).unwrap_or(0);
                            p.hero_wins[i] = f.get(1).and_then(|x| x.parse().ok()).unwrap_or(0);
                            p.hero_best[i] = f.get(2).and_then(|x| x.parse().ok()).unwrap_or(0.0);
                        }
                    }
                    if let Some(h) = k.strip_prefix("loadout.").and_then(HeroId::from_key) {
                        for (i, x) in v.split(',').enumerate().take(12) {
                            p.loadouts[h.index()][i] = x.parse().unwrap_or(0);
                        }
                    }
                    if let Some(st) = k.strip_prefix("stage.").and_then(StageId::from_key) {
                        let f: Vec<&str> = v.split(',').collect();
                        let i = st.index();
                        for d in 0..3 {
                            p.cleared[i][d] = f.get(d).is_some_and(|x| *x == "1");
                            p.stage_best[i][d] = f.get(3 + d).and_then(|x| x.parse().ok()).unwrap_or(0.0);
                        }
                        p.stage_bosses[i] = f.get(6).and_then(|x| x.parse().ok()).unwrap_or(0);
                    }
                    if let Some(key) = k.strip_prefix("enemy.") {
                        if let Some(i) = ALL_ENEMIES.iter().position(|e| key_of(e) == key) {
                            p.enemy_kills[i] = v.parse().unwrap_or(0);
                        }
                    }
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
        if !p.difficulty_open(p.stage, p.difficulty) {
            p.difficulty = Difficulty::Normal;
        }
        if !p.stage_open(p.stage) {
            p.stage = StageId::ShrineValley;
        }
        // Drop loadout ids that no longer exist or sit in the wrong slot.
        for h in 0..ALL_HEROES.len() {
            for pos in 0..LOADOUT.len() {
                let id = p.loadouts[h][pos];
                if id != 0 && !p.items.iter().any(|i| i.id == id && i.slot == LOADOUT[pos]) {
                    p.loadouts[h][pos] = 0;
                }
            }
        }
        p
    }
}

/// Save key for an enum value: its variant name in lower case (never rename variants).
fn key_of<T: std::fmt::Debug>(v: &T) -> String {
    format!("{v:?}").to_lowercase()
}

fn join<T: std::fmt::Debug>(items: &[T]) -> String {
    items.iter().map(key_of).collect::<Vec<_>>().join(",")
}

fn split<T: std::fmt::Debug + Copy>(v: &str, all: &[T]) -> Vec<T> {
    v.split(',').filter_map(|k| all.iter().find(|x| key_of(*x) == k).copied()).collect()
}

/// Discovered / total entries in the Collection.
pub fn collection_progress(p: &Profile) -> (usize, usize) {
    let found = p.enemy_kills.iter().filter(|k| **k > 0).count()
        + p.weapons_found.len()
        + p.awakenings_found.len()
        + p.passives_found.len()
        + p.shrines_found.len()
        + p.unlocked.len()
        + p.relics.len();
    let total = ALL_ENEMIES.len() + 2 * ALL_WEAPONS.len() + ALL_PASSIVES.len() + ALL_SHRINE_EVENTS.len() + ALL_HEROES.len() + ALL_RELICS.len();
    (found, total)
}

pub fn enemy_found(p: &Profile, k: EnemyKind) -> bool {
    p.enemy_kills[k.index()] > 0
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
        p.enemy_kills[2] = 77;
        p.weapons_found = vec![WeaponKind::Ofuda, WeaponKind::Taiko];
        p.awakenings_found = vec![WeaponKind::Taiko];
        p.passives_found = vec![PassiveKind::Guard];
        p.shrines_found = vec![ShrineEvent::GlassBlade];
        p.music_volume = 0.25;
        p.wins = 2;
        p.time_played = 1234.5;
        p.hero_runs[1] = 7;
        p.hero_best[1] = 612.0;
        p.cleared[1] = [true, true, false];
        p.stage_best[1][1] = 1500.5;
        p.stage_bosses[1] = 7;
        p.relics = vec![Relic::WardKey];
        p.stage = StageId::ShrineValley;
        p.difficulty = Difficulty::Nightmare;
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
        w.kills = 2_500;
        w.sen = 40;
        w.state = RunState::Won;
        let r = p.record_run(&w);
        assert_eq!(r.unlocked, vec![HeroId::Riku], "clearing the Shotengai as Ren unlocks Riku");
        assert!(p.weapons_found.contains(&WeaponKind::Ofuda));
        assert!(r.discovered >= 2, "Riku + Ofuda");
        assert_eq!(p.sen, 40 + Difficulty::Normal.first_clear_sen());
        assert!(p.record_run(&w).unlocked.is_empty(), "only unlocks once");
        // The same feat as another hero doesn't count for that unlock.
        let mut w = World::new(WorldConfig { hero: HeroId::Riku, ..Default::default() });
        w.state = RunState::Won;
        assert_eq!(p.record_run(&w).unlocked, vec![HeroId::Mika], "the Night Lord as Riku unlocks Mika, not Shun");
        let mut w = World::new(WorldConfig { hero: HeroId::Riku, stage: StageId::ShrineValley, ..Default::default() });
        w.state = RunState::Won;
        assert_eq!(p.record_run(&w).unlocked, vec![HeroId::Shun], "Shrine Valley as Riku unlocks Shun");
    }
}

#[cfg(test)]
mod campaign_tests {
    use super::*;
    use crate::WorldConfig;

    #[test]
    fn clears_open_stages_difficulties_and_relics() {
        let mut p = Profile::default();
        assert!(p.stage_open(StageId::Shotengai) && p.stage_open(StageId::ShrineValley));
        assert!(!p.stage_open(StageId::EmberLeaf));
        assert!(!p.difficulty_open(StageId::ShrineValley, Difficulty::Hard));
        let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, ..Default::default() });
        w.state = RunState::Won;
        w.time = 1520.0;
        let r = p.record_run(&w);
        assert_eq!(r.first_clear, Some(Difficulty::Normal));
        assert_eq!(r.stage_unlocked, Some(StageId::EmberLeaf));
        assert_eq!(p.sen, Difficulty::Normal.first_clear_sen());
        assert!(p.difficulty_open(StageId::ShrineValley, Difficulty::Hard));
        assert!(p.record_run(&w).first_clear.is_none(), "only the first clear pays");
        let mut w = World::new(WorldConfig { stage: StageId::ShrineValley, difficulty: Difficulty::Nightmare, ..Default::default() });
        w.state = RunState::Won;
        assert_eq!(p.record_run(&w).relic, Some(Relic::MoonveilBell));
        assert!(p.bonus().area > 0.09);
        // A world win is not a Night Lord.
        assert_eq!(p.night_lords, 0);
        assert_eq!(p.wins, 3);
    }
}

#[cfg(test)]
mod gear_tests {
    use super::*;
    use crate::WorldConfig;

    #[test]
    fn loot_banks_equips_saves_and_crafts() {
        let mut p = Profile { sen: 10_000, ..Default::default() };
        let mut w = World::new(WorldConfig::default());
        let mut rng = Rng::new(5);
        w.loot.push(gear::roll_item(&mut rng, Some(Slot::Ring), Rarity::Epic, 10, None));
        w.loot.push(gear::roll_item(&mut rng, Some(Slot::Ring), Rarity::Legendary, 10, None));
        w.state = RunState::Dead;
        let r = p.record_run(&w);
        assert_eq!(r.gear.len(), 2);
        let (a, b) = (r.gear[0], r.gear[1]);
        assert!(p.equip(HeroId::Ren, p.slot_for(HeroId::Ren, Slot::Ring), a));
        assert!(p.equip(HeroId::Ren, p.slot_for(HeroId::Ren, Slot::Ring), b), "second ring slot");
        assert!(!p.equip(HeroId::Ren, 0, a), "a ring doesn't fit the hat slot");
        let l = p.loadout(HeroId::Ren);
        assert!(l.bonus.crit_chance > 0.0 && l.uniques.len() == 1);
        p.dust = 10_000;
        assert!(p.upgrade(a) && p.reroll(b, 0, 99));
        let drawn = p.draw(10, 3);
        assert_eq!(drawn.len(), 10);
        assert_eq!(p.sen, 10_000 - gear::DRAW10_COST);
        let back = Profile::parse(&p.serialize());
        assert_eq!(back.items.len(), p.items.len());
        assert_eq!(back.loadouts, p.loadouts);
        assert_eq!(back.dust, p.dust);
        assert_eq!(back.item(a).unwrap().plus, 1);
        // The hero's gear reaches the run.
        let w = World::new(WorldConfig { gear: p.loadout(HeroId::Ren), ..Default::default() });
        assert!(w.bonus.crit_chance > 0.0);
    }

    #[test]
    fn arsenal_grants_a_slotless_weapon() {
        let mut rng = Rng::new(2);
        let mut it = gear::roll_item(&mut rng, Some(Slot::Arsenal), Rarity::Mythic, 20, None);
        it.arsenal = Some(WeaponKind::Familiars);
        let w = World::new(WorldConfig { gear: gear::Loadout::from_items(&[&it]), ..Default::default() });
        let extra = w.weapons.iter().find(|x| x.kind == WeaponKind::Familiars).unwrap();
        assert!(extra.extra && extra.awakened && extra.level == 8);
    }
}
