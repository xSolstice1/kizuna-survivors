# Graphics

Everything here is code and shaders: no generated art. Light and shadow have their own page
(`docs/lighting.md`).

## Maps read the same in every world

One rule: **rock is wall, ground is open, stairs lead between levels.**

- The terrain baker (`terrain_bake.rs`) writes the sim's own wall field into the right half of
  each chunk's mask (`sim::terrain::WallGrid`: the exact field the player collides with,
  `Terrain::wall_field`). The shader draws rock wherever it's positive, so what looks like a
  wall is one and nothing else is. Test: `drawn_walls_are_the_real_walls`.
- A drop facing south shows its rock face. A drop facing any other way is walled on its high
  side: a rocky rim, lit at the drop, outlined dark where open ground begins.
- Roads crossing a level change are stone stairs with curbs. Low ground is darker and cooler,
  high ground brighter. Every wall has contact shade at its foot.
- To check a spot: `KZ_WALLS=1` dots the real walls red over the picture (native), and
  `KZ_PERF_AT=x,y` stands the hero there (`cargo run -p sim --example find_ramps` prints a ramp
  and a wall near each world's start).

## The lens (`post.rs`)

The world is drawn into a target, then laid on the screen through one shader:

- bloom at a quarter and an eighth of the resolution, bounded so a screen full of spells
  glows instead of whiting out;
- a filmic shoulder for the brights;
- a colour grade per world (`look`): shadows' tint, highlights' gain, saturation, contrast,
  bloom threshold;
- tilt-shift, heat haze (`post::heat`), chromatic aberration that punches on heavy hits
  (`post::punch`), film grain and the vignette.

## Characters (`litsprite.rs`)

Sprites get volume from their own outline. One step towards the light outside the sprite
means a rim of light; one step away outside means a core shadow. The crowd is lit by the
moon, and the hero by the moon plus the strongest lamp near them (`Lights::rim_at`, which
also takes the lamp's colour).

## What fights leave (`worldfx.rs`)

Fallen enemies burn away along a noise pattern with a glowing edge. Blasts leave scorch
marks that glow hot for a moment, then fade.

## Weather and wind

`wind.rs` is one wind for grass, branches and motes. Rainward's lamps reflect as streaks on
the wet street (the light map), and rain ripples the ground (`ambient.rs`).

## Quality

The graphics setting is Auto, Low, Medium or High (`Profile::graphics`, Settings).

| Level | What runs |
| --- | --- |
| Low | Grade and vignette only. Half the grass and motes, no mist, flat sprites. |
| Medium | Adds bloom, grain and lit sprites. |
| High | Adds the wide bloom, tilt-shift, heat haze and aberration. |

Auto starts High and steps down after three seconds under ~45 fps; it never steps back up in a
session. Dev: `KZ_GFX=1..3` forces a level (native).
