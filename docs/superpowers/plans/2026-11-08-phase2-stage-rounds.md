# Phase 2 — Stage Rounds Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every stage runs as 20 time-based rounds with a headline every 4 rounds, steps difficulty up per round, and shows "Round N / 20" on a restyled progress bar.

**Architecture:** Rounds are derived from the existing `StageDef::final_at` (rounds 1–19 split `[0, final_at]` evenly; round 20 starts at `final_at` and lasts until the final boss dies or dawn). Stage data moves from absolute times to round numbers; the director fires events at round starts. Difficulty curves keep their shapes but read a stepped pace (`pace` at the current round's start) plus a bump on headline rounds. The client reads rounds from the sim.

**Tech Stack:** Rust 2021, deterministic `crates/sim` (unit tests + headless balance bot), macroquad `crates/client`.

**Spec:** `docs/superpowers/specs/2026-11-07-elements-rarity-design.md` — section 4 "Stage rounds and the progress bar" (and "Attack reach" for the current balance levers).

## Global Constraints

- **No commits.** The user commits. Each task ends with "show the diff for review".
- Rounds are **time-based**; enemies keep flowing as today.
- 20 rounds; rounds 1–19 split `[0, final_at]` equally (~47 s Shotengai, ~79 s 25-minute worlds); round 20 starts at `final_at`.
- Headlines: round 4 Uncommon wave, 8 Elite, 12 Leader, 16 Sub-boss (existing Mini), 20 Boss (existing Final); worlds with a mid Boss get it at round 18.
- Until phase 6, Uncommon and Leader headlines use existing enemies as stand-ins.
- The round-4 headline drops a treasure chest (replaces the early elite chest at 2:00).
- Dawn limit (`StageDef::limit`) unchanged.
- Rarity colours for headline tiers (spec §5): Uncommon green, Elite blue, Leader purple, Sub-boss orange, Boss red.
- UI: virtual pixels only; `kit::MARGIN` (24) from every edge; the game font has no '⚡', '←', '→'.
- Sim stays deterministic; existing balance levers (`XP_GAIN`, `MIGHT_MELEE`, `BOSS_HP`, reach constants) unchanged.
- README sections this phase changes get updated (Campaign night structure).

## Review Focus

1. Time exactly at a round boundary (e.g. `t == final_at`, `t == round_start(16)`) → belongs to the new round; boss spawns once, not twice or never (Task 1 tests).
2. Shotengai has no mid Boss and no elite wave → round 18 does nothing special there; round 8 still introduces elites (Task 1 test).
3. Dev menu "Skip to mini-boss / second boss / final boss" still lands just before the right round (Task 3 step).
4. A run that reaches dawn with the final boss alive still loses exactly as today (Task 1 test on `limit`).
5. Phone landscape (844×390): the round label and bar stay inside `MARGIN` and don't overlap the SPACE prompt (Task 3 screenshot).

---

### Task 1: Round model, stage data in rounds, headline events

**Files:**
- Modify: `crates/sim/src/campaign.rs` — `StageDef` fields and methods (~L236–L285), `world()` (~L289–L320), `STAGES` (~L322+).
- Modify: `crates/sim/src/content.rs` — remove `MIDBOSS_AT`, `SHRINE_TIMES`, `SWARM_RING_TIMES`, `ELITE_EVERY` if only tests use them; add round constants.
- Modify: `crates/sim/src/world.rs` — `director` (~L1246–L1335), the world's `next_elite` init (~L869), events enum, tests that use the removed constants.
- Modify: `crates/client/src/main.rs` (dev skips ~L391–L397), `crates/client/src/menu.rs` (~L1103), `crates/client/src/ui.rs` (`stage_progress` marks ~L297–L316) — only to compile against the new fields; Task 3 restyles them.

**Interfaces:**
- Produces (in `campaign.rs`):
  - `pub const ROUNDS: u8 = 20;`
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Headline { Uncommon, Elite, Leader, SubBoss, Boss }`
  - `impl StageDef { pub fn round_len(&self) -> f32; pub fn round_at(&self, t: f32) -> u8; pub fn round_start(&self, r: u8) -> f32; pub fn headline(r: u8) -> Option<Headline> }` — `round_len = final_at / 19`; `round_at` returns 1..=20 (`t >= final_at` → 20); `round_start(20) == final_at` exactly; `headline` maps 4,8,12,16,20 → Uncommon, Elite, Leader, SubBoss, Boss.
  - `StageDef` field changes: drop `mini_at` (mini spawns at `round_start(16)`); `boss: Option<EnemyKind>` (spawns at `round_start(18)`); `elite_wave: bool` (replaces `elite_wave_at`, fires at round 8); `crisis: Option<(u8, u8)>` inclusive rounds (`in_crisis` keeps its signature, now via `round_at`); `shrine_rounds: &'static [u8]`; `ring_rounds: &'static [u8]` (replace `shrine_times` / `ring_times`; each fires at that round's start).
  - Data — Shotengai: shrines `[5, 9, 13, 17]`, rings `[7, 13, 18]`, no boss, no elite wave, no crisis. Worlds (`world()`): shrines `[3, 6, 9, 13, 16]`, rings `[4, 8, 12, 16, 18]`, `elite_wave: true`, crisis `(16, 19)`.
  - In `world.rs`: `pub fn round(&self) -> u8` (= `stage().round_at(self.time)`); `Event::RoundStarted { round: u8, headline: Option<Headline> }` emitted once at each round start (including round 1 at t=0).
- Headline content in `director`:
  - Round 4 (Uncommon stand-in): a ring of `st.fodder_b` (count `24 × difficulty.density()`), plus one chest at the player's position + (0, −60) with tier `chest_tier(st.elite)`.
  - Round 8 (Elite): first elite spawns; afterwards elites keep the existing `elite_interval` cadence (`next_elite` starts at `round_start(8)`, not `ELITE_EVERY`). Worlds also fire the existing elite wave here.
  - Round 12 (Leader stand-in): 3 × `st.elite` spawned together on the spawn ring.
  - Round 16: `st.mini`. Round 18: `st.boss` if any. Round 20: `st.final_boss` (existing `night_lord_spawned` logic, unchanged time).

- [ ] **Step 1: Write the failing tests** in `campaign.rs` tests (or `world.rs` where a `World` is needed):

```rust
#[test]
fn rounds_split_the_night() {
    let s = StageId::Shotengai.def();
    assert_eq!(s.round_at(0.0), 1);
    assert_eq!(s.round_at(s.final_at - 0.01), 19);
    assert_eq!(s.round_at(s.final_at), 20);
    assert_eq!(s.round_at(s.limit), 20);
    assert!((s.round_start(16) - 15.0 * s.final_at / 19.0).abs() < 1e-3);
    assert_eq!(s.round_start(20), s.final_at);
}

#[test]
fn headlines_every_four_rounds() {
    use Headline::*;
    let got: Vec<_> = (1..=20).filter_map(|r| StageDef::headline(r).map(|h| (r, h))).collect();
    assert_eq!(got, vec![(4, Uncommon), (8, Elite), (12, Leader), (16, SubBoss), (20, Boss)]);
}
```

  World-level tests (adapt to the module's existing test helpers for building a `World` on a stage):
  - `mini_spawns_at_round_16_once`: step to `round_start(16) - DT` → no mini alive; step past it → exactly one `st.mini`; keep stepping a few seconds → still one spawn event.
  - `mid_boss_at_round_18_only_on_worlds`: ShrineValley spawns its `boss` at round 18; Shotengai spawns nothing extra at round 18.
  - `elites_start_at_round_8`: no `st.elite` alive before `round_start(8)`; one appears at it.
  - `round_4_drops_a_chest`: crossing `round_start(4)` adds one entry to `w.chests` and spawns `fodder_b` enemies.
  - `round_events_fire_once_each`: stepping 0 → `final_at + 1` emits `RoundStarted` for rounds 1..=20 exactly once each, with `headline` matching `StageDef::headline`.
  - `dawn_still_loses`: with the final boss alive at `limit`, state becomes `Dead` as today (reuse/keep the existing dawn test if there is one).
- [ ] **Step 2: Run, expect FAIL** — `cargo test -p sim --release rounds_ headlines_ round_ mini_spawns elites_start mid_boss dawn_` → compile errors for the missing items.
- [ ] **Step 3: Implement** the interfaces and data above; update every existing test that used `MIDBOSS_AT`, `SHRINE_TIMES`, `ring_times`, `mini_at` etc. to the round API (keep their intent). Update the client call sites to compile (dev skips use `round_start(16)`, `round_start(18)`, `final_at`; menu uses `def.boss` as `Option<EnemyKind>`; `stage_progress` marks use `round_start` of shrine/headline rounds).
- [ ] **Step 4: Run, expect PASS** — the new tests, then full `cargo test -p sim --release`, `cargo test -p client`, `cargo build -p client --release` (0 warnings).
- [ ] **Step 5: Show the diff for review.**

### Task 2: Stepped difficulty per round and rebalance

**Files:**
- Modify: `crates/sim/src/world.rs` — every `st.pace(self.time)` / `stage().pace(self.time)` used for difficulty (`hp_scale` site ~L1162, `damage_scale` ~L1192, `director` ~L1248, chest luck ~L4137, sen pouch ~L4375).
- Modify: `crates/sim/src/content.rs` — `HEADLINE_BUMP`.

**Interfaces:**
- Consumes: `StageDef::round_at`, `round_start`, `headline` (Task 1).
- Produces: `pub fn round_pace(&self) -> f32` on `World` = `st.pace(st.round_start(self.round()))`; `pub const HEADLINE_BUMP: f32 = 1.15;` multiplies enemy HP and contact/bullet damage scale while the current round is a headline round.
- Keep `late_hp`'s `(time / final_at)` term as is (it already rises smoothly; changing it is not required by the spec).

- [ ] **Step 1: Failing test** `difficulty_steps_per_round`: on a stage, enemy HP scale sampled at `round_start(5) + 1` equals the value at `round_start(6) - 1` (flat within a round) and is lower than at `round_start(6) + 1`; HP scale during round 8 (headline) ≥ `HEADLINE_BUMP ×` the round-7 value.
- [ ] **Step 2: Run, expect FAIL.**
- [ ] **Step 3: Implement** `round_pace` and use it everywhere difficulty used `pace(self.time)`; apply `HEADLINE_BUMP` in the HP and damage scale functions. Leave `AWAKEN_FROM` (real seconds) unchanged.
- [ ] **Step 4: Run, expect PASS**, then the full sim and client suites.
- [ ] **Step 5: Balance.** Baseline = the phase 1 commit `5fa6b9c` (`git archive 5fa6b9c | tar -x -C <scratch>/baseline`, separate `CARGO_TARGET_DIR`). Bot: `BALANCE_SEEDS=10 BALANCE_SEED0=21 BALANCE_STAGE=shrine_valley BALANCE_HERO=<h> cargo test --release -p sim balance -- --ignored --nocapture` for **ren, riku, shun**, plus `shotengai` for **ren** (5 seeds). Targets: wins within ±1 of baseline per hero (±1 of 5 on Shotengai); median death/clear time not earlier than baseline − 10%. Lever: `HEADLINE_BUMP` only (range 1.0–1.3). If no value hits every target, take the closest, stop, and report the table.
- [ ] **Step 6: Show the diff for review** with a before/after table.

### Task 3: Round progress bar, headline banner, README

**Files:**
- Modify: `crates/client/src/ui.rs` — `stage_progress` (~L292–L318) and its call in `hud` (~L245).
- Modify: `crates/client/src/render.rs` — event absorption (~L556 where `EliteWave`/`BossSpawned` set banners) for `RoundStarted`.
- Modify: `crates/client/src/audio.rs` (~L184) — headline round start plays `Sfx::Gong` (non-headline: silent).
- Modify: `README.md` — Campaign bullet "30-minute nights: …".

**Interfaces:**
- Consumes: `World::round()`, `StageDef::{round_start, headline, round_len}`, `Headline`, `Event::RoundStarted` (Task 1).
- Produces: `fn headline_color(h: Headline) -> Color` in `ui.rs` (Uncommon green `JADE`, Elite blue `SKY`, Leader purple `Color::new(0.72, 0.5, 1.0, 1.0)`, Sub-boss orange `Color::new(1.0, 0.62, 0.25, 1.0)`, Boss red `Color::new(1.0, 0.35, 0.35, 1.0)`); `fn headline_name(h: Headline) -> &'static str` ("UNCOMMON WAVE", "ELITE", "LEADER", "SUB-BOSS", "BOSS").

- [ ] **Step 1: Progress bar.** Replace the 6 px bar with a `kit::panel` strip, bottom-anchored at `screen_height() - kit::MARGIN - height`, width 460 (clamped to `screen_width() - 2 × MARGIN`), with 20 segments: past rounds filled, current round filling by time, future rounds empty; headline segments outlined/tinted with `headline_color`; shrine rounds get a small gold pip. Above the strip, left: **"Round 12 / 20"**; right: the next headline, e.g. "Leader in 1 round" / "Leader now" / after round 20 starts: "dawn in m:ss" (the existing dawn countdown; segment 20 drains toward dawn). The SPACE prompt and the alert banner must not overlap it — move the SPACE prompt up if needed.
- [ ] **Step 2: Headline banner.** On `RoundStarted { headline: Some(h), round }` show the existing banner (`fx.banner`) with `"ROUND {round} — {headline_name}"` for ~2.5 s; keep existing `BossSpawned`/`NightLordSpawned`/`EliteWave` banners from stacking on top in the same frame (headline banner wins). Gong on headline rounds.
- [ ] **Step 3: Dev skips** (main.rs) land 3 s before `round_start(16)`, `round_start(18)`, `final_at`; verify by reading the code.
- [ ] **Step 4: README** Campaign bullet → "**20 rounds a night**: every 4th round is a headline — an uncommon wave, the first elites, a leader, the sub-boss, and the final boss in round 20 — and the boss must fall before dawn." Keep the rest.
- [ ] **Step 5: Verify:** `cargo test -p client`, `cargo build -p client --release` (0 warnings), `./scripts/build-web.sh`; headless-Chrome screenshots of a run at 1280×720 and 844×390 (scratch dir, not committed) showing the bar and label, and one during a headline banner (use the dev menu "Skip to mini-boss" to reach round 16).
- [ ] **Step 6: Show the diff and screenshots for review.**
