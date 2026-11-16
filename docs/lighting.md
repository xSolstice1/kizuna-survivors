# Light and shadow

Every world is a moonlit night. Code: `crates/client/src/light/` (`mod.rs` the engine,
`sources.rs` what glows), the terrain's share in `terrain_bake.rs`.

## How a frame is lit

1. `draw_world` starts the frame's lights (`Lights::begin`: the stage's night ambient; last
   frame's steady lights become this frame's shadow casters, the "lamps") and adds the hero's
   own fill light (`add_fill`: no shadow, no bloom).
2. The ground, grass, props and enemies are drawn. Each casts **shadows**, all by one rule
   (`casts_into`): its own texture tinted black and laid on the ground from its feet, once
   along `light::SKEW` (away from the moon, upper left; fainter under a strong lamp, which
   outshines it) and once away from each steady lamp near it, longer the further it stands
   from the lamp and fading smoothly at the lamp's reach (no popping). A lamp doesn't shadow
   the prop it sits on. Shadows are darkest at the feet and fade towards the tip. Props and
   the terrain add their lights as they draw. Mist drifts over the ground (`ambient::draw_mist`).
3. `sources::gather` adds everything else that glows near the view, then `Lights::apply`
   renders the **light map** (quarter resolution: ambient plus soft additive glows) and
   multiplies it onto the world drawn so far, doubled and capped at 1.4x, so lit pools glow.
   Bright lights then **bloom**: a tight and a wide additive halo over whatever is in front.
4. The hero, attacks and effects are drawn after it at full colour, so they always read. The
   hero is tinted by `light_at` (the same light, on the CPU, eased) and casts the same shadows
   as everything else (`Lights::casts_at`) from the live, posed rig.
5. The terrain shader shades for the same moon: higher ground casts a soft shadow onto lower
   ground away from it, and edges facing it catch light (`MOON` in `terrain_bake.rs`).

## Adding something

- **Something that glows** (enemy, projectile, pickup, prop): add its `Glow` in
  `light/sources.rs` (reach, colour, height it shines from). Projectiles and pickups are matched
  exhaustively, so a new kind won't build until it says how it glows; enemies list only the
  ones that do. A one-off light (an effect, a boss attack): call `sources::emit_passing` (or
  `Lights::add_own`) between `begin` and `apply`. Only steady lights (`emit` / `add`: the
  shrine, fire pools, lanterns, props) cast shadows; anything moving or brief must not, or
  shadows swing about.
- **Something that should cast a shadow**: drawn through the enemy or prop paths it already
  does. Elsewhere, push it to a `light::Shadows` batch (`push(tex, feet, w, h, flip, alpha)`)
  and `flush()` it before drawing the sprite: it gets the moon's and the lamps' shadows like
  everything else.
- **A new rigged hero** gets the rig shadow and lighting with no code.
- **A new world**: give it a night ambient in `light::ambient` (light-map units, 0.5 = as
  painted; about 0.31-0.37 reads as night without hiding enemies) and its mist in
  `ambient::mist` (colour, density, drift).
- **Something that must stay vivid** (UI-like markers, telegraphs): draw it after `apply`.

## Checking

Native screenshots, no browser needed (`KZ_SHOT` saves the frame and exits):

    KZ_PERF=shrine_valley KZ_PERF_CALM=1 KZ_SHOT=/tmp/a.png target/release/client   # early night
    KZ_PERF=shrine_valley KZ_PERF_REAL=1 KZ_SHOT=/tmp/b.png target/release/client   # late, busy
    KZ_NOLIGHT=1 ...                                                                # lighting off

Judge it at game size and in motion; a crowd of lights must never wash the scene out.
