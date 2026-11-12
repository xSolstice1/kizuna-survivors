"""Prototype of the terrain bake (the look the client reproduces in Rust), to iterate quickly.

    python3 tools/ai/terrain_bake.py w1 [--size 1024x576] [--at 0,0]

Writes art/ai/preview/bake_wN.png at desktop size (2 px per world unit). Uses the same noise as
sim::terrain, but continuous: the sim rounds it to 16-unit cells for collision, the picture
draws its smooth contour lines, so edges are organic and stay within half a cell of the walls.

Per pixel: the level (low / ground / high) with soft edges; ground textures at 64 units per
repeat with slow brightness variation; roads as soft-edged paths with stairs where they climb;
a cliff face on every south-facing drop (the pixel's depth into the drop picks the texture row);
darker side walls; a shadow on the lower ground below each cliff.
"""
import argparse, pathlib

import numpy as np
from PIL import Image

ROOT = pathlib.Path(__file__).resolve().parents[2]
CELL, PX = 16.0, 2
TILE_UNITS = 64.0
FACE = 16.0               # cliff face height (world units) = one wall cell
STAGE_INDEX = {"w1": 1, "w2": 2, "w3": 3, "w4": 4, "w5": 5}
M = np.uint64(0xFFFFFFFF)


def hash32(x, y, salt):
    """sim::stage::hash, vectorised (x, y: int arrays)."""
    x = x.astype(np.int64).astype(np.uint64) & M
    y = y.astype(np.int64).astype(np.uint64) & M
    h = ((x * np.uint64(0x9E3779B1)) & M) ^ ((y * np.uint64(0x85EBCA77)) & M) ^ np.uint64((salt * 0xC2B2AE3D) & 0xFFFFFFFF)
    h ^= h >> np.uint64(15)
    h = (h * np.uint64(0x2C1B3C6D)) & M
    h ^= h >> np.uint64(12)
    h = (h * np.uint64(0x297A2D39)) & M
    return h ^ (h >> np.uint64(15))


def sm(t):
    t = np.clip(t, 0.0, 1.0)
    return t * t * (3 - 2 * t)


def noise(x, y, salt):
    """sim::terrain::noise (value noise, smoothstep-interpolated)."""
    xi, yi = np.floor(x).astype(np.int64), np.floor(y).astype(np.int64)
    fx, fy = sm(x - xi), sm(y - yi)
    v = lambda dx, dy: (hash32(xi + dx, yi + dy, salt) & np.uint64(0xFFFF)).astype(np.float32) / 65535.0
    top = v(0, 0) + (v(1, 0) - v(0, 0)) * fx
    bot = v(0, 1) + (v(1, 1) - v(0, 1)) * fx
    return top + (bot - top) * fy


def height(stage, wx, wy):
    """The continuous field behind sim::terrain::level_raw, in cell units."""
    s = 0x5EED0000 + stage * 977
    x, y = wx / CELL, wy / CELL
    n = noise(x / 15.0, y / 15.0, s) * 0.7 + noise(x / 6.0, y / 6.0, s + 1) * 0.3
    d = np.sqrt(x * x + y * y)
    k = sm((d - 10.0) / 8.0)
    return np.where(d < 18.0, 0.5 + (n - 0.5) * k, n)


def level(h):
    """Soft level 0..2 (thresholds as in level_raw, a little soft)."""
    return sm((h - 0.34) / 0.012 + 0.5) + sm((h - 0.62) / 0.012 + 0.5)


def road(stage, wx, wy):
    """0..1 road share: distance to the nearest road centre line (unrounded wobble)."""
    s = 0x5EED0000 + stage * 977
    x, y = wx / CELL, wy / CELL
    best = np.full(x.shape, 99.0, np.float32)
    for c, t, off in ((x, y, 11), (y, x, 23)):
        k0 = np.round(c / 24.0)
        for dk in (-1, 0, 1):
            k = k0 + dk
            wob = (noise(t / 18.0, k * 3.7, s + off) - 0.5) * 9.0
            best = np.minimum(best, np.abs(c - (k * 24 + wob + 0.5)))
    return sm((1.9 - best) / 0.5)


def sample(tex, wx, wy, units):
    h, w = tex.shape[:2]
    u = ((wx / units) % 1.0 * w).astype(np.int32) % w
    v = ((wy / units) % 1.0 * h).astype(np.int32) % h
    return tex[v, u]


def bake(world, x0, y0, W, H):
    stage = STAGE_INDEX[world]
    ys, xs = np.mgrid[0:H, 0:W].astype(np.float32)
    wx, wy = x0 + xs / PX, y0 + ys / PX
    tex = {n: np.asarray(Image.open(ROOT / "art" / "hd" / world / f"{n}.png").convert("RGB")).astype(np.float32)
           for n in ("ground_a", "ground_b", "trail", "low", "high", "stairs", "cliff")}
    # Softer ground: halve the contrast of the busy tiles around their mean.
    for n in ("ground_a", "ground_b", "low", "high"):
        t = tex[n]
        tex[n] = t.mean((0, 1)) + (t - t.mean((0, 1))) * 0.6
    L = level(height(stage, wx, wy))
    lo, hi = np.clip(1 - L, 0, 1), np.clip(L - 1, 0, 1)
    mid = np.clip(1 - lo - hi, 0, 1)
    patch = sm((noise(wx / 90.0, wy / 90.0, 7) - 0.5) * 5 + 0.5)
    g_mid = sample(tex["ground_a"], wx, wy, TILE_UNITS) * (1 - patch[..., None]) + sample(tex["ground_b"], wx, wy, TILE_UNITS) * patch[..., None]
    img = sample(tex["low"], wx, wy, TILE_UNITS) * lo[..., None] + g_mid * mid[..., None] + sample(tex["high"], wx, wy, TILE_UNITS) * hi[..., None]
    img *= (0.9 + 0.2 * noise(wx / 40.0, wy / 40.0, 9))[..., None]
    # Roads, and stairs where the level changes along them.
    r = road(stage, wx, wy)
    img = img * (1 - r[..., None]) + sample(tex["trail"], wx, wy, 48.0) * r[..., None]
    # A south-facing drop shows its rock face BELOW the edge (over the lower ground): a pixel
    # is on a face if the ground a little above it is higher; how far above picks the row.
    steps = (3.0, 6.0, 9.0, 12.0, 15.0, FACE + 2)
    above = [level(height(stage, wx, wy - d)) for d in steps]
    edge = np.full(L.shape, 99.0, np.float32)
    for d, a_ in zip(steps, above):
        edge = np.where((a_ - L > 0.5) & (edge > 90), d, edge)
    on_face = (edge < 90) & (r < 0.5)
    depth = np.where(on_face, edge / (FACE + 2), 0)          # 0 at the top edge, 1 at the foot
    cl = tex["cliff"]
    u = ((wx / 64.0) % 1.0 * cl.shape[1]).astype(np.int32) % cl.shape[1]
    v = np.clip((depth * 0.98 * cl.shape[0]).astype(np.int32), 0, cl.shape[0] - 1)
    fa = on_face.astype(np.float32) * (1 - r)
    img = img * (1 - fa[..., None]) + cl[v, u] * fa[..., None]
    # Shadow on the ground just below a face.
    shade = (level(height(stage, wx, wy - FACE - 8)) - L > 0.5) & ~on_face
    img *= (1 - 0.38 * shade.astype(np.float32) * (1 - r))[..., None]
    # Stairs where a road climbs.
    climb = r * ((np.abs(level(height(stage, wx, wy - 10)) - level(height(stage, wx, wy + 10))) + np.abs(level(height(stage, wx - 10, wy)) - level(height(stage, wx + 10, wy)))) > 0.5)
    img = img * (1 - climb[..., None] * 0.9) + sample(tex["stairs"], wx, wy, 32.0) * climb[..., None] * 0.9
    # Rims on the other sides: a thin dark edge where the ground drops north, west or east.
    rim = np.zeros(L.shape, np.float32)
    for dx, dy in ((0, -5), (-5, 0), (5, 0)):
        rim = np.maximum(rim, (L - level(height(stage, wx + dx, wy + dy)) > 0.5).astype(np.float32))
    img *= (1 - 0.45 * rim * (1 - r))[..., None]
    # A soft light rim on the top edge of each south face.
    lip = (L - level(height(stage, wx, wy + 3)) > 0.5) & (r < 0.5)
    img = np.where(lip[..., None], img * 1.25, img)
    return np.clip(img, 0, 255).astype(np.uint8)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("world")
    ap.add_argument("--size", default="1024x576", help="world units")
    ap.add_argument("--at", default="0,0", help="centre (world units)")
    a = ap.parse_args()
    sw, sh = map(float, a.size.split("x"))
    cx, cy = map(float, a.at.split(","))
    img = bake(a.world, cx - sw / 2, cy - sh / 2, int(sw * PX), int(sh * PX))
    out = ROOT / "art" / "ai" / "preview" / f"bake_{a.world}.png"
    Image.fromarray(img).save(out)
    print("wrote", out)


if __name__ == "__main__":
    main()
