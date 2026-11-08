# Phase 3 — Skill Rarity and Replacement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every weapon, passive and in-run item has one fixed rarity tier shown on its card; new skills are offered by tier weight; a full slot can be replaced by a new skill that inherits half the old level; then balance and the phase 1–2 debt list are paid.

**Architecture:** A new `Tier` enum lives in `crates/sim/src/content.rs` next to `Upgrade` and `PickupKind`, with a hard-coded tier per skill taken from a one-off bot measurement. `World::make_offer` becomes a weighted draw and may add one replacement card; taking it parks the card in `World::replacing` (the offer stays open, the sim stays paused) until the client's picker calls `World::replace_with`. The client only draws: tier frame/label on cards and a keyboard picker built on `kit::nav`.

**Tech Stack:** Rust 2021, deterministic `crates/sim` (unit tests + headless balance bot), macroquad `crates/client`.

**Spec:** `docs/superpowers/specs/2026-11-07-elements-rarity-design.md` — §2 "Replacing skills", §6 "Skills and passives" and "In-run items", and "Carried into phase 3".

## Global Constraints

- **No commits.** The user commits. Each task ends with "show the diff for review". No AI attribution anywhere.
- Skill tiers: **Common, Uncommon, Rare, Legendary, Mythic, Unique** (not the gear `Rarity`; gear keeps Epic/Exotic).
- Each of the 26 weapons and 10 passives has exactly one fixed tier; bucket shares roughly Common 30%, Uncommon 25%, Rare 20%, Legendary 13%, Mythic 8%, Unique 4%.
- Starting offer weights: Common 1.0, Uncommon 0.8, Rare 0.6, Legendary 0.4, Mythic 0.25, Unique 0.15. They apply **only to skills not yet owned**; levelling an owned skill keeps weight 1.0.
- Signature weapons are given at start regardless of tier.
- Replacement: at most **one** replacement card per offer; new level = `max(1, floor(old_level / 2))`; the replaced skill is removed completely (bonuses, awakening, evolution progress); the new skill is never awakened on arrival; an awakened weapon stays awakened when its complement passive is replaced; the signature weapon can be replaced.
- In-run items keep their existing spawn weights; tiers are chosen to match those weights.
- Colour is never the only carrier: every tier colour comes with its tier name.
- Balance targets: all 4 weapon slots filled by about round 8; at least 2 weapons maxed; a Unique skill offered in roughly 1 run in 3; replacement is a real choice, not an every-level-up card.
- UI: virtual 1280×720 canvas via `screen_width()/screen_height()`, everything `kit::MARGIN` (24) from every edge; the font has no `⚡`, `←`, `→`.
- Bot is noisy (±2–4 wins at 10 seeds): baseline from a commit via `git archive <commit> | tar -x -C <dir>` with its own `CARGO_TARGET_DIR`; tune on seeds 21–30, confirm on seeds 41–50; never tune and confirm on the same seeds.
- Record every number the user changes mid-flight in the spec and this plan.

## Review Focus

1. Slots full and every owned skill maxed: the offer must still give real choices, never a lone replacement card the player cannot escape (Task 2 test `maxed_and_full_gets_fallbacks`).
2. Cancel in the replace picker returns to the same three cards, not a re-roll (Task 2 test `cancel_keeps_the_offer`).
3. Replacing Vitality must take its +20 max HP per level back, and the new skill must not double-count it (Task 2 test `replacing_vitality_returns_its_hp`).
4. Arsenal `extra` weapons never appear in the picker and never count toward the 4 slots (Task 2 test `extra_weapons_are_not_replaceable`).
5. `pending_levels > 1` (shrine +3 levels): after a replacement the next offer opens, exactly as after a normal pick (Task 2 test `replacement_consumes_one_level`).

---

### Task 1: Tier model and measured tier table

**Files:**
- Modify: `crates/sim/src/content.rs` — new `Tier` near `Upgrade` (~L1047), `impl Upgrade` (~L1077), `impl PickupKind` (~L140), tests (~L1432).
- Modify: `crates/sim/src/world.rs` tests — new ignored `skill_strength` next to `damage_over` (~L5578).

**Interfaces:**
- Produces (in `content.rs`, re-exported the way `Upgrade` is):
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)] pub enum Tier { Common, Uncommon, Rare, Legendary, Mythic, Unique }`
  - `pub const ALL_TIERS: [Tier; 6]`
  - `impl Tier { pub fn name(self) -> &'static str; pub fn weight(self) -> f32 }` — names are the variant names; weights from Global Constraints.
  - `impl Upgrade { pub fn tier(self) -> Tier }` — weapons and passives from the measured table; `Heal`, `Sen`, `Blessing` → `Common` (equal weights today).
  - `impl PickupKind { pub fn tier(self) -> Tier }` — Feast Common (25); SwiftGeta, OniSake, ShrineBell, SenPurse Uncommon (15, 15, 15, 12); Hourglass Rare (10); ExorcismSeal Legendary (8).

- [ ] **Step 1: Measure.** Add `#[test] #[ignore] fn skill_strength()` in `world.rs` tests. Weapons: for each of `ALL_WEAPONS`, hero Ren, max level, not awakened: `crowd = damage_over(k, false, HeroId::Ren)` and `single` = the same setup with one Oni at distance 60 (hp 1e9); strength = `sqrt(crowd * single)`. Passives: for each of `ALL_PASSIVES`, 10 bot runs (Ren, `shrine_valley`, Normal, seeds 21–30, the `balance_full_run` loop) with that passive pushed at `passive_max` before the first step; strength = wins, ties by mean `w.time`. Print one table sorted by strength per kind. Run: `cargo test --release -p sim skill_strength -- --ignored --nocapture` (run in background; the passive half is ~100 bot runs).
- [ ] **Step 2: Bucket and show.** Weapons (26) by rank, strongest first: Unique 1, Mythic 2, Legendary 3, Rare 5, Uncommon 7, Common 8. Passives (10): Mythic 1, Legendary 1, Rare 2, Uncommon 3, Common 3 (no Unique passive). Put the measured table and the resulting tiers in this plan under "Measured tiers" below and **show it to the user before Task 2** (it decides what players see).
- [ ] **Step 3: Write the failing tests** in `content.rs` tests:

```rust
#[test]
fn tiers_follow_the_buckets() {
    let count = |it: &mut dyn Iterator<Item = Tier>, t: Tier| it.filter(|x| *x == t).count();
    let w: Vec<Tier> = ALL_WEAPONS.iter().map(|k| Upgrade::Weapon(*k).tier()).collect();
    let p: Vec<Tier> = ALL_PASSIVES.iter().map(|k| Upgrade::Passive(*k).tier()).collect();
    use Tier::*;
    assert_eq!(ALL_TIERS.map(|t| count(&mut w.iter().copied(), t)), [8, 7, 5, 3, 2, 1]);
    assert_eq!(ALL_TIERS.map(|t| count(&mut p.iter().copied(), t)), [3, 3, 2, 1, 1, 0]);
    assert!(MAXED_FALLBACKS.iter().all(|u| u.tier() == Common));
}

#[test]
fn pickup_tiers_follow_spawn_weights() {
    for a in ALL_PICKUPS {
        for b in ALL_PICKUPS {
            if a.weight() > b.weight() {
                assert!(a.tier() <= b.tier(), "{a:?} spawns more often than {b:?} but has a higher tier");
            }
        }
    }
}

#[test]
fn tier_weights_fall_with_rarity() {
    assert_eq!(ALL_TIERS.map(Tier::weight), [1.0, 0.8, 0.6, 0.4, 0.25, 0.15]);
}
```

- [ ] **Step 4: Run, expect FAIL** — `cargo test -p sim --release tiers_ pickup_tiers tier_weights` → `Tier` not found.
- [ ] **Step 5: Implement** the Interfaces block (tier match arms from Step 2).
- [ ] **Step 6: Run, expect PASS**, then `cargo test -p sim --release` (88 + 3 pass).
- [ ] **Step 7: Show the diff and the measured table for review.**

#### Measured tiers

Measured with `skill_strength` (release, Ren). Weapons: max level, not awakened, strength = sqrt(crowd * single). Passives: 10 bot runs each (seeds 21-30, `shrine_valley`, Normal), wins then mean time.

| Tier | Weapons (strongest first) |
|---|---|
| Unique | SpiritSeal (4452), VenomFang (4097) |
| Mythic | GravityWell (10987), LanternVigil (4011) |
| Legendary | GraveCleaver (3672), FoxfireOrb (3671), VoidLance (3379) |
| Rare | FoxMirage (3365), MukadeSpines (3087), Ofuda (3058), FlamePool (2999), Crescent (2137) |
| Uncommon | CrimsonTendrils (1978), IaiFlash (1946), YinYangOrbs (1576), StarFall (1483), Familiars (1347), MoonOrbs (1246), GaleShuriken (1210) |
| Common | RepulsionStar (1204), Devour (1179), SakuraField (1100), PurificationWave (880), Taiko (631), SoulTether (605), ThunderChain (524) |

| Tier | Passives (wins of 10, mean time s) |
|---|---|
| (baseline, no passive) | 9, 1553 |
| Mythic | Might (10, 1593) |
| Legendary | Guard (10, 1578) |
| Rare | Recovery (10, 1558), Vitality (10, 1546) |
| Uncommon | Haste (10, 1544), Abundance (9, 1560), Swiftness (9, 1550) |
| Common | Expanse (9, 1528), Persistence (7, 1598), Magnet (7, 1563) |

Task 4 later promoted VenomFang to Unique, which shifted every lower weapon one rank down the table above. Controller ruling (Task 4): Unique never goes to a signature weapon (signatures are given at start, never offered), so GravityWell (Kuon's signature, strongest) moved to Mythic and SpiritSeal (strongest non-signature) became Unique.

Vitality was first measured without its +20 HP per level (pushed past `apply_upgrade`); re-measured via `apply_upgrade`. Passive wins saturate: the no-passive baseline wins 9 of 10, so only Persistence and Magnet (7) are clearly below it. Order among the rest is tie-break noise. Ties inside a win count rank by higher mean time.

### Task 2: Weighted offers and skill replacement (sim)

**Files:**
- Modify: `crates/sim/src/world.rs` — `World` struct (~L731) and `new` (~L864), `upgrade_pool` (~L4352), `make_offer` (~L4370), `choose` (~L4393), `apply_upgrade` (~L4422), tests (~L4839); every test loop that calls `w.choose(0)` repeatedly (`play` helper ~L4705, `balance_full_run`, the specials tests) switches to `w.bot_pick()`.
- Modify: `crates/sim/src/bot.rs` — `bot_pick`.
- Modify: `crates/sim/src/content.rs` — `REPLACE_CHANCE`.

**Interfaces:**
- Consumes: `Tier::weight`, `Upgrade::tier` (Task 1).
- Produces:
  - `pub const REPLACE_CHANCE: f32 = 0.35;` (content.rs) — chance an offer carries a replacement card when one is possible. Tuned in Task 4.
  - `pub replacing: Option<Upgrade>` on `World` — the replacement card taken, waiting for a target. The offer stays `Some` while it is set.
  - `pub fn is_replacement(&self, u: Upgrade) -> bool` — `u` is an unowned weapon/passive and that kind's slots are full (weapons count only non-`extra`).
  - `pub fn replace_targets(&self) -> Vec<(Upgrade, u8)>` — the owned skills of `replacing`'s kind with their levels, in slot order (non-`extra` weapons only). Empty when `replacing` is `None`.
  - `pub fn replace_with(&mut self, target: Option<usize>)` — `None`: clear `replacing`, offer unchanged (Cancel). `Some(j)`: remove `replace_targets()[j]`, put the new skill in the **same slot index** at `max(1, old / 2)`, not awakened; then consume one pending level exactly like `choose`.
  - `pub fn bot_pick(&mut self)` (bot.rs, `impl World`) — if `replacing` is set: replace the target with the lowest level (ties: lowest `tier`); else `choose(0)`.
- `make_offer` rules:
  - Regular pool = `upgrade_pool(true)` (unchanged: owned levels + unowned only while slots are free). Weighted draw without replacement, weight `1.0` for owned skills and `u.tier().weight()` for unowned.
  - Replacement candidates = unowned weapons in the hero pool when weapon slots are full, plus unowned passives when passive slots are full.
  - If the regular pool is empty: offer = `MAXED_FALLBACKS`, no replacement card.
  - Else, with probability `REPLACE_CHANCE` and candidates non-empty: one weighted-by-tier candidate goes into a random position, the other cards come from the regular pool (max 3 cards total).
- `choose(i)` on a card where `is_replacement` is true sets `replacing = Some(u)`, keeps the offer and `pending_levels`, and returns. `choose` while `replacing` is set does nothing.
- Removing a weapon drops its `OwnedWeapon` entirely. Removing Vitality at level `n`: `max_hp -= 20 n`, hp clamped. Adding a passive at level `m` applies Vitality as `m` level-ups (`+20 m` max HP, heal `20 m`).
- Chests (`open_chest`) never produce replacement cards: they keep using `upgrade_pool` as today.

- [ ] **Step 1: Write the failing tests** (world.rs tests; use a helper that fills Ren's 4 weapon slots like `offers_respect_hero_pool_and_weapon_slots` does):
  - `rare_skills_are_offered_less`: Ren, empty slots apart from the signature, 4000 `make_offer`s; count first-time offers per unowned weapon; the mean count of Common weapons > the mean count of Legendary-or-above weapons × 1.8 (skip a tier with no weapon in Ren's pool).
  - `owned_levels_ignore_tier`: Ren with 4 owned weapons of different tiers, all at level 2, passives full and maxed; over 3000 offers each owned weapon appears within ±15% of the mean.
  - `at_most_one_replacement_card`: slots full, 2000 offers: `offer.iter().filter(|u| w.is_replacement(**u)).count() <= 1` every time, and > 0 in at least `REPLACE_CHANCE × 0.7` of offers.
  - `maxed_and_full_gets_fallbacks`: all slots full and maxed → offer == `MAXED_FALLBACKS` 500 times out of 500.
  - `replacement_inherits_half`: weapon in slot 2 at level 7; take a replacement card → `replacing.is_some()`, offer still `Some`, `pending_levels` unchanged; `replace_with(Some(2))` → slot 2 holds the new kind at level 3, not awakened, old kind gone, `pending_levels` −1. Also level 1 → 1.
  - `cancel_keeps_the_offer`: take replacement, `replace_with(None)` → `replacing == None`, offer identical to before.
  - `awakening_survives_complement_replacement`: weapon awakened, its `complement` passive replaced → weapon still `awakened`.
  - `replacing_vitality_returns_its_hp`: Vitality 4 (max HP +80) replaced by another passive → `max_hp` back to base; replace a passive by Vitality from level 6 → new Vitality 3 and `max_hp` = base + 60.
  - `extra_weapons_are_not_replaceable`: an `extra: true` weapon plus 4 normal ones → `replace_targets()` has 4 entries, none the extra.
  - `replacement_consumes_one_level`: `pending_levels = 3`, replace → a new offer opens, `pending_levels == 2`.
  - `signature_can_be_replaced`: the signature in slot 0 is in `replace_targets()`.
- [ ] **Step 2: Run, expect FAIL** — `cargo test -p sim --release rare_skills owned_levels at_most_one maxed_and_full replacement_ cancel_keeps awakening_survives replacing_vitality extra_weapons signature_can` → missing items.
- [ ] **Step 3: Implement** the Interfaces block; switch repeated `choose(0)` test loops and `balance_full_run` to `bot_pick()`.
- [ ] **Step 4: Run, expect PASS**, then `cargo test -p sim --release`, `cargo test -p client`, `cargo build -p client --release` (0 warnings; the client still calls `choose` only, so a replacement card would hang the overlay until Task 3 — acceptable between tasks, not shippable).
- [ ] **Step 5: Show the diff for review.**

### Task 3: Tier frames, replace picker, item tiers in the client, README

**Files:**
- Modify: `crates/client/src/ui.rs` — `level_up` (~L373), new `replace_picker`, new `tier_color`.
- Modify: `crates/client/src/main.rs` — overlay flow (~L558).
- Modify: `crates/client/src/chest_ui.rs` — card text (~L221).
- Modify: `crates/client/src/render.rs` — floor pickup draw (`pickup_color` users ~L1802, ~L2211) and the `PickupTaken` line (~L676).
- Modify: `README.md` — the level-up / skills description.

**Interfaces:**
- Consumes: `Tier`, `Upgrade::tier`, `PickupKind::tier` (Task 1); `World::{replacing, is_replacement, replace_targets, replace_with}` (Task 2).
- Produces:
  - `pub fn tier_color(t: Tier) -> Color` in `ui.rs` — reuse `armory::rarity_color` values: Common `(0.78, 0.78, 0.82)`, Uncommon `(0.45, 0.9, 0.5)`, Rare `(0.35, 0.65, 1.0)`, Legendary `(1.0, 0.65, 0.2)`, Mythic `(1.0, 0.3, 0.45)`, Unique = the Exotic shimmer.
  - `pub fn replace_picker(world: &World, assets: &Assets, focus: &mut usize) -> Option<Option<usize>>` in `ui.rs` — `Some(Some(j))` replace target `j`, `Some(None)` Cancel, `None` undecided; same contract as `ui::shrine`.

- [ ] **Step 1: Level-up cards.** Card frame (outline + top stripe) uses `tier_color(u.tier())`; the tag chip keeps its text and gains the tier name before it, e.g. "RARE · NEW WEAPON". A replacement card's tag is "REPLACE · <tier>" with the same frame. Focus ring keeps the accent of the card kind.
- [ ] **Step 2: Replace picker.** Header "Replace which skill?" and the new skill's name + "joins at Lv N" (N from the rule). One row per `replace_targets()` entry: icon, name, `Lv n`, tier chip, "→ Lv max(1, n/2)" written as "becomes Lv N"; last row "Cancel". Navigation via `kit::nav` + `NavInput::read()` (arrows/WS, Enter/Space, digits), mouse hover moves focus, `Escape` = Cancel. Rows stack vertically, panel centred, inside `MARGIN` at 844×390.
- [ ] **Step 3: Wire main.rs.** In the `world.offer` branch: if `world.replacing.is_some()` draw `replace_picker` (reset focus when it opens, same `fresh` guard as the other overlays, Space must not fire the ultimate that frame — reuse `overlay_picked`), else `level_up` as today. Cancel returns to the cards with focus on the replacement card.
- [ ] **Step 4: Items.** Chest cards: a small tier chip (`tier_color` + name) under the subtitle. Floor pickups: a thin `tier_color` ring under the pickup sprite. `PickupTaken` line: `"{name}! {describe}"` → `"{name} ({tier})! {describe}"`.
- [ ] **Step 5: README.** Where level-up is described, add: skills have a rarity tier (Common → Unique) shown on the card; rarer new skills are offered less often; with full slots, a card may offer to replace a skill, and the newcomer starts at half the old level.
- [ ] **Step 6: Verify:** `cargo test -p client`, `cargo build -p client --release` (0 warnings), `./scripts/build-web.sh`; headless-Chrome screenshots (scratch dir, not committed) at 1280×720 and 844×390 of a level-up with mixed tiers and of the replace picker (use the dev menu to fill slots, e.g. "max loadout", then gain a level).
- [ ] **Step 7: Show the diff and screenshots for review.**

### Task 4: Balance rarity weights and replacement

**Files:**
- Modify: `crates/sim/src/world.rs` — `balance_full_run` print line; new counters on `World`: `pub replacements: u32`, `pub unique_offered: bool`, `pub slots_full_round: Option<u8>` (set in `make_offer` / `replace_with` / `apply_upgrade`).
- Modify: `crates/sim/src/content.rs` — `Tier::weight` values, `REPLACE_CHANCE`.

**Interfaces:**
- Consumes: Task 2 offers and `bot_pick`.
- Produces: the print line gains `slots_full_round`, `maxed` (weapons at `MAX_LEVEL`), `unique_offered`, `replacements`.

- [ ] **Step 1: Baseline.** `git archive 9a434ef | tar -x -C <scratch>/baseline`, separate `CARGO_TARGET_DIR`. Seeds 21–30, `shrine_valley` Normal, heroes ren, riku, shun, toya, kuon; plus `shotengai` ren (5 seeds). Record wins and median time per hero.
- [ ] **Step 2: Measure the new build** on the same seeds with the new columns.
- [ ] **Step 3: Tune** on seeds 21–30, levers in this order: `REPLACE_CHANCE` (0.15–0.6), the tier weights (keep them strictly falling), and only then the inherit rule (alternative `max(1, ceil(old / 2))`). Targets (averaged over the five heroes): median `slots_full_round` ≤ 8; median `maxed` ≥ 2; `unique_offered` in 25–45% of runs; `replacements` per run 0.5–3; wins per hero within ±1 of baseline.
- [ ] **Step 4: Confirm** on seeds 41–50 with the chosen values; report both tables. If no setting meets every target, take the closest, stop, and report.
- [ ] **Step 5:** Record the final values in the spec ("Carried into phase 4") and this plan. Show the diff and tables for review.

**Task 4 result (final values, fix round 2):** weights stay at the spec values 1.0 / 0.8 / 0.6 / 0.4 / 0.25 / 0.15; `REPLACE_CHANCE` 0.15 (back inside the 0.15-0.6 range); inherit rule unchanged. Both Uniques kept (SpiritSeal, VenomFang); weapon buckets [7, 7, 5, 3, 2, 2]. Bot policy: `bot_pick` takes a replacement card only to overwrite the lowest-level skill of a strictly lower tier, otherwise it takes the first plain card. Unique target per Unique-capable hero (Ren, Toya). Seeds 21-30 / 41-50: Unique offered Ren 4 / 4 of 10, Toya 8 / 5 of 10; replacements 1.7 / 1.5 per run; wins vs baseline 9 / 9 on Ren, 6 / 5 on Riku (baseline 6 / 6), 8 / 7 on Shun (6 / 6), 8 / 8 on Toya (8 / 9), 9 / 10 on Kuon (9 / 8). Full tables: `.superpowers/sdd/2026-11-08-phase3-skill-rarity/task-4-report.md`.

### Task 5: Per-hero levers and unmeasured heroes

**Files:**
- Modify: `crates/sim/src/content.rs` — `hero_def` `(reach, reach_might)` match (~L768): per-hero values instead of per class.
- Modify: `crates/sim/src/world.rs` — `might()` (~L942) reads `reach_might` from a `HeroDef` built once in `World::new` (store `pub hero: HeroDef` or just the `f32`) instead of `hero_def()` per call.

**Interfaces:**
- Produces: `reach_might` stays the one per-hero damage lever (doc: "per-hero damage lever; melee heroes offset their shorter reach"). Constants: keep `MIGHT_MELEE` for Shun and Haru; add `MIGHT_TOYA` and `MIGHT_RIKU` (Riku starts 1.0).

- [x] **Step 1: Failing test** `hero_levers_are_per_hero`: `hero_def(HeroId::Toya).reach_might == MIGHT_TOYA`, `hero_def(HeroId::Shun).reach_might == MIGHT_MELEE`, `hero_def(HeroId::Riku).reach_might == MIGHT_RIKU`.
- [x] **Step 2: Run, expect FAIL; implement; run, expect PASS.**
- [x] **Step 3: Tune** on seeds 21–30, shrine_valley Normal, after Task 4's values: Toya ≤ 8 of 10 wins, Riku ≥ 8 of 10, and no finished hero (ren, riku, shun, toya, kuon) at 10/10 or ≤ 4/10. Confirm on seeds 41–50.
- [x] **Step 4: Report-only sweep** (no tuning unless a hero lands at 0 or 10 wins): kaede, yuzu, toya, haru on shrine_valley Normal; all ten heroes × every stage at Hard with 5 seeds (`BALANCE_DIFF=hard`). One table in the final message.
- [x] **Step 5:** `BOSS_HP` stays a `shortcut:` (human playtests needed); say so in the report. Show the diff and tables for review.

Final values: `MIGHT_MELEE` 2.3 (Shun, Haru), `MIGHT_TOYA` 2.3, `MIGHT_RIKU` 2.0. `might()` keeps the per-call `hero_def` lookup (tests change `cfg.hero` at runtime). Tables: `.superpowers/sdd/2026-11-08-phase3-skill-rarity/task-5-report.md`.


### Task 6: Debt tidy-ups and the rising chime

**Files:**
- Modify: `crates/sim/src/world.rs` — tests next to `gear_bolts_pick_only_enemies_in_reach` (~L4612); `random_targets` fallback (~L3541).
- Modify: `crates/client/src/synth.rs` (new `Sfx::Rise`), `crates/client/src/main.rs` (~L520), `crates/client/src/ui.rs` (`mouse_moved` placement), the dev debug overlay (12 px from the edge → `kit::MARGIN`).
- Modify: `crates/sim/src/content.rs` — T1 doc comments that restate the name: delete them.

- [ ] **Step 1: Failing tests** (same shape as `gear_bolts_pick_only_enemies_in_reach`: one enemy just out of reach, one in reach, assert only the in-reach one is targeted/hurt): `raijin_drums_strike_only_in_reach`, `yata_mirror_reflects_only_in_reach`, `ember_leaf_set_blasts_only_in_reach`, `rainward_set_lightning_only_in_reach`. If one already passes, keep it as a regression test.
- [ ] **Step 2: Run; fix** any filter that fails, in the shared selector, not per caller.
- [ ] **Step 3: `random_targets`:** test `fallback_targets_spread_inside_reach` — with no enemies, 200 targets: all in reach and fewer than 10% within 4 units of the reach-box edge. Implement by sampling the offset inside the reach box (`rng.range(-h.x, h.x)`, `rng.range(-h.y, h.y)`) instead of clamping a ring.
- [ ] **Step 4: Chime.** `Sfx::Rise`: three rising sine notes over ~0.45 s, built like the existing `Sfx::Bell` arm; the ultimate alert in `main.rs` plays it and the `shortcut:` comment goes.
- [ ] **Step 5: Tidy:** move `mouse_moved()` next to the other input helpers at the top of `ui.rs`; move the debug overlay to `kit::MARGIN`; delete the restating T1 doc comments.
- [ ] **Step 6: Verify** the full suite from the handoff (`cargo test -p sim --release`, `cargo test -p client`, `cargo build -p client --release` 0 warnings, `./scripts/build-web.sh`). Update `handoff.md` and the spec's "Carried into phase 4" section. Show the diff for review.
