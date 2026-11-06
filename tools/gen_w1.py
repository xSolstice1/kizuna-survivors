"""Generates World 1 (Hidden Shrine Valley) tiles, props, enemies and relic icon as .px sources.

    python3 tools/gen_w1.py

Writes art/src/w1/*.px and art/src/icons/relic_w1.px. Deterministic (seeded).

A secluded mountain shrine valley at night under a pale moon: mossy flagstones, fallen
sakura petals, stone lanterns, vermilion torii. Enemies are spirits and familiars.
"""
import math
import pathlib
import random
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import gen_stage  # noqa: E402
from gen_stage import Canvas  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "art/src/w1"
gen_stage.OUT = OUT


class C(Canvas):
    """Canvas with a few extra primitives."""

    def save(self, name, comment, outline=False):
        super().save(name, comment, outline)
        f = OUT / f"{name}.px"
        f.write_text(f.read_text().replace("tools/gen_stage.py", "tools/gen_w1.py", 1))

    def get(self, x, y):
        return self.g[y][x] if 0 <= x < self.w and 0 <= y < self.h else "."

    def poly(self, pts, c):
        for y in range(self.h):
            for x in range(self.w):
                px, py, inside = x + 0.5, y + 0.5, False
                for i in range(len(pts)):
                    (x1, y1), (x2, y2) = pts[i], pts[i - 1]
                    if (y1 > py) != (y2 > py) and px < (x2 - x1) * (py - y1) / (y2 - y1) + x1:
                        inside = not inside
                if inside:
                    self.g[y][x] = c

    def line(self, x0, y0, x1, y1, c, w=1):
        steps = int(max(abs(x1 - x0), abs(y1 - y0)) * 2) + 1
        for i in range(steps + 1):
            t = i / steps
            x, y = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
            if w == 1:
                self.px(round(x), round(y), c)
            else:
                self.disc(x, y, w / 2, c)

    def disc(self, cx, cy, r, c):
        for y in range(int(cy - r) - 1, int(cy + r) + 2):
            for x in range(int(cx - r) - 1, int(cx + r) + 2):
                if (x + 0.5 - cx) ** 2 + (y + 0.5 - cy) ** 2 <= r * r:
                    self.px(x, y, c)

    def recolor(self, frm, to, cond):
        for y in range(self.h):
            for x in range(self.w):
                if self.g[y][x] in frm and cond(x, y):
                    self.g[y][x] = to

    def shade_edge(self, mat, shade, dx, dy):
        """Pixels of `mat` whose neighbour at (dx,dy) is not `mat`-family become `shade`."""
        src = [r[:] for r in self.g]
        for y in range(self.h):
            for x in range(self.w):
                if src[y][x] == mat:
                    nx, ny = x + dx, y + dy
                    n = src[ny][nx] if 0 <= nx < self.w and 0 <= ny < self.h else "."
                    if n == ".":
                        self.g[y][x] = shade


# ───────────────────────────── ground ─────────────────────────────

GROUND = {
    "A": "#18232e",  # packed earth, base
    "B": "#1b2a33",  # earth, lighter
    "D": "#141c27",  # earth, dark
    "M": "#1d3133",  # moss
    "m": "#223a38",  # moss highlight
    "P": "#7a4a66",  # fallen petal (dim)
    "p": "#a06684",  # petal highlight
}


def periodic_noise(rng, period, size=32):
    n = size // period
    grid = [[rng.random() for _ in range(n)] for _ in range(n)]

    def f(x, y):
        gx, gy = x / period, y / period
        x0, y0 = int(gx) % n, int(gy) % n
        x1, y1 = (x0 + 1) % n, (y0 + 1) % n
        tx, ty = gx - int(gx), gy - int(gy)
        tx, ty = tx * tx * (3 - 2 * tx), ty * ty * (3 - 2 * ty)
        a = grid[y0][x0] * (1 - tx) + grid[y0][x1] * tx
        b = grid[y1][x0] * (1 - tx) + grid[y1][x1] * tx
        return a * (1 - ty) + b * ty

    return f


def edge_weight(x, y, band=5):
    """0 at the tile border, 1 in the interior: interior detail fades out so all tiles share edges."""
    d = min(x, y, 31 - x, 31 - y)
    return max(0.0, min(1.0, (d - 1) / band))


SHARED = None  # shared border field so every ground tile has identical edges


def base_field(seed):
    rng = random.Random(seed)
    n8, n4, n16 = periodic_noise(rng, 8), periodic_noise(rng, 4), periodic_noise(rng, 16)
    return [[0.5 * n16(x, y) + 0.32 * n8(x, y) + 0.18 * n4(x, y) for x in range(32)] for y in range(32)]


def earth(seed_interior, moss_bias=0.0):
    """Earth/moss tile whose 1-px border (and fading band) comes from the shared field."""
    inner = base_field(seed_interior)
    c = C(32, 32, dict(GROUND))
    drng = random.Random(seed_interior * 31 + 5)
    for y in range(32):
        for x in range(32):
            w = edge_weight(x, y)
            v = SHARED[y][x] * (1 - w) + inner[y][x] * w + moss_bias * w
            # tiny ordered dither so bands don't look like flat blobs
            v += ((x * 7 + y * 13) % 5 - 2) * 0.012
            if v < 0.36:
                col = "D"
            elif v < 0.50:
                col = "A"
            elif v < 0.60:
                col = "B"
            elif v < 0.70:
                col = "M"
            else:
                col = "m"
            c.g[y][x] = col
    return c, drng


def petals(c, rng, n, margin=3):
    for _ in range(n):
        x, y = rng.randint(margin, 31 - margin - 1), rng.randint(margin, 31 - margin)
        c.px(x, y, "P")
        if rng.random() < 0.6:
            c.px(x + 1, y, "P")
        if rng.random() < 0.5:
            c.px(x + rng.choice((0, 1)), y - 1 if rng.random() < 0.5 else y, "p")


def flagstone(c, x0, y0, x1, y1, rng):
    """A sunk, rounded stone slab in the interior."""
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            corner = (x in (x0, x1)) and (y in (y0, y1))
            if corner:
                continue
            c.px(x, y, rng.choice("SSST"))
    for x in range(x0 + 1, x1):
        c.px(x, y1, "s")
    for y in range(y0 + 1, y1):
        c.px(x1, y, "s")
    c.px(x0 + 1, y0 + 1, "H")
    for _ in range(2):  # moss creeping over the slab
        mx, my = rng.randint(x0, x1), rng.choice((y0, y1))
        c.px(mx, my, "M")
        c.px(mx + 1, my, "M")


STONE_G = {"S": "#262f40", "T": "#2a3446", "s": "#1a2130", "H": "#34405a"}


def ground_tiles():
    global SHARED
    SHARED = base_field(101)

    c, r = earth(11)
    petals(c, r, 9)
    c.save("ground_a", "W1 ground A: packed earth and moss with fallen sakura petals")

    c, r = earth(23, moss_bias=0.04)
    c.pal.update(STONE_G)
    flagstone(c, 6, 6, 14, 12, r)
    flagstone(c, 17, 8, 25, 15, r)
    flagstone(c, 9, 17, 18, 24, r)
    flagstone(c, 21, 19, 26, 24, r)
    petals(c, r, 6)
    c.save("ground_b", "W1 ground B: old mossy flagstones sunk in earth")

    # Decal: a ring of small sacred stones with a twist of rope and a heap of petals.
    c, r = earth(37)
    c.pal.update(STONE_G)
    c.pal.update({"R": "#6e6250", "r": "#4c4438", "W": "#8a8a90"})
    cx, cy = 16, 16
    for k in range(48):
        a = k / 48 * 2 * math.pi
        x, y = round(cx + 9.5 * math.cos(a) - 0.5), round(cy + 9.5 * math.sin(a) - 0.5)
        c.px(x, y, "R" if k % 3 else "r")
    for k in range(8):
        a = k / 8 * 2 * math.pi + 0.2
        x, y = round(cx + 9.5 * math.cos(a) - 0.5), round(cy + 9.5 * math.sin(a) - 0.5)
        c.rect(x - 1, y - 1, x + 1, y, "S")
        c.rect(x - 1, y + 1, x + 1, y + 1, "s")
        c.px(x - 1, y - 1, "H")
    # paper streamer scraps on the rope
    for a in (0.9, 2.5, 4.1, 5.6):
        x, y = round(cx + 9.5 * math.cos(a)), round(cy + 9.5 * math.sin(a))
        c.px(x, y + 1, "W")
        c.px(x, y + 2, "W")
    # petals collected in the middle
    for _ in range(18):
        a, d = r.random() * 6.283, r.random() ** 0.7 * 5
        x, y = round(cx + d * math.cos(a)), round(cy + d * math.sin(a))
        c.px(x, y, r.choice("PPp"))
    c.save("decal", "W1 decal: ring of sacred stones and rope with gathered petals")

    # Path: worn stone stairway column; edges shared with ground tiles.
    c, r = earth(53)
    c.pal.update({"S": "#28303f", "T": "#2c3546", "s": "#191f2c", "H": "#36415a", "E": "#202734"})
    x0, x1 = 5, 26
    for step in range(2):
        y0 = step * 16
        for y in range(y0, y0 + 16):
            for x in range(x0, x1 + 1):
                wob = (x == x0 and r.random() < 0.4) or (x == x1 and r.random() < 0.4)
                if wob:
                    continue
                c.px(x, y, r.choice("SSSTT"))
        # riser shadow at the bottom of each step, lip highlight at the top
        c.rect(x0 + 1, y0 + 13, x1 - 1, y0 + 15, "E")
        c.rect(x0 + 1, y0 + 15, x1 - 1, y0 + 15, "s")
        c.rect(x0 + 1, y0, x1 - 1, y0, "H")
        # slab joints (offset each step)
        jx = 13 if step == 0 else 18
        c.rect(jx, y0 + 1, jx, y0 + 12, "s")
        # wear: moss in cracks, a couple of petals
        for _ in range(5):
            c.px(r.randint(x0 + 1, x1 - 1), r.randint(y0 + 2, y0 + 12), "M")
        for _ in range(3):
            px, py = r.randint(x0 + 2, x1 - 2), r.randint(y0 + 2, y0 + 11)
            c.px(px, py, "P")
            c.px(px + 1, py, "p")
    # moss on the outer stair edges
    for y in range(32):
        if r.random() < 0.35:
            c.px(x0, y, "M")
        if r.random() < 0.35:
            c.px(x1, y, "M")
    c.save("path", "W1 path: worn stone stairway approach")


# ───────────────────────────── props ─────────────────────────────

STONE = {"S": "#7d8698", "s": "#5a6276", "d": "#3c4356", "H": "#a3acbc", "M": "#4c7a4c", "m": "#6c9c5a"}


def moss_spots(c, rng, n, mats="Ss"):
    for _ in range(n):
        x, y = rng.randint(0, c.w - 1), rng.randint(0, c.h - 1)
        if c.get(x, y) in mats:
            c.px(x, y, "M")
            if c.get(x + 1, y) in mats:
                c.px(x + 1, y, rng.choice("Mm"))


def props():
    rng = random.Random(77)

    # prop_a: stone toro lantern, warm glowing window.
    c = C(16, 32, {**STONE, "Y": "#ffe9a8", "O": "#ffb44a", "o": "#c46a2a"})
    c.rect(4, 28, 11, 31, "s")         # base plinth
    c.rect(3, 30, 12, 31, "d")
    c.rect(4, 28, 11, 28, "S")
    c.rect(6, 19, 9, 27, "S")          # pole
    c.rect(8, 19, 9, 27, "s")
    c.rect(5, 22, 10, 22, "s")         # pole ring
    c.rect(4, 17, 11, 18, "S")         # platform
    c.rect(4, 18, 11, 18, "s")
    c.rect(4, 11, 11, 16, "s")         # firebox
    c.rect(5, 12, 10, 15, "O")
    c.rect(6, 12, 9, 14, "Y")
    c.rect(7, 12, 8, 15, "o")          # window mullion
    c.rect(5, 12, 10, 12, "o")
    c.poly([(1, 11), (15, 11), (12, 7), (4, 7)], "S")   # kasa roof
    c.rect(1, 10, 14, 10, "s")
    c.px(1, 9, "H"), c.px(14, 9, "H")
    c.rect(5, 7, 10, 7, "H")
    c.rect(6, 5, 9, 6, "S")
    c.disc(8, 3.5, 2, "S")             # hoju finial
    c.px(7, 2, "H")
    moss_spots(c, rng, 14)
    c.save("prop_a", "W1 prop A: stone toro lantern (glowing)", outline=True)

    # prop_b: small wooden hokora shrine on a stone base.
    c = C(32, 32, {**STONE, "W": "#6a4a38", "w": "#4a3226", "R": "#c8413a", "r": "#8a2a2a",
                   "T": "#2c2a3e", "t": "#3e3c56", "P": "#eeeadc", "Y": "#ffd27a"})
    c.rect(4, 26, 27, 31, "S")          # stone base
    c.rect(4, 29, 27, 31, "s")
    c.rect(4, 26, 27, 26, "H")
    c.rect(3, 31, 28, 31, "d")
    c.rect(8, 13, 23, 25, "W")          # wooden body
    c.rect(8, 13, 9, 25, "w")
    c.rect(22, 13, 23, 25, "w")
    c.rect(11, 15, 20, 25, "w")         # doors
    c.rect(15, 15, 16, 25, "r")
    c.rect(12, 16, 14, 24, "R")
    c.rect(17, 16, 19, 24, "R")
    c.rect(12, 19, 14, 19, "r"), c.rect(17, 19, 19, 19, "r")
    c.px(13, 21, "Y"), c.px(18, 21, "Y")  # candle-glow through slats
    c.poly([(2, 14), (30, 14), (24, 6), (8, 6)], "T")  # roof
    c.rect(2, 12, 29, 13, "t")
    for x in range(4, 29, 3):
        c.rect(x, 7, x, 12, "t")
    c.rect(8, 5, 23, 6, "W")            # ridge
    c.rect(6, 4, 7, 5, "W"), c.rect(24, 4, 25, 5, "W")
    # shimenawa + shide paper streamers under the eave
    c.rect(8, 14, 23, 14, "Y")
    for x in (10, 15, 21):
        c.px(x, 15, "P"), c.px(x + 1, 16, "P"), c.px(x, 17, "P")
    c.rect(13, 24, 18, 25, "P")         # offering dish
    c.px(15, 23, "Y"), c.px(16, 23, "Y")
    moss_spots(c, rng, 10)
    c.recolor("t", "M", lambda x, y: y == 12 and rng.random() < 0.3)
    c.save("prop_b", "W1 prop B: small wooden hokora offering shrine", outline=True)

    # prop_c: vermilion torii fragment with shimenawa rope and shide.
    c = C(48, 32, {**STONE, "R": "#d4433a", "r": "#962c2c", "q": "#6a1e24", "B": "#22202c",
                   "Y": "#c8a860", "y": "#8e7440", "P": "#eeeadc"})
    for px0 in (7, 37):                 # pillars
        c.rect(px0, 8, px0 + 3, 29, "R")
        c.rect(px0 + 3, 8, px0 + 3, 29, "r")
        c.rect(px0 - 1, 28, px0 + 4, 31, "B")   # black kamaki base
        c.rect(px0 - 1, 28, px0 + 4, 28, "d")
    # kasagi top beam (cracked off on the right) + shimaki
    c.poly([(1, 4), (40, 5), (40, 8), (3, 7)], "B")
    c.poly([(0, 2), (4, 4), (2, 5)], "B")
    c.rect(4, 7, 40, 8, "R")
    c.rect(4, 8, 40, 8, "r")
    c.poly([(40, 4), (43, 6), (41, 7), (42, 9), (40, 9)], "r")  # broken end
    # nuki tie beam
    c.rect(4, 13, 44, 15, "R")
    c.rect(4, 15, 44, 15, "r")
    c.rect(22, 9, 25, 12, "R")          # gakuzuka strut
    c.rect(25, 9, 25, 12, "r")
    # shimenawa rope sagging between pillars
    for x in range(11, 37):
        u = (x - 11) / 25
        y = 17 + round(math.sin(u * math.pi) * 3)
        c.px(x, y, "Y")
        c.px(x, y + 1, "y" if x % 2 else "Y")
    for x in (16, 23, 30):              # zigzag shide
        y = 18 + round(math.sin((x - 11) / 25 * math.pi) * 3) + 1
        for k, (dx, dy) in enumerate(((0, 1), (1, 2), (0, 3), (1, 4), (0, 5))):
            c.px(x + dx, y + dy, "P")
    # moss and wear
    for _ in range(18):
        x, y = rng.randint(0, 47), rng.randint(5, 31)
        if c.get(x, y) in "Rr":
            c.px(x, y, rng.choice("qM"))
    c.rect(7, 22, 8, 23, "M"), c.rect(39, 25, 40, 27, "M"), c.px(10, 26, "m")
    c.save("prop_c", "W1 prop C: weathered vermilion torii fragment with shimenawa", outline=True)

    # prop_d: moss boulder with a paper charm.
    c = C(24, 24, {**STONE, "P": "#f0ecdc", "p": "#c8c0b0", "R": "#d43a3a", "Y": "#c8a860"})
    c.ellipse(12, 15, 10, 8.5, "s")
    c.ellipse(11, 13.5, 8.5, 7, "S")
    c.ellipse(9, 11, 3.5, 2.5, "H")
    c.ellipse(11, 7.5, 6, 2.5, "M")
    c.ellipse(9, 7, 3, 1.4, "m")
    for x in (4, 5, 17, 18, 19):
        c.px(x, 10 + (x % 3), "M")
    c.rect(2, 21, 21, 23, "d")
    c.rect(3, 20, 20, 20, "s")
    c.rect(4, 21, 19, 21, "M")
    # ofuda stuck slightly askew
    c.poly([(12, 11), (16, 10.5), (17, 20), (13, 20.5)], "P")
    c.line(13, 12, 16, 12, "R"), c.line(14, 14, 15, 14, "R")
    c.line(14, 16, 16, 16, "R"), c.line(14.5, 18, 15.5, 18, "R")
    c.line(13, 20, 16, 20, "p")
    c.line(1, 15, 22, 15, "Y")            # thin rope around
    c.recolor("Y", "S", lambda x, y: 12 <= x <= 17)
    c.save("prop_d", "W1 prop D: moss boulder bound with rope and a paper charm", outline=True)


# ───────────────────────────── enemies ─────────────────────────────

def enemies():
    # fodder_a: Kodama Sprite.
    c = C(16, 16, {"W": "#e4f0e4", "w": "#a8c8b4", "v": "#7a9a8c", "E": "#1e2a2a", "G": "#9affd8"})
    c.rect(6, 10, 10, 14, "w")          # little body
    c.rect(5, 15, 6, 15, "w"), c.rect(9, 15, 10, 15, "w")
    c.px(4, 11, "w"), c.px(11, 11, "w"), c.px(12, 12, "w")
    c.ellipse(8.5, 6.5, 6.5, 5.5, "W")
    c.ellipse(7, 8, 5, 3.5, "W")
    c.recolor("W", "w", lambda x, y: (x - 8.5) * -0.4 + (y - 6.5) > 2.6)
    c.rect(6, 13, 9, 14, "v")
    c.rect(7, 5, 8, 6, "E"), c.rect(11, 5, 12, 6, "E")   # hollow dot eyes
    c.px(10, 9, "E")                                     # small round mouth
    c.px(3, 3, "G")                                      # wisp spark
    c.save("fodder_a", "W1 fodder A: Kodama Sprite, pale tree spirit", outline=True)

    # fodder_b: Paper Crane Shikigami.
    c = C(16, 16, {"P": "#f2eee2", "p": "#c4bed2", "q": "#8a84a6", "R": "#e0403a", "G": "#ffd27a"})
    c.poly([(7, 9), (3, 1.5), (9, 8)], "p")              # far wing
    c.poly([(3, 10), (8, 7.5), (13, 10), (8, 12.5)], "P")  # body diamond
    c.poly([(8, 12.5), (13, 10), (10, 13)], "p")         # body underside fold
    c.poly([(5, 9.5), (10, 0.5), (11.5, 9)], "P")        # near wing, sharp
    c.line(10, 1, 9, 9, "p")                             # wing crease
    c.line(12, 10, 14, 4, "P"), c.line(13, 10, 15, 5, "P")  # neck
    c.px(15, 4, "P"), c.px(15, 5, "p")                   # beaked head bending forward
    c.line(4, 10, 1, 5, "P"), c.px(1, 4, "P")            # tail
    c.px(9, 5, "R"), c.px(9, 6, "R"), c.px(10, 6, "R")   # ink seal on wing
    c.px(14, 5, "G")
    c.rect(5, 14, 11, 14, "q")                           # faint shadow
    c.save("fodder_b", "W1 fodder B: Paper Crane Shikigami", outline=True)

    # ranged: Lantern Fairy.
    c = C(16, 16, {"S": "#f4e2d8", "H": "#5ab8c8", "h": "#357a9a", "D": "#b090e0", "d": "#7a5aaa",
                   "V": "#bff0ff", "v": "#7ac0e0", "Y": "#fff2b0", "O": "#ffc24a", "E": "#203040"})
    c.poly([(1, 2), (6, 6), (2, 8)], "V")          # wings
    c.poly([(1, 9), (6, 7), (3, 12)], "v")
    c.ellipse(7.5, 5, 3.3, 3.3, "H")              # hair
    c.rect(5, 3, 6, 9, "h")
    c.rect(7, 4, 9, 7, "S")                       # face
    c.px(9, 5, "E")
    c.rect(6, 1, 9, 2, "H")
    c.px(4, 2, "h")
    c.poly([(5, 8), (10, 8), (11, 14), (4, 14)], "D")  # dress
    c.rect(5, 12, 10, 14, "d")
    c.rect(6, 15, 6, 15, "S"), c.rect(9, 15, 9, 15, "S")
    c.rect(10, 9, 12, 9, "S")                     # arm out
    c.disc(13.5, 8.5, 2.3, "O")                   # light orb
    c.disc(13.3, 8.2, 1.3, "Y")
    c.save("ranged", "W1 ranged: Lantern Fairy with a light orb", outline=True)

    # blinker: Fox-Mask Phantom.
    c = C(16, 16, {"W": "#f4f0ea", "w": "#c8c2cc", "R": "#e83a4a", "E": "#1a1020",
                   "V": "#6a5ab8", "v": "#463a8a", "u": "#2e2a62", "G": "#ff9ac0"})
    # wispy trailing body (to the left / down)
    c.poly([(7, 8), (11, 12), (8, 15), (5, 14), (2, 15), (3, 12), (0.5, 11), (4, 9)], "V")
    c.poly([(5, 12), (7, 15), (3, 14)], "v")
    c.px(1, 13, "u"), c.px(2, 14, "u"), c.px(6, 14, "u")
    # mask: rounded head, ears up, snout to the right
    c.ellipse(9, 7, 4.5, 4.2, "W")
    c.poly([(5, 4.5), (6, 0.5), (8.5, 3.5)], "W")
    c.poly([(9.5, 3.5), (12, 0.5), (12.5, 5)], "W")
    c.poly([(11, 6), (15.5, 8.2), (11, 10.5)], "W")
    c.recolor("W", "w", lambda x, y: y >= 10 or x <= 5)
    c.px(6, 2, "R"), c.px(11, 2, "R"), c.px(11, 3, "R")    # inner ears
    c.line(8, 5, 10, 6, "R")                                # brow marks
    c.rect(10, 7, 11, 7, "E")                               # slit eye
    c.px(15, 8, "E")                                        # nose
    c.line(12, 9, 14, 9, "R")
    c.px(7, 9, "R"), c.px(7, 8, "G")
    c.save("blinker", "W1 blinker: Fox-Mask Phantom", outline=True)

    # elite: Komainu Guardian (stone lion-dog, sitting, glowing eyes).
    c = C(32, 32, {"S": "#808a9c", "s": "#5c6478", "d": "#3c4256", "H": "#a8b0c0",
                   "M": "#4c7a4c", "m": "#6e9e5a", "G": "#7affd0", "g": "#2ad8a0", "W": "#ece6d6", "R": "#6a2a3a"})
    rng = random.Random(5)
    c.ellipse(10, 21, 8, 8, "S")                  # haunch
    c.ellipse(9, 23, 6, 5, "s")
    c.rect(3, 27, 14, 30, "s")                    # hind paw
    c.poly([(3, 18), (0.5, 12), (3, 13), (2, 8), (5, 12), (6, 16)], "S")   # curly tail flame
    c.disc(2.5, 9, 1.8, "s"), c.disc(1.8, 13, 1.5, "s")
    c.poly([(12, 13), (24, 12), (25, 24), (14, 26)], "S")   # chest
    c.rect(19, 18, 23, 30, "S")                   # front legs
    c.rect(15, 20, 18, 30, "s")
    c.rect(22, 18, 23, 30, "s")
    c.rect(18, 29, 25, 30, "S")                   # front paws
    c.rect(14, 29, 18, 30, "s")
    c.px(20, 30, "d"), c.px(22, 30, "d"), c.px(24, 30, "d")
    c.rect(2, 31, 26, 31, "d")                    # base shadow
    # mane: ring of curls
    for k in range(10):
        a = math.pi * (0.55 + k * 0.13)
        c.disc(19 + 7 * math.cos(a), 9 + 7.5 * math.sin(a), 2.4, "s")
    for k in range(8):
        a = math.pi * (0.62 + k * 0.15)
        c.disc(19 + 6 * math.cos(a), 9 + 6.5 * math.sin(a), 1.6, "S")
    c.ellipse(21, 8, 6, 5.5, "S")                 # head
    c.poly([(23, 6), (30.5, 7), (30.5, 13), (24, 14)], "S")   # muzzle
    c.rect(24, 11, 30, 13, "d")                   # open mouth (a-gyo)
    c.rect(25, 12, 29, 12, "R")
    c.px(25, 11, "W"), c.px(28, 11, "W"), c.px(26, 13, "W"), c.px(29, 13, "W")
    c.rect(28, 7, 30, 8, "H")                     # nose
    c.rect(22, 5, 25, 5, "d")                     # heavy brow
    c.rect(23, 6, 25, 7, "G")                     # glowing eye
    c.px(25, 6, "g")
    c.disc(17, 5, 1.6, "s")                       # ear
    c.ellipse(21, 4.5, 3, 1.2, "H")
    for x, y in ((14, 16), (8, 15), (17, 22), (5, 24)):    # carved curls on body
        c.px(x, y, "d"), c.px(x + 1, y, "d"), c.px(x + 1, y - 1, "d")
    for _ in range(26):                           # moss
        x, y = rng.randint(0, 31), rng.randint(0, 31)
        if c.get(x, y) in "Ss" and y < 28:
            c.px(x, y, "M")
            if c.get(x + 1, y) in "Ss":
                c.px(x + 1, y, "m")
    c.rect(4, 13, 8, 14, "M"), c.rect(5, 12, 7, 12, "m")
    c.rect(18, 2, 21, 2, "M")
    c.save("elite", "W1 elite: Komainu Guardian, mossy stone lion-dog", outline=True)

    # mini: Kagura Bell Maiden (ghostly lilac/teal spirit dancer).
    c = C(32, 32, {"L": "#c0a6e6", "l": "#8a6cc0", "j": "#5a448e", "T": "#5ad0c4", "t": "#2a8a92",
                   "W": "#eee8fa", "w": "#bab0d8", "S": "#f0e4ea", "h": "#d8d0f0", "e": "#4ae0d0",
                   "Y": "#ffd86a", "y": "#c09030", "R": "#e8607a", "G": "#bffff4"})
    # hair flowing back (left), long
    c.poly([(9, 3), (16, 2), (17, 9), (12, 15), (5, 20), (3, 18), (7, 12)], "h")
    c.poly([(5, 16), (7, 12), (9, 15), (4, 22), (2, 21)], "w")
    # trailing spirit tail of the robe (no feet: she floats)
    c.poly([(8, 22), (22, 22), (21, 28), (17, 31), (15, 29), (12, 31), (10, 28), (6, 30), (7, 26)], "l")
    c.poly([(9, 24), (14, 23), (12, 29), (10, 27), (7, 29)], "j")
    c.poly([(16, 24), (21, 23), (19, 28), (17, 30)], "j")
    # layered robes: white under-layer, teal mid, lilac outer
    c.poly([(10, 12), (19, 12), (23, 24), (7, 24)], "L")
    c.poly([(12, 12), (17, 12), (19, 24), (11, 24)], "T")
    c.poly([(13, 12), (16, 12), (16.5, 24), (13.5, 24)], "W")
    c.line(8, 24, 22, 24, "t")
    c.rect(11, 17, 19, 18, "t")                # sash
    c.rect(14, 17, 15, 18, "Y")
    c.line(12, 19, 8, 26, "R")                 # sash ribbon trailing
    c.line(13, 19, 10, 27, "R")
    # long sleeves: left (back) hanging, right arm raised to the bells
    c.poly([(10, 12), (12, 13), (9, 22), (4, 23), (6, 17)], "L")
    c.poly([(5, 21), (9, 21), (9, 22), (4, 23)], "T")
    c.poly([(17, 12), (20, 11), (25, 9), (26, 11), (21, 16), (18, 16)], "L")
    c.poly([(21, 15), (24, 11), (26, 12), (23, 17)], "T")
    c.rect(25, 9, 26, 10, "S")                 # hand
    # head
    c.ellipse(15, 7, 3.6, 4, "S")
    c.poly([(11, 3), (19, 3), (18, 6), (12, 7)], "h")   # bangs
    c.rect(11, 4, 12, 10, "h")
    c.line(15, 7, 17, 7, "e")                 # closed glowing eyes
    c.px(18, 7, "e")
    c.px(17, 10, "R")
    # headdress: crescent ornament with teal flowers
    c.rect(12, 1, 18, 2, "Y")
    c.px(11, 2, "y"), c.px(19, 1, "y"), c.px(15, 0, "Y")
    c.px(13, 0, "T"), c.px(17, 0, "T")
    c.line(11, 3, 9, 8, "T")                  # hair ribbon
    # kagura bell cluster on a short handle
    c.line(26, 10, 27, 4, "y")
    for bx, by in ((27, 2), (25, 3), (29, 3), (26, 5), (28, 5), (27, 7), (29, 7)):
        c.disc(bx + 0.5, by + 0.5, 1.2, "Y")
        c.px(bx, by + 1, "y")
    c.px(27, 2, "G")
    c.line(27, 9, 30, 15, "T"), c.line(28, 9, 30, 13, "R")   # streamers
    c.line(26, 11, 28, 18, "L")
    c.px(3, 6, "G"), c.px(29, 22, "G"), c.px(4, 27, "e")     # spirit motes
    c.save("mini", "W1 mini-boss: Kagura Bell Maiden, ghostly spirit dancer", outline=True)

    boss()
    final()


def catmull(pts, n):
    out = []
    for i in range(len(pts) - 1):
        p0 = pts[max(i - 1, 0)]
        p1, p2 = pts[i], pts[i + 1]
        p3 = pts[min(i + 2, len(pts) - 1)]
        for k in range(n):
            t = k / n
            t2, t3 = t * t, t * t * t
            out.append(tuple(
                0.5 * (2 * p1[j] + (-p0[j] + p2[j]) * t + (2 * p0[j] - 5 * p1[j] + 4 * p2[j] - p3[j]) * t2
                       + (-p0[j] + 3 * p1[j] - 3 * p2[j] + p3[j]) * t3)
                for j in range(2)))
    out.append(pts[-1])
    return out


def boss():
    """Thousand-Torii Serpent: a coil of linked vermilion torii segments with a stone dragon head."""
    c = C(48, 48, {"R": "#e0473a", "r": "#a02c30", "q": "#6a1a26", "B": "#24202e", "b": "#3a3448",
                   "S": "#8a92a4", "s": "#5e667a", "d": "#3c4256", "H": "#b4bccc",
                   "G": "#ffe066", "g": "#ffa030", "W": "#efe8da", "P": "#eeeadc", "Y": "#c8a860", "M": "#4c7a4c"})
    ctrl = [(3, 37), (8, 43), (20, 43.5), (32, 42.5), (41, 38), (42.5, 31), (35, 27.5), (23, 28.5), (12, 28), (6, 22), (10, 16.5), (20, 16), (27, 14.5), (31, 12)]
    path = catmull(ctrl, 10)
    n = len(path)
    rad = lambda i: 1.6 + 3.4 * min(1, i / (n * 0.45))
    # resample by arc length into evenly spaced segments
    segs, acc = [path[0]], 0.0
    for (x0, y0), (x1, y1) in zip(path, path[1:]):
        acc += math.hypot(x1 - x0, y1 - y0)
        if acc >= 2.6:
            segs.append((x1, y1))
            acc = 0.0
    m = len(segs)
    rad = lambda i: 1.4 + 3.4 * min(1, i / (m * 0.45))
    def torii_bar(i):
        x, y = segs[i]
        x2, y2 = segs[i + 1]
        tx, ty = x2 - x, y2 - y
        L = math.hypot(tx, ty) or 1
        nx, ny = -ty / L, tx / L
        e = rad(i) + 2.2
        c.line(x - nx * e, y - ny * e, x + nx * e, y + ny * e, "B", w=2)      # kasagi
        c.px(round(x - nx * e), round(y - ny * e), "b")
        bx, by = x - tx / L * 2.4, y - ty / L * 2.4
        f = rad(i) + 0.6
        c.line(bx - nx * f, by - ny * f, bx + nx * f, by + ny * f, "q")      # nuki
        c.px(round(x + tx / L * 1.2), round(y + ty / L * 1.2), "Y")          # gold plaque

    pending = []
    for i, (x, y) in enumerate(segs):
        r_ = rad(i)
        c.disc(x, y + 1, r_, "q")                # underside shade
        c.disc(x, y, r_, "r")
        c.disc(x - 0.5, y - 0.6, r_ * 0.75, "R")  # lit top
        if i % 4 == 2 and i < m - 2:
            pending.append(i)
    for i in pending:
        torii_bar(i)
    # tail tip: a little paper streamer
    c.line(3, 36, 1, 31, "P"), c.px(2, 32, "Y")
    # head (stone dragon), facing right, jaw open
    c.ellipse(36, 11, 7, 5.5, "S")
    c.poly([(36, 6), (46.5, 9), (46.5, 12), (38, 13)], "S")     # upper snout
    c.poly([(36, 14), (45, 15), (44, 18), (35, 17)], "s")       # lower jaw
    c.rect(38, 13, 45, 14, "B")                                  # mouth
    for x in (39, 42, 45):
        c.px(x, 13, "W")
    c.px(40, 14, "W"), c.px(43, 14, "W")
    c.poly([(31, 7), (25, 1), (29, 2), (34, 6)], "s")           # horns
    c.poly([(35, 6), (33, 0.5), (37, 2.5), (38, 6)], "S")
    c.px(33, 1, "H"), c.px(26, 2, "H")
    c.line(33, 12, 30, 18, "s"), c.line(31, 11, 27, 16, "d")   # whiskers/mane
    c.rect(38, 8, 41, 9, "G")                                    # glowing eye
    c.px(41, 8, "g"), c.rect(37, 7, 41, 7, "d")
    c.rect(44, 9, 45, 9, "d")                                    # nostril
    c.line(34, 11, 44, 11, "H")
    c.rect(33, 14, 35, 15, "M"), c.px(30, 9, "M"), c.px(37, 16, "M")
    # head torii crown: tiny black bar behind horns
    c.line(28, 6, 34, 4, "B", w=1)
    c.save("boss", "W1 boss: Thousand-Torii Serpent", outline=True)


def final():
    """Mitsuhime of the Veiled Moon: moon princess with a halo, veil, vast silver-blue robes, orbs."""
    c = C(48, 48, {"O": "#dfe8f8", "o": "#9aaee0", "x": "#3e4a86",              # moon halo
                   "U": "#9ab6e2", "u": "#6a86c0", "n": "#455a98", "N": "#2e3a72",  # robes
                   "W": "#f2f4fc", "w": "#c4cce6",                              # under-robe / silver
                   "h": "#d4dcf2", "k": "#9aa4cc",                              # hair
                   "V": "#a6b0e0", "v": "#7a84c0",                              # veil
                   "S": "#f6eaf0", "E": "#7af0ff",                              # skin, eye glow
                   "Y": "#f0d080", "y": "#b8943c", "B": "#bfe8ff", "b": "#5ab0e8"})
    # moon halo behind the head: pale disc with subtle maria and a darker rim
    c.disc(24, 14, 14, "O")
    c.disc(24, 14, 12.6, "o")
    c.disc(24, 14, 11.6, "x")
    for (mx, my) in ((14, 8), (34, 9), (13, 17), (35, 18), (24, 1)):
        c.px(mx, my, "O")
    # hair: long silver, spreading below the shoulders
    c.poly([(17, 8), (31, 8), (34, 24), (31, 34), (17, 34), (14, 24)], "h")
    c.poly([(15, 22), (18, 18), (18, 34), (14, 32)], "k")
    c.poly([(33, 22), (30, 18), (30, 34), (34, 32)], "k")
    # vast robes flaring to the bottom
    c.poly([(18, 20), (30, 20), (44, 46), (4, 46)], "U")
    c.poly([(10, 36), (20, 22), (14, 46), (4, 46)], "u")
    c.poly([(38, 36), (28, 22), (34, 46), (44, 46)], "u")
    c.poly([(21, 21), (27, 21), (32, 46), (16, 46)], "W")       # under-layer front panel
    c.poly([(22, 22), (26, 22), (28, 46), (20, 46)], "w")
    c.line(22, 22, 20, 46, "n"), c.line(26, 22, 28, 46, "n")
    c.rect(2, 46, 45, 47, "N")                                   # hem pooling
    for x in range(3, 45, 4):
        c.rect(x, 45, x + 1, 46, "n")
    for x in range(5, 44, 6):                                    # gold hem trim dots
        c.px(x, 44, "Y")
    c.line(5, 45, 43, 45, "y")
    # obi with crescent clasp
    c.rect(19, 27, 29, 29, "N")
    c.rect(19, 27, 29, 27, "y")
    c.disc(24, 28, 1.6, "Y"), c.px(25, 27, "N")
    # huge sleeves hanging on both sides
    c.poly([(19, 21), (14, 24), (6, 36), (3, 42), (10, 41), (16, 30), (20, 26)], "U")
    c.poly([(29, 21), (34, 24), (42, 36), (45, 42), (38, 41), (32, 30), (28, 26)], "U")
    c.poly([(3, 42), (10, 41), (11, 43), (4, 44)], "n")
    c.poly([(45, 42), (38, 41), (37, 43), (44, 44)], "n")
    c.line(6, 36, 3, 42, "u"), c.line(42, 36, 45, 42, "u")
    c.line(14, 26, 8, 38, "u"), c.line(34, 26, 40, 38, "u")
    c.line(4, 41, 10, 40, "Y"), c.line(44, 41, 38, 40, "Y")      # sleeve trim
    # collar layers
    c.poly([(20, 19), (24, 24), (28, 19)], "W")
    c.line(20, 19, 24, 24, "U"), c.line(28, 19, 24, 24, "U")
    # head and veil
    c.ellipse(24, 13, 4.2, 4.8, "S")
    c.poly([(19, 8), (29, 8), (28.5, 11), (19.5, 11)], "h")      # fringe
    c.poly([(19.5, 11), (28.5, 11), (29, 19), (24, 20.5), (19, 19)], "V")  # veil
    c.line(19.5, 11, 28.5, 11, "Y")
    c.line(24, 12, 24, 20, "v")
    for x in (20, 22, 26, 28):
        c.px(x, 19, "v")
    c.px(22, 13, "E"), c.px(23, 13, "E"), c.px(26, 13, "E"), c.px(27, 13, "E")  # glow through veil
    # moon crown: crescent on top with jewels
    c.disc(24, 5, 3, "Y")
    c.disc(25, 4.2, 2.6, "O")
    c.px(24, 2, "O")
    c.px(19, 7, "Y"), c.px(29, 7, "Y"), c.px(18, 8, "y"), c.px(30, 8, "y")
    # long hair ornaments / streamers
    c.line(18, 9, 16, 16, "Y"), c.px(16, 17, "B")
    c.line(30, 9, 32, 16, "Y"), c.px(32, 17, "B")
    # floating orbs around her
    for (ox, oy, r_) in ((5, 22, 2.6), (43, 22, 2.6), (8, 9, 2.0), (40, 9, 2.0), (2.8, 33, 1.7), (45.2, 33, 1.7)):
        c.disc(ox, oy, r_, "b")
        c.disc(ox - 0.3, oy - 0.3, r_ * 0.65, "B")
        c.px(round(ox - 1), round(oy - 1), "W")
    c.save("final", "W1 final boss: Mitsuhime of the Veiled Moon", outline=True)


def relic():
    from gen_icons import Icon
    i = Icon({"S": "#d8dee8", "s": "#9aa4b8", "d": "#626c84", "W": "#ffffff", "Y": "#f0d080", "R": "#d8434a"})
    i.rect(7, 1, 8, 2, "R")                                      # cord loop
    i.px(6, 2, "R"), i.px(9, 2, "R")
    i.poly([(5, 3.5), (11, 3.5), (13, 11), (3, 11)], "S")        # bell body
    i.circle(8, 5, 3, "S")
    i.rect(3, 10, 13, 11, "s")                                   # lip
    i.poly([(9, 4), (11, 4), (12.5, 10), (10.5, 10)], "s")
    i.px(5, 5, "W"), i.px(5, 6, "W"), i.px(4, 8, "W")
    i.circle(8, 12.6, 1.3, "d")                                  # clapper
    i.rect(7, 12, 8, 13, "d")
    # crescent moon charm hanging to the right
    i.circle(13, 13.3, 2.3, "Y")
    i.circle(14.1, 12.6, 1.9, ".")
    i.line(11, 11, 12, 12, "R")
    i.save("relic_w1", "Moonveil Bell (World 1 relic)")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    ground_tiles()
    props()
    enemies()
    relic()
    print("wrote", len(list(OUT.glob("*.px"))), "World 1 sprites to", OUT, "+ icons/relic_w1.px")


if __name__ == "__main__":
    main()
