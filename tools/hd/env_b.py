"""HD ground tiles and props for worlds 3 (Pale Citadel), 4 (Rainward 13) and 5 (Shatterline).

    python3 tools/hd/env_b.py

Tiles: every ground tile of a world is built on one shared base (slab grid / asphalt) and
its own details are faded out towards the tile border, so ground_a, ground_b and decal
meet each other seamlessly in any arrangement. Paths are seamless with themselves vertically.
"""
import math
import pathlib
import random
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from PIL import Image, ImageChops, ImageDraw, ImageFilter  # noqa: E402

import hdkit  # noqa: E402
from hdkit import Art, SS, rgb  # noqa: E402


# ───────────────────────── helpers ─────────────────────────

def _edge_mask(w, h, band=2.6, inner=0.6):
    """L mask: 0 at the tile border, 255 once `band` art px inside."""
    n = 128
    m = Image.new("L", (n, n))
    px = m.load()
    for y in range(n):
        for x in range(n):
            ax, ay = (x + 0.5) / n * w, (y + 0.5) / n * h
            d = min(ax, ay, w - ax, h - ay)
            t = max(0.0, min(1.0, (d - inner) / band))
            t = t * t * (3 - 2 * t)
            px[x, y] = int(t * 255)
    return m.resize((w * SS, h * SS), Image.BILINEAR)


class Tile:
    """A tile split into a shared base (kept exactly at the edges) and faded details."""

    def __init__(self, base_fn, w=32, h=32):
        self.a = Art(w, h, tile=True)
        base_fn(self.a)
        self.base = self.a.img.copy()

    def finish(self, path):
        mask = _edge_mask(self.a.w, self.a.h)
        self.a.img = Image.composite(self.a.img, self.base, mask)
        return self.a.save(path)


def soft(a, cx, cy, rx, ry, color, alpha, steps=5):
    """A soft-edged patch (stacked ellipses)."""
    for i in range(steps):
        k = 1 - i / steps
        a.ellipse(cx, cy, rx * k, ry * k, color, alpha=alpha / steps * 1.6)


def halo(a, cx, cy, r, color, alpha=0.5, over=0.0):
    """Soft light halo behind the art (and optionally a little over it)."""
    m = Image.new("L", (a.W, a.H), 0)
    d = ImageDraw.Draw(m)
    d.ellipse([(cx - r * 0.55) * SS, (cy - r * 0.55) * SS, (cx + r * 0.55) * SS, (cy + r * 0.55) * SS], fill=255)
    m = m.filter(ImageFilter.GaussianBlur(r * SS * 0.45))
    m = ImageChops.multiply(m, _edge_mask(a.w, a.h, band=2.5, inner=0.0))   # never clipped hard at the canvas edge
    lay = Image.new("RGBA", (a.W, a.H), rgb(color))
    lay.putalpha(m.point(lambda v: int(v * alpha)))
    if over > 0:
        top = Image.new("RGBA", (a.W, a.H), rgb(color))
        top.putalpha(ImageChops.multiply(m, a.img.getchannel("A")).point(lambda v: int(v * over)))
    lay.alpha_composite(a.img)
    a.img = lay
    if over > 0:
        a.img.alpha_composite(top)


def behind(a, b):
    """Put art `b` behind art `a`."""
    b.img.alpha_composite(a.img)
    a.img = b.img


def bevel_slab(a, x0, y0, x1, y1, col, hi, lo, r=0.7):
    a.rect(x0, y0, x1, y1, lo, radius=r)
    a.rect(x0, y0, x1 - 0.55, y1 - 0.55, col, radius=r)
    a.stroke([(x0 + 0.8, y0 + 0.35), (x1 - 1.0, y0 + 0.35)], 0.45, hi, alpha=0.7, smooth=False)
    a.stroke([(x0 + 0.35, y0 + 0.8), (x0 + 0.35, y1 - 1.0)], 0.45, hi, alpha=0.45, smooth=False)


def specks(a, rng, n, box, cols, rr=(0.18, 0.35), alpha=(0.3, 0.6)):
    x0, y0, x1, y1 = box
    for _ in range(n):
        x, y = rng.uniform(x0, x1), rng.uniform(y0, y1)
        r = rng.uniform(*rr)
        a.ellipse(x, y, r, r * rng.uniform(0.7, 1.0), rng.choice(cols), alpha=rng.uniform(*alpha))


def crack(a, pts, w, col, alpha=1.0, glow_col=None):
    if glow_col:
        a.stroke(pts, w * 3.2, glow_col, alpha=0.18 * alpha)
        a.stroke(pts, w * 1.9, glow_col, alpha=0.35 * alpha)
    a.stroke(pts, w, col, alpha=alpha, taper=False)


# ───────────────────────── W3: Pale Citadel ─────────────────────────
W3 = dict(M="#1b1f2b", A="#2a3041", B="#2e3546", C="#272c3c", H="#353d51", D="#222735", S="#2c3a4a")


def w3_base(cols):
    """Slab grid: joints at y=0,16; x=0,16 on the top row, x=8,24 on the bottom row.
    The slab crossing the tile's left/right edge (bottom row) is always colour B."""
    p = W3

    def f(a):
        a.rect(0, 0, 32, 32, p["M"])
        tl, tr, bm = cols
        g = 0.45
        bevel_slab(a, 0 + g, 0 + g, 16 - g, 16 - g, tl, p["H"], p["D"])
        bevel_slab(a, 16 + g, 0 + g, 32 - g, 16 - g, tr, p["H"], p["D"])
        bevel_slab(a, 8 + g, 16 + g, 24 - g, 32 - g, bm, p["H"], p["D"])
        bevel_slab(a, 24 + g, 16 + g, 40 - g, 32 - g, p["B"], p["H"], p["D"])
    return f


def w3_slab_sheen(a, boxes):
    for (x0, y0, x1, y1) in boxes:
        soft(a, x0 + (x1 - x0) * 0.38, y0 + (y1 - y0) * 0.36, (x1 - x0) * 0.32, (y1 - y0) * 0.28, "#3a4560", 0.22)


def w3_tiles():
    p = W3
    rng = random.Random(31)
    # ground_a
    t = Tile(w3_base((p["A"], p["B"], p["C"])))
    a = t.a
    w3_slab_sheen(a, [(1, 1, 15, 15), (17, 1, 31, 15), (9, 17, 23, 31)])
    specks(a, rng, 14, (2, 2, 14, 14), [p["D"], p["C"]])
    specks(a, rng, 10, (18, 2, 30, 14), [p["D"], p["A"]])
    specks(a, rng, 12, (10, 18, 22, 30), [p["D"], p["A"]])
    crack(a, [(3.5, 3.5), (5, 5.5), (4.5, 7.5), (6.5, 9.5)], 0.35, p["D"])
    t.finish("w3/ground_a")

    # ground_b: same grid, the top-left slab split in two, a chipped split slab bottom middle
    t = Tile(w3_base((p["B"], p["A"], p["A"])))
    a = t.a
    a.rect(0.3, 7.2, 15.7, 8.3, p["M"])
    bevel_slab(a, 0.45, 8.3, 15.55, 15.55, p["B"], p["H"], p["D"])
    a.rect(8.3, 23.0, 23.7, 24.1, p["M"])
    bevel_slab(a, 8.45, 24.1, 23.55, 31.55, p["A"], p["H"], p["D"])
    w3_slab_sheen(a, [(1, 1, 15, 7), (17, 1, 31, 15), (9, 17, 23, 23)])
    specks(a, rng, 10, (2, 9, 14, 14), [p["D"], p["C"]])
    specks(a, rng, 12, (18, 2, 30, 14), [p["D"], p["C"]])
    specks(a, rng, 8, (10, 25, 22, 30), [p["D"], p["C"]])
    # weathered cracks with a cold mossy tint
    crack(a, [(17.5, 4), (20, 5.5), (23.5, 4.5), (26, 6.5)], 0.4, p["D"])
    crack(a, [(25, 5), (26.5, 9), (29, 10.5)], 0.3, p["D"])
    a.stroke([(18, 5.2), (20, 6.4), (23.5, 5.5)], 0.5, p["S"], alpha=0.6)
    soft(a, 6, 11.5, 3, 1.6, p["S"], 0.45)
    soft(a, 14, 27, 3.5, 1.5, p["S"], 0.35)
    a.poly([(23.55, 8), (22.2, 11), (23.55, 13)], p["M"], alpha=0.7)   # chipped edge
    t.finish("w3/ground_b")

    # decal: ground_a slabs cracked by a glowing spirit sigil
    t = Tile(w3_base((p["A"], p["B"], p["C"])))
    a = t.a
    w3_slab_sheen(a, [(1, 1, 15, 15), (17, 1, 31, 15), (9, 17, 23, 31)])
    cx, cy = 16, 15.5
    a.ellipse(cx, cy, 11, 10.5, "#0f2236", alpha=0.35)
    for rr, wd, col, al in ((9.6, 1.2, "#2f5578", 1.0), (9.6, 0.5, "#4f9be0", 0.9), (6.2, 0.45, "#4f9be0", 0.8)):
        pts = [(cx + rr * math.cos(t_ * math.pi / 18), cy + rr * math.sin(t_ * math.pi / 18)) for t_ in range(37)]
        a.stroke(pts, wd, col, alpha=al, smooth=False)
    # runes on the ring
    for k in range(8):
        ang = k * math.pi / 4 + 0.3
        x, y = cx + 7.9 * math.cos(ang), cy + 7.9 * math.sin(ang)
        a.ellipse(x, y, 0.6, 0.6, "#8fd0ff", alpha=0.85)
    # star inside
    star = [(cx + 6.0 * math.cos(-math.pi / 2 + k * 4 * math.pi / 5), cy + 6.0 * math.sin(-math.pi / 2 + k * 4 * math.pi / 5)) for k in range(6)]
    a.stroke(star, 0.4, "#4f9be0", alpha=0.75, smooth=False)
    # cracks radiating from the centre, glowing
    for pts in ([(cx, cy), (12.5, 11), (10, 7.5), (7, 5)], [(cx, cy), (20, 18.5), (22.5, 23)],
                [(cx, cy), (13, 19.5), (11.5, 24)], [(cx, cy), (19.5, 12), (23, 9)]):
        crack(a, pts, 0.42, "#a8dcff", glow_col="#4f9be0")
    soft(a, cx, cy, 2.6, 2.6, "#a8dcff", 0.9)
    a.ellipse(cx, cy, 0.9, 0.9, "#eaf6ff")
    t.finish("w3/decal")

    # path: polished dark-slab walkway between curb stones (seamless vertically)
    a = Art(32, 32, tile=True)
    a.rect(0, 0, 32, 32, "#141822")
    for x0, x1 in ((-0.5, 2.0), (30.0, 32.5)):
        for y0 in (0, 16):
            bevel_slab(a, x0, y0 + 0.4, x1, y0 + 15.6, "#363d50", "#434b62", "#2c3242", r=0.4)
    rows = [(0, "#1d2231", 20), (8, "#212738", 11), (16, "#1d2231", 20), (24, "#212738", 11)]
    for y0, col, j in rows:
        for xa, xb in ((2.9, j - 0.35), (j + 0.35, 29.1)):
            a.rect(xa, y0 + 0.35, xb, y0 + 7.65, "#181c28", radius=0.4)
            a.rect(xa, y0 + 0.35, xb - 0.4, y0 + 7.25, col, radius=0.4)
            a.stroke([(xa + 0.6, y0 + 0.6), (xb - 0.8, y0 + 0.6)], 0.35, "#2b3449", alpha=0.8, smooth=False)
    # polished reflections: soft diagonal sheen streaks
    for x, y in ((6, 2), (22, 10), (9, 18), (24, 27)):
        a.stroke([(x, y + 3), (x + 3, y)], 0.7, "#2b3449", alpha=0.55)
        a.stroke([(x + 1.6, y + 3.4), (x + 3.6, y + 1.4)], 0.35, "#36435e", alpha=0.45)
    a.save("w3/path")


def w3_props():
    # prop_a: spirit-flame brazier
    a = Art(16, 32)
    a.blob([(4.2, 27.5), (11.8, 27.5), (13.2, 31.6), (2.8, 31.6)], ("lin", "#5a5c76", "#2a2a3a", 90), smooth=False)
    a.rect(3, 30.6, 13, 31.8, "#2a2a3a", radius=0.4)
    a.ellipse(8, 27.6, 3.6, 0.9, "#6a6c86")
    a.rect(6.6, 17, 9.4, 27.5, ("lin", "#6a6c86", "#2a2a3a", 0))
    a.rect(5.8, 21.2, 10.2, 22.9, ("lin", "#6a6c86", "#2a2a3a", 0), radius=0.5)
    a.blob([(1, 12.4), (15, 12.4), (13.2, 15.2), (11.6, 17.0), (4.4, 17.0), (2.8, 15.2)], ("lin", "#5a5c76", "#2a2a3a", 90), smooth=False)
    a.rect(0.8, 12.0, 15.2, 13.3, "#6a6c86", radius=0.6)
    a.stroke([(2, 12.3), (14, 12.3)], 0.35, "#9a9cb8", alpha=0.8)
    a.rect(3, 13.4, 13, 14.3, "#8fd0ff", radius=0.4)          # inner glow under the flame
    a.shade_side("#000000", 0.25, side="right", amount=0.35)
    # flame
    fl = [(3, 12.2), (2.6, 9), (4, 6.2), (4.6, 3.4), (6.2, 5.0), (7.2, 2.0), (8.6, 4.4), (10.6, 1.6), (11.0, 4.8), (12.8, 7.6), (13.2, 10.2), (13, 12.2)]
    a.blob(fl, ("lin", "#8fd0ff", "#3d8ee8", 270))
    a.blob([(4.6, 12), (4.3, 9), (5.6, 6.5), (7.4, 4.8), (8.2, 6.8), (9.8, 5.6), (11.2, 8.4), (11.4, 12)], "#8fd0ff")
    a.blob([(6.2, 12), (6.0, 9.6), (7.4, 7.2), (8.8, 9.2), (10.0, 12)], "#eef9ff")
    a.ellipse(8, 10.6, 1.0, 1.4, "#ffffff")
    a.outline()
    halo(a, 8, 8, 6, "#3d8ee8", 0.55, over=0.12)
    a.save("w3/prop_a")

    # prop_b: gate pillar with a black tiled cap and a glowing seal
    a = Art(32, 32)
    a.rect(3.8, 26.8, 28.2, 32, ("lin", "#7d8398", "#5a5f72", 90), radius=0.4)
    a.rect(4.6, 26.8, 27.4, 28.0, "#c8ccda", radius=0.4)
    a.rect(3.8, 29.8, 28.2, 32, "#5a5f72")
    a.rect(8.0, 11, 24.0, 27, ("lin", "#c8ccda", "#7d8398", 0))
    a.rect(8.0, 11, 11.2, 27, "#c8ccda", alpha=0.6)
    a.rect(19.8, 11, 24.0, 27, "#7d8398", alpha=0.7)
    for y in (11.0, 16.0, 22.0):
        a.rect(8.0, y, 24.0, y + 0.9, "#5a5f72")
    crack(a, [(11.4, 12.2), (12.4, 13.5), (11.8, 15.2)], 0.4, "#5a5f72")
    crack(a, [(19.2, 23), (20.2, 24.6), (21.2, 26.4)], 0.4, "#5a5f72")
    # glowing seal
    a.blob([(14.0, 17.2), (18.0, 17.2), (16.8, 19.5), (18.0, 21.8), (14.0, 21.8), (15.2, 19.5)], "#2f5578")
    a.blob([(14.9, 17.7), (17.1, 17.7), (16.3, 19.5), (17.1, 21.3), (14.9, 21.3), (15.7, 19.5)], "#4f9be0")
    a.ellipse(16, 19.5, 0.55, 1.4, "#cfeaff")
    # roof
    roof = [(3.6, 0.3), (28.4, 0.3), (28.3, 2.0), (27.0, 3.6), (30.4, 4.0), (31.6, 7.2), (29.6, 8.6), (28.0, 10.8),
            (4.0, 10.8), (2.4, 8.6), (0.4, 7.2), (1.6, 4.0), (5.0, 3.6), (3.7, 2.0)]
    a.poly(roof, ("lin", "#2d3045", "#1e2030", 90))
    a.rect(5.6, 2.0, 26.4, 3.0, "#464c66", radius=0.4)
    a.rect(4.0, 9.0, 28.0, 10.0, "#464c66", radius=0.4)
    for x in range(5, 28, 3):
        a.rect(x + 0.1, 4.0, x + 1.3, 8.6, "#2d3045", radius=0.5)
        a.stroke([(x + 0.4, 4.3), (x + 0.4, 8.2)], 0.35, "#464c66", alpha=0.9)
    a.stroke([(4.2, 0.6), (27.8, 0.6)], 0.4, "#464c66", alpha=0.8)
    a.ellipse(3.9, 0.9, 1.0, 0.8, "#2d3045")
    a.ellipse(28.1, 0.9, 1.0, 0.8, "#2d3045")
    a.outline()
    halo(a, 16, 19.5, 4, "#4f9be0", 0.35)
    a.save("w3/prop_b")

    # prop_c: white citadel wall with a black-tiled top
    a = Art(48, 32)
    a.rect(2.8, 25.8, 45.2, 32, "#7d8398")
    for x in range(3, 45, 7):
        a.rect(x + 0.35, 26.0, x + 6.65, 28.8, ("lin", "#a9aec0", "#7d8398", 90), radius=0.3)
        a.rect(x + 0.35 + (3.5 if x < 41 else 0), 29.6, min(x + 6.65 + 3.5, 45.0), 32, "#7d8398", radius=0.3)
    a.rect(2.8, 28.8, 45.2, 29.6, "#5a5f72")
    a.rect(2.8, 29.6, 3.8, 32, "#7d8398")
    a.rect(4.0, 12.5, 44.0, 26, ("lin", "#b4b8c8", "#959aad", 0))
    a.rect(4.0, 12.5, 44.0, 15.2, "#7a8094", alpha=0.85)
    a.rect(4.0, 15.0, 44.0, 16.0, "#959aad", alpha=0.6)
    a.rect(37.5, 15.5, 44.0, 26, "#959aad", alpha=0.6)
    soft(a, 14, 21, 7, 3, "#c4c8d6", 0.5)
    # grille window
    a.rect(19.8, 16.8, 28.2, 22.2, "#5a5f72", radius=0.4)
    a.rect(20.4, 17.4, 27.6, 21.6, "#2d3045", radius=0.3)
    for x in (21.4, 23.2, 25.0, 26.8):
        a.rect(x - 0.35, 17.4, x + 0.35, 21.6, "#a9aec0")
    a.rect(20.4, 19.1, 27.6, 19.7, "#7d8398")
    crack(a, [(33.6, 16.0), (32.8, 18.3), (34.0, 20.5), (33.2, 23.0), (33.6, 25.0)], 0.45, "#7a8094")
    crack(a, [(34.0, 20.5), (35.6, 21.8)], 0.3, "#7a8094")
    roof = [(0.4, 0.3), (47.6, 0.3), (47.6, 9.6), (46.4, 10.5), (46.0, 11.6), (44.5, 13.0), (3.5, 13.0), (1.8, 11.6), (1.2, 10.4), (0.3, 9.6)]
    a.poly(roof, ("lin", "#2d3045", "#1e2030", 90))
    a.rect(2.2, 2.0, 45.8, 3.0, "#464c66", radius=0.4)
    a.rect(1.4, 11.0, 46.4, 12.0, "#464c66", radius=0.4)
    for x in range(3, 45, 3):
        a.rect(x + 0.1, 4.0, x + 1.3, 9.8, "#2d3045", radius=0.5)
        a.stroke([(x + 0.4, 4.3), (x + 0.4, 9.4)], 0.35, "#464c66", alpha=0.9)
    a.stroke([(1.0, 0.6), (47.0, 0.6)], 0.4, "#464c66", alpha=0.8)
    a.outline()
    a.save("w3/prop_c")

    # prop_d: pile of broken bone-white masks on rubble
    a = Art(24, 24)
    a.blob([(2.2, 18.2), (6, 17.0), (12, 17.2), (18, 17.0), (21.4, 18.2), (21.2, 21.4), (19.6, 23.6), (3.6, 23.6), (1.8, 21)], ("lin", "#5a6070", "#353a48", 90))
    a.blob([(9.5, 19.6), (15.5, 19.6), (16, 22), (9, 22.2)], "#454a5a")
    a.blob([(15.8, 17.6), (21, 17.6), (21, 20.4), (17, 20.6)], "#5a6070")
    # left mask (lying)
    a.blob([(1.6, 15.2), (5.5, 14.2), (7.6, 16.4), (6.6, 19.2), (2.2, 19.4), (1.0, 17.3)], ("lin", "#d9d4c2", "#b3ad9a", 0))
    a.ellipse(4.0, 16.6, 0.9, 0.75, "#14121c")
    # right mask
    a.blob([(15.4, 12.6), (18.8, 11.4), (21.2, 12.8), (21.5, 16.2), (19.4, 18.4), (15.8, 18.0), (14.8, 15)], ("lin", "#d9d4c2", "#b3ad9a", 0))
    a.ellipse(18.6, 14.6, 1.0, 0.8, "#14121c")
    a.ellipse(20.6, 14.4, 0.5, 0.65, "#14121c")
    # big front mask, broken on the right
    big = [(6.2, 9.2), (10.2, 8.4), (13.8, 9.4), (13.0, 11.4), (14.4, 12.6), (13.4, 14.6), (14.2, 16.6), (11.2, 18.2), (6.4, 18.0), (4.6, 15.0), (4.8, 11.6)]
    a.blob(big, ("lin", "#ece8d8", "#b3ad9a", 20))
    a.blob([(7.6, 11.0), (10.0, 10.7), (10.4, 12.8), (9.4, 14.1), (7.4, 13.6)], "#14121c")
    a.blob([(11.4, 11.4), (13.0, 11.6), (12.8, 13.6), (11.4, 13.6)], "#14121c")
    a.ellipse(9.0, 12.9, 0.75, 0.75, "#6fc0ff")
    a.ellipse(9.0, 12.9, 0.3, 0.3, "#e0f4ff")
    a.stroke([(8.5, 16.0), (10.5, 16.4), (11.6, 15.8)], 0.4, "#8a8574")
    a.stroke([(13.4, 9.6), (12.4, 11.0), (13.6, 12.6)], 0.35, "#8a8574")
    a.stroke([(6.4, 9.8), (9.8, 9.2)], 0.4, "#ffffff", alpha=0.6)
    a.shade_side("#3a3020", 0.18, side="bottom", amount=0.3)
    a.outline()
    halo(a, 9, 12.9, 2.2, "#6fc0ff", 0.5)
    a.save("w3/prop_d")


# ───────────────────────── W4: Rainward 13 ─────────────────────────
W4 = dict(C="#16191f", A="#1b1f27", B="#20252e", H="#2a303b", S="#262d3a", s="#2f3848", r="#2a1c24", k="#121419")


def w4_base(a):
    a.rect(0, 0, 32, 32, W4["A"])


def w4_asphalt(a, rng, light, dark):
    for (x, y, rx, ry) in light:
        soft(a, x, y, rx, ry, W4["B"], 0.7)
    for (x, y, rx, ry) in dark:
        soft(a, x, y, rx, ry, W4["C"], 0.7)
    specks(a, rng, 26, (2, 2, 30, 30), [W4["H"], W4["C"], W4["B"]], rr=(0.16, 0.3), alpha=(0.35, 0.7))


def w4_sheen(a, pts, w=0.55):
    """Wet reflection streaks."""
    for (x, y, l) in pts:
        a.stroke([(x, y), (x + l, y)], w, W4["S"], alpha=0.9)
        a.stroke([(x + l * 0.3, y), (x + l * 0.6, y)], w * 0.6, W4["s"], alpha=0.9)


def w4_tiles():
    rng = random.Random(41)
    t = Tile(w4_base)
    a = t.a
    w4_asphalt(a, rng, [(9, 15, 8, 6), (24, 4, 5, 3), (26, 20, 5, 6)], [(14, 27, 9, 4), (14, 3, 6, 3)])
    w4_sheen(a, [(11, 20.5, 4), (16, 23.6, 5), (24.6, 26.6, 5), (21.6, 18.6, 3)])
    soft(a, 21.5, 12.5, 2.5, 0.6, W4["r"], 0.8)
    soft(a, 14, 18.5, 1.2, 0.4, W4["r"], 0.8)
    t.finish("w4/ground_a")

    t = Tile(w4_base)
    a = t.a
    w4_asphalt(a, rng, [(6, 8, 6, 4), (27, 13, 4, 5)], [(14, 14, 9, 5), (7, 26, 6, 5), (26, 5, 4, 3)])
    # crack network
    crack(a, [(3.0, 18.4), (7, 19.4), (10, 20.6), (13, 19.4), (16, 20.6), (19, 19.2), (22, 20.8), (26.5, 19.6), (28.5, 18.6)], 0.5, W4["k"])
    crack(a, [(10, 20.6), (9.0, 22.6), (7.4, 23.4)], 0.35, W4["k"])
    crack(a, [(19, 19.2), (20.4, 21.6), (21.0, 23.6)], 0.35, W4["k"])
    crack(a, [(25, 20.2), (26, 21.8)], 0.3, W4["k"])
    a.stroke([(3.2, 19.0), (7, 20.0), (10, 21.2)], 0.25, W4["H"], alpha=0.5)
    w4_sheen(a, [(3, 17.4, 4), (7.6, 17.4, 8), (21.6, 22.6, 3), (14, 27.6, 7), (22, 8.6, 3)])
    soft(a, 13, 9.6, 2.4, 0.6, W4["r"], 0.8)
    soft(a, 26.5, 4.5, 1.4, 0.5, W4["r"], 0.8)
    t.finish("w4/ground_b")

    # decal: puddle reflecting the red moon
    t = Tile(w4_base)
    a = t.a
    w4_asphalt(a, rng, [(8, 5, 6, 4), (6, 27, 5, 4)], [(26, 26, 5, 4)])
    pud = [(5.6, 13.8), (9, 11.6), (14, 12.2), (18.6, 10.8), (24.6, 12.0), (26.8, 15.4), (25.4, 19.0), (21.6, 21.4), (15.0, 21.0), (10.6, 22.0), (6.6, 20.4), (5.0, 17.0)]
    a.blob(pud, "#262e3c")
    inner = [(x + (16 - x) * 0.07, y + (16.4 - y) * 0.1) for x, y in pud]
    a.blob(inner, "#12151c")
    a.blob([(6.8, 21.6), (8.6, 21.2), (9.4, 22.8), (7.4, 23.4)], "#12151c")           # small satellite puddle
    soft(a, 11, 15, 5, 2.5, "#1a2030", 0.8)                                          # sky reflection
    a.stroke([(8.6, 14.6), (10.6, 13.8), (12.4, 14.2)], 0.4, "#3a4458", alpha=0.6)
    a.stroke([(21.6, 19.2), (23.8, 18.2)], 0.4, "#3a4458", alpha=0.6)
    # red moon, soft with a vertical shimmer
    soft(a, 18.4, 15.4, 4.4, 3.4, "#5a1620", 0.85)
    a.ellipse(18.4, 15.2, 2.1, 1.8, "#8e222e")
    a.ellipse(18.4, 15.2, 1.45, 1.25, "#c8323c")
    a.ellipse(18.0, 14.8, 0.45, 0.4, "#ff9a9e", alpha=0.85)
    for (y, l, al) in ((17.8, 2.4, 0.6), (18.9, 1.8, 0.45), (19.9, 1.2, 0.35)):
        a.stroke([(18.4 - l, y), (18.4 + l, y)], 0.35, "#a02a36", alpha=al)
    a.stroke([(6.4, 13.6), (9, 12.0), (14, 12.6), (18.6, 11.2), (24.2, 12.4)], 0.3, "#3a4458", alpha=0.45)   # wet rim
    t.finish("w4/decal")

    # path: concrete sidewalk slabs between kerbs
    a = Art(32, 32, tile=True)
    rng = random.Random(44)
    a.rect(0, 0, 32, 32, "#1c1f25")
    a.rect(0, 0, 1.0, 32, "#22252c")
    a.rect(1.0, 0, 2.2, 32, "#3a3f4a")
    a.rect(29.8, 0, 31.0, 32, "#3a3f4a")
    a.rect(31.0, 0, 32, 32, "#22252c")
    a.stroke([(1.4, -1), (1.4, 33)], 0.35, "#4a505c", alpha=0.7, smooth=False)
    a.stroke([(30.2, -1), (30.2, 33)], 0.35, "#4a505c", alpha=0.7, smooth=False)
    for (x0, x1) in ((2.9, 16.0), (16.9, 29.1)):
        for y0 in (0, 16):
            a.rect(x0, y0 + 0.45, x1, y0 + 15.55, "#272a31", radius=0.4)
            a.rect(x0, y0 + 0.45, x1 - 0.4, y0 + 15.15, "#2b2f37", radius=0.4)
            a.stroke([(x0 + 0.6, y0 + 0.75), (x1 - 0.9, y0 + 0.75)], 0.35, "#30343d", alpha=0.9, smooth=False)
    soft(a, 9, 7, 5, 4, "#30343d", 0.5)
    soft(a, 23, 24, 5, 5, "#30343d", 0.4)
    specks(a, rng, 30, (3.5, 1, 28.5, 31), ["#30343d", "#272a31", "#22252c"], rr=(0.15, 0.3), alpha=(0.4, 0.8))
    crack(a, [(6, 2.4), (7.4, 5), (8.6, 7.4), (10.6, 9.2)], 0.4, "#1f2228")
    crack(a, [(19.8, 18.4), (19.2, 21), (18.6, 23.4), (20.4, 25.4), (21.0, 26.6)], 0.4, "#1f2228")
    crack(a, [(11.6, 21.4), (12.4, 23.6), (12.0, 25.0)], 0.35, "#1f2228")
    soft(a, 3.8, 14.4, 1.0, 1.6, "#2a3a30", 0.9)          # moss in the joints
    soft(a, 28.4, 30.4, 0.9, 1.8, "#2a3a30", 0.9)
    a.stroke([(24, 6.6), (26.6, 6.6)], 0.5, "#343c4c", alpha=0.8)  # wet sheen
    a.stroke([(5.6, 9.6), (7.4, 9.6)], 0.5, "#343c4c", alpha=0.8)
    a.stroke([(13, 26.6), (14.6, 26.6)], 0.5, "#343c4c", alpha=0.8)
    a.save("w4/path")


def w4_props():
    # prop_a: sodium street lamp
    a = Art(16, 32)
    a.blob([(4.2, 27.0), (11.0, 27.0), (12.0, 31.8), (3.2, 31.8)], ("lin", "#3c424e", "#272b33", 90), smooth=False)
    a.rect(4.2, 27, 11.0, 28.0, "#59606e", radius=0.4)
    a.rect(6.0, 6.0, 9.0, 27.2, ("lin", "#59606e", "#272b33", 0))
    a.rect(5.0, 18.0, 10.0, 20.0, ("lin", "#3c424e", "#272b33", 90), radius=0.4)
    a.rect(6.0, 12.0, 9.0, 15.0, "#5a3a1a")
    a.ellipse(7.2, 13.4, 0.8, 0.8, "#7a4e24")
    a.stroke([(7.5, 8.0), (7.4, 4.4), (9.0, 2.0), (11.0, 1.5)], 2.1, "#3c424e")
    a.stroke([(6.9, 8.0), (6.8, 4.6), (8.4, 2.4)], 0.5, "#59606e", alpha=0.8)
    a.blob([(9.2, 0.8), (15.8, 0.8), (16.0, 2.8), (9.4, 3.0)], ("lin", "#59606e", "#272b33", 90), smooth=False)
    a.rect(10.2, 3.0, 16.0, 5.6, "#e07a28", radius=0.6)
    a.rect(10.8, 3.6, 15.4, 5.0, "#ffb347", radius=0.5)
    a.rect(11.6, 3.8, 14.4, 4.6, "#fff0b0", radius=0.4)
    a.shade_side("#000000", 0.22, side="right", amount=0.25)
    a.outline()
    halo(a, 12.5, 4.5, 3.5, "#ffb347", 0.55, over=0.1)
    a.save("w4/prop_a")

    # prop_b: overflowing dumpster
    a = Art(32, 32)
    a.ellipse(8.5, 30.0, 1.8, 1.4, "#2a2a2e")
    a.ellipse(23.5, 30.0, 1.8, 1.4, "#2a2a2e")
    # side trash bags
    a.blob([(0.8, 26.4), (4, 24.6), (6.6, 25.6), (6.6, 28.6), (1.2, 28.8)], ("lin", "#363c48", "#262a33", 90))
    a.blob([(25.6, 24.4), (28.2, 23.0), (31.4, 24.8), (31.6, 28.6), (26.0, 28.8)], ("lin", "#363c48", "#262a33", 90))
    a.stroke([(28, 23.2), (28.5, 22.4)], 0.6, "#262a33")
    # body
    a.blob([(3.0, 15.0), (29.0, 15.0), (28.0, 28.8), (4.0, 28.8)], ("lin", "#447058", "#1f3529", 90), smooth=False)
    a.rect(3.0, 15.0, 29.0, 16.0, "#1f3529")
    for x in (9.6, 16.6, 23.6):
        a.rect(x - 0.4, 16.0, x + 0.4, 25.6, "#1f3529", alpha=0.9)
        a.rect(x + 0.4, 16.0, x + 0.9, 25.6, "#447058", alpha=0.5)
    a.rect(4.6, 25.6, 27.4, 28.8, "#1f3529")
    for x in (10.6, 12.0, 13.4, 14.8):
        a.rect(x, 20.0, x + 0.8, 22.0, "#a89a3a", radius=0.2)
    for p in ([(6.6, 18.2), (7.6, 20.0)], [(20.6, 22.4), (21.4, 23.8)], [(25.4, 19.4), (25.8, 20.6)], [(13.4, 24.2), (14, 25.2)]):
        a.stroke(p, 0.6, "#6a3a24", alpha=0.85)
    a.shade_side("#000000", 0.2, side="right", amount=0.3)
    # lid edge
    a.rect(2.2, 13.2, 29.8, 15.2, ("lin", "#447058", "#2f5040", 90), radius=0.6)
    a.stroke([(3, 13.5), (29, 13.5)], 0.35, "#6a9a7c", alpha=0.8)
    # bags piled on top
    for (cx, cy, rx, ry) in ((7, 11.0, 4.6, 3.4), (21, 10.0, 5.2, 4.0), (14, 8.6, 5.0, 4.2), (26.2, 11.6, 3.0, 2.4), (13.5, 5.6, 2.6, 1.8)):
        a.ellipse(cx, cy, rx, ry, ("lin", "#363c48", "#20232b", 70))
        a.stroke([(cx - rx * 0.6, cy - ry * 0.45), (cx - rx * 0.1, cy - ry * 0.8)], 0.5, "#566078", alpha=0.85)
        a.ellipse(cx - rx * 0.35, cy - ry * 0.55, 0.35, 0.3, "#7a92b0", alpha=0.9)
    a.blob([(18, 9), (23, 8.4), (24.4, 11), (19.2, 12)], "#6a5a44", alpha=0.9)      # cardboard
    a.stroke([(13.2, 4.2), (13.4, 2.8), (12.6, 2.0)], 0.6, "#262a33")
    a.outline()
    a.save("w4/prop_b")

    # prop_c: chain-link fence with KEEP OUT sign
    a = Art(48, 32)
    for x in (0.0, 21.6, 43.6):
        a.rect(x + 0.4, 29.6, x + 4.0, 32, "#2a2e36", radius=0.3)
    for x in (1.0, 23.0, 45.0):
        a.rect(x, 1.4, x + 2.0, 30.0, ("lin", "#5a6272", "#383e4a", 0), radius=0.4)
        a.ellipse(x + 1.0, 1.6, 1.3, 0.8, "#5a6272")
    a.rect(1.0, 3.0, 47.0, 5.0, ("lin", "#5a6272", "#383e4a", 90), radius=0.5)
    # sign
    a.rect(8.6, 8.6, 21.4, 18.4, "#d8d4c8", radius=0.6)
    a.rect(8.6, 8.6, 21.4, 12.0, "#a8262c", radius=0.6)
    a.rect(8.6, 11.0, 21.4, 12.0, "#a8262c")
    for x in (10.0, 12.8, 15.6, 18.4):
        a.rect(x, 9.6, x + 1.8, 10.6, "#d8d4c8", radius=0.3)
    a.rect(9.8, 13.4, 14.8, 14.2, "#6e1a20", radius=0.3)
    a.rect(15.8, 13.4, 19.8, 14.2, "#6e1a20", radius=0.3)
    a.rect(9.8, 15.4, 18.8, 16.2, "#6e1a20", radius=0.3)
    a.rect(8.6, 17.2, 21.4, 18.4, "#9a968c", radius=0.4)
    a.blob([(18.6, 15.4), (21.4, 14.6), (21.4, 17.4), (19.6, 17.6)], "#7a6248", alpha=0.9)
    a.outline(width=0.75)
    # chain-link mesh, behind the posts and sign, not outlined
    m = Art(48, 32)
    for x0, x1 in ((3.0, 23.0), (25.0, 45.0)):
        for k in range(-10, 14):
            xs = x0 + k * 2.6
            m.stroke([(xs, 5.0), (xs + 24.0, 29.0)], 0.32, "#4a5262", smooth=False)
            m.stroke([(xs, 29.0), (xs + 24.0, 5.0)], 0.32, "#353b47", smooth=False)
        # clip to the panel
        cm = Image.new("L", (m.W, m.H), 0)
        ImageDraw.Draw(cm).rectangle([x0 * SS, 5.0 * SS, x1 * SS, 29.6 * SS], fill=255)
        if x0 > 20:
            ImageDraw.Draw(cm).ellipse([30.4 * SS, 20.2 * SS, 38.8 * SS, 31 * SS], fill=0)   # torn hole
            ImageDraw.Draw(cm).rectangle([31.0 * SS, 25 * SS, 38.2 * SS, 30 * SS], fill=0)
        if x0 < 20:
            left = cm
        else:
            right = cm
    mask = ImageChops.lighter(left, right)
    m.img.putalpha(ImageChops.multiply(m.img.getchannel("A"), mask))
    m.stroke([(30.6, 23.2), (29.6, 25.4)], 0.35, "#4a5262")   # curled wire at the hole
    m.stroke([(38.6, 23.0), (39.8, 25.6)], 0.35, "#4a5262")
    m.rect(3.0, 28.8, 45.0, 29.5, "#353b47", alpha=0.6)
    # barbed wire on top
    for x in range(4, 46, 4):
        m.stroke([(x - 0.8, 0.6), (x + 0.8, 2.2)], 0.35, "#4a5262", smooth=False)
        m.stroke([(x + 0.8, 0.6), (x - 0.8, 2.2)], 0.35, "#4a5262", smooth=False)
    m.stroke([(1, 1.5), (47, 1.5)], 0.3, "#353b47", smooth=False)
    behind(a, m)
    a.save("w4/prop_c")

    # prop_d: traffic cone and a broken umbrella
    a = Art(24, 24)
    # umbrella
    a.stroke([(16.5, 15.5), (20.4, 10.2), (23.2, 7.6)], 0.8, "#7a7e8a")
    canopy = [(14.2, 15.6), (16.6, 12.8), (19.0, 12.2), (21.6, 13.6), (23.4, 16.6), (23.6, 21.8), (19.6, 22.2), (15.4, 22.2), (12.8, 21.0), (13.4, 18.0)]
    a.blob(canopy, ("lin", "#423f52", "#2c2a36", 60))
    for p in ([(18.8, 12.4), (16.8, 18), (16.0, 22)], [(18.8, 12.4), (19.6, 18), (20.6, 22)], [(18.8, 12.4), (22.6, 16.6), (23.4, 21)]):
        a.stroke(p, 0.4, "#7a7e8a", alpha=0.85)
    a.stroke([(15.4, 15.2), (17.4, 13.2)], 0.45, "#5a5870", alpha=0.8)
    # cone
    a.rect(1.4, 21.0, 12.4, 23.0, ("lin", "#a84418", "#6a2a10", 90), radius=0.5)
    cone = [(6.4, 2.6), (8.6, 2.6), (11.0, 21.2), (4.0, 21.2)]
    a.poly(cone, ("lin", "#ff8a40", "#a84418", 0))
    a.ellipse(7.5, 2.8, 1.1, 0.5, "#e86a24")
    # reflective bands (clipped onto the cone)
    for y0, y1 in ((9.8, 12.0), (15.0, 17.4)):
        a.rect(2.0, y0, 13.0, y1, ("lin", "#f4f0e8", "#a8a49c", 0), clip=True)
    a.stroke([(6.8, 4.0), (5.2, 19.8)], 0.5, "#ffb070", alpha=0.6)
    a.outline()
    a.save("w4/prop_d")


# ───────────────────────── W5: Shatterline ─────────────────────────
W5 = dict(A="#1c1922", B="#211d28", C="#18151d", H="#2a2532", M="#110e15", v="#2c1a3c", V="#4a2468")


def w5_base(a):
    a.rect(0, 0, 32, 32, W5["A"])


def w5_asphalt(a, rng, light, dark, n=24):
    for (x, y, rx, ry) in light:
        soft(a, x, y, rx, ry, W5["B"], 0.75)
    for (x, y, rx, ry) in dark:
        soft(a, x, y, rx, ry, W5["C"], 0.75)
    specks(a, rng, n, (2, 2, 30, 30), [W5["H"], W5["C"], W5["B"]], rr=(0.16, 0.3), alpha=(0.35, 0.7))


def vein(a, pts, w=0.55, bright=False):
    """A curse crack: dark split with a faint violet glow (brighter when `bright`)."""
    a.stroke(pts, w * 5.0, W5["V"], alpha=0.06 if not bright else 0.16)
    a.stroke(pts, w * 2.8, W5["V"], alpha=0.08 if not bright else 0.2)
    a.stroke(pts, w, W5["M"])
    a.stroke(pts, w * 0.4, "#5a2c80" if not bright else "#9a4ae0", alpha=0.85)


def w5_tiles():
    rng = random.Random(51)
    t = Tile(w5_base)
    a = t.a
    w5_asphalt(a, rng, [(22, 6, 7, 5), (6, 24, 6, 5)], [(12, 14, 6, 4), (26, 26, 5, 4)])
    vein(a, [(7.6, 3.0), (7.2, 6.4), (8.4, 8.4), (7.0, 11.0), (6.2, 13.4), (6.4, 16.4)])
    vein(a, [(7.0, 11.0), (4.6, 12.6)], 0.35)
    vein(a, [(22.0, 18.2), (22.4, 20.6), (21.2, 22.0), (20.6, 24.4), (20.2, 27.6)])
    vein(a, [(21.2, 22.0), (24.4, 21.6)], 0.35)
    t.finish("w5/ground_a")

    t = Tile(w5_base)
    a = t.a
    w5_asphalt(a, rng, [(8, 6, 6, 4), (24, 24, 5, 4)], [], n=14)
    # dark patched rectangle (fresh asphalt patch)
    patch = [(12.4, 9.4), (19, 9.0), (24.8, 9.6), (25.4, 14), (24.6, 21.0), (18, 21.8), (13.0, 21.2), (12.0, 15)]
    a.blob(patch, W5["C"], alpha=0.5)
    a.stroke(patch + [patch[0]], 0.35, W5["H"], alpha=0.25)
    specks(a, rng, 10, (13.5, 10.5, 24, 20.5), [W5["A"], W5["M"]], rr=(0.15, 0.25))
    vein(a, [(3.6, 9.6), (3.4, 12.0), (4.2, 14.2), (4.4, 16.6)])
    vein(a, [(28.8, 21.6), (29.2, 24.2), (28.8, 26.6), (29.4, 29.0)])
    t.finish("w5/ground_b")

    # decal: crosswalk paint fragment split by a glowing curse crack
    t = Tile(w5_base)
    a = t.a
    w5_asphalt(a, rng, [(6, 4, 5, 3), (26, 28, 5, 3)], [(24, 4, 4, 3)], n=14)
    frag = [(9.2, 6.2), (26.6, 6.0), (26.4, 12), (27.0, 18), (26.6, 24.8), (22.6, 25.4), (14.6, 25.2), (6.0, 24.6), (5.4, 19), (5.8, 13), (5.2, 9.4)]
    a.blob(frag, "#35313f", smooth=False)
    for (x, y, rx, ry) in ((8.4, 21, 1.6, 1.0), (22.6, 9.0, 1.2, 0.8), (19, 22.8, 1.8, 0.8), (7.4, 11, 1.2, 0.8)):
        a.ellipse(x, y, rx, ry, "#2e2b37")
    for (x, y) in ((11, 8.5), (20, 14.5), (24, 21), (10, 17)):
        a.ellipse(x, y, 0.4, 0.35, W5["A"])
    pts = [(15.6, 3.2), (15.4, 7.0), (15.8, 10.4), (15.0, 13.4), (14.4, 17.0), (14.6, 20.4), (14.2, 24.0), (14.0, 28.2)]
    a.stroke(pts, 3.0, "#9a4ae0", alpha=0.14)
    a.stroke(pts, 1.8, "#4a2468", alpha=0.7)
    a.stroke(pts, 0.9, W5["M"])
    a.stroke(pts, 0.42, "#9a4ae0")
    a.stroke([(15.4, 11.2), (15.2, 13.0)], 0.3, "#d8a0ff")
    a.stroke([(14.5, 19.4), (14.4, 21.0)], 0.3, "#d8a0ff")
    br = [(15.0, 13.4), (17.0, 14.4), (19.0, 15.6), (20.6, 17.4)]
    a.stroke(br, 1.4, "#4a2468", alpha=0.7)
    a.stroke(br, 0.55, W5["M"])
    a.stroke(br, 0.25, "#9a4ae0")
    t.finish("w5/decal")

    # path: dimmed zebra stripes running vertically
    a = Art(32, 32, tile=True)
    rng = random.Random(55)
    a.rect(0, 0, 32, 32, W5["A"])
    for (x, y, rx, ry) in ((2, 8, 2.5, 5), (16, 22, 3, 6), (30, 26, 2.5, 4), (16, 4, 3, 4)):
        soft(a, x, y, rx, ry, W5["B"], 0.6)
    for x0 in (4.0, 20.0):
        a.rect(x0, -1, x0 + 8.0, 33, "#34303c")
        a.rect(x0 + 0.4, -1, x0 + 1.0, 33, "#3c3846", alpha=0.6)
        for _ in range(7):
            y = rng.uniform(0, 32)
            a.ellipse(x0 + rng.uniform(1.5, 6.5), y, rng.uniform(0.6, 1.6), rng.uniform(0.4, 1.0), "#2c2933", alpha=0.9)
        for _ in range(4):
            y = rng.uniform(0, 32)
            a.ellipse(x0 + rng.choice((0.1, 7.9)), y, 0.45, rng.uniform(0.6, 1.4), W5["A"], alpha=0.8)   # chipped edges
    specks(a, rng, 18, (0, 0, 32, 32), [W5["H"], W5["C"]], rr=(0.15, 0.28), alpha=(0.3, 0.6))
    vein(a, [(17.8, 20.2), (18.0, 22.6), (17.4, 24.4), (18.0, 26.6), (17.6, 29.2)], 0.45)
    a.save("w5/path")


def w5_props():
    # prop_a: broken traffic signal, red and amber lit, cable dangling
    a = Art(16, 32)
    a.blob([(4.0, 28.0), (12.0, 28.0), (12.8, 31.8), (3.2, 31.8)], ("lin", "#5a5866", "#3c3a46", 90), smooth=False)
    a.rect(4.0, 28.0, 12.0, 29.0, "#7a7888", radius=0.4)
    a.rect(7.0, 10.0, 9.0, 28.2, ("lin", "#7a7888", "#3c3a46", 0))
    a.stroke([(7.4, 12), (5.6, 13.6), (4.4, 16.4), (4.6, 19.0), (4.0, 21.2)], 0.55, "#16141a")
    # signal head (tilted slightly)
    head = [(7.0, 0.4), (13.6, 0.6), (14.6, 1.6), (14.4, 9.2), (15.6, 9.4), (15.4, 11.6), (13.6, 12.6), (8.2, 12.4), (7.6, 11.0), (7.2, 5.0)]
    a.poly(head, ("lin", "#2a2832", "#16141a", 0))
    a.ellipse(11.4, 3.8, 1.7, 1.5, "#a02020")
    a.ellipse(11.4, 3.8, 1.25, 1.1, "#ff3a3a")
    a.ellipse(11.1, 3.5, 0.5, 0.45, "#ffd0c0")
    a.ellipse(11.8, 7.6, 1.7, 1.3, "#8a5a10")
    a.ellipse(11.8, 7.6, 1.3, 1.0, "#ffb030")
    a.ellipse(12.2, 11.2, 1.3, 1.0, "#2a3a2a")
    a.stroke([(9.0, 2.0), (13.6, 2.0)], 0.6, "#26242c")
    a.stroke([(7.8, 1.2), (8.0, 10.6)], 0.4, "#3c3a46", alpha=0.8)
    a.outline()
    # sparking cable end
    halo(a, 4.0, 21.4, 1.6, "#ffb030", 0.6)
    a.ellipse(4.0, 21.6, 0.45, 0.45, "#ffe08a")
    halo(a, 11.4, 3.8, 5, "#ff3a3a", 0.55, over=0.08)
    a.save("w5/prop_a")

    # prop_b: wrecked compact car (facing right)
    a = Art(32, 32)
    for wx in (7.6, 23.2):
        a.ellipse(wx, 27.6, 3.8, 3.6, "#151318")
    a.rect(3.0, 29.6, 29.0, 31.6, "#151318", radius=0.8)
    body = [(1.0, 16.4), (8.0, 15.0), (24.0, 15.0), (30.6, 17.0), (31.4, 22.0), (31.0, 25.6), (1.0, 25.6), (0.6, 20)]
    a.blob(body, ("lin", "#9a3a44", "#521a24", 90), smooth=False)
    cab = [(8.6, 15.2), (11.4, 9.0), (21.4, 8.8), (27.2, 15.2)]
    a.poly(cab, ("lin", "#7a2a34", "#521a24", 90))
    a.poly([(10.2, 15.0), (12.4, 10.2), (16.2, 10.2), (16.2, 15.0)], ("lin", "#6a8ab0", "#3a4a6a", 90))
    a.poly([(17.8, 15.0), (17.8, 10.2), (21.0, 10.2), (25.4, 15.0)], ("lin", "#6a8ab0", "#3a4a6a", 90))
    a.stroke([(13.2, 13.8), (14.4, 10.8)], 0.5, "#c8dcf0", alpha=0.6)
    a.stroke([(20.0, 10.6), (21.6, 12.6), (23.4, 13.4)], 0.35, "#151318")      # cracked glass
    a.stroke([(21.6, 12.6), (21.2, 14.6)], 0.3, "#151318")
    a.stroke([(1.4, 17.4), (16.6, 17.2)], 0.6, "#9a3a44")
    a.stroke([(2, 16.6), (28, 16.6)], 0.4, "#c05a62", alpha=0.6)
    a.rect(16.7, 15.2, 17.6, 25.0, "#521a24")
    a.rect(1.0, 22.2, 31.0, 25.6, "#521a24")
    a.rect(13.0, 18.1, 15.6, 18.9, "#46444e", radius=0.3)
    a.rect(29.2, 17.8, 31.6, 19.8, "#ffe08a", radius=0.4)
    a.rect(0.4, 17.8, 2.6, 19.8, "#ff5a3a", radius=0.4)
    a.rect(0.0, 22.8, 4.2, 24.6, "#8a8a96", radius=0.5)
    a.rect(27.8, 22.8, 32.0, 24.6, "#8a8a96", radius=0.5)
    a.stroke([(4.2, 20.0), (6.4, 21.4), (8.0, 20.6), (10.0, 22.2)], 0.45, "#a050ff")   # curse scratch
    a.stroke([(5.0, 19.2), (6.0, 20.2)], 0.3, "#d8a0ff")
    for wx in (7.6, 23.2):
        a.ellipse(wx, 27.4, 3.0, 2.9, "#151318")
        a.ellipse(wx, 27.4, 1.5, 1.4, ("lin", "#6a6874", "#2a2830", 90))
    a.outline()
    halo(a, 31.2, 18.8, 2.2, "#ffe08a", 0.5)
    halo(a, 7.0, 20.8, 2.4, "#a050ff", 0.35)
    a.save("w5/prop_b")

    # prop_c: bus shelter wreck on concrete barriers
    a = Art(48, 32)
    # sign post
    a.rect(42.0, 5.0, 44.2, 22.0, ("lin", "#3a3a46", "#24242c", 0))
    a.rect(40.8, 0.6, 47.4, 5.6, "#2a5aa0", radius=0.6)
    a.rect(42.0, 1.8, 46.2, 4.4, "#d8e0f0", radius=0.4)
    a.rect(42.8, 2.5, 45.4, 3.7, "#2a5aa0", radius=0.3)
    # shelter
    a.rect(4.8, 3.6, 39.6, 6.4, ("lin", "#3a3a46", "#24242c", 90), radius=0.5)
    a.rect(3.6, 6.0, 41.4, 18.4, "#3a3a46", radius=0.6)
    a.rect(5.8, 7.8, 39.2, 16.8, ("lin", "#6a8ab0", "#3a5a7a", 100))
    a.blob([(6.0, 8), (19, 8), (8, 16.6), (6.0, 16.6)], "#8ab0d0", alpha=0.35, smooth=False)
    for p in ([(23.6, 8.0), (22.6, 10.8), (20.8, 12.8), (17.4, 15.8)], [(22.6, 10.8), (24.6, 13.6), (25.2, 16.6)], [(26.4, 8.2), (25.2, 11.0), (22.6, 10.8)]):
        a.stroke(p, 0.4, "#24242c", smooth=False)
    for (x, y) in ((10, 9.4), (15, 9.6), (32, 9.4), (35.6, 9.6), (29, 13.8), (36, 13.6)):
        a.ellipse(x, y, 0.4, 0.4, "#8ab0d0", alpha=0.9)
    a.rect(3.6, 17.0, 41.4, 18.4, "#86848e", radius=0.4)
    # barriers (jersey shape)
    def barrier(x0, x1, ytop):
        mid = (x0 + x1) / 2
        a.poly([(x0 + 1.2, ytop), (x1 - 1.2, ytop), (x1 - 0.8, 27.0), (x1 + 0.6, 29.0), (x1 + 0.6, 31.8), (x0 - 0.6, 31.8), (x0 - 0.6, 29.0), (x0 + 0.8, 27.0)],
               ("lin", "#6a6872", "#4a4852", 90))
        a.rect(x0 - 0.6, 28.0, x1 + 0.6, 31.8, "#4a4852")
        a.stroke([(x0 - 0.2, 28.4), (x1 + 0.2, 28.4)], 0.4, "#86848e", alpha=0.8)
        a.rect(x0 + 1.0, ytop + 1.2, x1 - 1.0, ytop + 3.2, "#c8a030")
        for k in range(int(x0) + 1, int(x1), 3):
            a.poly([(k + 0.4, ytop + 1.2), (k + 1.6, ytop + 1.2), (k + 0.8, ytop + 3.2), (k - 0.4, ytop + 3.2)], "#2a2420", clip=True)
        a.rect(x0 + 1.2, ytop + 0.1, x1 - 1.2, ytop + 0.6, "#86848e", alpha=0.7)
        return mid
    barrier(3.2, 13.8, 19.0)
    barrier(33.2, 45.0, 19.0)
    barrier(18.0, 29.4, 18.4)
    pts = [(22.6, 18.6), (23.2, 21.6), (23.8, 25.0), (24.4, 28.4), (25.0, 31.4)]
    a.stroke(pts, 1.0, "#24242c")
    a.stroke(pts, 0.4, "#a050ff")
    a.outline()
    halo(a, 24, 25, 3, "#a050ff", 0.35)
    a.save("w5/prop_c")

    # prop_d: rubble heap with a cursed talisman
    a = Art(24, 24)
    a.stroke([(3.0, 8.8), (5.2, 12.4), (7.2, 15.0)], 0.7, "#5a3a3a")      # rebar
    a.stroke([(19.6, 8.6), (18.6, 11.0), (16.8, 14.2)], 0.7, "#5a3a3a")
    heap = [(1.4, 21.6), (3.2, 16.6), (6.4, 13.8), (11, 13.0), (15.6, 13.4), (19.6, 14.0), (23.2, 15.4), (23.4, 23.6), (1.6, 23.6)]
    a.blob(heap, ("lin", "#4a4856", "#2e2c38", 90), smooth=False)
    for (pts, col) in (([(4, 16.4), (8, 15.2), (10, 17.6), (6, 19.4), (3.6, 18.6)], "#5a5864"),
                       ([(11, 15.0), (16.6, 15.0), (17.2, 19.0), (11.4, 19.2)], "#5a3a3a"),
                       ([(18.0, 14.6), (23.0, 15.4), (23.0, 19.2), (18.6, 19.4)], "#5a5864"),
                       ([(4.4, 19.6), (8.0, 19.6), (8.2, 21.6), (4.0, 21.8)], "#5a3a3a"),
                       ([(16.4, 18.4), (21.0, 18.4), (21.2, 21.6), (17.0, 21.8)], "#8a4a3a"),
                       ([(9.6, 19.8), (15.0, 20.0), (15.2, 23.0), (9.4, 23.0)], "#5a5864")):
        a.blob(pts, col, smooth=False)
        x0 = min(p[0] for p in pts); y0 = min(p[1] for p in pts)
        x1 = max(p[0] for p in pts)
        a.stroke([(x0 + 0.6, y0 + 0.4), (x1 - 0.8, y0 + 0.4)], 0.4, "#7a7884", alpha=0.7)
    # talisman stuck in the heap
    tal = [(10.2, 1.4), (15.0, 1.0), (15.6, 6.0), (16.8, 13.0), (12.4, 13.4), (11.4, 7.0)]
    a.poly(tal, ("lin", "#e8dcb8", "#b0a080", 0))
    a.stroke([(12.4, 3.0), (14.2, 3.0)], 0.4, "#2a1a3a")
    a.stroke([(13.2, 2.0), (13.6, 6.0), (14.0, 9.4)], 0.45, "#2a1a3a")
    a.stroke([(12.8, 6.6), (15.0, 6.4)], 0.4, "#2a1a3a")
    a.stroke([(13.0, 10.6), (15.4, 10.4)], 0.4, "#2a1a3a")
    a.ellipse(13.7, 4.6, 0.55, 0.55, "#c070ff")
    a.ellipse(14.3, 8.4, 0.5, 0.5, "#c070ff")
    a.ellipse(14.7, 12.0, 0.5, 0.45, "#c070ff")
    a.outline()
    for (x, y, r) in ((9.6, 3.8, 0.45), (17.6, 3.4, 0.4), (8.6, 7.8, 0.5), (18.4, 9.6, 0.35)):
        halo(a, x, y, 1.3, "#7a30c0", 0.6)
        a.ellipse(x, y, r, r, "#c070ff")
    halo(a, 13.8, 7.5, 6, "#7a30c0", 0.35)
    a.save("w5/prop_d")


if __name__ == "__main__":
    w3_tiles(); w3_props()
    w4_tiles(); w4_props()
    w5_tiles(); w5_props()
    print("ok")
