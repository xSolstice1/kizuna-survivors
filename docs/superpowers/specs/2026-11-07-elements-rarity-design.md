# Elements, Rarity, Sets and New Enemies — Design

Date: 2026-11-07. Status: approved 2026-11-07.

## Intent

Kizuna Survivors has gear rarity but nothing that makes rarity *mean* something in a run, and the
in-run layer (skills, passives, pickups, enemies) has no rarity at all. This change adds one shared
language across the game:

- **Rarity** on skills, passives, in-run items, gear and enemies, so players can read power at a glance.
- **Elements** on skills, gear, enemies, heroes and worlds, with a strength/weakness cycle, so build
  choices and world choice interact and rare gear is worth chasing.
- **Set gear** as the top gear rarity, with 2/4/5/6-piece bonuses, farmed per world.
- **18 new enemies** that fill new enemy rarity tiers, with lore drawn from shonen anime archetypes.
- A batch of **UI fixes** (keyboard level-up, ultimate-ready alert, XP bar, screen spacing).

Success: a player can tell from colour alone how strong a card, item or enemy is; picks an element
build for a world on purpose; and has a long-term goal (complete a world set).

## Goals

1. Full keyboard control of level-up (and every choice overlay).
2. Skills can be replaced at any time; the new skill inherits half the old skill's level.
3. A whole-screen "ultimate ready" signal.
4. Stages split into 20 rounds that introduce enemy tiers evenly, shown on a restyled progress bar.
5. Six enemy rarity tiers and 18 new enemies.
6. Rarity on skills, passives, items, gear (gear adds a Set tier).
7. Set gear bonuses at 2, 4, 5 and 6 pieces, with ring-pair rules.
8. No screen element crowds a screen edge, on any supported aspect ratio.
9. Elements with a damage cycle, inherited by skills, gear, enemies, heroes and worlds.

## Non-goals

- No renames. Existing enemies keep their names and their internal `Rank`
  (Fodder/Elite/Mini/Boss/Final). Gear keeps Common…Mythic and Exotic. Rarity and element are
  **additional** attributes.
- No per-skill rarity rolls: each skill has **one fixed** rarity based on its strength.
- No new heroes, worlds or game modes.
- No lifted anime characters: new enemies are original names and designs built from archetypes.

## Anti-goals (failure even if it works)

- Light/Darkness becoming the only sensible build.
- Element multipliers making a world unbeatable without the "right" element.
- Existing players losing items or progress on update.
- Colour as the *only* carrier of rarity/element (each also gets a label or icon shape).

## Constraints

- Rust/macroquad, sim in `crates/sim` (deterministic, headless-testable), client in `crates/client`.
- Saves are text (`Item::serialize`/`parse`, `Profile` save). Old saves must load.
- New art is drawn with the `tools/hd` Python scripts (hdkit, Pillow), same style as the current HD art.
- Must keep running on phones in the browser (WASM build); no frame-time regressions.

---

## 1. Level-up keyboard control

Today level-up accepts only `1`/`2`/`3`. Add:

- `←/→` and `A/D` (and `↑/↓`, `W/S` where the cards stack) move a visible focus ring between cards.
- `Enter` or `Space` takes the focused card. `1`–`n` still take a card directly.
- Focus starts on the first card. Mouse hover moves focus too, so mouse and keys never disagree.
- The same focus model applies to the replace-a-skill picker (section 2), the chest overlay and the
  shrine-event choice.
- Space must not also fire the ultimate on the frame a card is taken.

## 2. Replacing skills

- Weapon slots (4) and passive slots (6) stay as they are.
- When slots of a kind are full, level-up offers may still include **unowned** skills of that kind.
- Taking one opens a picker: "Replace which skill?" listing owned skills of that kind with their
  levels, plus "Cancel" (back to the cards). Keyboard-navigable.
- The new skill's level = `max(1, floor(old_level / 2))`. Example: replace a Lv 7 → new skill at Lv 3.
- The replaced skill is gone completely: its passive bonuses, awakening and any evolution progress
  are removed. The new skill is never awakened on arrival.
- The hero's signature weapon can be replaced too (my call). Its Tier-1 ultimate does not depend on it.
- Offer weighting: rarity weights apply only to skills you don't own yet; levelling an owned skill
  stays uniform as today. At most **one** replacement card per offer as the starting rule.
- Balance (user request): the rarity weights, the replacement-card cap and the inherit-half rule are
  tuned together with the bot against the new mechanics (elements, rounds, Leaders). Targets: a typical
  run still fills all 4 weapon slots by about round 8 and maxes at least 2 weapons; Unique skills appear
  in roughly 1 run in 3; replacement is a real choice, not an every-level-up card.
- Awakening (my call): an awakened weapon stays awakened if its complement passive is later replaced.
  Replacement only touches the skill being replaced.

## 3. Ultimate-ready alert

When the special meter becomes ready:

- A full-screen pulse: the screen edges glow in the hero's element colour (section 9), 3 pulses
  over ~1.5 s, then a faint steady edge glow until the ultimate is used.
- A centred banner for ~2 s: **"ALERT! — ULTIMATE READY"**, subtitle "Press SPACE" on keyboard,
  "Tap ⚡" on touch.
- A short rising chime (synth, `crates/client/src/synth.rs`).
- Not repeated while it stays ready. Suppressed while a level-up or other overlay is open; it plays
  when the overlay closes if the ultimate is still ready.
- Auto-special (the `U` power-up) still works; if auto-special fires immediately, skip the banner.

## 4. Stage rounds and the progress bar

Today a stage is one continuous night with timed events (Mini at 8:00 or 12:00, an optional mid
Boss, the Final boss at 15:00 or 25:00), shown as a thin 6 px bar with tick marks
(`stage_progress` in `crates/client/src/ui.rs`). Players can't tell where they are or what's next.

### 20 rounds per stage

- Every stage is split into **20 rounds**. Rounds 1–19 split `[0, final_at]` equally (~47 s on
  Shotengai, ~79 s on the 25-minute worlds); **round 20 starts when the Final boss appears and lasts
  until it dies or dawn** (my call). Rounds are **time-based**: enemies keep flowing as today.
- The enemy tiers are spread evenly: **5 blocks of 4 rounds**. The last round of each block is a
  headline round for that block's tier, and from then on the tier joins the regular spawn mix:

| Rounds | Headline round | Introduces |
|---|---|---|
| 1–4 | Round 4 | Uncommon wave |
| 5–8 | Round 8 | Elite |
| 9–12 | Round 12 | Leader |
| 13–16 | Round 16 | Sub-boss (existing Mini) |
| 17–20 | Round 20 | Boss (existing Final; worlds with a mid Boss get it at round 18) |

- Difficulty steps up every round (enemy HP, damage and crowd size), with a bigger step on headline
  rounds. This replaces the continuous time curves; the bot re-tunes so total run difficulty stays
  roughly where it is today.
- Existing events (shrines, swarm rings, crisis, elite waves) are re-anchored to round boundaries.
- Existing encounters move to their rounds: e.g. Shotengai Mini 8:00 → round 16 (~11:45), first
  elite 2:00 → round 8; 25-minute worlds Mini 12:00 → round 16, mid Boss → round 18. A headline lands
  every 4 rounds, so the gaps are filled by the Uncommon and Leader rounds.
- The round-4 Uncommon headline drops a treasure chest, replacing the early elite chest players get
  today at 2:00, so early build pace is unchanged.
- Dawn limit unchanged: the Final boss must die before dawn.

### Progress bar

- Restyled to the house UI kit (`crates/client/src/kit.rs`): panel, radius, outline and gradient
  like the other bars, thick enough to read on a phone.
- Label **"Round 12 / 20"** plus the next headline ("Leader in 1 round").
- 20 segments; headline rounds marked with their tier's rarity colour and an icon.
- A banner at the start of each headline round ("ROUND 12 — LEADER").
- After the Final boss appears, the bar turns into the dawn countdown, as it does today.

## 5. Enemy rarity and new enemies

### Enemy rarity tiers (new attribute, `Rank` unchanged)

| Enemy rarity | Who | Colour cue |
|---|---|---|
| Common | existing Fodder | none |
| Uncommon | **new** (2 per world) | green nameplate/outline |
| Elite | existing Elite | blue |
| Leader | **new** (1 per world) | purple + banner |
| Sub-boss | existing Mini | orange |
| Boss | existing Boss and Final | red |

Effects of rarity: label + outline colour; XP and drops scale by tier (Uncommon ≈ 2× Common XP and a
small gear chance; Leader drops like an Elite and can drop a pickup). Exact numbers to the builder,
tuned with the bot.

### Leaders

A crew captain (One Piece archetype). First appears in the round-12 headline round (section 4), then
returns every 3 rounds; the round system schedules it (my call). While alive, enemies within its banner radius gain speed and damage; killing it
breaks the buff and briefly scatters (frightens) its crew. It has its own movement pattern and one
signature attack.

### The 18 new enemies

Original names and designs. Each world: 2 Uncommon + 1 Leader. Lore leans on that world's existing
inspiration; Leaders lean on One Piece. Draft roster (names, element and archetype; final lore and
behaviour to the builder, following the house lore voice of one or two short sentences):

| World | Uncommon | Uncommon | Leader |
|---|---|---|---|
| Shotengai (yokai: Kitaro/Natsume) | Rokurokubi Clerk (Earth): neck-stretching shopkeeper | Tsurube Drop (Wind): drops from awnings | Captain Kitsunebi (Fire): lantern-ship captain with a paper crew |
| Shrine Valley (Inuyasha/Touhou) | Shard Moth (Wind): a demon drawn to jewel shards | Bone-Eater Crow (Wind): carrion familiar | Admiral Tengu (Wind): a fallen mountain-tengu fleet captain |
| Ember Leaf (Naruto) | Rogue Puppeteer (Earth): strings a dummy ahead of it | Exploding Clay Bird (Fire): sculpted bomb that dives | Mist Swordsman Captain (Water): leads a squad from fog |
| Pale Citadel (Bleach) | Masked Fracción (Ice): sword released mid-fight | Hollow Husk (Ice): a towering, mindless hollow | Espada Quartermaster (Ice): numbered, commands the gate guards |
| Rainward (Tokyo Ghoul) | Kagune Runner (Water): sprints with a blade-tail | Mask Collector (Electric): wears its victims' masks | Drowned Pirate Captain (Water): flood-born crew |
| Shatterline (Jujutsu Kaisen) | Grade-Three Swarm Mother (Earth): sheds small curses | Cursed Tool Thief (Electric): blinks and steals pickups | Curse User Captain (Earth): a sorcerer gang boss |

Light and Darkness are reserved for Legendary-and-above things, so the new enemies use common
elements. Names in this table are working names; the builder may refine for tone but keep the
archetype and element.

Art: one HD sprite set per new enemy via `tools/hd` (new script(s) in the style of `enemies_w*.py`),
plus `.px` sources if the client requires them.

## 6. Rarity everywhere

### Skills and passives (fixed tier by strength)

Tiers: **Common, Uncommon, Rare, Legendary, Mythic, Unique**.

- Each of the 26 weapons and 10 passives gets exactly one tier.
- Tier is assigned by measured strength: the builder benchmarks each skill at max level with the
  headless bot (`crates/sim/src/bot.rs`, `crates/bench`) and buckets them, roughly
  Common 30%, Uncommon 25%, Rare 20%, Legendary 13%, Mythic 8%, Unique 4% of skills.
- Effects: card frame colour and tier label; offer weight
  (Common 1.0, Uncommon 0.8, Rare 0.6, Legendary 0.4, Mythic 0.25, Unique 0.15 — my call, tunable).
  Signature weapons are always given at start regardless of tier.

### In-run items

Pickups (`PickupKind`) and the maxed-out fallbacks (Heal, Sen, Blessing) each get a fixed tier on the
same scale, shown on their card/label. Their existing spawn weights stay the source of truth; the tier
is chosen to match those weights.

### Gear

Existing tiers stay: Common, Uncommon, Rare, Epic, Legendary, Mythic, Exotic. Add **Set** above Exotic.

| | Set |
|---|---|
| Rank | highest gear rarity (Common < … < Exotic < Set) |
| Slots | only the 6 slots of its world's set (section 7) |
| Bonuses | 6 rolled bonuses at high quality (Exotic-level rolls), stat multiplier above Exotic |
| Unique effect | yes: one random unique from `ALL_UNIQUES`, like Legendary/Mythic |
| Element | always its set's element |
| Salvage / upgrade / reroll | priced above Exotic |

Set membership becomes the Set rarity itself: newly rolled non-Set items no longer carry a set tag.

## 7. Set bonuses (2/4/5/6)

Each world set is exactly 6 pieces. Rings always come as a pair: a set either uses both ring slots or
none. **A set's rings only count when both are worn (the pair counts as 2 pieces).** So no tier ever
needs an odd number of rings. Arsenal is never a set piece.

| Set (world) | Element | Pieces |
|---|---|---|
| Lantern Street (Shotengai) | Fire | Hat, Top, Bottom, Gloves, Shoes, Charm |
| Moonveil (Shrine Valley) | Earth | Hat, Top, Necklace, Earring, Ring, Ring |
| Ember Leaf (Ember Leaf) | Wind | Top, Bottom, Gloves, Shoes, Ring, Ring |
| Pale Gate (Pale Citadel) | Ice | Hat, Top, Bottom, Necklace, Earring, Omamori |
| Rainward (Rainward) | Water | Gloves, Shoes, Necklace, Omamori, Ring, Ring |
| Shatterline (Shatterline) | Electric | Hat, Gloves, Earring, Charm, Ring, Ring |

A set's element is always its world's dominant element, so a world's set helps you *in* that world
(its resistance covers that world's hits). One set per common element (my call).

Bonus tiers:

- **2-piece:** the existing 2-piece stat bonus.
- **4-piece:** the existing 4-piece effect.
- **5-piece:** +25% damage of the set's element and +25% resistance to it.
- **6-piece:** a new signature effect per set that turns the set's element into a build (e.g. Rainward:
  every hit soaks; soaked enemies take +30% Electric; Pale Gate: slowed enemies shatter on death for
  area Ice damage). Exact six effects to the builder, one per set, each themed on its element.

### Set drop rates

- Only Mini, Boss and Final kills in a world can drop that world's Set pieces (3 chances per run).
- Chance per eligible kill: `SET_CHANCE × exotic_mult(difficulty, ascension) × (1 + gear_find)`.
- Target: collecting all 6 distinct pieces on Normal takes about **40 runs** (my call). Collecting 6
  distinct pieces takes ~14.7 drops on average (6 × H₆), so Normal needs ~0.37 pieces per run, i.e.
  `SET_CHANCE ≈ 0.12`. Higher difficulty and Ascension shorten it through `exotic_mult`.
- The builder verifies the target with a headless simulation and adjusts `SET_CHANCE`.
  **Built (phase 5): `SET_CHANCE` 0.13.** The bot measured 2.8 eligible kills per won Normal run (Riku,
  Shrine Valley, seeds 21–30: 3 kills in 8 runs, 2 in 2), which put 0.12 at ~45 runs; 0.13 gives ~40.5.
- Duplicate pieces still drop (salvage them for more Dust than Exotics).

## 8. Screen spacing

- Audit every screen (menu, armory, shrine draw, shop, collections, stats, settings, pause, level-up,
  chest, results) at 16:9, 4:3, 21:9 and phone landscape (e.g. 844×390 and 932×430).
- Rule: no button or text sits closer than a fixed safe margin (scaled by `ui_scale`) to any edge, and
  never inside the device's safe area insets.
- Known case: the Armory "Back" button is too close to the bottom edge.
- Deliver before/after screenshots for each screen and ratio.

## 9. Elements

Eight elements: **Fire, Water, Electric, Earth, Wind, Ice** (common) and **Light, Darkness** (rare).

### The cycle

```
Water ─douses─▶ Fire ─melts─▶ Ice ─stills─▶ Wind ─erodes─▶ Earth ─grounds─▶ Electric ─shocks─▶ Water
```

| Attacker → target | Damage |
|---|---|
| Common → the element it beats | ×1.5 |
| Common → the element that beats it | ×0.75 |
| Common → other common | ×1.0 |
| Common → Light or Darkness | ×0.9 |
| Light or Darkness → common | ×1.15 |
| Light ↔ Darkness | ×1.5 both ways |
| Same element | ×1.0 |

Light and Darkness are rare: only skills, gear and enemies of Legendary rarity or above (or Sub-boss /
Boss enemies) can carry them.

### What inherits an element

| Component | Rule |
|---|---|
| Skills (weapons) | One fixed element each. Damage uses the table against the target's element. |
| Passives | No element (they are stat boosts). |
| Heroes | An affinity element: +10% damage for skills of that element; the ultimate deals that element. Offence only: the hero has no defensive element, so enemy hits are never multiplied by the cycle; gear resistance is the only elemental defence (my call). |
| Enemies | One element each. Decides what hurts them most and the element of their hits on the player. |
| Worlds | A dominant element: most of that world's enemies carry it; the world's set uses it. |
| Gear | Each item has one element: a "+X% <element> damage" bonus and "<element> resistance" (reduces damage from enemies of that element, capped at 60% total). C–Mythic roll a common element; Legendary+ may roll Light/Darkness (10%); Arsenal takes its weapon's element; Set takes its set's; Exotics get a fixed one. |
| On-hit effects | Small chance per hit (≈15%, builder-tuned): Fire burns, Water soaks (soaked targets take +30% Electric), Electric chains to one more target, Earth staggers, Wind knocks back, Ice slows, Light marks (next hit crits), Darkness drains a little life. |
| Pickups, coins, chests | No element. |

### Draft assignments (builder may rebalance with the bot)

Worlds (dominant element = set element, my call): Shotengai Fire (lanterns), Shrine Valley Earth
(kodama, stone komainu), Ember Leaf Wind (leaf shinobi), Pale Citadel Ice, Rainward Water,
Shatterline Electric. About 60% of a world's enemies carry its element; the rest are mixed. World
bosses may differ and may carry Light/Darkness (e.g. Mitsuhime and Pale Seraph Light; Hollow Monarch
and Shatter Sovereign Darkness; Burning Shogun Fire).

Heroes: Ren Light, Riku Fire, Shun Darkness, Toya Earth, Kuon Ice, Kaede Electric, Mika Water,
Haru Wind, Nanami Fire, Yuzu Electric.

Weapons: rules for the builder rather than a fixed table —
- each common element has at least 3 weapons; Light ≤ 3 and Darkness ≤ 3, all of Legendary tier or above;
- element follows the weapon's name and look (ThunderChain Electric, FlamePool Fire, GaleShuriken Wind,
  VoidLance Darkness, PurificationWave Light, …);
- the result is shown to the user for sign-off before balancing.

## Attack reach (added 2026-11-08)

Attacks only damage or target enemies inside a reach zone around the player: never off screen, never
edge to edge, same size on every device. Base zone 192 × 108 world units each side (60% of a 16:9
desktop view), shrunk on small screens to the visible area minus 24 units. Each hero scales it by
class: ranged/mage ×1.0 (Ren, Kuon, Mika, Nanami), hybrid ×0.9 (Riku, Kaede, Yuzu), melee ×0.8
(Shun, Toya, Haru). Phase 3 balancing compensates melee heroes so they are not simply weaker.
Phase 1 already set XP_GAIN 2.0, MIGHT_MELEE 2.3 (melee) and BOSS_HP 0.5 (shortcut). Screen-clearing effects
(Exorcism Seal, ThousandCounter) hit the whole visible screen; enemies outside reach are drawn dimmed.

## Save compatibility

- Old saves must load with every item intact.
- Items get an element on load if they have none (deterministic from the item id, so it never changes).
- Old set-tagged items (user decision): **kept as legacy**. They keep their rarity and keep counting
  toward the old 2-piece and 4-piece bonuses only; they never count toward 5/6-piece bonuses. New Set
  pieces drop separately. No loss, no free upgrade.
- `Rarity` gains a variant: serialization must keep old rarity strings/indices valid.

## Delivery phases

1. UI pass: sections 1, 3, 8.
2. Stage rounds and progress bar: section 4 (Leader and Uncommon headline rounds use existing
   enemies as stand-ins until phase 6 lands).
3. Skill rarity and replacement: sections 2, 6 (skills, passives, items).
4. Elements core: section 9 (table, weapons, enemies, heroes, worlds, on-hit effects).
5. Gear: Set rarity, set tiers, drop rates, save migration (sections 6 gear, 7).
6. Enemies: rarity tiers, Leaders, 18 new enemies with art and lore (section 5).

Each phase ships playable on its own, and updates the `README.md` sections it changes (Controls,
the night structure under Campaign, the Gear list, enemy counts, `web/` in Layout, browser/PWA line).

Set gear never comes from the Shrine Draw: Draw odds and pity stay as published (my call).

## Left to the builder

- Exact multipliers, XP/drop numbers, timers and on-hit chances (bot-tuned, within the stated targets).
- Final weapon element table (to the rules above, shown before balancing).
- Final names/lore for the 18 enemies within the archetypes and elements given.
- The six 6-piece effects, one per set, themed on the set's element.
- Code structure.

## Open questions

None.

## Carried into phase 3 (from phase 1, 2026-11-08)

- Toya is overtuned under the reach rebalance (shrine_valley Normal: 5→10 of 10 wins vs pre-reach) and
  Shun swings by seed block (20→14 of 20 on seeds 41–60). One melee multiplier (`MIGHT_MELEE` 2.3)
  cannot fit both: add a per-hero lever.
- `BOSS_HP` 0.5 is a `shortcut:` tuned against the bot's ~140 px kiting; revisit with human playtests.
- Kaede, Yuzu, Toya, Haru and every stage other than Shrine Valley / Shotengai are unmeasured at Hard+.
- Untested selector filters: RaijinDrums, YataMirror, EmberLeaf/Rainward set hazards (Thunderclap is tested).
- Minor tidy-ups: fallback target points pile on the reach-box edge (`random_targets`); `hero_def()`
  rebuilt per `might()` call; T1 doc comments that restate names; `mouse_moved()` placement in ui.rs;
  dev-only debug overlay sits 12 px from the edge.
- Replace the reused `Sfx::Bell` with a dedicated rising chime for the ultimate alert.
- Riku fell from 9 to 6 wins of 10 on Shrine Valley Normal after rounds (seeds 21–30): the round-16
  sub-boss and round-18 mid boss are only 2 rounds apart (300 s before), so he often fights both; needs a
  per-hero lever. Phase 2 final values: HEADLINE_BUMP 1.01 (compounding), world rings [3, 7, 11, 15, 19],
  world crisis rounds 15–17.
- On 25-minute worlds the first chest now comes at round 4 (~3:57, was 2:00) and the first elite at
  round 8 (~9:12).

## Carried into phase 4

- Phase 3 skill rarity final values (fix round 2): tier weights stay at the spec values 1.0 / 0.8 / 0.6 / 0.4 /
  0.25 / 0.15; `REPLACE_CHANCE` 0.15; inherit rule unchanged (`max(1, old / 2)`).
- Weapon tiers: Unique = SpiritSeal + VenomFang (never a signature; each exists in one hero's pool, Ren's and
  Toya's), Mythic = GravityWell + LanternVigil, Legendary = GraveCleaver + FoxfireOrb + VoidLance; weapon buckets
  are [7, 7, 5, 3, 2, 2]. The Unique target is per Unique-capable hero (Ren, Toya), not a five-hero average.
- Bot policy: `bot_pick` replaces only the lowest-level skill of a strictly lower tier than the newcomer, else it
  takes the first plain card.
- Measured, seeds 21-30 / 41-50: Unique offered Ren 4 / 4 of 10, Toya 8 / 5 of 10; replacements 1.7 / 1.5 per
  run; weapon slots full by round 2; median 4 maxed; wins within 1 of baseline per hero except Shun +2 (tuning)
  and Kuon +2 (confirm).
- Phase 3 Task 5 per-hero damage levers (`reach_might`): `MIGHT_MELEE` 2.3 (Shun, Haru), `MIGHT_TOYA` 2.3 (unchanged),
  `MIGHT_RIKU` 2.0 (was 1.0). Riku shrine_valley Normal wins: 21-30 6 / 8 / 8 / 8 / 9 / 9 at 1.0 / 1.2 / 1.4 / 1.6 /
  2.0 / 2.5; fresh seeds 61-70 at 1.4 / 1.6 / 2.0 = 5 / 7 / 8, so 2.0 kept (only value reaching >= 8).
- Phase 4 recheck: Kuon scored 10/10 on seeds 41-50 (likely noise); Haru 1/10 and Kaede 4/10 at Normal are
  unfinished heroes, melee lever unverified. `BOSS_HP` stays a `shortcut:` until human playtests.
- Phase 3 Task 6 debts paid: reach tests for Raijin Drums, Yata Mirror and the Ember Leaf / Rainward sets (all already
  filtered, kept as regressions); `random_targets` fallback now samples inside the reach box instead of piling on its
  edge; ultimate alert plays a dedicated `Sfx::Rise`; `mouse_moved` moved up with the helpers; debug overlay at
  `kit::MARGIN`. Still open: no T1 doc comments restated a name (none deleted); `BOSS_HP` shortcut and the Haru / Kaede
  melee lever above.

## Carried into phase 5

- Phase 4 final values: `PROC_CHANCE` 0.15, `BURN_SECONDS` 3.0, `BURN_SHARE` 0.3, `BURN_TICK` 0.5, `SOAK_SECONDS` 4.0,
  `SOAK_ELECTRIC` 1.3, `CHAIN_RANGE` 90.0, `CHAIN_SHARE` 0.5, `STAGGER_SECONDS` 0.4, `KNOCK_EXTRA` 220.0,
  `SLOW_SECONDS` 2.0, `SLOW_FACTOR` 0.65, `DRAIN_HEAL` 0.5, `ELEM_STRONG` 1.5, `ELEM_WEAK` 0.75,
  `ELEM_INTO_RARE` 0.9, `ELEM_FROM_RARE` 1.15, `ELEM_RARE_CLASH` 1.5, `AFFINITY_DAMAGE` 1.1. No tuning was needed.
- Weapon, enemy and world element tables as built: see "Element tables" in the phase 4 plan.
- Measured (Normal, 10 seeds, vs `73ca295`): shrine_valley every hero within 2 wins in both blocks; no stage lost
  more than 3 for Ren or Riku; Light and Darkness heroes never beat the median delta by more than 2. Details in the
  plan's "Measured".
- Deferred to phase 5: gear elements and elemental resistance (cap 60% total); Arsenal takes its weapon's element;
  Set takes its set's element; items get an element on load, deterministically from the item id. Eclipse unique
  damage currently inherits the triggering hit's element advantage: revisit with gear elements.
- Rulings made in phase 4: PurificationWave is Water (Light is Legendary+ only); GravityWell is Darkness (Kuon's
  signature is off-affinity, as are Ren's Ofuda and Mika's StarFall); bosses ignore stagger; on-hit effects take the
  raw pre-multiplier hit (burn ticks are Fire hits, chain hits are Electric hits); a refreshed burn takes the new
  hit's dps, not the max; ultimates also get the x1.1 affinity.
- Open rechecks: Kuon 10/10 on seeds 41-50 again; Haru, Kaede, Mika, Nanami and Yuzu unmeasured; `BOSS_HP` stays a
  `shortcut:` until human playtests. Spec inconsistency for phase 6: Shrine Valley's new enemies are drafted Wind
  while the world is Earth, and Ember Leaf's are Earth/Fire/Water while the world is Wind. Reconcile before phase 6.
- Client follow-ups: no visual for burn, soak, slow, stagger or mark on enemies; replace picker not screenshotted.
- Burn ticks and chain hits go through `apply_hit`, so they roll crits, can spawn Eclipse, advance ThousandCounter and
  consume Light marks; revisit with gear elements.
- Playtest watch items next to `BOSS_HP`: Darkness drain sustain scales with hits per second (GravityWell, Hannya
  aura), lever is a drain cooldown; Ice slow applies at full strength to bosses, lever is a weaker boss slow.
- Client duplications: the element name is measured twice in the replace picker; the Affinity text format is
  duplicated between the armory and the menu.

## Carried into phase 6

- Phase 5 final values: `ELEM_ITEM` 0.02 (user, 2026-11-09: halved from 0.04; × item scale: ilvl × rarity mult × plus), `RESIST_CAP` 0.60,
  `Rarity::Set` index 7 (mult 1.75, dust 800, 6 affixes at `0.7 + 0.3 q`), `SET_CHANCE` 0.13,
  `SET6_STAGGER` 1.3, `SET6_SHATTER` 0.5 (capped at 200 × gear power), `SET6_SHATTER_R` 60, Lantern burn-pass
  range `CHAIN_RANGE` 90. Item element is save field 13; old items derive it from `hash(id, 7, 0xE1E)`.
- Element tables as built: Arsenal arts and Exotics in the phase 5 plan, "Element tables and rulings".
- The six 6-piece effects: Lantern Street burns pass on at death; Moonveil staggered enemies take +30%;
  Ember Leaf Wind procs at 2× chance; Pale Gate slowed enemies shatter; Rainward every Water hit soaks;
  Shatterline chains jump twice. An enemy killed by a shatter neither shatters nor passes its burn in the
  same pass (bounds cascades); the copy "shatter when they fall" slightly overpromises.
- Mid-phase user changes (2026-11-09):
  - Awakening fix: once every owned skill is maxed and slots are full, an offer carries one replace card at
    `MAXED_REPLACE_CHANCE` 0.25 (user's pick; `REPLACE_CHANCE` stays 0.15), preferring the missing complement
    passive of a maxed, un-awakened weapon, plus 2 distinct fallbacks; else Heal / Sen / Blessing. Measured
    maxed offers per run (Shrine Valley, seeds 21–30): Riku 3 / 13 / 24 (first ~18:30), Ren 7 / 24.5 / 45
    (first ~15:40) → ~3.3 / ~6.1 replace cards per run.
  - Off-screen hits: instrumented sim and real-client probes found no damage outside the screen except
    Exorcism Seal / ThousandCounter killing enemies whose centre was off screen (fixed: `in_view` is centre
    only). The rest was presentation. User choices: all circle visuals capped at `reach_half().x`, line
    visuals clipped to the reach box (`clip_to_reach`), damage numbers move with the player. Moonfall and
    Skeleton Hand now target in reach; off-view bursts are silent. The cap removes hits in the reach-box
    corners (192–220 from the player) for capped circles.
- Gear element damage stays uncapped (endless chase); the user halved `ELEM_ITEM` to 0.02 instead. A
  single-element endgame stack (11 pieces Set +10 ilvl 95, plus the 5-piece) now gives ~×3.4 that element
  (was ~×5.9). The bot plays without gear, so balance runs never see it.
- Still drawn past reach (disclosed, not capped): Regalia burst 300, special flourishes (Fox 230,
  Mukade 220, Domain 360 / 220, Hannya onset 160), travelling projectiles. The Divine and Hannya finales
  are capped visually; their pushes still reach 380 / 340 (push only, no damage).
- Save risk: an older build (e.g. a stale cached PWA) rejects rarity 7 and drops Set items on its next save.
- Phase 6 reconcile (from phase 4): Shrine Valley's drafted new enemies are Wind while the world is Earth;
  Ember Leaf's are Earth / Fire / Water while the world is Wind.
- Rechecks still open: Kuon 10/10 on seeds 41–50; Haru, Kaede, Mika, Nanami, Yuzu unmeasured; `BOSS_HP`
  `shortcut:` until human playtests; Darkness drain and Ice slow on bosses (phase 4 watch items).
- Balance vs `5f82b71` (Normal, seeds 21–30 / 41–50): see the phase 5 plan, "Measured".
