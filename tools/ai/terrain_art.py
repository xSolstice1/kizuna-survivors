"""Turn a world's generated terrain images into game textures.

    python3 tools/ai/terrain_art.py w1 --low <png> --high <png> --stairs <png> --cliff <png>

Writes art/hd/<world>/:
  low.png, high.png   128x128 ground tiles for low and high ground: lighting flattened (no
                      baked spotlight), made seamless, levels matched to the world's ground_a
                      (low darker, high lighter, so the three levels read at a glance).
  stairs.png          128x128 steps going up (drawn on ramps, rotated to the slope).
  cliff.png           256x128 cliff face, seamless left to right (drawn on south-facing cliffs;
                      power-of-two so the GPU can tile it on WebGL 1).
  trail.png           128x128 path surface for roads (optional --trail), a little lighter.
"""
import argparse, pathlib, sys

import numpy as np
from PIL import Image
from scipy import ndimage

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from restyle import match_levels, seamless  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[2]


def flatten(im, sigma=0.18):
    """Divide out large-scale lighting, keeping the detail and the average colour."""
    a = np.asarray(im.convert("RGB")).astype(np.float32)
    s = sigma * max(a.shape[:2])
    blur = np.stack([ndimage.gaussian_filter(a[..., c], s, mode="wrap") for c in range(3)], -1)
    out = a / np.maximum(blur, 1.0) * a.mean((0, 1))
    return Image.fromarray(out.clip(0, 255).astype(np.uint8))


def scale_levels(im, k):
    a = np.asarray(im.convert("RGB")).astype(np.float32) * k
    return Image.fromarray(a.clip(0, 255).astype(np.uint8))


def seamless_x(im):
    """Blend a strip with its half-width offset copy so its left and right edges meet."""
    a = np.asarray(im).astype(np.float32)
    w = a.shape[1]
    rolled = np.roll(a, w // 2, axis=1)
    xx = np.arange(w)
    d = np.minimum(xx, w - 1 - xx) / (w / 2)
    m = np.clip((d - 0.1) / 0.3, 0, 1)[None, :, None]
    return Image.fromarray((a * m + rolled * (1 - m)).clip(0, 255).astype(np.uint8))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("world")
    for k in ("low", "high", "stairs", "cliff"):
        ap.add_argument(f"--{k}", required=False)
    ap.add_argument("--trail")
    a = ap.parse_args()
    out = ROOT / "art" / "hd" / a.world
    ref = Image.open(out / "ground_a.png")
    if a.trail:
        im = flatten(Image.open(a.trail)).resize((128, 128), Image.LANCZOS)
        mixed = 0.5 * np.asarray(match_levels(im, ref).convert("RGB")).astype(np.float32) + 0.5 * np.asarray(im).astype(np.float32)
        seamless(scale_levels(Image.fromarray(mixed.clip(0, 255).astype(np.uint8)), 1.15)).convert("RGBA").save(out / "trail.png", optimize=True)
        print(f"wrote {a.world}/trail.png")
    if not a.low:
        return
    for name, k in (("low", 0.85), ("high", 1.35)):
        im = flatten(Image.open(a.__dict__[name])).resize((128, 128), Image.LANCZOS)
        # Half the world's palette (so it belongs), half its own colour (so it reads as different
        # ground), then darker or lighter than the ground between.
        own = np.asarray(im).astype(np.float32)
        own = own / own.mean() * np.asarray(ref.convert("RGB")).astype(np.float32).mean()
        mixed = 0.5 * np.asarray(match_levels(im, ref).convert("RGB")).astype(np.float32) + 0.5 * own
        im = scale_levels(Image.fromarray(mixed.clip(0, 255).astype(np.uint8)), k)
        seamless(im).convert("RGBA").save(out / f"{name}.png", optimize=True)
    Image.open(a.stairs).convert("RGB").resize((128, 128), Image.LANCZOS).convert("RGBA").save(out / "stairs.png", optimize=True)
    cliff = Image.open(a.cliff).convert("RGB")
    cliff = cliff.resize((256, round(256 * cliff.height / cliff.width)), Image.LANCZOS).resize((256, 128), Image.LANCZOS)
    seamless_x(cliff).convert("RGBA").save(out / "cliff.png", optimize=True)
    print("wrote", ", ".join(f"{a.world}/{n}.png" for n in ("low", "high", "stairs", "cliff")))


if __name__ == "__main__":
    main()
