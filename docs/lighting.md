# Light and shadow

Every world is a moonlit night. Code: `crates/client/src/light/` (`mod.rs` the engine,
`sources.rs` what glows), the terrain's share in `terrain_bake.rs`.

## How a frame is lit

1. `draw_world` starts the frame's lights (`Lights::begin`: the stage's night ambient) and
   adds the hero's own light (it casts no shadow).
2. The ground, props and enemies are drawn. Each casts a **shadow**: its own texture tinted
   black, slanted along `light::SKEW` (away from the moon, upper left) onto the ground from its
   feet, fainter the higher it floats. Props and the terrain add their lights as they draw.
3. `sources::gather` adds everything else that glows near the view, then `Lights::apply`
   renders the **light map** (quarter resolution: ambient plus soft additive glows) and
   multiplies it onto the world drawn so far, doubled and capped at 1.4x, so lit pools glow.
4. The hero, attacks and effects are drawn after it at full colour, so they always read. The
   hero is tinted by `light_at` (the same light, on the CPU) and casts a shadow from the live,
   posed rig, away from the strongest light near them (`skew_at`), falling back to the moon.
5. The terrain shader shades for the same moon: higher ground casts a soft shadow onto lower
   ground away from it, and edges facing it catch light (`MOON` in `terrain_bake.rs`).

## Adding something

- **Something that glows** (enemy, projectile, pickup, prop): add its `Glow` in
  `light/sources.rs` (reach, colour, height it shines from). Projectiles and pickups are matched
  exhaustively, so a new kind won't build until it says how it glows; enemies list only the
  ones that do. A one-off light (an effect, a boss attack): call `sources::emit` (or
  `Lights::add`) between `begin` and `apply`, i.e. before the enemies' pass ends.
- **Something that should cast a shadow**: drawn through the enemy or prop paths it already
  does. Elsewhere, push it to a `light::Shadows` batch (`push(tex, feet, w, h, flip, alpha)`)
  and `flush()` it before drawing the sprite; `push_skew` with `Lights::skew_at` makes it fall
  away from nearby lights like the hero's.
- **A new rigged hero** gets the rig shadow and lighting with no code.
- **A new world**: give it a night ambient in `light::ambient` (light-map units, 0.5 = as
  painted; about 0.35-0.40 reads as night without hiding enemies).
- **Something that must stay vivid** (UI-like markers, telegraphs): draw it after `apply`.

## Checking

Native screenshots, no browser needed (`KZ_SHOT` saves the frame and exits):

    KZ_PERF=shrine_valley KZ_PERF_CALM=1 KZ_SHOT=/tmp/a.png target/release/client   # early night
    KZ_PERF=shrine_valley KZ_PERF_REAL=1 KZ_SHOT=/tmp/b.png target/release/client   # late, busy
    KZ_NOLIGHT=1 ...                                                                # lighting off

Judge it at game size and in motion; a crowd of lights must never wash the scene out.
