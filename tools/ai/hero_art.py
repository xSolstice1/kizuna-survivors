"""Make a rigged hero's menu and HUD art from its design image.

    python3 tools/ai/hero_art.py <hero> <design.png>

Writes art/hd/<hero>_front.png (288x384, the design keyed, fitted, feet at the bottom: menus,
armory, hero select) and art/hd/<hero>_face.png (256x256 crop around the head, using the neck
point in art/rig/<hero>/cut.json: the HUD portrait).
"""
import json, pathlib, sys

import numpy as np
from PIL import Image

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from process import bleed, key_green  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[2]


def main():
    hero, design = sys.argv[1], sys.argv[2]
    rgb, a = key_green(Image.open(design))
    im = Image.fromarray(np.dstack([rgb, a * 255]).clip(0, 255).astype(np.uint8), "RGBA")
    fig = im.crop(im.getchannel("A").point(lambda v: 255 if v > 8 else 0).getbbox())
    W, H = 288, 384
    k = min((H - 8) / fig.height, (W - 4) / fig.width)
    fig = fig.resize((round(fig.width * k), round(fig.height * k)), Image.LANCZOS)
    front = Image.new("RGBA", (W, H))
    front.alpha_composite(fig, ((W - fig.width) // 2, H - fig.height))
    bleed(front).save(ROOT / "art" / "hd" / f"{hero}_front.png", optimize=True)
    nx, ny = json.loads((ROOT / "art" / "rig" / hero / "cut.json").read_text())["neck"]
    face = im.crop((nx - 250, ny - 440, nx + 250, ny + 60)).resize((256, 256), Image.LANCZOS)
    bleed(face).save(ROOT / "art" / "hd" / f"{hero}_face.png", optimize=True)
    print(f"wrote art/hd/{hero}_front.png and art/hd/{hero}_face.png")


if __name__ == "__main__":
    main()
