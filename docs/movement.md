# Movement skills

Every hero has a move on the movement key: **Space** or **Shift** go the way you're moving,
**right-click** goes towards the cursor, and on touch the **MOVE** button goes the way you're
dragging. The special is on **R** (the **SPECIAL** button on touch).

The sim (`crates/sim/src/world/movement.rs`, data in `content::move_def`):

- A move has charges that come back one at a time over its cooldown (Daichi has two). The hero
  can't be hit while it runs, and for a moment after.
- Travelling moves carry the hero along the path tick by tick, sliding along walls like walking
  does; teleports land on the farthest free point of the line (`free_to`), never in a wall.
- A move is never only a dash: each does something on the way or where it lands, and more in
  the hero's form. What it leaves on the ground (Ren's ofuda line, Yuzu's notes, the line
  Haru's cut lands on) is a `Trail`.
- The motion runs in `move_player`; what the move does to the field runs after the enemies'
  grid is built (`dash_actions`), so its hits always find the right enemies.

| Hero | Move | Does | In the form |
|---|---|---|---|
| Ren | Purifying Step | a glide; a line of ofuda cuts what crosses it and stops fireballs | Divine: lasts longer, cuts harder |
| Kaede | Lightning Flash | a teleport; the bolt chains through the path | |
| Mika | Starlight Leap | a hop landing in a burst of stars, pulls in drops | |
| Haru | Iaido Rush | a dash; the path is cut a beat later | |
| Nanami | Flask Jump | rides a flask blast, burning ground at the launch | |
| Yuzu | Encore Slide | a slide leaving notes: speed and +3 armour while on them | |
| Riku | Fox Blink | a blink leaving a fox decoy that draws enemies, then bursts | Nine-Tails: the tails lash the path |
| Shun | Veil Crossing | behind the strongest enemy near; it's marked (a sure critical) | Hannya: an afterimage explodes behind |
| Toya | Tendril Pull | dragged ahead; the way is cut and slowed | Mukade: three times the damage, wider |
| Kuon | Star Shift | swaps places with his star marker (or blinks and plants one); slows the old spot | Firmament: a quarter of the cooldown |
| Daichi | Severed Chain | the chain catches the first big enemy or wall on the line and reels him in to a ground smash | Daidara: the giant's fist follows him down |

Shrine Shop: **Footwork** (-8% cooldown per rank, 3 ranks) and **Second Step** (+1 charge).

Look (`crates/client/src/moves.rs`): the trails on the ground, a streak or blink per hero,
dust and a ring where a move lands, a leap lifting the hero on an arc (the shadow stays), and
Daichi's chain as iron links. Dev: `KZ_DASH=1` (with `KZ_PERF`) fires the move every 14 frames.

Balance (`BALANCE_DASH=1` makes the balance bot use it): on Shrine Valley, Normal, the bot
survives about a third longer with its move.
