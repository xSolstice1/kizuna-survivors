"""Build a world's terrain textures (what the terrain shader draws) from the 2K generations of
tools/ai/terrain_gen.py.

    python3 tools/ai/terrain_hd.py w1 [w2 ...]

Writes art/hd/<world>/terrain/ (JPEG q88):
  ground_a, low, high, trail   1024x1024, 128 world units per repeat (8 texels per unit).
  cliff                        1024x512 rock face, 64x32 units: top edge = the lip, bottom = the foot.

Tiles are made seamless WITHOUT cross-fading (which leaves ghost copies of rocks): the image model
usually paints a swatch that already repeats every 1024 px, so one period is cut out, and the
remaining mismatch at its edges is hidden by a minimum-error cut through an overlap band (image
quilting), so every pixel comes from one place in the picture. Large-scale lighting is divided
out and the levels set relative to the world's ground: low ground darker, high ground lighter.
Which raw is used, and the cliff's face rows, are in SOURCES below.
"""
import pathlib, sys

import numpy as np
from PIL import Image
from scipy import ndimage

ROOT = pathlib.Path(__file__).resolve().parents[2]
RAW = ROOT / "art" / "ai" / "raw"
T = 1024
BAND = 96

# world: {kind: raw index}, cliff face rows (lip, foot) in the 3168x1344 raw.
SOURCES = {
    "w1": ({"high": 1}, (170, 1320)),
    "w2": ({"trail": 1}, (170, 1320)),
    "w3": ({}, (110, 1325)),
    "w4": ({"ground": 1, "low": 1, "high": 1, "trail": 1}, (270, 1320)),
    "w5": ({}, (290, 1325)),
}
# Brightness and contrast (spread around the mean) relative to the ground's.
LEVEL = {"ground": (1.0, 0.9), "low": (0.8, 0.8), "high": (1.18, 0.8), "trail": (1.08, 0.75)}
# Whole-world brightness (the citadel's white stone glares otherwise).
GAIN = {"w3": 0.8}
# How much colour the detail keeps around the mean (the w1 road's pebbles read as confetti).
CHROMA = {"trail": 0.5}
CUT = ROOT / "art" / "ai" / "terrain"   # seamless tiles before levelling (slow to make)


def raw(world, kind, idx):
    return np.asarray(Image.open(RAW / f"{world}_t2_{kind}_{idx}.png").convert("RGB")).astype(np.float32)


def flatten(a, sigma=0.12):
    """Divide out large-scale lighting (wrapping, since the swatch repeats), keeping the mean."""
    s = sigma * max(a.shape[:2])
    blur = np.stack([ndimage.gaussian_filter(a[..., c], s, mode="wrap") for c in range(3)], -1)
    return a / np.maximum(blur, 1.0) * a.mean((0, 1))


def find_period(a, axis, lo=900, hi=1150):
    """The repeat of a swatch along an axis (px) and how well it repeats (mean abs diff / spread)."""
    g = a.mean(-1)
    n = g.shape[axis]
    best = (1e9, 0)
    for p in range(lo, min(hi, n - 200)):
        x, y = np.take(g, range(0, n - p), axis=axis), np.take(g, range(p, n), axis=axis)
        best = min(best, (float(np.abs(x - y).mean()), p))
    return best[1], best[0] / max(float(np.abs(g - g.mean()).mean()), 1e-3)


def min_cut(err):
    """Vertical minimum-error path through err (h x w): column index per row (dynamic programming)."""
    h, w = err.shape
    cost = err.copy()
    for y in range(1, h):
        prev = cost[y - 1]
        left = np.r_[np.inf, prev[:-1]]
        right = np.r_[prev[1:], np.inf]
        cost[y] += np.minimum(np.minimum(left, prev), right)
    path = np.zeros(h, int)
    path[-1] = int(np.argmin(cost[-1]))
    for y in range(h - 2, -1, -1):
        x = path[y + 1]
        lo, hi = max(0, x - 1), min(w, x + 2)
        path[y] = lo + int(np.argmin(cost[y, lo:hi]))
    return path


def wrap_x(src, w):
    """A strip of width w that tiles left to right, cut from src (width >= w + BAND): the first
    BAND columns are joined to the continuation past column w along a minimum-error cut."""
    a, b = src[:, :BAND], src[:, w:w + BAND]
    path = min_cut(np.abs(a - b).sum(-1))
    cols = np.arange(BAND)[None, :]
    # Left of the cut: the continuation (b) of the strip's end; right of it: the start (a).
    m = (cols >= path[:, None]).astype(np.float32)
    m = ndimage.uniform_filter1d(m, 3, axis=1)[..., None]   # 1-2 px soft edge only
    out = src[:, :w].copy()
    out[:, :BAND] = b * (1 - m) + a * m
    return out


def seamless_tile(a, square=True):
    """A seamless tile from a swatch: one period if it repeats, else as big as fits."""
    out = []
    for axis in (1, 0):
        p, q = find_period(a, axis)
        out.append(p if q < 0.45 else 1024)  # not periodic: same scale, cut to tile
    pw, ph = out
    if square:
        pw = ph = min(pw, ph) if abs(pw - ph) > 8 else pw
    x0 = max(0, (a.shape[1] - pw - BAND) // 2)
    y0 = max(0, (a.shape[0] - ph - BAND) // 2)
    t = a[y0:y0 + ph + BAND, x0:x0 + pw + BAND]
    t = wrap_x(t, pw)
    t = wrap_x(t.transpose(1, 0, 2), ph).transpose(1, 0, 2)
    return t, (pw, ph)


def lum(a):
    return float((a * [0.3, 0.59, 0.11]).sum(-1).mean())


def spread(a):
    return float(np.abs(a - a.mean((0, 1))).mean())


def level(a, target, contrast, chroma=1.0):
    m = a.mean((0, 1))
    d = a - m
    grey = d.mean(-1, keepdims=True)
    a = m + (grey + (d - grey) * chroma) * np.clip(contrast / max(spread(a), 1e-3), 0.5, 2.5)
    return a * (target / max(lum(a), 1.0))


def save(a, path, size):
    Image.fromarray(a.clip(0, 255).astype(np.uint8)).resize(size, Image.LANCZOS).save(path, quality=88, subsampling=0, optimize=True)


def build(world):
    idx, (lip, foot) = SOURCES[world]
    out = ROOT / "art" / "hd" / world / "terrain"
    out.mkdir(parents=True, exist_ok=True)
    CUT.mkdir(parents=True, exist_ok=True)
    tiles = {}
    for kind in ("ground", "low", "high", "trail"):
        cached = CUT / f"{world}_{kind}_{idx.get(kind, 0)}.png"
        if cached.exists() and cached.stat().st_mtime > (RAW / f"{world}_t2_{kind}_{idx.get(kind, 0)}.png").stat().st_mtime:
            tiles[kind] = np.asarray(Image.open(cached)).astype(np.float32)
            continue
        t, period = seamless_tile(flatten(raw(world, kind, idx.get(kind, 0))))
        Image.fromarray(t.clip(0, 255).astype(np.uint8)).save(cached)
        tiles[kind] = t
        print(f"{world} {kind}: period {period}")
    g, gs = lum(tiles["ground"]) * GAIN.get(world, 1.0), spread(tiles["ground"]) * GAIN.get(world, 1.0)
    for kind, t in tiles.items():
        k, c = LEVEL[kind]
        save(level(t, g * k, gs * c, CHROMA.get(kind, 1.0)), out / f"{'ground_a' if kind == 'ground' else kind}.jpg", (T, T))
    # Cliff: the face rows, flattened along x only (keep the darkening towards the foot),
    # 2:1 so a 32-unit face repeats every 64 units.
    c = raw(world, "cliff", idx.get("cliff", 0))[lip:foot]
    blur = ndimage.gaussian_filter1d(c, 0.15 * c.shape[1], axis=1, mode="wrap")
    c = c / np.maximum(blur, 1.0) * blur.mean(1, keepdims=True)
    h = c.shape[0]
    w = min(2 * h, c.shape[1] - BAND - 1)
    x0 = (c.shape[1] - w - BAND) // 2
    save(wrap_x(c[:, x0:x0 + w + BAND], w), out / "cliff.jpg", (T, T // 2))
    for old in ("ground_b", "stairs"):
        (out / f"{old}.jpg").unlink(missing_ok=True)
    print("wrote", out.relative_to(ROOT))


if __name__ == "__main__":
    for w in sys.argv[1:]:
        build(w)
