# Phase 4 — Elements Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every weapon, enemy, hero and world carries one of eight elements; hits are multiplied by the element cycle, heroes get +10% on their affinity element, hits can proc an element effect, and the player can read all of it on screen.

**Architecture:** A new `Element` enum and the cycle table live in `crates/sim/src/content.rs`; weapons, heroes, enemies (`enemies.rs`) and stages (`campaign.rs`) each answer `element()`. Every damage call (`World::hurt_enemy` → `apply_hit`) gains an explicit `elem: Option<Element>` so the compiler forces each of the ~23 call sites to decide; effects spawned by a weapon (projectiles, waves, pools, wells, blasts, lanterns, tethers, clones) store the element at spawn. On-hit effects are new per-enemy status columns on `Enemies`. The client only draws: element colour + name on cards, menus and hit numbers.

**Tech Stack:** Rust 2021, deterministic `crates/sim` (unit tests + headless balance bot), macroquad `crates/client`.

**Spec:** `docs/superpowers/specs/2026-11-07-elements-rarity-design.md` — §9 "Elements", §3 (alert colour = hero element), "Carried into phase 4".

## Global Constraints

- **No commits.** The user commits. Each task ends with "show the diff for review". No AI attribution anywhere.
- Eight elements: **Fire, Water, Electric, Earth, Wind, Ice** (common), **Light, Darkness** (rare).
- Cycle: Water ▶ Fire ▶ Ice ▶ Wind ▶ Earth ▶ Electric ▶ Water (each beats the next).
- Starting multipliers (spec table, bot-tunable): beats ×1.5; is beaten ×0.75; other common ×1.0; common → Light/Darkness ×0.9; Light/Darkness → common ×1.15; Light ↔ Darkness ×1.5 both ways; same element ×1.0; no element (`None`) ×1.0.
- Light/Darkness only on weapons of `Tier::Legendary` or above and enemies of `Rank::Mini` or above. Each common element ≥ 3 weapons; Light ≤ 3; Darkness ≤ 3.
- Passives, pickups, coins, chests: no element. Hero affinity is offence only: +10% damage for skills of that element, the ultimate deals that element, enemy hits on the player are never multiplied.
- On-hit effects: one roll per hit at `PROC_CHANCE` (start 0.15): Fire burns, Water soaks (soaked targets take +30% Electric), Electric chains to one more target, Earth staggers, Wind knocks back, Ice slows, Light marks (next hit crits), Darkness drains a little life.
- **Ruling: gear elements and elemental resistance move to phase 5** (they need the save migration the spec puts there). In phase 4 every gear-driven hit, stage hazard and pickup blast is `None` (neutral), and enemy elements only affect damage taken.
- Reach rule holds: every new hit (chain, burn tick) checks `in_reach` before it lands.
- Colour is never the only carrier: every element colour comes with its element name.
- UI: virtual 1280×720 canvas via `screen_width()/screen_height()`, everything `kit::MARGIN` (24) from every edge; the font has no `⚡`, `←`, `→`.
- Bot is noisy (±2–4 wins at 10 seeds): baseline from `73ca295` via `git archive 73ca295 | tar -x -C <dir>` with its own `CARGO_TARGET_DIR`; tune on seeds 21–30, confirm on 41–50; never tune and confirm on the same seeds. Use `bot_pick` in every test loop.
- Record every number the user changes mid-flight in the spec and this plan.

## Element tables (draft — user signs off at plan review)

**Weapons** (26; tier from phase 3):

| Element | Weapons |
|---|---|
| Fire (4) | FlamePool (Rare), FoxfireOrb (Legendary), Familiars (Uncommon), RepulsionStar (Common) |
| Water (3) | PurificationWave (Common), Devour (Common), VenomFang (Unique) |
| Electric (3) | ThunderChain (Common), IaiFlash (Uncommon), Taiko (Common) |
| Earth (3) | StarFall (Uncommon), CrimsonTendrils (Uncommon), MukadeSpines (Rare) |
| Wind (5) | Ofuda (Rare), SakuraField (Common), Crescent (Rare), GaleShuriken (Uncommon), FoxMirage (Rare) |
| Ice (3) | MoonOrbs (Uncommon), SoulTether (Common), YinYangOrbs (Uncommon) |
| Light (2) | SpiritSeal (Unique), LanternVigil (Mythic) |
| Darkness (3) | GraveCleaver (Legendary), GravityWell (Mythic), VoidLance (Legendary) |

PurificationWave is not Light, though the spec named it as an example: it is Common tier and the Light rule (Legendary+) wins.

**Heroes** (spec): Ren Light, Riku Fire, Shun Darkness, Toya Earth, Kuon Ice, Kaede Electric, Mika Water, Haru Wind, Nanami Fire, Yuzu Electric. Affinity weapons in pool: Ren 1, Riku 3, Shun 1, Toya 2, Kuon 1; signatures off-affinity: Ren (Ofuda Wind), Kuon (GravityWell Darkness), Mika (StarFall Earth).

**Worlds and enemies** (dominant ≥ 55% of the stage's roster kinds):

| Stage | Element | Enemies |
|---|---|---|
| Shotengai | Fire | Wisp Fire, Chochin Fire, Oni Fire, NightLord Fire; Kasa Water, Nopperabo Wind, GreatOni Earth |
| Shrine Valley | Earth | Kodama, FoxMaskPhantom, Komainu, BellMaiden, ToriiSerpent Earth; PaperCrane Wind, LanternFairy Fire, Mitsuhime Light |
| Ember Leaf | Wind | StrawPuppet, LeafShinobi, KunaiThrower, SmokeNinja, Kagerou Wind; IronRonin Earth, EmberToad Fire, BurningShogun Fire |
| Pale Citadel | Ice | LostSoul, MaskedHound, PaleArcher, ShadeDancer, GateCaptain Ice; BoneSentinel Earth, HollowMonarch Darkness, PaleSeraph Light |
| Rainward | Water | DrownedRat, RainGhoul, GutterSpitter, GutterMother, CrimsonRainLord Water; UmbrellaStalker Wind, WardButcher Fire, NightInvestigator Electric |
| Shatterline | Electric | CursedSpawn, WailingCommuter, CurseEye, GlitchShade, CrossingCalamity Electric; GradeOneBrute Earth, PatchworkSorcerer Water, ShatterSovereign Darkness |

## Review Focus

1. Electric chain must never jump to an out-of-reach enemy and never chain again from the chained hit (Task 3 test `chain_stays_in_reach_and_stops`).
2. A missed call site silently deals neutral damage: every weapon hit, awakened or not, must carry its weapon's element (Task 2 test `every_weapon_hit_carries_its_element`).
3. Status timers must move with their enemy on `swap_remove`, or a dead enemy's burn lands on a stranger (Task 3 test `statuses_follow_swap_remove`).
4. Burn ticks: keep counting down while the enemy is out of reach, dropping those ticks, and at most one `Hit` event per `BURN_TICK` (Task 3 test `burn_respects_reach`).
5. Darkness drain never heals above max HP (Task 3 test `drain_caps_at_max_hp`).

---

### Task 1: Element model and tables

**Files:**
- Modify: `crates/sim/src/content.rs` — `Element` near `Tier` (~L1060); `WeaponKind::element`; `HeroDef.element` in `hero_def` (~L760); tests (~L1500).
- Modify: `crates/sim/src/enemies.rs` — `EnemyKind::element` next to `rank` (~L208).
- Modify: `crates/sim/src/campaign.rs` — `StageId::element`; test next to `every_enemy_has_a_home_and_rosters_match_ranks` (~L469).

**Interfaces:**
- Produces (in `content.rs`, re-exported the way `Tier` is):
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Element { Fire, Water, Electric, Earth, Wind, Ice, Light, Darkness }`
  - `pub const ALL_ELEMENTS: [Element; 8]`
  - `impl Element { pub fn name(self) -> &'static str; pub fn is_rare(self) -> bool; pub fn beats(self) -> Option<Element> }` — `beats` follows the cycle; `None` for Light/Darkness.
  - `pub fn element_mult(attacker: Element, target: Element) -> f32` — the table in Global Constraints, as named constants `ELEM_STRONG` 1.5, `ELEM_WEAK` 0.75, `ELEM_INTO_RARE` 0.9, `ELEM_FROM_RARE` 1.15, `ELEM_RARE_CLASH` 1.5.
  - `pub const AFFINITY_DAMAGE: f32 = 1.1;`
  - `impl WeaponKind { pub fn element(self) -> Element }`; `HeroDef { pub element: Element, .. }`
  - `impl EnemyKind { pub fn element(self) -> Element }`; `impl StageId { pub fn element(self) -> Element }` — values from "Element tables".

- [ ] **Step 1: Write the failing tests** in `content.rs` tests:

```rust
#[test]
fn element_cycle_matches_spec() {
    use Element::*;
    let cycle = [Water, Fire, Ice, Wind, Earth, Electric];
    for (i, a) in cycle.iter().enumerate() {
        let b = cycle[(i + 1) % 6];
        assert_eq!(a.beats(), Some(b));
        assert_eq!(element_mult(*a, b), 1.5);
        assert_eq!(element_mult(b, *a), 0.75);
        assert_eq!(element_mult(*a, Light), 0.9);
        assert_eq!(element_mult(Darkness, *a), 1.15);
    }
    assert_eq!(element_mult(Fire, Wind), 1.0);
    assert_eq!(element_mult(Light, Darkness), 1.5);
    assert_eq!(element_mult(Darkness, Light), 1.5);
    assert_eq!(element_mult(Light, Light), 1.0);
}

#[test]
fn weapon_elements_follow_the_rules() {
    for e in ALL_ELEMENTS {
        let n = ALL_WEAPONS.iter().filter(|k| k.element() == e).count();
        if e.is_rare() { assert!(n <= 3, "{e:?} has {n}") } else { assert!(n >= 3, "{e:?} has {n}") }
    }
    for k in ALL_WEAPONS.iter().filter(|k| k.element().is_rare()) {
        assert!(Upgrade::Weapon(*k).tier() >= Tier::Legendary, "{k:?} is rare-element below Legendary");
    }
}
```

and in `campaign.rs` tests:

```rust
#[test]
fn worlds_are_mostly_their_element() {
    for s in ALL_STAGES {
        let d = s.def();
        let mut roster = vec![d.fodder, d.fodder_b, d.ranged, d.blinker, d.elite, d.mini, d.final_boss];
        roster.extend(d.boss);
        let own = roster.iter().filter(|k| k.element() == s.element()).count();
        assert!(own as f32 / roster.len() as f32 >= 0.55, "{s:?}");
        for k in &roster {
            assert!(!k.element().is_rare() || k.rank() >= Rank::Mini, "{k:?} is rare-element below Mini");
        }
    }
}
```

- [ ] **Step 2: Run** `cargo test -p sim --release -- element worlds_are` — expect compile failure (`Element` undefined).
- [ ] **Step 3: Implement** the Interfaces block. `hero_def` takes the hero elements from "Element tables".
- [ ] **Step 4: Run** `cargo test -p sim --release` — all pass (113 + 3).
- [ ] **Step 5:** Show the diff for review.

### Task 2: Element on every hit

**Files:**
- Modify: `crates/sim/src/world.rs` — `hurt_enemy`/`apply_hit` (~L2879), `nova`, `hurt_along`, `burst_projectile`, `Event::Hit` (~L80), `Projectiles` (~L492) and the effect structs `Wave`, `Strike`, `Pool`, `Well`, `Blast`, `Lantern`, `Tether`, `MirageClone`; every caller listed by `graft callers hurt_enemy` and `graft callers apply_hit`.
- Modify: `crates/client/src/render.rs:502` — destructure the new `Event::Hit` fields (drawing comes in Task 4).

**Interfaces:**
- Consumes: Task 1 `Element`, `element_mult`, `AFFINITY_DAMAGE`, `WeaponKind::element`, `HeroDef.element`, `EnemyKind::element`.
- Produces:
  - `fn hurt_enemy(&mut self, j: usize, amount: f32, dir: Vec2, elem: Option<Element>)` — reach check, `apply_hit`, then the proc roll (Task 3).
  - `fn apply_hit(&mut self, j: usize, amount: f32, dir: Vec2, elem: Option<Element>)` — multiplies by `element_mult(elem, enemy.element())` and by `AFFINITY_DAMAGE` when `elem == Some(hero element)`, before boss bonus and crits. No procs.
  - `Event::Hit { pos, amount, crit, elem: Option<Element>, mult: f32 }` — `mult` is `element_mult` alone (1.0 for `None`).
  - `Projectiles.elem: Vec<Option<Element>>`, filled by `push_kind`; a `pub elem: Option<Element>` field on each effect struct above, set at spawn.

Call-site rule: weapon, weapon signature, awakened form, clone, lantern, tether, wave, strike, pool, well, blast or projectile from a weapon → `Some(kind.element())`; specials (`use_special`, `start_*`, `update_fox/hannya/divine/domain`, Mukade) → `Some(hero.element)`; gear (`update_gear`, `update_art`, `reflect_shot`, eclipses, exotic blasts), stage hazards and pickups (`update_pickups`, `view_nova`) → `None`. A pool left by a burning blast takes the blast's element.

- [ ] **Step 1: Write the failing tests** in `world.rs` tests:
  - `element_multiplies_damage`: `world_with_enemy_at(40.0)` with the enemy kind swapped per case; `hit_hp_lost` (now passing `elem`) gives 5.0 for `None`, 7.5 Water → Fire (Wisp), 3.75 Ice → Fire, 5.5 for hero Riku hitting Nopperabo (Wind, neutral to Fire) with `Some(Fire)` (affinity ×1.1 only).
  - `every_weapon_hit_carries_its_element`: for each `k` in `ALL_WEAPONS`, for `awakened` in `[false, true]`: a world with only `k` at max level (awakened as given), a ring of 12 Oni (hp 1e9) at distance 60–150, step 10 s with zero input; every `Event::Hit` has `elem == Some(k.element())` and at least one hit happened.
  - `ultimate_hits_carry_hero_element`: for each finished hero, fill `player.special`, use the special, step 9 s; with `w.weapons` cleared first, every `Event::Hit` has `elem == Some(hero_def(h).element)`.
- [ ] **Step 2: Run** `cargo test -p sim --release -- element_multiplies every_weapon_hit ultimate_hits` — fails to compile (signature).
- [ ] **Step 3: Implement** the Interfaces block; fix every caller by the call-site rule; `hit_hp_lost` and other existing tests pass `None` unless they test elements.
- [ ] **Step 4: Run** `cargo test -p sim --release` and `cargo build -p client --release` — all pass, 0 warnings. Existing balance-free tests whose numbers moved because of multipliers get fixed by giving them a neutral enemy or `None`, never by loosening an assertion.
- [ ] **Step 5:** Show the diff for review.

### Task 3: On-hit element effects

**Files:**
- Modify: `crates/sim/src/content.rs` — effect constants next to `AFFINITY_DAMAGE`.
- Modify: `crates/sim/src/world.rs` — `Enemies` columns (+ `swap_remove`, `spawn_enemy`), `hurt_enemy` proc roll, new `on_hit_effect`, status tick in `step` (beside the pickup-buff timers ~L2366), `move_enemies` (~L1678) speed, `apply_hit` (soak, mark).

**Interfaces:**
- Consumes: Task 2 `hurt_enemy`, `apply_hit`.
- Produces:
  - Constants: `PROC_CHANCE` 0.15; `BURN_SECONDS` 3.0, `BURN_SHARE` 0.3 (of the hit, spread over the burn), `BURN_TICK` 0.5; `SOAK_SECONDS` 4.0, `SOAK_ELECTRIC` 1.3; `CHAIN_RANGE` 90.0, `CHAIN_SHARE` 0.5; `STAGGER_SECONDS` 0.4; `KNOCK_EXTRA` 220.0; `SLOW_SECONDS` 2.0, `SLOW_FACTOR` 0.65; `DRAIN_HEAL` 0.5 (HP per proc).
  - `Enemies` columns: `burn: Vec<f32>` (seconds left), `burn_dps: Vec<f32>`, `soak: Vec<f32>`, `stagger: Vec<f32>`, `slow: Vec<f32>`, `marked: Vec<bool>`.
  - `fn on_hit_effect(&mut self, j: usize, elem: Element, dealt: f32, dir: Vec2)` — called from `hurt_enemy` when `elem` is `Some` and `rng.f32() < PROC_CHANCE`. Burn and soak refresh (never stack). Chain hits the nearest other live enemy within `CHAIN_RANGE` that is `in_reach`, via `apply_hit` with `dealt × CHAIN_SHARE` and `Some(Electric)` (no proc, so no further chain). Stagger: `is_boss()` enemies ignore it. Knockback: `push += dir × KNOCK_EXTRA × knock[j]`. Slow: speed × `SLOW_FACTOR` while `slow > 0`. Mark: the next `apply_hit` on `j` crits and clears it. Drain: player hp += `DRAIN_HEAL`, capped at `max_hp`.
  - `apply_hit` multiplies `Some(Electric)` hits on soaked enemies by `SOAK_ELECTRIC`.
  - Burn ticks every `BURN_TICK` through `apply_hit(j, burn_dps × BURN_TICK, Vec2::ZERO, None)` only while `in_reach`.

- [ ] **Step 1: Write the failing tests** in `world.rs` tests (call `on_hit_effect` directly so no test depends on the proc roll):
  - `burn_respects_reach`: burn an in-reach Oni from a 100-damage hit; over `BURN_SECONDS` hp drops by 30 ± 0.5 (neutral element) with ≤ 7 `Hit` events; a second Oni burned then moved outside `reach_half` loses nothing.
  - `soak_boosts_only_electric`: soaked Kodama (Earth) takes `5 × element_mult(Electric, Earth) × 1.3` from `Some(Electric)` and plain `5 × element_mult(Fire, Earth)` from `Some(Fire)`.
  - `chain_stays_in_reach_and_stops`: three Oni in a line 60 apart, the third outside reach; Electric effect on the first hits the second for half, never the third, and the second's hp drops exactly once.
  - `stagger_skips_bosses`: staggered Wisp has zero velocity for 0.4 s, then moves; staggered GreatOni moves at once.
  - `slow_and_knockback`: slowed Wisp moves at 0.65× for 2 s; Wind effect adds push.
  - `mark_makes_next_hit_crit`: `crit_chance` 0, mark, next `hurt_enemy` event has `crit: true`, the one after does not.
  - `drain_caps_at_max_hp`: at `max_hp − 0.2`, Darkness effect leaves hp exactly `max_hp`.
  - `statuses_follow_swap_remove`: burn enemy 0, kill it, step; the enemy swapped into index 0 has `burn == 0`.
  - `proc_rate_is_about_fifteen_percent`: 4000 `hurt_enemy(.., Some(Earth))` on a 1e9-hp Oni, count staggers applied: 0.12–0.18.
- [ ] **Step 2: Run** `cargo test -p sim --release -- burn_ soak_ chain_ stagger_ slow_ mark_ drain_ statuses_ proc_rate` — fails.
- [ ] **Step 3: Implement** the Interfaces block.
- [ ] **Step 4: Run** `cargo test -p sim --release` — all pass.
- [ ] **Step 5:** Show the diff for review.

### Task 4: Show elements in the client

**Files:**
- Modify: `crates/client/src/ui.rs` — `element_color`, level-up card (~L410), replace picker (~L480), `special_color` callers.
- Modify: `crates/client/src/main.rs:523` — alert colour.
- Modify: `crates/client/src/render.rs:502` — hit numbers.
- Modify: `crates/client/src/menu.rs` — hero select (~L638), stage select (~L1038), collections enemy entry (~L580).

**Interfaces:**
- Consumes: Task 1 `Element::name`, `beats`, `element_mult`, `WeaponKind::element`, `HeroDef.element`, `EnemyKind::element`, `StageId::element`; Task 2 `Event::Hit { elem, mult }`.
- Produces: `pub fn element_color(e: Element) -> Color` in `ui.rs` — Fire (1.0, 0.45, 0.2), Water (0.3, 0.6, 1.0), Electric (1.0, 0.9, 0.3), Earth (0.75, 0.55, 0.3), Wind (0.45, 0.9, 0.6), Ice (0.7, 0.95, 1.0), Light (1.0, 0.95, 0.75), Darkness (0.7, 0.4, 0.95).

- [ ] **Step 1: Write the failing test** in `ui.rs` tests: `element_colors_are_distinct` — the 8 colours differ pairwise by > 0.15 in at least one channel, and every `name()` is non-empty.
- [ ] **Step 2: Run** `cargo test -p client element_colors` — fails.
- [ ] **Step 3: Implement:**
  - Weapon cards and replace-picker rows: an element chip (`FIRE`, element colour) after the tier chip, and one line above the description: `Strong vs Ice · Weak vs Water` (Light/Darkness: `Strong vs Darkness`). Passive and fallback cards get no chip.
  - Ultimate alert: `element_color(hero_def(..).element)` instead of `special_color` (spec §3). `special_color` stays for the form visuals at ~L183.
  - Hit numbers: `mult > 1.0` → element colour and +20% size; `mult < 1.0` → `FAINT`; otherwise unchanged.
  - Hero select: `Affinity: Fire — +10% Fire damage`. Stage select: `Element: Fire — most enemies here are Fire`. Collections enemy entry: element name in its colour before the lore.
- [ ] **Step 4: Run** `cargo test -p client`, `cargo build -p client --release` (0 warnings), `./scripts/build-web.sh`; take headless-Chrome screenshots (`--use-angle=swiftshader --enable-unsafe-swiftshader`) of level-up, hero select, stage select at 1280×720 and 844×390; check every new text sits ≥ `kit::MARGIN` from the edges and nothing overflows its card.
- [ ] **Step 5:** Show the diff and screenshots for review.

### Task 5: Balance, README and debts

**Files:**
- Modify: `crates/sim/src/content.rs` — effect and multiplier constants only.
- Modify: `README.md` — "The game" (new Elements paragraph), "Heroes" (affinity per hero), "Campaign" (world element).
- Modify: the spec — new "Carried into phase 5" section; this plan — final values.

- [ ] **Step 1: Baseline.** Extract `73ca295` to a temp dir; run the balance bot there and on the branch for Ren, Riku, Shun, Toya, Kuon on `shrine_valley` Normal, seeds 21–30; and Ren + Riku on all six stages, seeds 21–30. Run in background.
- [ ] **Step 2: Tune** (seeds 21–30 only) until: each hero within 2 wins of baseline on `shrine_valley`; no stage loses more than 3 wins for Ren or Riku (anti-goal: no world unbeatable without the right element); Shun (Darkness) and Ren (Light) gain no more than 2 wins over the median hero delta (anti-goal: Light/Darkness not the only build). Levers, in order: `PROC_CHANCE`, `DRAIN_HEAL`, `BURN_SHARE`, then `ELEM_STRONG`/`ELEM_WEAK`.
- [ ] **Step 3: Confirm** on seeds 41–50 with the same targets; recheck Kuon (10/10 on 41–50 in phase 3).
- [ ] **Step 4: Write** the final constants and the measured wins into this plan under "Measured" and into a spec "Carried into phase 5" section (gear elements + resistance, Arsenal = weapon element, Set = set element, item element on load; open rechecks). Update the README sections listed.
- [ ] **Step 5: Verify:** `cargo test -p sim --release`, `cargo test -p client`, `cargo build -p client --release` (0 warnings), `./scripts/build-web.sh`. Show the diff for review.

## Measured

Wins out of 10, Normal, baseline `73ca295` -> phase 4 build, same seeds. Seeds 21-30 = tune block, 41-50 = confirm block.

| Seeds 21-30, shrine_valley | Ren | Riku | Shun | Toya | Kuon |
|---|---|---|---|---|---|
| baseline -> phase 4 | 10 -> 10 | 8 -> 9 | 6 -> 6 | 8 -> 9 | 9 -> 8 |

| Seeds 21-30, by stage | shotengai | ember_leaf | pale_citadel | rainward | shatterline |
|---|---|---|---|---|---|
| Ren | 4 -> 9 | 4 -> 3 | 8 -> 9 | 8 -> 8 | 9 -> 9 |
| Riku | 7 -> 4 | 5 -> 4 | 9 -> 7 | 8 -> 7 | 9 -> 10 |

| Seeds 41-50 | Ren | Riku | Shun | Toya | Kuon |
|---|---|---|---|---|---|
| shrine_valley | 9 -> 10 | 9 -> 7 | 5 -> 6 | 8 -> 10 | 8 -> 10 |
| shotengai | 8 -> 7 | 5 -> 7 | | | |
| pale_citadel | | 8 -> 10 | | | |

Target checks:
- Every hero within 2 wins of baseline on shrine_valley in both blocks: met (worst Riku -2 on 41-50).
- No stage lost more than 3 wins for Ren or Riku: met (worst Riku shotengai -3 on 21-30, +2 on 41-50, so noise).
- Ren (Light) and Shun (Darkness) never exceeded the median hero delta by more than 2: met.
- Kuon recheck: 10/10 on seeds 41-50 again (phase 3 also 10/10; baseline 8). Strong on that seed block; recorded, not tuned.

Ruling: every target was met on the first measurement, so no constant was tuned. Final values are the planned ones:
`PROC_CHANCE` 0.15, `BURN_SECONDS` 3.0, `BURN_SHARE` 0.3, `BURN_TICK` 0.5, `SOAK_SECONDS` 4.0, `SOAK_ELECTRIC` 1.3,
`CHAIN_RANGE` 90.0, `CHAIN_SHARE` 0.5, `STAGGER_SECONDS` 0.4, `KNOCK_EXTRA` 220.0, `SLOW_SECONDS` 2.0,
`SLOW_FACTOR` 0.65, `DRAIN_HEAL` 0.5, `ELEM_STRONG` 1.5, `ELEM_WEAK` 0.75, `ELEM_INTO_RARE` 0.9,
`ELEM_FROM_RARE` 1.15, `ELEM_RARE_CLASH` 1.5, `AFFINITY_DAMAGE` 1.1 (confirmed against `crates/sim/src/content.rs`).
