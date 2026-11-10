# Phase 6 — Enemy Rarity, Leaders and 18 New Enemies Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every enemy has a rarity tier (Common, Uncommon, Elite, Leader, Sub-boss, Boss) shown as a coloured outline plus a shape cue. Each world gains 2 Uncommons and 1 Leader with HD art and lore. Leaders arrive at rounds 12 / 15 / 18 with a crew and a banner that buffs nearby enemies.

**Architecture:** `enemies.rs` owns the data: 18 new `EnemyKind` variants with defs, names, lore, elements, a new `EnemyRarity` derived per kind, and Leader `BossKit`s. Leaders use `Ai::Boss` with `Rank::Elite`, so the existing boss move code drives them while HP, chests and the boss bar treat them as Elites. `campaign.rs` adds each world's three newcomers to `StageDef`. `world.rs` replaces the round-4 and round-12 stand-ins and adds the Uncommon spawn share, banner, scatter and drops. The client draws the outline from a white silhouette texture per enemy, plus chevrons, banner, boss-bar rarity word and codex line. Art comes from a new `.px` generator and a new HD script.

**Tech Stack:** Rust 2021, deterministic `crates/sim` (unit tests + headless balance bot), macroquad `crates/client`, Python 3 + Pillow for `tools/`.

**Spec:** `docs/superpowers/specs/2026-11-07-elements-rarity-design.md`: §5, §4 (rounds), and "Phase 6 addendum (2026-11-09)", which wins where they differ. Read "Carried into phase 6" for debt.

## Global Constraints

- **No commits.** The user commits. Each task ends with "show the diff for review". No AI attribution anywhere.
- `Rank` is unchanged. New `EnemyRarity { Common, Uncommon, Elite, Leader, SubBoss, Boss }`. Uncommons are `Rank::Fodder` and Leaders are `Rank::Elite`. Existing kinds: Fodder → Common, Elite → Elite, Mini → SubBoss, Boss/Final → Boss.
- Elements of the 18 (addendum decision 1, verbatim): Rokurokubi Clerk Earth, Tsurube Drop Fire, Captain Kitsunebi Fire; Shard Moth Wind, Bone-Eater Crow Earth, Admiral Tengu Earth; Rogue Puppeteer Wind, Exploding Clay Bird Wind, Mist Swordsman Captain Water; Pale Citadel three Ice; Rainward Water / Electric / Water; Shatterline Swarm Mother Earth, Tool Thief Electric, Curse User Captain Electric. `worlds_are_mostly_their_element` keeps ≥ 55% counting all roster kinds.
- Rarity colours reuse `armory::rarity_color`: Uncommon → `Rarity::Uncommon`, Elite → `Rare`, Leader → `Epic`, SubBoss → `Legendary`, Boss → `Mythic`. Common has no outline. Shape cues: Uncommon 1 chevron, Elite 2, Leader banner flag + faint radius circle, Sub-boss / Boss the words "SUB-BOSS" / "BOSS" on the boss bar. The element ring (Mini+) and the pip are unchanged.
- Starting values (bot-tuned within the balance target): `BANNER_RADIUS` 140, `BANNER_SPEED` 1.25, `BANNER_DAMAGE` 1.25 (contact damage only), `SCATTER_TIME` 2.0, `UNCOMMON_SHARE` 0.03 (from round 5), Uncommon XP 4 (2× `fodder_b`), Leader pickup chance `LEADER_PICKUP_CHANCE` 0.5 in place of `ONI_PICKUP_CHANCE` (not stacked).
- Uncommon gear: about **+1 item per Normal run** on average. `UNCOMMON_GEAR` is derived from the measured Uncommon kills per run (Task 4). Set pieces stay Mini/Boss/Final only.
- Leaders spawn at the start of rounds `LEADER_ROUNDS` = [12, 15, 18], never while one is alive, and never in round 20. The round-12 Leader replaces the 3 stand-in elites (`world.rs` ~L1333). The round-4 Uncommon wave replaces the `fodder_b` ring and keeps the chest.
- Balance: phase 5's 30 configs (Ren, Riku, Shun, Toya, Kuon on Shrine Valley; Ren and Riku on the other 5 worlds; × 2 seed blocks), Normal, 10 seeds. Each config within 2 wins of `c1eca45`. A miss gets one re-run on a fresh block; only a repeated miss forces tuning. Tune on seeds 21–30; confirm on 81–90 and 91–100; re-run misses on 111–120.
- Variant names are save keys (`key_of` = lowercase Debug name). Pick them once in Task 1; never rename after.
- Names are original. The working names below avoid franchise terms (my call, the user checks at plan review): Masked Fracción → **Masked Retainer**, Espada Quartermaster → **Numbered Quartermaster**, Kagune Runner → **Bladetail Runner**. Archetypes and elements unchanged.
- UI: virtual 1280×720 via `kit::screen_width()/screen_height()`, `kit::MARGIN` (24) from every edge. The font lacks `⚡ ← → ×`. Elements show via `ui::element_tag` / `element_pip`, never a pill.
- Bot is noisy (±2–4 wins). Use `bot_pick` in test loops. Build test worlds with the hero in `WorldConfig` (`play_as`). Screenshots: headless Chrome `--headless=new --mute-audio --use-angle=swiftshader --enable-unsafe-swiftshader`.
- Record every number the user changes mid-flight in the spec and this plan.

## Roster (Task 1 data)

| World | Variant (save key) | Display name | Rarity | Element | `Ai` (def) | Leader kit moves |
|---|---|---|---|---|---|---|
| Shotengai | `RokurokubiClerk` | Rokurokubi Clerk | Uncommon | Earth | `Dash` | — |
| Shotengai | `TsurubeDrop` | Tsurube Drop | Uncommon | Fire | `Blink` | — |
| Shotengai | `CaptainKitsunebi` | Captain Kitsunebi | Leader | Fire | `Boss` | `Fan{5,0.6,150}` |
| Shrine Valley | `ShardMoth` | Shard Moth | Uncommon | Wind | `Chase` (fast, low HP) | — |
| Shrine Valley | `BoneEaterCrow` | Bone-Eater Crow | Uncommon | Earth | `Dash` | — |
| Shrine Valley | `AdmiralTengu` | Admiral Tengu | Leader | Earth | `Boss` | `Blink`, `Rain{4}` |
| Ember Leaf | `RoguePuppeteer` | Rogue Puppeteer | Uncommon | Wind | `Ranged Fan3` | — |
| Ember Leaf | `ClayBird` | Exploding Clay Bird | Uncommon | Wind | `Charger` | — |
| Ember Leaf | `MistCaptain` | Mist Swordsman Captain | Leader | Water | `Boss` | `Blink`, `Charge` |
| Pale Citadel | `MaskedRetainer` | Masked Retainer | Uncommon | Ice | `Dash` | — |
| Pale Citadel | `HollowHusk` | Hollow Husk | Uncommon | Ice | `Chase` (slow, high HP, radius 11) | — |
| Pale Citadel | `NumberedQuartermaster` | Numbered Quartermaster | Leader | Ice | `Boss` | `Ring{14,1,95}` |
| Rainward | `BladetailRunner` | Bladetail Runner | Uncommon | Water | `Dash` | — |
| Rainward | `MaskCollector` | Mask Collector | Uncommon | Electric | `Blink` | — |
| Rainward | `DrownedCaptain` | Drowned Pirate Captain | Leader | Water | `Boss` | `Summon{4}` (minion = world `fodder`) |
| Shatterline | `SwarmMother` | Grade-Three Swarm Mother | Uncommon | Earth | `Hop` | — |
| Shatterline | `ToolThief` | Cursed Tool Thief | Uncommon | Electric | `Blink` | — |
| Shatterline | `CurseCaptain` | Curse User Captain | Leader | Electric | `Boss` | `Rain{5}` |

Starting stats: Uncommon ≈ 3× its world's `fodder_b` HP, speed within ±10% of `fodder_b`, damage 11, radius 9, xp 4, knockback 0.5. Leader ≈ 1.8× its world's elite HP, speed 34, damage 24, radius 16, xp 60, knockback 0. Leader kit cycle 3.0 s. Lore: one or two short sentences in the house voice. Leaders lean on One Piece (crew, banner, ship); Uncommons on their world's inspiration.

## Review Focus

1. A frozen, staggered or Firmament-held Leader must hold still and not fire, like other non-bosses. `update_bosses` steers every `Ai::Boss` today, so it would ignore freeze (Task 3 test `frozen_leader_holds_still`).
2. Skipping several rounds at once (dev skip, lag catch-up loop in `director`) must spawn at most one Leader (Task 2 test `round_skip_spawns_one_leader`).
3. A Leader killed by any source, including a screen clear, ends the banner the same tick, and its crew scatters (Task 3 test `banner_ends_when_leader_dies`).
4. Leaders are never bosses: no boss bar, no `bosses_down`, no boss music (`main.rs:377`), no Set or Exotic roll, no phase bursts on Hard (Task 1 test `leaders_are_not_bosses`).
5. An old save with the 47 old enemy keys loads every kill count. The new kinds start at 0, and save → load → save is identical (Task 1 test `old_save_keeps_enemy_kills`).

---

### Task 1: Enemy data, rarity and rosters

**Files:**
- Modify: `crates/sim/src/enemies.rs`: 18 variants (each world's three after its existing kinds, in `EnemyKind` and `ALL_ENEMIES`, now `[EnemyKind; 65]`); `enemy_def`, `rank` (Leaders explicit `Rank::Elite`), `element`, `name`, `short_name`, `lore`; `EnemyRarity` + `EnemyKind::rarity`; `boss_kit` arms for the 6 Leaders; `boss_phases` returns 1 when `!kind.is_boss()`.
- Modify: `crates/sim/src/campaign.rs`: `StageDef { pub uncommon: [EnemyKind; 2], pub leader: EnemyKind, .. }`; `world(..)` gains a `[EnemyKind; 3]` argument; Shotengai literal; `home_stage`; both roster tests.
- Modify: `crates/sim/src/meta.rs` tests.
- Modify: `crates/client/src/render.rs`: nothing yet (new kinds fall back to the `oni` texture through `enemy_tex` until Task 5).

**Interfaces:**
- Produces: `pub enum EnemyRarity { Common, Uncommon, Elite, Leader, SubBoss, Boss }` (derive `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord`), `EnemyRarity::name(self) -> &'static str` ("Common", "Uncommon", "Elite", "Leader", "Sub-boss", "Boss"), `EnemyKind::rarity(self) -> EnemyRarity`, `EnemyKind::is_leader(self) -> bool`; `StageDef.uncommon`, `StageDef.leader`; the 18 variants named in the Roster table.

- [ ] **Step 1: Write the failing tests:**
  - `enemies.rs` `rarity_follows_rank_and_roster`: every kind in `ALL_ENEMIES` maps per Global Constraints; exactly 12 Uncommon and 6 Leader; every Uncommon is `Rank::Fodder`; every Leader is `Rank::Elite` with `enemy_def(..).ai == Ai::Boss`; `ALL_ENEMIES` has no duplicates; `index()` round-trips.
  - `campaign.rs` `worlds_are_mostly_their_element`: the roster includes `uncommon` and `leader`. Expected own-element counts: Shotengai 6/10, Shrine Valley 7/11, Ember Leaf 7/11, Pale Citadel 8/11, Rainward 7/11, Shatterline 7/11; assert ≥ 0.55. `every_enemy_has_a_home_and_rosters_match_ranks` also lists the newcomers and asserts `uncommon[*].rarity() == Uncommon`, `leader.rarity() == Leader`.
  - `enemies.rs` `leaders_are_not_bosses`: for each Leader, `!is_boss()`, `boss_phases(k, 2) == 1`, `chest_tier(k) == 2`, `coin_drop(k) == 5`.
  - `meta.rs` `old_save_keeps_enemy_kills`: build a profile text with `enemy.<key>=<n>` for the 47 old kinds only, parse, and assert each count; new kinds are 0; `serialize(parse(serialize(p))) == serialize(p)`.
- [ ] **Step 2: Run** `cargo test -p sim --release`: FAIL (variants / fields missing).
- [ ] **Step 3: Implement** the Roster table data and the Interfaces. Every `match` on `EnemyKind` covers the new kinds (no `_` arm added where one did not exist).
- [ ] **Step 4: Run** `cargo test -p sim --release` (all pass), `cargo test -p client`, and `cargo build -p client --release` (0 warnings).
- [ ] **Step 5:** Show the diff for review.

### Task 2: Spawning: Uncommon wave and share, Leader rounds and crew

**Files:**
- Modify: `crates/sim/src/content.rs`: `UNCOMMON_SHARE` 0.03 next to the spawn shares.
- Modify: `crates/sim/src/campaign.rs`: `pub const LEADER_ROUNDS: [u8; 3] = [12, 15, 18];`.
- Modify: `crates/sim/src/world.rs`: `start_round` (round 4: ring of `(24.0 * density) as usize` alternating `st.uncommon[0]`/`[1]`, chest kept; rounds in `LEADER_ROUNDS`: `spawn_leader`); `director` (from round 5, `UNCOMMON_SHARE` of refill spawns pick an Uncommon uniformly, checked before the ranged branch); `Event::LeaderSpawned { kind: EnemyKind }`.

**Interfaces:**
- Consumes: `StageDef.uncommon`, `StageDef.leader`, `EnemyKind::is_leader` (Task 1).
- Produces: `fn spawn_leader(&mut self)` spawns `st.leader` at `ring_point(cfg.spawn_radius)` with a crew of `(10.0 * density) as usize` × `st.fodder`, evenly spaced on a 50-unit circle around the Leader. It does nothing if any live enemy `is_leader()`. Pushes `Event::LeaderSpawned`.

- [ ] **Step 1: Write the failing tests** in `world.rs` tests:
  - `round_four_spawns_uncommons`: Shrine Valley world, advance to round 4; ≥ 20 live `ShardMoth`/`BoneEaterCrow` and 0 newly ringed `PaperCrane` from the headline; a chest exists.
  - `uncommons_join_the_mix`: count refill spawns of Uncommons over rounds 5–10 on a seed. Their share of refill spawns lies in 1.5–6%, and is 0 before round 4.
  - `leaders_arrive_on_their_rounds`: `LeaderSpawned` fires at the start of rounds 12, 15, 18 when the previous Leader has been killed (kill it with `kill_step`); never in round 20; round 12 spawns 0 extra `st.elite`.
  - `round_skip_spawns_one_leader`: jump `time` from round 11 to round 19 in one tick; exactly one live Leader.
- [ ] **Step 2: Run**: FAIL.
- [ ] **Step 3: Implement** the Interfaces.
- [ ] **Step 4: Run** `cargo test -p sim --release`: all pass.
- [ ] **Step 5:** Show the diff for review.

### Task 3: Banner, scatter and Leader control

**Files:**
- Modify: `crates/sim/src/content.rs`: `BANNER_RADIUS` 140.0, `BANNER_SPEED` 1.25, `BANNER_DAMAGE` 1.25, `SCATTER_TIME` 2.0.
- Modify: `crates/sim/src/world.rs`: `Enemies` columns `banner: Vec<bool>`, `scatter: Vec<f32>`, `scatter_from: Vec<Vec2>` (push in `spawn_enemy`, move in `swap_remove`); `fn update_banners(&mut self)` called each tick before `move_enemies`; speed in `move_enemies`; contact damage in `contact_damage`; scatter on Leader death in `remove_dead_enemies`; `update_bosses` skips a non-boss `Ai::Boss` enemy while `freeze > 0`, `stagger > 0`, or inside the Firmament domain (velocity zero, no move).

**Interfaces:**
- Produces: `Enemies.banner[i]` is true when enemy `i` is not a Leader, not `is_boss()`, and within `BANNER_RADIUS` of a live Leader. A dead Leader sets `scatter = SCATTER_TIME`, `scatter_from = leader pos` for every non-boss enemy within `BANNER_RADIUS`. A scattering enemy moves straight away from `scatter_from` at its own speed and ignores the player. `pub fn leaders(&self) -> impl Iterator<Item = usize>` (indices of live Leaders, for the client).

- [ ] **Step 1: Write the failing tests:**
  - `banner_buffs_only_nearby`: a Leader plus two Wisps at 100 and 200 units. After one tick the near Wisp has `banner` true and moves ~1.25× faster than the far one (compare `vel.length()`, separation off by spacing). Its contact hit on the player is 1.25× (use `hurt` delta at equal armor). The Leader itself has `banner` false.
  - `banner_ends_when_leader_dies`: kill the Leader with `kill_step`. The same tick, no enemy has `banner`, and the near Wisp has `scatter > 0` and moves away from the death point. After `SCATTER_TIME` + 0.1 s it chases the player again.
  - `frozen_leader_holds_still`: set `freeze` 1.0. The Leader's position is unchanged after 30 ticks, and no new `shots` or hazards come from it. Repeat with `stagger[i] = 1.0`.
  - Extend `statuses_follow_swap_remove` with `banner`, `scatter`, `scatter_from`.
- [ ] **Step 2: Run**: FAIL.
- [ ] **Step 3: Implement** the Interfaces.
- [ ] **Step 4: Run** `cargo test -p sim --release`: all pass.
- [ ] **Step 5:** Show the diff for review.

### Task 4: Drops and the gear budget

**Files:**
- Modify: `crates/sim/src/content.rs`: `LEADER_PICKUP_CHANCE` 0.5, `UNCOMMON_GEAR` (start 0.03, then set from Step 4).
- Modify: `crates/sim/src/world.rs` `remove_dead_enemies` (~L3521): Uncommon gear chance `UNCOMMON_GEAR * find` (min `Rarity::Common`) in place of the Fodder rate; the Elite pickup roll uses `LEADER_PICKUP_CHANCE` for Leaders; `balance_full_run` prints per run `uncommon_kills`, `leader_kills`, `gear_drops` (count of `Event::GearDrop`).

**Interfaces:**
- Consumes: `EnemyKind::rarity` (Task 1); spawns (Task 2).

- [ ] **Step 1: Write the failing test** `uncommon_and_leader_drops`: 2000 Uncommon kills give a gear-drop share within ±40% of `UNCOMMON_GEAR`; 2000 Leader kills give a pickup share in 0.4–0.6 and a chest every time; Common kills keep the 0.0004 rate (share < 0.003).
- [ ] **Step 2: Run**: FAIL. **Step 3: Implement.** **Step 4: Run**: pass.
- [ ] **Step 5: Measure:** `balance_full_run`, Riku and Ren, Shrine Valley, Normal, seeds 21–30. Set `UNCOMMON_GEAR = 1.0 / mean(uncommon_kills)` rounded to 3 decimals (target +1 item per run). Record kills, the value, and gear drops per run before and after in "Measured".
- [ ] **Step 6:** Show the diff for review.

### Task 5: Art: Shotengai, Shrine Valley, Ember Leaf (9 sprites)

**Files:**
- Create: `tools/gen_p6.py`: writes the 18 `.px` sources (in the `tools/gen_w*.py` Canvas style): Shotengai to `art/src/<key>.px` (like `wisp.px`), worlds to `art/src/w<n>/uncommon_a.px`, `uncommon_b.px`, `leader.px`. Sizes: Uncommon 24×24, Leader 32×32. Header comment `# <World> <rarity>: <Name> (generated by tools/gen_p6.py)`.
- Create: `tools/hd/enemies_p6.py`: HD redraws at exactly `UP` (4×) of each `.px` size, in the `enemies_w12.py` / `enemies_w5.py` style with `hdkit` (`a.outline()`, `a.save(path)`). Deterministic. Leaders read as captains (coat, hat or banner).
- Modify: `crates/client/src/render.rs` `world_enemy_src`: map all 18 (Shotengai keys `rokurokubi_clerk`, `tsurube_drop`, `captain_kitsunebi`).
- Modify: `crates/client/src/devmenu.rs` (dev only): in run, "Spawn Leader + Uncommons" calls a new `World::debug_spawn_newcomers()` (sim, `pub`, next to `debug_clear_enemies`: `spawn_leader` plus 3 of each Uncommon near the player). Out of run, "Mark every Otherworlder met" sets every `enemy_kills` entry to at least 1.

This task covers the 9 in the first three worlds. Task 6 does the rest in the same two scripts.

- [ ] **Step 1:** `python3 tools/gen_p6.py && python3 tools/hd/enemies_p6.py`, then `cargo build -p client --release` with 0 warnings and no `hd art ... is not 4x` warning at startup (`cargo run -p client --release` for 3 s, or the web console).
- [ ] **Step 2:** `./scripts/build-web.sh`; headless Chrome screenshots (always `--mute-audio`): each sprite in a run (dev "Spawn Leader + Uncommons" per world) and in the codex Otherworlders tab (dev "Mark every Otherworlder met"). Save them to the session scratchpad.
- [ ] **Step 3:** Show the diff and screenshots for review (art is judged by eye: silhouette reads at 1× zoom, matches its world's palette).

### Task 6: Art: Pale Citadel, Rainward, Shatterline (9 sprites)

Same files and steps as Task 5 for the remaining 9. After this task, no kind falls back to the `oni` texture. Add the client test `every_enemy_has_its_own_sprite`: every `ALL_ENEMIES` kind outside the 5 named prologue kinds and `Oni` has `world_enemy_src(k).is_some()`.

### Task 7: Rarity display, codex and README

**Files:**
- Modify: `crates/client/src/sprites.rs`: `pub fn silhouette(s: &Sprite) -> Texture2D` (rgb set to 255, alpha kept, same upscale path as `texture()`).
- Modify: `crates/client/src/render.rs`: `Assets.enemy_outlines: Vec<Texture2D>` built from the same sprite as each enemy texture; in `draw_world_enemy` for `rarity() > Common`, push the silhouette 4× at ±1.5 px offsets tinted with `enemy_rarity_color` before the sprite; chevrons (1 or 2 small `^` shapes drawn with `draw_triangle`, above the sprite) for Uncommon / Elite; Leader: a small flag (pole + pennant, rarity colour) above the head and a faint `draw_circle_lines` at `BANNER_RADIUS` (alpha ≤ 0.15). Banner-buffed enemies get no extra mark (my call: the circle shows the area).
- Modify: `crates/client/src/ui.rs`: `pub fn enemy_rarity_color(r: EnemyRarity) -> Option<Color>` (None for Common, else per Global Constraints); boss bar name prefixed `"SUB-BOSS  ·  "` / `"BOSS  ·  "` in that colour; `LeaderSpawned` shows the existing `ui::banner` with `"<Name> rallies the crew"`.
- Modify: `crates/client/src/menu.rs` `entry` (~L605): the second line becomes `"<World>  ·  <Rarity>  ·  Defeated: n"`, with the rarity word in its colour (or plain when Common). The `"(boss)"` suffix goes.
- Modify: `README.md`: enemy count 47 → 65, the rarity tiers and Leaders where Campaign describes the night.

- [ ] **Step 1: Write the failing client test** `enemy_rarity_colors_match_gear_tiers`: `enemy_rarity_color` equals `rarity_color` of the mapped gear tier for each non-Common rarity; `Common` is `None`; the colours of Uncommon, Elite, Leader, SubBoss and Boss differ pairwise by ≥ 0.12 (`distance`).
- [ ] **Step 2: Run** `cargo test -p client`: FAIL. **Step 3: Implement.** **Step 4: Run** `cargo test -p client` and `cargo build -p client --release` (0 warnings).
- [ ] **Step 5:** Screenshots, as in Task 5: a crowd with Common, Uncommon and Elite enemies plus a Leader with its circle; the Sub-boss bar; the codex line. Check `kit::MARGIN` at 16:9 and a phone aspect. Check frame time with 1500 enemies (dev stress, if available) against `c1eca45`; note it in "Measured".
- [ ] **Step 6:** Show the diff and screenshots for review.

### Task 8: Balance confirm and hand-off

- [ ] **Step 1:** Baseline: `git archive c1eca45 | tar -x -C <scratch>/base`, own `CARGO_TARGET_DIR`. Run the 30 configs (Global Constraints) on seeds 81–90 and 91–100, for both base and phase 6. Command per config: `BALANCE_SEEDS=10 BALANCE_SEED0=81 BALANCE_STAGE=<stage> BALANCE_HERO=<hero> cargo test --release -p sim balance -- --ignored --nocapture`. Expect about 60 min of bot time.
- [ ] **Step 2:** Any config more than 2 wins down: re-run both trees on seeds 111–120. If it misses again, tune `BANNER_*`, `UNCOMMON_SHARE` or the Leader / Uncommon stats on seeds 21–30 only, then re-confirm on 81–100. Record each change.
- [ ] **Step 3:** Final whole-branch review (`code-review` for correctness, `ponytail-review` for size), then a fix round.
- [ ] **Step 4:** Spec: add "Last phase (phase 7)" (final values, rulings, rechecks). This plan: "Measured". Update `handoff.md` and the memory note.
- [ ] **Step 5:** Run the verify block from `handoff.md` (`cargo test -p sim --release`, `cargo test -p client`, `cargo build -p client --release` with 0 warnings, `./scripts/build-web.sh`). Show the final diff. The user commits.

## Measured

**Uncommon gear (Task 4):** Shrine Valley, Normal, seeds 21–30. Uncommon kills per run: Riku ~670–710, Ren
~1100–1150 (mean ~900); Leader kills ~2.7–3 per run. Gear drops per run at the old Fodder rate → with Uncommon gear:
Riku 14.7 → 15.3, Ren 18.4 → 22.9 (unpaired: the RNG stream shifts between builds). At the 0.03 start value: 37.7 / 62.2.
**`UNCOMMON_GEAR` = 0.0015** (ruling: the plan's `1 / mean` = 0.001 ignores the 0.0004 Fodder rate Uncommons replace;
(0.0015 − 0.0004) × ~900 ≈ +1 item per run).

**Rarity display (Task 7):** stress test (~2000 Common enemies, headless Chrome + swiftshader, one sample each):
`c1eca45` 39 fps, phase 6 34–35 fps. Noisy; the outline path (8 batched quads per non-Common enemy) was not isolated.

**Balance vs `c1eca45`** (Normal, 10 seeds per block, wins of 10; phase 6 before Ember Leaf tuning):

| Config | 81–90 base → p6 | 91–100 base → p6 |
|---|---|---|
| Ren Shrine Valley | 10 → 10 | 9 → 10 |
| Riku Shrine Valley | 7 → 6 | 8 → 9 |
| Shun Shrine Valley | 10 → 8 | 9 → 10 |
| Toya Shrine Valley | 10 → 10 | 10 → 9 |
| Kuon Shrine Valley | 10 → 10 | 10 → 9 |
| Ren Shotengai | 6 → 6 | 7 → 4 (re-run 111–120: 6 → 9) |
| Ren Ember Leaf | 8 → 7 | 5 → 2 (re-run 111–120: 5 → 7) |
| Ren Pale Citadel | 8 → 8 | 8 → 7 |
| Ren Rainward | 9 → 10 | 9 → 9 |
| Ren Shatterline | 9 → 10 | 10 → 8 |
| Riku Shotengai | 3 → 6 | 5 → 6 |
| Riku Ember Leaf | 4 → 1 | 3 → 1 (re-run 111–120: 5 → 0, repeated miss) |
| Riku Pale Citadel | 9 → 10 | 9 → 10 |
| Riku Rainward | 8 → 9 | 9 → 9 |
| Riku Shatterline | 9 → 9 | 9 → 9 |

**Ember Leaf tuning** (diagnosis: Fan3 chip damage, the Rogue Puppeteer was a Kunai Thrower clone with 2× HP; the
Mist Captain added ~400–500 contact damage in rounds 9–16). Tuned on seeds 21–60 only:
Rogue Puppeteer `Fan3` every 4.0 s → `Single` every 4.5 s; Mist Captain HP 860 → 640, damage 24 → 16.
Confirm after tuning (base → tuned): Riku 81 4 → 3, 91 3 → 1, 111 5 → 1, 121 2 → 3 (14 → 8 over 40 runs);
Ren 81 8 → 7, 91 5 → 6, 111 5 → 7, 121 7 → 6 (25 → 26). One miss (111) whose re-run (121) passes: accepted under
the miss rule. Riku on Ember Leaf stays the weakest config and is carried as a recheck.
