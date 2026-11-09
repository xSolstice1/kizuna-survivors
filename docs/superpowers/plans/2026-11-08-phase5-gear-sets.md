# Phase 5 — Gear Elements, Set Rarity and Set Bonuses Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every item carries an element (damage bonus + resistance), Set becomes the top gear rarity with 2/4/5/6-piece bonuses dropped by a world's bosses, and old saves load with every item intact.

**Architecture:** `gear.rs` owns everything item-side: a stored `Item.element` (new save field 13, derived from the item id when missing), `Rarity::Set` (index 7, so old indices stay valid), the per-world piece lists, and `Loadout` gaining per-element damage / resistance arrays plus 5- and 6-piece state. `world.rs` reads the `Loadout`: `apply_hit` adds the gear element damage, `hurt_player` gains a source element for resistance, enemy shots remember their shooter's element, gear-driven hits stop passing `None`, the six 6-piece effects hook into the phase 4 status code, and Mini/Boss/Final kills roll Set drops. The client only draws.

**Tech Stack:** Rust 2021, deterministic `crates/sim` (unit tests + headless balance bot), macroquad `crates/client`.

**Spec:** `docs/superpowers/specs/2026-11-07-elements-rarity-design.md` — §6 "Gear", §7 "Set bonuses", §9 "What inherits an element" (Gear row), "Save compatibility", "Carried into phase 5".

## Global Constraints

- **No commits.** The user commits. Each task ends with "show the diff for review". No AI attribution anywhere.
- Gear tiers: Common < Uncommon < Rare < Epic < Legendary < Mythic < Exotic < **Set**. `Rarity::Set` is index 7; indices 0–6 keep their meaning in saves.
- Item element: C–Mythic roll a common element; Legendary+ may roll Light/Darkness (10%); Arsenal library weapon takes `WeaponKind::element`; Set takes its set's element; Exotics get a fixed one. Every item: "+X% <element> damage" and "<element> resistance".
- Resistance reduces damage from enemies of that element, **capped at 60% total** (`RESIST_CAP` 0.60), applied before armor. The hero has no defensive element.
- Set: only the 6 slots of its world's set (table in Task 3); 6 rolled bonuses at Exotic-level quality (`0.7 + 0.3 q`); stat multiplier above Exotic; one random unique from `ALL_UNIQUES`; priced above Exotic for salvage / upgrade / reroll.
- Rings come as a pair: **a Set's rings only count when both are worn (the pair counts as 2)**. Arsenal is never a set piece.
- 2-piece: existing `set_two`. 4-piece: existing 4-piece effect. 5-piece: +25% damage of the set's element and +25% resistance to it. 6-piece: one signature effect per set (Task 4 table).
- Newly rolled non-Set items carry no set tag. **Legacy set-tagged items** keep rarity and tag, count toward 2- and 4-piece only, never 5/6.
- Set drops: only Mini, Boss and Final kills in a world drop that world's pieces. Chance per eligible kill `SET_CHANCE × exotic_mult(difficulty, ascension) × (1 + gear_find)`, `SET_CHANCE` start 0.12, target ~40 Normal runs for all 6 distinct pieces. Duplicates still drop. Shrine Draw never gives Set.
- Old saves must load with every item intact; an item's derived element never changes between loads.
- Colour is never the only carrier: Set items show the label "Set", elements show their name.
- UI: virtual 1280×720 canvas via `screen_width()/screen_height()`, everything `kit::MARGIN` (24) from every edge; the font has no `⚡`, `←`, `→`.
- Bot is noisy (±2–4 wins at 10 seeds): baseline from `5f82b71` via `git archive 5f82b71 | tar -x -C <dir>` with its own `CARGO_TARGET_DIR`; same seeds 21–30 and 41–50. Use `bot_pick` in every test loop.
- Build test worlds with the hero in `WorldConfig` (`play_as`); never set `cfg.hero` afterwards (`hero_element` is cached at `new`).
- Record every number the user changes mid-flight in the spec and this plan.

## Element tables and rulings (draft — user signs off at plan review)

**Arsenal arts** (Legendary+ Arsenals take their art's element): RaijinDrums Electric, FoxProcession Fire, TenguFan Wind, DragonCoil Water, Moonfall Ice, SkeletonHand Darkness, YinYangBreaker Light, ShadowStar Darkness, GrassCutter Wind.

**Exotics** (fixed): LanternHeart Fire, JeweledBranch Light, UnbrokenYinYang Light, NineSeal Fire, CloneScroll Darkness, HollowMask Darkness, WhiteHaori Ice, HungerOrgan Darkness, ThousandCounter Water, SovereignFinger Darkness, ClosedHandVow Electric, Kusanagi Wind, YataMirror Light, Yasakani Earth.

**Gear-driven hits** (all currently `None`): Arsenal library weapon = its weapon's element (already); arts = art element; Exotic effects (Lantern ring, Nine-Seal foxfire, Jeweled Branch Last Word, Yata reflect, Thousand Counter blast) = the Exotic's element; Thunderclap Electric; Starburst Light; Ember Leaf 4-piece tags Wind; Rainward 4-piece lightning Electric. **Unchanged `None`:** stage hazards, pickups (`update_pickups`), friendly domains, Eclipse explosions (their amount already carries the crit hit's element multiplier — the phase 4 debt is closed by keeping it neutral so it is not multiplied twice).

**Player-hit sources:** contact = the element of the enemy dealing the worst contact hit; shots = the shooter's `EnemyKind::element` (boss patterns: the boss's); hazards `None` (no resistance applies).

**Other rulings:** element bonus per item = `ELEM_ITEM` 0.02 (halved from 0.04, 2026-11-09) × item scale (same scale as affixes: ilvl × rarity mult × plus); Set rarity mult 1.75, dust 800, affixes 6; Set items are named by set (`"Moonveil Silk Eboshi"`), the unique shows in the description; 5-piece resistance counts toward the 60% cap; **Set pieces are never auto-salvaged on a full bag** (they enter the bag over `BAG_SIZE`; every other drop keeps today's behaviour) — anti-goal "players losing items".

## Review Focus

1. An old save line (13 fields, no element) must load, keep every field, and derive the **same** element every load and across save → load → save (Task 1 test `old_items_get_a_stable_element`).
2. Resistance stacking from 12 items + a 5-piece must never pass 60%, and a hit from an enemy of another element must ignore it (Task 2 test `resistance_caps_and_matches_element`).
3. One Set ring worn alone must not count toward any tier, while a legacy ring alone still counts toward 2/4 as before (Task 3 test `set_rings_count_only_as_a_pair`).
4. A legacy set-tagged item plus 5 Set pieces gives 2/4 but not the 6-piece; 4 legacy + 2 Set gives 4-piece, not 5 (Task 3 test `legacy_pieces_stop_at_four`).
5. A Set piece dropped with a full bag lands in the bag, not in Dust (Task 5 test `set_pieces_survive_a_full_bag`).

---

### Task 1: Item element and save migration

**Files:**
- Modify: `crates/sim/src/gear.rs` — `Item.element`; `ArsenalArt::element`, `Exotic::element`; `pick_element`; `roll_item` / `roll_exotic`; `serialize` / `parse`; `Loadout.elem_damage` / `elem_resist`; tests.
- Modify: `crates/sim/src/content.rs` — `Element::key` and `Element::from_key` next to `name` (~L1105).
- Modify: `crates/client/src/armory.rs` — item card shows `"<Element> · +X% <Element> damage · Y% <Element> resistance"` in `element_color` (from `ui.rs`).

**Interfaces:**
- Produces:
  - `Element::key(self) -> &'static str` (lowercase name), `Element::from_key(&str) -> Option<Element>`.
  - `pub element: Element` on `Item`, serialized as field 13 (`Element::key`).
  - `ArsenalArt::element(self) -> Element`, `Exotic::element(self) -> Element` (tables above).
  - `fn pick_element(it: &Item, r: u32) -> Element` — the one rule for both new rolls (`r = rng.next_u32()`) and old saves (`r = stage::hash(id as i32, 7, 0xE1E)`): Exotic → fixed; Set → `set.element()` (Task 3 wires Set; here `rarity == Set` is unreachable); `arsenal: Some(k)` → `k.element()`; `art: Some(a)` → `a.element()`; Legendary+ → Light/Darkness when `r % 100 < 10` (split by bit), else common `r % 6`; below Legendary → common `r % 6`.
  - `pub const ELEM_ITEM: f32 = 0.02;` `Item::elem_value(&self) -> f32` = `ELEM_ITEM * scale`.
  - `Loadout { pub elem_damage: [f32; 8], pub elem_resist: [f32; 8], .. }`, indexed `e as usize`; `from_items` adds `elem_value` of each item to both.
  - `pub const RESIST_CAP: f32 = 0.60;` `Loadout::resist(&self, e: Element) -> f32` = `elem_resist[e].min(RESIST_CAP)`.

- [ ] **Step 1: Write the failing tests** in `gear.rs` tests:
  - `item_elements_follow_the_rules`: 2000 rolls per rarity Common…Mythic, any slot: below Legendary never `is_rare()`; Legendary+ non-Arsenal rare share in 5–15%; library Arsenal element == `arsenal.unwrap().element()`; art Arsenal == `art.unwrap().element()`; every `roll_exotic(e)` == `e.element()`; round trip keeps `element`.
  - `old_items_get_a_stable_element`: take a serialized item, drop field 13 (`rsplitn` the last `;`), parse twice → same element, equal to `pick_element(&it, hash(id))`; save → parse → save gives identical text; a 11-field line (pre-art saves) still parses.
  - `loadout_sums_item_elements`: three Fire items + one Ice → `elem_damage[Fire] ≈ sum of elem_value`, `resist(Ice) > 0`, `resist(Water) == 0`; twelve high-ilvl Fire items → `resist(Fire) == 0.60`.
- [ ] **Step 2: Run** `cargo test -p sim --release gear::` — FAIL (no `element` field).
- [ ] **Step 3: Implement** the Interfaces above. In `parse`, field 13 missing or unknown → `pick_element(&item, hash)` after the rest of the item is built.
- [ ] **Step 4: Run** `cargo test -p sim --release` — all pass (128 + 3); `cargo build -p client --release` — 0 warnings.
- [ ] **Step 5:** Show the diff for review.

### Task 2: Elements in combat (damage bonus, resistance, gear hits)

**Files:**
- Modify: `crates/sim/src/world.rs` — `apply_hit` (~L3000), `hurt_player` (~L1091) and its 4 callers (L1184, L1707, L4275, L4278 + tests), `Shot` (~L260) and its 8 `shots.push` sites, the gear hit sites listed in "Gear-driven hits".

**Interfaces:**
- Consumes: `Loadout.elem_damage`, `Loadout::resist`, `ArsenalArt::element`, `Exotic::element` (Task 1).
- Produces: `fn hurt_player(&mut self, raw: f32, elem: Option<Element>)`; `Shot { .., pub elem: Option<Element> }`.

- [ ] **Step 1: Write the failing tests** in `world.rs` tests:
  - `gear_element_damage_adds_to_hits`: world with a loadout of `elem_damage[Fire] = 0.5`; a Fire hit of 10 on a neutral-matchup enemy loses `10 * element_mult * 1.5` (compare against a zero-loadout world; no crit: `crit_chance` 0).
  - `resistance_caps_and_matches_element`: loadout `elem_resist[Fire] = 2.0`; `hurt_player(100.0, Some(Fire))` loses `100 * 0.4 - armor`; `hurt_player(100.0, Some(Water))` loses `100 - armor`; `None` same as Water.
  - `shots_carry_their_shooter_element`: spawn one shooting enemy on a stage, tick until a shot exists, assert `shot.elem == Some(kind.element())`.
  - `gear_hits_carry_an_element`: for each art in `ALL_ARTS` (world with that art equipped) and Thunderclap / Starburst / LanternHeart, tick until a `Hit` event; every `Hit` from it has `elem == Some(expected)` (follow phase 4's `every_weapon_hit_carries_its_element` pattern).
- [ ] **Step 2: Run** — FAIL (signature / field missing).
- [ ] **Step 3: Implement.** `apply_hit`: after the affinity line, `if let Some(e) = elem { amount *= 1.0 + self.cfg.gear.elem_damage[e as usize]; }`. `hurt_player`: `raw *= 1.0 - elem.map_or(0.0, |e| self.cfg.gear.resist(e))` before the form multipliers. `contact_damage` tracks the worst enemy's element with `worst`. Hazards pass `None`.
- [ ] **Step 4: Run** `cargo test -p sim --release` — all pass.
- [ ] **Step 5:** Show the diff for review.

### Task 3: Set rarity, pieces and 2/4/5/6 counting

**Files:**
- Modify: `crates/sim/src/gear.rs` — `Rarity::Set`, `ALL_RARITIES`, `name/mult/dust/arsenal_level/affixes`; `SET_PIECES`; `roll_set_piece`; `roll_item` stops tagging sets; `Item::name`; `set_text`; `Loadout::from_items` counting; `Loadout.sets6`.
- Modify: `crates/client/src/armory.rs` — `rarity_color(Rarity::Set)` (distinct from Exotic; label "Set"); salvage-below list keeps `take(6)`.

**Interfaces:**
- Produces:
  - `Rarity::Set` (index 7): name "Set", mult 1.75, dust 800, affixes 6, arsenal_level 8.
  - `pub fn set_pieces(s: StageId) -> [Slot; 6]` — spec §7 table verbatim: Shotengai Hat, Top, Bottom, Gloves, Shoes, Charm; ShrineValley Hat, Top, Necklace, Earring, Ring, Ring; EmberLeaf Top, Bottom, Gloves, Shoes, Ring, Ring; PaleCitadel Hat, Top, Bottom, Necklace, Earring, Omamori; Rainward Gloves, Shoes, Necklace, Omamori, Ring, Ring; Shatterline Hat, Gloves, Earring, Charm, Ring, Ring.
  - `pub fn roll_set_piece(rng: &mut Rng, s: StageId, ilvl: u16) -> Item` — uniform over the 6 entries (Ring twice), `rarity: Set`, `set: Some(s)`, 6 affixes at `0.7 + 0.3 q`, random unique, `element: s.element()` (also via `pick_element`).
  - `set_text(s) -> [&'static str; 4]` (2, 4, 5, 6 text; 5 = `"+25% <Element> damage and +25% <Element> resistance."`, 6 = Task 4 copy).
  - `Loadout { pub sets6: Vec<StageId>, .. }`, `Loadout::has_set6(s) -> bool`; `has_set` (4-piece) unchanged.
  - `pub fn set_count(items: &[&Item], s: StageId) -> (u32, u32)` = (legacy pieces, Set pieces after the ring rule), used by `from_items` and the armory panel.
- Counting: `legacy` = items with `set == Some(s)` and `rarity < Set` (each counts, as today); `new` = Set-rarity non-ring pieces + (2 if both Set rings of `s` worn else 0). 2-piece and 4-piece use `legacy + new`; 5-piece and 6-piece use `new` only. 5-piece adds 0.25 to `elem_damage[s.element()]` and `elem_resist[s.element()]`.

- [ ] **Step 1: Write the failing tests:**
  - `set_pieces_match_the_spec`: each `set_pieces(s)` has 6 entries, no Arsenal, Ring count 0 or 2; `roll_set_piece` over 600 rolls only gives those slots, all `Set`, element `s.element()`, unique `Some`, qualities ≥ 0.7; round trip keeps `Rarity::Set`; `roll_item` never sets `set` any more; `roll_rarity` and `draw_rarity` never return `Set` (20 000 rolls).
  - `set_rings_count_only_as_a_pair`: ShrineValley Set Hat, Top, Necklace, Earring + one Ring → 4-piece yes, 5-piece no (`elem_damage[Earth]` has no 0.25); add second Ring → 6-piece. Legacy Ring alone with 1 legacy Hat → 2-piece applies (`area` +0.15).
  - `legacy_pieces_stop_at_four`: 1 legacy + 5 Set (Shotengai) → `has_set` yes, `has_set6` no, no 0.25; 4 legacy + 2 Set → 4-piece yes, 5 no.
  - `set_prices_above_exotic`: `Rarity::Set.dust() > Rarity::Exotic.dust()`; a Set item's `upgrade_cost`, `reroll_cost`, `salvage` exceed an Exotic of the same ilvl.
- [ ] **Step 2: Run** — FAIL.
- [ ] **Step 3: Implement** the Interfaces. Update the existing test `sets_and_uniques_reach_the_loadout` (it builds legacy Legendary set items — keep it as the legacy regression).
- [ ] **Step 4: Run** `cargo test -p sim --release`, `cargo test -p client`, `cargo build -p client --release` (0 warnings: every `match` on `Rarity` covers `Set`).
- [ ] **Step 5:** Show the diff for review.

### Task 4: The six 6-piece effects

**Files:**
- Modify: `crates/sim/src/world.rs` — `hurt_enemy` / `on_hit_effect` / `apply_hit` and the kill handler (~L3425); cache `set6: [bool; 6]` (by `StageId::index`) at `World::new` like `hero_element`.
- Modify: `crates/sim/src/gear.rs` — 6-piece text in `set_text`.
- Modify: `crates/sim/src/content.rs` — constants below.

**Effects (draft — user signs off):**

| Set | 6-piece | Text |
|---|---|---|
| Lantern Street (Fire) | A burning enemy that dies passes its remaining burn to the nearest enemy in reach | "Burning Otherworlders pass their flames on when they fall." |
| Moonveil (Earth) | Staggered enemies take `SET6_STAGGER` 1.3× damage | "Staggered Otherworlders take 30% more damage." |
| Ember Leaf (Wind) | Wind hits roll their on-hit effect at `2 × PROC_CHANCE` | "Wind hits trigger their effect twice as often." |
| Pale Gate (Ice) | A slowed enemy that dies shatters: Ice hit of `SET6_SHATTER` 0.5 × its max HP (capped at 200 × `gear_power(1.0)`) to enemies within `SET6_SHATTER_R` 60 | "Slowed Otherworlders shatter when they fall, hurting those nearby." |
| Rainward (Water) | Every Water hit soaks (no roll needed for the soak) | "Every Water hit soaks. Soaked Otherworlders take 30% more Electric damage." |
| Shatterline (Electric) | An Electric chain jumps to one more target (2 total), still never from a chained hit | "Electric chains jump to one more Otherworlder." |

- [ ] **Step 1: Write the failing tests** (one each, world built with a loadout of 6 Set pieces of that world, `rng` effects forced where needed by setting the status directly):
  - `lantern_six_passes_burn_on_death`, `moonveil_six_boosts_staggered`, `ember_six_doubles_wind_procs` (10 000 Wind hits: proc share 25–35%), `pale_six_shatters_slowed` (neighbour within 60 takes Ice damage, one outside reach takes none), `rainward_six_always_soaks`, `shatterline_six_chains_twice` (two distinct targets, all in reach, no chain from the chained hits).
- [ ] **Step 2: Run** — FAIL.
- [ ] **Step 3: Implement.** Shatter and burn-pass respect reach (`hurt_enemy` / `in_reach`). Shatter hits go through `hurt_enemy` with `Some(Element::Ice)` but must not shatter recursively in the same tick more than once per dying enemy (collect, then apply).
- [ ] **Step 4: Run** `cargo test -p sim --release` — all pass.
- [ ] **Step 5:** Show the diff for review.

### Task 5: Set drops and the 40-run target

**Files:**
- Modify: `crates/sim/src/gear.rs` — `pub const SET_CHANCE: f32 = 0.12;` + Monte Carlo test.
- Modify: `crates/sim/src/world.rs` — kill handler (~L3437): Mini / Boss / Final kills roll `SET_CHANCE * exotic_mult(..) * find`, push `roll_set_piece(.., self.cfg.stage, drop_ilvl(..))` and `Event::GearDrop { rarity: Rarity::Set }`.
- Modify: `crates/sim/src/meta.rs` — `add_item`: `Rarity::Set` items skip the bag-full salvage.
- Modify: `crates/sim/src/bot.rs` or the `balance` ignored test — print eligible kills per run (count of Mini/Boss/Final kills) so the target can be checked against real runs.

- [ ] **Step 1: Write the failing tests:**
  - `set_collection_takes_about_forty_runs` (gear.rs): 2000 simulated players, each run 3 eligible kills at `SET_CHANCE * exotic_mult(0, 0)`, piece uniform over 6; mean runs to own all 6 distinct in 35–46.
  - `bosses_drop_this_worlds_set` (world.rs): force a Final kill with `SET_CHANCE` overridden through a world with huge `gear_find` (e.g. 1000) → `loot` has a `Set` item with `set == Some(stage)`; a Fodder / Elite kill never drops one.
  - `set_pieces_survive_a_full_bag` (meta.rs): fill to `BAG_SIZE`, `add_item(set piece)` returns a non-zero id and the bag holds `BAG_SIZE + 1`; a Mythic still salvages.
- [ ] **Step 2: Run** — FAIL.
- [ ] **Step 3: Implement.** Gate drops with `self.cfg.stress.is_none()` like the Exotic roll.
- [ ] **Step 4: Measure** eligible kills per Normal run: `BALANCE_SEEDS=10 BALANCE_SEED0=21 BALANCE_STAGE=shrine_valley BALANCE_HERO=riku cargo test --release -p sim balance -- --ignored --nocapture`. If wins average fewer than 3 eligible kills, the per-run figure in the Monte Carlo is the measured one and `SET_CHANCE` is adjusted so the mean lands ~40; record the value in "Measured" below and the spec.
- [ ] **Step 5: Run** `cargo test -p sim --release` — all pass. Show the diff for review.

### Task 6: Client display and README

**Files:**
- Modify: `crates/client/src/armory.rs` — set panel (~L159): `"<Set> set · <Element>"`, the 4 tier lines with the active ones lit and the count rule (`"(n/6 Set pieces, rings count as a pair)"`; legacy items say `"legacy: 2 and 4 only"`); `rarity_color(Rarity::Set)` used in the bag grid, item card, loadout summary (L350 adds 6-piece names).
- Modify: `crates/client/src/render.rs` (L1783, L2176) — `GearDrop` of `Set` uses the Set colour.
- Modify: `crates/client/src/armory.rs` loadout summary — per-element resistance line (`"Resist: Fire 24% · Ice 8%"`, cap shown as `"60% max"`).
- Modify: `README.md` — Gear section: elements on gear, Set tier and 2/4/5/6 bonuses, drop source.

- [ ] **Step 1:** `cargo test -p client` (9 pass) and `cargo build -p client --release` (0 warnings).
- [ ] **Step 2:** `./scripts/build-web.sh`; screenshot the armory with a dev-granted Set loadout (dev menu: add a "Grant Set pieces" button that adds all 6 pieces of the current world's set — dev only) using headless Chrome `--headless=new --mute-audio --use-angle=swiftshader --enable-unsafe-swiftshader`. Check every label sits `kit::MARGIN` from edges at 16:9 and a phone aspect.
- [ ] **Step 3:** Show the diff and screenshots for review.

### Task 7: Balance confirm and hand-off

- [x] **Step 1:** Bot vs `5f82b71` baseline, Normal, seeds 21–30 and 41–50, shrine_valley every finished hero + Ren / Riku on every stage (gear is default in bot runs, so only rng-stream noise is expected: every hero within 2 wins).
- [x] **Step 2:** Final whole-branch review (correctness + `ponytail-review`), fix round.
- [ ] **Step 3:** Spec: add "Carried into phase 6" (final values, rulings, rechecks); this plan: "Measured"; `handoff.md` and the memory note.
- [ ] **Step 4:** Run the full verify block from `handoff.md`; show the final diff. User commits.

## Measured

**Set drops (Task 5):** Riku, Shrine Valley, Normal, seeds 21–30: 10/10 won, 2.8 eligible Mini/Boss/Final kills
per run (3 in 8 runs, 2 in 2). Monte Carlo with that mix: `SET_CHANCE` 0.12 → ~45 runs, **0.13 → ~40.5 runs** (kept).

**Mid-phase user changes (2026-11-09):** `MAXED_REPLACE_CHANCE` 0.25 (replace card at max; was 100% in the first
fix, user asked to normalise); `ELEM_ITEM` 0.04 → **0.02** (halves both element damage and resistance per item;
single-element endgame stack ~×5.9 → ~×3.4); circle visuals capped at `reach_half().x`, lines clipped to reach,
damage numbers move with the player, `in_view` centre-only.

**Maxed offers per run** (Shrine Valley, seeds 21–30): Riku min 3 / median 13 / max 24, first at ~1109 s;
Ren 7 / 24.5 / 45, first at ~937 s → ~3.3 / ~6.1 replace cards per run at 0.25.

**Balance vs `5f82b71`** (Normal, 10 seeds per block, bot without gear; phase-5 tree before the final fix wave and
`ELEM_ITEM` change, neither of which the gearless bot can see). Every config within 2 wins; total +8 of 300 runs.

| Config | 21–30 base → p5 | 41–50 base → p5 |
|---|---|---|
| Ren Shrine Valley | 10 → 10 | 10 → 8 |
| Riku Shrine Valley | 9 → 9 | 7 → 9 |
| Shun Shrine Valley | 6 → 5 | 6 → 8 |
| Toya Shrine Valley | 9 → 9 | 10 → 9 |
| Kuon Shrine Valley | 8 → 9 | 10 → 10 |
| Ren Shotengai | 9 → 8 | 7 → 9 |
| Ren Ember Leaf | 3 → 4 | 8 → 7 |
| Ren Pale Citadel | 9 → 9 | 7 → 9 |
| Ren Rainward | 8 → 8 | 9 → 10 |
| Ren Shatterline | 9 → 10 | 9 → 10 |
| Riku Shotengai | 4 → 6 | 7 → 6 |
| Riku Ember Leaf | 4 → 3 | 1 → 1 |
| Riku Pale Citadel | 7 → 8 | 10 → 9 |
| Riku Rainward | 7 → 8 | 10 → 9 |
| Riku Shatterline | 10 → 10 | 9 → 10 |

Riku on Ember Leaf stays the weakest config (4 / 1 baseline); unchanged by phase 5, carried to phase 6.
