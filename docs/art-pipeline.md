# Art pipeline: how to make and add art

Everything in the game's art set (heroes, enemies, bosses, props, ground, icons) was made
with the tools in `tools/ai/`. This document is enough to add a new hero, enemy, icon or
stage, or to redo one, in the same style.

## Setup

- Python 3 with `numpy`, `pillow`, `scipy`, `opencv-python` (and `playwright` for headless
  screenshots, optional). `ffmpeg` for MP4 previews, optional.
- An OpenRouter API key in `.env` at the repo root (git-ignored):
  `OPENROUTER_API_KEY=sk-or-...`. Give the key a spending limit in OpenRouter.
- Every call is logged to `art/ai/spend.csv`. Raw generations go to `art/ai/` (git-ignored).
- Run the scripts from `tools/ai/` (`cd tools/ai && python3 rig_cut.py ...`), or from the repo
  root with `python3 tools/ai/<script>.py`. `gen.py` can be run with `python3 -I`.

## The style

- **Chibi, about 2 heads tall**, big head, short body. A hero is **31-32 world units** tall in
  game (64 px on a desktop, 32 px on a phone).
- **Dark navy outline** with varied weight, **glossy cel shading** (smooth gradients plus bright
  specular highlights, "polished vinyl figure"), large detailed anime eyes, hair as clean locks.
- The reference image is chibi Ren: `art/ai/raw/ren_chibi_gemini-3-pro-image_0.png` (local
  only). Keep a copy of it somewhere safe: every prompt uses it as the style reference.
  `art/hd/ren_front.png` (committed) is a lower-resolution copy that also works.
- **Judge art at game size, never full size**: `tools/ai/preview.py` (below).
- Originality: follow the IP rule in `PLAN.md`. No existing characters, no copied outfits.

## Models and cost (OpenRouter, Nov 2026)

| Model | Use | Cost per call |
|---|---|---|
| `google/gemini-3-pro-image` | Final art: hero designs, sprites, bosses, icons | ~$0.14 |
| `google/gemini-nano-banana-2.1` | Layer edits for rigs, ground tiles, drafts | ~$0.04 |

Small sprites go **four per call** (a 2x2 sheet), so a sprite costs ~$0.035 on pro.
A full hero (design + layers) is about $0.30-0.70. The whole game's art cost about $15.

## How the image model is used

One fact makes the whole pipeline work: **when you give the model an image and ask for an edit
("the same image, with X removed" or "the same sprite, redrawn"), it keeps the layout aligned
with the input, usually to within a pixel.** So:

- To **restyle** a sprite, put the old one on a flat key colour and ask for the same subject,
  pose and size, redrawn. The result drops into the old canvas: anchors and hitboxes stay put.
- To **cut a hero into parts**, ask for "the same image with the hair removed", "with the arms
  removed", and so on, then subtract: each part is where two edits differ.

Key colours: pure green `#00FF00`, or pure magenta `#FF00FF` for green subjects (so the key
doesn't eat them). `restyle.py` picks whichever clashes less with the sprite.

## Checking art: `preview.py`

    python3 tools/ai/preview.py <sprite.png ...> --name <out> [--height 32] [--stage w1]

Writes `art/ai/preview/<out>.png`: the candidates in a lineup with the current heroes and the
stage's enemies, and in a busy horde scene on a desktop, a retina screen and a phone. Green-screen
images are keyed automatically. If it doesn't read at phone size, it isn't done.

## Adding or redoing a sprite (enemy, prop, icon, misc)

The game loads `art/hd/<path>.png`. A sprite that also has a pixel source in `art/src/` must keep
**exactly 4x the pixel art's size** (`sprites::UP`), so keep the canvas size of the file you
replace.

**Redo existing art** with `restyle.py` (it reads the originals from git `HEAD`, so re-runs never
restyle a restyle):

    python3 tools/ai/restyle.py plan                      # batches and cost
    python3 tools/ai/restyle.py run --model pro --only w2  # one world (ids contain the folder)
    python3 tools/ai/restyle.py run --files w1/fodder_a.png w5/mini.png   # specific files, 1 per call
    python3 tools/ai/restyle.py harmonize                 # after redoing ground tiles

What it does: builds 2x2 input sheets on the key colour (one per call for bosses and tiles), calls
the model with chibi Ren as the style reference, keys the result, fits each sprite to the old
sprite's bounding box (same height, feet in the same place), and flips it back if the model
mirrored it. Raw outputs are cached in `art/ai/restyle/`; delete one to regenerate it.

**A brand-new sprite**: draw or generate a rough placeholder at the right canvas size (even a
quick sketch, or a previous sprite to restyle from), put it in `art/hd/`, commit it, then run
`restyle.py run --files <it>`. Or generate it directly with `gen.py` (below) and fit it into its
canvas yourself.

Known failure modes and fixes:
- **The style girl leaks in** (a pink-haired girl appears in a slot): the prompts already say
  "image 2 is only a style reference". Redo just that file with `--files`.
- **A creature turns into an anime girl** (the "faces like hers" wording pulls that way): add an
  identity note to `HINTS` in `restyle.py` ("It is a kodama ... not human"), then `--files`.
- **Pale green gets washed out**: the key-colour cleanup only touches edges now; if a sprite is
  strongly green, make sure it went on magenta (`pick_bg`).
- **Background not flat** (a gradient): flood-fill light/green pixels connected to the border to
  pure green before keying.

## Enemy animation (code, no art)

Enemies are single sprites animated in `crates/client/src/render.rs`: `motion(kind)` picks
**Walk** (step bounce, squash on footfalls, lean into the movement), **Hop** (arcs with a hard
landing squash) or **Float** (bob and sway, for spirits, birds and lanterns), and `enemy_pose`
turns that into a lift, a squash and a rotation about the feet. Every enemy also gets a hit
squash and breathes when standing still. A new enemy kind defaults to Walk; add it to the
`Hop` or `Float` list in `motion()` if it should move differently.

## Adding a hero (rigged, like Ren)

Heroes are **cut-out rigs**: about 9 layered parts animated in code with springs (the hair and
skirt lag and swing, the head and arms follow through). No frame-by-frame art.

### 1. Design (pro, ~$0.14)

For a redesign of an existing placeholder, use `art/prompts/hero_restyle.txt` with the old
sprite (scaled up onto green) and chibi Ren as references:

    python3 -I tools/ai/gen.py art/prompts/hero_restyle.txt <hero>_hero --model google/gemini-3-pro-image \
        --ref <old_on_green.png> art/ai/raw/ren_chibi_gemini-3-pro-image_0.png --aspect 1:1

For a new character, write a prompt like `art/prompts/ren_chibi.txt` (describe the character,
the style, "three-quarter view facing right, arms relaxed slightly away from the body, legs
slightly apart, flat green background"). Run `preview.py` on the candidates.

The pose matters for cutting: arms a little away from the body, a gap between the legs.

### 2. Layer edits (nano, ~$0.04 each)

Prompts in `art/prompts/layers/`. Each edit takes the previous image as its only reference.

| Hero has | Edits |
|---|---|
| Long hair, skirt (like Ren) | `nohair` (from the design), `haironly` (from the design), `noarms` and `nolegs` (both from the `nohair` result) |
| Short hair, trousers (like Riku, Shun, Toya, Kuon) | `noarms_any` (from the design) only. The legs are cut at the hip line; short hair stays on the head. |

    python3 -I tools/ai/gen.py art/prompts/layers/nohair.txt <hero>_L_nohair --model google/gemini-nano-banana-2.1 --ref <design.png> --aspect 1:1

Check each edit: it must change only what it says. If the model changed the face, the hair or
the clothes, run it again.

### 3. Cut (`rig_cut.py`, free)

Write `art/rig/<hero>/cut.json`. Copy `art/rig/ren/cut.json` (long hair, skirt) or
`art/rig/riku/cut.json` (short hair, trousers) and change the numbers. All coordinates are in the
1024 px source image; overlay a grid on the design to read them off:

- `sources`: `full` (the design), `nohair` (the design itself if there's no hair edit),
  `noarms`, and `hair` / `nolegs` if you made those edits.
- `neck`: the head's pivot, just under the chin.
- `arm_split_x`: a vertical line between the two arms (near arm left of it, far arm right).
- `arm_hull`: a few points at each shoulder and upper arm. Navy sleeves on a navy body don't
  show in the difference, so these points complete each arm's outline.
- `hem_y` (skirts) or `hip_y` (trousers): where the legs start.
- `leg_split_x`: a vertical line between the legs.
- `pivots`: the joint each part turns around: shoulders, elbows, hips, the neck, and the top of
  the head for `hair_back`.

Then:

    cd tools/ai && python3 rig_cut.py ../../art/rig/<hero>/cut.json

It writes the parts (`hair_back`, `arm_far_upper/fore`, `leg_far`, `leg_near`, `torso`, `head`,
`arm_near_upper/fore`) and `rig.json`. A part with nothing in it (no long hair, an arm the edit
didn't remove) becomes an empty speck, so every rig has the same parts.

### 4. Preview the motion (free)

    python3 rig_preview.py ../../art/rig/<hero> [idle walk]

Writes `art/rig/<hero>/preview_*.gif` and `strip_*.png` (git-ignored), and
`art/ai/preview/<hero>_rig.gif` at game sizes. This uses the same motion as the game. If a part
tears or floats, fix the pivots or `arm_hull` in `cut.json` and cut again.

### 5. Export and register (free)

    python3 rig_export.py ../../art/rig/<hero> --height 31

writes `art/hd/rig/<hero>/*.png` and `art/rig/<hero>/game.json`. Then:

- `python3 tools/ai/hero_art.py <hero> <design.png>` makes `art/hd/<hero>_front.png` (menus,
  288x384) and `art/hd/<hero>_face.png` (HUD portrait, 256x256 around the head).
- In `crates/client/src/render.rs`, add the hero to `rig_name()` and `rig_json()`, and to the
  front-view match in `Assets::load` if it isn't covered.

### 6. Forms (pro, ~$0.14)

Transformations (`riku_fox`, `shun_hannya`, ...) are single sprites. `art/prompts/hero_form.txt`
with the old form sprite and the new hero design as references, then fit it into the form's
canvas (keep the size; `sprites::sprite` checks it).

## Ground tiles and stages

Ground tiles are 128x128 and opaque. `restyle.py` redraws each one (nano is fine), makes it
seamless (blends it with its half-offset copy), and `harmonize` matches each stage's `ground_b`,
`path` and decal colours to `ground_a` and feathers decals into it, so alternating tiles don't
checkerboard. Look at a tiled preview before committing; image models are bad at tiling, so
expect to touch these up.

## Tips

- Spend on pro for anything players look at; use nano for layer edits and tiles.
- One try per asset is usually enough with these prompts; redo only what is clearly broken.
- Commit the art in `art/hd/` and the rig sources in `art/rig/<hero>/` (cut.json, rig.json,
  game.json and the part PNGs). Raw generations stay local in `art/ai/`.
- Steam requires disclosing AI-generated content. Store art and portraits are worth a human pass.
