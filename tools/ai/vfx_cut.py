"""Cut a generated 4x4 effects sheet into sprites for the client's effects kit.

    python3 tools/ai/vfx_cut.py <sheet.png> --bg black|green name1 name2 ... (16 names, "-" skips a cell)

Writes art/hd/vfx/<name>.png (128x128 RGBA), then add the names to `NAMES` in
crates/client/src/vfx.rs.
  --bg black   glowing effects drawn additively (art/prompts/vfx_sheet*.txt on black): alpha
               from the brightest channel, colour un-premultiplied, so black is invisible.
  --bg green   solid objects drawn normally (on a #00FF00 chroma-key screen): alpha from how far
               each pixel is from the key green, green spill removed from the edges.
Each cell's border is faded so neighbours (and any grid lines) never show.
"""
import argparse, pathlib

import numpy as np
from PIL import Image

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "art" / "hd" / "vfx"
CELL = 256


def fade_border(alpha, px):
    yy, xx = np.mgrid[0:CELL, 0:CELL]
    edge = np.minimum(np.minimum(xx, CELL - 1 - xx), np.minimum(yy, CELL - 1 - yy))
    return alpha * np.clip((edge - px) / 8, 0, 1)


def key_black(a):
    alpha = np.clip((a.max(axis=2) - 6) / (255 - 6), 0, 1)
    alpha = fade_border(alpha, 2)
    rgb = np.where(alpha[..., None] > 0.004, a / np.maximum(alpha[..., None] * 255, 1) * 255, 0)
    return rgb, alpha


def key_green(a):
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    # How much greener than the other channels: the key is (0, 255, 0).
    excess = g - np.maximum(r, b)
    alpha = 1 - np.clip((excess - 40) / 120, 0, 1)
    # Grid lines and anything dark touching the border go too.
    alpha = fade_border(alpha, 6)
    rgb = a.copy()
    # Despill: the green channel can't exceed the brighter of red and blue on edge pixels.
    spill = alpha < 0.98
    rgb[..., 1] = np.where(spill, np.minimum(g, np.maximum(r, b)), g)
    return rgb, alpha


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("sheet")
    ap.add_argument("--bg", choices=("black", "green"), required=True)
    ap.add_argument("names", nargs=16)
    a = ap.parse_args()
    src = Image.open(a.sheet).convert("RGB").resize((CELL * 4, CELL * 4), Image.LANCZOS)
    OUT.mkdir(parents=True, exist_ok=True)
    for k, n in enumerate(a.names):
        if n == "-":
            continue
        cell = np.asarray(src.crop(((k % 4) * CELL, (k // 4) * CELL, (k % 4 + 1) * CELL, (k // 4 + 1) * CELL))).astype(np.float32)
        rgb, alpha = (key_black if a.bg == "black" else key_green)(cell)
        im = Image.fromarray(np.dstack([rgb, alpha * 255]).clip(0, 255).astype(np.uint8), "RGBA")
        im.resize((128, 128), Image.LANCZOS).save(OUT / f"{n}.png", optimize=True)
        print("wrote", (OUT / f"{n}.png").relative_to(ROOT))


if __name__ == "__main__":
    main()
