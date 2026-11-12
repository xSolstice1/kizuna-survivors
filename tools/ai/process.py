"""Turn a generated character sheet (figures on flat green) into game sprites.

    python3 -I tools/ai/process.py <sheet.png> <out_name> --height 46 [--names idle walk]

Keys out the green, removes green spill on the edges, splits the sheet into figures
(left to right; small blobs such as a floating orb are dropped), trims each one, scales it
so the figure is `--height` world units tall (stored at sprites::UP = 4 texels per unit),
anchors the feet at the bottom edge, and writes art/hd/<out_name>_<name>.png.
"""
import argparse, pathlib
import numpy as np
from PIL import Image
from scipy import ndimage

ROOT = pathlib.Path(__file__).resolve().parents[2]
UP = 4


def key_green(img):
    a = np.asarray(img.convert("RGB")).astype(np.float32)
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    spill = g - np.maximum(r, b)                  # how much greener than the other channels
    alpha = np.clip((90.0 - spill) / 60.0, 0, 1)  # >90 = background, <30 = solid
    # Despill: where green leads, pull it down to the red/blue average (video compression bleeds
    # green into dark fabric; a plain clamp to max(r, b) still leaves navy looking teal).
    g2 = np.where(spill > 0, np.minimum(g, (r + b) / 2), g)
    rgb = np.stack([r, g2, b], -1)
    return rgb, alpha


def bleed(im):
    """Gives every transparent pixel the colour of the nearest opaque one, so GPU filtering
    (and mipmaps) don't pull a dark fringe in from (0,0,0,0)."""
    a = np.asarray(im).copy()
    solid = a[..., 3] > 8
    if solid.any() and not solid.all():
        _, (iy, ix) = ndimage.distance_transform_edt(~solid, return_indices=True)
        a[..., :3] = a[iy, ix, :3]
    return Image.fromarray(a, "RGBA")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("sheet"); ap.add_argument("name")
    ap.add_argument("--height", type=float, default=46.0)
    ap.add_argument("--names", nargs="*", default=["idle", "walk"])
    a = ap.parse_args()
    rgb, alpha = key_green(Image.open(a.sheet))
    # Figures: connected blobs of solid pixels; drop anything small (orbs, glints).
    lab, n = ndimage.label(ndimage.binary_closing(alpha > 0.5, iterations=3))
    sizes = ndimage.sum(np.ones_like(alpha), lab, range(1, n + 1))
    big = [i + 1 for i, s in enumerate(sizes) if s > sizes.max() * 0.25]
    boxes = sorted((ndimage.find_objects(lab)[i - 1], i) for i in big)
    boxes.sort(key=lambda t: t[0][1].start)
    for (sl, idx), name in zip(boxes, a.names):
        mask = ndimage.binary_dilation(lab == idx, iterations=4)   # keep soft edge pixels
        al = np.where(mask, alpha, 0)[sl]
        rgba = np.dstack([rgb[sl], al * 255]).clip(0, 255).astype(np.uint8)
        im = Image.fromarray(rgba, "RGBA")
        im = im.crop(im.getbbox())
        th = round(a.height * UP)
        tw = round(im.width * th / im.height)
        tw += tw % 2
        im = im.resize((tw, th), Image.LANCZOS)
        out = ROOT / "art" / "hd" / f"{a.name}_{name}.png"
        bleed(im).save(out)
        print("wrote", out.relative_to(ROOT), im.size, f"= {tw / UP:.1f} x {th / UP:.1f} units")


if __name__ == "__main__":
    main()
