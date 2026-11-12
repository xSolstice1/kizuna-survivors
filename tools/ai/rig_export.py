"""Export a cut-out rig (rig_cut.py output) for the game.

    python3 -I tools/ai/rig_export.py art/rig/ren --height 31

Writes art/hd/rig/<hero>/<part>.png (downscaled to TEX px per source px, enough for retina
screens at the hero's in-game size) and art/rig/<hero>/game.json, which the client embeds:
part rects and pivots stay in source pixels; `feet` is the source point placed on the player's
feet, and `units_per_px` turns source pixels into world units.
"""
import argparse, json, pathlib
from PIL import Image

from process import bleed

ROOT = pathlib.Path(__file__).resolve().parents[2]
FEET_Y = 990          # source y of the soles (matches rig_preview.py)
TEXELS_PER_UNIT = 8   # 2x the game's 4, so the rig stays crisp when zoomed on retina


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--height", type=float, default=31.0, help="figure height in world units")
    a = ap.parse_args()
    d = pathlib.Path(a.dir).resolve()
    hero = d.name
    rig = json.loads((d / "rig.json").read_text())
    tops = [p["offset"][1] for p in rig["parts"].values()]
    fig_h = FEET_Y - min(tops)
    units_per_px = a.height / fig_h
    k = TEXELS_PER_UNIT * units_per_px
    out = ROOT / "art" / "hd" / "rig" / hero
    out.mkdir(parents=True, exist_ok=True)
    for n in rig["order"]:
        im = Image.open(d / f"{n}.png").convert("RGBA")
        im = im.resize((max(1, round(im.width * k)), max(1, round(im.height * k))), Image.LANCZOS)
        bleed(im).save(out / f"{n}.png", optimize=True)
    feet_x = rig["parts"]["torso"]["pivot"][0]
    game = {
        "units_per_px": units_per_px,
        "feet": [feet_x, FEET_Y],
        "parts": {n: {"rect": [*p["offset"], *p["size"]], "pivot": p["pivot"]} for n, p in rig["parts"].items()},
    }
    (d / "game.json").write_text(json.dumps(game, indent=1))
    print(f"{hero}: {len(rig['order'])} parts at {k:.3f} texels/src px, {units_per_px:.4f} units/px")


if __name__ == "__main__":
    main()
