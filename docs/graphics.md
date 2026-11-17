# Graphics

Everything here is code and shaders: no generated art. Light and shadow have their own page
(`docs/lighting.md`).

## Maps read the same in every world

One rule: **rock is wall, ground is open, stairs lead between levels.**

- The terrain baker (`terrain_bake.rs`) writes the sim's own wall field into the right half of
  each chunk's mask (`sim::terrain::WallGrid`: the exact field the player collides with,
  `Terrain::wall_field`). The shader draws rock wherever it's positive, so what looks like a
  wall is one and nothing else is. Test: `drawn_walls_are_the_real_walls`.
- Platforms are raised ground: their surface runs to the edge, and the edge shows the
  height. A drop facing south shows its rock face (the wall). Any other drop has a lit lip
  and casts the platform's shadow on the lower ground; the sim's wall there is a thin lip
  (`RIM`, 6 units), so the top is walkable right up to where it falls away.
- Stairs are built objects: `sim::terrain::stairs` finds every point where a road's centre
  line crosses a level change (tested to lie on climbable road), and `Baker::draw_stairs`
  draws a straight flight there. It has pale stone treads lit at the nose, darker each step
  down, and risers that read as faces when the flight comes towards the viewer and as
  shadow lines otherwise. Side walls have lit tops, and a shadow falls at the foot. Where a
  road only runs along an edge, it's a sloped ramp, shaded down the slope.
- On High the ground has relief: its texture is read as height and lit by the moon.
- Low ground is darker and cooler, high ground brighter.
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
- shafts of moonlight slanting through the air (High);
- tilt-shift, heat haze (`post::heat`), chromatic aberration that punches on heavy hits
  (`post::punch`), film grain and the vignette.

## Characters (`litsprite.rs`, `herolight.rs`)

Sprites get volume from their own outline. Near the edge facing the light (but inside the
art's outline) a rim of light; along the far side a core shadow. The crowd is lit by the
moon (`litsprite.rs`).

The hero is a rig of many parts, so it is first drawn whole into an off-screen image at
screen resolution, then that image is lit (`herolight.rs`). That way the rim and shadow
follow only the outer silhouette, never the seams between parts. It has a lit side and a shadow side across the whole figure, is lit from above,
darker towards the feet, and takes the moon or the nearest lamp's softened colour
(`Lights::rim_at`). It also gets the world's colour grade (`post::grade`), so it sits in the
scene while staying sharp.

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
| High | Adds the wide bloom, tilt-shift, heat haze, aberration, light shafts and ground relief. |

Auto starts High and steps down after three seconds under ~45 fps; it never steps back up in a
session. Dev: `KZ_GFX=1..3` forces a level (native).
