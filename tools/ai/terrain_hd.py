"""Build a world's high-resolution terrain textures (what the terrain shader draws) from the
full-size generations in art/ai.

    python3 tools/ai/terrain_hd.py w1 [w2 ...]

Writes art/hd/<world>/terrain/ (JPEG, quality 90: opaque, and a tenth the size of PNG):
  ground_a, ground_b           512x512 main ground (128 world units per repeat), from the
                               restyled tiles (art/ai/restyle/tile_<w>_ground_*.png).
  low, high                    512x512 low and high ground (raw/<w>_low_*, <w>_high_*):
                               half the world's palette, half their own, darker / lighter.
  trail                        512x512 road surface (raw/<w>_trail_*).
  stairs                       512x256 one flight climbing to the top edge, seamless left to
                               right, spanning a whole cliff face (raw/<w>_stairs_wide_*,
                               prompt art/prompts/terrain/<w>_stairs_wide.txt).
  cliff                        512x256 rock face, top edge = the lip (raw/<w>_cliff_*).
Lighting is flattened and tiles made seamless. Faces are 64x32 world units per repeat (a face
is 2 cells, 32 units, tall).
"""
import pathlib, sys

import numpy as np
from PIL import Image

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from restyle import match_levels, seamless  # noqa: E402
from terrain_art import flatten, scale_levels, seamless_x  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[2]
AI = ROOT / "art" / "ai"
T = 512


def latest(pattern):
    found = sorted(AI.glob(pattern), key=lambda p: p.stat().st_mtime)
    if not found:
        raise SystemExit(f"no art/ai/{pattern}")
    return Image.open(found[-1]).convert("RGB")


def tile(im):
    return flatten(im).resize((T, T), Image.LANCZOS)


def strip(im):
    """Centre-crop to 2:1 and size to 512x256."""
    w, h = im.size
    if w / h > 2:
        cw = 2 * h
        im = im.crop(((w - cw) // 2, 0, (w - cw) // 2 + cw, h))
    return im.resize((T, T // 2), Image.LANCZOS)


def build(world):
    out = ROOT / "art" / "hd" / world / "terrain"
    out.mkdir(parents=True, exist_ok=True)
    save = lambda im, n: im.convert("RGB").save(out / f"{n}.jpg", quality=90, subsampling=0, optimize=True)
    a = seamless(tile(latest(f"restyle/tile_{world}_ground_a.png"))).convert("RGB")
    save(a, "ground_a")
    save(seamless(match_levels(tile(latest(f"restyle/tile_{world}_ground_b.png")), a)), "ground_b")
    ref = np.asarray(a).astype(np.float32)
    for name, k in (("low", 0.85), ("high", 1.3)):
        im = tile(latest(f"raw/{world}_{name}_*.png"))
        own = np.asarray(im).astype(np.float32)
        own = own / own.mean() * ref.mean()
        mixed = 0.5 * np.asarray(match_levels(im, a).convert("RGB")).astype(np.float32) + 0.5 * own
        save(seamless(scale_levels(Image.fromarray(mixed.clip(0, 255).astype(np.uint8)), k)), name)
    im = tile(latest(f"raw/{world}_trail_*.png"))
    mixed = 0.5 * np.asarray(match_levels(im, a).convert("RGB")).astype(np.float32) + 0.5 * np.asarray(im).astype(np.float32)
    save(seamless(scale_levels(Image.fromarray(mixed.clip(0, 255).astype(np.uint8)), 1.15)), "trail")
    save(seamless_x(strip(latest(f"raw/{world}_stairs_wide_*.png"))), "stairs")
    save(seamless_x(strip(latest(f"raw/{world}_cliff_*.png"))), "cliff")
    print("wrote", out.relative_to(ROOT))


if __name__ == "__main__":
    for w in sys.argv[1:]:
        build(w)
