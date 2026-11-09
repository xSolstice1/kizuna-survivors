# UI Pass After Elements and Set Gear — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Every UI surface shows elements one consistent way, nothing new crowds or breaks on desktop or phone, and the per-frame UI costs the audit found are gone.

**Architecture:** Client-only (`crates/client`), plus one sim string constant. One element language — a diamond pip + Title-case name drawn by `ui::element_tag` — replaces the pill (`element_chip`) and the seven ad-hoc formats; pills stay for rarity. Each surface then adopts the helper. Perf fixes are local (sort once, cheap counts, stack arrays, cached strings).

**Tech Stack:** Rust 2021, macroquad client; web build `./scripts/build-web.sh`; headless Chrome screenshots over CDP.

**Spec / source of findings:** `.superpowers/sdd/2026-11-08-phase5-gear-sets/ui-audit-report.md` (46 findings with IDs, file:line and the smallest fix; screenshots in the session scratchpad `ui-audit/shots/`). Design spec: `docs/superpowers/specs/2026-11-07-elements-rarity-design.md` (§8 spacing, anti-goal "colour never the only carrier").

## Global Constraints

- **No commits.** The user commits. No AI attribution anywhere.
- Virtual 1280×720 canvas via `kit::screen_width()/screen_height()`; everything ≥ `kit::MARGIN` (24) from every edge; never anchor to literal 720/1280.
- Nunito lacks `⚡ ← →`; `—` and `·` are fine.
- Colour is never the only carrier: an element is always pip **and** name, except compact cells/HUD slots where the full tag is one tap/hover away (bag cell, draw cell, HUD slot).
- Element text minimum size **14** (virtual) everywhere (phone 844×390 renders 12 at ~6.5 CSS px).
- Pills (filled rounded chips) are reserved for rarity/tier; elements never use a pill.
- No frame-time regressions on phones: no new per-frame allocations in hot loops.
- Headless Chrome always with `--mute-audio`. Screenshots at 1280×720 and 844×390 for every surface a task touches.
- Behaviour/data unchanged except: K1 text, H1 HUD offset, O1 rotate prompt, Light colour tweak.

## Rulings (controller, on the user's behalf — user pre-approved building from the audit)

- **H1:** the HUD follows `kit::MARGIN` like every other surface (spec goal 8 "no screen element crowds a screen edge"). Cost if wrong: HUD sits 12 px further in.
- **C4:** Light colour darkens to about (1.0, 0.88, 0.55) so it reads as coloured next to INK; `element_colors_are_distinct` extends to compare elements against every rarity colour and the theme colours GOLD / JADE / INK (min distance agreed by the test, ≥ 0.12).
- **C7:** only the size floor (14) ships now; the "touch-only UI_H ~600" idea is out of scope.
- **A6** (cache wrapped detail text) is skipped as low value. **N2** ships with N1 (centre numbers by cached width).
- **O1:** a "rotate your device" overlay when `screen_height() > screen_width()` on touch.

## Review Focus

1. Element pip + name must never collide with rarity pills or fall below size 14 on a 844×390 phone (screenshots).
2. The bag (300+ items) and the Collection must not regress in behaviour: same sort order, same found counts, cursor still works (tests where logic changed).
3. Damage numbers: cached text must match the amount, still centre-anchored and move with the player (phase 5 change).
4. Nothing drawn outside `kit::MARGIN` after the HUD offset (HUD, alert, round strip, chest hint).
5. Light still distinct from Ice and from GOLD after the colour tweak.

---

### Task 1: Element language and the broken string

Audit IDs: **C1, C2, C4, K1, L1, L2, P1, P2** (and the size floor from C7).

**Files:** `crates/client/src/ui.rs` (helpers, level-up, replace picker, colour test), `crates/client/src/menu.rs` (K1 recipe text), `crates/client/src/armory.rs` (affinity line uses `affinity_text`).

**Interfaces (produced, used by Tasks 3–4):**
- `pub fn element_tag(e: Element, x: f32, y: f32, size: f32) -> f32` — diamond pip (`draw_poly(.., 4, r, 45.0, colour)`, r ≈ size × 0.3) then Title-case name in the element colour; `y` is the text baseline; returns total width. `size` is clamped to ≥ 14.
- `pub fn element_tag_width(e: Element, size: f32) -> f32` — for right alignment (no second measure inside `element_tag`).
- `pub fn element_pip(e: Element, x: f32, y: f32, r: f32)` — compact pip for cells/HUD.
- `pub fn affinity_text(e: Element) -> String` — `"<E> affinity: +10% <E> damage"` (value from `AFFINITY_DAMAGE`).
- `element_chip` is deleted.

- [ ] Extend `element_colors_are_distinct` (elements vs all rarity colours + GOLD/JADE/INK, and Light vs Ice) and add a test that the K1 recipe string contains `fmt_time(AWAKEN_FROM)` (build the recipe text through a small fn you can test). See them fail.
- [ ] Implement the helpers; level-up card puts `element_tag` on the tag row after the tier pill (L1) and the matchup line uses pips (L2); replace picker computes the width once and shows the incoming skill's element in the header (P1, P2); armory/codex affinity lines use `affinity_text`; K1 recipe text from `AWAKEN_FROM`; Light colour per ruling.
- [ ] `cargo test -p client`, `cargo build -p client --release` (0 warnings), `cargo test -p sim --release`; web build; screenshots of level-up (weapon card with element), replace picker, collection Awakenings at 1280×720 and 844×390.

### Task 2: Per-frame cost

Audit IDs: **A5, K3, C6, N1, N2**.

**Files:** `crates/client/src/armory.rs` (bag sort), `crates/client/src/menu.rs` (collection counts), `crates/client/src/kit.rs` (`rounded`), `crates/client/src/render.rs` (damage numbers).

- [ ] Tests first where logic changes: a client test that the armory bag order for a fixed profile (mixed rarities/levels, ≥ 300 items incl. Set) is identical to the previous ordering function (keep a copy of the old key in the test); a test that `found(profile, tab, i)` agrees with `entry(..).found` for every tab/index of a sample profile.
- [ ] A5: collect `Vec<&Item>` and `sort_by_cached_key` (no `profile.item(id)` lookups in the key). K3: `found()` for counts and cells; full `entry()` only for the cursor. C6: `kit::rounded` uses a stack array. N1/N2: `DamageNumber` stores its text and width at push; elemental sizes 22/28; draw centred.
- [ ] Verify commands as Task 1; screenshots of the armory bag and the collection grid unchanged at 1280×720.

### Task 3: Out-of-run surfaces

Audit IDs: **M1, S1, S2, S3, S4, K2, T1, A1, A2, A3, A4, A7, A8, R1, R2, R3, D1**.

**Files:** `crates/client/src/menu.rs`, `crates/client/src/armory.rs`, `crates/client/src/devmenu.rs`.

- [ ] Implement each audit fix as written in the report (smallest fix column), using Task 1's helpers. A4: element pip in every item cell's top-right (bag, doll, draw, results), Set rarity gets a double outline, legacy set keeps a distinct marker. A3: extras deduped with counts (`Eclipse ×2`), resist line as pip + name per element, stat grid readable (short labels or 2 columns, ≥ 13). S3: wrap to two lines at 13.
- [ ] Verify commands; screenshots of main menu, stage select, collection (each tab), stats, armory (bag, item detail with Set + legacy, loadout totals), shrine draw results, dev menu at 1280×720 and 844×390.

### Task 4: In-run surfaces

Audit IDs: **H1, H2, H3, H4, H5, U1, U2, Q1, Q2, Q3, Z1, O1**.

**Files:** `crates/client/src/ui.rs` (HUD, pause, results, alert, rotate prompt), `crates/client/src/render.rs` (Mini+ ring colour), `crates/client/src/chest_ui.rs`.

- [ ] Implement each audit fix as written. H1 per ruling. H5: Mini+ ring in the enemy element colour, boss bar name with `element_tag`. U1: weapon element tags, affinity tag, one gear line ("<Set> set (n) · <Element> resist n%" from `world.cfg.gear`; omit when no gear). Q1: chest label via `armory::rarity_color`, rename chest palette `chest_color`. O1 per ruling.
- [ ] Verify commands; screenshots of HUD early and late, ultimate alert, level-up, chest, pause (with gear), results, and portrait 390×844 (rotate prompt).

### Task 5: Helper cleanup

Audit ID: **C5** (delete duplicated `wrap`, `centered`, `fit_size`, `fmt_time` copies; call `kit::`). Pure refactor — `safe-refactor`: no behaviour change, tests before and after identical, screenshots of two surfaces identical.

## Measured

All 46 audit IDs implemented except A6 (skipped per plan). Verify: sim 154, client 14, release 0 warnings, web build OK.
Screenshots (session scratchpad): `ui-audit/shots/` (before), `ui-pass/t1-keep`, `t2`, `t3`, `t4`, `t5`, `final` (after).
Changes beyond the audit: element colours Water / Wind / Darkness also moved (Wind is lime) to stay distinct from rarity
and theme colours; `Loadout.sets5` added in sim so pause and armory share the set tier; portrait mid-run pauses the run.
Results screen (fixed after the final review): reward lines step 24 px from `py + 308`, the loot row moved down, so 4
lines fit with loot; order is hero unlock, new world, first clear, relic, Nightmare+, then codex (dropped first). Deferred minors: see the SDD ledger (Light vs GOLD 0.143, `element_tag`
double-measure on right-aligned callers, S3 `take(2)`, `found()` test mirrors code, `gear_line` width on long set names).
