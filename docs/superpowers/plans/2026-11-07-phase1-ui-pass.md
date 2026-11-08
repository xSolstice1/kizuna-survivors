# Phase 1 — UI Pass Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Full keyboard control of every in-run choice overlay, a whole-screen "ultimate ready" alert, and no UI crowding a screen edge on any aspect ratio.

**Architecture:** Immediate-mode UI in `crates/client` (macroquad, 1280×720 virtual canvas via `kit::ui_scale`). The decisions that can be tested (focus movement, alert timing) live in small pure functions/structs with unit tests; the draw code calls them. Focus state is a `usize` owned by `main.rs` and passed by `&mut` into the overlay functions.

**Tech Stack:** Rust 2021, macroquad 0.4, `cargo test -p client`.

**Spec:** `docs/superpowers/specs/2026-11-07-elements-rarity-design.md` — sections 1, 3 and 8 (phase 1 of 6).

## Global Constraints

- **No commits.** The user commits. Each task ends with "show the diff for review", never `git commit`.
- Keep the existing `1`/`2`/`3` digit shortcuts working everywhere they work today.
- All layout maths in virtual pixels (`kit::screen_width()` / `kit::screen_height()`), never a hard-coded `720.0` or `1280.0` for anchoring.
- Safe edge margin: **24 virtual px** (`kit::MARGIN`) from every screen edge for any button or text.
- Must still build for the web: `./scripts/build-web.sh` passes.

## Review Focus

1. Space pressed to take a level-up card must not also fire the ultimate on the next sim step → ultimate fires only on a fresh Space press during play (Task 2 test step).
2. Ultimate becomes ready while a level-up / chest / shrine overlay is open → alert waits and plays once the overlay closes, if still ready (Task 3 test).
3. Ultimate ready, used, and ready again in the same run → alert plays again each time (Task 3 test).
4. Offers with 2 cards (shrine) or 4 cards → focus wraps within the real count, never indexes past it (Task 1 test).
5. Phone landscape with a notch (viewport-fit=cover) → no button under the notch or home indicator (Task 4 safe-area CSS + screenshot check).

---

### Task 1: Focus navigation model

**Files:**
- Modify: `crates/client/src/kit.rs` (add after `button`, ~L282)

**Interfaces:**
- Produces:
  - `pub struct NavInput { pub prev: bool, pub next: bool, pub confirm: bool, pub digit: Option<usize> }`
  - `impl NavInput { pub fn read() -> NavInput }` — prev = `Left|A|Up|W`, next = `Right|D|Down|S`, confirm = `Enter|KpEnter|Space`, digit = first pressed of `Key1..Key9` as 0-based index.
  - `pub fn nav(focus: usize, n: usize, input: &NavInput) -> (usize, Option<usize>)` — returns (new focus, picked index). Wraps. `n == 0` returns `(0, None)`. A digit `< n` picks that index directly and moves focus there; a digit `>= n` is ignored. Confirm picks the (post-move) focus.
  - `pub fn focus_ring(x: f32, y: f32, w: f32, h: f32, accent: Color)` — the shared focus outline (pulsing `rounded_outline`, 3 px, radius 16).
  - `pub const MARGIN: f32 = 24.0;`

- [ ] **Step 1: Write the failing tests** in a `#[cfg(test)] mod tests` at the end of `kit.rs`:

```rust
fn i(prev: bool, next: bool, confirm: bool, digit: Option<usize>) -> NavInput { NavInput { prev, next, confirm, digit } }

#[test]
fn nav_wraps_and_confirms() {
    assert_eq!(nav(0, 3, &i(false, true, false, None)), (1, None));
    assert_eq!(nav(2, 3, &i(false, true, false, None)), (0, None));
    assert_eq!(nav(0, 3, &i(true, false, false, None)), (2, None));
    assert_eq!(nav(1, 3, &i(false, false, true, None)), (1, Some(1)));
}

#[test]
fn nav_digits_and_bounds() {
    assert_eq!(nav(0, 2, &i(false, false, false, Some(1))), (1, Some(1)));
    assert_eq!(nav(0, 2, &i(false, false, false, Some(2))), (0, None));
    assert_eq!(nav(5, 2, &i(false, false, false, None)), (1, None)); // stale focus clamps
    assert_eq!(nav(0, 0, &i(false, true, true, None)), (0, None));
}
```

- [ ] **Step 2: Run, expect FAIL** — `cargo test -p client nav_` → "cannot find function `nav`".
- [ ] **Step 3: Implement** `NavInput`, `NavInput::read`, `nav`, `focus_ring`, `MARGIN` as specified. Stale focus clamps to `n - 1` before moving.
- [ ] **Step 4: Run, expect PASS** — `cargo test -p client nav_`.
- [ ] **Step 5: Show the diff for review.**

### Task 2: Keyboard control in level-up, shrine and chest overlays

**Files:**
- Modify: `crates/client/src/ui.rs` — `level_up` (~L248–L325), `shrine` (~L330–L372)
- Modify: `crates/client/src/main.rs` — overlay dispatch (~L522–L545), focus state near `special_queued` (~L119)
- Modify: `crates/client/src/chest_ui.rs` — continue prompt (~L244) and its key check

**Interfaces:**
- Consumes: `kit::NavInput`, `kit::nav`, `kit::focus_ring` (Task 1).
- Produces:
  - `pub fn level_up(world: &World, offer: &[Upgrade], assets: &Assets, focus: &mut usize) -> Option<usize>`
  - `pub fn shrine(offer: &[ShrineEvent], assets: &Assets, focus: &mut usize) -> Option<Option<usize>>` — focus indices `0..offer.len()` are cards, `offer.len()` is "Walk away".

- [ ] **Step 1: Level-up.** Replace the `Key1..Key3` loop with `kit::nav(*focus, offer.len(), &NavInput::read())`. Mouse hover sets `*focus = i`. The focused card (not only the hovered one) gets the lift, glow and `kit::focus_ring`. Header subtitle becomes `"Level {n}  ·  ←/→ choose  ·  Enter take"` on keyboard, unchanged on touch.
- [ ] **Step 2: Shrine.** Same model over `offer.len() + 1` entries; picking index `offer.len()` returns `Some(None)` (walk away). Escape still walks away. Focus ring also drawn around the "Walk away" button when focused.
- [ ] **Step 3: Chest.** Continue also accepts `Space` (Enter and click already work); prompt text becomes `"Click, Enter or Space to continue"`.
- [ ] **Step 4: main.rs.** Add `let mut overlay_focus = 0usize;`, pass `&mut overlay_focus` to both calls, and reset it to `0` whenever `world.offer` or `world.shrine_offer` changes from `None` to `Some` (track the previous frame's state in a `bool` `overlay_was_open`).
- [ ] **Step 5: Space-leak check (Review Focus 1).** The existing `else { special_queued = false; }` branch (~L470) should already cover this; only add code if the check fails. Verify by running natively: `cargo run -p client --release`, reach a level-up with a full ultimate meter (dev menu), take a card with Space, confirm the ultimate does **not** fire. Record the result in the review note.
- [ ] **Step 6: Verify.** `cargo test -p client` passes; `cargo build -p client --release` clean; manual run: level-up and shrine fully playable with arrows/WASD + Enter/Space, digits still work, mouse hover and keys agree.
- [ ] **Step 7: Show the diff for review.**

### Task 3: Whole-screen "ultimate ready" alert

**Files:**
- Modify: `crates/client/src/ui.rs` — add `UltAlert` and `ult_alert` draw fn; extract the special colour match in `hud` (~L96–L103) into `pub fn special_color(kind: SpecialKind) -> Color` (base colour, no flicker) and use it in both places.
- Modify: `crates/client/src/main.rs` — own an `UltAlert`, update it each frame after the sim block, draw after the HUD, play `synth::Sfx::Bell` when it fires.

**Interfaces:**
- Produces:
  - `pub struct UltAlert { fired: bool, pub t: f32 }` with `Default`
  - `impl UltAlert { pub fn update(&mut self, ready: bool, overlay_open: bool, dt: f32) -> bool }` — returns true on the frame the alert fires. Fires once per ready period, only when `ready && !overlay_open`; while firing, `t` counts up from 0; `ready == false` resets `fired` so the next ready period fires again.
  - `pub fn ult_alert(alert: &UltAlert, color: Color, touch: bool, ready: bool)` — draws: 3 edge pulses over the first 1.5 s (`t < 1.5`), then a faint steady edge glow while `ready`; centred banner for `t < 2.0`: title **"ALERT! — ULTIMATE READY"**, subtitle `"Press SPACE"` (keyboard) or `"Tap ⚡"` (touch). Banner y = `screen_height() * 0.30` so it never overlaps the stage bar or the top-left hero card.

- [ ] **Step 1: Write the failing tests** (`#[cfg(test)] mod tests` in `ui.rs`):

```rust
#[test]
fn alert_fires_once_per_ready_period() {
    let mut a = UltAlert::default();
    assert!(!a.update(false, false, 0.1));
    assert!(a.update(true, false, 0.1));
    assert!(!a.update(true, false, 0.1));
    assert!(!a.update(false, false, 0.1)); // used
    assert!(a.update(true, false, 0.1));   // ready again
}

#[test]
fn alert_waits_for_overlay_to_close() {
    let mut a = UltAlert::default();
    assert!(!a.update(true, true, 0.1));
    assert!(!a.update(true, true, 0.1));
    assert!(a.update(true, false, 0.1));
}
```

- [ ] **Step 2: Run, expect FAIL** — `cargo test -p client alert_` → "cannot find type `UltAlert`".
- [ ] **Step 3: Implement** `UltAlert::update`, `special_color`, `ult_alert`. `overlay_open` in `main.rs` = `world.offer.is_some() || world.shrine_offer.is_some() || world.chest_result.is_some() || screen == Screen::Paused || map_open`. `ready` = `world.player.special >= 1.0 && world.form_kind().is_none()`. Colour = `special_color(hero_def(world.cfg.hero).special)`. Keep the existing bottom "Press SPACE" prompt and touch-button pulse.
  - shortcut: reuses `Sfx::Bell`; a dedicated rising chime can be added to `synth.rs` later.
  - Phase 4 swaps `special_color` for the hero's element colour.
- [ ] **Step 4: Run, expect PASS** — `cargo test -p client alert_`.
- [ ] **Step 5: Manual check:** native run; fill the meter (dev menu) → edge pulses, banner, bell once; level up while ready → banner waits until the card is taken; with Auto-special on (`U`) the banner does not appear (meter is spent the step it fills).
- [ ] **Step 6: Show the diff for review.**

### Task 4: Screen spacing and safe areas

**Files:**
- Modify: `crates/client/src/armory.rs` — L328 (`720.0 - dy - dh - 22.0`), L493 and L609 (`720.0 - 52.0`, `720.0 - 56.0`) and any other hard-coded `720.0`/`1280.0` anchors found by `grep -n "720\.0\|1280\.0" crates/client/src/*.rs`.
- Modify: `crates/client/src/menu.rs`, `crates/client/src/ui.rs` — only where the screenshot audit shows a violation.
- Modify: `web/index.html` — canvas CSS.

**Interfaces:**
- Consumes: `kit::MARGIN` (Task 1).

- [ ] **Step 1: Baseline screenshots.** Build web (`./scripts/build-web.sh`), serve `dist/`, and with headless Chrome (puppeteer-core, scratch dir, not committed) screenshot every screen — main menu, armory, shrine draw, shrine shop, collections, stats, settings, pause, level-up, shrine choice, chest, results — at viewports **1280×720, 1024×768, 1920×820 (21:9), 844×390 and 932×430** (the last two with `deviceScaleFactor: 3`, `isMobile: true`). Save under the scratchpad.
- [ ] **Step 2: List violations:** any button or text closer than `MARGIN × ui_scale` physical px to an edge, any element off-screen, any overlap. The Armory "Back" button (12 px from the bottom today) must be on the list.
- [ ] **Step 3: Fix anchors.** Bottom-anchored elements use `screen_height() - MARGIN - height`; right-anchored use `screen_width() - MARGIN - width`. Armory content height uses `screen_height()` instead of `720.0`.
- [ ] **Step 4: Safe-area CSS** in `web/index.html`: position the canvas inside the safe area so miniquad sizes it there:

```css
canvas { position: fixed; top: env(safe-area-inset-top); right: env(safe-area-inset-right);
         bottom: env(safe-area-inset-bottom); left: env(safe-area-inset-left);
         width: auto; height: auto; }
```

  (`body` keeps the `#1c1828` background, so the inset strips match the game.)
- [ ] **Step 5: After screenshots** at the same viewports; every violation from Step 2 is gone and nothing new appears. Put before/after pairs side by side in one contact sheet (scratchpad) for the user.
- [ ] **Step 6: Verify** `cargo test -p client` and `./scripts/build-web.sh` pass.
- [ ] **Step 7: Show the diff and the contact sheet for review.**

### Task 5: Attacks only reach a fixed zone inside the screen

User request (2026-11-08): no attack may hit an enemy off screen, and reach must not extend edge to edge; reach must be the same on every device.

**Files:**
- Modify: `crates/sim/src/content.rs` — reach constants.
- Modify: `crates/sim/src/world.rs` — `hurt_enemy` (~L2802), `nearest_enemy` (~L2839), `nearest_index` (~L3426), a `view_half` field on `World`, tests.
- Modify: `crates/client/src/main.rs` — set the view each frame before the sim steps.

**Interfaces:**
- Produces:
  - `pub const REACH_HALF: Vec2 = Vec2 { x: 192.0, y: 108.0 };` and `pub const REACH_EDGE: f32 = 24.0;` and `pub const DEFAULT_VIEW_HALF: Vec2 = Vec2 { x: 320.0, y: 180.0 };` in `content.rs` (use the sim's own `Vec2`; if it has no const constructor, use `(f32, f32)` tuples).
  - Per-hero reach (user request 2026-11-08): `pub reach: f32` on `HeroDef` via `REACH_RANGED = 1.0` (Ren, Kuon, Mika, Nanami), `REACH_HYBRID = 0.9` (Riku, Kaede, Yuzu), `REACH_MELEE = 0.8` (Shun, Toya, Haru); `reach_half()` scales `REACH_HALF` by it. Test `melee_heroes_reach_less`.
  - `pub view_half: Vec2` on `World`, initialised to `DEFAULT_VIEW_HALF` (tests and the balance bot never set it, so they stay deterministic).
  - `pub fn reach_half(&self) -> Vec2` — per axis `min(REACH_HALF, view_half - REACH_EDGE)`, never below 0.
  - `pub fn in_reach(&self, pos: Vec2, radius: f32) -> bool` — true when the enemy's body (circle `pos`, `radius`) overlaps the axis-aligned box centred on `self.player.pos` with half-size `reach_half()`.
- Client: `world.view_half` = half the camera's visible world size, the same numbers `render::camera_for` uses: `(screen_width() / world_scale() / 2, screen_height() / world_scale() / 2)` with macroquad's screen size (not `kit::`).

- [ ] **Step 1: Write the failing tests** in `world.rs`'s test module:
  - `out_of_reach_enemies_take_no_damage`: spawn an enemy at `player.pos + (REACH_HALF.x + 40.0, 0)`, apply damage through the normal path a weapon uses (or `hurt_enemy` directly if the test module can reach it), assert its hp is unchanged; move it to `player.pos + (100, 0)`, apply the same damage, assert hp dropped.
  - `reach_ignores_extra_screen_width`: with `view_half = (900, 180)`, an enemy at `x = REACH_HALF.x + 40` still takes no damage.
  - `small_screens_shrink_reach`: with `view_half = (200, 120)`, `reach_half() == (176, 96)`.
  - `edge_overlap_counts`: an enemy whose centre is `radius - 1` outside the box is in reach.
- [ ] **Step 2: Run, expect FAIL** — `cargo test -p sim --release reach` / names above.
- [ ] **Step 3: Implement.** The only guard is at the top of `hurt_enemy`: `if !self.in_reach(pos, radius) { return; }` (one guard in the shared function; every weapon, burn, chain, explosion, special and Arsenal attack already routes through it). `nearest_enemy` and `nearest_index` skip enemies not `in_reach`, so auto-aim never targets out of reach. Do not touch enemy → player damage or spawning.
- [ ] **Step 4: Run, expect PASS**; then `cargo test -p sim --release` (full) and `cargo test -p client`.
- [ ] **Step 5: Client.** Set `world.view_half` in `main.rs` each frame before the fixed-step loop. Build native release and `./scripts/build-web.sh`.
- [ ] **Step 6: Balance smoke check:** run the balance bot once (`BALANCE_SEEDS=5 BALANCE_STAGE=shrine_valley BALANCE_HERO=riku cargo test --release -p sim balance -- --ignored --nocapture`) before and after; report the change in kills/clear time. No tuning in this task — phase 3 retunes.
- [ ] **Step 7: Show the diff for review.**

### Task 6: Rebalance for attack reach, and remove the invisible wall

User decision (2026-11-08): rebalance now. Task 5's reach rule took Riku on Shrine Valley Normal from 3/5 wins to 0/5 (like-for-like kills ~3× lower; XP snowball → level 40–52 instead of 143–164).

**Files:**
- Modify: `crates/sim/src/world.rs` — projectile hit loop (~L3920–3946), homing search (~L3890–3903), `random_targets` fallback (~L3480), comment at ~L2821.
- Modify: `crates/sim/src/content.rs` — balance constants; `REACH_EDGE` doc comment placement (~L18–22).
- Possibly modify: `crates/sim/src/campaign.rs` / `enemies.rs` only if a lever lives there.

**Part A — invisible wall (tests first):**
- [ ] `projectiles_pass_out_of_reach_enemies`: a projectile flying through an out-of-reach enemy keeps its pierce count and does not burst there; it still hits an in-reach enemy behind it.
- [ ] `homing_ignores_out_of_reach`: a homing projectile with one enemy out of reach and one in reach steers to the in-reach one.
- [ ] `random_targets` fallback point (no enemy in reach) lies inside `reach_half()` of the player.
- [ ] Implement: `continue` past out-of-reach enemies in the projectile hit loop; `in_reach` filter in the homing search; clamp the fallback point to the reach box. Move the `REACH_EDGE` doc line onto `REACH_EDGE`; make the ~L2821 comment state only the why.

**Part B — balance (bot-measured):**
- [ ] Baseline: extract the pre-reach code (tree `0d9d134959bf5ef4b73cadb075e9fb6055fde39b`) with `git archive 0d9d134959bf5ef4b73cadb075e9fb6055fde39b | tar -x -C <scratch>/baseline` and run the balance bot there for **Ren (ranged), Riku (hybrid), Shun (melee)** on `shrine_valley`, `BALANCE_SEEDS=10`. Record wins, median kills at death/win, median level at 15:00.
- [ ] Targets (same command on the working tree): per hero, wins within ±1 of its baseline out of 10, and median level at 15:00 within 25% of baseline. Melee and hybrid heroes must not trail ranged by more than 1 win out of 10 beyond what the baseline already showed.
- [ ] Levers, fewest first, each a **named constant in `content.rs`** with a one-line why: (1) XP gained per gem or `xp_needed` curve; (2) per-reach-class damage bonus (e.g. melee/hybrid compensation applied in `World::might()` from `HeroDef.reach`); (3) enemy HP / spawn pacing curves. Do not change the reach constants (user-fixed). Do not change individual weapons.
- [ ] Sanity: Ren on `shotengai` Normal, 5 seeds, still winnable (≥ baseline − 1).
- [ ] `cargo test -p sim --release`, `cargo test -p client`, `cargo build -p client --release` (0 warnings), `./scripts/build-web.sh`.
- [ ] Report a before/after table per hero and every lever changed with its value.
- [ ] Show the diff for review.
